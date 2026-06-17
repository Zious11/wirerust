---
document_type: maintenance-finding
sweep: Sweep-4
sweep_name: holdout-freshness-check
maintenance_run_id: maint-2026-06-17
producer: consistency-validator
timestamp: 2026-06-17T00:00:00Z
scope: HS-001..HS-100 (greenfield holdout set)
out_of_scope: "Feature holdout seeds (HS-W35..HS-W44) stored in wave-holdout-scenarios/; not individual HS-NNN files"
latest_release: v0.7.1
releases_since_greenfield_eval: 7
---

# Holdout Freshness Check — Sweep 4 (DF-030 Lifecycle Audit)

**Run date:** 2026-06-17
**Scope:** 100 greenfield holdout scenarios (HS-001..HS-100) in `.factory/holdout-scenarios/`
**Feature holdout seeds** (HS-W35..HS-W44) are out of scope — they live in
`wave-holdout-scenarios/` as monolithic wave files, not as individual HS-NNN markdown files,
and were evaluated as part of their respective feature cycles (v0.6.0 DNP3 F7, v0.7.0 ARP F4).

---

## Staleness Criteria Applied

| Criterion | Description |
|-----------|-------------|
| S1: Deprecated/removed BCs | Scenario references BC with `lifecycle_status: deprecated` or `retired` |
| S2: 3+ releases without evaluation | `last_evaluated: null` and 3+ releases shipped since last known evaluation |
| S3: Expected behavior no longer matches product | Body text or BC table count assertions contradict current product state |

---

## Release Timeline Reference

| Version | Release Date | Notes |
|---------|-------------|-------|
| v0.1.0 | 2026-06-08 | First release; greenfield Phase-4 holdout eval ran 2026-06-01 (pre-release) |
| v0.2.0 | 2026-06-09 | Post-greenfield release |
| v0.3.0 | 2026-06-09 | Post-greenfield release |
| v0.4.0 | 2026-06-10 | Post-greenfield release |
| v0.5.0 | 2026-06-10 | Post-greenfield release |
| v0.6.0 | 2026-06-12 | DNP3 feature release |
| v0.7.0 | 2026-06-16 | ARP Security Analyzer feature release |
| v0.7.1 | 2026-06-17 | E-17 QinQ/MACsec offset regression hardening (latest) |

**Greenfield Phase-4 evaluation date:** 2026-06-01 (EVAL-INDEX, cycles/v0.1.0-greenfield-spec/).
All 100 HS files carry `last_evaluated: null`. The Phase-4 evaluation ran pre-release, before
v0.1.0 was tagged on 2026-06-08. No formal re-evaluation has been recorded for any greenfield
scenario since then. Seven (7) releases have shipped since the Phase-4 evaluation date.

---

## Summary Table

| Criterion | Applicable Scenarios | Count |
|-----------|---------------------|-------|
| S1: References deprecated/retired BCs | None | 0 |
| S2: Not evaluated in 3+ releases | HS-001..HS-100 (all; 7 releases shipped since eval) | 100 |
| S3: Count assertions contradict product state | HS-008, HS-009 | 2 |

---

## Per-Scenario Recommendations

> **Column definitions:**
> - `current_lifecycle_status`: value of `lifecycle_status` in the file frontmatter
> - `recommended_status`: this sweep's recommendation — no file changes are made here
> - `reason`: staleness criterion(a) triggered

### Scenarios With Recommended Status Change to `stale`

The following 2 scenarios have BOTH S2 (unevaluated 7 releases) AND S3 (body count assertions
now contradict product state after the v0.7.0 ARP cycle added T0830/T1557.002).

| Scenario ID | Title | current_lifecycle_status | recommended_status | Reason |
|-------------|-------|--------------------------|-------------------|--------|
| HS-008 | MITRE ATT&CK Tactic Display Names and Kill-Chain Order Completeness | active | stale | S2+S3: BC table body says "technique_name returns Some for all 23 seeded IDs" (line 64); current SEEDED=25 after ARP cycle (BC-2.10.005 v1.8, src/mitre.rs SEEDED_TECHNIQUE_ID_COUNT=25). PG-ARP-F2-006 explicitly flags this scenario. Tactic count "17 total" in same table is correct. |
| HS-009 | MITRE Technique Catalog — Known ID Lookup, Unknown ID Graceful Handling | active | stale | S2+S3: Verification section says "15 currently-emitted technique IDs" and lists T0827 as the 15th (line 73-75); current EMITTED=17 after ARP cycle added T0830+T1557.002 (src/mitre.rs EMITTED_IDS list confirmed). PG-ARP-F2-006 explicitly flags this scenario. |

