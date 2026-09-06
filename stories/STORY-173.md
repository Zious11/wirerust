---
document_type: story
story_id: STORY-173
title: "IEC-104 Dispatcher Integration: DispatchTarget::Iec104 + T0881 Six-Part Atomic + --iec104 Flag + SUPPORTED_PORTS"
epic_id: E-22
wave: 82
points: 5
phase: f3
tdd_mode: strict
status: draft
version: "2.1"
modified:
  - date: 2026-07-15
    actor: sw-agent
    reason: "SR-173-01..08 BC-realignment: technique_info tactic corrected to MitreTactic::IcsInhibitResponseFunction (TA0107); BC-2.19.028 (IEC-104 findings cap) added as AC-173-007 + inputs; dispatcher iec104-field/guard/arm wiring (Decision 9 steps 4-5) added as AC-173-008; SUPPORTED_PORTS count corrected 8→9 / supported_protocols 7→8 (were conflated as 7→8); SEEDED_TECHNIQUE_IDS/SEEDED_TECHNIQUE_ID_COUNT names corrected (story had stale SEEDED_TECHNIQUE_COUNT); vp007_catalog_drift_guard corrected to #[test] not Kani; classify() signature corrected to (data, flow_key); Rule 9 no-match renumber noted; verify_all_seeded_ids_resolve added at count=29."
  - date: 2026-07-16
    actor: prose-fix-agent
    reason: "v2.1 (2026-07-16): F-173-501 prose accuracy — AC-173-005 corrected to KNOWN_PROTOCOLS partition (supported ∪ unsupported) per BC-2.18.004 + VP-041; removed nonexistent UNMONITORED_PORTS reference."
feature_id: feature-iec104
subsystems: [SS-05, SS-10, SS-12, SS-18, SS-19]
target_module: analyzer/iec104
depends_on: [STORY-172]
blocks: [STORY-174]
behavioral_contracts:
  - BC-2.05.012
  - BC-2.10.010
  - BC-2.12.025
  - BC-2.18.003
  - BC-2.18.004
  - BC-2.19.028
verification_properties:
  - VP-004
  - VP-007
  - VP-041
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.012.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.010.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.025.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.028.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "d04afd8"
---

# STORY-173: IEC-104 Dispatcher Integration: DispatchTarget::Iec104 + T0881 Six-Part Atomic + --iec104 Flag + SUPPORTED_PORTS

## Narrative

**As a** wirerust user and security analyst,
**I want** the IEC-104 analyzer wired into the stream dispatcher with a `--iec104` CLI flag,
T0881 registered in the MITRE catalog, and port 2404 added to `SUPPORTED_PORTS`,
**so that** the full IEC-104 passive analysis pipeline is activated end-to-end and the T0881
"Service Stop" technique is formally recorded in the MITRE catalog with all six atomic parts
committed together.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.05.012 | `DispatchTarget::Iec104` Rule 8 Port-2404 Dispatch | SS-05 dispatcher — add Iec104 variant + Rule 8 |
| BC-2.10.010 | T0881 Six-Part Atomic Commit in mitre.rs | SS-10 MITRE catalog — T0881 atomic registration |
| BC-2.12.025 | `--iec104` CLI Flag Enables IEC-104 Analysis | SS-12 CLI — new flag |
| BC-2.18.003 | SUPPORTED_PORTS Includes Port 2404 (Count 7→8) | SS-18 protocols catalog — 2404 added |
| BC-2.18.004 | Protocol Catalog Partition Invariant Preserved | SS-18 invariant — partition still valid after 2404 |
| BC-2.19.028 | MAX_IEC104_FINDINGS DoS Bound — Finding Cap Prevents Unbounded Growth | SS-19 analyzer — findings cap at `on_data` extend step |

## Acceptance Criteria

