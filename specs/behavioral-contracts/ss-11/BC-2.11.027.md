---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v1.1 2026-06-17: fix N=1 singleton model — K-cap does NOT apply to singletons; evidence renders unchanged per BC-2.11.010 (consistency audit remediation)", "v1.2 2026-06-17: F2 adversarial pass-1 — fix CRITICAL F-259-01: enforce positional first-K-members model throughout (PC-2/Invariant-2/PC-5/EC-004/test vectors); fix EC-004 total=2 not 3; add N=3/N=4 boundary vectors (F-259-07)"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.027: Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display

## Description

For a collapsed display group produced by the collapse pass (BC-2.11.025), the terminal
reporter renders at most K=3 representative evidence lines. Evidence lines are taken from
the first K findings in the group, in original emission order (the order the findings appear
in the input `findings` slice). Specifically, up to one evidence line per contributing
finding is taken from the first K group members. If a member finding has multiple evidence
lines, only the first evidence line of that finding is used as the representative sample.
Evidence lines beyond the K-sample window are silently elided from the terminal display
only. The complete `evidence` field of every finding in the group remains in the raw
`findings` slice and is available to JSON and CSV reporters unmodified (BC-2.11.029).

K=3 is a named constant. It is hardcoded in the implementation for v0.8.0 and is not
configurable via CLI flag. Future cycles may expose K as `--collapse-evidence-samples N`.

## Preconditions

1. `TerminalReporter.collapse_findings = true`.
2. `TerminalReporter.show_mitre_grouping = false` (flat mode).
3. A collapsed display group has been produced by the BC-2.11.025 collapse pass with N
   member findings (N≥1).
4. Each member finding carries zero or more strings in its `Finding.evidence: Vec<String>`.

## Postconditions

1. The terminal output for a collapsed group contains at most K=3 evidence lines
   (rendered as `    > <escaped_evidence_line>\n` per the existing render_finding_prefix
   format).
2. Evidence lines are drawn from the FIRST min(N, K) members in the group (by position in
   the original emission order — purely positional). For each of these inspected members, if
   `finding.evidence` is non-empty, `finding.evidence[0]` is emitted as the representative
   line; if `finding.evidence` is empty, that member contributes 0 lines and the window does
   NOT slide to the next member. The total number of rendered evidence lines is therefore
   at most min(N, K) but may be less if any inspected member has an empty evidence vec.
3. When N≥1 and all N findings have empty `evidence` vecs, zero evidence lines are rendered
   (no blank `> ` lines). This matches the pre-collapse behavior for findings with no evidence.
4. When the group has N>K findings, exactly K evidence lines appear in the terminal output
   (assuming all K selected findings have at least one evidence entry). No "N-K more evidence
   lines elided" annotation or similar indicator is emitted.
5. When the group has N≤K findings, all N members are inspected (since min(N,K)=N). Evidence
   is rendered as one line per member that has a non-empty evidence vec. The K cap is not
   reached; the positional window covers all members.
6. Each rendered evidence line is passed through `escape_for_terminal` before output,
   identical to the existing per-finding evidence rendering (BC-2.11.010).

## Invariants

1. K=3 is a compile-time named constant (e.g., `const COLLAPSE_EVIDENCE_SAMPLES: usize = 3`).
   It is not configurable by the caller or by any CLI flag in v0.8.0.
2. Evidence sampling is POSITIONAL: inspect the first min(N,K) members in group order (the
   first min(N,K) findings in the input slice that share the collapse key). From each
   inspected member, take `evidence[0]` IF the vec is non-empty; otherwise contribute 0 lines.
   The window does NOT slide past empty-evidence members — if member[0] has empty evidence,
   member[K] (index K) is still NOT inspected. The algorithm never "skips" an empty member to
   find the next non-empty one. It does NOT select "most representative", "highest confidence",
   or otherwise reorder by any criterion — it is purely positional, purely bounded, no sliding.
3. The evidence cap applies only to the terminal display. The `Finding.evidence` field on
   every finding in the group is never truncated or mutated.
4. Elision is silent: no elision marker (e.g., "... and N more") is rendered in v0.8.0.
   The count suffix on the header line (BC-2.11.026) is the only indicator of group size.
5. The `escape_for_terminal` invariant (BC-2.11.010) is preserved: every evidence line
   rendered by a collapsed group goes through `escape_for_terminal`, via the same code path
   used for non-collapsed rendering.
6. For a singleton group (N=1), the collapse feature does not alter evidence rendering in any
   way. The K-cap does NOT apply to singletons. The finding's evidence renders identically to
   the pre-v0.8.0 `render_finding_prefix` output — all evidence lines shown, governed by
   BC-2.11.010. A finding with 5 or 100 evidence lines will show all 5 or 100 lines when it
   is a singleton group, just as it did before v0.8.0.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group with N=1 member (singleton), 5 evidence lines | All 5 evidence lines rendered, unchanged from pre-v0.8.0 behavior (K-cap does NOT apply to singletons; singleton passes through unmodified per BC-2.11.010) |
