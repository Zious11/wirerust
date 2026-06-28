---
artifact: verification-property
vp_id: VP-037
title: "Modbus Carry-Direction Isolation"
status: draft
phase: P1
tool: proptest
subsystem: SS-14
module: "src/analyzer/modbus.rs"
producer: spec-steward
timestamp: 2026-06-28T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-005-modbus-tcp-analysis.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.14.002
ruling: RULING-MODBUS-SIBLING-001
verification_lock: false
---

# VP-037: Modbus Carry-Direction Isolation

## Property Statement

For any sequence of `on_data` calls with alternating `Direction::ClientToServer` (c2s,
Modbus master/client) and `Direction::ServerToClient` (s2c, Modbus slave/server)
deliveries carrying partial ADUs, the `carry_c2s` and `carry_s2c` buffers are
**never mixed**:

- A partial master-to-server ADU stashed in `carry_c2s` is NEVER prepended to a
  subsequent server-to-client delivery.
- A partial server-to-client ADU stashed in `carry_s2c` is NEVER prepended to a
  subsequent master-to-server delivery.
- Per-direction FC parse results (`fn_code_counts` contributions from each direction)
  match the counts produced by independent same-direction control runs: injecting
  interleaved s2c deliveries between c2s partial-ADU deliveries does NOT alter the c2s
  FC parse results, and vice versa.

Formally: for any (c2s_adu, s2c_adu) pair of ADU sequences with MBAP segment boundaries,
running them interleaved through `on_data` with alternating directions produces the same
`fn_code_counts` and `parse_errors` as running each direction's ADUs independently.

This directly prevents the cross-direction carry splice bug documented as
DRIFT-MODBUS-DIRECTION-001 in RULING-MODBUS-SIBLING-001: a partial master-to-server
MBAP header stashed in `carry_c2s` cannot corrupt the next server-to-client delivery.
The empirical repro (`scratch_EC_X1_splice_confirmed_garbled_write_fires_on_s2c_direction`,
commit 74f2913) proves the splice: before the fix, the interleaved run produced
`fn_code_counts[0x06] == 1` (garbled Write FC from spliced carry) instead of the
correct `fn_code_counts[0x03] == 1` (FC=0x03 Read Holding Registers).

## Verified BCs

| BC-ID | Description | How VP-037 Covers It |
|-------|-------------|----------------------|
| BC-2.14.002 v2.0 | `ModbusFlowState` MBAP partial-ADU carry — per-direction split (`carry_c2s`, `carry_s2c`) | VP-037 verifies Invariant 4 (direction isolation): `carry_c2s` and `carry_s2c` are never mixed; `on_data` selects exactly one buffer per `direction` argument on every call. Also covers EC-007 (partial c2s MBAP stashed in carry_c2s; next s2c call uses carry_s2c — clean s2c parse; carry_c2s retains c2s partial). |

### BC Version Note

BC-2.14.002 was amended from v1.0 to v2.0 by RULING-MODBUS-SIBLING-001. The v2.0
amendments relevant to VP-037 are:

