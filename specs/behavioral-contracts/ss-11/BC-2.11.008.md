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

# BC-2.11.008: TerminalReporter Escape Preserves Printable ASCII and UTF-8

## Description

`escape_for_terminal` passes printable ASCII characters (0x20-0x7E, excluding backslash 0x5C)
and all non-ASCII Unicode codepoints outside the C1 range (U+00A0 and above) through
unchanged. A Cyrillic hostname, emoji, or CJK string in a Finding summary appears unmodified
in the terminal output. Only control bytes and backslash are escaped.

## Preconditions

1. `escape_for_terminal` is called with a string containing printable ASCII, Cyrillic,
   emoji, or other non-C0/non-C1 Unicode.

## Postconditions

1. All printable ASCII characters (0x20-0x7E except backslash 0x5C) pass through unchanged.
2. All Unicode codepoints U+00A0 and above pass through unchanged.
3. The output string length (in chars) equals the input length for a purely printable input.
4. Mixing printable and control bytes: only the control bytes are escaped; printable bytes
   are unaffected.

## Invariants

1. The escape predicate is `c.is_ascii_control() || ('\u{80}'..='\u{9f}').contains(&c) || c == '\\'`.
   Any character NOT matching this predicate is pushed unchanged.
2. U+00A0 (NBSP, Non-Breaking Space) is NOT escaped. It is above U+009F and not a C0/DEL
   control code. Tests verify this explicitly.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Pure printable ASCII "hello world" | "hello world" unchanged |
| EC-002 | Cyrillic "primer.rf" | Passes through unchanged |
| EC-003 | Emoji "crab rust" | Passes through unchanged |
| EC-004 | U+00A0 (NBSP) | Passes through unchanged (NOT escaped) |
| EC-005 | Mix: Cyrillic + ESC injection + emoji | Cyrillic and emoji preserved; ESC escaped |
| EC-006 | Empty string | Empty string returned |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| "hello world 123 !@#" | "hello world 123 !@#" | happy-path |
| "primer.rf" (Cyrillic) | identical Cyrillic string | happy-path |
| "crab rust" (emoji) | identical emoji string | happy-path |
| "\u{a0}" (NBSP) | "\u{a0}" unchanged | edge-case |
| "primer\x1b[31m" | "primer\\u{1b}[31m" (Cyrillic preserved, ESC escaped) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Printable ASCII passes through | unit: preserves_printable_ascii (terminal.rs) |
| VP-TBD | Cyrillic passes through | unit: preserves_cyrillic (terminal.rs) |
| VP-TBD | Emoji passes through | unit: preserves_emoji (terminal.rs) |
| VP-TBD | Mixed content escapes only dangerous bytes | unit: mixed_content_escapes_only_dangerous_bytes (terminal.rs) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- printable Unicode preservation is a forensic requirement; mangling hostnames or SNI values would destroy forensic evidence |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- escape_for_terminal must not mangle legitimate forensic data) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | S-TBD |
| Origin BC | BC-RPT-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (escape contract for control bytes; this BC covers the preservation side)
- BC-2.11.009 -- composes with (C1 boundary: U+00A0 preservation confirmed here)

## Architecture Anchors

- `src/reporter/terminal.rs:44-61` -- escape_for_terminal implementation (else branch)
- `src/reporter/terminal.rs:329-388` -- inline unit tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:44-61` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: preserves_printable_ascii, preserves_cyrillic, preserves_emoji, mixed_content_escapes_only_dangerous_bytes (inline tests in terminal.rs)

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
