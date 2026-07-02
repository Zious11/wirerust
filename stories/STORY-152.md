---
document_type: story
story_id: STORY-152
title: "`protocols` CLI Subcommand — CLI Dispatch Wiring + Terminal Table Renderer + JSON Output (BC-2.12.022 + BC-2.18.001 + BC-2.18.002)"
epic_id: E-21
wave: 68
points: 8
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-protocol-coverage
github_issue: feature-protocol-coverage
subsystems: [SS-12, SS-18]
target_module: main
depends_on: [STORY-151]
blocks: []
behavioral_contracts:
  - BC-2.12.022
  - BC-2.18.001
  - BC-2.18.002
verification_properties:
  - VP-041
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored (F2 convergence complete)
# DF-CANONICAL-FRAME-HOLDOUT-001: This story carries canonical-value ACs per obligation #2.
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.022.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.001.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.002.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
input-hash: "14686f3"
---

# STORY-152: `protocols` CLI Subcommand + Terminal/JSON Rendering

> **DF-CANONICAL-FRAME-HOLDOUT-001 STORY**: This story contains mandatory canonical-value ACs
> for protocol framing invariants (EtherType constants and port numbers appearing in rendered output).

## Narrative

**As a** network security analyst using wirerust,
**I want** a `wirerust protocols` subcommand that prints a table (or JSON array) of the
protocol coverage catalog — filterable by `--supported`, `--unsupported`, or `--all` —
including EtherType display for L2 protocols, a port-102 collision footnote, and `--json` output,
**so that** I can quickly determine which ICS/IT protocols wirerust can dissect versus which
gaps exist in my network environment, without running a capture.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.12.022 | v1.0 | `wirerust protocols` Subcommand Dispatches to `run_protocols()` and Honors `--json` Flag | Primary: CLI wiring in cli.rs + main.rs dispatch |
| BC-2.18.001 | v1.4 | `protocols` Subcommand Terminal Catalog Output Lists All KNOWN_PROTOCOLS Entries | Primary: terminal table renderer with filter flags, [L2] indicator, EtherType column, port-102 footnote |
| BC-2.18.002 | v1.1 | `protocols` Subcommand JSON Mode Outputs Structured Protocol Array | Primary: JSON output schema under `"protocols"` key |

## Acceptance Criteria

### AC-152-001: `Commands::Protocols { filter, json }` variant added to `src/cli.rs`
**Traces to:** BC-2.12.022 v1.0 PC-1, Postcondition 1, Invariants 2–4; ADR-012 Decision 3

```rust
/// Protocol coverage catalog subcommand
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolFilter { All, Supported, Unsupported }

// In Commands enum:
Protocols {
    /// Show all protocols (default)
    #[arg(long, conflicts_with_all = &["supported", "unsupported"])]
    all: bool,
    /// Show only supported protocols
    #[arg(long, conflicts_with_all = &["all", "unsupported"])]
    supported: bool,
    /// Show only unsupported protocols
    #[arg(long, conflicts_with_all = &["all", "supported"])]
    unsupported: bool,
}
```

Or equivalently using a clap group with `ProtocolFilter` enum. Filter flags are mutually exclusive
(clap enforces via `conflicts_with` or group). Default behavior (no filter flag) is equivalent
to `--all` (BC-2.12.022 Invariant 3). The `--json` global flag is forwarded from the top-level
CLI (not a new flag on the `protocols` subcommand).

(traces to BC-2.12.022 v1.0 PC-1, PC-1..3, Invariants 2–4)

**Red-Gate tests (integration):**
- `test_BC_2_12_022_protocols_subcommand_exit_0` — `wirerust protocols` exits 0 and produces non-empty stdout
- `test_BC_2_12_022_mutually_exclusive_flags_error` — `wirerust protocols --supported --unsupported` exits non-zero (clap error)

### AC-152-002: `Commands::Protocols` dispatches to `run_protocols(filter, json)` in `src/main.rs`
**Traces to:** BC-2.12.022 v1.0 Postconditions 2–3, Invariants 1, 5

New dispatch arm in the main match block:
```rust
Commands::Protocols { all, supported, unsupported } => {
    let filter = if supported { ProtocolFilter::Supported }
                 else if unsupported { ProtocolFilter::Unsupported }
                 else { ProtocolFilter::All };
    run_protocols(filter, args.json);
}
```

