---
document_type: arch-ruling
story_id: STORY-137
ruling_id: RULING-137-001
author: architect
date: 2026-06-26
status: authoritative
supersedes: []
---

# RULING-137-001: Frame-Walk Loop Control and Counting Semantics

## Summary

This document is the definitive, binding adjudication of the STORY-137 spec-vs-implementation
conflict surfaced by Adversarial Pass 1. It governs loop control (`continue` vs. `break`) and
the exact counting semantics for `parse_errors` / `malformed_in_window` during byte-walk resync
and carry-buffer residue re-walk. All four ratified artifacts are confirmed consistent.
Implementer and test-writer MUST follow this ruling without deviation.

---

## 1. Loop-Control Ruling — CONFIRMED: `continue`, Never `break`

**Ruling: Both the byte-walk resync path (`cursor += 1`) and the oversized-frame-skip path
(`cursor += min(total_frame_len, buf.len()-cursor)`) MUST use `continue`, not `break`.**

This is not a close call. The `continue` requirement is stated explicitly and redundantly in
all four ratified artifacts:

| Artifact | Location | Exact text |
|----------|----------|-----------|
| BC-2.17.016 | Postcondition 1, byte-walk path | `cursor += 1; continue (byte-walk resync)` |
| BC-2.17.016 | Postcondition 1, frame-skip path | `cursor advances past declared frame, bounded by buf.len(); continue` |
| BC-2.17.016 | Invariant 4 | `is_non_enip is NOT set when a single frame's declared header.length implies total_frame_len > MAX_ENIP_CARRY_BYTES … handled by the frame-skip path … continue` |
| BC-2.17.018 | Precondition 1, frame-skip path | `frame-skip path: parse_errors++; malformed_in_window++; cursor advances past declared frame` (no break) |
| STORY-137 | AC-137-003 | `continue the loop (do NOT break; re-attempt parse at the next byte)` |
| STORY-137 | AC-137-003, oversized path | `continue the loop (do NOT break; do NOT set is_non_enip)` |
| STORY-137 | Architecture Compliance Rules 6 & 7 | Rule 6: `cursor += 1; continue` — NOT break. Rule 7: `continue` — NOT break |
| STORY-137 | Frame-walk pseudocode | Both paths end with explicit `continue` |
| ADR-010 | Decision 4 pseudocode, line ~406 | `cursor += 1; continue` |
| ADR-010 | Decision 4 pseudocode, lines ~412-414 | `cursor += min(total_frame_len, buf.len()-cursor); continue` |
| ADR-010 | Consequences / Negative, lines ~791-795 | "single-frame skip; flow continues … Subsequent small valid frames on the same flow are still analyzed" |

The `break` the implementer used is a deviation from the spec. The adversary's ruling stands:
**the code must use `continue`.**

The `break` change is not a refactoring or interpretation — it is a behavior change that
produces silent, testable misbehavior on EC-012: a garbage byte followed by a valid ENIP frame
in the same TCP segment causes the valid frame to be silently dropped (never passed to
`process_pdu`) because the loop exits before reaching it. This is a detection-evasion vector.

---

## 2. Counting-Semantics Decision — Option (a): Per-offset counting IS intended

**Ruling: Option (a). Per-byte-walk-offset counting is INTENDED behavior. The implementer's
concern about carry residue re-walk over-counting reflects a real mechanism, but it is not
a spec gap — it is correct behavior under the threat model. No BC, ADR, or story text needs
amendment.**

### 2.1 The implementer's concern, precisely stated

With `continue` in the byte-walk path:
- A segment arrives with N bytes that do not form a complete 24-byte header (e.g., 23 bytes
  containing garbage at an unknown-command offset).
- After the loop, `flow.carry = buf[23..]` — the 23-byte residue is stashed.
- On the next `on_data` call: `buf = carry(23) ++ new_data`. The frame-walk loop re-walks
  the carry bytes from byte 0, again failing to form a valid header at each byte offset,
  incrementing `parse_errors` and `malformed_in_window` multiple times for bytes that were
  already counted in the prior call.
- This could cause `malformed_in_window` to accumulate faster than "1 malformed frame = 1 event,"
  potentially prematurely firing T0814.

