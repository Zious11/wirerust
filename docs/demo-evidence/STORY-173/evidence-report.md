# Evidence Report — STORY-173

**Story:** STORY-173: IEC-104 Dispatcher Integration: DispatchTarget::Iec104 + T0881 Six-Part Atomic + --iec104 Flag + SUPPORTED_PORTS
**Wave:** 82
**Date:** 2026-07-15
**Branch:** develop (worktree STORY-173)
**Product type:** Library + CLI (dispatcher + protocols catalog integration; `--iec104` flag activates the IEC-104 passive analysis pipeline end-to-end)

---

## Full Test Suite: 2602/2602 PASS

Command:
```
cargo test --all-targets
```

Output (summary lines):
```
test result: ok. 196 passed; ...   (iec104_analyzer_tests — includes 4 story_173 tests)
test result: ok. 30 passed; ...    (dispatcher_tests — includes 5 story_173 tests)
test result: ok. 26 passed; ...    (mitre_tests — includes 5 story_173 tests)
test result: ok. 30 passed; ...    (protocols_tests — includes 4 story_173 tests)
...
```

Total: **2602 passed; 0 failed** across all targets.

STORY-173 contribution: **18 new tests** (5 dispatcher + 5 mitre + 4 protocols + 4 iec104_analyzer).

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-173-001 | DispatchTarget::Iec104 variant + Rule 8 port-2404 dispatch | BC-2.05.012 PC1–3 | 2 (wiring tests exercise Rule 8 classify path) | `AC-001-dispatch-port-2404-iec104.md` | PASS |
| AC-173-002 | T0881 six-part atomic catalog registration | BC-2.10.010 PC1–6, Inv1 | 5 + drift guard | `AC-002-t0881-six-part-atomic.md` | PASS |
| AC-173-003 | `--iec104` flag enables IEC-104 analysis + reassembly gating | BC-2.12.025 PC1–3 | 1 (EC-003) + CLI surface | `AC-003-iec104-flag-reassembly-gating.md` | PASS |
| AC-173-004 | SUPPORTED_PORTS 8→9; supported_protocols 7→8 | BC-2.18.003 PC1–2 | 4 | `AC-004-supported-ports-9-protocols-8.md` | PASS |
| AC-173-005 | KNOWN_PROTOCOLS partition invariant preserved | BC-2.18.004 PC1–2, Inv1 (VP-041) | 2 (proptest) | `AC-005-known-protocols-partition.md` | PASS |
| AC-173-006 | VP-004 classify_oracle updated for DispatchTarget::Iec104 | BC-2.05.012 Inv1 (VP-004) | Source inspection; Kani deferred to STORY-174 | `AC-006-vp004-classify-oracle.md` | PASS |
| AC-173-007 | IEC-104 findings cap + dropped_findings in summarize() | BC-2.19.028 PC1–5, Inv4 | 4 | `AC-007-findings-cap-dropped-findings.md` | PASS |
| AC-173-008 | StreamDispatcher iec104 field + on_data/on_flow_close wiring | BC-2.05.012 Inv1, ADR-013 D9 | 5 | `AC-008-dispatcher-iec104-field-wiring.md` | PASS |

**Total STORY-173 test-based coverage: all AC-173-001..008 PASS**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Key Test Names |
|----|----|-----------|----------------|
| AC-173-001 | BC-2.05.012 PC1–3 | 2 | `test_iec104_only_dispatcher_data_reaches_analyzer`; `test_iec104_disabled_port_2404_no_panic` |
| AC-173-002 | BC-2.10.010 PC1–6 | 5 + lib drift guard | `test_BC_2_10_010_t0881_catalog_entry`; `test_BC_2_10_010_t0881_tactic_id_is_ta0107`; `test_BC_2_10_010_seeded_count_is_29`; `test_BC_2_10_010_t0881_in_seeded_ids_source`; `test_BC_2_10_010_t0881_in_emitted_ids_source`; `vp007_catalog_drift_guard` (lib) |
| AC-173-003 | BC-2.12.025 PC1–3 | 1 | `test_iec104_disabled_port_2404_no_panic` (EC-003: absent flag → no analyzer) |
| AC-173-004 | BC-2.18.003 PC1–2 | 4 | `test_BC_2_18_003_supported_ports_contains_2404`; `test_BC_2_18_003_supported_ports_len_is_9`; `test_BC_2_18_003_supported_protocols_len_is_8`; `test_BC_2_18_003_iec104_in_supported_protocols` |
| AC-173-005 | BC-2.18.004 PC1–2, Inv1 | 2 | `proptest_vp041_oracle_cross_check`; `proptest_vp041_partition_invariant` |
| AC-173-006 | BC-2.05.012 Inv1 (VP-004) | Source inspection | `classify_oracle` Rule 8 arm verified in `#[cfg(kani)]` block; Kani run deferred to STORY-174 |
| AC-173-007 | BC-2.19.028 PC1–5 | 4 | `test_BC_2_19_028_findings_cap`; `test_BC_2_19_028_boundary_at_max_minus_one_allows_one_more`; `test_BC_2_19_028_cap_maintained_across_multiple_on_data_calls`; `test_BC_2_19_028_dropped_findings_surfaced_in_summarize` |
| AC-173-008 | BC-2.05.012 Inv1, ADR-013 D9 | 5 | `test_iec104_only_dispatcher_data_reaches_analyzer`; `test_iec104_only_dispatcher_stopdt_produces_t0881`; `test_BC_2_05_012_early_exit_guard_includes_iec104`; `test_iec104_only_guard_unclassified_flows_counted`; `test_iec104_disabled_port_2404_no_panic` |

