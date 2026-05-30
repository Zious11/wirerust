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

# BC-2.11.007: TerminalReporter Escapes C0+DEL+C1+Backslash in Finding Summary and Evidence

## Description

`TerminalReporter` calls `escape_for_terminal(s)` on all user-controlled string content before
printing. The function escapes: all C0 control bytes (0x00-0x1F) including TAB, LF, and CR
(these become `\t`, `\n`, `\r` respectively via `char::escape_default`), DEL (0x7F), the
entire C1 range (0x80-0x9F inclusive, including NEL U+0085 and CSI U+009B -- no exceptions
within this range), and backslash.
This prevents terminal injection attacks from attacker-controlled bytes in Finding summaries.

Note: BC-2.11.009 specifies the precise C1 range boundary; this BC covers the overall escaping
contract.

## Preconditions

1. A Finding with attacker-controlled bytes in `summary` or `evidence` is being rendered.
2. TerminalReporter is selected (not JsonReporter or CsvReporter).

## Postconditions

1. All C0 bytes (0x00-0x1F) are replaced with escape sequences via `char::escape_default`:
   ESC (0x1B) -> `\u{1b}`, NUL (0x00) -> `\u{0}`, TAB (0x09) -> `\t`, LF (0x0A) -> `\n`,
   CR (0x0D) -> `\r`. No C0 byte passes through raw.
2. DEL (0x7F) is replaced with an escape sequence (`\u{7f}`).
3. C1 bytes (0x80-0x9F) inclusive are replaced -- including NEL (U+0085). No C1 exception
   exists; the entire C1 range is escaped by the `('\u{80}'..='\u{9f}').contains(&c)` predicate
   at `terminal.rs:52`.
4. Backslash (0x5C) is replaced with `\\`.
5. Printable ASCII (0x20-0x7E, excluding 0x5C backslash), and UTF-8 codepoints U+00A0 and above
   (Cyrillic, emoji, CJK, etc.) pass through unmodified.
6. The escaped output contains NO raw C0, DEL, or C1 bytes.

## Invariants

1. `escape_for_terminal` has exactly ONE production call site: inside TerminalReporter.
   This is the architectural contract of ADR 0003 / INV-4.
2. JsonReporter does NOT call escape_for_terminal; it uses serde_json which applies RFC 8259
   escaping (a different set from terminal escaping).
3. Escaping is applied to BOTH `Finding.summary` AND each entry in `Finding.evidence`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ESC byte (0x1B) in summary | Escaped to `\u{1b}` or similar; raw 0x1B not printed |
| EC-002 | BEL byte (0x07) in evidence | Escaped |
| EC-003 | DEL byte (0x7F) | Escaped |
| EC-004 | Backslash in summary | Escaped to `\\` |
| EC-005 | Cyrillic characters in summary | Passed through unescaped |
| EC-006 | Emoji in summary | Passed through unescaped |
| EC-007 | CR (0x0D) in summary | Escaped to `\r` (char::escape_default short form) |
| EC-008 | LF (0x0A) in summary | Escaped to `\n` (char::escape_default short form) |
| EC-009 | C1 CSI byte (0x9B) in summary | Escaped (closes gap for terminal CSI injection) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| summary = "evil\x1b[31mtext" | rendered as "evil\u{1b}[31mtext" (no raw ESC) | happy-path |
| evidence = ["path: /\x00foo"] | rendered as ["path: /\u{0}foo"] (no raw NUL) | happy-path |
| summary = "clean text" | "clean text" unchanged | happy-path |
| summary = "caf\xc3\xa9" (UTF-8 for café) | "café" unchanged | edge-case |
| summary = "back\\slash" | "back\\\\slash" (escaped) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-012 | No raw C0 byte appears in TerminalReporter output | proptest: test_terminal_reporter_escapes_esc_bytes_in_summary |
| VP-012 | Printable ASCII and UTF-8 pass through unchanged | proptest: inline tests in terminal.rs |
| VP-012 | Both summary AND evidence are escaped | proptest: test_terminal_reporter_escapes_esc_bytes_in_summary asserts both |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and output") per domain/capabilities/cap-11-reporting-output.md -- terminal escaping is the display-layer safety contract that completes the raw-data pipeline |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation -- TerminalReporter is the sole escape owner) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.005 -- depends on (raw bytes at Finding construction are what get escaped here)
- BC-2.11.008 -- composes with (printable Unicode preservation)
- BC-2.11.009 -- composes with (C1 range boundary details)
- BC-2.11.010 -- composes with (escaping applied to both summary AND evidence)
- BC-2.11.011 -- composes with (escaping applied to analyzer-summary detail values)

## Architecture Anchors

- `src/reporter/terminal.rs:44` -- `fn escape_for_terminal(s: &str) -> String` implementation
- `src/reporter/terminal.rs:197` -- call site in `render_finding_prefix`: `escape_for_terminal(&f.summary)`
- `src/reporter/terminal.rs:216` -- call site for evidence: `escape_for_terminal(ev)` per finding evidence entry
- `src/reporter/terminal.rs:172` -- call site in analyzer-summary detail: `escape_for_terminal(&val.to_string())`
- `tests/reporter_tests.rs` -- test_terminal_reporter_escapes_esc_bytes_in_summary

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: test_terminal_reporter_escapes_esc_bytes_in_summary; 11 inline tests in terminal.rs

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to output string |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (TerminalReporter is stateless) |
| **Overall classification** | pure (escape_for_terminal is a pure string transformation) |
