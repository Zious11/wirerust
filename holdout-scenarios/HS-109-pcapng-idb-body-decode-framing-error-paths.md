---
document_type: holdout-scenario
level: ops
version: "1.0"  # Pass-8 M-2 remediation: IDB (BC-2.01.011) was the only framing BC whose body-decode error paths had NO holdout (only unit-test ACs). SHB/EPB/SPB all have holdouts (HS-103/104/107). This scenario closes that gap with 5 cases: (a) btl=16 body-too-short → E-INP-008; (b) reserved!=0 structural → E-INP-008; (c) options-TLV malformed length → E-INP-008; (d) if_tsresol code 9 with option_length=4 → E-INP-008; (e) well-formed IDB positive control → parses Ok.
status: draft
producer: product-owner
timestamp: 2026-06-20T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.011.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-109"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.011
verification_properties:
  - VP-026
  - VP-027
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng IDB Body-Decode Framing — Body-Too-Short, Reserved Field, Malformed Options TLV, and if_tsresol Length Enforcement

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The Interface Description Block (IDB, block type `0x00000001`) defines one capture interface.
wirerust extracts `linktype` and the optional `if_tsresol` TLV option (option code 9) from every
IDB. BC-2.01.011 specifies four distinct wirerust body-decode error paths that must produce
`Err` mapped to **E-INP-008** (not a panic, not E-INP-010, not a silent default). This scenario
covers all four error paths plus one positive-control case, closing the gap identified as
finding M-2 in adversary pass-8.

### IDB Wire Layout

```
Offset  Size  Field
0       2     linktype (u16 LE) — DataLink enum value
2       2     reserved (u16 LE) — MUST be 0x0000; non-zero → E-INP-008
4       4     snaplen (u32 LE) — read-and-discarded; not stored (F-M3 / Decision 21)
--- fixed fields end at body offset 8 ---
8+      N     options region (TLV-encoded, variable length)
              Each option: option-code u16 + option-length u16 + value (padded to 4-byte boundary)
              Terminates at opt_endofopt (code 0) or end-of-body.
```

**IDB outer block layout (all block types share the outer header):**

```
block_type:         01 00 00 00   # IDB type (LE u32)
block_total_length: XX 00 00 00   # total block size (LE u32, including this field + trailing repeat)
[body bytes]                       # body = block_total_length - 12
trailing_total_len: XX 00 00 00   # repeat of block_total_length
```

Outer header: `block_type[4] + block_total_length[4] + trailing_block_total_length[4]` = 12 bytes.
`body.len() = block_total_length - 12`.

**IDB fixed-field minimum:** `linktype[2] + reserved[2] + snaplen[4]` = 8 bytes.
Constructible E-INP-008 window (Decision 20): `12 ≤ block_total_length < 20` → body = 0–7 bytes < 8.

**Standard file prefix (SHB, used by all five cases):**

```
SHB (28 bytes, LE):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00
```

Breakdown: block_type=`0A0D0D0A`, btl=28 (`1C 00 00 00`), BOM=`4D 3C 2B 1A` (LE sentinel),
major=1 (`01 00`), minor=0 (`00 00`), section_length=-1 (`FF FF FF FF FF FF FF FF`),
trailing_btl=28 (`1C 00 00 00`).

---

### Case A — IDB btl=16 (crate frames; body=4 < 8 IDB fixed-field minimum) → E-INP-008

This is the canonical IDB Decision 20 case. `block_total_length = 16` means the crate CAN frame
the block (outer header: 12 bytes; body: 4 bytes — sufficient for the crate to deliver a
`RawBlock` to wirerust). However, the IDB body is only 4 bytes, which is below the 8-byte
minimum required for the IDB fixed fields (`linktype:2 + reserved:2 + snaplen:4` = 8 bytes).
wirerust's body-decode code receives a 4-byte body slice and MUST return **E-INP-008**
(body-too-short), NOT E-INP-010 (which is a crate-level framing failure). This mirrors the
SHB body-too-short case in HS-103 Case D and the SPB body-too-short case in HS-107 Case F.

**Byte-exact IDB block layout (total = 16 bytes):**

