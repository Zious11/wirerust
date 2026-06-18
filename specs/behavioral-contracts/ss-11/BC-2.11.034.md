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
  - "v1.1 2026-06-18: F2 adversarial round-1 fix — (1) Add EC-008: group whose members share mitre_techniques[0] (same primary technique, same collapse-key bucket) but differ in trailing/secondary technique IDs; MITRE line sourced from members[0] only; trailing-ID divergence elided from terminal, preserved in JSON/CSV. (2) Renumber mis-prefixed test function anchor: test_BC_2_11_031_grouped_collapse_mitre_line_em_dash_format → test_BC_2_11_034_grouped_collapse_mitre_line_em_dash_format."
  - "v1.2 2026-06-18: R2-2 — introduced: v0.10.0 → v0.9.0. R2-5 — Invariant 3 rescoped: BC-2.11.026 reference narrowed to representative SOURCING only (members[0] principle), not MITRE line format; format precedent is BC-2.11.016 (em-dash name expansion), not BC-026's bare flat format. Related-BCs updated to match."
  - "v1.3 2026-06-18: F2 adversarial round-3 fix (F-PB-M01) — Invariant 3 reworded to clarify that the positional members[0] sourcing MECHANIC is shared with BC-2.11.026 PC-7, but the ORDERING that establishes index 0 differs: grouped-collapse uses post-sort bucket order (ascending verdict rank Likely=0/Possible=1/Inconclusive=2/Unlikely=3, then confidence rank), not the emission order used in flat mode. Related-BCs note for BC-2.11.026 updated to match."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.034: MITRE Line Format in Grouped-Collapse — Em-Dash Name Expansion Sourced from Group Representative (`members[0]`); No `(xN)` on MITRE Line

## Description

For N≥2 groups under `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`,
the MITRE line is sourced from the group representative (`group_members[0]`, the first finding in
the post-sort bucket order that established the key) and uses the same name-expansion format as
`render_finding_grouped` (BC-2.11.016): `MITRE: <id> — <TechniqueName>` for known IDs, or
`MITRE: <id> (unknown)` for unrecognized IDs. The separator is U+2014 (EM DASH), not `--`.
The `(xN)` count suffix does NOT appear on the MITRE line — it appears only on the header line
(BC-2.11.031 PC-4).

For singletons (N=1) within a bucket, `render_finding_grouped` is called directly (BC-2.11.031
PC-2), and the MITRE line is governed entirely by BC-2.11.016. This BC is specifically concerned
with the MITRE line rendering for N≥2 collapsed groups, where `render_finding_grouped` is NOT
called (the header is rendered inline with the `(xN)` suffix) and the MITRE line must be
explicitly reproduced from `members[0]`.

Other group members' `mitre_techniques` are elided from terminal output, applying the same
positional-members[0] sourcing mechanic as BC-2.11.026 PC-7 (flat mode), but over the
post-sort bucket order rather than emission order (see Invariant 3).

## Preconditions

