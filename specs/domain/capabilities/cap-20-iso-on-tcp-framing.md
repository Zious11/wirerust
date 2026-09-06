---
document_type: domain-capability
capability_id: CAP-20
title: "ISO-on-TCP Framing (TPKT/COTP)"
subsystem: SS-20
feature: feature-s7comm
adr: ADR-014
introduced: v0.14.0
status: pending
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
---

# CAP-20: ISO-on-TCP Framing (TPKT/COTP)

## Description

wirerust gains a reusable, protocol-agnostic passive-parsing layer for **ISO-on-TCP**
transport — the TPKT (RFC 1006) + COTP (ISO 8073 / ITU-T X.224) substrate that carries
Siemens S7comm, and, in the future, IEC 61850 MMS and ICCP/TASE.2, all of which share
TCP port 102. This capability is deliberately factored out of the S7comm PDU dissector
(CAP-21) into its own module, `src/analyzer/iso_on_tcp.rs`, so that a future MMS or
ICCP/TASE.2 cycle can consume the same parsing functions unmodified — "build once,
benefit three times."

CAP-20 covers exactly two layers of framing, expressed as pure, stateless, free
functions with no `StreamAnalyzer` implementation and no per-flow state of its own:

1. **TPKT (RFC 1006)** — the 4-byte outer header present on every TCP segment carrying
   ISO-on-TCP traffic: version (1 byte, must equal `0x03`), reserved (1 byte, not
   validated), and length (2 bytes, big-endian, total packet length **including** the
   4-byte header itself, valid range `[4, 65535]`).
2. **COTP (ISO 8073 / ITU-T X.224)** — the Connection-Oriented Transport Protocol TPDU
   carried inside the TPKT payload. Three TPDU types are recognized: Connect Request
   (CR), Connect Confirm (CC) — both session-establishment TPDUs carrying no
   upper-layer payload — and Data Transfer (DT), which carries the steady-state
   upper-layer payload prefixed by a single protocol-ID byte (`0x32` classic S7comm,
   `0x72` S7comm-plus, or any other observed value). Any other TPDU-type code is left
   unparsed (returns `None`) rather than force-fit into one of the three recognized
   variants.

CAP-20 exports the **frozen** `TpktHeader` / `CotpTpduType` / `CotpHeader` types and the
`parse_tpkt_header` / `parse_cotp_header` free functions (ADR-014 Decision 1) as the
handoff interface to any consuming analyzer (CAP-21's `S7commAnalyzer` is the first
consumer). CAP-20 deliberately does **not** interpret the extracted `protocol_id`
byte — disambiguating `0x32` vs. `0x72` vs. other values is the consuming analyzer's
responsibility (ADR-014 Decision 2), keeping CAP-20 genuinely protocol-agnostic.

Directional carry buffers for TPKT frames spanning multiple TCP segments
(`MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`, walk-first residual-bound semantics) live on
the *consuming* analyzer's flow state (`S7commFlowState`, CAP-21), not inside CAP-20's
own module — CAP-20 is stateless by design (ADR-014 Decision 1).

**Non-goal (this cycle):** CAP-20 does not add a `DispatchTarget::IsoOnTcp` dispatcher
variant — it is a parsing library consumed by CAP-21, not an independent dispatch
target (ADR-014 Decision 1). CAP-20 also does not fix `main.rs::lookup_protocol_state`'s
dynamic port-102 coverage-gap misclassification (ADR-014 Decision 2/10's critical
caveat) — that requires the *analyzer's* parsed `protocol_id`, which is out of CAP-20's
stateless-library scope, and is deferred to a future F4 cycle.

## Behavioral Contracts

BC-2.20.001 through BC-2.20.016 (16 BCs; see behavioral-contracts/ss-20/). Authored in
feature-s7comm F2 part A (foundational framing layer). VP allocation (anticipated
VP-048 range per ADR-014 Decision 9) is deferred to the F2 INTEGRATE sub-burst.

## Traceability

| Field | Value |
|-------|-------|
| ADR | ADR-014 (Classic S7comm over ISO-on-TCP — Stream Dispatch and Parser Design), Decisions 1, 2, 4, 8, 9 |
| Subsystem | SS-20 (ARCH-INDEX.md Subsystem Registry) |
| Consuming capability | CAP-21 (S7comm Analysis) — `S7commAnalyzer` calls `parse_tpkt_header` then `parse_cotp_header` on every extracted TPKT frame |
| Future reuse | A future IEC 61850 MMS or ICCP/TASE.2 catalog-entry promotion (ADR-014 Decision 3 critical caveat) inherits `parse_tpkt_header`/`parse_cotp_header` with zero changes to `iso_on_tcp.rs` |
