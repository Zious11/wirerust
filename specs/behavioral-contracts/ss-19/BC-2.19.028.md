---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-15T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: []
input-hash: "d41d8cd"
---

# BC-2.19.028: MAX_IEC104_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth

## Description

`Iec104Analyzer` enforces a hard upper bound (`MAX_IEC104_FINDINGS`) on the total number of
`Finding` objects accumulated in `self.all_findings`. The cap is applied in `on_data` at the
point where `local_findings` are merged into `self.all_findings` (the extend step): only as
many findings from `local_findings` as fit within the remaining capacity are appended; the
remainder are counted but silently discarded.

When the cap is reached or would be exceeded: no new findings beyond the remaining slots are
appended, and `Iec104Analyzer.dropped_findings: u64` is incremented by the count of suppressed
findings. This counter is surfaced in `summarize()` output (detail key `"dropped_findings"`).
No `Finding` is emitted for the cap event itself.

Per-flow state (`Iec104FlowState`) — carry buffers, dedup flags, `ns_expected` — continues to
be updated regardless of the findings cap, so sequence-desync tracking and carry-overflow
reporting remain accurate if the cap is relaxed in a future configuration.

This mirrors the Modbus `MAX_FINDINGS` (BC-2.14.022), DNP3 `MAX_FINDINGS` (BC-2.15.022), and
EtherNet/IP `MAX_FINDINGS` (BC-2.17.022) patterns. The constant value (10,000) is shared
across all analyzers.

The doc comment on `detect_iec104_threats` MUST state that the caller (`on_data`) is
responsible for enforcing `MAX_IEC104_FINDINGS` on `self.all_findings` at the extend step;
`detect_iec104_threats` itself writes into a local buffer with no cap guard. The doc comment
on `on_data` MUST note the cap bound and cite BC-2.19.028. This is the fn-doc-comment
cardinality-bound documentation requirement from IEC104-FINDINGS-CAP-001.

## Preconditions

1. `self.all_findings.len() + local_findings.len() > MAX_IEC104_FINDINGS` at the extend step
   in `on_data` — i.e., the new batch of findings from the frame-walk loop would push the
   analyzer-level accumulator past the cap.

## Postconditions

1. Only findings that fit within the remaining capacity
   (`MAX_IEC104_FINDINGS - self.all_findings.len()` slots before the extend) are appended to
   `self.all_findings`; findings beyond that slot count are discarded.
2. `self.all_findings.len()` never exceeds `MAX_IEC104_FINDINGS` after any `on_data` call
   returns. This is the primary invariant and the anchor AC for STORY-173 must trace to this
   postcondition.
3. Per-flow state (`Iec104FlowState` fields: `carry_c2s`, `carry_s2c`, `ns_expected`,
   `session_started`, dedup flags) IS still updated — per-flow state tracks protocol activity
   regardless of the findings cap.
4. Per-direction dedup guards for carry-overflow (`carry_overflow_reported_c2s`,
   `carry_overflow_reported_s2c`) and malformed-LEN (`malformed_len_reported_c2s`,
   `malformed_len_reported_s2c`) are NOT set if the associated finding was discarded due to
   the cap (the guard prevents duplicate findings, but if the first firing was dropped, a
   future on_data call should be able to emit when capacity allows — this is an acceptable
   edge case; see EC-002 below).
5. `Iec104Analyzer.dropped_findings: u64` is incremented by exactly the count of findings
   discarded at the extend step. The counter is monotonically non-decreasing across the
   lifetime of the analyzer. It is surfaced in `summarize()` as detail key
   `"dropped_findings"`. No Finding is emitted; the silent-drop behavior is otherwise
   unchanged.

## Invariants

1. **MAX_IEC104_FINDINGS is a shared constant**: `MAX_IEC104_FINDINGS = 10_000` is the same
   value used by all analyzers (Modbus, DNP3, EtherNet/IP). It is not IEC-104-specific.
   [Mirrors BC-2.14.022, BC-2.15.022, BC-2.17.022.]
2. **Cap enforcement at the on_data extend step**: the cap is applied when `local_findings`
   is merged into `self.all_findings`, not inside `detect_iec104_threats`. The function
   `detect_iec104_threats` writes into a caller-supplied local buffer; the caller (`on_data`)
   is responsible for enforcing the cap on the analyzer-level accumulator at merge time.
