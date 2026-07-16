---
document_type: red-gate-log
level: ops
version: "1.0"
status: complete
producer: test-writer
timestamp: 2026-07-16T00:00:00Z
phase: f3
inputs:
  - .factory/stories/STORY-174.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.025.md
  - .factory/cycles/feature-iec104/research/story-174-scope-validation-followup.md
input-hash: "2d64e79"
traces_to: STORY-174
stub_architect_agent: N/A-hardening
stub_compile_verified: true
test_writer_agent: test-writer (wave-83)
red_gate_verified: true
---

# Red Gate Log: STORY-174 — IEC-104 Formal Hardening (wave-83)

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|---------------|-----------------|------|
| STORY-174 (hardening) | AC-174-008: 6 new known-bad fixtures in bin/test_check_green_doc_tense.py; AC-174-002: 2 VP-045 proptest harnesses upgraded from compile-only seams to asserting bodies | AC-174-008: YES (6/6 new fixtures fail — Red Gate for AC-174-008); AC-174-002: NO (expected-green re-verification — see Hardening-Mode Nuance below) | PASSED (hardening-mode semantics) |

## Hardening-Mode Context

STORY-174 is a formal-hardening story (phase: f3). The story's acceptance criteria
fall into three distinct categories with different Red Gate semantics:

1. **AC-174-008 (genuinely RED):** The green-doc-tense token extension requires new
   patterns in `bin/check-green-doc-tense` that do not yet exist. Known-bad fixtures
   for those patterns are RED by design until the implementer extends the tool.

2. **AC-174-002 (expected-GREEN re-verification):** The VP-045 proptest harnesses were
   compile-only seams (no assertions) shipped as STORY-172 scaffolding. Upgrading them
   to asserting harnesses and passing confirms that the already-delivered STORY-172
   implementation satisfies BC-2.19.025. These proptests are expected to pass — a
   failure here would indicate a real bug in STORY-172 code, not a Red Gate condition.

3. **AC-174-001/003/004/005/006/007 (verification-run ACs):** Kani proofs, fuzz runs,
   and cargo-mutants sweeps are verification-run acceptance criteria. They have no Red
   Gate state — they verify pre-existing implementations and are expected to pass once
   the implementer executes the harness runs.

## Stubs Created

None. STORY-174 is a hardening story with no new production code. All harness seams
were pre-created in STORY-167 (VP-044 Kani skeleton), STORY-168 (VP-046 proptest
skeleton), and STORY-172 (VP-045 proptest skeletons, VP-047 fuzz target). No new
`todo!()` stubs were introduced; this step is N/A for hardening stories.

Cargo check was run in the worktree and passes on the delivered STORY-172/173 code
prior to any modifications (verified by orchestrator dispatch).

## Red Gate Verification

### AC-174-008 (GENUINELY RED — Red Gate fires as required)

Known-bad fixtures added to `bin/test_check_green_doc_tense.py`:

| Fixture label | Pattern | Status |
|---------------|---------|--------|
| AC-174-008 pattern (a): All tests in this module MUST FAIL (interposed words) | `All tests\b.*\bMUST FAIL` | FAIL — gate did NOT flag expected violation |
| AC-174-008 pattern (a): All tests in this section MUST FAIL (section header variant) | `All tests\b.*\bMUST FAIL` | FAIL — gate did NOT flag expected violation |
| AC-174-008 pattern (b): FAILS Red Gate | `FAILS?\s+Red Gate` | FAIL — gate did NOT flag expected violation |
| AC-174-008 pattern (b): FAIL Red Gate (singular form) | `FAILS?\s+Red Gate` | FAIL — gate did NOT flag expected violation |
| AC-174-008 pattern (c): are todo!() stubs | `are\s+todo!\(\)\s+stub` | FAIL — gate did NOT flag expected violation |
| AC-174-008 pattern (c): are todo!() stub (singular) | `are\s+todo!\(\)\s+stub` | FAIL — gate did NOT flag expected violation |

Self-test runner output at commit `8cd55e6`:
```
Results: 66 passed, 6 failed.
```
Exit code: 1 (non-zero — Red Gate confirmed).

Known-good allowlist fixtures added (6 past-tense provenance cases): all PASS, zero
false positives. Proves patterns (a)-(c) do not regress against allowlisted prose.

**Implementer action required:** Extend `_VIOLATION_PATTERNS` in
`bin/check-green-doc-tense` with the three new regexes. After the extension,
`python3 bin/test_check_green_doc_tense.py` must exit 0.

### AC-174-002 (EXPECTED-GREEN re-verification — not a Red Gate failure)