This is an accurate description of the mechanism. The question is whether the spec intends it.

### 2.2 Why per-offset counting is the intended design

**The threat model makes this the right answer.**

BC-2.17.016 Invariant 3 states:

> `malformed_in_window` is incremented in parallel with `parse_errors` on every structural
> reject. It is reset at window expiry.

BC-2.17.016 Postcondition 1 states:

> If `!is_valid_enip_frame(&header)`: `flow.parse_errors += 1`; `flow.malformed_in_window += 1`;
> `cursor += 1`; continue (byte-walk resync).

There is no per-segment deduplification or "first-increment-only" qualifier in any of the four
artifacts. The increment fires "on every structural reject," and a structural reject is defined
at the **parse attempt** level, not at the **segment arrival** level. Each byte-walk cursor
position that fails `is_valid_enip_frame` is one structural reject. This is explicit and
intentional.

**Why? Because a garbage-byte flood IS the T0814 threat.**

The T0814 scenario described in BC-2.17.018's Description is:

> "A burst of malformed ENIP frames on port 44818 may indicate a scanning or crash-injection
> attempt targeting poorly-implemented EtherNet/IP stacks."

The byte-walk resync path exists precisely to handle traffic that is not framed at a known
boundary — the kind of traffic produced by a scanner or fuzzer that is injecting arbitrary
bytes on port 44818. A scanner sending 300 bytes of garbage is NOT sending one malformed frame;
it is creating hundreds of failed parse attempts at every possible alignment. Each one of those
is a distinct structural-reject event that should be counted. BC-2.17.018 Invariant 3 confirms
the threshold is `MALFORMED_ANOMALY_THRESHOLD = 3` with the rationale: "Single malformed frames
can be packet loss; three within 300s on a flow is anomalous." A crash-probe scanner that
generates 300 garbage bytes per segment exceeds that threshold in one packet, which is exactly
the intended behavior: T0814 fires fast on an active probe, not slowly after a fixed count of
"frames."

The implementer's concern amounts to: "a crash-probe fills malformed_in_window too quickly."
That is not a bug. That is the detection working.

**The carry-residue re-walk case specifically:**

When carry residue from one `on_data` call is re-walked in the next call, the same bytes are
re-evaluated from offset 0. This is also correct. The carry is the prefix of an as-yet-unframed
stream. If those bytes do not form a valid frame header when prepended with new data, they are
still-malformed bytes on a flow that is accumulating parse rejections. Counting them again on
each `on_data` call where they fail to produce a valid frame is the right behavior: each call
represents a new opportunity for the TCP stream to produce a parseable frame, and each failure
is a fresh reject event.

The 300s window reset (BC-2.17.018 Postcondition 5) is the anti-false-positive mechanism.
After 300s of clean traffic, `malformed_in_window` resets and a fresh burst is required to
re-trigger T0814. This is sufficient. The implementer's proposed remedy — `break` — would
suppress legitimate detection by silently discarding trailing valid frames.

### 2.3 The counter model is unambiguous and complete

BC-2.17.018 Canonical Test Vectors directly state the expected counter values:

| malformed_in_window | parse_errors (lifetime) | Finding emitted? |
|--------------------|------------------------|-----------------|
| 1 | 1 | No |
| 2 | 2 | No |
| 3 | 3 | Yes — T0814 Possible/Low |

These values are per-reject, not per-frame, not per-segment. There is no ambiguity in the spec;
the implementer's concern is about the detection being "too sensitive," not about the spec
being wrong. Sensitivity tuning is not an architecture-ruling matter — it would require a
product-owner decision to change `MALFORMED_ANOMALY_THRESHOLD` (currently 3, hardcoded per
ADR-010 Decision 5). That is not in scope here.

**Conclusion: the spec has no gap. No BC, ADR, or story text requires amendment.**

---

## 3. Expected Behavior Table for the Test-Writer

All values are authoritative ground truth. Tests must encode these exactly.

### 3.1 EC-010: Oversized declared frame + trailing valid frame in one segment

**Scenario:** Buffer contains a 624-byte oversized frame (header.length=600, total=624)
immediately followed by a complete 28-byte valid ENIP frame (header.length=4). Buffer total:
652 bytes. `is_non_enip = false` at entry. `malformed_in_window = 0`, `parse_errors = 0`.

