---
document_type: f5-remediation-record
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-06-12T15:23:29Z
cycle: "feature-8-dnp3-v0.5.0"
phase: feature-f5
pr: "#230"
merge_commit: e685664
merged_at: 2026-06-12T15:23:29Z
---

# F5 Remediation Record — Feature #8 DNP3 Scoped Adversarial

PR #230 MERGED to develop. Merge commit `e685664` (2026-06-12T15:23:29Z).
develop HEAD = e685664. main HEAD unchanged = c2df1b5 (v0.5.0).

---

## F5 Adversarial Review Scope

Scoped adversarial review of the DNP3 Feature #8 delta (changed/new code only).
Fresh context. Agentic-sliced pre-implementation adversarial review (3 parallel slices:
F-001 design, F-003 design, spec-consistency) run before TDD authoring.

---

## Issues Found and Fixed

### F-A-001 — PRE-EXISTING DIR-bit Bug (P0 BLOCKER)

`is_master_frame` masked bit 4 (0x10); DNP3 DIR is bit 7 (0x80). Canonical master
frames (CONTROL=0xC4) were not recognized as master frames; master_addrs_seen was
mis-populated with outstation addresses. Latent since STORY-107 (wave 36). Survived
~30 per-story adversarial passes because tests + BC-2.15.016 PC5 both used the same
wrong mask (self-consistent error).

Fix: mask 0x10 → 0x80. BC-2.15.016 → v1.3.

### F-F5-001 — Unexpected-Source Detection Entirely Unimplemented (P0 BLOCKER)

BC-2.15.010 Inv-5 PRIMARY gate (T1692.001 at count=1 from a non-first-seen master,
representing adversarial CONTROL injection) was absent from the codebase. A P0
holdout HS-W37-002 would have failed.

Fix: implemented `detect_unexpected_source_split` (first-seen-master learning,
pre-push snapshot, fall-through, one-shot, two-entry evidence).
BC-2.15.010 → v1.5 (EC-009/010/011 + dual-gate H1).

### F-F5-002 — IcsImpact/Impact Display Collision (MEDIUM)

`MitreTactic::IcsImpact` collided with existing Impact tactic in report sections.

Fix: "Impact" → "Impact (ICS)". No BC change.

### F-F5-003 — Byte-Walk Resync Silent Data Loss (Crain-Sistrunk Evasion) (HIGH)

Resync arm silently dropped malformed-frame accounting. Crain-Sistrunk evasion vector:
malformed frames could be partially processed without incrementing parse_errors.

Fix: unconditional resync arm + inline-resync in LENGTH-gate/overflow arms (count-once,
preserve head frame, no clear+return). BC-2.15.024 → v1.3 (three-path, principle-1).

---

## Convergence

| Pass | Result | Notes |
|------|--------|-------|
| P1 | BLOCKED | overflow under-count flag deviation |
| P2 | BLOCKED | evidence-field divergence |
| P3 | BLOCKED | BC-INDEX/STORY-108/PRD title-sync gaps |
| P4 | BLOCKED | cascading title-sync gaps |
| P5 | BLOCKED | further convergence |
| P6 | CLEAN | window 1/3 |
| P7 | BLOCKED | regression caught |
| P8 | CLEAN | window 2/3 |
| P9 | [process-gap] CHECKOUT-GUARD FAILURE | adversary reviewed develop not worktree; BLOCKERs invalid |
| P10 | CLEAN | window 3/3 — CONVERGED |

ARCHITECT REVISION-2 directives: 2 (original designs for F-001 and F-003 were unsound).

---

## Spec Changes (factory-artifacts)

| Spec | Version | Change |
|------|---------|--------|
| BC-2.15.009 | v1.2 | prose clarification |
| BC-2.15.010 | v1.5 | EC-009/010/011 + dual-gate H1 (unexpected-source) |
| BC-2.15.016 | v1.3 | PC5 mask corrected 0x10→0x80 |
| BC-2.15.024 | v1.3 | three-path resync, principle-1 |
| BC-INDEX | — | title sync |
| STORY-108 | — | title sync |
| PRD | — | title sync |
| STORY-106/107/108/109 + ~10 prd-input stories | input-hash | regenerated |

---

## F6-Gate Obligations

Carried from F4 delivery (3 pre-existing) + 1 new obligation added at F5 COMPLETE:

1. AC-005 Kani `verify_content_first_precedence_exhaustive` (STORY-106).
2. VP-023 draft→verified + VP-INDEX bump (after 4 STORY-106 Kani proofs).
3. VP-004 relock — include Rules 5/6.
4. NEW (F5): Confirm VP-023 Kani harnesses + any master-frame-dependent proof still
   hold under corrected `is_master_frame` mask (0x80). The DIR-bit fix changes the
   semantic meaning of master-frame classification; proofs must be re-validated.
