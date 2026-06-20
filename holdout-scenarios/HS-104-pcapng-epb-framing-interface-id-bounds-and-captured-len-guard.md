---
document_type: holdout-scenario
level: ops
version: "1.5"  # Pass-9 (LOW-2 + LOW-3): Case E downgraded — fixture uses non-4-aligned btl=47 which the crate rejects as E-INP-010 BEFORE PC6b (padding-overrun) can run. Case E now asserts NO-PANIC / graceful-Err; E-INP-010 (crate alignment rejection) is the expected primary path; PC6b (padding-overrun → E-INP-008) is noted as DEFENSE-IN-DEPTH / unreachable on a well-framed block per BC-2.01.012 PC6b. BC Linkage table, Evaluation Rubric, Edge Conditions, Failure Guidance, and Verification Approach updated to match. BC-2.01.012 v1.8 now carries explicit PC6a/PC6b anchor labels; this file's PC6a/PC6b citations now resolve.
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
vulnerability (out-of-bounds Vec access). This scenario exercises five boundary conditions
on the EPB parser: two named interface_id discriminant cases (with exact required error
codes), one valid captured_len boundary, and two invalid captured_len cases that must be
rejected before any allocation.

The two interface_id cases MUST produce DIFFERENT error discriminants. Any implementation
that returns the same error code for both the empty-table path and the OOB-on-non-empty
path fails this holdout, regardless of whether both paths exit non-zero.

### Case (empty) — EPB interface_id=0, zero IDBs → E-INP-009 EXACTLY

This case tests the empty-table discriminant. The error code MUST be E-INP-009 — not
E-INP-010, not a generic parse error.

1. A crafted pcapng file is presented containing:
   - SHB (LE, 28 bytes)
   - NO IDB — the interface table is empty (zero entries) when the EPB arrives
   - EPB with `interface_id = 0` (LE: `00 00 00 00`) and a minimal 14-byte Ethernet payload.
     `interface_id = 0` is chosen deliberately (the most natural first index) to confirm the
     empty-table check fires even for the "plausible" index 0 — not just for out-of-range values.
2. The user runs `wirerust analyze epb_no_idb.pcapng --json 2>&1`.
3. The tool exits non-zero. Stderr MUST contain an error message consistent with
   E-INP-009. The message MUST convey that the interface table was empty (no IDB has been
   parsed), for example:
   `"EPB references interface_id=0 but interface table is empty — no IDB has been parsed"`
   (or a substring thereof). No panic. No crash. The error MUST NOT be E-INP-010.

### Case (OOB) — EPB interface_id=5, 1-entry table → E-INP-010 EXACTLY

This case tests the OOB-on-non-empty-table discriminant. The error code MUST be E-INP-010 —
not E-INP-009, not a generic parse error. This case is structurally distinct from
Case (empty): the table is non-empty (one IDB parsed), but the referenced index is beyond
the table size.

1. A crafted pcapng file is presented containing:
   - SHB (LE, 28 bytes)
   - ONE IDB with linktype=1 (Ethernet), if_tsresol=6 — table now has exactly one entry at
     index 0
   - EPB with `interface_id = 5` (LE: `05 00 00 00`) and a minimal 14-byte Ethernet payload.
     `interface_id = 5` with a 1-entry table (valid range: only index 0) is a clear OOB on
     a non-empty table.
2. The user runs `wirerust analyze epb_oob_idb.pcapng --json 2>&1`.
3. The tool exits non-zero. Stderr MUST contain an error message consistent with E-INP-010.
   The message MUST convey that the referenced interface_id is out of range relative to the
   known interface table size, for example:
   `"EPB interface_id=5 out of range (table size=1)"`
   (or a substring thereof). No panic. No Vec index panic (which would produce
   `index out of bounds` in the Rust backtrace). The error MUST NOT be E-INP-009.

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

### Case E — non-4-aligned block_total_length: NO-PANIC / graceful-Err (PC6b defense-in-depth)

This case was previously described as exercising the PC6b padding-aware overrun check. That
framing was incorrect. As established by BC-2.01.012 v1.8 PC6b (per pass-9 LOW-2 analysis),
PC6b is UNREACHABLE on a crate-framed (4-aligned) block:

