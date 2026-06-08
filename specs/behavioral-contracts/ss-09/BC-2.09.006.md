---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/findings.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Feature-100 (pcap timestamps) — O-01 resolved: timestamp now appears in JSON for 21 of 22 emission sites (Some(DateTime<Utc>)); EC-005 updated from always-None to positive/negative cases; Invariant 2 updated. — 2026-06-08"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.006: Finding JSON Serialization: All None Option Fields Omitted via skip_serializing_if

## Description

`Finding` is `#[derive(Serialize)]`. All four Option fields use
`#[serde(skip_serializing_if = "Option::is_none")]`. When the value is None, the key is
completely absent from the JSON output. No Option field ever serializes as `null`.

This was fixed in P1.02 / #73: previously `mitre_technique` and `source_ip` always appeared
as `null` when None (only `timestamp` and `direction` had `skip_serializing_if`). The fix
made all four fields symmetric. Any downstream consumer that relied on `null` values for
absence detection must use key absence instead.

After feature-100 (issue #100): 21 of 22 emission sites set `timestamp: Some(DateTime<Utc>)`.
The `"timestamp"` key now appears in JSON for flow-data-path findings, serialized as an
ISO-8601 UTC string by chrono's serde integration. The segment-limit summary finding (the
22nd site) retains `timestamp: None` and therefore still produces no `"timestamp"` key.
No change to `findings.rs` is required — the existing `skip_serializing_if` serde attribute
handles `Some(DateTime<Utc>)` serialization automatically.

## Preconditions

1. A `Finding` struct is being serialized via `JsonReporter` (serde_json::to_string_pretty).
2. At least one Option field (mitre_technique, source_ip, timestamp, direction) is None.

## Postconditions

1. Fields with `Some(value)` appear normally in JSON with their value.
2. Fields with `None` are completely absent from the JSON object (no key, no `null`).
3. The JSON schema is symmetric: all four Option fields follow the same rule.
4. The four affected fields are: `mitre_technique`, `source_ip`, `timestamp`, `direction`.

## Invariants

1. Absence = None. Presence = Some(value). Never: presence = null.
2. After feature-100: 21 of 22 emission sites set `timestamp: Some(DateTime<Utc>)`, so
   `"timestamp"` appears in JSON for flow-data-path findings as an ISO-8601 UTC string
   (e.g., `"timestamp": "2001-09-08T21:46:40Z"`). The segment-limit summary finding (the
   22nd site) retains `timestamp: None` and produces no `"timestamp"` key in JSON.
3. `direction: Some(...)` is set by all HTTP and TLS analyzer findings; reassembly-engine
   findings leave it as None and therefore omit it from JSON.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with mitre_technique=None | JSON has no "mitre_technique" key at all |
| EC-002 | Finding with mitre_technique=Some("T1036") | JSON has `"mitre_technique": "T1036"` |
| EC-003 | Finding with direction=Some(ClientToServer) | JSON has `"direction": "ClientToServer"` |
| EC-004 | Reassembly-engine finding (direction=None) | JSON has no "direction" key |
| EC-005a | Flow-data-path finding (timestamp = Some after feature-100) | JSON has `"timestamp": "<ISO-8601 UTC string>"` (e.g., `"2001-09-08T21:46:40Z"`) |
| EC-005b | Segment-limit summary finding (timestamp = None; finalize aggregate) | JSON has no "timestamp" key (skip_serializing_if = "Option::is_none") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding { mitre_technique: None, direction: None, ... } | JSON: no "mitre_technique" key, no "direction" key | happy-path |
| Finding { mitre_technique: Some("T1036"), direction: Some(ClientToServer) } | JSON: has both keys with values | happy-path |
| Full pipeline HTTP finding (ts_sec=1_000_000) | JSON has `"timestamp": "2001-09-08T21:46:40Z"` | happy-path |
| Full pipeline segment-limit summary finding | No "timestamp" key in JSON for that finding | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | None fields produce absent keys (not null values) | unit: parse JSON, assert key not present |
| — | All four Option fields use skip_serializing_if | code: grep for skip_serializing_if in findings.rs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md -- JSON serialization symmetry is a key contract for SIEM consumers of Finding output |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-09 (findings.rs, C-14; reporter/json.rs, C-19) |
| Stories | STORY-070 |
| Origin BC | BC-FND-006 (pass-3 ingestion corpus, MEDIUM confidence; R4 material refinement) |

## Related BCs

- BC-2.11.001 -- composes with (JsonReporter produces the JSON that contains these findings)
- BC-2.09.001 -- composes with (Finding schema definition)
- BC-2.09.007 -- composes with (timestamp provenance; this BC defines the JSON serialization behavior for the Some(DateTime<Utc>) value BC-2.09.007 defines)

## Architecture Anchors

- `src/findings.rs` -- #[serde(skip_serializing_if = "Option::is_none")] on all 4 fields
- `tests/reporter_tests.rs` -- test_json_reporter_produces_valid_json

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **type constraint**: serde attribute enforces absence-not-null
- **assertion**: test_json_reporter_produces_valid_json (indirect)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
