---
pass: wave-gate
wave: 82
story: STORY-173
reviewer: vsdd-factory:code-reviewer + vsdd-factory:security-reviewer
date: 2026-07-16
diff_range: d64e5fe..084ff93
pr: 408
verdict: PASS
---

# Wave-82 Gate Code Review

## Scope

PR #408 — STORY-173 squash merge (feat: STORY-173 IEC-104 dispatcher integration + T0881 catalog
+ --iec104 flag + findings cap (wave-82)). Single-story wave. Files reviewed:

- `src/analyzer/iec104.rs` — dispatcher integration, detect_iec104_threats wiring, MAX_IEC104_FINDINGS cap
- `src/dispatcher.rs` — DispatchTarget::Iec104 arm, --iec104 CLI flag routing
- `src/protocols.rs` — T0881 catalog entry, SUPPORTED_PORTS IEC-104 entry
- `src/main.rs` — --iec104 flag addition
- `tests/iec104_analyzer_tests.rs` — new dispatcher-integration test suite (mod story_173)
- `CHANGELOG.md` — [Unreleased] entry per changelog-gate obligation

Review axes:
1. Correctness: detect_iec104_threats wiring, findings-cap enforcement (IEC104-FINDINGS-CAP-001),
   T0881 six-part atomic (BC-2.19.028 anchor), dispatch routing correctness.
2. Security: CWE-400/770 cap enforcement; is_valid_iec104_frame doc accuracy vs. wiring status.
3. Counters: flows_analyzed and packets_analyzed semantics vs. sibling analyzer patterns.
4. Test quality: discriminating assertions, demo evidence accuracy.
5. Maintainability: doc comments, process-gap carry-forwards.

## Context

Per-story adversarial ran 17 total passes:
- Initial convergence: 14 passes (streak P12/P13/P14 at 7b2a73e, D-457).
- Pre-merge LOW-fix burst: human chose "fix all 3 pre-merge" — LOW#1 (flows_analyzed),
  LOW#2 (packets_analyzed), SEC-001 (is_valid doc). Fix commits: 0bfc977 / 5325cf2 / 3ec6ac1.
- Re-convergence: 3 fresh passes (A/B/C CLEAN on 3ec6ac1).

Total: 17 adversarial passes. Production code FROZEN/CLEAN since P2. Post-P2 tail was
doc-accuracy and test-cosmetic reviewer variance.

## Verdict

**PASS.** All BLOCKING / HIGH / MEDIUM / LOW findings remediated before merge. Advisory
INFO findings accepted. Two test-tense advisories (A-12-01 and A-173-B-01) accepted
non-blocking and deferred to STORY-174 doc-currency grep-guard.

---

## Findings

### LOW-1: flows_analyzed counter semantics did not match sibling analyzers

- **Severity:** LOW (pre-merge; FIXED in commit 0bfc977)
- **Category:** correctness / API contract
- **Source:** pr-reviewer Pass A (pre-merge burst)
- **Finding ID:** LOW#1

**Description:** `flows_analyzed` in `Iec104Stats` was incremented on every new flow
arrival (test-packet counter), rather than counting flows that contained at least one
valid APDU — the semantic used by ENIP and DNP3 sibling analyzers. JSON consumers
expecting the mirrored behavior would receive inflated counts.

**Disposition:** FIXED (commit 0bfc977). Semantics aligned to sibling pattern: increment
only when a valid IEC-104 APDU is observed in the flow. 2604/0 tests post-fix.

---

### LOW-2: packets_analyzed counter semantics did not match sibling analyzers

- **Severity:** LOW (pre-merge; FIXED in commit 5325cf2)
- **Category:** correctness / API contract
- **Source:** pr-reviewer Pass A (pre-merge burst)
- **Finding ID:** LOW#2

**Description:** `packets_analyzed` was incremented on all data callbacks regardless of
whether a valid I-format APDU was decoded. DNP3 and ENIP mirror a valid-frame counter.
The mismatch would surface incorrect statistics in reports.

**Disposition:** FIXED (commit 5325cf2). Counter now incremented only on confirmed
valid-APDU frames, mirroring DNP3 (`packets_analyzed` = valid-APDU frames) per
BC-2.19.028 anchor.

---

### SEC-001: is_valid_iec104_frame doc comment overstated the function's gate role

