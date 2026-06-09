# Domain Research: Modbus TCP Protocol Analyzer for ICS/OT Forensics

**Feature:** wirerust #7 — Modbus TCP analyzer
**Date:** 2026-06-09
**Purpose:** Implementation-oriented reference feeding the F2 spec. Favors exact byte
layouts, code values, and concrete detection signals over prose.
**Authoritative sources:** Modbus.org *MODBUS Application Protocol Specification V1.1b3*
(`modbusprotocolspecification.pdf`); Modbus.org *MODBUS Messaging on TCP/IP Implementation
Guide V1.0b* §3.1.3 (`messagingimplementationguide.pdf`); MITRE ATT&CK for ICS
(`attack.mitre.org/techniques/ics`).

> **Confidence legend:** [SPEC] = verified against Modbus.org spec text; [MITRE] = verified
> against MITRE ATT&CK for ICS; [JUDGMENT] = detection threshold / heuristic that F2 must set
> explicitly (sources disagree or value is environment-dependent).

---

## 1. MBAP Header Wire Format [SPEC]

Modbus TCP runs on **TCP port 502**. Every Application Data Unit (ADU) is a 7-byte **MBAP
header** (MODBUS Application Protocol header) followed by the **PDU** (1-byte function code +
data). Defined in Messaging Implementation Guide V1.0b §3.1.3.

| Offset | Size | Field | Value / Semantics | Endianness |
|--------|------|-------|-------------------|------------|
| 0–1 | 2 | **Transaction Identifier** | Set by client; server **echoes** it unchanged in the response. Used to pair request↔response. | big-endian |
| 2–3 | 2 | **Protocol Identifier** | **Always `0x0000`** for Modbus. Non-zero ⇒ not Modbus. | big-endian |
| 4–5 | 2 | **Length** | Byte count of **everything that follows the Length field** = Unit ID (1) + PDU. | big-endian |
| 6 | 1 | **Unit Identifier** | Slave/sub-unit address; relevant for serial-gateway routing. Echoed in response. | n/a |
| 7… | 1+ | **PDU** | Function Code (1 byte) + function-specific data. | data fields big-endian |

**All multi-byte fields are big-endian (network byte order).** [SPEC] The protocol-wide rule
(spec §4.2): "MODBUS uses a big-Endian representation for addresses and data items… the most
significant byte is sent first." Register data inside the PDU is also big-endian.

**Length field — exact semantics [SPEC]:** counts the **Unit ID byte plus the full PDU**, i.e.
`Length = 1 + len(PDU)`. It does *not* count the 6 header bytes preceding it. Spec text:
"Number of following bytes." A Read-Holding-Registers request (UnitID + FC + 2-byte addr +
2-byte qty = 6 bytes after Length) gives **Length = 6**.

**Size bounds [SPEC]** (from spec V1.1b3, derived from the original RS-485 256-byte ADU):
- RS-485 ADU = 256 = address(1) + PDU(253) + CRC(2) ⇒ **max PDU = 253 bytes**.
- **Max Modbus TCP ADU = 253 + MBAP(7) = 260 bytes.**
- Therefore **valid `Length` range = 2 … 253** (min = UnitID + 1-byte FC-only PDU; max = UnitID + 252-byte PDU).
- Note: spec V1 originally set the TCP PDU limit to 249; **V1.1b harmonized it to 253** to match serial. Use 253.

**Parser validation rules for F2 (security-relevant):**
1. Reject ADU where `Length < 2` or `Length > 253` (malformed / fuzzing — Broadcom signature "TCP MODBUS - Illegal Packet Size", ASID 20676, flags >260-byte frames).
2. Reject / flag `Protocol Identifier != 0x0000` (see §7 — non-Modbus traffic on 502).
3. A single TCP segment may carry **multiple ADUs**, and one ADU may **span TCP segments** — frame strictly by `Length`, never by segment boundary. Re-sync the stream on `Length` to avoid desync cascades.

---

## 2. Function Codes [SPEC]

