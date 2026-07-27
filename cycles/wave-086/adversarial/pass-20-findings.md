---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 20
verdict: NOT_CONVERGED
novelty: "medium-low"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 20

**Date:** 2026-07-27
**Pass:** 20 of N

# WAVE-86 STORY-LEVEL ADVERSARIAL REVIEW — PASS 20

---

## PHASE A — ATTESTATION

### A0. Tool-profile disclosure

**Tools actually available to me this session: `Read`, `Grep`, `Glob` only. `Bash` is DENIED** (read-only adversary profile per DF-ADVERSARY-TOOLCHAIN-PAIRING-001). I did not and cannot execute `git`, `cargo`, `python3`, `wc`, or `grep(1)`.

Consequently:
- Every count/line-anchor assertion below labelled **[own-verify]** was produced by my own `Grep`/`Glob`/`Read` against absolute paths.
- Every assertion labelled **[supplied]** is taken from the orchestrator's SUPPLIED EXECUTION EVIDENCE block and is treated as ground truth.
- I assert no execution outcomes I could not produce. Axes requiring execution are explicitly dispositioned below.

### A1. Freshness

**Reviewing develop at SHA e8841d761f3f25f320f98977618e506e8b41a058 (v0.13.2 back-merge).**

I cannot independently read the SHA (no Bash). Independent post-v0.13.2 corroboration from tree content **[own-verify]**:

1. `/Users/zious/Documents/GITHUB/wirerust/Cargo.toml:3` → `version = "0.13.2"`. A pre-`b33e45f9` tree would read `0.13.1`.
2. `/Users/zious/Documents/GITHUB/wirerust/CHANGELOG.md:10` → `## [0.13.2] - 2026-07-25`, with `## [Unreleased]` at `:8`.
3. Post-STORY-180 / gate-fix-PR-#439 (`0ab6f52e`) content is live in the tree: `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs:345` reads `// detected, raising the total from 31 to 66 (+35 findings). The untimed contribution`, and `:27` carries `T0836 ×20 + T1692.001 ×46 = 66 total`. The 31→66 expectation update is the wave-85 gate fix.

No story worktree exists — both artifacts are unimplemented drafts; review target is `develop` + the two `.factory/stories/` files. Confirmed: `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/local-samples/` is absent from this checkout (Glob of `tests/fixtures/*` returns no `local-samples` entry) **[own-verify]**, consistent with supplied evidence #10.

### A2. Story-version assertion

**[own-verify]** Both files carry `version: "2.9"` at line 5:
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md:5` → `version: "2.9"` (status `draft`, points 4, wave 86, `input-hash: "9a0f34c"`)
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md:5` → `version: "2.9"` (status `draft`, points 5, wave 86, `input-hash: "9c9b12f"`)

No abort. Proceeding.

### A3. Discriminating grep-count assertions — ALL THREE MATCH

| Assertion | Expected | My result | Verdict |
|---|---|---|---|
| `fixture_present` in `/Users/zious/.../tests/iec104_e2e_real_pcaps_tests.rs` | 5 | **5** (`:63` def, `:166`, `:292`, `:383`, `:529`) | ✅ MATCH |
| `fixture_present` in `/Users/zious/.../tests/enip_e2e_real_pcaps_tests.rs` | 7 | **7** (`:66` def, `:163`, `:310`, `:446`, `:542`, `:626`, `:734`) | ✅ MATCH |
| git-tracked `bin/*.py` files | 6 | **6** (`test_lint_cycle_artifact.py`, `test_compute_input_hash.py`, `test_changelog_gate_content.py`, `test_validate_citations.py`, `test_check_green_doc_tense.py`, `test_gitignore_mutants_glob.py`) | ✅ MATCH |

Filesystem queried: `/Users/zious/Documents/GITHUB/wirerust` (main repo, `develop`). No methodology-suspect condition.

### A4. Factory-artifact path confirmation

**Confirmed.** Both stories were read exclusively from the main-repo path `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/`. Supporting artifacts read from `/Users/zious/Documents/GITHUB/wirerust/.factory/policies.yaml`, `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-INDEX.md`, `/Users/zious/Documents/GITHUB/wirerust/.factory/STATE.md`, `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-085/STORY-180/convergence-report.md`. No worktree `.factory/` snapshot was used.

**Input-hash**: not re-derived (no Bash). Per supplied evidence #4, both MATCH the canonical Python tool. **No input-hash drift reported.**

---

## Methodology

Full reads of both story files (1427 + 1327 lines). Independent corroboration performed against:

- `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense` (597 lines: `:4`, `:26-30`, `:31-85`, `:87-88`, `:90`, `:97`, `:206-217`, `:212-215`, `:360-457`, `:460-462`, `:465-499`, `:502-524`, `:527-546`, `:549-596`)
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py` (913 lines: `:1-40`, `:41-160`, `:205-275`, `:633-652`, `:680-735`, `:852-913`)
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_lint_cycle_artifact.py` (`:1-15`, `:120-131`)
- `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs` (`:1-120`, `:130-180`, `:265-305`, `:344-360`, `:496-510`)
- `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/E2E-PCAPS.md`, `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/README.md`, `/Users/zious/Documents/GITHUB/wirerust/.gitignore`, `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml` (`:28-67`, `:425-544`), `/Users/zious/Documents/GITHUB/wirerust/bin/fetch-e2e-pcaps` (`:154`, `:157`)
- `/Users/zious/Documents/GITHUB/wirerust/.factory/policies.yaml` (all 19 policy IDs enumerated; DF-GREEN-DOC-TENSE-SWEEP read in full at `:1045-1473`)

**Facts I independently CONFIRMED as true in both stories** (recorded so the orchestrator does not re-litigate them):