| EC-002 | Group with N=5 members, each with 1 evidence line | 3 evidence lines rendered (from members[0], members[1], members[2]); members[3] and members[4] evidence elided |
| EC-003 | Group with N=2 members, 1 evidence line each | Both evidence lines rendered (N≤K, no elision) |
| EC-004 | Group with N=5 members, members[0] has empty evidence, others have 1 each | Positional window inspects members[0], members[1], members[2] (first min(5,3)=3 members). member[0] contributes 0 lines (empty vec; window does NOT slide). member[1] + member[2] each contribute 1 line. **Total = 2 lines.** members[3] and members[4] are never inspected. |
| EC-005 | Group with N=5 members, all have empty evidence | Zero evidence lines rendered |
| EC-006 | Group with N=3 members, member[0] has 2 evidence lines, others have 1 each | Only evidence[0] from member[0] is used; total = 3 lines (one per member, first entry only) |
| EC-007 | Evidence line contains ESC byte | Escaped via escape_for_terminal before output; escape invariant preserved |
| EC-008 | collapse_findings=false | No collapse pass; evidence rendered in full per finding per pre-v0.8.0 behavior (BC-2.11.010 unchanged) |
| EC-009 | Group with N=10000, each finding has evidence | Exactly 3 evidence lines in terminal output; JSON reporter receives 10000 complete findings each with full evidence |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 identical-key findings each with evidence=["req_001"], ..., ["req_005"] | Terminal: 3 evidence lines (`> req_001`, `> req_002`, `> req_003`); evidence for members 4-5 elided | happy-path (evidence sampling) |
| 2 identical-key findings each with 1 evidence line | Both evidence lines rendered (N≤K; no elision) | happy-path (below cap) |
| 5 identical-key findings, all evidence=[] | Zero evidence lines rendered (no blank `>` lines) | edge-case (empty evidence) |
| 3 identical-key findings: member[0].evidence=["a","b"], member[1].evidence=["c"], member[2].evidence=["d","e"] | 3 evidence lines: `> a`, `> c`, `> d` (evidence[0] from each of the first 3 members in emission order; member[0].evidence[1]="b" and member[2].evidence[1]="e" elided) | edge-case (N=K=3, evidence[0]-from-first-3-members pattern) |
| Evidence line containing "\x1b[31m" (ANSI escape) | Rendered as `> \\x1b[31m` (escaped) | edge-case (EC-007) |
| N=3 identical-key findings, each with exactly 1 evidence line (evidence=["e0"], ["e1"], ["e2"]) | Header `(x3)`, exactly 3 evidence lines: `> e0`, `> e1`, `> e2` — NO elision (N≤K boundary: N=K=3, all members inspected, all contribute) | N≤K boundary (F-259-07) |
| N=4 identical-key findings, each with exactly 1 evidence line (evidence=["e0"], ["e1"], ["e2"], ["e3"]) | Header `(x4)`, exactly 3 evidence lines: `> e0`, `> e1`, `> e2` — member[3] evidence elided (N>K boundary: N=4>K=3, only first 3 members inspected) | N>K boundary (F-259-07) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Group of N>K renders exactly K evidence lines | unit: test_BC_2_11_027_evidence_capped_at_k |
| — | Group of N≤K renders all available evidence | unit: test_BC_2_11_027_evidence_below_cap_rendered_fully |
| — | Evidence from first K members (positional, not by content) | unit: test_BC_2_11_027_evidence_drawn_from_first_k_members |
| — | Evidence escape preserved through collapse path | unit: test_BC_2_11_027_escape_preserved_in_sampled_evidence |
| — | JSON output unaffected (all evidence present) | integration: test_BC_2_11_029_json_receives_full_findings (cross-BC) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- evidence sampling for collapsed groups is a terminal output formatting decision that controls how much contextual detail the analyst sees per finding cluster, directly governing the readability vs. completeness trade-off in the Reporting and Output capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- evidence truncation is a display-layer decision; the raw Finding.evidence vec is never mutated) |
| Architecture Module | SS-11 (reporter/terminal.rs) |
| Stories | STORY-118 |
| Issue | #259 (Collapse repeated low-value findings) |
| ADR | ADR-0003 (display-layer aggregation subsection; K=3 constant documented therein) |

## Related BCs

- BC-2.11.025 -- depends on (collapse pass produces the group and determines group membership order)
- BC-2.11.026 -- composes with (evidence lines rendered under the count-annotated header from BC-026)
- BC-2.11.029 -- depends on (elided evidence is preserved in the raw slice; JSON/CSV reporters see full evidence)
- BC-2.11.010 -- composes with (escape_for_terminal applied to each sampled evidence line identically to pre-collapse behavior)

## Architecture Anchors

- `src/reporter/terminal.rs:223-226` -- evidence rendering loop in render_finding_prefix (`for ev in &f.evidence`); the collapse path replaces this with a bounded loop over sampled evidence
- `src/findings.rs:141` -- `pub evidence: Vec<String>` (field that is sampled from, never mutated)

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
| **Global state access** | none (K is a compile-time constant) |
| **Deterministic** | yes — positional selection from slice is deterministic |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