### AC-173-001: DispatchTarget::Iec104 variant added and Rule 8 routes port 2404
**Traces to:** BC-2.05.012 postconditions 1–3
- Given the `StreamDispatcher` in `src/dispatcher.rs`
- When a TCP flow on port 2404 is classified
- Then `classify(data, flow_key)` returns `DispatchTarget::Iec104` (Rule 8, matching port 2404 via `[flow_key.lower_port(), flow_key.upper_port()].contains(&2404)`)
- `DispatchTarget::Iec104` variant is added to the `DispatchTarget` enum
- The VP-004 `classify_oracle` in the `#[cfg(kani)]` block is updated to include the Iec104 arm
  (ADR-013 Decision 9 — VP-004 oracle must be updated atomically with DispatchTarget::Iec104)

### AC-173-002: T0881 registered in mitre.rs via six-part atomic commit
**Traces to:** BC-2.10.010 postconditions 1–6 and invariant 1 (ADR-013 Decision 10)
- The following six changes MUST be delivered in a SINGLE git commit (six-part atomic):
  1. `"T0881"` added to `SEEDED_TECHNIQUE_IDS` array (28 → 29 entries)
  2. `SEEDED_TECHNIQUE_ID_COUNT` constant bumped from 28 to 29
  3. `EMITTED_IDS` array gains `"T0881"` entry (IEC-104 STOPDT findings emit this technique)
  4. `technique_info("T0881")` match arm returning `("Service Stop", MitreTactic::IcsInhibitResponseFunction)` — tactic is `IcsInhibitResponseFunction` (TA0107), not the string `"impact"`
  5. `vp007_catalog_drift_guard` `#[test]` (not Kani — run with `cargo test vp007_catalog_drift_guard`) passes at count=29; `verify_all_seeded_ids_resolve` (BC-2.10.010 PC-4) also passes at count=29
  6. `verify_all_emitted_ids_resolve` Kani harness passes for T0881 (EMITTED postcondition)
- Partial commits that pass tests but miss the count bump, EMITTED_IDS, or drift-guard update
  are NOT acceptable — all six parts must land together

### AC-173-003: `--iec104` CLI flag enables IEC-104 analysis
**Traces to:** BC-2.12.025 postconditions 1–3
- Given `cargo run -- --iec104 <pcap>`
- When wirerust processes a pcap containing IEC-104 traffic on port 2404
- Then `Iec104Analyzer` is instantiated and registered with the dispatcher
- Without `--iec104`, no IEC-104 analysis is performed (flag-gated per opt-in model)

### AC-173-004: SUPPORTED_PORTS includes port 2404 (SUPPORTED_PORTS 8→9; supported_protocols 7→8)
**Traces to:** BC-2.18.003 postconditions 1–2
- Given the `SUPPORTED_PORTS` constant in `src/protocols.rs`
- When `SUPPORTED_PORTS.contains(&2404)` is checked
- Then it returns `true`; `SUPPORTED_PORTS.len()` increases from 8 to 9
  (regression test: `protocols_tests.rs` line ~111, count → 9)
- `supported_protocols().len()` increases from 7 to 8
  (regression test: `protocols_tests.rs` line ~562, count → 8)
- Note: the `IEC 60870-5-104` entry already exists in `KNOWN_PROTOCOLS` as an unsupported
  entry; adding port 2404 to `SUPPORTED_PORTS` is the only change required to `protocols.rs`

### AC-173-005: Protocol catalog partition invariant preserved after adding port 2404
**Traces to:** BC-2.18.004 postconditions 1–2 and invariant 1 (VP-041 proptest)
- Given port 2404 added to `SUPPORTED_PORTS`
- When the partition invariant checks run (VP-041 proptest `proptest_vp041_oracle_cross_check` and `proptest_vp041_partition_invariant`)
- Then `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` (union completeness: every entry in `KNOWN_PROTOCOLS` appears in exactly one set) and `supported_protocols() ∩ unsupported_protocols() == ∅` (disjoint: no entry appears in both sets); the counting invariant `supported_protocols().len() + unsupported_protocols().len() == KNOWN_PROTOCOLS.len()` holds; adding port 2404 to `SUPPORTED_PORTS` moves the `IEC 60870-5-104` entry from `unsupported_protocols()` to `supported_protocols()` while the partition remains valid (BC-2.18.004 EC-003/EC-007)