New function `run_protocols(filter: ProtocolFilter, json: bool)` in `src/main.rs` calls:
- `all_protocols()` for `ProtocolFilter::All`
- `supported_protocols()` for `ProtocolFilter::Supported`
- `unsupported_protocols()` for `ProtocolFilter::Unsupported`

Exit code 0 on success. The `analyze` subcommand is NOT affected.

(traces to BC-2.12.022 v1.0 PC-2..3, Postconditions 2–3, 7; Invariants 1, 5)

### AC-152-003: Terminal table renderer — rows, columns, filter semantics (BC-2.18.001)
**Traces to:** BC-2.18.001 v1.4 Postconditions 1–5, 7–8; Invariants 1–2, 5; ADR-012 Decision 3

The terminal output prints one row per protocol entry in the filtered set, in catalog-declaration
order. Each row contains at minimum:
- **Name** — e.g., `Modbus/TCP`, `IEC 61850 GOOSE`
- **Category** — `ICS` or `IT` (exactly two variants)
- **Transport** — `TCP`, `UDP`, or `[L2]` for `transport=LinkLayer` entries
- **Port(s)** — comma-separated u16 values (e.g., `443, 8443` for TLS), or `—` for LinkLayer entries
- **EtherType** — hex+decimal display for LinkLayer entries with non-None ethertype (e.g., `0x88B8 (35000)`); `—` for TCP/UDP entries and ARP (ARP has `ethertype: None`)
- **Supported** — `yes` / `no`

Filter semantics:
- `--all` or no flag: all 30 entries
- `--supported`: only 7 entries (those in `supported_protocols()`)
- `--unsupported`: only 23 entries (those in `unsupported_protocols()`)

(traces to BC-2.18.001 v1.4 PC-1..3, Postconditions 1–5, 7–8; Invariants 1–2, 5)

**Red-Gate tests (integration):**
- `test_BC_2_12_022_protocols_supported_filter` — `--supported` output has exactly 7 rows
- `test_BC_2_18_001_all_row_count` — `--all` output row count == `all_protocols().len()` == 30
- `test_BC_2_18_001_supported_filter` — `--supported` output matches `supported_protocols()` exactly
- `test_BC_2_18_001_l2_transport_indicator` — GOOSE row in `--unsupported` output contains `[L2]`

### AC-152-004: Port-102 footnote present when port-102 entries are in the printed set
**Traces to:** BC-2.18.001 v1.4 Postcondition 6, Invariant 3

When any port-102 entry (S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2) is present in the
printed set, the output includes a fixed port-102 collision footnote:
`"NOTE: TCP/102 hosts S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 — gap reports on port 102 cannot be attributed to a single protocol."`

When no port-102 entry is in the printed set (e.g., `--supported` output where none of the four
TCP/102 protocols are supported), no footnote appears.

(traces to BC-2.18.001 v1.4 Postcondition 6, Invariant 3)

**Red-Gate tests:**
- `test_BC_2_18_001_port102_footnote` — `wirerust protocols --unsupported` stdout contains "TCP/102"
  collision note (four protocols named: S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2)
- `test_BC_2_18_001_port102_footnote_absent_supported` — `wirerust protocols --supported` stdout
  does NOT contain port-102 footnote (none of the four TCP/102 protocols are in the supported set)

> **DF-CANONICAL-FRAME-HOLDOUT-001 — port-102 collision:**
> The four protocols sharing TCP/102 are defined by ISO/IEC standards (S7comm via Siemens RFC,
> IEC 61850 MMS via IEC 61850-8-1, ICCP/TASE.2 via IEC 60870-6). All use ISO on TCP
> (RFC 1006 / TPKT framing), port 102. The footnote text must name all four protocols.
>
> `test_BC_2_18_001_port102_footnote_names_all_four` — asserts the footnote contains each of:
> "S7comm", "S7comm-plus", "IEC 61850 MMS", "ICCP" (or "ICCP/TASE.2") in the same output.

### AC-152-005: Link-layer note for `port_detectable: false` entries
**Traces to:** BC-2.18.001 v1.4 Postcondition 7, Invariant 4

Output includes a fixed link-layer/multicast note explaining that `transport=LinkLayer` entries
(those with `port_detectable: false`) will never appear in the `CoverageGapsSummary` dynamic
gap report because the dispatcher and UDP decode loop only observe TCP/UDP traffic.

The note may be a table footer, a footnote, or an annotation in the `[L2]` transport column.
It MUST be present when any LinkLayer entries appear in the output.

