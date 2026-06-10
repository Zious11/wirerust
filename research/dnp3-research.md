# Domain Research: DNP3 (IEEE 1815) Protocol Analyzer for ICS/OT Forensics

**Feature:** wirerust #8 — DNP3 analyzer (TCP-only, port 20000)
**Date:** 2026-06-10
**Purpose:** Implementation-oriented reference feeding the F2 spec evolution (F2 BCs + VP-023
derive from this). Favors exact byte layouts, code values, and concrete detection signals over
prose. Mirrors the structure/rigor of `modbus-tcp-research.md` and the version discipline of
`attack-ics-version-pin.md`.
**Authoritative sources:** IEEE Std 1815-2012 *Standard for Electric Power Systems
Communications — Distributed Network Protocol (DNP3)*; DNP Users Group *DNP3 Primer Rev A*
(`dnp.org`); RACOM DNP3 reference (`racom.eu/download/sw/prot/free/eng/dnp3.pdf`); Chipkin
*DNP3 Quick Reference* and *AN2013-004b Validation of Incoming DNP3 Data*; Wireshark DNP3
dissector reference (`wireshark.org/docs/dfref/d/dnp3.html`); Suricata DNP3 keyword docs;
CISA `icsnpp-dnp3` Zeek analyzer (`github.com/cisagov/icsnpp-dnp3`); MITRE ATT&CK for ICS
(`attack.mitre.org`), pinned version **ics-attack-19.1**.

> **Confidence legend:** [SPEC] = verified against IEEE 1815 / DNP Users Group / multiple
> concurring protocol references; [MITRE] = verified directly against attack.mitre.org primary
> pages or the official v18.1→v19.0 changelog; [JUDGMENT] = detection threshold / heuristic
> that F2 must set explicitly (sources disagree or value is environment-dependent);
> [UNVERIFIED] = could not confirm against an authoritative source — do not rely on without
> checking IEEE 1815 §-level text.

---

## ⚠️ CRITICAL FINDING — Two of the five locked MITRE techniques are REVOKED in ics-attack-19.1

The scope locked techniques **T0803 + T0828 + T0855 + T0814 + T0836**. Verified against the
official MITRE **v18.1→v19.0 detailed changelog** and live technique pages (the project's
pinned version is `ics-attack-19.1`):

| Locked ID | v19.1 status | Resolution |
|-----------|--------------|------------|
| **T0803** Block Command Message | **REVOKED** | Replaced by **T1691.001** *Block Operational Technology Message: Command Message* (tactic unchanged: Inhibit Response Function) |
| **T0855** Unauthorized Command Message | **REVOKED** | Replaced by **T1692.001** *Unauthorized Message: Command Message* (tactics: Evasion + Impair Process Control) |
| **T0828** | **NAME MISMATCH** — T0828 is *Loss of Productivity and Revenue*, **not** "Loss of Control" | The control-loss concept the human intended is **T0827** *Loss of Control* (Impact) |
| **T0814** Denial of Service | **Active** ✓ | No change (Inhibit Response Function) |
| **T0836** Modify Parameter | **Active** ✓ | No change (Impair Process Control) |

**This also means the project's existing `attack-ics-version-pin.md` is STALE on T0855** — it
listed T0855 as "Active" in v19.1, but the v18.1→v19.0 changelog explicitly revokes it. See
§7 for the verbatim changelog evidence and the F2 decision required. **F2 must decide whether
to emit the revoked parent IDs (T0803/T0855) for back-compat or migrate to the v19 sub-technique
IDs (T1691.001/T1692.001), and must replace the T0828 reference with T0827.** Per project
policy `DF-VALIDATION-001`, the stale version-pin entry is a validated finding eligible to be
filed as a GitHub issue.

---

## 1. Data Link Layer Frame Format [SPEC]

DNP3 over TCP (this analyzer: **TCP port 20000**, StreamDispatcher integration) carries the
**identical link-layer frame** used on serial — TCP is just the transport. Every link frame
begins with an 8-octet header (sync through source address) followed by a header CRC, then
0..16 *data blocks* each of ≤16 user octets + a 2-octet CRC.

### 1.1 Header layout (the fixed 8-octet header, before its CRC)

| Offset (in header) | Size | Field | Value / Semantics | Endianness |
|--------------------|------|-------|-------------------|------------|
| 0 | 1 | **START 1** | **`0x05`** | — |
| 1 | 1 | **START 2** | **`0x64`** (together the sync word `0x0564`) | — |
| 2 | 1 | **LENGTH** | Count of octets in the remainder of the frame **excluding** the two start octets, the LENGTH octet itself, **and all CRC octets**. **Includes** CONTROL(1) + DESTINATION(2) + SOURCE(2) + user data. Range **5..255**. | — |
| 3 | 1 | **CONTROL** | Bitfields — see §1.2 | — |
| 4–5 | 2 | **DESTINATION** | Destination link address | **little-endian** (low byte first) |
| 6–7 | 2 | **SOURCE** | Source link address | **little-endian** (low byte first) |
| (8–9) | 2 | **Header CRC** | CRC-16/DNP over header octets 0–7 (the 8-octet header). Not counted by LENGTH. | per CRC alg |

