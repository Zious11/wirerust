---
document_type: feature-architecture-delta
feature: EtherNet/IP + CIP Analyzer (SS-17)
feature_cycle: feature-enip-v0.11.0
issue: "#316"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-06-24T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
  - .factory/feature-f1-delta-analysis/enip-delta-analysis.md
inputs:
  - .factory/feature-f1-delta-analysis/enip-delta-analysis.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - src/dispatcher.rs
  - src/mitre.rs
  - src/analyzer/dnp3.rs
---

# EtherNet/IP + CIP Architecture Delta (F2 — Feature Mode)

## 1. Impact Boundary

This feature cycle (`feature-enip-v0.11.0`, issue #316) is a **purely additive change** to
wirerust. The existing six analyzers (Modbus, DNP3, ARP, HTTP, TLS, DNS) and all existing
infrastructure (reassembly, dispatcher, reporter, reader) are unchanged except for targeted
additive wiring in the three files listed below.

**Regression risk: near-zero** (see enip-delta-analysis.md §6 for full analysis).

## 2. New Subsystem: SS-17 (EtherNet/IP + CIP Analysis)

| Field | Value |
|-------|-------|
| SS-ID | SS-17 |
| Name | EtherNet/IP + CIP Analysis |
| Capability | CAP-17 |
| ADR | ADR-010 |
| Primary source file | `src/analyzer/enip.rs` (new) |
| Protocol | EtherNet/IP + CIP (ODVA) |
| Transport | TCP/44818 (explicit messaging); UDP/2222 deferred |
| BC namespace | BC-2.17.NNN |
| VP | VP-032 (Kani, P1, 4 sub-properties) |
| Estimated BC count | 24 (BC-2.17.001..024) |

## 3. Dispatcher Rule 7 (port 44818 TCP)

**Rule 7** is inserted as the new port fallback rule for EtherNet/IP, placed after the
existing DNP3 Rule 6 (port 20000). The existing "no match" tail becomes Rule 8.

```
Rule 7: ports contain 44818 → DispatchTarget::Enip
```

Content rules 1–2 (TLS signature, HTTP method prefix) take priority. A TLS ClientHello
on port 44818 correctly routes to TLS. Any flow on port 44818 that is neither TLS nor HTTP
is classified as ENIP and processed by the post-classification validity gate in
`EnipAnalyzer::on_data()`.

**`DispatchTarget::Enip` variant** added to the `DispatchTarget` enum after `Dnp3`.

**VP-004 oracle obligation:** The `classify_oracle` function in `dispatcher.rs`'s
`#[cfg(kani)] mod kani_proofs` gains the port-44818 → Enip arm immediately after
the port-20000 → Dnp3 arm, in lockstep with the production `classify()` function.

## 4. EnipFlowState and EnipAnalyzer Design

### 4.1 EnipFlowState

Per-flow state struct tracking the carry buffer, desync latch, detection window counters,
and lifetime error metrics. Full field list in ADR-010 Decision 4.

Key fields:
- `carry: Vec<u8>` — partial frame accumulation, bounded to `MAX_ENIP_CARRY_BYTES = 600`
- `is_non_enip: bool` — desync-safe bail latch (mirrors `is_non_dnp3`)
- `parse_errors: u64` — lifetime monotonic counter (never reset)
- `malformed_in_window: u64` + `malformed_anomaly_emitted: bool` — windowed T0814 gate
- `write_count_in_window`, `write_window_start_ts`, `write_burst_emitted` — 1s burst window
- `error_counts_in_window`, `error_window_start_ts`, `error_rate_emitted` — 10s error window
- `pdu_count: u64` — total PDU count for `summarize()`

### 4.2 EnipAnalyzer

Aggregate-level analyzer struct holding per-flow state and aggregate counters. Key
constants:
- `MAX_ENIP_CARRY_BYTES: usize = 600` (ADR-010 Decision 3)
- `MAX_FINDINGS: usize = 10_000` (mirrors Modbus/DNP3 poison-skip guard)
- `MALFORMED_ANOMALY_THRESHOLD: u64 = 3` (mirrors DNP3 pattern, BC-2.17.018)

Aggregate counters:
- `total_pdu_count: u64`
- `write_count: u64`
- `error_count: u64`
- `parse_errors: u64`
- `all_findings: Vec<Finding>`
- `dropped_findings: u64`
- `command_distribution: HashMap<u16, u64>`

`StreamHandler` impl pattern:
- `on_data()`: retrieve/create per-flow state, prepend carry, walk frames, call
  `process_pdu()` per valid PDU, re-insert flow state
- `on_flow_close()`: removes flow from `HashMap<FlowKey, EnipFlowState>`
- `StreamAnalyzer` marker trait for dispatch registration

### 4.3 Pure-core free functions (VP-032 targets)

These four functions are **free `fn`s** (not `impl EnipAnalyzer` methods):
- `parse_enip_header(data: &[u8]) -> Option<EnipHeader>` — Sub-A Kani target
- `classify_enip_command(cmd: u16) -> EnipCommandClass` — Sub-B Kani target
- `is_valid_enip_frame(h: &EnipHeader) -> bool` — Sub-C Kani target
- `classify_cip_service(service: u8) -> CipServiceClass` — Sub-D Kani target

Additional pure-core functions (not Kani-targeted in v0.11.0 scope):
- `parse_cpf_items(payload: &[u8]) -> Vec<CpfItem>`
- `parse_cip_header(item_data: &[u8]) -> Option<CipHeader>`
- `parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment>`

**F6 fuzz obligation (F-P9-002, MEDIUM):** `parse_cip_header` and `parse_cpf_items` are
attacker-facing length-driven parsers that are NOT covered by VP-032 (Kani Sub-A/B/C/D
only cover `parse_enip_header`, `classify_enip_command`, `is_valid_enip_frame`,
`classify_cip_service`). Both functions must receive cargo-fuzz no-panic / bounds-safety
fuzz harnesses in F6, analogous to VP-028 (pcapng reader fuzz). No new VP number is
required — these are lightweight fuzz harnesses, not formal Kani proofs. The F6
implementation story MUST include fuzz targets for both functions. See ADR-010 Decision 8
DEFERRED list for authoritative record.

## 5. MITRE ICS Technique Set (v0.11.0 TCP/44818 scope)

ATT&CK for ICS v19.1 (pin: `ics-attack-19.1`). Full table in ADR-010 Decision 7.

### New techniques to seed in `src/mitre.rs`

| ID | Name | Tactic | MitreTactic Variant | New to catalog? |
|----|------|--------|--------------------|----|
| T0858 | Change Operating Mode | ICS Execution (TA0104) | `IcsExecution` (NEW VARIANT) | YES |
| T0816 | Device Restart/Shutdown | ICS Inhibit Response Function (TA0107) | `IcsInhibitResponseFunction` (existing) | YES |
| T1693.001 | Modify Firmware: System Firmware | ICS Inhibit Response Function | `IcsInhibitResponseFunction` (existing) | YES (staged only in v0.11.0) |

### New MitreTactic variant required: `IcsExecution`

```rust
// In src/mitre.rs, MitreTactic enum, after IcsCommandAndControl:
/// ICS Execution tactic (TA0104) — T0858 "Change Operating Mode" and similar
/// execution-category findings (halt/stop PLC program via management protocol).
/// Distinct from Enterprise Execution (TA0002). Added atomically with T0858
/// emission (STORY-EIP-09, VP-007 obligation).
IcsExecution,
```

Display: `"Execution (ICS)"` (per D-069 pattern — ICS qualifier required).
`technique_tactic_id`: `"TA0104"`.
`all_tactics_in_report_order()`: append after `IcsCommandAndControl`.

### Already-seeded (reuse without new catalog entry)

T0846 (`IcsDiscovery`), T0888 (`IcsDiscovery`), T0836 (`IcsImpairProcessControl`),
T0814 (`IcsInhibitResponseFunction`), T1692.001 (`IcsImpairProcessControl`),
T1692.002 (`IcsImpairProcessControl`), T0806 (`IcsImpairProcessControl`).

### VP-007 atomic obligation (6-part burst, STORY-EIP-09)

When T0858 + T0816 + T1693.001 are added to `technique_info()`:
1. Add three `technique_info` match arms (T0858/T0816/T1693.001)
2. Add `"T0858"`, `"T0816"`, `"T1693.001"` to `SEEDED_TECHNIQUE_IDS`
3. Bump `SEEDED_TECHNIQUE_ID_COUNT`: 25 → 28
4. Add `"T0858"` and `"T0816"` to `EMITTED_IDS` (17 → 19); T1693.001 staged-only, do NOT add to EMITTED_IDS yet
5. Add `IcsExecution` variant to `MitreTactic` enum; update `fmt::Display`, `all_tactics_in_report_order()`, and `technique_tactic_id()`
6. Run `cargo test mitre` to confirm `vp007_catalog_drift_guard` passes

### ForwardOpen technique gap

ATT&CK for ICS v19.1 has no dedicated technique for CIP connection establishment anomaly.
Policy: emit `mitre_techniques: vec![]` (no tag) on ForwardOpen anomaly findings detected
in isolation. Emit T1692.001 only when the connection demonstrably carries an unauthorized
CIP command in the same session. Documented in ADR-010 Decision 7.

## 6. Verification Properties Delta

| VP-ID | Title | Tool | Phase | Module | Status |
|-------|-------|------|-------|--------|--------|
| VP-032 | EtherNet/IP + CIP Frame Parse Safety and Command/Service Classification | Kani | P1 | src/analyzer/enip.rs | draft (F2) |

VP-INDEX update (authoritative source → propagate to arch section files):
- `total_vps`: 31 → 32
- `p1_count`: 17 → 18
- `kani_count`: 14 → 15

## 7. File-Touch List

### New files to create

| Path | Purpose |
|------|---------|
| `src/analyzer/enip.rs` | Pure-core parser, CIP service classifier, EnipFlowState, EnipAnalyzer, StreamHandler impl, Kani harnesses |
| `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` | New ADR (WRITTEN in F2) |
| `docs/adr/0010-ethernet-ip-cip-stream-dispatch.md` | Mirror copy of ADR-010 (WRITTEN in F2) |
| `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001.md` through `BC-2.17.024.md` | 24 behavioral contracts (product-owner writes in F2) |
| `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` | Kani VP (WRITTEN in F2) |
| `.factory/phase-f2-spec-evolution/enip-architecture-delta.md` | This file |
| `.factory/phase-f2-spec-evolution/enip-verification-delta.md` | Verification delta (to be produced by spec-steward or architect in F2) |
| `.factory/cycles/feature-enip-cip/` | Cycle manifest and convergence tracking |

### Existing files to touch

| Path | Change | Scope |
|------|--------|-------|
| `src/analyzer/mod.rs` | Add `pub mod enip;` declaration | 1-line insertion |
| `src/dispatcher.rs` | Add `DispatchTarget::Enip` variant; add `enip: Option<EnipAnalyzer>` field; add Rule 7 (port 44818) to `classify()`; add `enip_analyzer()`, `take_enip_analyzer()` accessors; wire `on_data` routing for `Enip`; extend early-exit guard; extend `classify_oracle` in `#[cfg(kani)]` | ~35–45 lines across file; no changes to existing rules |
| `src/cli.rs` | Add `--enip` flag (bool, default-off, included by `--all`); add `--enip-write-burst-threshold` (u32, default 20) | ~10–20 lines in `Commands::Analyze` |
| `src/main.rs` | Add `enip`/threshold extraction; add threshold guard; extend `needs_reassembly`; add ENIP-with-no-reassemble WARNING; construct `EnipAnalyzer`; pass to `build_dispatcher`; call `take_enip_analyzer()` to collect findings/summary | ~25–35 lines, additive |
| `src/mitre.rs` | Seed T0858, T0816, T1693.001 in `technique_info()` and `all_seeded_technique_ids()`; add `IcsExecution` variant to `MitreTactic`; update `fmt::Display`, `all_tactics_in_report_order()`, `technique_tactic_id()`, `SEEDED_TECHNIQUE_ID_COUNT` (25→28), `EMITTED_IDS` (+T0858, +T0816); bump EMITTED count 17→19 | VP-007 atomic burst, ~25–30 lines |
| `.factory/specs/architecture/ARCH-INDEX.md` | Add SS-17 row to Subsystem Registry; update Document Map component count (24→25); add ADR-010 row to ADR table; add SS-17 to Bounded-Resource Design note | ~10 lines across 3 sections |
| `.factory/specs/architecture/verification-architecture.md` | Add VP-032 to Should Prove table; update P1 enumeration list; update Tooling Selection Kani row (VP list + count 14→15); version bump | Per VP-INDEX propagation obligation |
| `.factory/specs/architecture/verification-coverage-matrix.md` | Add VP-032 row to VP-to-Module table; add analyzer/enip.rs module row to Per-Module table; update Totals row (Kani 14→15, Total 31→32); version bump | Per VP-INDEX propagation obligation |
| `.factory/specs/verification-properties/VP-INDEX.md` | Add VP-032 row; bump total_vps 31→32, p1_count 17→18, kani_count 14→15; update tool summary table Kani VP-IDs list | Authoritative source of truth |
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | Add 24 new BC-2.17.NNN entries; bump version and total count (305→329); add SS-17 section | Product-owner writes |

### Files confirmed unchanged

`src/decoder.rs`, `src/findings.rs`, `src/reassembly/`, `src/reporter/`, `src/reader.rs`,
`src/summary.rs`, `src/lib.rs`, `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`,
`src/analyzer/arp.rs`, `src/analyzer/http.rs`, `src/analyzer/tls.rs`, `src/analyzer/dns.rs`.

## 8. Bounded-Resource Design (ARCH-INDEX Cross-Cutting section note)

SS-17 adds the following bounded-resource constraints (to be added to ARCH-INDEX.md
Cross-Cutting Concerns § Bounded-Resource Design):

```
- L3/SS-17: carry buffer bounded to 600 bytes per ENIP flow (MAX_ENIP_CARRY_BYTES);
  MAX_FINDINGS = 10,000 shared constant (same as SS-14/SS-15);
  MALFORMED_ANOMALY_THRESHOLD = 3 for T0814 windowed gate
```

## 9. Purity Boundary Classification

All pure-core functions in `src/analyzer/enip.rs` are free functions (`fn`, not `impl`
methods):
- **Pure Core (formally verifiable):** `parse_enip_header`, `classify_enip_command`,
  `is_valid_enip_frame`, `classify_cip_service`, `parse_cpf_items`, `parse_cip_header`,
  `parse_cip_request_path`
- **Effectful Shell (stateful, not formally verified):** `EnipFlowState` methods,
  `EnipAnalyzer::on_data()`, `EnipAnalyzer::on_flow_close()`, `EnipAnalyzer::summarize()`,
  `EnipAnalyzer::process_pdu()`

The pure/effectful boundary is enforced by the same structural rule as DNP3 and Modbus:
Kani proof harnesses call the free functions directly; the effectful shell is covered by
unit and integration tests.

## 10. Open Architecture Items for F3

| ID | Item | Action Required |
|----|------|----------------|
| OA-001 | `--enip-write-burst-threshold` default value | F3 story decomposition to decide: 20 (current proposal, matching Modbus) or higher (OQ-005, F1 analysis) |
| OA-002 | VP-007 `EMITTED_IDS` T1693.001 timing | Confirm T1693.001 staged-only in v0.11.0 (no BC emits firmware-detection findings); do NOT add to EMITTED_IDS until firmware-detection BC is implemented |
| OA-003 | `IcsExecution` MitreTactic variant | Confirmed required by ADR-010 Decision 7; implement in STORY-EIP-09 VP-007 atomic burst |
| OA-004 | UDP/2222 deferred | Confirm UDP/2222 scope exclusion in F3 story decomposition per F1 gate D-228 |
| F-P9-001 RESOLVED | 0x00B2-only CIP service detection (HIGH) | `parse_cip_header` call sites MUST be guarded with `if item.type_id == 0x00B2`; 0x00B1 Connected-item CIP request detection deferred to v0.12.0; BC-2.17.006 precondition MUST state `item.type_id == 0x00B2`. ForwardOpen/Close unaffected (0x00B2 carriers). See ADR-010 Decision 8. |
| F-P9-002 | `parse_cip_header` / `parse_cpf_items` fuzz obligation (MEDIUM) | F6 cargo-fuzz no-panic harnesses required for both functions; no new VP number. See §4.3 above. |
