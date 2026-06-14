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
  - "v1.3: DF-SIBLING-SWEEP-001 — fix stale cli.rs line anchor: targets field 116-118 → 132-134 (Commands::Analyze enum variant shifted due to Cli struct growing); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.4: F3-convergence de-pin — removed numeric line anchor for targets field; replaced with symbol anchor (drift-proof); verified live src: cli.rs Analyze.targets at :136 — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.006: Multiple Positional Targets Accepted in analyze

## Description

The `targets` field in `Commands::Analyze` and `Commands::Summary` accepts multiple
positional path arguments as a `Vec<PathBuf>`. There is no upper bound on the count. Each
target is processed sequentially by the packet loop in `run_analyze` / `run_summary`. Targets
may be file paths or directory paths (directory expansion is handled in `resolve_targets`).

## Preconditions

1. `Cli::try_parse_from()` is called with `analyze` or `summary` followed by multiple
   positional arguments.

## Postconditions

1. `targets` contains all positional path arguments in the order provided on the command line.
2. No deduplication is performed at parse time; duplicate paths are allowed.
3. Each target is a `PathBuf`; no existence validation is done at parse time.

## Invariants

1. `targets` uses `required = true` in clap; zero targets is a parse error.
2. clap does not limit `Vec<PathBuf>` length; any positive count is accepted.
3. The order of targets in the Vec matches the command-line order.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single target | targets=[path] |
| EC-002 | Two targets | targets=[path1, path2] |
| EC-003 | Same path twice | targets=[path, path] (duplicates kept) |
| EC-004 | Zero targets | Clap error: required argument missing |
| EC-005 | Target with spaces in path (quoted) | Correctly parsed as single PathBuf |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "analyze", "a.pcap", "b.pcap", "c.pcap"] | targets=[a.pcap, b.pcap, c.pcap] | happy-path |
| ["wirerust", "analyze", "single.pcap"] | targets=[single.pcap] | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Multiple targets accepted | unit: test_multiple_targets |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- accepting multiple positional targets and iterating over them in run_analyze / run_summary is core to CAP-12's per-target file expansion and packet-loop orchestration |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | STORY-086 |
| Origin BC | BC-CLI-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.001 -- composes with (positional targets are defined in the analyze subcommand)
- BC-2.12.011 -- composes with (each target may be a directory that expands to multiple files)

## Architecture Anchors

- `src/cli.rs` `Analyze.targets` field (Vec<PathBuf> with required = true in Commands::Analyze)
- `tests/cli_tests.rs` -- test_multiple_targets

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs` `Analyze.targets` field |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: Vec<PathBuf> with required = true
- **assertion**: test_multiple_targets

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
