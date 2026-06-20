---
document_type: holdout-scenario
level: ops
version: "1.3"  # Pass-5 S1 / ADR-009 rev 8 C-1: Case D and Case E reclassified from E-INP-010 → E-INP-008. EPB captured_len+padding overrunning the body is a wirerust body-decode rejection (crate already framed the block with btl>=12); all wirerust body-decode failures use E-INP-008 per Decision 20. Case D bound-by-body (captured_len > btl-32) and Case E padding-overrun both updated in scenario text, BC linkage table, verification approach, evaluation rubric, and edge conditions. Prior: v1.2 added Case E as E-INP-010. ADR-009 rev 7.
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.012.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-104"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.012
verification_properties:
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

# Holdout Scenario: pcapng EPB Framing — Interface-ID Bounds Checks and Captured-Len Guard

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The Enhanced Packet Block (EPB) contains a 32-bit `interface_id` field referencing the
interface table built from prior IDB blocks. An unchecked table index is a safety
vulnerability (out-of-bounds Vec access). This scenario exercises four boundary conditions
on the EPB parser: two interface_id bounds cases, one valid captured_len boundary, and one
invalid captured_len that must be rejected before any allocation.

### Case A — interface_id = u32::MAX with EMPTY interface table → E-INP-009

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - NO IDB — the interface table is empty when the EPB arrives
   - EPB with interface_id = 0xFFFFFFFF (u32::MAX) and a minimal Ethernet payload
2. The user runs `wirerust analyze epb_no_idb.pcapng --json 2>&1`.
3. The tool exits non-zero. Stderr contains an error referencing that no interface was
   found (or similar; message must indicate the EPB could not be processed without an
   interface). No panic. No crash. Error code or message substring consistent with E-INP-009.

### Case B — interface_id = u32::MAX on a 1-ENTRY table → E-INP-010 (OOB on non-empty table)

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - ONE IDB with linktype=1 (Ethernet), if_tsresol=6 — table now has one entry at index 0
   - EPB with interface_id = 0xFFFFFFFF (u32::MAX) — references index u32::MAX on a
     1-element table; this is an out-of-bounds access on a non-empty table (distinct from Case A)
2. The user runs `wirerust analyze epb_oob_idb.pcapng --json 2>&1`.
3. The tool exits non-zero. Stderr contains an error referencing that the interface_id
   is out of range relative to the number of known interfaces. No panic. No Vec index
   panic (which would produce `index out of bounds` in the Rust backtrace). Error
   consistent with E-INP-010.

### Case C — captured_len = block_total_length - 32 (VALID boundary)

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - IDB with linktype=1 and if_tsresol=6
   - EPB where captured_len is set to exactly (block_total_length - 32), which is the
     maximum valid value (12-byte outer header + 20-byte EPB body fixed fields = 32 bytes
     minimum; all remaining bytes are packet data, unpadded). For example:
     block_total_length = 48; fixed overhead = 32; packet data = 16 bytes;
     captured_len = 16 (= 48 - 32). The EPB payload is exactly 16 bytes of Ethernet frame
     (no padding bytes needed when captured_len is a multiple of 4).
2. The user runs `wirerust analyze epb_boundary_valid.pcapng --json`.
3. The tool exits 0. One packet is ingested. No error. This confirms the >= check uses
   the correct constant (32) and does not off-by-one reject a valid boundary case.

### Case D — captured_len = block_total_length - 31 (INVALID by 1) → E-INP-008

1. A crafted pcapng file is presented where the EPB's captured_len field in the raw bytes
   is set to (block_total_length - 31), i.e., one byte more than the maximum valid
   captured_len. For example: block_total_length = 48; captured_len = 17 (= 48 - 31).
   The field is deliberately set to exceed the (block_total_length - 32) ceiling. The crate
   successfully frames the block (btl = 48 >= 12, aligned); wirerust body-decode then
   rejects captured_len via the bound-by-body or padding-aware check.
