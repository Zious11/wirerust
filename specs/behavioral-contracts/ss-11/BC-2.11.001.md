---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/json.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
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

# BC-2.11.001: JsonReporter Renders JSON Object with summary/findings/analyzers Keys

## Description

`JsonReporter::render` produces a single JSON object with exactly three top-level keys:
`summary`, `findings`, and `analyzers`. The output is valid, pretty-printed JSON produced
by `serde_json::to_string_pretty`. The `unwrap()` call is infallible by construction because
`serde_json::Value` serialization cannot fail.

## Preconditions

1. `JsonReporter::render` is called with a `Summary`, a `&[Finding]`, and a
   `&[AnalysisSummary]`.
2. The `Summary` struct implements `serde::Serialize`.
3. All `Finding` fields implement `serde::Serialize`.

## Postconditions

1. The returned `String` is valid JSON (parseable by any RFC 8259 compliant parser).
2. The top-level object contains exactly the keys `"summary"`, `"findings"`, and
   `"analyzers"`.
3. `"findings"` is a JSON array; one element per `Finding` in the input slice.
4. `"analyzers"` is a JSON array; one element per `AnalysisSummary` in the input slice.
5. `"summary"` contains `total_packets`, `total_bytes`, `skipped_packets`,
   `unique_hosts`, `protocols`, and `services` sub-keys.
6. Output is pretty-printed (indented with spaces, one key per line).

## Invariants

1. The `unwrap()` at `json.rs:59` is infallible; `serde_json::to_string_pretty` cannot fail
   on a `serde_json::Value`.
2. No manual escaping is performed; ADR 0003 delegates escaping to serde_json's RFC 8259
   path (C0+DEL escaped as `\uNNNN`; C1 passed through as raw UTF-8).
3. Protocol keys are converted via `{k:?}` (Debug) format to produce string keys in the
   JSON map (e.g., `"Tcp"`, `"Udp"`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty findings slice | `"findings": []` |
| EC-002 | Empty analyzers slice | `"analyzers": []` |
| EC-003 | skipped_packets = 0 | `"skipped_packets": 0` is present |
| EC-004 | No hosts seen | `"unique_hosts": []` |
| EC-005 | Multiple protocol types | Protocol map has one key per distinct Protocol variant observed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary with 1 packet, 0 findings, 0 analyzers | JSON with summary.total_packets=1, findings=[], analyzers=[] | happy-path |
| Summary with skipped_packets=3 | "skipped_packets": 3 appears in summary | happy-path |
| findings=[one Finding] | "findings" array has length 1 | happy-path |
| Empty everything | All three arrays/objects present with zero entries | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Output is valid JSON | unit: test_json_reporter_produces_valid_json |
| VP-TBD | Top-level keys are exactly summary, findings, analyzers | unit: assert key presence |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC defines the machine-readable JSON output shape that is the primary API surface of the reporting capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- JsonReporter does not escape, delegates to serde_json per ADR 0003) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | S-TBD |
| Origin BC | BC-RPT-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.002 -- composes with (skipped_packets detail in summary)
- BC-2.11.003 -- composes with (RFC 8259 C0 escaping behavior)
- BC-2.11.004 -- composes with (Unicode preservation behavior)
- BC-2.09.006 -- depends on (Finding Option fields serialization)

## Architecture Anchors

- `src/reporter/json.rs:23-60` -- JsonReporter::render implementation
- `src/reporter/json.rs:59` -- infallible unwrap on serde_json::to_string_pretty

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: serde::Serialize derived on Summary and Finding
- **assertion**: test_json_reporter_produces_valid_json verifies parseable output
- **structural guarantee**: `serde_json::to_string_pretty` on a `serde_json::Value` cannot fail
  (Value implements Serialize and contains only JSON-representable types); the `unwrap()` at
  `json.rs:59` is therefore infallible by construction, not by documentation

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (returns owned String) |
| **Global state access** | none |
| **Deterministic** | yes (BTreeMap used for protocol/service maps; alphabetical key order) |
| **Thread safety** | Send + Sync (JsonReporter is a unit struct) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- pure string transformation, suitable for formal verification.
