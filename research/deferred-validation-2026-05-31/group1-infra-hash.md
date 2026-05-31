# Deferred-Finding Validation — Group 1 (Infra / Input-Hash)

**Date:** 2026-05-31
**Branch validated:** `develop` @ HEAD `9954d44` (per `.factory/STATE.md` line 105)
**Validator:** vsdd-factory:research-agent
**Policy authority:** DF-VALIDATION-001 (research-validated deferred findings, HIGH) — this
document is the mandatory pre-issue validation for both items below.
**Scope:** F-W21-TOOL-001 (HIGH, infra-gap) and F-W21-S079-HASH (MEDIUM, process-gap).

---

## Item 1 — F-W21-TOOL-001 [HIGH, infra-gap]

### Claim
The canonical tool `bin/compute-input-hash` — referenced by policy DF-INPUT-HASH-CANONICAL-001
(and ostensibly CLAUDE.md) — does NOT exist in the repo, rendering all input-hash freshness
checks un-runnable. Policy says the tool computes "MD5 and inputs in declaration order"; hashes
are stored as 7 hex chars in story frontmatter.

### VALIDITY: CONFIRMED-REAL (HIGH severity upheld, with one correction)

**Sub-claim 1 — tool absence: CONFIRMED.**
- No `bin/` directory exists. `Glob bin/**` → no files. `Glob **/compute-input-hash*` → no
  files. `Glob scripts/**`, `Glob tools/**` → no files. The repo has no script directory of
  any kind at the root.
- The string `compute-input-hash` appears in **zero** non-prose locations. `Grep` across the
  whole repo finds it only inside `.factory/policies.yaml` (the policy that mandates it),
  `.factory/STATE.md` (the two deferred findings themselves), and
  `.factory/cycles/drift-remediation-2026-05-29/{lessons.md,closed-items.md}` (the prose that
  first named it). It is never invoked, never defined, never committed.
- I cannot run `git log --all -- bin/compute-input-hash` (no Bash in this profile), but the
  working-tree evidence is decisive: the tool is not present on `develop` HEAD, and no artifact
  anywhere in `.factory/` ever quotes its source, a shebang, a language, or a sample invocation
  beyond the bare flags `--scan` / `--check` / `--update`. The tool was described in prose
  (2026-05-29 drift remediation) but, on the evidence in the tree, **was never authored as a
  committed file** — it appears to be a phantom: a tool that agents *believed* they were running
  ("--scan reports MATCH=48 STALE=0") but which has no on-disk existence. This makes the HIGH
  classification correct and arguably understated: the corpus-wide "MATCH=48 STALE=0"
  attestation in `lessons.md` (line 71) is **unverifiable** because the verifier does not exist.

**Sub-claim 2 — CLAUDE.md reference: MISCHARACTERIZED (minor).**
- The current `CLAUDE.md` (read in full, 73 lines) does **not** reference `bin/compute-input-hash`.
  It references DF-VALIDATION-001 and `.factory/policies.yaml` but contains no input-hash tooling
  section. The finding text and the STATE.md drift row (line 145) both assert "referenced by
  CLAUDE.md" — that is not true of the current file. The only live reference is in
  `policies.yaml` (DF-INPUT-HASH-CANONICAL-001, lines 643–684). This does not weaken the finding;
  it just means the citation should read "referenced by policy DF-INPUT-HASH-CANONICAL-001" and
  drop "CLAUDE.md."

### CRITICAL sub-task — Is the algorithm recoverable?

**Finding: the recipe is GENUINELY LOST, and — more importantly — it was never unambiguously
specified in the first place. The two surviving descriptions CONTRADICT each other.**

There are three written descriptions of the algorithm, and they do not agree:

| Source | Location | Description | Hashed material |
|--------|----------|-------------|-----------------|
| Policy DF-INPUT-HASH-CANONICAL-001 | `policies.yaml:646-647` | "MD5 and inputs in declaration order" | **ambiguous** — names or contents? |
| Drift lessons DR.L5 | `lessons.md:61-62` | "MD5 over the inputs-order **file list**" | reads as **file *names*** (the list), not contents |
| Orchestrator task framing (this dispatch) | — | "MD5 of concatenated declared-input file **contents** in declaration order, first 7 hex" | file **contents** |

