---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.022: `wirerust protocols` Subcommand Dispatches to `run_protocols()` and Honors `--json` Flag

## Description

`wirerust protocols` is a new top-level subcommand (alongside `analyze` and `summary`) that
dispatches to `run_protocols()` in `src/main.rs`. The subcommand accepts three optional
filter flags (`--all`, `--supported`, `--unsupported`) and honors the global `--json` flag
for machine-readable output. The dispatch wiring adds a new `Commands::Protocols` arm to
`src/cli.rs` and a new match arm in the main dispatch block in `src/main.rs`. No existing
subcommand semantics are changed.

## Related BCs

- BC-2.18.001 — depends on (terminal output rendered by run_protocols() for non-JSON path)
- BC-2.18.002 — depends on (JSON output rendered by run_protocols() for --json path)
- BC-2.12.023 — sibling (covers the `--coverage-gaps` flag on the `analyze` subcommand; unrelated to `protocols` subcommand)

## Preconditions

1. `wirerust protocols` is invoked (with optional filter flag: `--all`, `--supported`, or `--unsupported`).
2. The global `--json` flag may or may not be set.
3. No pcap file argument is provided to `protocols`; it is a pure-catalog subcommand.

## Postconditions

1. `Commands::Protocols { filter, json }` is the clap-parsed command variant for `wirerust protocols [--all | --supported | --unsupported] [--json]`.
2. The main dispatch block in `src/main.rs` routes `Commands::Protocols` to `run_protocols(filter, json)`.
3. `run_protocols()` calls:
   - `all_protocols()` for `--all` or no filter flag,
   - `supported_protocols()` for `--supported`,
   - `unsupported_protocols()` for `--unsupported`.
4. When `json == false` (default): output is the terminal table described in BC-2.18.001.
5. When `json == true` (global `--json` flag): output is the JSON described in BC-2.18.002.
6. Exit code is 0 on success.
7. The `analyze` subcommand is NOT affected; its behavior is unchanged.

## Invariants

1. `wirerust protocols` exits with code 0 (no pcap to analyze; no error conditions expected in the normal path).
2. The filter flags `--all`, `--supported`, `--unsupported` are mutually exclusive; clap enforces this via a group or by `conflicts_with` annotations.
3. The default behavior (no filter flag) is equivalent to `--all`.
4. LESSON-P1.04 ("no unwired flags"): the `--all`, `--supported`, `--unsupported` flags are all wired to observable behavior differences in output row count and content.
5. The `--json` flag on the `protocols` subcommand uses the same global flag already present on the top-level CLI (not a new flag).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `wirerust protocols` (no flags) | Equivalent to `--all`; all ~30 entries in terminal table; exit 0 |
| EC-002 | `wirerust protocols --supported` | Only 7 supported entries; exit 0 |
| EC-003 | `wirerust protocols --unsupported` | ~23 unsupported entries; port-102 footnote; L2 entries present; exit 0 |
| EC-004 | `wirerust protocols --json` | JSON output with `"protocols"` array; exit 0 |
| EC-005 | `wirerust protocols --supported --json` | JSON array with 7 supported entries; exit 0 |
| EC-006 | `wirerust protocols --supported --unsupported` | clap error (mutually exclusive flags); non-zero exit code (clap default) |
| EC-007 | `wirerust analyze <file>` alongside protocols subcommand | analyze behavior unchanged; protocols is a new independent subcommand |
| EC-008 | `wirerust protocols <file>` (spurious positional arg) | clap error (no positional argument accepted by protocols subcommand); non-zero exit |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `wirerust protocols` | Terminal table (~30 rows); exit 0 | happy-path |
| `wirerust protocols --supported` | Terminal table (7 rows); exit 0 | filter-supported |
| `wirerust protocols --json` | JSON with `"protocols"` array; exit 0 | json-mode |
| `wirerust protocols --unsupported --json` | JSON array (~23 entries); exit 0 | filter-unsupported-json |
| `wirerust protocols --all` | Same as no-flag (all ~30 entries); exit 0 | explicit-all |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `wirerust protocols` exits 0 and produces non-empty output | integration: `test_BC_2_12_022_protocols_subcommand_exit_0` |
| — | `--json` flag produces valid JSON with `"protocols"` key | integration: `test_BC_2_12_022_protocols_json_flag` |
| — | `--supported` filter reduces output to supported-only entries | integration: `test_BC_2_12_022_protocols_supported_filter` |
| — | Mutually exclusive flags produce clap error | unit: `test_BC_2_12_022_mutually_exclusive_flags_error` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md — the `protocols` subcommand is a new CLI entry point that orchestrates the static catalog query and output rendering; it belongs to the CLI Orchestration capability as a new `Commands` variant alongside `analyze` and `summary` |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (src/cli.rs — `Commands::Protocols` variant; src/main.rs — `run_protocols()` function + dispatch arm); SS-18 (src/protocols.rs — catalog functions called by run_protocols) |
| ADR | ADR-012 Decision 3 (OQ-3 resolution: terminal + --json output modes) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/cli.rs` — new `Commands::Protocols { filter: ProtocolFilter, json: bool }` variant (or equivalent); `ProtocolFilter` enum: `{ All, Supported, Unsupported }`; `json` is the global --json flag forwarded
- `src/main.rs` — new `Commands::Protocols { filter, json }` dispatch arm calling `run_protocols(filter, json)`
- `src/main.rs` — `fn run_protocols(filter: ProtocolFilter, json: bool)` — calls appropriate catalog function and renders output

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

(None assigned yet — integration tests serve as verification; no proptest property identified for dispatch wiring alone)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | stdout write (CLI dispatch is effectful shell layer) |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` is `&'static`) |
| **Deterministic** | yes |
| **Thread safety** | yes (read-only static data; single-threaded CLI) |
| **Overall classification** | effectful (CLI dispatch + stdout write); pure (catalog lookup) |
