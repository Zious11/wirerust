---
document_type: red-gate-log
level: ops
version: "1.0"
status: draft
producer: test-writer
timestamp: 2026-07-09T00:00:00Z
phase: f7
story_id: STORY-161
wave: "72"
inputs: []
input-hash: "d41d8cd"
traces_to: .factory/stories/STORY-161.md
stub_architect_agent: "n/a — governance-only story; no Rust stubs"
stub_compile_verified: false
test_writer_agent: test-writer
red_gate_verified: true
---

# Red Gate Log: Wave-72 / STORY-161

## Summary

STORY-161 is a **governance-only** story (E-11 convention). There are no Rust source
files, no BCs, and no test functions. The acceptance criteria are verified by file-content
greps and structural file checks. The "tests" for this story are those AC verification
commands; the Red Gate is established by confirming that every AC currently FAILS
(i.e., the required text is absent, the required fields are absent, or the required
values are null).

| Story | Verification Method | All Checks Red? | Gate |
|-------|---------------------|-----------------|------|
| STORY-161 | AC grep/field checks (7 ACs) | YES — 6 of 6 checked ACs are RED; cargo check GREEN (expected pass) | RED GATE ESTABLISHED |

---

## Red Gate Verification

### AC-161-001 — Mini-Merkle algorithm codified in VP-INDEX

**Check command:**
```bash
grep -n "proof.*anchor\|proof_file_hash" \
  .factory/specs/verification-properties/VP-INDEX.md | head -20
```

**Expected (GREEN state):** Output includes a heading "Multi-File Proof Anchor Algorithm"
or equivalent locatable prose.

**Observed (RED state):** The grep returns hits only within the `modified:` YAML
frontmatter field (inline historical changelog entries). No standalone section titled
"Multi-File Proof Anchor Algorithm" exists in VP-INDEX.md. The required prose defining
the mini-Merkle construction, LF normalization rule, and detection coverage note is absent.

**Current VP-INDEX version:** `"2.38"` (confirmed from frontmatter line 4). Expected
post-implementation version: `"2.39"`.

**Status: RED (expected)**

---

### AC-161-002 — Section-Scoping Rule explicit in VP-INDEX

**Check command:**
```bash
grep -A3 -E "Section-Scoping Rule|section.scoping|closing brace" \
  .factory/specs/verification-properties/VP-INDEX.md | head -20
```

**Expected (GREEN state):** Non-empty output containing the Section-Scoping Rule text
about the `#[cfg(kani)] mod kani_proofs { ... }` block boundary.

**Observed (RED state):** The grep returned empty output. No "Section-Scoping Rule",
"section.scoping", or "closing brace" text appears anywhere in VP-INDEX.md. The required
standalone named rule defining the byte-inclusive boundary (`#` of `#[cfg(kani)]`
through the matching closing `}` of `mod kani_proofs`) is absent.

**Status: RED (expected)**

---

### AC-161-003 — VP-024 proof_file_hash populated

**Check:** VP-024 frontmatter `proof_file_hash:` field value.

**Expected (GREEN state):** A 64-character lowercase hex string (no `null`, no comment).

**Observed (RED state):** Line 22 of vp-024-arp-parse-safety.md reads:
```
proof_file_hash: null  # No canonical recomputation method defined for VP-024 proof files. Follow-up: define hash method (e.g. SHA-256 of src/decoder.rs kani_proofs + src/analyzer/arp.rs kani_proofs modules) and populate. Tracked as FU-F6-KANI-CLEANUP.
```
The field holds `null` with the `FU-F6-KANI-CLEANUP` comment. The mini-Merkle hash
has not been computed or written.

**Current VP-024 version:** `"2.4"` (confirmed from frontmatter line 4). Expected
post-implementation version: `"2.5"`.

**Status: RED (expected)**

---

### AC-161-004 — VP-024 gains kani_version frontmatter field

**Check:**
```bash
grep -n "kani_version" \
  .factory/specs/verification-properties/vp-024-arp-parse-safety.md
```

**Expected (GREEN state):** At least one line containing `kani_version:`.