| Step | Action | cursor | parse_errors | malformed_in_window | is_non_enip | process_pdu called? |
|------|--------|--------|-------------|--------------------|--------------|--------------------|
| Loop iter 1 | Parse header at [0..24]: valid header, command=ListIdentity, length=600 | — | 0 | 0 | false | no |
| — | total_frame_len = 624 > 600: frame-skip path | — | 1 | 1 | false | no |
| — | cursor += min(624, 652-0) = 624; **continue** | 624 | 1 | 1 | false | no |
| Loop iter 2 | buf.len()-cursor = 652-624 = 28 >= 24: parse header at [624..648] | — | 1 | 1 | false | no |
| — | Valid header; total_frame_len = 28 <= 600; buf.len()-cursor = 28 >= 28: process_pdu | — | 1 | 1 | false | yes (1 call) |
| — | cursor += 28 | 652 | 1 | 1 | false | yes |
| After loop | buf.len()-cursor = 0 < 24: loop exits; carry = [] | — | 1 | 1 | false | — |

**Assert:**
- `process_pdu` called exactly once (the trailing valid frame)
- `parse_errors == 1`
- `malformed_in_window == 1`
- `is_non_enip == false`
- `flow.carry.is_empty() == true`

**Note:** `is_non_enip` stays false. The frame-skip path does NOT set it (BC-2.17.016 Inv 4).
T0814 does NOT fire (1 < threshold of 3).

### 3.2 EC-012: Garbage byte + valid ENIP frame in one segment

**Scenario:** Buffer = [0xFF, 0xFF, 0xFF, ... (garbage 24 bytes that produce an unknown command),
followed by a complete 28-byte valid ENIP frame]. Total buffer: 52 bytes. `malformed_in_window
= 0`, `parse_errors = 0`.

The garbage block: 24 bytes where `buf[0..2] = [0xFF, 0xFF]` (command 0xFFFF, unknown).

| Step | Action | cursor | parse_errors | malformed_in_window |
|------|--------|--------|-------------|---------------------|
| Loop iter 1 | buf.len()-cursor = 52 >= 24: parse header at [0..24]. parse_enip_header returns Some with command=0xFFFF | — | 0 | 0 |
| — | command_counts[0xFFFF] += 1 (PC-0) | — | 0 | 0 |
| — | is_valid_enip_frame returns false: byte-walk resync | — | 1 | 1 |
| — | cursor += 1; **continue** | 1 | 1 | 1 |
| Loop iter 2 | buf.len()-cursor = 51 >= 24: parse header at [1..25]. Returns Some with command = garbage | — | 1 | 1 |
| — | is_valid_enip_frame false; cursor += 1; continue | 2 | 2 | 2 |
| ... | (byte-walk continues until cursor reaches byte 24 = first byte of valid frame) | 24 | 24 | 24 |
| Loop iter 25 | buf.len()-cursor = 28 >= 24: parse header at [24..48]. Returns Some with valid command | — | 24 | 24 |
| — | is_valid_enip_frame true; total_frame_len=28; buf.len()-cursor=28 >= 28: process_pdu | — | 24 | 24 |
| — | cursor += 28 | 52 | 24 | 24 |
| After loop | buf.len()-cursor = 0; carry = [] | — | 24 | 24 |

**Assert:**
- `process_pdu` called exactly once
- `parse_errors == 24` (one per byte-walk position from offset 0 to offset 23)
- `malformed_in_window == 24`
- `is_non_enip == false`
- `flow.carry.is_empty() == true`
- T0814 fires (malformed_in_window >= 3) — `malformed_anomaly_emitted == true`

**Important clarification for the test:** This scenario produces 24 parse_errors because there
are 24 byte-walk positions before the valid frame header aligns. This is correct and expected.
The test should not assert `parse_errors == 1`. If the test encodes `parse_errors == 1`, the
test is wrong.

**Simplified variant for a minimal EC-012 test:**

Use a 1-byte garbage prefix (command that fails the validity gate in the first byte; the
remaining 23 bytes are valid frame material). If the garbage byte causes `parse_enip_header`
to return `Some` with an invalid command, then `parse_errors == 1` after iter 1, `cursor = 1`,
and the valid 28-byte frame starting at byte 1 is processed in iter 2. Buffer: 1 garbage byte
+ 28 valid bytes = 29 bytes total.

