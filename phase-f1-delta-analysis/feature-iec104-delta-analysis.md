---
document_type: feature-delta-analysis
feature_id: feature-iec104
cycle: feature-iec104
title: "IEC 60870-5-104 passive protocol analyzer"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  New analyzer module (src/analyzer/iec104.rs), dispatcher enum extension (DispatchTarget::Iec104
  — VP-004 Kani zone), a new subsystem (SS-19), ~25 new BCs, ~4 new VPs, one new ADR (ADR-0013),
  and a VP-007 MITRE atomic obligation. Fails every trivial criterion.
scope_classification: standard
status: draft
producer: architect
created: 2026-07-13
base_commit: 7b11b83
branch: develop
spec_at_analysis:
  bc_index: v2.22
  vp_index: v2.40
  arch_index: v2.12
  prd: current
  story_index: current
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
---

# F1 Delta Analysis — feature-iec104

> **Post-F2-Pass-1 fact correction (2026-07-13):** T0809→T0881 throughout this document.
> T0809 is "Data Destruction"; the correct ICS ATT&CK technique for STOPDT-induced service
> inhibition is T0881 "Service Stop" (tactic: IcsInhibitResponseFunction). Additionally,
> the control TypeID range is 45–51 (TypeID 52 is RESERVED; C_SE_ND_1 does not exist);
> all "45–52" references corrected to "45–51". These corrections align the delta with the
> canonical-fact-validation results from F2 Pass 1 and the remediated SS-19 shard.
>
> **Post-F2-Pass-3 T1692.002 reconciliation (2026-07-14):** This document previously
> contained four contradictions regarding T1692.002 emission. All have been resolved to
> "SEEDED-only / NOT emitted this cycle (staged)" per ADR-013 Decision 10 and every
> SS-19 BC (none emit T1692.002): (a) §6.1 table row corrected; (b) §6.2 conditional-promotion
> paragraph reframed as a future-cycle note; (c) §9.2 STORY-C scope corrected; (d) §10
> Open-Q3 recommendation changed from "yes, promote" to "no, stage for future cycle". The
> BC-2.19.* shard is authoritative: no BC emits T1692.002 in this cycle.

## 1. Feature Summary

Add a passive IEC 60870-5-104 (IEC-104) protocol analyzer to wirerust. IEC-104 is the
dominant SCADA telecontrol protocol for substation-to-control-center communication over
TCP/IP, running exclusively on port 2404. It carries two-layer framing (APCI outer frame,
ASDU payload) and includes connection-control commands (STARTDT/STOPDT/TESTFR) that are
high-value adversarial targets in ICS/OT environments.

The analyzer is passive only (read-only pcap/pcapng). It never sends packets. Scope:

- Parse APCI framing (start byte 0x68, LEN octet, 4 control octets; max APDU = 255 bytes)
- Discriminate three frame formats from CF1 low 2 bits: I-format (data), S-format
  (supervisory), U-format (unnumbered: STARTDT/STOPDT/TESTFR)
- For I-frames: extract ASDU TypeID, COT, CASDU, IOA
- Detect control commands (TypeIDs 45–51: C_SC, C_DC, C_RC, C_SE, C_BO), reset process
  (TypeID 105), and general interrogation (TypeID 100)
- Detect STOPDT abuse, non-canonical U-frame values (CVE-2026-1773 angle), and sequence
  counter desynchronization
- Emit findings with MITRE ATT&CK for ICS technique IDs

**Parser origin constraint (see Section 6 — CRITICAL):** The parser must be original Rust.
No dependency on the `iec60870-5` crate (non-free license) and no copying from Wireshark
(GPLv2) or lib60870 (GPLv3).

---

## 2. Intent and Scope Classification

### Intent Classification

**Classified intent:** `feature`

**Rationale:** Adds a new capability (IEC-104 dissection) that does not exist in the
codebase. The feature request says "add a passive IEC-104 protocol analyzer." Nothing is
corrected or broken.

### Feature Type Classification

**Classified type:** `backend`

