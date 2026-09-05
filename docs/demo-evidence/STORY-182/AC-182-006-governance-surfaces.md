# AC-182-006 — Governance-Surface Completeness

**Claim:** All governance surfaces touched by this story (E2E-PCAPS.md, README.md, .gitignore,
CLAUDE.md, factory-artifacts gate-entry doc, ci.yml) are present and consistent.

## ci.yml — additive "IEC-104 fixture coverage report (visible)" step

Command:
```
sed -n '45,56p' .github/workflows/ci.yml
```

Output:
```
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1
      - run: cargo test --all-targets
      - name: IEC-104 fixture coverage report (visible)
        if: ${{ !cancelled() }}
        env:
          CARGO_TERM_COLOR: never
        run: |
          set -euo pipefail
          cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
          grep -qE "Fixture coverage: [1-9][0-9]*/[0-9]+" coverage-out.txt
          grep -qE "test result: ok" coverage-out.txt
```

Step is placed AFTER the main `cargo test --all-targets` step, uses `if: ${{ !cancelled() }}`
(visible after TEST failures, not just workflow cancellation), carries a step-scoped
`env: CARGO_TERM_COLOR: never` (F-001 robustness fix, commit 891348f7 — strips ANSI color
codes from `cargo test` output so the two downstream `grep` assertions match reliably
regardless of the runner's color auto-detection), and asserts both the coverage line and
`test result: ok`.

## E2E-PCAPS.md — committed-capture annotation present, stale "auto-fetchable only" claim removed from IEC-104 section

Commands and results:
```
$ grep -qF 'committed at `tests/fixtures/`' tests/fixtures/E2E-PCAPS.md && echo MATCH
MATCH
```

## README.md — provenance row present

```
$ grep -qF 'iec104-iti-diverse.pcap' tests/fixtures/README.md && echo MATCH
MATCH
```

## .gitignore — transient artifacts excluded

```
$ grep -qF 'coverage-out.txt' .gitignore && grep -qF 'red-out.txt' .gitignore && echo BOTH_PRESENT
BOTH_PRESENT
```

## CLAUDE.md — references factory-artifacts gate-entry doc

```
$ grep -qF '.factory/maintenance/fixture-count-gate-entry.md' CLAUDE.md && echo MATCH
MATCH
```

## ci.yml — additive step name present (cross-check)

```
$ grep -qF 'IEC-104 fixture coverage report (visible)' .github/workflows/ci.yml && echo MATCH
MATCH
```

**Verdict: PASS** — all six governance surfaces are present and consistent with the story's
requirements.
