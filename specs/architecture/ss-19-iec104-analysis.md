---
document_type: architecture-section
artifact: architecture-section
level: L4
section: ss-19-iec104-analysis
subsystem_id: SS-19
phase: 1c
traces_to: ARCH-INDEX.md
version: "1.7"
status: draft
producer: architect
timestamp: 2026-07-13T00:00:00Z
feature_cycle: feature-iec104
inputs: []
input-hash: "d41d8cd"
modified:
  - date: 2026-07-13
    author: architect
    note: "F2 Pass 2 fact correction — T0809→T0881 (Service Stop; T0809 is Data Destruction), control TypeID range 45–52→45–51 (52 RESERVED / no C_SE_ND_1), T1692.002 marked as SEEDED/not-emitted-this-cycle per ADR-013 Decision 10 / PRD v1.53"
  - date: 2026-07-14
    author: architect
    note: "F2 Pass-4 remediation (DF-SIBLING-SWEEP-001, F-P4-H2): Bounded-Resource Note corrected — cursor-advance semantics reconciled to BC-2.19.026 (authoritative): 'at least 2 bytes per iteration (start byte + LEN)' → 'at least 1 byte per iteration (valid frame: LEN+2; malformed-LEN: 2-byte APCI stub; bad-start-byte: 1-byte resync scan)'; advancing 2 on a bad start byte would skip a real 0x68 at the next offset. Version bump 1.1→1.2. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
  - date: 2026-07-14
    author: architect
    note: "F2 Pass-5 remediation (DF-SIBLING-SWEEP-001, F-P5-M1/F-P5-L1): (F-P5-M1) Detection table T0836 row: 'Set-point writes (TypeIDs 48–51)' → 'Set-point + bitstring writes (C_SE 48–50, C_BO 51)' — TypeID 51 = C_BO_NA_1 bitstring (not a set-point); aligns to ADR-013 MITRE table / BC-2.19.019 authoritative wording. (F-P5-L1) Detection table T1692.001 shorthand normalized to verbatim name in both rows (TypeIDs 45–51 row + N(S) desync row): 'Unauthorized Command Message' → 'Unauthorized Message: Command Message' (matches mitre.rs technique_info canonical name). Version bump 1.2→1.3. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
  - date: 2026-07-14
    author: architect
    note: "F2 Pass-7 remediation (L1): T1692.002 shorthand label normalized to full verbatim name: 'T1692.002 Reporting Message' → 'T1692.002 Unauthorized Message: Reporting Message' (matches mitre.rs technique_info canonical name; sibling T1692.001 rows were already normalized in v1.3). Version bump 1.3→1.4. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
  - date: 2026-07-14
    author: architect
    note: "F2 Pass-8 remediation (F-P8-M2/F-P8-L2): Removed vestigial window machinery copy-pasted from DNP3/ENIP siblings — inapplicable to IEC-104. (F-P8-L2) window_start_ts: u32 field removed from Iec104FlowState (no BC governs a window property; T0814 detection is per-event, not time-windowed). (F-P8-M2) Bounded-Resource Note: phantom sentence 'Window arithmetic uses saturating_sub for backwards-timestamp safety (RULING-DNP3-SIBLING-001 pattern; see VP-045 Sub-B/C proptest harnesses)' removed — VP-045 is carry-buffer direction isolation with exactly two harnesses (proptest_vp045_direction_isolation, proptest_vp045_independent_run_equivalence); no Sub-B/C and no window property exist. Version bump 1.4→1.5. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
  - date: 2026-07-14
    author: architect
    note: "Human-mandated F2 gate follow-on (D-438): first-frame N(S) baseline guard — last_ns_c2s and last_ns_s2c promoted from u16 to Option<u16>. None = no I-frame seen yet in that direction; first observed I-frame sets Some(ns) baseline without emitting a desync finding; gap check runs only when state is already Some(prev). Field count stays 5. Version bump 1.5→1.6. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
  - date: 2026-07-15
    author: architect
    note: "ADR-013 Decision 3 reconciliation (SR-172-03 / EMIT-WITH-DEDUP ratification): detection table T0814 row updated — 'Malformed APCI / non-canonical U-frame' expanded to clarify LEN-OOB only with EMIT-WITH-DEDUP (one T0814 per flow direction); bad-start-byte noted as silent resync with no finding. Version bump 1.6→1.7. NOTE: this shard is an input to all 27 BC-2.19.* files — PO must recompute input-hashes for all 27 BCs using bin/compute-input-hash --write."
