---
document_type: holdout-scenario
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
waves: [35, 36, 37, 38, 39]
cycle: v0.6.0-dnp3
stories: [STORY-106, STORY-107, STORY-108, STORY-109, STORY-110]
feature_id: issue-008-dnp3-analyzer
github_issue: 8
mitre_version: ics-attack-19.1
confirmed_thresholds:
  direct_operate_threshold_default: 10
  detection_window_secs: 60
  block_cmd_timeout_secs: 10
  block_cmd_threshold: 3
  t0827_threshold: 3
  correlation_window_secs: 300
  malformed_anomaly_threshold: 3
---

# Waves 35–39 Holdout Scenarios: DNP3 TCP Analyzer (v0.6.0)

> **Purpose:** End-to-end integration holdout for the DNP3 TCP analyzer.
> Validates that crafted DNP3 frames and packet sequences produce the correct
> ICS threat findings with the exact MITRE v19.1 technique arrays (T1692.001,
> T1691.001, T0827, T0814, T0836), that existing analyzers are unaffected by the
> new port-20000 dispatch rule (Rule 6), and that all detection windows, thresholds,
> one-shot guards, and anomaly detectors fire correctly.
>
> **Evaluator note:** These scenarios are BLIND — the evaluator runs the finished
> implementation against the byte sequences or synthetic pcap descriptions below
> WITHOUT reading implementation source code. Pass/fail is determined solely by
> matching the EXACT EXPECTED OUTPUT specified in each scenario.
>
> **MITRE version discipline:** ALL technique IDs use ics-attack-19.1.
> T0855 is REVOKED — never appears. T0803 is REVOKED — never appears.
> T1692.001 replaces T0855. T1691.001 replaces T0803. T0827 is "Loss of Control"
> (Impact); T0828 is a different technique and must NOT appear in any finding.

---

## Per-Wave Gate Summary

| Wave | Story | Gate Criteria |
|------|-------|---------------|
| 35 | STORY-106 | Pure-core parse + classify + VP-023 Kani all four sub-properties |
| 36 | STORY-107 | Carry buffer + pending-requests bounds + master-addr cap |
| 37 | STORY-108 | Direct detections: T1692.001, T0814 (restart), T0836, co-emission, summarize |
| 38 | STORY-109 | Correlated/anomaly: T1691.001, T0827, broadcast, unsolicited, DISABLE, malformed |
| 39 | STORY-110 | End-to-end dispatch port-20000 + CLI threshold flag + VP-004 oracle |

---

## HS-W35-001: DL Header Parse — Canonical 10-Byte Minimum Vector

**Scope:** STORY-106 (BC-2.15.001, BC-2.15.003)
**Priority:** P0 (must-pass)
**Wave:** 35

**Input:** 10-byte slice:
```
05 64 05 C4 03 00 01 00 xx xx
```
Where `xx xx` are placeholder header-CRC bytes (not decoded as struct fields).
Fields: START1=0x05, START2=0x64, LENGTH=5, CONTROL=0xC4 (DIR=1, PRM=1, FCV=0, link-FC=4),
DEST=[0x03,0x00], SRC=[0x01,0x00].

**Operation:** `parse_dnp3_dl_header(&data[..10])`

**Assertions:**
1. Returns `Some(Dnp3DlHeader { start1: 0x05, start2: 0x64, length: 5, control: 0xC4, destination: 0x0003, source: 0x0001 })`.
2. `destination` decodes little-endian: bytes `[0x03, 0x00]` → `0x0003` (NOT `0x0300`).
3. `source` decodes little-endian: bytes `[0x01, 0x00]` → `0x0001`.
4. No panic.

---

## HS-W35-002: DL Header Parse — Extended Canonical Frame (BC-2.15.001 byte-level vector)

**Scope:** STORY-106 (BC-2.15.001, BC-2.15.003)
**Priority:** P0 (must-pass)
**Wave:** 35

**Input:** 10-byte slice:
```
05 64 0E C4 03 00 01 00 88 C5
```
Fields: START1=0x05, START2=0x64, LENGTH=14 (0x0E), CONTROL=0xC4, DEST=0x0003, SRC=0x0001.
(This is the canonical byte vector from BC-2.15.001 and BC-2.15.010.)

**Assertions:**
1. Returns `Some { start1: 0x05, start2: 0x64, length: 14, control: 0xC4, destination: 0x0003, source: 0x0001 }`.
2. `destination == 0x0003` (confirmed LE: `[0x03, 0x00]` → 3).
3. `source == 0x0001` (confirmed LE: `[0x01, 0x00]` → 1).
4. Bytes 8–9 (0x88, 0xC5) are NOT decoded as struct fields; no struct field for header CRC.

---

## HS-W35-003: DL Header Parse — Truncation Rejection and LE Disambiguation

**Scope:** STORY-106 (BC-2.15.002, BC-2.15.003)
**Priority:** P0 (must-pass)
**Wave:** 35

**Assertions:**
1. `parse_dnp3_dl_header(&[])` → `None` (zero-length; no panic).
2. `parse_dnp3_dl_header(&[0x05; 9])` → `None` (9 bytes, one short of minimum; no panic).
3. `parse_dnp3_dl_header(&[0x05; 10])` → `Some(...)` (exactly 10 bytes accepted).
4. **LE vs BE disambiguation**: bytes `[0x05, 0x64, 0x05, 0x00, 0x00, 0x01, 0xFD, 0xFF, 0x00, 0x00]`
   → `destination` = `u16::from_le_bytes([0x00, 0x01])` = 0x0100 (NOT 0x0001);
   `source` = `u16::from_le_bytes([0xFD, 0xFF])` = 0xFFFD.
   This vector distinguishes LE from BE: big-endian would give destination=0x0001, source=0xFFFD.
   The correct LE result for SRC bytes `[0xFD, 0xFF]` is 0xFFFD (broadcast-minus-2).
5. DEST bytes `[0xFF, 0xFF]` → `destination == 0xFFFF` (broadcast; u16 max).

