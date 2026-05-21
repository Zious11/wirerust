---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.011: Empty UA Emits Anomaly/Inconclusive/Low; Absent UA Does NOT

## Description

The HttpAnalyzer applies an intentionally asymmetric rule for the User-Agent header:
only a present-but-empty value (`user_agent == Some("")`) triggers an
`Anomaly/Inconclusive/Low` finding. A completely absent User-Agent header (`user_agent == None`)
does NOT fire. This asymmetry is documented in the source with cited research rationale
(http.rs:319-343): empty-UA is a stronger malware signal (Kheir 2015), while missing-UA
is routine for cron jobs and microservices. No MITRE technique ID is assigned.

## Preconditions

1. A complete HTTP request has been parsed.
2. `parsed.user_agent == Some("")` (header present, value empty after trim).

## Postconditions

1. A Finding is emitted with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - mitre_technique: None
   - summary: "Empty User-Agent header"
   - evidence: vec!["<method> <uri>"]
   - direction: Some(Direction::ClientToServer)
2. No finding is emitted when `parsed.user_agent == None` (header absent).

## Invariants

1. `find_header` returns `Some("")` for `User-Agent: \r\n` (whitespace only, after trim).
2. Absent header -> `None` -> no finding. This is the intentional design per domain-debt O-02.
3. `mitre_technique` is `None` for this finding.
4. The asymmetry is documented in the source and is NOT a defect; it is a deliberate policy.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | User-Agent: curl/7.0 | No finding |
| EC-002 | No User-Agent header present | No finding |
| EC-003 | User-Agent: (empty) | Finding emitted |
| EC-004 | User-Agent:   (whitespace only) | Finding emitted (trim produces "") |
| EC-005 | Empty UA + missing Host (HTTP/1.1) | Both findings emitted independently |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET / HTTP/1.1\r\nUser-Agent: \r\n\r\n | Finding(Anomaly/Inconclusive/Low, "Empty User-Agent header") | happy-path |
| GET / HTTP/1.1\r\nUser-Agent: curl/7.0\r\n\r\n | No UA finding | happy-path |
| GET / HTTP/1.1\r\n\r\n (no UA header) | No UA finding | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Empty UA emits Anomaly/Inconclusive/Low | unit: test_detect_empty_user_agent |
| — | Absent UA emits no finding | unit: test_missing_user_agent_no_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- empty User-Agent detection is one of the HTTP anomaly findings; absent-UA non-detection is an intentional design choice |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:344-356, C-12) |
| Stories | STORY-043 |
| Origin BC | BC-HTTP-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.009 -- related to (Host detection is symmetric both None and empty; UA is asymmetric)

## Architecture Anchors

- `src/analyzer/http.rs:344-356` -- empty UA detection block
- `src/analyzer/http.rs:319-343` -- source comments with Kheir 2015 rationale
- `tests/http_analyzer_tests.rs` -- test_detect_empty_user_agent, test_missing_user_agent_no_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:344-356` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if parsed.user_agent.as_deref() == Some("")`
- **documentation**: in-source Kheir 2015 citation and Snort/Suricata comparison
- **assertion**: test_detect_empty_user_agent, test_missing_user_agent_no_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
