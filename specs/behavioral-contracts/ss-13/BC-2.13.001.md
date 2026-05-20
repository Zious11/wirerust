---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/cli.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-13
capability: CAP-01
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: ["2026-05-20: corrected -- flag was removed by PR #74, not merely unwired"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.13.001: --threats Flag Does Not Exist; clap Rejects It as Unknown Argument

## Description

`--threats` is NOT declared in the current `src/cli.rs`. It was removed by PR #74 as part
of the remediation-cycle unwired-flag cleanup (LESSON-P1.04). No threats-specific analyzer
exists anywhere in the codebase. This BC documents the deliberate absence: any attempt to
pass `--threats` to wirerust results in a clap unknown-argument error. The earlier ingestion
contract (BC-ABS-001) described `--threats` as "parsed but unwired" -- that description was
accurate for the pre-PR-#74 codebase; it is wrong for the current shipped binary.

## Preconditions

1. The CLI is invoked with `--threats` as an argument (with any subcommand).

## Postconditions

1. clap rejects the argument with an `UnknownArgument` error.
2. wirerust exits with a non-zero exit code (clap default error exit).
3. No analysis is performed; no output is produced beyond the error message.

## Invariants

1. `--threats` does not appear in `Cli`, `Commands::Analyze`, or any subcommand in `src/cli.rs`.
2. No threats-specific analyzer struct exists in `src/`.
3. This is a documented out-of-scope absent behavior (Section 1.5), not a bug -- the flag was
   intentionally removed along with other unwired flags by PR #74.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | wirerust analyze --threats test.pcap | clap error: unexpected argument '--threats'; exit nonzero |
| EC-002 | wirerust --threats analyze test.pcap | clap error: unexpected argument '--threats'; exit nonzero |
| EC-003 | wirerust analyze --all test.pcap (without --threats) | All configured analyzers run normally; --threats is irrelevant |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Cli::try_parse_from(["wirerust", "analyze", "--threats", "test.pcap"]) | Err (unknown argument) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --threats argument is rejected by clap | unit: try_parse_from returns Err for --threats |
| VP-TBD | No threats-related field exists in Cli or Commands | code: grep finds no threats field declaration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- this absent behavior is documented at the CLI boundary, which governs ingestion configuration |
| L2 Domain Invariants | None |
| Architecture Module | SS-13 (cli.rs, C-3 -- flag absence documented by LESSON-P1.04 comment) |
| Stories | S-TBD |
| Origin BC | BC-ABS-001 (pass-3 ingestion corpus; updated: original body was pre-PR-#74 state) |

## Related BCs

- BC-2.13.002 -- related to (--beacon is similarly absent; same removal batch)
- BC-2.13.003 -- related to (--filter is also absent; same removal batch)

## Architecture Anchors

- `src/cli.rs` -- LESSON-P1.04 comment lists removed flags including --threats; no declaration present

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs` (lines 25-34 -- LESSON-P1.04 comment identifies --threats as removed by PR #74) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: cli.rs LESSON-P1.04 comment enumerates the five removed flags including --threats

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (clap parse only) |
| **Global state access** | none |
| **Deterministic** | yes (clap parse is deterministic) |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (no-op / absent) |

## Refactoring Notes

No refactoring needed. The flag was intentionally removed. When a threats-specific detection
subsystem is implemented, a new `--threats` flag will be added with a defined runtime effect
and this BC will be retired (see DF-030 deprecation protocol).
