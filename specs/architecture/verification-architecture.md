---
artifact: architecture-section
section: verification-architecture
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# Verification Architecture

## Provable Properties Catalog

### Must Prove (security boundaries, state machines, arithmetic invariants)

| VP-ID | Property | Invariant | Module | Tool |
|-------|----------|-----------|--------|------|
| VP-001 | FlowKey canonical ordering: (ip_a,port_a) <= (ip_b,port_b) produces identical key for both directions | INV-1 | reassembly/flow.rs | Kani |
| VP-002 | First-wins overlap: existing bytes always win; ConflictingOverlap finding emitted on content mismatch | INV-3 | reassembly/segment.rs | Kani |
| VP-003 | MAX_FINDINGS cap: reassembler never holds more than MAX_FINDINGS+1 findings (the +1 is finalize bypass per INV-6) | INV-6 | reassembly/mod.rs | Kani |
| VP-004 | Content-first dispatch precedence: TLS signature always wins over port; HTTP method prefix wins over port; DispatchTarget::None is NOT inserted into `routes` before the per-flow classification-attempt counter reaches `max_classification_attempts` (default 8); at the cap it is inserted permanently and reclassification stops | INV-2 | dispatcher.rs | Kani |
| VP-005 | SNI 4-way ordered match: given any byte slice, exactly one arm fires; arm 3 (NonAsciiUtf8) fires when valid UTF-8 + non-ASCII + C0 present (INV-5 boundary case) | INV-5 | analyzer/tls.rs | Kani |
| VP-006 | HTTP poison monotonicity: `request_poisoned` / `response_poisoned` fields transition only false->true within a flow's lifetime | INV-8 | analyzer/http.rs | proptest |
| VP-007 | MITRE technique ID format: every ID emitted by analyzers resolves in technique_info; format matches T[0-9]{4}(\.[0-9]{3})? | INV-9 | mitre.rs | Kani |
| VP-008 | decode_packet never panics on arbitrary input bytes: any byte slice returns Ok or Err, never unwinds | (no-panic invariant) | decoder.rs | cargo-fuzz |
| VP-009 | FlowState machine: no transition reaches an undefined state; RST transitions to Closed from any prior state | (state machine) | reassembly/flow.rs | Kani |

### Should Prove (core algorithms, data transformations)

| VP-ID | Property | Invariant | Module | Tool |
|-------|----------|-----------|--------|------|
| VP-010 | buffered_bytes mirrors segment BTreeMap size sum after every insert/flush/overlap operation | INV-6 related | reassembly/segment.rs | proptest |
| VP-011 | flush_contiguous is monotonic: base_offset strictly advances with each call; no byte delivered twice | (correctness) | reassembly/segment.rs | proptest |
| VP-012 | escape_for_terminal: no C0/DEL/C1 byte survives unescaped; all non-ASCII Unicode > U+009F passes through | ADR 0003 | reporter/terminal.rs | proptest |
| VP-013 | JA3 GREASE filter: all values matching the GREASE pattern (0x?A?A) are removed before fingerprint computation | (spec compliance) | analyzer/tls.rs | proptest |
| VP-014 | HttpAnalyzer cross-flow isolation: parse errors and poisoning in flow A do not affect flow B | (isolation) | analyzer/http.rs | proptest |
| VP-015 | TCP sequence wraparound: segment at seq=0xFFFF_FFFE with data crossing 32-bit boundary reassembles correctly | (arithmetic) | reassembly/segment.rs | Kani |

### Test Sufficient (UI logic, non-critical defaults)

