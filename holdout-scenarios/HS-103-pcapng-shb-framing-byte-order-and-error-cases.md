---
document_type: holdout-scenario
level: ops
version: "1.5"  # Pass-4 R4 / ADR-009 rev 7 Decision 20: added Case D — btl=16 (aligned, crate frames, body=4 < 16 SHB fixed-fields) → E-INP-008 (body-too-short, NOT the framing E-INP-010 path). This is the case pass-3 wrongly removed. The btl<12/misaligned → E-INP-010 case (Case C) and the semantic (bad BOM/major) → E-INP-008 cases (Case B) are retained. Prior version history: v1.4 H-1 (Pass-3 / ADR-009 rev 6): verified E-INP-008 / E-INP-010 split. v1.3 I-8 fix corrected Case C from E-INP-008 to E-INP-010. v1.2 BOM-3 fix corrected Case A BOM on-disk bytes.
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-103"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.010
verification_properties:
  - VP-026
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng SHB Framing — Big-Endian Byte-Order Magic, Invalid BOM, and Truncated SHB

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The Section Header Block (SHB) is the mandatory first block of every pcapng file. Its
Byte-Order Magic (BOM) field at bytes 8-11 of the SHB body (offset 12-15 from file start)
determines whether the file is little-endian or big-endian. This scenario tests three
SHB-level boundary conditions that exercise the byte-order detection path and the two
mandatory error returns.

### Case A — Byte-exact BE-magic SHB (0x1A2B3C4D)

1. A crafted pcapng file is presented in big-endian format:
   - File starts with block_type 0x0A0D0D0A (same in both endians — this field is
     endian-neutral by pcapng spec design)
   - block_total_length field: u32 value 0x0000001C (= 28 bytes, minimum SHB).
     Wire bytes in a BE-encoded section: `00 00 00 1C` (big-endian u32).
     (For reference, the same field in an LE-encoded section would be `1C 00 00 00`.)
   - Byte-Order Magic (BOM): the u32 value 0x1A2B3C4D stored big-endian; on-disk bytes `1A 2B 3C 4D`.
     (For reference, a little-endian section stores the same logical value as `4D 3C 2B 1A` on disk.)
   - major_version: 0x0001 (BE), minor_version: 0x0000 (BE), section_length: 0xFFFFFFFFFFFFFFFF (BE, -1 = unknown)
   - Followed by one IDB (block_type=0x00000001 in BE) with linktype=1 (BE) and no options
   - Followed by one EPB (block_type=0x00000006 in BE) with a minimal Ethernet frame
2. The user runs `wirerust analyze be_shb.pcapng --json`.
3. The tool exits 0. One packet is ingested. The evaluator confirms the tool accepted the
   big-endian file without error — the BOM was recognized and all subsequent block fields
   decoded in big-endian byte order.

### Case B — SHB with invalid Byte-Order Magic → Err

1. A crafted file is presented where the first block begins with block_type 0x0A0D0D0A
   (the pcapng SHB magic), valid block_total_length=28, but the BOM field contains
   0xDEADBEEF (neither 0x1A2B3C4D nor 0x4D3C2B1A).
2. The user runs `wirerust analyze invalid_bom.pcapng --json`.
3. The tool exits non-zero. An error is printed to stderr. No packets are emitted. No panic
   occurs. The error message indicates the file could not be parsed (it need not expose the
   specific byte values, but must not be a bare panic backtrace).

### Case C — SHB truncated at byte 15 (total block < 28 bytes, crate cannot frame) → Err with E-INP-010

1. A file is presented that contains only 15 bytes total — fewer than the minimum an SHB
   requires. The pcap-file crate cannot frame the block (block_total_length cannot even be
   fully read), so it returns `Err` before wirerust body-decode code runs. wirerust maps
   this crate-level framing error to **E-INP-010** (truncated/unframed block).
