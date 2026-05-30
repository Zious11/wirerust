---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.001: MitreTactic Display Renders Enterprise Tactics with Canonical Spacing

## Description

`MitreTactic` implements `fmt::Display` returning the canonical ATT&CK tactic name string
for each Enterprise tactic variant. Multi-word tactic names include spaces (e.g., "Resource
Development", "Defense Evasion", "Command and Control"). These strings are used as section
headers in the `--mitre` grouped terminal output and as JSON keys in MITRE-grouped reports.

## Preconditions

1. A `MitreTactic` value is formatted via Display.

## Postconditions

1. The 14 Enterprise tactic variants render as:
   - Reconnaissance => "Reconnaissance"
   - ResourceDevelopment => "Resource Development"
   - InitialAccess => "Initial Access"
   - Execution => "Execution"
   - Persistence => "Persistence"
   - PrivilegeEscalation => "Privilege Escalation"
   - DefenseEvasion => "Defense Evasion"
   - CredentialAccess => "Credential Access"
   - Discovery => "Discovery"
   - LateralMovement => "Lateral Movement"
   - Collection => "Collection"
   - CommandAndControl => "Command and Control"
   - Exfiltration => "Exfiltration"
   - Impact => "Impact"
2. Spaces are present in multi-word names as shown above.
3. "Command and Control" uses lowercase "and" (canonical ATT&CK form).

## Invariants

1. Display strings match MITRE ATT&CK v14+ canonical tactic names exactly.
2. The enum uses `f.write_str(name)` (not `write!`); no additional formatting applied.
3. ICS tactic variants use different rules (see BC-2.10.002).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | MitreTactic::CommandAndControl | "Command and Control" (lowercase "and") |
| EC-002 | MitreTactic::PrivilegeEscalation | "Privilege Escalation" |
| EC-003 | MitreTactic::Reconnaissance | "Reconnaissance" (single word) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", MitreTactic::DefenseEvasion) | "Defense Evasion" | happy-path |
| format!("{}", MitreTactic::CommandAndControl) | "Command and Control" | happy-path |
| format!("{}", MitreTactic::Reconnaissance) | "Reconnaissance" | happy-path |
| format!("{}", MitreTactic::Impact) | "Impact" | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All 14 Enterprise tactic variants render correct canonical strings | unit: exhaustive match on all_tactics_in_report_order() |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- tactic Display strings are the human-readable labels of the MITRE mapping capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format -- tactic names must match ATT&CK canonical form) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.002 -- related to (ICS tactic variants use the same impl with different strings)
- BC-2.10.003 -- composes with (all_tactics_in_report_order returns these variants)

## Architecture Anchors

- `src/mitre.rs:68-90` -- impl fmt::Display for MitreTactic

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:68-90` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: hardcoded &'static str match arms

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. When ATT&CK releases a new version that renames a tactic, this
Display impl must be updated in sync with the new version to preserve correct grouping.
