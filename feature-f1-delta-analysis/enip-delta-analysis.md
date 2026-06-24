---
document_type: feature-delta-analysis
feature: EtherNet/IP + CIP (Common Industrial Protocol) Analyzer
status: draft
producer: architect
timestamp: 2026-06-24T00:00:00Z
phase: F1
traces_to: .factory/research/next-ics-protocol-prevalence.md
proposed_ss: SS-17
proposed_version: v0.11.0
---

# Feature F1 Delta Analysis: EtherNet/IP + CIP Analyzer

## Executive Summary

This delta analysis confirms EtherNet/IP + CIP is the correct next ICS/OT protocol
and scopes the MVP feature cycle to **TCP/44818 explicit messaging only**. Adding an
ENIP/CIP analyzer is a **purely additive change** with near-zero regression risk to the
existing six analyzers (Modbus, DNP3, ARP, HTTP, TLS, DNS).

The pattern established by ADR-005 (Modbus) and ADR-007 (DNP3) fully governs this
feature. A new subsystem SS-17 (CAP-17) is proposed. The BC count mirrors the
DNP3 cycle: an estimated 22–26 new BCs covering encapsulation header parsing, CPF item
iteration, CIP service-code decode, MITRE ICS detection, malformed-frame handling, and
CLI/dispatch wiring. One new Kani VP (VP-032) is proposed for pure-core parse safety,
following the VP-022/VP-023/VP-024 pattern. A new ADR-010 is warranted.

The MVP is achievable in approximately 7–9 stories (analogous to the DNP3 cycle's 9
stories). Target version bump is v0.10.0 → **v0.11.0** (new analyzer = minor).
All feature-mode phases F2–F7 are required.

---

## 1. Existing-Pattern Findings

### 1.1 Analyzer Architecture Pattern (from modbus.rs and dnp3.rs)

Both existing ICS analyzers follow an identical four-layer pattern:

**Layer 1 — Pure-core parse functions** (free functions, not `impl` methods):
- A header-parse function returning `Option<HeaderStruct>` on short data.
- A validity gate function taking `&HeaderStruct` and returning `bool`.
- A function-code classifier with a wildcard `_ => Unknown` arm (Kani totality requirement).
- Frame-length arithmetic (DNP3 only; Modbus uses the MBAP length field directly).

**Layer 2 — Per-flow state struct** (`ModbusFlowState`, `Dnp3FlowState`):
- `HashMap<FlowKey, FlowState>` held inside the analyzer struct.
- Carry buffer for partial frames (DoS-guarded by a byte cap).
- Window-based detection counters with `wrapping_sub` for timestamp arithmetic.
- A `is_non_<protocol>` latch flag to bail on desync flows.

**Layer 3 — Analyzer struct** (`ModbusAnalyzer`, `Dnp3Analyzer`):
- Aggregate counters: `total_pdu_count`, `parse_errors`, `all_findings`, `dropped_findings`.
- `MAX_FINDINGS = 10_000` poison-skip guard.
- `process_pdu()` detection engine.
- `summarize()` returning `AnalysisSummary` with a `BTreeMap<String, serde_json::Value>`.

**Layer 4 — StreamHandler impl**:
- `on_data()`: retrieves/creates per-flow state, prepends carry buffer, walks frames,
  calls `process_pdu()` per valid PDU, re-inserts flow state.
- `on_flow_close()`: removes flow from the `HashMap`.
- `StreamAnalyzer` marker trait for dispatch registration.

### 1.2 Dispatcher Wiring Pattern (dispatcher.rs)

The `classify()` function in `src/dispatcher.rs` has a strict rule ordering:
1. Content: TLS signature
2. Content: HTTP method token
3. Port: 443/8443 → TLS
4. Port: 80/8080 → HTTP
5. Port: 502 → Modbus (ADR-005, Rule 5)
6. Port: 20000 → DNP3 (ADR-007, Rule 6)
7. None

EtherNet/IP TCP explicit messaging uses IANA-registered port **44818**. The new rule
becomes **Rule 7: Port 44818 → ENIP** and the existing None becomes Rule 8. This is
a one-line addition to `classify()`.

`StreamDispatcher` holds `modbus: Option<ModbusAnalyzer>` and `dnp3: Option<Dnp3Analyzer>`.
A new `enip: Option<EnipAnalyzer>` field is required with matching `::new()`, accessor,
`take_*_analyzer()`, and `on_data` routing.

### 1.3 CLI Pattern (cli.rs + main.rs)

