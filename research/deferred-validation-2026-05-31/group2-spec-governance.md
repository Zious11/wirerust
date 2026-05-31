# Deferred-Item Validation — Group 2: Spec / Governance (LOW)

**Date:** 2026-05-31
**Branch target for fixes:** `factory-artifacts` (no `develop` PR)
**Validator:** vsdd-factory research-agent
**Scope:** Four LOW spec/governance deferred items, all confined to the `.factory/` artifact tree.

**Method note:** All four items are pure internal-consistency / anchor-accuracy
facts about files in this repository. Each was grounded by reading the actual
files (spec + source/test targets). Perplexity/Context7 were evaluated for
applicability per item; in all four cases the question is "does artifact X match
artifact Y in THIS repo," which has no external-convention dependency. See the
per-item "External research" line and the Research Methods section.

---

## F-S058-P12-O1 — BC-2.07.005 Architecture-Anchor off-by-one

**VALIDITY: CONFIRMED (real off-by-one).**

**Claim:** BC-2.07.005 Architecture-Anchor cites `src/analyzer/tls.rs:726-748`
but the actual target block is `726-747`.

**Grounding:**
- `BC-2.07.005.md` cites the `726-748` range in **three** places:
  - line 118 — `| Architecture Module | SS-07 (analyzer/tls.rs:726-748, C-13) |`
  - line 129 — Architecture Anchors: `src/analyzer/tls.rs:726-748 -- on_data buffer-append logic with remaining/to_copy cap`
  - line 141 — Source Evidence Path: `src/analyzer/tls.rs:726-748`
- Actual source (`src/analyzer/tls.rs`): the buffer-append block (the `{ ... }`
  scoped block containing the `remaining`/`to_copy` cap logic) opens at **line 726**
  (`{`) and closes at **line 747** (`}`). Line 748 is **blank**; line 749 is the
  `self.try_parse_records(...)` call. The cited block therefore ends at 747, not 748.

**Current vs correct:**
| Location | Current | Correct |
|----------|---------|---------|
| line 118 (Architecture Module) | `analyzer/tls.rs:726-748` | `analyzer/tls.rs:726-747` |
| line 129 (Architecture Anchors) | `src/analyzer/tls.rs:726-748` | `src/analyzer/tls.rs:726-747` |
| line 141 (Source Evidence Path) | `src/analyzer/tls.rs:726-748` | `src/analyzer/tls.rs:726-747` |

**Recommended fix:** Change all three `726-748` occurrences to `726-747`. (Note:
the off-by-one exists in 3 cells, not 1 — sweep all three in one burst per
DF-SIBLING-SWEEP-001, otherwise an adversarial pass will re-find the untouched cell.)

