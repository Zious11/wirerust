---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.024.md
  - .factory/stories/STORY-153.md
  - .factory/stories/STORY-154.md
traces_to: .factory/specs/prd.md
id: "HS-131"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.05.010
  - BC-2.05.011
  - BC-2.12.024
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires two crafted pcap fixtures: (1) a pcap with valid DNS/53 UDP traffic (at least one DNS query or response on UDP/53 that dns_analyzer.can_decode() accepts — a standard DNS query pcap); (2) a pcap with TCP traffic to port 53 (DNS-over-TCP or simply a TCP flow to port 53 that wirerust has no TCP dissector for)."
canonical_value_scenario: true
canonical_spec_citation: "DNS uses UDP port 53 per RFC 1035 §4.2.1 'Server' ('Domain servers should receive 53 as the default UDP/TCP port'). DNS is a known-supported protocol in wirerust (dns_analyzer active on UDP/53). DNS traffic accepted by dns_analyzer.can_decode() must NOT appear in the gap report (it is classified, not a gap). TCP/53 is not in the wirerust catalog as TCP — DNS is UDP-only in the protocol catalog — so (Tcp, 53) → unknown (transport mismatch)."
input-hash: "2d69b79"
---

# Holdout Scenario: DNS/53 UDP Not Counted in Gap Report; TCP/53 → `unknown` (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies two DNS-related invariants using RFC 1035 §4.2.1 as the canonical
authority:

**DNS: UDP port 53 per RFC 1035 §4.2.1**
> "Domain servers should receive queries on the 53rd port of the Internet User Datagram
> Protocol (UDP)" — RFC 1035 §4.2.1 "Server" (November 1987).
> DNS is a known-supported protocol: wirerust has a DNS analyzer (`dns_analyzer.can_decode()`).
> DNS/53 UDP packets ACCEPTED by `can_decode()` must NOT be counted as unclassified gaps.
> This is the "supported-not-counted" invariant: supported protocols are not coverage gaps.

**TCP/53 (transport mismatch): state → `unknown`**
> DNS is catalogued with `transport: Transport::Udp` and `canonical_ports: &[53]`.
> A TCP observation on port 53 does not match the UDP-only catalog entry → `unknown`.
> This is the same transport-mismatch rule as TCP/47808 (BACnet/IP) in HS-129.

## Scenario

DNS is classified by `dns_analyzer.can_decode()` REGARDLESS of whether the `--dns` flag
is set. The `--dns` flag controls finding-emission (whether DNS statistics are included in
the analysis summary), NOT gap-classification. A DNS packet that `can_decode()` accepts
is always treated as "classified" and excluded from `udp_unclassified_counts`.

This means: even if the operator runs `wirerust analyze pcap.pcap --coverage-gaps` without
`--dns`, DNS/53 packets are still NOT counted as gaps. The gap counter and the DNS analyzer
gate are orthogonal.

### Case A — Valid DNS/53 UDP Packets: NOT in `udp_unclassified_counts`

1. The evaluator creates a pcap (`dns_traffic.pcap`) with valid DNS queries or responses on
   UDP/53. These must be packets that `dns_analyzer.can_decode()` accepts (standard DNS
   query format: UDP, port 53, valid DNS wire format with a query or response section).
2. The evaluator runs: `wirerust analyze dns_traffic.pcap --coverage-gaps --json`
   WITHOUT `--dns` (to confirm can_decode() is evaluated regardless of the DNS flag).
3. The tool exits 0.
4. The JSON `"coverage_gaps"."entries"` does NOT contain a UDP/53 entry.
   DNS/53 was classified by `can_decode()` → not a gap → no counter increment.
5. If any DNS traffic IS in the pcap and `can_decode()` returns true for it, the port 53
   entry must be completely absent from the gap report.

### Case B — DNS/53 Not in Gap Report Even Without `--dns` Flag

This is the same as Case A, confirming the gap-classification independence from `--dns`:
1. `wirerust analyze dns_traffic.pcap --coverage-gaps --json` (NO --dns)
2. UDP/53 entry absent from gap report.
   This confirms ADR-012 Decision 10: `dns_analyzer.can_decode()` is evaluated for gap
   classification regardless of whether DNS finding-emission is enabled.

### Case C — TCP/53 Traffic: State `unknown` (Transport Mismatch)

1. The evaluator creates a pcap (`dns_tcp.pcap`) with TCP traffic to port 53. This could be:
   - DNS-over-TCP (a valid but rare DNS transport), OR
   - Simply any TCP flow to port 53 (wirerust has no TCP/53 dissector).
2. The evaluator runs: `wirerust analyze dns_tcp.pcap --coverage-gaps --json`
   (With at least one analyzer enabled, e.g., `--http`, to satisfy the dual-gate.)
3. The tool exits 0.
4. The JSON `"coverage_gaps"."entries"` contains a TCP/53 entry:
   - `"transport": "TCP"`
   - `"port": 53`
   - `"state": "unknown"` — DNS is catalogued as `Transport::Udp`; TCP/53 does not match
     → transport mismatch → `unknown` (NOT `"known-supported"`)
5. The state MUST NOT be `"known-supported"` (which would signal a "dissector bug" —
   but DNS on TCP is simply not in the catalog as TCP).
   The state MUST NOT be `"known-unsupported"` either (DNS on UDP is supported, but this is TCP).

