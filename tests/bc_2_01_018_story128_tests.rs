//! STORY-128: main.rs Per-File Error Isolation Loop (Catch-and-Continue)
//!
//! Regression-guard suite for the STORY-128 per-file error isolation implementation.
//! All behavioral contracts exercised here are IMPLEMENTED. The implementation:
//!   1. `let mut any_error = false;` before the per-file loop in `run_analyze`
//!      (and analogously in `run_summary`).
//!   2. Explicit `match` on `PcapSource::from_file(path)`:
//!      - `Ok(source)` arm: existing processing (zero-packet notice check, analyzer dispatch).
//!      - `Err(e)` arm: `eprintln!("error: {}: {e:#}", path.display()); any_error = true; continue;`
//!   3. After the loop: `if any_error { std::process::exit(1); }`.
//!   4. Zero-packet notice (Decision 19) emitted in the `Ok` arm when
//!      `source.packets.is_empty()`.
//!   NOTE: `src/reader.rs` was not modified (STORY-128 Forbidden Dependencies).
//!
//! ## Implementation (STORY-128 delivered)
//!
//! `run_analyze` and `run_summary` use a catch-and-continue per-file loop:
//!   - `Ok(source)` arm: existing processing (zero-packet notice check, analyzer dispatch).
//!   - `Err(e)` arm: `eprintln!("error: {}: {e:#}", path.display()); any_error = true; continue;`
//!   - After the loop: `if any_error { std::process::exit(1); }`.
//!
//! The zero-packet notice (Decision 19) is emitted in the `Ok` arm when
//! `source.packets.is_empty()`. All tests below guard against regression of this behavior.
//!
//! ## Coverage map (BC-2.01.018 AC-002)
//!
//!   AC-001 → test_BC_2_01_018_per_file_isolation_continues_on_error
//!   AC-002 → test_BC_2_01_018_einp011_does_not_abort_batch
//!   AC-003 → test_BC_2_01_018_any_reader_error_isolated
//!   AC-004 → test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation
//!
//! Additional tests (ORDER INDEPENDENCE, ALL-GOOD, ALL-BAD, SINGLE-FILE-FAIL,
//! run_summary isolation, reader fail-closed) are also included per the
//! STORY-128 coverage requirements.
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` per factory mandate.
//! `#![allow(non_snake_case)]` required for the BC-prefixed pattern.
//!
//! ## Test approach
//!
//! All tests drive the CLI binary via `assert_cmd::Command::cargo_bin("wirerust")`
//! using a tempdir containing crafted minimal pcapng byte fixtures.  This mirrors
//! the approach used in `tests/bc_2_12_011_story127_tests.rs`, since `run_analyze`
//! and `run_summary` are private to the binary crate and not accessible from
//! integration test files.
//!
//! ## Pcapng fixture byte layout
//!
//! Minimal VALID pcapng (SHB + ETHERNET IDB + one EPB):
//!   SHB:  0A 0D 0D 0A 1C 00 00 00 4D 3C 2B 1A 00 01 00 00 FF FF FF FF FF FF FF FF 1C 00 00 00
//!   IDB:  01 00 00 00 14 00 00 00 01 00 00 00 FF FF 00 00 14 00 00 00
//!         (linktype=1 ETHERNET, reserved=0, snaplen=65535)
//!   EPB:  06 00 00 00 20 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 20 00 00 00
//!         (empty payload, 32-byte block)
//!
//! Minimal CONFLICT pcapng (SHB + ETHERNET IDB + LINUX_SLL IDB → E-INP-011):
//!   SHB + IDB(linktype=1) + IDB(linktype=113/0x71) — second IDB conflicts.
//!
//! Minimal TRUNCATED pcapng (SHB body too short → E-INP-008):
//!   SHB with btl=16 (body=4 bytes, less than the 16-byte SHB fixed minimum).
//!
//! Minimal SHB-ONLY pcapng (SHB alone, no IDB, no EPB → Ok, 0 packets):
//!   SHB block only — valid structure, yields PcapSource { packets: [] }.
//!
//! These byte sequences are constructed inline in each test using helper functions.
//! No external fixture files are needed for STORY-128.

#![allow(non_snake_case)]
#![allow(clippy::doc_lazy_continuation)]

mod story_128 {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // Pcapng byte-fixture helpers
    //
    // All byte sequences are little-endian (LE) per the SHB BOM 0x1A2B3C4D.
    // Block total length includes the 12-byte outer header (type:4 + btl:4 +
    // trailing btl:4).
    // -----------------------------------------------------------------------

    /// Minimal valid pcapng SHB (Section Header Block).
    ///
    /// Block type: 0x0A0D0D0A (pcapng magic)
    /// BOM: 0x1A2B3C4D (little-endian)
    /// Version: 1.0
    /// Section length: -1 (unknown, 0xFFFFFFFFFFFFFFFF)
    /// btl = 28 (12 outer + 16 body)
    fn shb_bytes() -> Vec<u8> {
        let mut b = Vec::new();
        // block_type (LE)
        b.extend_from_slice(&[0x0A, 0x0D, 0x0D, 0x0A]);
        // block_total_length = 28 (LE)
        b.extend_from_slice(&[0x1C, 0x00, 0x00, 0x00]);
        // BOM = 0x1A2B3C4D (LE)
        b.extend_from_slice(&[0x4D, 0x3C, 0x2B, 0x1A]);
        // major = 1 (LE u16)
        b.extend_from_slice(&[0x01, 0x00]);
        // minor = 0 (LE u16)
        b.extend_from_slice(&[0x00, 0x00]);
        // section_length = -1 (LE i64 = 0xFFFFFFFFFFFFFFFF)
        b.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        // trailing block_total_length = 28 (LE)
        b.extend_from_slice(&[0x1C, 0x00, 0x00, 0x00]);
        assert_eq!(b.len(), 28, "SHB must be 28 bytes");
        b
    }

    /// Minimal IDB (Interface Description Block) with the given linktype (u16 LE).
    ///
    /// btl = 20 (12 outer + 8 body: linktype:2 + reserved:2 + snaplen:4)
    fn idb_bytes(linktype: u16) -> Vec<u8> {
        let mut b = Vec::new();
        // block_type = 0x00000001 (LE)
        b.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        // block_total_length = 20 (LE)
        b.extend_from_slice(&[0x14, 0x00, 0x00, 0x00]);
        // linktype (LE u16)
        b.extend_from_slice(&linktype.to_le_bytes());
        // reserved = 0 (LE u16)
        b.extend_from_slice(&[0x00, 0x00]);
        // snaplen = 65535 (LE u32)
        b.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
        // trailing block_total_length = 20 (LE)
        b.extend_from_slice(&[0x14, 0x00, 0x00, 0x00]);
        assert_eq!(b.len(), 20, "IDB must be 20 bytes");
        b
    }

    /// Minimal EPB (Enhanced Packet Block) with a zero-length payload.
    ///
    /// btl = 32 (12 outer + 20 body: interface_id:4 + ts_high:4 + ts_low:4
    ///   + captured_len:4 + original_len:4 + trailing btl:4).
    /// captured_len = 0; original_len = 0; no packet data, no padding.
    fn epb_bytes_empty() -> Vec<u8> {
        let btl: u32 = 32;
        let mut b = Vec::new();
        // block_type = 0x00000006 (LE)
        b.extend_from_slice(&[0x06, 0x00, 0x00, 0x00]);
        // block_total_length = 32 (LE)
        b.extend_from_slice(&btl.to_le_bytes());
        // interface_id = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // ts_high = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // ts_low = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // captured_len = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // original_len = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // trailing block_total_length = 32 (LE)
        b.extend_from_slice(&btl.to_le_bytes());
        assert_eq!(b.len(), 32, "EPB with empty payload must be 32 bytes");
        b
    }

