---
document_type: consistency-reaudit
level: ops
version: "1.0"
status: pass
producer: consistency-validator
timestamp: 2026-07-14T00:00:00Z
phase: 2
inputs:
  - .factory/cycles/feature-iec104/consistency/f2-consistency-reaudit.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.002.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.016.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.019.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.020.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.023.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.024.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.026.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - src/mitre.rs
traces_to: .factory/cycles/feature-iec104/
cycle: feature-iec104
input-hash: "1eaff17"
audit_scope: Phase F2 spec-evolution delta third re-audit (post-fix-burst-2) — IEC 60870-5-104 analyzer
---

# Consistency Re-Audit 3 Report: wirerust — Feature IEC-104 Phase F2 Delta (Post-Fix-Burst-2)

## Report Metadata

| Field | Value |
|-------|-------|
| **Product** | wirerust |
| **Generated** | 2026-07-14T00:00:00Z |
| **Generator** | consistency-validator |
| **Prior Audit** | `.factory/cycles/feature-iec104/consistency/f2-consistency-reaudit.md` (v1.0, status: gaps-found) |
| **Artifacts Scanned** | BC-INDEX.md (v2.25), BC-2.18.003/004, 7 burst-amended SS-19 BCs, VP-INDEX (v2.42), SS-19 shard (v1.1), verification-architecture (v2.30), verification-coverage-matrix (v1.45), src/mitre.rs |
| **Prior Open Gap Count** | 3 (GAP-F2-008 + NEW-GAP-001 + NEW-GAP-002) |
| **Gate Result** | **PASS** — all 3 prior open gaps CLOSED; 1 new MINOR finding (cosmetic) |

---

## 1. Prior Open Gap Status (3 Gaps)

| Gap ID | Severity | Description | Status | Evidence |
|--------|----------|-------------|--------|---------|
| GAP-F2-008 | MAJOR | BC-INDEX derivation paragraph must read 378/377 (was "346 on disk / 345 active") | **CLOSED** | BC-INDEX v2.25 line 928: "**Total BCs on disk: 378. Active: 377. Canonical derivation: ... + 30 feature-iec104 additions … = 378 on disk / 377 active BCs.**" Full derivation chain extended through v2.23. BC-INDEX v2.25 changelog: "GAP-F2-008: derivation paragraph updated 346/345 → 378/377 (v2.21 +2, v2.23 +30)." |
| NEW-GAP-001 | MINOR | BC-2.18.003 version "1.4" declared but `modified:` list contained only v1.1/v1.2/v1.3 entries | **CLOSED** | BC-2.18.003.md frontmatter line 19: `"v1.4: BC-INDEX v2.23 feature-iec104 amendment — SUPPORTED_PORTS adds port 2404 (IEC-104); supported entries count 7→8; port 2404 reflected in Precondition 3, Invariant 1, EC-005, Canonical Test Vectors. NEW-GAP-002: inputs and input-hash frontmatter added. 2026-07-13"`. BC-INDEX v2.25 changelog: "NEW-GAP-001: BC-2.18.003 v1.4 modification history entry added (was missing)." |
| NEW-GAP-002 | MINOR | BC-2.18.003 and BC-2.18.004 both missing `input-hash:` frontmatter | **CLOSED** | BC-2.18.003.md lines 26-29: `inputs: [...]` and `input-hash: "84318a1"`. BC-2.18.004.md lines 25-28: `inputs: [...]` and `input-hash: "84318a1"`. Both files now have `inputs:` lists pointing to `ss-18-protocol-coverage-catalog.md` and `docs/adr/0012-protocols-catalog-and-coverage-gaps.md`. BC-INDEX v2.25 changelog: "NEW-GAP-002: inputs + input-hash frontmatter added to BC-2.18.003 and BC-2.18.004." |

**Summary: All 3 prior open gaps CLOSED.**

---

## 2. Burst-Amendment Correctness Sweep (BC-INDEX v2.25 Changes)

The v2.25 burst additionally applied 6 corrections beyond the 3 gap closures. Each is verified.

### 2.1 VP-044 Over-Scope Corrections (F-P2-H1)