(traces to BC-2.18.001 v1.4 Postcondition 7, Invariant 4, 5)

**Red-Gate test:**
- `test_BC_2_18_001_l2_note_present` — `wirerust protocols --unsupported` stdout contains a note
  about L2/LinkLayer protocols not appearing in gap reports

### AC-152-006: EtherType display for LinkLayer entries
**Traces to:** BC-2.18.001 v1.4 Postcondition 5; BC-2.18.002 v1.1 EC-003

In terminal output, LinkLayer entries with non-None `ethertype` display the EtherType as
`0xHHHH (DDDDD)` where HHHH is uppercase hex and DDDDD is decimal. ARP renders `—` in the
EtherType column (ARP has `ethertype: None`). TCP/UDP entries render `—` in the EtherType column.

> **DF-CANONICAL-FRAME-HOLDOUT-001 — EtherType display canonical values:**
>
> `test_BC_2_18_001_goose_ethertype_display` — GOOSE row in `wirerust protocols --unsupported`
> output contains `0x88B8 (35000)` (IEC 61850-8-1 §4; IEEE RA registry "IEC GOOSE").
>
> `test_BC_2_18_001_powerlink_ethertype_display` — POWERLINK row contains `0x88AB (34987)`
> (IEEE RA registry "ETHERNET Powerlink"; Wireshark ETHERTYPE_EPL_V2; ietf-ethertypes value 34987).
>
> `test_BC_2_18_001_arp_ethertype_dash` — ARP row EtherType column is `—` (ARP has ethertype: None).

(traces to BC-2.18.001 v1.4 Postcondition 5, EC-004; BC-2.18.002 v1.1 EC-003)

### AC-152-007: JSON output — `"protocols"` array schema (BC-2.18.002)
**Traces to:** BC-2.18.002 v1.1 Postconditions 1–6, Invariants 1–5; ADR-012 Decision 3

When `--json` global flag is set, `run_protocols()` outputs a single JSON object:
```json
{
  "protocols": [
    {
      "name": "Modbus/TCP",
      "category": "ICS",
      "transport": "TCP",
      "canonical_ports": [502],
      "ethertype": null,
      "port_detectable": true,
      "supported": true
    },
    {
      "name": "IEC 61850 GOOSE",
      "category": "ICS",
      "transport": "LinkLayer",
      "canonical_ports": [],
      "ethertype": 35000,
      "port_detectable": false,
      "supported": false
    }
    // ... all filtered entries
  ]
}
```

JSON schema requirements per BC-2.18.002:
- `"category"`: `"ICS"` or `"IT"` (exactly two string values; no `"L2"` value)
- `"transport"`: `"TCP"`, `"UDP"`, or `"LinkLayer"`
- `"canonical_ports"`: array of integers (empty array `[]` for LinkLayer entries)
- `"ethertype"`: integer (decimal) or `null`
- `"port_detectable"`: boolean
- `"supported"`: boolean
- Array elements in catalog-declaration order
- Output valid JSON (parseable by `jq`)

(traces to BC-2.18.002 v1.1 PC-1..3, Postconditions 1–6, Invariants 1–5)

**Red-Gate tests (integration):**
- `test_BC_2_12_022_protocols_json_flag` — `wirerust protocols --json` stdout is valid JSON with `"protocols"` array
- `test_BC_2_18_002_json_schema_valid` — `jq '.protocols | length'` on `--json --all` == 30
- `test_BC_2_18_002_l2_entries_no_ports` — all entries with `"port_detectable": false` have `"canonical_ports": []`
- `test_BC_2_18_002_supported_flag_matches_function` — `--json --supported` array length == 7

> **DF-CANONICAL-FRAME-HOLDOUT-001 — JSON canonical values:**
>
> `test_BC_2_18_002_goose_json_canonical` — GOOSE entry in `wirerust protocols --json --unsupported`
> has `"ethertype": 35000` (0x88B8; IEC 61850-8-1 §4; IEEE RA "IEC GOOSE") and
> `"transport": "LinkLayer"` and `"category": "ICS"`.
>
> `test_BC_2_18_002_bacnet_json_canonical` — BACnet/IP entry has `"transport": "UDP"`,
> `"canonical_ports": [47808]` (ASHRAE 135-2016 Annex J §J.2.1 port 0xBAC0).
>
> `test_BC_2_18_002_modbus_json_canonical` — Modbus/TCP entry has `"transport": "TCP"`,
> `"canonical_ports": [502]` (IANA/Modbus App Protocol v1.1b3 §4.3.1), `"supported": true`.

