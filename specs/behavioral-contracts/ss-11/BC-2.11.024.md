---
document_type: behavioral-contract
level: L3
version: "1.8"
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
  - "v1.3: pc4 timestamp example Z→+00:00 to match chrono to_rfc3339() DateTime<Utc> output + story AC-010; EC-010 hedge removed, output locked to +00:00 (Wave-22 P2 finding F-002) — 2026-05-30"
  - "v1.4: ADR-006 / Decision 13 §13.3 (F2 v0.3.0 BREAKING) — column 6 field renamed mitre_technique->mitre_techniques; type changed Option<String>->Vec<String>; encoding rule: empty vec->'', singleton vec->plain ID string, multi-element vec->semicolon-joined string; added EC-013/EC-014 for multi-value cases. — 2026-06-09"
  - "v1.5: ADD-ON 2 (research-backed, f2-multitag-schema.md §3) — clarify empty-cell is EMPTY STRING not null/[]/none; EC-001 updated; EC-015 added for consumer split guard (str.split(';') on empty produces [''] not [] — consumers must guard). — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, Postconditions, Invariants, EC-013, EC-014, Canonical Test Vectors, and Architecture Anchors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.7: Pass-15 D-01: Evidence Types Used section updated — guard clause description corrected from stale pre-v0.3.0 'four Option fields / csv.rs:82-85' to current shape: mitre_techniques Vec<String> via .join(\";\") at csv.rs:87, three Option fields (source_ip, direction, timestamp) via unwrap_or_default() at csv.rs:88-90. — 2026-06-13"
  - "v1.8: PG-ARP-F2-007 — fix stale csv.rs neutralize anchor: Architecture Anchor :94-97 → :99-102 (neutralize_csv_injection on mitre at :99, source_ip at :100, direction at :101, timestamp at :102); Postcondition 1 text clarified: join at csv.rs:87; neutralize on mitre at csv.rs:99; Postcondition 5 neutralize range updated from :94-97 → :99-102; Invariant 4 neutralize reference updated; verified against current HEAD — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.024: CsvReporter Encodes Optional Fields as Empty Strings and mitre_techniques as Semicolon-Joined String

<!--
  PREVIOUS VERSION SUMMARY (v1.3 -> v1.4):
  Field renamed: mitre_technique -> mitre_techniques (column 6)
  Type changed: Option<String> -> Vec<String>
  Postcondition 1 rewritten: None/Some semantics -> empty/singleton/multi vec semantics
  Encoding: empty vec -> ""; singleton vec -> plain ID; multi-element vec -> "T1692.001;T0836"
  Precondition 2: mitre_technique -> mitre_techniques
  EC-001: None -> vec![] empty; EC-002: Some("T1059") -> vec!["T1059"]
  EC-011 (injection): mitre_technique=Some("=...") -> mitre_techniques=vec!["=..."]
  EC-013 added: multi-value vec -> semicolon-joined cell
  EC-014 added: injection risk in multi-value vec (neutralize_csv_injection applied to joined string)
  Architecture Anchors: csv.rs:82 expression updated
-->

## Description

Column 6 of the CSV output (`mitre_techniques`) encodes the `Finding.mitre_techniques` field
as follows: empty vec produces an empty string cell `""`; a singleton vec produces the single
ID as a plain string (e.g., `"T1036"`); a multi-element vec produces the IDs joined with
semicolons and no spaces (e.g., `"T1692.001;T0836"`). Semicolons are neutral characters that do
not trigger CSV injection. The other three optional-field columns (7-9) are unchanged: `None`
produces `""` and `Some(v)` uses a type-specific string conversion.

All derived strings (including the joined techniques string) are processed through
`neutralize_csv_injection` before the csv write. The column count remains 9.

## Preconditions

1. A `Finding` row is being rendered by `CsvReporter`.
2. Any or all of `mitre_techniques` (Vec<String>), `source_ip`, `direction`, `timestamp`
   may be empty/None or non-empty/Some(_).

## Postconditions

1. `mitre_techniques` cell (column 6):
   - `vec![]` (empty) → empty string `""`
   - `vec!["T1036"]` (singleton) → `"T1036"` (same as pre-F2 `Some("T1036")` output)
   - `vec!["T1692.001", "T0836"]` (multi) → `"T1692.001;T0836"` (semicolon-joined, no spaces)
   The techniques are joined at csv.rs:87; the joined string is passed through `neutralize_csv_injection` at csv.rs:99.
