---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-28T00:00:00Z
cycle: "wave-086"
pass: 22
verdict: NOT_CONVERGED
novelty: "medium-low"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 22

**Date:** 2026-07-28
**Pass:** 22 of N

> **NOTE:** The verbatim adversary transcript was unavailable at /tmp/pass22.md at burst
> execution time (PG-W86-ADVERSARY-WRITE-PROFILE: adversary is read-only, returns text to
> orchestrator, does not write files). This file was constructed from orchestrator-verified
> ground truth (D-539 brief, §2). All finding IDs, severities, and descriptions are
> authoritative as recorded by the orchestrator.

---

## Summary

**Verdict:** NOT_CONVERGED
**Tally:** `6: 0C/0H/3M/1L/2N`
**Novelty:** MEDIUM-LOW (1 genuinely new; 1 new instance of known class; 2 induced by pass-21 burst)
**Tenth consecutive zero-HIGH pass** (P10, P14–P22)
**Trajectory tail:** →15→10→6. MEDIUM sub-trajectory: 9→4→3. Best pass of the wave on every measure.

**Axes clean (7 of 11):** 1 truthfulness, 4 false-GREEN, 5 self-referential-flag, 8 environment-blindness,
9 arithmetic, 10 scope integrity, 11 CI/gate realism.

**Axis 7 clean** in both regions specifically flagged for scrutiny (STORY-182 Task 8+Task 10a;
AC-182-006) — v2.11 contradiction fixes held.

**Axis 1 (truthfulness):** ~70 factual claims independently re-derived, zero false.

**Axis 9 (arithmetic):** Every count re-derived exact: 40/2/42, 45, 13+1=14, 12+14=26, 28+8=36,
32 RED-GATE headings, 3625 attribute lines, 10 fallthrough sites, 25 captures, 21 TCs.

**Axis 5 (self-referential-flag):** Hand-simulated all 36 patterns against every `#`-comment in
all six `bin/*.py` files — only 2 match, exactly as the story claims.

---

## Two New Audits Added This Pass

**AUDIT 4 — Hardcoded absolute line-range extractors in predicates**

Closes the gap that AUDIT 2 evaluates the baseline tree only and cannot see a predicate that
discriminates before the change and goes inert after it. This is precisely how F-W86S-P22-001
slipped through pass 21. Found exactly 1 locus, as the adversary predicted.

Script (verbatim executable spec):
```bash
# AUDIT 4: Find hardcoded absolute line-range extractors in AC predicates
# Covers: sed -n 'N,Mp', awk 'NR>=N' / 'NR==N', head -n N | tail
grep -nE "sed -n '[0-9]+,[0-9]+p'|awk 'NR>?=[0-9]+|head -n [0-9]+ \| tail" \
  .factory/stories/STORY-182.md \
  .factory/stories/STORY-183.md
```

**AUDIT 5 — Pipeline fences whose head lacks `set -euo pipefail`**

Closes AUDIT 1's blindness to single-command fences where a missing `pipefail` masks a
non-final-stage failure. Found 2 loci, both verified fail-closed.

Script (verbatim executable spec):
```bash
# AUDIT 5: Find bash fences in story files with a pipeline but no set -euo pipefail at head
# A "pipeline fence" is a ```bash block containing at least one | operator.
python3 - <<'EOF'
import re, sys
for fname in sys.argv[1:]:
    content = open(fname).read()
    for m in re.finditer(r'```bash\n(.*?)```', content, re.DOTALL):
        block = m.group(1)
        has_pipeline = '|' in block
        has_pipefail = 'set -euo pipefail' in block or 'set -e' in block
        if has_pipeline and not has_pipefail:
            start = content[:m.start()].count('\n') + 1
            print(f"{fname}:{start}: pipeline fence missing set -euo pipefail")
