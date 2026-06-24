---
document_type: consistency-audit
feature: feature-enip-v0.11.0 (EtherNet/IP + CIP Analyzer — SS-17)
scope: F2 spec-evolution delta
auditor: consistency-validator
timestamp: 2026-06-24T00:00:00Z
verdict: CONSISTENT WITH DEFECTS (5 findings; 0 CRITICAL, 1 HIGH, 3 MEDIUM, 1 LOW)
traces_to: .factory/phase-f2-spec-evolution/enip-prd-delta.md
---

# EtherNet/IP + CIP Analyzer — F2 Consistency Audit

## Audit Context

Fresh-context cross-document audit of the F2 spec-evolution delta for
`feature-enip-v0.11.0`. This pass reviews 24 BCs (BC-2.17.001..024), ADR-010,
VP-032, BC-INDEX.md (v1.74), ARCH-INDEX.md (v1.7), VP-INDEX.md (v2.11), the
MITRE research source (`enip-mitre-ics-tagging.md`), and the F2 delta docs
(`enip-architecture-delta.md`, `enip-prd-delta.md`).

---

## Check Results

### Check 1 — Index↔File Consistency

**Status: PASS**

**Evidence:**

- **File existence:** All 24 files BC-2.17.001.md through BC-2.17.024.md exist on
  disk at `.factory/specs/behavioral-contracts/ss-17/`.
- **Frontmatter uniformity:** Spot-checked BC-2.17.001, .002, .011, .012, .013,
  .015, .016, .018, .019, .021. All carry:
  - `document_type: behavioral-contract`
  - `level: L3`
  - `version: "1.0"`
  - `subsystem: SS-17`
  - `capability: CAP-17`
  - `lifecycle_status: active`
  - `producer: product-owner`
  - `timestamp: 2026-06-24T00:00:00Z`
  - `introduced: v0.11.0-feature-enip`
  - `traces_to: .factory/specs/domain/domain-spec.md` (consistent with DNP3/Modbus
    BC pattern)
- **BC-INDEX row alignment:** BC-INDEX.md v1.74 §ss-17 table contains exactly
  24 rows (BC-2.17.001..024), all marked `[WRITTEN]`, with correct origin
  `feature-enip-v0.11.0`. Row titles match BC H1 headings for all spot-checked BCs.
- **Arithmetic verification:**
  - Prior total on disk: 305 (BC-INDEX v1.73 state)
  - New: +24 (BC-2.17.001..024)
  - Result: 305 + 24 = **329** ✓ (BC-INDEX header claims 329)
  - Prior active: 304 (305 on disk − 1 retired BC-2.01.004)
  - New active: 304 + 24 = **328** ✓ (BC-INDEX header claims 328)
  - BC-INDEX v1.74 header: "305 prior + 24 new BC-2.17.001..024" — arithmetic
    correct.
- **BC-INDEX version:** v1.74 is consistent with the v1.73 prior state (pre-F2
  EtherNet/IP) recorded in `enip-prd-delta.md` frontmatter
  (`bc_index_version_before: "1.73"`, `bc_index_version_after: "1.74"`).
- **input-hash fields:** All 24 BCs carry `input-hash: TBD`. This is appropriate
  for F2 (spec-only phase); the hash will be computed at F3 story decomposition
  or pre-Phase-4 gate per the canonical algorithm documented in CLAUDE.md. No
  violation.

---

### Check 2 — Traceability Completeness

**Status: PASS**

**Evidence:**

- **CAP-17 → SS-17:** ARCH-INDEX.md v1.7 (2026-06-24 modification) adds SS-17
  "EtherNet/IP + CIP Analysis" to the Subsystem Registry with CAP-17, 24 BCs,
  and `analyzer/enip.rs`. CAP-17 domain capability file registered at
  `.factory/specs/domain/capabilities/cap-17-enip-cip-analysis.md` per
  `enip-prd-delta.md` §Domain Capability Registration.
- **SS-17 → each BC:** All 24 BCs carry `subsystem: SS-17` and `capability: CAP-17`
  in frontmatter. BC-INDEX §ss-17 section lists all 24. No orphaned BCs.