**LENGTH semantics — confirmed [SPEC]:** IEEE 1815-2012 defines LENGTH as "the number of
octets in the remainder of the frame, not including CRC check octets." It counts CONTROL +
DESTINATION + SOURCE + user data; it does **not** count the two sync octets, the LENGTH octet
itself, or any CRC octets. (DNP3 Primer Rev A; RACOM DNP3 ref; Chipkin AN2013-004b.)

- **Minimum LENGTH = 5** — a frame with no user data: CONTROL(1) + DEST(2) + SOURCE(2). Used
  by link-control frames (e.g. RESET_LINK / LINK_STATUS).
- **Maximum LENGTH = 255** — CONTROL(1) + DEST(2) + SOURCE(2) + **250 user octets**.

**Parser sanity gate (security-relevant):** reject any frame with `LENGTH < 5`. Chipkin
AN2013-004b documents the canonical outstation validation: a frame is invalid if its third
octet (LENGTH) is `< 5`, or if `LENGTH + 3 + (CRC octet count)` exceeds the device's max frame
size or the octets actually received. The `+3` accounts for the 2 sync octets + the LENGTH
octet that LENGTH does not count. [SPEC]

### 1.2 CONTROL octet bitfields [SPEC]

Bit 7 is MSB. (RACOM DNP3 ref; Chipkin DNP3 Quick Reference.)

| Bit(s) | Mask | Field | Meaning |
|--------|------|-------|---------|
| 7 | `0x80` | **DIR** | Direction. `1` = from master, `0` = from outstation. |
| 6 | `0x40` | **PRM** | Primary message. `1` = primary (initiating) station frame; `0` = secondary (responding). |
| 5 | `0x20` | **FCB** | Frame Count Bit. Alternates 0/1 across successive confirmed frames to detect lost/duplicated frames. Only meaningful when FCV=1. |
| 4 | `0x10` | **FCV / DFC** | When PRM=1: **FCV** (Frame Count Valid) — `1` ⇒ examine FCB. When PRM=0: **DFC** (Data Flow Control) — `1` ⇒ receive buffer full, pause. |
| 0–3 | `0x0F` | **Link function code** | 4-bit data-link function (distinct from the *application* function code in §3). |

**Link-layer function codes (4-bit, in CONTROL):** primary (PRM=1): `0x0`=RESET_LINK_STATES,
`0x2`=TEST_LINK_STATES, `0x3`=CONFIRMED_USER_DATA, `0x4`=UNCONFIRMED_USER_DATA,
`0x9`=REQUEST_LINK_STATUS. Secondary (PRM=0): `0x0`=ACK, `0x1`=NACK, `0x9`=LINK_STATUS,
`0xB`=NOT_SUPPORTED. [SPEC] — Note: user data (and thus the transport/application layers) is
carried only in CONFIRMED_USER_DATA (`0x3`) and UNCONFIRMED_USER_DATA (`0x4`) frames; the
analyzer should only descend into transport/app parsing for those two link functions.

### 1.3 CRC-16/DNP block layout — exact byte math [SPEC]

DNP3 does **not** put one CRC over the whole frame. It interleaves a 2-octet CRC after the
8-octet header and after **every** 16 user-data octets (the final data block may be 1..16
octets). v1 scope decision: **strip these CRC blocks structurally; do NOT validate** — but the
parser must locate them precisely to skip them.

**On-wire structure:**
```
[START1][START2][LEN][CTRL][DEST_lo][DEST_hi][SRC_lo][SRC_hi]   ← 8 header octets
[HDR_CRC_lo][HDR_CRC_hi]                                        ← 2 octets (header CRC)
[ up to 16 user octets ][CRC_lo][CRC_hi]                        ← data block 1
[ up to 16 user octets ][CRC_lo][CRC_hi]                        ← data block 2
...                                                             ← (final block 1..16 user octets)
```

**User-data octet count** `U = LENGTH − 5` (subtract CONTROL+DEST+SOURCE = 5).
`U` ranges 0..250.

**Number of data blocks (each ≤16 user octets):**
```
num_data_blocks = ceil(U / 16)          # 0 when U == 0
```

**Total CRC octets in the frame:**
```
crc_octets = 2 + 2 * num_data_blocks    # 2 for the header CRC, 2 per data block
```

**Total frame length on the wire (what the parser must consume from the stream):**
```
frame_len = 3                # START1 + START2 + LENGTH
          + LENGTH           # CONTROL + DEST + SOURCE + user data
          + 2                # header CRC
          + 2 * num_data_blocks   # one CRC per data block
        = 3 + LENGTH + 2 + 2 * ceil((LENGTH - 5) / 16)
```
Equivalently: `frame_len = 5 + LENGTH + 2 * ceil((LENGTH - 5) / 16)`.

**To recover the contiguous transport+application payload, the parser walks the data blocks,
copying ≤16 user octets per block and skipping each trailing 2-octet CRC.** The first user
octet of data block 1 is the **transport octet** (§2); application data follows.