3. **Per-flow state always updated**: protocol per-flow statistics and dedup state are updated
   regardless of the cap. This ensures carry-buffer management and sequence-desync tracking
   (BC-2.19.023, BC-2.19.024) remain accurate even when findings are suppressed.
4. **DoS bound**: `MAX_IEC104_FINDINGS` is the primary defense against an adversary flooding
   TCP/2404 with valid-looking IEC-104 I-frames to exhaust analyzer memory. The cap ensures
   `O(1)` memory regardless of traffic volume (CWE-400/770 defense).
5. **dropped_findings is a COUNTER ONLY — no Finding emitted**: the counter exists solely to
   make the silent cap-drop observable in `summarize()` output (detail key
   `"dropped_findings"`). No anomaly Finding, no warning, and no log entry is emitted when
   findings are dropped. The cap behavior is unchanged. This preserves Postconditions 1 and 2
   while adding observability parity with Modbus (BC-2.14.022), DNP3 (BC-2.15.022), and
   EtherNet/IP (BC-2.17.022).
6. **Fn-doc-comment cardinality-bound requirement (IEC104-FINDINGS-CAP-001)**: the doc
   comment on `detect_iec104_threats` must state the cardinality contract ("caller enforces
   `MAX_IEC104_FINDINGS` cap on the analyzer accumulator at extend time; this function writes
   into a local buffer with no cap guard"); the doc comment on `on_data` must note the cap
   and cite BC-2.19.028.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `self.all_findings.len() == MAX_IEC104_FINDINGS - 1`; frame walk produces 1 finding (e.g., T1692.001 from TypeID=45) | Finding appended (cap allows 1 more); `len == MAX_IEC104_FINDINGS`; `dropped_findings` unchanged |
| EC-002 | Cap reached on the first carry-overflow finding for a direction; `carry_overflow_reported_c2s` NOT set (finding dropped before guard was set); next on_data call for same direction | Since guard was not set, the carry-overflow finding can re-fire if carry again exceeds MAX_IEC104_CARRY_BYTES and cap allows; `dropped_findings` was incremented when the first fire was dropped |
| EC-003 | Frame walk produces 5 findings; cap has 3 remaining slots | First 3 findings appended; last 2 discarded; `dropped_findings += 2` |
| EC-004 | `self.all_findings.len() == MAX_IEC104_FINDINGS`; 10 subsequent on_data calls each producing 5 findings | No findings appended; `dropped_findings += 50` (5 per call × 10 calls); `all_findings.len()` stays at `MAX_IEC104_FINDINGS` |
| EC-005 | `MAX_IEC104_FINDINGS == 0` (hypothetical) | No findings ever appended; `dropped_findings` counts all; per-flow state still updated; theoretical edge case (not a supported config) |

## Canonical Test Vectors

| State | Event | Expected |
|-------|-------|---------|
| `all_findings.len() == MAX_IEC104_FINDINGS - 1` | Frame walk produces 1 finding (T1692.001) | Finding appended (`len → MAX_IEC104_FINDINGS`); `dropped_findings` unchanged |
| `all_findings.len() == MAX_IEC104_FINDINGS` | Frame walk produces 1 finding (T1692.001) | No append; `dropped_findings += 1`; `len` stays at `MAX_IEC104_FINDINGS` |
| `all_findings.len() == MAX_IEC104_FINDINGS - 2` | Frame walk produces 5 findings (T1692.001, T0836, T0827, T0814 × 2) | 2 appended; 3 discarded; `dropped_findings += 3` |
| `all_findings.len() == MAX_IEC104_FINDINGS` | 100 sequential on_data calls each producing 3 findings | No appends; `dropped_findings += 300` (3 per call × 100); `len` stays capped |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Cap enforcement: effectful shell; unit test | unit test |

VP coverage is advisory — deferred to STORY-174 or later. A unit test asserting
`all_findings.len() <= MAX_IEC104_FINDINGS` after high-volume frame injection is sufficient
for P0 gate.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — the MAX_IEC104_FINDINGS bound is a safety requirement of the IEC-104 analyzer; without it, an adversary could exhaust analyzer memory by sending a large number of valid-looking IEC-104 control commands on TCP/2404 (CWE-400/770) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — DoS protection applies to all flows routed to the IEC-104 analyzer) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 |
| Stories | STORY-173 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — safety/cap BC; no finding emission) |

