# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories STORY-130..138), Pass 1
VERDICT: FAIL — 4 CRITICAL, 6 HIGH, 5 MEDIUM, 3 LOW-obs. Novelty: HIGH. Root cause: ACs written from assumption, not transcribed from converged BC postconditions.
CRITICAL:
- F-01 (STORY-134): T0846 missing per-flow one-shot guard (list_identity_emitted); would fail must-pass HS-114 Case B. REMEDIATED.
- F-02 (STORY-138): entire BC table mis-anchored (BC-2.17.025/017/021/024 all wrong-labeled). REMEDIATED.
- F-03 (STORY-138): AC-138-002 invents out-of-scope session-anomaly T0814 (BC-2.17.025 defers session-handle validation to v0.12.0). DELETED.
- F-04 (STORY-137): T0814 model wrong — lifetime vs windowed; is_non_enip latched on T0814 (should be carry-overflow only); BREAK vs byte-walk resync/frame-skip. Would fail HS-117. REMEDIATED.
HIGH: F-05 (STORY-137 T0814 fields Anomaly/Possible/Low), F-06 (STORY-136 ForwardOpen fields Anomaly/Possible/Low), F-07 (STORY-136 LargeForwardOpen 0x5B omitted), F-08 (STORY-135 T0836 category=Execution), F-09 (STORY-135 test-name sync), F-10 (STORY-130 BC table mislabel BC-2.17.004 vs classify_cip_service; re-anchor).
MEDIUM: F-11 (STORY-135 Reset predicate→classify_cip_service==Reset), F-12 (STORY-130 test-name sync), F-13 (STORY-134 summary+evidence), F-14 (STORY-134 error-window fields/byte-2 offset/len>=4), F-15 (STORY-138 EnipSummary schema→BC-2.17.021 canonical parse_errors key).
LOW-obs: O-1 (STORY-132 fuzz obligation non-testable AC — process-gap candidate), O-2/O-3 (wave/dependency acyclic + correct — no defect).
Holdout coverage: holdouts OK; gaps were in stories (HS-114 vs F-01, HS-117 vs F-04). Wave/dependency graph CORRECT.
