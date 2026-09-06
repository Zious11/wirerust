---
document_type: domain-capability
capability_id: CAP-21
title: "S7comm Analysis"
subsystem: SS-21
feature: feature-s7comm
adr: ADR-014
introduced: v0.14.0
status: pending
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
---

# CAP-21: S7comm Analysis

## Description

wirerust gains passive analysis of Siemens S7comm, the proprietary PLC-programming and
HMI-communication protocol used overwhelmingly by S7-300/400 (classic S7comm,
COTP protocol-ID `0x32`) and S7-1200/1500 (S7comm-plus, protocol-ID `0x72`)
controllers, on TCP port 102. `S7commAnalyzer` (`src/analyzer/s7comm.rs`, SS-21) is the
**consumer** of CAP-20's ISO-on-TCP framing layer: on every TPKT frame extracted from a
port-102 flow, it calls `iso_on_tcp::parse_tpkt_header` then
`iso_on_tcp::parse_cotp_header`, then branches on the parsed `CotpHeader::protocol_id`
(ADR-014 Decision 2):

| `protocol_id` | Meaning | Analyzer behavior |
|---|---|---|
| `Some(0x32)` | Classic S7comm | Full S7comm PDU dissection (ROSCTR, PDU reference, parameter/data blocks, function codes) |
| `Some(0x72)` | S7comm-plus | Framing-level classification + unencrypted session-setup metadata only (ADR-014 Decision 6 — "observed, not dissected"); no function-code dissector |
| `None` (CR/CC TPDU) | Session establishment | Track connection state; defer classification until the first DT frame |
| `Some(other)` / unparseable | MMS, ICCP, or unrecognized ISO-on-TCP traffic | Left unclassified — never misattributed to S7comm |

Port TCP/102 dispatch is a single `DispatchTarget::S7comm` dispatcher rule (Rule 9);
there is no separate dispatcher rule per port-102 protocol identity — disambiguation is
entirely in-analyzer and, per ADR-014 Decision 2, is **load-bearing for correctness**,
not merely defense-in-depth (a first for wirerust's binary-ICS analyzers).

Classic S7comm PDU dissection drives new MITRE ATT&CK for ICS technique emissions —
T0843 (Program Download), T0889 (Modify Program), T0821 (Modify Controller Tasking), and
8 reused technique IDs (T0835, T0836, T0858, T0816, T0888, T0846, T0814, T1692.001) — per
ADR-014 Decision 5.

`S7commAnalyzer` owns `S7commFlowState`, including the directional TPKT/COTP carry
buffers (`carry_c2s` / `carry_s2c`, `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`) — carry
buffers are placed here, not in CAP-20, because CAP-20 is deliberately stateless
(ADR-014 Decision 1).

**Scope boundary for this capability entry:** BC-2.20.NNN (CAP-20, SS-20 framing) were
authored in feature-s7comm F2 part A. Part B is split in two: **part B1** (this burst)
authors the SS-21 **dissection** BCs — classic S7comm header/ROSCTR parsing, the
`S7commFlowState` lifecycle, the full function-code and Userdata-group classification
surface (`S7ClassicFunction`/`S7UserdataFunction`), and S7comm-plus framing-only
classification plus bounded session-setup metadata observation. Part B1 explicitly does
**not** author MITRE technique-emission BCs — it only names, per classification arm,
which of ADR-014 Decision 5's technique IDs the classification surface is built to
support. **Part B2** authors the MITRE technique-emission BCs (T0843/T0889/T0821 and the
8 reused IDs) on top of part B1's classification surface.

## Behavioral Contracts

BC-2.21.001 through BC-2.21.028 (28 BCs; see behavioral-contracts/ss-21/). Authored in
feature-s7comm F2 **part B1** (dissection layer): `S7commFlowState` and lifecycle
(001–003); classic S7comm header parse, `parse_s7comm_header` (004–009); Job/Ack_Data
function-code classification — Setup Communication, Read Var, Write Var + area codes,
Download/Upload triads, PLC Control PI-service decode, PLC Stop, totality (010–017);
Userdata function-group classification with the load-bearing group-0x03(Block)/
0x04(CPU/Read-SZL)/0x07(Time) correction, totality (018–023); S7comm-plus
framing-only classification, bounded session-setup metadata, TLS-deferral boundary
(024–026); unrecognized-`protocol_id`/unparseable-COTP-payload unclassified-gap paths
(027–028). MITRE technique-emission BCs (T0843/T0889/T0821 + 8 reused IDs) are
**pending, authored in feature-s7comm F2 part B2** — not part of this burst. VP
allocation (anticipated VP-048 range per ADR-014 Decision 9) is deferred to the F2
INTEGRATE sub-burst.

## Traceability

| Field | Value |
|-------|-------|
| ADR | ADR-014 (Classic S7comm over ISO-on-TCP — Stream Dispatch and Parser Design), Decisions 2, 3, 5, 6, 7, 8, 9 |
| Subsystem | SS-21 (ARCH-INDEX.md Subsystem Registry) |
| Consumes | CAP-20 (ISO-on-TCP Framing) — frozen `TpktHeader`/`CotpHeader`/`CotpTpduType` interface, `parse_tpkt_header`/`parse_cotp_header` free functions |
| Interacts with | CAP-18 (Protocol Coverage Catalog) — port-102 `Support` enum assignments (S7comm=Supported, S7comm-plus=DetectionOnly); see BC-2.18.005/006 |
| Non-goal (this cycle) | `main.rs::lookup_protocol_state` dynamic port-102 coverage-gap misclassification is NOT fixed by this capability (ADR-014 Decision 2/10 critical caveat) — deferred to F4 |
