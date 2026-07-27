---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 15
verdict: NOT_CONVERGED
novelty: "medium-low"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 15

**Date:** 2026-07-26/27
**Pass:** 15 of N
**Verdict:** NOT CONVERGED
**Novelty:** medium-low — v2.3/v2.4-induced second-order regressions; no recurring class; MEDs are
newly-induced by prior remediation bursts. THIRD consecutive zero-HIGH pass.
**Tally:** 14 findings — 0 CRIT / 0 HIGH / 5 MED / 6 LOW + 3 NITs (all fixed)
**Status:** REMEDIATED — D-532 state burst; STORY-182 v2.4→v2.5 + STORY-183 v2.4→v2.5
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary (verified-clean axes):** ground-truth predicate correctness; inert-predicate
elimination confirmed (zero live if:always() loci post-P14); no-literal-phrase standing discipline
(D-529) satisfied; DF-SIBLING-SWEEP-001 compliance confirmed; pathspec subsumption direction
consistent at all 7 loci (v2.5 holds); self-anchor elimination complete (zero :NNN intra-doc
self-citations); canonical hashes 9a0f34c/9c9b12f unchanged; Task-8 split 8a/8b ordering verified
sound post-P14 fix; PASS/FAIL-convention blocks structurally correct post-P15 hardening; ci.yml
order-dependence labels present.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P15-001 | MED | STORY-182 | FIXED v2.5 | README citation string absent from prescribed row; AC-182-002 mandates exact "ITI CC-BY-4.0" prefix in the provenance table row; v2.4 text used a non-canonical paraphrase at that row position. |
| F-W86S-P15-002 | MED | STORY-182 | FIXED v2.5 | move-aside guards lacked source-existence branch; when iec104-iti-diverse.pcap is absent (develop checkout without fixture fetch), the move-aside step attempts `mv` on a non-existent file and the trap restore attempts `mv` on a backup that was never created, producing a bogus error that masks the intended test-skip path. |
| F-W86S-P15-003 | MED | STORY-183 | FIXED v2.5 | pipefail-without-set-e + println-before-assert → false-GREEN capable verification blocks; Task-8b and AC-183-009 blocks use `set -o pipefail` but not `set -euo pipefail`; additionally several PASS/FAIL-convention blocks emit `println!("PASS [...]")` before the assert predicate, so a panic produces a PASS line on stdout before the panic aborts — harness reads PASS even on failure. Gate-BLOCKING for Task-8 verification obligation. |
| F-W86S-P15-004 | MED | STORY-182 | FIXED v2.5 | bc_2_12_011_story127_tests.rs mis-anchored as silent-skip sibling; v2.4 Notes classified this file in the LOCAL_SAMPLES/fixture_present silent-skip class alongside enip_e2e_real_pcaps_tests.rs. Truth: bc_2_12_011_story127_tests.rs uses a synthetic fallback (writes synthetic_16pkt_pcapng when LOCAL_SAMPLES is None, then runs full assertions against the synthetic file) — it is NOT a silent skip. Mislabel propagated to STATE.md DRIFT-e2e-sibling-harnesses row. |
| F-W86S-P15-005 | MED | STORY-183 | FIXED v2.5 | False monkey-patch rationale on load-bearing placement constraint; Task-12 prescribes that the pattern registration call must appear after the GOOD_CASES list and before the BAD_CASES list, citing "monkey-patch order dependency" as the reason. That rationale is incorrect — the constraint is structural: registration indexes patterns by list position, and reordering changes the index values used in assertions that reference specific pattern indices. A monkey-patch could be reordered freely; this placement cannot. |
| F-W86S-P15-006 | LOW | STORY-182 | FIXED v2.5 | Host tool availability assumption; Task-1 curl + sha256sum pipeline assumes `sha256sum` is available; macOS ships `shasum -a 256` only; story should note the platform-aware invocation or use the Python hashlib fallback (`python3 -c "import hashlib, sys; ..."`) for portability. |
| F-W86S-P15-007 | LOW | STORY-182 | FIXED v2.5 | git ls-files multi-glob dedup behavior; AC-183-008 verification step uses `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`; src/*.rs is a proper subset of src/**/*.rs, so matching files appear twice in the output; story did not note that the count assertion must pipe through `sort -u` (or equivalent dedup) before comparing to the expected file count. |
| F-W86S-P15-008 | LOW | STORY-182 | FIXED v2.5 | Implicit test-result-ok check for ci.yml fixture-count step; the ci.yml fixture-count gate step prescribes `echo "Fixture coverage: N/4 committed"` but does not prescribe an explicit `[ "$COUNT" -eq 4 ]` exit-nonzero gate; a count of 2 would print a well-formed string without failing CI. |
| F-W86S-P15-009 | LOW | STORY-183 | FIXED v2.5 | test_lint_cycle 21-pass count not quantitatively derived; Task-10 verification specifies that `test_lint_cycle_artifact.py` must produce exactly 21 sub-test passes, but the story does not show the derivation (21 = how many TIER-1 pattern self-test cases at post-P15 expansion), leaving implementers unable to distinguish a counting-change from a partial-failure. |
| F-W86S-P15-010 | LOW | STORY-183 | FIXED v2.5 | Python gate exit-code non-specificity; AC-183-009 verification forms check `$? -eq 0` (clean) vs non-zero (violations), but do not distinguish exit 1 (scanner found violations — expected in BAD_CASE runs) from exit 2+ (Python argument/import error); a conftest crash would look identical to a clean scan at the gate level. |
| F-W86S-P15-011 | LOW | STORY-183 | FIXED v2.5 | cargo e2e 66-finding expectation not flagged as CI-gating; the story references the 66/20/46 ITI diverse-capture expectation from PR #439 gate-fix but does not note this expectation is now enforced by a CI-gating test (tests/iec104_analyzer_tests.rs `test_iec104_iti_diverse_e2e_expectations`) for the first time; delivery must not regress it. Execution evidence required at delivery. |

