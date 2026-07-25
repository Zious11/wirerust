---
document_type: adversarial-findings
cycle: wave-086
pass: 2
pass_date: 2026-07-25
reviewer: vsdd-factory:adversary
finding_count: 23
severity_breakdown: "1C / 4H / 10M / 7L / 1N"
remediation_status: ALL_FIXED
stories_affected: [STORY-182, STORY-183]
story_versions_after: "STORY-182 v1.2 (input-hash f025b3b), STORY-183 v1.2 (input-hash 3831f42)"
clean_streak_after: 0
decision: D-518
po_ruling: "DF-GREEN-DOC-TENSE-SWEEP v2→v3 two-tier model (F-W86S-P2-006)"
---

# Wave-086 Adversarial Pass 2 — Findings Audit

All 23 findings from the wave-86 story adversarial pass 2.
Reviewed STORY-182 v1.1 and STORY-183 v1.1.
Remediation applied: STORY-182 v1.1→v1.2 (4 pts unchanged), STORY-183 v1.1→v1.2 (3→5 pts).

PO ruling (F-W86S-P2-006): DF-GREEN-DOC-TENSE-SWEEP v2→v3 two-tier model committed to
`policies.yaml`. TIER-1 = zero-FP automatable tokens (incl. `currently asserts`, `is
currently`, `currently outputs`). TIER-2 = context-dependent manual-sweep tokens (incl. `is
expected to`, `falls through to`, `is assumed to`). TIER-2 non-flagging is NOT a tool defect
and adversarial passes MUST NOT count it as a finding.

---

## STORY-183 Findings (10 findings: 1C / 2H / 3M / 3L / 1N)

### CRITICAL

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-001 | CRITICAL | STORY-183 v1.1 Patterns 30 (`currently falls through`) and 31 (`is expected to`) matched ZERO of the 9 real stale sites identified in the D-506 adversarial record (cycles/wave-085/STORY-180/convergence-report.md lines 63-66). The wave-85 lesson summary that seeded these patterns mislabeled the phrase classes: the actual stale phrasing observed was `currently asserts` (TIER-1 automatable) and `is expected to` (TIER-2 context-dependent). Pattern 30 (`currently falls through`) never appeared in real output; Pattern 31 (`is expected to`) is TIER-2 per the PO ruling and must NOT be in the automated TIER-1 list. With Patterns 30/31 ineffective, a document containing the real stale phrases from D-506 would produce a false-green. | story-writer + PO ruling | FIXED in v1.2: Patterns 30/31 removed from TIER-1 list. 9 new TIER-1 patterns 32-40 added covering the actual phrase classes from real D-506 stale sites (`currently asserts`, `is currently`, `currently outputs`, etc.). `is expected to` designated TIER-2 (manual sweep, non-flagging). PO ruling DF-GREEN-DOC-TENSE-SWEEP v3 embedded verbatim in STORY-183 body. Points raised 3→5 to reflect redesigned scope. |

### HIGH

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-002 | HIGH | STORY-183 v1.1 positive-coverage ACs (AC-183-007/008 from pass-1 remediation) test against phrase-list entries that were themselves incorrect (Patterns 30/31 as critiqued in F-001). A positive-coverage test that passes against a wrong pattern produces false confidence — the tool exits non-zero on fabricated input that would never appear in real code. | story-writer | FIXED in v1.2: Positive-coverage ACs rewritten to use real-world fixture text derived from D-506 convergence-report examples. AC-183-007/008 now specify the exact phrase text (`currently asserts the behavior` pattern) from real stale sites, not the removed Patterns 30/31. |
| F-W86S-P2-005 | HIGH | STORY-183 v1.1 AC-183-009 (D-506 efficacy AC) references `Expected RED: TypeID 58` as the regression fixture, but the D-506 convergence report (lines 63-66) identifies `currently asserts` as the primary stale-phrase class found in that session — not `Expected RED:`. The efficacy AC would pass against a tool that detects `Expected RED:` but fails to detect `currently asserts`, providing false assurance against the real regression vector. | story-writer | FIXED in v1.2: AC-183-009 updated to use `currently asserts` as the efficacy fixture text (matching the real D-506 stale phrase class). `Expected RED:` retained as a separate TIER-1 pattern (Pattern 26) but removed as the primary efficacy anchor. |
| F-W86S-P2-006 | HIGH → PO RULING | STORY-183 v1.1 includes `is expected to` and `falls through to` in the TIER-1 automated scan list. These phrases are context-dependent: `is expected to` in a doc comment may be descriptive prose ("the function is expected to return a value") rather than a stale test phrase. Flagging these would produce false positives in architectural documentation, ADRs, and PRD sections. | PO RULING: DF-GREEN-DOC-TENSE-SWEEP v2→v3 two-tier model. `is expected to` + `falls through to` → TIER-2 (manual sweep, non-flagging by automated tool). Zero-FP automatable tokens → TIER-1 only. | FIXED in v1.2: TIER-2 tokens removed from STORY-183 TIER-1 pattern list. policies.yaml updated with v3 two-tier model. |

