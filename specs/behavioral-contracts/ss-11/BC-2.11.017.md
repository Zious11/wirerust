---
document_type: behavioral-contract
level: L3
version: "1.15"
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
  - "v1.3: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchor: render_finding_flat 223-228 → 230-235 (fn at 230, closing at 235); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.7: PG-ARP-F2-007 — fix stale terminal.rs line anchors shifted by F2 multi-tag additions (STORY-100): render_finding_flat :230-235 → :232-238 (fn decl at 232, closing at 238); Invariant 1 fn ref :230-235 → :232-238; Source Evidence path updated; verified against current HEAD — 2026-06-13"
  - "v1.5: ADR-006 / Decision 13 §13.7 (F2 v0.3.0) — multi-tag rendering: single ID emits 'MITRE: T1036'; multi-tag emits 'MITRE: T0855, T0836' (comma-space separated); empty vec emits no MITRE line. Precondition 2 and EC-003 updated; EC-005/EC-006 added. — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, Postconditions, EC-005, EC-006, and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.8: issue-#259 F2 integrate (v0.8.0 collapse feature) — extend Description and add Invariant 5 and EC-007: when collapse_findings=true (default in v0.8.0), render_finding_flat is called per collapsed group with the collapsed representative Finding, and the header line appends a (xN) count suffix per BC-2.11.026; when N=1 (singleton), the header line is byte-identical to the pre-v0.8.0 output; when collapse_findings=false (--no-collapse), all existing postconditions remain byte-identical to pre-v0.8.0. Cross-references BC-2.11.025/026/028/029. ADR-0003 (display-layer aggregation subsection) cited. — 2026-06-17"
  - "v1.9 2026-06-17: F2 adversarial pass-1 — update Invariant 5: (xN) suffix is colorized identically with the header line (no uncolorized suffix; suffix appended to pre-color line string before colorization) (F-259-02)"
  - "v1.10 2026-06-17: F2 adversarial pass-2 — align Invariant 5 to path-(b) collapse-aware wrapper (F-A03): the flat collapse path uses a wrapper that builds the header with suffix; render_finding_prefix itself is unchanged; grouped mode is structurally suffix-free"
  - "v1.11 2026-06-17: F2 adversarial pass-4 — F-F2-A01: convert Invariant 5 and Description collapse paragraph from internal-call-structure prescription to observable-behavior contract; add MITRE line observable-behavior postcondition (PC-6); remove 'render_finding_flat is called once per group via this wrapper' call-graph claim; add non-normative implementation note per adjudicated model"
  - "v1.12 2026-06-17: F2 adversarial pass-9 — F-PA-01: add cross-reference to BC-2.11.026 PC-6 in Invariant 5 for the full color-ladder requirement; the (xN) suffix colorization is governed by BC-2.11.026 PC-6"
  - "v1.13 2026-06-17: F2 adversarial passes 12-14 — F-PA-A01: define 'representative finding' for N≥2 groups = group_members[0] (first in emission order); update PC-6 and EC-007 to reference group_members[0] explicitly; add canonical test vector for divergent-mitre case (member[0].mitre=[T1036], member[1].mitre=[], member[2].mitre=[T1059] → MITRE: T1036 from member[0])"
  - "v1.14 2026-06-17: issue-#62 F2 BC re-anchor — replace show_mitre_grouping/collapse_findings bool references with FindingsRender enum: Precondition 1 'show_mitre_grouping = false' → 'render != FindingsRender::Grouped (i.e. FlatCollapsed or FlatExpanded)'; Description and Postcondition 6 'collapse_findings = true/false' → 'render = FindingsRender::FlatCollapsed / FindingsRender::FlatExpanded'; Invariant 5 scoping boundary reworded. Rationale: illegal-state elimination. No behavioral change."
  - "v1.15 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — render_finding_flat fn :232-238 → :296-302; Invariant 1 fn ref + Architecture Anchor + Source Evidence path updated."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.017: Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash)

<!--
  PREVIOUS VERSION SUMMARY (v1.4 -> v1.5):
  Title updated: "<id>" -> "<id(s)>" to reflect multi-ID case
  Precondition 2: "mitre_technique set" -> "mitre_techniques non-empty"
  Postcondition 1: single ID unchanged; multi-ID format: "MITRE: T0855, T0836"
  EC-003: "mitre_technique=None" -> "mitre_techniques=vec![]"
  EC-005 added: Finding with 2 techniques renders comma-space-separated IDs
  EC-006 added: Finding with empty mitre_techniques -> no MITRE line
  Canonical vectors updated with multi-tag example
