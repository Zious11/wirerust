# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 4
VERDICT: FAIL — 2 CRITICAL, 2 HIGH, 2 MEDIUM. Novelty: MEDIUM-HIGH. Prior-pass fixes ALL HELD (single-arg is_valid_enip_frame, 0x00B2 gate, strict-> bursts, write_count/error_count aggregates, LargeForwardOpen, parse_errors, MAX_FINDINGS, VP-007). New defects found by going DEEPER — cross-checking embedded pseudocode + VP-frontmatter against the (correct) prose ACs.
- F4-01 (CRITICAL): VP-032 Sub-D Kani harness ORPHANED — STORY-130 defers to STORY-132 but STORY-132 had verification_properties:[], no Sub-D harness/AC/task (would fail F6). REMEDIATED (STORY-132 owns VP-032 Sub-D + AC-132-007 + harnesses; input-hash recomputed).
- F4-02 (CRITICAL): STORY-137 frame-walk pseudocode missing is_valid_enip_frame rejection path (would fail must-pass HS-117 Case A; prose AC was correct). REMEDIATED (validity-gate inserted into pseudocode).
- F4-03 (HIGH): STORY-138 stale write_window_start → write_window_start_ts. REMEDIATED.
- F4-04 (HIGH, process-gap): dependency-graph E-20 justifications cite non-existent field names + inverted STORY-132↔137 relationship. REMEDIATED.
- F4-05 (MED): STORY-134 AC ordering 006-before-005. REMEDIATED.
- F4-06 (MED): STORY-138 phantom cip_service_counts field. REMEDIATED.
Severity trajectory: P1 4C/6H → P2 1C/3H → P3 0C/2H → P4 2C/2H (deeper pseudocode/VP-frontmatter audit). Confirmed-held: all prior fixes intact.
