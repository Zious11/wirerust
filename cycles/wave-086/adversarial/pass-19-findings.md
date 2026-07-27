---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 19
verdict: NOT_CONVERGED
novelty: "moderate"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 19

**Date:** 2026-07-27
**Pass:** 19 of N

# WAVE-86 STORY-LEVEL ADVERSARIAL REVIEW — PASS 19

## Phase A — Attestation

**A0. Tool-profile disclosure (DF-ADVERSARY-TOOLCHAIN-PAIRING-001).** My dispatched tool profile is `read-only` — `Read`, `Grep`, `Glob` only. `Bash` is **denied**. I therefore **cannot execute** `git rev-parse`, `git branch`, `git ls-files`, `grep -c`, `cargo`, or `python3`. Every A1–A3 assertion below is satisfied either (a) from the SUPPLIED EXECUTION EVIDENCE in my dispatch, or (b) by an **independent Read/Grep/Glob corroboration** I performed myself. I flag which is which. No execution outcome is asserted beyond the supplied evidence.

**A1. Branch / freshness.**
- Cannot run `git rev-parse HEAD` / `git branch --show-current` (Bash denied).
- Dispatch-embedded environment block states: `Current branch: develop`, `Recent commits: e8841d76 Merge pull request #441 from Zious11/main` (HEAD).
- Supplied evidence header states develop @ `e8841d76`.
- **Attestation line:** *Reviewing develop at SHA e8841d761f3f25f320f98977618e506e8b41a058 (v0.13.2 back-merge).*
- Confirmed: no story worktree exists for this pass; STORY-182 and STORY-183 are unimplemented drafts. Review target = develop tree + the two `.factory/stories/` files.
- **Independent corroboration that the tree is the post-v0.13.2 develop tree:** `tests/iec104_e2e_real_pcaps_tests.rs:340` reads `Expected: 66 findings = T0836 ×20 + T1692.001 ×46` and `:343-350` carries the `Wave-85 change (STORY-180, BC-2.19.029/030)` 31→66 block. The pre-gate-fix (31-finding) state is absent. This is the post-`0ab6f52e` tree.

**A2. Story-version assertion** (own Grep, `.factory/stories/STORY-18{2,3}.md`):
```
.factory/stories/STORY-182.md:5:version: "2.8"    :6:status: draft   :12:points: 4   :38:input-hash: "9a0f34c"
.factory/stories/STORY-183.md:5:version: "2.8"    :6:status: draft   :12:points: 5   :40:input-hash: "9c9b12f"
```
Both **2.8** — matches EXPECTED. No abort. Stored input-hashes match the supplied canonical Python values (`9a0f34c` / `9c9b12f`) → **no input-hash drift reported** (DF-INPUT-HASH-CANONICAL-001).

**A3. Grep-count assertion — independently re-derived, not taken on faith.**

| Assertion | Expected | My own result | Method |
|---|---|---|---|
| `fixture_present` in `tests/iec104_e2e_real_pcaps_tests.rs` | 5 | **5** (`:63` def, `:166`, `:292`, `:383`, `:529` calls) | Grep content mode, line-numbered |
| `fixture_present` in `tests/enip_e2e_real_pcaps_tests.rs` | 7 | **7** | Grep count mode |
| `bin/*.py` file cardinality | 6 | **6** — `test_lint_cycle_artifact.py`, `test_compute_input_hash.py`, `test_changelog_gate_content.py`, `test_validate_citations.py`, `test_check_green_doc_tense.py`, `test_gitignore_mutants_glob.py` | Glob `bin/*.py` |

All three match. **No methodology-suspect divergence.** Filesystem queried: `/Users/zious/Documents/GITHUB/wirerust` (main repo working tree, `develop` checkout).

**A4. Factory-artifact path confirmation.** Both story files were read from the main-repo path `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md` and `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md`. No worktree-relative path was used. All supporting artifacts were read from absolute main-repo paths (DF-ADVERSARY-METHODOLOGY-001 — no `cd` was used; `cd` is impossible for me, Bash being denied).

**A5. Output-destination deviation (must be recorded).** I was asked to write to `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-086/adversarial/pass-19-findings.md`. My profile denies `Write`/`Edit`, and my standing instructions forbid emitting report `.md` files. **Findings are returned as this message text; the orchestrator must persist them via state-manager to that path.**

---

## Methodology

Read in full: both story files (1389 / 1291 lines). Read or grep-verified: `bin/check-green-doc-tense` (597 lines — docstring :1-204, `_VIOLATION_PATTERNS` :217-457, `_is_comment_line` :460-462, `_collect_rust_files` :465-499, `scan_file` :502-524, `_find_repo_root` :527-546, `main` :549-596); `bin/test_check_green_doc_tense.py` (914 lines); `bin/test_lint_cycle_artifact.py` (:1-20, :118-132, all `def test_tc*`); `bin/changelog-gate-check`; `bin/fetch-e2e-pcaps` :145-169; `.github/workflows/ci.yml` :25-64 and :425-544; `tests/iec104_e2e_real_pcaps_tests.rs` :1-145, :336-396; `tests/fixtures/README.md` (all 75 lines); `tests/fixtures/E2E-PCAPS.md` :1-60, :330-401; `tests/e2e_corpus_smoke_tests.rs` :203-226; `.gitignore`; `.factory/policies.yaml` (all 19 policy IDs; `DF-GREEN-DOC-TENSE-SWEEP` read in full :1045-1229); `.factory/cycles/wave-085/STORY-180/convergence-report.md` :55-79; `.factory/stories/STORY-INDEX.md` (STORY-182/183 rows + frontmatter totals).

**Claims I independently verified as TRUE** (a non-exhaustive list, recorded because the truthfulness axis is the highest-value axis and this pass found the stories unusually accurate):

