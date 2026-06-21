//! Pcap-format and pcapng-format capture-file reader.
//!
//! [`PcapSource::from_file`] reads a `.pcap` (libpcap) or `.pcapng` file into
//! memory and exposes the link-layer type plus a `Vec<RawPacket>` of frame
//! timestamps and bytes. Format detection uses a non-destructive magic-byte
//! probe (BC-2.01.009) that peeks the first four bytes without consuming them,
//! routing to the pcapng path (`RawBlock`/block walk) or the classic-pcap
//! path (`PcapReader`) as appropriate.
//!
//! ## Snaplen-truncated captures
//!
//! Captures taken with a snap length (`tcpdump -s 96`, etc.) record packets
//! whose on-wire `orig_len` legitimately exceeds the file's `snaplen` — only
//! the first `snaplen` bytes are actually stored. `pcap-file` 2.0.0's
//! validated `next_packet()` path wrongly rejects every such record with
//! `PacketHeader orig_len > snap_len` (the real invariant is only
//! `incl_len <= snaplen`). Because snaplen-truncated captures are common in
//! real-world forensics, this reader uses the *unvalidated*
//! `next_raw_packet()` path and converts the timestamp itself. Buffer safety
//! is unaffected: `RawPcapPacket` parsing still bounds `data` to exactly the
//! captured `incl_len` bytes (an over-long `incl_len` yields a hard error).
//!
//! For very large captures the all-in-memory model is a known limitation;
//! see the technical-debt register for a streaming-reader follow-up.

use std::io::{BufRead, BufReader, Read};

use anyhow::{Context, Result, anyhow};
use pcap_file::pcap::PcapReader;
use pcap_file::{DataLink, TsResolution};

// ─── pcapng canonical constants (BC-2.01.009 / ADR-009) ────────────────────

/// SHB block_type / file magic (endian-independent 4-byte literal).
const PCAPNG_MAGIC: [u8; 4] = [0x0A, 0x0D, 0x0D, 0x0A];

/// Classic pcap magic bytes (both byte orders and both timestamp resolutions).
const CLASSIC_PCAP_MAGICS: [[u8; 4]; 4] = [
    [0xA1, 0xB2, 0xC3, 0xD4], // LE microsecond
    [0xD4, 0xC3, 0xB2, 0xA1], // BE microsecond
    [0xA1, 0xB2, 0x3C, 0x4D], // LE nanosecond
    [0x4D, 0x3C, 0xB2, 0xA1], // BE nanosecond
];

/// BOM for big-endian pcapng section (on-disk bytes 1A 2B 3C 4D).
const SHB_BOM_BIG_ENDIAN: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];

/// BOM for little-endian pcapng section (on-disk bytes 4D 3C 2B 1A).
const SHB_BOM_LITTLE_ENDIAN: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// SHB block type code (used for second-SHB detection; = PCAPNG_MAGIC as u32).
const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;

/// OPB (obsolete Packet Block) type code.
const OPB_BLOCK_TYPE: u32 = 0x0000_0002;

/// SPB (Simple Packet Block) type code (BC-2.01.013 / ADR-009 Decision 22).
const SPB_BLOCK_TYPE: u32 = 0x0000_0003;

/// NRB (Name Resolution Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const NRB_BLOCK_TYPE: u32 = 0x0000_0004;

/// ISB (Interface Statistics Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const ISB_BLOCK_TYPE: u32 = 0x0000_0005;

/// EPB (Enhanced Packet Block) type code.
const EPB_BLOCK_TYPE: u32 = 0x0000_0006;

/// SJE (Systemd Journal Export Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const SJE_BLOCK_TYPE: u32 = 0x0000_0009;

/// DSB (Decryption Secrets Block) type code — explicit skip arm; body bytes MUST NOT be logged
/// (SEC-007: DSB carries TLS key material). No named `Block` enum variant exists in
/// `pcap_file::pcapng::Block` for DSB (9-variant enum per block_common.rs:146-166);
/// match the raw type bytes directly (BC-2.01.015 AC-001 F-07 / Architecture Compliance Rule 4).
const DSB_BLOCK_TYPE: u32 = 0x0000_000A;

/// IDB (Interface Description Block) type code.
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;

/// SHB body minimum: 16 bytes (BOM:4 + major:2 + minor:2 + section_length:8).
const SHB_BODY_FIXED_BYTES: usize = 16;

/// IDB body minimum: 8 bytes (linktype:2 + reserved:2 + snaplen:4).
const IDB_BODY_FIXED_BYTES: usize = 8;

/// EPB body minimum: 20 bytes (interface_id:4 + ts_high:4 + ts_low:4 +
/// captured_len:4 + original_len:4).
///
/// Named `EPB_FIXED_OVERHEAD_BYTES` per BC-2.01.012 Inv5 / Architecture Compliance Rule 2.
const EPB_FIXED_OVERHEAD_BYTES: usize = 20;

/// SPB fixed overhead: 4 bytes (body-relative; `original_len: u32` only).
///
/// MUST NOT be confused with `EPB_FIXED_OVERHEAD_BYTES = 20`.
/// Named `SPB_FIXED_OVERHEAD_BYTES` per BC-2.01.013 AC-004b / Architecture Compliance Rule 3.
/// Minimum valid SPB btl = 12 outer + 4 body-fixed = 16 bytes total.
pub const SPB_FIXED_OVERHEAD_BYTES: usize = 4;

/// Default if_tsresol for pcapng (microseconds = 10^-6, per pcapng spec §4.4).
const DEFAULT_TSRESOL: u8 = 6;

/// Precomputed powers of 10 for base-10 if_tsresol lookup (BC-2.01.014 / VP-025 Option A).
///
/// Index `e` holds `10^e` for `e ∈ [0, 19]`. Values for `e ≥ 20` exceed u64::MAX
/// and are handled by saturating to u64::MAX at the call site.
///
/// Generated at compile time; no runtime computation.
/// Option A per BC-2.01.014 VP-025 note: keeps Kani proof bounded without #[kani::unwind].
const BASE10_POWERS: [u64; 20] = [
    1,                          // 10^0
    10,                         // 10^1
    100,                        // 10^2
    1_000,                      // 10^3
    10_000,                     // 10^4
    100_000,                    // 10^5
    1_000_000,                  // 10^6  (µs default)
    10_000_000,                 // 10^7
    100_000_000,                // 10^8
    1_000_000_000,              // 10^9  (ns)
    10_000_000_000,             // 10^10
    100_000_000_000,            // 10^11
    1_000_000_000_000,          // 10^12
    10_000_000_000_000,         // 10^13
    100_000_000_000_000,        // 10^14
    1_000_000_000_000_000,      // 10^15
    10_000_000_000_000_000,     // 10^16
    100_000_000_000_000_000,    // 10^17
    1_000_000_000_000_000_000,  // 10^18
    10_000_000_000_000_000_000, // 10^19
];

// ─── Public types ────────────────────────────────────────────────────────────

