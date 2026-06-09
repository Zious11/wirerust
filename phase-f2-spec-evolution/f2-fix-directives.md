---
document_type: canonical-decisions
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "F2 Canonical Decisions — Architect Directives for Product-Owner and Formal-Verifier"
status: authoritative
producer: architect
created: 2026-06-09
supersedes: null
---

# F2 Canonical Decisions — Issue #7: Modbus TCP Analyzer

This file enumerates ten settled architectural decisions resolved during the F2 adversarial
and consistency review. Product-owner and formal-verifier MUST follow these decisions verbatim.
Do not re-litigate any item. If a downstream conflict arises, consult the architect before
deviating.

---

## Decision 1 — BC Bodies Are Authoritative; §11 / Index Titles Corrected

**Rule:** The BC FILE H1 headings are the canonical source of truth for concept→BC-ID mapping.
Index titles, PRD §11 tables, and architecture-delta §11 groupings that diverged from the
authored BC body H1s have been corrected in architecture-delta.md (this burst). Product-owner
MUST update BC-INDEX lines 346–349 (BC-2.14.016 through BC-2.14.019 title columns) and
PRD §11 tables to match.

**Canonical concept → BC-ID map (Group B — FC Classification):**

| BC-ID | Canonical Concept (from H1) |
|-------|-----------------------------|
| BC-2.14.005 | `classify_fc` Is Total Over All 256 FC Values — Complete Classification Enum (covers ALL classes including Read; no separate Read-only BC) |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) |

**BC-005 note:** BC-2.14.005's H1 says "totality over all 256 FC values". The BC-INDEX
title says "Read-Class Function Codes Classified as FunctionCodeClass::Read" — that is
WRONG. Product-owner MUST update BC-INDEX line 335 title to: "classify_fc Is Total Over
All 256 FC Values (Covers Read, Write, Diagnostic, Exception, and Unknown Classes)".

**Canonical concept → BC-ID map (Groups E/F — Detection):**

| BC-ID | Canonical Concept (from H1) | Technique(s) | Window |
|-------|-----------------------------|--------------|--------|
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window | T0831 | 5s fixed |
| BC-2.14.017 | Write-Rate Burst Exceeding `--modbus-write-threshold` Emits T0806 + T0855 | T0806 + T0855 companion | 1s configurable |
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 OR 0x0001 Emits T0814 | T0814 | per-PDU |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes | Anomaly (no MITRE tag pre-decision-8 fix) | 10s per code |
| BC-2.14.020 | Unusual or Unknown Function Code Observed Emits Anomaly Finding | T0846 for recon path (see Decision 8) | per-occurrence |

**Final SS-14 BC count: 25** (BC-2.14.001 through BC-2.14.025, all written).

---

## Decision 2 — ModbusFlowState: Complete Authoritative Field List

`ModbusFlowState` (private struct in `src/analyzer/modbus.rs`) declares ALL of the following
fields. This is the single source of truth; BCs MUST cite `architecture-delta.md §2.3` and
use these exact field names.

```rust
struct ModbusFlowState {
    // --- Transaction correlation ---
    /// Bounded to MAX_PENDING_TRANSACTIONS = 256 entries.
    pending: HashMap<(u16, u8), (u8, u32)>,  // (txn_id, unit_id) -> (request_fc, ts)

    // --- Per-flow aggregate counters (all-time) ---
    write_count: u64,
    exception_count: u64,
    pdu_count: u64,
    last_ts: u32,

    // --- T0806/T0855 write-burst rate window (1-second, configurable threshold) ---
    window_write_count: u32,
    window_start_ts: u32,
    window_burst_emitted: bool,

    // --- T0831 coordinated-write window (5-second, fixed) ---
    t0831_window_start_ts: u32,
    t0831_window_write_count: u32,
    t0831_burst_emitted: bool,

    // --- BC-2.14.019 exception-burst windows (per exception code) ---
    exception_window_counts: HashMap<u8, u32>,
    exception_window_start_ts: HashMap<u8, u32>,
    exception_burst_emitted: HashMap<u8, bool>,
}
```

