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
modified: ["v1.1 2026-06-17: fix Related BCs stale cross-ref BC-2.13.001 (--threats) → BC-2.13.004 (--verbose absent) (consistency audit remediation)", "v1.2 2026-06-17: F2 adversarial pass-1 — change PC-3 from indicative to imperative (code does not exist yet); mark Architecture Anchors as insertion targets pending STORY-118 (F-259-08)", "v1.3 2026-06-17: F2 adversarial pass-9 — F-PA-03: add EC-010 (--no-collapse absent, default --output terminal → collapse applies, default-on)", "v1.4 2026-06-17: F2 adversarial passes 12-14 — F-PB-01: drop '--no-color/--no-reassemble convention' citation (those are global flags; no_collapse is subcommand-scoped); replace with correct subcommand-scoped precedent (#[arg(long)] mitre: bool / dns: bool on Commands::Analyze); fix stale Architecture Anchor cli.rs:151-153 no_reassemble → cli.rs:150-152 mitre: bool (subcommand-scoped boolean precedent)", "v1.5 2026-06-17: issue-#62 F2 BC re-anchor — update all collapse_findings/show_mitre_grouping field-name references to FindingsRender enum: Description wiring note + Preconditions + Postconditions + Invariants 1-2 + EC rows updated. Rationale: illegal-state elimination (FindingsRender makes the three modes structurally exclusive). No behavioral change — the CLI flag wiring and observable output semantics are identical; only the struct field name changes from collapse_findings: bool to render: FindingsRender.", "v1.6 2026-06-18: F3 adversarial round-4 finding 1 (MEDIUM) scope/naming correction — PC3/Inv1/Inv6/Architecture-Anchor prescribed wiring expression used *mitre/no_collapse which are NOT in scope inside run_analyze; they live only in main() (src/main.rs:55-56). run_analyze receives the already-resolved bool params show_mitre_grouping (src/main.rs:107) and collapse_findings (src/main.rs:108). Corrected to in-scope param form. Added explicit note that the --mitre/--no-collapse→bool resolution happens at the main() call site (lines 79-80, UNCHANGED). Behavior identical — scope/naming correction only.", "v1.7 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — TerminalReporter struct REFACTOR TARGET :91-110 → :100-126 (FindingsRender enum at :100-111; TerminalReporter struct at :113-126); Architecture Anchor updated.", "v1.8 2026-06-18: STORY-119 spec-evolution — broaden to DUAL-SCOPE: --no-collapse now suppresses collapse in BOTH flat AND grouped modes. Description wiring updated to struct form (FindingsRender{grouping,collapse} construction). Postconditions 1-2/4 updated to struct form. Invariants 1-2 updated to struct wiring expression. Architecture Anchor updated to struct construction site (STORY-119 F4 target). EC-004/EC-005 updated to struct vocabulary. --mitre default-collapse behavior change noted (--mitre alone → {Grouped, Collapsed} since STORY-119; --mitre --no-collapse → {Grouped, Expanded})."]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.028: --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected

## Description

The `--no-collapse` CLI flag on the `analyze` subcommand provides an explicit opt-out from
the default-on collapse behavior. Since STORY-119, `--no-collapse` is **dual-scope**: it
suppresses collapse in both flat mode AND grouped mode. When `--no-collapse` is absent (the
default), the collapse pass is enabled in both modes. The flag is wired from `cli.rs` through
`main.rs` to the `TerminalReporter` `render` field using the STORY-119 struct construction:
`render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`.

The resulting `render` values are:
- `--no-collapse` absent, `--mitre` absent: `render = {Flat, Collapsed}` (collapse-ON, flat)
- `--no-collapse` present, `--mitre` absent: `render = {Flat, Expanded}` (collapse-OFF, flat)
- `--no-collapse` absent, `--mitre` present: `render = {Grouped, Collapsed}` (collapse-ON, grouped — NEW default since STORY-119)
- `--no-collapse` present, `--mitre` present: `render = {Grouped, Expanded}` (collapse-OFF, grouped — preserves pre-STORY-119 `--mitre` behavior)

