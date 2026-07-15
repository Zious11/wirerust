# PR #403 Review — STORY-169 IEC-104 ASDU header extraction (wave-78)

**Verdict: APPROVE**

Fresh-eyes review against `origin/develop` (b720fd9, already contains STORY-167/168).
True PR scope: 9 files, +1709 lines — `src/analyzer/iec104.rs` (+173),
`tests/iec104_analyzer_tests.rs` (+734), `CHANGELOG.md` (+17), 6 demo-evidence `.md` files.
No BLOCKING and no MAJOR findings.

## What I verified

**1. Correctness — all 9 field extractions match the BC specs exactly (`src/analyzer/iec104.rs:560-606`):**
- BC-2.19.015: `if asdu_body.len() < 6 { return None; }` — guard is `< 6` (not `< 10`). Correct.
- BC-2.19.016: `type_id = asdu_body[0]`; `sq = (asdu_body[1] & 0x80) != 0`; `count = asdu_body[1] & 0x7F`. Correct.
- BC-2.19.017: `cot_cause = asdu_body[2] & 0x3F`; `cot_pn = (asdu_body[2] & 0x40) != 0`;
  `cot_test = (asdu_body[2] & 0x80) != 0`; `cot_originator = asdu_body[3]`. Correct.
- BC-2.19.018: `casdu = u16::from_le_bytes([asdu_body[4], asdu_body[5]])`;
  `first_ioa = Some(u32::from_le_bytes([b6, b7, b8, 0]))` gated on `count > 0 && asdu_body.len() >= 9`,
  else `None`. Correct — 24-bit LE zero-extended.

**2. Test completeness — 27 STORY-169 tests, all pass.** Full file 91/91 (30 story_167 + 34 story_168
+ 27 story_169), exactly matching the evidence-report claim (row-verify + aggregate-count cross-check
per PG-W74-PRDESC-ROW-VERIFY: confirmed). EC-001..EC-008 all explicitly traced and covered; all four BC
postconditions covered; boundary vectors present (count=127 max, CASDU=0/65535, IOA=0xFFFFFF,
6/7/8/9-byte bodies). LE byte-order test (`[0x34,0x12,0x00]` → `Some(0x1234)`) correctly distinguishes
LE from BE.

**3. Purity — confirmed.** `parse_asdu(&[u8]) -> Option<Asdu>` is a pure free fn: no `self`, no I/O,
no finding emission, no state mutation. Determinism test and no-panic-on-short-lengths test present.

**4. Code quality — high.** Doc comments accurate and consistent with implementation. Pass-1 commit
(0debf98) removed the stale `todo!()` Red-Gate docstring. `cargo clippy --all-targets -- -D warnings`
clean; `cargo fmt --check` clean.

**5. CHANGELOG — present and accurate.** `[Unreleased] > Added` entry lists all 9 fields, the `< 6`
guard, LE extractions, and `first_ioa` eligibility. Satisfies changelog-gate (src/ modified).

**6. Architecture compliance — satisfied.** ADR-013 Decision 3: all 9 fields broken out; no packed
`vsq: u8` / `cot: u16`. Decision 8: correctly documented as VP-047 cargo-fuzz target (no-panic),
explicitly not a VP-044 Kani target.

## Findings

| # | Severity | Location | Finding | Recommendation |
|---|----------|----------|---------|----------------|
| 1 | MINOR (informational) | `docs/demo-evidence/STORY-169/*` | Demo evidence is markdown test-transcript form, not `.gif`/`.webm`. For a pure-core library free function with no CLI/web/interactive surface (effectful caller is STORY-170), a recording is not meaningful; matches merged STORY-167/168 convention. Not blocking. | Accept as-is for library product type. |
| 2 | NIT (out-of-scope) | `src/analyzer/iec104.rs:21` | Module docstring says `Iec104ParseError` is "extended in STORY-168," but the enum still has only the `Incomplete` variant. Line predates this PR (not in STORY-169 diff), so not a STORY-169 defect. | Fix opportunistically in a later IEC-104 story. |

No BLOCKING and no MAJOR findings. Implementation is correct against every BC postcondition, fully
tested including all documented edge cases and purity/no-panic properties, lint- and format-clean, and
architecturally compliant. Approving for merge.
