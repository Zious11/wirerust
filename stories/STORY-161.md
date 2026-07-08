---
document_type: story
story_id: STORY-161
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-08T00:00:00Z
phase: f7
level: feature
cycle: triage-2026-07-08
points: 3
priority: P3
depends_on: []
blocks: []
# Governance-only story — no BCs authored (E-11 convention; this story modifies VP governance docs, not Rust source)
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: .factory/specs/verification-properties/
subsystems: []
estimated_days: 1
wave: "72"
traces_to:
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/maintenance/issue-backlog-triage-2026-07-08.md
input-hash: "92569e0"
inputs:
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/maintenance/issue-backlog-triage-2026-07-08.md
---

# STORY-161: Codify Multi-File proof_file_hash Algorithm and Re-lock VP-024

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 72
**Points:** 3
**Priority:** P3

## Narrative

- **As a** spec-steward or future maintainer populating or verifying a Kani VP proof anchor
- **I want** the exact multi-file `proof_file_hash` algorithm to be codified in VP-INDEX prose,
  the highest-risk ambiguity (section-scoping) resolved explicitly, and VP-024's deferred
  `proof_file_hash: null` field populated per that algorithm
- **So that** the proof anchor can be independently verified and the `FU-F6-KANI-CLEANUP`
  obligation is discharged without ambiguity

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

GitHub issue #252 (`VP-024 multi-file proof_file_hash`) was validated and triaged on 2026-07-08
(triage record `triage-2026-07-08`, research verdicts 10/10 CONFIRMED).

VP-024 (`vp-024-arp-parse-safety.md`) was locked/verified at Phase F6 on 2026-06-16 with
`verification_lock: true`. At that time, the `proof_file_hash: null` field was left unpopulated
with the comment "No canonical recomputation method defined for VP-024 proof files. Follow-up:
define hash method and populate. Tracked as FU-F6-KANI-CLEANUP." (VP-024 v2.1 modified log.)

VP-024 spans two proof files because its five Kani harnesses live in two modules:
- `src/decoder.rs` — the `#[cfg(kani)] mod kani_proofs { ... }` block (Sub-A harnesses:
  `verify_extract_arp_frame_safety`, `verify_extract_arp_frame_eth_ipv4_correctness`,
  `verify_extract_arp_frame_none_on_bad_size`)
- `src/analyzer/arp.rs` — the `#[cfg(kani)] mod kani_proofs { ... }` block (Sub-B harness:
  `verify_classify_garp_total`; Sub-D harness: `verify_binding_table_cap`)