### AC-173-006: VP-004 classifier oracle updated for DispatchTarget::Iec104
**Traces to:** BC-2.05.012 invariant 1 (VP-004 oracle update — ADR-013 Decision 9)
- Given the `#[cfg(kani)]` block in `src/dispatcher.rs` containing `classify_oracle`
- When `DispatchTarget::Iec104` is added to the dispatcher
- Then `classify_oracle` must be updated in the SAME commit to include the Iec104 arm
- This ensures the VP-004 Kani proof continues to verify after IEC-104 is added

### AC-173-007: IEC-104 findings cap enforced in Iec104Analyzer (BC-2.19.028)
**Traces to:** BC-2.19.028 postconditions 1–5 and invariant 4 (DoS bound — IEC104-FINDINGS-CAP-001)
- `const MAX_IEC104_FINDINGS: usize = 10_000` added to `src/analyzer/iec104.rs` (mirrors DNP3
  `MAX_FINDINGS` and EtherNet/IP `MAX_FINDINGS` — same value and pattern, BC-2.15.022 /
  BC-2.17.022)
- `Iec104Analyzer` gains field `dropped_findings: u64`, initialized to 0 in `new()`
- Cap enforced at the `on_data` extend step: `local_findings` is truncated to the remaining
  capacity (`MAX_IEC104_FINDINGS - self.all_findings.len()` slots) before merging into
  `self.all_findings`; the discarded count is added to `self.dropped_findings`
- No `Finding` is emitted when findings are dropped (silent cap-drop, BC-2.19.028 invariant 5)
- `self.dropped_findings` is surfaced in `summarize()` as detail key `"dropped_findings"`
- Per-flow state (`Iec104FlowState` carry buffers, dedup flags, `ns_expected`,
  `session_started`) continues to be updated regardless of the findings cap (BC-2.19.028 PC-3)
- Doc comment on `detect_iec104_threats` MUST state caller enforces cap; cite BC-2.19.028
  Invariant 6 / IEC104-FINDINGS-CAP-001
- Doc comment on `on_data` MUST note the cap bound and cite BC-2.19.028
- Unit test `test_BC_2_19_028_findings_cap`: inject frames until findings would exceed
  `MAX_IEC104_FINDINGS`; assert `all_findings.len() <= MAX_IEC104_FINDINGS` after all
  `on_data` calls; assert `dropped_findings > 0`
- VP formal proof deferred to STORY-174; unit test is sufficient P0 gate per
  BC-2.19.028 Verification Properties section

### AC-173-008: StreamDispatcher gains `iec104` field (ADR-013 Decision 9 steps 4–5)
**Traces to:** BC-2.05.012 invariant 1 (VP-004 oracle), ADR-013 Decision 9 steps 4–5
- `StreamDispatcher` gains `iec104: Option<Iec104Analyzer>` field
- `new()` extended from 5 to 6 parameters: add `iec104: Option<Iec104Analyzer>` as last param;
  call sites at `dispatcher.rs` (~line 1467) and `main.rs` (~line 343) must be updated
- `set_iec104_analyzer(&mut self, analyzer: Iec104Analyzer)` setter added (mirrors
  `set_enip_analyzer` pattern)
- Early-exit guard in `on_data` extended with `&& self.iec104.is_none()` so a
  `--iec104`-only invocation does not silently drop all data (Decision 9 step 4)
- `on_data` gains `DispatchTarget::Iec104` match arm calling
  `iec104.on_data(flow_key.clone(), data, timestamp, direction)` (mirrors ENIP arm,
  Decision 9 step 5)
- `on_flow_close` gains `DispatchTarget::Iec104` match arm calling
  `iec104.on_flow_close(flow_key.clone())` (mirrors ENIP arm, Decision 9 step 5)
- Unit test: dispatcher constructed with ONLY `iec104` set (all others `None`); send data
  on a port-2404 flow; confirm data reaches `Iec104Analyzer` (catches silent-drop if
  `self.iec104.is_none()` is omitted from the early-exit guard)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `DispatchTarget::Iec104` | SS-05 dispatcher | `src/dispatcher.rs` | N/A (enum variant) |
