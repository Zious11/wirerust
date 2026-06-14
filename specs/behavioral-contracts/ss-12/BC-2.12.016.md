---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: DF-SIBLING-SWEEP-001 — fix stale main.rs line anchors: resolve_format 304-320 → 316-324 (fn at 316, closing at 324); reporter selection match 222-240 → 225-245; capability anchor ref updated; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.4: F3-convergence de-pin — removed numeric line anchors for resolve_format and reporter-selection match; replaced with symbol/concept anchors (drift-proof); verified live src: resolve_format at main.rs:392, reporter match at ~301 — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.016: Output Format Selection: json->JsonReporter, csv->CsvReporter, else Terminal

## Description

`resolve_format(cli)` determines the output format with the following precedence: (1) if
`cli.json.is_some()`, `OutputFormat::Json`; (2) else if `cli.csv.is_some()`, `OutputFormat::Csv`;
(3) else `cli.output_format` as-is. The match arm in `run_analyze`/`run_summary` then selects
the reporter: `Some(Json)` -> `JsonReporter`, `Some(Csv)` -> `CsvReporter`, `_ ` (None or
any unhandled variant) -> `TerminalReporter`.

## Preconditions

1. `resolve_format(cli)` is called after CLI parsing.
2. At most one of `--json`, `--csv`, or `--output-format` is active (clap enforces `--json`
   and `--csv` are mutually exclusive; `--output-format` may coexist but is lower precedence).

## Postconditions

1. `resolve_format` returns `Some(Json)` when `--json [<FILE>]` was given.
2. `resolve_format` returns `Some(Csv)` when `--csv [<FILE>]` was given.
3. `resolve_format` returns `cli.output_format` (which may be None) when neither `--json`
   nor `--csv` was given.
4. The reporter selected matches the resolved format.

## Invariants

1. `--json` flag on `Cli` is `Option<Option<PathBuf>>`; `cli.json.is_some()` is true even
   when `--json` is given without a file path (`cli.json = Some(None)`).
2. `--json` and `--csv` are mutually exclusive (clap enforces via `conflicts_with = "csv"` on
   the json field in cli.rs).
3. `TerminalReporter` is the default when no format flag is given.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --json (no path) | resolve_format returns Some(Json) |
| EC-002 | --json output.json | resolve_format returns Some(Json) |
| EC-003 | --csv | resolve_format returns Some(Csv) |
| EC-004 | --output-format json | resolve_format returns Some(Json) |
| EC-005 | No flags | resolve_format returns None; TerminalReporter used |
| EC-006 | --json and --output-format csv | --json wins (higher precedence) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| cli.json=Some(None), cli.csv=None, output_format=None | Some(Json) | happy-path |
| cli.json=None, cli.csv=Some(None) | Some(Csv) | happy-path |
| All None | None (TerminalReporter) | happy-path |
| cli.json=Some(None), output_format=Some(Csv) | Some(Json) (json wins) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | resolve_format precedence is correct | unit: code-level (HIGH -- code is explicit and deterministic) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- `resolve_format` in main.rs is CAP-12's output-channel selection step; it reads CLI flag state and returns the reporter variant to instantiate; this is entry-point orchestration (choosing which reporter to create), not the reporter's own rendering logic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | STORY-089 |
| Origin BC | BC-CLI-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.004 -- depends on (--output-format flag provides the fallback value)
- BC-2.12.017 -- composes with (output routing uses the resolved format)
- BC-2.11.001 -- depends on (JsonReporter is selected when format=Json)

## Architecture Anchors

- `src/main.rs` `resolve_format` function
- `src/main.rs` reporter-selection match in `run_analyze`

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs` `resolve_format` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: resolve_format has an explicit doc comment at line 304-311

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | N/A |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
