---
document_type: prd-delta
story: STORY-119
phase: F2-spec-evolution
burst: INTEGRATE
author: product-owner
timestamp: 2026-06-18T00:00:00Z
traces_to: .factory/specs/prd.md
versioning: held-release
target_release: v0.9.0-develop
---

# PRD Delta: STORY-119 — Grouped-Collapse and FindingsRender Type Reshape

## 1. Overview

STORY-119 delivers two coupled changes to the terminal reporting subsystem (SS-11):

1. **Grouped-Collapse Feature**: `--mitre` alone now routes to
   `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`,
   collapsing duplicate findings within each tactic bucket before output. Before
   STORY-119, `--mitre` always rendered every finding individually (`Collapse::Expanded`);
   grouped collapse was explicitly deferred.

2. **D-110 Type Reshape**: `FindingsRender` transitions from a 3-variant enum to a
   struct-of-orthogonal-enums, unlocking the previously missing
   `{Grouped, Collapsed}` mode and eliminating illegal states:
   ```rust
   // v0.9.0 (pre-STORY-119)
   pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }

   // STORY-119 (D-110)
   pub enum Grouping  { Grouped, Flat }
   pub enum Collapse  { Collapsed, Expanded }
   pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }
   ```

These changes are bundled into the unreleased v0.9.0 develop line. No separate
semver bump is made for STORY-119 alone (held-release versioning).

---

## 2. Feature: Grouped-Collapse (`--mitre` Default Behavior Change)

### 2.1 Pre-STORY-119 Behavior

`--mitre` alone routed to `FindingsRender::Grouped` which rendered every finding
individually within each tactic bucket, never collapsing duplicates. The `--no-collapse`
flag had no effect on `--mitre` output (it only governed flat mode).

### 2.2 Post-STORY-119 Behavior

| CLI Flags | Resulting Render Mode | Notes |
|-----------|----------------------|-------|
| (neither) | `{Flat, Expanded}` | unchanged |
| `--no-collapse` | `{Flat, Expanded}` | unchanged |
| `--mitre` | `{Grouped, Collapsed}` | **NEW DEFAULT** — collapses per bucket |
| `--mitre --no-collapse` | `{Grouped, Expanded}` | preserves pre-STORY-119 behavior via explicit opt-out |
| (implicit collapse) | `{Flat, Collapsed}` | unchanged |

### 2.3 --no-collapse Dual-Scope

Before STORY-119, `--no-collapse` suppressed flat-mode collapse only.
After STORY-119, `--no-collapse` suppresses collapse on BOTH axes:
- Flat mode: `collapse = Collapse::Expanded`
- Grouped mode: `collapse = Collapse::Expanded`

The wiring expression in `run_analyze` (F4-pending):
```rust
render: FindingsRender {
    grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat },
    collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded },
}
```

### 2.4 Per-Bucket Collapse Semantics

In `{Grouped, Collapsed}` mode:
- `collapse_findings_pass` is called once **per tactic bucket** (not once globally).
- The `CollapseKey` is the same `(category, verdict, confidence, summary)` four-tuple
  used in flat mode (BC-2.11.031).
- Each collapsed group emits a header line with `(xN)` suffix when N >= 2 (BC-2.11.031).
- Singleton groups (N = 1) render without suffix (BC-2.11.031).
- Bucket ordering is unchanged by collapse (BC-2.11.033).

---

## 3. D-110 Type Reshape: FindingsRender Enum to Struct

### 3.1 Migration Map