**Observed (RED state):** The grep returned no output (empty). The `kani_version:` field
does not exist in VP-024 frontmatter. The sibling placement alongside `proof_file_hash:`
and `verified_at_commit:` is absent.

**Status: RED (expected)**

---

### AC-161-005 — VP-024 FU-F6-KANI-CLEANUP resolved; verification_lock unchanged; LMR-001

**Check:** This AC's RED state is subsumed by AC-161-003 (proof_file_hash is null) and
AC-161-004 (kani_version absent). The `v2.5` modified-log entry does not yet exist
(VP-024 is still at v2.4). `verification_lock: true` is correctly set and must remain
unchanged throughout implementation.

**Observed:** VP-024 version is "2.4"; no `v2.5` entry in modified log; `proof_file_hash`
is null. All three components of AC-161-005 are unmet.

**Status: RED (expected)**

---

### AC-161-006 — CLAUDE.md "Two hash disciplines" note added

**Check:**
```bash
grep -n "Two Hash Disciplines\|Two hash disciplines\|proof_file_hash" \
  CLAUDE.md
```

**Expected (GREEN state):** A heading "Two Hash Disciplines" (or closely matching) with
prose distinguishing `input-hash` (MD5-first-7) from `proof_file_hash` (SHA-256
mini-Merkle).

**Observed (RED state):** No "Two Hash Disciplines" or "Two hash disciplines" heading
exists in CLAUDE.md. The grep for `proof_file_hash` returns zero results in CLAUDE.md.
The existing "Input Hash Computation" section covers only `input-hash` / `bin/compute-input-hash`;
the new subsection distinguishing the two hash disciplines has not been added.

**Status: RED (expected)**

---

### AC-161-007 — PR type (deferred; not a file-state check)

**Note:** This AC governs the PR title prefix (`docs:`). It cannot be verified at Red
Gate time (no PR exists yet). It will be verified at PR creation time. Not recorded as
RED or GREEN here; no file state to check.

---

## Precondition Findings

### P-001 — Commit 6e9f2cc: exists but NOT an ancestor of origin/develop

**Verification commands:**
```bash
git cat-file -t 6e9f2cc          # → "commit" (exists)
git merge-base --is-ancestor 6e9f2cc origin/develop  # → exit 1 (NOT_ANCESTOR)
```

