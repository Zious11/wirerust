---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.021: Port-20000 Flow Dispatched to Dnp3Analyzer (DispatchTarget::Dnp3, Rule 6)

## Description

The `StreamDispatcher.classify()` function routes TCP flows to `DispatchTarget::Dnp3` when the
flow's port set contains 20000, AFTER all content-based rules (TLS, HTTP) and earlier port
fallback rules (Modbus/502) have failed to match. This is ADR-007 Rule 6. A TLS ClientHello
or HTTP method prefix on port 20000 matches Rule 1 or Rule 2 (content rules) before reaching
Rule 6, preserving INV-2 / VP-004 precedence. `Dnp3Analyzer` is only instantiated when the
dispatcher routes a flow to `DispatchTarget::Dnp3`.

## Preconditions

1. `StreamDispatcher.classify(ports, data)` is called for a new TCP flow.
2. Rules 1–5 (TLS content, HTTP content, TLS port 443/8443, HTTP port 80/8080, Modbus port 502)
   have all returned `false` (no match).
3. `ports` contains 20000 (IANA-registered DNP3 TCP port). [SPEC: IANA port assignment; dnp3-research.md §1]

## Postconditions

1. `classify()` returns `DispatchTarget::Dnp3`.
2. The `StreamDispatcher` instantiates a `Dnp3Analyzer` for this flow (if not already present).
3. Subsequent `on_data` calls for this flow are delivered to `Dnp3Analyzer::on_data`.
4. `StreamDispatcher.dnp3: Option<Dnp3Analyzer>` is `Some(...)` for flows classified as DNP3.

**Content-first precedence (INV-2 preserved):**
5. A TLS ClientHello (`data[0]==0x16 && data[1]==0x03`) on port 20000 returns `DispatchTarget::Tls`
   (Rule 1), NOT `DispatchTarget::Dnp3`. Rule 6 is never reached.
6. An HTTP method prefix on port 20000 returns `DispatchTarget::Http` (Rule 2), NOT `DispatchTarget::Dnp3`.

## Invariants

1. **Rule 6 position**: port-20000 is Rule 6 — after all content rules and after port-502 Modbus
   (Rule 5). This ensures no existing flow classification is stolen. [ADR-007 Decision 1]
2. **Fallback becomes Rule 7**: `DispatchTarget::None` (previously "Rule 6") becomes Rule 7 after
   this change. [ADR-007 Decision 1 Rule Table]
3. **VP-004 oracle obligation**: the `classify_oracle` in `dispatcher.rs`'s `#[cfg(kani)] mod`
   MUST gain the port-20000 → Dnp3 arm immediately after port-502 → Modbus. VP-004 proves
   oracle ≡ production for all 65536² port combinations. [ADR-007 Decision 1; VP-023-verification-delta §2.1]
4. **Early-exit guard**: `if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() && self.dnp3.is_none()` — the guard MUST include `self.dnp3.is_none()` to prevent silent data-drop when only a DNP3 analyzer is active. [ADR-007 §StreamDispatcher struct delta]
5. **take_dnp3_analyzer()**: post-finalization, `dispatcher.take_dnp3_analyzer()` moves the `Dnp3Analyzer` out for result collection, mirroring `take_modbus_analyzer()`. [dnp3-architecture-delta.md §4.5]

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Port 20000, no content data | Rule 6 fires: `DispatchTarget::Dnp3` |
| EC-002 | Port 20000, TLS ClientHello content | Rule 1 fires: `DispatchTarget::Tls` (Rule 6 never reached) |
| EC-003 | Port 20000, HTTP GET | Rule 2 fires: `DispatchTarget::Http` (Rule 6 never reached) |
| EC-004 | Port 502 (Modbus) | Rule 5 fires: `DispatchTarget::Modbus` |
| EC-005 | Port 12345 (unknown) | Rule 7 fires: `DispatchTarget::None` |
| EC-006 | Port 20000 AND port 502 in same flow | Rule 5 fires: `DispatchTarget::Modbus` (Rule 5 precedes Rule 6) |
| EC-007 | DNP3 analyzer present, HTTP/TLS analyzers absent | Early-exit guard passes (dnp3 is not None); `on_data` proceeds |

## Canonical Test Vectors

| ports | data[0..2] | Expected `DispatchTarget` |
|-------|-----------|--------------------------|
| {20000} | `[0x05, 0x64, ...]` (DNP3) | `Dnp3` (Rule 6) |
| {20000} | `[0x16, 0x03, ...]` (TLS) | `Tls` (Rule 1) |
| {20000} | `[0x47, 0x45, ...]` ("GE" = HTTP GET) | `Http` (Rule 2) |
| {502} | any | `Modbus` (Rule 5) |
| {443} | any | `Tls` (Rule 3) |
| {12345} | any | `None` (Rule 7) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | `classify_oracle` must include port-20000 → Dnp3 arm; `verify_content_first_precedence_exhaustive` passes for all 65536² port combinations | Kani (obligation in F4: oracle update per ADR-007 Decision 1 + VP-023-verification-delta §2.1) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — correct port-20000 dispatch is the entry gate for the DNP3/ICS analyzer capability; without Rule 6, no DNP3 flows reach the analyzer regardless of protocol content |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — Rule 6 is a port-fallback rule that fires AFTER all content rules, preserving INV-2) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24) + SS-05 (dispatcher.rs); ADR-007 Decision 1 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — dispatch routing BC; no finding emission) |

## Related BCs

- BC-2.15.001 through BC-2.15.022 — all depend on (this BC is a prerequisite; Dnp3Analyzer only receives data when Rule 6 fires)

## Architecture Anchors

- `src/dispatcher.rs` — `fn classify()` Rule 6: `if ports.contains(&20000) { return DispatchTarget::Dnp3; }`
- `src/dispatcher.rs` — `enum DispatchTarget { Http, Tls, Modbus, Dnp3, None }` (Dnp3 variant added)
- `src/dispatcher.rs` — `StreamDispatcher.dnp3: Option<Dnp3Analyzer>` field
- `src/dispatcher.rs` — `#[cfg(kani)] mod kani_proofs::classify_oracle` — must gain port-20000 arm
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §4` — complete dispatcher integration spec
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 1`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-004 — Content-First Dispatch Precedence (Kani; `classify_oracle` must gain port-20000 arm in F4 to keep proof green)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 1 (Rule 6; oracle obligation); dnp3-architecture-delta.md §4 (full dispatcher integration); ADR-007 §"Rule ordering after this change" |
| **Confidence** | high — architectural decision; mirrors ADR-005 Modbus precedent |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads ports (input); instantiates Dnp3Analyzer on match |
| **Deterministic** | yes — same ports/content always produce same DispatchTarget |
| **Thread safety** | single-threaded |
| **Overall classification** | dispatcher (SS-05 / SS-15 boundary) |
