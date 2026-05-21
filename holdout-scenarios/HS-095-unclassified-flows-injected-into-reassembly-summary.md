---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.014.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.015.md
input-hash: "529c948"
traces_to: .factory/stories/STORY-086.md
id: "HS-095"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.015
  - BC-2.12.014
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Unclassified Flows Count Appears in Reassembly Summary; Absent Without Reassembler

## Scenario

The stream dispatcher tracks flows that could not be classified (unknown protocol). This
count must be injected into the reassembly analyzer's detail map after `finalize()` is called.
If no reassembler was used, this key must not appear at all.

**Part A — with reassembler:**
1. The tool is invoked with `--http` (or `--tls`) against a pcap that has at least one
   TCP flow with an unclassified protocol (not HTTP, not TLS).
2. The JSON or terminal output for the reassembly analyzer section includes a field or
   detail entry named `"unclassified_flows"` with a non-negative integer value.
3. Even if `unclassified_flows = 0`, the key is present in the output (zero is explicitly
   tracked, not omitted).

**Part B — without reassembler:**
1. The tool is invoked with `--dns` only (no `--http`, no `--tls`, no `--reassemble`).
2. The reassembly analyzer section does NOT appear in the output.
3. No `"unclassified_flows"` key appears anywhere in the output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.015 | Postcondition 1: unclassified_flows injected when reassembler present | Part A: key present in reassembly summary |
| BC-2.12.015 | Invariant 1: absent without reassembler | Part B: key not present in DNS-only run |
| BC-2.12.014 | Postcondition 3: skipped_packets = total_decode_errors | Context: error counting does not interfere |

## Verification Approach

**Part A:**
Run `wirerust analyze --http --json <pcap-with-mixed-tcp-traffic>`. Parse the JSON.
Find the analyzer summary with name `"TCP Reassembly"` (or similar).
Assert: that summary's `detail` map contains key `"unclassified_flows"`.
Assert: the value is a non-negative integer.
Assert: even if the value is 0, the key is present.

**Part B:**
Run `wirerust analyze --dns --json <pcap>`. Parse the JSON.
Assert: no analyzer summary named `"TCP Reassembly"` exists.
Assert: searching the entire JSON output for `"unclassified_flows"` returns no match.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Part A has the key; Part B does not.
- **Edge case handling** (weight: 0.2): Zero unclassified_flows is still reported (not omitted).
- **Error quality** (weight: 0.1): No crash when the dispatcher has zero unclassified flows.
- **Performance** (weight: 0.05): No performance impact from the key injection.
- **Data integrity** (weight: 0.15): The injection happens after `finalize()`; not before.

## Edge Conditions

- Reassembler present, all flows classified: `"unclassified_flows": 0` appears (present, not omitted).
- `--no-reassemble --http`: reassembler is None; unclassified_flows is absent; warning appears on stderr.
- `--reassemble` without any stream analyzer: reassembler is constructed; unclassified_flows still injected.

## Failure Guidance

"HOLDOUT LOW: HS-095 (satisfaction: 0.XX) -- The unclassified_flows count was missing from the reassembly analyzer summary when a reassembler was active, or incorrectly appeared in the output when no reassembler was constructed."
