---
document_type: behavioral-contract
level: L3
version: "1.4"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.002: ICS Tactics Render Unprefixed

## Description

The three ICS-unique `MitreTactic` variants render WITHOUT an "ICS:" prefix or any other
matrix qualifier: `IcsInhibitResponseFunction` => "Inhibit Response Function",
`IcsImpairProcessControl` => "Impair Process Control", and `IcsImpact` => "Impact". This
mirrors the MITRE ICS ATT&CK display convention for tactic names (which are used directly in
grouped reports alongside Enterprise tactic names). The design intention per mitre.rs is to
merge Enterprise and ICS findings into a single tactic-grouped report with one section per
tactic name. `IcsImpact` was added as the third ICS-unique variant in Feature #8 (DNP3) to
support T0827 "Loss of Control" (ICS Impact tactic TA0105).

## Preconditions

1. A `MitreTactic::IcsInhibitResponseFunction`, `MitreTactic::IcsImpairProcessControl`, or
   `MitreTactic::IcsImpact` value is formatted via Display.

## Postconditions

1. `IcsInhibitResponseFunction` displays as "Inhibit Response Function".
2. `IcsImpairProcessControl` displays as "Impair Process Control".
3. `IcsImpact` displays as "Impact" (canonical ICS ATT&CK tactic name for TA0105).
4. No "ICS:" or other matrix prefix is prepended for any of the three ICS variants.

## Invariants

1. The ICS tactic names appear in `all_tactics_in_report_order()` AFTER all 14 Enterprise tactics.
2. The Display strings are the MITRE ICS ATT&CK canonical tactic names (unprefixed).
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
| EC-003 | IcsImpact formatted | "Impact" (no prefix; canonical ICS TA0105 tactic name) |
| EC-004 | all three ICS tactics in report order | IcsInhibitResponseFunction [14], IcsImpairProcessControl [15], IcsImpact [16] (0-indexed) after all 14 Enterprise tactics |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", MitreTactic::IcsInhibitResponseFunction) | "Inhibit Response Function" | happy-path |
| format!("{}", MitreTactic::IcsImpairProcessControl) | "Impair Process Control" | happy-path |
| format!("{}", MitreTactic::IcsImpact) | "Impact" | happy-path (new F2 DNP3) |

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
