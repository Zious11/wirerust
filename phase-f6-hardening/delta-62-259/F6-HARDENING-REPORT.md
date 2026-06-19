# Phase F6 — Targeted Hardening Report: Grouped-Collapse Delta (issue #62 / #259)

- **Date:** 2026-06-19
- **Target HEAD:** `adcf4e94be024ea64e29c2861d394dc5e9052631` (develop) — confirmed via `git rev-parse HEAD`.
- **Baseline:** `v0.8.0`
- **Crate version:** 0.9.0
- **Mode:** Feature Mode F6 (delta-scoped hardening; full-tree regression + security)

## Delta Scope (source, `git diff v0.8.0..develop -- src/`)

| File | Lines | Change |
|------|-------|--------|
| `src/reporter/terminal.rs` | +279/-81 | `FindingsRender { grouping, collapse }` struct; `render_findings_grouped_collapsed`; 4-arm dispatch; `collapse_findings_pass_refs` shared helper |
| `src/main.rs` | +68 | `grouping_from_flag` / `collapse_findings_from_flag` helpers + construction sites |
| `src/cli.rs` | +13 | `--mitre` / `--no-collapse` flags |

Non-source diff: `Cargo.toml` (version bump 0.8.0→0.9.0 only), `Cargo.lock` (no crate added/removed), `CHANGELOG.md`, `README.md`, `docs/adr/0003`, test files. **No dependency churn.**

The delta touches ONLY terminal rendering + CLI wiring. It does NOT touch `decoder.rs`, `dispatcher.rs`, or any protocol parser.

---

## 1. Mutation Testing (cargo-mutants 27.0.0)

Scope: `--file src/reporter/terminal.rs --file src/main.rs`, `--timeout 120`.

| Metric | Count |
|--------|-------|
| Total mutants planned | 109 |
| Caught (killed) | 85 |
| Missed (survived) | 15 |
| Unviable | 8 |
| Baseline build | 1 (Success) |

**Kill rate (viable basis: caught / (caught+missed)) = 85 / 100 = 85.0%.**

`src/reporter/terminal.rs` and `src/main.rs` are not assigned a per-module criticality
threshold in `module-criticality.md` (rendering/CLI plumbing is not in the CRITICAL/HIGH
pure-core set covered by Kani). Treating this delta as MEDIUM (display reshape, no wire
invariant) → 80% target → **85% PASSES**. The single delta-introduced survivor is a
cosmetic color-discrimination gap (below), not a logic-correctness escape.

### Survivor inventory (15) — delta vs. pre-existing

Only **1 of 15** survivors is in delta-introduced code.

**DELTA-introduced survivor (1):**

| Loc | Mutant | Why it survives |
|-----|--------|-----------------|
| `terminal.rs:568:33` | delete match arm `Confidence::High` in `render_findings_grouped_collapsed` | Color-ladder arm. Deleting `Confidence::High => red().bold()` falls through to `_ => yellow()`. The collapsed-path color test (`test_BC_2_11_031_grouped_collapse_color_ladder`) only asserts that `(x2)` sits **inside** the ANSI color span (before the reset) — it does NOT assert *which* color. A yellow header still satisfies that, so the mutant is not killed. Cosmetic only (red+bold vs yellow for Likely/High); no functional/security impact. Sibling arm `terminal.rs:430` in `render_findings_collapsed` IS caught, so the flat path has a color-discriminating test the grouped-collapsed path lacks. |

**Pre-existing survivors (14) — NOT delta-introduced:**

- `terminal.rs:299:21` — delete `Confidence::High` arm in `render_finding_prefix`. This
  helper and its color arm **predate the delta** (present unchanged at `v0.8.0`, was line 257).
  Same color-discrimination class as L568. Outside F6 delta remediation scope.
- `main.rs` (13 survivors): all in `main()` (L74) and `run_analyze` (L154–285) — the
  pre-existing packet-decode loop, `ReassemblyConfig` construction, ARP match-guard, and
  packet/byte counters. These are integration-glue lines exercised only by end-to-end CLI
  runs; they are pre-existing `run_analyze` plumbing, not the delta's flag helpers.
  **The delta flag helpers `grouping_from_flag` / `collapse_findings_from_flag` have NO
  survivors** — `grouping_from_flag` mutants are Unviable (return-type default) and the
  polarity is pinned by `test_BC_2_11_030_*` and `test_BC_2_11_028_*` guards.

### SPECIFIC CHECK — confidence_rank secondary sort key (F5 Pass C O-2)