- The crate rejects any `block_total_length` that is not 4-byte aligned before handing the
  block to wirerust, returning a crate `Err` that wirerust maps to **E-INP-010**.
- Therefore the fixture below (btl=47, not 4-aligned) is EXPECTED to be rejected at the
  crate framing layer — E-INP-010 is the expected primary path, not E-INP-008.
- The PC6b guard (padding-overrun → E-INP-008) is defense-in-depth: it would fire only if
  a non-aligned block somehow bypassed the crate alignment gate (e.g., a crate bug or future
  refactoring). It CANNOT be triggered by normal well-framed pcapng input.

**What this case actually tests:** that wirerust exits non-zero with a graceful `Err` (no
panic, no Rust backtrace) on an adversarial non-4-aligned block. It does NOT discriminate
between E-INP-010 (expected crate rejection) and E-INP-008 (hypothetical wirerust body-decode
rejection if PC6b were reachable) — either is acceptable. The PRIMARY observable is safety:
no panic, no OOB slice, no silent success.

**Fixture layout (block_total_length = 47, not 4-byte aligned):**

- `block_total_length = 47` (LE: `2F 00 00 00`).
- `captured_len = 15` (LE: `0F 00 00 00`). If the crate were to deliver this body:
  naive check `15 <= 47 - 32 = 15` PASSES; `15 % 4 = 3`, pad = 1; padded extent = 16 > 15:
  PC6b would fire. But the crate rejects btl=47 as not 4-aligned BEFORE delivering the body,
  so PC6b is never reached. The crate rejection → E-INP-010 is the expected code.

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
3. The tool exits non-zero. An error is printed to stderr. No panic, no index OOB backtrace,
   no Rust `unwrap` failure. **Expected path:** crate alignment rejection → **E-INP-010**
   (block framing error — btl=47 not 4-byte aligned). If the crate accepts btl=47 (unexpected
   path), wirerust body-decode discovers the PC6b padding overrun → **E-INP-008**. Either
   error code is acceptable. The key observables are: exit non-zero, error on stderr, NO
   PANIC. SEC-004: no data allocation before the guard fires. PC6b is DEFENSE-IN-DEPTH per
   BC-2.01.012 PC6b — this case does not claim to exercise the PC6b code path under normal
   crate behavior.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.012 | PC5a — EPB with interface_id evaluated against EMPTY table (zero IDBs) → E-INP-009 EXACTLY (not E-INP-010) | Case (empty): interface_id=0, no IDB → must produce E-INP-009 and no other code |
| BC-2.01.012 | PC5b — EPB with interface_id OOB on NON-EMPTY table (>= 1 IDB, interface_id >= table.len()) → E-INP-010 EXACTLY (not E-INP-009) | Case (OOB): interface_id=5, 1-entry table → must produce E-INP-010 and no other code |
| BC-2.01.012 | AC-001 discriminant split — the two interface_id error paths MUST return DIFFERENT codes | Both named cases together: E-INP-009 != E-INP-010 is a required invariant |
| BC-2.01.012 | Postcondition 3 — EPB packet data bounded by captured_len; valid boundary accepted | Case C: off-by-one check: btl-32 MUST be accepted |
| BC-2.01.012 | PC6a — EPB bound-by-body: captured_len > body.len() → E-INP-008 (wirerust body-decode failure; live reachable guard; crate framed the block) | Case D: btl-31 MUST be rejected (one byte over the valid boundary); error is E-INP-008 |
| BC-2.01.012 | PC6b — EPB padding-aware bound (DEFENSE-IN-DEPTH; unreachable on crate-framed 4-aligned block): non-4-aligned btl rejected by crate as E-INP-010 before PC6b can run; if crate delivers body anyway → E-INP-008 | Case E: fixture uses btl=47 (not 4-aligned); expected path is crate alignment rejection → E-INP-010; PC6b is not exercised under normal crate behavior; observable is NO-PANIC / graceful-Err |
| BC-2.01.012 | SEC-004 AC — guard-before-allocate: captured_len check precedes data allocation | Cases D and E: no allocation before the check fires |
| BC-2.01.012 | No-panic AC (SEC-005) — Err returned for all invalid inputs | All cases: no panic permitted on any case |

