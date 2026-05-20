---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/terminal.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
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

# BC-2.11.006: TerminalReporter Shows Skipped: N Packets Only When N > 0

## Description

`TerminalReporter::render` conditionally renders the `Skipped: N packets (decode errors)`
warning line in the report header. The line is shown only when `summary.skipped_packets > 0`.
When `skipped_packets = 0`, the line is completely absent from the output. This differs from
`JsonReporter` which always includes `skipped_packets` (even when zero) per BC-2.11.002.

## Preconditions

1. `TerminalReporter::render` is called with a `Summary`.
2. `summary.skipped_packets` is set by the calling code.

## Postconditions

1. When `summary.skipped_packets > 0`: a warning line `  Skipped: N packets (decode errors)\n`
   is rendered after the `Packets/Bytes/Hosts` header line.
2. When `summary.skipped_packets == 0`: the warning line is entirely absent from the output.
3. When `use_color = true` and `skipped_packets > 0`, the line is rendered in yellow
   (via `owo_colors::OwoColorize::yellow()`).
4. When `use_color = false`, the line is rendered in plain text.

## Invariants

1. The conditional is `if summary.skipped_packets > 0` at `terminal.rs:94`.
2. The `Packets: N  Bytes: N  Hosts: N` header line is ALWAYS rendered regardless of
   `skipped_packets` value.
3. The warning line is colored only when `use_color = true`; the underlying text is
   the same regardless of color mode.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | skipped_packets = 0 | No Skipped line in output |
| EC-002 | skipped_packets = 1 | "  Skipped: 1 packets (decode errors)\n" present |
| EC-003 | skipped_packets = u64::MAX | Line present with large number |
| EC-004 | use_color=true, skipped_packets=3 | Line rendered in yellow |
| EC-005 | use_color=false, skipped_packets=3 | Line rendered in plain text |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary with skipped_packets=0 | output does not contain "Skipped:" | happy-path |
| Summary with skipped_packets=5 | output contains "Skipped: 5 packets" | happy-path |
| Summary with skipped_packets=0, use_color=true | No yellow line in output | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Line present when skipped_packets > 0 | unit: test_terminal_reporter_shows_skipped_when_nonzero |
| VP-TBD | Line absent when skipped_packets = 0 | unit: test_terminal_reporter_hides_skipped_when_zero |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- the conditional rendering of the skipped-packets warning is part of the terminal output formatting contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | S-TBD |
| Origin BC | BC-RPT-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.002 -- contrasts with (JsonReporter always includes skipped_packets even when zero)
- BC-2.12.014 -- depends on (skipped_packets is populated from decode errors)

## Architecture Anchors

- `src/reporter/terminal.rs:94-104` -- conditional skipped_packets rendering
- `tests/reporter_tests.rs` -- test_terminal_reporter_shows_skipped_when_nonzero, test_terminal_reporter_hides_skipped_when_zero

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:94-104` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: `if summary.skipped_packets > 0` at line 94
- **assertion**: test_terminal_reporter_shows_skipped_when_nonzero, test_terminal_reporter_hides_skipped_when_zero

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (output to String) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
