---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-08T00:00:00Z
phase: 1a
inputs: []
input-hash: "d41d8cd"
extracted_from: null
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.12.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
breaking: false
breaking_note: >
  The schema_version field is additive (new key in the envelope). However it co-ships
  with the BREAKING enum casing change (BC-2.11.036) in v0.12.0, so the combined v0.12.0
  release is a breaking JSON surface change. The JSON schema is a governed surface outside
  cargo-semver-checks scope (CHANGELOG + BC enforcement).
---

# BC-2.11.037: JSON Report Envelope Includes `schema_version` Field; Value `"2"`; Always Emitted; Enables Consumer Schema Gating

## Description

The `JsonReporter` output gains a new top-level `schema_version` field alongside the five
existing envelope keys (`summary`, `findings`, `analyzers`, `mitre_domain`,
`mitre_attack_version`). The value is the constant string `"2"`. The field is emitted
unconditionally in every JSON report, regardless of input content.

**Rationale for value `"2"`**: The pre-v0.12.0 JSON format has no `schema_version` field;
consumers that receive JSON without this field are implicitly dealing with schema version 1
(PascalCase enum values). The v0.12.0 release introduces the first breaking JSON schema
change — enum-value casing (BC-2.11.036) — making the new format schema version 2.
Consumers can use this field to gate: `schema_version == "2"` confirms they are reading
the v0.12.0+ format with lowercase/snake_case enum values. Future breaking JSON schema
changes will increment this value.

The `schema_version` value is a **string** (not an integer) to remain forward-compatible
with minor revision suffixes (e.g., `"2.1"` for non-breaking additions) if ever needed.
The implementation uses a compile-time constant in `src/reporter/json.rs`.

**Surface scope**: `schema_version` is JSON-only. The field does not appear in CSV output
(no CSV envelope concept; BC-2.11.020 governs the nine CSV columns) and does not appear in
terminal output (BC-2.11.019 governs terminal section order). This mirrors the
`mitre_domain` and `mitre_attack_version` policy per BC-2.11.001 Invariant 6.

This field is **additive** to the JSON envelope in isolation, but it co-ships in the same
v0.12.0 release as the BREAKING enum casing change (BC-2.11.036). The combined v0.12.0
JSON surface change must be documented in the CHANGELOG and is outside
`cargo-semver-checks` scope.

## Preconditions

1. `JsonReporter::render` is called with any `Summary`, `&[Finding]`, and
   `&[AnalysisSummary]` (even empty slices).
2. A compile-time constant `SCHEMA_VERSION: &str = "2"` is defined in `src/reporter/json.rs`.

## Postconditions

1. The JSON output object contains a `"schema_version"` key at the top level.
2. The value of `"schema_version"` is the string `"2"` (a JSON string, not a JSON number).
3. The `"schema_version"` key is present regardless of whether the findings slice is empty,
   whether any analyzers ran, or any other input condition. It is unconditional.
4. The `"schema_version"` key does NOT appear in CSV output. The `CsvReporter` is
   unaffected by this BC.
5. The `"schema_version"` key does NOT appear in terminal output. The `TerminalReporter`
   is unaffected by this BC.
6. After BC-2.11.037 ships, the JSON envelope contains exactly **six** top-level keys:
   `"summary"`, `"findings"`, `"analyzers"`, `"mitre_domain"`, `"mitre_attack_version"`,
   and `"schema_version"`. (BC-2.11.001 governs the other five; this BC adds the sixth.)

## Invariants

1. **`schema_version` is a constant, not derived.** The value `"2"` is a compile-time
   constant in `src/reporter/json.rs`, equivalent in nature to `MITRE_DOMAIN` and
   `MITRE_ATTACK_VERSION` (BC-2.11.001 Invariants 4 and 5). It is NOT computed from input
   data, NOT inferred from findings, and NOT absent under any input condition.
2. **Schema version semantics are monotonically increasing integers-as-strings.** `"1"`
   would represent the pre-v0.12.0 implicit format (no `schema_version` field present).
   `"2"` is the first explicitly versioned format. Future increments (if any) will be
   `"3"`, `"4"`, etc. Minor JSON additions within a major schema version do NOT increment
   the schema version.