- STORY-183 "`_VIOLATION_PATTERNS` (line 217) contains **28 tuples**, labeled Pattern 1 through Pattern 29 … docstring token list has 37 items / 36 tuples — item 5 shares tuple 4": docstring items 1–29 at `:31-85`; tuple 4 at `:230-233` (`RED GATE:.*tests must fail`) subsumes docstring items 4 **and** 5; 28 tuples. Post-story 29+8 = **37 items**, 28+8 = **36 tuples**. Arithmetic exact.
- STORY-183 "~3625 Rust attribute lines": my Grep of `^\s*#\[|^\s*#!\[` over all Rust files returns **3625** across 127 files. **Exact.**
- STORY-183 "40 `//`-line fixtures + 2 `#` lines at 258/261 = 42 violations": BAD_CASES entries counted via `^    \($` restricted to :52–:319 = **40**; total multi-line fixtures 87, `//`-content lines 85 (2 GOOD non-comment cases). Independent regex evaluation of every `#` line in `bin/*.py` against all 28 shipped patterns: **exactly 2 match** — `:258` (Pattern 26 fires on the quoted `"harness skeleton compiles"`) and `:261` (Pattern 29 fires on `"fails until wired"`). Candidate lines `:170`, `:211`, `:213`, `:214`, `:256`, `:259`, `:260` all provably do **not** match. **42 is exact.**
- STORY-183 rename arithmetic: `_collect_rust_files` occurs exactly **13** times in `bin/test_check_green_doc_tense.py` (`:688,699,705,707,711,718,726,839,843,859,872,891,905`) plus one `rust_files` prose site at `:721` (`"check the \`if not rust_files:\` guard in main()]"`) = **14**. Exact.
- STORY-183 all monkey-patch anchors: AC-158-005 block `:698-726` (patch `_find_repo_root` at `:704`), AC-162-003 block `:858-905` (patch at `:871`), `finally` ends `:905`, `print()` at `:907`. **All exact.** `import tempfile` at `:640` function-scope — exact. `BAD_CASES` annotation at `:51` — exact. Quote-bearing BAD fixtures at `:91`/`:97`; `:402` is a GOOD_CASE non-comment line — exact.
- STORY-183 AC-183-008's 10 `falls through to` sites: my Grep returns **exactly** `src/analyzer/tls.rs:930`, `tests/bc_2_16_d078_lax_malformed_tests.rs:18,90,216`, `tests/bc_2_16_d078b_lax_some_arm_tests.rs:335`, `tests/main_story_089_tests.rs:890`, `tests/dnp3_f6_story140_group_a_survivors.rs:812,850`, `tests/bc_f6_mutation_gap_tests.rs:791,793`. **All 10, no extras.**
- STORY-183 zero-FP claim: case-insensitive Grep of all 8 new TIER-1 tokens across all Rust files **and** all of `bin/` → **0 hits**. Top-level `src/*.rs` (exactly 10 files: decoder, reader, lib, findings, summary, dispatcher, main, mitre, protocols, cli) → **0** `//`-comment hits against the shipped pattern set. Post-story `python3 bin/check-green-doc-tense` can exit 0.
- STORY-183 convergence-report citation: `:63-66` is F-180-P1-003 naming `currently asserts` + `is expected to`; `:68-70` is the PG-W85-003 paragraph carrying the broader `"Expected RED:"` / `"currently falls through"` labels. **Exact, including the "primary citation" nuance.**
- STORY-183 `bin/test_lint_cycle_artifact.py` scrub loci: `:3` `— RED GATE version.`, `:5` `TC1–TC8 implement all eight test cases…`, `:6` `All tests MUST FAIL until bin/lint-cycle-artifact is created.`, `:125` `# Test cases (TC1–TC8)`. **All exact.** 21 `def test_tc*` functions labelled TC1–TC21 → the prescribed `TC1–TC21 … 21 self-tests` replacement is correct. All three AC-183-009 grep predicates are genuinely discriminating (current counts 1/1/2 → required 0/0/0).
- STORY-182 all `tests/iec104_e2e_real_pcaps_tests.rs` anchors: `:10-13`, `:23-28`, `:39`, `:47-49`, `:51`, `:53-57`, `:59-62`, `:63`, `:90`, `:97`, `:138`, `:273`, `:353-354`, `:503-504`, `:529`. **All exact.** All four `FIXTURE_GATED_TESTS` test names match `fn test_*` definitions at `:165`, `:291`, `:382`, `:528` **verbatim** (DF-AC-TEST-NAME-SYNC-001 satisfied; module-qualified + `--exact` throughout).
- STORY-182 needle discipline: `fixture_present("` occurs exactly 4× (`:166,292,383,529`) = `FIXTURE_GATED_TESTS.len()`. I re-derived every prescribed comment/assertion line in AC-182-001/005 and Task 7 — **zero** contiguous occurrences of the needle. Non-tautological.
- STORY-182 fixture-count anchors: `tests/fixtures/` holds exactly **25** capture files (`.pcap|.pcapng|.cap|.trace`); `tests/fixtures/local-samples/` absent; `.gitignore:10` covers only `/tests/fixtures/local-samples/`; `mutants.out*/` at `:12`; `coverage-out.txt` **not** present → Task 10b is a real deliverable.
- STORY-182 provenance data: E2E-PCAPS.md `:358` (14 KB, sha `07b9a087…`), `:359` (11 KB, sha `292c18a8…`), `:337-340`, `:352-359`, `:374-380`, `:391-396`, `:3-6`, `:48-50` — all exact. Wireshark pair shas at `bin/fetch-e2e-pcaps:154,157` — **exact**. README `:7-22` notice body / `:24-26` malware clause / `:30-34` provenance table / `:41-44` not-recorded clause — all exact. TypeIDs 58/59/61/63 and the 66 = T0836×20 + T1692.001×46 decomposition confirmed at `tests/iec104_e2e_real_pcaps_tests.rs:343-350`.
- ci.yml anchors `:47`, `:434`, `:442`, `:462`, `:463`, `:473`, `:533` — all exact; changelog trigger regex `^(src/|Cargo\.toml$|bin/)` at `:524` confirms STORY-182's "no CHANGELOG required" and STORY-183's "CHANGELOG required".
- No fixtures-directory-enumeration test exists that a newly committed pcap would break (checked all `read_dir` sites in `tests/`). No `docs/`, `README.md`, or `CLAUDE.md` reference to `check-green-doc-tense` exists → STORY-183's sweep surface is correctly bounded.
- Semantic anchoring: STORY-INDEX `:296`/`:297` titles are **byte-identical** to the story H1s; points 4/5; wave 86; status draft; v2.8; `:455` wave row `2 stories / 9 pts`. Clean.

---

## Findings

| ID | Sev | Story | Locus | One-line |
|---|---|---|---|---|
| F-W86S-P19-001 | MEDIUM | 182 | Task 7 bullet 2 (`:1004-1006`) → `tests/iec104_e2e_real_pcaps_tests.rs:11-12` | Module-docstring false-green claim left unswept while its byte-adjacent sibling at `:62` is explicitly called out as false |
| F-W86S-P19-002 | MEDIUM | 182 | AC-182-002 Verification (`:437-447`), Task 1 (`:856-932`) | Two multi-command verification blocks lack `set -euo pipefail`; the "non-waivable" ≤100 KB size gate is maskable |
| F-W86S-P19-003 | MEDIUM | 182 | Task 8 (`:1091-1100`), Task 10a (`:1174`) | Wave-gate N/M obligation is self-contradictory and mis-fires on fixture-bearing hosts; text lands permanently in CLAUDE.md |
| F-W86S-P19-004 | MEDIUM | 182 | ACs vs Tasks 7/8/10a/10b | 4 of 9 declared file deliverables have no acceptance criterion; all 5 ACs can pass with them omitted |
| F-W86S-P19-005 | MEDIUM | 183 | Task 9 (`:1044-1049`) vs `bin/check-green-doc-tense:558-562` | Empty-collection negative assertion is non-discriminating: the message it greps for goes to **stderr**, never captured |
| F-W86S-P19-006 | MEDIUM | 183 | Task 10 bullet 1 (`:1060-1065`) / Task 2 (`:896-902`) vs `bin/check-green-doc-tense:4`, `:467`, `:472` | "test files"→"source files" rename prescribed at `:577`+ci.yml but not at the tool's headline scope line `:4` |
| F-W86S-P19-007 | LOW | 183 | Task 4 (`:936`) | Prescribed `:261` replacement still contains `until`…`wired` on one line; escapes Pattern 29 only via an unstated `\b`-boundary accident |
| F-W86S-P19-008 | LOW | 183 | Task 9 (`:1011-1013`) | Python scoping rationale is inverted (function-scope import shadows module-level, not the reverse) |
| F-W86S-P19-009 | LOW | 183 | Task 2 (`:913-919`) vs `CHANGELOG.md:851` | CHANGELOG-preservation instruction enumerates 1 of 2 historical loci affected by the rename |
| F-W86S-P19-010 | NIT | 183 | AC-183-001 (`:209-212`) vs Task 2 (`:907-912`) | Rename-site categorisation inconsistent (`6 functional + 7 prose + 1` vs `6 functional + 8 prose`) |

---

### F-W86S-P19-001 — MEDIUM — STORY-182 — unswept false-green claim in the module docstring

**Locus.** Story: Task 7 bullet 2, `STORY-182.md:1004-1006`. Target: `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs:11-12`.

**Defect.** Task 7 gives the module docstring a *narrow* instruction — rewrite only the location sentence:

> `tests/iec104_e2e_real_pcaps_tests.rs` module docstring (lines 10–13): update "Captures live in `tests/fixtures/local-samples/`" to acknowledge `tests/fixtures/` (committed) and `tests/fixtures/local-samples/` (gitignored corpus).

But the same docstring range contains the exact false-green assertion this story exists to eliminate:

```
11 //! that directory is absent or a specific fixture file is missing, the affected test prints a
12 //! skip notice and returns immediately. This keeps CI green without fixtures while still
13 //! failing loudly (assertion-level) when fixtures are present. `#[ignore]` is NOT used.
```

Two bullets later, Task 7 handles the **byte-adjacent sibling** of that identical claim with an explicit falsity verdict and a full rewrite prescription (`STORY-182.md:1014-1021`):

> `tests/iec104_e2e_real_pcaps_tests.rs` lines :59-62 … the current text says "present in local-samples … keeps CI green when gitignored local-samples not populated" — **this is false post-story**. … Rewrite to committed-first wording …

Source `:62` reads `/// This keeps CI green when the gitignored local-samples directory is not populated.` — the same claim, same file, ~50 lines apart. Post-story the module-level claim is **strictly false for the committed partition**: `iec104-iti-diverse.pcap` is always present, and AC-182-005's hard-assert makes its absence a *CI-red*, not a green skip. Leaving `:11-12` intact ships the most-read prose in the file asserting the very semantics AC-182-004/005 abolish.

