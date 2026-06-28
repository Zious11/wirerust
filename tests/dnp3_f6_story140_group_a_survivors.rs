//! F6 Mutation-Hardening Tests — STORY-140 Group A Survivors (11 mutants).
//!
//! These tests target the 11 Group-A mutants that survived the prior cargo-mutants
//! re-run on `src/analyzer/dnp3.rs`.  The existing tests in
//! `dnp3_f6_story140_mutation_tests.rs` were structurally correct but did not
//! construct inputs that DISCRIMINATE the specific operator mutations listed below.
//!
//! ## The 11 Surviving Mutants
//!
//! | Line:col | Mutation | Why prior test did not kill |
//! |----------|----------|-----------------------------|
//! | 409:53   | `-` → `+` in `remaining_capacity = MAX_DNP3_FRAME_LEN - carry.len()` | See note below |
//! | 410:23   | `>` → `>=` in `data.len() > remaining_capacity` | exact-fit case not tested |
//! | 410:23   | `>` → `==` in `data.len() > remaining_capacity` | multi-byte delivery not distinguished |
//! | 467:25   | `&&` → `\|\|` in `is_sync = byte[0]==0x05 && byte[1]==0x64` | both junk bytes, || also false |
//! | 479:49   | `==` → `!=` on `w[0]` in sub-10 resync find | carry had no embedded partial sync |
//! | 479:57   | `&&` → `\|\|` in sub-10 resync find | carry had no byte equal to 0x64 at w[1] |
//! | 479:65   | `==` → `!=` on `w[1]` in sub-10 resync find | carry had no embedded partial sync |
//! | 511:53   | `==` → `!=` on `w[0]` in sub-10 did_process resync find | arm not exercised |
//! | 511:61   | `&&` → `\|\|` in sub-10 did_process resync find | arm not exercised |
//! | 511:69   | `==` → `!=` on `w[1]` in sub-10 did_process resync find | arm not exercised |
//! | 555:17   | `\|\|` → `&&` in ≥10-byte sync-gate predicate | carry had both bytes wrong |
//!
//! ## Kill Strategy Summary
//!
//! - **409 (`-→+`)**: carry=290, deliver 5 bytes. `remaining_capacity = 292-290 = 2`.
//!   Mutant: `remaining_capacity = 292+290 = 582`. `5 > 582` = false → no overflow,
//!   parse_errors stays 0. Correct: `5 > 2` = true → parse_errors = 1.
//!   *Note: the prior test_BC_2_15_016_overflow_arm_parse_errors_and_malformed should
//!   already kill this, but it sets carry via direct mutation then calls on_data with
//!   fresh data — if it passes under the mutant, it means cargo-mutants is not applying
//!   this mutation to the version under test.  The test below is a clean reproduction.*
//!
//! - **410 `>→>=`**: carry=289 bytes, deliver 3 bytes (exact fit: 292-289=3).
//!   Correct `>`: `3 > 3` = false → no overflow, parse_errors = 0.
//!   Mutant `>=`: `3 >= 3` = true → overflow, parse_errors = 1.
//!   Assert parse_errors == 0.
//!
//! - **410 `>→==`**: carry=289, deliver 4 bytes. `remaining_capacity = 3`.
//!   Correct `>`: `4 > 3` = true → overflow, parse_errors = 1.
//!   Mutant `==`: `4 == 3` = false → no overflow, parse_errors = 0.
//!   Assert parse_errors == 1.
//!   (This distinguishes `>` from `==` for data.len() > remaining_capacity + 1.)
//!
//! - **467 `&&→||`**: carry = `[0x05, 0xAA, ...]` (5 bytes, first byte = 0x05, second ≠ 0x64).
//!   Correct `&&`: `(0x05==0x05) && (0xAA==0x64)` = true && false = false → is_sync=false → resync.
//!   Mutant `||`: `(0x05==0x05) || (0xAA==0x64)` = true || false = true → is_sync=true → no resync.
//!   Assert parse_errors == 1 (resync must fire).
//!
//! - **479×3 (sub-10 non-sync resync find)**: carry = `[0xAA, 0xBB, 0x64, 0x05, 0x64]`
//!   (5 bytes, is_sync=false). Skip(1) search:
//!   - `==→!=` on w[0]: correct finds `[0x05,0x64]` at window 3 (drain ..3).
//!     Mutant: `w[0]!=0x05 && w[1]==0x64` matches `[0xBB,0x64]` at window 1 (drain ..1).
//!     Carry after: correct=[0x05,0x64], mutant=[0xBB,0x64,0x05,0x64].
//!     Assert carry[0] == 0x05.
//!
//!   - `&&→||` on the find: correct finds `[0x05,0x64]` at window 3 first.
//!     Mutant `||`: `[0xBB,0x64]` at window 1 matches (`0xBB==0x05` false, `0x64==0x64` true).
//!     Wait: `||` on w[0]==0x05 || w[1]==0x64 → `[0xBB,0x64]`: false||true = true → match!
//!     Carry after: correct=[0x05,0x64], mutant=[0xBB,0x64,0x05,0x64].
//!     Assert carry[0] == 0x05.
//!
//!   - `==→!=` on w[1]: carry = `[0xAA, 0x05, 0x64, ...]`. Skip(1) finds `[0x05,0x64]`.
//!     Mutant `w[0]==0x05 && w[1]!=0x64` → on `[0x05,0x64]`: `true && false` = false.
//!     No match → clear carry. Correct: carry starts with 0x05.
//!     Assert carry.len() >= 2 && carry[0] == 0x05.
//!
//! - **511×3 (sub-10 did_process resync find)**: Reach via: complete valid frame + bad-length
//!   residue `[0x05, 0x64, 0x02, 0x05, 0x64]` in carry. After consuming frame,
//!   did_process=true, residual is `[0x05,0x64,0x02,0x05,0x64]` (5 bytes), sub-10 path,
//!   is_sync=true, did_process=true, carry_len=5>=3, LENGTH=0x02 < 5 → fires. drain(..1).
//!   Carry = `[0x64, 0x02, 0x05, 0x64]`. find at 511.
//!   - Correct: finds `[0x05,0x64]` at window index 2 → drain ..2 → carry = `[0x05,0x64]`.
//!   - `==→!=` w[0]: no match → clear.
//!   - `&&→||`: `[0x64,0x02]`: `false||false=false`; `[0x02,0x05]`: `false||false=false`;
//!     `[0x05,0x64]`: `true||true=true` → same as correct. Hmm — need a different carry.
//!
//!   For `&&→||` at 511: use `[0x05, 0x64, 0x02, 0xAA, 0x64]`. After drain(..1):
//!   carry = `[0x64, 0x02, 0xAA, 0x64]`. Windows: `[0x64,0x02]`, `[0x02,0xAA]`, `[0xAA,0x64]`.
//!   - Correct `&&`: `[0xAA,0x64]` → `0xAA==0x05 && 0x64==0x64` = false && true = false. No match → clear.
//!   - Mutant `||`: `[0x64,0x02]` → `0x64==0x05 || 0x02==0x64` = false||false=false.
//!     `[0x02,0xAA]` → false||false=false. `[0xAA,0x64]` → `false || true` = true → match at index 2!
//!     drain ..2 → carry = `[0xAA,0x64]`.
//!   Assert: carry.len() == 0 (correct) vs carry.len() == 2 (mutant). Assert carry is empty.
//!
//! - **555 `||→&&`**: carry = `[0x05, 0xAA, ...]` (10 bytes, carry[0]=0x05, carry[1]=0xAA ≠ 0x64).
//!   Correct `||`: `(0x05!=0x05) || (0xAA!=0x64)` = false || true = true → sync-loss fires.
//!   Mutant `&&`: `false && true` = false → skips sync-loss, goes to LENGTH check.
//!   Assert parse_errors == 1 (sync-loss must fire exactly once).
//!
//! ## Naming Convention
//!
//! `test_BC_S_SS_NNN_<assertion>()` per factory DF-TEST-NAMESPACE-001.
//!
//! ## Zero production-code changes
//!
//! This file contains ONLY tests. No `src/` files are modified.

