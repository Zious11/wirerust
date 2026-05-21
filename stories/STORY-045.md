---
document_type: story
story_id: "STORY-045"
epic_id: "E-4"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.019.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.021.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.022.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.024.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.025.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-041, STORY-044]
blocks: [STORY-046]
behavioral_contracts:
  - BC-2.06.019
  - BC-2.06.021
  - BC-2.06.022
  - BC-2.06.024
  - BC-2.06.025
verification_properties: []
priority: "P0"
cycle: v0.1.0-brownfield
wave: null
target_module: src/analyzer/http.rs
subsystems: [SS-06]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-045`

# STORY-045: Flow Lifecycle, Cross-Flow Isolation, and Buffer/Map Caps

## Narrative
- **As a** forensic analyst
- **I want to** have `on_flow_close` correctly clean up per-flow HTTP state, confirm that errors and poisoning in one flow cannot affect other flows, and verify that per-direction header buffers and per-map cardinality are bounded by their respective caps
- **So that** wirerust remains memory-safe and forensically correct when analyzing high-volume captures with many concurrent flows

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.019 | on_flow_close Removes Per-Flow State; Reopening Same Key Starts Fresh |
| BC-2.06.021 | Cross-Flow Isolation: Errors and Poisoning Do Not Leak |
| BC-2.06.022 | Per-Direction Header Buffer Capped at MAX_HEADER_BUF (65536) |
| BC-2.06.024 | Per-Map Cardinality Cap: New Keys Dropped Past MAX_MAP_ENTRIES |
| BC-2.06.025 | uris List Capped at MAX_URIS=10000 |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.019 postcondition 1-4)
`on_flow_close` calls `self.flows.remove(flow_key)`, dropping the entire `HttpFlowState` including buffers, error counts, and poison flags. No other HttpAnalyzer aggregate state (transactions, parse_errors, non_http_flows, etc.) is modified.
- **Test:** `test_flow_close_cleans_up_state`

### AC-002 (traces to BC-2.06.019 postcondition 3-4)
After `on_flow_close`, a subsequent `on_data` for the same FlowKey creates a brand-new `HttpFlowState::new()` with request_poisoned=false, response_poisoned=false, error_count=0, counted_as_non_http=false, and empty buffers.
- **Test:** `test_poison_cleared_after_flow_close`

### AC-003 (traces to BC-2.06.019 invariant 2)
`on_flow_close` ignores the CloseReason parameter (it is `_reason`). A flow close with any reason produces the same result.
- **Test:** `test_flow_close_cleans_up_state`

### AC-004 (traces to BC-2.06.021 postcondition 1-3)
When two distinct FlowKeys are active and flow A has parse errors or has been poisoned, flow B's `HttpFlowState` (error counts, poison flags) is completely unaffected. Aggregate counters (`parse_errors`, `non_http_flows`) are global sums but do not gate per-flow behavior.
- **Test:** `test_cross_flow_isolation_parse_errors`

### AC-005 (traces to BC-2.06.021 invariant 1-2)
`flows: HashMap<FlowKey, HttpFlowState>` provides per-key isolation by construction. Only `on_flow_close` removes entries; entries do not affect each other.
- **Test:** `test_cross_flow_isolation_poisoning`

### AC-006 (traces to BC-2.06.022 postcondition 1-4)
For each `on_data` call, only `min(data.len(), MAX_HEADER_BUF - buf.len())` bytes are appended to the direction buffer. The direction buffer size never exceeds `MAX_HEADER_BUF = 65,536` bytes. Bytes past the cap are silently dropped without error or counter increment.
- **Test:** `test_buffer_cap_no_panic_on_oversized_headers`

### AC-007 (traces to BC-2.06.022 invariant 1-3)
`MAX_HEADER_BUF = 65,536` is defined as a constant. The cap applies per-direction independently (request_buf and response_buf have separate caps). No finding is emitted when the cap is reached.
- **Test:** `test_buffer_cap_no_panic_on_oversized_headers`

### AC-008 (traces to BC-2.06.024 postcondition 1-4)
When a map (`methods`, `hosts`, or `user_agents`) has reached `MAX_MAP_ENTRIES = 50,000` distinct keys, any new unique key is silently NOT inserted. Existing keys continue to increment normally past the cap.
- **Test:** `test_map_cardinality_cap_drops_new_keys`

### AC-009 (traces to BC-2.06.024 invariant 2-3)
Guard pattern: `if self.methods.len() < MAX_MAP_ENTRIES || self.methods.contains_key(&parsed.method)`. The `contains_key` short-circuit allows EXISTING keys to increment even when the map is at cap. `status_codes` uses u16 keys and has no explicit MAX_MAP_ENTRIES guard.
- **Test:** `test_existing_keys_increment_at_cap`

### AC-010 (traces to BC-2.06.025 postcondition 1-3)
When `self.uris.len() == MAX_URIS (10,000)`, new request URIs are NOT appended to `self.uris`. Other counters (methods, hosts, etc.) are still updated for the request. No error or counter increment occurs for the dropped URI.
- **Test:** `test_uris_capped_at_max_uris`

### AC-011 (traces to BC-2.06.025 invariant 1-3)
`MAX_URIS = 10,000`. Guard: `if self.uris.len() < MAX_URIS { self.uris.push(...) }`. The same URI can appear multiple times (no deduplication). URIs dropped at cap are permanently lost.
- **Test:** `test_uris_capped_at_max_uris`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| on_flow_close | src/analyzer/http.rs:540-542 | effectful-shell (removes HashMap entry) |
| flows HashMap | src/analyzer/http.rs:114-126 | effectful-shell (per-flow isolation structure) |
| buffer cap logic | src/analyzer/http.rs:513-529 | effectful-shell (min clamp on on_data) |
| map entry guards | src/analyzer/http.rs:375-389 | effectful-shell (conditional insert) |
| uris push guard | src/analyzer/http.rs:391-393 | effectful-shell (conditional push) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Flow close on poisoned flow; same key reopened | New flow starts with poison=false |
| EC-002 | on_flow_close for a FlowKey not in self.flows | remove() is a no-op; no panic |
| EC-003 | Flow close with partial request in buffer | Buffer discarded with the state |
| EC-004 | Flow A poisoned; Flow B valid request | Flow B result identical to standalone execution |
| EC-005 | Buffer at exactly 65536 | Next on_data appends 0 bytes |
| EC-006 | Buffer at 65535; on_data sends 100 bytes | 1 byte appended; 99 dropped silently |
| EC-007 | Map at 50000 keys; new unique method | Not inserted; no panic |
| EC-008 | Map at 50000 keys; existing method | Count incremented normally |
| EC-009 | uris at 9999; new request | URI appended (len=10000) |
| EC-010 | uris at 10000; new request | URI NOT appended; len stays 10000 |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (on_flow_close, buffer cap, map guards) | effectful-shell | All mutate HttpAnalyzer state |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| Referenced code (http.rs:114-126, 375-393, 513-542) | ~4,000 |
| Test files (http_analyzer_tests.rs) | ~3,000 |
| BC files (5 BCs) | ~5,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~17,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-011 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement `on_flow_close` per BC-2.06.019 (flows.remove; no aggregate counter mutation; CloseReason ignored)
4. [ ] Verify cross-flow isolation per BC-2.06.021 (HashMap structure is sufficient; add isolation tests)
5. [ ] Implement buffer cap per BC-2.06.022 (min clamp in on_data; MAX_HEADER_BUF=65536; per-direction independent caps)
6. [ ] Implement map cardinality cap per BC-2.06.024 (len<MAX_MAP_ENTRIES||contains_key guard; MAX_MAP_ENTRIES=50000)
7. [ ] Implement uris Vec cap per BC-2.06.025 (len<MAX_URIS guard; MAX_URIS=10000)
8. [ ] Run all tests; verify all pass
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-041 | flows is a `HashMap<FlowKey, HttpFlowState>`; each flow entry is fully independent | `entry().or_insert_with(HttpFlowState::new)` creates fresh state on first access after close | Buffer cap uses `saturating_sub` to avoid underflow; always clamp with `min` |
| STORY-044 | Aggregate counters (parse_errors, non_http_flows) are global and not per-flow | on_flow_close must NOT reset aggregate counters | `counted_as_non_http` is per-flow state in HttpFlowState, not in HttpAnalyzer |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `MAX_HEADER_BUF = 65,536` matches TLS's `MAX_BUF` | BC-2.06.022 invariant 1 | Code review: confirm constant value |
| `MAX_MAP_ENTRIES = 50,000` | BC-2.06.024 invariant 1 | Code review: confirm constant at http.rs:24 |
| `MAX_URIS = 10,000` | BC-2.06.025 invariant 1 | Code review: confirm constant at http.rs:23 |
| on_flow_close does NOT modify aggregate counters (transactions, parse_errors, non_http_flows) | BC-2.06.019 invariant 1 | Unit test: verify aggregate counters unchanged after close |
| Existing map keys always increment even at cap (contains_key short-circuit) | BC-2.06.024 invariant 3 | Unit test: AC-009 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | HashMap::remove, Vec::len, saturating_sub, min |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | on_flow_close (540-542); buffer cap (513-529); map guards (375-389); uris guard (391-393) |
| tests/http_analyzer_tests.rs | modify | Add: test_flow_close_cleans_up_state, test_poison_cleared_after_flow_close, test_cross_flow_isolation_parse_errors, test_cross_flow_isolation_poisoning, test_buffer_cap_no_panic_on_oversized_headers, test_map_cardinality_cap_drops_new_keys, test_uris_capped_at_max_uris |