**Polynomial:** CRC-16/DNP uses poly **`0x3D65`** (`x¹⁶+x¹³+x¹²+x¹¹+x¹⁰+x⁸+x⁶+x⁵+x²+1`),
`init=0x0000`, `refin=true`, `refout=true`, `xorout=0xFFFF`. (reveng CRC catalogue;
RACOM DNP3 ref.) [SPEC] — *Not needed for v1 (CRC not validated)*, but documented so a future
validation pass is unambiguous. Note: `xorout` is `0xFFFF` per the reveng catalogue
("CRC-16/DNP"); the task's preview text truncated this value — flagged so F2 does not record
`xorout=0x0000`. [SPEC, xorout cross-checked against reveng catalogue]

### 1.4 Maximum frame size — confirmed 292 octets [SPEC]

With LENGTH at its max 255: `U = 250` user octets ⇒ `num_data_blocks = ceil(250/16) = 16`
(15 full 16-octet blocks = 240, plus one 10-octet block).

```
2   sync (START1+START2)
1   LENGTH
1   CONTROL
4   DEST(2) + SOURCE(2)
250 user data
2   header CRC
32  16 data-block CRCs (16 blocks × 2)
---
292 octets total on the wire
```

DNP Users Group Primer Rev A states verbatim: "the maximum length link layer frame is 292
octets if all the CRC and header octets are counted" and "the maximum number of octets in the
data payload is 250, not including CRC octets." **Confirmed: max DNP3 link frame = 292 bytes.**
[SPEC]

**Application-data capacity note [SPEC]:** of the 250 user octets, **1 is consumed by the
transport octet**, so a single link frame carries at most **249 application-layer octets**
(DNP Primer). Larger application fragments span multiple link frames, reassembled via the
transport function (§2).

---

## 2. Transport Function [SPEC]

The transport function is a **single octet** — the first user octet of the link-frame payload
(data block 1, offset 0 after CRC-stripping). It segments/reassembles application fragments
across link frames.

| Bit(s) | Mask | Field | Meaning |
|--------|------|-------|---------|
| 7 | `0x80` | **FIN** | Final segment of the application fragment. |
| 6 | `0x40` | **FIR** | First segment of the application fragment. |
| 0–5 | `0x3F` | **SEQUENCE** | 6-bit transport sequence (0..63), increments per segment; wraps mod 64. |

(Wireshark DNP3 dissector; CISA `icsnpp-dnp3`; Triangle MicroWorks DNP3 Overview.) [SPEC]

**FIR=1 identifies the first transport segment of an application fragment.** Per the locked
scope ("parse the application function code from the FIR=1 first fragment only"), the analyzer
extracts the Application Control octet + Application Function Code (§3) **only from the
transport segment whose FIR bit is set** (`transport_octet & 0x40 != 0`). Segments with FIR=0
are continuation segments — their first application bytes are *not* a fresh app header and must
not be re-parsed as a function code. A single-frame fragment has **FIR=1 and FIN=1** in the
same transport octet (`0xC0 | seq`).

---

## 3. Application Layer [SPEC]

The application fragment (after the transport octet) begins with an **Application Control**
octet, then the **Application Function Code** octet, then objects.

### 3.1 Application Control octet [SPEC]