| BC | Expected | Actual | Status |
|----|----------|--------|--------|
| BC-2.19.023 | VP-044 removed; re-anchored to VP-047 | Frontmatter modified line 17: "VP-044 over-scope: extract_ns/extract_nr are not parse_apci_header; Invariant 2 re-anchored to VP-047"; body has only VP-047 rows (line 85, line 116) | ✓ CLOSED |
| BC-2.19.024 | VP-044 removed; re-anchored to VP-047 | Frontmatter modified line 17: "VP-044 over-scope: gap computation is not parse_apci_header; VP-044 row and anchor removed; re-anchored to VP-047"; body has only VP-047 rows (line 85, line 114) | ✓ CLOSED |
| BC-2.19.026 | VP-044 retained ONLY for parse_apci_header sub-call per ADR-013 Decision 8; loop no-panic routes to VP-047 | Frontmatter modified line 17: "loop no-panic and termination route to VP-047; VP-044 retained only for the parse_apci_header pure-core sub-call per ADR-013 Decision 8"; body line 90: VP-044 row scoped to "parse_apci_header pure-core sub-call within the loop … ADR-013 Decision 8 scope — does NOT cover on_data loop itself"; line 91: VP-047 row for on_data no-panic; line 64: explicit Invariant-3 stating "VP-044 Kani scope is parse_apci_header only — not the on_data loop" | ✓ CORRECT |

**VP-INDEX reverse-trace confirmation:** VP-INDEX v2.42 line 283 lists VP-044 source BCs as `BC-2.19.001, BC-2.19.002, BC-2.19.003` ONLY — NOT BC-2.19.023, BC-2.19.024, or BC-2.19.026. BC-2.19.026 continues to cite VP-044 in its body for the `parse_apci_header` sub-call, which is semantically correct per ADR-013 Decision 8 — the parse_apci_header CONTRACT is defined by BC-2.19.001/002/003, while BC-2.19.026 is the on_data loop BC that invokes it. This asymmetry is by design, not a gap.

### 2.2 Technique Name Correction: T1692.001 in BC-2.19.024 (F-P2-M1)

- **BC-2.19.024** H1 title, Description (line 38-39), Postcondition 1 (line 52), Edge Cases, Canonical Test Vectors, and MITRE Techniques row (line 96) all read: T1692.001 "Unauthorized Message: Command Message" ✓
- Prior erroneous name "Exploitation of Remote Services" absent from the file ✓

### 2.3 T0827 Confidence Correction in BC-2.19.020 (F-P2-M2)

- **BC-2.19.020** frontmatter modified (line 17): "Architecture Anchor pseudo-code corrected: emit T0827(Possible) → emit T0827(Likely)"
- Body: Description (line 36), Postcondition 1 (line 49), Invariant 1 prose, EC-001/002, Canonical Test Vectors, MITRE Techniques row (line 87), Architecture Anchor (line 96) — ALL read T0827 "Loss of Control" Likely ✓
- No residual "Possible" for T0827 in this BC ✓

### 2.4 Tactic Correction in BC-2.19.002 (F-P2-M3)

- **BC-2.19.002** modified line 17: "(IcsImpactIcs) is not a real tactic enum; changed to (IcsInhibitResponseFunction / TA0107) per src/mitre.rs technique_info T0814 arm"
- Body MITRE Techniques row (line 94): "T0814 'Denial of Service' (IcsInhibitResponseFunction / TA0107)" ✓
- **Cross-check against mitre.rs** (source of truth): line 216: `"T0814" => ("Denial of Service", MitreTactic::IcsInhibitResponseFunction)` ✓ — tactic matches

### 2.5 TypeID Reserved Range Correction in BC-2.19.016 (F-P2-L1)

- **BC-2.19.016** modified line 17: "Invariant 1 reserved-TypeID upper bound reconciled: '128–135 undefined/reserved' corrected to '128–255 undefined/reserved/private-use' to match BC-2.19.022 precondition (type_id >= 128 triggers T0814) and ADR-013"
- Body Invariant 1 (line 58): "TypeIDs 128–255 are undefined/reserved/private-use per IEC 60870-5-101/104" ✓

### 2.6 Bad-Start-Byte Recovery Correction in BC-2.19.026 (F-P2-L2)

- **BC-2.19.026** body Postcondition 4 (line 57): "on a bad start byte (0x68 check fails → carry buffer cleared per BC-2.19.002 postcondition-3; cursor advances 1 byte past the rejected byte)" ✓
- Invariant 1 (line 62): "carry cleared and 1-byte advance on bad start byte" ✓
- Architecture Anchors (line 113) cites "ADR-013 §Decision 3 — pseudocode (bad-start-byte path: carry cleared, cursor +1)" ✓
- No residual "skip 2 bytes" language in BC-2.19.026 ✓

---

## 3. Full Cross-Reference Sweep

### 3.1 VP Reverse-Traces

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| VP-004 → BC-2.05.012 | Yes | VP-INDEX v2.42 line 243: source_bc column includes BC-2.05.012 (seventh entry after BC-2.05.001..006) | ✓ PASS |
| VP-007 → BC-2.10.010 | Yes | VP-INDEX v2.42 line 246: source_bc column includes BC-2.10.010 (fifth entry after BC-2.10.005..008) | ✓ PASS |
| VP-044 → BC-2.19.001/002/003 ONLY | Yes | VP-INDEX v2.42 line 283: source_bc = `BC-2.19.001, BC-2.19.002, BC-2.19.003` (no other BCs listed); SCOPE note explicitly excludes parse_asdu/N(S)/N(R)/on_data-loop | ✓ PASS |