Full standard set from spec V1.1b3 §6, with READ vs **WRITE** (state-changing, higher
forensic risk) classification. WRITE and management codes are the ones an analyzer should
weight heavily.

| FC (hex) | Name | Class | Notes |
|----------|------|-------|-------|
| `0x01` | Read Coils | READ | bit-addressable outputs |
| `0x02` | Read Discrete Inputs | READ | bit-addressable inputs |
| `0x03` | Read Holding Registers | READ | the dominant SCADA poll |
| `0x04` | Read Input Registers | READ | |
| `0x05` | **Write Single Coil** | **WRITE** | single output bit; toggling interlocks |
| `0x06` | **Write Single Register** | **WRITE** | single 16-bit reg; setpoint/parameter change |
| `0x07` | Read Exception Status | READ (diag) | serial-line status byte |
| `0x08` | **Diagnostics** | **WRITE/MGMT** | sub-function coded — see below; several are state-changing/DoS |
| `0x0B` | Get Comm Event Counter | READ (diag) | |
| `0x0C` | Get Comm Event Log | READ (diag) | |
| `0x0F` | **Write Multiple Coils** | **WRITE** | bulk output write |
| `0x10` | **Write Multiple Registers** | **WRITE** | bulk register write; bulk parameter manipulation |
| `0x11` | Report Server/Slave ID | READ (recon) | device fingerprinting target |
| `0x14` | Read File Record | READ | |
| `0x15` | **Write File Record** | **WRITE** | file-record write |
| `0x16` | **Mask Write Register** | **WRITE** | AND/OR-mask register mutation (subtle, easy to miss) |
| `0x17` | **Read/Write Multiple Registers** | **WRITE** | atomic read+write; **the write half is state-changing** |
| `0x18` | Read FIFO Queue | READ | |
| `0x2B` | **Encapsulated Interface Transport (MEI)** | MIXED/MGMT | MEI type `0x0E` = Read Device Identification (recon); `0x0D` = CANopen. Tunnels sub-protocols. |

**WRITE set (treat as state-changing / elevated risk):** `0x05, 0x06, 0x0F, 0x10, 0x15,
0x16, 0x17`, plus state-changing **`0x08` Diagnostics** sub-functions.

**Diagnostics (`0x08`) sub-function codes [SPEC]** — sub-function is a 2-byte big-endian field
after the FC. Forensically important ones:

| Sub-func | Name | Risk |
|----------|------|------|
| `0x0000` | Return Query Data (loopback) | benign |
| `0x0001` | **Restart Communications Option** | **state-changing** — restarts the port, can clear the comm event log; DoS-capable |
| `0x0004` | **Force Listen Only Mode** | **DoS** — slave stops responding to all commands (no further responses until reset) |
| `0x000A` | **Clear Counters and Diagnostic Register** | **anti-forensic** — wipes diagnostic state/counters |

---

## 3. Exception Responses [SPEC]

A server signals an error by returning the **function code with the high bit set**:
`exception_FC = request_FC | 0x80` (i.e. `request_FC + 0x80`), followed by **one exception
code byte**. Example: a failed Read Holding Registers (`0x03`) returns `0x83` + code.

**Detection rule:** any response FC byte `>= 0x80` is an exception response; mask with
`& 0x7F` to recover the original FC for correlation.

| Code | Name | Forensic meaning |
|------|------|------------------|
| `0x01` | Illegal Function | FC not supported by device. **Bursts across many FCs = function-code scanning / capability enumeration.** |
| `0x02` | Illegal Data Address | Address not present. **Sweeps across addresses = register/coil map enumeration (recon).** |
| `0x03` | Illegal Data Value | Value/quantity out of range. Malformed or fuzzing input; over-large quantity fields. |
| `0x04` | Server Device Failure | Unrecoverable device error during the action. Possible impact of an attack or device stress. |
| `0x05` | Acknowledge | Long-running request accepted; client should poll. Normal — but pairs with `0x06`. |
| `0x06` | Server Device Busy | Device busy. **Repeated/rising rate may indicate resource exhaustion / DoS pressure.** |
| `0x07` | Negative Acknowledge | Request can't be performed (program-function context). Rare; flag if frequent. |
| `0x08` | Memory Parity Error | Memory parity fault (file-record ops). Hardware/integrity issue. |
| `0x0A` | Gateway Path Unavailable | Serial-gateway misconfig/overload — no path to target. |
| `0x0B` | Gateway Target Device Failed to Respond | Target behind gateway silent. **Can indicate a device knocked offline (DoS impact).** |

