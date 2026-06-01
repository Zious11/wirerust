---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.4: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_077 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.5: DF-SIBLING-SWEEP-001 — fix stale terminal.rs range anchor: 196-218 → 203-226 (render_finding_prefix fn starts at 203, closes at 226); verified against HEAD cfe0112a — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.012: TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped

## Description

End-to-end integration contract: when the HTTP analyzer emits a path-traversal Finding whose
`summary` contains a C1 CSI codepoint (U+009B, as might appear in an obfuscated URI), the
`TerminalReporter` escapes that codepoint to `\u{9b}` in the terminal output. This BC
verifies the complete pipeline from Finding construction to terminal rendering, confirming
that the raw-data/display-layer contract (INV-4/ADR 0003) holds end-to-end.

## Preconditions

1. An HTTP path-traversal Finding has been constructed with `summary` containing U+009B
   (C1 CSI byte) as a codepoint embedded in a URI string.
2. `TerminalReporter::render` is called with this Finding in the findings slice.

## Postconditions

1. The rendered output contains `\u{9b}` in place of the raw U+009B codepoint.
2. No raw 0xC2 0x9B byte sequence (UTF-8 encoding of U+009B) appears in the output.
3. The path-traversal finding is otherwise rendered normally (category, verdict, confidence
   shown in the header line).

## Invariants

1. The Finding's `summary` carries raw bytes at construction time (INV-4).
2. The only escape call is in TerminalReporter::render_finding_prefix, not in the HTTP
   analyzer or Finding constructor.
3. This BC is the end-to-end regression test that confirms no escaping happened early
   in the pipeline.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | C1 CSI in summary of path-traversal finding | Escaped to `\u{9b}` |
| EC-002 | C1 CSI in evidence of path-traversal finding | Escaped to `\u{9b}` |
| EC-003 | C1 CSI + Cyrillic in same summary | CSI escaped; Cyrillic preserved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HTTP path-traversal Finding with summary="URI: /\u{9b}31m../etc/passwd" | output contains "URI: /\\u{9b}31m../etc/passwd" | happy-path |
| Same Finding rendered by JsonReporter | output contains raw 0xC2 0x9B (C1 NOT escaped by JSON) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-012 | End-to-end C1 escaping in terminal output | proptest: test_http_finding_c1_csi_escaped_by_terminal_reporter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this end-to-end BC confirms the raw-data/display-layer separation holds across the complete HTTP analyzer to terminal renderer pipeline |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- end-to-end verification of the contract) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-012 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (escape_for_terminal is the mechanism)
- BC-2.11.009 -- composes with (C1 range escaping is what fires here)
- BC-2.09.005 -- depends on (raw bytes in Finding are the prerequisite for this end-to-end test)
- BC-2.06.005 -- related to (path-traversal finding is the specific finding type tested)

## Architecture Anchors

- `src/reporter/terminal.rs:203-226` -- render_finding_prefix (the escape call site)
- `tests/reporter_terminal_tests.rs` -- mod story_077 :: test_BC_2_11_012_http_finding_c1_end_to_end

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_http_finding_c1_csi_escaped_by_terminal_reporter (end-to-end test)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed. This is a regression-style integration BC that verifies the
cross-layer contract; its scope deliberately spans SS-06 (HTTP) and SS-11 (reporting).
