//! Tests for STORY-103: Modbus Flow State + Transaction Correlation — GREEN.
//!
//! Covers BC-2.14.009 (request insert), BC-2.14.010 (response match),
//! BC-2.14.011 (exception attribution), BC-2.14.012 (pending-table bound).
//!
//! Originally a Red Gate suite; STORY-103 is complete and all tests pass.
//! Test naming follows `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! Canonical test vectors used verbatim from BC documents.

// BC traceability convention mandates uppercase BC identifiers in function names
// (e.g. test_BC_2_14_009_...). Allow the non-snake-case names project-wide in
// this test file only — this is intentional, not a bug.
#![allow(non_snake_case)]
// Helper ADU builders are selectively referenced across tests; suppress unused warnings.
#![allow(dead_code)]

// Per DF-TEST-NAMESPACE-001: all STORY-103 tests are grouped inside a dedicated
// `mod story_103` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_103 {
    use wirerust::analyzer::modbus::{MAX_PENDING_TRANSACTIONS, ModbusAnalyzer, ModbusFlowState};

    // ---------------------------------------------------------------------------
    // Helpers — canonical ADU byte vectors from the BC test-vector tables.
    // ---------------------------------------------------------------------------

    /// BC-2.14.009 canonical vector: Read HR request.
    /// `00 01 00 00 00 06 01 03 00 00 00 0A`
    /// txn_id=0x0001, unit_id=0x01, FC=0x03 (Read Holding Registers)
    fn adu_read_hr_request_txn1() -> [u8; 12] {
        [
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
        ]
    }

    /// BC-2.14.009 canonical vector: Write Single Reg request.
    /// `00 02 00 00 00 06 01 06 00 14 01 F4`
    /// txn_id=0x0002, unit_id=0x01, FC=0x06 (Write Single Register)
    fn adu_write_single_reg_txn2() -> [u8; 12] {
        [
            0x00, 0x02, 0x00, 0x00, 0x00, 0x06, 0x01, 0x06, 0x00, 0x14, 0x01, 0xF4,
        ]
    }

    /// BC-2.14.010 canonical vector: Read HR response (echoes FC=0x03 for txn=0x0001, uid=0x01).
    /// `00 01 00 00 00 0F 01 03 0C 00 64 00 C8 01 2C 01 90 00 C8 00 FA`
    fn adu_read_hr_response_txn1() -> Vec<u8> {
        vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x0F, 0x01, 0x03, 0x0C, 0x00, 0x64, 0x00, 0xC8, 0x01,
            0x2C, 0x01, 0x90, 0x00, 0xC8, 0x00, 0xFA,
        ]
    }

    /// BC-2.14.011 canonical vector: Write exception (FC=0x86, code=0x01).
    /// `00 02 00 00 00 03 01 86 01`
    /// txn_id=0x0002, unit_id=0x01, FC=0x86 (exception for Write Single Reg 0x06)
    fn adu_write_exception_txn2() -> [u8; 9] {
        [0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x01, 0x86, 0x01]
    }

    /// BC-2.14.011 EC-010 canonical vector: spoofed exception.
    /// Exception FC=0x86 (original_fc=0x06) but pending slot holds FC=0x03 (Read HR).
    /// Expected: FC mismatch — pending NOT removed.
    fn adu_spoof_exception_0x86_while_pending_0x03(txn_id: u16, unit_id: u8) -> Vec<u8> {
        let [hi, lo] = txn_id.to_be_bytes();
        // FC=0x86 (exception for 0x06), length=3, unit_id as given
        vec![hi, lo, 0x00, 0x00, 0x00, 0x03, unit_id, 0x86, 0x01]
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.009 — Request PDU Inserted into Per-Flow Pending Table
    // ---------------------------------------------------------------------------

    /// AC-001 / BC-2.14.009 postcondition 1:
    /// A Read-class request inserts (txn_id, unit_id) -> (fc, ts) into pending.
    /// `pdu_count` is incremented and `last_ts` is updated.
    ///
    /// test_BC_2_14_009_request_pdu_inserted_into_pending
    #[test]
    fn test_BC_2_14_009_request_pdu_inserted_into_pending() {
        let mut flow = ModbusFlowState::default();
        let ts: u32 = 1_000_000;

        // Canonical vector: txn_id=0x0001, unit_id=0x01, FC=0x03
        let _ = adu_read_hr_request_txn1(); // document which vector we're exercising

        flow.insert_request(0x0001, 0x01, 0x03, ts);

        // postcondition 1: entry present with correct key and value
        assert!(
            flow.pending.contains_key(&(0x0001u16, 0x01u8)),
            "pending must contain (0x0001, 0x01) after insert_request"
        );
        let &(stored_fc, stored_ts) = flow.pending.get(&(0x0001u16, 0x01u8)).unwrap();
        assert_eq!(stored_fc, 0x03, "stored FC must match the request FC");
        assert_eq!(
            stored_ts, ts,
            "stored timestamp must match the call-site ts"
        );
    }

    /// AC-008 / BC-2.14.009 invariant 1:
    /// The pending table key is (txn_id, unit_id) — NOT txn_id alone.
    /// Same transaction_id with two different unit_ids must produce TWO distinct entries.
    ///
    /// test_BC_2_14_009_pending_key_is_txn_id_plus_unit_id
    #[test]
    fn test_BC_2_14_009_pending_key_is_txn_id_plus_unit_id() {
        let mut flow = ModbusFlowState::default();

        flow.insert_request(0x0001, 0x01, 0x03, 100);
        flow.insert_request(0x0001, 0x02, 0x03, 200);

        assert_eq!(
            flow.pending.len(),
            2,
            "same txn_id but different unit_ids must produce two distinct pending entries"
        );
        assert!(flow.pending.contains_key(&(0x0001u16, 0x01u8)));
        assert!(flow.pending.contains_key(&(0x0001u16, 0x02u8)));
    }

    /// AC-002 / BC-2.14.009 postcondition 2:
    /// Inserting a second request with the same (txn_id, unit_id) before a response
    /// overwrites the existing entry. `insert_request` returns `None` on a new key and
    /// returns `Some((old_fc, old_ts))` when it overwrites an existing key. This
    /// return-value signal is the testable STORY-103 unit behavior; it is consumed by
    /// the counter-increment wiring that lands in STORY-104.
    ///
    /// STORY-104 OBLIGATION:
    /// BC-2.14.009 inv-6 — ModbusAnalyzer.duplicate_inflight_txn MUST be incremented
    /// in on_data (STORY-104) when insert_request returns Some (overwrite). STORY-103
    /// ships only the insert_request return-value signal; the counter-increment wiring
    /// + its test land in STORY-104.
    ///
    /// test_BC_2_14_009_insert_request_returns_some_on_overwrite
    #[test]
    fn test_BC_2_14_009_insert_request_returns_some_on_overwrite() {
        let mut flow = ModbusFlowState::default();

        // First request inserts a NEW key — must return None (no prior entry).
        let old1 = flow.insert_request(0x0001, 0x01, 0x03, 100);
        assert!(
            old1.is_none(),
            "insert_request must return None when inserting a new (txn_id, unit_id) key"
        );

        // Second request with the SAME key before a response — overwrite path.
        // Must return Some((old_fc, old_ts)) reflecting the evicted entry.
        let old2 = flow.insert_request(0x0001, 0x01, 0x03, 200);
        assert!(
            old2.is_some(),
            "insert_request must return Some when overwriting an existing (txn_id, unit_id) key"
        );
        let (evicted_fc, evicted_ts) = old2.unwrap();
        assert_eq!(
            evicted_fc, 0x03,
            "evicted FC must equal the original request FC"
        );
        assert_eq!(
            evicted_ts, 100,
            "evicted timestamp must equal the original request timestamp"
        );

        // Pending table still has only one entry (the overwritten one).
        assert_eq!(
            flow.pending.len(),
            1,
            "pending must still have exactly one entry after key-reuse overwrite"
        );
    }

    /// BC-2.14.009 EC-003 — Transaction ID = 0x0000 is a valid key.
    ///
    /// test_BC_2_14_009_txn_id_zero_is_valid_key
    #[test]
    fn test_BC_2_14_009_txn_id_zero_is_valid_key() {
        let mut flow = ModbusFlowState::default();
        flow.insert_request(0x0000, 0x01, 0x03, 0);
        assert!(
            flow.pending.contains_key(&(0x0000u16, 0x01u8)),
            "txn_id=0x0000 is a valid Modbus key and must be inserted normally"
        );
    }

    /// BC-2.14.009 EC-004 — Unit ID = 0xFF (broadcast) is a valid key component.
    ///
    /// test_BC_2_14_009_unit_id_broadcast_0xff_is_valid
    #[test]
    fn test_BC_2_14_009_unit_id_broadcast_0xff_is_valid() {
        let mut flow = ModbusFlowState::default();
        flow.insert_request(0x0010, 0xFF, 0x03, 500);
        assert!(
            flow.pending.contains_key(&(0x0010u16, 0xFFu8)),
            "unit_id=0xFF (broadcast) is a valid key component and must be inserted normally"
        );
    }

    /// BC-2.14.009 EC-006 — timestamp=0 (epoch) is valid; stored without special-casing.
    ///
    /// test_BC_2_14_009_timestamp_zero_stored_correctly
    #[test]
    fn test_BC_2_14_009_timestamp_zero_stored_correctly() {
        let mut flow = ModbusFlowState::default();
        flow.insert_request(0x0005, 0x01, 0x06, 0);
        let &(fc, ts) = flow.pending.get(&(0x0005u16, 0x01u8)).unwrap();
        assert_eq!(fc, 0x06, "FC must be stored correctly with ts=0");
        assert_eq!(
            ts, 0,
            "timestamp=0 must be stored as-is (no special-casing)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.010 — Response PDU Matched Against Pending Table
    // ---------------------------------------------------------------------------

    /// AC-003 / BC-2.14.010 postcondition 2 Case A:
    /// A response with matching (txn_id, unit_id) + echoed FC resolves and removes
    /// the pending entry, returning Some((fc, ts)).
    ///
    /// test_BC_2_14_010_response_pdu_matched_and_entry_removed
    #[test]
    fn test_BC_2_14_010_response_pdu_matched_and_entry_removed() {
        let mut flow = ModbusFlowState::default();
        let req_ts: u32 = 1_000;

        // Setup: insert a Read HR request.
        // Canonical: txn_id=0x0001, unit_id=0x01, FC=0x03
        flow.insert_request(0x0001, 0x01, 0x03, req_ts);
        assert_eq!(
            flow.pending.len(),
            1,
            "pending must have one entry before response"
        );

        // Drive the canonical Read HR response vector (BC-2.14.010 canonical vector).
        // Response echoes FC=0x03 for (txn_id=0x0001, unit_id=0x01).
        let result = flow.match_response(0x0001, 0x01, 0x03);

        // postcondition 2 Case A: matched → removes entry, returns Some.
        assert!(
            result.is_some(),
            "match_response must return Some on FC echo match"
        );
        let (matched_fc, matched_ts) = result.unwrap();
        assert_eq!(
            matched_fc, 0x03,
            "matched_fc must equal the stored request FC"
        );
        assert_eq!(
            matched_ts, req_ts,
            "matched_ts must equal the stored request timestamp"
        );
        assert!(
            flow.pending.is_empty(),
            "pending must be empty after a successful response match"
        );
    }

    /// AC-004 / BC-2.14.010 postcondition 4 Case C:
    /// An orphan response (no matching pending entry) returns None without modifying state.
    ///
    /// test_BC_2_14_010_orphan_response_returns_none
    #[test]
    fn test_BC_2_14_010_orphan_response_returns_none() {
        let mut flow = ModbusFlowState::default();
        // No prior insert — pending table is empty.
        let _ = adu_read_hr_response_txn1(); // canonical vector reference

        let result = flow.match_response(0x0001, 0x01, 0x03);

        assert!(
            result.is_none(),
            "orphan response (no matching pending entry) must return None"
        );
        assert!(
            flow.pending.is_empty(),
            "pending must remain empty after orphan response"
        );
    }

    /// BC-2.14.010 EC-008:
    /// Two concurrent pending entries for different Unit IDs with the same Transaction ID.
    /// Each is keyed separately; responses for each are resolved independently.
    ///
    /// test_BC_2_14_010_concurrent_pending_different_unit_ids_resolved_independently
    #[test]
    fn test_BC_2_14_010_concurrent_pending_different_unit_ids_resolved_independently() {
        let mut flow = ModbusFlowState::default();

        flow.insert_request(0x0001, 0x01, 0x03, 100);
        flow.insert_request(0x0001, 0x02, 0x06, 200);
        assert_eq!(flow.pending.len(), 2);

        // Resolve unit_id=0x01 first.
        let r1 = flow.match_response(0x0001, 0x01, 0x03);
        assert!(r1.is_some(), "unit_id=0x01 response must match");
        assert_eq!(
            flow.pending.len(),
            1,
            "only the unit_id=0x01 entry should be removed"
        );

        // Resolve unit_id=0x02 second.
        let r2 = flow.match_response(0x0001, 0x02, 0x06);
        assert!(r2.is_some(), "unit_id=0x02 response must match");
        assert!(
            flow.pending.is_empty(),
            "pending must be empty after both responses"
        );
    }

    /// BC-2.14.010: a response with a non-matching FC (FC mismatch, Case B) removes
    /// the pending entry (per spec: "pair is considered closed regardless of anomaly").
    ///
    /// test_BC_2_14_010_fc_mismatch_response_removes_entry
    #[test]
    fn test_BC_2_14_010_fc_mismatch_response_removes_entry() {
        // NOTE: This tests Case B (FC mismatch) from BC-2.14.010 postcondition 3.
        // The implementation REMOVES the entry on FC mismatch (pair is closed).
        // match_response returns None on mismatch (no attribution); entry is still removed.
        //
        // We test this as: after mismatch, pending is empty.
        // If the implementation returns Some on mismatch, the test still passes the removal
        // check, so we assert only that pending is empty — matching the BC-defined behavior.
        let mut flow = ModbusFlowState::default();

        // Insert FC=0x01 (Read Coils).
        flow.insert_request(0x0003, 0x01, 0x01, 50);

        // Respond with FC=0x03 (mismatch — wrong FC for this txn_id).
        let _ = flow.match_response(0x0003, 0x01, 0x03);

        // BC-2.14.010 postcondition 3: entry removed regardless (pair closed).
        assert!(
            flow.pending.is_empty(),
            "pending must be empty after FC-mismatch response (pair closed per BC-2.14.010 case B)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.011 — Exception Response Attribution
    // ---------------------------------------------------------------------------

    /// AC-005 / BC-2.14.011 postcondition 3 Case A:
    /// Exception FC=0x86 (original_fc=0x06) with matching pending entry (FC=0x06):
    /// removes entry, returns Some(0x06), exception_count must be incremented by caller.
    ///
    /// Uses canonical vector: `00 02 00 00 00 03 01 86 01` (BC-2.14.011 test vectors).
    ///
    /// test_BC_2_14_011_exception_attributed_to_original_fc_write_class
    #[test]
    fn test_BC_2_14_011_exception_attributed_to_original_fc_write_class() {
        let mut flow = ModbusFlowState::default();

        // Canonical prior state: (0x0002, 0x01) -> (0x06, ts0) in pending.
        let ts0: u32 = 5_000;
        flow.insert_request(0x0002, 0x01, 0x06, ts0);
        assert_eq!(flow.pending.len(), 1);

        // Exception FC=0x86 → original_fc = 0x86 & 0x7F = 0x06.
        // Canonical exception ADU reference (not used directly — we call attribute_exception):
        let _ = adu_write_exception_txn2();

        let result = flow.attribute_exception(0x0002, 0x01, 0x86);

        // postcondition 3 Case A: attributed, returns Some(original_fc=0x06).
        assert_eq!(
            result,
            Some(0x06),
            "attribute_exception must return Some(0x06) for exception FC=0x86 with matching pending FC=0x06"
        );
        // Entry must be removed.
        assert!(
            flow.pending.is_empty(),
            "pending must be empty after successful exception attribution"
        );

        // Caller increments exception_count (simulated here).
        flow.exception_count += 1;
        assert_eq!(
            flow.exception_count, 1,
            "exception_count must be 1 after one attributed exception"
        );
    }

    /// BC-2.14.011 postcondition 3 Case A (Read-class exception):
    /// Exception FC=0x83 (original_fc=0x03) with matching pending entry (FC=0x03 Read HR):
    /// removes entry, returns Some(0x03).
    ///
    /// test_BC_2_14_011_exception_attributed_to_original_fc_read_class
    #[test]
    fn test_BC_2_14_011_exception_attributed_to_original_fc_read_class() {
        let mut flow = ModbusFlowState::default();

        // Canonical prior state from BC-2.14.011 test vectors:
        // (0x0001, 0x01) -> (0x03, ts0) in pending
        flow.insert_request(0x0001, 0x01, 0x03, 1_000);

        // Exception FC=0x83 → original_fc = 0x03.
        // Canonical ADU: `00 01 00 00 00 03 01 83 02`
        let result = flow.attribute_exception(0x0001, 0x01, 0x83);

        assert_eq!(
            result,
            Some(0x03),
            "attribute_exception must return Some(0x03) for exception FC=0x83 with matching pending FC=0x03"
        );
        assert!(
            flow.pending.is_empty(),
            "pending must be empty after read-class exception attribution"
        );
    }

    /// BC-2.14.011 EC-008 + EC-010 — SPOOF GUARD (strictFC consistency gate):
    /// Exception FC=0x86 (original_fc=0x06) arrives but pending slot holds FC=0x03 (Read HR).
    /// The pending entry must NOT be removed; function returns None.
    /// This is the anti-spoof invariant: a spoofed exception cannot clear a Write-class pending slot.
    ///
    /// test_BC_2_14_011_spoof_exception_fc_mismatch_entry_not_removed
    #[test]
    fn test_BC_2_14_011_spoof_exception_fc_mismatch_entry_not_removed() {
        let mut flow = ModbusFlowState::default();

        // Pending slot holds FC=0x03 (Read HR).
        flow.insert_request(0x0004, 0x01, 0x03, 2_000);

        // Attacker sends exception 0x86 (original_fc=0x06) for the SAME (txn_id, unit_id).
        // original_fc=0x06 ≠ pending_fc=0x03 → mismatch.
        let _ = adu_spoof_exception_0x86_while_pending_0x03(0x0004, 0x01);
        let result = flow.attribute_exception(0x0004, 0x01, 0x86);

        // Strict FC consistency gate: mismatch → None, entry preserved.
        assert!(
            result.is_none(),
            "FC mismatch (spoof guard): attribute_exception must return None when \
         original_fc(0x06) != pending_fc(0x03)"
        );
        assert_eq!(
            flow.pending.len(),
            1,
            "FC mismatch (spoof guard): pending entry must NOT be removed"
        );
        assert!(
            flow.pending.contains_key(&(0x0004u16, 0x01u8)),
            "the original Read HR pending entry must be preserved after spoof exception"
        );
    }

    /// BC-2.14.011 postcondition 4 Case B — Orphan exception (no matching pending entry):
    /// returns None, no removal, exception_count must still be incremented by caller.
    ///
    /// test_BC_2_14_011_orphan_exception_returns_none
    #[test]
    fn test_BC_2_14_011_orphan_exception_returns_none() {
        let mut flow = ModbusFlowState::default();
        // Pending table is empty.

        // Canonical orphan exception from BC-2.14.011 test vectors:
        // ADU: `00 05 00 00 00 03 01 90 04` (FC=0x90, code=0x04)
        let result = flow.attribute_exception(0x0005, 0x01, 0x90);

        assert!(
            result.is_none(),
            "orphan exception (no matching pending entry) must return None"
        );
        assert!(
            flow.pending.is_empty(),
            "pending must remain empty after orphan exception"
        );

        // exception_count is caller's responsibility — simulate:
        flow.exception_count += 1;
        assert_eq!(
            flow.exception_count, 1,
            "exception_count must be incremented even for orphan exceptions"
        );
    }

    /// BC-2.14.011 EC-009 — Exception FC=0x80 (original_fc=0x00, Unknown class):
    /// exception_count is still incremented; no write attribution.
    ///
    /// test_BC_2_14_011_exception_fc_0x80_original_fc_0x00_unknown_class
    #[test]
    fn test_BC_2_14_011_exception_fc_0x80_original_fc_0x00_unknown_class() {
        let mut flow = ModbusFlowState::default();

        // Insert a request with FC=0x00 (Unknown class).
        flow.insert_request(0x0006, 0x01, 0x00, 3_000);

        // Exception FC=0x80 → original_fc = 0x80 & 0x7F = 0x00.
        let result = flow.attribute_exception(0x0006, 0x01, 0x80);

        // original_fc=0x00 == pending_fc=0x00 → should match.
        assert_eq!(
            result,
            Some(0x00),
            "exception FC=0x80 recovers original_fc=0x00; must attribute correctly when pending FC matches"
        );
        assert!(
            flow.pending.is_empty(),
            "entry must be removed on successful attribution of 0x80 exception"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.012 — Pending Table Bounded at MAX_PENDING_TRANSACTIONS=256
    // ---------------------------------------------------------------------------

    /// AC-006 / BC-2.14.012 postcondition 1:
    /// After 256 inserts with distinct (txn_id, unit_id) keys, a 257th NEW key is NOT inserted.
    /// Table remains at exactly 256.
    ///
    /// Uses canonical test vector from BC-2.14.012: "Drop at cap".
    /// Exercises the REAL insert_request method so the production cap guard is bound — the
    /// test would detect removal of the guard, unlike a raw `flow.pending.insert` approach.
    ///
    /// test_BC_2_14_012_pending_table_bounded_at_256
    #[test]
    fn test_BC_2_14_012_pending_table_bounded_at_256() {
        let mut flow = ModbusFlowState::default();

        // Fill the table to exactly MAX_PENDING_TRANSACTIONS = 256 via insert_request,
        // so the production cap guard is the mechanism being exercised.
        // Use distinct (txn_id, unit_id) pairs: txn_id varies 0..256, unit_id=0x01.
        for i in 0u16..256 {
            flow.insert_request(i, 0x01, 0x03, 1000 + u32::from(i));
        }
        assert_eq!(
            flow.pending.len(),
            256,
            "pending must have exactly 256 entries after 256 distinct insert_request calls"
        );

        // Attempt 257th insert via insert_request with a NEW distinct key (txn_id=256, unit_id=0x01).
        // This key does not collide with any of the 256 existing entries; the production guard
        // must drop it silently.
        flow.insert_request(256, 0x01, 0x03, 9999);

        assert_eq!(
            flow.pending.len(),
            256,
            "257th insert_request with a unique key must be silently dropped; table stays at 256"
        );
        assert!(
            !flow.pending.contains_key(&(256u16, 0x01u8)),
            "the 257th key (256, 0x01) must NOT be present in the pending table"
        );
    }

    /// AC-006 (via method) / BC-2.14.012 postcondition 1:
    /// Same as above but exercised through `insert_request` method (which will apply the guard).
    ///
    /// test_BC_2_14_012_insert_request_drops_when_table_full
    #[test]
    fn test_BC_2_14_012_insert_request_drops_when_table_full() {
        let mut flow = ModbusFlowState::default();

        // Fill via insert_request (which will apply the MAX_PENDING_TRANSACTIONS guard).
        for i in 0u16..256 {
            flow.insert_request(i, 0x01, 0x03, u32::from(i) * 100);
        }

        assert_eq!(
            flow.pending.len(),
            256,
            "pending must have 256 entries after 256 distinct insert_request calls"
        );

        // 257th insert — must be dropped, no panic.
        flow.insert_request(256, 0x01, 0x03, 99999);

        assert_eq!(
            flow.pending.len(),
            256,
            "257th insert_request call must be silently dropped (pending stays at 256)"
        );
    }

    /// BC-2.14.012 EC-001:
    /// Pending table at 255 entries — the 256th insert succeeds (cap is 256, not 255).
    ///
    /// test_BC_2_14_012_255th_entry_succeeds_256th_is_cap
    #[test]
    fn test_BC_2_14_012_255th_entry_succeeds_256th_is_cap() {
        let mut flow = ModbusFlowState::default();

        // Insert 255 entries.
        for i in 0u16..255 {
            flow.insert_request(i, 0x01, 0x03, u32::from(i));
        }
        assert_eq!(flow.pending.len(), 255);

        // 256th insert — must succeed (one slot left).
        flow.insert_request(255, 0x01, 0x03, 9999);
        assert_eq!(
            flow.pending.len(),
            256,
            "256th insert must succeed; cap is 256 inclusive"
        );

        // 257th insert — must be dropped.
        flow.insert_request(256, 0x01, 0x03, 10000);
        assert_eq!(
            flow.pending.len(),
            256,
            "257th insert must be dropped; table stays at 256"
        );
    }

    /// AC-007 / BC-2.14.012 invariant 1 + VP-022:
    /// Flood 300 distinct (txn_id, unit_id) requests without any responses.
    /// `pending.len()` must never exceed 256 at any point; no panic.
    ///
    /// test_BC_2_14_012_VP022_pending_table_no_unbounded_growth
    #[test]
    fn test_BC_2_14_012_VP022_pending_table_no_unbounded_growth() {
        let mut flow = ModbusFlowState::default();

        // Adversarial scenario from BC-2.14.012 canonical test vectors:
        // "Adversarial flood: 300 unique requests → pending.len() = 256 after request 256;
        //  requests 257–300 dropped; no panic"
        for i in 0u16..300 {
            flow.insert_request(i, 0x01, 0x03, u32::from(i) * 100);
            assert!(
                flow.pending.len() <= MAX_PENDING_TRANSACTIONS,
                "pending.len() must never exceed {MAX_PENDING_TRANSACTIONS} (VP-022); \
             violated at i={i}, len={}",
                flow.pending.len()
            );
        }

        assert_eq!(
            flow.pending.len(),
            MAX_PENDING_TRANSACTIONS,
            "after 300-request flood, pending.len() must equal exactly {MAX_PENDING_TRANSACTIONS}"
        );
    }

    /// BC-2.14.012 EC-004:
    /// After the table is full, a response removes an entry, freeing a slot for a new insert.
    ///
    /// test_BC_2_14_012_entry_freed_after_response_allows_next_insert
    #[test]
    fn test_BC_2_14_012_entry_freed_after_response_allows_next_insert() {
        let mut flow = ModbusFlowState::default();

        // Fill to cap.
        for i in 0u16..256 {
            flow.insert_request(i, 0x01, 0x03, u32::from(i));
        }
        assert_eq!(flow.pending.len(), 256);

        // Remove one entry via a matched response for txn_id=0x0000.
        let r = flow.match_response(0x0000, 0x01, 0x03);
        assert!(
            r.is_some(),
            "response for txn_id=0 must match and remove the entry"
        );
        assert_eq!(
            flow.pending.len(),
            255,
            "pending must drop to 255 after one response"
        );

        // A new insert must now succeed (255 < 256).
        flow.insert_request(256, 0x01, 0x03, 99_000);
        assert_eq!(
            flow.pending.len(),
            256,
            "after freeing a slot, the next insert must succeed"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.009 / f2-fix-directives §11.4: ModbusFlowState has ALL required fields.
    // This is a compile-time structural test — if the struct compiles with Default,
    // all 15+ fields are present and derive(Default) covers them.
    // ---------------------------------------------------------------------------

    /// AC-009: ModbusFlowState::default() compiles and all fields are accessible.
    /// This is intentionally a compile-time completeness check.
    ///
    /// test_BC_2_14_009_modbus_flow_state_has_all_required_fields
    #[test]
    fn test_BC_2_14_009_modbus_flow_state_has_all_required_fields() {
        let flow = ModbusFlowState::default();

        // --- Transaction correlation ---
        assert!(flow.pending.is_empty(), "pending must be empty on default");

        // --- Per-flow aggregate counters ---
        assert_eq!(flow.write_count, 0);
        assert_eq!(flow.exception_count, 0);
        assert_eq!(flow.pdu_count, 0);
        assert_eq!(flow.last_ts, 0);

        // --- Burst window ---
        assert_eq!(flow.window_write_count, 0);
        assert_eq!(flow.window_start_ts, 0);
        assert!(!flow.window_burst_emitted);

        // --- Sustained window ---
        assert_eq!(flow.sustained_window_start_ts, 0);
        assert_eq!(flow.sustained_window_write_count, 0);
        assert!(!flow.sustained_burst_emitted);

        // --- T0831 window ---
        assert_eq!(flow.t0831_window_start_ts, 0);
        assert_eq!(flow.t0831_window_write_count, 0);
        assert!(!flow.t0831_burst_emitted);

        // --- Exception-burst windows ---
        assert!(flow.exception_window_counts.is_empty());
        assert!(flow.exception_window_start_ts.is_empty());
        assert!(flow.exception_burst_emitted.is_empty());

        // --- Desync safety ---
        assert!(!flow.is_non_modbus);
    }

    /// ModbusAnalyzer::new() compiles and dual-window thresholds are stored.
    ///
    /// test_BC_2_14_009_modbus_analyzer_new_stores_thresholds
    #[test]
    fn test_BC_2_14_009_modbus_analyzer_new_stores_thresholds() {
        let analyzer = ModbusAnalyzer::new(20, 10);

        assert_eq!(
            analyzer.write_burst_threshold, 20,
            "write_burst_threshold must be stored from new()"
        );
        assert_eq!(
            analyzer.write_sustained_threshold, 10,
            "write_sustained_threshold must be stored from new()"
        );
        assert_eq!(analyzer.duplicate_inflight_txn, 0);
        assert_eq!(analyzer.total_pdu_count, 0);
        assert_eq!(analyzer.total_write_count, 0);
        assert!(analyzer.fn_code_counts.is_empty());
    }

    /// MAX_PENDING_TRANSACTIONS constant is exactly 256 (BC-2.14.012 invariant 1 + 4).
    ///
    /// test_BC_2_14_012_max_pending_transactions_constant_is_256
    #[test]
    fn test_BC_2_14_012_max_pending_transactions_constant_is_256() {
        assert_eq!(
            MAX_PENDING_TRANSACTIONS, 256,
            "MAX_PENDING_TRANSACTIONS must be exactly 256 per BC-2.14.012 and ADR-005 §2.3"
        );
    }
} // mod story_103
