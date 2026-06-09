---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.025: StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules); Routes on_data and on_flow_close to ModbusAnalyzer

## Description

`StreamDispatcher` adds `DispatchTarget::Modbus` as the fourth variant of the dispatch
target enum (after `Http`, `Tls`, `None`) and implements Rule 5 in the `classify` function:
flows whose lower or upper port equals 502 are classified as `DispatchTarget::Modbus`
**only if** no content rule (Rule 1: TLS signature, Rule 2: HTTP method prefix) and no
existing port fallback (Rule 3: 443/8443 TLS, Rule 4: 80/8080 HTTP) has fired. This ordering
ensures that TLS or HTTP content appearing on port 502 is never stolen from the correct
analyzer. Once classified, `on_data` routes to `ModbusAnalyzer::on_data` and `on_flow_close`
routes to `ModbusAnalyzer::on_flow_close`. The accessor `modbus_analyzer()` and take pattern
`take_modbus_analyzer()` mirror the existing `tls_analyzer()` / `take_tls_analyzer()` pair.
The VP-004 Kani oracle (`classify_oracle`) must mirror this rule exactly.

## Preconditions

1. `StreamDispatcher` is constructed with `modbus: Option<ModbusAnalyzer>` as the third
   parameter of `StreamDispatcher::new`.
2. `DispatchTarget` enum has been extended with a `Modbus` variant:
   ```rust
   enum DispatchTarget { Http, Tls, Modbus, None }
   ```
3. All exhaustive match arms over `DispatchTarget` in `dispatcher.rs` have been updated.
4. `StreamDispatcher.modbus: Option<ModbusAnalyzer>` field has been added.
5. The `classify` function is the private production classification function. The `classify_oracle`
   in the Kani harness section of `dispatcher.rs` is the formal-verification mirror.

## Postconditions

### P1: DispatchTarget::Modbus classification rule (Rule 5)

The `classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget` function applies rules in
strict priority order:

1. **Rule 1 (content: TLS)**: if `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03` →
   return `DispatchTarget::Tls`. Fires even for port-502 flows.