---

## Findings Detail

### F-W86S-P15-001 (MED) — README citation string absent from prescribed row

**Story:** STORY-182
**Status:** FIXED v2.5 — canonical "ITI CC-BY-4.0" prefix string added at prescribed row position

**Description:** AC-182-002 mandates that the README.md provenance table include a row with the exact
string pattern "ITI CC-BY-4.0" to serve as a stable grep anchor for the citation-currency discipline.
STORY-182 v2.4 Task 7's prescribed README table row used a paraphrase ("ICS-CERT ITI corpus,
CC-BY-4.0 licensed") at the relevant row position rather than the mandated "ITI CC-BY-4.0" prefix string.

An implementer following v2.4 would write a table row that passes visual inspection but fails any
automated citation-currency grep anchored on "ITI CC-BY-4.0".

**Fix (v2.5):**
Task 7 README table row rewritten to include the exact mandated "ITI CC-BY-4.0" prefix at the
prescribed position. A cross-reference note added to AC-182-002 citing the grep anchor form.

---

### F-W86S-P15-002 (MED) — move-aside guards lacked source-existence branch

**Story:** STORY-182
**Status:** FIXED v2.5 — source-existence guard added to move-aside and trap-restore procedures

**Description:** STORY-182 v2.4 prescribed move-aside procedures of the form:

```bash
BACKUP="/tmp/e2e_pcap_backup_$$.pcap"
mv tests/fixtures/iec104-iti-diverse.pcap "$BACKUP"
trap 'mv "$BACKUP" tests/fixtures/iec104-iti-diverse.pcap' EXIT
```

On a clean develop checkout (without running `bin/fetch-e2e-pcaps`), the fixture file does not
exist. The `mv` on a non-existent source exits non-zero with an error message, and the trap
restore then attempts `mv "$BACKUP"` on a file that was never created, producing a second
spurious error. The combined output masks the intended local-samples-absent skip signal.