| Claim | Verified |
|---|---|
| S-183: 28 tuples in `_VIOLATION_PATTERNS` at `:217`, labels 1–29 in docstring | ✅ 28 `re.compile` entries |
| S-183: 13 `_collect_rust_files` sites + `rust_files` prose at `:721` = 14 total; 6 functional (`:699/:707/:726/:859/:872/:905`), 7 prose (`:688/:705/:711/:718/:839/:843/:891`) | ✅ EXACT |
| S-183: AC-158-005 block `:698-726` patch at `:704`; AC-162-003 block `:858-905` patch at `:871`; `finally` ends `:905`; `print()` at `:907` | ✅ EXACT |
| S-183: 40 BAD_CASES entries → 40 `//` violations, + 2 `#` lines at `:258`/`:261` = 42 | ✅ EXACT (40 tuple closers `:57`–`:325`; my own pattern-by-pattern analysis of every `#` line in all 6 `bin/*.py` confirms **only** `:258` and `:261` match) |
| S-183: prescribed `\b`-escaped replacements for `:258`/`:261` do NOT match Pattern 26/29 | ✅ (leading `b` of `\b` is a word char → boundary cannot fire) |
| S-183: 10 live `falls through to` sites, all 10 enumerated correctly | ✅ EXACT (matches policies.yaml `:1275-1280`) |
| S-183: 8 new TIER-1 tokens have **zero** live matches in `src/` and `tests/` comment lines, and zero in `bin/` | ✅ verified by direct grep |
| S-183: 10 newly-in-scope top-level `src/*.rs` files have zero matches vs the 28 existing patterns | ✅ verified; `src/*.rs` Glob = exactly 10 files |
| S-183: `tests/iec104_analyzer_tests.rs:6950` contiguity blind spot ("currently these fall") | ✅ EXACT |
| S-183: convergence-report `:63-66` = D-506 token ground truth; `:68-70` = broader/incorrect PG-W85-003 labels | ✅ EXACT |
| S-183: 12 new BAD + 14 new GOOD = 26; 28+8 = 36 tuples; 37 docstring items | ✅ arithmetic correct, Task 7/8a split sums exactly |
| S-183: `bin/test_lint_cycle_artifact.py` `:3`/`:5`/`:6`/`:125` all stale as described; the 3 AC-183-009 greps are discriminating (currently 1/1/2) | ✅ EXACT |
| S-183: `tempfile` at function scope `:640`; `subprocess`/`shutil` absent from top-level imports; runner writes `bad_{passed}.rs` at `:652`; BAD_CASES annotation at `:51`; quote-bearing fixtures at `:91`/`:97` | ✅ EXACT |
| S-183: git pathspec semantics (`src/*.rs` strictly subsumes `src/**/*.rs` by 10 files; `src/*.rs` LOAD-BEARING) | ✅ consistent with supplied #5; no residual inversion anywhere in the story |
| S-183: ci.yml `:434`/`:442`/`:462`; AC-183-005 three-dot form matches ci.yml `:533`; `bin-selftest` at `:473` lacks both steps | ✅ EXACT |
| S-182: `fixture_present` `:63`, `run_iec104_pipeline` `:97`, `LOCAL_SAMPLES` `:51`, `use std::path::Path` `:39`, banner `:53-57`, fixture-root `:47-49`, helper doc `:59-62`, mapping table `:23-28`, licence prose `:138`/`:273`, per-test licence `:353-354`/`:503-504` | ✅ EXACT |
| S-182: `grep -c 'keeps CI green'` = 2 (`:12`, `:62`) | ✅ EXACT |
| S-182: all 4 `FIXTURE_GATED_TESTS` fn names exist (`:165`, `:291`, `:382`, `:528`) | ✅ EXACT |
| S-182: needle `concat!("fixture_present", "(\"")` count = 4 today; zero contiguous needle occurrences in the prescribed block | ✅ verified char-by-char |
| S-182: E2E-PCAPS.md `:3-6`, `:48-50`, `:337-340`, `:352-359`, `:358` sha `07b9a087…`/14 KB/173 pkts, `:359` sha `292c18a8…`/11 KB, `:374-380`, `:388`/`:389` URLs, `:391-396` | ✅ EXACT |
| S-182: README.md `:5-34`, notice `:7-22` + malware clause `:24-26`, provenance table `:30-34`, "remaining fixtures" `:41-44`; `iec104-iti-diverse.pcap` absent | ✅ EXACT |
| S-182: 25 committed capture files in `tests/fixtures/` | ✅ EXACT (counted 25 `.pcap`/`.pcapng`/`.cap`/`.trace`) |
| S-182: `bin/fetch-e2e-pcaps:154,157` Wireshark sha256 values | ✅ EXACT byte-for-byte |
| S-182: `.gitignore` covers only `/tests/fixtures/local-samples/` (`:10`); `coverage-out.txt` absent; `mutants.out*/` present | ✅ EXACT |
| S-182: CLAUDE.md has exactly 6 `.factory/maintenance/` protocol-doc rows at table bottom | ✅ EXACT |
| Both: STORY-INDEX `:297`/`:298` titles/points/status/version match story H1s byte-for-byte; `:456` wave-86 = 2 stories / 9 pts (4+5) | ✅ EXACT |
| Both: all 4 STATE.md drift rows (`DRIFT-docstring-scan` `:220`, `DRIFT-py-surface-outside-bin` `:224`, `DRIFT-e2e-sibling-harnesses`, `DRIFT-stale-red-scrub`) exist as claimed | ✅ |
| S-183: policy DF-GREEN-DOC-TENSE-SWEEP v6 TIER-1/TIER-2 assignments match story exactly; policy `:1351-1359` STORY-183 direct-scrub obligation is satisfied (superset) | ✅ |
| Both: prescribed Rust snippets type-check on inspection (`&&str`→`&str` deref coercion at call sites; `AsRef<Path> for &T`; `matches().count()` vs `len()` both `usize`) | ✅ |

**Self-validation:** 3 refinement iterations run. Iteration 2 dropped a proposed finding that "M MUST equal 4" in Task 8 is tautological — it is in fact discriminating (it fires when `FIXTURE_MANIFEST` legitimately grows to 5 with both const and assertion co-updated). Iteration 3 dropped a proposed finding on `src/**/*.rs` pathspec claims — the current text is correct at every locus.

---

## FINDINGS

| ID | Sev | Story | Locus | One-line |
|---|---|---|---|---|
| F-W86S-P20-001 | MEDIUM | 183 | Task 2, `:883-886` | Falsely attributes `_find_repo_root` monkey-patching to Task 9; contradicts AC-183-001 + Task 9 |
| F-W86S-P20-002 | MEDIUM | 182 | AC-182-006, `:818-823` | Tautological predicate — `grep -c 'tests/fixtures/' E2E-PCAPS.md -ge 1` passes on baseline |
| F-W86S-P20-003 | MEDIUM | 182+183 | AM/FSR vs ACs | ci.yml deliverables in both stories have no AC predicate |
| F-W86S-P20-004 | MEDIUM | 182 | AC-182-005 Verification `:798-812` | RED demonstration cannot fail (`\|\| true`, no assertion, no `set -e`) |
| F-W86S-P20-005 | MEDIUM | 183 | AC-183-002 Verification `:310-318` | Block cannot fail — `echo "Exit code: $?"` swallows every exit code |
| F-W86S-P20-006 | MEDIUM | 182 | `:342`, `:480`, `:1145` | 3 bash blocks still lack head `set -euo pipefail`; first check non-gating |
| F-W86S-P20-007 | MEDIUM | 182 | Task 8 `:1126-1137`, Task 10a `:1211` | Same paragraph declares N<1 both blocking AND evidence-only |
| F-W86S-P20-008 | MEDIUM | 182 | AC-182-006 `:832-835` | `test -f .factory/...` environment-blind; `.factory/` gitignored, not in the PR |
| F-W86S-P20-009 | MEDIUM | 183 | Task 9 `:1043-1044` | Hermetic `#`-fixture has no single-line mandate → self-flag hazard under AC-183-002 |
| F-W86S-P20-010 | LOW | 182 | AC-182-006 `:816` | "four governance deliverables" vs 5 bullets / 6 predicates |
| F-W86S-P20-011 | LOW | 183 | Task 4 `:945-946` | ":259 safe by the same mechanism" — wrong mechanism (verdict correct) |
| F-W86S-P20-012 | LOW | 183 | Task 5 `:963`, EC-005 | 46 GOOD_CASES multi-line fixtures unconverted and unacknowledged (pending intent verification) |
| F-W86S-P20-013 | LOW | 182 | FSR `:1323`, Task 7 `:1037-1038` | False-green claim anchored `:11-12`; sentence spans `:12-13` |
| F-W86S-P20-014 | NIT | 183 | AC-183-007 Pattern 33 comment `:552` | "fires when 'through' is absent" overstates the regex |
| F-W86S-P20-015 | LOW | 182 | Narrative `:56-62` | "structurally prevented" vs the story's own retrospective (detection, not prevention) |

---

### F-W86S-P20-001 — MEDIUM — STORY-183 — `[process-gap]`

**Locus:** `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md:883-886` (Task 2).

**Defective claim (verbatim):**
```
   **`repo_root` derivation:** use `mod._find_repo_root(Path(mod.__file__).resolve().parent)`
   (where `mod` is the imported module). The hermetic test functions (Task 9) monkey-patch
   `_find_repo_root`, so the derivation choice is load-bearing for non-hermetic self-tests —
   using the script's own file location ensures correct repo root resolution.
```

**Evidence — this directly contradicts two other loci in the same story:**

`STORY-183.md:222-232` (AC-183-001):
> "Note: the PRE-EXISTING monkey-patch sections AC-158-005 (bin/test_check_green_doc_tense.py:698-726, patch at :704) and AC-162-003 (:858-905, patch at :871) patch `_find_repo_root` … **The Task 9 hermetic test runs a fresh subprocess and cannot see parent-process monkey patches**"

`STORY-183.md:1029-1030` (Task 9):
> "The Task 9 hermetic test runs a fresh subprocess and cannot see parent-process monkey patches; placement of the hermetic test relative to the finally blocks does NOT affect the subprocess."

**Repo corroboration [own-verify]** — the monkey-patchers are pre-existing, not Task 9's:
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py:704` → `mod._find_repo_root = lambda _s: _hermetic_005  # type: ignore[attr-defined]`
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py:871` → `mod._find_repo_root = lambda _start: _hermetic_root  # type: ignore[attr-defined]`

