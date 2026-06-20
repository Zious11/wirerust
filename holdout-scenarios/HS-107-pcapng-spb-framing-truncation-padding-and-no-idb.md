---
document_type: holdout-scenario
level: ops
version: "1.3"  # Pass-4 R4 / ADR-009 rev 7 Decision 20: added Case F — btl=12 (aligned, crate frames, body=0 < 4 SPB fixed-field original_len) → E-INP-008 (body-too-short). Distinct from Case E misaligned/btl<12 → E-INP-010 (crate framing failure). Prior: v1.2 P3-re-audit FINDING-P3-003/004: added VP-031, restated Case B three-way min.
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.013.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-107"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.013
verification_properties:
  - VP-028
  - VP-031
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng SPB Framing — Truncation, Padding Strip, No-IDB Guard, Minimum-Length Rejection, and Body-Too-Short (E-INP-008 vs E-INP-010 Split)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The Simple Packet Block (SPB, block type `0x00000003`) is the only packet-bearing block type
in the pcapng spec that carries NO per-packet timestamp and NO interface_id field. Its compact
format means the framing semantics differ materially from the Enhanced Packet Block (EPB):
`SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: the `original_len: u32` field only), giving a
minimum `block_total_length` of 16 bytes (12-byte outer header + 4-byte body-fixed). The
outer `data` slice from the crate INCLUDES 32-bit padding bytes; wirerust MUST strip them via
`captured_len = min(original_len, snaplen_from_idb[0])` before producing a `RawPacket`.

This is the only holdout scenario covering SPB; BC-2.01.013 has no other holdout (C-2 / I-14
gap from ADR-009 rev 5 HS-completeness map). Case F (btl=12 → E-INP-008) was added in
Pass-4 R4 (ADR-009 rev 7 Decision 20) to distinguish the wirerust body-decode error path
from the crate-level framing error path (Case E, btl=14 → E-INP-010).

### SPB Wire Layout

```
Offset  Size  Field
0       4     block_type = 0x00000003 (little-endian: 03 00 00 00)
4       4     block_total_length (u32 LE) — total length of this block incl. all fields
8       4     original_len (u32 LE) — original captured packet length
12      N     packet data (N = actual captured bytes)
12+N    P     padding to 4-byte boundary (P = (4 - N%4) % 4 bytes of 0x00; omitted if N%4==0)
12+N+P  4     block_total_length (trailing repeat, same value as offset 4)

block_total_length = 12 (outer header: type[4]+total_len[4]+trailing_len[4]) + 4 (original_len) + padded_data_len
                   = 16 + ceil(N/4)*4
```

### Case A — SPB with original_len <= snaplen (no truncation, data returned intact)

1. A crafted pcapng file is presented containing:
   - SHB (little-endian, block_total_length=28, BOM=`4D 3C 2B 1A`, major=1, minor=0,
     section_length=`FF FF FF FF FF FF FF FF`)
   - IDB (block_type=`01 00 00 00`, linktype=1 Ethernet, snaplen=65535;
     block_total_length=20: 12 outer + 8 body-fixed for linktype[2]+reserved[2]+snaplen[4])
   - SPB with a 20-byte Ethernet payload (N=20, padding=(4-20%4)%4=0,
     block_total_length=16+20=36, LE: `24 00 00 00`):
     ```
     block_type:         03 00 00 00
     block_total_length: 24 00 00 00   # 36 decimal
     original_len:       14 00 00 00   # 20 decimal — original_len <= snaplen (65535)
     packet_data:        [20 bytes of valid Ethernet frame payload]
     trailing_total_len: 24 00 00 00
     ```
2. The user runs `wirerust analyze spb_no_trunc.pcapng --json`.
3. The tool exits 0. The JSON output includes `total_packets: 1`. The packet's data length
   is 20 bytes (original_len; no truncation, no extraneous padding bytes).

**Byte-exact SPB layout for Case A (total block = 36 bytes):**
```
Hex: 03 00 00 00  24 00 00 00  14 00 00 00
     [20 bytes of Ethernet frame data, e.g. FF FF FF FF FF FF DE AD BE EF 00 01 08 00 45 00 00 00 00 00]
     24 00 00 00
