---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: DF-SIBLING-SWEEP-001 — fix stale cli.rs line anchor: output_format field 47-49 → 57-59; verified against HEAD cfe0112a — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.004: --output-format json Parses to Some(OutputFormat::Json)

## Description

The `--output-format` global flag accepts a `ValueEnum` variant: `json` or `csv`. When
`--output-format json` is given, `cli.output_format = Some(OutputFormat::Json)`. When absent,
`cli.output_format = None`. The `--json` and `--csv` global flags are SEPARATE from
`--output-format`; they provide shorthand with optional file path arguments.

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called with `--output-format <fmt>`.

## Postconditions

1. `cli.output_format = Some(OutputFormat::Json)` when `--output-format json`.
2. `cli.output_format = Some(OutputFormat::Csv)` when `--output-format csv`.
3. `cli.output_format = None` when the flag is absent.
4. An unrecognized format value causes a clap parse error.

## Invariants

1. `OutputFormat` is a `ValueEnum` with two variants: `Json` and `Csv`.
2. `output_format` is `Option<OutputFormat>` -- the Option wrapping is by clap
   when `value_enum` is used without a default_value.
3. `--output-format` is separate from `--json <FILE>` / `--csv <FILE>`. The
   `resolve_format()` function in main.rs gives `--json`/`--csv` flags higher
   precedence than `--output-format`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --output-format json | Some(OutputFormat::Json) |
| EC-002 | --output-format csv | Some(OutputFormat::Csv) |
| EC-003 | --output-format absent | None |
| EC-004 | --output-format xml (invalid) | Clap error |
| EC-005 | --json flag with --output-format csv | --json wins in resolve_format (higher precedence) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "--output-format", "json", "summary", "x.pcap"] | output_format=Some(Json) | happy-path |
| ["wirerust", "summary", "x.pcap"] | output_format=None | happy-path |
| ["wirerust", "--output-format", "xml", "summary", "x.pcap"] | Clap parse error | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | --output-format json parses to Some(Json) | unit: test_summary_subcommand (includes --output-format check) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- the --output-format flag is declared on the Cli struct and parsed at the L0 entry layer; it is an input to resolve_format in main.rs, which is CAP-12's output-channel selection responsibility, not CAP-11's rendering logic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | STORY-087 |
| Origin BC | BC-CLI-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.016 -- composes with (resolve_format uses this field to select reporter)
- BC-2.12.017 -- composes with (output routing uses the resolved format)

## Architecture Anchors

- `src/cli.rs:57-59` -- output_format field on Cli
- `src/main.rs:316-324` -- resolve_format function
- `tests/cli_tests.rs` -- test_summary_subcommand

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:57-59` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: OutputFormat ValueEnum derive
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
