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

# BC-2.12.007: --reassemble and --no-reassemble are Mutually Exclusive (clap conflicts_with)

## Description

The `--reassemble` flag at `cli.rs:62` declares `conflicts_with = "no_reassemble"`. Clap
interprets this as a bidirectional conflict: passing both `--reassemble` AND `--no-reassemble`
in any order causes `Cli::try_parse_from` to return an error of kind
`clap::error::ErrorKind::ArgumentConflict`. No runtime code is reached.

This is a MEDIUM-confidence contract because no test currently passes both flags simultaneously
to assert the conflict fires. A future maintainer removing the `conflicts_with` attribute
would silently break the invariant; main.rs assumes the flags are never both true.

If both flags were somehow passed simultaneously, the downstream code at main.rs:90-94 would:
1. Emit a stderr warning ("Warning: --http/--tls require TCP reassembly, but
   --no-reassemble is set. Stream analysis will be skipped.").
2. Skip reassembler creation (no_reassemble wins at runtime).

## Preconditions

1. `Cli::try_parse_from` is called with arguments containing both `--reassemble` and
   `--no-reassemble`.

## Postconditions

1. Returns `Err` with error kind `ArgumentConflict`.
2. No runtime logic is executed (parse fails before subcommand dispatch).

## Invariants

1. The conflict is declared on `--reassemble` only (asymmetric declaration). Clap makes it
   bidirectional regardless of which flag declares the conflict.
2. This constraint is enforced by clap's arg parsing, not by application logic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --reassemble followed by --no-reassemble | ArgumentConflict error |
| EC-002 | --no-reassemble followed by --reassemble | ArgumentConflict error (symmetric) |
| EC-003 | --reassemble alone | OK |
| EC-004 | --no-reassemble alone | OK |
| EC-005 | Neither flag | OK (both default to false) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "--reassemble", "--no-reassemble", "analyze", "test.pcap"] | Err with ArgumentConflict | error |
| ["wirerust", "--no-reassemble", "--reassemble", "analyze", "test.pcap"] | Err with ArgumentConflict | error |
| ["wirerust", "--reassemble", "analyze", "test.pcap"] | Ok (parses successfully) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-018 | Both flags together produce ArgumentConflict | unit: Cli::try_parse_from(both flags) returns Err with ArgumentConflict kind |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the clap conflicts_with constraint between --reassemble and --no-reassemble is declared on the Cli struct (cli.rs:62) and enforced at parse time before any pipeline wiring; this is a CLI entry-point invariant, not a PCAP ingestion or reassembly behavior |
| L2 Domain Invariants | None |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | S-TBD |
| Origin BC | BC-CLI-007 (pass-3 ingestion corpus, MEDIUM confidence -- clap enforces; no test passes both flags; R4 finding) |

## Related BCs

- BC-2.12.005 -- composes with (both flags are part of the reassembly flag set)
- BC-2.12.009 -- related to (--no-reassemble forces reassembly off regardless)

## Architecture Anchors

- `src/cli.rs:62` -- `#[arg(long, global = true, conflicts_with = "no_reassemble")]` on --reassemble
- `src/cli.rs:63` -- `pub reassemble: bool`
- `src/cli.rs:67` -- `pub no_reassemble: bool`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:62-67` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **documentation**: clap attribute declared; bidirectional effect from asymmetric declaration
- **inferred**: no test currently asserts the conflict path

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (clap parse is pure) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