/// Per-interface metadata extracted from an IDB (BC-2.01.011 PC1/PC2/PC3).
///
/// One entry is pushed onto the interface table (`Vec<InterfaceInfo>`) for each
/// IDB parsed, in IDB encounter order (BC-2.01.011 Invariant 1 — 0-based index).
///
/// # Field constraints (BC-2.01.011 / ADR-009 rev 9)
///
/// - `linktype`  — extracted from IDB body bytes 0–1 (byte-order-corrected per
///   section endianness established by the SHB BOM).
/// - `if_tsresol` — the raw `if_tsresol` option byte (option code 9) when present;
///   defaults to `6` (10^-6 microseconds, pcapng spec default) when absent.
///   Interpretation: bit 7 == 0 → base-10 exponent `e`; bit 7 == 1 → base-2
///   exponent `e & 0x7F`. Consumed by the BC-2.01.014 timestamp-conversion helper.
///
/// # Prohibited field
///
/// `snaplen` MUST NOT be added (F-M3 / ADR-009 rev 9 Decision 21 / BC-2.01.011 PC4):
/// snaplen is read from IDB bytes 4–7 only to advance the cursor and is immediately
/// discarded — wirerust has no consumer for it this cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceInfo {
    /// Link-layer type for this interface (BC-2.01.011 PC1 / Invariant 4).
    pub linktype: DataLink,
    /// Timestamp resolution exponent byte (BC-2.01.011 PC2).
    ///
    /// Defaults to `6` (10^-6 µs) when the `if_tsresol` TLV option is absent
    /// from the IDB options region (pcapng spec §4.4 default).
    pub if_tsresol: u8,
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub timestamp_secs: u32,
    pub timestamp_usecs: u32,
    pub data: Vec<u8>,
}

/// Section endianness established once by the SHB BOM (BC-2.01.010 PC1 / Inv4).
/// Propagated to all downstream block decoders; they MUST NOT re-detect per-block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionEndianness {
    /// On-disk BOM bytes `1A 2B 3C 4D`.
    BigEndian,
    /// On-disk BOM bytes `4D 3C 2B 1A`.
    LittleEndian,
}

#[derive(Debug)]
pub struct PcapSource {
    pub packets: Vec<RawPacket>,
    pub datalink: DataLink,
    /// Total blocks entering the skip arm during pcapng block walk
    /// (any unknown block type, OPB, etc.). Always 0 for classic pcap.
    /// Populated by `from_pcap_reader`; consumed by `main.rs` for the
    /// zero-packet notice (BC-2.01.009 PC6 / Decision 19).
    pub skipped_blocks: u32,
    /// Sub-count of `skipped_blocks` that were Obsolete Packet Blocks
    /// (type `0x00000002`). `opb_skipped <= skipped_blocks` always.
    pub opb_skipped: u32,
}

// ─── SHB parse result ────────────────────────────────────────────────────────

/// Output of the pure-core SHB body decoder (BC-2.01.010 postconditions).
///
/// This type is returned by `parse_shb_body` so that the caller (block-walk
/// loop) can store `endianness` and propagate it to all downstream decoders
/// without re-detecting per-block (BC-2.01.010 Invariant 4).
#[derive(Debug, Clone, Copy)]
pub struct ShbInfo {
    /// Section endianness determined once from the BOM (BC-2.01.010 PC1).
    pub endianness: SectionEndianness,
    /// pcapng major version (must be 1; BC-2.01.010 PC2).
    pub major_version: u16,
    /// pcapng minor version (any value ≥ 0 is accepted; BC-2.01.010 PC2).
    pub minor_version: u16,
}

// ─── Pure-core helper: SHB body decode ──────────────────────────────────────

/// Decode a raw SHB body slice into [`ShbInfo`].
///
/// Called as a pure-core helper (VP-026 Kani target) and by unit tests. The
/// `body` slice is the bytes after the 12-byte outer block header (i.e.,
/// starting with the 4-byte BOM field). The integration path uses
/// `PcapNgParser` from the `pcap-file` crate for SHB framing; this function
/// provides the pure-core contract verification target.
///
/// # Error routing (BC-2.01.010 PC5)
///
/// - `body.len() < SHB_BODY_FIXED_BYTES` (< 16) → E-INP-008 (body-too-short).
/// - BOM not in canonical table → E-INP-008 (invalid BOM).
/// - `major_version != 1` → E-INP-008 (unsupported version).
/// - Crate framing rejections (btl < 12, misaligned, EOF) produce E-INP-010
///   upstream before this function is called; those cases never reach here.
///
/// # Version field byte order
///
/// Per the pcapng spec §4.1, all multi-byte fields in the SHB after the BOM
/// are encoded in the byte order established by the BOM. This function decodes
/// `major_version` and `minor_version` using the endianness determined from
/// the BOM field (BC-2.01.010 PC1 / Invariant 4).
///
/// # Panics
///
/// Never panics. All error conditions return `Err`. `unwrap()`, `expect()`,
/// `panic!()`, and `unreachable!()` are prohibited in this function
/// (BC-2.01.010 AC-005 / SEC-005). VP-026 (Kani) will verify totality.
pub fn parse_shb_body(body: &[u8]) -> Result<ShbInfo> {
    // BC-2.01.010 PC5 case (b): body too short for SHB fixed fields → E-INP-008.
    if body.len() < SHB_BODY_FIXED_BYTES {
        return Err(anyhow!(
            "SHB body too short: expected at least {} bytes for SHB fixed fields, got {} \
             (E-INP-008: body-too-short; btl in range 12..28)",
            SHB_BODY_FIXED_BYTES,
            body.len()
        ));
    }

    // Read BOM bytes (body[0..4]) — raw on-disk bytes (BC-2.01.010 PC1 canonical table).
    // SAFETY: body.len() >= 16 checked above; all index accesses are in-bounds.
    let bom: [u8; 4] = [body[0], body[1], body[2], body[3]];

    // Canonical BOM table (BC-2.01.010 PC1, single normative source):
    //   On-disk 1A 2B 3C 4D → big-endian
    //   On-disk 4D 3C 2B 1A → little-endian
    //   Any other           → E-INP-008
    let endianness = if bom == SHB_BOM_BIG_ENDIAN {
        SectionEndianness::BigEndian
    } else if bom == SHB_BOM_LITTLE_ENDIAN {
        SectionEndianness::LittleEndian
    } else {
        return Err(anyhow!(
            "SHB BOM invalid: on-disk bytes {:02X} {:02X} {:02X} {:02X} match neither \
             big-endian (1A 2B 3C 4D) nor little-endian (4D 3C 2B 1A) row of the \
             canonical BOM table (E-INP-008: invalid BOM)",
            bom[0],
            bom[1],
            bom[2],
            bom[3]
        ));
    };

    // Parse major_version and minor_version in the byte order established by the BOM.
    //
    // Per the pcapng spec §4.1, all multi-byte fields in the SHB after the BOM are
    // encoded in the section endianness. A big-endian BOM (1A 2B 3C 4D) means
    // major/minor are big-endian; a little-endian BOM (4D 3C 2B 1A) means they are
    // little-endian (BC-2.01.010 PC1 / Invariant 4).
    let major_version = match endianness {
        SectionEndianness::BigEndian => u16::from_be_bytes([body[4], body[5]]),
        SectionEndianness::LittleEndian => u16::from_le_bytes([body[4], body[5]]),
    };
    let minor_version = match endianness {
        SectionEndianness::BigEndian => u16::from_be_bytes([body[6], body[7]]),
        SectionEndianness::LittleEndian => u16::from_le_bytes([body[6], body[7]]),
    };

    // BC-2.01.010 PC2: major_version must be 1; any other value → E-INP-008.
    if major_version != 1 {
        return Err(anyhow!(
            "Unsupported pcapng major version: {major_version} (only major version 1 is \
             supported; E-INP-008: semantic failure)"
        ));
    }

    // section_length (body[8..16]) is accepted regardless of value (BC-2.01.010 PC3).
    // We do not use it for bounds checking.

    Ok(ShbInfo {
        endianness,
        major_version,
        minor_version,
    })
}

