---
document_type: feature-research
feature_id: feature-iec104
cycle: feature-iec104
title: "IEC 60870-5-104 Passive Analyzer — Domain & Framing Research"
producer: research-agent
created: 2026-07-13
base_commit: 7b11b83
branch: develop
status: complete
feeds:
  - IEC-104 passive analyzer feature (framing model, TypeID catalog, security flags)
  - ADR (future): IEC-104 stream dispatch & APCI/APDU two-layer parser design
  - mitre module (IEC-104 → ATT&CK for ICS technique mappings)
discipline: DF-VALIDATION-001 (every claim cited; library versions verified against registries; inconclusive items flagged)
---

# Research — IEC 60870-5-104 Passive Analyzer

Grounds a new **read-only / passive** IEC-104 analyzer in wirerust. wirerust decodes captured
pcap/pcapng traffic and emits security findings; it **never transmits packets**. Everything below
is framed for a passive dissector + detection engine, not an interactive IEC-104 stack.

> **Sourcing note.** IEC 60870-5-104 is a paywalled IEC standard; the authoritative wire-format
> facts below are cited to (a) a publicly hosted copy of the IEC-104 spec PDF, (b) the Wireshark
> dissector, (c) the widely-referenced `viduq/iec104-cheat-sheet`, and (d) vendor interoperability
> guides (Beckhoff, Kepware, Elseta, scadaprotocols.com). Vendor/community sources are flagged where
> they stand in for a primary standard. CVE/CWE/ATT&CK claims are cited to NVD/CISA/MITRE. Findings
> are **as of 2026-07-13**; the ICS threat and CVE landscape drifts. Any inference not directly
> supported by a source is marked **[inference]** or **[inconclusive]**.

---

## 1. Framing / Wire Format

IEC-104 is a **two-layer application framing** over a raw TCP byte stream: an outer **APCI**
(Application Protocol Control Information) that every frame carries, wrapping an inner **ASDU**
(Application Service Data Unit) that only I-format frames carry. The APCI+ASDU together form an
**APDU** (Application Protocol Data Unit). [S1][S2][S9]

### 1.1 APCI structure (present on every frame)

```
 +--------+--------+--------+--------+--------+--------+
 | 0x68   | LEN    | CF1    | CF2    | CF3    | CF4    |   (+ ASDU if I-format)
 +--------+--------+--------+--------+--------+--------+
   start    length     4 control-field octets
```

- **Start byte = `0x68`** — every APDU begins with `0x68`; it is the frame-sync anchor a passive
  parser scans for. [S1][S2][S3]
- **Length octet (LEN)** — counts **all bytes that follow the length octet**, i.e. the **4 control
  octets + the entire ASDU**. It **excludes** the start byte and the length octet itself. [S1][S3]
  - Consequence for a parser: total on-wire frame size = `LEN + 2`.
  - **LEN range: 4 … 253.** Minimum 4 = an S- or U-format frame (4 control octets, no ASDU).
    Maximum 253 ⇒ max APDU = 255 bytes total. A passive analyzer should flag `LEN < 4` or
    `LEN > 253` as a **malformed length octet**. [S1][S3] **[inference on exact bounds from LEN
    semantics; the 255-byte APDU ceiling is stated in the spec]**
- **4 control-field octets (CF1–CF4)** — encode the frame format and sequence numbers (below).

### 1.2 The three frame formats — discrimination from CF1 low bits

The frame format is decided by the **two least-significant bits of the first control octet (CF1)**:
[S1][S4][S11]

| Format | CF1 bit 0 | CF1 bit 1 | Meaning |
|--------|-----------|-----------|---------|
| **I-format** (Information transfer) | `0` | — | Carries an ASDU (data). Last bit 0. |
| **S-format** (Supervisory) | `1` | `0` | Ack-only (numbered supervision). Low bits `01`. |
| **U-format** (Unnumbered control) | `1` | `1` | STARTDT / STOPDT / TESTFR. Low bits `11`. |

Discrimination rule for a dissector: test `CF1 & 0x01`; if 0 → **I**; else test `CF1 & 0x03`
→ `0x01` = **S**, `0x03` = **U**. [S1][S4][S11]

