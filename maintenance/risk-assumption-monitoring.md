---
document_type: maintenance-sweep-report
sweep: risk-assumption-monitoring
sweep_number: 12
run_id: maint-2026-07-09
pipeline: STEADY_STATE
product: wirerust
released_version: v0.11.5
date: 2026-07-09
trigger: scheduled
producer: consistency-validator
read_only: true
---

# Risk & Assumption Monitoring — maint-2026-07-09

**Run ID:** maint-2026-07-09
**Date:** 2026-07-09
**Latest release:** v0.11.5 (released 2026-07-07); prior release chain for this sweep: v0.11.3 (2026-07-06), v0.11.4 (2026-07-06), v0.11.5 (2026-07-07).
**Baseline for this sweep:** maint-2026-07-06 (v0.11.4).
**Develop HEAD at sweep time:** `716054a` (14 unreleased commits since v0.11.5; wave-72 + maint-2026-07-08 + dependabot #386).
**Scope:** L2 Domain Spec shards, PRD, ADRs (ADR-001..ADR-012), NFR catalog, domain-debt register, tech-debt register, STATE.md.

---

## Summary

| Sweep | Status | Findings | PRs Opened | Issues Created |
|-------|--------|----------|-----------|----------------|
| Dependency Audit | N/A | — | — | See Sweep 1 |
| Documentation Drift | N/A | — | — | See Sweep 2 |
| Pattern Consistency | N/A | — | — | See Sweep 3 |
| Holdout Freshness | N/A | — | — | See Sweep 4 |
| Performance Baseline | N/A | — | — | See Sweep 5 |
| Risk & Assumption Monitoring | NEEDS_ATTENTION | 4 resolved, 2 carry-forward P1 | 0 | 0 |

---

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Verdict: NEEDS_ATTENTION (improving)**

Two major escalation items (ASM-CAND-003, ASM-CAND-009) resolved via formal acceptance (PR #382). One P1 risk (R-CAND-001, weak-cipher heap) remains open and unaddressed for 6+ months. Governance gap (no formal registry, TD-MAINT-RISK-REGISTRY-BACKFILL) is now three sweeps old at P1. Product is structurally sound — no mitigation was invalidated, no memory-DoS is present, and all counter sites now use saturating arithmetic after PR #384.

---

## Dependency Audit

> N/A for this sub-sweep (Sweep 12 = risk-assumption-monitoring). Dependency auditing is
> covered by Sweep 1 (dependency-audit) in the same maint-2026-07-09 cycle.

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| — | — | None new this sweep | — | — | See Sweep 1 |

---

## Documentation Drift

> N/A for this sub-sweep (Sweep 12 = risk-assumption-monitoring). Documentation drift is
> covered by Sweep 2 (doc-drift) in the same maint-2026-07-09 cycle. RAM-specific doc
> gaps are tracked below in ASM-CAND-008 (ADR-007 Decision 3 Crain/Sistrunk caveat) and
> R-CAND-008 (README MACsec ARP limitation).

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| ADR-007 Decision 3 | Crain/Sistrunk caveat | missing normative note | low | REC-005 (XS, next doc PR — 3rd sweep) |
| README | Known Limitations | missing MACsec ARP limitation | low | REC-006 (XS, next doc PR — 3rd sweep) |
| tech-debt register footer | P1 candidates note | stale: lists resolved THRESHOLD-CALIB-001 | low | REC-010 (XS, next maintenance pass) |

---

## Pattern Consistency

> N/A for this sub-sweep (Sweep 12 = risk-assumption-monitoring). Pattern consistency is
> covered by Sweep 3 (pattern-consistency) in the same maint-2026-07-09 cycle.

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| — | — | — | — | See Sweep 3 |

---

## Holdout Scenario Freshness

> N/A for this sub-sweep (Sweep 12 = risk-assumption-monitoring). Holdout freshness is
> covered by Sweep 4 (holdout-freshness) in the same maint-2026-07-09 cycle.

| Metric | Value |
|--------|-------|
| Total scenarios | covered by Sweep 4 |
| Still valid | covered by Sweep 4 |
| Stale (intentional change) | covered by Sweep 4 |
| Missing coverage | covered by Sweep 4 |

---

## Performance Baseline

> N/A for this sub-sweep (Sweep 12 = risk-assumption-monitoring). Performance baseline is
> covered by Sweep 5 (performance-baseline) in the same maint-2026-07-09 cycle.

| Benchmark | Previous | Current | Delta | Status |
|-----------|----------|---------|-------|--------|
| — | — | — | — | See Sweep 5 |

---

## Trend (Last 3 RAM Sweeps)

RAM sweep history (prior runs of this sweep type):

| Date | ASM open/unvalidated | R open | Escalation-worthy | Resolved | Registry status |
|------|---------------------|--------|-------------------|----------|-----------------|
| 2026-06-17 (maint-2026-06-17) | 7 | 5 | 2 (ASM-003, ASM-009) | — (baseline) | DEFERRED P2 |
| 2026-07-06 (maint-2026-07-06) | 8 | 7 | 3 (ASM-003, ASM-009, RAM-001) | R-004, R-007, R-009 (3) | DEFERRED P2→P1 |
| 2026-07-09 (maint-2026-07-09) | 9 | 5 | 1 (RAM-001 only) | ASM-003/009 ACCEPTED, R-006 ACCEPTED, R-012 RESOLVED (4) | DEFERRED P1 (3 sweeps) |

---

## Executive Summary

The wirerust factory **still has no formal ASM-NNN / R-NNN registry**. Tech-debt item TD-MAINT-RISK-REGISTRY-BACKFILL remains DEFERRED at P1 — now three maintenance sweeps without action (maint-2026-06-17, maint-2026-07-06, maint-2026-07-09). This is the highest-governance risk.

**Two significant positive resolutions occurred since maint-2026-07-06:**

1. **TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED (PR #382, 2026-07-08):** Human decision to formally accept all three uncalibrated threshold assumptions. README § Known Limitations section was added, documenting ASM-CAND-003 (reassembly anomaly thresholds), ASM-CAND-009 (ARP storm rate 50/s), and R-CAND-006 (DNP3 T0827 direct-operate threshold) as uncalibrated engineering defaults with explicit written rationale. Both ASM-CAND-003 and ASM-CAND-009 escalation status is now RESOLVED. R-CAND-006 moves to ACCEPTED.

2. **REBIND-COUNT-SATURATING-001 RESOLVED (PR #384, 2026-07-08):** R-CAND-012 (rebind_count non-saturating arithmetic) resolved as part of the PF-001 sweep (109 sites converted to saturating_add). R-CAND-012 is now RESOLVED.

**Wave-72 delivery (PRs #387–#391, develop HEAD 716054a)** delivered 4 stories (STORY-158..161): CHANGELOG gate hardening, ADR-012 protocols-catalog docs, JSON enum casing + schema_version envelope, and VP-024 proof_file_hash re-lock. None of these introduce new analyzer logic, new ADRs, or new design assumptions. No existing mitigation was invalidated.

**No ASM has been invalidated.** No new design assumptions were introduced by wave-72 or v0.11.5. The architecture portfolio is stable at 7 analyzers + SS-18 protocol catalog (ARCH-INDEX v2.12, unchanged).

**Summary counts:**

| Category | Total | Still open / unvalidated | Resolved since prior sweep | Escalation-worthy |
|----------|-------|--------------------------|---------------------------|-------------------|
| Informal assumptions (ASM-CAND) | 11 | 9 | ASM-003 ACCEPTED, ASM-009 ACCEPTED (2) | 0 (down from 2) |
| Informal risks (R-CAND) | 12 | 5 | R-CAND-006 ACCEPTED, R-CAND-012 RESOLVED (2) | 1 (R-CAND-001 P1, unchanged) |
| Formal ASM-NNN identifiers | 0 | — | — | — |
| Formal R-NNN identifiers | 0 | — | — | — |

**Gate result:** ADVISORY. No formal registry exists; no gate can pass or fail. TD-MAINT-RISK-REGISTRY-BACKFILL is now 3 maintenance sweeps old without action — P1 confirmed, must complete before next ICS protocol feature cycle.

---

## Section 1 — Formal ASM/R Registry Status

### Finding RAM-001 (HIGH — AGING) — No formal ASM-NNN / R-NNN registry (PERSISTING, THIRD SWEEP)

No `specs/risk-register.md` or `specs/assumptions.md` has been created. The finding from maint-2026-06-17 is unchanged and now three sweeps old. TD-MAINT-RISK-REGISTRY-BACKFILL remains at P1 (promoted from P2 by maint-2026-07-06 sweep; P1 status not acted on in maint-2026-07-08 or this sweep).

Since maint-2026-07-06, no new architectural assumptions were added (wave-72 was CI/docs/format, no new protocols or ADRs). The count remains: 11 ASM-CAND + 12 R-CAND.

**Status:** PERSISTING HIGH. With wave-72 closed and no active feature cycle, this is the ideal window to complete the backfill. Registry backfill is the mandatory prerequisite for verifying VSDD spec-coherence criteria 42–50.

---

## Section 2 — Informal Assumptions (ASM Candidates)

### ASM-CAND-001 — Port-only classification is sufficient for Modbus TCP (ADR-005 Decision 1)

**Prior status (maint-2026-07-06):** STABLE.
**Current status:** STABLE — NO CHANGE.

Wave-72 and v0.11.5 deliver no Modbus-touching code. VP-022 Kani proof remains in force. Dispatch gate unaffected by CI/docs wave-72 stories.

**Assessment:** Stable. Recommend formal ASM-001.

---

### ASM-CAND-002 — Port-only classification is sufficient for DNP3 TCP (ADR-007 Decision 1)

**Prior status (maint-2026-07-06):** STABLE — POSITIVE DEVELOPMENT.
**Current status:** STABLE — NO CHANGE.

v0.11.5 PR #370 added DNP3 `dropped_findings` / `master_addrs_dropped` / `pending_requests_evicted` counters. These improve telemetry fidelity but do not touch the dispatch gate or classification assumption. VP-023 Kani proof remains in force.

**Assessment:** Stable. Recommend formal ASM-002.

---

### ASM-CAND-003 — Anomaly thresholds are adequate for forensic use without labelled corpus (O-03)

**Prior status (maint-2026-07-06):** ESCALATE-CRITICAL.
**Current status:** ACCEPTED-FORMALLY — ESCALATION RESOLVED.

TD-MAINT-THRESHOLD-CALIB-001 was resolved as RESOLVED-ACCEPTED by human decision on 2026-07-08. PR #382 (624bae3) added README § Known Limitations "Reassembly anomaly thresholds" subsection documenting `--overlap-threshold` (50), `--small-segment-threshold` (100), `--small-segment-max-bytes` (16), and `--out-of-window-threshold` (100) as uncalibrated engineering defaults with explicit rationale. CLI overridability is stated. A future calibration exercise against labelled ICS traffic remains a P3 backlog option.

This closes the ESCALATE-CRITICAL flag open since before v0.7.1 (12+ releases). The prior sweep's primary escalation recommendation has been satisfied.

**Assessment:** ACCEPTED-FORMALLY. Recommend formal ASM-003 with status ACCEPTED, cross-referencing PR #382.

---

### ASM-CAND-004 — Full pcap eager-load into Vec<RawPacket> is acceptable (NFR-VIO-001)

**Prior status (maint-2026-07-06):** STABLE KNOWN DEBT.
**Current status:** STABLE KNOWN DEBT — NO CHANGE.

NFR-VIO-001 remains OPEN-DEBT. No streaming refactor has been scoped.

**Assessment:** Stable. Recommend formal ASM-004.

---

### ASM-CAND-005 — TLS byte-0/byte-1 gate (0x16 0x03) is adequate (Smell #10)

**Prior status (maint-2026-07-06):** STABLE LOW.
**Current status:** STABLE — NO CHANGE.

Wave-72 STORY-160 (JSON enum casing + schema_version) updates output formatting only. The layering separation (gate → dispatch → reassembly) confirmed by ADR-011 is unchanged.

**Assessment:** Stable. Recommend formal ASM-005.

---

### ASM-CAND-006 — ARP binding table LRU eviction at capacity 65,536 is acceptable (ADR-008)

**Prior status (maint-2026-07-06):** STABLE — RISK POSTURE IMPROVED BY v0.11.4.
**Current status:** STABLE — NO CHANGE.

The `bindings_evicted` counter from PR #365 continues to surface evictions. PR #384's saturating_add changes improved counter discipline but did not touch eviction logic. Assumption design terms remain correct and documented in ADR-008.

**Assessment:** Stable. Recommend formal ASM-006.

---

### ASM-CAND-007 — DNP3 FIR=1-only parse is sufficient for v1 detections (ADR-007 Decision 4)

**Prior status (maint-2026-07-06):** MONITOR.
**Current status:** MONITOR — NO CHANGE.

No new DNP3 multi-fragment detections were added. v0.11.5 PR #370 added observability counters but did not change detection scope. Wave-72 stories are CI/docs/format only.

**Assessment:** Monitor. Re-evaluate if next ICS feature adds DNP3 multi-fragment detections. Recommend formal ASM-007.

---

### ASM-CAND-008 — DNP3 CRC skip is safe for PCAP replay of real captures (ADR-007 Decision 3)

**Prior status (maint-2026-07-06):** LOW — NO CHANGE.
**Current status:** LOW — CARRY FORWARD (THIRD SWEEP).

The Crain/Sistrunk caveat documentation fix to ADR-007 Decision 3 remains unactioned. No DNP3 parse-path code touched in this period.

**Assessment:** Low. Add Crain/Sistrunk CRC-caveat to ADR-007 Decision 3 in the next docs PR. Recommend formal ASM-008.

---

### ASM-CAND-009 — ARP storm rate default (50 frames/s) is a conservative engineering choice (ADR-008)

**Prior status (maint-2026-07-06):** ESCALATE — FAR PAST THRESHOLD.
**Current status:** ACCEPTED-FORMALLY — ESCALATION RESOLVED.

Bundled with ASM-CAND-003 in TD-MAINT-THRESHOLD-CALIB-001. PR #382 README § Known Limitations documents the 50 frames/second default as an engineering choice without OT-network reference guidance, with CLI overridability noted. The `storm_counters_evicted` counter (v0.11.4) provides ongoing observability. PR #384 ensured all arp.rs counter-increment sites use saturating_add.

**Assessment:** ACCEPTED-FORMALLY. Recommend formal ASM-009 with status ACCEPTED, cross-referencing PR #382.

---

### ASM-CAND-010 — Port-44818 classification is sufficient for EtherNet/IP TCP (ADR-010)

**Prior status (maint-2026-07-06):** STABLE (5 releases since introduction).
**Current status:** STABLE — NO CHANGE. 6 releases since introduction (add v0.11.5 + wave-72 develop).

Wave-72 and v0.11.5 deliver no ENIP protocol changes. PR #384 converted ENIP counter increments to saturating_add but did not change the dispatch gate or classification logic. IANA-exclusive port registration remains the practical discriminator.

**Assessment:** Stable. Recommend formal ASM-010. Flag dedicated ENIP gate Kani proof for next ENIP hardening cycle (REC-007, unchanged from prior sweep).

---

### ASM-CAND-011 — TLS handshake-only reassembly is sufficient for v1 TLS detections (ADR-011)

**Prior status (maint-2026-07-06):** STABLE (4 releases since introduction).
**Current status:** STABLE — NO CHANGE. 5 releases since introduction (add v0.11.5).

Wave-72 STORY-160 includes no TLS analyzer changes. VP-039 and VP-040 remain in force. BC-INDEX v2.22 maintains the handshake-scope invariant.

**Assessment:** Stable. Re-evaluate if a future TLS feature requires post-handshake record analysis. Recommend formal ASM-011.

---

## Section 3 — Informal Risks (R-NNN Candidates)

### R-CAND-001 — Unbounded weak-cipher evidence Vec (O-06 / NFR-RES-023) [PERSISTING]

**Prior status (maint-2026-07-06):** MEDIUM (P1), GitHub #102 open.
**Current status:** MEDIUM (P1) — STILL OPEN; NO ACTION IN 6+ MONTHS.

NFR-RES-023 remains OPEN. No `MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap has shipped. The `tls.rs` weak-cipher evidence Vec remains data-dependent-cardinality (upper bound ~9,216 entries, worst-case heap ~270–500 KB). PF-001 (PR #384) converted counter sites to saturating_add but the weak-cipher evidence Vec is a collection structure, not a counter — it was not in scope. Wave-72 STORY-150 (TLS carry-path) has not yet shipped. GitHub #102 remains open.

**Assessment:** MEDIUM (P1). Three sweeps without action. Must be addressed in the next TLS-touching story. Recommend formal R-001.

---

### R-CAND-002 — README "multi-GB captures" claim vs. eager Vec<RawPacket> load (NFR-VIO-001) [PERSISTING]

**Prior status (maint-2026-07-06):** LOW-MEDIUM.
**Current status:** LOW-MEDIUM — NO CHANGE.

NFR-VIO-001 OPEN-DEBT unchanged. Wave-72 doc PRs (#388, #390) did not address this README language gap.

**Assessment:** Low-Medium. Bundle with next documentation cleanup. Recommend formal R-002.

---

### R-CAND-003 — Single-platform CI / ubuntu-latest only (NFR-VIO-010 / NFR-PORT-001) [PERSISTING]

**Prior status (maint-2026-07-06):** MEDIUM.
**Current status:** MEDIUM — NO CHANGE.

CI still runs only on ubuntu-latest. Wave-72 CI hardening (PR #391 action-pin-gate, PR #387 CHANGELOG gate) improves supply-chain security but does not add macOS/Windows matrix jobs. Cross-compile release matrix (4 platforms) continues to catch build-time failures only.

**Assessment:** Medium. Recommend formal R-003.

---

### R-CAND-004 — rayon unused dependency [RESOLVED, maint-2026-06-22]

**Status:** RESOLVED — PR #304. No further action.

---

### R-CAND-005 — Port-502 false-routing risk for non-Modbus binary protocols (ACCEPTED) [PERSISTING]

**Prior status (maint-2026-07-06):** ACCEPTED KNOWN RISK.
**Current status:** ACCEPTED KNOWN RISK — NO CHANGE.

VP-022 Kani-verified three-point gate remains in force. Recommend formal R-005 with status ACCEPTED.

---

### R-CAND-006 — DNP3 T0827 emission threshold misconfiguration risk (ADR-007) [RESOLVED-ACCEPTED]

**Prior status (maint-2026-07-06):** MEDIUM (P2) — AGING.
**Current status:** ACCEPTED-FORMALLY (PR #382).

Bundled into TD-MAINT-THRESHOLD-CALIB-001 resolution. PR #382 README § Known Limitations "DNP3 direct-operate burst threshold" subsection documents the default 10 as a research-bounded engineering choice (5–20 range per dnp3-research.md §5.1) with written rationale. No ongoing escalation required.

**Assessment:** ACCEPTED-FORMALLY. Recommend formal R-006 with status ACCEPTED, cross-referencing PR #382.

---

### R-CAND-007 — RUSTSEC-2026-0097 transitive rand 0.8.5 [RESOLVED, maint-2026-06-22]

**Status:** RESOLVED — PR #304. No further action.

---

### R-CAND-008 — VLAN/QinQ/MACsec ARP offset detection limitation (ACCEPTED) [PERSISTING]

**Prior status (maint-2026-07-06):** ACCEPTED KNOWN LIMITATION.
**Current status:** ACCEPTED KNOWN LIMITATION — NO CHANGE.

MACsec ARP limitation (CWE-693) correctly documented by design (STORY-117, E-17). PR #382 did not add a MACsec entry to README § Known Limitations. REC-006 (one sentence to README) unactioned for three sweeps.

**Assessment:** Accepted. Batch REC-006 into next docs PR. Recommend formal R-008 with status ACCEPTED.

---

### R-CAND-009 — Memory-DoS for DNP3/ENIP per-flow state (CWE-401+CWE-770) [RESOLVED, maint-2026-07-06]

**Status:** RESOLVED in v0.11.3 (PR #362). Issue #342 CLOSED. No further action.

---

### R-CAND-010 — Unsafe split-borrow in `src/analyzer/enip.rs` (SEC-001) [PERSISTING]

**Prior status (maint-2026-07-06):** LOW (P2), OPEN.
**Current status:** LOW (P2) — OPEN, NO CHANGE.

Unsafe self/flows split-borrow in `enip.rs::on_data` PDU dispatch loop remains present (tech-debt-register SEC-001). The `for pdu in pdu_queue` loop (lines 992–999) derives a `*mut EnipFlowState` raw pointer from `self.flows.get_mut(&flow_key)`, then calls `self.process_pdu(unsafe { &mut *flow_ptr }, ...)`, creating simultaneous aliasing between `&mut self` and the raw pointer into `self.flows`. Note: the carry-buffer select at lines 825–829 is already safe (`std::mem::take`). PF-001 (PR #384) converted counter increments but did not touch the PDU dispatch borrow pattern. Sound under the stated invariant that `process_pdu` never accesses `self.flows`; risk is fragility under future `process_pdu` or `EnipAnalyzer` refactoring. Absorbed into STORY-181 (wave-85, drafted D-494).

**Assessment:** Low (P2). Target: wave-85 / STORY-181. Recommend formal R-010 with status OPEN/ACCEPTED-PENDING-REFACTOR.

---

### R-CAND-011 — Port-44818 false-routing risk for non-EtherNet/IP binary protocols (ADR-010) [PERSISTING]

**Prior status (maint-2026-07-06):** LOW, ACCEPTED.
**Current status:** LOW, ACCEPTED — NO CHANGE.

IANA-exclusive port registration remains the practical discriminator. No new ENIP-touching code shipped. Absent dedicated Kani harness (REC-007) remains flagged for next ENIP hardening cycle.

**Assessment:** Accepted. Recommend formal R-011 with status ACCEPTED.

---

### R-CAND-012 — `rebind_count` arithmetic non-saturating in `src/analyzer/arp.rs` [RESOLVED]

**Prior status (maint-2026-07-06):** LOW/INFORMATIONAL.
**Current status:** RESOLVED — PR #384 (c4eb1f4, 2026-07-08).

Folded into PF-001 sweep. `entry.rebind_count` (u32, `arp.rs:856`) converted to `saturating_add(1)` alongside all other arp.rs counter sites. NFR-REL-003 saturating arithmetic convention now uniformly applied across all 14 arp.rs counter sites.

**Assessment:** RESOLVED. Record as formal R-012 with status RESOLVED, closed by PR #384.

---

## Section 4 — Prior Items: ASM-CAND-003/009 and R-CAND-006 Register Status Verification

Per task: verify that register status for ASM-CAND-003, ASM-CAND-009, and R-CAND-006 reflects PR #382 formal acceptance and TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED disposition.

| Item | Prior Status | PR #382 Action | Current Status | Register Reflects? |
|------|-------------|----------------|----------------|--------------------|
| ASM-CAND-003 | ESCALATE-CRITICAL | README § Known Limitations "Reassembly anomaly thresholds" added | ACCEPTED-FORMALLY | YES |
| ASM-CAND-009 | ESCALATE | README § Known Limitations "ARP storm rate" added | ACCEPTED-FORMALLY | YES |
| R-CAND-006 | MEDIUM P2 / AGING | README § Known Limitations "DNP3 direct-operate burst threshold" added | ACCEPTED-FORMALLY | YES |
| TD-MAINT-THRESHOLD-CALIB-001 | P1 OPEN | Resolved as RESOLVED-ACCEPTED (human decision 2026-07-08) | RESOLVED-ACCEPTED | YES |

**Verdict:** All three items correctly reflect their ACCEPTED status. Note: tech-debt register footer P1 candidates note still lists TD-MAINT-THRESHOLD-CALIB-001 alongside TD-MAINT-RISK-REGISTRY-BACKFILL — the footer is stale (THRESHOLD-CALIB-001 resolved); update tracked as REC-010.

---

## Section 5 — Mitigation Validity Check Against Current Architecture

### Check: v0.11.5 release (PRs #369, #370, #371, #382, #383, #384) impact on mitigations

| Change | Impacted mitigation? | Verdict |
|--------|---------------------|---------|
| PR #369 (doc-drift fixes) | Documentation only | NOT INVALIDATED |
| PR #370 (DNP3 dropped_findings / observability counters) | Adds counters; dispatch gate unchanged | NOT INVALIDATED |
| PR #371 (crossbeam-epoch 0.9.18→0.9.20) | Dev-dep only; zero runtime footprint | NOT INVALIDATED |
| PR #382 (README Known Limitations) | Documentation / formal acceptance | NOT INVALIDATED (strengthens ASM-003/009, R-006) |
| PR #383 (SEC-010/011: debug_assert + anti-gameability) | Test-code guards; no production path changes | NOT INVALIDATED |
| PR #384 (PF-001: 109 saturating_add conversions) | Counter discipline improvement; no detection logic changed | NOT INVALIDATED |

### Check: Wave-72 delivery (PRs #386–#391) impact on mitigations

| Change | Impacted mitigation? | Verdict |
|--------|---------------------|---------|
| PR #386 (indicatif 0.18.5→0.18.6) | Cosmetic dep bump; zero analyzer surface | NOT INVALIDATED |
| PR #387 (CHANGELOG gate + CI scan-guard hardening) | Pure CI; no product-code changes | NOT INVALIDATED |
| PR #388 (ADR-012 protocols catalog docs) | Documentation of existing SS-18; no new assumptions | NOT INVALIDATED |
| PR #389 (JSON enum casing + schema_version) | Output format change (reporter layer only); detection logic unchanged | NOT INVALIDATED |
| PR #390 (VP-024 proof_file_hash + Multi-File Proof Anchor docs) | Documentation of proof algorithm; no code changes | NOT INVALIDATED |
| PR #391 (action-pin-gate scan guard hardening) | Pure CI supply-chain hardening | NOT INVALIDATED |

**Verdict:** No existing mitigation was invalidated by v0.11.5 or wave-72 delivery.

---

## Section 6 — Invalidated Assumption / Missing Risk Escalation Check

No design assumption was invalidated during this sweep period. The product did not add new protocol analyzers, modify dispatch gates, or change detection thresholds. No new ICS protocol feature cycle is in progress. No new R-NNN item is required as a consequence of any assumption change. The ASM-CAND-003/009 formal acceptance represents the closing of long-running escalation items, not an invalidation.

---

## Section 7 — Resolved Since Prior Sweep (maint-2026-07-06)

| Item | Resolution | PR / Release |
|------|-----------|-------------|
| ASM-CAND-003: anomaly thresholds (ESCALATE-CRITICAL → ACCEPTED) | README Known Limitations + written rationale; TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED | PR #382 (624bae3), 2026-07-08 |
| ASM-CAND-009: ARP storm rate (ESCALATE → ACCEPTED) | README Known Limitations + written rationale; bundled with ASM-003 | PR #382 (624bae3), 2026-07-08 |
| R-CAND-006: DNP3 T0827 threshold (MEDIUM/AGING → ACCEPTED) | README Known Limitations + written rationale; bundled with ASM-003/009 | PR #382 (624bae3), 2026-07-08 |
| R-CAND-012: rebind_count non-saturating (INFORMATIONAL → RESOLVED) | Folded into PF-001 sweep; arp.rs:856 converted to saturating_add | PR #384 (c4eb1f4), 2026-07-08 |

---

## Section 8 — Prioritized Recommendations

| Priority | ID | Action | Effort | Status change |
|----------|----|--------|--------|---------------|
| HIGH | REC-001 | Complete TD-MAINT-RISK-REGISTRY-BACKFILL: create `specs/risk-register.md` (R-001..R-012) and `specs/assumptions.md` (ASM-001..ASM-011) before the next ICS protocol feature cycle. Three sweeps at P1 without action; wave-72 closed is the ideal window. | M (1 session) | P1 — 3rd sweep without action |
| HIGH | REC-003 | R-CAND-001 (weak-cipher heap, NFR-RES-023, GitHub #102): add `MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap with "+N more" annotation in the next TLS-touching story. P1 for 6+ months, not addressed by PF-001 or wave-72. | S | Unchanged (P1, 6+ months) |
| MEDIUM | REC-004 | R-CAND-010 (SEC-001 unsafe `*mut EnipFlowState` split-borrow at enip.rs:992-999): refactor to safe take-remove-reinsert pattern (self.flows.remove before the PDU dispatch loop, insert after); superseded by STORY-181 (wave-85). | S | Carry forward P2 |
| LOW | REC-005 | ASM-CAND-008 / ADR-007 Decision 3: add Crain/Sistrunk CRC-caveat as normative note. Three sweeps unactioned. | XS | Carry forward (3rd sweep) |
| LOW | REC-006 | R-CAND-008 (MACsec ARP limitation): add one sentence to README § Known Limitations referencing STORY-117. Three sweeps unactioned. Bundle with next docs PR. | XS | Carry forward (3rd sweep) |
| LOW | REC-007 | R-CAND-011 (Port-44818 ENIP gate, no dedicated Kani proof): add `classify_enip_gate` Kani harness analogous to VP-022/VP-023 in the next ENIP hardening cycle. v0.12.0 candidate. | S | Carry forward |
| INFORMATIONAL | REC-009 | VSDD consistency criteria 42–50 remain unverifiable without a formal registry. Ensure STATE.md DRIFT items reflect this persisting gap. | XS | Unchanged |
| INFORMATIONAL | REC-010 | Tech-debt register footer P1 candidates note stale: still lists resolved TD-MAINT-THRESHOLD-CALIB-001. Update footer to list only TD-MAINT-RISK-REGISTRY-BACKFILL as current P1. | XS | NEW |

---

## Section 9 — Conclusion

wirerust does not maintain a formal ASM-NNN / R-NNN risk register. Informal tracking continues to function but the governance gap has widened over three consecutive sweeps without action.

**Positive developments since maint-2026-07-06:** TD-MAINT-THRESHOLD-CALIB-001 was resolved via formal acceptance (PR #382), closing the two longest-running escalation flags (ASM-CAND-003 and ASM-CAND-009) and converting R-CAND-006 to accepted. R-CAND-012 was resolved by the PF-001 saturating_add sweep (PR #384). Wave-72 delivered 4 stories without introducing new protocol assumptions or invalidating any existing mitigation. ARCH-INDEX v2.12 is unchanged — the architecture portfolio is stable.

**Remaining open concerns of substance:**
1. **TD-MAINT-RISK-REGISTRY-BACKFILL (P1, three sweeps)** — with no active feature cycle, this is the ideal window.
2. **R-CAND-001 (P1, 6+ months)** — weak-cipher evidence Vec is the only remaining data-bounded risk with no observable signal at worst-case cardinality; STORY-150 is the closest candidate vehicle.
3. **Three documentation carry-forwards (LOW, XS each)** — REC-005 (ADR-007 Crain/Sistrunk), REC-006 (README MACsec), REC-010 (tech-debt register footer) — all waiting for the next docs PR.