## Related BCs

- BC-2.19.019 through BC-2.19.022, BC-2.19.024, BC-2.19.025 — all depend on (MAX_IEC104_FINDINGS cap guard is a precondition for each detection BC; if findings are near cap, some from a given on_data call may be discarded at the extend step)
- BC-2.19.025 — composes with (carry-overflow T0814 findings enter via local_findings and are subject to the same cap at extend time)
- BC-2.14.022 — precedent (Modbus MAX_FINDINGS pattern)
- BC-2.15.022 — precedent (DNP3 MAX_FINDINGS pattern; RULING-DNP3-SIBLING-001 consistency; constant value 10_000 matched)
- BC-2.17.022 — precedent (EtherNet/IP MAX_FINDINGS pattern; same dropped_findings counter design)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `const MAX_IEC104_FINDINGS: usize = 10_000;` (shared constant; mirrors `dnp3::MAX_FINDINGS` and `enip::MAX_FINDINGS`)
- `src/analyzer/iec104.rs` — cap enforcement at `on_data` extend step: `let remaining_cap = MAX_IEC104_FINDINGS.saturating_sub(self.all_findings.len()); if local_findings.len() > remaining_cap { self.dropped_findings = self.dropped_findings.saturating_add((local_findings.len() - remaining_cap) as u64); local_findings.truncate(remaining_cap); } self.all_findings.extend(local_findings);`
- `src/analyzer/iec104.rs` — `Iec104Analyzer.dropped_findings: u64` (aggregate counter; incremented on each cap-suppressed extend; surfaced as detail key `"dropped_findings"` in `summarize()`; initialized 0 in `Iec104Analyzer::new()`)
- `src/analyzer/iec104.rs` — doc comment on `detect_iec104_threats` must state: "Caller (`on_data`) is responsible for enforcing `MAX_IEC104_FINDINGS` on `Iec104Analyzer::all_findings` at the extend step. This function writes into a caller-supplied local buffer with no cap guard. (BC-2.19.028 Invariant 6 / IEC104-FINDINGS-CAP-001)"
- `src/analyzer/iec104.rs` — doc comment on `on_data` must note: "Findings from `local_findings` are appended to `self.all_findings` subject to the `MAX_IEC104_FINDINGS` cap; excess findings are discarded and counted in `self.dropped_findings` (BC-2.19.028)."
- BC-2.14.022 (Modbus precedent), BC-2.15.022 (DNP3 precedent), BC-2.17.022 (EtherNet/IP precedent — same dropped_findings pattern and 10_000 value)

## Story Anchor

STORY-173

## VP Anchors

(none — cap enforcement; no formal proof target; unit test sufficient)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | BC-2.15.022 (DNP3 precedent, MAX_FINDINGS=10_000, dropped_findings pattern); BC-2.17.022 (EtherNet/IP precedent); fidelity finding SR-173-02 (BLOCKING — IEC104-FINDINGS-CAP-001) |
| **Confidence** | high — architectural safety requirement; mirrors established Modbus/DNP3/EtherNet/IP pattern; constant value and scope match DNP3 sibling exactly |
| **Extraction Date** | 2026-07-15 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads `all_findings.len()`; conditionally appends to `all_findings`; increments `dropped_findings` |
| **Deterministic** | yes — same frame sequence produces same cap behavior |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (cap guard within `on_data` extend step) |

## DF-SIBLING-SWEEP-001 Consistency Record

Per DF-SIBLING-SWEEP-001, SS-15 and SS-19 bound values and scope are verified consistent:

| Dimension | DNP3 (BC-2.15.022) | IEC-104 (BC-2.19.028) | Consistent? |
|-----------|---------------------|------------------------|-------------|
| MAX_FINDINGS value | 10,000 | 10,000 | yes |
| Scope | per-session (Dnp3Analyzer::all_findings) | per-session (Iec104Analyzer::all_findings) | yes |
| Counter field | Dnp3Analyzer.dropped_findings: u64 | Iec104Analyzer.dropped_findings: u64 | yes |
| Finding emitted for drop | no | no | yes |
| Counter surfaced in summarize() | yes (detail key "dropped_findings") | yes (detail key "dropped_findings") | yes |
| One-shot guard behavior on drop | NOT set (guard missed; future window may re-fire) | NOT set (guard missed; future on_data may re-fire) | yes |