The orchestrator already brute-forced 18 content-based MD5 recipes against the two known-good
hashes (STORY-086 `4a6449b`, 4 BC inputs; STORY-087 `1de3972`, 3 BC inputs) and **none**
reproduced either stored hash. I independently reviewed the additional hypotheses requested:

- **Does the hash cover `traces_to` (prd.md) or the story file itself?** STORY-086's `inputs:`
  list is exactly 4 BC files and does NOT list `prd.md`; STORY-087's is 3 BC files, no `prd.md`.
  STORY-079, by contrast, DOES list `prd.md` as a 4th input. So "inputs" is literally the
  frontmatter `inputs:` array, whatever it contains — not a fixed "BCs only" or "BCs + prd" rule.
  A recipe that always appends prd.md would be wrong for 086/087; a recipe that hashes exactly
  the listed inputs is the only consistent reading. (This was almost certainly within the 18
  variants already tried, since the inputs array is the declared input set.)
- **Frontmatter-stripped content / story-file content:** not separately testable without running
  MD5 (no Bash), but note that 086/087 inputs are BC files, not the story file, so "hash the
  story body" is excluded by the inputs-list semantics.
- **Different truncation (last-7, mid-substring):** plausible but unfalsifiable here — and even
  if a last-7 or offset-substring variant happened to reproduce both 7-char values, two 28-bit
  targets provide only ~56 bits of constraint against an unbounded recipe space (concatenation
  order, separators, trailing-newline handling, CRLF-vs-LF, path-prefixing, encoding, truncation
  offset). A match found by brute force over that space would be **coincidental, not
  evidential** — it would not prove it is the original recipe, and would silently mis-fire on the
  46 other stories. This is the core reason reconstruction is unsound even if a candidate
  "passes": two short truncated hashes cannot uniquely identify a hashing protocol.