// ─── Pure-core helper: 64-bit pcapng timestamp normalization ────────────────

/// Convert a pcapng EPB 64-bit split-tick timestamp to `(ts_sec, ts_usecs)`.
///
/// This is the designated pure-core Kani proof target (VP-025 / ADR-009 Decision 4 /
/// BC-2.01.014). It is the ONLY place in wirerust that interprets the `if_tsresol`
/// exponent byte — the EPB arm calls this function and MUST NOT perform timestamp
/// arithmetic inline.
///
/// # Arguments
///
/// - `ts_high` — high 32 bits of the pcapng 64-bit tick counter (EPB body bytes 4-7).
/// - `ts_low`  — low 32 bits of the pcapng 64-bit tick counter (EPB body bytes 8-11).
/// - `if_tsresol` — raw `if_tsresol` byte from the IDB options TLV (code 9), or
///   `6u8` (the pcapng spec default for microseconds) when the option is absent.
///   Bit 7 selects the base: 0 → base-10 (`10^(e & 0x7F)` ticks/sec);
///   1 → base-2 (`2^(e & 0x7F)` ticks/sec, e clamped to [0,63]).
///
/// # Returns
///
/// `(ts_sec: u32, ts_usecs: u32)` where:
/// - `ts_sec` saturates at `u32::MAX` for post-Y2106 timestamps (BC-2.01.014 PC6).
/// - `ts_usecs` is always in `[0, 999_999]` (BC-2.01.014 Invariant 3).
///
/// # Panics
///
/// Never panics for any `(u32, u32, u8)` input. VP-025 (Kani) formally verifies
/// this totality claim over the full symbolic input space.
///
/// # Forbidden
///
/// MUST NOT call `EnhancedPacketBlock::timestamp` or any crate Duration type.
/// Signature MUST contain only Rust primitive integer types (BC-2.01.014 BC).
pub fn pcapng_timestamp_to_secs_usecs(ts_high: u32, ts_low: u32, if_tsresol: u8) -> (u32, u32) {
    // BC-2.01.014 PC1: combine split ticks into 64-bit value.
    // Safe: both operands are u64; shift is exactly 32; OR with u32 cannot overflow u64.
    let ticks: u64 = ((ts_high as u64) << 32) | (ts_low as u64);

    // BC-2.01.014 PC4: µs fast path (if_tsresol == 6 exactly, base-10, 10^6).
    // MANDATORY saturation via .min(u32::MAX as u64) — bare as u32 wraps for large ts_high (M-3).
    if if_tsresol == 6 {
        let ts_sec = (ticks / 1_000_000).min(u32::MAX as u64) as u32;
        let ts_usecs = (ticks % 1_000_000) as u32;
        return (ts_sec, ts_usecs);
    }

    let ticks_per_sec: u64 = if if_tsresol & 0x80 == 0 {
        // BC-2.01.014 PC2: base-10, e = if_tsresol & 0x7F.
        // Option A: precomputed lookup table for e ∈ [0, 19]; saturate to u64::MAX for e ≥ 20.
        // This keeps the Kani VP-025 proof bounded without #[kani::unwind] annotations.
        let e = (if_tsresol & 0x7F) as usize;
        if e < BASE10_POWERS.len() {
            BASE10_POWERS[e]
        } else {
            u64::MAX
        }
    } else {
        // BC-2.01.014 PC3: base-2, e = if_tsresol & 0x7F.
        // MANDATORY: clamp e to [0, 63] before shift — 1u64 << 64 panics with overflow-checks.
        // wirerust release profile sets overflow-checks = true (ADR-009 rev 9 / BC-2.01.014 Inv6).
        let e = (if_tsresol & 0x7F).min(63) as u32;
        // checked_shl returns None only for shift >= 64; after clamp to 63 this is unreachable.
        1u64.checked_shl(e).unwrap_or(u64::MAX)
    };

    // BC-2.01.014 PC2/PC3: compute ts_sec with mandatory saturation (PC6 / VP-025 totality).
    // ticks_per_sec >= 1 always (PC7: no division by zero).
    let ts_sec = (ticks / ticks_per_sec).min(u32::MAX as u64) as u32;

    // BC-2.01.014 PC2/PC3: compute ts_usecs via u128 intermediate to prevent overflow.
    // (ticks % ticks_per_sec) * 1_000_000 overflows u64 for base-2 e >= 43.
    let ts_usecs =
        (((ticks % ticks_per_sec) as u128 * 1_000_000u128) / ticks_per_sec as u128) as u32;

    (ts_sec, ts_usecs)
}

// ─── Pure-core helper: IDB options TLV walk ─────────────────────────────────

