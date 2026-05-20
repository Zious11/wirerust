---
document_type: verification-property
level: L4
version: "1.0"
status: draft
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
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
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

1. If `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`:
   result is `DispatchTarget::Tls` regardless of port numbers.
2. Else if the data starts with a recognized HTTP method token (`GET `, `POST `,
   `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `, `CONNECT `, `TRACE `, `HTTP/`):
   result is `DispatchTarget::Http` regardless of port numbers.
3. Else if data is too short (< 5 bytes) or matches neither signature:
   port 443 or 8443 -> `DispatchTarget::Tls`; port 80 or 8080 -> `DispatchTarget::Http`.
4. Otherwise: `DispatchTarget::None`.

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
        // Even if port is 80 (HTTP port), TLS signature wins
        let port: u16 = 80;
        let data: [u8; 5] = [0x16, 0x03, 0x03, 0x00, 0x00]; // TLS record header
        let result = classify(&data, port);
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
        let mut data: [u8; 5] = kani::any();
        let port: u16 = kani::any();
        let result = classify(&data, port);
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

`src/dispatcher.rs` -- `classify()` function and `routes: HashMap<FlowKey, DispatchTarget>`.

`src/dispatcher.rs:37-79` per ingestion corpus reference.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