The seven fields added from the adversarial review are:
`window_burst_emitted`, `t0831_window_start_ts`, `t0831_window_write_count`,
`t0831_burst_emitted`, `exception_window_counts`, `exception_window_start_ts`,
`exception_burst_emitted`.

`dropped_findings` is a per-ANALYZER counter, NOT a per-flow field. See Decision 3.

---

## Decision 3 — `dropped_findings` Is the 6th Summary Key on `ModbusAnalyzer`

`ModbusAnalyzer` (not `ModbusFlowState`) carries:

```rust
pub struct ModbusAnalyzer {
    // ... existing fields ...
    /// Count of findings silently dropped because all_findings.len() >= MAX_FINDINGS.
    /// ALWAYS present in summarize() output, even when zero.
    dropped_findings: u64,

    /// Monotonic count of flows for which at least one PDU was processed.
    /// Used by summarize() — NOT derived from self.flows.len() (see Decision 4).
    total_flows_analyzed: u64,
}
```

`summarize()` returns SIX keys (not five):

| Key | Type | Semantics |
|-----|------|-----------|
| `"pdu_count"` | `Value::Number(u64)` | Valid PDUs processed (past 3-point gate), all flows |
| `"write_count"` | `Value::Number(u64)` | Write-class FC PDUs, request direction, all flows |
| `"exception_count"` | `Value::Number(u64)` | Exception-response PDUs (FC >= 0x80), all flows |
| `"parse_errors"` | `Value::Number(u64)` | ADUs that failed the 3-point gate |
| `"function_code_distribution"` | `Value::Object` | FC → count map, hex-string keys, count > 0 only |
| `"dropped_findings"` | `Value::Number(u64)` | Findings silently dropped due to MAX_FINDINGS cap; ALWAYS present (0 when cap not reached) |

**Product-owner obligation:** Update BC-2.14.021 invariant 1 from "five key names" to
"six key names" and replace "sole source of truth for five Modbus summary keys" with
"sole source of truth for six Modbus summary keys". Add `dropped_findings` row to
BC-2.14.021 postcondition table. Remove the phrase "sole source of truth" if it implies
no future extension; instead say "these six keys are the complete and authoritative set
for v1 and must not be omitted".

---

## Decision 4 — `flows_analyzed` Race Fix: Monotonic `total_flows_analyzed` Counter

`on_flow_close` removes the flow from `self.flows` (correct — bounded memory). Therefore
`summarize()` MUST NOT derive a flow count from `self.flows.len()` (which would be zero
after all flows close). Instead:

- `ModbusAnalyzer` carries `total_flows_analyzed: u64` (see Decision 3 field list).
- `total_flows_analyzed` is incremented by 1 the FIRST time `on_data` is called for a
  given `flow_key` (i.e., on `HashMap::entry(...).or_insert_with(...)` insertion, not on
  every subsequent call).
- `summarize()` reads `self.total_flows_analyzed` for any flows-related summary output.

This field is not currently a summary key in BC-2.14.021 but is available for future use
and for internal diagnostics (e.g., the `dropped_findings` denominator).

---

## Decision 5 — Write-Burst Threshold: Single Configurable 1-Second Window

**Approved scope (amended):** `--modbus-write-threshold` is a SINGLE `u32` representing
the maximum write FCs per 1-second window per flow (default 10). There is NO separate
"2-second sustained" window in v1. The dual-window concept from the research notes
("10/s sustained / 20 burst") is documented as recommended tuning values, not as
two separate implementation windows.

**Canonical constants (single window):**

```rust
const WRITE_RATE_WINDOW_SECS: u32 = 1;       // fixed 1-second window width
const DEFAULT_MODBUS_WRITE_THRESHOLD: u32 = 10; // writes/window (configurable)
```

**Approved scope line (corrected):** Replace the old scope line:
> "CLI-configurable `--modbus-write-threshold` (default 10 writes/s sustained / 20 burst)"

With:
> "CLI-configurable `--modbus-write-threshold` (default 10 writes per 1-second window per flow)"