**Why it matters:** Task 2 is the task an implementer executes for the rename + self-test additions. Reading Task 2 alone, the implementer is told the hazard originates in "Task 9" — a task that does not yet exist in the file and that (per AC-183-001 and Task 9) is provably *not* the hazard source. The actual hazard is placing the new in-process `_collect_source_files(repo_root)` assertions inside the `:698-726` or `:858-905` blocks. The mis-attribution can lead the implementer to the wrong placement and a spurious FAIL.

**Sibling-sweep provenance:** v2.5 changelog records `F-W86S-P15-005 MED (two loci corrected: (a) AC-183-001 :222-226 …; (b) Task 9 :998-1003 …)`. Exactly two loci were fixed; the third sibling statement in Task 2 was not swept. This is the DF-SIBLING-SWEEP-001 "one-fixed/one-missed inside a single file" shape.

**Prescribed fix:** Replace `:884-885` with: *"The PRE-EXISTING monkey-patch blocks — AC-158-005 (`:698-726`, `_find_repo_root` patched at `:704`) and AC-162-003 (`:858-905`, patched at `:871`) — return hermetic tempdirs, so these two assertions MUST be inserted outside them (immediately after the `finally` at `:905`, before the `print()` at `:907`). Task 9's hermetic test is a subprocess and does NOT monkey-patch anything."*

---

### F-W86S-P20-002 — MEDIUM — STORY-182

**Locus:** `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md:818-823` (AC-182-006, first bullet).

**Defective claim (verbatim):**
```
- Then `tests/fixtures/E2E-PCAPS.md` documents at least one `tests/fixtures/` path reference:
  ```bash
  set -euo pipefail
  test "$(grep -c 'tests/fixtures/' tests/fixtures/E2E-PCAPS.md)" -ge 1
```

**Evidence [own-verify]** — the predicate already holds on baseline `develop`, before any story work. `tests/fixtures/E2E-PCAPS.md` contains `tests/fixtures/` at (at minimum) four pre-existing lines:
- `:6` → `` `PCAP-CORPUS-001`). They live, gitignored, under `tests/fixtures/local-samples/`. ``
- `:16` → `` `tests/fixtures/local-samples/`, verifying every checksum. ``
- `:48` → `` > A tiny committed fixture, `tests/fixtures/modbus-write.pcap` (8 packets), is ``
- `:393` → `` 1. Drop the `.pcap` in `tests/fixtures/local-samples/` (gitignored). ``

**Why it matters:** This is a **non-discriminating predicate** (axis 2). It cannot fail, and it would pass with the entire Task 7 E2E-PCAPS.md sweep — six prescribed loci (`:3-6`, `:48-50`, `:337-340`, `:352-359`, `:374-380`, `:391-396`) — skipped in full. AC-182-006's stated purpose is "governance-surface completeness"; on this surface it verifies nothing.

**Novelty class:** remediation-induced. AC-182-006 was **added in v2.9** (changelog: `F-004 MED (AC-182-006 added after AC-182-005 … governance-surface completeness AC …)`), one pass after `P18-006` removed a *different* tautology (`M MUST equal FIXTURE_MANIFEST.len()`).

**Prescribed fix:** Replace the predicate with one keyed to text Task 7 actually introduces and that is provably absent today. Verified absent from `E2E-PCAPS.md` **[own-verify]** — the substring `committed at` returns zero hits. Use:
```bash
grep -qF 'committed at `tests/fixtures/`' tests/fixtures/E2E-PCAPS.md   # Task 7 :358-row annotation
test "$(grep -c 'All are auto-fetchable via `bin/fetch-e2e-pcaps`' tests/fixtures/E2E-PCAPS.md)" -eq 0
```
(The second is discriminating: `:340` currently reads `analyzer (SS-19). All are auto-fetchable via \`bin/fetch-e2e-pcaps\`.` and Task 7 mandates amending exactly that claim.)

---

### F-W86S-P20-003 — MEDIUM — STORY-182 + STORY-183 — `[process-gap]`

**Loci:**
- STORY-182: `:855` (Architecture Mapping), `:1328` (FSR), `:1288-1310` (ACR) declare the additive ci.yml step — but **no AC** asserts its presence.
- STORY-183: `:811` (Architecture Mapping), `:1236` (FSR), `:1138-1146` (Task 10) declare ci.yml prose edits at `:434`/`:442`/`:462` — but **no AC** covers them.

**Defective condition:** Two declared deliverables have no independently checkable acceptance predicate (axis 3: "every declared deliverable needs an AC"). STORY-182's ci.yml step is the *only* mechanism the story offers for CI visibility of the coverage summary — AC-182-004 outcome (e) *describes* it, but AC-182-004's Verification block only runs `cargo test` locally; it never inspects `.github/workflows/ci.yml`. AC-182-006 ("governance-surface completeness", added v2.9) enumerates five surfaces and omits ci.yml entirely.