The research-validated design (triage record entry #252) specifies:
- **Mini-Merkle construction:** `sha256( sha256(fileA_section_bytes) || sha256(fileB_section_bytes) )`
  in declared file order. This detects cross-file proof-function moves that raw concatenation
  would miss (e.g., moving a harness from decoder.rs to arp.rs would flip the order and produce
  a different hash even if byte content were identical).
- **Section-scoping:** the unit of hashing per file is the entire `#[cfg(kani)] mod kani_proofs
  { ... }` block, including the `#[cfg(kani)]` attribute line and the closing brace. This is the
  highest-risk ambiguity — if section boundaries are vague, independent recomputation diverges.
- **LF normalization:** each extracted section must be LF-normalized (`\r\n` → `\n`, lone `\r`
  → `\n`) before hashing, for OS-independent results.
- **Two hash disciplines:** `input-hash` uses MD5-first-7 (advisory drift detection, per the
  canonical `bin/compute-input-hash` tool); `proof_file_hash` uses SHA-256 mini-Merkle
  (integrity anchor, not advisory). These disciplines are deliberately distinct and must not
  be conflated.
- **kani_version field:** VP-024 gains a `kani_version:` sibling frontmatter field pinned to
  the Kani release current at re-lock time, verified from
  `github.com/model-checking/kani/releases`.

### What this story does NOT do

- No Kani harnesses are re-run. `verification_lock: true` is already set and remains true.
- No Rust source files are modified.
- No new VPs are created.
- No BCs are authored or amended.

The story discharges `FU-F6-KANI-CLEANUP` solely by populating the governance fields.

## Acceptance Criteria

### AC-161-001 (Mini-Merkle algorithm codified in VP-INDEX)

VP-INDEX (`.factory/specs/verification-properties/VP-INDEX.md`) gains a prose section titled
"Multi-File Proof Anchor Algorithm" (or an equivalent heading that makes it locatable via
`grep -n "proof.*anchor\|proof_file_hash" VP-INDEX.md`). The section defines:

1. The mini-Merkle construction verbatim:
   `sha256( sha256(LF-normalized section bytes of fileA) || sha256(LF-normalized section bytes of fileB) )`
   where `||` denotes byte concatenation of the raw (non-hex) SHA-256 digest bytes, and files
   are processed in the order they appear in the VP frontmatter `module:` field.

2. The LF normalization rule: `\r\n` → `\n`; lone `\r` → `\n`.

3. A note on detection coverage: this construction detects (a) harness content changes in
   either file, (b) cross-file harness moves (the section hashes swap position), and (c)
   additions or deletions of entire harness blocks.

VP-INDEX version is bumped from `"2.35"` to `"2.36"` and the `modified:` field updated.

### AC-161-002 (Section-scoping rule explicit in VP-INDEX)

The same VP-INDEX prose section (AC-161-001) states explicitly:

> The section boundary for hashing is the entire `#[cfg(kani)] mod kani_proofs { ... }` block,
> starting from the `#[cfg(kani)]` attribute line (inclusive) and ending at the closing brace
> of the module (inclusive). Whitespace and comments inside the block are included verbatim.

This rule must be stated as a standalone named rule (e.g., **Section-Scoping Rule**) so that
an independent implementer can unambiguously identify the byte range to hash without reading the
VP body. The closing brace inclusion rule is the most common source of off-by-one errors and
must be confirmed explicitly.

Verification:

```bash
grep -A3 "Section-Scoping Rule\|section.scoping\|closing brace" \
  .factory/specs/verification-properties/VP-INDEX.md | head -20
```

must emit the rule text (non-empty output).

### AC-161-003 (VP-024 proof_file_hash populated)

VP-024 (`vp-024-arp-parse-safety.md`) `proof_file_hash:` field is populated with the computed
mini-Merkle hash. The comment `# No canonical recomputation method defined...` and the
`null` value are both replaced. The implementer computes the hash by:

1. Extracting the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/decoder.rs`
   (byte range: from the `#[cfg(kani)]` attribute line to the closing `}` of `mod kani_proofs`).
2. Extracting the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/analyzer/arp.rs`
   (same boundary rule).
3. LF-normalizing both extracted byte strings.
4. Computing `sha256_A = SHA-256(normalized_decoder_section)`.
5. Computing `sha256_B = SHA-256(normalized_arp_section)`.
6. Computing `final_hash = SHA-256(sha256_A_raw_bytes || sha256_B_raw_bytes)`.
7. Storing the hex representation of `final_hash` (lowercase hex, full 64 chars) in the
   `proof_file_hash:` field.

After population, an independent recomputation from the same source files at the same commit
(`verified_at_commit: "6e9f2cc"`) produces the identical hash.

> **Note for implementer:** Verify that `6e9f2cc` is still on the `develop` branch history
> before computing from HEAD. If the kani_proofs blocks have been amended since that commit,
> compute from the `6e9f2cc` snapshot, not from current HEAD.

### AC-161-004 (VP-024 gains kani_version frontmatter field)

VP-024 frontmatter gains a `kani_version:` field as a sibling of `proof_file_hash:` and
`verified_at_commit:`, pinned to the Kani release current at re-lock time. The implementer
MUST verify the current stable Kani release from `github.com/model-checking/kani/releases`
at implementation time and record the exact version string (e.g., `"0.65.0"` or whatever is
current). The triage record noted "latest ~0.65.0" as an approximation; the story AC requires
the exact verified value.

The field form is:
```yaml
kani_version: "X.Y.Z"  # verified at re-lock 2026-07-08 from github.com/model-checking/kani/releases
```

### AC-161-005 (VP-024 FU-F6-KANI-CLEANUP resolved; verification_lock unchanged)

The `FU-F6-KANI-CLEANUP` marker is resolved:
1. The `proof_file_hash: null  # No canonical recomputation method defined...` line is replaced
   with the populated `proof_file_hash: "<64-char-hex>"` (no trailing comment).
2. VP-024 `modified:` log gains a `v2.5` entry documenting: proof_file_hash populated per
   mini-Merkle algorithm codified in VP-INDEX v2.36; kani_version field added; FU-F6-KANI-CLEANUP
   resolved; verification_lock remains true; no proof content changed.
3. VP-024 version is bumped from `"2.4"` to `"2.5"`.
4. `verification_lock: true` remains unchanged — this story does NOT re-run harnesses and does
   NOT modify any proof content (harness code, postconditions, or property statements).

### AC-161-006 (CLAUDE.md "Two hash disciplines" note added)

`CLAUDE.md` gains a new paragraph under the **Input Hash Computation** section (or as a new
standalone subsection immediately after it) titled "Two Hash Disciplines" (or a closely matching
heading). The note states:

> **Two hash disciplines in this repository are deliberately distinct:**
>
> - `input-hash` (story frontmatter): MD5-first-7 hex, computed by `bin/compute-input-hash`
>   (canonical Python tool). Purpose: advisory drift detection for spec inputs. Lightweight,
>   not a security primitive.
> - `proof_file_hash` (VP frontmatter): SHA-256 mini-Merkle over Kani proof sections,
>   full 64-char hex. Purpose: integrity anchor for formal verification artifacts. Tamper-evident.
>
> Do not conflate the two. `input-hash` and `proof_file_hash` use different algorithms,
> different truncations, and serve different roles. Changing an `input-hash` has no effect
> on `proof_file_hash` and vice versa.

The exact wording may vary; the note must be present and clearly distinguish the two hash uses.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Multi-file algorithm prose | `.factory/specs/verification-properties/VP-INDEX.md` (amend) | Documentation |
| Section-scoping rule | `.factory/specs/verification-properties/VP-INDEX.md` (amend) | Documentation |
| proof_file_hash population | `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` (amend) | Documentation |
| kani_version field | `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` (amend) | Documentation |
| FU-F6-KANI-CLEANUP resolved | `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` (amend) | Documentation |
| Two hash disciplines note | `CLAUDE.md` (amend) | Documentation |

No Rust source files, no tests, no CI configuration.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `VP-INDEX.md` | Documentation artifact | Governance prose; no code |
| `vp-024-arp-parse-safety.md` | Documentation artifact | VP frontmatter and lifecycle log; append-only per VSDD L4 rules (proof content unchanged) |
| `CLAUDE.md` | Documentation artifact | Project guidance file |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | VP-024 `verification_lock: true` during proof_file_hash population | Lock is NOT cleared; this is a governance amendment, not a re-proof. Append-only rule for L4 docs applies |
| EC-002 | kani_proofs block in decoder.rs has additional harnesses added since `6e9f2cc` | Compute from the `6e9f2cc` snapshot, not from current HEAD. Document the commit used in the v2.5 modified log |
| EC-003 | Trailing whitespace or CRLF in extracted kani_proofs block on Windows checkout | LF-normalize before hashing; OS-independent result required |
| EC-004 | Order of files in mini-Merkle construction | decoder.rs is fileA, arp.rs is fileB, matching VP-024 `module:` field declaration order: `src/analyzer/arp.rs + src/decoder.rs`. However the module: field lists arp.rs first — implementer MUST note the actual hash computation order used and document it in the v2.5 entry |
| EC-005 | VP-INDEX section placement | The algorithm section should appear before the catalog table (near the Summary) so it is readable without scrolling to the end of the file |

> **EC-004 implementer note:** The VP-024 `module:` field reads `src/analyzer/arp.rs + src/decoder.rs`
> (arp.rs listed first). The algorithm specifies "files in declared order." The implementer must
> decide whether "declared order" means the module: field order (arp.rs → decoder.rs) or the
> logical dependency order (decoder.rs provides Sub-A, arp.rs provides Sub-B/D). The implementer
> MUST document the chosen order explicitly in the VP-024 v2.5 modified log so the hash can be
> independently reproduced.

## Tasks

1. **Read VP-024 and VP-INDEX in their entirety.** Understand the existing proof structure,
   the `FU-F6-KANI-CLEANUP` comment, and the VP-INDEX catalog row for VP-024.

2. **Verify current Kani version.** Fetch `github.com/model-checking/kani/releases` to find
   the latest stable Kani release tag. Record the exact version string.

3. **Add algorithm prose to VP-INDEX.** Under a new "Multi-File Proof Anchor Algorithm" section
   (near the Summary, before the catalog table), add:
   - The mini-Merkle construction formula (AC-161-001)
   - The Section-Scoping Rule (AC-161-002)
   - LF normalization rule
   - Detection coverage note
   Bump VP-INDEX version to `"2.36"` and update the `modified:` field.

4. **Compute proof_file_hash.** From the `6e9f2cc` checkout:
   a. Extract the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/decoder.rs`.
   b. Extract the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/analyzer/arp.rs`.
   c. LF-normalize both.
   d. Compute sha256_A and sha256_B of the normalized sections.
   e. Compute `final_hash = sha256(sha256_A_bytes || sha256_B_bytes)`.
   f. Record the file order used (EC-004).

5. **Populate VP-024 proof_file_hash.** Replace the `proof_file_hash: null  # ...` line with
   `proof_file_hash: "<final_hash>"`. Add the `kani_version:` field. Add v2.5 modified log entry.
   Bump version to `"2.5"`. Do NOT change `verification_lock`, `status`, `proof_completed_date`,
   or any proof content.

6. **Add CLAUDE.md note.** Add the "Two hash disciplines" paragraph (AC-161-006) under or after
   the Input Hash Computation section.

7. **Verify no product code changes.** Run `git -C /path/to/develop diff HEAD` and confirm zero
   changes to `src/`, `tests/`, or `.github/`. The diff must touch only `.factory/` files and
   `CLAUDE.md`.

## Previous Story Intelligence

Lessons from analogous governance stories:

- **STORY-157 (wave-71, E-11, 5 pts):** Codified four wave-70 process gaps including
  `PG-HASH-EMPTY-INPUTS` (input-hash empty-inputs handling) and `PG-HASH-HOOK-DIVERGENCE`.
  Pattern: amend governance doc → update CLAUDE.md → commit to factory-artifacts branch.
  This story follows the same no-product-code pattern.
- **STORY-158 (wave-TBD, E-11, 3 pts):** Wave-71 process-gap codifications. Governance-only.
  Source for the "append to CLAUDE.md under existing section" pattern.
- **VP-024 v2.0 modified log (F6 LOCK):** The `proof_file_hash` and `verified_at_commit` fields
  were explicitly deferred at F6 lock time: "left null pending develop HEAD after F6 PR merges —
  do not populate from speculative values." The `verified_at_commit: "6e9f2cc"` field WAS
  populated in v2.1. The `proof_file_hash` field was NOT populated (FU-F6-KANI-CLEANUP).
  This story completes the deferred obligation.

## Architecture Compliance Rules

- This story modifies ONLY: `VP-INDEX.md`, `vp-024-arp-parse-safety.md`, and `CLAUDE.md`.
  No Rust source files, no test files, no CI, no Cargo.toml.
- VP-024 is a locked (verified) L4 document. The append-only rule applies. This story makes
  only governance amendments (frontmatter field population, modified log append) — it does NOT
  modify proof content (harness code, postconditions, property statements, or the BC anchor
  table).
- The `verification_lock: true` flag must NOT be cleared or modified.
- The `FU-F6-KANI-CLEANUP` follow-up is resolved by populating the hash field, not by removing
  it or changing its governance intent.

## Library & Framework Requirements

None beyond standard command-line SHA-256 utilities (`sha256sum`, Python's `hashlib.sha256`,
or equivalent). No Rust toolchain changes.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/specs/verification-properties/VP-INDEX.md` | Modify | Add "Multi-File Proof Anchor Algorithm" prose section; bump version 2.35→2.36 |
| `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` | Modify | Populate proof_file_hash; add kani_version; bump version 2.4→2.5; add v2.5 modified log entry |
| `CLAUDE.md` | Modify | Add "Two hash disciplines" note after Input Hash Computation section |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4 k |
| VP-INDEX algorithm section (~30 lines) | ~0.5 k |
| VP-024 frontmatter + modified log amendment | ~0.3 k |
| `CLAUDE.md` paragraph (~10 lines) | ~0.1 k |
| **Total** | **~4.9 k** |

Well within context window. No story split required.

## Notes

- **Provenance:** GitHub issue #252. Validated by research agent (triage-2026-07-08, 10/10
  CONFIRMED). Story drafted in wave-72 planning burst.
- **FU-F6-KANI-CLEANUP is the originating obligation.** VP-024 v2.1 modified log records this
  as a follow-up at the time of F6 lock (2026-06-16). The triage-2026-07-08 session determined
  the algorithm and unblocked the story.
- **EC-004 file order must be resolved by the implementer.** The story cannot pre-determine the
  canonical order without inspecting the VP-024 module: field declaration. The implementer must
  choose and document the order in the v2.5 modified log.
- **kani_version verification is mandatory.** The triage record says "~0.65.0" as an estimate.
  The AC requires the exact version from the GitHub releases page. Do not guess.
- **CLAUDE.md note is in scope for a governance-only story.** CLAUDE.md is a project guidance
  file, not a product file. The "Two hash disciplines" note is consistent with prior STORY-157
  and STORY-158 CLAUDE.md amendments.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-08 | story-writer | Initial authorship — triage-2026-07-08 #252 follow-up: codify multi-file proof_file_hash mini-Merkle algorithm in VP-INDEX; populate VP-024 proof_file_hash + kani_version; resolve FU-F6-KANI-CLEANUP; add CLAUDE.md two-hash-disciplines note; wave-72 draft. |
