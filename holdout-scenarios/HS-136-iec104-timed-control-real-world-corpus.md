---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-23T00:00:00Z
phase: f3
inputs: []
input-hash: "d41d8cd"
traces_to: .factory/specs/prd.md
id: "HS-136"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "feature-iec104"
behavioral_contracts:
  - BC-2.19.029
  - BC-2.19.030
  - BC-2.19.022
verification_properties:
  - VP-047
lifecycle_status: active
introduced: wave-85-spec-evolution
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: IEC104-TIMED-CMD-GAP-001
fixture_needed: false
fixture_note: "Uses publicly-available ICS network capture corpora. See corpus_source fields below."
---

# Holdout Scenario: IEC-104 Timed Control Commands — Real-World Corpus Validation

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **REAL-WORLD CORPUS:** This scenario validates timed-command detection against publicly-available
> ICS network traffic captures. It tests (1) zero false positives against known-good IEC-104
> traffic that contains no attacker-issued time-tagged commands, and (2) correct detection in
> known-problematic traffic containing time-tagged commands issued outside normal engineering
> workflow (indicating potential unauthorized use).

## Corpus Sources

### Known-Good Corpus: 4SICS ICS Village 2015 Network Traffic

**Source:** 4SICS Geek Lounge — ICS Village 2015 pcap release  
**URL:** https://www.netresec.com/?page=PCAP4SICS  
**Maintainer:** Netresec (Erik Hjelmvik); updated on each 4SICS conference release  
**Size:** ~11 GB total; select only files with IEC-104 traffic (~200 MB subset, files 3–5 of the 2015 set)  
**License:** Publicly available for research and tool development  
**Rationale:** Represents legitimate industrial network traffic from a live ICS environment at
the 4SICS conference (Stockholm). Contains IEC-104 control commands from legitimate SCADA
engineers — any T1692.001/T0836 hits from TypeIDs 58–64 in this corpus would be false positives,
since no adversarial activity was documented for these captures.

### Known-Problematic Corpus: ICS-PCAP Repo Red Team Samples

**Source:** GitHub `ICS-PCAP` (ics-pcap.com), specifically the `iec104-attacks/` directory  
**URL:** https://github.com/automayt/ICS-PCAP  
**Maintainer:** Dale Peterson / Digital Bond community repository  
**Size:** Small targeted attack pcaps, typically < 5 MB each  
**License:** Public domain / research use  
**Rationale:** Contains documented IEC-104 attack scenarios including command injection.
The `iec104-attacks/` subdirectory includes crafted frames with TypeIDs spanning the control
direction, some of which use CP56Time2a variants (TypeIDs 58–64). Known findings from prior
analysis include unauthorized switching commands (TypeID 58–60) against a simulated RTU.

## Scenario

### Case A — Known-Good Corpus: False-Positive Rate Below Threshold

**Input:** 4SICS 2015 IEC-104 pcap subset (~200 MB, legitimate SCADA session).

The evaluator runs:
```bash
wirerust analyze 4sics_iec104_subset.pcap --iec104 --json
```

Counts findings where `mitre_techniques` contains `"T1692.001"` OR `"T0836"` AND the
`summary` contains `"time-tagged"` (indicating a TypeID 58–64 hit).

**Expected:** 0 such findings. Any TypeID 58–64 hit in normal SCADA engineering traffic
is a false positive because timed commands are uncommon in standard engineering sessions
and the 4SICS corpus contains no documented adversarial TypeID 58–64 activity.

**Acceptable threshold:** ≤ 1 false positive per 1,000 IEC-104 frames in the corpus.
If the corpus contains ~10,000 IEC-104 ASDUs, the total TypeID-58–64 false-positive
findings must be ≤ 10. (Operator-initiated timed commands during a test session are
acceptable — they are not false positives in the detection sense, only TypeIDs from
attacker-emulated frames would be. The threshold accommodates any legitimate timed
commands present in the corpus.)

### Case B — Known-Problematic Corpus: Attacker Timed-Command Detection

**Input:** ICS-PCAP `iec104-attacks/` pcap containing documented TypeID 58–60 injection
against a simulated RTU.

The evaluator runs:
```bash
wirerust analyze iec104_attack_timed_switching.pcap --iec104 --json
```

Counts findings where `mitre_techniques` contains `"T1692.001"` AND `summary` contains
`"time-tagged"` or timed mnemonic (C_SC_TA/C_DC_TA/C_RC_TA).