| Rule 8 classify | SS-05 dispatcher | `src/dispatcher.rs` | Pure (classification fn) |
| VP-004 oracle update | SS-05 Kani | `src/dispatcher.rs` | `#[cfg(kani)]` |
| T0881 catalog entry | SS-10 MITRE | `src/mitre.rs` | N/A (catalog data) |
| `--iec104` flag | SS-12 CLI | `src/cli.rs` | Effectful (CLI parsing) |
| `main.rs` wiring | SS-12 entry | `src/main.rs` | Effectful |
| Port 2404 in SUPPORTED_PORTS | SS-18 protocols | `src/protocols.rs` | N/A (constant) |
| `MAX_IEC104_FINDINGS` cap + `dropped_findings` | SS-19 analyzer | `src/analyzer/iec104.rs` | Effectful (cap at extend step) |
| `Iec104Analyzer` registration | SS-19 | `src/analyzer/mod.rs` | Effectful |

Subsystem anchors:
- SS-05 owns dispatch (add `DispatchTarget::Iec104` and Rule 8 per ARCH-INDEX.md §SS-05)
- SS-10 owns MITRE catalog (T0881 six-part atomic per ARCH-INDEX.md §SS-10)
- SS-12 owns CLI flags (`--iec104` per ARCH-INDEX.md §SS-12)
- SS-18 owns protocol catalog (SUPPORTED_PORTS per ARCH-INDEX.md §SS-18)
- SS-19 owns IEC-104 analyzer (module registration in `src/analyzer/mod.rs`)

## T0881 Six-Part Atomic Commit (ADR-013 Decision 10)

**CRITICAL: all six changes below MUST land in a SINGLE git commit. No partial delivery.**

```
Commit: feat: STORY-173 — wire IEC-104 dispatch + T0881 catalog + CLI flag

Six-part atomic for T0881 (BC-2.10.010 ADR-013 Decision 10):
1. "T0881" added to SEEDED_TECHNIQUE_IDS array (28 → 29 entries)
2. SEEDED_TECHNIQUE_ID_COUNT bumped 28 → 29
3. EMITTED_IDS array updated with "T0881"
4. technique_info("T0881") arm: ("Service Stop", MitreTactic::IcsInhibitResponseFunction)
5. vp007_catalog_drift_guard #[test] passes at count=29 (cargo test, not cargo kani)
6. verify_all_emitted_ids_resolve Kani harness passes for T0881
```

This is the same pattern as T0858/T0816/T1693.001 in STORY-133 (ENIP) and T0809/T0836 in STORY-109
(DNP3). The count drift check + EMITTED_IDS + technique_info arm are co-dependent: any partial
landing causes Kani proof failures.

## Tasks

- [ ] `src/dispatcher.rs`: Add `DispatchTarget::Iec104` variant + Rule 8 arm in
  `classify(data: &[u8], flow_key: &FlowKey)` matching via
  `[flow_key.lower_port(), flow_key.upper_port()].contains(&2404)`
- [ ] `src/dispatcher.rs`: Update VP-004 `classify_oracle` in `#[cfg(kani)]` to include Iec104 arm
  (same commit as variant + Rule 8 — Decision 9 step 3)
- [ ] `src/dispatcher.rs`: Add `iec104: Option<Iec104Analyzer>` field; extend `new()` to 6 params;
  update early-exit guard with `&& self.iec104.is_none()`; add `Iec104` arms to `on_data` and
  `on_flow_close`; update module doc-comment rule ladder (Rule 8 → 2404/Iec104, Rule 9 → No match/None);
  update `SUPPORTED_PORTS` doc-comment to add `2404 → DispatchTarget::Iec104` line
  (Decision 9 steps 4–5, AC-173-008)