### MEDIUM

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-011 | MEDIUM | STORY-183 v1.1 AC-183-003 specifies `bin/*.py` glob for extensionless Python script detection but the shebang-detection logic description is ambiguous: "identified as Python via shebang line" does not specify whether this means `#!/usr/bin/env python3` only, or any shebang containing `python`. Scripts using `#!/usr/bin/python` (absolute path) or `#!/usr/bin/env python` (Python 2 legacy) would be misidentified if the implementation tests only for `python3`. | story-writer | FIXED in v1.2: AC-183-003 shebang detection tightened to match any shebang containing `python` (case-insensitive substring match on the shebang line), covering `python`, `python3`, `python2`, and absolute-path variants. |
| F-W86S-P2-015 | MEDIUM | STORY-183 v1.1 specifies Option B (integrate check into the existing `cargo test` hook) as an alternative implementation path. Option B introduces a dependency on Rust test harness behavior that is out of scope for a pure-Python bin/ tool story and would require changes to `build.rs` or a test helper — contradicting the E-11 "Python-only bin/ tooling" framing. | story-writer | FIXED in v1.2: Option B removed entirely. Story body revised to specify standalone Python script invocation only. AC scope aligned to `bin/check-green-doc-tense` standalone execution. |
| F-W86S-P2-018 | MEDIUM | STORY-183 v1.1 AC-183-012 (convention note) says "follows `bin/test_<tool>.py` pattern" but does not specify the self-test invocation mechanism. `bin/compute-input-hash` has `bin/test_compute_input_hash.py` invoked via `python3 bin/test_compute_input_hash.py` in the CI `bin/` Python self-tests step. STORY-183 must specify the same invocation to be picked up by the existing CI gate. | story-writer | FIXED in v1.2: AC-183-012 updated to specify `python3 bin/test_check_green_doc_tense.py` as the invocation, matching the existing CI `bin/` self-test invocation pattern. |

### LOW

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-020 | LOW | STORY-183 v1.1 does not specify what the tool prints when it finds zero stale phrases (clean scan). `bin/compute-input-hash` prints the hash; the check tool should print something like "OK: N files scanned, 0 stale phrases" or remain silent. Without specifying, implementations may print verbose output that clutters CI logs or print nothing (making it hard to distinguish a hung process from a clean scan). | story-writer | FIXED in v1.2: AC-183-013 added — clean scan produces exactly one summary line `OK: {N} files scanned, 0 stale phrases found` to stdout; exit 0. |
| F-W86S-P2-022 | LOW | STORY-183 v1.1 does not specify behavior when invoked with zero arguments. `bin/check-green-doc-tense` with no path args would either scan nothing (vacuous success) or error. The clean-worktree false-green scenario requires that running the tool against `bin/` with no explicit args defaults to scanning all `bin/` Python files — but this default must be explicit. | story-writer | FIXED in v1.2: AC-183-014 added — zero-arg invocation defaults to scanning `bin/` directory (all `.py` files + extensionless Python scripts); equivalent to `bin/check-green-doc-tense bin/`. |
| F-W86S-P2-023 | NITPICK | STORY-183 v1.1 body section "Background" uses present-tense claim "the tool currently scans" which is stale for a story describing a tool that does not yet exist. Should be "the tool will scan" or "AC-183-001 requires the tool to scan". | story-writer | FIXED in v1.2: "currently scans" → "will scan" (pre-delivery tense). |

---

## STORY-182 Findings (12 findings: 0C / 2H / 7M / 3L)

