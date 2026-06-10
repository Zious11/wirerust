---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.4: F5 ADV-F5-HIGH-001 — corrected canonical ts_sec=1_000_000 vector from 2001-09-08 to arithmetically-correct 1970-01-12T13:46:40Z (1_000_000_000 ≠ 1_000_000). — 2026-06-08"
  - "v1.5: ADR-006 / Decision 13 (v0.3.0 BREAKING) — mitre_technique field replaced by mitre_techniques: Vec<String> with skip_serializing_if = Vec::is_empty; JSON key rename + value changes from scalar string to array; EC-001/EC-002 updated; EC-006 added (multi-tag). — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in EC-006 and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.006: Finding JSON Serialization: Empty Vec Fields Omitted; mitre_techniques Serialized as Array

<!--
  PREVIOUS VERSION SUMMARY (v1.4 -> v1.5):
  Field renamed: mitre_technique: Option<String> -> mitre_techniques: Vec<String>
  skip_serializing_if changed: "Option::is_none" -> "Vec::is_empty" for mitre_techniques
  EC-001: mitre_technique=None -> mitre_techniques=vec![] (empty, key absent)
  EC-002: mitre_technique=Some("T1036") -> mitre_techniques=vec!["T1036"] -> JSON: "mitre_techniques":["T1036"]
  EC-006 added: mitre_techniques=vec!["T0855","T0836"] -> JSON: "mitre_techniques":["T0855","T0836"]
  Canonical vectors updated accordingly.
  Description, Postconditions, Invariants updated to reflect the Vec<String> model and ADR-006.
-->

## Description

`Finding` is `#[derive(Serialize)]`. The `mitre_techniques: Vec<String>` field uses
`#[serde(skip_serializing_if = "Vec::is_empty")]` — the key is absent when the vec is empty
(no technique attributed) and present as a JSON array when non-empty (one or more techniques).
The remaining three Option fields (`source_ip`, `timestamp`, `direction`) continue to use
`#[serde(skip_serializing_if = "Option::is_none")]`, so None produces key-absence and not null.

This was fixed in P1.02 / #73: previously `mitre_technique` and `source_ip` always appeared
as `null` when None. The v0.3.0 ADR-006 change additionally renames `mitre_technique` to
`mitre_techniques` and changes its JSON value from a scalar string to an array, while
preserving the key-absent-when-empty behavior.

After feature-100 (issue #100): 21 of 22 original emission sites set `timestamp: Some(DateTime<Utc>)`.
The `"timestamp"` key appears in JSON for flow-data-path findings. The segment-limit summary
finding retains `timestamp: None` and produces no `"timestamp"` key.

## Preconditions

1. A `Finding` struct is being serialized via `JsonReporter` (serde_json::to_string_pretty).
2. At least one of (mitre_techniques, source_ip, timestamp, direction) is empty/None.

## Postconditions

1. `source_ip`, `timestamp`, and `direction`: `Some(value)` appears normally; `None` is
   completely absent from the JSON object (no key, no `null`).
2. `mitre_techniques`: empty vec (`vec![]`) produces no key in JSON; non-empty vec produces
   `"mitre_techniques": [...]` with the technique IDs as JSON strings.
3. The JSON schema behavior is: absence = empty/None; presence = non-empty/Some(value).
   Never: presence = null.
4. The three affected Option fields are: `source_ip`, `timestamp`, `direction`. The
   `mitre_techniques` field uses `Vec::is_empty` semantics (not `Option::is_none`).

## Invariants

1. Absence = empty vec or None. Presence = non-empty vec or Some(value). Never: null.
2. After feature-100: 21 of 22 emission sites set `timestamp: Some(DateTime<Utc>)`, so
   `"timestamp"` appears in JSON for flow-data-path findings as an ISO-8601 UTC string
   (e.g., `"timestamp": "1970-01-12T13:46:40Z"` for ts_sec=1_000_000). The segment-limit
   summary finding (the 22nd site) retains `timestamp: None` and produces no `"timestamp"` key.
3. `direction: Some(...)` is set by all HTTP and TLS analyzer findings; reassembly-engine
   findings leave it as None and therefore omit it from JSON.
4. `mitre_techniques` produces a JSON ARRAY (not a scalar string) when non-empty. A
   single-technique finding serializes as `"mitre_techniques": ["T1027"]`, not `"mitre_technique": "T1027"`.
   Downstream consumers MUST update any parser that expected a scalar string at key `"mitre_technique"`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with mitre_techniques=vec![] (empty) | JSON has no "mitre_techniques" key at all (Vec::is_empty skips it) |
| EC-002 | Finding with mitre_techniques=vec!["T1036"] | JSON has `"mitre_techniques": ["T1036"]` (array with one element) |
| EC-003 | Finding with direction=Some(ClientToServer) | JSON has `"direction": "ClientToServer"` |
| EC-004 | Reassembly-engine finding (direction=None) | JSON has no "direction" key |
| EC-005a | Flow-data-path finding (timestamp = Some after feature-100) | JSON has `"timestamp": "<ISO-8601 UTC string>"` (e.g., `"1970-01-12T13:46:40Z"` for ts_sec=1_000_000) |
| EC-005b | Segment-limit summary finding (timestamp = None; finalize aggregate) | JSON has no "timestamp" key (skip_serializing_if = "Option::is_none") |
| EC-006 | Finding with mitre_techniques=vec!["T1692.001","T0836"] (Modbus register write, co-attributed; T1692.001 is ICS sub-technique, v19 successor to revoked T0855) | JSON has `"mitre_techniques": ["T1692.001","T0836"]` (array with two elements) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding { mitre_techniques: vec![], direction: None, ... } | JSON: no "mitre_techniques" key, no "direction" key | happy-path |
| Finding { mitre_techniques: vec!["T1036"], direction: Some(ClientToServer) } | JSON: `"mitre_techniques": ["T1036"]`, `"direction": "ClientToServer"` | happy-path |
| Full pipeline HTTP finding (ts_sec=1_000_000) | JSON has `"timestamp": "1970-01-12T13:46:40Z"` | happy-path |
| Full pipeline segment-limit summary finding | No "timestamp" key in JSON for that finding | edge-case |
| Modbus register write Finding { mitre_techniques: vec!["T1692.001","T0836"] } | JSON: `"mitre_techniques": ["T1692.001","T0836"]` | happy-path (co-attributed) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Empty vec mitre_techniques produces absent key (not null, not empty array) | unit: parse JSON, assert "mitre_techniques" key not present |
| — | Non-empty mitre_techniques produces JSON array, not scalar string | unit: parse JSON, assert value is array |
| — | source_ip/timestamp/direction None produces absent key | unit: parse JSON, assert key not present |
| — | mitre_techniques uses skip_serializing_if = Vec::is_empty; source_ip/timestamp/direction use Option::is_none | code: grep for skip_serializing_if in findings.rs |

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

- `src/findings.rs` -- `#[serde(skip_serializing_if = "Vec::is_empty")]` on `mitre_techniques`; `#[serde(skip_serializing_if = "Option::is_none")]` on `source_ip`, `timestamp`, `direction`
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
