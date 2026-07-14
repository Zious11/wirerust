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
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "c08bb4c"
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

## Acceptance Criteria

### AC-173-001: DispatchTarget::Iec104 variant added and Rule 8 routes port 2404
**Traces to:** BC-2.05.012 postconditions 1–3
- Given the `StreamDispatcher` in `src/dispatcher.rs`
- When a TCP flow on port 2404 is classified
- Then `classify(port)` returns `DispatchTarget::Iec104` (Rule 8)
- `DispatchTarget::Iec104` variant is added to the `DispatchTarget` enum
- The VP-004 `classify_oracle` in the `#[cfg(kani)]` block is updated to include the Iec104 arm
  (ADR-013 Decision 9 — VP-004 oracle must be updated atomically with DispatchTarget::Iec104)

### AC-173-002: T0881 registered in mitre.rs via six-part atomic commit
**Traces to:** BC-2.10.010 postconditions 1–6 and invariant 1 (ADR-013 Decision 10)
- The following six changes MUST be delivered in a SINGLE git commit (six-part atomic):
  1. `SEEDED_TECHNIQUE_COUNT` constant bumped from 28 to 29
  2. `T0881` technique ID constant added to `mitre.rs` constants block
  3. `EMITTED_IDS` array gains `T0881` entry
  4. `technique_info(id)` match arm added for T0881: `("Service Stop", "impact")`
  5. `vp007_catalog_drift_guard` Kani harness updated to pass at count=29
  6. `verify_all_emitted_ids_resolve` Kani harness passes for T0881 (EMITTED postcondition)
- Partial commits that pass tests but miss the count bump, EMITTED_IDS, or Kani harness update
  are NOT acceptable — all six parts must land together

### AC-173-003: `--iec104` CLI flag enables IEC-104 analysis
**Traces to:** BC-2.12.025 postconditions 1–3
- Given `cargo run -- --iec104 <pcap>`
- When wirerust processes a pcap containing IEC-104 traffic on port 2404
- Then `Iec104Analyzer` is instantiated and registered with the dispatcher
- Without `--iec104`, no IEC-104 analysis is performed (flag-gated per opt-in model)

### AC-173-004: SUPPORTED_PORTS includes port 2404 (count 7→8)
**Traces to:** BC-2.18.003 postconditions 1–2
- Given the `SUPPORTED_PORTS` constant in `src/protocols.rs` (or equivalent)
- When `SUPPORTED_PORTS.contains(&2404)` is checked
- Then it returns `true`; the count increases from 7 to 8

### AC-173-005: Protocol catalog partition invariant preserved after adding port 2404
**Traces to:** BC-2.18.004 postconditions 1–2 and invariant 1 (VP-041 proptest)
- Given port 2404 added to `SUPPORTED_PORTS`
- When the partition invariant check runs (VP-041 proptest `proptest_vp041_*`)
- Then `SUPPORTED_PORTS`, `UNMONITORED_PORTS`, and the unclassified set form a valid partition
  of the overall port space (no overlaps, no missing coverage for documented ports)