**Fix (v2.5):** Source-existence guard added:
```bash
if [ -f tests/fixtures/iec104-iti-diverse.pcap ]; then
  mv tests/fixtures/iec104-iti-diverse.pcap "$BACKUP"
  trap 'mv "$BACKUP" tests/fixtures/iec104-iti-diverse.pcap' EXIT
fi
```
Three move-aside procedure sites updated. Trap restore correspondingly guarded to check backup
existence before restoring.

---

### F-W86S-P15-003 (MED) — pipefail-without-set-e + println-before-assert → false-GREEN verification blocks

**Story:** STORY-183
**Status:** FIXED v2.5 — set -euo pipefail added; PASS/FAIL blocks reordered (predicate-first)

**Description:** Two compound defects in STORY-183 v2.4 verification blocks:

**Defect A (pipefail-without-set-e):** Shell verification blocks prescribed `set -o pipefail` but
not `set -e`. Without `set -e`, a command failure (non-zero exit) in the middle of a block does not
abort execution — only the final command's exit status is returned. A failing intermediate step is
silently swallowed. This affects Task-8b's per-pattern RED verification loop.

**Defect B (println-before-assert):** Several PASS/FAIL-convention blocks (established in P14)
were written as:
```rust
println!("PASS [pattern-{N}]");
assert!(condition, "pattern-{N} must trigger FAIL");
```
The `println!("PASS [...]")` emits to stdout before the assertion is evaluated. If the assertion
panics (condition false), the panic is printed to stderr but stdout already carries "PASS [...]".
The tdd_mode harness, which parses stdout for PASS/FAIL lines, sees PASS and reports no failures —
even on a pattern that does not trigger RED on the violating file.

This defect is **gate-BLOCKING**: Task-8b's RED-gate verification obligation (established in P14
F-W86S-P14-001) would be systematically satisfied by false-GREEN output.

**Fix (v2.5):**
- All verification shell blocks updated to `set -euo pipefail`.
- All PASS/FAIL-convention blocks reordered: predicate evaluated first, then PASS or FAIL line
  emitted based on the result (not before it):
  ```rust
  if condition {
      println!("PASS [pattern-{N}]");
  } else {
      println!("FAIL [pattern-{N}]: expected RED trigger not detected");
  }
  ```
- Task-8b note updated: "PASS line must only appear after predicate confirms RED trigger."

---

### F-W86S-P15-004 (MED) — bc_2_12_011_story127_tests.rs mis-anchored as silent-skip sibling

**Story:** STORY-182
**Status:** FIXED v2.5 — bc_2_12_011_story127_tests.rs removed from silent-skip class; class corrected
in both STORY-182 Notes and STATE.md DRIFT-e2e-sibling-harnesses row

**Description:** STORY-182 v2.4 Notes §"Sibling e2e harnesses (deferred)" listed three files in
the LOCAL_SAMPLES/fixture_present silent-skip class:
- `tests/enip_e2e_real_pcaps_tests.rs`
- `tests/bc_2_12_011_story127_tests.rs`
- `tests/e2e_corpus_smoke_tests.rs`

This classification is incorrect for `bc_2_12_011_story127_tests.rs`. Inspection of that file
reveals it uses a **synthetic fallback**: when `LOCAL_SAMPLES` is `None`, it writes a small
synthetic 16-packet pcapng file to a temp path, then runs full BC-2.12.011 assertions against the
synthetic data. It does not silently skip; it always executes assertions.

The silent-skip class is: `enip_e2e_real_pcaps_tests.rs` (identical `fixture_present` conditional
skip idiom as IEC-104) and `e2e_corpus_smoke_tests.rs` (directory-level skip variant at lines
~206-224). `bc_2_12_011_story127_tests.rs` is not a member of this class.

The mislabeling also propagated to the STATE.md DRIFT-e2e-sibling-harnesses row, which listed the
same three files. Both loci corrected by this burst.

