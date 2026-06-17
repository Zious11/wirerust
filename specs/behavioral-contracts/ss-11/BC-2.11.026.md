---
document_type: behavioral-contract
level: L3
version: "1.8"
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
modified: ["v1.1 2026-06-17: F2 adversarial pass-1 — relax suffix colorization: (xN) suffix IS colorized with the header line (no seam for uncolorized suffix in render_finding_prefix); update Invariant 4, PC-4, EC-008 (F-259-02)", "v1.2 2026-06-17: F2 adversarial pass-2 — path-(b) collapse-aware wrapper prescribed as canonical in PC-4 (F-A03); dispatch anchor 149-160→149-162 (F-A05); EC-005 test vector added (F-A06)", "v1.3 2026-06-17: F2 adversarial pass-3 — add evidence emission sentence to PC-4 (F-F2X-03); fix EC row order EC-009/EC-008 → EC-008/EC-009 monotonic (F-F2X-02); fix arch anchor: remove stale 'appended here' alternative", "v1.4 2026-06-17: F2 adversarial pass-4 — F-F2-A01: convert PC-4 from internal-call-structure prescription to observable-behavior contract; remove 'path-(b) function-call graph' normative language; add non-normative implementation note; F-F2-O01: anchor :203-226 → :203-227; update EC-007 STRUCTURAL guarantee to observable-behavior form", "v1.5 2026-06-17: F2 adversarial pass-5 — F1: remove residual 'path-(b) separation' label from EC-009 body; reword to observable-behavior form", "v1.6 2026-06-17: F2 adversarial passes 6-8 — LOW-1: add red-bold (Likely/High) canonical test vector to confirm (xN) suffix is inside the red-bold colorization span for that branch", "v1.7 2026-06-17: F2 adversarial pass-9 — F-PA-01: add explicit normative PC-6 color-ladder requirement: same Likely+High→red().bold()/Likely+other→yellow/Possible→yellow/Inconclusive→cyan/Unlikely→dimmed logic applied to pre-suffix string BEFORE colorization; appending suffix after ANSI reset is NON-CONFORMANT", "v1.8 2026-06-17: F2 adversarial passes 12-14 — F-PA-A01: define 'representative finding' for N≥2 groups: group_members[0] (first member in emission order); add PC-7 (MITRE line sources group_members[0].mitre_techniques; other members' MITRE elided from terminal); add canonical test vector for divergent-mitre case"]
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
   single-finding output for that finding.
3. The count value `N` equals `Vec.len()` of the findings grouped under that key; it is
   always a positive integer (N≥1 by construction; empty groups are never created).
4. **OBSERVABLE LINE ORDER — COLLAPSE PATH:** For a collapsed group the terminal emits, in
   order: (1) the header line `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary>
   (x<N>)\n` (colorized, suffix included when N≥2), (2) up to K=3 sampled evidence lines
   each passed through `escape_for_terminal` (BC-2.11.027), (3) the MITRE line using
   `mitre_techniques.join(", ")` format IF `mitre_techniques` is non-empty (BC-2.11.017 PC-6).
   The ` (xN)` suffix MUST NOT appear on the MITRE line or any evidence line — it appears
   only on the header line. The ` (xN)` suffix is colorized identically with the rest of the
   header line (suffix is part of the pre-colorization string). The grouped/`--mitre` path
   MUST NOT emit a ` (xN)` suffix on any finding, regardless of group size (BC-2.11.013
   Invariant 4). **Implementation note (non-normative; F4 decides):** F4 MAY build the
   header inline in a collapse-aware wrapper, factor a shared helper, or use any other
   internal call graph — provided the observable line order above holds, the ` (xN)` suffix
   appears only in the flat/collapsed path (never grouped), evidence is K-capped per
   BC-2.11.027, and every evidence line passes through `escape_for_terminal`.
5. The count suffix is not subject to `escape_for_terminal`; it is a hardcoded format string
   and contains no attacker-controlled content.
6. **COLOR-LADDER REQUIREMENT (normative):** The collapse header path MUST apply the same
   verdict/confidence color-selection logic as `terminal.rs:209-221` to a pre-color string
   that ALREADY INCLUDES the ` (xN)` suffix. The ladder is:
   - `Likely` + `High` → `red().bold()`
   - `Likely` + any other confidence → `yellow`
   - `Possible` (any confidence) → `yellow`
   - `Inconclusive` (any confidence) → `cyan`
   - `Unlikely` (any confidence) → `dimmed`
   The ` (xN)` suffix MUST be part of the string that is passed to the color function — i.e.,
   the suffix MUST be appended BEFORE colorization is applied. Appending the suffix after the
   ANSI color-reset sequence (i.e., constructing the suffix outside the color span) is
   NON-CONFORMANT and violates EC-008 and the test vector for the red-bold branch.
