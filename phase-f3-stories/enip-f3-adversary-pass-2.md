# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 2

VERDICT: FAIL — 1 CRITICAL, 3 HIGH, 5 MEDIUM, 3 LOW. Novelty: HIGH. Pass-1 fixes HELD (STORY-134 one-shot, 135 T0858/T0816, 137 windowed T0814, 138 parse_errors/no-session-anomaly all confirmed clean). New defects in STORY-130 (foundational parse) + CIP seams.

- F2-01 (CRITICAL, STORY-130): two contradictory is_valid_enip_frame signatures (2-arg buffer-fit vs BC-2.17.003 1-arg command-only); VP-032 Sub-C harness proves wrong property + breaks STORY-137 call site. REMEDIATED.
- F2-02 (HIGH, STORY-130/132): EnipCommand/Unknown(u16) vs BC-2.17.004 EnipCommandClass/payloadless-Unknown; STORY-132 cross-ref wrong. REMEDIATED.
- F2-03 (HIGH, STORY-132): general_status offset 2+path*2+1 vs BC-2.17.008 byte-2 response offset; corrupts T0888 Pattern B. REMEDIATED.
- F2-04 (HIGH, STORY-136): re-added raw service&0x80 predicate BC-2.17.007 Inv1 forbids (partial-fix regression vs STORY-135). REMEDIATED.
- F2-05 (MED): error_window_start → error_window_start_ts (STORY-134/138). REMEDIATED.
- F2-06 (MED): STORY-134 AC-134-005 trace Inv3 → Precondition 2. REMEDIATED.
- F2-07 (MED): STORY-134 summary {N} → {total_errors}. REMEDIATED.
- F2-08 (MED): STORY-134 HashMap<u8,u32> → u64. REMEDIATED.
- F2-09 (MED): STORY-135 T0836 verdict Likely/Medium verify. REMEDIATED.
- F2-10 (LOW): dependency-graph changelog scratch-text cleanup. REMEDIATED.
- F2-11 (LOW, [process-gap]): BC-2.17.010 Description says "per-occurrence" contradicting its own one-shot postconditions — story correctly resolved to one-shot. DEFERRED: PO to fix BC-2.17.010 Description (+ recompute STORY-134 input-hash) before F3 gate.
- F2-12 (LOW): STORY-137 test-name ordering (set-equality holds; non-blocking). DEFERRED.