### AC-152-008: Exit code 0; `analyze` subcommand unchanged
**Traces to:** BC-2.12.022 v1.0 Postcondition 6, Invariant 7; BC-2.18.001 v1.4 Postcondition 9; BC-2.18.002 v1.1 Postcondition 5

`wirerust protocols` exits with code 0 on all filter + json combinations.
`wirerust analyze <file>` behavior is UNCHANGED.

(traces to BC-2.12.022 v1.0 PC-6, Invariant 7; BC-2.18.001 v1.4 PC-9)

**Red-Gate tests (integration):**
- `test_BC_2_12_022_protocols_subcommand_exit_0` — already listed in AC-152-001
- `test_BC_2_12_022_analyze_unaffected` — `wirerust analyze` with a test pcap produces output
  identical to pre-story behavior (no `protocols` key in analyze JSON output)

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `Commands::Protocols { ... }` variant + `ProtocolFilter` enum | `src/cli.rs` (modify) | Pure (clap types) |
| `Commands::Protocols` dispatch arm | `src/main.rs` (modify) | Effectful (CLI dispatch) |
| `run_protocols(filter, json)` function | `src/main.rs` (modify) | Effectful (stdout write) |
| Terminal table renderer (within `run_protocols`) | `src/main.rs` | Effectful (stdout) |
| JSON renderer (within `run_protocols`) | `src/main.rs` | Effectful (stdout) |
| Integration tests | `tests/integration_tests.rs` or `tests/cli_tests.rs` | Effectful (process spawning) |

SS-12 (CLI/Entry) consumes SS-18 (catalog functions). Layer: L0 Entry depends on pure-core.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-152-1 | BC-2.12.022 EC-001 | `wirerust protocols` (no flags) | Equivalent to `--all`; 30 rows; exit 0 |
| EC-152-2 | BC-2.12.022 EC-006 | `wirerust protocols --supported --unsupported` | clap error; non-zero exit |
| EC-152-3 | BC-2.12.022 EC-007 | `wirerust analyze <file>` after protocols subcommand added | Analyze behavior unchanged |
| EC-152-4 | BC-2.12.022 EC-008 | `wirerust protocols <file>` (spurious positional arg) | clap error; non-zero exit |
| EC-152-5 | BC-2.18.001 EC-001 | `--supported` filter | 7 rows only; no port-102 footnote; ARP shown with `[L2]` + `—` ports |
| EC-152-6 | BC-2.18.001 EC-002 | `--unsupported` filter | ~23 rows; 5 L2 entries with `[L2]` indicator; port-102 footnote present |
| EC-152-7 | BC-2.18.001 EC-004 | GOOSE in `--unsupported` output | category=ICS, transport=[L2], ports=—, ethertype=0x88B8 (35000), supported=no |
| EC-152-8 | BC-2.18.001 EC-005 | BACnet/IP in `--unsupported` output | transport=UDP, port=47808, supported=no |
| EC-152-9 | BC-2.18.002 EC-003 | GOOSE JSON element | `"ethertype": 35000`, `"transport": "LinkLayer"`, `"category": "ICS"` |
| EC-152-10 | BC-2.18.002 EC-007 | ARP JSON element | `"transport": "LinkLayer"`, `"ethertype": null`, `"supported": true`, `"canonical_ports": []` |

## Estimated Complexity

**Story points: 8** (cli.rs new variant with clap group; main.rs dispatch arm + run_protocols()
function; terminal table renderer with EtherType column + port-102 footnote + L2 note + filter
semantics; JSON renderer per BC-2.18.002 schema; integration test suite with canonical-value
assertions per DF-CANONICAL-FRAME-HOLDOUT-001)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC-2.12.022 (v1.0) | ~3,500 |
| BC-2.18.001 (v1.4) | ~6,000 |
| BC-2.18.002 (v1.1) | ~5,000 |
| ADR-012 (Decisions 3, 7) | ~4,000 |
| src/cli.rs (existing structure) | ~4,000 |
| src/main.rs (existing structure, run_analyze reference) | ~12,000 |
| src/protocols.rs (STORY-151 output — new file) | ~6,000 |
| tests/integration_tests.rs (existing test patterns) | ~8,000 |
| Tool outputs | ~2,000 |
| **Total estimate** | **~53,000** |

