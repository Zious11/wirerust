---
document_type: behavioral-contract
level: L3
version: "1.4"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave-21 wave-level consistency lens — SS-11 reporter VP proof-method family harmonization (DF-SIBLING-SWEEP-001; sibling of the 2026-05-30 VP-020 correction): VP-012 VP-table Proof Method cells corrected unit→proptest; VP-012 proof_method=proptest is authoritative (unbounded Unicode input space) — 2026-05-30"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs test fn line anchors: fn escapes_c1_nel_and_csi at :368 → :375; fn escapes_c1_range_boundaries at :381 → :388; bounding range 367-389 kept (both fns remain within it); verified against HEAD cfe0112a — 2026-06-01"
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
| VP-012 | NEL and CSI are escaped | proptest: escapes_c1_nel_and_csi (terminal.rs) |
| VP-012 | Boundary values U+0080/U+009F escape; U+00A0 preserved | proptest: escapes_c1_range_boundaries (terminal.rs) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- C1 escaping is the second tier of the terminal injection defense; terminals in DEC S8C1T mode treat U+009B as 8-bit ESC[, which attackers can exploit via SNI or URI bytes |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- escape_for_terminal is the sole C1 escape owner) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (overall escape contract; this BC provides C1 boundary details)
- BC-2.11.008 -- composes with (U+00A0 preservation is confirmed here)
- BC-2.11.005 -- contrasts with (JsonReporter does NOT escape C1; this asymmetry is by design)

## Architecture Anchors

- `src/reporter/terminal.rs:52` -- C1 predicate in escape_for_terminal
- `src/reporter/terminal.rs:367-396` -- escapes_c1_nel_and_csi (fn at :375) and escapes_c1_range_boundaries (fn at :388) inline tests

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