For each analyzer, the CLI has:
- A boolean flag (`--modbus`, `--dnp3`), included by `--all`.
- One or more threshold flags (`--modbus-write-burst-threshold`, `--dnp3-direct-operate-threshold`).
- A guard in `run_analyze()` that rejects zero/invalid thresholds.
- An `enable_<protocol>` bool threaded into `build_dispatcher()`.
- The analyzer included in `needs_reassembly`.
- A WARNING emitted when `--<protocol>` is used with `--no-reassemble`.
- `dispatcher.take_<protocol>_analyzer()` at the end of `run_analyze()` to collect
  findings and append the summary.

### 1.4 MITRE ICS Pattern (mitre.rs)

New ICS MITRE techniques are seeded in `src/mitre.rs`:
- Each new technique ID gets a `("Name", MitreTactic::IcsXxx)` tuple in `technique_info()`.
- The `all_seeded_technique_ids()` const array is extended.
- VP-007 (MITRE Technique ID Format) is an atomic obligation — any new ICS technique
  added to the seed catalog must be propagated to the VP-007 Kani harness and BC-2.10.005
  simultaneously (STORY-109 precedent).

### 1.5 SS/CAP Numbering

Current subsystem registry (from ARCH-INDEX.md v1.6):
- SS-14: Modbus/ICS Analysis (CAP-14, analyzer/modbus.rs, 25 BCs)
- SS-15: DNP3/ICS Analysis (CAP-15, analyzer/dnp3.rs, 24 BCs)
- SS-16: ARP Security Analysis (CAP-16, analyzer/arp.rs, 16 BCs)

The next available subsystem number is **SS-17** with **CAP-17**.

### 1.6 VP Numbering

VP-INDEX.md v2.10 catalogs 31 VPs (VP-001..VP-031). The next VP is **VP-032**.
Current tool counts: Kani=14, proptest=10, fuzz=2, integration/unit=5.

---

## 2. Impact Boundary

### 2.1 New Files to Create

| Path | Purpose |
|------|---------|
| `src/analyzer/enip.rs` | Pure-core parser, CIP service classifier, `EnipFlowState`, `EnipAnalyzer`, `StreamHandler` impl |
| `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001.md` through `BC-2.17.0NN.md` | 22–26 new BC files |
| `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` | Kani VP for pure-core parse functions |
| `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` | New ADR (mirrors ADR-007) |
| `.factory/phase-f2-spec-evolution/enip-architecture-delta.md` | F2 architecture delta |
| `.factory/phase-f2-spec-evolution/enip-verification-delta.md` | F2 verification delta |
| `.factory/research/enip-research.md` | Research output for CIP wire format, Wireshark dissector analysis, MITRE ICS mapping |
| `.factory/cycles/feature-enip-cip/` | Cycle manifest and convergence tracking |

### 2.2 Existing Files to Touch

| Path | Change Required | Scope |
|------|----------------|-------|
| `src/analyzer/mod.rs` | Add `pub mod enip;` declaration | 1-line insertion |
| `src/dispatcher.rs` | Add `DispatchTarget::Enip` variant; add `enip: Option<EnipAnalyzer>` field to `StreamDispatcher`; add Rule 7 (port 44818) to `classify()`; add `enip_analyzer()`, `take_enip_analyzer()` accessors; wire `on_data` routing for `DispatchTarget::Enip` | ~30–40 lines across the file; no changes to existing rules |
| `src/cli.rs` | Add `--enip` flag (bool, default-off, included by `--all`); add `--enip-xxx-threshold` flag(s) for any configurable detection thresholds | ~10–20 lines in the `Commands::Analyze` struct |
| `src/main.rs` | Add `enip`/threshold extraction from CLI args; add threshold guard(s); extend `needs_reassembly`; add ENIP-with-no-reassemble WARNING; construct `EnipAnalyzer`; pass to `build_dispatcher`; call `take_enip_analyzer()` to collect findings/summary | ~25–35 lines, additive |
| `src/mitre.rs` | Seed new EtherNet/IP + CIP MITRE ICS technique IDs in `technique_info()` and `all_seeded_technique_ids()` | ~5–10 new entries in two locations |
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | Add 22–26 new BC-2.17.NNN entries; bump version and total count | Version bump + new rows |
| `.factory/specs/architecture/ARCH-INDEX.md` | Add SS-17 row to Subsystem Registry; update Document Map component count; update ADR table with ADR-010 | 3 targeted changes |
| `.factory/specs/architecture/verification-architecture.md` | Add VP-032 to Provable Properties Catalog; update P1 enumeration list and count | Per VP-INDEX propagation obligation |
| `.factory/specs/architecture/verification-coverage-matrix.md` | Add VP-032 row; update SS-17 Kani count; update Totals row | Per VP-INDEX propagation obligation |
| `.factory/specs/verification-properties/VP-INDEX.md` | Add VP-032 row; bump total_vps 31→32, p1_count 17→18, kani_count 14→15 | Authoritative source of truth |
| `docs/adr/` | New ADR-0010 in the public `docs/adr/` ADR stream (mirrors pattern for ADR-005/0007) | New file |

