---
document_type: holdout-scenario
level: ops
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.001.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/stories/STORY-151.md
  - .factory/stories/STORY-152.md
traces_to: .factory/specs/prd.md
id: "HS-124"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.18.001
  - BC-2.18.003
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: false
fixture_note: "No pcap fixture needed. Pure catalog command."
canonical_value_scenario: true
canonical_spec_citation: "GOOSE EtherType 0x88B8 = 35000 decimal per IEC 61850-8-1 §4 and IEEE RA EtherType registry 'IEC GOOSE'; POWERLINK EtherType 0x88AB = 34987 decimal per IEEE RA registry 'ETHERNET Powerlink' (EPSG assignment), Wireshark ETHERTYPE_EPL_V2, IETF ietf-ethertypes value 34987; EtherCAT EtherType 0x88A4 = 34980 decimal per IEEE RA registry 'EtherCAT Protocol' (EtherCAT Technology Group / Beckhoff Automation GmbH), IEC 61158-3-12, Wireshark ETHERTYPE_ETHERCAT; PROFINET EtherType 0x8892 = 34962 decimal per IEEE RA registry entry for PROFINET (Siemens AG / PROFIBUS & PROFINET International PI), IEC 61158-4-10, Wireshark ETHERTYPE_PROFINET."
input-hash: "80107d1"
---

# Holdout Scenario: `protocols` Terminal — GOOSE, POWERLINK, EtherCAT, and PROFINET-DCP EtherType Canonical Values (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies that wirerust uses the authoritative IEEE-registered EtherType
values for IEC 61850 GOOSE and Ethernet POWERLINK as they appear in the public CLI output.
These values are derived INDEPENDENTLY of the project's implementation:

**IEC 61850 GOOSE: EtherType 0x88B8 (decimal 35000)**
> Source: IEC 61850-8-1 §4 (GOOSE and GSSE EtherType assignment); IEEE Registration
> Authority EtherType registry entry "IEC GOOSE" (EtherType hex 88B8).
> Wrong value guard: `0x88BA` (35002) is Sampled Values, NOT GOOSE.
> A pre-F2 erroneous value `34992` (0x88B0) must NOT appear.

**Ethernet POWERLINK: EtherType 0x88AB (decimal 34987)**
> Source: IEEE Registration Authority EtherType registry entry "ETHERNET Powerlink" (EPSG
> Ethernet POWERLINK Standardization Group assignment). Corroborated by:
> Wireshark `epan/etypes.h` constant `ETHERTYPE_EPL_V2 = 0x88AB`;
> IETF `ietf-ethertypes` YANG module (2019-03-04) `ethernet-powerlink { value 34987 }`.
> Wrong value guard: `0x3E3F` (16063) is the obsolete POWERLINK V1 value; must NOT appear.

**EtherCAT Protocol: EtherType 0x88A4 (decimal 34980)**
> Source: IEEE Registration Authority EtherType registry entry "EtherCAT Protocol" (assigned
> to EtherCAT Technology Group / Beckhoff Automation GmbH). Corroborated by:
> Wireshark `epan/etypes.h` constant `ETHERTYPE_ETHERCAT = 0x88A4`;
> IEC 61158-3-12:2019 (EtherCAT Physical Layer and Data-Link Layer specification);
> ETG.1000.6 EtherCAT Application Layer specification §2 (Frame Format).
> Wrong value guard: `0x88A5` (34981) is an off-by-one; must NOT appear.
> `0x88A4` = 8×4096 + 8×256 + 10×16 + 4 = 32768 + 2048 + 160 + 4 = 34980 (decimal).

**PROFINET Real-Time / DCP: EtherType 0x8892 (decimal 34962)**
> Source: IEEE Registration Authority EtherType registry — EtherType 0x8892 assigned for
> PROFINET (registered by Siemens AG on behalf of PROFIBUS & PROFINET International (PI)).
> Corroborated by: Wireshark `epan/etypes.h` constant `ETHERTYPE_PROFINET = 0x8892`;
> IEC 61158-4-10 (PROFINET Data Link Layer protocol); PROFIBUS & PROFINET International
> PI specification PN-IO. This EtherType covers both PROFINET cyclic Real-Time (RT Class
> 1/2/3) frames and PROFINET DCP (Device Configuration Protocol) frames — both use 0x8892.
> Wrong value guards: `0x8893` (34963) is an off-by-one; `0x8100` (33024) is 802.1Q VLAN
> tagging, NOT PROFINET; neither must appear in the PROFINET row.
> `0x8892` = 8×4096 + 8×256 + 9×16 + 2 = 32768 + 2048 + 144 + 2 = 34962 (decimal).

