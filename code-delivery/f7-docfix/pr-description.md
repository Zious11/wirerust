## Fix: F7 Convergence Documentation Fixes

**Branch:** `docs/f7-convergence-doc-fixes`
**Closes:** #62, #259
**Severity:** MED/MINOR/LOW (documentation only)
**Behavior change:** None — doc/help-text corrections only, one new regression test

---

### What Changed

This PR resolves 5 F7 holistic/consistency review findings. All changes are documentation, help text, and ADR corrections. No runtime behavior is altered.

| Finding | Severity | Change |
|---------|----------|--------|
| F-PASS-A-001 | MED | `src/cli.rs` `--mitre` doc-comment now mentions collapse-by-default + `--no-collapse` opt-out; adds regression test `mitre_help_text_mentions_collapse_behavior` |
| F-03 | MED | `docs/adr/0003` "Flat Mode Only for v0.8.0" section got supersession notice — 3 now-false collapse assertions marked pre-v0.9.0 with forward-ref to Grouped-Mode Collapse |
| F-04 | MINOR | `docs/adr/0003:238` bool vocabulary → struct vocabulary (`render.collapse == Collapse::Collapsed`) |
| F-05 | MINOR | `README.md` `--no-collapse` entry gained the `(xN)` suffix sentence to match cli.rs |
| F-PASSC-002 | LOW | `CHANGELOG.md` `[0.9.0]` date corrected: 2026-06-18 → 2026-06-19 |

### Files Changed

- `src/cli.rs` — `--mitre` and `--no-collapse` doc-comment updates
- `tests/cli_integration_tests.rs` — new `mitre_help_text_mentions_collapse_behavior` test
- `docs/adr/0003-reporting-pipeline-layering.md` — supersession notice + struct vocabulary fix
- `README.md` — `--no-collapse` description alignment
- `CHANGELOG.md` — release date correction

### Why

The F7 holistic review (issue #259, tracked from issue #62) identified these 5 documentation gaps introduced when v0.9.0 shipped the `FindingsRender` struct refactor. The headline behavior — collapse-by-default in both flat and `--mitre` modes — was present in code but absent from `--help` and docs, creating a UX discoverability gap.

### Testing

- All existing tests pass (`cargo test --all-targets`): 0 failures
- New test `mitre_help_text_mentions_collapse_behavior` added and mutation-fail verified (test fails when the collapse description is removed from help text)
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo fmt --check`: clean
- Demo recording: not required — no behavior change

### Verification: No Internal Finding-ID Leakage

A first draft of F-PASS-A-001 accidentally embedded the internal finding-ID `F-PASS-A-001` in the `--mitre` help text. This was caught and removed in commit `b34b3ab`. The `src/` directory has been verified to contain no internal finding-IDs (F-PASS-A-001, F-03, F-04, F-05, F-PASSC-002) in user-facing content.

### Pre-Merge Checklist

- [x] Branch uses semantic naming (`docs/f7-convergence-doc-fixes`)
- [x] PR title uses allowed semantic type (`docs`)
- [x] No internal finding-IDs in user-facing `--help` / doc-comments
- [x] All tests pass locally
- [x] clippy -D warnings clean
- [x] fmt clean
- [x] No behavior change (doc/help-text only)
- [x] New test for F-PASS-A-001 regression
- [ ] CI checks pass (pending)
- [ ] PR reviewer approved (pending)
- [ ] Security reviewer cleared (pending)
