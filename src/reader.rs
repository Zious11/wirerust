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
        let mut datalink: Option<DataLink> = None;
        let mut skipped_blocks: u32 = 0;
        let mut opb_skipped: u32 = 0;
        let mut block_seq: u32 = 1; // SHB was block #1

        while !src.is_empty() {
            let (rem, raw_block) = parser.next_raw_block(src).map_err(|e| {
                anyhow!("pcapng block framing error: {e} (E-INP-010: crate framing rejection)")
            })?;
            src = rem;
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
                    // BC-2.01.011 / ADR-009 Decision 2: parse IDB for linktype.
                    let blk_body = raw_block.body.as_ref();
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
                    let blk_body = raw_block.body.as_ref();
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
        }

        // SHB-only files (no IDB) are structurally valid (BC-2.01.009 EC-010 / F-M4).
        // M-3 (architect ruling): DataLink::from(0) = NULL sentinel when no IDB seen.
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
