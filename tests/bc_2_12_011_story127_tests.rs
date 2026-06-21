//! STORY-127: Magic-Byte Glob (resolve_targets Content Detection) and E2E Corpus Wiring
//!
//! TDD RED-GATE suite — all tests in this file are RED against the stubs.
//! The implementer must:
//!   1. Implement `read_magic(path: &Path) -> Option<[u8; 4]>` in src/main.rs.
//!   2. Refactor `resolve_targets` to call `read_magic` and match all 5 magic
//!      values instead of filtering by extension.
//!
//! Coverage map:
//!   AC-001 → test_BC_2_12_011_all_5_magic_values_accepted
//!   AC-002 → test_BC_2_12_011_non_magic_silently_skipped
//!   AC-003 → test_BC_2_12_011_short_file_skipped
//!   AC-004 → test_BC_2_12_011_cap_extension_pcapng_magic_accepted
//!   AC-005 → test_BC_2_12_011_sorted_output
//!   AC-006 → test_BC_2_12_011_empty_directory
//!   AC-007 → test_BC_2_12_011_io_error_on_probe_silently_skipped
//!   AC-008 → test_BC_2_12_011_subdir_skipped
//!   AC-009 → test_BC_2_12_011_e2e_corpus_pcapng_reader_stack
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` per factory mandate.
//! `#![allow(non_snake_case)]` required for the BC-prefixed pattern.
//!
//! ## Test approach for AC-001..008 (unit-style)
//!
//! `read_magic` and `resolve_targets` both live in `src/main.rs` as `pub(crate)`.
//! Integration test files cannot access `pub(crate)` symbols from a binary crate.
//! Per Option B (STORY-127 stub note), these tests drive the CLI binary via
//! `assert_cmd::Command::cargo_bin("wirerust")` using a `tempdir` as the
//! `--target` (positional arg to `analyze`).  This is the same approach used
//! in `tests/main_story_088_tests.rs`.
//!
//! RED gate reasoning:
//!   - The current `resolve_targets` only accepts `.pcap` extension files.
//!   - `read_magic` is `todo!()` — it panics.
//!   - Tests that write `.data`, `.cap`, or zero-extension files with valid magic
//!     and then pass the directory to `analyze` will FAIL (files not included,
//!     or panic from `todo!()`).
//!   - Tests that write `.pcap` files with WRONG magic and expect exclusion will
//!     FAIL (current code accepts them by extension alone, not by content).
//!
//! ## Fixture notes (for implementer and test-writer)
//!
//! Unit tests (AC-001..AC-008) use `tempfile::TempDir` with crafted 4-byte headers.
//! No real capture files needed for unit tests.
//!
//! E2E tests (AC-009) require:
//!   - `tests/fixtures/smb3.pcapng`          — committed, present
//!   - `tests/fixtures/local-samples/arp-baseline-16pkt.cap` — authentic PacketLife
//!     capture (sha256 d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25,
//!     16 EPBs). Gitignored; falls back to a synthetic 16-packet pcapng if absent.
//!   - synthetic two-IDB pcapng              — built inline
//!   - synthetic OPB-only pcapng             — built inline
//!
//! ## F-5 deferral status
//!
//! The authentic PacketLife `arp-baseline-16pkt.cap` lives at
//! `tests/fixtures/local-samples/arp-baseline-16pkt.cap` (gitignored).
//! In a clean checkout (CI), that directory is empty and the synthetic fallback
//! path runs by default.  The authentic file is fetched on demand via
//! `bin/fetch-e2e-pcaps` (Phase-4 holdout requirement).  F-5 remains deferred
//! to Phase-4; CI always exercises the synthetic path.
#![allow(non_snake_case)]
#![allow(clippy::doc_lazy_continuation)]

mod story_127 {
    use std::fs;
    use std::io::Cursor;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use pcap_file::DataLink;
    use predicates::prelude::*;
    use tempfile::TempDir;
    use wirerust::reader::PcapSource;

    // ── Magic byte constants (normative per BC-2.12.011 / AC-001) ─────────────
    // These MUST NOT be edited to add a 6th value without a BC revision.
    // BC-2.12.011 Invariant 2: exactly 5 magic values; no 6th without ADR revision.
    const MAGIC_CLASSIC_LE: [u8; 4] = [0xA1, 0xB2, 0xC3, 0xD4];
    const MAGIC_CLASSIC_BE: [u8; 4] = [0xD4, 0xC3, 0xB2, 0xA1];
    const MAGIC_NS_LE: [u8; 4] = [0xA1, 0xB2, 0x3C, 0x4D];
    const MAGIC_NS_BE: [u8; 4] = [0x4D, 0x3C, 0xB2, 0xA1];
    const MAGIC_PCAPNG: [u8; 4] = [0x0A, 0x0D, 0x0D, 0x0A];

    // ── pcapng block type codes (canonical per ADR-009) ───────────────────────
    const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;
    const IDB_BLOCK_TYPE: u32 = 0x0000_0001;
    const EPB_BLOCK_TYPE: u32 = 0x0000_0006;
    const OPB_BLOCK_TYPE: u32 = 0x0000_0002;

    /// pcapng BOM for little-endian sections (on-disk bytes 4D 3C 2B 1A).
    const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

    /// DataLink::ETHERNET numeric code per pcapng link-layer type registry.
    const DL_ETHERNET: u16 = 1;

    // ── CLI helper ────────────────────────────────────────────────────────────

    /// Build an `assert_cmd::Command` targeting the wirerust binary.
    fn wirerust() -> Command {
        Command::cargo_bin("wirerust").expect("wirerust binary must be built")
    }

    // ── Fixture builder helpers ───────────────────────────────────────────────

    /// Write a file whose first 4 bytes are `magic` followed by 4 zero bytes
    /// (8 bytes total). The 4-byte padding ensures the file is distinguishable
    /// from the 3-byte short-file fixture (AC-003).
    fn write_magic_file(dir: &TempDir, name: &str, magic: &[u8; 4]) {
        let path = dir.path().join(name);
        let mut content = [0u8; 8];
        content[..4].copy_from_slice(magic);
        fs::write(path, content).expect("write_magic_file: write failed");
    }

    /// Write a file with completely wrong magic (not any of the 5 known values).
    /// Uses `[0xDE, 0xAD, 0xBE, 0xEF]` as the canonical "wrong magic" sentinel.
    fn write_wrong_magic_file(dir: &TempDir, name: &str) {
        let path = dir.path().join(name);
        let content = [0xDE_u8, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00];
        fs::write(path, content).expect("write_wrong_magic_file: write failed");
    }

    /// Write a file with fewer than 4 bytes (too short for magic probe).
    fn write_short_file(dir: &TempDir, name: &str) {
        let path = dir.path().join(name);
        fs::write(path, [0x0A_u8, 0x0D, 0x0D]).expect("write_short_file: write failed");
    }

