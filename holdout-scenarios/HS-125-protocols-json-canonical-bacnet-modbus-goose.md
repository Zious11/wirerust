---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.002.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.022.md
  - .factory/stories/STORY-151.md
  - .factory/stories/STORY-152.md
traces_to: .factory/specs/prd.md
id: "HS-125"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.18.002
  - BC-2.12.022
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: false
fixture_note: "No pcap fixture needed. Pure catalog command. jq must be available for JSON assertions."
canonical_value_scenario: true
canonical_spec_citation: "BACnet/IP UDP port 47808 (0xBAC0) per ASHRAE 135-2016 Annex J §J.2.1; Modbus/TCP port 502 per IANA registry and Modbus Application Protocol Specification v1.1b3 §4.3.1; GOOSE EtherType 35000 (0x88B8) per IEC 61850-8-1 §4 and IEEE RA registry."
input-hash: "d786aa9"
---

# Holdout Scenario: `protocols --json` — Canonical Values for BACnet/IP, Modbus/TCP, and GOOSE (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies that `wirerust protocols --json` emits the authoritative protocol
framing values in its JSON output. These values are derived from primary protocol specifications
independently of the project's source code:

**BACnet/IP: UDP port 47808 (0xBAC0)**
> Source: ASHRAE 135-2016, Annex J §J.2.1, "BACnet/IP":
> "The UDP port number 47808 (hex BAC0) shall be used for BACnet/IP communication."
> Wrong value guard: port 47809 or a TCP entry for BACnet/IP.

**Modbus/TCP: TCP port 502**
> Source: IANA service registry (port 502/tcp = "mbap"); Modbus Application Protocol
> Specification v1.1b3 §4.3.1: "Well-Known TCP Port 0+502".
> Wrong value guard: port 503 or a UDP entry for Modbus.

**IEC 61850 GOOSE: EtherType 35000 decimal (0x88B8 hex)**
> Source: IEC 61850-8-1 §4; IEEE RA EtherType registry "IEC GOOSE".
> In JSON, ethertype MUST be the integer 35000 (decimal), NOT a hex string "0x88B8".

## Scenario

When `wirerust protocols --json` is invoked, the output is a single JSON object with a
`"protocols"` array. Each entry is a JSON object conforming to the BC-2.18.002 schema:
`{ "name", "category", "transport", "canonical_ports", "ethertype", "port_detectable", "supported" }`.

### Case A — JSON Output Is Valid and Has `"protocols"` Array with 30 Entries

1. The evaluator runs: `wirerust protocols --json`
2. The tool exits 0.
3. stdout is valid JSON (parseable by `jq` without error).
4. The top-level object has a `"protocols"` key.
5. `jq '.protocols | length'` == 30.

### Case B — BACnet/IP Entry: `"transport": "UDP"`, `"canonical_ports": [47808]`, `"supported": false` (ASHRAE 135-2016 Annex J §J.2.1)

1. The evaluator runs: `wirerust protocols --json --unsupported`
2. `jq '.protocols[] | select(.name | test("BACnet"; "i"))'` extracts the BACnet/IP entry.
3. The entry MUST have:
   - `"transport": "UDP"` — BACnet/IP uses UDP exclusively (ASHRAE 135-2016 Annex J §J.2.1)
   - `"canonical_ports": [47808]` — UDP port 47808 (0xBAC0 = 47808 decimal; ASHRAE §J.2.1)
   - `"port_detectable": true` — BACnet/IP has a known UDP port
   - `"supported": false` — wirerust has no BACnet/IP dissector
   - `"ethertype": null` — BACnet/IP is a UDP protocol, not a LinkLayer protocol
4. The port must be the INTEGER `47808`, not a string.
5. The transport must be `"UDP"`, NOT `"TCP"` (BACnet/IP does not use TCP by default).

### Case C — Modbus/TCP Entry: `"transport": "TCP"`, `"canonical_ports": [502]`, `"supported": true` (Modbus App Protocol v1.1b3 §4.3.1)

1. The evaluator runs: `wirerust protocols --json --supported`
2. `jq '.protocols[] | select(.name | test("Modbus"; "i"))'` extracts the Modbus/TCP entry.
3. The entry MUST have:
   - `"transport": "TCP"` — Modbus/TCP uses TCP (IANA port 502/tcp; Modbus App Protocol v1.1b3 §4.3.1)
   - `"canonical_ports": [502]` — TCP port 502 (IANA "mbap")
   - `"supported": true` — wirerust has a Modbus dissector
   - `"port_detectable": true`
   - `"ethertype": null`
4. The port must be the INTEGER `502`, not a string.

### Case D — GOOSE Entry: `"ethertype": 35000`, `"transport": "LinkLayer"`, `"category": "ICS"` (IEC 61850-8-1 §4)

1. The evaluator runs: `wirerust protocols --json --unsupported`
2. `jq '.protocols[] | select(.name | test("GOOSE"; "i"))'` extracts the IEC 61850 GOOSE entry.
3. The entry MUST have:
   - `"ethertype": 35000` — decimal integer (NOT a string; NOT hex "0x88B8")
     Value: 0x88B8 = 35000 decimal per IEC 61850-8-1 §4 and IEEE RA "IEC GOOSE"
   - `"transport": "LinkLayer"` — GOOSE is a Layer-2 protocol (NOT TCP or UDP)
   - `"category": "ICS"` — GOOSE is an ICS protocol (NOT "IT"; NOT "L2")
   - `"canonical_ports": []` — empty array (no TCP/UDP port)
   - `"port_detectable": false`
   - `"supported": false`
