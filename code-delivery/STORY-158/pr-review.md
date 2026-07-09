# PR #387 Fresh-Eyes Review — STORY-158 (Follow-up)

**Verdict:** APPROVE

**Reason:** The single blocking finding from the prior review — CI-side clippy drift on Rust 1.97 stable causing `clippy::for_kv_map` failures at `src/reporter/terminal.rs:502` and `:550` — has been resolved by commit `c4831bc` ("refactor(reporter): iter_mut → values_mut (clippy::for_kv_map, Rust 1.97 drift)"). All 12 required CI checks are green on the current HEAD, and the PR is `MERGEABLE`.

## Verification of the fix (`c4831bc`)

Reviewed the commit diff directly via `gh api`. The change is confined to `src/reporter/terminal.rs`, 2 lines added / 2 lines removed, applied identically at both offending sites:

```diff
- for (_, items) in buckets.iter_mut() {
+ for items in buckets.values_mut() {
```

Semantic equivalence check:
- `HashMap::iter_mut()` yields `(&K, &mut V)`; discarding the key with `_` leaves only `&mut V`, which is exactly what `values_mut()` yields.
- Both loop bodies reference only `items` (`items.sort_by_key(|(idx, f)| ...)`); the key is never used.
- No control-flow, allocation, iteration-order, or observable-behavior change.
- Follows clippy's own suggested autofix — the idiomatic clean fix, not a `#[allow]` suppression.

Commit message is honest about the drift origin ("Rust 1.97 drift") and correctly scopes the change ("No behavior change"). Semantic PR type `refactor(reporter)` matches the change class.

## CI status on HEAD `c4831bc`

All 12 required checks passing (queried via `gh pr view 387`):

| Job                                                        | Status  |
|------------------------------------------------------------|---------|
| Clippy                                                     | SUCCESS |
| Test                                                       | SUCCESS |
| Format                                                     | SUCCESS |
| Fuzz build                                                 | SUCCESS |
| Audit                                                      | SUCCESS |
| Deny                                                       | SUCCESS |
| Semantic PR                                                | SUCCESS |
| Action pin gate                                            | SUCCESS |
| Trust-boundary (test-seam gate)                            | SUCCESS |
| Help-provenance gate                                       | SUCCESS |
| Green-doc-tense gate (DF-GREEN-DOC-TENSE-SWEEP)            | SUCCESS |
| CHANGELOG gate (AC-158-001, PG-W71-CHANGELOG)              | SUCCESS |

Mergeable state: `MERGEABLE`.

## Prior verification carried forward

The prior review pass verified all eight acceptance criteria (AC-158-001 through AC-158-008), diff coherence, description accuracy, commit quality, dependency status, and SHA-pinned action refs. The clippy-fix commit does not touch any of that surface — it is a two-line refactor in a file unrelated to the STORY-158 scope. All prior verified conclusions carry forward unchanged.

## Findings

None. The prior blocking finding is resolved. No new findings introduced by the clippy fix. The optional NITs from the prior review remain optional and are not merge blockers.

## Recommendation

Merge.
