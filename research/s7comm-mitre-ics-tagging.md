# Research: Siemens S7comm → MITRE ATT&CK for ICS Technique Tagging (validation brief)

- **Date:** 2026-09-06
- **Type:** general (technology / external-standard verification)
- **Scope:** Validate a MAXIMAL candidate set of ATT&CK-for-ICS techniques for the planned
  wirerust **S7comm** analyzer. wirerust is a **PASSIVE, offline pcap forensics analyzer** —
  a technique is only emittable when it is defensibly detectable from **S7comm PDU bytes
  observed on the wire** (function code / ROSCTR / parameter fields). No host artifacts, no
  active probing.
- **Feeds:** the S7comm analyzer Behavioral Contracts, the S7comm ADR, and `src/mitre.rs`
  catalogue seeding (VP-007 atomic obligation).
- **Precedent format:** `.factory/research/enip-mitre-ics-tagging.md` (OQ-004).
- **Status:** COMPLETE — decisive per-technique recommendations; two exclusions justified;
  one version-currency flag raised (see §Version pin).
- **Constraint:** research only — no code modified.

---

## TL;DR

- **Seed 3 NEW IDs this cycle:** **T0843** Program Download, **T0889** Modify Program,
  **T0821** Modify Controller Tasking. All three are CURRENT (not revoked) in the latest
  ics-attack release and are defensibly evidenced by classic S7comm block-transfer PDUs.
- **Reuse 8 already-seeded IDs** (add S7comm emission call-sites, no new catalogue entry):
  **T0835** Manipulate I/O Image, **T0836** Modify Parameter, **T0858** Change Operating Mode,
  **T0816** Device Restart/Shutdown, **T0888** Remote System Information Discovery,
  **T0846** Remote System Discovery, **T0814** Denial of Service, **T1692.001** Unauthorized
  Message: Command Message (the successor to revoked T0855).
  - Confirmed: **T0835 and T0836 are already seeded AND already emitted** (Modbus) in
    `src/mitre.rs` — see §Already-seeded confirmation.
- **EXCLUDE 2 (with justification):** **T0851** Rootkit and **T0873 / T0873.001** Project
  File Infection — both are CURRENT/active techniques but are host/file-artifact behaviors
  **not network-observable** from S7comm PDUs. See §Exclusions.
- **DEFER (optional, low confidence):** **T0813** Denial of Control — only indirectly
  inferable; recommend NOT seeding this cycle. See §Deferred.
- **Version pin:** the codebase pins `ics-attack-19.1`. The current release is
  **`ics-attack-v19.2`** (released **2026-08-06**), which made **no ICS technique-catalog
  changes**. All mappings below are valid under both. See §Version pin for the pin-currency
  recommendation.

---

## Version pin — confirm and currency flag

| Fact | Value | Source |
|------|-------|--------|
| Codebase pin today | `ics-attack-19.1` (`src/reporter/json.rs:29`, ADR-0006, ADR-0010) | codebase read 2026-09-06 |
| ADR-013 reference | `ics-attack-19.1` (per the task brief) | codebase |
| **Current released version (Sep 2026)** | **`ics-attack-v19.2`**, released **2026-08-06** | attack.mitre.org/resources/updates/ , /resources/versions/ |
| v19.2 nature | **Agile minor** — updated **Groups + Software for Enterprise**; **no ICS technique revocations, deprecations, renames, or new ICS techniques/sub-techniques** vs v19.1 | attack.mitre.org/resources/updates/ |

**Interpretation.** Every S7comm mapping in this brief is technique-level and is unchanged
between v19.1 and v19.2 — the S7comm seeding decision is **safe regardless of which of the two
minors the project pins**. However, the report envelope currently advertises `ics-attack-19.1`
while `v19.2` is the live release. Two options for the analyzer/spec PR (decision belongs to
the human, not this research):

1. **Keep `ics-attack-19.1`** — technically defensible because no ICS technique the project
   emits changed in v19.2; the pin still names a real, valid release. Lowest churn.
2. **Bump to `ics-attack-19.2`** (canonical STIX bundle would be `ics-attack-19.2.json`) —
   most current; matches the live release. Recommended if the project wants the envelope to
   name the latest release. Because v19.2 touched no ICS techniques, the bump is a pure
   currency update with zero technique-ID impact.