**Fix (v2.5):**
- STORY-182 Notes §"Sibling e2e harnesses" corrected: `bc_2_12_011_story127_tests.rs` removed
  from the class list; clarifying note added: "synthetic-fallback class (always runs assertions)
  is distinct from silent-skip class."
- STATE.md DRIFT-e2e-sibling-harnesses row corrected per D-532.

---

### F-W86S-P15-005 (MED) — False monkey-patch rationale on load-bearing placement constraint

**Story:** STORY-183
**Status:** FIXED v2.5 — structural rationale substituted for false monkey-patch claim

**Description:** STORY-183 v2.4 Task-12 prescribed that the pattern registration call must appear
after the GOOD_CASES list and before the BAD_CASES list, with the stated rationale:

> "Order matters due to monkey-patch dependency: registration modifies the module state that
> BAD_CASES inspection relies on."

This rationale is incorrect. `bc_2_12_011_story127_tests.rs` does not use monkey-patching; it
uses a standard Rust test module with deterministic list positions. The actual constraint is:

> Pattern registration indexes entries by their sequential list position. Assertions in
> AC-183-007 and the self-test harness reference patterns by their assigned index number
> (e.g., `PATTERN_LIST[30]` for Pattern 30). Inserting the registration call after GOOD_CASES
> means all pattern indices are offset by the GOOD_CASES list length — changing every existing
> index reference throughout the story and test fixtures.

"Monkey-patch" implies a runtime-override mechanism; the actual constraint is compile-time
list-position arithmetic. An implementer reading the false rationale might conclude that any
placement before BAD_CASES evaluation (including arbitrary mid-list positions) is acceptable.

**Fix (v2.5):** Rationale replaced with the structural constraint explanation. Task-12 note
updated: "Pattern index assignment is positional. Registration must appear at position N in
the file where N equals the count of GOOD_CASES entries, so that PATTERN_LIST[K] for K ≥ 30
resolves to the correct new pattern."

---

### F-W86S-P15-006 (LOW) — Host tool availability assumption

**Story:** STORY-182
**Status:** FIXED v2.5 — platform-aware invocation note added

**Description:** Task-1 fixture fetch pipeline prescribes:
```bash
curl -fsSL "$URL" | sha256sum > checksums.txt
```
`sha256sum` is a GNU coreutils command available on Linux but not macOS (which ships
`shasum -a 256`). On a developer macOS machine, the prescribed step fails silently (pipeline
failure masked by `|`) or exits non-zero, depending on shell settings.

**Fix (v2.5):** Note added to Task-1 prescribing platform-aware invocation:
```bash
SHA_CMD=$(command -v sha256sum 2>/dev/null || echo "shasum -a 256")
$SHA_CMD < fixture.pcap
```
Alternatively, the Python hashlib fallback (`python3 -c "import hashlib,sys; ..."`) is
noted as the portable option.

---

### F-W86S-P15-007 (LOW) — git ls-files multi-glob dedup behavior

**Story:** STORY-182
**Status:** FIXED v2.5 — dedup note added to AC-183-008 verification step