**Rationale:** CLI tool with no web or UI surface. All changes are in Rust source (new
`src/analyzer/iec104.rs`, modified `src/dispatcher.rs`, `src/mitre.rs`, `src/protocols.rs`,
`src/analyzer/mod.rs`, `src/cli.rs`, `src/main.rs`). No frontend, no network I/O, no
external services.

### Trivial Scope Classification

**Classified scope:** `standard`

Trivial checklist — ALL must be true for quick-dev routing; none hold here:

- [x] Single module/file? **No** — new `src/analyzer/iec104.rs` plus at minimum six
  modified files.
- [x] No new BCs? **No** — ~25 new BCs across new SS-19 and amended SS-05/SS-10/SS-12/SS-18.
- [x] No architecture change? **No** — new subsystem SS-19, new ADR-0013, new component
  in module-decomposition.md, ARCH-INDEX Subsystem Registry extension.
- [x] No new external dependencies? **True** — original parser; no new crate dependencies.
- [x] Regression risk LOW? **No** — dispatcher.rs is a VP-004 Kani-verified module; adding
  `DispatchTarget::Iec104` and Rule 8 requires `classify_oracle` extension and proof re-run.

Standard pipeline applies: full F1 → F7.

---

## 3. Impact Boundary

### 3.1 Source-Code Component Map

| File | Change Type | Subsystem | Risk | Rationale |
|------|-------------|-----------|------|-----------|
| `src/analyzer/iec104.rs` | **NEW** | SS-19 (new) | LOW | Pure-core APCI/ASDU parser; new file, zero existing-code breakage risk |
| `src/dispatcher.rs` | MODIFIED | SS-05 | MEDIUM | Add `DispatchTarget::Iec104` enum variant, Rule 8 (port 2404), `iec104: Option<Iec104Analyzer>` field, `on_data`/`on_flow_close` match arms, early-exit guard extension, and `classify_oracle` extension in VP-004 Kani section; VP-004 proofs must be re-run at F6 |
| `src/analyzer/mod.rs` | MODIFIED | SS-05 | LOW | Add `pub mod iec104;` declaration — one line |
| `src/protocols.rs` | MODIFIED | SS-18 | LOW | Add 2404 to `SUPPORTED_PORTS` constant; `supported_protocols()` count increases 7→8; `unsupported_protocols()` shrinks 23→22 automatically (pure derivation, no manual list) |
| `src/mitre.rs` | MODIFIED | SS-10 | MEDIUM | VP-007 atomic obligation: add T0881 ("Service Stop") to `SEEDED_TECHNIQUE_IDS` (28→29), bump `SEEDED_TECHNIQUE_ID_COUNT` to 29, add `technique_info("T0881")` arm, add T0881 to `EMITTED_IDS`; T1692.002 remains catalog-only (SEEDED; not emitted this cycle — staged) |
| `src/cli.rs` | MODIFIED | SS-12 | LOW | Add `--iec104` boolean flag to `Commands::Analyze`; additive clap change; existing flags unaffected |
| `src/main.rs` | MODIFIED | SS-12 | LOW | Wire `--iec104` flag, construct `Iec104Analyzer`, call `take_iec104_analyzer()` in post-loop summary assembly, extend `needs_reassembly` condition |

### 3.2 Architecture Components

| Component | Status |
|-----------|--------|
| C-27: `src/analyzer/iec104.rs` — IEC-104 Analysis | **NEW** (next component ID after C-26) |
| C-4: `src/dispatcher.rs` (StreamDispatcher) | MODIFIED — new variant + rule + analyzer field |
| C-5: `src/analyzer/mod.rs` | MODIFIED — module declaration |
| C-26: `src/protocols.rs` (Protocol Coverage Catalog) | MODIFIED — SUPPORTED_PORTS |
| C-21: `src/mitre.rs` (MITRE Mapping) | MODIFIED — new technique T0881 |
| C-22: `src/cli.rs` | MODIFIED — new flag |
| C-23: `src/main.rs` | MODIFIED — new wiring |

