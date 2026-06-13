---
document_type: holdout-scenario
level: ops
version: "1.1"
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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.008.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.009.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.010.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.011.md
input-hash: "d270d81"
traces_to: .factory/stories/STORY-041.md
id: "HS-058"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.008
  - BC-2.06.009
  - BC-2.06.010
  - BC-2.06.011
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Header Anomaly Detections Are Independent and Threshold-Correct

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains five HTTP requests in the same flow:
   - (A) `DELETE /resource HTTP/1.1\r\nHost: api.test\r\n\r\n` (unusual method)
   - (B) `GET /page HTTP/1.1\r\n\r\n` (HTTP/1.1, no Host header)
   - (C) `GET /` followed by 2049 'a' characters for the URI `HTTP/1.1\r\nHost: h.test\r\n\r\n` (URI exactly 1 character over the 2048-character threshold)
   - (D) `POST /login HTTP/1.1\r\nHost: h.test\r\nUser-Agent: \r\n\r\n` (User-Agent present but empty after trim)
   - (E) `GET /page HTTP/1.1\r\nHost: h.test\r\n\r\n` (clean request — no User-Agent header at all)
2. The analyst runs wirerust on this pcap.
3. Request (A) produces one finding for unusual method (no MITRE technique).
4. Request (B) produces one finding for missing Host on HTTP/1.1 (no MITRE technique).
5. Request (C) produces one finding for long URI; the summary contains the exact byte count (2049); evidence is truncated to 200 characters.
6. Request (D) produces one finding for empty User-Agent (no MITRE technique).
7. Request (E) produces ZERO findings — absent User-Agent is not flagged.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.008 | postcondition 1; invariant 1-2 | DELETE is unusual method; case-sensitive match; no MITRE |
| BC-2.06.009 | postcondition 1-3 | HTTP/1.1 without Host produces finding; HTTP/1.0 would be exempt |
| BC-2.06.010 | postcondition 1; invariant 1-3 | URI > 2048 fires; threshold is strictly greater-than; byte count in summary |
| BC-2.06.011 | postcondition 1-2; invariant 2 | Empty UA (Some("")) fires; absent UA (None) does NOT fire |

## Verification Approach

Craft a pcap with the five HTTP requests. Run wirerust with JSON output.

1. Assert `findings` contains exactly 4 findings total.
2. Assert one finding has `summary` containing "DELETE" and `mitre_techniques` key is absent (empty vec, omitted via `skip_serializing_if`).
3. Assert one finding has `summary` containing "Host header" or "Host" and HTTP/1.1 context.
4. Assert one finding has `summary` containing "2049" (the exact URI byte count).
5. Assert one finding has `summary` containing "User-Agent".
6. Assert NO finding relates to request (E) (absent User-Agent produces zero findings).
7. Assert all four findings have `direction == "ClientToServer"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly 4 findings for the 5 requests; correct detection for each anomaly type.
- **Edge case handling** (weight: 0.35): Absent UA produces no finding (asymmetry); URI at exactly 2049 triggers (not 2048); method match is case-sensitive.
- **Error quality** (weight: 0.15): Long URI summary contains exact byte count; evidence is truncated to 200 chars.
- **Data integrity** (weight: 0.1): MITRE technique is null for all four header anomaly findings.

## Edge Conditions

- User-Agent absent vs. User-Agent empty: only empty triggers; absent is intentionally ignored.
- Long-URI threshold is strictly greater-than 2048 — a URI of exactly 2048 characters must NOT trigger.
- Method matching is case-sensitive — "delete" (lowercase) would not trigger the CONNECT/TRACE/DELETE/OPTIONS check.
- Multiple anomalies in one request (not tested here but implied): they fire independently.

## Failure Guidance

"HOLDOUT LOW: HS-058 (satisfaction: 0.XX) -- HTTP header anomaly detection miscounted findings; check UA absent/empty asymmetry, long-URI threshold (> 2048 not >= 2048), or method case-sensitivity."