## Scenario

The terminal output of `wirerust protocols --unsupported` must display the EtherType values
for IEC 61850 GOOSE and Ethernet POWERLINK using the canonical IEEE-assigned values. These
values appear in an EtherType column rendered as `0xHHHH (DDDDD)` where HHHH is uppercase
hex and DDDDD is decimal. ARP — despite being a Layer-2 protocol — has `ethertype: None`
and must show `—` in the EtherType column. LinkLayer transport entries must show `[L2]` in
the Transport column to indicate they are not port-detectable.

### Case A — GOOSE Row: EtherType Column Shows `0x88B8 (35000)`

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The row for "IEC 61850 GOOSE" (or "GOOSE") contains:
   - Transport column: `[L2]` (indicating LinkLayer transport, not port-based)
   - EtherType column: **`0x88B8 (35000)`** (IEC 61850-8-1 §4; IEEE RA "IEC GOOSE")
   - Supported column: `no`
4. The value `34992`, `0x88B0`, `0x88BA`, or `35002` must NOT appear in the GOOSE row.
   - `35002` / `0x88BA` = IEC 61850 Sampled Values (a different protocol).
   - `34992` / `0x88B0` = an erroneous value from a pre-F2 draft; NOT the correct GOOSE EtherType.

### Case B — POWERLINK Row: EtherType Column Shows `0x88AB (34987)`

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The row for "Ethernet POWERLINK" contains:
   - Transport column: `[L2]`
   - EtherType column: **`0x88AB (34987)`** (IEEE RA "ETHERNET Powerlink"; EPSG assignment;
     Wireshark `ETHERTYPE_EPL_V2`; IETF `ietf-ethertypes` value 34987)
   - Supported column: `no`
4. The value `16063`, `0x3E3F`, or `34991` must NOT appear in the POWERLINK row.
   - `0x3E3F` (16063) = obsolete POWERLINK V1 value (B&R proprietary precursor); NOT the
     current IEEE-assigned V2 value.

### Case C — ARP Row: EtherType Column Shows `—`

1. The evaluator runs: `wirerust protocols --supported` (ARP is in the supported set).
2. The tool exits 0.
3. The row for "ARP" contains:
   - EtherType column: **`—`** (ARP's `ethertype` field is `None` in the catalog; despite
     being a LinkLayer protocol, ARP detection uses `DecodedFrame::Arp`, not EtherType matching)
4. ARP is supported (Supported column: `yes`).

### Case D — `[L2]` Transport Indicator for LinkLayer Entries

1. The evaluator runs: `wirerust protocols --unsupported`
2. All five LinkLayer entries (IEC 61850 GOOSE, IEC 61850 Sampled Values, PROFINET-RT/DCP,
   EtherCAT, Ethernet POWERLINK) must show `[L2]` in the Transport column.
3. TCP/UDP entries must NOT show `[L2]` — they show `TCP` or `UDP`.

### Case E — IEC 61850 Sampled Values: Distinct EtherType 0x88BA (35002)

1. The evaluator runs: `wirerust protocols --unsupported`
2. The row for "IEC 61850 Sampled Values" contains EtherType `0x88BA (35002)` — NOT `0x88B8`.
   This guards against transposing GOOSE and SV EtherTypes. GOOSE and SV differ by exactly 2.

### Case F — EtherCAT Row: EtherType Column Shows `0x88A4 (34980)`

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The row for "EtherCAT" contains:
   - Transport column: `[L2]` (indicating LinkLayer transport, not port-based)
   - EtherType column: **`0x88A4 (34980)`** (IEEE RA "EtherCAT Protocol"; EtherCAT Technology
     Group / Beckhoff Automation GmbH; IEC 61158-3-12; ETG.1000.6 §2 Frame Format)
   - Supported column: `no`