**U-format function encoding (CF1 value; CF2–CF4 = 0x00).** The U-format sets exactly one function
bit-pair. Well-known CF1 values (used by Wireshark and the cheat-sheet): [S4][S11]

| U-function | CF1 value |
|------------|-----------|
| STARTDT act | `0x07` |
| STARTDT con | `0x0B` |
| STOPDT act | `0x13` |
| STOPDT con | `0x23` |
| TESTFR act | `0x43` |
| TESTFR con | `0x83` |

A passive analyzer can treat any U-frame whose CF1 is **not** one of these six as a
**reserved/invalid U-format bit pattern** (candidate finding). [S4][S11] **[inference — flagging
non-canonical U patterns is a defensible detection, not a standard-mandated one]**

### 1.3 Send/receive sequence numbers — N(S)/N(R)

Both N(S) and N(R) are **15-bit** counters (modulo 32768), packed across two octets with the LSB of
the low octet reserved for the format bit. [S1][S4][S11]

- **I-format:** `N(S)` in CF1–CF2, `N(R)` in CF3–CF4.
  - `N(S) = (CF1 >> 1) | (CF2 << 7)` — CF1 bit 0 is the I-format marker (0).
  - `N(R) = (CF3 >> 1) | (CF4 << 7)` — CF3 bit 0 is reserved (0).
- **S-format:** only `N(R)` present, in CF3–CF4 (same layout); CF1=`0x01`, CF2=`0x00`.
- **U-format:** no sequence numbers.

These are **flow-control / retransmission** counters, **not** cryptographic freshness — they give
no replay protection. [S1][SEC15]

### 1.4 ASDU structure (inside I-frames only)

The ASDU = a 6-octet **Data Unit Identifier (DUI)** + one or more Information Objects. [S1][S4][SC]

| Field | Size (IEC-104) | Notes |
|-------|----------------|-------|
| **Type Identification (TypeID)** | 1 octet | Semantic type of the objects (see §2). |
| **Variable Structure Qualifier (VSQ)** | 1 octet | bit 7 = **SQ** (sequence vs. individual addressing); bits 0–6 = **number of objects** (0–127). |
| **Cause of Transmission (COT)** | **2 octets** | byte 1: bits 0–5 = 6-bit **cause**; bit 6 = **P/N** (positive/negative confirm); bit 7 = **T** (test). byte 2 = 8-bit **Originator Address (OA)**. |
| **Common Address of ASDU (CASDU)** | **2 octets** | station/sector address (little-endian). |
| **Information Object Address (IOA)** | **3 octets** | per information object (little-endian). |

The IEC-104 field widths are **fixed** (COT=2, CASDU=2, IOA=3) — unlike the serial IEC-101 profile
where they are configurable. This is stated in the interoperability spec. [S1][SC] The COT test bit
(T) and P/N bit are directly security-relevant (§3). [S1][SC]

### 1.5 Transport: TCP, port 2404, stream reassembly

- **Default port: 2404/TCP** — IANA-registered `iec-104` (TCP and UDP; UDP is rare in practice).
  [S8-IANA (from sibling coverage research)][S2]
- IEC-104 runs over a **raw TCP byte stream**. TCP segment boundaries do **not** align with APDU
  boundaries. A passive analyzer MUST perform **per-flow reassembly**:
  - **Multiple APDUs per TCP segment** (walk `0x68`/LEN framing repeatedly within one segment).
  - **An APDU may span multiple segments** (carry-buffer a partial frame across `on_data` calls).
  This is structurally identical to wirerust's existing DNP3 / ENIP / TLS carry-buffer design
  (§5). [S1][S2] — mirrors ADR-0007 (DNP3) and ADR-0010 (ENIP) reassembly patterns.

---

## 2. Key TypeIDs (monitoring vs. control)

TypeIDs are shared between IEC-104 and IEC-101 (same numeric registry). Decimal values below are
cross-checked across Beckhoff, Elseta, scadaprotocols.com, and the OpenMUC j60870 enum. [T1][T7][T9]

### 2.1 Monitoring direction (M_* — telemetry, low security priority to *flag*)

