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
traces_to: .factory/specs/prd.md
id: "HS-129"
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
fixture_note: "Requires two crafted pcap fixtures: (1) a pcap with a UDP flow to destination port 47808 (BACnet/IP port — no wirerust BACnet dissector, so it will be unclassified); (2) a pcap with a TCP flow to destination port 47808 (transport mismatch — BACnet/IP is UDP-only in catalog). These must be real pcap files with at least one packet each. The evaluator may craft them using scapy, tcpreplay, or any pcap construction tool."
canonical_value_scenario: true
canonical_spec_citation: "BACnet/IP uses UDP port 47808 (0xBAC0) per ASHRAE 135-2016 Annex J §J.2.1 'BACnet/IP': 'The UDP port number 47808 (hex BAC0) shall be used for BACnet/IP communication.' The catalog entry for BACnet/IP has transport=UDP; a TCP observation on port 47808 does not match the catalog entry (transport mismatch → unknown)."
input-hash: "79d554c"
---

# Holdout Scenario: `CoverageGapsSummary` — BACnet/IP UDP/47808 as `known-unsupported`; TCP/47808 as `unknown` (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies the tri-state classification for BACnet/IP's canonical UDP port.

**BACnet/IP: UDP port 47808 (decimal) = 0xBAC0 (hex)**
> Source: ASHRAE 135-2016, Annex J §J.2.1, "BACnet/IP":
> "The UDP port number 47808 (hex BAC0) shall be used for BACnet/IP communication."
> The catalog entry `canonical_ports: &[47808]` with `transport: Transport::Udp` means:
> - `(Udp, 47808)` → `known-unsupported` (catalog match; not supported)
> - `(Tcp, 47808)` → `unknown` (transport mismatch: BACnet/IP is UDP-only in catalog)

This scenario is authored INDEPENDENTLY of the project's implementation by consulting the
ASHRAE standard. The canonical value 47808 (0xBAC0) is hardcoded in Annex J §J.2.1 of
the BACnet standard — not derived from the project's source code.

## Scenario

When wirerust observes unclassified UDP traffic to port 47808 (a port it does not dissect —
BACnet/IP has no wirerust dissector), the CoverageGapsSummary must classify the port as
`known-unsupported` and attribute it to "BACnet/IP" by name. This tests both the UDP
counter population (BC-2.05.010) and the tri-state classifier's transport-aware catalog
lookup (BC-2.12.024).

When wirerust observes TCP traffic to port 47808, the catalog lookup finds BACnet/IP (a UDP
entry) but the transport does not match TCP — so the classification must be `unknown`, not
`known-unsupported`. This transport-aware rule prevents false attributions.

### Case A — UDP/47808 Unclassified: State `known-unsupported`, Name `BACnet/IP`

1. The evaluator creates a pcap with a UDP flow: source port = any ephemeral port, destination
   port = 47808 (BACnet/IP). No BACnet/IP dissector exists in wirerust, so the UDP decoder
   loop will reach the end of all dissectors and this packet will be counted as unclassified.
2. The evaluator runs: `wirerust analyze bacnet_udp.pcap --coverage-gaps --json`
   (With `--coverage-gaps` to enable gap detection; without any BACnet-specific flag.)
3. The tool exits 0.
4. The JSON `"coverage_gaps"."entries"` contains at least one entry:
   - `"transport": "UDP"`
   - `"port": 47808` — the integer 47808 (canonical BACnet/IP port per ASHRAE 135-2016 Annex J §J.2.1)
   - `"state": "known-unsupported"` — BACnet/IP is in the catalog as unsupported
   - `"name": "BACnet/IP"` (or similar; the name from the catalog entry)
5. The count must be at least 1 (the unclassified packet was counted).
6. The state must NOT be `"unknown"` — BACnet/IP IS in the catalog as UDP/47808.

### Case B — UDP/47808 Terminal Output: `known-unsupported` Entry Visible

1. The evaluator runs: `wirerust analyze bacnet_udp.pcap --coverage-gaps` (no --json)
2. The terminal output contains the gap entry for UDP/47808 labeled as known-unsupported
   and/or as BACnet/IP.

### Case C — TCP/47808: State `unknown` (Transport Mismatch — BACnet Is UDP-Only)

1. The evaluator creates a pcap with a TCP flow: source port = any ephemeral port, destination
   port = 47808. This TCP traffic does not match any known TCP dissector.
2. The evaluator runs: `wirerust analyze bacnet_tcp.pcap --coverage-gaps --json`
3. The tool exits 0.
4. The JSON `"coverage_gaps"."entries"` contains a TCP/47808 entry:
   - `"transport": "TCP"`
   - `"port": 47808`
   - `"state": "unknown"` — NOT `"known-unsupported"`.
     BACnet/IP is catalogued as `Transport::Udp` in KNOWN_PROTOCOLS. A TCP observation on
     port 47808 does not match the catalog entry (transport mismatch). The lookup returns
     `None` → `unknown`. (ASHRAE 135-2016 Annex J §J.2.1 specifies UDP only.)
5. There must be NO `"name"` field for this entry (unknown entries have no catalog match).

### Case D — `(Tcp, 47808)` vs `(Udp, 47808)` Distinction in Same Run

