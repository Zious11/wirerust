# PR #402 Review — STORY-168 (IEC-104 Frame Discrimination + Session SM, wave-77)

## VERDICT: APPROVE

Fresh-eyes review complete. All 8 checklist areas pass. No BLOCKING or MAJOR
findings — three NITs only, none merge-blocking.

## Checklist Results

### 1. Correctness — `classify_frame_format` (PASS)
`src/analyzer/iec104.rs:206`. `if cf1 & 0x01 == 0x00 → IFormat / else if cf1 & 0x03 == 0x01 → SFormat / else → UFormat`.
- Correct 2-bit discrimination: bit0=0 → IFormat; bits1:0=0b01 → SFormat; bits1:0=0b11 → UFormat.
- The `cf1 & 0x03 == 0x02` case is correctly absorbed by the I-format guard (`cf1 & 0x01 == 0x00`, bit0=0) → IFormat, per IEC 60870-5-104 §5.1. It cannot leak into S/U.
- if/else-if/else is exhaustive by construction — every u8 maps to exactly one variant, no unhandled case, no panic. Confirmed by the exhaustive-256 unit test and the VP-046 proptest.

### 2. Session SM — `process_u_frame` (PASS) — matches BC-2.19.010–014
- STARTDT-act (0x07) / STARTDT-con (0x0B): `session_started=true`, no finding; idempotent.
- STOPDT-act (0x13): `Verdict::Possible` if was_started else `Verdict::Likely`; `session_started=false`; Likely path appends `"STOPDT received without prior STARTDT on this flow"` (BC-2.19.012 PC3).
- STOPDT-con (0x23): `session_started=false`, no finding.
- TESTFR-act/con (0x43/0x83): no finding, state unchanged.
- Non-canonical `_`: T0814 `Anomaly/Possible`, state NOT advanced (fail-closed).

### 3. Test Completeness (PASS)
34 STORY-168 tests, distribution 5+4+7+3+2+3+3+6+1 verified against the file. EC-004 (idempotent STARTDT) @1023; EC-005 (cold STOPDT→Likely) @1144; non-canonical 0x0F, 0xFF, 0x03, 0x1B tested for T0814. Category assertions (`Impact`/`Anomaly`) present as regression guards.

### 4. ADR-013 Compliance (PASS)
- Decision 4: `classify_frame_format(cf1: u8) -> FrameFormat` takes only `u8`, never `&Iec104FlowState` — purity intact.
- Decision 5: exactly six canonical constants `{0x07, 0x0B, 0x13, 0x23, 0x43, 0x83}`.

### 5. CHANGELOG (PASS)
`[Unreleased] › Added` entry present, covers `classify_frame_format`, `process_u_frame`, `FrameFormat`, `Iec104FlowState::session_started`, VP-046 skeleton, T0881/T0814. Satisfies PG-W71-CHANGELOG (src/ trigger).

### 6. Row-Verify — PG-W74-PRDESC-ROW-VERIFY (PASS)
All 3 claimed entries confirmed at exact lines: AC-168-001 @701, AC-168-005 @1075, AC-168-008 @1357. Aggregate 30 (STORY-167) + 34 (STORY-168) = 64 matches CI `64 passed; 0 failed; 0 ignored`.

### 7. Diff Coherence (PASS)
Every changed file relates to STORY-168; no unrelated changes. Production delta 264 lines; large test file (882 lines) expected for strict-TDD.

### 8. Demo Evidence (PASS)
`evidence-report.md` + 6 per-AC files present, all 9 ACs covered, PG-W70-DEMO-SCRUB PASS. Pure-core library scope; VHS/Playwright correctly N/A.

## Findings (all NIT — non-blocking)

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| NIT | description | PR description Test Evidence table (line 95) labels AC-168-003 as "6 unit + 1 proptest"; the 7th test (`invariant_vp046_totality_exhaustive_all_256_values`) is a unit test, and the proptest belongs to AC-168-009. Per-AC distribution table (line 122) is correct. | Relabel AC-168-003 as "7 unit"; aggregate math unaffected. |
| NIT | coherence | `tests/iec104_analyzer_tests.proptest-regressions` commits seed `# shrinks to cf1 = 0` — a Red-Gate stub-failure artifact, not a genuine regression of shipped code. | Optionally prune; harmless if retained. |
| NIT (accepted-by-design) | correctness | `process_u_frame` `_` arm assumes U-format; a release-build mis-dispatch of a non-U CF1 would emit a T0814 whose text asserts "bits1:0=0b11". Guarded by `debug_assert!` and documented caller contract (dispatch after `classify_frame_format`==UFormat, STORY-173). | No action required. |

## Recommendation
APPROVE — merge after CI green.
