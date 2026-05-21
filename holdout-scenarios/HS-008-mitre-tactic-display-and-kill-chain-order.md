---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-071.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.001.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.004.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md
input-hash: "409bc32"
traces_to: .factory/specs/prd.md
id: "HS-008"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.10.001
  - BC-2.10.003
  - BC-2.10.004
  - BC-2.10.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: MITRE ATT&CK Tactic Display Names and Kill-Chain Order Completeness

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A SOC operator uses wirerust to analyze a pcap containing TLS anomalies and TCP
   reassembly evasion behavior, both of which produce findings with MITRE technique IDs.
2. The operator invokes the `--mitre` grouping mode in terminal output.
3. The terminal output groups findings under tactic headers that appear in kill-chain order
   (Reconnaissance first, Exfiltration toward the end).
4. Each tactic header uses the canonical ATT&CK display name — for example,
   "Command and Control" (with lowercase "and", correct spacing), not "CommandAndControl".
5. The operator checks a known technique ID (T1036 — Masquerading) and observes the
   correct tactic assignment under "Defense Evasion".
6. Each tactic appears at most once in the grouped output — no tactic header is duplicated.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.001 | Postcondition 1 — Enterprise tactics render with canonical spacing (e.g., "Command and Control") | Step 4: tactic display names |
| BC-2.10.003 | Postcondition 1 — all_tactics_in_report_order returns kill-chain order first then ICS-unique | Step 3: tactic header ordering |
| BC-2.10.004 | Postcondition 1 — all_tactics_in_report_order contains every variant exactly once (16 total) | Step 6: no duplicate tactic headers |
| BC-2.10.005 | Postcondition 1 — technique_name returns Some for all 15 seeded IDs | Step 5: T1036 resolves to "Masquerading" |

## Verification Approach

Run on a pcap that triggers both T1036 (conflicting overlap) and T1027 (TLS SNI anomaly):

```
wirerust analyze --mitre mixed_anomalies.pcap
```

Check terminal output:
1. Tactic headers appear in kill-chain order — "Reconnaissance", "Resource Development",
   "Initial Access", ... "Defense Evasion", ... "Exfiltration", "Command and Control"
   at the end.
2. "Command and Control" must be spelled exactly — lowercase "and", two spaces.
3. "Defense Evasion" must appear and T1036 findings must appear under it.
4. No tactic header is repeated.
5. The technique name "Masquerading" must appear alongside T1036 findings.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Kill-chain order is correct; no tactic skipped,
  no tactic repeated; canonical display names used.
- **Data integrity** (weight: 0.3): T1036 resolves to "Masquerading" under "Defense Evasion";
  T1027 resolves correctly under "Defense Evasion".
- **Edge case handling** (weight: 0.15): Findings with no MITRE technique appear under
  "Uncategorized" section, not silently dropped.
- **Error quality** (weight: 0.1): Unknown technique IDs render with "(unknown)" label rather
  than crashing.

## Edge Conditions

- A capture with findings spanning 3+ different tactics exercises the full ordering logic.
- If only one tactic is triggered, remaining tactic headers should still appear (or not) —
  verify that empty tactic buckets do not produce empty headers in the output.
- ICS-specific tactics (T0xxx) should appear after the Enterprise kill-chain order.

## Failure Guidance

"HOLDOUT LOW: HS-008 (satisfaction: 0.XX) — MITRE tactic display names are wrong (wrong
casing or spacing), kill-chain order is not followed, or duplicate tactic headers appear."