**Evidence:**
- **[supplied #11]** The strings `Fixture coverage`, `if: always()`, and `cancelled()` are **not present anywhere in ci.yml** today. A predicate for STORY-182's step is therefore trivially discriminating.
- **[own-verify]** `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml:434` → `  # DF-GREEN-DOC-TENSE-SWEEP gate: detect stale RED-phase comment headers in test files.` and `:462` → `      - name: Scan for stale RED-phase comment headers in test files`. The phrase `in test files` occurs 2× → a discriminating predicate for STORY-183 exists.
- **[own-verify]** `ci.yml:442` → `  # Implementation: bin/check-green-doc-tense scans tracked tests/*.rs and src/**/*.rs`.

**Prescribed fix:**
- STORY-182 — add to AC-182-006:
  ```bash
  grep -qF 'IEC-104 fixture coverage report (visible)' .github/workflows/ci.yml
  grep -qF 'Fixture coverage: [1-9][0-9]*/[0-9]+' .github/workflows/ci.yml
  grep -qF '!cancelled()' .github/workflows/ci.yml
  ```
- STORY-183 — add a new AC (or extend AC-183-001's Then clause):
  ```bash
  test "$(grep -c 'in test files' .github/workflows/ci.yml)" -eq 0        # currently 2
  grep -qF 'src/*.rs' .github/workflows/ci.yml
  grep -qF 'bin/*.py' .github/workflows/ci.yml
  ```

**Process-gap:** No template or agent-prompt rule requires that every Architecture-Mapping / FSR row map to at least one checkable AC predicate. This gap produced two independent instances in the same wave.

---

### F-W86S-P20-004 — MEDIUM — STORY-182

**Locus:** `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md:798-812` (AC-182-005 Verification).

**Defective block (verbatim `:803-811`):**
```bash
# Verify the hard-assert fires by temporarily renaming a committed capture:
# (manual test only — do not automate file removal in CI)
[ ! -e /tmp/iec104-iti-diverse.pcap.bak ] || { echo "backup path occupied — clean up first"; exit 1; }
mv tests/fixtures/iec104-iti-diverse.pcap /tmp/iec104-iti-diverse.pcap.bak
# This block EXPECTS cargo test to FAIL (hard-assert fires); || true ensures the restore
# line runs unconditionally under set -e (expected-failure block must use || true or trap):
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact || true
# Expected: test_fixture_manifest_report FAILS with assertion message citing iec104-iti-diverse.pcap
mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap  # restore
```

**Why it is defective (axis 3 + axis 4):**
1. The block contains **no `set -euo pipefail`** anywhere. The `mv` at `:806` can fail silently; if it does, `:809` runs against a still-present fixture, the hard-assert does *not* fire, and `|| true` masks that entirely.
2. `|| true` converts the expected-FAIL into unconditional success. There is **no predicate at all** asserting the assertion actually fired. The success condition is the subjective comment "`# Expected: … FAILS with assertion message citing iec104-iti-diverse.pcap`".
3. The block therefore **passes even if the hard-assert is never implemented** — i.e. it would pass if the AC-182-005 work were skipped.

**Aggravating factor — this block is designated the story's only RED evidence.** `STORY-182.md:1368-1376` (Notes):
> "**tdd_mode: strict — E-11 template note:** … satisfied for this governance story by the documented manual RED demonstration (move the committed capture aside … **observe the hard-assert fire** citing the missing capture, then restore — see AC-182-005 hard-assert verification block)."

`tdd_mode: strict` (frontmatter `:21`) is thus grounded in a block whose success criterion is "observe".

**Prescribed fix** — replace `:809` with an inverted-gate form that fails when the assert does *not* fire:
```bash
set -uo pipefail
if cargo test --test iec104_e2e_real_pcaps_tests \
     iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact > red-out.txt 2>&1; then
  mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap
  echo "FAIL: expected hard-assert to fire with the committed capture moved aside"; exit 1
fi
grep -qF "REGRESSION: committed fixture 'iec104-iti-diverse.pcap' is absent" red-out.txt \
  || { mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap; \
       echo "FAIL: test failed for the wrong reason"; exit 1; }
mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap  # restore
```

---

### F-W86S-P20-005 — MEDIUM — STORY-183 — `[process-gap]`

**Locus:** `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md:310-318` (AC-183-002 Verification).

**Defective block (verbatim):**
```bash
# After all scrubs and rewording, full bin/*.py scan must exit 0:
python3 bin/check-green-doc-tense
echo "Exit code: $?"  # Must be 0

# Self-test must pass:
python3 bin/test_check_green_doc_tense.py
echo "Exit code: $?"  # Must be 0
```

**Why it is defective (axis 4):** `echo` always exits 0. Under no shell setting can this block exit non-zero:
- No `set -e`, so `python3 bin/check-green-doc-tense` returning 1 does not stop the block.
- Even *with* `set -e`, a command whose failure is immediately consumed as the predicate of a following statement is not the issue here — the problem is that `echo "Exit code: $?"` **prints** the code rather than **testing** it, and is the last command in the block. The block's exit status is `echo`'s.

This is precisely the "prints PASS/FAIL instead of evaluating the predicate" anti-pattern, and it guards the story's **hard** zero-false-positive requirement — `STORY-183.md:1208-1211` (ACR): *"**Zero-false-positive hard requirement:** `python3 bin/check-green-doc-tense` MUST exit 0 after delivery."* The hard requirement's own verification cannot fail.

**Sibling-sweep provenance:** v2.3 changelog records `F-W86S-P13-004 MED (AC-183-009: two inverted-gate loci fixed — grep -c exits 1 when count is 0, replaced with test "$(grep -c ...)" -eq 0 gating form at both :743-748 and :762-765; "return 0" → "print 0")`. The identical class was fixed at AC-183-009 and not swept into AC-183-002.

**Prescribed fix:**
```bash
set -euo pipefail
python3 bin/check-green-doc-tense          # gates: exits non-zero on any violation
python3 bin/test_check_green_doc_tense.py  # gates: exits non-zero on any failed self-test
echo "PASS: gate clean and self-test green"
```

---

### F-W86S-P20-006 — MEDIUM — STORY-182 — `[process-gap]`

**Loci (3):**
- `STORY-182.md:342-345` — AC-182-001 Verification, Environment A
- `STORY-182.md:480-484` — AC-182-003 Verification, first check
- `STORY-182.md:1145-1148` — Task 9 Environment A, first check

**Defective condition:** In each block, the first `cargo test … | grep …` runs **before** any `set -euo pipefail`, and is followed by further commands. Its non-zero exit is therefore silently discarded.

Concretely, AC-182-001 Verification (`:342-372`):
```bash
# line 343-345 — NO set -euo pipefail yet:
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact 2>&1 | grep -E "1 passed"
# Must show: test result: ok. 1 passed (committed+local fixtures all present)
...
# line 352:
if [ -d tests/fixtures/local-samples ]; then
...
# line 356 — set -euo pipefail appears only HERE, inside the if-branch:
  set -euo pipefail
```
Identically, AC-182-003 (`:482-484` runs before `set -euo pipefail` at `:489`) and Task 9 Env A (`:1146-1148` runs before `set -euo pipefail` at `:1153`).

**Sibling-sweep provenance (this is the core of the finding):** the v2.9 changelog explicitly records a sweep of exactly this class and reports **two** fixes:
> `F-002 MED (two missing `set -euo pipefail` added: (a) first line of AC-182-002 Verification bash block, after ```bash fence before git ls-files; (b) first line of Task 1 bash block, before # Step 1a:)`

The sweep found 2 of 5 such block heads. Three remain. Additionally, the same class was already remediated in v2.5 (`F-W86S-P15-003`), v2.6 (`F-W86S-P16-001/002`) and v2.7 (`P17-001/002`) — this is the **fifth consecutive pass** in which the "verification-block gating" class recurs.

**Prescribed fix:** insert `set -euo pipefail` as the first line immediately after the ```` ```bash ```` fence at `:342`, `:480`, and `:1145`. Then, for consistency with the already-hardened blocks, replace `… 2>&1 | grep -E "1 passed"` with the tee-to-file form (`| tee coverage-out.txt` then `grep -q`), because `set -o pipefail` alone still lets a `grep` match on a partially-succeeded run.

**Process-gap:** there is no hook or lint step that checks story-spec fenced `bash` blocks for a head `set -euo pipefail`. Five passes of manual sweeping have not converged this class. Candidate: extend `bin/lint-cycle-artifact` (or a new `bin/lint-story-bash-blocks`) to assert every multi-command ```` ```bash ```` fence in `.factory/stories/*.md` begins with `set -euo pipefail`.

---

### F-W86S-P20-007 — MEDIUM — STORY-182

**Loci:** `STORY-182.md:1126-1137` (Task 8, "Enforceable wave-gate obligation") and the mirrored prescription at `STORY-182.md:1211` (Task 10a CLAUDE.md row).

**Defective text (verbatim `:1126-1137`, emphasis on the two clauses in tension):**
```
   - **Enforceable wave-gate obligation (F-026):** Before G1 of any wave-gate evaluation that
     includes e2e pcap tests, run:
     `cargo test … test_fixture_manifest_report -- --exact --nocapture`
     and record the printed N/M in the gate entry. **M MUST equal 4 (the literal).** **N MUST
     equal 1 (`COMMITTED_FIXTURES.len()`) when `tests/fixtures/local-samples/` is absent, or
     4 (`FIXTURE_MANIFEST.len()`) when it is fully populated; an intermediate N is legitimate
     on a partially-populated host and is recorded as-is.** **Any `M ≠ 4`, or any `N < 1`,
     blocks gate entry pending investigation.** Absence of a committed-partition member from
     `tests/fixtures/` is caught independently by the AC-182-005 hard-assert (`cargo test`
     failure) — that assert, not the recorded N/M, is the blocking gate; the recorded N/M is
     the evidence artifact.
```

**Why it is a genuine contradiction (axis 7 — "any obligation simultaneously described as blocking-and-enforceable AND as evidence-only"):**

`N < 1` **is** the committed-fixture-absence condition. `N = present.len()` over `FIXTURE_MANIFEST`, and `COMMITTED_FIXTURES = ["iec104-iti-diverse.pcap"]` is the only entry guaranteed present in a clean checkout; `N = 0` means that capture is absent. The paragraph therefore assigns blocking authority for one and the same condition to **two different mechanisms** in adjacent sentences:
- sentence 1: "**Any … `N < 1`, blocks gate entry pending investigation.**"
- sentence 2: "Absence of a committed-partition member … — **that assert, not the recorded N/M, is the blocking gate**; the recorded N/M is the evidence artifact."

**Novelty class — remediation-induced, over three passes:**
- v2.7 `P17-004a`: *"wave-gate obligation rewritten: N/M is evidence artifact not independent gate"* → made it evidence-only.
- v2.8 `P18-006`: *"replaced tautology … with discriminating predicate 'M MUST equal 4 (the literal) and N MUST equal COMMITTED_FIXTURES.len() … any other N/M pair blocks gate entry'"* → re-added a blocking predicate without removing the P17 disclaimer.
- v2.9 `F-003`: rewrote the blocking condition to `M ≠ 4 or N < 1` — still without reconciling the disclaimer.

**Prescribed fix:** pick one authority per failure class and state it once. Recommended wording:
> "`M ≠ 4` blocks gate entry (manifest-size drift not co-updated here). `N` is recorded as evidence only, together with the environment declaration (local-samples absent / partially populated / fully populated); committed-capture absence is blocked by the AC-182-005 hard-assert, which fails `cargo test` and therefore fails the `grep -qE \"test result: ok\"` check in this same command — `N` itself is not a blocking datum."

Apply the identical correction to the CLAUDE.md row prescribed at `:1211`, which currently reproduces both clauses verbatim.

---

### F-W86S-P20-008 — MEDIUM — STORY-182

**Locus:** `STORY-182.md:832-835` (AC-182-006, fourth bullet).

**Defective claim (verbatim):**
```
- And `.factory/maintenance/fixture-count-gate-entry.md` exists on the factory-artifacts branch:
  ```bash
  test -f .factory/maintenance/fixture-count-gate-entry.md
  ```
```

**Evidence [own-verify]:**
- `/Users/zious/Documents/GITHUB/wirerust/.gitignore:4` → `.factory/`. The entire `.factory/` tree is gitignored on `develop`.
- The story itself states the file is not in the develop PR — `STORY-182.md:1327` (FSR): `| \`.factory/maintenance/fixture-count-gate-entry.md\` | New (factory-artifacts branch — committed by state-manager, NOT in develop PR) |`, and `:1382-1384` (Notes): *"**Note:** `.factory/maintenance/fixture-count-gate-entry.md` is NOT in the develop PR. It is committed separately to the `factory-artifacts` branch by state-manager."*

**Why it matters (axis 8):** `test -f` on a relative path resolves against cwd. It succeeds **only** in the main repo root where `factory-artifacts` is mounted as a worktree at `.factory/`. It fails in: a CI checkout of `develop`, any `git worktree` for the story branch (where `.factory/` is gitignored and absent), and any fresh clone. The predicate is stated unconditionally with no environment qualifier — in a story that is otherwise scrupulous about environment declaration (Env A / Env B protocol, the `if [ -d tests/fixtures/local-samples ]` guards, and the "WITHOUT-LOCAL-SAMPLES PRECONDITION" annotations). This is an internal-consistency gap as much as an environment-blindness one.

Secondary: the check is also mis-worded. `test -f` verifies presence in the **working tree at the mount point**, not "on the factory-artifacts branch". A branch-scoped check would be `git cat-file -e factory-artifacts:maintenance/fixture-count-gate-entry.md`.

**Prescribed fix:** qualify the environment and use a branch-accurate predicate:
```bash
# Environment: main repo root ONLY (factory-artifacts mounted at .factory/).
# NOT runnable from a story worktree or a CI develop checkout — .factory/ is gitignored (.gitignore:4).
test -f .factory/maintenance/fixture-count-gate-entry.md
# Branch-scoped alternative (runnable anywhere the ref is fetched):
git cat-file -e factory-artifacts:maintenance/fixture-count-gate-entry.md
```

---

### F-W86S-P20-009 — MEDIUM — STORY-183

**Locus:** `STORY-183.md:1043-1044` (Task 9, hermetic-e2e fixture creation).

**Defective claim (verbatim):**
```
   - Create `<tmp>/bin/violating.py` with a `#`-prefixed comment line containing a TIER-1
     phrase (e.g., `"# currently asserts the implementation is complete\n"`)
```

**Why it is a self-inflicted-failure hazard (axis 5):**

After AC-183-002, `#`-prefixed lines in `.py` files become scan-eligible (`STORY-183.md:294-298`):
```python
      if stripped.startswith("#") and suffix == ".py":
          return True
```
`bin/test_check_green_doc_tense.py` is itself a `.py` file and, per ACR `:1210-1211`, **must remain in the scan set** ("No skip-file pragma is permitted on the self-test file"). Scanning is line-based (`scan_file()` at `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense:515-518` iterates `text.splitlines()` and tests `stripped`), so it cannot distinguish a real `#` comment from a `#`-prefixed **content line inside a triple-quoted string**.

Task 5 mandates single-line form for the 40 `//` BAD_CASES fixtures (`:948-966`) precisely because of this line-based blindness. **There is no equivalent mandate for the new `#`-content class**, and the story's edge cases do not cover it — EC-003 (`:824`) covers only `"// Expected RED: …"` (`.py` line beginning with `"`), and EC-005 (`:826`) covers only `//` multi-line literals.

If the implementer writes the Task 9 fixture as:
```python
        (tmp / "bin" / "violating.py").write_text(
            """\
# currently asserts the implementation is complete
"""
        )
```
then the content line `# currently asserts the implementation is complete` in `bin/test_check_green_doc_tense.py` strips to a `#`-prefixed line in a `.py` file → `_is_comment_line(stripped, ".py")` returns `True` → **Pattern 32 fires** → `python3 bin/check-green-doc-tense` exits 1 → AC-183-002's hard zero-FP requirement and AC-183-001's `Then` clause both break.

**Corroboration [own-verify]:** the token `currently asserts` currently has **zero** occurrences anywhere under `/Users/zious/Documents/GITHUB/wirerust/bin/` — so the only way it enters `bin/*.py` is via this story's Task 9 prescription, making this the sole new instance of the class.

**Prescribed fix:** add to Task 9:
> *"The violating-fixture content MUST be written as a **single-line** Python string literal — e.g. `(tmp / \"bin\" / \"violating.py\").write_text(\"# currently asserts the implementation is complete\\n\")`. Do NOT use a triple-quoted multi-line literal: after AC-183-002 a `#`-prefixed content line inside a `.py` source file IS scan-eligible and would self-flag Pattern 32 in `bin/test_check_green_doc_tense.py`, breaking the AC-183-002 zero-FP requirement (same mechanism as Task 5, new `#` class)."*

Also add an EC row mirroring EC-005 for the `#`-content case.

---

### F-W86S-P20-010 — LOW — STORY-182

**Locus:** `STORY-182.md:816`.

**Claim:** "The **four** governance deliverables introduced by this story are all present and consistent."

**Evidence:** AC-182-006 then enumerates **five** `- Then/- And` bullets (`:818`, `:824`, `:828`, `:832`, `:836`) carrying **six** shell predicates (the first bullet has two: the `E2E-PCAPS.md` grep and the `README.md` grep). Neither reading of "four" maps to the enumeration; additionally, two of the five bullets (`E2E-PCAPS.md` sweep, `keeps CI green` scrub) are amendments to existing files rather than "introduced" deliverables, while two genuinely introduced deliverables (the committed pcap, the ci.yml step) are absent from the list entirely.

**Fix:** change to "The governance surfaces touched by this story are all present and consistent" and enumerate the count explicitly if a count is wanted (after F-W86S-P20-003 lands, that becomes six bullets / eight predicates).

---

### F-W86S-P20-011 — LOW — STORY-183

**Locus:** `STORY-183.md:945-946` (Task 4, final sentence).

**Claim:** "Sibling lines `:213` and `:259` in the same comment blocks are already safe **by the same mechanism** and are deliberately left unchanged."

**Evidence [own-verify]** — the two lines are safe for *different* reasons:

`/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py:213`
```
    #   (a) All tests\b.*\bMUST FAIL   (interposed words between tokens)
```
Pattern 23 is `re.compile(r"All tests\b.*\bMUST FAIL", re.IGNORECASE)` (`check-green-doc-tense:376`). The `\bMUST FAIL` assertion cannot fire because the character preceding `MUST` in the text is the literal `b` of `\b` — a word character. **This IS the `\b`-escape mechanism.** ✅

`/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py:259`
```
    #   (b) compile-only\s+seams?       — "compile-only seam(s)" present-tense
```
This line contains **no `\b` escape before any trigger word**. It is safe because Pattern 27 (`\b(?:exposes?|is\s+a|are)\s+compile-only\s+seams?`, `:429`) and Pattern 28 (`\b(?:are|is)\s+(?:currently\s+)?compile-only`, `:440`) both require an `exposes|is a|are` verb immediately preceding `compile-only`, and neither occurrence on `:259` has one. **Different mechanism entirely.**

**Why it matters:** Task 4's whole point is to teach the implementer the *match-level* safety mechanism and warn "**Do NOT 'clean up' these lines by removing the `\b` escapes**". An implementer generalising the stated mechanism to `:259` could conclude a non-existent `\b` is load-bearing there, or conversely could safely-but-incorrectly reason about other sibling lines (e.g. `:260`, which is also safe by verb-prefix absence, not by `\b`).

**Fix:** *"Sibling line `:213` is already safe by the same `\b`-escape mechanism. Sibling lines `:259` and `:260` are safe for a different reason — Patterns 27/28 require an `exposes|is a|are` verb immediately before `compile-only`, which is absent — so they need no `\b` guard. All three are deliberately left unchanged."*

---

### F-W86S-P20-012 — LOW — STORY-183 — (pending intent verification)

**Locus:** `STORY-183.md:963` (Task 5) and `:826` (EC-005).

**Condition:** Task 5 scopes the multi-line→single-line conversion to BAD_CASES only: *"Apply this conversion to all ~40 multi-line fixtures in **BAD_CASES**."* EC-005 likewise: *"Resolution: convert each **such** fixture to single-line format."*

**Evidence [own-verify]:** `GOOD_CASES` at `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py:332-632` contains **47** entries (tuple closers at `:338`…`:631`), of which ~46 use the same `"""\` multi-line form with `//`-prefixed content lines. Total `^\s*//` lines in the file: **85** (40 BAD + ~45 GOOD).

Post-story these 46 lines remain comment-shaped source lines in a scanned file. I verified **[own-verify]** that none matches any of the 36 post-story patterns (zero hits in `bin/` for all 8 new TIER-1 tokens; and GOOD_CASES are by construction non-matching for the 28 existing ones). So there is **no live defect**.

**Why report it:** DF-SIBLING-SWEEP-001 names "one-fixed/one-missed pairs inside a single file" as its target shape. The story converts one half of a structurally identical pair and is silent about the other half, leaving 46 latent self-flag surfaces for any future pattern addition (the story itself queues one: `unimplemented!()`, `:1285-1286`). Per the intent-adjudication rule I do not assert this should be fixed — leaving GOOD_CASES multi-line may be deliberate, since a GOOD_CASE that matches a pattern is by definition a test-design error caught by the self-test.

**Fix (minimum):** add one sentence to Task 5 or EC-005: *"GOOD_CASES (47 entries, ~46 multi-line) are deliberately NOT converted — a GOOD_CASE that matches any pattern is a test-design error the self-test already catches. Note the residual: 46 `//`-prefixed source lines remain in the scanned file and must be re-checked whenever a new pattern is appended."*

---

### F-W86S-P20-013 — LOW — STORY-182

**Locus:** `STORY-182.md:1323` (FSR Notes cell) and `:1037-1041` (Task 7, bullet 2).

**Claim:** "update module docstring :10-13 (false-green claim at **:11-12** must be rewritten — 'This keeps CI green without fixtures' is FALSE post-story…)".

**Evidence [own-verify]** `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs`:
```
10	//! Captures live in `tests/fixtures/local-samples/` (gitignored — see E2E-PCAPS.md). When
11	//! that directory is absent or a specific fixture file is missing, the affected test prints a
12	//! skip notice and returns immediately. This keeps CI green without fixtures while still
13	//! failing loudly (assertion-level) when fixtures are present. `#[ignore]` is NOT used.
```
The false-green sentence begins on `:12` and ends on `:13`. Line `:11` contains no part of it. Correct anchor is `:12-13`.

The outer range `:10-13` is correct, and the quoted text is exact, so an implementer can locate the target — hence LOW. Not covered by the accepted "approximately `:NNN`" residual, because these two loci state `:11-12` without a hedge.

**Fix:** `:11-12` → `:12-13` at both loci.

---

### F-W86S-P20-014 — NIT — STORY-183

**Locus:** `STORY-183.md:552` (Pattern 33 in-source comment prescribed for `bin/check-green-doc-tense`).

**Claim:** `# Pattern fires when "through" is absent between the verb and the destination phrase.`

**Evidence:** the prescribed regex two lines below (`:554`) is `re.compile(r"falls\s+to\s+the\s+wildcard", re.IGNORECASE)`. It fires only on that exact four-token phrase. It does **not** fire on "falls to the default arm", "falls to `_`", "fell to the wildcard", or any other "through"-free variant. The stated firing condition is far broader than the regex.

**Fix:** *"Pattern fires only on the contiguous phrase `falls to the wildcard`; the TIER-2 form `falls through to` is not matched because the interposed `through` breaks the `falls\s+to` adjacency."*

---

### F-W86S-P20-015 — LOW — STORY-182

**Locus:** `STORY-182.md:56-62` (Narrative, "So that" clause).

**Claim:** "the wave-85 gate G1 initial FAIL (D-510, PG-W85-005) **is structurally prevented** for the IEC-104 harness delivered here".

**Evidence — the story's own Notes contradict the prevention framing** (`STORY-182.md:1393-1401`):
> "**Wave-85 gate G1 retrospective:** D-510 was triggered on a **fixture-bearing host** where `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` ran with the stale assertion of 31 findings when the correct count … is 66. This was a **stale-assertion failure, not a clean-worktree silent-skip**."

And within the same Narrative sentence: "the wrong-fixture-content class **remains the implementer's obligation** to prevent via accurate expectations".

D-510 was a *loud* FAIL of a test that *did* run. What STORY-182 delivers is earlier and universal **detection** of that class (the committed fixture makes the test run in CI on every push rather than only on fixture-bearing hosts), plus structural prevention of the *silent-skip* class. It does not prevent a stale expectation from failing a gate.

**Why LOW not higher:** the colon-introduced explanation immediately reframes the claim as "a stale-expectation failure now fails CI on every run", which is accurate, and v1.4 already removed a harder "cannot recur" overclaim (`F-011`). The residual imprecision is the word "prevented" applied to a failure class the story explicitly does not prevent.

**Fix:** *"…the clean-worktree silent-skip class behind PG-W85-005 is structurally eliminated for the IEC-104 harness delivered here, and the D-510 stale-expectation class is now detected on every CI run rather than only on fixture-bearing hosts (detection, not prevention — accurate expectations remain the implementer's obligation)…"*

---

## PER-AXIS DISPOSITION

| # | Axis | Disposition |
|---|---|---|
| 1 | **Truthfulness against the repo** | **FINDINGS: F-011, F-013, F-014 (all LOW/NIT).** Otherwise remarkably clean. I independently verified ~45 discrete factual claims — file contents, line anchors, sha256 values, pathspec semantics, counts, tool behaviour, CI structure — and **every substantive claim checked out exactly**, including the 40/42-violation arithmetic, the 13+1 rename-site enumeration, the `:704`/`:871` monkey-patch anchors, the 10 `falls through to` sites, the 25 committed captures, both sha256 values against `bin/fetch-e2e-pcaps:154,157`, and the 10 top-level `src/*.rs` files. Zero-FP claims re-derived independently and confirmed. |
| 2 | **Tautological / non-discriminating predicates** | **FINDING: F-002** (`grep -c 'tests/fixtures/' E2E-PCAPS.md -ge 1` passes on baseline; 4 pre-existing occurrences). Also contributes to F-004 (`\|\| true` RED demo) and F-005 (`echo "Exit code: $?"`). **Cleared on re-examination:** Task 8's "M MUST equal 4 (the literal)" is discriminating (fires when the manifest legitimately grows with const+assert co-updated) — I initially proposed this as tautological and dropped it in self-validation iteration 2. AC-182-005's needle-count assertion is non-vacuous (count 4 today; the `concat!` split provably prevents self-match). AC-183-001's `p.parent.name == "src"` check is discriminating (fails if `src/*.rs` is dropped). |
| 3 | **Verifiability (both directions)** | **FINDINGS: F-003** (deliverable→AC: ci.yml deliverables in both stories have no AC) and **F-004** (subjective/unfailable success condition on the sole `tdd_mode: strict` RED evidence). AC→Task direction: **clean** — every AC in both stories maps to at least one Task (S-182: 001→T3/4/5/6, 002→T1/2, 003→T3/5, 004→T6/9, 005→T6/8/9, 006→T2/7/8/10; S-183: 001→T2, 002→T3/4/5, 003→T6/7, 004→T6/7, 005→T11, 006→T3/7/8a, 007→T8a/8b, 008→T8a, 009→T13). |
| 4 | **False-GREEN / silent-skip risk** | **FINDINGS: F-004, F-005, F-006.** Positive result worth recording: the additive ci.yml step spec (`STORY-182.md:1296-1301`) is **correctly designed** — `set -euo pipefail` first, `tee` to file, then two greps on the file (`Fixture coverage: [1-9][0-9]*/[0-9]+` **and** `test result: ok`), with the runtime-computed N and the explicit note that "manifest prints coverage BEFORE asserts, so a failing run can still write '4/4' — second grep prevents false-GREEN". This satisfies the CI-as-Code positive-coverage-assertion axis in full. Also correct: all five `grep -c` sites use the gating `test "$(grep -c …)" -eq 0` form, not the inverted bare `grep -c`. |
| 5 | **Self-referential-flag hazard** | **FINDING: F-009** (Task 9 `#`-content fixture, no single-line mandate). I evaluated **every** prescribed in-source comment line in both stories at match level against all 36 patterns: all 12 new BAD_CASES, all 14 new GOOD_CASES, the 5 reworded GOOD annotations, the 3 TIER-2 annotations, the efficacy annotation, the suffix-scoping annotation, both Task 4 replacements, and all 8 new tuple comment blocks. **All are match-safe** — the story's "describe without quoting" discipline holds, the `\b`-escape mechanism for `:258`/`:261` is provably correct, and every prescribed fixture is single-line so its source line strips to `"` not `//`/`#`. F-009 is the single residual, and it is in the one place the story prescribes `#`-prefixed *content* rather than a `#` *comment*. |
| 6 | **Sibling-sweep completeness (DF-SIBLING-SWEEP-001, CRITICAL)** | **FINDINGS: F-001** (v2.5 fixed 2 of 3 `_find_repo_root`-attribution loci), **F-005** (v2.3 fixed the inverted-gate class at AC-183-009 only), **F-006** (v2.9 fixed 2 of 5 `set -euo pipefail` block heads), **F-007** (P17/P18/v2.9 each edited the same paragraph without reconciling), **F-012** (BAD_CASES converted, GOOD_CASES not, unacknowledged). All are one-fixed/one-missed shapes inside a single file. **Cleared:** the deferred sibling e2e harnesses (`enip_e2e_real_pcaps_tests.rs` 7 sites, `e2e_corpus_smoke_tests.rs:206-224`, `bc_2_12_011_story127_tests.rs`) are correctly characterised and are accepted residuals — no NEW instance found. `tests/fixtures/mk_modbus_*.py` and `fuzz/seed_corpus.py` correctly out of scope (EC-010 + `DRIFT-py-surface-outside-bin` confirmed present at `STATE.md:224`). |
| 7 | **Internal consistency and contradiction** | **FINDINGS: F-001** (Task 2 vs AC-183-001 vs Task 9) and **F-007** (blocking-and-enforceable AND evidence-only, in one paragraph, twice). Both are exactly the shapes this axis names. **Clean:** frontmatter ↔ body ↔ AC traces ↔ Tasks ↔ ACR verified consistent in both stories; `behavioral_contracts: []` matches the "(none — E-11 convention)" body sections; every `traces_to` entry exists; ACR rules do not contradict any AC; the `src/*.rs`-subsumes-`src/**/*.rs` semantics are stated identically at all 8 loci with no residual inversion; STORY-INDEX ↔ story H1/points/status/version sync is exact. |
| 8 | **Environment-blindness** | **FINDING: F-008** (`test -f .factory/…` holds only in the main repo root; `.factory/` gitignored at `.gitignore:4`; file explicitly excluded from the develop PR). Otherwise **strong** — the Env A / Env B two-environment protocol, the `if [ -d tests/fixtures/local-samples ]` guards at all three move-aside sites, the "WITHOUT-LOCAL-SAMPLES PRECONDITION" annotations, the `--nocapture`-vs-CI stdout partition, and the "on a fixture-bearing host the output will show 4/4, not 1/4" caveats are all correct and correctly scoped. |
| 9 | **Arithmetic and enumeration** | **FINDING: F-010** ("four governance deliverables" vs 5 bullets / 6 predicates). Everything else checks out: 12 BAD + 14 GOOD = 26 (Task 7's 4+3 and Task 8a's 8+11 sum exactly); 28 + 8 = 36 tuples; 37 docstring items / 36 tuples (item 5 shares tuple 4 — verified: docstring items 4/5 both map to `RED GATE:.*tests must fail`); 13 + 1 = 14 rename sites (6 functional + 7 prose + `:721`); 40 `//` + 2 `#` = 42; 1/4 and 4/4 coverage arithmetic; 10 `falls through to` sites; 10 top-level `src/*.rs` files (31−21); 25 committed captures; 66 = T0836×20 + T1692.001×46; 4 pts + 5 pts = 9 (STORY-INDEX `:456`); 6 CLAUDE.md protocol rows. |
| 10 | **Scope integrity** | **FINDING: F-015** (Narrative "structurally prevented" broader than the delivered mechanism). Otherwise clean: STORY-182's "No `src/` changes, no `bin/` changes, no `Cargo.toml` changes" and CHANGELOG non-obligation are both correct against the ci.yml `changelog-gate` trigger set (`ci.yml:524` → `'^(src/\|Cargo\.toml$\|bin/)'`); STORY-183's CHANGELOG obligation is correctly asserted; residual-exclusion lists (bare tokens, `unimplemented!()`, extension-less `bin/` executables, `.py` outside `bin/`, contiguity blind spot) are accurate and each maps to a real STATE.md drift row. |
| 11 | **CI/gate realism** | **CLEAN.** The additive step is executable as written and coherently placed (`test` job at `ci.yml:40-47`, after `- run: cargo test --all-targets` at `:47`). `if: ${{ !cancelled() }}` is valid GitHub Actions expression syntax. The `--exact` filter `iec104_e2e_real_pcaps::test_fixture_manifest_report` is the correct libtest path for a test inside `mod iec104_e2e_real_pcaps` in an integration target. GitHub's default Linux shell is `bash -e {0}` **[supplied #12]**, and the block additionally sets `set -euo pipefail`. The `!cancelled()` visibility claim is correctly scoped ("guarantees execution, not output"; not visible after compile failures). No step-success claim is made via the presence of an output string — no `steps.<id>.outcome` misuse. STORY-183's ci.yml edits are comment/step-name-only, so the `action-pin-gate` and `changelog-gate` jobs are genuinely unaffected. The declared disjointness of the two stories' ci.yml edit regions (`:40-47` vs `:434`/`:442`/`:462`) is verified correct. |

**EXECUTION-REQUIRED — evidence supplied, axis satisfied:** `bin/test_lint_cycle_artifact.py` 21/21 (#1), `bin/test_compute_input_hash.py` 9/9 (#2), `bin/check-green-doc-tense` exit 0 / 114 files (#3), canonical input-hash MATCH ×2 (#4), git pathspec cardinality 31/21 (#5), tool line anchors (#6), `bin/*.py` = 6 (#7), CHANGELOG historical loci (#8), fixture-idiom census (#9), local-samples absent (#10), ci.yml anchors + absent strings (#11), toolchain/shell (#12).

**EXECUTION-REQUIRED — evidence NOT supplied (no axis silently skipped):**
- `cargo test --all-targets` / `cargo clippy --all-targets -- -D warnings` on `develop @ e8841d76`. Not needed for this pass — both stories are unimplemented drafts with no code to compile. I reviewed the prescribed Rust for type-correctness by inspection only (see Methodology table); **an implementer must still compile it**.
- `python3 bin/test_check_green_doc_tense.py` post-remediation. Not applicable pre-implementation.
- `git ls-files tests/fixtures/ | grep -cE '\.(pcap|pcapng|cap|trace)$'`. I corroborated the value **25** via `Glob` of the working tree, which is index-equivalent here because `.gitignore` covers only `local-samples/` and that directory is absent. Treat as [own-verify, filesystem-derived], not as git-index output.

---

## NOVELTY ASSESSMENT

Inferred from the stories' internal changelog tables (v1.0 → v2.9, 20 rows each) since I cannot read prior pass reports.

| Finding | Class | Basis |
|---|---|---|
| F-001 | **(c) induced by a prior remediation** — and (b) recurrence | v2.5 `F-W86S-P15-005` fixed "two loci" of the `_find_repo_root`-attribution error and left the Task 2 sibling. Both a partial-fix residue and an instance of the sibling-sweep class. |
| F-002 | **(c) induced by a prior remediation** | AC-182-006 was created in v2.9 (`F-004`). A brand-new tautology introduced one pass after v2.8 `P18-006` removed a different one — the pattern class was live in the author's attention and still recurred. |
| F-003 | **(a) genuinely new** | No changelog row in either story addresses AC-coverage of ci.yml deliverables. STORY-183's ci.yml row has existed since v1.1 (`F-009 MED (ci.yml sibling-prose sweep)`) with no AC in 19 passes; STORY-182's since v1.4 (`F-014`). A 19-pass blind spot. |
| F-004 | **(a) genuinely new** | v2.3 `F-W86S-P13-008` added the `\|\| true` *for restore-safety* and v2.4 `F-W86S-P14-006` added the pre-existence guard — both hardened the *housekeeping* of this block while leaving its *predicate* absent. The predicate gap itself appears never to have been raised. |
| F-005 | **(b) recurrence of a pattern class** | Same inverted/non-gating-verification class as v2.3 `F-W86S-P13-004` (AC-183-009) — fixed there, never swept to AC-183-002. |
| F-006 | **(b) recurrence, 5th consecutive pass** | v2.5 `F-W86S-P15-003`, v2.6 `F-W86S-P16-001/002`, v2.7 `P17-001/002`, v2.9 `F-002` all remediated this exact class. Severity is decaying (HIGH→MED) but the class is not converging — 3 loci survive after a sweep that reported 2. |
| F-007 | **(c) induced by a prior remediation**, 3-pass accumulation | v2.7 `P17-004a` (→ evidence-only) then v2.8 `P18-006` (→ blocking) then v2.9 `F-003` (→ refined blocking) each edited the paragraph; none removed the superseded clause. Classic accumulate-don't-replace residue. The v2.9 "whole-region rewrite discipline (D-536)" recorded in STORY-INDEX `:11` was evidently not applied to this region. |
| F-008 | **(a) genuinely new** | AC-182-006 is one pass old; its factory-artifacts branch predicate has not previously been examined for environment scope. |
| F-009 | **(a) genuinely new** | The `#`-eligibility mechanism landed in v1.3 (`F-008`, `_is_comment_line` suffix scoping) and the Task 9 hermetic test in v1.4 (`F-010`). The *interaction* — that Task 9's `#`-content string is a new self-flag class not covered by Task 5's `//`-only conversion mandate — appears in none of the 19 changelog rows. This is the highest-value new finding: it is the fifth-recurrence class (`self-referential predicate`, v2.2 `F-W86S-P12-001`) surfacing in a new syntactic guise. |
| F-010 | **(c) induced by a prior remediation** | AC-182-006's heading is v2.9-new. |
| F-011 | **(c) induced by a prior remediation** | v2.9 `F-007` rewrote Task 4's safety criterion from phrase-level to match-level and added the sibling-lines sentence; the mechanism over-generalisation entered with that rewrite. |
| F-012 | **(b) recurrence of a pattern class** | The BAD/GOOD asymmetry is structural and dates to v1.2 (`F-011`, "scope 40 //-lines + 2 # lines = 42"). Not previously framed as a sibling gap. |
| F-013 | **(a) genuinely new** | The `:11-12` anchor entered in v2.9 (`F-001`). |
| F-014 | **(c) induced by a prior remediation** | v2.3 `F-W86S-P13-003` replaced a *wrong* discriminator with this *over-broad* one; v2.4 `F-W86S-P14-004` then reworded the block for line-wrap safety without revisiting the claim. |
| F-015 | **(b) recurrence, decaying** | v1.4 `F-011` already softened this Narrative ("'cannot recur' overclaim removed"). The residual "structurally prevented" is the tail of that same class. Genuinely low-value; reported for completeness under axis 10. |

**Novelty verdict:** **MEDIUM-LOW.** Nine of fifteen findings are recurrences (b) or remediation-induced (c) — the story pair is deep in the diminishing-returns regime and the dominant failure mode is now *accumulate-don't-replace* editing plus incomplete sweeps, not substantive spec gaps. However this pass produced **four genuinely-new (a) findings** (F-003, F-004, F-008, F-009), two of which (F-003, F-009) are structural and would not have been found without fresh re-derivation: F-003 is a 19-pass blind spot, and F-009 is a new syntactic guise of the class that already recurred five times. This is consistent with the Fresh-Context Compounding Value principle and argues against declaring convergence yet.

**Not-converged mechanism:** the recurring meta-pattern across passes 15–20 is that each remediation burst sweeps the *cited* loci plus a self-selected subset, reports a count, and misses the remainder (v2.9 reported "two missing `set -euo pipefail`" against five candidate blocks; v2.5 reported "two loci" against three). Until the sweep is mechanised, this class will keep consuming passes.

---

## `[process-gap]` TAGGING

- **F-W86S-P20-001** `[process-gap]` — sibling-sweep escape. DF-SIBLING-SWEEP-001 `enforcement` clause: *"Adversaries should flag any sibling-regression that escapes the sweep as a process-gap with a pointer to DF-SIBLING-SWEEP-001."*
- **F-W86S-P20-003** `[process-gap]` — no template or agent-prompt rule requires that every Architecture-Mapping / FSR row map to at least one checkable AC predicate. Produced two independent instances in one wave, and one of them (STORY-183's ci.yml row) has survived 19 passes.
- **F-W86S-P20-005** `[process-gap]` — same non-gating-verification class as F-006; sweep escape.
- **F-W86S-P20-006** `[process-gap]` — **primary process gap of this pass.** Five consecutive passes have manually remediated "story-spec bash verification block is non-gating" and the class has not converged. There is no hook, linter, or checklist step that mechanically checks fenced `bash` blocks in `.factory/stories/*.md` for a head `set -euo pipefail`. **Recommended codification:** add a `bin/lint-story-bash-blocks` selftest-backed checker (asserting every multi-command ```` ```bash ```` fence begins with `set -euo pipefail`, and flagging `\|\| true` and `echo "…$?"` as non-gating idioms), wire it into the `bin-selftest` job alongside the pending PG-W84-012 (D-525) work, and reference it from DF-SIBLING-SWEEP-001's "ALL remediations" grep checklist.

Note for the Cycle-Closing Checklist: F-006's codification candidate overlaps the already-pending PG-W84-012 (D-525) `bin-selftest` ops task (`STATE.md:256` carry-forward (a)) — batching them is likely cheaper than two devops dispatches.

---

## VERDICT

**NOT_CONVERGED**

**Tally: 15: 0C / 0H / 9M / 5L / 1N**

**Rationale.** Zero CRITICAL and zero HIGH for the eighth consecutive pass — the substantive spec content of both stories is in genuinely good shape, and the factual-accuracy axis is the cleanest I can construct against this repo (roughly 45 discrete claims verified exact, including every sha256, every line anchor, every count, and the full zero-false-positive analysis re-derived independently). The mis-anchoring axis is **clean**: every module/package name, file path, test-function name, BC/policy/AC reference, and STORY-INDEX row resolves to a real workspace artifact with matching content.

But NITPICK_ONLY requires zero MEDIUM, and nine MEDIUMs stand — none of them cosmetic:

- Two are **internal contradictions** that would actively mislead an implementer (F-001 mis-attributes the monkey-patch hazard; F-007 gives two opposite answers for whether `N < 1` blocks gate entry).
- Three are **predicates that cannot fail** (F-002 tautology; F-004 the sole `tdd_mode: strict` RED evidence; F-005 the hard zero-FP requirement's own verification) — each would pass with its underlying work entirely skipped.
- One is a **19-pass structural blind spot** (F-003: ci.yml deliverables in *both* stories with no AC).
- One is a **new-guise recurrence of the class that already recurred five times** (F-009: `#`-content self-flag).
- One is an **incomplete sweep reported as complete** (F-006: 3 of 5 loci remain after a 2-locus fix).
- One is **environment-blindness in a story that is otherwise exemplary about environment declaration** (F-008).

Recommended remediation posture: apply **whole-region rewrites** (D-536) rather than clause edits for F-001, F-005, F-006 and F-007 — three of those four are precisely the accumulate-don't-replace residue that clause-level editing produced. F-002/F-003/F-008/F-010 should be fixed as a single coherent rewrite of AC-182-006, which is one pass old and already carries three of this pass's findings.
