# Phase F6 — Kani Formal Verification Results (Feature #7 — Modbus TCP analyzer)

**Feature:** Modbus TCP analyzer (issue #7, v0.4.0)
**VPs in scope:** VP-022 (Modbus MBAP parse safety + FC classification) and VP-004 (dispatcher content-first precedence, extended with port-502 Rule 5)
**develop HEAD:** `68a3306`
**Date:** 2026-06-09
**kani version:** cargo-kani 0.67.0 (installed at `~/.cargo/bin`, CBMC backend present)
**Kani actually ran:** YES — all harnesses executed to completion.

---

## Summary

| Metric | Value |
|--------|-------|
| Kani harnesses attempted | 5 |
| Kani harnesses **SUCCESSFUL** | 5 |
| Failures / counterexamples | 0 |
| Timeouts | 0 |
| Unwind bounds required for Modbus harnesses | 0 (all straight-line / no loops) |

**Verdict: all 5 Kani harnesses report `VERIFICATION:- SUCCESSFUL`.** VP-022 and the
VP-004 port-502 extension are formally discharged.

---

## Per-harness results

### VP-022 (`src/analyzer/modbus.rs`, module `kani_proofs`)

Run command:
`cargo kani --harness verify_parse_mbap_header_safety --harness verify_is_valid_modbus_adu_gate --harness verify_classify_fc_total --harness verify_classify_fc_exception_iff_high_bit`

| Harness | Sub-property | BCs | Result | Checks |
|---------|--------------|-----|--------|--------|
| `verify_parse_mbap_header_safety` | A.1–A.3 — no panic/OOB; `None` iff `len<8`; BE field decode on `Some` | BC-2.14.001/002 | **VERIFICATION:- SUCCESSFUL** | 0 of 140 failed |
| `verify_is_valid_modbus_adu_gate` | A.4 — gate true iff `proto==0 && 2<=len<=254` | BC-2.14.003/004 | **VERIFICATION:- SUCCESSFUL** | all SUCCESS |
| `verify_classify_fc_total` | B — `classify_fc` totality + Read/Write/Diagnostic/Unknown membership over all 256 FCs | BC-2.14.005/007/008 | **VERIFICATION:- SUCCESSFUL** | all SUCCESS |
| `verify_classify_fc_exception_iff_high_bit` | C — `Exception` iff `fc>=0x80`; mask invariant `(fc&0x7F)<0x80` | BC-2.14.006 | **VERIFICATION:- SUCCESSFUL** | all SUCCESS |

Aggregate: `Complete - 4 successfully verified harnesses, 0 failures, 4 total.`
Verification time: ~0.36 s (parse harness) — all under 1 s, matching the VP-022 estimate.

Note on `verify_classify_fc_total`: the implemented harness goes beyond the original
one-sided spec skeleton — it computes the FULL expected classification for every `u8` and
asserts `class == expected` (a biconditional mapping check). This catches a wrong-mapping
bug (e.g. returning `Read` for an undefined FC) that a tautological variant-exhaustion check
could not.

### VP-004 (`src/dispatcher.rs`, module `kani_proofs`)

Run command: `cargo kani --harness verify_content_first_precedence_exhaustive`

| Harness | Property | Result |
|---------|----------|--------|
| `verify_content_first_precedence_exhaustive` | `classify(data,key) == classify_oracle(...)` for symbolic ports + 8 symbolic bytes — TLS/HTTP content signatures beat ALL port rules, AND the port-502 → Modbus Rule 5 fallback mirrors the oracle exactly | **VERIFICATION:- SUCCESSFUL** |

Aggregate: `Complete - 1 successfully verified harnesses, 0 failures, 1 total.`

The VP-004 proof still holds after the Rule-5 (port-502 → `DispatchTarget::Modbus`) extension
to both `classify` and `classify_oracle`: content rules (TLS `0x16 0x03`, HTTP method tokens)
still strictly precede the port-502 fallback, so the content-first invariant is preserved.

---

## Environment confirmation

- `cargo kani --version` → `cargo-kani 0.67.0`
- `~/.kani/kani-0.67.0` present; CBMC backend resolved (proofs emitted full check tables,
  e.g. 140 SAT checks for the parse harness, all `Status: SUCCESS`).
- No harness required a `#[kani::unwind(N)]` bound for the Modbus sub-properties (no loops);
  the dispatcher harness retains its pre-existing bounds where applicable.

**No deferral.** Kani ran for real and all harnesses passed.