---

## HS-W35-004: Three-Point Validity Gate — Biconditional Exhaustive

**Scope:** STORY-106 (BC-2.15.004)
**Priority:** P0 (must-pass)
**Wave:** 35

**Operation:** `is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool`

**Assertions (six vectors covering all partial-match failure modes):**
1. `{start1:0x05, start2:0x64, length:5, ...}` → `true` (all conditions met, LENGTH at minimum).
2. `{start1:0x05, start2:0x64, length:255, ...}` → `true` (all conditions met, LENGTH at max).
3. `{start1:0x04, start2:0x64, length:5, ...}` → `false` (wrong START1).
4. `{start1:0x05, start2:0x63, length:5, ...}` → `false` (wrong START2).
5. `{start1:0x05, start2:0x64, length:4, ...}` → `false` (LENGTH=4, below minimum 5).
6. `{start1:0x00, start2:0x00, length:0, ...}` → `false` (all conditions fail).

**Biconditional:** `is_valid` returns `true` IFF `start1==0x05 AND start2==0x64 AND length>=5`.
No partial-true states allowed.

---

## HS-W35-005: FC Classification — Totality and Set Membership

**Scope:** STORY-106 (BC-2.15.005, BC-2.15.006)
**Priority:** P0 (must-pass)
**Wave:** 35

**Operation:** `classify_dnp3_fc(fc: u8) -> Dnp3FcClass`

**Totality assertions:**
1. Never panics for any input in 0x00..=0xFF (no `unreachable!()` arms).
2. Returns exactly one of `{Read, Write, Control, Restart, Management, Response, Unknown}`.
3. FC=0xFF → `Unknown` (high byte, no special meaning in DNP3).
4. FC=0x80 → `Unknown` (undefined; not in any named set).

**Set membership assertions (exact FC → class mappings):**
5. FC=0x01 (READ) → `Read`.
6. FC=0x02 (WRITE) → `Write`.
7. FC=0x03 (SELECT) → `Control`.
8. FC=0x04 (OPERATE) → `Control`.
9. FC=0x05 (DIRECT_OPERATE) → `Control`.
10. FC=0x06 (DIRECT_OPERATE_NR) → `Control` (NOT Write, NOT Unknown — confirmed in BC-2.15.006 EC-007).
11. FC=0x0D (COLD_RESTART) → `Restart`.
12. FC=0x0E (WARM_RESTART) → `Restart`.
13. FC=0x0F (INITIALIZE_DATA) → `Management` (NOT Restart — confirmed exclusion in STORY-108 AC-005).
14. FC=0x07 (IMMED_FREEZE) → `Management`.
15. FC=0x81 (RESPONSE) → `Response`.
16. FC=0x82 (UNSOLICITED_RESPONSE) → `Response`.
17. FC=0x83 (AUTHENTICATE_RESP) → `Response`.

---

## HS-W35-006: compute_dnp3_frame_len — Formula Correctness at Boundaries

**Scope:** STORY-106 (BC-2.15.007)
**Priority:** P0 (must-pass)
**Wave:** 35

**Operation:** `compute_dnp3_frame_len(length: u8) -> Option<usize>`
**Formula:** `Some(5 + length + 2 * ceil((length - 5) / 16))` for length >= 5; `None` for length < 5.

**Assertions:**
1. `compute_dnp3_frame_len(0)` → `None` (below minimum 5).
2. `compute_dnp3_frame_len(4)` → `None` (below minimum 5).
3. `compute_dnp3_frame_len(5)` → `Some(10)` — U=0 blocks, 5+5+0=10.
4. `compute_dnp3_frame_len(6)` → `Some(13)` — U=1 user byte, ceil(1/16)=1 block, 5+6+2=13.
5. `compute_dnp3_frame_len(21)` → `Some(28)` — U=16 bytes, ceil(16/16)=1 block, 5+21+2=28.
6. `compute_dnp3_frame_len(22)` → `Some(31)` — U=17 bytes, ceil(17/16)=2 blocks, 5+22+4=31.
   (This is the block-boundary crossing: ≤16 user bytes → 1 CRC pair; 17+ → 2 CRC pairs.)
7. `compute_dnp3_frame_len(255)` → `Some(292)` — U=250, ceil(250/16)=16 blocks, 5+255+32=292.
8. Result always in range [10, 292] for any input in 5..=255. No overflow. No panic for all 256 u8 values.

---

## HS-W35-007: Transport FIR=1 Gating — Extract vs Skip

**Scope:** STORY-106 (BC-2.15.008)
**Priority:** P0 (must-pass)
**Wave:** 35

**Scenario A — FIR=1 fragment (transport_octet=0xC0):**
Deliver a 10-byte-minimum frame with transport octet 0xC0 (FIR=1, FIN=1, SEQ=0) and
application bytes `[0x81, 0x05, ...]` (App Control=0x81, App FC=0x05 DIRECT_OPERATE).

**Assertions A:**
1. `transport_is_fir(0xC0)` → `true` (bit 6 set: `0xC0 & 0x40 = 0x40 != 0`).
2. App FC 0x05 IS extracted from payload position after transport octet.
3. `classify_dnp3_fc(0x05)` called; `fn_code_counts[0x05]` incremented.
4. `frame_count` incremented.

**Scenario B — FIR=0 continuation (transport_octet=0x80):**
Deliver the same frame but with transport octet 0x80 (FIR=0, FIN=1, SEQ=0).

**Assertions B:**
5. `transport_is_fir(0x80)` → `false` (bit 6 clear: `0x80 & 0x40 = 0`).
6. No App FC extraction. `fn_code_counts` unchanged.
7. `frame_count` still incremented (continuation frame counts as a frame).
8. No finding emitted (no detection branches entered without FIR=1).

---

## HS-W35-008: Desync Bail — Non-DNP3 Traffic Silenced

**Scope:** STORY-106 (BC-2.15.009)
**Priority:** P0 (must-pass)
**Wave:** 35

