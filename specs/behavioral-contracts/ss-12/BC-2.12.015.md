---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.015: dispatcher.unclassified_flows() Injected into Reassembly Summary

## Description

After the packet loop and reassembler finalization, `dispatcher.unclassified_flows()` is
injected as the `"unclassified_flows"` key in the reassembly `AnalysisSummary.detail` map.
This gives forensic operators visibility into how many TCP flows were seen but not classified
as HTTP or TLS, in the same summary block as reassembly statistics.

## Preconditions

1. `run_analyze` has completed the packet loop and reassembler finalization.
2. A reassembler was constructed (i.e., `reassembler.is_some()`).
3. `dispatcher.unclassified_flows()` returns a count.

## Postconditions

1. `reasm_summary.detail.insert("unclassified_flows", serde_json::json!(count))`.
2. The reassembly AnalysisSummary's detail map contains the key `"unclassified_flows"`.
3. The value is a JSON number (u64) equal to the count from `dispatcher.unclassified_flows()`.

## Invariants

1. This injection happens only when a reassembler was constructed (inside `if let Some(ref reasm) = reassembler`).
2. When no reassembler was constructed, the dispatcher stats are not collected.
3. `unclassified_flows` is the only dispatcher stat injected into the reassembly summary.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No reassembler constructed | unclassified_flows not injected |
| EC-002 | reassembler present, 0 unclassified flows | "unclassified_flows": 0 in detail |
| EC-003 | reassembler present, N unclassified flows | "unclassified_flows": N |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| reassembler present, dispatcher.unclassified_flows()=3 | detail["unclassified_flows"]=3 | happy-path |
| no reassembler | detail map has no unclassified_flows key | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | unclassified_flows injected into reassembly summary | unit: code-level (MEDIUM -- not directly tested) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the injection of dispatcher.unclassified_flows() into the reassembly summary detail map (main.rs:204-208) is wiring performed in run_analyze after finalization; this is an entry-point assembly step that bridges the dispatcher stat into the summary object, not a reporter rendering concern |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-015 (pass-3 ingestion corpus, MEDIUM confidence -- code is explicit; no direct test) |

## Related BCs

- BC-2.05.007 -- depends on (dispatcher.unclassified_flows counter is defined in SS-05)

## Architecture Anchors

- `src/main.rs:204-208` -- unclassified_flows insertion into reassembly summary detail

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:204-208` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code at lines 204-208 is explicit

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | N/A |
| **Overall classification** | mixed |

#### Refactoring Notes

No refactoring needed.
