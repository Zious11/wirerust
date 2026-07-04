---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified:
  - "v1.1: BC-2.12.022-FWFIX-SYNC-001 / F-W68-01 — reconcile to shipped behavior: Commands::Protocols variant drops per-variant json flag; dispatch changed to run_protocols(filter, cli: &Cli) consuming cli.json: Option<Option<PathBuf>>; bare --json/--output-format json → JSON to STDOUT; --json=PATH → JSON to file via write_output pipeline; --csv/--output-format csv → explicit error + non-zero exit; EC-009/EC-010 added; PC-1/2/4/5/6 and Architecture Anchors updated. 2026-07-04"
  - "v1.2: F5-RECONCILE-COMPLETION / F-F5P2-001 — exhaustive sweep: PC-1 and PC-2 reconciled from phantom Commands::Protocols {filter} single-field form to shipped 3-bool form (all: bool, supported: bool, unsupported: bool with conflicts_with_all mutual exclusion); Architecture Anchors updated to describe shipped clap variant and dispatch arm (all, supported, unsupported, ..); ProtocolFilter derived at dispatch arm, not stored as variant field; zero residual filter/json-bool phantom patterns. 2026-07-04"
  - "v1.3: F5-RECONCILE-COMPLETION / F-F5P7-001 — VP-table test-name drift: replace phantom test_BC_2_12_022_protocols_json_file_routing with test_BC_2_12_022_json_path_writes_file; add missing row for test_BC_2_12_022_output_format_json (--output-format json → JSON to STDOUT path); replace phantom test_BC_2_12_022_protocols_csv_rejection with test_BC_2_12_022_csv_rejected; H1 title corrected from run_protocols(cli: &Cli) to run_protocols(filter: ProtocolFilter, cli: &Cli) to match shipped two-arg signature. 2026-07-04"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.022: `wirerust protocols` Subcommand Dispatches to `run_protocols(filter: ProtocolFilter, cli: &Cli)` with `--json[=PATH]` File Routing and `--csv` Rejection

## Description

`wirerust protocols` is a new top-level subcommand (alongside `analyze` and `summary`) that
dispatches to `run_protocols()` in `src/main.rs`. The subcommand accepts three optional
filter flags (`--all`, `--supported`, `--unsupported`) and routes output through the shared
`write_output` pipeline: `--json` (bare) or `--output-format json` writes JSON to STDOUT;
`--json=<PATH>` writes JSON to the file at PATH; `--csv` / `--output-format csv` produces an
explicit error with non-zero exit (no silent fallback). The `json` field on the top-level
`Cli` struct is typed as `Option<Option<PathBuf>>`. The dispatch wiring adds a new
`Commands::Protocols` arm to
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

1. `Commands::Protocols { all: bool, supported: bool, unsupported: bool }` is the clap-parsed command variant for `wirerust protocols [--all | --supported | --unsupported]`. The three flags are mutually exclusive via clap `conflicts_with_all` annotations on each field; there is no `filter` field on the variant — `ProtocolFilter` is derived at the dispatch arm in `src/main.rs`. Output-format flags (`--json[=PATH]`, `--output-format json|csv`) are carried on the top-level `Cli` struct as `cli.json: Option<Option<PathBuf>>` (the same global field shared with `analyze` and `summary`).
2. The main dispatch block in `src/main.rs` matches `Commands::Protocols { all, supported, unsupported, .. }`, derives `ProtocolFilter` (= `Supported` if `*supported`; `Unsupported` if `*unsupported`; `All` otherwise), and calls `run_protocols(filter, cli)` where `filter` is the derived `ProtocolFilter` value and `cli: &Cli` is the top-level parsed CLI struct.
3. `run_protocols()` calls:
   - `all_protocols()` for `--all` or no filter flag,
   - `supported_protocols()` for `--supported`,
   - `unsupported_protocols()` for `--unsupported`.
4. When `cli.json == None` (no `--json` and no `--output-format json`): output is the terminal table described in BC-2.18.001.
5. When `cli.json == Some(None)` (bare `--json` or `--output-format json`): JSON output per BC-2.18.002 is written to STDOUT. When `cli.json == Some(Some(path))` (`--json=<PATH>`): JSON output per BC-2.18.002 is routed through the shared `write_output` pipeline to the file at `path` (same pipeline used by `analyze` and `summary`).
6. When `--csv` or `--output-format csv` is specified: `run_protocols` emits an explicit error message to STDERR and exits with a non-zero exit code. There is no silent fallback to terminal output.
7. Exit code is 0 on success (non-CSV, non-error paths).
8. The `analyze` subcommand is NOT affected; its behavior is unchanged.

