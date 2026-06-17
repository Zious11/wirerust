# Maintenance Sweep Report — maint-2026-06-17

**Run:** maint-2026-06-17 | **Mode:** STEADY_STATE background sweep | **Baseline:** develop e1273c8 / v0.7.1
**Overall verdict:** NON-BLOCKING — zero CRITICAL findings, zero CVEs, zero security blockers.

## Sweeps Run
Applicable (Rust CLI): dependency-audit, doc-drift, pattern-consistency, holdout-freshness, performance-regression, spec-coherence, tech-debt-register, risk-assumption-monitoring.
N/A (skipped): DTU-fidelity (dtu_required:false), accessibility-regression (no UI), design-drift (no UI/design system).
Adaptation: holdout-freshness routed to consistency-validator (holdout-evaluator is write-restricted by info-asymmetry design).

## Findings by Sweep

### 1. Dependency Audit — NON-BLOCKING (5 findings, all LOW/MEDIUM)
- FINDING-001 RUSTSEC-2026-0097 rand 0.8.5 unsound (LOW, build-dep only). NEW: rand 0.8.6 now available — `cargo update -p rand` resolves it; CI --ignore flag can be retired. AUTO-FIX.
- FINDING-005 zerocopy 0.8.48->0.8.52 (MEDIUM precautionary, no active advisory). AUTO-FIX.
- FINDING-002 8 license-not-encountered deny.toml warnings (LOW). MANUAL (allowlist trim).
- FINDING-003 syn v1/v2 duplicate (LOW, expected proc-macro migration). NO ACTION.
- FINDING-004 35 crates with updates available (LOW). AUTO-FIX with test-verify (shlex 1.3->2.0 major, build-dep).
- number_prefix RUSTSEC-2025-0119 RESOLVED (indicatif bump). 0 unmaintained warnings.

