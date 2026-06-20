---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.015.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-105"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.015
verification_properties:
  - VP-029
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng Block-Walk Skip — Forward Progress at End-of-Stream and DSB Followed by Valid EPB

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The pcapng block-walk loop must correctly skip unknown block types using the
`block_total_length` field and must not consume bytes that belong to subsequent blocks.
This scenario tests two critical forward-progress properties: an unknown block positioned
exactly at the end of the stream (no trailing bytes), and a DSB (Decryption Secrets Block,
type 0x0000000A) followed by a valid EPB — confirming the DSB skip does not misadvance the
cursor and consume the EPB.

### Case A — Unknown block exactly filling end-of-stream

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - IDB with linktype=1 (Ethernet), if_tsresol=6
   - One valid EPB carrying a minimal Ethernet packet (so the file is not empty)
   - One unknown block (block_type=0xDEADBEEF or any value not in
     {0x0A0D0D0A, 0x00000001, 0x00000006, 0x00000003, 0x00000002, 0x00000004,
     0x00000005, 0x00000009, 0x0000000A}) with block_total_length set to exactly
     the remaining bytes in the file — i.e., this unknown block fills the file to EOF
     with no trailing padding.
2. The user runs `wirerust analyze unknown_block_eof.pcapng --json`.
3. The tool exits 0. One packet is in the output (the EPB before the unknown block was
   processed). The unknown block at EOF is silently skipped. No error, no partial output
   corruption, no panic.

### Case B — DSB (block_type=0x0000000A) followed by a valid EPB

The DSB (Decryption Secrets Block) is a known pcapng block type used to carry TLS key
material. By ADR-009 Decision 2, it arrives at wirerust as an unknown block (there is no
`Block::DecryptionSecrets` variant in `pcap-file` 2.0.0; it falls through to `Block::Unknown`
on the high-level API, or on the raw-block path its type is simply not one of the handled
types). The DSB MUST be silently skipped without consuming any bytes from the following EPB.

1. A crafted pcapng file is presented containing:
   - SHB (LE)
   - IDB with linktype=1 (Ethernet), if_tsresol=6
   - DSB block (block_type=0x0000000A, a realistic but minimal body — e.g., 12 bytes of
     fake key material, yielding block_total_length=12+12=24 bytes total). The DSB body
     contents are arbitrary dummy bytes (not real key material).
   - Immediately after the DSB: one valid EPB carrying a minimal Ethernet frame
2. The user runs `wirerust analyze dsb_then_epb.pcapng --json`.
3. The tool exits 0. One packet is in the output. The DSB was silently skipped and the
   EPB was processed correctly. No warning about the DSB appears in any output (stdout
   or stderr). The DSB body bytes do NOT appear in any diagnostic output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.015 | Postcondition 1 — unknown block silently skipped; loop continues | Case A: unknown block at EOF is skipped without error |
| BC-2.01.015 | Postcondition 2 — skip advances past full block_total_length bytes | Case A: the block fills EOF exactly; cursor advances to EOF without reading past it |
| BC-2.01.015 | Postcondition 3 — block immediately following unknown block is processed correctly | Case B: EPB following the DSB is processed; the skip did not over-consume bytes |
| BC-2.01.015 | Invariant — DSB body bytes NOT logged or printed at any log level (SEC-007) | Case B: no DSB-body content in stdout or stderr |
| BC-2.01.015 | Invariant — no panic; break on Err(_) from the crate (Decision 8) | Both cases: no panic; unknown block at EOF triggers crate Err on next read → break |
| BC-2.01.015 | Edge case — block at exact EOF boundary (no trailing bytes) | Case A: EOF is hit cleanly after the unknown block |

## Verification Approach

```
wirerust analyze unknown_block_eof.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, JSON output with total_packets >= 1 (the EPB before the unknown block).
No error on stderr.

```
wirerust analyze dsb_then_epb.pcapng --json
echo "Exit: $?"
```
Expect: exit 0, JSON output with total_packets = 1 (the EPB after the DSB).
No mention of the DSB, its key material, or its block_type in any output stream.

For Case B, the evaluator additionally checks:
- `wirerust analyze dsb_then_epb.pcapng --json 2>&1 | grep -i "decryption\|secrets\|0x0A\|key"` returns
  empty — no DSB-related content is logged (SEC-007 compliance).

## Evaluation Rubric

- **Forward-progress correctness** (weight: 0.40): Case A skips the unknown block without
  error; Case B skips the DSB without consuming EPB bytes.
- **Functional correctness** (weight: 0.30): In both cases, the EPB(s) preceding or
  following unknown blocks are correctly processed and appear in output.
- **Security discipline** (weight: 0.20): Case B — no DSB body bytes in any output at any
  level. DSB carries TLS key material in real captures; leaking it to stdout/stderr would
  be a security defect.
- **No-panic safety** (weight: 0.10): No panic for either input. The end-of-stream after
  an unknown block must not trigger a panic — the crate returns Err on the next read and
  the loop breaks cleanly.

## Edge Conditions

- An unknown block at exact EOF: after the block-walk loop processes this block's
  block_total_length bytes, the next read attempt returns EOF (Err or empty). The loop must
  break cleanly, not retry infinitely or panic. ADR-009 Decision 8 mandates `break on Err(_)`.
- DSB body contents must not be logged at any log level. In a real pcapng capture, the DSB
  body contains TLS session keys (in NSS Key Log format). Logging these would constitute
  key material leakage. The evaluator checks stdout and stderr for any DSB-body fingerprints.
- The DSB block_type 0x0000000A is not in wirerust's handled set (SHB/IDB/EPB/SPB); it
  arrives as an unrecognized type on the raw-block path and is processed by the default
  skip arm of the block-type dispatch.
- Case A uses a completely unknown block_type (not DSB) to test the generic skip path
  independently of the DSB-specific SEC-007 log-guard requirement.

## Fixture Construction Note

Case B DSB fixture: block_type=0x0000000A (LE), block_total_length=24 (12 outer + 12 body),
body = 12 arbitrary bytes (e.g., all zeros or `0x61626364...` dummy data), trailing length=24.
The EPB immediately follows at file offset 24.

## Failure Guidance

"HOLDOUT LOW: HS-105 (satisfaction: 0.XX) — pcapng block-walk skip has defects.
Case A failure (exit non-zero or panic at EOF) indicates the break-on-Err discipline
is absent or the forward-progress advance miscomputes for EOF-boundary blocks.
Case B failure (missing packet) indicates the DSB skip over-consumed bytes from the EPB.
Case B failure (DSB bytes in output) indicates SEC-007 log-guard is absent.
See BC-2.01.015, VP-029, ADR-009 Decision 2 (unknown blocks), Decision 8 (forward-progress)."
