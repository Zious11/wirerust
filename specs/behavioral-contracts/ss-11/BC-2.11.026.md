---
document_type: behavioral-contract
level: L3
version: "1.13"
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
modified: ["v1.1 2026-06-17: F2 adversarial pass-1 — relax suffix colorization: (xN) suffix IS colorized with the header line (no seam for uncolorized suffix in render_finding_prefix); update Invariant 4, PC-4, EC-008 (F-259-02)", "v1.2 2026-06-17: F2 adversarial pass-2 — path-(b) collapse-aware wrapper prescribed as canonical in PC-4 (F-A03); dispatch anchor 149-160→149-162 (F-A05); EC-005 test vector added (F-A06)", "v1.3 2026-06-17: F2 adversarial pass-3 — add evidence emission sentence to PC-4 (F-F2X-03); fix EC row order EC-009/EC-008 → EC-008/EC-009 monotonic (F-F2X-02); fix arch anchor: remove stale 'appended here' alternative", "v1.4 2026-06-17: F2 adversarial pass-4 — F-F2-A01: convert PC-4 from internal-call-structure prescription to observable-behavior contract; remove 'path-(b) function-call graph' normative language; add non-normative implementation note; F-F2-O01: anchor :203-226 → :203-227; update EC-007 STRUCTURAL guarantee to observable-behavior form", "v1.5 2026-06-17: F2 adversarial pass-5 — F1: remove residual 'path-(b) separation' label from EC-009 body; reword to observable-behavior form", "v1.6 2026-06-17: F2 adversarial passes 6-8 — LOW-1: add red-bold (Likely/High) canonical test vector to confirm (xN) suffix is inside the red-bold colorization span for that branch", "v1.7 2026-06-17: F2 adversarial pass-9 — F-PA-01: add explicit normative PC-6 color-ladder requirement: same Likely+High→red().bold()/Likely+other→yellow/Possible→yellow/Inconclusive→cyan/Unlikely→dimmed logic applied to pre-suffix string BEFORE colorization; appending suffix after ANSI reset is NON-CONFORMANT", "v1.8 2026-06-17: F2 adversarial passes 12-14 — F-PA-A01: define 'representative finding' for N≥2 groups: group_members[0] (first member in emission order); add PC-7 (MITRE line sources group_members[0].mitre_techniques; other members' MITRE elided from terminal); add canonical test vector for divergent-mitre case", "v1.9 2026-06-17: issue-#62 F2 BC re-anchor — replace collapse_findings/show_mitre_grouping bool references with FindingsRender enum: Preconditions 1-2 + EC-006 + EC-007 + EC-009 updated. Rationale: illegal-state elimination. No behavioral change.", "v1.10 2026-06-18: F3 adversarial round-4 finding 2 (MEDIUM) stale dispatch anchor (DF-SIBLING-SWEEP-001) — Architecture Anchor cited FINDINGS dispatch at terminal.rs:149-162, but line 149 is the HOSTS section (if self.show_hosts_breakdown). Verified against src/reporter/terminal.rs: actual FINDINGS dispatch if-chain is at lines 185-207 (if !findings.is_empty() block through closing brace). Re-anchored Architecture Anchor to correct range 185-207.", "v1.11 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — FINDINGS dispatch :185-207 → :200-226; render_finding_prefix :203-227 → :267-291; PC-6 color-ladder normative reference :209-221 → :391 (render_findings_collapsed color ladder); Architecture Anchors updated.", "v1.12 2026-06-18: STORY-119 spec-evolution — PC-4 (OBSERVABLE LINE ORDER) revised: the suffix-free guarantee on the grouped path is now scoped to {Grouped, Expanded} only; {Grouped, Collapsed} emits (xN) per-bucket per BC-2.11.031. Preconditions narrowed to {Flat, Collapsed}. EC-007 and EC-009 updated to struct form distinguishing {Grouped, Expanded} (no suffix) vs {Grouped, Collapsed} (suffix per-bucket). Description updated to reflect dual-scope of the (xN) rule.", "v1.13 2026-06-18: F2 adversarial round-1 fix — Canonical Test Vectors last row: 'render = FindingsRender::FlatCollapsed' migrated to struct form 'render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }'. Stale enum-variant reference eliminated (Issue 2 remediation)."]
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

1. `TerminalReporter.render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`
   (`{Flat, Collapsed}` — the flat-mode collapsed path). The (xN) suffix rule defined in this
   BC applies to the flat-mode path. The grouped-mode analogue is BC-2.11.031.
2. Flat mode is guaranteed by `render.grouping == Grouping::Flat` at the type level — no
   separate flag check is needed.
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
4. **OBSERVABLE LINE ORDER — FLAT-COLLAPSE PATH ({Flat, Collapsed}):** For a collapsed group
   the terminal emits, in order: (1) the header line `  [<Category>] <VERDICT> (<CONFIDENCE>)
   - <escaped_summary> (x<N>)\n` (colorized, suffix included when N≥2), (2) up to K=3
   sampled evidence lines each passed through `escape_for_terminal` (BC-2.11.027), (3) the
   MITRE line `    MITRE: <ids>\n` where `<ids>` is `mitre_techniques.join(", ")` from the
   group representative (`group_members[0]`) — emitted only IF `mitre_techniques` is non-empty
   (BC-2.11.017 PC-6). The flat-collapse MITRE line uses the bare `MITRE: <ids>` format with
   NO em-dash and NO technique name expansion — consistent with BC-2.11.016 Invariant 4
   ("In default (flat) mode, `render_finding_flat` renders `MITRE: <id>` only (no expansion)")
   and BC-2.11.017 PC-1/PC-2. The em-dash expansion is ONLY used in grouped mode
   (`render_finding_grouped`; BC-2.11.016). The ` (xN)` suffix MUST NOT appear on the MITRE
   line or any evidence line — it appears only on the header line. The ` (xN)` suffix is
   colorized identically with the rest of the header line (suffix is part of the
   pre-colorization string).
   **Grouped-path suffix rule (STORY-119):** The `{Grouped, Expanded}` path (`--mitre
   --no-collapse`) MUST NOT emit a ` (xN)` suffix on any finding — that path renders each
   finding individually with no count annotation (BC-2.11.013 Invariant 4). The `{Grouped,
   Collapsed}` path (`--mitre` alone) DOES emit ` (xN)` per-bucket for N≥2 groups within
   each tactic bucket (BC-2.11.031). The suffix-free guarantee from Invariant 4 now applies
   only to `{Grouped, Expanded}`, not to all grouped-mode paths.
   **Implementation note (non-normative; F4 decides):** F4 MAY build the header inline in a
   collapse-aware wrapper, factor a shared helper, or use any other internal call graph —
   provided the observable line order above holds, evidence is K-capped per BC-2.11.027, and
   every evidence line passes through `escape_for_terminal`.
