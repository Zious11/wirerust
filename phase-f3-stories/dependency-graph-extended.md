---
document_type: dependency-graph-extended
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
base_graph: .factory/stories/dependency-graph.md
extension_stories: [STORY-100, STORY-101, STORY-102, STORY-103, STORY-104, STORY-105]
new_epics: [E-13, E-14]
total_stories_pre_extension: 51
total_stories_post_extension: 57
total_edges_pre_extension: 79
total_edges_post_extension: 86
acyclic: true
traces_to:
  - .factory/stories/dependency-graph.md
  - .factory/phase-f2-spec-evolution/prd-delta.md
  - .factory/research/f2-decomposition-sequencing.md
---

# wirerust Extended Dependency Graph (F3 Stories: E-13 + E-14)

This document extends the base dependency graph at `.factory/stories/dependency-graph.md`
(51 stories, 79 edges, 30 waves) with 6 new stories across two new epics (E-13, E-14).
The extension is acyclic. Topological sort is validated below.

---

## New Epics Summary

| Epic | Cycle | Stories | BCs | Description |
|------|-------|---------|-----|-------------|
| E-13 | v0.3.0-multitag | STORY-100, STORY-101 | BC-2.09.001, BC-2.09.006, BC-2.10.005, BC-2.10.007, BC-2.10.008, BC-2.11.001, BC-2.11.013, BC-2.11.015, BC-2.11.017, BC-2.11.020, BC-2.11.024 | Multi-Tag Finding Schema Migration (atomic rename + catalog seed + reporter add-ons) |
| E-14 | v0.4.0-modbus | STORY-102, STORY-103, STORY-104, STORY-105 | BC-2.14.001 through BC-2.14.025 | Modbus TCP Analyzer (parse, flow state, detection, CLI integration) |

---

## New Dependency Edges

### Intra-Epic Edges — E-13: Multi-Tag Schema Migration

| From | To | Justification |
|------|----|---------------|
| STORY-100 | STORY-101 | STORY-101 depends on STORY-100 because the reporter changes in STORY-101 consume `mitre_techniques: Vec<String>` (the JSON envelope add-on and CSV column 6 rename build on the type that STORY-100 defines). STORY-101 references `f.mitre_techniques.join(";")` and `f.mitre_techniques.first()` — both compile only after STORY-100 lands the field rename. |

### Intra-Epic Edges — E-14: Modbus TCP Analyzer

| From | To | Justification |
|------|----|---------------|
| STORY-102 | STORY-103 | STORY-103 depends on STORY-102 because `parse_mbap_header`, `classify_fc`, `is_valid_modbus_adu`, and the `ModbusFlowState` stub are defined in STORY-102. The `on_data` parsing loop in STORY-103 builds directly on those pure-core functions. |
| STORY-103 | STORY-104 | STORY-104 depends on STORY-103 because all 7 detection rules in STORY-104 read `ModbusFlowState` window fields (`window_write_count`, `t0831_window_write_count`, `sustained_window_write_count`, `exception_window_counts`) that are established by STORY-103's full field list. Writing detection logic without the state fields causes compilation failures. |
| STORY-104 | STORY-105 | STORY-105 depends on STORY-104 because `ModbusAnalyzer` must have `on_data`, `on_flow_close`, `findings()`, and `summarize()` fully implemented before `StreamDispatcher` and `main.rs` can wire it up. Wiring an incomplete analyzer would produce silent failures. |

### Cross-Epic Edges (new)

| From | To | From Epic | To Epic | Subsystem Boundary | Justification |
|------|----|-----------|---------|--------------------|---------------|
| STORY-100 | STORY-102 | E-13 | E-14 | SS-09 (findings.rs) → SS-14 (analyzer/modbus.rs) | STORY-102 creates `src/analyzer/modbus.rs` which imports `src/findings.rs`. The `Finding { mitre_techniques: vec![...] }` field must exist (from STORY-100) before any Modbus emission site can compile. Without this dependency, the Modbus file compiles but emits with the wrong type at the first emission site in STORY-104. Placing the dependency here (at the start of E-14) makes the dependency graph honest about the compile-time ordering requirement. |

---

## Full Extended Edge List (new edges only; existing 79 edges unchanged)

