---
document_type: evidence-report
story_id: STORY-150
wave: "71"
branch: feature/STORY-150-tls-drain-dry
head: 9b9010c
recorded: 2026-07-07
producer: demo-recorder
scrub: PG-W70-DEMO-SCRUB applied — zero absolute host paths in all committed artifacts
---

# STORY-150 Demo Evidence Report

**Story:** STORY-150 — TLS Drain-Loop DRY Refactor (TLS-DRAIN-DUP-001) + Kani VP-039 + Mutation Re-run  
**Wave:** 71 | **Branch:** `feature/STORY-150-tls-drain-dry` | **Head:** `9b9010c`

All recordings were produced using VHS 0.11.0 from the story worktree root.
`Wait+Line` was avoided for fast commands (completes before the check runs in VHS 0.11.0);
`Sleep`-based waits were used throughout.

---

## AC-150-001: Drain-loop DRY — single parse_tls_message_handshake call site

**Acceptance criterion:** exactly ONE `parse_tls_message_handshake` call and at most ONE
`msg_bytes` extraction in `process_handshake_carry`; no substantive logic block duplicated
between the C2S and S2C arms.

### Success path
| Artifact | Description |
|----------|-------------|
| `AC-150-001-drain-loop-green.gif` | `cargo test --test bc_150_drain_loop_dry_tests` — 10/10 PASS (2 structural tests + 7 behavior-preservation pins + 1 marker test) |
| `AC-150-001-drain-loop-green.webm` | Same recording, archival format |
| `AC-150-001-drain-loop-green.tape` | VHS script source |

**Result:** 10 passed; 0 failed.

### Error path
| Artifact | Description |
|----------|-------------|
| `AC-150-001-drain-loop-error-path.gif` | `git log --oneline -10` + `git show 10551ad --stat` — shows the red gate commit at 10551ad where the two structural tests FAILED before implementation |
| `AC-150-001-drain-loop-error-path.webm` | Same recording, archival format |
| `AC-150-001-drain-loop-error-path.tape` | VHS script source |

**Red gate narrative:** At commit `10551ad` (test-writer phase), both structural tests
failed as expected: `test_BC_150_001_..._parse_hs_call_not_duplicated` found 2 call sites
(not 1), and `test_BC_150_001_..._msg_bytes_extraction_not_duplicated` found 2 extraction
sites (not 1). This confirmed the red gate before implementation. Full log:
`.factory/cycles/wave-71/STORY-150/implementation/red-gate-log.md`.

---

## AC-150-002: Kani VP-039 harnesses pass after refactor

**Acceptance criterion:** all Kani harnesses in `kani_proofs_vp039` pass without modification
to their assertions after the AC-150-001 refactor.

### Success path — verify_no_usize_overflow_on_advance (fastest harness)
| Artifact | Description |
|----------|-------------|
| `AC-150-002-kani-vp039-overflow.gif` | Displays RESULTS + SUMMARY sections from pre-run transcript: 12/12 checks SUCCESS, 2/2 cover properties SATISFIED |
| `AC-150-002-kani-vp039-overflow.webm` | Same recording, archival format |
| `AC-150-002-kani-vp039-overflow.tape` | VHS script source |
| `AC-150-002-kani-vp039-overflow-transcript.txt` | Full scrubbed Kani output (paths replaced with `$REPO`/`$STDLIB` per PG-W70-DEMO-SCRUB) |

**Harness result:** `verify_no_usize_overflow_on_advance` — 12 checks PASS, verification
time 0.06s, 2/2 cover properties SATISFIED (non-vacuity confirmed).

### Orchestrator-verified results for remaining harnesses (AC-150-002 completeness)
These harnesses were verified by the orchestrator agent at commit `5fe40e7` and are cited
from `.factory/cycles/wave-71/STORY-150/implementation/evidence.md`. Re-running is omitted
(hours-long CBMC runs not appropriate for demo recording; the transcript for the fastest
harness is the live-run evidence):

| Harness | Checks | Result |
|---------|--------|--------|
| `verify_drain_loop_cursor_safety` | 75 | PASS |
| `verify_no_usize_overflow_on_advance` | 12 | PASS (live run this report) |
| `verify_carry_bounded_after_append` | 12 | PASS |

---

## AC-150-003: VP-039 line-correspondence table updated

**Acceptance criterion:** the VP-039 line-correspondence table references refactored code
structure; all stale references to the old duplicated arms removed.

### Success path
| Artifact | Description |
|----------|-------------|
| `AC-150-003-vp039-table.gif` | `sed -n '1807,1816p' src/analyzer/tls.rs` showing the 8-row table (model step → production line); `grep` confirming single `parse_tls_message_handshake` call at line 932 and single `let msg_bytes` extraction at line 927 |
| `AC-150-003-vp039-table.webm` | Same recording, archival format |
| `AC-150-003-vp039-table.tape` | VHS script source |

**Table location:** `src/analyzer/tls.rs` lines 1807–1816 (inside `mod kani_proofs_vp039`
header comment). The table describes the DRY-refactored `process_handshake_carry` structure
post AC-150-001 (STORY-150 / AC-150-001: per-direction arms unified; single shared
extraction + parse site).

**1:1 correspondence confirmed:**
- `parse_tls_message_handshake` appears at line 932 (single call site, DOWN from 2 pre-refactor)
- `let msg_bytes` extraction appears at line 927 (single site, DOWN from 2 pre-refactor)
- `consumed += 4 + body_len` cursor advance at line 957