#![allow(non_snake_case)]
#![allow(clippy::doc_lazy_continuation)]

mod group_a_survivors {
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, a))
    }

    fn port20000_key() -> FlowKey {
        // lower=(10.0.0.1,20000), upper=(10.0.0.2,54321)
        FlowKey::new(ip(1), 20000, ip(2), 54321)
    }

    /// Minimal valid DNP3 frame — 10 bytes (LENGTH=5 → frame_len=10).
    ///
    /// [0x05, 0x64, 0x05, 0xC4, 0x03, 0x00, 0x01, 0x00, CRC-lo, CRC-hi]
    /// CRC placeholder bytes (0x00) are intentionally wrong — the parser
    /// accepts frames even with invalid CRCs (no CRC validation in the analyzer).
    fn minimal_valid_frame() -> Vec<u8> {
        vec![0x05, 0x64, 0x05, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]
    }

    /// Establish a flow with one valid frame so carry is empty and is_non_dnp3=false.
    fn establish_flow(analyzer: &mut Dnp3Analyzer, key: &FlowKey) {
        analyzer.on_data(
            key.clone(),
            &minimal_valid_frame(),
            0,
            Direction::ClientToServer,
        );
        let flow = analyzer.flows.get(key).expect("flow must exist");
        assert_eq!(
            flow.carry_c2s.len(),
            0,
            "precondition: carry_c2s empty after valid frame consumed"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 409:53  `-` → `+`
    // remaining_capacity = MAX_DNP3_FRAME_LEN [op] carry.len()
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 EC-004 / kills dnp3.rs line 409:53 (`-`→`+`):
    ///
    /// Carry at 290 bytes; deliver 5 bytes.
    /// Correct: remaining_capacity = 292 - 290 = 2. `5 > 2` → overflow → parse_errors=1.
    /// Mutant `+`: remaining_capacity = 292 + 290 = 582. `5 > 582` = false → no overflow,
    /// parse_errors stays 0.
    ///
    /// REGRESSION-GUARD: if remaining_capacity arithmetic flips sign, overflow never fires
    /// on a near-full carry — a DoS vector (BC-2.15.016 EC-004).
    #[test]
    fn test_BC_2_15_016_overflow_arm_line409_minus_to_plus_carry290_deliver5() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Directly set carry_c2s to 290 bytes of junk (no embedded sync after byte 1).
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s.extend(std::iter::repeat_n(0xAA_u8, 290));
            assert_eq!(
                flow.carry_c2s.len(),
                290,
                "precondition: carry must be 290 bytes"
            );
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        // Deliver 5 bytes. remaining_capacity = 292-290 = 2. data.len()=5 > 2 → overflow.
        // Mutant `-→+`: remaining_capacity = 582; 5 > 582 = false → no overflow.
        analyzer.on_data(key.clone(), &[0xBB_u8; 5], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 1,
            "BC-2.15.016 / kills line 409:53 (`-`→`+`): overflow arm must fire when \
             carry(290)+data(5)>292; mutant `+` inflates remaining_capacity to 582, \
             suppressing the overflow → parse_errors stays 0 instead of 1"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "BC-2.15.016 / kills line 409:53: malformed_in_window must also be 1 \
             (overflow arm increments both counters)"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 410:23  `>` → `>=`
    // if data.len() [op] remaining_capacity
    //
    // Strategy: use carry = 291 bytes of a valid 292-byte frame header prefix
    // (remaining_capacity = 1).  The frame-walk on a complete valid frame does NOT
    // increment parse_errors.  Delivering exactly 1 byte (exact fit):
    //   Correct `>`: 1 > 1 = false → extend (no overflow). carry = 292. Frame processed.
    //     parse_errors = 0.
    //   Mutant `>=`: 1 >= 1 = true → overflow fires. parse_errors = 1. inline resync
    //     (sync at pos 0, drain ..0 = no-op). Frame-walk on same 292-byte carry.
    //     parse_errors = 1 (just from overflow arm, no additional from frame-walk).
    // Discriminator: parse_errors == 0 under correct `>`, == 1 under `>=`.
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 EC-004 / kills dnp3.rs line 410:23 (`>`→`>=`):
    ///
    /// Carry at 291 bytes (one byte short of a 292-byte frame, LENGTH=255).
    /// remaining_capacity = MAX_DNP3_FRAME_LEN - 291 = 1.
    /// Deliver exactly 1 byte (exact fit: data.len() == remaining_capacity).
    ///
    /// Correct `>`: `1 > 1` = false → normal extend. carry = 292. The 292-byte frame
    /// header is valid (start1=0x05, start2=0x64, length=0xFF≥5) → frame_count += 1.
    /// No parse_errors from overflow arm; no parse_errors from frame-walk. parse_errors=0.
    ///
    /// Mutant `>=`: `1 >= 1` = true → overflow fires. parse_errors = 1 (from overflow arm).
    /// inline resync finds [0x05,0x64] at pos 0 → drain ..0 = no-op. Same 292-byte frame
    /// processed by frame-walk (no additional parse_errors). Total parse_errors = 1.
    ///
    /// REGRESSION-GUARD: `>=` fires overflow on exact-fit delivery — a benign append that
    /// fills carry to exactly MAX_DNP3_FRAME_LEN is incorrectly counted as an overflow event
    /// (BC-2.15.016 PC1: no overflow unless bytes would EXCEED the cap).
    #[test]
    fn test_BC_2_15_016_overflow_arm_line410_gt_to_gte_exact_fit_no_overflow() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Build a 291-byte carry: a 292-byte frame prefix with the last byte missing.
        // Frame layout for LENGTH=255 (frame_len=292):
        //   [0x05, 0x64, 0xFF, 0xC4, 0x03, 0x00, 0x01, 0x00, CRC1, CRC2, payload×282]
        // is_valid_dnp3_frame_header: start1==0x05, start2==0x64, length==0xFF >= 5 → true.
        // carry_len = 291 < 292 (frame_len) → frame-walk breaks immediately (partial, no errors).
        // remaining_capacity = 292 - 291 = 1.
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            // Sync + LENGTH=255 + CTRL + DEST_L + DEST_H + SRC_L + SRC_H + CRC×2 + payload×281
            flow.carry_c2s
                .extend_from_slice(&[0x05, 0x64, 0xFF, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
            flow.carry_c2s.extend(std::iter::repeat_n(0x00_u8, 281)); // 281 more payload/CRC bytes
            assert_eq!(
                flow.carry_c2s.len(),
                291,
                "precondition: carry must be 291 bytes (one short of frame_len=292)"
            );
            assert_eq!(flow.carry_c2s[0], 0x05, "precondition: carry[0] == 0x05");
            assert_eq!(flow.carry_c2s[1], 0x64, "precondition: carry[1] == 0x64");
            assert_eq!(
                flow.carry_c2s[2], 0xFF,
                "precondition: carry[2] == 0xFF (LENGTH=255)"
            );
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
            flow.frame_count = 0;
        }

        // Deliver exactly 1 byte (the last payload byte).
        // data.len() == remaining_capacity == 1 — exact fit.
        // Correct `>`: 1 > 1 = false → extend → carry = 292 (complete frame). No overflow.
        // Mutant `>=`: 1 >= 1 = true → overflow arm. parse_errors = 1.
        analyzer.on_data(key.clone(), &[0x00_u8; 1], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 0,
            "BC-2.15.016 / kills line 410:23 (`>`→`>=`): delivering exactly remaining_capacity \
             bytes (exact fit: 1 byte into 1 remaining slot) must NOT trigger overflow → \
             parse_errors must stay 0; mutant `>=` fires overflow on exact-fit → parse_errors=1"
        );
        assert_eq!(
            flow.malformed_in_window, 0,
            "BC-2.15.016 / kills line 410:23: malformed_in_window must also be 0 on exact-fit \
             delivery"
        );
    }

    /// BC-2.15.016 / confirms overflow fires for data.len() > remaining_capacity (companion):
    ///
    /// Carry at 291 bytes (remaining_capacity=1). Deliver 2 bytes (one over capacity).
    /// Correct `>`: `2 > 1` = true → overflow. Only 1 byte appended (data[..1]). carry = 292.
    ///   Inline resync finds [0x05,0x64] at pos 0 → no-op. Frame processed. No extra errors.
    ///   parse_errors = 1 (from overflow arm).
    /// Mutant `==`: `2 == 1` = false → no overflow. extend(data) = 2 bytes. carry = 293 bytes.
    ///   Frame-walk: frame_len=292, carry.len()=293 → complete. Process. drain(..292). carry=1 byte.
    ///   carry_len=1 < 2 → skip is_sync check → break. parse_errors = 0.
    ///
    /// Together with the exact-fit test, these two form a discriminating pair: only correct `>`
    /// passes both tests simultaneously (parse_errors==0 on exact-fit, parse_errors==1 on +1).
    ///
    /// REGRESSION-GUARD: `==` misses all overflows except when data.len() is exactly 1 more
    /// than remaining_capacity — a near-full carry receiving 2+ excess bytes is silently extended
    /// beyond MAX_DNP3_FRAME_LEN (BC-2.15.016 PC2 violation).
    #[test]
    fn test_BC_2_15_016_overflow_arm_line410_gt_to_eq_one_over_fires() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Same 291-byte carry setup as the exact-fit test (remaining_capacity=1).
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&[0x05, 0x64, 0xFF, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
            flow.carry_c2s.extend(std::iter::repeat_n(0x00_u8, 281));
            assert_eq!(flow.carry_c2s.len(), 291);
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        // Deliver 2 bytes: data.len()=2 > remaining_capacity=1 → overflow under correct `>`.
        // Mutant `==`: 2==1=false → no overflow → carry gets 2 bytes → 293 total.
        analyzer.on_data(key.clone(), &[0x00_u8; 2], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 1,
            "BC-2.15.016 / kills line 410:23 (`>`→`==`): data.len()=2 > remaining_capacity=1 \
             must trigger overflow → parse_errors=1; mutant `==` computes `2==1`=false → no \
             overflow arm → parse_errors=0 (the excess byte sneaks through into carry)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "BC-2.15.016 / kills line 410:23 (`>`→`==`): malformed_in_window must be 1 \
             when overflow fires"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 467:25  `&&` → `||`
    // is_sync = carry[0]==0x05 [op] carry[1]==0x64  (sub-10-byte arm)
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 / kills dnp3.rs line 467:25 (`&&`→`||`) in sub-10-byte is_sync check:
    ///
    /// Carry = [0x05, 0xAA, 0xCC, 0xDD, 0xEE] (5 bytes).
    /// First byte == 0x05 (matches), second byte == 0xAA ≠ 0x64 (no match).
    ///
    /// Correct `&&`: `(0x05==0x05) && (0xAA==0x64)` = true && false = **false**
    ///   → is_sync=false → `!is_sync=true` → resync arm fires → parse_errors += 1.
    ///
    /// Mutant `||`: `(0x05==0x05) || (0xAA==0x64)` = true || false = **true**
    ///   → is_sync=true → `!is_sync=false` → resync arm does NOT fire → parse_errors stays 0.
    ///
    /// REGRESSION-GUARD: `||` would treat any carry starting with 0x05 (but wrong second byte)
    /// as a valid sync — a near-sync carry fragment would pass undetected as a partial frame,
    /// suppressing the parse_error that marks sync-loss (BC-2.15.009 / BC-2.15.024 Inv-2).
    #[test]
    fn test_BC_2_15_016_sub10_is_sync_line467_and_to_or_first_byte_match_second_wrong() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // carry_c2s = [0x05, 0xAA, 0xCC, 0xDD, 0xEE]:
        //   carry[0] = 0x05 (== 0x05, matches first sync byte)
        //   carry[1] = 0xAA (!= 0x64, wrong second sync byte)
        //   carry_len = 5 (< 10 → sub-10 path, >= 2 → is_sync check applies)
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&[0x05, 0xAA, 0xCC, 0xDD, 0xEE]);
            assert_eq!(flow.carry_c2s.len(), 5);
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "precondition: first byte must be 0x05"
            );
            assert_ne!(
                flow.carry_c2s[1], 0x64,
                "precondition: second byte must not be 0x64"
            );
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        // Deliver empty data → triggers frame-walk → sub-10 path fires.
        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 1,
            "BC-2.15.016 / kills line 467:25 (`&&`→`||`): carry[0]==0x05 but carry[1]!=0x64 \
             must be treated as NON-sync (is_sync=false) → resync must fire → parse_errors=1; \
             mutant `||` short-circuits on carry[0]==0x05 → is_sync=true → resync suppressed \
             → parse_errors remains 0. This allows a near-sync fragment to bypass detection."
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "BC-2.15.016 / kills line 467:25: malformed_in_window must be 1 (resync increments both)"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 479:49  `==` → `!=` on w[0]  (sub-10 non-sync resync find, `.skip(1)`)
    // find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)  ← w[0] targeted
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 / kills dnp3.rs line 479:49 (`==`→`!=` on w[0]):
    ///
    /// Carry = [0xAA, 0x05, 0x64, 0xBB, 0xCC] (5 bytes, is_sync=false: carry[0]=0xAA).
    /// Sub-10 resync find with skip(1) scans from window index 1:
    ///   Window 0: [0xAA, 0x05] (skipped by skip(1))
    ///   Window 1: [0x05, 0x64] → correct: `0x05==0x05 && 0x64==0x64` = true → match at index 1
    ///   → drain ..1 → carry = [0x05, 0x64, 0xBB, 0xCC] → carry[0] = 0x05.
    ///
    /// Mutant `w[0] != 0x05 && w[1] == 0x64`:
    ///   Window 1: [0x05, 0x64] → `0x05!=0x05 = false` → false && true = false → no match.
    ///   Window 2: [0x64, 0xBB] → true && false = false.
    ///   Window 3: [0xBB, 0xCC] → true && false = false.
    ///   → None → carry.clear() → carry[0] does not exist.
    ///
    /// REGRESSION-GUARD: `!=` on w[0] inverts the sync-byte test — the real sync word
    /// [0x05, 0x64] is rejected, junk bytes pass instead (BC-2.15.009 Inv-1).
    #[test]
    fn test_BC_2_15_016_sub10_resync_find_line479_eq_to_neq_w0_embedded_sync() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // carry = [0xAA, 0x05, 0x64, 0xBB, 0xCC]:
        //   carry[0] = 0xAA: is_sync = false (triggers resync path)
        //   carry[1..3] = [0x05, 0x64]: the real sync at offset 1 (found by skip(1))
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&[0xAA, 0x05, 0x64, 0xBB, 0xCC]);
            assert_eq!(flow.carry_c2s.len(), 5);
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        // Resync must have found the embedded [0x05,0x64] at offset 1 and drained ..1.
        // carry should now start with [0x05, 0x64, ...] — not be empty.
        assert!(
            flow.carry_c2s.len() >= 2,
            "BC-2.15.016 / kills line 479:49 (`==`→`!=` w[0]): sub-10 resync find with embedded \
             [0x05,0x64] at offset 1 must NOT clear carry (sync word found → drain ..1); \
             mutant `!=` inverts sync byte match → [0x05,0x64] never matches → carry cleared"
        );
        if flow.carry_c2s.len() >= 2 {
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "BC-2.15.016 / kills line 479:49: after resync, carry[0] must be 0x05 \
                 (drain brought sync word to front); mutant `!=` clears carry entirely"
            );
            assert_eq!(
                flow.carry_c2s[1], 0x64,
                "BC-2.15.016 / kills line 479:49: after resync, carry[1] must be 0x64"
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 479:57  `&&` → `||`  (sub-10 non-sync resync find)
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 / kills dnp3.rs line 479:57 (`&&`→`||`):
    ///
    /// Carry = [0xAA, 0xBB, 0x64, 0x05, 0x64] (5 bytes, is_sync=false: carry[0]=0xAA).
    /// The carry has a partial match at offset 1 ([0xBB,0x64]: second byte = 0x64 but first ≠ 0x05),
    /// and a full match at offset 3 ([0x05,0x64]).
    ///
    /// Skip(1) search (windows from index 1):
    ///   Window 1: [0xBB, 0x64] → correct `&&`: false && true = false (not sync).
    ///                             Mutant `||`: false || true = **true** → match at index 1!
    ///   Window 2: [0x64, 0x05] → correct: false. Mutant: true||false=true (but already matched).
    ///   Window 3: [0x05, 0x64] → correct: true → match at index 3!
    ///
    /// Correct: drain ..3 → carry = [0x05, 0x64] → carry[0] = 0x05.
    /// Mutant `||`: drain ..1 → carry = [0xBB, 0x64, 0x05, 0x64] → carry[0] = 0xBB ≠ 0x05.
    ///
    /// REGRESSION-GUARD: `||` matches a partial sync (wrong first byte, correct second byte),
    /// repositioning carry to a non-sync head — the subsequent frame-walk will fire a spurious
    /// sync-loss event or fail to parse legitimate traffic (BC-2.15.009 Inv-1).
    #[test]
    fn test_BC_2_15_016_sub10_resync_find_line479_and_to_or_partial_match() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // carry = [0xAA, 0xBB, 0x64, 0x05, 0x64]:
        //   [0xBB, 0x64] at window 1: partial match (w[1]==0x64 but w[0]!=0x05)
        //   [0x05, 0x64] at window 3: the real sync word
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&[0xAA, 0xBB, 0x64, 0x05, 0x64]);
            assert_eq!(flow.carry_c2s.len(), 5);
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        // Correct: resync finds [0x05,0x64] at window 3 → drain ..3 → carry = [0x05,0x64].
        // Mutant `||`: finds [0xBB,0x64] at window 1 → drain ..1 → carry[0] = 0xBB.
        if flow.carry_c2s.len() >= 2 {
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "BC-2.15.016 / kills line 479:57 (`&&`→`||`): resync must find the REAL sync \
                 word [0x05,0x64] at offset 3, not the partial match [0xBB,0x64] at offset 1; \
                 mutant `||` accepts any window where w[1]==0x64 → drain at wrong offset → \
                 carry[0]=0xBB instead of 0x05"
            );
            assert_eq!(
                flow.carry_c2s[1], 0x64,
                "BC-2.15.016 / kills line 479:57: carry[1] must be 0x64 after proper resync"
            );
        } else {
            // carry was cleared — this means the None arm fired (neither correct nor mutant
            // found a sync). This should not happen with [0x05,0x64] in the carry.
            panic!(
                "BC-2.15.016 / kills line 479:57: carry must not be empty when [0x05,0x64] \
                 is embedded in the carry; resync should have found and preserved the sync word"
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 479:65  `==` → `!=` on w[1]  (sub-10 non-sync resync find)
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 / kills dnp3.rs line 479:65 (`==`→`!=` on w[1]):
    ///
    /// Carry = [0xAA, 0x05, 0x64, 0xBB, 0xCC] (same as the w[0] test).
    /// Skip(1) search from window index 1:
    ///   Window 1: [0x05, 0x64] → correct `w[0]==0x05 && w[1]==0x64`: true && true = true → match.
    ///             Mutant `w[0]==0x05 && w[1]!=0x64`: true && (0x64!=0x64=false) = false → no match.
    ///   Window 2: [0x64, 0xBB] → correct: false; mutant: false && true = false.
    ///   Window 3: [0xBB, 0xCC] → correct: false; mutant: false && true = false.
    ///   → Mutant → None → carry.clear().
    ///
    /// Correct: drain ..1 → carry = [0x05, 0x64, 0xBB, 0xCC] → carry[0]=0x05.
    /// Mutant: carry.clear() → carry is empty.
    ///
    /// REGRESSION-GUARD: `!=` on w[1] inverts the second sync-byte check — the real sync
    /// word is rejected when w[1]=0x64, causing a spurious carry-clear and data loss.
    #[test]
    fn test_BC_2_15_016_sub10_resync_find_line479_eq_to_neq_w1_embedded_sync() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Same carry as the w[0] test — [0xAA, 0x05, 0x64, 0xBB, 0xCC].
        // The discriminating factor is which mutation we are testing; both use the same input
        // because [0x05,0x64] at offset 1 distinguishes w[0]==0x05 and w[1]==0x64 separately.
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&[0xAA, 0x05, 0x64, 0xBB, 0xCC]);
            assert_eq!(flow.carry_c2s.len(), 5);
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.carry_c2s.len() >= 2,
            "BC-2.15.016 / kills line 479:65 (`==`→`!=` w[1]): embedded [0x05,0x64] at offset 1 \
             must survive resync (drain ..1); mutant `w[1]!=0x64` rejects the real sync word at \
             window 1 → no match → carry cleared entirely"
        );
        if flow.carry_c2s.len() >= 2 {
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "BC-2.15.016 / kills line 479:65: carry[0] must be 0x05 after resync \
                 (sync word at offset 1 found and preserved)"
            );
            assert_eq!(
                flow.carry_c2s[1], 0x64,
                "BC-2.15.016 / kills line 479:65: carry[1] must be 0x64 after resync"
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutants 511:53, 511:61, 511:69 — sub-10 did_process resync find
    // (the arm: did_process_in_this_call && carry_len >= 3 && valid-sync && bad-LENGTH)
    //
    // Reached only when:
    //   1. carry_len < 10 (sub-10 path)
    //   2. carry[0..2] == [0x05, 0x64] (is_sync=true)
    //   3. did_process_in_this_call == true (a full frame was consumed earlier in this call)
    //   4. carry_len >= 3
    //   5. compute_dnp3_frame_len(carry[2]) == None  (LENGTH < 5)
    //
    // To set did_process_in_this_call=true in the same on_data call, we put a full
    // valid frame in carry followed by the bad-length residue.  on_data with empty data
    // then: (a) consumes the valid frame → did_process=true; (b) finds residue.
    // ───────────────────────────────────────────────────────────────────────

    /// Build a carry payload that, when processed by a single on_data call, exercises
    /// the `did_process` sub-10 arm and leaves `residue` (relative to the sync word
    /// immediately after drain(..1)) for the find-at-511 to process.
    ///
    /// Structure: valid_frame (10 bytes) || sync_bad_len (3 bytes) || extra_tail
    ///
    /// `extra_tail` is appended after the bad-length triplet to create the window
    /// content that distinguishes the mutations.
    fn carry_for_511_arm(extra_tail: &[u8]) -> Vec<u8> {
        // minimal_valid_frame = [0x05,0x64,0x05,0xC4,0x03,0x00,0x01,0x00,0x00,0x00]
        // After it is consumed, carry starts at the sync_bad_len part.
        let mut carry = minimal_valid_frame();
        carry.push(0x05); // second frame sync byte 0
        carry.push(0x64); // second frame sync byte 1
        carry.push(0x02); // LENGTH=2 < 5 → bad → LENGTH-gate fires drain(..1)
        carry.extend_from_slice(extra_tail);
        carry
    }

    /// BC-2.15.016 / kills dnp3.rs line 511:53 (`==`→`!=` on w[0]):
    ///
    /// After drain(..1), carry = [0x64, 0x02, 0x05, 0x64] (if extra_tail=[0x05,0x64]).
    /// Correct find: windows [0x64,0x02], [0x02,0x05], [0x05,0x64] →
    ///   Window 2: `0x05==0x05 && 0x64==0x64` = true → match at index 2 → drain ..2
    ///   → carry = [0x05, 0x64].
    ///
    /// Mutant `w[0]!=0x05 && w[1]==0x64`:
    ///   Window 0: [0x64,0x02] → 0x64!=0x05=true && 0x02==0x64=false → false.
    ///   Window 1: [0x02,0x05] → true && false → false.
    ///   Window 2: [0x05,0x64] → 0x05!=0x05=false → false. → None → clear.
    ///
    /// REGRESSION-GUARD: the did_process sub-10 arm inverts sync-byte 0 test → real
    /// sync word rejected, carry cleared → subsequent frame delivery finds empty carry,
    /// losing a valid pending frame head (BC-2.15.024 PC1).
    #[test]
    fn test_BC_2_15_016_sub10_did_process_find_line511_eq_to_neq_w0() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Build carry: [valid_frame(10)] ++ [0x05,0x64,0x02] ++ [0x05,0x64]
        // After valid frame consumed: residue = [0x05,0x64,0x02,0x05,0x64] (5 bytes).
        // Sub-10 arm fires: is_sync=true, did_process=true, carry_len=5>=3, LENGTH=2<5.
        // drain(..1): carry = [0x64,0x02,0x05,0x64].
        // find at 511 should locate [0x05,0x64] at index 2 → drain ..2 → [0x05,0x64].
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&carry_for_511_arm(&[0x05, 0x64]));
            assert_eq!(
                flow.carry_c2s.len(),
                15,
                "precondition: carry = 10+3+2 = 15 bytes"
            );
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        // Correct: carry = [0x05,0x64] after find-at-511 resync.
        // Mutant `!=` w[0]: carry cleared (None arm).
        assert!(
            flow.carry_c2s.len() >= 2,
            "BC-2.15.016 / kills line 511:53 (`==`→`!=` w[0]): did_process sub-10 arm resync \
             must find [0x05,0x64] at offset 2 after drain(..1) and drain to sync; \
             mutant `!=` inverts first-byte check → [0x05,0x64] never matches → carry cleared"
        );
        if flow.carry_c2s.len() >= 2 {
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "BC-2.15.016 / kills line 511:53: carry[0] must be 0x05 after resync at 511"
            );
            assert_eq!(
                flow.carry_c2s[1], 0x64,
                "BC-2.15.016 / kills line 511:53: carry[1] must be 0x64 after resync at 511"
            );
        }
    }

    /// BC-2.15.016 / kills dnp3.rs line 511:61 (`&&`→`||`):
    ///
    /// Need a carry where, after drain(..1), the find windows contain a partial match
    /// (w[1]==0x64 but w[0]!=0x05) BEFORE the real sync word.
    ///
    /// Use extra_tail = [0xAA, 0x64] (so residue = [0x05,0x64,0x02,0xAA,0x64]).
    /// After drain(..1): carry = [0x64, 0x02, 0xAA, 0x64].
    /// Windows: [0x64,0x02] idx 0, [0x02,0xAA] idx 1, [0xAA,0x64] idx 2.
    ///
    /// Correct `&&`: no window matches `w[0]==0x05 && w[1]==0x64` → None → clear carry.
    /// Mutant `||`: window 2 [0xAA,0x64] → `0xAA==0x05=false || 0x64==0x64=true` = true → match!
    ///   drain ..2 → carry = [0xAA, 0x64] → carry[0]=0xAA.
    ///
    /// Assert carry.len() == 0 (correct: no sync found, carry cleared).
    ///
    /// REGRESSION-GUARD: `||` on the find predicate accepts any window with second byte == 0x64
    /// as a "sync" → the carry is mispositioned to a non-sync location, introducing a spurious
    /// sync-loss event or a phantom-frame parse on the next iteration.
    #[test]
    fn test_BC_2_15_016_sub10_did_process_find_line511_and_to_or_partial_second_byte_match() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Carry: [valid_frame(10)] ++ [0x05,0x64,0x02] ++ [0xAA,0x64]
        // Residue after valid frame: [0x05,0x64,0x02,0xAA,0x64] (5 bytes).
        // drain(..1) → [0x64,0x02,0xAA,0x64].
        // find at 511:
        //   Correct `&&`: [0xAA,0x64] → false&&true=false. No match → clear.
        //   Mutant `||`: [0xAA,0x64] → false||true=true → drain ..2 → [0xAA,0x64].
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&carry_for_511_arm(&[0xAA, 0x64]));
            assert_eq!(
                flow.carry_c2s.len(),
                15,
                "precondition: carry = 10+3+2 = 15 bytes"
            );
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.carry_c2s.len(),
            0,
            "BC-2.15.016 / kills line 511:61 (`&&`→`||`): after drain(..1), carry=[0x64,0x02,0xAA,0x64] \
             contains no real [0x05,0x64] sync → find must return None → carry.clear() → len==0; \
             mutant `||` matches [0xAA,0x64] as 'sync' → drain ..2 → carry=[0xAA,0x64] (len==2)"
        );
    }

    /// BC-2.15.016 / kills dnp3.rs line 511:69 (`==`→`!=` on w[1]):
    ///
    /// Use extra_tail = [0x05, 0x64] (same as the w[0] test at 511:53).
    /// After drain(..1): carry = [0x64, 0x02, 0x05, 0x64].
    ///
    /// Correct find: window 2 [0x05,0x64] → `0x05==0x05 && 0x64==0x64` = true → match at 2
    ///   → drain ..2 → carry = [0x05,0x64].
    ///
    /// Mutant `w[0]==0x05 && w[1]!=0x64`:
    ///   Window 0: [0x64,0x02] → false && true = false.
    ///   Window 1: [0x02,0x05] → false && true = false.
    ///   Window 2: [0x05,0x64] → `0x05==0x05=true && 0x64!=0x64=false` → false. No match.
    ///   → None → clear.
    ///
    /// REGRESSION-GUARD: `!=` on w[1] rejects the real sync word → carry cleared → valid
    /// pending frame header is discarded (same class of bug as 511:53).
    #[test]
    fn test_BC_2_15_016_sub10_did_process_find_line511_eq_to_neq_w1() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // Same carry as 511:53 test — [valid_frame(10)] ++ [0x05,0x64,0x02,0x05,0x64].
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s
                .extend_from_slice(&carry_for_511_arm(&[0x05, 0x64]));
            assert_eq!(flow.carry_c2s.len(), 15);
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.carry_c2s.len() >= 2,
            "BC-2.15.016 / kills line 511:69 (`==`→`!=` w[1]): find at 511 must locate real \
             [0x05,0x64] and drain to it; mutant `w[1]!=0x64` rejects the real sync → None → clear"
        );
        if flow.carry_c2s.len() >= 2 {
            assert_eq!(
                flow.carry_c2s[0], 0x05,
                "BC-2.15.016 / kills line 511:69: carry[0] must be 0x05 after resync at 511"
            );
            assert_eq!(
                flow.carry_c2s[1], 0x64,
                "BC-2.15.016 / kills line 511:69: carry[1] must be 0x64 after resync at 511"
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // Mutant 555:17  `||` → `&&`
    // if carry[0]!=0x05 [op] carry[1]!=0x64  (≥10-byte sync-gate)
    // ───────────────────────────────────────────────────────────────────────

    /// BC-2.15.016 / kills dnp3.rs line 555:17 (`||`→`&&`) in the ≥10-byte sync-gate:
    ///
    /// Carry = [0x05, 0xAA, 0xAA×8] (10 bytes).
    ///   carry[0] = 0x05 (== 0x05 → first byte MATCHES)
    ///   carry[1] = 0xAA (!= 0x64 → second byte DOES NOT match)
    ///   carry_len = 10 ≥ 10 → enters the ≥10-byte loop body.
    ///
    /// Sync-gate predicate `carry[0]!=0x05 || carry[1]!=0x64`:
    ///   Correct `||`: `(0x05!=0x05) || (0xAA!=0x64)` = false || true = **true**
    ///     → sync-loss arm fires → parse_errors += 1 → resync.
    ///   Mutant `&&`: `false && true` = **false**
    ///     → sync-loss skipped → falls through to LENGTH check.
    ///     carry[2]=0xAA=170 → compute_dnp3_frame_len(170) = Some(197) → 10<197 → break.
    ///     parse_errors unchanged (stays 0).
    ///
    /// REGRESSION-GUARD: `&&` requires BOTH bytes to be wrong before treating the carry as
    /// non-sync — a carry starting with 0x05 followed by the wrong second byte would silently
    /// pass the sync-gate, causing incorrect partial-frame stashing and suppressing sync-loss
    /// detection (BC-2.15.009 / BC-2.15.024 Inv-2).
    #[test]
    fn test_BC_2_15_016_ge10_sync_gate_line555_or_to_and_first_byte_match_second_wrong() {
        let key = port20000_key();
        let mut analyzer = Dnp3Analyzer::new(10);
        establish_flow(&mut analyzer, &key);

        // carry_c2s = [0x05, 0xAA, 0xAA×8] (10 bytes):
        //   carry[0] = 0x05: matches 0x05 (first sync byte is correct)
        //   carry[1] = 0xAA: does NOT match 0x64 (second sync byte is wrong)
        //   carry_len = 10 ≥ 10 → ≥10-byte loop body (not sub-10 path)
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry_c2s.clear();
            flow.carry_c2s.push(0x05);
            flow.carry_c2s.push(0xAA);
            flow.carry_c2s.extend(std::iter::repeat_n(0xAA_u8, 8));
            assert_eq!(
                flow.carry_c2s.len(),
                10,
                "precondition: carry must be 10 bytes"
            );
            assert_eq!(flow.carry_c2s[0], 0x05, "precondition: carry[0] == 0x05");
            assert_ne!(flow.carry_c2s[1], 0x64, "precondition: carry[1] != 0x64");
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
        }

        // Deliver empty data → frame-walk loop enters with carry_len=10.
        // carry[0]=0x05, carry[1]=0xAA.
        // Correct `||`: `0x05!=0x05 || 0xAA!=0x64` = false || true = true → sync-loss.
        // Mutant `&&`: `false && true` = false → falls through to LENGTH check → no sync-loss.
        analyzer.on_data(key.clone(), &[], 1, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 1,
            "BC-2.15.016 / kills line 555:17 (`||`→`&&`): carry[0]==0x05 but carry[1]!=0x64 \
             must trigger sync-loss (≥10-byte gate: carry[0]!=0x05 || carry[1]!=0x64 = true); \
             mutant `&&` requires BOTH bytes wrong → `false && true` = false → sync-loss \
             suppressed → parse_errors stays 0. This is the exact dual of the 467 bug."
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "BC-2.15.016 / kills line 555:17: sync-loss arm must also increment malformed_in_window"
        );
    }
}
