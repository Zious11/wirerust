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
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md
  - .factory/stories/STORY-153.md
  - .factory/stories/STORY-154.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-130"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.12.024
  - BC-2.05.010
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires two crafted pcap fixtures: (1) a pcap with at least one TCP flow to port 102 that closes without being classified (no S7comm/MMS/ICCP dissector in wirerust, so it becomes DispatchTarget::None); (2) a pcap with no TCP/102 traffic, to verify the footnote is absent."
canonical_value_scenario: true
canonical_spec_citation: "TCP port 102 is used by S7comm (Siemens S7 PLC communication), S7comm-plus (Siemens S7-1200/S7-1500), IEC 61850 MMS per IEC 61850-8-1 via ISO-on-TCP (RFC 1006), and ICCP/TASE.2 per IEC 60870-6 via ISO-on-TCP (RFC 1006 / TPKT framing). All four protocols multiplex on TCP/102 using the ISO Transport Service on top of TCP, making port-102 gap reports unattributable to a single protocol."
input-hash: "704ff63"
---

# Holdout Scenario: `CoverageGapsSummary` — Port-102 Collision Footnote Triggered by TCP/102 Traffic (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies that when unclassified TCP/102 traffic appears in the gap report,
a collision footnote is rendered that names all four protocols sharing port 102.

**Four protocols sharing TCP/102 via ISO-on-TCP (RFC 1006 / TPKT):**
1. S7comm — Siemens S7 PLC protocol (S7-300/S7-400 series)
2. S7comm-plus — Siemens S7 updated protocol (S7-1200/S7-1500 series)
3. IEC 61850 MMS (Manufacturing Message Specification) — IEC 61850-8-1, ISO-on-TCP per RFC 1006
4. ICCP/TASE.2 — IEC 60870-6, ISO-on-TCP per RFC 1006 / TPKT

> Source: RFC 1006 §5 (ISO Transport Service on top of TCP, assigned port 102);
> Wireshark dissector registrations for all four protocols at port 102;
> IEC 61850-8-1 §4 (MMS over TCP/102); IEC 60870-6 (ICCP/TASE.2 over TCP/102).
> This is a CANONICAL-VALUE scenario because the four-protocol collision on TCP/102 is
> a well-documented fact independently of the project's implementation.

The port-102 footnote must appear adjacent to the TCP/102 gap entry when that entry has
a non-zero count. It must NOT appear when TCP/102 count is zero or the entry is absent.

## Scenario

### Case A — TCP/102 Unclassified Traffic: Collision Footnote Present in Terminal Output

1. The evaluator creates a pcap (`port102_traffic.pcap`) with at least one TCP flow closing
   on port 102 that wirerust does NOT have a dissector for. (wirerust has no S7comm/MMS/ICCP
   dissector; a TCP flow on port 102 that closes will be counted as `DispatchTarget::None`
   when `--coverage-gaps` is enabled and at least one analyzer is active.)
2. The evaluator runs: `wirerust analyze port102_traffic.pcap --coverage-gaps`
   (With at least one analyzer enabled, e.g., `--http` or `--all`, so the dual-gate passes.)
3. The tool exits 0.
4. The terminal output's CoverageGapsSummary contains:
   - A TCP/102 gap entry (port 102, TCP transport).
   - Adjacent to or below this entry: a collision footnote naming ALL FOUR protocols:
     "S7comm", "S7comm-plus", "IEC 61850 MMS", and "ICCP" (or "ICCP/TASE.2" or "TASE.2").
5. The TCP/102 entry is NOT suppressed — count is shown alongside the footnote.

### Case B — JSON Mode: TCP/102 Entry Has `"collision_note"` Field Naming All Four Protocols

1. The evaluator runs: `wirerust analyze port102_traffic.pcap --coverage-gaps --json`
   (With at least one analyzer enabled.)
2. The tool exits 0.
3. `jq '.coverage_gaps.entries[] | select(.port == 102 and .transport == "TCP")'` shows:
   - `"state": "known-unsupported"` — TCP/102 is catalogued as carrying unsupported protocols
   - `"collision_note": <string>` — the collision_note field is PRESENT and non-null
   - The collision_note string contains at minimum "S7comm", "S7comm-plus", "MMS", "ICCP"
     (or equivalent protocol names; all four must be represented).
4. The entry is NOT absent from the JSON output (port-102 collision annotation does not
   suppress the entry; it annotates it).

### Case C — No TCP/102 Traffic: Collision Footnote Absent

1. The evaluator creates a pcap (`no_port102.pcap`) with unclassified traffic that does
   NOT include TCP/102 flows (e.g., only TCP/9600 or UDP/47808 unclassified traffic).
2. The evaluator runs: `wirerust analyze no_port102.pcap --coverage-gaps`
   (With at least one analyzer enabled.)
3. The tool exits 0.
4. The terminal output does NOT contain a port-102 collision footnote.
5. The JSON `"coverage_gaps"."entries"` for TCP/102 is either absent or has count=0.
   In either case, the collision_note field is absent or null.