### 3.3 Subsystem Impact

| Subsystem | Impact | Scope |
|-----------|--------|-------|
| SS-19 (IEC-104 Analysis) | **NEW** — must be added to ARCH-INDEX Subsystem Registry | `src/analyzer/iec104.rs` |
| SS-05 (Protocol Dispatch) | MODIFIED — new enum variant, new classify rule, new analyzer field | `dispatcher.rs`, `analyzer/mod.rs` |
| SS-10 (MITRE Mapping) | MODIFIED — new T0881 catalog entry; VP-007 atomic obligation | `mitre.rs` |
| SS-12 (CLI / Entry) | MODIFIED — new `--iec104` flag and wiring | `cli.rs`, `main.rs` |
| SS-18 (Protocol Coverage Catalog) | MODIFIED — SUPPORTED_PORTS gains 2404 | `protocols.rs` |
| SS-11 (Reporting) | DEPENDENT — may need `Iec104Summary` section rendering | `reporter/terminal.rs`, `reporter/json.rs` |
| All other subsystems (SS-01/02/04/06/07/08/09/13/14/15/16/17) | NOT CHANGED | Regression baseline |

---

## 4. Dispatch Strategy

### 4.1 Dispatch Rule Assignment

IEC-104 is dispatched as **Rule 8: port-2404 primary** — a new arm inserted after the
existing ENIP Rule 7 (port 44818) and before the `None` fallback.

Updated classify() rule order:
```
Rule 1: TLS content signature (0x16 0x03) → DispatchTarget::Tls
Rule 2: HTTP token match → DispatchTarget::Http
Rule 3: TLS port fallback (443, 8443) → DispatchTarget::Tls
Rule 4: HTTP port fallback (80, 8080) → DispatchTarget::Http
Rule 5: Modbus port (502) → DispatchTarget::Modbus
Rule 6: DNP3 port (20000) → DispatchTarget::Dnp3
Rule 7: ENIP port (44818) → DispatchTarget::Enip
Rule 8: IEC-104 port (2404) → DispatchTarget::Iec104   ← NEW
Rule 9: None
```

### 4.2 Content Signature Decision

The IEC-104 start byte `0x68` is NOT added as a primary content-signature rule. Rationale:

- `0x68` is the ASCII code for `h`, which appears frequently in HTTP responses, binary
  formats, and other protocols unrelated to IEC-104. Unlike TLS (`0x16 0x03`) or ENIP
  (`0x65 0x00` or `0x63 0x00` command words), a single byte is not a reliable discriminator.
- Port-only dispatch is the established pattern for DNP3 (port 20000) and ENIP (port 44818).
  IEC-104 follows the same pattern.
- A post-classification validity gate (`is_valid_iec104_frame` checking start byte == 0x68
  and 4 ≤ LEN ≤ 253) compensates without polluting the classify() rule table.

This matches ADR-007 Decision 1 (DNP3 port-only) and ADR-010 Decision 1 (ENIP port-only).

### 4.3 Reassembly and Carry Buffer

IEC-104 reuses the existing `TcpReassembler` infrastructure.

- **Carry buffer size:** 255 bytes per direction (max APDU = LEN + 2 = 253 + 2 = 255).
  Smaller than DNP3 (292 bytes) and ENIP (600 bytes).
- **Directional split:** `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`, matching the
  DNP3 and ENIP pattern (ADR-007 Decision 2, ADR-010 Decision 2).
- **Carry cap enforcement:** each directional carry buffer is a length-bounded `Vec<u8>` capped at
  255 bytes (`MAX_IEC104_CARRY_BYTES`). IEC-104 has no windowed detection — no time-window /
  `saturating_sub` machinery (ADR-013 Decision 2). [F2 Pass-9 correction: prior text was
  copy-pasted from DNP3/ENIP sibling docs and does not apply to IEC-104.]
- **Entry point:** `on_data(flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)`
  — same signature as DNP3/ENIP (direction-threaded).

### 4.4 VP-004 Oracle Obligation