Fits within a 200k context window (~27%).

## Tasks

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   Add integration tests (new `mod story_152 { ... }` in `tests/integration_tests.rs` or `tests/cli_tests.rs`):
   - `test_BC_2_12_022_protocols_subcommand_exit_0` — `wirerust protocols` exits 0
   - `test_BC_2_12_022_mutually_exclusive_flags_error` — conflicting flags → non-zero exit
   - `test_BC_2_12_022_protocols_supported_filter` — `--supported` → 7-row count in output
   - `test_BC_2_12_022_protocols_json_flag` — `--json` → valid JSON with `"protocols"`
   - `test_BC_2_12_022_analyze_unaffected` — analyze subcommand unchanged
   - `test_BC_2_18_001_all_row_count` — `--all` row count = 30
   - `test_BC_2_18_001_supported_filter` — supported filter output
   - `test_BC_2_18_001_port102_footnote` — port-102 footnote present for unsupported
   - `test_BC_2_18_001_port102_footnote_absent_supported` — no footnote for `--supported`
   - `test_BC_2_18_001_port102_footnote_names_all_four` — footnote names all 4 protocols
   - `test_BC_2_18_001_l2_transport_indicator` — GOOSE row shows `[L2]`
   - `test_BC_2_18_001_l2_note_present` — L2/port-detectable note present
   - `test_BC_2_18_001_goose_ethertype_display` — `0x88B8 (35000)` in output (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_001_powerlink_ethertype_display` — `0x88AB (34987)` in output (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_001_arp_ethertype_dash` — ARP EtherType column is `—`
   - `test_BC_2_18_002_json_schema_valid` — jq parseable; `.protocols | length` == 30
   - `test_BC_2_18_002_l2_entries_no_ports` — all `port_detectable:false` entries have `canonical_ports: []`
   - `test_BC_2_18_002_supported_flag_matches_function` — `--json --supported` length == 7
   - `test_BC_2_18_002_goose_json_canonical` — GOOSE ethertype=35000 in JSON (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_002_bacnet_json_canonical` — BACnet/IP UDP 47808 in JSON (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_002_modbus_json_canonical` — Modbus/TCP 502 supported=true in JSON (DF-CANONICAL-FRAME-HOLDOUT-001)
   All tests MUST FAIL (no `protocols` subcommand exists yet).

2. **Add `Commands::Protocols` variant to `src/cli.rs` (AC-152-001)**
   - Define `ProtocolFilter` enum (or use clap group/exclusive args)
   - Add `Protocols { ... }` to `Commands` enum with mutual-exclusion constraints
   - Verify: `wirerust protocols --help` prints; mutual-exclusion test GREEN; binary still compiles

3. **Add dispatch arm + `run_protocols()` stub in `src/main.rs` (AC-152-002)**
   - Add `Commands::Protocols { ... }` match arm calling `run_protocols(filter, json)`
   - Add `fn run_protocols(filter: ProtocolFilter, json: bool)` with `todo!()` body
   - Verify: `wirerust protocols` compiles; exit-0 test GREEN (todo panics but let's ensure
     the dispatch arm doesn't panic before reaching todo in the no-op case)
   Actually: add minimal stub that prints empty output and returns; exit-0 will pass

4. **Implement terminal table renderer (AC-152-003 through AC-152-005)**
   - Implement row formatting: Name | Category | Transport (`[L2]` for LinkLayer) | Port(s) | EtherType | Supported
   - EtherType display: `0xHHHH (DDDDD)` for non-None ethertype entries; `—` otherwise
   - Port-102 footnote: conditional on any of the four TCP/102 protocols in the filtered set
   - Link-layer note: present when any LinkLayer entries appear
   - Filter routing: `ProtocolFilter::All` → `all_protocols()`, etc.
   - Verify: all terminal-output tests turn GREEN including canonical EtherType tests

5. **Implement JSON renderer (AC-152-007)**
   - Serialize each `KnownProtocol` to the JSON schema from BC-2.18.002 PC-3
   - Use `serde_json` (already in dependencies) or manual JSON construction
   - Verify: all JSON-output tests turn GREEN including `test_BC_2_18_002_goose_json_canonical`

6. **Verify exit code 0 and analyze subcommand unchanged (AC-152-008)**
   - Confirm `wirerust protocols` exits 0 for all filter + json combinations
   - Run full analyze integration test to confirm no regression

7. **Full regression sweep**
   - `cargo test --all-targets` — ALL tests GREEN
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — clean

8. **Micro-commit and PR** targeting `develop` (wave 68, parallel with STORY-154)

## Previous Story Intelligence

**From STORY-151 (direct predecessor):**
STORY-151 creates `src/protocols.rs` with `KNOWN_PROTOCOLS`, `SUPPORTED_PORTS`, `all_protocols()`,
`supported_protocols()`, `unsupported_protocols()`, `ProtocolCategory`, and `Transport`.
This story consumes those functions directly. Read the `Transport` enum — it has `LinkLayer` (not
just `Tcp`/`Udp`), which drives the `[L2]` display logic.

**Key STORY-151 lessons for this story:**
- `KNOWN_PROTOCOLS.len() == 30` (7 supported + 23 unsupported)
- ARP has `ethertype: None` and `canonical_ports: &[]` but IS in `supported_protocols()`
- GOOSE `ethertype: Some(35000)` (not 34992 — the corrected value from BC v1.1)
- POWERLINK `ethertype: Some(34987)` (IEEE RA assigned 0x88AB)
- Port-102 footnote condition: check if any port-102 entries appear in the filtered set

**From existing CLI patterns (STORY-086..090):**
Look at `run_analyze()` in `src/main.rs` for the pattern of calling multiple functions and
rendering output. The `run_protocols()` function is simpler (no pcap, no StreamDispatcher)
but follows the same dispatch-and-render structure.

**From JSON reporting (STORY-076):**
The `serde_json::to_string_pretty` or manual JSON construction pattern already exists in
`src/reporter/json.rs`. Prefer `serde_json` for the `"protocols"` array rather than manual
string concatenation.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-012 + BC-2.12.022/BC-2.18.001/002

1. **`Commands::Protocols` MUST be a separate variant in the `Commands` enum** — not a subcommand of `analyze` (ADR-012 Decision 3; BC-2.12.022 PC-1).
2. **Filter flags are mutually exclusive** — clap must enforce `--all`, `--supported`, `--unsupported` as conflicting; no two flags accepted simultaneously (BC-2.12.022 Invariant 2).
3. **Default (no flag) == `--all`** — no filter flag produces the same output as `--all` (BC-2.12.022 Invariant 3).
4. **`analyze` subcommand unchanged** — the `Commands::Analyze` arm MUST NOT be modified (BC-2.12.022 Invariant 7; regression baseline).
5. **JSON output `"category"` values are `"ICS"` or `"IT"` only** — no `"L2"` value (ADR-012 Decision 7; BC-2.18.002 PC-3).
6. **GOOSE EtherType display: `0x88B8 (35000)`** — not `34992` (pre-F2 erroneous value; corrected in BC-2.18.002 v1.1; DF-CANONICAL-FRAME-HOLDOUT-001).
7. **POWERLINK EtherType display: `0x88AB (34987)`** — IEEE RA assigned value (DF-CANONICAL-FRAME-HOLDOUT-001).
8. **Port-102 footnote is row-presence-triggered** — it appears IF AND ONLY IF any of S7comm, S7comm-plus, IEC 61850 MMS, or ICCP/TASE.2 is in the printed set (BC-2.18.001 Invariant 3; Postcondition 6 v1.4).
9. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL test functions in `mod story_152 { ... }` wrapper.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `clap` | (existing) | `Commands::Protocols` variant + mutual-exclusion flags |
| `serde_json` | (existing) | JSON rendering for `--json` output |

No new dependencies. This story modifies `src/cli.rs` and `src/main.rs` only (plus tests).

**Forbidden dependencies:** `run_protocols()` MUST NOT call into `StreamDispatcher`, analyzer modules,
or the reporter pipeline. It is a pure CLI → catalog → render path.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/cli.rs` | Modify | Add `Commands::Protocols { all, supported, unsupported }` variant + `ProtocolFilter` enum |
| `src/main.rs` | Modify | Add `Commands::Protocols` dispatch arm + `run_protocols(filter, json)` function |
| `tests/integration_tests.rs` | Modify | Add `mod story_152 { ... }` with integration tests per AC-152-001 through AC-152-008 |

No new source files.

## Revision History

| Version | Date | Change | Finding IDs |
|---------|------|--------|-------------|
| v1.0 | 2026-07-02 | Initial story authored for feature-protocol-coverage F3 decomposition | — |