### Case D — Dual-Gate: No TCP/102 Counts Without an Active Analyzer

1. The evaluator creates a pcap with TCP/102 traffic.
2. The evaluator runs: `wirerust analyze port102_traffic.pcap --coverage-gaps`
   WITHOUT any analyzer flags (no --http, no --tls, no --all). Only `--coverage-gaps`.
3. The dual-gate (coverage_gaps_enabled AND ≥1 analyzer present) means the TCP counter
   fires ONLY when at least one analyzer is active. Without any analyzer, the counter should
   NOT increment (or increment zero times).
4. Expected: either no TCP/102 entry in the CoverageGapsSummary, or count=0.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.010 | TCP counter: `(TransportProto::Tcp, 102)` incremented for None-target TCP/102 flows | Cases A, B: counter fires |
| BC-2.12.024 | Port-102 collision footnote triggered when (Tcp,102) count > 0 | Case A: footnote present |
| BC-2.12.024 | Collision footnote names S7comm, S7comm-plus, MMS, ICCP/TASE.2 | Cases A, B: all four named |
| BC-2.12.024 | JSON: collision_note field in TCP/102 entry | Case B: field present |
| BC-2.12.024 | Collision footnote absent when TCP/102 count == 0 or absent | Case C: no footnote |
| BC-2.05.010 | Dual-gate: counter requires coverage_gaps_enabled AND ≥1 analyzer | Case D: no count without analyzers |

<!-- HIDDEN TRACEABILITY: BC-2.12.024 Postcondition 2 (PORT_102_NOTE adjacent to TCP/102 entry);
     BC-2.12.024 Invariant 2 (footnote if and only if (Tcp,102) count > 0; row-specific, not global);
     BC-2.05.010 PC-1 (dual-gate: coverage_gaps_enabled AND analyzer-present guard);
     BC-2.12.024 EC-003 (TCP/102 non-zero count → footnote present) -->

## Fixture Creation Guidance

**Port-102 traffic fixture:**
- TCP SYN to port 102 followed by FIN or RST, so the flow closes as `DispatchTarget::None`.
- Use `min(src_port, dst_port) = 102` as the `lower_port` key:
  - src=1234, dst=102: `lower_port()` = 102 ✓
  - src=102, dst=1234: `lower_port()` = 102 ✓ (bidirectional normalization)
- Invoke with `--http` or `--all` (or any single analyzer) to satisfy the dual-gate.

## Verification Approach

```bash
# Case A — TCP/102 footnote in terminal (RFC 1006 / ISO-on-TCP / TPKT; 4 protocols)
wirerust analyze port102_traffic.pcap --coverage-gaps --http | grep -i 'S7comm'
# Expect: at least one line mentioning S7comm (in the collision footnote)

wirerust analyze port102_traffic.pcap --coverage-gaps --http | grep -i 'MMS\|IEC 61850'
# Expect: at least one line

wirerust analyze port102_traffic.pcap --coverage-gaps --http | grep -i 'ICCP\|TASE'
# Expect: at least one line

# Case B — JSON collision_note
wirerust analyze port102_traffic.pcap --coverage-gaps --http --json | \
  jq '.coverage_gaps.entries[] | select(.port == 102 and .transport == "TCP") | .collision_note'
# Expect: non-null string containing "S7comm", "S7comm-plus", "MMS"/"IEC 61850 MMS", "ICCP"/"TASE.2"

# Case C — no TCP/102 traffic: no footnote
wirerust analyze no_port102.pcap --coverage-gaps --http | grep -i 'S7comm'
# Expect: zero lines

# Case D — no analyzer: TCP/102 counter zero
wirerust analyze port102_traffic.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries[] | select(.port == 102) | .count'
# Expect: 0 or entry absent (no analyzers active = dual-gate fails)
```

## Evaluation Rubric

- **All four protocols named in footnote** (weight: 0.40): Cases A, B: S7comm, S7comm-plus,
  IEC 61850 MMS, ICCP/TASE.2. CANONICAL-VALUE must-pass. Missing any name is a specification
  violation. All four share TCP/102 via RFC 1006 / ISO-on-TCP / TPKT.
- **JSON collision_note field** (weight: 0.30): Case B: `"collision_note"` non-null in TCP/102 entry.
- **Footnote absent without TCP/102 traffic** (weight: 0.20): Case C: conditional, not global.
- **Dual-gate** (weight: 0.10): Case D: no count without active analyzer.

## Failure Guidance

"HOLDOUT FAIL: HS-130 — port-102 collision footnote incomplete or mispositioned.
If the footnote is absent for TCP/102 traffic: either the TCP counter did not fire (check
dual-gate: coverage_gaps_enabled AND ≥1 analyzer), or the PORT_102_NOTE conditional is wrong.
If the footnote is present but does not name all four protocols: the PORT_102_NOTE constant
text is missing one or more names (S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2).
If the footnote appears even without TCP/102 traffic: the footnote is unconditional (wrong).
It must be row-specific: BC-2.12.024 Invariant 2."
