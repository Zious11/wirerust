---
document_type: behavioral-contract
level: L3
version: "1.2"
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
  - "v1.1 2026-06-18: F2 adversarial round-1 fix — (1) Sort direction corrected throughout: 'descending' verdict/confidence-rank → 'ascending by rank (Likely=0/High=0 first)' in Description, PC-5, Invariant 4, and EC-007, to match BC-2.11.014 authoritative rank definitions. (2) EC-007 parenthetical 'higher verdict rank' → 'lower verdict-rank value (Likely=0), surfaced first by ascending sort'. (3) Mis-prefixed test-function anchors in Verification Properties renumbered from test_BC_2_11_030_* to test_BC_2_11_033_*."
  - "v1.2 2026-06-18: R2-1 — propagate corrected verdict-rank enumeration: Description, PC-5, and Invariant 4 now list all four verdicts (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3) to match terminal.rs:447-454 source. R2-2 — introduced: v0.10.0 → v0.9.0."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.033: Tactic-Bucket Ordering Invariant Under Grouped-Collapse — Bucket Sequence Unchanged; Collapse Operates Within Buckets Only

## Description

Under `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`, the
MITRE tactic-bucket ordering and membership rules are identical to the `{Grouped, Expanded}`
path (BC-2.11.013). Tactic buckets appear in `all_tactics_in_report_order()` sequence
(MITRE Enterprise kill-chain order, then ICS tactics); the `Uncategorized` bucket appears
last. The per-bucket collapse pass does NOT alter which bucket a finding belongs to, and does
NOT alter the order in which buckets are emitted.

The collapse transform is strictly a within-bucket operation: `collapse_findings_pass` is
called once per bucket's finding slice, not across the global findings slice. A finding's
tactic bucket membership is determined by `mitre_techniques[0]` (or `Uncategorized` if the
vec is empty or the first ID has no known tactic) — exactly as in BC-2.11.013 Invariant 2.
Collapse does not reassign findings to different buckets.

This BC also specifies the sort-then-collapse ordering within each bucket: findings in a
bucket are sorted ascending by rank — verdict-rank ascending (Likely=0 first, Possible=1,
Inconclusive=2, Unlikely=3), confidence-rank ascending (High=0 first, Medium=1, Low=2), then
emission-index ascending — BEFORE the collapse pass is applied (BC-2.11.014 defines the rank
assignments). This means the group representative (`members[0]`) is the first finding in the
sorted bucket order, not the first in the original global emission order.

## Preconditions

1. `TerminalReporter.render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
2. `findings` is non-empty `&[Finding]`.
3. `all_tactics_in_report_order()` is available and returns the canonical tactic sequence.

## Postconditions

1. Tactic bucket headers appear as `  ## <TacticName>\n` in the output, in the order
   returned by `all_tactics_in_report_order()` — identical to BC-2.11.013 PC-2.
2. A tactic bucket header is emitted only when at least one finding belongs to that bucket
   (same emission guard as BC-2.11.013 PC-3). Collapse within a bucket does not affect
   whether the bucket header is emitted — even if all findings in a bucket collapse to a
   single group, the bucket header is still emitted.
3. The `Uncategorized` bucket header (`  ## Uncategorized\n`) appears last among emitted
   buckets, collecting findings where `mitre_techniques` is empty OR where
   `technique_tactic(mitre_techniques[0])` returns `None` — identical to BC-2.11.013 PC-4.
4. Bucket membership is unchanged by collapse. A finding assigned to bucket B under
   `{Grouped, Expanded}` is assigned to the same bucket B under `{Grouped, Collapsed}`.
