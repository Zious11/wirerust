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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.13.002: --beacon Flag Does Not Exist; No C2 Beacon Analyzer Exists

## Description

`--beacon` is NOT declared in the current `cli.rs`. It was removed by PR #74 as part of
the remediation-cycle unwired-flag cleanup (Smell #3 closure). No C2 beacon analyzer
exists anywhere in the codebase. This BC documents the deliberate absence: C2 beaconing
detection is out of scope for the current release (Section 1.5 Out of Scope). Any attempt
to pass `--beacon` to wirerust results in a clap unknown-argument error.

## Preconditions

1. The CLI is invoked with `--beacon` as an argument.

## Postconditions

1. clap rejects the argument with an `UnknownArgument` error.
2. wirerust exits with a non-zero exit code.
3. No beacon detection is performed under any invocation.

## Invariants

1. `--beacon` does not appear in `Cli`, `Commands::Analyze`, or any subcommand.
2. No `C2BeaconAnalyzer` or equivalent struct exists in `src/`.
3. This is a documented out-of-scope feature (domain-debt / Section 1.5), not a bug.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | wirerust analyze --beacon test.pcap | clap error: unexpected argument '--beacon'; exit nonzero |
| EC-002 | wirerust analyze --all test.pcap (without --beacon) | All configured analyzers run; no beacon detection regardless |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Cli::try_parse_from(["wirerust", "analyze", "--beacon", "test.pcap"]) | Err (unknown argument) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --beacon argument is rejected by clap | unit: try_parse_from returns Err for --beacon |
| VP-TBD | No beacon analyzer exists in src/ | code: grep -r 'beacon\|Beacon' src/ finds no analyzer |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- this absent behavior is documented at the CLI boundary, which governs ingestion configuration |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-13 (cli.rs, C-3 -- flag absence documented) |
| Stories | S-TBD |
| Origin BC | BC-ABS-002 (pass-3 ingestion corpus, HIGH confidence absent) |

## Related BCs

- BC-2.13.001 -- related to (--threats also absent / unwired; similar pattern)
- BC-2.13.003 -- related to (--filter also absent; similar pattern)

## Architecture Anchors

- `src/cli.rs` -- no --beacon declaration (see LESSON-P1.04 comment on removed flags)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: cli.rs LESSON-P1.04 comment lists removed flags including --beacon

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes (clap parse is deterministic) |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (no-op / absent) |

## Refactoring Notes

No refactoring needed. The flag was intentionally removed. When C2 beacon detection is
implemented, a new --beacon flag will be added and this BC will be retired (see DF-030
deprecation protocol).