### 2. Documentation Drift — 14 findings (4 HIGH, 6 MEDIUM, 4 LOW)
- H-1 README: ARP analyzer (v0.7.0) entirely undocumented (flags, protocol table, architecture). AUTO-FIX.
- H-2/H-3 README: CSV output omitted / mislabeled as roadmap (shipped v0.1.0). AUTO-FIX.
- H-4 ADR-0002: TLS shown "planned"; Modbus/DNP3/ARP missing from analyzer table. AUTO-FIX.
- M-1 README architecture diagram stale (HTTP/TLS only). M-2 ADR-005/006/007 cited in code+CHANGELOG but .md files MISSING. M-3 lib.rs crate doc lists only DNS/HTTP/TLS. M-4 --hosts flag undocumented. M-5 reassembly tuning flags undocumented. M-6 ~70 stale "RED:" comments in 5 test files (= issue #254).
- L-1..L-4 minor stale ADR snippets, rayon unused dep, T0855 revoked-ID in CHANGELOG, stale line ref.

### 3. Pattern Consistency — 12 findings (3 HIGH, 5 MEDIUM, 4 LOW)
- PC-001 Dnp3Analyzer does NOT implement StreamHandler/StreamAnalyzer trait; bespoke on_data, drops Direction. ~2-4 days. MANUAL.
- PC-002 inconsistent fully-qualified ThreatCategory paths (modbus/dnp3/arp) vs module-level imports elsewhere. AUTO-FIX (cosmetic).
- PC-003 Dnp3Analyzer lacks dropped_findings counter (Modbus/reassembly have it). MANUAL (small).
- PC-004 chrono path style; PC-005 duplicated MAC/IPv4 format strings; PC-006 modbus analyzer_name "modbus" lowercase vs others uppercase (breaking change to fix); PC-007 BTreeMap import location; PC-008 Modbus dual summarize() method.
- PC-009..012 decimal-vs-hex FC keys, missing doc, non-deterministic JSON key order, blanket allow(dead_code).

### 4. Holdout Freshness — 100 scenarios; 2 stale, 98 active, 0 retired
- HS-008 stale: "23 seeded IDs" -> now 25 (ARP T0830/T1557.002). AUTO-FIX.
- HS-009 stale: "15 emitted IDs" -> now 17 (STORY-114). AUTO-FIX.
- HS-018 missing lifecycle_status frontmatter field (schema gap). AUTO-FIX.
- S2 universal: all carry last_evaluated: null since Phase-4 (2026-06-01), 7 releases ago — informational.

### 5. Performance Regression — benchmark suite exists; informational (no NFR latency targets)
- 4 WARNING regressions 10.9%-21.9% (decode/tls +21.9%, decode/dns +10.9%, summary/segmented +11.7%, reassembly/segmented +20.1%).
- reassembly/tls.pcap +54.5% median CRITICAL-variance — std_dev inflated 65x; RE-RUN required before believing.
- Root-cause hypothesis: DecodedFrame::Arp match variant adds i-cache/branch pressure in hot decode loop. Product decision, not correctness. NFR-PERF-002/003 perpetually DEFERRED (no fixtures).

### 6. Spec Coherence (33 criteria) — PASS; 3 MAJOR + 4 MINOR, ALL pre-existing/tracked
- Structural integrity ALL PASS: 283 BC files = BC-INDEX; 24 VP files = VP-INDEX; 70 story files = STORY-INDEX; tags v0.1.0-v0.7.1 all present.
- F-MAJ-001 "68-story"/"68 stories,457 pts" stale labels (should be 70 / v0.7.1). F-MAJ-002 VP-024 "v2.3" label lag (authoritative v2.4) = DRIFT-E17-VERSIONLABEL-LAG-001. F-MAJ-003 epics.md "12 Subsystems" (SS-14/15/16 exist) = DRIFT-EPICS-REGISTRY-STRUCTURAL-001.
- 4 MINOR drift items all tracked in STATE.md.

### 7. Tech Debt Register — 18 existing; 11 recommended new; 3 closures; 3 OVERDUE
- OVERDUE: DRIFT-DNP3-DIRECTION-001 (2 releases past target); O-07 rayon unused (8 releases); FU-REPO-WIDE-DOC-DEBT / #254 (scheduled after STORY-114 merge which happened 2026-06-15).
- WARNINGS: #252 VP-024 proof_file_hash before next F6; DRIFT-F2-COUNT-001 holdout count drift; #254 RED-prose will trigger MEDIUM adversarial findings next corpus sweep.
- Closure: DRIFT-MITRE-EMITTED-LABEL-001 INVALID (claim factually wrong); DRIFT-ETHERPARSE-0.20 + DRIFT-ENGINE-RELEASECONFIG RESOLVED.

### 8. Risk & Assumption Monitoring — STRUCTURAL GAP
- No formal ASM-NNN / R-NNN registry exists; VSDD criteria 42-50 structurally unverifiable. 9 informal assumptions, 8 informal risks catalogued.
- Unvalidated 2+ releases: ASM-CAND-003 (reassembly thresholds, 8 releases), ASM-CAND-009 (ARP storm default 50/s).
- Top risks: R-CAND-001 unbounded weak-cipher evidence Vec (#102, MEDIUM); rayon unused (LOW); RUSTSEC rand (LOW).
- Recommendation: backfill specs/risk-register.md + specs/assumptions.md before next ICS protocol feature.

## Classification: Auto-Fixable vs Manual

### Auto-fixable (small, mechanical — fix-PR candidates)
- A1 Dependency bumps: `cargo update -p rand` + `-p zerocopy` (+ retire CI --ignore for RUSTSEC-2026-0097).
- A2 README + ADR-0002 + lib.rs doc-drift: document ARP/DNP3/Modbus/CSV (H-1..H-4, M-1, M-3).
- A3 #254 / M-6: strip ~70 stale "RED:" comments from passing tests.
- A4 Holdout count fixes: HS-008 (25), HS-009 (17), HS-018 lifecycle_status field.
- A5 Spec label lags: STORY-INDEX/STATE "70 stories", VP-024 "v2.4", epics "Subsystems" count.
- A6 Pattern cosmetics: PC-002/PC-007 import-style normalization.
- A7 Missing ADR files: create ADR-005/006/007 (M-2) — or correct the dangling citations.

### Manual / judgment-required (NOT auto-fixed)
- B1 PC-001 DNP3 StreamHandler trait conformance (~2-4 days, design).
- B2 PC-006 modbus analyzer_name casing (breaking output change).
- B3 PC-003 DNP3 dropped_findings counter.
- B4 Performance regressions — product decision; first RE-RUN reassembly/tls benchmark.
- B5 Full `cargo update` (35 crates incl shlex major) — needs test-verify.
- B6 Risk/assumption registry backfill (structural).
- B7 deny.toml allowlist trim.

## Overdue / Notifications
- WARNING (overdue tech debt): DRIFT-DNP3-DIRECTION-001, O-07 rayon unused, #254 RED-prose.
- INFO: routine findings; no BLOCKING/CRITICAL CVE.

## Recommended Fix-PR Batch (max 10 PRs, awaiting human gate)
1. chore(deps): cargo update -p rand -p zerocopy; retire RUSTSEC-2026-0097 ignore (A1)
2. docs(readme): document ARP/DNP3/Modbus analyzers + CSV output (A2)
3. docs(adr): fix ADR-0002 analyzer table + create/repair ADR-005/006/007 citations (A2/A7)
4. test: strip stale RED-gate comments repo-wide — closes #254 (A3)
5. fix(holdout): refresh HS-008/HS-009 counts + add HS-018 lifecycle_status (A4)
6. docs(spec): correct 70-story / VP-024 v2.4 / epics subsystem-count labels (A5)
7. refactor: normalize ThreatCategory/BTreeMap import style PC-002/PC-007 (A6)
