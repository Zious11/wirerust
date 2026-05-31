# Adversarial Review — STORY-096 (Implementation) — Pass 5

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, facade mode) |
| Scope | STORY-096 — `tests/cli_story_096_tests.rs` + traceability to BC-2.13.001/002/003/004, STORY-096.md |
| Strategy under review | brownfield-formalization (zero src changes) — tests prove ABSENCE of removed flags |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 5 |
| Date | 2026-05-31 |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree HEAD | abc4b4b (base develop c2445dc) + uncommitted pass-2 & pass-3 test fixes |
| Verdict | **CLEAN** (0 findings) — 2nd consecutive clean pass |

## Checkout Guard (DF-ADVERSARY-CHECKOUT-GUARD-001)

- Branch: `feature/STORY-096-absent-flag-rejection` — OK (not develop).
- Worktree-base attestation: HEAD abc4b4b, base develop c2445dc.
- `grep -c '#[test]'` = 14 (10 AC + 4 EC) — matches story.
- Only `tests/cli_story_096_tests.rs` modified (facade: zero src changes).

## Supplied / Self-Run Evidence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001)

| Axis | Result |
|------|--------|
| `cargo test --test cli_story_096_tests` | 14 passed; 0 failed |
| `cargo clippy --test cli_story_096_tests -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| S-7.01 AC-006: `pcap.version` dotted | FAILED (caught) — pass-2 fix holds |
| S-7.01 AC-004: `C2BeaconAnalyzer` in `src/summary.rs` | FAILED (caught) — pass-3 fix holds |
| NEW: reintroduce `--filter` as real Analyze flag | AC-005 FAILED + EC-003 FAILED (caught) |
| NEW: reintroduce `--beacon` as real Analyze flag | AC-003 FAILED + EC-002 FAILED (caught) |
| NEW: remove `--http` field | compile error E0026 in bin + test → build fails (caught) |
| NEW: add a harmless extra flag to Analyze | AC-010 + EC-004 still ok (NOT brittle / not over-specified) |

## Method

Fresh-context pass. Re-derived BC invariants independently. Ran the mandatory S-7.01
re-verification of both prior fixes, then attacked axes not probed in passes 1–4: the
mutation-resistance of the clap-REJECTION tests for `--filter` / `--beacon` (reintroduce
the flag → rejection test must fail), and the mutation-resistance + non-brittleness of
the POSITIVE-parse tests AC-010 / EC-004. All probes were live mutations against
worktree source with clean restoration verified.

## Prior-Fix Propagation Audit (S-7.01) — BOTH HOLD

- Pass-2 AC-006 `declares_dep`: `pcap.version` dotted form still caught.
- Pass-3 AC-004 runtime `src/` walk: `C2BeaconAnalyzer` in `src/summary.rs` still caught.
- No sibling-layer propagation required (each fix is localized to one test).

## Critical Findings

None.

## Important Findings

None.

## Observations

- **O-1 (non-finding):** All clap-rejection ACs/ECs are mutation-resistant. Reintroducing
  `--filter` (Option<String>) → AC-005 + EC-003 fail (`parse_err().unwrap_err()` panics
  when the flag becomes valid). Reintroducing `--beacon` → AC-003 + EC-002 fail.
  Combined with pass-3's confirmation for `--threats`/`--verbose`/`-v`, every
  flag-rejection test is mutation-resistant.
- **O-2 (non-finding):** EC-004 is mutation-resistant via compile-time coupling — its
  destructuring match `Commands::Analyze { http, targets, .. }` fails to COMPILE
  (E0026) if `--http` is removed, so the mutation is caught as a build failure (CI red),
  not silently. AC-010 likewise binds `targets`.
- **O-3 (non-finding):** Positive-parse tests are not over-specified — adding an
  unrelated harmless flag to `Analyze` leaves AC-010/EC-004 green (they assert only the
  fields they care about via `..`), so they will not false-FAIL on legitimate future
  flag additions.
- **O-4 (non-finding):** EC-003 fidelity to BC-2.13.003 EC-001 (`--filter "tcp port 80"`)
  is exact; AC-005 matches the BC canonical vector (`--filter tcp`). EC-001
  (`--threats` before subcommand) matches BC-2.13.001 EC-002. DF-AC-TEST-NAME-SYNC-001 +
  DF-TEST-NAMESPACE-001 PASS.

## Policy Rubric Compliance

| Policy | Verdict |
|--------|---------|
| DF-AC-TEST-NAME-SYNC-001 (MEDIUM) | PASS |
| DF-TEST-NAMESPACE-001 (MEDIUM) | PASS |
| DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH) | satisfied |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM) | satisfied |
| DF-SIBLING-SWEEP-001 (CRITICAL) | N/A — clean pass, no remediation |
| DF-VALIDATION-001 (HIGH) | N/A |

## Novelty Assessment

Novelty: **LOW** — no new gaps. The flag-rejection and positive-parse axes probed this
pass are all mutation-resistant (some via compile-time coupling). No findings;
remaining residue is the accepted name-pinned-absence limit noted in pass 4 (O-1 type
alias). Convergence confirmed on this axis.

## Verdict

**CLEAN after Pass 5** — 2nd of 3 consecutive clean passes. Both prior fixes hold;
flag-rejection and positive-parse tests confirmed mutation-resistant. One more clean
pass (6) required to reach the 3-consecutive-clean convergence minimum.