4. The value `34981`, `0x88A5` must NOT appear in the EtherCAT row.
   - `0x88A5` (34981) is an off-by-one that has no IEEE-assigned meaning; its appearance
     indicates an incorrect EtherType constant in the catalog.

### Case G — PROFINET-RT/DCP Row: EtherType Column Shows `0x8892 (34962)`

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The row for "PROFINET-RT/DCP" (or "PROFINET") contains:
   - Transport column: `[L2]` (indicating LinkLayer transport, not port-based)
   - EtherType column: **`0x8892 (34962)`** (IEEE RA PROFINET; Siemens AG / PROFIBUS &
     PROFINET International PI; IEC 61158-4-10; Wireshark `ETHERTYPE_PROFINET`)
   - Supported column: `no`
4. The values `34963`, `0x8893`, `33024`, or `0x8100` must NOT appear in the PROFINET row.
   - `0x8893` (34963) = off-by-one; has no IEEE-assigned meaning.
   - `0x8100` (33024) = IEEE 802.1Q VLAN tag; NOT PROFINET. Its presence would indicate
     a critical catalog data corruption (a standard Ethernet framing field mistaken for
     a PROFINET EtherType).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.18.001 | EtherType column: `0xHHHH (DDDDD)` format for LinkLayer entries | Cases A, B, E |
| BC-2.18.001 | ARP EtherType column is `—` (ethertype: None) | Case C |
| BC-2.18.001 | Transport column: `[L2]` for LinkLayer entries | Case D |
| BC-2.18.003 | GOOSE ethertype: Some(0x88B8) = Some(35000) in catalog | Case A: observable via display |
| BC-2.18.003 | POWERLINK ethertype: Some(0x88AB) = Some(34987) in catalog | Case B: observable via display |
| BC-2.18.003 | ARP: `ethertype: None`, `canonical_ports: &[]`, `port_detectable: false` | Case C |
| BC-2.18.001 | EtherType column: `0x88A4 (34980)` for EtherCAT row; wrong-value guard 0x88A5 | Case F |
| BC-2.18.001 | EtherType column: `0x8892 (34962)` for PROFINET row; wrong-value guards 0x8893, 0x8100 | Case G |

<!-- HIDDEN TRACEABILITY: BC-2.18.003 Architecture Compliance Rule 6 (GOOSE=0x88B8 not 34992);
     BC-2.18.003 Architecture Compliance Rule 7 (POWERLINK=0x88AB not 0x3E3F);
     BC-2.18.001 Postcondition 5 (EtherType column format);
     BC-2.18.001 Invariant 4 ([L2] marker for port_detectable:false entries);
     EtherCAT canonical source: IEEE RA "EtherCAT Protocol" 0x88A4; IEC 61158-3-12; ETG.1000.6;
     PROFINET canonical source: IEEE RA 0x8892 (Siemens AG / PI); IEC 61158-4-10 -->

## Verification Approach

```bash
# Case A — GOOSE EtherType display (canonical: IEC 61850-8-1 §4; IEEE RA "IEC GOOSE")
wirerust protocols --unsupported | grep -i 'GOOSE'
# Expect: row containing "0x88B8 (35000)" AND "[L2]" AND "no"
# Must NOT contain: "34992", "0x88B0", "35002", "0x88BA"

# Case B — POWERLINK EtherType display (canonical: IEEE RA "ETHERNET Powerlink"; EPSG; Wireshark ETHERTYPE_EPL_V2)
wirerust protocols --unsupported | grep -i 'POWERLINK'
# Expect: row containing "0x88AB (34987)" AND "[L2]" AND "no"
# Must NOT contain: "16063", "0x3E3F", "34991"

# Case C — ARP EtherType is dash
wirerust protocols --supported | grep -i 'ARP'
# Expect: row containing "—" in EtherType column AND "yes" in Supported column

# Case D — [L2] for all LinkLayer entries
wirerust protocols --unsupported | grep '\[L2\]'
# Expect: at least 5 lines (GOOSE, SV, PROFINET, EtherCAT, POWERLINK)

# Case E — SV EtherType distinct from GOOSE
wirerust protocols --unsupported | grep -i 'Sampled Values'
# Expect: "0x88BA (35002)" — NOT "0x88B8"

# Case F — EtherCAT EtherType display (canonical: IEEE RA "EtherCAT Protocol"; IEC 61158-3-12; ETG.1000.6)
wirerust protocols --unsupported | grep -i 'EtherCAT'
# Expect: row containing "0x88A4 (34980)" AND "[L2]" AND "no"
# Must NOT contain: "34981", "0x88A5"

# Case G — PROFINET EtherType display (canonical: IEEE RA 0x8892; Siemens AG / PI; IEC 61158-4-10)
wirerust protocols --unsupported | grep -i 'PROFINET'
# Expect: row containing "0x8892 (34962)" AND "[L2]" AND "no"
# Must NOT contain: "34963", "0x8893", "33024", "0x8100"
```