**Description:** The AC-183-008 verification step prescribes counting files via:
```bash
git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py | wc -l
```
`src/*.rs` matches files already matched by `src/**/*.rs` (since `**` includes depth 0 in
git's pathspec interpretation). Files in `src/` itself appear twice in the output. Without
deduplication, `wc -l` overcounts, causing the assertion to fail on a correctly-scoped scan.

**Fix (v2.5):** Dedup step added before count:
```bash
git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py | sort -u | wc -l
```
Note added explaining the glob overlap and the dedup requirement.

---

### F-W86S-P15-008 (LOW) — Implicit test-result-ok check for ci.yml fixture-count step

**Story:** STORY-182
**Status:** FIXED v2.5 — explicit count-gate assertion added to ci.yml step

**Description:** The ci.yml fixture-count gate step in STORY-182 v2.4 prescribes:
```bash
COUNT=$(git ls-files tests/fixtures/*.pcap | wc -l | tr -d ' ')
echo "Fixture coverage: $COUNT/4 committed"
```
This echoes the count but does not fail the CI step if `COUNT` is not 4. A checkout with
only 2 fixtures would print `"Fixture coverage: 2/4 committed"` and proceed without error.

**Fix (v2.5):** Explicit gate assertion added:
```bash
[ "$COUNT" -eq 4 ] || { echo "FAIL: expected 4 committed fixtures, got $COUNT"; exit 1; }
```
ci.yml step prescription updated to include the gate assertion as a mandatory component.

---

### F-W86S-P15-009 (LOW) — test_lint_cycle 21-pass count not quantitatively derived

**Story:** STORY-183
**Status:** FIXED v2.5 — derivation note added

**Description:** Task-10 specifies that `test_lint_cycle_artifact.py` must produce exactly 21
sub-test passes. The number 21 is asserted without showing how it is derived from the story's
own structure. Implementers cannot confirm whether a 20-pass result indicates a partial failure
or a valid change in the test count without independent analysis.

**Fix (v2.5):** Derivation note added: "21 = 8 TIER-1 pattern self-tests (Patterns 30-37 via
tdd_mode, Task-8b) + 9 AC-183-009 gate-check predicates + 4 AC-183-007 scanner-output-format
assertions. If this arithmetic changes (e.g., a new pattern is added), Task-10 count must be
updated before commit."

---

### F-W86S-P15-010 (LOW) — Python gate exit-code non-specificity

**Story:** STORY-183
**Status:** FIXED v2.5 — exit-code semantics table added

**Description:** AC-183-009 verification forms check the script's exit code via `[ $? -eq 0 ]`
(clean run) vs non-zero (violations found or error). The three meaningful exit codes are:
- 0: scanner ran and found zero violations
- 1: scanner ran and found ≥1 violation
- 2+: argument error, import failure, or runtime exception

A conftest crash (ImportError, SyntaxError) exits with a non-zero code indistinguishable from
a clean "found violations" result at the gate level. An implementer who accidentally introduces
a Python syntax error would see the gate "pass" (non-zero expected in BAD_CASE runs).

**Fix (v2.5):** Exit-code semantics table added to AC-183-009. Gate verification now explicitly
checks: BAD_CASE runs expect exit 1 (not "any non-zero"); GOOD_CASE runs expect exit 0; any
exit ≥2 is an error condition that must fail the gate regardless of case type.

---

### F-W86S-P15-011 (LOW) — cargo e2e 66-finding expectation not flagged as CI-gating

**Story:** STORY-183
**Status:** FIXED v2.5 — CI-gating note added; EXECUTION-REQUIRED flag (ix) added