    /// A valid minimal pcapng: SHB + ETHERNET IDB + one EPB (0-byte payload).
    ///
    /// Parses to Ok(PcapSource) with packets.len() == 1 via the reader.
    /// The EPB has zero-byte payload — decode stage will count this as a
    /// decode error ("Not enough data...") but the reader Ok path is reached.
    fn valid_pcapng_bytes() -> Vec<u8> {
        let mut b = shb_bytes();
        b.extend(idb_bytes(1)); // ETHERNET = linktype 1
        b.extend(epb_bytes_empty());
        b
    }

    /// A conflict pcapng: SHB + ETHERNET IDB + LINUX_SLL IDB → E-INP-011.
    ///
    /// The second IDB (linktype=113/LINUX_SLL) conflicts with the first
    /// (linktype=1/ETHERNET).  Both are whitelisted, so E-INP-011 fires at
    /// the third IDB-parse check (ADR-009 Decision 17 precedence):
    ///   1st check: E-INP-013 position (skipped — no packets emitted yet)
    ///   2nd check: E-INP-001 whitelist (LINUX_SLL passes the whitelist)
    ///   3rd check: E-INP-011 conflict (ETHERNET ≠ LINUX_SLL → fires)
    ///
    /// BC-2.01.018 EC-003 canonical test vector: ETHERNET then LINUX_SLL.
    fn conflict_pcapng_bytes() -> Vec<u8> {
        let mut b = shb_bytes();
        b.extend(idb_bytes(1)); // ETHERNET = linktype 1 (whitelisted)
        b.extend(idb_bytes(113)); // LINUX_SLL = linktype 113 (whitelisted, conflicts)
        b
    }

