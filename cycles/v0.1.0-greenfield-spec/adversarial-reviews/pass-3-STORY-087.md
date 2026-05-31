# Adversarial Review — STORY-087 (Implementation) — Pass 3

| Field | Value |
|-------|-------|
| Target | implementation (test formalization) |
| Scope | STORY-087 — `tests/cli_story_087_tests.rs` + traceability to BC-2.12.004/005/007, VP-018, BC-INDEX, VP-INDEX |
| Strategy under review | brownfield-formalization (zero src changes) |
| Artifacts read | STORY-087.md; BC-2.12.004/005/007; tests/cli_story_087_tests.rs; src/cli.rs; BC-INDEX.md; VP-INDEX.md; vp-018; policies.yaml |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 3 |
| Date | 2026-05-31 |
| HEAD | d9f91bc |
| Verdict | **CLEAN** (no Critical/High/Medium/Low) |

## Method

Fresh attack angles distinct from Pass 2: (a) BC-clause coverage gaps — which
postconditions/edge cases have NO test; (b) mutation resistance — would each test
catch a plausible implementation bug; (c) traceability integrity across
BC-INDEX, VP-INDEX, and VP-018; (d) the MEDIUM-confidence posture of BC-2.12.007.

## Coverage-Gap Analysis (BC clause → test)

Enumerated every BC postcondition/invariant/edge-case and checked for a test:

| BC clause | Covered? | Notes |
|-----------|----------|-------|
| BC-2.12.004 PC-1..PC-4, all EC | YES | AC-001..004 |
| BC-2.12.005 PC-3 (depth=10) | YES | AC-005 |
| BC-2.12.005 PC-4 (memcap=1024) | YES | AC-006 |
| BC-2.12.005 PC-5 (thresholds None/Some) | YES | AC-007 |
| BC-2.12.005 PC-6 (overlap 0-255) | YES | AC-008 (256 reject), EC-003 (255 accept) |
| BC-2.12.005 PC-7 (small_segment_threshold 0-2048) | **NO upper-bound test** | not a story AC — see Observation O-1 |
| BC-2.12.005 PC-8 (small_segment_max_bytes 0-2048) | partial | EC-002 covers 0; upper bound 2049 untested — not a story AC |
| BC-2.12.005 Inv-3 (ignore-ports CSV Vec<u16>) | YES | AC-009 |
| BC-2.12.007 PC-1, Inv-1, EC-001..005 | YES | AC-010/011/012 |

**Observation O-1 (non-finding):** PC-7 and PC-8 upper-bound *rejection* paths
(`--small-segment-threshold 2049`, `--small-segment-max-bytes 2049`) and the
`out_of_window_threshold` Some-value path are not exercised. These are NOT story
ACs — the STORY-087 AC set deliberately traces to PC-3/4/5/6 and Inv-3 only.
PC-7/PC-8 upper bounds are BC-level invariants the story chose not to elevate to
ACs. The test file fully and faithfully covers its declared AC/EC contract; this
is a scoping decision in the story, not a defect in the test file under review.
`out_of_window_threshold` has no clap range validator in cli.rs (plain
`Option<u32>`), consistent with BC-2.12.005 listing ranges only for
overlap/small-segment-threshold/max-bytes — so there is no range behavior to test
there. Recorded as an observation; no severity assigned.

## Mutation-Resistance Spot-Checks

| Plausible bug | Caught by | Result |
|---------------|-----------|--------|
| depth/memcap defaults swapped (10↔1024) | AC-005 `assert_ne!(depth,1024)` + AC-006 `assert_ne!(memcap,10)` | CAUGHT |
| OutputFormat variant confusion (json↔csv) | AC-001/002 `assert_ne!` against sibling variant | CAUGHT |
| conflicts_with removed | AC-010/011 assert `ArgumentConflict` kind specifically | CAUGHT |
| conflict declared one-directional only | AC-011 reversed-order test | CAUGHT |
| `Some(0)` collapsed to `None` for max-bytes | EC-002 distinguishes `Some(0)` from None | CAUGHT |
| ignore-ports parsed as single value not split | AC-009 asserts len==2 and element order | CAUGHT |
| overlap range off-by-one (rejects 255) | EC-003 asserts 255 accepted | CAUGHT |
| overlap range too wide (accepts 256) | AC-008 asserts 256 → ValueValidation | CAUGHT |

Negative assertions are discriminating; no test would pass under the listed
mutations. No tautological or vacuous assertions found.

## Traceability Integrity

- **BC-INDEX.md:** BC-2.12.004/005/007 all `[WRITTEN]`, P0, origins
  BC-CLI-004/005/007. No mis-trace.
- **VP-INDEX.md / VP-018:** VP-018 → BC-2.12.007, status `test-sufficient`. The
  BC-2.12.007 VP table specifies proof method "unit: `Cli::try_parse_from`(both
  flags) returns Err with ArgumentConflict kind" — exactly what AC-010/011 do.
  VP-INDEX line 101's alternate assert_cmd exit-code phrasing is a
  proof-*option*, not a contradiction; the unit-parse method satisfies the VP.
- **Story frontmatter:** `behavioral_contracts: [004,005,007]` and
  `verification_properties: [VP-018]` match the body BC table and AC traces.

## Policy Rubric Compliance

- **DF-TEST-NAMESPACE-001 — CLEAN.** 16 tests inside `mod story_087`; zero
  flat-root functions.
- **DF-AC-TEST-NAME-SYNC-001 — CLEAN.** All 12 AC `**Test:**` citations resolve
  to exactly one in-suite `fn test_*`. No ambiguity, no line-anchor errors.
- **DF-TEST-CITATION-SWEEP-001 — CLEAN.** No live test-name citation outside the
  test file; no sweep target left unfixed.
- **DF-SIBLING-SWEEP-001 / DF-VALIDATION-001 — N/A** in scope.

## Findings

None. No Critical/High/Medium/Low. One non-severity coverage observation (O-1),
explained as story-scoping, not a test defect.

## Trajectory

Pass 1: 2 Low (fixed) → Pass 2: 1 Low (non-blocking parity note) → Pass 3: 0.
Monotonic decrease maintained.

## Verdict

**CLEAN.** First fully-clean pass (zero findings of any severity). Coverage,
mutation-resistance, and traceability all hold.
