---
document_type: per-story-convergence-report
story_id: STORY-115
step: "4.5"
verdict: CONVERGED
policy: BC-5.39.001
consecutive_clean_passes: 3
worktree_branch: worktree-issue-9-story-115-arp-d3-storm
develop_base: 7c0f453
final_head: dcdbf95
timestamp: 2026-06-15T00:00:00Z
producer: state-manager
---

# Per-Story Adversarial Convergence Report — STORY-115, Step 4.5

## Verdict

**CONVERGED** (BC-5.39.001 — 3 consecutive clean fresh-context passes on frozen diff at
HEAD dcdbf95)

## Worktree

- Branch: `worktree-issue-9-story-115-arp-d3-storm`
- Base develop: `7c0f453`
- Final HEAD: `dcdbf95`

## Clean Passes (all on HEAD dcdbf95)

| Pass | Adversary Dispatch SHA | Verdict | Zero-Findings |
|------|----------------------|---------|---------------|
| PASS 1 | a6f45a32 | CLEAN | Yes — ZERO FINDINGS |
| PASS 2 | acbe2f5b | CLEAN | Yes — ZERO FINDINGS |
| PASS 3 | a58db908 | CLEAN | Yes — ZERO FINDINGS |

All 3 consecutive passes returned ZERO FINDINGS under full BC-set completeness sweep.

## BC-Set Completeness Sweep (all 3 passes)

All BCs in scope verified — every clause has an implementation path. All 14 axes PASS
across all 3 passes.

| BC | Description | Key Clauses Verified | Axes Result |
|----|-------------|---------------------|-------------|
| BC-2.16.008 | D3 storm detection — 3-step: window-expiry/init → in-window increment → rate eval | count/max(1,elapsed) (no +1 bias); one-shot guard; window reset; GARP coverage; LRU-guard; Inv6 | PASS |
| BC-2.16.013 | `--arp-storm-rate` CLI flag wired to `ArpAnalyzer::new(spoof_threshold, storm_rate)` | default value; CLI→new() plumbing | PASS |
| BC-2.16.010 | `storm_findings` summary VALUE wiring (cross-story extension) | summary map emits storm_findings count | PASS |

Additional axes verified: GARP/D3 interaction (detect_storm called before GARP early-return);
MAX_STORM_COUNTERS=4096 LRU eviction discipline; saturating_sub arithmetic safety; MEDIUM/Anomaly
finding with empty MITRE + evidence source_mac/frame_count/window_secs/rate_pps.

## Execution Evidence

- `cargo test --all-targets`: **1571 passed / 0 failed**
- `cargo fmt --check`: clean
- `cargo clippy --all-targets -- -D warnings`: clean
- rustfmt version: 1.9.0-stable (CI-matched)
- `src/findings.rs` == develop baseline (zero diff — no reporter/catalog change)
- `src/mitre.rs` == develop baseline (zero diff — no reporter/catalog change; T0814 withheld
  per DF-VALIDATION-001; storm detection emits empty `mitre_techniques: []`)
- T0814 not emitted (DF-VALIDATION-001: requires research-agent validation before filing)
- Develop-bound diff scoped to exactly 4 files:
  `src/analyzer/arp.rs`, `src/cli.rs`, `src/main.rs`,
  `tests/bc_2_16_story115_arp_tests.rs`

## Prior-Cycle History (findings raised and ALL resolved)

### C1 / F1-GARP (HIGH) — GARP storm detection bypass

**Finding:** D3 storm detection was UNREACHABLE for GARP frames. `detect_storm` was called
AFTER the GARP early-return branch in `analyze_packet`, meaning a GARP-flood DoS could never
trigger a D3 storm finding — the entire attack class was silently invisible.

**Discovery:** Caught by passes 2 and 3 via BC-completeness + GARP/D3 interaction analysis.
Pass 1 missed it (reviewed detection path in isolation without tracing the GARP branch ordering).
This is strong positive evidence for the 3-pass requirement: a single pass would have shipped
the gap.

**Fix:** Hoisted `detect_storm` to a single call site BEFORE the GARP branch (commit 38933c5).
RED regression test `test_storm_detected_for_garp_flood` added — asserts that a GARP-flood
sequence triggers a storm finding.

**Note on pass-1 miss:** Pass 1 reviewed the detection path in isolation and found it correct;
it did not trace the intra-function call ordering relative to the GARP early-return. The
GARP/D3 interaction axis was not enumerated as an explicit check in the pass-1 dispatch.
This recurrence is addressed by extending the per-story adversary BC-completeness sweep to
explicitly include GARP/D3 interaction as an axis (noted in lessons.md).

### F-1 Storm-LRU spurious eviction (MEDIUM / real implementation bug)

**Finding:** `insert_storm_counter_lru` lacked the `contains_key` guard that `insert_binding_lru`
carries. When an already-present MAC's counter was being re-initialized at capacity, the LRU
eviction dropped an INNOCENT different MAC rather than preserving the existing entry. A storm
counter for a MAC that was already tracked would silently evict a bystander.

**Fix:** Added the `contains_key` guard (commit 8d5be0c). RED regression test
`test_storm_lru_no_spurious_eviction_on_existing_mac_reinit` added — asserts no bystander
MAC is evicted when an already-tracked MAC's counter is re-initialized.

### Confidence serde over-reach (caught pre-Step-4.5)

**Finding:** The implementer had added `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` to the
shared `Confidence` enum to satisfy AC-015's casing requirement. This was a tool-wide JSON
contract change — every Finding's `confidence` field would have changed case in the JSON output,
affecting all analyzers and holdout scenarios.

