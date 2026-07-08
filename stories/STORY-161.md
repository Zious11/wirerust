---
document_type: story
story_id: STORY-161
epic_id: E-11
version: "1.3"
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
input-hash: "dddd9fd"
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
- **kani_version field:** VP-024 gains a `kani_version:` sibling frontmatter field recording
  the HISTORICAL Kani version that performed the verification at
  `verified_at_commit: "6e9f2cc"`. The implementer MUST attempt historical recovery (CI logs,
  Cargo.lock, toolchain records at commit `6e9f2cc`). Honest-unknown fallback:
  `"unknown (pre-LMR verification, <proof_completed_date>)"` if unrecoverable. Recording the
  currently-available GitHub release is FORBIDDEN. Full cargo-kani re-run is the
  always-preferred alternative (per **LMR-002**; document in modified log).

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
   are processed in the order they appear in the VP frontmatter `module:` field. The `module:`
   field order is normative. For VP-024 specifically: arp.rs is fileA (listed first in `module:`)
   and decoder.rs is fileB.

2. The LF normalization rule: `\r\n` → `\n`; lone `\r` → `\n`.

3. A note on detection coverage: this construction detects (a) harness content changes in
   either file, (b) cross-file harness moves (the section hashes swap position), and (c)
   additions or deletions of entire harness blocks.

VP-INDEX version is bumped from `"2.38"` to `"2.39"` and the `modified:` field updated.

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

The `module:` field order is normative: arp.rs is fileA (listed first in VP-024's `module:`
field: `src/analyzer/arp.rs + src/decoder.rs`), decoder.rs is fileB.

1. Extracting the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/analyzer/arp.rs`
   (byte range: from the `#[cfg(kani)]` attribute line to the closing `}` of `mod kani_proofs`).
   This is fileA — arp.rs is listed first in VP-024's `module:` field.
2. Extracting the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/decoder.rs`
   (same boundary rule). This is fileB.
3. LF-normalizing both extracted byte strings.
4. Computing `sha256_A = SHA-256(normalized_arp_section)`.
5. Computing `sha256_B = SHA-256(normalized_decoder_section)`.
6. Computing `final_hash = SHA-256(sha256_A_raw_bytes || sha256_B_raw_bytes)`.
7. Storing the hex representation of `final_hash` (lowercase hex, full 64 chars) in the
   `proof_file_hash:` field.

After population, an independent recomputation from the same source files at the same commit
(`verified_at_commit: "6e9f2cc"`) produces the identical hash.

> **Note for implementer:** Verify that `6e9f2cc` is still on the `develop` branch history
> before computing from HEAD. If the kani_proofs blocks have been amended since that commit,
> compute from the `6e9f2cc` snapshot, not from current HEAD.

### AC-161-004 (VP-024 gains kani_version frontmatter field — per LMR-003 + LMR-002)

The `kani_version:` field did not exist at VP-024 lock time. Adding it is a new-field append
governed by **LMR-003 (Locked-Doc-Appendable Provenance Field Allowlist)** — `kani_version:`
is the only field currently on the allowlist (no hash/checksum/digest values; sibling placement
required; see AC-161-005 for modified-log requirements).

Per **LMR-002 (Historical Kani Version Record)**, `kani_version:` records the HISTORICAL
Kani version that performed the verification at `verified_at_commit: "6e9f2cc"` — not the
version available at population time.

The implementer MUST attempt historical recovery of the Kani version at commit `6e9f2cc`:
check CI logs from the verification run, `Cargo.lock` at that commit, and any toolchain
records present at `6e9f2cc`. If the historical version is unrecoverable, the honest-unknown
fallback MUST be used:

```yaml
kani_version: "unknown (pre-LMR verification, <proof_completed_date>)"
```

**Recording the currently-available GitHub release is FORBIDDEN** — it would misrepresent
which version actually performed the proof.

**Full cargo-kani re-run is the always-preferred alternative**: if a re-run is feasible,
run `cargo kani` against the harnesses and record the exact version used; document the
re-run in the VP-024 v2.5 modified log.

VP-024 frontmatter gains the `kani_version:` field as a sibling of `proof_file_hash:` and
`verified_at_commit:`. The field form after historical recovery or re-run is:

```yaml
kani_version: "X.Y.Z"  # historical version at verified_at_commit 6e9f2cc; recovered from <source>
```

Or, if using the honest-unknown fallback:

```yaml
kani_version: "unknown (pre-LMR verification, <proof_completed_date>)"
```

### AC-161-005 (VP-024 FU-F6-KANI-CLEANUP resolved; verification_lock unchanged; LMR-001)

The `FU-F6-KANI-CLEANUP` marker is resolved per **LMR-001 (Deferred-Null Anchor
First-Population)**: a `null` sentinel left at lock time is absent-at-lock, not a locked
value. First-population (null → computed hash) requires **NO unlock ceremony**;
`verification_lock: true` stays set throughout. The modified-log entry is required to
record the population event.

1. The `proof_file_hash: null  # No canonical recomputation method defined...` line is replaced
   with the populated `proof_file_hash: "<64-char-hex>"` (no trailing comment).
