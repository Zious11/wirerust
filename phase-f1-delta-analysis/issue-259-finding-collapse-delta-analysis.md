---
document_type: f1-delta-analysis
feature_issue: "#259"
feature_title: "Collapse repeated low-value findings into counted summary lines"
feature_mode_cycle: "v0.7.1 → v0.8.0"
date: 2026-06-17
producer: architect
validation_report: .factory/research/issue-259-finding-collapse-validation.md
validation_verdict: GENUINE/HIGH
status: draft
---

# F1 Delta Analysis — Issue #259: Finding Collapse / Counted Summary Lines

## 1. Impact Boundary

### Files Touched (IN SCOPE)

| File | Change Type | Exact Scope |
|------|-------------|-------------|
| `src/reporter/terminal.rs` | Extend | Add `collapse_findings` boolean field to `TerminalReporter` struct (line 63–75); add collapse pass in `render()` (line 149–160 dispatch block); add new private helper `collapse_findings_flat(findings: &[Finding]) -> Vec<CollapsedFinding>` or equivalent; add `CollapsedFinding` type (likely a local struct: `key`, `count: usize`, `representative: &Finding`, `evidence_samples: Vec<String>`). The escape contract (`escape_for_terminal`, VP-012) is unchanged — it is called identically on the collapsed representative's summary and evidence. |
| `src/main.rs` | Extend | Add `--no-collapse` (or chosen flag name) to the `TerminalReporter` construction site in `run_analyze` (line 370–375); thread the flag value. One extra field at the struct literal. The `run_summary` path (line 432–435) does not produce findings and is therefore unaffected. |
| `src/cli.rs` | Extend | Add `--no-collapse` (or chosen flag name) under `Commands::Analyze`. One new `#[arg(long)]` boolean field, following the `--mitre` pattern (line 151–153). |
| `tests/reporter_terminal_tests.rs` | Extend | Add a `collapse_reporter()` helper (mirroring `plain_reporter()` at line 68–74); add test cases for: count display, evidence-sample retention, non-collapsed findings unaffected, collapse disabled (opt-out), threshold boundary (if threshold semantics chosen), interaction with `--mitre` grouped mode. Existing tests do NOT change because `plain_reporter()` uses `collapse_findings: false`/default. |

### Files Explicitly NOT Touched

| File | Reason |
|------|--------|
| `src/findings.rs` | The `Finding` struct is the raw forensic record. No new fields, no new methods. The aggregation key is derived at display time; `Finding` remains raw-data-only per ADR-0003. |
| `src/analyzer/http.rs` | The empty-UA emitter (lines 359–371) stays exactly as-is. One finding per matching request is correct behavior — the flood problem is a _display_ problem, not an emission problem. |
| `src/analyzer/tls.rs`, `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`, `src/analyzer/arp.rs` | All analyzers untouched. They emit raw findings; the terminal reporter aggregates on render. |
| `src/reporter/json.rs` | Machine-readable output. MUST NOT be changed. JSON consumers (SIEM ingestion, CSV pipelines) receive the complete, unaggregated `Vec<Finding>`. ADR-0003 layering principle enforces this. |
| `src/reporter/csv.rs` | Same as JSON — unaffected. CSV reporter iterates every finding individually (BC-2.11.020–024). |
| `src/reporter/mod.rs` | The `Reporter` trait signature (`render(summary, findings, analyzer_summaries)`) is unchanged. The collapse logic is a private detail inside `TerminalReporter::render`. |
| `src/dispatcher.rs`, `src/reassembly/`, `src/mitre.rs`, `src/decoder.rs` | Upstream pipeline stages — untouched. |

### Confirmed Reporter-Only Aggregation Boundary

The aggregation boundary is strictly enforced: `TerminalReporter::render` receives `findings: &[Finding]` (the raw, unaggregated slice). It internally applies the collapse pass before rendering. The caller (`main.rs`) passes the same findings slice to all reporters — the slice is never modified. JSON and CSV reporters downstream receive the original slice unmodified. This is the Wireshark Expert Info pattern: raw frames intact, Expert Information is a display-layer lens only.

