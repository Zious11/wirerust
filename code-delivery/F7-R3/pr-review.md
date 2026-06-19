# PR Review — #274 `docs(adr-0003): sync #[non_exhaustive] + broaden help-provenance gate (F7-R3)`

**Verdict: APPROVE**

Independent fresh-eyes review. Scope reviewed: the actual PR diff (`gh pr diff 274`)
against `develop` — 2 files, +54/-14: `.github/workflows/ci.yml` and
`docs/adr/0003-reporting-pipeline-layering.md`. This is genuinely doc + CI only;
no `src/`, `tests/`, or `Cargo.toml` changes.

> Note: a `git diff HEAD~2 HEAD` spans PR #273 as well as #274 and shows `src/`/`tests/`
> changes that are **not** part of this PR. I reviewed the true PR boundary
> (merge-base diff) to avoid attributing #273's code changes to this PR.

## What I Verified (no rubber-stamp)

### 1. ADR Decision code block mirrors shipped `terminal.rs` — VERIFIED
Compared the ADR Decision block against `docs/f7-r3-adr-sync:src/reporter/terminal.rs:118-160`:
- `#[non_exhaustive]` present on `Grouping`, `Collapse`, and `FindingsRender` in both. ✓
- Derive list matches exactly: `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. ✓
- `FindingsRender` has the two `pub` fields `grouping: Grouping`, `collapse: Collapse`. ✓
- `impl FindingsRender { pub fn new(grouping, collapse) -> Self { Self { grouping, collapse } } }`
  present in both. ✓
The ADR doc-comments are a lightly elaborated (not byte-identical) mirror, which is
appropriate for an ADR. Structure and signatures match the shipped code.

### 2. Binding Rule 5 Forward-compatibility paragraph — VERIFIED CORRECT
The inserted paragraph correctly states the Rust semantics: a `#[non_exhaustive]` struct
cannot be constructed via struct-literal syntax **outside the defining crate**, so external
crates must use `FindingsRender::new(...)`; same-crate code may still use struct literals.
This is accurate. The cross-reference "See CHANGELOG.md §[0.9.0] Forward-compatibility (F7-R2)"
resolves — that section exists (CHANGELOG.md line 31 on the branch). This genuinely closes
the F-A-001 semantic-anchoring gap: `terminal.rs` cites "ADR-0003 / LESSON-P2.10" as rationale
for `#[non_exhaustive]`, and ADR-0003 now actually records that decision.

### 3. Corrected line anchors `:502` / `:511` — VERIFIED CORRECT
On both `develop` and the PR branch, `src/main.rs`:
- `fn collapse_findings_from_flag` is at line **502** (ADR was stale `:505`, now `:502`). ✓
- `fn grouping_from_flag` is at line **511** (ADR was stale `:514`, now `:511`). ✓
Both corrections are exact.

### 4. Broadened regex `\b[A-Z]{2,}-[0-9A-Z]` catches all claimed prefixes — VERIFIED
Tested against a fixture with all nine claimed prefixes. All match:
`BC-`, `STORY-`, `LESSON-`, `VP-`, `ADR-`, `EC-`, `AC-`, `TD-`, `PG-`. ✓
The documented comment-block pattern and the actual `run:`-step grep pattern
(`ci.yml:264`) are identical — no doc/code drift.

### 5. False-positive risk for legitimate help text — VERIFIED SAFE
Tested the false-positive candidates from the PR description. None match:
`MITRE ATT&CK`, `JSON`, `TCP`, `ARP`, `Modbus`, `DNP3`, `non-zero`, `opt-in`,
`TLS-encrypted`, `(xN)`. ✓ The hyphen-then-`[0-9A-Z]` suffix correctly anchors to
ID-structured tokens only. The `\b` word boundary additionally prevents matching
camelCase identifiers (`fooBC-2`, `wordAC-7` do **not** match), which is desirable.
Most importantly: **zero matches against the actual `src/cli.rs` on the PR branch**, and
there are no uppercase-hyphen tokens at all in its `///` doc-comments. The O-1 safety claim holds.

### 6. PR description accuracy — VERIFIED ACCURATE
The description's file scope, line-anchor claims, regex claims, and false-positive analysis
all match what is in the diff and the codebase. Mermaid diagrams are consistent. The
"no src/ or test changes" claim is true for this PR's boundary.

### 7. CI YAML validity — VERIFIED
The modified `ci.yml` parses cleanly (`yaml.safe_load`). The `actions/checkout` SHA pin
(`df4cb1c...` # v6.0.3) is a 40-char SHA, compliant with the repo's action-pin gate.

## Findings

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| NIT | description | The false-positive analysis enumerates space-separated acronyms (MITRE, JSON, TCP, ARP) but does not call out the hyphenated form `MITRE-T1046`, which *would* match the broadened regex. The shipped `src/cli.rs` uses "MITRE ATT&CK" (space form), so there is no real false positive today. | Optional: add a one-line note that a future hyphenated `MITRE-Txxxx` mapping in a `///` doc-comment would trip the gate, so a maintainer adding one knows to move it to a `//` comment. Non-blocking. |

No BLOCKING and no SUGGESTION-level findings.

## Checklist Summary
1. Diff coherence — all changes relate to F7-R3 (ADR sync + gate broadening). ✓
2. Description accuracy — matches the diff and codebase. ✓
3. Test coverage — N/A; doc + CI only, no behavior change. The broadened gate itself is the
   regression guard and passes (zero matches on current `cli.rs`). ✓
4. Demo evidence — N/A; no user-visible behavior change (correctly stated in PR). ✓
5. Commit quality — semantic PR title (`docs(adr-0003): ...`) is valid. ✓
6. Diff size — 68 lines, well under the 500-line flag. ✓
7. Missing changes — none; all three F-A-001/M-1/M-2/O-1 remediations present. ✓
8. Dependency status — depends on F7-R2 shipped code (`#[non_exhaustive]` + `::new`),
   which is already on `develop` (PR #273 merged). ✓

**APPROVE.** This is a clean, accurate documentation-and-gate sync. Every quantitative claim
(line anchors, regex prefix coverage, false-positive safety, CHANGELOG cross-ref) was
independently verified against the codebase and holds.