**The decisive fact:** the surviving prose says the tool hashed the "**file list**" (names),
while the corpus-freshness intuition (and the orchestrator's framing) assumes it hashed
**contents**. If DR.L5 is literally accurate (MD5 of the *list of paths*), then the hash would
NOT change when a BC's content changes v1.2→v1.3 — which would make the entire F-W21-S079-HASH
"staleness" concern moot, and would mean the hashes never detected content drift at all, only
input-set drift. The two descriptions imply two different tools with two different purposes, and
**no committed source exists to adjudicate between them.** That is the definition of an
unrecoverable recipe.

**Conclusion: algorithm is LOST and was never deterministically pinned. Do not reconstruct.**

### External research — re-baseline vs reconstruction

Perplexity reasoning (multi-source) and a second cross-validation search both land
unambiguously on **re-baseline with a new, documented, deterministic algorithm**, NOT
reconstruction. Key grounded points:

1. **The hash is an internal drift-detector, not a public/contractual identifier.** Nothing
   external (no cache, no third-party system) keys off these 7-char values. That places it in the
   same category as build-cache keys and lockfile checksums, where the standard move on a recipe
   change is "invalidate and recompute," not "preserve old values."
   [NIST SP 800-53r5 — integrity/config baselines must use documented, reproducible mechanisms;
   Sommerville, *Software Engineering* 9e — baselines are versioned artifacts re-creatable from a
   specified process.]

2. **Precedent is uniform across content-addressable / lockfile systems:** git's SHA-1→SHA-256
   migration defines a *new* object format rather than back-compatible IDs; `package-lock.json`
   and `Cargo.lock` carry a `version`/`lockfileVersion` field and are rewritten wholesale on
   format change; Bazel remote-cache keys are simply invalidated and rebuilt when the key recipe
   changes; DVC recomputes and rewrites `.dvc` metadata when hashing config changes. In none of
   these does anyone reverse-engineer or preserve a lost short hash.
   [docs.github.com "Dealing with line endings"; the lockfile-version pattern is standard npm/Cargo.]

3. **Two short truncated hashes cannot validate a reconstructed recipe** — even a brute-forced
   match is coincidental and will mis-fire on edge cases (CRLF/LF, trailing newline, encoding,
   separator). Reconstruction leaves you with an undocumented, fragile recipe you must document
   and test *anyway* — strictly more work than defining a clean one.

4. **Determinism / CRLF pitfall is the central design risk** and is directly relevant here:
   STORY-079's input BC-2.11.020 was just corrected CRLF→LF (2026-05-30). A content hash that
   reads working-tree bytes without line-ending normalization is non-deterministic across
   machines and across `core.autocrlf` settings. The canonical algorithm MUST normalize line
   endings (replace `\r\n` and bare `\r` with `\n`) before hashing, or hash the git **blob**
   bytes (which are LF-normalized in-repo when `.gitattributes` enforces `text=auto`/`eol=lf`).
   [mybluelinux.com, dev.to/kevinshu, docs.github.com on git CRLF normalization +
   `git add --renormalize`.]

### RECOMMENDED FIX (decisive)

**Re-baseline. Author one clean canonical tool and regenerate all 48 story hashes in a single
mechanical commit. Do not attempt reconstruction.**

Concrete recommendation:

- **Language: Python** (a single `bin/compute-input-hash` script with `#!/usr/bin/env python3`).
  Rationale: it must be runnable by the orchestrator and by CI without a compile step; it parses
  YAML frontmatter and walks an inputs list. A Rust bin would couple a meta-tooling script to the
  crate's build and is the wrong layer. Python3 is already assumed available in this dev/CI
  environment (the factory tooling is Python/agent-driven). A POSIX `sh` + `md5`/`md5sum` variant
  is viable but brittle for YAML parsing and for cross-platform `md5` vs `md5sum` naming — Python
  is the lower-risk choice.

- **Canonical algorithm (specify exactly, then it becomes ground truth — its disagreement with
  any old hash is expected and fine):**
  1. Parse the story's YAML frontmatter; read the `inputs:` list **in declaration order** (do not
     sort).
  2. For each input path, read the file as UTF-8 bytes; **normalize line endings**: replace
     `\r\n` → `\n`, then bare `\r` → `\n`.
  3. Concatenate the normalized contents in declaration order. Recommended: prefix each file with
     its repo-relative path + `\n` then content + `\n`, so a rename or reorder also registers as
     drift and there is no concatenation-boundary ambiguity. (Document whichever choice is made.)
  4. `md5(concatenated_bytes)`, lowercase hex, **first 7 characters**.
  5. Modes: `--scan` (report MATCH/STALE counts across all stories), `--check <story>` (exit
     nonzero on mismatch), `--update <story>` (rewrite the `input-hash` field).

- **Re-baseline all 48 stories** in one commit (`chore: re-baseline story input-hash via new
  canonical compute-input-hash tool`). Record in the commit body that pre-existing hashes were
  produced by a lost, non-reproducible legacy tool and are superseded. Add a CRLF-vs-LF unit test
  and run `--scan` in CI as the freshness gate.

- **Future-proof:** add an `input-hash-algo: v2` (or tool-version) marker so the next algorithm
  change is a clean versioned migration rather than another archaeology exercise.

- **Citation cleanup:** correct the finding/STATE row to drop "referenced by CLAUDE.md" (the
  current CLAUDE.md does not reference the tool); the live reference is
  DF-INPUT-HASH-CANONICAL-001.

**Issue-readiness:** VALID per DF-VALIDATION-001 — finding is real and open on `develop` HEAD
`9954d44`. Eligible to be filed as a GitHub issue (infra-gap, HIGH: author canonical
`bin/compute-input-hash` + re-baseline 48 hashes + CI freshness gate), with the citation
correction above folded in.

---

## Item 2 — F-W21-S079-HASH [MEDIUM, process-gap]