---

# SS-19: IEC-104 Analysis

## [Section Content]

SS-19 provides wirerust with passive analysis of IEC 60870-5-104 (IEC-104) TCP traffic.
IEC-104 is the dominant SCADA telecontrol protocol for substation-to-control-center
communication, running on TCP port 2404, defined in IEC 60870-5-104:2006.

The analyzer uses a two-layer APCI/ASDU framing model (ADR-013): the outer APCI frame
carries a start byte `0x68`, a LEN octet (4–253), and four control octets (CF1–CF4). The
inner ASDU is extracted from I-format frames only and provides TypeID, COT, CASDU, and IOA.

**Parser origin constraint (ADR-013 Decision 7):** The `iec60870-5` crate (proprietary)
and all GPL sources (Wireshark dissector, lib60870) are BANNED. Original Rust parser only.

---

## Subsystem Purpose

SS-19 detects high-value adversarial scenarios in substation traffic:

| Detection Scenario | MITRE Technique | Confidence |
|--------------------|----------------|------------|
| Control commands (TypeIDs 45–51) | T1692.001 Unauthorized Message: Command Message | Possible |
| Set-point + bitstring writes (C_SE 48–50, C_BO 51) | T0836 Modify Parameter | Possible |
| STOPDT-act observed | T0881 Service Stop | Possible |
| STOPDT without prior STARTDT | T0881 Service Stop | Likely |
| Malformed APCI (LEN out of [4,253]; EMIT-WITH-DEDUP: one T0814 per flow direction). Bad start byte: silent resync, no finding. Non-canonical U-frame. | T0814 Denial of Service | Possible/Likely |
| N(S) desync gap > k=12 | T1692.001 Unauthorized Message: Command Message | Possible |
| Reset process (TypeID 105) | T0827 Loss of Control | Likely |
| M_* spoofed telemetry (TypeIDs 1,3,5,9,11,13) | T1692.002 Unauthorized Message: Reporting Message | SEEDED; NOT emitted this cycle (staged per ADR-013 Decision 10) |

---

## Module: C-27 — `src/analyzer/iec104.rs`

**Classification: PURE CORE (parser functions) / EFFECTFUL SHELL (analyzer entry point).**

### Pure Core (Kani/proptest amenable)

Free `fn`s in module scope — NOT `impl` methods — required for Kani amenability:

- `parse_apci_header(data: &[u8]) -> Option<ApciHeader>` — validates start byte == 0x68,
  4 ≤ LEN ≤ 253, extracts CF1–CF4. VP-044 Kani target.
- `classify_frame_format(cf1: u8) -> FrameFormat` — discriminant on CF1 low bits:
  bit 0 = 0 → IFormat; bits 1:0 = 0b01 → SFormat; bits 1:0 = 0b11 → UFormat. Total over
  all 256 u8 values. VP-046 proptest target.
- `extract_ns(cf1: u8, cf2: u8) -> u16` — 15-bit N(S) little-endian extraction.
- `extract_nr(cf3: u8, cf4: u8) -> u16` — 15-bit N(R) little-endian extraction.
- `is_valid_iec104_frame(data: &[u8]) -> bool` — post-classification validity gate.

### Effectful Shell

- `Iec104Analyzer::on_data(...)` — frame-walk entry point; prepends carry, dispatches.
  VP-047 cargo-fuzz target.
- `Iec104Analyzer::on_flow_close(...)` — removes per-flow state.
- `Iec104Analyzer::summarize()` — builds `Iec104Summary`.

---

## Data Model

