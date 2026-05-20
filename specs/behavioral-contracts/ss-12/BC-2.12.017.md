---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
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

# BC-2.12.017: Output Routed to File if --json/--csv <FILE>; Stdout Otherwise

## Description

`write_output` in main.rs routes the rendered output to a file when `--json <FILE>` or
`--csv <FILE>` was given with a path argument, or to stdout (via `println!`) otherwise.
Specifically: `cli.json = Some(Some(path))` writes to that path; `cli.csv = Some(Some(path))`
writes to that path; any other combination (no path, no flag, or `--output-format`) prints
to stdout. This behavior was previously absent (BC-ABS-006/BC-ABS-007) and was wired by
the remediation cycle.

## Preconditions

1. `write_output(output, cli)` is called with the rendered output string.
2. `cli.json` and `cli.csv` may contain Some(Some(path)) for file output.

## Postconditions

1. When `cli.json = Some(Some(path))`: `std::fs::write(path, output)` with anyhow context.
2. When `cli.csv = Some(Some(path))`: `std::fs::write(path, output)` with anyhow context.
3. Otherwise: `println!("{output}")` to stdout.
4. Only one of the two file-write arms can be active (--json and --csv are mutually exclusive).

## Invariants

1. `cli.json` is `Option<Option<PathBuf>>`; `Some(None)` means `--json` given without path
   (print to stdout). `Some(Some(path))` means write to file.
2. `cli.csv` has the same nested Option structure.
3. `write_output` is called ONCE per run, after all analysis is complete.
4. File write errors are wrapped with anyhow context: "Failed to write JSON output to <path>"
   or "Failed to write CSV output to <path>".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --json output.json | Writes to output.json |
| EC-002 | --json (no path) | Prints to stdout |
| EC-003 | --csv results.csv | Writes to results.csv |
| EC-004 | --csv (no path) | Prints to stdout |
| EC-005 | No --json/--csv flags | Prints to stdout |
| EC-006 | File write fails (permissions) | Err with anyhow context message |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| cli.json=Some(Some(PathBuf::from("out.json"))) | File out.json created with output | happy-path |
| cli.json=Some(None) | stdout contains output | happy-path |
| cli.json=None, cli.csv=None | stdout contains output | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --json <FILE> writes to file | unit: write_output with tempfile (MEDIUM -- not directly tested) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- output routing to file vs stdout is a core part of the reporting output delivery mechanism |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-017 (pass-3 ingestion corpus, MEDIUM confidence -- file-write path now wired; was previously BC-ABS-006/BC-ABS-007 absent behaviors, now retired) |

## Related BCs

- BC-2.12.016 -- depends on (format selection precedes output routing)

## Architecture Anchors

- `src/main.rs:322-338` -- write_output function
- `src/main.rs:329-332` -- file-write arms for --json <FILE> and --csv <FILE>
- `src/main.rs:333-337` -- stdout fallback via println!

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:322-338` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: write_output has an explicit doc comment at lines 322-328
- **inferred**: file-write path is wired but not unit-tested

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to file or stdout |
| **Global state access** | none |
| **Deterministic** | yes (given same input) |
| **Thread safety** | N/A |
| **Overall classification** | effectful shell |

#### Refactoring Notes

Note: the base ingestion document (BC-CLI-017) described file flags as "ignored." This was
accurate for the state at ingestion time but is now INCORRECT. The `write_output` function
was wired by the remediation cycle (PRs #84 et al.), closing BC-ABS-006 and BC-ABS-007.
This BC reflects the current shipped behavior.
