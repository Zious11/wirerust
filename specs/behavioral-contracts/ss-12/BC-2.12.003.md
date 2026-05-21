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

# BC-2.12.003: Global Flag --no-color Parsed and Stored

## Description

The `--no-color` global flag is declared on the `Cli` struct with `global = true`, meaning it
can appear before or after the subcommand name. When present, `cli.no_color = true`. In
`main.rs`, the `use_color` variable is computed as `!cli.no_color && std::env::var("NO_COLOR").is_err()`,
so either `--no-color` or the `NO_COLOR` environment variable independently disables color.

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called.
2. The `--no-color` flag may or may not be present.

## Postconditions

1. `cli.no_color = true` when `--no-color` is present; `false` otherwise.
2. When `cli.no_color = true`, `use_color = false` in main.rs.
3. `use_color` is passed to `TerminalReporter { use_color, ... }`.

## Invariants

1. `--no-color` is a global clap flag; it can appear before the subcommand.
2. The `no_color` field is `bool` on `Cli` struct; it is NOT an `Option<bool>`.
3. The conjunction with `NO_COLOR` env var is at main.rs:43, not in clap parsing.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --no-color before subcommand | no_color=true |
| EC-002 | --no-color after subcommand and targets | no_color=true (global flag) |
| EC-003 | --no-color absent | no_color=false |
| EC-004 | NO_COLOR env set, --no-color absent | use_color=false (handled in main.rs, not here) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "--no-color", "analyze", "cap.pcap"] | cli.no_color=true | happy-path |
| ["wirerust", "analyze", "--no-color", "cap.pcap"] | cli.no_color=true (global) | happy-path |
| ["wirerust", "analyze", "cap.pcap"] | cli.no_color=false | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | --no-color flag sets no_color=true | unit: test_no_color_flag |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the --no-color global flag is declared on the Cli struct (cli.rs C-3) and consumed in main.rs to compute use_color before subcommand dispatch; this is CLI orchestration / entry-point wiring, not reporter rendering logic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | S-TBD |
| Origin BC | BC-CLI-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.010 -- composes with (NO_COLOR env var is the complementary mechanism)
- BC-2.11.018 -- depends on (colorization behavior flows from this flag)

## Architecture Anchors

- `src/cli.rs:44-45` -- no_color field on Cli struct
- `src/main.rs:43` -- use_color computation
- `tests/cli_tests.rs` -- test_no_color_flag

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:44-45` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: `bool` field with `#[arg(long, global = true)]`
- **assertion**: test_no_color_flag

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