### Case D — Combined Pcap: UDP/53 Absent; TCP/53 Present as `unknown`

1. The evaluator creates a combined pcap with BOTH UDP/53 DNS traffic AND TCP/53 traffic.
2. The evaluator runs: `wirerust analyze combined_dns.pcap --coverage-gaps --json`
3. The JSON gap entries:
   - UDP/53 entry: ABSENT (classified by dns_analyzer)
   - TCP/53 entry: PRESENT with `state: "unknown"`
4. The UDP classification success (DNS accepted) does NOT prevent the TCP entry from
   appearing. They are counted on separate (Udp, 53) and (Tcp, 53) keys.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.010 | UDP counter NOT incremented for packets accepted by dns_analyzer.can_decode() | Cases A, B: UDP/53 absent from gap report |
| BC-2.05.010 | can_decode() evaluated regardless of enable_dns flag (ADR-012 Decision 10) | Case B: NO --dns flag but DNS still excluded |
| BC-2.05.011 | Per-port counts are exact: DNS-accepted packets produce zero (Udp,53) increment | Cases A, B: zero count for DNS/53 |
| BC-2.12.024 | Tri-state: (Tcp, 53) → unknown (DNS UDP-only in catalog; transport mismatch) | Case C: state=unknown |
| BC-2.12.024 | Transport-aware lookup: UDP/53 absent (classified); TCP/53 present as unknown | Case D: combined distinct behavior |

<!-- HIDDEN TRACEABILITY: BC-2.05.010 EC-010 (UDP/53 DNS accepted → NOT in udp_unclassified_counts);
     BC-2.05.010 EC-014 (enable_dns=false but DNS still excluded via can_decode());
     BC-2.12.024 EC-010 (TCP/53 is unknown because DNS is UDP-only in catalog);
     ADR-012 Decision 10 (dns_analyzer.can_decode() gap-classification orthogonal to --dns flag) -->

## Fixture Creation Guidance

**DNS/53 UDP fixture:**
- A standard DNS query: UDP src_port=12345, dst_port=53; DNS wire format payload.
- Minimal DNS query (12-byte header + question): QR=0, QDCOUNT=1, ANCOUNT/NSCOUNT/ARCOUNT=0,
  question: QNAME="\x00" (root), QTYPE=A (1), QCLASS=IN (1).
- Any well-formed DNS query that a standard DNS parser accepts will work.
- `dns_analyzer.can_decode()` accepts UDP datagrams with valid DNS wire format.

**TCP/53 fixture:**
- A TCP SYN to port 53 followed by FIN (or just a TCP SYN packet if the analyzer observes
  on flow close). Use: src_port=12345, dst_port=53.
- `min(src_port, dst_port)` = 53 → key `(Tcp, 53)`.

## Verification Approach

```bash
# Case A — UDP/53 DNS traffic NOT in gap report (RFC 1035 §4.2.1: DNS = UDP port 53)
wirerust analyze dns_traffic.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries[] | select(.port == 53 and .transport == "UDP")'
# Expect: no output (empty — UDP/53 classified, not a gap)

# Case B — same without --dns flag
wirerust analyze dns_traffic.pcap --coverage-gaps --json | \
  jq '[.coverage_gaps.entries[] | select(.transport == "UDP" and .port == 53)] | length'
# Expect: 0

# Case C — TCP/53 unknown (transport mismatch)
wirerust analyze dns_tcp.pcap --coverage-gaps --http --json | \
  jq '.coverage_gaps.entries[] | select(.port == 53 and .transport == "TCP")'
# Expect: {"transport": "TCP", "port": 53, ..., "state": "unknown"}
# Must NOT be "known-supported" or "known-unsupported"

# Case D — combined
wirerust analyze combined_dns.pcap --coverage-gaps --http --json | \
  jq '.coverage_gaps.entries | map(select(.port == 53))'
# Expect: only TCP/53 with state=unknown; NO UDP/53 entry
```

## Evaluation Rubric

- **DNS/53 UDP NOT in gap report** (weight: 0.40): Cases A, B: UDP/53 absent.
  CANONICAL-VALUE must-pass. If UDP/53 appears, `dns_analyzer.can_decode()` is not being
  evaluated in the gap-classification path. DNS port 53 is canonical per RFC 1035 §4.2.1.
- **TCP/53 → unknown** (weight: 0.35): Case C: state=unknown.
  CANONICAL-VALUE must-pass. DNS is UDP-only in the catalog; TCP/53 has no catalog match.
- **Combined behavior** (weight: 0.25): Case D: UDP/53 absent, TCP/53 present as unknown.

## Failure Guidance

"HOLDOUT FAIL: HS-131 — DNS/53 gap-classification error.
Case A failure (UDP/53 IN gap report): dns_analyzer.can_decode() is not being evaluated
in the UDP decode loop's gap-classification path, OR the result is not used to gate the
udp_unclassified_counts increment. See BC-2.05.010 Invariant 7 and AC-153-005 EC-006.
Case B failure (UDP/53 appears without --dns flag): the gap-classification check is gated
on the --dns flag instead of on can_decode(). The two are orthogonal per ADR-012 Decision 10.
Case C failure (TCP/53 not 'unknown'): the tri-state lookup is matching DNS's UDP catalog
entry against a TCP observation. Transport must be checked. See BC-2.12.024 EC-010."