Adding `DispatchTarget::Iec104` and Rule 8 requires, in the **same commit**:

1. Add `DispatchTarget::Iec104` variant to the `DispatchTarget` enum.
2. Add the port-2404 arm to `classify()`.
3. Add the corresponding arm to `classify_oracle` in the `#[cfg(kani)] mod kani_proofs`
   block so the oracle remains a syntactic mirror of production.
4. Extend the early-exit guard (`self.iec104.is_none()`) to include the new analyzer.
5. Add `Iec104` arms to `on_data` and `on_flow_close` match blocks.

This is the same obligation recorded in ADR-010 Decision 1 for ENIP. Failure to update
the oracle invalidates the VP-004 `verify_content_first_precedence_exhaustive` proof.

---

## 5. New Spec Artifacts

### 5.1 New Behavioral Contracts (proposed IDs, subject to F2 finalization)

**SS-19 additions (new subsystem — ~22 new BCs, BC-2.19.001..022 approximate range):**

The exact count will be finalized in F2. The following areas require BCs:

| Area | Approx BC Count | Representative Topics |
|------|----------------|-----------------------|
| APCI header parsing | 4 | Start byte = 0x68, LEN bounds (4–253), control-octet extraction, APDU total-length calculation |
| Frame format discrimination | 3 | I-format (CF1 bit 0 = 0), S-format (CF1 bits 1:0 = 0b01), U-format (CF1 bits 1:0 = 0b11) |
| U-format session control | 4 | STARTDT-ACT/CON, STOPDT-ACT/CON, TESTFR-ACT/CON detection, non-canonical CF1 anomaly |
| ASDU parsing (I-frames) | 4 | TypeID extraction, VSQ, COT, CASDU, IOA; ASDU minimum-length guard |
| Control command detection | 4 | C_SC (45), C_DC (46), C_RC (47), C_SE (48–50), C_BO (51), C_RP (105), general interrogation (100) → T1692.001 findings |
| Sequence counter tracking | 3 | N(S)/N(R) 15-bit extraction, desync detection (gap > k-window), findings emission |
| Carry buffer + flow lifecycle | 4 | 255-byte directional cap, carry_c2s/carry_s2c isolation, frame-walk loop, on_flow_close cleanup |

**Estimated total for SS-19: ~22 new BCs**

**SS-05 additions (1 new BC, next ID = BC-2.05.012):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.05.012 | `classify()` Rule 8 — TCP port 2404 maps to `DispatchTarget::Iec104` | P0 |

**SS-10 additions (1 new BC, next ID = BC-2.10.010):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.10.010 | T0881 ("Service Stop") is cataloged in `SEEDED_TECHNIQUE_IDS`, resolves in `technique_info()`, and appears in `EMITTED_IDS` | P1 |

**SS-12 additions (1 new BC, next ID = BC-2.12.025):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.12.025 | `--iec104` flag enables IEC-104 analysis; included in `--all`; default false | P1 |

**Total new BCs: ~25**

### 5.2 Amended Behavioral Contracts

| BC | Amendment Scope |
|----|----------------|
| BC-2.18.003 (`supported_protocols()` derivation) | SUPPORTED_PORTS must include 2404; supported count increases 7→8 |
| BC-2.18.004 (SUPPORTED_PORTS membership) | Add 2404 to the port enumeration |

**Total amended BCs: 2**

### 5.3 New Verification Properties (proposed IDs, next available = VP-044)

