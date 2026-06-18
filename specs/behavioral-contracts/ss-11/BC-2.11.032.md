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
  - "v1.1 2026-06-18: F2 adversarial round-1 fix — Invariant 3 sort direction corrected: 'verdict-rank desc, confidence-rank desc' → 'ascending by rank (Likely=0/High=0 first)' to match BC-2.11.014 authoritative rank definitions. No behavioral change."
  - "v1.2 2026-06-18: R2-1 — propagate corrected verdict-rank enumeration: Invariant 3 now lists all four verdicts (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3) to match terminal.rs:447-454 source. R2-2 — introduced: v0.10.0 → v0.9.0."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.032: Per-Bucket Evidence Sampling in Grouped-Collapse Mode — First min(N,K=3) Members Positionally; No Sliding Window

## Description

For a collapsed group of N≥2 findings within a MITRE tactic bucket under
`FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`, the terminal
reporter renders at most K=3 representative evidence lines per group. Evidence lines are taken
from the first `min(N, K)` group members in the post-sort bucket order (the order findings
appear within the bucket after the verdict/confidence/emission-index sort). For each inspected
member, if `finding.evidence` is non-empty, `finding.evidence[0]` is emitted as the
representative sample; if a member has an empty evidence vec, it contributes 0 lines and the
inspection window does NOT slide to the next member.

This BC directly mirrors BC-2.11.027 (flat-mode evidence sampling), scoped to per-bucket
groups in grouped-collapse mode. The `COLLAPSE_EVIDENCE_SAMPLES = 3` constant is shared. The
positional no-sliding-window invariant applies identically.

## Preconditions