```
`original_len = 0x14 = 20`. `block_total_length = 0x24 = 36`. Available padded data =
36 - 16 = 20 bytes. `captured_len = min(20, 65535) = 20`. `data.len() == 20`. No padding.

### Case B — SPB with original_len > snaplen (captured data clamped to snaplen)

1. A crafted pcapng file is presented containing:
   - SHB (LE, same as Case A)
   - IDB with snaplen=100 (LE: `64 00 00 00` at body offset 4; block_total_length=20)
   - SPB where a 200-byte packet was captured but snaplen=100 limits what was stored.
     The block body holds 100 bytes of actual data plus 0 padding bytes (100%4==0).
     `block_total_length = 16 + 100 = 116` (LE: `74 00 00 00`).
     `original_len = 200` (LE: `C8 00 00 00`).
     ```
     block_type:         03 00 00 00
     block_total_length: 74 00 00 00   # 116 decimal
     original_len:       C8 00 00 00   # 200 decimal — original_len > snaplen
     packet_data:        [100 bytes]
     trailing_total_len: 74 00 00 00
     ```
2. The user runs `wirerust analyze spb_snaplen_clamp.pcapng --json`.
3. The tool exits 0. `total_packets: 1`. The `RawPacket` data length is 100 bytes
   (`captured_len = min(original_len=200, snaplen=100, block_body_available=100) = 100`;
   all three limits coincide here — snaplen and block_body_available both cap at 100;
   a two-way `min(original_len, snaplen)` impl would produce the same result in this fixture,
   but would diverge if block_body_available < snaplen, e.g. a file truncated mid-body).
   The tool does NOT attempt to read 200 bytes from a 100-byte block body; no out-of-bounds
   read occurs.

### Case C — SPB with original_len NOT 4-byte-aligned (padding bytes stripped)

1. A crafted pcapng file is presented containing:
   - SHB (LE, same as Case A)
   - IDB with snaplen=65535
   - SPB where 13 bytes of packet data were stored. 13 % 4 = 1 → 3 bytes of 0x00 padding.
     Padded data length = 13 + 3 = 16 bytes.
     `block_total_length = 16 + 16 = 32` (LE: `20 00 00 00`).
     `original_len = 13` (LE: `0D 00 00 00`).
     ```
     block_type:         03 00 00 00
     block_total_length: 20 00 00 00   # 32 decimal
     original_len:       0D 00 00 00   # 13 decimal
     packet_data:        [13 bytes, e.g. AA BB CC DD EE FF 00 11 22 33 44 55 66]
     padding:            00 00 00      # 3 bytes to reach 4-byte boundary
     trailing_total_len: 20 00 00 00
     ```
2. The user runs `wirerust analyze spb_unaligned.pcapng --json`.
3. The tool exits 0. `total_packets: 1`. The `RawPacket` data length is **13 bytes** —
   the 3 padding bytes have been stripped. `data.len() == 13`, NOT 16.
   The holdout-evaluator confirms `data.len()` via a downstream assertion that no anomalous
   trailing-null-byte corruption is present in the frame (i.e., a 14-byte Ethernet header
   cannot be constructed from 13 bytes — the tool may emit a decode-skip for the frame, but
   must not panic and must NOT include the padding bytes in the data slice).

**Key observable:** `data.len()` MUST equal `captured_len = min(original_len=13, snaplen=65535) = 13`,
not the padded length of 16.

### Case D — SPB before any IDB (empty interface table) → Err E-INP-009

1. A crafted pcapng file is presented containing:
   - SHB (LE, same as Case A)
   - **NO IDB** — the interface table is empty when the SPB arrives
   - SPB with a minimal 4-byte payload (N=4, padding=0,
     block_total_length=16+4=20, LE: `14 00 00 00`):
     ```
     block_type:         03 00 00 00
     block_total_length: 14 00 00 00   # 20 decimal
     original_len:       04 00 00 00   # 4 decimal
     packet_data:        AA BB CC DD
     trailing_total_len: 14 00 00 00
     ```
2. The user runs `wirerust analyze spb_no_idb.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr. The error message is consistent
   with E-INP-009 ("no interface defined" / "SPB without prior IDB" or similar). No panic.
   No `index out of bounds` Rust backtrace. No JSON output on stdout.