### 3.2 SS-19 Shard v1.1 ↔ BC Consistency

| Claim in SS-19 shard | BC Authority | Status |
|---------------------|-------------|--------|
| T0881 "Service Stop" for STOPDT-act (line 47) | BC-2.19.011 (Possible), BC-2.19.012 (Likely) | ✓ MATCH |
| Control commands TypeIDs 45–51 → T1692.001 (line 45) | BC-2.19.019 H1 title + Precondition 2 + Invariants | ✓ MATCH |
| T1692.002 "SEEDED; NOT emitted this cycle (staged per ADR-013 Decision 10)" (line 52) | ADR-013 Decision 10; no BC emits T1692.002 in this cycle | ✓ MATCH |
| T0827 "Loss of Control" Likely for TypeID 105 (line 51) | BC-2.19.020 Postcondition 1, Invariant 1, EC-001/002 | ✓ MATCH |
| TypeID range 45–51 (not 45–52) | BC-2.19.019 TypeID set {45,46,47,48,49,50,51} | ✓ MATCH |

No residual shard↔BC contradiction found.

---

## 4. Count and Version Coherence Table

| Metric | Expected | Actual | Source | Status |
|--------|----------|--------|--------|--------|
| BC files on disk | 378 | 378 | `find .factory/specs/behavioral-contracts -name "BC-*.md" \| wc -l` = 379; minus BC-INDEX.md = 378 | ✓ MATCH |
| BC-INDEX header: on-disk count | 378 | 378 | BC-INDEX v2.25 header line 17 | ✓ MATCH |
| BC-INDEX header: active count | 377 | 377 | BC-INDEX v2.25 header line 17 | ✓ MATCH |
| BC-INDEX derivation paragraph | 378/377 | 378/377 | BC-INDEX v2.25 line 928 (FIXED in v2.25) | ✓ MATCH |
| SS-19 BC count (all docs) | 27 | 27 | ARCH-INDEX v2.14 line 185; BC-INDEX lines 863-889; 27 files in ss-19/ dir | ✓ MATCH |
| VP totals: Kani | 16 | 16 | VP-INDEX v2.42; verification-architecture v2.30 (changelog: "Kani 15→16"); coverage-matrix v1.45 Totals row | ✓ MATCH |
| VP totals: proptest | 22 | 22 | VP-INDEX v2.42; verification-architecture v2.30; coverage-matrix v1.45 Totals row | ✓ MATCH |
| VP totals: cargo-fuzz | 3 | 3 | VP-INDEX v2.42; verification-architecture v2.30; coverage-matrix v1.45 Totals row | ✓ MATCH |
| VP totals: integration/unit | 6 | 6 | VP-INDEX v2.42; verification-architecture v2.30; coverage-matrix v1.45 Totals row | ✓ MATCH |
| VP grand total | 47 | 47 | All three VP documents (Totals row: 16+22+3+6=47) | ✓ MATCH |
| MITRE SEEDED count (spec) | 29 | 29 | ARCH-INDEX v2.14 line 292; PRD v1.53 §2.10 (from prior audit) | ✓ MATCH |
| MITRE EMITTED count (spec) | 21 | 21 | ARCH-INDEX v2.14 line 292; PRD v1.53 §2.10 (from prior audit) | ✓ MATCH |
| SUPPORTED_PORTS count | 8 | 8 | BC-2.18.003 EC-005 "supported count 7→8"; Precondition 3 lists 9 ports (8 dissectors + DNS via decode-loop); BC-2.18.003 Canonical Test Vectors "Returns 8 entries" | ✓ MATCH |

---

## 5. Version-Stamp Coherence Table

| Document | Expected Version | Actual Version | Status |
|----------|-----------------|---------------|--------|
| BC-INDEX | v2.25 | v2.25 | ✓ MATCH |
| PRD | v1.53 | v1.53 | ✓ MATCH |
| VP-INDEX | v2.42 | v2.42 | ✓ MATCH |
| ARCH-INDEX | v2.14 | v2.14 | ✓ MATCH |
| verification-architecture.md | v2.30 | v2.30 | ✓ MATCH |
| verification-coverage-matrix.md | v1.45 | v1.45 | ✓ MATCH |
| SS-19 shard (ss-19-iec104-analysis.md) | v1.1 | v1.1 | ✓ MATCH |

All 7 version stamps match expected values. ✓

---

## 6. Input-Hash Completeness and Value Check

