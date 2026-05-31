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

# BC-2.11.005: JsonReporter Passes C1 Codepoints Through as Raw UTF-8

## Description

`JsonReporter` passes C1 control codepoints (U+0080-U+009F, e.g., NEL U+0085, CSI U+009B)
through as raw UTF-8 two-byte sequences in the JSON output. `serde_json` does not escape C1
codepoints because RFC 8259 only requires escaping C0 (U+0000-U+001F). A round-trip
(serialize then deserialize) recovers the original bytes exactly. This asymmetry is the
documented reason why `TerminalReporter` must apply its own C1 escape even on JSON-rendered
analyzer-summary detail values (BC-2.11.011).

## Preconditions

1. A `Finding` with C1 codepoints in `summary` or `evidence` is passed to
   `JsonReporter::render`.
2. The string contains valid UTF-8 encoding of C1 codepoints (e.g., U+009B encoded as
   the two-byte sequence 0xC2 0x9B).

## Postconditions

1. C1 codepoints (U+0080-U+009F) appear in the JSON output as raw UTF-8 byte sequences,
   NOT as backslash-uNNNN escape sequences.
2. The output is valid JSON per RFC 8259 (C1 are valid UTF-8 and RFC 8259 does not require
   their escaping).
3. A round-trip recovers the original bytes.

## Invariants

1. serde_json does NOT escape codepoints above U+001F that are valid UTF-8. C1 codepoints
   in the range U+0080-U+009F are valid two-byte UTF-8 and are passed through.
2. This differs from TerminalReporter which DOES escape C1 (BC-2.11.007/BC-2.11.009).
   The asymmetry is intentional per ADR 0003.
3. Downstream JSON consumers that feed the output into a terminal MUST apply C1 escaping
   themselves, or risk 8-bit terminal control interpretation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | U+009B (CSI) in summary | Two-byte UTF-8 0xC2 0x9B in JSON string; NOT a backslash-u escape |
| EC-002 | U+0085 (NEL) in evidence | Two-byte UTF-8 0xC2 0x85 in JSON string value |
| EC-003 | U+0080 (first C1) | Passes through as raw two-byte UTF-8 |
| EC-004 | U+009F (last C1) | Passes through as raw two-byte UTF-8 |
| EC-005 | U+00A0 (NBSP, just past C1) | Passes through as raw UTF-8 |
| EC-006 | C0 ESC (0x1B) + C1 CSI (U+009B) in same string | ESC becomes backslash-u001b; CSI is raw UTF-8 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| summary has U+009B (C1 CSI) | JSON output bytes contain 0xC2 0x9B, not the text backslash-u009b | happy-path |
| Round-trip for U+0085 (NEL) | Deserialized value equals original string | happy-path |
| summary has ESC (C0) followed by U+009B (C1) | ESC is a backslash-u escape; C1 CSI is raw bytes | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | C1 codepoints pass through as raw UTF-8 | unit: test_http_finding_c1_csi_in_json_reporter |
| — | Round-trip preserves C1 bytes | unit: test_http_finding_c1_csi_in_json_reporter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the C1 pass-through behavior is a documented asymmetry between JSON and terminal reporters that downstream consumers of the JSON API must understand |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- C1 bytes in JSON are the consumer's escaping responsibility) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | STORY-076 |
| Origin BC | BC-RPT-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.003 -- composes with (C0 escaping; C1 is the complementary non-escaped range)
- BC-2.11.009 -- contrasts with (TerminalReporter DOES escape C1)
- BC-2.11.011 -- related to (this asymmetry forces terminal reporter to re-escape JSON-rendered analyzer detail)

## Architecture Anchors

- `src/reporter/terminal.rs:29-46` -- module doc explaining why terminal must escape C1 even on JSON-rendered values
- `tests/reporter_json_tests.rs` -- test_BC_2_11_005_c1_passthrough_raw_utf8

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_http_finding_c1_csi_in_json_reporter
- **documentation**: terminal.rs module doc explains the C1 gap in serde_json

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed. The C1 pass-through is intentional: JSON consumers receive raw data
and are responsible for their own terminal safety.