| Proposed ID | Tool | Phase | Description |
|-------------|------|-------|-------------|
| VP-044 | Kani | P0 | `parse_apci_header` arithmetic safety: for any input slice, no integer overflow in LEN arithmetic; slice index always within bounds; frame-walk loop terminates (LEN ≥ 4 ensures forward progress); LEN bounds rejection (LEN < 4 or LEN > 253) is total. Pure-core free `fn` — Kani-amenable. |
| VP-045 | proptest | P1 | Directional carry buffer isolation: arbitrary byte sequences fed independently to carry_c2s and carry_s2c never cause cross-contamination; each independently caps at 255 bytes; carry contents from direction A are never visible in direction B. |
| VP-046 | proptest | P1 | U/S/I frame format discrimination is total and deterministic: for any byte value in CF1, `classify_frame_format(cf1)` returns exactly one variant; applying it twice to the same input returns the same result; the three partitions (I: bit0=0, S: bits1:0=0b01, U: bits1:0=0b11) are exhaustive and non-overlapping. |
| VP-047 | fuzz | P1 | APCI parser fuzz harness: `cargo-fuzz` over arbitrary byte sequences passed to the IEC-104 `on_data` entry point never triggers a panic, unwrap, or OOB access. |

**Total new VPs: 4 (VP-044 through VP-047)**

### 5.4 New Architecture Decisions

| ADR | Title | Scope |
|-----|-------|-------|
| ADR-0013 | IEC-104 Stream Dispatch and Parser Design | New subsystem SS-19; two-layer APCI/ASDU framing; port-2404 Rule 8; 255-byte directional carry cap; U/S/I discrimination from CF1 low 2 bits; T0881 new MITRE entry; post-classification validity gate; licensing constraint (original Rust parser only); pure-core free fn design for Kani VP-044 |

### 5.5 New Subsystem

| ID | Name | Capability | Primary Source File | BC Namespace |
|----|------|-----------|---------------------|--------------|
| SS-19 | IEC-104 Analysis | CAP-19 (new) | `src/analyzer/iec104.rs` | BC-2.19.NNN |

---

## 6. Security Findings — MITRE ATT&CK for ICS Mapping

### 6.1 Finding-to-Technique Map

| Detection Scenario | Technique ID | Technique Name | Status in Catalog | Finding Category | Verdict |
|--------------------|-------------|----------------|-------------------|-----------------|---------|
| Control commands (C_SC/C_DC/C_RC/C_SE/C_BO TypeIDs 45–51) observed | **T1692.001** | Unauthorized Message: Command Message (v19 remap of T0855) | ALREADY SEEDED + EMITTED | Impact | Possible |
| Set-point + bitstring writes (C_SE 48–50, C_BO 51) observed | **T0836** | Modify Parameter | ALREADY SEEDED + EMITTED | Impact | Possible |
| C_SE + C_SC actuation on same flow | **T0831** | Manipulation of Control | NOT emitted by IEC-104 this cycle (pre-existing EMITTED via Modbus analyzer; correlated C_SE+C_SC detection deferred to a future cycle) | Impact | Possible |
| STOPDT observed (data acquisition halted) | **T0881** | Service Stop | **NEW — must be added** | Impact | Possible |
| STOPDT without prior STARTDT (anomalous session) | **T0881** | Service Stop | **NEW — must be added** | Anomaly | Likely |
| Malformed APCI (LEN out of bounds, start byte ≠ 0x68) | **T0814** | Denial of Service | ALREADY SEEDED + EMITTED | Anomaly | Possible |
| Non-canonical U-format CF1 value (CVE-2026-1773 angle) | **T0814** | Denial of Service | ALREADY SEEDED + EMITTED | Anomaly | Likely |
| Spoofed / replayed M_* telemetry (TypeIDs 1, 3, 5, 9, 11, 13) | **T1692.002** | Unauthorized Message: Reporting Message | SEEDED; catalog-only (NOT emitted this cycle — M_* spoofed telemetry detection staged, out of feature-iec104 scope) | Impact | Possible |
| Reset process command (C_RP_NA_1, TypeID 105) | **T0827** | Loss of Control | ALREADY SEEDED + EMITTED | Impact | Likely |
| Sequence counter desynchronization (N(S) gap > k-window) | **T1692.001** | Unauthorized Message: Command Message | ALREADY SEEDED + EMITTED | Anomaly | Possible |

### 6.2 VP-007 Atomic Obligation

Adding T0881 triggers the VP-007 six-part atomic obligation (per ADR-010 §VP-007 decision):

