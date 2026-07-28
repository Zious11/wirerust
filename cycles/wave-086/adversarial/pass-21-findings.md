---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-28T00:00:00Z
cycle: "wave-086"
pass: 21
verdict: NOT_CONVERGED
novelty: "low-to-moderate"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 21

**Date:** 2026-07-28
**Pass:** 21 of N

# WAVE-86 STORY-LEVEL ADVERSARIAL REVIEW — PASS 21

## Phase A Attestation

**A0. Tool-profile disclosure.**
My profile is **read-only: `Read`, `Grep`, `Glob` only. `Bash` is DENIED.** I cannot execute `git`, `cargo`, `python3`, `shasum`, or any shell. Accordingly:
- I assert **no** execution outcome of my own. Every exit code, test count, `wc -l`, `git ls-files`, or `len()` value I cite is either (a) taken from the orchestrator's SUPPLIED EXECUTION EVIDENCE (labelled **[supplied]**), or (b) derived from my own `Read`/`Grep`/`Glob` of file contents (labelled **[own-read]**).
- Grep counts below are `ripgrep` line counts over the working tree, **not** `git ls-files` output. Where an assertion is defined over the git index, I say so and rely on the supplied evidence for the index-scoped value.
- Axes fully evaluable by static reading were evaluated. **No axis is flagged EXECUTION-REQUIRED** — every predicate I judged, I judged by reading the target file plus the supplied baselines. Where a bash *semantic* claim is load-bearing (finding 002) I state it as a documented POSIX/bash property, not as an executed result.

**A1. Freshness.**
Reviewing develop at SHA **e8841d761f3f25f320f98977618e506e8b41a058** (v0.13.2 back-merge). There is NO story worktree — both stories are unimplemented drafts; all reads are from the main checkout at `/Users/zious/Documents/GITHUB/wirerust/`.

Independent tree-content corroboration that this is the post-v0.13.2 tree **[own-read]**:
- `/Users/zious/Documents/GITHUB/wirerust/CHANGELOG.md:1888` → `[Unreleased]: https://github.com/Zious11/wirerust/compare/v0.13.2...HEAD` — the comparison base is `v0.13.2`, i.e. 0.13.2 is the most recent released tag in this tree.
- `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs:27` records `T0836 ×20 + T1692.001 ×46 = 66` for the ITI diverse test — the post-STORY-180 / post-PR-#439 (`0ab6f52e`) expectation, not the pre-gate-fix 31.
- `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml:473` contains the `bin-selftest:` job (wave-74/84 era), and `:442` still reads `tests/*.rs and src/**/*.rs` — i.e. STORY-183 is **not** yet applied.

**A2. Story-version assertion.** **[own-read]**
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md:5` → `version: "2.10"` ✓
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md:5` → `version: "2.10"` ✓
No abort.

**A3. Discriminating grep-count assertions.**

| Assertion | Expected | Observed | Verdict |
|---|---|---|---|
| `fixture_present` in `tests/iec104_e2e_real_pcaps_tests.rs` | 5 | **5** (`:63` defn, `:166`, `:292`, `:383`, `:529`) **[own-read]** | ✓ |
| `fixture_present` in `tests/enip_e2e_real_pcaps_tests.rs` | 7 | **7** (`:66` defn, `:163`, `:310`, `:446`, `:542`, `:626`, `:734`) **[own-read]** | ✓ |
| git-tracked `bin/*.py` | 6 | **6** via `Glob('bin/*.py')` — `test_lint_cycle_artifact.py`, `test_compute_input_hash.py`, `test_changelog_gate_content.py`, `test_validate_citations.py`, `test_check_green_doc_tense.py`, `test_gitignore_mutants_glob.py` **[own-read]**; identical set to supplied evidence item 8 **[supplied]**. Working-tree glob ≠ index query, but the two agree exactly. | ✓ |
| `keeps CI green` in `tests/iec104_e2e_real_pcaps_tests.rs` | 2 | **2** (`:12`, `:62`) **[own-read]** | ✓ |

**A4. Factory-artifact path confirmation.** Both stories were read from `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/` — specifically `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md` (1458 lines, read in full across 4 paged reads) and `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md` (1353 lines, read in full across 4 paged reads). Supporting artifacts read from `/Users/zious/Documents/GITHUB/wirerust/.factory/policies.yaml` (19 policies confirmed by `Grep 'id: DF-'` → 19 hits at `:2,25,231,324,398,451,518,610,692,750,803,853,906,974,1045,1613,1688,1846,1922`), `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-INDEX.md`, and `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-085/STORY-180/convergence-report.md`.

---

## Methodology

1. Read both stories end-to-end (no page skipped), then re-derived every factual claim from the cited file independently rather than trusting the story's own annotation.
2. **Anchor sweep (own-read).** Verified every live line anchor in both stories against the actual file: `tests/iec104_e2e_real_pcaps_tests.rs` (`:10-13`, `:23-28`, `:39`, `:47-49`, `:51`, `:53-57`, `:59-62`, `:63`, `:90`, `:97`, `:138`, `:273`, `:353-354`, `:503-504`, `:529`) — **all 15 correct**; `bin/check-green-doc-tense` (`:4`, `:26-30`, `:31-85`, `:87-88`, `:90`, `:97`, `:192`, `:212-215`, `:217`, `:325`, `:376`, `:455`, `:460-462`, `:465-474`, `:477`, `:490`, `:502`, `:519`, `:549`, `:556`, `:557-563`, `:577`, `:581-585`, `:591`) — **all correct**; `bin/test_check_green_doc_tense.py` (`:41`, `:51`, `:91`, `:97`, `:213`, `:258-261`, `:332`, `:402`, `:640`, `:688`, `:699`, `:705`, `:707`, `:711`, `:718`, `:721`, `:726`, `:839`, `:843`, `:859`, `:872`, `:891`, `:905`, `:907`) — **all correct**; `bin/test_lint_cycle_artifact.py` (`:3`, `:5`, `:6`, `:125`) — **all correct**; `.github/workflows/ci.yml` (`:47`, `:434`, `:442`, `:462`, `:463`, `:473`, `:533`) — **all correct**; `tests/fixtures/E2E-PCAPS.md` (`:3-6`, `:48-50`, `:337-340`, `:352-359`, `:358`, `:359`, `:374-380`, `:391-396`) — **all correct**; `tests/fixtures/README.md` (`:5-34`, `:7-22`, `:24-26`, `:30-34`, `:41-44`) — **all correct**; `.gitignore:4`/`:10` — correct; `CHANGELOG.md:741`/`:851` — correct; `bin/fetch-e2e-pcaps:154`/`:157` — correct.
3. **Regex-execution-by-hand.** For axis 5 I hand-executed each of the 29 shipped `_VIOLATION_PATTERNS` regexes (read verbatim from `bin/check-green-doc-tense:217-457`) plus the 8 prescribed new regexes against **every** `#`- and `//`-prefixed line the two stories prescribe, and against every existing `#` line in all 6 `bin/*.py` files.
4. **Predicate audit.** For each `grep`/`test` predicate in both stories I computed its baseline truth value from the tree and asked (a) does it discriminate, (b) is it satisfiable by executing the Tasks as written, (c) does it fail-safe under `set -euo pipefail`.
5. **Arithmetic re-derivation.** Independently recounted the 40/42/45 fixture-line figures, 13+1 rename sites, 12/14/26 case counts, 29→37 docstring items / 28→36 tuples, 4-entry manifest, 25 committed captures, 10 `falls through to` sites, 21 TC functions.
6. Cross-read `DF-GREEN-DOC-TENSE-SWEEP` v6 in full (`policies.yaml:1045-1611`) and checked STORY-183's tier assignments token-by-token against it.