| TypeID | Mnemonic | Meaning |
|--------|----------|---------|
| 1 | M_SP_NA_1 | Single-point information |
| 3 | M_DP_NA_1 | Double-point information |
| 5 | M_ST_NA_1 | Step position information |
| 9 | M_ME_NA_1 | Measured value, normalized |
| 11 | M_ME_NB_1 | Measured value, scaled |
| 13 | M_ME_NC_1 | Measured value, short float |

Monitoring objects are **reports**, not actions. They matter for spoofed-telemetry detection
(§3, T0852) but are not operator control actions.

### 2.2 Control direction (C_* — commands) — **the security-relevant set**

| TypeID | Mnemonic | Meaning | Operator control action on grid equipment? |
|--------|----------|---------|--------------------------------------------|
| **45** | **C_SC_NA_1** | Single command | **YES** — e.g. open/close a switch or breaker |
| **46** | **C_DC_NA_1** | Double command | **YES** — breaker open/close (2-bit, anti-ambiguity) |
| **47** | **C_RC_NA_1** | Regulating step command | **YES** — tap-changer / raise-lower regulation |
| **48** | **C_SE_NA_1** | Set-point, normalized | **YES** — set-point write |
| **49** | **C_SE_NB_1** | Set-point, scaled | **YES** — set-point write |
| **50** | **C_SE_NC_1** | Set-point, short float | **YES** — set-point write |
| 51 | C_BO_NA_1 | Bitstring 32-bit command | **YES** — bulk output write **[inference on 51; sequential allocation]** |
| **100** | **C_IC_NA_1** | (General) interrogation command | System — reconnaissance-relevant, not a physical action |
| **101** | **C_CI_NA_1** | Counter interrogation command | System |
| 102 | C_RD_NA_1 | Read command | System (read) |
| **103** | **C_CS_NA_1** | Clock synchronization | System — **time manipulation** (see §3) |
| **105** | **C_RP_NA_1** | Reset process command | **YES** — forces a process/RTU reset (disruptive) |
| 107 | C_TS_TA_1 | Test command with time tag (CS104) | System (test) |

> **Note on C_TS:** the older `C_TS_NA_1` = 104 exists in the IEC-101 fixed-frame world; CS104 uses
> `C_TS_TA_1` = 107. The `perplexity_ask` pass returned "107/108" for test commands; treat the exact
> C_TS numbering as **[inconclusive]** and confirm against the spec table before hard-coding. [T9]

### 2.3 File transfer (F_* — 120–126 range)

| TypeID | Mnemonic | Meaning |
|--------|----------|---------|
| 120 | F_FR_NA_1 | File ready |
| 121 | F_SR_NA_1 | Section ready |
| 122 | F_SC_NA_1 | Call directory / select / call file / call section |
| 123 | F_LS_NA_1 | Last section / last segment |
| 124 | F_AF_NA_1 | Ack file / ack section |
| 125 | F_SG_NA_1 | Segment |
| 126 | F_DR_TA_1 | Directory (with time tag) |

Values 120–124 are consistently cited; **125/126 are documented mainly in extended vendor tables**
and should be treated as **[medium-confidence]** until spec-verified. [T7][T9] File transfer is
security-relevant (config/firmware movement channel) but is a secondary detection surface.

### 2.4 Control-action TypeIDs to surface (summary)

**Physical/disruptive operator actions:** `C_SC_NA_1 (45)`, `C_DC_NA_1 (46)`, `C_RC_NA_1 (47)`,
`C_SE_NA_1/NB_1/NC_1 (48/49/50)`, `C_BO_NA_1 (51)`, `C_RP_NA_1 (105)`.
**System/time control (elevated interest):** `C_IC_NA_1 (100)`, `C_CI_NA_1 (101)`, `C_CS_NA_1 (103)`.

---

## 3. Security behaviors a passive analyzer should flag

IEC-104 has **no authentication, no integrity, and no encryption by default** — it relies entirely on
TCP/IP lower layers. This is the root cause of nearly every finding below. [SEC1][SEC3][SEC7][SEC15]
(IEC 62351-3/-5 add TLS + application auth but are rarely deployed and out of scope for a passive
plaintext dissector — their absence is itself a finding.) [SEC-CWE311/319]

### 3.1 Unauthenticated control commands (highest priority)

