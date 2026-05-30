---
document_type: behavioral-contract
level: L3
version: "1.3"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.011: TerminalReporter Escapes Analyzer-Summary Detail Values

## Description

`TerminalReporter` applies `escape_for_terminal` to the string rendering of each value in
`AnalysisSummary.detail` before printing. Analyzer detail maps (e.g., TLS `top_snis`, HTTP
`recent_uris`) may contain attacker-controlled hostnames and URIs. `serde_json` renders these
values for display but passes C1 codepoints through as raw UTF-8 (BC-2.11.005); the terminal
reporter must therefore re-escape the rendered JSON string to close the C1 gap.

## Preconditions

1. `TerminalReporter::render` is processing at least one `AnalysisSummary`.
2. The `AnalysisSummary.detail` map has one or more key-value entries.
3. At least one value may contain attacker-controlled bytes (hostnames, URIs).

## Postconditions

1. Each detail value is converted to a string via `val.to_string()` (the `serde_json::Value`
   Display impl) then passed through `escape_for_terminal` before being rendered.
2. C0, DEL, C1, and backslash bytes in detail values are escaped in the terminal output.
3. The key names are NOT escaped (they are program-controlled, not attacker-controlled).

## Invariants

1. The escape call is at terminal.rs:172: `escape_for_terminal(&val.to_string())`.
2. The escaping applies even when `serde_json` has already partially escaped C0 bytes
   (double-escaping of C0 is acceptable because C0 appears as `\uNNNN` after serde_json,
   which is printable ASCII and harmless). The primary purpose is to catch C1 bytes that
   serde_json does NOT escape.
3. This is the third and final escape call site in TerminalReporter, after summary (BC-010)
   and evidence (BC-010).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Detail value with C1 CSI byte (U+009B) | CSI escaped to `\u{9b}` in output |
| EC-002 | Detail value with Cyrillic hostname | Cyrillic preserved unchanged |
| EC-003 | Detail value is a JSON number (no attacker bytes) | Number string passes through unchanged |
| EC-004 | Detail map is empty | Section still rendered; no crash |
| EC-005 | Detail key contains control bytes | Key is not escaped (program-controlled) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| detail["top_snis"] = Value::String("\u{9b}31m") | rendered as "top_snis: \\u{9b}31m" | happy-path |
| detail["count"] = Value::Number(42) | "count: 42" unchanged | happy-path |
| detail["host"] = Value::String("primer.rf") | "host: primer.rf" (Cyrillic preserved) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-012 | C1 in detail value is escaped | proptest: test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries |
| VP-012 | End-to-end HTTP analyzer detail with C1 escaped | proptest: test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- escaping analyzer-summary detail values completes the terminal injection defense for all three attacker-controlled output fields |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- analyzer detail maps carry raw attacker data that must be escaped at terminal) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.005 -- depends on (C1 pass-through in JsonReporter is the gap this BC closes)
- BC-2.11.007 -- composes with (escape_for_terminal function applies here too)
- BC-2.11.010 -- composes with (this BC completes the three-field escaping contract)

## Architecture Anchors

- `src/reporter/terminal.rs:164-174` -- analyzer summary detail loop with escape (`for (key, val)` at 164)
- `src/reporter/terminal.rs:165-171` -- inline comment explaining the C1 gap in serde_json
- `tests/reporter_tests.rs` -- test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:164-174` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries, test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter
- **documentation**: inline comment at lines 165-171 explaining the C1 gap rationale

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