`--no-collapse` has no effect on JSON output (`--output json`) or CSV output (`--output csv`).
Both machine-readable formats always emit every finding individually regardless of the flag,
because the collapse pass is a private detail of `TerminalReporter` and is never applied to
`JsonReporter` or `CsvReporter`.

The flag is scoped to the `analyze` subcommand only; it has no effect on the `summary`
subcommand (which has no findings section). It follows the same subcommand-scoped boolean
pattern as `--mitre` and `--dns` on `Commands::Analyze` (cli.rs:150-152), not the global-flag
pattern of `--no-color` or `--no-reassemble` (those are global flags on the top-level `Cli`
struct and are unrelated to subcommand-scoped opt-outs).

## Preconditions

1. The user invokes `wirerust analyze <pcap> [--no-collapse]` (or omits the flag for default
   behavior).
2. The flag is defined as `#[arg(long)]` `no_collapse: bool` on `Commands::Analyze` in
   `src/cli.rs`, following the same subcommand-scoped boolean precedent as `#[arg(long)]
   mitre: bool` / `dns: bool` on `Commands::Analyze` (cli.rs:150-152), destructured in
   `run_analyze` (main.rs:54-64) as `args.no_collapse`.
3. The `no_collapse` field MUST be wired in `src/main.rs` `run_analyze` by STORY-119 (F4):
   `render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }` at the `TerminalReporter` construction site inside `run_analyze`, using the in-scope bool params `show_mitre_grouping` (line 107) and `collapse_findings` (line 108). The `--mitre`/`--no-collapse`→bool resolution happens at the `main()` call site (lines 79-80, UNCHANGED): `show_mitre_grouping == *mitre` and `collapse_findings == !no_collapse` (via `collapse_findings_from_flag`); `run_analyze` signature is UNCHANGED. Per LESSON-P1.04, an unwired flag is a spec violation.

## Postconditions

