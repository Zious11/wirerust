---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.022: MAX_FINDINGS Cap and Poison-Skip Behavior for ModbusAnalyzer

## Description

`ModbusAnalyzer` enforces the same `MAX_FINDINGS = 10_000` cap used by `HttpAnalyzer` and
`TlsAnalyzer`. When `all_findings.len()` reaches `MAX_FINDINGS`, all subsequent finding-push
sites perform a poison-skip: the finding is discarded without error and no further findings
are added. Aggregate counters (`pdu_count`, `write_count`, `exception_count`, `fn_code_counts`,
`parse_errors`) continue to be incremented normally regardless of cap state. A `dropped_findings`
counter tracks how many findings were discarded after the cap was hit, enabling forensic
operators to understand whether the cap was a limiting factor. The cap is a bounded-resource
guarantee, not a silent data loss — operators can observe it via the summary output.

## Preconditions

1. `ModbusAnalyzer` has been processing PDUs via `on_data` calls.
2. At some point, `self.all_findings.len()` reaches `MAX_FINDINGS = 10_000`.
3. Subsequently, a PDU is processed that would normally emit one or more findings.

## Postconditions

1. **No finding is pushed** when `self.all_findings.len() >= MAX_FINDINGS`.
   The guard is checked at each individual finding push site:
   ```rust
   if self.all_findings.len() < MAX_FINDINGS {
       self.all_findings.push(finding);
   } else {
       self.dropped_findings += 1;
   }
   ```
2. `self.dropped_findings` is incremented by 1 for each finding that was discarded (not per
   PDU, but per skipped finding push attempt).
3. Aggregate counters are UNAFFECTED by the cap:
   - `total_pdu_count` incremented for every valid PDU.
   - `total_write_count` incremented for every write-class FC.
   - `total_exception_count` incremented for every exception-class FC.
   - `fn_code_counts` incremented for every valid PDU's FC.
   - `parse_errors` incremented for every invalid ADU.
   These counters are never gated by the findings cap.
4. `flow.write_count`, `flow.window_write_count`, `flow.t0831_window_write_count`,
   and related per-flow counters continue to be updated. The burst/coordination detectors
   continue running even after the cap. The `window_burst_emitted` and `t0831_burst_emitted`
   flags are still set (to prevent future failed push attempts from being tried repeatedly).
5. `summarize()` includes `dropped_findings` in the `detail` map:
   - Key: `"dropped_findings"`, Value: `Value::Number(self.dropped_findings)`.
   - This key is ALWAYS present in the summary (value 0 if no findings were dropped).
6. `all_findings.len()` never exceeds `MAX_FINDINGS = 10_000`.

## Invariants

1. **`MAX_FINDINGS = 10_000`** is the same constant used by `HttpAnalyzer`, `TlsAnalyzer`,
   and the `TcpReassembler` engine. It must be the same value across all components — a
   single `const MAX_FINDINGS: usize = 10_000` shared or re-declared with the same value.
2. **Poison-skip model** (terminology consistent with the existing codebase): once the cap
   is reached, no subsequent finding is ever added. The analyzer is "poisoned" for finding
   emission. This is a deterministic, reproducible state.
3. **Dropped findings are NOT silent**: `dropped_findings` is surfaced in `summarize()` and
   is rendered by all three reporter surfaces (terminal, JSON, CSV) so operators know the
   cap was hit.
4. **`dropped_findings` counter is also capped implicitly**: as a `u64`, it overflows at
   2^64 which is beyond any practical capture size. No explicit secondary cap is needed.
5. **Per-PDU multi-finding emission and the cap**: when a single PDU would produce N findings
   (e.g., T0855 + T0836 + T0831 for a second holding-register write within the 5s window, or
   T0855 + T0806 + burst-T0855 for a burst-threshold-crossing write), the cap is checked
   independently at each push site. If the cap hits after the 1st finding of 3 are pushed, the
   2nd and 3rd findings are dropped and `dropped_findings` is incremented by 2. Note: for a
   single non-burst holding-register write (0x06/0x10/0x16), at most 2 findings are emitted
   (T0855 + T0836); T0835 is suppressed for those FCs per the T0836 priority rule.