> **Flag:** re-confirm at S7comm-analyzer PR time whether a v19.3/v20 has shipped (ATT&CK
> cadence is ~biannual majors with intermittent Agile minors). As of 2026-09-06 the latest is
> v19.2 (2026-08-06). Confidence: high (two independent attack.mitre.org pages).

---

## Revocation diligence (T0855 / T0856 / T0857 / T0809 lesson applied)

Per the project's standing revocation-lesson discipline (do **not** seed a revoked ID), each
candidate was checked against the v19 ICS Revocations list:

| Old ID | v19 status | Successor | Relevance to S7comm brief |
|--------|-----------|-----------|---------------------------|
| T0855 Unauthorized Command Message | **REVOKED** | **T1692.001** Unauthorized Message: Command Message | Use T1692.001 (already seeded/emitted). Do NOT seed T0855. |
| T0856 Spoof Reporting Message | **REVOKED** | T1692.002 Unauthorized Message: Reporting Message | Not proposed for S7comm (already seeded, catalogue-only). |
| T0857 System Firmware | **REVOKED** | T1693.001 Modify Firmware: System Firmware | Not proposed for S7comm this cycle. T1693.001 already seeded (staged). Do NOT seed T0857. |
| T0809 Data Destruction | **STILL CURRENT** (not revoked — common misconception) | T0809 (unchanged) | Not passively evidenced by S7comm; not proposed. |

All three NEW candidates (T0843, T0889, T0821) and all reuse candidates were verified
**present and active** (not on any v19/v19.2 revocation or deprecation list). Notably, several
gained sub-techniques in v19 but the **parent IDs remain valid**:

- **T0843** gained `.001 Download All`, `.002 Online Edit`, `.003 Program Append` — parent T0843 still valid.
- **T0846** gained `.001 Port Scan`, `.002 Broadcast Discovery`, `.003 Multicast Discovery` — parent T0846 still valid.
- **T0873** gained `.001` (Siemens Project File Format) — parent T0873 still valid (still excluded, see §Exclusions).

---

## S7comm wire-field basis (passive detection primitives)

Classic S7comm framing: `TPKT → COTP (ISO-on-TCP / RFC 1006) → S7comm` on **TCP/102**. S7comm
protocol id = `0x32`; header carries **ROSCTR**, PDU reference, parameter length, data length.

**ROSCTR (message type):** `0x01` Job (request) · `0x02` Ack · `0x03` Ack_Data (response) ·
`0x07` Userdata (extended: SZL, block enum, diagnostics, security, time).

**Job / Ack_Data function codes** (Wireshark `packet-s7comm.c`, gmiru, snap7 constants):

| FC | Operation | Passive relevance |
|----|-----------|-------------------|
| `0xF0` | Setup Communication | session negotiation; scan/flood context |
| `0x04` | Read Var | read of I/O/DB/markers; monitoring/recon context |
| `0x05` | Write Var | **primary write indicator** (I/O, markers, DB, timers, counters) |
| `0x1A` | Request Download | **start of block download → PLC** |
| `0x1B` | Download Block | download chunk |
| `0x1C` | Download Ended | download terminate (complete `0x1A→0x1B→0x1C` = strong signal) |
| `0x1D` | Start Upload | block upload PLC→station (backup/collection, **not** program download) |
| `0x1E` | Upload | upload chunk |
| `0x1F` | End Upload | upload terminate |
| `0x28` | PLC Control / PI-Service | decode service string: `P_PROGRAM` (start/state), `_INSE` (activate block), `_DELE` (delete block), `_GARB` (memory compress), `_MODU` (RAM→ROM) |
| `0x29` | PLC Stop | dedicated STOP request |

**Userdata (ROSCTR `0x07`) groups / subfunctions** — *corrected against the Wireshark
dissector* (my initial query mis-stated block-group as 0x07):

| Group | Meaning | Key subfunctions |
|-------|---------|------------------|
| `0x03` Block functions | block enumeration | `0x01` List blocks · `0x02` List blocks of type · `0x03` Get block info |
| `0x04` CPU functions | CPU services | `0x01` **Read SZL** (System Status List), plus diagnostics/alarms |
| `0x07` Time functions | clock read/set | (not block functions — common doc error) |

**S7ANY area codes** (Write/Read Var target): `0x80` direct peripheral · `0x81` inputs (I/PE) ·
`0x82` outputs (Q/PA) · `0x83` markers (M) · `0x84` data block (DB) · `0x85` instance DB (DI) ·
`0x1C` counters · `0x1D` timers.

