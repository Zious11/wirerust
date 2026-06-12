# Fresh-Eyes Pre-Merge Review — PR #230

**PR:** fix(dnp3): F5 remediation — DIR-bit fix + unexpected-source detection + resync accounting + IcsImpact display
**Branch:** fix/dnp3-f5-unexpected-source (HEAD b5f22ea) → develop (ddfa576)
**Scope:** src/analyzer/dnp3.rs, src/mitre.rs, 3 test files. +1905 / -48, 5 files.

## Verdict: APPROVE

No blocking or major findings. The four fixes are correct, well-scoped, and well-tested. A small number of NITs only.

---

## What I verified (no rubber-stamp)

### 1. DIR-bit fix (0x10 → 0x80) — CORRECT
- Per IEEE 1815 DNP3 link-layer, the CONTROL DIR bit is bit 7 (0x80); bit 4 (0x10) is FCV/DFC. The fix is correct.
- Canonical master CONTROL=0xC4: `0xC4 & 0x80 = 0x80` → true. Verified.
- `git grep '& 0x10'` across src/analyzer/dnp3.rs: the only remaining `& 0x10` is at line 1204 (`app_ctrl & 0x10` = UNS bit of the APPLICATION control byte, IEEE 1815 §7.2.3) — a different byte/field, correctly left untouched. No code path still uses 0x10 as a DIR mask.
- Corrected test `test_BC_2_15_016_is_master_frame_dir_bit` asserts the right values: 0xC4/0xD4/0xFF → true; 0x00/0x04/0x44/0x10 → false. The previously-wrong 0xEF false-case (0xEF has bit7 set → IS master) was correctly replaced with 0x44, and a dedicated 0x10→false assertion was added. Non-vacuous and exact.

### 2. Regression risk to STORY-106..110 from the mask change — NONE
- `git grep` shows only TWO readers of `master_addrs_seen`/`is_master_frame` in src: the push gate (line 540) and the new unexpected-source detection (line 605). No other STORY-106..110 detection (burst, broadcast, unsolicited, block-timeout, malformed-anomaly) reads `master_addrs_seen` or `is_master_frame`.
- Behavior change is therefore confined: `master_addrs_seen` now actually populates from canonical 0xC4 master traffic (it was inert before). Since the only consumer is the new F-001 detection, there is no regression surface. STORY-107's `build_master_frame` uses 0xD4 (bit7 set under both masks) so existing tests stay green — confirmed by commit note and test diff.

### 3. detect_unexpected_source_split (F-F5-001) — CORRECT
- **Snapshot ordering:** `src_was_known` / `expected_set_established` captured at lines 535-536, strictly BEFORE the `master_addrs_seen.push()` at 544. First-ever master → `expected_set_established=false` → no false positive. Correct.
- **Fall-through:** the gate `if is_master_frame && expected_set_established && !src_was_known` calls the detection but does NOT early-return; pending_requests insertion and `detect_control_class_burst_split` still run. Verified `direct_operate_count` increments on the intruder frame (test B-1 asserts count==2). No false-negative on the volumetric path.
- **master_set excludes intruder:** `src` is pushed into `master_addrs_seen` before the Control arm runs, so the `.filter(|&&a| a != src)` is required and correct — finding shows `[0x0001]` not `[0x0001, 0x0099]`. Verified against the pinned summary test.
- **One-shot guard:** `flow.unexpected_source_emitted` is flow-lifetime, set after the single emission; second distinct intruder (B-7) and address rotation are suppressed. Correct anti-flood behavior.
- **Two-entry evidence:** `app_fc` is un-dropped (was `_app_fc`); evidence is `["FC=0x.. dest=.. src=..", "expected_masters=.."]`. Matches B-1's exact assertion.
- **is_non_dnp3 / cap-full cases:** B-8 (bailed flow no-op) and B-6 (master_addrs_seen at cap=64 still fires) both verified. No false-positive on redundant/known master (src_was_known guard); no false-negative under cap saturation.

