---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-29T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: fix-tls-clienthello-frag-F2-scope-addition
modified:
  - "v1.1: fix-tls-clienthello-frag adversary burst — C-1 Inv-4+Anchors post-block increment placement + borrow rationale; C-2 EC-002 full-drop test seam anchor added; C-3 PC-3 to_copy equivalence qualifier removed; I-2 PC-4 strengthened to value-equality; I-3 VP table expanded to 5 canonical names mapped to VP-040 Sub-A..Sub-E; I-4 tls.rs:887-889→887-890; OBS-1 VP-039-hedge→VP-040 definitive — 2026-06-29"
  - "v1.2: fix-tls-clienthello-frag architect VP-040 6-test reconciliation — added Sub-A full-drop row (test_BC_2_07_043_buffer_saturation_full_drop; uses fill_buf_for_testing seam, remaining==0 case) to VP table and VP Anchors; Architecture Anchors EC-002 test reference corrected from _observable to _buffer_saturation_full_drop — 2026-06-29"
  - "v1.3: fix-tls-clienthello-frag adversary F-F2-SEAM-SIG-001 — corrected fill_buf_for_testing seam signature from flow_key: FlowKey (by value) to flow_key: &FlowKey (by reference), matching VP-040 call sites and the &FlowKey convention of all five sibling TLS test seams (client_buf_len_for_testing etc., tls.rs:957+) — 2026-06-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.043: Per-Direction Buffer Saturation Tail-Drop Is Observable via buffer_saturation_drops Counter

## Description

When `TlsAnalyzer::on_data` discards incoming bytes because the per-direction stream
buffer (`client_buf` or `server_buf`) is at or near `MAX_BUF = 65,536` bytes (the
tail-drop branch specified in BC-2.07.005), the analyzer increments a `TlsAnalyzer`-level
aggregate counter `buffer_saturation_drops` by 1. The counter records the NUMBER OF
TAIL-DROP EVENTS (one per `on_data` call that discards any bytes), not the number of
bytes discarded. This makes the previously-silent `MAX_BUF` tail-drop observable in
`summarize()` output, providing defense-in-depth telemetry regardless of whether the
saturation path is reachable by any known current attack vector.

The byte-drop semantics of BC-2.07.005 are UNCHANGED: bytes that exceed the cap are
still discarded. Only TELEMETRY is added. No finding is emitted; no `parse_errors`
increment occurs; the existing silent truncation behavior is preserved except that the
event is now counted.

## Related BCs

- BC-2.07.005 — composes with (BC-2.07.005 defines the tail-drop; this BC adds observability
  to the same code path; BC-2.07.005 Inv-3 cross-references this BC)
- BC-2.07.039 — parallel pattern (handshake_reassembly_overflows is the same counter
  design applied to the carry-buffer overflow; buffer_saturation_drops mirrors that pattern
  for the stream buffer)
- BC-2.07.031 — depends on (summarize() emits AnalysisSummary with TLS stats detail map;
  buffer_saturation_drops key must appear in that detail map)

## Preconditions

1. `TlsAnalyzer::on_data` is called for a flow direction with `data.len() > 0`.
2. The per-direction buffer (`client_buf` or `server_buf`) has `remaining =
   MAX_BUF.saturating_sub(buf.len())` bytes of capacity remaining.
3. A tail-drop is about to occur: `data.len() > remaining`. This covers both the
   partial-copy case (`remaining > 0` but `remaining < data.len()`) and the full-drop
   case (`remaining == 0`, where the `if remaining > 0` guard causes the entire `data`
   slice to be skipped). NOTE: do NOT use the equivalent formulation `to_copy <
   data.len()` — `to_copy` is only computed inside the `if remaining > 0` arm and is
   never set in the full-drop case (`remaining == 0`), so that form would miss the
   full-drop path.

## Postconditions

1. `TlsAnalyzer.buffer_saturation_drops` is incremented by exactly 1.
2. The counter increment fires ONCE per `on_data` call in which any tail-drop occurs,
   regardless of how many bytes are dropped (1 byte or 65,536 bytes dropped in a single
   call both count as 1 event).
3. The counter applies to BOTH directions (`Direction::ClientToServer` and
   `Direction::ServerToClient`); both client_buf and server_buf saturation events
   increment the same aggregate counter.
