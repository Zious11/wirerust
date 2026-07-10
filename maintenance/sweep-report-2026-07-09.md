# Maintenance Sweep Report — maint-2026-07-09

**Run ID:** maint-2026-07-09
**Date:** 2026-07-09
**Prior run:** maint-2026-07-08
**develop HEAD at open:** 716054a (v0.11.5 + 14 unreleased commits)
**develop HEAD at close:** 716054a (no code PRs this run — factory-artifacts only)
**Sweeps executed:** 1 (deps), 2 (doc-drift), 3 (patterns), 4 (holdouts), 5 (perf), 7 (spec-coherence), 8 (tech-debt), risk-asm (sweep 12)
**Sweeps skipped:** Sweep 6 (DTU — dtu_required: false), Sweep 9 (a11y/design — CLI product, no UI)

---

## Sweep Verdicts Summary

| Sweep | Result | Finding Count | Notable |
|-------|--------|---------------|---------|
| 1 — Dependency Audit | **CLEAN** | 0 new; 2 pre-existing LOW (DEP-006/007, both deferred) | 193 crates; DB 1159 advisories; indicatif 0.18.5→0.18.6 only delta (Dependabot #386) |
| 2 — Doc Drift | **4 findings** | 0 HIGH, 1 MED, 1 LOW, 2 INFO | DD-001 MED (README --coverage-gaps), DD-002 LOW (README DNP3 counters); all 9 prior findings (N-1..N-8, NEW-001) RESOLVED |
| 3 — Pattern Consistency | **CLEAN** | 0 new; 3 carry-forward LOW (PF-002/003/004) | Clippy 0 warnings; fmt clean; wave-72 delta scan 0 new findings across all 7 categories |
| 4 — Holdout Freshness | **132/132 CLEAN** | 0 stale | 13 PR #389 JSON-shape scenarios confirmed repaired + verified against binary at 716054a; HS-INDEX-ENIP-WAVE-DRIFT-001 unchanged (Route C deferred) |
| 5 — Performance | **NOISE-SUSPECT** | 0 actionable regressions | Wave-72 delta cold-path only; AC-149-003 INDETERMINATE (25.698 µs FAIL / 24.075 µs PASS straddling 24.445 µs ceiling); quiescent re-run recommended |
| 7 — Spec Coherence | **PASS 33/33** | 1 new LOW (SC-001); 9 carry-forward LOW | Prior FAILs C8/C27/C29 all resolved; input-hash scan MATCH=115 STALE=0 |
| 8 — Tech Debt Register | **v1.7** | 4 rows added (all P3), 3 updated, 0 resolved | TD-MAINT-RISK-REGISTRY-BACKFILL P1 (3 sweeps unactioned); 0 overdue items |
| Risk-Assumption Monitoring | **NEEDS_ATTENTION (improving)** | 4 resolved, 2 carry-forward P1 | ASM-003+009 ACCEPTED via PR #382; R-CAND-006 ACCEPTED, R-CAND-012 RESOLVED via PR #384 |

---

## Key Findings by Sweep

### Sweep 1 — Dependency Audit

Source: `maintenance/dependency-audit.md`

**Verdict: CLEAN.** `cargo audit` returned zero advisories against all 193 locked crates (RustSec DB 1159 advisories). `cargo deny` exited 0 with 9 warnings — all known and pre-registered (DEP-006 × 8 unused license-allowlist entries, DEP-007 × 1 syn v1/v2 duplicate). `cargo outdated` not installed (skipped). The only dependency change vs maint-2026-07-08 is indicatif 0.18.5 → 0.18.6 (Dependabot PR #386, already merged); no advisory against 0.18.6.

Pre-existing deferred items re-confirmed: DEP-006 (deny.toml 8 unused `license-not-encountered` entries) and DEP-007 (syn 1.0.109 + 2.0.117 dual-version; upstream resolution required). Both remain valid deferrals. No fix PR required.

**Finding counts from artifact: CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 2 (both pre-existing, deferred).**

---

### Sweep 2 — Doc Drift

Source: `maintenance/doc-drift.md`

**Verdict: 4 findings (1 MED, 1 LOW, 2 INFO).** All 9 prior findings (N-1..N-8 from maint-2026-07-06, NEW-001 from maint-2026-07-08) are fully resolved. The four new findings are:

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| DD-001 | MED | README "Analyze flags" block missing `--coverage-gaps` flag (added in v0.11.2 PR #355; `protocols` subcommand was fixed as N-1 but `--coverage-gaps` remains absent) | PROPOSED Route A (next docs PR) |
| DD-002 | LOW | README DNP3 TCP Analyzer section missing 3 observability counters: `dropped_findings`, `master_addrs_dropped`, `pending_requests_evicted` (added in v0.11.5 PR #370) | PROPOSED Route A (next docs PR) |
| DD-003 | INFO | README has no mention of `schema_version` envelope field or lowercase JSON enum casing (live in develop since PR #389; CHANGELOG `[Unreleased]` is authoritative for pre-release consumers) | Defer to v0.12.0 release cut |
| DD-004 | INFO | ADR-008 numbering gap — docs/adr/ sequence reads 0001–0007, 0009–0012 with no ADR-008 | Optional placeholder file |

CLAUDE.md, CHANGELOG, ADR-012, ADR-0001, ADR-0002, lib.rs crate doc, README flags block (14 flags), and feature bullets all verified clean for wave-72 delta.

---

### Sweep 3 — Pattern Consistency

Source: `maintenance/pattern-consistency.md`

**Verdict: CLEAN (0 new findings).** Clippy exits 0 at develop 716054a (0 warnings). `cargo fmt --check` exits 0. Wave-72 delta scan (c4eb1f4..716054a) across all 7 categories returned no new issues: 0 new diagnostic-counter `+=` sites, 0 new `as` numeric casts, 0 production `self.flows[` index usage, 0 error-handling divergence, 0 naming-convention drift, 0 test-structure inconsistency, 0 stale enum-casing assertions (all 13 PR #389 JSON-shape assertions verified lowercase/snake_case).

PF-001 (109 `+=` → `saturating_add`, resolved PR #384 c4eb1f4) remains RESOLVED with no regression. Carry-forward items unchanged: PF-002 LOW (dnp3.rs 4 free-function naming gaps), PF-003 LOW (enip.rs `check_t0814` prefix), PF-004 LOW (Dnp3/EnipAnalyzer trait gap — ADR-007/010 intentional).

**Finding counts from artifact: NEW 0; RESOLVED 1 (PF-001, carried from prior); STILL-OPEN 3 (PF-002/003/004, all LOW).**

---

### Sweep 4 — Holdout Freshness

Source: `maintenance/holdout-freshness.md`

**Verdict: CLEAN.** All 132 holdout scenarios remain `lifecycle_status: active`; 0 stale; 0 retired. HS-INDEX version: v2.13. The 13 PR #389 JSON-shape scenarios (HS-021, 032, 034, 059, 064, 065, 074, 075, 024, 033, 035, 050, 054) were repaired in the wave-72-repair burst and verified against the binary at develop 716054a:
- 6 scenarios: PascalCase → lowercase verdict/confidence + snake_case category (BC-2.11.036 v1.2)
- 2 scenarios: 5-key → 6-key JSON envelope adds `schema_version` (BC-2.11.037); `jq 'keys'` confirmed `["analyzers","findings","mitre_attack_version","mitre_domain","schema_version","summary"]`
- 5 scenarios: DF-SIBLING-SWEEP-001 category/confidence literals lowercased; regex sweep of all 132 scenarios found 0 PascalCase enum literals in JSON assertion contexts

HS-INDEX-ENIP-WAVE-DRIFT-001 (wave/story column drift for waves 58-61, STORY-130..138): CONFIRMED, deferred Route C per human decision 2026-07-08, unchanged this sweep.

Advisory (not marked stale): HS-082 optional-verify step uses mixed-case substrings where `Verdict::fmt` emits all-caps; cleanup candidate for a future precision pass.

**Finding counts from artifact: active 132 / stale 0 / retired 0.**

---

### Sweep 5 — Performance

Source: `maintenance/performance.md`

**Verdict: NOISE-SUSPECT — 0 actionable regressions.** Two passes of both benchmark suites at develop 716054a. Machine exhibited 9–21 severe outliers per 100 samples with run-to-run point-estimate swings up to 74% between consecutive runs — same noise signature as maint-2026-07-08. Wave-72 delta (c4eb1f4..716054a) is provably cold-path only (reporter/json.rs serialization path, findings.rs serde annotations, arp.rs comment); no hot-path code changed.

| Benchmark | Run 1 (µs) | Run 2 (µs) | vs Jun-22 anchor | Verdict |
|-----------|-----------|-----------|-----------------|---------|
| reassembly/tls.pcap (AC-149-003) | 25.698 | 24.075 | +5.2% / −1.4% | NOISE-SUSPECT |
| All other pipeline benchmarks | — | — | +4.6% to +148% | NOISE-SUSPECT |
| tls_fragmented/3-record-carry-drain | 1.7004 | 1.8823 | −17.7% / −8.9% vs Jul-08 | NOISE-SUSPECT |

**AC-149-003 status: INDETERMINATE.** Run 1 (25.698 µs) is 5.1% above the 24.445 µs ceiling (FAIL); run 2 (24.075 µs) is 1.5% below (PASS). The 6.3% run-to-run spread cannot be resolved under current machine conditions. No fix PR warranted. `performance-baseline.md` NOT updated (correct — noisy data must not become anchor). Jun-22 controlled re-run values remain authoritative.

---

### Sweep 7 — Spec Coherence

Source: `maintenance/spec-coherence.md`

**Verdict: PASS 33/33 criteria (0 FAIL).** This is the second consecutive PASS at 33/33 (also maint-2026-07-08). All three prior FAIL criteria from maint-2026-07-06 (C8 VP-INDEX phantom entries, C27 BC-2.16.016 no active story, C29 module-criticality frozen) are resolved. Input-hash scan: MATCH=115 STALE=0.

**Artifact count verification (PROP-MAINT-03 — from index files):**

| Artifact | Disk Count | Index Claim | Match |
|----------|-----------|-------------|-------|
| STORY-*.md (excl. INDEX) | 115 | STORY-INDEX `total_stories: 115` | YES |
| VP files (vp-*.md) | 43 | VP-INDEX `total_vps: 43` | YES |
| BC files (active) | 347 | BC-INDEX v2.22: 348 on disk, 347 active | YES |
| Release tags (v-prefix) | 21 | v0.1.0..v0.11.5 | YES |

Wave-72 spec delta coherence: VP-INDEX v2.35→v2.39 (Multi-File Proof Anchor Algorithm + EC-005), VP-024 v2.4→v2.5 (proof_file_hash populated, verification_lock:true), STORY-162 draft (E-11, wave-TBD, input-hash MATCH), STORY-INDEX v3.33 wave-72 row (DELIVERED & CLOSED) — all COHERENT.

**New finding:**

| ID | Severity | Criterion | Description |
|----|----------|-----------|-------------|
| SC-001 | LOW | C18 (auxiliary) | STORY-INDEX line 99 registry header narrative lists stories through STORY-157 but omits STORY-158..162. Authoritative numeric fields all correct; only narrative text is incomplete. |

**Carry-forward items (all LOW/MINOR admin, Route C deferred by human decision):**

| ID | Description | Gap |
|----|-------------|-----|
| SC-PERSIST-001 (F-NEW-MIN-001) | STORY-INDEX tally 337 vs 347 active BCs | gap=10 |
| SC-PERSIST-002 (F-NEW-MIN-003) | stories_delivered counter 106; claimed actual 101 | off by 5 |
| SC-PERSIST-003 (F-NEW-MIN-004) | BC body propagation pending STORY-046/058/103/104/108 | 5 stories |
| F-MAJ-003 | epics.md structural debt: total_bcs=337 vs 347 active; story count 107 vs 115 | gap +8 |
| EPICS-STORY-COUNT-DRIFT-001 | epics.md count 107 vs 115 actual | gap grew 4→8 since maint-2026-07-08 |
| F-MIN-001..004 | DRIFT items: F2-COUNT-001, BC-2.15.024-EC006-PROSE-001, E16-BC-BACKLINK-GAP-001, VP024-BTREEMAP-PROSE-001 | spec-wording only |

**Finding counts from artifact: PASS 33/33; FAIL 0; NEW LOW 1 (SC-001); carry-forward 9 LOW/MINOR.**

---

### Sweep 8 — Tech Debt Register

Source: `maintenance/tech-debt-check.md`

**Verdict: Register v1.7.** This was a register-reconciliation pass only; no new PRs in this sweep.

**4 rows added (all P3):**

| ID | Description | Priority |
|----|-------------|----------|
| DNP3-CLOSEDFLOW-REOPEN-REUSE-001 | DNP3 `closed_flow_direct_operates` Vec lists same FlowKey under NAT port reuse. Spec-conformant per BC-2.15.021 PC-4. Optional docstring rename only. | P3 |
| TD-W7.1-PUBLIC-API-BASELINE | `cargo public-api` two-step setup deferred per no-flaky-stub policy (CLAUDE.md W7.1). | P3 |
| TD-INPUT-HASH-CI-GATE | `bin/compute-input-hash --scan` not wired into develop CI. Manual Phase-4 gate in place. | P3 |
| TD-DTOLNAY-PIN-EXEMPTION | `dtolnay/rust-toolchain@stable`/`@nightly` allowlisted in action-pin-gate. Resolution approach undecided. | P3 |

**3 rows updated:** maint-2026-07-09 Sweep 8 summary note added; resolved TD-MAINT-THRESHOLD-CALIB-001 removed from P1 candidate list; `last_updated` bumped 2026-07-08→2026-07-09.

**Items reconciled (no new row needed):** HS-INDEX-ENIP-WAVE-DRIFT-001 and EPICS-TOTAL-BCS-DRIFT-001 subsumed in existing ROUTE-C-DEFERRED row. DEP-006/007 rows unchanged. STORY-162 confirmed covering wave-72 S-7.02 process gaps.

**Overdue items: 0. Human-triage watchlist:** TD-MAINT-RISK-REGISTRY-BACKFILL (P1, 3 sweeps unactioned); SEC-W71-001 (VALIDATED-PENDING-FILING); TD-DTOLNAY-PIN-EXEMPTION (raise at next CI/supply-chain review).

**Finding counts from artifact: rows added 4 / rows updated 3 / rows resolved 0 / P1 open 1 / overdue 0.**

---

### Risk-Assumption Monitoring (Sweep 12)

Source: `maintenance/risk-assumption-monitoring.md`

**Verdict: NEEDS_ATTENTION (improving).** No formal ASM-NNN / R-NNN registry exists (TD-MAINT-RISK-REGISTRY-BACKFILL P1, now 3 sweeps unactioned). No mitigation was invalidated by wave-72 or v0.11.5. ARCH-INDEX v2.12 unchanged.

**Summary counts (from artifact):**

| Category | Total | Still open | Resolved since prior sweep | Escalation-worthy |
|----------|-------|------------|---------------------------|-------------------|
| Informal assumptions (ASM-CAND) | 11 | 9 | ASM-003 ACCEPTED, ASM-009 ACCEPTED (2, via PR #382) | 0 (down from 2) |
| Informal risks (R-CAND) | 12 | 5 | R-CAND-006 ACCEPTED (PR #382), R-CAND-012 RESOLVED (PR #384) | 1 (R-CAND-001 P1) |

Wave-72 mitigation validity check: all 6 wave-72 PRs (#386–#391) — NOT INVALIDATED. Trajectory-tail: →1→0→0→0.

**Key carry-forward concerns:**
1. **R-CAND-001 (P1, 6+ months open)** — Unbounded weak-cipher evidence Vec in `tls.rs` (NFR-RES-023, GitHub #102). `MAX_WEAK_CIPHER_EVIDENCE = 64` cap not yet shipped. Closest vehicle: STORY-150.
2. **TD-MAINT-RISK-REGISTRY-BACKFILL (P1, 3 sweeps)** — No `specs/risk-register.md` or `specs/assumptions.md`. Ideal window: wave-72 closed, no active feature cycle.
3. **REC-005/006/010 (LOW, XS each)** — ADR-007 Crain/Sistrunk CRC-caveat, README MACsec ARP limitation, tech-debt register footer stale P1 note. All three sweeps unactioned. Batch into next docs PR.

---

## Proposed Fix Routes

All routes are PROPOSED pending human approval.

### ROUTE A — Docs PR (single vehicle)

Batch the following into one `docs:` or `chore:` PR:

| Item | Severity | Notes |
|------|----------|-------|
| DD-001 | MED | README "Analyze flags" block: add `--coverage-gaps` flag description, opt-in rationale (ADR-012 Decision 8), and usage example `wirerust analyze capture.pcap --all --coverage-gaps` |
| DD-002 | LOW | README DNP3 TCP Analyzer section: add `dropped_findings` / `master_addrs_dropped` / `pending_requests_evicted` counter table (mirrors ARP section format) |
| REC-005 | LOW (XS) | ADR-007 Decision 3: add Crain/Sistrunk CRC-caveat normative note (3rd sweep carry-forward) |
| REC-006 | LOW (XS) | README § Known Limitations: add one sentence on MACsec/VLAN ARP offset detection limitation (3rd sweep carry-forward) |
| REC-010 | LOW (XS) | Tech-debt register footer: remove resolved TD-MAINT-THRESHOLD-CALIB-001 from P1 candidates note; list only TD-MAINT-RISK-REGISTRY-BACKFILL |
| DD-004 | INFO (optional) | Add `docs/adr/0008-withdrawn.md` placeholder to close the ADR-008 numbering gap |
| DEP-006 | LOW | Prune deny.toml unused `license-not-encountered` allowlist entries (8 entries); its deferral note says fold into next docs/chore PR |

### ROUTE B — Factory-Artifacts Only (no code PR)

| Item | Action |
|------|--------|
| SC-001 | Extend STORY-INDEX line 99 registry header narrative to include STORY-158..162 |
| SC-PERSIST-002 | Verify the stories_delivered discrepancy: STATE.md records 106; spec-coherence sweep claims actual delivered count is 101. Reconcile with delivery evidence (D-NNN decision log) before adjusting counter. |

### NO-ACTION

- **Sweeps 1, 3, 4:** All clean; no findings requiring a fix vehicle.
- **Sweep 5 (perf):** Noise-blocked. Quiescent controlled re-run recommended before AC-149-003 adjudication or baseline update (recommendation standing from maint-2026-07-08). No fix PR warranted from noisy data.

### HUMAN-TRIAGE (Gate Questions)

| Item | Context |
|------|---------|
| SEC-W71-001 issue filing | CWE-22 path-traversal in `bin/compute-input-hash`; status VALIDATED-PENDING-FILING since 2026-07-08. Human deferred filing; still open. Recommend filing before any feature cycle touching `bin/compute-input-hash`. |
| TD-MAINT-RISK-REGISTRY-BACKFILL P1 scheduling | 3 sweeps unactioned. Wave-72 closed and no active feature cycle — this is the ideal execution window. Requires creating `specs/risk-register.md` (R-001..R-012) and `specs/assumptions.md` (ASM-001..ASM-011). Estimated effort: ~1 session. |
| R-CAND-001 P1 (issue #102, vehicle STORY-150) | Weak-cipher evidence Vec unbounded (NFR-RES-023). 6+ months at P1. `MAX_WEAK_CIPHER_EVIDENCE = 64` cap needed. Confirm STORY-150 as the fix vehicle for the next TLS-touching cycle. |
| AC-149-003 INDETERMINATE | Run 1 FAIL (25.698 µs), Run 2 PASS (24.075 µs). Cannot adjudicate without a quiescent re-run. Human decides: schedule controlled benchmark session or defer to next maintenance sweep. |
| Route-C deferrals | HS-INDEX ENIP wave/story column fix (HS-INDEX-ENIP-WAVE-DRIFT-001), epics.md tally reconcile (EPICS-TOTAL-BCS-DRIFT-001 + F-MAJ-003), BC/CWE-tag consistency (F-MIN-001..004, SC-PERSIST-001/002/003) — all deferred Route C per prior human decision (2026-07-08). Status unchanged this run. |

---

## Deferred / DF-VALIDATION-001 Status

**No new deferred findings were created this run.** All items encountered in this sweep were either:
- Reconciled into the existing tech-debt register v1.7 (4 new P3 rows, 3 updated rows), or
- Carry-forwards from prior human Route-C deferrals, or
- Confirmed as matching prior sweep dispositions with no status change.

As a result, no research-agent (DF-VALIDATION-001) validation pass is required for this run. The register is now at v1.7 with 0 overdue items.

SEC-W71-001 retains its VALIDATED-PENDING-FILING status from maint-2026-07-08 (no change; filing deferred by human election).

---

## Summary

| Metric | Value |
|--------|-------|
| Findings identified (all sweeps) | 5 new (DD-001 MED, DD-002 LOW, DD-003/004 INFO, SC-001 LOW); 9+ carry-forwards |
| Actionable findings resolved this run | 0 (no code PRs; all prior-run resolutions tracked in maint-2026-07-08 report) |
| PRs merged | 0 (this sweep is factory-artifacts only) |
| Tech-debt register rows added | 4 (all P3) |
| Blocking issues at close | 0 |
| develop HEAD at close | 716054a (unchanged — no code PRs) |
| Spec coherence criteria passing | 33/33 (PASS; 0 FAIL) |
| Holdout scenarios active / stale | 132 / 0 |
| Input-hash scan | MATCH=115 STALE=0 |
