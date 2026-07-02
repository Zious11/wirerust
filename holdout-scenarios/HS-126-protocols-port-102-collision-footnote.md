---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.001.md
  - .factory/stories/STORY-152.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-126"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.18.001
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
canonical_spec_citation: "TCP port 102 shared by S7comm (Siemens RFC), S7comm-plus (Siemens S7-400/S7-1500), IEC 61850 MMS (IEC 61850-8-1 via ISO-on-TCP/TPKT), and ICCP/TASE.2 (IEC 60870-6 via RFC 1006/TPKT). All four use ISO-on-TCP (RFC 1006) framing on port 102."
input-hash: "8f890ab"
---

# Holdout Scenario: `protocols` Terminal — Port-102 Collision Footnote Names All Four Protocols (DF-CANONICAL-FRAME-HOLDOUT-001)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Canonical Value Obligation (DF-CANONICAL-FRAME-HOLDOUT-001)

This scenario verifies the port-102 collision footnote: four distinct industrial protocols
all share TCP port 102 via ISO-on-TCP (RFC 1006 / TPKT) framing, making gap reports on
port 102 unattributable to a single protocol.

**The four protocols sharing TCP/102:**
1. **S7comm** — Siemens S7 communication protocol for S7-300/S7-400 series PLCs (Siemens
   proprietary; documented in reverse-engineering research and Wireshark dissector)
2. **S7comm-plus** — Siemens S7 communication for S7-1200/S7-1500 series (updated S7comm;
   documented via Wireshark and ICS security research)
3. **IEC 61850 MMS (Manufacturing Message Specification)** — defined in IEC 61850-8-1,
   uses ISO-on-TCP (RFC 1006) on port 102
4. **ICCP/TASE.2** — Inter-Control Center Communications Protocol, IEC 60870-6 (TASE.2),
   also uses ISO-on-TCP (RFC 1006) on port 102

> Source: RFC 1006 §5 ("ISO Transport Service on top of the TCP") — the foundational
> standard for ISO-on-TCP; Wireshark protocol dissector registrations for all four protocols
> on port 102; IEC 61850-8-1 §4 for MMS; IEC 60870-6 for ICCP/TASE.2.
> Wrong omission guard: all FOUR protocol names must appear in the footnote.

## Scenario

When any of the four TCP/102 protocols appears in the catalog output, a footnote must warn
the operator that the port-102 entry in a gap report cannot be attributed to a single protocol.
The footnote is row-presence-triggered: it appears only when at least one TCP/102 entry is
in the filtered output set, and it names all four protocols.

When the `--supported` filter is used, none of the four TCP/102 protocols are in the
supported set (none have wirerust dissectors), so no footnote appears.

### Case A — `--unsupported` Output Contains Port-102 Collision Footnote

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The output contains a footnote or note referencing TCP/102 sharing.
4. The footnote MUST name ALL FOUR protocols:
   - "S7comm" (or "S7 comm")
   - "S7comm-plus" (or "S7comm-Plus", "S7 comm-plus")
   - "IEC 61850 MMS" (or "MMS")
   - "ICCP" (or "ICCP/TASE.2", "TASE.2")
5. The footnote text references TCP/102 (e.g., "TCP/102" or "port 102").

### Case B — `--supported` Output Has No Port-102 Footnote

1. The evaluator runs: `wirerust protocols --supported`
2. The tool exits 0.
3. The output does NOT contain a port-102 collision footnote.
   None of S7comm, S7comm-plus, IEC 61850 MMS, or ICCP/TASE.2 is in the supported set,
   so the conditional footnote trigger does not fire.

### Case C — Four TCP/102 Catalog Entries Appear in `--unsupported` Output

1. The evaluator runs: `wirerust protocols --unsupported`
2. The output contains separate rows for S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2.
3. All four rows show port 102 in the Port(s) column.
4. All four rows show TCP in the Transport column (or `TCP` in text).
5. All four rows show `no` in the Supported column.

### Case D — No-Filter Output Also Contains the Footnote

1. The evaluator runs: `wirerust protocols`  (default == --all)
2. The output contains the port-102 footnote (all four unsupported TCP/102 protocols are
   included in the `--all` output set).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.18.001 | Port-102 footnote present when any TCP/102 entry in filtered set | Cases A, D |
| BC-2.18.001 | Port-102 footnote names all four protocols (S7comm, S7comm-plus, MMS, ICCP) | Case A: all four names present |
| BC-2.18.001 | Port-102 footnote absent when no TCP/102 entry in set (--supported) | Case B: no footnote |
| BC-2.18.001 | Four TCP/102 catalog rows present in unsupported output | Case C: 4 rows, port 102 each |

<!-- HIDDEN TRACEABILITY: BC-2.18.001 Postcondition 6 (port-102 footnote conditional trigger);
     BC-2.18.001 Invariant 3 (footnote if and only if TCP/102 entries in output);
     BC-2.18.003 EC-007 (all four port-102 entries in unsupported set) -->

## Verification Approach

```bash
# Case A — footnote present and names all four protocols (RFC 1006 / ISO-on-TCP / TPKT framing)
wirerust protocols --unsupported | grep -i 'TCP.*102\|port.*102\|102.*TCP'
# Expect: at least one line with a footnote mentioning TCP/102

wirerust protocols --unsupported | grep -i 'S7comm'
# Expect: at least one footnote/note line (in addition to the S7comm data row)

wirerust protocols --unsupported | grep -i 'S7comm-plus\|S7comm.*plus'
# Expect: at least one mention in footnote

wirerust protocols --unsupported | grep -i 'MMS\|IEC 61850 MMS'
# Expect: at least one mention in footnote

wirerust protocols --unsupported | grep -i 'ICCP\|TASE'
# Expect: at least one mention in footnote

# Case B — no footnote for --supported
wirerust protocols --supported | grep -i 'TCP.*102\|port.*102\|102.*TCP'
# Expect: zero lines (no TCP/102 footnote)

# Case C — four catalog rows, each with port 102
wirerust protocols --unsupported | grep '102'
# Expect: at least 4 data rows with port 102 (S7comm, S7comm-plus, MMS, ICCP/TASE.2)
# plus the footnote line
```

## Evaluation Rubric

- **All four protocols named** (weight: 0.50): Case A: footnote contains S7comm, S7comm-plus,
  IEC 61850 MMS, and ICCP/TASE.2. CANONICAL-VALUE must-pass. Missing even one protocol name
  is a specification violation.
- **Footnote absence for --supported** (weight: 0.25): Case B: no port-102 footnote when TCP/102
  protocols not in the filtered set.
- **Four catalog rows with port 102** (weight: 0.25): Case C: all four unsupported TCP/102
  entries appear with port 102 in their rows.

## Failure Guidance

"HOLDOUT FAIL: HS-126 — port-102 collision footnote incomplete or absent.
Case A: if the footnote is present but does not name all four protocols, the footnote text
constant is missing one or more names. All four must appear: S7comm, S7comm-plus, IEC 61850 MMS,
ICCP/TASE.2 — all share TCP/102 via ISO-on-TCP (RFC 1006/TPKT).
Case B: if the footnote appears for --supported output, the trigger condition is wrong —
footnote must be conditional on TCP/102 entries being present in the CURRENT filtered set.
See BC-2.18.001 Postcondition 6 and Invariant 3."