### AC-173-006: VP-004 classifier oracle updated for DispatchTarget::Iec104
**Traces to:** BC-2.05.012 invariant 1 (VP-004 oracle update — ADR-013 Decision 9)
- Given the `#[cfg(kani)]` block in `src/dispatcher.rs` containing `classify_oracle`
- When `DispatchTarget::Iec104` is added to the dispatcher
- Then `classify_oracle` must be updated in the SAME commit to include the Iec104 arm
- This ensures the VP-004 Kani proof continues to verify after IEC-104 is added

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
1. SEEDED_TECHNIQUE_COUNT: 28 → 29
2. T0881 constant added to mitre.rs
3. EMITTED_IDS array updated with T0881
4. technique_info(T0881) arm: ("Service Stop", "impact")
5. vp007_catalog_drift_guard harness: count 28 → 29
6. verify_all_emitted_ids_resolve: passes for T0881 (EMITTED postcondition)
```

This is the same pattern as T0858/T0816/T1693.001 in STORY-133 (ENIP) and T0809/T0836 in STORY-109
(DNP3). The count drift check + EMITTED_IDS + technique_info arm are co-dependent: any partial
landing causes Kani proof failures.

## Tasks

- [ ] `src/dispatcher.rs`: Add `DispatchTarget::Iec104` variant + Rule 8 case in `classify(port: u16)`
- [ ] `src/dispatcher.rs`: Update VP-004 `classify_oracle` in `#[cfg(kani)]` to include Iec104 arm
- [ ] `src/mitre.rs`: Perform six-part atomic T0881 registration (all in one commit, per above)
- [ ] `src/cli.rs`: Add `--iec104` flag to `CliArgs` struct (BC-2.12.025)
- [ ] `src/main.rs`: Wire `--iec104` flag to instantiate + register `Iec104Analyzer`
- [ ] `src/protocols.rs` (or equivalent): Add port 2404 to `SUPPORTED_PORTS` (count 7→8)
- [ ] `src/analyzer/mod.rs`: Add `pub mod iec104;` and expose `Iec104Analyzer`
- [ ] Write integration tests:
  - `test_BC_2_05_012_dispatch_port_2404` — classify(2404) == Iec104
  - `test_BC_2_10_010_t0881_catalog_entry` — T0881 in technique_info + EMITTED_IDS
  - `test_BC_2_12_025_iec104_flag` — --iec104 flag wires analyzer
  - `test_BC_2_18_003_supported_ports_includes_2404` — SUPPORTED_PORTS count=8
- [ ] Verify `cargo test --all-targets` passes
- [ ] Verify `cargo kani --harness vp007_catalog_drift_guard` passes (T0881 count=29)
- [ ] Verify `cargo kani --harness verify_all_emitted_ids_resolve` passes for T0881

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.05.012 | classify(2404) on both src and dst port | Returns Iec104 (Rule 8 matches either direction) |
| EC-002 | BC-2.10.010 | SEEDED count wrong (partial commit) | Kani harness fails — prevents partial delivery |
| EC-003 | BC-2.12.025 | `--iec104` absent from CLI invocation | No Iec104Analyzer created; port 2404 flows unanalyzed |
| EC-004 | BC-2.18.003 | SUPPORTED_PORTS check at count=7 after change | Fails — must be 8 after adding 2404 |
| EC-005 | BC-2.18.004 | Port 2404 accidentally also in UNMONITORED_PORTS | VP-041 partition proptest catches overlap |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| BC files (5 cross-subsystem BCs × ~700 each) | ~3,500 |
| ADR-013 (integration decisions 1, 9, 10) | ~12,000 |
| delta-analysis.md | ~5,000 |
| src/dispatcher.rs (existing) | ~5,000 |
| src/mitre.rs (existing) | ~5,000 |
| src/cli.rs + src/main.rs + src/protocols.rs | ~4,000 |
| Test files delta | ~2,000 |
| TOTAL | ~40,500 |

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
- **ADR-013 Decision 9**: `DispatchTarget::Iec104` addition requires ATOMIC update of
  `classify_oracle` in `#[cfg(kani)]` block — same commit, same PR.
- **ADR-013 Decision 10**: T0881 six-part atomic commit — all six mitre.rs changes together.
  The SEEDED count bump + EMITTED_IDS + technique_info arm are co-dependent; partial commits
  cause Kani drift-guard failures.
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
| `src/dispatcher.rs` | MODIFY | Add `DispatchTarget::Iec104`; Rule 8 port-2404 arm; VP-004 oracle update |
| `src/mitre.rs` | MODIFY | Six-part T0881 atomic: SEEDED count, T0881 const, EMITTED_IDS, technique_info, Kani harness |
| `src/cli.rs` | MODIFY | Add `--iec104` bool flag to `CliArgs` |
| `src/main.rs` | MODIFY | Wire `--iec104` → instantiate `Iec104Analyzer` + register |
| `src/protocols.rs` | MODIFY | Add 2404 to `SUPPORTED_PORTS`; count 7→8 |
| `src/analyzer/mod.rs` | MODIFY | `pub mod iec104;` + `pub use iec104::Iec104Analyzer;` |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- PARTIAL T0881 registration is forbidden: if any of the six parts is missing at commit time,
  the Kani drift-guard will fail — this is the enforcement mechanism, not a soft guideline
