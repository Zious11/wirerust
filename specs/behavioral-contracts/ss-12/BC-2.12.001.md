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

# BC-2.12.001: analyze Subcommand Parses Positional Targets and All Flags

## Description

The `analyze` subcommand accepts one or more positional `targets` (required) and the boolean
flags `--dns`, `--http`, `--tls`, `--mitre`, and `--all`, plus the short form `-a` for `--all`.
All flags default to false. Clap parses these into the `Commands::Analyze` variant with the
corresponding fields set. Global flags (from the `Cli` struct) are available in both subcommands.

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called with arguments starting with
   `analyze` as the subcommand name.
2. At least one positional target is provided (clap `required = true`).

## Postconditions

1. `cli.command` is `Commands::Analyze { targets, dns, http, tls, mitre, all, .. }`.
2. `targets` contains all positional path arguments in order.
3. `dns`, `http`, `tls`, `mitre`, `all` are `true` only when their respective flags are present.
4. Absent flags default to `false`.
5. Global flags (`no_color`, `output_format`, `json`, `csv`, `reassemble`, etc.) are parsed
   on the `Cli` struct, not in the subcommand variant.

## Invariants

1. `targets` is `Vec<PathBuf>` with `required = true`; clap returns an error when no targets
   are given.
2. The `--all` flag is equivalent to `--dns --http --tls` in terms of enabling the analyzers
   (handled in main.rs:57-58, not in CLI parsing).
3. `--mitre` is a separate flag from `--all` that does NOT enable extra analyzers; it only
   affects the rendering mode.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No targets given | Clap error: required argument missing |
| EC-002 | Single target, no analyzer flags | targets=[path], all flags false |
| EC-003 | --all flag | all=true, dns/http/tls remain false in struct |
| EC-004 | --dns --http --tls individually | dns=true, http=true, tls=true, all=false |
| EC-005 | --mitre flag | mitre=true; does not imply any analyzer |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "analyze", "cap.pcap"] | Commands::Analyze{targets=[cap.pcap], all false} | happy-path |
| ["wirerust", "analyze", "--dns", "--tls", "cap.pcap"] | dns=true, tls=true, http=false | happy-path |
| ["wirerust", "analyze", "--all", "cap.pcap"] | all=true, others false in struct | happy-path |
| ["wirerust", "analyze", "--mitre", "cap.pcap"] | mitre=true, all false | happy-path |
| ["wirerust", "analyze"] | Err (no targets) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | analyze subcommand parses all flags | unit: test_analyze_subcommand, test_mitre_flag_parses_on_analyze |
| — | --mitre flag defaults to false | unit: test_mitre_flag_defaults_false |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the analyze subcommand definition (argument parsing, flag defaults, required targets) is precisely the CLI orchestration concern owned by CAP-12: cli.rs declares the Commands::Analyze variant and clap parses it at the L0 entry layer |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | S-TBD |
| Origin BC | BC-CLI-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.005 -- composes with (reassembly flags are part of the CLI surface)
- BC-2.12.008 -- composes with (--all behavioral effect)
- BC-2.12.006 -- composes with (multiple positional targets)

## Architecture Anchors

- `src/cli.rs:113-139` -- Commands::Analyze variant definition
- `tests/cli_tests.rs` -- test_analyze_subcommand, test_mitre_flag_parses_on_analyze

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:113-139` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: clap derive attributes enforce required and default_value
- **assertion**: test_analyze_subcommand, test_mitre_flag_parses_on_analyze, test_mitre_flag_defaults_false

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (clap parse is pure) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