**Expected:** At least one T1692.001 finding per injected TypeID 58/59/60 frame.
If the known-problematic corpus contains N frames with TypeIDs 58–60, all N must produce
T1692.001 findings (false-negative rate = 0 for documented attack frames).

### Case C — Known-Problematic Corpus: Parameter-Write Timed Commands

**Input:** ICS-PCAP or equivalent corpus containing TypeID 61–64 frames (timed set-point
commands issued outside nominal engineering workflow — e.g., outside maintenance windows,
from unknown source IPs).

The evaluator runs:
```bash
wirerust analyze iec104_attack_timed_setpoint.pcap --iec104 --json
```

Counts findings where `mitre_techniques` contains both `"T1692.001"` AND `"T0836"`.

**Expected:** For each TypeID 61–64 frame in the attack portion, exactly 2 findings
(T1692.001 + T0836 pair). False-negative rate = 0 for documented attack frames.

### Case D — Mixed Corpus: No Bleed from Non-Timed Arms

Using the 4SICS legitimate corpus, verify that existing untimed TypeID 45–51 frames
(if present in the corpus) still produce their expected T1692.001/T0836 findings, and
that the addition of timed arms did not alter untimed-arm behavior.

**Expected:** Any TypeID 45–47 frames in legitimate SCADA traffic that previously
produced T1692.001 findings continue to do so. TypeID 48–51 frames continue to produce
T1692.001 + T0836. No change in per-arm finding counts vs. pre-wave-85 baseline.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Corpus Case |
|-------|--------------|-------------|
| BC-2.19.029 | Postcondition 1 — T1692.001 for TypeIDs 58–60 | Case B |
| BC-2.19.030 | Postconditions 1–2 — T1692.001 + T0836 for TypeIDs 61–64 | Case C |
| BC-2.19.022 v1.1 | Invariant 1 — silently-logged range {52–57, 65–99} only | Case A (no overflow) |

## Verification Approach

```bash
# Case A: false-positive rate check (known-good corpus)
wirerust analyze 4sics_iec104_subset.pcap --iec104 --json | \
  jq '[.findings[] | select(
    (.mitre_techniques[] == "T1692.001") and
    (.summary | test("time-tagged|C_SC_TA|C_DC_TA|C_RC_TA|C_SE_TA|C_SE_TB|C_SE_TC|C_BO_TA"))
  )] | length'
# Expect: ≤ 10 (threshold: ≤ 1 per 1,000 IEC-104 frames)

# Case B: attack detection (TypeID 58–60 injection)
wirerust analyze iec104_attack_timed_switching.pcap --iec104 --json | \
  jq '[.findings[] | select(
    (.mitre_techniques[] == "T1692.001") and
    (.summary | test("time-tagged|C_SC_TA|C_DC_TA|C_RC_TA"))
  )] | length'
# Expect: == N (one per injected TypeID 58/59/60 frame in the corpus)

# Case C: timed set-point attack detection
wirerust analyze iec104_attack_timed_setpoint.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T0836")] | length'
# Expect: > 0 (one per TypeID 61–64 frame)

# Case D: untimed-arm non-regression on real corpus
# Isolate untimed-only T1692.001 findings by excluding timed-mnemonic summaries.
# Untimed summaries contain "(C_SC/C_DC/C_RC)" or "(C_SE/C_BO)" — no "time-tagged" qualifier.
# Timed summaries contain "time-tagged" or a timed mnemonic (C_SC_TA/C_DC_TA etc.).
# Negating the timed-mnemonic pattern selects only untimed-arm findings.
wirerust analyze 4sics_iec104_subset.pcap --iec104 --json | \
  jq '[.findings[] | select(
    (.mitre_techniques[] == "T1692.001") and
    (.summary | test("time-tagged|C_SC_TA|C_DC_TA|C_RC_TA|C_SE_TA|C_SE_TB|C_SE_TC|C_BO_TA") | not)
  )] | length'
# Expect: same count as pre-wave-85 (no regression on untimed arms)
```

## Evaluation Rubric

- **False-positive rate below threshold** (weight: 0.30): Case A — ≤ 1 TypeID-58–64 hit
  per 1,000 IEC-104 frames on legitimate corpus.
- **Zero false-negatives on known attack frames** (weight: 0.40): Cases B + C — every
  documented TypeID 58–60 and 61–64 attack frame produces findings.
- **Untimed-arm non-regression on real corpus** (weight: 0.20): Case D — same per-arm
  finding counts as pre-wave-85 on 4SICS corpus.
- **No TypeID overflow into neighbors** (weight: 0.10): Case A — zero T1692.001 findings
  with TypeID 52–57 or 65–99 origin (verified via summary string pattern).