1. When `--no-collapse` is absent (default) and `--mitre` is absent:
   `TerminalReporter.render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.
   The FINDINGS section renders collapsed groups with ` (xN)` suffixes per BC-2.11.025 and
   BC-2.11.026.
2. When `--no-collapse` is present and `--mitre` is absent:
   `TerminalReporter.render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }`.
   The FINDINGS section renders one display line per raw `Finding` in the input slice. No
   ` (xN)` count suffix appears on any line. Every finding's full evidence is rendered (no
   evidence sampling per BC-2.11.027). The output is byte-identical to the pre-v0.8.0 terminal
   output for the same input.
3. In all modes, `JsonReporter` and `CsvReporter` receive the complete, unmodified
   `findings: &[Finding]` slice and render every finding individually. The `--no-collapse`
   flag has no observable effect on JSON or CSV output.
4. **DUAL-SCOPE (STORY-119):** `--no-collapse` acts as a universal collapse suppressor across
   BOTH the flat and grouped rendering paths:
   - `--mitre` absent, `--no-collapse` present: `render = {Flat, Expanded}` — no flat collapse.
   - `--mitre` present, `--no-collapse` absent: `render = {Grouped, Collapsed}` — grouped
     collapse IS applied per-bucket (BC-2.11.031). This is the NEW default for `--mitre` since
     STORY-119.
   - `--mitre` present, `--no-collapse` present: `render = {Grouped, Expanded}` — no grouped
     collapse; each finding rendered individually via `render_finding_grouped`; no ` (xN)`
     suffix. This preserves the pre-STORY-119 `--mitre` behavior via the explicit opt-out.
   The `collapse` axis of the `FindingsRender` struct is determined exclusively by
   `collapse_findings`; the `grouping` axis by `show_mitre_grouping`. They are fully
   orthogonal.

## Invariants

1. The `no_collapse` field on `Commands::Analyze` is a boolean. It is `true` when the flag
   is present, `false` when absent. The `--mitre`/`--no-collapse`→bool resolution happens at
   the `main()` call site (lines 79-80): `show_mitre_grouping == *mitre` and
   `collapse_findings == !no_collapse` (via `collapse_findings_from_flag`). Inside
   `run_analyze`, the `TerminalReporter.render` field is constructed using the STORY-119
   struct form (F4 target):
   `render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`.
   The two axes are fully orthogonal: no combination is illegal.
2. Default behavior (both flags absent) is `{Flat, Collapsed}` (collapse-ON, flat rendering).
   This is intentional: the canonical terminal UX is the collapsed view. When `--mitre` is
   present (and `--no-collapse` absent), the default is now `{Grouped, Collapsed}` — symmetric
   collapse-on behavior for both rendering modes (STORY-119 D-110 approved behavior change).
   Analysts who require per-finding lines can explicitly opt out with `--no-collapse` in either mode.
3. The flag is a pure boolean. It does not accept a value (not `--no-collapse=true`). There
   is no `--collapse` alias in v0.8.0.
4. The flag is `analyze`-subcommand-scoped. The `summary` subcommand has no `no_collapse`
   field because it emits no findings section.
5. The flag must be documented in the `--help` output for `analyze`. The help text must
   describe both the default-on behavior and the opt-out semantics, and must clarify that
   JSON/CSV output is unaffected.
6. Per LESSON-P1.04 (no unwired flags): the `no_collapse` field in `cli.rs` MUST be wired
   to `TerminalReporter.render` in `main.rs` via the two-field struct expression in Invariant 1
   (STORY-119 F4 target), using the in-scope `show_mitre_grouping` and `collapse_findings`
   params inside `run_analyze`. The `*mitre`/`no_collapse` names from `Commands::Analyze` are
   resolved at the `main()` call site (lines 79-80) before being passed as bool params to
   `run_analyze`. An unwired flag is a spec violation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --no-collapse absent, --mitre absent (→ `render = {Flat, Collapsed}`), 5 identical findings | Terminal: collapsed, `(x5)` suffix; JSON: 5 finding objects |
| EC-002 | --no-collapse present, --mitre absent (→ `render = {Flat, Expanded}`), 5 identical findings | Terminal: 5 individual lines, no suffix; JSON: 5 finding objects |
| EC-003 | --no-collapse present, --mitre absent, all findings are unique | Terminal output byte-identical to pre-v0.8.0 (no collapse was happening anyway) |
| EC-004 | --no-collapse present with --mitre (→ `render = {Grouped, Expanded}`) | Grouped mode renders each finding individually via `render_finding_grouped`; no ` (xN)` suffix. Preserves the pre-STORY-119 `--mitre` behavior explicitly via opt-out. |
| EC-005 | --no-collapse absent with --mitre (→ `render = {Grouped, Collapsed}`) | Grouped-collapse mode: per-bucket collapse applies (BC-2.11.031); `(xN)` suffix on N≥2 groups within each tactic bucket. This is the new STORY-119 default for `--mitre`. |
| EC-006 | --no-collapse with --output json | JSON output is identical to --no-collapse absent with --output json; the flag has no effect on JsonReporter |
| EC-007 | --no-collapse with --output csv | CSV output is identical to --no-collapse absent with --output csv; the flag has no effect on CsvReporter |
| EC-008 | summary subcommand invoked (no --no-collapse field) | No error; summary subcommand has no findings section and no no_collapse field; unaffected |
| EC-009 | --no-collapse present, no findings in pcap | Empty FINDINGS section (or absent section) as usual; no error |
| EC-010 | --no-collapse absent, default --output (terminal) | Collapse applies (default-on); `TerminalReporter.render = FindingsRender::FlatCollapsed`; collapsed groups with (xN) suffixes rendered per BC-2.11.025/026 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 identical `(Anomaly, Inconclusive, Low, "Empty UA")` findings, `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` (--no-collapse, flat) | FINDINGS section has 5 individual lines, no `(x5)` suffix anywhere | happy-path (flat opt-out) |
| 5 identical findings, `render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (default) | FINDINGS section has 1 collapsed line with `(x5)` suffix | happy-path (flat default) |
| 5 identical findings rendered to JSON reporter (any `render` variant) | JSON output contains 5 finding objects regardless | happy-path (JSON unaffected) |
| 5 identical findings in tactic bucket, `render = FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` (`--mitre` alone, new default) | FINDINGS section has tactic bucket header + 1 collapsed line with `(x5)` suffix + K=3 evidence + MITRE line | happy-path (grouped-collapsed default, STORY-119) |
| 5 identical findings in tactic bucket, `render = FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`) | FINDINGS section has tactic bucket header + 5 individual lines, no suffix | happy-path (grouped-expanded opt-out) |
| --no-collapse present, mix of 1 unique + 3 identical findings | 4 individual lines, no suffixes | mixed scenario (opt-out) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `render = {Flat, Expanded}` (--no-collapse, flat) produces one line per finding, no (xN) suffix | unit: test_BC_2_11_028_no_collapse_flag_one_line_per_finding |
| — | `{Flat, Collapsed}` (default) vs `{Flat, Expanded}`: output differs for repeated findings | unit: test_BC_2_11_028_default_vs_opt_out_output_difference |
| — | JSON reporter output identical regardless of `render` variant | integration: test_BC_2_11_029_json_receives_full_findings (cross-BC) |
| — | Flag is wired (`no_collapse=true` → `render.collapse == Collapse::Expanded` in TerminalReporter) | unit: test_BC_2_11_028_flag_wired_to_reporter_field |
| — | `--no-collapse` with `--mitre` produces `{Grouped, Expanded}` (suffix-free grouped mode) | unit: test_BC_2_11_028_no_collapse_with_mitre_produces_grouped_expanded |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the --no-collapse opt-out flag is a direct output control that the Reporting and Output capability must expose so analysts can select between collapsed and expanded terminal views; it is the user-facing toggle for the display mode introduced in v0.8.0 |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- the flag controls only the terminal display layer; JSON/CSV raw output is invariant to it) |
| Architecture Module | SS-11 (reporter/terminal.rs, src/cli.rs, src/main.rs) |
| Stories | STORY-118 |
| Issue | #259 (Collapse repeated low-value findings) |
| ADR | ADR-0003 (display-layer aggregation subsection) |

