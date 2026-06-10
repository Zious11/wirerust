---
document_type: feature-delta-analysis
feature_id: issue-008-dnp3-analyzer
github_issue: 8
title: "Add DNP3 protocol analyzer"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  New module (src/analyzer/dnp3.rs, ~600-900 LOC), dispatcher surgery
  (new DispatchTarget::Dnp3 variant + port-20000 classify branch), mitre.rs
  catalog extension (T0803 is NEW — must seed+emit), new subsystem SS-15,
  new VP-023, new ADR for DNP3 integration decisions. Minimum 6 source files
  changed plus factory spec artifacts. Not a trivial parameter tweak.
scope_classification: standard
status: draft
producer: architect
created: 2026-06-10
base_commit: fb2c875
branch: develop
prior_feature_precedent: issue-007-modbus-analyzer (D-032..D-046)
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
  - .factory/STATE.md (D-032..D-046 Modbus precedents)
---

# F1 Delta Analysis — Issue #8: Add DNP3 Protocol Analyzer

## 1. Feature Summary

Implement a DNP3 (Distributed Network Protocol 3 / IEEE 1815) analyzer for
ICS/OT forensics. Per the issue the analyzer must:

- Detect DNP3 on port 20000 (TCP/UDP stated).
- Parse the DNP3 data-link layer header: start bytes 0x0564, length byte,
  control byte, destination address (u16), source address (u16).
- Parse transport layer (transport header byte) and application layer function
  codes (application control + function code byte).
- Track: read vs write operations, direct operate commands, cold/warm restarts.
- Detect anomalies: unauthorized control commands, broadcast messages,
  unsolicited responses.
- Map to MITRE ICS: T0855 (Unauthorized Command Message — already emitted by
  Modbus), T0803 (Block Command Message — NEW to the catalog).
- Summary: function code distribution, control operation counts.

---

## 2. Intent and Scope Classification

