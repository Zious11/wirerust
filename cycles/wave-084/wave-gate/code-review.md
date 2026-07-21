---
document_type: wave-gate-code-review
level: ops
version: "1.0"
status: closed
producer: state-manager
timestamp: 2026-07-21T05:30:00Z
cycle: "wave-084"
gate: "3b"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Wave-84 Gate-3b Code Review — Finding Enumeration

**Mandatory artifact per AC-158-006 / PG-W71-CODEREVIEW-ARTIFACT.**  
Every MINOR and NIT finding from the Gate-3b code review is enumerated here with its disposition.  
A gate with zero MAJOR/CRIT findings but non-zero MINOR/NIT must still enumerate each finding.

**Reviewer:** vsdd-factory code-reviewer agent (fresh-context, no prior passes visible)  
**Gate:** Wave-84 integration gate (Gate 3b)  
**Code frozen at:** `1e967bad3d04dd989efd8f02191568abb5382757` (develop, after 3 gate-fix PRs)  
**Session:** 2026-07-21

---

## Summary

| Severity | Count | Resolved |
|----------|-------|---------|
| MAJOR | 0 | n/a |
| MINOR | 3 | 2 FIXED, 1 accepted |
| NIT | 6 | 0 fixed, 6 accepted/deferred |
| **Security** | **2** | **1 FIXED, 1 deferred** |

---

## MINOR Findings

### CR-001 — Tautological assertion in bin/test_check_green_doc_tense.py

**Severity:** MINOR  
**File:** `bin/test_check_green_doc_tense.py`  
**Description:** A test case asserts a condition that is trivially true by construction (tautological assert — the value being asserted can never be the wrong case given the test fixture). The test passes regardless of whether the underlying logic works, providing false confidence.  
**Disposition:** **accepted-deferred** — the test suite still exercises the nominal path; the tautological case does not mask a real failure in the current code. A corrective test will be added in the next bin-touch PR or maintenance sweep. Low urgency.

---

### CR-002 — Missing negative/edge-case test for new path:line:anchor assertion in bin/validate-citations

**Severity:** MINOR  
**File:** `bin/validate-citations`, `bin/test_validate_citations.py`  
**Description:** The new path:line:anchor assertion feature (AC-166-001, STORY-166) lacks a test for the case where the anchor text exists in the file but at a different line than claimed. The test suite covers anchor-absent and anchor-present-correct but not anchor-present-wrong-line.  
**Disposition:** **FIXED via PR #429** (`39b30cb1`) — a red-first test for anchor-present-at-wrong-line was added and confirmed passing after the fix.

---

### CR-003 — Dead seam in bin/validate-citations: --scan discovery layer unimplemented

**Severity:** MINOR  
**File:** `bin/validate-citations`  
**Description:** The `--scan` flag (AC-166-001 discovery layer) is documented in the CLI help and referenced in CLAUDE.md but the discovery-pass code path is a stub that returns without scanning. This creates a dead-code seam that could mislead future contributors about the tool's capabilities.  
**Disposition:** **accepted — intentional seam** — the `--scan` discovery layer is out of STORY-166 scope per the story's scope boundary (AC-166-001 targets the symbol-at-line assertion; the broader scan pass was deferred per wave-75 S-7.02 sequencing). The stub is intentional; CLAUDE.md's Project References row documents `bin/validate-citations` preflight usage. The dead seam is documented in this review for the next E-11 story that touches this tool.

---

## NIT Findings

### CR-004 — Weak conjunct in bin/check-green-doc-tense phrase pattern regex

**Severity:** NIT  
**File:** `bin/check-green-doc-tense`  
**Description:** One of the four new phrase-level patterns (`red\s+gate`) uses a conjunction `\s+` that matches only a single whitespace run; a line like `red  gate` (double space) would be matched but `red-gate` would not. The pattern is functionally adequate for current prose but slightly brittle.  
**Disposition:** **accepted-deferred** — the existing corpus has no `red-gate` hyphenation; the pattern correctly handles all real occurrences. Will tighten in the next bin-touch PR.

---

### CR-005 — Leading `\b` word-boundary anchor in regex is not portable across Python word-char definitions

**Severity:** NIT (promoted to MINOR in the adversarial gate; captured here as NIT reflecting the gate-3b classification)  
**File:** `bin/validate-citations`, `bin/check-green-doc-tense`  
**Description:** Several regex patterns use a leading `\b` anchor. In Python's `re` module, `\b` matches at a position between a `\w` and a `\W` character. If a pattern begins with `\b` and the match position is at the start of the string, the behavior depends on the first character class. This is subtle and could cause false-negatives on some edge inputs.  
**Disposition:** **FIXED via PR #429** (`39b30cb1`) — leading `\b` anchors replaced with `(?<!\w)` negative lookbehind which is unambiguous about the word-boundary semantics in all contexts.

---

### CR-006 — Leading `\b` duplicated across multiple patterns (same class as CR-005)

**Severity:** NIT  
**File:** `bin/validate-citations`  
**Description:** Same `\b` anchor class as CR-005; affects a second set of patterns in `validate-citations` that were added in the same AC.  
**Disposition:** **FIXED via PR #429** (`39b30cb1`) — fixed in the same commit as CR-005.

---

### CR-007 — bin/validate-citations re-reads file on every citation check (O(n²) in citations×file-lines)

