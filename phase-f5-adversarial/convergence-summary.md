---
document_type: f5-convergence-summary
producer: adversary-via-orchestrator
date: 2026-07-17
cycle: feature-iec104
rounds: 5
develop_head_at_convergence: b36b884
---

# F5 Scoped Adversarial Convergence Summary — feature-iec104

## Configuration

| Field | Value |
|-------|-------|
| Cycle | feature-iec104 (IEC 60870-5-104 passive analyzer, E-22) |
| Scope | feature-iec104 delta: STORY-167..174 + FIX-P4-001 |
| Base commit | fedcea4 (v0.12.1 release, main HEAD) |
| Rounds | 5 |
| develop HEAD at convergence | b36b884 (FIX-F5-004 merged, PR #415) |
| Feature code frozen since | R2 — 9c5aa9a (FIX-F5-001) |

---

## Round-by-Round Trajectory

| Round | Reviewed SHA | Findings | Disposition |
|-------|-------------|----------|-------------|
| R1 | 7e95f71 | 1 HIGH (F-01 BC-2.19.011 PC-3 source_ip) + 4 MEDIUM (F-02..F-05 parity/prose) | → FIX-F5-001 (PR #411 9c5aa9a) |
| R2 | 9c5aa9a | 0 code + 2 MEDIUM doc-accuracy (F5R2-01 provenance, F5R2-02 fabricated T0881 JSON) | Code CONVERGED; → FIX-F5-002 (PR #412 b356545) |
| R3 | b356545 | 1 HIGH doc-accuracy (F-B1 fabricated FIX-P4-001 demo evidence ×4 files) | → FIX-F5-003 (PR #413 9eab53f) |
| R4 | 9eab53f | 1 MEDIUM doc-accuracy (F-B2 CHANGELOG Example-3 mitre_techniques [] vs ["T0814"]) | → FIX-F5-004 (PR #415 b36b884) |
| R5 | b36b884 | 0 CRITICAL/HIGH/MEDIUM; 1 LOW non-blocking (TypeID 45 prose mislabel, code correct) | NITPICK_ONLY → **CONVERGED** |

Findings trajectory: 5 → 2 → 1 (HIGH, docs) → 1 (MEDIUM, docs) → 0.

---

## Fix-PRs Delivered

| PR | Branch | SHA | Findings | develop HEAD at merge |
|----|--------|-----|----------|-----------------------|
| #411 | fix/FIX-F5-001 | 9c5aa9a | F-01..F-05 (1H+4M, code) | 9c5aa9a |
| #412 | fix/FIX-F5-002 | b356545 | F5R2-01/02 (2M, docs) | b356545 |
| #413 | fix/FIX-F5-003 | 9eab53f | F-B1 (1H, docs — fabricated FIX-P4-001 demo JSON ×4 files) | 9eab53f |
| #415 | fix/FIX-F5-004 | b36b884 | F-B2 (1M, docs — CHANGELOG Example-3 mitre_techniques) | b36b884 |

All four PRs human-executed merges (squash) per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER.

---

## Sweep Results at R5 (develop b36b884)

| Sweep | Result |
|-------|--------|
| BC-completeness | 31/31 PASS — all 31 feature-iec104 BCs (BC-2.19.001..031) have test coverage |
| Canonical-frame | 19/19 byte-exact — all IEC 60870-5-104 invariants undisturbed by fix-PRs |
| Kani non-vacuity | PASS — VP-044 89 checks (5 facets); VP-045/046 non-vacuous proptests |
| Fuzz | 1.95M execs, 0 crashes (VP-047) |
| Mutation score | 117/122 = 95.9% — unchanged from F4 convergence |

---

## Root Cause of R3–R5 Tail

The three post-code-convergence rounds (R3, R4, R5) were exclusively documentation-accuracy
issues. Root cause: **PG-DEMO-JSON-FABRICATION** — the demo-recorder hand-wrote JSON/enum
values rather than deriving them from actual `cargo run`/`cargo test` serialized output.
This produced three occurrences: FIX-F5-001 report (R2 F5R2-02), FIX-P4-001 demo-evidence
×3 artifacts (R3 F-B1), and the FIX-F5-003 CHANGELOG Example-3 (R4 F-B2).

Feature code and tests were sound throughout; all five F4 per-story adversarial convergences
(STORY-167..174) remained valid.

---

## Carry-Forward (Non-Blocking)

**IEC104-DEMO-TYPEID45-MISLABEL (LOW):** TypeID 45 (C_SC_NA_1, Single Command / control
command) described as "monitoring direction" in `docs/demo-evidence/FIX-P4-001/
evidence-report.md:46` and `AC-P4-001-test-results.txt:61`. Production code is correct at
`iec104.rs:744-748`. Prose-only; non-blocking. Fold into next docs-currency sweep or
cycle-close.

---

## Final Verdict

**F5 CONVERGED** — feature-iec104 delta at develop `b36b884`.

5 rounds. Feature code frozen since R2 (9c5aa9a); R3–R5 tail was documentation-accuracy
only. BC-completeness 31/31, canonical-frame 19 byte-exact, Kani non-vacuity PASS, fuzz
1.95M execs 0 crashes, mutants 117/122=95.9%.

**Next phase:** F6 targeted hardening (`vsdd-factory:phase-f6-targeted-hardening`) on the
feature-iec104 delta, then F7 delta convergence → release cut.
