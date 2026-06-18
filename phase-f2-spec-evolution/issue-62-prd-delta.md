---
document_type: prd-delta
feature_issue: "#62"
feature_title: "Refactor TerminalReporter to enum-of-modes (FindingsRender)"
phase: f2
date: 2026-06-17
producer: product-owner
status: complete
---

# PRD Delta — Issue #62: FindingsRender Enum BC Re-anchoring

## Summary

Phase F2 spec evolution for issue #62 is a **re-anchoring pass only**. No new BCs were
created, no postconditions were changed, no test vectors were modified. The only change is
replacing bool field references (`show_mitre_grouping`, `collapse_findings`) with
`FindingsRender` enum variant references in existing BC Preconditions, Descriptions, and
related Invariant/EC rows.

## Migration Map Verification

The migration map was verified against `src/reporter/terminal.rs:187-197`:

```rust
if self.show_mitre_grouping {          // → FindingsRender::Grouped
    self.render_findings_grouped(...)
} else if self.collapse_findings {     // → FindingsRender::FlatCollapsed
    self.render_findings_collapsed(...)
} else {                               // → FindingsRender::FlatExpanded
    for f in findings { self.render_finding_flat(...) }
}
```

The dispatch confirms:
- `show_mitre_grouping = true` wins regardless of `collapse_findings` → `Grouped`
- `show_mitre_grouping = false, collapse_findings = true` → `FlatCollapsed`
- `show_mitre_grouping = false, collapse_findings = false` → `FlatExpanded`
- `run_summary` construction site → `render: FindingsRender::FlatCollapsed` by convention
  (inert — `run_summary` emits no FINDINGS section; no BC governs this path; the value is a
  structural placeholder distinguishing it from the dispatch-derived `run_analyze` mapping above)

This matches the F1-approved design, the research brief's migration map, and the
spec-changelog migration map exactly. All four construction sites are accounted for.

## BCs Touched (re-anchoring only)

| BC ID | Before | After | Sections Changed |
|-------|--------|-------|-----------------|
| BC-2.11.013 | v1.11 | v1.12 | Description, Precondition 1, Invariant 4, EC-007 |
| BC-2.11.014 | v1.6  | v1.7  | Precondition 1 |
| BC-2.11.017 | v1.13 | v1.14 | Description, Precondition 1, Postcondition 6, Invariants 1/5, EC-004/007/008, test vectors |
| BC-2.11.019 | v1.6  | v1.7  | Postcondition 9, Invariant 7, EC-008, EC-009 |
| BC-2.11.025 | v1.6  | v1.7  | Description, Preconditions 1-2, Invariant 5, EC-011 |
| BC-2.11.026 | v1.8  | v1.9  | Preconditions 1-2, EC-006, EC-007, EC-009 |
| BC-2.11.027 | v1.4  | v1.5  | Preconditions 1-2, EC-008 |
| BC-2.11.028 | v1.4  | v1.5  | Description, Preconditions 1-3, Postconditions 1-2/4, Invariants 1-2/6, EC-001..005, Architecture Anchors |
| BC-2.11.010 | v1.8  | v1.9  | Invariant 4 + EC-006 + EC-007 (fix-burst: missed in pass-1) |
| BC-2.11.015 | v1.7  | v1.8  | Precondition 1 (fix-burst: missed in pass-1) |
| BC-2.11.016 | v1.6  | v1.7  | Precondition 1 (fix-burst: missed in pass-1) |
| BC-2.11.029 | v1.2  | v1.3  | Precondition 4 + PC-1 inline qualifier + Architecture Anchors (fix-burst: missed in pass-1) |
| BC-2.11.025 | v1.7  | v1.8  | VP-table row: 'show_mitre_grouping=true suppresses collapse' → 'render = FindingsRender::Grouped suppresses collapse' (F2 adv-pass-2 fix F-6) |

**Total BCs touched: 12** (8 first-pass re-anchors + 4 fix-burst catches + 1 VP-table fix; includes .025 twice as it crossed two patch waves).

## No-Change Confirmation

Per F1 delta analysis, BC-2.11.018 (Colorization) was confirmed NOT to require re-anchoring
because `use_color` is an orthogonal field that remains a `bool` and is unaffected by the
refactor. BC-2.11.018 is the ONLY true no-change SS-11 reporter BC in this delta. The F1
document listed 9 BCs including BC-2.11.018 in a table but noted "no change needed" for it.
The full set of 12 touched BCs above is complete; no other SS-11 reporter BC was missed.

## Enum Type Context

```rust
/// Governs which rendering path the FINDINGS section uses.
/// Replaces the show_mitre_grouping: bool + collapse_findings: bool pair (v0.8.0).
/// The illegal state (show_mitre_grouping=true && collapse_findings=true) is now
/// structurally unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingsRender {
    /// Group findings by MITRE tactic (--mitre flag).
    Grouped,
    /// Collapse repeated findings into counted groups (default, v0.8.0+).
    FlatCollapsed,
    /// One display line per raw finding (--no-collapse / pre-v0.8.0 behavior).
    FlatExpanded,
}

pub struct TerminalReporter {
    pub use_color: bool,
    pub show_hosts_breakdown: bool,
    pub render: FindingsRender,
}
```

## Architecture Delta

**Updated (fix-burst 2026-06-17):** ADR-0003 was amended during the human gate review. A
"Render-Mode Enum (Issue #62 — v0.9.0)" subsection was added to ADR-0003 with Binding Rule 5:
`TerminalReporter.render: FindingsRender` is the authoritative render-mode discriminant; the
prior `show_mitre_grouping: bool` + `collapse_findings: bool` pair is retired. This supersedes
the original F1 recommendation that no ADR amendment was warranted.

Artifacts changed:
- `docs/adr/0003-reporting-pipeline-layering.md` — "Render-Mode Enum (Issue #62 — v0.9.0)" subsection added; Binding Rule 5 added.
- `architecture/ARCH-INDEX.md` — SS-11 entry updated to reflect `FindingsRender` enum field.

## Verification Delta

None. No new VPs. VP-012 unchanged. The refactor's correctness is compiler-verified
(missing construction sites = compile error) plus existing test suite passing after refactor.

## PRD Section Changes

No PRD section changes beyond BC index updates. The PRD Section 3 (Interface Definitions)
and Section 5 (Error Taxonomy) are unaffected — no CLI surface changes, no new error codes.

## story-writer Handoff

**FREEZE DECISION:** STORY-077, STORY-078, and STORY-118 are COMPLETED stories and are
frozen as-built records. They are NOT re-anchored as part of this spec evolution pass.
The enum migration is carried exclusively by STORY-120 (the new implementation story).

Stories affected by BC changes (frontmatter `bcs:` arrays and body AC tables):
- STORY-120 (new story carrying the FindingsRender enum migration implementation)

STORY-077/078/118 body content referencing `collapse_findings`/`show_mitre_grouping`
field names are frozen at their as-built state and intentionally not updated. Their
AC language reflects the v0.8.0 implementation they were written against. STORY-120
inherits the v1.8-anchored BC vocabulary.

Story-writer must propagate under `bc_array_changes_propagate_to_body_and_acs` policy
for STORY-120 only. The frontmatter `bcs:` arrays do NOT change (BC IDs are unchanged).