**Result: the F5 O-2 coverage gap is CONFIRMED REAL, but is NOT visible as a mutation survivor.**

Nuance (important):

- `confidence_rank` is a **module-level single-source function** (BC-014) shared by both
  `render_findings_grouped` (Grouped+Expanded) and `render_findings_grouped_collapsed`
  (Grouped+Collapsed). cargo-mutants generated function-body mutants at `terminal.rs:85`
  (`confidence_rank -> 0`, `confidence_rank -> 1`); **both were CAUGHT**.
- BUT they are killed by `test_BC_2_11_014_sort_by_confidence_within_same_verdict`, which
  runs through `mitre_reporter()` = `{Grouped, Expanded}` — the **render_findings_grouped**
  path, NOT the collapsed path.
- cargo-mutants 27.0 does **not** emit a mutant that drops the `confidence_rank` element
  from the inline `sort_by_key` tuple at `terminal.rs:534–535` inside
  `render_findings_grouped_collapsed` (its operator set mutates whole-function bodies and
  binary ops, not tuple-element deletion inside a closure). So there is no survivor that
  directly probes the collapsed path's secondary key — the absence of a survivor here is a
  **blind spot of the mutation operator set, not evidence of coverage.**
- Manual confirmation: the only same-verdict ordering test on the collapsed path,
  `test_BC_2_11_033_first_occurrence_in_sorted_bucket_order`, discriminates solely on
  **verdict_rank** (Likely rank=0 vs Inconclusive rank=2). Its two same-verdict members are
  both Likely/High, so they do not exercise `confidence_rank` at all. **No test exists with
  two same-verdict, different-confidence findings rendered through `{Grouped, Collapsed}`.**

**Verdict on O-2: CONFIRMED real coverage gap.** Recommend adding a regression test:
two findings, same tactic bucket, same verdict (e.g. both `Likely`), different confidence
(`High` vs `Low`), emitted Low-first, rendered via `grouped_collapse_reporter()`, asserting
the `High` member's group header renders before the `Low` member's. This would (a) pin the
collapsed-path secondary sort key, and (b) is the test the F5 reviewer asked for. Filed as
**F6-FINDING-1** below.

---

## 2. Full Regression Suite

`cargo test --all-targets` — **1697 passed, 0 failed, 0 ignored** across all binaries
(lib unit + integration + bench-as-test). Byte-identical output gate (the `(xN)` /
byte-identical singleton / flat-path-unchanged tests in `story_118`/`story_119`) all green.

`cargo fmt --check` — PASS (exit 0).
`cargo clippy --all-targets -- -D warnings` — PASS (no warnings, exit 0).

---

## 3. VP-012 `escape_for_terminal` (proptest)

VP-012 is **locked/verified** (`verification_lock: true`, `proof_completed_date 2026-06-02`,
hash `0b1fd48d…`). Not edited. Re-ran to confirm it still holds on the reshaped path:

- 4 proptest properties (1000 cases each): `prop_no_dangerous_bytes_survive`,
  `prop_printable_ascii_unchanged`, `prop_non_ascii_unicode_above_c1_unchanged`,
  `prop_escape_is_length_conserving` — **PASS**.
- Grouped-collapse path escaping confirmed by `story_119::test_BC_2_11_031_escape_for_terminal_in_grouped_collapse_path`
  and `story_119::test_BC_2_11_032_escape_preserved_in_bucket_evidence` — **PASS**. The
  collapsed-path summary (`terminal.rs:558`) and sampled evidence (`terminal.rs:586`) both
  route through `escape_for_terminal`. No new escaping bypass introduced.

---

## 4. Kani Proofs + Fuzz Targets — UNAFFECTED (no re-run required)

Kani-harness-bearing modules: `decoder.rs`, `dispatcher.rs`, `reassembly/{segment,mod,flow}.rs`,
`analyzer/{dnp3,arp,tls,modbus}.rs`, `mitre.rs`.
Fuzz targets: `fuzz_decode_packet`, `fuzz_dnp3_parse`, `fuzz_modbus_parse`.

The source delta is confined to `cli.rs`, `main.rs`, `reporter/terminal.rs`. **None of the
proof-bearing or fuzz-target modules are in the diff.** The terminal reporter consumes
already-constructed `Finding`/`Summary` values and emits a `String`; it feeds no input back
into the decoder/dispatcher/parser modules. Therefore the Kani proofs and fuzz targets are
**provably unaffected — no input change reaches those modules** — and do NOT need re-running
for this delta. (They remain valid at their last green run.)

