---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.024: CsvReporter Encodes None Optional Fields as Empty Strings and Direction as Debug Variant Name

## Description

Four `Finding` fields are `Option<T>`: `mitre_technique` (column 6), `source_ip` (column 7),
`direction` (column 8), and `timestamp` (column 9). When any of
these is `None`, `CsvReporter` substitutes an empty string `""` for the cell value. When
`Some`, each uses a type-specific string conversion: `mitre_technique` as `as_deref()`,
`source_ip` via `IpAddr::to_string()`, `direction` via `format!("{d:?}")` (Debug formatting),
and `timestamp` via `DateTime::to_rfc3339()`. All four values are subsequently processed
through `neutralize_csv_injection` before the csv write.

## Preconditions

1. A `Finding` row is being rendered by `CsvReporter`.
2. Any or all of `mitre_technique`, `source_ip`, `direction`, `timestamp` may be `None`
   or `Some(_)`.

## Postconditions

1. `mitre_technique` cell: if `None`, empty string `""`; if `Some(s)`, the string `s`
   (via `as_deref().unwrap_or("")` at csv.rs:82).
2. `source_ip` cell: if `None`, empty string `""`; if `Some(ip)`, `ip.to_string()`
   (decimal-dotted for IPv4, colon-hex for IPv6) at csv.rs:83.
3. `direction` cell: if `None`, empty string `""`; if `Some(d)`, `format!("{d:?}")`
   which produces the variant name `"ClientToServer"` or `"ServerToClient"` (Debug
   representation of the `Direction` enum) at csv.rs:84.
4. `timestamp` cell: if `None`, empty string `""`; if `Some(t)`, `t.to_rfc3339()` --
   an ISO 8601 / RFC 3339 string (e.g., `"2024-01-15T12:34:56Z"`) at csv.rs:85.
5. All four derived strings are individually passed through `neutralize_csv_injection`
   at csv.rs:94-97 before being written.
6. The CSV cell is always present (may be empty); absent `Option` values are NEVER
   represented by omitting the column.

## Invariants

1. The column count is always 9 regardless of which Option fields are None.
2. The encoding for `direction` is Debug (`{:?}`), not Display; the output is CamelCase
   variant names (`"ClientToServer"`, `"ServerToClient"`), not lowercase or other formats.
3. The timestamp encoding is `to_rfc3339()`; the output is always a valid RFC 3339 string
   when not empty (UTC timezone designation included).
4. None-to-empty-string conversion is performed via `unwrap_or("")` and
   `unwrap_or_default()`; no sentinel values like `"null"`, `"N/A"`, or `"-"` are used.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | mitre_technique = None | Column 6 cell is empty string `""` |
| EC-002 | mitre_technique = Some("T1059") | Column 6 cell is `"T1059"` |
| EC-003 | source_ip = None | Column 7 cell is empty string `""` |
| EC-004 | source_ip = Some(192.168.1.1) | Column 7 cell is `"192.168.1.1"` |
| EC-005 | source_ip = Some(::1) | Column 7 cell is `"::1"` (IPv6 compact form via IpAddr::to_string) |
| EC-006 | direction = None | Column 8 cell is empty string `""` |
| EC-007 | direction = Some(ClientToServer) | Column 8 cell is `"ClientToServer"` |
| EC-008 | direction = Some(ServerToClient) | Column 8 cell is `"ServerToClient"` |
| EC-009 | timestamp = None | Column 9 cell is empty string `""` |
| EC-010 | timestamp = Some(2024-01-15T12:34:56Z) | Column 9 cell is `"2024-01-15T12:34:56+00:00"` or `"2024-01-15T12:34:56Z"` (to_rfc3339 format) |
| EC-011 | mitre_technique = Some("=HYPERLINK(...)") | Cell is `"'=HYPERLINK(...)"` (neutralized) |
| EC-012 | All four Option fields are None simultaneously | All four cells are empty string; row has 5 non-empty + 4 empty cells |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All Option fields None | Columns 6-9 cells are all empty strings | happy-path (no optionals) |
| direction=Some(ClientToServer) | Column 8 is `"ClientToServer"` | happy-path |
| direction=Some(ServerToClient) | Column 8 is `"ServerToClient"` | happy-path |
| source_ip=Some(10.0.0.1) | Column 7 is `"10.0.0.1"` | happy-path |
| source_ip=Some(2001:db8::1) | Column 7 is `"2001:db8::1"` | edge-case (IPv6) |
| mitre_technique=Some("=cmd") | Column 6 is `"'=cmd"` (neutralized) | edge-case (injection) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | None options produce empty-string cells, not "null" or absent columns | unit |
| — | direction Debug format produces CamelCase variant names | unit |
| — | timestamp to_rfc3339 produces valid RFC 3339 strings | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC defines the encoding convention for optional Finding fields in the CSV channel, directly determining what analysts see in their spreadsheet for absent data |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- direction and timestamp are formatted at render time; the raw Finding carries Option<T>) |
| Architecture Module | SS-11 (reporter/csv.rs:82-97) |
| Stories | STORY-080 |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.020 -- depends on (these cells occupy columns 6-9 of the fixed schema)
- BC-2.11.021 -- composes with (all derived strings are neutralized before write)
- BC-2.11.023 -- depends on (per-row render contract; this BC specifies optional-field encoding within each row)

## Architecture Anchors

- `src/reporter/csv.rs:82` -- `f.mitre_technique.as_deref().unwrap_or("")`
- `src/reporter/csv.rs:83` -- `f.source_ip.map(|ip| ip.to_string()).unwrap_or_default()`
- `src/reporter/csv.rs:84` -- `f.direction.map(|d| format!("{d:?}")).unwrap_or_default()`
- `src/reporter/csv.rs:85` -- `f.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default()`
- `src/reporter/csv.rs:94-97` -- `neutralize_csv_injection` applied to all four optional-derived strings

## Story Anchor

STORY-080 -- CsvReporter implementation (LESSON-P2.03)

## VP Anchors

- — -- optional field encoding unit tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/csv.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: explicit `unwrap_or("")` / `unwrap_or_default()` calls at csv.rs:82-85 for all four Option fields
- **type constraint**: `Direction` does not implement `Display`; `format!("{d:?}")` is the only available string conversion, making Debug the mandated format
- **documentation**: `Direction` doc comment at handler.rs:22 confirms CamelCase serde representation ("ClientToServer", "ServerToClient")

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes (to_rfc3339 on a fixed DateTime<Utc> is deterministic) |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- each conversion is a single expression. Note: if `Direction` gains a `Display` impl in the future, this BC must be updated to reflect whether Display or Debug is used, as the output strings would differ.