2. **Rule 2 (content: HTTP)**: if `data` starts with any of the 10 HTTP method tokens
   (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `, `CONNECT `, `TRACE `,
   `HTTP/`) → return `DispatchTarget::Http`. Fires even for port-502 flows.
3. **Rule 3 (port: TLS)**: if lower or upper port is 443 or 8443 → return `DispatchTarget::Tls`.
4. **Rule 4 (port: HTTP)**: if lower or upper port is 80 or 8080 → return `DispatchTarget::Http`.
5. **Rule 5 (port: Modbus)**: if lower or upper port is 502 → return `DispatchTarget::Modbus`.
6. **Rule 6 (no match)**: return `DispatchTarget::None`.

**Critical ordering guarantee**: port 502 is checked only after content rules (1–2) and after
the established 443/8443/80/8080 port fallbacks (3–4). A TLS ClientHello on port 502 is
classified as `Tls` (Rule 1). An HTTP GET request on port 502 is classified as `Http` (Rule 2).
Port 502 with no identifiable content and no conflicting port only reaches Rule 5.

### P2: on_data routing

1. When `routes.get(flow_key) == Some(DispatchTarget::Modbus)`:
   ```rust
   DispatchTarget::Modbus => {
       if let Some(ref mut modbus) = self.modbus {
           modbus.on_data(flow_key, direction, data, offset, timestamp);
       }
   }
   ```
2. If `self.modbus.is_none()` (analyzer disabled), the arm is a no-op — data is silently
   ignored (no error, no finding).
3. The early-exit guard is extended to include `self.modbus.is_none()`:
   ```rust
   if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() {
       return;
   }
   ```

### P3: on_flow_close routing

1. When `routes.remove(flow_key) == Some(DispatchTarget::Modbus)`:
   ```rust
   Some(DispatchTarget::Modbus) => {
       if let Some(ref mut modbus) = self.modbus {
           modbus.on_flow_close(flow_key, reason);
       }
   }
   ```
2. The `unclassified_flows` increment guard is extended:
   ```rust
   Some(DispatchTarget::None) | None => {
       if self.http.is_some() || self.tls.is_some() || self.modbus.is_some() {
           self.unclassified_flows += 1;
       }
   }
   ```
   This ensures that a Modbus-only run (`--modbus` without `--http` or `--tls`) correctly
   counts unclassified flows.

### P4: Accessor and take pattern

1. `pub fn modbus_analyzer(&self) -> Option<&ModbusAnalyzer>` returns `self.modbus.as_ref()`.
2. `pub fn take_modbus_analyzer(&mut self) -> Option<ModbusAnalyzer>` returns `self.modbus.take()`.
3. These methods mirror `tls_analyzer()` / `take_tls_analyzer()` exactly in shape and semantics.
4. `take_modbus_analyzer()` is called ONCE in `main.rs` after `reassembler.finalize()` to
   collect findings and summary. After `take()`, `self.modbus` is `None`.

### P5: VP-004 Kani oracle mirroring

1. The `classify_oracle` function in the Kani harness section of `dispatcher.rs` must include
   the Rule 5 arm for port 502 → `DispatchTarget::Modbus`.
2. If `classify_oracle` omits Rule 5, `verify_content_first_precedence_exhaustive` will fail:
   for any symbolic input where `port_a == 502` or `port_b == 502` and no content rule fires,
   `got` (production) returns `Modbus` while `want` (oracle) returns `None`.
3. The extended oracle is:
   ```rust
   // Rule 5: Modbus port fallback (ADR-005 — MUST mirror production exactly).
   if ports.contains(&502) {
       return DispatchTarget::Modbus;
   }
   ```
   placed after Rules 3–4 and before Rule 6 in `classify_oracle`.

## Invariants

1. **INV-2 (Content-First Dispatch)**: Rules 1–2 inspect payload bytes first. Port 502 is only
   reached when both content rules return no match. TLS and HTTP traffic on port 502 are
   correctly classified by content rules, not stolen by the Modbus port rule.
2. **Modbus port is IANA-assigned**: TCP 502 is the IANA-registered port for Modbus TCP
   (per Modbus.org specification and IANA service list). Rule 5 matches only port 502
   (not any nearby port). There is no range or wildcard.
3. **`DispatchTarget::Modbus` is enum variant 3** (zero-indexed): order in the enum definition
   is `Http, Tls, Modbus, None`. All exhaustive match arms (in `on_data`, `on_flow_close`, and
   any test code) must include `Modbus`.
4. **No Modbus analyzer = silent no-op**: when `self.modbus.is_none()`, `DispatchTarget::Modbus`
   routing arms perform no work. This is consistent with how `Http` and `Tls` arms behave
   when `self.http.is_none()` / `self.tls.is_none()`.
5. **Classification is cached**: once a flow is classified as `DispatchTarget::Modbus` and
   the result is cached in `routes`, all subsequent `on_data` calls for that flow use the
   cached route — no re-classification. This is consistent with BC-2.05.005.
6. **`take_modbus_analyzer` is post-finalize only**: the take pattern moves the analyzer out of
   the dispatcher after all traffic is processed. Calling it mid-run would silently disable
   Modbus routing for subsequent flows (the arm becomes a no-op). The contract is: call once,
   after `reassembler.finalize()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Port-502 flow where first bytes are TLS ClientHello (`0x16 0x03`) | Rule 1 fires; classified as `DispatchTarget::Tls`; ModbusAnalyzer never receives data |
| EC-002 | Port-502 flow where first bytes are `GET /` | Rule 2 fires; classified as `DispatchTarget::Http`; ModbusAnalyzer never receives data |
| EC-003 | Port-502 flow with valid MBAP bytes (no TLS/HTTP content) | Rule 5 fires; classified as `DispatchTarget::Modbus`; ModbusAnalyzer receives data |
| EC-004 | Port-502 flow with non-Modbus, non-TLS, non-HTTP binary data | Rule 5 fires (port match); ModbusAnalyzer receives data; `parse_errors` incremented for invalid ADUs |
| EC-005 | Port-443 flow with MBAP bytes (attacker using Modbus on TLS port) | Rule 3 fires; classified as `DispatchTarget::Tls`; ModbusAnalyzer does NOT receive data |
| EC-006 | `self.modbus.is_none()` (Modbus disabled) + port-502 flow | Rule 5 still fires (classification happens regardless); `on_data` arm is no-op; flow counted in `unclassified_flows` is NOT incremented (it was classified, just with no active analyzer) |
| EC-007 | `classify_oracle` missing Rule 5 | `verify_content_first_precedence_exhaustive` fails for port-502 symbolic inputs |
| EC-008 | `take_modbus_analyzer()` called before `finalize()` | `self.modbus` becomes `None`; subsequent port-502 `on_data` calls are no-ops; findings from flows closed after take are lost. This is a contract violation — take must only be called post-finalize. |
| EC-009 | `unclassified_flows` with Modbus-only run | A flow that never gets classified (Rule 6) while `self.modbus.is_some()` increments `unclassified_flows`. Correctly handled by the extended guard. |
| EC-010 | Fragmented MBAP header: first on_data call delivers bytes 0-5 (incomplete header) | Flow is classified as `DispatchTarget::Modbus` (port match); `on_data` is called with 6 bytes; `parse_mbap_header` returns `None` (< 8 bytes); `parse_errors` incremented; subsequent on_data call with remaining bytes completes the ADU |

## Canonical Test Vectors

| Setup | Expected Behavior | Category |
|-------|------------------|----------|
| `FlowKey` with port 502; first data bytes `\x00\x01\x00\x00\x00\x06\x01\x03` (valid MBAP) | `classify` returns `DispatchTarget::Modbus` | happy-path: port-502 classification |
| `FlowKey` with port 502; first data bytes `\x16\x03\x03...` (TLS ClientHello) | `classify` returns `DispatchTarget::Tls` (Rule 1 wins) | edge-case: TLS on Modbus port |
| `FlowKey` with port 502; first data bytes `GET /api HTTP/1.1` | `classify` returns `DispatchTarget::Http` (Rule 2 wins) | edge-case: HTTP on Modbus port |
| `FlowKey` with port 443; first data bytes `\x00\x01\x00\x00\x00\x06\x01\x03` (MBAP) | `classify` returns `DispatchTarget::Tls` (Rule 3 wins) | edge-case: MBAP on TLS port |
| `FlowKey` with port 8080; first data bytes `\x00\x01\x00\x00\x00\x06\x01\x03` (MBAP) | `classify` returns `DispatchTarget::Http` (Rule 4 wins) | edge-case: MBAP on HTTP port |
| `FlowKey` with port 9999; first data bytes `\x00\x01\x00\x00\x00\x06\x01\x03` (MBAP, unknown port) | `classify` returns `DispatchTarget::None` (no rule matches) | edge-case: MBAP on non-Modbus unknown port |
| `FlowKey` with port 502; `self.modbus = None`; valid MBAP data | Classification cached as `Modbus`; `on_data` arm is no-op; no findings | edge-case: Modbus disabled |
| Kani: symbolic 8-byte data + symbolic ports, Rule 1–4 not matching, `port_a == 502` | `classify` and `classify_oracle` both return `Modbus` | Kani: VP-004 oracle consistency |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | `classify_oracle` must include Rule 5 (port 502 → `DispatchTarget::Modbus`) with identical placement relative to Rules 1–4; `verify_content_first_precedence_exhaustive` covers port-502 symbolic inputs | Kani: extended oracle per architecture-delta.md §3.6; proof verifies `classify(data, key) == classify_oracle(data, lower, upper)` for all symbolic 8-byte inputs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the dispatcher integration contract that routes Modbus TCP flows to the ICS analysis capability; without this routing rule, the ModbusAnalyzer would never receive data even when constructed |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — this BC explicitly encodes INV-2: content rules 1–2 fire before port rules 3–5; TLS/HTTP traffic on port 502 cannot be stolen by the Modbus rule) |
| Architecture Module | SS-05 (dispatcher.rs); SS-14 (analyzer/modbus.rs C-22) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | N/A (routing/dispatch contract, not a detection) |