```
block_type:         01 00 00 00   # IDB type (LE u32)
block_total_length: 10 00 00 00   # 16 decimal (LE u32)
[body: 4 bytes — cannot contain the 8 required IDB fixed fields]
  body bytes (any 4 bytes, e.g.): 00 01 00 00
trailing_total_len: 10 00 00 00   # 16 decimal (LE u32)
```

Total block: 12-byte outer header + 4-byte body = 16 bytes.

**Full fixture file (SHB + IDB-btl16, total = 28 + 16 = 44 bytes):**

```
SHB (28 bytes):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB-btl16 (16 bytes):
  01 00 00 00  10 00 00 00  00 01 00 00  10 00 00 00
```

Note: the 4-byte body `00 01 00 00` could look like `linktype=0x0100=256` (LE), but there is
no remaining body for `reserved` or `snaplen`, so wirerust's body-length guard fires first.
The crate successfully frames this 16-byte block and delivers a 4-byte body to wirerust.
wirerust MUST check `body.len() >= 8` and return E-INP-008 when the check fails.

1. A crafted 44-byte pcapng file is presented with the layout above.
2. The user runs `wirerust analyze idb_btl16.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr consistent with **E-INP-008** (IDB
   structural parse failure — wirerust body-decode finds body < 8 bytes). The error is NOT
   E-INP-010 (which would indicate a crate framing failure; the crate succeeded here). No
   packets are emitted. No panic occurs. No Rust backtrace.

**Exit code:** non-zero (1 or 2).

---

### Case B — IDB reserved field != 0 (bytes 2–3 nonzero) → E-INP-008

The pcapng spec requires the IDB `reserved` field (body offset 2–3, u16) to be zero. The
`pcap-file` crate enforces this at `interface_description.rs:48-49`: a non-zero reserved field
is treated as a structural IDB error. wirerust mirrors this enforcement on the raw-block path —
a non-zero `reserved` field MUST return `Err` mapped to **E-INP-008**.

**IDB body layout with reserved = 0x0100 (non-zero):**

```
Offset  Value     Bytes (LE)     Field
0-1     1         01 00          linktype = ETHERNET (DataLink::ETHERNET = 1)
2-3     0x0100    00 01          reserved = 256 (NON-ZERO — MUST be 0x0000)
4-7     65535     FF FF 00 00    snaplen = 65535 (read-and-discarded per F-M3)
```

**Full IDB block (total = 20 bytes, btl=20):**

```
block_type:         01 00 00 00   # IDB type (LE)
block_total_length: 14 00 00 00   # 20 decimal (LE)
linktype:           01 00          # ETHERNET
reserved:           00 01          # 0x0100 — non-zero: structural error → E-INP-008
snaplen:            FF FF 00 00    # 65535 (read-and-discarded)
trailing_total_len: 14 00 00 00   # 20 decimal (LE)
```

**Full fixture file (SHB + IDB-nonzero-reserved, total = 28 + 20 = 48 bytes):**

```
SHB (28 bytes):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB (20 bytes, reserved=0x0100):
  01 00 00 00  14 00 00 00  01 00  00 01  FF FF 00 00  14 00 00 00
```

1. A crafted 48-byte pcapng file is presented with the layout above.
2. The user runs `wirerust analyze idb_nonzero_reserved.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr consistent with **E-INP-008** (IDB
   structural parse failure — non-zero reserved field). No packets are emitted. No panic. No
   Rust backtrace.

**Exit code:** non-zero.

---

### Case C — IDB options-TLV with option_length exceeding remaining body → E-INP-008

This case exercises the IDB options-walk TLV bounds-check (BC-2.01.011 AC-005, EC-011, PC6).
The IDB body contains a well-formed fixed-fields section (8 bytes), followed by an options region
where an option's `option_length` field claims more bytes than remain in the body. wirerust MUST
bounds-check `option_length` against the number of remaining bytes BEFORE reading the option
value. Consuming bytes past the end of the options region (OOB read / panic) is prohibited.

**IDB body layout with malformed options TLV:**

```
Fixed fields (8 bytes):
  Offset 0-1: 01 00          linktype = ETHERNET
  Offset 2-3: 00 00          reserved = 0 (valid)
  Offset 4-7: FF FF 00 00    snaplen = 65535 (read-and-discarded)

Options region (4 bytes — only room for one minimal TLV header, no room for its claimed value):
  Offset 8-9:   02 00        option_code = 2 (if_name — an arbitrary unknown option)
  Offset 10-11: 20 00        option_length = 32 (claims 32 bytes of value; only 0 bytes remain)
  [no value bytes — the body ends here; option_length exceeds remaining body by 32 bytes]
```

