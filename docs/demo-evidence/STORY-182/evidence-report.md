# Evidence Report — STORY-182

**Story:** STORY-182: E2E Fixture Manifest + Committed Representative Captures — Eliminate
False-Green `cargo test` in Clean Worktrees
**Epic:** E-11 (Tooling and Self-Improvement)
**Wave:** 86
**Date:** 2026-09-04
**Branch:** feature/STORY-182-iec104-e2e-fixture-manifest
**Product type:** Test infrastructure / CLI (`cargo test` harness change) — evidence captured
as raw terminal output transcripts (per demo-recorder dispatch for this story), not VHS/GIF,
since the deliverable is `cargo test` behavior rather than an interactive CLI product.

---

## Full Suite Status (context)

Implementation was reported complete and green prior to this recording session
(full suite passes; `iec104-iti-diverse.pcap` committed). This session captures
per-acceptance-criterion evidence only; it does not modify source or tests.

---

## Coverage Map

| AC | Description | Evidence File | Verdict |
|----|-------------|---------------|---------|
| AC-182-001 | Shared `fixture_path()` resolver; manifest coverage report + `FIXTURE-SKIPPED:` lines in both fixture-bearing (4/4) and clean-worktree-equivalent (1/4) environments | `AC-182-001-fixture-manifest-skip-reporting.md` | PASS |
| AC-182-002 | Committed capture `iec104-iti-diverse.pcap` tracked, ≤100 KB, sha256-verified against `E2E-PCAPS.md` | `AC-182-002-committed-capture-integrity.md` | PASS |
| AC-182-003 | Committed fixture always resolves via `COMMITTED_SAMPLES`; gated test runs to completion; zero `[iec104-e2e] SKIP:` lines for the committed fixture | `AC-182-003-committed-fixture-always-runs.md` | PASS |
| AC-182-004 | Clean-worktree run has fully observable outcome: committed test passes, gitignored fixtures skip visibly, manifest test passes when fixtures present | `AC-182-001-fixture-manifest-skip-reporting.md`, `AC-182-004-005-regression-guard.md` | PASS |
| AC-182-005 | Hard-assert: `test_fixture_manifest_report()` FAILS with `REGRESSION: committed fixture '...' is absent` when the committed capture is missing; recovers to green on restore | `AC-182-004-005-regression-guard.md` | PASS |
| AC-182-006 | Governance surfaces (ci.yml step, E2E-PCAPS.md, README.md, .gitignore, CLAUDE.md reference) present and consistent | `AC-182-006-governance-surfaces.md` | PASS |

---

## AC-182-001 / AC-182-004 Summary (manifest report, skip reporting, two-environment protocol)

Two-environment protocol executed on this host (which has `tests/fixtures/local-samples/`
populated — a fixture-bearing host):

- **Environment A (local-samples present):** `Fixture coverage: 4/4 fixtures present (0 fixture-gated tests will be skipped)` — no `FIXTURE-SKIPPED:` lines.
- **Environment B (local-samples moved aside for the duration of the command, then restored):** `Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)`, with three `FIXTURE-SKIPPED:` lines for `iec104.pcap`, `iec104-sq.pcapng`, `iec104-iti-dissect.pcap` — matches the story's canonical clean-checkout string exactly.
- CI-mode (no `--nocapture`): test passes silently, no `println!()` output surfaces, confirming the documented stdout/CI-visibility partition.

## AC-182-002 Summary (committed capture integrity)

`git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap` exits 0 (tracked).
Size: 13952 bytes (≤ 102400 byte gate). sha256:
`07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7` — matches
`tests/fixtures/E2E-PCAPS.md:358` exactly.

## AC-182-003 Summary (committed fixture never skips)

`test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` passes (not skipped);
`grep -c '\[iec104-e2e\] SKIP:'` on the captured output returns `0` — the committed fixture
never trips the skip path.

## AC-182-004 / AC-182-005 Summary (RED path — regression guard)

The committed `iec104-iti-diverse.pcap` was moved to a scratch backup and
`test_fixture_manifest_report()` was re-run:

```
thread 'iec104_e2e_real_pcaps::test_fixture_manifest_report' (...) panicked at tests/iec104_e2e_real_pcaps_tests.rs:813:13:
[iec104-e2e] REGRESSION: committed fixture 'iec104-iti-diverse.pcap' is absent from tests/fixtures/ — this is a broken checkout. Run `git checkout tests/fixtures/` to restore.
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

Process exit code `101` (genuine CI-visible failure). The file was then restored from the
scratch backup and the test re-run, confirming green:
`test result: ok. 1 passed; 0 failed; ...`. `git status` immediately after restoration
reported `nothing to commit, working tree clean` — the worktree was returned to its exact
pre-demo state.

## AC-182-006 Summary (governance surfaces)

All six required governance-surface checks passed:
- `.github/workflows/ci.yml` contains the "IEC-104 fixture coverage report (visible)" step with `if: ${{ !cancelled() }}`, placed after `cargo test --all-targets`.
- `tests/fixtures/E2E-PCAPS.md` contains the `committed at \`tests/fixtures/\`` annotation.
- `tests/fixtures/README.md` contains the `iec104-iti-diverse.pcap` provenance row.
- `.gitignore` lists both `coverage-out.txt` and `red-out.txt`.
- `CLAUDE.md` references `.factory/maintenance/fixture-count-gate-entry.md`.

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

The mandatory gate command (per `.factory/maintenance/demo-evidence-scrub-gate.md`) was run
against all captured raw output prior to writing these evidence files, matching for absolute
macOS- or Linux-style home-directory paths and tilde-form home references.

Result: **zero matches** — no absolute host paths or tilde-form home references were present
in any captured `cargo test` output, `git` output, or `grep` excerpts (all paths surfaced by
these commands are already repo-relative, e.g. `tests/fixtures/...`, `target/debug/deps/...`).
No scrubbing/substitution was required.

The gate was re-run against the final committed `docs/demo-evidence/STORY-182/` directory
after writing all files: zero results (gate PASS). (This sentence intentionally avoids
reproducing the gate's own regex literal, which would otherwise self-trigger the gate.)
