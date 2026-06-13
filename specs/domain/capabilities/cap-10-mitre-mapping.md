---
artifact: L2-cap-10
traces_to: ../domain-spec.md
cap_id: CAP-10
title: MITRE ATT&CK Mapping
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
modified:
  - date: 2026-06-10
    actor: architect
    reason: "Remap revoked ATT&CK-ICS v19 IDs: T0855→T1692.001 (Unauthorized Message: Command Message), T0856→T1692.002 (Unauthorized Message: Reporting Message) (issue #222)."
  - date: 2026-06-10
    actor: architect
    reason: "Fix F2 staleness: catalog expanded to 21 IDs (add Modbus techniques T0836/T0814/T0806/T0835/T0831/T0888); correct emitted count 6→13; correct never-emitted count 9→8 (T1692.001 is emitted, not staged); fix IcsInhibitResponseFunction paragraph (T0814→IcsInhibitResponseFunction is active, not unreachable); update BC-2.10.005 total (issue #222)."
  - date: 2026-06-10
    actor: architect
    reason: "F2 delta (issue #8 DNP3 TCP): catalog expanded to 23 IDs (add T1691.001 Block Operational Technology Message: Command Message → IcsInhibitResponseFunction, T0827 Loss of Control → IcsImpact); emitted count 13→15; add IcsImpact variant to MitreTactic enum (17 variants / 3 ICS-unique); update BC-2.10.005 total."
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-7 remediation F-C-P7-001: add PLANNED forward-declaration for T0830 (LateralMovement, ICS) and T1557.002 (CredentialAccess, Enterprise) — ARP F2 STORY-114; update seeded/emitted counts to current/after-STORY-114 form."
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-9 remediation F-C-P9-001: correct tactic column for T0846 and T0888 from non-existent IcsDiscovery to Discovery — the MitreTactic enum has only 3 ICS-unique variants (IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact); there is no IcsDiscovery variant (confirmed src/mitre.rs and the ## MitreTactic enum (E-27) section (lines 81-85))."
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-10 remediation F-C-P10-002: stale line citation in pass-9 changelog entry corrected from 'lines 76-77' to 'lines 80-82' (subsequently corrected to 'lines 81-85' by Pass-11 F-C-P11-002) — the ## MitreTactic enum (E-27) header is at line 81."
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-11 remediation F-C-P11-002: pass-9/10 changelog line citation for the MitreTactic enum section corrected from 'lines 80-82' to 'lines 81-85' — the ## MitreTactic enum (E-27) header is at line 81 and the variant prose spans lines 83-85."
  - date: 2026-06-13
    actor: product-owner
    reason: "ARP-F2 Pass-14 remediation: CLI --mitre flag section stale 'mitre_technique: Option<String>' prose updated to 'mitre_techniques: Vec<String>' (empty vec → key absent; ADR-006 Decision 13; STORY-100 AC-008). Version bumped 1.7→1.8."
version: "1.8"
---

# CAP-10: MITRE ATT&CK Mapping

## What the system does today

`mitre.rs` (C-16) provides a static lookup table (`technique_info`) mapping MITRE technique
IDs to `(technique_name: &'static str, tactic: MitreTactic)` pairs. The `TerminalReporter`
groups findings by tactic when `--mitre` is set.

**Sources:** C-16 mitre.rs (module-decomposition.md). BC-2.10.001..009.

## Technique catalog

The `technique_info` function contains 23 current IDs in its match arms (21 post-Feature-#7 IDs
plus 2 new DNP3 ICS techniques added in Feature #8: T1691.001, T0827), expanding to 25 total
after STORY-114 (ARP F2) adds T0830 and T1557.002:

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
| T0846 | Remote System Discovery | Discovery (*catalogued, never emitted*) |
| T1692.001 | Unauthorized Message: Command Message | IcsImpairProcessControl |
| T1692.002 | Unauthorized Message: Reporting Message | IcsImpairProcessControl (*catalogued, never emitted*) |
| T0885 | Commonly Used Port | CommandAndControl (*catalogued, never emitted*) |
| T0836 | Modify Parameter | IcsImpairProcessControl |
| T0814 | Denial of Service | IcsInhibitResponseFunction |
| T0806 | Brute Force I/O | IcsImpairProcessControl |
| T0835 | Manipulate I/O Image | IcsImpairProcessControl |
| T0831 | Manipulation of Control | IcsImpairProcessControl |
| T0888 | Remote System Information Discovery | Discovery |
| T1691.001 | Block Operational Technology Message: Command Message | IcsInhibitResponseFunction |
| T0827 | Loss of Control | IcsImpact |
| T0830 | Adversary-in-the-Middle | LateralMovement (*PLANNED STORY-114 (ARP F2); not in develop HEAD until STORY-114*) |
| T1557.002 | Adversary-in-the-Middle: ARP Cache Poisoning | CredentialAccess (*PLANNED STORY-114 (ARP F2); not in develop HEAD until STORY-114*) |

**Emitted (15 current / 17 after STORY-114):** T1027, T1036, T1046, T1083, T1499.002, T1505.003, T1692.001, T0836, T0814, T0806, T0835, T0831, T0888, T1691.001, T0827 (current 15); T0830 and T1557.002 added after STORY-114.
**Catalogued but never emitted (8):** T1040, T1071, T1071.001, T1071.004, T1573, T0846, T1692.002, T0885.

These 8 staged IDs are documented in mitre.rs source comments (P3.04 / #89; open item O-04).
They are pre-positioned for future analyzers (DNS tunneling, additional ICS protocol analysis, etc.)
and are intentionally present in the catalog without corresponding emission sites.

## MitreTactic enum (E-27)

17 variants: 14 Enterprise ATT&CK tactics (Reconnaissance through Impact) + 3 ICS-unique
(`IcsInhibitResponseFunction`, `IcsImpairProcessControl`, `IcsImpact`). The enum is `#[non_exhaustive]`
so adding new tactics in a future ATT&CK version is non-breaking for downstream match consumers.

**IcsInhibitResponseFunction (active, reachable via T0814 and T1691.001):** `MitreTactic::IcsInhibitResponseFunction`
is declared (mitre.rs), appears in `Display` and in `all_tactics_in_report_order`. T0814 (Denial
of Service — Diagnostics Force Listen Only sub-function) maps to `IcsInhibitResponseFunction` and
is actively emitted by the Modbus analyzer (Feature #7). T1691.001 (Block Operational Technology
Message: Command Message) maps to `IcsInhibitResponseFunction` and is emitted by the DNP3 analyzer
(Feature #8). This tactic is reachable via both emission paths.

**IcsImpact (active, reachable via T0827):** `MitreTactic::IcsImpact` (ICS Impact, TA0105) was
added in Feature #8 (issue #8, ADR-007). T0827 (Loss of Control) is a derived/correlated finding
emitted by the DNP3 analyzer when unauthorized command activity is detected. `IcsImpact` appears
in `Display` and `all_tactics_in_report_order`.

## CLI --mitre flag

When `--mitre` is set in `Commands::Analyze`, `TerminalReporter` renders findings grouped by
tactic. Unknown technique IDs (not in the catalog) display as `<id> (unknown)`
(terminal.rs:248-249). `JsonReporter` does not group by tactic; it emits the raw
`mitre_techniques: Vec<String>` field per Finding (key absent from JSON when vec is empty;
ADR-006 Decision 13).

## Unknown-ID handling (VO-6)

If an analyzer emits a malformed or unrecognized MITRE technique ID:
- `technique_name()` returns `None`.
- `technique_tactic()` returns `None`.
- Terminal reporter shows `<id> (unknown)`.
- JSON output contains the raw ID string unmodified.

## BC references

BC-2.10.001..004: MitreTactic Display rendering + all_tactics_in_report_order.
BC-2.10.005: technique_name returns Some for every seeded ID (25 total after STORY-114; 23 current).
BC-2.10.006: technique_name returns None for unknown IDs.
BC-2.10.007: technique_tactic returns correct tactic for every seeded ID.
BC-2.10.008: all emitted technique IDs resolve in lookup.
BC-2.10.009: MitreTactic is #[non_exhaustive].
Tactic-grouped rendering is in ss-11: BC-2.11.013..017 (TerminalReporter --mitre grouping).