-->

## Description

When `TerminalReporter.render` is `FindingsRender::FlatCollapsed` or `FindingsRender::FlatExpanded`
(i.e., not `FindingsRender::Grouped`), each finding's MITRE line reads `    MITRE: <id(s)>\n` --
all technique IDs from `mitre_techniques` joined with `", "` (comma-space), with no em-dash
separator, no technique names, and no `(unknown)` labels. For singleton vecs the output is
identical to the pre-F2 single-technique format (`MITRE: T1036`). For multi-element vecs the
output is `MITRE: T1692.001, T0836`. Findings with empty `mitre_techniques` produce no MITRE
line. Findings are rendered in their original emission order with no tactic bucketing or sorting.

**v0.8.0 collapse interaction (BC-2.11.025/BC-2.11.026/BC-2.11.027):** When
`render = FindingsRender::FlatCollapsed` (the v0.8.0 default), the terminal emits one display
group per collapsed group. For each group the OBSERVABLE output lines are, in order: (1) the
header line with ` (xN)` suffix when N≥2 (colorized; BC-2.11.026), (2) up to K=3 sampled
evidence lines (BC-2.11.027), (3) the MITRE line IF `mitre_techniques` is non-empty — identical
in format to this BC's Postcondition 1 (comma-space-joined IDs from the representative finding).
When N=1 (singleton group), the output is byte-identical to the pre-v0.8.0 single-finding
output. When `render = FindingsRender::FlatExpanded` (`--no-collapse`), all postconditions in
this BC remain byte-identical to the pre-v0.8.0 behavior.

## Preconditions

1. `TerminalReporter.render` is `FindingsRender::FlatCollapsed` or `FindingsRender::FlatExpanded`
   (any flat mode; i.e., not `FindingsRender::Grouped`).
2. A finding has a non-empty `mitre_techniques` vec.

## Postconditions

1. The MITRE line reads: `    MITRE: <id1>, <id2>, ...\n` where IDs are joined with `", "`
   in the order they appear in `mitre_techniques`. For singleton vecs: `    MITRE: T1036\n`.
   For two-element vecs: `    MITRE: T1692.001, T0836\n`.
2. No em-dash, no technique names, no `(unknown)` labels.
3. No `## TacticName` or `## Uncategorized` headers in the FINDINGS section.
4. Findings render in their original slice order.
5. Findings with empty `mitre_techniques` produce no MITRE line (no blank line either).
6. **v0.8.0 collapse path:** When `render = FindingsRender::FlatCollapsed`, for each collapsed
   group the MITRE line (if `mitre_techniques` non-empty) is emitted after the header line and
   after the K-sampled evidence lines, using the same `mitre_techniques.join(", ")` format as
   Postconditions 1–2. The MITRE line content comes from `group_members[0]` — the first
   finding in emission order that established the group's key (the representative finding per
   BC-2.11.026 PC-7). Other members' `mitre_techniques` are elided from terminal output but
   preserved in JSON/CSV (BC-2.11.029). The MITRE line does NOT carry the ` (xN)` suffix —
   the count suffix appears only on the header line (BC-2.11.026 Invariant 2 / EC-007).

## Invariants

1. Flat modes (`FindingsRender::FlatCollapsed` or `FindingsRender::FlatExpanded`) use
   `render_finding_flat` (terminal.rs:296-302) which iterates `mitre_techniques` and joins
   IDs with `", "`. If empty, skips the MITRE line entirely.
2. `render_finding_flat` never calls `technique_name()` or `technique_tactic()`.
3. This mode is the "no --mitre flag" case; `FindingsRender::Grouped` requires the `--mitre`
   CLI flag.
4. The join separator is `", "` (comma followed by single space). No trailing separator.
5. **v0.8.0 collapse path — OBSERVABLE LINE ORDER (BC-2.11.025/026/027):** When
   `render = FindingsRender::FlatCollapsed`, for each collapsed group the terminal emits —
   in order — (1) the header line with ` (xN)` suffix (colorized; N≥2) or without suffix
   (N=1), (2) up to K=3 sampled evidence lines each passed through `escape_for_terminal`,
   (3) the MITRE line using `mitre_techniques.join(", ")` format from Postconditions 1–2
   IF `mitre_techniques` is non-empty. The ` (xN)` suffix is colorized identically with
   the rest of the header line (see BC-2.11.026 PC-6 for the full color-ladder requirement:
   Likely+High→red().bold(), Likely+other→yellow, Possible→yellow, Inconclusive→cyan,
   Unlikely→dimmed; the suffix MUST be inside the color span). When
   `render = FindingsRender::Grouped`, the collapse pass is NOT applied and all existing
   postconditions hold unchanged (BC-2.11.025 Invariant 5). The `FindingsRender` enum
   makes the former impossible state (`show_mitre_grouping=true && collapse_findings=true`)
   structurally unrepresentable.
   **Implementation note (non-normative; F4 decides):** F4 MAY reimplement the flat-render
   inline in a collapse-aware wrapper OR factor a shared header/MITRE helper — the internal
   function call graph is unconstrained PROVIDED the observable line order above holds, the
   ` (xN)` suffix appears only in the `FlatCollapsed` path (never `Grouped`), evidence is
   K-capped, and every evidence line passes through `escape_for_terminal`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with mitre_techniques=["T1036"] (single known ID) | "MITRE: T1036\n" (no name) — identical to pre-F2 output |