**Why it matters (not cosmetic).** `:11-13` is the file's contract statement for future test authors. An author reading "This keeps CI green without fixtures" is being told the silent-skip idiom is the sanctioned design — exactly the PG-W85-005 recurrence vector STORY-182 is chartered to close. This is a DF-SIBLING-SWEEP-001 asymmetry, not a wording nit.

**Prescribed fix.** Extend Task 7 bullet 2 to prescribe the docstring rewrite explicitly, mirroring the `:59-62` prescription:

> `tests/iec104_e2e_real_pcaps_tests.rs` module docstring (lines 10–13): (a) update "Captures live in `tests/fixtures/local-samples/`" to acknowledge `tests/fixtures/` (committed) and `tests/fixtures/local-samples/` (gitignored corpus); (b) **the claim at `:12` — "This keeps CI green without fixtures" — is FALSE post-story for the committed partition and MUST be rewritten**: committed captures in `tests/fixtures/` always run in CI, and their absence is a hard-assert CI failure via `test_fixture_manifest_report()`; only the gitignored corpus partition skips (advisory, `--nocapture`-visible). Keep the `#[ignore]` is NOT used sentence at `:13` (still true — see Background §`#[ignore]` rejection).

Also add `:10-13` to the File Structure Requirements Notes cell for this file (`STORY-182.md:1286`), which currently enumerates `:47-49`, `:53-57`, `:59-62` but describes the module docstring only as "update module docstring".

---

### F-W86S-P19-002 — MEDIUM — STORY-182 — two verification blocks lack `set -euo pipefail`; the "non-waivable" size gate is maskable

**Locus.** `STORY-182.md:437-447` (AC-182-002 Verification) and `STORY-182.md:856-932` (Task 1 Steps 1a–1g).

**Defect.** AC-182-002's Verification block, verbatim, opens with no shell hardening:

```bash
git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap
# (comment)
test "$(wc -c <"tests/fixtures/iec104-iti-diverse.pcap")" -le 102400
# (comment)
test "$(shasum -a 256 tests/fixtures/iec104-iti-diverse.pcap | cut -d' ' -f1)" = "07b9a087…"
```

Without `set -e`, a block's exit status is that of its **last** command. So if `wc -c` reports 150000 (size gate FAILS) but the sha256 matches, the block exits **0**. The same is true of `git ls-files --error-unmatch` failing. This directly defeats the story's own hard claim at `:429-434`:

> **Size gate (hard):** … The size constraint is not waivable.

Task 1 (`:856-932`) is worse: ~15 sequenced commands (`mkdir -p`, four conditional curl+shasum gates, Step 1d size `test`, Steps 1e/1g sha `test`s) with **no** `set -euo pipefail`. Step 1d's size failure cannot stop Steps 1e–1g, and the block's status is Step 1g's.

**Sibling-sweep evidence (this is a partial-fix regression, not a fresh oversight).** The v2.5/v2.6/v2.7 changelog rows record `set -o pipefail` → `set -euo pipefail` hardening applied to **seven** sibling blocks: F-W86S-P15-003 (three blocks: AC-182-003, AC-182-004, Task 9 Env A), F-W86S-P16-001 (Task 9 Env A again), F-W86S-P16-002 (ACR ci.yml step), P17-001, P17-002 (five Env-B blocks). Present state confirms hardening at `:356`, `:363`, `:488`, `:541`, `:548`, `:561`, `:1116`, `:1134`, `:1155`, `:1162`, `:1260`. The **two** un-hardened blocks are precisely the two that carry the size/integrity gates — the highest-consequence predicates in the story.

**Prescribed fix.** Prepend `set -euo pipefail` as the first line of both blocks:

```bash
# AC-182-002 Verification
set -euo pipefail
git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap
test "$(wc -c <"tests/fixtures/iec104-iti-diverse.pcap")" -le 102400
test "$(shasum -a 256 tests/fixtures/iec104-iti-diverse.pcap | cut -d' ' -f1)" = "07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7"
```

For Task 1, insert `set -euo pipefail` immediately before Step 1a's `mkdir -p`. Note the existing `|| { echo …; rm …; exit 1; }` guards inside Steps 1b/1c/1f remain correct under `-e` (the `||` suppresses `-e` for the tested command), and `! shasum … | grep -q "$SHA"` inside `if` conditions is likewise `-e`-exempt — so adding `-euo pipefail` is safe and changes no intended control flow.

---

### F-W86S-P19-003 — MEDIUM — STORY-182 — wave-gate N/M obligation is self-contradictory and mis-fires on the environment where the gate actually runs

**Locus.** `STORY-182.md:1091-1100` (Task 8, "Enforceable wave-gate obligation (F-026)") and `STORY-182.md:1174` (Task 10a, the row that lands **permanently in CLAUDE.md**).

**Defect (a) — internal contradiction.** Task 8 says, two sentences apart:

> "… M MUST equal 4 (the literal) and N MUST equal `COMMITTED_FIXTURES.len()` (currently 1) in a clean checkout; **any other N/M pair blocks gate entry pending investigation.**"

then

> "… — **the recorded N/M is the evidence artifact, not an independent gate.**"

These are mutually exclusive: either the N/M pair blocks gate entry (it is a gate) or it is only evidence (it is not a gate). The Task 10a CLAUDE.md row reproduces **both** claims in a single cell.

**Defect (b) — the predicate mis-fires in the real gate environment.** The predicate is qualified "in a clean checkout", but no expected value is stated for the other environment, and the blocking clause is unqualified. On a fixture-bearing host — which the story itself establishes is where wave gates are actually run — `local-samples/` is present, all four fixtures resolve, and the printed value is `4/4`. `N=4 ≠ COMMITTED_FIXTURES.len()=1`, so `4/4` is "any other N/M pair" and **blocks gate entry**. The story's own Task 9 Env A (`:1136`) *requires* `Fixture coverage: 4/4` as the pass condition for that environment. Directly self-contradictory.

That the wave gate runs on fixture-bearing hosts is not speculation — the story states it twice: `:1087-1088` ("initial FAIL was on a fixture-bearing host") and `:1356-1359` ("D-510 was triggered on a **fixture-bearing host**").

**Novelty note.** This is *induced by the pass-18 remediation*: the v2.8 changelog row records "P18-006 MED (Task 8 `:1064-1065`: replaced tautology 'M MUST equal FIXTURE_MANIFEST.len()' with discriminating predicate 'M MUST equal 4 (the literal) and N MUST equal COMMITTED_FIXTURES.len() (currently 1)…')". The tautology was correctly removed; the replacement introduced an environment-blind blocking rule and did not retire the "not an independent gate" sentence it now contradicts.

**Prescribed fix.** Rewrite the obligation in both loci to be environment-explicit and to state exactly one enforcement posture:

> Before G1 of any wave-gate evaluation that includes e2e pcap tests, run
> `cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture`
> and record the printed `N/M` in the gate entry. **M MUST equal 4 (the literal).** **N MUST equal 1 (`COMMITTED_FIXTURES.len()`) when `tests/fixtures/local-samples/` is absent, or 4 (`FIXTURE_MANIFEST.len()`) when it is fully populated;** an intermediate N is legitimate on a partially-populated host and is recorded as-is. **Any `M ≠ 4`, or any `N < 1`, blocks gate entry pending investigation.** Absence of a committed-partition member from `tests/fixtures/` is caught independently by the AC-182-005 hard-assert (`cargo test` failure) — that assert, not the recorded N/M, is the blocking gate; the recorded N/M is the evidence artifact.

