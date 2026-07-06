---
document_type: maintenance-sweep-report
sweep: risk-assumption-monitoring
sweep_number: 11
run_id: maint-2026-07-06
pipeline: STEADY_STATE
product: wirerust
released_version: v0.11.4
date: 2026-07-06
trigger: scheduled
producer: consistency-validator
read_only: true
---

# Maintenance Sweep 11 — Risk and Assumption Monitoring

**Run ID:** maint-2026-07-06
**Date:** 2026-07-06
**Latest release:** v0.11.4 (released 2026-07-06); prior releases: v0.11.2 (2026-07-05), v0.11.3 (2026-07-06), v0.11.4 (2026-07-06).
**Baseline for this sweep:** maint-2026-06-17 (v0.7.1); the prior risk-assumption-monitoring report is the only prior sweep of this type.
**Scope:** L2 Domain Spec shards, PRD, ADRs (ADR-001..ADR-012), NFR catalog, domain-debt register, tech-debt register, STATE.md.

---

## Executive Summary

The wirerust factory **still has no formal ASM-NNN / R-NNN registry**. Tech-debt item
TD-MAINT-RISK-REGISTRY-BACKFILL remains DEFERRED since maint-2026-06-17. All risk and
assumption content is still distributed informally across O-NNN (domain-debt), NFR-VIO-NNN
(nfr-catalog), ADR Consequences sections, and tech-debt register entries.

