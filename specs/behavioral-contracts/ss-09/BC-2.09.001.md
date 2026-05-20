---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.001: Finding Constructed with Required Fields and Optional Fields

## Description

The `Finding` struct is constructed directly via struct-literal syntax at each of the 22
emission sites. Required fields (`category`, `verdict`, `confidence`, `summary`, `evidence`)
must always be provided. Optional fields (`mitre_technique`, `source_ip`, `timestamp`,
`direction`) are always `Option<T>`. All 22 current emission sites set `timestamp: None`
(domain-debt O-01; the field exists in the struct but is never populated). There is no
builder or constructor helper -- every site provides the full literal.

## Preconditions

1. An analyzer or engine has detected a condition warranting a Finding.
2. The caller has appropriate `category`, `verdict`, `confidence`, `summary`, and `evidence`
   values ready.

## Postconditions

1. A `Finding` value is constructed with:
   - `category`: one of `ThreatCategory` variants (Reconnaissance, Anomaly, Execution, Persistence, etc.)
   - `verdict`: one of `Verdict::Likely | Unlikely | Inconclusive`
   - `confidence`: one of `Confidence::High | Medium | Low`
   - `summary`: raw `String` (per ADR 0003; no escape applied at construction)
   - `evidence`: `Vec<String>` (raw; 0 or more entries)
   - `mitre_technique`: `Option<String>` (None or a technique ID string)
   - `source_ip`: `Option<IpAddr>` (None in all current 22 emission sites)
   - `timestamp`: `Option<DateTime<Utc>>` (None in all current 22 emission sites -- O-01)
   - `direction`: `Option<Direction>` (Some for HTTP/TLS findings; None for reassembly findings)
2. No allocation beyond the struct fields themselves.
3. The constructed value is valid to pass to any reporter.

## Invariants

1. All 22 current emission sites set `timestamp: None` (O-01; forensic gap).
2. All 22 current emission sites set `source_ip: None` (no IP attribution at Finding level).
3. HTTP and TLS analyzer findings set `direction: Some(...)`.
4. Reassembly engine findings set `direction: None`.
5. `summary` and `evidence` carry raw bytes (ADR 0003 / INV-4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | evidence vec is empty | Valid Finding; reporters omit evidence section |
| EC-002 | summary is empty string | Valid Finding; unusual but not rejected |
| EC-003 | mitre_technique = Some("T1036") | Value flows through to JSON output and MITRE grouping |
| EC-004 | mitre_technique = None | No "mitre_technique" key in JSON (skip_serializing_if) |
| EC-005 | Finding with direction = Some(ServerToClient) | direction field set; JSON emits "ServerToClient" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HttpAnalyzer path-traversal detection | Finding { category: Anomaly, verdict: Likely, confidence: High, direction: Some(ClientToServer), timestamp: None } | happy-path |
| Reassembly conflicting overlap | Finding { category: Anomaly, direction: None, timestamp: None } | happy-path |
| TLS SNI control-byte detection | Finding { category: Anomaly, mitre_technique: Some("T1027") } | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 22 emission sites produce Finding with timestamp=None | grep: no site sets timestamp: Some(...) |
| VP-TBD | HTTP and TLS findings carry direction=Some | unit: assert direction is Some after HTTP/TLS analysis |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 -- this BC defines the full schema of the Finding struct which is the core output type of CAP-09 |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-09 (findings.rs, C-10) |
| Stories | S-TBD |
| Origin BC | BC-FND-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.005 -- composes with (raw-bytes contract for summary/evidence)
- BC-2.09.006 -- composes with (JSON serialization of None fields)
- BC-2.09.002 -- composes with (Display rendering of this struct)

## Architecture Anchors

- `src/findings.rs:119-146` -- Finding struct definition with all fields and serde attributes

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:119-146` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: struct-literal syntax at emission sites; compiler enforces all fields
- **documentation**: O-01 doc comment on timestamp; INV-4 doc comment

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (Finding is an owned value) |
| **Overall classification** | pure |

## Refactoring Notes

O-01 (timestamp always None) is the key open item. Wiring would require threading
`RawPacket.timestamp_secs` through `StreamHandler::on_data` to every emission site. Until
O-01 is resolved, the `chrono` crate dependency is present but the timestamp field carries
no forensic value.