## Verification Approach

```
wirerust analyze epb_no_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr MUST be E-INP-009 (empty-table discriminant) — message
must indicate the interface table was empty (no IDB parsed). E-INP-010 is a FAILURE for this
case, as is a generic parse error without a distinguishable code. No JSON on stdout.

```
wirerust analyze epb_oob_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr MUST be E-INP-010 (OOB-on-non-empty discriminant) —
message must mention interface_id=5 out of range relative to table size=1
(`"EPB interface_id=5 out of range (table size=1)"` or equivalent substring). E-INP-009 is a
FAILURE for this case. The error MUST NOT be a raw Rust panic backtrace
(`index out of bounds: the len is 1 but the index is 5`).

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
Expect: non-zero exit, error on stderr, no JSON on stdout, NO PANIC and no Rust backtrace.
**Expected primary path:** crate alignment rejection of btl=47 (not 4-byte aligned) →
E-INP-010. **Acceptable alternative:** if crate delivers the body (unexpected), wirerust
PC6b defense-in-depth fires → E-INP-008. Either error code passes this case. This case does
NOT require discriminating E-INP-008 vs E-INP-010 — the key observable is graceful failure
with no panic. PC6b (padding-overrun → E-INP-008) is DEFENSE-IN-DEPTH per BC-2.01.012 PC6b
and is not exercised by this fixture under normal crate behavior (see pass-9 LOW-2 analysis).

## Evaluation Rubric

- **Security / no-panic** (weight: 0.35): Case (empty), Case (OOB), Case D, Case E — no
  unchecked Vec index panic; all invalid inputs produce graceful errors, not Rust backtraces.
- **Functional correctness** (weight: 0.25): Case C accepted at the valid boundary (exit 0,
  one packet); confirms the guard uses the correct constant (32, not 31).
- **Discriminant correctness** (weight: 0.20): Case (empty) MUST produce E-INP-009 and Case
  (OOB) MUST produce E-INP-010. These are two required distinct codes — a single error code
  used for both paths scores 0 on this dimension even if both exit non-zero. Partial credit
  (0.10) if one case is correct and the other is wrong.
- **Guard ordering** (weight: 0.10): Cases D and E — the error must be returned BEFORE any
  data allocation. Observable via exit 0 absence and no partial JSON output.
- **Padding-aware bound / framing safety** (weight: 0.10): Case E — the fixture uses a
  non-4-aligned btl=47 which the crate MUST reject (alignment framing rejection → E-INP-010)
  before any captured_len or padding arithmetic runs. PC6b (the padding-aware overrun check
  → E-INP-008) is defense-in-depth and is NOT exercised under normal crate behavior (see
  BC-2.01.012 PC6b reachability note). This dimension scores on: exit non-zero, no panic,
  graceful Err — NOT on distinguishing E-INP-008 vs E-INP-010 for this case.

## Edge Conditions

- Case (empty) uses `interface_id=0` (the most natural first index) to confirm the
  empty-table check fires even when the field value is "plausible". The check MUST fire on
  table.len()==0 regardless of the interface_id value. Required code: E-INP-009.
- Case (OOB) uses `interface_id=5` with a 1-entry table (valid range: index 0 only).
  The check must be performed as `interface_id as usize >= table.len()` (or equivalent),
  not as a signed comparison. u32::MAX is also a valid OOB value but is not required by this
  holdout — the discriminant requirement applies to any OOB value on a non-empty table.
  Required code: E-INP-010.
- The discriminant split requirement (E-INP-009 for empty, E-INP-010 for OOB-non-empty)
  is a mandatory behavioral invariant per BC-2.01.012 PC5a/PC5b and AC-001. Implementations
  that collapse both paths into a single code are incorrect regardless of message clarity.
