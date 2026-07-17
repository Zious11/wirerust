# Changelog

All notable changes to wirerust are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version numbers follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- **IEC-104 findings now carry `source_ip` and `timestamp` JSON keys (FIX-F5-001,
  BC-2.19.011 PC-3).**

  All IEC-104 `Finding` emit sites previously left `source_ip: None` and
  `timestamp: None`, causing those keys to be absent from IEC-104 JSON output.
  The fix threads the initiator IP (resolved from the 5-tuple `FlowKey` by
  direction, mirroring the DNP3/EtherNet/IP house pattern) and the packet
  timestamp (`ts` parameter) through all 12 `Finding` constructors — 10 via
  function parameters and 2 inline in `on_data`.

  This is an additive, backward-compatible JSON change: the two keys now appear
  on IEC-104 findings where they were previously absent. JSON consumers that
  tolerate unknown keys or use subset/contains assertions are unaffected.

  **Emit sites enriched (8 function + 2 inline = 10 total):**
  - `process_u_frame`: STOPDT-act T0881 + non-canonical U-frame T0814.
  - `detect_iec104_threats`: TypeIDs 45–47 T1692.001, TypeIDs 48–51 T1692.001 +
    T0836, TypeID 105 T0827, TypeIDs 0/128–255 T0814.
  - `track_ns_desync`: N(S) desync T1692.001.
  - `on_data` inline: carry-overflow T0814 + malformed-LEN T0814.

  **Signature changes:** `process_u_frame`, `detect_iec104_threats`, and
  `track_ns_desync` each gain `source_ip: Option<IpAddr>` and
  `timestamp: Option<chrono::DateTime<chrono::Utc>>` parameters (callers updated).