- **Severity:** LOW (security-adjacent; FIXED in commit 3ec6ac1; also captured as A-173-A-01)
- **Category:** documentation accuracy / defensive framing
- **Source:** security reviewer pass + adversarial Pass A (A-173-A-01)
- **CWE:** N/A — doc-only, function not wired as gate
- **Finding ID:** SEC-001 / A-173-A-01

**Description:** `is_valid_iec104_frame` was documented in a way that implied it acts
as a security validity gate. In fact the function is a standalone predicate used for
quick-triage; it is NOT wired into the dispatch path. Misleading framing risked a
future implementer wiring it as a gate and inadvertently re-opening the F-172-001
evasion channel (the walk-first residual-bound design in BC-2.19.025 v1.3 is what
provides the security property).

BC-2.19.006 v1.1 updated to v1.2: reframed as "standalone lightweight predicate" with
an explicit note that it is NOT a security gate. BC-INDEX updated v2.32→v2.33.

**Disposition:** FIXED (commit 3ec6ac1). Doc comment now accurately describes scope.
BC-2.19.006 v1.2 annotation prevents future wiring confusion.

---

### INFO-3: Demo evidence is Markdown-rendered rather than terminal-captured

- **Severity:** INFO / ADVISORY
- **Category:** demo evidence format
- **Source:** pr-reviewer review cycle
- **Finding ID:** INFO#3

