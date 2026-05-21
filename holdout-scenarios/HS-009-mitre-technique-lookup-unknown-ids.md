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
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.006.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.008.md
input-hash: "409bc32"
traces_to: .factory/specs/prd.md
id: "HS-009"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.10.005
  - BC-2.10.006
  - BC-2.10.007
  - BC-2.10.008
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: MITRE Technique Catalog — Known ID Lookup, Unknown ID Graceful Handling

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A malware researcher runs wirerust on a capture containing HTTP path traversal, TLS
   SNI obfuscation, web shell access, and TCP stream manipulation — a rich set of
   detections that exercises multiple MITRE technique IDs.
2. Every finding in the output that carries a known MITRE technique ID (T1083, T1505.003,
   T1046, T1036, T1027) shows the correct human-readable technique name in the `--mitre`
   terminal view.
3. Each known technique ID maps to the correct parent tactic (T1083 -> Reconnaissance,
   T1505.003 -> Persistence, T1027 -> Defense Evasion, T1036 -> Defense Evasion).
4. If a future caller were to query an unknown technique ID (e.g., "T9999"), the lookup
   returns a "none" or empty result — no panic, no crash.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.005 | Postcondition 1 — technique_name returns Some for all 15 seeded IDs | Step 2: all emitted technique IDs resolve |
| BC-2.10.006 | Postcondition 1 — technique_name returns None for unknown IDs | Step 4: graceful unknown-ID handling |
| BC-2.10.007 | Postcondition 1 — technique_tactic returns correct tactic for every seeded ID | Step 3: tactic-to-technique mapping accuracy |
| BC-2.10.008 | Postcondition 1 — all technique IDs currently emitted by analyzers resolve in lookup | Step 2: end-to-end resolution coverage |

## Verification Approach

```
wirerust analyze --mitre --output-format json multi_anomaly.pcap
```

For each finding with a `mitre_technique_id`:
- Confirm the name visible in terminal matches the ATT&CK catalogue entry.
- Confirm the tactic in terminal matches the expected parent tactic.

Lookup exhaustion: run wirerust on a capture that produces each of the 5 currently-emitted
technique IDs (T1083, T1505.003, T1046, T1036, T1027). All 5 must resolve.

Synthesize a test where a non-standard ID would be queried (e.g., via a code path test)
and confirm no panic occurs.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All emitted technique IDs resolve to correct
  names and tactics with no errors.
- **Data integrity** (weight: 0.3): Tactic assignments match MITRE ATT&CK Enterprise v14
  catalog entries (e.g., T1027 -> Defense Evasion, not Execution).
- **Edge case handling** (weight: 0.1): Unknown ID returns None, not panic or empty string.
- **Error quality** (weight: 0.1): No spurious warnings or errors when technique lookup succeeds.

## Edge Conditions

- T1505.003 is a sub-technique ID with a period; the lookup must handle the period correctly
  and not treat "T1505" and "T1505.003" as the same ID.
- T0886 or similar ICS technique IDs should not confuse the lookup (ICS IDs are in the catalog).
- The 9 catalogued-but-never-emitted IDs (T1040, T1071, etc.) should still return Some from
  technique_name — they are in the catalog even if no analyzer emits them.

## Failure Guidance

"HOLDOUT LOW: HS-009 (satisfaction: 0.XX) — one or more emitted MITRE technique IDs does
not resolve to the correct name or tactic, or an unknown ID causes a crash."
