# ADR 0005: Binary ICS Protocol Integration Strategy

**Status:** Accepted
**Date:** 2026-06-10
**Context:** Adding Modbus TCP (v0.4.0, issue #7) as the first binary ICS/OT protocol analyzer
required decisions about how binary protocols fit into the existing `StreamDispatcher` / `StreamAnalyzer`
architecture (ADR 0001 / ADR 0002), which was originally designed around text-based protocols (HTTP)
and TLS content-signature detection.

## Problem

Modbus TCP on port 502 has no distinctive content signature in the first bytes of the TCP stream —
the 7-byte MBAP header starts with a variable transaction ID and protocol ID. Content-based
classification (Rules 1–2 in the stream dispatcher) cannot identify it reliably. The existing
port-hint fallback rules (3–4) covered only 443/8443 (TLS) and 80/8080 (HTTP).

A subsequent ICS analyzer — DNP3 TCP on port 20000 (v0.6.0, issue #8) — faces the same problem:
its sync bytes (`0x05 0x64`) are valid but non-unique in isolation, and the DNP3 spec requires
inspecting both sync bytes together with the LENGTH field to validate a frame. Port-based routing
is the correct primary mechanism.

Key constraints that shaped the decision:

1. ICS protocols run on IANA-assigned ports (502 for Modbus, 20000 for DNP3) and almost never
   appear on other ports in real network captures.
2. Forensic captures may contain traffic from both IT and OT network segments; the dispatcher
   must not misclassify HTTP or TLS flows as Modbus or DNP3.
3. The VP-004 port-precedence invariant (content-signature rules fire before port rules) must be
   preserved so TLS-on-port-502 or HTTP-on-port-502 flows are not misrouted to the Modbus analyzer.
4. The `StreamAnalyzer` trait (ADR 0002) must not be changed for binary protocols; the same per-flow
   state pattern and `summarize()` / `findings()` interface is appropriate.

## Decision

**Binary ICS protocol analyzers are integrated using port-based classification rules added to the
stream dispatcher, appended after all existing content-signature and known-port rules.**

Specifically:

- **Rule 5** (added v0.4.0): Port 502 → `DispatchTarget::Modbus`.
  Fires after Rules 1–4 (TLS content, HTTP content, port-443/8443, port-80/8080), so TLS or
  HTTP running on port 502 is classified by its content signature first.

- **Rule 6** (added v0.6.0): Port 20000 → `DispatchTarget::Dnp3`.
  Fires after Rules 1–5, so TLS, HTTP, and Modbus content signatures and ports take precedence.

Both rules are port-only (no content sniffing). This is a deliberate exception to the
content-first principle of ADR 0001, justified by the fact that these protocols have no
distinctive first-byte signatures suitable for content detection.

The dispatcher struct gains a new `Option<FooAnalyzer>` field per ICS analyzer. Adding a future
ICS protocol requires: (1) a new `DispatchTarget` variant, (2) a new port rule appended after the
existing rules, (3) a new `Option<Analyzer>` field on `StreamDispatcher`, (4) a new rule arm in
`on_data` and `on_flow_close`. No changes to the reassembly engine are needed.

## Alternatives Considered

### Content-signature detection for Modbus / DNP3

Attempt to detect Modbus by checking `data[2..=3] == [0x00, 0x00]` (protocol ID = 0 invariant) or
DNP3 by checking `data[0..=1] == [0x05, 0x64]`.

- **Pro:** Consistent with the content-first principle.
- **Con:** Both signatures can appear in other protocols. Modbus's protocol-ID invariant
  (bytes 2–3 always 0x0000) is true but the surrounding bytes are variable; false positives
  are possible in binary streams. DNP3's sync pair `0x05 0x64` is sufficiently rare but
  requires >= 2 bytes and a 3-point validity gate (sync + LENGTH validity + CONTROL field
  plausibility) for any useful confidence. Adding all three checks as content-signature rules
  would produce a noisy classifier for a feature that port routing handles cleanly.
- **Rejected:** Port-based routing is accurate for IANA-assigned ports; the complexity of
  content-based ICS detection is not justified.

### A plugin/registry model for new protocol rules

A global registry where analyzers register their own port numbers and content signatures,
rather than explicit fields and match arms in `StreamDispatcher`.

- **Rejected:** Matches the "Analyzer Registry with Auto-Discovery" alternative rejected in
  ADR 0002. Magic registration obscures control flow. The number of analyzers is small;
  explicit wiring is clearer and easier to audit.

## Rationale

- **VP-004 invariant preserved.** Content-signature rules still fire before any port rule,
  so a TLS session on port 502 is still classified as TLS.
- **IANA ports are stable.** Port 502 (Modbus) and port 20000 (DNP3) are IANA-assigned and
  are almost never used for other protocols in real OT network captures.
- **Minimal change surface.** Each new ICS protocol adds ~10 lines to `dispatcher.rs` and one
  new `Option` field. No changes to the reassembly engine, trait definitions, or reporter.
- **Consistent with ADR 0002.** The `StreamAnalyzer` trait, per-flow state, bounded-resource
  constants, and `summarize()` / `findings()` interface are identical for Modbus and DNP3.

## Consequences

- `src/dispatcher.rs`: `DispatchTarget` enum grows one variant per ICS protocol. `StreamDispatcher`
  struct gains one `Option<Analyzer>` field per ICS protocol. Classification function appends one
  port-match branch per ICS protocol. `on_data` and `on_flow_close` delegate to the new field.
- `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`: implement `StreamAnalyzer`. Port-specific
  parsing (MBAP header for Modbus; data-link frame-walk with carry buffer for DNP3) is encapsulated
  within each analyzer module.
- The dispatcher comment block at the top of `src/dispatcher.rs` documents the rule order
  (Rules 1–7) and cross-references this ADR and ADR 0007 for traceability.
- Future binary protocols (e.g., DNP3 on a non-standard port, IEC 60870-5-104 on port 2404)
  follow the same pattern: append a port rule and add an `Option` field.
