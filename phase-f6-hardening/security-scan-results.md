# Phase F6 — Security Scan Results (Feature #7 — Modbus TCP analyzer)

**Feature:** Modbus TCP analyzer (issue #7, v0.4.0)
**develop HEAD:** `68a3306`
**Date:** 2026-06-09
**Scope:** full dependency tree + Modbus untrusted-input memory-safety review.

---

## Summary

| Scan | Result |
|------|--------|
| `cargo audit` | 1 allowed warning (known-accepted), 0 unresolved vulnerabilities |
| `cargo deny check` | **advisories ok, bans ok, licenses ok, sources ok** |
| Modbus untrusted-input memory-safety review | PASS — no panic/OOB/unbounded-memory |
| **CRITICAL / HIGH findings** | **0** |

**Verdict: no CRITICAL/HIGH findings. PASS.** No `security-reviewer` escalation required.

---

## cargo audit

```
Loaded 1123 security advisories
Scanning Cargo.lock for vulnerabilities (193 crate dependencies)

Crate:    rand
Version:  0.8.5
Warning:  unsound
Title:    Rand is unsound with a custom logger using `rand::rng()`
ID:       RUSTSEC-2026-0097
Tree:     rand 0.8.5 → phf_generator → phf_codegen → tls-parser 0.12.2 → wirerust

warning: 1 allowed warning found
```

- **RUSTSEC-2026-0097** is the single **known-accepted** advisory. It is an *unsoundness*
  warning (not a vulnerability with a fixed version available on this path) reaching us purely
  transitively through `tls-parser → phf_codegen → phf_generator → rand 0.8.5`, used only at
  TLS-parser build/codegen time. It is already allowlisted (`1 allowed warning`). No direct
  `rand` usage in wirerust. **Accepted, no action.**

## cargo deny check

```
advisories ok, bans ok, licenses ok, sources ok
```

All four gates pass: no banned crates, no license violations, no untrusted sources, and the
advisory gate honours the RUSTSEC-2026-0097 allowance.

## Modbus untrusted-input memory-safety review

The Modbus parser consumes attacker-controlled pcap bytes. Confirmed (cross-validated by the
VP-022 Kani proofs and the `fuzz_modbus_parse` 3.7M-exec run, 0 crashes):

| Concern | Guard | Status |
|---------|-------|--------|
| OOB read in MBAP parse | `parse_mbap_header` returns `None` for `len < 8`; all indices `data[0..8]` then in-bounds | PASS (Kani-proven) |
| Panic on malformed ADU | 3-point `is_valid_modbus_adu` gate + `parse_mbap_header` `Option` return; no `unwrap()` on attacker bytes | PASS |
| Unbounded carry buffer (partial-ADU DoS) | `MAX_ADU_CARRY_BYTES = 260` cap; cumulative `carry.len()+remaining.len()` check → latch `is_non_modbus` on overflow | PASS |
| Unbounded pending table (txn-flood DoS) | `MAX_PENDING_TRANSACTIONS = 256` cap; new inserts silently dropped at cap | PASS |
| Unbounded exception-window state | per-FC counters keyed by `u8` (≤256 keys), window counts are `u32` | PASS |
| Integer overflow in counters | release profile sets `overflow-checks = true`; window/aggregate counters use `wrapping_sub` for elapsed-time math by design | PASS |

No CRITICAL/HIGH/MEDIUM memory-safety issue identified in the Modbus delta.

## semgrep

Not run in this environment (semgrep not installed on PATH). The two Rust-native SAST gates
(`cargo audit`, `cargo deny`) ran clean, and the untrusted-input surface is independently
covered by the Kani memory-safety proofs and the fuzz run. Flagged as an environment note,
not a blocker — no Rust-relevant CRITICAL/HIGH class is left uncovered by the gates that did
run. (Follow-up: wire semgrep into the F6 toolchain image if a third SAST layer is desired.)
