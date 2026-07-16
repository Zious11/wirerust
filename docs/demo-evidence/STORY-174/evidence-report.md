# STORY-174 Demo Evidence Report

**Story:** STORY-174 — IEC-104 Formal Hardening: VP-044 Kani + VP-045/046 Proptest + VP-047 Fuzz + VP-004/007 Re-run + cargo-mutants  
**Wave:** 83  
**Date recorded:** 2026-07-16  
**Worktree HEAD:** e62701f (CONVERGED)

All 8 ACs verified green on the feature branch.

---

## AC-174-001: VP-044 Kani — parse_apci_header safety

| Field | Value |
|-------|-------|
| Artifact | `ac-174-001-kani-parse-apci-header.txt` |
| Command | `cargo kani --harness verify_parse_apci_header_safety` |
| Outcome | VERIFICATION SUCCESSFUL — 0 of 89 checks failed |
| Notes | Five facets confirmed: len<6→None, start≠0x68→None, LEN<4→None, LEN>253→None, valid→Some |

---

## AC-174-002: VP-045 proptest — carry direction isolation (non-vacuous)

| Field | Value |
|-------|-------|
| Artifact | `ac-174-002-proptest-vp045.txt` |
| Command | `cargo test --test iec104_analyzer_tests proptest_vp045` |
| Outcome | 2 passed (proptest_vp045_direction_isolation, proptest_vp045_independent_run_equivalence) |
| Notes | Both asserting harnesses with prop_assert! on post-on_data state. Interleaved C2S/S2C chunks; carry isolation and <=255 bound verified. |

---

## AC-174-003: VP-046 proptest — classify_frame_format totality

| Field | Value |
|-------|-------|
| Artifact | `ac-174-003-proptest-vp046.txt` |
| Command | `cargo test --test iec104_analyzer_tests proptest_vp046` |
| Outcome | 1 passed (proptest_vp046_frame_format_totality) |
| Notes | Exhaustive sweep over all 256 u8 CF1 values; bit patterns IFormat/SFormat/UFormat all correct |

---

## AC-174-004: VP-047 fuzz — fuzz_iec104_parser 60s no crashes

| Field | Value |
|-------|-------|
| Artifact | `ac-174-004-fuzz-iec104-parser.txt` |
| Command | `cargo +nightly fuzz run fuzz_iec104_parser -- -max_total_time=60` |
| Outcome | DONE — 618,615 runs in 61 seconds, 0 crashes, 0 panics |
| Notes | cov: 450, ft: 1436, corp: 289/83Kb. on_data loop termination confirmed for all arbitrary byte sequences. |

---

## AC-174-005: VP-004 Kani re-run — dispatcher oracle with Iec104

| Field | Value |
|-------|-------|
| Artifact | `ac-174-005-kani-dispatcher-oracle.txt` |
| Command | `cargo kani --harness verify_content_first_precedence_exhaustive` |
| Outcome | VERIFICATION SUCCESSFUL — 0 of 440 failed (10 unreachable) |
| Notes | DispatchTarget::Iec104 Rule 8 arm covered. ADR-013 Decision 9 obligation satisfied. |

---

## AC-174-006: VP-007 Kani re-run — T0881 MITRE catalog integrity

| Field | Value |
|-------|-------|
| Artifact | `ac-174-006-kani-vp007-catalog.txt` |
| Command (a) | `cargo kani --harness verify_all_emitted_ids_resolve` |
| Outcome (a) | VERIFICATION SUCCESSFUL — 0 of 122 failed (1 unreachable) |
| Command (b) | `cargo test --lib vp007_catalog_drift_guard` |
| Outcome (b) | 1 passed (SEEDED count=29) |
| Notes | T0881 resolves correctly via technique_name/technique_tactic. ADR-013 Decision 10 satisfied. |

---

## AC-174-007: cargo-mutants sweep — mutation score >= 80%

| Field | Value |
|-------|-------|
| Artifact | `ac-174-007-mutants-disposition.txt` |
| Evidence source | `target/mutants-disposition.md` + targeted tests |
| Command | `cargo test --test iec104_analyzer_tests test_AC_174_007` |
| Outcome | 2 passed; kill rate 117/122 = 95.9% (>= 80% requirement) |
| Notes | 28 acceptable survivals in #[cfg(kani)] module (compiled out in normal builds); 5 production equivalent mutants documented in mutants-disposition.md. |

---

## AC-174-008: Stale Red-Gate comment guard extended + baseline scrubbed

| Field | Value |
|-------|-------|
| Artifact | `ac-174-008-green-doc-tense.txt` |
| Command (a) | `python3 bin/test_check_green_doc_tense.py` |
| Outcome (a) | 72 passed, 0 failed (including 6 new AC-174-008 pattern cases) |
| Command (b) | `python3 bin/check-green-doc-tense` |
| Outcome (b) | PASS: 112 files scanned, 0 stale headers |
| Notes | Three new patterns added: `All tests\b.*\bMUST FAIL`, `FAILS?\s+Red Gate`, `are\s+todo!\(\)\s+stub`. Stale headers at tests/iec104_analyzer_tests.rs ~L662-663, ~L1498, ~L1544 scrubbed. |

---

## Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate command (per `.factory/maintenance/demo-evidence-scrub-gate.md`) run before commit:
grep for absolute host path patterns across `docs/demo-evidence/STORY-174/`.

Result: **zero results** — gate PASSED.

All absolute host paths have been replaced with `<repo>/` placeholders in transcript files.