Apply the identical wording to the CLAUDE.md row at `:1174`, and add the corrected wording to the Task 8 gate-entry artifact bullet list. Delete the now-redundant "any other N/M pair" phrasing entirely.

---

### F-W86S-P19-004 — MEDIUM — STORY-182 — four of nine declared file deliverables have no acceptance criterion

**Locus.** ACs AC-182-001…005 (`STORY-182.md:209-811`) vs Tasks 7, 8, 10a, 10b (`:995-1075`, `:1076-1100`, `:1169-1182`) and the Architecture Mapping / File Structure Requirements tables (`:813-828`, `:1282-1293`).

**Defect.** The story declares nine modified/created artifacts. Mapping each to an AC:

| Deliverable | AC coverage |
|---|---|
| `tests/iec104_e2e_real_pcaps_tests.rs` code | AC-182-001/003/005 ✓ |
| `tests/fixtures/iec104-iti-diverse.pcap` | AC-182-002 ✓ |
| `tests/fixtures/README.md` provenance row | AC-182-002 ✓ |
| `.github/workflows/ci.yml` additive step | AC-182-004(e) ✓ |
| `tests/fixtures/E2E-PCAPS.md` (6-locus sweep) | **none** |
| `tests/iec104_e2e_real_pcaps_tests.rs` prose sweep (`:10-13,:23-28,:47-49,:53-57,:59-62,:90,:353-354`) | **none** |
| `.factory/maintenance/fixture-count-gate-entry.md` (new) | **none** |
| `CLAUDE.md` Project References row | **none** |
| `.gitignore` `coverage-out.txt` entry | **none** |

Every "Then" clause across the five ACs is satisfiable by a PR that touches only the test file's code, the pcap, README, and ci.yml. A delivery that skipped the entire E2E-PCAPS.md sweep, never created the gate-entry artifact, never added the CLAUDE.md row, and never gitignored `coverage-out.txt` would **pass all five ACs**. `.gitignore` is the sharpest instance: Background §Gitignore (`:160-163`) states the entry "**IS required**" and that the file "must not be accidentally committed" — but nothing verifiable enforces it, so the CI-written `coverage-out.txt` can be committed by accident on the very PR that introduces it.

This is an *acceptance-criteria completeness* defect, not a task-list defect: the Tasks are fully specified; they are simply unverifiable at gate time.

**Prescribed fix.** Add one acceptance criterion covering the four uncovered deliverables, with mechanically checkable "Then" clauses. Suggested AC-182-006:

```
### AC-182-006 (traces to PG-W85-005 — governance-surface completeness)

- Then `tests/fixtures/E2E-PCAPS.md` records iec104-iti-diverse.pcap as committed and
  iec104-iti-dissect.pcap as gitignored at all six swept loci:
    set -euo pipefail
    test "$(grep -c 'tests/fixtures/' tests/fixtures/E2E-PCAPS.md)" -ge 1
    grep -q 'iec104-iti-diverse.pcap' tests/fixtures/README.md
- And `grep -qF 'coverage-out.txt' .gitignore`
- And `grep -qF '.factory/maintenance/fixture-count-gate-entry.md' CLAUDE.md`
- And `test -f .factory/maintenance/fixture-count-gate-entry.md`  # factory-artifacts branch
- And no doc-comment in tests/iec104_e2e_real_pcaps_tests.rs still asserts the
  local-samples-only fixture root:
    test "$(grep -c 'keeps CI green' tests/iec104_e2e_real_pcaps_tests.rs)" -eq 0
```

(The last predicate also mechanically closes F-W86S-P19-001 — it currently returns 2 and must return 0.)

---

### F-W86S-P19-005 — MEDIUM — STORY-183 — Task 9's empty-collection negative assertion cannot fail as specified (stderr is never captured)

**Locus.** Story: `STORY-183.md:1044-1049`. Target behaviour: `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense:557-563`.

**Defect.** Task 9 prescribes, verbatim:

> - If the process output does NOT contain "no tracked source files found" (verifies exit-1 was a genuine violation, not the empty-collection guard in `main()` at `:557-563` — both paths exit 1, so this negative assertion distinguishes them): print `  PASS  [hermetic-e2e: not empty-collection exit]` …

The message it greps for is written to **stderr**, not stdout:

```python
556    rust_files = _collect_rust_files(repo_root)
557    if not rust_files:
558        print(
559            "ERROR: no tracked Rust files found; scan target may be wrong. "
560            "Verify the scan target in bin/check-green-doc-tense.",
561            file=sys.stderr,
562        )
563        return 1
```

Meanwhile the *other* two Task-9 assertions inspect stdout — the violation report is `print(f"FAIL [{rel}:{lineno}]: {label}")` at `:570`, unredirected. The story never states which stream(s) to capture. The natural implementation (`subprocess.run(..., capture_output=True)` then inspecting `proc.stdout`, consistent with the third assertion) makes the negative assertion **vacuously true on every run**: `"no tracked source files found" not in proc.stdout` is always true because that string can never appear on stdout.

**Consequence.** The single assertion whose entire purpose is to distinguish a genuine-violation exit-1 from a false-green empty-collection exit-1 becomes a no-op. The failure mode it guards against is real and live: if the merged pathspec `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py` were mistyped, or if `git add bin/violating.py` silently failed, the hermetic test would collect **zero** files, `main()` would exit 1 via the empty-collection guard, the exit-code assertion would print PASS, and the negative assertion would also print PASS. Only the third assertion (output names `violating.py`) would fail — and if an implementer regressed that check too, the whole hermetic e2e test would be a false green. This is precisely the POL-11 / CI-as-Code false-green class.

**Novelty note.** Induced by pass-18: the v2.8 changelog records "P18-007 LOW (Task 9 negative assertion added …)". The assertion was added; the stream it must read was not specified.

**Prescribed fix.** Make the capture explicit in Task 9, and drop the ambiguous word "output":

```
- Run the copy with both streams captured:
    proc = subprocess.run([sys.executable, str(tmp/"bin"/"check-green-doc-tense")],
                          capture_output=True, text=True)
    combined = proc.stdout + proc.stderr
- If proc.returncode == 1: print "  PASS  [hermetic-e2e: exit 1 on violation]" …
- If "no tracked source files found" NOT in `combined` (note: main() writes this
  message to STDERR via file=sys.stderr at bin/check-green-doc-tense:558-562 — the
  assertion is vacuous if only proc.stdout is inspected): print
  "  PASS  [hermetic-e2e: not empty-collection exit]" …
- If a FAIL line naming `bin/violating.py` plus the pattern label appears in
  proc.stdout (violation reports go to STDOUT, bin/check-green-doc-tense:570-571):
  print "  PASS  [hermetic-e2e: output names violating.py]" …
```

Additionally add a fourth, positively-discriminating assertion so the trio is not solely negative: assert `len(_collect_source_files(tmp)) == 1` inside the hermetic repo.

---

### F-W86S-P19-006 — MEDIUM — STORY-183 — "test files" → "source files" rename prescribed at `:577` but not at the tool's headline scope declaration `:4`

**Locus.** Story: Task 10 bullet 1 (`STORY-183.md:1060-1065`), Task 10 final tool bullet (`:1099-1102`), Task 2 rename-propagation bullet (`:896-902`). Target: `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense`.

**Defect.** The story prescribes the noun-phrase rename explicitly for `:577` and for ci.yml, and *only* the glob-list update for `:4`. Complete census of scope-noun loci in the tool (my Grep):

```
  4: Scans tracked test files (tests/*.rs and src/**/*.rs cfg(test) modules) for
 12: During strict TDD, test files are written before implementation with module-
467:     Collect test-scope Rust files:
468:       - tests/*.rs    (integration test files)
472:     excluded so newly-added test files do not cause false failures before
559:             "ERROR: no tracked Rust files found; scan target may be wrong. "
577:             f"Found {total_violations} stale RED-phase comment(s) in test files "
```

