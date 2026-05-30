---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/cli.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.002: summary Subcommand Parses Targets and --hosts Flag

## Description

The `summary` subcommand accepts one or more positional `targets` (required) and the boolean
`--hosts` flag. The `--hosts` flag, when present, enables per-host breakdown rendering in
`TerminalReporter` (LESSON-P1.03). The `--services` flag that previously existed has been
removed. Global flags from the `Cli` struct apply to this subcommand as well.

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called with `summary` as the subcommand.
2. At least one positional target is provided.

## Postconditions

1. `cli.command` is `Commands::Summary { targets, hosts }`.
2. `targets` contains all positional path arguments.
3. `hosts = true` only when `--hosts` is present; defaults to `false`.
4. The subcommand has no `--services` flag (removed in the LESSON-P1.04 cleanup).

## Invariants

1. `targets` is `Vec<PathBuf>` with `required = true`.
2. `--hosts` is a boolean flag with no value argument.
3. When `hosts = true`, `run_summary` passes `show_hosts_breakdown = true` to
   `TerminalReporter`; when `hosts = false`, `show_hosts_breakdown = false`.
4. The JSON reporter always includes `unique_hosts` in its output regardless of `--hosts`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --hosts flag absent | hosts=false in Commands::Summary |
| EC-002 | --hosts flag present | hosts=true |
| EC-003 | No targets | Clap error: required argument missing |
| EC-004 | --services flag (removed) | Clap error: unexpected argument |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "summary", "cap.pcap"] | Commands::Summary{targets=[cap.pcap], hosts=false} | happy-path |
| ["wirerust", "summary", "--hosts", "cap.pcap"] | hosts=true | happy-path |
| ["wirerust", "summary"] | Err (no targets) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | summary subcommand parses targets and --hosts | unit: test_summary_subcommand |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- the summary subcommand declaration (Commands::Summary variant, required targets, --hosts flag) is CLI orchestration owned by CAP-12; it is the lighter entry-point path in main.rs that runs run_summary without stream analyzers |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | STORY-086 |
| Origin BC | BC-CLI-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.001 -- related to (analyze is the sibling subcommand)
- BC-2.11.019 -- related to (HOSTS section in terminal output controlled by --hosts)

## Architecture Anchors

- `src/cli.rs:141-155` -- Commands::Summary variant definition
- `tests/cli_tests.rs` -- test_summary_subcommand

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:141-155` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: clap derive attributes
- **assertion**: test_summary_subcommand

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
