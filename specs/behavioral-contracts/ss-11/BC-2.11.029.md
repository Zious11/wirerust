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
modified: ["v1.1 2026-06-17: fix Postcondition 3 — remove misleading 'N=1 ≤ K=3' reasoning; singleton renders identically to pre-v0.8.0 (consistency audit remediation)", "v1.2 2026-06-17: F2 adversarial pass-1 — add precise csv.rs line anchors (csv.rs:40 neutralize, csv.rs:76 render loop); mark terminal.rs:63-75 as insertion target (F-259-05, F-259-08)", "v1.3 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Precondition 4: 'collapse_findings = true' → 'render = FindingsRender::FlatCollapsed'; PC-1 inline qualifier: 'collapse_findings' → 'render' field; Architecture Anchors: INSERTION TARGET wording updated from old bool field names to FindingsRender enum. Rationale: illegal-state elimination. No behavioral change."]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.029: Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs

## Description

The collapse feature (BC-2.11.025 through BC-2.11.028) is strictly a terminal display-layer
transform. The raw `findings: &[Finding]` slice passed to every reporter via
`Reporter::render(summary, findings, analyzer_summaries)` is identical for all reporters:
`TerminalReporter`, `JsonReporter`, and `CsvReporter` each receive the same, complete,
unmodified slice. The collapse pass is applied only inside `TerminalReporter::render` and has
no observable effect on the output of `JsonReporter` or `CsvReporter`. No code path upstream
of the multi-reporter dispatch in `main.rs` may apply the collapse pass, pre-filter the slice,
or deduplicate findings before passing them to reporters.

Additionally, any finding that appears exactly once in the input slice (a non-repeated finding)
must remain individually visible in the terminal output — it must never acquire a ` (x1)` count
suffix or be merged into another group. Non-repeated findings pass through the terminal reporter
unchanged relative to the pre-v0.8.0 rendering (singleton group behavior per BC-2.11.026
postcondition 2).

This BC is the enforcement contract for ADR-0003's "Display-Layer Aggregation" extension: raw
frames are intact in all machine-readable outputs; aggregation is a display-layer lens only.

## Preconditions

1. `Reporter::render` has been invoked on `TerminalReporter` with `findings: &[Finding]`.
2. `Reporter::render` has been invoked on `JsonReporter` with the same `findings` slice.
3. `Reporter::render` has been invoked on `CsvReporter` with the same `findings` slice.
4. The collapse feature is enabled (`TerminalReporter.render = FindingsRender::FlatCollapsed`).
5. The input `findings` slice contains at least one repeated finding (for the interesting case)
   and at least one non-repeated finding (for the non-interference case).

## Postconditions

1. `JsonReporter` produces exactly one JSON finding object per element of the input `findings`
   slice. If the slice contains N findings, the JSON array contains exactly N objects.
   No finding is omitted, merged, or deduplicated in JSON output regardless of
   `TerminalReporter.render` variant.
2. `CsvReporter` produces exactly one CSV row per element of the input `findings` slice.
   If the slice contains N findings, the CSV body (excluding the header row) contains exactly N
   rows. No row is omitted, merged, or deduplicated in CSV output.
3. A non-repeated finding (one that is the sole finding with its `(category, verdict,
   confidence, summary)` key) is rendered individually in terminal output. Its header line
   has no ` (xN)` suffix. Its terminal rendering is identical to the pre-v0.8.0 output for
   that finding — the collapse feature does not alter the rendering of non-repeated findings.
   Evidence is unaffected by the collapse feature: the evidence rendering path for singleton
   groups is identical to the pre-collapse `render_finding_prefix` call, which renders all
   evidence lines (governed by BC-2.11.010).
4. The `findings` slice is never mutated or pre-filtered upstream of the reporter dispatch
   in `main.rs`. The same slice reference is passed to all reporters.
5. The `--no-collapse` flag (BC-2.11.028) does not affect JSON or CSV output; both always
   produce N objects/rows regardless of the flag.

## Invariants

1. The collapse pass is a private method of `TerminalReporter` and is not reachable from
   `JsonReporter` or `CsvReporter`. Code structure enforces this boundary; there is no
   shared aggregation state between reporters.
2. The `Reporter` trait signature (`fn render(&self, summary: &Summary, findings: &[Finding],
   analyzer_summaries: &[AnalysisSummary]) -> String`) accepts `findings` as an immutable
   slice reference. The trait contract prohibits mutation.
3. The multi-reporter dispatch in `main.rs` passes the same `findings` slice to all enabled
   reporters in sequence. No intermediate transformation, cloning, or filtering of the slice
   occurs between reporter invocations.
4. A singleton group (N=1 in the collapse pass) and a non-repeated finding are the same
   concept: any finding unique in the input slice forms a singleton group, which renders
   without a count suffix.
5. The JSON finding count invariant (postcondition 1) must be verified by an integration test
   that feeds N identical findings, renders to both terminal and JSON, asserts terminal has 1
   collapsed group, and asserts JSON has N objects.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | N=5 identical findings, collapse=true, JSON output | JSON: 5 finding objects; terminal: 1 collapsed group with `(x5)` |