1. Add `"T0881"` to `SEEDED_TECHNIQUE_IDS` array (28 → 29 entries)
2. Bump `SEEDED_TECHNIQUE_ID_COUNT` constant to 29
3. Add `technique_info("T0881")` arm returning `("Service Stop", MitreTactic::IcsInhibitResponseFunction)`
4. Add `"T0881"` to `EMITTED_IDS` (IEC-104 STOPDT findings emit this)
5. Verify `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT` (the VP-007 drift guard)
6. Verify `technique_info` resolves all SEEDED IDs (the VP-007 catalog completeness harness)

T1692.002 is NOT promoted to EMITTED in this cycle — M_* spoofed telemetry detection is
staged and out of feature-iec104 scope (per ADR-013 Decision 10 and every BC-2.19.* contract).
If T1692.002 is promoted in a future cycle, the `EMITTED_IDS` list gains an entry but
`SEEDED_TECHNIQUE_IDS` and `SEEDED_TECHNIQUE_ID_COUNT` do NOT change (T1692.002 is already
seeded). That promotion is a future-cycle decision requiring its own ADR amendment.

### 6.3 CWE Set

| CWE | Name | Relevance |
|-----|------|-----------|
| CWE-306 | Missing Authentication for Critical Function | IEC-104 has no authentication layer in the base protocol; all control commands are accepted from any TCP source |
| CWE-319 | Cleartext Transmission of Sensitive Information | All APCI/ASDU frames are transmitted in plaintext (no TLS) |
| CWE-294 | Authentication Bypass by Capture-Replay | Sequence numbers (N(S)/N(R)) can be replayed from passive capture |
| CWE-184 | Incomplete List of Disallowed Inputs | Non-canonical U-format CF1 values (e.g., 0x23, 0x83 with reserved bits set) not rejected by some implementations (CVE-2026-1773) |
| CWE-311 | Missing Encryption of Sensitive Data | SCADA commands (C_SC, C_SE) transmitted without confidentiality protection |

### 6.4 Technique ID Verification Note