**Full IDB block (total = 24 bytes, btl=24, body=12 bytes):**

```
block_type:         01 00 00 00   # IDB type (LE)
block_total_length: 18 00 00 00   # 24 decimal (LE)
linktype:           01 00          # ETHERNET
reserved:           00 00          # 0 (valid)
snaplen:            FF FF 00 00    # 65535 (read-and-discarded)
option_code:        02 00          # code 2 (if_name; arbitrary; should be skipped)
option_length:      20 00          # 32 bytes claimed — EXCEEDS remaining body (0 bytes left)
trailing_total_len: 18 00 00 00   # 24 decimal (LE)
```

Body = 24 - 12 = 12 bytes. Fixed fields consume 8 bytes. Options region: 4 bytes
(`option_code[2] + option_length[2]`). Remaining body after the TLV header: 0 bytes.
`option_length = 32` claims 32 value bytes, but 0 remain — TLV bounds-check MUST fire.

**Full fixture file (SHB + IDB-malformed-options, total = 28 + 24 = 52 bytes):**

```
SHB (28 bytes):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB (24 bytes, malformed options TLV):
  01 00 00 00  18 00 00 00  01 00  00 00  FF FF 00 00
  02 00  20 00
  18 00 00 00
```

1. A crafted 52-byte pcapng file is presented with the layout above.
2. The user runs `wirerust analyze idb_malformed_options.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr consistent with **E-INP-008** (IDB
   structural parse failure — options-TLV `option_length` exceeds remaining body; OOB read
   attempt detected and rejected before any slice access). No packets are emitted. No panic.
   No Rust backtrace.

**Exit code:** non-zero.

---

### Case D — IDB if_tsresol (code 9) option with option_length=4 (must be 1) → E-INP-008

This case exercises the `if_tsresol` option-length enforcement added in BC-2.01.011 v1.6
(F-M5 / ADR-009 rev 9). The `if_tsresol` option (option code 9) encodes a single `u8` byte.
An `option_length != 1` is a malformed TLV. wirerust MUST NOT silently ignore this or fall back
to the default exponent 6. Instead it MUST return `Err` mapped to **E-INP-008**.

**IDB body layout with if_tsresol option_length=4:**

```
Fixed fields (8 bytes):
  Offset 0-1: 01 00          linktype = ETHERNET
  Offset 2-3: 00 00          reserved = 0 (valid)
  Offset 4-7: FF FF 00 00    snaplen = 65535 (read-and-discarded)

Options region — if_tsresol TLV with wrong length:
  Offset 8-9:   09 00        option_code = 9 (if_tsresol)
  Offset 10-11: 04 00        option_length = 4 (WRONG — must be 1 for a single-byte u8)
  Offset 12-15: 06 00 00 00  4 bytes of claimed value (padded to 4-byte boundary)
                               (value bytes; content does not matter — length check fires first)
  Offset 16-17: 00 00        opt_endofopt (code 0)
  Offset 18-19: 00 00        option_length = 0 (endofopt has no value)
```

**Full IDB block (total = 32 bytes, btl=32, body=20 bytes):**

```
block_type:         01 00 00 00   # IDB type (LE)
block_total_length: 20 00 00 00   # 32 decimal (LE)
linktype:           01 00          # ETHERNET
reserved:           00 00          # 0 (valid)
snaplen:            FF FF 00 00    # 65535 (read-and-discarded)
option_code:        09 00          # 9 = if_tsresol
option_length:      04 00          # 4 — WRONG (must be 1); wirerust MUST reject → E-INP-008
option_value:       06 00 00 00    # 4 claimed bytes (padded value; content irrelevant)
opt_endofopt:       00 00          # code 0 = end-of-options
eoo_length:         00 00          # 0 = no value
trailing_total_len: 20 00 00 00   # 32 decimal (LE)
```

Body = 32 - 12 = 20 bytes. Fixed fields: 8 bytes. Options: 12 bytes (if_tsresol TLV: 8 bytes;
opt_endofopt TLV: 4 bytes). Remaining body after all options: 0 bytes (exact fit).
The `if_tsresol` option itself is structurally bounded (option_length=4 with 12 remaining bytes
when processing starts — the OOB check passes). The failure is at the SEMANTIC level:
`option_length != 1` for `if_tsresol` (option code 9) is a malformed TLV per BC-2.01.011
AC-005 and EC-013. wirerust MUST return E-INP-008 on detecting this.

**Full fixture file (SHB + IDB-tsresol-wrong-len, total = 28 + 32 = 60 bytes):**

```
SHB (28 bytes):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB (32 bytes, if_tsresol option_length=4):
  01 00 00 00  20 00 00 00  01 00  00 00  FF FF 00 00
  09 00  04 00  06 00 00 00
  00 00  00 00
  20 00 00 00