2. VP-024 `modified:` log gains a `v2.5` entry documenting: proof_file_hash populated per
   mini-Merkle algorithm codified in VP-INDEX v2.39; kani_version field added per LMR-003
   (field absent at lock time; on Locked-Doc-Appendable Provenance Field Allowlist; lock not
   cleared); kani_version value sourced via LMR-002 historical recovery; FU-F6-KANI-CLEANUP
   resolved per LMR-001; verification_lock remains true throughout (no unlock ceremony
   performed); no proof content changed.
3. VP-024 version is bumped from `"2.4"` to `"2.5"`.
4. `verification_lock: true` remains unchanged — this story does NOT re-run harnesses and does
   NOT modify any proof content (harness code, postconditions, or property statements). Per
   LMR-001, null-to-value first-population is a governance amendment, not a re-proof event.

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
| EC-004 | Order of files in mini-Merkle construction | arp.rs is fileA, decoder.rs is fileB — the `module:` field order is normative (`src/analyzer/arp.rs + src/decoder.rs`, arp.rs listed first). No implementer decision required; this order is fully determined by the VP-024 frontmatter |
| EC-005 | VP-INDEX section placement | The algorithm section should appear before the catalog table (near the Summary) so it is readable without scrolling to the end of the file |

## Tasks

1. **Read VP-024 and VP-INDEX in their entirety.** Understand the existing proof structure,
   the `FU-F6-KANI-CLEANUP` comment, and the VP-INDEX catalog row for VP-024.

2. **Recover historical Kani version at commit `6e9f2cc` (per LMR-002).** Check CI logs from
   the verification run at commit `6e9f2cc`, `Cargo.lock` at that commit, and any toolchain
   records present at `6e9f2cc`. If the historical version is unrecoverable, use the
   honest-unknown fallback: `"unknown (pre-LMR verification, <proof_completed_date>)"`. Do
   NOT record the currently-available GitHub release — that is FORBIDDEN per LMR-002. If a
   full cargo-kani re-run is feasible, prefer that path and document the re-run in the
   v2.5 modified log.

3. **Add algorithm prose to VP-INDEX.** Under a new "Multi-File Proof Anchor Algorithm" section
   (near the Summary, before the catalog table), add:
   - The mini-Merkle construction formula (AC-161-001)
   - The Section-Scoping Rule (AC-161-002)
   - LF normalization rule
   - Detection coverage note
   Bump VP-INDEX version to `"2.39"` and update the `modified:` field.

4. **Compute proof_file_hash.** From the `6e9f2cc` checkout (arp.rs=fileA, decoder.rs=fileB
   per the normative module: field order):
   a. Extract the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/analyzer/arp.rs`
      (fileA — listed first in VP-024's `module:` field).
   b. Extract the `#[cfg(kani)] mod kani_proofs { ... }` block from `src/decoder.rs` (fileB).
   c. LF-normalize both.
   d. Compute sha256_A = SHA-256(normalized_arp_section) and sha256_B = SHA-256(normalized_decoder_section).
   e. Compute `final_hash = sha256(sha256_A_bytes || sha256_B_bytes)`.
   f. Record in the v2.5 modified log: "proof_file_hash computed with arp.rs=fileA, decoder.rs=fileB per module: field order."

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
- The `verification_lock: true` flag must NOT be cleared or modified. Per **LMR-001
  (Deferred-Null Anchor First-Population)**, null-to-value first-population is a governance
  amendment event, not a re-proof event. No unlock ceremony is performed; the lock stays true.
- The `kani_version:` field addition is a new-field append governed by **LMR-003 (Locked-Doc-
  Appendable Provenance Field Allowlist)**. Only fields on the allowlist (currently only
  `kani_version:`) may be appended to a locked L4 doc without an unlock ceremony. Conditions:
  no hash/checksum/digest values, sibling placement, modified-log entry citing LMR-003 and
  confirming lock not cleared.