**Scenario:** Deliver the following as the first segment to a fresh `Dnp3FlowState`:
```
FF FE AB CD EF 01 02 03 04 05 06 07 08 09 0A 0B
```
(16 bytes; no `0x05 0x64` sync word at offset 0.)

**Assertions:**
1. After first `on_data` call: `flow.is_non_dnp3 == true`.
2. Second `on_data` call with valid DNP3 bytes: immediate no-op; `flow.is_non_dnp3` remains `true` (latch is one-way).
3. `flow.carry.len()` does NOT grow after latch is set.
4. No findings emitted for any subsequent segment.
5. `flow.frame_count == 0` (no frames processed on a bailed flow).

**Negative (valid sync at offset 0):** First 2 bytes are `0x05 0x64` → `is_non_dnp3` remains `false`; normal processing continues.

---

## HS-W36-001: Carry Buffer — Accumulate and Cap at 292

**Scope:** STORY-107 (BC-2.15.016)
**Priority:** P0 (must-pass)
**Wave:** 36

**Setup:** Create a fresh `Dnp3FlowState`. Pre-load `flow.carry` with 290 bytes of arbitrary
data (simulating a partial large frame). Then call `on_data` with 5 additional bytes.

**Assertions:**
1. After `on_data`: `flow.carry.len() == 292` (capped at MAX_DNP3_FRAME_LEN).
2. 3 bytes were discarded (290 + 5 = 295 > 292; only 2 accepted, 3 dropped).
3. `flow.parse_errors == 1` (one overflow event incremented the lifetime counter).
4. No panic. No carry growth beyond 292.

**Carry-frame-consumption scenario:**
5. Pre-load carry with exactly one complete 10-byte frame (LENGTH=5, which gives
   `compute_dnp3_frame_len(5) = 10`) plus 3 trailing bytes of the next partial frame.
   After the frame-consume loop: `flow.carry.len() == 3` (frame consumed; 3 bytes remain).
   `flow.frame_count == 1`.

---

## HS-W36-002: Pending-Requests — Bounded at 256 with Oldest-Eviction

**Scope:** STORY-107 (BC-2.15.016 postconditions 8/9/10)
**Priority:** P0 (must-pass)
**Wave:** 36

**Setup:** Insert 256 Control-class request entries into `flow.pending_requests` with keys
`((dest_addr=N, app_seq=0), request_ts=N)` for N=0..=255 (unique keys, timestamps 0 through 255).

**Assertions:**
1. After 256 inserts: `flow.pending_requests.len() == 256`.
2. Insert a 257th entry: key=(dest_addr=300, app_seq=0), request_ts=500.
3. After 257th insert: `flow.pending_requests.len() == 256` (cap maintained).
4. The entry with the smallest `request_ts` (ts=0, key=(0,0)) has been evicted.
5. The 257th entry (ts=500) IS present in the map.
6. All 255 remaining original entries (ts=1..=255) are still present.
7. The evicted entry generates NO T1691.001 finding (eviction is NOT a block-timeout event).

**DIRECT_OPERATE_NR exclusion:**
8. FC=0x06 (DIRECT_OPERATE_NR) does NOT insert into `pending_requests`. After sending
   FC=0x06 to a flow with an empty `pending_requests`, the map remains empty.

---

## HS-W37-001: T1692.001 — Direct-Operate Burst at Threshold Boundary

**Scope:** STORY-108 (BC-2.15.010)
**Priority:** P0 (must-pass)
**Wave:** 37

**Frame construction:** Each frame is a minimal well-formed DNP3 DIRECT_OPERATE:
```
Link:      05 64 0E C4 03 00 01 00 [hdr-crc]
Transport: C0  (FIR=1, FIN=1, SEQ=0)
App Ctrl:  81
App FC:    05  (DIRECT_OPERATE)
```
All 11 frames share the same flow (src=0x0001, dest=0x0003), delivered within 60 seconds
(timestamps t=0s through t=10s, e.g., 1 frame per second for first 10, then frame 11 at t=59s).

**Assertions:**
1. After 10 Control FCs (count=10, threshold=10): `flow.direct_operate_count == 10`. NO finding emitted (`10 > 10` is `false`).
2. After 11th Control FC (count=11): exactly ONE `Finding` pushed.
3. `finding.mitre_techniques == vec!["T1692.001"]`.
4. `finding.verdict == Verdict::Likely`.
5. `finding.confidence == Confidence::Medium`.
6. `finding.summary` contains `"threshold 10"` (echoed threshold).
7. `flow.direct_operate_emitted == true`.

**One-shot guard:**
8. Deliver 5 additional Control FCs (total 16). No additional T1692.001 finding.
9. `self.all_findings.len() == 1` throughout frames 12–16.
10. `flow.direct_operate_count == 16` (counter keeps incrementing past guard).

---

## HS-W37-002: T1692.001 — Unexpected Source Fires at Count=1

**Scope:** STORY-108 (BC-2.15.010 Invariant 5)
**Priority:** P0 (must-pass)
**Wave:** 37

**Scenario:** A Control-class FC arrives from source address 0x0099 which is NOT in the
expected/allowlisted master-address set for this flow. Count=1 (first occurrence). Default
threshold=10.

**Assertion:** The unexpected-source check fires independently of the burst threshold.
At count=1, a T1692.001 finding IS emitted (source-address check is the primary gate;
10/60s is the secondary volumetric gate).

**Note for evaluator:** If the implementation gates the unexpected-source check behind the
same `direct_operate_count > threshold` condition as the burst check, this test will fail.
The two checks are independent per BC-2.15.010 Invariant 5.

---

## HS-W37-003: T0814 — COLD_RESTART and WARM_RESTART Per-Occurrence (No Threshold)

**Scope:** STORY-108 (BC-2.15.011)
**Priority:** P0 (must-pass)
**Wave:** 37

