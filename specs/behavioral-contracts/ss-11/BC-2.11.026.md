---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-17T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.8.0
modified: ["v1.1 2026-06-17: F2 adversarial pass-1 — relax suffix colorization: (xN) suffix IS colorized with the header line (no seam for uncolorized suffix in render_finding_prefix); update Invariant 4, PC-4, EC-008 (F-259-02)"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.026: Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix

## Description

After the collapse pass (BC-2.11.025), each display group is rendered with a count-annotated
header line. When a group has N≥2 members, the header line appends a ` (xN)` suffix directly
after the summary text, producing the format:
`  [Category] VERDICT (CONFIDENCE) - summary (xN)`. When a group has exactly N=1 member
(singleton), the header line is rendered without any count suffix — the output is byte-identical
to the pre-v0.8.0 single-finding rendering for that finding. This rule ensures that singletons
remain visually indistinguishable from the pre-collapse output, and that analysts can immediately
identify repeated findings from the ` (xN)` annotation alone.

The `(xN)` suffix is appended to the already-escaped summary string produced by
`escape_for_terminal`. The parenthetical suffix text itself is hardcoded and contains no
attacker-controlled bytes; it does not require additional escaping.

## Preconditions

1. `TerminalReporter.collapse_findings = true`.
2. `TerminalReporter.show_mitre_grouping = false` (flat mode).
3. The collapse pass (BC-2.11.025) has grouped the findings slice into display groups, each with
   a count N (the number of findings in the group).
4. The `escape_for_terminal` function has been applied to the group's representative `summary`
   field before the suffix is appended.

## Postconditions

1. For a display group with N≥2: the header line for that group reads:
   `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n`
   where `<N>` is the exact integer count of findings in the group (rendered as a decimal
   integer with no leading zeros, no space between `x` and `N`).
2. For a display group with N=1 (singleton): the header line reads:
   `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary>\n`
   No ` (xN)` suffix, no ` (x1)` suffix. Output is byte-identical to the pre-v0.8.0
   single-finding rendering produced by `render_finding_prefix`.
3. The count value `N` equals `Vec.len()` of the findings grouped under that key; it is
   always a positive integer (N≥1 by construction; empty groups are never created).
4. The ` (xN)` suffix is appended after `escaped_summary`, forming part of the header line
   before colorization. The suffix is colorized identically to the rest of the header line:
   `render_finding_prefix` (or its collapse-aware extension) appends ` (xN)` to the
   pre-color `line` string when N≥2, and the ENTIRE line including the suffix is then
   colorized together via the verdict/confidence color match. There is no seam in the current
   `render_finding_prefix` implementation for an uncolorized suffix. The implementation path:
   either `render_finding_prefix` receives a count parameter and builds `line` as
   `"  [Cat] VERDICT (CONF) - {escaped_summary} (xN)"` before colorizing, or a
   collapse-aware sibling function builds the line with the suffix before the color match.
5. The count suffix is not subject to `escape_for_terminal`; it is a hardcoded format string
   and contains no attacker-controlled content.

## Invariants

1. The suffix format is ` (x<N>)` — space, open-paren, literal `x`, decimal integer, close-paren.
   Examples: ` (x2)`, ` (x3142)`, ` (x10000)`. There are no alternative formats.
2. Singleton groups (N=1) produce no suffix. The absence of a suffix for singletons is
   intentional: it avoids noise and preserves backward compatibility for unique findings.
3. The count is computed as the exact group size from the collapse pass; it is never rounded,
   truncated, or represented as a range.
4. Color styling (from `use_color`) is applied to the COMPLETE header line including the
   ` (xN)` suffix. The ` (xN)` suffix is appended to the pre-color `line` string BEFORE the
   color match is applied, so it is colorized identically to the summary text. The
   `render_finding_prefix` implementation builds `line` atomically — colorizing the whole
   thing — with no seam to attach an uncolorized suffix after the color codes are closed.
5. Evidence lines (from `Finding.evidence`) are rendered after the header line, under the
   same count-annotated header. Evidence sampling rules are governed by BC-2.11.027.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group with N=1 (singleton) | Header line has NO count suffix; byte-identical to pre-v0.8.0 output |
| EC-002 | Group with N=2 | Header line ends with ` (x2)` before newline |
| EC-003 | Group with N=3142 | Header line ends with ` (x3142)` |
| EC-004 | Group with N=10000 | Header line ends with ` (x10000)` (no truncation, no abbreviation) |
| EC-005 | Summary ends with whitespace before suffix | Suffix appended directly after the whitespace; one space before `(x...)`; result may have double space — acceptable, no trimming |
| EC-006 | collapse_findings=false (opt-out) | No collapse pass runs; no count suffix on any finding; behavior per BC-2.11.028 |
| EC-007 | show_mitre_grouping=true | Collapse pass not applied; no count suffix regardless of group sizes |
| EC-008 | Group with N=2 and use_color=true | Complete header line (including ` (x2)` suffix) colored per verdict/confidence — the suffix is part of the pre-color `line` string and is colorized together with the summary text |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 findings all `(Anomaly, Inconclusive, Low, "Empty UA")`, collapse_findings=true | Header line contains `"Empty UA (x3)"` | happy-path (count display) |
| 1 finding `(Anomaly, Inconclusive, Low, "Empty UA")`, collapse_findings=true | Header line contains `"Empty UA"` with no `(x1)` suffix | happy-path (singleton) |
| 3142 findings all `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` | Header line ends with `"Empty User-Agent header (x3142)"` | happy-path (large count) |
| 1 unique finding + 5 identical findings | Unique finding: no suffix; identical group: `(x5)` suffix | mixed scenario |
| 2 findings same key, use_color=false | Header `"  [Anomaly] INCONCLUSIVE (LOW) - Empty UA (x2)\n"` | happy-path (no color) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | N≥2 group produces (xN) suffix with correct count | unit: test_BC_2_11_026_count_suffix_for_n_ge_2 |
| — | N=1 singleton produces no count suffix | unit: test_BC_2_11_026_singleton_no_suffix |
| — | Count is exact integer (no rounding or abbreviation) | unit: test_BC_2_11_026_large_count_exact |
| — | Suffix format is space-paren-x-integer-paren | unit: test_BC_2_11_026_suffix_format |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC defines the count-annotation rendering format that makes the collapse feature human-readable and useful in the terminal output; the ` (xN)` suffix is the direct output contract of the Reporting capability for repeated findings |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- count computation occurs at display time in the terminal reporter; the raw Finding slice carries no count field) |
| Architecture Module | SS-11 (reporter/terminal.rs) |
| Stories | STORY-118 |
| Issue | #259 (Collapse repeated low-value findings) |
| ADR | ADR-0003 (display-layer aggregation subsection) |

## Related BCs

- BC-2.11.025 -- depends on (collapse pass produces groups with counts; this BC governs how the count is rendered)
- BC-2.11.027 -- composes with (evidence lines rendered under the count-annotated header)
- BC-2.11.028 -- depends on (opt-out disables collapse, which removes count suffix from all output)
- BC-2.11.010 -- composes with (escape_for_terminal applied to summary before suffix appended)

## Architecture Anchors

- `src/reporter/terminal.rs:203-226` -- render_finding_prefix (summary escape + header line construction; the (xN) suffix is appended here or in a new wrapper function for collapsed groups)
- `src/reporter/terminal.rs:149-160` -- FINDINGS dispatch block (flat path where count-annotated render replaces the direct render_finding_flat call when collapse=true)

## Story Anchor

STORY-118

## VP Anchors

- — (new VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — count is Vec.len(); suffix format is a constant string |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