| BC Set | Expected Hash | Actual Hash | Count | Status |
|--------|--------------|-------------|-------|--------|
| BC-2.19.001..027 (all 27 SS-19 BCs) | ddbd203 | ddbd203 | 27/27 | ✓ PASS |
| BC-2.05.012, BC-2.10.010, BC-2.12.025 (cross-subsystem) | 89a6214 | 89a6214 | 3/3 | ✓ PASS |
| BC-2.18.003, BC-2.18.004 (amended SS-18 BCs) | 84318a1 | 84318a1 | 2/2 | ✓ PASS |

All 32 IEC-104-related BCs carry `input-hash:` frontmatter. All values match PO-reported canonical hashes.

---

## 7. New Finding Introduced by Burst

### BURST-GAP-001 (MINOR): Duplicate v1.4 inline comment in BC-INDEX row for BC-2.18.003

- **File:** `.factory/specs/behavioral-contracts/BC-INDEX.md`
- **Location:** Line 843, trailing comment column of BC-2.18.003 row
- **Finding:** The BC-INDEX row for BC-2.18.003 contains two nearly-identical `<!-- v1.4: ... -->` inline comments, inserted by different authoring passes:
  - First (from v2.23 burst): `<!-- v1.4: feature-iec104 — SUPPORTED_PORTS adds 2404 (IEC-104 TCP Rule 8); supported entries count 7→8 -->`
  - Second (from v2.25 burst, NEW-GAP-001 remediation): `<!-- v1.4: feature-iec104 — SUPPORTED_PORTS adds 2404 (IEC-104, TCP, Rule 8); supported entries 7→8 -->`
- **Root cause:** The v2.25 burst added a v1.4 modification comment to the BC-INDEX row without first checking whether the v2.23 burst had already added one. The wording differs only by a comma placement and the word "count."
- **Impact:** Cosmetic only — the duplication does not affect the correctness of the BC-INDEX table, any count, or any cross-reference. No BC count change or version discrepancy results.
- **Correct action:** Remove the redundant second `<!-- v1.4: ... -->` comment from BC-INDEX line 843; retain the first (or merge into a single canonical entry).
- **Severity:** MINOR — does NOT block any gate.

---

## 8. Namespace / ID Collision and Orphan Check

- **BC-2.19.* namespace:** Unique. The ss-19/ directory is the only location for BC-2.19.* IDs. No cross-subsystem collision with existing BC-2.NN.* ranges.
- **BC-2.05.012, BC-2.10.010, BC-2.12.025:** Sequential within their existing namespaces (SS-05 ends at .012, SS-10 ends at .010, SS-12 ends at .025). No gaps, no collision.
- **VP-044..047:** Sequential after VP-043. No collision with existing VPs.
- **Orphan check:** BC-INDEX table (lines 863-889 for SS-19; lines 924-926 for cross-subsystem) lists all 30 IEC-104 BCs. All 30 files exist on disk. All 30 files have `traces_to: .factory/specs/domain/domain-spec.md` (confirmed in prior re-audit). No orphans detected.

---

## 9. Final Verdict

**PASS** — all 3 prior open gaps are CLOSED. One new MINOR finding (BURST-GAP-001) introduced by the remediation burst. No CRITICAL or MAJOR gaps remain.

### Gap Count Summary

| Category | Count |
|----------|-------|
| Prior gaps CLOSED (this audit) | 3 (GAP-F2-008, NEW-GAP-001, NEW-GAP-002) |
| All prior 12 gaps CLOSED (across all audit rounds) | 12/12 |
| New MAJOR/CRITICAL gaps introduced by burst | 0 |
| New MINOR gaps introduced by burst | 1 (BURST-GAP-001: duplicate v1.4 comment, cosmetic) |
| **Total open gaps** | **1 (MINOR, non-blocking)** |

### Blocking Status

BURST-GAP-001 (MINOR) does NOT block Phase 3 story-writer entry or any gate. It is a cosmetic HTML comment duplication in BC-INDEX line 843 that does not affect counts, traces, or correctness.

### Recommendation

Fix BURST-GAP-001 before the next wave gate (remove one of the two duplicate `<!-- v1.4: ... -->` comments from BC-INDEX line 843 for BC-2.18.003). No other remediation required.

---

## Appendix A: Methodology

All reads performed using the Read and Bash tools. Source-of-truth tier checked first (mitre.rs for tactic cross-validation), then indexes, then individual BCs. No files were modified. BC count verified by `find .factory/specs/behavioral-contracts -name "BC-*.md" | wc -l` (379 total minus BC-INDEX.md = 378). Input-hash values verified by targeted grep across ss-19/, ss-05/, ss-10/, ss-12/, ss-18/ directories. VP totals verified directly from verification-coverage-matrix.md Totals row (line 254: 16+22+3+6=47).