### Claim
STORY-079's input-hash `903f0d0` is likely stale because its input BC-2.11.020 changed v1.2→v1.3
(CRLF→LF correction, 2026-05-30); cannot recompute because the canonical tool is missing.

### VALIDITY: CONFIRMED-REAL but UNDER-STATED — the input is now at v1.4, not v1.3

- **STORY-079 frontmatter (verified):** `input-hash: "903f0d0"`; `inputs:` =
  `BC-2.11.020.md, BC-2.11.021.md, BC-2.11.022.md, prd.md` (4 inputs, in that declaration order);
  story `version: "1.3"`, `wave: 21`. (`.factory/stories/STORY-079.md:10-16`.)
- **BC-2.11.020 version (verified):** the BC frontmatter now reads `version: "1.4"`
  (`.factory/specs/behavioral-contracts/ss-11/BC-2.11.020.md:4`) — **past the v1.3 named in the
  finding.** So the input has changed at least twice (→v1.3 CRLF→LF on 2026-05-30, then →v1.4),
  and the stored `903f0d0` predates both. The staleness concern is therefore **stronger** than
  the finding states, not weaker. (Severity MEDIUM is appropriate — it is a single-story
  bookkeeping staleness, not a correctness defect in shipped code; STORY-079 is `status: draft`
  with ZERO src changes.)
- **BLOCKED on Item 1: CONFIRMED.** With no `bin/compute-input-hash`, the hash cannot be
  recomputed or even *checked* by the canonical (and policy-mandated, hand-compute-forbidden)
  mechanism. DF-INPUT-HASH-CANONICAL-001 forbids hand-computation, so there is literally no
  sanctioned way to resolve this item while Item 1 is open. This is a strict dependency.

### Does it resolve automatically once Item 1 is fixed?

