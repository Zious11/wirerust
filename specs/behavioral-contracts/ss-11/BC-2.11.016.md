---
document_type: behavioral-contract
level: L3
version: "1.10"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/terminal.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchors: MITRE expansion range 239-244 → 246-251 (fn render_finding_grouped body: match at 246-250, close at 252), em-dash literal :241 → :248; guard clause at :240 → :247; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.6: PG-ARP-F2-007 — fix stale terminal.rs line anchors shifted by F2 multi-tag additions (STORY-100): fn render_finding_grouped body range :246-251 → :249-261 (is_empty guard at 249; ids join at 252; first-technique name lookup at 254-260; Some/em-dash arm at 259; None/unknown arm at 260); em-dash literal :248 → :259; verified against current HEAD — 2026-06-13"
  - "v1.7 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Precondition 1: 'show_mitre_grouping = true' → 'render = FindingsRender::Grouped'. Rationale: illegal-state elimination. No behavioral change."
  - "v1.5: ARP-F2 Pass-14 Burst-7 — mitre_technique (singular) → mitre_techniques (Vec<String>) in Precondition 2, Postcondition 4, EC-003, and all three Canonical Test Vector rows. Shipped Finding struct uses Vec<String>; 'no MITRE line' condition is empty vec, not None. — 2026-06-13"
  - "v1.8 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — render_finding_grouped MITRE expansion body :249-261 → :313-327 (is_empty guard at 313; ids join at 316; first-technique name lookup at 318-325; known branch with em-dash at 323; unknown branch at 324; closing at 327); em-dash literal :259 → :323; Architecture Anchor + Source Evidence path updated."
  - "v1.10 2026-06-18: STORY-119 split D-120 — traceability backlinks updated: Stories field expanded from STORY-078 to STORY-078, STORY-122 (A, preserves em-dash MITRE line format byte-identical in render_finding_grouped), STORY-119 (B, grouped-collapse reuses this format for N=1 singletons and em-dash logic for N≥2 group reps per BC-2.11.034). No normative change."
  - "v1.9 2026-06-18: STORY-119 vocabulary migration — D-110 struct form: FindingsRender::Grouped → render.grouping == Grouping::Grouped in Precondition 1. No behavioral change."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.016: MITRE Grouping Expands Per-Finding Line with Em-Dash and Name

## Description

In MITRE-grouped rendering, each finding's MITRE line is expanded from the bare `MITRE: <id>`
format to `MITRE: <id> -- <TechniqueName>` for technique IDs that are present in the catalog.
The separator is U+2014 (EM DASH), NOT the ASCII hyphen-minus sequence `--`. Unknown IDs
render as `MITRE: <id> (unknown)`.

## Preconditions

1. `TerminalReporter.render.grouping == Grouping::Grouped` (applies to both `{Grouped, Collapsed}`
   and `{Grouped, Expanded}` paths; the em-dash MITRE expansion occurs on all `render_finding_grouped`
   calls regardless of collapse axis).
2. A finding has a non-empty `mitre_techniques` vec with at least one technique ID in the catalog.

## Postconditions

1. The MITRE line reads: `    MITRE: <id> \u{2014} <name>\n` where `<name>` is the string
   returned by `technique_name(id)`.
2. The separator character is U+2014 (EM DASH), not two hyphens `--`.
3. For unknown IDs, the line reads: `    MITRE: <id> (unknown)\n`.
4. For `mitre_techniques = vec![]` (empty), no MITRE line is rendered (not even "(unknown)"). The key-absent-when-empty rule: `skip_serializing_if = Vec::is_empty` in the JSON path; no MITRE line in the terminal path.

## Invariants

1. The em-dash character U+2014 is hardcoded at terminal.rs:323 as `\u{2014}`.
2. `technique_name(id)` is the authoritative source for the name string.
3. The MITRE line is rendered by `render_finding_grouped`, called only in grouping mode.
4. In default (flat) mode, `render_finding_flat` renders `MITRE: <id>` only (no expansion).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Known ID T1036 | "MITRE: T1036 \u{2014} Masquerading\n" (or whatever technique_name returns) |
| EC-002 | Unknown ID T9999 | "MITRE: T9999 (unknown)\n" |
| EC-003 | mitre_techniques = vec![] (empty) | No MITRE line rendered for this finding |
| EC-004 | Downstream grep for ASCII "--" separator | Will miss em-dash; must grep for U+2014 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with mitre_techniques=["T1036"] in grouped mode | MITRE line contains U+2014 and technique name | happy-path |
| Finding with mitre_techniques=["T9999"] (unknown) | "MITRE: T9999 (unknown)" | happy-path |
| Finding with mitre_techniques=vec![] in grouped mode | No MITRE line in output | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Em-dash and name in grouped MITRE line | unit: mitre_grouping_expands_per_finding_line_with_technique_name |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the expanded MITRE line format with em-dash and technique name is a documented output encoding contract that downstream grep-based pipelines must account for |
| L2 Domain Invariants | INV-9 (MITRE Technique ID Format -- the expansion uses the catalog's name mapping) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078, STORY-122 (A — preserves em-dash MITRE line format in render_finding_grouped byte-identical), STORY-119 (B — reuses this format for N=1 singletons; grouped-collapse em-dash logic for N≥2 reps governed by BC-2.11.034) |
| Origin BC | BC-RPT-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.017 -- contrasts with (default mode does NOT expand; BC-016 is grouped-mode only)
- BC-2.11.015 -- composes with (unknown-ID handling uses "(unknown)" not em-dash)
- BC-2.10.005 -- depends on (technique_name provides the expansion string)

## Architecture Anchors

- `src/reporter/terminal.rs:313-327` -- render_finding_grouped MITRE line expansion (is_empty guard at 313; ids join at 316; first-technique name lookup at 318-325; known branch with em-dash at 323; unknown branch at 324; closing at 327)
- `src/reporter/terminal.rs:323` -- `\u{2014}` em-dash literal
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_016_known_id_em_dash_and_name

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:313-327` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: mitre_grouping_expands_per_finding_line_with_technique_name
- **guard clause**: `match technique_name(id)` branch at line 247

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed. The U+2014 em-dash character is a hidden output-encoding contract;
downstream pipelines that grep for ASCII `--` will silently miss these lines.