2. `source_ip` cell (column 7): if `None`, empty string `""`; if `Some(ip)`, `ip.to_string()`
   (decimal-dotted for IPv4, colon-hex for IPv6) at csv.rs:88.
3. `direction` cell (column 8): if `None`, empty string `""`; if `Some(d)`, `format!("{d:?}")`
   which produces the variant name `"ClientToServer"` or `"ServerToClient"` (Debug
   representation of the `Direction` enum) at csv.rs:89.
4. `timestamp` cell (column 9): if `None`, empty string `""`; if `Some(t)`, `t.to_rfc3339()` --
   an ISO 8601 / RFC 3339 string (e.g., `"2024-01-15T12:34:56+00:00"`) at csv.rs:90.
   `chrono::DateTime<Utc>::to_rfc3339()` always emits the `+00:00` offset form, never
   a bare `Z` suffix.
5. All four derived strings (joined-techniques, source_ip_str, direction_str, timestamp_str)
   are individually passed through `neutralize_csv_injection` at csv.rs:99-102 before write.
6. The CSV cell is always present (may be empty); absent/empty values are NEVER represented
   by omitting the column.

## Invariants

1. The column count is always 9 regardless of which fields are empty or None.
2. The encoding for `direction` is Debug (`{:?}`), not Display; the output is CamelCase
   variant names (`"ClientToServer"`, `"ServerToClient"`), not lowercase or other formats.
3. The timestamp encoding is `to_rfc3339()`; the output is always a valid RFC 3339 string
   when not empty (UTC timezone designation included).
4. Empty-to-empty-string conversion uses `join(";")` on an empty vec producing `""` (csv.rs:87),
   and `unwrap_or_default()` for the three Option fields (csv.rs:88-90); the neutralize calls
   on these four derived strings are at csv.rs:99-102 within the write_record block; no sentinel
   values like `"null"`, `"N/A"`, or `"-"` are used. The empty cell is ALWAYS an empty string
   (`""`); it is NEVER the literal text `"null"` or `"[]"`. Consumers MUST guard against
   the split-on-empty-cell false-positive (see EC-015); wirerust does not insert any
   separator value into an empty cell to work around the consumer language's split behavior.
5. The semicolon join separator (`;`) for `mitre_techniques` is chosen because semicolons
   are neutral CSV characters and do not require quoting. The `neutralize_csv_injection`
   guard is applied to the entire joined string, not to individual IDs.
6. **Field delimiter is COMMA (`,`).** The CSV writer uses comma as the column separator
   (standard RFC 4180 CSV). The semicolon-joined `mitre_techniques` cell therefore stays in
   a single column regardless of how many techniques are joined — e.g., `"T1692.001;T0836"` is
   one cell, not two, because semicolons are not the field delimiter. Consumers must NOT
   split on semicolons to get columns; semicolons split technique IDs within the cell only.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | mitre_techniques = vec![] (empty) | Column 6 cell is EMPTY STRING `""` — NOT the literal text `"null"`, `"[]"`, `"N/A"`, or `"-"`. Consumers calling `str.split(';')` on an empty string receive `['']` (a single empty element) — they MUST guard with `if cell.is_empty() { vec![] }` or equivalent to avoid a spurious single-empty-element result. See EC-015 for the consumer contract. |