### Scenarios With Recommended Status `active` (Evaluated/Refreshed at Risk Boundary)

HS-025 (`should-pass`) references BC-2.10.002 (ICS tactic display) and BC-2.10.009
(`#[non_exhaustive]`). Both BCs remain `lifecycle_status: active`; the non-exhaustive
attribute and ICS tactic rendering behavior are unchanged. However, S2 applies: 7 releases
have shipped without re-evaluation. HS-025 behavior (ICS tactic display) was implicitly
exercised during the ARP F2 adversarial convergence (33 passes verified ICS tactic rendering
through BC-2.10.002). Recommendation: keep `active` but schedule re-evaluation.

| Scenario ID | Title | current_lifecycle_status | recommended_status | Reason |
|-------------|-------|--------------------------|-------------------|--------|
| HS-025 | ICS Tactic Display and Non-Exhaustive Enum Stability | active | active | S2 only: 7 releases uneval'd, but behavior exercised during ARP F2 adversarial (33 passes on BC-2.10.002). BCs active; expected behavior matches product. No count assertion drift. Should-pass classification unchanged. |

### Scenarios With Recommended Status `active` (S2 only — no content drift)

All 97 remaining greenfield scenarios (HS-001..HS-007, HS-010..HS-024, HS-026..HS-100,
excluding HS-008, HS-009, HS-025) trigger S2 exclusively. No deprecated BCs are referenced
by any scenario (BC-ABS-004..009 were retired during the remediation cycle but the surviving
BC-2.13.001..004 active replacements are correctly referenced by HS-086). No scenario body
text contains behavioral assertions that contradict the product after reviewing the DNP3 and
ARP feature cycles.

| Scenario ID | Title (abbreviated) | current_lifecycle_status | recommended_status | Reason |
|-------------|---------------------|--------------------------|-------------------|--------|
| HS-001 | PCAP Link-Type Boundary | active | active | S2 only — no content drift; BC-2.01.001/004 active; behavior unchanged |
| HS-002 | Empty Capture and Corrupt-Header Behavior | active | active | S2 only — no content drift |
| HS-003 | Ethernet, RAW IPv4, IPv6 Decode Paths | active | active | S2 only — BC-2.02.009 was revised v1.6 (three-way postcondition for ARP) but HS-003 tests IPv4/IPv6 paths only, not ARP path |
| HS-004 | Linux SLL Cooked Capture, ICMP, Non-IP | active | active | S2 only — no content drift |
| HS-005 | App Protocol Hints, Frame Length, TCP Flags | active | active | S2 only — no content drift |
| HS-006 | Finding One-Liner Format | active | active | S2 only — no content drift |
| HS-007 | JSON Finding Serialization | active | active | S2 only — no content drift |
| HS-010 | FlowKey Symmetry | active | active | S2 only — no content drift |
| HS-011 | DNS Query/Response Counting | active | active | S2 only — no content drift |
| HS-012 | Non-TCP Packet Filtering, Reassembly Stats | active | active | S2 only — no content drift |
| HS-013 | Three-Way Handshake and RST Close | active | active | S2 only — no content drift |
| HS-014 | Mid-Stream Join — Partial Captures | active | active | S2 only — no content drift |
| HS-015 | Real-World Corpus — Clean PCAP | active | active | S2 only — no content drift |
| HS-016 | Real-World Corpus — Known-Problematic Evasion | active | active | S2 only — no content drift |
| HS-017 | E-1 to E-7 Cross-Subsystem Finding Construction | active | active | S2 only — no content drift |
| HS-018 | Forensic Fidelity — Attacker Bytes Preserved in JSON | active | active | S2 only — no content drift; note: file lacks `lifecycle_status` field in YAML (field absent from frontmatter; minor schema gap, not a staleness issue) |
| HS-019 | TCP Seq Wraparound Reassembly | active | active | S2 only — no content drift |
| HS-020 | DNS and TCP Parallel Wave 4 | active | active | S2 only — no content drift |
| HS-021 | RST/FIN Close and Timeout Lifecycle | active | active | S2 only — no content drift |
| HS-022 | Decoder No-Panic Safety | active | active | S2 only — no content drift |
| HS-023 | Waves 1-5 Full Integration | active | active | S2 only — no content drift |
| HS-024 | Source IP Field — Present for Reassembly, Absent for HTTP/TLS | active | active | S2 only — no content drift |
| HS-026..HS-050 | TCP Reassembly Engine scenarios (25 scenarios) | active | active | S2 only — no content drift; TCP reassembly behavior unchanged across feature cycles |
| HS-051..HS-072 | HTTP and TLS Analysis scenarios (22 scenarios) | active | active | S2 only — no content drift; HTTP/TLS analysis behavior unchanged; HS-070 cross-subsystem scenario still valid |
| HS-073..HS-083 | Reporting and Output Format scenarios (11 scenarios) | active | active | S2 only — no content drift; reporter behavior unchanged |
| HS-084..HS-100 | CLI, Entry Point, and E2E scenarios (17 scenarios) | active | active | S2 only — no content drift; HS-086 (removed flags) correctly references active BC-2.13.001..004 |