- **BC → VP-032:** VP-032 frontmatter `bcs:` array lists exactly
  [BC-2.17.001, .002, .003, .004, .007]. VP-032 Verified BCs table lists the
  same 5 BCs (Sub-A: 001/002, Sub-B: 004, Sub-C: 003, Sub-D: 007). VP citations
  in BC body Verification Properties tables confirmed:
  - BC-2.17.001 VP table: VP-032 Sub-A ✓
  - BC-2.17.002 VP table: VP-032 Sub-A ✓
  - BC-2.17.011 VP table: VP-032 Sub-D (indirect) ✓
  - BC-2.17.015 VP table: VP-032 Sub-D (indirect) ✓
  - BC-2.17.016 VP table: VP-032 Sub-A (indirect) ✓
  - BC-2.17.018 VP table: VP-032 Sub-C (indirect) ✓
- **VP-INDEX accuracy:** VP-INDEX.md v2.11 row for VP-032 lists Verified BCs as
  "BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.007" — matches
  VP-032 file exactly. No dangling references.
- **No BC claims a non-existent VP:** Non-Kani BCs (BC-2.17.008..010, .012..018,
  .020..024) correctly cite "(none)" or indirect VP-032 references. No BC in
  SS-17 references VP-001 through VP-031.
- **Forward/backward trace:** BC frontmatter `traces_to: domain-spec.md` (backward
  to L2). BC-INDEX SS-17 section (forward index). VP-032 → ARCH-INDEX + ADR-010
  (upward). ADR-010 → existing ADRs (005, 007 pattern). No orphans detected.

---

### Check 3 — MITRE Tag Correctness

**Status: PASS WITH NOTE**

**Evidence:**

**Verified tag mapping against `enip-mitre-ics-tagging.md` (all BCs that emit techniques):**

| BC | Technique emitted | Research source | Match? |
|----|------------------|-----------------|--------|
| BC-2.17.010 | T0846 (IcsDiscovery TA0102) | §4b: T0846 ListIdentity ✓ | MATCH |
| BC-2.17.011 | T0858 (IcsExecution TA0104) | §1: T0858 CIP Stop ✓ | MATCH |
| BC-2.17.012 | T0836 (IcsImpairProcessControl TA0105) | §5: T0836 write ✓ | MATCH |
| BC-2.17.013 | T0816 (IcsInhibitResponseFunction TA0107) | §2: T0816 Reset ✓ | MATCH |
| BC-2.17.014 | T0888 (IcsDiscovery TA0102) | §4a: T0888 identity-read ✓ | MATCH |
| BC-2.17.015 | vec![] (intentionally empty) | §7: ForwardOpen = no dedicated technique ✓ | MATCH |
| BC-2.17.018 | T0814 (IcsInhibitResponseFunction TA0107) | T0814 malformed/DoS (seeded) ✓ | MATCH |

**Revoked IDs confirmed absent:**