EOF .factory/stories/STORY-182.md .factory/stories/STORY-183.md
```

---

## Five Orchestrator Mechanical Audits (Post-Burst, All Clean)

- `AUDIT1=0` (fence head `set -euo pipefail`, >1 command)
- `AUDIT2=2 correctly-guarded only` (both `coverage-out.txt` predicates preceded by `test -s` in-fence; changelog rows excluded)
- `AUDIT3=0` (assignment-position substitution under `set -e`)
- `AUDIT4=0` (hardcoded absolute line-range extractors — NEW this pass)
- `AUDIT5=0` (pipeline fences without `set -euo pipefail` — NEW this pass)

---

## Findings

---

### F-W86S-P22-001 — MEDIUM — STORY-182 — Line-range drift in AC-182-006 predicate

**Severity:** MEDIUM
**Story:** STORY-182
**Axis:** 2 (AC predicate correctness)

**Description:**
AC-182-006's predicate used `sed -n '337,345p' | grep -c` to check the IEC-104 section of
E2E-PCAPS.md. This predicate discriminated on the v2.11 baseline but would go **inert after
the story's own edits**: Task 7 edits E2E-PCAPS.md `:3-6` and `:48-50`, both above `:337`,
and the target sentence at `:340` had only 5 lines of slack in a 9-line window; ≥6 lines of
growth pushes it out → predicate passes vacuously. Also under-inclusive (the IEC-104 section
spans `:337-390`).

**Fix applied (D-539):**
Replaced with a content-anchored form:
```bash
awk '/^## IEC 60870-5-104/{f=1;next} /^## /{f=0} f' docs/E2E-PCAPS.md | grep -c
```
Verified to return 1 on baseline (discriminating) and drift-proof against insertions above
the section heading.

---

### F-W86S-P22-002 — MEDIUM — STORY-182 — `red-out.txt` sweep incomplete

**Severity:** MEDIUM
**Story:** STORY-182
**Axis:** 3 (AC coverage completeness)

**Description:**
The v2.11 row claimed "all four loci" for the `red-out.txt` sweep. Three live loci remained
singular and the verifying AC had no predicate at all. The claim was unverifiable without a
confirming residual grep; the pre-existing changelog row asserted exhaustiveness without
evidence.

**Fix applied (D-539):**
Added `grep -qF 'red-out.txt' .gitignore` to AC-182-006 (baseline 0 → fully discriminating).
Pluralised Task 11 and Notes §Develop PR. Confirming residual grep documented.

---

### F-W86S-P22-003 — MEDIUM — STORY-182 — Deliverable with no Task

**Severity:** MEDIUM
**Story:** STORY-182
**Axis:** 6 (deliverable↔task coverage)

**Description:**
The additive `ci.yml` step was declared in five places (AC-182-004(e), AC-182-006 predicates,
Architecture Mapping, FSR, ACR) but no Task instructed creating it — asymmetric with Task 10,
which prescribes the CLAUDE.md row and `.gitignore` lines verbatim. This was traceable to the
D-520 F-014 orchestrator ruling never being carried into a Task.

**Fix applied (D-539):**
Added Task 10 sub-bullet **(c)** carrying the step name, `if: ${{ !cancelled() }}`, placement
after `ci.yml:47`, and the four-line `run:` block verbatim from the ACR.

---

### F-W86S-P22-004 — LOW — STORY-183 — AC-183-006 type description wrong

**Severity:** LOW
**Story:** STORY-183
**Axis:** 1 (truthfulness)

**Description:**
AC-183-006's paraphrase said AC-183-001 "returns a **set** containing
`bin/test_check_green_doc_tense.py`" — wrong type (source: `-> list[Path]`) and describing
the repo-relative-string comparison AC-183-001 explicitly forbids. Third locus missed by a
two-locus `replace_all` at v2.6.

**Fix applied (D-539):**
Corrected to `list[Path]` + compare-by-`.name`.

---

### N-1 — NIT — STORY-182 — README.md predicate flag

**Severity:** NIT
**Story:** STORY-182

**Description:**
README.md predicate given its own bullet; `-q` flag changed to `-qF` for literal-string
matching discipline.

**Fix applied (D-539):** `-q` → `-qF`, bullet isolated.

---

### N-2 — NIT — STORY-183 — AC-183-001 unprescribed third class assertion

**Severity:** NIT
**Story:** STORY-183

**Description:**
AC-183-001's unprescribed third class assertion folded into the `.name` bullet as an
explanatory note.

**Fix applied (D-539):** Folded into `.name` bullet as explanatory note.

---

## AUDIT 5 Hardening (Consistency, Not Bug Fix)

`set -euo pipefail` added to two pipeline-bearing fences (STORY-182 Task 2 sha256;
STORY-183 AC-183-005 changelog-gate). **Both were already fail-closed — verified**
(`changelog-gate-check` exits 1 on empty stdin and on no-plus-lines; the shasum form
yields `test "" = "<hash>"` → exit 1). Recorded as consistency hardening, NOT fixing a
live false-GREEN (truth-preservation, D-530).

---

## Remediation Status

All 6 findings remediated in D-539 burst. STORY-182 v2.11→v2.12, STORY-183 v2.11→v2.12.
Canonical hashes unchanged: `9a0f34c` / `9c9b12f` (inputs: unchanged).
Streak: 0/3. Pass 23 next.
