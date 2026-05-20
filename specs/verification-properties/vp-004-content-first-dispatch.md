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

`DispatchTarget::None` is NEVER stored in the `routes` HashMap.
`DispatchTarget::Http` and `DispatchTarget::Tls` are stored and never evicted
except by `on_flow_close`.

## Source Contract

- **Primary BC:** BC-2.05.001 -- TLS content signature routes flow to TLS regardless of port
- **Postcondition:** TLS signature (0x16 0x03 ...) always wins over port number
- **Invariant:** INV-2 (Content-First Dispatch Precedence, inv-01-core-invariants.md)
- **ADR:** ADR 0001 (Content-First Stream Protocol Dispatch, docs/adr/0001-content-first-stream-dispatch.md)
- **Related BC:** BC-2.05.002 -- HTTP method prefix routes flow to HTTP
- **Related BC:** BC-2.05.003 -- Port fallback: 443/8443->TLS, 80/8080->HTTP when content insufficient
- **Related BC:** BC-2.05.004 -- Unknown content and unknown port returns DispatchTarget::None
- **Related BC:** BC-2.05.005 -- Classification cached per FlowKey after first non-None result
- **Related BC:** BC-2.05.006 -- DispatchTarget::None is NOT cached; reclassification retried

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

    #[kani::proof]
    fn verify_none_not_cached() {
        let mut dispatcher = StreamDispatcher::new(None, None);
        let key = FlowKey::new(/* ... */);
        // Call on_data with data that classifies as None
        let unknown_data: [u8; 3] = [0xAA, 0xBB, 0xCC]; // no signature; no known port
        dispatcher.on_data(&key, Direction::ClientToServer, &unknown_data, 0);
        // The routes map must NOT contain an entry for this key
        assert!(!dispatcher.routes.contains_key(&key));
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
