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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.006.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.007.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.012.md
input-hash: "5db4ba5"
traces_to: .factory/stories/STORY-041.md
id: "HS-053"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.005
  - BC-2.06.006
  - BC-2.06.007
  - BC-2.06.012
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: URI Threat Detections Fire Correctly and Independently

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains four HTTP/1.1 GET requests across the same flow, with URIs: (a) `/images/photo.jpg`, (b) `/wp-admin/post.php`, (c) `/uploads/c99.php?cmd=id`, (d) `/etc/passwd/../..%2f..`.
2. The analyst runs wirerust on this pcap.
3. Request (a) produces zero findings.
4. Request (b) produces exactly one finding with MITRE technique T1046 (Network Service Discovery, Discovery tactic) and verdict Inconclusive.
5. Request (c) produces exactly one finding for web-shell detection with MITRE T1505.003.
6. Request (d) produces exactly one finding for path traversal with MITRE T1083.
7. No finding from request (d) leaks MITRE technique from the web-shell check, and vice versa.
8. A fifth request with URI `/cmd.php/../../etc/passwd` produces TWO findings — one for web-shell, one for path traversal.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.005 | postcondition 1-2; invariant 1 | Path traversal patterns; four exact strings; T1083; per-request |
| BC-2.06.006 | postcondition 1; invariant 1-2 | Web-shell patterns; case-insensitive substring; T1505.003 |
| BC-2.06.007 | postcondition 1; invariant 1-2 | Admin panel paths; Inconclusive/Low; T1046 |
| BC-2.06.012 | postcondition 1-3 | Clean URI (request a) produces zero findings |

## Verification Approach

Craft a pcap with the five HTTP GET requests described. Run wirerust with JSON output.

1. Inspect `findings` array — assert exactly 5 findings total.
2. Assert `findings` contains a T1083 finding with evidence containing `../` (request d).
3. Assert `findings` contains a T1505.003 finding with evidence containing `c99.php` (request c).
4. Assert `findings` contains a T1046 finding with evidence containing `wp-admin` (request b).
5. Assert `findings` contains TWO findings for request (e) — one T1083, one T1505.003.
6. Assert no finding has URI `/images/photo.jpg` in its evidence.
7. Assert all URI-based findings have `direction: "ClientToServer"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Correct findings emitted for each attack URI; clean URI produces zero findings.
- **Edge case handling** (weight: 0.35): Overlapping URI (both web-shell and traversal) produces two independent findings.
- **Error quality** (weight: 0.1): Findings include truncated URI in summary (max 120 chars) and full raw URI in evidence.
- **Data integrity** (weight: 0.1): MITRE technique codes are exact string values matching T1083, T1505.003, T1046.

## Edge Conditions

- URI matching is lowercase (case-insensitive) so `C99.PHP` must also trigger web-shell finding.
- Multiple detection patterns in one URI — both findings must appear, not just the first.
- Normal URIs never trigger any detection regardless of how many similar-looking path segments they contain.

## Failure Guidance

"HOLDOUT LOW: HS-053 (satisfaction: 0.XX) -- URI threat detection produced wrong finding count, wrong MITRE codes, or failed to suppress findings for clean URIs; check pattern lists and independence of detection gates."
