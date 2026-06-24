# Research: EtherNet/IP + CIP → MITRE ATT&CK for ICS Technique Tagging (OQ-004)

- **Date:** 2026-06-24
- **Type:** general (technology / standards verification)
- **Resolves:** OQ-004 (F1 delta analysis open question) — CIP "Stop" technique ambiguity
- **Feeds:** ADR-010, EtherNet/IP analyzer Behavioral Contracts (F2), `src/mitre.rs` catalogue
- **Status:** COMPLETE — recommendations decisive; one mapping flagged ambiguous (see §Flagged)

## TL;DR — OQ-004 resolution

**CIP "Stop" is `T0858 Change Operating Mode`, NOT `T0814 Denial of Service` and NOT `T0857`.**
This is verified against the live (v19.1) `T0858` page, whose description explicitly
covers halting a PLC user program and cites Triton, PLC-Blaster, and INCONTROLLER —
all of which perform exactly the run→stop transition a CIP Stop service issues.
`T0814` (traffic-level availability degradation) is a less precise fit; `T0857` is
firmware-only AND has been revoked (see below).

## Current ATT&CK / ICS matrix version

- **Current version: ATT&CK v19.1**, released **2026-04-28** (v19.0 April 2026, v19.1 minor follow-up).
  Source: https://attack.mitre.org/resources/updates/ (live, confirmed 2026-06-24).
- ICS domain under v19: 12 tactics, 79 techniques, 18 sub-techniques.
- **Project convention is already pinned** to `ics-attack-19.1` (see
  `.factory/research/attack-ics-version-pin.md`). Cite this string in ADR-010 and the BCs.
  No version bump is required by this research.

## Critical v19 revocations affecting these mappings

These three older ICS IDs that appear in the OQ-004 question are **REVOKED** (not merely
deprecated) under ATT&CK v19 and MUST NOT be emitted by the EtherNet/IP analyzer:

| Revoked ID (old) | Replaced by (current) | Verified |
|------------------|------------------------|----------|
| T0855 Unauthorized Command Message | **T1692.001** Unauthorized Message: Command Message | live T1692.001 page (mod. 2026-05-12) + updates page |
| T0856 Spoof Reporting Message | **T1692.002** Unauthorized Message: Reporting Message | updates page |
| T0857 System Firmware | **T1693.001** Modify Firmware: System Firmware | live T1693.001 page (mod. 2026-05-12) + updates page |