---

## 2. Affected Specs and BCs

### Existing BCs Touched or Extended

| BC ID | Title | Nature of Change |
|-------|-------|-----------------|
| BC-2.11.019 | TerminalReporter Renders Sections in Correct Order | New postcondition: when `collapse_findings = true`, the FINDINGS section body renders collapsed groups (one line per unique key) instead of one line per raw finding. Section presence/ordering (postconditions 1–6) is unchanged. |
| BC-2.11.013 | MITRE Grouping Emits Tactic Headers in Canonical Order | Extension: when both `show_mitre_grouping = true` and `collapse_findings = true`, collapsed groups should be applied within each tactic bucket, or collapse should be disabled in grouped mode (design choice — see §9, OQ-3). |
| BC-2.11.017 | render_finding_flat Multi-ID MITRE rendering | Extension: the collapsed variant of `render_finding_flat` needs a count suffix `(xN)` at the end of the header line. The existing flat invariant is unchanged for `collapse_findings = false`. |
| BC-2.11.010 | render_finding_prefix summary + evidence rendering | Extension: for a collapsed group, the representative evidence shown is a _sample_ (not all N items). New invariant: at most K evidence lines are shown (K to be decided; see §9 OQ-4). |

### New BCs Required

Estimate: **5 new BCs** under SS-11 (BC-2.11.025 through BC-2.11.029 approximately), covering:

| Proposed ID | Scope |
|-------------|-------|
| BC-2.11.025 | Aggregation key definition: two findings with identical `(category, verdict, confidence, summary)` are the same group; any difference in any of these four fields produces a distinct group. `evidence` content, `mitre_techniques`, `source_ip`, `timestamp`, and `direction` are NOT part of the key (they vary per instance and are retained as samples). |
| BC-2.11.026 | Count display format: a collapsed group of N identical findings renders the header line as `[category] verdict (confidence) - summary (xN)`. Singleton groups (N=1) are rendered without the count suffix to avoid noise. This is the occurrence-count annotation pattern per Wireshark Expert Info. |
| BC-2.11.027 | Evidence sampling: a collapsed group retains at most K evidence lines from the first K findings in the group (in original emission order). K is a named constant (default 3, pending §9 OQ-4 adjudication). Evidence lines beyond K are discarded for terminal display only; they remain in the JSON/CSV raw stream. |
| BC-2.11.028 | Opt-out flag (`--no-collapse`): when this flag is present, the FINDINGS section reverts to one line per raw finding (no count suffix, all evidence shown), identical to the current behavior. When absent, collapse is default-on. |
| BC-2.11.029 | Threshold semantics (if threshold chosen over always-collapse): collapse only applies when a group has count >= T (default T=1 means always; T=2 collapses only genuinely repeated findings). If threshold approach is adopted, the threshold value is settable via `--collapse-threshold N`. This BC may be deferred or combined with BC-2.11.028 depending on §9 OQ-2 adjudication. |

**Total BC impact: 4 extended + 5 new = 9 BC changes in SS-11.** The Finding struct (SS-09) BCs are unchanged. Analyzer BCs (SS-06) are unchanged.

---

## 3. ADR Decision

### Recommendation: Extend ADR-0003 with a Section on Display-Layer Aggregation

**Do NOT create a new ADR.** ADR-0003 already establishes the governing principle: "The data layer holds raw bytes; the display layer formats for its medium." Aggregation/dedup is a natural extension of this principle — it is another display-layer transform that does not mutate the canonical finding stream. A new ADR would be redundant and would split the conceptual narrative.

Instead, append a subsection to ADR-0003 (`docs/adr/0003-reporting-pipeline-layering.md`) titled "Display-Layer Aggregation (issue #259)." This subsection captures:

1. The precedent (Wireshark Expert Info; ntopng; Splunk dedup — already validated in `.factory/research/issue-259-finding-collapse-validation.md`).
2. The key design choices listed below and their resolutions.
3. The binding rule for future reporter authors: "Aggregation of repeated findings for human readability belongs in the terminal reporter, not in the finding stream. JSON and CSV consumers MUST receive the complete, unaggregated slice."