### HIGH

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-003 | HIGH | STORY-182 v1.1 AC-182-002 specifies committed fixtures at `tests/fixtures/iec104/real/` but also references a `committed-samples/` directory in the story body as an intermediate staging location. These two paths contradict: `committed-samples/` is not a recognized project directory and would require a new tree entry, whereas the existing `tests/iec104_e2e_real_pcaps_tests.rs` uses `tests/fixtures/iec104/`. A developer following the body description would create an orphan `committed-samples/` directory that CI cannot find. | story-writer | FIXED in v1.2: `committed-samples/` directory dropped entirely from story body. All references unified to `tests/fixtures/iec104/real/`. |
| F-W86S-P2-004 | HIGH | STORY-182 v1.1 includes "loud-claims" assertions (e.g., `assert!(fixtures_present, "expected {} fixtures to be present", N)`) that assume a fixed fixture count N. Libtest captures stdout/stderr from passing tests by default, so a failing assertion message citing N would not appear in CI output unless `--nocapture` is passed or `eprintln!` is used. The story's AC relies on message visibility that may not hold under default `cargo test` invocation. | story-writer | FIXED in v1.2: Loud-claims rewritten to use `eprintln!` for the fixture-count context and `assert!` for the boolean condition separately. AC-182-003 note added specifying that the assertion message must appear in CI output under `cargo test` without `--nocapture`. |

### MEDIUM

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-007 | MEDIUM | STORY-182 v1.1 AC-182-004 specifies a `#[test] fn fixture_manifest_all_present()` test but does not specify whether this test should be `#[ignore]` for the clean-worktree case. If the test is NOT `#[ignore]` and fixture files are absent, `cargo test` will fail on a clean checkout, which is the exact scenario the story is supposed to prevent (a failing test, not a false-green). The AC must specify the `#[ignore]` rationale or an alternative that preserves correct behavior on both clean and fixture-present checkouts. | story-writer | FIXED in v1.2: AC-182-004 updated — `fixture_manifest_all_present()` is NOT `#[ignore]`; it asserts that all manifest entries resolve to extant files. On clean checkout with no committed fixtures AND no manifest, this test fails with a clear message rather than passing silently. The `#[ignore]` rejection rationale added to story body: using `#[ignore]` would defeat the purpose of the gate. |
| F-W86S-P2-008 | MEDIUM | STORY-182 v1.1 manifest-count assertions do not specify the count source of truth. If the manifest is auto-generated from the committed fixture directory, then adding a new fixture file updates the manifest count automatically. If the manifest is hand-maintained, the count drifts. The story must specify how the manifest file is maintained to prevent count drift. | story-writer | FIXED in v1.2: AC-182-001 note added — manifest is hand-maintained (one relative path per line); adding a new fixture requires manually adding its path to `tests/fixtures/iec104/manifest.txt`. The test `fixture_manifest_all_present()` enforces that all listed paths exist but does not enforce count. Count drift is prevented by CI failing if a listed path is absent. |
| F-W86S-P2-009 | MEDIUM | STORY-182 v1.1 does not specify the license metadata required for committed ITI CC-BY-4.0 captures. CC-BY-4.0 requires attribution. The captures must carry a `LICENSE` file or `README` with attribution text, otherwise the commit violates the license terms. The story must specify what attribution artifact accompanies the committed fixtures. | story-writer | FIXED in v1.2: AC-182-006 added — committed captures directory `tests/fixtures/iec104/real/` must contain a `LICENSE.txt` crediting ITI (Idaho National Laboratory / Idaho Technology Inc.) and citing the CC BY 4.0 license URL, consistent with the attribution used in `tests/iec104_e2e_real_pcaps_tests.rs`. |
| F-W86S-P2-010 | MEDIUM | STORY-182 v1.1 AC-182-005 specifies failure behavior when the manifest file itself is absent ("fails with clear message"), but the v1.1 wording says "manifest file missing" without specifying whether the STORY-182 gate test panics or exits non-zero. Under `cargo test`, a panic and a failing `assert!` both produce a FAILED result, but the displayed messages differ. The AC must specify which failure mode is expected. | story-writer | FIXED in v1.2: AC-182-005 updated — when manifest file is absent, `fixture_manifest_all_present()` calls `panic!("fixture manifest missing: {path} — ensure tests/fixtures/iec104/manifest.txt is committed")`, which produces a FAILED test with the full message in CI output even under default stdout capture. |
| F-W86S-P2-012 | MEDIUM | STORY-182 v1.1 references `run_iec104_pipeline` as the integration test entry point, but does not specify whether this function returns a `Result` or panics on error. If it panics on file-not-found (absent fixture), the test fails with a panic traceback rather than a structured assertion failure, making the CI failure message unhelpful for diagnosing the fixture-absent scenario. | story-writer | FIXED in v1.2: AC-182-003 updated — `run_iec104_pipeline` invocation is wrapped in a `fixture_path()` resolver that returns `Option<PathBuf>`; if `None` (fixture absent), the test uses `assert!(fixture.is_some(), "fixture missing: {name}")` rather than passing `None` to `run_iec104_pipeline`. |
| F-W86S-P2-013 | MEDIUM | STORY-182 v1.1 gate-entry artifact task description is ambiguous: "gate-entry artifact" is not defined. From context it appears to mean an artifact produced during the wave gate (e.g., a `cargo test` run log showing the manifest gate test PASS), but this is not stated. Without a clear definition, the gate-entry check may be interpreted as a CI step, a manual verification, or a committed file. | story-writer | FIXED in v1.2: AC-182-007 added — "gate-entry artifact" is defined as a `cargo test --test fixture_manifest_all_present` run on the fixture-populated branch (i.e., `develop` after PR merge), with exit 0 and test output showing `test fixture_manifest_all_present ... ok`. This output must appear in the PR description test-evidence table. |
| F-W86S-P2-014 | MEDIUM | STORY-182 v1.1 does not specify the Rust module path for the `fixture_manifest_all_present` test. If it lives in `tests/fixture_manifest_tests.rs`, it requires a `[[test]]` entry in `Cargo.toml`. If it lives in `src/lib.rs` under `#[cfg(test)]`, it runs with `cargo test --lib`. The AC must specify the module location to avoid ambiguity in implementation. | story-writer | FIXED in v1.2: AC-182-004 updated — `fixture_manifest_all_present` lives in `tests/fixture_manifest_tests.rs` (integration test file), invoked as `cargo test --test fixture_manifest_tests`. No `Cargo.toml` `[[test]]` entry required (auto-discovered from `tests/` directory). |

