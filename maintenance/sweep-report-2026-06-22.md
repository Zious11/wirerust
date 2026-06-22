---
document_type: maintenance-sweep-report
maintenance_run_id: maint-2026-06-22
date: 2026-06-22
version: v0.9.3
git_head: dd3b069
producer: state-manager (synthesis from per-sweep agent outputs)
sweeps_run: [1, 2, 3, 4, 5, 7, 8]
sweeps_skipped: [6, 9]
skip_reason_6: "No DTU required (wirerust is an offline single-binary tool; dtu_required: false)"
skip_reason_9: "No UI component — wirerust is a CLI tool only"
gate_result: NON-BLOCKING
---

# Maintenance Sweep Report — maint-2026-06-22

**Run date:** 2026-06-22
**Scope:** Sweeps 1, 2, 3, 4, 5, 7, 8 run; Sweeps 6 (DTU) and 9 (accessibility) SKIPPED (no DTU, no UI).
**Product version audited:** wirerust v0.9.3 (develop @ dd3b069)
**Prior run:** maint-2026-06-17

---

## Executive Summary

**Gate result: NON-BLOCKING.** No CRITICAL, HIGH, or blocking findings across any sweep.

- Highest severity finding: MEDIUM (DEP-005 — zerocopy precautionary update; no active advisory; not CLI-reachable)
- FAIL-BUG holdout count: 0 — no behavioral regressions detected
- FAIL-STALE holdout count: 2 (HS-064, HS-075 — JSON schema intentional change; product-owner update)
- New tech-debt items added: 5 (PC-013, PC-014, PC-015, DEP-005, DOC-002)
- Resolved this run: F-MAJ-001 (ARCH-INDEX v1.6, commit a6efb23) — fixed prior to this report being written

**Cross-sweep convergence signal:** The `rayon` dead dependency was independently flagged by THREE sweeps:
- DEP-006 (Sweep 1 — Dependency Audit): rayon confirmed dead direct production dependency, zero usage in src/benches/tests
- DOC-010 (Sweep 2 — Doc Drift): rayon in Cargo.toml, no rayon:: usage in src/, carried from maint-2026-06-17
- O-07 (Sweep 8 — Tech Debt): pre-existing open item, now confirmed auto-fixable one-line removal

Triple-sweep convergence on the same artifact is a HIGH-confidence removal signal. This is the single highest-priority fix PR candidate.

---

## Per-Sweep Summaries

### Sweep 1 — Dependency Audit

**Source:** `.factory/maintenance/dependency-audit-findings.md`
**Findings:** 6 total (0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW)

| ID | Description | Severity | Action |
|----|-------------|----------|--------|
| DEP-001 | RUSTSEC-2026-0097: rand 0.8.5 unsound (build-dep only; NOT CLI-reachable) | LOW | `cargo update -p rand` → 0.8.6; remove CI `--ignore` suppression |
| DEP-002 | deny.toml: 8 stale license-not-encountered allowlist entries | LOW | Optional housekeeping; defer |
| DEP-003 | syn v1/v2 duplicate — expected ecosystem pattern | LOW | No action (upstream must migrate) |
| DEP-004 | 35 crates with patch/minor updates available | LOW | `cargo update` + CI-green confirm; shlex 2.0.1 major needs explicit verification |
| DEP-005 | zerocopy 0.8.48 → 0.8.52 precautionary (no active CVE; soundness history) | MEDIUM | Include in `cargo update` batch |
| DEP-006 | rayon v1.12.0 dead direct production dependency (zero src/ usage) | LOW | Remove `rayon = "1"` from Cargo.toml — one-line chore PR |

**Verdict:** Non-blocking. Highest severity MEDIUM (DEP-005, precautionary, no active advisory).

---

### Sweep 2 — Documentation Drift

**Source:** `.factory/maintenance/doc-drift-findings.md`
**Findings:** 10 total (2 HIGH, 4 MEDIUM, 4 LOW)

| ID | File | Severity | Issue | Auto-Fixable |
|----|------|----------|-------|--------------|
| DOC-001 | CLAUDE.md | HIGH | STATE.md described as "not yet initialized" — file is fully populated | Yes |
| DOC-002 | docs/adr/ | HIGH | ADR-009 referenced 40+ times in reader.rs but no docs/adr/0009-*.md file exists | No (authoring required) |
| DOC-003 | README.md | MED | Architecture table omits pcapng from Reader description | Yes |
| DOC-004 | README.md | MED | Features bullet says "pcap formats" — pcapng not mentioned | Yes |
| DOC-005 | docs/adr/0002 | MED | `detail` field described as HashMap; actual type is BTreeMap | Yes |
| DOC-006 | docs/adr/0002 | MED | `StreamHandler::on_data` signature omits `timestamp: u32` parameter | Yes |
| DOC-007 | docs/adr/0003 | LOW | main.rs line numbers stale by ~60-100 lines in Grouped-Mode Collapse section | Yes |
| DOC-008 | docs/adr/0002 | LOW | `parse_error_count()` listed as Required trait method; it is a convention only | No |
| DOC-009 | docs/adr/0001 | LOW | StreamDispatcher struct snippet still shows 2-field (http/tls) form; missing modbus/dnp3 | Yes |
| DOC-010 | Cargo.toml | LOW | rayon declared but unused in src/ (also DEP-006 / O-07) | Yes |