### 4. F-F5-003 resync accounting (three arms) — CORRECT, no double/under-count
- **Resync arm (Change 1):** now unconditionally `parse_errors += 1; malformed_in_window += 1` before the byte-walk. Covers Path B (junk at clean boundary). No clear+return data loss — `carry.clear()` here is fresh-start, `is_non_dnp3` not set.
- **LENGTH-gate arm (Change 2):** after `drain(..1)`, performs inline `windows(2)` resync to next [0x05,0x64] (or clear). This means the next loop iteration begins at a valid sync head or empty carry, so the resync arm does NOT re-fire for the same event → no double-count (Path A). Verified by D-2 (parse_errors==1, not 2) and D-5 (3 fake-sync triplets → exactly 3, not 6).
- **Overflow arm (Change 3R):** inline resync replaces the old clear+return. Preserves a valid head frame instead of silently discarding it (closes F-B-002 evasion DoS). Verified by D-4: valid head frame consumed (frame_count>=1), and DISTINCT trailing junk after the frame is correctly counted as Event 2 (parse_errors==2) via structural separation — the `overflow_counted_this_call` flag was correctly removed (commit 76460fd) because it under-counted genuinely independent later sync-loss events.
- **Double-count prevention is structural** (path separation), not flag-based — the overflow/LENGTH-gate inline resync leaves carry at a sync head or empty, so the resync arm is only entered for genuinely new events. This is the right mechanism and matches the REV-2 directive.
- **carry bounded:** all three arms either drain ≥1 byte or operate within the 292 cap; VP-023 preserved. All-junk carry-cap tests (test_carry_buffer_cap_at_292) correctly updated to parse_errors==1 with carry cleared (no valid frame → inline resync clears → resync arm never entered).

### 5. F-F5-002 IcsImpact Display — CORRECT
- `IcsImpact => "Impact (ICS)"` is now distinct from `Impact => "Impact"`. One-line change, verified by C-1 (assert_ne) and C-2 (reporter renders 2 distinct `## ` headers). No other Display arm collides.

### 6. Code quality / test quality / hygiene
- No dead code: the removed `overflow_counted_this_call` flag is fully gone (no orphan refs). `app_fc` param now used. Stale STUB doc-comment removed.
- No copy-paste errors: the three inline-resync blocks are intentional near-duplicates of the same `windows(2)` pattern; main resync arm correctly uses `.skip(1)` (already past offset 0), while overflow/LENGTH-gate variants scan from offset 0 — both correct for their respective carry states.
- Tests are non-vacuous with exact assertions (pinned summary string, exact counter values, exact evidence vec). RED-gate rationale documented per test.
- Commits are conventional-format, scoped, with clear bodies; semantic PR title valid (`fix(dnp3): ...`).
- Description matches diff: 23 test functions in the new file (matches PR table); files-changed table accurate.

---

## Findings

### NIT-1 — Comment-vs-code micro-inconsistency in overflow inline-resync (src/analyzer/dnp3.rs ~349-362)
The overflow arm's inline resync scans `windows(2)` from index 0 (no `.skip(1)`), whereas the main resync arm uses `.skip(1)`. This is *correct* in both cases (overflow carry may legitimately have a valid frame at offset 0; the main resync arm is only reached when offset 0 is already known-non-sync). However the inline comment "Structurally identical to Change 2" slightly overstates the symmetry — Change 2 (LENGTH-gate) runs after a `drain(..1)` so its scan also effectively starts past the old head, while the overflow scan deliberately includes offset 0. Consider a one-line note that the offset-0 inclusion is intentional (preserves a head frame already at the carry front). No behavior impact.

### NIT-2 — Verbose block comments (src/analyzer/dnp3.rs ~322-362, ~414-440)
The REV-2 rationale comments are very long (20+ lines per arm). Accurate and useful for the non-obvious structural-separation invariant, but border on duplicating the adjudication doc. Optional: trim to the load-bearing invariant and cite the doc. No action required.

---

## Checklist
- [x] All 5 changed files reviewed
- [x] DIR-bit fix correct per IEEE 1815; no residual 0x10 DIR mask; corrected test asserts right values
- [x] Unexpected-source: snapshot ordering, fall-through, master_set exclusion, one-shot, two-entry evidence — all correct; no FP/FN path found
- [x] Resync: three arms, no double-count, no under-count, no clear+return data loss, carry bounded
- [x] IcsImpact distinct; reporter renders 2 sections
- [x] No STORY-106..110 regression from mask change (only 2 readers, both in scope)
- [x] Test quality: non-vacuous, exact assertions
- [x] Commit hygiene: conventional, scoped, clear

**APPROVE — merge-ready. Two NITs are optional cleanups; no blocking or major findings.**