2. The user runs `wirerust analyze epb_boundary_invalid.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr. No data allocation is performed
   before the check (SEC-004: guard precedes allocation). No panic. Error consistent with
   **E-INP-008** (wirerust body-decode failure — crate framed the block; wirerust rejects
   the body content).

### Case E — captured_len NOT a multiple of 4, padding would exceed body → E-INP-008

This case exercises the padding-aware bound check — distinct from Cases C and D which
use `captured_len` that is already a multiple of 4 (no padding needed). When
`captured_len` is not a multiple of 4, the crate pads the data slice to the next 4-byte
boundary. The guard must account for this padding: the total bytes consumed is
`ceil(captured_len / 4) * 4`, not just `captured_len`. If `captured_len` passes the
`<= btl-32` check but `captured_len + padding_bytes` exceeds the available body, the
implementation must return **E-INP-008** (wirerust body-decode failure — crate already
framed the block with btl >= 12; wirerust body-decode discovers the padding overrun) —
not proceed to an out-of-bounds slice or panic.

**Fixture layout (total block = 48 bytes, captured_len ≡ 3 mod 4):**

- `block_total_length = 48` (LE: `30 00 00 00`), same as Case C.
- `captured_len = 15` (LE: `0F 00 00 00`). Note: `15 <= 48 - 32 = 16` — the naive
  `<= btl-32` check PASSES. However, `15 % 4 = 3`, so padding = 1 byte. The padded
  data extent = `15 + 1 = 16` bytes. Total body bytes consumed = 20 (fixed fields) +
  16 (padded data) = 36. But the block body is `48 - 12 = 36` bytes total, with 20
  bytes fixed fields, leaving only 16 bytes for padded data — so `15 + 1 = 16` fits
  exactly. This is a VALID case.

To make this adversarial, use `captured_len = 19` (LE: `13 00 00 00`) instead:
- `19 <= 48 - 32 = 16` — this FAILS the naive check and is caught by Case D. Not useful.

Use a larger `block_total_length` to give more room. **Revised fixture:**

- `block_total_length = 52` (LE: `34 00 00 00`).
- Available body = `52 - 12 = 40` bytes. Fixed fields = 20 bytes. Available for padded
  data = `40 - 20 = 20` bytes.
- `captured_len = 19` (LE: `13 00 00 00`). `19 <= 52 - 32 = 20` — passes naive check.
  But `19 % 4 = 3`, so padding = 1 byte. Padded extent = `19 + 1 = 20` bytes.
  This fits exactly in the 20 available bytes — again a valid case.
- To force an over-run: `captured_len = 19` with `block_total_length = 48` (body = 36,
  data zone = 16 bytes). `19 <= 48 - 32 = 16` is FALSE — Case D rejects it before
  padding matters.

**Correct adversarial construction** for a non-mult-of-4 over-run:
- `block_total_length = 52` (body = 40, data zone = 20).
- `captured_len = 19`: naive check: `19 <= 20` PASSES. Padded: `19 + 1 = 20 <= 20` — VALID.
- `captured_len = 21` (LE: `15 00 00 00`): naive check: `21 <= 20` FAILS. Rejected by Case D.

The padding-aware boundary fault only manifests when the block is constructed so that
`captured_len <= btl-32` (passes the raw-len check) BUT the aligned extent
`ceil(captured_len/4)*4 > (btl - 32)` (exceeds the padded slot). Concretely:

- `block_total_length = 52`: data zone = 20 bytes.
- `captured_len = 17` (≡ 1 mod 4): naive check `17 <= 20` PASSES. Padded = `17 + 3 = 20`: fits.
- `captured_len = 21` (≡ 1 mod 4): naive check `21 <= 20` FAILS (Case D).

True conflict: `captured_len = 19` (≡ 3 mod 4) with `block_total_length = 48`:
- `block_total_length = 48`, data zone = `48 - 32 = 16` bytes.
- Naive check: `19 <= 16` FAILS — caught by raw-len guard, not the padding path.

The only way to reach the padding-sensitive path is when the raw check passes but the
padded size overflows. This requires `captured_len <= btl-32` but
`captured_len + ((4 - captured_len%4) % 4) > btl-32`.

For `btl-32 = N`, need `captured_len <= N` and `captured_len + pad > N`.
If `captured_len = N` and `N % 4 != 0`, then `pad > 0`, so `N + pad > N`. VALID path.

**Concrete fixture: `block_total_length = 47` (not 4-aligned), `captured_len = 15`.**
- `block_total_length = 47` (LE: `2F 00 00 00`).
- Data zone raw: `47 - 32 = 15` bytes.
- `captured_len = 15` (LE: `0F 00 00 00`). Naive check: `15 <= 15` PASSES.
- `15 % 4 = 3`, pad = 1. Padded extent = 16 > 15: **OVER-RUNS the data zone**.
- wirerust MUST return **E-INP-008** (wirerust body-decode failure — padding overrun;
  the crate framed the block, wirerust body-decode rejects the padded extent).
  No panic. No out-of-bounds slice access.

Note: `block_total_length = 47` is not 4-byte aligned itself, which is already a
structural violation per pcapng spec (all block lengths must be multiples of 4). A
conforming implementation may reject this before even checking `captured_len`, also
yielding E-INP-010. Either rejection path is acceptable — the key observable is:
exit non-zero, error on stderr, no panic.

**EPB body (body-relative bytes, block_total_length=47):**
```
interface_id:   00 00 00 00   # interface 0
ts_high:        00 00 00 00
ts_low:         00 00 00 00
captured_len:   0F 00 00 00   # 15 decimal
original_len:   0F 00 00 00   # 15 decimal
packet_data:    [15 bytes, e.g. AA BB CC DD EE FF 00 11 22 33 44 55 66 77 88]
(no trailing total_len — block is truncated/malformed at byte 47)
```

1. A crafted pcapng file is presented: SHB (28 bytes, LE) + IDB (20 bytes, LE, linktype=1,
   snaplen=65535) + EPB as above (47 bytes, captured_len=15 with non-4-aligned btl). Total
   file: 28 + 20 + 47 = 95 bytes.
2. The user runs `wirerust analyze epb_nonmult4_boundary.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr. No allocation of a 15-byte
   slice beyond the available data zone occurs (SEC-004). No panic, no index OOB backtrace.
   Error consistent with **E-INP-008** (wirerust body-decode failure — padding-aware bound
   exceeded or structurally invalid block_total_length not a multiple of 4; either way
   the crate returned a block or rejected it as framing-invalid → E-INP-010, but if the
   crate accepted btl=47 and delivered the body, wirerust body-decode discovers the
   padding overrun → E-INP-008). Either E-INP-008 or E-INP-010 is acceptable — the
   key observable is: exit non-zero, error on stderr, no panic.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.012 | Postcondition 5 (empty-table clause) — EPB with interface_id referencing empty table → E-INP-009 | Case A: the explicit empty-table path |