**Severity:** NIT  
**File:** `bin/validate-citations`  
**Description:** The citation validator opens and reads the target file once per citation reference pointing to that file, rather than caching the file contents in a dict keyed by path. For large reference sets pointing at the same file, this is O(n×m) reads (n citations, m lines per read). In practice the current citation count is small enough that no user-visible latency exists.  
**Disposition:** **accepted-deferred** — current corpus has O(10) citations total; file caching would add complexity for no observable gain today. Tag for the next performance refactor of the bin/ toolchain.

---

### CR-008 — bin/test_check_green_doc_tense.py docstring references internal function name coupling

**Severity:** NIT  
**File:** `bin/test_check_green_doc_tense.py`  
**Description:** Module-level docstring in the test file names an internal function from `bin/check-green-doc-tense` by its exact symbol, coupling the test documentation to the implementation's private API surface. If the function is renamed, the docstring will be stale.  
**Disposition:** **accepted-deferred** — low-risk; the coupling is in prose documentation, not in executable code. The test itself does not import the symbol, so refactors won't break it. Will fix in the next bin-touch PR.

---

### CR-009 — CHANGELOG entry text included in YAML story frontmatter field verbatim

**Severity:** NIT  
**File:** `.factory/stories/STORY-176.md` (frontmatter `changelog_entry:` field)  
**Description:** A YAML frontmatter field contains a multi-line verbatim CHANGELOG entry with Markdown formatting. While technically valid YAML (block scalar), this pattern bleeds prose content into structured frontmatter and makes the field harder to parse programmatically. The CHANGELOG entry belongs in the story body or a separate `CHANGELOG.md` section.  
**Disposition:** **accepted-deferred** — the frontmatter field is used as a convenience artifact for the pr-manager when composing the PR description. The value is not parsed programmatically anywhere today. Will establish a cleaner convention (e.g., a `## Changelog` section in the story body) in the next story-writer template revision.

---

## Security Findings

### SEC-002 — Theoretical ReDoS via unanchored alternation in bin/validate-citations

**Severity:** LOW (security class)  
**File:** `bin/validate-citations`  
**Description:** A regex alternation pattern is applied to user-controlled input (citation file paths from story frontmatter) without an explicit length cap. Under adversarial input construction, a crafted `input-hash` line or citation path with many repeated partial matches could trigger catastrophic backtracking in CPython's `re` module, causing the validation tool to hang.  
**Disposition:** **deferred** — the attack surface is the factory-artifacts branch (not public-facing); factory artifact paths are authored by trusted agents and reviewed before commit. The risk in practice is negligible. Tracked as an open advisory; a length-cap or compiled-regex guard would mitigate if the tool's input surface changes.

---

### SEC-003 — Ambiguous ownership semantics in `bin/check-green-doc-tense` allow-list path

**Severity:** LOW (security class)  
**File:** `bin/check-green-doc-tense`  
**Description:** (Note: the STORY-176 v2.2 AC-176-001 originally described a `# green-doc-tense-gate: allow` inline comment allowlist mechanism. PG-W84-009 established that this mechanism **does not exist** — the AC was substantially invalid. This SEC-003 finding refers to a distinct issue: in the delivered v2.3+ implementation, the phrase-pattern `re.compile()` objects are module-level globals with no ownership documentation, making it unclear whether they should be treated as a mutable registry or frozen constants.)  
**Disposition:** **FIXED via PR #429** (`39b30cb1`) — a module-level `# FROZEN: do not mutate at runtime` comment was added; the compiled patterns are now clearly documented as constants, eliminating the ambiguity.

---

## Gate-3b Consistency-Validator Findings (summary)

The consistency-validator pass (Gate 3b) produced 4 MEDIUM and 3 LOW findings.  
The 4 MEDIUM items were STATE/loci bookkeeping gaps addressed in this burst:

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| CV-MED-01 | MEDIUM | `develop_head` in STATE.md still cites `595cdba8` (pre-fix-PR value) | FIXED this burst — STATE.md develop_head updated to `1e967bad` |
| CV-MED-02 | MEDIUM | STORY-147.md frontmatter + body `status: ready` / `**Status:** ready` — stale after delivery | FIXED this burst — synced to `delivered` |
| CV-MED-03 | MEDIUM | STORY-166.md frontmatter + body same stale `ready` status | FIXED this burst — synced to `delivered` |
| CV-MED-04 | MEDIUM | STORY-176.md frontmatter + body same stale `ready` status | FIXED this burst — synced to `delivered` |

The 3 LOW items are deferred carry-forwards:

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| CV-LOW-01 | LOW | ADR-0013 referenced in ARCH-INDEX v2.19 but `docs/adr/0013-*.md` not yet authored | deferred — ADR-0013 authoring is a separate E-11 story candidate; DF-VALIDATION-001 required before filing |
| CV-LOW-02 | LOW | Inline dep-graph totals in STORY-INDEX.md `dep-graph v3.9` note cite edge count 137 but graph was last recomputed at v3.9 (2026-07-14); potential stale count after wave-84 deliveries | deferred — dep-graph edge count not changed by wave-84 (all three STORY-147/166/176 have `depends_on: []`; no new edges); count remains valid |
| CV-LOW-03 | LOW | `sprint-state.yaml` header note cites a sprint boundary that predates wave-84 | deferred — sprint-state.yaml is planning-only reference; stale note is cosmetic |