**YES — fully automatic, no separate work.** The recommended fix for Item 1 regenerates ALL 48
story hashes via the new canonical tool in a single commit. STORY-079 is one of those 48; its
`903f0d0` will be overwritten with the freshly-computed value over the current v1.4 content of
BC-2.11.020 (and the current 021/022/prd.md). There is **no STORY-079-specific remediation** —
re-baselining the corpus subsumes it. Caveat: this is only true if the new algorithm hashes
*contents* (so a v1.3→v1.4 content change is reflected). If the legacy tool genuinely hashed the
file-*list* (per DR.L5's wording), the old hash would never have been content-sensitive and this
finding would have been moot from the start — another reason the contents-based canonical
algorithm in Item 1's fix is the right choice: it makes the freshness check actually mean what
everyone assumed it meant.

**Issue-readiness:** VALID per DF-VALIDATION-001, but should NOT be filed as a standalone issue.
Recommend recording it as a **sub-task / acceptance-criterion of the Item 1 issue** ("after
re-baseline, confirm STORY-079 input-hash is recomputed over BC-2.11.020 v1.4"). Filing it
separately would create a guaranteed-duplicate that closes the moment Item 1's re-baseline lands.

---

## Summary Table

| Item | Validity | Severity | Disposition |
|------|----------|----------|-------------|
| F-W21-TOOL-001 | CONFIRMED-REAL (citation "CLAUDE.md" mischaracterized → use DF-INPUT-HASH-CANONICAL-001). Algorithm GENUINELY LOST + never unambiguously specified (prose contradicts: "file list" vs "contents"). | HIGH (upheld) | File issue: author Python `bin/compute-input-hash` with a NEW documented, LF-normalized, contents-based MD5/first-7 algorithm; re-baseline all 48 hashes in one commit; add CI freshness gate + algo-version marker. Do NOT reconstruct. |
| F-W21-S079-HASH | CONFIRMED-REAL, under-stated (input BC-2.11.020 is now **v1.4**, finding said v1.3). Strictly BLOCKED on Item 1. | MEDIUM (upheld) | Do NOT file standalone — fold into Item 1 as a sub-task/AC. Resolves automatically when the corpus re-baseline runs. |

## Decisive answer on the re-baseline question
**Re-baseline, unambiguously.** The recipe is lost and was never deterministically specified; the
hash is an internal drift-detector with no external consumers; two 28-bit truncated targets cannot
validate any reconstructed recipe; and every comparable system (git SHA migration, Cargo.lock,
package-lock.json, Bazel cache keys, DVC) handles a lost/changed hash recipe by versioning and
recomputing wholesale, never by reverse-engineering. Define one clean canonical algorithm
(LF-normalized MD5 of declared-input contents in declaration order, first 7 hex), regenerate all
48 hashes in a single commit, and gate freshness in CI.

---

## Sources

External (web, via Perplexity — verified May 2026):
- NIST SP 800-53 Rev. 5 — Security and Privacy Controls (integrity/configuration baselines must
  use documented, reproducible mechanisms): https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-53r5.pdf
- Ian Sommerville, *Software Engineering*, 9th ed. (baselines as versioned, re-creatable
  artifacts): https://engineering.futureuniversity.com/BOOKS%20FOR%20IT/Software-Engineering-9th-Edition-by-Ian-Sommerville.pdf
- GitHub Docs — "Dealing with line endings" (CRLF/LF normalization, `git add --renormalize`):
  https://docs.github.com/articles/dealing-with-line-endings
- "Normalizing line endings in Git: CRLF vs LF": https://www.mybluelinux.com/normalizing-line-endings-in-git-crlf-vs.-lf/
- "Git and normalization of line endings" (dev.to/kevinshu): https://dev.to/kevinshu/git-and-normalization-of-line-endings-228j

Internal (codebase, ground-truth — `develop` HEAD `9954d44`):
- `.factory/policies.yaml:643-684` — DF-INPUT-HASH-CANONICAL-001 ("MD5 and inputs in declaration
  order"; "tool is at bin/compute-input-hash")
- `.factory/cycles/drift-remediation-2026-05-29/lessons.md:57-77` — DR.L5/PG-HASH-001
  ("MD5 over the inputs-order **file list**"; "--scan reports MATCH=48 STALE=0")
- `.factory/stories/STORY-079.md:10-16` — inputs (4) + `input-hash: "903f0d0"`
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.020.md:4` — `version: "1.4"`
- `.factory/stories/STORY-086.md:10-15` — inputs (4) + `input-hash: "4a6449b"`
- `.factory/stories/STORY-087.md:10-14` — inputs (3) + `input-hash: "1de3972"`
- `.factory/STATE.md:105,108,144-145` — develop HEAD + both deferred-finding drift rows
- `CLAUDE.md` (full, 73 lines) — confirms NO reference to `bin/compute-input-hash`
- Tool absence: `Glob bin/**`, `**/compute-input-hash*`, `scripts/**`, `tools/**` all empty;
  `Grep compute-input-hash` finds only prose/policy/STATE occurrences, zero invocations/source.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity reason | 1 | Re-baseline vs reconstruction best practice; lockfile/CAS migration precedent; determinism + CRLF pitfalls |
| Perplexity search | 1 | Cross-validation: truncated-MD5 determinism, git blob/CRLF normalization, recompute-on-lost-tool |
| Glob | 5 | Confirm absence of bin/, compute-input-hash, scripts/, tools/; locate BC/story files |
| Grep | 5 | Search whole repo for tool name, algorithm description, MD5/input-hash references |
| Read | 6 | CLAUDE.md, policies.yaml, STORY-079/086/087, BC-2.11.020, lessons.md, STATE.md |
| Training data | 0 areas | None relied upon for claims — all version/algorithm facts verified against tree; all external claims cited |

**Total MCP tool calls:** 2 (Perplexity) + 21 (local Read/Grep/Glob) = 23
**Training data reliance:** low — every factual claim is grounded in either a read file (with
line citation) or a cited external source. The contradiction between the two algorithm
descriptions and the BC-2.11.020 v1.4 correction were both discovered by direct file reads, not
assumed.