**ADR-005 Decision 4 consequence:** Update the EMITTED_IDS note accordingly (see Decision 8).

**Product-owner obligation:** BC-2.14.017 invariant 2 ("Default threshold semantics")
already correctly says "more than 10 write FCs within any contiguous 1-second window" —
no change needed. BC-2.14.024 must confirm `--modbus-write-threshold` is a single-window
threshold. The "10/s-sustained / 20-in-1s" language from research notes is NOT encoded
as hardcoded values in the implementation.

---

## Decision 6 — Offset-Advance Safety on Validity-Gate Failure (Desync/DoS)

Specify the desync policy explicitly in `ModbusAnalyzer::on_data` for ADUs that fail
the validity gate:

**Protocol-ID failure** (`h.protocol_id != 0x0000`):
Mark the flow as non-Modbus by setting a `flow.is_non_modbus: bool = true` flag on
`ModbusFlowState`. On all subsequent `on_data` calls for that flow key, if
`flow.is_non_modbus` is true, return immediately without parsing. Do NOT advance by
`6 + h.length` (attacker-controlled). This is the bail-out policy.

**Length-range failure** (`h.length < 2 || h.length > 253`):
Advance by `max(8, 6 + (h.length as usize))` clamped to `data.len()`, OR break the
loop and wait for the next `on_data` call (the safer option). The SAFE DEFAULT is:
`break` — stop advancing and discard the rest of the current segment. Increment
`parse_errors`. This prevents a malformed Length field from producing an OOB advance.

**Field to add to `ModbusFlowState`:**

```rust
    /// Set to true when a protocol_id != 0x0000 failure is seen; flow is silently ignored
    /// from that point (not flagged, not parsed — non-Modbus binary traffic on port 502).
    is_non_modbus: bool,
```

Add `is_non_modbus` to the complete field list in Decision 2 above (field count is now
one more than listed there; the Decision 2 list is the base, `is_non_modbus` is added
by this decision).

**Product-owner obligation:** Update BC-2.14.003 edge cases to state: "on protocol_id
failure, the flow is marked non-Modbus (`is_non_modbus = true`) and no further parsing
occurs for that flow. The claim 'resync works after a malformed ADU' is WITHDRAWN for
protocol-id failures — resync is only meaningful within a valid Modbus stream."
Update BC-2.14.004 to state: "on Length-range failure, the parser breaks the loop and
discards remaining segment data (no offset advance past attacker-controlled length)."

---

## Decision 7 — Per-PDU Multi-Technique Emission Policy (Co-emission Cap)

To bound finding amplification and prevent write-flood scenarios from exhausting
MAX_FINDINGS, the following per-PDU emission policy applies:

**Write-technique priority rule (T0836 > T0835 > T0855 for the single per-write finding):**

A single write-class PDU emits AT MOST ONE finding per write-technique tier:
- Tier 1 (most specific): T0836 (holding-register writes: FC 0x06, 0x10, 0x16) — if
  applicable, emit T0836 and SKIP T0835 for that PDU.
- Tier 1 (alternative): T0835 (coil/output writes: FC 0x05, 0x0F, and 0x10 overlap) —
  if T0836 does not apply but T0835 does, emit T0835.
- Tier 2: T0855 — always emitted once per write-class PDU (regardless of T0836/T0835),
  as it represents the broadest "unauthorized command" signal.

So a single holding-register write (e.g. FC=0x06) emits: T0836 + T0855 (two findings per
PDU), NOT T0836 + T0835 + T0855 (three).

**T0855 burst companion (BC-2.14.017):** emitted AT MOST ONCE per burst event (per window
overflow), not per write PDU within the burst. The `window_burst_emitted` flag already
enforces this.

**T0831 burst finding:** emitted AT MOST ONCE per 5-second window (per flow). The
`t0831_burst_emitted` flag already enforces this.

**Maximum findings per single write PDU (worst case):**

