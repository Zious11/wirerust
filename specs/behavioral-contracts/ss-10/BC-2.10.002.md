---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.6: F5 ICS tactic-ID correctness fix. Three new ICS-unique MitreTactic variants added: IcsDiscovery (Display 'Discovery (ICS)', TA0102), IcsCollection (Display 'Collection (ICS)', TA0100), IcsCommandAndControl (Display 'Command and Control (ICS)', TA0101). Same D-069 disambiguation rationale applies: all three use parenthetical '(ICS)' qualifier to prevent duplicate section headers when co-rendered with Enterprise Discovery/Collection/CommandAndControl in a grouped report. Description, Postconditions, Invariants 2/4, Edge Cases EC-004, and Canonical Test Vectors updated. BC-2.10.003 grows to 20 total variants. — 2026-06-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.002: ICS Tactics Render Without "ICS:" Prefix; Colliding Names Disambiguated with "(ICS)" Qualifier

## Description

All six ICS-unique `MitreTactic` variants render WITHOUT a bare "ICS:" prefix. For variants
whose tactic name collides with an Enterprise tactic name, a parenthetical "(ICS)" qualifier
is appended to prevent duplicate section headers in co-rendered Enterprise+ICS grouped reports
(WCAG 2.4.6; same D-069 rationale):
- `IcsInhibitResponseFunction` => "Inhibit Response Function" (no Enterprise namesake; no qualifier needed)
- `IcsImpairProcessControl` => "Impair Process Control" (no Enterprise namesake; no qualifier needed)
- `IcsImpact` => "Impact (ICS)" (disambiguates from Enterprise Impact TA0040; D-069)
- `IcsDiscovery` => "Discovery (ICS)" (disambiguates from Enterprise Discovery TA0007; F5)
- `IcsCollection` => "Collection (ICS)" (disambiguates from Enterprise Collection TA0009; F5)
- `IcsCommandAndControl` => "Command and Control (ICS)" (disambiguates from Enterprise CommandAndControl TA0011; F5)
The design produces a single tactic-grouped report with unambiguous section headers; the
`(ICS)` qualifier supplies the matrix context. src/mitre.rs:91 emitting "Impact (ICS)" is the
correct and authoritative implementation (D-069, supersedes D-067). The three F5 variants
follow the same disambiguation convention.

## Preconditions

1. A `MitreTactic::IcsInhibitResponseFunction`, `MitreTactic::IcsImpairProcessControl`,
   `MitreTactic::IcsImpact`, `MitreTactic::IcsDiscovery`, `MitreTactic::IcsCollection`,
   or `MitreTactic::IcsCommandAndControl` value is formatted via Display.

## Postconditions

1. `IcsInhibitResponseFunction` displays as "Inhibit Response Function" (no prefix; no Enterprise namesake).
2. `IcsImpairProcessControl` displays as "Impair Process Control" (no prefix; no Enterprise namesake).
3. `IcsImpact` displays as "Impact (ICS)" (parenthetical ICS qualifier required to disambiguate
   from Enterprise `Impact` (TA0040) when both appear in the same grouped findings report;
   per D-069 adjudication and mitre-impact-tactic-disambiguation.md research).
4. `IcsDiscovery` displays as "Discovery (ICS)" (parenthetical ICS qualifier required to
   disambiguate from Enterprise `Discovery` (TA0007); same D-069 rationale; F5).
5. `IcsCollection` displays as "Collection (ICS)" (parenthetical ICS qualifier required to
   disambiguate from Enterprise `Collection` (TA0009); same D-069 rationale; F5).
6. `IcsCommandAndControl` displays as "Command and Control (ICS)" (parenthetical ICS qualifier
   required to disambiguate from Enterprise `CommandAndControl` (TA0011); same D-069 rationale; F5).
7. No bare "ICS:" colon-prefix is prepended for any ICS variant. The `(ICS)` parenthetical is
   a disambiguation qualifier, not an "ICS:" prefix.

## Invariants

1. The ICS tactic names appear in `all_tactics_in_report_order()` AFTER all 14 Enterprise tactics.
2. The Display strings are the MITRE ICS ATT&CK canonical tactic names, rendered without a
   bare "ICS:" prefix. ICS variants whose names collide with Enterprise tactic names use the
   disambiguated "(ICS)" parenthetical form to prevent duplicate section headers in co-rendered
   Enterprise+ICS reports (WCAG 2.4.6; D-069 convention). The human-readable Display is the
   only field that uses the "(ICS)" qualifier; machine-readable fields (TA-id, domain) use
   MITRE canonical values.
3. A consumer matching on tactic names must account for these ICS names appearing in the
   same list as Enterprise tactic names; there is no structural separation.
4. With six ICS variants, `all_tactics_in_report_order()` has length 20 (14 Enterprise + 6 ICS).
   ICS variants appear at positions [14]–[19] (0-indexed):
   IcsInhibitResponseFunction [14], IcsImpairProcessControl [15], IcsImpact [16],
   IcsDiscovery [17], IcsCollection [18], IcsCommandAndControl [19].
   See BC-2.10.003 v1.5 (length 17→20).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IcsInhibitResponseFunction formatted | "Inhibit Response Function" (no prefix; no Enterprise namesake) |
| EC-002 | IcsImpairProcessControl formatted | "Impair Process Control" (no prefix; no Enterprise namesake) |
| EC-003 | IcsImpact formatted | "Impact (ICS)" (parenthetical ICS qualifier; disambiguates from Enterprise Impact TA0040; D-069) |
| EC-004 | all six ICS tactics in report order | IcsInhibitResponseFunction [14], IcsImpairProcessControl [15], IcsImpact [16], IcsDiscovery [17], IcsCollection [18], IcsCommandAndControl [19] (0-indexed) after all 14 Enterprise tactics |
| EC-005 | IcsDiscovery formatted | "Discovery (ICS)" (parenthetical ICS qualifier; disambiguates from Enterprise Discovery TA0007; F5) |
| EC-006 | IcsCollection formatted | "Collection (ICS)" (parenthetical ICS qualifier; disambiguates from Enterprise Collection TA0009; F5) |
| EC-007 | IcsCommandAndControl formatted | "Command and Control (ICS)" (parenthetical ICS qualifier; disambiguates from Enterprise CommandAndControl TA0011; F5) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", MitreTactic::IcsInhibitResponseFunction) | "Inhibit Response Function" | happy-path |
| format!("{}", MitreTactic::IcsImpairProcessControl) | "Impair Process Control" | happy-path |
| format!("{}", MitreTactic::IcsImpact) | "Impact (ICS)" | happy-path (D-069: ICS qualifier required to disambiguate from Enterprise Impact TA0040) |
| format!("{}", MitreTactic::IcsDiscovery) | "Discovery (ICS)" | happy-path (F5: ICS qualifier required to disambiguate from Enterprise Discovery TA0007) |
| format!("{}", MitreTactic::IcsCollection) | "Collection (ICS)" | happy-path (F5: ICS qualifier required to disambiguate from Enterprise Collection TA0009) |
| format!("{}", MitreTactic::IcsCommandAndControl) | "Command and Control (ICS)" | happy-path (F5: ICS qualifier required to disambiguate from Enterprise CommandAndControl TA0011) |

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

- `src/mitre.rs:89-91` -- existing ICS tactic Display arms (IcsInhibitResponseFunction :89, IcsImpairProcessControl :90, IcsImpact :91)
- `src/mitre.rs` (F5 — new arms after :91) -- IcsDiscovery => "Discovery (ICS)", IcsCollection => "Collection (ICS)", IcsCommandAndControl => "Command and Control (ICS)"

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