5. Within each bucket, findings are sorted ascending by rank — verdict-rank ascending
   (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending
   (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE the per-bucket
   `collapse_findings_pass` is applied (BC-2.11.014 defines the rank assignments). The result
   of this sort determines the within-bucket group order and the group representative identity
   (`members[0]` is the first finding in the sorted order that established the key).
6. The per-bucket `collapse_findings_pass` produces groups whose order within the bucket is
   first-occurrence in the SORTED bucket order (not first-occurrence in the global emission
   order). This is the "post-sort first-occurrence" definition for grouped-collapse mode.

## Invariants

1. `all_tactics_in_report_order()` is the authoritative bucket iteration order. Its return
   value is not affected by any collapse state.
2. Bucket assignment uses `mitre_techniques[0]` as the primary key — identical to BC-2.11.013
   Invariant 2. The collapse key `(category, verdict, confidence, summary)` is orthogonal to
   bucket assignment.
3. The per-bucket collapse pass is applied to the sorted-bucket slice for each tactic bucket
   independently and sequentially in tactic-order. There is no global cross-bucket collapse
   pass; `collapse_findings_pass` never receives the full global `findings` slice in grouped
   mode.
4. Sort-then-collapse ordering: the per-bucket sort — ascending by verdict-rank (Likely=0
   first, Possible=1, Inconclusive=2, Unlikely=3), ascending by confidence-rank (High=0 first,
   Medium=1, Low=2), ascending by emission-index — PRECEDES `collapse_findings_pass` for that
   bucket (BC-2.11.014 defines the rank values). This ordering is required to produce a
   deterministic and semantically meaningful group representative (the lowest rank-value
   finding, i.e., highest severity, wins the representative slot by appearing first in the
   ascending sort).
5. A finding can belong to at most one tactic bucket (determined by `mitre_techniques[0]` or
   the `Uncategorized` bucket). Multi-tag findings with `mitre_techniques = ["T1036", "T1059"]`
   land in the bucket for `T1036`'s tactic only (ADR-006 §13.7 primary-tactic approximation;
   consistent with BC-2.11.013 Invariant 2).
6. An empty bucket (no findings assigned) produces no header and no collapse activity for
   that tactic — identical behavior to `{Grouped, Expanded}`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All findings belong to one tactic bucket | Only that bucket's header emitted; no other headers; `Uncategorized` header not emitted (empty bucket guard applies) |
| EC-002 | Findings span three tactic buckets; middle bucket has all-singleton groups after collapse | All three bucket headers emitted in `all_tactics_in_report_order()` sequence; middle bucket findings rendered individually (no `(xN)` suffixes in that bucket) |
| EC-003 | Same collapse key present in two different tactic buckets | Finding in bucket A forms its own group in bucket A; finding in bucket B forms its own group in bucket B; they are never merged across buckets |
| EC-004 | A tactic bucket has 5 findings; after per-bucket collapse they reduce to 2 groups (one N=3, one N=2) | Bucket header still emitted; 2 group headers with `(x3)` and `(x2)` suffixes respectively; bucket header does not change |
| EC-005 | Findings with `mitre_techniques=[]` (empty) | Assigned to `Uncategorized` bucket; `Uncategorized` emitted last; collapse applied within `Uncategorized` bucket per same rules |
| EC-006 | Multi-tag finding `mitre_techniques=["T1692.001","T0836"]` and another finding `mitre_techniques=["T0836"]` with same collapse key | Both findings assigned based on their respective `mitre_techniques[0]` — T1692.001's tactic and T0836's tactic respectively; they land in DIFFERENT buckets and are NOT cross-collapsed |
| EC-007 | Per-bucket sort: two findings same collapse key, different verdict ranks (Likely vs Inconclusive), in same bucket | After sort: Likely finding is `members[0]` because it has lower verdict-rank value (Likely=0), surfaced first by ascending sort; it becomes group representative; `(x2)` suffix; group representative's fields used for header rendering |
| EC-008 | `{Grouped, Expanded}` path (--mitre --no-collapse) | Tactic bucket ordering is identical to `{Grouped, Collapsed}` — same `all_tactics_in_report_order()`, same bucket membership, same Uncategorized-last rule; no collapse pass; one finding per line |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `{Grouped, Collapsed}`, findings in 3 tactic buckets (Discovery, Execution, Uncategorized) | Output: Discovery bucket header first, Execution bucket header second, Uncategorized bucket header last — regardless of how many findings collapse within each bucket | happy-path (bucket order preserved) |
| `{Grouped, Collapsed}`, two findings with same collapse key: first has `mitre_techniques=["T1046"]` (Discovery), second has `mitre_techniques=["T1059"]` (Execution) | Two separate singleton groups in their respective buckets — no cross-bucket collapse despite identical collapse key | cross-bucket isolation (EC-003) |
| `{Grouped, Collapsed}`, same report as `{Grouped, Expanded}` baseline | Bucket headers appear in identical order in both modes; findings within each bucket differ (collapsed vs expanded) but bucket sequence is identical | invariant preservation (PC-1) |
| `{Grouped, Collapsed}`, a bucket with 3 same-key findings (N=3 → 1 group) | Bucket header still emitted; 1 collapsed group with `(x3)` suffix inside bucket | non-empty-bucket-header invariant (PC-2) |
| `{Grouped, Collapsed}`, per-bucket sort: 2 same-key findings with verdict Likely and Inconclusive respectively | Likely finding is `members[0]` after ascending-rank sort (Likely=0 surfaces first); used as group representative for header rendering; `(x2)` suffix | sort-then-collapse representative (EC-007) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Tactic bucket order under `{Grouped, Collapsed}` matches `all_tactics_in_report_order()` | unit: test_BC_2_11_013_grouped_collapsed_preserves_bucket_order |
| — | Cross-bucket findings with same collapse key are NOT merged | unit: test_BC_2_11_033_different_buckets_not_cross_collapsed |
| — | Sort precedes collapse within each bucket (representative = lowest-rank-value / highest-severity finding by key) | unit: test_BC_2_11_033_first_occurrence_in_sorted_bucket_order |
| — | Uncategorized bucket emitted last under `{Grouped, Collapsed}` | unit: test_BC_2_11_033_uncategorized_last_under_grouped_collapse |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — this BC defines the structural invariant that the MITRE kill-chain tactic ordering is preserved under grouped-collapse; maintaining a consistent, predictable tactic sequence is a core output-structure responsibility of the Reporting capability for analysts relying on MITRE ATT&CK organization |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — tactic bucket assignment and ordering are display-layer decisions; the raw Finding slice and its `mitre_techniques` fields are never mutated by collapse) |
| Architecture Module | SS-11 (reporter/terminal.rs — `render_findings_grouped_collapsed`, F4-pending; reuses `all_tactics_in_report_order()` and tactic HashMap unchanged) |
| Stories | STORY-119 |
| Issue | #259 (Collapse repeated low-value findings — grouped-mode extension) |
| ADR | ADR-0003 (Binding Rule 5 revised, STORY-119; grouped-mode collapse subsection); ADR-006 §13.7 (primary-tactic approximation for multi-tag findings) |