Any observed **control-direction command** (`C_SC/C_DC/C_RC/C_SE/C_BO/C_RP`) is by definition
unauthenticated. A passive analyzer should surface control commands as security-relevant events and
correlate on volume/timing (e.g. burst of `C_SC` open/close, or a `C_RP` reset).
Maps to **MITRE ATT&CK for ICS T0855 Unauthorized Command Message**. [SEC1][SEC7][ATT-T0855]

### 3.2 Malformed / anomalous framing

- **Malformed length octet:** `LEN < 4` or `LEN > 253`, or `LEN` inconsistent with the parsed ASDU.
- **Invalid/reserved TypeID:** TypeIDs in reserved ranges (e.g. 22–29, 41–44, 52–57, 66–69, 71–99
  gaps, 109–119, 127+) — reserved-range use is anomalous and a fuzzing/exploit indicator. [T9]
  **[inference on exact reserved gaps — verify against the spec's TypeID table before hard-coding.]**
- **COT anomalies:** an out-of-range COT for a given TypeID, or a **COT test bit (T=1)** on
  production traffic, or a **negative confirmation (P/N=1)** on a command (rejection). [S1][SC]
- **Spontaneous vs. interrogated mismatch:** e.g. a monitoring value arriving with COT=spontaneous(3)
  when the flow context implies interrogation(20), or vice-versa — indicates injection/spoofing.
  **[inference — a defensible heuristic; not a standard-defined error]**

### 3.3 Connection-control (U-format) sequencing anomalies

- **STARTDT/STOPDT sequencing:** I-format data observed **before** a STARTDT act/con handshake, or
  a STOPDT injected mid-session to disrupt data transfer. Abuse maps to **T0809 Service Stop /
  T0829 Impair Process Control**. [SEC1][SEC8][ATT-T0809]
- **Non-canonical U-frame CF1** (not one of the six values in §1.2) — reserved-bit misuse. This class
  is directly tied to **CVE-2026-1773** (DoS on invalid U-format frames, **CWE-184**). [SEC-CVE1773]

### 3.4 Sequence-number (N(S)/N(R)) desync, rollover, window violations

- **N(S)/N(R) desynchronization:** a received N(R) that does not acknowledge outstanding N(S)es, or
  N(S) jumping non-monotonically — indicates injection, MITM, or replay.
- **15-bit rollover** at 32768 — legitimate but a boundary a naive parser can mishandle; flag
  suspicious resets to 0.
- **k / w flow-control window violations:** default **k = 12** (max unacknowledged I-frames sent) and
  **w = 8** (ack after w received I-frames). More than **k** unacknowledged I-frames in flight, or an
  ack cadence that ignores **w**, is a protocol-state violation worth flagging. Standard timers:
  **t0 = 30s, t1 = 15s, t2 = 10s, t3 = 20s**. [S-KW10][S-KW12][S-KW21]

### 3.5 Replay & spoofing

- **Replay:** IEC-104 sequence numbers are flow-control, not freshness → replayed command/telemetry
  ASDUs are accepted. Detect duplicate command ASDUs (same TypeID/IOA/COT/payload). Maps to
  **CWE-294 Authentication Bypass by Capture-Replay**; behaviorally part of **T0855 / T0852**.
  [SEC15][SEC18][CWE294]
- **Spoofed telemetry:** injected M_* reports to mask real state → **T0852 Spoof Reporting Message**.
  [SEC3][ATT-T0852]

### 3.6 Known CVEs (implementation-level, exploitable via IEC-104 traffic)

| CVE | Context | Issue | CWE |
|-----|---------|-------|-----|
| **CVE-2026-1773** | IEC-104 impl (RTU500 & others), bidirectional config | **DoS on invalid U-format frames** | **CWE-184** Incomplete List of Disallowed Inputs [SEC-CVE1773] |
| **CVE-2022-2502** | Hitachi Energy RTU500 HCI IEC-104 (+ IEC 62351-5) | Buffer overflow / CMU reboot on crafted IEC-104 message | Buffer error (CWE-119/120-class) [SEC-CVE2502] |
| **CVE-2019-6831** | Schneider Modicon Eth/Serial RTU (2404/TCP) | Connection drop on high IEC-104 packet volume | **CWE-754** Improper Check for Exceptional Conditions [SEC-CISA] |
| **CVE-2019-6810** | Schneider Modicon Eth/Serial RTU | Unauthorized command execution via IEC-104 | **CWE-284** Improper Access Control [SEC-CISA] |

> These target **products** implementing IEC-104, not the spec itself, but each is exploitable via
> on-wire IEC-104 traffic and validates the framing-anomaly detection surface above. [SEC-CVE1773]
> [SEC-CVE2502][SEC-CISA]

### 3.7 CWE reference set (design/systemic)

- **CWE-306** Missing Authentication for Critical Function (control commands / STARTDT-STOPDT).
- **CWE-319** Cleartext Transmission of Sensitive Information (plaintext ASDUs).
- **CWE-311** Missing Encryption of Sensitive Data.
- **CWE-294** Authentication Bypass by Capture-Replay.
- **CWE-184** Incomplete List of Disallowed Inputs (malformed U-frames → CVE-2026-1773).
[SEC-CWE-set]

### 3.8 MITRE ATT&CK for ICS mapping (for wirerust's `mitre` module)

| IEC-104 behavior | Technique |
|------------------|-----------|
| Control command injection (C_SC/C_DC/C_RC/C_SE/C_BO/C_RP) | **T0855** Unauthorized Command Message |
| Spoofed / replayed telemetry (M_*) | **T0852** Spoof Reporting Message |
| Set-point / parameter write (C_SE, C_BO) | **T0831** Modify Parameter |
| Physical process manipulation via commands | **T0830** Manipulation of Control |
| STOPDT abuse / malformed-U DoS / flooding | **T0809** Service Stop / **T0829** Impair Process Control |
| MITM enabling the above | **T0830/T0855** (MITM as enabler; see also T0842 Network Sniffing for the plaintext exposure) |

[ATT-T0855][ATT-T0852][ATT-T0809] — **verify each technique ID against the live MITRE ATT&CK for ICS
matrix** at integration time; ATT&CK IDs are re-versioned periodically. **[flag]**

---

## 4. Reference implementations (learn framing from — do NOT copy)

| Implementation | License | Role | Framing edge cases worth studying |
|----------------|---------|------|-----------------------------------|
| **Wireshark `epan/dissectors/packet-iec60870.c`** | **GPL-2.0-or-later** [R-WS] | Passive dissector (closest analog to wirerust) | Buffer-length guards before field access; I/S/U discrimination; U-format function decode; TCP reassembly via `tcp_dissect_pdus`. **License note: GPLv2 — do not copy code into MIT/Apache wirerust; study behavior only.** |
| **MZ Automation `lib60870-C`** | **GPL-3.0** (commercial dual-license via MZ) [R-LIB] | Full production stack | Field-proven length validation, sequence/window (k/w) state machine, canonical U-format handling. GPLv3 — reference only. |
| **Rust `iec60870-5` (crates.io)** | **NON-FREE custom license — "NOT FREE for any commercial or production use"** [R-RUST] | Transport-agnostic telegram codec | Rust-safe I/S/U + telegram types. **DO NOT take as a dependency** — the license forbids production use. (This corrects an earlier tool response that inferred "MIT"; that inference was wrong — see §6.) |
| Rust `lib60870` bindings (docs.rs) | Wraps GPL-3.0 lib60870-C ⇒ effectively **GPLv3** [R-RUSTBIND] | FFI bindings | Inherits lib60870-C framing; GPLv3 taint. Reference only. |
| Go `xgbt/go-iec104` | **MIT** (pkg.go.dev metadata) [R-GO1] | Pure-Go stack | APDU framing, I/S/U, seq numbers. Permissive — safe to read for design. |
| Go `wendy512/iec104` | **Apache-2.0** [R-GO2] | Client | Client-side framing/U-format. Permissive. |
| Go `shangfabao/go-iecp5` | **LGPL-3.0** [R-GO3] | CS104 | References lib60870. |
| Python `Fraunhofer-FIT-DIEN/iec104-python` (`c104`) | **GPL-3.0** [R-PY] | Simulation (pybind11 over lib60870-C) | Full framing via lib60870-C. GPLv3. |

**Licensing takeaway:** the two most instructive references (Wireshark, lib60870-C) are **GPL** and the
Rust `iec60870-5` crate is **non-free**. wirerust must implement its own parser from the **spec + the
permissively-licensed Go implementations (MIT/Apache) as design references** and must **not** vendor or
depend on any of the GPL/non-free code. Study framing *behavior*, write original Rust. [R-WS][R-RUST]

---

## 5. Comparison to DNP3 & Modbus (already in wirerust)

### 5.1 Structurally similar (reuse existing patterns)

- **Port-based TCP stream dispatch.** Like Modbus (502, ADR-0005) and DNP3 (20000, ADR-0007), IEC-104
  has a well-known port (**2404**) and no reliable content signature for early bytes beyond the `0x68`
  start marker. Add an IEC-104 **port-2404 dispatch rule** appended after content-signature rules,
  preserving the VP-004 port-precedence invariant — exactly the DNP3 Rule-6 pattern. [ADR-0007 D1]
- **Per-flow carry buffer + frame-walk.** IEC-104 needs the same "accumulate bytes, walk complete
  frames, stash residual partial frame" loop as DNP3's `carry_c2s/carry_s2c` design, bounded to a
  fixed cap (IEC-104 max APDU = **255 bytes**, so a 255-byte directional carry cap). Directional carry
  separation (DRIFT-DNP3-DIRECTION-001) applies identically. [ADR-0007 D2]
- **Direction-threaded `on_data`.** Master↔RTU direction resolution mirrors DNP3's direction-based
  source-IP resolution (RULING-DNP3-SIBLING-001) and Modbus's client/server pattern — client→server =
  controlling station (initiates to 2404), server→client = controlled station/RTU. [ADR-0007 D3]
- **Bounded per-flow state.** Same discipline as DNP3 Decision 4: cap tracked addresses, pending
  command tables, findings, and use `saturating_sub` windowed counters with strict `>` expiry.
  [F2 Pass-9 descope note: the MVP descoped full windowed detection to a simple N(S)-gap > k=12
  desync check (BC-2.19.024); no time-window / `saturating_sub` / `window_start_ts` state exists
  in the delivered `Iec104FlowState`. The carry-cap discipline (bounded `Vec<u8>`) is real; the
  time-windowed counter pattern described here was not implemented. See ADR-013 Decision 2/6.]
- **Unauthenticated-command threat model.** Like Modbus (write coils/registers) and DNP3 (control
  relay output block), IEC-104 control ASDUs are the security payload → T0855 correlation reuse.

### 5.2 Genuinely new (design work required)

- **Two-layer APCI/APDU framing.** Modbus/TCP has a single MBAP header; DNP3 has a data-link header +
  transport + application layers but a *single* framing pass. IEC-104 introduces an **outer APCI that
  is itself a discriminated union** (I vs S vs U) — the parser must first classify the frame *format*
  before it even knows whether an ASDU exists. This U/S/I discrimination on CF1 low bits has **no
  analog** in the existing analyzers.
- **Connection-control state machine (STARTDT/STOPDT/TESTFR).** Neither Modbus nor DNP3 has an
  explicit application-layer session start/stop handshake. IEC-104's U-format handshake is a new
  stateful surface (data-transfer-enabled vs -disabled) that unlocks a whole finding class (§3.3).
- **15-bit dual sequence numbers with k/w windowing.** DNP3 has sequence/confirm but not the IEC-104
  k=12/w=8 sliding-window acknowledgement scheme. Window-violation detection (§3.4) is new logic.
- **No CRC in the APDU.** DNP3 carries block CRCs (ADR-0007 D3 counts-but-doesn't-verify them);
  IEC-104 has **no application-layer checksum** (TCP-only integrity), so there is no CRC accounting in
  the frame-length arithmetic — frame length is simply `LEN + 2`. Simpler length math than DNP3's
  `10 + ceil((LEN-5)/16)*18`, but zero integrity signal to leverage.

---

## 6. Inconclusive / flagged items (DF-VALIDATION-001)

- **`iec60870-5` crate license — RESOLVED but corrects a tool error.** crates.io API reports the
  license as "Non-standard"; lib.rs states verbatim it **"IS NOT FREE for any commercial or production
  use."** An intermediate `perplexity_ask` response *inferred* "MIT" — that inference was **wrong**.
  Treat this crate as **non-free / unusable as a dependency**. Confidence: **high** (two registry
  sources agree). [R-RUST]
- **Exact C_TS numbering (104 vs 107/108)** — **[inconclusive]**; IEC-101 fixed-frame vs CS104 differ.
  Verify against the spec TypeID table before hard-coding a test-command detection.
- **File-transfer TypeIDs 125/126 (F_SG_NA_1, F_DR_TA_1)** — **[medium confidence]**; documented in
  extended vendor tables, not confirmed against a primary source here.
- **Reserved-TypeID gap ranges (§3.2)** — **[inference]**; the precise reserved ranges must be read
  from the spec's TypeID allocation table, not assumed from sequential gaps.
- **LEN bounds (4…253)** — derived from APCI semantics + the 255-byte APDU ceiling; the ceiling is
  spec-stated, the exact min/max are **[inference]** and should be confirmed.
- **MITRE technique IDs** — re-verify against the live ATT&CK for ICS matrix at implementation time.
- **Primary IEC standard** — no paywalled IEC 60870-5-104:2006 copy was purchased; wire-format facts
  rest on a publicly hosted spec PDF [S1], Wireshark, and corroborating vendor guides. For a formal
  spec citation in an ADR, cite **IEC 60870-5-104** by number and note the secondary corroboration.

---

## Recommended detection candidates (priority-ordered for the analyzer)

1. **Any control command observed** (C_SC/C_DC/C_RC/C_SE/C_BO/C_RP) → T0855 finding (INFO→WARN by
   volume/timing). *(highest signal, lowest false-positive risk)*
2. **Malformed length octet / invalid TypeID / reserved-range TypeID** → malformed-frame finding
   (fuzzing/exploit indicator; ties to CVE-2026-1773 class).
3. **Non-canonical U-format CF1** and **STARTDT/STOPDT sequencing anomalies** → T0809/T0829.
4. **COT test bit (T=1) on production traffic** and **negative-confirm (P/N=1) on commands**.
5. **Sequence desync / k-window overrun** → injection/MITM indicator.
6. **Replayed command ASDU** (duplicate TypeID/IOA/COT/payload) → CWE-294 / T0855.
7. **Clock-sync command (C_CS_NA_1)** → time-manipulation watch (elevated interest).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of IEC-104 wire format, TypeIDs, security, and reference parsers (reasoning_effort=high). Output (105 KB) captured to a tool-results file; citation set extracted via Grep — see note below. |
| Perplexity perplexity_ask | 4 | Focused, short-output verification of: (a) APCI framing + N(S)/N(R) bit layout; (b) exact numeric TypeIDs; (c) security attack patterns + CVEs + CWEs + ATT&CK IDs; (d) ASDU field octet sizes + U-format functions + k/w + t0–t3 timers; (e) reference-implementation licenses. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | Not used — the load-bearing library fact (crate license) was verified directly against crates.io API + lib.rs, which is more authoritative for license/version than doc snippets. |
| WebFetch | 3 | crates.io API (iec60870-5 version list = 0.2.1 latest; license "Non-standard"); lib.rs (license verbatim: "NOT FREE for commercial/production use" — corrected the ask's MIT inference); (prior-report + template reads were local file Reads). |
| WebSearch | 0 | — |
| Read (local) | 4 | Research index (absent), prior sibling research (house style), ADR-0007 (DNP3 comparison), glob of phase-f1 dir. |
| Training data | ~2 areas | U-format CF1 hex values (0x07/0x0B/0x13/0x23/0x43/0x83) and reserved-TypeID gap intuition — both flagged **[inference]** and cross-checked against cited cheat-sheet where possible. |

**Total MCP tool calls:** 5 Perplexity (1 research + 4 ask) + 3 WebFetch = 8.
**Training data reliance:** **low** — every load-bearing numeric fact (start byte, LEN semantics, CF1
discrimination, N(S)/N(R) width, TypeIDs, k/w/timers, CVEs, CWEs, ATT&CK, licenses) is web-cited;
the crate license was double-verified against two registries and a tool-inference error was caught
and corrected. Items resting on inference or single sources are explicitly flagged in §6.

> **Extraction note (transparency):** the primary `perplexity_research` call succeeded but returned
> ~105 KB, exceeding the tool-result inline limit; it was saved to a tool-results file. That file is a
> single physical JSON line, so ripgrep's long-line guard prevented reading the prose body inline. The
> **citation URL set was extracted successfully** (20 sources incl. the IEC-104 spec PDF, Wireshark
> mirror, ceur-ws/utwente/royalholloway security papers, MITRE ATT&CK T1692, CWE-294, crates.io). To
> avoid relying on an unread body, the four `perplexity_ask` calls independently re-derived and
> re-cited every fact used in this report. No claim here rests on the unread portion of the deep-research
> body.

---

## Citation key

- **[S1]** IEC 60870-5-104 spec (publicly hosted PDF): cdn.standards.iteh.ai/samples/5990/.../IEC-60870-5-104-2000.pdf
- **[S2]** scadaprotocols.com — "How IEC 60870-5-104 works over Ethernet" / ASDU structure
- **[S3]** scadaprotocols.com — IEC-104 over Ethernet (LEN semantics)
- **[S4][S11]** github.com/viduq/iec104-cheat-sheet ; escholarship.org qt6mq5m039 (control-field bits)
- **[S9]** infosys.beckhoff.com TF6500 IEC 60870-5-10x (framing)
- **[SC]** scadaprotocols.com/iec104-asdu-structure ; vattenfalleldistribution VTR04 (DUI field sizes)
- **[S-KW10/12/21]** filedn.eu CE_HELP RTU docs ; support.ptc.com Kepware IEC-104 Interoperability Guide ; docs.oracle.com NMS SCADA (k/w, t0–t3)
- **[T1][T7][T9]** infosys.beckhoff.com (M_* TypeIDs) ; scadaprotocols.com/iec104-asdu-structure ; wiki.elseta.com IEC 60870-5-104 ; openmuc.org j60870 ASduType javadoc
- **[SEC1]** ceur-ws.org/Vol-2874/paper13.pdf (IEC-104 attacks)
- **[SEC3]** infonomics-society.org — Passive Security Monitoring for IEC 60870-5-104 SCADA
- **[SEC7]** Radoglou et al. 2019 — Attacking IEC 60870-5-104 SCADA Systems
- **[SEC8]** cytal.co.uk/protocols/iec-60870-5-104
- **[SEC15]** scirp.org jcc 2022011214435437 (replay/no-auth)
- **[SEC18]** ids.uni-bremen MOCAST 2020 (MITM + injection)
- **[SEC-CVE1773]** sentinelone / tenable / askarlabs — CVE-2026-1773 (CWE-184, DoS invalid U-frames)
- **[SEC-CVE2502]** nvd.nist.gov CVE-2022-2502 (RTU500 buffer overflow)
- **[SEC-CISA]** cisa.gov ICSA-20-044-01 (CVE-2019-6831 CWE-754, CVE-2019-6810 CWE-284)
- **[CWE294]** cwe.mitre.org/data/definitions/294.html
- **[ATT-T0855/T0852/T0809]** attack.mitre.org ATT&CK for ICS ; T1692 (referenced)
- **[R-WS]** github.com/wireshark/wireshark (GPLv2+) ; boundary/wireshark packet-iec104.c mirror
- **[R-LIB]** github.com/mz-automation/lib60870 user_guide (GPLv3)
- **[R-RUST]** crates.io/api/v1/crates/iec60870-5 (v0.2.1, "Non-standard") ; lib.rs/crates/iec60870-5 ("NOT FREE for commercial/production use")
- **[R-RUSTBIND]** docs.rs/lib60870
- **[R-GO1]** pkg.go.dev/github.com/xgbt/go-iec104 (MIT)
- **[R-GO2]** pkg.go.dev/github.com/wendy512/iec104 (Apache-2.0)
- **[R-GO3]** libraries.io go-iecp5 (LGPL-3.0)
- **[R-PY]** github.com/Fraunhofer-FIT-DIEN/iec104-python (GPLv3)