1. The evaluator constructs a combined pcap with BOTH a UDP/47808 flow and a TCP/47808 flow.
2. The evaluator runs: `wirerust analyze combined.pcap --coverage-gaps --json`
3. The JSON `"coverage_gaps"."entries"` contains TWO entries:
   - One with `"transport": "UDP", "port": 47808, "state": "known-unsupported"` (BACnet/IP)
   - One with `"transport": "TCP", "port": 47808, "state": "unknown"` (no catalog match)
4. These are distinct entries with distinct transport labels and distinct states.
   The classifier must not conflate TCP and UDP observations for the same port number.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.010 | UDP counter: `(TransportProto::Udp, 47808)` incremented for unclassified UDP/47808 | Cases A, B: counter fires |
| BC-2.12.024 | Tri-state: (Udp, 47808) → known-unsupported (BACnet/IP in catalog as UDP/47808) | Case A: state=known-unsupported (ASHRAE 135-2016 Annex J §J.2.1) |
| BC-2.12.024 | Tri-state: (Tcp, 47808) → unknown (transport mismatch; BACnet is UDP-only) | Case C: state=unknown |
| BC-2.12.024 | Transport-aware lookup: different transport → different state for same port | Case D: two distinct entries |
| BC-2.12.024 | known-unsupported entries include `"name"` field; unknown entries do not | Cases A, C: name present/absent |

<!-- HIDDEN TRACEABILITY: BC-2.12.024 EC-009 (TCP/47808 → unknown because BACnet is UDP-only in catalog);
     BC-2.05.010 EC-002 (UDP/47808 as archetypal BACnet flaggable example);
     BC-2.12.024 Invariant 4 (transport-aware lookup; no cross-transport match);
     BC-2.05.010 Invariant 3 (UDP counter keys are (TransportProto::Udp, port)) -->

## Fixture Creation Guidance

**UDP/47808 fixture (`bacnet_udp.pcap`):**
A minimal pcap with one UDP datagram:
- Link type: Ethernet (LINKTYPE_ETHERNET)
- Ethernet: src/dst MAC arbitrary; EtherType=0x0800 (IPv4)
- IPv4: src/dst IP arbitrary; protocol=17 (UDP)
- UDP: src_port=54321, dst_port=47808; payload=any (e.g., 4 zero bytes)
This pcap ensures `min(src_port, dst_port) = 47808` as the `lower_port` key used by the
UDP decode loop.

**TCP/47808 fixture (`bacnet_tcp.pcap`):**
A minimal pcap with a TCP flow closing normally:
- TCP: src_port=54321, dst_port=47808; SYN+ACK+FIN sequence or just a FIN
The flow will be classified as `DispatchTarget::None` (no TCP dissector for port 47808),
causing the per-port TCP counter to fire for `(Tcp, 47808)`.

## Verification Approach

```bash
# Case A — UDP/47808 known-unsupported (ASHRAE 135-2016 Annex J §J.2.1: UDP port 0xBAC0 = 47808)
wirerust analyze bacnet_udp.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries[] | select(.port == 47808 and .transport == "UDP")'
# Expect: {"transport": "UDP", "port": 47808, "count": >=1,
#           "state": "known-unsupported", "name": "BACnet/IP"}

# Case C — TCP/47808 unknown (transport mismatch)
wirerust analyze bacnet_tcp.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries[] | select(.port == 47808 and .transport == "TCP")'
# Expect: {"transport": "TCP", "port": 47808, "count": >=1, "state": "unknown"}
# Must NOT have "name" field; state must NOT be "known-unsupported"

# Case D — two distinct entries
wirerust analyze combined.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries | map(select(.port == 47808)) | length'
# Expect: 2 (one UDP + one TCP)
```

## Evaluation Rubric

- **UDP/47808 known-unsupported** (weight: 0.40): Case A: state="known-unsupported", name="BACnet/IP".
  CANONICAL-VALUE must-pass. Wrong state means the UDP counter or the tri-state classifier is broken.
  Canonical authority: ASHRAE 135-2016 Annex J §J.2.1 (UDP port 0xBAC0 = 47808).
- **TCP/47808 unknown** (weight: 0.35): Case C: state="unknown" for TCP transport mismatch.
  CANONICAL-VALUE must-pass. Wrong state (known-unsupported) means the transport-aware lookup
  is not checking transport — it matches port alone.
- **Two distinct entries for same port, different transport** (weight: 0.25): Case D: separate
  entries with distinct transport and distinct state.

## Failure Guidance

"HOLDOUT FAIL: HS-129 — BACnet/IP tri-state classification incorrect.
Case A failure (UDP/47808 not known-unsupported): either (a) the UDP counter did not fire
(dns_analyzer.can_decode() is incorrectly accepting UDP/47808), or (b) lookup_protocol_state
returns 'unknown' instead of 'known-unsupported'. The catalog must have BACnet/IP with
transport=Udp and canonical_ports=[47808] per ASHRAE 135-2016 Annex J §J.2.1.
Case C failure (TCP/47808 not unknown): the tri-state lookup is ignoring transport — matching
BACnet/IP on port 47808 regardless of transport. A TCP observation must NOT match a UDP-only
catalog entry. See BC-2.12.024 Invariant 4 and EC-009."
