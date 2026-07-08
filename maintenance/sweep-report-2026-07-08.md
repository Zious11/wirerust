# Maintenance Sweep Report — maint-2026-07-08

**Run ID:** maint-2026-07-08
**Date:** 2026-07-08
**Prior run:** maint-2026-07-06
**develop HEAD at open:** b642c0f (v0.11.5 + 5 unreleased commits)
**develop HEAD at close:** c4eb1f4 (v0.11.5 + 8 unreleased commits, 3 PRs merged this run)
**Sweeps executed:** 1 (deps), 2 (doc-drift), 3 (patterns), 4 (holdouts), 5 (perf), 7 (spec-coherence), DF-VALIDATION-001 triage (11 items)
**Sweeps skipped:** Sweep 6 (DTU — dtu_required: false), Sweep 9 (a11y — CLI product, no UI)

---

## Sweep Verdicts Summary

| Sweep | Result | Finding Count | Notable |
|-------|--------|---------------|---------|
| 1 — Dependency Audit | **CLEAN** | 0 new; 2 pre-existing LOW (DEP-006/007, both deferred) | DB advisory count 1159 baseline recorded |
| 2 — Doc Drift | **5 findings** | 1 HIGH, 1 MED, 2 LOW, 1 INFO | NEW-001 ADR-012 missing → STORY-159 drafted |
| 3 — Pattern Consistency | **MEDIUM finding** | Clippy CLEAN (0 warnings); PF-001 MEDIUM; PF-002/003/004 LOW | PF-001: 109 plain `+=` sites → saturating_add (PR #384) |
| 4 — Holdout Freshness | **21/21 PASS** | 0 FAIL / 0 STALE | HS-INDEX-ENIP-WAVE-DRIFT-001 confirmed; deferred Route C |
| 5 — Performance | **NOISE-SUSPECT** | 0 actionable regressions | STORY-150 TLS drain refactor cleared (+1.8% within noise); controlled re-run recommended for non-TLS paths |
| 7 — Spec Coherence | **PASS** | 5 NEW findings (all LOW admin) | 3 stale backlog rows identified; STORY-INDEX-TALLY-DRIFT-001 + EPICS-STORY-COUNT-DRIFT-001 |
| DF-VALIDATION-001 Triage | 11 items | CONFIRMED×4, REFUTED×1, UNVERIFIABLE×1, ALREADY-RESOLVED×1, CLOSED-BY-DESIGN×1, NOT-DEFECT×1, CONFIRMED-RESIDUAL×2 | SEC-W71-001 CONFIRMED CWE-22 VALIDATED-PENDING-FILING |

---

## DF-VALIDATION-001 Triage — Verdict Table

| ID | Verdict | Disposition |
|----|---------|-------------|
| SEC-W71-001 | CONFIRMED (CWE-22, still open on b642c0f) | VALIDATED-PENDING-FILING — human deferred GitHub issue filing 2026-07-08 |
| REBIND-COUNT-SATURATING-001 | CONFIRMED (13 sibling +=  sites; u32 type corrected from stated u64) | RESOLVED — folded into PR #384 PF-001 sweep (c4eb1f4) |
| INPUT-HASH-ERROR-STORIES-001 | REFUTED (STORY-091/121 inputs:[] by design; STORY-001 hash MATCH) | CLOSED — backlog row removed; no GitHub issue |
| HS-INDEX-ENIP-WAVE-DRIFT-001 | CONFIRMED (waves 58-61, stories STORY-130..138) | DEFERRED Route C (human decision 2026-07-08) |
| EPICS-TOTAL-BCS-DRIFT-001 | CONFIRMED (delta 8; 6 named + 2 unresolved) | DEFERRED Route C (human decision 2026-07-08) |
| DNP3-CLOSEDFLOW-REOPEN-REUSE-001 | CONFIRMED observation / REFUTED as defect (spec-conformant BC-2.15.021 PC-4) | DEFERRED — no GitHub issue; optional docstring rename |
| CR-001 (wave-71) → CR-W71-001 | UNVERIFIABLE (no persisted finding detail) | CLOSED-UNVERIFIABLE — renamed CR-W71-001 to avoid collision with closed CR-001 (PR #177); PG-W71-CODEREVIEW-ARTIFACT codified in STORY-158 v1.1 AC-158-006 |
| STORY-148-BASIS-RESOLVED-001 | ALREADY-RESOLVED (D-399, 2026-07-07) | CLOSED — backlog row updated |
| SEC-010 | CONFIRMED (test/bench-only, zero `as u16` in src/) | RESOLVED — PR #383 (3ebd801): debug_assert guard added to wrap_as_tls_record |
| SEC-011 | CONFIRMED residual (Gap A closed at 5b41eca; Gap B anti-gameability enumeration hole) | RESOLVED — PR #383 (3ebd801): self.flows[ index-syntax check added to anti-gameability loop |
| SEC-W70-001 | CONFIRMED observation / NOT a defect (spec-designed BC-2.04.024 inv-4/AC-007b; BC-2.16.016 house precedent) | CLOSED-BY-DESIGN — no GitHub issue; symmetric BC-2.07.NNN PO backlog note |

---

## Fix Routes — Delivered PRs

### PR #382 (624bae3) — docs: Known Limitations + Operator-Boundary Sweep
- Closes **TD-MAINT-THRESHOLD-CALIB-001** as FORMALLY ACCEPTED (human decision 2026-07-08)
- Adds README "Known Limitations" section for uncalibrated detection-threshold defaults (ASM-CAND-003/009, R-CAND-006)
- Adversarial convergence: strict 3/3 consecutive-clean fresh-context passes
- Notable defects caught by convergence passes:
  - arp.rs rustdoc operator-boundary prose drift (adversary pass 2, not caught in pass 1)
  - README + `--help` ARP tuning rationale INVERTED (HIGH — stated "decrease threshold" for false positives, correct is "increase threshold"); fixed before merge

### PR #383 (3ebd801) — test: SEC-010 fixture guard + SEC-011 Gap B + NEW-004
- **SEC-010** RESOLVED: `debug_assert!(payload.len() <= u16::MAX as usize, "…")` added to `tests/common/tls_fragmented_fixture.rs:wrap_as_tls_record`; DF-SIBLING-SWEEP parity check performed on 6 test files (other `.len() as u16` sites documented in PR body)
- **SEC-011 Gap B** RESOLVED: `!body.contains("self.flows[")` assertion added to anti-gameability loop in `tests/bc_149_single_borrow_invariant_tests.rs`
- **NEW-004** RESOLVED: test count comment `tests/integration_tests.rs:1161` updated "All 20" → "All 22 tests pass"
- Adversarial convergence: strict 3/3 consecutive-clean fresh-context passes

### PR #384 (c4eb1f4) — refactor: PF-001 — 109 plain `+=` → `saturating_add`
- **PF-001** RESOLVED: 109 plain `+=` sites on diagnostic counters converted across `src/analyzer/dns.rs`, `src/analyzer/arp.rs` (5 sites incl. REBIND-COUNT-SATURATING-001), `src/analyzer/tls.rs`, `src/analyzer/enip.rs`, `src/analyzer/dnp3.rs`, `src/reassembly/lifecycle.rs`, `src/dispatcher.rs`, `src/main.rs`
- **REBIND-COUNT-SATURATING-001** RESOLVED: `entry.rebind_count += 1` (u32, arp.rs:856) included in sweep
- Intentional exclusion: `packet_index` sites (loop/cursor variables, not diagnostic counters) documented in PR body
- Finding count discrepancy resolved: sweep-report enumerated ~48 sites; re-run by fixer counted 109 actual sites (DF-SIBLING-SWEEP-001 vindicated — grep-derived counts in reports must be re-run by fixer)
- Adversarial convergence: strict 3/3 consecutive-clean fresh-context passes

---

## Route Deferrals (Human Decision 2026-07-08)

| Route | Content | Decision |
|-------|---------|----------|
| Route B | NEW-002 (README --coverage-gaps flag undocumented) + NEW-003 (ADR-0001 snippet missing STORY-153 fields) | DEFERRED — batch into next chore/docs PR |
| Route C | HS-INDEX v2.13 ENIP waves fix + epics.md/STORY-INDEX tally reconcile + BC/CWE-tag consistency (ADV-6) | DEFERRED — batch into next spec-coherence sweep |
| SEC-W71-001 issue | CWE-22 path-traversal hardening in bin/compute-input-hash | DEFERRED — human elected not to file immediately; VALIDATED-PENDING-FILING status retained |

---

## Story Actions

| Action | Story | Notes |
|--------|-------|-------|
| DRAFTED | STORY-159 | NEW-001 doc-drift: ADR-012 public doc authoring + CLAUDE.md Project References row; E-11, 3 pts; STORY-INDEX v3.24 |
| AMENDED v1.1 | STORY-158 | +AC-158-006: PG-W71-CODEREVIEW-ARTIFACT — wave gate code-review output MUST be written to cycles/wave-NNN/wave-gate/code-review.md; STORY-INDEX v3.25 |

---

## Deferred New Register Items (maint-2026-07-08)

| ID | Description | Severity | DF-VALIDATION-001-gated? |
|----|-------------|----------|--------------------------|
| SEC-010-PARITY-SWEEP-001 | File-wide bare `.len() as u16/u32` parity sweep: tls_fragmented fixture + 5 other test files; ~16 pre-existing sites; PR #383 addressed wrap_as_tls_record only | LOW | No (parity sweep is optional hardening) |
| CHANGELOG-D3-T0830-DRIFT-001 | CHANGELOG ~670 says D3 "Attributed to T0830" vs code `mitre_techniques:[]` — adversary OBS | LOW | Yes |
| HASHMAP-ENTRY-SATURATING-001 | ~15 `entry().or_insert(0) += 1` sites across analyzers — HashMap entry idiom that bypasses saturating_add; optional chore | LOW | Yes |
| ARP-RATE-INTDIV-DOC-001 | Integer-division window-rate comprehension note in arp.rs doc-comments | LOW | No (doc polish) |
| DNP3-TUNING-BIDIR-001 | DNP3 threshold-tuning guidance references bidirectional flow assumptions not stated in README | LOW | No (doc polish) |
| UNIT-FMT-5-20S-001 | Unit formatting inconsistency: some `--help` output uses "5s" vs "20 sec" style | LOW | No (doc polish) |
| README-OPTIONS-L117-NEUTRAL-001 | README options section line 117 uses value-laden phrasing where neutral description preferred | LOW | No (doc polish) |
| ROUTE-B-DEFERRED | README --coverage-gaps flag undocumented (NEW-002) + ADR-0001 STORY-153 fields absent (NEW-003) | LOW | No |
| ROUTE-C-DEFERRED | HS-INDEX ENIP waves fix + epics.md/STORY-INDEX BC tally + BC-2.16.016 narrative reconcile | LOW | Yes (for any GitHub issue) |
| PERF-RERUN-001 | Controlled benchmark re-run required: 2026-07-08 run noise-suspect (11-20 severe outliers/100 samples); cannot use as regression anchor | LOW | No |

---

## Performance Baseline — 2026-07-08 Note

All 7 pipeline benchmarks showed 11–20 severe outliers per 100 samples. This is the same machine-noise fingerprint as the June-17 run (later confirmed thermal interference in the June-22 controlled re-run). The STORY-150 TLS drain-loop regression check shows +1.8% vs STORY-149 pre-story anchor (p=0.37, no change detected). The long-run reassembly/tls.pcap trend: 26.353 µs (+13.2% vs May-19 anchor 23.281 µs, consistent with prior sweeps). **CRITICAL/WARNING flags on non-TLS paths MUST be treated as NOISE-SUSPECT pending a controlled re-run.** See PERF-RERUN-001.

---

## Summary

| Metric | Value |
|--------|-------|
| Findings identified (all sweeps) | 30+ (most pre-existing or LOW admin) |
| Actionable findings resolved this run | 4 (TD-MAINT-THRESHOLD-CALIB-001, SEC-010, SEC-011, PF-001+REBIND) |
| PRs merged | 3 (#382, #383, #384) |
| Adversarial passes used | ~13 (strict 3/3 gate per PR) |
| Stories drafted/amended | 2 (STORY-159 NEW, STORY-158 v1.1 amended) |
| Open deferred items added to register | 10 |
| Blocking issues at close | 0 |
| develop HEAD at close | c4eb1f4 (8 unreleased commits since v0.11.5) |
