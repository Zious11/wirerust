---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3 (2026-05-28): F-W16-S042-P5-003 invariant-1 line-precise anchor prose added — shell_patterns array at http.rs:206-217; iter().any() guard at http.rs:218; finding push at http.rs:219-232; if-body closing `}` at http.rs:233. Matches precision of BC-2.06.005 v1.6. Verified against src/analyzer/http.rs:206-233. Closes F-W16-S042-P5-003 (006 direction). — 2026-05-28"
  - "v1.4 (2026-05-29): F-DRIFT2A-001 — fixed stale domain/capabilities/cap-06-http-analysis.md citation to domain/capabilities/cap-06-http-analysis.md in L2 Capability and Capability Anchor Justification rows."
  - "v1.5 (2026-06-13): ARP-F2-Pass14-Burst5 — Postcondition 1 mitre_technique: Some(\"T1505.003\") → mitre_techniques: vec![\"T1505.003\"] (Finding struct field renamed to plural Vec<String>)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.006: Web-Shell URI Patterns Emit Execution/Likely/Medium Finding (T1505.003)

## Description

When the parsed HTTP request URI (lowercased) contains any of the known web-shell path
patterns (`/shell.php`, `/shell.asp`, `/shell.jsp`, `/cmd.php`, `/cmd.asp`, `/cmd.jsp`,
`/c99.php`, `/r57.php`, `/webshell`, `/backdoor`), the HttpAnalyzer emits an
`Execution/Likely/Medium` finding tagged with MITRE T1505.003 (Server Software Component:
Web Shell). The URI is included in the evidence without escaping (ADR 0003).

## Preconditions

1. A complete HTTP/1.1 or HTTP/1.0 request has been parsed.
2. The lowercased URI contains at least one of the 10 shell_patterns strings as a substring.
3. The flow direction is ClientToServer.

## Postconditions

1. A Finding is emitted with:
   - category: Execution
   - verdict: Likely
   - confidence: Medium
   - mitre_techniques: vec!["T1505.003"]
   - summary: "Possible web shell access: <truncated URI (120 chars max)>"
   - evidence: vec!["URI: <raw URI>"]
   - direction: Some(Direction::ClientToServer)
2. The finding fires per-request (not per-flow-once).

## Invariants

1. URI comparison is case-insensitive (lowercased before pattern match). The `shell_patterns` array is defined at http.rs:206-217; the iter().any() guard is at http.rs:218; the finding push block spans http.rs:219-232; the if-body closing `}` is at http.rs:233.
2. Pattern match is substring: a URI like `/uploads/c99.php?cmd=id` triggers the finding.
3. Raw URI bytes preserved in evidence (INV-4 / ADR 0003).
4. If the URI also matches path-traversal patterns, BOTH findings are emitted independently.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI = "/shell.php" | Finding emitted (T1505.003) |
| EC-002 | URI = "/uploads/SHELL.PHP" (uppercase) | Finding emitted (case-insensitive match) |
| EC-003 | URI = "/webshell/config.xml" | Finding emitted (substring match on /webshell) |
| EC-004 | URI = "/backdoor" | Finding emitted |
| EC-005 | URI = "/cmd.php/../etc/passwd" | Two findings: T1505.003 (web shell) + T1083 (path traversal) |
| EC-006 | URI = "/download.php" | No finding (not in pattern list) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /shell.php HTTP/1.1\r\n... | Finding(Execution/Likely/Medium, T1505.003) | happy-path |
| GET /c99.php?id=1 HTTP/1.1\r\n... | Finding(Execution/Likely/Medium, T1505.003) | happy-path |
| GET /normal/page.php HTTP/1.1\r\n... | No T1505.003 finding | happy-path |
| GET /webshell/cmd.php HTTP/1.1\r\n... | Two findings (webshell + cmd.php both match) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Web-shell URI pattern emits T1505.003 finding | unit: test_detect_webshell_path |
| — | Non-shell URI emits no web-shell finding | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- web shell detection is one of the core HTTP anomaly findings |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:206-233, C-12) |
| Stories | STORY-042 |
| Origin BC | BC-HTTP-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.005 -- related to (both detections can fire on the same request independently)
- BC-2.06.026 -- composes with (raw URI bytes preserved per ADR 0003)

## Architecture Anchors

- `src/analyzer/http.rs:206-233` -- web shell detection block (shell_patterns array + finding push)
- `tests/http_analyzer_tests.rs` -- test_detect_webshell_path

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:206-233` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `shell_patterns.iter().any(|p| uri_lower.contains(p))`
- **assertion**: test_detect_webshell_path

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
