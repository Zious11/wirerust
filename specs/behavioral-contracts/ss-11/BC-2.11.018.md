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

# BC-2.11.018: TerminalReporter Colorization: Likely/High=Red Bold, etc.

## Description

When `TerminalReporter.use_color = true`, finding header lines are colorized by the
combination of `Verdict` and `Confidence` using `owo_colors`: `Likely + High` produces red
bold; `Likely + other` produces yellow; `Inconclusive` produces cyan; `Unlikely` produces
dimmed. When `use_color = false`, plain text is rendered without ANSI codes. Colorization
applies to the per-finding header line only; evidence lines, section headers, and the skipped-
packets warning use their own color rules.

## Preconditions

1. `TerminalReporter.use_color = true` (color mode enabled).
2. A finding is being rendered with `render_finding_prefix`.

## Postconditions

1. Likely + High: finding header line contains red + bold ANSI codes.
2. Likely + (Medium or Low): finding header line contains yellow ANSI codes.
3. Inconclusive (any confidence): finding header line contains cyan ANSI codes.
4. Unlikely (any confidence): finding header line contains dimmed ANSI codes.
5. When `use_color = false`: no ANSI codes in output; plain text only.
6. Skipped-packets warning is rendered in yellow when `use_color = true` and
   `skipped_packets > 0` (separate from finding colorization).

## Invariants

1. Colorization is implemented via `owo_colors::OwoColorize` trait methods.
2. Tests run with `use_color = false` to avoid ANSI code assertions in test output.
   Color behavior is visually verified, not byte-asserted (ADR-0003 amendment recommended;
   BC-RPT-018 KEEP-MEDIUM rationale).
3. Section headers (`section()` method) are bold + underline when `use_color = true`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Likely/High with use_color=true | Red bold ANSI codes in output |
| EC-002 | Likely/Medium with use_color=true | Yellow ANSI codes |
| EC-003 | Inconclusive/High with use_color=true | Cyan ANSI codes |
| EC-004 | Unlikely with use_color=true | Dimmed ANSI codes |
| EC-005 | Any verdict with use_color=false | No ANSI codes; plain text |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding Likely/High, use_color=false | Plain text header, no ANSI | happy-path |
| Finding Inconclusive/Medium, use_color=false | Plain text, no ANSI | happy-path |
| (Visual verification only) Finding Likely/High, use_color=true | Red bold rendering | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Color rules are visually correct | manual / visual review (no byte assertion per KEEP-MEDIUM rationale) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- colorization is a terminal output presentation feature of the reporting capability |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-018 (pass-3 ingestion corpus, MEDIUM confidence -- tests run with use_color=false; ANSI assertion cost-prohibitive per P3 R4 KEEP-MEDIUM rationale) |

## Related BCs

- BC-2.11.019 -- composes with (section order governs where colored finding lines appear)

## Architecture Anchors

- `src/reporter/terminal.rs:203-213` -- Verdict/Confidence colorization match in render_finding_prefix
- `src/reporter/terminal.rs:99-103` -- skipped_packets yellow colorization
- `src/reporter/terminal.rs:183-186` -- section() bold+underline colorization

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:203-213` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: source code at lines 203-213 clearly shows the color mapping
- **inferred**: tests run with use_color=false; no test asserts ANSI color byte sequences

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

If formal color testing is needed: consider capturing raw output bytes with use_color=true
and asserting the presence of specific ANSI escape sequences. ADR-0003 amendment recommended
to document color rules as "visual-only, not byte-tested."