---

## Findings

| ID | Sev | Story | Locus | One-line |
|---|---|---|---|---|
| F-W86S-P21-001 | MEDIUM | 183 | `:560` | Prescribed `#` comment contains the literal phrase Pattern 33 matches — violates the story's own rule; reverses the v2.4 fix |
| F-W86S-P21-002 | MEDIUM | 182 | `:496-497`, `:1184-1185` | `SKIP_COUNT="$(grep -c …)"` aborts under `set -e` in the expected-pass case (false-RED); v2.10 replaced a working form with a broken one |
| F-W86S-P21-003 | MEDIUM | 182 | `:837` vs Task 7 `:1092-1095` | Whole-file `-eq 0` predicate needs 2 lines changed; Task 7 authorises only 1; the other is a true, out-of-scope ENIP claim → AC unsatisfiable |
| F-W86S-P21-004 | MEDIUM | 182 | Task 8 `:1153-1162` vs Task 10a `:1241` | Blocking wave-gate procedure cites a `grep -qE "test result: ok"` that is absent from the command it prescribes → false-GREEN gate entry; contradicts Task 10a |
| F-W86S-P21-005 | LOW | 182 | `:848-855` | "runs from the main repo root ONLY" vs the only supplied command being labelled "runnable anywhere the ref is fetched" |
| F-W86S-P21-006 | LOW | 182 | `:816`, `:822` | `red-out.txt` transient artifact not gitignored, although `coverage-out.txt` is, for identical stated reasons |
| F-W86S-P21-007 | LOW | 182 | `:56-58` | Narrative scopes the silent-skip elimination by *harness*, not by *partition*; 3 of 4 gated tests still return early + report `ok` |
| F-W86S-P21-008 | LOW | 182 | Task 7 `:1096-1097` vs `:836` | AC requires an exact literal that Task 7 supplies only as an "e.g." example |
| F-W86S-P21-009 | LOW | 183 | ci.yml `:436` (unswept) | Sibling "test files" prose in the same comment block as the 3 swept loci; not swept, not caught by the AC predicate *(pending intent verification)* |
| F-W86S-P21-010 | NIT | 183 | `:986` | "~46" GOOD_CASES residual lines; actual is 45 |

---

### F-W86S-P21-001 — MEDIUM — self-referential-flag regression in the Pattern 33 comment