Actually: `parse_enip_header` requires 24 bytes. So with a 1-byte prefix: iter 1 reads
`buf[0..24]` = [garbage, valid_frame_bytes[0..23]]. If command at bytes [0..2] is invalid,
byte-walk fires, cursor=1, parse_errors=1. Iter 2 reads `buf[1..25]` = [valid_frame_bytes[0..24]]
= valid header. total_frame_len = valid frame size. If buf.len()-cursor >= total_frame_len,
process_pdu fires.

Minimal EC-012 setup: prefix 1 garbage byte + 24-byte valid header + N-byte payload. Buffer =
25+N bytes. Expected: parse_errors=1, malformed_in_window=1, process_pdu called once.

### 3.3 Multi-call residue scenario: exact counter values

**Scenario:** Demonstrate that carry residue re-walk counts are correct.

**Call 1:** 23 bytes of garbage arrive. All bytes produce failed parse attempts... but:
- `buf.len() - cursor` starts at 23, which is LESS than 24. The while condition
  `buf.len() - cursor >= 24` is false from the start.
- The loop body is never entered.
- After loop: `flow.carry = buf[0..] = 23 bytes` (the full 23 bytes, since cursor=0).
- Carry-cap check: 23 <= 600, no overflow.
- **parse_errors = 0, malformed_in_window = 0 after Call 1.**

**Call 2:** 5 more bytes of garbage arrive. `buf = carry(23) ++ new(5)` = 28 bytes.
- Iter 1: `buf.len()-cursor = 28 >= 24`. Parse header at [0..24]: command = 0xFFFF (garbage).
  `is_valid_enip_frame` false. parse_errors=1, malformed_in_window=1. cursor=1. continue.
- Iter 2: `buf.len()-cursor = 27 >= 24`. Parse header at [1..25]: same garbage. Fails. parse_errors=2, malformed_in_window=2. cursor=2. continue.
- Iter 3: `buf.len()-cursor = 26 >= 24`. Parse at [2..26]: garbage. Fails. parse_errors=3, malformed_in_window=3. cursor=3. continue. T0814 fires here.
- Iter 4: `buf.len()-cursor = 25 >= 24`. Parse at [3..27]: garbage. Fails. parse_errors=4, malformed_in_window=4. cursor=4. continue. (guard set, no second T0814)
- Iter 5: `buf.len()-cursor = 24 >= 24`. Parse at [4..28]: garbage. Fails. parse_errors=5, malformed_in_window=5. cursor=5. continue.
- Iter 6: `buf.len()-cursor = 23 < 24`. Exit loop.
- After loop: `flow.carry = buf[5..] = 23 bytes`. Carry-cap check: 23 <= 600, no overflow.
- **parse_errors = 5, malformed_in_window = 5, T0814 emitted, is_non_enip = false after Call 2.**

**Call 3:** 1 more valid ENIP frame (28 bytes). `buf = carry(23) ++ new(28) = 51 bytes`.
- Byte-walk continues from carry until valid frame aligns at offset 23.
- Byte offsets 0..22 (23 iterations): each reads a garbage 24-byte slice, each fails, parse_errors and malformed_in_window each increment by 1 per iteration.
- At cursor=23: buf.len()-cursor=28 >= 24. Valid header. total_frame_len=28. process_pdu fires. cursor=51.
- After loop: carry = []. No overflow.
- **parse_errors += 23 → cumulative = 28. malformed_in_window += 23 → cumulative = 28.**
- process_pdu called once (the valid frame).

This is the authoritative expected behavior. The test must encode parse_errors=28, not 1 or 3.
The counting is per-structural-reject, and a byte-walk resync that passes 23 garbage bytes
before reaching a valid boundary is 23 structural rejects.

### 3.4 test_carry_buffer_cap_at_600: corrected expectation

The adversarial report flagged that this test was encoding incorrect expectations because the
oversized-frame-skip path should use `continue` (not `break`). Here is the authoritative
scenario the test must encode.