2. The user runs `wirerust analyze truncated_shb.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr referencing a truncated or
   malformed file. The error is consistent with E-INP-010 (not E-INP-008). No packets are
   emitted. No panic occurs.

### Case D — SHB with btl=16 (aligned, crate frames, body=4 < 16 SHB fixed-fields) → Err with E-INP-008

This is the Decision 20 case that pass-3 wrongly removed. `block_total_length = 16` means
the crate CAN frame the block (outer header: 12 bytes; body: 4 bytes — enough for the crate
to deliver a `RawBlock` to wirerust). However, the SHB body is only 4 bytes, which is below
the 16-byte minimum required for the SHB fixed fields (BOM:4 + major:2 + minor:2 +
section_length:8 = 16). wirerust's body-decode code receives a 4-byte body slice and MUST
return **E-INP-008** (body-too-short), not E-INP-010 (which is a crate-level framing
failure). This path is distinct from Case C because the crate successfully delivers a
`RawBlock` — the error is at the wirerust body-decode layer, not the crate framing layer.

**Byte-exact file layout (total = 28 bytes):**
```
SHB outer header (12 bytes, LE):
  block_type:         0A 0D 0D 0A   # SHB magic (endian-neutral)
  block_total_length: 10 00 00 00   # 16 decimal (LE u32)
  [body: 4 bytes of whatever follows — cannot be a valid BOM+fields]
  [body bytes, e.g.]: DE AD BE EF   # 4 bytes: crate delivers this as body
  trailing_total_len: 10 00 00 00   # 16 decimal (LE u32)
```
Total block bytes: 12 outer + 4 body = 16. File may contain ONLY this 16-byte block.

Note on endianness: `block_total_length = 16` is written in LE (`10 00 00 00`) because
the crate reads the SHB BTL before it knows endianness. The SHB magic `0x0A0D0D0A` is
endian-neutral. The 4-byte body cannot be a valid BOM and cannot fill any of the required
SHB fixed fields (the BOM alone is 4 bytes, but major+minor+section_length need 12 more).

1. A crafted 16-byte pcapng file is presented with the layout above. The crate frames the
   block and delivers a 4-byte body to wirerust.
2. The user runs `wirerust analyze shb_btl16.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr. The error is consistent with
   **E-INP-008** (body too short for SHB fixed fields — wirerust body-decode layer). The
   error is NOT E-INP-010 (which would indicate a crate framing failure). No packets are
   emitted. No panic occurs.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.010 | Postcondition 1 — SHB with LE BOM accepted | (Baseline; covered by Cases A/B/C/D setup infrastructure) |