**Description:** STORY-183 v2.4 references the ITI diverse-capture 66-finding expectation
(PR #439 gate-fix 0ab6f52e) in the context of the E2E integration check. The reference does
not note that this expectation is now enforced by `test_iec104_iti_diverse_e2e_expectations`
in `tests/iec104_analyzer_tests.rs` as a CI-gating test — the first time this count is
machine-enforced in CI.

STORY-183's bin/check-green-doc-tense changes touch comment blocks in `src/` files. If any
edit accidentally alters a detection heuristic (via a comment that contains a probe string),
the 66-finding count could change. The story did not flag this as a delivery-time execution
obligation.

**Fix (v2.5):** CI-gating note added to the EXECUTION-REQUIRED section and referenced from
the AC-183-009 delivery checklist. New flag (ix) added: "Verify `cargo test
test_iec104_iti_diverse_e2e_expectations` passes on the delivery branch (66/20/46 expectation
must not regress)."

---

## NIT-Observations (3 items — all actioned)

1. **NIT-01 (ACTIONED, STORY-182):** move-aside rationale comment in Task procedure body used
   passive voice ("the backup is restored") rather than active imperative ("restore the backup");
   reworded to imperative voice per E-11 writing conventions.

2. **NIT-02 (ACTIONED, STORY-183):** `set -euo pipefail` abbreviated inconsistently across two
   task bodies — one wrote `set -e -u -o pipefail`, the other `set -euo pipefail`; unified to
   `set -euo pipefail` throughout.

3. **NIT-03 (ACTIONED, STORY-183):** A task note cited "see Task 5" as a cross-reference without
   a title; updated to "see Task 5 — PASS/FAIL runner convention" for navigation stability when
   task numbers shift.

---

## EXECUTION-REQUIRED Flags (9 items — carried to delivery)

The following items require execution evidence at delivery or gate time. Items (i)–(viii)
carried from prior passes; item (ix) is new from pass-15.

**(i) Python selftest exit code:** Confirm `python3 bin/test_compute_input_hash.py` exits 0
on develop=e8841d76. Baseline for AC-183-009 item (c).

**(ii) cargo test/clippy on prescribed block:** Confirm `cargo test --all-targets` and
`cargo clippy --all-targets -- -D warnings` both exit 0 on develop=e8841d76 (pre-delivery
baseline; STORY-182/183 add no src/ changes so this is a no-change confirmation).

**(iii) 66-finding expectation vs committed capture (CI-GATING — first time):** Verify
`cargo test test_iec104_iti_diverse_e2e_expectations` passes on the delivery branch with the
66/20/46 expectation. This test is now CI-gating; delivery must not regress it.

**(iv) sha256/size of fetched captures:** Document exact sha256 and byte-size of
`tests/fixtures/iec104-iti-diverse.pcap` (and dissect variant if committed) on
develop=e8841d76 to anchor the AC-182-002 integrity verification claim.

**(v) hermetic harness end-to-end:** Execute Task 9 hermetic harness (copy script into
fresh tmp dir, run against violating.py, confirm FAIL line appears) to verify the hermetic
environment isolation before delivery.

**(vi) ci.yml step behavior on runner:** Confirm that the ci.yml step prescribed in
STORY-182 Task 10 produces the expected "Fixture coverage: N/4 committed" output AND the
explicit count-gate assertion passes on an actual GitHub Actions runner against develop=e8841d76.

**(vii) git ls-files result sets (with dedup):** Document exact file count from
`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py | sort -u | wc -l` on
develop=e8841d76 to anchor STORY-183 "N files scanned" claim per AC-183-008 verification.
Note: dedup via `sort -u` required due to src/*.rs ⊆ src/**/*.rs overlap (F-W86S-P15-007).

**(viii) Pattern self-test exit codes:** Confirm that each of the 8 new TIER-1 patterns
(30-37) produces at least one FAIL line when run against violating.py in tdd_mode on
develop=e8841d76 (post-Task-8b split; predicate-first PASS/FAIL blocks confirmed by
F-W86S-P15-003 fix).

**(ix) cargo e2e 66-finding CI gate (new — P15):** Explicit delivery obligation (see also
item iii above): run `cargo test test_iec104_iti_diverse_e2e_expectations --release` on the
delivery branch and confirm 66/20/46 expectation holds. Failure blocks delivery.

---

## Verified-Clean Table (pass-15 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in annotation text |
| Intra-document :NNN self-citations | CONFIRMED ZERO — structural elimination from D-530 holds through v2.5 |
| Pathspec subsumption direction (7-loci agreement) | CONFIRMED — src/*.rs strictly subsumes src/**/*.rs at all 7 loci in v2.5 |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.4→v2.5 edits |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed; no sibling regressions |
| if:always() loci | CONFIRMED ZERO — P14 fix confirmed carried through v2.5 |
| Task-8 8a/8b split ordering | CONFIRMED — v2.5 preserves P14 split; 8b gate-BLOCKING issue resolved by P15 fix |
| PASS/FAIL-convention predicate-first form | CONFIRMED (post-fix) — all blocks predicate-first in v2.5 |
| ci.yml order-dependence labels | CONFIRMED — present at all absolute citation sites |
| Ground-truth axis | CONFIRMED CLEAN — zero findings |
| Inert-predicate axis | CONFIRMED CLEAN — zero live if:always() loci |
| Policy-compliance axis | CONFIRMED CLEAN — zero findings |

---

## Pass-15 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: medium-low — "v2.3/v2.4-induced second-order regressions; no recurring class."
No HIGH findings. THIRD consecutive zero-HIGH pass (P10, P14, P15).

Severity profile: P13 0C/2H/4M/9L → P14 0C/0H/3M/3L → P15 0C/0H/5M/6L.
P15 tally (14) is higher than P14 (8) but all findings are second-order regressions
from v2.3/v2.4 remediation work; no new conceptual classes introduced.

HIGH count history: P10:0H (first) → P11:1H → P12:1H → P13:2H → P14:0H (second) → P15:0H (third).
Third consecutive zero-HIGH confirms HIGH-severity axis is clean. Streak clock was not advanced
because MED/LOW findings remain.

Pass tallies (P1–P15): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15 / 8 / 14.
Total across all passes: 250 findings. Canonical hashes: 9a0f34c / 9c9b12f.

Pass-16 next after D-532 remediation burst.

---

## Remediation (D-532)

**Date:** 2026-07-26/27
**Burst:** D-532 STATE BURST — WAVE-86 ADVERSARIAL PASS 15 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 14 findings FIXED at STORY-182 v2.5 / STORY-183 v2.5. STATE.md DRIFT-e2e-sibling-harnesses
row corrected per F-W86S-P15-004.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P15-001 (MED) | FIXED v2.5 | STORY-182 | README row rewritten with mandated "ITI CC-BY-4.0" prefix string at prescribed position |
| F-W86S-P15-002 (MED) | FIXED v2.5 | STORY-182 | Source-existence guards added to all 3 move-aside + trap-restore procedures |
| F-W86S-P15-003 (MED) | FIXED v2.5 | STORY-183 | `set -euo pipefail` added to all shell blocks; PASS/FAIL blocks reordered predicate-first |
| F-W86S-P15-004 (MED) | FIXED v2.5 | STORY-182 | bc_2_12_011_story127_tests.rs removed from silent-skip class in STORY-182 Notes + STATE.md DRIFT row corrected |
| F-W86S-P15-005 (MED) | FIXED v2.5 | STORY-183 | Monkey-patch rationale replaced with structural list-position-index constraint explanation |
| F-W86S-P15-006 (LOW) | FIXED v2.5 | STORY-182 | Platform-aware sha256 invocation note + Python hashlib fallback added |
| F-W86S-P15-007 (LOW) | FIXED v2.5 | STORY-182 | `sort -u` dedup step added to git ls-files count assertion |
| F-W86S-P15-008 (LOW) | FIXED v2.5 | STORY-182 | Explicit `[ "$COUNT" -eq 4 ] \|\| exit 1` gate assertion added to ci.yml step |
| F-W86S-P15-009 (LOW) | FIXED v2.5 | STORY-183 | 21-pass derivation note added (8 pattern + 9 AC-183-009 + 4 AC-183-007) |
| F-W86S-P15-010 (LOW) | FIXED v2.5 | STORY-183 | Exit-code semantics table added (0=clean; 1=violations; 2+=error) |
| F-W86S-P15-011 (LOW) | FIXED v2.5 | STORY-183 | CI-gating note added; EXECUTION-REQUIRED flag (ix) added for 66-finding gate |

**3 NITs actioned:** move-aside passive-voice reword (STORY-182); `set -euo pipefail` consistency (STORY-183);
task cross-reference title added (STORY-183).

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Third zero-HIGH pass confirmed:** Pass-15 extends the confirmed-clean HIGH-severity axis.
HIGH count: P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H. Streak 0/3 (MED/LOW remain).

**Streak:** 0/3. Pass 16 pending adversary dispatch. Trajectory-tail: →10→15→8→14.