**Scenario A — COLD_RESTART (FC=0x0D):**
Frame: `05 64 0E C4 03 00 01 00 [crc] C0 81 0D [objects] [crc]`
(Transport=0xC0 FIR=1, App Ctrl=0x81, App FC=0x0D COLD_RESTART)

**Assertions A:**
1. ONE `Finding` pushed: `mitre_techniques == vec!["T0814"]`.
2. `finding.verdict == Verdict::Likely`.
3. `finding.confidence == Confidence::High`.
4. `finding.summary` contains `"FC 0x0D"` and `"COLD_RESTART"` and src/dest addresses.
5. `flow.restart_event_count == 1` (incremented unconditionally, even when capped).

**Scenario B — WARM_RESTART (FC=0x0E):**
Same structure, App FC=0x0E.
6. Additional ONE `Finding` pushed: `mitre_techniques == vec!["T0814"]`.
7. `flow.restart_event_count == 2`.

**Scenario C — Two COLD_RESTARTs same flow:**
8. Two T0814 findings total (per-occurrence, no one-shot guard for restarts).

**FC=0x0F exclusion:**
9. Frame with App FC=0x0F (INITIALIZE_DATA): `classify_dnp3_fc(0x0F) == Management`.
   NO T0814 finding. `flow.restart_event_count` unchanged.

---

## HS-W37-004: T0836 — WRITE Per-Occurrence; NOT Also T1692.001

**Scope:** STORY-108 (BC-2.15.012)
**Priority:** P0 (must-pass)
**Wave:** 37

**Frame:** `05 64 0E C4 03 00 01 00 [crc] C0 81 02 [objects] [crc]`
(App FC=0x02 WRITE, FIR=1)

**Assertions:**
1. ONE `Finding` pushed: `mitre_techniques == vec!["T0836"]`.
2. `finding.verdict == Verdict::Likely`.
3. `finding.confidence == Confidence::Medium`.
4. `finding.summary` contains `"WRITE"` and src/dest addresses.
5. T1692.001 is NOT in the techniques array (WRITE is Write-class, NOT Control-class; never co-tagged).
6. `flow.direct_operate_count` is NOT incremented by a WRITE FC.

**Repeat for a second WRITE FC on the same flow:**
7. Second `Finding` pushed (per-occurrence, no one-shot guard for WRITE).
8. Two total findings in `all_findings`, both `["T0836"]`.

---

## HS-W37-005: Co-Emission Ordering — Direct Finding Before Derived T0827

**Scope:** STORY-108 (BC-2.15.013)
**Priority:** P0 (must-pass)
**Wave:** 37

**Setup:** Pre-accumulate state: `flow.restart_event_count=2`, `flow.block_event_count=0`,
`flow.loss_of_control_emitted=false` (within 300s window).
Deliver one COLD_RESTART frame (FC=0x0D). This 3rd restart/block event crosses T0827_THRESHOLD=3.

**Assertions:**
1. Two findings are pushed in `all_findings` during this single `on_data` call.
2. `all_findings[i].mitre_techniques == vec!["T0814"]` — T0814 direct finding is FIRST.
3. `all_findings[i+1].mitre_techniques == vec!["T0827"]` — T0827 derived finding is SECOND.
4. T0827 finding is NOT a separate top-level finding from a different code path — it is emitted within the SAME `on_data` call, after T0814.
5. `flow.restart_event_count == 3` (incremented to 3, triggering T0827).
6. `flow.loss_of_control_emitted == true`.

**MAX_FINDINGS boundary case:**
Pre-fill `self.all_findings` to `MAX_FINDINGS - 1`. Deliver the same triggering COLD_RESTART.
7. `all_findings.len() == MAX_FINDINGS` (T0814 pushed, cap consumed).
8. T0827 NOT pushed (cap hit before second push). `all_findings.len()` does not exceed MAX_FINDINGS.
9. `flow.restart_event_count` still incremented to 3 (counter not gated by cap).

---

## HS-W37-006: summarize() — Function-Code Distribution and Zero-Flow Case

**Scope:** STORY-108 (BC-2.15.020)
**Priority:** P0 (must-pass)
**Wave:** 37

**Setup:** Process: 5 frames with App FC=0x05 (DIRECT_OPERATE), 3 frames with App FC=0x01 (READ), 1 frame with App FC=0x0D (COLD_RESTART). All FIR=1.

**Assertions:**
1. `summarize()` returns output containing `function_code_distribution`.
2. `fn_code_counts[0x05] == 5` (five DIRECT_OPERATE).
3. `fn_code_counts[0x01] == 3` (three READ).
4. `fn_code_counts[0x0D] == 1` (one COLD_RESTART).
5. FC bytes with count=0 do NOT appear in the distribution (non-sparse output).
6. `total_frames` reflects all complete frames processed.
7. `flows_analyzed` reflects the number of distinct flows processed.
8. ~~`findings_emitted` equals `self.all_findings.len()`.~~ **REMOVED (F-F5-006):** BC-2.15.020
   does NOT require a `findings_emitted` key in `summarize()` output. The authoritative
   summarize() output fields are: `function_code_distribution`, `control_operation_counts`,
   `total_frames`, `total_parse_errors`, `flows_analyzed`. Adding `findings_emitted` would
   require a BC-2.15.020 amendment AND an implementation change — that is larger scope and
   not in scope for this cycle. Assertion 8 is dropped to align holdout to BC-2.15.020 as
   specified. The implementation is BC-correct; this holdout was over-specifying.
   (F-F5-006: aligned to BC-2.15.020; findings_emitted not a BC-required summarize field)

**Zero-flow case:**
9. A fresh `Dnp3Analyzer` with no flows analyzed: `summarize()` returns output with zero counts (not absent/None).

---

## HS-W38-001: T1691.001 — Block-Command 3-of-300s Threshold

**Scope:** STORY-109 (BC-2.15.014)
**Priority:** P0 (must-pass)
**Wave:** 38