/// Walk the IDB options region and extract the `if_tsresol` exponent byte.
///
/// `body` is the **full IDB body slice** (everything after the 12-byte outer
/// block header), starting at byte 0 of the IDB body (`linktype u16 @0-1`).
/// The options region begins at body offset 8 (immediately after the 8-byte IDB
/// fixed fields: `linktype:2 + reserved:2 + snaplen:4`).
///
/// # Caller contract
///
/// The caller MUST have already validated `body.len() >= IDB_BODY_FIXED_BYTES`
/// (≥ 8 bytes) before calling this function. Passing a shorter slice is a
/// programming error; behavior is unspecified (the implementation may return
/// the default without reading).
///
/// # Returns
///
/// `Ok(e)` where `e` is the raw `if_tsresol` option byte:
/// - If the `if_tsresol` option (code 9) is present with `option_length == 1`,
///   returns the single value byte unchanged.
/// - If `if_tsresol` is absent (no option with code 9 found before
///   `opt_endofopt` or end-of-body), returns `DEFAULT_TSRESOL` (6).
///
/// # Errors
///
/// - `option_length` of any option exceeds the number of remaining body bytes
///   (before any read of the value or padding) → `Err` (E-INP-008).
/// - `if_tsresol` option (code 9) present with `option_length != 1`
///   → `Err` (E-INP-008; F-M5 / ADR-009 rev 9: MUST NOT silently default).
///
/// # TLV walk invariants (BC-2.01.011 PC6)
///
/// - Bounds-check `option-length` against remaining bytes BEFORE reading value
///   or padding.
/// - `opt_endofopt` (code 0) or end-of-body terminates the walk immediately.
/// - Unknown option codes are silently skipped (value + 4-byte-aligned padding
///   consumed). Exception: code 9 is not "unknown" — it receives enforcement.
/// - `if_tsoffset` (code 10) is silently skipped (Decision 21).
/// - `option_code` and `option_length` are decoded using the section endianness
///   established by the SHB BOM (BC-2.01.010 Invariant 4).
///
/// # Panics
///
/// Never panics. All error conditions return `Err`. `unwrap()`, `expect()`,
/// `panic!()`, and unchecked slice indexing are prohibited (SEC-005 /
/// BC-2.01.011 AC-001).
pub fn parse_idb_options(body: &[u8], endianness: SectionEndianness) -> Result<u8> {
    // The options region begins at body offset 8 (after the 8-byte IDB fixed fields:
    // linktype:2 + reserved:2 + snaplen:4). If the body has no bytes beyond the fixed
    // fields, there are no options → return the default.
    //
    // Caller contract: body.len() >= IDB_BODY_FIXED_BYTES (≥ 8) must hold.
    // If body is shorter than 8 bytes, the options region is empty; return default.
    let opts = if body.len() > IDB_BODY_FIXED_BYTES {
        &body[IDB_BODY_FIXED_BYTES..]
    } else {
        return Ok(DEFAULT_TSRESOL);
    };

    // Walk the TLV options region (BC-2.01.011 PC6).
    // Each TLV: option_code:u16 + option_length:u16 + value (option_length bytes)
    // + padding to next 4-byte boundary.
    // BC-2.01.010 Invariant 4: option_code and option_length MUST be decoded with
    // the section endianness established by the SHB BOM — NOT hardcoded LE.
    let mut cursor = 0usize;
    let remaining = opts;

    loop {
        // Need at least 4 bytes for the TLV header (code:2 + length:2).
        if cursor + 4 > remaining.len() {
            // End of options region without finding opt_endofopt — treat as end-of-body.
            break;
        }

        let opt_code = match endianness {
            SectionEndianness::BigEndian => {
                u16::from_be_bytes([remaining[cursor], remaining[cursor + 1]])
            }
            SectionEndianness::LittleEndian => {
                u16::from_le_bytes([remaining[cursor], remaining[cursor + 1]])
            }
        };
        let opt_len = match endianness {
            SectionEndianness::BigEndian => {
                u16::from_be_bytes([remaining[cursor + 2], remaining[cursor + 3]]) as usize
            }
            SectionEndianness::LittleEndian => {
                u16::from_le_bytes([remaining[cursor + 2], remaining[cursor + 3]]) as usize
            }
        };
        cursor += 4;

        // opt_endofopt (code 0) terminates the walk immediately.
        if opt_code == 0 {
            break;
        }

        // Bounds-check: option_length must not exceed remaining bytes BEFORE reading.
        // (BC-2.01.011 PC6 / AC-005 / SEC-005 — no OOB read)
        if cursor + opt_len > remaining.len() {
            return Err(anyhow!(
                "IDB options TLV overrun: option code {opt_code} declares length {opt_len} \
                 but only {} bytes remain in the options region \
                 (E-INP-008: malformed IDB options TLV)",
                remaining.len().saturating_sub(cursor)
            ));
        }

        // Advance cursor past value + 4-byte-aligned padding.
        let padded = (opt_len + 3) & !3;

        if opt_code == 9 {
            // if_tsresol option: MUST have option_length == 1 exactly.
            // F-M5 / ADR-009 rev 9: any other length is a malformed TLV → E-INP-008.
            // MUST NOT silently ignore or default.
            if opt_len != 1 {
                return Err(anyhow!(
                    "IDB if_tsresol option (code 9) has option_length={opt_len}, expected 1 \
                     (E-INP-008: malformed if_tsresol TLV; F-M5 / ADR-009 rev 9)"
                ));
            }
            // Extract the single-byte value and return immediately.
            // (bounds already checked above: cursor + opt_len <= remaining.len())
            return Ok(remaining[cursor]);
        }

        // Unknown option code (including if_tsoffset = 10 per Decision 21):
        // silently skip (value bytes + 4-byte-aligned padding consumed).
        cursor += padded;
    }

    // if_tsresol absent → return default (BC-2.01.011 PC2 / EC-001).
    Ok(DEFAULT_TSRESOL)
}

// ─── Pure-core helper: SPB captured-len arithmetic (BC-2.01.013 / VP-031) ───

/// Compute the SPB `captured_len` from `original_len` and the SPB body slice.
///
/// This is the VP-031 pure-core proptest target (ADR-009 Decision 22 / BC-2.01.013 AC-002).
/// It encapsulates the canonical formula so VP-031 can exercise it property-based:
///
///   `spb_data_available = body.len() - SPB_FIXED_OVERHEAD_BYTES`
///   `captured_len       = min(original_len, spb_data_available as u32)`
///
/// # Precondition
///
/// `body.len() >= SPB_FIXED_OVERHEAD_BYTES` (≥ 4). The caller (SPB arm) MUST have
/// already checked this and returned `Err` if not. Passing a shorter slice produces
/// a saturating result via `saturating_sub`; the body-decode path guards this ahead
/// of the call (BC-2.01.013 AC-004a).
///
/// # Returns
///
/// `captured_len: u32` in `[0, min(original_len, body.len()-4)]`.
///
/// The bare `body.len()` (WITHOUT subtracting 4) MUST NOT be used — it is 4 bytes
/// too large because it counts the `original_len` field itself (ADR-009 Decision 22
/// Inv2 / Architecture Compliance Rule 2 / BC-2.01.013 AC-002).
///
/// # Panics
///
/// Never panics. Uses `saturating_sub`; no overflow possible on u32 min.
/// VP-031 (proptest) formally verifies this over arbitrary `(u32, Vec<u8>)` inputs.
pub fn spb_captured_len(original_len: u32, body: &[u8]) -> u32 {
    // ADR-009 Decision 22 / BC-2.01.013 AC-002 / Architecture Compliance Rule 2:
    //   spb_data_available = body.len() - SPB_FIXED_OVERHEAD_BYTES  (canonical symbol)
    //   captured_len       = min(original_len, spb_data_available)
    //
    // saturating_sub: if body.len() < SPB_FIXED_OVERHEAD_BYTES the caller guards this
    // with an Err before calling here; saturating_sub yields 0 as a safe fallback.
    // The bare body.len() (WITHOUT subtracting 4) MUST NOT be used — it is 4 bytes
    // too large because it counts the original_len field itself.
    let spb_data_available = (body.len().saturating_sub(SPB_FIXED_OVERHEAD_BYTES)) as u32;
    original_len.min(spb_data_available)
}

// ─── PcapSource impl ─────────────────────────────────────────────────────────

