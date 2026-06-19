---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-18T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.9.0
modified:
  - "v1.1 2026-06-18: F2 adversarial round-1 fix — PC-4 sort direction corrected: 'verdict-rank (desc), confidence-rank (desc)' → 'ascending by rank (Likely=0/High=0 first)' to match BC-2.11.014 authoritative definition. No behavioral change; rank=0 means highest severity and is sorted first by ascending comparison, making the description of 'descending severity' formerly used in this BC misleading and internally inconsistent with BC-014's explicit rank assignments."
  - "v1.2 2026-06-18: R2-1 — propagate corrected verdict-rank enumeration: PC-4 now lists all four verdicts (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3) to match terminal.rs:447-454 source. R2-2 — introduced: v0.10.0 → v0.9.0. R2-6 — Invariant 4 and Invariant 5 reworded to observable-behavior form (drop implementation-sharing/no-duplication prescription; state externally testable invariants instead)."
  - "v1.3 2026-06-18: F3 adversarial round-1 remediation (C-1) — Architecture Anchors: collapse_findings_pass bullet replaced with collapse_findings_pass_refs (F4-new private helper; accepts &[&'a Finding]; called once per bucket; collapse_findings_pass at :340 retained as thin adapter for flat-mode caller). Invariant 3: reworded to name collapse_findings_pass_refs as the shared implementation called by both flat and grouped mode; flat-mode adapter delegation noted; no Finding value cloned."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.031: Per-Bucket Count Suffix — N≥2 Group Within a Tactic Bucket Renders Header with ` (xN)` Suffix; Singleton (N=1) Renders Without Suffix

## Description

Within each MITRE tactic bucket under `FindingsRender { grouping: Grouping::Grouped, collapse:
Collapse::Collapsed }`, the per-bucket collapse pass produces groups of findings sharing the same
`(category, verdict, confidence, summary)` key. A group of N≥2 identical-key findings within a
bucket renders its header line with a ` (xN)` suffix appended before colorization — identical
in format and colorization convention to the flat-mode suffix rule (BC-2.11.026). A singleton
group (N=1 within a bucket) renders via `render_finding_grouped` with no count suffix — the
output is byte-identical to the `{Grouped, Expanded}` path for that finding.

The ` (xN)` suffix is per-bucket, not per-report: two findings with the same collapse key but
in different MITRE tactic buckets each form their own singleton or group within their respective
bucket and are never cross-collapsed (see BC-2.11.030, BC-2.11.033).

## Preconditions

1. `TerminalReporter.render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
2. The per-bucket collapse pass has been applied to each tactic bucket's finding slice
   (via `collapse_findings_pass_refs` called once per bucket, not across the global findings slice).
3. The collapse key is the four-tuple `(category: ThreatCategory, verdict: Verdict,
   confidence: Confidence, summary: String)` — identical to the flat-mode collapse key
   (BC-2.11.025 Invariant 1).
4. Findings within a bucket have been sorted ascending by rank — verdict-rank ascending
   (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending
   (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE the collapse pass
   is applied (BC-2.11.033 establishes this sort-then-collapse ordering; BC-2.11.014 defines
   the rank assignments). The group representative is the first member in the post-sort bucket
   order.
5. `escape_for_terminal` has been applied to the group representative's `summary` field before
   the suffix is appended (VP-012 invariant; BC-2.11.010).

## Postconditions

1. For a group of N≥2 within a tactic bucket: the header line reads:
   `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n`
   where `<N>` is the exact integer count of findings in that per-bucket group (decimal,
   no leading zeros, no space between `x` and `N`). This format is identical to BC-2.11.026
   PC-1, scoped to grouped-collapse groups.
2. For a singleton group (N=1 within a bucket): `render_finding_grouped` is called for that
   finding. The output is byte-identical to the `{Grouped, Expanded}` path for the same
   finding — no count suffix, MITRE name expansion per BC-2.11.016.
3. **COLOR-LADDER REQUIREMENT (normative):** The grouped-collapse header path MUST apply the
   same verdict/confidence color-selection logic as `terminal.rs:391` to a pre-color string
   that ALREADY INCLUDES the ` (xN)` suffix. The same color ladder as BC-2.11.026 PC-6:
   - `Likely` + `High` → `red().bold()`
   - `Likely` + any other confidence → `yellow`
   - `Possible` (any confidence) → `yellow`
   - `Inconclusive` (any confidence) → `cyan`
   - `Unlikely` (any confidence) → `dimmed`
   The ` (xN)` suffix MUST be part of the string passed to the color function — suffix BEFORE
   colorization; appending the suffix after the ANSI reset is NON-CONFORMANT.
4. The ` (xN)` suffix MUST NOT appear on the MITRE line, any evidence line, or the tactic
   bucket header (`## <TacticName>`). It appears only on the finding-group header line.