| EC-002 | Finding with mitre_techniques=["T9999"] (unknown ID) | "MITRE: T9999\n" (no "(unknown)" label) |
| EC-003 | Finding with mitre_techniques=vec![] (empty) | No MITRE line rendered for this finding |
| EC-004 | `render = FindingsRender::FlatCollapsed` or `FindingsRender::FlatExpanded`, multiple findings | Rendered in emission order |
| EC-005 | Finding with mitre_techniques=["T1692.001","T0836"] (multi-tag, Modbus register write; T1692.001 = v19 ICS sub-technique, successor to revoked T0855) | "MITRE: T1692.001, T0836\n" (both IDs, comma-space separated) |
| EC-006 | Finding with mitre_techniques=["T0806","T1692.001"] (burst finding) | "MITRE: T0806, T1692.001\n" |
| EC-007 | `render = FindingsRender::FlatCollapsed`, group of N=5 identical findings, mitre_techniques=["T1036"] | Header line: `  [Category] VERDICT (CONFIDENCE) - summary (x5)\n`; MITRE line (from group_members[0]): `    MITRE: T1036\n`; count suffix appears on the header line, not on the MITRE line |
| EC-008 | `render = FindingsRender::FlatExpanded` (--no-collapse), N=5 identical findings | 5 individual header lines, each without suffix; each with `    MITRE: T1036\n`; byte-identical to pre-v0.8.0 output |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with mitre_techniques=["T1036"], `render = FindingsRender::FlatExpanded` | Output contains "MITRE: T1036" without em-dash | happy-path (singleton — backward-compat) |
| Finding with mitre_techniques=["T1692.001","T0836"], `render = FindingsRender::FlatExpanded` | Output contains "MITRE: T1692.001, T0836" | happy-path (multi-tag) |
| Finding with mitre_techniques=[], `render = FindingsRender::FlatExpanded` | No "MITRE:" line in output for that finding | edge-case (empty) |
| Findings rendered flat (`FindingsRender::FlatCollapsed` or `FindingsRender::FlatExpanded`) | No "## Defense Evasion" header present | happy-path |
| 3 findings all same collapse key, `render = FindingsRender::FlatCollapsed`, member[0].mitre_techniques=["T1036"], member[1].mitre_techniques=[], member[2].mitre_techniques=["T1059"] | MITRE line reads `    MITRE: T1036\n` (from group_members[0]); member[1] and member[2] mitre_techniques are elided from terminal output; all 3 findings' full mitre_techniques preserved in JSON/CSV output (BC-2.11.029) | representative-finding (F-PA-A01 divergent-mitre case) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Default mode uses bare MITRE: <id> format | unit: default_rendering_unchanged_when_mitre_flag_off |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the default rendering contract is the baseline terminal output format; the expanded MITRE line format is an opt-in mode |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-017 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.016 -- contrasts with (grouped mode adds em-dash + name)
- BC-2.11.013 -- contrasts with (grouped mode adds tactic headers; default mode has none)
- BC-2.11.025 -- composes with (v0.8.0 collapse: flat dispatch block routes through collapsed groups when collapse=true; this BC governs the MITRE line format per group representative)
- BC-2.11.026 -- composes with (v0.8.0 collapse: (xN) count suffix on the header line; this BC governs the MITRE line which is a separate output line below the header)
- BC-2.11.028 -- depends on (--no-collapse opt-out; when flag present, collapse=false and this BC's postconditions are byte-identical to pre-v0.8.0)

## Architecture Anchors

- `src/reporter/terminal.rs:296-302` -- render_finding_flat (default path)
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_017_default_mode_bare_mitre_id

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:296-302` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: default_rendering_unchanged_when_mitre_flag_off

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
