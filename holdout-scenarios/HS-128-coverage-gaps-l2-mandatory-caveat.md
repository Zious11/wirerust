---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.024.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.023.md
  - .factory/stories/STORY-154.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-128"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.12.024
  - BC-2.12.023
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires: (1) an empty or near-empty pcap (no TCP/UDP traffic) to test L2 caveat with empty entries array; (2) any pcap with at least one unclassified TCP or UDP flow to test L2 caveat with non-empty entries."
input-hash: "d4fbaeb"
---

# Holdout Scenario: `CoverageGapsSummary` — Mandatory L2/Multicast Caveat Always Present

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The `CoverageGapsSummary` section (produced when `--coverage-gaps` is set) MUST include
a mandatory L2/multicast structural limitation caveat in EVERY invocation, regardless of
whether the entries array is empty or non-empty. This caveat explains that LinkLayer
protocols (GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) have no
TCP/UDP port and are therefore invisible to the dynamic gap report — a fundamental detection
blind spot that operators must be warned about.

The caveat is NOT configurable and NOT suppressible. It is a fixed static string.

### Case A — Empty Pcap + `--coverage-gaps`: Caveat Present Despite No Entries

1. The evaluator creates or obtains an empty pcap file (valid pcap with LINKTYPE_ETHERNET
   header but zero packets, OR a pcap/pcapng with no TCP/UDP packets). Call it `empty.pcap`.
2. The evaluator runs: `wirerust analyze empty.pcap --coverage-gaps`
3. The tool exits 0.
4. The terminal output contains the `CoverageGapsSummary` section.
5. The caveat text references LinkLayer protocols and names at minimum GOOSE and the reason
   it cannot appear (no TCP/UDP port). Expected keyword: "Layer-2", "LinkLayer", "L2", or
   "GOOSE" in the caveat.
6. The entries section is empty (or shows "no entries" / zero gap entries).
7. The L2 caveat IS present even though entries is empty. This is mandatory: the caveat
   always appears when `--coverage-gaps` is set.

### Case B — `--json` Empty Pcap + `--coverage-gaps`: JSON Has `"caveat_l2"` Non-Null String

1. The evaluator runs: `wirerust analyze empty.pcap --coverage-gaps --json`
2. The tool exits 0.
3. `jq '."coverage_gaps"."caveat_l2"'` is a non-null, non-empty string.
4. `jq '."coverage_gaps"."entries" | length'` == 0 (no gap entries for empty pcap).
5. The caveat_l2 string references L2 protocols (contains "Layer-2", "L2", "LinkLayer",
   or names like "GOOSE").

### Case C — `--coverage-gaps` With Unclassified Traffic: Caveat STILL Present (Not Only for Empty)

1. The evaluator obtains a pcap with at least one unclassified TCP or UDP flow (any port
   not dissected by wirerust, such as TCP/9600 or UDP/47808 without `--bacnet` flag).
2. The evaluator runs: `wirerust analyze unclassified.pcap --coverage-gaps`
3. The tool exits 0.
4. The terminal output contains the L2 caveat text AND at least one gap entry.
5. The L2 caveat is present even when entries are non-empty. This confirms the caveat is
   truly unconditional, not just shown when the entries list is empty.

### Case D — JSON Schema: `"coverage_gaps"` Object Form `{ "caveat_l2": "...", "entries": [...] }`

1. The evaluator runs: `wirerust analyze empty.pcap --coverage-gaps --json`
2. The JSON `"coverage_gaps"` value is an OBJECT (not an array, not a flat dict of string keys).
3. It has EXACTLY the structure: `{ "caveat_l2": <string>, "entries": <array> }`.
4. There is no top-level `"caveat_l2"` key — it is nested under `"coverage_gaps"`.

### Case E — L2 Caveat Names the Five LinkLayer Protocols

1. The evaluator runs: `wirerust analyze empty.pcap --coverage-gaps`
2. The L2 caveat text in the terminal output mentions at minimum: "GOOSE" AND one of
   "Sampled Values", "PROFINET", "EtherCAT", or "POWERLINK".
   (The exact wording may vary, but the five LinkLayer protocols whose absence from the
   gap report might surprise operators must be represented.)

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.024 | L2 caveat always present when --coverage-gaps set | Cases A, C: empty and non-empty |
| BC-2.12.024 | L2 caveat present even when entries array is empty | Case A: empty pcap still shows caveat |
| BC-2.12.024 | JSON: caveat_l2 non-null string in coverage_gaps object | Cases B, D |
| BC-2.12.024 | JSON schema is object form { caveat_l2, entries } NOT flat | Case D |
| BC-2.12.024 | Caveat mentions L2 protocols including GOOSE | Case E |
| BC-2.12.023 | coverage_gaps JSON key present when flag set | Cases B, D |

<!-- HIDDEN TRACEABILITY: BC-2.12.024 Postcondition 1 (L2 caveat always in CoverageGapsSummary);
     BC-2.12.024 Invariant 1 (caveat not configurable; not suppressible; always present);
     BC-2.12.024 EC-001 (empty pcap case: entries empty but caveat still present);
     BC-2.12.023 Postcondition 3 (JSON schema: object form per v1.2 correction from BC-2.12.023 PC-3) -->

## Verification Approach

```bash
# Case A — empty pcap: caveat present, entries empty
wirerust analyze empty.pcap --coverage-gaps | grep -i 'Layer\|L2\|GOOSE\|LinkLayer\|multicast'
# Expect: at least one line with L2 caveat text

# Case B — JSON caveat_l2
wirerust analyze empty.pcap --coverage-gaps --json | jq '."coverage_gaps"."caveat_l2"'
# Expect: non-null string containing L2/GOOSE/LinkLayer reference

wirerust analyze empty.pcap --coverage-gaps --json | jq '."coverage_gaps"."entries" | length'
# Expect: 0

# Case C — non-empty entries: caveat still present
wirerust analyze unclassified.pcap --coverage-gaps | grep -i 'Layer\|L2\|GOOSE\|LinkLayer'
# Expect: at least one line (caveat is unconditional)

# Case D — JSON is an object, not flat dict
wirerust analyze empty.pcap --coverage-gaps --json | jq '."coverage_gaps" | type'
# Expect: "object"

wirerust analyze empty.pcap --coverage-gaps --json | jq '."coverage_gaps" | keys'
# Expect: includes "caveat_l2" and "entries"
```

## Evaluation Rubric

- **L2 caveat always present (empty pcap)** (weight: 0.40): Case A: caveat text appears even
  when entries are empty. This is the key invariant: the caveat is unconditional.
- **JSON caveat_l2 non-null** (weight: 0.25): Case B: JSON has `"caveat_l2"` as non-null string.
- **L2 caveat with non-empty entries** (weight: 0.20): Case C: caveat present when entries exist.
- **JSON object form** (weight: 0.15): Case D: coverage_gaps is an object, not a flat dict.

## Failure Guidance

"HOLDOUT FAIL: HS-128 — L2 caveat not present or conditional.
Case A failure: if CoverageGapsSummary appears but the L2 caveat is absent when entries are
empty, the caveat is gated on entries being non-empty. The caveat MUST appear unconditionally
(BC-2.12.024 Invariant 1). The L2_CAVEAT_TEXT constant must always be rendered.
Case D failure: if jq sees 'coverage_gaps' as an array or flat key-value object instead of
a nested object with 'caveat_l2' and 'entries', the JSON schema uses the pre-BC-v1.2 format.
Use the authoritative object form: { 'caveat_l2': '...', 'entries': [...] }."