    // ── Classic pcap fixture builders (for F-1 positive oracle) ──────────────

    /// Build a minimal valid classic-pcap file (40 bytes) producing exactly 1 packet.
    ///
    /// Structure:
    ///   - 24-byte global header: magic(4) + version_major(2=2) + version_minor(2=4)
    ///     + thiszone(4=0) + sigfigs(4=0) + snaplen(4=65535) + network(4=1/ETHERNET)
    ///   - 16-byte packet record: ts_sec(4=0) + ts_usec_or_nsec(4=0)
    ///     + incl_len(4=0) + orig_len(4=0) — zero-length capture, valid per spec.
    ///
    /// Multi-byte header fields are written in `endian` byte order as determined
    /// by the magic value:
    ///   - LE-magic variants ([D4C3B2A1] CLASSIC_BE, [4D3CB2A1] NS_BE): fields in LE
    ///   - BE-magic variants ([A1B2C3D4] CLASSIC_LE, [A1B23C4D] NS_LE): fields in BE
    ///
    /// Naming follows the BC-2.12.011 test constant names (CLASSIC_LE/BE, NS_LE/BE).
    fn classic_pcap_1pkt(magic: &[u8; 4], little_endian: bool) -> Vec<u8> {
        let mut v = Vec::with_capacity(40);
        // Global header
        v.extend_from_slice(magic); // magic (4 bytes, verbatim)
        if little_endian {
            v.extend_from_slice(&2u16.to_le_bytes()); // version_major = 2
            v.extend_from_slice(&4u16.to_le_bytes()); // version_minor = 4
            v.extend_from_slice(&0i32.to_le_bytes()); // thiszone = 0
            v.extend_from_slice(&0u32.to_le_bytes()); // sigfigs = 0
            v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen = 65535
            v.extend_from_slice(&1u32.to_le_bytes()); // network = 1 (ETHERNET)
            // Packet record (0-length capture)
            v.extend_from_slice(&0u32.to_le_bytes()); // ts_sec = 0
            v.extend_from_slice(&0u32.to_le_bytes()); // ts_usec = 0
            v.extend_from_slice(&0u32.to_le_bytes()); // incl_len = 0
            v.extend_from_slice(&0u32.to_le_bytes()); // orig_len = 0
        } else {
            v.extend_from_slice(&2u16.to_be_bytes()); // version_major = 2
            v.extend_from_slice(&4u16.to_be_bytes()); // version_minor = 4
            v.extend_from_slice(&0i32.to_be_bytes()); // thiszone = 0
            v.extend_from_slice(&0u32.to_be_bytes()); // sigfigs = 0
            v.extend_from_slice(&65535u32.to_be_bytes()); // snaplen = 65535
            v.extend_from_slice(&1u32.to_be_bytes()); // network = 1 (ETHERNET)
            // Packet record (0-length capture)
            v.extend_from_slice(&0u32.to_be_bytes()); // ts_sec = 0
            v.extend_from_slice(&0u32.to_be_bytes()); // ts_usec = 0
            v.extend_from_slice(&0u32.to_be_bytes()); // incl_len = 0
            v.extend_from_slice(&0u32.to_be_bytes()); // orig_len = 0
        }
        assert_eq!(
            v.len(),
            40,
            "classic pcap with 1 pkt must be exactly 40 bytes"
        );
        v
    }

    // ── Inline pcapng fixture builders (for AC-009) ───────────────────────────