**Setup:** On a single flow, send three SELECT (FC=0x03) requests without responses,
each with a distinct `(dest_addr, app_seq)` correlation key. Each request times out after
`BLOCK_CMD_TIMEOUT_SECS = 10s` with no FC=0x81 RESPONSE arriving.

**Timestamps:** request 1 at t=0s (timeout at t=10s), request 2 at t=20s (timeout at t=30s),
request 3 at t=40s (timeout at t=50s). All within CORRELATION_WINDOW_SECS=300s.

**Assertions:**
1. `flow.block_event_count == 3` after all three timeouts.
2. Exactly ONE `Finding` pushed when `block_event_count` reaches 3.
3. `finding.mitre_techniques == vec!["T1691.001"]`.
4. `finding.verdict == Verdict::Possible`.
5. `finding.confidence == Confidence::Low`.
6. `flow.block_finding_emitted_this_window == true`.
7. A 4th block-timeout in the same 300s window: `block_event_count == 4`, NO additional T1691.001 finding.

**DIRECT_OPERATE_NR exclusion:**
8. FC=0x06 request times out (no response): `flow.block_event_count` is NOT incremented.
   FC=0x06 intentionally expects no response; this is not a block event.

---

## HS-W38-002: T1691.001 — Block Events Not Reset at 120s (Trace B Regression)

**Scope:** STORY-109 (BC-2.15.014, BC-2.15.015 invariant — single 300s window)
**Priority:** P0 (must-pass)
**Wave:** 38

**Background:** A previous design had a 120s sub-window for T1691.001. The v1.2 redesign
collapsed to a single 300s CORRELATION_WINDOW_SECS. This test verifies the old behavior
is NOT present.

**Scenario:**
- Block event 1 at t=0s (SELECT request, no response within 10s → timeout at t=10s).
- Block event 2 at t=150s (SELECT request, no response within 10s → timeout at t=160s).
- No window expiry at t=120s (the 300s window does NOT reset at 120s).

**Assertions:**
1. At t=160s (after 2nd timeout): `flow.block_event_count == 2`.
2. The count was NOT reset to 0 at t=120s.
3. Block event 3 at t=200s (timeout at t=210s): `block_event_count == 3`; T1691.001 emitted.
4. T0827 threshold check: `restart_event_count + block_event_count` = 0 + 3 = 3 >= T0827_THRESHOLD.
   T0827 IS emitted after the T1691.001 (co-emission: T1691.001 first, T0827 second per BC-2.15.013).

---

## HS-W38-003: T0827 — Combined Restart + Block Accumulation (Trace B)

**Scope:** STORY-109 (BC-2.15.015)
**Priority:** P0 (must-pass)
**Wave:** 38

**Canonical Trace B from BC-2.15.015:**
- Block event 1 at t=0s (timeout at t=10s): `block_event_count=1`.
- Block event 2 at t=150s (timeout at t=160s): `block_event_count=2`.
  (Key: the 300s window did NOT reset at t=120s.)
- COLD_RESTART at t=200s: `restart_event_count=1`.
  Combined: 2 + 1 = 3 >= T0827_THRESHOLD.

**Assertions:**
1. T0814 emitted for the COLD_RESTART (per BC-2.15.011; `["T0814"]`, Likely/High).
2. T0827 emitted AFTER T0814 in the SAME `on_data` call.
3. `finding.mitre_techniques == vec!["T0827"]`.
4. `finding.category` reflects tactic `IcsImpact` (new `MitreTactic::IcsImpact` variant).
5. `flow.loss_of_control_emitted == true`.
6. One-shot guard: a 4th restart/block event in the same 300s window does NOT emit a second T0827.

**T0827 must NOT fire from a single event:**
7. Flow with only `restart_event_count=1, block_event_count=0`: T0827 NOT emitted
   (1 < T0827_THRESHOLD=3).
8. Flow with only `restart_event_count=2, block_event_count=0`: T0827 NOT emitted
   (2 < 3).

---

## HS-W38-004: Correlation Window — Six-Field Expiry Reset

**Scope:** STORY-109 (BC-2.15.015 postcondition 3)
**Priority:** P0 (must-pass)
**Wave:** 38

**Setup:** Pre-accumulate on a single flow within window 1 (starting at t=0s):
- `restart_event_count=2`
- `block_event_count=2`
- `block_finding_emitted_this_window=true`
- `loss_of_control_emitted=true`
- `malformed_in_window=2`
- `malformed_anomaly_emitted=false`
- `parse_errors=5` (lifetime counter)

Advance timestamp to t=300s (window expiry: `now_ts.wrapping_sub(correlation_window_start_ts) >= 300`).
Deliver any frame to trigger the expiry handler.

**Assertions (all six windowed fields reset; lifetime counter preserved):**
1. `flow.restart_event_count == 0` (reset).
2. `flow.block_event_count == 0` (reset).
3. `flow.block_finding_emitted_this_window == false` (reset).
4. `flow.loss_of_control_emitted == false` (reset).
5. `flow.malformed_in_window == 0` (reset).
6. `flow.malformed_anomaly_emitted == false` (reset).
7. `flow.parse_errors == 5` (NOT reset — lifetime monotonic counter).
8. `flow.correlation_window_start_ts` updated to now_ts (new window started).

---

## HS-W38-005: Broadcast Control Anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF

**Scope:** STORY-109 (BC-2.15.018)
**Priority:** P0 (must-pass)
**Wave:** 38

**Scenario A — Broadcast DIRECT_OPERATE (DEST=0xFFFF):**
Frame: `05 64 0E C4 FF FF 01 00 [crc] C0 81 05 [objects] [crc]`
DEST bytes `[0xFF, 0xFF]` → `destination=0xFFFF`. SRC=0x0001. App FC=0x05 (DIRECT_OPERATE, Control-class).

**Assertions A:**
1. ONE `Finding` pushed: `mitre_techniques == vec!["T1692.001"]`.
2. `finding.verdict == Verdict::Possible`.
3. `finding.confidence == Confidence::Medium`.
4. `flow.direct_operate_count` incremented by 1 (broadcast Control still feeds the burst threshold).

