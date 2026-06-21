//! STORY-128: main.rs Per-File Error Isolation Loop (Catch-and-Continue)
//!
//! TDD RED-GATE suite — all isolation tests in this file are RED against the
//! pre-refactor loop.  The implementer must:
//!   1. Add `let mut any_error = false;` before the per-file loop in `run_analyze`
//!      (and analogously in `run_summary`).
//!   2. Replace the `?`-propagation on `PcapSource::from_file(path).with_context(...)?`
//!      with an explicit `match`:
//!      - `Ok(source)` arm: existing processing (zero-packet notice check, analyzer dispatch).
//!      - `Err(e)` arm: `eprintln!("error: {}: {e:#}", path.display()); any_error = true; continue;`
//!   3. After the loop, add: `if any_error { std::process::exit(1); }` before `Ok(())`.
//!   4. Emit the zero-packet notice (Decision 19) in the `Ok` arm when
//!      `source.packets.is_empty()`:
//!      `eprintln!("notice: {}: 0 packets read from pcapng file", path.display());`
//!   NOTE: `src/reader.rs` MUST NOT be modified (STORY-128 Forbidden Dependencies).
//!
//! ## RED Gate Explanation (pre-refactor current state)
//!
//! Current `run_analyze` wraps the per-file loop in an immediately-invoked closure
//! and propagates reader errors via `?` (src/main.rs line 243-244):
//!
//!   ```rust
//!   let source = PcapSource::from_file(path)
//!       .with_context(|| format!("Failed to read {}", path.display()))?;
//!   ```
//!
//! This means the FIRST bad file causes the closure to return `Err`, which is
//! stored in `capture_result` and then propagated at line 315 (`capture_result?`).
//! All remaining files in the directory are NEVER processed.
//!
//! Current `run_summary` has the same pattern with a direct `?` at line 412-413.
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
    // Pre-refactor RED: the `?` on `from_file(...)?` aborts the closure on
    // file_a's Err.  If file_a sorts BEFORE file_b, file_b is never processed.
    // The test asserts catch-and-continue behavior → RED pre-implementation.
    // -----------------------------------------------------------------------

    /// AC-001 / BC-2.01.018 AC-002: directory with [bad.pcap (E-INP-011
    /// conflict), good.pcapng (valid)] sorted alphabetically so the bad file
    /// processes first.  The batch MUST complete; the good file MUST be analyzed;
    /// exit code MUST be 1; the bad file MUST produce an error notice on stderr.
    ///
    /// ## Pre-refactor RED gate
    ///
    /// Files sorted: "a-conflict.pcapng" < "b-valid.pcapng" (alphabetic LE).
    /// Current code: `from_file("a-conflict.pcapng")?` → Err(E-INP-011) →
    /// closure returns Err → `capture_result?` propagates → `run_analyze` returns
    /// Err early.  "b-valid.pcapng" is NEVER processed.
    ///
    /// The assertion `stdout contains "Packets: 1"` FAILS because the packet from
    /// "b-valid.pcapng" is never counted: the run aborted after "a-conflict.pcapng".
    ///
    /// ## Expected post-refactor behavior
    ///
    /// match on `from_file("a-conflict.pcapng")` → Err arm:
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
        // Sorts BEFORE b-valid.pcapng to maximize RED gate discrimination:
        // current code aborts on this file, never reaching b-valid.pcapng.
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

        // RED: current code exits non-zero (aborts after a-conflict.pcapng Err)
        // AND stdout does NOT contain "Skipped: 1 packets" because b-valid.pcapng
        // is never processed.
        //
        // Post-refactor: exit 1 (any_error=true), b-valid.pcapng IS processed
        // (1 raw packet with 0-byte payload → decode error → Skipped: 1 packets),
        // AND stderr contains the a-conflict.pcapng error notice.
        //
        // Discriminating assertion: stderr contains the path-prefixed error for
        // a-conflict.pcapng.  This is RED because current code propagates the
        // Err to run_analyze (which returns Err to main → anyhow error chain,
        // NOT the "error: <path>: ..." per-file format required by AC-001).
        assert
            // Exit 1: any_error=true (a-conflict.pcapng failed)
            .failure()
            // Per-file error notice for the bad file on stderr:
            //   "error: <path>/a-conflict.pcapng: ..."
            // RED: current code emits the anyhow error chain without the
            // "error: <path>:" prefix format (it's a top-level bail, not
            // a per-file eprintln).
            .stderr(predicate::str::contains("a-conflict.pcapng"))
            // The good file's packet MUST be counted in the final summary.
            // "Skipped: 1 packets" or "Packets: 1" depending on decode result.
            // Using "Skipped: 1" as the discriminator: current code never reaches
            // b-valid.pcapng, so "Skipped: 1" is absent from stdout.
            // RED: current code aborts before processing b-valid.pcapng →
            // "Skipped: 1" is NOT in stdout → this assertion FAILS under current code.
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
    /// ## Pre-refactor RED gate
    ///
    /// Files sorted: "1-conflict.pcapng" < "2-good.pcapng".
    /// Current code: aborts on "1-conflict.pcapng" Err → "2-good.pcapng" never
    /// processed → stdout "Skipped: 1 packets" absent → test FAILS.
    ///
    /// ## Expected post-refactor behavior
    ///
    /// Err arm for "1-conflict.pcapng": emits "error: <path>: ... link-type
    /// conflict ..." to stderr; any_error=true; continue.
    /// Ok arm for "2-good.pcapng": processes 1 EPB (decode error on 0-byte
    /// payload → Skipped: 1 packets in stdout).
    /// exit 1 (any_error=true).
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
            // RED: current code never reaches 2-good.pcapng.
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
    /// ## Pre-refactor RED gate
    ///
    /// Files sorted: "a-truncated.pcapng" < "b-valid.pcapng".
    /// Current code: from_file("a-truncated.pcapng") → Err(E-INP-008) →
    /// `?` propagates → run_analyze returns Err early.
    /// "b-valid.pcapng" never processed → "Skipped: 1 packets" absent → FAILS.
    ///
    /// ## Expected post-refactor behavior
    ///
    /// Err arm for "a-truncated.pcapng": eprintln per-file error; continue.
    /// Ok arm for "b-valid.pcapng": 1 EPB counted (decode error); Skipped: 1.
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
            // b-valid.pcapng IS processed: 1 EPB counted as decode error.
            // RED: current code never reaches b-valid.pcapng.
            .stdout(predicate::str::contains("Skipped: 1 packets"));
    }

    // -----------------------------------------------------------------------
    // AC-004: Zero-packet notice not suppressed by isolation
    //
    // ADR-009 Decision 19: a valid SHB-only file with 0 packets MUST emit the
    // zero-packet notice.  The isolation Err arm for a sibling bad file MUST NOT
    // suppress the zero-packet notice from the Ok arm of the SHB-only file.
    //
    // Pre-refactor RED: (a) the zero-packet notice is not emitted at all
    // (it's part of STORY-128's implementation), AND (b) if the bad file sorts
    // first, the SHB-only file is never reached.  Either condition makes this RED.
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
    /// ## Pre-refactor RED gate
    ///
    /// RED reason 1: zero-packet notice is NOT implemented in main.rs yet
    ///   (STORY-128's implementation adds it) → "0 packets" absent from stderr.
    /// RED reason 2: current code aborts on "a-conflict.pcapng" → "b-shb-only.pcapng"
    ///   never processed → notice never emitted → doubly RED.
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
            // RED reason 1: notice not yet implemented → "0 packets" absent.
            // RED reason 2: current code aborts before reaching b-shb-only.pcapng.
            .stderr(predicate::str::contains("b-shb-only.pcapng"))
            .stderr(predicate::str::contains("0 packets"));
    }

    // -----------------------------------------------------------------------
    // ORDER INDEPENDENCE: bad file FIRST vs. LAST vs. MIDDLE
    //
    // These are the strongest isolation discriminators: if the bad file is
    // first in sort order and current code aborts, all subsequent good files
    // are never processed.  All three orderings MUST produce the same outcome:
    // good files analyzed, bad file error on stderr, exit 1.
    // -----------------------------------------------------------------------

    /// ORDER INDEPENDENCE (bad file FIRST): directory sorted so the bad file
    /// ("a-bad.pcapng") sorts BEFORE the good files ("b-good.pcapng",
    /// "c-good.pcapng").  Both good files MUST be processed.
    ///
    /// This is the canonical isolation discriminator: current code aborts after
    /// "a-bad.pcapng" → "b-good.pcapng" and "c-good.pcapng" never processed.
    ///
    /// Post-refactor: isolation catches "a-bad.pcapng" Err; "b-good.pcapng"
    /// and "c-good.pcapng" both processed (1 packet each); Skipped: 2 packets.
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
            // RED: current code aborts after a-bad.pcapng → 0 packets processed.
            .stdout(predicate::str::contains("Skipped: 2 packets"));
    }

    /// ORDER INDEPENDENCE (bad file LAST): directory sorted so the bad file
    /// ("c-bad.pcapng") sorts AFTER the good files ("a-good.pcapng",
    /// "b-good.pcapng").  Both good files are processed before the bad file.
    ///
    /// This test is PARTIALLY GREEN under the current code: the good files
    /// ARE processed before "c-bad.pcapng" (sort order means they execute
    /// first).  But the exit code and error notice format differ:
    ///   - Current code: "c-bad.pcapng" causes run_analyze to return Err →
    ///     anyhow top-level error printed → exit 1.  The "Skipped: 2" DOES
    ///     appear in stdout (good files processed first).  But the error
    ///     message format is the anyhow chain, NOT the per-file "error: <path>:"
    ///     prefix format required by AC-001.
    ///
    /// The discriminating assertion is that stderr contains the per-file
    /// "error: <path>: ..." format (not just a raw anyhow chain dump).
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
            // RED: current code emits the anyhow chain via main's error propagation,
            // NOT the per-file "error: <path>:" eprintln format.
            .stderr(predicate::str::contains("c-bad.pcapng"))
            // Both good files processed BEFORE the bad file:
            // 2 EPBs (0-byte payload) → 2 decode errors → Skipped: 2.
            .stdout(predicate::str::contains("Skipped: 2 packets"));
    }

    /// ORDER INDEPENDENCE (bad file MIDDLE): directory sorted so the bad file
    /// ("b-bad.pcapng") sorts BETWEEN two good files ("a-good.pcapng",
    /// "c-good.pcapng").
    ///
    /// Post-refactor: "a-good.pcapng" processed; "b-bad.pcapng" caught and
    /// isolated; "c-good.pcapng" processed.  Both good files contribute to
    /// "Skipped: 2 packets".
    ///
    /// RED: current code aborts after "b-bad.pcapng" → "c-good.pcapng" never
    /// processed → "Skipped: 2 packets" absent (only 1 packet from a-good).
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
            // RED: current code aborts on b-bad → c-good never processed → Skipped: 1.
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
    /// ## Pre-refactor behavior
    ///
    /// Current code: first bad file → Err → `?` propagates → run_analyze returns
    /// Err early.  Second bad file never reaches the error arm.  The PROCESS
    /// COMPLETES (no crash) but only the first error is reported (via anyhow),
    /// NOT as a per-file "error: <path>:" notice.
    ///
    /// ## Expected post-refactor behavior
    ///
    /// Both bad files processed through the Err arm:
    ///   - eprintln("error: a-bad.pcapng: ..."); any_error=true; continue.
    ///   - eprintln("error: b-bad.pcapng: ..."); any_error=true; continue.
    /// After loop: std::process::exit(1).
    /// Both paths appear in stderr.
    ///
    /// The discriminating assertion: BOTH paths appear in stderr.
    /// RED: current code only reports the first bad file (second is never reached).
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
            // RED: current code aborts after "a-bad.pcapng" → "b-bad.pcapng"
            // never processed → only "a-bad.pcapng" in stderr.
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
    /// Pre-refactor RED: `summary` exits non-zero AND "b-good.pcapng" is never
    /// processed → "Packets: 0" or no "Skipped" entry for the good file.
    ///
    /// Post-refactor: bad file error on stderr; good file processed;
    /// summary shows 0 packets (EPB with 0-byte payload fails decode in
    /// run_summary's decode loop, counted in skipped_packets) with exit 1.
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
            // RED: current code aborts on a-bad → b-good never processed → Skipped absent.
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
    /// ## Pre-refactor RED gate
    ///
    /// The zero-packet notice is not yet emitted by main.rs (STORY-128 adds it).
    /// Current code: valid SHB-only file → Ok arm → reader loop executes (0 packets)
    /// → no notice emitted → "0 packets" absent from stderr → test FAILS.
    ///
    /// ## Expected post-refactor behavior
    ///
    /// After `Ok(source)` in the match arm, post-refactor code checks
    /// `source.packets.is_empty()` and emits:
    ///   "notice: <path>/shb-only.pcapng: 0 packets read from pcapng file"
    /// Exit 0 (valid file, not an error).
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
            // Zero-packet notice MUST appear on stderr.
            // RED: current code does not emit this notice → FAILS.
            .stderr(predicate::str::contains("shb-only.pcapng"))
            .stderr(predicate::str::contains("0 packets"));
    }
}
