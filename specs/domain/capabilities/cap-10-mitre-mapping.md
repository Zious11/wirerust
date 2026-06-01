---
artifact: L2-cap-10
traces_to: ../domain-spec.md
cap_id: CAP-10
title: MITRE ATT&CK Mapping
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-10: MITRE ATT&CK Mapping

## What the system does today

`mitre.rs` (C-16) provides a static lookup table (`technique_info`) mapping MITRE technique
IDs to `(technique_name: &'static str, tactic: MitreTactic)` pairs. The `TerminalReporter`
groups findings by tactic when `--mitre` is set.

**Sources:** C-16 mitre.rs (module-decomposition.md). BC-2.10.001..009.

## Technique catalog

The `technique_info` function contains 15 IDs in its match arms (pass-2 R2 confirmed; pass-8
corrects pass-6's "16" claim):

| ID | Technique name | Tactic |
|---|---|---|
| T1027 | Obfuscated Files or Information | DefenseEvasion |
| T1036 | Masquerading | DefenseEvasion |
| T1040 | Network Sniffing | CredentialAccess (*catalogued, never emitted*) |
| T1046 | Network Service Discovery | Discovery |
| T1071 | Application Layer Protocol | CommandAndControl (*catalogued, never emitted*) |
| T1071.001 | Web Protocols | CommandAndControl (*catalogued, never emitted*) |
| T1071.004 | DNS | CommandAndControl (*catalogued, never emitted*) |
| T1083 | File and Directory Discovery | Discovery |
| T1499.002 | Service Exhaustion Flood | Impact |
| T1505.003 | Web Shell | Persistence |
| T1573 | Encrypted Channel | CommandAndControl (*catalogued, never emitted*) |
| T0846 | Remote System Discovery | Discovery |
| T0855 | Unauthorized Command Message | IcsImpairProcessControl (*catalogued, never emitted*) |
| T0856 | Spoof Reporting Message | IcsImpairProcessControl (*catalogued, never emitted*) |
| T0885 | Commonly Used Port | CommandAndControl (*catalogued, never emitted*) |

**Emitted (6):** T1027, T1036, T1046, T1083, T1499.002, T1505.003.
**Catalogued but never emitted (9):** T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885.

These 9 staged IDs are documented in mitre.rs source comments (P3.04 / #89; open item O-04).
They are pre-positioned for future analyzers (DNS tunneling, ICS protocol analysis, etc.)
and are intentionally present in the catalog without corresponding emission sites.

## MitreTactic enum (E-27)

16 variants: 14 Enterprise ATT&CK tactics (Reconnaissance through Impact) + 2 ICS-unique
(`IcsInhibitResponseFunction`, `IcsImpairProcessControl`). The enum is `#[non_exhaustive]`
so adding new tactics in a future ATT&CK version is non-breaking for downstream match consumers.

**IcsInhibitResponseFunction (unreachable, open item):** `MitreTactic::IcsInhibitResponseFunction`
is declared (mitre.rs:64), appears in `Display` (mitre.rs:85) and in `all_tactics_in_report_order`
(mitre.rs:111), but no `technique_info` arm maps any technique ID to it. It is therefore
unreachable via any current emission path. This is analogous to the staged ICS techniques
(T0855, T0856): a forward declaration awaiting a Modbus/DNP3 analyzer that will assign a
technique to this tactic. Tracked as part of O-04.

## CLI --mitre flag

When `--mitre` is set in `Commands::Analyze`, `TerminalReporter` renders findings grouped by
tactic. Unknown technique IDs (not in the catalog) display as `<id> (unknown)`
(terminal.rs:248-249). `JsonReporter` does not group by tactic; it emits the raw
`mitre_technique: Option<String>` field per Finding (omitted from JSON when None).

## Unknown-ID handling (VO-6)

If an analyzer emits a malformed or unrecognized MITRE technique ID:
- `technique_name()` returns `None`.
- `technique_tactic()` returns `None`.
- Terminal reporter shows `<id> (unknown)`.
- JSON output contains the raw ID string unmodified.

## BC references

BC-2.10.001..004: MitreTactic Display rendering + all_tactics_in_report_order.
BC-2.10.005: technique_name returns Some for every seeded ID (15 total).
BC-2.10.006: technique_name returns None for unknown IDs.
BC-2.10.007: technique_tactic returns correct tactic for every seeded ID.
BC-2.10.008: all emitted technique IDs resolve in lookup.
BC-2.10.009: MitreTactic is #[non_exhaustive].
Tactic-grouped rendering is in ss-11: BC-2.11.013..017 (TerminalReporter --mitre grouping).