6. **`findings()` accessor returns `all_findings.clone()`** (consistent with `HttpAnalyzer`
   and `TlsAnalyzer` accessors). The returned slice is bounded by `MAX_FINDINGS`.
7. **The cap is NOT configurable** via CLI flags. `MAX_FINDINGS = 10_000` is a fixed
   compile-time constant. Operators who need more findings must process the capture in
   smaller time windows.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `all_findings.len() == 9_999`; a second holding-register write within the 5s window arrives (FC=0x06, emitting T0855 + T0836 + T0831) | T0855 fills slot 10_000 (len → 10_000). T0836 dropped (`dropped_findings=1`). T0831 dropped (`dropped_findings=2`). T0835 is NOT part of this scenario (suppressed for 0x06 per T0836 priority). |
| EC-002 | `all_findings.len() == 10_000`; another write FC arrives | All 3 findings dropped; `dropped_findings += 3`. Counters incremented. |
| EC-003 | `all_findings.len() == 10_000`; a T0806 burst fires | T0806 + burst T0855 both dropped; `dropped_findings += 2`. Burst state (`window_burst_emitted`) set anyway. |
| EC-004 | 0 PDUs processed | `all_findings.len() == 0`; `dropped_findings == 0`. Summary key present: `"dropped_findings": 0`. |
| EC-005 | `MAX_FINDINGS` cap hit exactly at 10_000 findings | `all_findings.len() == 10_000`; no panic; no overflow. Next push attempt → drop. |
| EC-006 | Capture produces exactly 9_999 findings | `dropped_findings == 0` in summary. No truncation message. |

## Canonical Test Vectors

| Setup | Expected Behavior | Category |
|-------|------------------|----------|
| Pre-fill `all_findings` to 9_999 entries; send FC=0x06 as the 2nd holding-register write within the 5s window (would generate T0855 + T0836 + T0831) | T0855 at index 9_999 (cap reached); T0836 dropped; T0831 dropped; `dropped_findings=2`; `write_count=1`; `fn_code_counts[0x06]=1` (T0835 NOT generated — suppressed by T0836 priority for 0x06) | edge-case (cap hit mid-PDU) |
| Pre-fill `all_findings` to 10_000; send 10 additional write PDUs (non-burst, holding-register) | No findings added; `dropped_findings += 2` per PDU (each 0x06/0x10 emits T0855 + T0836 — 2 findings per write); all counters incremented normally | edge-case (fully poisoned) |
| Empty analyzer; process 5_000 write PDUs each generating T0855 only | `all_findings.len() == 5_000`; `dropped_findings == 0`; no cap hit | happy-path (below cap) |
| summarize() called after `dropped_findings=42` | `detail["dropped_findings"] == 42` | happy-path (summary key) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | (indirect) cap constant correctness via all_findings type invariant | Integration test (not Kani — heap-bounded) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the bounded-resource guarantee for the ICS analysis capability's findings output, ensuring the analyzer cannot exhaust memory on adversarial captures |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — indirectly: bounded memory enables reliable multi-protocol analysis) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; `all_findings: Vec<Finding>`; `dropped_findings: u64`) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | N/A (resource-bounding mechanism, not a detection) |

## Related BCs

- BC-2.14.013 through BC-2.14.020 — all governed by (every finding push in these BCs is gated by the cap check in this BC)
- BC-2.14.021 — composes with (`dropped_findings` key in summarize() output)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `MAX_FINDINGS: usize = 10_000` constant
- `src/analyzer/modbus.rs` — `all_findings: Vec<Finding>` and `dropped_findings: u64` fields of `ModbusAnalyzer`
- `src/analyzer/modbus.rs` — poison-skip guard at each finding push site:
  `if self.all_findings.len() < MAX_FINDINGS { push } else { dropped_findings += 1 }`
- `src/analyzer/http.rs` and `src/analyzer/tls.rs` — reference implementations with same cap pattern

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — (indirect) cap constant consistency with existing analyzers

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.2 (MAX_FINDINGS cap: "same MAX_FINDINGS = 10_000 cap as the reassembly engine and existing analyzers"; poison-skip model per HttpAnalyzer and TlsAnalyzer); architecture-delta.md Appendix (MAX_FINDINGS const = 10_000) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell (mutates all_findings, dropped_findings) |