> Note: there is no `0x09` in the standard exception set; codes are `0x01–0x08`, `0x0A`,
> `0x0B`. [SPEC]

---

## 4. Request vs Response Disambiguation [SPEC + JUDGMENT]

**Modbus TCP carries no explicit message-type / direction flag.** An analyzer must infer it.
Reliable heuristics, in priority order:

1. **TCP direction (primary).** Packet whose **destination port = 502** ⇒ **request**
   (client→server). Packet whose **source port = 502** ⇒ **response** (server→client). This
   is the strongest signal on a normally-configured segment. [JUDGMENT — fails if a device
   uses 502 as an ephemeral source port, rare but possible; corroborate with #2/#3.]
2. **Exception flag.** Response FC `>= 0x80` ⇒ **exception response**, unambiguously a
   response regardless of direction inference. [SPEC]
3. **Transaction ID + Unit ID + FC correlation.** The server **echoes Transaction ID, Unit
   ID, and (for success) the FC**. Match a packet against the outstanding request keyed on
   `(TCP 5-tuple, Transaction ID, Unit ID)`; the echoed FC (or `FC|0x80`) confirms it is the
   response. [SPEC]
4. **Structural disambiguation** (when direction is ambiguous and txn state is unknown): for
   several FCs the request and response PDUs differ in shape (e.g. Read Holding Registers
   *request* = addr+qty = 4 data bytes; *response* = byte-count + register data). Use as a
   tiebreaker only. [JUDGMENT]

**Recommendation for F2:** maintain a per-connection transaction table keyed on
`(Transaction ID, Unit ID)`; use port-direction as the default classifier and the echoed-FC
correlation to validate and to detect anomalies (orphan responses, FC mismatch between
request and response, duplicate Transaction IDs in flight).

---

## 5. Attack / Anomaly Detection Patterns → MITRE ATT&CK for ICS [MITRE + JUDGMENT]

For each: the concrete Modbus signal and severity rationale. MITRE technique IDs verified
against ATT&CK for ICS.

| Technique | ID | Tactic | Concrete Modbus signal | Severity rationale |
|-----------|-----|--------|------------------------|--------------------|
| **Unauthorized Command Message** | **T0855** | Impair Process Control | Any WRITE FC (`0x05/0x06/0x0F/0x10/0x16/0x17`) from a source not on the known-writer allowlist, or to a register/coil outside the expected write set. | Direct actuation of the physical process by an unauthorized actor. **High.** Primary detection target on a read-dominated segment. |
| **Modify Parameter** | **T0836** | Impair Process Control | `0x06` / `0x10` / `0x16` writes to holding registers that hold **setpoints, alarm thresholds, limits, or config** (e.g. moving a tank high-alarm from 90%→98%). | Corrupts process safety envelope; often stealthy (single small write). **High.** |
| **Denial of Service** | **T0814** | Inhibit Response Function | `0x08` Diagnostics sub-func **`0x0004` Force Listen Only** (slave goes silent) or **`0x0001` Restart Communications**; also malformed/oversized ADUs (>260) and flooding. | Removes operator visibility/control of the device. **High.** Force-Listen-Only is an unambiguous, cheap-to-detect single-packet signal. |
| **Brute Force I/O** | **T0806** | Impair Process Control | Rapid repeated WRITE bursts to the same coil/register (toggling an interlock, ramping a pump setpoint to find failure point). | Stress/abuse of an I/O point to force failure. **High.** Detected via the rate threshold in §5.1. |
| **Manipulate I/O Image** | **T0835** | Impair Process Control | Writes (`0x05/0x0F` to coils, `0x06/0x10` to registers) that alter the device's I/O image to mask or falsify process state. | Can hide an attack from operators. **High.** Distinguish from legitimate control writes by writer identity + target. |
| **Manipulation of Control** | **T0831** | Impair Process Control | Coordinated WRITE sequences that drive the process outside safe bounds (e.g. raise setpoint while suppressing the alarm threshold). | Physical-process damage. **High.** Correlate multiple writes across registers within a short window. |

**Cheap, high-value single-packet detectors (recommend F2 implement first):**
- `0x08`/`0x0004` Force Listen Only → **T0814** (one-byte-pattern match, near-zero FP).
- `0x08`/`0x0001` Restart Comms → **T0814**.
- `0x08`/`0x000A` Clear Counters → anti-forensic flag (no clean single ATT&CK ID; treat as
  Evasion/Inhibit-Response indicator).
- Any WRITE FC → candidate **T0855** pending allowlist check.

### 5.1 "Rapid write burst" — quantitative threshold [JUDGMENT]

**Sources disagree** — this is explicitly a judgment call F2 must pin:
- Normal SCADA write rate is **low single digits per second**; brief legitimate bursts of
  ~10/s occur at process startup/shutdown.
- One source: sustained **>15 writes/s** is a defensible flag threshold.
- Another: sustained **>10/s**, or brief bursts **>25/s**, warrants investigation.

**Recommendation for F2 (state explicitly in spec):**
- Default flag: **> 10 write-FC requests/second sustained over a ≥2 s window**, OR a burst of
  **> 20 writes within any 1 s window**, to the same `(Unit ID, register/coil)` target.
- Make both numbers **config-tunable** with a documented default; per-environment calibration
  is expected. Tighten for writes to the *same* point (brute-force / T0806) vs writes spread
  across many points.
- Confidence: [JUDGMENT] — there is no spec-defined rate. Ship a default, log the violation
  with the observed rate, and let operators tune.

### 5.2 Function codes genuinely "unusual" on a normal SCADA segment [JUDGMENT]

A normal segment is **dominated by READ polling** (`0x03` Read Holding Registers, `0x01/0x02/0x04`).
Treat these as anomalous when seen, weighted by rarity:
- **Any WRITE** (`0x05/0x06/0x0F/0x10/0x16/0x17`) — uncommon vs reads; baseline the legitimate writers.
- **`0x08` Diagnostics** (esp. sub-funcs `0x0001/0x0004/0x000A`) — management/abuse, rarely in steady-state polling.
- **`0x11` Report Server ID** and **`0x2B`/MEI `0x0E` Read Device Identification** — **reconnaissance/fingerprinting**; unusual mid-operation.
- **`0x15` Write File Record**, **`0x14` Read File Record** — uncommon; flag.
- Exception bursts (§3) across many FCs/addresses — **scanning**.

---

## 6. MITRE ATT&CK for ICS vs Enterprise — representation note [MITRE]

**Confirmed:** ATT&CK for ICS is a **separate matrix** from Enterprise ATT&CK.
- **ICS technique IDs use the `T0xxx` namespace** (e.g. `T0855`), **distinct from Enterprise
  `Txxxx`** (e.g. `T1059`). They are not interchangeable.
- **Implication for wirerust's MITRE mapping type:** the type must be able to represent
  **`T0xxx`** IDs and tag which matrix a technique belongs to (ICS vs Enterprise). Do not
  assume a single `Txxxx` regex; ICS IDs are `T0` + 3 digits. A `matrix: enum { Enterprise, Ics }`
  discriminator (or an explicit namespace field) is the clean modeling choice for F2.
- **ICS tactics (12):** Initial Access, Execution, Persistence, Privilege Escalation, Evasion,
  Discovery, Lateral Movement, Collection, Command and Control, **Inhibit Response Function**,
  **Impair Process Control**, Impact. The last three (esp. the two bolded ICS-specific
  tactics) are where the Modbus techniques in §5 live. These tactic names do **not** exist in
  Enterprise ATT&CK.

---

## 7. Known False-Positive Considerations [JUDGMENT]

1. **Legitimate high-frequency polling.** Reads (`0x01–0x04`) at high rates are *normal* and
   must **not** trip write-burst detection. Gate the rate detector on **write FCs only**.
2. **Engineering-workstation commissioning bursts.** During commissioning/maintenance an
   engineering workstation issues legitimate write bursts (`0x06/0x10`) and recon (`0x11`,
   MEI `0x0E`). Mitigate with a **writer/host allowlist** and optional maintenance-window
   suppression. Flag writes from *unknown* hosts harder than from known engineering stations.
3. **Adaptive control writes without a human.** Some controllers legitimately auto-write
   setpoints. Baseline per-register write behavior rather than treating all parameter writes
   as T0836.
4. **Port-502-but-not-Modbus.** Other traffic can land on TCP 502. **Confirm Modbus before
   analyzing:** require `Protocol Identifier == 0x0000` **and** a **plausible `Length`
   (2…253)** **and** a known/plausible function code. If those fail, classify as
   non-Modbus-on-502 and **do not raise Modbus findings** (avoids misclassification). This
   three-point check (ProtoID + Length sanity + FC plausibility) is the cheapest reliable
   Modbus validity gate. [SPEC for the field constraints; JUDGMENT for using them as a gate.]
5. **Duplicate Transaction IDs / orphan responses** can be normal under heavy pipelining or
   packet loss — require a sustained pattern before flagging, not a single occurrence.

---

## Open Items / Where Sources Disagree (for F2 to resolve explicitly)

- **Write-burst rate threshold** (§5.1): sources span 10–25 writes/s. F2 must pick a tunable
  default. [JUDGMENT]
- **Port-direction reliability** (§4): destination-port=502 is the primary request signal but
  can be fooled by unusual port reuse; F2 should specify whether to fall back to txn-table
  correlation when direction is ambiguous. [JUDGMENT]
- **`0x08`/`0x000A` Clear Counters** has no single clean ATT&CK-for-ICS technique ID; modeled
  here as an anti-forensic/Evasion indicator. F2 should decide its finding category. [JUDGMENT]
- **`0x2B` (Encapsulated Interface Transport)** tunnels sub-protocols (CANopen `0x0D`, Read
  Device ID `0x0E`); depth of MEI inspection is a scope decision for F2.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | (1) MBAP header wire format & size bounds vs Modbus.org spec; (2) full function-code + diagnostics sub-function table vs spec V1.1b3; (3) exception response format/codes + request-vs-response disambiguation; (4) MITRE ATT&CK for ICS technique mapping, ICS-vs-Enterprise matrix, write-burst thresholds. All `reasoning_effort: high`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable (protocol spec, not a software library API). |
| Tavily | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 2 areas | Cross-check only: hex/code values and ATT&CK IDs were corroborated against model knowledge but every load-bearing fact is sourced to Modbus.org spec or MITRE ATT&CK for ICS above. |

**Total MCP tool calls:** 4 (all `perplexity_research`, `sonar-deep-research`, high effort).
**Training data reliance:** low — all byte layouts, FC/exception codes, and technique IDs are
verified against the Modbus.org V1.1b3 spec, Messaging Implementation Guide V1.0b, and MITRE
ATT&CK for ICS. The only non-spec content is the write-burst rate threshold (§5.1), which is
explicitly flagged [JUDGMENT] because authoritative sources disagree.

**Source citations** (from the four deep-research passes): Modbus.org
`modbusprotocolspecification.pdf` (V1.1b3, §4.2, §6), Modbus.org
`messagingimplementationguide.pdf` (V1.0b §3.1.3), MITRE ATT&CK for ICS
(T0855/T0836/T0814/T0806/T0835/T0831; ICS tactic list), Broadcom attack-signature ASID 20676
(Illegal Packet Size), en.wikipedia.org/wiki/Modbus (size cross-check),
wingpath.co.uk ModTest manual (PDU-limit history).
