## Summary

F5 scoped-adversarial remediation for the grouped-collapse delta (STORY-119/B + STORY-122/A, issue #62 / issue #259). Remediates 3 findings from the F5 review pass. Behavior is **byte-identical** — no output change to any user-visible path. This is documentation accuracy + coverage hardening only.

Closes: addresses F5 findings on #62 / #259 (no standalone issue filed per DF-VALIDATION-001)

---

## Findings Remediated

| Finding ID | Severity | Category | Fix |
|------------|----------|----------|-----|
| F-B-001 | HIGH | Doc accuracy | CHANGELOG [0.9.0] corrected |
| F-B-002 | MEDIUM | Doc accuracy | README flag help synced to cli.rs doc-comments |
| MEDIUM-1 | MEDIUM | Test quality / construction-site coverage | `grouping_from_flag` helper extracted + non-tautological tests |

### F-B-001 (HIGH) — CHANGELOG [0.9.0] correctness

The CHANGELOG [0.9.0] `### Changed (BREAKING)` block previously described only the
`FindingsRender` three-variant enum (STORY-120/PR #266). It did **not** record the Phase 2
reshape to a struct of two orthogonal enums (STORY-122/A/PR #268), omitted the grouped-collapse
behavioral change and `--no-collapse` dual-scope (STORY-119/B/PR #269), and contained a false
claim that "Terminal output is byte-identical across all three modes" (untrue after STORY-119/B
lands). The stale 0.8.0 forward-reference ("grouped-mode collapse is deferred to a future
release") also remained after STORY-119/B shipped it.

**Fix (commit `1bcd286`):** Rewrote CHANGELOG [0.9.0] to record both phases explicitly, removed
the false byte-identical claim, corrected the 0.8.0 forward-reference to read "grouped-mode
collapse shipped in 0.9.0".

### F-B-002 (MEDIUM) — README flag-help accuracy

The README `--mitre` and `--no-collapse` help text did not reflect STORY-119/B behavior:
`--no-collapse` was described as suppressing collapse only in flat mode (pre-0.9.0 behavior),
and `--mitre` help did not mention default collapse.

**Fix (commit `7a3b872`):** README `--mitre` and `--no-collapse` entries updated to match the
authoritative doc-comments in `src/cli.rs` (or equivalent), reflecting dual-scope suppression
and default grouped collapse.

### MEDIUM-1 (MEDIUM) — Non-tautological grouping construction-site coverage

The `(show_mitre_grouping, collapse_findings) → FindingsRender{grouping, collapse}` mapping in
`src/main.rs` was untested by any real assertion. The `story_119` AC-001/002/003 tests in
`tests/reporter_terminal_tests.rs` constructed literal mirror copies of the mapping logic
(`if true { Grouping::Grouped } else { ... }`) — tautologies that would pass even if the
production wiring were entirely swapped.

**Fix (commit `ccfee5b`):** Three coordinated changes:

1. **Extract helper** (`src/main.rs`): Added pure function `grouping_from_flag(show_mitre_grouping: bool) -> Grouping` alongside existing `collapse_findings_from_flag`. Rewired the `FindingsRender` construction site to call it. Added unit tests `test_bc_2_11_030_grouping_flag_polarity` verifying both polarities — a Grouped/Flat swap fails immediately.

2. **Rewrite tautological tests** (`tests/reporter_terminal_tests.rs`): Removed the `if true/false { X } else { Y }` constructions from AC-001/002/003 in `mod story_119`. Replaced with direct `FindingsRender` construction and observable rendering assertions (tactic header presence, `(xN)` suffix, individual line count).

3. **E2E CLI guard** (`tests/cli_integration_tests.rs`): Added two `assert_cmd` tests:
   - `mitre_flag_emits_tactic_headers_and_collapse_suffix`: `--all --mitre` emits `## ` tactic headers AND `(xN)` suffix.
   - `mitre_no_collapse_emits_tactic_headers_without_collapse_suffix`: `--all --mitre --no-collapse` emits `## ` headers but no `(xN)` suffix.
   Fixture: `tests/fixtures/http-ooo.pcap` (existing, 1,209 B).

**Sanity check:** Swapping `Grouped`/`Flat` in `grouping_from_flag` fails the unit test AND both e2e tests. Assertions are non-tautological.

---

## Architecture Changes

No architectural change. `grouping_from_flag` is a pure helper extracted from an inline
`if/else` at the construction site — factoring only, no new subsystem.

```mermaid
graph TD
    A[src/main.rs run_analyze] --> B[grouping_from_flag]
    A --> C[collapse_findings_from_flag]
    B --> D[FindingsRender construction]
    C --> D
    D --> E[TerminalReporter::render]
```

---

## Spec Traceability

```mermaid
flowchart LR
    BC030["BC-2.11.030\n(grouping flag)"] --> AC001["STORY-119/B AC-001\n(--mitre groups)"]
    BC030 --> UT["unit test\ntest_bc_2_11_030_grouping_flag_polarity"]
    BC030 --> E2E1["e2e: mitre_flag_emits_tactic_headers_and_collapse_suffix"]
    BC031["BC-2.11.031\n(--no-collapse dual-scope)"] --> AC002["STORY-119/B AC-002\n(--no-collapse)"]
    BC031 --> E2E2["e2e: mitre_no_collapse_emits_tactic_headers_without_collapse_suffix"]
```

---

## Story Dependencies

```mermaid
graph LR
    S120["STORY-120\n(FindingsRender enum)\nMERGED PR #266"] --> S122["STORY-122/A\n(struct reshape)\nMERGED PR #268"]
    S122 --> S119["STORY-119/B\n(grouped-collapse)\nMERGED PR #269"]
    S119 --> THIS["fix/f5-grouped-collapse-remediation\n(this PR — F5 remediation)"]
```

All dependency PRs are merged.

---

## Test Evidence

| Suite | Count | Result |
|-------|-------|--------|
| cargo test --all-targets | 64 suites, all tests | 0 failures |
| cargo clippy --all-targets -- -D warnings | — | Clean |
| cargo fmt --check | — | Clean |

New tests added by this PR:
- `test_bc_2_11_030_grouping_flag_polarity` — unit test in `src/main.rs` (both polarities)
- `mitre_flag_emits_tactic_headers_and_collapse_suffix` — e2e CLI test
- `mitre_no_collapse_emits_tactic_headers_without_collapse_suffix` — e2e CLI test

---

## Security Review

N/A — no new logic paths, no user-controlled data routing, no parser changes. Documentation
and test changes only (plus pure-function helper extraction from existing inline `if/else`).

---

## Risk Assessment

| Dimension | Assessment |
|-----------|------------|
| Blast radius | Low — 3 files changed: CHANGELOG.md, README.md, src/main.rs (helper extraction), tests/ (tautology removal + e2e addition) |
| Behavior change | None — byte-identical for all user-visible output paths |
| Performance impact | None |
| Breaking change | None |

---

## Holdout Evaluation

N/A — evaluated at wave gate. No new behavioral contract introduced; behavior byte-identical.

---

## Adversarial Review

N/A — this PR IS the F5 adversarial remediation. Reviewed by pr-reviewer and security-reviewer
as part of fix-pr-delivery flow.

---

## AI Pipeline Metadata

| Field | Value |
|-------|-------|
| Pipeline mode | fix-pr-delivery (F5 remediation, no stubs/Red Gate/wave gates) |
| Stories | STORY-119/B (grouped-collapse), STORY-122/A (struct reshape) |
| Feature | E-8 / issue #62 / issue #259 grouped-collapse delta |
| Branch | `fix/f5-grouped-collapse-remediation` |
| Base | `develop` |

---

## Pre-Merge Checklist

- [x] All 3 F5 findings remediated (F-B-001 HIGH, F-B-002 MEDIUM, MEDIUM-1 MEDIUM)
- [x] Behavior byte-identical (no user-visible output change)
- [x] `cargo test --all-targets` passes (64 suites, 0 failures)
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] Tautological tests replaced with observable-rendering assertions
- [x] E2E tests verify non-tautological wiring (swap breaks both e2e tests)
- [x] All dependency PRs merged (#266, #268, #269)
- [ ] CI checks passing (pending push)
- [ ] pr-reviewer APPROVE (pending)
- [ ] security-reviewer APPROVE (pending)
- [ ] HUMAN GATE — merge requires explicit human authorization
