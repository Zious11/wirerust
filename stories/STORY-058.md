---
document_type: story
story_id: "STORY-058"
epic_id: "E-5"
version: "1.5"
status: draft
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.005.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.029.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.031.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.033.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.035.md
input-hash: "360abff"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-052, STORY-053]
blocks: [STORY-076]
behavioral_contracts:
  - BC-2.07.004
  - BC-2.07.005
  - BC-2.07.029
  - BC-2.07.031
  - BC-2.07.033
  - BC-2.07.035
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 18
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-REL-003
  - NFR-REL-010
  - NFR-RES-014
  - NFR-RES-015
  - NFR-RES-016
  - NFR-OBS-002
  - NFR-SEC-008
  - NFR-RES-020
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-058`

# STORY-058: Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output

## Narrative
- **As a** forensic analyst
- **I want** the TLS analyzer to enforce per-direction buffer caps and per-record payload limits, gracefully handle parse failures and non-handshake records without panicking, clean up per-flow state on flow close, and emit a complete `AnalysisSummary` with all required TLS statistics fields in alphabetically ordered BTreeMap keys
- **So that** the analyzer is memory-safe under adversarial traffic, statistically complete in its summary output, and correctly interoperable with the reporting pipeline

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.004 | TLS Record Payload > MAX_RECORD_PAYLOAD Increments parse_errors and truncated_records |
| BC-2.07.005 | Per-Direction Buffer Capped at MAX_BUF = 65536 Bytes |
| BC-2.07.029 | Bad TLS Record Body Increments parse_errors; No Panic |
| BC-2.07.031 | summarize Emits AnalysisSummary with TLS Stats Detail Map |
| BC-2.07.033 | TLS Analyzer Ignores Non-Handshake Records |
| BC-2.07.035 | on_flow_close Drops Per-Flow TlsFlowState |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.004 postcondition 1-4)
When `try_parse_records` reads the 5-byte TLS record header and finds `payload_len > MAX_RECORD_PAYLOAD` (18,432 bytes), both `parse_errors` and `truncated_records` are incremented by 1. The direction buffer (`client_buf` or `server_buf`) is cleared entirely. `try_parse_records` returns. No finding is emitted. `handshakes_seen` is NOT incremented.
- **Test:** `test_oversized_sni_exceeds_record_payload_limit`

### AC-002 (traces to BC-2.07.004 invariant 1-2)
`parse_errors` and `truncated_records` are ALWAYS incremented together for oversized records — never independently. Buffer clearing is unconditional: ALL buffered bytes for that direction are dropped. (The `clear()` is unconditional by construction — note per BC-2.07.004 v1.3 that the specific scenario of a valid partial record preceding an oversized record at a later buffer offset is not reachable via `on_data`, since the parser reads from `buf[0]` and returns at the incompleteness check first; so any assertion about "preceding partial records" is defensive/by-inspection, not exercised by a discriminating test.)
- **Test:** `test_oversized_after_valid_hello_increments_both`

### AC-003 (traces to BC-2.07.004 edge case EC-001)
A TLS record with `payload_len = 18,432` exactly (the boundary, at or equal) is accepted for parsing — no `truncated_records` increment. A record with `payload_len = 18,433` (one over) triggers both increments.
- **Test:** `test_record_payload_boundary_18432_vs_18433`

### AC-004 (traces to BC-2.07.005 postcondition 1-4)
When `on_data` is called with new bytes for a flow direction that has not yet completed both hellos, at most `MAX_BUF - current_buf_len` bytes from `data` are appended to the buffer. If the buffer is already full (`current_buf_len >= MAX_BUF = 65,536`), no bytes are appended. No error is returned; no counter is incremented for buffer overflow.
- **Test:** `test_buffer_cap_appends_at_most_max_buf`; `test_buffer_cap_appends_at_most_max_buf_literal_residue`

### AC-005 (traces to BC-2.07.005 invariant 1-2)
`client_buf.len()` and `server_buf.len()` are always `<= MAX_BUF = 65,536`. The cap is computed as `remaining = MAX_BUF.saturating_sub(state.buf.len()); to_copy = data.len().min(remaining)`. This is a non-panicking calculation for any input size.
- **Test:** `test_buffer_full_append_noop`; `test_buffer_full_append_noop_literal`

### AC-006 (traces to BC-2.07.005 invariant 3)
Buffer overflow is silent. No finding, no log line, and no counter tracks how many bytes were dropped beyond the cap. `parse_errors` and `truncated_records` are NOT incremented for buffer overflow.
- **Test:** `test_buffer_overflow_silent_no_counters`

### AC-007 (traces to BC-2.07.029 postcondition 1-5)
When `parse_tls_plaintext` is called on a well-sized record (payload_len <= MAX_RECORD_PAYLOAD) with `record_type == 0x16` (Handshake) and returns `Err(_)`, `parse_errors` is incremented by 1. No finding is emitted, no panic occurs, the flow state remains in the `flows` HashMap, and the parsing loop continues or returns.
- **Test:** `test_parse_error_counter`

### AC-008 (traces to BC-2.07.029 invariant 1-2)
`parse_errors` increments ONLY for genuine parse failures (nom `Err(_)` on a handshake record). Oversized records use BOTH `parse_errors` AND `truncated_records`. The difference `parse_errors - truncated_records` counts genuine parse failures that are not DoS-protection drops.
- **Test:** `test_malformed_handshake_increments_parse_errors_only`

### AC-009 (traces to BC-2.07.031 postcondition 1-9)
`TlsAnalyzer::summarize` returns an `AnalysisSummary` with `analyzer_name = "TLS"`, `packets_analyzed = handshakes_seen`, and a `detail` BTreeMap containing all required keys: `"cipher_suites"`, `"ja3_hashes"`, `"ja3s_hashes"`, `"parse_errors"`, `"tls_versions"`, `"top_snis"`, `"truncated_records"`. `top_snis` is a JSON array of up to 20 SNI strings sorted by count descending; ties broken by SNI name ascending (lexicographic); deterministic across runs regardless of HashMap/insertion order. Other keys are JSON objects/numbers.
- **Test:** `test_summarize_output`; integration test `test_summarize_has_all_required_fields`

### AC-010 (traces to BC-2.07.031 invariant 1-4)
`detail` is a `BTreeMap` (NOT a HashMap), ensuring alphabetically ordered keys in JSON output (LESSON-P2.09 compliance). `top_snis` contains at most 20 entries; sorted by count descending with ties broken by SNI name ascending, then `.take(20)`; the resulting array is fully deterministic given the same (sni, count) pairs regardless of `sni_counts` HashMap internal ordering or insertion sequence. `version_counts` u16 keys are converted to decimal String via `k.to_string()` for the JSON map.
- **Test:** `test_summarize_output`; `test_summarize_top_snis_capped_at_20` (assert `detail` key ordering and `top_snis.len() <= 20`)

### AC-011 (traces to BC-2.07.031 postcondition 8-9)
`detail["parse_errors"]` is a JSON number equal to `self.parse_errors`. `detail["truncated_records"]` is a JSON number equal to `self.truncated_records`. Both keys are ALWAYS present, even when both values are 0.
- **Test:** `test_fresh_summarize_truncated_records_zero`

### AC-012 (traces to BC-2.07.033 postcondition 1-4)
In `try_parse_records`, after extracting a complete TLS record with `record_type != 0x16` (non-Handshake: e.g., ChangeCipherSpec 0x14, Alert 0x15, AppData 0x17), the record bytes are consumed (drained from the buffer) and the loop `continue`s. No `parse_errors` increment, no finding emitted, no counter change.
- **Test:** `test_appdata_record_skipped_then_hello`

### AC-013 (traces to BC-2.07.033 invariant 1-2)
Non-handshake records are consumed (drained from the buffer) even though they are not parsed — this prevents buffer stalls. The early-return for `done()` at `on_data` entry (BC-2.07.034) is a separate mechanism; this AC covers the within-loop skip for non-handshake records BEFORE the done check is reached.
- **Test:** `test_within_loop_nonhandshake_skip_before_done` (sends a non-handshake record + ClientHello in one `on_data` call while flow is NOT done, proving the within-loop skip at tls.rs:678-682); `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently` (multi-type coverage of EC-006/EC-007)

### AC-014 (traces to BC-2.07.035 postcondition 1-4)
When `on_flow_close` is called with a `flow_key` present in `self.flows`, `self.flows.remove(flow_key)` is called. The `TlsFlowState` is dropped, freeing `client_buf` and `server_buf` memory. `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`, `handshakes_seen`, `parse_errors`, and `all_findings` are all UNCHANGED. `flows.len()` decreases by 1.
- **Test:** `test_on_flow_close_drops_state_preserves_aggregates`

### AC-015 (traces to BC-2.07.035 invariant 1-2)
Per-flow state cleanup is the ONLY operation in `on_flow_close`; no analysis is performed at close time. The `_reason` parameter (CloseReason) is ignored by `TlsAnalyzer`. If `on_flow_close` is called with a key NOT in `flows`, `HashMap::remove` returns `None` — no panic.
- **Test:** `test_on_flow_close_absent_key_no_panic`

### AC-016 (traces to BC-2.07.031 postcondition 3 / invariant 2 / EC-004)
When multiple SNIs share the same count, the tied group is ordered by SNI name ascending (lexicographic); the full `top_snis` array is deterministic across all runs regardless of `sni_counts` HashMap internal ordering or insertion sequence. Sort key: `b.count.cmp(a.count).then_with(|| a.sni.cmp(b.sni))`.
- **Test:** `test_summarize_top_snis_ties_broken_alphabetically`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `try_parse_records` oversized-record guard | src/analyzer/tls.rs:643-653 | effectful-shell (mutates parse_errors, truncated_records, buffer) |
| `on_data` buffer-append with cap | src/analyzer/tls.rs:726-748 | effectful-shell (mutates client_buf or server_buf) |
| nom error handling in `try_parse_records` | src/analyzer/tls.rs:700-712 | effectful-shell (mutates parse_errors) |
| Non-handshake record skip in `try_parse_records` | src/analyzer/tls.rs:678-682 | effectful-shell (drains buffer, continues loop) |
| `summarize` | src/analyzer/tls.rs:763-808 | pure-core (reads all state; no mutation) |
| `on_flow_close` | src/analyzer/tls.rs:752-754 | effectful-shell (mutates flows HashMap) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `payload_len = 18,432` (boundary, not over) | Accepted; no truncation counter increment |
| EC-002 | `payload_len = 65,535` (max u16) | Both counters incremented; buffer cleared |
| EC-003 | Multiple oversized records in sequence on same flow | Each independently increments both counters |
| EC-004 | Buffer at 65,535 bytes; data is 2 bytes | 1 byte appended; 1 byte dropped |
| EC-005 | Buffer at 0; data is 65,537 bytes | 65,536 bytes appended; 1 byte dropped |
| EC-006 | TLS record with type 0x17 (AppData) | Consumed silently; loop continues |
| EC-007 | TLS record with unknown type 0x18 | Consumed silently (same code path as AppData) |
| EC-008 | Analyzer with no data (fresh instance) | `summarize`: packets_analyzed=0; all maps empty; parse_errors=0; truncated_records=0 |
| EC-009 | More than 20 distinct SNIs seen | `top_snis` has exactly 20 entries |
| EC-010 | `on_flow_close` for key not in `flows` | No-op; no panic |
| EC-011 | Reopening same FlowKey after close | New `TlsFlowState` created fresh on next `on_data` |
| EC-012 | Multiple SNIs with equal counts, inserted in reverse alphabetical order | top_snis tied group appears in ascending alphabetical order; result identical regardless of insertion order (BC-2.07.031 EC-004) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (summarize) | pure-core | Reads all count maps and counters; no mutation; deterministic via BTreeMap |
| src/analyzer/tls.rs (try_parse_records, on_data buffer cap, on_flow_close) | effectful-shell | Mutates buffers, counters, and/or flows HashMap |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,000 |
| Referenced code (tls.rs lines 643-653, 678-682, 700-712, 726-808) | ~5,500 |
| Test files (tls_analyzer_tests.rs buffer/parse/summarize tests) | ~4,500 |
| BC files (6 BCs) | ~7,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~23,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~12%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-016 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement oversized-record guard in `try_parse_records`: if `payload_len > MAX_RECORD_PAYLOAD`, increment `parse_errors += 1` and `truncated_records += 1`, clear buffer, return
4. [ ] Implement per-direction buffer cap in `on_data`: `remaining = MAX_BUF.saturating_sub(state.buf.len()); to_copy = data.len().min(remaining); state.buf.extend_from_slice(&data[..to_copy])`
5. [ ] Implement nom error handling in `try_parse_records`: on `Err(_)`, increment `parse_errors += 1`; no panic; continue/return
6. [ ] Implement non-handshake record skip in `try_parse_records`: `if record_type != 0x16 { drain consumed bytes; continue; }`
7. [ ] Implement `summarize`: return `AnalysisSummary { analyzer_name: "TLS", packets_analyzed: self.handshakes_seen, detail: BTreeMap }` with all 7 required keys
8. [ ] Implement `on_flow_close`: `self.flows.remove(flow_key)` only; no analysis at close time
9. [ ] Write `test_oversized_sni_exceeds_record_payload_limit` (payload_len > 18,432)
10. [ ] Write boundary test: payload_len=18,432 does not increment; 18,433 does
11. [ ] Write buffer cap tests (65,537 bytes appended; 1 byte to full buffer)
12. [ ] Write `test_parse_error_counter` (malformed handshake record)
13. [ ] Write `test_summarize_output` (assert all 7 detail keys; BTreeMap ordering; top_snis cap)
14. [ ] Write `on_flow_close` test (state dropped; aggregate stats preserved)
15. [ ] Run all tests; verify all pass
16. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-052 | `on_data` done-check is the FIRST operation; buffer append happens AFTER the done-check | Buffer cap check at `on_data` entry is the second gate after the done-check | MAX_BUF = 65,536 and MAX_RECORD_PAYLOAD = 18,432 are defined as `const` at file scope |
| STORY-053 | `handle_server_hello` sets `server_hello_seen`; `truncated_records` field is on `TlsAnalyzer` (not `TlsFlowState`) | `parse_errors` and `truncated_records` are aggregate counters on `TlsAnalyzer`, not per-flow | `version_counts` is a `HashMap<u16, u64>` — must convert u16 keys to decimal String for JSON in summarize |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Both `parse_errors` AND `truncated_records` are ALWAYS incremented together for oversized records | BC-2.07.004 invariant 1 | Unit test: AC-002 asserts both counters change together |
| Buffer cap uses `saturating_sub` — must not panic on underflow | BC-2.07.005 invariant 2 | Code review: confirm `MAX_BUF.saturating_sub(...)` not subtraction operator |
| `summarize` uses BTreeMap for `detail` (NOT HashMap) — deterministic JSON key ordering | BC-2.07.031 invariant 1 (LESSON-P2.09) | Code review: confirm `BTreeMap::new()` at summarize |
| `top_snis` sorted by count descending; ties broken by SNI name ascending; `.take(20)`; deterministic across runs | BC-2.07.031 invariant 2 / postcondition 3 | Unit tests: AC-010 (>20 SNIs cap), AC-016 (tiebreaker determinism) |
| `version_counts` keys in summarize output are decimal strings (e.g., "771"), NOT hex strings | BC-2.07.031 invariant 3 | Unit test: AC-010 assert key is "771" not "0x0303" |
| `truncated_records` is a separate field from `parse_errors`; both appear in summarize detail | BC-2.07.031 postcondition 8-9 | Unit test: AC-011 |
| `on_flow_close` performs ONLY `self.flows.remove(flow_key)` — no analysis at close time | BC-2.07.035 invariant 1 | Code review: confirm no analysis calls in `on_flow_close` body |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `parse_tls_plaintext` nom parser; TLS record type constants |
| serde_json | 1 | Serialization of BTreeMap detail in AnalysisSummary |
| Rust std | 2024 edition (stable) | `BTreeMap`, `Vec::drain`, `usize::saturating_sub`, `Vec::extend_from_slice` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | Oversized-record guard (643-653), buffer cap (726-748), nom error handling (700-712), non-handshake skip (678-682), `summarize` (763-808), `on_flow_close` (752-754) |
| tests/tls_analyzer_tests.rs | modify | `test_oversized_sni_exceeds_record_payload_limit`, `test_oversized_after_valid_hello_increments_both`, `test_record_payload_boundary_18432_vs_18433`, `test_buffer_cap_appends_at_most_max_buf`, `test_buffer_cap_appends_at_most_max_buf_literal_residue`, `test_buffer_full_append_noop`, `test_buffer_full_append_noop_literal`, `test_buffer_overflow_silent_no_counters`, `test_parse_error_counter`, `test_malformed_handshake_increments_parse_errors_only`, `test_summarize_output`, `test_summarize_has_all_required_fields`, `test_summarize_top_snis_capped_at_20`, `test_summarize_top_snis_ties_broken_alphabetically`, `test_fresh_summarize_truncated_records_zero`, `test_appdata_record_skipped_then_hello`, `test_stop_after_handshake`, `test_within_loop_nonhandshake_skip_before_done`, `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently`, `test_on_flow_close_drops_state_preserves_aggregates`, `test_on_flow_close_absent_key_no_panic` |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| v1.4 | 2026-06-01 | story-writer | FIX-P5-003 — add AC-016 (top_snis tiebreaker: SNI name ASC, deterministic); expand AC-009 and AC-010 descriptions with SNI-name-ascending tiebreaker and determinism guarantee; add EC-012; update Architecture Compliance Rules and FSR for top_snis determinism (BC-2.07.031 v1.3 postcondition 3 / invariant 2 / EC-004) |
| v1.3 | 2026-05-29 | story-writer | Qualify AC-002 "preceding partial record" sub-clause as defensive/by-inspection per BC-2.07.004 v1.3 (F-S058-P6-002): that scenario is not reachable via the public `on_data` API (parser reads from `buf[0]` and returns at the incompleteness check before a later oversized record could be encountered). Normative assertion retained: buffer clearing is unconditional. |
| v1.2 | 2026-05-29 | story-writer | Re-point AC-013 to within-loop-skip test (F-S058-P1-002): `test_stop_after_handshake` removed (proves done()-short-circuit, not within-loop skip); canonical citation is now `test_within_loop_nonhandshake_skip_before_done` + `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently`. Add literal-cap residue test citations to AC-004 (`test_buffer_cap_appends_at_most_max_buf_literal_residue`) and AC-005 (`test_buffer_full_append_noop_literal`). FSR enumerates all 20 test fn names. Normative AC scenarios, BC list, and tls.rs line anchors unchanged. |
| v1.1 | 2026-05-29 | story-writer | AC-citation sync — AC-002/003/004/005/006/008/010/011/012/014/015 cite concrete test fn names (DF-AC-TEST-NAME-SYNC-001, proactive PG-W17-001 fix). FSR row for tests/tls_analyzer_tests.rs updated to enumerate all 16 concrete test names. Normative AC text, BC list, narrative, edge cases, and scenarios unchanged. |
| v1.0 | 2026-05-21 | story-writer | Initial decomposition. |
