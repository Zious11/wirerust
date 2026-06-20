---
document_type: holdout-scenario
level: ops
version: "1.0"
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

### Case D — captured_len = block_total_length - 31 (INVALID by 1) → E-INP-010

1. A crafted pcapng file is presented where the EPB's captured_len field in the raw bytes
   is set to (block_total_length - 31), i.e., one byte more than the maximum valid
   captured_len. For example: block_total_length = 48; captured_len = 17 (= 48 - 31).
   The field is deliberately set to exceed the (block_total_length - 32) ceiling.
2. The user runs `wirerust analyze epb_boundary_invalid.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr. No data allocation is performed
   before the check (SEC-004: guard precedes allocation). No panic. Error consistent with
   E-INP-010.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.012 | Postcondition 3 — EPB with interface_id referencing empty table → E-INP-009 | Case A: the explicit empty-table path |
| BC-2.01.012 | Postcondition 4 — EPB with interface_id OOB on non-empty table → E-INP-010 | Case B: the non-empty OOB path; distinct error code from Case A |
| BC-2.01.012 | Postcondition 2 — EPB captured_len at valid boundary accepted | Case C: off-by-one check: btl-32 MUST be accepted |
| BC-2.01.012 | Postcondition 5 — EPB captured_len > block_total_length - 32 → E-INP-010 | Case D: btl-31 MUST be rejected (one byte over the valid boundary) |
| BC-2.01.012 | SEC-004 AC — guard-before-allocate: captured_len check precedes data allocation | Case D: no allocation before the check fires |
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

## Evaluation Rubric

- **Security / no-panic** (weight: 0.40): Cases A, B, D — no unchecked Vec index panic;
  all invalid inputs produce graceful errors, not Rust backtraces.
- **Functional correctness** (weight: 0.30): Case C accepted at the valid boundary (exit 0,
  one packet); confirms the guard uses the correct constant (32, not 31).
- **Error specificity** (weight: 0.20): Cases A and B produce distinct error signals —
  A is empty-table (no IDB seen yet), B is OOB-on-non-empty (IDB present but interface_id
  beyond the table size). Ideally these are distinguishable on stderr; at minimum both must
  be non-zero exit + readable error.
- **Guard ordering** (weight: 0.10): Case D — the error must be returned BEFORE any data
  allocation. Observable via exit 0 absence and no partial JSON output.

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
Case D exit 0 means captured_len guard is absent or fires after allocation.
See BC-2.01.012, VP-027, ADR-009 Decision 2 (EPB section), SEC-004."