Since the prior sweep (maint-2026-06-17, baseline v0.7.1), four significant architectural
additions occurred: EtherNet/IP analyzer (ADR-010, v0.11.0), TLS handshake reassembly
(ADR-011, v0.11.1), protocol coverage catalog / SS-18 (ADR-012, v0.11.2), and a
memory-DoS fix for DNP3/ENIP unbounded per-flow state (PR #362, v0.11.3). Observability
counters (`bindings_evicted`, `storm_counters_evicted`, `dropped_transactions`,
`dropped_map_entries`) shipped in v0.11.4 (PR #365).

**Two prior-sweep risks were fully resolved** between sweeps (R-CAND-004 rayon dep,
R-CAND-007 RUSTSEC-2026-0097). One significant new risk was identified and resolved within
the same maintenance session (R-CAND-009, memory-DoS CWE-401+CWE-770, PR #362 v0.11.3).
Two new architectural assumptions were introduced by ADR-010 and ADR-011 (ASM-CAND-010,
ASM-CAND-011).

The **two highest-priority open assumptions** (ASM-CAND-003, anomaly threshold calibration;
ASM-CAND-009, ARP storm rate default) have now been unvalidated for well past the 2-release
escalation threshold — ASM-CAND-009 was already at the threshold at v0.7.1 and has since
accumulated 4+ additional releases. Both require formal disposition before the next ICS
protocol feature cycle. No existing mitigation was invalidated by any v0.11.x architectural
addition.

**Summary counts:**

| Category | Total | Still open / unvalidated | Resolved since prior sweep | Escalation-worthy |
|----------|-------|--------------------------|---------------------------|-------------------|
| Informal assumptions (ASM-CAND) | 11 | 8 | 0 (none newly validated) | 3 |
| Informal risks (R-CAND) | 12 | 7 | R-CAND-004, R-CAND-007, R-CAND-009 (3) | 2 |
| Formal ASM-NNN identifiers | 0 | — | — | — |
| Formal R-NNN identifiers | 0 | — | — | — |

**Gate result:** ADVISORY. No formal registry exists; no gate can pass or fail.
TD-MAINT-RISK-REGISTRY-BACKFILL is 2 maintenance sweeps old without action — now approaching
HIGH priority, as EtherNet/IP and SS-18 have added 2 new untracked design assumptions.

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Current: NEEDS_ATTENTION**

Three risks resolved (R-CAND-004, R-CAND-007, R-CAND-009), observability improved by
v0.11.4 counters. However, two threshold assumptions (ASM-CAND-003, ASM-CAND-009) are
past escalation threshold with no formal disposition, the risk registry backfill remains
DEFERRED at P2 (recommended promotion to P1), and R-CAND-001 (weak-cipher heap) has been
open for 6+ months without a truncation cap. The product is structurally sound — no
mitigation has been invalidated and no memory-DoS is present — but governance items are aging.

---

## Dependency Audit

> N/A for this sub-sweep (Sweep 11 = risk-assumption-monitoring). Dependency auditing is
> covered by Sweep 1 (dependency-audit) in the same maint-2026-07-06 cycle.

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| — | — | None new this sweep | — | — | See Sweep 1 |

---

## Documentation Drift

> N/A for this sub-sweep (Sweep 11 = risk-assumption-monitoring). Documentation drift is
> covered by Sweep 8 (doc-drift) in the same maint-2026-07-06 cycle. RAM-specific doc
> gaps are tracked below in ASM-CAND-008 (ADR-007 Decision 3 Crain/Sistrunk caveat) and
> R-CAND-008 (README MACsec limitation).

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| ADR-007 Decision 3 | Crain/Sistrunk caveat | missing normative note | low | REC-005 (XS, next doc PR) |
| README | Known Limitations | missing MACsec ARP limitation | low | REC-006 (XS, next doc PR) |

---

## Pattern Consistency

> N/A for this sub-sweep (Sweep 11 = risk-assumption-monitoring). Pattern consistency is
> covered by Sweep 7 (pattern-consistency) in the same maint-2026-07-06 cycle.

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| — | — | — | — | See Sweep 7 |

---

## Holdout Scenario Freshness

> N/A for this sub-sweep (Sweep 11 = risk-assumption-monitoring). Holdout freshness is
> covered by Sweep 4 (holdout-freshness) in the same maint-2026-07-06 cycle.

| Metric | Value |
|--------|-------|
| Total scenarios | covered by Sweep 4 |
| Still valid | covered by Sweep 4 |
| Stale (intentional change) | covered by Sweep 4 |
| Missing coverage | covered by Sweep 4 |

---

## Performance Baseline

> N/A for this sub-sweep (Sweep 11 = risk-assumption-monitoring). Performance baseline is
> covered by Sweep 5 (performance-baseline) in the same maint-2026-07-06 cycle.

| Benchmark | Previous | Current | Delta | Status |
|-----------|----------|---------|-------|--------|
| — | — | — | — | See Sweep 5 |

---

## Trend (Last 5 Sweeps)

RAM sweep history (prior runs of Sweep 11):

| Date | ASM open/unvalidated | R open | Escalation-worthy | Resolved | Registry status |
|------|---------------------|--------|-------------------|----------|-----------------|
| 2026-06-17 (maint-2026-06-17) | 7 | 5 | 2 (ASM-003, ASM-009) | — (baseline) | DEFERRED P2 |
| 2026-07-06 (maint-2026-07-06) | 8 | 7 | 3 (ASM-003, ASM-009, RAM-001) | R-004, R-007, R-009 | DEFERRED P2→P1 recommended |

Prior sweep dates before maint-2026-06-17: no prior RAM sweeps on record (maint-2026-06-17 was
the first run of Sweep 11 for this product).

---

## Section 1 — Formal ASM/R Registry Status

### Finding RAM-001 (MEDIUM→approaching HIGH) — No formal ASM-NNN / R-NNN registry (PERSISTING, AGED)

The finding from the maint-2026-06-17 sweep is unchanged. No `specs/risk-register.md` or
`specs/assumptions.md` has been created. `grep -rn "ASM-[0-9]" .factory/specs/` and
`grep -rn "R-[0-9][0-9][0-9]" .factory/specs/` return zero results.

Tech-debt item TD-MAINT-RISK-REGISTRY-BACKFILL remains DEFERRED since maint-2026-06-17.
Since that sweep, three additional ICS-relevant design assumptions have accrued (ADR-010
port-44818, ADR-011 TLS-reassembly scope, ADR-012 static catalog accuracy) without
formal registration. VSDD criteria 42-50 remain structurally unverifiable.

**Updated recommended action:** Promote TD-MAINT-RISK-REGISTRY-BACKFILL from P2 to P1.
Backfill `specs/risk-register.md` with R-001..R-012 and `specs/assumptions.md` with
ASM-001..ASM-011 before the next ICS protocol feature cycle. This now covers 11 ASM-CAND
and 12 R-CAND entries (up from 9+8 in the prior sweep).

---

## Section 2 — Informal Assumptions (ASM Candidates)

### ASM-CAND-001 — Port-only classification is sufficient for Modbus TCP (ADR-005 Decision 1)

**Prior status (maint-2026-06-17):** STABLE.
**Current status:** STABLE — NO CHANGE.

Modbus dispatch (port 502 + three-point validity gate: Protocol ID 0x0000, Length 2..254,
plausible FC) is unaffected by ADR-010 (port 44818, separate dispatcher arm), ADR-011 (TLS
reassembly, different subsystem entirely), and ADR-012 (static catalog, read-only pure-core).
VP-022 Kani proof remains in force. No Modbus-touching story shipped since v0.7.1.

**Assessment:** Stable. Recommend formal ASM-001. No escalation required.

---

### ASM-CAND-002 — Port-only classification is sufficient for DNP3 TCP (ADR-007 Decision 1)

**Prior status (maint-2026-06-17):** STABLE.
**Current status:** STABLE — POSITIVE DEVELOPMENT.

DNP3 dispatch (port 20000 + sync-word 0x0564 gate) is unaffected by v0.11.x additions.
The memory-DoS fix (PR #362, v0.11.3) corrected a missing `on_flow_close` wiring — it
fixed the state lifecycle, not the dispatch or classification gate. VP-023 Kani proof
remains in force. DRIFT-DNP3-DIRECTION-001 remains deferred but does not affect this
classification assumption.

**New context:** PR #362 added `Dnp3Analyzer::on_flow_close`, ensuring per-flow state is
purged on close. The classification assumption was always sound; the code gap was in
lifecycle management, not routing. The assumption's mitigations are strengthened.

**Assessment:** Stable. Recommend formal ASM-002. No escalation required.

---

### ASM-CAND-003 — Anomaly thresholds are adequate for forensic use without labelled corpus (O-03)

**Prior status (maint-2026-06-17):** ESCALATE (already past 2-release threshold).
**Current status:** ESCALATE-CRITICAL — NO VALIDATION PERFORMED ACROSS 10+ ADDITIONAL RELEASES.

O-03 remains open from v0.1.0 through v0.11.4. `ReassemblyConfig` thresholds
(`overlap_alert_threshold=50`, `small_segment_alert_threshold=100`,
`small_segment_max_bytes=16`, `out_of_window_alert_threshold=100`) remain
research-documented but empirically uncalibrated. CLI overridability remains the only
mitigation. The port-coverage feature cycle (v0.11.2) added a new capability surface but
did not validate any reassembly thresholds.

The 2-release escalation threshold was first crossed before v0.7.1. The assumption has
now been unvalidated across 12+ releases with no disposition record.

**Assessment:** ESCALATE-CRITICAL. Formal disposition required before the next ICS
protocol feature. Either (a) formally accept the assumption as a known limitation with
written rationale (research-cited, CLI-overridable) and record in README § Known
Limitations, OR (b) scope a threshold-calibration exercise against labelled ICS traffic
corpus post-v0.12.0. Neither option has occurred across any of the 12 releases.
Promote to P1. Bundle with ASM-CAND-009 and R-CAND-006 in a single calibration track.

---

### ASM-CAND-004 — Full pcap eager-load into Vec<RawPacket> is acceptable (NFR-VIO-001)

**Prior status (maint-2026-06-17):** STABLE KNOWN DEBT.
**Current status:** STABLE KNOWN DEBT — NO CHANGE.

NFR-VIO-001 remains OPEN-DEBT. No streaming refactor has been scoped. The pcapng reader
(ADR-009) uses the same all-in-memory model and correctly declares its RSS bound (NFR-PERF-005:
peak RSS <= pcapng_file_size × 2.0, validated by criterion bench in the pcapng F6 cycle).
The assumption is explicitly documented and accepted.

**Assessment:** Stable. Recommend formal ASM-004. No escalation required.

---

### ASM-CAND-005 — TLS byte-0/byte-1 gate (0x16 0x03) is adequate (Smell #10)

**Prior status (maint-2026-06-17):** STABLE LOW.
**Current status:** STABLE LOW — NOT INVALIDATED by ADR-011.

ADR-011 (TLS handshake reassembly) adds a multi-fragment reassembly path on top of the
existing TLS gate. The 2-byte gate fires at the content-classification layer (dispatcher);
reassembly occurs after a flow is routed to the TLS analyzer. ADR-011 does not alter the
gate condition itself.

The Smell #10 risk surface is zero in ARP/DNP3/Modbus/ENIP contexts — all four ICS
analyzers bypass the content-first gate entirely (ARP via DecodedFrame::Arp; DNP3/Modbus/ENIP
via port-fallback rules that fire before content classification). This is now confirmed for
four ICS analyzers, up from three at v0.7.1.

**Assessment:** Stable. The layering separation (gate → dispatch → reassembly) confirmed
by ADR-011 architecture. Recommend formal ASM-005. No escalation required.

---

### ASM-CAND-006 — ARP binding table LRU eviction at capacity 65,536 is acceptable (ADR-008)

**Prior status (maint-2026-06-17):** STABLE.
**Current status:** STABLE — RISK POSTURE IMPROVED BY v0.11.4.

v0.11.4 (PR #365, D-385) added the `bindings_evicted` observability counter to
`ArpAnalyzer`. LRU evictions are now surfaced in `AnalysisSummary.detail` and JSON output.
This directly addresses the previous observability gap that made the eviction assumption's
impact invisible to forensic analysts.

The design assumption (65,536 capacity; LRU eviction not cryptographically safe; acceptable
for offline forensics) remains correct and documented in ADR-008. The `bindings_evicted`
counter enables analysts to detect high-eviction captures where the assumption's impact
may be significant. BC-2.16.008 and BC-2.16.010 were amended (BC-INDEX v2.18) to formalize
the counter behavior.

**Assessment:** Stable with improved observability. Recommend formal ASM-006 with an updated
note that `bindings_evicted` counter (v0.11.4) makes eviction events visible to analysts.
No escalation required.

---

### ASM-CAND-007 — DNP3 FIR=1-only parse is sufficient for v1 detections (ADR-007 Decision 4)

**Prior status (maint-2026-06-17):** MONITOR.
**Current status:** MONITOR — NO CHANGE.

No new DNP3 detections requiring application-layer fragment continuity have been added since
v0.7.1. The memory-DoS fix (PR #362) corrected `on_flow_close` wiring without changing
detection scope. EtherNet/IP and protocol-coverage features are orthogonal to DNP3 parse
depth.

STORY-148 (drafted as the SEC-005/SEC-006 fix vehicle, now reconciliation-pending after
PR #362 resolved those findings out-of-cycle) will need scoping; however, even if re-scoped
to add DNP3 flow-count metrics or eviction hardening, that would not touch the FIR=1 detection
assumption.

**Assessment:** Monitor. Still valid for current scope. If the next ICS feature adds DNP3
multi-fragment detections (e.g., large WRITE_SINGLE or multi-frame OPERATE), this assumption
must be re-evaluated. Recommend formal ASM-007.

---

### ASM-CAND-008 — DNP3 CRC skip is safe for PCAP replay of real captures (ADR-007 Decision 3)

**Prior status (maint-2026-06-17):** LOW.
**Current status:** LOW — NO CHANGE.

No new DNP3 behavior has shipped since v0.7.1 that affects CRC handling. The memory-DoS
fix (PR #362) operated on the `on_flow_close` and `summarize()` lifecycle path, not the
parse path. The Crain/Sistrunk caveat documented in
`.factory/research/dnp3-f2-scope-threshold-validation.md` ("CRC validation would not have
caught Crain/Sistrunk frames anyway") remains an unwritten normative note not propagated
to ADR-007 Decision 3.

**Assessment:** Low. The documentation-only fix (add Crain/Sistrunk caveat to ADR-007
Decision 3) was recommended in the prior sweep and remains unactioned — carry forward.
Recommend formal ASM-008.

---

### ASM-CAND-009 — ARP storm rate default (50 frames/s) is a conservative engineering choice (ADR-008)

**Prior status (maint-2026-06-17):** MONITOR (approaching 2-release escalation threshold
at v0.7.1; 2 releases old at time of prior sweep).
**Current status:** ESCALATE — FAR PAST THRESHOLD.

Since v0.7.1, where this assumption was 2 releases old and "approaching escalation
threshold," 6+ additional releases have shipped without calibration. The assumption has
been unvalidated for the full v0.7.x..v0.11.4 release span.

v0.11.4 context: PR #365 (D-385) added the `storm_counters_evicted` counter, which surfaces
ARP storm state evictions from the per-source LRU (BC-2.14.012 v1.1, BC-2.14.021 v1.2).
While this improves observability of storm state overflow, it does not calibrate the 50/s
threshold itself. The `storm_counters_evicted` counter provides new telemetry that could
support retroactive validation — analysts can now observe how often storm state is evicted
in legitimate captures. However, that path from observation to validated threshold has not
been walked.

**Assessment:** ESCALATE. Same class as ASM-CAND-003 — long-standing engineering default
without empirical calibration, now well past the escalation threshold. The `storm_counters_evicted`
counter is a meaningful new validation aid but not a substitute for calibration. Promote to P1.
Bundle with ASM-CAND-003 and R-CAND-006 in a unified threshold-calibration action track.
Recommend formal ASM-009 with the counter noted as a validation aid.

---

### ASM-CAND-010 — Port-44818 classification is sufficient for EtherNet/IP TCP (ADR-010) — NEW SINCE PRIOR SWEEP

**Source:** ADR-010, Context (port assignment) and Decision 1 (three-layer parse structure).
**Assumption:** Port 44818 (IANA-registered exclusively for EtherNet/IP) combined with an
ENIP command-code validity gate (0x0065 RegisterSession, 0x0066 UnRegisterSession, 0x006F
SendRRData, 0x0070 SendUnitData; non-zero session handle for 0x006F/0x0070) provides
adequate discrimination from non-ENIP binary protocols on that port.

**Status as of v0.11.4:** VALID. EtherNet/IP analyzer shipped in v0.11.0. Per-flow state
lifecycle was corrected in v0.11.3 (PR #362: `on_flow_close` now properly purges ENIP state,
eliminating the previously unbounded memory risk). VP-041/042/043 verify catalog-layer
correctness at the protocol coverage layer. No regression in v0.11.3 or v0.11.4. The
ENIP gate is more complex than Modbus/DNP3 (two-byte command code from a sparse set, plus
session-handle check for Send commands) — this is documented in ADR-010.

**Note:** Unlike Modbus VP-022 and DNP3 VP-023, there is no Kani proof for the ENIP
port-44818 dispatch gate specifically. The classify_oracle VP-004 harness was updated to
include the ENIP arm, providing coverage at the dispatch-classification level, but a
dedicated gate-validity Kani harness does not exist. This is flagged in REC-007.

**Releases since assumption introduced:** v0.11.0, v0.11.1, v0.11.2, v0.11.3, v0.11.4 (5 releases).
**Assessment:** Stable. IANA-exclusive port registration reduces false-routing likelihood.
Recommend formal ASM-010. No immediate escalation; flag for next ENIP hardening cycle
(Kani proof for gate validity).

---

### ASM-CAND-011 — TLS handshake-only reassembly is sufficient for v1 TLS detections (ADR-011) — NEW SINCE PRIOR SWEEP

**Source:** ADR-011 design decisions; NFR-RES-021 (handshake short-circuit).
**Assumption:** Reassembling only the TLS handshake (ClientHello + ServerHello) is sufficient
for all v1 TLS security detections. Post-handshake record data is discarded after both
`client_hello_seen` and `server_hello_seen` are set. No detection-relevant signal is missed
by this early-exit design.

**Status as of v0.11.4:** VALID. ADR-011 shipped in the fix-tls-clienthello-frag cycle
(v0.11.1). VP-039 (TLS fragment reassembly, 3 Kani proofs PASS) and VP-040 (ClientHello
reconstruction) both verified in F6. Holdout: 8/8 must-pass scenarios satisfied. BC-INDEX
v2.3 (post-F5) and subsequent amendments through v2.18 maintain the handshake-scope invariant.
This assumption directly parallels ASM-CAND-007 (DNP3 FIR=1 only) — both bound the analysis
to the first meaningful signaling exchange.

**Releases since assumption introduced:** v0.11.1, v0.11.2, v0.11.3, v0.11.4 (4 releases).
**Assessment:** Stable. If a future TLS feature requires post-handshake record analysis
(e.g., session ticket analysis, extended master secret detection), this assumption must be
revisited. Recommend formal ASM-011.

---

## Section 3 — Informal Risks (R-NNN Candidates)

### R-CAND-001 — Unbounded weak-cipher evidence Vec (O-06 / NFR-RES-023) [PERSISTING]

**Prior status (maint-2026-06-17):** MEDIUM (P1), GitHub #102 open.
**Current status:** MEDIUM (P1) — STILL OPEN; NOT CAUGHT BY SILENT-LIMIT AUDIT.

NFR-RES-023 remains OPEN. No `MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap has shipped.
The `tls.rs` weak-cipher evidence Vec (`src/analyzer/tls.rs:497-516`) remains
data-dependent-cardinality (upper bound ~9,216 entries, worst-case heap ~270-500 KB).

The v0.11.4 silent-limit audit (D-385, 13 sites) confirmed `dropped_map_entries` for TLS/HTTP
distribution maps (SNI counts, session-ID lengths, URI counts, header-value counts) but the
**weak-cipher evidence Vec** is a distinct data structure and was not among the 4 resolved
gaps. The `dropped_map_entries` counter does NOT cover the weak-cipher evidence Vec. GitHub
#102 remains open.

**Assessment:** MEDIUM (P1). The silent-limit audit that shipped 4 counters did not address
this item. This is the one remaining data-bounded risk with no observable signal when the
worst-case path fires. Should be captured in the next TLS-touching story or silent-limit
follow-up pass. Recommend formal R-001.

---

### R-CAND-002 — README "multi-GB captures" claim vs. eager Vec<RawPacket> load (NFR-VIO-001) [PERSISTING]

**Prior status (maint-2026-06-17):** LOW-MEDIUM.
**Current status:** LOW-MEDIUM — NO CHANGE.

NFR-VIO-001 OPEN-DEBT is unchanged. The pcapng reader (ADR-009) correctly declares its
RSS bound (NFR-PERF-005), but the front-page README language still may overstate capability
for classic pcap files without clarifying the ~1.5x file-size RAM requirement.

**Assessment:** Low-Medium. Trivial fix. Bundle with next documentation cleanup. Recommend
formal R-002.

---

### R-CAND-003 — Single-platform CI / ubuntu-latest only (NFR-VIO-010 / NFR-PORT-001) [PERSISTING]

**Prior status (maint-2026-06-17):** MEDIUM.
**Current status:** MEDIUM — NO CHANGE; RISK SURFACE SLIGHTLY ENLARGED.

CI remains ubuntu-latest only. EtherNet/IP (ADR-010) uses explicit little-endian byte reads
throughout (`u16::from_le_bytes`, `u32::from_le_bytes` etc.) — this is architecturally safer
than implicit byte ordering, but the two-level ENIP framing complexity (24-byte fixed header
+ variable CPF items + CIP payload) slightly increases the surface for platform-specific
edge cases relative to Modbus/DNP3. No macOS/Windows CI matrix job has been added.

The release CI (`release.yml`) cross-compiles for 4 platforms, catching build-time failures.
Runtime behavior on non-Linux platforms remains the unmitigated risk.

**Assessment:** Medium (unchanged). Recommend formal R-003.

---

### R-CAND-004 — rayon unused dependency (NFR-VIO-006 / O-07) [RESOLVED since prior sweep]

**Prior status (maint-2026-06-17):** LOW (P2), NFR-VIO-006 OPEN.
**Current status:** RESOLVED — PR #304 (maint-2026-06-22).

`rayon = "1"` removed from Cargo.toml in PR #304. O-07 closed in tech-debt-register.
NFR-VIO-006 resolved. No further action required.

---

### R-CAND-005 — Port-502 false-routing risk for non-Modbus binary protocols (ACCEPTED KNOWN RISK) [PERSISTING]

**Prior status (maint-2026-06-17):** ACCEPTED KNOWN RISK.
**Current status:** ACCEPTED KNOWN RISK — NO CHANGE.

VP-022 Kani-verified three-point gate remains in force. No new analyzer interacts with
port-502 routing. Recommend formal R-005 with status ACCEPTED.

---

### R-CAND-006 — DNP3 T0827 emission threshold misconfiguration risk (ADR-007) [PERSISTING, AGING]

**Prior status (maint-2026-06-17):** MEDIUM (P2).
**Current status:** MEDIUM (P2) — AGING, NO VALIDATION PERFORMED.

`--dnp3-direct-operate-threshold` default remains an engineering judgment (5-20 range per
dnp3-research.md §5.1) without empirical calibration. Same class as ASM-CAND-003/009. The
DNP3 memory-DoS fix (PR #362) corrected `on_flow_close` lifecycle — it did not change the
T0827 detection threshold or calibration status.

**Assessment:** Medium, aging. Cross-references the same labelled-ICS-traffic corpus needed
for ASM-CAND-003 and ASM-CAND-009. Bundle into the single threshold-calibration P1 track.
Recommend formal R-006.

---

### R-CAND-007 — RUSTSEC-2026-0097 transitive rand 0.8.5 [RESOLVED since prior sweep]

**Prior status (maint-2026-06-17):** LOW, ACCEPTED-TRANSITIVE.
**Current status:** RESOLVED — PR #304 (maint-2026-06-22).

rand bumped to 0.8.6; CI `--ignore RUSTSEC-2026-0097` flag removed; `cargo audit` clean.
No further action required.

---

### R-CAND-008 — VLAN/QinQ/MACsec ARP offset detection limitation (ACCEPTED KNOWN LIMITATION) [PERSISTING]

**Prior status (maint-2026-06-17):** ACCEPTED KNOWN LIMITATION.
**Current status:** ACCEPTED KNOWN LIMITATION — NO CHANGE.

The MACsec ARP limitation (CWE-693, ciphertext-opacity) is correctly documented by design
(STORY-117, E-17). The ARP analyzer refactor in PR #366 (v0.11.4, cosmetic
`insert_binding_lru` return-bool dedup) did not affect the offset detection path.
Recommend formal R-008 with status ACCEPTED. README Known Limitations entry is still absent
(prior sweep REC-006); carry forward.

---

### R-CAND-009 — Memory-DoS for DNP3/ENIP per-flow state (CWE-401+CWE-770) — IDENTIFIED AND RESOLVED IN THIS CYCLE

**Source:** maint-2026-07-01 (SEC-005 + SEC-006 surfaced); research-agent validation
`.factory/research/deferred-security-perf-validation-2026-07.md`.
**Risk:** `dispatcher.rs` `on_flow_close` had both DNP3 and ENIP arms stubbed, meaning
neither analyzer's per-flow state was purged on flow close. In long captures with many
flows, DNP3 and ENIP state maps would grow without bound (CWE-401 resource leak, CWE-770
unbounded allocation). Severity: MEDIUM (offline-bounded, but RSS could grow to multiple
GB on large captures).

**Resolution:** PR #362 (v0.11.3, 2026-07-06): `on_flow_close` wired for both ENIP and
DNP3 dispatch arms. `Dnp3Analyzer::on_flow_close` method added; `summarize()` aggregated
over both open and closed flows. Smoke-test confirmed 541 DNP3 flows purged; RSS 303 MB
at 2.25M-packet test. Security-reviewer APPROVE; pr-reviewer APPROVE; CI 11/11. Issue #342 CLOSED.

**Status:** RESOLVED in v0.11.3. Record as formal R-009 (status: RESOLVED, closed by PR #362).

---

### R-CAND-010 — Unsafe split-borrow in `src/analyzer/enip.rs` (SEC-001) — NEW SINCE PRIOR SWEEP

**Source:** PR #334 security review (pre-v0.11.0); tech-debt-register SEC-001; research-agent
validation `.factory/research/deferred-security-perf-validation-2026-07.md`.
**Risk:** `enip.rs` `on_data` uses pointer-derived split-borrow to separately access
`state.carry_c2s` and `state.carry_s2c` from `state`. The borrow is sound under the stated
invariant (the two carry fields are disjoint and non-aliasing), but the pattern is fragile
under refactoring — future `EnipFlowState` struct changes that violate the invariant would
produce undefined behavior.

**Research-agent verdict:** CONFIRMED present, sound-as-written. DOWNGRADED to LOW (no
exploit channel; safe-refactor to `get_disjoint`/index approach recommended). Filed in
tech-debt-register as SEC-001, status OPEN, v0.12.0 candidate.

**Impact:** LOW (P2). Unsafe block is correctly bounded for current struct layout.
Not a runtime exploitability issue as shipped. Recommend formal R-010 with status
OPEN/ACCEPTED-PENDING-REFACTOR. Track as v0.12.0 candidate.

---

### R-CAND-011 — Port-44818 false-routing risk for non-EtherNet/IP binary protocols (ADR-010) — NEW SINCE PRIOR SWEEP

**Source:** ADR-010, Consequences; parallel to R-CAND-005 (port-502 Modbus).
**Risk:** Non-ENIP binary protocols on TCP/44818 (custom framing, administrative tools,
test equipment) are mis-routed to `EnipAnalyzer` and incur the command-code validity gate.
If the gate fails to reject a non-ENIP frame whose first two bytes happen to be a valid
ENIP command code, a false ENIP finding could be emitted.

ENIP gate strength: the command code check discriminates from a sparse set (~4 of 65,536
possible 2-byte values for the primary commands), plus a session-handle non-zero check for
Send commands. This is weaker than DNP3's 0x0564 sync word but stronger than relying on port
alone. Port 44818 is IANA-registered exclusively for EtherNet/IP, reducing false-routing
likelihood in practice. Unlike VP-022 (Modbus) and VP-023 (DNP3), there is no dedicated
Kani harness for the ENIP gate discriminator (only VP-004 classify_oracle was updated).

**Impact:** LOW. IANA-exclusive registration is a strong practical discriminator. Recommend
formal R-011 with status ACCEPTED. Flag dedicated ENIP gate Kani proof for the next ENIP
hardening cycle (see REC-007).

---

### R-CAND-012 — `rebind_count` arithmetic non-saturating in `src/analyzer/arp.rs` — NEW SINCE PRIOR SWEEP (LOW/INFORMATIONAL)

**Source:** PR #366 security review observation REBIND-COUNT-SATURATING-001 (2026-07-06).
**Risk:** `rebind_count` in `src/analyzer/arp.rs` uses plain `+= 1` rather than
`saturating_add(1)`. Theoretical u64 overflow at ~1.8 × 10^19 rebind events.

**Research-agent triage (D-386):** FALSE-POSITIVE at practical threat level. u64 overflow
unreachable within bounded offline captures (4 GiB pcap max; ARP frame minimum ~42 bytes;
overflow requires > 4.4 × 10^17 frames — ~58,000 years at 1 Gbps). Same class as SEC-004/SEC-007
(already cleared as u64 counters). DF-VALIDATION-001-gated before any GitHub issue filing.

**Impact:** LOW/INFORMATIONAL. Recommend formal R-012 with status ACCEPTED. Optional
hardening: change `rebind_count += 1` to `rebind_count = rebind_count.saturating_add(1)` for
consistency with NFR-REL-003 saturating arithmetic convention.

---

## Section 4 — Mitigation Validity Check Against v0.11.x Architecture

### Check: ADR-010 (EtherNet/IP) impact on existing mitigations

| Pre-existing risk / mitigation | Impacted by ADR-010? | Verdict |
|-------------------------------|----------------------|---------|
| R-CAND-001 (weak-cipher heap) | No — tls.rs unchanged | NOT INVALIDATED |
| R-CAND-002 (eager Vec load) | No — reader.rs unchanged | NOT INVALIDATED |
| R-CAND-005 (Port-502 Modbus routing) | No — dispatch.rs port-502 arm unchanged | NOT INVALIDATED |
| VP-022 Modbus Kani proof | No — ModbusAnalyzer unchanged | NOT INVALIDATED |
| VP-023 DNP3 Kani proof | No — Dnp3Analyzer unchanged | NOT INVALIDATED |
| TLS loose gate (Smell #10) | No — ENIP frames use DecodedFrame path; gate irrelevant | NOT INVALIDATED |
| MAX_FINDINGS cap (10,000) | No — ENIP findings flow through same cap | NOT INVALIDATED |
| VP-004 classify_oracle harness | Updated with ENIP arm atomically with ADR-010 delivery | NOT INVALIDATED (re-verified) |

### Check: ADR-011 (TLS handshake reassembly) impact on existing mitigations

ADR-011 adds per-flow handshake reassembly state inside `TlsAnalyzer`. The 2-byte TLS gate
(ASM-CAND-005, Smell #10) fires in the dispatcher BEFORE the `TlsAnalyzer.on_data` call;
reassembly is an internal analyzer concern and does not alter the gate condition. VP-039
(TLS fragment reassembly) and VP-040 (ClientHello reconstruction) are new additions, not
replacements of any prior proof.

**Verdict:** ADR-011 does NOT invalidate any prior mitigation. It adds two new formal proofs.

### Check: ADR-012 / SS-18 (protocol coverage catalog) impact on existing mitigations

SS-18 (`src/protocols.rs`) is a PURE CORE module: compile-time constants and pure functions
only. No side effects, no global mutable state, no interaction with existing dispatch,
reassembly, or analyzer subsystems at runtime. VP-041 (proptest), VP-042 (classify/catalog
consistency), VP-043 (gap counter correctness) are entirely additive.

**Verdict:** ADR-012/SS-18 does NOT invalidate any prior mitigation.

### Check: PR #362 (DNP3/ENIP memory-DoS fix) impact on existing mitigations

PR #362 adds `on_flow_close` wiring for `DispatchTarget::Enip` and `DispatchTarget::Dnp3`
in `dispatcher.rs`. The `on_data`, `on_new_flow`, and classification paths are unchanged.
All per-flow state is now purged on close. VP-023 and all ENIP-related Kani proofs remain
valid.

**Verdict:** PR #362 does NOT invalidate any prior mitigation. It resolves R-CAND-009 and
improves RSS bounds for long captures.

### Check: PR #365 (observability counters) impact on existing mitigations

The four new counters (`bindings_evicted`, `storm_counters_evicted`, `dropped_transactions`,
`dropped_map_entries`) add `u64` fields with `saturating_add` increments. The 8 amended
BCs (BC-INDEX v2.18) formalize counter behavior without changing detection logic, finding
emission, or any resource bound.

**Verdict:** PR #365 does NOT invalidate any prior mitigation. It improves observability
for ASM-CAND-006 and ASM-CAND-009.

---

## Section 5 — Invalidated Assumption / Missing Risk Escalation Check

Since the prior sweep (maint-2026-06-17), no design assumption has been invalidated. The
memory-DoS finding (R-CAND-009) was a missing implementation (stubbed `on_flow_close`
arms in `dispatcher.rs`), not an assumption invalidation — the design assumption that
per-flow state must be purged on flow close was always correct in the specs and ADRs;
the code simply did not implement it.

The prior-sweep-documented assumption invalidation (microsecond-scale window in BC-2.14.016/017,
corrected in F5 of the DNP3 feature cycle, v0.6.0) remains the only recorded assumption
invalidation in project history. No new invalidation requiring a risk escalation was found
in this sweep.

---

## Section 6 — Resolved Since Prior Sweep (maint-2026-06-17)

| Item | Resolution | PR / Release |
|------|-----------|-------------|
| R-CAND-004: rayon unused dep | RESOLVED — `rayon` removed from Cargo.toml | PR #304, maint-2026-06-22 |
| R-CAND-007: RUSTSEC-2026-0097 rand 0.8.5 | RESOLVED — rand bumped to 0.8.6; CI --ignore removed | PR #304, maint-2026-06-22 |
| R-CAND-009: DNP3/ENIP per-flow memory-DoS | IDENTIFIED AND RESOLVED in same session | PR #362, v0.11.3, 2026-07-06 |
| ASM-CAND-006 risk posture improved | `bindings_evicted` counter makes ARP LRU evictions observable | PR #365, v0.11.4, 2026-07-06 |
| ASM-CAND-009 observability improved | `storm_counters_evicted` counter surfaces storm state overflow | PR #365, v0.11.4, 2026-07-06 |

---

## Section 7 — Prioritized Recommendations

| Priority | ID | Action | Effort | Status change |
|----------|----|--------|--------|---------------|
| HIGH | REC-001 | Promote TD-MAINT-RISK-REGISTRY-BACKFILL from P2 to P1. Backfill `specs/risk-register.md` (R-001..R-012) and `specs/assumptions.md` (ASM-001..ASM-011) before the next ICS protocol feature cycle. Two additional architectural assumptions (ASM-010, ASM-011) have accrued since the prior sweep with no formal home. | M (1 session) | P2 → P1 |
| HIGH | REC-002 | ASM-CAND-003 (anomaly threshold calibration) and ASM-CAND-009 (ARP storm rate default): both are past the escalation threshold by 8+ releases (ASM-003) and 6+ releases (ASM-009). Bundle into a single P1 calibration track with R-CAND-006 (DNP3 T0827 threshold): either (a) formally accept with written rationale + README § Known Limitations entry, or (b) scope validation exercise post-v0.12.0. Continuing to carry all three as unaddressed is no longer tenable. | S–M | MONITOR → ESCALATE-CRITICAL (P1) |
| MEDIUM | REC-003 | R-CAND-001 (weak-cipher heap, NFR-RES-023, GitHub #102): add `MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap with "+N more" entry in the next TLS-touching story. The v0.11.4 silent-limit audit (D-385) did not cover this Vec — it must be in the next silent-limit or observability pass. | S | Unchanged (P1, no action in 6+ months) |
| MEDIUM | REC-004 | R-CAND-010 (SEC-001 unsafe split-borrow in enip.rs): refactor to safe `get_disjoint`/index pattern. v0.12.0 candidate (tech-debt-register SEC-001 OPEN). Fragility increases with any future `EnipFlowState` struct change. | S | NEW → P2 |
| LOW | REC-005 | ASM-CAND-008 / ADR-007 Decision 3 documentation gap: add Crain/Sistrunk CRC-caveat as a normative note. Documentation-only, no code change. Carry forward from prior sweep. | XS | Unchanged |
| LOW | REC-006 | R-CAND-008 (MACsec ARP limitation): add one sentence to README § Known Limitations referencing STORY-117. Bundle with documentation cleanup PR. Carry forward from prior sweep. | XS | Unchanged |
| LOW | REC-007 | R-CAND-011 (Port-44818 ENIP gate, no dedicated Kani proof): add a `classify_enip_gate` Kani harness analogous to VP-022/VP-023 in the next ENIP hardening cycle. Not urgent (IANA-exclusive port, VP-004 updated); schedule for v0.12.0 planning. | S | NEW → P3 |
| INFORMATIONAL | REC-008 | R-CAND-012 (rebind_count non-saturating): optional — change `rebind_count += 1` to `rebind_count = rebind_count.saturating_add(1)` for consistency with NFR-REL-003. DF-VALIDATION-001-gated before GitHub issue. | XS | NEW → informational |
| INFORMATIONAL | REC-009 | All 80 VSDD consistency criteria remain unverifiable for criteria 42-50 (ASM/R traceability) until a formal registry exists. Record in STATE.md as a persisting DRIFT item if not already noted. | XS | Unchanged |

---

## Section 8 — Conclusion

wirerust does not maintain a formal ASM-NNN / R-NNN risk register. Informal tracking across
domain-debt, NFR-VIO, ADR Consequences, and the tech-debt register has been adequate through
12+ releases, but the governance gap is widening: the analyzer portfolio now spans 7 analyzers
(HTTP, TLS, DNS, Modbus, DNP3, ARP, EtherNet/IP) plus the SS-18 protocol coverage catalog,
and 11 design assumptions are tracked only implicitly.

Three significant positive developments occurred since the maint-2026-06-17 baseline: the
rayon and RUSTSEC-2026-0097 risks were closed cleanly (PR #304); the memory-DoS risk for
DNP3/ENIP per-flow state was identified and fixed in the same maintenance session (PR #362,
v0.11.3, issue #342 closed); and the v0.11.4 observability counters materially improved the
forensic audit trail for two long-standing accepted design assumptions (ARP binding eviction,
ARP storm state). No existing mitigation was invalidated by any of the v0.11.x architectural
additions.

The two most pressing open items are: (1) the formal registry backfill
(TD-MAINT-RISK-REGISTRY-BACKFILL, now P1 — two sweeps without action), and (2) the
threshold-calibration track for ASM-CAND-003 / ASM-CAND-009 / R-CAND-006, which has now
accumulated an unjustifiable backlog of unvalidated engineering defaults without even a
formal written acceptance rationale.
