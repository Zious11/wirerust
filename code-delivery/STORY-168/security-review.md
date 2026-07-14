# Security Review — STORY-168 PR #402

**Date:** 2026-07-14  
**Reviewer:** vsdd-factory:security-reviewer (claude-sonnet-4-6)  
**Verdict: APPROVE WITH NOTES — No CRITICAL or HIGH findings**

## Scope

Functions added to `src/analyzer/iec104.rs` (STORY-168 additions only):
- `classify_frame_format(cf1: u8) -> FrameFormat` — pure-core, 1-byte bitwise dispatch
- `process_u_frame(state: &mut Iec104FlowState, cf1: u8) -> Option<Finding>` — effectful SM
- `Iec104FlowState::session_started: bool` field

Context: passive ICS analyzer processing untrusted network bytes on port 2404.

## Findings

| ID | Severity | CWE | Finding | Disposition |
|----|----------|-----|---------|-------------|
| SEC-001 | MEDIUM | CWE-400/770 | `carry_c2s`/`carry_s2c` fields in `Iec104FlowState` declared with doc comment claiming `MAX_IEC104_CARRY_BYTES` cap, but constant not defined anywhere in codebase. DoS risk (unbounded growth) opens when STORY-171 wires carry buffer. Peers (DNP3, Modbus) define and enforce their carry bound constants. | Deferred to STORY-171 — MUST be an AC before STORY-171 ships. Safe in this PR (no STORY-168 code writes carry buffers). |
| SEC-002 | LOW | CWE-617 | `debug_assert!(classify_frame_format(cf1) == FrameFormat::UFormat)` in `process_u_frame` at lines 263–266 panics on mis-dispatch in debug builds. No-op in `--release`. Not network-triggerable in production. | Accepted-by-design. Documents STORY-173 dispatcher precondition. |
| SEC-003 | LOW | CWE-668 | All `Iec104FlowState` fields are `pub`. `session_started` can be bypassed without going through `process_u_frame`. Not network-exploitable — only accessible by Rust code in the same crate. | Accepted-by-design at this story scope. Field visibility scoping is an arch concern for a later refactor. |
| SEC-004 | INFO | N/A | `"T0881"` string used before STORY-173 catalog entry. `Finding.mitre_techniques: Vec<String>` is plain data — no catalog lookup at construction, no panic risk. | No action required. STORY-173 is the control. |

## Confirmed Non-Findings

| Concern | Result | Reasoning |
|---------|--------|-----------|
| Panic on adversarial CF1 | CLEAN | Exhaustive `match` with `_` wildcard; no unwrap/expect/slice indexing |
| Format string injection (CWE-134) | N/A | `cf1: u8` cannot carry format specifiers; Rust compile-time macro |
| Integer overflow | N/A | No arithmetic in STORY-168 additions |
| Unbounded state (this PR) | CLEAN | Single `bool` field; carry buffers not written |
| Fail-closed `_` arm | CONFIRMED | `state.session_started` not modified in `_` arm; T0814 emitted |
| Pure/effectful boundary (ADR-013 D4) | CONFORMS | `classify_frame_format(cf1: u8)` — no state parameter |
| CVE-2026-1773 reference | INFORMATIONAL | Embedded in analyst-readable `summary`/`evidence` strings only |
