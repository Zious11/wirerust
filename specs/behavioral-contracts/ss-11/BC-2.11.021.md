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

# BC-2.11.021: CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote

## Description

Every cell value written by `CsvReporter` passes through `neutralize_csv_injection` before
being handed to the `csv` crate. If the cell's first character is one of `=`, `+`, `-`, `@`,
TAB (U+0009), or CR (U+000D), the function prepends a single-quote character (`'`, U+0027)
to the entire value. This prevents spreadsheet applications (Excel, LibreOffice, Google
Sheets) from interpreting attacker-controlled packet payload bytes as formula directives.

## Preconditions

1. A cell value derived from a `Finding` field is about to be written to the CSV output.
2. The cell value is a `&str` slice of a valid UTF-8 `String`.
3. All nine column values are individually processed through `neutralize_csv_injection`
   before the `write_record` call at csv.rs:88-98.

## Postconditions

1. If the first character of the input is `=`, `+`, `-`, `@`, `\t`, or `\r`, the output
   string begins with `'` followed by the entire original input unchanged.
2. If the first character is any other character (including an empty string whose
   `chars().next()` returns `None`), the output string is identical to the input string.
3. Only the FIRST character is inspected; no other characters in the value are altered.
4. An empty string input (`""`) is returned unchanged (no prefix added).

## Invariants

1. The neutralization function is applied to ALL nine column values for EVERY data row
   without exception (csv.rs:89-97 calls it on each field individually).
2. The function does not inspect or alter any character after the first.
3. The function does not alter bytes mid-string or strip any content; it only prepends.
4. The `csv` crate's own RFC 4180 quoting is applied AFTER neutralization, so the `'`
   prefix is preserved verbatim in the output (it is not a special character in CSV).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cell value starts with `=` | Output: `'=<rest of value>` |
| EC-002 | Cell value starts with `+` | Output: `'+<rest of value>` |
| EC-003 | Cell value starts with `-` | Output: `'-<rest of value>` |
| EC-004 | Cell value starts with `@` | Output: `'@<rest of value>` |
| EC-005 | Cell value starts with TAB (U+0009) | Output: `'\t<rest of value>` (literal tab still present after prefix) |
| EC-006 | Cell value starts with CR (U+000D) | Output: `'\r<rest of value>` (literal CR still present after prefix) |
| EC-007 | Cell value starts with `'` (already prefixed or summary contains quote) | No change; single-quote is not a trigger character |
| EC-008 | Cell value is empty string `""` | Returned unchanged; `chars().next()` is `None` |
| EC-009 | Cell value starts with a printable ASCII letter `A`..`z` | Returned unchanged |
| EC-010 | Cell value starts with a digit `0`..`9` | Returned unchanged |
| EC-011 | trigger character is in position 2+ but not position 1 | No prefix added; only first char is inspected |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `"=SUM(A1:A2)"` | `"'=SUM(A1:A2)"` | happy-path |
| `"+payload"` | `"'+payload"` | happy-path |
| `"-1"` | `"'-1"` | happy-path |
| `"@admin"` | `"'@admin"` | happy-path |
| `"\tindented"` | `"'\tindented"` | edge-case (TAB) |
| `"\rcarriage"` | `"'\rcarriage"` | edge-case (CR) |
| `""` | `""` | edge-case (empty) |
| `"normal text"` | `"normal text"` | happy-path (no trigger) |
| `"'already-prefixed"` | `"'already-prefixed"` | edge-case (single-quote not a trigger) |
| `"a=formula"` | `"a=formula"` | edge-case (trigger not at position 0) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | neutralize_csv_injection produces `'`-prefix for all 6 trigger chars and only those | unit: parametric test over trigger set |
| VP-TBD | neutralize_csv_injection returns identity for empty string | unit |
| VP-TBD | neutralize_csv_injection does not alter bytes after position 0 | proptest: for all non-trigger-prefixed inputs, output == input |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC describes a security property of the CSV output path: neutralizing formula injection for the analyst-facing spreadsheet export channel |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- Finding.summary/evidence carry raw attacker bytes; CsvReporter's neutralization is the display-layer sanitization, mirroring TerminalReporter's control-byte escaping per ADR 0003) |
| Architecture Module | SS-11 (reporter/csv.rs:40-44, lines 89-97) |
| Stories | S-TBD |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.020 -- depends on (this neutralization is applied per-cell before the fixed 9-column schema write)
- BC-2.11.007 -- related to (TerminalReporter's analogous C0/DEL escaping at display layer per ADR 0003 -- different mechanism, same principle)
- BC-2.09.005 -- depends on (Finding.summary/evidence store raw bytes, making neutralization mandatory at render time)

## Architecture Anchors

- `src/reporter/csv.rs:40-44` -- `neutralize_csv_injection` function definition
- `src/reporter/csv.rs:89-97` -- application of `neutralize_csv_injection` to all nine data columns
- `src/reporter/csv.rs:18-31` -- module doc comment describing the OWASP CSV injection threat model

## Story Anchor

S-TBD -- CsvReporter implementation (LESSON-P2.03)

## VP Anchors

- VP-TBD -- neutralization unit tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/csv.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: `neutralize_csv_injection` at csv.rs:40-44 uses an explicit `match` on `s.chars().next()` with the six trigger characters listed as a pattern arm
- **documentation**: module doc comment at csv.rs:18-31 states the OWASP rationale and names all six trigger characters
- **assertion**: function is called on all nine columns at csv.rs:89-97 with no bypass path

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function, no captured state) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- pure string transformation, ideal for property-based testing with proptest.
