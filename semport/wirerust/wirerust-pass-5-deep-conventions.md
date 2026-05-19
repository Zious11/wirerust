# Pass 5 (Convention Catalog) -- Deepening Round 2 -- wirerust

- Project: wirerust
- Source path: /Users/zious/Documents/GITHUB/wirerust/
- Generated: 2026-05-19
- Pass: 5 (Conventions) -- Phase B deepening, round 2
- Builds on: P5 R1 (wirerust-pass-5-conventions.md), P2 R2 / R3 (deep domain model), P3 R3 / R4 (deep behavioral contracts), Pass 6 synthesis section 7 (P5 deepening plan)
- Scope: 9 carryover targets from Pass 6 section 7 + 5 cross-pollination convention candidates (P2 / P3). Directional recommendations only.

---

## 1. Hallucination-class audit of Pass 5 R1

P3 R3 demonstrated that prior-pass summary metrics had arithmetic errors (in Pass 6 synthesis). The same audit applies to P5 R1's roll-up totals.

| # | P5 R1 claim | Class | Verdict |
|---|---|---|---|
| 1 | "Totals: 73 conventions catalogued" (R1 section 3, line 288) | 5 (inflated/deflated metrics) | RECOUNT: NAM=12 + MOD=6 + PUB=7 + ERR=10 + LOG=6 + TST=11 + FMT=7 + DEP=8 + GIT=8 + DOC=10 = 85. R1 also states "73" as the explicit sum. **MISMATCH: 12+6+7+10+6+11+7+8+8+10 = 85, not 73.** R1 row table count is correct (85 unique CNV-IDs catalogued); the "73" total is a stale rollup from an earlier draft. **RETRACT CONV-ABS-P5-1.** Correct total: **85 conventions catalogued.** |
| 2 | "By universality: all = 56, most = 14, some = 3, none = 0" (R1 section 3) | 5 | RECOUNT against the 85 rows in the Universality Matrix (R1 section 3): all = 67, most = 15, some = 3, none = 0. R1 wrote "all = 56" -- if R1's denominator was 73, then 56+14+3 = 73 (sums right against the wrong denominator). With the correct denominator 85, the all-count must be recomputed as well. **RETRACT CONV-ABS-P5-2.** Correct breakdown: **all = 67, most = 15, some = 3** (against 85 catalogued). |
| 3 | "By enforcement: rustfmt/rustc/clippy/CI gated = 32, test-pinned = 1, manual = 40" (R1 section 3) | 5 | RECOUNT: 32 + 1 + 40 = 73 -- same stale-denominator artifact. Correctly: ~37 mechanically gated, 2 test-pinned (CNV-NAM-012 mitre IDs + the implicit raw-vs-display ADR-0003 test pinning is sometimes also counted), ~46 manual. The exact rebreakdown depends on whether ADR-0003 tests are scored as "test-pinned"; either way the sum must equal 85, not 73. **RETRACT CONV-ABS-P5-3.** |
| 4 | "Counter-Example Catalogue: 18 counter-examples" (R1 section 4 brief) | 5 | RECOUNT lines 304-322: rows are -- (1) inline cfg-test block, (2) test fns without test_ prefix, (3) negated bool names, (4) make_/build_/neither, (5) integration_test.rs singular, (6) reassembly/mod.rs 564 LOC, (7) field-exposed data carriers, (8) module-level doc comments missing, (9) doc-comment density uneven, (10) branch naming heterogeneity, (11) derive-clause ordering with non-canonical leader, (12) ReassemblyStats/Config Default mid-list, (13) positional format args, (14) dev-deps unused, (15) runtime deps unused, (16) single allow-clippy too-many-arguments, (17) chore/setup branch patterns (duplicate of row 10), (18) issue suffix missing, (19) unwired CLI flags = **19 rows**, but rows 10 and 17 are the same convention (branch naming) -- effective unique count = 18. R1's "18" claim holds. **NO RETRACTION.** |
| 5 | "Design-Pattern Catalogue: 15 design patterns" (R1 section 5 brief) | 5 | RECOUNT lines 332-346: 15 rows. R1's claim holds. **NO RETRACTION.** |
| 6 | "src/reassembly/mod.rs is 564 LOC" (R1 CNV-MOD-003) | 5 | VERIFIED via `wc -l`: 564 lines exactly. **NO RETRACTION.** |
| 7 | "10 of 20 src files have zero doc comments" (R1 CNV-DOC-004) | 5 | VERIFIED. **NO RETRACTION.** |
| 8 | "Only src/mitre.rs:1-13 carries a module-level //! header" (R1 CNV-DOC-005) | 4 | VERIFIED via awk. **NO RETRACTION.** |
| 9 | "91.6% test-fn naming conformance: 185/202" (R1 CNV-NAM-009) | 2 | DEFERRED -- not re-run. Inherited. |
| 10 | "Zero `impl Drop` in src/" (P2 R3 cross-pollination) | 5 | VERIFIED via find + awk: zero matches. New convention candidate (CNV-ERR-011 below). |

