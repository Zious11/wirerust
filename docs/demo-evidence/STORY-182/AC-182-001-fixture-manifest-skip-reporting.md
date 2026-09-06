# AC-182-001 — Shared Path Resolver + Fixture Manifest with Skip Reporting

**Claim:** `fixture_path()` is the shared resolver used by both `fixture_present()` and
`run_iec104_pipeline()`. `test_fixture_manifest_report()` prints a `Fixture coverage: N/M`
summary and `FIXTURE-SKIPPED:` lines for absent fixtures, visible with `--nocapture`.

Two-environment protocol (F-006): the same test is run once with `tests/fixtures/local-samples/`
present (fixture-bearing host — Environment A) and once with it moved aside (clean-worktree
equivalent — Environment B).

## Environment A — fixture-bearing host (local-samples present) → 4/4

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
Fixture coverage: 4/4 fixtures present (0 fixture-gated tests will be skipped)
test iec104_e2e_real_pcaps::test_fixture_manifest_report ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

All 4 `FIXTURE_MANIFEST` entries resolve (committed `iec104-iti-diverse.pcap` +
3 local-samples corpus files); no `FIXTURE-SKIPPED:` lines.

## Environment B — clean-worktree equivalent (local-samples moved aside) → 1/4

`tests/fixtures/local-samples/` was moved to a scratch location for the duration of this
command, then restored immediately afterward (worktree verified clean via `git status`
after restore — see `AC-182-004-005-regression-guard.md`).

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)
FIXTURE-SKIPPED: 'iec104.pcap' absent — corpus test will not run (check tests/fixtures/ for committed or tests/fixtures/local-samples/ for corpus)
FIXTURE-SKIPPED: 'iec104-sq.pcapng' absent — corpus test will not run (check tests/fixtures/ for committed or tests/fixtures/local-samples/ for corpus)
FIXTURE-SKIPPED: 'iec104-iti-dissect.pcap' absent — corpus test will not run (check tests/fixtures/ for committed or tests/fixtures/local-samples/ for corpus)
test iec104_e2e_real_pcaps::test_fixture_manifest_report ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

Matches the story's canonical clean-checkout string exactly:
`Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)`.

## CI-mode (no `--nocapture`) — passes silently

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact
```

Output (still under Environment B, local-samples aside):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
test iec104_e2e_real_pcaps::test_fixture_manifest_report ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

No `println!()` output appears without `--nocapture` (as documented in the story's
"Stdout vs. CI visibility" section) — test still reports `ok`.

**Verdict: PASS** — shared resolver confirmed by identical `fixture_path()` behavior across
both environments; manifest report + skip reporting confirmed in both the fixture-bearing
(4/4) and clean-worktree-equivalent (1/4) cases.