No changes to: `src/decoder.rs`, `src/findings.rs`, `src/reassembly/`, `src/reporter/`,
`src/reader.rs`, `src/summary.rs`, `src/lib.rs`, or any existing analyzer (modbus.rs,
dnp3.rs, arp.rs, http.rs, tls.rs, dns.rs).

---

## 3. Proposed SS-17 and BC List

**Subsystem:** SS-17 — EtherNet/IP + CIP Analysis  
**Capability:** CAP-17  
**Module:** `src/analyzer/enip.rs`  
**ADR:** ADR-010

The BC structure mirrors SS-15 (DNP3, 24 BCs). Estimated 22–26 BCs for MVP (TCP/44818 scope).

### 3.1 Parse / Header BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.001 | `parse_enip_header(data: &[u8])` returns `None` when `data.len() < 24` (minimum ENIP encapsulation header); otherwise returns `Some(EnipHeader)` with all 10 fields decoded big-endian from fixed offsets. |
| BC-2.17.002 | `EnipHeader` field contracts: `command` at bytes 0–1; `length` at bytes 2–3 (payload byte count after header); `session_handle` at bytes 4–7; `status` at bytes 8–11; `sender_context` at bytes 12–19 (8-byte opaque context); `options` at bytes 20–23. All decoded big-endian. |
| BC-2.17.003 | `is_valid_enip_frame(h: &EnipHeader) -> bool` — 2-point validity gate: (1) `h.command` is a recognized EtherNet/IP encapsulation command value; (2) OPTIONAL `h.status == 0x0000` for requests (response frames may carry non-zero status). Returns `true` iff both conditions are satisfied. Wildcard arm guarantees totality for unrecognized commands. |
| BC-2.17.004 | `classify_enip_command(cmd: u16) -> EnipCommandClass` — total classification over all 65,536 `u16` values; wildcard arm `_ => EnipCommandClass::Unknown` mandatory; no `unreachable!`; covers: `ListServices (0x0004)`, `ListIdentity (0x0063)`, `ListInterfaces (0x0064)`, `RegisterSession (0x0065)`, `UnRegisterSession (0x0066)`, `SendRRData (0x006F)`, `SendUnitData (0x0070)`, `IndicateStatus (0x0072)`, `Cancel (0x0075)`. |
| BC-2.17.005 | Common Packet Format (CPF) item-list parse: given a valid ENIP frame with a CPF payload (`SendRRData`/`SendUnitData`), `parse_cpf_items(payload: &[u8]) -> Vec<CpfItem>` decodes `item_count` (2 bytes LE), then iterates `item_count` items each with `type_id` (2 bytes LE), `length` (2 bytes LE), and `data[0..length]`. Returns empty vec on short payload. |

### 3.2 CIP Service / Object-Model BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.006 | CIP service code extraction: when the CPF item list contains a Connected Data Item (type `0x00B1`) or Unconnected Data Item (type `0x00B2`), extract the CIP header: `service` (1 byte); `request_path_size` (1 byte, in words); `request_path` (`request_path_size * 2` bytes). |
| BC-2.17.007 | `classify_cip_service(service: u8) -> CipServiceClass` — total classification over all 256 `u8` values; wildcard `_ => Unknown`; covers: `GetAttributeAll (0x01)`, `SetAttributeAll (0x02)`, `GetAttributeList (0x03)`, `SetAttributeList (0x04)`, `GetAttributeSingle (0x0E)`, `SetAttributeSingle (0x10)`, `ForwardOpen (0x54)`, `LargeForwardOpen (0x5B)`, `ForwardClose (0x4E)`, `Reset (0x05)`, `Start (0x06)`, `Stop (0x07)`, `MultipleServicePacket (0x0A)`. Response-bit (0x80) masking: any service code with the high bit set is a `Response` classification. |
| BC-2.17.008 | CIP error response detection: when a CIP response frame carries `service & 0x80 != 0` and `general_status != 0x00`, the frame is a CIP error response. Extract `general_status` (1 byte at fixed CIP response offset) and `additional_status_size` (1 byte). |
| BC-2.17.009 | CIP request-path segment parse: iterate segments in `request_path`; each segment header byte encodes the segment type (top 3 bits) and format (bottom 5 bits). For Class segments (type `0b001`) and Instance segments (type `0b010`): extract the class/instance ID (1 or 2 bytes depending on format). Return early on short data (no panic). |