- The `FU-F6-KANI-CLEANUP` follow-up is resolved by populating the hash field, not by removing
  it or changing its governance intent.

## Library & Framework Requirements

None beyond standard command-line SHA-256 utilities (`sha256sum`, Python's `hashlib.sha256`,
or equivalent). No Rust toolchain changes.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/specs/verification-properties/VP-INDEX.md` | Modify | Add "Multi-File Proof Anchor Algorithm" prose section; bump version 2.38→2.39 |
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
- **EC-004 file order is fully determined.** The canonical order is arp.rs = fileA, decoder.rs =
  fileB, per the VP-024 `module:` field (`src/analyzer/arp.rs + src/decoder.rs`, arp.rs listed
  first). The `module:` field order is normative (F-W72-P1-001). No implementer decision required;
  the v2.5 modified log records the algorithm reference, not an order choice.
- **kani_version is the HISTORICAL version at commit `6e9f2cc` (LMR-002).** The triage record
  noted "~0.65.0" as an estimate. The AC requires historical recovery (CI logs, Cargo.lock,
  toolchain records at `6e9f2cc`). Recording the currently-available GitHub release is
  FORBIDDEN per LMR-002. Use the honest-unknown fallback if unrecoverable; prefer a full
  cargo-kani re-run if feasible (document in modified log).
- **CLAUDE.md note is in scope for a governance-only story.** CLAUDE.md is a project guidance
  file, not a product file. The "Two hash disciplines" note is consistent with prior STORY-157
  and STORY-158 CLAUDE.md amendments.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-07-08 | story-writer | Adversary P3 fixes: F-W72-P3-003 (HIGH) — LMR-003 alignment: AC-161-001 bump math corrected 2.37→2.38 to 2.38→2.39 (VP-INDEX now v2.38 per commit 8a4977f). AC-161-004 header updated to cite LMR-003 + LMR-002; LMR-003 governance paragraph added (field absent at lock time; on Locked-Doc-Appendable Provenance Field Allowlist; sibling placement; lock not cleared). AC-161-005 modified-log template updated: VP-INDEX v2.38→v2.39; kani_version entry now cites LMR-003 with required conditions; kani_version value sourced via LMR-002. Architecture Compliance Rules: LMR-003 bullet added alongside LMR-001. Tasks item 3: bump target 2.38→2.39. File Structure Requirements: bump notation 2.37→2.38 corrected to 2.38→2.39. |
| 1.2 | 2026-07-08 | story-writer | Adversary P2 fixes: F-W72-P2-007 (HIGH) — LMR-002 alignment throughout: AC-161-004 rewritten (kani_version records HISTORICAL version at 6e9f2cc; historical recovery required; honest-unknown fallback if unrecoverable; current release FORBIDDEN; re-run always-preferred); Background kani_version bullet updated; Tasks item 2 updated for historical recovery; Notes kani_version note updated. VP-INDEX bump math updated throughout from 2.36→2.37 to 2.37→2.38 (AC-161-001, Tasks item 3, File Structure Requirements, AC-161-005 modified-log template). |
| 1.1 | 2026-07-08 | story-writer | Adversary P1 fixes: F-W72-P1-001 (CRITICAL) — file-order canonicalized: arp.rs=fileA, decoder.rs=fileB per VP-024 module: field; AC-161-001 explicit statement added, AC-161-003 steps 1–6 reordered, EC-004 carve-out removed, EC-004 Notes updated. F-W72-P1-004 — LMR-001 adoption: AC-161-005 cites LMR-001 explicitly, no unlock ceremony language, lock stays true throughout; VP-INDEX required bump updated 2.35→2.36 → 2.36→2.37 (spec-steward commit b0248ba took v2.36); Tasks item 3 + File Structure Requirements updated. F-W72-P1-006 — kani_version comment wording: "re-lock 2026-07-08" → "population <implementation-date>"; Background + Notes "re-lock time" → "population time". Architecture Compliance Rules updated with LMR-001 citation. |
| 1.0 | 2026-07-08 | story-writer | Initial authorship — triage-2026-07-08 #252 follow-up: codify multi-file proof_file_hash mini-Merkle algorithm in VP-INDEX; populate VP-024 proof_file_hash + kani_version; resolve FU-F6-KANI-CLEANUP; add CLAUDE.md two-hash-disciplines note; wave-72 draft. |