- Precondition 3 (carry prepend): `buf = (match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`
- Postcondition 1 (carry stash): `match direction { ClientToServer => flow.carry_c2s = remaining, ServerToClient => flow.carry_s2c = remaining }`
- Invariant 1 (carry bounded): `flow.carry_c2s.len() <= MAX_ADU_CARRY_BYTES = 260` AND `flow.carry_s2c.len() <= MAX_ADU_CARRY_BYTES = 260`
- Invariant 4 (new): `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects exactly one of the two buffers based on the `direction` argument on every call
- EC-007 (new): Partial c2s MBAP (< 8 bytes) stashed in carry_c2s; next call is s2c direction → `carry_s2c` (empty) prepended to s2c data → clean s2c parse; `carry_c2s` retains c2s partial

## Relationship to VP-033 (ENIP Analog) and VP-035 (DNP3 Analog)

VP-037 is structurally identical to VP-033 (EtherNet/IP Carry-Buffer Direction Isolation)
and VP-035 (DNP3 Carry-Buffer Direction Isolation), but targets `src/analyzer/modbus.rs`.
The same two-harness proptest strategy applies. Key differences from VP-035 (DNP3):

- **Frame format:** Modbus uses an 8-byte MBAP header (TxnID[2] + ProtoID[2] + Length[2] +
  UnitID[1] + FC[1]) rather than DNP3's 10-byte DL header. Partial ADU is any prefix < 8
  bytes (MBAP header minimum).
- **Metric:** `fn_code_counts` (per-FC tally) rather than `frame_count`. The proptest
  asserts FC correctness via `fn_code_counts[fc] == 1` for the expected FC, and that no
  garbled FC appears (the DRIFT-MODBUS-DIRECTION-001 symptom was `fn_code_counts[0x06] == 1`
  instead of `fn_code_counts[0x03] == 1`).
- **BC trace:** BC-2.14.002 v2.0 Invariant 4 (vs. BC-2.17.016 v2.0 Invariant 7 for ENIP;
  BC-2.15.016 v2.0 Invariant 6 for DNP3).
- **No FIR gate:** Modbus has no equivalent of DNP3's FIR=1 transport-layer gate. Direction
  isolation operates directly at the MBAP/ADU carry level.
- **`direction` param already present:** Unlike ENIP (which required a signature change),
  Modbus `on_data` already received `direction: Direction` in the pre-fix code. The fix is
  purely additive within `modbus.rs` (no dispatcher.rs changes).

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives
`ModbusFlowState` directly without any I/O. The test constructs synthetic byte sequences
(valid MBAP headers with known FC values), calls `on_data` with alternating directions,
and asserts FC count invariants. No file I/O, no network, no global state.

**Why proptest and NOT Kani:** The direction-isolation invariant is trivially enforced by
the `match direction { ClientToServer => ..., ServerToClient => ... }` carry-select in the
production code. The operationally meaningful check is the behavioral invariant: that
interleaving s2c deliveries between c2s partial-ADU deliveries does NOT corrupt FC counts.
This is a state-machine property over sequences — the natural domain of proptest. The
DRIFT-MODBUS-DIRECTION-001 repro (commit 74f2913) was a sequence-based scenario; proptest
directly encodes the same evidence.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp037_modbus_carry_direction_isolation {
    use super::*;
    use proptest::prelude::*;

    /// VP-037: carry_c2s and carry_s2c are never mixed across directions (Modbus).
    ///
    /// Strategy: generate a partial c2s MBAP prefix (bytes 0..split_offset, where
    /// split_offset < 8 — MBAP header minimum), a complete s2c MBAP ADU (FC=0x03
    /// Read Holding Registers, 13 bytes total), and the completing c2s bytes.
    ///
    /// Assert: after the interleaved run, fn_code_counts[c2s_fc] == 1 (correct c2s FC),
    /// fn_code_counts[s2c_fc] == 1 (correct s2c FC), parse_errors == 0, no garbled FC.
    ///
    /// This directly encodes the DRIFT-MODBUS-DIRECTION-001 repro: before the fix, the
    /// interleaved run produced fn_code_counts[0x06] == 1 (garbled Write FC from spliced
    /// carry) instead of fn_code_counts[0x03] == 1 (correct s2c FC).
    proptest! {
        #[test]
        fn proptest_vp037_direction_isolation_fn_code_counts(
            // Segment split point: where to cut the c2s ADU's first delivery.
            // Range 0..6 (values 0,1,2,3,4,5) — each is a partial MBAP prefix strictly
            // below the 8-byte MBAP minimum, so the first delivery never completes an ADU.
            // split_offset == 0 is the degenerate case: deliver nothing in delivery 1
            // (the full c2s ADU is delivered whole in delivery 3, no c2s carry involved).
            // split_offset in 1..=5 exercises the carry-stash/prepend path (partial c2s
            // MBAP stashed in carry_c2s while a full s2c ADU is processed via carry_s2c).
            // Reconciled to implementation reality at F7 (DIM3-01): impl + STORY-141
            // AC-141-011 use 0usize..6; the skeleton previously used 1usize..7.
            split_offset in 0usize..6,
        ) {
            let mut state = ModbusFlowState::default();
            let flow_key = FlowKey::test_key(); // synthetic flow key
            let ts: u32 = 100;

            // Build a complete c2s ADU: TxnID=0x0001, ProtoID=0, Length=6, UnitID=1, FC=0x06 (Write)
            // FC=0x06 = Write Single Register (write-class, direction: ClientToServer)
            let c2s_adu = build_minimal_modbus_adu(0x0001u16, 0x06u8); // 13 bytes total
            // Build a complete s2c ADU: TxnID=0x0006, ProtoID=0, Length=6, UnitID=1, FC=0x03 (Read)
            // FC=0x03 = Read Holding Registers response (direction: ServerToClient)
            let s2c_adu = build_minimal_modbus_adu(0x0006u16, 0x03u8); // 13 bytes total

            // Delivery 1: partial c2s MBAP prefix (bytes 0..split_offset) → stashed in carry_c2s.
            // split_offset == 0 is degenerate: deliver nothing (no c2s carry stashed); the
            // full c2s ADU is then delivered whole in delivery 3.
            if split_offset > 0 {
                state.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, ts);
                prop_assert_eq!(state.parse_errors, 0,
                    "partial delivery must not produce parse errors");
            }

            // Delivery 2: complete s2c ADU → carry_s2c used (carry_c2s NOT involved)
            state.on_data_test(&flow_key, Direction::ServerToClient, &s2c_adu, 0, ts);
            prop_assert_eq!(state.parse_errors, 0,
                "s2c delivery must not produce parse errors from c2s carry contamination");

            // Delivery 3: completing c2s bytes (split_offset..end) → carry_c2s prepended
            state.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, ts);
            prop_assert_eq!(state.parse_errors, 0,
                "completing c2s delivery must not produce parse errors");

            // Verify correct FC distribution — no garbled FCs from carry splice
            let fc_03_count = state.fn_code_counts.get(&0x03u8).copied().unwrap_or(0);
            let fc_06_count = state.fn_code_counts.get(&0x06u8).copied().unwrap_or(0);
            prop_assert_eq!(fc_03_count, 1,
                "FC=0x03 (s2c read) must appear exactly once (not garbled by carry splice)");
            prop_assert_eq!(fc_06_count, 1,
                "FC=0x06 (c2s write) must appear exactly once");
        }

        #[test]
        fn proptest_vp037_independent_run_equivalence(
            // Range 0..6 — see strategy note on the first harness above. split_offset == 0
            // is the degenerate (whole-ADU) case; 1..=5 exercise the carry path.
            split_offset in 0usize..6,
        ) {
            let flow_key = FlowKey::test_key();
            let ts: u32 = 100;
            let c2s_adu = build_minimal_modbus_adu(0x0001u16, 0x06u8);
            let s2c_adu = build_minimal_modbus_adu(0x0006u16, 0x03u8);

            // Interleaved run (skip the empty partial delivery when split_offset == 0)
            let mut interleaved = ModbusFlowState::default();
            if split_offset > 0 {
                interleaved.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, ts);
            }
            interleaved.on_data_test(&flow_key, Direction::ServerToClient, &s2c_adu, 0, ts);
            interleaved.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, ts);

            // Independent c2s-only run
            let mut c2s_only = ModbusFlowState::default();
            if split_offset > 0 {
                c2s_only.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, ts);
            }
            c2s_only.on_data_test(&flow_key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, ts);

            // Independent s2c-only run
            let mut s2c_only = ModbusFlowState::default();
            s2c_only.on_data_test(&flow_key, Direction::ServerToClient, &s2c_adu, 0, ts);

            // Invariant: interleaved fn_code_counts == union of independent runs
            let interleaved_fc_03 = interleaved.fn_code_counts.get(&0x03u8).copied().unwrap_or(0);
            let interleaved_fc_06 = interleaved.fn_code_counts.get(&0x06u8).copied().unwrap_or(0);
            let c2s_fc_06 = c2s_only.fn_code_counts.get(&0x06u8).copied().unwrap_or(0);
            let s2c_fc_03 = s2c_only.fn_code_counts.get(&0x03u8).copied().unwrap_or(0);

            prop_assert_eq!(interleaved_fc_06, c2s_fc_06,
                "interleaved c2s FC=0x06 count must equal independent c2s-only count");
            prop_assert_eq!(interleaved_fc_03, s2c_fc_03,
                "interleaved s2c FC=0x03 count must equal independent s2c-only count");
            prop_assert_eq!(interleaved.parse_errors,
                c2s_only.parse_errors + s2c_only.parse_errors,
                "interleaved parse_errors must equal sum of independent runs");
        }
    }

    /// Build a minimal valid Modbus TCP ADU (MBAP header + PDU).
    /// Layout: TxnID[2] + ProtoID[2] + Length[2] + UnitID[1] + FC[1] + Data[5]
    /// Total: 13 bytes. Length field = 6 (UnitID + FC + 4 data bytes, but we
    /// use Length=6 to satisfy the [2,254] gate: UnitID(1) + FC(1) + data(4)).
    fn build_minimal_modbus_adu(txn_id: u16, fc: u8) -> Vec<u8> {
        let txn_hi = (txn_id >> 8) as u8;
        let txn_lo = (txn_id & 0xFF) as u8;
        vec![
            txn_hi, txn_lo,     // Transaction ID (big-endian per Modbus TCP spec)
            0x00, 0x00,         // Protocol ID = 0 (Modbus)
            0x00, 0x06,         // Length = 6 (UnitID + FC + 4 data bytes)
            0x01,               // Unit ID = 1
            fc,                 // Function Code
            0x00, 0x01,         // Data byte 1-2 (register address)
            0x00, 0x01,         // Data byte 3-4 (register count / value)
            0xFF,               // Data byte 5 (padding to reach adu_len=12)
        ]
    }
}
```