Proptest harnesses upgraded in `tests/iec104_analyzer_tests.rs` (mod `story_172`):

| Harness | BC | Assertion added | Outcome |
|---------|----|-----------------|---------|
| `proptest_vp045_direction_isolation` | BC-2.19.025 invariant 1, invariant 3 | Interleaved direction-tagged-chunk generator; three-analyzer isolation-witness pattern; `prop_assert_eq!` on carry isolation; `prop_assert!` on MAX_IEC104_CARRY_BYTES bound | PASS (2/2) |
| `proptest_vp045_independent_run_equivalence` | BC-2.19.025 invariant 2 | `prop_assert_eq!` on `carry_c2s`, `carry_s2c`, `frame_count` across two independent analyzer instances | PASS (2/2) |

Cargo test output:
```
test story_172::proptest_vp045_independent_run_equivalence ... ok
test story_172::proptest_vp045_direction_isolation ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 196 filtered out; finished in 0.06s
```

No real bugs were exposed. The delivered STORY-172 implementation satisfies
BC-2.19.025 invariants 1, 2, and 3.

### AC-174-001, AC-174-003, AC-174-004, AC-174-005, AC-174-006, AC-174-007 (Verification-run ACs — no Red Gate state)

These ACs require executing Kani proofs, proptest runs, cargo-fuzz, cargo-mutants, and
cargo-kani re-runs. They verify pre-existing implementations; there is no stub or
compile-only seam to put in a Red state. They are expected to pass (or expose bugs)
when the implementer executes the harness runs.

**AC-174-005 harness-name note:** STORY-174 references `verify_dispatcher_oracle` as
the VP-004 harness name. The canonical VP-004 harnesses are:
- `verify_tls_signature_beats_port`
- `verify_content_first_precedence_exhaustive`
- `verify_none_two_phase_caching`

The story hedge "or equivalent VP-004 harness" applies. The implementer should run
all canonical VP-004 harnesses to satisfy AC-174-005.

## Regression Check

| Existing Tests | Status |
|----------------|--------|
| `cargo test --all-targets` — full suite (all integration + unit tests) | All pass: 0 failed across all test binaries |

The VP-045 proptest upgrades compile cleanly and do not disturb any existing tests.

## Hand-Off to Implementer

**Stories ready for implementation:** STORY-174

**Implementation work items (ordered by dependency):**

1. **AC-174-008** — Extend `bin/check-green-doc-tense` `_VIOLATION_PATTERNS` with:
   - `(a)` `re.compile(r"All tests\b.*\bMUST FAIL", re.IGNORECASE)`
   - `(b)` `re.compile(r"FAILS?\s+Red Gate", re.IGNORECASE)`
   - `(c)` `re.compile(r"are\s+todo!\(\)\s+stub", re.IGNORECASE)`
   Then scrub stale headers at `tests/iec104_analyzer_tests.rs` ~L662-663, ~L1498,
   ~L1544 to GREEN-accurate prose. Add `[Unreleased]` CHANGELOG entry (`bin/` touches
   trigger the changelog-gate). Run `python3 bin/test_check_green_doc_tense.py` → exit 0.

2. **AC-174-001** — Run `cargo kani --harness verify_parse_apci_header_safety` → green.

3. **AC-174-003** — Run `cargo test proptest_vp046` → green (already expected-pass from
   STORY-168 implementation; confirm here).

4. **AC-174-004** — Run `cargo fuzz run fuzz_iec104_parser -- -max_total_time=60` → no crashes.

5. **AC-174-005** — Run canonical VP-004 Kani harnesses (see harness-name note above) → green.

6. **AC-174-006** — Run `cargo kani --harness vp007_catalog_drift_guard` and
   `cargo kani --harness verify_all_emitted_ids_resolve` → green (SEEDED count=29).

7. **AC-174-007** — Run `cargo mutants -- --package wirerust` on `src/analyzer/iec104.rs`;
   triage surviving mutants; document in `cycles/wave-83/wave-gate/code-review.md`.

**Guidance:**
- Do not modify production code except for targeted bug fixes found during proof/fuzz runs.
- The VP-045 proptests at `tests/iec104_analyzer_tests.rs` are now asserting (STORY-174
  test-writer completed). No further test changes needed for AC-174-002.
- IEC104-FINDING-DIRECTION-001 is explicitly out of scope — do NOT implement it here.
- If Kani harness VP-044 times out, add `kani::assume` bounds per ADR-013 Decision 8
  (VP-044 scope is `parse_apci_header` ONLY — not `on_data`).