| Bit(s) | Mask | Field | Meaning |
|--------|------|-------|---------|
| 7 | `0x80` | **FIR** | First fragment of the application response/request sequence. |
| 6 | `0x40` | **FIN** | Final fragment. |
| 5 | `0x20` | **CON** | Confirm requested — receiver must return an Application CONFIRM. |
| 4 | `0x10` | **UNS** | **Unsolicited.** Set in **unsolicited responses** (and in the master's CONFIRM of an unsolicited response). |
| 0–3 | `0x0F` | **SEQ** | 4-bit application sequence (0..15). |

(Wireshark DNP3 dissector; Schneider Geo SCADA DNP3 driver docs; Suricata `dnp3_ind`.) [SPEC]

**UNS bit — confirmed meaning [SPEC]:** UNS=1 marks a message as part of the unsolicited
exchange. Outstations report event data without being polled by sending an
**UNSOLICITED_RESPONSE (`0x82`)** with the Application-Control **UNS bit set**. (DNP Primer;
Geo SCADA: "outstations report the event data to the DNP3 master by transmitting unsolicited
messages (also referred to as unsolicited responses).")

### 3.2 Application Function Code table — confirmed hex values [SPEC]

Confirmed against the CISA `icsnpp-dnp3` Zeek analyzer constants
(`scripts/consts.zeek`), Suricata `dnp3_func` keyword list, Wireshark dissector, and Tenable
plugin docs (restart codes). The task's required values are all confirmed:

| Hex | Dec | Name | Class | Notes |
|-----|-----|------|-------|-------|
| `0x00` | 0 | **CONFIRM** | control | App-layer confirmation (no data). |
| `0x01` | 1 | **READ** | READ | Dominant SCADA poll (integrity poll = READ Class 0). |
| `0x02` | 2 | **WRITE** | **WRITE** | Writes objects/parameters. **→ T0836 Modify Parameter.** |
| `0x03` | 3 | **SELECT** | **CONTROL** | Select half of Select-Before-Operate (SBO). |
| `0x04` | 4 | **OPERATE** | **CONTROL** | Operate half of SBO. Actuates the selected point. |
| `0x05` | 5 | **DIRECT_OPERATE** | **CONTROL** | One-shot control, response expected (bypasses SBO). |
| `0x06` | 6 | **DIRECT_OPERATE_NR** | **CONTROL** | Direct operate, **no response** (NR). |
| `0x07` | 7 | IMMED_FREEZE | mgmt | Freeze counters. |
| `0x08` | 8 | IMMED_FREEZE_NR | mgmt | Freeze, no response. |
| `0x09` | 9 | FREEZE_CLEAR | mgmt | Freeze and clear. |
| `0x0A` | 10 | FREEZE_CLEAR_NR | mgmt | …no response. |
| `0x0B` | 11 | FREEZE_AT_TIME | mgmt | Time-synchronized freeze. |
| `0x0C` | 12 | FREEZE_AT_TIME_NR | mgmt | …no response. |
| `0x0D` | 13 | **COLD_RESTART** | **MGMT/DISRUPTIVE** | Full device restart. **→ T0814 DoS.** |
| `0x0E` | 14 | **WARM_RESTART** | **MGMT/DISRUPTIVE** | Partial restart. **→ T0814 DoS.** |
| `0x0F` | 15 | INITIALIZE_DATA | mgmt | (obsolete in some profiles) |
| `0x10` | 16 | INITIALIZE_APPL | mgmt | Initialize application. |
| `0x11` | 17 | START_APPL | mgmt | Start application. |
| `0x12` | 18 | STOP_APPL | mgmt | Stop application. |
| `0x13` | 19 | SAVE_CONFIG | mgmt | Save configuration. |
| `0x14` | 20 | ENABLE_UNSOLICITED | mgmt | Enable unsolicited reporting. |
| `0x15` | 21 | DISABLE_UNSOLICITED | mgmt | Disable unsolicited reporting. |
| `0x16` | 22 | ASSIGN_CLASS | mgmt | Assign objects to event classes. |
| `0x17` | 23 | DELAY_MEASURE | mgmt | Round-trip delay measurement. |
| `0x18` | 24 | RECORD_CURRENT_TIME | mgmt | |
| `0x19` | 25 | OPEN_FILE | file | |
| `0x1A` | 26 | CLOSE_FILE | file | |
| `0x1B`–`0x21` | 27–33 | (file/auth ops) | file/auth | DELETE_FILE, GET_FILE_INFO, AUTHENTICATE_FILE, ABORT_FILE, etc. |
| `0x81` | 129 | **RESPONSE** | response | Solicited response to a request. |
| `0x82` | 130 | **UNSOLICITED_RESPONSE** | response | Outstation-initiated; carries event data, UNS bit set (§3.1). |
| `0x83` | 131 | AUTHENTICATE_RESP | response | Secure Authentication response. |

**Required-value confirmation (task §3):** CONFIRM=0x00, READ=0x01, WRITE=0x02, SELECT=0x03,
OPERATE=0x04, DIRECT_OPERATE=0x05, DIRECT_OPERATE_NR=0x06, COLD_RESTART=0x0D,
WARM_RESTART=0x0E, RESPONSE=0x81, UNSOLICITED_RESPONSE=0x82 — **all confirmed** against the
CISA icsnpp-dnp3 constants, Suricata, and Wireshark. [SPEC]

> The **control set** the analyzer should weight as state-changing actuation: SELECT `0x03`,
> OPERATE `0x04`, DIRECT_OPERATE `0x05`, DIRECT_OPERATE_NR `0x06`. The **parameter-write**
> code: WRITE `0x02`. The **disruptive restart** codes: COLD_RESTART `0x0D`, WARM_RESTART
> `0x0E`.

---

## 4. Addressing Anomalies [SPEC + UNVERIFIED on exact reservations]

DESTINATION/SOURCE are 2-octet **little-endian** link addresses (§1.1). Address space
0x0000–0xFFFF (65 536 values).

| Address (range) | Meaning | Confidence |
|-----------------|---------|------------|
| `0x0000`–`0xFFEF` | Individual station addresses (assignable to masters/outstations). | [SPEC] |
| `0xFFF0`–`0xFFFB` | Reserved. | [UNVERIFIED] — IEEE 1815 reserves a block immediately below the broadcast range; the exact lower bound (`0xFFF0` vs other) could not be confirmed against primary spec text. F2 should confirm against IEEE 1815-2012 Table for "Reserved addresses" before relying on a specific lower bound. |
| `0xFFFC` | Reserved (self-address support / special). | [UNVERIFIED] — see self-address note below. |
| `0xFFFD` | **Broadcast, confirmation required** (DIR/CON semantics: outstation should confirm). | [SPEC — concurring secondary refs; exact IEEE clause UNVERIFIED] |
| `0xFFFE` | **Broadcast, confirmation optional / mandatory-per-frame** (broadcast with application confirmation as requested). | [SPEC — concurring secondary refs; exact IEEE clause UNVERIFIED] |
| `0xFFFF` | **Broadcast, no confirmation** ("all stations", unconfirmed). | [SPEC] — widely concurring (RACOM, VTScada): `0xFFFF` = all-stations broadcast. |

**Broadcast `0xFFFD`/`0xFFFE`/`0xFFFF` — confirmed as the three broadcast destination
addresses [SPEC for existence of the triple; per-address confirm-semantics partially
UNVERIFIED]:** Multiple references agree DNP3 reserves the top three destination addresses as
broadcasts that differ only in confirmation handling. The precise mapping —
`0xFFFD` = broadcast requiring application confirmation, `0xFFFE` = broadcast with optional
confirmation, `0xFFFF` = broadcast with no confirmation — is reported consistently by secondary
sources (VTScada DNP3 addressing; vendor device profiles) but I could **not** pin it to a quoted
IEEE 1815-2012 clause in this pass. **F2 action:** confirm the exact `0xFFFD`/`0xFFFE` semantics
against IEEE 1815-2012 §"Broadcast addresses" before encoding confirm-handling logic; treating
**any** destination in `0xFFFD–0xFFFF` as "broadcast" is safe and spec-supported regardless.

**Self-address range [UNVERIFIED]:** IEEE 1815-2012 defines a **self-address** feature
(address `0xFFFC` per several implementations) letting an outstation that does not know its own
configured address respond to a special self-address query. The exact reserved value(s) and
enable/disable behavior could **not** be confirmed against primary spec text in this pass.
F2 should verify against IEEE 1815-2012 before encoding self-address detection.

**Detection-relevant addressing signals [JUDGMENT]:**
- A **control command** (SELECT/OPERATE/DIRECT_OPERATE) sent to a **broadcast** destination
  (`0xFFFD`–`0xFFFF`) is anomalous — control should be unicast to a specific outstation.
- A frame whose **SOURCE** is not on the known-master allowlist but carries DIR=1 (claims to be
  from a master) is a spoofing/unauthorized-master signal.
- Destinations in the reserved block (below broadcast) are malformed/suspicious.

---

## 5. Detection-Relevant Behaviors → MITRE ATT&CK for ICS [MITRE + JUDGMENT]

All technique IDs/names/tactics verified directly against attack.mitre.org and the official
**v18.1→v19.0 detailed changelog** for the pinned **ics-attack-19.1**. See §7 for the
revocation evidence. The table below gives **both** the v19 canonical IDs and the legacy IDs
the scope locked, so F2 can choose its emission policy explicitly.

| Behavior (DNP3 signal) | v19.1 canonical | Legacy (locked) | Tactic | Rationale |
|------------------------|-----------------|-----------------|--------|-----------|
| **(a) Unauthorized control command** — SELECT `0x03` / OPERATE `0x04` / DIRECT_OPERATE `0x05` / DIRECT_OPERATE_NR `0x06` from a SOURCE not on the known-master allowlist, or to an unexpected outstation/point. | **T1692.001** *Unauthorized Message: Command Message* | **T0855** *Unauthorized Command Message* (REVOKED) | Evasion + Impair Process Control | Direct actuation by an unauthorized actor. **High.** Primary detection target. |
| **(b) Block Command Message** — command messages prevented from reaching the outstation (e.g. on this passive analyzer: a master's SELECT/OPERATE/DIRECT_OPERATE issued but the expected outstation RESPONSE/CONFIRM never appears within window; or observed link-layer disruption). | **T1691.001** *Block Operational Technology Message: Command Message* | **T0803** *Block Command Message* (REVOKED) | Inhibit Response Function | Inhibits corrective response. The MITRE description: a blocked command message "can inhibit response functions from correcting a disruption or unsafe condition." Ukraine 2015 (Sandworm) is the canonical example. **High.** |
| **(c) Cold/warm restart abuse** — COLD_RESTART `0x0D` / WARM_RESTART `0x0E`, especially repeated or from an unexpected source. | **T0814** *Denial of Service* | T0814 (unchanged) | Inhibit Response Function | Restart renders the outstation temporarily unresponsive — removes operator visibility/control. **High.** Cheap single-packet detector. |
| **(d) Unsolicited-response anomaly** — UNSOLICITED_RESPONSE `0x82` (UNS bit set) bursts, from unexpected outstations, or where unsolicited was never ENABLE_UNSOLICITED'd. | **T0814** *Denial of Service* (flood) and/or **T1692.002** *Unauthorized Message: Reporting Message* (spoofed/forged reports) | (no clean legacy single ID) | Inhibit Response Function / Evasion+Impair | Flooding masks real events or exhausts the master; forged unsolicited reports falsify process state. **Medium–High.** [JUDGMENT — see §5.1.] |
| **Parameter write** — WRITE `0x02` to objects holding setpoints/limits/config. | **T0836** *Modify Parameter* | T0836 (unchanged) | Impair Process Control | Corrupts the process safety envelope; often a single stealthy write. **High.** |
| **(Outcome) Loss of operator control** — sustained inability to issue/confirm control commands (outstation flooded, restarted, or commands blocked). | **T0827** *Loss of Control* | (scope said T0828 — **WRONG NAME**) | **Impact** | Terminal *impact*, not a method. See §6 verdict. |

**Co-emission rules the locked scope specified, mapped to v19.1:**
- WRITE `0x02` → **T0836 Modify Parameter** ✓ (unchanged).
- COLD/WARM_RESTART `0x0D`/`0x0E` → **T0814 Denial of Service** ✓ (unchanged).
- Control commands (SELECT/OPERATE/DIRECT_OPERATE) → legacy **T0855** = v19 **T1692.001** —
  the scope said "control commands → T0855," which is **the revoked ID**; emit T1692.001 (or
  T0855 as a legacy alias) per F2's emission policy.

**Cheap, high-value single-packet detectors (recommend F2 implement first):**
- COLD_RESTART `0x0D` / WARM_RESTART `0x0E` → T0814 (one-byte FC match, near-zero FP on a
  steady-state polled segment).
- Any control FC (`0x03`–`0x06`) from a non-allowlisted SOURCE → T1692.001/T0855 candidate.
- UNSOLICITED_RESPONSE `0x82` from an outstation with no prior ENABLE_UNSOLICITED → anomaly.

### 5.1 Unsolicited-response anomaly threshold [JUDGMENT]

There is no spec-defined rate. Unsolicited responses are **legitimate and expected** when the
outstation has been configured/enabled for them (Class 1/2/3 events). Flag as anomalous when:
- UNSOLICITED_RESPONSE bursts exceed a **config-tunable rate** (default suggestion: a sustained
  rate well above the segment's established baseline — F2 must pin a number, as with the Modbus
  write-burst threshold), OR
- they originate from an outstation for which **no ENABLE_UNSOLICITED (`0x14`)** was observed in
  the capture, OR
- they target/claim a SOURCE outside the known-outstation set.
Confidence: [JUDGMENT] — baseline-relative; ship a tunable default and log the observed rate.

### 5.2 Passive-analyzer caveat for "Block Command Message" (T1691.001) [JUDGMENT]

wirerust is a **passive PCAP analyzer**, not an inline device. It cannot *observe blocking
directly* (a blocked message is, by definition, absent). The defensible detection signal is
**request-without-response correlation**: a master control command (SELECT/OPERATE/
DIRECT_OPERATE, expecting a response) with **no corresponding outstation RESPONSE (`0x81`) or
CONFIRM within a timeout window**, especially across multiple commands. F2 must specify the
correlation key (5-tuple + app SEQ + outstation address) and the timeout. This is an
*inference*, not a direct observation — flag with appropriate confidence. [JUDGMENT]

---

## 6. T0803 / T0828 / T0855 / T0827 — verdicts [MITRE]

**T0803 Block Command Message — VERDICT: REVOKED in ics-attack-19.1.**
- Replaced by **T1691.001** *Block Operational Technology Message: Command Message*, tactic
  **Inhibit Response Function** (TA0107). T1691.001 created 2026-04-20, last modified
  2026-05-12 (live, active). The DNP3 behavior is valid (blocked control command inhibits
  corrective response). **F2 must emit T1691.001** (or T0803 only as a documented legacy alias).

**T0855 Unauthorized Command Message — VERDICT: REVOKED in ics-attack-19.1.**
- Replaced by **T1692.001** *Unauthorized Message: Command Message*, tactics **Evasion**
  (TA0103) + **Impair Process Control** (TA0106). T1692.001 created 2026-04-20, last modified
  2026-05-12 (live, active). DNP3 mapping (unauthorized SELECT/OPERATE/DIRECT_OPERATE) is
  valid. **F2 must emit T1692.001** (or T0855 only as a documented legacy alias).
- **⚠️ Contradicts `attack-ics-version-pin.md`**, which lists T0855 as "Active" in v19.1. That
  entry is **stale** (validated against the v18.1→v19.0 changelog — see §7). This is a
  `DF-VALIDATION-001`-eligible finding.

**T0828 — VERDICT: NAME MISMATCH; the technique the scope wanted is T0827.**
- **T0828** *Loss of Productivity and Revenue* (Impact, TA0105, active; v1.0, last modified
  2025-04-16) — describes economic loss from disruption, **not** loss of operator control.
- **T0827** *Loss of Control* (Impact, TA0105, active; v1.0, last modified 2026-05-12) is the
  technique for "operators cannot issue commands." **This is the correct mapping for a DNP3
  control-loss condition.**
- **Defensibility:** mapping DNP3 control-loss (operator unable to issue/confirm control because
  the outstation is flooded/restarted/blocked) to **T0827 Loss of Control** is **defensible** —
  T0827's own description is "a sustained loss of control … in which operators cannot issue any
  commands." But note T0827 is an **Impact-tactic *outcome***, not a method. On a passive
  analyzer it should be emitted as a **derived/correlated finding** (the *consequence* of
  observed T0814/T1691.001 conditions), not from a single packet. If F2 wants the economic
  framing instead, T0828 applies — but that is a different claim and **not** what "loss of
  control" means. **Recommendation: replace the locked "T0828" with T0827, emitted as a
  correlated impact finding.** [MITRE + JUDGMENT]

**T0814 Denial of Service — VERDICT: valid, unchanged.** Inhibit Response Function (TA0107),
active. DNP3 trigger: COLD/WARM_RESTART, malformed/flood, unsolicited flood. ✓

**T0836 Modify Parameter — VERDICT: valid, unchanged.** Impair Process Control (TA0106),
active. DNP3 trigger: WRITE `0x02` to setpoint/limit/config objects. ✓

---

## 7. MITRE primary-source evidence (verbatim) [MITRE]

From the official **v18.1→v19.0 detailed changelog**
(`attack.mitre.org/docs/changelogs/v18.1-v19.0/changelog-detailed.html`), confirmed via
`perplexity_reason` (domain-restricted to attack.mitre.org) and the v19 updates page
(`attack.mitre.org/resources/updates/updates-april-2026/`):

- **[T0803] Block Command Message** — *"This object has been revoked by [T1691.001] Command
  Message."*
- **[T0855] Unauthorized Command Message** — revoked by **[T1692.001]** *Unauthorized Message:
  Command Message* (same revocation pattern; v1.2).
- v19 release note: the April 2026 v19 release **adds sub-techniques to ICS ATT&CK** — the
  mechanism by which these standalone techniques were consolidated under parent techniques
  T1691 / T1692.

Live-page confirmations (WebFetch, attack.mitre.org):
- **T0827** = "Loss of Control", Impact (TA0105), active, last-mod 2026-05-12.
- **T0828** = "Loss of Productivity and Revenue", Impact (TA0105), active, last-mod 2025-04-16.
- **T1691.001** = "Block Operational Technology Message: Command Message", Inhibit Response
  Function (TA0107), created 2026-04-20, last-mod 2026-05-12.
- **T1692.001** = "Unauthorized Message: Command Message", Evasion (TA0103) + Impair Process
  Control (TA0106), created 2026-04-20, last-mod 2026-05-12.
- **T0836** = "Modify Parameter", Impair Process Control (TA0106), active.
- **T0814** = "Denial of Service", Inhibit Response Function (TA0107), active.

---

## 8. Known False-Positive Considerations [JUDGMENT]

1. **Legitimate unsolicited reporting.** UNSOLICITED_RESPONSE is *normal* once
   ENABLE_UNSOLICITED'd. Gate §5.1 on rate/baseline + absence of an enabling exchange, not on
   mere presence.
2. **Commissioning / maintenance control bursts.** Engineering workstations legitimately issue
   SELECT/OPERATE and restarts during maintenance windows. Mitigate with a **master/source
   allowlist** and optional maintenance-window suppression; weight control from *unknown*
   sources harder.
3. **Request-without-response is not always blocking.** Packet loss, capture gaps, or
   DIRECT_OPERATE_NR (`0x06`, no-response *by design*) produce response-less commands legitimately.
   **Do not** map DIRECT_OPERATE_NR's missing response to T1691.001. Require a sustained pattern
   (§5.2).
4. **Port-20000-but-not-DNP3.** Validate before analyzing: require sync `0x0564`, a plausible
   LENGTH (5..255), and a known link function / app FC before raising DNP3 findings. This
   three-point gate (sync + LENGTH sanity + FC plausibility) mirrors the Modbus validity gate.
5. **Broadcast control is rare but not always hostile.** Some time-sync / global operations use
   broadcast legitimately; weight broadcast *control* (SELECT/OPERATE) harder than broadcast
   reads/time-sync.

---

## 9. Open Items / Where Sources Disagree (for F2 to resolve explicitly)

- **MITRE emission policy (BLOCKING):** emit v19 sub-technique IDs (T1691.001 / T1692.001) vs
  legacy parents (T0803 / T0855)? The pinned version revokes the legacy IDs. Recommend
  **emit v19 IDs**; if back-compat with prior wirerust output is required, emit v19 ID with the
  legacy ID as a documented `revoked-alias`. [MITRE]
- **T0828 → T0827 correction (BLOCKING):** scope said T0828 "Loss of Control"; T0828 is "Loss
  of Productivity and Revenue." Use **T0827 Loss of Control**. [MITRE]
- **`attack-ics-version-pin.md` is stale on T0855** — file as a validated `DF-VALIDATION-001`
  finding. [MITRE]
- **Broadcast confirm-semantics** (`0xFFFD`/`0xFFFE`): exact per-address confirmation behavior
  is [UNVERIFIED] against quoted IEEE 1815 text. Confirm before encoding confirm-handling. [SPEC/UNVERIFIED]
- **Self-address range** (`0xFFFC`?) is [UNVERIFIED]. Confirm against IEEE 1815-2012. [UNVERIFIED]
- **Reserved address lower bound** (`0xFFF0`?) is [UNVERIFIED]. Confirm against IEEE 1815-2012. [UNVERIFIED]
- **CRC `xorout`** = `0xFFFF` per reveng catalogue (not `0x0000`) — record correctly even though
  v1 does not validate CRC. [SPEC]
- **Unsolicited-anomaly + block-command thresholds** (§5.1, §5.2): no spec rate/timeout; F2 must
  pin tunable defaults, as with the Modbus write-burst threshold. [JUDGMENT]

---

## Sources

| # | Source | Used for |
|---|--------|----------|
| [1] | IEEE Std 1815-2012, *Distributed Network Protocol (DNP3)* | Authoritative protocol definition (LENGTH semantics, frame structure, addressing) |
| [2] | DNP Users Group, *DNP3 Primer Rev A* (`dnp.org/Portals/0/AboutUs/DNP3 Primer Rev A.pdf`) | 250 payload / 292 max-frame / 249 app-octet figures; layer model |
| [3] | RACOM DNP3 reference (`racom.eu/download/sw/prot/free/eng/dnp3.pdf`) | Header/CONTROL bitfields, little-endian addressing, CRC block layout |
| [4] | Chipkin *DNP3 Quick Reference* | CONTROL octet, link function codes |
| [5] | Chipkin *AN2013-004b Validation of Incoming DNP3 Data* | LENGTH<5 reject rule; frame-size validation arithmetic |
| [6] | Wireshark DNP3 dissector (`wireshark.org/docs/dfref/d/dnp3.html`) | Transport octet bits; app control bits; FC values |
| [7] | CISA `icsnpp-dnp3` Zeek (`github.com/cisagov/icsnpp-dnp3/blob/main/scripts/consts.zeek`) | Application function-code hex table |
| [8] | Suricata DNP3 keywords (`docs.suricata.io/en/latest/rules/dnp3-keywords.html`) | `dnp3_func` / `dnp3_ind` FC + indicator confirmation |
| [9] | reveng CRC catalogue (`reveng.sourceforge.io/crc-catalogue/16.htm`) | CRC-16/DNP poly 0x3D65, init, refin/refout, xorout |
| [10] | Schneider Geo SCADA DNP3 driver docs | Unsolicited-response semantics; ENABLE/DISABLE_UNSOLICITED |
| [11] | Tenable plugin (`tenable.com/plugins/nnm/49`) | COLD_RESTART = FC 13 (0x0D) confirmation |
| [12] | MITRE ATT&CK **v18.1→v19.0 detailed changelog** (`attack.mitre.org/docs/changelogs/v18.1-v19.0/changelog-detailed.html`) | **T0803/T0855 revocation evidence** |
| [13] | attack.mitre.org technique pages T0814 / T0827 / T0828 / T0836 / T1691.001 / T1692.001 | Names, tactics, active status, created/modified dates |
| [14] | attack.mitre.org v19 updates (`/resources/updates/updates-april-2026/`) | v19 ICS sub-technique addition; revocation listing |
| [15] | VTScada DNP3 addressing (`vtscada.com/help/Content/D_Tags/Dev_DNP3Addressing.htm`) | Broadcast/self-address (secondary, partial) |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Data Link Layer wire format, LENGTH semantics, CRC block math, 292-byte max — `reasoning_effort: high`; (2) Transport + Application layer bits + full FC table — `high`; (3) MITRE ATT&CK-ICS technique verification (T0803/T0828/T0855/T0814/T0836) — `high`. |
| Perplexity perplexity_reason | 1 | Cross-validation of the T0803/T0855 revocation + T0814/T0836/T0827/T0828 tactics against the official v18.1→v19.0 changelog (domain-restricted to attack.mitre.org / industrialcyber.co / redtrident.com, `search_context_size: high`). |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable (protocol spec + threat framework, not a software-library API). |
| Tavily | 0 | — |
| WebFetch | 6 | Direct primary-source verification of attack.mitre.org pages: T0828, T0827, T1691.001, T1692.001, and the v19 April-2026 update/changelog page (plus two empty returns on T0803/T0855 bare pages — resolved via the changelog + reason pass). |
| WebSearch | 0 | — |
| Training data | 1 area | Cross-check only on well-known DNP3 facts; every load-bearing value is sourced to IEEE 1815 / DNP Users Group / protocol references or attack.mitre.org. |

**Total MCP tool calls:** 4 (3 `perplexity_research` high-effort + 1 `perplexity_reason` high-context).
**WebFetch (non-MCP) primary-source verifications:** 6.
**Training data reliance:** low — all byte layouts, FC hex values, and MITRE IDs/tactics are
verified against IEEE 1815 / DNP Users Group / Wireshark / CISA icsnpp-dnp3 / reveng catalogue
and attack.mitre.org primary pages + the official v18.1→v19.0 changelog. Items that could not
be pinned to primary IEEE 1815 clause text (exact broadcast confirm-semantics, self-address
value, reserved lower bound) are explicitly marked **[UNVERIFIED]** for F2 to confirm.

**Cross-source conflict flagged:** the deep-research synthesis on MITRE initially claimed
T0803/T0855 were *deprecated* and asserted T0828≠Loss of Control; this was **independently
confirmed correct** against the official MITRE changelog and live pages, and it **contradicts
the project's existing `attack-ics-version-pin.md`** (which is therefore stale on T0855). The
conflict is resolved in favor of the primary MITRE changelog. [MITRE]