- T0855, T0856, T0857 — searched all 24 BCs; none found. ✓
- Research file explicitly confirms revocation of these three IDs (updated per issue
  #222 for T0855/T0856; T0857→T1693.001 confirmed by research).

**T1693.001 staged-only (correct):**

- ADR-010 Decision 7 and `enip-prd-delta.md` both confirm T1693.001 is seeded in
  the catalog but NOT emitted in v0.11.0 (firmware detection is staged). No BC in
  BC-2.17.001..024 emits T1693.001. ✓

**NOTE (not a defect — informational):** The research file §Catalogue impact suggests
the tactic note for T0858 in existing tooling "leans IcsImpairProcessControl" but
ADR-010 Decision 7 (the authoritative decision document) explicitly adopts
`IcsExecution` (TA0104) as the primary tactic. The BC-INDEX v1.74 header and
BC-2.17.011 body both consistently use `IcsExecution TA0104`. The research file
note is pre-decisional background; the ADR is authoritative. No inconsistency.

---

### Check 4 — Scope Consistency (UDP/2222 Deferred, ForwardOpen over TCP)

**Status: PASS**

**Evidence:**

- **UDP/2222 deferred:** Searched all 24 BC files. No BC describes UDP/2222 cyclic
  I/O detection. ADR-010 Decision 6 explicitly defers UDP/2222 to post-v0.11.0.
  Deferred MITRE tags for UDP/2222 (T1692.001 output injection, T1692.002 input
  spoofing) are documented in ADR-010 as staged-only and never emitted in any BC.
  The `enip-prd-delta.md` confirms D-229 deferral applies.
- **TCP/44818 scope confirmed:** BC-2.17.019 specifies Rule 7 as TCP port 44818
  classification only. All detection BCs (BC-2.17.010..015, .018) are reached via
  the ENIP TCP frame-walk loop in `on_data()` — exclusively TCP/44818 reachable.
- **ForwardOpen over TCP:** BC-2.17.015 — ForwardOpen connection-establishment
  detection is via TCP/44818 `SendRRData`/`SendUnitData` frames. The
  `classify_cip_service(0x54 / 0x5B)` dispatch fires within the TCP ENIP frame-walk
  loop. No UDP path exists for this BC. ✓
- **BC-INDEX ss-17 section header confirms:** "BCs 001-007: ENIP parse and
  classification (Group A–D)... Detection BCs (Group F — finding emission): T0846
  (ListIdentity)..." — all detection BCs are TCP-44818 reachable.

---

### Check 5 — ADR↔BC Alignment

**Status: PASS WITH MINOR NOTE**

**Evidence:**

**ADR-010 Decision ↔ BC mapping:**

| ADR-010 Decision | Expected BC | Observed in BC | Consistent? |
|------------------|-------------|----------------|-------------|
| Decision 1: Rule 7 port 44818 | BC-2.17.019 | BC-2.17.019 description: Rule 7 after DNP3 Rule 6 ✓ | YES |
| Decision 2: two-level parser, 24-byte header | BC-2.17.001/002 | BC-2.17.001: `parse_enip_header` None for <24 ✓; BC-2.17.002: BE field layout ✓ | YES |
| Decision 3: MAX_ENIP_CARRY_BYTES=600 | BC-2.17.016 | BC-2.17.016 Invariant 1: "flow.carry.len() <= MAX_ENIP_CARRY_BYTES = 600" ✓ | YES |
| Decision 4: EnipFlowState design | BC-2.17.016/017/024 | BC-2.17.016 carry-walk, BC-2.17.017 on_flow_close, BC-2.17.024 pdu_count ✓ | YES |
| Decision 5: ForwardOpen in-scope | BC-2.17.015 | BC-2.17.015 Postcondition 1 + Description: ForwardOpen detection ✓ | YES |
| Decision 6: UDP/2222 deferred | (no BC) | No BC covers UDP/2222 ✓ | YES |
| Decision 7: T0858 for CIP Stop | BC-2.17.011 | BC-2.17.011 Invariant 1: "T0858 is the correct v19.1 technique" ✓; `IcsExecution TA0104` ✓ | YES |
| Decision 7: ForwardOpen mitre_techniques vec![] | BC-2.17.015 | BC-2.17.015 Postcondition 1: `mitre_techniques: vec![]` ✓; evidence string cites "ADR-010 Decision 7" ✓ | YES |
| Decision 7: SEEDED 25→28, EMITTED 17→19 | (mitre.rs obligation) | Consistently stated in ADR-010 Consequences + VP-007 atomic obligation + BC-INDEX v1.74 header ✓ | YES |

**Minor Note (not a defect):** ADR-010 Decision 7 VP-007 obligation header reads
"5-part" and lists items 1–5. `enip-architecture-delta.md` §VP-007 obligation lists
the same "5-part" header but adds a 6th step ("Run `cargo test mitre`"). The extra
step is implementation guidance, not a conflicting architectural decision. The 5
normative items are identical in both documents. LOW severity informational note
only; no remediation required.

---

### Check 6 — Pattern Parity with DNP3 (SS-15)

**Status: PASS WITH ONE DEFECT**

**Evidence:**

**Confirmed parity items:**

| Pattern | DNP3 (SS-15) | EtherNet/IP (SS-17) | Consistent? |
|---------|-------------|---------------------|-------------|
| `traces_to` in frontmatter | `domain-spec.md` | `domain-spec.md` | YES |
| lifecycle frontmatter fields | `lifecycle_status`, `deprecated`, `deprecated_by`, etc. | Same fields present ✓ | YES |
| MAX_FINDINGS cap | BC-2.15.NNN cites 10000 | BC-2.17.022 cites MAX_FINDINGS=10000 ✓ | YES |
| parse_errors key naming | BC-2.15.020 v1.4: `parse_errors` (not `total_parse_errors`) | BC-2.17.021 Invariant 1: "must be `parse_errors` (not `total_parse_errors`)" with explicit lesson cite ✓ | YES |
| is_non_enip/is_non_dnp3 latch | `is_non_dnp3: bool` permanent latch | BC-2.17.016 Invariant 4: "is_non_enip is permanent" ✓; mirrors is_non_dnp3 pattern ✓ | YES |
| malformed anomaly threshold | BC-2.15.024: MALFORMED_ANOMALY_THRESHOLD=3 | BC-2.17.018: MALFORMED_ANOMALY_THRESHOLD=3 ✓ | YES |
| two-counter model (parse_errors + windowed) | BC-2.15.024 §Invariants | BC-2.17.018 Invariant 1: "Two-counter model (mirrors BC-2.15.024)" ✓ | YES |
| VP-032 structure (4 sub-harnesses) | VP-023 had 4 harnesses | VP-032 has 4 harnesses (Sub-A/B/C/D) ✓; mirror pattern stated in VP-032 Feasibility ✓ | YES |
| summarize() key pattern | DNP3 uses `parse_errors` | BC-2.17.021: `parse_errors` key named explicitly ✓ | YES |

**DEFECT F6-001 (MEDIUM) — BC-INDEX SS-17 Group F label mislabels ForwardOpen technique:**

BC-INDEX.md §ss-17 section comment header states:

```
> BCs 010-015: Detection BCs (Group F — finding emission): T0846 (ListIdentity), T0858 (CIP Stop),
>   T0836 (write-burst), T0816 (CIP Reset), T0888 (Identity-read/error-burst), T1692.001 (ForwardOpen).
```

This labels BC-2.17.015 (ForwardOpen) as emitting "T1692.001", but BC-2.17.015 explicitly
emits `mitre_techniques: vec![]` (empty, per ADR-010 Decision 7). The correct label is
"(none — mitre_techniques: vec![])" or "(ForwardOpen — no technique)". The BC-INDEX table
row for BC-2.17.015 in the same section correctly states `(none — mitre_techniques: vec![])`,
so there is an internal inconsistency within BC-INDEX itself: the section comment header
says T1692.001, the table row says none. The body BC file and ADR-010 are both authoritative
and correct (empty). The comment header is incorrect metadata.

**Severity:** MEDIUM — this is a spec document comment inconsistency, not a normative BC
body defect. It will not affect implementation (the table row and BC body are authoritative)
but could mislead a story-writer or implementer reading the section header.

**Remediation:** Correct BC-INDEX.md §ss-17 section header to read:
```
> BCs 010-015: Detection BCs (Group F — finding emission): T0846 (ListIdentity), T0858 (CIP Stop),
>   T0836 (write-burst), T0816 (CIP Reset), T0888 (Identity-read/error-burst), (ForwardOpen — no technique).
```

---

### Check 7 — Internal Numeric/Threshold Consistency

**Status: PASS WITH DEFECTS**

**Evidence:**

**Constants verified across documents:**

| Constant | ADR-010 | enip-architecture-delta | BC bodies | Consistent? |
|----------|---------|------------------------|-----------|-------------|
| MAX_ENIP_CARRY_BYTES = 600 | Decision 3 ✓ | §4.2 ✓ | BC-2.17.016 Invariant 1 ✓ | YES |
| MAX_FINDINGS = 10000 | Decision 8 (implied) | §4.2 ✓ | BC-2.17.022 ✓ | YES |
| MALFORMED_ANOMALY_THRESHOLD = 3 | (implied by DNP3 pattern) | §4.2 ✓ | BC-2.17.018 Invariant 3 ✓ | YES |
| write-burst default = 20/1s | Decision 9 ✓ | (referenced) | BC-2.17.012 ✓ | YES |

**Consistent "write burst window = 1 second":** BC-2.17.012 description "within the
1-second window" ✓, consistent with ADR-010 Decision 9 `--enip-write-burst-threshold
(u32, default: 20)` with 1s window.

**Consistent "malformed window = 300 seconds":** BC-2.17.018 "Within the correlation
window (proposed: 300s)" + EC/Canonical Test Vectors. `enip-architecture-delta.md`
§4.1 field `malformed_in_window` — 300s matches the DNP3 pattern established in
BC-2.15.024.

**SEEDED/EMITTED counts verified:**

| Claim | Source | Value | Cross-reference | Consistent? |
|-------|--------|-------|-----------------|-------------|
| SEEDED 25→28 | ADR-010 Consequences ✓ | +3 (T0858, T0816, T1693.001) | enip-prd-delta.md §O-04 "25→28" ✓ | YES |
| EMITTED 17→19 | ADR-010 Consequences ✓ | +2 (T0858, T0816) | enip-prd-delta.md §O-04 "17→19" ✓ | YES |
| T1693.001 staged-only (SEEDED not EMITTED) | ADR-010 Decision 7 ✓ | No BC emits T1693.001 ✓ | enip-architecture-delta §VP-007 step 4 ✓ | YES |

---

**DEFECT F7-001 (HIGH) — BC-2.17.007 is VP-032 Sub-D Kani target but is NOT listed in BC-INDEX v1.74 changelog VP citation list**

BC-INDEX v1.74 header states: "VP citations changed in: BC-2.17.001..007 (VP-032 Sub-A/B/C/D)."
This is actually consistent (BCs 001–007 span Sub-A through Sub-D). However, examining
closely: VP-032 Sub-D is `classify_cip_service` — BC-2.17.007. The BC-INDEX header lists
"BC-2.17.001..007" which covers 001, 002, 003, 004, 005, 006, 007. But BC-2.17.005 and
BC-2.17.006 are NOT Kani-targeted BCs (they are pure parse functions for CPF and CIP header
but are not in VP-032's `bcs:` frontmatter). VP-032 `bcs:` = [001, 002, 003, 004, 007].

The statement "VP citations changed in: BC-2.17.001..007" is imprecise (implies all 7 BCs have
VP-032 citations) but BCs 005 and 006 do not cite VP-032 as a direct proof target (they have
"(none — enables T0858/T0816/T0836/T0888)" in the VP table). This is a metadata inaccuracy:
the range notation implies a contiguous block when the actual VP-cited set is {001, 002, 003,
004, 007} — a non-contiguous set.

**Severity:** HIGH — the inaccuracy is in a header annotation, not in the normative BC body
or VP-032 file (both of which are correct). However, it represents a potential confusion
source for the story-writer and implementer about which BCs carry Kani obligations. The VP-032
file and BC bodies are the authoritative sources and are correct; the BC-INDEX header note is
an annotation that should be fixed for clarity.

**Remediation:** Correct BC-INDEX.md v1.74 header annotation to read:
"VP citations changed in: BC-2.17.001, .002, .003, .004, .007 (VP-032 Sub-A/B/C/D; BCs 005
and 006 are pure-core parse functions enabling detections but are not formal Kani targets in
VP-032)."

---

**DEFECT F7-002 (MEDIUM) — VP-007 obligation step count: ADR-010 says "5-part" but enip-architecture-delta lists 6 steps**

ADR-010 Decision 7 reads: "VP-007 atomic obligation **(5-part**, mirrors ADR-007 Decision 5
playbook):" and lists exactly 5 numbered items (1: technique_info arms, 2: SEEDED array,
3: SEEDED_ID_COUNT bump, 4: EMITTED_IDS additions, 5: MitreTactic::IcsExecution).

`enip-architecture-delta.md` §VP-007 atomic obligation reads: "VP-007 atomic obligation
**(5-part burst,** STORY-EIP-09)" but lists 6 numbered items — the same 5 normative items
plus a 6th: "Run `cargo test mitre` to confirm `vp007_catalog_drift_guard` passes."

The 6th item is implementation guidance (a test verification step), not a new normative
obligation, but the numbering creates an apparent count discrepancy between the two
documents.

**Severity:** MEDIUM — no normative content is lost; both documents agree on the 5 normative
atomic changes. The discrepancy is presentational and could cause confusion when the
story-writer counts steps in the STORY-EIP-09 VP-007 burst.

**Remediation:** Align `enip-architecture-delta.md` §VP-007 to either: (a) keep 5 items and
move `cargo test mitre` to a "Verification:" note outside the numbered list, or (b) update
the header to read "(6-step burst)" and add a note clarifying step 6 is verification, not
an atomic code change.

---

**DEFECT F7-003 (MEDIUM) — enip-prd-delta.md §MITRE Catalog Delta row for T1693.001 description inconsistency**

`enip-prd-delta.md` §MITRE Catalog Delta includes:

```
| T1693.001 | (GetAndClear firmware service) | (future) | staged not emitted | Seed but do not emit in v0.11.0 |
```

The "Name" cell reads "(GetAndClear firmware service)" which is not the technique name.
T1693.001's canonical ATT&CK name is "Modify Firmware: System Firmware" (per
`enip-mitre-ics-tagging.md` §3 and ADR-010 Decision 7 table). "GetAndClear" is
apparently a CIP service code that might trigger this technique, not the technique name
itself.

**Severity:** MEDIUM — the row is in a delta document (not a BC body) and T1693.001 is
staged-only (never emitted in v0.11.0), so this is unlikely to cause implementation error.
However, it could confuse the F4 story-writer building STORY-EIP-09 who reads the delta
first.

**Remediation:** Correct `enip-prd-delta.md` T1693.001 row Name cell to "Modify Firmware:
System Firmware" and add a Notes cell "Trigger: CIP firmware download service (0x4B or
vendor-specific); staged per ADR-010 Decision 7".

---

**DEFECT F7-004 (LOW) — VP-007 enip-architecture-delta step 6 cargo test step adds an undocumented 6th item but this is also present in ADR-010 Consequences**

*(already partially covered in F7-002; noting separately for completeness as a distinct
artifact site)*

`enip-architecture-delta.md` §VP-007 item 6 says "Run `cargo test mitre` to confirm
`vp007_catalog_drift_guard` passes." ADR-010 §Decision 7 lists this implicitly in the
Consequences section ("Run `cargo test mitre` to confirm `vp007_catalog_drift_guard`
passes") but not in the 5-part obligation list. The test step is undocumented in ADR-010's
numbered list and appears only in the architecture-delta. This is a minor process gap:
there is no authoritative home for "run this test" as part of the atomic burst.

**Severity:** LOW — does not affect correctness; the test must obviously be run. The
ADR-010 Consequences section implies it ("VP-007 formal correctness is preserved after
the 5-part atomic update"). This is presentation/process guidance rather than a
normative specification defect.

**Remediation:** In ADR-010 Decision 7 VP-007 obligation, add after item 5: "6.
**Verification:** Run `cargo test mitre` to confirm `vp007_catalog_drift_guard` passes.
This is a correctness gate, not part of the atomic code change."

---

## Findings Summary

| ID | Check | Status | Finding | Severity |
|----|-------|--------|---------|---------|
| — | 1: Index↔file consistency | PASS | All 24 BCs exist; frontmatter consistent; arithmetic 305+24=329 on disk, 304+24=328 active ✓ | — |
| — | 2: Traceability completeness | PASS | CAP-17→SS-17→each BC→VP-032 chain intact; VP-032 bcs: [001,002,003,004,007] consistent with BC bodies ✓ | — |
| — | 3: MITRE tag correctness | PASS | All tags match research file; no revoked IDs (T0855/T0856/T0857 absent); T1693.001 staged-only ✓ | — |
| — | 4: Scope consistency | PASS | No UDP/2222 detection; ForwardOpen over TCP/44818 only ✓ | — |
| — | 5: ADR↔BC alignment | PASS | All 9 ADR-010 Decisions reflected correctly in BCs; MAX_ENIP_CARRY_BYTES=600 ✓ | — |
| F6-001 | 6: Pattern parity | DEFECT | BC-INDEX §ss-17 section comment header mislabels BC-2.17.015 (ForwardOpen) as emitting "T1692.001"; BC body and table row correctly say `vec![]` | MEDIUM |
| — | 6: Pattern parity (rest) | PASS | parse_errors key, is_non_enip latch, MAX_FINDINGS=10000, MALFORMED_ANOMALY_THRESHOLD=3, two-counter model — all consistent with DNP3 pattern ✓ | — |
| F7-001 | 7: Numeric/threshold consistency | DEFECT | BC-INDEX v1.74 header "VP citations changed in: BC-2.17.001..007" is imprecise — non-contiguous set {001,002,003,004,007}; BCs 005 and 006 do NOT carry VP-032 as a direct target | HIGH |
| F7-002 | 7: Numeric/threshold consistency | DEFECT | ADR-010 calls VP-007 obligation "5-part" (5 items); enip-architecture-delta lists same header "5-part" but enumerates 6 steps | MEDIUM |
| F7-003 | 7: Numeric/threshold consistency | DEFECT | enip-prd-delta.md T1693.001 "Name" cell reads "(GetAndClear firmware service)" instead of canonical "Modify Firmware: System Firmware" | MEDIUM |
| F7-004 | 7: Numeric/threshold consistency | DEFECT | VP-007 step 6 (cargo test) undocumented in ADR-010's numbered obligation list; only appears in architecture-delta | LOW |

---

## Overall Verdict

**CONSISTENT WITH DEFECTS**

The F2 spec-evolution delta for `feature-enip-v0.11.0` is **internally consistent and
consistent with the existing analyzer corpus** on all normative dimensions. The 24 BCs are
well-formed, the traceability chain (CAP-17 → SS-17 → BC → VP-032) is intact, all MITRE
tags are correct per ics-attack-19.1, no revoked IDs are present, no UDP/2222 scope
violations exist, and the DNP3 naming and structural pattern is faithfully replicated
(including the critical `parse_errors` key rename lesson from BC-2.15.020 v1.4 D-220).

Five defects were identified, none of which block F3 story decomposition or F4
implementation:

- **F7-001 (HIGH):** BC-INDEX header annotation imprecisely lists BCs 001..007 as VP-032
  citation sites when the correct set is {001, 002, 003, 004, 007}. The VP-032 file and
  BC bodies are authoritative and correct. Fix the annotation before story-writer uses it.
- **F6-001 (MEDIUM):** BC-INDEX §ss-17 section comment header labels ForwardOpen (BC-2.17.015)
  as "T1692.001 (ForwardOpen)" — should be "(ForwardOpen — no technique)". BC body is correct.
- **F7-002 (MEDIUM):** VP-007 obligation step count discrepancy between ADR-010 (5 items)
  and architecture-delta (6 steps). Align before STORY-EIP-09 is written.
- **F7-003 (MEDIUM):** enip-prd-delta.md T1693.001 row uses wrong technique name.
- **F7-004 (LOW):** VP-007 cargo-test verification step is undocumented in ADR-010's
  numbered list.

No normative BC content, no VP-032 property, no MITRE assignment, no constant, and no
architectural decision is incorrect. All five findings are annotation/metadata defects in
index or delta documents, not in BC bodies, ADR-010, or VP-032.

**Gate recommendation:** PASS for F3 story decomposition, subject to fixing F7-001 (HIGH)
before story-writer reads BC-INDEX §ss-17 VP citation note. F6-001, F7-002, F7-003 should
be corrected in the same pass. F7-004 may be deferred to ADR-010 cleanup.

---

## Artifact Versions Audited

| Artifact | Version | Date |
|----------|---------|------|
| BC-INDEX.md | v1.74 | 2026-06-24 |
| ARCH-INDEX.md | v1.7 | 2026-06-24 |
| VP-INDEX.md | v2.11 | 2026-06-24 |
| ADR-010-ethernet-ip-cip-stream-dispatch.md | proposed (v1.0) | 2026-06-24 |
| vp-032-enip-parse-safety.md | (initial) | 2026-06-24 |
| enip-architecture-delta.md | v1.0 | 2026-06-24 |
| enip-prd-delta.md | v1.0 | 2026-06-24 |
| enip-mitre-ics-tagging.md | COMPLETE | 2026-06-24 |
| BC-2.17.001..024 (all 24 files) | v1.0 each | 2026-06-24 |
