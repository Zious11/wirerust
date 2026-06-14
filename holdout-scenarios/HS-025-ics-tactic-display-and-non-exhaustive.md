---
document_type: holdout-scenario
level: ops
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-071.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.004.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.009.md
input-hash: "1c1c7a0"
# NOTE (F-C-3): input-hash is informational only — bin/compute-input-hash scopes to
# .factory/stories/ exclusively (per CLAUDE.md); holdout scenario hashes are NOT
# maintained by the canonical drift tool and may not reflect current inputs.
traces_to: .factory/specs/prd.md
id: "HS-025"
category: "behavioral-subtleties"
must_pass: "false"
priority: "should-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.10.002
  - BC-2.10.004
  - BC-2.10.009
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: ICS Tactic Display and Non-Exhaustive Enum Stability

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A security toolchain integrator using wirerust in a pipeline wants to verify that the
   MITRE ICS tactics display correctly without a "ICS:" prefix — they must render just the
   tactic name (e.g., "Inhibit Response Function"), not "ICS: Inhibit Response Function".
   EXCEPTION: `IcsImpact` renders as "Impact (ICS)" — the "(ICS)" is a parenthetical
   disambiguation qualifier (not an "ICS:" prefix) required to distinguish the ICS Impact
   tactic (TA0105) from Enterprise Impact (TA0040, bare "Impact") when both appear in the
   same grouped report. The no-"ICS:"-prefix rule applies to all OTHER ICS tactics
   (IcsInhibitResponseFunction → "Inhibit Response Function",
   IcsImpairProcessControl → "Impair Process Control"). "Impact (ICS)" does NOT violate
   the no-prefix rule; it is a parenthetical suffix.
2. The integrator also verifies that all_tactics_in_report_order contains exactly 17 entries
   (14 Enterprise + 3 ICS-unique) with no duplicates.
3. The integrator builds a downstream parser for the tactic strings. They confirm that
   adding a new MitreTactic variant in a future wirerust version would NOT break their
   parser, because MitreTactic is `#[non_exhaustive]` and match statements on it include
   a catch-all arm.
4. The ICS-specific tactic variants appear AFTER all Enterprise variants in the
   all_tactics_in_report_order output — they are at the end, not intermixed.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.002 | PC1 — ICS tactics render without "ICS:" prefix; PC3 — IcsImpact renders as "Impact (ICS)" (parenthetical ICS qualifier, not a prefix; distinguishes TA0105 ICS Impact from TA0040 Enterprise Impact); PC4 — the "(ICS)" qualifier does NOT constitute an "ICS:" prefix | Step 1: ICS tactic display names including IcsImpact disambiguation |
| BC-2.10.004 | Postcondition 1 — all_tactics_in_report_order contains every variant exactly once (17 total) | Step 2: completeness and deduplication |
| BC-2.10.009 | Postcondition 1 — MitreTactic is #[non_exhaustive] | Step 3: Rust ABI stability guarantee |

## Verification Approach

```
wirerust analyze --mitre ics_relevant.pcap
```

If no ICS-mapped findings are emitted, use a test that directly invokes
`all_tactics_in_report_order()` and prints each tactic's Display form:

- Count: exactly 17 tactics.
- No "ICS:" prefix on any tactic name. NOTE: "Impact (ICS)" is NOT an "ICS:" prefix — the
  parenthetical suffix "(ICS)" is a disambiguation qualifier. The assertion is that no tactic
  string STARTS WITH "ICS:" (prefix form). "Impact (ICS)" passes this check.
- IcsImpact renders as exactly "Impact (ICS)" (not bare "Impact"), distinguishing it from
  Enterprise MitreTactic::Impact which renders as bare "Impact" (TA0040 vs TA0105).
- ICS-specific tactic names appear last (after "Exfiltration" / "Command and Control").

For the non_exhaustive check:
Compile a downstream Rust crate that matches on `MitreTactic` with a wildcard arm `_ => {}`.
Confirm that a non-exhaustive enum forces the wildcard — this is a compile-time check,
not a runtime check. The scenario verifies that the enum is declared #[non_exhaustive].

```
grep -r "non_exhaustive" src/
```

Expect: at least one `#[non_exhaustive]` on the MitreTactic enum definition.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): all_tactics_in_report_order has exactly 17
  entries; no tactic name starts with "ICS:" (prefix form); ICS-unique tactics appear last.
- **Data integrity** (weight: 0.3): Each of the 17 canonical ATT&CK tactic names matches
  the expected string (case-sensitive). IcsImpact expected string is "Impact (ICS)" (not bare
  "Impact"); IcsInhibitResponseFunction expected string is "Inhibit Response Function";
  IcsImpairProcessControl expected string is "Impair Process Control". Enterprise
  MitreTactic::Impact expected string is bare "Impact" (TA0040, no qualifier).
- **Edge case handling** (weight: 0.2): The #[non_exhaustive] attribute is actually present
  in source code (not just asserted to be there).
- **Error quality** (weight: 0.1): Iteration over all 17 tactics produces no errors.

## Edge Conditions

- If ICS tactics are never emitted in practice, the ordering contract is still testable
  by calling all_tactics_in_report_order() directly in a test.
- The three ICS-unique tactics at the end of the list must be in a consistent order (not
  randomized) between runs.
- MitreTactic serialization in JSON must produce stable string values that match the
  Display output.

## Failure Guidance

"HOLDOUT LOW: HS-025 (satisfaction: 0.XX) — ICS tactic names have 'ICS:' prefix (note:
'Impact (ICS)' parenthetical qualifier is NOT a prefix violation), IcsImpact renders as bare
'Impact' instead of 'Impact (ICS)', tactic count is not 17, or #[non_exhaustive] is missing
from MitreTactic definition."
