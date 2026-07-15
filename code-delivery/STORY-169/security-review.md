# Security Review — STORY-169 (PR #403)

**Reviewer:** security-reviewer agent
**Date:** 2026-07-14
**PR:** #403 — feat: STORY-169 IEC-104 ASDU header extraction (wave-78)
**Branch:** feature/STORY-169-iec104-asdu-extraction

## Verdict: APPROVE

No CRITICAL or HIGH findings. The `parse_asdu` implementation is correctly guarded against all untrusted-input failure modes.

## Findings

| ID | Severity | CWE | Location | Description | Status |
|----|----------|-----|----------|-------------|--------|
| SEC-001 | LOW | CWE-400 | src/analyzer/iec104.rs:168,171 | `MAX_IEC104_CARRY_BYTES` constant documented in comments but not defined in code; carry buffers `carry_c2s`/`carry_s2c` are declared but never mutated in STORY-169 scope — zero exploit surface in this PR. Must be added before STORY-171 reassembly loop is wired. | Dormant — deferred to STORY-171 as mandatory pre-condition |

## Focus Area Dispositions

### Panic Safety — PASS
`parse_asdu` has one guard at the top (`if asdu_body.len() < 6 { return None; }`). After the guard: bytes [0..=5] are always valid; bytes [6..=8] are gated by `count > 0 && asdu_body.len() >= 9`. No off-by-one. No `unwrap()`, `expect()`, `todo!()`, or `unreachable!()` in the production code path.

### Bounds Checking — PASS
Min-6 guard and min-9 guard are correct and sufficient. `asdu_body[8]` is the highest index accessed; the `len >= 9` guard ensures validity.

### Integer Overflow — PASS
All expressions use `& 0x7F`, `& 0x3F`, `& 0x40`, `& 0x80` (u8 AND, no overflow), `u16::from_le_bytes` and `u32::from_le_bytes` (stdlib, well-defined). The forced `0` high byte bounds `first_ioa` to 24-bit range.

### DoS Surface — PASS
O(1) work and O(1) allocation per call. No loops, no recursion, no heap allocation. At most 9 fixed byte positions accessed regardless of slice length.

### OWASP Top 10 — N/A
Pure-core byte extraction with no I/O, no auth, no SQL, no shell interaction.

## Mandatory Pre-Condition for STORY-171
Before STORY-171 wires reassembly logic, add:
```rust
pub const MAX_IEC104_CARRY_BYTES: usize = 255;
```
And enforce the carry buffer length after every append, following the `MAX_ENIP_CARRY_BYTES` enforcement pattern at `src/analyzer/enip.rs:955`. Failure to do this before wiring the reassembly loop would escalate SEC-001 from LOW to HIGH.
