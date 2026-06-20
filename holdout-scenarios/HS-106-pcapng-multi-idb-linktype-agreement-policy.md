---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.018.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-106"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.018
verification_properties:
  - VP-030
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng Multi-IDB Linktype Agreement Policy — Conflict Rejected, Uniform Accepted

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

A pcapng file may contain multiple Interface Description Blocks (IDBs), one per capture
interface. wirerust requires all IDBs in a section to agree on linktype (ADR-009 Decision 3).
This policy preserves the single-DataLink model in PcapSource without touching decoder.rs.
This scenario tests the two critical outcomes: conflict rejected immediately on the second
IDB (before any EPB), and two same-linktype IDBs accepted with packets from both interfaces.

### Case A — Two IDBs with different linktypes → E-INP-011 on the SECOND IDB (before any EPB)

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - IDB-0: linktype=1 (Ethernet / DLT_EN10MB)
   - IDB-1: linktype=113 (Linux Cooked Capture v1 / DLT_LINUX_SLL)
   - EPB for interface 0 (valid Ethernet frame — but the file is rejected BEFORE this EPB
     is processed, because the conflict is detected at IDB-1 parse time)
2. The user runs `wirerust analyze two_idb_conflict.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr referencing a linktype conflict or
   multi-interface mismatch (E-INP-011 or equivalent). No packets are in JSON output.
   No panic occurs.
4. Critical timing: the rejection MUST occur at or before IDB-1 processing — not deferred
   to the first EPB with interface_id=1. The observable check: exit non-zero AND no packet
   data in stdout.

### Case B — Two IDBs with SAME linktype (both linktype=1) → Accepted, packets from both interfaces read

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - IDB-0: linktype=1 (Ethernet), if_tsresol=6
   - IDB-1: linktype=1 (Ethernet), if_tsresol=6
   - EPB for interface 0 (interface_id=0) carrying a minimal Ethernet frame (packet 1)
   - EPB for interface 1 (interface_id=1) carrying a different minimal Ethernet frame (packet 2)
2. The user runs `wirerust analyze two_idb_same.pcapng --json`.
3. The tool exits 0. JSON output contains total_packets = 2. Both packets (from interfaces
   0 and 1) are in the output. No error. This confirms that multi-IDB files are accepted
   when linktypes agree, and that interface_id=1 is correctly resolved from the 2-entry
   interface table.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.018 | Postcondition 1 — all IDBs same linktype: accepted | Case B: two IDBs linktype=1; both EPBs processed |
| BC-2.01.018 | Postcondition 2 — IDB linktype conflict returns E-INP-011 | Case A: second IDB with different linktype triggers immediate rejection |
| BC-2.01.018 | Postcondition 3 — rejection on SECOND IDB, before any EPB | Case A: the error fires at IDB-1 parse time, not at first EPB for interface 1 |
| BC-2.01.018 | Postcondition 4 — multi-IDB same-linktype: packets from all interfaces read | Case B: interface_id=1 EPB is resolved correctly from the 2-entry table |
| BC-2.01.018 | No-panic invariant | Both cases: no panic |

## Verification Approach

```
wirerust analyze two_idb_conflict.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr referencing linktype conflict or multi-interface
disagreement, no JSON on stdout. The stderr message should be human-readable and ideally
identify the conflicting linktype values (e.g., "linktype 1 (IDB-0) vs 113 (IDB-1)").

```
wirerust analyze two_idb_same.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, JSON with total_packets = 2, no errors on stderr.

For Case A, additional check:
```
wirerust analyze two_idb_conflict.pcapng --json 2>/dev/null | wc -c
```
Should output `0` (no JSON on stdout), confirming no partial packet output before rejection.

For Case B, the evaluator confirms that both packets appear in JSON output — verifying
that the interface_id=1 EPB was correctly dispatched using the second IDB's linktype (1)
rather than rejected as OOB or producing a link-layer decode error.

## Evaluation Rubric

- **Correctness — conflict detection** (weight: 0.40): Case A: second IDB with different
  linktype results in non-zero exit with readable error. The rejection is at IDB-1 time
  (before any EPB), confirmed by absence of JSON output.
- **Correctness — multi-IDB acceptance** (weight: 0.35): Case B: both interfaces' packets
  appear in output; total_packets = 2; no errors.
- **Error quality** (weight: 0.15): Case A error message is human-readable and identifies
  the file as having conflicting interface linktypes; does not expose raw pointer addresses
  or Rust internal details.
- **No-panic safety** (weight: 0.10): No panic for either input.

## Edge Conditions

- The conflict check fires on the SECOND IDB (IDB-1), not on the first EPB that
  references interface_id=1. This is a strict policy: as soon as a conflicting IDB is
  seen, the entire file is rejected. This is the correct behavior because the error occurs
  at the structural level, not at packet-dispatch time.
- linktype=1 and linktype=113 are specifically chosen because both are supported by
  wirerust's existing link-layer decoder (Ethernet and Linux Cooked Capture are in the
  whitelist). The conflict is purely at the multi-IDB policy level — wirerust could decode
  both individually, but the single-DataLink model prohibits mixing them in one PcapSource.
- Case B uses interface_id=0 and interface_id=1 in the EPBs. The interface table after two
  same-linktype IDBs has two entries; interface_id=1 maps to table[1] (the second IDB).
  This exercises the multi-entry table resolution without the OOB conditions tested in HS-104.
- A pcapng file with 3 or more IDBs all having the same linktype is a logical extension of
  Case B; the proptest property (VP-030) covers arbitrary sequences.

## Known Limitation

As noted in ADR-009 Consequences (Negative / Trade-offs): the multi-IDB conflict policy
will reject legitimate multi-NIC capture files that mix interface types (e.g., Ethernet +
Linux Cooked, as in this Case A). Users can work around this with `mergecap -w out.pcapng
-I 0 <file>` to isolate a single interface. This limitation is accepted for this cycle and
is explicitly documented in BC-2.01.018.

## Failure Guidance

"HOLDOUT LOW: HS-106 (satisfaction: 0.XX) — pcapng multi-IDB policy has defects.
Case A exit 0 or partial output means the linktype-conflict check is absent or deferred.
Case A panic means the conflict check is absent and the second IDB causes a state corruption.
Case B exit non-zero means same-linktype multi-IDB files are incorrectly rejected (over-strict).
Case B total_packets = 1 means the interface_id=1 EPB was skipped or caused an OOB error.
See BC-2.01.018, VP-030, ADR-009 Decision 3 and Decision 11."