### 3.3 Detection / MITRE BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.010 | `ListIdentity (0x0063)` detection: any ENIP frame with `command == 0x0063` is classified as a device enumeration attempt (T0846 Remote System Discovery, ICS Discovery tactic). Emit one finding per frame. Confidence: Medium. |
| BC-2.17.011 | `ListServices (0x0004)` and `ListInterfaces (0x0064)` detection: same T0846 mapping as `ListIdentity`; emit one finding per frame. Together with BC-2.17.010, these three commands constitute the CIP recon surface. |
| BC-2.17.012 | `SetAttributeSingle (0x10)` / `SetAttributeAll (0x02)` / `SetAttributeList (0x04)` detection: CIP attribute-write detected → T0836 Modify Parameter (ICS Impair Process Control). Emit one finding per CIP service with write classification. Confidence: Medium. |
| BC-2.17.013 | `Reset (0x05)` detection: CIP Reset service to any target object → T0816 Device Restart (ICS Inhibit Response Function). Emit one finding per Reset service frame. Confidence: High. |
| BC-2.17.014 | `Stop (0x07)` / `ForwardClose (0x4E)` detection: CIP Stop or explicit connection teardown → T0857 System Firmware / T0814 Denial of Service (inhibit channel). Confidence: Medium. Emit one finding per frame. |
| BC-2.17.015 | Write burst detector (analogous to BC-2.14.017): count CIP write-class services per 1-second window per flow; when count exceeds `write_burst_threshold` within 1 second, emit T0806 Brute Force I/O / T1692.001 finding (Confidence: High). Configurable threshold via `--enip-write-burst-threshold` (default: 20). |
| BC-2.17.016 | CIP error-rate anomaly detector: count CIP error responses per `general_status` code per flow in a 10-second window; when count > threshold (default: 5), emit T0888 Remote System Information Discovery anomaly finding (possible CIP object scanning / error enumeration). Mirrors BC-2.14.019 exception-burst pattern. |
| BC-2.17.017 | Unknown CIP service code detection: when `classify_cip_service(service & 0x7F) == Unknown`, emit an anomaly finding with no MITRE tag (unknown-service). Confidence: Low. |

### 3.4 Malformed / Structural BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.018 | Malformed ENIP frame detection: when `is_valid_enip_frame(h)` returns `false` (unknown command or status anomaly), increment `parse_errors` and the per-flow `malformed_in_window` counter. When `malformed_in_window > MALFORMED_ANOMALY_THRESHOLD` (default: 3), emit a one-shot T0814 DoS anomaly finding. Mirrors BC-2.15.024. |
| BC-2.17.019 | CPF item-count overflow guard: when the declared `item_count` in a CPF payload would require more bytes than the payload `length` field allows, treat as parse error (increment `parse_errors`); do not panic; skip the frame. |
| BC-2.17.020 | Per-flow carry buffer: when an `on_data` chunk delivers a partial ENIP encapsulation header or partial CPF payload, stash the tail in a per-flow carry buffer bounded by `MAX_ENIP_CARRY_BYTES` (≥ max ENIP encapsulation header + CPF overhead; recommend 600 bytes). When the cumulative carry exceeds the cap, latch `is_non_enip = true` and increment `parse_errors`. |

### 3.5 Aggregate / Summarize BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.021 | `EnipAnalyzer::summarize()` returns `AnalysisSummary` with `analyzer_name: "enip"` and a `BTreeMap` with keys: `pdu_count`, `write_count`, `error_count`, `parse_errors`, `command_distribution`, `dropped_findings`. |
| BC-2.17.022 | `MAX_FINDINGS = 10_000` poison-skip guard on `EnipAnalyzer::all_findings`; any finding push past the cap increments `dropped_findings` (no panic). Mirrors BC-2.14.022. |

### 3.6 Dispatch / CLI BCs

| Proposed ID | One-Line Description |
|-------------|---------------------|
| BC-2.17.023 | `StreamDispatcher` Rule 7: a TCP flow whose lower or upper port equals 44818 and which has not matched content rules 1–2 or port rules 3–6 is classified as `DispatchTarget::Enip`. Content rules take precedence (TLS/HTTP on port 44818 route correctly). |
| BC-2.17.024 | `--enip` flag enables the ENIP analyzer; `--all` implies `--enip`. When `--enip` is set without TCP reassembly, emit a WARNING and disable ENIP (same pattern as `--modbus` and `--dnp3`). |

**Total estimated BCs: 24** (BC-2.17.001 through BC-2.17.024).
Additional BCs may arise from adversarial review in F2 (precedent: DNP3 went 22→24 in F2).

---

## 4. Proposed Verification Properties