## Related BCs

- BC-2.11.025 -- depends on (the collapse pass this flag disables)
- BC-2.11.026 -- depends on (the count suffix rendering this flag suppresses when off)
- BC-2.11.029 -- composes with (JSON/CSV raw-stream invariant; this flag does not affect those reporters)
- BC-2.13.004 -- context (--verbose Does Not Exist is BC-2.13.004; --no-collapse is the new negation flag in this family, following the same absent-flag documentation pattern)

## Architecture Anchors

- `src/cli.rs:150-152` -- `#[arg(long)] mitre: bool` (subcommand-scoped boolean precedent on `Commands::Analyze`; same pattern as `no_collapse`) (existing code; reference only)
- `src/main.rs:~run_analyze` -- **F4-pending STORY-119 target:** `render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }` at TerminalReporter construction site, using the in-scope bool params `show_mitre_grouping` (line 107) and `collapse_findings` (line 108). The `--mitre`/`--no-collapse`→bool resolution happens at `main()` lines 79-80 (UNCHANGED): `*mitre` → `show_mitre_grouping`; `collapse_findings_from_flag(*no_collapse)` → `collapse_findings`. Replaces the v0.9.0 three-arm if-expression for `FindingsRender` enum.
- `src/reporter/terminal.rs:100-126` -- **F4-pending replacement target:** current v0.9.0 `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` at :100-111 replaced by `pub enum Grouping { Grouped, Flat }` + `pub enum Collapse { Collapsed, Expanded }` + `pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }` per D-110 (story-119-type-design.md §2.1). `pub struct TerminalReporter` with `pub render: FindingsRender` field at :113-126 is structurally unchanged (field name `render` remains; field type changes from enum to struct).

## Story Anchor

STORY-118

## VP Anchors

- — (new VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (flag parsing is I/O; this BC governs the downstream behavioral effect) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (TerminalReporter itself is pure; CLI parsing is effectful upstream) |