Story dispositions:
- `:559` — **explicitly** prescribed (Task 2: → `"no tracked source files found"`) ✓
- `:577`, `:581-585` — **explicitly** prescribed (Task 10: "update scope descriptions from 'test files' to 'source files'") ✓
- ci.yml `:434`, `:462` — **explicitly** prescribed ("in test files" → "in source files") ✓
- `:4` — prescribed for **globs only** ("update the scope declaration from 'tests/*.rs and src/**/*.rs' to enumerate all four globs"). The stale noun phrase `Scans tracked **test files**` and the now-false parenthetical `cfg(test) modules` are **not** addressed.
- `:467` `Collect test-scope Rust files:` — Task 2 says "update scope descriptions to enumerate the full pathspec"; ambiguous whether the noun phrase is in scope.
- `:472` `so newly-added **test files** do not cause false failures` — **not addressed**.
- `:12` — legitimately unchanged (problem-pattern narration, not scope).

`:4` is the single most load-bearing sentence in the tool: it is the first line of the module description, the string a maintainer reads to learn what the gate covers. Post-story the gate scans **10 top-level `src/*.rs` production files** (`src/main.rs`, `src/cli.rs`, `src/decoder.rs`, …) and **6 `bin/*.py` files** — none of which are "test files" or "`cfg(test)` modules". Shipping `:4` unchanged leaves the tool's own headline documentation materially false while the story edits the very same sentence.

This is the same failure shape prior passes have flagged twice in this story (P17-006, P18-001/P18-002 all concern Task 10 bullets that fixed one prose locus and mis-scoped its siblings) — a recurrence of the pattern class, at a locus not previously enumerated.

**Prescribed fix.** Amend Task 10 bullet 1 to cover both the glob list and the noun phrase, and add the two unswept siblings:

```
- `bin/check-green-doc-tense` line :4 (module docstring headline scope text): rewrite the
  WHOLE sentence, not only the glob list. Current: "Scans tracked test files (tests/*.rs and
  src/**/*.rs cfg(test) modules) for". Required: "Scans tracked source files (tests/*.rs,
  src/**/*.rs, src/*.rs, and bin/*.py) for" — the parenthetical "cfg(test) modules" is FALSE
  post-story: 10 top-level src/*.rs production files and 6 bin/*.py files are now in scope and
  are not test modules.
- `bin/check-green-doc-tense` line :467 (`_collect_*` docstring first line): "Collect
  test-scope Rust files:" -> "Collect scanned source files:"; add the src/*.rs and bin/*.py
  bullets alongside the existing tests/*.rs and src/**/*.rs bullets.
- `bin/check-green-doc-tense` line :472: "so newly-added test files do not cause false
  failures" -> "so newly-added source files do not cause false failures".
```

Also update the FSR Notes cell for `bin/check-green-doc-tense` (`STORY-183.md:1198`), which currently says only "update module docstring scope text and token list".

---

### F-W86S-P19-007 — LOW — STORY-183 — Task 4's prescribed `:261` replacement survives Pattern 29 only by an unstated regex-boundary accident *(pending intent verification)*

**Locus.** `STORY-183.md:935-936` (Task 4, line ~261 prescription).

**Defect.** Task 4 states the safety rule as a *phrase* rule:

> The comments MUST NOT contain the literal flagged phrase at all (removing quotes is insufficient)

and then prescribes:

```
#   (d) \buntil\b[^\n]*\bwired\b   — CI-wiring-incomplete prose (pattern 29)
```

Pattern 29 is not a phrase pattern. It is `re.compile(r"\buntil\b.*\bwired\b(?!\s+(?:it|the|a|that|this|them)\b)", re.IGNORECASE)` (`bin/check-green-doc-tense:455`) — it fires on **any** line containing `until` … `wired`. The prescribed replacement contains both words on one line.

It escapes only because the literal `\b` prefixes make the surrounding text `…b`+`until` and `…b`+`wired`: the regex's `\b` assertion requires a word boundary immediately before `u` and before `w`, and in both cases the preceding character is the word-character `b`. No boundary → no match. I verified this by hand against the exact byte sequence, and the same accident is what saves the untouched sibling at `:213` (`#   (a) All tests\b.*\bMUST FAIL` escapes Pattern 23 `All tests\b.*\bMUST FAIL` because `\bMUST` sees `b`+`M`).

**Why this is a real risk, not pedantry.** The story states one criterion (no literal phrase) while the prescribed line satisfies a *different*, unstated criterion (no regex match, via boundary suppression). Any benign edit that removes the regex escapes reintroduces the violation and turns the delivery red. Concretely, the "cleaner" rewrite an implementer might reach for —

```
#   (d) until … wired   — CI-wiring-incomplete prose (pattern 29)
```

— **does match** Pattern 29 (`until` bounded by spaces, `wired` bounded by spaces, lookahead sees a space+`—`), breaking AC-183-002's hard zero-FP requirement. The story's own changelog records this exact hazard class recurring five times (v2.2 row: "HIGH = 5th self-referential-predicate recurrence"; D-529 imposed a standing no-literal-phrase discipline), so the mechanism deserves to be written down rather than left implicit.

Two further siblings in the same comment blocks are safe-but-unexplained and should be recorded so a future editor does not "fix" them into violations: `:259` (`#   (b) compile-only\s+seams?  — "compile-only seam(s)" present-tense` — quotes a literal token, escapes only because Pattern 27 requires an `exposes|is a|are` prefix) and `:213` above.

**Prescribed fix.** In Task 4, replace the phrase-level rule with a match-level rule and record the mechanism:

> **Safety criterion (not "no literal phrase" — that is insufficient for regex patterns like 29 and 23):** the rewritten line MUST NOT *match* any of the 36 patterns. Verify mechanically after rewording with `python3 bin/check-green-doc-tense`.
> The prescribed replacements are safe by a specific, deliberate mechanism: writing the pattern in regex-literal form places the escape character `\b` immediately before the trigger word, so the text reads `…buntil` / `…bwired` / `…bMUST` and Pattern 29's / Pattern 23's `\b` assertion cannot fire (the preceding `b` is a word character). **Do NOT "clean up" these lines by removing the `\b` escapes** — `#  (d) until … wired` DOES match Pattern 29 and will fail the gate.
> Sibling lines `:213` and `:259` in the same comment blocks are already safe by the same mechanism (`:213`) or by Pattern 27's required verb prefix (`:259`) and are deliberately left unchanged.

---

### F-W86S-P19-008 — LOW — STORY-183 — inverted Python-scoping rationale in Task 9's import note

**Locus.** `STORY-183.md:1011-1013`.

**Defect.** The story states:

> Note: `tempfile` is already imported at function scope at approximately `:640` — do NOT add it as a top-level import; a top-level `import tempfile` would **shadow the existing function-scope import**.

The direction is backwards. `bin/test_check_green_doc_tense.py:640` reads `import tempfile` as the first statement inside `def run_tests()` (`:639`). That creates a **function-local** binding; per Python scoping the local binding shadows any module-level `tempfile`, never the reverse. Adding a top-level `import tempfile` is in fact harmless here: the local import at `:640` precedes every use, so no `UnboundLocalError` is possible and the local name wins inside `run_tests()`.

**Consequence.** Low, but non-zero: an implementer who trusts the stated mechanism may conclude that top-level imports generally break function-scope ones and refactor the *other* two required imports (`subprocess`, `shutil`) to function scope "for consistency" — the opposite of what the same bullet mandates ("add both at the top of the test file").

**Prescribed fix.** Replace the rationale with the correct fact:

> Note: `tempfile` is already imported at function scope at `bin/test_check_green_doc_tense.py:640` (first statement of `run_tests()`), which is the same function the new code is inserted into — so `tempfile` is already in scope and no import is needed. Do not add a redundant top-level `import tempfile`. (A top-level import would be harmless — the function-local binding at `:640` shadows the module-level name, not the reverse — but it is redundant.) `subprocess` and `shutil` are NOT present at any scope and MUST be added as top-level imports alongside the existing `import sys` / `import textwrap` at `:15-16`.

---

### F-W86S-P19-009 — LOW — STORY-183 — CHANGELOG-preservation instruction enumerates 1 of 2 affected historical loci

**Locus.** `STORY-183.md:913-919` (Task 2, "Do NOT update `CHANGELOG.md` line ~741").