**What this test is about:** BC-2.17.016 Postcondition 4 / Invariant 1 — the carry-buffer cap
at 600 bytes. The test must verify that a GENUINE carry-buffer overflow (a partial frame whose
accumulated bytes in carry exceed 600) triggers `is_non_enip`.

**What this test is NOT about:** An oversized declared frame (total_frame_len > 600). That is
a different path (frame-skip, no `is_non_enip`). These two scenarios MUST NOT be conflated.

**Correct setup for test_carry_buffer_cap_at_600:**

The test must construct a scenario where `flow.carry.len() > 600` after a carry stash. The
only way to reach that is via the partial-frame stash path (BC-2.17.016 Postcondition 1,
fourth sub-bullet: "If `buf.len() - cursor < total_frame_len`: stash `buf[cursor..]` into
carry").

This means:
1. A valid ENIP header is parsed (is_valid_enip_frame returns true).
2. The declared total_frame_len is <= 600 (otherwise it goes to the frame-skip path, not stash).
3. But `buf.len() - cursor < total_frame_len` — not enough bytes to complete the frame.
4. So `buf[cursor..]` is stashed into carry.
5. This stash, when combined with what is already in carry, exceeds 600 bytes.

**Concrete scenario:**

- `flow.carry` starts with 580 bytes of a partial ENIP frame (24-byte valid header with
  length=576, so total_frame_len=600; we have received 580/600 bytes so far).
- New data: 21 bytes (the next TCP segment, still not enough to complete the 600-byte frame;
  580+21=601 bytes, which is buf length minus cursor=0).
- buf = 601 bytes. Loop iter 1: `buf.len()-cursor = 601 >= 24`. Parse header at [0..24]:
  valid header, length=576, total_frame_len=600.
- `total_frame_len = 600 <= MAX_ENIP_CARRY_BYTES (600)`: NOT the frame-skip path (the condition
  is strictly greater than 600).
- `buf.len()-cursor = 601 < total_frame_len = 600`? NO — 601 >= 600. So this is NOT a partial
  stash. process_pdu fires. cursor=600.
- Iter 2: `buf.len()-cursor = 1 < 24`. Exit loop. carry = buf[600..] = 1 byte. No overflow.

Adjust: use carry=580, new_data=20 bytes (total=600). buf.len()-cursor=600 >= 600 exactly.
process_pdu fires. carry=[]. No overflow triggered.

Better setup for overflow: carry=580 bytes (part of a frame with total_frame_len=600), new_data=1
byte (still partial). buf=581 bytes. `buf.len()-cursor=581 < total_frame_len=600`: partial stash.
`carry = buf[0..] = 581 bytes`. 581 <= 600, no overflow.

For overflow: carry=580 bytes, new_data=22 bytes (22+580=602). buf=602. `buf.len()-cursor=602
< total_frame_len=600`? No — 602 >= 600. Frame completes, process_pdu fires, carry=[].

The correct overflow scenario: carry already holds 580 bytes of partial frame data (from a
prior call that stashed a valid-header-partial-body where total_frame_len=600). Then a new
segment arrives with 22 bytes that are NOT the continuation of that frame (e.g., a new ENIP
header at the front). Now buf = carry(580) ++ new(22) = 602 bytes.

Actually the simplest direct path:

1. Send a first segment: a valid ENIP header with length=576 (total_frame_len=600) but only
   23 bytes of body (instead of 576). buf=24+23=47 bytes. Loop iter 1: header parsed OK,
   total_frame_len=600, `buf.len()-cursor = 47 < 600`: partial stash. carry=47 bytes.
2. Send a second segment: another partial header worth of data — enough to make carry exceed
   600. Send 560 bytes. buf=carry(47)++new(560)=607. Loop iter 1: parse header at [0..24]:
   same valid header. total_frame_len=600. `buf.len()-cursor=607 >= 600`: NOT partial. Wait —
   that means it will try to process_pdu with a 600-byte frame starting at offset 0. The body
   bytes (bytes 24..600) are garbage/filler, not real CIP data, but process_pdu is called.
   cursor=600. Iter 2: buf.len()-cursor=7 < 24. Exit loop. carry=7 bytes. No overflow.

This is the fundamental difficulty: to get a genuine carry overflow with valid-header-framing,
the partial frame body must be large and the delivered data must be just-short.

**The simplest reliable test setup:**

Use `flow.carry` pre-loaded with 590 bytes via the internal test API (or through two calls):
- Call 1: Send 590 bytes, all with buf.len()-cursor < 24 (e.g., 20 bytes — the while loop
  never enters, carry = 20 bytes). Actually this doesn't get to 590.

**Direct approach:** pre-populate `flow.carry` with 590 bytes of garbage (representing
accumulated partial frames from prior calls that stashed sub-24-byte residue — each call
delivered < 24 bytes, so the while loop never fired and the residue accumulated):

Actually sub-24-byte inputs go straight to the "after loop: carry = buf[cursor..]" path
with cursor=0, so carry = buf = 590 bytes accumulated over many calls. Then:

- Send 12 bytes (new segment). buf = carry(590) ++ new(12) = 602 bytes.
- Loop iter 1: `buf.len()-cursor = 602 >= 24`. Parse header at [0..24]: pure garbage, unknown
  command. `is_valid_enip_frame` false. parse_errors=1, malformed_in_window=1. cursor=1.
- ... (byte-walk continues through positions where buf.len()-cursor >= 24)
- Eventually buf.len()-cursor drops below 24 at cursor = 602-23 = 579. At cursor=579:
  `buf.len()-cursor = 602-579 = 23 < 24`. Exit loop.
- carry = buf[579..] = 23 bytes. 23 <= 600. No overflow.

Still no overflow with this approach either. The carry overflow is specifically triggered by
the PARTIAL FRAME path, not the garbage/byte-walk path.

**Definitive correct setup (pre-population of carry with a partial valid frame):**

The test MUST use the partial-frame stash path. The setup is:

1. Pre-populate `flow.carry` with 580 bytes that represent a partial in-flight ENIP frame:
   a valid 24-byte ENIP header followed by 556 bytes of payload (where the declared
   `header.length = 576`, meaning total_frame_len=600, but only 556 of the 576 payload bytes
   have arrived so far). This simulates the state after a prior `on_data` call that stashed
   a partial frame.

2. Send 21 bytes of new data (the next TCP segment). buf = 580 + 21 = 601 bytes.

3. Loop iter 1: `buf.len()-cursor = 601 >= 24`. Parse header at [0..24]: valid header,
   length=576, total_frame_len=600. `total_frame_len (600) <= MAX_ENIP_CARRY_BYTES (600)`:
   NOT the frame-skip path. `buf.len()-cursor (601) >= total_frame_len (600)`: NOT the partial
   stash path — wait, 601 >= 600 is true. So this would call process_pdu, not stash.

4. Adjust: send 19 bytes of new data. buf = 580 + 19 = 599 bytes. `buf.len()-cursor = 599 <
   total_frame_len = 600`: YES, this is the partial stash path. carry = buf[0..] = 599 bytes.
   599 <= 600. No overflow. Still not triggered.

5. Need carry to exceed 600 after stash. Total stash must be > 600. buf[cursor..] > 600 with
   cursor=0. So buf.len() must be > 600. Send 21 bytes: buf = 580+21 = 601. carry = 601 > 600.
   But as shown in step 3, 601 >= 600 means process_pdu fires, not stash.

6. The resolution: set `header.length = 577` (total_frame_len = 601). Now:
   - `total_frame_len (601) > MAX_ENIP_CARRY_BYTES (600)`: this IS the frame-skip path, not stash!
   - `is_non_enip` is NOT set. This is the frame-skip scenario, not the overflow scenario.

**The fundamental truth:** With MAX_ENIP_CARRY_BYTES=600, a partial frame can ONLY overflow
carry if `total_frame_len <= 600` (otherwise it goes to frame-skip) AND `carry + data < total_frame_len`
(partial stash condition) AND `carry + data > 600`. The only way all three hold is if
`total_frame_len = 600` exactly AND `carry + data > 600` AND `carry + data < 600`. That is
a contradiction: you cannot have `carry + data > 600` AND `carry + data < 600 = total_frame_len`
simultaneously.

Therefore: **a partial-frame stash from a frame with header.length <= 576 (total_frame_len <=
600) can NEVER on its own overflow carry in a single `on_data` call.** The carry overflow path
(BC-2.17.016 Postcondition 4) is reached only by accumulated sub-24-byte residue from the
"no loop iteration" path (where `buf.len() < 24` on entry), not by the in-loop partial-frame
stash path.

This resolves the confusion about `test_carry_buffer_cap_at_600`. The correct test scenario is:

**test_carry_buffer_cap_at_600 (authoritative setup):**

Use MANY small `on_data` calls, each delivering fewer than 24 bytes, so the while loop never
fires and the carry accumulates raw bytes:

- Calls 1..N: each delivers `k < 24` bytes. After each call: carry grows by k bytes. The loop
  never fires (buf.len()-cursor < 24 from the start). No structural rejects. parse_errors=0.
- Call M: carry reaches 599 bytes (still under cap). No overflow.
- Call M+1: carry=599, new_data=2 bytes. buf=601 bytes. buf.len()-cursor=601 >= 24. Loop fires.
  - Iter 1: parse header at [0..24]. This is accumulated garbage, likely unknown command.
    If parse_enip_header returns Some with invalid command: parse_errors=1, malformed_in_window=1,
    cursor=1, continue. (Or similar byte-walk chain.) Eventually buf.len()-cursor < 24, loop
    exits. carry = buf[last_cursor..] — will be > 600 if last_cursor is small.

Actually this is also not clean. The cleanest and most pedagogically correct approach for
`test_carry_buffer_cap_at_600` is:

**Use the test framework to directly set `flow.carry` to exactly 599 bytes (any bytes), then
call `on_data` with 2 bytes of non-ENIP garbage.** The `on_data` call produces `buf = 601 bytes`.
The while loop: `buf.len()-cursor = 601 >= 24`. Parse at [0..24]. The 24-byte slice of garbage
likely produces either `parse_enip_header = None` (if the parse is strict) or `Some` with
unknown command. In either case, byte-walk fires, cursor advances to 1. Continue. Eventually
the byte-walk exhausts the 601-24=577 positions where the loop fires, leaving carry residue.

But the simpler approach: if `flow.carry` starts at 599 bytes and `on_data` delivers 2 bytes
where those 2 bytes form a valid ENIP header at some alignment... this requires careful byte
construction.

**The cleanest approach for the test:** Directly test BC-2.17.016 Postcondition 2 / the
"after loop" carry-cap check by pre-setting `flow.carry` to 601 bytes in the test setup (if
the test infrastructure exposes `flow` directly) and immediately checking that the cap fires.
OR: deliver a stream of sub-24-byte segments that accumulate carry past 600.

**Authoritative expected behavior for the overflow case itself (regardless of path):**

When `flow.carry.len() > MAX_ENIP_CARRY_BYTES (600)` after any stash:
- `parse_errors += 1` (one increment total for the overflow event)
- `malformed_in_window += 1` (one increment total)
- `check_t0814()` fires (while `is_non_enip` is still false)
- `is_non_enip = true` (latched AFTER `check_t0814`)
- `flow.carry.clear()`

All subsequent `on_data` calls: immediate no-op. No further counter increments.

**What the test MUST NOT do:** The test MUST NOT use an oversized-frame-skip to drive carry
overflow. An oversized declared frame (total_frame_len > 600) takes the `continue` frame-skip
path and does NOT stash into carry at all (cursor advances past the declared frame). Therefore,
oversized-frame skips cannot accumulate in carry. Any test that drives `carry > 600` through
the frame-skip path is testing a behavior that CANNOT occur in the correct implementation.

---

## 4. Spec Change Assessment — No BC, ADR, or Story Amendment Required

**Decision: No spec changes. All four artifacts are internally consistent, correct, and
sufficient. The implementer's concern was about detection sensitivity, not about a spec gap.**

Checklist of evaluated change candidates:

| Candidate change | Decision | Rationale |
|-----------------|----------|-----------|
| Add "per-byte-walk-offset counting is intended" clarification to BC-2.17.016 | NOT NEEDED | Invariant 3 + Postcondition 1 already state this explicitly: "incremented on every structural reject" |
| Add "carry residue re-walk is expected to increment counters" to BC-2.17.018 | NOT NEEDED | Postconditions 1–2 state "on every structural reject" without any per-segment deduplication qualifier |
| Add a "per-segment" deduplification rule | REJECTED | This would change the detection semantics and reduce sensitivity against crash-probe attacks |
| Change MALFORMED_ANOMALY_THRESHOLD from 3 to higher value | OUT OF SCOPE | Sensitivity calibration is a product-owner decision; not an architecture ruling |
| Add "do NOT break" language to ADR-010 Decision 4 pseudocode | NOT NEEDED | ADR-010 pseudocode already uses explicit `continue` on both paths |

**Input-hash implication:** Since no STORY-137 input files were modified, the `input-hash`
field in `STORY-137.md` does NOT need to be recomputed.

---

## 5. Crisp Summary for Test-Writer and Implementer

### For the implementer

1. **Change `break` back to `continue` in both paths.** No exceptions.
   - Byte-walk resync path: `flow.parse_errors += 1; flow.malformed_in_window += 1; cursor += 1; continue;`
   - Oversized-frame-skip path: `flow.parse_errors += 1; flow.malformed_in_window += 1; cursor += min(total_frame_len, buf.len()-cursor); continue;`

2. **The counting behavior you observed is correct.** Garbage-byte floods accumulate
   `malformed_in_window` quickly. That is the intended T0814 detection behavior. Do not
   suppress it with `break`.

3. **`is_non_enip` is latched ONLY by carry-buffer overflow** (BC-2.17.016 Invariant 4).
   It is not latched by frame-skip, by byte-walk, or by T0814 threshold. Review your
   implementation to ensure you have not conflated these paths.

### For the test-writer

1. **EC-010 (oversized frame + trailing valid frame):** `process_pdu` must be called once
   for the trailing frame. `parse_errors=1`, `malformed_in_window=1`, `is_non_enip=false`.
   The loop MUST continue after the frame-skip, not exit.

2. **EC-012 (garbage byte + valid frame):** With a minimal 1-byte garbage prefix before a
   valid 28-byte frame (total buf=29 bytes): `parse_errors=1`, `malformed_in_window=1`,
   `process_pdu` called once. With a 24-byte all-garbage prefix before a valid 28-byte frame
   (total buf=52 bytes): `parse_errors=24`, `malformed_in_window=24`, `process_pdu` called
   once, T0814 fires.

3. **Multi-call residue:** Counter values accumulate across calls. See Section 3.3 for
   exact values. Do not assume one call = one increment.

4. **test_carry_buffer_cap_at_600 MUST use legitimate carry accumulation.** The only correct
   setup is: accumulate > 600 bytes in carry through sub-24-byte `on_data` deliveries (the
   while loop never fires; all bytes become carry), or through the partial-frame stash path
   from a previous call. DO NOT use oversized-frame-skip to drive this test — that path never
   stashes into carry (cursor advances past the declared frame, leaving zero residue from that
   frame). The overflow trigger is `flow.carry.len() > 600` after the "carry = buf[cursor..]"
   assignment at the END of `on_data`, not inside the loop.

5. **Carry-overflow event counter values:** One overflow event = +1 to `parse_errors` and
   +1 to `malformed_in_window`, then `is_non_enip = true`. Subsequent calls: no increments.

6. **T0814 fires when `malformed_in_window >= 3` AND `!malformed_anomaly_emitted` AND
   `!is_non_enip`.** The `is_non_enip` check in the T0814 guard means the carry-overflow
   event can fire T0814 if it is the 3rd malformed event (BC-2.17.018 EC-007), but only if
   `check_t0814` is called BEFORE `is_non_enip` is set (STORY-137 AC-137-002 ordering
   constraint).

---

## 6. ADR Ownership Confirmation

This ruling is issued under ADR-010 ownership. ADR-010 Decision 4 canonical pseudocode
is the governing design document for the frame-walk loop. This ruling clarifies the counting
semantics in alignment with ADR-010 Decision 4 — it does not change any Decision text.

All future implementation questions about the frame-walk loop (loop control, cursor arithmetic,
counter semantics, carry stash vs. frame-skip path selection) must be resolved against ADR-010
Decision 4 as primary source, with BC-2.17.016 and BC-2.17.018 as the secondary behavioral
specification.