| PDU type | Findings | Techniques |
|----------|----------|------------|
| Holding-register write, first-in-burst, second-in-T0831-window | T0836 + T0855 + T0806 (burst) + T0855 (burst companion) + T0831 | 5 findings maximum |
| Holding-register write, mid-burst (burst already fired, T0831 already fired) | T0836 + T0855 | 2 findings |
| Coil write, first-in-burst | T0835 + T0855 + T0806 (burst) + T0855 (burst companion) | 4 findings maximum |

This policy replaces the "parallel co-fire" model. Document as deliberate v1 detection
policy in architecture-delta §2.6.

**Product-owner obligation:** Update BC-2.14.013/014/015 postconditions to reflect the
priority/selection rule:
- BC-2.14.014 (T0836): add invariant "when T0836 applies, T0835 is NOT also emitted for
  the same PDU (T0836 takes priority over T0835 for FC 0x06/0x10/0x16)".
- BC-2.14.015 (T0835): add precondition "only applies when the FC is NOT in the T0836
  subset (i.e., FC is 0x05 or 0x0F — coil-only writes)".
- BC-2.14.013 (T0855): add clarification "T0855 is emitted for every write-class PDU
  regardless of which of T0836/T0835 fires".
- BC-2.14.016 (T0831) postcondition 97: clarify "T0855 and T0836 are ALSO emitted for
  this PDU per BCs 013/014 — the priority rule applies to T0836 vs T0835, not to T0855
  which always fires independently".

---

## Decision 8 — Recon FCs Map to T0846; EMITTED_IDS Updated to 13

BC-2.14.020 recon path (FC=0x11 Report Server ID, FC=0x2B/0x0E Read Device ID) MUST
emit `mitre_technique: Some("T0846".to_string())` (Remote System Discovery). The current
BC-2.14.020 body says `mitre_technique: None` — that is WRONG. Product-owner MUST update
BC-2.14.020 recon-path postconditions.

**T0846 is already SEEDED** in `SEEDED_TECHNIQUE_IDS`. No new seeded entry is needed.
**SEEDED_TECHNIQUE_ID_COUNT remains 20.**

**EMITTED_IDS corrected to 13 (was 12):**

```rust
const EMITTED_IDS: &[&str] = &[
    // Enterprise (6, unchanged):
    "T1027",      // TLS: SNI anomaly
    "T1036",      // Reassembly: conflicting overlap
    "T1046",      // HTTP: admin panel
    "T1083",      // HTTP: path traversal
    "T1499.002",  // HTTP: header flood
    "T1505.003",  // HTTP: web shell
    // ICS — Modbus analyzer (7 new):
    "T0855",      // Modbus: write-class FC / unauthorized command + burst companion
    "T0836",      // Modbus: 0x06/0x10/0x16 parameter writes
    "T0814",      // Modbus: 0x08 Force Listen Only / Restart Comms
    "T0806",      // Modbus: write burst rate exceeded
    "T0835",      // Modbus: I/O image manipulation writes (coil-only)
    "T0831",      // Modbus: coordinated write sequence (5s window)
    "T0846",      // Modbus: recon FCs (0x11 Report Server ID, 0x2B/0x0E Read Device ID)
];
// Total: 6 + 7 = 13
```

**Corrected MITRE counts:**

| Metric | Value |
|--------|-------|
| SEEDED_TECHNIQUE_ID_COUNT | 20 (11 Enterprise + 9 ICS: T0846/T0855/T0856/T0885/T0836/T0814/T0806/T0835/T0831) |
| EMITTED count (after Modbus) | 13 (6 Enterprise + 7 ICS) |
| ICS techniques emitted (new) | 7: T0855, T0836, T0814, T0806, T0835, T0831, T0846 |

**`technique_matrix` discriminator** correctly classifies T0846: `b[1] == b'0'` → ICS
(T0846 is a 5-char ID, second byte is ASCII '0'). No change to the discriminator logic.

**ADR-005 Decision 4 update:** Replace "6 Modbus-emitted ICS IDs: T0855, T0836, T0814,
T0806, T0835, T0831" with "7 Modbus-emitted ICS IDs: T0855, T0836, T0814, T0806, T0835,
T0831, T0846".

**Product-owner obligation:** Update BC-2.14.020 recon-path postcondition
`mitre_technique` from `None` to `Some("T0846".to_string())`.