| Old Variant (v0.9.0 enum) | New Struct Form (STORY-119) | Semantic Change |
|---------------------------|----------------------------|-----------------|
| `FindingsRender::Grouped` | `{ grouping: Grouping::Grouped, collapse: Collapse::Expanded }` | None — preserves suffix-free grouped behavior |
| `FindingsRender::FlatCollapsed` | `{ grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | None |
| `FindingsRender::FlatExpanded` | `{ grouping: Grouping::Flat, collapse: Collapse::Expanded }` | None |
| (did not exist) | `{ grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` | **NEW** — grouped-collapse |

### 3.2 Four-Arm Dispatch

The old 3-arm match is replaced by a 4-arm match on the Cartesian product:

```rust
match (self.render.grouping, self.render.collapse) {
    (Grouping::Grouped, Collapse::Expanded)   => self.render_findings_grouped(&mut out, findings),
    (Grouping::Grouped, Collapse::Collapsed)  => self.render_findings_grouped_collapsed(&mut out, findings),
    (Grouping::Flat,    Collapse::Collapsed)  => self.render_findings_collapsed(&mut out, findings),
    (Grouping::Flat,    Collapse::Expanded)   => { for f in findings { self.render_finding_flat(&mut out, f); } }
}
```

The new arm `(Grouping::Grouped, Collapse::Collapsed)` dispatches to the new
`render_findings_grouped_collapsed` function (F4-pending implementation).

---

## 4. New Behavioral Contracts (BC Count: 29 → 34)

Five new BCs were authored in the preceding burst. Each governs a distinct aspect
of the grouped-collapse mode:

| BC ID | Title | Governs |
|-------|-------|---------|
| BC-2.11.030 | CLI-to-Render Mode Mapping Contract | Maps `(--mitre, --no-collapse)` flag combinations to all 4 render mode quadrants |
| BC-2.11.031 | Per-Bucket Count Suffix Rule | `(xN)` suffix emission in `{Grouped, Collapsed}` mode; singleton omits suffix |
| BC-2.11.032 | Per-Bucket Evidence Sampling | Evidence lines shown per-bucket in grouped-collapse output |
| BC-2.11.033 | Tactic-Bucket Ordering Invariant Under Grouped-Collapse | Collapse does not reorder buckets; canonical tactic order preserved |
| BC-2.11.034 | MITRE Line Format in Grouped-Collapse | Collapsed header line format: `<tactic>: (<n> findings, xN)` |

All five BCs are located at:
`.factory/specs/behavioral-contracts/ss-11/BC-2.11.030.md` through `BC-2.11.034.md`

---

## 5. Revised Behavioral Contracts (Part A — Substantive Revisions)

Four existing BCs received substantive revisions to remove deferral language and
update normative scope to reflect the STORY-119 feature being delivered.

### BC-2.11.013 — MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last

**Version bump:** v1.13 → v1.14

| Change Area | Before | After |
|-------------|--------|-------|
| Precondition 1 | `FindingsRender::Grouped` | `render.grouping == Grouping::Grouped` (struct form) |
| Invariant 4 | "Collapse within grouped/--mitre mode is deferred to STORY-119" | Dual-state invariant: `{Grouped, Expanded}` — no `(xN)` suffix on headers; `{Grouped, Collapsed}` — per-bucket `(xN)` suffix emitted (per BC-2.11.031) |
| EC-007 | Single entry: grouped path never emits `(xN)` | Split into EC-007a (`{Grouped, Expanded}` — no suffix) and EC-007b (`{Grouped, Collapsed}` — emits per-bucket suffix) |
| Related BCs | Did not reference 030-034 | Added BC-2.11.030 through BC-2.11.034 |
| Description | Referenced deferred STORY-119 work | Updated to describe both collapse states |

### BC-2.11.025 — Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key

**Version bump:** v1.10 → v1.11

| Change Area | Before | After |
|-------------|--------|-------|
| Scope | Implicitly covered flat mode; Invariant 5 deferred grouped collapse | Explicitly scoped to `Grouping::Flat`; deferral language retired |
| Precondition 1 | `FindingsRender::FlatCollapsed` | `render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` |
| Precondition 2 | Vague collapse-enabled reference | `render.collapse == Collapse::Collapsed` |
| Invariant 5 | "Grouped-mode collapse is deferred to STORY-119" | "This BC governs Grouping::Flat only. For grouped-mode collapse, see BC-2.11.031." |
| EC-011 | Referenced `FlatCollapsed` variant | Updated to struct form |
| VP row | Generic flat-collapse VP | Updated description to reference flat scope |

### BC-2.11.026 — Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix

**Version bump:** v1.11 → v1.12

| Change Area | Before | After |
|-------------|--------|-------|
| Preconditions | Applied to any collapse mode | Narrowed to `{Flat, Collapsed}` explicitly |
| Postcondition 4 | "Grouped path MUST NOT emit (xN) suffix" | Revised: grouped-expanded path is still suffix-free; grouped-collapsed path (`{Grouped, Collapsed}`) emits `(xN)` per-bucket per BC-2.11.031 |
| EC-006 | Single grouped entry | Split to distinguish `{Grouped, Expanded}` (no suffix) vs `{Grouped, Collapsed}` (emits suffix) |
| EC-007 | Single grouped entry | Same split |
| EC-009 | Single grouped entry | Same split |
| Canonical Test Vectors | Two vectors referenced old enum form | Updated to struct form |

### BC-2.11.028 — --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering

**Version bump:** v1.7 → v1.8

| Change Area | Before | After |
|-------------|--------|-------|
| Description | Scoped to flat-mode opt-out only | Completely rewritten: dual-scope (suppresses collapse in both flat AND grouped modes) |
| Postcondition 1 | `FindingsRender::FlatExpanded` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` |
| Postcondition 2 | Single mode reference | Four render modes documented: all combinations of `(Flat/Grouped) x (Collapsed/Expanded)` |
| Postcondition 4 | `--mitre` + `--no-collapse` unspecified | Explicitly documents `{Grouped, Expanded}` result |
| Invariants 1, 2, 6 | Old enum variant form | Updated to struct form |
| Precondition 3 | Old if-expression wiring | Updated to new struct construction wiring |
| EC-001 through EC-005 | Single-axis edge cases | Updated to cover both axes; EC-004/005 added for grouped-mode opt-out |
| Architecture Anchor | Legacy `run_analyze` wiring expression | F4-pending struct construction target |
| Canonical Test Vectors | Two vectors (flat mode only) | Four vectors: flat expanded, flat collapsed, grouped expanded (--no-collapse + --mitre), grouped collapsed |
| VP rows | One VP | Two VPs: flat opt-out + grouped opt-out (`--no-collapse --mitre` case) |

---

## 6. Vocabulary Sweep (Part B — Consuming-Surface Migration)

All SS-11 BC files were scanned for references to the v0.9.0 three-variant enum
(`FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded`).
Eight additional BCs were found and updated. Each received a version bump and a
modified stanza citing STORY-119 vocabulary migration.

This sweep enforces the lesson from PG-62-F5-POSTMERGE-ANCHOR-001 / STORY-121:
consuming-surface references must be updated in the same burst as the type redesign,
not deferred to a follow-up pass.

| BC ID | Old Version | New Version | References Updated |
|-------|-------------|-------------|-------------------|
| BC-2.11.010 | v1.10 | v1.11 | Invariant 4, EC-006, EC-007 — `FindingsRender::FlatCollapsed` → struct form |
| BC-2.11.014 | v1.8 | v1.9 | Precondition 1 — `FindingsRender::Grouped` → `render.grouping == Grouping::Grouped` |
| BC-2.11.015 | v1.9 | v1.10 | Precondition 1 — `FindingsRender::Grouped` → `render.grouping == Grouping::Grouped` |
| BC-2.11.016 | v1.8 | v1.9 | Precondition 1 — `FindingsRender::Grouped` → `render.grouping == Grouping::Grouped` (note: em-dash expansion applies to both collapse states) |
| BC-2.11.017 | v1.15 | v1.16 | Description, Preconditions 1-2, Postcondition 6, Invariants 1/3/5, EC-004/007/008, Canonical Test Vectors — extensive `FlatCollapsed`/`FlatExpanded`/`Grouped` → struct form |
| BC-2.11.019 | v1.9 | v1.10 | Postcondition 9 (four-arm dispatch), Invariant 7, EC-008/009 (split into EC-009a/009b), Related BCs (added BC-2.11.031) |
| BC-2.11.027 | v1.6 | v1.7 | Preconditions 1-2 — `FindingsRender::FlatCollapsed` → `{Flat, Collapsed}`; EC-008 `FlatExpanded` → `{Flat, Expanded}` |
| BC-2.11.029 | v1.6 | v1.7 | Precondition 4 — `FindingsRender::FlatCollapsed` → `render.collapse == Collapse::Collapsed`; Architecture Anchor wiring expression → struct form |

All 8 BCs are located at:
`.factory/specs/behavioral-contracts/ss-11/BC-2.11.{010,014,015,016,017,019,027,029}.md`

---

## 7. Versioning: Held-Release

STORY-119 is bundled into the **unreleased v0.9.0 develop line**. No separate
semver bump is made for STORY-119 alone. The version number will advance as part
of the v0.9.0 release cycle after all STORY-119 F4 implementation work is complete
and CI is green.

This follows the held-release pattern established for this project: spec-evolution
and type-design work in F2 does not bump the version; the version bumps only when
the corresponding implementation merges to main.

---

## 8. SS-11 BC Count Summary

| Metric | Before STORY-119 | After STORY-119 |
|--------|-----------------|-----------------|
| Total SS-11 BCs | 29 | 34 |
| New BCs added | — | 5 (BC-2.11.030–034) |
| Substantively revised | — | 4 (BC-2.11.013, 025, 026, 028) |
| Vocabulary-sweep-only | — | 8 (BC-2.11.010, 014, 015, 016, 017, 019, 027, 029) |
| Total BCs touched | — | 12 |
| Retired BCs | 0 | 0 |

No BC IDs were renumbered or retired. Append-only ID protection is maintained per
the VSDD methodology.

---

## 9. Architecture Anchor Note (F4-Pending)

The following implementation targets are F4-pending (not yet in shipped code).
BCs reference these anchors with `F4-pending` annotation:

- `src/reporter/terminal.rs` — `pub enum FindingsRender` to be replaced with
  `pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }`
  plus `pub enum Grouping` and `pub enum Collapse` (D-110).
- `src/reporter/terminal.rs` — new `render_findings_grouped_collapsed` function
  implementing per-bucket collapse + output for the `{Grouped, Collapsed}` arm.
- `src/main.rs` (`run_analyze`) — TerminalReporter construction expression to use
  the new struct form: `render: FindingsRender { grouping: ..., collapse: ... }`.
- Four-arm `match (self.render.grouping, self.render.collapse)` dispatch replacing
  the existing three-arm `if`-chain.

The F4 implementer must reconcile actual line numbers against BC architecture anchors
after implementing D-110, per the anchor drift protocol.

---

## 10. Confirmation

- No story files were edited during this burst. Story-writer owns story file updates (F3).
- No git commits were made. State-manager commits all `.factory/` git operations.
- The vocabulary sweep covered all 29 pre-existing SS-11 BCs. Zero old enum variant
  references remain in any BC file under `.factory/specs/behavioral-contracts/ss-11/`.