```rust
pub struct ApciHeader { start: u8, len: u8, cf1: u8, cf2: u8, cf3: u8, cf4: u8 }
pub enum FrameFormat { IFormat, SFormat, UFormat }
pub enum UCommand {
    StartdtAct, StartdtCon, StopDtAct, StopDtCon,
    TestfrAct, TestfrCon,
    NonCanonical(u8),   // CVE-2026-1773 angle: reserved bits set
}
pub struct Iec104FlowState {
    carry_c2s:       Vec<u8>,     // ≤ MAX_IEC104_CARRY_BYTES = 255
    carry_s2c:       Vec<u8>,     // ≤ MAX_IEC104_CARRY_BYTES = 255
    session_started: bool,
    last_ns_c2s:     Option<u16>, // 15-bit N(S), c2s direction
    last_ns_s2c:     Option<u16>, // 15-bit N(S), s2c direction
}
// Option<u16>: None = no I-frame seen yet in this direction (fresh flow OR mid-capture start);
// first observed I-frame establishes the baseline and emits no desync finding (BC-2.19.024).
pub struct Iec104Analyzer {
    flows:              HashMap<FlowKey, Iec104FlowState>,
    findings:           Vec<Finding>,
    frames_seen:        u64,
    i_frames:           u64,
    s_frames:           u64,
    u_frames:           u64,
    stopdt_count:       u64,
    malformed_count:    u64,
    noncanonical_count: u64,
    seq_desync_count:   u64,
}
```

Bounded-resource constants: `MAX_IEC104_CARRY_BYTES = 255` (max APDU = LEN + 2 = 255);
k-window = 12 (IEC 60870-5-104 §5.3 max unacknowledged frames); `MAX_FINDINGS = 10,000` (shared).

---

## Capability Decomposition

| Capability | BC Namespace (approx.) |
|-----------|------------------------|
| APCI header parse + LEN validation | BC-2.19.001..006 |
| Frame format discrimination (I/S/U) | BC-2.19.007..009 |
| U-format session control (STARTDT/STOPDT/TESTFR) | BC-2.19.010..014 |
| ASDU extraction from I-frames (TypeID/VSQ/COT/CASDU/IOA) | BC-2.19.015..018 |
| Control command detection (TypeIDs 45–51, 100, 105) | BC-2.19.019..022 |
| Sequence counter tracking (N(S)/N(R) desync detection) | BC-2.19.023..024 |
| Carry buffer + flow lifecycle (255-byte directional cap) | BC-2.19.025..027 |

All IEC-104 BCs use `BC-2.19.NNN` namespace. Exact count finalized by product-owner (F2).

---

## Amended-BC Touchpoints for Product-Owner

**Note:** These are architectural intent records. The product-owner writes/amends the BCs.

**BC-2.18.003** — `SUPPORTED_PORTS` must add 2404; supported count 7 → 8.
Current: `&[502, 20000, 44818, 443, 8443, 80, 8080, 53]`
Required: `&[502, 20000, 44818, 2404, 443, 8443, 80, 8080, 53]`

**BC-2.18.004** — Add port 2404 to the explicit port enumeration.

VP-041 (`proptest_vp041_oracle_cross_check`) will verify the updated derivation once
2404 is in `SUPPORTED_PORTS` — no additional VP change needed.

---

## Subsystem Boundaries

SS-19 has no intra-wirerust imports. Consuming subsystems wired at F2/F3 integration:

- `dispatcher.rs` (SS-05): `DispatchTarget::Iec104`, Rule 8, `classify_oracle` extension
  (VP-004 oracle obligation per ADR-013 Decision 9)
- `mitre.rs` (SS-10): T0881 atomic obligation — 6-part (ADR-013 Decision 10)
- `protocols.rs` (SS-18): 2404 in `SUPPORTED_PORTS` (see amended-BC touchpoints)
- `cli.rs` / `main.rs` (SS-12): `--iec104` flag, analyzer construction and wiring

---

## Bounded-Resource Note

Two directional carry buffers, each ≤ 255 bytes (`MAX_IEC104_CARRY_BYTES`). Smallest
carry cap of all binary ICS analyzers (DNP3: 292 bytes, ENIP: 600 bytes). Frame-walk
loop advances at least 1 byte per iteration (valid frame: LEN+2; malformed-LEN: 2-byte APCI stub; bad-start-byte: 1-byte resync scan to next 0x68 candidate) — no infinite loop (per BC-2.19.026).