- Case C uses a captured_len that is a multiple of 4 (16 bytes) to avoid padding
  complications. The captured_len guard is `captured_len <= block_total_length - 32`,
  so 16 <= 48 - 32 = 16: exactly equal, which must be accepted (the condition is <=, not <).
- Case E uses `block_total_length = 47` (not 4-byte aligned), `captured_len = 15`. Per
  BC-2.01.012 PC6b (v1.8), the PC6b padding-overrun check is DEFENSE-IN-DEPTH and
  UNREACHABLE on a crate-framed 4-aligned block: the crate rejects btl=47 as not 4-aligned
  before delivering the body, producing E-INP-010 (crate framing rejection). PC6b (→
  E-INP-008) would only fire if the crate delivered a non-aligned body — which normal crate
  behavior does not do. The evaluator accepts either E-INP-010 (expected: crate alignment
  rejection) or E-INP-008 (unexpected: crate passed non-aligned body, PC6b defense-in-depth
  caught it). The required observable is: exit non-zero, error on stderr, no panic — NOT
  a specific discriminating error code. This case does NOT exercise PC6b under normal crate
  behavior.

## Fixture Construction Note

All five EPB fixtures require careful byte-level construction:
- EPB body layout (raw-block path, body-relative): interface_id[4] + ts_high[4] + ts_low[4]
  + captured_len[4] + original_len[4] = 20 bytes, followed by packet data + padding to 4-byte
  alignment.
- Case (empty): SHB (28 bytes) + EPB with interface_id=0x00000000; NO IDB present. The
  interface table is empty. EPB with minimal 14-byte Ethernet payload; block_total_length = 12
  + 20 + 16 = 48.
- Case (OOB): SHB (28 bytes) + IDB (20 bytes, one entry at index 0) + EPB with
  interface_id=0x00000005; block_total_length = 48. The 1-entry table has valid range [0,0];
  interface_id=5 is OOB.
- block_total_length = 12 (outer header: type[4] + total_len[4] + trailing_len[4]) +
  20 (body fixed) + captured_len (rounded up to 4-byte alignment). For Case C: 12+20+16=48.
- Case D: same layout as Case C but captured_len field set to 17 in the raw bytes (even
  though actual data present is only 16 bytes); block_total_length remains 48. This produces
  captured_len (17) > block_total_length - 32 (16): the invalid boundary.

## Failure Guidance

"HOLDOUT LOW: HS-104 (satisfaction: 0.XX) — EPB framing guards have defects.
Case (empty) panic (index OOB on empty slice) means empty-table check is absent — the code indexed Vec without checking table.len()==0.
Case (empty) exit non-zero with E-INP-010 instead of E-INP-009 means the discriminant split is absent — the empty-table and OOB paths are collapsed into a single code; fix by splitting the check as per BC-2.01.012 PC5a.
Case (OOB) panic (index OOB: len=1, index=5) means OOB check is absent on non-empty table.
Case (OOB) exit non-zero with E-INP-009 instead of E-INP-010 means the discriminant split is absent — the same empty-table code is returned for non-empty OOB; fix by splitting the check as per BC-2.01.012 PC5b.
Case C exit non-zero means off-by-one in the captured_len guard (should be <= not <).
Case D exit 0 means captured_len guard is absent or fires after allocation. Expected error: E-INP-008 (wirerust body-decode failure).
Case E exit 0 means the crate accepted the non-4-aligned btl=47 AND wirerust did not reject it — either the crate's alignment gate is absent or wirerust did not propagate the crate error. Expected: exit non-zero. If there is a panic or Rust backtrace, the crate accepted btl=47 AND wirerust's PC6b defense-in-depth guard is absent or the PC6a guard is absent — raw captured_len=15 was used to slice a 15-byte zone without accounting for the 1 pad byte, causing OOB access. Fix: ensure crate alignment rejection is propagated as E-INP-010 (normal path); also code PC6b as a defense-in-depth guard even though it is not operationally reachable under normal crate behavior. See BC-2.01.012 PC6b reachability note (v1.8).
See BC-2.01.012 PC5a/PC5b, AC-001, VP-027, ADR-009 rev 9 Decision F-H4 (discriminant split), Decision 2 (EPB section), SEC-004."