---

## S2 Aggregate: Full List

All 100 scenarios meet S2 (unevaluated in 3+ releases). The following is a condensed
status roll-up. Per the task scope, S2-only scenarios remain recommended `active`; S2+S3
scenarios are recommended `stale`.

| Recommended Status | Count | Scenario IDs |
|-------------------|-------|-------------|
| `stale` (S2+S3) | 2 | HS-008, HS-009 |
| `active` (S2 only, no content drift) | 98 | HS-001..HS-007, HS-010..HS-024, HS-025 (see note), HS-026..HS-100 |
| `retired` | 0 | None |
| **TOTAL** | **100** | |

---

## Findings Register

### FIND-HF-001 (Major): HS-008 Body Count Assertion Stale — 23 seeded IDs should be 25

**Scenario:** HS-008 (`HS-008-mitre-tactic-display-and-kill-chain-order.md`)
**Criterion triggered:** S2, S3
**Evidence:**
- HS-008 BC Linkage table (line 64): "technique_name returns Some for all 23 seeded IDs"
- Current product: `src/mitre.rs` `SEEDED_TECHNIQUE_ID_COUNT = 25` (T0830, T1557.002 added
  by ARP cycle STORY-114, PR #240, confirmed at v0.7.0 release)
- BC-2.10.005 v1.8 (2026-06-16): Postcondition 1 updated from 23 to 25 seeded IDs
- Policy context: PG-ARP-F2-006 (STATE.md) explicitly flags HS-008 as carrying stale
  greenfield-era counts "not swept across DNP3 cycle"

**Impact:** If holdout evaluator runs HS-008 and tests exactly 23 IDs per the body instruction,
they miss T0830 and T1557.002. The evaluator would produce a false-pass score (implementation
handles 25 IDs correctly but only 23 were tested).

**Recommended fix:** Update HS-008 line 64 to read "technique_name returns Some for all 25
seeded IDs" and add T0830/T1557.002 to the verification approach list. Body version bump
required (1.2 → 1.3 or appropriate next version).

---

### FIND-HF-002 (Major): HS-009 Body Count Assertion Stale — 15 emitted IDs should be 17

**Scenario:** HS-009 (`HS-009-mitre-technique-lookup-unknown-ids.md`)
**Criterion triggered:** S2, S3
**Evidence:**
- HS-009 verification section (lines 73-75): "15 currently-emitted technique IDs (T1027,
  T1036, T1046, T1083, T1499.002, T1505.003, T1692.001, T0836, T0814, T0806, T0835, T0831,
  T0888, T1691.001, T0827). All 15 must resolve."
- Current product: `src/mitre.rs` EMITTED_IDS has 17 entries — list above plus T0830
  (ARP Adversary-in-the-Middle) and T1557.002 (ARP Cache Poisoning), both added by
  STORY-114 / PR #240 at v0.7.0
- BC-2.10.008 v1.13 (2026-06-15): EMITTED count updated 15→17
- Policy context: PG-ARP-F2-006 (STATE.md) explicitly flags HS-009 as carrying stale counts

**Impact:** Same class as FIND-HF-001. An evaluator following the body instruction would
test only 15 IDs and miss T0830/T1557.002. Because the ARP analyzer emits these IDs on
spoofed-traffic captures, a correctly implemented end-to-end ARP test would produce them,
and they MUST resolve in the catalog. A 15-ID test would not catch a regression on the
new ARP-specific entries.

**Recommended fix:** Update HS-009 verification section to list 17 emitted IDs (add T0830
and T1557.002 with their tactic/description notes), update the count from 15 to 17. Body
version bump required.

---

