---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/findings.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
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

# BC-2.09.005: Finding.summary and Evidence Store RAW Post-from_utf8_lossy Bytes per ADR 0003

## Description

The raw-data contract (ADR 0003, INV-4): `Finding.summary` and `Finding.evidence` fields carry
raw bytes obtained from `String::from_utf8_lossy()` on attacker-controlled input. No escape
function is applied at the analyzer layer. `escape_for_terminal` is called ONLY by
`TerminalReporter`. `JsonReporter` delegates escaping to serde_json (RFC 8259). This ensures
that SIEM consumers of JSON output see the original attacker bytes, not an escaped form.

## Preconditions

1. An analyzer (HttpAnalyzer, TlsAnalyzer, or TcpReassembler) is constructing a Finding.
2. The Finding's summary or evidence fields contain attacker-controlled bytes (e.g., a URI,
   SNI hostname, or payload excerpt).

## Postconditions

1. `Finding.summary` is a `String` containing the post-`from_utf8_lossy` bytes without any
   additional escaping.
2. `Finding.evidence` is a `Vec<String>` with the same guarantee.
3. `escape_for_terminal` is NOT called at any Finding construction site.
4. JSON output produced by `JsonReporter` contains the raw bytes (escaped only per RFC 8259
   by serde_json, which is transparent to the forensic consumer).

## Invariants

1. `escape_for_terminal` has exactly ONE call site in production code: inside `TerminalReporter`.
   Any other call site is a violation of ADR 0003.
2. The compiler does NOT enforce this. Any analyzer that calls `escape_for_terminal` at
   construction time violates the invariant silently.
3. `from_utf8_lossy` is the only transformation applied: invalid UTF-8 sequences are replaced
   by U+FFFD replacement characters, but otherwise bytes are preserved as-is.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI contains ESC byte (0x1B) | Finding.summary contains literal 0x1B byte; JSON output encodes as  |
| EC-002 | SNI hostname contains C0 control bytes | Raw bytes in Finding.summary; TerminalReporter escapes on display |
| EC-003 | HTTP header contains non-UTF-8 bytes | from_utf8_lossy replaces invalid sequences with U+FFFD |
| EC-004 | Evidence contains newline | Raw newline preserved; JsonReporter encodes as \n via serde |
| EC-005 | HTTP URI contains `{:?}` Debug-format bytes | Never happens: analyzers use from_utf8_lossy, not Debug format |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HTTP finding with ESC byte in URI | finding.summary contains 0x1B (not \\u001b) | happy-path |
| TLS finding with non-UTF-8 SNI | finding.summary contains from_utf8_lossy output (U+FFFD for invalid bytes) | happy-path |
| TerminalReporter renders finding with ESC byte | Output contains \\u{1b} or similar escape form | integration |
| JsonReporter renders finding with ESC byte | Output contains  (RFC 8259 serde encoding) | integration |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Finding.summary contains raw C0 bytes (not escaped form) at construction | unit: test_non_utf8_sni_preserves_raw_bytes_in_summary |
| VP-TBD | escape_for_terminal has exactly one production call site | manual/grep: assert not called in any analyzer |
| VP-TBD | JSON output of finding with ESC byte produces  (serde) not escaped with \\u{1b} | unit: test_output_sanitization_layering_contract |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 -- the raw-data contract is the foundational invariant of the Finding type's data preservation guarantee |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-09 (findings.rs:120-145, C-10; all analyzer emission sites) |
| Stories | S-TBD |
| Origin BC | BC-FND-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (TerminalReporter is the sole escape caller)
- BC-2.11.003 -- composes with (JsonReporter uses serde RFC 8259 escaping)
- BC-2.07.020 -- specific instance (TLS SNI preserves raw bytes)
- BC-2.06.026 -- specific instance (HTTP header bytes preserved)

## Architecture Anchors

- `src/findings.rs:120` -- `pub struct Finding` definition
- `src/findings.rs:124-125` -- `pub summary: String`, `pub evidence: Vec<String>` fields
- `src/findings.rs:155-156` -- `See ADR 0003` doc comment on Display impl
- `src/reporter/terminal.rs:44` -- `fn escape_for_terminal(s: &str) -> String` -- the sole production call site
- `tests/reporter_tests.rs` -- test_output_sanitization_layering_contract

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:120-145` (struct + serde attrs), `:155-156` (ADR 0003 doc comment) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **documentation**: ADR 0003 doc comment in findings.rs
- **assertion**: test_output_sanitization_layering_contract, test_non_utf8_sni_preserves_raw_bytes_in_summary

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (construction of struct value) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (Finding is owned value) |
| **Overall classification** | pure (Finding construction is a pure data operation) |

## Refactoring Notes

The enforcement is by convention only (doc comment). Formal enforcement would require either:
a) a newtype wrapping String that can only be constructed from raw bytes, or
b) a lint rule checking call sites of escape_for_terminal.
Neither exists today; the convention is maintained by code review.