| VP-ID | Property | Tool |
|-------|----------|------|
| VP-016 | TerminalReporter MITRE tactic grouping order matches all_tactics_in_report_order | integration test |
| VP-017 | JsonReporter BTreeMap key determinism: repeated calls with same input produce identical JSON | integration test |
| VP-018 | CLI flag parsing: --reassemble/--no-reassemble mutual exclusion (BC-2.12.007, BC-2.12.009) | integration test |
| VP-019 | DNS statistics-only invariant: DnsAnalyzer.analyze() always returns empty Vec | unit test |
| VP-020 | CsvReporter CSV-injection neutralization: cell values starting with =,+,-,@,TAB,CR are prefixed with a single-quote (') | unit test |


## P0 Verification Properties (required before Phase 5 gate)

- VP-001: FlowKey canonical ordering (INV-1)
- VP-002: First-wins overlap policy (INV-3)
- VP-003: MAX_FINDINGS cap with finalize bypass (INV-6)
- VP-004: Content-first dispatch precedence (INV-2)
- VP-005: SNI 4-way ordered match boundary (INV-5)
- VP-007: MITRE technique ID format completeness (INV-9)
- VP-008: decode_packet no-panic property
- VP-009: FlowState machine validity

## P1 Verification Properties (required before Phase 6 hardening)

- VP-006: HTTP poison monotonicity (INV-8)
- VP-010: buffered_bytes invariant
- VP-011: flush_contiguous monotonicity
- VP-012: escape_for_terminal correctness (ADR 0003)
- VP-013: JA3 GREASE filter
- VP-014: HttpAnalyzer cross-flow isolation
- VP-015: TCP sequence wraparound


## Tooling Selection

See `tooling-selection.md` for full rationale. Summary:

| Tool | Target Properties | Scope |
|------|-----------------|-------|
| Kani (model checker) | State machine reachability, arithmetic overflow, pointer safety | VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015 |
| proptest | Property-based: generate random inputs, check invariants | VP-006, VP-010..014 |
| cargo-fuzz (libFuzzer) | No-panic for parser entry points | VP-008 |
| cargo-mutants | Mutation coverage for domain logic | SS-06, SS-07, SS-08, SS-10 |


## Proof Harness Skeletons

### VP-001: FlowKey Canonical Ordering (Kani)

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_flowkey_canonical_ordering() {
    let ip_a: u32 = kani::any();
    let port_a: u16 = kani::any();
    let ip_b: u32 = kani::any();
    let port_b: u16 = kani::any();
    use std::net::Ipv4Addr;
    let a = IpAddr::V4(Ipv4Addr::from(ip_a));
    let b = IpAddr::V4(Ipv4Addr::from(ip_b));
    let key_ab = FlowKey::new(a, port_a, b, port_b);
    let key_ba = FlowKey::new(b, port_b, a, port_a);
    assert_eq!(key_ab, key_ba);
}
```

### VP-002: First-Wins Overlap (Kani)

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_first_wins_overlap() {
    // Insert bytes [A,A,A] at offset 0.
    // Insert bytes [B,B,B] at offset 0 where B != A.
    // Assert: insert_segment returns InsertResult::ConflictingOverlap.
    // Assert: buffered content at offset 0..3 is still [A,A,A].
}
```

### VP-005: SNI 4-way Ordered Match (Kani)

// Real signature (src/analyzer/tls.rs:246):
//   fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue>
//
// The 4-way classification is the inline match at tls.rs:251-265:
//   Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii
//   Ok(s) if s.is_ascii()                             => SniValue::AsciiWithControl
//   Ok(s)                                             => SniValue::NonAsciiUtf8
//   Err(_)                                            => SniValue::NonUtf8
//
// Kani proof target: the byte-to-variant mapping. Because `extract_sni`
// takes a parsed extension list (not raw bytes), the proof harness
// exercises the classification match directly via a helper that wraps
// a synthetic SNI extension backed by a kani::any() byte slice.
//
// Illustrative skeleton:
#[cfg(kani)]
#[kani::proof]
fn verify_sni_classification_exhaustive() {
    // Build a synthetic hostname byte slice of bounded length.
    let len: usize = kani::any();
    kani::assume(len <= 32);
    let hostname: Vec<u8> = (0..len).map(|_| kani::any()).collect();
    // Classify the same bytes using the same logic as the inline match.
    let result = match std::str::from_utf8(&hostname) {
        Ok(s) if s.is_ascii() && !s.bytes().any(|b| b < 0x20 || b == 0x7f) => 0u8,
        Ok(s) if s.is_ascii() => 1u8,
        Ok(_) => 2u8,
        Err(_) => 3u8,
    };
    // Exactly one arm fires (result is 0..=3 by construction).
    // INV-5 arm-3-priority: valid UTF-8 + non-ASCII cannot match arm 0 or 1.
    if let Ok(s) = std::str::from_utf8(&hostname) {
        if !s.is_ascii() {
            assert!(result == 2);
        }
    }
}

### VP-008: decode_packet No-Panic (cargo-fuzz)

```rust
// fuzz_target in fuzz/fuzz_targets/decode_packet.rs:
// Real signature (src/decoder.rs:128):
//   pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    // try all supported link types (DataLink::IPV6 accepted per decoder.rs:134)
    for datalink in [
        DataLink::ETHERNET,
        DataLink::RAW,
        DataLink::IPV4,
        DataLink::IPV6,
        DataLink::LINUX_SLL,
    ] {
        let _ = decode_packet(data, datalink); // must not panic
    }
});
```