5. The count value `N` equals `Vec.len()` of the findings grouped under that key within the
   bucket; it is always a positive integer (N≥1 by construction).
6. Cross-bucket suffix independence: two groups with the same collapse key in different tactic
   buckets each emit their own independent `(xN)` suffix based on their own bucket group count.
   A group of 3 in bucket A and a group of 2 in bucket B produce `(x3)` and `(x2)` respectively,
   never a combined `(x5)`.

## Invariants

1. The suffix format is ` (x<N>)` — space, open-paren, literal `x`, decimal integer,
   close-paren. Identical to BC-2.11.026 Invariant 1. No alternative formats.
2. Singleton groups (N=1) within a bucket produce no count suffix. The absence of a suffix for
   singletons preserves visual continuity with the existing `{Grouped, Expanded}` output.
3. The collapse pass producing the per-bucket groups uses `collapse_findings_pass_refs` — the
   shared collapse-logic implementation called by both flat mode and grouped mode. It is called
   once per bucket's findings slice (as `&[&Finding]`), not across all findings. The flat-mode
   adapter `collapse_findings_pass` delegates to the same function. No `Finding` value is cloned.
4. The rendered evidence cap for grouped-collapse bucket groups equals K=3 — the same value as
   the flat-mode collapse evidence cap. Evidence sampling within a grouped-collapse bucket group
   is governed by BC-2.11.032.
5. The color selection for a given (verdict, confidence) pair in the grouped-collapse header is
   identical to the color selected for the same (verdict, confidence) pair in flat-mode collapse.
   The mapping is the same color ladder described in PC-3 above.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single finding in a bucket (N=1 in bucket) | No `(xN)` suffix; rendered via `render_finding_grouped`; byte-identical to `{Grouped, Expanded}` for that finding |
