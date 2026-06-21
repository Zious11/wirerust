---
document_type: f5-convergence-summary
cycle: feature-pcapng-reader
phase: F5
status: CONVERGED
converged_at: 2026-06-21
develop_head_at_convergence: 3fc0e67
consecutive_clean_passes: 3
passes_total: 8
---

# F5 Scoped Adversarial Refinement — Convergence Summary
# Cycle: feature-pcapng-reader

**STATUS: CONVERGED — 3 consecutive clean adversarial passes (passes 6, 7, 8) on develop=3fc0e67.**
**PAUSED for human review before F6 (phase-by-phase cadence, D-186 binding).**

---

## Pass Record

### Pass 1 — NOT CLEAN

- **Base commit:** e75a797 (pre-PR #287)
- **develop at dispatch:** 97c66b0 was the server-side merge result; local was at e75a797 stale base.
- **Findings:**
  - F-F5P1-001 (HIGH): VP-027 Kani harness was tautological — harness inlined the EPB decode
    logic directly instead of calling `decode_epb_body`. Discriminant-twin proof was vacuous
    (proven trivially true by re-derivation, not by function contract). Violated VP-027's
    non-vacuity obligation.
  - F-F5P1-002 (MED): `read_magic` stub doc-comment used present-tense "returns" voice for
    stale RED-Gate descriptions. Multiple story-126/proptest/kani doc sites still red-tense.
  - F-F5P1-003 (MED): `format_zero_packet_notice` re-reads `self.packets_emitted` after
    the TOCTOU window — doc commentary described a potential race in the read sequence
    (informational; test coverage confirmed correct ordering).
  - O-1 (LOW): Comment precision nit on btl=16 arm characterization.
  - O-2 (LOW): `decode_epb_body_discriminant` twin equivalence not automated — divergence
    detectable only by re-running `cargo kani`.
- **Resolution:** PR #287 merged → develop=97c66b0.
  - `decode_epb_body` extracted as standalone function; VP-027 harness rewritten to call it.
  - Real discriminant-twin proof: 687 Kani checks, 0 failed; non-vacuity confirmed.
  - `is_pcapng` discriminant added for content-detection path.
  - Doc-tense fixes applied to shipped/GREEN test sites.
  - Spec bumps: BC-2.01.013 v1.10, VP-INDEX v2.9 (VP-027 draft→active), ADR-009 rev 12
    (Decisions 25/26).
- **Decision recorded:** D-188.

---

### Pass 2a — METHODOLOGY HALT (not a content pass)

- **develop at dispatch:** e75a797 (stale — local develop not fast-forwarded after server-side
  gh merge of PR #287).
- **Finding F-F5P2-001 (process-gap):** Adversary reviewed stale tree; entire review void.
- **Resolution:** `git pull --ff-only` to advance local develop to 97c66b0. Re-dispatched.
- **Process codification:** DF-DEVELOP-FRESHNESS-001 triggered; fast-forward is now mandatory
  after any server-side `gh pr merge` before same-session adversary dispatch. See PG-F5-FRESHNESS-001.

---

### Pass 2 (fresh) — NOT CLEAN

- **Base commit:** 97c66b0 (post-PR #287)
- **Findings:** 3 MED doc-tense findings — stale RED-Gate/todo prose on shipped-GREEN tests
  in story126, proptest, and kani modules. Header-level fixes from Pass-1 resolution missed
  per-test `/// RED:` lines and sibling-file sites.
- **Resolution:** PR #288 merged → develop=292c5e4.

---

### Pass 3 — NOT CLEAN

- **Base commit:** 5eaf587 (post-PR #288 local advance)
- **Findings:**
  - F-F5P3-001 (MED, DF-SIBLING-SWEEP-001): Doc-tense sweep was incomplete. Siblings
    story124/125/127/128 and 13 proptest harness sites still carried stale GREEN-phase doc
    annotations from the RED-gate era. A header-only sweep missed per-body sites.
- **Resolution:** PR #289 merged → develop=5eaf587 (complete sibling sweep across all
  story modules + 13 proptest sites).

---

### Pass 4 — NOT CLEAN

- **Base commit:** 2dd5209 (post-PR #289 local advance)
- **Findings:**
  - F-F5P4-001 (MED): 16 present-tense-false RED doc-comments inside story126 per-test
    `/// RED:` lines — these were in individual test bodies and were missed by the
    header-level sweep in passes 2 and 3.
- **Resolution:** PR #290 merged → develop=2dd5209.

---

### Pass 5 — NOT CLEAN

- **Base commit:** 3fc0e67 (post-PR #290 local advance — same SHA as final)
- **Findings:**
  - F-F5P5-001 (MED): story126 doc-comment lines 968-976 stated "NRB/ISB/SJE/DSB currently
    fall to wildcard" — false statement; named arms for those block types exist in reader.rs
    lines 1228-1254 and were shipped as part of the STORY-126 implementation. Doc diverged
    from implementation.
  - F-F5P5-002 (LOW): `read_magic` single `read()` call is vulnerable to short-read on slow
    pipes/TTYs — a valid magic header may be delivered in multiple small reads, causing
    false-negative `is_pcapng()` detection.
- **Resolution:** PR #291 merged → develop=3fc0e67.
  - story126 doc corrected to match shipped named arms.
  - `read_magic` refactored to use `read_exact` with graceful EOF/error handling.
  - 2 guard tests added.

---

### Pass 6 — CLEAN (Clean window: 1/3)

- **Base commit:** 3fc0e67
- **Scope:** BC sweep — all 11 active BCs (BC-2.01.009 through .018, BC-2.12.011) verified
  against implementation.
- **Result:** CLEAN. 0 blocking findings. 1 informational BASE10_POWERS observation
  (BC-authorized; non-actionable).
- **Consecutive clean counter:** 1/3.

---

### Pass 7 — CLEAN (Clean window: 2/3)

- **Base commit:** 3fc0e67
- **Scope:** Security and correctness depth — all attack classes against EPB/SPB/SHB/IDB
  paths, integer overflow, CWE-835 forward-progress, endianness invariants, VP-027
  non-vacuity re-confirmed.
- **Result:** CLEAN. 0 blocking findings. VP-027 non-vacuous confirmed (687 checks).
- **Consecutive clean counter:** 2/3.

---

### Pass 8 — CLEAN (Clean window: 3/3) — CONVERGENCE ACHIEVED

- **Base commit:** 3fc0e67
- **Scope:** Spec-to-implementation traceability + test-quality depth — all ACs verified
  test-backed, non-tautological. BC-2.01.009..018 + BC-2.12.011 full traceability sweep.
- **Result:** CLEAN. 0 blocking findings.
  - O-1 (LOW, informational): BC-2.01.017 PC1 illustrative "e.g." context strings diverge
    from shipped strings — non-blocking; authoritative mandates in BC-2.01.012 match impl.
    Recorded as DRIFT-F5-O1-017STRINGS for spec-housekeeping.
  - O-2 (LOW, informational): `decode_epb_body_discriminant` twin equivalence not automated
    (carried from Pass-1 O-2 / SEC-001 — no regression between passes; same open item).
- **Consecutive clean counter:** 3/3 — BC-5.39.001 gate SATISFIED.

---

## F5-EXIT Execution Evidence (orchestrator-run on develop=3fc0e67)

```
cargo test --all-targets
  result: ALL GREEN, exit 0

cargo kani --harness vp027_epb_parse_safety
  result: VERIFICATION SUCCESSFUL
  0 of 687 checks failed
  elapsed: 6.25s
```

Closes Pass-8 O-2 (DF-ADVERSARY-TOOLCHAIN-PAIRING-001) — Kani toolchain confirmed
operational on converged tree.

---

## Process-Gap Findings (for follow-up codification)

### PG-F5-FRESHNESS-001 (from F-F5P2-001)

After a server-side `gh pr merge`, local develop must be fast-forwarded before any
same-session adversary dispatch. Mitigation applied this session (mandatory
`git pull --ff-only` added to F5 fix-PR flow). Follow-up: codify as permanent
pr-manager/post-merge workflow step (self-improvement epic).

### PG-F5-DOCTENSE-TOKENS-001 (from Pass-4 O-1 / F-F5P5-003)

The DF-GREEN-DOC-TENSE-SWEEP grep token set misses bare `RED:`, "doesn't exist yet",
"no <X> arm", "falls to `_`", "falls through to", "currently has NO", "wildcard",
"currently satisfied by". This gap caused approximately 4 extra passes. Target:
expand DF-GREEN-DOC-TENSE-SWEEP token list in policies.yaml (high priority; recommend
applying before next doc-tense-bearing cycle).

---

## Findings Not Blocking F6

All items below are OPEN but non-blocking. Carry forward to F6/F7 or maintenance:

| ID | Severity | Summary | Status |
|----|----------|---------|--------|
| SEC-001 (F5P1) | MED | No automated twin-equivalence test for decode_epb_body vs discriminant-twin | Backlog — DO NOT BLOCK F6 |
| SEC-002 (F5P1) | LOW | wrapping_sub auditor clarity in PC6b padding computation | Backlog — DO NOT BLOCK F6 |
| DRIFT-F5-O1-017STRINGS | LOW | BC-2.01.017 PC1 illustrative strings diverge from shipped strings | Spec-housekeeping, defer to F6/F7 or maintenance |

---

## Decision Record

- **D-188:** F5 Pass-1 resolved (PR #287 merged 97c66b0). VP-027 genuine Kani proof.
- **D-189:** F5 CONVERGED — 3 consecutive clean passes (6, 7, 8) on develop=3fc0e67.
  F5 phase_status = CONVERGED. PAUSED for human review before F6.