## Evaluation Rubric

- **GOOSE canonical value** (weight: 0.25): Case A: `0x88B8 (35000)` exactly in GOOSE row.
  This is a CANONICAL-VALUE must-pass: wrong EtherType means catalog data is corrupted.
- **POWERLINK canonical value** (weight: 0.20): Case B: `0x88AB (34987)` in POWERLINK row.
  Wrong value (e.g., `0x3E3F`) means the obsolete V1 value was used.
- **EtherCAT canonical value** (weight: 0.20): Case F: `0x88A4 (34980)` in EtherCAT row.
  This is a CANONICAL-VALUE must-pass: IEEE RA assigns 0x88A4 to EtherCAT Technology Group.
  Wrong value `0x88A5` (34981) indicates an off-by-one catalog error.
- **PROFINET canonical value** (weight: 0.15): Case G: `0x8892 (34962)` in PROFINET row.
  This is a CANONICAL-VALUE must-pass: IEEE RA assigns 0x8892 for PROFINET (Siemens AG / PI).
  Wrong value `0x8893` (34963) is an off-by-one; `0x8100` indicates a critical catalog corruption.
- **ARP dash** (weight: 0.10): Case C: ARP EtherType is `—` (ethertype: None).
- **[L2] indicator** (weight: 0.05): Case D: all 5 LinkLayer entries show `[L2]`.
- **SV distinct from GOOSE** (weight: 0.05): Case E: SV has `0x88BA (35002)` not `0x88B8`.

## Failure Guidance

"HOLDOUT FAIL: HS-124 — EtherType canonical value incorrect.
Case A failure (GOOSE): if `34992` or `0x88B0` appears, the pre-F2 erroneous value was used
(BC-2.18.001 Architecture Compliance Rule 6 violation). The correct value is `35000` / `0x88B8`
per IEC 61850-8-1 §4. If `35002` / `0x88BA` appears in the GOOSE row, GOOSE and Sampled Values
were transposed.
Case B failure (POWERLINK): if `0x3E3F` (16063) appears, the obsolete V1 value was used.
The correct IEEE RA assigned V2 value is `34987` / `0x88AB`. See powerlink-ethertype-verification.md.
Case C failure (ARP): ARP `ethertype` field is `None`; the display must be `—` not a hex value.
Case F failure (EtherCAT): if `34981` / `0x88A5` appears, the catalog has an off-by-one error.
The correct IEEE RA value is `34980` / `0x88A4` (EtherCAT Technology Group / Beckhoff Automation
GmbH per IEEE RA registry 'EtherCAT Protocol'; IEC 61158-3-12; ETG.1000.6 §2 Frame Format).
Any other value indicates either a placeholder constant or incorrect registry lookup.
Case G failure (PROFINET): if `34963` / `0x8893` appears, the catalog has an off-by-one error.
If `33024` / `0x8100` appears, the 802.1Q VLAN EtherType was mistakenly used — this is a critical
catalog corruption. The correct IEEE RA value is `34962` / `0x8892` (Siemens AG / PROFIBUS &
PROFINET International PI; IEC 61158-4-10; Wireshark ETHERTYPE_PROFINET)."
