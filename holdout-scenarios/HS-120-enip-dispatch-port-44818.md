---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.019.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.020.md
  - .factory/stories/STORY-131.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-120"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.019
  - BC-2.17.020
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires three crafted pcaps: (1) ENIP ListIdentity frame on port 44818 (should produce T0846 when --enip enabled); (2) same frame on port 44819 (should NOT produce T0846 — wrong port); (3) --enip not passed, port 44818 (no ENIP analysis at all)."
---

# Holdout Scenario: Traffic on Port 44818 Classified to ENIP Analyzer; ENIP Disabled Without --enip/--all

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The StreamDispatcher uses a port-based classification rule to route TCP flows on port 44818
to the EtherNet/IP analyzer. This is Rule 7 in the dispatch chain (after TLS, HTTP, port 443,
port 80, Modbus port 502, DNP3 port 20000). The ENIP analyzer must also be explicitly enabled
via `--enip` or `--all`; without these flags, no ENIP analysis is performed even if traffic
on port 44818 is present.

### Case A — Port 44818 Traffic With --enip Reaches ENIP Analyzer

1. A crafted PCAP is presented: TCP flow with destination port 44818; valid ENIP ListIdentity
   frame (command=0x0063, length=0, all zeros header otherwise).
2. The user runs: `wirerust analyze enip_port_44818.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: EXACTLY ONE T0846 finding (ListIdentity emits T0846 on first frame).
   This confirms the flow was dispatched to the ENIP analyzer and processed.

### Case B — Port 44819 Traffic: No ENIP Analysis (Wrong Port)

1. A second crafted PCAP: identical ENIP ListIdentity frame bytes, but the TCP flow uses
   destination port 44819 (one above the ENIP port).
2. The user runs: `wirerust analyze enip_port_44819.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: ZERO T0846 findings (the flow is not dispatched to ENIP because
   it does not match port 44818). No ENIP analysis occurs for port 44819.

### Case C — Port 44818 Without --enip: No ENIP Analysis (Disabled by Default)

1. A third PCAP: identical to Case A (port 44818, valid ListIdentity frame).
2. The user runs: `wirerust analyze enip_port_44818.pcap --json`
   (No --enip or --all flag.)
3. The tool exits 0.
4. The evaluator confirms: ZERO T0846 findings. Without --enip, the EnipAnalyzer is not
   constructed and no ENIP analysis runs.

### Case D — --all Flag Enables ENIP (Implied by --all)

1. The user runs: `wirerust analyze enip_port_44818.pcap --all --json`
   (--all implicitly includes --enip.)
2. The tool exits 0.
3. The evaluator confirms: ONE T0846 finding (ENIP analyzer enabled via --all).

### Case E — --enip Without TCP Reassembly: Warning Emitted

1. A PCAP is presented.
2. The user runs: `wirerust analyze some.pcap --enip --json`
   but WITHOUT `--tcp-reassembly` or `--all`. (Note: if --enip implies
   `needs_reassembly` and the analyzer auto-enables TCP reassembly, this test may
   produce different behavior. The evaluator should check whether the tool warns about
   missing TCP reassembly or silently self-enables it.)
3. Per BC-2.17.020: if --enip is set without --tcp-reassembly, a warning must be emitted
   to stderr and ENIP analysis is disabled (if TCP reassembly is strictly required and not
   auto-enabled). OR: if --enip auto-enables TCP reassembly, no warning and analysis runs.
   The evaluator must determine which behavior is implemented and verify it is consistent
   with BC-2.17.020.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.019 | Rule 7 — port 44818 classified as DispatchTarget::Enip | Case A: port 44818 + --enip → T0846 |
| BC-2.17.019 | Rule 7 applies only to port 44818, not 44819 | Case B: port 44819 → no ENIP dispatch |
| BC-2.17.020 | --enip flag enables EnipAnalyzer | Cases A/D: T0846 when --enip or --all enabled |
| BC-2.17.020 | Without --enip (and not --all): no EnipAnalyzer | Case C: no --enip → zero T0846 |
| BC-2.17.020 | --all includes --enip | Case D: --all → T0846 |
| BC-2.17.020 | --enip without TCP reassembly: warning | Case E: evaluator checks warning behavior |

<!-- HIDDEN TRACEABILITY: BC-2.17.019 Invariant 1 (content-first: TLS/HTTP take priority over port rule); BC-2.17.020 Postcondition 3 (analyzer not constructed when --enip absent) -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_port_44818.pcap` — TCP flow, src port arbitrary, dst port 44818; one ENIP
   ListIdentity frame (24 bytes: command=0x0063, all other fields zero).
2. `enip_port_44819.pcap` — Identical but dst port 44819.

Note: The Case E behavior (--enip without TCP reassembly) may use an existing PCAP;
no new fixture needed — any non-ENIP PCAP with a TCP flow works.

## Verification Approach

```bash
wirerust analyze enip_port_44818.pcap --enip --json
# Expect: exit 0; 1 T0846 finding.

wirerust analyze enip_port_44819.pcap --enip --json
# Expect: exit 0; ZERO T0846 findings.

wirerust analyze enip_port_44818.pcap --json
# Expect: exit 0; ZERO T0846 findings (ENIP disabled without --enip).

wirerust analyze enip_port_44818.pcap --all --json
# Expect: exit 0; 1 T0846 finding.
```

## Evaluation Rubric

- **Port 44818 dispatched** (weight: 0.35): Case A: T0846 present when --enip + port 44818.
- **Port 44819 not dispatched** (weight: 0.20): Case B: zero T0846 for wrong port.
- **ENIP disabled without --enip** (weight: 0.25): Case C: zero T0846 without flag.
- **--all enables ENIP** (weight: 0.20): Case D: T0846 present with --all.

## Failure Guidance

"HOLDOUT FAIL: HS-120 — dispatch or flag wiring incorrect. If Case B (port 44819) produces
T0846, the port dispatch is matching on partial port numbers or the rule is too broad. If
Case C (no --enip) produces T0846, the EnipAnalyzer is constructed unconditionally instead
of only when --enip or --all is set. If Case A produces zero T0846, either the StreamDispatcher
Rule 7 is not matching port 44818, or the EnipAnalyzer is not wired to the dispatcher. See
BC-2.17.019 (Rule 7) and BC-2.17.020 (--enip flag)."