impl PcapSource {
    /// Read a pcap or pcapng capture from any `Read` source.
    ///
    /// Internally wraps `R` in a `BufReader` and peeks the first four bytes
    /// via `BufReader::fill_buf()` WITHOUT calling `consume()` — the byte at
    /// offset 0 is still the next readable byte after the probe (BC-2.01.009
    /// PC3 / Invariant 1 / AC-007).
    ///
    /// Routing:
    /// - `[0x0A, 0x0D, 0x0D, 0x0A]` → pcapng path (BC-2.01.009 PC1)
    /// - classic-pcap magic → classic-pcap path (BC-2.01.009 PC2)
    /// - anything else → `Err` with unrecognized-magic context (BC-2.01.009 PC4)
    pub fn from_pcap_reader<R: Read>(reader: R) -> Result<Self> {
        // Wrap the caller's reader in BufReader so fill_buf() peek is available.
        // AC-007: this function owns the wrap; caller need not do it.
        let mut buf_reader = BufReader::new(reader);

        // Peek 4 bytes without consuming (BC-2.01.009 PC3).
        // fill_buf() fills the internal buffer and returns a reference to it.
        // We read from the slice WITHOUT calling consume() — stream position is
        // unchanged after this block.
        let magic: [u8; 4] = {
            let filled = buf_reader
                .fill_buf()
                .context("Failed to read pcap magic bytes")?;
            if filled.len() < 4 {
                return Err(anyhow!(
                    "stream too short: expected at least 4 bytes for pcap magic, got {}",
                    filled.len()
                ));
            }
            [filled[0], filled[1], filled[2], filled[3]]
        };
        // NOTE: fill_buf() was NOT followed by consume() — byte 0 is still next.

        if magic == PCAPNG_MAGIC {
            // ── pcapng branch ─────────────────────────────────────────────
            // ADR-009 Decision 1: use PcapNgParser raw-block path (pcap-file 2.0.0).
            // ADR-009 Decision 13: all-in-memory model.
            // The BufReader still has byte 0 unconsumed; collect the full stream
            // (including the already-peeked 4 bytes) into memory before parsing.
            let mut raw = Vec::new();
            buf_reader
                .read_to_end(&mut raw)
                .context("Failed to read pcapng stream")?;
            Self::read_pcapng_crate(&raw)
        } else if CLASSIC_PCAP_MAGICS.contains(&magic) {
            // ── classic-pcap branch (unchanged) ───────────────────────────
            // The existing implementation path; structurally unchanged after the
            // probe insertion above. The BufReader has byte 0 unconsumed.
            let mut pcap_reader =
                PcapReader::new(buf_reader).context("Failed to parse pcap header")?;

            let header = pcap_reader.header();
            let datalink = header.datalink;
            match datalink {
                DataLink::ETHERNET
                | DataLink::RAW
                | DataLink::IPV4
                | DataLink::IPV6
                | DataLink::LINUX_SLL => {}
                other => {
                    return Err(anyhow!(
                        "Unsupported pcap link type: {other:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"
                    ));
                }
            }

            let ts_resolution = header.ts_resolution;
            let mut packets = Vec::new();

            while let Some(raw_packet) = pcap_reader.next_raw_packet() {
                let raw_packet = raw_packet.context("Failed to read packet")?;
                let timestamp_usecs = match ts_resolution {
                    TsResolution::MicroSecond => raw_packet.ts_frac,
                    TsResolution::NanoSecond => raw_packet.ts_frac / 1_000,
                };
                packets.push(RawPacket {
                    timestamp_secs: raw_packet.ts_sec,
                    timestamp_usecs,
                    data: raw_packet.data.into_owned(),
                });
            }

            Ok(PcapSource {
                packets,
                datalink,
                skipped_blocks: 0,
                opb_skipped: 0,
            })
        } else {
            Err(anyhow!(
                "unrecognized pcap magic: {:02X} {:02X} {:02X} {:02X}",
                magic[0],
                magic[1],
                magic[2],
                magic[3]
            ))
        }
    }