1. `TerminalReporter.render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
2. A per-bucket collapse pass has produced a group of N≥2 findings within a tactic bucket.
   Group membership and order are determined by `collapse_findings_pass` applied to the
   post-sort bucket slice (BC-2.11.033 establishes that sort precedes collapse within each bucket).
3. `N` is the count of findings in the per-bucket group (N≥2; singleton groups are handled by
   `render_finding_grouped` per BC-2.11.031 PC-2 and their evidence is not capped by this BC).
4. Each member finding carries zero or more strings in `Finding.evidence: Vec<String>`.

## Postconditions

1. The terminal output for a grouped-collapse group of N≥2 contains at most K=3 evidence
   lines, each rendered as `    > <escaped_evidence_line>\n`.
2. Evidence lines are drawn from the FIRST `min(N, K)` members of the per-bucket group (by
   position in the post-sort bucket order — purely positional). For each of these inspected
   members, if `finding.evidence` is non-empty, `finding.evidence[0]` is emitted; if the
   member has an empty evidence vec, it contributes 0 lines. The total number of rendered
   evidence lines is therefore at most `min(N, K)` but may be less if any inspected member
   has an empty evidence vec.
3. When N≥2 and all inspected members have empty `evidence` vecs, zero evidence lines are
   rendered (no blank `    > ` lines).
4. When the group has N>K findings within the bucket, exactly K evidence lines appear (assuming
   all K selected members have at least one evidence entry). No "N-K more evidence lines elided"
   annotation or similar indicator is emitted. The `(xN)` header suffix (BC-2.11.031) is the
   only indicator of group size.
5. When the group has N≤K findings, all N members are inspected. Evidence is rendered as one
   line per member that has a non-empty evidence vec. The K cap is not reached.
6. Each rendered evidence line is passed through the `escape_for_terminal` function before
   output (VP-012; BC-2.11.010).

## Invariants

1. K=3 is the `COLLAPSE_EVIDENCE_SAMPLES` named constant — shared with flat-mode collapse;
   not configurable by CLI flag in v0.9.0.
2. Evidence sampling is POSITIONAL: inspect the first `min(N, K)` members in the group's
   post-sort-bucket order. From each inspected member, take `evidence[0]` IF non-empty;
   otherwise contribute 0 lines. The window does NOT slide past empty-evidence members —
   if `member[0]` has empty evidence, `member[K]` (index K) is still NOT inspected. No
   "skip-empty" logic; no content-based reordering; purely positional, purely bounded.
3. The "post-sort bucket order" for group membership is the order produced by the per-bucket
   sort — ascending by verdict-rank (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3),
   ascending by confidence-rank (High=0 first, Medium=1, Low=2), then emission-index ascending
   — applied BEFORE the collapse pass (BC-2.11.014 defines the rank assignments). The group
   representative (`members[0]`) is the first finding in this sorted order that established the
   group's key (BC-2.11.025 Invariant 6 analogue for grouped mode).
4. Evidence sampling is a display-only operation. `Finding.evidence` vecs are never truncated
   or mutated. JSON/CSV reporters receive the complete, unmodified `findings` slice
   (BC-2.11.029 invariant applies across all reporter types).
5. Elision is silent: no elision marker (e.g., "... and N more evidence lines") is rendered.
6. For singleton groups (N=1) within a bucket, this BC does not apply. Singletons are
   rendered via `render_finding_grouped`, which emits all evidence lines for that finding
   per BC-2.11.010 (no K-cap for singletons; identical to the `{Grouped, Expanded}` path).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Singleton in a bucket (N=1) | All evidence lines rendered (K-cap NOT applied to singletons; handled by `render_finding_grouped` per BC-2.11.031 PC-2) |
| EC-002 | Group of N=5 in a bucket, each member with 1 evidence line | 3 evidence lines rendered (from members[0], [1], [2]); members[3] and [4] evidence elided |
| EC-003 | Group of N=2 in a bucket, each with 1 evidence line | Both evidence lines rendered (N≤K; no elision) |
| EC-004 | Group of N=5, members[0] has empty evidence, others have 1 each | Positional window inspects members[0], [1], [2]. member[0] contributes 0 lines (window does NOT slide). members[1] and [2] each contribute 1 line. Total = 2 lines; members[3] and [4] never inspected |
| EC-005 | Group of N=5, all members have empty evidence | Zero evidence lines rendered |
| EC-006 | Group of N=3, member[0] has 2 evidence lines, others have 1 each | Only evidence[0] from member[0] used; total = 3 lines (one per member[0..2], first entry only) |
| EC-007 | Evidence line contains ESC byte (e.g., `"\x1b[31m"`) | Escaped via `escape_for_terminal`; raw ESC byte `0x1b` renders as `\u{1b}` via `char::escape_default` |
| EC-008 | Two buckets each with a group of N=5; same collapse key | Each bucket independently samples 3 evidence lines from its own group members; evidence from bucket A does not contaminate bucket B and vice versa |
| EC-009 | N=10000 within a bucket, each member with evidence | Exactly 3 evidence lines in terminal output; all 10000 complete findings with evidence preserved in JSON/CSV output |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `{Grouped, Collapsed}`, tactic bucket group of N=5, each member with evidence `["req_001"]`..`["req_005"]` (post-sort order) | Terminal: 3 evidence lines (`> req_001`, `> req_002`, `> req_003`); members[3] and [4] evidence elided | happy-path (bucket evidence sampling) |
| `{Grouped, Collapsed}`, tactic bucket group of N=2, each with 1 evidence line | Both evidence lines rendered (N≤K; no elision) | happy-path (below cap) |
| `{Grouped, Collapsed}`, tactic bucket group of N=5, members[0].evidence=[], others have 1 each | 2 evidence lines rendered (members[1] and [2]); member[0] contributes 0 (window does NOT slide to member[3]) | edge-case (EC-004 — no-sliding-window) |
| `{Grouped, Collapsed}`, tactic bucket group of N=3, member[0].evidence=["a","b"], member[1].evidence=["c"], member[2].evidence=["d","e"] | 3 evidence lines: `> a`, `> c`, `> d` (evidence[0] from each of the first 3 members; member[0].evidence[1]="b" and member[2].evidence[1]="e" elided) | edge-case (N=K=3, evidence[0]-from-first-3-members pattern) |
| Evidence line containing raw `"\x1b[31m"` in a grouped-collapse group | Rendered as `> \u{1b}[31m` (escaped via `char::escape_default`) | edge-case (EC-007 escape in grouped path) |
| `{Grouped, Collapsed}`, two tactic buckets each with N=5 same-key group, independent evidence | Each bucket emits its own 3 evidence lines independently | cross-bucket isolation (EC-008) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Group of N>K in a bucket renders exactly K evidence lines | unit: test_BC_2_11_032_evidence_sampling_k3_in_bucket |
| — | Group of N≤K in a bucket renders all available evidence | unit: test_BC_2_11_032_evidence_below_cap_in_bucket |
| — | Evidence drawn from first K members positionally (no sliding window) | unit: test_BC_2_11_032_evidence_positional_no_slide |
| — | Evidence escape preserved through grouped-collapse path | unit: test_BC_2_11_032_escape_preserved_in_bucket_evidence |
| — | Cross-bucket evidence isolation (EC-008) | unit: test_BC_2_11_032_cross_bucket_evidence_isolation |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — per-bucket evidence sampling for grouped-collapse groups is a terminal output formatting decision that controls how much contextual evidence the analyst sees per finding cluster within each MITRE tactic bucket, directly governing the readability vs. completeness trade-off of the Reporting capability in grouped mode |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — evidence truncation is a display-layer decision applied per-bucket; the raw Finding.evidence vec is never mutated; JSON/CSV reporters see full evidence) |
| Architecture Module | SS-11 (reporter/terminal.rs — `render_findings_grouped_collapsed`, F4-pending) |
| Stories | STORY-119 |
| Issue | #259 (Collapse repeated low-value findings — grouped-mode extension) |
| ADR | ADR-0003 (Binding Rule 5 revised, STORY-119; grouped-mode collapse subsection) |

## Related BCs

- BC-2.11.027 — mirrors (same K=3 positional no-sliding-window rule, scoped to per-bucket groups in grouped mode; flat-mode analogue)
- BC-2.11.031 — composes with (evidence lines rendered under the count-annotated group header produced by BC-031)
- BC-2.11.030 — depends on (CLI mapping that activates `{Grouped, Collapsed}`)
- BC-2.11.033 — depends on (sort order within bucket determines group member order for sampling)
- BC-2.11.029 — composes with (elided evidence preserved in raw slice; JSON/CSV reporters see full evidence)
- BC-2.11.010 — composes with (`escape_for_terminal` applied to each sampled evidence line)

## Architecture Anchors

- `src/reporter/terminal.rs:340-360` — `collapse_findings_pass` (existing; reused per-bucket without modification; returns `Vec<(CollapseKey, Vec<&Finding>)>` in first-occurrence-within-bucket order)
- `src/reporter/terminal.rs:73` — `COLLAPSE_EVIDENCE_SAMPLES` constant (K=3; shared with flat-mode; not duplicated)
- `src/reporter/terminal.rs` — `render_findings_grouped_collapsed` — **F4-pending new function:** implements per-bucket sampling loop (mirrors evidence loop logic from `render_findings_collapsed` at `:376-423` but scoped per-bucket, with `render_finding_grouped` for singletons)
- `src/reporter/terminal.rs:287-290` — evidence loop in `render_finding_prefix` (flat-mode precedent for `for ev in &f.evidence`; grouped-collapse replaces this with a bounded K-sampled loop per bucket group)
- `src/findings.rs:141` — `pub evidence: Vec<String>` (field sampled from; never mutated)

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
| **Global state access** | none (K is a compile-time constant) |
| **Deterministic** | yes — positional selection from sorted-bucket slice is deterministic |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