## Invariants

1. `wirerust protocols` exits with code 0 (no pcap to analyze; no error conditions expected in the normal path).
2. The filter flags `--all`, `--supported`, `--unsupported` are mutually exclusive; clap enforces this via a group or by `conflicts_with` annotations.
3. The default behavior (no filter flag) is equivalent to `--all`.
4. LESSON-P1.04 ("no unwired flags"): the `--all`, `--supported`, `--unsupported` flags are all wired to observable behavior differences in output row count and content.
5. The `--json[=PATH]` flag uses the same `cli.json: Option<Option<PathBuf>>` field already present on the top-level `Cli` (not a new field on the `Protocols` variant). `None` = terminal output; `Some(None)` = JSON to STDOUT; `Some(Some(path))` = JSON written to file via `write_output` pipeline. `--csv` / `--output-format csv` is rejected with an explicit error — there is no silent fallback.

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
| EC-009 | `wirerust protocols --json=output.json` | JSON written to `output.json` file via shared `write_output` pipeline; exit 0 |
| EC-010 | `wirerust protocols --csv` or `wirerust protocols --output-format csv` | Explicit error to STDERR; non-zero exit code; no silent fallback to terminal output |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `wirerust protocols` | Terminal table (~30 rows); exit 0 | happy-path |
| `wirerust protocols --supported` | Terminal table (7 rows); exit 0 | filter-supported |
| `wirerust protocols --json` | JSON with `"protocols"` array; exit 0 | json-mode |
| `wirerust protocols --unsupported --json` | JSON array (~23 entries); exit 0 | filter-unsupported-json |
| `wirerust protocols --all` | Same as no-flag (all ~30 entries); exit 0 | explicit-all |
| `wirerust protocols --json=out.json` | `out.json` created with valid JSON (`"protocols"` array); exit 0 | json-file-routing |
| `wirerust protocols --csv` | STDERR error message; non-zero exit | csv-rejection |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `wirerust protocols` exits 0 and produces non-empty output | integration: `test_BC_2_12_022_protocols_subcommand_exit_0` |
| — | `--json` flag produces valid JSON with `"protocols"` key | integration: `test_BC_2_12_022_protocols_json_flag` |
| — | `--supported` filter reduces output to supported-only entries | integration: `test_BC_2_12_022_protocols_supported_filter` |
| — | Mutually exclusive flags produce clap error | unit: `test_BC_2_12_022_mutually_exclusive_flags_error` |
| — | `--json=PATH` routes JSON to the specified file; exit 0 | integration: `test_BC_2_12_022_json_path_writes_file` |
| — | `--output-format json` writes JSON to STDOUT | integration: `test_BC_2_12_022_output_format_json` |
| — | `--csv` / `--output-format csv` produces explicit error + non-zero exit | integration: `test_BC_2_12_022_csv_rejected` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md — the `protocols` subcommand is a new CLI entry point that orchestrates the static catalog query and output rendering; it belongs to the CLI Orchestration capability as a new `Commands` variant alongside `analyze` and `summary` |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (src/cli.rs — `Commands::Protocols` variant; src/main.rs — `run_protocols()` function + dispatch arm); SS-18 (src/protocols.rs — catalog functions called by run_protocols) |
| ADR | ADR-012 Decision 3 (OQ-3 resolution: terminal + --json output modes) |
| Stories | STORY-152 (F3 feature-protocol-coverage — protocols CLI subcommand dispatch wiring + run_protocols() function) |

## Architecture Anchors

- `src/cli.rs` — `Commands::Protocols { all: bool, supported: bool, unsupported: bool }` variant — three bool flags with clap `conflicts_with_all` mutual exclusion; no `filter` field on the variant (no direct `ProtocolFilter` stored); no `json` field on the variant (output routing is via the top-level `Cli.json: Option<Option<PathBuf>>`); `ProtocolFilter` enum: `{ All, Supported, Unsupported }` (derived at dispatch time in `src/main.rs`)
- `src/main.rs` — `Commands::Protocols { all, supported, unsupported, .. }` dispatch arm: derives `ProtocolFilter` from the three bools (`Supported` if `*supported`; `Unsupported` if `*unsupported`; `All` otherwise), then calls `run_protocols(filter, cli)` where `cli: &Cli`
- `src/main.rs` — `fn run_protocols(pf: ProtocolFilter, cli: &Cli)` — calls appropriate catalog function; routes output via `write_output(cli, ...)` pipeline per `cli.json`; rejects `--csv` / `--output-format csv` with explicit error + non-zero exit

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