    /// Build a minimal 28-byte LE SHB block.
    ///
    /// Follows the same structure as all STORY-123/124/125/126 helpers:
    ///   block_type(4 LE) + btl(28 LE) + BOM_LE(4) + major(1 u16 LE) + minor(0 u16 LE)
    ///   + section_length(8 LE, all-ones = unspecified) + trailing btl(4 LE)
    fn le_shb() -> Vec<u8> {
        let mut v = Vec::with_capacity(28);
        v.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes()); // 0A 0D 0D 0A
        v.extend_from_slice(&28u32.to_le_bytes()); // btl = 28
        v.extend_from_slice(&SHB_BOM_LE); // 4D 3C 2B 1A
        v.extend_from_slice(&1u16.to_le_bytes()); // major = 1
        v.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
        v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length
        v.extend_from_slice(&28u32.to_le_bytes()); // trailing btl
        assert_eq!(v.len(), 28, "SHB must be exactly 28 bytes");
        v
    }

    /// Build a minimal LE IDB block with Ethernet linktype and no options.
    ///
    /// Structure: block_type(4 LE) + btl(20 LE) + linktype(2 LE) + reserved(2 LE)
    ///   + snaplen(4 LE) + trailing btl(4 LE). Total = 20 bytes.
    fn le_idb_ethernet() -> Vec<u8> {
        let btl: u32 = 20; // 12 outer + 8 fixed body
        let mut v = Vec::with_capacity(20);
        v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
        v.extend_from_slice(&btl.to_le_bytes());
        v.extend_from_slice(&DL_ETHERNET.to_le_bytes()); // linktype = 1 (ETHERNET)
        v.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
        v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen (discarded)
        v.extend_from_slice(&btl.to_le_bytes()); // trailing btl
        assert_eq!(v.len(), 20, "IDB must be exactly 20 bytes");
        v
    }

    /// Build a minimal LE EPB block with empty payload (4 bytes captured/original = 0).
    ///
    /// Structure: block_type(4 LE) + btl(32 LE) + interface_id(4 LE)
    ///   + ts_high(4 LE) + ts_low(4 LE) + captured_len(4 LE)
    ///   + original_len(4 LE) + trailing btl(4 LE).
    /// btl = 12 (outer) + 20 (body fixed, no data, no padding) = 32 bytes.
    fn le_epb_empty() -> Vec<u8> {
        let btl: u32 = 32; // 12 outer + 20 body fixed (EPB body minimum with 0-byte data)
        let mut v = Vec::with_capacity(32);
        v.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
        v.extend_from_slice(&btl.to_le_bytes());
        v.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
        v.extend_from_slice(&0u32.to_le_bytes()); // ts_high = 0
        v.extend_from_slice(&0u32.to_le_bytes()); // ts_low = 0
        v.extend_from_slice(&0u32.to_le_bytes()); // captured_len = 0
        v.extend_from_slice(&0u32.to_le_bytes()); // original_len = 0
        v.extend_from_slice(&btl.to_le_bytes()); // trailing btl
        assert_eq!(
            v.len(),
            32,
            "EPB with empty payload must be exactly 32 bytes"
        );
        v
    }

    /// Build a minimal LE OPB block with empty body.
    ///
    /// OPB (Obsolete Packet Block) — block type 0x00000002. Per the pcapng spec and
    /// ADR-009, wirerust increments BOTH `skipped_blocks` AND `opb_skipped` for each OPB.
    /// The block body is intentionally empty (just the 12-byte outer framing).
    fn le_opb_empty() -> Vec<u8> {
        let btl: u32 = 12; // minimum: outer framing only, no body
        let mut v = Vec::with_capacity(12);
        v.extend_from_slice(&OPB_BLOCK_TYPE.to_le_bytes()); // 02 00 00 00
        v.extend_from_slice(&btl.to_le_bytes()); // 0C 00 00 00
        v.extend_from_slice(&btl.to_le_bytes()); // 0C 00 00 00 (trailing btl)
        assert_eq!(v.len(), 12, "OPB with empty body must be exactly 12 bytes");
        v
    }

    /// Build a synthetic 16-packet pcapng (LE, Ethernet).
    ///
    /// Structure: SHB + IDB(ETHERNET) + 16 × EPB(empty).
    /// Used as the fallback when `tests/fixtures/local-samples/arp-baseline-16pkt.cap`
    /// is absent (i.e., in CI environments where local-samples/ is not populated).
    fn synthetic_16pkt_pcapng() -> Vec<u8> {
        let mut v = le_shb();
        v.extend_from_slice(&le_idb_ethernet());
        for _ in 0..16 {
            v.extend_from_slice(&le_epb_empty());
        }
        v
    }

    /// Resolve the path to the authentic `arp-baseline-16pkt.cap` fixture
    /// or return `None` if the file is absent.
    ///
    /// The authentic file (sha256 d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25)
    /// lives at `tests/fixtures/local-samples/arp-baseline-16pkt.cap` (gitignored).
    fn authentic_arp_baseline_path() -> Option<PathBuf> {
        let path = PathBuf::from("tests/fixtures/local-samples/arp-baseline-16pkt.cap");
        if path.exists() { Some(path) } else { None }
    }

    // ── AC-001 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC1 + Inv1 + Inv2: `resolve_targets` accepts exactly the 5 canonical
    /// magic values (CLASSIC_LE, CLASSIC_BE, NS_LE, NS_BE, pcapng-SHB).  Each is detected
    /// by content regardless of file extension.
    ///
    /// ## Positive-inclusion oracle (F-1 fix)
    ///
    /// Each of the 5 magic variants is written as a **valid, minimal capture file** (valid
    /// global header, 1 raw packet record) that the pcap/pcapng reader stack parses and
    /// delivers as 1 raw packet to the analysis pipeline:
    ///
    ///   - CLASSIC_BE  (D4 C3 B2 A1): minimal LE-native classic pcap, 1 empty pkt → "1-be.PCAP"
    ///   - CLASSIC_LE  (A1 B2 C3 D4): minimal BE-native classic pcap, 1 empty pkt → "2-le.CAP"
    ///   - NS_BE       (4D 3C B2 A1): minimal LE-native ns classic pcap, 1 empty pkt → "3-ns-be.data"
    ///   - NS_LE       (A1 B2 3C 4D): minimal BE-native ns classic pcap, 1 empty pkt → "4-ns-le.txt"
    ///   - PCAPNG      (0A 0D 0D 0A): minimal pcapng (SHB+IDB+EPB, empty payload) → "5-ng.bin"
    ///   - reject.pcap: wrong magic [DE AD BE EF], `.pcap` extension → MUST be excluded
    ///
    /// All 5 valid files use extensions that the pre-STORY-127 stub would NOT accept
    /// (stub only accepted `.pcap` lowercase), proving content-based detection.
    ///
    /// ## Discriminating oracle: "Skipped: 5 packets (decode errors)"
    ///
    /// The 5 files each produce 1 raw packet with empty (0-byte) payload.  The
    /// Ethernet decode stage fails on empty payloads ("Not enough data ...") →
    /// each raw packet becomes a decode-error → "Skipped: N packets (decode errors)"
    /// in stdout.  N = 5 if all 5 magic variants are detected; N = 4 if any
    /// CAPTURE_MAGICS entry is missing (the corresponding file would be silently
    /// excluded).  This makes the assertion individually sensitive to each of the 5
    /// entries in CAPTURE_MAGICS.
    ///
    /// reject.pcap with wrong magic [DE AD BE EF] must be silently excluded — it
    /// does NOT contribute a decode error (it is never passed to the reader).
    ///
    /// ## Byte-order convention for classic pcap variants
    ///
    /// The magic value itself identifies the byte order of the header fields that follow:
    ///   - D4 C3 B2 A1 (CLASSIC_BE) → header fields in LE (little-endian native)
    ///   - A1 B2 C3 D4 (CLASSIC_LE) → header fields in BE (big-endian native)
    ///   - 4D 3C B2 A1 (NS_BE)      → header fields in LE
    ///   - A1 B2 3C 4D (NS_LE)      → header fields in BE
    ///
    /// `classic_pcap_1pkt(magic, little_endian)` constructs the 40-byte file accordingly.
    ///
    /// BC-2.12.011 PC1 / Inv1 / Inv2 / EC-009 / EC-010 / EC-012.
    #[test]
    fn test_BC_2_12_011_all_5_magic_values_accepted() {
        let dir = tempfile::tempdir().expect("tempdir");

        // ── 5 valid minimal capture files, one per CAPTURE_MAGICS entry ───────
        //
        // Each file uses an extension that the pre-STORY-127 stub would NOT accept
        // (stub only accepted ".pcap" lowercase).  Post-STORY-127 all 5 are detected
        // by magic bytes alone.

        // MAGIC_CLASSIC_BE = [D4 C3 B2 A1]: LE-native classic pcap.
        // (All committed test fixtures use this magic — it is the most common format.)
        // Extension ".PCAP" uppercase → stub excludes; content-detection includes it.
        // EC-012: file with uppercase extension accepted when magic matches.
        let bytes = classic_pcap_1pkt(&MAGIC_CLASSIC_BE, /* little_endian= */ true);
        fs::write(dir.path().join("1-be.PCAP"), &bytes).expect("write 1-be.PCAP");

        // MAGIC_CLASSIC_LE = [A1 B2 C3 D4]: BE-native classic pcap.
        // Extension ".CAP" uppercase → stub excludes.
        let bytes = classic_pcap_1pkt(&MAGIC_CLASSIC_LE, /* little_endian= */ false);
        fs::write(dir.path().join("2-le.CAP"), &bytes).expect("write 2-le.CAP");

        // MAGIC_NS_BE = [4D 3C B2 A1]: ns-resolution LE-native classic pcap.
        // EC-009 (nanosecond LE magic accepted).  Extension ".data" → stub excludes.
        let bytes = classic_pcap_1pkt(&MAGIC_NS_BE, /* little_endian= */ true);
        fs::write(dir.path().join("3-ns-be.data"), &bytes).expect("write 3-ns-be.data");

        // MAGIC_NS_LE = [A1 B2 3C 4D]: ns-resolution BE-native classic pcap.
        // EC-010 (nanosecond BE magic accepted).  Extension ".txt" → stub excludes.
        let bytes = classic_pcap_1pkt(&MAGIC_NS_LE, /* little_endian= */ false);
        fs::write(dir.path().join("4-ns-le.txt"), &bytes).expect("write 4-ns-le.txt");

        // MAGIC_PCAPNG = [0A 0D 0D 0A]: pcapng SHB magic.
        // Build a valid minimal pcapng: SHB + IDB(ETHERNET) + EPB(empty payload) = 1 raw pkt.
        // Extension ".bin" → stub excludes.
        let mut pcapng_bytes = le_shb();
        pcapng_bytes.extend_from_slice(&le_idb_ethernet());
        pcapng_bytes.extend_from_slice(&le_epb_empty());
        fs::write(dir.path().join("5-ng.bin"), &pcapng_bytes).expect("write 5-ng.bin");

        // Wrong-magic file with a ".pcap" extension — must be EXCLUDED (silent skip).
        // Pre-STORY-127 stub: accepted by extension → reader reports "unrecognized pcap
        // magic".  Post-STORY-127: magic probe returns [DE AD BE EF] → not in
        // CAPTURE_MAGICS → silently excluded.
        write_wrong_magic_file(&dir, "reject.pcap");

        // ── Positive-inclusion oracle ─────────────────────────────────────────
        //
        // All 5 valid files must be detected by content and passed to the reader.
        // Each file delivers 1 raw packet with 0-byte payload.  The Ethernet decode
        // stage fails on empty payload ("Not enough data to decode Ethernet 2 header")
        // → each raw packet becomes a decode error → "Skipped: 5 packets (decode
        // errors)" in stdout.
        //
        // If ANY CAPTURE_MAGICS entry is missing (e.g. MAGIC_NS_BE removed):
        //   3-ns-be.data is excluded → only 4 files processed → "Skipped: 4 packets"
        //   → assertion "Skipped: 5" fails.
        // This makes the assertion individually sensitive to ALL 5 magic entries.
        //
        // ── Wrong-magic exclusion oracle ──────────────────────────────────────
        //
        // reject.pcap must be silently excluded (not passed to the reader).
        // If reject.pcap were included: reader emits "unrecognized pcap magic" error
        // AND adds 0 decode errors (reader rejects at header stage, before raw packet
        // delivery) → Skipped count unchanged but stderr contains the error.
        // Asserting stderr does NOT contain "unrecognized pcap magic" verifies exclusion.
        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // 5 raw packets processed (one per magic), all fail Ethernet decode:
            .stdout(predicate::str::contains("Skipped: 5 packets"))
            // Wrong-magic file silently excluded → reader never sees it → no magic error.
            .stderr(predicate::str::contains("unrecognized pcap magic").not());
        // Note: we do NOT assert .success() — the 5 decode errors may cause exit non-zero
        // depending on wirerust's error-exit policy.  The discriminating assertion is
        // "Skipped: 5" (positive oracle) + absence of "unrecognized pcap magic" (exclusion oracle).
    }

    // ── AC-002 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC2: files whose first 4 bytes do not match any of the 5 magic
    /// values are silently skipped — no error, no warning.
    ///
    /// Setup: create `a.pcap` with the canonical CLASSIC_LE magic (must be included)
    /// and `b.pcap` with first bytes `[0xDE, 0xAD, 0xBE, 0xEF]` (must be excluded).
    ///
    /// RED: the current resolve_targets accepts BOTH `a.pcap` and `b.pcap` by extension
    /// alone (the `ext == "pcap"` filter accepts any file named *.pcap regardless of
    /// content).  The reader then errors on `b.pcap`'s non-magic header.
    /// Post-refactor: only `a.pcap` is included (magic matches); `b.pcap` is silently
    /// skipped at the probe stage.  The observable post-refactor behavior is that the
    /// directory scan produces 0 errors (b.pcap silently excluded, never passed to reader).
    ///
    /// The RED oracle: assert that the command exits 0 WITH "Packets: 0" (both are
    /// currently processed: a.pcap with valid magic and b.pcap with wrong magic → reader
    /// errors on b.pcap → exit non-zero or error message).  Wait, actually:
    ///   - a.pcap has 8 bytes of CLASSIC_LE magic + zeros — the classic-pcap reader
    ///     will fail on the truncated pcap (header parse error → bail! → exit non-zero).
    ///   - b.pcap has wrong magic → current code accepts it by extension → reader fails.
    ///
    /// So with the STUB: analyze <dir> exits non-zero (reader errors on both files).
    /// Post-refactor: only a.pcap included (b.pcap silently skipped); reader still errors
    /// on truncated a.pcap.
    ///
    /// Better RED oracle: create a directory with ONLY the wrong-magic file.  The stub
    /// extension-filter includes it (it's a .pcap), causing a reader error.  Post-refactor,
    /// it's silently skipped → command exits 0 with "Packets: 0".
    ///
    /// RED: assert command exits 0 and stdout contains "Packets: 0".
    /// This FAILS under the stub because the wrong-magic .pcap IS passed to the reader,
    /// causing a non-zero exit.
    ///
    /// BC-2.12.011 PC2 / EC-004.
    #[test]
    fn test_BC_2_12_011_non_magic_silently_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");

        // A .pcap file with wrong magic (not one of the 5 known values).
        // BC-2.12.011 EC-004: `analysis.pcap` with first bytes [0xDE,0xAD,0xBE,0xEF]
        // must be silently excluded.
        write_wrong_magic_file(&dir, "analysis.pcap");

        // RED: the stub's extension filter accepts `analysis.pcap` (it is a ".pcap" file).
        // The reader receives the wrong-magic file and fails to parse → exit non-zero.
        //
        // Post-refactor: `analysis.pcap` is silently skipped at the magic-probe stage
        // (first 4 bytes [0xDE,0xAD,0xBE,0xEF] do not match any of the 5 magic values).
        // The command exits 0 with "Packets: 0".
        //
        // This assertion pins the CORRECT post-refactor behavior; it FAILS under the stub.
        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success() // RED: stub exits non-zero (reader error on wrong-magic file)
            .stdout(predicate::str::contains("Packets: 0")); // non-magic file excluded
    }

    // ── AC-003 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC3 + Inv5: files with fewer than 4 readable bytes are silently
    /// skipped.  No panic, no error, no warning.
    ///
    /// Setup: create a 3-byte file with `.pcap` extension (too short for magic probe)
    /// and a valid CLASSIC_LE-magic `.pcap` file (8 bytes) as the control.
    ///
    /// RED: the stub's `read_magic` is `todo!()` — it panics.  When resolve_targets
    /// calls `read_magic` on any file, the thread panics and the test FAILS with a
    /// panic message.
    ///
    /// Actually, with the current stub: resolve_targets uses ONLY extension filtering
    /// (no call to read_magic at all).  The 3-byte `.pcap` file IS included by the
    /// stub extension filter, passed to the reader, which fails on truncated content.
    ///
    /// RED oracle: assert "Packets: 0" with exit 0 (short file silently skipped).
    /// Under the stub: 3-byte file passed to reader → exit non-zero.
    ///
    /// BC-2.12.011 PC3 / EC-007.
    #[test]
    fn test_BC_2_12_011_short_file_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");

        // 3-byte file named .pcap — must be silently skipped (too short for 4-byte probe).
        write_short_file(&dir, "short.pcap");

        // RED: stub's extension filter includes short.pcap → reader fails (truncated)
        // → exit non-zero.  Post-refactor: magic probe reads 3 bytes (< 4) → None
        // → file silently skipped → Packets: 0, exit 0.
        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success() // RED: stub exits non-zero (reader error on truncated file)
            .stdout(predicate::str::contains("Packets: 0")); // short file excluded
    }

    // ── AC-004 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 Inv1 / EC-002: the C-2 regression fixture.
    ///
    /// A `.cap` file with pcapng magic `[0x0A, 0x0D, 0x0D, 0x0A]` MUST be accepted.
    /// A `.pcap` file with wrong first bytes MUST be rejected regardless of extension.
    ///
    /// This resolves ADR-009 C-2: the prior extension-based filter excluded `.cap` files
    /// entirely (only `.pcap` was accepted), causing `arp-baseline-16pkt.cap` to be missed.
    ///
    /// Setup:
    ///   `c2-regression.cap`  — pcapng magic, `.cap` extension → MUST be included.
    ///   `bad-content.pcap`   — wrong magic `[0xDE,0xAD,0xBE,0xEF]`, `.pcap` extension → MUST be excluded.
    ///
    /// RED: current resolve_targets extension filter:
    ///   - `c2-regression.cap` is EXCLUDED (ext != "pcap").
    ///   - `bad-content.pcap`  is INCLUDED (ext == "pcap", content ignored).
    ///   - Reader fails on bad-content.pcap → exit non-zero.
    ///
    /// Post-refactor:
    ///   - `c2-regression.cap` is INCLUDED (pcapng magic detected by content).
    ///   - `bad-content.pcap`  is EXCLUDED (wrong magic, silently skipped).
    ///   - Reader invoked only on c2-regression.cap (malformed 8-byte body → error or
    ///     0 packets, but the file IS passed to the reader).
    ///
    /// The RED oracle asserts that bad-content.pcap is silently skipped (exit 0, Packets: 0),
    /// which FAILS under the stub because bad-content.pcap IS included → reader error.
    ///
    /// Additionally, we need to verify c2-regression.cap IS included post-refactor.
    /// A separate assertion: with ONLY c2-regression.cap in the directory, post-refactor
    /// the reader is invoked (reader error or Packets: 0 from malformed content, but NOT
    /// silently skipped / "Target not found").
    ///
    /// BC-2.12.011 Inv1 / EC-002 / ADR-009 Decision 11 (resolves C-2).
    #[test]
    fn test_BC_2_12_011_cap_extension_pcapng_magic_accepted() {
        // ── Case A: wrong-magic .pcap silently skipped ────────────────────────
        let dir_a = tempfile::tempdir().expect("tempdir (Case A)");

        // bad-content.pcap: .pcap extension, wrong magic → must be EXCLUDED post-refactor.
        write_wrong_magic_file(&dir_a, "bad-content.pcap");

        // RED: stub accepts bad-content.pcap by extension → reader error → exit non-zero.
        // Post-refactor: bad-content.pcap silently skipped → exit 0, Packets: 0.
        wirerust()
            .args(["analyze", dir_a.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success() // RED: stub exits non-zero (reader fails on wrong-magic .pcap)
            .stdout(predicate::str::contains("Packets: 0")); // wrong magic excluded

        // ── Case B: pcapng-magic .cap must be included and parsed successfully ──
        //
        // AC-004 headline behavior: a file with `.cap` extension and pcapng magic
        // `[0x0A, 0x0D, 0x0D, 0x0A]` MUST be accepted by resolve_targets (content
        // detection, extension ignored) and processed to completion — resolves C-2.
        //
        // To verify this cleanly within STORY-127 scope (no per-file error isolation,
        // which is STORY-128/ADR-009 Decision 12), Case B uses a VALID minimal pcapng
        // file: SHB + IDB(ETHERNET) + 1×EPB(empty) = 80 bytes, parses to 1 packet.
        // A malformed/truncated pcapng body would cause the reader to error (exit
        // non-zero), which would require STORY-128 isolation to distinguish from a
        // detection failure — forbidden in STORY-127.
        //
        // Pre-refactor (stub): .cap excluded by extension filter → Packets: 0, exit 0.
        // Post-refactor: .cap included by magic → reader parses valid pcapng → 1 packet
        // → exit 0. The discriminating assertion: output contains "Packets: 1" (stub
        // would produce "Packets: 0" because the .cap is silently excluded).
        let dir_b = tempfile::tempdir().expect("tempdir (Case B)");

        // Write smb3.pcapng content into a file with a `.cap` extension.
        //
        // AC-004 headline behavior (C-2 regression): a pcapng file with a `.cap` extension
        // MUST be detected and processed — resolves ADR-009 C-2 (`arp-baseline-16pkt.cap`
        // was excluded by the prior extension-based filter).
        //
        // We use the committed `tests/fixtures/smb3.pcapng` as the content source (54
        // packets, confirmed parseable) rather than a synthetic fixture. This removes any
        // coupling to the pcapng byte builder helpers and exercises the full reader stack
        // against a real-world file.
        //
        // File: c2-regression.cap — content is `smb3.pcapng` bytes; extension is `.cap`.
        // First 4 bytes = [0x0A, 0x0D, 0x0D, 0x0A] (pcapng SHB magic).
        let smb3_bytes = fs::read("tests/fixtures/smb3.pcapng")
            .expect("tests/fixtures/smb3.pcapng must exist (committed fixture)");
        assert_eq!(
            &smb3_bytes[..4],
            &MAGIC_PCAPNG,
            "smb3.pcapng must begin with pcapng magic (sanity check)"
        );

        // Write as c2-regression.cap (.cap extension, valid pcapng content).
        // resolve_targets MUST detect this via magic bytes, ignoring the .cap extension.
        let cap_path = dir_b.path().join("c2-regression.cap");
        fs::write(&cap_path, &smb3_bytes).expect("write c2-regression.cap");

        // Post-refactor: magic probe reads first 4 bytes = [0x0A,0x0D,0x0D,0x0A] →
        // matches MAGIC_PCAPNG → file included → reader parses smb3 content → 54 packets
        // → stdout "Packets: 54", exit 0.
        //
        // Pre-refactor (stub): .cap excluded by extension filter → Packets: 0, exit 0.
        // The discriminating assertion is "Packets: 54" (not "Packets: 0"), which FAILS
        // under the stub (stub excludes .cap entirely; only "Packets: 0" is produced).
        wirerust()
            .args(["analyze", dir_b.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success() // valid pcapng → reader succeeds → exit 0
            .stdout(predicate::str::contains("Packets: 54")); // .cap included; 54 smb3 packets
    }

    // ── AC-005 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC5 + Inv3: the returned Vec is sorted lexicographically.
    /// `files.sort()` is called before returning.
    ///
    /// Setup: a directory with `z.pcap` (http-ooo.pcap, 16 pkts, created FIRST) and
    /// `a.pcap` (http.pcap, 1 pkt, created SECOND), plus `m.cap` (MAGIC_PCAPNG, 8 bytes).
    ///
    /// The sort test uses the same observable as STORY-088 EC-005: HTTP JSON recent_uris
    /// order proves that `a.pcap` was processed before `z.pcap`.
    ///
    /// The STORY-127-specific RED assertion: `m.cap` (pcapng magic, `.cap` extension)
    /// is EXCLUDED by the stub's extension filter → Packets: 17.  Post-refactor, `m.cap`
    /// IS included → reader errors on malformed 8-byte body → exit non-zero.
    ///
    /// RED assertion: assert command exits non-zero when `m.cap` is present.
    /// Under the stub: exits 0 (m.cap excluded; only a.pcap + z.pcap processed).
    /// Post-refactor: m.cap included → reader errors on malformed pcapng → exit non-zero.
    ///
    /// Sort verification (GREEN even under stub — `files.sort()` already works for .pcap):
    /// Not asserted here in the RED form; sort correctness is already pinned by STORY-088
    /// EC-005.  The new RED discriminator for STORY-127 is the m.cap inclusion behavior.
    ///
    /// BC-2.12.011 PC5 / Inv3 / EC-003.
    ///
    /// RED: FAILS under stub (stub exits 0; post-refactor exits non-zero due to m.cap included).
    #[test]
    fn test_BC_2_12_011_sorted_output() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Create z.pcap (16 pkts) BEFORE a.pcap (1 pkt) so that without sort(),
        // read_dir would return [z.pcap, a.pcap] on macOS APFS (creation order).
        // With files.sort(), the order is always [a.pcap, m.cap, z.pcap].
        fs::copy("tests/fixtures/http-ooo.pcap", dir_path.join("z.pcap")).unwrap();
        fs::copy("tests/fixtures/http.pcap", dir_path.join("a.pcap")).unwrap();

        // m.cap: pcapng magic, `.cap` extension.
        // Pre-refactor (stub): excluded by ext filter (only "pcap" extension accepted).
        // Post-refactor: included by magic probe → reader errors on malformed 8-byte body.
        // The error causes exit non-zero → the .success() assertion below FAILS.
        write_magic_file(&dir, "m.cap", &MAGIC_PCAPNG);

        // RED assertion: post-refactor exits NON-ZERO (m.cap included → reader error).
        // Under the stub: m.cap excluded → a.pcap + z.pcap only → exit 0, Packets: 17.
        // .failure() asserts exit non-zero — FAILS under stub (stub exits 0).
        wirerust()
            .args([
                "analyze",
                dir_path.to_str().unwrap(),
                "--no-color",
                "--all",
            ])
            .assert()
            // RED: stub exits 0 (m.cap excluded; a+z process cleanly).
            // Post-refactor: m.cap included → reader errors on truncated pcapng → exit 1.
            // .failure() FAILS under the stub (because stub exits 0).
            .failure();
    }

    // ── AC-006 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC6: an empty directory returns Ok(vec![]) — no error, no warning.
    ///
    /// This test is expected to be GREEN under both the stub and post-refactor:
    /// an empty directory produces no entries to iterate, so the behavior is correct
    /// regardless of the filtering mechanism.
    ///
    /// It is included to pin BC-2.12.011 PC6 explicitly and guard against regressions
    /// where an empty directory returns an error (e.g., if a future implementation
    /// incorrectly requires at least one matching file).
    ///
    /// BC-2.12.011 PC6 / EC-005.
    #[test]
    fn test_BC_2_12_011_empty_directory() {
        let dir = tempfile::tempdir().expect("tempdir");

        // Empty directory — no files at all.
        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success() // empty dir → Ok(vec![]) → exit 0
            .stdout(predicate::str::contains("Packets: 0")) // no files processed
            .stderr(predicate::str::contains("Target not found").not()); // NOT an error
    }

    // ── AC-007 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC4 + Inv6: files that fail the magic-probe I/O (permission denied,
    /// unreadable) are silently skipped at the probe stage.  Probe failure does NOT abort
    /// directory scanning.
    ///
    /// Setup: create `b-unreadable.pcap` (CLASSIC_LE magic, then `chmod 000`) and
    /// `a-readable.pcap` (a real pcap fixture — http.pcap — that parses cleanly).
    ///
    /// Sort order ensures `a-readable.pcap` is processed FIRST (alphabetically).
    /// The stub extension filter includes BOTH files.  After processing `a-readable.pcap`
    /// successfully, it attempts to open `b-unreadable.pcap` → permission denied → the
    /// reader error exits non-zero AND stderr contains "Permission denied".
    ///
    /// Post-refactor: `read_magic(b-unreadable.pcap)` returns `None` (permission denied)
    /// → file silently skipped.  Only `a-readable.pcap` is included.  Command exits 0.
    ///
    /// RED oracle: assert exit SUCCESS with "Packets: 1" (a-readable.pcap only).
    /// Under the stub, `b-unreadable.pcap` is included → reader fails → exit non-zero.
    ///
    /// Note: `chmod 000` is non-portable (Windows does not support Unix permissions).
    /// This test is `#[cfg(unix)]`.
    ///
    /// BC-2.12.011 PC4 / Inv6.
    #[cfg(unix)]
    #[test]
    fn test_BC_2_12_011_io_error_on_probe_silently_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // a-readable.pcap: real pcap fixture (http.pcap = 1 packet).  Processes cleanly.
        // Named 'a-' so it is processed BEFORE b-unreadable (alphabetic sort order).
        fs::copy("tests/fixtures/http.pcap", dir_path.join("a-readable.pcap"))
            .expect("copy http.pcap → a-readable.pcap");

        // b-unreadable.pcap: chmod 000 → read_magic returns None → silently skipped.
        // Named 'b-' so it is processed AFTER a-readable (alphabetic sort order).
        // This ensures the stub does NOT abort before seeing b-unreadable.
        write_magic_file(&dir, "b-unreadable.pcap", &MAGIC_CLASSIC_LE);
        let unreadable_path = dir_path.join("b-unreadable.pcap");
        fs::set_permissions(&unreadable_path, fs::Permissions::from_mode(0o000))
            .expect("chmod 000 must succeed on Unix");

        // RED: stub includes both .pcap files by extension (sorted: a-readable, b-unreadable).
        //   Step 1: a-readable.pcap → reader processes http.pcap → 1 packet, ok.
        //   Step 2: b-unreadable.pcap → reader.open() → permission denied → exit non-zero.
        // The command exits 1 (reader error). "Packets: 1" never appears in stdout.
        //
        // Post-refactor:
        //   Step 1: read_magic(a-readable.pcap) → CLASSIC_LE magic → included.
        //   Step 2: read_magic(b-unreadable.pcap) → permission denied → None → SKIPPED.
        //   Only a-readable.pcap included → reader produces 1 packet → exit 0.
        //   stderr does NOT contain "Permission denied" (probe failure is silent).
        //
        // RED assertion: exit SUCCESS and "Packets: 1".
        // Under the stub, exit fails (b-unreadable → reader error) → .success() FAILS.
        wirerust()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            // RED: stub exits non-zero (reader can't open b-unreadable.pcap).
            // Post-refactor: b-unreadable silently skipped; a-readable → 1 packet → exit 0.
            .success()
            .stdout(predicate::str::contains("Packets: 1"))
            // Discriminating: probe failure MUST NOT appear in stderr post-refactor.
            .stderr(predicate::str::contains("Permission denied").not());

        // Restore permissions so TempDir cleanup can delete the file.
        fs::set_permissions(&unreadable_path, fs::Permissions::from_mode(0o644))
            .expect("chmod 644 restore must succeed");
    }

    // ── AC-008 ────────────────────────────────────────────────────────────────

    /// BC-2.12.011 PC7 + Inv4: subdirectories and symlinks-to-directories are skipped.
    /// `is_file()` check precedes magic probe.  Expansion is NOT recursive.
    ///
    /// Setup: create a subdirectory `captures/` inside the tempdir, containing a valid
    /// `.pcap` file (CLASSIC_LE magic, 8 bytes).  The top-level tempdir itself has no
    /// files.
    ///
    /// Both stub and post-refactor should skip the subdirectory (`is_file()` is already
    /// present in the stub's loop).  This test guards against regressions where the
    /// `is_file()` check is removed.
    ///
    /// Observable: directory with only a subdir → Packets: 0, exit 0.
    ///
    /// BC-2.12.011 PC7 / Inv4 / EC-011.
    #[test]
    fn test_BC_2_12_011_subdir_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Create captures/ subdirectory.
        let subdir = dir_path.join("captures");
        fs::create_dir(&subdir).expect("create captures/ subdir");

        // Write a valid pcap file INSIDE the subdirectory (should NOT be processed).
        write_magic_file(
            // write_magic_file expects &TempDir but we need to write to subdir directly
            // — use fs::write directly.
            &dir,      // unused dir reference; overridden below
            "ignored", // placeholder; actual write below
            &MAGIC_CLASSIC_LE,
        );
        // Direct write to the subdirectory path:
        let nested_path = subdir.join("nested.pcap");
        let mut content = [0u8; 8];
        content[..4].copy_from_slice(&MAGIC_CLASSIC_LE);
        fs::write(&nested_path, content).expect("write nested.pcap");
        // Remove the placeholder written to dir root by write_magic_file above:
        let _ = fs::remove_file(dir_path.join("ignored"));

        // Assert: directory contains only captures/ (a subdir) → Packets: 0.
        // is_file() = false for captures/ → skipped.
        // nested.pcap is inside captures/ → NOT reached (non-recursive scan).
        wirerust()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success() // empty top-level → exit 0
            .stdout(predicate::str::contains("Packets: 0")); // subdir not recursed
    }

    // ── AC-009 (E2E corpus) ───────────────────────────────────────────────────

    /// BC-2.12.011 EC-001..002 / STORY-127 E2E corpus wiring.
    ///
    /// Runs `PcapSource::from_pcap_reader` (the full reader stack from
    /// STORY-123..126) against four fixtures and asserts expected packet counts
    /// and datalink values.
    ///
    /// ## Sub-cases
    ///
    /// 1. `smb3.pcapng` — routes to pcapng reader (STORY-123 probe + STORY-125 EPB parse);
    ///    result is `Ok(PcapSource)` with `packets.len() > 0`.
    ///    (STORY-123 already pins the exact file acceptance; we assert non-empty packets here.)
    ///
    /// 2. `arp-baseline-16pkt.cap` — pcapng content with `.cap` extension; resolves C-2.
    ///    `packets.len() == 16`.  Uses the authentic PacketLife file if present at
    ///    `tests/fixtures/local-samples/arp-baseline-16pkt.cap` (sha256
    ///    d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25, 16 EPBs).
    ///    Falls back to a synthetic 16-packet pcapng in a tempfile if absent (CI).
    ///
    /// 3. Synthetic two-IDB pcapng (both ETHERNET) — built inline; `Ok(PcapSource)` with
    ///    `datalink == DataLink::ETHERNET`.
    ///
    /// 4. Synthetic OPB-only pcapng — built inline; `Ok(PcapSource)` with
    ///    `packets.len() == 0` and `opb_skipped > 0`.
    ///
    /// ## F-5 deferral status
    ///
    /// The authentic `arp-baseline-16pkt.cap` lives at
    /// `tests/fixtures/local-samples/arp-baseline-16pkt.cap` (gitignored).  In a clean
    /// checkout — including CI — that directory is empty and the synthetic 16-packet
    /// fallback runs by default.  The authentic file is fetched on demand via
    /// `bin/fetch-e2e-pcaps` and is required for Phase-4 holdout validation.
    /// F-5 remains deferred to Phase-4; CI always exercises the synthetic path.
    ///
    /// ## RED gate status
    ///
    /// Sub-cases 1 and 2 exercise `PcapSource::from_pcap_reader` / `from_file` — these
    /// are library functions (not dependent on resolve_targets) and are expected to be
    /// GREEN if STORY-123..126 are merged.  The STORY-127 worktree is branched from
    /// develop at commit 56a10e9 which includes the merged pcapng reader stack.
    ///
    /// Sub-cases 3 and 4 also use the library reader directly and are expected GREEN
    /// if the reader stack (EPB parse, OPB skip) is implemented.
    ///
    /// ALL sub-cases will be GREEN in E2E once the reader stack is complete.
    /// The resolve_targets-dependent behavior (AC-001..008) tests are the RED tests.
    ///
    /// This test is marked as a single `#[test]` function with sub-case structure
    /// per the AC-009 specification.
    #[test]
    fn test_BC_2_12_011_e2e_corpus_pcapng_reader_stack() {
        // ── Sub-case 1: smb3.pcapng ──────────────────────────────────────────
        //
        // smb3.pcapng is a committed fixture (15692 bytes, pcapng format, LE section).
        // BC-2.12.011 EC-001: file with pcapng magic accepted by from_pcap_reader.
        // Asserts Ok result and non-empty packets (STORY-123 + STORY-125 EPB parse).
        {
            let path = std::path::Path::new("tests/fixtures/smb3.pcapng");
            assert!(
                path.exists(),
                "smb3.pcapng must exist at tests/fixtures/smb3.pcapng"
            );

            let result = PcapSource::from_file(path);
            assert!(
                result.is_ok(),
                "AC-009 sub-case 1: smb3.pcapng must return Ok(PcapSource); got: {:?}",
                result.err()
            );
            let source = result.unwrap();
            assert!(
                !source.packets.is_empty(),
                "AC-009 sub-case 1: smb3.pcapng must yield at least 1 packet (EPB blocks present)"
            );
        }

        // ── Sub-case 2: arp-baseline-16pkt.cap (C-2 regression fixture) ──────
        //
        // The authentic PacketLife capture has .cap extension and pcapng content.
        // sha256: d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25.
        // 16 EPBs confirmed by block-structure parse.
        //
        // F-5 deferral: RESOLVED locally (authentic file present, 16 EPBs verified).
        // CI fallback: synthetic 16-packet pcapng written to a tempfile.
        {
            let (path, _tempdir) = match authentic_arp_baseline_path() {
                Some(p) => (p, None),
                None => {
                    // Authentic file absent (CI environment without local-samples/).
                    // Fall back to synthetic 16-packet pcapng (identical structure to
                    // STORY-123's ensure_arp_baseline_fixture() but inline here).
                    let td = tempfile::tempdir()
                        .expect("AC-009 sub-case 2: tempdir for synthetic fallback");
                    let p = td.path().join("arp-baseline-16pkt.cap");
                    let bytes = synthetic_16pkt_pcapng();
                    fs::write(&p, &bytes)
                        .expect("AC-009 sub-case 2: write synthetic arp-baseline fixture");
                    (p, Some(td))
                }
            };

            let result = PcapSource::from_file(&path);
            assert!(
                result.is_ok(),
                "AC-009 sub-case 2: arp-baseline-16pkt.cap (.cap extension, pcapng content) \
                 must return Ok(PcapSource); resolves C-2. got: {:?}",
                result.err()
            );
            let source = result.unwrap();
            assert_eq!(
                source.packets.len(),
                16,
                "AC-009 sub-case 2: arp-baseline-16pkt.cap must yield exactly 16 packets. \
                 If authentic file is present: 16 real ARP EPBs (F-5 resolved). \
                 If synthetic fallback: 16 synthetic EPBs. got: {}",
                source.packets.len()
            );
        }

        // ── Sub-case 3: synthetic two-IDB pcapng (both ETHERNET) ─────────────
        //
        // BC-2.12.011 EC-001 / STORY-124 multi-IDB agreement pass.
        // Two IDBs with the same linktype (ETHERNET) must produce Ok(PcapSource)
        // with datalink == DataLink::ETHERNET.
        //
        // Structure: SHB + IDB(ETHERNET) + IDB(ETHERNET) + EPB(interface_id=0)
        // The EPB references interface 0 (first IDB); both IDBs are ETHERNET.
        // STORY-124 BC-2.01.018: two IDBs with same linktype → agreement pass → accepted.
        {
            let mut bytes = le_shb();
            bytes.extend_from_slice(&le_idb_ethernet()); // IDB #0: ETHERNET
            bytes.extend_from_slice(&le_idb_ethernet()); // IDB #1: ETHERNET (same → agreement)
            // EPB referencing interface 0 (first IDB) with empty payload.
            bytes.extend_from_slice(&le_epb_empty());

            let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
            assert!(
                result.is_ok(),
                "AC-009 sub-case 3: two-IDB pcapng (both ETHERNET) must return Ok; \
                 STORY-124 BC-2.01.018 multi-IDB agreement pass. got: {:?}",
                result.err()
            );
            let source = result.unwrap();
            // BC-2.12.011: resolved datalink must be ETHERNET (IDB #0 linktype).
            assert_eq!(
                source.datalink,
                DataLink::ETHERNET,
                "AC-009 sub-case 3: two-IDB pcapng (both ETHERNET) must have \
                 datalink == DataLink::ETHERNET; got {:?}",
                source.datalink
            );
            // Verify the EPB was counted.
            assert_eq!(
                source.packets.len(),
                1,
                "AC-009 sub-case 3: two-IDB pcapng with 1 EPB must yield exactly 1 packet"
            );
        }

        // ── Sub-case 4: synthetic OPB-only pcapng ────────────────────────────
        //
        // BC-2.12.011 EC-001 / STORY-126 skip dispatch + counter surfacing.
        // A pcapng with SHB + IDB(ETHERNET) + OPB must produce Ok(PcapSource)
        // with packets.len() == 0 and opb_skipped > 0.
        //
        // OPB (Obsolete Packet Block, type 0x00000002) is explicitly skipped
        // (not parsed) per STORY-126 dispatch; opb_skipped counter incremented.
        {
            let mut bytes = le_shb();
            bytes.extend_from_slice(&le_idb_ethernet()); // IDB required for OPB context
            bytes.extend_from_slice(&le_opb_empty()); // OPB #1: skipped, opb_skipped++
            bytes.extend_from_slice(&le_opb_empty()); // OPB #2: skipped, opb_skipped++

            let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
            assert!(
                result.is_ok(),
                "AC-009 sub-case 4: OPB-only pcapng must return Ok (OPBs skipped, not errored); \
                 got: {:?}",
                result.err()
            );
            let source = result.unwrap();
            assert_eq!(
                source.packets.len(),
                0,
                "AC-009 sub-case 4: OPB-only pcapng must yield 0 packets (OPBs not parsed)"
            );
            assert!(
                source.opb_skipped > 0,
                "AC-009 sub-case 4: OPB-only pcapng must have opb_skipped > 0; \
                 got opb_skipped = {}. STORY-126 dual-counter requirement.",
                source.opb_skipped
            );
            assert_eq!(
                source.opb_skipped, 2,
                "AC-009 sub-case 4: 2 OPBs → opb_skipped must be exactly 2; got {}",
                source.opb_skipped
            );
        }
    }
}
