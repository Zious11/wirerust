---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified: ["v1.1 2026-06-17: fix Related BCs stale cross-ref BC-2.13.001 (--threats) → BC-2.13.004 (--verbose absent) (consistency audit remediation)", "v1.2 2026-06-17: F2 adversarial pass-1 — change PC-3 from indicative to imperative (code does not exist yet); mark Architecture Anchors as insertion targets pending STORY-118 (F-259-08)", "v1.3 2026-06-17: F2 adversarial pass-9 — F-PA-03: add EC-010 (--no-collapse absent, default --output terminal → collapse applies, default-on)"]
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
the default-on collapse behavior introduced in v0.8.0. When `--no-collapse` is present, the
terminal reporter renders one display line per raw `Finding` in the input slice — identical to
the pre-v0.8.0 behavior. When `--no-collapse` is absent (the default), the collapse pass is
enabled. The flag is wired from `cli.rs` through `main.rs` to the `TerminalReporter`
`collapse_findings` field (set to `false` when `--no-collapse` is present, `true` when absent).

`--no-collapse` has no effect on JSON output (`--output json`) or CSV output (`--output csv`).
Both machine-readable formats always emit every finding individually regardless of the flag,
because the collapse pass is a private detail of `TerminalReporter` and is never applied to
`JsonReporter` or `CsvReporter`.

The flag follows the existing `--no-color`, `--no-reassemble` negation convention in the
wirerust CLI. It is scoped to the `analyze` subcommand only; it has no effect on the
`summary` subcommand (which has no findings section).

## Preconditions

1. The user invokes `wirerust analyze <pcap> [--no-collapse]` (or omits the flag for default
   behavior).
2. The flag is defined as `#[arg(long)]` `no_collapse: bool` on `Commands::Analyze` in
   `src/cli.rs`, following the `--no-color` / `--no-reassemble` pattern.
3. The `no_collapse` field MUST be wired in `src/main.rs` `run_analyze` by STORY-118:
   `collapse_findings: !args.no_collapse` at the `TerminalReporter` construction site
   (insertion target: ~main.rs `run_analyze` function; the `TerminalReporter` struct and the
   `collapse_findings` field do not exist yet in the codebase as of this spec — they are
   created by STORY-118). Per LESSON-P1.04, an unwired flag is a spec violation.

## Postconditions

1. When `--no-collapse` is absent (default): `TerminalReporter.collapse_findings = true`.
   The FINDINGS section renders collapsed groups with ` (xN)` suffixes per BC-2.11.025 and
   BC-2.11.026.
2. When `--no-collapse` is present: `TerminalReporter.collapse_findings = false`. The FINDINGS
   section renders one display line per raw `Finding` in the input slice. No ` (xN)` count
   suffix appears on any line. Every finding's full evidence is rendered (no evidence sampling
   per BC-2.11.027). The output is byte-identical to the pre-v0.8.0 terminal output for the
   same input.
3. In both modes, `JsonReporter` and `CsvReporter` receive the complete, unmodified
   `findings: &[Finding]` slice and render every finding individually. The `--no-collapse`
   flag has no observable effect on JSON or CSV output.
4. The flag does not interact with `--mitre`. When `--no-collapse` is present, grouped mode
   is unaffected (it already renders individually). When `--no-collapse` is absent and
   `--mitre` is active, collapse is suppressed by the `show_mitre_grouping = true` guard
   (BC-2.11.025 invariant 5).

## Invariants

1. The `no_collapse` field on `Commands::Analyze` is a boolean. It is `true` when the flag
   is present, `false` when absent. The `TerminalReporter.collapse_findings` field is
   `!args.no_collapse`.
2. Default behavior (flag absent) is collapse-ON. This is intentional: the canonical v0.8.0
   UX for terminal output is the collapsed view. Analysts who require per-finding lines for
   scripting or detailed triage can explicitly opt out.
3. The flag is a pure boolean. It does not accept a value (not `--no-collapse=true`). There
   is no `--collapse` alias in v0.8.0.
