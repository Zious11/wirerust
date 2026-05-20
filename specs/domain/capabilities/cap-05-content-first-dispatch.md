---
artifact: L2-cap-05
traces_to: ../domain-spec.md
cap_id: CAP-05
title: Content-First Protocol Dispatch
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-05: Content-First Protocol Dispatch

## What the system does today

`StreamDispatcher` (E-21, C-15) implements `StreamHandler` and sits between the TCP
reassembly engine and the protocol analyzers. For each reassembled data chunk, it classifies
the flow's protocol by inspecting the first bytes of content before consulting port numbers
(ADR 0001).

**Sources:** C-15 dispatcher.rs. BC-DSP-001..009.

## Classification algorithm (classify function)

```
1. If data.len() >= 5 AND data[0] == 0x16 AND data[1] == 0x03:
       -> DispatchTarget::Tls   (TLS record type + major version)
2. Else if data starts with b"GET " | b"POST " | b"PUT " | b"DELETE " |
          b"HEAD " | b"OPTIONS " | b"CONNECT " | b"PATCH " | b"TRACE ":
       -> DispatchTarget::Http
3. Else if flow.lower_port or flow.upper_port in {80, 443, 8080, 8443}:
       -> DispatchTarget::Http or Tls (port-based fallback)
4. Else -> DispatchTarget::None
```

TLS strictly wins over HTTP for any data starting with `0x16 0x03` regardless of port. The
HTTP method-prefix check is unreachable for any data that begins with these two bytes
(pass-2 R3 Target 4 confirmed mechanically).

## Loose TLS gate (Smell #10)

The TLS check validates only byte 0 (`0x16`) and byte 1 (`0x03`). It does NOT check byte 2
(TLS minor version: 0x00..0x04) or bytes 3-4 (record length). Any TCP data beginning with
`0x16 0x03 <any> <any> <any>` is dispatched to TLS. Low theoretical risk (Smell #10,
advisory). Zero tests exercise the misroute path.

## Caching behavior (INV-2)

- `DispatchTarget::Http` and `DispatchTarget::Tls` are cached in `routes: HashMap<FlowKey,
  DispatchTarget>` on first classification.
- `DispatchTarget::None` operates in two phases:
  Phase 1 (attempt count < `max_classification_attempts`): `None` is NOT cached in `routes`;
  `classify()` re-runs on each subsequent `on_data` chunk and increments the per-flow
  `classification_attempts` counter.
  Phase 2 (attempt count reaches `max_classification_attempts`): `DispatchTarget::None` IS
  inserted permanently into `routes` and `classify()` is no longer called for that flow.
  The `classification_attempts` entry is removed at that point.
  (dispatcher.rs:137-148; LESSON-P2.11)

## max_classification_attempts knob (P2.11 / #80)

`StreamDispatcher` now carries `max_classification_attempts: u32` (default
`DEFAULT_MAX_CLASSIFICATION_ATTEMPTS`) and a `classification_attempts: HashMap<FlowKey, u32>`
counter. When a flow's attempt count reaches the cap, subsequent `on_data` calls for that
flow are forwarded to no analyzer (the flow stays effectively unclassified until close). This
closes the unbounded-reclassification O(packets_in_flow) cost for flows that never produce
enough data to classify.

The knob is configurable via `StreamDispatcher::with_max_classification_attempts()`.

## Unclassified flows

When `on_flow_close` fires for a flow with no cached route, the `unclassified_flows` counter
increments (accessible via `dispatcher.unclassified_flows()`). Handshake-only flows (SYN
without data) are counted here; this metric can be misleading since these are legitimate.

## Analyzer wiring (public fields -- Smell #6)

`StreamDispatcher.http: Option<HttpAnalyzer>` and `.tls: Option<TlsAnalyzer>` are public
fields. Main.rs drains findings from them directly. This pub-field exposure is a low-severity
smell (unchanged).

## BC references

BC-DSP-001..009: content-first precedence (001-004), None-not-cached (005/006), unclassified
counter (007/008), route-remove on close (009).
