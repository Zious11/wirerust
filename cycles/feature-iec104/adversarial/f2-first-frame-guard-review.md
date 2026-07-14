---
document_type: adversarial-review-record
scope: feature-iec104 F2 first-frame-guard enhancement
producer: state-manager
date: 2026-07-14
---

# F2 First-Frame-Guard Enhancement — Adversarial Review & Consistency Check

Human-mandated at F2 gate (D-438 follow-on, D-439). Enhancement applied to SS-19
shard v1.5→v1.6 and BC-2.19.023/024 before gate closure.

## Adversarial Review Verdict: CLEAN

Two LOW prose-only findings; both applied before BC-2.19.024 v1.3 was committed:

1. **LOW-1** — BC-2.19.024 v1.2 Description section did not mention the
   `None`→`Some` first-frame establishment semantics explicitly. Fixed in v1.3:
   Description reworded to state that `last_ns_c2s`/`last_ns_s2c` begin as `None`
   and transition to `Some` on first received I-frame.

2. **LOW-2** — BC-2.19.024 v1.2 lacked canonical example vectors for the
   `None` initial state and the `None`→`Some(0)` first-frame establishment path.
   Fixed in v1.3: canonical vectors section updated with mid-capture chain
   `5000 → 5001 → 5020` and wrap-around `32767 → 1` examples; both hand-verified
   correct (modulo-32768 arithmetic).

Enhancement adjudged **SOUND**: Option<u16> type change is a narrowly-scoped
precision improvement that eliminates a speculative initial-N(S) assumption.
No existing BC invariants violated. F2 remains converged with this enhancement.

## Pre-Existing Observation (carried to F3/F4)

**RETRANSMIT-NS-FALSEPOS-001** — Backwards, retransmitted, or reordered I-frames
carry an N(S) value lower than the previously-seen N(S). When the first-frame guard
fires (`last_ns` = `None → Some(received_ns)`), a subsequent retransmit with a
lower N(S) computes a large 15-bit gap value (e.g., `(32768 - 5001 + 5000) mod 32768 =
32767`). This large gap may trigger a T1692.001 false-positive with elevated
confidence. Pre-existing in the converged gap arithmetic — not introduced by the
first-frame guard. Implementer (F3) and holdout evaluator (F4) should consider
retransmit tolerance (e.g., ignore N(S) regressions below a threshold, or suppress
gap-count increment when `|new_ns - last_ns| mod 32768 > MAX_FORWARD_WINDOW`).

## Consistency Check: PASS (7/7)

All seven consistency checks passed:

| # | Check | Result |
|---|-------|--------|
| 1 | SS-19 v1.6 field types match BC-2.19.023 v1.2 preconditions | PASS |
| 2 | SS-19 v1.6 field types match BC-2.19.024 v1.3 preconditions | PASS |
| 3 | BC-2.19.024 v1.3 canonical vectors internally consistent (mod-32768 arithmetic) | PASS |
| 4 | Mid-capture chain `5000→5001→5020` gap=19 correct | PASS |
| 5 | Wrap-around `32767→1` gap=2 correct (`(1 - 32767 + 32768) mod 32768 = 2`) | PASS |
| 6 | BC-2.19.023 v1.2 None→Some transition description consistent with v1.3 | PASS |
| 7 | VP-047 fuzz-target text consistent with Option<u16> type | PASS |

## Spec Versions After Enhancement

| Artifact | Before | After |
|----------|--------|-------|
| SS-19 shard | v1.5 | v1.6 |
| BC-2.19.023 | v1.1 | v1.2 |
| BC-2.19.024 | v1.1 | v1.3 |
| BC-INDEX | v2.28 | v2.28 (no index-count change — type-only amendment) |
| All other indices | unchanged | unchanged |

Input-hashes for all 27 BC-2.19.* files recomputed with canonical Python tool
(`bin/compute-input-hash --write`) after the enhancement. New hash: `a153144`
(was `f5a97d3`). Scan result: MATCH=27 STALE=0 for SS-19 shard.