3. **Absence signals old format.** Consumers that receive a JSON report without a
   `schema_version` key can infer they are reading the pre-v0.12.0 format (schema v1,
   PascalCase enum values). This is a deliberate design: the absence-vs-presence
   distinction removes the need for a separate format detection heuristic.
4. **JSON-only surface.** The `schema_version` field has no analog in CSV or terminal
   output. The three reporters (JSON, CSV, terminal) are independent output surfaces;
   this BC governs only the JSON surface.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty findings and analyzers slices | `"schema_version": "2"` still present in envelope |
| EC-002 | Report with 0 packets analyzed | `"schema_version": "2"` still present in envelope |
| EC-003 | `schema_version` field position in output | Top-level key; exact key ordering is serde_json Value insertion order (not specified normatively — consumers MUST NOT rely on key order) |
| EC-004 | `schema_version` field in CSV output | Absent — CSV reporters do not emit envelope fields |
| EC-005 | `schema_version` field in terminal output | Absent — terminal reporters do not emit envelope fields |
| EC-006 | Future schema version increment | Value will be `"3"` (or higher); the string `"2"` is specific to the v0.12.0 schema |
| EC-007 | Pre-v0.12.0 JSON report (no schema_version field) | Consumer receives no `schema_version` key — this is the implicit schema v1 signal; not an error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Any valid `JsonReporter::render` call | Top-level JSON object contains `"schema_version": "2"` | happy-path |
| Report with empty findings and analyzers | `"schema_version": "2"` present alongside `"findings": []` and `"analyzers": []` | happy-path (unconditional field) |
| Report with multiple findings from all analyzers | `"schema_version": "2"` at top level; value is string `"2"` not integer `2` | happy-path (string type, not number) |
| CSV output for same report | No `schema_version` column or value in CSV | surface-independence |
| Terminal output for same report | No `schema_version` line in terminal | surface-independence |
| JSON parsed with RFC-8259 compliant parser | `schema_version` field readable as a string `"2"` | JSON-correctness |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `"schema_version"` key is present in JSON report output | unit: test_BC_2_11_037_schema_version_present_in_json |
| — | `"schema_version"` value is the string `"2"` (not integer `2`, not `null`, not absent) | unit: test_BC_2_11_037_schema_version_value_is_two |
| — | `"schema_version"` present when findings slice is empty | unit: test_BC_2_11_037_schema_version_unconditional_empty_findings |
| — | `"schema_version"` absent from CSV output | unit: test_BC_2_11_037_schema_version_absent_from_csv |
| — | `"schema_version"` absent from terminal output | unit: test_BC_2_11_037_schema_version_absent_from_terminal |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 — `schema_version` is a versioning signal in the JSON report envelope, enabling consumers of the Reporting capability's JSON output to reliably distinguish schema formats across releases |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — schema_version is JSON-only; CSV and terminal reporters are unaffected) |
| Architecture Module | SS-11 (reporter/json.rs — new `SCHEMA_VERSION` constant) |
| Stories | TBD (story-writer assigns) |
| Issue | #255 (snake_case JSON enums — schema_version co-ships with BC-2.11.036 in v0.12.0) |
| ADR | None required (additive constant field; consistent with ADR-0003 serde delegation pattern and BC-2.11.001 Invariant 4 constant-field pattern for MITRE_DOMAIN) |

## Related BCs

- BC-2.11.001 — JSON envelope shape (advisory pointer in v1.8; this BC adds the sixth top-level key)
- BC-2.11.036 — JSON enum-value casing (co-ships in v0.12.0; schema_version="2" is the machine-readable signal that enum values are now lowercase/snake_case)

## Architecture Anchors

- `src/reporter/json.rs` — new compile-time constant: `const SCHEMA_VERSION: &str = "2";`
  (analogous to existing `MITRE_DOMAIN` and `MITRE_ATTACK_VERSION` constants per BC-2.11.001)
- `src/reporter/json.rs` — `JsonReporter::render`: add `"schema_version": SCHEMA_VERSION`
  to the top-level JSON object alongside the five existing envelope keys

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (constant string insertion into an in-memory Value; returns owned String) |
| **Global state access** | none |
| **Deterministic** | yes — compile-time constant; output is always `"schema_version": "2"` |
| **Thread safety** | Send + Sync (constant; no mutable state) |
| **Overall classification** | pure |
