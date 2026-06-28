# ADR 0007: DNP3 Stream Dispatch and Parser Design

**Status:** Accepted
**Date:** 2026-06-12
**Modified:** 2026-06-27 (RULING-DNP3-SIBLING-001): Decision 3 carry split per-direction
(carry_c2s / carry_s2c), Direction threading into on_data, saturating_sub window expiry
(all wrapping_sub sites), strict-'>' operator pin for 300s correlation-window expiry (>= → >),
direction-based source-IP resolution replacing port-20000 heuristic. Decision 2
Dnp3FlowState carry buffer updated: carry: Vec<u8> → carry_c2s: Vec<u8> + carry_s2c: Vec<u8>.
**Context:** v0.6.0 (issue #8, STORY-106..110, PRs #219–#231). Adding DNP3 TCP analysis raised
several design decisions about protocol dispatch, frame parsing, carry-buffer design, and
bounded-resource constraints. This ADR records those decisions so future contributors understand
the rationale and constraints without reverse-engineering them from the code.

## Problem

DNP3 (IEEE Std 1815-2012) is a serial-heritage ICS protocol encapsulated in TCP on port 20000.
Unlike HTTP or TLS, it has no universally distinctive first-byte sequence suitable for
content-based classification, and it is a binary framing protocol with a 10-byte data-link header
(8 header bytes + 2 CRC bytes) followed by data blocks also carrying CRCs. Several constraints
are non-obvious:

1. **Dispatch ambiguity:** `0x05 0x64` sync bytes can appear in other binary data. Content-based
   dispatch is unreliable; port-based dispatch is used (Rule 6 in the stream dispatcher, following
   ADR 0005's pattern for Modbus).

2. **Fragmented delivery:** TCP segments rarely align with DNP3 frame boundaries. The parser must
   maintain a per-flow carry buffer and walk frames across segment boundaries.

3. **CRC presence:** IEEE Std 1815-2012 specifies 2-byte CRCs appended to the 8-byte header and
   to each 16-byte data block. wirerust's DNP3 analyzer reads CRC bytes as part of the on-wire
   frame length accounting but does not verify or strip them (see Decision 3).

4. **Little-endian addresses:** DEST and SOURCE link addresses in the DNP3 data-link header are
   decoded little-endian (`u16::from_le_bytes`), per IEEE Std 1815-2012 §8.2. This is non-obvious
   for engineers familiar with big-endian network protocols.

5. **Kani harness requirements:** Formal verification (VP-023) requires that pure-core parse
   functions are free `fn`s (not `impl` methods), that the FC classifier has a wildcard arm
   (`_ => Unknown`), and that `parse_dnp3_dl_header` does NOT validate sync or LENGTH so the
   harness can range over all 2^80 inputs.

## Decisions

### Decision 1: Port-20000 dispatch as Rule 6

DNP3 TCP flows are classified using a port-20000 rule appended after Rules 1–5 in the stream
dispatcher. This follows the pattern established for Modbus in ADR 0005 (Rule 5, port 502).
The rule fires after all content-signature rules (TLS record header, HTTP method prefix) and
all prior port rules, so TLS or HTTP traffic on port 20000 is correctly classified by content
before reaching the DNP3 port rule. The VP-004 port-precedence invariant is preserved.

Cross-reference: dispatcher Rules 1–7 are documented in `src/dispatcher.rs` module comment.

### Decision 2: 10-byte header parse + 292-byte carry buffer

The DNP3 data-link header is parsed from 8 bytes (bytes 0–7 of the frame; bytes 8–9 are CRC).
The maximum on-wire frame size is 292 bytes: a 10-byte header section (8 header + 2 CRC) plus
up to 16 × 18-byte data blocks (16 data bytes + 2 CRC each) = 10 + 16 × 18 = 298 — but the
LENGTH field maximum of 255 constrains the useful data to at most 282 bytes, and with CRC
accounting the maximum total frame is:
  `frame_len = 10 + ceil((LENGTH - 5) / 16) × 18`
When LENGTH = 255: `ceil(250 / 16) × 18 = 16 × 18 = 288`; plus 4 (header: 8 bytes − 4 control
bytes that are already counted in LENGTH) = no, using the formula `compute_dnp3_frame_len`:
  `10 + ((length as usize - 5 + 15) / 16) * 18` which for length=255 gives 292.
This is proven by VP-023 Sub-D (Kani harness). The per-flow carry buffer is sized at 292 bytes
(`MAX_DNP3_FRAME_LEN`) to hold at most one partial frame. RULING-DNP3-SIBLING-001 §1.3: the
single `carry: Vec<u8>` is replaced with two directional carry buffers — `carry_c2s: Vec<u8>`
(master-to-outstation) and `carry_s2c: Vec<u8>` (outstation-to-master) — to prevent
cross-direction carry-buffer splice (DRIFT-DNP3-DIRECTION-001). Each directional carry is
independently bounded at 292 bytes.

DEST and SOURCE fields are decoded little-endian: `u16::from_le_bytes([data[4], data[5]])` and
`u16::from_le_bytes([data[6], data[7]])`, per IEEE Std 1815-2012 §8.2.

### Decision 3: CRC bytes are counted but not verified

The on-wire frame-length arithmetic accounts for CRC bytes when computing `frame_len` (so the
carry-buffer frame-walk advances past the correct number of bytes), but CRC contents are not
checked. Rationale: CRC verification would add complexity and a dependency on a CRC-16/DNP
implementation, while providing no benefit for the threat detections wirerust currently emits
(which depend on function codes and source/destination addresses, not data integrity). A malformed
CRC is already detectable as a parse-invalid frame (D11-equivalent for DNP3) if the frame fails
the 3-point validity gate (`is_valid_dnp3_frame_header`).

**`on_data` signature (RULING-DNP3-SIBLING-001 §1.5):**

```rust
// Before:
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32)

// After:
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)
```

`Direction` is `crate::reassembly::handler::Direction`. All `Dnp3Analyzer::on_data` call sites
in `src/dispatcher.rs` pass `direction` from the `StreamHandler` context, matching the STORY-139
pattern for ENIP.

**Directional carry select (RULING-DNP3-SIBLING-001 §1.2):**

On every `on_data` call the active carry buffer is selected by direction before the frame-walk
loop runs:

```
let active_carry = match direction {
    ClientToServer => &mut flow.carry_c2s,
    ServerToClient => &mut flow.carry_s2c,
};
```

`active_carry` is used for ALL carry operations in that call: accumulate incoming bytes,
cap-check (292-byte bound), inline-resync, frame-walk prepend, drain-after-consume, and
residual-stash-back. The two directional carry buffers are NEVER mixed on a single call.

**Window-expiry (RULING-DNP3-SIBLING-001 §2.2 — all wrapping_sub → saturating_sub):**

All windowed timestamp comparisons in `on_data` and helper methods use `saturating_sub`
instead of `wrapping_sub`. Under `saturating_sub`, a backwards-clock packet
(`now_ts < window_start`) yields elapsed=0, which does NOT exceed any threshold — so the
window is NOT reset and burst accumulation is preserved. This eliminates the
DRIFT-DNP3-CLOCK-001 evasion path where an adversarially injected stale-timestamp packet
could abort detection.

The 300s correlation-window expiry additionally uses strict `>` instead of `>=`
(RULING-DNP3-SIBLING-001 §2.3, DRIFT-DNP3-OP-001), consistent with all other window expiry
checks in `dnp3.rs` and with the ENIP fix in STORY-139.

**Source-IP resolution (RULING-DNP3-SIBLING-001 §1.4):**

`resolve_master_ip` is updated from the port-20000 heuristic to direction-based resolution,
mirroring the Modbus pattern (`src/analyzer/modbus.rs` ~355-382):

```rust
// Direction::ClientToServer = master (initiates connections to port 20000)
// Direction::ServerToClient = outstation (listens on port 20000)
let master_ip = match direction {
    Direction::ClientToServer => flow_key.src_ip_of(direction),
    Direction::ServerToClient => flow_key.dst_ip_of(direction),
};
```

**DNP3 292-byte carry-cap reachability (RULING-DNP3-SIBLING-001 §4):**

Unlike the ENIP 600-byte cap (assessed unreachable via RULING-137-002), the DNP3 292-byte
cap IS reachable. The overflow arm (`carry.len() + data.len() > MAX_DNP3_FRAME_LEN`) can be
triggered by repeated sub-292-byte partial-frame deliveries until carry is full, then one
additional delivery. After the carry split, each directional carry is independently capped:
`carry_c2s.len() <= 292` AND `carry_s2c.len() <= 292`. The overflow arm must check each
carry independently. This cap is LIVE SPEC — do NOT mark as unreachable (contrast with
RULING-137-002 for ENIP).

### Decision 4: Bounded-resource constants

All per-flow state is bounded to prevent memory exhaustion on adversarial captures:

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_DNP3_FRAME_LEN` | 292 bytes | Carry buffer size; proven by VP-023 Sub-D |
| `MAX_MASTER_ADDRS` | 64 | Tracked master-station source addresses per flow |
| `MAX_PENDING_REQUESTS` | 256 | Pending control requests for T1691.001 correlation |
| `MAX_FINDINGS` | 10 000 | Hard cap on findings per analyzer |
| `CORRELATION_WINDOW_SECS` | 300 s | Shared reset window for six windowed counters |
| `BLOCK_CMD_TIMEOUT_SECS` | 10 s | Per-request timeout for T1691.001 block-command inference |
| `BLOCK_CMD_THRESHOLD` | 3 | Block events within window to fire T1691.001 |
| `T0827_THRESHOLD` | 3 | Combined restart + block events to fire T0827 |
| `DETECTION_WINDOW_SECS` | 60 s | Direct-operate burst detection window |

The six windowed correlation counters reset together when the elapsed time since
`correlation_window_start_ts` reaches `CORRELATION_WINDOW_SECS`. This shared reset is simpler
to implement and reason about than per-counter independent windows; the tradeoff is that a burst
of events near the end of one window resets all counters simultaneously, potentially missing a
sustained pattern across the boundary. This is acceptable for a first-pass ICS threat detector.

### Decision 5: Architecture compliance rules for pure-core functions

To satisfy VP-023 Kani formal verification:

- `parse_dnp3_dl_header`, `is_valid_dnp3_frame_header`, `classify_dnp3_fc`,
  `compute_dnp3_frame_len`, `transport_is_fir`, `has_user_data` are free `fn`s, NOT `impl`
  methods on `Dnp3Analyzer`. Kani calls them directly without constructing the struct.
- `classify_dnp3_fc` MUST contain `_ => Dnp3FcClass::Unknown` as the final match arm.
  No `unreachable!` is permitted; the wildcard arm is required for the VP-023 Sub-B totality proof.
- `parse_dnp3_dl_header` does NOT check sync bytes (`0x05 0x64`) or LENGTH validity. The
  separation between parsing (extract fields) and validation (`is_valid_dnp3_frame_header`)
  allows VP-023 Sub-A to range over all 2^80 possible 10-byte inputs.
- `compute_dnp3_frame_len` uses integer ceiling arithmetic `(u + 15) / 16` — no floating-point.
- This module MUST NOT import `crate::analyzer::modbus` or any external DNP3 crate.

## Consequences

- `src/analyzer/dnp3.rs`: exposes a custom `on_data(flow_key, data, ts, direction)` /
  `on_flow_close` interface (RULING-DNP3-SIBLING-001 §1.5 — `direction: Direction` added);
  does **not** implement `StreamAnalyzer` or `StreamHandler`. Pure-core functions are free `fn`s.
  `Dnp3FlowState` carries two directional 292-byte carry buffers (`carry_c2s`, `carry_s2c`),
  per-flow master address set, pending request table, and six windowed counters.
  `Dnp3Analyzer` aggregates findings and per-flow state across all flows.
- `src/dispatcher.rs`: `DispatchTarget::Dnp3` variant added. Rule 6 (port 20000) appended.
  `dnp3: Option<Dnp3Analyzer>` field added to `StreamDispatcher`.
- VP-023 Kani harnesses (sub-properties A–D) are gated by `#[cfg(kani)]` in
  `src/analyzer/dnp3.rs` and proven correct.
- Tests are in `tests/dnp3_analyzer_tests.rs` following the prose-style naming convention
  (ADR 0002 / CLAUDE.md).
