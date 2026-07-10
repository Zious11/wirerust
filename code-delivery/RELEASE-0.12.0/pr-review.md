# PR #394 Review — release/0.12.0 → main

**Verdict:** REQUEST_CHANGES (PASS-WITH-NOTES on code contents; merge blocked on state)

Fresh-eyes review by pr-reviewer (Opus 4.7). Scope: gitflow release PR bringing
`main` from `v0.11.4` to `v0.12.0`. 354 files, +7562/-773 across 21 commits from
`develop` since `v0.11.5` on develop side, plus the two release-specific
commits (`chore: bump version to 0.12.0` and `docs: finalize CHANGELOG for
v0.12.0`).

The code contents of the release are clean and internally consistent. The
merge itself is blocked on state (conflicts + missing CI evidence), not on
code.

---

## BLOCKER — merge readiness (state, not code)

### B1. Mergeable state is `CONFLICTING` / `mergeStateStatus: DIRTY`

`gh pr view 394 --json mergeable,mergeStateStatus` returns `CONFLICTING` /
`DIRTY`. The branch cannot merge into `main` as-is. Almost certainly the
`CHANGELOG.md` conflicts with whatever `main` currently holds at `[0.11.4]`,
since this PR is adding both a `[0.11.5]` and a `[0.12.0]` section on top of
it. Resolve before requesting merge.

### B2. No CI checks visible on the PR (`statusCheckRollup: []`)

Release-to-`main` PRs must show a green CI before merge (project convention;
CLAUDE.md release ritual: "after CI is green"). Either CI has not been
dispatched, workflow triggers filter out `release/*` → `main`, or the checks
list is genuinely empty. Confirm CI ran on the head SHA and produced a green
result before merging.

---

## MINOR

### M1. Version bump on `main` is `0.11.4 → 0.12.0`, not `0.11.5 → 0.12.0`

`Cargo.toml` patch:

```diff
-version = "0.11.4"
+version = "0.12.0"
```