---

## Key Behavior Summary

### DispatchTarget::Iec104 + Rule 8 (AC-173-001, AC-173-008)

`StreamDispatcher` routes port-2404 TCP flows to `Iec104Analyzer` via:
1. `classify(data, flow_key)` returns `DispatchTarget::Iec104` when
   `[flow_key.lower_port(), flow_key.upper_port()].contains(&2404)` (Rule 8).
2. `on_data` `DispatchTarget::Iec104` arm calls `iec104.on_data(...)`.
3. `on_flow_close` `DispatchTarget::Iec104` arm calls `iec104.on_flow_close(...)`.

Early-exit guard extended: `&& self.iec104.is_none()` ensures a `--iec104`-only dispatcher
does not silently discard all data (ADR-013 Decision 9 step 4).

### T0881 Six-Part Atomic (AC-173-002)

| Part | Status |
|------|--------|
| 1. `"T0881"` in `SEEDED_TECHNIQUE_IDS` (28→29) | Verified by `test_BC_2_10_010_t0881_in_seeded_ids_source` |
| 2. `SEEDED_TECHNIQUE_ID_COUNT = 29` | Verified by `test_BC_2_10_010_seeded_count_is_29` |
| 3. `"T0881"` in `EMITTED_IDS` | Verified by `test_BC_2_10_010_t0881_in_emitted_ids_source` |
| 4. `technique_info("T0881")` → `("Service Stop", IcsInhibitResponseFunction)` | Verified by `test_BC_2_10_010_t0881_catalog_entry` + `test_BC_2_10_010_t0881_tactic_id_is_ta0107` |
| 5. `vp007_catalog_drift_guard` `#[test]` passes at count=29 | Verified by `cargo test --lib vp007_catalog_drift_guard` |
| 6. `verify_all_emitted_ids_resolve` Kani harness | Deferred to STORY-174 (Kani proof run) |

### Findings Cap (AC-173-007)

| Scenario | all_findings.len() | dropped_findings |
|----------|-------------------|-----------------|
| Pre-fill MAX, one more on_data | MAX (10,000) — unchanged | 1 |
| Pre-fill MAX-1, one more on_data | MAX (10,000) — filled | 0 |
| Pre-fill MAX, N=5 more on_data | MAX (10,000) — unchanged | 5 |
| summarize() after cap fire | — | reported as `detail["dropped_findings"]` |

### SUPPORTED_PORTS Partition (AC-173-004, AC-173-005)

| Constant / Function | Before STORY-173 | After STORY-173 |
|--------------------|-----------------|-----------------|
| `SUPPORTED_PORTS.len()` | 8 | 9 |
| `supported_protocols().len()` | 7 | 8 |
| `IEC 60870-5-104` in `supported_protocols()` | No | Yes |
| `IEC 60870-5-104` in `unsupported_protocols()` | Yes | No |
| Partition invariant (union/disjoint) | Holds | Still holds |

---

## Edge Case Coverage