### VP-032 — EtherNet/IP + CIP Frame Parse Safety and Command Classification

**Primary tool:** Kani (model checking)  
**Phase:** P1  
**Module:** `src/analyzer/enip.rs`  
**Verified BCs:** BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.007  
**Status:** draft (produced in F2; proof harnesses authored in F4 TDD)

**Sub-properties (4 Kani harnesses, mirrors VP-022/VP-023 pattern):**

- **Sub-A — Header parse safety:** For any `&[u8]` of bounded length, `parse_enip_header(data)` returns `None` when `data.len() < 24` and `Some(EnipHeader)` otherwise with no panic, no bounds violation, and no use of attacker-controlled bytes as slice indices beyond the fixed offsets.
- **Sub-B — Command classification totality:** `classify_enip_command(cmd: u16)` returns a valid `EnipCommandClass` variant for every possible `u16` value (65,536 symbolic inputs); the `Unknown` arm is reachable and proven non-vacuous.
- **Sub-C — Validity gate biconditional:** `is_valid_enip_frame(h)` returns `true` iff `h.command` is in the known-command set (exactly the set enumerated in BC-2.17.003). Proven for all possible `u16` command values.
- **Sub-D — CIP service classification totality:** `classify_cip_service(service: u8)` returns a valid `CipServiceClass` variant for every possible `u8` value (256 symbolic inputs); the response-bit mask logic (high bit set → `Response`) is proven; the `Unknown` arm is reachable.

**Feasibility:** Feasible. All four target functions are pure-core free functions with no I/O, no heap allocation beyond bounded slices, and fully bounded input domains. The Kani harness unwind bounds are comparable to VP-022 (MBAP parse safety, 4 harnesses all SUCCESSFUL).

**VP-INDEX update when VP-032 is produced:**
- `total_vps`: 31 → 32
- `p1_count`: 17 → 18
- `kani_count`: 14 → 15

---

## 5. ADR Recommendation

**ADR-010: Binary ICS Protocol Integration (EtherNet/IP + CIP TCP, ODVA)**

A new ADR is warranted. Precedent: ADR-005 covered Modbus TCP; ADR-007 covered DNP3.
EtherNet/IP + CIP introduces structural differences that require explicit architectural
decisions:

1. **Port fallback rule position:** Rule 7 (port 44818); existing rules 1–6 unchanged.
2. **CPF (Common Packet Format) framing layer:** ENIP has a two-level frame structure
   (ENIP encapsulation header → CPF item list → CIP payload) versus Modbus's single
   MBAP layer. The parser must iterate CPF items before reaching CIP — this is an
   architectural choice that affects the pure-core function decomposition and the
   purity boundary.
3. **CIP object-model depth vs. MVP scope:** The full CIP object model (Assembly,
   Connection Manager, Identity, etc.) is too deep for a v0.11.0 MVP. ADR-010 must
   explicitly declare the MVP subset (encapsulation header + CPF iteration + CIP service
   code extraction) and defer deeper object traversal.
4. **MITRE ICS technique set:** New techniques T0846, T0836, T0816, T0857 (if added),
   T0806, T0888 applied to CIP traffic must be registered in `mitre.rs` with VP-007
   atomic obligation.
5. **Carry buffer sizing:** ENIP frames on TCP/44818 can carry payloads up to 65,511
   bytes (the `length` field is u16 after the 24-byte encapsulation header). A carry
   buffer cap must be set — recommend 600 bytes (sufficient for a maximum-common-size
   ENIP encapsulation header + CPF overhead; deeper payloads are walked incrementally).
   This is a deliberate MVP tradeoff to avoid multi-KB per-flow carry buffers.
6. **ADR ID:** ADR-010 in `.factory/specs/architecture/decisions/` and mirrored as
   `docs/adr/0010-ethernet-ip-cip-stream-dispatch.md`.

---

## 6. Regression Risk

**Assessment: near-zero.**

The change is purely additive:
- `src/analyzer/enip.rs` is a new file; it cannot break existing analyzers.
- Changes to `src/analyzer/mod.rs` (one `pub mod enip;` line) are non-breaking.
- Changes to `src/dispatcher.rs` insert Rule 7 before the existing "No match" arm
  (Rule 7 in the current ordering). Rules 1–6 are unchanged; a port-44818 flow was
  previously unclassified (None) and now gets classified as Enip. This is the intended
  behavior and cannot affect Modbus (502), DNP3 (20000), TLS (443/8443), HTTP (80/8080),
  or content-matched flows.
- `src/cli.rs` and `src/main.rs` changes are additive flag additions; existing flags
  are untouched.