## Related BCs

- BC-2.05.001 — composes with (Rule 1: TLS content signature; still fires before Rule 5 for port-502 TLS flows)
- BC-2.05.002 — composes with (Rule 2: HTTP method prefix; still fires before Rule 5 for port-502 HTTP flows)
- BC-2.05.003 — composes with (Rules 3–4: TLS/HTTP port fallbacks; checked before Rule 5)
- BC-2.05.005 — composes with (classification caching: Modbus flows are cached after first classification)
- BC-2.05.008 — composes with (early-exit guard extended with `self.modbus.is_none()`)
- BC-2.14.023 — depends on (this BC assumes ModbusAnalyzer is constructed per BC-2.14.023; dispatch is a no-op if `self.modbus == None`)
- BC-2.14.001 through BC-2.14.022 — all governed by (this BC establishes the routing precondition that `on_data` reaches ModbusAnalyzer; all analyzer BCs assume routing is in place)

## Architecture Anchors

- `src/dispatcher.rs` — `DispatchTarget::Modbus` enum variant (new fourth variant)
- `src/dispatcher.rs` — `modbus: Option<ModbusAnalyzer>` field on `StreamDispatcher`
- `src/dispatcher.rs` — `classify` function Rule 5: `if ports.contains(&502) { return DispatchTarget::Modbus; }`
- `src/dispatcher.rs` — `on_data` Modbus arm (architecture-delta.md §3.4)
- `src/dispatcher.rs` — `on_flow_close` Modbus arm (architecture-delta.md §3.5)
- `src/dispatcher.rs` — `unclassified_flows` guard extension (architecture-delta.md §3.5)
- `src/dispatcher.rs` — `classify_oracle` Rule 5 extension (VP-004 requirement, architecture-delta.md §3.6)
- `src/dispatcher.rs` — `modbus_analyzer()` and `take_modbus_analyzer()` accessor pair (architecture-delta.md §3.2)
- `.factory/phase-f2-spec-evolution/architecture-delta.md §3` — complete Dispatcher Integration Design

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-004 — `verify_content_first_precedence_exhaustive`: extended to cover `DispatchTarget::Modbus` via `classify_oracle` Rule 5 (architecture-delta.md §3.6, critical requirement)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §3 (Dispatcher Integration Design, all sub-sections); architecture-delta.md §3.6 ("VP-004 Kani Harness Extension — CRITICAL"); ADR-005 (binary ICS protocol integration decision) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes — same flow key and data bytes always produce same dispatch target |
| **Overall classification** | effectful shell (mutates `self.routes` HashMap, `self.modbus` Option) |