- [ ] `src/mitre.rs`: Perform six-part atomic T0881 registration (all in one commit, per above)
- [ ] `src/cli.rs`: Add `--iec104` flag to `CliArgs` struct (BC-2.12.025)
- [ ] `src/main.rs`: Wire `--iec104` flag to instantiate + register `Iec104Analyzer`
- [ ] `src/protocols.rs`: Add port 2404 to `SUPPORTED_PORTS` (count 8→9; `supported_protocols()` 7→8)
- [ ] `src/analyzer/iec104.rs`: Add `const MAX_IEC104_FINDINGS: usize = 10_000`; add
  `dropped_findings: u64` field; enforce cap at `on_data` extend step; surface in `summarize()`;
  update doc comments to cite BC-2.19.028 Invariant 6 / IEC104-FINDINGS-CAP-001 (AC-173-007)
- [ ] `src/analyzer/mod.rs`: Add `pub mod iec104;` and expose `Iec104Analyzer`
- [ ] Write integration tests:
  - `test_BC_2_05_012_dispatch_port_2404` — `classify(data, flow_key)` with port=2404 → Iec104
  - `test_BC_2_10_010_t0881_catalog_entry` — T0881 in technique_info + EMITTED_IDS
  - `test_BC_2_12_025_iec104_flag` — --iec104 flag wires analyzer
  - `test_BC_2_18_003_supported_ports_includes_2404` — `SUPPORTED_PORTS.len()==9`; `supported_protocols().len()==8`
  - `test_BC_2_19_028_findings_cap` — inject >MAX_IEC104_FINDINGS findings; assert
    `all_findings.len() <= MAX_IEC104_FINDINGS` and `dropped_findings > 0`
  - `test_iec104_only_dispatcher` — dispatcher with ONLY `iec104` set; port-2404 flow reaches
    `Iec104Analyzer` (catches silent-drop guard bug)
- [ ] Verify `cargo test --all-targets` passes
- [ ] Verify `cargo test vp007_catalog_drift_guard` passes (T0881 count=29, `#[test]` — not Kani)
- [ ] Verify `cargo test verify_all_seeded_ids_resolve` passes at count=29 (BC-2.10.010 PC-4)
- [ ] Verify `cargo kani --harness verify_all_emitted_ids_resolve` passes for T0881
- [ ] OPTIONAL (IEC104-FINDING-DIRECTION-001): populate `Finding.direction` when emitting
  IEC-104 findings — direction is known at dispatch time; nice-to-have for analyst context.
  Not a blocking AC; include only if `Finding.direction` field already exists on the struct.

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.05.012 | classify(2404) on both src and dst port | Returns Iec104 (Rule 8 matches either direction) |
| EC-002 | BC-2.10.010 | SEEDED count wrong (partial commit) | Kani harness fails — prevents partial delivery |
| EC-003 | BC-2.12.025 | `--iec104` absent from CLI invocation | No Iec104Analyzer created; port 2404 flows unanalyzed |
| EC-004 | BC-2.18.003 | SUPPORTED_PORTS check at count=8 before change | After adding 2404, len must be 9 (not 8); regression at protocols_tests.rs:~111 |
| EC-005 | BC-2.18.004 | `IEC 60870-5-104` entry accidentally appears in both `supported_protocols()` and `unsupported_protocols()` (e.g., port 2404 in `SUPPORTED_PORTS` but entry also returned by `unsupported_protocols()`) | `proptest_vp041_partition_invariant` catches the disjointness violation; `proptest_vp041_oracle_cross_check` catches oracle inconsistency |
| EC-006 | BC-2.19.028 | `all_findings` reaches `MAX_IEC104_FINDINGS`; subsequent `on_data` produces more findings | `dropped_findings` incremented; `all_findings.len()` stays at cap; no Finding emitted |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| BC files (6 cross-subsystem BCs × ~700 each) | ~4,200 |
| ADR-013 (integration decisions 1, 9, 10) | ~12,000 |
| delta-analysis.md | ~5,000 |
| src/dispatcher.rs (existing) | ~5,000 |
| src/mitre.rs (existing) | ~5,000 |
| src/analyzer/iec104.rs (existing, cap changes) | ~3,000 |
| src/cli.rs + src/main.rs + src/protocols.rs | ~4,000 |
| Test files delta | ~2,500 |
| TOTAL | ~45,700 |