**Fix:** Reverted (commit 17ce551). AC-015 re-asserted at the confidence LEVEL
case-insensitively. JSON envelope-casing alignment deferred to follow-up FU-JSON-CASING
(governed cross-cutting change requiring BC-2.11.001 + BC-2.09.004 scope + ADR note).

### mod story_115 namespace (pre-Step-4.5)

`mod story_115` un-nested to be a sibling of `mod story_114` (commit 43cd23f) per
DF-TEST-NAMESPACE-001. Confirmed before adversary dispatch.

### DF-GREEN-DOC-TENSE recurrences (doc-tense fix commits)

After the GARP-flood fix (C1/F1-GARP) and after the F-1 LRU fix, newly-added regression
tests were authored with RED-era present-tense prose ("this test currently fails", "the code
lacks the guard", "MUST FAIL until X is fixed"). These comments became false immediately after
the Green fix landed, but the implementer's GREEN-step doc sweep repeatedly missed the
just-added test's own prose.

Two successive rounds of doc fixes were required:
- After GARP-flood fix: test `test_storm_detected_for_garp_flood` doc-comment fixed (F-2 in
  convergence journey).
- After LRU fix: test `test_storm_lru_no_spurious_eviction_on_existing_mac_reinit` doc-comment
  fixed (final commit dcdbf95).

This is the 7th+ recurrence of DF-GREEN-DOC-TENSE across the ARP feature cycle, and the first
time it was triggered specifically by a newly-added regression test's own doc-comment (the prior
occurrences were in pre-existing module headers and stale count references). See lessons.md
PG-ARP-F4-REDTEST-DOC-TENSE (policy extension proposed below).

## Deliverables

| Deliverable | BCs Satisfied |
|-------------|--------------|
| D3 ARP storm detection — 3-step algorithm (window-expiry/init, in-window increment, rate eval `count/max(1,elapsed)` no +1 bias); one-shot guard; window reset; MAX_STORM_COUNTERS=4096 LRU with `contains_key` guard; `saturating_sub` arithmetic safety; MEDIUM/Anomaly finding with `mitre_techniques: []` and evidence `source_mac/frame_count/window_secs/rate_pps` | BC-2.16.008 |
| GARP coverage: `detect_storm` called BEFORE GARP early-return (hoisted call site, commit 38933c5) | BC-2.16.008 Inv/GARP interaction |
| `--arp-storm-rate` CLI flag wired to `ArpAnalyzer::new(spoof_threshold, storm_rate)` | BC-2.16.013 |
| `storm_findings` summary VALUE wiring in output map (cross-story extension from BC-2.16.010) | BC-2.16.010 |

## Accepted Deferrals (Non-Blocking)

| Item | Rationale |
|------|-----------|
| T0814 not emitted on storm findings | DF-VALIDATION-001: MITRE technique assignment requires research-agent validation before filing; storm detection emits `mitre_techniques: []` per current spec scope |
| `--arp-storm-rate=0` F3 latitude (EC-006) | Non-panicking behavior; no BC or Invariant requires special-casing zero rate |
| Commit-history shape (stub/test collapsed into impl commit) | Red Gate was performed and is verifiable via this report + test prose and the two regression-test commits (38933c5, 8d5be0c) |

## Watch-Items Resolved

**BC-2.16.010-PC2-SIGNATURE watch-item from STORY-114-step45.md:**
STORY-115 correctly implements `ArpAnalyzer::new(spoof_threshold, storm_rate)` with `storm_rate`
as the second parameter. `storm_findings` VALUE wiring is confirmed. Watch-item CLOSED.

## Follow-Up Items Registered (do NOT action now)

**FU-JSON-CASING:** Align `Confidence`/`Verdict`/`ThreatCategory` serde to uppercase (matching
Display and BC-2.09.004) as a governed cross-cutting JSON-contract change. Requires updating
BC-2.11.001 + BC-2.09.004 scope + an ADR note. Currently the JSON envelope is internally
consistent PascalCase; this is an improvement, not a defect.

**FU-BC-2.10.007-MARKER:** Verify/update BC-2.10.007's PLANNED marker for `technique_tactic`
now that STORY-114 made T0830/T1557.002 resolvable via `technique_tactic`.

**FU-REPO-WIDE-DOC-DEBT:** Standalone docs chore PR for repo-wide stale RED-gate prose (modbus/
dnp3/reassembly/csv test files + STORY-113/114 baseline mod tests/`mod story_114` "RED: stub not
wired" artifacts). Carried from STORY-114; do NOT bundle into STORY-115 PR or any feature story.

**FU-STORM-NEW-ATTR:** `src/analyzer/arp.rs` ~line 272 doc attributes the `storm_rate` param to
STORY-114; `storm_rate` is STORY-115's deliverable. Minor doc cleanup; fold into a standalone
chore.

## Final Status

STORY-115 Step-4.5 adversarial convergence: **CONVERGED**.
This is the FINAL E-16 story (wave 44). ARP feature F4 implementation complete.
Ready for: demo-recording + PR (9-step pr-manager flow).
Worktree HEAD: `dcdbf95`. input-hash: `2e0eca2` (verified 2026-06-15; unchanged throughout).
develop HEAD remains: `7c0f453` (no merge yet).

Next after PR merge: demo-recorder → pr-manager → F4-wave-level convergence + F4 holdout eval
→ F5 / F6 / F7.