4. `buffer_saturation_drops` appears as key `"buffer_saturation_drops"` in the `detail`
   `HashMap<String, Value>` returned by `TlsAnalyzer::summarize()`, mirroring the
   `"truncated_records"` key pattern at `src/analyzer/tls.rs:887-890`. The value MUST
   equal the current counter value exactly: `detail["buffer_saturation_drops"] ==
   self.buffer_saturation_drops`. A mere key-presence check without verifying the numeric
   value would pass a broken implementation that always inserts `0`.
5. `buffer_saturation_drops` is NOT reset when a flow closes (`on_flow_close` drops
   `TlsFlowState` only; the counter lives on `TlsAnalyzer`, not on `TlsFlowState`).
6. `parse_errors` is NOT incremented. `truncated_records` is NOT incremented. No
   `Finding` is pushed to `all_findings`. The byte-drop behavior of BC-2.07.005 is
   unchanged; only the counter changes.

## Invariants

1. `buffer_saturation_drops` is a `u64` field on `TlsAnalyzer` (NOT on `TlsFlowState`),
   mirroring `truncated_records: u64` at `src/analyzer/tls.rs:319` and
   `handshake_reassembly_overflows: u64` at the carry-layer level.
2. `buffer_saturation_drops` is initialized to `0` in `TlsAnalyzer::new()` and never
   reset between flows or between `on_data` calls.
3. The counter is monotonically non-decreasing across the lifetime of a `TlsAnalyzer`
   instance.
4. The counter increments ONLY at the tail-drop branch of the `on_data` buffer-append
   logic — no other code path in `TlsAnalyzer` increments it. PLACEMENT CONSTRAINT:
   `self.buffer_saturation_drops += 1` MUST be placed AFTER the `&mut state` (i.e.,
   `&mut TlsFlowState`) buffer-append block closes, not inside it. The reason: the
   per-direction match arm borrows `self.flows` mutably to obtain `state: &mut
   TlsFlowState`; while that borrow is live, `self` cannot be mutated (Rust borrow
   rules). The correct implementation pattern is: (a) inside the per-direction match
   arm, detect the drop condition `data.len() > remaining` and set a local `bool`
   flag (e.g., `let did_drop = data.len() > remaining;`); (b) after the `&mut state`
   block closes (the mutable borrow is released), check the flag and call
   `self.buffer_saturation_drops += 1`. This places the increment between the
   buffer-append block and the `try_parse_records` call.
5. When `data.len() == 0`, no tail-drop can occur (0-byte slice trivially fits any
   remaining capacity), so the counter is not incremented on a zero-length `on_data` call.

## F-EV-001 Defense-in-Depth Note

The F-EV-001 validation (`F-EV-001-clientbuf-saturation-validation.md`) determined the
silent tail-drop is NOT-EXPLOITABLE for ClientHello blinding under current code, because:

- Durable undrained residue in `client_buf` is capped at ~18,437 bytes by the
  drain-complete-records-first loop (one incomplete valid record), leaving ~47,099 bytes
  of headroom — far short of `MAX_BUF`.
- The only mechanism to force durable residue toward `MAX_BUF` trips the TELEMETERED
  oversize guard (BC-2.07.004: `parse_errors`++ and `truncated_records`++), contradicting
  the "zero-telemetry blinding" premise.
- Single `on_data` slices are bounded by one IP datagram TCP payload (< `MAX_BUF`).

However, F-EV-001 identified two preconditions that would make the path
CONDITIONALLY-EXPLOITABLE if either were introduced:

- **P1:** If the reassembler began coalescing adjacent contiguous segments into a single
  `on_data` slice > 64 KB (currently `segment.rs:398` emits one chunk per BTreeMap entry,
  no coalescing).
- **P2:** If an IPv6 jumbogram path delivered a single TCP payload > 65,535 bytes.

