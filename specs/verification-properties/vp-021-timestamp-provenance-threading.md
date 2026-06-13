---
document_type: verification-property
level: L4
version: "2.1"
status: verified
producer: product-owner
timestamp: 2026-06-08T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.09.007
bcs:
  - BC-2.09.007
  - BC-2.04.055
module: src/reassembly/mod.rs
proof_method: integration + proptest
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-09"
proof_file_hash: "207d3f685f711c54c093b218bb38ae43b30c56b44a74d2b190828bb44c7cc53f"
verified_at_commit: "256a490"
lifecycle_status: active
introduced: v0.2.0-feature-100
modified:
  - "v2.0: Phase-F6 verification locked 2026-06-09 @ develop 256a490. status→verified, verification_lock→true. Proof evidence: tests/timestamp_threading_tests.rs — VP-021 end-to-end integration test (hot-path + close-flush + segment-limit-None), all-u32 proptest (prop_finding_timestamp_matches_on_data_timestamp), boundary proptest (prop_cross_flow_timestamp_isolation). F6 mutation result: 100% effective kill on the timestamp delta. 1147 tests green. Kani appropriately excluded (inline chrono conversion; totality by closed-form u32-range reasoning + all-u32 proptest + boundary tests). proof_file_hash set (SHA-256 of tests/timestamp_threading_tests.rs @256a490)."
  - "v2.1 (2026-06-13, PG-ARP-F2-007 anchor-drift sweep): Source Location line anchors corrected for F2 shifts. HttpAnalyzer::on_data: http.rs:501→:524. TlsAnalyzer::on_data: tls.rs:771→:798. Lock fields unchanged."
deprecated: null
deprecated_by: null
replacement: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-021: Timestamp Provenance Threading

## Property Statement

For any `Finding` emitted from a flow-data path (hot-path flush via `flush_contiguous_data`
or close-flush via `close_flow`), `Finding.timestamp` equals `Some(ts)` where `ts` is
derived from the `timestamp: u32` argument passed by the reassembler to
`StreamHandler::on_data` at the flush call site, with the following two-case semantics:

1. **Hot-path flush** (`flush_contiguous_data`): `timestamp` equals the current packet's
   `timestamp_secs` — the pcap `ts_sec` of the packet that caused the flush.
2. **Close-flush** (`close_flow` — FIN, RST, timeout, eviction): `timestamp` equals
   `TcpFlow.last_seen` — the most-recently-seen packet timestamp for the flow.

Additionally:
- The segment-limit summary finding emitted from `finalize` retains `timestamp: None`
  (it is a post-capture aggregate, not tied to any specific packet).
- The `u32 → DateTime<Utc>` conversion is lossless for all valid pcap `ts_sec` values.
- Cross-flow isolation holds: a timestamp from flow A does not appear in a Finding from flow B.

**Why NOT Kani:** This property involves `DateTime<Utc>` conversion (chrono crate types),
`HashMap<FlowKey, u32>` per-flow timestamp storage, and the full TCP reassembly pipeline.
These are not pure arithmetic invariants amenable to bounded model checking. Kani's symbolic
execution does not handle arbitrary HashMap state and chrono datetime operations within
tractable proof bounds. Integration tests and proptest are the appropriate tools.

## Source Contracts

- **Primary BC:** BC-2.09.007 — Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site
- **Primary BC:** BC-2.04.055 — StreamHandler::on_data Carries Capture-Relative Timestamp Parameter
- **Postconditions verified:** BC-2.09.007 §Postconditions 1-4; BC-2.04.055 §Postconditions 1-3

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test | Rust test (`tests/reassembly_engine_tests.rs` or new `tests/timestamp_threading_tests.rs`) | N/A | End-to-end: synthetic pcap with known ts_sec → assert Finding.timestamp matches expected DateTime<Utc> |
| Property test | proptest | No — arbitrary ts_sec values | Hot-path and close-flush paths; arbitrary timestamp values; cross-flow isolation |

## Proof Harness Skeleton

