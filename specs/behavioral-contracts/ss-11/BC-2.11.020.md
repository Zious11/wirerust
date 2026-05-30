---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Correct CRLF→LF line-terminator claim in postcondition 1 and description (STORY-079 formalization finding — 2026-05-30)"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.020: CsvReporter Emits Exactly Nine Columns in Fixed Header Order

## Description

`CsvReporter::render` writes a fixed RFC 4180 header row as its first output line with
exactly nine fields in a defined, stable order: `category`, `verdict`, `confidence`,
`summary`, `evidence`, `mitre_technique`, `source_ip`, `direction`, `timestamp`. This
column contract is intentionally locked so downstream parsers (spreadsheets, SIEM pipelines,
analyst scripts) can rely on positional column indices without schema discovery.

## Preconditions

1. `CsvReporter::render` is called with any `&Summary`, any `&[Finding]` (including empty),
   and any `&[AnalysisSummary]`.
2. The internal `csv::WriterBuilder` is initialized writing to an in-memory `Vec<u8>` buffer.

## Postconditions

1. The returned `String` begins with the header line:
   `category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp`
   followed by a LF (`\n`) line terminator. The `csv` crate's `WriterBuilder::new()` defaults
   to LF-only termination; RFC 4180 CRLF would require an explicit `.terminator(Terminator::CRLF)`
   call, which the implementation does not configure. RFC 4180-compliant readers accept LF line
   endings, so interoperability with spreadsheets and SIEM pipelines is preserved.
2. Every data row contains exactly nine comma-separated fields, in the same column order as
   the header.
3. The column order is: (1) `category`, (2) `verdict`, (3) `confidence`, (4) `summary`,
   (5) `evidence`, (6) `mitre_technique`, (7) `source_ip`, (8) `direction`, (9) `timestamp`.
4. The output is valid RFC 4180 CSV: fields containing commas, double-quotes, or newlines
   are quoted by the `csv` crate automatically.
5. The entire output is valid UTF-8 (guaranteed by construction from UTF-8 String inputs).

## Invariants

1. Column count is exactly 9 in every row including the header; no row may omit or add a
   column.
2. Column order is immutable; a future addition requires a new BC and a versioned schema
   change, not in-place reordering.
3. The `csv::WriterBuilder` default separator is a comma (U+002C); no alternative separator
   is configured.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | findings slice is empty | Output is header row only (one line); still valid CSV |
| EC-002 | findings slice has 1 element | Header + exactly 1 data row = 2 lines |
| EC-003 | findings slice has 10,000 elements | Header + 10,000 data rows; column count constant |
| EC-004 | A field value contains a comma | csv crate wraps that field in double quotes; column count unchanged |
| EC-005 | A field value contains a double-quote | csv crate escapes as `""` inside a quoted field |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Empty findings slice | CSV string starts with `category,verdict,...,timestamp` header only | happy-path |
| One Finding (Anomaly/Likely/High, summary="test") | Row 2 columns 1-3 are `Anomaly,LIKELY,HIGH` | happy-path |
| Field value `"a,b"` (contains comma) | Field is quoted: `"a,b"` in the CSV output | edge-case |
| Field value contains `"` character | Field is double-quote escaped per RFC 4180 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Header row is first line and contains exactly the 9 expected column names in order | unit |
| — | Every row in output has exactly 9 comma-separated fields | unit / proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC defines the CSV output column schema that is the fixed API surface of the CsvReporter, directly implementing the reporting output capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- CsvReporter applies neutralization at render time; the Finding retains raw bytes) |
| Architecture Module | SS-11 (reporter/csv.rs, lines 62-73) |
| Stories | STORY-079 |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.021 -- composes with (CSV-injection neutralization applied to each cell before write)
- BC-2.11.022 -- composes with (evidence Vec joined before being placed in column 5)
- BC-2.11.023 -- composes with (one data row per Finding)
- BC-2.11.024 -- composes with (None optional fields produce empty-string cells in columns 6-9)

## Architecture Anchors

- `src/reporter/csv.rs:62-73` -- `write_record` call that emits the fixed 9-column header
- `src/reporter/csv.rs:58` -- `csv::WriterBuilder::new().from_writer(Vec::new())` initialization

## Story Anchor

STORY-079 -- CsvReporter implementation (LESSON-P2.03)

## VP Anchors

- — -- column schema unit test

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/csv.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: `write_record` at csv.rs:62-73 enumerates the exact 9 field names as an array literal; order is structural
- **documentation**: module doc comment (csv.rs:1-30) states "fixed header" and "Order is stable so downstream parsers can rely on column positions"
- **type constraint**: `csv::WriterBuilder` enforces RFC 4180 formatting including quoting

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (writes to in-memory Vec<u8> buffer only; returns owned String) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (CsvReporter is a unit struct) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- pure string transformation over an in-memory buffer. Suitable for unit testing with golden-string comparison.