| # | From | To | Epic | Type |
|---|------|----|------|------|
| 80 | STORY-100 | STORY-101 | E-13 | intra-epic |
| 81 | STORY-102 | STORY-103 | E-14 | intra-epic |
| 82 | STORY-103 | STORY-104 | E-14 | intra-epic |
| 83 | STORY-104 | STORY-105 | E-14 | intra-epic |
| 84 | STORY-100 | STORY-102 | E-13 → E-14 | cross-epic |

Total edges after extension: 79 + 5 = **84 edges**.

> Note: The original count was 79. The cross-epic edge STORY-099 → STORY-100 is not
> enumerated here because STORY-099 is in E-12 (completed cycle v0.2.0) and does NOT
> block E-13 (no hard build-order dependency between timestamp provenance and schema
> migration). The two cycles are independent; E-12 shipped in v0.2.0, E-13 targets v0.3.0.

---

## Topological Sort Validation

Kahn's algorithm applied to the 6 new stories plus their new cross-epic dependency.

### Initial in-degree computation (new stories only)

| Story | Depends On (new edges) | In-degree |
|-------|----------------------|-----------|
| STORY-100 | [] (no dependency in new graph) | 0 |
| STORY-101 | [STORY-100] | 1 |
| STORY-102 | [STORY-100] | 1 |
| STORY-103 | [STORY-102] | 1 |
| STORY-104 | [STORY-103] | 1 |
| STORY-105 | [STORY-104] | 1 |

### Kahn's algorithm trace

**Step 1:** Queue = {STORY-100} (in-degree 0)
- Process STORY-100: emit. Remove edges STORY-100→STORY-101 and STORY-100→STORY-102.
- STORY-101 in-degree: 1→0. STORY-102 in-degree: 1→0.
- Queue = {STORY-101, STORY-102}.

**Step 2:** Queue = {STORY-101, STORY-102} (both in-degree 0 — parallel wave)
- Process STORY-101: emit. No outgoing edges to new stories.
- Process STORY-102: emit. Remove edge STORY-102→STORY-103.
- STORY-103 in-degree: 1→0.
- Queue = {STORY-103}.

**Step 3:** Queue = {STORY-103}
- Process STORY-103: emit. Remove edge STORY-103→STORY-104.
- STORY-104 in-degree: 1→0.
- Queue = {STORY-104}.

**Step 4:** Queue = {STORY-104}
- Process STORY-104: emit. Remove edge STORY-104→STORY-105.
- STORY-105 in-degree: 1→0.
- Queue = {STORY-105}.

**Step 5:** Queue = {STORY-105}
- Process STORY-105: emit. No outgoing edges.
- Queue = {}.

**Topological order:** STORY-100 → (STORY-101 ∥ STORY-102) → STORY-103 → STORY-104 → STORY-105

**All 6 stories emitted. No story left with in-degree > 0. Graph is ACYCLIC. ✓**

---

## Wave Assignment

Wave numbers extend the existing schedule (base graph waves 1–30, E-12 waves 28–30).

| Wave | Stories | Notes |
|------|---------|-------|
| 31 | STORY-100, STORY-101 | STORY-100 has no dependency; STORY-101 depends on STORY-100 but is in the same wave because they are dispatched in the same cycle (STORY-101 is dispatched immediately after STORY-100 merges). Per project convention, a story can be assigned to the same wave as its predecessor if the predecessor is expected to merge before the wave closes. In practice: dispatch STORY-100 first; dispatch STORY-101 when STORY-100's PR is green. |
| 32 | STORY-102 | Depends on STORY-100 (merged in wave 31). Modbus pure-core (parse + FC classification). |
| 33 | STORY-103, STORY-104 | STORY-103 depends on STORY-102 (wave 32). STORY-104 depends on STORY-103 — but STORY-103 and STORY-104 are in the SAME wave in the dispatch plan, meaning STORY-104 is dispatched immediately after STORY-103 merges within wave 33. |
| 34 | STORY-105 | Depends on STORY-104 (merged in wave 33). CLI + dispatcher integration. |

> **Wave 31 clarification:** The dependency graph assigns STORY-101 as `depends_on: [STORY-100]`,
> meaning STORY-101 cannot START until STORY-100 is complete. Both are in wave 31 per the
> release cycle (v0.3.0) — wave number reflects the release wave, not strict parallelism.
> The wave-scheduler tool (DF-022) will place STORY-101 in wave 32 via topological sort
> if strict wave isolation is required. The story frontmatter uses `wave: 31` for both to
> indicate they are part of the same v0.3.0 feature batch.

