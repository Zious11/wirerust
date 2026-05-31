# Adversarial Review — STORY-096 (Implementation) — Pass 4

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, facade mode) |
| Scope | STORY-096 — `tests/cli_story_096_tests.rs` + traceability to BC-2.13.001/002/003/004, STORY-096.md |
| Strategy under review | brownfield-formalization (zero src changes) — tests prove ABSENCE of removed flags |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 4 |
| Date | 2026-05-31 |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree HEAD | abc4b4b (base develop c2445dc) + uncommitted pass-2 & pass-3 test fixes |
| Verdict | **CLEAN** (0 findings; 0 Critical/High/Medium) — 1st consecutive clean pass |

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
| Prior-fix AC-006: `pcap` inline + dotted | both FAILED (caught) — pass-2 fix holds |
| Prior-fix AC-004: `C2BeaconAnalyzer` in `src/summary.rs` | FAILED (caught) — pass-3 fix holds |
| NEW AC-006: `pcap` under `[target.'cfg(unix)'.dependencies]` | FAILED (caught) |
| NEW AC-006: `[target.'cfg(unix)'.dependencies.pcap]` nested | FAILED (caught) |
| NEW AC-006: `bpf-sys` (dep key) | caught at predicate level (verified by 19-case unit check, pass 2) |
| Brittleness AC-006: comment mentioning `pcap` | test ok (no false-fail) |
| Brittleness AC-006: `pcap-file` as `[dependencies.pcap-file]` table | test ok (sanity guard satisfied; `pcap` key not tripped) |
| Brittleness AC-002/009: LESSON-P1.04 comment present | test ok (comment text not a false-fail) |
| AC-004 type-alias `type BeaconAnalyzer = DnsAnalyzer;` | test ok — see O-1 (accepted name-pinned limit, not a finding) |

## Method

Fresh-context pass. Re-derived BC-2.13.001..004 invariants independently. Executed the
mandatory Partial-Fix Regression Discipline (S-7.01) re-verification of BOTH prior
fixes, then attacked NEW vectors not probed in passes 1–3: (a) `pcap` declared in
`[target.'cfg(...)'.dependencies]` tables (inline + nested header); (b) AC-004
type-alias evasion; (c) brittleness / false-FAIL surface (comments, legitimate
`pcap-file` table refactor, LESSON-P1.04 comment text). All probes were live mutations
against worktree source with clean restoration verified after each.

## Prior-Fix Propagation Audit (S-7.01) — BOTH HOLD

- **Pass-2 fix (AC-006 `declares_dep`)**: `pcap` inline + dotted still caught; and the
  same structural matcher also catches the NEW `[target.'cfg(unix)'.dependencies]`
  inline form and the `[target.'cfg(unix)'.dependencies.pcap]` nested-header form,
  because `declares_dep` checks `dependencies.{name}]` / `dependencies.{name}.` as
  substrings (target-cfg tables included). No residual gap.
- **Pass-3 fix (AC-004 runtime `src/` walk)**: `C2BeaconAnalyzer` in `src/summary.rs`
  still caught; coverage guard finds 24 `.rs` files (threshold 20). No regression.
- **Sibling layer (b)**: both fixes are localized to single tests; no sibling test
  shares the fixed predicate, so no sibling propagation required.

## Critical Findings

None.

## Important Findings

None.

## Observations

- **O-1 (non-finding, accepted limit):** `type BeaconAnalyzer = DnsAnalyzer;` (a type
  ALIAS) evades AC-004's `struct`/`impl` declaration predicates. This is NOT a real gap:
  BC-2.13.002 invariant 2 forbids a beacon-analyzer **struct** ("or equivalent struct");
  a type alias declares no new type and carries no beacon-detection logic (it merely
  renames an existing analyzer). Reintroducing actual beacon detection requires a real
  `struct`/`impl` with logic, which the runtime walk catches. Flagging the inert alias
  would be a nitpick contrivance, not a production-failure vector. Inherent, accepted
  limit of name-pinned absence tests.
- **O-2 (non-finding):** AC-006 is robust against false-FAIL — a comment line mentioning
  `pcap` does not trip `declares_dep` (anchors require the dependency-key prefix), and
  a legitimate refactor of `pcap-file` to a `[dependencies.pcap-file]` table header
  keeps the test green (sanity guard matches `dependencies.pcap-file]`; the `pcap` key
  is not a prefix of `pcap-file`). Not brittle.
- **O-3 (non-finding):** AC-002/009 do not false-FAIL on the LESSON-P1.04 comment, which
  names all four removed flags as text — the field-declaration predicates
  (`pub threats` / `"    threats:"` / `long = "threats"`, and verbose analogues) do not
  match comment prose.
- **O-4 (non-finding):** DF-AC-TEST-NAME-SYNC-001 PASS; DF-TEST-NAMESPACE-001 PASS;
  story FSR `tests/cli_tests.rs` vs dedicated file — cosmetic, documented (matches prior
  disposition).

## Policy Rubric Compliance

| Policy | Verdict |
|--------|---------|
| DF-AC-TEST-NAME-SYNC-001 (MEDIUM) | PASS — 10/10 AC citations resolve 1:1 to `fn test_*` |
| DF-TEST-NAMESPACE-001 (MEDIUM) | PASS — all 14 tests in `mod story_096` |
| DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH) | satisfied |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM) | satisfied (cargo + live mutations + prior-fix re-verify self-run) |
| DF-SIBLING-SWEEP-001 (CRITICAL) | N/A — no remediation this pass (clean) |
| DF-VALIDATION-001 (HIGH) | N/A — no deferred finding filed |

## Novelty Assessment

Novelty: **LOW** — no new gaps. New vectors probed this pass (target-cfg dependency
tables, AC-004 type-alias, brittleness/false-FAIL surface) either resolve to the
existing fixes (target-cfg caught) or to accepted name-pinned limits (inert type
alias). Findings, if any, would be refinements, not gaps. The implementation has
converged on the substantive mutation-resistance axes; remaining residue is the
inherent limit of structural name-pinned absence tests, consistent with the facade
strategy.

## Verdict

**CLEAN after Pass 4** — 1st of 3 consecutive clean passes. Both prior fixes
(F-W24-S096-P1-001 / P2-001 AC-006 dependency-key matcher; F-W24-S096-P3-001 AC-004
full-`src/`-walk) verified to hold under fresh-context re-attack. No new blocking
findings. Two more clean passes (5, 6) required to reach the 3-consecutive-clean
convergence minimum.