```rust
// Integration test: end-to-end pipeline with known timestamp
#[test]
fn test_finding_timestamp_hot_path() {
    // Craft two packets: SYN then HTTP request, both at ts_sec=1_000_000
    // Run through TcpReassembler::process_packet → HttpAnalyzer::on_data
    // Assert Finding.timestamp == Some(DateTime::from_timestamp(1_000_000, 0).unwrap())
    let expected_ts = DateTime::from_timestamp(1_000_000, 0).unwrap();
    // ... build reassembler + http analyzer + run packets ...
    let findings = reassembler.take_findings();
    let http_findings: Vec<_> = findings.iter()
        .filter(|f| f.timestamp.is_some())
        .collect();
    assert!(!http_findings.is_empty(), "expected at least one finding with timestamp");
    for f in http_findings {
        assert_eq!(f.timestamp, Some(expected_ts),
            "finding timestamp does not match expected pcap ts_sec");
    }
}

#[test]
fn test_finding_timestamp_segment_limit_summary_is_none() {
    // Drive reassembler past MAX_SEGMENTS_PER_DIRECTION to produce segment-limit summary
    // Assert the segment-limit Finding has timestamp = None
    // ... (see BC-2.04.054 for setup pattern) ...
    let findings = reassembler.take_findings();
    let summary = findings.iter().find(|f| f.summary.contains("segment limit"));
    assert!(summary.is_some(), "expected segment-limit summary finding");
    assert_eq!(summary.unwrap().timestamp, None,
        "segment-limit summary must have timestamp: None");
}

// Proptest: arbitrary ts_sec → Finding.timestamp matches
proptest! {
    #[test]
    fn prop_finding_timestamp_matches_on_data_timestamp(
        ts_sec in 0u32..u32::MAX,
    ) {
        // Build minimal reassembler + HttpAnalyzer
        // Inject SYN + HTTP request packet at ts_sec
        // Collect findings; assert all non-summary findings have timestamp = Some(ts)
        let expected = DateTime::from_timestamp(ts_sec as i64, 0).unwrap();
        // ... drive pipeline ...
        for finding in non_summary_findings {
            prop_assert_eq!(finding.timestamp, Some(expected));
        }
    }

    #[test]
    fn prop_cross_flow_timestamp_isolation(
        ts_a in 1u32..500_000u32,
        ts_b in 500_001u32..1_000_000u32,
    ) {
        // Two distinct flows A and B; A's packets at ts_a, B's packets at ts_b
        // Assert: findings attributed to flow A have timestamp derived from ts_a
        // Assert: findings attributed to flow B have timestamp derived from ts_b
        // (requires distinct src IPs to differentiate flow A/B findings)
        // ...
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded for integration test (single known ts_sec); unbounded for proptest (arbitrary u32) | |
| Proof complexity | Medium | End-to-end involves full reassembly + HTTP/TLS analyzer pipeline; proptest tests key subset |
| Tool support | High | Standard Rust integration tests + proptest (already used by VP-006, VP-014) |
| Estimated proof time | < 60 seconds (proptest); < 5 seconds (integration) | |
| Kani suitability | NOT SUITABLE | DateTime conversion, HashMap state, and full reassembly pipeline exceed tractable Kani bounds |
| Implementation dependency | Yes — requires F4 Story A (trait wiring) and Story B (analyzer emission) to be complete | Proof is unverified until F4 delivers |

## Source Location

- `src/reassembly/mod.rs` — `flush_contiguous_data`: call site passes `timestamp: u32` to `handler.on_data`
- `src/reassembly/lifecycle.rs` — `close_flow`: call site passes `flow.last_seen` to `handler.on_data`
- `src/reassembly/handler.rs:49` — `StreamHandler::on_data` trait method (new `timestamp: u32` parameter)
- `src/analyzer/http.rs:524` — `HttpAnalyzer::on_data` stores per-flow timestamp; emission sites use it
- `src/analyzer/tls.rs:798` — `TlsAnalyzer::on_data` stores per-flow timestamp; emission sites use it
- `src/findings.rs` — `Finding.timestamp: Option<DateTime<Utc>>` (no change needed; field already exists)

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created (draft) | 2026-06-08 | product-owner (F2 spec evolution) |
| Tests committed | 2026-06-09 | test-writer (STORY-099 / Feature #100 F4) |
| Tests passing (1147 green) | 2026-06-09 | formal-verifier (F6 hardening PASS) |
| Locked (VERIFIED) | 2026-06-09 | spec-steward (Phase-F6 gate @ develop 256a490) |