`buffer_saturation_drops` makes the primitive non-silent regardless of future reachability,
pre-empting both P1 and P2 defensively. Any future refactor introducing coalescing or
jumbogram support will NOT introduce a new silent drop — it will be immediately visible in
`summarize()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Buffer at 65,535; `data` is 2 bytes → `to_copy=1`, `remaining=1`, tail-drop of 1 byte | `buffer_saturation_drops` incremented by 1; 1 byte appended, 1 byte dropped |
| EC-002 | Buffer at 65,536 (full, `remaining==0`); `data` is 1,000 bytes → `if remaining > 0` guard skips the append entirely | `buffer_saturation_drops` incremented by 1; 0 bytes appended (full-drop event). TESTABILITY NOTE (per F-EV-001): the full-drop state (buffer parked at exactly MAX_BUF) cannot be reached via a single `on_data` call on a fresh flow through the public API — the drain-complete-records-first loop processes bytes as they arrive, preventing durable MAX_BUF residue. This case MUST be exercised through the test seam `fill_buf_for_testing` (see Architecture Anchors). |
| EC-003 | Buffer at 65,536 (full); `data` is 0 bytes | `remaining == 0` but `data.len() == 0`; precondition P3 (`data.len() > remaining`) is NOT met because 0 > 0 is false; `buffer_saturation_drops` NOT incremented |
| EC-004 | Buffer at 0; `data` is 65,537 bytes → `to_copy=65,536`, 1 byte dropped | `buffer_saturation_drops` incremented by 1; 65,536 bytes appended |
| EC-005 | Buffer at 0; `data` is exactly 65,536 bytes → `to_copy=65,536 = data.len()`, no tail-drop | `buffer_saturation_drops` NOT incremented (no drop event) |
| EC-006 | Two consecutive `on_data` calls each trigger a tail-drop (e.g., buffer near full, two large slices arrive) | `buffer_saturation_drops` incremented by 1 per call; after two calls, counter == 2 |
| EC-007 | Both `client_buf` saturation (ClientToServer) and `server_buf` saturation (ServerToClient) occur in the same session | Both events increment the SAME aggregate counter; final value equals total drop-event count across both directions |
| EC-008 | `summarize()` called when `buffer_saturation_drops == 0` (no drops have occurred) | `"buffer_saturation_drops": 0` appears in the `detail` map; zero value is still present (matches pattern of `truncated_records` and `parse_errors` which always appear) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Buffer at 65,535; `data` 2 bytes (ClientToServer) | `buffer_saturation_drops == 1`; `buf.len() == 65,536`; `parse_errors == 0`; `truncated_records == 0` | happy-path |
| Buffer at 65,536 (full); `data` 1,000 bytes (ClientToServer) | `buffer_saturation_drops == 1`; `buf.len()` unchanged at 65,536; `parse_errors == 0` | edge-case |
| Buffer at 0; `data` exactly 65,536 bytes | `buffer_saturation_drops == 0` (no drop); `buf.len() == 65,536` | no-drop boundary |
| `summarize()` after 3 tail-drop events | `detail["buffer_saturation_drops"] == 3`; key present | observability |
| `summarize()` after zero tail-drop events | `detail["buffer_saturation_drops"] == 0`; key present | observability-zero |
| ServerToClient buffer at 65,535; `data` 2 bytes, then ClientToServer buffer at 65,535; `data` 2 bytes | `buffer_saturation_drops == 2`; both directions use same aggregate counter | cross-direction |

## Verification Properties

| VP-NNN | Sub | Property | Proof Method |
|--------|-----|----------|-------------|
| VP-040 | Sub-A | `buffer_saturation_drops` incremented exactly once per tail-drop event (partial-drop: remaining > 0 but remaining < data.len(); single ≥65,537-byte on_data slice, no seam) | unit: `test_BC_2_07_043_buffer_saturation_observable` (Red-Gate test; architect authors in VP-040) |
| VP-040 | Sub-A | `buffer_saturation_drops` incremented exactly once per tail-drop event (full-drop: remaining == 0; uses `fill_buf_for_testing` seam to park buffer at MAX_BUF before call) | unit: `test_BC_2_07_043_buffer_saturation_full_drop` |
| VP-040 | Sub-B | Counter NOT incremented when no tail-drop occurs (exact-fit case: data.len() == remaining, no bytes discarded) | unit: `test_BC_2_07_043_no_drop_no_counter` |
| VP-040 | Sub-C | Counter NOT reset at flow close (counter lives on TlsAnalyzer, not TlsFlowState) | unit: `test_BC_2_07_043_counter_persists_across_flows` |
| VP-040 | Sub-D | `summarize()` detail map value EQUALS the actual drop count, not merely contains the key (value-equality; exercises N drops then asserts detail["buffer_saturation_drops"] == N) | unit: `test_BC_2_07_043_summarize_value_equals_drop_count` |
| VP-040 | Sub-E | Both directions (ClientToServer and ServerToClient) increment the same aggregate counter | unit: `test_BC_2_07_043_both_directions_increment_same_counter` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS Traffic Analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS Traffic Analysis") per domain/capabilities/cap-07-tls-analysis.md — this BC adds observability telemetry to the per-direction stream buffer cap that is part of the TLS analysis bounded-resource design, directly within the TLS Traffic Analysis capability scope |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation; counter output is a numeric stat surfaced in detail map, not raw bytes) |
| Architecture Module | SS-07 (analyzer/tls.rs — `on_data` buffer-append logic :820-835; `summarize()` detail map insertion :887-890; `TlsAnalyzer` struct field) |
| Stories | STORY-146 |
| Origin | F-EV-001 defense-in-depth recommendation per `.factory/research/F-EV-001-clientbuf-saturation-validation.md` §7 |

## Architecture Anchors

- `src/analyzer/tls.rs:820-835` — `on_data` buffer-append logic; the drop-condition
  `data.len() > remaining` is DETECTED inside this block (set a local bool flag), but the
  `self.buffer_saturation_drops += 1` increment MUST be placed AFTER this block closes
  (after the `&mut state` borrow is released, before `try_parse_records` is called) — see
  Inv-4 borrow-constraint rationale. This block covers both `Direction::ClientToServer` and
  `Direction::ServerToClient` arms.
- `src/analyzer/tls.rs:319` — `truncated_records: u64` (type precedent for new `buffer_saturation_drops: u64` field)
- `src/analyzer/tls.rs:887-890` — `truncated_records` summarize() insertion (pattern to mirror
  for `buffer_saturation_drops`; insert statement spans lines 887-890)
- `src/analyzer/tls.rs` (TlsAnalyzer struct) — new field `buffer_saturation_drops: u64`
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_043_buffer_saturation_observable` (Red-Gate test,
  partial-drop path; architect authors in VP-040)
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_043_buffer_saturation_full_drop` (full-drop path;
  architect authors in VP-040 using fill_buf_for_testing seam to pre-park buffer at MAX_BUF)
- TEST SEAM (C-2): `#[doc(hidden)] pub fn fill_buf_for_testing(&mut self, flow_key: &FlowKey,
  direction: Direction, n: usize)` — fills the per-direction buffer to `n` bytes
  (up to MAX_BUF) by directly setting `client_buf`/`server_buf` length; required to
  exercise EC-002 (full-drop: buffer parked at MAX_BUF before `on_data` call) since
  that state is not reachable via the public `on_data` API alone. The architect MUST
  author the EC-002 test (`test_BC_2_07_043_buffer_saturation_full_drop`) using this seam.

## Story Anchor

STORY-146 (TLS Buffer Saturation Telemetry — `buffer_saturation_drops` Counter; wave 66, dep=STORY-144)

## VP Anchors

- VP-040 Sub-A — `test_BC_2_07_043_buffer_saturation_observable` (partial-drop: ≥65,537-byte slice, no seam)
- VP-040 Sub-A — `test_BC_2_07_043_buffer_saturation_full_drop` (full-drop: remaining==0, uses fill_buf_for_testing seam)
- VP-040 Sub-B — `test_BC_2_07_043_no_drop_no_counter` (no-drop boundary)
- VP-040 Sub-C — `test_BC_2_07_043_counter_persists_across_flows` (counter persistence)
- VP-040 Sub-D — `test_BC_2_07_043_summarize_value_equals_drop_count` (value-equality)
- VP-040 Sub-E — `test_BC_2_07_043_both_directions_increment_same_counter` (cross-direction)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `TlsAnalyzer.buffer_saturation_drops`; mutates `client_buf` or `server_buf` via existing BC-2.07.005 append logic |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (`&mut self`) |
| **Overall classification** | mixed |
