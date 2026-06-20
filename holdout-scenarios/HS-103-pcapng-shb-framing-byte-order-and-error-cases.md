---
document_type: holdout-scenario
level: ops
version: "1.3"  # I-8 fix: corrected Case C expected error code from E-INP-008 to E-INP-010. A block_total_length < 28 (or a 15-byte file where the crate cannot frame the block) is rejected by the pcap-file crate before wirerust body-decode runs — the crate returns Err, which wirerust maps to E-INP-010 (unframed/truncated block). E-INP-008 applies only when the crate successfully frames the SHB body but the body is < 28 bytes of fixed fields — a distinct path that requires block_total_length >= 12 but body < 28. BOM-3 fix (v1.2): corrected Case A BOM on-disk bytes from 4D 3C 2B 1A (LE) to 1A 2B 3C 4D (BE); corrected case title and prose that misnamed 0x4D3C2B1A as the BE sentinel
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

### Case C — SHB truncated at byte 15 (total block < 28 bytes) → Err with E-INP-010

1. A file is presented that contains only 15 bytes total — fewer than the minimum an SHB
   requires. The pcap-file crate cannot frame the block (block_total_length cannot even be
   fully read), so it returns `Err` before wirerust body-decode code runs. wirerust maps
   this crate-level framing error to **E-INP-010** (truncated/unframed block). Note: a
   block_total_length = 16 case (12-byte outer header + 4 bytes of body) would also be
   rejected as structurally invalid, and also maps to E-INP-010 since the crate-level
   framing check fires. E-INP-008 applies only when the crate successfully frames an SHB
   body but that body is < 16 fixed-bytes wide (BOM:4 + major:2 + minor:2 + section_len:8)
   — which requires block_total_length >= 12 but body < 28. That distinct path is NOT
   exercised by a 15-byte file.
2. The user runs `wirerust analyze truncated_shb.pcapng --json 2>&1`.
3. The tool exits non-zero. An error is printed to stderr referencing a truncated or
   malformed file. The error is consistent with E-INP-010 (not E-INP-008). No packets are
   emitted. No panic occurs.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.010 | Postcondition 1 — SHB with LE BOM accepted | (Baseline; covered by Cases A/B/C setup infrastructure) |
| BC-2.01.010 | Postcondition 2 — SHB with BE BOM (on-disk `1A 2B 3C 4D`, u32 value 0x1A2B3C4D) accepted; subsequent fields decoded BE | Case A: the BE-magic path must be recognized and used |
| BC-2.01.010 | Postcondition 3 — SHB with invalid BOM returns Err | Case B: invalid BOM must produce a graceful error, not a panic or silent wrong-decode |
| BC-2.01.010 | Postcondition 4 / E-INP-010 — SHB block_total_length < 12 (crate can't frame block) returns Err(E-INP-010) | Case C: 15-byte file can't be framed by crate → crate Err → wirerust E-INP-010 (not E-INP-008, which requires a successfully-framed body) |
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

For Case A, an additional check: if the BE-decoded EPB linktype is Ethernet (1), and a
minimal valid Ethernet frame is present, the packet should appear in the output with
the correct frame type. A linktype mismatch (e.g., if the tool misreads BE linktype=1
as LE 0x01000000 = 16777216) would produce a linktype-unsupported error — this would
indicate the BE byte-order path is broken.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Case A accepted and packet decoded; Cases B
  and C rejected with non-zero exit.
- **No-panic safety** (weight: 0.30): No panic for any of the three inputs — including
  the adversarial invalid-BOM case.
- **Error quality** (weight: 0.15): Cases B and C produce readable error messages on
  stderr; the message must not be a raw Rust backtrace or `thread 'main' panicked`.
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

## Failure Guidance

"HOLDOUT LOW: HS-103 (satisfaction: 0.XX) — pcapng SHB framing has defects.
Case A failure (exit non-zero) indicates BE byte-order detection is absent or broken.
Case B failure (exit 0 or panic) indicates invalid BOM is not being rejected.
Case C failure (exit 0, panic, or wrong error code E-INP-008 instead of E-INP-010) indicates the crate-level framing-error path is not being mapped correctly; a 15-byte file cannot provide a full SHB outer header and must map to E-INP-010.
See BC-2.01.010, VP-026, and ADR-009 Decision 2."
