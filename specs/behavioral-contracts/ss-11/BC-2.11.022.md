---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/csv.rs
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

# BC-2.11.022: CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell

## Description

`Finding::evidence` is a `Vec<String>` that may contain zero, one, or many evidence
strings. `CsvReporter` flattens this vector into a single CSV cell using `join("; ")` --
a semicolon followed by a single space. The joined string is then passed through
CSV-injection neutralization and written as the fifth column (`evidence`). The `csv` crate
applies RFC 4180 quoting if the joined string contains commas, quotes, or newlines.

## Preconditions

1. `f.evidence` is a `Vec<String>` field on a `Finding` being rendered.
2. `CsvReporter::render` is processing a data row for this finding.

## Postconditions

1. The `evidence` cell value equals `f.evidence.join("; ")`: all elements concatenated
   with the literal two-character separator `"; "` (U+003B U+0020) between consecutive
   elements.
2. If `f.evidence` is empty (`vec![]`), the joined result is an empty string `""`;
   the cell is present but empty.
3. If `f.evidence` has exactly one element, the result is that element with no separator
   appended or prepended.
4. The joined string is subsequently processed by `neutralize_csv_injection` before the
   csv write (csv.rs:93 applies `neutralize_csv_injection(&evidence)`).
5. The entire joined-and-neutralized value occupies exactly one CSV cell (column 5);
   it never spans multiple columns.

## Invariants

1. The separator is exactly `"; "` (two characters: U+003B, U+0020); no other separator
   is used.
2. The join is performed before neutralization and before RFC 4180 quoting; both
   subsequent layers see the already-joined string.
3. Individual evidence strings are not individually quoted or escaped before joining;
   they are concatenated as-is then the combined result is processed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | evidence is empty `vec![]` | Cell is empty string `""` |
| EC-002 | evidence has exactly one element `["foo"]` | Cell value is `"foo"` with no separator |
| EC-003 | evidence has two elements `["a", "b"]` | Cell value is `"a; b"` |
| EC-004 | evidence has three elements | Elements separated by `"; "`: `"e1; e2; e3"` |
| EC-005 | An evidence string itself contains `"; "` | The separator occurs naturally in the cell; no disambiguation; downstream parser cannot distinguish embedded vs. join separator |
| EC-006 | Joined string starts with `=` or other trigger char (first evidence element starts with `=`) | `neutralize_csv_injection` prefixes the entire joined string with `'` |
| EC-007 | Joined string contains commas | `csv` crate wraps the cell in double-quotes per RFC 4180 |
| EC-008 | Evidence element is an empty string `""` | Contributes empty segment; consecutive separators possible (e.g., `"; "`) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| evidence=[] | evidence cell is empty string | happy-path (empty) |
| evidence=["single item"] | evidence cell is `"single item"` | happy-path |
| evidence=["first", "second"] | evidence cell is `"first; second"` | happy-path |
| evidence=["a", "b", "c"] | evidence cell is `"a; b; c"` | happy-path |
| evidence=["=formula", "other"] | evidence cell is `"'=formula; other"` (neutralized prefix on joined string) | edge-case |
| evidence=["item,with,commas"] | evidence cell is `"item,with,commas"` wrapped in RFC 4180 quotes | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty evidence produces empty cell | unit |
| VP-TBD | N elements produce exactly N-1 occurrences of `"; "` separator in the joined string | unit / proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC defines how multi-valued evidence is encoded for the spreadsheet-oriented CSV output channel |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- individual evidence strings contain raw bytes; encoding/joining is a display-layer decision) |
| Architecture Module | SS-11 (reporter/csv.rs:81) |
| Stories | S-TBD |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.020 -- depends on (joined evidence string is placed in the fixed column-5 slot)
- BC-2.11.021 -- composes with (joined string is passed to neutralize_csv_injection before write)
- BC-2.11.023 -- depends on (per-row encoding; this BC describes column-5 encoding specifically)

## Architecture Anchors

- `src/reporter/csv.rs:81` -- `let evidence = f.evidence.join("; ");`
- `src/reporter/csv.rs:93` -- `neutralize_csv_injection(&evidence)` applied to joined string

## Story Anchor

S-TBD -- CsvReporter implementation (LESSON-P2.03)

## VP Anchors

- VP-TBD -- evidence join unit tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/csv.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: `f.evidence.join("; ")` at csv.rs:81 is an explicit single-expression encoding decision with a named separator literal
- **documentation**: csv.rs:77-80 comment states "`evidence` is a Vec<String>; flatten with `; ` so the whole list lives in one cell"

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- straightforward Vec join, pure and easily testable.