7. **REPRESENTATIVE FINDING (normative):** For all N≥1, the "representative finding" of a
   collapsed group is `group_members[0]` — the first finding in emission order that established
   the group's collapse key. This definition is consistent across the N=1 singleton case (which
   trivially has only one member) and all N≥2 multi-member cases. The representative finding's
   `mitre_techniques` is used for the MITRE line (see PC-4 item (3)); other members' `mitre_techniques`
   are elided from terminal output but are preserved unmodified in the raw `findings` slice
   passed to JSON/CSV reporters (BC-2.11.029).

## Invariants

1. The suffix format is ` (x<N>)` — space, open-paren, literal `x`, decimal integer, close-paren.
   Examples: ` (x2)`, ` (x3142)`, ` (x10000)`. There are no alternative formats.
2. Singleton groups (N=1) produce no suffix. The absence of a suffix for singletons is
   intentional: it avoids noise and preserves backward compatibility for unique findings.
3. The count is computed as the exact group size from the collapse pass; it is never rounded,
   truncated, or represented as a range.
4. Color styling (from `use_color`) is applied to the COMPLETE header line including the
   ` (xN)` suffix. The ` (xN)` suffix is colorized identically to the summary text — there
   is no uncolorized suffix fragment.
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
| EC-007 | show_mitre_grouping=true, multiple identical-key findings | Collapse pass not applied; no count suffix on any finding regardless of group sizes. OBSERVABLE GUARANTEE: no ` (xN)` suffix appears in the terminal output for any grouped-mode finding, at any input volume. This is the same guarantee as BC-2.11.013 Invariant 4 |
| EC-008 | Group with N=2 and use_color=true | Complete header line (including ` (x2)` suffix) colored per verdict/confidence — the suffix is part of the pre-color `line` string and is colorized together with the summary text |
| EC-009 | show_mitre_grouping=true, N=100 identical-key findings | 100 individual lines, none with a ` (xN)` suffix — suffix-free guarantee enforced by the grouped path being structurally suffix-free (it never appends a count suffix), even at large N |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 findings all `(Anomaly, Inconclusive, Low, "Empty UA")`, collapse_findings=true | Header line contains `"Empty UA (x3)"` | happy-path (count display) |
| 1 finding `(Anomaly, Inconclusive, Low, "Empty UA")`, collapse_findings=true | Header line contains `"Empty UA"` with no `(x1)` suffix | happy-path (singleton) |
| 3142 findings all `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` | Header line ends with `"Empty User-Agent header (x3142)"` | happy-path (large count) |
| 1 unique finding + 5 identical findings | Unique finding: no suffix; identical group: `(x5)` suffix | mixed scenario |
| 2 findings same key, use_color=false | Header `"  [Anomaly] INCONCLUSIVE (LOW) - Empty UA (x2)\n"` | happy-path (no color) |
| 2 findings `(Reconnaissance, Likely, High, "Port scan")`, use_color=true | Complete header including ` (x2)` suffix is wrapped in the `red().bold()` color span — i.e., the output is `<red_bold_open>  [Reconnaissance] LIKELY (HIGH) - Port scan (x2)<red_bold_close>\n`; the suffix is INSIDE the bold-red span, not appended after the color reset | happy-path (EC-008 red-bold branch — LOW-1) |
| 2 findings with summary=`"Empty UA "` (trailing space), collapse_findings=true | Header line ends with `"Empty UA  (x2)\n"` — two spaces before `(x2)` (one from summary trailing space, one from the suffix's leading space); no trimming applied | edge-case (EC-005 — trailing whitespace, double-space pinned) |
| 3 findings all same collapse key, member[0].mitre_techniques=["T1036"], member[1].mitre_techniques=[], member[2].mitre_techniques=["T1059"], collapse_findings=true | MITRE line reads `    MITRE: T1036\n` (from group_members[0]); member[1] and member[2] mitre_techniques are elided from terminal; all 3 findings' mitre_techniques preserved in JSON/CSV output | representative-finding (F-PA-A01 divergent-mitre) |

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

- `src/reporter/terminal.rs:203-227` -- render_finding_prefix (summary escape + header line construction; called by grouped mode — observable: grouped output never carries a (xN) suffix)
- `src/reporter/terminal.rs:149-162` -- FINDINGS dispatch block (flat path; collapse-aware render inserted here when collapse=true; includes `out.push('\n')` at :161 and block close at :162)

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