## Related BCs

- BC-2.11.013 — depends on (tactic-bucket structure, `all_tactics_in_report_order()` iteration, Uncategorized-last rule; this BC asserts those invariants are preserved under grouped-collapse)
- BC-2.11.030 — depends on (CLI mapping that activates `{Grouped, Collapsed}`)
- BC-2.11.031 — composes with (per-bucket suffix rule that applies within each bucket in the order this BC defines)
- BC-2.11.032 — composes with (per-bucket evidence sampling scoped to the groups this BC's ordering determines)
- BC-2.11.034 — composes with (MITRE line format for the group representatives ordered per this BC)

## Architecture Anchors

- `src/reporter/terminal.rs:432-483` — `render_findings_grouped` (existing; tactic-bucket loop at `:469`; per-bucket sort at `:463-467`; **F4-pending modification target** for adding collapse pass after sort)
- `src/reporter/terminal.rs` — `render_findings_grouped_collapsed` — **F4-pending new function:** outer tactic-bucket loop mirrors `render_findings_grouped`; inner per-bucket collapse and grouped-collapse rendering replaces `render_finding_grouped` direct call
- `src/reporter/terminal.rs:340-360` — `collapse_findings_pass` (existing; called once per bucket slice in the F4 implementation, not once for the global slice)

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
| **Deterministic** | yes — `all_tactics_in_report_order()` is deterministic; sort is stable |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