```

1. A crafted 60-byte pcapng file is presented with the layout above.
2. The user runs `wirerust analyze idb_tsresol_wrong_len.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr consistent with **E-INP-008** (IDB
   structural parse failure — `if_tsresol` TLV has `option_length = 4`, which is not 1;
   wirerust MUST NOT silently default to exponent 6 or ignore the malformed TLV). No packets
   are emitted. No panic. No Rust backtrace.

**Exit code:** non-zero.

**Key distinction from Case C:** Case C has an option whose `option_length` exceeds remaining body
bytes (OOB-level bounds check fails). Case D has an option whose `option_length` is within bounds
(4 bytes available, 4 claimed) but whose semantics are wrong for `if_tsresol` specifically
(semantic length enforcement: `if_tsresol` MUST be `option_length == 1`). Both paths produce
E-INP-008, but they exercise different guard points in the options-walk code.

---

### Case E — Well-formed IDB (positive control) → parses Ok, contributes to valid single-packet read

This positive-control case verifies that the error cases above do not reflect a global regression
in IDB parsing. A well-formed IDB with `linktype = ETHERNET` (1), `reserved = 0`, `snaplen` any,
and a valid `if_tsresol` option (code 9, option_length=1, value=0x06 for microseconds) MUST parse
correctly and produce a `DataLink::ETHERNET` interface entry at index 0.

The file also contains one EPB with a minimal Ethernet frame so that a single packet is ingested
and the output is non-trivially verifiable.

**Well-formed IDB body layout:**

```
Offset 0-1: 01 00          linktype = ETHERNET (DataLink::ETHERNET = 1)
Offset 2-3: 00 00          reserved = 0 (valid)
Offset 4-7: FF FF 00 00    snaplen = 65535 (read-and-discarded per F-M3)

Options region:
  Offset 8-9:   09 00      option_code = 9 (if_tsresol)
  Offset 10-11: 01 00      option_length = 1 (correct — single byte)
  Offset 12:    06         value = 0x06 (base-10 microseconds; the default exponent)
  Offset 13-15: 00 00 00   3 bytes of padding to 4-byte boundary
  Offset 16-17: 00 00      opt_endofopt (code 0)
  Offset 18-19: 00 00      option_length = 0 (no value)
```

**Full IDB block (total = 32 bytes, btl=32, body=20 bytes):**

```
block_type:         01 00 00 00
block_total_length: 20 00 00 00   # 32 decimal (LE)
linktype:           01 00
reserved:           00 00
snaplen:            FF FF 00 00
option_code:        09 00          # if_tsresol
option_length:      01 00          # 1 (correct)
option_value:       06             # exponent = 6 (microseconds, base-10)
padding:            00 00 00       # 3 bytes to reach 4-byte boundary
opt_endofopt:       00 00
eoo_length:         00 00
trailing_total_len: 20 00 00 00
```

**Minimal EPB (Enhanced Packet Block) carrying one Ethernet frame (total = 60 bytes):**

A minimal well-formed EPB at interface_id=0, timestamps=0, with 14 bytes of Ethernet frame data
(minimum Ethernet header: dst_mac[6] + src_mac[6] + ethertype[2] = 14 bytes; no payload —
the decoder may skip it as too short, but must not panic; `total_packets` MUST be 1):