| Field | Value |
|-------|-------|
| Intent | feature (new capability; no prior DNP3 analysis exists) |
| Feature type | backend |
| Trivial | NO |
| Trivial justification | New module + dispatcher structural change + mitre.rs catalog extension + new VP + new subsystem BCs + ADR; minimum 6 source files changed |
| Scope classification | standard — full F1-F7 cycle required |
| Recommended path depth | Full F1-F7 (identical to Modbus Feature #7) |
| Recommended subsystem | SS-15 (new; Modbus took SS-14) |

**Rationale for full F1-F7:** DNP3 is the most complex binary ICS protocol in
the issue backlog — layered header stack (data-link, transport, application),
CRC-16/DNP every 16 bytes, stateful fragment reassembly for multi-frame
application messages, and a larger function code space than Modbus. The
formal-verification target (pure-core parse function) is a Kani candidate
with meaningful state bounds. This is not a trivial protocol add.

---

## 3. Integration Architecture Decision (CRITICAL)

### Issue Statement

GitHub #8 says "implements ProtocolAnalyzer" and "port 20000 TCP/UDP". Two
integration paths exist in the codebase:

**Path A — StreamHandler + StreamDispatcher (Modbus precedent)**
- Used by: ModbusAnalyzer (Feature #7), HttpAnalyzer, TlsAnalyzer.
- Receives reassembled TCP stream bytes via `StreamHandler::on_data`.
- Wired into `StreamDispatcher` via a new `DispatchTarget` variant.
- Entry point: `src/dispatcher.rs` classify() port-fallback rule (currently
  Rules 1-5; DNP3/TCP would be Rule 6).

**Path B — ProtocolAnalyzer / packet-level (DNS precedent)**
- Used by: DnsAnalyzer only.
- Receives each raw `ParsedPacket` from the packet loop in `src/main.rs`.
- Sees both TCP and UDP packets.
- Does NOT use reassembly; processes packet-at-a-time.

### Concrete Analysis Against the Codebase

`src/analyzer/dns.rs` implements `ProtocolAnalyzer::can_decode` by checking
`TransportInfo::Udp` and `TransportInfo::Tcp` ports directly (dns.rs:42-67).
It is invoked in the main packet loop (main.rs:200-203) BEFORE the reassembler.
No carry buffer, no stream state, packet-level only.

`src/analyzer/modbus.rs` implements `StreamHandler::on_data`. It carries an
explicit per-flow carry buffer (`flow.carry: Vec<u8>`) to handle ADU spanning
across TCP segment boundaries. The dispatcher's `classify()` matches port 502
as Rule 5 (after all content rules). ModbusAnalyzer owns its flow HashMap and
is passed to `StreamDispatcher::new()`.

**DNP3 data-link frames are exactly the same problem class as Modbus ADUs:**
fixed-length framing (10-byte data-link header, body, CRC fields), carried over
a TCP byte stream. DNP3 application messages are further fragmented across
multiple transport-layer frames — each transport frame up to 249 bytes of
application data. Cross-segment framing is guaranteed in real captures.

The `ProtocolAnalyzer` (DNS) path cannot handle:
1. Carry buffers for partial frames spanning TCP segments.
2. Fragment reassembly for multi-transport-frame application messages.
3. Correct direction tracking (ClientToServer vs ServerToClient) derived from
   TCP flow state — needed for "who sent the unauthorized command".

**RECOMMENDATION: TCP-only via StreamHandler + StreamDispatcher (mirror Modbus)**

This mirrors D-032 exactly: DNP3/TCP should implement `StreamHandler` and be
wired into `StreamDispatcher` with a `DispatchTarget::Dnp3` variant and a new
port-20000 classify rule (Rule 6 after port 502/Rule 5).

### HUMAN SCOPE DECISION #1: TCP-only or TCP+UDP dual-path

**Context:** DNP3 is defined for TCP (port 20000, most deployments), UDP
(port 20000, some older/constrained devices), and serial. The GitHub issue
mentions both TCP and UDP. UDP DNP3 is connectionless, has no reassembly
concerns, and does not need a carry buffer. However:

- The `ProtocolAnalyzer` path (DNS UDP model) processes per-packet and would
  work for UDP-only DNP3 frames that fit in one packet.
- DNP3 application messages > 249 bytes require transport-layer fragmentation
  even in UDP; a per-packet analyzer cannot reassemble them.
- In practice, ICS forensics PCAPs for DNP3 are overwhelmingly TCP. UDP DNP3
  on port 20000 is rare in SCADA deployments; most modern masters/outstations
  use TCP.
- Implementing dual-path (StreamHandler for TCP + ProtocolAnalyzer for UDP)
  adds significant complexity with minimal practical value for v1.

**Recommendation:** TCP-only for v1 (mirror Modbus). Document UDP as a v2
enhancement. If the human disagrees and wants UDP: add a `ProtocolAnalyzer`
path for UDP-only DNP3 in the same feature cycle using the DNS pattern, with
limited detection (no application-layer reassembly, data-link parse only).

---

## 4. DNP3 Protocol-Correctness Notes

These are the key protocol facts that must be validated against DNP3 / IEEE
1815 documentation during F2 spec work. Items marked [CONFIRM] require
external research-agent verification before F2 BCs are written.

### 4.1 Data-Link Layer Header (10 bytes)

```
Byte 0-1:   Start bytes 0x05 0x64 (constant; discriminates DNP3)
Byte 2:     Length (number of bytes from byte 2 to end of frame, excluding CRC bytes)
            Minimum valid length: 5 (for ACK/NACK frames). Maximum: 255.
            [CONFIRM: exact length semantics — whether length includes CRC fields]
Byte 3:     Control byte
            Bits 7-6: DIR (direction) and PRM (primary message) flags
            Bits 5-4: FCB/FCV (frame count bit / frame count valid)
            Bits 3-0: function code (data-link function code)
            [CONFIRM: exact bit layout, DIR flag direction semantics]
Byte 4-5:   Destination address (u16 little-endian)
            0xFFFF = broadcast (all stations)
            0xFFFC–0xFFFE = broadcast variants [CONFIRM: exact broadcast range]
Byte 6-7:   Source address (u16 little-endian)
Byte 8-9:   CRC (u16 CRC-16/DNP of bytes 0-7)
```

### 4.2 Transport Function (1 byte, follows data-link header)

```
Bit 7: FIN (final fragment)
Bit 6: FIR (first fragment)
Bits 5-0: Sequence number
```

For single-fragment application messages (common for small reads/writes), both
FIR and FIN are set. Multi-fragment: FIR=1/FIN=0, FIR=0/FIN=0, ..., FIR=0/FIN=1.

### 4.3 Application Layer (follows transport byte)

```
Byte 0: Application control byte
        Bit 7: FIR (first fragment — mirrors transport FIR)
        Bit 6: FIN (final fragment — mirrors transport FIN)
        Bit 5: CON (confirm requested)
        Bit 4: UNS (unsolicited response if set in response direction)
        Bits 3-0: Sequence number
Byte 1: Function code (0x00-0xFF)
```

### 4.4 Key Function Codes [CONFIRM against IEEE 1815 Table 3-14]

| FC (hex) | Name | MITRE mapping |
|----------|------|---------------|
| 0x01 | READ | No MITRE tag (benign read) |
| 0x02 | WRITE | T0855 (Unauthorized Command Message) |
| 0x03 | SELECT | T0855 (precursor to direct operate) |
| 0x04 | OPERATE | T0855 (first step of select-before-operate) |
| 0x05 | DIRECT_OPERATE | T0855 (bypasses SBO — HIGH risk) |
| 0x06 | DIRECT_OPERATE_NO_ACK | T0855 (no acknowledgment — highest risk) |
| 0x07 | IMMED_FREEZE | T0803 candidate [CONFIRM] |
| 0x09 | FREEZE_CLEAR | T0803 candidate [CONFIRM] |
| 0x0D | COLD_RESTART | T0814 (Denial of Service) |
| 0x0E | WARM_RESTART | T0814 (Denial of Service) |
| 0x13 | ENABLE_UNSOLICITED | anomaly — enables unsolicited responses |
| 0x14 | DISABLE_UNSOLICITED | anomaly |
| 0x81 | RESPONSE | normal (server response) |
| 0x82 | UNSOLICITED_RESPONSE | anomaly (spontaneous report without poll) |
| 0x83 | AUTHENTICATE_RESP | [CONFIRM: SA extensions] |

**[CONFIRM]:** The exact FC-to-MITRE mapping above is a first-pass derivation.
F2 requires research-agent confirmation against MITRE ATT&CK for ICS v19.1
(already pinned in mitre_attack_version per D-038) and IEEE 1815-2012/2024.

### 4.5 Broadcast Addresses

DNP3 destination 0xFFFF (and possibly 0xFFFC/0xFFFE/0xFFFD) are broadcast
addresses. A command to a broadcast address is a HUMAN SCOPE DECISION (#4)
on whether to map T0855 or a separate MITRE technique.

### 4.6 CRC-16/DNP — THE CRITICAL IMPLEMENTATION DECISION

DNP3 uses CRC-16/DNP (a specific polynomial: 0x3D65, reflected input, reflected
output, init 0x0000, final XOR 0xFFFF — distinct from standard CRC-16/ARC)
appended to every 16-byte data block (the first block is the 10-byte data-link
header + 6 bytes of transport/application data; subsequent blocks are up to
16 bytes of application data each, followed by a 2-byte CRC).

**This means a DNP3 application message is NOT a contiguous byte stream** —
it has CRC bytes interspersed every 16 bytes that must either be validated and
stripped or skipped during parsing. This is structurally unlike Modbus MBAP
(no CRC; length-delimited; simple offset advancing).

**[CONFIRM]: CRC polynomial, block size, and stripping algorithm.**

### HUMAN SCOPE DECISION #2: CRC validation strictness

**Options:**
1. **Skip CRCs entirely** (v1): Strip/skip the CRC bytes from each block
   during parsing without computing them. Fast, zero-false-negatives on
   valid traffic, but accepts corrupt/crafted frames. Appropriate for forensic
   replay analysis of real captures (corrupt packets are rare in PCAP files).
2. **Validate CRCs** (strict): Compute CRC-16/DNP on each block and discard
   frames with CRC failures. More correct but requires the CRC polynomial
   implementation; invalid frames are silently dropped (parse_errors).
3. **Validate and emit finding on CRC failure**: A CRC failure in a network
   capture may indicate replay attacks, crafted packets, or capture corruption.
   Emitting an anomaly finding on CRC mismatch provides forensic signal.

**Recommendation:** Option 1 (skip CRCs) for v1, same spirit as Modbus "skip
CRC" approach. Record this as a DEFERRED item. CRC validation can be added
in a later cycle as an enhancement. The CRC-stripping algorithm (block-skip
logic) is the minimum required for correct application-layer parsing.

### 4.7 Transport-Layer Fragment Reassembly

### HUMAN SCOPE DECISION #3: Application-layer fragment reassembly depth

**Context:** A single DNP3 "application message" may span multiple transport
frames (each frame: max 249 bytes application data after CRC stripping). Common
messages like READ responses with many data objects can span 3-5 frames.

**Options:**
1. **Data-link layer only**: Parse the data-link header and emit findings based
   on FC in the first transport frame. Do not reassemble multi-frame messages.
   Simple, no state. Misses function codes that appear only in continuation
   frames (rare in practice — FC is always in the first fragment).
2. **Single-frame application parse**: Parse FC and application control byte
   from the first fragment only (FIR=1 frames). No reassembly buffer needed
   since FC is always in the first fragment per IEEE 1815.
3. **Full fragment reassembly**: Buffer all fragments with the same sequence
   number, concatenate application data after CRC stripping, parse the complete
   application message. Necessary for parsing multi-object data responses (reads).

**Recommendation:** Option 2 for v1 (parse FC from first fragment, track FIR/FIN
flags, no reassembly buffer). FC is always in byte 1 of the application layer
of the first fragment. Detection goals (unauthorized commands, restarts) all
fire on FCs that appear in the first fragment. Full reassembly adds complexity
disproportionate to v1 detection goals.

---

## 5. Affected Artifacts Inventory

### 5.1 NEW Artifacts

| Artifact | Notes |
|----------|-------|
| `src/analyzer/dnp3.rs` | New `Dnp3Analyzer` struct implementing `StreamHandler` (TCP path). Pure-core parse fns + detection engine. |
| Subsystem SS-15 "DNP3/ICS Analysis" | New BC namespace BC-2.15.NNN. Modbus took SS-14; next is SS-15. |
| VP-023 (proposed) | DNP3 data-link parse safety + function-code classification. Kani candidate (pure-core parse fns). See §7. |
| ADR-007 (proposed) | DNP3 integration decisions: TCP-only vs dual-path, CRC handling, fragment reassembly depth. |
| `.factory/specs/behavioral-contracts/ss-15/` | New directory: BC-2.15.001..NNN files. |
| `vp-023-dnp3-parse-safety.md` | VP-023 file in verification-properties/. |

### 5.2 MODIFIED Source Files

| Component | Risk | Change Description |
|-----------|------|--------------------|
| `src/dispatcher.rs` | **HIGH** | Add `DispatchTarget::Dnp3` variant; extend `classify()` port-fallback to recognize port 20000 as Rule 6 (after port 502 Rule 5); add `dnp3: Option<Dnp3Analyzer>` field to `StreamDispatcher`; add `dnp3_analyzer()` and `take_dnp3_analyzer()` accessors; route `on_data` and `on_flow_close` to Dnp3 arm. The VP-004 Kani `classify_oracle` in `kani_proofs` must gain the port-20000 arm in lockstep — this is the exact same risk as Modbus (D-032: "CRITICAL — VP-004 oracle"). |
| `src/mitre.rs` | **CRITICAL** | T0803 ("Block Command Message") is NOT currently seeded. Add `"T0803" => ("Block Command Message", MitreTactic::IcsImpairProcessControl)` arm; bump `SEEDED_TECHNIQUE_ID_COUNT` from 21 to 22; add `"T0803"` to `SEEDED_TECHNIQUE_IDS`; add `"T0803"` to `EMITTED_IDS` in kani_proofs (T0855 is already in EMITTED_IDS from Modbus v0.4.0). The `vp007_catalog_drift_guard` test sweeps all T[0-9]{4} IDs and fails mechanically if the arm and SEEDED_TECHNIQUE_IDS are not updated atomically. |
| `src/analyzer/mod.rs` | LOW | Add `pub mod dnp3;`. Single-line, no logic. |
| `src/main.rs` | **MEDIUM** | Wire `--dnp3` / `--all` flags to `Dnp3Analyzer` construction; extend `needs_reassembly` to include `enable_dnp3`; collect DNP3 findings post-finalize; push DNP3 summary to `analyzer_summaries`. Mirror the 4-step Modbus pattern exactly (main.rs:167-174, 264-267). |
| `src/cli.rs` | **MEDIUM** | Add `#[arg(long)] dnp3: bool` flag to `Commands::Analyze`; add `*dnp3 \|\| *all` expansion. If CLI-configurable thresholds are approved (HUMAN SCOPE DECISION #4), add `--dnp3-direct-operate-threshold` or similar flags. |
| `src/dispatcher.rs` Kani proofs | **HIGH** | VP-004 `classify_oracle` must gain port-20000 → Dnp3 arm. `verify_content_first_precedence_exhaustive` proof asserts `got == want`; oracle must mirror production identically. See D-032 for the exact prior-cycle playbook. |

### 5.3 MODIFIED Factory Spec Artifacts

| Artifact | Change |
|----------|--------|
| `BC-INDEX.md` | Add SS-15 section (BC-2.15.001..NNN rows) |
| `ARCH-INDEX.md` | Add SS-15 to subsystem registry |
| `VP-INDEX.md` | Add VP-023; bump total 22→23; bump p1 8→9; bump kani 9→10 |
| `verification-architecture.md` | Add VP-023 row to Provable Properties Catalog; update P1 list; update summary counts |
| `verification-coverage-matrix.md` | Add VP-023 row; update per-module Kani count; update Totals row (Kani 9→10, total 22→23) |
| `module-criticality.md` | Add `Dnp3Analyzer` with criticality classification |

### 5.4 DEPENDENT (Regression Zone — must stay green)

| Component | Risk | Notes |
|-----------|------|-------|
| `tests/dispatcher_tests.rs` | HIGH | VP-004 tests (TLS-beats-port, HTTP content detection, port-502/Modbus Rule 5) must stay green after Rule 6 (port 20000) is added |
| `tests/mitre_tests.rs` | CRITICAL | `vp007_catalog_drift_guard` mechanically fails if T0803 is added to `technique_info` without updating `SEEDED_TECHNIQUE_IDS` + count |
| `src/reassembly/handler.rs` | NONE | `StreamHandler` trait unchanged |
| `src/reassembly/mod.rs` | NONE | `TcpReassembler::process_packet` routes transparently |
| `src/analyzer/modbus.rs` | NONE | Modbus unchanged; existing 1338 tests stay green |
| `tests/modbus_*` | NONE | No Modbus test changes required |
| All HTTP/TLS/DNS tests | NONE | Dispatcher additions are backward-compatible |

**Regression baseline:** 1338 tests (develop HEAD fb2c875, v0.4.0 state).

---

## 6. MITRE Delta

### Current State (post v0.4.0)

- Seeded IDs: 21 (11 Enterprise + 10 ICS)
- Emitted IDs: 13 (6 Enterprise + 7 ICS: T0855/T0836/T0835/T0831/T0806/T0814/T0888)
- `SEEDED_TECHNIQUE_ID_COUNT`: 21
- VP-007 status: verified/locked

### DNP3 MITRE Requirements

| Technique | Current status | Action required |
|-----------|---------------|-----------------|
| T0855 (Unauthorized Command Message) | SEEDED + EMITTED (Modbus) | No catalog change; DNP3 will co-emit. VP-007 EMITTED_IDS already includes T0855. |
| T0803 (Block Command Message) | NOT SEEDED, NOT EMITTED | **NEW**: add `technique_info` arm, seed in SEEDED_TECHNIQUE_IDS, add to EMITTED_IDS. Bump SEEDED_TECHNIQUE_ID_COUNT 21→22. |
| T0814 (Denial of Service) | SEEDED + EMITTED (Modbus) | No catalog change; DNP3 cold/warm restart detectors will co-emit. |
| T0888 (Remote System Information Discovery) | SEEDED + EMITTED (Modbus) | No catalog change; DNP3 DISABLE_UNSOLICITED or request-scanning may co-emit (confirm in F2). |

**T0803 tactic assignment:** MITRE ATT&CK for ICS v19.1 maps T0803 ("Block
Command Message") to the tactic "Inhibit Response Function" (TA0107).
This maps to the existing `MitreTactic::IcsInhibitResponseFunction` variant.
[CONFIRM: exact tactic assignment against ICS v19.1 catalog.]

**VP-007 atomic update obligation (verbatim from D-032/D-033 playbook):**
Adding T0803 requires updating FIVE things in the same commit:
1. `technique_info` match arm
2. `SEEDED_TECHNIQUE_IDS` array
3. `SEEDED_TECHNIQUE_ID_COUNT` constant (21→22)
4. `EMITTED_IDS` in `kani_proofs` module
5. Run `cargo test mitre` before the PR merges to confirm `vp007_catalog_drift_guard` passes

### HUMAN SCOPE DECISION #4: MITRE ICS technique breadth for DNP3

**Issue specification says:** T0855 (Unauthorized Command Message) and T0803
(Block Command Message). The Modbus precedent (D-032) added 7 ICS techniques
by human decision. The question is whether to limit DNP3 to the issue-specified
2 techniques or expand.

**Candidate expansion set (for human consideration):**
- T0803 Block Command Message (ICS Inhibit Response Function) — REQUIRED per issue
- T0855 Unauthorized Command Message (ICS Impair Process Control) — REQUIRED per issue
- T0814 Denial of Service — cold/warm restart (0x0D/0x0E) naturally maps here
- T0828 Loss of Control — possible for DIRECT_OPERATE_NO_ACK pattern [CONFIRM]
- T0836 Modify Parameter — WRITE FC (0x02) maps here (consistent with Modbus)

**Recommendation:** Minimal set for v1: T0803 + T0855 + T0814 (3 techniques).
T0836 for WRITE is consistent with Modbus precedent and costs only one
additional SEEDED entry (already seeded from Modbus) with no catalog change.
Expanding to T0828 requires new catalog entry and [CONFIRM] per ICS v19.1.
Defer T0828 and any beyond T0836 to later cycles.

---

## 7. Verification Property: VP-023 (Proposed)

**Title:** "DNP3 Data-Link Frame Parse Safety and Function-Code Classification"

**Module:** `src/analyzer/dnp3.rs`

**Phase:** P1 (consistent with VP-022 Modbus assignment; new code, no legacy debt)

**Tool:** Kani

**Sub-properties (draft):**
- Sub-A: DNP3 data-link header extraction never panics on any input shorter
  than 10 bytes (minimum DNP3 frame). Kani harness: symbolic `&[u8]` of
  length 0..10, assert no panic on `parse_dnp3_dl_header`.
- Sub-B: Start-byte validation is a necessary precondition — `parse_dnp3_dl_header`
  returns `None` iff `data[0] != 0x05 || data[1] != 0x64`. Kani harness:
  symbolic 2-byte prefix, assert return is None when prefix does not match
  0x05 0x64.
- Sub-C: Application function code classification is total over all 256 FC
  values with no gaps (analogous to VP-022 sub-B for Modbus). Kani harness:
  symbolic `u8` FC, assert `classify_dnp3_fc(fc)` returns a valid variant
  for every value.
- Sub-D (if CRC validation is approved): CRC-16/DNP computation is deterministic
  and matches a reference vector. Kani + unit test sufficient.

**Feasibility:** HIGH. All sub-properties operate on small bounded inputs
(byte slices ≤10 bytes, single u8). Kani handles these with no bound explosion.
Analogous to VP-022 which ran successfully against Kani (D-044: "VP-022
LOCKED @68a3306; cargo kani 0.67.0, CBMC 140+ SAT checks").

**Verified BCs (draft):** BC-2.15.001 (start-byte detection), BC-2.15.002
(length field semantics), BC-2.15.003 (address field decoding), BC-2.15.004
(validity gate), BC-2.15.005 (FC classification totality), BC-2.15.006
(FC classification correctness). Exact BC numbers assigned in F2.

---

## 8. DTU Assessment

**Verdict: DTU_REQUIRED = false**

DNP3 is a self-contained binary protocol analysis module. Like Modbus (D-032:
"DTU_REQUIRED: false"), the DNP3 analyzer:
- Has no external service dependencies.
- Processes bytes from PCAPs (local file I/O only, already in the pipeline).
- Does not call any external API, webhook, or third-party service.
- CRC-16/DNP implementation is pure local computation.
- MITRE technique lookup is the existing local `technique_info` function.

No DTU clones, no API research agents required for the analyzer itself.
External research agents are needed only for:
- Fetching DNP3 / IEEE 1815 specification details (non-service dependency).
- Confirming MITRE ATT&CK ICS v19.1 technique-to-tactic mappings.

These are research inputs for F2 spec authoring, not runtime service
dependencies of the deployed binary.

---

## 9. Regression Risk and Test Safety Net

### Safety Net

The full existing test suite (1338 tests, develop HEAD fb2c875) is the
regression baseline. No existing tests are expected to fail from this feature
(all changes are additive or extend existing patterns).

### Highest-Risk Shared Touchpoints

1. **`src/dispatcher.rs` / VP-004 Kani oracle (HIGH):** Adding Rule 6
   (port 20000 → Dnp3) to `classify()` requires updating `classify_oracle` in
   the Kani proofs module identically. The `verify_content_first_precedence_exhaustive`
   proof asserts `got == want` across all 65536^2 port combinations. If oracle
   and production diverge, this Kani proof fails at F6. This is the same
   risk that required careful handling in the Modbus cycle (D-032: "Kani
   oracle must mirror production exactly").
   
   **New rule-order invariant to verify:** DNP3 (port 20000, Rule 6) must
   come AFTER all content rules (1-2) and port-fallback rules (3-5). A
   TLS/HTTP flow on port 20000 whose first chunk has a recognizable signature
   must match Rules 1-2 first. The `verify_tls_signature_beats_port` proof
   (now uses HTTP fallback port as adversarial test) should be extended or
   supplemented with a port-20000 variant to confirm TLS beats Rule 6.

2. **`src/mitre.rs` / VP-007 drift guard (CRITICAL):** The
   `vp007_catalog_drift_guard` test sweeps 10 million ID permutations. Adding
   T0803 to `technique_info` without updating `SEEDED_TECHNIQUE_IDS` causes
   an immediate test failure with an explicit error message. The atomic 5-part
   update obligation (see §6) from the Modbus cycle (D-033) applies identically.

3. **`src/dispatcher.rs` early-exit guard:** The current guard at line 217
   checks `self.http.is_none() && self.tls.is_none() && self.modbus.is_none()`.
   This must be extended to `&& self.dnp3.is_none()` or the guard pattern
   maintained consistently.

4. **`src/main.rs` 4-step wiring:** The Modbus cycle identified that omitting
   step 2 (`needs_reassembly` extension) silently disables the analyzer even
   when `--dnp3` is passed. The exact same risk applies here. The
   `needs_reassembly` predicate at main.rs:109 must include `enable_dnp3`.

---

## 10. Recommended F1-F7 Path

| Phase | Scope |
|-------|-------|
| F1 | Delta analysis (this document) + human gate |
| F2 | Spec evolution: BC-2.15.NNN (estimated 18-25 BCs), VP-023 spec, ADR-007, T0803 MITRE seed. Research: IEEE 1815 data-link header, transport function, application FCs, CRC-16/DNP polynomial. Adversarial spec review (Claude + Gemini cross-model per D-034 playbook). |
| F3 | Story decomposition: estimated 4-5 stories (parse core, detection engine, flow state + carry buffer, dispatcher/CLI wiring, VP-023 Kani harnesses). Wave schedule. |
| F4 | TDD implementation per story. Per-story Claude + Gemini adversarial convergence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001). |
| F5 | Combined-delta adversarial review of full analyzer (same pattern as D-043 caught CRITICAL timestamp-units bug). |
| F6 | Kani VP-023 (sub-A/B/C), fuzz DNP3 parse, mutation testing on classifier. cargo audit/deny clean. |
| F7 | Convergence: holdout, e2e pcap acceptance test (crafted DNP3 PCAP fixture), consistency audit. |

**Story decomposition estimate (F3):**
- STORY-NNN: DNP3 data-link header parser + CRC-stripping + start-byte validation (pure core, VP-023 Kani)
- STORY-NNN+1: DNP3 transport + application layer FC parse + flow state
- STORY-NNN+2: Detection engine (FC-level findings: unauthorized commands, restarts, unsolicited responses, broadcast)
- STORY-NNN+3: Dispatcher wiring (Rule 6, DispatchTarget::Dnp3) + CLI (--dnp3) + VP-004 oracle update + VP-007 T0803 seed
- STORY-NNN+4 (optional): broadcast anomaly detector + DIRECT_OPERATE_NO_ACK escalation

**BC estimate:** 18-25 BCs for SS-15 (comparable to Modbus SS-14 at 25 BCs per D-033).

---

## 11. Human Scope Decisions for F1 Gate

The following decisions must be made by the human before F2 spec work begins.
These mirror the D-032 pattern where human decisions at F1 gate defined the
scope for F2.

### DECISION 1 — Integration path: TCP-only or TCP+UDP dual-path

**Recommendation:** TCP-only for v1. StreamHandler + StreamDispatcher pattern,
mirror Modbus. UDP deferred to v2.

**If TCP+UDP:** Add a `ProtocolAnalyzer` path for UDP-only DNP3 in the same
cycle (DNS-style, per-packet, no reassembly). This requires two integration
paths for the same protocol, adding F2/F3 complexity.

### DECISION 2 — CRC validation strictness

**Recommendation:** Skip CRCs (strip by block structure, do not compute).
Defer CRC validation and CRC-failure findings to a later cycle. The CRC
stripping algorithm (skip 2 bytes every 16 data bytes) is required regardless
for correct application-layer parsing.

**If validate:** Add CRC-16/DNP computation (polynomial 0x3D65; confirm in
F2). Gate parse on CRC pass; increment parse_errors on fail. Optionally emit
anomaly finding on CRC mismatch. Adds ~20 lines of pure-core code and 2-3
BCs; the CRC function itself is a Kani candidate.

### DECISION 3 — Application-layer fragment reassembly depth

**Recommendation:** Parse FC from first fragment only (FIR=1 frames). No
multi-frame reassembly buffer for v1. Detection goals are all served by
first-fragment FCs.

**If full reassembly:** Buffer fragments by (source_addr, dest_addr, seq_num),
concatenate after CRC stripping, parse complete application message. Adds
significant per-flow state and complexity. Necessary for parsing data object
responses but not for FC-based detection.

### DECISION 4 — MITRE ICS technique breadth

**Recommendation:** Minimal set: T0803 (new, catalog addition required) +
T0855 (already emitted) + T0814 (already emitted; cold/warm restart) + T0836
(already seeded; DNP3 WRITE 0x02 maps here consistently with Modbus). This
adds exactly one new catalog entry (T0803).

**If expand to T0828 (Loss of Control):** Requires additional catalog entry,
tactic assignment confirmation against ICS v19.1, and [CONFIRM] research.

### DECISION 5 — CLI-configurable thresholds

**Context:** Modbus added `--modbus-write-burst-threshold` and
`--modbus-write-sustained-threshold`. DNP3 has analogous concepts:
DIRECT_OPERATE rate (how many in a time window before firing), restart
detection threshold.

**Recommendation:** Add `--dnp3-direct-operate-threshold` (default: N to be
decided in F2 based on protocol norms) as a minimum. Mirror the Modbus CLI
pattern. The exact default and threshold semantics are F2 decisions.

---

## 12. Consistency with Modbus Precedent Decisions

This analysis was produced with explicit reference to D-032 through D-046
(Modbus F1-F7 cycle). Key alignments:

| Precedent | DNP3 Alignment |
|-----------|---------------|
| D-032: StreamHandler integration (not ProtocolAnalyzer) | Confirmed: TCP-only StreamHandler recommended (DECISION 1) |
| D-032: Subsystem SS-14 new | SS-15 new for DNP3 (next sequential) |
| D-032: VP-022 Kani P1 | VP-023 Kani P1 recommended (same rationale: new code, no legacy debt) |
| D-032: VP-007 atomic update obligation | Same 5-part atomic update for T0803 (§6) |
| D-033: MITRE T0836 absent, must seed | T0803 absent, must seed — same pattern |
| D-034: dual-window rate detection | DNP3 DIRECT_OPERATE rate detection analogous; CLI thresholds recommended (DECISION 5) |
| D-034: Research-agent validation before F2 BCs | IEEE 1815 FC table, CRC polynomial, broadcast range all need [CONFIRM] research |
| D-043: F5 combined-delta critical timestamp-units bug | CC-004 policy: DNP3 must have at least 1 test through dispatcher boundary with timestamp_secs-shaped values |
| D-044: Kani VP-004 oracle update required | `classify_oracle` port-20000 arm required; highest-risk file in feature (§9) |
| D-046: DTU_REQUIRED = false | Confirmed: no external service dependencies (§8) |

---

## Appendix A: Key Source Line References

| Location | Current Lines | Relevance for DNP3 |
|----------|-------------|-------------------|
| `dispatcher.rs` classify() | 155-204 | Port-fallback section; Rule 6 (port 20000) goes after Rule 5 (port 502) at ~line 200 |
| `dispatcher.rs` on_data | 206-269 | Add `DispatchTarget::Dnp3` match arm |
| `dispatcher.rs` on_flow_close | 272-303 | Add `Some(DispatchTarget::Dnp3)` close arm |
| `dispatcher.rs` StreamDispatcher struct | 55-70 | Add `dnp3: Option<Dnp3Analyzer>` field |
| `dispatcher.rs` new() | 77-92 | Update constructor signature |
| `dispatcher.rs` early-exit guard | 217 | Extend to `&& self.dnp3.is_none()` |
| `dispatcher.rs` Kani classify_oracle | 363-398 | Must add port-20000 → Dnp3 arm after port-502 arm |
| `analyzer/mod.rs` | 14-17 | Add `pub mod dnp3;` |
| `main.rs` run_analyze needs_reassembly | 109 | Extend to include `enable_dnp3` |
| `main.rs` analyzer construction | 167-174 | Add Dnp3Analyzer construction pattern |
| `main.rs` post-finalize findings | 264-267 | Add `take_dnp3_analyzer()` / findings collection |
| `cli.rs` Commands::Analyze | 131-172 | Add `dnp3: bool` and threshold flags |
| `mitre.rs` technique_info | 122-168 | Add T0803 arm (suggested: after T0803/before T0855) |
| `mitre.rs` SEEDED_TECHNIQUE_IDS | 287-312 | Add "T0803" |
| `mitre.rs` SEEDED_TECHNIQUE_ID_COUNT | 320 | Bump 21 → 22 |
| `mitre.rs` EMITTED_IDS (kani_proofs) | 208-224 | Add "T0803" |
| `decoder.rs` app_protocol_hint | 112 | Port 20000 hint — check if already present (if not, add "DNP3") |

---

## Appendix B: Items Requiring External Research (F2 Prerequisite)

Before F2 BCs can be written, the following must be confirmed via research-agent
(Perplexity / IEEE 1815 documentation):

1. **[CONFIRM-1]** DNP3 data-link length byte exact semantics: does `length`
   count from byte 2 (inclusive) to end of frame excluding CRC bytes, or
   including CRC bytes? This affects the ADU-size calculation in the carry-
   buffer and parse loop, analogous to the Modbus MBAP length off-by-one
   that was corrected in D-039.

2. **[CONFIRM-2]** Exact broadcast address range: is it 0xFFFF only, or does
   0xFFFC/0xFFFD/0xFFFE also qualify? Affects BC for broadcast anomaly detection.

3. **[CONFIRM-3]** CRC-16/DNP polynomial: 0x3D65 reflected? Init 0x0000, final
   XOR 0xFFFF? Confirm against IEEE 1815-2012 Annex B or DNP Users Group
   documentation.

4. **[CONFIRM-4]** Function code complete table from IEEE 1815 Table 3-14:
   confirm 0x03=SELECT, 0x04=OPERATE, 0x05=DIRECT_OPERATE, 0x06=DIRECT_OPERATE_NO_ACK,
   0x07=IMMED_FREEZE, 0x09=FREEZE_CLEAR, 0x0D=COLD_RESTART, 0x0E=WARM_RESTART,
   0x13=ENABLE_UNSOLICITED, 0x14=DISABLE_UNSOLICITED, 0x82=UNSOLICITED_RESPONSE.

5. **[CONFIRM-5]** T0803 (Block Command Message) tactic in ICS ATT&CK v19.1:
   confirm it is "Inhibit Response Function" (TA0107) mapping to
   `MitreTactic::IcsInhibitResponseFunction`. Confirm T0828 (Loss of Control)
   tactic assignment if DECISION 4 expands to include it.

6. **[CONFIRM-6]** Whether T0814 (DoS, ICS Inhibit Response Function) is the
   correct MITRE mapping for DNP3 COLD_RESTART (0x0D) and WARM_RESTART (0x0E),
   consistent with the Modbus FC=0x08 (force-listen-only/restart) mapping.
