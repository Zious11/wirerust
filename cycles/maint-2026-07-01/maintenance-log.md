---
document_type: maintenance-log
run_id: maint-2026-07-01
started: "2026-07-01"
completed: "2026-07-01"
mode: maintenance
status: complete
develop_head: 3a60317965e62bef9895e857c8a26fc3b8d03ad0
develop_head_at_start: ba6fbd85846a7665516d6222715f4de924aaa8e5
prior_run: maint-2026-06-22
version_at_start: v0.11.1
sweeps_planned:
  - dependency-supply-chain
  - code-quality-pattern
  - doc-comment-drift
  - spec-anchor-drift
  - security
  - performance
sweeps_skipped:
  - ui-design-drift
skip_reason_ui_design_drift: "No UI component — wirerust is a CLI tool only"
producer: state-manager (bootstrap)
---

# Maintenance Log — maint-2026-07-01

**Run date:** 2026-07-01
**Scope:** 6 parallel sweeps (UI/design-drift skipped — non-UI CLI tool)
**Product version audited:** wirerust v0.11.1 (develop @ ba6fbd8)
**Prior run:** maint-2026-06-22 (COMPLETE 2026-06-23; 38 observations; 0 blocking)

---

## Context

Cycle `fix-tls-clienthello-frag` CLOSED 2026-07-01 (D-316). v0.11.1 released (PR #347 main
merge, PR #348 back-merge). D-303 maintenance pause LIFTED. This run picks up where
maint-2026-06-22 left off plus new work introduced in the TLS fragmentation feature cycle
(PRs #341/#343/#344/#345/#346/#347/#348, stories STORY-144..147).

Key additions since maint-2026-06-22 (v0.9.3 @ dd3b069) → now (v0.11.1 @ ba6fbd8):
- TLS handshake reassembly across records (src/analyzer/tls.rs, major expansion)
- Kani formal proofs (VP-039, 3 harnesses — src/kani_proofs/)
- mod f6_hardening (12 mutation-gap tests)
- anyhow bump 1.0.102 → 1.0.103 (RUSTSEC-2026-0190 cleared)
- 6 new BCs + 3 amended; 2 new VPs (VP-039, VP-040); ADR-011

Open carry-forwards from maint-2026-06-22 that are sweep targets this run:
- BC-ANCHOR-DRIFT-OUTOFCYCLE-001 (stale tls.rs anchors: BC-2.07.004:124, BC-2.07.028:109, STORY-054:127)
- PG-BC-ANCHOR-VALIDATION-001 (no automated line-anchor validation)
- PC-013 (4 `.expect()` calls in arp.rs — MEDIUM)
- PC-014 (DNP3 telemetry key naming divergence — LOW)

---

## Sweep Categories

### Sweep 1 — Dependency / Supply-Chain

**Agent:** dx-engineer (scan) + security-reviewer (analysis)
**Status:** PENDING

Scope:
- `cargo audit` — check for new RustSec advisories since anyhow bump (RUSTSEC-2026-0190 cleared in F6; verify no new advisories introduced)
- `cargo deny check` — license compliance, bans, advisories
- `cargo outdated` — identify minor/patch updates available
- Supply-chain integrity: verify SHA-pinned GitHub Actions in ci.yml are still valid (no new Dependabot alerts)
- Check: any new Dependabot PRs opened since PR #325/#311 (merged 2026-06-29)?

**Findings:** (to be populated after sweep)

---

### Sweep 2 — Security

**Agent:** security-reviewer
**Status:** PENDING

Scope (incremental — new surface since maint-2026-06-22):
- Review Kani proof harnesses (src/kani_proofs/) for unsound assumptions
- Review mod f6_hardening boundary conditions (MAX_BUF exact-pin, clear-and-recover patterns)
- TLS reassembly buffer arithmetic: integer overflow / underflow windows
- `fill_buf_for_testing` public seam (TLS-FILLBUF-PUBLIC-SEAM-001 — W7.1 backlog item)
- SEC-001-ENIP (unsafe split-borrow enip.rs on_data) — status check, still open?
- Carry-forward: SEC-002/SEC-006 — closed-by-design per F6; confirm no regression

**Findings:** (to be populated after sweep)

---

### Sweep 3 — Code Quality / Pattern

**Agent:** code-reviewer + consistency-validator
**Status:** PENDING

Scope:
- Pattern carry-forwards: PC-013 (arp.rs `.expect()` calls), PC-014 (DNP3 telemetry key), PC-015 (ARP findings cap)
- New code from TLS feature: tls.rs reassembly patterns, error handling consistency
- mod f6_hardening: style, naming, coverage conventions
- Kani harness code quality (src/kani_proofs/)
- Check for new TODO/FIXME/HACK comments added during TLS feature cycle
- Pattern consistency: does new tls.rs code match existing module conventions?

**Findings:** (to be populated after sweep)

---

### Sweep 4 — Doc / Comment Drift

**Agent:** technical-writer + consistency-validator
**Status:** PENDING

Scope:
- CLAUDE.md — verify Project References table reflects ADR-011 (TLS reassembly decisions)
- docs/adr/ — ADR-009 was referenced in maint-2026-06-22; authored status?
  ADR-011 was authored this cycle (TLS ClientHello/ServerHello fragmentation reassembly);
  verify it exists and is findable
- README.md — does it reflect TLS fragmentation detection capability added in v0.11.1?
- src/analyzer/tls.rs — comment accuracy after large reassembly expansion
- STORY-054 stale anchor (tls.rs:127) — part of BC-ANCHOR-DRIFT-OUTOFCYCLE-001
- Carry-forward: DOC-002 (ADR-009 authoring) — was it resolved in prior run or still open?

**Findings:** (to be populated after sweep)

---

### Sweep 5 — Spec / Anchor Drift

**Agent:** consistency-validator + spec-steward
**Status:** PENDING

Scope:
- BC-ANCHOR-DRIFT-OUTOFCYCLE-001 (stale tls.rs anchors: BC-2.07.004:124, BC-2.07.028:109, STORY-054:127)
- PG-BC-ANCHOR-VALIDATION-001 — assess feasibility of automated symbol-line resolver or symbol-only-anchor policy
- BC-INDEX v2.3 / VP-INDEX v2.28 — verify all 6 new BCs + 3 amended BCs have correct anchors after tls.rs expansion
- Story-to-BC mapping: STORY-144..147 — spot-check traceability
- HS-INDEX: any new stale scenarios from TLS protocol changes?
- Spec coherence (33 criteria, DF-030): run representative subset against v0.11.1 state

**Findings:** (to be populated after sweep)

---

### Sweep 6 — Performance

**Agent:** performance-engineer
**Status:** PENDING

Scope:
- Re-run criterion benchmarks against v0.11.1 baseline
- Focus: TLS reassembly path (new multi-record reassembly may affect throughput on fragmented captures)
- Compare against maint-2026-06-22 baseline: `decode/tls.pcap` (+12.2%), `reassembly/segmented.pcap` (+19.4%)
- Are prior REGRESSION-MINOR items still stable or worsening?
- New benchmark candidate: fragmented TLS handshake path throughput

**Findings:** (to be populated after sweep)

---

### SKIPPED — UI / Design Drift

**Reason:** wirerust is a CLI tool only. No UI component. Sweep 9 (accessibility) and Sweep 10 (design drift) do not apply. Consistent with maint-2026-06-22 skip of Sweeps 6 (DTU) and 9 (accessibility/UI).

---

## Findings Summary

All 6 sweeps complete. develop advanced ba6fbd8 → 3a60317 (PRs #349 + #350 merged).

| Sweep | Status | CRITICAL | HIGH | MEDIUM | LOW | FAIL-BUG |
|-------|--------|---------|------|--------|-----|---------|
| 1 — Dependency/Supply-Chain | COMPLETE | 0 | 0 | 0 | 1 | 0 |
| 2 — Security | COMPLETE | 0 | 0 | 2 | 2 | 0 |
| 3 — Code Quality/Pattern | COMPLETE | 0 | 0 | 1 | 4 | 0 |
| 4 — Doc/Comment Drift | COMPLETE | 0 | 2 | 0 | 0 | 0 |
| 5 — Spec/Anchor Drift | COMPLETE | 0 | 0 | 0 | 3 | 0 |
| 6 — Performance | COMPLETE | 0 | 1 | 0 | 0 | 0 |
| **Total** | **COMPLETE** | **0** | **3** | **3** | **10** | **0** |

---

## Findings Detail and Dispositions

### RESOLVED this sweep — Merged to develop

#### Sweep 4 — Doc / Comment Drift

**DOC-README-ENIP-001** (HIGH) — README.md did not reflect EtherNet/IP (ENIP/CIP) analyzer
capability added in v0.11.0.
**Disposition:** RESOLVED → PR #350 (docs refresh). develop=3a60317.

**DOC-README-TLS-REASSM-001** (HIGH) — README.md did not reflect TLS handshake-message
reassembly across records (fragmentation detection) added in v0.11.1.
**Disposition:** RESOLVED → PR #350 (docs refresh). develop=3a60317.

**ADR-011-PLACEMENT-001** — ADR-011 (TLS ClientHello/ServerHello fragmentation reassembly
decisions) was authored during feature cycle but not promoted to docs/adr/.
CLAUDE.md Project References table also missing 0010 + 0011 entries.
**Disposition:** RESOLVED → PR #350 (ADR-011 promoted to docs/adr/0011; CLAUDE.md ADR list
updated to include 0010 + 0011). Note: CVE-2021-25742 misattribution caught in review and
removed from ADR-011 before merge.

#### Sweep 3 — Code Quality / Pattern

**TLS-STALE-COMMENT-001a** — Stale RED-tense / todo!() comment in tls.rs carry path (C2S arm).
**Disposition:** RESOLVED → PR #349 (stale-comment cleanup). develop=b451c481 (squash).

**TLS-STALE-COMMENT-001b** — Stale RED-tense / todo!() comment in tls.rs carry path (S2C arm).
**Disposition:** RESOLVED → PR #349.

**TLS-SILENT-COMMENT-001** — Silent `//` comment suppressing an assertion placeholder in tls.rs.
**Disposition:** RESOLVED → PR #349.

**TLS-FRAMEC-TEST-DOC-001** — Stale test documentation comment in mod f6_hardening referencing
intermediate RED-gate state.
**Disposition:** RESOLVED → PR #349.

**ENIP-TODO-FUZZ F-P9-002** — todo!() fuzz placeholder in enip.rs (residual from F-P9 adversarial
pass).
**Disposition:** RESOLVED → PR #349.

**ARP-PC015-REDGATE** — Stale RED-gate comment in arp.rs (PC-015 carry-forward from prior cycle).
**Disposition:** RESOLVED → PR #349.

**MODBUS-RED-GATE** — Stale RED-gate comment in modbus.rs.
**Disposition:** RESOLVED → PR #349.

**TLS-LINE-REF** — Stale line-number reference comment in tls.rs (tls.rs:NNN cross-reference
now stale after reassembly expansion).
**Disposition:** RESOLVED → PR #349.

**TLS-FRAMEC-TEST-DOC-001** — Second stale test doc comment instance (F-P9-002 overlap).
**Disposition:** RESOLVED → PR #349. (Total: 9 stale comments cleared in PR #349.)

---

### ROUTED TO FOLLOW-UP STORIES

**SEC-005** (MEDIUM, CWE-400) — `StreamDispatcher::on_flow_close` in src/dispatcher.rs
(~lines 409–414) contains a no-op arm for the ENIP analyzer: the close event is never
forwarded to `EnipAnalyzer::on_flow_close`. `EnipAnalyzer.flows` (enip.rs ~line 782,
`.entry().or_default()`) grows monotonically — every distinct port-44818 flow inserts an
entry that is never removed. A crafted pcap with many short-lived ENIP flows exhausts heap
memory (DoS). Additionally the dead aggregation state (session stats, counters) degrades
summary correctness since STORY-138. Real bug.
**Disposition:** → STORY-148 (E-20, wave TBD, 5 pts). SEC-005 + SEC-006 grouped.

**SEC-006** (MEDIUM, CWE-400) — DNP3 flow map in src/analyzer/dnp3.rs has no cap on the
`flows` HashMap: a crafted pcap with many unique source IPs accumulates unbounded per-flow
state. Unlike the ENIP case on_flow_close IS wired for DNP3, but the map grows without bound
between close events. Add a cap (e.g., MAX_FLOWS const) and eviction policy consistent with
the existing ENIP and TLS patterns.
**Disposition:** → STORY-148 (grouped with SEC-005). E-20, wave TBD, 5 pts.

**PERF-001** (HIGH) — Repeated HashMap lookups in `try_parse_records` (tls.rs): the current
implementation acquires `flows.get()` / `flows.get_mut()` multiple times per record in a
way that prevents the entry API from being used, forcing two hash probes per path. On
fragmented captures with many flows this is the dominant hotspot. The `reassembly/tls.pcap`
Criterion benchmark crossed +10.3% vs. the May-19 baseline.
**Disposition:** → STORY-149 (E-11, wave TBD, 5 pts).

**PERF-002** (HIGH, grouped with PERF-001) — Unnecessary `Vec` allocations in the carry-drain
path of `try_parse_records`: intermediate drain results are collected into a `Vec` before
dispatch, adding allocator pressure on every fragmented handshake record.
**Disposition:** → STORY-149.

**PERF-003/004/005** (LOW, grouped) — Minor alloc hotspots: hex string formatting in the JA3
path, cipher suite string allocations, and SNI buffer copies. Not causing the regression but
candidates for a combined tidy pass inside STORY-149.
**Disposition:** → STORY-149.

**BENCHMARK-GAP-001** (LOW) — No Criterion benchmark fixture for a multi-record
fragmented-handshake pcap. The existing `reassembly/tls.pcap` exercises the reassembly path
but does not isolate the fragmented-handshake scenario specifically, making regression
detection imprecise.
**Disposition:** → STORY-149 (add fixture as part of performance recovery story).

**TLS-DRAIN-DUP-001** (MEDIUM) — The C2S and S2C carry-drain arms in `try_parse_records`
(tls.rs) are ~220 lines of symmetric duplication. Structurally identical except for the
carry buffer field, overflow counter, and dispatch target. This duplication makes each future
tls.rs change twice as expensive and was identified as a maintainability risk by the
adversarial review (F5 pass-2).
**Disposition:** → STORY-150 (E-11, wave TBD, 5 pts). TLS-DRAIN-DUP-001. Includes mandatory
Kani VP-039 re-run and mutation re-run to confirm no behavioral regression from the refactor.

---

### DEFERRED — Tracked backlog with exact corrections captured

**BC-ANCHOR-DRIFT-OUTOFCYCLE-001** (LOW, expanded this sweep) — 12 stale line-anchor sites
confirmed in tls.rs / spec files. Exact corrections:
- BC-2.07.004: tls.rs:319→:339; tls.rs:689-699→:731-741
- BC-2.07.028: tls.rs:379-383→:421; tls.rs:421-427→:455-469; tls.rs:435-515→:477-558; tls.rs:413-515→~:455-558
- STORY-054: tls.rs:497-517→:563-598; tls.rs:570-582→:656-669; tls.rs:519-539→:600-621; tls.rs:584-604→~:672-691; Tasks-table line 208
All 12 sites are in one file and are catchable by an automated symbol-line validator.
**Target:** next maintenance sweep, or fold into STORY-150 (which touches tls.rs).
**Disposition:** DEFERRED — exact fixes captured above; user deselected for this sweep.

**ARCH-INDEX-COUNT-DRIFT-001** (LOW, new this sweep) — ARCH-INDEX SS-11 BC count states 34 but
should be 35 (BC-2.11.035 added by issue-#64 STORY-129). SS-16 states 15 but should be 16
(BC-2.16.016 added). ARCH-INDEX SS-sum of 334 should be 336. No functional impact.
**Target:** next maintenance sweep.
**Disposition:** DEFERRED.

**TLS-SUMMARIZE-MAPTYPE-001** (LOW) — BC-2.07.043 PC-4 specifies HashMap<String,Value> for
`summarize()` output but the implementation uses BTreeMap<String,serde_json::Value>
(per LESSON-P2.09 spec-to-impl feedback from F5). Also affects Canonical Test Vectors wording
and VP-040 Sub-D assertion. No behavioral bug — BTreeMap satisfies all contracts; this is
a spec-wording gap.
**Target:** keep as backlog.
**Disposition:** DEFERRED.

**SEC-004** (LOW, CWE-190) + **SEC-007** (LOW) — 7+ parse_errors/counter `+= 1` sites should
use `saturating_add` for correctness under pathological inputs. Also minor clippy hygiene:
MQ-003 (S2C match_same_arms), MQ-004 (SNI match arms), MQ-005 (reader.rs ts_usecs
debug_assert). Grouped as low-priority "counter-saturation + minor clippy hygiene" backlog
item / candidate trivial PR.
**Target:** next maintenance sweep or standalone trivial PR.
**Disposition:** DEFERRED.

**SEC-001-ENIP** (LOW-as-implemented) — Unsafe split-borrow in enip.rs `on_data`
(pre-existing, v0.12.0 candidate). Sound but fragile pattern.
**Target:** v0.12.0.
**Disposition:** DEFERRED — confirmed still open.

**MAINT-SC-001** (LOW) — indicatif 0.18.4→0.18.6 patch update available; 41 transitive
dependency updates available as a safe `cargo update` batch (no semver-breaking changes).
cargo-outdated and cargo-udeps are not installed (blind spots on major-version drift and
unused deps). deny.toml has 8 stale license allowlist entries.
**Target:** optional dep-refresh / next maintenance sweep.
**Disposition:** DEFERRED.

---

### Process-Gap Reinforcement

**PG-BC-ANCHOR-VALIDATION-001** (S-7.02) — This sweep CONFIRMED 12 stale anchor sites in one
file (tls.rs), all catchable by an automated symbol-line validator (STORY-091). Strengthens
the tooling case. The exact correction list above (BC-ANCHOR-DRIFT-OUTOFCYCLE-001) is the
second confirmation of this gap.

**IDX-003** (RESOLVED this sweep) — STORY-121's 3 pts were never added to the frontmatter
`total_points` counter when v2.0 of STORY-INDEX was created (D-103). STORY-INDEX v3.10
frontmatter now corrected: 656→659. Reconciled in maint-2026-07-01.

---

## Fix PR Candidates

| ID | PR | Merged | Squash SHA | Scope |
|----|-----|--------|-----------|-------|
| Stale-comment cleanup | #349 | YES | b451c481 | 9 stale RED-tense/todo!() comments cleared |
| Docs refresh + ADR-011 | #350 | YES | 3a60317 | README ENIP+TLS-reassembly sections; ADR-011 promoted; CLAUDE.md ADR list 0010+0011 |

develop HEAD after merges: `3a60317965e62bef9895e857c8a26fc3b8d03ad0`

---

## Gate Result

**COMPLETE — 2026-07-01**

- 0 CRITICAL CVEs remaining (cargo deny PASS; cargo audit clean).
- All auto-fix PRs merged (#349, #350). develop=3a60317.
- All 6 sweeps complete.
- 3 follow-up stories drafted (STORY-148/149/150).
- Backlog deferred items documented with exact corrections.
- IDX-003 total_points reconciled (656→659).
- STORY-INDEX v3.10, total_stories 103.
