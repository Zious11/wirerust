---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.003: JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde

## Description

`JsonReporter` performs no manual escaping of Finding content. Instead, it delegates all
escaping to `serde_json`, which per RFC 8259 escapes C0 control bytes (U+0000-U+001F) as
`\uNNNN` sequences. The serialized JSON round-trips correctly: a consumer reading the JSON
and unescaping will recover the original raw bytes.

## Preconditions

1. A `Finding` with C0 control bytes in `summary` or `evidence` is passed to
   `JsonReporter::render`.
2. The `serde_json` crate is the serializer (no custom `Serialize` impl on `Finding.summary`).

## Postconditions

1. C0 control bytes (0x00-0x1F) in Finding.summary or evidence appear as `\uNNNN` escape
   sequences in the JSON output string (e.g., ESC 0x1B becomes the six-character sequence
   backslash-u-0-0-1-b in the JSON text).
2. DEL (0x7F) is NOT escaped by serde_json (above the C0 range); raw 0x7F in JSON value.
3. The JSON output is valid per RFC 8259.
4. A round-trip (serialize then deserialize) recovers the original bytes exactly.

## Invariants

1. JsonReporter NEVER calls `escape_for_terminal`. That function is TerminalReporter-only
   per ADR 0003 / INV-4.
2. serde_json escapes C0 (0x00-0x1F) as `\uXXXX` but passes DEL (0x7F) and C1 (0x80-0x9F)
   through as raw UTF-8. This asymmetry is the documented difference from TerminalReporter.
3. This behavior is deterministic: identical input always produces identical JSON.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ESC (0x1B) in summary | JSON contains the text backslash-u-001b |
| EC-002 | NUL (0x00) in summary | JSON contains the text backslash-u-0000 |
| EC-003 | DEL (0x7F) in summary | Passes through as raw 0x7F (NOT a backslash-u escape) |
| EC-004 | C1 codepoint U+009B in summary | Passes through as raw UTF-8 two-byte sequence |
| EC-005 | Backslash in summary | serde_json escapes to double-backslash |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| summary has ESC (0x1B) followed by [31m | JSON string literal contains backslash-u001b[31m | happy-path |
| summary has NUL (0x00) | JSON string literal contains backslash-u0000 | happy-path |
| summary has backslash | JSON contains double-backslash | happy-path |
| summary has DEL (0x7F) | JSON contains raw 0x7F byte (no escape) | edge-case |
| Round-trip: serialize then deserialize | Recovered bytes equal original | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-017 | C0 bytes become backslash-uNNNN in JSON output | unit: test_output_sanitization_layering_contract |
| VP-017 | Round-trip recovers original bytes | unit: test_output_sanitization_layering_contract |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC defines the JSON escaping contract that downstream tooling relies on to safely decode attacker-controlled bytes from packet payloads |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- serde_json is the display layer for JSON; no escape at Finding construction) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | S-TBD |
| Origin BC | BC-RPT-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- contrasts with (TerminalReporter uses different escape set: C0+DEL+C1+backslash)
- BC-2.11.005 -- composes with (C1 pass-through is the complementary behavior)
- BC-2.09.005 -- depends on (raw bytes stored in Finding are what get escaped here)

## Architecture Anchors

- `src/reporter/json.rs:5-12` -- module doc explaining the no-manual-escaping contract
- `docs/adr/0003-reporting-pipeline-layering.md` -- ADR establishing this separation

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_output_sanitization_layering_contract (round-trip test)
- **documentation**: module docstring explicitly states "No escaping is performed here"

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