### LOW

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P2-016 | LOW | STORY-182 v1.1 story title "Fixture Manifest + Committed Representative Captures: Gate False-Green cargo test in Clean Worktrees" is verbose (12 words before the colon). E-11 story titles in STORY-INDEX use concise STORY-NNN format. The title length will cause the STORY-INDEX wave-86 row to overflow the table column. | story-writer | FIXED in v1.2: Title shortened to "E2E Fixture Manifest + Committed ITI Captures: Gate False-Green in Clean Worktrees" (consistent with wave-86 scope label in D-516). |
| F-W86S-P2-017 | LOW | STORY-182 v1.1 does not specify the number of committed ITI capture files required. Committing 1 file satisfies "committed representative samples" literally, but the approach-c rationale in PG-W85-005 references "representative captures" implying coverage of multiple IEC-104 message types. A minimum count ensures the fixtures are non-trivially representative. | story-writer | FIXED in v1.2: AC-182-002 updated — minimum 3 committed capture files in `tests/fixtures/iec104/real/`, covering at least: (1) a Type I (single-command) packet, (2) a STARTDT activation packet, (3) a time-tagged measurement packet. Minimum count enforced by `fixture_manifest_all_present()` counting manifest entries. |
| F-W86S-P2-021 | LOW | STORY-182 v1.1 references `PG-W85-005` in the story body without noting the resolution status. At the time of wave-86 delivery, PG-W85-005 is RESOLVED (STORY-182 itself is the resolution vehicle per D-516/D-518). The story body should note its own closure of PG-W85-005 to make the audit trail complete. | story-writer | FIXED in v1.2: Story body updated — "This story resolves PG-W85-005 (wave-85 gate G1 gap: fixture-absent false-green). Resolution vehicle: STORY-182 per D-516." |

---

## Findings Not in STORY-182/183 Scope (Orchestrator-Routed)

| ID | Severity | Claim | Orchestrator Ruling | Disposition |
|----|----------|-------|---------------------|-------------|
| F-W86S-P2-019 | LOW | STORY-INDEX v3.97 wave-86 row shows `7 pts` (STORY-182: 4 pts, STORY-183: 3 pts) but pass-2 remediation raises STORY-183 to 5 pts (total 9 pts). The STORY-INDEX wave-table column `scheduled_pts` and the E-11 epic subtotal are stale. The STORY-INDEX body note at top says v3.97 but does not yet reflect the points increase. | state-manager (STORY-INDEX body currency) | FIXED: STORY-INDEX v3.97→v3.98 (state-manager D-518 burst): wave-table 707→709; E-11 73→75 pts; wave-86 row 7→9 pts; STORY-183 row 3→5 pts. |

---

## Summary

| Category | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 4 |
| MEDIUM | 10 |
| LOW | 7 |
| NITPICK | 1 |
| **Total** | **23** |

All 23 findings fixed in STORY-182 v1.2 and STORY-183 v1.2.
Clean streak after pass 2: **0/3**. Next: adversarial pass 3.
PO ruling DF-GREEN-DOC-TENSE-SWEEP v2→v3 committed to `policies.yaml` (F-W86S-P2-006).
F-007 [process-gap] from pass-1 subsumed by PG-W86-001 (see `process-gap-ledger.md`).