**External research:** None applicable — pure codebase fact (line-count of a
scoped block in this repo's source).

---

## F-W21-VP-METHOD — VP-018 / VP-019 proof_method frontmatter divergence

**VALIDITY: CONFIRMED (frontmatter diverges; same class as the Wave-21 VP-012/016/017 fix).**

**Claim:** VP-018 (cli.rs/SS-12) and VP-019 (dns.rs/SS-08) carry
`proof_method: manual` in frontmatter, diverging from VP-INDEX and from the VP
body and consuming-BC rows.

**Grounding — VP-018 (`vp-018-cli-reassemble-mutual-exclusion.md`):**
- Frontmatter line 15: `proof_method: manual`
- VP body (lines 57-61): Proof Method table = **"Integration test | assert_cmd / CLI test"**; Test Specification header comment `// tests/cli_tests.rs (using assert_cmd)`.
- `VP-INDEX.md`:
  - line 41 tool-count table groups VP-018 under **integration/unit**
  - line 67 catalog: Tool = **integration**, Phase = test-sufficient
  - line 101: "VP-018 | CLI test (assert_cmd): mutual exclusion exit code"
- `verification-coverage-matrix.md` line 34: VP-018 = **integration**.
- `verification-architecture.md` line 46: VP-018 = **integration test**.
- Consuming BC VP-table rows are **inconsistent with all of the above** — they say `unit:`:
  - `BC-2.12.007.md` line 82: `| VP-018 | ... | unit: Cli::try_parse_from(both flags) returns Err ...`
  - `BC-2.12.009.md` line 83: `| VP-018 | ... | unit: code-level verification (MEDIUM -- not directly tested) |`

**Grounding — VP-019 (`vp-019-dns-statistics-only.md`):**
- Frontmatter line 17: `proof_method: manual`
- VP body (lines 61-67): Proof Method table = **"Unit test | Rust test"**; "This is 'test sufficient' ... A single unit test confirms the invariant."
- `VP-INDEX.md`:
  - line 41 tool-count table groups VP-019 under **integration/unit**
  - line 68 catalog: Tool = **unit**, Phase = test-sufficient
  - line 102: "VP-019 | Unit test: empty Vec<Finding> assertion for all DNS packets"
- `verification-coverage-matrix.md` line 35: VP-019 = **unit**.
- `verification-architecture.md` line 47: VP-019 = **unit test**.
- Consuming BC VP-table rows (BC-2.08.001-004) **all use `unit:`** — already consistent with the index:
  - `BC-2.08.001.md` line 80, `BC-2.08.002.md` lines 80-81, `BC-2.08.003.md` lines 78-79, `BC-2.08.004.md` line 74 — all `unit:`.

**Wave-21 precedent confirmed:** `VP-INDEX.md` line 8 records the Wave-21
DF-SIBLING-SWEEP-001 harmonization that corrected VP-017 frontmatter
`manual→integration` (and confirmed VP-012 proptest). `vp-017-json-key-determinism.md`
frontmatter line 15 now reads `proof_method: integration`. So the corrective
pattern (frontmatter `manual` → the index/body method) is established and was
applied to the SS-11 reporter VP family but **not extended to VP-018/VP-019**.

**Current vs correct:**
| VP | Frontmatter now | Body says | VP-INDEX / matrix / arch | Consuming BC rows | Correct `proof_method` |
|----|-----------------|-----------|--------------------------|-------------------|------------------------|
| VP-018 | `manual` | integration | integration | `unit` (BC-2.12.007/009) | **`integration`** |
| VP-019 | `manual` | unit | unit | `unit` (BC-2.08.001-004) | **`unit`** |

**Recommended fix:**
1. `vp-018-...md` frontmatter line 15: `manual` → `integration`.
2. `vp-019-...md` frontmatter line 17: `manual` → `unit`.
3. **Secondary divergence to sweep in the same burst (VP-018 only):** the two
   consuming BC rows for VP-018 (`BC-2.12.007.md:82`, `BC-2.12.009.md:83`) say
   `unit:` but the authoritative method is `integration`. Harmonize those two
   rows `unit:` → `integration:` (mirrors exactly what Wave-21 did for the
   VP-017 BC rows). VP-019's BC rows already say `unit:` and need no change.
4. Add a `modified:` frontmatter entry to each VP file documenting the correction
   (the VP-017 fix did this).

**External research:** None applicable — this is an internal-consistency fix.
The "manual / integration / unit" values are this project's own VP proof-method
taxonomy (defined in VP-INDEX + verification-architecture.md), not an external
standard, and the authoritative value for each VP is already unambiguously fixed
by four other in-repo artifacts. No Perplexity/Context7 lookup was warranted.

---

## F-W22-BC-ANCHOR — SS-11 reporter BC Architecture-Anchor staleness

**VALIDITY: CONFIRMED that the anchors cite `tests/reporter_tests.rs` + pre-formalization
test names — but the claim's sub-assertion "that file no longer exists" is MISCHARACTERIZED.**
The file `tests/reporter_tests.rs` STILL EXISTS and STILL CONTAINS the cited test
names. The real defect is that the anchors point at the *legacy* test file/names
instead of the *authoritative per-story formalization* tests produced in Waves 20-22.
The re-anchoring is still warranted; the justification is "stale-pointing," not "dangling."

**Grounding — which BCs cite the legacy file:**
`tests/reporter_tests.rs` is cited in the Architecture-Anchor (or VP/test-name)
section of **11** of the 24 SS-11 BCs:
`BC-2.11.004, .005, .006, .007, .011, .012, .013, .014, .015, .016, .017`.
(The other 13 SS-11 BCs do not cite that file in an anchor; several — e.g.
BC-2.11.001 — anchor only to `src/reporter/json.rs` source lines and cite test
*names* in their VP tables.) So the affected scope is **11 of 24**, not all 24.

Example stale anchors (file + old test name, all real lines):
| BC | Line | Stale anchor |
|----|------|--------------|
| BC-2.11.004 | 100 | `tests/reporter_tests.rs -- test_json_reporter_preserves_cyrillic_as_readable_unicode` |
| BC-2.11.005 | 107 | `tests/reporter_tests.rs -- test_http_finding_c1_csi_in_json_reporter` |
| BC-2.11.006 | 101 | `tests/reporter_tests.rs -- test_terminal_reporter_shows_skipped_when_nonzero, ...hides_skipped_when_zero` |
| BC-2.11.007 | 126 | `tests/reporter_tests.rs -- test_terminal_reporter_escapes_esc_bytes_in_summary` |
| BC-2.11.013 | 107 | `tests/reporter_tests.rs -- mitre_grouping_emits_tactic_headers_in_canonical_order` |

**Grounding — file-existence and current test layout:**
- `tests/reporter_tests.rs` — **EXISTS** (not deleted). It still defines the cited
  legacy tests, e.g. `test_terminal_reporter_shows_skipped_when_nonzero` (line 517),
  `test_terminal_reporter_escapes_esc_bytes_in_summary` (line 550),
  `test_json_reporter_preserves_cyrillic_as_readable_unicode` (line 671),
  `test_http_finding_c1_csi_in_json_reporter` (line 862),
  `mitre_grouping_emits_tactic_headers_in_canonical_order` (line 972). So the claim
  "old test names that no longer exist" is inaccurate as literally stated.
- The Wave 20-22 brownfield-formalization split produced three per-surface files
  that ARE the authoritative formalization homes:
  - `tests/reporter_json_tests.rs` — top-level BC-prefixed fns under PG-W17-001
    naming (e.g. `test_BC_2_11_001_top_level_keys`, `test_BC_2_11_002_...`,
    `test_BC_2_11_003_...`). Covers BC-2.11.001..005.
  - `tests/reporter_terminal_tests.rs` — `mod story_077 { ... }`, `mod story_078 { ... }`
    with BC-prefixed fns (e.g. `test_BC_2_11_006_skipped_packets_zero_no_line`,
    `test_BC_2_11_007_esc_byte_escaped`, `test_BC_2_11_013_tactic_headers_in_canonical_order`).
    Covers BC-2.11.006..019 (terminal surface).
  - `tests/reporter_csv_tests.rs` — `mod story_079 { ... }`, `mod story_080 { ... }`
    with BC-prefixed fns (e.g. `test_BC_2_11_020_header_row_first_and_exact`,
    `test_BC_2_11_021_neutralize_all_six_trigger_chars`). Covers BC-2.11.020..024.

**Recommended fix (re-anchoring map):**
For each of the 11 affected BCs, replace the `tests/reporter_tests.rs -- <old_name>`
anchor with the corresponding per-story formalization test:
- BC-2.11.004, .005 (JSON surface) → `tests/reporter_json_tests.rs -- test_BC_2_11_00N_<...>`
- BC-2.11.006, .007, .011, .012, .013, .014, .015, .016, .017 (terminal surface)
  → `tests/reporter_terminal_tests.rs -- mod story_077|story_078 :: test_BC_2_11_0NN_<...>`
- (CSV BCs .020-.024 are not in the affected set; they were authored post-split
  and already anchor to `reporter_csv_tests.rs`.)

Map each old name to its BC-prefixed successor by BC number (e.g. BC-2.11.007's
`test_terminal_reporter_escapes_esc_bytes_in_summary` → `test_BC_2_11_007_esc_byte_escaped`
in `reporter_terminal_tests.rs` mod story_077). This is a bulk-mechanical sweep
across 11 files.

**Decision needed (flag to owner):** whether `tests/reporter_tests.rs` (the legacy
pre-split file that still compiles and runs) should be *retired* as part of this fix
or left in place. If it is retired later, any anchor still pointing at it becomes a
true dangling reference — re-anchoring now pre-empts that.

**External research:** None applicable — pure codebase fact (which file/test names
exist in this repo's `tests/` tree, and which BC files cite them).

---

## FSR-row staleness — STORY-086 / 087 / 096 cite `tests/cli_tests.rs`

**VALIDITY: CONFIRMED (stories cite the wrong test file; recurrence is real and
already tracked). Root-cause flag: WORTH RAISING — story-template / authoring
discipline gap.**

**Claim:** STORY-086/087/096 Token-Budget / Files-Touched ("FSR") rows cite
`tests/cli_tests.rs`, but the actual formalization tests live in
`cli_story_086_tests.rs` / `_087_` / `_096_` per DF-TEST-NAMESPACE-001.

**Grounding — story citations:**
| Story | Token-Budget row | Files-Touched row |
|-------|------------------|-------------------|
| STORY-086 | line 128: `tests/cli_tests.rs (existing tests)` | line 174: `tests/cli_tests.rs | modify | Add AC-001..AC-010 ...` |
| STORY-087 | line 133: `tests/cli_tests.rs` | line 179: `tests/cli_tests.rs | modify | Add AC-001..AC-012 ...` |
| STORY-096 | line 126: `tests/cli_tests.rs` | line 173: `tests/cli_tests.rs | modify | Add AC-001..AC-010 ...` |

**Grounding — actual formalization homes:**
- `tests/cli_story_086_tests.rs` — `mod story_086 { ... }` with AC/BC-aligned fns
  (e.g. `test_analyze_subcommand_basic_parse`, `test_analyze_requires_at_least_one_target`).
- `tests/cli_story_087_tests.rs` and `tests/cli_story_096_tests.rs` exist as the
  per-story namespaced homes (same pattern).
- `tests/cli_tests.rs` and `tests/cli_integration_tests.rs` also exist (legacy /
  cross-cutting), but the STORY-086/087/096 formalization ACs were delivered into
  the `cli_story_NNN_tests.rs` files per the DF-TEST-NAMESPACE-001 per-story
  `mod story_NNN` convention (the same collision-avoidance pattern PR #146 applied
  to HTTP tests; see lessons.md PG-W18-003 on the flat-vs-per-story namespace risk).

**Current vs correct:**
| Story | Current citation | Correct citation |
|-------|------------------|------------------|
| STORY-086 | `tests/cli_tests.rs` (2 rows) | `tests/cli_story_086_tests.rs` |
| STORY-087 | `tests/cli_tests.rs` (2 rows) | `tests/cli_story_087_tests.rs` |
| STORY-096 | `tests/cli_tests.rs` (2 rows) | `tests/cli_story_096_tests.rs` |

**Recommended fix:** In each of the three story files, update both the Token-Budget
row and the Files-Touched row to cite the per-story `cli_story_NNN_tests.rs` file.
6 row-edits total across 3 files. (Per-story sweep: update both rows in a single
burst per story so an adversarial pass doesn't re-find the untouched row.)

**Recurrence / root-cause assessment (FLAG):**
- Recurrence is confirmed and already logged: `STATE.md` line 109 records this as
  "FSR-row staleness ... 3 occurrences; deferred optional batch-cleanup; recorded
  lessons.md W24.L4." Independent prior FSR-discipline lessons exist (lessons.md
  ~L570-574 brownfield-formalization FSR-declaration rule; ~L795-800 PG-W18-002
  test-citation sweep checklist).
- **Root cause is template/authoring-side, not a one-off typo.** The CLI story
  template seeds the Token-Budget and Files-Touched rows with the generic
  `tests/cli_tests.rs` placeholder, which authors carry through unchanged even
  though delivery routes the tests into `cli_story_NNN_tests.rs` per
  DF-TEST-NAMESPACE-001. Because the placeholder is template-seeded, every new CLI
  story reproduces the drift — hence 3 recurrences (086, 087, 096).
- **Recommendation worth filing (subject to DF-VALIDATION-001 — this report is that
  validation):** Update the CLI story template so the test-file placeholder is the
  per-story `tests/cli_story_<NNN>_tests.rs` form (or add an explicit authoring
  checklist item under DF-TEST-NAMESPACE-001 requiring FSR test-file rows to use the
  per-story namespace). This converts a recurring per-story cleanup into a
  one-time template fix.

**External research:** None applicable — pure codebase fact + this repo's own
DF-TEST-NAMESPACE-001 convention.

---

## Summary table

| Item | Validity | Affected artifacts | Fix size | External research |
|------|----------|--------------------|----------|-------------------|
| F-S058-P12-O1 | CONFIRMED | BC-2.07.005 (3 cells) | 3 edits, 1 file | None — codebase fact |
| F-W21-VP-METHOD | CONFIRMED | vp-018, vp-019 frontmatter (+ BC-2.12.007/009 rows for VP-018) | 2 frontmatter + 2 BC-row edits | None — internal consistency |
| F-W22-BC-ANCHOR | CONFIRMED (one sub-claim mischaracterized: legacy file still exists) | 11 of 24 SS-11 BCs | ~11 files, bulk-mechanical | None — codebase fact |
| FSR-row staleness | CONFIRMED + root-cause flag | STORY-086/087/096 (6 rows) + CLI story template | 6 edits + 1 template fix | None — codebase fact |

**Cross-cutting note:** Three of the four items (F-S058-P12-O1, F-W21-VP-METHOD,
FSR-row staleness) each have a *secondary location* that must be swept in the same
burst as the primary fix (3rd cell / consuming-BC rows / 2nd story row), consistent
with the DF-SIBLING-SWEEP-001 lessons. Apply each fix as a single multi-location
burst to avoid the documented "adversary re-finds the untouched location" cascade.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Read | 9 | BC-2.07.005, VP-018/019, VP-INDEX, VP-017, BC-2.11.001, BC-2.12.007, src/analyzer/tls.rs, reporter_tests.rs |
| Grep | 13 | locate anchor/test citations across SS-11 BCs, VP-018/019 consumers, story FSR rows, test-file structures, STATE.md/lessons.md drift records |
| Glob | 11 | enumerate VP files, SS-11 BCs, reporter/cli test files, story files |
| Perplexity search | 0 | not applicable — see per-item notes |
| Perplexity reason | 0 | not applicable |
| Perplexity deep_research | 0 | not applicable |
| Context7 | 0 | not applicable — no external library/API question |
| Tavily (all) | 0 | not applicable |
| WebFetch / WebSearch | 0 | not applicable |
| Training data | 0 areas | every claim grounded in a read of the actual file |

**Total MCP tool calls:** 0 (intentional — see below)
**Training data reliance:** low — no claim rests on model knowledge; each is a
direct file-content fact verified by Read/Grep against this repository.

**Why zero external research is correct here:** All four items are internal
consistency / anchor-accuracy facts (does in-repo artifact X match in-repo
artifact Y, and does a cited line range / file / test name match the actual
target). Per the task instruction, Perplexity/Context7 were considered for each
and rejected: the "VP proof-method taxonomy" in F-W21-VP-METHOD is this project's
own (manual/integration/unit defined in VP-INDEX + verification-architecture.md),
not an external standard, and the authoritative value was already fixed
unambiguously by four other in-repo artifacts. No genuine external-convention
question arose. This is flagged explicitly per the "no external research
applicable — pure codebase fact" guidance.