**Broadcast + burst both retained:**
5. Deliver 11 broadcast DIRECT_OPERATE frames (DEST=0xFFFF) within 60s. After frame 11:
   `all_findings` contains MORE THAN ONE `T1692.001` finding:
   - The per-occurrence broadcast anomaly findings (up to MAX_FINDINGS).
   - At count=11: the BC-2.15.010 burst-threshold finding ALSO fires.
   Both are retained; no deduplication on technique ID alone.

**Broadcast DEST=0xFFFD and 0xFFFE:**
6. DEST=0xFFFD with Control FC: anomaly finding fired (0xFFFD >= 0xFFFD, in broadcast range).
7. DEST=0xFFFE with Control FC: anomaly finding fired.

**Broadcast READ — negative case:**
8. DEST=0xFFFF with FC=0x01 (READ, NOT Control-class): NO broadcast anomaly finding.
   `direct_operate_count` NOT incremented.

---

## HS-W38-006: Unsolicited Response Anomaly — UNS Bit / FC=0x82 Without Prior ENABLE

**Scope:** STORY-109 (BC-2.15.019)
**Priority:** P1 (nice-to-have)
**Wave:** 38

**Scenario A — Unsolicited with no prior ENABLE_UNSOLICITED (0x14):**
Frame with App FC=0x82 (UNSOLICITED_RESPONSE), App Control UNS bit set (0x10),
on a fresh flow with `flow.enable_unsolicited_seen==false` and `flow.response_seen==false`.

**Assertions A:**
1. ONE `Finding` pushed: `mitre_techniques == vec!["T0814"]`.
2. `finding.verdict == Verdict::Possible`.
3. `finding.confidence == Confidence::Low`.
4. `flow.unsolicited_anomaly_emitted == true` (one-shot guard set).
5. Second FC=0x82 on same flow: NO additional anomaly finding (one-shot guard).

**Scenario B — ENABLE_UNSOLICITED suppresses anomaly:**
Deliver FC=0x14 (ENABLE_UNSOLICITED) first → `flow.enable_unsolicited_seen = true`.
Then deliver FC=0x82:
6. NO unsolicited anomaly finding (ENABLE was seen; this is expected behavior).

---

## HS-W38-007: DISABLE_UNSOLICITED T0814 (Likely/Medium) and ENABLE T0814 (Possible/Low)

**Scope:** STORY-109 (BC-2.15.023)
**Priority:** P0 (must-pass)
**Wave:** 38

**Scenario A — DISABLE_UNSOLICITED (FC=0x15):**
Frame with App FC=0x15 on FIR=1 fragment. Detection via raw FC byte check (NOT via classify_dnp3_fc).

**Assertions A:**
1. ONE `Finding` pushed: `mitre_techniques == vec!["T0814"]`.
2. `finding.verdict == Verdict::Likely`.
3. `finding.confidence == Confidence::Medium`.
4. `finding.summary` contains `"DISABLE_UNSOLICITED"` and `"alarm suppression"` (or equivalent alarm-blinding context).
5. Per-occurrence: deliver a second FC=0x15. A second T0814 finding IS emitted (no one-shot guard for DISABLE).

**Scenario B — ENABLE_UNSOLICITED (FC=0x14):**
Frame with App FC=0x14 on FIR=1.
6. ONE `Finding` pushed: `mitre_techniques == vec!["T0814"]`.
7. `finding.verdict == Verdict::Possible`.
8. `finding.confidence == Confidence::Low`.
9. Per-occurrence (no one-shot guard).

**Severity split confirmation:**
10. DISABLE (0x15) is Likely/Medium; ENABLE (0x14) is Possible/Low. The two must NOT be equal in verdict/confidence.

---

## HS-W38-008: Malformed-Frame Anomaly — 3-of-300s Crain-Sistrunk-Style Threshold

**Scope:** STORY-109 (BC-2.15.024)
**Priority:** P0 (must-pass)
**Wave:** 38

**Scenario: Three structurally-malformed frames within 300s.**
A "malformed frame" is any input that triggers a structural reject path:
- LENGTH < 5 (validity gate reject)
- Frame-length mismatch (computed `frame_len` != bytes available after LENGTH byte decoding)
- Carry overflow (bytes beyond MAX_DNP3_FRAME_LEN=292 discarded)

Deliver three such frames within a 300s window on the same flow.

**Assertions:**
1. `flow.parse_errors == 3` after 3 malformed frames (lifetime counter).
2. `flow.malformed_in_window == 3` after 3 malformed frames (windowed counter).
3. ONE `Finding` pushed when `malformed_in_window` reaches 3: `mitre_techniques == vec!["T0814"]`.
4. `finding.verdict == Verdict::Possible`.
5. `finding.confidence == Confidence::Low`.
6. `finding.summary` contains "malformed" or "structural" and references Crain-Sistrunk or crash-probe context.
7. `flow.malformed_anomaly_emitted == true` (one-shot guard set).
8. A 4th malformed frame in the same window: `parse_errors=4`, `malformed_in_window=4`, NO second T0814 (one-shot guard).

**Crain-Sistrunk-style frame (structurally malformed, valid-CRC-style probe):**
A frame with `LENGTH` field set to a value that causes the computed `frame_len` to exceed the
remaining bytes in the carry buffer — a structural length mismatch indicative of a crash probe.
This frame MUST be caught by the malformed-frame detection path.

**parse_errors is lifetime (never reset):**
9. After the 300s window expiry: `flow.parse_errors` remains at its accumulated value (e.g., 3).
   `flow.malformed_in_window` resets to 0 at window expiry.

---

## HS-W38-009: Negative / False-Positive Guard — Legitimate Low-Rate Control

**Scope:** STORY-108 + STORY-109 (guard for T1692.001 and T1691.001)
**Priority:** P0 (must-pass)
**Wave:** 38