### Key Design Choices Needing Adjudication (for ADR appendix)

These are not all resolvable by the architect alone — they are enumerated here and raised as open questions in §9.

**A. Aggregation Key**

The natural key is `(category, verdict, confidence, summary)`. Including `evidence` in the key would prevent collapsing (every request has a different URI in its evidence), which defeats the purpose. Excluding `source_ip` and `timestamp` means findings from different sources collapse — appropriate for a global "how many times did I see X" view, but it loses per-source granularity in the collapsed display. Wireshark keys on (severity, message text), which is the direct analogue of `(verdict, summary)` — the narrowest key. The broader key `(category, verdict, confidence, summary)` seems correct for wirerust because `Anomaly/INCONCLUSIVE/LOW` vs `Reconnaissance/INCONCLUSIVE/LOW` on the same summary text are semantically different finding types.

**B. Collapse Eligibility Scope**

Option 1 (universal): Collapse all repeated findings regardless of verdict/confidence. This is the Wireshark approach — every repeated anomaly is collapsed, even HIGH severity ones, because the count is always informative.

Option 2 (selective): Collapse only findings matching `Verdict::Inconclusive | Verdict::Unlikely` or `Confidence::Low`. This avoids collapsing `LIKELY HIGH` findings which a defender may want to see individually for triage. The risk is the boundary being surprising — why does `INCONCLUSIVE MEDIUM` collapse but not `POSSIBLE MEDIUM`?

Opinion: universal collapse (option 1) with the count always visible is safer and more consistent. The count is informative even for HIGH-confidence findings. The opt-out flag covers analysts who need the full stream.

**C. Threshold**

Option 1 (always collapse, N=1 minimum): Every finding that repeats collapses. Singleton groups show no count suffix. This is the simplest contract.

Option 2 (threshold T): Collapse only when count >= T. At T=2, singletons are never shown with a count — they render as today. This is effectively option 1 because T=1 and T=2 differ only in whether singletons get a `(x1)` suffix (which option 1 also suppresses for singletons).

Opinion: always-collapse (option 1) is simpler. A threshold flag adds surface area without meaningful benefit if singleton groups already render without a count suffix.

**D. Default On vs Opt-In**

Option 1 (default-on, `--no-collapse` opt-out): Breaking change for any script that currently greps for exact line counts in terminal output. However, the validation report confirms the flood scenario is the real production pain. Default-on maximizes benefit to new users and matches the Wireshark model where Expert Info is always shown.

Option 2 (default-off, `--collapse` opt-in): Preserves existing behavior for existing users. Requires explicit adoption. Lower risk for issue #63 (golden test) because existing tests pass without change.

This is the most consequential choice and is explicitly flagged in §9 as OQ-1.

---

## 4. CLI Surface Delta

### Proposed Change

Add one flag to `Commands::Analyze` in `src/cli.rs`:

```
/// Disable collapsing of repeated identical findings into a counted
/// summary line. By default, the terminal reporter collapses groups of
/// identical (category/verdict/confidence/summary) findings into a single
/// line annotated with an occurrence count (e.g. "(x3142)"). Pass this
/// flag to see every finding individually, equivalent to the behavior
/// before v0.8.0. Has no effect on JSON or CSV output, which always
/// emit every finding individually.
#[arg(long)]
no_collapse: bool,
```

OR (if default-off is chosen):

```
/// Collapse repeated identical findings into a single counted summary
/// line (e.g. "[Anomaly] INCONCLUSIVE (LOW) - Empty User-Agent header
/// (x3142)"). Applies to terminal output only; JSON and CSV always emit
/// every finding.
#[arg(long)]
collapse: bool,
```