| BC-2.01.012 | Postcondition 5 (OOB-on-non-empty clause) — EPB with interface_id OOB on non-empty table → E-INP-010 | Case B: the non-empty OOB path; distinct error code from Case A |
| BC-2.01.012 | Postcondition 3 — EPB packet data bounded by captured_len; valid boundary accepted | Case C: off-by-one check: btl-32 MUST be accepted |
| BC-2.01.012 | Postcondition 6a/6b — EPB captured_len > block_total_length - 32 → E-INP-008 (wirerust body-decode failure; crate framed the block) | Case D: btl-31 MUST be rejected (one byte over the valid boundary); error is E-INP-008 |
| BC-2.01.012 | Postcondition 6b — EPB padding-aware bound: captured_len passes raw check but padded extent exceeds data zone → E-INP-008 (wirerust body-decode failure; crate framed the block) | Case E: captured_len ≡ 3 mod 4, raw check passes, padded size overflows; no panic/OOB |
| BC-2.01.012 | SEC-004 AC — guard-before-allocate: captured_len check precedes data allocation | Cases D and E: no allocation before the check fires |
| BC-2.01.012 | No-panic AC (SEC-005) — Err returned for all invalid inputs | All cases: no panic permitted on any case |

## Verification Approach

```
wirerust analyze epb_no_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr (E-INP-009 or equivalent message), no JSON on stdout.

```
wirerust analyze epb_oob_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr mentioning interface_id out of range or interface
table size (E-INP-010 or equivalent). The error message MUST NOT be a raw Rust panic backtrace
(`index out of bounds: the len is 1 but the index is 4294967295`).

```
wirerust analyze epb_boundary_valid.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, one packet in JSON output (total_packets = 1).

```
wirerust analyze epb_boundary_invalid.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr, no JSON on stdout.