```
EPB body-relative layout:
  interface_id:   00 00 00 00   # interface 0
  ts_high:        00 00 00 00   # timestamp = 0
  ts_low:         00 00 00 00
  captured_len:   0E 00 00 00   # 14 decimal (LE)
  original_len:   0E 00 00 00   # 14 decimal (LE)
  packet_data:    FF FF FF FF FF FF  DE AD BE EF 00 01  08 00
                  (broadcast dst, fake src, ethertype IPv4)
  padding:        00 00         # 14 % 4 = 2; pad = 2 bytes to reach 4-byte boundary

block_total_length = 12 (outer header) + 20 (EPB fixed body) + 14 (data) + 2 (padding) = 48
```

**Full EPB block (total = 48 bytes):**

```
block_type:         06 00 00 00   # EPB type (LE)
block_total_length: 30 00 00 00   # 48 decimal (LE)
interface_id:       00 00 00 00
ts_high:            00 00 00 00
ts_low:             00 00 00 00
captured_len:       0E 00 00 00   # 14
original_len:       0E 00 00 00   # 14
packet_data:        FF FF FF FF FF FF  DE AD BE EF 00 01  08 00
padding:            00 00
trailing_total_len: 30 00 00 00   # 48 decimal (LE)
```

**Full fixture file (SHB + IDB + EPB, total = 28 + 32 + 48 = 108 bytes):**

```
SHB (28 bytes):
  0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
  FF FF FF FF FF FF FF FF  1C 00 00 00

IDB (32 bytes, well-formed):
  01 00 00 00  20 00 00 00  01 00  00 00  FF FF 00 00
  09 00  01 00  06  00 00 00
  00 00  00 00
  20 00 00 00

EPB (48 bytes):
  06 00 00 00  30 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00
  0E 00 00 00  0E 00 00 00
  FF FF FF FF FF FF  DE AD BE EF 00 01  08 00  00 00
  30 00 00 00
```

1. A crafted 108-byte pcapng file is presented with the layout above.
2. The user runs `wirerust analyze idb_wellformed.pcapng --json`.
3. The tool exits 0. The JSON output contains `"total_packets": 1`. No error on stderr. The IDB
   was parsed correctly: `linktype = ETHERNET`, `if_tsresol` exponent = 6 (microseconds). The
   EPB references interface_id=0, which is valid (table has one entry). The 14-byte Ethernet
   frame is either decoded or counted as a skipped packet (insufficient for a full IP parse),
   but `total_packets` MUST be 1 (not 0, not an error). No panic.

**Exit code:** 0.

**Key observable:** The JSON output includes `"total_packets": 1`. If `total_packets` is absent
or zero, the IDB positive-control path is broken. If exit is non-zero, the IDB parser has a
regression that rejects valid input.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.011 | PC5 / EC-008 — btl=16 (12≤btl<20, crate frames, body=4 < 8 IDB fixed-field minimum) → E-INP-008 (wirerust body-decode path, NOT crate framing) | Case A: body-too-short path; crate successfully delivers 4-byte body; wirerust body-decode MUST check body.len()>=8 and return E-INP-008 |
| BC-2.01.011 | PC4 / EC-010 — IDB `reserved` field non-zero → E-INP-008 (structural IDB error; mirrors crate enforcement at interface_description.rs:48-49) | Case B: non-zero reserved field (0x0100) produces E-INP-008; linktype/snaplen are present and valid — the reserved check fires independently |
| BC-2.01.011 | PC6 / AC-005 / EC-011 — IDB options-TLV with option_length exceeding remaining body bytes → E-INP-008 (no panic, no OOB read) | Case C: option_length=32 with 0 remaining body bytes; OOB bounds check fires before any slice access |
| BC-2.01.011 | PC6 / AC-005 / EC-013 — if_tsresol (code 9) option with option_length=4 (must be 1) → E-INP-008 (semantic length enforcement; NOT silently defaulted) | Case D: if_tsresol TLV with option_length=4 is structurally bounded (no OOB) but semantically malformed; wirerust MUST reject with E-INP-008, not default to exponent 6 |
| BC-2.01.011 | PC1 / PC2 / PC3 / AC-003 — well-formed IDB with linktype=ETHERNET, reserved=0, if_tsresol code 9 option_length=1, value=0x06 → interface table entry with DataLink::ETHERNET and exponent=6; EPB referencing interface 0 succeeds | Case E: positive-control; confirms error cases above do not indicate a global IDB regression |
| BC-2.01.011 | AC-001 — no-panic (SEC-005): all error cases MUST return Err, never panic | Cases A–D: no panic permitted on any malformed input |
| BC-2.01.011 | AC-002 — interface table Vec<InterfaceInfo>; O(1) index | Case E: one IDB produces one entry at index 0; EPB at interface_id=0 resolves correctly |