**Defect.** The story's preservation instruction is singular and locus-specific:

> **Do NOT update** `CHANGELOG.md` line ~741: that entry is a SHIPPED HISTORICAL changelog entry (preserved per DF-SIBLING-SWEEP-001); the `_collect_rust_files` name in that entry documents what shipped in STORY-176 …

My Grep of `CHANGELOG.md` for both renamed identifiers returns **two** loci:

```
741:    guard) rather than `2` (root-not-found guard) when `_collect_rust_files` returns
851:  non-zero when no tracked Rust files are found, preventing a silent false-CI-PASS if the
```

`:741` cites the **function name** (correctly enumerated). `:851` cites the **error-string prose** — `"no tracked Rust files"` — which is exactly the string Task 2 renames to `"no tracked source files found"` in `bin/check-green-doc-tense:559`. `:851` is not mentioned anywhere in the story.

The story's `_collect_rust_files` accounting is itself accurate (that identifier appears exactly once in CHANGELOG.md). The gap is that the *error-string* half of the rename has an unenumerated historical prose sibling. Correct disposition is identical to `:741` — **preserve** — but an implementer executing a mechanical `grep -rn 'Rust files'` sweep for the rename has no instruction covering `:851` and may either edit it (violating DF-SIBLING-SWEEP-001's changelog-preservation clause, which `policies.yaml:44` states as "preserve historical changelog entries") or stall.

**Prescribed fix.** Extend the Task 2 preservation note to enumerate both:

> **Do NOT update either CHANGELOG.md locus affected by this rename** — both are SHIPPED HISTORICAL entries preserved per DF-SIBLING-SWEEP-001 (`policies.yaml`: "preserve historical changelog entries"):
> - `:741` — cites `_collect_rust_files` by name (documents what shipped in STORY-162/176);
> - `:851` — cites the pre-rename error-string prose "no tracked Rust files are found" (documents the AC-158-005 zero-file guard as shipped).
> A parenthetical annotation (e.g. "(renamed `_collect_source_files` / 'no tracked source files found' in STORY-183)") may be added inline to either but is NOT required. Similarly, historical references in delivered story specs (STORY-158, STORY-162) are preserved as shipped spec provenance — do NOT sweep `.factory/stories/` history.

---

### F-W86S-P19-010 — NIT — STORY-183 — rename-site categorisation inconsistent between AC-183-001 and Task 2

**Locus.** `STORY-183.md:209-212` vs `:907-912`.

AC-183-001: "13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total) … 6 functional monkey-patch sites …; **7** `_collect_rust_files` prose sites at …; plus :721 — a `rust_files` prose site".

Task 2: same header ("13 … + 1 … (14 total)") but then enumerates "6 functional monkey-patch sites" + "**8** prose/comment sites (approximately :688,:705,:711,:718,**:721**,:839,:843,:891)".

Both reach 14, but Task 2's "8 prose sites" silently folds `:721` (a `rust_files` site, not a `_collect_rust_files` site) into the `_collect_rust_files` prose count, contradicting its own `13 + 1` header. My Grep confirms the AC-183-001 split is the correct one: `_collect_rust_files` occurs exactly 13× (`:688,699,705,707,711,718,726,839,843,859,872,891,905`); `:721` is the sole `rust_files`-only line.

**Prescribed fix.** In Task 2, change "8 prose/comment sites" to "7 `_collect_rust_files` prose/comment sites (approximately :688,:705,:711,:718,:839,:843,:891) **plus 1 `rust_files`-only prose site at :721**" so the enumeration matches the `13 + 1` header and AC-183-001.

---

## Per-axis disposition (explicit, including clean axes)