ATT&CK for ICS technique IDs should be verified against the live ICS matrix at
implementation time (F2/F3 phase). The following IDs are confirmed stable in v19.1:
T1692.001, T1692.002, T0836, T0831, T0814, T0827, T0881. T0831 and T0814 are stable
and pre-v19 (no remap). T0881 is v19 stable (not remapped). T0852 ("Spoof Reporting
Message") from older ATT&CK ICS is cited in some CVEs but in v19.1 it refers to "Rogue
Master Device" — use T1692.002 for spoofed telemetry, not T0852. **Do not seed T0852 without
live matrix verification at F3 entry.**

---

## 7. Licensing Constraint (CRITICAL)

**Status: BLOCKER on any code copy or external crate dependency.**

### 7.1 Prohibited Sources

| Source | License | Prohibition |
|--------|---------|-------------|
| `iec60870-5` crate (crates.io) | "NOT FREE for commercial or production use" (proprietary) | **BANNED** — must not be listed in Cargo.toml or Cargo.lock |
| Wireshark IEC-104 dissector | GPLv2 | **BANNED** — cannot copy code or logic |
| lib60870 (MZ Automation) | GPLv3 | **BANNED** — cannot copy code or logic |
| Erlang SCADA lib | GPLv2 | **BANNED** |

### 7.2 Permitted References (design study only)

| Source | License | Permitted Use |
|--------|---------|--------------|
| xgbt/go-iec104 (Go) | MIT | Design reference for frame parsing logic; no code copy |
| wendy512/iec104 (Go) | Apache-2.0 | Design reference; no code copy |
| IEC 60870-5-104 standard (IEC) | Proprietary | Public tables and framing diagrams; no verbatim copy |

### 7.3 Enforcement

The ADR-0013 MUST include an explicit licensing decision ("Decision N: Original Rust
parser only — `iec60870-5` crate and all GPL sources are prohibited") so the obligation
is machine-checkable during code review and the PR checklist covers it explicitly.

---

## 8. Regression Risk

### 8.1 VP-004 Kani Zone (dispatcher.rs)

**Risk level: MEDIUM**

`src/dispatcher.rs` contains three active VP-004 Kani harnesses:
- `verify_tls_signature_beats_port`
- `verify_content_first_precedence_exhaustive`
- `verify_none_two_phase_caching`

And two VP-043 Kani harnesses for UDP gap counting:
- `vp043_udp_gap_counter_increments_for_none_target`
- `vp043_udp_gap_counter_no_increment_for_known_target`

**Impact of IEC-104 change:**

- `verify_content_first_precedence_exhaustive` asserts `got == want` where `want` comes
  from `classify_oracle`. Adding Rule 8 to both `classify()` and `classify_oracle` in
  lockstep preserves the invariant. Risk of regression is LOW if the oracle is updated
  atomically.
- `verify_none_two_phase_caching` models the None-caching state machine. This harness does
  not depend on specific port assignments and is unaffected.
- `verify_tls_signature_beats_port` asserts content beats port for TLS. Port 2404 is not
  a TLS port, so this harness is unaffected.
- VP-043 harnesses are about UDP gap counting and are unaffected by a new TCP dispatch rule.

**Mitigation:** Implement the `classify_oracle` extension in the same commit as the
`classify()` Rule 8 arm. VP-004 proofs must be re-run at F6 (formal hardening story).

### 8.2 VP-007 Kani Zone (mitre.rs)

**Risk level: MEDIUM**

Adding T0881 to `SEEDED_TECHNIQUE_IDS` requires atomic update of all six components
(see Section 6.2). The VP-007 drift guard test `SEEDED_TECHNIQUE_IDS.len() ==
SEEDED_TECHNIQUE_ID_COUNT` will fail if the constant is not bumped from 28 to 29. The
`verify_all_seeded_ids_resolve` Kani harness will fail if the `technique_info("T0881")`
arm is missing. The `verify_all_emitted_ids_resolve` harness will fail if T0881 is in
`EMITTED_IDS` but not in `technique_info`.

**Mitigation:** The VP-007 atomic obligation must be executed in a single commit per the
established pattern (ADR-007 Decision 4, ADR-010 §VP-007 decision).

### 8.3 Regression Baseline (must not be modified)

The following files must not change and their tests must remain green:

```
src/analyzer/arp.rs          src/analyzer/dnp3.rs
src/analyzer/dns.rs          src/analyzer/enip.rs
src/analyzer/http.rs         src/analyzer/modbus.rs
src/analyzer/tls.rs          src/decoder.rs
src/reader.rs                src/findings.rs
src/summary.rs               src/reassembly/mod.rs
src/reassembly/flow.rs       src/reassembly/handler.rs
src/reassembly/lifecycle.rs  src/reassembly/segment.rs
src/reassembly/config.rs     src/reassembly/stats.rs
```

---

## 9. Story and Wave Estimate

### 9.1 Comparable Cycles

| Cycle | Stories | Story Points | Waves | Notes |
|-------|---------|-------------|-------|-------|
| feature-protocol-coverage | 5 | ~23 | 3 | New catalog module, no parser |
| feature-dnp3 (issue #8) | ~8 | ~30 | 3 | Single-layer transport framing + state machine |
| feature-enip (issue #316) | ~11 | ~42 | 4 | Two-level ENIP/CPF/CIP framing, larger ASDU model |

IEC-104 complexity profile: two-layer APCI/ASDU framing (comparable to ENIP's two levels)
but with a simpler ASDU structure (fixed field widths vs. CIP object model), a session
state machine (STARTDT/STOPDT/TESTFR), and sequence number tracking. Estimated above DNP3,
below ENIP.

### 9.2 Proposed Story Breakdown

| Story | Scope | Points |
|-------|-------|--------|
| STORY-A | Core APCI parser: `parse_apci_header`, LEN bounds, frame-walk loop, 255-byte directional carry, start-byte/LEN validation, `is_valid_iec104_frame` gate; VP-044 Kani harness skeleton | 5 |
| STORY-B | Frame format discrimination: U/S/I from CF1 low 2 bits; ASDU extraction from I-frames; TypeID/VSQ/COT/CASDU/IOA field parsing; VP-046 proptest harness | 5 |
| STORY-C | Control command detection: C_SC/C_DC/C_RC/C_SE/C_BO (TypeIDs 45–51) and C_RP (105) → T1692.001 + T0836 findings; M_* spoofed telemetry (TypeIDs 1,3,5,9,11,13) cataloged as T1692.002 (SEEDED only — NOT emitted this cycle; detection logic staged for future cycle) | 5 |
| STORY-D | U-format session state machine: STARTDT/STOPDT/TESTFR detection, T0881 findings for STOPDT abuse, non-canonical U-frame CF1 anomaly (CVE-2026-1773 angle) → T0814; VP-007 MITRE atomic obligation (T0881 catalog entry) | 5 |
| STORY-E | Sequence counter tracking: N(S)/N(R) 15-bit extraction, k-window desync detection, sequence-gap findings | 3 |
| STORY-F | Dispatcher wiring + VP-004 oracle extension + protocols.rs SUPPORTED_PORTS + `analyzer/mod.rs` declaration + `--iec104` CLI flag + main.rs construction + `take_iec104_analyzer()` | 5 |
| STORY-G | Reporting wiring: `Iec104Summary` struct, terminal reporter IEC-104 section, JSON output | 3 |
| STORY-H | Formal hardening: VP-044 Kani run, VP-045/VP-046 proptest suites, VP-047 fuzz harness, VP-004 proof re-run, cargo-mutants kill-rate sweep on IEC-104 parser | 5 |

**Total: 8 stories / ~36 story points**

### 9.3 Wave Plan

| Wave | Stories | Focus |
|------|---------|-------|
| Wave N | STORY-A, STORY-B | Parser core — new file only, no cross-module changes, LOW regression risk |
| Wave N+1 | STORY-C, STORY-D, STORY-E, STORY-F, STORY-G | Detection + integration — dispatcher and mitre zones touched; MEDIUM regression risk |
| Wave N+2 | STORY-H | Formal hardening — VP-004 Kani re-run, VP-044/VP-047 fuzz/proptest |

**Total: 3 waves (conservative: 4 if VP-044 Kani iteration required)**

---

## 10. Recommended Next Step

Phase F1 is complete pending human approval of scope.

Open questions to resolve before F2:

1. **Scope of ASDU depth:** Should the analyzer parse the full IOA per-object list (variable
   structure qualifier VSQ can specify up to 127 objects per ASDU) or only the first IOA?
   Parsing only the first IOA simplifies the parser and is sufficient for threat detection.
   Recommendation: first IOA only for MVP; full VSQ loop as a future enhancement.

2. **Session state machine scope:** Should the analyzer track STARTDT/STOPDT ACT/CON pairs
   (full four-way handshake) or only detect STARTDT/STOPDT-ACT frames? Recommendation:
   ACT detection only for MVP; ACT/CON pairing as a future enhancement.

3. **T1692.002 promotion:** RESOLVED — T1692.002 is NOT promoted to EMITTED in this
   cycle. M_* spoofed telemetry detection is staged and out of feature-iec104 scope per
   ADR-013 Decision 10. The BC-2.19.* shard contains no BC that emits T1692.002. Promotion
   requires M_* detection logic (TypeID-based passive inference) and would constitute a
   scope expansion beyond what was specified in feature-iec104. Defer to a future cycle
   with its own ADR amendment and BC additions.

4. **ADR-0013 scope:** The new ADR should explicitly enumerate the port-2404 Rule 8
   decision, the 255-byte carry cap rationale, the pure-core free-fn constraint for
   Kani VP-044, and the licensing prohibition. F2 author should use ADR-007 and ADR-010
   as structural templates.

Human approval gate: explicit sign-off on scope (stories A–H, 3 waves, SS-19 new
subsystem) is required before Phase F2 (spec evolution) begins.
