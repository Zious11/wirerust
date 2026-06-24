---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.019: StreamDispatcher Rule 7 — Port 44818 TCP Classified as DispatchTarget::Enip

## Description

`StreamDispatcher::classify()` gains Rule 7: when a flow's port set contains port 44818 (the
IANA-registered EtherNet/IP explicit messaging TCP port), and Rules 1–6 have not matched,
the flow is classified as `DispatchTarget::Enip`. Rule 7 is inserted after the existing
Rule 6 (port 20000 → DNP3). The previous implicit "no match" tail becomes Rule 8. Content
rules 1–2 (TLS signature, HTTP method prefix) take absolute priority — a TLS ClientHello
on port 44818 correctly routes to `DispatchTarget::Tls`. This is a documented exception to
ADR-0001 (content-first principle), following the same pattern as Modbus (Rule 5) and DNP3
(Rule 6).

## Preconditions

1. `classify(ports, data)` is called by `StreamDispatcher` for a new flow.
2. Rules 1–6 have not matched (TLS, HTTP, 443/8443, 80/8080, 502, 20000 all absent).
3. The flow's port set contains 44818.

## Postconditions

1. `classify()` returns `DispatchTarget::Enip`.
2. `EnipAnalyzer::on_data()` receives all subsequent TCP bytes for this flow.
3. Rules 1–6 are unchanged — no existing flow classification is affected.
4. Rule 8 (no match → `DispatchTarget::None`) is unaffected and remains the fallback.

## Invariants

1. **Content-first precedence preserved**: TLS signature (0x16, 0x03) and HTTP method prefix
   take priority. A flow on port 44818 carrying TLS bytes routes to `DispatchTarget::Tls`
   (Rule 1), never to `DispatchTarget::Enip`. The post-classification validity gate
   (`is_valid_enip_frame`) is the compensating control for non-ENIP traffic on 44818.
2. **Rule ordering**: after this change, Rule 7 (port 44818 → Enip) is immediately followed
   by Rule 8 (no match → None). No rule between 7 and 8 exists.
3. **VP-004 oracle obligation**: the `classify_oracle` function in
   `dispatcher.rs #[cfg(kani)] mod kani_proofs` must gain the port-44818 → Enip arm
   immediately after the port-20000 → Dnp3 arm. Divergence between oracle and production
   causes VP-004 to fail at F6.
4. **Early-exit guard extension**: the `is_none()` check on `self.enip` must be added to the
   existing early-exit guard (mirrors ADR-007 DNP3 pattern for `self.dnp3.is_none()`).
5. **DispatchTarget::Enip variant**: added to the enum after `DispatchTarget::Dnp3`. All
   exhaustive matches on `DispatchTarget` must be updated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow on port 44818, no TLS/HTTP bytes | `DispatchTarget::Enip` |
| EC-002 | Flow on port 44818 with TLS ClientHello (0x16, 0x03 prefix) | `DispatchTarget::Tls` (Rule 1 fires first) |
| EC-003 | Flow on port 44818 with HTTP GET prefix | `DispatchTarget::Http` (Rule 2 fires first) |
| EC-004 | Flow on port 80 (HTTP, not 44818) | `DispatchTarget::Http` (Rule 4) — Rule 7 not reached |
| EC-005 | Flow on port 502 (Modbus) | `DispatchTarget::Modbus` (Rule 5) — Rule 7 not reached |
| EC-006 | Flow on port 99 (unknown) | `DispatchTarget::None` (Rule 8 fallback) |
| EC-007 | Flow on port 44818 with ENIP analyzer not configured (enip is None) | Early-exit guard fires; `DispatchTarget::None` (no ENIP analyzer to route to) |

## Canonical Test Vectors

| ports | data prefix | Expected DispatchTarget | Rule |
|-------|------------|------------------------|------|
| `{44818, 12345}` | `[0x65, 0x00, ...]` (ENIP) | `Enip` | 7 |
| `{44818, 12345}` | `[0x16, 0x03, ...]` (TLS) | `Tls` | 1 |
| `{44818, 12345}` | `[0x47, 0x45, 0x54, ...]` (HTTP GET) | `Http` | 2 |
| `{80, 12345}` | `[0x65, 0x00, ...]` | `Http` | 4 |
| `{99, 12345}` | any | `None` | 8 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | Content-first precedence exhaustive (classify_oracle and classify must agree for all port combinations including 44818); oracle must include port-44818 → Enip arm | Kani: `verify_content_first_precedence_exhaustive` — fails if oracle and production diverge (VP-007 obligation STORY-EIP-09 includes oracle update) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — port-44818 TCP classification is the entry point for all EtherNet/IP analysis; without Rule 7, no ENIP traffic is routed to the EnipAnalyzer and none of the MITRE detections (T0858, T0816, T0836, T0846, T0888) are reachable |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — content rules 1–2 have absolute priority over port fallbacks; this invariant is proven by VP-004) |
| Architecture Module | SS-05 (dispatcher.rs), SS-17 (analyzer/enip.rs); ADR-010 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — dispatcher rule; no finding emission) |

## Related BCs

- BC-2.17.003 — composes with (validity gate is the compensating control for port-only classification)
- BC-2.17.016 — depends on (on_data is called only for flows classified as Enip)

## Architecture Anchors

- `src/dispatcher.rs` — `classify()` — Rule 7 arm: `if ports.contains(&44818) { return DispatchTarget::Enip; }`
- `src/dispatcher.rs` — `DispatchTarget::Enip` variant added after `Dnp3`
- `src/dispatcher.rs` — `StreamDispatcher.enip: Option<EnipAnalyzer>` field
- `src/dispatcher.rs` — early-exit guard: `&& self.enip.is_none()` added
- `src/dispatcher.rs` — `#[cfg(kani)] mod kani_proofs` → `classify_oracle`: port-44818 → Enip arm
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 1` (Rule 7)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-004 — content-first precedence exhaustive (oracle must include port-44818 → Enip)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 1 (Rule 7 specification and rule table); IANA port 44818 EtherNet/IP assignment (IETF RFC 4897) |
| **Confidence** | high — port 44818 is IANA-registered for EtherNet/IP; Rule 7 pattern is identical to Rules 5 and 6 |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads ports set; no mutation |
| **Deterministic** | yes — same port set always produces same DispatchTarget |
| **Thread safety** | Send + Sync (read-only classification function) |
| **Overall classification** | pure-core-adjacent (dispatcher classify is stateless; effectful in terms of routing decision) |