4. The flag is `analyze`-subcommand-scoped. The `summary` subcommand has no `no_collapse`
   field because it emits no findings section.
5. The flag must be documented in the `--help` output for `analyze`. The help text must
   describe both the default-on behavior and the opt-out semantics, and must clarify that
   JSON/CSV output is unaffected.
6. Per LESSON-P1.04 (no unwired flags): the `no_collapse` field in `cli.rs` MUST be wired
   to `TerminalReporter.collapse_findings` in `main.rs`. An unwired flag is a spec violation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --no-collapse absent, 5 identical findings | Terminal: collapsed, `(x5)` suffix; JSON: 5 finding objects |
| EC-002 | --no-collapse present, 5 identical findings | Terminal: 5 individual lines, no suffix; JSON: 5 finding objects |
| EC-003 | --no-collapse present, all findings are unique | Terminal output byte-identical to pre-v0.8.0 (no collapse was happening anyway) |
| EC-004 | --no-collapse present with --mitre | Grouped mode renders individually; --no-collapse has no additional effect (already no collapse in grouped mode) |
| EC-005 | --no-collapse absent with --mitre | Grouped mode renders individually (collapse suppressed by show_mitre_grouping guard per BC-2.11.025 invariant 5) |
| EC-006 | --no-collapse with --output json | JSON output is identical to --no-collapse absent with --output json; the flag has no effect on JsonReporter |
| EC-007 | --no-collapse with --output csv | CSV output is identical to --no-collapse absent with --output csv; the flag has no effect on CsvReporter |
| EC-008 | summary subcommand invoked (no --no-collapse field) | No error; summary subcommand has no findings section and no no_collapse field; unaffected |
| EC-009 | --no-collapse present, no findings in pcap | Empty FINDINGS section (or absent section) as usual; no error |
| EC-010 | --no-collapse absent, default --output (terminal) | Collapse applies (default-on); TerminalReporter.collapse_findings=true; collapsed groups with (xN) suffixes rendered per BC-2.11.025/026 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 identical `(Anomaly, Inconclusive, Low, "Empty UA")` findings, collapse_findings=false (--no-collapse) | FINDINGS section has 5 individual lines, no `(x5)` suffix anywhere | happy-path (opt-out) |
| 5 identical findings, collapse_findings=true (default) | FINDINGS section has 1 collapsed line with `(x5)` suffix | happy-path (default) |
| 5 identical findings rendered to JSON reporter (collapse_findings=false or true) | JSON output contains 5 finding objects regardless | happy-path (JSON unaffected) |
| --no-collapse present, mix of 1 unique + 3 identical findings | 4 individual lines, no suffixes | mixed scenario (opt-out) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | collapse_findings=false produces one line per finding, no (xN) suffix | unit: test_BC_2_11_028_no_collapse_flag_one_line_per_finding |
| — | collapse_findings=true (default) vs false: output differs for repeated findings | unit: test_BC_2_11_028_default_vs_opt_out_output_difference |
| — | JSON reporter output identical regardless of collapse_findings | integration: test_BC_2_11_029_json_receives_full_findings (cross-BC) |
| — | Flag is wired (no_collapse=true → collapse_findings=false in TerminalReporter) | unit: test_BC_2_11_028_flag_wired_to_reporter_field |

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

- `src/cli.rs:151-153` -- `#[arg(long)] no_reassemble: bool` pattern to follow for `no_collapse` (existing code; reference only)
- `src/main.rs:~run_analyze` -- **INSERTION TARGET (code TBD by STORY-118):** `collapse_findings: !args.no_collapse` at TerminalReporter construction. The `collapse_findings` field does NOT exist on TerminalReporter yet; line numbers will be determined when STORY-118 adds the field. The ~370-375 range is a pre-story approximation.
- `src/reporter/terminal.rs:63-75` -- **INSERTION TARGET (code TBD by STORY-118):** `collapse_findings: bool` field to be added to TerminalReporter struct. Currently the struct has only `use_color`, `show_mitre_grouping`, `show_hosts_breakdown` — `collapse_findings` is not yet present.

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