**Exit code:** non-zero (process exits with a non-success status; exact code is 1 or 2).

### Case E — Truncated SPB (block_total_length=14, below 12-byte crate minimum) → Err E-INP-010

1. A crafted pcapng file is presented where the SPB block header claims
   `block_total_length = 14` — which is below the 12-byte minimum that the pcap-file crate
   requires to frame any block (type[4] + total_len[4] + trailing_len[4] = 12 bytes outer
   header alone). The crate cannot frame this block and returns `Err` before wirerust
   body-decode code runs. wirerust maps this to **E-INP-010** (crate-level framing failure).
   ```
   block_type:         03 00 00 00
   block_total_length: 0E 00 00 00   # 14 decimal — BELOW 12-byte crate framing minimum
   [at most 2 bytes — file may be truncated here]
   ```
   Note: a block_total_length of 14 is rejected by the crate before wirerust sees a body.
   The `original_len` field is never reached. This is E-INP-010 (crate-level), not E-INP-008
   (wirerust body-decode-level).
2. The user runs `wirerust analyze truncated_spb.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr referencing a truncated or
   malformed block. The error is consistent with **E-INP-010** (crate-level framing failure).
   No panic. No JSON output on stdout.

**Exit code:** non-zero.

**Note — distinction from Cases D and F:** Case D is a structural error at the file level
(no IDB seen at all before the SPB — E-INP-009). Case E is a crate-level framing error
(block_total_length below the crate framing minimum of 12 — E-INP-010; wirerust body-decode
is never reached). Case F is a wirerust body-decode error (btl=12 frames correctly but
body=0 < 4 SPB fixed-fields — E-INP-008). All three are distinct error codes and paths.

### Case F — SPB with btl=12 (aligned, crate frames, body=0 < 4 SPB fixed-field) → Err E-INP-008

This is the Decision 20 case for SPB, analogous to HS-103 Case D for SHB. With
`block_total_length = 12`, the 12-byte outer header is exactly the block (type[4] +
total_len[4] + trailing_len[4] = 12 bytes). The crate CAN frame this block and delivers
a **zero-byte body** to wirerust. However, the SPB fixed-overhead is `SPB_FIXED_OVERHEAD_BYTES
= 4` (the `original_len: u32` field). A 0-byte body is < 4 bytes — wirerust body-decode MUST
return **E-INP-008** (body-too-short), NOT E-INP-010 (which is a crate-level framing failure).

**Byte-exact block layout (total = 12 bytes):**
```
block_type:         03 00 00 00   # SPB type (LE)
block_total_length: 0C 00 00 00   # 12 decimal (LE u32)
trailing_total_len: 0C 00 00 00   # 12 decimal — NO body bytes between outer fields
```
Total bytes: 12. The crate frames this as a valid block with a 0-byte body. wirerust
receives `body = &[]` (empty slice) and must check `body.len() >= 4` (for `original_len`).

**Full file layout (SHB + IDB + SPB-btl12):**
```
SHB (28 bytes, LE):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB (20 bytes, LE, linktype=1, snaplen=65535):
  01 00 00 00  14 00 00 00  01 00  00 00  FF FF 00 00  14 00 00 00

SPB-btl12 (12 bytes, LE):
  03 00 00 00  0C 00 00 00  0C 00 00 00