The `[0.11.5]` CHANGELOG section is being **added by this PR**, and the
compare link `[0.11.5]: v0.11.4...v0.11.5` assumes a `v0.11.5` tag exists.
Consequence: `main` never received a proper `release/0.11.5` → `main`
gitflow PR. Either (a) `v0.11.5` was tagged off `develop` (violates
CLAUDE.md release ritual: "`main` is updated only through gitflow-proper
merges — never by direct commits") and this PR is silently absorbing that
governance gap, or (b) `v0.11.5` was never tagged and the compare link will
404. **Action:** verify `git tag -l v0.11.5` on origin. If absent, drop or
fix the `[0.11.5]` compare link before merge; if present, log the process
gap for a governance retro.

### M2. STORY-149 demo evidence is `.txt` transcripts only, no `.gif`/`.webm`

Every other story in the diff (150/156/157/159/160/161) ships gif+webm+tape
triples. STORY-149 ships only four `.txt` transcript files:

- `AC-149-001-bounded-borrow-invariant.txt`
- `AC-149-002-fragmented-fixture.txt`
- `AC-149-003-perf-recovery.txt`
- `AC-149-005-no-regressions.txt`

Not a release-PR blocker (per-story evidence was accepted at story-delivery
time on develop), but worth surfacing to the pr-manager retro since the
pr-reviewer checklist elevates `.txt`-only demos to BLOCKING at story-PR
time.

### M3. `Cargo.lock` change should be spot-audited

`Cargo.lock` is included in the release commit (as expected). Verify the
lock changes reflect only `indicatif 0.18.4 → 0.18.6` and `crossbeam-epoch
0.9.18 → 0.9.20`, not silent transitive upgrades. `Cargo.lock` is excluded
from the CHANGELOG-gate trigger set per CLAUDE.md, so a stray transitive
would slip past that gate.

---

## NIT

### N1. `src/summary.rs:59` — PF-001 saturating-arithmetic inconsistency

```diff
-        self.total_packets += 1;
+        self.total_packets = self.total_packets.saturating_add(1);
         self.total_bytes += packet.packet_len as u64;
```

`total_packets` was converted to `saturating_add`, but `total_bytes` on the
very next line was left as `+=`. Either an intentional scope carve-out
(bytes accumulator classified differently from a "diagnostic counter") or a
missed site in PF-001. Not a release blocker — PF-001 landed on develop and
shipped; noting for a follow-up sweep.

### N2. `src/reporter/json.rs:82` — `.unwrap()` on `to_string_pretty`

Pre-existing panic path in the JSON reporter, not introduced by this PR.
Noted only because the file was touched for STORY-160 (schema_version
envelope).

---

## PASS — verified items

1. **Version bump correct in `Cargo.toml`**: `0.11.4 → 0.12.0` (see M1 for
   the caveat about the missing `main`-side `0.11.5`).
2. **CHANGELOG structure correct**: `[Unreleased]` (empty) is retained
   above the new `[0.12.0] - 2026-07-10` block; both `[0.11.5]` and
   `[0.12.0]` compare links added; `[Unreleased]` compare link updated to
   `v0.12.0...HEAD`. Date `2026-07-10` matches today.
3. **BREAKING CHANGE accurately documented and implemented**:
   - CHANGELOG contains a full before/after mapping table
     (Verdict/Confidence/ThreatCategory) with an explicit "Terminal display
     tokens UNCHANGED" clause and a "Direction enum retains PascalCase"
     scope carve-out.
   - `src/findings.rs` implementation matches exactly:
     `#[serde(rename_all = "lowercase")]` on `Verdict` and `Confidence`;
     `#[serde(rename_all = "snake_case")]` on `ThreatCategory`.
     `fmt::Display` impls untouched. `Direction` untouched.
   - `src/reporter/json.rs` adds `SCHEMA_VERSION: &str = "2"` and injects
     `"schema_version": SCHEMA_VERSION` into the output object — matches
     the "value is a JSON **string**" claim.
4. **Release-specific commits are cleanly scoped**:
   - `c8488beb chore: bump version to 0.12.0` touches only `Cargo.toml`
     (+1/-1) and `Cargo.lock` (+1/-1).
   - `795fc9d9 docs: finalize CHANGELOG for v0.12.0` touches only
     `CHANGELOG.md` (+4/-1) — new `[Unreleased]` header + `[0.12.0]` link
     entry + compare link updates.
5. **No accidental inclusions in the diff**: no
   `.bak`/`.orig`/`.log`/`.tmp`/`.env`/`.DS_Store`/`target/` paths; no
   `<<<<<<<`/`>>>>>>>` conflict markers in any patch body. All 91 added
   files are legitimate benches (`benches/tls_fragmented.rs`), bin tools
   (`bin/lint-cycle-artifact` and its test harness), test fixtures
   (`tests/bc_149_*`, `tests/bc_150_*`,
   `tests/common/tls_fragmented_fixture.rs`), or per-story demo-evidence
   bundles.
6. **`Cargo.toml` new `[[bench]] tls_fragmented`** correctly accompanies
   STORY-149's shipped feature (CHANGELOG mentions it explicitly).
7. **Diff size** (~7,562 additions across 354 files) is large but
   appropriate for a minor release accumulating 21 commits; the vast
   majority of the volume is docs / demo-evidence, not source. `src/`
   changes are contained to 16 files with modest diffs.

---

## Recommendation

Once **B1** (rebase/resolve conflicts against `main`) and **B2** (verify
green CI on head SHA) are cleared and **M1** (verify `v0.11.5` tag exists
on origin, fix compare link if not) is confirmed, this release is safe to
merge. The code contents of the release itself — the BREAKING JSON casing
change, the schema envelope, the version + CHANGELOG mechanics — are all
coherent, complete, and match their documentation.
