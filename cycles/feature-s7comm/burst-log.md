---
document_type: burst-log
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-09-06T21:15:00Z
cycle: "feature-s7comm"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Burst Log — feature-s7comm

## Burst 1 (2026-09-06) — D-559 F2 Completion Gate Approved, BC Input-Hash Sweep

First burst-log entry for the feature-s7comm cycle. Full structured entry below.

Archived STATE.md Current Phase Steps row evicted by this burst (verbatim, unabridged — full text also
remains in STATE.md's Decisions Log at row `D-554`, unchanged):

> **D-554 MAINT-2026-09-05 POST-RUN EXECUTION RECONCILIATION (2026-09-05) — all 5 human-authorized
> Rust-dep Dependabot PRs (#458 clap/#443 serde_json/#442 anyhow/#444 serde/#459 owo-colors) MERGED to
> develop; develop tip 0b1ea806→adc9428d; CI fully green (Test/Fuzz success), zero regression.
> DEP-SOAK-FOLLOWUP-2026-07-27 + ROUTE-BC-DEFER-2026-07-11 CLEARED. Scope note: human also merged 2 held
> Actions-bump PRs beyond the authorized set (#457/#456); held-set narrowed to #455/#449/#436. #451 now
> DIRTY/conflicting; #407 unchanged BEHIND/mergeable; doc-fix still QUEUED. No release — still v0.13.3;
> main unchanged 46ebd6e3. Pipeline remains CLEAN/PAUSED. trajectory-tail →0→0→0→0. | **COMPLETE (D-554)**
> | (state-manager) STATE.md + maintenance/sweep-report-2026-09-05.md committed to factory-artifacts
> (single-commit burst, TD-VSDD-053). D-553 checkpoint archived to Session Resume Checkpoint history;
> D-554 checkpoint written. D-549 CPS row evicted (full text preserved verbatim in Decisions Log D-549
> row).

---

## Burst: D-559 F2 COMPLETION GATE APPROVED — BC INPUT-HASH SWEEP + F3 OPEN (2026-09-06)

**Parent-commit:** HEAD of factory-artifacts immediately prior to this burst's
commit (see `git -C .factory log -1 --format='%H' HEAD^` at commit time). Per
TD-VSDD-053, the current factory-artifacts HEAD is `git -C .factory log -1`,
not a string cited in this artifact.

**Adversary verdict:** N/A — bookkeeping/gate-approval + mechanical hash-rebaseline
burst; no adversarial pass conducted as part of this burst. F2 spec content was
not modified (only the `input-hash:` frontmatter field on 61 BC files was
corrected to match the now-final ARCH-INDEX.md v2.24 content); no code exists
yet for this cycle to review.

**Summary:** Two-task atomic burst per human directive. **Task 1 (canonical BC
input-hash sweep, F2 consistency follow-up flagged at D-558):** the ~60
new/amended feature-s7comm BCs carried `input-hash:` frontmatter left
advisory-stale after ARCH-INDEX.md was bumped to v2.24 post-authoring. Verified
via `bin/compute-input-hash` (per-file and directory-scoped verification, since
the tool's default `--scan` glob targets `.factory/stories/`) that all 62
in-scope BCs were STALE except `BC-2.21.037` (already rebaselined to `cf116b5`
at D-558). Rebaselined the remaining 61 via `bin/compute-input-hash --write`
(canonical tool only, per CLAUDE.md PG-HASH-HOOK-DIVERGENCE — never the bash
hook): `BC-2.05.013` → `cf116b5`; `BC-2.18.003`/`004`/`005`/`006` → `f156347`
(distinct `inputs:` set from the SS-20/SS-21 group); `BC-2.20.001`–`016` and
`BC-2.21.001`–`041` (excl. `037`) → `cf116b5`. Re-verified: all 62
feature-s7comm BCs now MATCH; the pre-existing 22-story background-stale set
(unrelated STORY-*.md files) is UNCHANGED — confirmed via a full
`.factory/stories/STORY-*.md` scan, still exactly 22 STALE, none newly
introduced, none of the 22 accidentally rewritten. **Task 2 (record F2
completion-gate approval → F3 open, D-559):** human approved the feature-s7comm
F2 completion gate (2026-09-06) with 3 decisions — (1) F2 APPROVED → F3 OPEN
(incremental-stories, epic E-23, wave-087); MITRE dispositions accepted (T0816
zero-call-sites this cycle, T0846 Setup-Communication-sweep scope, T1692.001
gated unexpected-source model, `Finding.confidence` per-finding limitation,
port-102 dynamic-gap classifier fix deferred to F4); (2) ADR-014 + the
CLAUDE.md port-102 edit are HELD for F4 — left uncommitted and inert (NOT
stashed) on the `develop` working tree, since F3 does not touch develop;
recorded as an explicit F4 obligation (`F4-OBLIGATION-ADR014-CLAUDEMD` in
Active Carry-Forwards) — the first F4 implementation PR must commit both and
move ADR-014 status proposed→accepted; (3) the BC hash sweep = Task 1 above.
`develop_head` UNCHANGED `97361cd4`; released `v0.13.3` unchanged;
`stories_delivered` 120 unchanged (F3 will add story counts). STATE.md updated
across frontmatter, EXACT RESUME POINT, Project Metadata, Phase Progress,
Concurrent Cycles, Current Phase Steps, Decisions Log, Active Carry-Forwards,
and Session Resume Checkpoint. STATE.md remains ~118KB/NEEDS-COMPACT —
advisory noted, `/compact-state` not performed this burst.

**Files touched (Dim-1): 66 unique files**

- .factory/specs/behavioral-contracts/ss-05/BC-2.05.013.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md (`input-hash` `4e9573e`→`f156347`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md (`input-hash` `4e9573e`→`f156347`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-18/BC-2.18.005.md (`input-hash` `4e9573e`→`f156347`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-18/BC-2.18.006.md (`input-hash` `4e9573e`→`f156347`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.001.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.002.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.003.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.004.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.005.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.006.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.007.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.008.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.009.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.010.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.011.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.012.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.013.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.014.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.015.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-20/BC-2.20.016.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.001.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.002.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.003.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.004.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.005.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.006.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.007.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.008.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.009.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.010.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.011.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.012.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.013.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.014.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.015.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.016.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.017.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.018.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.019.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.020.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.021.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.022.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.023.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.024.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.025.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.026.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.027.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.028.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.029.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.030.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.031.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.032.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.033.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.034.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.035.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.036.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.038.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.039.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.040.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/specs/behavioral-contracts/ss-21/BC-2.21.041.md (`input-hash` `8f268fc`→`cf116b5`, canonical rebaseline; no content change)
- .factory/STATE.md (D-559 transition: frontmatter version/last_amended/phase/current_step/current_cycle, EXACT RESUME POINT, Project Metadata Mode cell + Last-Updated row, Phase Progress F2 row→APPROVED + new F3 row OPEN, Concurrent Cycles feature-s7comm row, Current Phase Steps D-559 added + D-554 evicted, Decisions Log D-559 row, Active Carry-Forwards `F4-OBLIGATION-ADR014-CLAUDEMD` row added, Session Resume Checkpoint replaced, size-budget banner reconciled)
- .factory/cycles/feature-s7comm/session-checkpoints.md (created; D-558 checkpoint archived verbatim)
- .factory/cycles/feature-s7comm/burst-log.md (this file, created)
- .factory/stories/STORY-151.md (`input-hash` cascade-corrected `ebb35fc`→`e6626dc`: BC-2.18.003/004 rebaseline in this burst changed those files' raw bytes, transitively invalidating STORY-151's own hash since it lists them as `inputs:`; no content change)
- .factory/stories/STORY-173.md (`input-hash` cascade-corrected `00757f7`→`c0cb50f`: same BC-2.18.003/004 cascade as STORY-151; no content change)

**Codifications:** None — this burst is a canonical-hash-rebaseline +
human-gate-decision reconciliation burst, not a process-gap codification
event. No new PG-* entries; no policy changes.

**Dim-2 Attestation:** N/A — bookkeeping/gate-approval burst; no shell gates
applicable. No compilation or test execution performed; feature-s7comm has no
source code yet (F3/F4 have not started).

**Dim-5 Attestation:** N/A — no WASM binary changes. This burst writes only
`.factory/` artifacts.

**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst
commits exclusively to the factory-artifacts branch. ADR-014 + the CLAUDE.md
port-102 edit remain uncommitted and inert on the develop working tree by
explicit human decision (HELD for F4) — this burst does not touch develop.

**Dim-7 Attestation:** N/A — no test suite changes. Canonical input-hash
integrity verified via `bin/compute-input-hash --scan` (all 62 feature-s7comm
BCs MATCH post-rebaseline; pre-existing 22-story background-stale set
unchanged) per the state-burst Single-Commit Protocol (TD-VSDD-053).

**Post-verification cascade correction:** rebaselining `BC-2.18.003`/`004` (Task 1) changed those files' raw bytes. `STORY-151.md` and `STORY-173.md` both list `BC-2.18.003.md`/`BC-2.18.004.md` as `inputs:` (per the canonical algorithm, `input-hash` is computed over the raw bytes of every declared input file), so the BC rebaseline transitively invalidated their own input-hash values (both had been correctly rebaselined to MATCH at D-558, before this burst's further BC edits). A post-commit `--scan` re-verification caught this (MATCH count dropped 114→112, STALE rose 22→24). Both stories were re-rebaselined via the canonical tool (`STORY-151` `ebb35fc`→`e6626dc`; `STORY-173` `00757f7`→`c0cb50f`), restoring the background-stale set to exactly the original 22-story identity (verified byte-for-byte set-equal via diff against the pre-burst scan). No other story's hash was affected — confirmed via full `.factory/stories/STORY-*.md` re-scan.

**Closes:** feature-s7comm F2 completion gate (D-559, 2026-09-06) — human
approved F2→F3 transition with MITRE dispositions accepted, ADR-014/CLAUDE.md
HELD for F4 (obligation recorded), and the canonical BC input-hash sweep
complete. F3 incremental-stories (epic E-23, wave-087) is now OPEN.

---

## Burst: STORY-184 F4 In-Flight Adversarial Remediation — AC-Citation Sync (P1) + RFC-1006 §6 Correction & Length-Floor Divergence Rationale (P3) + Cascade Rehash (2026-09-06)

**Not a phase transition.** STORY-184 (F4, wave 87) is still mid-convergence —
this burst records factory-side spec corrections raised by STORY-184's own
adversarial review loop (Pass 1). No D-number bump, no Phase Progress row
change, no `current_step`/phase edit. The worktree code-side fixes for the
same review pass are committed separately on the `feature/STORY-184-tpkt-header-parser`
develop branch — out of scope for this factory-artifacts burst.

**Parent-commit:** HEAD of factory-artifacts immediately prior to this burst's
commit (see `git -C .factory log -1 --format='%H' HEAD^` at commit time). Per
TD-VSDD-053, the current factory-artifacts HEAD is `git -C .factory log -1`,
not a string cited in this artifact.

**Adversary verdict:** Pass 1 finding F-184-P1-001 (AC test-citation drift —
4 of STORY-184's acceptance criteria cited test function names that did not
match the names actually written by the test-writer) plus a Pass-3-class
finding on BC-2.20.001/002/003/014 (stale RFC 1006 section citation: TPKT
packet format is RFC 1006 §6, not §5) and an accompanying documentation gap
(BC-2.20.003/004 did not record why `parse_tpkt_header`'s `length >= 4` accept
floor intentionally diverges from RFC 1006 §6's stated packet-level `min=7`).
Remediated in this burst; STORY-184 convergence loop continues in a
subsequent pass.

**Files touched (Dim-1): 8 unique files**

- `.factory/stories/STORY-184.md` — AC-184-001/002/003/004 `**Test:**` citations
  updated to the actual test function names (`test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector`,
  `test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector`,
  `test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector`,
  `test_BC_2_20_004_valid_input_returns_some_header_length_4_canonical_vector`);
  `input-hash` cascade-rewritten `f8042db`→`a97f298` (BC content changed, see Rehash below).
  No AC semantics, thresholds, or traceability changed — citation-only fix.
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.001.md` — `RFC 1006 §5` →
  `RFC 1006 §6` citation correction (verified: TPKT packet format is RFC 1006 §6).
  `input-hash` unchanged (`cf116b5`, confirmed no-op — see Rehash below).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.002.md` — same §5→§6
  citation correction. `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.003.md` — same §5→§6
  citation correction, plus an additive "Rationale Note" section documenting
  the intentional layering divergence between `parse_tpkt_header`'s
  `length >= 4` structural-floor accept threshold and RFC 1006 §6's stated
  semantic packet-level `min=7` (COTP-presence validation deferred to the
  SS-21 COTP layer). Additive documentation only — accept range/postconditions
  unchanged. `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.004.md` — same §5→§6
  citation correction plus the same class of additive Rationale Note.
  `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.014.md` — same §5→§6
  citation correction. `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/stories/STORY-186.md` — no content change; `input-hash`
  cascade-rewritten `7a4a145`→`ce86f8c` (cites `BC-2.20.014.md` as input).
- `.factory/stories/STORY-194.md` — no content change; `input-hash`
  cascade-rewritten `8fdd307`→`0444185` (cites `BC-2.20.001.md` as input).

**Rehash (canonical tool only, `bin/compute-input-hash --write`):**
- `BC-2.20.001/002/003/004/014` own `input-hash` fields: verified via the
  canonical tool — **unchanged (no-op)**. Per the canonical algorithm, a BC's
  `input-hash` is computed from the raw bytes of its own declared `inputs:`
  (for these 5 files: `docs/adr/0014-...md` + `ARCH-INDEX.md`), not from the
  BC's own body text. Editing the BC's own prose does not alter either input
  file's bytes, so all 5 recomputed to the same stored value (`cf116b5`) —
  confirmed, not rewritten.
- `STORY-184.md`: `f8042db` → `a97f298` (BC-2.20.001/002/003/004 are listed
  as its `inputs:`; their raw bytes changed, invalidating the story's hash).
- Cascade sweep via `bin/compute-input-hash --scan`: identified `STORY-186.md`
  (cites `BC-2.20.014.md` as input) and `STORY-194.md` (cites `BC-2.20.001.md`
  as input) as newly cascade-stale. Rehashed both:
  `STORY-186.md` `7a4a145` → `ce86f8c`; `STORY-194.md` `8fdd307` → `0444185`.
  No content change to either story — hash-only cascade correction.
- Note on tooling: `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`
  is one of the `inputs:` for these BCs/stories but is HELD uncommitted on
  develop pending the first F4 implementation PR (F4-OBLIGATION-ADR014-CLAUDEMD,
  carried forward since D-559/D-561). Its bytes are already committed,
  byte-identical, on `feature/STORY-184-tpkt-header-parser` (commit `886bd3af`).
  The hash tool requires the file to exist at the resolved repo-root path to
  read it; it was read transiently from that branch to compute the hashes
  above, then removed — `docs/adr/` on the develop working tree was verified
  clean (`git status --porcelain docs/adr/` empty) before and after, and no
  develop-branch file was added, staged, or committed by this burst.

**Post-rehash verification:** `bin/compute-input-hash --scan` re-run after
all rewrites: `STORY-184.md`/`STORY-186.md`/`STORY-194.md` all report MATCH;
MATCH=125, STALE=22 — the STALE set is byte-for-byte identical to the
pre-existing 22-story background-stale set (`STORY-001..005`, `STORY-076..080`,
`STORY-129`, `STORY-157..159`, `STORY-161`, `STORY-164..165`, `STORY-175..179`)
— unchanged, none newly introduced, none accidentally rewritten.

**Codifications:** None — this burst is a factory-spec citation/rationale
correction + canonical-hash-rebaseline burst, not a process-gap codification
event. No new PG-* entries; no policy changes.

**Dim-2 Attestation:** N/A — no shell gates applicable. This burst edits
Markdown spec/story prose and frontmatter only; no compilation or test
execution was performed as part of this burst (the corresponding code-side
fix and its test run live on the `feature/STORY-184-tpkt-header-parser`
develop branch, out of scope here).

**Dim-5 Attestation:** N/A — no WASM binary changes. This burst writes only
`.factory/` artifacts.

**Dim-6 Attestation:** N/A — no source code or develop-branch changes. This
burst commits exclusively to the factory-artifacts branch. The transient
ADR-014 read (see tooling note above) touched no tracked or untracked state
on develop after cleanup.

**Dim-7 Attestation:** N/A — no test suite changes from this burst. Canonical
input-hash integrity re-verified via `bin/compute-input-hash --scan` (see
Post-rehash verification above).

**Closes:** STORY-184 adversarial Pass 1 finding F-184-P1-001 (AC-citation
drift) and the associated RFC-1006 §6 citation/rationale gap on
BC-2.20.001/002/003/004/014, factory-side only. STORY-184 remains OPEN
in F4 convergence — this is not a completion or phase-gate event.

---

## Burst: STORY-184 F4 In-Flight RFC 1006 §6 Min-Length=7 Rework, Human Ruling (2026-09-06)

**Not a phase transition.** STORY-184 (F4, wave 87) is still mid-convergence —
this burst records a factory-side spec rework directed by explicit human
ruling (RFC 1006 §6's stated packet-length minimum of 7 supersedes the prior
`>=4` structural-floor threshold). No D-number bump, no Phase Progress row
change, no `current_step`/phase edit; the convergence streak resets and
Pass 1 re-runs next against the reworked threshold. The worktree code/test/
CHANGELOG changes for the same rework are committed separately on the
develop story branch as `a23fb6ba` — out of scope for this factory-artifacts
burst.

**Parent-commit:** `1611dbd7f0b73e76331ff9c41bb1ed8eebf0462f` ("factory:
STORY-184 adversarial remediation — AC-citation sync (P1) + RFC-1006 §6
correction & length-floor divergence rationale (P3) + cascade rehash") — the
factory-artifacts HEAD immediately prior to this burst's commit. Per
TD-VSDD-053, the current factory-artifacts HEAD is `git -C .factory log -1`,
not a string cited in this artifact going forward.

**Adversary verdict:** N/A — this burst is not an adversarial-pass
remediation. It records a direct human ruling that retires the prior BC-
2.20.003/004 "intentional `>=4` vs RFC-min-7 layering divergence" rationale
(itself documented in the immediately-preceding burst above) and replaces
`parse_tpkt_header`'s accept floor with the RFC 1006 §6-conformant minimum
of 7. STORY-184's own convergence loop (adversarial Pass 1 remediation) is
unaffected by this note and continues in a subsequent pass.

**Files touched (Dim-1): 5 unique files**

- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.001.md` — additive
  clarifying note distinguishing the 4-byte `data.len() < 4` **structural
  read-guard** (this BC) from the 7-byte decoded-length **semantic floor**
  (BC-2.20.003/004). No precondition/postcondition change. `input-hash`
  unchanged (`cf116b5`, confirmed no-op — see Rehash below).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.003.md` — title and
  threshold `length < 4` → `length < 7`; "Rationale Note" section rewritten
  from "intentional `>=4` vs RFC-min-7 divergence" to "RFC 1006 §6-conformant
  minimum" (human ruling, 2026-09-06, retires the prior divergence rationale);
  edge cases EC-003..EC-007 and canonical test vectors renumbered/updated for
  the new `4`/`5`/`6` reject band and `7` accept floor; composes-with note for
  BC-2.20.004 updated to `[7, 65535]`; architecture-anchor planned-code
  fragment updated to `if length < 7`. `input-hash` unchanged (`cf116b5`,
  confirmed no-op — see Rehash below).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.004.md` — accept-range
  precondition/description `[4, 65535]` → `[7, 65535]`; "Rationale Note"
  rewritten to "Accept Floor is RFC 1006 §6-Conformant" (human ruling,
  2026-09-06); EC-001 and canonical test vectors updated to the `length == 7`
  minimum (the former `length == 4` happy-path vector removed as it is now a
  reject case). `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/specs/behavioral-contracts/ss-20/BC-2.20.015.md` — EC-001 and
  canonical test vectors' example byte sequences updated from `length == 4`
  to `length == 7` for consistency with the new floor (stale
  example/citation fix only — no semantic change to this BC's own resync
  contract). `input-hash` unchanged (`cf116b5`, confirmed no-op).
- `.factory/stories/STORY-184.md` — AC-184-003/AC-184-004 threshold prose
  `< 4`/`[4, 65535]` → `< 7`/`[7, 65535]` (RFC 1006 §6 minimum), AC-184-004
  `**Test:**` citation repointed to
  `test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector`,
  BC-summary table and Dev Notes/EC-006/EC-007 threshold prose swept
  consistently; `input-hash` cascade-rewritten `a97f298`→`24c7b1e` (BC content
  changed, see Rehash below).

**Rehash (canonical tool only, `bin/compute-input-hash --write`):**
- `BC-2.20.001/003/004/015` own `input-hash` fields: verified via the
  canonical tool — **unchanged (no-op)**. Each BC's `input-hash` is computed
  from the raw bytes of its own declared `inputs:` (`docs/adr/0014-...md` +
  `ARCH-INDEX.md`), not from the BC's own body text; editing the BC's own
  prose/thresholds does not alter either input file's bytes, so all 4
  recomputed to the same stored value (`cf116b5`) — confirmed, not rewritten.
- `STORY-184.md`: `a97f298` → `24c7b1e` (BC-2.20.001/003/004 are listed as
  its `inputs:`; their raw bytes changed, invalidating the story's hash).
- Cascade sweep via `bin/compute-input-hash --scan`: identified `STORY-186.md`
  (cites `BC-2.20.015.md` as input) and `STORY-194.md` (re-verification
  anchor citing `BC-2.20.001.md` as input) as newly cascade-stale. Rehashed
  both: `STORY-186.md` `ce86f8c` → `87f3feb`; `STORY-194.md` `0444185` →
  `7e8e4cb`. No content change to either story — hash-only cascade
  correction.
- Note on tooling: `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`
  is one of the `inputs:` for these BCs/stories but is HELD uncommitted on
  develop pending the first F4 implementation PR (F4-OBLIGATION-ADR014-CLAUDEMD,
  carried forward since D-559/D-561). Its bytes are already committed,
  byte-identical, on the develop story branch (commit `886bd3af`). The hash
  tool requires the file to exist at the resolved repo-root path to read it;
  it was read transiently from that commit to compute the hashes above, then
  removed — `git status --porcelain` on develop was verified clean before
  and after, and no develop-branch file was added, staged, or committed by
  this burst.

**Post-rehash verification:** `bin/compute-input-hash --scan` re-run after
all rewrites: `STORY-184.md`/`STORY-186.md`/`STORY-194.md` all report MATCH;
MATCH=125, STALE=22 — the STALE set is byte-for-byte identical to the
pre-existing 22-story background-stale set (`STORY-001..005`, `STORY-076..080`,
`STORY-129`, `STORY-157..159`, `STORY-161`, `STORY-164..165`, `STORY-175..179`)
— unchanged, none newly introduced, none accidentally rewritten.

**Codifications:** None — this burst is a factory-spec threshold-correction
+ canonical-hash-rebaseline burst driven by direct human ruling, not a
process-gap codification event. No new PG-* entries; no policy changes.

**Dim-2 Attestation:** N/A — no shell gates applicable. This burst edits
Markdown spec/story prose and frontmatter only; no compilation or test
execution was performed as part of this burst (the corresponding code-side
rework and its test run live on the develop story branch as `a23fb6ba`, out
of scope here).

**Dim-5 Attestation:** N/A — no WASM binary changes. This burst writes only
`.factory/` artifacts.

**Dim-6 Attestation:** N/A — no source code or develop-branch changes. This
burst commits exclusively to the factory-artifacts branch. The transient
ADR-014 read (see tooling note above) touched no tracked or untracked state
on develop after cleanup.

**Dim-7 Attestation:** N/A — no test suite changes from this burst. Canonical
input-hash integrity re-verified via `bin/compute-input-hash --scan` (see
Post-rehash verification above).

**Closes:** N/A — no adversarial finding ID closed by this burst; it is a
direct human ruling applied ahead of the next adversarial pass. STORY-184
remains OPEN in F4 convergence — this is not a completion or phase-gate
event.

---

**Burst note (2026-09-06):** In-flight STORY-184 F4 adversarial remediation, partial-fix-regression sweep completion — not a phase transition, no D-number/phase change. The prior burst above applied the RFC-1006 §6 min-length 4→7 correction but left stale `< 4` / `[4,65535]` references in BC-2.20.001's VP-row and Related-BCs section, BC-2.20.002's Related-BCs section, and BC-2.20.004's Related-BCs prose (BC-2.20.003 was already fully correct); this burst closes that gap. Cascade rehash via `bin/compute-input-hash --write` (canonical tool only): STORY-184 and STORY-194 rewritten (both cite the amended BCs as inputs); BC files' own `input-hash` unchanged (`cf116b5`, confirmed no-op — inputs are ADR-014 + ARCH-INDEX.md, not the BC body). Post-sweep `--scan`: MATCH=125, STALE=22 — the 22-story background-stale set is unchanged.

---
