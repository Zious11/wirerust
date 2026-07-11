# PR Review — #396 `chore: maint-2026-07-11 cleanup`

**Verdict: APPROVE** — code is clean, gates green, no behavioral change. One MEDIUM
documentation-disposition item (MED-1) should be reconciled before merge; it is not a
code-correctness defect.

> Verdict-state note: GitHub's API blocks BOTH `--approve` and `--request-changes` on this
> PR because the author is the same account authenticated with `gh` (self-review is
> disallowed). No formal review *state* can be recorded by the authoring account — this
> review is therefore posted as a formal `gh pr review --comment` entry (state COMMENTED)
> carrying the complete verdict. A second reviewer/account must submit the formal Approve.

Fresh-eyes review of the diff, PR description, and disposition table only (diff-only
information wall — no codebase history consulted). This is a maintenance chore touching
docs, `bin/` Python, and source comments/lint attributes. No behavioral logic changes.

## Per-fix verification

| Finding | Verified in diff? | Notes |
|---|---|---|
| CR-001 | Yes | Hermetic `_find_repo_root` patch (tempdir + `.factory`) added; assertion tightened `!= 0` → `== 1`; both helpers restored in `finally`. Correct. |
| CR-002 | Yes | Docstring + inline comment reworded "6 levels" → "6 candidates (start inclusive)", matching `range(6)`. Correct. |
| CR-003 | Yes | `str.startswith(str(...))` → `_result_c.is_relative_to(_root_c)`. Proper hierarchy check. Correct. |
| PG-W-README-JSON-SCHEMA | Yes | `arp_summary` claim replaced with `analyzers[i].detail`. Internally consistent. |
| README-OPTIONS-L117-NEUTRAL-001 | Yes | `--arp-storm-rate` now states "at or above which a storm finding is emitted" + engineering-default caveat. Correct. |
| DNP3-TUNING-BIDIR-001 | Yes | Bidirectional / mirror-tap note added to threshold guidance. Correct. |
| DOC-NEW-001 | Yes | `PC-023` → `PC-020` in ADR-0002. Correct. |
| NEW-003 | Yes | `unclassified_port_counts` + `coverage_gaps_enabled` added to ADR-0001 struct snippet. Plausible (cannot confirm against real struct behind wall). |
| CHANGELOG-D3-T0830-DRIFT-001 | Yes | Strikethrough errata on v0.7.0 D3 entry noting `mitre_techniques: []`. Aligns with `arp.rs` code comment. |
| ARP-RATE-INTDIV-DOC-001 | Yes | "[integer division; truncates fractional rates]" added to `detect_storm` doc. Correct. |
| UNIT-FMT-5-20S-001 | Yes (see NIT-1) | `1-second window` → `1s window` in `cli.rs`. |
| PC-NEW-001 | Yes | Exactly 9 `#[allow(unused)]` removed from `pub const` items. Safe: `pub` items are exempt from `dead_code`, and CI clippy `-D warnings` is green, confirming no regression. |
| PC-NEW-002 | Partial (see MED-1) | 3 rationale comments added. |

## Findings

### [MEDIUM] MED-1 — PC-NEW-002: disposition table and CHANGELOG give conflicting counts

- Disposition table: *"6 too_many_arguments suppressions lack rationale | FIXED — 3 rationale comments added."*
- CHANGELOG: *"add rationale comments to the 3 ... suppressions **that lacked them**."*

These conflict. Either (a) there are 6 total suppressions of which only 3 lacked
rationale — then the finding wording "6 ... lack rationale" is wrong but the fix is
complete; or (b) 6 genuinely lacked rationale and only 3 were addressed — then 3 still
lack rationale and the disposition should read PARTIAL, not FIXED, and the CHANGELOG's
"the 3 that lacked them" is misleading. This cannot be disambiguated from the diff (the
other 3 suppressions are not in the diff, and the codebase is behind the diff-only wall).
Given this PR's entire purpose is accurate finding disposition, resolve the ambiguity
before merge: state explicitly whether 3 suppressions still lack rationale (relabel
PARTIAL) or whether only 3 of 6 ever lacked it (fix the finding wording). Non-blocking.

### [NIT] NIT-1 — UNIT-FMT-5-20S-001 disposition references "2s" but the change is "1-second" → "1s"

The disposition cell reads *"cli.rs Modbus arg '1-second' vs '2s' inconsistency"*, yet
the actual (and CHANGELOG-described) change harmonizes `1-second window` → `1s window`.
The `2s` reference appears to be a stray token in the disposition table. The code change
is correct and consistent with adjacent `Ns window` formatting; only the table wording is off.

### [NIT] NIT-2 — CHANGELOG-D3 errata switches technique ID without explanation

The struck-through original says "Attributed to **T0830**" while the errata says
"**T0814** attribution withheld." The errata correctly matches the `arp.rs` code
(T0814 withheld), but a reader is left wondering why T0830 became T0814. A half-sentence
noting the original T0830 attribution was itself erroneous would make the errata
self-explanatory. Append-only strikethrough errata on a historical CHANGELOG entry is
acceptable practice.

## Checklist summary

- Diff coherence: PASS — every hunk maps to a listed finding.
- Description accuracy: PASS except MED-1.
- No behavioral changes: PASS — only docs, comments, docstrings, and `#[allow(unused)]`
  removals; clippy `-D warnings` green confirms no dead-code regression.
- CHANGELOG completeness: PASS — 12 bullets cover all 13 findings (PC-NEW-001/002 share
  one bullet); `[Unreleased]` entry present as required (src/ and bin/ touched).
- Diff size: PASS — well under 500 lines.
- Demo evidence: N/A — maintenance chore, no acceptance criteria / behavioral surface. Exempt.
- Dependencies: none.

## Verdict

**APPROVE.** No code-correctness defects; changes accurately address their findings,
introduce no behavioral change, and all gates are green. Recommend reconciling MED-1 (the
PC-NEW-002 disposition count — table says "6 lack / FIXED, 3 added"; CHANGELOG says "the 3
that lacked them") before merge so the record unambiguously states whether the fix is
complete or partial; this matters because the PR's purpose is accurate finding disposition.
NITs are optional. (Platform note: GitHub blocks both `--approve` and `--request-changes`
on a self-authored PR, so a formal review *state* cannot be set by the authoring account;
a second reviewer must submit the formal Approve.)
