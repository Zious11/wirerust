---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/json.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_json_tests.rs formalization (F-W22-BC-ANCHOR) — 2026-05-31"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.004: JsonReporter Preserves Non-ASCII Unicode in Readable Form

## Description

`JsonReporter` preserves non-ASCII Unicode codepoints (Cyrillic, CJK, emoji, etc.) in their
readable UTF-8 form in JSON output -- they are NOT converted to `\uNNNN` escape sequences.
`serde_json` by default emits printable non-ASCII Unicode as raw UTF-8. A Cyrillic hostname
like `primer.rf` (Cyrillic) appears in the JSON output literally, not as `\u...` escapes.

## Preconditions

1. A `Finding` with non-ASCII Unicode (e.g., Cyrillic, CJK, emoji) in `summary` or
   `evidence` is passed to `JsonReporter::render`.
2. serde_json is configured with default settings (not `serde_json::to_string` with
   explicit ASCII-only mode).

## Postconditions

1. Cyrillic, CJK, and emoji characters appear as raw UTF-8 in the JSON output, not as
   `\u` escape sequences.
2. The output remains valid JSON per RFC 8259.
3. A JSON parser (any standard RFC 8259 implementation) can decode the output and recover
   the original Unicode string.

## Invariants

1. serde_json's default serializer (`to_string_pretty`) does NOT escape printable
   non-ASCII Unicode. This is a library-level guarantee.
2. TerminalReporter preserves these same characters (BC-2.11.008); the two reporters
   are consistent on Unicode preservation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cyrillic hostname in summary | "primer.rf" appears as literal UTF-8, not \u{43f}... |
| EC-002 | Emoji in evidence | "crab emoji" appears as literal UTF-8 |
| EC-003 | CJK characters | Preserved as raw UTF-8 |
| EC-004 | Mix of ASCII and Cyrillic | ASCII stays ASCII; Cyrillic stays Cyrillic |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| summary = "primer.rf" (Cyrillic) | JSON contains literal Cyrillic bytes, not \u escapes | happy-path |
| summary = "crab 🦀 rust" | JSON contains literal emoji character | happy-path |
| summary = "normal ASCII" | "normal ASCII" unchanged in JSON | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Cyrillic preserved as readable Unicode | unit: test_json_reporter_preserves_cyrillic_as_readable_unicode |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- preserving non-ASCII Unicode in JSON output is essential for forensic readability of internationalized hostnames and payloads |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- non-ASCII forensic data must not be lossy-encoded) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | STORY-076 |
| Origin BC | BC-RPT-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.003 -- composes with (C0 escaping is the complement of Unicode preservation)
- BC-2.11.008 -- related to (TerminalReporter similarly preserves printable Unicode)
- BC-2.09.005 -- depends on (raw bytes at Finding construction flow through)

## Architecture Anchors

- `src/reporter/json.rs:23-60` -- serde_json::to_string_pretty call
- `tests/reporter_json_tests.rs` -- test_BC_2_11_004_cyrillic_preserved_readable

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_json_reporter_preserves_cyrillic_as_readable_unicode
- **inferred**: serde_json default behavior documented in crate docs

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
