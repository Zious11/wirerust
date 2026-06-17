---
document_type: story
story_id: STORY-118
epic_id: E-18
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-06-17T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: []
blocks: [STORY-119]
behavioral_contracts:
  - BC-2.11.010
  - BC-2.11.013
  - BC-2.11.017
  - BC-2.11.019
  - BC-2.11.025
  - BC-2.11.026
  - BC-2.11.027
  - BC-2.11.028
  - BC-2.11.029
verification_properties: [VP-012]
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
feature_id: e18-finding-collapse
github_issue: 259
wave: 47
# BC status: BC-2.11.025 v1.5, BC-2.11.026 v1.8, BC-2.11.027 v1.3, BC-2.11.028 v1.4,
#             BC-2.11.029 v1.2, BC-2.11.010 v1.8, BC-2.11.013 v1.11, BC-2.11.017 v1.13,
#             BC-2.11.019 v1.6 — all authored and CONVERGED (F2 passes 1-14).
# Subsystem anchor: SS-11 owns this story's scope because terminal finding-collapse
#   is a display-layer transform in reporter/terminal.rs (SS-11 per ARCH-INDEX
#   Subsystem Registry). The --no-collapse flag addition touches src/cli.rs and
#   src/main.rs but the wiring is a thin SS-12 glue that does not create a separate
#   story — per the cli.rs:150-152 mitre/dns precedent (BC-2.11.028 v1.4).
# ADR note: STORY-118's implementation PR MUST carry the uncommitted ADR-0003
#   "Display-Layer Aggregation" working-tree change from docs/adr/0003-reporting-pipeline-layering.md.
#   This change exists on develop (uncommitted) and must ride the STORY-118 PR to keep
#   the decision record in sync with the shipped v0.8.0 feature.
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.029.md
  - .factory/phase-f1-delta-analysis/issue-259-finding-collapse-delta-analysis.md
input-hash: "432f43e"
---

# STORY-118: Terminal Finding-Collapse — Flat Mode (v0.8.0)

## Narrative

- **As a** network security analyst using wirerust to triage high-volume pcap captures
- **I want** repeated identical findings in the terminal output to be collapsed into a
  single annotated group with a ` (xN)` count suffix, with full evidence and JSON/CSV
  output unaffected
- **So that** I can quickly identify noise patterns (e.g., an empty-User-Agent flood
  of 10,000 requests) without wading through thousands of identical terminal lines,
  while retaining forensic completeness in all machine-readable formats

## Behavioral Contracts

| BC | Version | Title |
|----|---------|-------|
| BC-2.11.025 | v1.5 | Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic |
| BC-2.11.026 | v1.8 | Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix |
| BC-2.11.027 | v1.3 | Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display |
| BC-2.11.028 | v1.4 | --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected |
| BC-2.11.029 | v1.2 | Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs |
| BC-2.11.010 | v1.8 | TerminalReporter Escapes Both Summary AND Each Evidence Line |
| BC-2.11.013 | v1.11 | MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last |
| BC-2.11.017 | v1.13 | Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash) |
| BC-2.11.019 | v1.6 | TerminalReporter Renders Sections in Correct Order |

## Acceptance Criteria

**Pass-7 process note:** Because no formal VP is assigned to the new collapse BCs
(test-sufficient per verification-coverage-matrix v1.12), every AC test named below
MUST physically exist in the test suite after F4 implementation. The test names are
the binding quality handle for these BCs. The test-writer and implementer MUST create
each named test; a test that exists only as an AC citation but has no corresponding
function body is a spec violation.

---

### AC-001 — Default-ON collapse: N identical findings produce one group
*(traces to BC-2.11.025 postcondition 1)*

Given `TerminalReporter` with `collapse_findings=true` (the default) and
`show_mitre_grouping=false`, when `render()` receives N≥2 findings all sharing the
same `(category, verdict, confidence, summary)` four-tuple key, then the rendered
FINDINGS section contains exactly one header line for that key (no duplicate lines).

- **Test:** `test_BC_2_11_025_identical_findings_collapse_to_one_group`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-002 — First-occurrence order preserved across groups
*(traces to BC-2.11.025 postcondition 2)*

Given a `findings` slice containing: 3 findings with key A at indices 0, 2, 4 and
2 findings with key B at indices 1, 3, when collapsed, the output renders group A
first (position 0 in input) then group B second (position 1 in input), regardless
of the insertion order of subsequent matches.

- **Test:** `test_BC_2_11_025_first_occurrence_order`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-003 — Four-field key: evidence difference does NOT prevent collapse
*(traces to BC-2.11.025 postcondition 4)*

Given two findings that share `(category, verdict, confidence, summary)` but differ
only in `evidence`, when `collapse_findings=true`, the two findings are merged into
one display group with count N=2.