- **Documentation accuracy corrections for FIX-F5-001 and FIX-P4-001 evidence artifacts
  (FIX-F5-002).**

  Corrects three categories of inaccuracy introduced during demo-evidence authoring; no
  source or test code is changed:

  1. **Wrong provenance for sibling source_ip/timestamp enrichment:** The FIX-F5-001
     evidence report (`docs/demo-evidence/FIX-F5-001/evidence-report.md`) incorrectly
     cited STORY-172 and STORY-173 as the origin of DNP3/EtherNet/IP source_ip enrichment.
     Those stories implement IEC-104 carry buffers and the IEC-104 dispatcher respectively;
     the DNP3/EtherNet/IP house pattern for source_ip+timestamp originates from the
     S-139/S-140 lineage (PR #328). Corrected to cite S-139/S-140 (PR #328) and to note
     that IEC-104 additionally populates `direction: Some(direction)`, which DNP3 and
     EtherNet/IP do not.

  2. **Fabricated JSON in Before/After block:** The evidence-report JSON examples contained
     incorrect field values (`category: "anomaly"`, `confidence: "high"`, fabricated summary
     and evidence strings, `direction: "client_to_server"` with wrong casing). Replaced with
     the actual T0881 STOPDT-act finding values from `src/analyzer/iec104.rs` lines 382–396:
     `category: "impact"`, `confidence: "medium"`, real summary string,
     `evidence: ["CF1=0x13 (STOPDT-act)"]`, `direction: "ClientToServer"` (serde default,
     no `rename_all`).

  3. **Wrong year in example timestamps:** Example timestamps using `2025-07-17` corrected
     to `2026-07-17`.

- **IEC-104 findings now carry the `direction` JSON key (FIX-P4-001,
  IEC104-FINDING-DIRECTION-001).**

  All IEC-104 `Finding` emit sites previously left `direction: None`, causing the
  `direction` key to be absent from IEC-104 JSON output (the field uses
  `#[serde(skip_serializing_if = "Option::is_none")]`). This is an additive,
  backward-compatible JSON change — JSON consumers that tolerate unknown keys or use
  subset/contains assertions are unaffected.

  The fix brings IEC-104 direction enrichment into conformance with the TLS / Modbus /
  HTTP analyzers, which already set `direction: Some(direction)` on every emitted finding.
  Note: DNP3 and EtherNet/IP analyzers set `direction: None`; IEC-104's direction
  population therefore exceeds the DNP3/EtherNet/IP baseline (which provides only
  `source_ip` + `timestamp` parity, not direction).

  **Emit sites fixed (10 total):**
  - `process_u_frame`: STOPDT-act T0881 finding + non-canonical U-frame T0814 finding.
  - `detect_iec104_threats`: TypeIDs 45–47 T1692.001, TypeIDs 48–51 T1692.001 + T0836,
    TypeID 105 T0827, TypeIDs 0/128–255 T0814.
  - `track_ns_desync`: N(S) desync T1692.001; redundant `format!("direction=…")` evidence
    line dropped (structured field carries the same information).
  - `on_data` inline: carry-overflow T0814 + malformed-LEN T0814.

  **Signature changes:** `process_u_frame` and `detect_iec104_threats` each gain a
  `direction: Direction` parameter (callers updated).

### Changed

- **`bin/check-green-doc-tense` green-doc-tense gate extended with three new IEC-104 phrasings
  (STORY-174, AC-174-008, PG-REDGREEN-COMMENT-CLEANUP).**

  Adds patterns 23–25 to the `_VIOLATION_PATTERNS` token list to catch stale Red-Gate section
  headers that slipped through the original gate across STORY-167..173 because the existing
  patterns required exact token adjacency:

  - **Pattern 23** (`All tests\b.*\bMUST FAIL`, case-insensitive): catches module/section
    headers with interposed qualifiers such as "All tests in this module MUST FAIL" or "All
    tests in this section MUST FAIL". Subsumes the original pattern 1 for these phrasings.
  - **Pattern 24** (`FAILS?\s+Red Gate`, case-insensitive): catches compile-only-seam
    assertions like "FAILS Red Gate" or "FAIL Red Gate". Past-tense "failed Red Gate" is
    exempt (the 'ed' suffix prevents the `\s+` from matching after "fail").
  - **Pattern 25** (`(?:are|is)\s+todo!\(\)\s+stub`, case-insensitive): catches present-tense
    stub-state assertions like "are todo!() stubs" and "is todo!() stub". Past-tense "were"
    and provenance "originated as" are exempt.

  Three baseline stale headers in `tests/iec104_analyzer_tests.rs` (~L662-663, ~L1498, ~L1544)
  scrubbed to GREEN-accurate prose. Self-test passes at 72/72 cases; tree-wide scan finds 0
  violations after the scrub. No new CI job; extends the existing `green-doc-tense-gate`.

### Added

- **IEC-104 dispatcher integration: `DispatchTarget::Iec104`, `--iec104` flag, T0881 catalog
  entry, port 2404 in `SUPPORTED_PORTS`, and `MAX_IEC104_FINDINGS` cap (STORY-173, wave-82,
  BC-2.05.012, BC-2.10.010, BC-2.12.025, BC-2.18.003, BC-2.18.004, BC-2.19.028,
  ADR-013 Decisions 1/9/10).**

  Wires the IEC-104 passive analyzer into the full wirerust pipeline across five subsystems:

  - **Dispatcher wiring (SS-05, AC-173-008):** `StreamDispatcher` gains `iec104:
    Option<Iec104Analyzer>` field, a 6-parameter `new()`, `set_iec104_analyzer()` setter, and
    `iec104_analyzer()` / `take_iec104_analyzer()` accessors. `on_data` Iec104 arm routes
    port-2404 flow data to `Iec104Analyzer::on_data`; `on_flow_close` Iec104 arm forwards
    to `Iec104Analyzer::on_flow_close`. Early-exit guard extended with `&& self.iec104.is_none()`
    so `--iec104`-only invocations are not silently dropped (ADR-013 Decision 9 steps 4–5).

  - **MITRE catalog — T0881 six-part atomic (SS-10, AC-173-002):** `SEEDED_TECHNIQUE_IDS`
    gains `"T0881"` (28→29 entries); `SEEDED_TECHNIQUE_ID_COUNT` bumped to 29; `EMITTED_IDS`
    updated; `technique_info("T0881")` arm returns `("Service Stop",
    MitreTactic::IcsInhibitResponseFunction)` (TA0107); `vp007_catalog_drift_guard` and
    `verify_all_seeded_ids_resolve` pass at count=29 (ADR-013 Decision 10).

  - **CLI flag (SS-12, AC-173-003):** `--iec104` boolean flag added to `CliArgs`; `main.rs`
    constructs and registers `Iec104Analyzer` when the flag is present (default-off opt-in
    model per BC-2.12.025).

  - **Protocol catalog (SS-18, AC-173-004/005):** port 2404 added to `SUPPORTED_PORTS`
    (count 8→9); `supported_protocols()` count 7→8; VP-041 partition proptest verifies
    supported_protocols() ∪ unsupported_protocols() partitions KNOWN_PROTOCOLS (disjoint,
    complete coverage) after port 2404 addition.

  - **Findings cap (SS-19, AC-173-007 / BC-2.19.028):** `const MAX_IEC104_FINDINGS: usize =
    10_000` added to `src/analyzer/iec104.rs`; `Iec104Analyzer` gains `dropped_findings: u64`
    field; cap enforced at the `on_data` extend step by truncating `local_findings` to the
    remaining capacity and accumulating the discarded count into `dropped_findings`; surfaced
    in `summarize()` as detail key `"dropped_findings"`. Mirrors the DNP3/EtherNet/IP
    `MAX_FINDINGS` pattern (BC-2.15.022 / BC-2.17.022). Per-flow state continues updating
    regardless of the cap.

  - **Real `flows_analyzed` counter (SS-19, STORY-173 LOW#1 / BC-2.19.028 observability):**
    `Iec104Analyzer` gains `flows_analyzed: u64` field (initialized 0); `on_flow_close`
    increments it when `HashMap::remove` returns `Some` (closed-flow count). `summarize()`
    now computes `detail["flows_analyzed"]` as `self.flows_analyzed + self.flows.len()` —
    closed flows plus still-open flows — replacing the previous `self.flows.len()`-only
    value that returned 0 after both flows closed. Mirrors the ENIP `flows_analyzed` and
    DNP3 `closed_flows_count` patterns.

  - **Real `packets_analyzed` counter (SS-19, STORY-173 LOW#2 / BC-2.19.028 observability):**
    `Iec104FlowState` gains `frame_count: u64` (initialized 0 via `Default`); incremented
    once per successful `parse_apci_header` call in the `on_data` frame-walk loop (valid
    start-byte + LEN in [4,253] + full frame available; bad-start-byte skips and
    malformed-LEN stubs are not counted). `Iec104Analyzer` gains `total_frames_closed: u64`;
    `on_flow_close` folds the removed flow's `frame_count` into it. `summarize()` now
    returns `packets_analyzed = self.total_frames_closed + Σ open-flow.frame_count` —
    replacing the previous `all_findings.len()` proxy that returned 0 for finding-free
    frames (e.g. TESTFR-act). Mirrors the DNP3 `total_frames_closed` + open-flow sum pattern.

  - **SEC-001 doc correction (SS-19, STORY-173 SEC-001):** `is_valid_iec104_frame` doc
    rewritten to accurately describe it as a standalone pure predicate and VP-047 fuzz
    seam — not wired as a dispatch gate by design. Its equivalent validation is performed
    inline in the `on_data` frame-walk loop (start-byte check + LEN-range check) per
    walk-first residual-bound anti-evasion semantics (ADR-013 Decisions 1/2). Module-doc
    updated to match.

- **IEC-104 carry buffers + frame-walk loop + flow lifecycle (STORY-172, wave-81,
  BC-2.19.025–027, ADR-013 Decision 3).**

  Implements the outer processing infrastructure for the IEC-104 passive analyzer in
  `src/analyzer/iec104.rs`:

  - `Iec104Analyzer::on_data(flow_key, data, ts, direction)`: effectful shell that
    prepends the directional carry buffer to the delivery, walks the combined buffer
    processing every complete APCI frame, and stashes any incomplete tail back into
    the directional carry. WALK-FIRST-RESIDUAL-BOUND carry-overflow guard (F-172-001):
    the directional carry alone is checked against MAX_IEC104_CARRY_BYTES=255 (not the
    aggregate carry+delivery); if carry.len() > 255 (adversarial state injection;
    unreachable from conformant traffic), the carry is cleared and ONE T0814
    `Anomaly/Possible/Medium` emitted on the first overflow per direction via
    per-direction dedup flags `carry_overflow_reported_c2s` /
    `carry_overflow_reported_s2c` (BC-2.19.025 v1.3 invariants 4–5; SEC-001-S168
    defense-in-depth). The delivery is always walked regardless — no delivery is ever
    discarded before frame extraction (anti-evasion per F-172-001 and Ptacek/Newsham
    1998; BC-2.19.025 invariant 2). Malformed-LEN frames (valid 0x68 + LEN outside
    [4, 253]) advance 2 bytes and emit ONE T0814 on the first occurrence per direction
    via per-direction dedup flags `malformed_len_reported_c2s` /
    `malformed_len_reported_s2c` (BC-2.19.026 invariant 5; EMIT-WITH-DEDUP). Bad start
    bytes advance 1 byte with no finding. Complete valid frames are dispatched to
    `process_u_frame`, `parse_asdu` + `detect_iec104_threats`, or `track_ns_desync`
    per frame format. VP-047 fuzz target (`fuzz_iec104_parser`).

  - `Iec104Analyzer::on_flow_close(flow_key)`: removes the `Iec104FlowState` entry
    from the flow map; carry bytes are silently discarded (dropped with the state);
    no finding emitted; unknown flow keys are a no-op (BC-2.19.027).

  - `Iec104FlowState` now fully wired with all 9 fields: `carry_c2s`, `carry_s2c`,
    `session_started`, `last_ns_c2s`, `last_ns_s2c`, `malformed_len_reported_c2s`,
    `malformed_len_reported_s2c`, `carry_overflow_reported_c2s`,
    `carry_overflow_reported_s2c`.

  - VP-045 proptest skeletons `proptest_vp045_direction_isolation` and
    `proptest_vp045_independent_run_equivalence` in `tests/iec104_analyzer_tests.rs`
    verify carry direction isolation (full execution in STORY-174).

- **IEC-104 N(S)/N(R) extraction + `Option<u16>` first-frame-baseline desync detection
  (STORY-171, wave-80, BC-2.19.023–024, ADR-013 Decision 6).**

  Implements N(S) sequence-number tracking and desynchronization detection in
  `src/analyzer/iec104.rs`:

  - `extract_ns(cf1, cf2) -> u16`: pure-core free function extracting the 15-bit send
    sequence number from I-format CF1/CF2 bytes via
    `((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` — range [0, 32767]
    (BC-2.19.023 postcondition 1).

  - `extract_nr(cf3, cf4) -> u16`: pure-core free function extracting the 15-bit receive
    sequence number from I/S-format CF3/CF4 bytes via the symmetric formula.
    N(R) is transient — not stored in `Iec104FlowState` (BC-2.19.023 postcondition 4).

  - `track_ns_desync(state, current_ns, direction) -> Option<Finding>`: effectful function
    implementing the three-path `Option<u16>` first-frame guard and k=12 window check:
    - **Path A** (state `None`): sets `Some(current_ns)` baseline; NO finding unconditionally.
      Prevents false positives on mid-capture starts where first N(S) is arbitrary
      (BC-2.19.024 postcondition A; ADR-013 Decision 6 invariant 3).
    - **Path B** (state `Some(prev)`, 15-bit gap ≤ 12): updates state; no finding
      (BC-2.19.024 postcondition B).
    - **Path C** (state `Some(prev)`, 15-bit gap > 12): updates state and emits T1692.001
      "Unauthorized Message: Command Message" with `Verdict::Possible`,
      `ThreatCategory::Impact` — sequence desynchronization or replay injection detected
      (BC-2.19.024 postcondition C).
    - Gap uses `current_ns.wrapping_sub(prev) & 0x7FFF` — the `& 0x7FFF` mask is
      mandatory to collapse `wrapping_sub`'s 2^16 wrap to the 15-bit N(S) range
      (BC-2.19.024 invariant 1).
    - `Direction::ClientToServer` selects `last_ns_c2s`; `Direction::ServerToClient`
      selects `last_ns_s2c` — directional fields updated independently (AC-171-007).

- **IEC-104 control command detection: `detect_iec104_threats` (STORY-170, wave-79,
  BC-2.19.017/019–022, ADR-013 Decision 8).**

  Implements TypeID dispatch for the IEC-104 passive analyzer in `src/analyzer/iec104.rs`:

  - TypeIDs 45–47 (C_SC_NA_1, C_DC_NA_1, C_RC_NA_1 — switching commands): emit T1692.001
    "Unauthorized Message: Command Message" with `Verdict::Possible`, `ThreatCategory::Impact`
    (BC-2.19.019 postcondition 1; invariant 2).

  - TypeIDs 48–51 (C_SE_NA_1, C_SE_NB_1, C_SE_NC_1, C_BO_NA_1 — set-point/bitstring writes):
    emit T1692.001 Possible AND T0836 "Modify Parameter" Possible — two findings per ASDU
    (BC-2.19.019 postconditions 1–2).

  - TypeID 105 (C_RP_NA_1 — Reset Process Command): emit T0827 "Loss of Control" with
    `Verdict::Likely` (BC-2.19.020; v1.1 correction: Likely, not Possible).

  - TypeIDs 100, 101, 103 (C_IC_NA_1, C_CI_NA_1, C_CS_NA_1 — interrogation/clock-sync):
    no finding emitted — benign administrative commands (BC-2.19.021 postcondition 1).

  - TypeID=0 or TypeID in [128, 255] (undefined/private-use/reserved): emit T0814 "Denial
    of Service" with `Verdict::Possible`, `ThreatCategory::Anomaly` (BC-2.19.022
    postcondition 1). TypeIDs in [1, 127] not in any detection set are silently logged
    with no finding (BC-2.19.022 invariant 1).

  - `cot_test=true` tagging: when `asdu.cot_test == true`, ` [TEST]` is appended to every
    emitted finding's `summary` field for analyst noise reduction (BC-2.19.017 invariant 1;
    AC-170-007).

  - **CASDU and first_ioa target-address context in findings (Pass-1 adversarial
    remediation, F-170-001; BC-2.19.019 postcondition 3; BC-2.19.020 postcondition 2).**
    Every finding emitted by `detect_iec104_threats` now includes `"CASDU=<value>"` as
    an evidence entry (always present) and `"first_ioa=<decimal>"` when
    `asdu.first_ioa` is `Some` — enabling analysts to identify which RTU/IED and IO
    address was targeted by the control command. Applied to all four finding-emitting
    arms: TypeIDs 45–47 (T1692.001), 48–51 (T1692.001 + T0836, both findings), 105
    (T0827), and 0/128–255 (T0814). The `[TEST]` tagging path operates on `summary`
    only and is unaffected by the evidence additions.

  45 new tests in `tests/iec104_analyzer_tests.rs` (mod `story_170`); combined IEC-104 suite
  is now 136 tests (story_167: 30, story_168: 34, story_169: 27, story_170: 45).
  Pass-1 adversarial additions (F-170-001): `test_F_170_001_casdu_appears_in_finding_evidence_for_control_type`,
  `test_F_170_001_first_ioa_appears_in_finding_evidence_when_some`,
  `test_BC_2_19_017_start_idx_guard_preexisting_finding_not_tagged`.

- **IEC-104 ASDU DUI header extraction: `parse_asdu` + `Asdu` struct (STORY-169, wave-78,
  BC-2.19.015–018, ADR-013 Decision 8).**

  Adds pure-core ASDU header extraction to `src/analyzer/iec104`:

  - `Asdu` struct with nine broken-out DUI fields: `type_id` (u8), `sq` (bool), `count` (u8),
    `cot_cause` (u8), `cot_pn` (bool), `cot_test` (bool), `cot_originator` (u8), `casdu` (u16),
    `first_ioa: Option<u32>`. No packed `vsq: u8` or `cot: u16` fields (ADR-013 Decision 3).

  - `parse_asdu(asdu_body: &[u8]) -> Option<Asdu>`: pure-core free function. Returns `None`
    when `asdu_body.len() < 6` (6-byte DUI minimum guard; BC-2.19.015; caller emits T0814).
    On the accept path, extracts all nine fields: TypeID verbatim from byte 0; SQ and count
    from VSQ byte 1 (BC-2.19.016); COT cause/P-N/T/originator from bytes 2–3 (BC-2.19.017);
    CASDU as 16-bit LE from bytes 4–5 (BC-2.19.018). `first_ioa` is
    `Some(24-bit LE zero-extended to u32)` when `count > 0` AND `len >= 9`; `None` otherwise
    (BC-2.19.018). No panic for any input (VP-047 fuzz seam).

- **IEC-104 frame format discrimination + U-format session state machine (STORY-168, wave-77,
  BC-2.19.007–014, ADR-013 Decisions 4/5; T0881/T0814 emission).**

  Extends `src/analyzer/iec104` with pure-core frame classification and an effectful U-frame
  session state machine:

  - `classify_frame_format(cf1: u8) -> FrameFormat`: pure-core free function; total over all
    256 u8 CF1 values; no panic (BC-2.19.007–009; VP-046 proptest; ADR-013 Decision 4).
    Classifies by low 2 bits of CF1: bit 0 = 0 → IFormat, bits1:0 = 0b01 → SFormat,
    bits1:0 = 0b11 → UFormat.

  - `process_u_frame(state: &mut Iec104FlowState, cf1: u8) -> Option<Finding>`: effectful
    session state machine for STARTDT/STOPDT/TESTFR U-frames (ADR-013 Decision 5;
    BC-2.19.010–014). Dispatch table:
    - STARTDT-act (0x07) / STARTDT-con (0x0B): `session_started = true`; no finding.
      Idempotent (BC-2.19.010).
    - STOPDT-act (0x13): emits T0881 "Service Stop" `Impact/Possible` (if session active) or
      `Impact/Likely` (if no prior STARTDT — anomalous stop); sets `session_started = false`
      (BC-2.19.011/012).
    - STOPDT-con (0x23): `session_started = false`; no finding (ACT-only MVP; BC-2.19.012).
    - TESTFR-act (0x43) / TESTFR-con (0x83): no finding; session state unchanged (BC-2.19.013).
    - Non-canonical U CF1 (any other value with bits1:0 = 0b11): emits T0814 `Anomaly/Possible`
      (CVE-2026-1773 fail-closed; BC-2.19.014). Session state NOT advanced.

  - `Iec104FlowState::session_started: bool` field: initialized `false` via `Default`;
    governs T0881 confidence escalation (BC-2.19.010–012).

  - VP-046 proptest skeleton `proptest_vp046_frame_format_totality` exercising all 256 CF1
    values (AC-168-009; full proof run in STORY-174).

  34 new tests in `tests/iec104_analyzer_tests.rs` (mod story_168) covering all
  BC-2.19.007–014 postconditions, edge cases, and VP-046 proptest. All 64 IEC-104 tests
  (30 STORY-167 + 34 STORY-168) pass; no pre-existing regressions.

  **Pass-1 adversarial remediation (STORY-168 wave-77):** T0881 Likely-path finding
  (STOPDT-act without prior STARTDT) now includes a distinguishing evidence entry
  "STOPDT received without prior STARTDT on this flow" (BC-2.19.012 postcondition 3).
  This makes the cold-start anomaly self-describing without requiring session-timeline
  correlation by the analyst.

- **IEC-104 APCI core parser: `parse_apci_header` pure-core free function + VP-044 Kani
  skeleton (STORY-167, wave-76, BC-2.19.001–006, ADR-013 Decisions 1/3/8).**

  New `src/analyzer/iec104` module implementing the IEC 60870-5-104 (IEC-104) APCI header
  parser as a pure-core free function with zero external dependencies (ADR-013 Decision 7
  licensing constraint — no `iec60870-5`, Wireshark, or lib60870 code):

  - `parse_apci_header(data: &[u8]) -> Option<ApciHeader>`: returns None for input shorter
    than 6 bytes (BC-2.19.001), start byte ≠ 0x68 (BC-2.19.002), LEN < 4 (BC-2.19.003), or
    LEN > 253 (BC-2.19.004); returns `Some(ApciHeader)` with CF1–CF4 extracted verbatim from
    bytes [2..6] for valid input (BC-2.19.005). Overflow-safe: `len + 2` ≤ 255 for all valid
    LEN values. VP-044 Kani formal verification target (full proof run: STORY-174).

  - `is_valid_iec104_frame(data: &[u8]) -> bool`: lightweight post-classification gate for
    port-2404-dispatched flows (BC-2.19.006). Returns true iff `data.len() >= 2`,
    `data[0] == 0x68`, and `4 <= data[1] <= 253`. Consistent with `parse_apci_header`:
    gate-true ∧ data.len() >= 6 ⟹ parse returns Some (BC-2.19.006 invariant 2).

  - `ApciHeader` struct (`start`, `len`, `cf1`–`cf4`; all `u8`; `#[derive(Debug, Clone,
    PartialEq, Eq)]`).

  - `Iec104ParseError` error enum skeleton (extended in STORY-168).

  - VP-044 Kani harness skeleton under `#[cfg(kani)]` (ADR-013 Decision 8; full proof:
    STORY-174).

  30 new tests in `tests/iec104_analyzer_tests.rs` covering all BC-2.19.001–006
  postconditions, boundary values, and cross-function invariants (no pre-existing test
  regressions).

## [0.12.1] - 2026-07-13

### Added

- **`bin/validate-citations`: mechanical citation preflight validator (STORY-164,
  AC-164-002, wave-74, PG-W73-CITATION-VALIDATOR; F-S164P1-002/004 +
  F-S164P2-002/003/004 remediation).**

  New Python 3.10+ stdlib tool that reads a citations table (file argument or
  stdin) and verifies each `path:LINE` / `path:LINE-LINE` anchor against the
  filesystem. Exits 0 when all citations are valid, 1 on any failure (FILE NOT
  FOUND / INVALID LINE / INVALID RANGE / LINE OUT OF RANGE / MALFORMED / NOT A
  FILE / OUTSIDE REPO / UNREADABLE), 2 on usage error. Paths are resolved
  relative to the repo root (WIRERUST_REPO_ROOT or upward walk). Self-tested by
  `bin/test_validate_citations.py` (22 tests).

  F-S164P1-002: non-blank, non-comment lines that do not match the citation regex
  are now reported as `MALFORMED: <line>` and cause exit 1 rather than being
  silently skipped (false PASS). F-S164P1-004: line numbers less than 1 (e.g.
  `file.md:0`, `file.md:0-5`) are now rejected as `INVALID LINE` rather than being
  silently accepted as in-bounds.

  F-S164P2-002: the `FAIL: K of N` denominator N now counts every non-blank,
  non-comment line (valid citations and MALFORMED lines alike), so a malformed-
  only input correctly reports `FAIL: 1 of 1` rather than `FAIL: 1 of 0`.

  F-S164P2-003 (CWE-22): absolute paths and parent-directory escapes are now
  rejected with `OUTSIDE REPO: <path>`. Python's pathlib `/` operator discards
  the left side for absolute right-hand values, so `repo_root / '/etc/passwd'`
  silently became `/etc/passwd`; `.resolve()` + `.is_relative_to()` containment
  catches both absolute and `../` traversal forms. Parity with the same class
  identified for `bin/compute-input-hash` in GitHub #392 (not fixed here;
  deferred to the #392 issue).

  F-S164P2-004 / F-S164P3-003: unreadable or non-UTF-8 citations files now
  produce a documented exit-2 usage error rather than an uncaught traceback.
  `UnicodeDecodeError` (non-UTF-8 bytes) and `OSError` (PermissionError,
  IsADirectoryError, etc.) are both caught; each emits a descriptive `Error:`
  message to stderr and exits 2. Test T19 covers the `chmod 000` path, with a
  skip guard when the process can read mode-0 files (root environments).

  F-S164P6-001: the stdin branch (`sys.stdin.read()`) diverged from the
  file-argument path — non-UTF-8 bytes on stdin raised an uncaught
  `UnicodeDecodeError` traceback and exited 1. Fixed by reading
  `sys.stdin.buffer` (raw bytes) and decoding explicitly; the same
  `UnicodeDecodeError` catch now applies, emitting a `Error: stdin is not
  valid UTF-8:` message and exiting 2. Test T20 covers this path.

  F-S164P8-001: cited target files were validated for existence but not for
  being a regular file or being readable. A citation to a directory (e.g.
  `docs:5`) passed `exists()` then crashed with `IsADirectoryError` traceback
  in `count_lines()`. A citation to a chmod-000 file produced a
  `PermissionError` traceback. Fixed by adding an `is_file()` check (→ `NOT A
  FILE: <path>`, exit 1) between `exists()` and the INVALID LINE guards, and
  wrapping the `count_lines()` call in `try/except OSError` (→ `UNREADABLE:
  <path>`, exit 1). Tests T21 (directory) and T22 (chmod 000, root-skipped)
  cover both paths.

  Addresses PG-W73-CITATION-VALIDATOR: the wave-73 gate adversarial review found
  CRITICAL-severity fabricated citations in STORY-163's own evidence artifact.
  This tool provides a mechanical preflight gate so such errors are caught before
  dispatch rather than by the adversary.

- **`bin/changelog-gate-check`: extracted changelog-gate content assertion
  (STORY-164, AC-164-003, wave-74, PG-W73-CHANGELOG-GATE-CONTENT; F-S164P1-001
  + F-S164P2-001 remediation).**

  The changelog-gate content-assertion logic is extracted from `.github/workflows/
  ci.yml` into `bin/changelog-gate-check` (a standalone bash script invoked by
  ci.yml). This enables the gate logic to be directly exercised by behavioral tests.

  F-S164P1-001 (HIGH): the original inline CONTENT_LINES pipeline lacked `||
  true` on its terminal grep, causing `set -euo pipefail` to kill the CI step
  before the `-eq 0` diagnostic branch could run — making the blank-only-touch
  FAIL path dead code. The fix wraps the grep chain in `{ ... || true; }` so
  an empty selection reliably resolves to `CONTENT_LINES=0` and the diagnostic
  message prints. `bin/test_changelog_gate_content.py` gains five behavioral
  tests (B01–B05) that execute the gate script against crafted diff fixtures
  (real content, blank-only, header-only, deletions-only, direct-path exec-bit
  guard) and were confirmed FAIL against the broken logic, PASS after the fix.

  F-S164P2-001: test B05 added to invoke the script via its direct path (no
  `bash` prefix), verifying the committed git file mode is 100755 and the
  shebang is valid. ci.yml uses the bare path `bin/changelog-gate-check`; a
  missing exec bit would fail CI with exit 126 while all bash-prefixed tests
  stayed green. The script was already committed at 100755; B05 is the guard.

### Changed



- **`bin/check-green-doc-tense`: extract `_find_repo_root` helper + add hermetic
  main()-guard self-tests (STORY-162, wave-73, F-W72G-P2-OBS-001).**

  The repo-root sentinel walk in `main()` is extracted into a standalone
  `_find_repo_root(start: Path) -> Path | None` helper that walks upward up to
  6 levels looking for a `.git` entry (file or directory) or a `.factory/`
  subdirectory. `main()` now delegates to this helper, enabling hermetic monkey-
  patching in tests without relying on the live `.git` or `.factory/` of the
  develop checkout.

  `bin/test_check_green_doc_tense.py` gains five new hermetic self-tests
  (AC-162-003 / AC-162-004, F-W72G-P2-OBS-001):
  - Three `_find_repo_root` unit tests verifying the `.factory/` OR-sentinel,
    `.git` directory sentinel, and `.git` file (worktree) sentinel arms.
  - One no-sentinel regression guard asserting `_find_repo_root` returns `None`
    or an ancestor outside the temp tree when neither `.git` nor `.factory/`
    is present.
  - One precision exit-code test asserting `main()` returns exactly `1` (zero-file
    guard) rather than `2` (root-not-found guard) when `_collect_rust_files` returns
    `[]` and a repo root is reliably found via a hermetic temp fixture.

  Codifies wave-72 process-gap F-W72G-P2-OBS-001 per S-7.02 cycle-close obligation
  (F-S161P1-001 / VP-INDEX LMR-003 template-conformance exemption on factory side).

- **maint-2026-07-11 cleanup — CR-001/002/003 + doc drift + dnp3 lint hygiene.**

  - `bin/test_check_green_doc_tense.py` AC-158-005 test: add hermetic `_find_repo_root`
    patch (mirroring AC-162-003 pattern); tighten assertion from `exit_code != 0` to
    `exit_code == 1`; restore both patched helpers in `finally` (CR-001).
  - `bin/check-green-doc-tense` `_find_repo_root` docstring/comment: reword to
    "at most 6 candidates (start inclusive)" to match `range(6)` behavior (CR-002).
  - `bin/test_check_green_doc_tense.py` test (c): replace `str.startswith` with
    `Path.is_relative_to()` for correct filesystem-hierarchy containment check (CR-003).
  - README.md `--arp-storm-rate` option description: add `>=` directional semantics and
    calibration note (README-OPTIONS-L117-NEUTRAL-001).
  - README.md ARP JSON schema note: correct `arp_summary` key claim — ARP counters are
    flat in `analyzers[i].detail`, not nested under `arp_summary` (PG-W-README-JSON-SCHEMA).
  - README.md DNP3 threshold-tuning note: add bidirectional-flow assumption and mirror-tap
    guidance (DNP3-TUNING-BIDIR-001).
  - `docs/adr/0002-modular-protocol-analyzers.md`: correct tech-debt item ID `PC-023` →
    `PC-020` for `EnipAnalyzer` `StreamHandler` deviation (DOC-NEW-001).
  - `docs/adr/0001-content-first-stream-dispatch.md`: add `unclassified_port_counts` and
    `coverage_gaps_enabled` fields to the `StreamDispatcher` struct snippet (NEW-003).
  - `CHANGELOG.md` v0.7.0 D3 ARP-storm entry: add inline errata noting `mitre_techniques: []`
    per DF-VALIDATION-001 / BC-2.16.008 Invariant 3 (CHANGELOG-D3-T0830-DRIFT-001).
  - `src/cli.rs` Modbus arg doc-comments: harmonize "1-second window" → "1s window" for
    consistency with adjacent arg format (UNIT-FMT-5-20S-001).
  - `src/analyzer/arp.rs` `detect_storm` doc-comment: note integer truncation in rate
    formula (ARP-RATE-INTDIV-DOC-001).
  - `src/analyzer/dnp3.rs`: remove 9 spurious `#[allow(unused)]` attributes from
    actively-used `pub const` items (PC-NEW-001); add rationale comments to 3 of the
    6 `#[allow(clippy::too_many_arguments)]` suppressions — the 3 that lacked them;
    the remaining 3 carried pre-existing `// N args: …` rationale (PC-NEW-002).
  - `src/analyzer/dnp3.rs` `Dnp3FlowState` doc-comment: reword stale present-tense
    "are stubs … contain no logic yet" to past-tense provenance "were stubs through
    STORY-107 and are fully implemented as of STORY-108/109" (F-P1-001).
  - `src/analyzer/arp.rs` two test doc-comments: remove stale "RED GATE: these two new
    keys are absent from the current summarize() implementation" — both keys are fully
    implemented and the tests are GREEN (F-P1-001 sibling sweep, DF-GREEN-DOC-TENSE-SWEEP).
  - `src/analyzer/arp.rs` doc-comment count sweep: correct five remaining "eleven" →
    "thirteen" occurrences (module doc, `summarize()` API doc, section comment); add
    `bindings_evicted` and `storm_counters_evicted` to the `summarize()` key-contract
    enumeration (F-P2-001, DF-SIBLING-SWEEP-001).

## [0.12.0] - 2026-07-10

### Changed (BREAKING)

- **`verdict`, `confidence`, and `category` JSON field values aligned to lowercase/snake_case
  (STORY-160, wave-72, BC-2.11.036, issue #255).**

  > **BREAKING CHANGE (JSON surface only — v0.12.0).**
  > JSON schema changes are outside `cargo-semver-checks` scope; this entry and the
  > `schema_version` envelope field (see `### Added` below) are the authoritative change
  > notices.

  1. **`verdict`, `confidence`, and `category` JSON field values are now lowercase /
     snake_case** (Suricata EVE / ECS / OCSF convention).
     Full mapping (BC-2.11.036):

     | Enum | Variant | Pre-v0.12.0 JSON | v0.12.0+ JSON |
     |---|---|---|---|
     | `Verdict` | `Likely` | `"Likely"` | `"likely"` |
     | `Verdict` | `Unlikely` | `"Unlikely"` | `"unlikely"` |
     | `Verdict` | `Inconclusive` | `"Inconclusive"` | `"inconclusive"` |
     | `Verdict` | `Possible` | `"Possible"` | `"possible"` |
     | `Confidence` | `High` | `"High"` | `"high"` |
     | `Confidence` | `Medium` | `"Medium"` | `"medium"` |
     | `Confidence` | `Low` | `"Low"` | `"low"` |
     | `ThreatCategory` | `LateralMovement` | `"LateralMovement"` | `"lateral_movement"` |
     | `ThreatCategory` | `CredentialAccess` | `"CredentialAccess"` | `"credential_access"` |
     | `ThreatCategory` | `C2` | `"C2"` | `"c2"` |
     | `ThreatCategory` | `Reconnaissance` | `"Reconnaissance"` | `"reconnaissance"` |
     | `ThreatCategory` | `Exfiltration` | `"Exfiltration"` | `"exfiltration"` |
     | `ThreatCategory` | `Persistence` | `"Persistence"` | `"persistence"` |
     | `ThreatCategory` | `Execution` | `"Execution"` | `"execution"` |
     | `ThreatCategory` | `Anomaly` | `"Anomaly"` | `"anomaly"` |
     | `ThreatCategory` | `Suspicious` | `"Suspicious"` | `"suspicious"` |
     | `ThreatCategory` | `Impact` | `"Impact"` | `"impact"` |

  2. **Terminal Display tokens (`"LIKELY"`, `"HIGH"`) and CSV output are UNCHANGED.**
     The `fmt::Display` implementations for `Verdict`, `Confidence`, and `ThreatCategory`
     are not modified; `serde::Serialize` and `fmt::Display` are independent surfaces.

  3. **JSON schema changes are outside `cargo-semver-checks` scope.** Consumers that
     pattern-match exact enum string values in JSON output (e.g., `verdict == "Likely"`,
     `category == "LateralMovement"`) must update to the new lowercase/snake_case forms.

  4. **Known heterogeneity:** The `Direction` enum (`ClientToServer` / `ServerToClient`)
     retains PascalCase JSON serialization in v0.12.0. Casing alignment is scoped to
     `verdict`, `confidence`, and `category` only (BC-2.11.036 scope carve-out).

### Added

- **`"schema_version": "2"` envelope field in every JSON report (STORY-160, BC-2.11.037).**
  Absence of this field signals the pre-v0.12.0 format (implicit schema v1, PascalCase enum
  values). The value is a JSON **string** (not an integer) to remain forward-compatible with
  minor revision suffixes.

- **CHANGELOG CI gate, `bin/lint-cycle-artifact`, and `bin/check-green-doc-tense`
  zero-file-guard hardening (STORY-158, wave-72) [process-gap].** Four wave-71 process
  gaps codified as durable project artifacts: (1) `changelog-gate` CI job (pull_request
  only) fails when `src/`, `Cargo.toml`, or `bin/` are modified without a corresponding
  `CHANGELOG.md` update, enforcing the CHANGELOG obligation that wave-71 PRs missed
  (PG-W71-CHANGELOG). (2) `bin/lint-cycle-artifact` (Python 3, stdlib-only) validates
  cycle artifact identity fields (`story_id:` and `bcs:` frontmatter) against the parent
  story and on-disk BC files, catching fabricated or borrowed BC IDs before adversarial
  review (PG-W71-CYCLE-ARTIFACT-IDENTITY). (3) `bin/check-green-doc-tense` now exits
  non-zero when no tracked Rust files are found, preventing a silent false-CI-PASS if the
  scan target moves (PG-W71-CI-SCAN-GUARDS). (4) `trust-boundary` CI job gains a
  `test -d src/` guard before the grep scan, mirroring the SEC-001 pattern in
  `help-provenance-gate` (PG-W71-CI-SCAN-GUARDS).

- **Fragmented-handshake Criterion benchmark `tls_fragmented/3-record-carry-drain` +
  `[[bench]]` target (STORY-149, PR #374, closes #360).** A new Criterion benchmark
  exercises the TLS carry-path under realistic 3-record fragmented-handshake conditions,
  providing a regression fixture for the restructured carry path. CI-gated
  bounded-borrow source-inspection tests (`tests/bc_149_single_borrow_invariant_tests.rs`)
  verify the single-borrow invariant holds across the restructured code paths.

### Changed

- **TLS carry-path restructured for single-borrow HashMap access (STORY-149, PR #374).**
  `try_parse_records` refactored into three cooperating functions (`prepare_record_step`,
  `RecordStep`, `process_handshake_carry`) to eliminate a double-borrow on the per-flow
  carry buffer. The `reassembly/tls.pcap` Criterion benchmark recovered −7.88% regression
  (23.841 µs measured, +2.41% vs May-19 anchor — within the ±5 % recovery target). Zero
  behavior change: 8-pass adversarial convergence and holdout re-evaluation score 0.920
  unchanged.

- **TLS handshake drain-loop DRY unification in `process_handshake_carry` (STORY-150,
  PR #379).** Single `msg_bytes` extraction and single `parse_tls_message_handshake` call
  site with direction-guarded dispatch arms replace two duplicated extraction+parse sequences
  (defense-in-depth refactor). Behavior-preserving: Kani VP-039 3/3 proofs re-verified,
  zero new mutation survivors.

- **Bumped `indicatif` 0.18.5 → 0.18.6** (Windows dumb-terminal detection fix, indicatif#818, dependabot #386).

### Fixed

- **Absolute host paths scrubbed from 193 committed demo-evidence files (PR #376,
  F-W70P2-002).** All absolute host filesystem paths in `docs/demo-evidence/` have been
  replaced with `<REPO-ROOT>` and `<HOME>` placeholder tokens. These are scrub markers —
  not environment variables — indicating where former machine-specific paths appeared.
  See `docs/DEMO-EVIDENCE.md` for the placeholder convention.

- **Factory input-hash tool edge cases: empty inputs and inline comment stripping
  (STORY-157, PR #380).** `bin/compute-input-hash`: `inputs: []` (empty inputs list) now
  derives hash `d41d8cd` (MD5 of empty bytes) instead of raising an error; inline
  ` # comment` suffixes are stripped from input path entries before file resolution.
  `CLAUDE.md` documents the canonical-tool/hook divergence (PG-HASH-HOOK-DIVERGENCE),
  edge cases, and Python 3.10+ floor.

### Tests / Internal

- **BC-2.16.016 ARP unbounded-findings coverage (STORY-156, PR #378).** Standalone
  `summarize()` no-`dropped_findings` regression pin closes the coverage gap for
  BC-2.16.016 unbounded-findings behavior; docstring anchor corrected. CLI `--arp`
  `long_help` unbounded-findings documentation coverage pinned.

### Docs / Internal

- **Governance codification: multi-file `proof_file_hash` mini-Merkle algorithm (STORY-161,
  wave-72) [governance].** Multi-file `proof_file_hash` mini-Merkle algorithm codified in
  VP-INDEX v2.39; VP-024 v2.5 proof anchor populated and `kani_version` recorded
  (factory-artifacts branch). `CLAUDE.md` gains "Two Hash Disciplines" note distinguishing
  `input-hash` (MD5-first-7, advisory) from `proof_file_hash` (SHA-256 mini-Merkle,
  integrity anchor).

- **Public ADR-012 authored for protocols catalog and coverage-gaps system (STORY-159,
  wave-72) [doc-drift].** `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` created,
  resolving a maintenance-sweep finding (NEW-001, HIGH) that identified 38 lines across
  six source and test files citing ADR-012 with no corresponding public document. The new
  ADR covers all ten design decisions from the `feature-protocol-coverage` cycle (v0.11.2,
  PRs #351–#357): hand-curated static array, tri-state Suricata-derived vocabulary,
  port-detection caveats, catalog scope, supported-set derivation, TCP+UDP dynamic
  detection (including Decision 6 Clarification on increment-site semantics), category
  tagging, `--coverage-gaps` explicit flag, `CoverageGapsSummary` report section, and UDP
  gap classification decoupled from `enable_dns`. Format follows the ADR-0009 precedent:
  markdown headings, no YAML frontmatter, all internal factory IDs stripped.
  `CLAUDE.md` Project References table updated to include the new entry. Inline comment at
  `tests/integration_tests.rs:1166` normalized from `ADR-012 Dec 10` to the canonical
  `ADR-012 Decision 10` form, closing the one abbreviated citation in the codebase.

- **Wave-72 integration-gate hardening: `action-pin-gate` existence guard + positive
  coverage assertion; STORY-159 tape path scrub (F-W72G-P1-001, SEC-W72-001, wave-72).**
  (1) `action-pin-gate` CI job gains a scan-target existence guard
  (`test -d .github/workflows/` + zero-file check) that mirrors the SEC-001 pattern from
  `trust-boundary` and `help-provenance-gate` (PG-W71-CI-SCAN-GUARDS): a renamed or emptied
  scan target now fails loudly instead of trivially PASSing. A positive-coverage assertion
  (`VALIDATED` counter) ensures the gate processed at least one remote action ref; the PASS
  line now reports the validated count (e.g., "PASS: N remote action ref(s) validated, 0
  mutable"). (2) Five STORY-159 VHS tape scripts in `docs/demo-evidence/STORY-159/` had
  `~/Documents/GITHUB/wirerust` absolute host paths in their `Type "cd …"` lines; scrubbed
  to `<REPO-ROOT>` matching the STORY-160 tape convention (SEC-W72-001, CWE-200). Binary
  `.gif`/`.webm` artifacts are historical evidence and not re-rendered. Note: the
  demo-evidence scrub-gate doc (`.factory/maintenance/demo-evidence-scrub-gate.md`) needs a
  `~/` tilde-expansion pattern extension — that file lives on factory-artifacts and is
  routed to the orchestrator separately.

## [0.11.5] - 2026-07-06

### Added

- **Three DNP3 observability counters surfacing previously silent analyzer state (PR #370,
  BC-2.15.016/020/022).** Three new monotonic counters expose DNP3 resource-cap events that
  were previously invisible to operators. All counters are purely additive — new keys appear in
  `summarize()` JSON output only; detection logic, Finding emission, and all behavioral
  invariants are unchanged.

  - **`dropped_findings`** (DNP3 summary) — incremented at each of 11 `MAX_FINDINGS` cap-check
    sites when a finding is suppressed. `MAX_FINDINGS = 10_000`; the counter makes finding-cap
    pressure observable. (BC-2.15.022)

  - **`master_addrs_dropped`** (DNP3 summary) — incremented when a new-unique master address is
    silently ignored because the `MAX_MASTER_ADDRS = 64` cap is full. Existing-address hits do
    not increment the counter. (BC-2.15.016 v2.1 PC-6)

  - **`pending_requests_evicted`** (DNP3 summary) — incremented when `insert_pending_request`
    LRU-evicts an entry. Follows the `insert_binding_lru`-returns-bool pattern established in
    PR #366 (ARP). (BC-2.15.016 v2.1 PC-10)

  Direct precedent: PR #365 / #366 added the equivalent counter pattern for ARP
  (`bindings_evicted`, `storm_counters_evicted`), Modbus (`dropped_transactions`), and
  HTTP/TLS (`dropped_map_entries`). The DNP3 counters close the remaining observability gap in
  the silent-limit audit.

### Security

- **Bumped `crossbeam-epoch` 0.9.18 → 0.9.20** to clear RUSTSEC-2026-0204 (invalid pointer
  dereference in the `fmt::Pointer` implementation of `crossbeam-epoch` 0.9.18, fixed in
  0.9.20). This is a dev-dependency-only transitive dependency (`criterion` → `crossbeam-epoch`)
  with no exposure in production builds. (PR #371)

### Docs / Internal

- **Doc-drift fixes from maint-2026-07-06 sweep (PR #369).** Closes all HIGH and MEDIUM
  documentation-drift findings from the maintenance sweep:

  - **README** — added the `protocols` subcommand to the CLI reference section (was absent
    despite being a live, shipped command since v0.11.2).

  - **ADR-0001 (stream dispatch) and ADR-0002 (modular analyzers)** — added EtherNet/IP (ENIP)
    to both ADRs, which previously omitted it despite ENIP being a full Rule 7 dispatcher
    entry since v0.11.0. `src/lib.rs` public module docs updated to match.

  - **Observability counter documentation** — added user-facing documentation for the
    `--counters` / observability counter surface introduced in v0.11.3.


## [0.11.4] - 2026-07-06

### Added

- **Four observability counters surfacing previously silent analyzer state (PR #365, BC-INDEX
  v2.18).** The silent-limit observability audit is now closed. Four counter fields are newly
  emitted in `summarize()` JSON output and terminal output across four analyzers:

  - **`bindings_evicted`** (ARP summary) — cumulative count of LRU-evicted ARP binding-table
    entries since analysis start. The ARP analyzer's binding table is capped at 65 536 entries;
    this counter exposes how many entries have been silently dropped when the cap is reached.
    ARP summary key count: 11 → 13 (also adds `storm_counters_evicted`).

  - **`storm_counters_evicted`** (ARP summary) — cumulative count of LRU-evicted ARP
    storm-counter entries since analysis start. The storm-counter LRU table is capped at 4 096
    entries; this counter exposes how many entries have been silently dropped.

  - **`dropped_transactions`** (Modbus summary) — cumulative count of Modbus pending-transaction
    map entries dropped when the per-flow cap is reached. Previously the cap silently discarded
    new pending entries; the counter makes the drop event visible in output. Modbus summary key
    count: 6 → 7.

  - **`dropped_map_entries`** (TLS summary and HTTP summary) — cumulative count of entries
    dropped from the TLS SNI/fingerprint maps and the HTTP host/path maps when per-map caps are
    reached. Previously cap-triggered drops were silent; the counter makes them visible. HTTP
    summary key count: 9 → 10; TLS summary gains one additional field.

### Tests / Internal

- **Negative regression tests for eviction/drop no-Finding invariants + HTTP existing-key
  AC-008 (PR #366).** Test module `bc_silent_resource_caps_tests` adds negative regression
  coverage asserting that ARP binding-table and storm-counter eviction, and Modbus
  dropped-transaction-map overflows, do not emit spurious Findings — the counters increment
  but the finding list remains clean. HTTP AC-008 coverage (existing-key update does not
  create a duplicate map entry) is also covered. Includes a cosmetic refactor of the ARP
  analyzer's `insert_binding_lru` path for clarity.

## [0.11.3] - 2026-07-06

### Security

- **Fixed unbounded per-flow memory growth in the DNP3 and EtherNet/IP (ENIP) analyzers
  (issue #342, CWE-401/CWE-770).** The stream dispatcher now purges DNP3/ENIP per-flow
  state on flow close (mirroring Modbus/HTTP/TLS), bounding analyzer memory to live flows;
  closed-flow state is folded into aggregates so findings/summary output are unchanged.
  (PR #362)

## [0.11.2] - 2026-07-05

### Added

- **`protocols` subcommand — coverage catalog table + JSON output (STORY-152, PR #353).**
  A new top-level subcommand `wirerust protocols` prints a formatted table of every protocol
  in the `KNOWN_PROTOCOLS` catalog alongside its classification (analyzed, gap, or
  unclassified) and associated CLI flag. Pass `--json [FILE]` to emit the catalog as
  structured JSON suitable for downstream tooling. `--csv` is explicitly rejected with a
  clear error. The subcommand is available without any input file.

- **`analyze --coverage-gaps` flag — tri-state CoverageGapsSummary report (STORY-154,
  PR #355).** When `--coverage-gaps` is passed to `wirerust analyze`, the analysis output
  includes a `CoverageGapsSummary` section that classifies each protocol in the capture into
  one of three states: `covered` (traffic observed and an analyzer was enabled), `gap`
  (traffic observed but no analyzer enabled), or `unclassified` (traffic observed for a
  protocol not in `KNOWN_PROTOCOLS`). The tri-state report is emitted in both terminal and
  JSON output and is designed for gap-driven coverage workflows.

- **`KNOWN_PROTOCOLS` catalog + partition functions — SS-18 (STORY-151, PR #351).**
  A new static catalog `KNOWN_PROTOCOLS` enumerates all protocols wirerust is aware of,
  together with their default port(s), CLI flag, and category. Partition functions
  (`is_covered`, `is_gap`, `is_unclassified`) operate over capture traffic against the
  catalog, forming the data backbone for the protocols subcommand and coverage-gaps report.
  VP-041 formally verified via Kani.

- **Dispatcher unclassified-protocol gap counters for TCP + UDP (STORY-153, PR #352).**
  The `StreamDispatcher` now accumulates per-port counters for TCP and UDP traffic that
  does not match any known protocol rule. These counters feed the `unclassified` bucket in
  the `CoverageGapsSummary`, giving operators visibility into novel or undocumented
  protocols in a capture.

### Fixed

- **`protocols --json=PATH` path argument honored; `--csv` rejected (PR #354, wave-68
  F-W68-01).** The `protocols` subcommand now correctly writes JSON output to the path
  supplied via `--json=PATH` (previously ignored, always writing to stdout). Passing `--csv`
  to `protocols` now returns a clear validation error — CSV output is not defined for the
  protocols catalog.

### Security

- **E-21 formal hardening — VP-041/042/043 proven (PR #357).** Kani proof harnesses
  Sub-A through Sub-C verify partition-function totality (VP-041), coverage-gap
  classification correctness (VP-042), and unclassified-counter monotonicity (VP-043).
  A cargo-fuzz target and mutation-testing pass cover the new dispatcher counter paths,
  achieving a 100 % effective kill rate on the E-21 detection delta.

## [0.11.1] - 2026-07-01

### Fixed

- **TLS handshake-message reassembly across TLS records (TLS-CLIENTHELLO-FRAG-001, HIGH).**
  The TLS analyzer previously parsed ClientHello and ServerHello records only when the full
  handshake message arrived in a single TLS record. A TLS peer that fragments a handshake
  message across record boundaries (valid per RFC 8446 §5.1 / RFC 5246 §6.2.1) caused wirerust to miss the
  SNI extension, JA3 fingerprint, and JA3S fingerprint entirely — a trivially exploitable
  evasion path. The analyzer now maintains a per-direction carry buffer that accumulates
  record payloads across records until a complete handshake message is available, then
  parses it. Carry bounds are enforced per-direction: per-message cap 65 536 bytes
  (the maximum TLS handshake message size), per-record cap 18 432 bytes (maximum TLS
  record payload). On overflow the carry buffer is cleared and recovery continues from
  the next record (clear-and-recover policy). Closes silent SNI/JA3/JA3S fingerprint-evasion
  via fragmented handshakes. [STORY-144 #341, STORY-145 #343, STORY-146 #344, BC-2.07.038–042]

### Added

- **TLS buffer-saturation telemetry.** A new `buffer_saturation_drops` counter is included
  in the TLS analyzer summary. It increments each time the per-direction carry buffer reaches
  its cap and is cleared (overflow-and-recover event). Exposes carry-overflow frequency for
  threat-hunting and capacity tuning without changing the existing wire format of other
  summary fields. [STORY-146, PR #344, BC-2.07.043]

### Security

- **TLS reassembly path formally hardened** — Kani VP-039 proof harnesses (3 non-vacuous),
  a cargo-fuzz target, and 12 mutation-gap tests added for the new reassembly path, closing
  the formal-verification obligation for the carry-buffer bounds and clear-and-recover
  semantics. [PR #345]

- **Bumped `anyhow` 1.0.102 → 1.0.103** to clear RUSTSEC-2026-0190 (advisory against
  1.0.102 only; no behavior change). [PR #346]

## [0.11.0] - 2026-06-29

### Added

- **EtherNet/IP (ENIP) + CIP protocol analyzer** — the headline feature of this release
  (Feature #316, STORY-130..139, PRs #317–#334, ADR-010). wirerust now analyzes TCP/44818
  flows using the ODVA EtherNet/IP + Common Industrial Protocol (CIP) stack. The analyzer
  is enabled with `--enip` (also covered by `--all`) and requires stream reassembly.

  **Protocol coverage:**

  - Parses the 24-byte ENIP encapsulation header (all fields, little-endian per ODVA
    specification): command, length, session_handle, status, sender_context, options.
    [STORY-130, PR #317, BC-2.17.001/002]
  - Classifies all 65,536 possible u16 command values into the 9 ODVA known commands
    (ListServices, ListIdentity, ListInterfaces, RegisterSession, UnRegisterSession,
    SendRRData, SendUnitData, IndicateStatus, Cancel) plus an `Unknown` catch-all.
    [STORY-130, PR #317, BC-2.17.004]
  - Parses Common Packet Format (CPF) item lists from `SendRRData` (0x006F) and
    `SendUnitData` (0x0070) payloads: bounded item-count walk, type_id recognition for
    Null Address (0x0000), Connected Address (0x00A1), Connected Data (0x00B1), and
    Unconnected Data (0x00B2) items. CIP service extraction and request-path segment
    parse apply to Unconnected Data Items (0x00B2) only in this release.
    [STORY-132, PR #319, BC-2.17.005/006/007/009]
  - Dispatched as Rule 7 in the `StreamDispatcher` — port-44818 fallback after the
    existing TLS, HTTP, Modbus (port 502), and DNP3 (port 20000) rules. Content-signature
    rules (TLS record, HTTP prefix) take priority. [STORY-131, PR #318, ADR-010 Decision 1]
  - Per-flow state (`EnipFlowState`) with a 600-byte per-direction carry buffer
    (`carry_c2s` / `carry_s2c`), frame-walk loop, and session summary folded at
    capture end. [STORY-136/137/138, PRs #326–#329, BC-2.17.016/017/021]

  **CLI flags:**

  - `--enip` — enable EtherNet/IP TCP analysis (default-off; included by `--all`)
  - `--enip-write-burst-threshold N` — T0836 write-burst threshold: fires when more than
    N CIP write-class service requests (SetAttributesAll, SetAttributeList,
    SetAttributeSingle) are observed in any 1-second window per flow (default: 50)
  - `--enip-error-burst-threshold M` — T0888 error-burst threshold: fires when more than
    M CIP error responses (non-zero `general_status`) are observed in any 10-second window
    per flow (default: 5; strict `>` semantics)

  **MITRE ATT&CK for ICS detections (ics-attack-19.1):**

  - **T0846 Remote System Discovery** — emitted per flow on the first ENIP ListIdentity
    (command 0x0063) frame; one-shot guard per flow. [STORY-134, PR #323, BC-2.17.010]
  - **T0888 Remote System Information Discovery** — two detection patterns:
    Pattern A: CIP GetAttribute{All,List,Single} request targeting Identity Object
    (Class 0x01) in the request path; Pattern B: CIP error-response burst exceeding
    `--enip-error-burst-threshold` within a 10-second window. [STORY-134/135, PRs #323/#324,
    BC-2.17.014]
  - **T0858 Change Operating Mode** — emitted per CIP Stop service (service code 0x07)
    request, indicating a controller run-to-stop transition command. [STORY-135, PR #324,
    BC-2.17.011]
  - **T0816 Device Restart/Shutdown** — emitted per CIP Reset service (service code 0x05)
    request. [STORY-135, PR #324, BC-2.17.013]
  - **T0836 Modify Parameter** — emitted when CIP write-class services (SetAttributesAll
    0x02, SetAttributeList 0x04, SetAttributeSingle 0x10) exceed
    `--enip-write-burst-threshold` within a 1-second window per flow. [STORY-135, PR #324,
    BC-2.17.012]
  - **T0814 Denial of Service** — malformed-frame anomaly; fires when 3 or more
    structurally invalid ENIP frames accumulate in a 300-second window per flow. Shared
    technique ID with the DNP3/Modbus analyzers. [STORY-137, PR #327, BC-2.17.018]

  New `MitreTactic::IcsExecution` enum variant added (TA0104) for T0858 "Change Operating
  Mode". MITRE catalog grew from 25 to 28 seeded technique IDs; emitted count grew from
  17 to 20 (T0858, T0816, T0846 added to the emitted set). T0846 promoted from
  seeded-only to emitted for the first time via ENIP ListIdentity detection.
  [STORY-133, PR #320, BC-2.10.008, VP-007]

  **Session summary (`enip_summary`):** `summarize()` produces a 7-key JSON object —
  `command_distribution`, `total_pdu_count`, `parse_errors`, `write_count`,
  `error_count`, `flows_analyzed`, `dropped_findings` — folding both closed and
  still-open flows at call time. [STORY-138, PR #329, BC-2.17.021]

  **Formal verification and quality assurance:**

  - VP-032 Kani proof harnesses Sub-A through Sub-D: `parse_enip_header` all-input
    safety, `classify_enip_command` totality, `is_valid_enip_frame` biconditional,
    `classify_cip_service` totality. [STORY-130/132, BC-2.17.001–004/007]
  - `fuzz_enip_cip_parse` cargo-fuzz harness covering `parse_cpf_items`,
    `parse_cip_header`, `parse_cip_request_path`, and `parse_enip_header` — F-P9-002
    obligation discharged. [PR #332]
  - Full-pipeline E2E tests against real ENIP/CIP pcaps: holdout scenarios HS-110
    through HS-122 verified (6 test cases, real-world captures). [PR #333]

### Changed

- **ENIP session summary wire format cleaned up.** The `enip_summary` JSON output uses
  the canonical key name `"parse_errors"` (not `"total_parse_errors"`) from day one,
  consistent with the lesson learned from the DNP3 rename in v0.10.0. The summary wire
  format was further cleaned up to ensure consistent field ordering and null-safety.
  [PR #331, BC-2.17.021 Invariant 1]

- **Green-doc-tense CI gate added.** A new CI job (`green-doc-tense-gate`) runs
  `bin/check-green-doc-tense` on all tracked source and test files, failing if any
  doc-comment or changelog entry uses aspirational tense markers ("will", "planned",
  "future") in contexts that assert current behavior. The gate includes a self-test
  (`bin/test_check_green_doc_tense.py`) that verifies 10 known-bad and 14 known-good
  patterns. [PR #321, b9b2e93]

### Fixed

- **ENIP source-IP attribution corrected.** The per-direction source-IP resolution
  in `on_data` was incorrect: it used a port-44818 heuristic that misidentified the
  client when the FlowKey's lower port was 44818. Replaced with direction-based
  attribution (`Direction::ClientToServer` maps to the TCP initiator; `ServerToClient`
  maps to the TCP responder), mirroring the Modbus pattern. Finding `source_ip` fields
  now correctly reflect the sending endpoint. [PR #328, AC-139-002]

- **ENIP `summarize()` includes still-open flows.** `summarize()` previously reported
  only counters accumulated from closed flows, silently undercounting `total_pdu_count`,
  `flows_analyzed`, `parse_errors`, and `command_distribution` whenever flows were still
  open at capture end. The summary now folds all still-open `EnipFlowState` entries into
  the aggregate at call time (RULING-W61-001). [PR #330, BC-2.17.021 Postcondition 1]

- **Modbus EC-X1: per-direction carry buffer split (`carry_c2s` / `carry_s2c`).**
  The Modbus analyzer previously used a single shared carry buffer for both directions, allowing
  a response packet's trailing bytes to be spliced into the next request's reassembly window
  (cross-direction carry-buffer contamination). The carry buffer is now split into two
  independent fields keyed by direction, eliminating the splice. [STORY-141, PR #336,
  BC-2.14.EC-X1]

- **Modbus EC-X2: `saturating_sub` for clock-backwards window reset.**
  A non-monotonic timestamp (e.g. packet re-ordering or NTP step) caused the time-delta
  computation in the Modbus window-reset path to underflow (wrapping subtraction on an unsigned
  value). The subtraction now uses `saturating_sub`, preventing the underflow and keeping the
  window-reset logic correct when clocks move backwards. [STORY-141, PR #336, BC-2.14.EC-X2]

- **DNP3 EC-X1: per-direction carry buffer split (`carry_c2s` / `carry_s2c`).**
  Same cross-direction carry-buffer splice fix applied to the DNP3 analyzer. [STORY-140,
  PR #335, BC-2.15.EC-X1]

- **DNP3 EC-X2: `saturating_sub` for clock-backwards window reset.**
  Same saturating subtraction fix applied to the DNP3 window-reset path. [STORY-140, PR #335,
  BC-2.15.EC-X2]

- **DNP3 desync-latch: complete-predicate gated on `frame_count == 0`.**
  The DNP3 desync-latch complete-predicate fired unconditionally, which could produce a spurious
  desync event on the very first frame of a session before any real desync had occurred. The
  predicate is now gated on `frame_count == 0` so it only triggers after at least one valid
  frame has been observed. [STORY-142, PR #336, BC-2.15.DESYNC]

- **ENIP EC-X1: per-direction carry buffer split (`carry_c2s` / `carry_s2c`).**
  Same cross-direction carry-buffer splice fix applied to the EtherNet/IP analyzer. [STORY-139,
  PR #334, BC-2.17.EC-X1]

- **ENIP EC-X2: `saturating_sub` for clock-backwards window reset.**
  Same saturating subtraction fix applied to the ENIP window-reset path. [STORY-139, PR #334,
  BC-2.17.EC-X2]

## [0.10.0] - 2026-06-24

### Breaking Changes

- **DNP3 analyzer output: renamed summary key `total_parse_errors` → `parse_errors`.**
  The `detail` map produced by the DNP3 analyzer now uses the key `"parse_errors"` instead of
  `"total_parse_errors"`, aligning DNP3 with sibling analyzers (HTTP, TLS, Modbus) that already
  use `"parse_errors"`. JSON consumers reading DNP3 summary output must migrate the key name.
  [PC-014, BC-2.15.020 v1.4, STORY-108 AC-010]

  **Migration:** Replace any lookup of `detail["total_parse_errors"]` with
  `detail["parse_errors"]` in your consumer. For `jq` users:
  `jq '.[] | .detail.total_parse_errors'` → `jq '.[] | .detail.parse_errors'`.

## [0.9.4] - 2026-06-23

### Added

- **Per-finding `mitre_attack` JSON array for SIEM consumers (issue #64).** Each finding in JSON
  output now carries a `mitre_attack` array. Every element is an object with the fields `id`,
  `name`, `tactic_id`, `tactic_name`, and `reference`, resolved from the static MITRE catalog at
  report time. Downstream SIEM ingestion pipelines can consume structured technique metadata
  directly without maintaining a separate ID-to-name lookup.

### Fixed

- **ICS-matrix tactic IDs corrected for ICS techniques.** ICS techniques previously emitted
  Enterprise-matrix tactic IDs; they now emit the correct ICS-matrix tactic IDs. Three new ICS
  tactic variants were added: `IcsDiscovery` (TA0102), `IcsCollection` (TA0100), and
  `IcsCommandAndControl` (TA0101). Two technique-to-tactic mappings were corrected:
  - **T0830 Adversary-in-the-Middle** reclassified from its previous tactic to **Collection
    (TA0100)**.
  - **T0831 Manipulation of Control** reclassified from its previous tactic to **Impact
    (TA0105)**.

### Docs

- Corrected the ARP tactic column in README to reflect the updated ICS-matrix tactic assignments.
- Superseded the stale MITRE mapping design doc; current behavior is authoritative.

## [0.9.3] - 2026-06-22

### Added

- **pcapng capture-format reader.** wirerust now reads pcapng files in addition to classic
  pcap. Format is detected by a magic-byte probe on the first four bytes of the file
  (pcapng SHB magic `0x0A0D0D0A`), so pcapng files are accepted regardless of file
  extension — including when passing a directory, where the file list is now built by
  magic-byte detection rather than by extension filter alone (`.pcapng` files were
  previously excluded from directory expansion).

  The reader parses four block types:

  - **SHB** (Section Header Block) — both big- and little-endian byte orders.
  - **IDB** (Interface Description Block) — up to 65,535 interfaces per file; all
    interfaces in a single file must share the same link type. The `if_tsresol` IDB
    option (code 9) is parsed to determine timestamp resolution; nanosecond captures
    (e.g. `if_tsresol = 0x09`) are converted correctly to microseconds for analysis.
  - **EPB** (Enhanced Packet Block) — packet data, interface ID lookup, and per-packet
    timestamp reconstruction using the interface's `if_tsresol`.
  - **SPB** (Simple Packet Block) — parsed and yielded as packets with no timestamp
    (SPB carries no timestamp field).

  The following block types are silently skipped: NRB (Name Resolution Block), ISB
  (Interface Statistics Block), DSB (Decryption Secrets Block), OPB (Obsolete Packet
  Block), and any unrecognized block type. Multi-section files (a second SHB) are
  rejected — use `mergecap` or `editcap` to re-save as a single-section file.

  The same five link types supported for classic pcap (Ethernet 1, Raw IP 101, Linux
  Cooked/SLL 113, IPv4 228, IPv6 229) are supported for pcapng.

  A 4 GiB per-file size cap (E-INP-014) is enforced via `fstat` on the already-open
  file descriptor before the full file is loaded into memory.

- **`PcapSource::is_pcapng` discriminant field.** The `PcapSource` struct now carries
  a public `is_pcapng: bool` field that is `true` when the file was identified as pcapng
  by magic-byte detection. Used internally for the zero-packet notice wording
  ("pcapng file" vs. "pcap file").

- **Per-file error isolation for batch analysis.** When analyzing a directory, a parse
  error or read failure on one file is reported to stderr and skipped; remaining files
  in the batch continue to be processed. Files that parse successfully but contain zero
  packets emit a notice to stderr: "notice: \<path\>: 0 packets read from \<pcap|pcapng\>
  file", with the OPB-clause appended when the file contained Obsolete Packet Blocks
  that were skipped.

- **New input-validation error codes** (pcapng-specific guards):

  | Code | Condition |
  |------|-----------|
  | E-INP-010 | pcapng block framing rejection — crate-level framing error (btl misaligned, EOF mid-block, zero-advance forward-progress stall) or EPB interface ID out of range on a non-empty interface table. |
  | E-INP-011 | Multi-IDB link-type conflict — a subsequent Interface Description Block declares a link type that differs from the first interface's link type. |
  | E-INP-012 | Second Section Header Block — multi-section pcapng files are not supported. |
  | E-INP-013 | IDB after first packet block — an Interface Description Block appears after the first EPB or SPB has already been emitted, an ordering not supported by wirerust. |
  | E-INP-014 | File too large — pcapng file exceeds the 4 GiB in-memory limit; message instructs the user to split the capture or use a streaming tool. |
  | E-INP-015 | Interface table cap exceeded — pcapng file declares more than 65,535 Interface Description Blocks. |

  (Codes E-INP-008 and E-INP-009 — SHB/IDB/EPB body-too-short and empty interface
  table, respectively — were also introduced in this delta as part of the pcapng reader
  but do not appear in the above table as they describe internal structural failures
  rather than user-actionable input constraints.)

### Fixed

- **TCP reassembly CWE-407 null-eviction storm (PR #298).** When the flow table reached
  `max_flows` and a new flow arrived, the eviction loop's break condition (`<= max_flows`)
  fired immediately on the first iteration, causing an O(F log F) sort with zero flows
  actually evicted. On captures with frozen or duplicate timestamps — where the
  time-based idle expiry never fires — every new flow beyond the cap triggered a full
  sort with no eviction, producing quadratic behavior. On a 120,000-flow
  frozen-timestamp capture the wall time was ~75 s before this fix.

  Three mitigations were applied:

  - **R1 (CWE-401 zombie segments):** Segments whose end offset lies strictly below the
    reassembly flush cursor are now rejected instead of being inserted into the gap map,
    preventing unbounded zombie segment accumulation.
  - **R2 (null-eviction storm fix):** The break condition changed from `<= max_flows` to
    `< max_flows`, ensuring at least one flow is evicted on each eviction call.
  - **R3 (batch eviction to headroom):** `max_flows`-triggered eviction now evicts down
    to 90% of `max_flows` in one call (headroom target = `max(1, max_flows * 9 / 10)`),
    amortizing the O(F log F) sort across the next ~10% of new-flow admissions. The same
    120,000-flow frozen-timestamp scenario completes in ~0.76 s after these fixes.

- **R4 packet-index cadence expiry (defense-in-depth for frozen timestamps).** A
  packet-index sweep runs every N packets (`expiry_sweep_interval`, configurable) and
  expires flows idle for more than `idle_packet_threshold` packets, independent of
  capture timestamps. This ensures idle flows are reclaimed even on captures where all
  packet timestamps are identical or otherwise frozen.

- **`read_magic` short-read race eliminated.** The magic-byte probe used by directory
  expansion previously called `read()` and accepted a short read as a valid result, meaning
  a file with exactly 4 bytes might not return all four bytes on a single `read()` call.
  Changed to `read_exact()`, which either fills the buffer or returns an error, so files
  shorter than 4 bytes correctly return `None` and files of exactly 4 bytes are read
  reliably.

- **pcapng block-walk forward-progress guard (CWE-835).** The block-walk loop now
  checks that the parser advances after each block; a zero-advance result is treated as a
  framing anomaly (E-INP-010) rather than looping indefinitely.

- **pcapng file-size gate uses `fstat` on the open fd (CWE-367 advisory).** The size
  check now calls `metadata()` on the already-open file descriptor rather than a second
  path-based `stat()` call, closing the TOCTOU window between magic-byte detection and
  size enforcement.

- **pcapng IDB options TLV parsed with section endianness.** The `parse_idb_options`
  function previously read option TLV fields as fixed little-endian. It now uses the
  section endianness (big or little) detected from the SHB byte-order magic, so
  `if_tsresol` and other IDB options are decoded correctly from big-endian pcapng files.

### Security

- CWE-407 + CWE-401 mitigated in the TCP reassembly engine (see Fixed — PR #298).
- CWE-835 forward-progress guard added to the pcapng block-walk loop.
- CWE-367 TOCTOU window for pcapng file-size gate closed by switching to `fstat` on
  the open file descriptor.
- Block sequence counter in the pcapng block-walk uses `saturating_add` to prevent
  wraparound (SEC-005).

## [0.9.2] - 2026-06-19

### Fixed

- **DNP3 `control_operation_counts` was non-deterministic across process runs.**
  `Dnp3Analyzer::summarize()` previously called `self.flows.values().enumerate()`
  over a `HashMap<FlowKey, Dnp3FlowState>`. Because `HashMap` uses a per-process
  random seed (HashBrown), the iteration order changed each run, causing the
  flow index assigned by `enumerate()` to map to a different flow on every
  invocation. The `BTreeMap` key-sort masked the issue at the key level (keys
  `"0".."N-1"` were always sorted), but the VALUE at each key was
  non-deterministic. Running `wirerust analyze <dnp3-capture> --all` twice on the
  same file produced different `control_operation_counts` output (confirmed on a
  real 26K-packet DNP3 capture in post-release e2e testing).

  Fix: derive `Ord` + `PartialOrd` on `FlowKey` (lexicographic order on
  `(lower_ip, lower_port, upper_ip, upper_port)`; `IpAddr` and `u16` both
  implement `Ord`). In `summarize()`, sort `flows.iter()` by `FlowKey` before
  `enumerate()`, so index→value assignment is stable across all process runs.
  JSON schema is unchanged — keys remain `"0".."N-1"` strings in a BTreeMap.
  Traces to BC-2.15.020 postcondition 1.

## [0.9.1] - 2026-06-19

### Fixed

- **`--no-collapse` help text and README referenced non-existent flags
  `--output json` / `--output csv`.** There is no `--output` flag in wirerust;
  the real flags are `--json <FILE>`, `--csv <FILE>`, and
  `--output-format <fmt>`. The doc-comment in `src/cli.rs` and the corresponding
  line in `README.md` both said "Has no effect on --output json or --output csv."
  Corrected to "Has no effect on --json, --csv, or --output-format json|csv
  output." Behavior is unchanged — JSON and CSV output were already
  collapse-invariant; only the help text wording was wrong.

## [0.9.0] - 2026-06-19

### Changed (BREAKING)

- **`TerminalReporter` findings-render mode: two bools → `FindingsRender` enum → `FindingsRender`
  struct of two orthogonal enums (STORY-120 PR #266, STORY-122/A PR #268).**
  This entry supersedes the three-variant enum description that shipped in an earlier 0.9.0
  pre-release entry.

  *Phase 1 (STORY-120, PR #266):* The `show_mitre_grouping: bool` and `collapse_findings: bool`
  public fields on `TerminalReporter` were removed and replaced by a single `render: FindingsRender`
  field typed as a three-variant enum (`Grouped`, `FlatCollapsed`, `FlatExpanded`).

  *Phase 2 (STORY-122/A, PR #268):* `FindingsRender` was reshaped from a three-variant enum into
  a **struct of two orthogonal enums**: `{ grouping: Grouping, collapse: Collapse }`. The
  `Grouping` enum has variants `Grouped` and `Flat`; the `Collapse` enum has variants `Collapsed`
  and `Expanded`. All four combinations are valid. The three named enum variants (`Grouped`,
  `FlatCollapsed`, `FlatExpanded`) no longer exist. Per RFC 1105 this is an additional breaking
  change: any code that matched or constructed the three-variant enum must migrate to the
  two-field struct. The 0.8.x → 0.9.0 minor bump covers both phases.

  *Forward-compatibility (F7-R2):* `Grouping`, `Collapse`, and `FindingsRender` (in
  `wirerust::reporter::terminal`) are now marked `#[non_exhaustive]`, allowing future
  variants or fields to be added without a semver-breaking change. Because `FindingsRender`
  is `#[non_exhaustive]`, external crates must construct it via the new
  `FindingsRender::new(grouping, collapse)` constructor rather than a struct literal
  (struct-literal construction of a `#[non_exhaustive]` struct is rejected by the compiler
  outside the defining crate).

### Changed

- **`--mitre` now collapses identical findings within each MITRE tactic bucket by default
  (STORY-119/B, PR #269).** When `--mitre` is passed, `wirerust analyze` routes output through
  the new `render_findings_grouped_collapsed` path, which groups identical findings (same category,
  verdict, confidence, summary) within each tactic bucket into a single line with a `(xN)` count
  suffix and up to K=3 representative evidence samples. Singletons render without a count suffix.
  Terminal output for `--mitre` is **no longer byte-identical** to the pre-0.9.0 grouped output.
  JSON and CSV output are unaffected.

- **`--no-collapse` is now dual-scope (STORY-119/B, PR #269).** Previously `--no-collapse`
  suppressed collapse only in flat (non-`--mitre`) mode. It now suppresses collapse in both flat
  and grouped (`--mitre`) modes. Passing `--no-collapse` restores one-line-per-finding output
  regardless of whether `--mitre` is also passed.

## [0.8.0] - 2026-06-17

### Added

- `--no-collapse` flag for `wirerust analyze` to opt out of terminal finding-collapse (closes
  #259, STORY-118). Pass `--no-collapse` to restore the pre-v0.8.0 one-line-per-finding output.

### Changed

- **Terminal `analyze` output now collapses repeated findings by default.** Findings that share
  the same (category, verdict, confidence, summary) are collapsed into a single line with a
  `(xN)` count suffix and up to 3 representative evidence samples (K=3). This is a
  **display-layer-only behavioral change**: JSON and CSV output are unaffected, and
  `--mitre`-grouped mode was unchanged in 0.8.0; grouped-mode collapse shipped in 0.9.0.
  Pass `--no-collapse` to disable. Governed by ADR-0003 Display-Layer Aggregation.

## [0.7.1] - 2026-06-17

### Added

- Regression test coverage for VLAN / QinQ (802.1ad double-tag) / MACsec link-extension ARP
  offset handling — 10 tests across `tests/bc_2_16_qinq_macsec_offset_tests.rs` and
  `tests/bc_2_16_e17_macsec_offset_tests.rs` (issue #253, STORY-116/117). Includes an
  off-by-8 SCI-accounting guard for MACsec-tagged ARP.

### Notes

- No runtime behavior change: the VLAN/QinQ/MACsec offset handling itself shipped in 0.7.0;
  this release adds regression guards. MACsec-over-ARP offset correctness is proven by
  etherparse source + upstream proptests + synthetic tests and is documented as an
  evidence-backed limitation (no public on-wire MACsec+ARP capture exists).

## [0.7.0] - 2026-06-16

### Added

- **ARP Security Analyzer** (issue #9, epic E-16) for link-layer and OT network forensics.
  Detects five threat classes with MITRE ATT&CK attribution:

  - **D1 ARP spoofing** — binding-conflict detection with MEDIUM→HIGH severity escalation
    (configurable `--arp-spoof-threshold`, default 3 conflicts). Attributed to **T0830
    Adversary-in-the-Middle** and **T1557.002 ARP Cache Poisoning**.
  - **D2 Gratuitous ARP (GARP)** — unsolicited GARP frames flagged as Possible; binding-conflict
    GARP (GARP where the announced MAC differs from the established binding) escalated to Likely.
  - **D3 ARP storms** — high-rate ARP flood detection (configurable `--arp-storm-rate`, default
    50 frames/window). ~~Attributed to **T0830**.~~ (Corrected: D3 findings emit
    `mitre_techniques: []` — T0814 attribution withheld per DF-VALIDATION-001 / BC-2.16.008
    Invariant 3. See v0.7.0 shipping state vs. current behavior.)
  - **D11 Malformed ARP frames** — strict + lax/snaplen-truncated ARP parsing; frames that fail
    both passes are flagged as malformed-protocol anomalies.
  - **D12 L2/L3 MAC mismatch** — Ethernet source MAC vs. ARP sender hardware address mismatch
    detection, flagging potential header spoofing.

  New CLI flags: `--arp` (enable; also included in `-a`/`--all`), `--arp-spoof-threshold N`,
  `--arp-storm-rate N`. Binding-table LRU cap: 65 536 entries; storm-counter LRU cap: 4 096
  entries.

  Implemented across STORY-111..115 (PRs #236, #238, #239, #240, #241) with formal hardening
  in PRs #242–#251.

### Changed

- Migrated the packet decoder from **etherparse 0.16 to 0.20** (`DecodedFrame{Ip,Arp}` model).
  Strict and lax/snaplen-truncated ARP parsing added; VLAN/QinQ/MACsec link-extension offset
  handling included.
- Bumped **chrono 0.4.44 → 0.4.45** (#237).

### Verified

- **VP-024 ARP parse-safety and binding-cap** formally verified: 5 Kani proof harnesses proven
  correct, cargo-fuzz 16.2 M executions / 0 crashes, cargo-mutants 98.9 % kill rate on the
  ARP delta.

## [0.6.0] - 2026-06-12

### Added

- **DNP3 TCP protocol analyzer** for ICS/OT network forensics (Feature #8, PRs #219–#231).
  Analyzes TCP streams on port 20000 per IEEE Std 1815-2012 (DNP3); dispatched as Rule 6 in the
  stream dispatcher after content-signature rules (TLS record, HTTP prefix) and port rules for
  TLS, HTTP, and Modbus — it never misclassifies TLS or HTTP traffic
  (BC-2.15.021 INV-2, ADR-007 Decision 1).

  Parses the 10-byte DNP3 data-link layer header: sync bytes, LENGTH, CONTROL, DEST/SRC link
  addresses (little-endian per IEEE 1815-2012 §8.2). Classifies application-layer function codes
  into six classes: Read, Write, Control, Restart, Management, Response. Per-flow state with a
  292-byte carry-buffer frame-walk handles fragmented TCP delivery and desync detection.

  Emits findings mapped to **5 MITRE ATT&CK for ICS techniques**:

  - **T1692.001** Unauthorized Message: Command Message — direct-operate burst (Control-class FCs
    exceed the per-flow threshold within a 60-second detection window), unexpected master source
    (Control FC from a source address not in the established master set), and broadcast control
    command (Control FC to a DNP3 broadcast destination address)
  - **T1691.001** Block Operational Technology Message: Command Message — Control-class requests
    that receive no matching RESPONSE (FC 0x81) within 10 seconds contribute to a block-event
    counter; fires when >= 3 block events accumulate within the 300-second correlation window
  - **T0827** Loss of Control — fires when the combined count of restart events and block-command
    events reaches >= 3 within the 300-second correlation window (co-emitted after T0814 or
    T1691.001)
  - **T0814** Denial of Service — emitted per cold/warm restart command (FC 0x0D / FC 0x0E), and
    as a malformed-frame anomaly when >= 3 parse-invalid frames are observed within the 300-second
    correlation window
  - **T0836** Modify Parameter — emitted per WRITE command (FC 0x02)

  Additional T0814 trigger sources (Inhibit Response Function):
  - DISABLE_UNSOLICITED (FC 0x15): verdict Likely / confidence Medium — alarm suppression /
    event-blinding primitive; emitted per occurrence.
  - ENABLE_UNSOLICITED (FC 0x14): verdict Possible / confidence Low — unsolicited reporting
    control; emitted per occurrence; also sets the per-flow context flag that suppresses the
    unsolicited-response anomaly.
  - Unsolicited-response anomaly: UNSOLICITED_RESPONSE (FC 0x82) arrives on a flow where
    ENABLE_UNSOLICITED was never observed and no solicited exchange has been seen; verdict
    Possible / confidence Low; one-shot per flow (T0814).

  Bounded-resource design: per-flow state capped at 64 tracked master addresses, 256 pending
  requests, and 10,000 total findings; 300-second correlation window with six windowed counters
  reset together (ADR-007 Decision 4).

- **CLI flags for the DNP3 analyzer:**
  - `--dnp3` — enable DNP3 TCP analysis (also included in `-a`/`--all`; default-off,
    BC-2.15.021)
  - `--dnp3-direct-operate-threshold N` — per-flow direct-operate burst threshold; fires T1692.001
    when Control-class FC count exceeds N within the 60-second detection window (default: 10,
    BC-2.15.017)

- **Dispatcher Rule 6** — Port-20000 classification added to the stream dispatcher as Rule 6
  (STORY-110, ADR-007 Decision 1). Fires after content-signature rules (Rules 1–2) and port rules
  for TLS/HTTP/Modbus (Rules 3–5), preserving the VP-004 port-precedence invariant.

- **`MitreTactic::IcsImpact` tactic variant** — new variant added to the `MitreTactic` enum
  (STORY-109, VP-007 obligation). Maps to the MITRE ATT&CK for ICS "Impact" tactic (TA0105).
  Used exclusively by T0827 "Loss of Control". Added atomically with the T0827 emission branch
  and the `technique_info("T0827")` catalog entry.

- **`T1691.001` and `T0827` catalog entries** — two new technique IDs seeded in the static MITRE
  catalog (`technique_info`): T1691.001 "Block Operational Technology Message: Command Message"
  (IcsInhibitResponseFunction) and T0827 "Loss of Control" (IcsImpact). Total catalog size: 23
  technique IDs (STORY-109, VP-007).

- **Formal verification and quality assurance for the DNP3 analyzer:**
  - VP-023 (Kani): parse safety sub-properties A–D: all-input range, FC totality, frame-length
    bounds, carry-buffer progress.
  - Fuzz testing: `fuzz_dnp3_parse` target added (PR #229).
  - Mutation testing: 100% effective kill rate on the detection core including edge cases for
    window-seeding (PR #231).

- **T0814 full detection surface documented** (DRIFT-DNP3-DOC-T0814-COMPLETENESS-001). The DNP3
  T0814 "Denial of Service / Inhibit Response Function" technique is emitted from five trigger
  sources: cold/warm restart command (FC 0x0D/0x0E; verdict Likely/High), DISABLE_UNSOLICITED
  (FC 0x15; verdict Likely/Medium), ENABLE_UNSOLICITED (FC 0x14; verdict Possible/Low),
  unsolicited-response anomaly (FC 0x82 on a flow with no prior ENABLE_UNSOLICITED; verdict
  Possible/Low), and malformed-frame anomaly (>= 3 parse-invalid frames in the 300s window;
  verdict Possible/Low). README and CHANGELOG now enumerate all five sources.

## [0.5.0] - 2026-06-10

### Fixed

- **Behavioral change — emitted output:** Remapped revoked MITRE ATT&CK-ICS techniques to their
  replacement IDs in the pinned ics-attack-19.1 catalog (issue #222):
  - `T0855` "Unauthorized Command Message" → **`T1692.001`** "Unauthorized Message: Command Message"
    (ICS sub-technique under parent T1692 "Unauthorized Message"). **Behavioral change:** Modbus
    findings now emit `T1692.001` instead of `T0855` in the `mitre_techniques` field of all JSON,
    terminal, and CSV output. Tactic (IcsImpairProcessControl) and co-emission ordering are unchanged.
  - `T0856` "Spoof Reporting Message" → **`T1692.002`** "Unauthorized Message: Reporting Message"
    (ICS sub-technique under T1692). Catalog-only (seeded, never emitted); no emitted output affected.

## [0.4.0] - 2026-06-10

### Added

- **Modbus TCP protocol analyzer** for ICS/OT network forensics (Feature #7, issue #7, PRs #211–#218).
  Detects Modbus traffic on port 502; parses the MBAP header (transaction ID, protocol ID, length,
  unit ID) and function code; per-flow transaction correlation with bounded pending-table (request /
  response matching). Emits findings mapped to **7 MITRE ATT&CK for ICS techniques**:
  - T0855 Unauthorized Command Message (write-class function codes) (→ remapped to T1692.001 in v0.5.0)
  - T0836 Modify Parameter (write-register / write-coil)
  - T0835 Manipulate I/O Image (force-listen-only, write-multiple coils)
  - T0831 Manipulation of Control (mask write register, write file record)
  - T0806 Brute Force I/O (sustained coil/register write flooding)
  - T0814 Denial of Service (exception-burst flooding pattern)
  - T0888 Remote System Information Discovery (FC-scanning / register-map enumeration via exception
    burst on recon function codes 0x01/0x02)

  Multi-tag co-emission: one finding per write PDU carrying the union of applicable techniques.
  Dual-window write-rate detection: burst threshold (>20 writes/1 s, configurable) + sustained
  threshold (>10 writes/s over ≥2 s, configurable). Exception-burst anomaly detection triggers
  T0888 on recon-code exception runs. Per-analyzer summary reports function-code distribution,
  write count, exception count, and PDU count.

- **CLI flags for the Modbus analyzer:**
  - `--modbus` — enable Modbus TCP analysis (also included in `-a`/`--all`)
  - `--modbus-write-burst-threshold N` — burst detection threshold (default 20 writes/1 s)
  - `--modbus-write-sustained-threshold N` — sustained-rate threshold (default 10 writes/s over ≥2 s)

- **Dispatcher port-502 classification** — Rule 5 in the stream dispatcher classifies port-502
  flows for Modbus after content-signature rules and the 443/8443/80/8080 port rules; it never
  steals HTTP or TLS traffic (VP-004 port-precedence invariant preserved, PR #214).

- **Formal verification and quality assurance for the Modbus analyzer:**
  - VP-022 (Kani): MBAP parse safety, function-code classification totality, exception-code
    biconditional invariant.
  - Fuzz testing: 3.7 M executions, 0 crashes (PR #216).
  - Mutation testing: 100 % effective kill rate on the detection core (PR #216).
  - E2E integration: pcap fixture + end-to-end flow tests (PR #217).
  - T0888 blemish fix: exception-burst correctly emits T0888 for recon function codes 0x01/0x02
    (PR #218, BC-2.14.019).

- **Architecture records:**
  - ADR-005 — Binary ICS protocol integration strategy.
  - ADR-006 — Multi-technique Finding attribution model.

## [0.3.0] - 2026-06-09

### Changed (BREAKING)

- **Finding MITRE attribution: scalar → array (ECS-aligned).** `Finding.mitre_technique: Option<String>` has been renamed to `mitre_techniques: Vec<String>`. In JSON output the field is now `"mitre_techniques"` (an array); it is omitted entirely when empty. Downstream JSON consumers must update to read an array instead of a scalar. In CSV output the column is renamed `mitre_techniques`; multiple values are semicolon-joined (e.g. `T0855;T0836`); a single value is written without a separator; an empty value is an empty string. The terminal reporter now renders `MITRE: T0855, T0836` for multi-technique findings and groups by the first technique's tactic. This aligns the schema with Elastic ECS `threat.technique.id` (PR #209, STORY-100/101).

- **JSON report envelope: new fields.** Every JSON report now includes two top-level envelope fields: `"mitre_domain": "ics-attack"` and `"mitre_attack_version": "ics-attack-19.1"`. The domain is constant (wirerust targets the ATT&CK for ICS matrix). The version is pinned to ATT&CK for ICS v19.1 (released 2026-04-28), which covers all 21 seeded technique IDs including the 6 staged ICS entries (STORY-101, PR #209).

### Migration

Downstream consumers of wirerust JSON or CSV output must update for this release:

- **JSON**: The finding attribute changed from `"mitre_technique": "T1027"` (string, may be absent) to `"mitre_techniques": ["T1027"]` (array, omitted when empty). Update any field reads to `obj["mitre_techniques"][0]` for single-technique findings or iterate the array for multi-technique ones.
- **CSV**: Column 6 changed from `mitre_technique` to `mitre_techniques`. Multi-value cells are semicolon-joined; split on `";"` to get individual technique IDs.
- **JSON envelope**: Two new top-level keys (`mitre_domain`, `mitre_attack_version`) are now always present. If your parser requires a strict fixed key set, add these two keys to your allowlist.

### Added

- **MITRE ICS catalog expanded.** The technique catalog grew from 15 to 21 seeded entries. Six new ICS technique IDs are staged for the upcoming Modbus analyzer (STORY-104): T0836 (Modify Parameter), T0814 (Deny Control), T0806 (Brute Force I/O), T0835 (Manipulate I/O Image), T0831 (Manipulation of Control), T0888 (Remote System Information Discovery). T0855 (Unauthorized Command Message) is now emitted by the TLS analyzer. Total emitted count: 13 (6 Enterprise + 7 ICS), up from 6 emitted in v0.2.0 (PR #209, STORY-100/101).

## [0.2.0] - 2026-06-09

### Added

- **Finding timestamp provenance** — every `Finding` now carries a
  `capture_ts` field populated with the pcap capture-relative timestamp of
  the packet that triggered the finding. The timestamp is threaded from the
  pcap reader through `StreamHandler::on_data` all the way to each Finding
  emission site in the TLS and HTTP analyzers. It is surfaced as an RFC 3339
  string in JSON output and as a new `timestamp` column in CSV output
  (#100; PRs #197, #198, #199; BC-2.04.055, BC-2.09.007, VP-021).
  Segment-limit summary findings intentionally carry no timestamp (correct
  by design).

### Fixed

- SNI control-byte summary now correctly surfaces control bytes in the
  human-readable finding for mixed control + non-ASCII values (#104, PR #194).
- Weak-cipher evidence vector is capped at 64 entries with an elision marker
  to prevent unbounded growth on adversarial captures (#102, PR #195).

### CI / Build / Supply-chain

- Migrated release workflow actions from Node 20 to Node 24 with fresh
  SHA-pinned refs (`upload-artifact` v7.0.1, `download-artifact` v8.0.1,
  `softprops/action-gh-release` v3.0.0); added Dependabot tracking for
  workflow actions (PR #192).
- SHA-pinned all remaining CI actions (`actions/checkout`, `rust-cache`,
  `cargo-deny`, `amannn/action-semantic-pull-request`) and added the
  **action-pin-gate** enforcement job that fails CI if any action ref is
  not a 40-char hex SHA (PR #196).
- Test and spec hardening for timestamp provenance: exact-value assertions
  replacing approximate checks, stale doc-comment corrections (PRs #200, #201).

## [0.1.0] - 2026-06-08

### Added

**Core pipeline**

- PCAP reader supporting five link types: Ethernet (1), Raw IP (101), Linux
  Cooked / SLL (113), IPv4 (228), and IPv6 (229). Snaplen-truncated captures
  (e.g. `tcpdump -s 96`) are accepted via the unvalidated raw-record path.
  pcapng is not supported.
- Zero-copy L2–L4 packet decoding via `etherparse`. The full capture is loaded
  into memory as a `Vec<RawPacket>` before analysis; available RAM determines
  the practical file-size limit.
- Single-pass analysis pipeline: Reader → Decoder → Analyzers → Reporter,
  producing host/service/protocol summaries and threat findings in one pass.
- Directory expansion: pass a directory path and wirerust processes every
  `.pcap` file found within it (`.pcapng` files are excluded).

**TCP stream reassembly engine**

- Forensic-grade TCP stream reassembly with a first-wins overlap policy
  (earlier-arriving data wins on byte conflicts).
- Configurable per-direction depth limit (`--reassembly-depth`, default 10 MB)
  and global memory cap (`--reassembly-memcap`, default 1024 MB).
- Evasion and anomaly detection: overlapping-segment counting
  (`--overlap-threshold`, default 50 per flow direction), consecutive
  small-segment detection (`--small-segment-threshold`, default 100 run
  length; `--small-segment-max-bytes`, default 16 B), and out-of-window
  segment counting (`--out-of-window-threshold`, default 100).
- Interactive-protocol port exemption from small-segment detection (default:
  ports 23 and 513; overridable via `--small-segment-ignore-ports`).
- Idle-flow expiry: flows silent longer than `--flow-timeout` seconds
  (default 300) are evicted from the flow table.
- Reassembly statistics surfaced in all output formats: bytes reassembled,
  segment-limit drops, overlap count, out-of-window count, and small-segment
  count.

**Protocol analyzers**

- DNS analyzer: traffic statistics including query/response counts,
  top queried hostnames, and query-type distribution.
- HTTP/1.x analyzer (requires TCP reassembly): stream-level request and
  response parsing with detection for path traversal sequences, web-shell
  indicators, unusual HTTP methods, missing or empty Host headers, and other
  header anomalies. Parse-error isolation prevents one poisoned stream from
  affecting other flows.
- TLS analyzer: ClientHello and ServerHello parsing; SNI extraction and
  classification (clean ASCII, ASCII control bytes C0/DEL, valid non-ASCII
  UTF-8, non-UTF-8 bytes); JA3 and JA3S fingerprinting with GREASE
  value filtering; weak cipher detection; deprecated SSL 2.0 and 3.0
  detection.
- Stream dispatcher: content-first protocol classification (TLS record
  signature, HTTP prefix, then port-based fallback) with classification
  caching and a configurable retry budget (`max_classification_attempts`).

**Threat detection and MITRE ATT&CK**

- Finding system with verdict, confidence score, source IP, direction tag,
  and optional MITRE ATT&CK technique ID.
- Static MITRE ATT&CK catalog mapping technique IDs (T-format) to tactic and
  technique name, consumed by the terminal reporter when `--mitre` is passed.
- `--mitre` flag groups terminal output by ATT&CK tactic with technique names
  displayed alongside each finding.

**Output formats and CLI**

- Colored terminal reporter with MITRE tactic grouping, top-SNI and top-host
  tables, reassembly statistics section, and skipped-packet accounting.
  Deterministic tie-ordering for top-SNI and top-host tables.
- JSON reporter: structured output with deterministic field ordering,
  `skipped_packets` counter, and `dropped_findings` counter. `#[non_exhaustive]`
  on public enums for forward compatibility.
- CSV reporter: 9-column findings table (tactic, verdict, confidence,
  source IP, destination IP, port, protocol, description, MITRE technique).
  CSV-injection neutralization applied to all string fields. Evidence strings
  joined with a pipe separator.
- Output routing: `--output-format json|csv` writes to stdout; `--json [FILE]`
  and `--csv [FILE]` write to a file (or stdout if no path is given).
  `--json` and `--csv` are mutually exclusive.
- `analyze` subcommand with `--dns`, `--http`, `--tls`, `--mitre`, and
  `-a/--all` flags. HTTP analysis automatically enables TCP reassembly.
- `summary` subcommand with optional `--hosts` flag for a per-host IP
  breakdown. Outputs total packets, bytes, protocol distribution, and
  service-hint counts.
- `--no-color` flag disables ANSI color globally.
- Zero, non-integer, or out-of-range values for `--reassembly-depth` and
  `--reassembly-memcap` are rejected at argument-parse time.

**Observability**

- `dropped_findings` counter tracks findings discarded when the per-analyzer
  cap is reached; surfaced in JSON output.
- `skipped_packets` counter tracks packets skipped during decode; surfaced in
  all output formats.
- `truncated_records` counter tracks snaplen-truncated records; surfaced in
  JSON output.
- Criterion micro-benchmarks for hot paths in the decoder and reassembly
  engine.

### Security

- Bumped `indicatif` from 0.17 to 0.18 to transitively drop the unmaintained
  `number_prefix` crate (RUSTSEC-2025-0119).
- `cargo audit` and `cargo deny` supply-chain checks added to CI.
- Release profile enables `overflow-checks = true` so integer overflows are
  caught in release builds.
- Output sanitization in the terminal reporter guards against C1 control bytes
  in packet-derived strings.

[Unreleased]: https://github.com/Zious11/wirerust/compare/v0.12.1...HEAD
[0.12.1]: https://github.com/Zious11/wirerust/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/Zious11/wirerust/compare/v0.11.5...v0.12.0
[0.11.5]: https://github.com/Zious11/wirerust/compare/v0.11.4...v0.11.5
[0.11.4]: https://github.com/Zious11/wirerust/compare/v0.11.3...v0.11.4
[0.11.3]: https://github.com/Zious11/wirerust/compare/v0.11.2...v0.11.3
[0.11.2]: https://github.com/Zious11/wirerust/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/Zious11/wirerust/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/Zious11/wirerust/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/Zious11/wirerust/compare/v0.9.4...v0.10.0
[0.9.4]: https://github.com/Zious11/wirerust/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/Zious11/wirerust/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/Zious11/wirerust/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/Zious11/wirerust/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/Zious11/wirerust/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Zious11/wirerust/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/Zious11/wirerust/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/Zious11/wirerust/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Zious11/wirerust/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Zious11/wirerust/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Zious11/wirerust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Zious11/wirerust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Zious11/wirerust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Zious11/wirerust/releases/tag/v0.1.0