4. The ethertype must NOT be `34992` (pre-F2 erroneous value) or `35002` (Sampled Values).
5. The category must NOT be `"L2"` — there is no L2 category variant. L2-ness is expressed
   via `"transport": "LinkLayer"` and `"port_detectable": false`.

### Case E — `--supported` JSON: Array Length 7

1. The evaluator runs: `wirerust protocols --json --supported`
2. `jq '.protocols | length'` == 7.

### Case F — All `"port_detectable": false` Entries Have `"canonical_ports": []`

1. The evaluator runs: `wirerust protocols --json`
2. `jq '.protocols[] | select(.port_detectable == false) | .canonical_ports | length'`
   for every such entry must equal 0 (empty array).
3. No entry with `"port_detectable": false` may have a non-empty `"canonical_ports"` array.

### Case G — `"category"` Values Are Only `"ICS"` or `"IT"`

1. The evaluator runs: `wirerust protocols --json`
2. `jq '.protocols[].category' | sort -u` must produce only `"ICS"` and `"IT"`.
3. The value `"L2"` must NOT appear in any entry's category field.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.18.002 | JSON valid; `"protocols"` array; 30 entries | Case A |
| BC-2.18.002 | BACnet/IP: UDP transport, port 47808 | Case B (ASHRAE 135-2016 Annex J §J.2.1) |
| BC-2.18.002 | Modbus/TCP: TCP transport, port 502, supported=true | Case C (Modbus App Protocol v1.1b3 §4.3.1) |
| BC-2.18.002 | GOOSE: ethertype=35000 (decimal), transport=LinkLayer, category=ICS | Case D (IEC 61850-8-1 §4) |
| BC-2.18.002 | `--supported` JSON length == 7 | Case E |
| BC-2.18.002 | All port_detectable:false entries have canonical_ports:[] | Case F |
| BC-2.18.002 | category is only "ICS" or "IT" (never "L2") | Case G |
| BC-2.12.022 | --json flag produces valid JSON with "protocols" key | Cases A-G |

<!-- HIDDEN TRACEABILITY: BC-2.18.002 Postconditions 1-6 (schema); BC-2.18.002 Invariant 2 (category ICS/IT only);
     BC-2.18.003 Invariant 3 (ARP special case); ADR-012 Decision 7 (no L2 category variant) -->

## Verification Approach

```bash
# Case A — valid JSON, 30 entries
wirerust protocols --json | jq '.protocols | length'
# Expect: 30

# Case B — BACnet/IP canonical (ASHRAE 135-2016 Annex J §J.2.1: UDP port 0xBAC0 = 47808)
wirerust protocols --json --unsupported | jq '.protocols[] | select(.name | test("BACnet"; "i"))'
# Expect: {"name": "BACnet/IP", "category": "ICS", "transport": "UDP",
#           "canonical_ports": [47808], "ethertype": null, "port_detectable": true, "supported": false}

# Case C — Modbus canonical (Modbus App Protocol v1.1b3 §4.3.1: TCP port 502)
wirerust protocols --json --supported | jq '.protocols[] | select(.name | test("Modbus"; "i"))'
# Expect: {"name": "Modbus/TCP", ..., "transport": "TCP", "canonical_ports": [502], "supported": true}

# Case D — GOOSE canonical (IEC 61850-8-1 §4: EtherType 0x88B8 = 35000 decimal)
wirerust protocols --json --unsupported | jq '.protocols[] | select(.name | test("GOOSE"; "i")) | .ethertype'
# Expect: 35000 (integer, not string)

wirerust protocols --json --unsupported | jq '.protocols[] | select(.name | test("GOOSE"; "i")) | .transport'
# Expect: "LinkLayer"

# Case G — no "L2" category
wirerust protocols --json | jq '.protocols[].category' | sort -u
# Expect: only "ICS" and "IT"
```

## Evaluation Rubric

- **BACnet/IP canonical (UDP/47808)** (weight: 0.30): Case B: transport="UDP", canonical_ports=[47808].
  CANONICAL-VALUE must-pass. Wrong transport or port is a catalog data defect.
- **GOOSE canonical (ethertype=35000)** (weight: 0.25): Case D: ethertype=35000 integer in JSON.
  CANONICAL-VALUE must-pass. Wrong value (34992 or 35002) is a catalog data defect.
- **JSON structure and array length** (weight: 0.20): Cases A and E: valid JSON, 30 total, 7 supported.
- **Modbus canonical (TCP/502)** (weight: 0.15): Case C: transport="TCP", canonical_ports=[502], supported=true.
- **Invariants** (weight: 0.10): Cases F and G: port_detectable:false → empty ports; category ∈ {ICS, IT}.

## Failure Guidance

"HOLDOUT FAIL: HS-125 — JSON canonical value incorrect.
Case B failure: BACnet/IP must have transport='UDP' and canonical_ports=[47808] per ASHRAE 135-2016
Annex J §J.2.1. If transport='TCP', the catalog entry has the wrong transport.
Case D failure: GOOSE ethertype must be integer 35000 in JSON (not string '0x88B8', not 34992).
The value 34992 (0x88B0) is a pre-F2 erroneous value; the correct IEC 61850-8-1 §4 value is 35000.
Case G failure: 'L2' category value means ADR-012 Decision 7 was violated. L2-ness is expressed
via transport='LinkLayer', not a third category variant. See BC-2.18.002 Invariant 2."