## Edge Conditions

- **Legitimate timed commands in normal operations:** Engineering workstations may legitimately
  issue TypeID 58–64 commands during commissioning, maintenance, or testing. These are true
  positives at the detection level (Verdict::Possible means "warrants investigation") — the
  CASDU/IOA evidence lets operators confirm legitimacy. They do NOT count against the false-
  positive threshold, which applies only to unexpected TypeID-58–64 frames with no operational
  context.
- **4SICS corpus caveat:** If the corpus contains a SCADA session where an engineer explicitly
  issued timed commands, those will produce T1692.001 findings. This is correct behavior. The
  evaluator must manually confirm any hit against the pcap content before labeling it a false
  positive.
- **TypeID count = 0 ASDUs:** An ASDU with VSQ=0 (count=0) still emits the finding(s) —
  emission is count-independent (BC-2.19.029 Invariant 3 and BC-2.19.030 Invariant 3). The
  real corpus may contain such frames; they count toward the false-positive and false-negative
  thresholds just like any other ASDU.
- **COT test bit in corpus:** Some captures include frames with the test bit set (COT bit 7).
  These produce findings with " [TEST]" in the summary. They are valid findings and count toward
  the false-negative threshold (they should still be detected) but are annotated.
- **CP56Time2a parsing variance:** TypeIDs 58–64 each have different information-object sizes.
  If the ICS-PCAP corpus uses non-standard CP56Time2a encodings, parsing failures may silently
  drop the frame to the `_` catch-all. The evaluator must confirm frame-level parse success
  (no "malformed ASDU" log entries) before attributing a miss to a false-negative.
- **ICS-PCAP repo directory structure:** The `ICS-PCAP` repo directory names may differ between
  clone dates. If `iec104-attacks/` is absent, check `IEC104/`, `iec-104/`, or `attacks/`.
  The holdout-evaluator agent must resolve the correct path at evaluation time.

## Category: real-world-corpus

This scenario IS the real-world corpus validation. Cases A and B are the primary corpus
cases required by the holdout-scenario-template.md rc.23 real-world corpus mandate.

| Field | Description |
|-------|-------------|
| corpus_source | Known-good: 4SICS 2015 ICS Village pcap (https://www.netresec.com/?page=PCAP4SICS); Known-problematic: ICS-PCAP iec104-attacks/ (https://github.com/automayt/ICS-PCAP) |
| corpus_size | Known-good: ~200 MB IEC-104 subset (~10,000 ASDUs est.); Known-problematic: < 5 MB targeted attack pcaps |
| known_edge_cases | Legitimate timed commands in SCADA engineering sessions (not false positives); CP56Time2a variant lengths across TypeIDs |
| false_positive_threshold | ≤ 1 TypeID-58–64 T1692.001 hit per 1,000 IEC-104 frames on 4SICS known-good corpus |
| false_negative_threshold | 0 missed TypeID 58–64 frames in ICS-PCAP documented attack scenarios |

## Fixture Creation Obligation

No synthetic fixture is needed. The corpora are publicly available:

1. **4SICS 2015 ICS Village pcap:** Download from https://www.netresec.com/?page=PCAP4SICS.
   Filter for IEC-104 traffic: `tshark -r <pcap> -Y "iec104" -w 4sics_iec104_subset.pcap`.
   The holdout-evaluator agent is responsible for filtering and staging this corpus.

2. **ICS-PCAP attacks:** Clone https://github.com/automayt/ICS-PCAP. Navigate to
   `ICS-PCAP/IEC104/` or `ICS-PCAP/iec104-attacks/` (directory name varies by repo
   version). Use pcaps with documented TypeID 58–60 or 61–64 frames.

Both corpora are reproducible and version-stable (4SICS pcaps are archived; ICS-PCAP
is git-tracked).

## Failure Guidance

"HOLDOUT FAIL: HS-136 (satisfaction: 0.XX) — IEC-104 timed-command real-world corpus
validation. If Case A false-positive rate exceeds threshold, timed-arm detection is
over-broad — verify arms are strictly bounded to 58..=60 and 61..=64, not wider ranges.
If Case B/C misses documented attack frames, the match arm implementation is incorrect
or the CP56Time2a offset parsing is failing, causing frame misclassification. If Case D
shows untimed-arm count change, a match arm collision with pre-existing arms has occurred.
See BC-2.19.029 Invariant 6 and BC-2.19.030 Invariant 6 for neighbor-silence invariants."
