---
document_type: maintenance-sweep-output
sweep: pattern-consistency+spec-coherence
sweep_id: maint-2026-09-05 / Sweep-3+Sweep-7
producer: consistency-validator
develop_head: 0b1ea8064f821e8d1de3378a13ddd5aaf386aa42
date: 2026-09-05
---

# Maintenance Sweep Findings — maint-2026-09-05

Run: **maint-2026-09-05**, Sweeps 3 (Pattern Consistency) + 7 (Spec Coherence).
Scope A: `src/` (+ non-test call sites), codebase at `0b1ea806` (develop, v0.13.3).
Scope B: `.factory/` worktree (factory-artifacts branch, currently `cf6a114b`), index-first.
Analysis only — no source artifacts modified, nothing committed.

Baseline gates confirmed clean: `cargo clippy --all-targets -- -D warnings` = 0 warnings;
`cargo fmt --check` = clean.

This file replaces the stale `maint-2026-07-08` run (develop `b642c0f`) in full.

---

## Summary Table

| ID | Section | Description | Classification |
|----|---------|-------------|----------------|
| PF-A-001 | A | `reporter/json.rs:81` bare `.unwrap()` vs `reporter/csv.rs` `.expect("…cannot fail")` convention | MANUAL-FIX (S) |
| PF-A-002 | A | `reassembly/mod.rs:299,318,372,513,620` bare `.unwrap()` vs `enip.rs:798,996` justified `.expect()` convention | MANUAL-FIX (S) |
| PF-A-003 | A | `iec104.rs:101` — `Iec104ParseError` enum defined, never returned (dead skeleton), diverges from repo-wide `Option<T>` parse-error convention | MANUAL-FIX (S) |
| PF-A-004 | A | `ts` (dnp3.rs:397, iec104.rs:1260) vs `timestamp` (handler.rs trait family) parameter-naming inconsistency across analyzer families | MANUAL-FIX (S) / NIT |
| PF-A-005 | A | `-W clippy::pedantic -W clippy::nursery` surfaces 2119 warnings beyond the `-D warnings` gate; dominant categories judgment-requiring: `missing_const_for_fn` (110), `must_use_candidate`+`must_use` (91+37), `struct_field_names` (76), `cast_possible_truncation` (18+10+7+6+6+5, binary-parser safety-relevant), `redundant_clone` (14, Family-B `.clone()` sites) | MANUAL-FIX (M) |
| PF-A-006 | A | Subset of the same pedantic sweep is mechanically `cargo clippy --fix`-able if opted in: `cast_lossless` (~69), `uninlined_format_args` (35), `redundant_closure` (35) — not currently gated, no action required unless opted in | AUTO-FIXABLE |
| PF-A-007 | A | Error-handling strategy overall: `Option<T>` convention for fallible parse paths (dnp3/enip/modbus/tls/http/iec104), `anyhow::Result`+typed error enums for I/O (reader.rs). No production `todo!()`/`unimplemented!()`; all remaining panic-family sites are `#[cfg(test)]`/`#[cfg(kani)]` or size-guarded infallible casts. | CLEAN |
| PF-A-008 | A | Naming conventions old vs new modules: uniform `<Proto>Analyzer`/`<Proto>FlowState` trio + `on_data`/`on_flow_close`/`summarize` dispatch surface across tls/modbus/http/dnp3/enip/iec104; "analyzer" terminology consistent throughout | CLEAN |
| PF-A-009 | A | Two documented dispatch families — `StreamHandler`/`StreamAnalyzer` trait family (http/tls/modbus, ADR-0001/0002/0011) vs inherent-method family (dnp3/enip/iec104, ADR-0005/0007/0010/0013) — split is explicitly specified in ADR-0005 lines 90-104, not an undocumented drift | CLEAN (not a violation) |
| PF-A-010 | A | IEC-104 architecture parity vs dnp3/enip: same struct trio, same inherent `on_data`/`on_flow_close` signatures, same single-file module layout, same `#[cfg(kani)] mod kani_proofs` placement, same paired `*_analyzer_tests.rs`/`*_e2e_real_pcaps_tests.rs` test organization | CLEAN |
| PF-A-011 | A | Import ordering vs `rustfmt.toml`: `cargo fmt --check` clean; grouping convention (std → external → crate::, alphabetized) applied uniformly old vs new modules (verified dnp3.rs, iec104.rs, enip.rs, tls.rs, http.rs, modbus.rs, reassembly/mod.rs, dispatcher.rs) | CLEAN |
| SC-001 | B | STATE.md Drift Items `PG-W84-LOCAL-BATCH`, `PG-W85-003`, `PG-W85-005` still read "pending human story-approval gate" / cite STORY-182 v2.12 & STORY-183 v2.13 as not-yet-delivered — stale against current truth: D-546 (gate PASSED 2026-09-04), D-548 (STORY-182 DELIVERED, PR #460), D-549 (STORY-183 DELIVERED, PR #462), D-550 (WAVE-86 GATE CLOSED), D-551 (v0.13.3 RELEASED). 3 rows need their "Summary"/"Target" text updated to reflect delivery + release. | SPEC-DRIFT (effort S) |
| SC-002 | B | Index version-claim verification: BC-INDEX (v2.37 ✓), VP-INDEX (v2.47 ✓), ARCH-INDEX (v2.20 ✓), STORY-INDEX (v4.23 ✓), epics.md (v2.3 ✓), stories/dependency-graph.md (v3.12 ✓) — all 6 claimed versions match actual frontmatter exactly. (Note: `specs/architecture/dependency-graph.md` is a separate architecture-section artifact at v1.6 — distinct scope, not the "dep-graph" referenced by the v3.12 claim, no conflict.) | CLEAN |
| SC-003 | B | VP-INDEX self-consistency: `total_vps: 47` = kani(16)+proptest(22)+fuzz(3)+integration_unit(6) = 47; also = p0(9)+p1(32)+test_sufficient(6) = 47. Arithmetic holds both ways. | CLEAN |
| SC-004 | B | BC coverage completeness: epics.md v2.3 `total_bcs: 380` (active) reconciles exactly against BC-INDEX v2.37's canonical derivation ("Total BCs on disk: 381. Active: 380.") — 0 unassigned, 0 double-assigned, 0 residual gap (per D-545/D-546 reconciliation). | CLEAN |
| SC-005 | B | L1→L4 chain integrity: no `product-brief.md` exists at any path, but this is a documented brownfield-project exception — `specs/domain/domain-spec.md` frontmatter explicitly states no L1 brief exists and traces instead to the brownfield ingestion corpus (`wirerust-pass-8-deep-synthesis.md`); BC-INDEX correctly traces to `prd.md`; `specs/domain-spec/{assumptions,risk-register}.md` `traces_to: ../domain/domain-spec.md` resolves correctly. | CLEAN (documented exception) |
| SC-006 | B | VP↔BC alignment spot check: VP-047 `source_bc` correctly extended with BC-2.19.029/030 (STORY-180 delivery, CV-008 RESOLVED, v2.47 modified-log) — new IEC-104 timed-command BCs propagated to VP-INDEX without drift. | CLEAN |
| SC-007 | B | Remaining open Drift Items (`STORY-INDEX-IN-INPUTS-CHURN`, `DRIFT-docstring-scan`, `DRIFT-e2e-sibling-harnesses`, `DRIFT-stale-red-scrub`, `DRIFT-py-surface-outside-bin`, `DRIFT-TOOLCHAIN-ROLL-CLIPPY`, `DRIFT-STORY183-INHERITED-PATTERN-DOC-COMMENTS`) verified still accurately reflect current truth — no drift-of-the-drift-item found beyond SC-001. | CLEAN (verified accurate) |

**Category counts (both sections combined):**

| Classification | Count |
|---|---|
| AUTO-FIXABLE | 1 |
| MANUAL-FIX | 5 |
| ARCH-VIOLATION | 0 |
| SPEC-DRIFT | 1 |
| CLEAN | 11 |
| **Total findings** | **18** |

**By section:** Part A (Pattern Consistency) = 11 findings (1 AUTO-FIXABLE, 5 MANUAL-FIX, 0 ARCH-VIOLATION, 5 CLEAN). Part B (Spec Coherence) = 7 findings (1 SPEC-DRIFT, 6 CLEAN).

---

# Part A — Pattern Consistency

## PF-A-001 — Bare `.unwrap()` vs justified `.expect()` in reporters (MANUAL-FIX, S)

**Evidence:** `reporter/json.rs:81` — `serde_json::to_string_pretty(&output).unwrap()` (bare).
`reporter/csv.rs:74,104,109,110` — same infallible-serialization class, but written as
`.expect("… cannot fail")`.

**Finding:** Both are genuinely infallible (in-memory buffer serialization), but the two reporter
modules use opposite conventions for documenting *why*. `csv.rs`'s justified-`.expect()` style is
the better pattern (self-documenting for future readers/Kani proof-writers).

**Remediation:** Change `json.rs:81` to `.expect("serde_json serialization of internal struct cannot fail")` to match `csv.rs`.

## PF-A-002 — Bare `.unwrap()` vs justified `.expect()` after `HashMap` insert (MANUAL-FIX, S)

**Evidence:** `reassembly/mod.rs:299,318,372,513,620` — `self.flows.get_mut(key).unwrap()` (bare, all
provably safe — key was just inserted). `enip.rs:798` — `.expect("just inserted")`; `enip.rs:996` —
`.expect("flow exists: inserted above and not removed")`.

**Finding:** Same "insert-then-get_mut" invariant-guard pattern, inconsistent justification style
between the shared reassembly module and the enip analyzer.

**Remediation:** Add `.expect("just inserted")`-style messages to the 5 `reassembly/mod.rs` sites.

## PF-A-003 — Dead `Iec104ParseError` enum skeleton (MANUAL-FIX, S)

**Evidence:** `iec104.rs:101` declares `pub enum Iec104ParseError` (single `Incomplete` variant,
`pub` only so `dead_code` doesn't fire) but it is never returned — `parse_apci_header` (iec104.rs:461)
and `parse_asdu` (iec104.rs:645) both use the repo-wide `Option<T>` convention like every other
protocol analyzer.

**Finding:** This is the one BC-19/IEC-104 divergence from an otherwise clean, uniform
`Option<T>`-across-all-analyzers error convention. Tracked for STORY-168 per the code comment.

**Remediation:** Either wire `Iec104ParseError` into the two parse functions' return types, or
remove the unused skeleton until STORY-168 actually needs it.

## PF-A-004 — `ts` vs `timestamp` parameter naming (MANUAL-FIX, S / NIT)

**Evidence:** `dnp3.rs:397`, `iec104.rs:1260` name the timestamp parameter `ts`; the
`StreamHandler` trait family (`handler.rs`) names the equivalent parameter `timestamp`.

**Finding:** Cosmetic-only naming drift between the inherent-dispatch family (binary-ICS
protocols) and the trait-dispatch family. Does not affect behavior.

**Remediation:** Low priority; align on `timestamp` at a convenient future touch of these files.

## PF-A-005 — Clippy pedantic/nursery lint drift beyond `-D warnings` (MANUAL-FIX, M)

**Evidence:** `cargo clippy --all-targets -W clippy::pedantic -W clippy::nursery` → 2119 warnings.
Representative non-test, judgment-requiring categories:
- `missing_const_for_fn` (nursery, 110) — e.g. `dnp3.rs:1292,1947,1973,1993,2033,2050`
- `must_use_candidate` + fn-level `must_use` (pedantic, 91+37)
- `struct_field_names` repetition (pedantic, 76)
- `cast_possible_truncation` (pedantic, 18+10+7+6+6+5) — e.g. `reader.rs:872`, `arp.rs:2312,4314,4467,4557` — safety-relevant in a binary-parser codebase, worth a targeted pass
- `redundant_clone` (nursery, 14) — e.g. `tls.rs:1512` and the Family-B `FlowKey`-by-value clone
  sites noted in PF-A-010

**Finding:** The current `-D warnings` gate is intentionally scoped to default clippy lints (per
CLAUDE.md). `doc_markdown` alone accounts for 1412 of the 2119 hits (missing code-span backticks in
doc comments) — purely cosmetic noise, not worth gating on. `cast_possible_truncation` and
`redundant_clone` are the two categories with genuine safety/efficiency value for this codebase.

**Remediation:** Do **not** globally enable `clippy::pedantic`/`clippy::nursery` (noise-to-signal
too low). Open a dedicated MANUAL-FIX PR that opts in narrowly to `clippy::cast_possible_truncation`
and `clippy::redundant_clone` only, triaging each hit as either a genuine bounds issue (`try_from`/
guard) or an intentional `#[allow(...)]` with a one-line rationale.

## PF-A-006 — Auto-fixable subset of the pedantic sweep (AUTO-FIXABLE)

**Evidence:** Within the same pedantic sweep: `cast_lossless` ("use `From`", ~69 hits across 5
size classes), `uninlined_format_args` (35), `redundant_closure` (35, e.g. `http.rs:622`,
`tls.rs:1039`, `decoder.rs:319`, `reassembly/flow.rs:163,173`, `reassembly/mod.rs:796`).

**Finding:** These three categories are mechanically resolvable via `cargo clippy --fix
--all-targets -- -W clippy::cast_lossless -W clippy::uninlined_format_args -W
clippy::redundant_closure` with no manual judgment needed. No action required unless/until these
lints are opted into the gate — flagged here purely as "if you do enable them, these three are
free."

## PF-A-007 through PF-A-011 — CLEAN

- **PF-A-007 (error-handling overall):** `Option<T>` convention holds uniformly across
  dnp3/enip/modbus/tls/http/iec104 parse paths; `anyhow::{Context,Result,anyhow}` + typed
  `EpbDecodeError`/`ShbDecodeError` enums for pcapng reader I/O (`reader.rs:423,635`). No
  production `todo!()`/`unimplemented!()` (the 2 grep hits at `tls.rs:1362`, `enip.rs:737` are in
  comments). All panic-family call sites outside PF-A-001/002/003 are either `#[cfg(test)]`,
  `#[cfg(kani)] mod kani_proofs`, or size-guarded infallible array-conversion casts
  (`decoder.rs:429-435,505,536`; `main.rs:815,1294,1296`).
- **PF-A-008 (naming conventions):** Uniform `<Proto>Analyzer`/`<Proto>FlowState` struct-trio +
  `on_data`/`on_flow_close`/`summarize` dispatch surface across every stream-based analyzer;
  packet-based `DnsAnalyzer`/`ArpAnalyzer` correctly drop `FlowState` (stateless/L2, by design).
  "Analyzer" terminology used consistently; no stray "handler"/"processor"/"dissector" naming on
  structs.
- **PF-A-009 (two dispatch families, not a violation):** The `StreamHandler`/`StreamAnalyzer`
  trait family (http/tls/modbus) vs the inherent-`on_data` family (dnp3/enip/iec104) split is
  explicitly documented in ADR-0005 lines 90-104 and referenced again in ADR-0007/0010/0013. Not
  undocumented drift.
- **PF-A-010 (IEC-104 architecture parity):** iec104.rs mirrors dnp3.rs/enip.rs exactly — same
  struct trio, same inherent `on_data`/`on_flow_close` signatures (dispatcher call sites
  `dispatcher.rs:451,461,468,503,511,519`), same single-file module layout, same
  `#[cfg(kani)] mod kani_proofs` placement (`iec104.rs:1553`), same paired
  `iec104_analyzer_tests.rs`/`iec104_e2e_real_pcaps_tests.rs` test organization mirroring the enip
  pair. One non-blocking architect note: by-value `FlowKey` in Family B forces a per-PDU
  `.clone()` (clippy::pedantic `needless_pass_by_value` at `dnp3.rs:397`, `enip.rs:693,1607`,
  `iec104.rs:1536`) — a documented tradeoff, folded into PF-A-005.
- **PF-A-011 (import ordering):** `cargo fmt --check` clean; `rustfmt.toml` pins only
  `edition=2024, max_width=100, use_field_init_shorthand, use_try_shorthand` (no
  `group_imports`/`imports_granularity`, both nightly-unstable), so grouping is convention-only —
  applied uniformly (std → external → `crate::`, alphabetized within groups) across old and new
  modules alike (spot-checked `dnp3.rs:28-33`, `iec104.rs:47-53`, `enip.rs:202-207`, `tls.rs:16-31`,
  `http.rs:14-21`, `modbus.rs:19-24`, `reassembly/mod.rs:44-55`, `dispatcher.rs:32-41`).

---

# Part B — Spec Coherence

## SC-001 — Stale STATE.md Drift Item rows (SPEC-DRIFT, effort S)

**Evidence:** `.factory/STATE.md` Drift Items table, rows `PG-W84-LOCAL-BATCH`, `PG-W85-003`,
`PG-W85-005` — all three still read "**wave-86 story adversarial CONVERGED 3/3 (D-544, passes
25/26/27); STORY-183 v2.13; pending human story-approval gate.**" and cite STORY-182 v2.12 /
STORY-183 v2.13 as awaiting the story-approval gate.

**Current truth (STATE.md Decisions Log, same file):**
- D-546 (2026-09-04): WAVE-86 HUMAN STORY-APPROVAL GATE PASSED
- D-548 (2026-09-05): STORY-182 DELIVERED (PR #460, `35ffa135`)
- D-549 (2026-09-05): STORY-183 DELIVERED (PR #462, `b273af21`)
- D-550 (2026-09-05): WAVE-86 GATE CLOSED + S-7.02 CYCLE-CLOSE COMPLETE
- D-551 (2026-09-05): v0.13.3 RELEASED (main `46ebd6e3`, back-merged to develop `0b1ea806`)

**Finding:** These 3 Drift Item rows were not updated as the wave progressed from
gate-approval → delivery → gate-close → release, even though 5 subsequent Decisions Log entries
supersede their "pending" language. This is exactly the class of self-contradicting Drift Item row
that D-544/D-545 previously caught and fixed for `PG-W84-LOCAL-BATCH`/`PG-W85-003`/`PG-W85-005`'s
predecessor state — it has recurred one delivery cycle later.

**Remediation (spec-steward/state-manager):** Update all 3 rows' "Summary" and "Target" columns to
state STORY-182/183 DELIVERED, wave-86 gate CLOSED, and v0.13.3 RELEASED; mark
`RESOLVED — archive at next compact` per the row style used for `DRIFT-BACKMERGE-SQUASH-001` and
similar closed items.

## SC-002 through SC-007 — CLEAN

- **SC-002 (index version-claim verification):** BC-INDEX `version: "2.37"` ✓, VP-INDEX
  `version: "2.47"` ✓, ARCH-INDEX `version: "2.20"` ✓, STORY-INDEX `version: "4.23"` ✓, epics.md
  `version: "2.3"` ✓, `stories/dependency-graph.md` `version: "3.12"` ✓ — all 6 claimed versions
  match actual frontmatter exactly, zero stale-index-version-claims. (`specs/architecture/dependency-graph.md`
  is a distinct architecture-section artifact at `version: "1.6"` — different scope, not in
  conflict with the stories-side dep-graph claim.)
- **SC-003 (VP↔BC / VP-INDEX self-consistency):** `total_vps: 47` = kani(16)+proptest(22)+fuzz(3)+
  integration_unit(6) = 47, and = p0(9)+p1(32)+test_sufficient(6) = 47. Both arithmetic checks pass.
- **SC-004 (BC coverage completeness):** epics.md v2.3 `total_bcs: 380` (active) reconciles exactly
  against BC-INDEX v2.37's canonical derivation line ("Total BCs on disk: 381. Active: 380.") — 0
  unassigned, 0 double-assigned, 0 residual gap, per the D-545/D-546 reconciliation burst.
- **SC-005 (L1→L4 chain integrity):** No `product-brief.md` exists anywhere under `.factory/specs/`
  — but this is a documented brownfield-project exception: `specs/domain/domain-spec.md`
  frontmatter explicitly states no L1 brief exists for this project and traces instead to the
  brownfield ingestion corpus (`traces_to: .factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`).
  BC-INDEX correctly traces to `prd.md`; `specs/domain-spec/{assumptions,risk-register}.md`
  `traces_to: ../domain/domain-spec.md` resolves correctly to `specs/domain/domain-spec.md`. Not a
  defect.
- **SC-006 (VP↔BC spot check / Story→BC mapping):** VP-047's `source_bc` set was correctly extended
  with BC-2.19.029/030 in the v2.47 modified-log entry (CV-008 RESOLVED, tied to STORY-180 delivery
  D-507) — new IEC-104 timed-command BCs propagated into the VP catalog without drift.
- **SC-007 (remaining open Drift Items, verified still accurate):** `STORY-INDEX-IN-INPUTS-CHURN`,
  `DRIFT-docstring-scan`, `DRIFT-e2e-sibling-harnesses`, `DRIFT-stale-red-scrub`,
  `DRIFT-py-surface-outside-bin`, `DRIFT-TOOLCHAIN-ROLL-CLIPPY`,
  `DRIFT-STORY183-INHERITED-PATTERN-DOC-COMMENTS` — all still open and their row text still
  accurately reflects current truth; no drift-of-the-drift-item found beyond SC-001.

---

## Scope Note

This sweep was **index-first and sample-based** per the run instructions (INDEX files loaded;
individual BC/VP/story detail files opened only where index inspection surfaced a question). It is
not a full 80-criterion consistency-validator pass — ARCH-INDEX structural completeness (Criterion
4/10) and exhaustive Story→BC reverse-coverage (Criteria 5, 27, 34) were spot-checked, not
exhaustively walked. No dangling index→file references were found in the files sampled.
