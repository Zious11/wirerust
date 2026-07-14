# Security Review — STORY-167 IEC-104 APCI Core Parser

**PR:** #401  
**Reviewer:** vsdd-factory:security-reviewer  
**Date:** 2026-07-14  
**Verdict: SECURITY REVIEW CLEAN**

---

## Scope

Parser handles untrusted network bytes on port 2404 (ICS/SCADA passive monitoring).
Reviewed `src/analyzer/iec104.rs` for: panic-safety, bounds, DoS resilience, OWASP Top 10,
CWE-class issues. Parser is passive/read-only — no unbounded allocation, no network writes.

---

## Findings

No CRITICAL, HIGH, MEDIUM, or LOW findings.

| Check | CWE | Result |
|-------|-----|--------|
| Out-of-bounds Read | CWE-125 | Not present — all indices guarded by `len >= 6` early return |
| Integer Overflow/Wraparound | CWE-190 | Not present — `len as usize + 2` bounded to 255 for valid LEN |
| Panic on arbitrary input | — | Not possible — length gate precedes every slice index access |
| Unbounded allocation / memory DoS | — | Not present — O(1) fixed struct return; no heap allocation in parser |
| CPU DoS | — | Not present — both functions are unconditional O(1); no loops |
| OWASP A03:2021 Injection | — | Not applicable — passive byte reader, no output channels |
| Information disclosure | — | Not present — returns None/false only; no content disclosed |
| Unsafe code | — | None — pure safe Rust; zero `unsafe` blocks |
| Thread safety | — | Not a concern — pure free functions, no global state |

---

## Detail

### Panic safety — `parse_apci_header` (lines 111–139)

All slice accesses (`data[0]`–`data[5]`) are dominated by the `data.len() < 6` early return
at line 113. No panic path exists.

### Panic safety — `is_valid_iec104_frame` (line 162)

Short-circuit evaluation: `data[0]` only evaluated after `data.len() >= 2`, `data[1]` only
after `data[0] == 0x68`. Both indices safe.

### Integer arithmetic

`h.len` is validated in `[4, 253]` before `Some` is returned. `h.len as usize + 2` ∈ [6, 255].
No overflow on any supported platform.

### Allocation

`parse_apci_header` allocates nothing. Returns `Option<ApciHeader>` (6-byte value type).
The `vec![0u8; len]` in the Kani harness is `#[cfg(kani)]`-gated — unreachable in production.

### Unsafe code audit

Zero `unsafe` blocks in `src/analyzer/iec104.rs`. All slice indexing uses Rust's built-in
bounds-checked `Index` trait.

---

## INFO Observations (no action required)

1. **Doc phrasing:** `parse_apci_header` doc comment says `start: data[0]`; code writes
   `start: 0x68`. Semantically identical (guard at line 117 ensures `data[0] == 0x68`).
   No security impact — documentation accuracy NIT only.

2. **Public fields:** `ApciHeader` fields are all `pub`, allowing trusted callers to construct
   out-of-spec instances. Cannot be exploited via untrusted bytes (parser is the only entry
   point for network data). No security impact.

---

## Summary

The IEC-104 APCI parser is a sound implementation for a passive ICS network parser. Defense-
in-depth is correctly applied: every slice access is guarded, arithmetic is bounded, allocation
is absent, and the implementation is entirely safe Rust. No action required before merge.