- **Test:** `test_BC_2_11_025_key_discriminator_evidence_nondiscriminating`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-004 — Four-field key: category difference prevents collapse
*(traces to BC-2.11.025 invariant 1)*

Given two findings that share `verdict`, `confidence`, and `summary` but differ
in `category`, when collapsed, the output renders two distinct display groups.

- **Test:** `test_BC_2_11_025_key_discriminator_category`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-005 — show_mitre_grouping=true suppresses collapse; no (xN) suffix ever
*(traces to BC-2.11.025 invariant 5 / BC-2.11.013 invariant 4)*

Given `TerminalReporter` with `collapse_findings=true` AND `show_mitre_grouping=true`,
when `render()` receives N=100 identical-key findings, the output contains 100
individual finding lines and NO ` (xN)` suffix appears on any line in the FINDINGS
section. The observable guarantee: the structured grouped path is structurally
suffix-free at any input volume.

- **Test:** `test_BC_2_11_025_grouped_mode_bypasses_collapse`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_013_grouped_mode_suffix_free`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-006 — Severity-agnostic collapse: Likely/High findings collapse normally
*(traces to BC-2.11.025 postcondition 7 / edge case EC-014)*

Given two findings both with `verdict=Likely`, `confidence=High`, and the same
`summary`, when `collapse_findings=true`, the two findings collapse into one group
with ` (x2)` suffix — the collapse is severity-agnostic and applies to all verdicts
and confidences equally.

- **Test:** `test_BC_2_11_025_severity_agnostic_collapse_likely_high`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-007 — Raw-byte key: ESC-byte summary distinguishes two groups
*(traces to BC-2.11.025 postcondition 8 / invariant 1 / edge case EC-015)*

Given two findings where `finding[0].summary = "x\x1b"` (raw ESC byte) and
`finding[1].summary = "x"`, when `collapse_findings=true`, the two findings form
two distinct display groups — raw-byte string comparison is used for the key; the
escape pass occurs at render time, not during key construction.

- **Test:** `test_BC_2_11_025_raw_byte_key_esc_distinguishes_groups`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-008 — N=1 singleton: no count suffix; output byte-identical to pre-v0.8.0
*(traces to BC-2.11.026 postcondition 2 / invariant 2)*

Given a single finding (N=1) with `collapse_findings=true`, the rendered header
line contains no ` (x1)` suffix and no ` (xN)` suffix of any kind. The complete
output line is byte-identical to what `render_finding_flat` / `render_finding_prefix`
would have produced before v0.8.0 for that finding.

- **Test:** `test_BC_2_11_026_singleton_no_suffix`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-009 — N≥2 group: header line contains correct (xN) suffix with exact count
*(traces to BC-2.11.026 postcondition 1 / invariant 1)*

Given 3 identical-key findings, when `collapse_findings=true`, the rendered header
line contains the string `"Empty UA (x3)"` (or equivalent with the actual summary).
Given 3142 identical-key findings, the header contains `"(x3142)"` — no rounding,
no abbreviation.

- **Test:** `test_BC_2_11_026_count_suffix_for_n_ge_2`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_026_large_count_exact`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-010 — Suffix format: space-paren-x-integer-paren; no alternative format
*(traces to BC-2.11.026 invariant 1)*

The suffix format is exactly ` (x<N>)` — one leading space, open-paren, literal `x`,
decimal integer with no leading zeros, close-paren. No alternative forms (e.g., `[x2]`,
`(2x)`, ` x2`) are emitted.

- **Test:** `test_BC_2_11_026_suffix_format`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-011 — (xN) suffix is INSIDE the color span; colorized with full header line
*(traces to BC-2.11.026 postcondition 6 / invariant 4)*

Given 2 findings with `(Reconnaissance, Likely, High, "Port scan")` and
`use_color=true`, when `collapse_findings=true`, the rendered header output
wraps the COMPLETE header string including ` (x2)` inside the `red().bold()`
ANSI color span. The ` (x2)` suffix is NOT appended after the color-reset sequence.
The observable test: the output byte sequence contains `(x2)` before the ANSI reset.

- **Test:** `test_BC_2_11_026_suffix_colorized_inside_span_red_bold`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-012 — Color-ladder: Inconclusive→cyan; Likely+other→yellow; Unlikely→dimmed
*(traces to BC-2.11.026 postcondition 6 — color-ladder full coverage)*

The same color-ladder logic from `terminal.rs:209-221` (Likely+High→red().bold(),
Likely+other→yellow, Possible→yellow, Inconclusive→cyan, Unlikely→dimmed) is applied
to the pre-suffix string BEFORE colorization. Each branch is covered by at least one
test.