```
Total file: 28 + 20 + 12 = 60 bytes.

1. The user runs `wirerust analyze spb_btl12.pcapng --json 2>&1`.
2. The tool exits non-zero. An error is printed to stderr. The error is consistent with
   **E-INP-008** (body too short for SPB fixed field `original_len` — wirerust body-decode
   layer receives a 0-byte body and rejects it). The error is NOT E-INP-010 (which would
   indicate a crate framing failure). No packets emitted. No panic.

**Exit code:** non-zero.

**Key distinction from Case E:** Case E (btl=14) is rejected by the crate before wirerust
sees a body → E-INP-010. Case F (btl=12) is framed by the crate with a zero-byte body →
wirerust E-INP-008. The crate framing threshold is 12 bytes (the outer header size); at
exactly 12 bytes the crate succeeds but delivers an empty body.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.013 | Postcondition 1 — captured_len = min(original_len, snaplen); padding stripped | Cases A and C: correct data length after padding strip |
| BC-2.01.013 | Postcondition 1 — data bounded by min(original_len, snaplen, block_body) | Case B: snaplen clamp prevents over-read |
| BC-2.01.013 | Postcondition 5 / AC-001 — empty interface table → E-INP-009 | Case D: guard fires before idb[0] access |
| BC-2.01.013 | Postcondition 6 / EC-005 — block_total_length < 12 (crate framing failure) → E-INP-010 | Case E: crate cannot frame btl=14; crate Err → wirerust E-INP-010 |
| BC-2.01.013 | Postcondition 6 / EC-005 — btl=12 (crate frames, body=0 < 4 SPB fixed-field) → E-INP-008 | Case F: Decision 20 body-too-short path; distinct from Case E crate framing path |
| BC-2.01.013 | AC-002 (padding strip) — data.len() == captured_len, NOT padded length | Case C: primary padding-strip assertion |
| BC-2.01.013 | AC-003 (no-panic, SEC-005) — Err returned for malformed inputs, no panic | Cases D, E, F: no panic on adversarial inputs |
| BC-2.01.013 | Invariant 2 — data bounded by min(original_len, snaplen) and block body | Cases B and C: bounds invariant |
| BC-2.01.013 | Invariant 3 — RawPacket.timestamp_secs = 0, timestamp_usecs = 0 for SPBs | Case A: SPB timestamps are zero |

## Verification Approach

```
wirerust analyze spb_no_trunc.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, `"total_packets": 1`, packet data length 20 bytes when examined.

```
wirerust analyze spb_snaplen_clamp.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, `"total_packets": 1`, packet data clamped to 100 bytes (not 200).

```
wirerust analyze spb_unaligned.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, `"total_packets": 1`, packet data 13 bytes (padding stripped; not 16).

```
wirerust analyze spb_no_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with E-INP-009, no JSON on stdout.

```
wirerust analyze truncated_spb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-010** (crate-level framing
failure — btl=14 cannot be framed by crate), no JSON on stdout.

```
wirerust analyze spb_btl12.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (body too short for
SPB fixed field — crate successfully framed the 12-byte block with 0-byte body; wirerust
body-decode rejects it). NOT E-INP-010. No JSON on stdout. No panic.

## Evaluation Rubric

- **Padding-strip correctness** (weight: 0.25): Cases A and C — `data.len()` is exactly
  `captured_len = min(original_len, snaplen)` after the crate's padded slice is trimmed.
  Case A confirms no truncation when not needed. Case C confirms padding bytes are NOT included.
- **Snaplen clamp correctness** (weight: 0.20): Case B — `data.len() == 100` when
  `min(original_len=200, snaplen=100, block_body_available=100) = 100`; no read beyond block body.
  The three-way min makes the bound observable: all three limits converge to 100 in this fixture.
- **No-panic safety** (weight: 0.25): Cases D, E, F — unchecked `idb[0]` access, crate
  framing failures, and body-too-short conditions all return graceful `Err`, never panic.
- **E-INP-008 / E-INP-010 split (Decision 20)** (weight: 0.15): Case E produces E-INP-010
  (crate framing failure, btl=14); Case F produces E-INP-008 (wirerust body-decode failure,
  btl=12 with 0-byte body). These must NOT be conflated — the split verifies that the
  error code correctly reflects whether the crate or wirerust body-decode rejected the block.
- **Error specificity** (weight: 0.10): Cases D, E, F produce distinct errors
  (E-INP-009 / E-INP-010 / E-INP-008); stderr messages are human-readable, not raw panics.
- **Timestamp zero invariant** (weight: 0.05): Case A — `RawPacket.timestamp_secs = 0` and
  `timestamp_usecs = 0`; observable via `skipped_packets` count remaining at 0 and
  no timestamp-related error in output.

## Edge Conditions

- SPBs are rare in practice (Wireshark does not generate them), but the pcapng specification
  requires compliant readers to support them. Avoiding SPB tests leaves a gap in the reader
  security surface.
- The pcapng spec requires that when SPBs are present, there is exactly one IDB (so that
  the implicit `idb[0]` access is always safe). The E-INP-009 case (Case D) exercises the
  guard for the zero-IDB case — a malformed but grammatically plausible file.
- `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative) vs `EPB_FIXED_OVERHEAD_BYTES = 20` (body-
  relative). Confusing these constants would cause a wrong minimum-length check and/or
  incorrect padding-strip calculation. Case E probes whether the constant is correct.