| BC-2.01.010 | Postcondition 2 — SHB with BE BOM (on-disk `1A 2B 3C 4D`, u32 value 0x1A2B3C4D) accepted; subsequent fields decoded BE | Case A: the BE-magic path must be recognized and used |
| BC-2.01.010 | Postcondition 3 — SHB with invalid BOM returns Err(E-INP-008) | Case B: invalid BOM must produce E-INP-008 (semantic body-decode failure), not a panic or silent wrong-decode |
| BC-2.01.010 | Postcondition 4 / E-INP-010 — SHB block_total_length < 12 (crate can't frame block) returns Err(E-INP-010) | Case C: 15-byte file can't be framed by crate → crate Err → wirerust E-INP-010 (not E-INP-008, which requires a successfully-framed body) |
| BC-2.01.010 | Postcondition 4 / E-INP-008 — SHB btl=16 (crate frames, body=4 < 16 SHB fixed-fields) returns Err(E-INP-008) | Case D: Decision 20 body-too-short path: crate delivers 4-byte body; wirerust body-decode rejects it as E-INP-008, NOT E-INP-010 |
| BC-2.01.010 | No-panic invariant (SEC-005 AC) — must return Err for any malformed/truncated input | All cases: no panic permitted |

## Verification Approach

```
wirerust analyze be_shb.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, JSON output, total_packets >= 1.

```
wirerust analyze invalid_bom.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error text on stderr, no JSON on stdout.

```
wirerust analyze truncated_shb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error text on stderr referencing a truncated or malformed block
(E-INP-010 — crate-level framing failure; NOT E-INP-008 which is a body-decode error
on a successfully-framed-but-too-short SHB body), no JSON on stdout.

```
wirerust analyze shb_btl16.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, error on stderr consistent with **E-INP-008** (body too short for
SHB fixed fields — crate successfully framed the 16-byte block and delivered a 4-byte body,
but wirerust body-decode rejects it). NOT E-INP-010 (which is the crate framing path). No
JSON on stdout. No panic.

For Case A, an additional check: if the BE-decoded EPB linktype is Ethernet (1), and a
minimal valid Ethernet frame is present, the packet should appear in the output with
the correct frame type. A linktype mismatch (e.g., if the tool misreads BE linktype=1
as LE 0x01000000 = 16777216) would produce a linktype-unsupported error — this would
indicate the BE byte-order path is broken.

## Evaluation Rubric

- **Functional correctness** (weight: 0.40): Case A accepted and packet decoded; Cases B,
  C, and D rejected with non-zero exit.
- **No-panic safety** (weight: 0.30): No panic for any of the four inputs — including
  the adversarial invalid-BOM case and the btl=16 body-too-short case.
- **Error quality / E-INP split** (weight: 0.20): Cases B and D produce E-INP-008 (body-
  decode layer); Case C produces E-INP-010 (crate framing layer). The distinction between
  these two error codes is the primary Decision 20 assertion. Stderr messages must not be
  raw Rust backtraces.
- **BE byte-order correctness** (weight: 0.10): In Case A, the linktype is correctly
  decoded from BE bytes, enabling packet dispatch to the Ethernet decoder path.

## Edge Conditions

- BE-magic files are rare in practice but required by the pcapng specification and
  used by some non-Wireshark capture tools. The SHB byte-order magic is the ONLY
  reliable way to determine file endianness.
- The block_type field 0x0A0D0D0A is endian-neutral: it reads the same in both LE and
  BE because of its byte symmetry. The BOM field immediately distinguishes LE from BE.
- SHB minimum byte count: 28 (12-byte outer header + 16-byte body: BOM:4 + major:2 +
  minor:2 + section_length:8). A block_total_length < 28 is structurally invalid
  independently of whether the BOM is valid.
- A file with a valid pcapng outer magic (0x0A0D0D0A) but invalid BOM is a distinct
  error category from a completely unknown file format. Both result in error, but the
  error path is different.
- **E-INP-008 vs E-INP-010 boundary (Decision 20):** The crate frames a block when
  `block_total_length >= 12` and the declared length matches the bytes available. For
  `btl = 16`, the crate delivers a 4-byte body; wirerust receives a valid `RawBlock`
  and must check `body.len() >= 16` (SHB fixed-fields minimum). Failing that check
  is **E-INP-008** (body-decode error). For `btl < 12` (e.g., a 15-byte file), the
  crate cannot frame the block at all — the crate returns `Err` before wirerust sees
  a body; wirerust maps that to **E-INP-010** (framing error). These two paths MUST
  NOT be conflated: E-INP-008 requires a successfully-framed block; E-INP-010 does not.

## Failure Guidance

"HOLDOUT LOW: HS-103 (satisfaction: 0.XX) — pcapng SHB framing has defects.
Case A failure (exit non-zero) indicates BE byte-order detection is absent or broken.
Case B failure (exit 0 or panic) indicates invalid BOM is not being rejected (E-INP-008 path missing).
Case C failure (exit 0, panic, or wrong error code E-INP-008 instead of E-INP-010) indicates the crate-level framing-error path is not being mapped correctly; a 15-byte file cannot provide a full SHB outer header and must map to E-INP-010, not E-INP-008.
Case D failure (exit 0, panic, or wrong error code E-INP-010 instead of E-INP-008) indicates the body-too-short path is missing or conflated with the crate framing path. btl=16 means the crate successfully delivers a 4-byte body; wirerust body-decode MUST check body.len() >= 16 and return E-INP-008 — NOT E-INP-010.
See BC-2.01.010, VP-026, ADR-009 Decision 2 and Decision 20."