- **Test:** `test_BC_2_11_026_color_ladder_inconclusive_cyan`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_026_color_ladder_likely_other_yellow`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_026_color_ladder_possible_yellow`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_026_color_ladder_unlikely_dimmed`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-013 — Evidence sampling: N>K shows exactly K=3 lines (first K members)
*(traces to BC-2.11.027 postcondition 2 / invariant 2)*

Given 5 identical-key findings, each with exactly one evidence line (`"req_001"`
through `"req_005"`), when `collapse_findings=true`, the terminal output contains
exactly 3 evidence lines: `> req_001`, `> req_002`, `> req_003`. Evidence for
members[3] and members[4] is elided from terminal output.

- **Test:** `test_BC_2_11_027_evidence_capped_at_k`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-014 — Evidence sampling: N≤K renders all available evidence
*(traces to BC-2.11.027 postcondition 5)*

Given 2 identical-key findings each with 1 evidence line, when `collapse_findings=true`,
both evidence lines are rendered (no elision — N≤K=3).

Given 3 identical-key findings each with 1 evidence line (N=K boundary), all 3
evidence lines are rendered (no elision at the boundary).

- **Test:** `test_BC_2_11_027_evidence_below_cap_rendered_fully`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-015 — Evidence sampling: empty first-member contributes 0 lines; window does NOT slide
*(traces to BC-2.11.027 postcondition 2 / invariant 2 — positional no-slide rule)*

Given 5 identical-key findings where `members[0].evidence = []` and `members[1..4]`
each have 1 evidence line, when `collapse_findings=true`, the positional window inspects
`members[0]`, `members[1]`, `members[2]` (first min(5,3)=3 members). `members[0]`
contributes 0 lines (empty vec; window does NOT slide). Total rendered evidence = 2
lines (from `members[1]` and `members[2]`). `members[3]` and `members[4]` are never
inspected.

- **Test:** `test_BC_2_11_027_evidence_drawn_from_first_k_members`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-016 — Evidence lines pass through escape_for_terminal in the collapse path
*(traces to BC-2.11.027 postcondition 6 / BC-2.11.010 invariant 4)*

Given an evidence line containing a raw ESC byte (`"\x1b[31m"`), when rendered in
a collapsed group, the output is `> \\x1b[31m` (escaped). The collapse wrapper calls
`escape_for_terminal` directly on each sampled line — the function-level escape
guarantee is preserved.

- **Test:** `test_BC_2_11_027_escape_preserved_in_sampled_evidence`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-017 — Singleton: K-cap does NOT apply; all evidence lines rendered
*(traces to BC-2.11.027 invariant 6)*

Given a singleton group (N=1) with 5 evidence lines, when `collapse_findings=true`,
all 5 evidence lines are rendered — the K=3 cap does not apply to singletons.
Output is byte-identical to pre-v0.8.0 behavior for that finding.

- **Test:** `test_BC_2_11_027_singleton_evidence_not_capped`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-018 — --no-collapse restores one-line-per-finding; no (xN) suffix anywhere
*(traces to BC-2.11.028 postcondition 2)*

Given `collapse_findings=false` (i.e., `--no-collapse` flag present) and 5
identical-key findings, the terminal output contains 5 individual header lines,
no ` (x5)` suffix appears anywhere, and all evidence is rendered in full per finding.
The output is byte-identical to pre-v0.8.0 terminal output for the same input.
When `collapse_findings=true` (default) and `collapse_findings=false` (opt-out) are
compared against the same 5-finding identical-key input, the two outputs are observably
different: the default output has 1 header with ` (x5)` and 3 evidence lines; the
opt-out output has 5 individual headers each with their own evidence.

- **Test:** `test_BC_2_11_028_no_collapse_flag_one_line_per_finding`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_028_default_vs_opt_out_output_difference`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-019 — --no-collapse flag is wired: no_collapse=true → collapse_findings=false
*(traces to BC-2.11.028 postcondition 1 / invariant 1 / BC-2.11.028 precondition 3)*

The `no_collapse: bool` field on `Commands::Analyze` in `src/cli.rs` is wired via
the following pattern: `no_collapse` is destructured from `Commands::Analyze` in
`main()` (main.rs:49-64), threaded as a new positional `bool` parameter into
`run_analyze`, and set as `collapse_findings: !no_collapse` at the `TerminalReporter
{ … }` construction site (main.rs:370-376). This mirrors exactly how `*mitre` becomes
`show_mitre_grouping`. An unwired flag (one that appears in `cli.rs` but is never
referenced in `main.rs`) is a spec violation per BC-2.11.028 invariant 6 / LESSON-P1.04.

- **Test:** `test_BC_2_11_028_flag_wired_to_reporter_field`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-020 — JSON reporter: N identical findings produce N objects regardless of collapse flag
*(traces to BC-2.11.029 postcondition 1 / invariant 1)*

Given 1000 identical `(Anomaly, Inconclusive, Low, "Empty UA")` findings rendered with
`collapse_findings=true` to both terminal and JSON reporters using the same `findings`
slice, the JSON output contains exactly 1000 finding objects. The terminal output
contains exactly 1 collapsed group with ` (x1000)`. The collapse pass is ephemeral
inside `TerminalReporter::render` and never touches the `findings` slice.

- **Test:** `test_BC_2_11_029_json_receives_full_findings`
  (in `tests/reporter_terminal_tests.rs` or `tests/reporter_json_tests.rs`)

---

### AC-021 — CSV reporter: N identical findings produce N rows regardless of collapse flag
*(traces to BC-2.11.029 postcondition 2)*

Given 5 identical-key findings rendered with `collapse_findings=true` to both terminal
and CSV reporters, the CSV body contains exactly 5 data rows (one per finding). The
`--no-collapse` flag does not affect CSV output.

- **Test:** `test_BC_2_11_029_csv_receives_full_findings`
  (in `tests/reporter_terminal_tests.rs` or `tests/reporter_csv_tests.rs`)

---

### AC-022 — Non-repeated finding renders individually; no suffix; byte-identical to pre-v0.8.0
*(traces to BC-2.11.029 postcondition 3)*

Given a finding that appears exactly once in the input slice (unique key), when
`collapse_findings=true`, the finding is rendered without a count suffix. Its terminal
output is identical to the pre-v0.8.0 single-finding rendering (same as singleton
group behavior — BC-2.11.026 postcondition 2).

- **Test:** `test_BC_2_11_029_non_repeated_finding_no_suffix`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-027 — --no-collapse flag does NOT affect JSON or CSV output
*(traces to BC-2.11.029 postcondition 5)*

Given the same `findings` slice rendered to JSON and CSV reporters, the presence or
absence of `--no-collapse` (i.e., `collapse_findings=true` vs `collapse_findings=false`)
has no observable effect on JSON or CSV output. Both formats always emit all N finding
objects/rows regardless of the terminal collapse setting. The `collapse_findings` field
is an attribute of `TerminalReporter` only and is not consulted by `JsonReporter` or
`CsvReporter`.

- **Test:** `test_BC_2_11_029_no_collapse_flag_json_invariant`
  (in `tests/reporter_terminal_tests.rs` or `tests/reporter_json_tests.rs`)

---

### AC-028 — escape_for_terminal is called on each sampled evidence line in the collapse path
*(traces to BC-2.11.010 invariant 4)*

Given a collapsed group where one of the first K=3 member findings has an evidence line
containing a raw ESC byte (`"\x1b[31minjected\x1b[0m"`), when `collapse_findings=true`,
the rendered evidence line for that member is `> \x1b[31minjected\x1b[0m` (escaped —
raw ESC bytes replaced with their `\xNN` representation). The collapse path calls
`escape_for_terminal` directly on each sampled evidence line; the function-level
escape guarantee (BC-2.11.010 invariant 4) is not weakened by the collapse wrapper.

- **Test:** `test_BC_2_11_010_escape_in_collapse_path`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-023 — MITRE line sources group_members[0]; other members' MITRE elided from terminal
*(traces to BC-2.11.026 postcondition 7 / BC-2.11.017 postcondition 6)*

Given 3 findings all sharing the same collapse key where `members[0].mitre_techniques
= ["T1036"]`, `members[1].mitre_techniques = []`, `members[2].mitre_techniques =
["T1059"]`, when `collapse_findings=true`, the terminal MITRE line reads
`    MITRE: T1036\n` (from `group_members[0]`). Members[1] and members[2]
`mitre_techniques` are elided from terminal output. All 3 findings' full
`mitre_techniques` are preserved in JSON/CSV output (BC-2.11.029).

- **Test:** `test_BC_2_11_026_mitre_line_from_representative_finding`
  (in `tests/reporter_terminal_tests.rs`)
- **Test:** `test_BC_2_11_017_collapsed_mitre_line_from_representative`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-024 — Collapse uses Vec accumulator with linear-scan PartialEq; no HashMap
*(traces to BC-2.11.025 postcondition 9 / invariant 7)*

The collapse pass MUST use a `Vec<(CollapseKey, Vec<&Finding>)>` insertion-ordered
accumulator with linear-scan `PartialEq` matching. A `HashMap` (non-deterministic
iteration order) is prohibited. This is verifiable by determinism test: given any
input slice, two successive calls to `render()` with the same input produce byte-identical
output.

- **Test:** `test_BC_2_11_025_deterministic_output_same_input`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-025 — Section order unchanged; collapse affects only FINDINGS body content
*(traces to BC-2.11.019 postcondition 9 / invariant 7)*

The overall section order (WIRERUST TRIAGE REPORT → PROTOCOLS → SERVICES → FINDINGS →
ANALYZER: sections) is unchanged when `collapse_findings=true`. Only the FINDINGS
section body content changes (collapsed groups instead of individual lines). The section
header (`FINDINGS\n`) and the `if !findings.is_empty()` guard are preserved.

- **Test:** `test_BC_2_11_019_section_order_unchanged_with_collapse`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-026 — Flood canonical case: 5 empty-UA findings collapse to 1 group with 3 evidence lines
*(traces to BC-2.11.025 canonical test vector / BC-2.11.027 postcondition 2)*

Given 5 findings all `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` with
distinct evidence strings `["GET /a"]`, ..., `["GET /e"]` (format: `method + " " + uri`,
no HTTP version token — per `src/analyzer/http.rs:365` `format!("{} {}", method, uri)`)
and timestamps that MAY differ (timestamp is a non-key field), when `collapse_findings=true`,
the FINDINGS section contains exactly 1 display group with count 5. Exactly 3 evidence
lines are rendered: `> GET /a`, `> GET /b`, `> GET /c`.
Evidence for `/d` and `/e` is elided from terminal output.

- **Test:** `test_BC_2_11_025_flood_canonical_empty_ua_five_findings`
  (in `tests/reporter_terminal_tests.rs`)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `TerminalReporter::render` — collapse pass (new) | `src/reporter/terminal.rs` | Pure core |
| `TerminalReporter.collapse_findings: bool` (new field) | `src/reporter/terminal.rs:63-75` (insertion target) | Pure data |
| `CollapseKey` type alias or struct (new) | `src/reporter/terminal.rs` | Pure data |
| `COLLAPSE_EVIDENCE_SAMPLES: usize = 3` (new const) | `src/reporter/terminal.rs` | Compile-time constant |
| Flat FINDINGS dispatch block (extended) | `src/reporter/terminal.rs:149-162` | Pure core |
| `escape_for_terminal` function (existing; called directly by collapse wrapper) | `src/reporter/terminal.rs` | Pure |
| `--no-collapse` flag: `no_collapse: bool` on `Commands::Analyze` (new) | `src/cli.rs` | Effectful (CLI parsing) |
| `collapse_findings: !no_collapse` wiring (new) — `no_collapse` destructured from `Commands::Analyze` in `main()`, threaded as positional bool into `run_analyze` | `src/main.rs` | Effectful (glue) |
| `JsonReporter::render` (unchanged) | `src/reporter/json.rs` | Pure |
| `CsvReporter::render` (unchanged) | `src/reporter/csv.rs` | Pure |

Architecture section references:
- `architecture/module-decomposition.md` (SS-11 C-20, `src/reporter/terminal.rs`)
- ADR-0003 (`docs/adr/0003-reporting-pipeline-layering.md` — Display-Layer Aggregation section; **this ADR change MUST ride STORY-118's PR, see Tasks**)

## Subsystem Anchor Justification

SS-11 owns this story's scope because terminal finding-collapse is a display-layer
transform in `src/reporter/terminal.rs` — the core SS-11 module per ARCH-INDEX Subsystem
Registry. The `--no-collapse` CLI flag addition touches `src/cli.rs` (SS-12) and
`src/main.rs` (SS-12) only as thin wiring glue, following the exact same subcommand-scoped
boolean precedent as `--mitre` and `--dns` (BC-2.11.028 v1.4). This is insufficient scope
to split into a separate SS-12 story.

## Dependency Anchor Justification

- `depends_on: []` — STORY-118 extends SS-11 (`TerminalReporter`) which already exists
  (landed in STORY-077/078). The collapse feature adds a field, a constant, and a private
  collapse method — no new story predecessor is required. The `--no-collapse` flag follows
  the established `--mitre`/`--dns` pattern in SS-12 (STORY-087 wired those; the pattern
  is known-good). No new build-order dependency is introduced.
- `blocks: [STORY-119]` — STORY-119 (grouped-mode collapse) depends on STORY-118's
  `collapse_findings` field and the `CollapseKey` type existing in `terminal.rs` before
  the grouped path can be extended with collapse logic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All N findings identical in all four key fields | One display group with count N (AC-001, AC-026) |
| EC-002 | All findings have distinct keys | N display groups, all singleton (N=1 each), no suffix (AC-022) |
| EC-003 | Two findings share key but differ only in evidence | Collapsed into one group (AC-003) |
| EC-004 | summary contains raw ESC byte vs clean summary | Two distinct groups (raw-byte key) (AC-007) |
| EC-005 | show_mitre_grouping=true AND collapse_findings=true | Collapse NOT applied; 0 (xN) suffixes (AC-005) |
| EC-006 | --no-collapse present, 5 identical findings | 5 individual lines, no suffix (AC-018) |
| EC-007 | N≥2 with use_color=true, Likely+High verdict | (x2) suffix inside red().bold() span (AC-011) |
| EC-008 | Group members[0] has empty evidence, others have 1 each | 2 evidence lines (not 3); window does not slide (AC-015) |
| EC-009 | N=1 singleton, 5 evidence lines | All 5 lines rendered; K-cap not applied (AC-017) |
| EC-010 | 1000 findings to terminal + JSON | Terminal: 1 group (x1000); JSON: 1000 objects (AC-020) |
| EC-011 | divergent mitre_techniques across group members | MITRE line from group_members[0] only (AC-023) |

## Tasks

1. **[F4 scope — RED]** Read `src/reporter/terminal.rs` (full file) and `src/cli.rs`
   (Commands::Analyze block) and `src/main.rs` (run_analyze function) to confirm current
   state of `TerminalReporter` struct (no `collapse_findings` field yet), the flat FINDINGS
   dispatch block (terminal.rs:149-162), and the `--mitre`/`--dns` boolean precedent
   (cli.rs:150-152).
2. **[F4 scope — RED]** Create `tests/reporter_terminal_tests.rs` mod block
   `mod story_118` (or equivalent) with all 26 test functions named in ACs 001-026 above.
   All test function bodies MUST use `todo!()` (or equivalent) — the Red Gate density
   check requires ≥50% `todo!()` bodies before the implementer is dispatched.
3. **[F4 scope — GREEN — cli.rs]** Add `#[arg(long)] no_collapse: bool` to
   `Commands::Analyze` in `src/cli.rs`, following the `mitre: bool` / `dns: bool`
   subcommand-scoped boolean precedent at cli.rs:150-152 (BC-2.11.028 precondition 2).
4. **[F4 scope — GREEN — terminal.rs struct]** Add `collapse_findings: bool` field to
   `TerminalReporter` struct (insertion target: terminal.rs:63-75).
5. **[F4 scope — GREEN — terminal.rs const]** Add `const COLLAPSE_EVIDENCE_SAMPLES: usize = 3`
   (BC-2.11.027 invariant 1).
6. **[F4 scope — GREEN — terminal.rs collapse pass]** Implement the private
   `collapse_findings_pass<'a>(findings: &'a [Finding]) -> Vec<(CollapseKey, Vec<&'a Finding>)>`
   function using a `Vec` accumulator with linear-scan `PartialEq` matching. No HashMap.
   No IndexMap. `CollapseKey` is a struct or 4-tuple `(ThreatCategory, Verdict, Confidence, String)`
   derived from `PartialEq` only (BC-2.11.025 invariant 7).
7. **[F4 scope — GREEN — terminal.rs dispatch]** Extend the flat FINDINGS dispatch block
   at terminal.rs:149-162: check `self.collapse_findings`. When `true`, call the collapse
   pass and render each group with: (1) header + ` (x<N>)` suffix when N≥2 (colorized
   per the color-ladder including the suffix); (2) up to K=3 sampled evidence lines calling
   `escape_for_terminal` directly; (3) MITRE line from `group_members[0]` if
   `mitre_techniques` is non-empty (BC-2.11.026 PC-4 observable line order).
8. **[F4 scope — GREEN — main.rs]** Destructure `no_collapse` from `Commands::Analyze`
   in `main()` (main.rs:49-64), add it as a new positional `bool` parameter to
   `run_analyze`, and set `collapse_findings: !no_collapse` at the `TerminalReporter
   { … }` construction site (main.rs:370-376). This mirrors exactly how `*mitre`
   becomes `show_mitre_grouping` (BC-2.11.028 precondition 3 / invariant 6).
   - **Second construction site (run_summary, ~main.rs:432):** adding `collapse_findings`
     to `TerminalReporter` requires the `run_summary` construction site to also initialize
     the new field (Rust requires all struct fields). Set `collapse_findings: true` there
     (the value is inert — `run_summary` emits no FINDINGS section; BC-2.11.028 Invariant 4
     is analyze-scoped). This is the only `run_summary` change; the field is initialized
     for completeness, not to enable collapse behavior in the summary path.
9. **[ADR obligation]** STORY-118's implementation PR MUST include the uncommitted
   `docs/adr/0003-reporting-pipeline-layering.md` "Display-Layer Aggregation" section
   addition. This change currently exists uncommitted on `develop`. It must ride the
   STORY-118 PR — do NOT commit it as a standalone commit before the feature lands, as
   the ADR documents a design decision that is not yet implemented.
10. **[F4 scope — REFACTOR + verify]** Run `cargo test --all-targets` green. Run
    `cargo clippy --all-targets -- -D warnings` clean. Run `cargo fmt --check` clean.
11. **(Post-delivery)** Compute and update `input-hash:` via
    `bin/compute-input-hash --write .factory/stories/STORY-118.md`.

## Previous Story Intelligence

This story extends E-8 (Reporting and Output Formats). Key predecessor lessons:

**STORY-077** (TerminalReporter — escape_for_terminal): Established `escape_for_terminal`
function and `TerminalReporter` struct. The `collapse_findings` field is added to the same
struct.

**STORY-078** (TerminalReporter — MITRE grouping, section order, colorization): Established
the `show_mitre_grouping` guard (the flat vs. grouped branch at terminal.rs:149-162), the
color-ladder logic at terminal.rs:209-221, and `render_finding_prefix` / `render_finding_flat`
/ `render_findings_grouped`. STORY-118's collapse pass is inserted into the flat branch
of this existing dispatch. The color-ladder logic is REUSED (not reimplemented) — the
collapse-aware wrapper builds the pre-suffix header string, appends ` (xN)`, then passes
the complete string through the existing color-ladder.

**STORY-087** (Output Format Flags and Reassembly Config Flags): Established the pattern
for subcommand-scoped boolean flags (`--mitre`, `--dns`). The `--no-collapse` flag follows
the exact same pattern.

**Key lesson (from adversarial convergence):** Do NOT use `HashMap` or `IndexMap` for the
collapse accumulator. `ThreatCategory`, `Verdict`, and `Confidence` derive `PartialEq` but
NOT `Hash` in v0.8.0. `indexmap` is not in `Cargo.toml`. The canonical implementation is
`Vec<(CollapseKey, Vec<&Finding>)>` with linear-scan `PartialEq` matching. Any deviation
from this causes either a compile error (HashMap/IndexMap without Hash) or a non-deterministic
output order (HashMap without Hash-derive). See BC-2.11.025 invariant 7.

## Architecture Compliance Rules

Derived from ADR-0003 (Display-Layer Aggregation), BC-2.11.025 invariants 4-7, and
BC-2.11.029 invariants 1-3:

1. **Collapse is private to TerminalReporter** — the collapse pass method is private (`fn collapse_findings_pass`).
   No other reporter (JsonReporter, CsvReporter) calls it or is aware of it.
2. **Reporter trait immutable slice** — `Reporter::render` accepts `findings: &[Finding]`
   (immutable reference). The trait contract prohibits mutation. The collapse pass creates
   an ephemeral `Vec<(CollapseKey, Vec<&Finding>)>` that borrows from the original slice;
   the original slice is never modified.
3. **No upstream pre-filtering** — `main.rs` passes the same unmodified `findings` slice to
   all reporters. The collapse result is ephemeral, used only within `TerminalReporter::render`.
4. **Vec accumulator is canonical** — `Vec<(CollapseKey, Vec<&Finding>)>` with linear-scan
   `PartialEq` matching is the only permitted implementation. HashMap is prohibited. IndexMap
   is only viable if `Hash` is derived on all three key enums AND `indexmap` is added to
   `Cargo.toml` — neither is done in v0.8.0.
5. **Suffix is appended BEFORE colorization** — the ` (xN)` suffix must be part of the string
   passed to the color function. Appending the suffix after the ANSI reset is NON-CONFORMANT
   (BC-2.11.026 postcondition 6 / invariant 4).
6. **grouped/--mitre path is structurally suffix-free** — the `render_findings_grouped` path
   and `render_finding_prefix` are never modified by STORY-118. The collapse pass lives
   entirely in the flat branch of the `if self.show_mitre_grouping` dispatch.
7. **K=3 is a named compile-time constant** — `const COLLAPSE_EVIDENCE_SAMPLES: usize = 3`.
   Not a magic number inline in the loop.
8. **escape_for_terminal is a function-level guarantee** — the collapse path calls
   `escape_for_terminal` directly on each sampled evidence line. It does NOT delegate to
   `render_finding_prefix`'s evidence loop (which handles all evidence for one finding;
   the collapse path handles evidence[0] from each of the first K members across multiple
   findings).

## Forbidden Dependencies

- `src/reporter/terminal.rs` MUST NOT import `HashMap` or `BTreeMap` for the collapse
  accumulator. If the build gains such an import for this purpose, it MUST fail the review.
- `src/reporter/json.rs` and `src/reporter/csv.rs` MUST NOT be modified by STORY-118.
  Any PR touching these files for collapse-related logic indicates a scope violation that
  breaks BC-2.11.029 invariant 1.
- `Cargo.toml` MUST NOT add `indexmap` as a dependency for v0.8.0 collapse (BC-2.11.025
  invariant 7). If added, the Vec-accumulator constraint is vacated but the Hash-derive
  obligation must then be met on `ThreatCategory`, `Verdict`, and `Confidence` — this
  combination is disallowed in v0.8.0 per the locked design.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `owo-colors` | existing (same as STORY-077/078) | Color-ladder: `.red().bold()`, `.yellow()`, `.cyan()`, `.dimmed()` |
| `chrono` | existing | `timestamp: Option<DateTime<Utc>>` in `Finding` — non-key field; not used in collapse |
| No new dependencies | — | Collapse uses only stdlib `Vec`, `PartialEq`, and the existing `escape_for_terminal` function |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reporter/terminal.rs` | **Modify** | Add `collapse_findings: bool` field to `TerminalReporter`; add `COLLAPSE_EVIDENCE_SAMPLES` const; add private `collapse_findings_pass` function; extend flat FINDINGS dispatch block (terminal.rs:149-162) |
| `src/cli.rs` | **Modify** | Add `#[arg(long)] no_collapse: bool` to `Commands::Analyze` (following cli.rs:150-152 mitre/dns precedent) |
| `src/main.rs` | **Modify** | Destructure `no_collapse` from `Commands::Analyze` in `main()` (main.rs:49-64); thread it as a new positional `bool` parameter into `run_analyze`; set `collapse_findings: !no_collapse` at `TerminalReporter { … }` construction (main.rs:370-376), mirroring how `*mitre` becomes `show_mitre_grouping` |
| `docs/adr/0003-reporting-pipeline-layering.md` | **Modify (must ride this PR)** | Add "Display-Layer Aggregation" section documenting the collapse feature design decision |
| `tests/reporter_terminal_tests.rs` | **Modify** | Add `mod story_118` block with all test functions named in ACs above (27 ACs post-Fix; see BC cross-check section for full test inventory) |
| `src/reporter/json.rs` | **No change** | JsonReporter is unaffected |
| `src/reporter/csv.rs` | **No change** | CsvReporter is unaffected |
| `src/findings.rs` | **No change** | Finding struct is unaffected |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~9,000 |
| BC files (9 BCs — 025/026/027/028/029/010/013/017/019) | ~24,000 |
| `src/reporter/terminal.rs` (full file, ~340 lines) | ~7,000 |
| `src/cli.rs` (Commands::Analyze block, ~30 lines) | ~1,000 |
| `src/main.rs` (run_analyze function, ~50 lines) | ~1,500 |
| `src/findings.rs` (Finding struct, reference) | ~1,000 |
| `tests/reporter_terminal_tests.rs` (existing mod story_077/078 tests) | ~8,000 |
| F1 delta analysis (issue-259-finding-collapse-delta-analysis.md, relevant sections) | ~3,000 |
| Tool outputs (cargo test, clippy) | ~1,500 |
| **Total estimated** | **~56,000** |

Within 20-30% of agent context window (200k context → 20-30% = 40-60k). This story
is at the boundary. If the test-writer produces 26+ test stubs in a single pass, split
the test-writing burst from the implementation burst to keep each pass under the budget.

## BC Frontmatter ↔ Body Cross-Check (DF-SIBLING-SWEEP-001)

Every BC in the `behavioral_contracts:` frontmatter array is cited by at least one AC:
- BC-2.11.025: AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-024, AC-026
- BC-2.11.026: AC-008, AC-009, AC-010, AC-011, AC-012, AC-023
- BC-2.11.027: AC-013, AC-014, AC-015, AC-016, AC-017
- BC-2.11.028: AC-018, AC-019
- BC-2.11.029: AC-020, AC-021, AC-022, AC-027
- BC-2.11.010: AC-016, AC-028
- BC-2.11.013: AC-005
- BC-2.11.017: AC-023
- BC-2.11.019: AC-025

Every AC cites a BC trace clause. All 9 BCs appear in both frontmatter and body.

DF-AC-TEST-NAME-SYNC-001 per-BC own-prefix test inventory (post-Fix-4):
- BC-2.11.025: test_BC_2_11_025_* (9 tests)
- BC-2.11.026: test_BC_2_11_026_* (10 tests: suffix_colorized_inside_span_red_bold,
  color_ladder_inconclusive_cyan, color_ladder_likely_other_yellow,
  color_ladder_possible_yellow, color_ladder_unlikely_dimmed, singleton_no_suffix,
  count_suffix_for_n_ge_2, large_count_exact, suffix_format,
  mitre_line_from_representative_finding)
- BC-2.11.027: test_BC_2_11_027_* (5 tests)
- BC-2.11.028: test_BC_2_11_028_* (3 tests: no_collapse_flag_one_line_per_finding,
  default_vs_opt_out_output_difference, flag_wired_to_reporter_field)
- BC-2.11.029: test_BC_2_11_029_* (4 tests: json_receives_full_findings,
  csv_receives_full_findings, non_repeated_finding_no_suffix,
  no_collapse_flag_json_invariant)
- BC-2.11.010: test_BC_2_11_010_* (1 test: escape_in_collapse_path — AC-028; additionally
  AC-016 traces to BC-2.11.010 via test_BC_2_11_027_escape_preserved_in_sampled_evidence
  as a cross-trace; the own-prefix test is test_BC_2_11_010_escape_in_collapse_path)
- BC-2.11.013: test_BC_2_11_013_grouped_mode_suffix_free (AC-005)
- BC-2.11.017: test_BC_2_11_017_collapsed_mitre_line_from_representative (AC-023)
- BC-2.11.019: test_BC_2_11_019_section_order_unchanged_with_collapse (AC-025)

Verification_properties: VP-012 (escape_for_terminal correctness — extended by collapse
path, AC-016).
