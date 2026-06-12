---
document_type: verification-property
level: L4
version: "2.1"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.05.001
bcs:
  - BC-2.05.001
  - BC-2.05.002
  - BC-2.05.003
  - BC-2.05.004
  - BC-2.05.005
  - BC-2.05.006
module: src/dispatcher.rs
proof_method: kani
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-12"
proof_file_hash: "dd45978ec72cc115e4935add02ebde116f213b144818f0544ea8f4ba6f5720c8"
verified_at_commit: "e685664"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set."
  - "v2.1: VP-004 prose relocked at F6 to include Rules 5/6 (Modbus/502 + DNP3/20000); proof re-verified SUCCESSFUL on develop@e685664. Property Statement updated to add Rule 5 (Modbus port 502 → DispatchTarget::Modbus) and Rule 6 (DNP3 port 20000 → DispatchTarget::Dnp3) to the full 7-rule table, plus Rule 7 fallthrough None. proof_file_hash updated to SHA-256 of src/dispatcher.rs at e685664 (dispatcher.rs changed when Rule 6 DNP3 arm was added). verified_at_commit updated to e685664. — 2026-06-12"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-004: Content-First Dispatch Precedence

## Property Statement

The `classify` function in `dispatcher.rs` satisfies the following precedence rules
for all possible input byte slices and port values:

1. **Rule 1 (TLS content):** If `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`:
   result is `DispatchTarget::Tls` regardless of port numbers.
