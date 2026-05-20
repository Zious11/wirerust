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

# BC-2.11.002: JsonReporter Includes skipped_packets in Summary

## Description

`JsonReporter::render` unconditionally includes `skipped_packets` in the JSON `summary`
object, even when the value is zero. This field tracks the count of packets that failed
`decode_packet` and were counted but not analyzed. Its presence in every JSON output (not
just when non-zero) allows downstream scripts to distinguish "no errors" from "field absent."

## Preconditions

1. `JsonReporter::render` is called with any `Summary` instance.
2. `Summary.skipped_packets` is set (defaults to 0 via `Summary::new`).

## Postconditions

1. The JSON `summary` object contains a `"skipped_packets"` key.
2. When `skipped_packets = 0`, the value is `0` (not absent, not null).
3. When `skipped_packets > 0`, the value equals the count of decode-failed packets.
4. `skipped_packets` serializes as a JSON integer (u64).

## Invariants

1. `skipped_packets` is ALWAYS present in JSON output, regardless of its value.
   This differs from TerminalReporter which only shows the skipped-packets line
   when N > 0 (BC-2.11.006).
2. The value is set by `summary.skipped_packets = total_decode_errors` in main.rs
   after the packet loop completes (main.rs:183).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | skipped_packets = 0 | "skipped_packets": 0 present |
| EC-002 | skipped_packets = 1 | "skipped_packets": 1 |
| EC-003 | Large value (u64::MAX) | Serialized as JSON integer |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary with default skipped_packets=0 | "skipped_packets": 0 in output | happy-path |
| Summary with skipped_packets=3 | "skipped_packets": 3 in JSON summary | happy-path |
| Empty capture (zero packets, zero skipped) | "skipped_packets": 0 present | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | skipped_packets present when zero | unit: test_json_reporter_skipped_packets_zero_by_default |
| VP-TBD | skipped_packets present when non-zero | unit: test_json_reporter_includes_skipped_packets |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- skipped_packets is a required summary field for forensic completeness; its unconditional inclusion in JSON output is part of the machine-readable reporting contract |
| L2 Domain Invariants | None directly; related to INV-4 (raw data flows through) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | S-TBD |
| Origin BC | BC-RPT-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.001 -- composes with (overall JSON output shape)
- BC-2.11.006 -- contrasts with (TerminalReporter suppresses line when N=0; JsonReporter always includes)
- BC-2.12.014 -- depends on (skipped_packets is populated from decode errors in main.rs)

## Architecture Anchors

- `src/reporter/json.rs:47-56` -- serde_json::json! macro block including skipped_packets
- `src/main.rs:183` -- summary.skipped_packets = total_decode_errors assignment

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: skipped_packets field on Summary is u64 with #[derive(Serialize)]
- **assertion**: test_json_reporter_skipped_packets_zero_by_default, test_json_reporter_includes_skipped_packets

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
