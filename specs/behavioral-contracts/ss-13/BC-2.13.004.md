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

# BC-2.13.004: --verbose Flag Does Not Exist; No Verbose Logging Mode

## Description

`--verbose` is NOT declared in the current `cli.rs`. It was removed by PR #74 as part of
the remediation-cycle unwired-flag cleanup (LESSON-P1.04). No verbose logging, debug
output, or log-level control is implemented. wirerust produces a fixed output structure
(terminal / JSON / CSV depending on --output-format) with no verbosity knob. Passing
`--verbose` or `-v` causes a clap unknown-argument error.

## Preconditions

1. The CLI is invoked with `--verbose` or `-v`.

## Postconditions

1. clap rejects the argument with an `UnknownArgument` error.
2. wirerust exits with a non-zero exit code.
3. Under any invocation without --verbose: output level is fixed; no additional debug
   information is emitted to stdout or stderr (progress bars go to stderr via indicatif,
   but this is always-on, not controlled by a verbosity flag).

## Invariants

1. `--verbose` and `-v` are not declared in the CLI surface.
2. No log-level control exists; there is no log crate integration or env-logger dependency.
3. The only stderr output is the indicatif progress bar (always-on) and any eprintln!
   warnings from one-shot AtomicBool tripwires (ADR 0004).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | wirerust analyze --verbose test.pcap | clap error; exit nonzero |
| EC-002 | wirerust analyze -v test.pcap | clap error (short -v not declared) |
| EC-003 | wirerust analyze test.pcap (no verbose flag) | Fixed output; no extra verbosity |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Cli::try_parse_from(["wirerust", "analyze", "--verbose", "test.pcap"]) | Err (unknown argument) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --verbose argument is rejected by clap | unit: try_parse_from returns Err for --verbose |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- output verbosity is a CLI/entry-point concern; its absence is documented here as a boundary contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-13 (cli.rs, C-3 -- flag absence) |
| Stories | S-TBD |
| Origin BC | BC-ABS-010 (pass-3 ingestion corpus, HIGH confidence absent) |

## Related BCs

- BC-2.13.001 -- related to (same absent-flag pattern)
- BC-2.13.002 -- related to (--beacon also absent)
- BC-2.13.003 -- related to (--filter also absent)

## Architecture Anchors

- `src/cli.rs` -- no --verbose or -v declaration
- LESSON-P1.04 comment in cli.rs documenting removed flags

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: cli.rs LESSON-P1.04 comment; Smell #3 closed by #74

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (absent/no-op) |

## Refactoring Notes

When verbose/debug output is added (e.g., via the tracing or log crate), a --verbose
flag will be declared and this BC will be retired (DF-030). Note that ADR 0004's
process-wide AtomicBool warnings already provide a lightweight one-shot diagnostic
mechanism that does not require a verbosity flag.