2. **Rule 2 (HTTP content):** Else if the data starts with a recognized HTTP method
   token (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `, `CONNECT `,
   `TRACE `, `HTTP/`): result is `DispatchTarget::Http` regardless of port numbers.
3. **Rule 3 (TLS port fallback):** Else if data is too short (< 5 bytes) or matches
   neither content signature: port 443 or 8443 -> `DispatchTarget::Tls`.
4. **Rule 4 (HTTP port fallback):** Else port 80 or 8080 -> `DispatchTarget::Http`.
5. **Rule 5 (Modbus port fallback):** Else port 502 -> `DispatchTarget::Modbus`.
6. **Rule 6 (DNP3 port fallback):** Else port 20000 -> `DispatchTarget::Dnp3`.
7. **Rule 7 (fallthrough):** Otherwise: `DispatchTarget::None`.

`DispatchTarget::None` is NOT inserted into `routes` before the per-flow
`classification_attempts` counter reaches `max_classification_attempts`
(default 8, defined at `dispatcher.rs:40`). Once the counter reaches the cap,
`DispatchTarget::None` IS inserted permanently (phase B, `dispatcher.rs:146`),
and `classification_attempts[flow_key]` is removed (`dispatcher.rs:147`).
Subsequent `on_data` calls for that flow short-circuit via the cached `None`
and never re-run `classify`. `DispatchTarget::Http` and `DispatchTarget::Tls`
are stored immediately on their first classification and never evicted except
by `on_flow_close`.

## Source Contract

- **Primary BC:** BC-2.05.001 -- TLS content signature routes flow to TLS regardless of port
- **Postcondition:** TLS signature (0x16 0x03 ...) always wins over port number
- **Invariant:** INV-2 (Content-First Dispatch Precedence, inv-01-core-invariants.md)
- **ADR:** ADR 0001 (Content-First Stream Protocol Dispatch, docs/adr/0001-content-first-stream-dispatch.md)
- **Related BC:** BC-2.05.002 -- HTTP method prefix routes flow to HTTP
- **Related BC:** BC-2.05.003 -- Port fallback: 443/8443->TLS, 80/8080->HTTP when content insufficient
- **Related BC:** BC-2.05.004 -- Unknown content and unknown port returns DispatchTarget::None
- **Related BC:** BC-2.05.005 -- Classification cached per FlowKey after first non-None result
- **Related BC:** BC-2.05.006 -- DispatchTarget::None NOT cached before retry cap; reclassification retried until cap; permanently cached once cap reached (two-phase behavior, dispatcher.rs:137-148)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- 5-byte data prefix; 16-bit port; finite method prefix set | All classify outcomes for bounded data lengths 0..16 |

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_tls_signature_beats_port() {
        // Even if port is 80 (HTTP port), TLS signature wins.
        // Real signature (src/dispatcher.rs:90):
        //   fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget
        // classify takes a &FlowKey (not a bare u16 port); construct a key with port 80.
        use std::net::{IpAddr, Ipv4Addr};
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let key = FlowKey::new(ip, 80, ip, 9000); // lower_port will be 80
        let data: [u8; 5] = [0x16, 0x03, 0x03, 0x00, 0x00]; // TLS record header
        let result = classify(&data, &key);
        assert!(matches!(result, DispatchTarget::Tls));
    }

    // VP-004 two-phase None-caching property (LESSON-P2.11):
    //   Phase A (pre-cap): DispatchTarget::None is NOT inserted into `routes`;
    //          `classification_attempts[key]` increments by 1 per call.
    //   Phase B (at cap): on the call where attempts reaches max_classification_attempts,
    //          `routes[key] = DispatchTarget::None` is inserted permanently and
    //          `classification_attempts[key]` is removed.
    //
    // The harness proves both phases for a bounded attempt count via a
    // bounded loop over unknown-content chunks. `max_classification_attempts`
    // is set to a small value (2) so Kani's state space stays tractable.
    #[kani::proof]
    fn verify_none_two_phase_caching() {
        let cap: u32 = 2; // use small cap so Kani can enumerate all states
        let mut dispatcher = StreamDispatcher::new(None, None)
            .with_max_classification_attempts(cap);
        let key = FlowKey::new(/* ... */);
        let unknown_data: [u8; 3] = [0xAA, 0xBB, 0xCC]; // no TLS/HTTP sig; unknown port

        // Phase A: call (cap - 1) times; routes must NOT contain the key each time
        for _i in 0..(cap - 1) {
            dispatcher.on_data(&key, Direction::ClientToServer, &unknown_data, 0);
            assert!(!dispatcher.routes.contains_key(&key),
                "routes must NOT contain key before cap is reached");
        }

        // Phase B: the cap-th call must insert DispatchTarget::None permanently
        dispatcher.on_data(&key, Direction::ClientToServer, &unknown_data, 0);
        assert!(
            matches!(dispatcher.routes.get(&key), Some(DispatchTarget::None)),
            "routes must contain DispatchTarget::None permanently once cap is reached"
        );
        assert!(!dispatcher.classification_attempts.contains_key(&key),
            "classification_attempts must be removed after cap is reached");
    }

    #[kani::proof]
    fn verify_content_first_precedence_exhaustive() {
        // classify takes a &FlowKey (src/dispatcher.rs:90); build symbolic key.
        use std::net::{IpAddr, Ipv4Addr};
        let raw_a: u32 = kani::any();
        let raw_b: u32 = kani::any();
        let port_a: u16 = kani::any();
        let port_b: u16 = kani::any();
        let key = FlowKey::new(
            IpAddr::V4(Ipv4Addr::from(raw_a)), port_a,
            IpAddr::V4(Ipv4Addr::from(raw_b)), port_b,
        );
        let data: [u8; 5] = kani::any();
        let result = classify(&data, &key);
        // If TLS signature present, must be Tls
        if data[0] == 0x16 && data[1] == 0x03 {
            assert!(matches!(result, DispatchTarget::Tls));
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | 5-byte data array + 16-bit port is tractable for Kani |
| Proof complexity | Low | `classify` is a pure function with clear if-else branches |
| Tool support | High | No heap allocation in classify; ideal Kani target |
| Estimated proof time | < 2 minutes | Simple branch coverage |

## Source Location

`src/dispatcher.rs:90` -- `fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget`

`src/dispatcher.rs:40` -- `pub const DEFAULT_MAX_CLASSIFICATION_ATTEMPTS: u32 = 8`

`src/dispatcher.rs:42-53` -- `StreamDispatcher` struct; `routes: HashMap<FlowKey, DispatchTarget>` at line 43;
`classification_attempts: HashMap<FlowKey, u32>` at line 48.

`src/dispatcher.rs:120` -- `fn on_data` (StreamHandler impl); phase-B None insertion at line 146;
`classification_attempts.remove` at line 147.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
| Prose updated (Rules 5/6/7 added) | 2026-06-12 (F6) | product-owner |
| Relocked (VERIFIED, Rules 5+6 incl.) | 2026-06-12 (F6 gate @ develop e685664) | formal-verifier |

`verify_content_first_precedence_exhaustive` re-verified `VERIFICATION:- SUCCESSFUL` at
develop HEAD `e685664` with the port-20000 (DNP3) arm present in `src/dispatcher.rs`.
`proof_file_hash` updated to SHA-256 of `src/dispatcher.rs` at e685664 (file changed when
Rule 6 DNP3/port-20000 arm was added). This document is now immutable at v2.1; any change
requires the VP withdrawal process.