- `src/mitre.rs` additions only extend the seed catalog. VP-007 Kani proof will need
  to be re-run after the new technique IDs are seeded (this is an expected and
  well-precedented obligation from STORY-109).

**Existing test suites remain valid:** all existing unit/integration tests for Modbus,
DNP3, ARP, TLS, HTTP, DNS, reassembly, and dispatcher continue to run without modification.

**VP-007 obligation (BLOCKING per STORY-109 precedent):** seeding new MITRE ICS technique
IDs in `mitre.rs` requires propagating them to the VP-007 Kani harness and BC-2.10.005
in the same commit burst. Failing to do so will break the VP-007 proof.

---

## 7. DTU Assessment

**DTU_REQUIRED: false** for EtherNet/IP + CIP.

The project-wide DTU assessment (`.factory/specs/dtu-assessment.md`, completed 2026-05-20)
already determined `dtu_required: false` for the project. The EtherNet/IP + CIP analyzer
does not change this:
- It is a **passive network analyzer** reading pcap files — it does not call any external
  service (no REST API, no Rockwell/ODVA SDK, no CIP client library).
- The parser decodes from raw bytes on disk; the authoritative reference for the wire
  format is the Wireshark `packet-enip.c` + `packet-cip.c` dissectors (open source, file-
  readable) and the ODVA PUB00123R1 white paper.
- No Digital Twin clone is required; correctness is validated by TDD against known-good
  pcap captures.

Note: the `rust-ethernet-ip` crate and `rseip-cip` crate identified in the research phase
are CIP **client** libraries, not passive parsers. Do NOT depend on them as production
dependencies. They may be studied for encode/decode logic but the ENIP analyzer must be
a self-contained pure parser consistent with the project's zero-external-dependency
philosophy for analyzers (all existing ICS analyzers import only stdlib + crate::analyzer).

---

## 8. Test-Corpus Availability

**Primary source:** `automayt/ICS-pcap` on GitHub (https://github.com/automayt/ICS-pcap).
Per research verification (WebFetch confirmed), this repository has:
- `ETHERNET_IP/` folder — EtherNet/IP explicit-messaging captures.
- `EIP/` folder — additional EtherNet/IP captures.

**Secondary sources:**
- Wireshark SampleCaptures wiki — historically carries ENIP/CIP sample captures.
- ITI (Idaho National Laboratory) ICS Security Tools — pcap collections from ICS test beds.
- Claroty Team82 / CISA advisories — some advisories include replay pcaps.

**TDD + holdout split procedure (mirroring DNP3 cycle):**

1. Pull all pcaps from `automayt/ICS-pcap/ETHERNET_IP/` and `EIP/` into
   `.factory/holdout-scenarios/enip/dev-corpus/` before F3 story decomposition.
2. Before F4 (holdout evaluation), reserve ~20% as a holdout set in
   `.factory/holdout-scenarios/enip/holdout/` with a signed manifest.
3. The implementing agent (F4) sees only the dev corpus. The holdout evaluator (F4)
   sees only the holdout set and the public API surface.

**Concern:** The research notes that the ENIP pcap corpus in `automayt/ICS-pcap` is
curated and may not cover all CIP service codes or all error conditions. Adversarial
corpus injection (crafted malformed frames) should supplement the real captures during
F5 (adversarial review). This mirrors the DNP3 cycle's approach (Crain-Sistrunk
malformed-frame coverage required crafted frames, not captured traffic).

---

## 9. MITRE ICS Technique Set for EtherNet/IP + CIP

New techniques to add to `src/mitre.rs`. Confirmed against MITRE ATT&CK for ICS
v19.1 (the pinned version used by existing analyzers per VP-007/BC-2.10.007):

| Technique ID | Name | Tactic | ICS Tactic | Used by |
|-------------|------|--------|-----------|---------|
| T0846 | Remote System Discovery | IcsDiscovery (TA0102) | Discovery | ListIdentity, ListServices, ListInterfaces (BC-2.17.010/011) |
| T0816 | Device Restart/Shutdown | IcsInhibitResponseFunction (TA0107) | Inhibit Response Function | CIP Reset service (BC-2.17.013) |

**Already seeded** (no new registration needed — verify in mitre.rs before F3):
- T0836 (Modify Parameter) — seeded at mitre.rs:179
- T0806 (Brute Force I/O) — seeded at mitre.rs:181
- T0814 (Denial of Service) — seeded at mitre.rs:180
- T0888 (Remote System Information Discovery) — seeded at mitre.rs:184
- T1692.001 (already mapped for Modbus write-class; reuse for CIP write detection)

**Open question for F2:** Determine whether T0857 (System Firmware) is appropriate for
CIP `Stop` service or whether T0814 (DoS) alone is sufficient. This requires a research
pass against MITRE ATT&CK for ICS v19.1 sub-technique definitions. Raised as Open
Question OQ-004 below.