### FIND-HF-003 (Minor): HS-018 Missing `lifecycle_status` Field in Frontmatter

**Scenario:** HS-018 (`HS-018-raw-data-contract-no-escape-in-json.md`)
**Criterion triggered:** None (not a staleness issue — schema gap only)
**Evidence:** `grep "lifecycle_status" HS-018*.md` returns empty. All other 99 HS files
have `lifecycle_status: active`. The file has `last_evaluated: null`, `stale_reason: null`,
`retired: null` — all other lifecycle fields present. The `lifecycle_status` key itself
is absent.

**Impact:** Minor: automated lifecycle queries (e.g., `grep -h "lifecycle_status"`) will
under-count active scenarios by 1. Not a behavioral staleness risk.

**Recommended fix:** Add `lifecycle_status: active` to HS-018 frontmatter between the
`behavioral_contracts` block and `introduced` field.

---

### FIND-HF-004 (Observation): All 100 Greenfield Scenarios Unevaluated Since Pre-Release Phase 4

**Criterion triggered:** S2 (universal)
**Evidence:** All 100 HS files carry `last_evaluated: null`. The Phase-4 evaluation date was
2026-06-01; v0.1.0 released 2026-06-08; 7 subsequent releases (v0.2.0..v0.7.1) shipped
without a documented formal re-run. EVAL-INDEX.md confirms no re-evaluation was conducted.

**Impact per-scenario assessment:** S2 triggers for all 100, but content review (S3) found
drift only in HS-008 and HS-009. The underlying behavior covered by the remaining 98
scenarios (TCP reassembly, HTTP/TLS analysis, PCAP ingest, DNS, MITRE, reporting, CLI) was
not altered by any of the v0.2.0..v0.7.1 feature cycles (DNP3 added new SS-15 subsystem,
ARP added new SS-16 subsystem; neither touched the existing E-1..E-9 subsystem behavior).
The regression gate at each feature release included explicit regression coverage (VP-004,
VP-007 atomics, and per-release CI green), providing indirect evidence of continued
holdout-scenario validity.

**Recommended follow-up:** Establish a holdout re-evaluation cadence policy. The immediate
action needed is to re-evaluate HS-008 and HS-009 with corrected content (FIND-HF-001/002),
then run a full or sampled re-evaluation pass to close the `last_evaluated: null` gap across
all 100 scenarios. PG-ARP-F2-006 (STATE.md) deferred this as a policy codification item.

---

## Fix-PR Candidates

| Finding | Priority | Recommended Fix Artifact | Action |
|---------|----------|--------------------------|--------|
| FIND-HF-001 | P1 | HS-008 body update (23→25 seeded, add T0830/T1557.002) | Create fix story or ad-hoc factory-artifacts commit |
| FIND-HF-002 | P1 | HS-009 body update (15→17 emitted, add T0830/T1557.002 to list) | Create fix story or ad-hoc factory-artifacts commit |
| FIND-HF-003 | P2 | HS-018 frontmatter: add `lifecycle_status: active` | Single-line factory-artifacts commit |
| FIND-HF-004 | P2 | Policy: codify holdout re-evaluation cadence | STATE.md policy entry or standalone policy doc |

---

## Scope Note: Feature Holdout Seeds (HS-W35..HS-W44)

The DNP3 (HS-W35..HS-W39, 32 scenarios) and ARP (HS-W40..HS-W44, 28 seeds) feature
holdouts are stored as monolithic wave files in
`.factory/feature/wave-holdout-scenarios/wave-35-39-holdout.md` and
`.factory/feature/wave-holdout-scenarios/wave-40-44-holdout.md`. They are not individual
HS-NNN files and are therefore out of scope for this per-file freshness sweep.

Evaluation status per the session reviews:
- **DNP3 (HS-W35..HS-W39):** Evaluated at F7 convergence (2026-06-12); holdout corpus was
  the adversarial review substrate. HS-W36-001 carry assertion was updated (F-CC-001 fix,
  PR #233). HS-W37-002 canonical DIR-bit citation updated (F-S2-001 fix, PR #232). Both
  fixes confirmed by F7 3/3 clean-streak. Release gate: 5-dim PASS at v0.6.0.
- **ARP (HS-W40..HS-W44):** Formal Phase-4 holdout evaluation ran (F4 wave-adversary
  convergence 2026-06-15); initial mean 0.997; post-D-075 fix: 15/15 mean 1.0. Release
  gate: 5-dim PASS at v0.7.0. No outstanding staleness issues identified for ARP seeds.