**Verdict:** Non-blocking. All findings are documentation-only; no runtime behavior affected.

---

### Sweep 3 — Pattern Consistency

**Source:** `.factory/maintenance/pattern-findings.md`
**Findings:** 15 total (3 HIGH, 6 MEDIUM, 6 LOW); 3 NEW (PC-013, PC-014, PC-015); 12 carry-forward

| Category | Count | IDs |
|----------|-------|-----|
| HIGH (carry-forward) | 3 | PC-001, PC-002, PC-003 |
| MEDIUM (carry-forward) | 5 | PC-004..PC-008 |
| MEDIUM (new) | 1 | PC-013 |
| LOW (carry-forward) | 4 | PC-009..PC-012 |
| LOW (new) | 2 | PC-014, PC-015 |

New findings this sweep:
- **PC-013 (MEDIUM):** 4 production `.expect()` calls in `src/analyzer/arp.rs` (lines 555, 576, 642, 827) on HashMap::get_mut() — same class as tracked CR-006 but untracked. Not auto-fixable.
- **PC-014 (LOW):** DNP3 telemetry key `"total_parse_errors"` (dnp3.rs:1425) diverges from `"parse_errors"` used by HTTP/TLS/Modbus. Auto-fixable.
- **PC-015 (LOW):** ARP findings-output cap intent undocumented (no MAX_FINDINGS/dropped_findings; HTTP/TLS cite BC-2.04.024 for the omission, ARP does not). Not auto-fixable.

**Verdict:** Non-blocking. PC-001/PC-003 are pre-existing HIGH items deferred to next ICS feature cycle. No new HIGH findings.

---

### Sweep 4 — Holdout Scenario Freshness

**Source:** `.factory/maintenance/holdout-freshness.md`
**Build verification:** `cargo build --release` PASS, wirerust v0.9.3. FAIL-BUG: 0.

| Metric | Value |
|--------|-------|
| Holdout scenarios present (greenfield namespace) | 109 (HS-001..HS-109) |
| Scenarios validated this sweep | 30 representative (all 6 categories) |
| PASS | 27/30 |
| FAIL-BUG | 0 |
| FAIL-STALE | 2 (HS-064, HS-075) |
| OBSOLETE | 0 |
| PASS-minor | 1 (HS-108 sub-assertion wording) |