**Flag name consistency with existing conventions:**
- Existing negation flags: `--no-color`, `--no-reassemble`. The `--no-collapse` form is consistent with these.
- Existing positive flags: `--mitre`, `--hosts`, `--http`, `--all`. The `--collapse` form follows the `--mitre` pattern (off by default, enables a display mode).
- No flag named `--verbose` (issue #62 noted it was a previously removed unwired flag — must not be reused to avoid confusion with the BC-2.13.001 absent-behavior contract which rejects `--verbose` as an error).
- No flag named `--expand` (not in any existing convention; less discoverable).

The flag is `Commands::Analyze`-scoped only (no effect in `summary` subcommand which has no findings section). It must be wired to `TerminalReporter::collapse_findings` (or equivalent field name) in `main.rs` `run_analyze` at lines 370–375, consistent with LESSON-P1.04 (no unwired flags).

**Issue #62 interaction:** The issue #62 trigger condition is "when a 3rd render flag is added." This feature adds the third boolean field to `TerminalReporter` (`use_color`, `show_mitre_grouping`, `show_hosts_breakdown` — already three — plus the collapse field makes four). The issue states "refactoring becomes acute around 3-4 parameters." This F1 should recommend that the F2 spec explicitly acknowledges issue #62 and either defers the enum-of-modes refactor again (with documented rationale) or includes it as part of this story. It should not be an untracked surprise.

---

## 5. Regression Risk

### JSON/CSV Must Remain Un-Aggregated

CRITICAL. `Reporter::render` receives the `findings: &[Finding]` slice. The collapse pass is strictly internal to `TerminalReporter::render`. The JSON reporter at `src/reporter/json.rs` and CSV reporter at `src/reporter/csv.rs` receive the same original slice. No code path should ever pass a pre-collapsed slice to a reporter. The implementation must not move the aggregation step upstream of the multi-reporter dispatch in `main.rs`.

Test obligation: add an integration test that feeds N identical findings, renders to both terminal and JSON, asserts terminal output has 1 collapsed line with `(xN)`, and asserts JSON output has exactly N finding objects.

### Non-Repeated Findings Remain Individually Visible

Any finding that appears exactly once must NOT be affected by the collapse path. The count suffix must not appear on singleton groups. The existing rendering for unique findings must be byte-identical to the current output.

Test obligation: mixed input with 1 unique + N identical; assert unique finding renders as current, identical ones collapse.

### Grouped-Mode Interaction (--mitre + collapse)

When `show_mitre_grouping = true`, the findings are already organized into tactic buckets. Applying collapse within each bucket is feasible (each `render_finding_grouped` call would become collapse-aware). However, the interaction is non-trivial: the current `render_findings_grouped` passes individual `&Finding` references to `render_finding_grouped` after sorting within each bucket (lines 303–315). A collapse pass within grouped mode would need to happen after bucketing and sorting but before rendering each bucket.

Decision options: (a) collapse within each tactic bucket independently, (b) disable collapse when `--mitre` is active (mutually exclusive), (c) implement for flat mode only in this cycle and defer grouped mode. This is explicitly flagged as OQ-3 in §9.

### Determinism / Ordering of Collapsed Output

The collapsed output must be deterministic. The order of collapsed groups must be stable given the same input slice. The simplest approach: preserve the order of first occurrence — the group's slot in the output is determined by the first finding matching that key (same logic as the current flat rendering, which uses slice order). Groups are not re-sorted after collapsing.

The existing `BC-2.11.019` invariant 6 (PROTOCOLS/SERVICES deterministic sort) does not apply to findings because findings are not sorted by the terminal reporter today (they render in emission order). This remains true after collapse.

### Existing Terminal Snapshot / Golden Test Risk

Issue #63 is open: no golden/snapshot test exists for `TerminalReporter` output today. Existing tests are substring-based. This means:
- Existing tests will NOT break if collapse is default-off (new flag is absent from `plain_reporter()`).
- Existing tests WILL need updating if collapse is default-on, because the output format changes for inputs with repeated findings. The reporter tests at `tests/reporter_terminal_tests.rs` (1,746 lines) use a `plain_reporter()` helper (line 68–73) that constructs `TerminalReporter` with specific field values — adding a new field with a default value follows the same pattern as `show_hosts_breakdown`.

If default-on is chosen: all existing tests that use `plain_reporter()` will now construct a reporter with collapse enabled. If any existing test input contains repeated findings, those tests will fail. A quick audit of test inputs is needed during F2/F3.

### Interaction with Issue #255 (JSON Casing)

Issue #255 proposes adding `#[serde(rename_all = "snake_case")]` to `Confidence`, `Verdict`, and `ThreatCategory`. This is a breaking JSON schema change. It does not interact with the collapse feature (which is terminal-only), but if both features are merged in the same v0.8.0 release, the combined JSON schema change must be documented in the release notes. No implementation conflict exists — the features touch different files. However, sequencing stories matters: if #255 lands first and breaks existing JSON tests, it should be resolved before the collapse stories add more JSON-surface tests.

### Interaction with Issue #62 (TerminalReporter Refactor)

At three boolean fields today (`use_color`, `show_mitre_grouping`, `show_hosts_breakdown`), adding a fourth (`collapse_findings`) crosses the threshold issue #62 identified. If the struct is refactored to an enum-of-modes or a `TerminalReporterConfig` sub-struct as part of this feature, all construction sites must be updated: `main.rs` lines 370–375 and 432–435, `tests/reporter_terminal_tests.rs` `plain_reporter()` at line 68–73 and `mitre_reporter()` at line 656–661. The refactor is mechanical but has a broad touch surface.

---

## 6. Affected Tests

### Existing Tests That Will Change

| Condition | Effect |
|-----------|--------|
| Default-on chosen | `plain_reporter()` helper must set `collapse_findings: true`; any test with repeated identical findings in its input will produce different output. Audit of test inputs needed in F2 before authoring. |
| Default-off chosen | `plain_reporter()` sets `collapse_findings: false`; all existing tests pass without change. Only new tests are added. |
| Issue #62 refactor included | Every `TerminalReporter { ... }` literal in `tests/reporter_terminal_tests.rs` (2 sites: `plain_reporter` line 68, `mitre_reporter` line 656) and `src/main.rs` (2 sites: lines 370, 432) requires updating. |

### New Test Classes Required

| Test Class | What It Verifies | BC Coverage |
|------------|-----------------|-------------|
| Identical-findings collapse | N identical `(category, verdict, confidence, summary)` findings → 1 collapsed line with `(xN)` suffix | BC-2.11.025, BC-2.11.026 |
| Singleton no-count | A group of 1 renders without count suffix | BC-2.11.026 |
| Evidence sampling | Group of N (>K) → at most K evidence lines in output | BC-2.11.027 |
| Mixed input | 1 unique finding + N identical → unique renders individually, identical collapses | BC-2.11.025, BC-2.11.026 |
| Opt-out (--no-collapse) | Same N identical input, `collapse_findings: false` → N lines, no count suffix | BC-2.11.028 |
| JSON output unchanged | N identical findings through JsonReporter → N finding objects in output | ADR-0003 + BC-2.11.001–005 (unchanged) |
| Grouped mode interaction | Behavior when `show_mitre_grouping: true` and `collapse_findings: true` | BC-2.11.013 extension |
| Key discriminator | Two findings differing only in `evidence` but same key → collapse | BC-2.11.025 |
| Key discriminator | Two findings differing in `category` → do NOT collapse | BC-2.11.025 |
| Terminal safety preserved | Collapsed summary still runs through `escape_for_terminal` | VP-012 (unchanged, but must be exercised through collapse path) |

### Holdout Scenario Implications

For Phase F4 holdout evaluation, the primary scenario to construct is: a synthetic pcap (or mock finding slice) producing 1000+ identical `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` findings. Holdout verifies: terminal output shows exactly 1 collapsed line with `(x1000)` (or whatever count); JSON output contains exactly 1000 finding objects; `--no-collapse` terminal output reverts to 1000 lines.

---

## 7. Epic Placement

### Recommendation: Extend Epic E-8 (Reporting and Output Formats)

Epic E-8 already owns SS-11 (reporter) BCs and the `--mitre` grouped-mode feature (STORY-078). Finding collapse is a display-layer augmentation of the same `TerminalReporter`. It does NOT warrant a new epic — it is a natural extension of E-8's stated goal: "A SOC operator can select ... terminal (default) output."

Do NOT place this in E-4 (HTTP Analysis) even though the canonical example is the HTTP empty-UA finding. The collapse feature is generic — it will benefit any high-frequency finding from any analyzer (TLS, ARP, Modbus). Coupling it to E-4 would scope it incorrectly.

### Estimated Story Count: 2 Stories

| Story | Scope | Points (est.) |
|-------|-------|----------------|
| STORY-118 | Terminal reporter collapse: `CollapsedFinding` type, collapse pass in `render()`, flat-mode rendering with `(xN)` count, evidence sampling (at most K lines), `--no-collapse` opt-out flag wired in CLI + `main.rs`. New BCs BC-2.11.025–028 authored in F2. TDD mode: standard (new production code). | 8 pts |
| STORY-119 | Grouped-mode interaction (if not deferred): extend `render_findings_grouped` to apply collapse within each tactic bucket. BC-2.11.029 or BC-2.11.013 extension. Alternatively, this story is deferred to a follow-on cycle if grouped-mode collapse is out of scope for v0.8.0. | 3–5 pts |

**Total: 2 stories (11–13 pts), release target v0.8.0.** STORY-118 is the primary deliverable and can ship independently. STORY-119 is the grouped-mode extension and may be deferred.

Next available story IDs: STORY-118, STORY-119 (STORY-117 is the current highest).

---

## 8. Verification-Property Impact

### No New Formal VP Required

The collapse feature does not introduce a property that requires formal verification (Kani / proptest / cargo-fuzz):

- **Correctness of count:** The count is simply `Vec.len()` of grouped findings. A Kani proof is overkill; a unit test covering the collapse-key logic is sufficient.
- **No-loss invariant (JSON/CSV unchanged):** This is a process invariant — the collapse pass is never applied to the JSON/CSV render path. Enforced by code structure (collapse is a private method of `TerminalReporter` not reachable from `JsonReporter`/`CsvReporter`) and by the integration test mandated in §6.
- **Terminal safety (escape_for_terminal still called):** VP-012 already covers `escape_for_terminal` correctness via proptest (4 properties, 1000 cases each). The collapse path calls `escape_for_terminal` through the same `render_finding_prefix` invocation as today. VP-012 is unchanged. The integration test for collapse should use a summary string containing a control byte to confirm the escape path is not bypassed.

### Classification: Test-Sufficient

The collapse feature is **test-sufficient** (unit + integration tests). It does not meet the bar for a new formal VP:
- No complex arithmetic that could overflow.
- No state-machine invariant.
- No security boundary.
- No cross-flow or cross-process interaction.

The existing VP-012 (`escape_for_terminal Correctness`, proptest, P1) is the only VP that touches `terminal.rs`. It is unaffected by this feature.

---

## 9. Open Questions for the F1 Human Gate

These design choices require adjudication before F2 (spec evolution) can begin. They are enumerated in priority order.

### OQ-1 (BLOCKING): Default-On vs Opt-In

**Question:** Should collapse be the default behavior (`--no-collapse` to opt out) or an explicit opt-in (`--collapse` to enable)?

**Trade-offs:**
- Default-on: maximizes benefit for new users; matches Wireshark's model; breaking change for terminal output consumers who script against line counts (rare but possible); requires existing tests to explicitly set the field.
- Opt-in: zero regression risk; no existing test changes; lower adoption friction for current users; the flooding problem persists until users discover the flag.

**Recommendation from F1 analysis:** Default-on. The flooding scenario is the primary motivation. The change affects only terminal output; JSON/CSV consumers are unaffected. Terminal output is not a machine-readable contract (that is what JSON/CSV are for). However, this is the single most consequential UX choice and should be explicitly confirmed by the human gate.

### OQ-2 (IMPORTANT): Threshold vs Always-Collapse

**Question:** Should collapse always apply (singleton groups render without count suffix), or only when count >= T (with a `--collapse-threshold N` CLI parameter)?

**Recommendation from F1 analysis:** Always-collapse (no threshold flag). The count is always informative. Hiding singletons behind a threshold adds configuration surface without clear benefit. Singletons already look identical to the current output (no count suffix). However, if OQ-1 resolves as default-on, the always-collapse behavior may cause more test updates — the human gate should confirm both OQ-1 and OQ-2 together.

### OQ-3 (IMPORTANT): Grouped-Mode (--mitre) Interaction

**Question:** When both `--mitre` and collapse are active, does collapse apply within each tactic bucket? Or are the modes mutually exclusive?

**Options:**
- A: Collapse within each tactic bucket independently (most user-friendly, most implementation effort).
- B: Mutually exclusive — `--mitre` and collapse cannot both be active; `--mitre` implies `--no-collapse`. Simplest to implement and spec.
- C: Flat mode only — collapse only applies when `show_mitre_grouping = false`. Same as B but expressed as feature scope rather than a conflict.

**Recommendation from F1 analysis:** Option C (flat mode only) for v0.8.0, with a deferred STORY-119 for grouped-mode support in a follow-on cycle. This keeps STORY-118 focused and avoids the complexity of collapse within the already-complex tactic-bucket sort logic (BC-2.11.014).

### OQ-4 (REFINEMENT): Evidence Sample Count K

**Question:** How many evidence lines should a collapsed group retain?

Default proposal: K=3 (retain the first 3 evidence lines from the group, one per finding). This shows enough context to confirm the finding type without flooding the output with evidence from all N occurrences. K=1 is the minimum useful (the representative finding only); K=5 is the maximum that feels manageable on screen.

**This is a refinement choice, not a blocker.** K=3 is a reasonable default; it can be made configurable via `--collapse-evidence-samples N` if desired, or hardcoded as a named constant.

---

## Summary

| Dimension | Assessment |
|-----------|-----------|
| Impact boundary | `terminal.rs` (display layer only), `cli.rs` (one flag), `main.rs` (one construction-site field). JSON/CSV reporters untouched. `findings.rs` and all analyzers untouched. |
| New BCs | 5 new (BC-2.11.025–029), 4 extended (BC-2.11.010, .013, .017, .019). Total: 9 BC changes in SS-11. |
| ADR recommendation | Extend ADR-0003 with a display-layer aggregation subsection. No new ADR. |
| Story estimate | 2 stories (STORY-118 primary, STORY-119 grouped-mode extension or defer). v0.8.0 target. |
| Epic placement | Extend Epic E-8 (Reporting and Output Formats). |
| New VP | None. Test-sufficient. VP-012 (`escape_for_terminal`) unchanged. |
| Top regression risk | (1) JSON/CSV must remain unaggregated — enforce by code structure + integration test; (2) `--no-collapse` opt-out must produce byte-identical output to current behavior; (3) issue #62 fourth-bool threshold — acknowledge or address in F2. |
| Open design questions | OQ-1 (default-on vs opt-in — BLOCKING); OQ-2 (threshold — IMPORTANT); OQ-3 (grouped-mode interaction — IMPORTANT); OQ-4 (evidence sample count K — refinement). |

---

## Related References

- Validation report: `/Users/zious/Documents/GITHUB/wirerust/.factory/research/issue-259-finding-collapse-validation.md`
- ADR to extend: `docs/adr/0003-reporting-pipeline-layering.md`
- Primary source location: `src/reporter/terminal.rs:149–160` (FINDINGS dispatch block), `src/reporter/terminal.rs:203–238` (render_finding_prefix + render_finding_flat)
- Canonical empty-UA emitter: `src/analyzer/http.rs:359–371`
- CLI construction convention: `src/cli.rs:151–153` (--mitre flag as pattern to follow)
- `TerminalReporter` construction sites: `src/main.rs:370–375` (analyze), `src/main.rs:432–435` (summary — unaffected)
- Reporter test helper: `tests/reporter_terminal_tests.rs:68–73` (plain_reporter)
- Dependent open issues: #62 (enum-of-modes refactor threshold), #63 (golden/snapshot tests), #255 (JSON casing — orthogonal)
- Next story IDs: STORY-118 (primary), STORY-119 (grouped-mode)
- Epic: E-8, SS-11, BC-2.11.XXX series
