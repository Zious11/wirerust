---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: archive
producer: state-manager
timestamp: 2026-07-18T04:30:00Z
cycle: "feature-iec104"
inputs: []
input-hash: "d41d8cd"
traces_to: STATE.md
---

# Convergence Trajectory — feature-iec104

Per-story convergence status extracted from STATE.md Convergence Status table on 2026-07-18.
All stories are CONVERGED (feature-iec104 F4 complete). Full adversarial pass details are in
the per-story convergence reports under `cycles/feature-iec104/STORY-NNN/`.

## Finding Progression (Per-Story Summary)

| Cycle/Story | Passes | Trajectory | Status |
|------------|--------|-----------|--------|
| feature-iec104 F4 per-story (STORY-171) | 4 | →2→0→0→0 | CONVERGED 3-clean (BC-5.39.001) |
| feature-iec104 F4 per-story (STORY-172) | 6 | →(2H+1L+1N)→1L→1NIT→0→0→0 | CONVERGED 3-clean (BC-5.39.001) streak P4/P5/P6 |
| feature-iec104 F4 per-story (STORY-173) | 17 (14+3) | →(1H+3doc)→(1M+1N)→NITs→CLEAN(P6)→1N→4N→CLEAN(P9/P10)→1N→CLEAN(P12/P13/P14)→3LOWfix→CLEAN(A/B/C) | CONVERGED 3-clean (BC-5.39.001) re-converged A/B/C post-LOW-fixes (D-458) |
| feature-iec104 F4 per-story (STORY-174) | 7 | →1M→1M→NIT→1M→0→0→0 | CONVERGED 3-clean streak P5/P6/P7 (BC-5.39.001) D-462 |

## F5 Scoped Adversarial (Phase-level)

| Round | Date | Findings | Code State | Verdict |
|-------|------|----------|------------|---------|
| R1 | 2026-07-17 | 1H+4M (source_ip/prose) | develop 7e95f71 | FINDINGS_REMAIN → FIX-F5-001 |
| R2 | 2026-07-17 | 2M (doc-accuracy) | develop 9c5aa9a (code CONVERGED) | FINDINGS_REMAIN → FIX-F5-002 |
| R3 | 2026-07-17 | 1H (demo-evidence fabrication) | develop b356545 | FINDINGS_REMAIN → FIX-F5-003 |
| R4 | 2026-07-17 | 1M (CHANGELOG) | develop 9eab53f | FINDINGS_REMAIN → FIX-F5-004 |
| R5 | 2026-07-17 | 1L (non-blocking, TypeID-45 mislabel prose) | develop b36b884 | NITPICK_ONLY — CONVERGED (D-468) |

## Trajectory Shorthand (F5)

`5M→2M→1H→1M→1L(NB)` — code frozen since R2; R3–R5 tail was doc-accuracy only.

## Per-Pass Details

### Pass F5-R1 (2026-07-17)

**Findings:** 5 (0 CRIT, 1 HIGH, 4 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0/3

BC-completeness sweep 31/31 PASS; canonical-frame sweep 19 invariants byte-exact. F-01 HIGH BC-2.19.011 PC-3 source_ip unmet (untested blind spot). F-02/03/04/05 MEDIUM: source_ip/timestamp parity, stale prose (4 siblings), false forward-ref, stale count. All 5 batched to FIX-F5-001.

---

### Pass F5-R2 (2026-07-17)

**Findings:** 2 (0 CRIT, 0 HIGH, 2 MED, 0 LOW)
**Novelty:** MEDIUM
**Convergence counter:** 0/3

Code CONVERGED — all R1 findings verified fixed; direction→source_ip DNP3-parity-exact; tests non-vacuous. F5R2-01 MEDIUM wrong provenance (S-139/S-140 lineage), F5R2-02 MEDIUM fabricated T0881 JSON → FIX-F5-002.

---

### Pass F5-R3 (2026-07-17)

**Findings:** 1 (0 CRIT, 1 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH (docs/demo-evidence)
**Convergence counter:** 0/3

R2 doc-fixes verified. F-B1 HIGH: FIX-P4-001 demo-evidence artifacts still fabricated (category 'Protocol'/verdict 'Anomaly'/confidence 'High' — non-existent variants; wrong MITRE technique). Root cause: demo-recorder hand-writing JSON. PG-DEMO-JSON-FABRICATION filed → FIX-F5-003.

---

### Pass F5-R4 (2026-07-17)

**Findings:** 1 (0 CRIT, 0 HIGH, 1 MED, 0 LOW)
**Novelty:** MEDIUM (docs)
**Convergence counter:** 0/3

F-B1 verified fixed. F-B2 MEDIUM: FIX-F5-002 CHANGELOG false-correction claim + CHANGELOG Example-3 mitre_techniques [] → ["T0814"] → FIX-F5-004.

---

### Pass F5-R5 (2026-07-17)

**Findings:** 1 (0 CRIT, 0 HIGH, 0 MED, 1 LOW non-blocking)
**Novelty:** LOW
**Convergence counter:** 3/3 — NITPICK_ONLY CONVERGED

F-B2 verified fixed. 1 LOW non-blocking: TypeID 45 (C_SC_NA_1 control command) described as "monitoring direction" in demo-evidence prose; code correct at iec104.rs:744-748. Feature code frozen since R2 (9c5aa9a). BC-completeness 31/31 + canonical-frame 19 byte-exact clean. F5 gate PASSED (D-468).

---

## Frontmatter Fields (extracted from STATE.md)

<!-- No adversary_pass_* frontmatter fields were present in STATE.md for this cycle.
     The F4 per-story convergence data above was extracted from the Convergence Status
     table in STATE.md body. The F5 phase-level pass data was extracted from Decisions
     Log entries D-465..D-468. -->

