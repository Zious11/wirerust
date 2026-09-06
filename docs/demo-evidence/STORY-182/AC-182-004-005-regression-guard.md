# AC-182-004 / AC-182-005 — Regression Guard: Hard-Assert on Committed Fixture Absence (RED Path)

**Claim:** `test_fixture_manifest_report()` FAILS (not silently passes) with a visible
`REGRESSION: committed fixture '...' is absent` assertion message when the committed
`iec104-iti-diverse.pcap` is missing from `tests/fixtures/` — this is CI-visible because
panics are never suppressed by capture mode. When the file is restored, the test is green
again.

**Procedure:** the committed capture was moved to a scratch backup location (not deleted),
the test was run to capture the failure, then the file was moved back to its original path
before any other command executed. `git status` was checked immediately after restoration
to confirm the worktree returned to its exact pre-demo state.

## RED: committed fixture moved aside → hard-assert panic

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
Fixture coverage: 4/4 fixtures present (0 fixture-gated tests will be skipped)

thread 'iec104_e2e_real_pcaps::test_fixture_manifest_report' (511717814) panicked at tests/iec104_e2e_real_pcaps_tests.rs:813:13:
[iec104-e2e] REGRESSION: committed fixture 'iec104-iti-diverse.pcap' is absent from tests/fixtures/ — this is a broken checkout. Run `git checkout tests/fixtures/` to restore.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test iec104_e2e_real_pcaps::test_fixture_manifest_report ... FAILED

failures:

failures:
    iec104_e2e_real_pcaps::test_fixture_manifest_report

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test iec104_e2e_real_pcaps_tests`
```

Test process exit code: `101` (non-zero — genuinely fails CI, does not silently pass).

Note: the `Fixture coverage: 4/4` line printed above the panic is expected and correct in
this environment — `fixture_path()` (used by the coverage println!) still resolves
`iec104-iti-diverse.pcap` via `tests/fixtures/local-samples/` (present on this fixture-bearing
host), while the hard-assert loop checks the **committed location directly**
(`tests/fixtures/iec104-iti-diverse.pcap`, via `Path::exists()`, not `fixture_path()`) per the
story's F-005 design note — this is exactly why the direct-path check (not the resolver) is
required to catch committed-fixture regressions even when local-samples is populated.

## Restore + confirm green again

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
Fixture coverage: 4/4 fixtures present (0 fixture-gated tests will be skipped)
test iec104_e2e_real_pcaps::test_fixture_manifest_report ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

## Worktree-clean confirmation after restore

Command:
```
git status
```

Output:
```
On branch feature/STORY-182-iec104-e2e-fixture-manifest
nothing to commit, working tree clean
```

`git status` reports "nothing to commit, working tree clean" — the temporary move-aside of
the committed fixture left no trace; the working tree is byte-identical to its pre-demo state.

**Verdict: PASS** — the regression guard fails loudly and specifically
(`REGRESSION: committed fixture '...' is absent`) when the committed fixture is missing, and
recovers to green the moment the file is restored, with the worktree left clean.
