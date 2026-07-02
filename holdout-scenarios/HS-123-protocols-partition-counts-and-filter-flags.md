---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.022.md
  - .factory/stories/STORY-151.md
  - .factory/stories/STORY-152.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-123"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.18.003
  - BC-2.18.004
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
fixture_note: "No pcap fixture needed. `wirerust protocols` is a pure-catalog command that requires no input file. The evaluator requires only a correctly built wirerust binary."
input-hash: "77d0911"
---

# Holdout Scenario: `protocols` Subcommand — Partition Counts and Filter Flag Semantics

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

`wirerust protocols` is a new top-level subcommand that queries the static protocol coverage
catalog and renders it in either terminal table or JSON format. The catalog contains exactly
30 entries: 7 supported (Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, HTTP) and 23
unsupported. The three filter flags (`--all`, `--supported`, `--unsupported`) must each
produce the correct row count and the flags must be mutually exclusive.

This scenario verifies the partition invariant (7 + 23 == 30, with no overlap, no omissions)
and the filter flag wiring — observable from the CLI alone with no source code access.

### Case A — No Filter Flag: All 30 Entries (Default == `--all`)

1. The evaluator runs: `wirerust protocols`
2. The tool exits 0.
3. The evaluator counts the data rows in the output (excluding header rows, footers, and
   footnote lines). The count MUST be exactly 30.
4. Non-zero output must appear on stdout.

### Case B — `--supported`: Exactly 7 Rows

1. The evaluator runs: `wirerust protocols --supported`
2. The tool exits 0.
3. The output contains exactly 7 protocol rows.
4. ARP must appear in the output (ARP is supported via the DecodedFrame::Arp path; it has
   no canonical port but is explicitly included in the supported set).
5. DNS must appear in the output (DNS/UDP/53 is actively dissected).
6. The following protocols must NOT appear: BACnet/IP, S7comm, IEC 61850 GOOSE, PROFINET-RT,
   EtherCAT, Ethernet POWERLINK, or IEC 61850 Sampled Values (all unsupported).

### Case C — `--unsupported`: Exactly 23 Rows

1. The evaluator runs: `wirerust protocols --unsupported`
2. The tool exits 0.
3. The output contains exactly 23 protocol rows.
4. BACnet/IP, S7comm, IEC 61850 GOOSE, and Ethernet POWERLINK must all appear.
5. Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, and HTTP must NOT appear.

### Case D — `--all` Explicit: Equivalent to No Flag

1. The evaluator runs: `wirerust protocols --all`
2. The tool exits 0.
3. The output row count is exactly 30 — identical behavior to Case A.

### Case E — Mutually Exclusive Flags: Clap Error

1. The evaluator runs: `wirerust protocols --supported --unsupported`
2. The tool exits with a NON-ZERO exit code (clap argument conflict error).
3. The error message references conflicting flags.

### Case F — Spurious Positional Argument: Clap Error

1. The evaluator runs: `wirerust protocols somefile.pcap`
2. The tool exits with a NON-ZERO exit code (clap error; `protocols` accepts no positional args).

### Case G — Partition Invariant (Row Count Cross-Check)

Using the outputs from Cases B and C:
- `--supported` row count (7) + `--unsupported` row count (23) MUST equal `--all` row count (30).
- No protocol name must appear in both `--supported` and `--unsupported` output.
  The evaluator may verify disjointness by extracting the name column from each output and
  confirming the two name sets do not intersect.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.18.003 | `supported_protocols()` returns exactly 7 entries including ARP special case | Case B: 7 rows; ARP present |
| BC-2.18.003 | `unsupported_protocols()` returns complement (23 entries) | Case C: 23 rows |
| BC-2.18.004 | Union invariant: supported + unsupported == KNOWN_PROTOCOLS (30); disjoint | Case G: 7+23==30; no overlap |
| BC-2.12.022 | `wirerust protocols` dispatches correctly; exits 0 on success | Cases A-D: exit 0 |
| BC-2.12.022 | Filter flags are mutually exclusive (clap enforces) | Case E: non-zero exit |
| BC-2.12.022 | Default (no flag) == `--all` | Cases A and D: identical count |
| BC-2.12.022 | No positional argument accepted | Case F: non-zero exit |

<!-- HIDDEN TRACEABILITY: BC-2.18.003 Invariant 3 (ARP special case via p.name=="ARP", not port intersection);
     BC-2.18.003 Invariant 4 (unsupported derived as complement, not hand-maintained list);
     BC-2.18.004 Postconditions 1-5 (30 total, 7 supported, 23 unsupported, union==30, disjoint);
     BC-2.12.022 Invariant 2 (mutual exclusion); BC-2.12.022 Invariant 3 (no-flag == --all) -->

## Verification Approach

```bash
# Case A — no flags
wirerust protocols | grep -v '^[[:space:]]*$' | grep -v '^#\|^\-\|^Name\|^NOTE\|^Layer\|^Dynamic\|^Consult' | wc -l
# Expect: 30 (approximate; header/footer lines excluded)

# Case B — supported
wirerust protocols --supported | grep -c 'yes'
# Expect: 7 (counting rows where Supported column == yes)

wirerust protocols --supported | grep -i 'ARP'
# Expect: at least one line

wirerust protocols --supported | grep -i 'DNS'
# Expect: at least one line

wirerust protocols --supported | grep -i 'BACnet'
# Expect: zero lines (BACnet is unsupported)

# Case C — unsupported
wirerust protocols --unsupported | grep -c 'no'
# Expect: 23 (approximate; counting rows where Supported column == no)

# Case D — explicit --all
wirerust protocols --all | wc -l
# Expect: same line count as `wirerust protocols`

# Case E — conflict
wirerust protocols --supported --unsupported; echo "exit: $?"
# Expect: non-zero exit code

# Case G — JSON cross-check (most precise)
wirerust protocols --json | jq '.protocols | length'
# Expect: 30

wirerust protocols --json --supported | jq '.protocols | length'
# Expect: 7

wirerust protocols --json --unsupported | jq '.protocols | length'
# Expect: 23
```

## Evaluation Rubric

- **30-entry total** (weight: 0.25): Case A/D: `--all` and no-flag produce exactly 30 entries.
- **7-entry supported set** (weight: 0.25): Case B: exactly 7 rows; ARP and DNS present; BACnet/GOOSE absent.
- **23-entry unsupported set** (weight: 0.20): Case C: exactly 23 rows; complement of Case B.
- **Partition invariant** (weight: 0.15): Case G: 7+23==30; disjoint name sets.
- **Flag gating** (weight: 0.15): Cases E and F: clap error on bad invocations.

## Failure Guidance

"HOLDOUT FAIL: HS-123 — partition or filter wiring incorrect.
If Case A produces ≠ 30 rows: `KNOWN_PROTOCOLS` has wrong entry count (expected 30 = 7+23).
Check that all entries are declared and none were accidentally omitted or duplicated.
If Case B produces ≠ 7 rows: `supported_protocols()` filter is wrong. ARP must be included
via the explicit `p.name == 'ARP'` special case (not port intersection — ARP has no ports).
If Case G shows overlap (a name in both supported and unsupported): `unsupported_protocols()`
is not derived as the complement of `supported_protocols()`; check for a hand-maintained list.
See BC-2.18.003 (supported/unsupported logic) and BC-2.18.004 (partition invariant)."