| # | Axis | Finding? |
|---|---|---|
| 1 | **Truthfulness against the repo** | **CLEAN — remarkably so.** I re-derived every checkable numeric and line-anchor claim in both stories (see Methodology). Every one resolved: 28 tuples / 29 docstring items / item-5-shares-tuple-4; 3625 attribute lines; 40+2=42 violations; 13+1 rename sites; 25 committed captures; 10 top-level `src/*.rs`; 10 `falls through to` sites verbatim; 21 TCs; all `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`, `bin/test_lint_cycle_artifact.py`, `bin/fetch-e2e-pcaps`, `tests/iec104_e2e_real_pcaps_tests.rs`, `tests/fixtures/README.md`, `tests/fixtures/E2E-PCAPS.md`, `.gitignore`, and `ci.yml` anchors; convergence-report `:63-66`/`:68-70` nuance; all four test-fn names. **Zero false, stale, or unverifiable factual claims found.** |
| 2 | **Tautological / non-discriminating predicates** | **F-W86S-P19-005** (Task 9 negative assertion, stderr) and **F-W86S-P19-003** (contradictory + environment-blind N/M gate). All other predicates checked and discriminating: `FIXTURE_MANIFEST.len()==4` and `FIXTURE_GATED_TESTS.len()==4` compare to **fixed literals** (the prior tautology is genuinely gone); needle-count vs registry-len compares independent sources; `parent()` equality (not `starts_with`) correctly rejects `local-samples/` as a subdir; the three AC-183-009 greps currently return 1/1/2 and are required to be 0/0/0; AC-183-001's two class assertions (`any(p.suffix==".py")`, `any(p.parent.name=="src" and p.suffix==".rs")`) are both unsatisfiable without the new globs; AC-183-006's negative guard + `.py` BAD cases are **jointly** discriminating (a forgotten `path.suffix` pass-through fails the `.py` cases while the guard still passes). |
| 3 | **Verifiability** | **F-W86S-P19-004** (four deliverables with no AC). Otherwise every AC has an independently runnable success condition. |
| 4 | **False-GREEN / silent-skip risk** | **F-W86S-P19-002** (two un-hardened blocks) and **F-W86S-P19-005**. Otherwise clean: 11 blocks carry `set -euo pipefail`; the `grep -c \|\| true` non-gating anti-pattern is fully eradicated (replaced everywhere by `test "$(grep -c …)" -eq 0`, which correctly survives `set -e` because a failing command substitution in argument position does not trip `-e`); the `| tee /dev/stderr | grep -q` SIGPIPE pattern is gone; both Env-B blocks pair `grep -qE "Fixture coverage: 1/4"` **with** `grep -qE "test result: ok"`, closing the "coverage line printed before the assert fires" false-green; the AC-182-005 expected-failure block correctly uses `\|\| true` + explicit restore; the three move-aside procedures all carry backup-path pre-existence guards plus `trap … EXIT`. |
| 5 | **Self-referential-flag hazard** | **F-W86S-P19-007** (LOW — mechanism unstated / fragile). No *actual* self-flag exists: I evaluated every `#` comment line the story prescribes for `bin/test_check_green_doc_tense.py` against all 36 patterns (28 shipped + 8 new) and every `//` comment the story prescribes for `tests/iec104_e2e_real_pcaps_tests.rs` — **zero matches**. All BAD/GOOD fixture *content* strings and *label* strings begin with `"` or `'` and are correctly non-comment lines. The P12 "no-literal-phrase" discipline held. The `concat!`-split needle in AC-182-005 is verified to produce zero contiguous occurrences of `fixture_present("` anywhere in the prescribed block. |
| 6 | **Sibling-sweep completeness (DF-SIBLING-SWEEP-001, CRITICAL)** | **Three findings: F-W86S-P19-001** (`:11-12` vs `:62`, same file), **F-W86S-P19-006** (`:4`/`:467`/`:472` vs `:577`, same file), **F-W86S-P19-009** (CHANGELOG `:851` vs `:741`, same file). This was the highest-yield axis this pass — all three are one-fixed/one-missed pairs *within a single file*, the shape the policy names as a "sibling-discipline regression". Deferred siblings (`enip_e2e_real_pcaps_tests.rs`, `e2e_corpus_smoke_tests.rs:206-224`, `bc_2_12_011_story127_tests.rs`, the three stale RED-prose sites, `tests/fixtures/mk_modbus_*.py`, `fuzz/seed_corpus.py`) were verified accurate as characterised and are **not** reported per the accepted-residuals list. |
| 7 | **Internal consistency** | **F-W86S-P19-003** (Task 8 self-contradiction), **F-W86S-P19-004** (Task→AC gap), **F-W86S-P19-010** (NIT). Frontmatter clean in both: `behavioral_contracts: []` / `verification_properties: []` / `assumption_validations: []` / `risk_mitigations: []` all consistent with the "(none — E-11 convention)" body sections; `points` matches STORY-INDEX; `traces_to` files all exist; `inputs` hash MATCHes. Reverse direction (every AC has a Task) is **complete**: AC-182-001→T3/4/5/6, 002→T1/2, 003→T3/5, 004→T9/T11, 005→T6/T8/T9; AC-183-001→T2, 002→T3, 003/004→T6/T7, 005→T11, 006→T7/T8a, 007→T6/T8a/T8b, 008→T12, 009→T13. |
| 8 | **Arithmetic and enumeration** | **CLEAN.** 12 BAD + 14 GOOD = 26 reconciles both ways (T7's 4+3 plus T8a's 8+11 = 12+14); 28+8 = 36 tuples and 29+8 = 37 docstring items; 6 functional + 7 prose + 1 = 14; 1 committed + 3 gitignored = 4 manifest entries; 1/4 → 3 skipped; 31 − 21 = 10 top-level `src/*.rs`; T0836×20 + T1692.001×46 = 66; x=15, 2y=20, x+2y=35, 31+35=66; 10 `falls through to` = 1+3+1+1+2+2. No arithmetic error found. |
| 9 | **Scope integrity** | **CLEAN of blocking issues.** Both narratives are honestly scoped: STORY-182 explicitly limits the structural claim to "the IEC-104 harness delivered here", names the wrong-fixture-content class as the implementer's residual obligation, and states sibling harnesses "retain the structural gap until a follow-up story". STORY-183 documents its residual classes (EC-010, EC-011, Notes §Story scope clarification, §Deferred scrub obligation) accurately, including the Pattern-31/32 contiguity blind spot. One over-broad phrase noted as an Observation below. |
| 10 | **CI/gate realism** | **CLEAN.** STORY-182's additive step is executable as written and coherently placed (`test` job, after `:47`); `if: ${{ !cancelled() }}` semantics are stated correctly, including the honest caveat that the step runs but cannot emit output after a compile failure; the two-grep form means success is determined by outcome, not by string presence alone; no new `uses:` → `action-pin-gate` unaffected; GitHub's default Linux shell is bash so the explicit `set -euo pipefail` is effective. STORY-183's ci.yml edits are comment/step-name only at three verified line anchors, correctly declared non-functional. The two stories' ci.yml edit regions (`~:40-47` vs `:434/:442/:462`) are genuinely disjoint, and both stories carry matching order-dependence warnings. |
| — | **POL-11 / CI-as-code positive-coverage** | **CLEAN.** `green-doc-tense-gate` emits a runtime-computed positive-coverage line (`PASS: … ({len(rust_files)} files scanned)`, `bin/check-green-doc-tense:591`), guarded by a genuine empty-collection exit-1 (`:557-563`); the self-test runs **first** in the same job (`:461`) so the new `.py`-membership class assertion is CI-gated; `action-pin-gate` has an explicit `VALIDATED -eq 0` zero-signal guard (`:427-431`). STORY-182's additive step likewise emits a computed `N/M` and gates on it. The only positive-coverage defect found is F-W86S-P19-005, reported above. |
| — | **Semantic anchoring audit** | **CLEAN.** STORY-INDEX titles byte-identical to H1s; `epic_id: E-11` correct for both; `target_module` (`tests/` and `bin/`) matches actual scope; all cited module/file paths resolve to real workspace artifacts; policy IDs cited (`DF-GREEN-DOC-TENSE-SWEEP`, `DF-TEST-NAMESPACE-001`, `DF-TEST-CITATION-SWEEP-001`, `DF-SIBLING-SWEEP-001`, `DF-VALIDATION-001`, `AC-158-001`) all exist in `policies.yaml`/`ci.yml` with the semantics claimed. **No mis-anchoring found.** |
| — | **DF-KANI-NONVACUITY-001** | N/A — no `#[kani::proof]` harness in either story. |
| — | **DF-TEST-NAMESPACE-001** | **Compliant** — STORY-182 mandates `test_fixture_manifest_report()` and all three consts inside `mod iec104_e2e_real_pcaps` at three separate loci. |
| — | **DF-AC-TEST-NAME-SYNC-001 (incl. unique-resolution rule)** | **Compliant** — all 20+ cargo-test invocations use the module-qualified `iec104_e2e_real_pcaps::<name>` form with `--exact`; all four names verified against `fn test_*` definitions. |
| — | **Compilability / executability spot-check of prescribed code** | **CLEAN.** I type-checked the AC-182-005 Rust block by hand: `&&str: AsRef<Path>` holds via the blanket `impl<T: AsRef<U>> AsRef<U> for &T`, so `Path::join(name)` compiles; match-ergonomics destructuring of `&(&str, &str)` yields `&&str` matching `contains`'s expected `&&str`; `harness_src.contains(&format!(...))` uses `&String: Pattern`; `assert_eq!(resolved.parent(), Some(committed_dir.as_path()))` compares `Option<&Path>` to `Option<&Path>`; `concat!` over two literals is const-evaluable. The AC-183-002/003/004/007 Python is syntactically valid, the top-level `|` alternation in Pattern 34 is correctly unparenthesised, `doesn['’]?t` covers both apostrophe glyphs plus the elided form, and the runner extension `ext = entry[3] if len(entry) > 3 else ".rs"` composes correctly with the existing `label, content = entry[0], entry[1]` unpacking at `:650` (no unpacking change needed) and the `.rs`-only GOOD loop at `:675-676`. |
| — | **First-match-wins label collisions** | **CLEAN.** I evaluated all 8 new BAD fixtures against all 28 shipped patterns in list order: none is intercepted by an earlier pattern, so every fixture reports under its intended `"Pattern NN"` label. All 14 new GOOD fixtures evaluated against all 36 patterns: none matches. In particular `"// which currently fails.\n"` does **not** trip Pattern 31 (`currently\s+falls?\b` cannot match `fails`) — the TIER-2 GOOD case and the TIER-1 pattern do not collide. |

---

## Observations (non-blocking)

- **STORY-183 Narrative over-reach (LOW, adjudicable).** `STORY-183.md:61-62` — "Python files in `bin/` containing stale RED-phase documentation are never silently skipped by the gate" — is broader than the story delivers: docstring prose remains invisible (EC-011, accepted residual DRIFT-docstring-scan) and extension-less `bin/` executables remain out of scope (EC-010). The FILES are no longer skipped, which is a defensible reading; but "never silently skipped" reads as line-level coverage. Suggested tightening: "`#`-comment lines in `bin/*.py` files … are never silently skipped (docstring prose remains a documented residual — EC-011)".
- **Dead alias, no action needed.** `bin/test_check_green_doc_tense.py:34` binds `_is_comment = mod._is_comment_line` and never uses it (1 occurrence in the file). AC-183-002's default-valued `suffix` parameter keeps this alias valid, so the rename/extension is source-compatible; noting only so a future pass does not mistake it for an unswept call site.
- **`bin/changelog-gate-check` message imprecision (out of scope).** `bin/changelog-gate-check:28` prints "no content added to **[Unreleased] section**" although the script never inspects section placement (it counts added non-blank, non-`^+##` lines, `:20-25`). STORY-183's AC-183-005 annotation is *more accurate than the tool's own message*. Pre-existing tool prose, outside both stories' scope; recording for a future maintenance sweep.
- **`[process-gap]`** — none. Neither finding class in this pass points at an agent prompt, hook, rule file, or pipeline template; all ten findings are content defects localised to the two story artifacts. No process-gap follow-up is required for the Cycle-Closing Checklist from this pass.

---

## Novelty assessment

I cannot read passes 1–18. Classification is inferred from the two stories' internal changelog tables (v1.0→v2.8) and from what a prior pass demonstrably already touched.