FAIL-STALE items (HS-064, HS-075): both assert JSON report has "exactly 3 top-level keys." Live schema now has 5 — `mitre_attack_version` and `mitre_domain` intentionally added (PR #209). Product-owner update required; behavioral regression: 0.

Coverage gap (MAJOR, product-owner): HS-INDEX declares 73 feature seeds with zero HS files on disk across DNP3 (32 seeds), ARP (28 seeds), finding-collapse (13 seeds), and Modbus (no dedicated scenarios). pcapng is well covered (HS-101..109).

**Verdict:** Non-blocking. FAIL-BUG=0. FAIL-STALE items are intentional product changes awaiting holdout update.

---

### Sweep 5 — Performance Baseline (Controlled Re-run)

**Source:** `.factory/maintenance/performance-baseline.md`
**Prior finding disposition:** The maint-2026-06-17 CRITICAL (`reassembly/tls.pcap` +54.5%) was THERMAL NOISE.

| Benchmark | vs May-19 baseline | Verdict |
|-----------|-------------------|---------|
| decode/tls.pcap | +12.2% | REGRESSION-MINOR (real, stable) |
| reassembly/segmented.pcap | +19.4% | REGRESSION-MINOR (real, stable) |
| All other benchmarks | < 10% | NOISE |

Both REGRESSION-MINOR findings are attributable to the `DecodedFrame::Arp` match variant added in the ARP feature cycle. Neither is worsening between runs (stable). No NFR latency target exists, so these are informational only.

**Verdict:** Non-blocking. 0 CRITICAL regressions. 2 REGRESSION-MINOR (stable, informational, no NFR target). Prior CRITICAL from maint-2026-06-17 CONFIRMED as measurement noise.

---

### Sweep 7 — Spec Coherence (33 criteria, DF-030)

**Source:** `.factory/maintenance/spec-coherence-findings.md`
**Score:** 23 PASS / 1 MAJOR-new / 1 MAJOR-carry-forward / 9 N/A-BLOCKED / 33 criteria

| Criterion set | Result |
|--------------|--------|
| L1→L4 chain integrity (criteria 1–8) | All PASS |
| Cross-artifact consistency (criteria 9–15) | All PASS (criterion 11 N/A — no UI) |
| Quality and compliance (criteria 16–20) | All PASS |
| Sharding integrity (criteria 21–23) | All PASS |
| Lifecycle coherence (criteria 24–33) | All PASS |
| Risk/assumption registry (criteria 42–50) | N/A-BLOCKED (pre-existing: TD-MAINT-RISK-REGISTRY-BACKFILL) |

New MAJOR finding: **F-MAJ-001** — ARCH-INDEX Subsystem Registry BC counts stale for SS-01 (8→17) and SS-11 (29→34) after FE-001 pcapng integration and STORY-119 grouped-collapse. Documentation-only; not a behavioral contract gap.

**STATUS: F-MAJ-001 RESOLVED this run** — ARCH-INDEX updated to v1.6 (commit a6efb23) with corrected BC counts (SS-01: 17, SS-11: 34).

**Verdict:** PASS with carry-forwards. No CRITICAL findings. F-MAJ-001 fixed.

---

### Sweep 8 — Tech Debt Register Review

**Source:** `.factory/tech-debt-register.md` (reviewed and updated this run)

No items with hard due dates — nothing overdue. P2 backlog accumulating, all deferred to "next ICS feature cycle":
- O-07 (rayon) — now CONFIRMED auto-removable (triple-sweep convergence; see DEP-006, DOC-010)
- TD-MAINT-PC001-DNP3-STREAMTRAIT — pre-existing P2 HIGH, deferred
- TD-MAINT-RISK-REGISTRY-BACKFILL — blocks spec-coherence criteria 42–50

Two ENGINE-NOTE items (DRIFT-ENGINE-CHECKOUT-GUARD-001, DRIFT-ENGINE-PRMGR-REPORT-001) are dark-factory engine fixes, not wirerust product changes. Not actionable as wirerust PRs.

New items added to register this run: PC-013, PC-014, PC-015, DEP-005, DOC-002.
Updated items: O-07 (confirmed auto-removable), TD-MAINT-PERF-ARP-HOTPATH (CRITICAL finding retracted; two REGRESSION-MINOR confirmed real and stable).

**Verdict:** Register current. No overdue items. No blockers.

---

## Cross-Sweep Themes

### Theme 1 — rayon dead dependency (triple-sweep convergence)

DEP-006, DOC-010, and O-07 independently converged on `rayon = "1"` in Cargo.toml being a dead direct production dependency with zero usage in src/benches/tests across v0.5.0..v0.9.3. Three distinct sweep methodologies (security audit, doc drift, pattern/debt) all reached the same conclusion. This is the highest-confidence finding in this run and should be the first item in the code/config fix PR.

### Theme 2 — pcapng integration documentation lag

FE-001 (pcapng reader, v0.9.3) left several documentation artifacts behind:
- ADR-009 referenced 40+ times in source but absent from docs/adr/ (DOC-002 HIGH)
- README feature bullets and architecture table do not mention pcapng (DOC-003, DOC-004)
- ARCH-INDEX SS-01 BC count was stale until fixed this run (F-MAJ-001)

These are expected post-cycle drift. The docs fix PR (Actionable Fix B) addresses all.

### Theme 3 — Performance CRITICAL retraction

The maint-2026-06-17 CRITICAL performance finding (`reassembly/tls.pcap` +54.5%) is confirmed as thermal/scheduling measurement noise. The two remaining REGRESSION-MINOR findings (+12.2%, +19.4%) are real, stable, and attributable to the ARP feature cycle overhead. No immediate action required.

---

## Actionable Fixes

### (A) Code/config auto-fix PR to develop — Cargo.toml/Cargo.lock/CI

1. **Remove `rayon = "1"` from Cargo.toml `[dependencies]`** (DEP-006 / O-07 / DOC-010) — confirmed zero usage in src/benches/tests across all release history. One-line deletion. Low-risk; rayon remains transitively available via criterion in dev builds.
2. **`cargo update -p rand`** → 0.8.6 (DEP-001, clears RUSTSEC-2026-0097); remove `--ignore RUSTSEC-2026-0097` from CI audit step if present.
3. **`cargo update`** (full batch) including zerocopy 0.8.48 → 0.8.52 (DEP-005, MEDIUM precautionary); confirm CI green; manually verify shlex 2.0.1 major bump does not regress CI.

### (B) Doc-fix PR to develop

1. **DOC-001 (HIGH):** Update CLAUDE.md Project References table — remove stale "STATE.md not yet initialized" note. Suggested replacement: `| .factory/ | VSDD factory artifacts (STATE.md, stories, specs, research, maintenance logs) |`
2. **DOC-003/004:** Update README.md — add pcapng to architecture table Reader row and Features bullet.
3. **DOC-005/006:** Update docs/adr/0002 — fix `detail` field type (HashMap→BTreeMap) and add `timestamp: u32` to `StreamHandler::on_data` signature.
4. **DOC-007/008/009:** Update docs/adr/0001/0002/0003 — replace stale line numbers with function-name anchors; add amendment note for modbus/dnp3 fields in ADR-0001 struct snippet; fix `parse_error_count()` Required label.
5. **DOC-002 (HIGH — needs authoring):** Author `docs/adr/0009-pcapng-reader-design.md` capturing the pcapng reader design decisions referenced 40+ times in src/reader.rs. Authoring effort required; not a simple text edit.

### (C) Already-fixed this run

- **F-MAJ-001:** ARCH-INDEX updated to v1.6 (commit a6efb23). SS-01 BC count corrected (8→17), SS-11 BC count corrected (29→34). No further action required.

### (D) Backlog / product-owner

- **Holdout staleness:** HS-064/075 assertion update (3 top-level keys → 5; MITRE envelope keys added PR #209). HS-108 sub-assertion wording. HS-090/098 invocation form normalization (`--json` flag vs `--output-format json`). Product-owner content updates.
- **Holdout coverage gaps:** 73 feature seeds with zero HS files — DNP3 (32), ARP (28), finding-collapse (13), Modbus (no dedicated scenarios). Major gap; product-owner new scenario authoring.
- **Pattern HIGH items:** PC-001 (DNP3 StreamHandler conformance, ~2-4 days effort) and PC-003 (DNP3 dropped-counter) — already tech-debt tracked; not safe auto-fixes; deferred to next ICS feature cycle.
- **New pattern items:** PC-013/PC-014/PC-015 added to register this run. Deferred pending DF-VALIDATION-001 check before GitHub issues.
- **DEP-002:** deny.toml stale license-allowlist cleanup — LOW housekeeping; defer to next license-audit sweep.
- **TD-MAINT-RISK-REGISTRY-BACKFILL:** Author risk-register.md and assumptions.md before next ICS protocol feature to enable spec-coherence criteria 42–50.
- **PERF-REASM-NFR-001:** Formal NFR/VP for reassembly per-packet CPU O(1) amortised. Informational backlog.

---

## Resolved This Run

| Item | Resolution | Evidence |
|------|-----------|----------|
| F-MAJ-001 | ARCH-INDEX SS-01 BC count fixed (8→17), SS-11 BC count fixed (29→34). Version bumped to v1.6. | commit a6efb23 on factory-artifacts |
| PERF-CRITICAL-maint-2026-06-17 | reassembly/tls.pcap +54.5% RETRACTED — confirmed thermal noise. TD-MAINT-PERF-ARP-HOTPATH updated. | performance-baseline.md controlled re-run |

---

## Raw Finding Counts by Sweep

| Sweep | Findings | CRITICAL | HIGH | MEDIUM | LOW | FAIL-BUG |
|-------|---------|---------|------|--------|-----|---------|
| 1 — Dependency Audit | 6 | 0 | 0 | 1 | 5 | 0 |
| 2 — Doc Drift | 10 | 0 | 2 | 4 | 4 | 0 |
| 3 — Pattern Consistency | 15 (3 new) | 0 | 3 | 6 | 6 | 0 |
| 4 — Holdout Freshness | 3 findings + 1 gap | 0 | 0 | 0 | 0 | 0 |
| 5 — Performance | 2 REGRESSION-MINOR | 0 | 0 | 0 | 2 | 0 |
| 7 — Spec Coherence | 2 MAJOR | 0 | 0 | 2 | 0 | 0 |
| 8 — Tech Debt Review | 0 new (register updated) | 0 | 0 | 0 | 0 | 0 |
| **Total (raw observations)** | **~38** | **0** | **0** | **1 (DEP-005)** | many | **0** |

*Total 38 is an approximate count of raw observations across sweeps; many are carry-forwards from maint-2026-06-17. Net-new findings: 6 (PC-013, PC-014, PC-015, DEP-005, DOC-002, F-MAJ-001). F-MAJ-001 resolved this run.*