    /// pcapng parse path using `pcap-file` 2.0.0's `PcapNgParser` API.
    ///
    /// ADR-009 rev 9 Decision 1 — `PcapNgParser::new` + `next_raw_block` path.
    /// ADR-009 rev 9 Decision 13 — all-in-memory model (`raw` is a fully-buffered slice).
    ///
    /// ## Error taxonomy (H-2)
    ///
    /// - `PcapNgParser::new` `IncompleteBuffer` → E-INP-010 (framing: btl<12,
    ///   misaligned, EOF-before-trailer).
    /// - `PcapNgParser::new` `InvalidField("SectionHeaderBlock: block length < 16")`
    ///   → E-INP-008 (wirerust body-decode: body too short).
    /// - `PcapNgParser::new` `InvalidField("SectionHeaderBlock: invalid magic number")`
    ///   → E-INP-008 (wirerust body-decode: invalid BOM).
    /// - All other `PcapNgParser::new` errors → E-INP-010.
    /// - `next_raw_block` errors → E-INP-010.
    /// - `major_version != 1` → E-INP-008 (checked from `parser.section()`).
    /// - Second SHB block encountered → E-INP-012.
    /// - EPB before any IDB → E-INP-009.
    ///
    /// ## Section endianness (BC-2.01.010 Invariant 4)
    ///
    /// Section endianness is established once from `parser.section().endianness`
    /// (derived by the crate from the SHB BOM). All IDB and EPB body multi-byte
    /// fields are decoded using this endianness. The crate handles the SHB btl
    /// chicken-and-egg problem (reads btl as BE, then swap_bytes for LE sections).
    ///
    /// The slice MUST start at byte 0 of the pcapng stream. The probe has already
    /// confirmed the leading 4 bytes are PCAPNG_MAGIC.
    fn read_pcapng_crate(raw: &[u8]) -> Result<PcapSource> {
        use pcap_file::pcapng::PcapNgParser;
        use pcap_file::{Endianness, PcapError};

        // ── Parse the SHB via pcap-file 2.0.0 PcapNgParser ──────────────────
        //
        // PcapNgParser::new reads and validates the SHB block, detecting BOM
        // endianness and returning (rem, parser). On error, map to the wirerust
        // error taxonomy (H-2):
        //   IncompleteBuffer                    → E-INP-010 (framing)
        //   InvalidField("block length < 16")   → E-INP-008 (SHB body too short)
        //   InvalidField("invalid magic number") → E-INP-008 (invalid BOM)
        //   other InvalidField / IoError         → E-INP-010
        // Error taxonomy (H-2):
        //   InvalidField("block length < 16") → SHB body too short (body was reachable
        //     but under 16 bytes) → E-INP-008 (wirerust body-decode failure).
        //   All other crate errors (IncompleteBuffer, other InvalidField, IoError) →
        //     framing-level rejections → E-INP-010 (crate-fired provenance).
        let (mut src, mut parser) = PcapNgParser::new(raw).map_err(|e| match &e {
            PcapError::InvalidField(msg) if msg.contains("block length < 16") => {
                anyhow!("pcapng SHB body too short: {e} (E-INP-008: SHB body decode failure)")
            }
            PcapError::InvalidField(msg) if msg.contains("invalid magic number") => {
                anyhow!(
                    "pcapng SHB invalid BOM (unrecognized byte-order mark): {e} \
                     (E-INP-008: invalid BOM)"
                )
            }
            _ => anyhow!("pcapng SHB parse failed: {e} (E-INP-010: crate framing rejection)"),
        })?;

        // ── Validate major version (BC-2.01.010 PC2) ─────────────────────────
        let major_version = parser.section().major_version;
        if major_version != 1 {
            return Err(anyhow!(
                "Unsupported pcapng major version: {major_version} (only major version 1 is \
                 supported; E-INP-008: semantic failure)"
            ));
        }

        // ── Derive section endianness from SHB BOM (BC-2.01.010 Inv 4) ──────
        //
        // The crate decodes the BOM from the SHB body and stores it in
        // `parser.section().endianness`. All subsequent block body multi-byte fields
        // MUST be decoded using this endianness (never re-detected per-block).
        let section_endianness = match parser.section().endianness {
            Endianness::Big => SectionEndianness::BigEndian,
            Endianness::Little => SectionEndianness::LittleEndian,
        };

        // ── Walk subsequent blocks ────────────────────────────────────────────
        // BC-2.01.010 Invariant 4: section_endianness propagated to all decoders.

        let mut packets = Vec::new();
        // BC-2.01.011 AC-002: interface table MUST be Vec<InterfaceInfo>, NOT HashMap.
        // Interface indexes are 0-based and assigned in IDB encounter order (Invariant 1).
        let mut interfaces: Vec<InterfaceInfo> = Vec::new();
        // E-INP-013 position check (Decision 15 / AC-004): track emitted packet count.
        let mut packets_emitted: u32 = 0;
        let mut skipped_blocks: u32 = 0;
        let mut opb_skipped: u32 = 0;
        let mut block_seq: u32 = 1; // SHB was block #1

        while !src.is_empty() {
            let prev_len = src.len();
            let (rem, raw_block) = parser.next_raw_block(src).map_err(|e| {
                // The crate parses IDB blocks inside next_raw_block_inner (via try_into_block)
                // to maintain its internal interfaces list. This means the crate's IDB parser
                // runs before wirerust's own body checks. Two IDB errors from the crate must
                // be remapped from E-INP-010 to E-INP-008 (ADR-009 Decision 20):
                //
                //   - "block length < 8"  — IDB body too short (wirerust's body-decode window:
                //     12 ≤ btl < 20; crate frames the block but body < 8 IDB fixed-field bytes).
                //     Per Decision 20 Tier 2: body-decode failure → E-INP-008.
                //   - "reserved != 0"     — crate-enforced structural IDB error; wirerust remaps
                //     to E-INP-008 (ADR-009 Decision 24).
                //     Per Decision 20 Tier 2: structural body-decode failure → E-INP-008.
                //
                // All other crate errors are Tier 1 framing rejections → E-INP-010.
                //
                // STRING-COUPLING NOTE (ADR-009 Decision 23 precedent / H-3):
                // The "reserved != 0" check below is a deliberate string-coupling on the pcap-file
                // crate's error message (PcapError::InvalidField "InterfaceDescriptionBlock:
                // reserved != 0", defined in interface_description.rs:47-49).
                //
                // Empirical finding (2026-06-20): next_raw_block calls try_into_block for IDB
                // blocks internally (parser.rs:104). try_into_block invokes
                // InterfaceDescriptionBlock::from_slice which checks reserved != 0 and returns
                // Err BEFORE returning the RawBlock. Wirerust never receives the RawBlock body
                // for a reserved!=0 IDB — the crate fully validates and rejects before returning
                // to wirerust. A wirerust-side reserved pre-check is therefore impossible on the
                // next_raw_block API surface (the body is inaccessible from the Err path).
                //
                // The IDB reserved/length validation is crate-enforced inside next_raw_block;
                // wirerust remaps the InvalidField error to E-INP-008 per ADR-009 Decision 24
                // (rev 11). BC-2.01.011 v1.8 documents this as crate-enforced delegation, not
                // mirroring. The string-coupling on "reserved != 0" is load-bearing and is
                // guarded by test_BC_2_01_011_nonzero_reserved_e_inp_008 (asserts E-INP-008
                // present and E-INP-010 absent); a crate message change will cause that test to
                // catch the regression.
                let msg = e.to_string();
                if msg.contains("block length < 8") {
                    anyhow!(
                        "pcapng IDB body too short: body < 8 IDB fixed-field bytes \
                         (E-INP-008: body-too-short; constructible window 12 ≤ btl < 20)"
                    )
                } else if msg.contains("reserved != 0") {
                    anyhow!(
                        "pcapng IDB reserved field is non-zero (structural IDB error) \
                         (E-INP-008: pcapng IDB reserved field must be zero)"
                    )
                } else {
                    anyhow!("pcapng block framing error: {e} (E-INP-010: crate framing rejection)")
                }
            })?;
            src = rem;
            // Forward-progress guard (CWE-835 / ADR-009 Decision 8): if the crate
            // returned Ok but consumed zero bytes, the loop would spin forever.
            // This is a defensive guard — no known pcap-file 2.0.0 version triggers it on
            // well-formed input; it fires only if a future crate regression produces a
            // zero-advance Ok. Treat as a framing anomaly (E-INP-010 framing layer).
            if src.len() >= prev_len {
                return Err(anyhow!(
                    "pcapng block walk stalled: no forward progress at block #{block_seq} \
                     (rem={} bytes, prev={} bytes) \
                     (E-INP-010: framing anomaly, zero-advance guard)",
                    src.len(),
                    prev_len
                ));
            }
            block_seq = block_seq.saturating_add(1);

            match raw_block.type_ {
                SHB_BLOCK_TYPE => {
                    // BC-2.01.010 AC-002 / ADR-009 Decision 7: second SHB → E-INP-012.
                    return Err(anyhow!(
                        "pcapng multi-section files are not supported (second Section Header \
                         Block at block #{block_seq}) \
                         (hint: split the capture into single-section files, or re-save with \
                         'mergecap -w out.pcapng <file>' or 'editcap' which emit single-section \
                         pcapng) (E-INP-012)"
                    ));
                }

                IDB_BLOCK_TYPE => {
                    // BC-2.01.011 / BC-2.01.016 / BC-2.01.018 / ADR-009 Decision 17.
                    //
                    // THREE-LEVEL PRECEDENCE — apply in EXACT order (Decision 17):
                    //   1. E-INP-013 position check FIRST (body NOT decoded if fires)
                    //   2. E-INP-001 whitelist check SECOND
                    //   3. E-INP-011 conflict check THIRD
                    //
                    // CHECK 1 — E-INP-013: IDB after first packet block (Decision 15 / AC-004).
                    // `packets_emitted > 0` means a packet block has already been emitted.
                    // The IDB body is NOT decoded; interface table NOT updated.
                    if packets_emitted > 0 {
                        return Err(anyhow!(
                            "pcapng interface description block after first packet block — \
                             unsupported ordering (E-INP-013)"
                        ));
                    }

                    // Now decode the IDB body (body-length check is wirerust's responsibility
                    // on the raw path — M-1 / BC-2.01.011 AC-007 Architecture Anchor).
                    let blk_body = raw_block.body.as_ref();
                    if blk_body.len() < IDB_BODY_FIXED_BYTES {
                        return Err(anyhow!(
                            "pcapng IDB body too short: expected at least {} bytes, got {} \
                             (E-INP-008: body-too-short)",
                            IDB_BODY_FIXED_BYTES,
                            blk_body.len()
                        ));
                    }

                    // Decode linktype (body[0..2]) using section endianness.
                    let link_raw = match section_endianness {
                        SectionEndianness::BigEndian => {
                            u16::from_be_bytes([blk_body[0], blk_body[1]])
                        }
                        SectionEndianness::LittleEndian => {
                            u16::from_le_bytes([blk_body[0], blk_body[1]])
                        }
                    };

                    // CHECK 2 — E-INP-001: whitelist check (BC-2.01.016 / Decision 17 check 2).
                    let new_dl = DataLink::from(u32::from(link_raw));
                    match new_dl {
                        DataLink::ETHERNET
                        | DataLink::RAW
                        | DataLink::IPV4
                        | DataLink::IPV6
                        | DataLink::LINUX_SLL => {}
                        other => {
                            return Err(anyhow!(
                                "Unsupported pcap link type: {other:?}. Supported: Ethernet \
                                 (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"
                            ));
                        }
                    }

                    // CHECK 3 — E-INP-011: multi-IDB linktype agreement (BC-2.01.018 / Decision 17
                    // check 3). Compare against the first registered interface's linktype.
                    // Lazy check: first mismatch fires immediately (BC-2.01.018 PC4).
                    // Message format per error-taxonomy E-INP-011 and BC-2.01.018 AC-001(b):
                    //   "pcapng multi-interface link-type conflict: interface 0 has {first:?},
                    //    interface {n} has {other:?} (hint: ...tcpdump -i any...)"
                    if !interfaces.is_empty() && interfaces[0].linktype != new_dl {
                        let first = interfaces[0].linktype;
                        let n = interfaces.len(); // 0-based index of the new (conflicting) IDB
                        return Err(anyhow!(
                            "pcapng multi-interface link-type conflict: interface 0 has \
                             {first:?}, interface {n} has {new_dl:?} \
                             (hint: this commonly occurs with 'tcpdump -i any' captures that \
                             mix link types; wirerust requires a single link type per file) \
                             (E-INP-011)"
                        ));
                    }

                    // All three checks passed. Extract if_tsresol from the IDB options TLV.
                    // parse_idb_options walks body[8..] for the if_tsresol option (code 9).
                    // BC-2.01.010 Invariant 4: propagate section_endianness so option
                    // code/length are decoded with the correct byte order.
                    let if_tsresol = parse_idb_options(blk_body, section_endianness)
                        .context("IDB options TLV parse failed (E-INP-008)")?;

                    // Push to interface table (BC-2.01.011 PC3 / Invariant 1).
                    // snaplen (body[4..8]) is read-and-discarded; NOT stored (F-M3).
                    interfaces.push(InterfaceInfo {
                        linktype: new_dl,
                        if_tsresol,
                    });
                }

                EPB_BLOCK_TYPE => {
                    // BC-2.01.012 / ADR-009 Decision 2: EPB carries packet data.
                    // 5-step evaluation order per BC-2.01.012 PC9 — MUST NOT be reordered.
                    let blk_body = raw_block.body.as_ref();

                    // (i) Minimum body length gate — wirerust-owned check (M-1 / BC-2.01.012 AC-003).
                    // The crate does NOT run EnhancedPacketBlock parser on the raw path.
                    if blk_body.len() < EPB_FIXED_OVERHEAD_BYTES {
                        return Err(anyhow!(
                            "pcapng EPB body too short: expected at least {} bytes, got {} \
                             (E-INP-008: body-too-short)",
                            EPB_FIXED_OVERHEAD_BYTES,
                            blk_body.len()
                        ));
                    }

                    // (ii) Read interface_id from EPB body bytes 0-3 in section endianness.
                    // Safe: body.len() >= 20 guaranteed by step (i).
                    let interface_id = match section_endianness {
                        SectionEndianness::BigEndian => {
                            u32::from_be_bytes([blk_body[0], blk_body[1], blk_body[2], blk_body[3]])
                        }
                        SectionEndianness::LittleEndian => {
                            u32::from_le_bytes([blk_body[0], blk_body[1], blk_body[2], blk_body[3]])
                        }
                    };

                    // (iii) Interface table empty check — BEFORE any captured_len arithmetic.
                    // PC5a: empty-table → E-INP-009 with exact message (BC-2.01.012 PC5a).
                    if interfaces.is_empty() {
                        return Err(anyhow!(
                            "EPB references interface_id={interface_id} but interface table is \
                             empty — no IDB has been parsed (E-INP-009)"
                        ));
                    }

                    // (iv) OOB-on-non-empty check — E-INP-010 (DIFFERENT from empty-table E-INP-009).
                    // PC5b: interface_id >= table.len() on non-empty table → E-INP-010.
                    // SEC-005: this bounds check MUST precede every index into interfaces[].
                    if interface_id as usize >= interfaces.len() {
                        let table_size = interfaces.len();
                        return Err(anyhow!(
                            "EPB interface_id={interface_id} out of range (table size={table_size}) \
                             (E-INP-010)"
                        ));
                    }

                    // Interface lookup is now safe — interface_id is in-bounds.
                    let iface = &interfaces[interface_id as usize];

                    // (v) Read remaining EPB fixed fields + captured_len validation.
                    // Read ts_high @4-7, ts_low @8-11, captured_len @12-15, original_len @16-19.
                    let (ts_high, ts_low, captured_len, _original_len) = match section_endianness {
                        SectionEndianness::BigEndian => {
                            let ts_high = u32::from_be_bytes([
                                blk_body[4],
                                blk_body[5],
                                blk_body[6],
                                blk_body[7],
                            ]);
                            let ts_low = u32::from_be_bytes([
                                blk_body[8],
                                blk_body[9],
                                blk_body[10],
                                blk_body[11],
                            ]);
                            let cl = u32::from_be_bytes([
                                blk_body[12],
                                blk_body[13],
                                blk_body[14],
                                blk_body[15],
                            ]);
                            let ol = u32::from_be_bytes([
                                blk_body[16],
                                blk_body[17],
                                blk_body[18],
                                blk_body[19],
                            ]);
                            (ts_high, ts_low, cl, ol)
                        }
                        SectionEndianness::LittleEndian => {
                            let ts_high = u32::from_le_bytes([
                                blk_body[4],
                                blk_body[5],
                                blk_body[6],
                                blk_body[7],
                            ]);
                            let ts_low = u32::from_le_bytes([
                                blk_body[8],
                                blk_body[9],
                                blk_body[10],
                                blk_body[11],
                            ]);
                            let cl = u32::from_le_bytes([
                                blk_body[12],
                                blk_body[13],
                                blk_body[14],
                                blk_body[15],
                            ]);
                            let ol = u32::from_le_bytes([
                                blk_body[16],
                                blk_body[17],
                                blk_body[18],
                                blk_body[19],
                            ]);
                            (ts_high, ts_low, cl, ol)
                        }
                    };

                    // PC6a (unconditional bound-by-body, LIVE REACHABLE GUARD — BC-2.01.012 PC6a):
                    // captured_len can never exceed body.len() regardless of block_total_length.
                    let available = blk_body.len().saturating_sub(EPB_FIXED_OVERHEAD_BYTES);
                    if captured_len as usize > available {
                        return Err(anyhow!(
                            "pcapng EPB captured_len {captured_len} exceeds available body \
                             bytes {available} (E-INP-008: captured_len > body extent)"
                        ));
                    }

                    // PC6b (padding-aware overhead, DEFENSE-IN-DEPTH — BC-2.01.012 PC6b):
                    // Unreachable via crate-framed 4-aligned blocks (crate alignment rejection
                    // subsumes this path). Coded per BC-2.01.012 AC-002 as a safety net.
                    // pad_len(n) = (4 - n % 4) % 4
                    let pad_len = (4usize.wrapping_sub(captured_len as usize % 4)) % 4;
                    if EPB_FIXED_OVERHEAD_BYTES
                        .saturating_add(captured_len as usize)
                        .saturating_add(pad_len)
                        > blk_body.len()
                    {
                        return Err(anyhow!(
                            "pcapng EPB padding-overrun: 20 + {captured_len} + {pad_len} > {} \
                             (E-INP-008: wirerust body-decode padding overrun; defense-in-depth)",
                            blk_body.len()
                        ));
                    }

                    // Slice packet data bounded by captured_len (BC-2.01.012 PC3 / Invariant 2).
                    let packet_data = &blk_body[EPB_FIXED_OVERHEAD_BYTES
                        ..EPB_FIXED_OVERHEAD_BYTES + captured_len as usize];

                    // Timestamp routing: call the pure-core helper with per-interface if_tsresol.
                    // BC-2.01.012 PC2 / BC-2.01.014: helper owns the ticks combine and all arithmetic.
                    // MUST NOT use EnhancedPacketBlock::timestamp (ns-hardcoded; wrong for µs captures).
                    let (ts_sec, ts_usecs) =
                        pcapng_timestamp_to_secs_usecs(ts_high, ts_low, iface.if_tsresol);

                    packets.push(RawPacket {
                        timestamp_secs: ts_sec,
                        timestamp_usecs: ts_usecs,
                        data: packet_data.to_vec(),
                    });
                    // Increment packets_emitted for E-INP-013 position check (AC-005 / Decision 15).
                    packets_emitted = packets_emitted.saturating_add(1);
                }

                SPB_BLOCK_TYPE => {
                    // BC-2.01.013 / ADR-009 Decision 22: SPB carries packet data without timestamp.
                    // SPB has NO interface_id field → always uses interface 0.
                    let blk_body = raw_block.body.as_ref();

                    // (i) Empty-table guard (BC-2.01.013 AC-001 / BC-2.01.017 PC1 E-INP-009).
                    // SPB always binds to interface 0; interface table must be non-empty.
                    if interfaces.is_empty() {
                        return Err(anyhow!(
                            "pcapng Simple Packet Block encountered before any Interface \
                             Description Block: SPB encountered but interface table is empty \
                             — no IDB has been parsed (E-INP-009)"
                        ));
                    }

                    // (ii) Body-length guard (BC-2.01.013 AC-004a / ADR-009 Decision 22).
                    // The crate accepts btl=12 (body=0 bytes) as valid framing; wirerust
                    // MUST check body.len() >= SPB_FIXED_OVERHEAD_BYTES (4) itself.
                    if blk_body.len() < SPB_FIXED_OVERHEAD_BYTES {
                        return Err(anyhow!(
                            "Failed to read pcapng Simple Packet Block: body too short for \
                             SPB fixed fields: expected at least {} bytes, got {} \
                             (E-INP-008: body-too-short)",
                            SPB_FIXED_OVERHEAD_BYTES,
                            blk_body.len()
                        ));
                    }

                    // (iii) Decode original_len from body[0..4] in section endianness.
                    // Safe: body.len() >= 4 guaranteed by step (ii).
                    let original_len = match section_endianness {
                        SectionEndianness::BigEndian => {
                            u32::from_be_bytes([blk_body[0], blk_body[1], blk_body[2], blk_body[3]])
                        }
                        SectionEndianness::LittleEndian => {
                            u32::from_le_bytes([blk_body[0], blk_body[1], blk_body[2], blk_body[3]])
                        }
                    };

                    // (iv) Compute captured_len via pure-core helper (ADR-009 Decision 22 / VP-031).
                    // spb_data_available = body.len() - 4; captured_len = min(original_len, avail).
                    let captured_len = spb_captured_len(original_len, blk_body) as usize;

                    // (v) Slice packet data to exactly captured_len bytes (padding stripped).
                    // body layout: [original_len:4][packet_data (+ padding)].
                    // Safe: captured_len <= body.len()-4 <= body.len()-SPB_FIXED_OVERHEAD_BYTES by formula.
                    let packet_data = &blk_body
                        [SPB_FIXED_OVERHEAD_BYTES..SPB_FIXED_OVERHEAD_BYTES + captured_len];

                    // (vi) SPB has no per-packet timestamp — produce zero timestamps
                    // (BC-2.01.013 PC3 / AC-003 / zero-timestamp mandate).
                    packets.push(RawPacket {
                        timestamp_secs: 0,
                        timestamp_usecs: 0,
                        data: packet_data.to_vec(),
                    });

                    // (vii) MANDATORY: increment packets_emitted so a late IDB triggers E-INP-013
                    // (STORY-126-SPB-PACKETS-EMITTED-001 / ADR-009 Decision 15).
                    packets_emitted = packets_emitted.saturating_add(1);
                }

                OPB_BLOCK_TYPE => {
                    // BC-2.01.015 AC-001 F-07 / AC-003 / AC-007: OPB is obsolete.
                    // Skip it; increment BOTH skipped_blocks AND opb_skipped (dual-counter).
                    // OPB packet data intentionally NOT ingested (replaced by EPB).
                    skipped_blocks = skipped_blocks.saturating_add(1);
                    opb_skipped = opb_skipped.saturating_add(1);
                }

                NRB_BLOCK_TYPE => {
                    // BC-2.01.015 AC-001 F-07: NRB (Name Resolution Block) — explicit skip arm.
                    // Increments skipped_blocks only (no sub-counter).
                    // No diagnostic output at any log level (AC-008 / SEC-007).
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }

                ISB_BLOCK_TYPE => {
                    // BC-2.01.015 AC-001 F-07: ISB (Interface Statistics Block) — explicit skip arm.
                    // Increments skipped_blocks only.
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }

                SJE_BLOCK_TYPE => {
                    // BC-2.01.015 AC-001 F-07: SJE (Systemd Journal Export Block) — explicit skip arm.
                    // Increments skipped_blocks only.
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }

                DSB_BLOCK_TYPE => {
                    // BC-2.01.015 AC-001 F-07 / SEC-007: DSB (Decryption Secrets Block) — explicit
                    // skip arm. Body bytes MUST NOT be logged, printed, or surfaced at any severity
                    // level — DSB carries TLS key material (SEC-007).
                    // No named Block enum variant exists for DSB; match raw type bytes directly
                    // (Architecture Compliance Rule 4 / BC-2.01.015 AC-006 note).
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }

                _ => {
                    // Deliberate documented unknown-block handler (BC-2.01.015 AC-001 F-07 catch-all).
                    // This arm handles genuinely-unknown block types not listed above.
                    // NOT a silent drop — this is the intentional explicit catch-all for unrecognized
                    // block types. Increments skipped_blocks only.
                    // No diagnostic output at any log level (AC-008).
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }
            }
        }

        // SHB-only files (no IDB) are structurally valid (BC-2.01.009 EC-010 / F-M4).
        // M-3 (architect ruling): DataLink::from(0) = NULL sentinel when no IDB seen.
        // Derive final datalink from interfaces[0].linktype (or NULL sentinel if no IDB).
        let final_datalink = interfaces
            .first()
            .map(|i| i.linktype)
            .unwrap_or(DataLink::from(0));

        Ok(PcapSource {
            packets,
            datalink: final_datalink,
            skipped_blocks,
            opb_skipped,
        })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        Self::from_pcap_reader(reader)
    }
}
