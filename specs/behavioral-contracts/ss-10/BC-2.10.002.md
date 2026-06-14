---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/mitre.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-10
capability: CAP-10
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Feature #8 DNP3 analyzer (F2). Added third ICS-unique MitreTactic variant: IcsImpact (Display 'Impact', ICS Impact tactic TA0105). Postconditions, Invariants, Edge Cases, and Canonical Test Vectors extended. all_tactics_in_report_order grows from 16→17 elements. — 2026-06-10"
  - "v1.4: Pass-8 remediation F-C-P8-M01: Architecture Anchors and Source Evidence re-anchored from stale :85-87 to verified :89-91 (IcsInhibitResponseFunction :89, IcsImpairProcessControl :90, IcsImpact :91). Lines 85-88 are Enterprise tactic Display arms (Collection :85, CommandAndControl :86, Exfiltration :87, Impact :88). — 2026-06-12"
  - "v1.5: D-069 adjudication — SUPERSEDES D-067. Research (mitre-impact-tactic-disambiguation.md; WCAG 2.4.6 unique headings; MITRE ATT&CK TA0040/TA0105) confirms IcsImpact Display must be 'Impact (ICS)' (disambiguated), NOT bare 'Impact'. A grouped findings report that co-renders Enterprise Impact (TA0040) and ICS Impact (TA0105) in the same output without a matrix-selection guard violates WCAG 2.4.6 (non-unique section headers). The shipped code src/mitre.rs:91 = 'Impact (ICS)' is CORRECT. The prior spec assertion 'Impact' (bare) was wrong. Updated: BC title (BC-2.10.002 title unchanged — describes ICS tactic rendering); Description; PC3; PC4; Invariant 2; EC-003; Canonical test vector for IcsImpact. Enterprise MitreTactic::Impact stays bare 'Impact'. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.002: ICS Tactics Render Without "ICS:" Prefix; IcsImpact Disambiguated as "Impact (ICS)"

## Description

The three ICS-unique `MitreTactic` variants render WITHOUT an "ICS:" prefix, with one
exception: `IcsImpact` renders as `"Impact (ICS)"` (not bare `"Impact"`) to disambiguate from
Enterprise `MitreTactic::Impact` (TA0040, bare "Impact") when both appear in the same grouped
report. `IcsInhibitResponseFunction` => "Inhibit Response Function" (no prefix);
`IcsImpairProcessControl` => "Impair Process Control" (no prefix);
`IcsImpact` => "Impact (ICS)" (parenthetical ICS qualifier — required per WCAG 2.4.6 to
distinguish from Enterprise Impact in a co-rendered section list). The design intention per
mitre.rs is to merge Enterprise and ICS findings into a single tactic-grouped report; the
`(ICS)` qualifier supplies the matrix context that a single-domain Navigator view would
otherwise provide. `IcsImpact` was added as the third ICS-unique variant in Feature #8 (DNP3)
to support T0827 "Loss of Control" (ICS Impact tactic TA0105). src/mitre.rs:91 emitting
"Impact (ICS)" is the correct and authoritative implementation (D-069, supersedes D-067).

## Preconditions

1. A `MitreTactic::IcsInhibitResponseFunction`, `MitreTactic::IcsImpairProcessControl`, or
   `MitreTactic::IcsImpact` value is formatted via Display.

## Postconditions

1. `IcsInhibitResponseFunction` displays as "Inhibit Response Function" (no prefix).
2. `IcsImpairProcessControl` displays as "Impair Process Control" (no prefix).
3. `IcsImpact` displays as "Impact (ICS)" (parenthetical ICS qualifier required to disambiguate
   from Enterprise `Impact` (TA0040) when both appear in the same grouped findings report;
   per D-069 adjudication and mitre-impact-tactic-disambiguation.md research).
4. No bare "ICS:" colon-prefix is prepended for any of the three ICS variants. The `(ICS)`
   parenthetical on IcsImpact is a disambiguation qualifier, not an "ICS:" prefix.

## Invariants

1. The ICS tactic names appear in `all_tactics_in_report_order()` AFTER all 14 Enterprise tactics.
2. The Display strings are the MITRE ICS ATT&CK canonical tactic names, rendered without a
   bare "ICS:" prefix. `IcsImpact` uses the disambiguated form "Impact (ICS)" rather than the
   bare canonical name "Impact" to prevent duplicate section headers in co-rendered Enterprise+ICS
   reports (WCAG 2.4.6; D-069). The human-readable Display is the only field that uses the
   "(ICS)" qualifier; machine-readable fields (TA-id, domain) use MITRE canonical values.
3. A consumer matching on tactic names must account for these ICS names appearing in the
   same list as Enterprise tactic names; there is no structural separation.
4. With three ICS variants, `all_tactics_in_report_order()` has length 17 (14 Enterprise + 3 ICS).
   IcsImpact appears at position [16] (0-indexed) after IcsInhibitResponseFunction [14] and
   IcsImpairProcessControl [15]. See BC-2.10.003 (length 16→17) and BC-2.10.004 (count 16→17).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IcsInhibitResponseFunction formatted | "Inhibit Response Function" (no prefix) |
| EC-002 | IcsImpairProcessControl formatted | "Impair Process Control" (no prefix) |
| EC-003 | IcsImpact formatted | "Impact (ICS)" (parenthetical ICS qualifier; disambiguates from Enterprise Impact TA0040; D-069) |
| EC-004 | all three ICS tactics in report order | IcsInhibitResponseFunction [14], IcsImpairProcessControl [15], IcsImpact [16] (0-indexed) after all 14 Enterprise tactics |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", MitreTactic::IcsInhibitResponseFunction) | "Inhibit Response Function" | happy-path |
| format!("{}", MitreTactic::IcsImpairProcessControl) | "Impair Process Control" | happy-path |
| format!("{}", MitreTactic::IcsImpact) | "Impact (ICS)" | happy-path (D-069: ICS qualifier required to disambiguate from Enterprise Impact TA0040) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | ICS tactic names render without prefix | unit: assert_eq on both ICS variants |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- ICS tactic Display is part of the MITRE mapping capability's output |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.001 -- related to (Enterprise tactic Display uses same impl)
- BC-2.10.003 -- composes with (ICS tactics appear last in all_tactics_in_report_order)

## Architecture Anchors

- `src/mitre.rs:89-91` -- ICS tactic Display arms (IcsInhibitResponseFunction :89, IcsImpairProcessControl :90, IcsImpact :91)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:89-91` (IcsInhibitResponseFunction :89, IcsImpairProcessControl :90, IcsImpact :91) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: hardcoded &'static str for ICS variants

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