---

## AC-150-004: Mutation testing — no new survivors

**Acceptance criterion:** `cargo mutants --jobs 1` reports no new surviving mutants relative
to pre-refactor baseline; mutation score on carry-drain path not degraded.

**Note:** This AC is covered by orchestrator-verified results only — re-running
`cargo mutants --jobs 1` takes multiple hours and is not appropriate for demo recording.

**Cited result** (from `.factory/cycles/wave-71/STORY-150/implementation/evidence.md`,
verified at head `5fe40e7`):

| Metric | Value |
|--------|-------|
| Total mutants analyzed | 45 |
| Caught | 42 |
| Pre-existing known misses | 3 (`compute_ja3` — acknowledged technical debt) |
| New misses introduced | **0** |
| Regression | **None** |

All 3 surviving mutants are pre-existing in the `compute_ja3` function and were present
before STORY-150. No new uncovered mutants on the carry-drain path.

---

## AC-150-005: Full CI gates pass

**Acceptance criterion:** `cargo test --all-targets` passes; `cargo clippy --all-targets
-- -D warnings` passes; `cargo fmt --check` passes.

### Success path
| Artifact | Description |
|----------|-------------|
| `AC-150-005-full-gates.gif` | All three gates run sequentially: `cargo test --all-targets` (grep for test result lines), `cargo clippy`, `cargo fmt --check && echo fmt:PASS` — all green |
| `AC-150-005-full-gates.webm` | Same recording, archival format |
| `AC-150-005-full-gates.tape` | VHS script source |

**Gate results:**
- `cargo test --all-targets`: all test result lines show `ok`
- `cargo clippy --all-targets -- -D warnings`: `Finished dev profile` (zero warnings)
- `cargo fmt --check`: exit 0, `fmt:PASS` confirmed

---

## AC-150-006: Anchor sweep — zero numeric line anchors in BC files

**Acceptance criterion:** BC-2.07.004.md and BC-2.07.028.md have zero `tls.rs:[digit]`
numeric line anchors (symbol anchors only, TD-031 compliant); STORY-054.md has corrected
line-range updates.

### Success path
| Artifact | Description |
|----------|-------------|
| `AC-150-006-anchor-sweep.gif` | `grep -c` showing 0 numeric anchors in both BC files; `grep 'tls\\.rs::'` showing symbol anchors present; `grep 'tls\\.rs:6'` on STORY-054.md showing corrected line-ranges |
| `AC-150-006-anchor-sweep.webm` | Same recording, archival format |
| `AC-150-006-anchor-sweep.tape` | VHS script source |

**Grep results confirmed (pre-verified):**
- `BC-2.07.004.md`: `grep -c 'tls\.rs:[0-9]'` → **0** (zero numeric anchors)
- `BC-2.07.028.md`: `grep -c 'tls\.rs:[0-9]'` → **0** (zero numeric anchors)
- `BC-2.07.004.md` symbol anchors: `tls.rs::TlsAnalyzer::prepare_record_step`, `::MAX_RECORD_PAYLOAD`, `::truncated_records` (TD-031 compliant)
- `BC-2.07.028.md` symbol anchors: `tls.rs::TlsAnalyzer::increment`, `::handle_client_hello` ×3 (TD-031 compliant)
- `STORY-054.md` corrected ranges: `tls.rs:60-68` (`is_weak_cipher`), `tls.rs:71-79` (`is_weak_server_cipher`), `tls.rs:82-87` (`cipher_name`), `tls.rs:628-662`, `tls.rs:735-748`, `tls.rs:664-685`, `tls.rs:750-771`

---

## Coverage Summary

| AC | Criterion | Success Path | Error Path | Status |
|----|-----------|-------------|------------|--------|
| AC-150-001 | Drain-loop DRY (single parse site) | `AC-150-001-drain-loop-green.{gif,webm}` | `AC-150-001-drain-loop-error-path.{gif,webm}` (red gate history) | PASS |
| AC-150-002 | Kani VP-039 harnesses pass | `AC-150-002-kani-vp039-overflow.{gif,webm}` + transcript | Cited: orchestrator evidence.md (hours-long runs) | PASS |
| AC-150-003 | VP-039 table updated | `AC-150-003-vp039-table.{gif,webm}` | N/A (structural check) | PASS |
| AC-150-004 | Mutation — no new survivors | Cited: orchestrator evidence.md (hours-long run) | N/A | PASS |
| AC-150-005 | Full CI gates | `AC-150-005-full-gates.{gif,webm}` | N/A (all-pass gate) | PASS |
| AC-150-006 | Anchor sweep | `AC-150-006-anchor-sweep.{gif,webm}` | N/A (grep zero-result) | PASS |

---

## PG-W70-DEMO-SCRUB Compliance

All committed text artifacts (`.tape`, `.txt`, `.md`) have been verified to contain zero
absolute host paths. Absolute paths in the Kani transcript were replaced with `$REPO` and
`$STDLIB` placeholders before writing the transcript file. VHS tape scripts use only
relative paths (relative to the worktree root where `vhs` is invoked).

Verification: `grep -rE '(absolute-host-path-pattern)' docs/demo-evidence/STORY-150/*.tape *.txt *.md`
Result: **zero matches** for absolute host paths in any committed text artifact.
