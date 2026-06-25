# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 9

## Verdict

**FAIL** — 0 CRITICAL, 2 HIGH, 0 MEDIUM, 0 LOW. Novelty: MEDIUM.

Pass-8 command_counts relocation HELD structurally (STORY-137 frame-walk single site confirmed everywhere); 2 new attribution/label defects (prose anchors that drifted, not logic).

## Finding Summary

| ID | Severity | Title | Disposition |
|----|----------|-------|-------------|
| F-P9-001 | HIGH | STORY-134 line 192 note misattributes command_counts owner to "STORY-138 frame-walk/process_pdu" — stale post-Pass-8 | REMEDIATED |
| F-P9-002 | HIGH | dependency-graph E-20 specific-notes block (~lines 1193-1196) has 5 wrong BC labels (.023/.024/.025/.013/.012; .012/.013 swapped; .024/.025 falsely "MAX_FINDINGS/session T0814 DoS" — a deferred/forbidden feature) | REMEDIATED |

## Finding Detail

### F-P9-001 (HIGH) — STORY-134 prose note misattributes command_counts owner

**Location:** STORY-134, line 192 note block

**Description:** A prose note at STORY-134 line 192 attributes the canonical `command_counts` increment site to "STORY-138 frame-walk/process_pdu". This was accurate prior to Pass-8 but is now stale: Pass-8 relocated `command_counts` to STORY-137 `on_data` frame-walk (BC-2.17.016 PC-0). The note contradicts BC-2.17.016 PC-0 and the Pass-8 remediation. STORY-134 was missed during Pass-8 propagation — the sweep updated STORY-137/138/130 but not STORY-134's prose note.

**Canonical site:** STORY-137 `on_data` frame-walk, immediately after `parse_enip_header` returns `Some`, before `is_valid_enip_frame` (BC-2.17.016 PC-0).

**Fix:** Update STORY-134 line 192 note to correctly attribute command_counts to STORY-137 on_data frame-walk per BC-2.17.016 PC-0.

**Status:** REMEDIATED.

---

### F-P9-002 (HIGH) — dependency-graph E-20 specific-notes block has 5 wrong BC labels

**Location:** dependency-graph.md, E-20 specific-notes block, approximately lines 1193–1196

**Description:** The E-20 specific-notes block contains 5 wrong BC label references:
- `.023` and `.024` falsely annotated as "MAX_FINDINGS/session T0814 DoS" — this is a deferred/forbidden feature, not a live BC; .024 and .025 are the actual write-burst and error-burst threshold BCs
- `.012` and `.013` labels are swapped relative to their BC titles
- `.025` annotated with a description belonging to a different BC

These labels contradict: (1) the BC H1 titles in the ss-17 BC files, and (2) the coverage map in the same dependency-graph file. The E-20 notes block drifted when BCs were renumbered/added during F2 addendum work and Pass-1 (BC-2.17.025 session-handshake, BC-2.17.026 error-burst CLI flag) but the specific-notes block was not re-verified.

**Fix:** Correct the 5 label references in the E-20 specific-notes block to match canonical BC titles: .012/.013 order restored, .023/.024/.025 descriptions corrected to match actual BC content (write-burst threshold, error-burst threshold, session-handshake tracking — not deferred DoS feature).

**Status:** REMEDIATED.

---

## Confirmed-Clean Items

The following areas were examined and found clean in Pass 9:

- `command_counts` single-site placement: STORY-137 `on_data` frame-walk is the sole increment site across STORY-137, STORY-138, and BC-2.17.016 PC-0 / BC-2.17.004 (confirmed; F-P9-001 was prose only, not logic)
- 26-BC coverage: all BC-2.17.001..026 assigned across STORY-130..138
- VP-032 harnesses: Sub-A/B/C/D + vp032_cip_service_request_partition confirmed owned and correct
- All 7 `enip_summary` increment sites confirmed with canonical locations
- `strict->` to windows carry-overflow ordering: check_t0814 before is_non_enip latch (Pass-5 fix held)
- Wave/dependency acyclic structure: STORY-130..138 chain verified
- All 13 holdout scenarios HS-110..122: present and git-tracked

## Severity Trajectory

| Pass | CRITICAL | HIGH | Notes |
|------|----------|------|-------|
| P1 | 4 | 6 | Root-cause: AC→BC fidelity gap |
| P2 | 1 | 3 | Parse/seam defects |
| P3 | 0 | 2 | Structural defects |
| P4 | 2 | 2 | VP orphan + pseudocode gap |
| P5 | 0 | 1 | Carry-overflow ordering |
| P6 | 0 | 1 | Dead counter (flows_analyzed) |
| P7 | 0 | 0 | **PASS** — first clean pass |
| P8 | 0 | 1 | command_counts structural contradiction |
| P9 | 0 | 2 | Attribution/label drift (prose only) |

**Convergence counter: 0/3** (reset — streak broken by P8 FAIL; P9 also FAIL).

## Process Gap Note

Pass-8 BC relocation propagated to STORY-137/138/130 but missed STORY-134's prose note + the dep-graph specific-notes block. Sibling-sweep must include ALL stories referencing the relocated value + dep-graph note blocks, not just the primary story owners. Any "X owned by Y" resolution claim must be verified with a corpus-wide grep for the old attribution string before the burst is closed.