## Verification Approach

```
wirerust analyze idb_btl16.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (IDB body-decode: body=4 < 8
IDB fixed-field minimum; wirerust body-decode path, NOT crate framing). NOT E-INP-010. No JSON.
No panic.

```
wirerust analyze idb_nonzero_reserved.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (IDB structural: non-zero
reserved field). No JSON. No panic. The file is structurally complete (btl=20, body=8 bytes);
the error is semantic at body-decode time.

```
wirerust analyze idb_malformed_options.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (IDB options-walk:
option_length=32 exceeds remaining body; OOB bounds-check fires). No JSON. No panic. No Rust
backtrace with `index out of bounds`.

```
wirerust analyze idb_tsresol_wrong_len.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (IDB options-walk:
if_tsresol TLV option_length=4 != 1; semantic length enforcement fires; NOT silently defaulted
to exponent 6). No JSON. No panic.

```
wirerust analyze idb_wellformed.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, `"total_packets": 1` in JSON output, no error on stderr. Confirms the four
error-path cases above are targeted rejections, not global IDB-parsing regressions.

## Evaluation Rubric

- **No-panic safety** (weight: 0.30): Cases A, B, C, D — no panic for any malformed IDB input.
  The error must be returned gracefully. A Rust backtrace (`thread 'main' panicked`,
  `index out of bounds`, `attempt to subtract with overflow`) is a FAILURE regardless of exit code.
- **Error code correctness — E-INP-008 not E-INP-010** (weight: 0.25): Cases A, B, C, D MUST
  produce E-INP-008 (wirerust body-decode path). Specifically, Case A MUST NOT produce E-INP-010
  (btl=16 is framed by the crate; the error is at the wirerust body-decode layer). Collapsing the
  two codes into a single error loses the Decision 20 distinction.
- **Positive-control correctness** (weight: 0.25): Case E — exit 0, `total_packets: 1`. Confirms
  that the IDB happy path is functional. A Case E failure indicates the error cases above are not
  targeted but reflect a broader IDB-parsing defect.
- **if_tsresol semantic enforcement** (weight: 0.10): Case D — `option_length=4` for
  `if_tsresol` MUST be rejected with E-INP-008. A silent fallback to exponent 6 (default) is a
  FAILURE of the AC-005 semantic-length enforcement even if the tool subsequently "works".
- **Options-TLV bounds-check** (weight: 0.10): Case C — `option_length=32` with 0 remaining body
  bytes MUST be caught before any slice access. A successful parse of the malformed TLV (exit 0)
  or an unguarded slice panic are both FAILURES.

## Edge Conditions

- **Decision 20 boundary (Case A):** The constructible E-INP-008 window for IDB is
  `12 ≤ block_total_length < 20` — the crate frames the block (btl ≥ 12, alignment OK) but the
  body is 0–7 bytes (btl − 12 < 8). `btl = 16` gives `body = 4 bytes < 8`. `btl < 12` or a
  non-4-aligned `btl` would instead produce E-INP-010 (crate framing rejection). The holdout
  uses `btl = 16` (the canonical fixture per BC-2.01.011 EC-008) to land squarely in the
  wirerust body-decode path.
- **Reserved = 0 vs. non-zero (Case B):** The pcapng spec says reserved "should" be zero; the
  `pcap-file` crate treats non-zero as an error. wirerust mirrors this on the raw-block path.
  The test uses `reserved = 0x0100` (bytes: `00 01` in LE at body offset 2–3). A value of
  `0x0001` (bytes: `01 00`) would be indistinguishable from `linktype = 1` in a naive read —
  use `0x0100` to avoid any ambiguity with the ETHERNET linktype byte pattern.
- **Options region in body: Case C vs. Case D distinction:**
  - Case C: `option_length` exceeds remaining body bytes — the GENERIC bounds check that applies
    to ALL option codes fires. wirerust MUST catch this before reading any value bytes.
  - Case D: `option_length` is within remaining body bounds but is semantically wrong for
    `if_tsresol` specifically (code 9 requires exactly `option_length = 1`). The GENERIC bounds
    check would pass; the SPECIFIC `if_tsresol` semantic check must additionally fire.
  These are two distinct code paths in the options-walk loop.
- **Snaplen in Case A:** The 4-byte body in Case A (`00 01 00 00`) cannot be a valid `reserved`
  field (the `linktype` at offset 0–1 would be `0x0100 = 256`, and there are no bytes remaining
  for `reserved` and `snaplen`). wirerust's `body.len() >= 8` guard MUST fire before any field
  reads. If the guard is absent, wirerust may attempt to read `linktype` from bytes 0–1 and then
  crash attempting `reserved` or `snaplen` from a 2-byte or 0-byte slice.
- **if_tsresol with option_length=4 (Case D):** The 4 value bytes (`06 00 00 00`) happen to
  contain `0x06` at offset 0 — the correct exponent for microseconds. A naive implementation
  might read the first byte and silently succeed. The AC-005 enforcement requires that wirerust
  CHECK `option_length == 1` BEFORE reading any value byte. If wirerust reads byte 0 and defaults,
  it passes the functional test but fails the contract test. The holdout-evaluator infers this
  from the `E-INP-008` error requirement — there is no output-observation shortcut.

## Fixture Construction Note

All five fixtures share the same SHB prefix. Fixtures differ only in the IDB block:

| Case | Fixture filename          | btl  | IDB body anomaly                          | Total file size |
|------|---------------------------|------|-------------------------------------------|-----------------|
| A    | `idb_btl16.pcapng`        | 16   | body = 4 bytes < 8 IDB fixed-field min   | 44 bytes        |
| B    | `idb_nonzero_reserved.pcapng` | 20 | reserved field = 0x0100 (non-zero)       | 48 bytes        |
| C    | `idb_malformed_options.pcapng` | 24 | option_length = 32, 0 body bytes remain | 52 bytes        |
| D    | `idb_tsresol_wrong_len.pcapng` | 32 | if_tsresol option_length = 4 (not 1)    | 60 bytes        |
| E    | `idb_wellformed.pcapng`   | 32   | well-formed IDB + EPB (positive control)  | 108 bytes       |

Cases B–E all use `block_total_length ≥ 20`, so the crate frames the block and delivers a
body of at least 8 bytes; all rejections are at the wirerust body-decode layer (E-INP-008),
not the crate framing layer (E-INP-010). Case A alone uses `btl = 16` to produce a 4-byte body.

## Failure Guidance

"HOLDOUT LOW: HS-109 (satisfaction: 0.XX) — IDB body-decode error paths have defects.
Case A failure (exit 0, panic, or E-INP-010 instead of E-INP-008) indicates the IDB body-too-short path is missing or conflated with the crate framing path. btl=16 means the crate delivers a 4-byte body; wirerust body-decode MUST check body.len()>=8 and return E-INP-008, NOT E-INP-010.
Case B failure (exit 0 or panic) indicates the IDB reserved-field check is absent; a non-zero reserved field (0x0100) must be rejected with E-INP-008 matching the crate's interface_description.rs:48-49 enforcement.
Case C failure (exit 0, panic, or index OOB backtrace) indicates the IDB options-walk TLV bounds-check is absent; option_length=32 with 0 remaining bytes must be caught before any slice access. A 'index out of bounds' Rust backtrace is a FAILURE.
Case D failure (exit 0 or silent default to microseconds exponent) indicates the if_tsresol option_length semantic enforcement (AC-005 / F-M5 / ADR-009 rev 9) is absent. option_length=4 for if_tsresol (code 9) MUST produce E-INP-008 — NOT a silent fallback to the default exponent 6.
Case E failure (exit non-zero or total_packets != 1) indicates a regression in IDB happy-path parsing; a well-formed IDB+EPB file must parse successfully and produce exactly one packet.
See BC-2.01.011 PC4/PC5/PC6/AC-001/AC-005/EC-008/EC-010/EC-011/EC-013, VP-026, VP-027, ADR-009 rev 7 Decision 20, ADR-009 rev 9 F-M5."
