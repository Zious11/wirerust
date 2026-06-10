---
artifact: L2-cap-10
traces_to: ../domain-spec.md
cap_id: CAP-10
title: MITRE ATT&CK Mapping
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.2"
modified:
  - date: 2026-06-10
    actor: architect
    reason: "Remap revoked ATT&CK-ICS v19 IDs: T0855→T1692.001 (Unauthorized Message: Command Message), T0856→T1692.002 (Unauthorized Message: Reporting Message) (issue #222)."
  - date: 2026-06-10
    actor: architect
    reason: "Fix F2 staleness: catalog expanded to 21 IDs (add Modbus techniques T0836/T0814/T0806/T0835/T0831/T0888); correct emitted count 6→13; correct never-emitted count 9→8 (T1692.001 is emitted, not staged); fix IcsInhibitResponseFunction paragraph (T0814→IcsInhibitResponseFunction is active, not unreachable); update BC-2.10.005 total (issue #222)."
---

# CAP-10: MITRE ATT&CK Mapping

## What the system does today

`mitre.rs` (C-16) provides a static lookup table (`technique_info`) mapping MITRE technique
IDs to `(technique_name: &'static str, tactic: MitreTactic)` pairs. The `TerminalReporter`
groups findings by tactic when `--mitre` is set.

**Sources:** C-16 mitre.rs (module-decomposition.md). BC-2.10.001..009.

## Technique catalog

The `technique_info` function contains 21 IDs in its match arms (F2 corrected; 15 brownfield IDs
plus 6 new Modbus ICS techniques added in Feature #7: T0836, T0814, T0806, T0835, T0831, T0888):

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
| T0846 | Remote System Discovery | IcsDiscovery (*catalogued, never emitted*) |
| T1692.001 | Unauthorized Message: Command Message | IcsImpairProcessControl |
| T1692.002 | Unauthorized Message: Reporting Message | IcsImpairProcessControl (*catalogued, never emitted*) |
| T0885 | Commonly Used Port | CommandAndControl (*catalogued, never emitted*) |
| T0836 | Modify Parameter | IcsImpairProcessControl |
| T0814 | Denial of Service | IcsInhibitResponseFunction |
| T0806 | Brute Force I/O | IcsImpairProcessControl |
| T0835 | Manipulate I/O Image | IcsImpairProcessControl |
| T0831 | Manipulation of Control | IcsImpairProcessControl |
| T0888 | Remote System Information Discovery | IcsDiscovery |

**Emitted (13):** T1027, T1036, T1046, T1083, T1499.002, T1505.003, T1692.001, T0836, T0814, T0806, T0835, T0831, T0888.
**Catalogued but never emitted (8):** T1040, T1071, T1071.001, T1071.004, T1573, T0846, T1692.002, T0885.

These 8 staged IDs are documented in mitre.rs source comments (P3.04 / #89; open item O-04).
They are pre-positioned for future analyzers (DNS tunneling, additional ICS protocol analysis, etc.)
and are intentionally present in the catalog without corresponding emission sites.

## MitreTactic enum (E-27)

16 variants: 14 Enterprise ATT&CK tactics (Reconnaissance through Impact) + 2 ICS-unique
(`IcsInhibitResponseFunction`, `IcsImpairProcessControl`). The enum is `#[non_exhaustive]`
so adding new tactics in a future ATT&CK version is non-breaking for downstream match consumers.

**IcsInhibitResponseFunction (active, reachable via T0814):** `MitreTactic::IcsInhibitResponseFunction`
is declared (mitre.rs:64), appears in `Display` (mitre.rs:85) and in `all_tactics_in_report_order`
(mitre.rs:111). T0814 (Denial of Service — Diagnostics Force Listen Only sub-function) maps to
`IcsInhibitResponseFunction` in `technique_info` and is actively emitted by the Modbus analyzer
(Feature #7). This tactic is therefore reachable via the T0814 emission path.

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
BC-2.10.005: technique_name returns Some for every seeded ID (21 total).
BC-2.10.006: technique_name returns None for unknown IDs.
BC-2.10.007: technique_tactic returns correct tactic for every seeded ID.
BC-2.10.008: all emitted technique IDs resolve in lookup.
BC-2.10.009: MitreTactic is #[non_exhaustive].
Tactic-grouped rendering is in ss-11: BC-2.11.013..017 (TerminalReporter --mitre grouping).