| ID | Classification | Basis |
|---|---|---|
| F-W86S-P19-001 | **(a) genuinely new** | Task 7's `:59-62` bullet was authored in the v2.8 burst (P18-003 row: "added three new test-file bullets for :47-49, :53-57, :59-62"). That burst created the explicit "this is false post-story" verdict for `:62` and did **not** revisit the `:10-13` bullet, which has stood unchanged since v1.1. The asymmetry only came into existence at v2.8, so no earlier pass could have seen it. |
| F-W86S-P19-002 | **(b) recurrence of a pattern class, at unswept loci** | The `set -euo pipefail` hardening class is the single most-repeated remediation in STORY-182 (F-W86S-P15-003, P16-001, P16-002, P17-001, P17-002 — five separate bursts, ~11 blocks). Passes 15–18 swept the *cargo-test* blocks; the two blocks carrying the size/integrity gates were never in any burst's scope. Same class, new loci. |
| F-W86S-P19-003 | **(c) induced by a prior remediation** | Directly traceable: v2.8 changelog "P18-006 MED (Task 8 :1064-1065: replaced tautology 'M MUST equal FIXTURE_MANIFEST.len()' with discriminating predicate …)". The pass-18 fix removed a genuine tautology and introduced (i) an environment-blind blocking clause and (ii) a contradiction with the "not an independent gate" sentence that P17-004a had added one pass earlier. Two consecutive remediations of the same paragraph produced mutually inconsistent text. |
| F-W86S-P19-004 | **(a) genuinely new** | The four uncovered deliverables were each added by a *different* pass as a **Task** (F-015 → CLAUDE.md row at v1.4; F-012 → E2E-PCAPS.md sweep at v1.3, extended at v1.9/v2.2; F-004/F-026 → gate-entry artifact at v1.2/v1.5; F-P9-011 → `.gitignore` at v1.9). No pass appears to have audited AC↔deliverable coverage after those accretions. The gap is an emergent property of 18 passes of task-list growth without a matching AC, which is exactly the class a fresh-context pass is positioned to see. |
| F-W86S-P19-005 | **(c) induced by a prior remediation** | v2.8 changelog "P18-007 LOW (Task 9 negative assertion added: process output must NOT contain 'no tracked source files found' to distinguish genuine violation exit-1 from empty-collection guard exit-1 at main() :557-563)". Pass 18 correctly identified the two-exit-1-paths ambiguity and added the guard; it did not specify the stream, and `bin/check-green-doc-tense:561` puts the message on stderr. The assertion did not exist before pass 18. |
| F-W86S-P19-006 | **(b) recurrence of a pattern class, at a previously-unenumerated locus** | Task 10's prose-sweep bullets have been re-scoped in three consecutive bursts (P17-006, then P18-001 and P18-002, the latter splitting bullet 1 into four sub-bullets for `:4`, `:26-30`, `:31-85`, `:90`). That split explicitly narrowed the `:4` instruction to "glob-scope text" while a separate bullet performed the `test files`→`source files` rename at `:577`. `:4`'s stale noun phrase and `:467`/`:472` have never appeared in any burst. Same class (Task 10 mis-scoping), new loci. |
| F-W86S-P19-007 | **(b) recurrence — 6th instance of the self-referential-flag class, in meta form** | The changelog records five prior recurrences of this class (v2.1 F-P11-001, v2.2 F-W86S-P12-001 "5th self-referential-predicate recurrence", v2.4 F-W86S-P14-004, v2.6 F-W86S-P16-004, plus v1.5 F-006/v1.7 F-003 correcting the "quoting prevents flagging" fallacy), and D-529 imposed a standing discipline. Task 4's prescription is not itself a violation — but it encodes the *wrong safety criterion* for regex patterns, which is the root cause the five prior fixes each treated symptomatically. Genuinely new framing of a well-worn class. |
| F-W86S-P19-008 | **(a) genuinely new** | Introduced at v2.5 (F-W86S-P15-007: "Task 9 imports :1000-1001: subprocess and shutil are new top-level imports; tempfile already at function scope :640 — top-level add shadows; note added"). The inverted-shadowing claim entered with that note and has survived passes 16, 17, 18 unchallenged. |
| F-W86S-P19-009 | **(a) genuinely new** | The CHANGELOG-preservation instruction was authored at v1.5 (F-004) and refined at v1.6 (F-005) and v1.8 (F-012), each time reasoning about `_collect_rust_files` **by name**. The `:851` locus does not contain that identifier — it contains the error-string prose — so an identifier-scoped sweep would systematically miss it. The error-string rename itself was only added at v1.5 (F-021), after the preservation note was written. |
| F-W86S-P19-010 | **(a) genuinely new, minor** | Introduced at v1.7 (F-005: "'13 references'→'13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total)'; :721 added to prose list"). That burst updated the header in both AC-183-001 and Task 2 but folded `:721` into Task 2's prose count without renumbering, creating the 7-vs-8 divergence. |

**Aggregate novelty signal.** 5 of 10 findings are genuinely new, 3 are recurrences of known classes at loci never previously swept, and 2 are directly induced by the pass-17/pass-18 remediations. **Novelty: MODERATE — decaying but not zero.** The character of the findings has shifted decisively: pass 19 found **zero** false factual claims, **zero** arithmetic errors, **zero** mis-anchors, and **zero** actual self-flagging instances — the axes that dominated earlier passes (per the changelog: CRIT/HIGH pattern redesigns at v1.1–v1.3, HIGH pathspec truth-inversions at v2.3, HIGH self-referential predicates at v2.1/v2.2) are now clean. What remains is a distinct and narrower class: **(i) one-fixed/one-missed sibling pairs inside a single file (3 of 10), and (ii) defects introduced by the two most recent remediation bursts (2 of 10, both in paragraphs edited in consecutive passes).** Both signatures indicate the stories are close to converged but that the *remediation process itself* is now the dominant defect source — successive single-locus bursts on the same paragraph are producing contradictions faster than they remove them (F-W86S-P19-003 is the clearest case: P17-004a and P18-006 edited the same paragraph and left it self-contradictory). **Recommendation for the pass-19 remediation burst:** fix all ten, then run a *whole-paragraph* re-read of Task 8/Task 10a (STORY-182) and Task 9/Task 10 (STORY-183) rather than a locus-targeted edit, and add the mechanical predicates from F-W86S-P19-004's prescribed AC-182-006 so the four uncovered deliverables become gate-checkable rather than prose-only.

---

## Verdict

**NOT_CONVERGED**

**Tally: `10: 0C/0H/6M/3L/1N`**

Zero CRITICAL, zero HIGH — the seventh consecutive zero-HIGH pass by the changelog's own accounting, and the first pass (on this pass's evidence) with a fully clean truthfulness axis. The six MEDIUMs block convergence: two are false-GREEN/non-discriminating-predicate defects (F-W86S-P19-002, F-W86S-P19-005), three are DF-SIBLING-SWEEP-001 one-fixed/one-missed pairs (F-W86S-P19-001, -006, -009 — the last LOW-severity), one is an internal contradiction in text destined for permanent CLAUDE.md governance (F-W86S-P19-003), and one is an acceptance-criteria completeness gap that leaves four of nine declared deliverables unverifiable (F-W86S-P19-004). None requires redesign; all ten have concrete, bounded, single-burst prescriptions above.

**Files referenced (absolute paths):**
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-INDEX.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/policies.yaml`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-085/STORY-180/convergence-report.md`
- `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense`
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py`
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_lint_cycle_artifact.py`
- `/Users/zious/Documents/GITHUB/wirerust/bin/changelog-gate-check`
- `/Users/zious/Documents/GITHUB/wirerust/bin/fetch-e2e-pcaps`
- `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml`
- `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs`
- `/Users/zious/Documents/GITHUB/wirerust/tests/enip_e2e_real_pcaps_tests.rs`
- `/Users/zious/Documents/GITHUB/wirerust/tests/e2e_corpus_smoke_tests.rs`
- `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/README.md`
- `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/E2E-PCAPS.md`
- `/Users/zious/Documents/GITHUB/wirerust/CHANGELOG.md`
- `/Users/zious/Documents/GITHUB/wirerust/.gitignore`

