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
capability: CAP-12
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

# BC-2.13.003: --filter <BPF> Flag Does Not Exist; No BPF Filter Applied

## Description

`--filter` (BPF expression filter) is NOT declared in the current `cli.rs`. It was removed
by PR #74. No BPF filtering capability exists in the codebase: wirerust reads every packet
from the pcap file, decodes it, and counts failures as skipped packets. No pre-processing
filter is applied to restrict which packets are analyzed. This is a documented absent
behavior -- BPF-style filtering is out of scope for the current release (Section 1.5).

## Preconditions

1. The CLI is invoked with `--filter <BPF_EXPRESSION>` as arguments.

## Postconditions

1. clap rejects the argument with an `UnknownArgument` error.
2. wirerust exits with a non-zero exit code.
3. Under any invocation WITHOUT --filter: all packets in the pcap file are processed
   (subject only to link-type gating and per-packet decode errors).

## Invariants

1. `--filter` does not appear in the CLI surface.
2. No BPF expression evaluation exists in `src/`.
3. All packets from an accepted pcap file are processed; there is no packet-level
   pre-filtering mechanism.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | wirerust analyze --filter "tcp port 80" test.pcap | clap error; exit nonzero |
| EC-002 | wirerust analyze test.pcap | All packets processed; no filter applied |
| EC-003 | pcap with 1M packets | All 1M packets attempted (no BPF pre-filter) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Cli::try_parse_from(["wirerust", "analyze", "--filter", "tcp", "test.pcap"]) | Err (unknown argument) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --filter argument is rejected by clap | unit: try_parse_from returns Err for --filter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- this BC documents an absent CLI flag; the CLI argument surface (including the non-existence of --filter) is what CAP-12 governs |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-13 (cli.rs, C-3 -- flag absence documented) |
| Stories | S-TBD |
| Origin BC | BC-ABS-003 (pass-3 ingestion corpus, HIGH confidence absent) |

## Related BCs

- BC-2.13.001 -- related to (same absent-flag pattern)
- BC-2.13.002 -- related to (--beacon also absent)

## Architecture Anchors

- `src/cli.rs` -- no --filter declaration

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: cli.rs LESSON-P1.04 comment; domain-debt Smell #3 closed by #74

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (absent/no-op) |

## Refactoring Notes

When BPF filtering is implemented, a --filter flag will be added and this BC will be
retired (DF-030). BPF filtering would require integration with a BPF library (e.g., pcap
crate's filter API) or a post-read packet predicate.
