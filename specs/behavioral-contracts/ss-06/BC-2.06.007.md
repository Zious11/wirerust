---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: Wave 16 Pass-2 (F-W16-S042-P2-002) — rewrite EC-005 expected behavior: /administrator DOES fire because admin matching is substring-based (.contains); removed contradictory WAIT: inline self-correction — 2026-05-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.007: Admin Panel Paths Emit Reconnaissance/Inconclusive/Low Finding (T1046)

## Description

When the parsed HTTP request URI (lowercased) contains any of the admin-panel path substrings
(`/wp-admin`, `/admin`, `/phpmyadmin`, `/manager`), the HttpAnalyzer emits a
`Reconnaissance/Inconclusive/Low` finding tagged with MITRE T1046 (Network Service Discovery).
This is a low-confidence indicator because admin paths are routinely accessed by legitimate
administrators; the finding is informational rather than definitive.

## Preconditions

1. A complete HTTP/1.1 or HTTP/1.0 request has been parsed.
2. The lowercased URI contains at least one of the 4 admin_patterns strings as a substring.
3. The flow direction is ClientToServer.

## Postconditions

1. A Finding is emitted with:
   - category: Reconnaissance
   - verdict: Inconclusive
   - confidence: Low
   - mitre_technique: Some("T1046")
   - summary: "Admin panel access: <truncated URI (120 chars max)>"
   - evidence: vec!["URI: <raw URI>"]
   - direction: Some(Direction::ClientToServer)
2. The finding fires per-request.

## Invariants

1. URI comparison is case-insensitive (lowercased before match).
2. Pattern match is substring: `/site/admin/settings` triggers the finding via `/admin`.
3. Raw URI bytes preserved in evidence (INV-4 / ADR 0003).
4. The finding is independent of other detections on the same request.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI = "/wp-admin/post.php" | Finding emitted |
| EC-002 | URI = "/phpmyadmin" | Finding emitted |
| EC-003 | URI = "/manager/html" | Finding emitted (Tomcat manager pattern) |
| EC-004 | URI = "/ADMIN" (uppercase) | Finding emitted (case-insensitive) |
| EC-005 | URI = "/administrator/index.php" | Finding emitted — "/administrator" contains "/admin" as a substring; matching is substring-based (http.rs:237 uses `.contains`), not token-exact |
| EC-006 | URI = "/sadmin/config" | Finding emitted ("/sadmin" contains "/admin" as substring) |
| EC-007 | URI = "/my-site/content" | No finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /wp-admin/edit.php HTTP/1.1\r\n... | Finding(Reconnaissance/Inconclusive/Low, T1046) | happy-path |
| GET /admin HTTP/1.1\r\n... | Finding(Reconnaissance/Inconclusive/Low, T1046) | happy-path |
| GET /index.html HTTP/1.1\r\n... | No finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Admin panel URI emits T1046 finding with Inconclusive/Low | unit: test_detect_admin_panel_paths |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- admin panel access detection is one of the HTTP reconnaissance findings |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:235-249, C-12) |
| Stories | STORY-042 |
| Origin BC | BC-HTTP-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.005 -- related to (independent detection; can co-fire on same request)
- BC-2.06.006 -- related to (independent detection; can co-fire on same request)

## Architecture Anchors

- `src/analyzer/http.rs:235-249` -- admin panel detection block
- `tests/http_analyzer_tests.rs` -- test_detect_admin_panel_paths

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:235-249` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `admin_patterns.iter().any(|p| uri_lower.contains(p))`
- **assertion**: test_detect_admin_panel_paths

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
