---
document_type: behavioral-contract
level: L3
version: "1.8"
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
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs range anchor: 196-218 → 203-226 (render_finding_prefix fn starts at 203, closes at 226); inline line refs updated: summary escape :197 → :204, evidence escape :215-216 → :222-223; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.5: issue-#259 F2 integrate (v0.8.0 collapse feature) — extend Invariant 3 and add Invariant 4; add EC-006 and EC-007 for collapse-interaction: when collapse_findings=true, evidence rendering for a collapsed group is bounded to at most K=3 representative lines per BC-2.11.027; escape_for_terminal invariant is unchanged and applies identically to each sampled evidence line through the same code path. Added cross-references BC-2.11.025/BC-2.11.027/BC-2.11.029. ADR-0003 (display-layer aggregation subsection) cited. — 2026-06-17"
  - "v1.6 2026-06-17: F2 adversarial pass-3 — fix Invariant 4 and EC-007: change false 'same call site in render_finding_prefix' claim to correct 'same escape_for_terminal FUNCTION' claim; the flat collapse wrapper calls escape_for_terminal directly, NOT via render_finding_prefix's evidence loop (F-F2X-01)"
  - "v1.7 2026-06-17: F2 adversarial pass-4 — F-F2-O01: anchor :203-226 → :203-227; Source Evidence path updated to :203-227"
  - "v1.8 2026-06-17: F2 adversarial pass-5 — F1: remove residual 'path-(b)' label from Invariant 4 body (BC-2.11.026 path-(b)) → 'The flat collapse wrapper calls...'"
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
4. Under the v0.8.0 collapse feature (BC-2.11.025, `collapse_findings = true`), evidence
   rendering for a collapsed group is bounded to at most K=3 representative lines (BC-2.11.027).
   The `escape_for_terminal` FUNCTION invariant is unchanged: every evidence line that IS
   rendered — whether from a collapsed group or a singleton — goes through `escape_for_terminal`.
   The flat collapse wrapper calls `escape_for_terminal` on each sampled
   evidence line directly (per BC-2.11.026 PC-4 observable line-order contract); it does NOT
   delegate to `render_finding_prefix`'s evidence loop,
   because that loop renders all entries of a single finding whereas the collapse path samples
   `evidence[0]` across up to K members from different findings (BC-2.11.027 positional model).
   The escape GUARANTEE is preserved — only the structural call path differs. The K-sample cap
   reduces the number of evidence lines rendered; it does not bypass or weaken the escape
   requirement for those that are rendered. The full `Finding.evidence` vec is never mutated
   (BC-2.11.029).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Control byte in summary, clean evidence | Summary escaped; evidence passes through |
| EC-002 | Clean summary, control byte in evidence[0] | Summary clean; evidence[0] escaped |
| EC-003 | Control bytes in both summary and evidence | Both escaped independently |
| EC-004 | Evidence is empty vec | No evidence lines rendered; no crash |
| EC-005 | Evidence with multiple entries, each with ESC | Each entry independently escaped |
| EC-006 | collapse_findings=true, group of N>3 findings, each with 1 evidence entry containing ESC | At most 3 evidence lines rendered; each of the 3 rendered lines is individually escaped; evidence from findings 4..N is elided from terminal but remains in raw slice (BC-2.11.027/BC-2.11.029) |
| EC-007 | collapse_findings=true, representative evidence line contains C1 codepoint | C1 codepoint escaped via the `escape_for_terminal` FUNCTION called directly in the collapse wrapper (NOT via render_finding_prefix's evidence loop); collapse path does not bypass the C1 escape — the function-level guarantee is preserved regardless of which code path calls it |

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
| VP-012 | Both summary and evidence are escaped | proptest: test_terminal_reporter_escapes_esc_bytes_in_summary (asserts both fields) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the requirement to escape both summary AND evidence is the complete terminal injection defense for Finding display |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- both sub-fields of Finding are raw data that must be escaped at display) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-077 |
| Origin BC | BC-RPT-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.007 -- composes with (BC-007 establishes the escape function; this BC establishes which fields it applies to)
- BC-2.11.011 -- composes with (analyzer-summary detail values are the third field class that gets escaped)
- BC-2.09.005 -- depends on (raw bytes in Finding.summary and Finding.evidence are what get escaped here)
- BC-2.11.025 -- composes with (v0.8.0 collapse: groups findings by key; this BC's escape invariant applies to each sampled evidence line in a collapsed group)
- BC-2.11.027 -- composes with (v0.8.0 collapse: evidence sampling bounds the evidence lines rendered; escape applies to each rendered line)
- BC-2.11.029 -- depends on (the raw Finding.evidence vec is never mutated by the collapse pass; escape operates on unmodified evidence strings)

## Architecture Anchors

- `src/reporter/terminal.rs:203-227` -- render_finding_prefix (escape applied to summary line 204 and evidence lines 222-223)

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:203-227` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_terminal_reporter_escapes_esc_bytes_in_summary (asserts both summary AND evidence are escaped)
- **type constraint**: escape_for_terminal called on both f.summary (line 204) and ev (line 223)

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