Agent context window ~200k tokens. This story uses ~20% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-172 (carry buffers + frame-walk loop + flow lifecycle)
- STORY-172 completed the full `Iec104Analyzer` implementation with `on_data` and `on_flow_close`
- This story wires it into the dispatcher (SS-05), MITRE catalog (SS-10), CLI (SS-12), and
  protocols catalog (SS-18) — touching 5 subsystems
- The T0881 constant was referenced (but not registered) in STORY-168's `emit_finding` calls;
  STORY-173 performs the atomic catalog registration
- The VP-004 Kani proof from an earlier story established the `classify_oracle` pattern;
  this story extends it to include the new `Iec104` arm (ADR-013 Decision 9)

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 1**: Port 2404 → Rule 8 dispatch. No content-signature in `classify()`.
  Real signature: `fn classify(data: &[u8], flow_key: &FlowKey)`. Port 2404 matched via
  `[flow_key.lower_port(), flow_key.upper_port()].contains(&2404)`. After adding Rule 8
  (IEC-104), the old "No match → None" becomes Rule 9.
- **ADR-013 Decision 9**: `DispatchTarget::Iec104` addition is a six-step atomic obligation:
  steps 1–3 (enum variant, Rule 8 arm, `classify_oracle` update) and step 6 (re-run
  `verify_content_first_precedence_exhaustive`) are covered by AC-173-001 and AC-173-006;
  steps 4–5 (`iec104` field on `StreamDispatcher`, early-exit guard extension, `on_data`/
  `on_flow_close` Iec104 arms) are covered by AC-173-008.
- **ADR-013 Decision 10**: T0881 six-part atomic commit — all six mitre.rs changes together.
  The `SEEDED_TECHNIQUE_IDS` array + `SEEDED_TECHNIQUE_ID_COUNT` bump + `EMITTED_IDS` +
  `technique_info` arm are co-dependent; partial commits cause drift-guard failures.
  `vp007_catalog_drift_guard` is a `#[test]`, not a Kani harness — run with `cargo test`.
- **VP-004**: After adding `Iec104` to `DispatchTarget`, the VP-004 Kani proof must be re-run
  (STORY-174 formal hardening). The oracle update in this story enables that re-run.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ | enum variant, match arm, const |
| kani | latest | VP-004 oracle update + VP-007 T0881 verification |
| proptest | latest | VP-041 SUPPORTED_PORTS partition check |

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/dispatcher.rs` | MODIFY | Add `DispatchTarget::Iec104`; Rule 8 port-2404 arm; VP-004 oracle update; `iec104` field; `new()` 5→6 params; early-exit guard; `on_data`/`on_flow_close` Iec104 arms; module doc Rule 8/9 renumber |
| `src/mitre.rs` | MODIFY | Six-part T0881 atomic: `SEEDED_TECHNIQUE_IDS` add T0881, `SEEDED_TECHNIQUE_ID_COUNT` 28→29, `EMITTED_IDS`, `technique_info` arm (MitreTactic::IcsInhibitResponseFunction), drift-guard test |
| `src/cli.rs` | MODIFY | Add `--iec104` bool flag to `CliArgs` |
| `src/main.rs` | MODIFY | Wire `--iec104` → instantiate `Iec104Analyzer` + register |
| `src/protocols.rs` | MODIFY | Add 2404 to `SUPPORTED_PORTS`; count 8→9; `supported_protocols()` 7→8 |
| `src/analyzer/iec104.rs` | MODIFY | Add `MAX_IEC104_FINDINGS` const; `dropped_findings` field; cap at extend step; `summarize()` detail key; doc comments citing BC-2.19.028 |
| `src/analyzer/mod.rs` | MODIFY | `pub mod iec104;` + `pub use iec104::Iec104Analyzer;` |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- PARTIAL T0881 registration is forbidden: if any of the six parts is missing at commit time,
  the Kani drift-guard will fail — this is the enforcement mechanism, not a soft guideline