**Finding:** Commit `6e9f2cc` (PR #250 merge, 2026-06-16, "develop HEAD at F6 PR #250
merge") EXISTS in the local object store but is **NOT** a reachable ancestor of
`origin/develop`. This is consistent with develop's squash-linear history: the F6 PR
merge commit is accessible as an object but sits off the squash-linearized develop
spine.

**Impact (per story EC-002):** The `kani_proofs` blocks in `src/analyzer/arp.rs` and
`src/decoder.rs` have drifted since that commit (~932 changed lines in arp.rs noted in
story context). Per AC-161-003 and EC-002, the `proof_file_hash` **MUST be computed from
the `6e9f2cc` snapshot, not from current HEAD**. The implementer must use:

```bash
git show 6e9f2cc:src/analyzer/arp.rs   # fileA section extract
git show 6e9f2cc:src/decoder.rs        # fileB section extract
```

The v2.5 modified-log entry must document: "proof_file_hash computed with arp.rs=fileA,
decoder.rs=fileB per module: field order; extracted from commit 6e9f2cc (develop HEAD
at F6 PR #250 merge, 2026-06-16)."

**Status: PRECONDITION RECORDED — implementer must extract from 6e9f2cc snapshot**

---

## Baseline Cargo Check

**Command:** `cargo check` in `.worktrees/STORY-161`

**Result:** PASS — `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 6.68s`

No Rust surface changes are introduced by this story. The cargo check baseline is green
and must remain green after all governance amendments are applied (no Rust files touched).

**Status: GREEN (expected baseline)**

---

## Stubs Created

None. This is a governance-only story (E-11 convention). No Rust stubs are created.
The story modifies only:
- `.factory/specs/verification-properties/VP-INDEX.md` (factory-artifacts branch)
- `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` (factory-artifacts branch)
- `CLAUDE.md` (develop branch)

---

## Regression Check

| Existing Tests | Status |
|----------------|--------|
| All project tests (cargo check baseline) | cargo check passes — no Rust surface touched |

No pre-existing tests are at risk. This story introduces zero Rust source changes.

---

## Hand-Off to Implementer

**Stories ready for implementation:** STORY-161

**Implementation guidance:**

1. **Factory-artifacts branch work (VP-INDEX + VP-024):** All `.factory/` amendments
   land on the `factory-artifacts` branch, not on `develop`. The develop-side diff
   for the PR touches only `CLAUDE.md`.

2. **proof_file_hash computation — MANDATORY FROM 6e9f2cc SNAPSHOT:**
   Extract `#[cfg(kani)] mod kani_proofs { ... }` blocks using:
   ```bash
   git show 6e9f2cc:src/analyzer/arp.rs    # fileA (listed first in VP-024 module: field)
   git show 6e9f2cc:src/decoder.rs         # fileB
   ```
   LF-normalize both sections. Compute:
   ```
   sha256_A = SHA-256(normalized arp.rs section)
   sha256_B = SHA-256(normalized decoder.rs section)
   final_hash = SHA-256(sha256_A_raw_bytes || sha256_B_raw_bytes)
   ```
   Dual-tool verification required (bash sha256sum pipeline AND Python hashlib). Both
   command lines archived in the VP-024 v2.5 modified-log entry. LMR-001 forbids later
   correction — first write is permanent.

3. **kani_version — historical recovery:** Attempt to recover Kani version at commit
   `6e9f2cc` from CI logs, `Cargo.lock` at that commit, and toolchain records. If
   unrecoverable, use honest-unknown fallback:
   `"unknown (pre-LMR verification, 2026-06-16)"`. Recording the current GitHub
   release is FORBIDDEN per LMR-002.

4. **VP-INDEX version bump:** 2.38 → 2.39. Add "Multi-File Proof Anchor Algorithm"
   section with mini-Merkle formula, Section-Scoping Rule, LF normalization rule,
   and detection coverage note. Cross-link with "VP Lock Mutation Rules" section
   (EC-005 requirement).

5. **VP-024 version bump:** 2.4 → 2.5. Populate proof_file_hash, add kani_version
   field, add v2.5 modified-log entry. Do NOT touch verification_lock, status,
   proof_completed_date, or any proof content.

6. **CLAUDE.md note:** Add "Two Hash Disciplines" subsection after the existing
   "Input Hash Computation" section distinguishing input-hash (MD5-first-7) from
   proof_file_hash (SHA-256 mini-Merkle). Commit to develop.

7. **PR title:** `docs: codify multi-file proof_file_hash algorithm + VP-024 re-lock`
   (docs: prefix — develop-side diff is CLAUDE.md only).

---

## Red Gate Summary

| AC | Description | State |
|----|-------------|-------|
| AC-161-001 | VP-INDEX gains "Multi-File Proof Anchor Algorithm" section | RED |
| AC-161-002 | VP-INDEX gains Section-Scoping Rule | RED |
| AC-161-003 | VP-024 proof_file_hash populated (currently null + FU-F6-KANI-CLEANUP) | RED |
| AC-161-004 | VP-024 gains kani_version frontmatter field (currently absent) | RED |
| AC-161-005 | VP-024 FU-F6-KANI-CLEANUP resolved; v2.5 modified-log entry; version 2.4→2.5 | RED |
| AC-161-006 | CLAUDE.md gains "Two Hash Disciplines" note (currently absent) | RED |
| AC-161-007 | PR uses docs: prefix (deferred; no PR yet) | DEFERRED |
| Precondition P-001 | 6e9f2cc exists but NOT ancestor of origin/develop; compute from snapshot | RECORDED |
| Cargo check baseline | cargo check passes on worktree | GREEN |

**Red Gate verdict: ESTABLISHED.** All verifiable ACs are confirmed RED. Implementation
may proceed. No AC passes prematurely. Cargo check baseline is green and must remain
green throughout (no Rust files touched).