### Implementation Notes

- `on_data_test` is a thin test-visible wrapper around the production `on_data` logic.
  It mirrors the pattern used for VP-033/VP-035 wrappers in `EnipFlowState`/`Dnp3FlowState`.
- `build_minimal_modbus_adu` produces a 13-byte ADU satisfying both the MBAP gate (Length
  in [2,254]) and providing a unique FC byte at offset 7.
- The `split_offset in 0..6` range keeps the first delivery a partial MBAP prefix
  (strictly below the 8-byte minimum) for offsets 1..=5, forcing the carry-stash path;
  offset 0 is the degenerate whole-ADU case (first delivery skipped). This range matches
  the implementation and STORY-141 AC-141-011 (reconciled at F7, DIM3-01). The earlier
  skeleton used `1..7`; both ranges stay within the partial-MBAP domain, so the property
  was covered either way (the audit finding was advisory/NON-BLOCKING).
- `fn_code_counts` is the canonical metric: the DRIFT-MODBUS-DIRECTION-001 splice symptom
  was an incorrect FC appearing in `fn_code_counts` due to carry contamination.
- The `FlowKey::test_key()` helper (or equivalent) creates a synthetic flow key for the
  state injection test. The exact method depends on the production API.
- Proptest default test count (100 cases) is sufficient; the direction-isolation property
  is structural, not numeric.

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **State-machine test over synthetic sequences:** `ModbusFlowState` is driven by
   synthetic byte sequences, making it deterministic and reproducible. No external I/O.

