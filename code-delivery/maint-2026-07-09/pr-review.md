# PR #393 Fresh-Eyes Review

**PR:** [Zious11/wirerust#393](https://github.com/Zious11/wirerust/pull/393)
**Title:** docs: maint-2026-07-09 Route A documentation sweep
**Base:** develop  <-  **Head:** docs/maint-2026-07-09-doc-sweep
**Reviewer:** pr-reviewer (fresh-eyes, model: Opus 4.7)
**Verdict:** APPROVE_WITH_SUGGESTIONS

## Scope Verified

Diff exercises the four files declared in the PR description:

| File | Net | Item(s) |
|------|-----|---------|
| `README.md` | +37 / -1 | DD-001 (`--coverage-gaps`), DD-002 (DNP3 counters), REC-006 (MACsec-Modified ARP limitation) |
| `docs/adr/0007-dnp3-stream-dispatch-and-parser-design.md` | +11 | REC-005 (Crain/Sistrunk caveat on Decision 3) |
| `docs/adr/0008-withdrawn-placeholder.md` | +23 (new) | DD-004 (withdrawn placeholder) |
| `deny.toml` | 0 / -8 | DEP-006 (prune 8 unused license allowlist entries) |

Removed license entries (counted independently): `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`,
`ISC`, `Unicode-DFS-2016`, `Zlib`, `CC0-1.0`, `MPL-2.0`, `0BSD` — 8 entries, matches
PR description.

No changes to `src/`, `Cargo.toml`, `bin/`, or `.github/workflows/`; `changelog-gate` and
`action-pin-gate` correctly not triggered.

## Fresh-Eyes Assessment

### 1. Accuracy of factual claims
- Tri-state coverage vocabulary (`known-supported` / `known-unsupported` / `unknown`)
  and ADR-012 Decision 8 reference are internally consistent with wave-72 ADR-012 work
  recorded in the recent commit log.
- DNP3 counter constants (`MAX_FINDINGS = 10 000`, `MAX_MASTER_ADDRS = 64`,
  `MAX_PENDING_REQUESTS = 256`) and the effect claim (evicted pending requests can
  suppress a T1691.001 block-command finding) are internally coherent with the T1691.001
  attribution already listed in the DNP3 line above.
- 802.1Q / 802.1ad / 802.1AE mappings are correct (Q-VLAN / QinQ / MACsec).
- CWE-693 (Protection Mechanism Failure) is an appropriate CWE anchor for the
  MACsec-Modified boundary.
- Crain/Sistrunk (2014) is a real, well-known DNP3 interop/security research anchor;
  the described symptom (CRC-omitted DNP3-over-TCP frames breaking `compute_dnp3_frame_len`
  boundary math) is a plausible and specific interop hazard, not a hand-wave.
- ADR-0008 placeholder claim "ADRs 0001-0007 and 0009-0012 remain in force" matches the
  ADR list in CLAUDE.md (Project References table).

No factual defects found.

### 2. Completeness
All six gate items are present in the diff at the granularity the PR description promises.

### 3. Internal consistency
- New DNP3 counter block is placed under the DNP3 CLI-flags stanza, immediately after the
  `--dnp3-direct-operate-threshold` entry — natural reading order.
- MACsec-Modified ARP limitation is inserted before `## Roadmap`, adjacent to the existing
  DNP3 threshold-context paragraph. Placement is consistent with prior "Known
  Limitations"-style entries.
- ADR-0007 caveat is inserted after the Decision 3 CRC-omission paragraph — the correct
  anchor location for a CRC-related caveat.
- ADR-0008 self-references (`ADR-0007`, `ADR-0009`, `ADR-012`) all align with the
  project's actual ADR set.

### 4. Internal references / paths that shouldn't be public
The new prose cites internal identifiers: `D-078`, `BC-2.16.015 / BC-2.16.009`,
`STORY-116 / STORY-117`, `EC-007 / EC-009(c)`, `E-17`, `ASM-CAND-008`, `ASM-CAND-006`.
These are consistent with the project's already-established public-doc citation style
(ADR-0007 already cites `RULING-DNP3-SIBLING-001 §1.5` in its shipped text). Not a leak.

No `.factory/` file paths, secrets, or credentials appear in the diff. `doc-drift-findings.md`
and `doc-drift.md` are referenced by name inside ADR-0008 as historical anchors — those
filenames are internal but do not resolve to accessible paths in the public tree, so they
read as forensic breadcrumbs rather than exposure.

### 5. Prose quality and clarity
Overall clean, direct, technically dense in the style of the rest of the README. Adversarial
convergence appears to have already removed the two prose-quality snags flagged in earlier
passes (fabricated ADR-0008 history, ADR-0008 timeline phrasing).

## Findings

### Non-blocking suggestions

| ID | Severity | Category | File | Finding | Suggestion |
|----|----------|----------|------|---------|------------|
| S1 | NITPICK | prose | `README.md` (DNP3 counters section) | Constants typeset with a space thousands separator: `MAX_FINDINGS = 10 000`. A hurried reader can parse this as two tokens; it also does not match the Rust literal spelling. | Use `10_000` (Rust literal) or `10,000` (English). `64` and `256` are fine as-is. |
| S2 | NITPICK | prose | `README.md` (MACsec limitation) | The hyphenated compound "documented-unverified" is unusual and slightly forced. | "Documented but unverified" reads more naturally without loss of meaning. |
| S3 | NITPICK | prose | `README.md` (DNP3 counters section) | ``at `analyzers[i].detail` `` uses informal `[i]` array-index notation. | Consider `analyzers[].detail` or `analyzers[*].detail` (jq-style), or spell out "in the `detail` object of each entry in the `analyzers` array". |
| S4 | NITPICK | dependency-hygiene | `deny.toml` | Pruning `ISC` and `BSD-2-Clause` from the allowlist is defensible under a "current transitive graph only" rule, but these are common Rust-ecosystem licenses. The next unrelated dep addition that pulls in either will fail `cargo deny`, forcing a re-add in a downstream PR. | No action requested — flagged so the team is not surprised when it happens. |
| S5 | NITPICK | prose | `docs/adr/0008-withdrawn-placeholder.md` | "ADR-0008 was reserved at some point prior to ADR-0009" is deliberately vague. Prior convergence pass explicitly accepted the ambiguity, and this is honest phrasing. | If a concrete anchor exists (commit/PR that introduced ADR-0009 skipping 0008), citing it would remove the residual "when exactly" ambiguity. Acceptable as-is. |

### Blocking findings

None.

## Adversarial Convergence Evidence Reviewed

PR body reports 6 fresh-context passes (P1-P3 fixed HIGH/LOW/NITPICK, P4-P6 consecutive
CLEAN on HEAD bcb3593). The specific historical findings reported (F-RA-P1-001 tri-state
vocabulary, F-RA-P1-002 JSON key name, F-RA-P2-001 fabricated ADR-0008 audit history,
F-RA-P2-002 dnp3_summary field location, F-RA-P3-001 inverted VLAN/QinQ/MACsec claim,
N-RA-P3-N1 ADR-0008 timeline) all match plausible mistake classes for this specific diff,
and I confirm the current shipping text does not exhibit any of them.

## Verdict Rationale

The diff is coherent with the PR description, internally consistent, and free of factual,
completeness, or hygiene defects that I can detect from the diff alone. The five NITPICKs
above are style-only and do not block merge.

**APPROVE_WITH_SUGGESTIONS** — safe to merge; suggestions may be addressed in a follow-up
docs sweep if desired.
