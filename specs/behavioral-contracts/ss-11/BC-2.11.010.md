---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.010: TerminalReporter Escapes Both Summary AND Each Evidence Line

## Description

`TerminalReporter` applies `escape_for_terminal` to BOTH `Finding.summary` AND to each
individual entry in `Finding.evidence`. Neither field is printed raw to the terminal. This
ensures that attacker-controlled bytes embedded in any part of a Finding -- the primary
summary line or any supporting evidence detail -- cannot inject terminal control sequences.

## Preconditions

1. `TerminalReporter::render` is processing a `Finding` that has both `summary` and
   `evidence` populated.
2. Both fields may contain attacker-controlled bytes.

## Postconditions

1. `Finding.summary` is passed through `escape_for_terminal` before being included in
   the rendered output line.
2. Each string in `Finding.evidence` is independently passed through `escape_for_terminal`
   before being rendered.
3. No raw C0/DEL/C1/backslash byte from either field appears in the output.

## Invariants

1. Escaping is applied in `render_finding_prefix` at terminal.rs (the shared rendering helper
   used by both `render_finding_flat` and `render_finding_grouped`).
2. Both the summary call site and the evidence loop call site use the same `escape_for_terminal`
   function.
3. This contract applies in both default (flat) and MITRE-grouped rendering modes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Control byte in summary, clean evidence | Summary escaped; evidence passes through |
| EC-002 | Clean summary, control byte in evidence[0] | Summary clean; evidence[0] escaped |
| EC-003 | Control bytes in both summary and evidence | Both escaped independently |
| EC-004 | Evidence is empty vec | No evidence lines rendered; no crash |
| EC-005 | Evidence with multiple entries, each with ESC | Each entry independently escaped |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| summary="\x1bESC", evidence=["clean"] | Summary escaped; evidence untouched | happy-path |
| summary="clean", evidence=["\x1bESC"] | Summary clean; evidence escaped | happy-path |
| summary="\x1b", evidence=["\x1b", "\x07"] | Both escaped; two evidence lines | happy-path |
| summary="normal", evidence=[] | Summary rendered; no evidence lines | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-012 | Both summary and evidence are escaped | unit: test_terminal_reporter_escapes_esc_bytes_in_summary (asserts both fields) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- the requirement to escape both summary AND evidence is the complete terminal injection defense for Finding display |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- both sub-fields of Finding are raw data that must be escaped at display) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (BC-007 establishes the escape function; this BC establishes which fields it applies to)
- BC-2.11.011 -- composes with (analyzer-summary detail values are the third field class that gets escaped)
- BC-2.09.005 -- depends on (raw bytes in Finding.summary and Finding.evidence are what get escaped here)

## Architecture Anchors

- `src/reporter/terminal.rs:196-218` -- render_finding_prefix (escape applied to summary line 197 and evidence lines 215-216)

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:196-218` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_terminal_reporter_escapes_esc_bytes_in_summary (asserts both summary AND evidence are escaped)
- **type constraint**: escape_for_terminal called on both f.summary (line 197) and ev (line 216)

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