2. **Bounded input space:** `split_offset` ranges over 0..6 (6 values: 0,1,2,3,4,5). The
   proptest strategy covers the full variation space in a modest number of cases. The
   range is non-vacuous (DF-KANI-NONVACUITY-001): offsets 1..=5 reach the carry-stash and
   carry-prepend code paths with a genuine partial c2s MBAP held in `carry_c2s` while a
   full s2c ADU is processed via `carry_s2c`; offset 0 still asserts the interleaved FC
   counts and `parse_errors == 0`. The assertions discriminate (they pass on the fixed
   implementation and fail on the pre-fix splice stub), so no input is filtered away and
   the property statement is not trivially true.

3. **DRIFT-MODBUS-DIRECTION-001 repro equivalence:** The two-harness proptest strategy
   directly encodes the repro scenario from RULING-MODBUS-SIBLING-001 §1.1. The first
   harness (FC isolation + no parse_errors) is the minimal correctness assertion; the second
   (independent-run equivalence) is the general direction-isolation invariant.

4. **Structural correctness:** After the carry split, the two-buffer match-arm makes this
   property trivially satisfied by construction. The proptest is a regression guard.

5. **Precedent:** VP-033 (EnipFlowState carry-direction isolation) and VP-035
   (Dnp3FlowState carry-direction isolation) use the identical proptest pattern. VP-037
   is the Modbus sibling.

**Not Kani because:** The property is about behavioral isolation over sequences, not about
arithmetic safety of individual functions. Kani proves properties of individual calls; the
carry-direction invariant is a property of call sequences — the natural domain of proptest.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-037 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to STORY-141 (Modbus carry/clock fix) | draft |
| F4 (TDD implementation) | `proptest_vp037_direction_isolation_fn_code_counts` + `proptest_vp037_independent_run_equivalence` authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |
| F7 (delta convergence) | Skeleton `split_offset` range reconciled `1usize..7` → `0usize..6` to match implementation + STORY-141 AC-141-011 (audit finding DIM3-01, NON-BLOCKING/advisory); property remains non-vacuous (DF-KANI-NONVACUITY-001) | draft (no lock change) |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-037 is added (after VP-036):
- `total_vps`: 36 → 37
- `p1_count`: 22 → 23
- `proptest_count`: 14 → 15
- `draft` count: 5 → 6 (VP-032, VP-033, VP-034, VP-035, VP-036, VP-037 all draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-037

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + analyzer/modbus.rs Per-Module row + Totals row)