| EC-002 | 1 unique + N identical findings, collapse=true, JSON output | JSON: N+1 finding objects; terminal: 1 individual line (no suffix) + 1 collapsed group |
| EC-003 | N=5 identical findings, collapse=false (--no-collapse), JSON output | JSON: 5 finding objects; terminal: 5 individual lines; same JSON regardless of flag |
| EC-004 | N=5 identical findings, CSV output | CSV: 5 data rows (one per finding); terminal: 1 collapsed group |
| EC-005 | Finding appears exactly once (unique key) | Terminal: rendered individually, no suffix, full evidence; JSON: 1 object; CSV: 1 row |
| EC-006 | All N findings are unique, collapse=true | Terminal: N individual lines (N singleton groups, no suffixes); JSON: N objects; same output as pre-v0.8.0 |
| EC-007 | Finding with summary containing attacker-controlled bytes | JSON: raw bytes serialized as JSON string (serde Serialize); terminal: bytes escaped via escape_for_terminal; CSV: neutralize_csv_injection applied; each reporter applies its own encoding independently |
| EC-008 | Collapse pass applied, then JSON rendered | JSON rendering occurs on the original slice, not on any collapsed intermediate. The collapse pass result is ephemeral, used only within TerminalReporter::render |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 1000 `(Anomaly, Inconclusive, Low, "Empty UA")` findings, terminal + JSON render | Terminal: 1 line with `(x1000)`; JSON: array of 1000 objects | primary integration test (ADR-0003 invariant) |
| 1 unique finding + 5 identical findings, terminal + JSON | Terminal: 1 line (no suffix) + 1 line `(x5)`; JSON: 6 objects | mixed scenario |
| 1 finding (no repetition), collapse=true | Terminal: 1 line, no suffix; JSON: 1 object; CSV: 1 row | singleton / non-repeated finding |
| N=5 identical findings, CSV render | CSV body has 5 rows; column values identical across rows | CSV invariant |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | JSON reporter produces N objects for N identical findings when collapse=true | integration: test_BC_2_11_029_json_receives_full_findings |
| — | CSV reporter produces N rows for N identical findings when collapse=true | integration: test_BC_2_11_029_csv_receives_full_findings |
| — | Non-repeated finding renders without count suffix | unit: test_BC_2_11_029_non_repeated_finding_no_suffix |
| — | --no-collapse flag does not affect JSON finding count | integration: test_BC_2_11_029_no_collapse_flag_json_invariant |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC enforces the fundamental contract that the Reporting and Output capability preserves forensic integrity: machine-readable outputs always carry the complete finding stream regardless of any display-layer transforms applied to the terminal view |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- this BC is the explicit enforcement of INV-4 for the collapse feature; the raw finding stream is the canonical data; terminal collapse is a lens, not a filter) |
| Architecture Module | SS-11 (reporter/terminal.rs, reporter/json.rs, reporter/csv.rs, reporter/mod.rs, src/main.rs) |
| Stories | STORY-118 |
| Issue | #259 (Collapse repeated low-value findings) |
| ADR | ADR-0003 (display-layer aggregation subsection; the binding rule: "JSON and CSV consumers MUST receive the complete, unaggregated slice") |

## Related BCs

- BC-2.11.025 -- depends on (collapse pass that this BC constrains to terminal-layer only)
- BC-2.11.028 -- composes with (opt-out flag; regardless of flag value, JSON/CSV are unaffected)
- BC-2.11.001 -- composes with (JsonReporter finding count invariant; this BC extends it to cover the collapse scenario)
- BC-2.11.019 -- composes with (TerminalReporter section structure; the FINDINGS section content changes with collapse but JSON/CSV are separate reporters)

## Architecture Anchors

- `src/reporter/mod.rs` -- `Reporter` trait signature (`fn render(&self, ..., findings: &[Finding], ...) -> String`); immutable slice reference enforces no-mutation at the trait boundary
- `src/reporter/json.rs` -- JsonReporter::render iterates every finding in the slice; no collapse path
- `src/reporter/csv.rs:40` -- `neutralize_csv_injection(s: &str) -> String` (confirmed present at csv.rs:40); called for every field of every finding
- `src/reporter/csv.rs:76` -- `for f in findings { ... }` render loop (confirmed present at csv.rs:76); iterates every finding in the slice; no collapse path
- `src/main.rs:~run_analyze` -- **INSERTION TARGET (code TBD by STORY-118):** TerminalReporter construction site; `render: FindingsRender` field will be set here (replacing the former `collapse_findings: bool` + `show_mitre_grouping: bool` pair). Pre-story line-range approximation: ~370-375.
- `src/reporter/terminal.rs:63-75` -- **INSERTION TARGET (code TBD by STORY-118):** TerminalReporter struct; `pub render: FindingsRender` field replaces `pub show_mitre_grouping: bool` + `pub collapse_findings: bool` (current struct has only `use_color`, `show_mitre_grouping`, `show_hosts_breakdown`); collapse pass will be a private method added by STORY-118

## Story Anchor

STORY-118

## VP Anchors

- — (new VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (this BC governs the data flow across reporters, which is pure) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
