---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: Wave 16 Pass-1 prose fix (F-W16-S042-P1-002) — stray quote in backslash-pattern negation corrected (`..\\\"` → `..\\ `) — 2026-05-28"
  - "v1.4: Wave 16 Pass-2 (F-W16-S042-P2-001) — tighten invariant 1 anchor 187-191 → 187-190 (191 is closing brace, not a contains() call) — 2026-05-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.005: Path Traversal in URI Emits Reconnaissance/Likely/High Finding Mapped to T1083

## Description

When the parsed HTTP request URI (lowercased) contains any of the substrings `../`,
`..%2f`, `..%252f`, or `....//`, the HttpAnalyzer emits a `Reconnaissance/Likely/High`
finding tagged with MITRE T1083 (File and Directory Discovery). The raw URI bytes are
preserved in the finding evidence without escaping (ADR 0003).

## Preconditions

1. A complete HTTP/1.1 or HTTP/1.0 request has been parsed by httparse.
2. The lowercased request URI contains at least one of the substrings `../`, `..%2f`, `..%252f`, or `....//`.
3. The flow direction is ClientToServer (path traversal is a client-originated anomaly).

## Postconditions

1. A Finding is emitted with:
   - category: Reconnaissance
   - verdict: Likely
   - confidence: High
   - mitre_technique: Some("T1083")
   - summary: `format!("Path traversal in URI: {}", truncate_uri(&parsed.uri, 120))` (URI truncated to 120 chars in summary)
   - evidence: `vec![format!("URI: {}", parsed.uri)]` (full raw URI, NO truncation in evidence)
   - direction: Some(Direction::ClientToServer)
2. The detection fires per-request (not per-flow-once).
3. The HTTP parse statistics are updated normally (transaction count, method count, etc.).

## Invariants

1. The check is a substring match on the lowercased URI: `uri_lower.contains("../") || uri_lower.contains("..%2f") || uri_lower.contains("..%252f") || uri_lower.contains("....//")`. These are the exact four patterns at http.rs:187-190 (line 191 is the closing brace of the `if` block, not a pattern call). There is NO backslash (`..\`) traversal variant in source.
2. Raw URI bytes are preserved in evidence (ADR 0003 / INV-4).
3. The finding fires even if the request also triggers other detections (e.g., admin path).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI = "/../etc/passwd" | Finding emitted with T1083 |
| EC-002 | URI = "/normal/path/../../sensitive" | Finding emitted |
| EC-003 | URI = "..%2fetc%2fpasswd" (URL-encoded) | Finding emitted |
| EC-004 | URI = "..%252f" (double-encoded) | Finding emitted |
| EC-005 | URI = "....//etc/passwd" | Finding emitted |
| EC-006 | URI = "/normal/path" | No finding |
| EC-007 | HTTP/1.0 request with path traversal | Finding emitted (HTTP/1.0 is NOT exempt) |
| EC-008 | Request also has empty User-Agent | Both findings emitted independently |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /../etc/passwd HTTP/1.1\r\n... | Finding(Reconnaissance/Likely/High, T1083, direction=ClientToServer) | happy-path |
| GET /..%2fetc/passwd HTTP/1.1\r\n... | Finding(Reconnaissance/Likely/High, T1083) | happy-path |
| GET /normal/path HTTP/1.1\r\n... | No finding | happy-path |
| GET /../../../boot.ini HTTP/1.1\r\n... | Exactly one finding (not one per ..) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Path traversal URI emits T1083 finding | unit: test_detect_path_traversal |
| — | URL-encoded traversal is detected | unit: test_detect_encoded_traversal |
| — | Normal path emits no finding | unit: test_no_findings_for_normal_request |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP traffic analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP traffic analysis") per capabilities.md §CAP-06 -- path traversal detection is one of the core HTTP anomaly findings |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:186-202, C-12) |
| Stories | STORY-042 |
| Origin BC | BC-HTTP-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.026 -- composes with (raw header bytes preserved per ADR 0003)
- BC-2.06.015 -- related to (poisoning could suppress this detection after 3 consecutive errors)

## Architecture Anchors

- `src/analyzer/http.rs:186-191` -- path traversal detection: `if uri_lower.contains("../")` and encoded variants
- `src/analyzer/http.rs:192-202` -- Finding construction and push (T1083, Reconnaissance/Likely/High)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:186-202` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: test_detect_path_traversal, test_detect_encoded_traversal

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