5. The count suffix is not subject to `escape_for_terminal`; it is a hardcoded format string
   and contains no attacker-controlled content.
6. **COLOR-LADDER REQUIREMENT (normative):** The collapse header path MUST apply the same
   verdict/confidence color-selection logic as `terminal.rs:391` to a pre-color string
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
| EC-006 | `render = {Grouping::Flat, Collapse::Expanded}` (--no-collapse opt-out, flat path) | No collapse pass runs; no count suffix on any finding; behavior per BC-2.11.028 |
| EC-007a | `render = {Grouping::Grouped, Collapse::Expanded}` (`--mitre --no-collapse`), multiple identical-key findings | No collapse pass; no count suffix on any finding in grouped output. OBSERVABLE GUARANTEE: no ` (xN)` suffix on any finding at any input volume. The `{Grouped, Expanded}` path renders each finding individually via `render_finding_grouped`. |
| EC-007b | `render = {Grouping::Grouped, Collapse::Collapsed}` (`--mitre` alone), multiple identical-key findings in same tactic bucket | Per-bucket collapse applies (BC-2.11.031); `(xN)` suffix emitted on N≥2 groups within each bucket. This is the STORY-119 default for grouped mode. The suffix appears on the finding-group header line, NOT on the tactic bucket header (`## TacticName`) or MITRE line. |
| EC-008 | Group with N=2 and use_color=true | Complete header line (including ` (x2)` suffix) colored per verdict/confidence — the suffix is part of the pre-color `line` string and is colorized together with the summary text |
| EC-009a | `render = {Grouping::Grouped, Collapse::Expanded}` (`--mitre --no-collapse`), N=100 identical-key findings | 100 individual lines, none with a ` (xN)` suffix — suffix-free guarantee of the `{Grouped, Expanded}` path |
| EC-009b | `render = {Grouping::Grouped, Collapse::Collapsed}` (`--mitre` alone), N=100 identical-key findings in the same tactic bucket | 1 collapsed header line with ` (x100)` suffix + K=3 evidence lines + MITRE line from members[0]; per BC-2.11.031 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 findings all `(Anomaly, Inconclusive, Low, "Empty UA")`, `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | Header line contains `"Empty UA (x3)"` | happy-path (count display) |
| 1 finding `(Anomaly, Inconclusive, Low, "Empty UA")`, `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | Header line contains `"Empty UA"` with no `(x1)` suffix | happy-path (singleton) |
| 3142 findings all `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` | Header line ends with `"Empty User-Agent header (x3142)"` | happy-path (large count) |
| 1 unique finding + 5 identical findings | Unique finding: no suffix; identical group: `(x5)` suffix | mixed scenario |
| 2 findings same key, use_color=false | Header `"  [Anomaly] INCONCLUSIVE (LOW) - Empty UA (x2)\n"` | happy-path (no color) |
| 2 findings `(Reconnaissance, Likely, High, "Port scan")`, use_color=true | Complete header including ` (x2)` suffix is wrapped in the `red().bold()` color span — i.e., the output is `<red_bold_open>  [Reconnaissance] LIKELY (HIGH) - Port scan (x2)<red_bold_close>\n`; the suffix is INSIDE the bold-red span, not appended after the color reset | happy-path (EC-008 red-bold branch — LOW-1) |
| 2 findings with summary=`"Empty UA "` (trailing space), `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | Header line ends with `"Empty UA  (x2)\n"` — two spaces before `(x2)` (one from summary trailing space, one from the suffix's leading space); no trimming applied | edge-case (EC-005 — trailing whitespace, double-space pinned) |
| 3 findings all same collapse key, member[0].mitre_techniques=["T1036"], member[1].mitre_techniques=[], member[2].mitre_techniques=["T1059"], `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | MITRE line reads `    MITRE: T1036\n` (from group_members[0]); member[1] and member[2] mitre_techniques are elided from terminal; all 3 findings' mitre_techniques preserved in JSON/CSV output | representative-finding (F-PA-A01 divergent-mitre) |

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

- `src/reporter/terminal.rs:267-291` -- render_finding_prefix (summary escape + header line construction; called by grouped mode — observable: grouped output never carries a (xN) suffix)
- `src/reporter/terminal.rs:200-226` -- FINDINGS dispatch block (flat path; collapse-aware render inserted here when collapse=true; if !findings.is_empty() at :200; block close + out.push('\n') at :225-226)
- `src/reporter/terminal.rs:391` -- color ladder in render_findings_collapsed (Likely+High→red().bold(), Likely+other→yellow, Possible→yellow, Inconclusive→cyan, Unlikely→dimmed; suffix included in pre-color string)

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
