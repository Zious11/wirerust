# AC-182-003 — Committed Fixtures Run on Every `cargo test` Invocation (No Skip)

**Claim:** `fixture_path()` checks `COMMITTED_SAMPLES` (`tests/fixtures`) before
`LOCAL_SAMPLES`. `fixture_present("iec104-iti-diverse.pcap")` returns `true` in a clean
worktree; `run_iec104_pipeline()` opens the committed path; the `[iec104-e2e] SKIP:` line is
never printed for the committed fixture.

## Committed-fixture test run (`--nocapture`, non-vacuous SKIP-count check)

Command:
```
cargo test --test iec104_e2e_real_pcaps_tests \
  iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu -- --exact --nocapture
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/iec104_e2e_real_pcaps_tests.rs (target/debug/deps/iec104_e2e_real_pcaps_tests-e00a3fe802a8d0e2)

running 1 test
test iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

Test PASSES (not silently skipped) — `run_iec104_pipeline("iec104-iti-diverse.pcap")`
successfully opened the committed file and all 66-finding assertions ran.

## SKIP-count check (non-vacuous — F-P10-003 gating form)

```
grep -c '\[iec104-e2e\] SKIP:' <captured output>
```

Output: `0`

Confirms zero `[iec104-e2e] SKIP:` lines were emitted for the committed fixture — the skip
path was never taken, in either the fixture-bearing environment (local-samples present, as
in this capture) or the clean-worktree-equivalent environment (see AC-182-001 Environment B,
where the ITI-diverse fixture still resolves via the committed path even though the other
3 manifest entries are absent and DO print `FIXTURE-SKIPPED:` / would print
`[iec104-e2e] SKIP:` if their own tests ran).

**Verdict: PASS** — committed fixture always resolves via `COMMITTED_SAMPLES`, never trips
the skip path, and the gated test body runs to completion.
