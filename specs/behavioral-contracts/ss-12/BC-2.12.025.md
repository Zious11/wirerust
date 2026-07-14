---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "8b69772"
---

# BC-2.12.025: `--iec104` Flag Enables IEC-104 Analysis; Included in `--all`; Default False

## Description

The CLI exposes a `--iec104` flag (`bool`, default `false`) that enables the IEC-104
passive analyzer (SS-19). When `--iec104` is set, `DispatchTarget::Iec104` is enabled
and TCP/2404 flows are dispatched to `Iec104Analyzer`. When not set, TCP/2404 flows
receive no IEC-104 analysis. The flag follows the existing opt-in pattern for protocol
analyzers (cf. `--modbus`, `--dnp3`, `--enip`). The `--all` flag implicitly sets
`--iec104` (along with all other protocol flags). The flag appears in `--help` output
and is documented in the CLI reference.

## Preconditions

1. The user invokes the CLI with `--iec104` or `--all`.

## Postconditions

1. When `--iec104` is passed: `CliArgs::iec104 == true`; `Iec104Analyzer` is instantiated and bound to `DispatchTarget::Iec104`.
2. When `--iec104` is NOT passed and `--all` is NOT passed: `CliArgs::iec104 == false`; no `Iec104Analyzer` instantiated; TCP/2404 flows are treated as unclassified.
3. When `--all` is passed: `CliArgs::iec104 == true` (implied by `--all`).
4. `--help` output includes `--iec104 Enable IEC 60870-5-104 (IEC-104) analysis [default: false]`.

## Invariants

1. **Opt-in default**: IEC-104 analysis is off by default, consistent with all other binary protocol analyzers in the CLI.
2. **`--all` inclusion**: `--all` enables every protocol analyzer including IEC-104; this is the one-flag convenience mode.
3. **Flag isolation**: `--iec104` does not affect any other analyzer's behavior; each analyzer has an independent enable flag.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--iec104` without `--all` | Only IEC-104 enabled (plus any other explicit flags) |
| EC-002 | `--all` without `--iec104` | All analyzers enabled, including IEC-104 |
| EC-003 | Neither `--iec104` nor `--all` | IEC-104 disabled; TCP/2404 → unclassified |
| EC-004 | `--iec104 --iec104` (duplicate flag) | Parsed as `true` (idempotent) |
| EC-005 | `wirerust --help` | Output contains `--iec104` with description |

## Canonical Test Vectors

| CLI invocation | CliArgs::iec104 | IEC-104 active |
|----------------|-----------------|----------------|
| `wirerust -r cap.pcap --iec104` | `true` | yes |
| `wirerust -r cap.pcap --all` | `true` | yes |
| `wirerust -r cap.pcap --modbus` | `false` | no |
| `wirerust -r cap.pcap` | `false` | no |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (unit) | Flag parse round-trip; `--all` implies `iec104=true` | Unit test in `tests/cli_args.rs` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Entry Point and Flag Parsing") per ARCH-INDEX.md §SS-12 |
| Capability Anchor Justification | CAP-12 ("CLI Entry Point and Flag Parsing") per ARCH-INDEX.md §SS-12 — the `--iec104` flag is a CLI entry-point capability that controls feature activation for the IEC-104 analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-12 (src/cli.rs, src/main.rs); ADR-013 Decision 1 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — CLI flag) |

## Related BCs

- BC-2.05.012 — depends on (Rule 8 is only active when --iec104 is set)
- BC-2.12.001..024 — composes with (existing CLI flag BCs; --iec104 follows the same pattern)

## Architecture Anchors

- `src/cli.rs` — `#[arg(long)] iec104: bool` (default false)
- `src/main.rs` — `if args.all { args.iec104 = true; }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 1`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

(unit tests in tests/cli_args.rs)