Sources: Wireshark S7comm dissector (`packet-s7comm.c` / `.h`), gmiru S7comm articles pt.1/2,
snap7/sharp7 constants, labshocksecurity/inprotech S7comm anatomy write-ups.

> **S7comm-plus / TLS caveat.** The classic `0x32` function-code table does **not** apply to
> S7comm-plus (S7-1200/1500, TIA Portal) — a separate proprietary object/service protocol with
> per-packet integrity material, and increasingly **TLS-wrapped** (secure PG/HMI: S7-1500 fw
> 2.9+, S7-1200 fw 4.5+). Under TLS an offline analyzer without keys sees only endpoints /
> sizes / timing — it **cannot** recover download/write/START-STOP/SZL semantics. The analyzer
> must classify each flow as (1) classic S7comm/0x32, (2) legacy-parseable S7comm-plus,
> (3) TLS-protected, or (4) unknown, and only emit technique tags for (1) [and, if a parser is
> built, legacy (2)]. All mappings below assume **classic S7comm/0x32 plaintext**.

---

## Per-technique validation table

Legend — passive-confidence: **Direct** = the S7 operation itself is identifiable from PDU
fields; **Conditional** = operation is visible but the ATT&CK claim (malicious/unauthorized/
task-change/impact) needs baseline, payload decode, success-response, or temporal context;
**None** = not established by S7comm PDUs.

| ID | Name | Current? (v19.2) | Already seeded? | S7comm detection pattern | Passive-confidence | Recommendation |
|----|------|:----------------:|:---------------:|--------------------------|--------------------|----------------|
| **T0843** | Program Download | ✅ current (parent; +sub .001/.002/.003) | ❌ NEW | Complete block-download to PLC: `Job` FC `0x1A` Request Download → `0x1B` Download Block → `0x1C` Download Ended; optional `0x28 _INSE` activate. Correlate request+success response. | **Direct** (transfer); Conditional (subtype/intent) | **SEED + EMIT** |
| **T0889** | Modify Program | ✅ current | ❌ NEW | Same `0x1A→0x1C` download, or `0x28 _INSE`/`_DELE` block activate/delete — proves a program/block operation. Malicious-change claim needs block decode / baseline. | **Conditional (strong)** | **SEED + EMIT** (co-tag with T0843) |
| **T0821** | Modify Controller Tasking | ✅ current | ❌ NEW | Program-download traffic involving organization blocks (e.g. OB1 → cyclic execution). FC alone can't prove tasking change; needs transferred-block-type ID. | **Conditional** | **SEED + EMIT** (low-confidence co-tag; guard on block-type when decodable) |
| **T0835** | Manipulate I/O Image | ✅ current | ✅ **seeded + emitted (Modbus)** | `Write Var 0x05` to area `0x81` inputs / `0x82` outputs / `0x80` direct peripheral (force/override). | **Conditional** | **REUSE — add S7comm emit call-site** |
| **T0836** | Modify Parameter | ✅ current | ✅ **seeded + emitted (Modbus)** | `Write Var 0x05` to `DB 0x84` / marker `0x83` at a configured parameter address (needs address engineering-meaning). | **Conditional** | **REUSE — add S7comm emit call-site** |
| **T0858** | Change Operating Mode | ✅ current | ✅ seeded + emitted (ENIP) | `0x29 PLC Stop` (run→stop); `0x28` PI-Service `P_PROGRAM` (start/run/state). Match Job+response; strongest with CPU-status SZL follow-up. | **Direct** (requested command) | **REUSE — add S7comm emit call-site** |
| **T0816** | Device Restart/Shutdown | ✅ current | ✅ seeded + emitted (ENIP) | `0x28` PI-Service restart/reset ops (hot/cold start) — decode the service string, do NOT match bare `0x28`. Network disappearance = corroboration. | **Conditional→Direct** (on decoded restart op) | **REUSE — add S7comm emit call-site** |
| **T0888** | Remote System Information Discovery | ✅ current | ✅ seeded + emitted (Modbus) | `ROSCTR 0x07` Userdata: CPU group `0x04` subfn `0x01` **Read SZL**; Block group `0x03` subfn `0x01/0x02/0x03` list/get-block-info. | **Direct** (discovery op) | **REUSE — add S7comm emit call-site** |
| **T0846** | Remote System Discovery | ✅ current (parent; +sub .001/.002/.003) | ✅ seeded + emitted (ENIP) | Not from a single S7 session. TCP SYN sweep across many hosts on **:102**, or repeated COTP/`Setup 0xF0` to many addresses → `.001 Port Scan` more specific. | **Conditional** (from wider pcap, not one PDU) | **REUSE — emit only on multi-host sweep evidence** |
| **T0814** | Denial of Service | ✅ current | ✅ seeded + emitted (ENIP/Modbus/DNP3) | Connection flood, excessive `0xF0 Setup`, malformed-length storms, sustained no-response — windowed burst, not single frame. | **Conditional** | **REUSE — emit on burst/malformed thresholds only** |
| **T1692.001** | Unauthorized Message: Command Message (successor to revoked T0855) | ✅ current | ✅ seeded + emitted | Any S7 command (write / download / STOP / START / delete-block) from a source outside an allowlist / maintenance window. Bytes alone don't prove "unauthorized". | **Conditional** (needs policy context) | **REUSE — co-tag only with positive unauthorized-source evidence** |
| **T0813** | Denial of Control | ✅ current | ❌ (not seeded) | Only indirect: connection resets, repeated failed jobs, absence of expected responses. A `0x29 STOP` maps better to T0858, not T0813. | **Conditional / indirect (low)** | **DEFER — do not seed this cycle** (see §Deferred) |
| **T0851** | Rootkit | ✅ current | n/a | Block/firmware transfer is suspicious but does not demonstrate hiding/interception. Only content-signature matching could, which is not a ROSCTR/FC-field conclusion. | **None** (host/artifact) | **EXCLUDE** (see §Exclusions) |
| **T0873** | Project File Infection | ✅ current (parent) | n/a | Infection lives in a project file on an engineering workstation. A later download shows deployment, not source-file infection. | **None** (host/file) | **EXCLUDE** |
| **T0873.001** | Project File Infection: Siemens Project File Format | ✅ current (new v19 sub) | n/a | Same limitation — STEP 7/WinCC/TIA project file at rest; needs file hashes / filesystem / engineering-tool telemetry. | **None** (host/file) | **EXCLUDE** |

