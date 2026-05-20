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

# BC-2.11.009: TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved

## Description

`escape_for_terminal` escapes all C1 control codepoints in the range U+0080 through U+009F
inclusive (e.g., NEL U+0085, CSI U+009B). U+00A0 (Non-Breaking Space) is NOT escaped because
it is outside the C1 range and is a legitimate printable whitespace character. This closes the
narrow but real vector where DEC S8C1T terminals can interpret 8-bit C1 sequences.

## Preconditions

1. `escape_for_terminal` is called with a string containing C1 codepoints.
2. The string is valid UTF-8 (C1 codepoints in Rust `String` are always valid multi-byte
   UTF-8, e.g., U+009B -> 0xC2 0x9B).

## Postconditions

1. All codepoints in U+0080-U+009F are replaced with `char::escape_default` output
   (e.g., U+0085 -> `\u{85}`, U+009B -> `\u{9b}`).
2. U+00A0 (NBSP) passes through unchanged.
3. U+00A1 and above (outside C1 range) pass through unchanged.

## Invariants

1. The C1 predicate is `('\u{80}'..='\u{9f}').contains(&c)` in the escape_for_terminal
   implementation (terminal.rs:52).
2. The range boundary is inclusive on both ends: U+0080 escapes, U+009F escapes,
   U+00A0 does NOT escape.
3. `char::escape_default` is the escape function; it produces `\u{XX}` for codepoints
   without short-form escapes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | U+0085 (NEL) | Escaped to `\u{85}` |
| EC-002 | U+009B (CSI) | Escaped to `\u{9b}` |
| EC-003 | U+0080 (first C1) | Escaped to `\u{80}` |
| EC-004 | U+009F (last C1) | Escaped to `\u{9f}` |
| EC-005 | U+00A0 (NBSP, just past C1) | Passes through unchanged |
| EC-006 | U+007F (DEL, already C0) | Escaped by C0 branch, not C1 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| "line1\u{85}line2" | "line1\\u{85}line2" | happy-path |
| "before\u{9b}31mafter" | "before\\u{9b}31mafter" | happy-path |
| "\u{80}" | "\\u{80}" | edge-case |
| "\u{9f}" | "\\u{9f}" | edge-case |
| "\u{a0}" | "\u{a0}" unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | NEL and CSI are escaped | unit: escapes_c1_nel_and_csi (terminal.rs) |
| VP-TBD | Boundary values U+0080/U+009F escape; U+00A0 preserved | unit: escapes_c1_range_boundaries (terminal.rs) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- C1 escaping is the second tier of the terminal injection defense; terminals in DEC S8C1T mode treat U+009B as 8-bit ESC[, which attackers can exploit via SNI or URI bytes |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- escape_for_terminal is the sole C1 escape owner) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | S-TBD |
| Origin BC | BC-RPT-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (overall escape contract; this BC provides C1 boundary details)
- BC-2.11.008 -- composes with (U+00A0 preservation is confirmed here)
- BC-2.11.005 -- contrasts with (JsonReporter does NOT escape C1; this asymmetry is by design)

## Architecture Anchors

- `src/reporter/terminal.rs:52` -- C1 predicate in escape_for_terminal
- `src/reporter/terminal.rs:369-388` -- escapes_c1_nel_and_csi and escapes_c1_range_boundaries tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:52` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **guard clause**: `('\u{80}'..='\u{9f}').contains(&c)` at line 52
- **assertion**: escapes_c1_nel_and_csi, escapes_c1_range_boundaries (inline tests)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