| EC-002 | All findings in a tactic bucket have identical collapse key (N = bucket size) | One header line with `(x<bucket_size>)` suffix; evidence sampling per BC-2.11.032 |
| EC-003 | N=2 group in one bucket, all singletons in another bucket | `(x2)` suffix appears only in the first bucket; second bucket has no suffixes |
| EC-004 | Same collapse key present in two different tactic buckets (3 in bucket A, 2 in bucket B) | `(x3)` in bucket A for that key; `(x2)` in bucket B for that key; NO cross-bucket merge |
| EC-005 | `render = {Grouped, Expanded}` (`--mitre --no-collapse`): N=100 identical-key findings in a bucket | 100 individual finding lines, none with `(xN)` suffix — suffix-free guarantee of `{Grouped, Expanded}` path (BC-2.11.013 Invariant 4 continues to hold for the expanded path) |
| EC-006 | N=2 group, use_color=true, verdict=Likely, confidence=High | Complete header including ` (x2)` suffix is wrapped in `red().bold()` span — suffix inside the color span |
| EC-007 | Tactic bucket header (`## IcsImpairProcessControl`) followed by grouped-collapse groups | Tactic header itself carries no `(xN)` suffix; only finding-group header lines carry the suffix |
| EC-008 | N=3142 findings all in one bucket with same collapse key | Header ends with ` (x3142)` — no truncation, no abbreviation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `{Grouped, Collapsed}`, tactic bucket contains 3 findings with same collapse key `(Anomaly, Inconclusive, Low, "Empty UA")` | Header line: `  [Anomaly] INCONCLUSIVE (LOW) - Empty UA (x3)\n` (within that bucket) | happy-path (bucket suffix) |
| `{Grouped, Collapsed}`, tactic bucket contains 1 finding (singleton) | Header line contains no `(x1)` suffix; output byte-identical to `{Grouped, Expanded}` for that finding | happy-path (singleton no suffix) |
| `{Grouped, Collapsed}`, two buckets: bucket A has 3 same-key findings `(Anomaly, Inconclusive, Low, "Empty UA")`; bucket B has the same key but only 2 members | Bucket A: `(x3)` suffix; Bucket B: `(x2)` suffix — cross-bucket counts are independent | cross-bucket (EC-004) |
| `{Grouped, Collapsed}`, 2 findings `(Reconnaissance, Likely, High, "Port scan")` in one bucket, `use_color=true` | Complete header including ` (x2)` suffix is inside `red().bold()` ANSI span | color-ladder (EC-006) |
| `{Grouped, Expanded}` (`--mitre --no-collapse`), 100 same-key findings in a bucket | 100 individual lines, zero `(xN)` suffixes in entire output | suffix-free opt-out (EC-005) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | N≥2 group within a bucket produces `(xN)` suffix with correct count | unit: test_BC_2_11_031_grouped_collapse_suffix_format |
| — | N=1 singleton within a bucket produces no count suffix | unit: test_BC_2_11_031_singleton_no_suffix_in_bucket |
| — | Suffix is inside color span (red-bold for Likely/High) | unit: test_BC_2_11_031_grouped_collapse_color_ladder |
| — | Cross-bucket counts are independent | unit: test_BC_2_11_031_cross_bucket_suffix_independence |
| — | `{Grouped, Expanded}` path produces zero `(xN)` suffixes even for large N | unit: test_BC_2_11_013_grouped_mode_suffix_free (existing; covers `{Grouped, Expanded}` path) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — this BC defines the per-bucket count annotation format that makes grouped-mode collapse human-readable; the ` (xN)` suffix within each MITRE tactic bucket is the direct output contract of the Reporting capability for collapsed grouped mode |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — count computation occurs at display time; the raw Finding slice carries no count field and is never mutated) |
| Architecture Module | SS-11 (reporter/terminal.rs — `render_findings_grouped_collapsed`, F4-pending) |
| Stories | STORY-119 |
| Issue | #259 (Collapse repeated low-value findings — grouped-mode extension) |
| ADR | ADR-0003 (Binding Rule 5 revised, STORY-119; grouped-mode collapse subsection) |

## Related BCs

- BC-2.11.026 — mirrors (same ` (xN)` format and color-ladder rule, scoped to per-bucket groups in grouped mode)
- BC-2.11.030 — depends on (CLI mapping that activates `{Grouped, Collapsed}`)
- BC-2.11.032 — composes with (evidence sampling under the count-annotated group header)
- BC-2.11.033 — composes with (tactic-bucket ordering; collapse operates within buckets only)
- BC-2.11.034 — composes with (MITRE line format sourced from group representative for N≥2 groups)
- BC-2.11.025 — composes with (same `CollapseKey` four-tuple; `collapse_findings_pass_refs` is the shared collapse-logic implementation; `collapse_findings_pass` in flat mode delegates to it)
- BC-2.11.013 — depends on (tactic-bucket structure; `{Grouped, Expanded}` path suffix-free guarantee unchanged)

## Architecture Anchors

- `src/reporter/terminal.rs:432-483` — `render_findings_grouped` (existing; **F4-pending modification target:** per-bucket collapse pass inserted before inner rendering loop at ~:472-474; bucket sort at ~:463-467 already in place)
- `src/reporter/terminal.rs` — `render_findings_grouped_collapsed` — **F4-pending new function:** per-bucket collapse + grouped-collapse header rendering; does not yet exist; `render_findings_collapsed` at `:376-423` (flat-mode) is the structural precedent
- `src/reporter/terminal.rs` — `collapse_findings_pass_refs` (F4-new private helper; single source of collapse logic; accepts `&[&'a Finding]`; called once per bucket with emission-indices stripped). `collapse_findings_pass` at `:340` is retained as a thin adapter for the flat-mode caller and preserves its existing signature; it delegates to `collapse_findings_pass_refs`.
- `src/reporter/terminal.rs:391` — color ladder in `render_findings_collapsed` (flat-mode precedent for identical suffix-in-pre-color-string pattern)
- `src/reporter/terminal.rs:311-327` — `render_finding_grouped` (existing; called for N=1 singletons within a bucket in grouped-collapse path)

## Story Anchor

STORY-119

## VP Anchors

- — (VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — count is Vec.len() per bucket; suffix format is a constant string |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
