---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.016.md
  - .factory/stories/STORY-137.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-118"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.016
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap with two TCP segments: segment 1 has an oversized ENIP frame (declared length > 576 bytes so total > 600), segment 2 has a well-formed valid ENIP frame. Tests that the oversize skip does not drop the whole flow."
---

# Holdout Scenario: Oversize Declared Frame Skipped; Subsequent Valid Frame Still Analyzed

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

An ENIP frame whose declared payload length (header.length field) causes the total frame
size `24 + header.length` to exceed the carry buffer cap (`MAX_ENIP_CARRY_BYTES = 600`)
is SKIPPED by the frame-walk loop — the cursor advances past it. Critically, the skip does
NOT set `is_non_enip = true`. Subsequent frames on the same flow continue to be analyzed
normally. This tests that the frame-walk loop's per-frame skip is truly per-frame, not a
whole-flow abort.

**Key distinction from carry-overflow:** An oversize *declared-length* frame (where the
header.length field declares more than 576 payload bytes) is skipped via the frame-skip
path. A carry-buffer *overflow* (where the partial-frame stash would exceed 600 bytes) sets
`is_non_enip = true` and silences the flow. These are two different code paths. This holdout
tests the frame-skip path.

### Case A — Oversize Frame Followed by Valid Frame: Both Processed

1. A crafted PCAP is presented: one TCP flow on port 44818; two TCP segments delivered
   in sequence:
   - Segment 1: An ENIP header with command=0x006F (SendRRData), length=0x0230 (560 bytes,
     so total=24+560=584 bytes, which is within a single segment — BUT this requires 584
     bytes of TCP payload; to keep the fixture simple, use length=0x0201 (513 bytes, total
     24+513=537 bytes, still within 600 — wait, that's < 600). Instead, use
     **length=0x0258 (600 bytes, total=24+600=624 > MAX=600)** — this exceeds the cap.
     The TCP segment only carries the 24-byte header + some bytes but declares 600 payload
     bytes; alternatively, present a TCP segment with the full 624 bytes where the declared
     length causes a skip.
   - **Simplest approach for F4:** Present the ENIP header alone (24 bytes) with
     length=0x0258 (600), making total_frame_len=624 > 600. The frame-walk sees 24 bytes,
     parses the header (24 bytes available >= 24), checks total_frame_len=624 > 600, triggers
     the skip path, advances cursor by min(624, remaining_buf_len). Since only 24 bytes are
     available, cursor advances by 24 (the whole segment). The oversized frame is skipped.
   - Segment 2: A valid 28-byte ENIP frame (24-byte header with command=0x0063 (ListIdentity),
     length=0, status=0, session=0) — this is a well-formed zero-payload ListIdentity.

2. The user runs: `wirerust analyze enip_oversize_then_valid.pcap --enip --json`
3. The tool exits 0 (no panic).
4. The evaluator confirms:
   - A T0846 finding IS present (the ListIdentity in segment 2 was processed; if `is_non_enip`
     had been set, the ListIdentity would be silently ignored and no T0846 finding would appear).
   - No panic. No crash.

### Case B — Oversize Frame Alone: Increments parse_errors, No T0846 (No Panics)

1. A second PCAP: only the oversized frame (24-byte header, length=0x0258=600, no valid
   frames following).
2. The user runs: `wirerust analyze enip_oversize_only.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: no panic; pdu_count = 0 (no valid frame was fully processed);
   no T0846 or other detection finding. If enough oversized frames are present (>= 3),
   a T0814 finding may appear (oversize path increments malformed_in_window — covered in
   HS-117). With only one oversize frame, no T0814 should appear.

### Case C — Oversized Frame Does NOT Set is_non_enip (Flow Continues)

This is already verified by Case A (if is_non_enip were set, the subsequent ListIdentity
would not be analyzed and no T0846 would appear). The evaluator should note explicitly:
the presence of the T0846 finding in Case A is the observable proof that `is_non_enip`
was NOT set by the oversized frame.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.016 | Frame-skip path — oversized declared frame: cursor advances, NOT is_non_enip=true | Case A: subsequent valid frame analyzed (T0846 appears) |
| BC-2.17.016 | Postcondition: frame-skip path increments parse_errors; malformed_in_window++ | Case B: no panic despite oversize |
| BC-2.17.016 | Frame-skip cursor advance: min(24 + header.length, buf.len() - cursor) | Cases A/B: bounded advance prevents OOB |
| BC-2.17.016 | is_non_enip=true path is CARRY OVERFLOW only, not frame-skip | Case C: oversize skip != is_non_enip=true |

<!-- HIDDEN TRACEABILITY: BC-2.17.016 Postcondition — frame-skip path (line: if 24 + header.length as usize > MAX_ENIP_CARRY_BYTES); carry-overflow path (line: if carry.len() > MAX_ENIP_CARRY_BYTES → is_non_enip = true) are distinct. -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_oversize_then_valid.pcap` — TCP flow dst port 44818; segment 1: 24-byte ENIP
   header with command=0x006F, length=0x0258 (600 — causes total 624 > 600 cap); segment 2
   (possibly in same TCP segment or a subsequent one): 24-byte ENIP header with command=0x0063
   (ListIdentity), length=0. The two frames can be concatenated in one TCP segment if desired:
   bytes 0-23 (oversize header), bytes 24-47 (ListIdentity header).
2. `enip_oversize_only.pcap` — TCP flow with only the 24-byte oversize header, no following
   valid frame.

## Verification Approach

```bash
wirerust analyze enip_oversize_then_valid.pcap --enip --json
# Expect: exit 0; 1 T0846 finding (ListIdentity processed after oversize skip);
#         no panic. The T0846 proves the flow was NOT silenced by is_non_enip.

wirerust analyze enip_oversize_only.pcap --enip --json
# Expect: exit 0; no panic; zero T0846 findings; pdu_count = 0.
```

## Evaluation Rubric

- **Flow continuity after oversize skip** (weight: 0.50): Case A produces T0846 for the
  ListIdentity following the oversize frame. If T0846 is absent, is_non_enip was incorrectly
  set — HIGH severity regression.
- **No panic on oversize** (weight: 0.25): Both cases exit 0 without panic.
- **Oversize-only graceful handling** (weight: 0.25): Case B produces no crash, no T0846,
  and pdu_count = 0.

## Failure Guidance

"HOLDOUT FAIL: HS-118 — oversize frame incorrectly silences the flow. If Case A produces
no T0846 finding for the valid ListIdentity frame that follows an oversized frame, the
frame-walk loop set is_non_enip=true for the oversize event instead of only advancing the
cursor. Fix: the oversize-declared-frame path must increment parse_errors and malformed_in_window,
advance the cursor by min(total_frame_len, buf.len() - cursor), and then CONTINUE the loop
without setting is_non_enip. Only carry-buffer OVERFLOW (after stashing) should set
is_non_enip=true. See BC-2.17.016 Postcondition frame-skip path."