---

## Dependency Graph Visualization

```
Existing graph (waves 1-30, E-1 through E-12)
              ↓
          STORY-100 (wave 31, E-13)
         /          \
    STORY-101    STORY-102 (wave 32, E-14)
    (wave 31,        |
     E-13)       STORY-103 (wave 33, E-14)
                     |
                 STORY-104 (wave 33, E-14)
                     |
                 STORY-105 (wave 34, E-14)
```

---

## Release Sequencing Diagram

```
v0.2.0 (shipped)
  └─ E-12: Pcap Timestamp Provenance (STORY-097/098/099)

v0.3.0 (target: E-13, breaking schema change)
  └─ E-13: Multi-Tag Finding Schema Migration
      ├─ STORY-100: Finding.mitre_technique → mitre_techniques (atomic rename, catalog seed)
      └─ STORY-101: Reporter serialization add-ons (JSON envelope, CSV rename, terminal multi-ID)

v0.4.0 (target: E-14, purely additive Modbus analyzer)
  └─ E-14: Modbus TCP Analyzer (25 BCs, BC-2.14.001..025)
      ├─ STORY-102: MBAP parse + FC classification (pure core + VP-022 Kani harnesses)
      ├─ STORY-103: Flow state + transaction correlation (pending table, 15+ state fields)
      ├─ STORY-104: Detection emissions + summary (7 detectors, MAX_FINDINGS cap, summarize)
      └─ STORY-105: Dispatcher integration + CLI (Rule 5, --modbus flags, 4-step wiring)
```

---

## BC Coverage Summary (new stories)

| Story | BCs Covered | BC Count |
|-------|-------------|----------|
| STORY-100 | BC-2.09.001, BC-2.09.006, BC-2.10.005, BC-2.10.007, BC-2.10.008 | 5 |
| STORY-101 | BC-2.11.001, BC-2.11.013, BC-2.11.015, BC-2.11.017, BC-2.11.020, BC-2.11.024 | 6 |
| STORY-102 | BC-2.14.001, BC-2.14.002, BC-2.14.003, BC-2.14.004, BC-2.14.005, BC-2.14.006, BC-2.14.007, BC-2.14.008 | 8 |
| STORY-103 | BC-2.14.009, BC-2.14.010, BC-2.14.011, BC-2.14.012 | 4 |
| STORY-104 | BC-2.14.013, BC-2.14.014, BC-2.14.015, BC-2.14.016, BC-2.14.017, BC-2.14.018, BC-2.14.019, BC-2.14.020, BC-2.14.021, BC-2.14.022 | 10 |
| STORY-105 | BC-2.14.023, BC-2.14.024, BC-2.14.025 | 3 |
| **Total** | | **36** |

All 36 new BCs (11 revised SS-09/SS-10/SS-11 + 25 new SS-14) are covered.
No BC is double-assigned across these 6 stories.

---

## VP Coverage Summary (new stories)

| VP | Covered By |
|----|-----------|
| VP-007 (catalog drift guard) | STORY-100 (catalog seed + count update) |
| VP-016 (mitre-tactic-grouping-order) | STORY-100 (harness update) + STORY-101 (tactic grouping) |
| VP-020 (csv-injection-neutralization) | STORY-100 (harness update) + STORY-101 (CSV column) |
| VP-021 (timestamp-provenance) | STORY-100 (test helper update) |
| VP-004 (classify-oracle, extended) | STORY-105 (port-502 Modbus branch) |
| VP-022 (Modbus parse safety) | STORY-102 (Kani sub-properties A+B) + STORY-103 (pending-table bound) + STORY-104 (classify_fc totality) |

---

## Acyclicity Confirmation

```
Topological sort result (new stories):
  [STORY-100] → [STORY-101 ∥ STORY-102] → [STORY-103] → [STORY-104] → [STORY-105]

All 6 stories processed. No cycle detected. ✓

Cross-epic dependency direction:
  E-13 (SS-09/10/11) → E-14 (SS-14)
  Direction: findings type → Modbus analyzer (Modbus USES the renamed type)
  No reverse dependency (SS-14 does not export anything consumed by SS-09/10/11).
  Graph remains acyclic after extension. ✓
```