**Story / locus:** STORY-183 `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md:560` (inside AC-183-007's prescribed `_VIOLATION_PATTERNS` block for `bin/check-green-doc-tense`).

**Defective text (prescribed `#` comment line):**
```
      # Pattern fires only on the contiguous phrase `falls to the wildcard`; the TIER-2 form
```

**Defective claim:** The story mandates, in three separate places, that no in-source `#` comment may contain a literal flagged phrase — and asserts the mechanism is match-level, not phrase-level:
- Background §Scope of `bin/*.py` glob, `:170-172`: "**In-source `# Pattern NN:` comments** in `bin/check-green-doc-tense` must use non-flagging wording (not containing the literal flagged phrase) so they do not trigger the gate if the tool is later included in its own scan."
- Task 6 `:998` and Task 8b `:1032-1034`: "In-source `# Pattern NN:` comments MUST NOT contain the literal flagged phrase (quoting does not prevent regex match — see Task 4)."
- AC-183-007 §No-literal-phrase sweep obligation `:688-695`: "No annotation line in the fixture block above may match any of the 36 patterns."

Line `:560` violates all three.

**Evidence (own-read, hand-executed):**
- Pattern 33's own regex, as prescribed at STORY-183 `:564`: `re.compile(r"falls\s+to\s+the\s+wildcard", re.IGNORECASE)`.
- Line `:560` contains the contiguous byte sequence `falls to the wildcard` (the surrounding backticks are outside the match span; `\s+` matches the single spaces). The regex **matches**. Quoting/backticking does not prevent a regex match — the story itself states this at Task 4 `:947-950`.
- Contrast: every *other* new-pattern comment I hand-checked is clean. Pattern 30's block (`:341-344`) contains no `Expected RED:`; Pattern 31's block (`:413-417`) never places `currently` adjacent to `falls` (`# Word "currently" is the discriminator` → next char is `"`, not whitespace+`fall`); Pattern 35's block (`:573-577`) was already scrubbed of `currently has no` at v2.6. So `:560` is the sole outlier.
- **This is a regression of a previously-closed finding.** STORY-183's own changelog, v2.4 row (`:1338`): "F-W86S-P14-004 LOW (Pattern 33 comment `:539-540` reworded: **eliminated contiguous 'falls to the wildcard'** across line wrap — new 5-line comment describes discriminator **without placing 'falls' adjacent to 'to the wildcard' on any single line**)." The v2.10 row (`:1332`) then records: "F-014 LOW (AC-183-007 Pattern 33 comment: … replaced with '**Pattern fires only on the contiguous phrase `falls to the wildcard`**; …')". The pass-20 remediation re-introduced exactly the text pass-14 removed.

**Why it matters despite the tool being extension-less:** `bin/check-green-doc-tense` has no `.py` suffix and is therefore outside `git ls-files -- bin/*.py`, so the gate does not scan itself today (STORY-183 `:166-169`, EC-010). The defect is therefore *latent*, not gate-breaking. But (a) it is a direct, mechanically-verifiable violation of a rule the story declares mandatory three times, (b) it defeats the exact contingency the rule was written for ("so they do not trigger the gate **if the tool is later included in its own scan**"), and (c) it is the 6th recurrence of the self-referential-predicate class (per STORY-INDEX `:19`, the class already produced a HIGH at pass-12). Silently re-introducing a closed defect in a discipline the story itself elevates to a standing rule (D-529) is a MEDIUM.

**Prescribed fix:** Restore the v2.9 (pass-14-compliant) wording — describe the discriminator without placing `falls` adjacent to `to the wildcard` on any single line. E.g. replace `:560-562` with:
```
      # Pattern fires only on the contiguous four-token phrase (verb + `to the wildcard`
      # with no intervening word); the TIER-2 form `falls through to` is not matched
      # because the interposed `through` breaks the verb→`to` adjacency.
```
Then re-run the AC-183-007 `:688-695` sweep obligation over the *whole* prescribed block, and add a standing check that every future edit to a `# Pattern NN` comment is re-verified against the pattern it documents.

---

### F-W86S-P21-002 — MEDIUM — `SKIP_COUNT="$(grep -c …)"` aborts under `set -euo pipefail` in the expected-pass case

**Story / loci:** STORY-182 `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md:496-497` (AC-182-003 Verification) and `:1184-1185` (Task 9 Environment A). Both fences open with `set -euo pipefail` at `:482` and `:1171` respectively.

**Defective code (identical at both loci):**
```bash
test -s coverage-out.txt
# SKIP-count check (non-vacuous — file existence asserted above):
SKIP_COUNT="$(grep -c '\[iec104-e2e\] SKIP:' coverage-out.txt)"
test "${SKIP_COUNT}" -eq 0
# Expected: 0 SKIP lines (committed fixture always found; pattern catches ANY iec104-e2e SKIP)
```

**Defective claim:** that this block passes when zero SKIP lines are present. It cannot.

**Evidence:**
- `grep -c` exits **1** when the count is zero (it still prints `0` on stdout, but the exit status signals "no lines selected"). This is POSIX-specified `grep` behaviour, not implementation-specific.
- A shell command consisting **only** of a variable assignment is a simple command with no command name. POSIX: "If there is no command name … the exit status of the command shall be the exit status of the last command substitution performed." Bash implements this, and `set -e` fires on it. So `SKIP_COUNT="$(grep -c … )"` **terminates the script with status 1 precisely in the success case** (zero SKIP lines) — before `test "${SKIP_COUNT}" -eq 0` ever runs.
- The declared expectation at `:498` and `:1186` is `0`. Therefore the block reports FAILURE exactly when the story's postcondition holds, and would only "succeed" if at least one SKIP line were present — at which point `test … -eq 0` fails. **The block cannot pass in any state.**
- **This is remediation-induced.** STORY-182 v2.10 changelog `:1437`: "AUDIT-2 (two tautological SKIP-count predicates at AC-182-003 Verification and Task 9 Env A fixed: `test "$(grep -c ...)" -eq 0` **replaced with** `test -s coverage-out.txt` guard **+ variable-assignment form** …)". The pre-v2.10 form `test "$(grep -c …)" -eq 0` was **immune** to this problem: with a command name (`test`) present, the command's own exit status governs and the substitution's failure is discarded. The v2.10 change traded a (correctly-diagnosed) vacuity problem for a hard false-RED.
- **Shape not covered by Audit 14.** The orchestrator's mechanical audit inspects `test "$(grep -c …)" <op> N`. This locus is `VAR="$(grep -c …)"` followed by `test "${VAR}" <op> N` — outside the audit's pattern. This is precisely the "shape it does NOT cover" the dispatch invited. I confirmed the audit's own target shape is safe: STORY-182 `:837`, `:866`; STORY-183 `:270`, `:786-788`, `:804-806` all use `test "$(...)"` and are unaffected.

**Prescribed fix:** Keep the `test -s coverage-out.txt` non-vacuity guard (it is the correct part of the v2.10 fix) and make the count read failure-tolerant, or invert to a negated `grep -q`:
```bash
test -s coverage-out.txt
! grep -q '\[iec104-e2e\] SKIP:' coverage-out.txt
```
(or `SKIP_COUNT="$(grep -c '\[iec104-e2e\] SKIP:' coverage-out.txt || true)"`). Apply at **both** loci in the same burst.

---

### F-W86S-P21-003 — MEDIUM — AC-182-006's whole-file E2E-PCAPS.md predicate is unsatisfiable by executing Task 7 as written

**Story / locus:** STORY-182 `:836-838` (AC-182-006), against Task 7's E2E-PCAPS.md sweep list at `:1086-1104`.

**Defective predicate (`:837`):**
```bash
test "$(grep -c 'All are auto-fetchable via `bin/fetch-e2e-pcaps`' tests/fixtures/E2E-PCAPS.md)" -eq 0
```

**Evidence (own-read):** `grep -c` counts matching **lines**. `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/E2E-PCAPS.md` contains the exact literal on **two** lines:
- `:279` — `E2E validation of the ENIP analyzer (SS-17). All are auto-fetchable via \`bin/fetch-e2e-pcaps\`.`
- `:340` — `analyzer (SS-19). All are auto-fetchable via \`bin/fetch-e2e-pcaps\`.`

(Confirmed by `Grep 'auto-fetchable'` which returned exactly 5 lines — `:33`, `:54`, `:64` use the shorter "All are auto-fetchable." form and do not match the literal; only `:279` and `:340` match.)

Task 7's sweep list (`:1086-1104`) enumerates six E2E-PCAPS.md loci: `:3-6`, `:48-50`, `:337-340`, `:352-359`, `:374-380`, `:391-396`. **`:279` is not among them.** The Architecture Mapping row (`:880`) and the FSR row (`:1356`) enumerate the same six loci. So executing the story as written changes `:340` only → the predicate returns `1`, and AC-182-006 fails.

Worse, satisfying the predicate literally would require editing `:279`, and that edit would make the document **less** accurate: no ENIP capture is committed by this story (STORY-182 is IEC-104-scoped by design; the ENIP harness is the explicitly deferred `DRIFT-e2e-sibling-harnesses` sibling per Notes `:1393`), so `:279`'s "All are auto-fetchable" remains **true** for the ENIP section. The story would be forcing an out-of-scope, factually-wrong edit to satisfy its own gate.

This is remediation-induced: v2.10 F-002 (`:1437`) introduced this predicate to replace a tautological one. The replacement is correctly *discriminating* (baseline 2 ≠ 0) but is scoped to the whole file while the change is section-scoped.

**Prescribed fix:** Scope the predicate to the IEC-104 section, e.g. replace `:837` with a positive assertion on the rewritten `:337-340` text plus a bounded negative:
```bash
grep -qF 'is committed directly in `tests/fixtures/`' tests/fixtures/E2E-PCAPS.md
test "$(sed -n '337,345p' tests/fixtures/E2E-PCAPS.md | grep -c 'All are auto-fetchable via `bin/fetch-e2e-pcaps`')" -eq 0
```
or state the whole-file expected count as `-eq 1` with an inline note naming `:279` as the ENIP-section occurrence that MUST remain. Either way, add an explicit "do NOT edit `:279`" directive to Task 7 so the intent is unambiguous.

---

### F-W86S-P21-004 — MEDIUM — Task 8's blocking wave-gate procedure cites a check absent from the command it prescribes (false-GREEN vector), and contradicts Task 10a

**Story / loci:** STORY-182 `:1153-1162` (Task 8, "Enforceable wave-gate obligation (F-026)") vs `:1241` (Task 10a, CLAUDE.md Project References row).

**Defective text (`:1153-1162`, verbatim):**
```
- **Enforceable wave-gate obligation (F-026):** Before G1 of any wave-gate evaluation that
  includes e2e pcap tests, run:
  `cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture`
  and record the printed N/M in the gate entry. **`M ≠ 4` blocks gate entry** … 
  committed-capture absence is blocked by the AC-182-005 hard-assert, which fails
  `cargo test` and therefore fails the `grep -qE "test result: ok"` check in this same
  command — `N` itself is not a blocking datum.
```

**Defect 1 — the cited check does not exist in the prescribed command.** The command is a bare `cargo test … --nocapture`. There is no `tee`, no pipe, and no `grep -qE "test result: ok"`. Every other block in the story that relies on that grep constructs it explicitly (`:358-360`, `:549-551`, `:569-571`, `:1201-1203`, `:1223-1225`, ACR `:1328-1330`). Task 8 asserts the mitigation is "in this same command" when it demonstrably is not.

**Defect 2 — this is a live false-GREEN vector, by the story's own reasoning.** STORY-182 `:1206-1208` states, correctly: "manifest prints coverage BEFORE asserts, so **a failing run can still write '4/4'** — second grep prevents false-GREEN". Confirmed against the prescribed implementation: `test_fixture_manifest_report()`'s `println!("Fixture coverage: …")` block (`:309-319`) precedes the entire hard-assert partition (`:619` onward). Therefore a gate operator who follows Task 8 literally — run the bare command, "record the printed N/M" — will read `Fixture coverage: 1/4` off stdout from a **failing** run (committed capture absent → hard-assert fires), find `M = 4`, apply the stated rule "`M ≠ 4` blocks gate entry", conclude *not blocked*, and record `N = 1` as "evidence only". Gate entry proceeds on a broken checkout. The only thing standing between the operator and that outcome is cargo's exit status, which Task 8 never instructs them to check — it instead points them at a grep that isn't there.

**Defect 3 — Task 8 and Task 10a state different mechanisms.** Task 10a `:1241` says: "committed-capture absence is blocked by the AC-182-005 hard-assert (**cargo test failure**) — that assert is the blocking gate". That is correct and grep-free. The two loci, which v2.10 F-007 (`:1437`) explicitly rewrote *together* to resolve an earlier contradiction, now disagree on the enforcement mechanism. This is the Task 8 / Task 10a pair the dispatch names as historically affected.

**Prescribed fix:** Make Task 8's prescribed command self-gating so the stated mechanism is real:
```bash
set -euo pipefail
cargo test --test iec104_e2e_real_pcaps_tests \
  iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
grep -qE "test result: ok" coverage-out.txt          # blocks on hard-assert failure
grep -qE "Fixture coverage: [0-9]+/4" coverage-out.txt   # M must be 4
```
and reword `:1160-1162` to: "committed-capture absence is blocked by the AC-182-005 hard-assert, which fails `cargo test`; the `grep -qE "test result: ok"` in the command above is what surfaces that failure — a non-zero `cargo test` exit blocks gate entry regardless of the printed N/M". Sweep Task 10a `:1241` so both loci name the same single mechanism.

---

### F-W86S-P21-005 — LOW — AC-182-006 factory-artifacts block contradicts itself within four lines

**Story / locus:** STORY-182 `:848-855`.

**Defective text:**
```
- And `.factory/maintenance/fixture-count-gate-entry.md` exists on the factory-artifacts branch
  (this predicate runs from the main repo root ONLY — `.factory/` is gitignored at
  `.gitignore:4` and absent from a story worktree or CI develop checkout):
  ```bash
  # Environment: main repo root ONLY (factory-artifacts mounted at .factory/).
  # Branch-scoped alternative (runnable anywhere the ref is fetched):
  git cat-file -e factory-artifacts:maintenance/fixture-count-gate-entry.md
  ```
```

**Evidence:** The prose and the in-fence comment both restrict execution to "main repo root ONLY", and the stated reason is that `.factory/` is gitignored (`/Users/zious/Documents/GITHUB/wirerust/.gitignore:4` → `.factory/` ✓ own-read). But the **only** command supplied is the one labelled "Branch-scoped alternative (runnable anywhere the ref is fetched)" — `git cat-file -e` reads the git object store, not the working tree, so `.factory/` being gitignored/absent is irrelevant to it. The two statements in the same block contradict: a reader in a CI develop checkout is told the predicate does not apply to them, while the supplied command works there as soon as the ref is fetched. This is residue from the v2.10 F-008 fix (`:1437`), which replaced a working-tree `test -f` with the branch-scoped form but left the working-tree-scoped preamble in place.

**Prescribed fix:** Delete the "main repo root ONLY" restriction and its `.gitignore:4` rationale (they describe the retired `test -f` form) and replace with: "Environment: any checkout where the `factory-artifacts` ref has been fetched (`git fetch origin factory-artifacts`). Reads the object store, not the working tree, so the `.gitignore:4` exclusion of `.factory/` does not affect it."

---

### F-W86S-P21-006 — LOW — `red-out.txt` transient artifact not gitignored, while its sibling `coverage-out.txt` is

**Story / loci:** STORY-182 `:816` and `:822` introduce `red-out.txt`; Task 10b `:1243-1249`, Background §Gitignore placement `:155-163`, Architecture Mapping `:885`, FSR `:1360`, `traces_to` `:34` and Notes `:1407-1411` all cover `coverage-out.txt` only.

**Evidence:** `Grep 'red-out\.txt'` over STORY-182 returns exactly three hits: `:816`, `:822`, and one immutable v2.10 changelog row. The story's own stated rationale for the `coverage-out.txt` entry (`:160-162`) is: "**a `.gitignore` entry IS required for `coverage-out.txt`** — the transient CI artifact … **This file must not be accidentally committed.**" `red-out.txt` is created by AC-182-005's RED-demonstration block (`:816`: `> red-out.txt 2>&1`) at the repository root, is untracked, and is not covered by any `.gitignore` entry (`/Users/zious/Documents/GITHUB/wirerust/.gitignore` own-read: `/target`, `.claude/worktrees/`, `.worktrees/`, `.factory/`, `.factory-demos/`, `demo-evidence/`, `/tests/fixtures/local-samples/`, `mutants.out*/`, `mutants-f6*/`). The identical rationale applies with identical force.

This is a one-fixed/one-missed pair inside a single file — the exact shape DF-SIBLING-SWEEP-001 names — and it is remediation-induced: `coverage-out.txt`'s gitignore entry landed at v2.0 (F-P10-001), while `red-out.txt` was introduced eight versions later at v2.10 (F-004) without sweeping the `.gitignore` deliverable.

**Severity rationale (LOW, not MEDIUM):** `red-out.txt` is produced only by a procedure the story explicitly marks "manual test only — do not automate file removal in CI" (`:811`), whereas `coverage-out.txt` is produced by an automated CI step. The exposure is materially smaller.

**Prescribed fix:** Extend Task 10b's `.gitignore` block to two entries and propagate to the Architecture Mapping row `:885`, the FSR `.gitignore` row `:1360`, Background §Gitignore placement `:160-163`, and Task 11 / Notes §Develop PR:
```
# Transient CI artifact from IEC-104 fixture coverage report step (STORY-182)
coverage-out.txt
# Transient artifact from the AC-182-005 manual RED demonstration (STORY-182)
red-out.txt
```

---

### F-W86S-P21-007 — LOW — Narrative scopes the silent-skip elimination by harness rather than by partition

**Story / locus:** STORY-182 `:56-58`.

**Defective claim:** "the **clean-worktree silent-skip class** behind PG-W85-005 is structurally **eliminated for the IEC-104 harness delivered here**".

**Evidence:** The story's own partition semantics (`:793-798`) state the opposite for three quarters of that harness: "Gitignored corpus (iec104.pcap, iec104-sq.pcapng, iec104-iti-dissect.pcap in `tests/fixtures/local-samples/`): advisory only — absent → stdout FIXTURE-SKIPPED notice (**visible only with --nocapture**), **test still passes**." And AC-182-004(b) `:507-508`: "The two Wireshark fixture tests skip via the existing `fixture_present()` stderr path (visible with `--nocapture 2>&1`; **not visible in standard CI output without it**)."

Confirmed against the tree: three of the four `fixture_present()` call sites (`tests/iec104_e2e_real_pcaps_tests.rs:166`, `:292`, `:529` — own-read) guard tests whose fixtures remain gitignored after this story, and each does `return;` on absence, which libtest reports as a pass. So in a clean worktree, 3 of 4 fixture-gated tests in this harness continue to silently report `ok`. What the story actually eliminates is the silent-skip class **for the committed partition** (1 of 4), and it converts the other three from silent to *reported-when-`--nocapture`-is-passed*.

The "(detection, not prevention — accurate expectations remain the implementer's obligation)" caveat that v2.10 F-015 added (`:1437`) is attached to the D-510 stale-expectation clause, not to the silent-skip clause. The harness-level scoping is therefore broader than the delivered mechanism.

**Prescribed fix:** Partition-scope the clause, e.g.: "…is structurally eliminated **for the committed partition of the IEC-104 harness** (the one committed ITI capture always runs in CI); the three gitignored fixtures now emit visible FIXTURE-SKIPPED notices via the additive CI step instead of skipping silently, but their tests still report `ok` on absence…".

---

### F-W86S-P21-008 — LOW — AC-182-006 requires an exact literal that Task 7 supplies only as an example

**Story / loci:** STORY-182 `:836` (AC predicate) vs Task 7 `:1096-1097` (implementation instruction).

**Evidence:**
- AC-182-006 `:836`: `grep -qF 'committed at \`tests/fixtures/\`' tests/fixtures/E2E-PCAPS.md` — `-qF` is a **fixed-string** match, so the byte sequence `` committed at `tests/fixtures/` `` must appear verbatim.
- Task 7 `:1096-1097`: "annotate the `:358` `iec104-iti-diverse.pcap` row with committed status (**e.g.**, add an inline note "committed at `tests/fixtures/`")". "e.g." presents the string as one acceptable option among many.
- Baseline: `Grep 'committed at \`tests/fixtures/\`'` over `tests/fixtures/E2E-PCAPS.md` → **0 matches** (own-read), so the predicate is genuinely discriminating — but an implementer who follows Task 7's "e.g." and writes, say, "committed to `tests/fixtures/`" or "tracked in `tests/fixtures/`" produces a correct annotation that **fails** the AC.

**Prescribed fix:** Make Task 7 `:1096-1097` mandatory and verbatim: "annotate the `:358` row with the **exact literal** inline note `` committed at `tests/fixtures/` `` — AC-182-006 asserts this string with `grep -qF`; equivalent paraphrases will fail the gate."

---

### F-W86S-P21-009 — LOW *(pending intent verification)* — unswept sibling "test files" prose in `ci.yml:436`

**Story / locus:** STORY-183 Task 10 `:1162-1170` (ci.yml stale-prose sweep) and the FSR row `:1261`, which enumerate `:434`, `:442`, `:462` only.

**Evidence (own-read):** `Grep 'test files|tests/\*\.rs|src/\*\*'` over `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml` returns four lines, not three:
- `:434` — "…comment headers **in test files**." → swept by Task 10
- `:436` — "Problem: during strict TDD, **test files** receive module-level or section-level comments asserting that tests "MUST FAIL"…" → **not swept**
- `:442` — "…scans tracked tests/*.rs and src/**/*.rs" → swept
- `:462` — "…comment headers **in test files**" → swept

`:436` sits inside the same contiguous comment block (`:434-451`) as two of the three swept loci and uses the same "test files" noun phrase that Task 10 identifies as stale. It is not caught by AC-183-001's predicate either: `test "$(grep -c 'in test files' ci.yml)" -eq 0` matches "**in** test files" (`:434`, `:462`) but not "test files receive" (`:436`). So `:436` survives both the sweep and the gate.

Post-story the scanned set includes `bin/*.py` and top-level `src/*.rs` — and the originating defect for PG-W84-010 was stale RED prose in `bin/test_check_green_doc_tense.py`, a Python file, not a `.rs` test file (STORY-183 `:88-91`). So `:436`'s problem statement now under-describes the class the gate covers.

**Intent caveat:** I cannot adjudicate whether `:436` is intended as a *scope* statement (in which case it is stale and must be swept) or as a *historical problem-origin* narrative (in which case "test files" is where the anti-pattern was first observed and remains accurate as history). Per DF-SIBLING-SWEEP-001's intent-adjudication rule I report the difference at LOW and tag it `(pending intent verification)` rather than silently skipping it.

**Prescribed fix (if adjudicated stale):** Add a Task 10 bullet: "Line `:436`: 'during strict TDD, test files receive…' → 'during strict TDD, **test and self-test source files** receive…' — the gate's scan set now includes `bin/*.py` self-tests (the originating PG-W84-010 site) and top-level `src/*.rs`." If adjudicated historical, add a one-line note to Task 10 recording that `:436` is deliberately left as problem-origin narrative, so pass 22 does not re-raise it.

---

### F-W86S-P21-010 — NIT — GOOD_CASES residual line count off by one

**Story / locus:** STORY-183 Task 5 `:986-987`: "Residual: **~46** `//`-prefixed source lines remain in the scanned file from GOOD_CASES multi-line fixtures".

**Evidence (own-read):** `Grep '^\s*//'` over `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py` → **85** lines total. `BAD_CASES` opens at `:51` and `GOOD_CASES` at `:332`. Enumerating the 85 hits: 40 fall in `[51,331]` (`:55,61,67,73,79,85,91,97,103,109,118,124,130,136,142,148,154,160,166,175,181,187,193,199,205,220,226,232,238,244,250,266,273,280,287,294,301,308,315,322`) and **45** fall in `[332,632]`. So the GOOD_CASES residual is **45**, not ~46.

The load-bearing figures are all **exact** and verified: 40 `//`-line BAD fixtures + 2 `#` lines (`:258`, `:261` — I hand-executed all 29 shipped regexes against every `#` line in all 6 `bin/*.py` files and confirmed exactly these two match, via Pattern 26 `\bskeleton\s+compiles?\b` on `:258` and Pattern 29 `\buntil\b.*\bwired\b` on `:261`) = **42**, matching `:158-159` and AC-183-002 `:311`.

**Prescribed fix:** `~46` → `45` at `:986`.

---

## Per-axis disposition

**Axis 1 — Truthfulness against the repo. CLEAN (exceptionally so).**
Every one of the ~70 live line anchors I checked resolves correctly (see Methodology §2 for the enumeration). Notable verified claims: `_VIOLATION_PATTERNS` at `:217` with 28 tuples / 29 docstring items, item 5 sharing tuple 4 → `29−1=28`, `+8 = 36 tuples / 37 items` ✓; `src/*.rs` strictly subsumes `src/**/*.rs` by 10 top-level files **[supplied]**, and my glob of `src/*.rs` (direct children only) confirms 0 pattern matches ✓; rename-site arithmetic **13 `_collect_rust_files` lines + 1 `rust_files`-only line at `:721` = 14** confirmed by `Grep -c` (13 and 14 respectively) ✓; the 6 functional monkey-patch sites `:699/:707/:726/:859/:872/:905` and 7 prose sites `:688/:705/:711/:718/:839/:843/:891` all confirmed individually ✓; `finally` block ends `:905`, `print()` at `:907` ✓; CHANGELOG `:741` contains `_collect_rust_files` and `:851` contains "no tracked Rust files are found" ✓; `bin/fetch-e2e-pcaps:154,157` carry the exact two Wireshark sha256s cited in Task 1 Step 1g ✓; `bin/test_lint_cycle_artifact.py` has exactly **21** `def test_tc*` functions, matching the "TC1–TC21 / 21 self-tests" reword ✓; **25** committed captures under `tests/fixtures/` matching the `.pcap|.pcapng|.cap|.trace` set ✓; exactly **10** `falls through to` sites in `*.rs`, matching AC-183-008's enumeration **file-for-file and line-for-line** ✓; convergence-report `:63-66` and `:68-70` say precisely what STORY-183 Background claims ✓; STORY-INDEX `:298-299` titles are byte-identical to both story H1s ✓; the DF-GREEN-DOC-TENSE-SWEEP v6 TIER-1 token list (`policies.yaml:1109-1140`) maps 1:1 onto Patterns 30–37 with no orphan and no invention ✓. **No truthfulness finding.**

**Axis 2 — Tautological / non-discriminating predicates. CLEAN.**
I evaluated all 14 `grep`/`test`-shaped predicates in the two stories against the baseline tree and confirmed **every one is currently FALSE**, i.e. discriminating: `keeps CI green` = 2 ✓; `All are auto-fetchable via …` = 2 ✓; `committed at \`tests/fixtures/\`` = 0 ✓; `iec104-iti-diverse.pcap` in README = 0 ✓; `coverage-out.txt` in `.gitignore` = 0 ✓; `fixture-count-gate-entry.md` in CLAUDE.md = 0 ✓; the three ci.yml step-name/pattern/`!cancelled()` greps = 0 ✓ **[supplied]**; `in test files` = 2 ✓; `src/*.rs` in ci.yml = 0 ✓ (verified by hand: `src/**/*.rs` does **not** contain the fixed substring `src/*.rs`); `bin/*.py` in ci.yml = 0 ✓; `RED GATE version` = 1, `MUST FAIL until bin/lint-cycle-artifact` = 1, `TC1–TC8` = 2 ✓. I specifically hunted the shapes Audit 14 does not cover — `grep -q` on already-present text (none found), `test -f` on an existing file (the former AC-182-006 instance was correctly retired to `git cat-file -e`, now ABSENT ✓), assertions on generated artifacts without an existence guard (both `coverage-out.txt` sites now carry `test -s`; the `grep -q … coverage-out.txt` sites fail-safe with exit 2 under `set -e`), and predicates satisfied by the story's own example text (none — all predicates target repo files). Also verified AC-183-006's negative guard is genuinely discriminating: if `_is_comment_line` treated `#` globally rather than `.py`-scoped, the `good_{n}.rs` GOOD_CASE would fail. **No finding on this axis** — but note F-W86S-P21-003 is a *satisfiability* defect in an otherwise correctly-discriminating predicate.

**Axis 3 — Verifiability, both directions. ONE finding (F-W86S-P21-008).**
Forward (deliverable → predicate): all 12 STORY-182 Architecture Mapping rows and all 8 STORY-183 rows have at least one AC predicate. STORY-183's `:258`/`:261` scrub and the ~40-fixture conversion are both covered by the `python3 bin/check-green-doc-tense` exit-0 gate, which is genuinely load-bearing because both lines currently match. AC-183-009's three greps cover all four Task-13 scrub lines (`:3` via "RED GATE version", `:6` via "MUST FAIL until…", `:5` **and** `:125` both via "TC1–TC8" `-eq 0`) ✓. Backward (AC → Task): every AC traces to a numbered Task. The one coupling defect is the "e.g." vs `grep -qF` mismatch at F-W86S-P21-008. I considered and **rejected** a finding for Task 7's four E2E-PCAPS.md prose loci (`:3-6`, `:48-50`, `:374-380`, `:391-396`) having no AC predicate — mechanical predication of narrative prose sweeps is not a reasonable requirement, and the loci are individually enumerated in Task 7, the Architecture Mapping row, and the FSR.

**Axis 4 — False-GREEN / silent-skip risk. TWO findings (F-W86S-P21-002, F-W86S-P21-004).**
Beyond those: the AC-182-005 RED-demonstration block (`:812-825`) is correctly built — the expected failure is consumed by an `if`, so `set -e` does not fire, and the block then makes a **positive** assertion on the panic text (`grep -qF "REGRESSION: committed fixture 'iec104-iti-diverse.pcap' is absent" red-out.txt`) with restore-on-both-paths. That is the correct expected-failure shape. The story's predicate-before-print analysis is right and I verified it against the prescribed code: `println!` at `:309-319` precedes the assert partition at `:619+`, which is exactly why the paired `grep -qE "test result: ok"` is load-bearing at every locus that has it. The `2>&1`/no-`2>&1` partition is correct throughout (the Env A manifest-report block deliberately omits `2>&1` because `test_fixture_manifest_report` never calls `fixture_present`, and `:1193-1199` says so). SIGPIPE-unsafe `| tee /dev/stderr | grep -q` pipelines are absent (retired at v1.8). Task 1's fetch gates are correctly inside `if` conditions so `pipefail` does not misfire. STORY-183's single-command fences without `set -euo pipefail` (AC-183-003, AC-183-004 Verification) are safe: a one-command fence's status *is* the command's status, consistent with Audit 14's >1-command scope.

**Axis 5 — Self-referential-flag hazard. ONE finding (F-W86S-P21-001).**
I hand-executed all 29 shipped regexes (read verbatim from `bin/check-green-doc-tense:217-457`) plus all 8 prescribed ones against **every** comment line the stories prescribe. Results: (a) all 14 `# Pattern NN BAD/GOOD (.rs|.py):` fixture-block annotations in AC-183-007 are clean ✓; (b) all four TIER-2 GOOD_CASE `#` annotation blocks are clean — `# … Tool MUST NOT flag` does not match Pattern 1 (`All tests MUST FAIL`), and `currently fails` does not match Pattern 31 (`currently\s+falls?\b` needs `fall`, not `fail`) ✓; (c) AC-183-006's suffix-scoping guard block is clean ✓; (d) every BAD/GOOD **content** string and **label** string is a Python source line beginning with `"`, `'`, or `(` after `.strip()`, so none is comment-eligible — including the label at `:563` which contains `'falls to the wildcard'` and the Pattern 34 label at `:569` which contains `doesn't exist yet` ✓; (e) Pattern 30/31/32/34/35/36/37 `#` comment blocks are all clean, and I confirmed the v2.6 Pattern-35 and v2.2 GOOD-annotation fixes are still in place ✓. `:560` is the **sole** match. I also verified the match-level (not phrase-level) safety mechanism the story relies on for Task 4: Pattern 26 is `\bskeleton\s+compiles?\b` and Pattern 29 is `\buntil\b.*\bwired\b`, and both prescribed replacements defeat them because the regex-literal `\b` places a word character (`b`) immediately before the trigger token — `…bskeleton`, `…buntil` — so the leading `\b` assertion cannot fire ✓. Likewise verified the story's claims that `:213` is already safe by the same mechanism (Pattern 23 `All tests\b.*\bMUST FAIL` fails because `\bMUST` is preceded by the `b` of `\b`) ✓ and that `:259`/`:260` are safe by a *different* mechanism (Patterns 27/28 require an `exposes|is a|are` verb immediately before `compile-only`, absent on both lines) ✓ — the v2.10 F-011 correction is accurate. `#`-eligibility being `.py`-scoped is correctly handled throughout (EC-012, `~3625` Rust attribute lines excluded).

**Axis 6 — Sibling-sweep completeness (DF-SIBLING-SWEEP-001). TWO findings (F-W86S-P21-006, F-W86S-P21-009).**
Both are one-fixed/one-missed pairs. Clean elsewhere: STORY-183's `_collect_rust_files` → `_collect_source_files` rename sweep is complete and correctly enumerated (all 13+1 sites verified individually), and correctly *excludes* both CHANGELOG loci (`:741`, `:851`) plus delivered story specs as shipped provenance ✓. STORY-182's `if: always()` → `if: ${{ !cancelled() }}` sweep shows 0 live loci ✓. The `1/4` count-anchor sweep is consistent across Background `:140-147`/`:205`, AC-182-001 `:334`, AC-182-004 `:526`, AC-182-005 `:650-656`, EC-003, Task 9 Env B, Task 11 ✓. The `wc -c` portability fix is applied at both loci (`:444`, `:951`) ✓. The AC-182-005 needle-guard discipline holds: I confirmed the prescribed comment block contains **zero** contiguous `fixture_present("` occurrences (the `concat!("fixture_present", "(\"")` split, `fixture_present(name_var)`, and `fixture_present()` forms are all non-matching) and that the current tree has exactly 4 literal call sites = `FIXTURE_GATED_TESTS.len()` ✓. The `fn <name>` coupling loop is genuinely non-self-referential: in the registry literal the name is preceded by `("`, and in the `:25-28` mapping table by a backtick — neither yields `fn ` ✓. Adjudicated-out-of-scope siblings (ENIP harness, `e2e_corpus_smoke_tests.rs:206-224`, `bc_2_12_011_story127_tests.rs`, `tests/fixtures/mk_modbus_*.py`, `fuzz/seed_corpus.py`, extension-less `bin/` executables, the three deferred stale-RED sites) were **not** re-reported.

**Axis 7 — Internal contradiction. THREE findings (F-W86S-P21-004, F-W86S-P21-005, F-W86S-P21-007).**
I gave targeted attention to both historically-affected regions. **Task 8 / Task 10a:** the "`M ≠ 4` blocks / `N` is evidence only" rule is now internally coherent (the v2.10 F-007 fix held) — but the *mechanism sentence* is not, and the two loci now name different mechanisms → F-W86S-P21-004. **AC-182-006 block:** one intra-block contradiction survives → F-W86S-P21-005; the rest of the block is coherent, and the "obligation described as both blocking-and-enforceable AND evidence-only" pattern is **gone** from both loci ✓. Frontmatter ↔ body coherence: both stories declare `behavioral_contracts: []` / `verification_properties: []` and both bodies state "_(none — E-11 convention…)_" with zero AC BC-traces ✓; `points: 4`/`5` match the H1 blocks, STORY-INDEX `:298-299`, and the epic totals at STORY-INDEX `:10` ✓; `target_module: tests/` and `bin/` match the respective Architecture Mapping tables ✓; `traces_to` entries all resolve (including STORY-183's `convergence-report.md`, added at v2.3) ✓. ACR ↔ Task ↔ AC alignment verified for the shared-resolver contract, the `#[ignore]` prohibition, the constants-placement rule, the needle guard, and the additive-ci.yml rule ✓. STORY-183's ACR "TIER-2 exclusion is policy, not a defect" matches `policies.yaml:1460-1462` verbatim in substance ✓.

**Axis 8 — Environment-blindness. CLEAN.**
`tests/fixtures/local-samples/` is absent in this checkout **[supplied item 11]** and confirmed gitignored at `.gitignore:10` (own-read). Every one of the four `1/4`-producing blocks (AC-182-001 `:353-368`, AC-182-004 `:544-559`, Task 9 Env B `:1218-1233`) is wrapped in the `if [ -d tests/fixtures/local-samples ]; then … else … fi` guard with a `[ ! -e /tmp/ls-bak ]` pre-existence check and an unconditional `trap … EXIT` restore ✓. Env A explicitly declares its Task-1 precondition ✓. The `4/4` value is correctly labelled environment-specific and is on the documented co-update list (`:647-649`) ✓. `.factory/` environment scoping is correct everywhere except the F-W86S-P21-005 residue: the gate-entry doc is routed to the `factory-artifacts` branch via state-manager in the Architecture Mapping (`:884`), FSR (`:1357`), Notes (`:1412-1414`), and the `traces_to` inline comment (`:33`) — consistently ✓. EC-008 correctly declares the `CARGO_MANIFEST_DIR` source-read assumption and accepts it for this repo's CI ✓.

**Axis 9 — Arithmetic and enumeration. ONE NIT (F-W86S-P21-010).**
All load-bearing figures re-derived and **exact**: 40 + 2 = 42 ✓ (hand-verified by enumerating all 85 `//` lines and hand-executing all 29 regexes against all `#` lines in `bin/*.py`); 12 new BAD_CASES (2+2+2+2+1+1+1+1) ✓ and 14 new GOOD_CASES (1+1+1+1+6+3+1) ✓ summing to 26, matching the Architecture Mapping row `:820` and the FSR row `:1259`, and matching the Task 7 (4 BAD + 3 GOOD) / Task 8a (8 BAD + 11 GOOD) split ✓; 28 + 8 = 36 tuples, 29 + 8 = 37 docstring items ✓; 13 + 1 = 14 rename sites ✓; `FIXTURE_MANIFEST.len() = 4` = `FIXTURE_GATED_TESTS.len() = 4` = 4 literal `fixture_present("` call sites ✓; `COMMITTED_FIXTURES.len() = 1` consistent with 1/4 ✓; 25 committed captures ✓; 21 TC functions ✓; 10 `falls through to` sites ✓; 6 CLAUDE.md protocol-doc rows ✓; 19 policies ✓; ~3625 Rust attribute lines is an approximation, correctly hedged. Only the "~46" is off (actual 45), and it is already hedged with "~".

**Axis 10 — Scope integrity. ONE finding (F-W86S-P21-007).**
Otherwise clean and unusually careful. STORY-182: `Forbidden modifications: src/**/*, Cargo.toml, bin/*, CHANGELOG.md` with the single documented `ci.yml` carve-out (F-014) is consistent across the FSR, ACR, and Notes ✓; the "No CHANGELOG entry required" determination is correct against `ci.yml:494-499`, which excludes `tests/` and `.github/` from the trigger set and does not include `CLAUDE.md` or `.gitignore` ✓. STORY-183: CHANGELOG **is** required (`bin/` in the trigger set at `ci.yml:524`) and AC-183-005's verification uses the true CI-equivalent three-dot form matching `ci.yml:533` byte-for-byte ✓; `tests/**/*` correctly marked read-only-permitted for the AC-183-008 zero-FP check ✓; L-W84-003/AC-165-001 correctly assessed as not triggered (no new `bin/test_*.py` file) ✓; PG-W84-012/D-525 correctly scoped to three ops sub-tasks with an explicit "do NOT add a CI wiring task for this story" ✓; the residual-TIER-1 exclusions and the `unimplemented!()` follow-up are honestly enumerated in Notes §Story scope clarification ✓. Both stories' disjoint-ci.yml-region cross-notes agree with each other (`STORY-182:1415-1422` ↔ `STORY-183:1293-1300`) and correctly label anchors as `develop@e8841d76` baseline, order-dependent ✓.

**Axis 11 — CI/gate realism. CLEAN.**
The additive step is executable as written and coherently placed: the `test` job is at `ci.yml:40-47` with `- run: cargo test --all-targets` at `:47` (own-read), and the story places the new step immediately after it ✓. `if: ${{ !cancelled() }}` is a valid GHA expression, and the story's visibility semantics are precisely right — "`!cancelled()` guarantees execution, not output; after checkout/compile failures the step runs but cannot emit the coverage line" (`:520-524`, `:1320-1323`) ✓. The step's `run` block opens with `set -euo pipefail`, which is necessary because GitHub's default Linux `run` shell is `bash -e {0}` **without** `pipefail` unless `shell: bash` is declared — the story's own note distinguishing CI-bash-`-e`-default from the local case (`:573-574`) is accurate ✓. Gating is by the step's own exit status (cargo exit → `set -e`; then two `grep -q` on the tee'd file), i.e. execution truth rather than output-string presence ✓. No new SHA-pinned actions are introduced, so the `action-pin-gate` job (`:427-432`) is unaffected, as STORY-183's ACR states ✓. `grep -qF 'Fixture coverage: [1-9][0-9]*/[0-9]+' ci.yml` will match because the ACR's prescribed run block embeds that exact regex text as a literal ✓. STORY-183's ci.yml edits are non-functional (two comment lines + one step name) and do not touch `run:` bodies ✓. The one *procedural* gate that is not realism-clean is Task 8's wave-gate command → F-W86S-P21-004.

---

## Novelty assessment

| Finding | Novelty | Basis |
|---|---|---|
| 001 | **(c) induced by a prior remediation — and a recurrence of a closed class** | v2.4 F-W86S-P14-004 removed this exact contiguous phrase; v2.10 F-014 re-introduced it. Also the 6th instance of the self-referential-predicate class (STORY-INDEX `:19` records it as a HIGH at pass-12). |
| 002 | **(c) induced by a prior remediation** | v2.10 AUDIT-2 replaced the `set -e`-safe `test "$(grep -c …)"` form with a `set -e`-fatal assignment form at both loci. |
| 003 | **(c) induced by a prior remediation** | v2.10 F-002 introduced the whole-file predicate to fix a tautology; the new predicate is discriminating but section-blind. |
| 004 | **(c) induced by a prior remediation** | v2.10 F-007 rewrote Task 8 + Task 10a together to resolve a blocking/evidence-only contradiction; the rewrite introduced a phantom grep in one locus and left the pair divergent. |
| 005 | **(c) induced by a prior remediation** | v2.10 F-008 swapped `test -f` for `git cat-file -e` but left the working-tree-scoped preamble that only made sense for the retired form. |
| 006 | **(c) induced by a prior remediation** | v2.10 F-004 introduced `red-out.txt`; the `coverage-out.txt` gitignore deliverable (v2.0 F-P10-001) was not swept. |
| 007 | **(b) recurrence of a class** | Narrative over-scoping has been remediated at v1.6 F-002, v1.4 F-011, v2.10 F-015. This is the same class at a finer granularity (harness-scoped vs partition-scoped) — the residual the prior three fixes did not reach. |
| 008 | **(a) genuinely new** | No changelog row in either story addresses AC-predicate ↔ Task exact-literal coupling. |
| 009 | **(b) recurrence of a class** | ci.yml stale-prose sweep has been extended three times (v1.6 F-017 added `:434`/`:462`; v2.0 F-P10-002; v2.10 F-003 added the predicates). `:436` is the next sibling in the same block, never enumerated. |
| 010 | **(a) genuinely new** | The 40/42 figures were made exact at v1.2 F-011; the GOOD_CASES residual was introduced at v2.10 F-012 and never counted. |

**Aggregate read.** Six of ten findings are remediation-induced. That is the signature the two mechanical audits were introduced to suppress, and it is now the dominant residual mode: the *content* of both stories is extremely accurate (axis 1 is clean across ~70 anchors and every load-bearing count is exact), but each remediation burst has a non-trivial probability of introducing a new defect in the region it touches. Notably, **three of the four MEDIUMs (002, 003, 004) are defects in verification/gate machinery, not in the delivered mechanism** — the stories describe what to build correctly; they describe how to *prove* it was built incorrectly. Novelty is **MODERATE**: these are not rewordings of prior findings, but they are all confined to pass-20's own edit surface. A pass-21 remediation that touches only the v2.10 diff regions, plus a diff-scoped re-audit of that burst, should be sufficient — I would not expect a broad re-sweep to be productive.

---

## `[process-gap]` tagging

- **[process-gap] F-W86S-P21-002 — Audit 14's predicate-shape definition is too narrow.** The orchestrator's mechanical audit matches `test "$(grep -c …)" <op> N`. It does not match `VAR="$(grep -c …)"` + `test "${VAR}" <op> N`, which is a *strictly more dangerous* shape: the `test "$(…)"` form discards the substitution's exit status, whereas the bare-assignment form promotes it and lets `set -e` fire. Audit 14 should be widened to flag **any** `grep -c` whose exit status can reach `set -e` — i.e. any `grep -c` that is not an argument to another command and not in a condition context — and to require `|| true` or a negated `grep -q` in those positions. Without this, the audit will keep certifying "0 live predicates pass on baseline" for a block that cannot pass in *any* state.
- **[process-gap] F-W86S-P21-001 — the no-literal-phrase discipline (D-529) has no mechanical enforcement, and regressed.** The rule is stated three times in STORY-183 and was verified clean at passes 12, 14, and 16, yet pass-20's remediation re-introduced a violation that pass-14 had specifically removed. A story-writer-side check is needed: after any edit to a `# Pattern NN` comment (or any `#` line prescribed for a scanned `.py` file), re-execute the pattern's own regex against the edited line. This is mechanically trivial and would have caught `:560` at authoring time. Recommend adding it to the DF-SIBLING-SWEEP-001 STORY-edits checklist, since it is a "same-artifact sibling" rule in substance.
- **[process-gap] F-W86S-P21-009 — the ci.yml prose sweep is anchor-driven rather than phrase-driven.** Task 10 enumerates three line anchors; the AC predicate greps the phrase `in test files`. `:436` uses "test files receive", so it falls between the two nets. Prose-sweep tasks should be specified as *phrase* sweeps ("grep the file for `test files` and adjudicate every hit") rather than as anchor lists, so that the sweep and its gate share one definition of the target set.

---

## Verdict

**NOT_CONVERGED**

**Tally: 10: 0C / 0H / 4M / 5L / 1N**

Ninth consecutive zero-CRITICAL, zero-HIGH pass. The four MEDIUMs are all concrete and mechanically fixable, and all four are confined to the pass-20 (v2.10) edit surface:

- **F-W86S-P21-001** — restore the pass-14-compliant Pattern 33 comment wording; re-run the AC-183-007 `:688-695` sweep over the whole prescribed block.
- **F-W86S-P21-002** — make both `SKIP_COUNT` reads `set -e`-safe (`|| true`, or negate with `grep -q`) at `:496-497` and `:1184-1185`.
- **F-W86S-P21-003** — section-scope the E2E-PCAPS.md predicate at `:837`, and add an explicit "do NOT edit `:279`" directive to Task 7.
- **F-W86S-P21-004** — make Task 8's wave-gate command self-gating (`tee` + `grep -qE "test result: ok"`), and sweep Task 10a `:1241` so both loci name one mechanism.

Because six of ten findings are induced by the immediately-preceding remediation burst, I recommend the pass-21 remediation be followed by a **diff-scoped re-audit of the v2.10→v2.11 delta specifically** (rather than another full sweep) before pass 22 is dispatched — the evidence across passes 13, 19, 20, and 21 is that the dominant defect source is now the burst itself, not the artifact.

**Key file paths referenced:**
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-182.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-183.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/policies.yaml`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/stories/STORY-INDEX.md`
- `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-085/STORY-180/convergence-report.md`
- `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense`
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_check_green_doc_tense.py`
- `/Users/zious/Documents/GITHUB/wirerust/bin/test_lint_cycle_artifact.py`
- `/Users/zious/Documents/GITHUB/wirerust/bin/fetch-e2e-pcaps`
- `/Users/zious/Documents/GITHUB/wirerust/tests/iec104_e2e_real_pcaps_tests.rs`
- `/Users/zious/Documents/GITHUB/wirerust/tests/enip_e2e_real_pcaps_tests.rs`
- `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/E2E-PCAPS.md`
- `/Users/zious/Documents/GITHUB/wirerust/tests/fixtures/README.md`
- `/Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml`
- `/Users/zious/Documents/GITHUB/wirerust/.gitignore`
- `/Users/zious/Documents/GITHUB/wirerust/CHANGELOG.md`