---

## Already-seeded confirmation (`src/mitre.rs`, read 2026-09-06)

Confirmed against the `technique_info` match and `SEEDED_TECHNIQUE_IDS` array:

- **T0835 "Manipulate I/O Image"** — seeded (`IcsImpairProcessControl`) **and** in `EMITTED_IDS`
  (Modbus). ✅ believed-seeded confirmed.
- **T0836 "Modify Parameter"** — seeded (`IcsImpairProcessControl`) **and** in `EMITTED_IDS`
  (Modbus). ✅ believed-seeded confirmed.
- Also already seeded and reusable for S7comm: **T0858** (`IcsExecution`), **T0816**
  (`IcsInhibitResponseFunction`), **T0888** (`IcsDiscovery`), **T0846** (`IcsDiscovery`),
  **T0814** (`IcsInhibitResponseFunction`), **T1692.001** (`IcsImpairProcessControl`).
- Current `SEEDED_TECHNIQUE_ID_COUNT = 29`. No S7comm analyzer exists yet — `src/protocols.rs`
  only carries S7comm/S7comm-plus **catalogue** entries (port-102 collision), no PDU parser
  and no `mitre_techniques: vec!` emission.

**S7comm-specific emission call-sites for the reuse IDs** (for the analyzer PR, not this
research): T0835 ← `Write Var 0x05` to `0x80/0x81/0x82`; T0836 ← `Write Var 0x05` to
`0x84/0x83`; T0858 ← `0x29 STOP` / `0x28 P_PROGRAM` start; T0816 ← `0x28` restart PI-service;
T0888 ← `0x07/0x04/0x01 Read SZL` and `0x07/0x03/*` block-list; T0846 ← multi-host :102 sweep;
T0814 ← flood/malformed burst; T1692.001 ← any command from unauthorized source.

---

## Exclusions (justified)