The wirerust codebase already encodes the T0855→T1692.001 and T0856→T1692.002 remap
(`src/mitre.rs`, issue #222). T1693.001 is **NOT yet seeded** in the catalogue — the
EtherNet/IP analyzer PR that emits firmware-update findings will need to add it (see §Catalogue impact).

## Recommended technique mappings

| Behavior | Recommended Technique ID | Technique Name | Tactic (ICS) | Confidence | Source URL |
|----------|--------------------------|----------------|--------------|------------|------------|
| 1. CIP "Stop" / device-stop service (halt PLC user program) | **T0858** | Change Operating Mode | Execution / Evasion | **High** | https://attack.mitre.org/techniques/T0858/ |
| 2. CIP "Reset" / device restart service | **T0816** | Device Restart/Shutdown | Inhibit Response Function | **High** | https://attack.mitre.org/techniques/T0816/ |
| 3. CIP firmware update / flash download | **T1693.001** (system) / T1693.002 (module) | Modify Firmware: System/Module Firmware | Persistence / Inhibit Response Function / Impair Process Control | **High** | https://attack.mitre.org/techniques/T1693/001/ |
| 4a. CIP identity/attribute read (single device profiling) | **T0888** | Remote System Information Discovery | Discovery (ICS) | **High** | https://attack.mitre.org/techniques/T0888/ |
| 4b. CIP ListIdentity (network-wide enumeration) | **T0846** | Remote System Discovery | Discovery (ICS) | **High** | https://attack.mitre.org/techniques/T0846/ |
| 5. CIP write / SetAttribute (parameter modification) | **T0836** (primary) + T1692.001 (if unauthorized/rogue) | Modify Parameter / Unauthorized Message: Command Message | Impair Process Control / Evasion | **High** (T0836) / Medium (overlap) | https://attack.mitre.org/techniques/T0836/ |
| 6. EtherNet/IP UDP/2222 implicit I/O abuse — output injection | **T1692.001** | Unauthorized Message: Command Message | Evasion / Impair Process Control | **High** | https://attack.mitre.org/techniques/T1692/001/ |
| 6. EtherNet/IP UDP/2222 implicit I/O abuse — input spoofing | **T1692.002** | Unauthorized Message: Reporting Message | Evasion / Impair Process Control | High | https://attack.mitre.org/techniques/T1692/002/ |
| 6. Implicit I/O in-line modification (positioned attacker) | T0830 | Adversary-in-the-Middle | Collection (ICS) | Medium | https://attack.mitre.org/techniques/T0830/ |
| 7. CIP ForwardOpen connection establishment anomaly | **T1692.001** (best available) | Unauthorized Message: Command Message | Evasion / Impair Process Control | **Low–Medium (ambiguous)** | https://attack.mitre.org/techniques/T1692/001/ |

### Per-question detail

**1. CIP "Stop" → T0858 (resolves OQ-004).** Verified live: the T0858 description explicitly
addresses halting/stopping PLC programs (a controller run→stop transition) and cites Triton
("ability to halt or run a program through the TriStation protocol"), PLC-Blaster ("stops
execution of the user program on the target"), and INCONTROLLER. T0814 Denial of Service is
keyed to traffic-pattern / resource-exhaustion availability loss, which a targeted CIP Stop
is not. T0857 is firmware-only and revoked. **Single most accurate ID: T0858.** Tactics on the
live page are Execution (TA0104) and Evasion (TA0103); note the project catalogue currently
maps the conceptually-equivalent run-state techniques under `IcsImpairProcessControl` — see
§Catalogue impact for the tactic decision the analyzer PR must make.

**2. CIP "Reset" → T0816 confirmed.** T0816 explicitly models adversary-triggered device
restart/shutdown via automation/management protocols — a direct match for a CIP Reset service.
Under Inhibit Response Function. (Per-page tactic confirmed via deep-research citation; not
independently re-fetched but unambiguous and consistent across sources.)

**3. CIP firmware update → T1693.001 (system) / T1693.002 (module), NOT T0857, NOT T0858.**
T0857 System Firmware is **revoked** → replaced by T1693.001 (verified live, mod. 2026-05-12).
The T1693.001 page explicitly calls for monitoring ICS management/file-transfer protocols for
firmware-change functions — exactly a CIP firmware download. T0858 Change Operating Mode is an
*adjunct* (an attacker may Stop the controller to flash it) but does not represent the firmware
content change itself. Use T1693.001 for controller/system firmware; T1693.002 for I/O or
comms-module firmware.

**4. Identity enumeration — split by scope.** T0888 (Remote System *Information* Discovery) for
device-specific Identity-Object attribute reads (vendor/product/revision/serial profiling).
T0846 (Remote System Discovery) for network-wide ListIdentity broadcast/multicast enumeration
that returns a *list of systems by IP/identifier*. Both under ICS Discovery. wirerust already
emits T0888 (Modbus recon) and seeds T0846 — no new catalogue entry needed.

**5. CIP SetAttribute/write → T0836 primary.** T0836 Modify Parameter is the process-level
behavior (setpoints, thresholds, tuning constants) and the single most accurate primary
technique. T1692.001 Command Message is a *secondary/co-tag* applicable only when the write is
delivered by an unauthorized/rogue master. The old T0855 the question references is revoked →
T1692.001. Recommendation: emit T0836 as primary; co-tag T1692.001 only when the analyzer has
positive evidence the message source is unauthorized (wirerust's multi-technique attribution,
ADR-0006, supports this).

**6. EtherNet/IP UDP/2222 implicit I/O abuse → T1692.001 / T1692.002.** There is **no dedicated
"I/O image manipulation" ICS technique**. (Note: the project catalogue seeds T0835 "Manipulate
I/O Image" and T0806 "Brute Force I/O" for Modbus — these remain valid v19 IDs and are the
closest *process-impact* framing, but they describe manipulating the controller's internal I/O
image via control logic, not spoofing EtherNet/IP cyclic frames on the wire.) For wire-level
implicit-messaging abuse the accurate IDs are: unauthorized **output** injection = T1692.001;
spoofed **input** telemetry = T1692.002; in-line modification by a positioned attacker = T0830.
T0856 (the old "Spoof Reporting Message" the question mentions) is revoked → T1692.002.

**7. CIP ForwardOpen anomaly → no dedicated technique (FLAGGED, see below).**

## Flagged: genuinely ambiguous mappings

- **(7) CIP ForwardOpen connection-establishment anomaly — AMBIGUOUS.** ATT&CK v19 has **no
  technique that names CIP connections or ForwardOpen**, and no technique focused purely on OT
  connection anomalies. The best available mapping depends on what the ForwardOpen is *used for*:
  - As a carrier for unauthorized commands from a rogue master → **T1692.001** (the T1692.001
    detection guidance explicitly cites "new or unexpected connections to controllers ... via
    rogue masters").
  - As reconnaissance → T0846 / T0888.
  - As remote-access/lateral-movement → T0886 Remote Services (IT-framed; weak fit for OT CIP).
  - From an internet-exposed device → T0883 Internet Accessible Device (Initial Access).
  **Recommendation:** if the analyzer emits a finding for an anomalous ForwardOpen *in isolation*
  (no command payload yet observed), prefer NOT tagging a process-impact technique. Tag T1692.001
  only when the connection demonstrably carries an unauthorized command. Document this gap in
  ADR-010 — do not invent a technique. Confidence: Low–Medium.

- **(5) T0836 vs T1692.001 overlap — partially ambiguous.** Both can apply to the same SetAttribute
  write. Resolved by treating T0836 as the process-level primary and T1692.001 as an
  authorization-conditional co-tag (Medium confidence on the co-tag heuristic).

## Catalogue impact (`src/mitre.rs`) — for the analyzer PR, not this research

This research does **not** modify code. The EtherNet/IP analyzer PR will need to seed the
following **new** IDs not currently in the `technique_info` catalogue (current seeded count 25):

| New ID to seed | Name | Suggested tactic variant | Notes |
|----------------|------|--------------------------|-------|
| **T0858** | Change Operating Mode | decision needed (live page: Execution/Evasion; project precedent leans `IcsImpairProcessControl`) | required for CIP Stop |
| **T0816** | Device Restart/Shutdown | `IcsInhibitResponseFunction` (TA0107) | required for CIP Reset |
| **T1693.001** | Modify Firmware: System Firmware | `IcsInhibitResponseFunction` or new Persistence-ICS variant | required for CIP firmware; T0857 is revoked — do NOT seed T0857 |
| **T1693.002** | Modify Firmware: Module Firmware | (same) | optional, module firmware |
| **T1692.002** | Unauthorized Message: Reporting Message | `IcsImpairProcessControl` | already seeded — reuse |

Already-seeded and reusable: T0846, T0888, T0836, T1692.001, T1692.002, T0830, T0835, T0806.

**Tactic note for T0858 / T1693.001:** the live v19 pages list ICS-specific tactic IDs that the
project's `MitreTactic` enum may not yet model in the exact pairing MITRE uses (e.g., T0858 →
Execution TA0104 / Evasion TA0103; T1693.001 → Persistence / Inhibit Response Function / Impair
Process Control). The `mitre.rs` catalogue assigns ONE tactic per ID. The analyzer PR must pick
the single authoritative ICS tactic per the project's existing convention (cf. the F5
`f5-ics-technique-tactic-authoritative.md` decision referenced in `mitre.rs`) and may need a new
`MitreTactic` variant (e.g. an ICS Execution / ICS Persistence / ICS Evasion variant) — this is a
VP-007 atomic-obligation consideration, flagged here for the spec-evolution (F2) work.

## Source verification summary

All seven behaviors mapped to current (v19.1) IDs. Independently re-fetched and verified live
against attack.mitre.org on 2026-06-24: T0858 (current, halts-program + Triton/PLC-Blaster
confirmed), T1692.001 (current, mod. 2026-05-12), T1693.001 (current, mod. 2026-05-12), updates
page (v19.1 / 2026-04-28). Revocations T0855→T1692.001, T0856→T1692.002, T0857→T1693.001 confirmed
via the ATT&CK updates page and consistent with prior project research (`attack-ics-version-pin.md`,
issue #222). T0816, T0846, T0888, T0836, T0830, T0883, T0886 mappings sourced from the
deep-research sweep with per-technique attack.mitre.org citations; not all independently re-fetched
but all are stable, non-revoked v19 IDs already corroborated by the project's existing catalogue.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Full EtherNet/IP+CIP → ICS technique sweep (high reasoning_effort), all 7 behaviors + version + deprecations, with per-technique attack.mitre.org citations |
| Perplexity perplexity_ask | 1 | Confirm revoked-vs-deprecated status + replacement IDs for T0855/T0856/T0857 |
| WebFetch | 5 | Live-verify T0858 (OQ-004 core), T1692.001, T1693.001, updates page (version), + T0857/T0855 (returned empty = consistent with revocation) |
| Read | 3 | `src/mitre.rs` catalogue, prior `attack-ics-version-pin.md`, research dir listing |
| Training data | 1 area | EtherNet/IP CIP service semantics (ForwardOpen, ListIdentity, UDP/2222 implicit messaging) — protocol background only; all ATT&CK mappings web-verified |

**Total MCP tool calls:** 2 (1 perplexity_research + 1 perplexity_ask). Plus 5 WebFetch live verifications.
**Training data reliance:** low — every recommended technique ID was either independently
re-fetched from attack.mitre.org (2026-06-24) or cross-corroborated by both the deep-research
sweep and the project's existing pinned catalogue. Protocol semantics are model knowledge but the
ATT&CK attribution (the load-bearing claim) is fully web-grounded.