| EC-002 | mitre_techniques = vec!["T1059"] (singleton) | Column 6 cell is `"T1059"` (identical to prior Some("T1059") output) |
| EC-003 | source_ip = None | Column 7 cell is empty string `""` |
| EC-004 | source_ip = Some(192.168.1.1) | Column 7 cell is `"192.168.1.1"` |
| EC-005 | source_ip = Some(::1) | Column 7 cell is `"::1"` (IPv6 compact form via IpAddr::to_string) |
| EC-006 | direction = None | Column 8 cell is empty string `""` |
| EC-007 | direction = Some(ClientToServer) | Column 8 cell is `"ClientToServer"` |
| EC-008 | direction = Some(ServerToClient) | Column 8 cell is `"ServerToClient"` |
| EC-009 | timestamp = None | Column 9 cell is empty string `""` |
| EC-010 | timestamp = Some(2024-01-15T12:34:56 UTC) | Column 9 cell is `"2024-01-15T12:34:56+00:00"` (chrono::DateTime<Utc>::to_rfc3339() always emits +00:00, never bare Z) |
| EC-011 | mitre_techniques = vec!["=HYPERLINK(...)"] (injection attempt) | Cell is `"'=HYPERLINK(...)"` (neutralize_csv_injection applied to joined string) |
| EC-012 | All fields empty/None simultaneously | All four cells are empty string; row has 5 non-empty + 4 empty cells |
| EC-013 | mitre_techniques = vec!["T1692.001","T0836"] (Modbus register write, multi-tag; T1692.001 = v19 ICS sub-technique, successor to revoked T0855) | Column 6 cell is `"T1692.001;T0836"` (semicolon-joined, no spaces) |
| EC-014 | mitre_techniques = vec!["T0806","T1692.001"] (burst finding, multi-tag; T1692.001 = v19 ICS sub-technique, successor to revoked T0855) | Column 6 cell is `"T0806;T1692.001"` |
| EC-015 | Consumer splits column 6 on `;` when cell is empty string `""` | `"".split(';')` returns `[""]` in most languages (Python: `['']`, Rust `str::split`: `[""]`, Splunk `makemv delim=";"` on empty cell: empty multivalue). Consumers MUST check `if cell.is_empty()` before splitting and return an empty collection — NOT a single-element collection containing an empty string. This guard prevents spurious `[""]` results in analytics. Wirerust is not responsible for consumer split behavior; this EC documents the required consumer contract. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All fields empty/None | Columns 6-9 cells are all empty strings | happy-path (no optionals) |
| direction=Some(ClientToServer) | Column 8 is `"ClientToServer"` | happy-path |
| direction=Some(ServerToClient) | Column 8 is `"ServerToClient"` | happy-path |
| source_ip=Some(10.0.0.1) | Column 7 is `"10.0.0.1"` | happy-path |
| source_ip=Some(2001:db8::1) | Column 7 is `"2001:db8::1"` | edge-case (IPv6) |
| mitre_techniques=vec!["=cmd"] | Column 6 is `"'=cmd"` (neutralized) | edge-case (injection) |
| mitre_techniques=vec!["T1692.001","T0836"] | Column 6 is `"T1692.001;T0836"` | happy-path (multi-tag) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | None options produce empty-string cells, not "null" or absent columns | unit |
| — | direction Debug format produces CamelCase variant names | unit |
| — | timestamp to_rfc3339 produces valid RFC 3339 strings | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC defines the encoding convention for optional Finding fields in the CSV channel, directly determining what analysts see in their spreadsheet for absent data |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- direction and timestamp are formatted at render time; the raw Finding carries Option<T>) |
| Architecture Module | SS-11 (reporter/csv.rs:87-102) |
| Stories | STORY-080 |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.020 -- depends on (these cells occupy columns 6-9 of the fixed schema)
- BC-2.11.021 -- composes with (all derived strings are neutralized before write)
- BC-2.11.023 -- depends on (per-row render contract; this BC specifies optional-field encoding within each row)

## Architecture Anchors

- `src/reporter/csv.rs:87` -- `f.mitre_techniques.join(";")` (replaces `f.mitre_technique.as_deref().unwrap_or("")`; empty vec produces `""` via join)
- `src/reporter/csv.rs:88` -- `f.source_ip.map(|ip| ip.to_string()).unwrap_or_default()`
- `src/reporter/csv.rs:89` -- `f.direction.map(|d| format!("{d:?}")).unwrap_or_default()`
- `src/reporter/csv.rs:90` -- `f.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default()`
- `src/reporter/csv.rs:99-102` -- `neutralize_csv_injection` applied to all four optional-derived strings (mitre at :99, source_ip at :100, direction at :101, timestamp at :102) within the write_record block at :92-:103

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

- **guard clause**: `mitre_techniques.join(";")` at csv.rs:87 for the Vec<String> field (empty vec → ""); `unwrap_or_default()` calls at csv.rs:88-90 for the three Option fields (source_ip, direction, timestamp); neutralize_csv_injection applied to these four derived strings at csv.rs:99-102
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

The `mitre_technique.as_deref().unwrap_or("")` expression at csv.rs:82 is replaced by
`mitre_techniques.join(";")` in v0.3.0. An empty Vec<String> joined on ";" produces `""`,
which is the correct empty-cell encoding. No other structural change is required for the
four optional-field columns. Note: if `Direction` gains a `Display` impl in the future,
this BC must be updated to reflect whether Display or Debug is used, as the output strings
would differ.