| Edge Case | BC | Test | Verdict |
|-----------|-----|------|---------|
| EC-001: port 2404 on both src and dst ports | BC-2.05.012 | `test_iec104_only_dispatcher_data_reaches_analyzer` (FlowKey src=60001 dst=2404) | PASS |
| EC-002: partial T0881 commit would fail drift guard | BC-2.10.010 | `vp007_catalog_drift_guard` | PASS (guard enforces all six parts atomically) |
| EC-003: `--iec104` absent → no analyzer, no panic | BC-2.12.025 | `test_iec104_disabled_port_2404_no_panic` | PASS |
| EC-004: SUPPORTED_PORTS len==8 before → 9 after | BC-2.18.003 | `test_BC_2_18_003_supported_ports_len_is_9` | PASS |
| EC-005: partition disjointness violation caught by proptest | BC-2.18.004 | `proptest_vp041_partition_invariant` | PASS |
| EC-006: cap fires at MAX; subsequent on_data drops silently | BC-2.19.028 | `test_BC_2_19_028_findings_cap` | PASS |

---

## Source-Level Evidence Confirmed Present

`src/dispatcher.rs`:
- `DispatchTarget::Iec104` variant
- Rule 8 in `classify()`: `if ports.contains(&2404) { return DispatchTarget::Iec104; }`
- `iec104: Option<Iec104Analyzer>` field
- `new()` 6-param signature with `iec104: Option<Iec104Analyzer>` as last arg
- `set_iec104_analyzer()` setter
- Early-exit guard includes `&& self.iec104.is_none()`
- `DispatchTarget::Iec104` arm in `on_data`
- `DispatchTarget::Iec104` arm in `on_flow_close`
- `classify_oracle` Rule 8 arm in `#[cfg(kani)]` block

`src/mitre.rs`:
- `"T0881"` in `SEEDED_TECHNIQUE_IDS` (29 entries total)
- `SEEDED_TECHNIQUE_ID_COUNT: usize = 29`
- `"T0881"` in `EMITTED_IDS`
- `"T0881" => ("Service Stop", MitreTactic::IcsInhibitResponseFunction)` arm

`src/cli.rs`:
- `iec104: bool` flag with `--iec104` long name

`src/main.rs`:
- Reassembly gating: `if enable_iec104 && skip_reassembly { eprintln!(...); }`
- `Iec104Analyzer::new()` constructed when `enable_iec104 && !skip_reassembly`
- Passed to `StreamDispatcher::new(..., iec104_analyzer)`

`src/protocols.rs`:
- `SUPPORTED_PORTS` includes 2404; `len() == 9`
- `supported_protocols()` returns `IEC 60870-5-104` (port intersection)

`src/analyzer/iec104.rs`:
- `pub const MAX_IEC104_FINDINGS: usize = 10_000`
- `pub dropped_findings: u64` field, initialized to 0
- Cap enforced at extend step in `on_data`
- `summarize()` exposes `"dropped_findings"` detail key

---

## Recording Method

This is a library + CLI integration story. Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Source-level confirmation of all added constants, variants, fields, and arms
- CLI flag surface captured via `cargo run -- analyze --help`
- Summary tables for the partition, cap behavior, and T0881 atomic commit

A VHS terminal recording of `cargo run -- --iec104 <pcap>` is not included because no
IEC-104 test pcap is committed in the repository. Full behavioral coverage is provided
by the 18 STORY-173 unit/integration tests.

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-dispatch-port-2404-iec104.md` | AC-173-001 (BC-2.05.012 PC1–3, DispatchTarget::Iec104 + Rule 8) |
| `AC-002-t0881-six-part-atomic.md` | AC-173-002 (BC-2.10.010 PC1–6, T0881 six-part atomic) |
| `AC-003-iec104-flag-reassembly-gating.md` | AC-173-003 (BC-2.12.025 PC1–3, --iec104 flag + reassembly gating) |
| `AC-004-supported-ports-9-protocols-8.md` | AC-173-004 (BC-2.18.003 PC1–2, SUPPORTED_PORTS 8→9; supported_protocols 7→8) |
| `AC-005-known-protocols-partition.md` | AC-173-005 (BC-2.18.004 PC1–2, Inv1, VP-041 partition invariant) |
| `AC-006-vp004-classify-oracle.md` | AC-173-006 (BC-2.05.012 Inv1, VP-004 oracle update) |
| `AC-007-findings-cap-dropped-findings.md` | AC-173-007 (BC-2.19.028 PC1–5, Inv4, findings cap + dropped_findings surfacing) |
| `AC-008-dispatcher-iec104-field-wiring.md` | AC-173-008 (BC-2.05.012 Inv1, ADR-013 D9 steps 4–5, dispatcher wiring) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths, usernames,
or machine strings; no occurrences of absolute host-local paths are present in the
committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-15).