1. `TerminalReporter.render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
2. A per-bucket collapse pass has produced a group of N≥2 findings within a tactic bucket.
3. The group representative is `group_members[0]` — the first finding in the post-sort bucket
   order that established the group key (BC-2.11.033 Invariant 4 / BC-2.11.025 Invariant 6
   analogue for grouped-collapse mode).
4. `group_members[0].mitre_techniques` is a `Vec<String>` (may be empty, may have one or more IDs).
5. The `technique_name(id)` catalog lookup function is available (same function as used by
   `render_finding_grouped` per BC-2.11.016).

## Postconditions

1. For a collapsed group of N≥2 in a tactic bucket: after the header line and evidence lines,
   the MITRE line is rendered from `group_members[0].mitre_techniques`:
   - If `group_members[0].mitre_techniques` is non-empty, the MITRE line uses the same
     format as `render_finding_grouped`: the IDs are joined as a comma-separated list
     (`mitre_techniques.join(", ")`); the first ID's name is looked up via `technique_name()`;
     if known, the line reads `    MITRE: <ids_joined> \u{2014} <name>\n`; if unknown, the
     line reads `    MITRE: <ids_joined> (unknown)\n`. The separator is U+2014 (EM DASH).
   - If `group_members[0].mitre_techniques` is empty, no MITRE line is rendered (same
     empty-vec guard as BC-2.11.016 PC-4).
2. The `(xN)` count suffix does NOT appear on the MITRE line. The count suffix is scoped to
   the header line only (BC-2.11.031 PC-4).
3. Other group members' `mitre_techniques` (members[1], members[2], ..., members[N-1]) are
   elided from terminal output. Their technique data is preserved unmodified in the raw
   `findings` slice available to JSON/CSV reporters (BC-2.11.029).
4. For singleton groups (N=1) within a bucket, `render_finding_grouped` is called (BC-2.11.031
   PC-2) and the MITRE line is governed entirely by BC-2.11.016. This BC's postconditions
   apply only to N≥2 grouped-collapse groups.
5. The observable MITRE line order: within a collapsed group's output block, the order is:
   (1) header line with `(xN)` suffix, (2) up to K=3 evidence lines (BC-2.11.032),
   (3) MITRE line from `members[0]` (this PC). The `(xN)` suffix appears ONLY on (1).

## Invariants

1. The MITRE line uses U+2014 (EM DASH) as the separator for known technique IDs — identical
   to BC-2.11.016 Invariant 1. No ASCII `--` or other substitute separator.
2. The name expansion logic is the same as in `render_finding_grouped`: `technique_name(id)` 
   for the first technique ID in the vec, `Some(name)` → em-dash format, `None` → `(unknown)`
   format. This is not re-implemented but reused from the same catalog lookup (F4 may inline or
   delegate; the observable output format is normative).
3. The MITRE line is sourced from the group representative (`members[0]`) regardless of N.
   The MITRE lines of other group members are never rendered in the terminal for grouped-collapse
   groups. This BC applies the positional `members[0]` sourcing MECHANIC of BC-2.11.026 PC-7,
   but over the post-sort bucket order (ascending verdict rank Likely=0/Possible=1/Inconclusive=2/Unlikely=3,
   then ascending confidence rank High=0/Medium=1/Low=2, then ascending emission-index) —
   NOT the emission order used in flat mode. The two modes share the positional-members[0]
   mechanic but differ in which ordering establishes index 0. The MITRE line FORMAT
   (em-dash name expansion, `(unknown)` fallback) follows BC-2.11.016 (the authoritative
   grouped-mode MITRE format contract) — NOT BC-2.11.026's bare flat format.
4. An empty `mitre_techniques` vec on `members[0]` produces no MITRE line for the group (same
   empty-vec guard as BC-2.11.016 PC-4 and `render_finding_grouped`'s `is_empty` check at
   `terminal.rs:313`).
5. The `(xN)` suffix is scoped to the header line only. Its presence on the header line
   does not affect or propagate to the MITRE line, evidence lines, or tactic bucket header.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | N=3 group, `members[0].mitre_techniques=["T1046"]` (known technique, Discovery tactic) | MITRE line: `    MITRE: T1046 \u{2014} Network Service Discovery\n`; header line has `(x3)` suffix; MITRE line has no suffix |
| EC-002 | N=3 group, `members[0].mitre_techniques=["T9999"]` (unknown technique ID) | MITRE line: `    MITRE: T9999 (unknown)\n`; header line has `(x3)` suffix; MITRE line has no suffix |
| EC-003 | N=3 group, `members[0].mitre_techniques=[]` (empty) | No MITRE line rendered for this group; header + evidence only |
| EC-004 | N=3 group, divergent mitre_techniques: `members[0].mitre_techniques=["T1046"]`, `members[1].mitre_techniques=["T1059"]`, `members[2].mitre_techniques=[]` | MITRE line from members[0] only: `    MITRE: T1046 \u{2014} Network Service Discovery\n`; members[1] and [2] MITRE elided from terminal; all preserved in JSON/CSV |
| EC-005 | N=2 group, `members[0].mitre_techniques=["T1692.001","T0836"]` (multi-tag) | MITRE line: `    MITRE: T1692.001, T0836 \u{2014} <name_of_T1692.001>\n`; IDs joined with `", "`; name from first ID only (consistent with `render_finding_grouped` behavior per BC-2.11.016) |
| EC-006 | Singleton (N=1) in a bucket | `render_finding_grouped` called directly; BC-2.11.016 governs MITRE line; this BC does not apply |
| EC-007 | N=3 group, `members[0].mitre_techniques=["T1046"]`; complete output block | Output order: (1) header line `  [<Cat>] <VERDICT> (<CONF>) - <summary> (x3)\n`, (2) up to 3 evidence lines `    > ...\n`, (3) MITRE line `    MITRE: T1046 \u{2014} Network Service Discovery\n` — `(x3)` appears only in (1) |
| EC-008 | N=3 group, members share same `mitre_techniques[0]` ("T1046") but differ in trailing/secondary IDs: `members[0].mitre_techniques=["T1046","T0836"]`, `members[1].mitre_techniques=["T1046"]`, `members[2].mitre_techniques=["T1046","T1059"]` | MITRE line sourced from `members[0]` only: `    MITRE: T1046, T0836 \u{2014} Network Service Discovery\n`; trailing-ID divergence of members[1] and [2] is elided from terminal output; all three findings' full `mitre_techniques` vecs are preserved unmodified in the raw findings slice for JSON/CSV reporters (BC-2.11.029) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `{Grouped, Collapsed}`, bucket group of N=3, `members[0].mitre_techniques=["T1046"]` (known), all have same collapse key `(Anomaly, Inconclusive, Low, "Port scan")` | Block contains: header `  [Anomaly] INCONCLUSIVE (LOW) - Port scan (x3)\n`, then evidence (per BC-2.11.032), then MITRE line `    MITRE: T1046 \u{2014} Network Service Discovery\n`; no `(x3)` on MITRE line | happy-path (em-dash expansion for grouped-collapse) |
| `{Grouped, Collapsed}`, bucket group of N=2, `members[0].mitre_techniques=["T9999"]` (unknown) | MITRE line: `    MITRE: T9999 (unknown)\n`; header has `(x2)` suffix; MITRE line has no suffix | unknown technique (EC-002) |
| `{Grouped, Collapsed}`, bucket group of N=3, `members[0].mitre_techniques=[]` | No MITRE line in block; header has `(x3)` suffix; evidence only | empty mitre_techniques (EC-003) |
| `{Grouped, Collapsed}`, bucket group of N=3, divergent mitre_techniques on members (EC-004 setup) | MITRE line uses only members[0] technique; other members' techniques elided from terminal; all preserved in JSON output | representative-finding MITRE (EC-004) |
| `{Grouped, Collapsed}` vs `{Grouped, Expanded}`: same bucket, same singleton finding | Both paths produce identical MITRE line for singleton (`render_finding_grouped` called in both cases for N=1) — no regression from grouped-collapse on singleton MITRE output | singleton MITRE preservation (EC-006) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | N≥2 grouped-collapse group MITRE line uses em-dash + name from members[0] | unit: test_BC_2_11_034_grouped_collapse_mitre_line_em_dash_format |
| — | MITRE line for unknown technique uses `(unknown)` format | unit: test_BC_2_11_034_unknown_technique_in_grouped_collapse |
| — | `(xN)` suffix absent from MITRE line | unit: test_BC_2_11_034_suffix_not_on_mitre_line |
| — | Divergent mitre_techniques across members: terminal uses members[0] only | unit: test_BC_2_11_034_divergent_mitre_representative_sourcing |
| — | Singleton MITRE line unchanged under grouped-collapse (uses `render_finding_grouped`) | unit: test_BC_2_11_034_singleton_mitre_via_render_finding_grouped |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — this BC defines the MITRE technique line format for collapsed groups in grouped-mode; the em-dash name expansion in the MITRE line is a direct output-quality contract of the Reporting capability that ensures analysts see human-readable technique names even for collapsed multi-finding groups |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — MITRE line is a display-layer rendering; the raw Finding.mitre_techniques vec is never mutated; non-representative members' techniques are preserved in the raw slice and available to JSON/CSV reporters) |
| Architecture Module | SS-11 (reporter/terminal.rs — `render_findings_grouped_collapsed`, F4-pending; reuses `technique_name()` catalog function and em-dash format from `render_finding_grouped` at `:313-327`) |
| Stories | STORY-119 |
| Issue | #259 (Collapse repeated low-value findings — grouped-mode extension) |
| ADR | ADR-0003 (Binding Rule 5 revised, STORY-119; grouped-mode collapse subsection) |

## Related BCs

- BC-2.11.016 — mirrors (authoritative MITRE line FORMAT for grouped mode: em-dash name expansion, `(unknown)` fallback; this BC extends it to collapsed N≥2 groups by sourcing the MITRE data from members[0])
- BC-2.11.026 — composes with (PC-7: applies the positional members[0] sourcing MECHANIC of BC-2.11.026 PC-7 to grouped-mode collapse, but over the post-sort bucket order — ascending verdict/confidence rank — NOT the emission order used in flat mode; the MITRE line FORMAT follows BC-2.11.016, not BC-026's bare flat format)
- BC-2.11.031 — composes with (header line produces `(xN)` suffix; this BC confirms the MITRE line in the same output block carries no suffix)
- BC-2.11.033 — depends on (sort-then-collapse ordering that determines who is members[0])
- BC-2.11.030 — depends on (CLI mapping that activates `{Grouped, Collapsed}`)
- BC-2.11.029 — composes with (non-representative members' mitre_techniques preserved in raw slice for JSON/CSV reporters)

## Architecture Anchors

- `src/reporter/terminal.rs:313-327` — `render_finding_grouped` body: `is_empty` guard at `:313`; IDs join at `:316`; first-technique name lookup at `:318-325`; em-dash arm at `:323`; `(unknown)` arm at `:324` — **F4-pending reuse target:** the grouped-collapse `render_findings_grouped_collapsed` function reproduces this MITRE line logic for N≥2 group representatives (may factor to a shared helper or inline; observable format is normative)
- `src/reporter/terminal.rs` — `render_findings_grouped_collapsed` — **F4-pending new function:** for N≥2 groups, renders header inline (with `(xN)` suffix) then evidence (BC-2.11.032) then MITRE line (this BC)
- `src/reporter/terminal.rs:311-327` — `render_finding_grouped` (existing; called for N=1 singletons within a bucket in grouped-collapse path; BC-2.11.016 governs its MITRE line output)

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
| **Deterministic** | yes — catalog lookup is deterministic; members[0] is deterministic given stable sort |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
