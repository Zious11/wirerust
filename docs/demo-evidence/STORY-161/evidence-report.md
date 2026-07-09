# Demo Evidence Report — STORY-161

**Story:** Codify Multi-File proof_file_hash Algorithm and Re-lock VP-024  
**Story ID:** STORY-161  
**Branch:** docs/STORY-161-two-hash-disciplines  
**HEAD at evidence generation:** efe047a  
**Recorded:** 2026-07-09  
**Tool:** VHS 0.11.0 / Font: Menlo  

---

## Coverage Summary

| AC | Description | Demo Artifact | Status |
|----|-------------|--------------|--------|
| AC-161-001 | Multi-File Proof Anchor Algorithm section in VP-INDEX v2.39 | `AC-001-002-vp-index-algorithm.*` | Recorded |
| AC-161-002 | Section-Scoping Rule text in VP-INDEX (non-empty grep output) | `AC-001-002-vp-index-algorithm.*` | Recorded |
| AC-161-003 | Hash recomputation from 6e9f2cc snapshot matches stored VP-024 value | `AC-003-hash-recomputation.*` | Recorded |
| AC-161-004 | VP-024 `kani_version: "0.67.0"` sibling placement (LMR-002 historical recovery) | `AC-004-005-vp024-frontmatter.*` | Recorded |
| AC-161-005 | VP-024 `verification_lock: true` unchanged; version `"2.5"`; FU-F6-KANI-CLEANUP absent | `AC-004-005-vp024-frontmatter.*` | Recorded |
| AC-161-006 | CLAUDE.md "Two Hash Disciplines" subsection distinguishes MD5-first-7 vs SHA-256 mini-Merkle | `AC-006-claude-md-two-hash-disciplines.*` | Recorded |
| AC-161-007 | PR title uses `docs:` semantic prefix | PR-time — no capture | N/A (PR-time) |

---

## Recorded Demos

### AC-161-001 + AC-161-002 — VP-INDEX Multi-File Proof Anchor Algorithm

**Verification:** `grep -n 'Multi-File Proof Anchor|proof_file_hash' VP-INDEX.md` returns lines
from the new section at version 2.39. `grep -A3 -E 'Section-Scoping Rule|section.scoping|closing brace'`
returns the rule text (non-empty output, satisfying the AC-161-002 verification command verbatim).

| File | Size |
|------|------|
| `AC-001-002-vp-index-algorithm.gif` | 506 KB |
| `AC-001-002-vp-index-algorithm.webm` | 587 KB |
| `AC-001-002-vp-index-algorithm.tape` | VHS script |

**Factory-half note:** VP-INDEX.md lives on the `factory-artifacts` branch. The recording
runs against the checked-out `.factory/` worktree mounted at `<repo>/.factory/`.

---

### AC-161-003 — Hash Recomputation (money shot)

**Verification:** Live mini-Merkle recomputation from commit `6e9f2cc` snapshot:
- `git show 6e9f2cc:src/analyzer/arp.rs` → extract `#[cfg(kani)] mod kani_proofs { ... }` block
- `git show 6e9f2cc:src/decoder.rs` → extract `#[cfg(kani)] mod kani_proofs { ... }` block
- LF-normalize both sections (zero CR bytes in either file; normalization is a no-op)
- `sha256_A = sha256(arp.rs section)` = `c9e6414f92ea6ea50d31dcad870693b65d48b69ea9989dd8807518b2cd493e95`
- `sha256_B = sha256(decoder.rs section)` = `1e7370ae91660caa82a7301a07f32bfe7ee08a46ccb35cff9965d2787d7d4049`
- `final_hash = sha256(sha256_A_raw || sha256_B_raw)` = `48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5`
- **Stored in VP-024 frontmatter:** `48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5`
- **MATCH: True**

Section sizes: arp.rs = 4738 bytes, decoder.rs = 8192 bytes. Zero CR bytes; normalization was no-op.
File order: arp.rs = fileA (listed first in VP-024 `module:` field), decoder.rs = fileB.

| File | Size |
|------|------|
| `AC-003-hash-recomputation.gif` | 214 KB |
| `AC-003-hash-recomputation.webm` | 541 KB |
| `AC-003-hash-recomputation.tape` | VHS script |

---

### AC-161-004 + AC-161-005 — VP-024 Frontmatter Fields

**Verification:** grep shows:
- `kani_version: "0.67.0"` — historical version at `verified_at_commit: "6e9f2cc"`, recovered per LMR-002
- `verification_lock: true` — unchanged throughout (no unlock ceremony performed)
- `version: "2.5"` — bumped from 2.4 per AC-161-005
- `proof_file_hash: "48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5"` — populated (not null)
- `FU-F6-KANI-CLEANUP` — absent from the `proof_file_hash:` line; discharge recorded in v2.5 modified log

| File | Size |
|------|------|
| `AC-004-005-vp024-frontmatter.gif` | 129 KB |
| `AC-004-005-vp024-frontmatter.webm` | 200 KB |
| `AC-004-005-vp024-frontmatter.tape` | VHS script |

---

### AC-161-006 — CLAUDE.md Two Hash Disciplines

**Verification:** grep shows the `### Two Hash Disciplines` subsection immediately after the
`### Known Tool Divergences (PG-HASH-HOOK-DIVERGENCE)` section. The subsection clearly
distinguishes:
- `input-hash`: MD5-first-7 hex, advisory drift detection, `bin/compute-input-hash`
- `proof_file_hash`: SHA-256 mini-Merkle, 64-char hex, integrity anchor, tamper-evident

| File | Size |
|------|------|
| `AC-006-claude-md-two-hash-disciplines.gif` | 129 KB |
| `AC-006-claude-md-two-hash-disciplines.webm` | 159 KB |
| `AC-006-claude-md-two-hash-disciplines.tape` | VHS script |

---

## Scrub Gate (PG-W70-DEMO-SCRUB)

**Command:** PG-W70-DEMO-SCRUB gate (`grep -rE` for absolute host-user paths in `docs/demo-evidence/STORY-161/`)  
**Result:** Zero matches — PASS  
**Run at:** 2026-07-09, before commit

---

## Factory-Half ACs (VP-INDEX + VP-024 amendments)

AC-161-001, AC-161-002, AC-161-003, AC-161-004, AC-161-005 all verify amendments that live
on the `factory-artifacts` branch (VP-INDEX.md and vp-024-arp-parse-safety.md). The recordings
demonstrate the content of those files as checked out via the `.factory/` worktree mount.
The develop-side PR diff contains only `CLAUDE.md` (AC-161-006), consistent with AC-161-007
which specifies `docs:` as the PR prefix.

AC-161-007 (PR title uses `docs:` semantic prefix) is verified at PR-time — not captured.