- Padding bytes are always 0x00 per the pcapng spec, but the EVALUATOR does NOT assume the
  implementation validates their value — the observable contract is only `data.len()`.
- For Case C, the downstream Ethernet decoder may reject the 13-byte frame as too short
  for a valid Ethernet header (minimum 14 bytes); this is expected and not a failure.
  The assertion is solely `data.len() == 13` (or equivalently, `skipped_packets` is
  incremented for the frame rather than 0 — either is acceptable so long as no panic
  occurs and data does not contain the 3 padding bytes).

## Fixture Construction Note

All six SPB fixtures share the same SHB and IDB prefix (except Case D which omits the IDB;
Case F uses the standard IDB). The Case F fixture (`spb_btl12.pcapng`) has only a
12-byte SPB block (no body bytes between the outer header fields — the trailing BTL
immediately follows the leading BTL).

**SHB (28 bytes, LE):**
```
Hex: 0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
     FF FF FF FF FF FF FF FF  1C 00 00 00
```
Breakdown: block_type=`0A0D0D0A`, btl=28 (`1C 00 00 00`), BOM=`4D 3C 2B 1A` (LE sentinel),
major=1 (`01 00`), minor=0 (`00 00`), section_length=-1 (`FF FF FF FF FF FF FF FF`),
trailing_btl=28 (`1C 00 00 00`).

**IDB (20 bytes, LE, linktype=Ethernet, snaplen=65535):**
```
Hex: 01 00 00 00  14 00 00 00  01 00  00 00  FF FF 00 00  14 00 00 00
```
Breakdown: block_type=`00000001` (`01 00 00 00`), btl=20 (`14 00 00 00`),
linktype=1 (`01 00`), reserved=0 (`00 00`), snaplen=65535 (`FF FF 00 00`),
trailing_btl=20 (`14 00 00 00`).

**IDB (20 bytes, LE, snaplen=100, for Case B):**
```
Hex: 01 00 00 00  14 00 00 00  01 00  00 00  64 00 00 00  14 00 00 00
```
snaplen=100 (`64 00 00 00`).

**Case A SPB payload note:** Use 20 bytes of any Ethernet-like data; the decoder may or may
not fully parse it depending on the ethertype. The holdout passes if `total_packets: 1` and
no error. A zero-padded payload is acceptable: `[00 * 20]`.

## Failure Guidance

"HOLDOUT LOW: HS-107 (satisfaction: 0.XX) — SPB framing has defects.
Case A failure (exit non-zero) indicates SPB block parsing is absent or crashes on valid input.
Case B failure (data.len() > 100) indicates snaplen clamping is missing for SPBs.
Case C failure (data.len() == 16 instead of 13) indicates padding bytes are not stripped.
Case D failure (panic / index OOB) indicates the empty-interface-table guard (H-4 fix) is
absent or not applied on the SPB path.
Case E failure (exit 0 or panic) indicates the crate-level minimum framing check is absent; btl=14 cannot be framed by the crate and must produce E-INP-010.
Case F failure (exit 0, panic, or wrong error code E-INP-010 instead of E-INP-008) indicates the body-too-short path is missing or conflated with the crate framing path. btl=12 means the crate delivers a 0-byte body; wirerust body-decode MUST check body.len() >= 4 (for original_len) and return E-INP-008, not E-INP-010.
See BC-2.01.013 (AC-001 through AC-004), VP-028, VP-031, ADR-009 rev 4 Decision 2 (SPB section), ADR-009 rev 7 Decision 20."