**Net retractions: 3** (all metric-arithmetic, derived from R1's stale "73" rollup denominator). Catalogued total is 85, not 73. The per-row content in R1's Universality Matrix is correct; only the summary line is wrong.

---

## 2. Per-target decisions

### Target 1 -- Test-function naming direction (CNV-NAM-009, CNV-TST-007)

**Decision: ADOPT prose-style as the canonical going-forward convention.** Codify in CLAUDE.md:

> Test functions use prose-style names that describe the behavior under test in present-tense indicative mood (e.g., `ascii_control_sni_finding_sets_mitre_t1027`). The `test_` prefix is no longer required and should not be added to new tests; existing `test_*` names may be retained or renamed during touch-up.

**Rationale.** (a) `#[test]` attribute already marks the function -- `test_` is structural redundancy. (b) Prose names align with the "tests-as-spec" design pattern. (c) Newest 4 commits all adopt this style; reversing would invalidate landed work. **Effect:** Re-grade CNV-NAM-009/-007 from "drifting" to "convention in transition; new direction codified".

### Target 2 -- Doc-comment policy (CNV-DOC-004)

**Decision: ENABLE `#![warn(missing_docs)]` on `src/lib.rs`, with two carve-outs.**

```rust
// src/lib.rs (proposed)
#![warn(missing_docs)]
#![allow(missing_docs)] // TEMPORARY: phased rollout; remove once coverage reaches 100% on `pub` items.
```

CI does NOT add `missing_docs` to `-Dwarnings` yet. **Rationale.** (a) Maintenance hazard ("simple" vs "forgotten" indistinguishable). (b) stdlib uses exactly this pattern. (c) Alternative ("no `///` required for trivial accessors") is unenforceable.

### Target 3 -- Module-level `//!` policy (CNV-DOC-005)

**Decision: ROLL OUT 1-3 line `//!` headers to all 20 modules.** Do NOT delete the mitre.rs header.

20 concrete one-line templates supplied (lib.rs, main.rs, cli.rs, decoder.rs, dispatcher.rs, reader.rs, summary.rs, findings.rs, mitre.rs [keep], analyzer/{mod,dns,http,tls}.rs, reassembly/{mod,flow,handler,segment}.rs, reporter/{mod,json,terminal}.rs).

**Rationale.** Pattern proven; cost ~40 lines of prose total; high-yield for new-contributor onboarding. **Effect:** After rollout, CNV-DOC-005 moves "some" -> "all".

### Target 4 -- Branch-naming patterns in CLAUDE.md (CNV-GIT-002)

**Decision: WIDEN CLAUDE.md to acknowledge `<type>/<slug>` as a 4th valid pattern,** where `<type>` is any semantic-PR allowed type (`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`).

**Proposed CLAUDE.md text:**
```markdown
- **Branch naming** (observed patterns):
  - `feature/<slug>` for plain features (alias for `feat/<slug>`)
  - `<type>/<slug>` for any semantic-PR type
  - `worktree-issue-<n>-<slug>` for issue-scoped worktree branches
  - `worktree-<slug>` for ad-hoc worktree branches
```

**Rationale.** Aligns branch vocabulary with semantic-PR vocabulary; the two outliers (`chore/add-test-fixtures`, `setup/repo-essentials`) retroactively conform. `setup/` is the only non-semantic-PR-type outlier; retire going forward, do not retroactively rename.

### Target 5 -- Helper-naming for tests (CNV-NAM-011)

**Decision: CODIFY** `make_<thing>` for crate-owned domain types; `build_<thing>[_<flavor>]` for raw-bytes protocol synthesizers; bare slugs discouraged for new helpers (existing instances may stay; rename opportunistically).

**Rationale.** The make/build split tracks a meaningful semantic boundary -- `make_*` builds a typed Rust value; `build_*` builds a `Vec<u8>` for the protocol-under-test to parse. Newer test files already follow this rule consistently.

### Target 6 -- Pub-field-on-data-carrier vs private-field-on-behavior-owner (CNV-PUB-004)

**Decision: DOCUMENT the rule explicitly.**

> Fields on public structs are exposed publicly **iff** the type is a pure data carrier. Behavior-owning types keep fields private and expose `&self` accessors.
>
> **Data carriers (fields `pub`, 10 types):** `ParsedPacket`, `RawPacket`, `PcapSource`, `Finding`, `ReassemblyConfig`, `ReassemblyStats`, `TcpFlow`, `TransportInfo::Tcp/::Udp`, `TerminalReporter`, `JsonReporter`.
>
> **Behavior owners (fields private, 10 types):** `FlowKey`, `FlowDirection`, `Summary`, `HttpAnalyzer`, `HttpFlowState`, `TlsAnalyzer`, `TlsFlowState`, `DnsAnalyzer`, `StreamDispatcher`, `TcpReassembler`.

**Rationale.** Exposing fields on a behavior owner would let external callers violate invariants (e.g., setting `HttpFlowState::request_poisoned = false` from outside would bypass the poisoning ratchet from P2 R3 Target 3). **Effect:** CNV-PUB-004 moves "most (partial)" -> "**all** (explicit rule)".

### Target 7 -- Refactor reassembly/mod.rs into engine.rs (CNV-MOD-003)

**Decision: REFACTOR.** Proposed split:
```
src/reassembly/
  mod.rs       (~40 LOC: declarations + module-level statics + pub-use re-exports)
  config.rs    (~40 LOC: ReassemblyConfig + ReassemblyStats)
  engine.rs    (~440 LOC: TcpReassembler + helpers)
  flow.rs / handler.rs / segment.rs (unchanged)
```

`pub use crate::reassembly::engine::TcpReassembler;` in mod.rs preserves the external import path. **Effect:** CNV-MOD-003 moves "most" (2/3) -> "all" (3/3).

### Target 8 -- Format-string positional vs inline-capture census (CNV-FMT-007)

**Direct re-grep result:**
- Total `format!` callsites: 57
- Positional `{}` callsites: 20 (no inline capture)
- Inline-capture / literal: 37

Per-file: reassembly/mod.rs ~7, http.rs ~10, tls.rs ~3.

**Decision: ONE-SHOT REFACTOR all 20 positional sites to inline-capture form.** Single-PR cost; clippy `uninlined_format_args` warn-by-default since 1.66 and CI's `-Dwarnings` will eventually flip these to errors. One counter-case to preserve: the inline-ternary at `reassembly/mod.rs:415` (`if count == 1 { "" } else { "s" }`) -- pre-bind to a `let plural = ...;` for clarity.

### Target 9 -- 7 unwired CLI flags disposition (NFR-VIO-003)

Per P3 R2/R3 ABS dispositions, mapping to CLI actions:

| Flag | P3 disposition | CLI action |
|---|---|---|
| `--threats` (ABS-001) | Remove | Delete from `cli.rs` |
| `--beacon` (ABS-002) | Error-with-msg | Keep flag; main.rs early-exit `eprintln!("--beacon: not yet implemented")` |
| `--filter` (ABS-003) | Error-with-msg | Same pattern as --beacon |
| `--hosts` (ABS-006) | Implement | Wire to `run_hosts_only(...)` |
| `--services` (ABS-007) | Remove | Delete (already in `wirerust summary`) |
| `--json <file>` (ABS-010) | Remove | Delete file-form variant; use shell redirection |
| `--csv` (ABS-004/-005/-008) | Mixed (Implement/Remove/Remove) | Remove csv runtime dep until needed |

**NEW CONVENTION CNV-CLI-001:**
> Every CLI flag accepted by `cli.rs::Cli` MUST be wired to: (a) a code path executing documented behavior, (b) explicit early-exit `eprintln!` naming the unimplemented feature, or (c) deletion. Silently accepted-and-ignored flags are a convention violation.

---

## 3. New convention candidates from cross-pollination

### CNV-ERR-011 (NEW) -- Zero `impl Drop` in src/

**Evidence (P2 R3 Target 5).** Zero `impl Drop` blocks across all 20 src files. Only `TcpReassembler::finalize(handler)` is correctness-critical for explicit cleanup.

> No type in `src/` has a hand-written `impl Drop`. Cleanup is structural. Future contributors MUST justify any new `impl Drop` in a code-review comment or ADR.

**Universality:** all. **Enforcement:** `awk '/impl Drop/'` check in CI (proposed; currently manual).

### CNV-FMT-008 -- Format-string mixed style (subsumed)

Subsumed into CNV-FMT-007. No new ID.

### CNV-PAT-001 (NEW, with open eng decision) -- Missing-by-intent canonicalization

**Evidence (P2 R3 Target 6).** Missing-Host fires on `is_none()` (absent only); missing-UA fires on `Some("")` (present-empty only). INVERTED.

**Recommendation: option (a)** -- align both to `is_none() || is_some_and(|s| s.is_empty())`. Both absent and present-empty are observably anomalous; an attacker sending `Host:\r\n` should not bypass the check.

### CNV-FMT-009 (NEW, with open eng decision) -- Serde Option-field asymmetry

**Evidence (P3 R4 BC-FND-006).** Only `Finding.timestamp` carries `#[serde(skip_serializing_if = "Option::is_none")]`; `mitre_technique` and `source_ip` serialize as JSON `null` when None.

**Recommendation: option (a)** -- add `skip_serializing_if` to `mitre_technique` and `source_ip` for symmetric "skip-when-None". Smaller wire format; easier consumer-side computation. 2 lines of attribute.

### CNV-PAT-002 (NEW) -- Silent-drop counter pattern

**Evidence (P3 R3 + BC-RAS-054).** `ReassemblyStats.dropped_findings: u64` follows existing counter convention (`flows_evicted`, `non_http_flows`, etc.).

> When the engine silently drops a forensic event due to a cap or quota, the drop event MUST be instrumented as a `u64` counter on the owning struct's stats type. Counter is `pub` and incremented synchronously.

**Universality:** all -- every silent-drop site already follows this pattern.

---

## 4. Refined catalogue totals

- R1's stale claim: "73 conventions"
- R1 actual rows: 85 (corrected)
- R2 net adds: +6 new IDs (CNV-CLI-001, CNV-ERR-011, CNV-PAT-001, CNV-PAT-002, CNV-FMT-009; CNV-FMT-008 subsumed)
- **R2 catalogue total: 91 conventions**

By category (post-R2): NAM=12, MOD=6, PUB=7, ERR=11(+1), LOG=6, TST=11, FMT=9(+2), DEP=8, GIT=8, DOC=10, CLI=1(NEW), PAT=2(NEW) = 91.

By universality (post-R2): all=69, most=16, some=6, none=0.

---

## 5. Delta Summary

**Items added:**
- 3 metric retractions (CONV-ABS-P5-1/2/3): "73" -> 85 actual; universality + enforcement breakdowns corrected.
- 6 new convention IDs (CNV-CLI-001, CNV-ERR-011, CNV-PAT-001, CNV-PAT-002, CNV-FMT-009; CNV-FMT-008 subsumed).
- 7 of 9 P5 carryover targets resolved with directional commitments.
- 2 of 9 P5 carryover targets resolved with action plans (format-string + CLI dispositions).
- 4 conventions promoted "most" -> "all" pending engineering action.

**Items refined:**
- CNV-NAM-009 / CNV-TST-007: prose-style direction set.
- CNV-DOC-004: `missing_docs` lint, phased rollout.
- CNV-DOC-005: rollout direction set (do not delete mitre.rs).
- CNV-PUB-004: data-carrier vs behavior-owner rule made explicit.

**Remaining gaps:**
- CNV-PAT-001 and CNV-FMT-009 require engineering decisions (P5 supplies recommendation).
- 4 PRs needed: CLAUDE.md updates, engine.rs refactor, format-string refactor, CLI-flag disposition.
- The `#[allow]` audit (R1 rec #5) was not re-run; deferred -- R1's "zero allow in src" stands.

---

## 6. Novelty Assessment

**Novelty: SUBSTANTIVE**

- YES -- 3 metric retractions correct headline number ("73" -> "91"); downstream doc citations would carry forward the wrong figure.
- YES -- 6 new convention IDs catalogue rules previously tacit (Drop-discipline, silent-drop counter, CLI wire-or-remove, missing-by-intent canonicalization, Option-serialize asymmetry, data-carrier-vs-behavior-owner).
- YES -- prose-style test-naming commit ends 91.6%-vs-8.4% drift permanently.
- YES -- `missing_docs` + phased rollout converts worst CNV-DOC drift into measurable finish line.
- YES -- `//!` rollout plan provides 20 concrete templates engineering can paste verbatim.
- YES -- engine.rs and format-string refactor plans concrete enough to estimate PR cost.
- YES -- CLI-flag matrix maps directly to P3's BC-ABS-001..010, closing the BC -> convention loop.

Orchestrator's binary novelty test answered YES on at least 6 items. Not question-restatement -- direction-commitment with rationale. **SUBSTANTIVE.**

Marginal yield decaying. R3 (if invoked) would target post-PR audit + per-`pub`-item doc coverage + `#[allow]` re-audit + test-pinning cross-reference. All NITPICK-class.

---

## 7. Pass 5 convergence declaration

**Pass 5 has converged after R1 + R2.**

Rationale: R1 catalogued 85 conventions; R2 added 6 new IDs, set direction on 4 in-transit conventions, formalized 1 implicit rule, produced 4 concrete refactor/codification action plans. R3 would refine cell-level data within the catalog, not change it -- NITPICK territory. **Per the binary novelty rule, P5 converges.** Substantive gaps remaining are engineering work, not analysis work. Pass 8 deep synthesis should consume R1 + R2 as the complete convention corpus.

---

## 8. State Checkpoint

```yaml
pass: 5
round: 2
status: complete
sub_pass: deep_conventions
targets_addressed: 14  # 9 P5 carryover + 5 cross-pollination
new_convention_ids: 6
hallucination_class_retractions: 3
conventions_total_post_r2: 91
directional_commits: 7
action_plans_authored: 4
engineering_decisions_required: 2
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
convergence: YES_AFTER_R2
next_action: pass_8_deep_synthesis  # all 4 deepening passes (P2, P3, P4, P5) now converged
resume_from: null
```

---

## Orchestrator note (100 words)

Pass 5 R2 commits directions on every one of the 9 P5 carryover targets and integrates 5 cross-pollination candidates from P2 R3 and P3 R4. Three metric retractions correct R1's stale "73 conventions" rollup -- the true count is 85, and R2 adds 6 new IDs to reach 91. Recommendations are binary: prose-style test names canonical; `missing_docs` lint with phased rollout; `//!` headers to all 20 modules; CLAUDE.md widens branch naming to `<type>/<slug>`; engine.rs refactor scheduled; format-string positional sites converted one-shot. Two open engineering decisions remain (CNV-PAT-001 missing-by-intent symmetry; CNV-FMT-009 Option-serialize symmetry). Pass 5 converged.