---

## Decision 9 — T0831 FC Subset Is {0x06, 0x10, 0x16}

The canonical FC subset for T0831 (Manipulation of Control) detection is
**{0x06, 0x10, 0x16}** — holding-register write FCs only.

The architecture-delta §12 scoping note historically said "0x10" only — that was a
simplification. BC-2.14.016 body (authoritative) correctly states `0x06, 0x10, or 0x16`.
Architecture-delta §12 has been corrected in this burst to state the canonical subset.

**PRD-delta EC-MOD-007 update (product-owner obligation):** Replace:
> "the trigger is: two or more Write Multiple Registers (0x10) FCs"

With:
> "the trigger is: two or more write FCs targeting holding registers (FC 0x06, 0x10, or 0x16)"

Also delete: "no BC authored yet for T0831 detection trigger" — BC-2.14.016 exists and
is fully written.

**Constant name:** `T0831_WINDOW_SECS = 5` (fixed, not CLI-configurable in v1).
**Field names** on `ModbusFlowState`: `t0831_window_start_ts`, `t0831_window_write_count`,
`t0831_burst_emitted` (all u32/bool as specified in Decision 2).

---

## Decision 10 — ADR-005 Cleanup

The following cleanup items have been applied to ADR-005 in this burst:

1. **Removed template placeholder text** ("[2-5 paragraphs]…" in Rationale section).
2. **Pending-table value type:** `(FunctionCode, u32)` → `(u8, u32)` everywhere.
   `FunctionCode` was an undefined type alias; the concrete type is `u8`.
3. **Single-window threshold:** Decision 3 of ADR-005 updated to reflect single 1-second
   window (no "2s sustained" window). The EMITTED_IDS count updated from 6 to 7 ICS IDs.
4. **Status** remains `proposed` (pending F2 gate passage and human confirmation).
5. **T0846 in EMITTED_IDS:** ADR-005 Decision 4 updated to list 7 ICS emitted IDs.

---

## Summary: Files Changed in This Burst

| File | Changes |
|------|---------|
| `.factory/phase-f2-spec-evolution/architecture-delta.md` | §2.3 (complete field list), §2.7 (6 summary keys + dropped_findings field on analyzer, total_flows_analyzed), §4.3 (EMITTED_IDS 13 entries), §11 (BC titles corrected Groups B/E/F, 25-BC final count), §12 (T0831 FC subset {0x06,0x10,0x16}), Appendix (field list updated), §1 scope line, §2.4/2.5 (desync policy), §2.6 (multi-technique policy + T0846 in emission table) |
| `.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` | Decision 3 pending-table type (u8), Decision 4 EMITTED ICS count 6→7 + T0846 added, Rationale placeholder removed, single-window scope |
| `.factory/specs/architecture/ARCH-INDEX.md` | SS-14 BC Count 0→25 |
| `.factory/phase-f2-spec-evolution/f2-fix-directives.md` | This file (new) |

**Pending (product-owner, next burst):**
- BC-2.14.005 title in BC-INDEX (line 335): update to totality wording
- BC-2.14.016/017/018/019 titles in BC-INDEX (lines 346–349): already correct in BC bodies; BC-INDEX column titles need correcting
- BC-2.14.020 recon-path `mitre_technique: None` → `Some("T0846")`
- BC-2.14.021 "five keys" → "six keys", add `dropped_findings` row
- BC-2.14.003/004 desync edge cases updated
- BC-2.14.013/014/015 priority/selection rule added
- PRD-delta EC-MOD-007: FC subset and "no BC authored yet" deletion

**Pending (formal-verifier, next burst):**
- VP-022 §9 BC anchor list: add BC-2.14.016 (T0831 traces to t0831_burst_emitted field
  provable at flow state level — note: only pure-core sub-properties are Kani targets;
  T0831 timing logic is effectful shell; VP-022 scope stays at parse + classify_fc)
- VP-007 dependency note: update EMITTED count from 12 to 13
- verification-delta.md EMITTED_IDS table: update from 12 to 13 entries