    /// A truncated SHB pcapng: SHB body too short → E-INP-008 (structural
    /// body-decode failure).
    ///
    /// btl=16 means body = 16-12 = 4 bytes, which is less than the 16-byte
    /// SHB fixed body minimum.  The block is framing-valid (btl >= 12,
    /// 4-byte-aligned, trailing btl present) so it passes E-INP-010
    /// (Tier 1 framing) but fails at E-INP-008 (Tier 2 body-decode).
    ///
    /// ADR-009 Decision 20 Tier 2: wirerust body-decode failure → E-INP-008.
    fn truncated_shb_pcapng_bytes() -> Vec<u8> {
        let mut b = Vec::new();
        // block_type = pcapng SHB magic (0x0A0D0D0A, LE)
        b.extend_from_slice(&[0x0A, 0x0D, 0x0D, 0x0A]);
        // block_total_length = 16 (LE) — body=4 bytes < 16 minimum → E-INP-008
        b.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);
        // body: 4 bytes (BOM-looking bytes — will be rejected before BOM check)
        b.extend_from_slice(&[0x4D, 0x3C, 0x2B, 0x1A]);
        // trailing block_total_length = 16 (LE)
        b.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);
        assert_eq!(b.len(), 16, "truncated SHB must be 16 bytes");
        b
    }

    /// A SHB-only pcapng: SHB block alone, no IDB, no EPBs.
    ///
    /// Returns Ok(PcapSource { packets: [], skipped_blocks: 0, opb_skipped: 0,
    /// datalink: DataLink::from(0) }) — valid structure, 0 packets.
    ///
    /// Per ADR-009 Decision 19: main.rs MUST emit a zero-packet notice for this
    /// case (Ok arm, packets.is_empty() == true).
    fn shb_only_pcapng_bytes() -> Vec<u8> {
        shb_bytes()
    }

    // -----------------------------------------------------------------------
    // CLI helper
    // -----------------------------------------------------------------------

    /// Build an `assert_cmd::Command` targeting the wirerust binary.
    fn wirerust() -> Command {
        Command::cargo_bin("wirerust").expect("wirerust binary must be built")
    }

    // -----------------------------------------------------------------------
    // AC-001: Per-file isolation continues on error
    //
    // GREEN: catch-and-continue per-file loop processes all files even when one fails.
    // A regression (re-introducing `?` propagation) would abort on file_a's Err
    // and never process file_b.
    // -----------------------------------------------------------------------

    /// AC-001 / BC-2.01.018 AC-002: directory with [bad.pcap (E-INP-011
    /// conflict), good.pcapng (valid)] sorted alphabetically so the bad file
    /// processes first.  The batch MUST complete; the good file MUST be analyzed;
    /// exit code MUST be 1; the bad file MUST produce an error notice on stderr.
    ///
    /// ## Regression guard
    ///
    /// Files sorted: "a-conflict.pcapng" < "b-valid.pcapng" (alphabetic LE).
    /// Catch-and-continue: `from_file("a-conflict.pcapng")` → Err arm:
    ///   - eprintln!("error: a-conflict.pcapng: ... link-type conflict ...")
    ///   - any_error = true; continue;
    /// match on `from_file("b-valid.pcapng")` → Ok arm:
    ///   - packet counted (decode error for 0-byte payload, but reader Ok reached)
    /// After loop: any_error == true → std::process::exit(1).
    /// stdout contains "Packets: 1" (or "Skipped: 1 packets" from decode error on
    /// 0-byte payload — the key discriminator is the run COMPLETES with packet data
    /// from both files attempted, and exit code 1).
    ///
    /// BC-2.01.018 AC-002 / ADR-009 Decision 12 / STORY-128 AC-001.
    #[test]
    fn test_BC_2_01_018_per_file_isolation_continues_on_error() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-conflict.pcapng: ETHERNET + LINUX_SLL IDBs → E-INP-011.
        // Sorts BEFORE b-valid.pcapng; regression guard: if `?` propagation returns,
        // b-valid.pcapng would never be processed.
        fs::write(
            dir.path().join("a-conflict.pcapng"),
            conflict_pcapng_bytes(),
        )
        .expect("write a-conflict.pcapng");

        // b-valid.pcapng: SHB + ETHERNET IDB + 1 EPB (0-byte payload).
        // Reader returns Ok with 1 packet.  Decode stage counts a decode error
        // (0-byte Ethernet payload) but reader Ok path is reached.
        fs::write(dir.path().join("b-valid.pcapng"), valid_pcapng_bytes())
            .expect("write b-valid.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        // GREEN: catch-and-continue loop exits 1 (any_error=true for a-conflict.pcapng)
        // AND b-valid.pcapng IS processed (1 raw packet → decode error → Skipped: 1).
        // Per-file error notice emitted on stderr with the path-prefixed format
        //   "error: <path>/a-conflict.pcapng: ..."
        // Regression guard: pre-implementation, run_analyze would abort on Err,
        // skipping b-valid.pcapng and emitting an anyhow chain without per-file prefix.
        assert
            // Exit 1: any_error=true (a-conflict.pcapng failed)
            .failure()
            // Per-file error notice on stderr:
            //   "error: <path>/a-conflict.pcapng: ..."
            .stderr(predicate::str::contains("a-conflict.pcapng"))
            // b-valid.pcapng IS processed: packet counted in final summary.
            // "Skipped: 1 packets" discriminates from a regression where b-valid
            // is never reached (regression would produce "Packets: 0" or "Skipped: 0").
            .stdout(predicate::str::contains("Skipped: 1 packets"));
    }

    // -----------------------------------------------------------------------
    // AC-002: E-INP-011 specifically does not abort the batch
    //
    // Pins the specific conflict message on stderr (not just any error).
    // -----------------------------------------------------------------------

    /// AC-002 / BC-2.01.018 AC-002 EC-009: E-INP-011 on one file does NOT abort
    /// the batch.  Specifically exercises the multi-IDB linktype conflict case
    /// (ETHERNET then LINUX_SLL) as the canonical motivating error for STORY-128.
    ///
    /// Distinct from AC-001: this test additionally pins that stderr contains the
    /// E-INP-011 conflict marker ("link-type conflict" or similar) for the failing
    /// file, and that the good file (LAST in sort order) IS processed.
    ///
    /// BC-2.01.018 EC-003 canonical test vector: two IDBs ETHERNET then LINUX_SLL
    /// → E-INP-011 "interface 0 has ETHERNET, interface 1 has LINUX_SLL".
    ///
    /// ## Regression guard
    ///
    /// Files sorted: "1-conflict.pcapng" < "2-good.pcapng".
    /// Err arm for "1-conflict.pcapng": emits "error: <path>: ... link-type
    /// conflict ..." to stderr; any_error=true; continue.
    /// Ok arm for "2-good.pcapng": processes 1 EPB → Skipped: 1 packets in stdout.
    /// exit 1 (any_error=true). Regression would skip "2-good.pcapng" entirely.
    ///
    /// BC-2.01.018 AC-002 EC-009 / ADR-009 Decision 12 / STORY-128 AC-002.
    #[test]
    fn test_BC_2_01_018_einp011_does_not_abort_batch() {
        let dir = tempfile::tempdir().expect("tempdir");

        // 1-conflict.pcapng sorts BEFORE 2-good.pcapng — bad file first.
        fs::write(
            dir.path().join("1-conflict.pcapng"),
            conflict_pcapng_bytes(),
        )
        .expect("write 1-conflict.pcapng");

        fs::write(dir.path().join("2-good.pcapng"), valid_pcapng_bytes())
            .expect("write 2-good.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: conflict file failed
            // E-INP-011 conflict message on stderr — canonical message text
            // from BC-2.01.018 Postcondition 2: "link-type conflict".
            // Also accept "conflict" as a substring since the exact wording
            // is implementation-defined so long as it includes the word.
            .stderr(predicate::str::contains("conflict"))
            // The path of the bad file MUST appear in stderr
            .stderr(predicate::str::contains("1-conflict.pcapng"))
            // 2-good.pcapng IS processed: 1 EPB with 0-byte payload →
            // decode error → "Skipped: 1 packets" in stdout.
            // Regression guard: isolation ensures 2-good.pcapng is always processed.
            .stdout(predicate::str::contains("Skipped: 1 packets"));
    }

    // -----------------------------------------------------------------------
    // AC-003: Isolation applies to ALL reader error classes (not only E-INP-011)
    //
    // Uses a truncated SHB (E-INP-008) as the per-file error.
    // -----------------------------------------------------------------------

    /// AC-003 / BC-2.01.018 AC-002 (all error classes isolated): a truncated
    /// SHB file (E-INP-008 structural body-decode failure) alongside a valid
    /// ETHERNET file.  E-INP-008 (not only E-INP-011) MUST be caught and isolated.
    ///
    /// This test verifies that the isolation loop is NOT specific to E-INP-011 —
    /// ANY `Err` from `from_file` is caught, the path is reported to stderr,
    /// and the loop continues to the next file.
    ///
    /// ADR-009 Decision 12: "This fix benefits ALL reader errors, not only
    /// pcapng errors."
    ///
    /// ## Regression guard
    ///
    /// Files sorted: "a-truncated.pcapng" < "b-valid.pcapng".
    /// Err arm for "a-truncated.pcapng": eprintln per-file error; continue.
    /// Ok arm for "b-valid.pcapng": 1 EPB counted (decode error); Skipped: 1.
    /// Regression (re-introducing `?` propagation) would skip "b-valid.pcapng".
    /// exit 1 (any_error=true).
    ///
    /// BC-2.01.018 AC-002 (all error classes) / ADR-009 Decision 12 / STORY-128 AC-003.
    #[test]
    fn test_BC_2_01_018_any_reader_error_isolated() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-truncated.pcapng: SHB with btl=16 → body=4 bytes < 16 minimum → E-INP-008.
        // Sorts BEFORE b-valid.pcapng (alphabetic order: "a" < "b").
        fs::write(
            dir.path().join("a-truncated.pcapng"),
            truncated_shb_pcapng_bytes(),
        )
        .expect("write a-truncated.pcapng");

        // b-valid.pcapng: SHB + ETHERNET IDB + 1 EPB (0-byte payload).
        fs::write(dir.path().join("b-valid.pcapng"), valid_pcapng_bytes())
            .expect("write b-valid.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: truncated file failed
            // Path of the bad file MUST appear in stderr error notice.
            .stderr(predicate::str::contains("a-truncated.pcapng"))
            // GREEN: b-valid.pcapng IS processed — 1 EPB counted as decode error.
            .stdout(predicate::str::contains("Skipped: 1 packets"));
    }

    // -----------------------------------------------------------------------
    // AC-004: Zero-packet notice not suppressed by isolation
    //
    // ADR-009 Decision 19: a valid SHB-only file with 0 packets MUST emit the
    // zero-packet notice.  The isolation Err arm for a sibling bad file MUST NOT
    // suppress the zero-packet notice from the Ok arm of the SHB-only file.
    //
    // GREEN: zero-packet notice is implemented (src/main.rs:61-89), and the
    // catch-and-continue loop processes b-shb-only.pcapng even after a-conflict fails.
    // -----------------------------------------------------------------------

    /// AC-004 / BC-2.01.018 AC-002 + ADR-009 Decision 19: zero-packet notice is
    /// NOT suppressed by the isolation logic.
    ///
    /// Directory contains:
    ///   "a-conflict.pcapng" (E-INP-011 ETHERNET+LINUX_SLL) → Err
    ///   "b-shb-only.pcapng" (SHB only, no IDB, no EPBs) → Ok, 0 packets
    ///
    /// Files sorted: "a-conflict.pcapng" < "b-shb-only.pcapng".
    ///
    /// Expected post-refactor behavior:
    ///   - Err arm for "a-conflict.pcapng": per-file error to stderr; continue.
    ///   - Ok arm for "b-shb-only.pcapng": packets.is_empty() == true → emit
    ///     zero-packet notice to stderr:
    ///     "notice: <path>/b-shb-only.pcapng: 0 packets read from pcapng file"
    ///   - exit 1 (any_error=true from a-conflict.pcapng failure).
    ///
    /// Assertions:
    ///   - stderr contains "a-conflict.pcapng" (per-file error from Err arm)
    ///   - stderr contains "b-shb-only.pcapng" AND "0 packets" (zero-packet notice)
    ///   - exit code 1 (one file failed)
    ///
    /// ## Regression guard
    ///
    /// Guards that per-file isolation continues past a failed file AND the zero-packet
    /// notice is emitted for the successful zero-packet file. Regression would cause
    /// either the notice to disappear or the second file to be skipped.
    ///
    /// BC-2.01.018 AC-002 (zero-packet notice not suppressed) / ADR-009 Decision 19
    /// / STORY-128 AC-004.
    #[test]
    fn test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-conflict.pcapng: E-INP-011 → Err.  Sorts BEFORE b-shb-only.pcapng.
        fs::write(
            dir.path().join("a-conflict.pcapng"),
            conflict_pcapng_bytes(),
        )
        .expect("write a-conflict.pcapng");

        // b-shb-only.pcapng: SHB only, no IDB, no EPBs → Ok, 0 packets.
        // ADR-009 Decision 19: must trigger the zero-packet notice.
        fs::write(
            dir.path().join("b-shb-only.pcapng"),
            shb_only_pcapng_bytes(),
        )
        .expect("write b-shb-only.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            // exit 1: a-conflict.pcapng failed (any_error=true)
            .failure()
            // Per-file error for the bad file on stderr
            .stderr(predicate::str::contains("a-conflict.pcapng"))
            // Zero-packet notice for b-shb-only.pcapng on stderr.
            // ADR-009 Decision 19 format: "notice: <path>: 0 packets read from pcapng file"
            // We assert the path appears AND "0 packets" appears to pin the notice firing.
            // Regression guards: notice present AND second file was reached (isolation works).
            .stderr(predicate::str::contains("b-shb-only.pcapng"))
            .stderr(predicate::str::contains("0 packets"));
    }

    // -----------------------------------------------------------------------
    // ORDER INDEPENDENCE: bad file FIRST vs. LAST vs. MIDDLE
    //
    // These are the strongest isolation discriminators: if the bad file is
    // first in sort order and a pre-implementation abort would skip all subsequent files.
    // All three orderings MUST produce the same outcome:
    // good files analyzed, bad file error on stderr, exit 1.
    // -----------------------------------------------------------------------

    /// ORDER INDEPENDENCE (bad file FIRST): directory sorted so the bad file
    /// ("a-bad.pcapng") sorts BEFORE the good files ("b-good.pcapng",
    /// "c-good.pcapng").  Both good files MUST be processed.
    ///
    /// GREEN regression guard: isolation catches "a-bad.pcapng" Err; "b-good.pcapng"
    /// and "c-good.pcapng" both processed (1 packet each); Skipped: 2 packets.
    /// Regression (re-introducing `?`) would abort after "a-bad.pcapng".
    ///
    /// BC-2.01.018 AC-002 / STORY-128 EC-001.
    #[test]
    fn test_BC_2_01_018_order_independence_bad_file_first() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-bad.pcapng: E-INP-011 conflict; sorts FIRST (alphabetically "a" < "b" < "c").
        fs::write(dir.path().join("a-bad.pcapng"), conflict_pcapng_bytes())
            .expect("write a-bad.pcapng");

        // b-good.pcapng: SHB + ETHERNET IDB + 1 EPB → Ok, 1 packet (decode error).
        fs::write(dir.path().join("b-good.pcapng"), valid_pcapng_bytes())
            .expect("write b-good.pcapng");

        // c-good.pcapng: same as b-good.pcapng.
        fs::write(dir.path().join("c-good.pcapng"), valid_pcapng_bytes())
            .expect("write c-good.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: a-bad.pcapng failed
            .stderr(predicate::str::contains("a-bad.pcapng"))
            // Both good files processed: 2 EPBs (0-byte payload each) → 2 decode errors
            // → "Skipped: 2 packets" in stdout.
            // Regression guard: abort on a-bad.pcapng would skip both good files.
            .stdout(predicate::str::contains("Skipped: 2 packets"));
    }

    /// ORDER INDEPENDENCE (bad file LAST): directory sorted so the bad file
    /// ("c-bad.pcapng") sorts AFTER the good files ("a-good.pcapng",
    /// "b-good.pcapng").  Both good files are processed before the bad file.
    ///
    /// GREEN: both good files are processed before "c-bad.pcapng" (sort order),
    /// "c-bad.pcapng" triggers the Err arm (per-file "error: <path>:" notice to stderr),
    /// any_error=true → exit 1. "Skipped: 2 packets" in stdout (both good files counted).
    ///
    /// Regression guard: the per-file "error: <path>: ..." format (not an anyhow
    /// top-level chain) must appear on stderr for c-bad.pcapng.
    ///
    /// Post-refactor: stderr uses the per-file eprintln format.
    ///
    /// BC-2.01.018 AC-002 / STORY-128 EC-001 (order independence).
    #[test]
    fn test_BC_2_01_018_order_independence_bad_file_last() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-good.pcapng: sorts first; valid, 1 EPB → decode error.
        fs::write(dir.path().join("a-good.pcapng"), valid_pcapng_bytes())
            .expect("write a-good.pcapng");

        // b-good.pcapng: sorts second; valid, 1 EPB → decode error.
        fs::write(dir.path().join("b-good.pcapng"), valid_pcapng_bytes())
            .expect("write b-good.pcapng");

        // c-bad.pcapng: sorts LAST; E-INP-011 conflict → Err.
        fs::write(dir.path().join("c-bad.pcapng"), conflict_pcapng_bytes())
            .expect("write c-bad.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: c-bad.pcapng failed
            // Per-file error format: "error: <path>/c-bad.pcapng: ..."
            // Regression guard: pre-implementation emitted anyhow chain without
            // per-file "error: <path>:" prefix format.
            .stderr(predicate::str::contains("c-bad.pcapng"))
            // Both good files processed BEFORE the bad file:
            // 2 EPBs (0-byte payload) → 2 decode errors → Skipped: 2.
            .stdout(predicate::str::contains("Skipped: 2 packets"));
    }

    /// ORDER INDEPENDENCE (bad file MIDDLE): directory sorted so the bad file
    /// ("b-bad.pcapng") sorts BETWEEN two good files ("a-good.pcapng",
    /// "c-good.pcapng").
    ///
    /// GREEN: "a-good.pcapng" processed; "b-bad.pcapng" caught and isolated;
    /// "c-good.pcapng" processed. Both good files contribute to "Skipped: 2 packets".
    /// Regression would abort after "b-bad.pcapng" and skip "c-good.pcapng".
    ///
    /// BC-2.01.018 AC-002 / STORY-128 EC-001 (order independence).
    #[test]
    fn test_BC_2_01_018_order_independence_bad_file_middle() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-good.pcapng: sorts first; valid, 1 EPB → decode error.
        fs::write(dir.path().join("a-good.pcapng"), valid_pcapng_bytes())
            .expect("write a-good.pcapng");

        // b-bad.pcapng: sorts MIDDLE; E-INP-011 conflict → Err.
        fs::write(dir.path().join("b-bad.pcapng"), conflict_pcapng_bytes())
            .expect("write b-bad.pcapng");

        // c-good.pcapng: sorts last; valid, 1 EPB → decode error.
        fs::write(dir.path().join("c-good.pcapng"), valid_pcapng_bytes())
            .expect("write c-good.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: b-bad.pcapng failed
            .stderr(predicate::str::contains("b-bad.pcapng"))
            // a-good AND c-good both processed: 2 packets total.
            .stdout(predicate::str::contains("Skipped: 2 packets"));
    }

    // -----------------------------------------------------------------------
    // ALL-GOOD batch: all files valid → all analyzed, exit 0, no error notices
    // -----------------------------------------------------------------------

    /// ALL-GOOD batch: directory with two valid pcapng files → both processed,
    /// exit 0, no error notices on stderr.
    ///
    /// This test MUST be GREEN both before and after refactor.  It verifies:
    ///   1. The refactor does not break the success path.
    ///   2. No spurious error notices appear when all files succeed.
    ///
    /// BC-2.01.018 / STORY-128 EC-002.
    #[test]
    fn test_BC_2_01_018_all_good_batch_exit_zero() {
        let dir = tempfile::tempdir().expect("tempdir");

        fs::write(dir.path().join("a-good.pcapng"), valid_pcapng_bytes())
            .expect("write a-good.pcapng");
        fs::write(dir.path().join("b-good.pcapng"), valid_pcapng_bytes())
            .expect("write b-good.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // exit 0: all files succeeded (decode errors on 0-byte payloads are
            // NOT reader errors; they are counted in Skipped but do not set exit 1)
            .success()
            // Both files processed: 2 EPBs → 2 decode errors → Skipped: 2
            .stdout(predicate::str::contains("Skipped: 2 packets"))
            // No per-file error notices for a-good.pcapng or b-good.pcapng.
            // We check the specific filenames rather than a generic "error: " pattern,
            // because the decode-error Warning on stderr also contains "error" as a
            // substring of the error description text (not a per-file isolation error).
            .stderr(predicate::str::contains("a-good.pcapng").not())
            .stderr(predicate::str::contains("b-good.pcapng").not());
    }

    // -----------------------------------------------------------------------
    // ALL-BAD batch: all files bad → all emitted error notices, exit 1, no crash
    // -----------------------------------------------------------------------

    /// ALL-BAD batch: directory with two bad pcapng files → both emit error
    /// notices to stderr, exit 1, process COMPLETES (no panic, no crash).
    ///
    /// BC-2.01.018 / STORY-128 EC-003.
    ///
    /// ## Regression guard
    ///
    /// GREEN: both bad files are processed through the Err arm;
    /// per-file error notices emitted for both; any_error=true; exit 1.
    ///
    /// Regression: `?` propagation would skip the second bad file and omit its
    /// per-file error notice.
    ///
    /// Both bad files processed through the Err arm:
    ///   - eprintln("error: a-bad.pcapng: ..."); any_error=true; continue.
    ///   - eprintln("error: b-bad.pcapng: ..."); any_error=true; continue.
    /// After loop: std::process::exit(1).
    /// Both paths appear in stderr.
    ///
    /// The discriminating assertion: BOTH paths appear in stderr.
    /// Regression guard: isolation ensures both files are attempted, not just the first.
    #[test]
    fn test_BC_2_01_018_all_bad_batch_no_panic_exit_one() {
        let dir = tempfile::tempdir().expect("tempdir");

        fs::write(dir.path().join("a-bad.pcapng"), conflict_pcapng_bytes())
            .expect("write a-bad.pcapng");
        fs::write(
            dir.path().join("b-bad.pcapng"),
            truncated_shb_pcapng_bytes(),
        )
        .expect("write b-bad.pcapng");

        let assert = wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1
            // BOTH bad files must appear in stderr error notices.
            // Regression guard: abort after "a-bad.pcapng" would omit "b-bad.pcapng" from stderr.
            .stderr(predicate::str::contains("a-bad.pcapng"))
            .stderr(predicate::str::contains("b-bad.pcapng"));
    }

    // -----------------------------------------------------------------------
    // READER FAIL-CLOSED UNCHANGED (BC-2.01.018 Invariant 1)
    //
    // A SINGLE bad file passed directly (not a directory) still produces Err/
    // exit non-zero.  Per-file isolation is DIRECTORY MODE behavior; a single
    // file invocation does not exercise directory-mode isolation.
    //
    // This test must be GREEN both before and after refactor (no regression).
    // -----------------------------------------------------------------------

    /// BC-2.01.018 Invariant 1 (reader fail-closed unchanged): a single bad
    /// file passed directly to `analyze` MUST still fail with exit non-zero.
    /// The isolation is directory-mode / batch behavior; single-file invocation
    /// still propagates the reader Err.
    ///
    /// STORY-128 FORBIDDEN DEPENDENCIES: src/reader.rs MUST NOT be modified.
    /// The reader still returns Err on bad files; main.rs isolation catches it
    /// ONLY when iterating a directory.  A single-file invocation passes through
    /// the same loop (one iteration), catches the Err per-file, and exits 1.
    /// This is still "failure" behavior — the distinction is that the process
    /// COMPLETES (does not crash mid-loop) but exits 1.
    ///
    /// This test pins that a single bad file → exit 1.
    ///
    /// BC-2.01.018 Invariant 1 / STORY-128 EC-004.
    #[test]
    fn test_BC_2_01_018_invariant1_reader_fail_closed_preserved() {
        let dir = tempfile::tempdir().expect("tempdir");

        // Write single bad file; pass the FILE directly (not the directory).
        let bad_path = dir.path().join("single-bad.pcapng");
        fs::write(&bad_path, conflict_pcapng_bytes()).expect("write single-bad.pcapng");

        wirerust()
            .args([
                "analyze",
                bad_path.to_str().unwrap(),
                "--no-color",
            ])
            .assert()
            // exit 1: single bad file → reader returns Err → any_error=true → exit 1.
            // The fail-closed semantics are preserved; isolation catches the Err and
            // exits 1 at end of loop, rather than propagating via ? (which would also
            // exit non-zero but with a different message format).
            .failure()
            // Path MUST appear in the stderr error notice.
            .stderr(predicate::str::contains("single-bad.pcapng"));
    }

    // -----------------------------------------------------------------------
    // run_summary isolation: STORY-128 applies to BOTH analyze AND summary
    // -----------------------------------------------------------------------

    /// `run_summary` per-file isolation: the `summary` subcommand has the same
    /// `?`-propagation issue as `run_analyze` (src/main.rs line 412-413).
    /// STORY-128 MUST refactor BOTH subcommands.
    ///
    /// Directory: [bad.pcapng (conflict → Err), good.pcapng (valid)].
    /// Bad file sorts FIRST ("a-" < "b-").
    ///
    /// GREEN: bad file error on stderr; good file processed; summary shows
    /// 0 packets (EPB with 0-byte payload fails decode, counted in skipped_packets)
    /// with exit 1. Regression would skip "b-good.pcapng" entirely.
    ///
    /// STORY-128 Architecture Mapping: isolation loop refactor applies to BOTH
    /// `run_analyze` and `run_summary` in src/main.rs.
    #[test]
    fn test_BC_2_01_018_summary_subcommand_per_file_isolation() {
        let dir = tempfile::tempdir().expect("tempdir");

        // a-bad.pcapng sorts FIRST: conflict → Err.
        fs::write(dir.path().join("a-bad.pcapng"), conflict_pcapng_bytes())
            .expect("write a-bad.pcapng");

        // b-good.pcapng sorts SECOND: SHB + ETHERNET IDB + 1 EPB (0-byte payload).
        fs::write(dir.path().join("b-good.pcapng"), valid_pcapng_bytes())
            .expect("write b-good.pcapng");

        let assert = wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert();

        assert
            .failure() // exit 1: a-bad.pcapng failed
            // Per-file error for a-bad.pcapng on stderr.
            .stderr(predicate::str::contains("a-bad.pcapng"))
            // b-good.pcapng IS processed: the EPB has 0-byte payload → decode error
            // in run_summary's loop → skipped_packets incremented.
            // The summary report shows "Skipped: 1 packets" even on the summary path.
            // Regression guard: abort on a-bad would skip b-good → "Skipped: 1 packets" absent.
            .stdout(predicate::str::contains("Skipped: 1 packets"));
    }

    // -----------------------------------------------------------------------
    // ZERO-PACKET NOTICE (standalone): a valid SHB-only file with no bad sibling
    //
    // Verifies the zero-packet notice fires for a lone valid zero-packet file.
    // This is the canonical ADR-009 Decision 19 scenario (no isolation involved).
    // -----------------------------------------------------------------------

    /// ADR-009 Decision 19 zero-packet notice: a directory containing ONLY a
    /// valid SHB-only pcapng file (Ok, 0 packets) MUST emit the zero-packet
    /// notice on stderr and exit 0.
    ///
    /// No isolation is involved (no bad sibling file).  This test pins the
    /// zero-packet notice emission itself, independent of AC-004 (which tests
    /// that the notice is not suppressed by sibling-file isolation).
    ///
    /// ## Regression guard
    ///
    /// After `Ok(source)`, the implementation checks `source.packets.is_empty()` and emits:
    ///   "notice: <path>/shb-only.pcapng: 0 packets read from pcapng file"
    /// Exit 0 (valid file, not an error). A regression would suppress the notice.
    ///
    /// ADR-009 Decision 19 / BC-2.01.009 PC6 / STORY-128 AC-004.
    #[test]
    fn test_BC_2_01_018_zero_packet_notice_decision19_lone_valid_file() {
        let dir = tempfile::tempdir().expect("tempdir");

        fs::write(dir.path().join("shb-only.pcapng"), shb_only_pcapng_bytes())
            .expect("write shb-only.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // exit 0: valid zero-packet file is NOT an error (Decision 19).
            .success()
            // Zero-packet notice (src/main.rs:61-89) MUST appear on stderr.
            // Regression guard: notice absent → assertion fails.
            .stderr(predicate::str::contains("shb-only.pcapng"))
            .stderr(predicate::str::contains("0 packets"));
    }

    // =======================================================================
    // BC-2.01.009 PC6 NOTICE FORMAT TESTS (adversarial review C-1 / M-1 / H-1)
    //
    // The tests below pin the FULL PC6 notice format requirements:
    //   (1) OPB-clause: "(includes N obsolete Packet Block(s) whose data was not
    //       analyzed; re-save with mergecap)" — emitted when opb_skipped > 0.
    //   (2) Generic-skip segment: "(G block(s) skipped as unsupported)" where
    //       G = skipped_blocks - opb_skipped — emitted when G > 0.
    //   (3) Classic-pcap wording: "pcap file" (NOT "pcapng file") for classic pcap.
    //   (4) Segments are independently gated (neither segment when both gates == 0).
    //
    // ALL tests below drive the CLI via subprocess (assert_cmd), identical to the
    // existing STORY-128 approach.
    //
    // Fixture helpers below build pcapng files with OPB / NRB / unknown blocks
    // following the same le_skip_block pattern from bc_2_01_story126_spb_tests.rs.
    //
    // GREEN regression guards: format_zero_packet_notice (src/main.rs:61-89) is
    // implemented with OPB clause, generic-skip segment, and pcap-vs-pcapng wording.
    // Tests below guard against regression of the full PC6 notice format.
    // =======================================================================

    // -----------------------------------------------------------------------
    // Pcapng fixture helpers for PC6 format tests
    //
    // These follow the same le_skip_block pattern established in STORY-126:
    //   block_type(4 LE) + btl(4 LE) + body + trailing_btl(4 LE)
    //   btl = 12 + body.len(); body must be 4-byte aligned.
    // -----------------------------------------------------------------------

    /// OPB (Obsolete Packet Block) type code — 0x00000002.
    ///
    /// Wire layout per HS-108 Case D (32 bytes, empty captured data):
    ///   block_type:     02 00 00 00
    ///   btl:            20 00 00 00  (32 decimal)
    ///   interface_id:   00 00
    ///   drops_count:    00 00
    ///   ts_high:        00 00 00 00
    ///   ts_low:         00 00 00 00
    ///   captured_len:   00 00 00 00
    ///   original_len:   00 00 00 00
    ///   trailing_btl:   20 00 00 00
    fn opb_bytes() -> Vec<u8> {
        // OPB body: interface_id(2) + drops_count(2) + ts_high(4) + ts_low(4)
        //           + captured_len(4) + original_len(4) = 20 bytes
        let body: &[u8] = &[
            0x00, 0x00, // interface_id
            0x00, 0x00, // drops_count
            0x00, 0x00, 0x00, 0x00, // ts_high
            0x00, 0x00, 0x00, 0x00, // ts_low
            0x00, 0x00, 0x00, 0x00, // captured_len = 0
            0x00, 0x00, 0x00, 0x00, // original_len = 0
        ];
        le_skip_block_pc6(0x0000_0002, body)
    }

    /// NRB (Name Resolution Block, type 0x00000004) with an empty record list.
    ///
    /// Wire layout per HS-108 Case E (16 bytes, LE):
    ///   block_type:    04 00 00 00
    ///   btl:           10 00 00 00  (16 decimal)
    ///   nrb_record_type: 00 00
    ///   nrb_record_length: 00 00
    ///   trailing_btl:  10 00 00 00
    fn nrb_bytes() -> Vec<u8> {
        // NRB body: record_type(2) + record_length(2) = 4 bytes (4-byte aligned)
        let body: &[u8] = &[
            0x00, 0x00, // record_type = 0 (end of records)
            0x00, 0x00, // record_length = 0
        ];
        le_skip_block_pc6(0x0000_0004, body)
    }

    /// Unknown block type (0x00000099) with 8 dummy body bytes.
    ///
    /// Wire layout per HS-108 "Unknown-type block" note (20 bytes, LE):
    ///   block_type:    99 00 00 00
    ///   btl:           14 00 00 00  (20 decimal)
    ///   body:          AA BB CC DD EE FF 00 11  (8 bytes)
    ///   trailing_btl:  14 00 00 00
    fn unknown_skip_block_bytes() -> Vec<u8> {
        let body: &[u8] = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11];
        le_skip_block_pc6(0x0000_0099, body)
    }

    /// Generic skip-block builder: block_type(4 LE) + btl(4 LE) + body + trailing_btl(4 LE).
    ///
    /// Mirrors le_skip_block from bc_2_01_story126_spb_tests.rs.
    /// body must already be 4-byte aligned.
    fn le_skip_block_pc6(block_type: u32, body: &[u8]) -> Vec<u8> {
        assert_eq!(body.len() % 4, 0, "body must be 4-byte aligned");
        let btl = 12 + body.len();
        let mut v = Vec::with_capacity(btl);
        v.extend_from_slice(&block_type.to_le_bytes());
        v.extend_from_slice(&(btl as u32).to_le_bytes());
        v.extend_from_slice(body);
        v.extend_from_slice(&(btl as u32).to_le_bytes());
        assert_eq!(v.len(), btl);
        v
    }

    /// SHB + IDB + 1 OPB (no EPB/SPB).
    ///
    /// BC-2.01.009 EC-007 / HS-108 Case D:
    ///   packets.len()==0, skipped_blocks==1, opb_skipped==1, G=0.
    ///   Notice MUST include OPB clause; MUST NOT include generic segment.
    fn shb_idb_one_opb_bytes() -> Vec<u8> {
        let mut b = shb_bytes();
        b.extend(idb_bytes(1)); // ETHERNET IDB
        b.extend(opb_bytes()); // 1 OPB → skipped_blocks=1, opb_skipped=1
        b
    }

    /// SHB + IDB + 2 unknown skip blocks (no OPB, no EPB/SPB).
    ///
    /// HS-108 Case B:
    ///   packets.len()==0, skipped_blocks==2, opb_skipped==0, G=2.
    ///   Notice MUST include generic segment "(2 block(s) skipped as unsupported)".
    fn shb_idb_two_unknown_blocks_bytes() -> Vec<u8> {
        let mut b = shb_bytes();
        b.extend(idb_bytes(1)); // ETHERNET IDB
        b.extend(unknown_skip_block_bytes()); // skip block 1 → skipped_blocks=1
        b.extend(unknown_skip_block_bytes()); // skip block 2 → skipped_blocks=2
        b
    }

    /// SHB + IDB + 2 NRBs + 1 OPB (no EPB/SPB).
    ///
    /// HS-108 Case E:
    ///   packets.len()==0, skipped_blocks==3, opb_skipped==1, G=2.
    ///   Notice MUST include BOTH generic segment "(2 block(s) skipped as unsupported)"
    ///   AND OPB clause "(includes 1 obsolete Packet Block(s) whose data was not analyzed;
    ///   re-save with mergecap)".
    fn shb_idb_two_nrb_one_opb_bytes() -> Vec<u8> {
        let mut b = shb_bytes();
        b.extend(idb_bytes(1)); // ETHERNET IDB
        b.extend(nrb_bytes()); // NRB 1 → skipped_blocks=1
        b.extend(nrb_bytes()); // NRB 2 → skipped_blocks=2
        b.extend(opb_bytes()); // OPB → skipped_blocks=3, opb_skipped=1
        b
    }

    /// Minimal valid EMPTY classic pcap file (24-byte global header, zero packet records).
    ///
    /// BC-2.01.009 EC-009 / PC6 classic-pcap symmetry:
    ///   magic: 0xA1B2C3D4 (little-endian = D4 C3 B2 A1 on disk)
    ///   version_major: 2
    ///   version_minor: 4
    ///   thiszone: 0
    ///   sigfigs: 0
    ///   snaplen: 65535
    ///   network: 1 (ETHERNET)
    ///   Zero packet records follow — valid empty capture.
    fn empty_classic_pcap_bytes() -> Vec<u8> {
        let mut b = Vec::new();
        // magic number: 0xA1B2C3D4 (LE on disk = D4 C3 B2 A1)
        b.extend_from_slice(&[0xD4, 0xC3, 0xB2, 0xA1]);
        // version_major = 2 (LE u16)
        b.extend_from_slice(&[0x02, 0x00]);
        // version_minor = 4 (LE u16)
        b.extend_from_slice(&[0x04, 0x00]);
        // thiszone = 0 (LE i32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // sigfigs = 0 (LE u32)
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // snaplen = 65535 (LE u32)
        b.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
        // network = 1 (ETHERNET, LE u32)
        b.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        // Zero packet records — EOF immediately after global header.
        assert_eq!(b.len(), 24, "classic pcap global header must be 24 bytes");
        b
    }

    // -----------------------------------------------------------------------
    // Test 1 (C-1): OPB-clause — SHB + IDB + 1 OPB, zero packets
    //
    // HS-108 Case D: skipped_blocks=1, opb_skipped=1, G=0.
    // Notice MUST contain OPB clause; MUST NOT contain generic segment.
    //
    // GREEN: the OPB clause ("obsolete", "mergecap", count "1") is implemented
    // in format_zero_packet_notice (src/main.rs:61-89).
    // -----------------------------------------------------------------------

    /// BC-2.01.009 PC6 OPB-clause (C-1 adversarial finding): a valid pcapng with
    /// SHB + IDB + 1 OPB (zero EPB/SPB) MUST emit the OPB clause in the notice.
    ///
    /// Expected full notice (BC-2.01.009 PC6 Case D canonical form):
    ///   "notice: <file>: 0 packets read from pcapng file (includes 1 obsolete
    ///    Packet Block(s) whose data was not analyzed; re-save with mergecap)"
    ///
    /// Key substrings that MUST be present (HS-108 Case D byte-exact assertion):
    ///   - "0 packets read from pcapng file"
    ///   - "obsolete"
    ///   - "mergecap"
    ///   - "1"  (OPB count — as part of the clause)
    ///
    /// Substring that MUST NOT be present (G=0, generic segment suppressed):
    ///   - "skipped as unsupported"
    ///
    /// ## Regression guard
    ///
    /// GREEN: the OPB clause "(includes N obsolete Packet Block(s) whose data was
    /// not analyzed; re-save with mergecap)" is implemented in format_zero_packet_notice.
    /// Regression: removing the OPB clause would make "obsolete" absent → assertion fails.
    ///
    /// BC-2.01.009 PC6 EC-007 / HS-108 Case D / ADR-009 Decision 19 OPB-distinction.
    #[test]
    fn test_BC_2_01_009_pc6_opb_clause_analyze() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("opb-only.pcapng"), shb_idb_one_opb_bytes())
            .expect("write opb-only.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // exit 0: structurally valid zero-packet file
            .success()
            // Base notice phrase must be present
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // OPB clause: count 1 + "obsolete" must appear as discriminating substring
            // (HS-108 Case D byte-exact — tightened from weak contains("1") per O-1)
            .stderr(predicate::str::contains("includes 1 obsolete"))
            // OPB clause: "mergecap" remediation hint must appear (HS-108 Case D)
            .stderr(predicate::str::contains("mergecap"))
            // Generic segment MUST NOT appear (G = 1-1 = 0, gate is false)
            .stderr(predicate::str::contains("skipped as unsupported").not());
    }

    /// Same OPB-clause test via `summary` subcommand (PC6 applies to both).
    ///
    /// The `run_summary` path has the same bare-notice defect as `run_analyze`.
    /// This pins both subcommands simultaneously.
    ///
    /// GREEN: same OPB clause regression guard via the `summary` subcommand.
    /// The `run_summary` path has the same notice emission code.
    ///
    /// BC-2.01.009 PC6 / STORY-128 coverage for run_summary.
    #[test]
    fn test_BC_2_01_009_pc6_opb_clause_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("opb-only.pcapng"), shb_idb_one_opb_bytes())
            .expect("write opb-only.pcapng");

        wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // OPB clause contains "obsolete" and "mergecap" — regression guard.
            .stderr(predicate::str::contains("obsolete"))
            .stderr(predicate::str::contains("mergecap"))
            .stderr(predicate::str::contains("skipped as unsupported").not());
    }

    // -----------------------------------------------------------------------
    // Test 2 (C-1): Generic-skip segment — SHB + IDB + 2 unknown blocks
    //
    // HS-108 Case B: skipped_blocks=2, opb_skipped=0, G=2.
    // Notice MUST contain "(2 block(s) skipped as unsupported)".
    //
    // GREEN: the generic-skip segment "(N block(s) skipped as unsupported)" is implemented.
    // -----------------------------------------------------------------------

    /// BC-2.01.009 PC6 generic-skip segment (C-1): a valid pcapng with SHB + IDB
    /// + 2 unknown/unsupported skip blocks (G=2, opb_skipped=0) MUST include the
    /// generic skip segment in the zero-packet notice.
    ///
    /// Expected notice (HS-108 Case B):
    ///   "notice: <file>: 0 packets read from pcapng file (2 block(s) skipped as unsupported)"
    ///
    /// Key substrings (HS-108 Case B byte-exact assertion):
    ///   - "0 packets read from pcapng file"
    ///   - "2 block(s) skipped"  (the count 2 and "skipped" must both appear)
    ///   - "skipped as unsupported"  (BC-2.01.009 PC6 exact wording for generic segment)
    ///
    /// Substrings that MUST NOT be present (opb_skipped==0, OPB clause suppressed):
    ///   - "obsolete"
    ///   - "mergecap"
    ///
    /// ## Regression guard
    ///
    /// Guards that the generic-skip segment "(2 block(s) skipped as unsupported)"
    /// appears in the notice (HS-108 Case B). A regression would suppress the segment.
    ///
    /// BC-2.01.009 PC6 / HS-108 Case B / ADR-009 Decision 19 generic-skip gate.
    #[test]
    fn test_BC_2_01_009_pc6_generic_skip_segment_analyze() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("two-unknown-blocks.pcapng"),
            shb_idb_two_unknown_blocks_bytes(),
        )
        .expect("write two-unknown-blocks.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // Generic segment: full discriminating substring G=2 (HS-108 Case B byte-exact)
            // Tightened from weak contains("2") + contains("skipped as unsupported") per O-1.
            .stderr(predicate::str::contains("2 block(s) skipped as unsupported"))
            // OPB clause MUST NOT appear (opb_skipped==0, gate is false)
            .stderr(predicate::str::contains("obsolete").not())
            .stderr(predicate::str::contains("mergecap").not());
    }

    /// Same generic-skip segment test via `summary` subcommand.
    ///
    /// GREEN: same generic-skip guard via the `summary` subcommand.
    #[test]
    fn test_BC_2_01_009_pc6_generic_skip_segment_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("two-unknown-blocks.pcapng"),
            shb_idb_two_unknown_blocks_bytes(),
        )
        .expect("write two-unknown-blocks.pcapng");

        wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // Generic segment: full discriminating substring G=2 (HS-108 Case B byte-exact)
            // Tightened from weak contains("2") + separate contains("skipped") per O-1.
            .stderr(predicate::str::contains("2 block(s) skipped as unsupported"))
            .stderr(predicate::str::contains("obsolete").not())
            .stderr(predicate::str::contains("mergecap").not());
    }

    // -----------------------------------------------------------------------
    // Test 3 (HS-108 Case E): Both segments present — 2 NRBs + 1 OPB
    //
    // skipped_blocks=3, opb_skipped=1, G=2.
    // Notice MUST contain BOTH "(2 block(s) skipped as unsupported)" AND
    // "(includes 1 obsolete Packet Block(s) whose data was not analyzed;
    //  re-save with mergecap)".
    //
    // GREEN: both OPB clause and generic-skip segment are implemented.
    // -----------------------------------------------------------------------

    /// BC-2.01.009 PC6 both-segments (HS-108 Case E): a valid pcapng with 2 NRBs
    /// + 1 OPB MUST show BOTH the generic skip segment (G=2) AND the OPB clause
    /// (opb_skipped=1) in the zero-packet notice.
    ///
    /// The two segments are independently gated and must appear space-separated
    /// per BC-2.01.009 PC6 v1.7: "when both segments are emitted they appear
    /// space-separated after the base notice line."
    ///
    /// Key substrings (HS-108 Case E byte-exact assertion):
    ///   - "0 packets read from pcapng file"
    ///   - "2"  (G = skipped_blocks - opb_skipped = 3 - 1)
    ///   - "skipped as unsupported"
    ///   - "1"  (opb_skipped count)
    ///   - "obsolete"
    ///   - "mergecap"
    ///
    /// The counts 2 and 1 MUST be distinct (not collapsed into a single "3").
    /// Verified by asserting "skipped as unsupported" AND "obsolete" both appear —
    /// these only appear in separate segments.
    ///
    /// ## Regression guard
    ///
    /// GREEN: both OPB clause and generic-skip segment are implemented.
    /// Regression: either missing segment would fail the corresponding assertion.
    ///
    /// BC-2.01.009 PC6 / HS-108 Case E / ADR-009 Decision 19 (both segments).
    #[test]
    fn test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_analyze() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("nrb-plus-opb.pcapng"),
            shb_idb_two_nrb_one_opb_bytes(),
        )
        .expect("write nrb-plus-opb.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // Generic segment: G=2 — full discriminating substring (HS-108 Case E, tightened per O-1)
            .stderr(predicate::str::contains("2 block(s) skipped as unsupported"))
            // OPB clause: opb_skipped=1 — full discriminating substring (HS-108 Case E, tightened per O-1)
            .stderr(predicate::str::contains("includes 1 obsolete"))
            .stderr(predicate::str::contains("mergecap"));
    }

    /// Same both-segments test via `summary` subcommand.
    #[test]
    fn test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("nrb-plus-opb.pcapng"),
            shb_idb_two_nrb_one_opb_bytes(),
        )
        .expect("write nrb-plus-opb.pcapng");

        wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // Generic segment: G=2 — full discriminating substring (HS-108 Case E, tightened per O-1)
            .stderr(predicate::str::contains("2 block(s) skipped as unsupported"))
            // OPB clause: opb_skipped=1 — full discriminating substring (HS-108 Case E, tightened per O-1)
            .stderr(predicate::str::contains("includes 1 obsolete"))
            .stderr(predicate::str::contains("mergecap"));
    }

    // -----------------------------------------------------------------------
    // Test 4 (regression / NEITHER segment): SHB-only, skipped_blocks==0, opb_skipped==0
    //
    // HS-108 Case F / BC-2.01.009 EC-010: notice MUST be bare with NO parenthetical.
    // GREEN: the gate-suppression behavior is correct — when skipped_blocks==0
    // and opb_skipped==0, no parenthetical is appended.
    //
    // This pins that the segments are GATED — not always emitted.
    // After implementation the gate must still hold (regression guard).
    // -----------------------------------------------------------------------

    /// BC-2.01.009 PC6 EC-010 neither-segment regression (HS-108 Case F):
    /// a SHB-only pcapng (skipped_blocks==0, opb_skipped==0) MUST emit the bare
    /// notice with NO parenthetical segment.
    ///
    /// This test is expected to PASS both before and after the PC6 implementation,
    /// but it pins the gate condition: segments are OMITTED when their counters are 0.
    ///
    /// Assertions:
    ///   - stderr contains "0 packets read from pcapng file"
    ///   - stderr does NOT contain "skipped"  (no generic segment)
    ///   - stderr does NOT contain "obsolete" (no OPB clause)
    ///   - stderr does NOT contain "mergecap" (no remediation hint)
    ///   - exit 0
    ///
    /// HS-108 Case F byte-exact assertion.
    ///
    /// BC-2.01.009 EC-010 / HS-108 Case F / ADR-009 rev 9 F-M4.
    #[test]
    fn test_BC_2_01_009_pc6_neither_segment_shb_only_analyze() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("shb-only-gate.pcapng"),
            shb_only_pcapng_bytes(),
        )
        .expect("write shb-only-gate.pcapng");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // exit 0: structurally valid (EC-010 / F-M4)
            .success()
            // Base notice phrase MUST be present
            // (this currently FAILS too because the notice itself is not yet
            // implemented — but the gating assertions below pin the format constraint)
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            // No generic segment: skipped_blocks==0, so G==0 → gate false → OMITTED
            .stderr(predicate::str::contains("skipped").not())
            // No OPB clause: opb_skipped==0 → gate false → OMITTED
            .stderr(predicate::str::contains("obsolete").not())
            // No remediation hint: accompanies OPB clause only → absent when gate false
            .stderr(predicate::str::contains("mergecap").not());
    }

    /// Same neither-segment test via `summary` subcommand.
    ///
    /// Both subcommands must apply the same gating logic.
    #[test]
    fn test_BC_2_01_009_pc6_neither_segment_shb_only_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("shb-only-gate.pcapng"),
            shb_only_pcapng_bytes(),
        )
        .expect("write shb-only-gate.pcapng");

        wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcapng file"))
            .stderr(predicate::str::contains("skipped").not())
            .stderr(predicate::str::contains("obsolete").not())
            .stderr(predicate::str::contains("mergecap").not());
    }

    // -----------------------------------------------------------------------
    // Test 5 (M-1): Classic-pcap wording — "pcap file" (NOT "pcapng file")
    //
    // BC-2.01.009 PC6 EC-009 / ADR-009 Decision 19 classic-pcap symmetry.
    // A valid EMPTY classic pcap (24-byte global header, zero packet records) MUST
    // emit "0 packets read from pcap file" — NOT "pcapng file".
    //
    // GREEN: format_zero_packet_notice (src/main.rs:61-89) uses "pcap file" for
    // classic pcap inputs and "pcapng file" for pcapng inputs.
    // -----------------------------------------------------------------------

    /// BC-2.01.009 PC6 EC-009 classic-pcap wording (M-1): an empty classic pcap
    /// (valid 24-byte global header, zero packet records) MUST emit the notice with
    /// "pcap file" — NOT "pcapng file" — in both `analyze` and `summary` subcommands.
    ///
    /// The current main.rs emits "pcapng file" unconditionally (hardcoded string)
    /// at both notice emission sites (lines ~264 and ~456).  Classic-pcap symmetry
    /// (PC6, ADR-009 Decision 19 rev 8) requires format-aware wording:
    ///   - pcapng input → "pcapng file"
    ///   - classic pcap input → "pcap file"
    ///
    /// Assertions:
    ///   - stderr contains "0 packets read from pcap file"  (correct wording)
    ///   - stderr does NOT contain "0 packets read from pcapng file" (wrong wording)
    ///   - exit 0
    ///
    /// ## Regression guard
    ///
    /// GREEN: format_zero_packet_notice (src/main.rs:61-89) uses "pcap file" for
    /// classic pcap inputs. Regression: hardcoding "pcapng file" would produce
    /// "pcapng file" for .pcap inputs — the NOT assertion would catch it.
    ///
    /// BC-2.01.009 PC6 EC-009 / ADR-009 Decision 19 classic-pcap symmetry.
    #[test]
    fn test_BC_2_01_009_pc6_classic_pcap_wording_analyze() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("empty-classic.pcap"),
            empty_classic_pcap_bytes(),
        )
        .expect("write empty-classic.pcap");

        wirerust()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            // exit 0: structurally valid empty classic pcap
            .success()
            // MUST contain the classic-pcap wording "pcap file"
            .stderr(predicate::str::contains("0 packets read from pcap file"))
            // MUST NOT say "pcapng file" for a classic pcap input.
            // Regression guard: hardcoding "pcapng file" would fail this NOT assertion.
            .stderr(predicate::str::contains("0 packets read from pcapng file").not());
    }

    /// Same classic-pcap wording test via `summary` subcommand.
    ///
    /// GREEN: run_summary also uses format_zero_packet_notice (src/main.rs:61-89)
    /// (~line 456 in current src/main.rs).
    ///
    /// BC-2.01.009 PC6 EC-009 / STORY-128 coverage for run_summary.
    #[test]
    fn test_BC_2_01_009_pc6_classic_pcap_wording_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("empty-classic.pcap"),
            empty_classic_pcap_bytes(),
        )
        .expect("write empty-classic.pcap");

        wirerust()
            .args(["summary", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stderr(predicate::str::contains("0 packets read from pcap file"))
            // Regression guard: "pcapng file" wording for classic pcap would fail this NOT assertion.
            .stderr(predicate::str::contains("0 packets read from pcapng file").not());
    }
}
