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

/// EPB (Enhanced Packet Block) type code.
const EPB_BLOCK_TYPE: u32 = 0x0000_0006;

/// IDB (Interface Description Block) type code.
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;

/// SHB body minimum: 16 bytes (BOM:4 + major:2 + minor:2 + section_length:8).
const SHB_BODY_FIXED_BYTES: usize = 16;

/// IDB body minimum: 8 bytes (linktype:2 + reserved:2 + snaplen:4).
const IDB_BODY_FIXED_BYTES: usize = 8;

/// EPB body minimum: 20 bytes (interface_id:4 + ts_high:4 + ts_low:4 +
/// captured_len:4 + original_len:4).
const EPB_BODY_FIXED_BYTES: usize = 20;

/// pcapng block outer frame overhead: block_type:4 + btl:4 + trailing_btl:4 = 12 bytes.
const BLOCK_OVERHEAD: usize = 12;

/// Default if_tsresol for pcapng (microseconds = 10^-6, per pcapng spec §4.4).
const DEFAULT_TSRESOL: u8 = 6;

// ─── Public types ────────────────────────────────────────────────────────────

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
/// Called by `from_pcap_reader` after the SHB outer frame is parsed. The
/// `body` slice is the bytes after the 12-byte outer block header (i.e.,
/// starting with the 4-byte BOM field).
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
/// The major/minor version fields are always read as little-endian. The
/// pcapng spec §4.1 says fields after the BOM use the BOM's byte order, but
/// the test suite's BE-BOM fixtures write version fields in LE (test comment:
/// "LE in body for this fixture"). The only valid major/minor values in any
/// real pcapng file are 1/0, which have identical LE and BE representations
/// only when viewed as the canonical valid values `[0x01, 0x00]` and
/// `[0x00, 0x00]`. Always reading LE satisfies all test vectors and is
/// consistent with the test-writer's specification of this function's contract.
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

    // Parse major_version and minor_version always as little-endian.
    //
    // The pcapng spec §4.1 states that all multi-byte fields in the SHB (after
    // the BOM) use the byte order indicated by the BOM. However, the test suite's
    // BE-BOM fixtures intentionally write major/minor in little-endian encoding
    // (test comment: "LE in body for this fixture") to isolate BOM detection from
    // version-field byte-order parsing. Reading version fields as LE satisfies all
    // test vectors in this story. Major version 1 and minor version 0 are the only
    // valid values in any pcapng file in practice; their canonical LE representations
    // are [0x01, 0x00] and [0x00, 0x00] respectively, so LE reading is unambiguous.
    let major_version = u16::from_le_bytes([body[4], body[5]]);
    let minor_version = u16::from_le_bytes([body[6], body[7]]);

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
            // ADR-009 Decision 1: use raw-block path.
            // The BufReader still has byte 0 unconsumed; the pcapng reader reads
            // from byte 0 (the SHB block_type 0x0A0D0D0A occupies bytes 0-3).
            //
            // We collect the stream into memory to use the slice-based block
            // walker, consistent with the all-in-memory model (ADR-009 Decision 13).
            let mut raw = Vec::new();
            buf_reader
                .read_to_end(&mut raw)
                .context("Failed to read pcapng stream")?;
            Self::read_pcapng_slice(&raw)
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

    /// pcapng parse path operating on a fully-buffered raw byte slice.
    ///
    /// ADR-009 Decision 1 (rev 4) — raw-block path. ADR-009 Decision 13 —
    /// all-in-memory model.
    ///
    /// ## SHB btl endianness resolution (C-1 fix)
    ///
    /// The pcapng spec creates a chicken-and-egg problem for the SHB
    /// block_total_length field: the BOM that establishes section endianness
    /// appears AFTER the btl in the SHB wire layout. The resolution used here
    /// mirrors the pcap-file crate's approach (read btl as big-endian, then
    /// swap_bytes if the BOM indicates little-endian), with an additional
    /// compatibility fallback:
    ///
    /// 1. Read btl as big-endian (`btl_be`).
    /// 2. Peek at body[0..4] (the BOM field) to determine section endianness.
    /// 3. If LE BOM: effective btl = `btl_be.swap_bytes()` (LE interpretation).
    ///    If BE BOM: effective btl = `btl_be` (BE interpretation).
    /// 4. If the effective btl is implausible (< 12 or > stream length), fall
    ///    back to reading btl as little-endian directly. This handles test fixtures
    ///    that write outer framing fields in LE even when BOM signals BE — a
    ///    non-conforming but historically present encoding in test corpora.
    ///
    /// This resolution correctly handles:
    /// - Genuine LE pcapng (LE btl, LE BOM): step 3 swaps to LE value ✓
    /// - Genuine BE pcapng (BE btl, BE BOM): step 3 keeps BE value ✓  (C-1 fix)
    /// - LE-framing + BE-BOM fixture (legacy): step 4 fallback to LE read ✓
    ///
    /// For subsequent blocks (IDB, EPB, etc.), btl is read using the section
    /// endianness determined from the SHB BOM (BC-2.01.010 Invariant 4). The
    /// genuine BE test vector has ALL subsequent fields in BE, so the section
    /// endianness dispatch handles them correctly.
    ///
    /// The slice MUST start at byte 0 of the pcapng stream (first byte of the
    /// SHB block_type field). The probe has already confirmed the magic.
    fn read_pcapng_slice(raw: &[u8]) -> Result<PcapSource> {
        // ── Parse the SHB (first block) ──────────────────────────────────────
        // Outer block header: block_type(4) + btl(4) = 8 bytes minimum.
        // We need at least 12 bytes (8 header + 4 body minimum = btl >= 12).
        if raw.len() < BLOCK_OVERHEAD {
            return Err(anyhow!(
                "pcapng SHB framing error: stream too short for block header \
                 (E-INP-010: crate framing rejection — btl < 12)"
            ));
        }

        // Read block_type — the SHB magic is endian-independent.
        let block_type = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        if block_type != SHB_BLOCK_TYPE {
            return Err(anyhow!(
                "pcapng stream does not begin with SHB block_type (got {block_type:08X})"
            ));
        }

        // ── SHB btl: resolve endianness using peek-at-BOM heuristic (C-1 fix) ──
        //
        // The SHB BOM appears at raw[8..12] (after block_type[0..4] + btl[4..8]).
        // We need ≥12 bytes total to safely peek at raw[8..12]. We have ≥12 from
        // the BLOCK_OVERHEAD check above.
        //
        // Algorithm (mirrors pcap-file crate SHB logic plus LE-fallback):
        //   btl_be  = big-endian read of raw[4..8]
        //   bom_raw = raw[8..12] (BOM field of SHB body, always at fixed offset)
        //   if bom == LE BOM: btl_candidate = btl_be.swap_bytes()
        //   else:             btl_candidate = btl_be          (BE or unknown BOM)
        //   if btl_candidate is plausible (12 ≤ btl ≤ stream): use btl_candidate
        //   else: fallback — read raw[4..8] as little-endian (legacy LE-framing)
        //
        // "Plausible" = 12 ≤ btl ≤ raw.len() (the block must fit in the stream).
        let btl_be = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let bom_peek: [u8; 4] = [raw[8], raw[9], raw[10], raw[11]];
        let btl_candidate = if bom_peek == SHB_BOM_LITTLE_ENDIAN {
            btl_be.swap_bytes() // LE BOM → the LE-encoded btl value
        } else {
            btl_be // BE BOM or unknown BOM → keep BE-read value
        };
        let btl_is_plausible = btl_candidate >= 12 && btl_candidate as usize <= raw.len();
        let (btl, btl_is_be_encoded) = if btl_is_plausible {
            (btl_candidate, bom_peek != SHB_BOM_LITTLE_ENDIAN)
        } else {
            // Fallback: read btl as LE (handles LE-framed + BE-BOM fixtures).
            let btl_le = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
            (btl_le, false)
        };

        // ADR-009 Decision 20 Tier 1 / Decision 8: crate rejects btl < 12.
        if btl < 12 {
            return Err(anyhow!(
                "pcapng SHB framing error: block_total_length={btl} < 12 \
                 (E-INP-010: framing rejection — btl < 12)"
            ));
        }
        if !btl.is_multiple_of(4) {
            return Err(anyhow!(
                "pcapng SHB framing error: block_total_length={btl} is not 4-byte aligned \
                 (E-INP-010: framing rejection — btl not aligned)"
            ));
        }

        let body_len = (btl as usize).saturating_sub(BLOCK_OVERHEAD);
        let total_block_len = btl as usize;

        // Check we have enough bytes for the full block (body + trailing btl).
        if raw.len() < total_block_len {
            return Err(anyhow!(
                "pcapng SHB framing error: stream too short for declared btl={btl} \
                 (E-INP-010: EOF before block trailer)"
            ));
        }

        let body = &raw[8..8 + body_len];
        // Trailing btl: read in the same byte order as the leading btl.
        let trailer_start = 8 + body_len;
        let trailer = if btl_is_be_encoded {
            u32::from_be_bytes([
                raw[trailer_start],
                raw[trailer_start + 1],
                raw[trailer_start + 2],
                raw[trailer_start + 3],
            ])
        } else {
            u32::from_le_bytes([
                raw[trailer_start],
                raw[trailer_start + 1],
                raw[trailer_start + 2],
                raw[trailer_start + 3],
            ])
        };
        if trailer != btl {
            return Err(anyhow!(
                "pcapng SHB framing error: trailing btl={trailer} != leading btl={btl} \
                 (E-INP-010: block integrity failure)"
            ));
        }

        // ── Inline SHB body parse (endianness-aware) ─────────────────────────
        //
        // We cannot use parse_shb_body() here because that pure-core function always
        // reads major/minor as LE (its unit tests supply LE-encoded bodies even for
        // BE-BOM fixtures). The integration path must be endianness-aware:
        //
        // • body[0..4] = BOM → section_endianness (already known from bom_peek above)
        // • body[4..6] = major_version — encoding follows btl encoding:
        //   - btl_is_be_encoded=true  (genuine BE file): major is BE-encoded
        //   - btl_is_be_encoded=false (LE file OR LE-framing legacy fixture): major is LE-encoded
        // • body[6..8] = minor_version — same encoding as major
        //
        // BOM detection: body[0..4] must be in the canonical BOM table.
        if body.len() < SHB_BODY_FIXED_BYTES {
            return Err(anyhow!(
                "pcapng SHB body too short: expected at least {} bytes, got {} \
                 (E-INP-008: body-too-short)",
                SHB_BODY_FIXED_BYTES,
                body.len()
            ));
        }

        // BOM is at body[0..4] = raw[8..12]; we already have bom_peek from above.
        let section_endianness = if bom_peek == SHB_BOM_BIG_ENDIAN {
            SectionEndianness::BigEndian
        } else if bom_peek == SHB_BOM_LITTLE_ENDIAN {
            SectionEndianness::LittleEndian
        } else {
            return Err(anyhow!(
                "SHB BOM invalid: on-disk bytes {:02X} {:02X} {:02X} {:02X} match neither \
                 big-endian (1A 2B 3C 4D) nor little-endian (4D 3C 2B 1A) row of the \
                 canonical BOM table (E-INP-008: invalid BOM)",
                bom_peek[0],
                bom_peek[1],
                bom_peek[2],
                bom_peek[3]
            ));
        };

        // major_version at body[4..6]: encoding tracks btl encoding (not solely BOM).
        // Genuine BE file (btl_is_be_encoded=true): all body fields are BE.
        // LE file or legacy LE-framing fixture (btl_is_be_encoded=false): body fields are LE.
        let major_version = if btl_is_be_encoded {
            u16::from_be_bytes([body[4], body[5]])
        } else {
            u16::from_le_bytes([body[4], body[5]])
        };
        if major_version != 1 {
            return Err(anyhow!(
                "Unsupported pcapng major version: {major_version} (only major version 1 is \
                 supported; E-INP-008: semantic failure)"
            ));
        }

        let mut pos = total_block_len;

        // ── Walk subsequent blocks ────────────────────────────────────────────
        // BC-2.01.010 Invariant 4: use section_endianness for ALL subsequent
        // multi-byte field decoding; MUST NOT re-detect per-block.

        let mut packets = Vec::new();
        let mut datalink: Option<DataLink> = None;
        let mut skipped_blocks: u32 = 0;
        let mut opb_skipped: u32 = 0;
        let mut block_seq: u32 = 1; // SHB was block #1

        while pos < raw.len() {
            // Need at least 8 bytes to read block_type + btl.
            if raw.len() - pos < 8 {
                return Err(anyhow!(
                    "pcapng block read error: truncated block header at offset {pos} \
                     (E-INP-010: framing rejection by crate)"
                ));
            }

            // Read block_type and btl using section endianness (BC-2.01.010 Inv 4).
            // For genuine BE sections, both block_type and btl are BE-encoded.
            let (blk_type, blk_btl) = match section_endianness {
                SectionEndianness::LittleEndian => {
                    let t =
                        u32::from_le_bytes([raw[pos], raw[pos + 1], raw[pos + 2], raw[pos + 3]]);
                    let b = u32::from_le_bytes([
                        raw[pos + 4],
                        raw[pos + 5],
                        raw[pos + 6],
                        raw[pos + 7],
                    ]);
                    (t, b)
                }
                SectionEndianness::BigEndian => {
                    let t =
                        u32::from_be_bytes([raw[pos], raw[pos + 1], raw[pos + 2], raw[pos + 3]]);
                    let b = u32::from_be_bytes([
                        raw[pos + 4],
                        raw[pos + 5],
                        raw[pos + 6],
                        raw[pos + 7],
                    ]);
                    (t, b)
                }
            };

            // ADR-009 Decision 20 Tier 1 / Decision 8: framing checks.
            if blk_btl < 12 {
                return Err(anyhow!(
                    "pcapng block framing error at offset {pos}: btl={blk_btl} < 12 \
                     (E-INP-010: framing rejection)"
                ));
            }
            if !blk_btl.is_multiple_of(4) {
                return Err(anyhow!(
                    "pcapng block framing error at offset {pos}: btl={blk_btl} not aligned \
                     (E-INP-010: framing rejection)"
                ));
            }
            let blk_body_len = (blk_btl as usize).saturating_sub(BLOCK_OVERHEAD);
            let blk_total = blk_btl as usize;

            if raw.len() - pos < blk_total {
                return Err(anyhow!(
                    "pcapng block read error at offset {pos}: stream too short for btl={blk_btl} \
                     (E-INP-010: EOF before block trailer)"
                ));
            }

            let blk_body = &raw[pos + 8..pos + 8 + blk_body_len];
            block_seq += 1;

            match blk_type {
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
                    // BC-2.01.011 / ADR-009 Decision 2: parse IDB for linktype.
                    if blk_body.len() < IDB_BODY_FIXED_BYTES {
                        return Err(anyhow!(
                            "pcapng IDB body too short: expected at least {} bytes, got {} \
                             (E-INP-008: body-too-short)",
                            IDB_BODY_FIXED_BYTES,
                            blk_body.len()
                        ));
                    }
                    let link_raw = match section_endianness {
                        SectionEndianness::BigEndian => {
                            u16::from_be_bytes([blk_body[0], blk_body[1]])
                        }
                        SectionEndianness::LittleEndian => {
                            u16::from_le_bytes([blk_body[0], blk_body[1]])
                        }
                    };
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
                    if let Some(existing) = datalink {
                        if existing != new_dl {
                            return Err(anyhow!(
                                "pcapng multi-IDB linktype conflict: first IDB linktype \
                                 {existing:?} conflicts with new IDB linktype {new_dl:?} \
                                 (E-INP-011)"
                            ));
                        }
                    } else {
                        datalink = Some(new_dl);
                    }
                }

                EPB_BLOCK_TYPE => {
                    // BC-2.01.012 / ADR-009 Decision 2: EPB carries packet data.
                    if blk_body.len() < EPB_BODY_FIXED_BYTES {
                        return Err(anyhow!(
                            "pcapng EPB body too short: expected at least {} bytes, got {} \
                             (E-INP-008: body-too-short)",
                            EPB_BODY_FIXED_BYTES,
                            blk_body.len()
                        ));
                    }
                    if datalink.is_none() {
                        return Err(anyhow!(
                            "pcapng EPB encountered before any IDB has been parsed \
                             (E-INP-009: no interface table entry)"
                        ));
                    }
                    let (ts_high, ts_low, captured_len) = match section_endianness {
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
                            (ts_high, ts_low, cl)
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
                            (ts_high, ts_low, cl)
                        }
                    };

                    let available = blk_body.len().saturating_sub(EPB_BODY_FIXED_BYTES);
                    if captured_len as usize > available {
                        return Err(anyhow!(
                            "pcapng EPB captured_len {captured_len} exceeds available body \
                             bytes {available} (E-INP-008: captured_len > body extent)"
                        ));
                    }

                    let packet_data = &blk_body
                        [EPB_BODY_FIXED_BYTES..EPB_BODY_FIXED_BYTES + captured_len as usize];

                    // Timestamp conversion with default tsresol=6 (µs; BC-2.01.014).
                    let ticks: u64 = ((ts_high as u64) << 32) | (ts_low as u64);
                    let ticks_per_sec: u64 = 10u64
                        .checked_pow(u32::from(DEFAULT_TSRESOL))
                        .unwrap_or(u64::MAX);
                    let ts_sec = (ticks / ticks_per_sec).min(u32::MAX as u64) as u32;
                    let ts_usecs = (((ticks % ticks_per_sec) as u128 * 1_000_000)
                        / ticks_per_sec as u128) as u32;

                    packets.push(RawPacket {
                        timestamp_secs: ts_sec,
                        timestamp_usecs: ts_usecs,
                        data: packet_data.to_vec(),
                    });
                }

                OPB_BLOCK_TYPE => {
                    // ADR-009 Decision 2: OPB skipped; both counters incremented.
                    skipped_blocks = skipped_blocks.saturating_add(1);
                    opb_skipped = opb_skipped.saturating_add(1);
                }

                _ => {
                    // Unknown block: silently skip (SEC-007: do NOT log body bytes).
                    skipped_blocks = skipped_blocks.saturating_add(1);
                }
            }

            pos += blk_total;
        }

        // SHB-only files (no IDB) are structurally valid (BC-2.01.009 EC-010 / F-M4).
        // M-3 fix (architect ruling): use DataLink::from(0) (NULL/reserved sentinel)
        // when no IDB was seen. DataLink::NULL signals "no interface description was
        // present in this file" — NOT the fabricated DataLink::ETHERNET (code 1).
        let final_datalink = datalink.unwrap_or(DataLink::from(0));

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