```
wirerust analyze epb_nonmult4_boundary.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with E-INP-008 (wirerust body-decode:
padding-aware bound exceeded) or E-INP-010 (if crate rejects btl=47 as not 4-aligned,
i.e., crate framing rejection). Either is acceptable — see Case E step 3. No JSON on
stdout. No panic, no Rust backtrace.

## Evaluation Rubric

- **Security / no-panic** (weight: 0.35): Cases A, B, D, E — no unchecked Vec index panic;
  all invalid inputs produce graceful errors, not Rust backtraces.
- **Functional correctness** (weight: 0.25): Case C accepted at the valid boundary (exit 0,
  one packet); confirms the guard uses the correct constant (32, not 31).
- **Error specificity** (weight: 0.20): Cases A and B produce distinct error signals —
  A is empty-table (no IDB seen yet), B is OOB-on-non-empty (IDB present but interface_id
  beyond the table size). Ideally these are distinguishable on stderr; at minimum both must
  be non-zero exit + readable error.
- **Guard ordering** (weight: 0.10): Cases D and E — the error must be returned BEFORE any
  data allocation. Observable via exit 0 absence and no partial JSON output.
- **Padding-aware bound** (weight: 0.10): Case E — the guard accounts for
  `ceil(captured_len/4)*4` (padded extent), not just the raw `captured_len`. A naive
  `captured_len <= btl-32` check without padding compensation would silently proceed to
  an out-of-bounds slice or panic on this fixture.

## Edge Conditions

- u32::MAX (4,294,967,295) as interface_id is the adversarial maximum for this field.
  A naive cast to usize on a 32-bit platform would fail at Vec indexing; on a 64-bit
  platform it would also fail (table has 0 or 1 entries). The check must be performed as
  `interface_id as usize >= table.len()` (or equivalent), not as a signed comparison.
- The empty-table case (Case A) must produce E-INP-009, not E-INP-010. The distinction
  matters: E-INP-009 means "no IDB at all before this EPB" (a structural file error);
  E-INP-010 means "IDB present but the referenced index is out of range" (a different
  structural error). The test confirms both paths exist independently.
- Case C uses a captured_len that is a multiple of 4 (16 bytes) to avoid padding
  complications. The captured_len guard is `captured_len <= block_total_length - 32`,
  so 16 <= 48 - 32 = 16: exactly equal, which must be accepted (the condition is <=, not <).
- Case E isolates the padding-aware bound: a captured_len that passes the raw
  `<= btl-32` check but whose aligned extent (`ceil(captured_len/4)*4`) exceeds the
  available data zone. This is a distinct failure mode from Case D (raw-len overflow)
  and from Case C (valid boundary). The fixture uses `block_total_length = 47` (not
  4-aligned), `captured_len = 15`, giving padded extent = 16 > raw zone = 15. The
  evaluator accepts either a "padding overflow" error (E-INP-008, wirerust body-decode
  failure after crate delivers the block) or a "block length not 4-aligned" error
  (E-INP-010, crate framing rejection of btl=47). Both indicate a structural malformation;
  either exit-non-zero path is acceptable. If the crate accepts btl=47, the expected
  code is E-INP-008.

## Fixture Construction Note

All four EPB fixtures require careful byte-level construction:
- EPB body layout (raw-block path, body-relative): interface_id[4] + ts_high[4] + ts_low[4]
  + captured_len[4] + original_len[4] = 20 bytes, followed by packet data + padding to 4-byte
  alignment.
- block_total_length = 12 (outer header: type[4] + total_len[4] + trailing_len[4]) +
  20 (body fixed) + captured_len (rounded up to 4-byte alignment). For Case C: 12+20+16=48.
- Case D: same layout as Case C but captured_len field set to 17 in the raw bytes (even
  though actual data present is only 16 bytes); block_total_length remains 48. This produces
  captured_len (17) > block_total_length - 32 (16): the invalid boundary.

## Failure Guidance

"HOLDOUT LOW: HS-104 (satisfaction: 0.XX) — EPB framing guards have defects.
Case A panic (index OOB on empty slice) means empty-table check absent.
Case B panic (index OOB: len=1, index=4294967295) means OOB check absent.
Case C exit non-zero means off-by-one in the captured_len guard (should be <= not <).
Case D exit 0 means captured_len guard is absent or fires after allocation. Expected error: E-INP-008 (wirerust body-decode failure).
Case E exit 0 or panic means the padding-aware bound is absent: the guard checks raw captured_len but does not account for ceil(captured_len/4)*4 (padded extent). A 15-byte slice with 1 pad byte reads 16 bytes from a 15-byte zone — silent OOB or panic. Expected error: E-INP-008 (wirerust body-decode: padding-overrun) or E-INP-010 (crate framing rejection if btl=47 is rejected as not 4-aligned).
See BC-2.01.012, VP-027, ADR-009 Decision 2 (EPB section), SEC-004."
