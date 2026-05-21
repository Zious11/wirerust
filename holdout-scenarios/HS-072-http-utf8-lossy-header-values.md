---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-041.md
  - .factory/stories/STORY-042.md
  - .factory/stories/STORY-043.md
  - .factory/stories/STORY-044.md
  - .factory/stories/STORY-045.md
  - .factory/stories/STORY-046.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.026.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-041.md
id: "HS-072"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.026
  - BC-2.06.001
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Header Values With Non-UTF-8 Bytes Are Stored With Replacement Characters

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains an HTTP/1.1 request where:
   - The `Host` header value contains a non-UTF-8 byte sequence: `\xff` followed by `example.com` (a byte that cannot start a valid UTF-8 sequence).
   - The `User-Agent` header value has leading and trailing whitespace: `   curl/7.88.0   `.
   - The URI contains no special characters: `/index.html`.
2. The analyst runs wirerust on this pcap.
3. The `top_hosts` field in the HTTP summary contains an entry where the non-UTF-8 byte has been replaced by U+FFFD: the key is `\u{fffd}example.com` (or its visual equivalent with the replacement diamond).
4. The `user_agents` field in the HTTP summary contains `curl/7.88.0` (whitespace trimmed — no leading/trailing spaces).
5. The raw URI `/index.html` flows into detection code without any transformation.
6. Zero findings are emitted (no traversal, no web-shell, no admin panel, no unusual method).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.026 | postcondition 1-4; invariant 4 | find_header uses from_utf8_lossy + trim; non-UTF-8 replaced with U+FFFD; raw URI not escaped |
| BC-2.06.001 | postcondition 1-7 | Host and UA values correctly extracted with lossy UTF-8 and trim semantics |

## Verification Approach

Craft a pcap with the HTTP request described. Run wirerust with JSON output.

1. Inspect `analyzers[HTTP].detail.top_hosts`. Assert the Host value stored contains the U+FFFD replacement character (e.g., a unicode replacement diamond in the JSON output).
2. Inspect `analyzers[HTTP].detail.user_agents`. Assert the UA value is `"curl/7.88.0"` without leading/trailing whitespace.
3. Assert `findings` is empty (URI `/index.html` is clean).
4. Assert `analyzers[HTTP].detail.parse_errors == "0"` (non-UTF-8 host header is not a parse error).
5. Assert `analyzers[HTTP].detail.transactions` and `methods` are correctly updated.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Non-UTF-8 host bytes replaced with U+FFFD; UA whitespace trimmed; URI not transformed.
- **Edge case handling** (weight: 0.3): Non-UTF-8 in a header value is not a parse error and does not increment parse_errors; it is silently normalized.
- **Error quality** (weight: 0.15): No findings emitted for this clean (but byte-unusual) request.
- **Data integrity** (weight: 0.1): find_header applies from_utf8_lossy before trim, in that order.

## Edge Conditions

- `from_utf8_lossy` must be applied BEFORE `trim` — if trim is applied first on raw bytes, it could panic on non-UTF-8.
- The raw URI from `req.path` must NOT be transformed at this layer — raw bytes flow into detection and into `Finding.evidence`.
- A User-Agent header with only whitespace (e.g., `User-Agent:   `) produces `Some("")` after trim, triggering the empty-UA finding. This scenario uses `curl/7.88.0` with whitespace, not whitespace-only.

## Failure Guidance

"HOLDOUT LOW: HS-072 (satisfaction: 0.XX) -- HTTP header value extraction failed; non-UTF-8 host byte was not replaced with U+FFFD (from_utf8_lossy required), or User-Agent whitespace was not trimmed, or parse_errors was incorrectly incremented."