- **T0851 Rootkit — EXCLUDE.** Active in v19.2, but a rootkit is a host-resident concealment
  behavior (hiding programs/files/services, intercepting APIs). An S7comm block or firmware
  transfer on the wire is at most a *carrier*; the ROSCTR/FC/subfunction fields never evidence
  concealment. Only deep payload-signature matching (not a passive protocol-field conclusion,
  and out of a metadata forensics analyzer's scope) could touch this. **Not network-observable.**
- **T0873 / T0873.001 Project File Infection — EXCLUDE.** Active in v19.2 (v19 added the
  `.001 Siemens Project File Format` sub-technique). The infection occurs in a STEP 7 / WinCC /
  TIA **project file at rest on an engineering workstation**. A subsequent `0x1A→0x1C` download
  can show that project-derived code was deployed, but **cannot** show the source project file
  was infected — that requires file hashes, project archives, filesystem events, or
  engineering-tool telemetry, none of which are on the S7comm wire. **Not network-observable.**
  (If the analyzer ever detects a *download of a block whose contents match a known-malicious
  signature*, the accurate tag is T0843/T0889 on the transfer — not T0873 on the file.)

## Deferred (optional, low confidence)

- **T0813 Denial of Control — DEFER (do not seed this cycle).** Active and valid, but only
  *indirectly* inferable from passive S7comm (connection resets, repeated failed jobs, absence
  of expected control responses) and always ambiguous vs. benign faults. A `0x29 PLC Stop` is
  better attributed to **T0858 Change Operating Mode** (and possibly T0814), not automatically
  T0813. Seeding it now would create an ID with no clean, defensible emission predicate.
  Revisit only if the analyzer later grows a robust control-loss temporal heuristic.

---

## Seeding decision for this cycle

**Seed 3 NEW catalogue entries** in `src/mitre.rs` (bringing `SEEDED_TECHNIQUE_ID_COUNT`
`29 → 32`), each a VP-007 atomic obligation (seed + emit branch + `SEEDED_TECHNIQUE_IDS` +
count, in lockstep):

| New ID | Name | Likely ICS tactic (VERIFY at PR time) | Enum impact |
|--------|------|----------------------------------------|-------------|
| **T0843** | Program Download | Lateral Movement (ICS) — *per live page, confirm* | may need a **new `MitreTactic` ICS Lateral Movement variant (TA0109)** |
| **T0889** | Modify Program | Persistence (ICS) — *per live page, confirm* | may need a **new `MitreTactic` ICS Persistence variant (TA0110)** |
| **T0821** | Modify Controller Tasking | Execution (ICS) → maps to existing `IcsExecution` (TA0104) — *confirm* | likely reuses existing variant |

> **Tactic caveat (flagged unverifiable-without-fetch):** `src/mitre.rs` assigns exactly ONE
> tactic per ID, and the exact single authoritative ICS tactic for T0843/T0889 was not
> independently re-fetched from the live technique pages in this pass (the deep-research sweep
> confirmed the IDs are *current* and detection-relevant, but per-page tactic pairings for
> T0843/T0889/T0821 should be re-verified against attack.mitre.org before the enum variant is
> chosen — same discipline the ENIP brief applied for T0858/T1693.001). If T0843 (Lateral
> Movement) or T0889 (Persistence) require ICS tactics the `MitreTactic` enum does not yet
> model, a new variant + `tactic_id()` + `technique_tactic_id()` arm must be added atomically
> (VP-007). This is the one item in this brief that needs a live-page confirmation step.

**Reuse (no new catalogue entry, add S7comm emission call-sites):** T0835, T0836, T0858,
T0816, T0888, T0846, T0814, T1692.001.

**Exclude:** T0851, T0873, T0873.001. **Defer:** T0813.

**Version pin:** mappings valid under `ics-attack-19.1` (current codebase pin) and
`ics-attack-19.2` (live release, no ICS changes). Recommend the S7comm PR consider bumping the
envelope to **`ics-attack-19.2`** as a currency update, but this is orthogonal to the S7comm
seeding and carries zero technique-ID risk.

---

## Flagged / unverifiable

1. **Per-page tactic pairing for T0843 / T0889 / T0821** — IDs confirmed current and
   detection-relevant via deep research; the single authoritative ICS tactic per ID was not
   independently re-fetched. Verify against the live technique pages before choosing the
   `MitreTactic` variant. (See tactic caveat above.)
2. **v19.3/v20 currency** — latest confirmed release is v19.2 (2026-08-06). Re-confirm at PR
   time that no newer release changed the ICS catalogue.
3. **S7comm-plus / TLS flows** — no defensible technique emission from encrypted or
   unparsed-proprietary flows; all mappings assume classic `0x32` plaintext (documented above).
4. **`0x28` PI-Service ambiguity** — `0x28` multiplexes start / activate / delete / compress /
   RAM→ROM by service string; the analyzer MUST decode the service name before mapping (bare
   `0x28` is not sufficient for T0858 vs T0816 vs T0889).

---

## Sources (MITRE primary — cite in the ADR/BCs)

| Technique | URL |
|-----------|-----|
| T0843 Program Download (+ .001/.002/.003) | https://attack.mitre.org/techniques/T0843/ |
| T0889 Modify Program | https://attack.mitre.org/techniques/T0889/ |
| T0821 Modify Controller Tasking | https://attack.mitre.org/techniques/T0821/ |
| T0835 Manipulate I/O Image | https://attack.mitre.org/techniques/T0835/ |
| T0836 Modify Parameter | https://attack.mitre.org/techniques/T0836/ |
| T0851 Rootkit | https://attack.mitre.org/techniques/T0851/ |
| T0873 / .001 Project File Infection | https://attack.mitre.org/techniques/T0873/ · https://attack.mitre.org/techniques/T0873/001/ |
| T0888 Remote System Information Discovery | https://attack.mitre.org/techniques/T0888/ |
| T0846 Remote System Discovery (+ .001) | https://attack.mitre.org/techniques/T0846/ · https://attack.mitre.org/techniques/T0846/001/ |
| T0858 Change Operating Mode | https://attack.mitre.org/techniques/T0858/ |
| T0816 Device Restart/Shutdown | https://attack.mitre.org/techniques/T0816/ |
| T0813 Denial of Control | https://attack.mitre.org/techniques/T0813/ |
| T0814 Denial of Service | https://attack.mitre.org/techniques/T0814/ |
| T1692.001 Unauthorized Message: Command Message | https://attack.mitre.org/techniques/T1692/001/ |
| T0809 Data Destruction (still current) | https://attack.mitre.org/techniques/T0809/ |
| v19 April-2026 update / ICS revocations | https://attack.mitre.org/resources/updates/updates-april-2026/ |
| ATT&CK versions (v19.2 current) | https://attack.mitre.org/resources/versions/ · https://attack.mitre.org/resources/updates/ |

**S7comm protocol structure:** Wireshark S7comm dissector `packet-s7comm.c` / `.h`
(fossies.org / wireshark.org), wiki.wireshark.org/S7comm, gmiru.com S7comm articles pt.1 & pt.2,
gmiru.com/resources/s7proto/constants.txt, dokuwiki.hampel-soft.com S7 constants, snap7/sharp7
constants, labshocksecurity.com & inprotech.es S7comm anatomy write-ups, NSFOCUS Read-SZL
analysis. S7comm-plus / TLS: Black Hat EU-17 (Lei) & US-24 (Dankner) S7comm-plus papers,
Siemens TIA secure-communication docs (S7-1500 fw 2.9+, S7-1200 fw 4.5+).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | High-context deep sweep: all 14 candidate techniques × (current/revoked status + passive S7comm-PDU detectability) with S7comm FC/ROSCTR/subfunction/area mappings and per-technique attack.mitre.org citations |
| Perplexity perplexity_ask | 2 | (1) Confirm current ATT&CK version as of Sep-2026 (→ v19.2, 2026-08-06); (2) confirm v19.2 made no ICS technique-catalogue changes and the 10 key IDs remain active |
| Read | 4 | `src/mitre.rs` (seeded/emitted sets, count=29, pin), `src/protocols.rs` (S7comm catalogue entry), `enip-mitre-ics-tagging.md` (format precedent), `attack-ics-version-pin.md` + `mitre-ics-v19-catalog-audit.md` (revocation ground-truth) |
| Grep | 3 | Locate `ics-attack` pin (`src/reporter/json.rs`), S7comm references, confirm no existing S7comm analyzer/emission |
| Training data | 1 area | S7comm/ISO-on-TCP protocol background framing only — all ATT&CK attributions and S7 function-code/subfunction values are web-verified (Wireshark dissector + gmiru + MITRE) |

**Total MCP tool calls:** 3 (1 perplexity_research + 2 perplexity_ask)
**Training data reliance:** low — every load-bearing claim (technique current/revoked status,
current ATT&CK version, S7comm function-code/ROSCTR/userdata-group values incl. the block-group
0x03 correction) is web-grounded to attack.mitre.org and the Wireshark S7comm dissector. The
only residual unverified item is the exact single-tactic pairing for T0843/T0889/T0821, which is
explicitly flagged for a live-page confirmation step at PR time.