---

## 5. New VP Assessment — NO new VP required

**Decision: this rendering reshape introduces NO new verification property.**

Justification:
- A formal VP (Kani proof or proptest invariant) is warranted when the delta introduces a
  new **wire-protocol / decoder / parser invariant** or a new **safety/security** property
  over untrusted input. This delta does neither: it is a terminal-display reshape over
  already-validated `Finding` values.
- The one security-relevant invariant on this path — terminal-escape neutralization of
  attacker-controlled summary/evidence bytes — is **already covered by the locked VP-012**,
  and §3 confirms the reshaped collapsed path still routes through `escape_for_terminal`.
  No new escape surface was created.
- MITRE tactic grouping order (the grouping half of the reshape) is already covered by the
  verified **VP-016** (`reporter/terminal.rs`, integration). The collapse half is an
  ordering/deduplication display concern adequately covered by example-based tests
  (`story_118`/`story_119`, BC-2.11.025–034), which is the correct rigor tier for a
  deterministic display transform.
- Remaining gap is a **test-coverage** gap (O-2 secondary sort key), not a missing formal
  invariant. It is remediated by adding one example test (F6-FINDING-1), not by minting a VP.

---

## 6. Security / Supply-Chain (full tree)

| Tool | Result |
|------|--------|
| `cargo audit` | exit 0. 1 *allowed warning*: RUSTSEC-2026-0097 (`rand 0.8.5` unsound, via `phf_generator → phf_codegen → tls-parser`). Build-time transitive dep; not in the delta; no dependency change in this delta. No vulnerabilities. |
| `cargo deny check` | **advisories ok, bans ok, licenses ok, sources ok** (exit 0). |
| `cargo clippy --all-targets -- -D warnings` | PASS (exit 0). |
| `cargo fmt --check` | PASS (exit 0). |

No HIGH or CRITICAL findings → no security-reviewer escalation triggered. The `rand`
advisory is a pre-existing transitive-dependency state unrelated to the grouped-collapse
delta (the delta adds/removes zero dependencies).

---

## Findings

### F6-FINDING-1 (coverage gap — confirms F5 Pass C O-2)

**`render_findings_grouped_collapsed` secondary sort key (`confidence_rank`) is untested in
the collapsed path.** No test renders two same-verdict / different-confidence findings
through `{Grouped, Collapsed}`. The shared `confidence_rank` fn-body mutant is killed only
via the Expanded path; the collapsed path's inline `sort_by_key` tuple is not directly
mutated by cargo-mutants 27.0, so the gap is invisible to mutation metrics. Recommend a
regression test (see §1 SPECIFIC CHECK). Severity: LOW (correctness is provided by the
shared single-source fn; this is a missing guard against a future divergence of the
collapsed-path sort tuple). Non-blocking.

### F6-FINDING-2 (cosmetic — color-discrimination gap, delta)

`terminal.rs:568` `Confidence::High` color arm in `render_findings_grouped_collapsed`
survives mutation: the collapsed color-ladder test asserts span placement but not the red+bold
color. Optional: tighten `test_BC_2_11_031_grouped_collapse_color_ladder` to assert the
red+bold ANSI code for Likely/High (mirroring the caught flat-path L430 sibling). Severity:
LOW cosmetic. Non-blocking.

---

## VERDICT

**HARDENED** (with two non-blocking LOW findings).

- Mutation: **85.0% kill rate** (85/100 viable); only **1 of 15** survivors is
  delta-introduced and it is cosmetic (color discrimination).
- Regression: **1697 passed, 0 failed**; fmt + clippy clean.
- VP-012 (locked) re-confirmed; grouped-collapse path escapes summary+evidence.
- Kani proofs + fuzz targets **provably unaffected** (modules not in diff) — no re-run needed.
- **No new VP required** — terminal-display reshape with no new wire/decoder invariant;
  escape security already covered by locked VP-012, grouping order by verified VP-016.
- Security: `cargo deny` clean; `cargo audit` clean (1 pre-existing allowed transitive
  warning, unrelated to delta); no HIGH/CRITICAL → no security-reviewer escalation.
- **confidence_rank flag (F5 O-2): CONFIRMED real coverage gap** (F6-FINDING-1) — a test
  should be added; the gap does not block hardening because correctness is provided by the
  shared single-source `confidence_rank` fn (killed via the Expanded path).