**VP-007 obligation:** The two new technique IDs (T0846, T0816) must be added to
`src/mitre.rs`, BC-2.10.005, and the VP-007 Kani harness in the same burst. This is a
hard invariant from STORY-109 and the VP-007 proof structure. T0846 was already seeded
in mitre.rs (line 168: `"T0846" => ("Remote System Discovery", MitreTactic::IcsDiscovery)`)
— confirm this covers ENIP usage without a new enum variant. T0816 appears absent from
the current seed catalog and must be added.

---

## 10. Scope Confirmation: TCP/44818 MVP

The research recommendation and this delta analysis confirm the MVP scope:

**IN SCOPE (v0.11.0):**
- TCP/44818 explicit messaging (EtherNet/IP encapsulation + CPF + CIP service layer).
- ENIP encapsulation header parse (all 10 fields).
- CPF item iteration (item-count bounded walk, type-ID recognition for 0x00B1/0x00B2).
- CIP service-code extraction and classification.
- Detection: recon (ListIdentity/ListServices), write (SetAttribute*), Reset, error-rate.
- MITRE ICS tactic attribution.
- Malformed-frame handling with per-flow carry buffer.
- `--enip` CLI flag + `--all` inclusion.
- Kani VP-032 for pure-core parse safety.

**DEFERRED (post-v0.11.0):**
- UDP/2222 implicit/cyclic I/O (requires assembly-object semantics and Connection Manager state).
- TCP/2221 or UDP/2221 EtherNet/IP Secure (TLS/DTLS channel — encrypted, passive parser limited to metadata).
- Full CIP object-model traversal (Assembly Object, Connection Manager full state machine).
- CIP path-segment deep parse beyond class/instance extraction (attribute IDs, network segments, etc.).
- ForwardOpen/LargeForwardOpen connection-establishment state tracking.

---

## 11. Cycle Plan

### 11.1 Proposed Epic

**Epic: EtherNet/IP + CIP ICS Analyzer (SS-17)**  
**Version:** v0.11.0 (minor bump; new analyzer)  
**GitHub Issue:** to be created (pending human gate; DF-VALIDATION-001 satisfied by `.factory/research/next-ics-protocol-prevalence.md`)

### 11.2 Estimated Stories

Based on the DNP3 cycle (9 stories for 24 BCs, STORY-106 through STORY-113 plus STORY-110
for dispatcher wiring) and the Modbus cycle (similarly structured), the ENIP cycle
estimates **7–9 stories**:

| Story (proposed) | Scope | Key BCs |
|-----------------|-------|---------|
| STORY-EIP-01 | Pure-core parser: `EnipHeader`, `parse_enip_header`, `is_valid_enip_frame`, `classify_enip_command` + Kani harness skeletons (VP-032 Sub-A/B/C) | BC-2.17.001, .002, .003, .004 |
| STORY-EIP-02 | CPF item-list parse, CIP service extraction, `classify_cip_service`, CIP error extraction + VP-032 Sub-D harness | BC-2.17.005, .006, .007, .008, .009 |
| STORY-EIP-03 | Per-flow state (`EnipFlowState`), carry-buffer frame-walk, `on_data` structure, `is_non_enip` latch | BC-2.17.020, .022 |
| STORY-EIP-04 | Detection engine: recon detections (ListIdentity, ListServices, ListInterfaces) → T0846; Reset → T0816 | BC-2.17.010, .011, .013 |
| STORY-EIP-05 | Detection engine: write detections (SetAttribute*) → T0836/T1692.001; Stop/ForwardClose → T0814; unknown service | BC-2.17.012, .014, .017 |
| STORY-EIP-06 | Burst/rate detectors: write burst (1-second window) → T0806; CIP error-rate anomaly (10-second window) → T0888; malformed-frame one-shot → T0814 | BC-2.17.015, .016, .018, .019 |
| STORY-EIP-07 | `EnipAnalyzer` struct, `summarize()`, `MAX_FINDINGS` guard, dispatcher wiring (Rule 7 port 44818), `DispatchTarget::Enip`, `StreamDispatcher` field + accessors | BC-2.17.021, .022, .023 |
| STORY-EIP-08 | CLI wiring: `--enip` flag, `--enip-write-burst-threshold`, threshold guard, `needs_reassembly`, WARNING, `take_enip_analyzer()`, findings/summary collection | BC-2.17.024 |
| STORY-EIP-09 | MITRE registration (T0846 confirmation/T0816 addition), VP-007 atomic propagation, VP-032 lock (Kani harnesses green), integration tests against dev corpus pcaps | BC-2.10.005/006 (VP-007 obligation) |

