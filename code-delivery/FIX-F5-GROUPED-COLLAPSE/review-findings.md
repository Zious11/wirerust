---
document_type: review-findings
fix_pr_id: FIX-F5-GROUPED-COLLAPSE
pr_number: 270
pr_url: https://github.com/Zious11/wirerust/pull/270
branch: fix/f5-grouped-collapse-remediation
base: develop (181d5e2)
status: APPROVED — AWAITING HUMAN GATE
---

# Review Findings — Fix-PR #270 (F5 Grouped-Collapse Remediation)

## Convergence Table

| Cycle | Findings | Blocking | Non-Blocking | Fixed | Remaining |
|-------|----------|----------|--------------|-------|-----------|
| 1 | 0 | 0 | 0 | 0 | 0 |

**Verdict after cycle 1: APPROVE (pr-reviewer) + APPROVE (security-reviewer)**
Convergence achieved in 1 cycle.

---

## Security Review Findings (Cycle 1)

| ID | Title | Severity | Blocking | Resolution |
|----|-------|----------|---------|------------|
| — | No findings | — | — | — |

**Analysis:**
- `grouping_from_flag(bool) -> Grouping`: pure function, no user-controlled data, no unsafe, no I/O.
- Test additions use existing fixture `tests/fixtures/http-ooo.pcap` via `assert_cmd` — no new file paths.
- No new dependencies added (Cargo.toml unchanged).
- No new public API surface.
- No new network, filesystem, or process-execution paths.
- CHANGELOG and README changes are documentation-only.
- `cargo audit` CI check passes (9/9).

**Security verdict: APPROVE**

---

## PR Review Findings (Cycle 1)

| ID | Title | Severity | Blocking | Resolution |
|----|-------|----------|---------|------------|
| — | No blocking findings | — | — | — |

**Verification by finding:**

**F-B-001 (HIGH) — CHANGELOG [0.9.0] accuracy: RESOLVED**
- CHANGELOG [0.9.0] now records Phase 1 (STORY-120/PR #266 — enum) and Phase 2 (STORY-122/A/PR #268 — struct reshape) separately.
- STORY-119/B PR #269 grouped-collapse behavioral change recorded under `### Changed`.
- `--no-collapse` dual-scope recorded.
- False "byte-identical across all three modes" claim removed.
- Stale 0.8.0 forward-reference "deferred to a future release" corrected to "shipped in 0.9.0".

**F-B-002 (MEDIUM) — README flag-help accuracy: RESOLVED**
- README `--mitre` and `--no-collapse` entries updated to reflect 0.9.0 behavior.
- `--no-collapse` now described as dual-scope (flat and grouped).
- `--mitre` help mentions default collapse behavior.

**MEDIUM-1 (MEDIUM) — Non-tautological construction-site coverage: RESOLVED**
- `grouping_from_flag` extracted as pure function in `src/main.rs:506+`.
- `test_bc_2_11_030_grouping_flag_polarity` tests both polarities (`true → Grouped`, `false → Flat`).
- `mod story_119` AC-001/002/003 tests: `if true { X } else { Y }` constructs removed; replaced with direct enum construction + observable rendering assertions (tactic header presence, `(xN)` suffix, individual line count).
- E2e tests `mitre_flag_emits_tactic_headers_and_collapse_suffix` and `mitre_no_collapse_emits_tactic_headers_without_collapse_suffix` added in `tests/cli_integration_tests.rs`.
- Sanity-checked: swapping `Grouped/Flat` in `grouping_from_flag` fails unit test AND both e2e tests.

**PR reviewer verdict: APPROVE**

---

## CI Results (Cycle 1)

| Check | Status | Duration |
|-------|--------|----------|
| Test | PASS | 46s |
| Clippy | PASS | 19s |
| Format | PASS | 8s |
| Semantic PR | PASS | 4s |
| Action pin gate | PASS | 5s |
| Audit | PASS | 11s |
| Deny | PASS | 18s |
| Trust-boundary (test-seam gate) | PASS | 5s |
| Fuzz build | PASS | 1m15s |

**CI verdict: 9/9 PASS**
Run: https://github.com/Zious11/wirerust/actions/runs/27838569971

---

## Dependency Check

| Story/PR | Description | Status |
|----------|-------------|--------|
| STORY-120 / PR #266 | FindingsRender enum migration | MERGED (a4263c73) |
| STORY-122/A / PR #268 | FindingsRender struct reshape | MERGED (8696448) |
| STORY-119/B / PR #269 | Grouped-collapse + --mitre default flip | MERGED (181d5e2) |

**All upstream dependencies merged.**

---

## Current Gate Status

| Gate | Status |
|------|--------|
| Security review | APPROVE (0 findings) |
| PR reviewer | APPROVE (0 findings) |
| CI | 9/9 PASS |
| Deps merged | YES (PRs #266, #268, #269) |
| Human gate | PENDING — explicit authorization required before merge |