**Description:** Several STORY-173 demo-evidence files present output as fenced
Markdown code blocks rather than raw terminal captures or VHS recordings. This is
consistent with how STORY-172 demo evidence was produced (same reviewer cycle accepted
the same format on PR #406).

**Disposition:** ACCEPTED (advisory). Markdown-rendered demo evidence is acceptable
when terminal artifacts are not available. No change required. Observation noted for
demo-recorder process improvement.

---

### A-12-01: Test module header comment retains present-tense "stubs" language

- **Severity:** NIT / ADVISORY (accepted non-blocking)
- **Category:** doc-tense drift (PG-REDGREEN-COMMENT-CLEANUP)
- **Source:** adversarial pass A-12 (wave-82 re-convergence pass A)
- **Finding ID:** A-12-01

**Description:** One test module block in `tests/iec104_analyzer_tests.rs` (mod story_173)
contains a comment header that uses present-tense "Red Gate" phrasing from the stub
era. This is cosmetic; the tests are GREEN and the stubs are implemented. Part of the
broader PG-REDGREEN-COMMENT-CLEANUP pattern (5th confirmed occurrence).

**Disposition:** ACCEPTED-DEFERRED-STORY-174. STORY-174 formal hardening wave will
add a CI grep-guard that enforces removal of stale Red-Gate phrases, which resolves
this class of finding process-wide.

---

### A-173-B-01: Adversarial Pass B advisory — test assertion tense in doc comment

- **Severity:** NIT / ADVISORY (accepted non-blocking)
- **Category:** doc-tense drift
- **Source:** adversarial re-convergence Pass B
- **Finding ID:** A-173-B-01

**Description:** A doc comment in a story_173 test body used future-tense "should"
phrasing ("should emit...") rather than asserting what the test actually verifies.
Advisory only; no correctness impact.

**Disposition:** ACCEPTED-DEFERRED-STORY-174. Same vehicle as A-12-01: STORY-174
doc-currency grep-guard covers this class.

---

## Adversarial Finding Trajectory Summary (17 passes)

Initial convergence (P1..P14, D-457, HEAD 7b2a73e):

| Pass | Clean | Finding summary |
|------|-------|-----------------|
| P1 | No | F-173-001 HIGH (T0881 tactic string "impact" → MitreTactic::IcsInhibitResponseFunction; compilation blocker from D-456 SR-173-01) + 3 doc findings |
| P2 | No | 1 MEDIUM (dispatch wiring assertion gap) + 1 NIT |
| P3 | No | doc-tense NITs |
| P4 | No | doc-tense NITs |
| P5 | No | doc-tense NITs |
| P6 | CLEAN | streak #1 |
| P7 | No | 1 NIT stale protocols.rs cardinality |
| P8 | No | 4 NITs stale mitre.rs seeded-count assertions |
| P9 | CLEAN | streak #1 |
| P10 | CLEAN | streak #2 |
| P11 | No | 1 NIT non-discriminating EMITTED_IDS test (F-173-1101) |
| P12 | CLEAN | streak #1 |
| P13 | CLEAN | streak #2 |
| P14 | CLEAN | streak #3 — initial CONVERGED (D-457) |

Pre-merge LOW-fix burst (human decision: "fix all 3 pre-merge"):
- LOW#1 flows_analyzed: fixed in 0bfc977
- LOW#2 packets_analyzed: fixed in 5325cf2
- SEC-001 / A-173-A-01 is_valid doc: fixed in 3ec6ac1

Re-convergence passes (A/B/C, HEAD 3ec6ac1):

| Pass | Clean | Finding summary |
|------|-------|-----------------|
| A | CLEAN (advisory A-173-A-01 accepted non-blocking) | streak #1 |
| B | CLEAN (advisory A-173-B-01 accepted non-blocking) | streak #2 |
| C | CLEAN | streak #3 — CONVERGED (D-458) |

Production code FROZEN/CLEAN since P2. Post-P2 adversarial variance was doc-accuracy
and test-cosmetic reviewer calibration drift.

---

## Finding Disposition Table

*(Per AC-158-006 / PG-W71-CODEREVIEW-ARTIFACT — all dispositions human-authorized via
orchestrator-verified D-458 facts.)*

| ID | Severity | File | Description | Disposition |
|----|----------|------|-------------|-------------|
| LOW#1 | LOW | `src/analyzer/iec104.rs` | flows_analyzed semantics mismatch vs. sibling analyzers | FIXED (commit 0bfc977, 2604/0 tests) |
| LOW#2 | LOW | `src/analyzer/iec104.rs` | packets_analyzed semantics mismatch vs. sibling analyzers | FIXED (commit 5325cf2) |
| SEC-001 / A-173-A-01 | LOW | `src/analyzer/iec104.rs` | is_valid_iec104_frame doc overstated gate role; BC-2.19.006 v1.1→v1.2 | FIXED (commit 3ec6ac1; BC-INDEX v2.32→v2.33) |
| INFO#3 | INFO | `docs/demo-evidence/STORY-173/` | Demo evidence Markdown-rendered vs. terminal-captured | ACCEPTED (advisory; consistent with wave-81 precedent) |
| A-12-01 | NIT | `tests/iec104_analyzer_tests.rs` | Test module header retains stub-era Red-Gate phrasing | ACCEPTED-DEFERRED-STORY-174 (PG-REDGREEN-COMMENT-CLEANUP; STORY-174 grep-guard) |
| A-173-B-01 | NIT | `tests/iec104_analyzer_tests.rs` | Test doc comment future-tense "should" phrasing | ACCEPTED-DEFERRED-STORY-174 (same vehicle as A-12-01) |

---

## Security Review Summary (IEC104-FINDINGS-CAP-001 CLOSED)

IEC104-FINDINGS-CAP-001 (CWE-400/770, unbounded findings Vec deferred from sec-review-170
M-001 / PR #404) was the primary security obligation for STORY-173.

| Check | Result |
|-------|--------|
| MAX_IEC104_FINDINGS constant defined | PASS (value: 10_000, mirroring DNP3 BC-2.15.022 / ENIP BC-2.17.022) |
| dropped_findings counter present | PASS (surfaced in summary output) |
| Cap enforced before detect_iec104_threats call | PASS |
| is_valid_iec104_frame NOT wired as gate | VERIFIED (doc clarified; walk-first residual-bound per BC-2.19.025 v1.3 is the security property) |
| F-172-001 evasion channel remains closed | VERIFIED (walk-first design unchanged; is_valid doc fix non-breaking) |

**IEC104-FINDINGS-CAP-001 RESOLVED.** No open CWE-400/770 exposure in the IEC-104
dispatcher path at this wave gate.

---

## Summary

No BLOCKING / MAJOR / MINOR findings unresolved at gate close. Three LOW findings
(LOW#1, LOW#2, SEC-001) fixed pre-merge. One INFO accepted. Two NIT advisories
deferred to STORY-174 doc-currency grep-guard. IEC104-FINDINGS-CAP-001 security
obligation met. CI 13/13 green. 2604/0 tests on final HEAD 3ec6ac1.

**Gate status: CLOSED — PASS (D-458, 2026-07-16)**