The actual story count and BC assignment will be determined by the F2 and F3 agents
after the ADR-010 and architecture delta are written. The split above is advisory.

### 11.3 BC Count Delta

- Current BC-INDEX: v1.73, 305 BCs (304 active)
- Proposed addition: +24 (BC-2.17.001..024)
- Post-ENIP BC-INDEX: v1.7x, ~329 BCs

### 11.4 Phases Required

All feature-mode phases are required:

| Phase | Purpose |
|-------|---------|
| **F2** | Spec evolution: produce ADR-010, enip-architecture-delta.md, enip-verification-delta.md; write all 24 BC-2.17.NNN files; register SS-17 in ARCH-INDEX; register VP-032 in VP-INDEX/verification-architecture/coverage-matrix |
| **F3** | Story decomposition: produce STORY-EIP-01..09 files; pull dev corpus pcaps; create holdout split manifest |
| **F4** | TDD implementation: red-green-refactor per story; CLI integration; pcap validation against dev corpus |
| **F5** | Adversarial review: adversary pass against ENIP parser (CPF malformed-item attacks, CIP path-segment fuzzing, DoS via giant `length` field, session-handle spoofing) |
| **F6** | Formal hardening: run Kani VP-032 Sub-A/B/C/D; run cargo-fuzz against `parse_enip_header`; mutation testing against classifier |
| **F7** | Convergence: holdout evaluation against reserved pcap set; confirm ENIP analyzer returns expected finding types + counts; gate pass |

---

## 12. Open Questions for Human Gate

| ID | Question | Decision Needed Before |
|----|----------|----------------------|
| OQ-001 | **UDP/2222 scope:** Confirm UDP/2222 cyclic I/O is deferred to a follow-on cycle (not part of v0.11.0). The delta analysis recommends deferral; human confirmation locks the scope. | F2 kickoff |
| OQ-002 | **Carry buffer cap:** The ENIP `length` field is a 16-bit value allowing payloads up to 65,511 bytes. The recommended cap is 600 bytes (sufficient for common explicit-messaging exchanges). A higher cap (e.g., 4,096) would handle more large CIP payloads but increases per-flow memory. Human decision needed on the tradeoff. | F2 (ADR-010 Decision 5) |
| OQ-003 | **ForwardOpen tracking:** ForwardOpen (CIP connection establishment) contains useful metadata (O→T/T→O RPI, connection serial number) that could enable connection-lifecycle detection. Include in MVP or defer? | F2 |
| OQ-004 | **T0857 System Firmware:** Is CIP `Stop` service correctly tagged T0814 (DoS) or T0857 (System Firmware)? Requires a research-agent lookup against MITRE ATT&CK for ICS v19.1 before BC-2.17.014 is finalized. | F2 spec evolution |
| OQ-005 | **`--enip-write-burst-threshold` default:** Modbus default is 20; DNP3 uses `direct_operate_threshold` default of 10. EtherNet/IP CIP write commands are common in normal manufacturing operations. A default of 20 may generate false positives. Should the default be higher (e.g., 50) or should a sustained-rate detector also be added (mirroring Modbus BC-2.14.017 Invariant 2)? | F3 story decomposition |
| OQ-006 | **GitHub issue creation:** The research is validated per DF-VALIDATION-001. Who creates the GitHub issue for this feature cycle? (Requires human action per CLAUDE.md policy.) | Before F2 kickoff |

---

## 13. Artifact References

**Inputs to this analysis:**
- Research: `.factory/research/next-ics-protocol-prevalence.md`
- Code: `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`, `src/analyzer/mod.rs`
- Code: `src/dispatcher.rs`, `src/cli.rs`, `src/main.rs`, `src/mitre.rs`
- Specs: `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.73, 305 BCs)
- Specs: `.factory/specs/architecture/ARCH-INDEX.md` (v1.6)
- Specs: `.factory/specs/verification-properties/VP-INDEX.md` (v2.10, 31 VPs)
- Specs: `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`
- State: `.factory/STATE.md` (QUIESCED, released v0.10.0 at develop=ff4b82b)

**Outputs produced by this delta analysis (F1 deliverable):**
- This file: `.factory/feature-f1-delta-analysis/enip-delta-analysis.md`

**Outputs to be produced in F2 (not yet created):**
- `.factory/phase-f2-spec-evolution/enip-architecture-delta.md`
- `.factory/phase-f2-spec-evolution/enip-verification-delta.md`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md`
- `docs/adr/0010-ethernet-ip-cip-stream-dispatch.md`
- `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001.md` through `BC-2.17.024.md`
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md`