**Scenario A — Legitimate below-threshold control (no T1692.001):**
Deliver 10 Control-class FCs (SELECT or DIRECT_OPERATE) within 60s on a single flow.
Default threshold=10. `10 > 10` is false.

**Assertion:** NO T1692.001 finding. `flow.direct_operate_count == 10`. `all_findings` is empty.

**Scenario B — Normal read polling (no findings):**
Deliver 50 READ (FC=0x01) frames within 60s on a single flow.

**Assertion:** NO findings of any kind. `fn_code_counts[0x01] == 50`. No T1692.001, T0836, T0814, T1691.001, T0827.

**Scenario C — Single packet-loss timeout (no T1691.001):**
One SELECT (FC=0x03) request times out with no response (block_event_count=1). No further
block timeouts. BLOCK_CMD_THRESHOLD=3.

**Assertion:** `flow.block_event_count == 1`. NO T1691.001 finding (1 < 3 threshold).

**Scenario D — Two block timeouts only (no T0827):**
Two SELECT requests time out. `block_event_count=2, restart_event_count=0`. Combined=2 < T0827_THRESHOLD=3.

**Assertion:** NO T0827 finding. T1691.001 also NOT emitted (2 < BLOCK_CMD_THRESHOLD=3).

---

## HS-W39-001: Dispatcher — Port-20000 Routes to Dnp3Analyzer (Rule 6)

**Scope:** STORY-110 (BC-2.15.021)
**Priority:** P0 (must-pass)
**Wave:** 39

**Assertions:**
1. `StreamDispatcher::classify(ports={20000}, data=[arbitrary non-TLS non-HTTP bytes])` → `DispatchTarget::Dnp3`.
2. Rule 6 fires ONLY when Rules 1–5 all returned false.
3. `DispatchTarget::None` is now Rule 7 (not Rule 6 as before DNP3 integration).

**Rule ordering: port 502 and 20000 conflict:**
4. `classify(ports={502, 20000}, ...)` → `DispatchTarget::Modbus` (Rule 5 before Rule 6).
5. `classify(ports={502}, ...)` → `DispatchTarget::Modbus` (unchanged).
6. `classify(ports={12345}, ...)` → `DispatchTarget::None` (Rule 7).

---

## HS-W39-002: Content-First Precedence — TLS/HTTP on Port 20000 Not Stolen

**Scope:** STORY-110 (BC-2.15.021 postconditions 5/6; VP-004)
**Priority:** P0 (must-pass)
**Wave:** 39

**Assertions:**
1. `classify(ports={20000}, data=[0x16, 0x03, 0x01, ...])` (TLS ClientHello prefix) → `DispatchTarget::Tls` (Rule 1 wins; NOT Dnp3).
2. `classify(ports={20000}, data=[0x47, 0x45, 0x54, 0x20, ...])` (HTTP `GET ` prefix) → `DispatchTarget::Http` (Rule 2 wins; NOT Dnp3).
3. Rule 6 (port 20000) is NEVER reached for content-matched flows.
4. `classify(ports={443}, ...)` → `DispatchTarget::Tls` (Rule 3; unaffected by new Rule 6).
5. `classify(ports={80}, ...)` → `DispatchTarget::Http` (Rule 4; unaffected by new Rule 6).
6. A port-20000 flow carrying raw DNP3 sync bytes `[0x05, 0x64, ...]` (no TLS/HTTP prefix) → `DispatchTarget::Dnp3` (Rule 6 fires correctly for non-TLS/HTTP binary content).

---

## HS-W39-003: Non-DNP3 Traffic on Port 20000 — is_non_dnp3 Bail, No False Findings

**Scope:** STORY-110 (BC-2.15.021, BC-2.15.009 interaction)
**Priority:** P0 (must-pass)
**Wave:** 39

**Scenario:** A port-20000 TCP flow dispatched to `Dnp3Analyzer`. The first 16 bytes contain
no `[0x05, 0x64]` sync word at offset 0 (e.g., a non-DNP3 binary protocol or garbage data).

**Assertions:**
1. `flow.is_non_dnp3` is set to `true` after the desync-bail check.
2. All subsequent `on_data` calls for this flow are no-ops.
3. No findings are emitted for this flow.
4. The dispatcher does NOT crash. The `Dnp3Analyzer` continues to handle other flows normally.
5. `summarize()` for this flow shows `frame_count=0`, `parse_errors=0` (no frames processed).

---

## HS-W39-004: --dnp3-direct-operate-threshold CLI Flag — Override Changes Firing Point

**Scope:** STORY-110 (BC-2.15.017)
**Priority:** P0 (must-pass)
**Wave:** 39

**Test A — Lower threshold (--dnp3-direct-operate-threshold 3):**
Run with threshold=3. Deliver 4 Control-class FCs within 60s.

**Assertions A:**
1. T1692.001 finding emitted at 4th FC (count=4 > threshold=3).
2. Finding summary contains `"threshold 3"`.
3. No finding at 3 FCs (count=3 > 3 is false).

**Test B — Threshold 0 (fires immediately):**
`--dnp3-direct-operate-threshold 0`. Deliver 1 Control FC. Count=1 > 0 = true.
4. T1692.001 fires on the very first Control FC. Finding summary contains `"threshold 0"`.

**Test C — Default (threshold=10 when flag omitted):**
Invocation without `--dnp3-direct-operate-threshold`. Deliver 10 Control FCs.
5. No finding at count=10. Finding summary (when fired at 11) contains `"threshold 10"`.

---

## HS-W39-005: End-to-End — Crafted DNP3 Synthetic PCAP with Full Detection Surface

**Scope:** STORY-110 (dispatcher integration + CLI wiring + all detections)
**Priority:** P0 (must-pass)
**Wave:** 39

**Setup:** Craft a synthetic DNP3 TCP pcap (port 20000) or deliver via the CLI `analyze`
subcommand with `--dnp3` flag. The capture contains:

1. 11 DIRECT_OPERATE (FC=0x05) frames within 60s on flow A → triggers T1692.001 (count=11 > 10).
2. 1 COLD_RESTART (FC=0x0D) frame on flow A → triggers T0814 per-occurrence.
3. 1 WRITE (FC=0x02) frame on flow B → triggers T0836 per-occurrence.
4. 3 SELECT (FC=0x03) requests on flow C, each timing out after 10s without response → triggers T1691.001 (block_count=3).
5. 1 DISABLE_UNSOLICITED (FC=0x15) on flow D → triggers T0814 Likely/Medium.
6. 3 malformed frames (LENGTH<5 or frame-length mismatch) on flow E within 300s → triggers T0814 Possible/Low (malformed anomaly).
7. 1 broadcast DIRECT_OPERATE (DEST=0xFFFF, FC=0x05) on flow F → triggers T1692.001 broadcast anomaly.

Run: `wirerust analyze <dnp3.pcap> --dnp3 --format json`

**Assertions:**
1. Process exits 0.
2. JSON output has non-empty `"findings"` array.
3. At least one finding with `mitre_techniques` containing `"T1692.001"` (Control burst and/or broadcast anomaly).
4. At least one finding with `mitre_techniques` containing `"T0814"` (COLD_RESTART, DISABLE_UNSOLICITED, or malformed anomaly).
5. At least one finding with `mitre_techniques` containing `"T0836"` (WRITE).
6. At least one finding with `mitre_techniques` containing `"T1691.001"` (block-command).
7. `"analyzers"` output includes a DNP3 summary entry with `total_frames > 0`.
8. No crash. No stack overflow. No panic message in stderr.
9. T0855 NEVER appears in any finding's `mitre_techniques` (revoked technique).
10. T0803 NEVER appears in any finding's `mitre_techniques` (revoked technique).
11. T0828 NEVER appears in any finding's `mitre_techniques` (wrong technique; "Loss of Control" is T0827).

---

## HS-W39-006: Regression on Existing Analyzers After Waves 35–39

**Scope:** All existing analyzers (HTTP, TLS, DNS, Modbus, Reassembly, Reporters)
**Priority:** P0 (must-pass)
**Wave:** 39

**Assertions:**
1. `cargo test --all-targets` exits 0 after all 5 DNP3 stories are merged.
2. Existing VP assertions pass: VP-004 (classify-oracle, extended to Rule 6/Dnp3),
   VP-007 (catalog-drift-guard: SEEDED_TECHNIQUE_ID_COUNT=23, EMITTED_IDS.len()=15, catalogue-only derived 23−15=8),
   VP-022 (Modbus Kani proofs unchanged), VP-023 (DNP3 Kani proofs: all four sub-properties).
3. No test function that was green pre-Wave-35 has turned red.
4. `cargo clippy --all-targets -- -D warnings` clean.
5. `cargo fmt --check` clean.
6. The VP-007 catalog counts hold: `SEEDED_TECHNIQUE_ID_COUNT == 23`,
   `EMITTED_IDS.len() == 15` (kani_proofs-local const, via the mitre drift-guard test),
   catalogue-only derived as `SEEDED_TECHNIQUE_ID_COUNT - EMITTED_IDS.len() == 8`
   (no named `CATALOGUE_ONLY_TECHNIQUE_IDS` const exists).
7. T1691.001 and T0827 appear in BOTH the SEEDED and EMITTED sets (added in STORY-109).
8. MitreTactic::IcsImpact variant compiles successfully (new enum variant from STORY-109).

---

## HS-W39-007: VP-023 Kani Four Sub-Properties — All Pass

**Scope:** STORY-106 (VP-023 formal verification)
**Priority:** P0 (must-pass)
**Wave:** 39

**Operations (requires nightly toolchain via cargo-kani):**
```
cargo kani --harness verify_parse_dnp3_dl_header_safety
cargo kani --harness verify_classify_dnp3_fc_total
cargo kani --harness verify_is_valid_dnp3_frame_gate
cargo kani --harness verify_compute_dnp3_frame_len
```

**Assertions:**
1. All four harnesses report `VERIFICATION:- SUCCESSFUL`.
2. Sub-A (`verify_parse_dnp3_dl_header_safety`): parse returns `None` for len<10, `Some` for len>=10; LE decode correct for all symbolic inputs.
3. Sub-B (`verify_classify_dnp3_fc_total`): totality for all 256 FC values; set-membership assertions for Control/Restart/Write/Read/Response/Management.
4. Sub-C (`verify_is_valid_dnp3_frame_gate`): biconditional for all symbolic `Dnp3DlHeader` inputs.
5. Sub-D (`verify_compute_dnp3_frame_len`): formula correctness, [10,292] bound, no overflow for all 256 `u8` inputs.
6. No harness panics. No harness takes more than 60s each.

---

## Waves 35–39 Release Gate (v0.6.0)

Before creating the v0.6.0 release tag:

1. All HS-W35-001 through HS-W39-007 pass.
2. `cargo test --all-targets` exits 0 on a clean checkout.
3. `cargo clippy --all-targets -- -D warnings` clean.
4. `cargo fmt --check` clean.
5. VP-023 Kani proofs verified (nightly): all four sub-properties SUCCESSFUL.
6. VP-004 Kani oracle updated: `classify_oracle` includes port-20000 → Dnp3 arm.
   `verify_content_first_precedence_exhaustive` reports SUCCESSFUL.
7. `wirerust analyze --help` shows `--dnp3` and `--dnp3-direct-operate-threshold` flags
   with correct default value (10).
8. VP-007 catalog counts confirmed: `SEEDED_TECHNIQUE_ID_COUNT==23`, `EMITTED_IDS.len()==15`,
   catalogue-only derived as `23−15==8` (no named `CATALOGUE_ONLY_TECHNIQUE_IDS` const).
9. Verify v0.5.0 is already tagged (v0.6.0 must be a forward increment from v0.5.0).
10. Verify T0855 and T0803 do NOT appear anywhere in `src/mitre.rs` EMITTED set
    (revoked techniques; v19.1 compliance).
