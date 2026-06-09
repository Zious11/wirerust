---
document_type: feature-delta-analysis
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "Add Modbus TCP protocol analyzer"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  New module (src/analyzer/modbus.rs, ~400-600 LOC), dispatcher surgery
  (new DispatchTarget variant + port-502 classify branch + Modbus field + accessor/take),
  mitre.rs catalog extension (T0836 must be added), new behavioral contracts in a new
  subsystem namespace, and a new VP. Five distinct files require non-trivial changes.
  Not a trivial parameter tweak or docs-only change.
scope_classification: standard
status: draft
producer: architect
created: 2026-06-09
base_commit: 4cfc4c4
branch: develop
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
---

# F1 Delta Analysis — Issue #7: Add Modbus TCP Protocol Analyzer

## 1. Feature Summary

Implement a Modbus TCP analyzer for ICS/OT network forensics. The analyzer must:

- Detect Modbus TCP flows on port 502 (the IANA-registered port).
- Parse the 7-byte MBAP header (transaction ID `u16`, protocol ID `u16`, length `u16`,
  unit ID `u8`) and the function code byte that follows.
- Track a per-flow set of statistics: function-code distribution, write-operation count,
  exception-response count.
- Emit `Finding` instances for: (a) suspicious write bursts (rapid multiple write
  function codes within a flow window), (b) unusual/diagnostic function codes (0x08
  Diagnostics, 0x2B Encapsulated Interface Transport, and any reserved/vendor-specific
  codes), and (c) exception responses (high bit set on function code, i.e., function code
  >= 0x80).
- Map Findings to MITRE ATT&CK for ICS: T0855 (Unauthorized Command Message) for
  write-class function codes and unsolicited commands; T0836 (Modify Parameter) for
  write-multiple-registers / force-coil patterns.
- Surface a per-capture summary via `StreamAnalyzer::summarize` with keys:
  `function_code_distribution`, `write_count`, `exception_count`, `pdu_count`.

**Integration contract clarification:** GitHub #7 states "ModbusAnalyzer implements
ProtocolAnalyzer" but no `ProtocolAnalyzer` trait route exists for TCP. The correct
integration contract is `StreamHandler` + `StreamAnalyzer` (defined in
`src/reassembly/handler.rs`) as implemented by `HttpAnalyzer` and `TlsAnalyzer`. The
`ProtocolAnalyzer` trait (`src/analyzer/mod.rs:52`) is the UDP/packet-level route used
by `DnsAnalyzer`; it is not applicable to a flow-oriented binary protocol. This analysis
assumes `StreamHandler` integration throughout.

---

## 2. Intent and Scope Classification

| Field | Value |
|-------|-------|
| Intent | feature (new capability, no prior Modbus analysis exists) |
| Feature type | backend |
| Trivial | NO |
| Trivial justification | New module + dispatcher structural change + mitre.rs catalog extension + new VP + new subsystem BCs; minimum 5 files changed |
| Scope classification | standard — full F1-F7 cycle required |
| Recommended subsystem | SS-14 (new; see §6) |

---

## 3. Impact Boundary Table

### NEW Artifacts

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| `src/analyzer/modbus.rs` | NEW | `ModbusAnalyzer` struct implementing `StreamHandler` + `StreamAnalyzer`. Per-flow `HashMap<FlowKey, ModbusFlowState>`. No such file exists today. |
| Subsystem SS-14 "Modbus/ICS Analysis" | NEW | New behavioral-contract namespace BC-2.14.NNN. See §6 for subsystem justification. |
| VP-022 (proposed) | NEW | Modbus MBAP parse safety + function-code classification correctness. See §7. |
| MITRE T0836 entry in `mitre.rs` | NEW | T0836 ("Modify Parameter") is NOT currently seeded in `technique_info` (confirmed: `src/mitre.rs` line 122-155 has T0855/T0856/T0885 but no T0836). A new `Some`-returning arm must be added. `SEEDED_TECHNIQUE_IDS`, `SEEDED_TECHNIQUE_ID_COUNT`, and the `EMITTED_IDS` list in the Kani proofs must all update in lockstep to keep VP-007 green. |

### MODIFIED Files

| Component | Classification | Change Description |
|-----------|---------------|-------------------|
| `src/dispatcher.rs` | MODIFIED (HIGH RISK) | Add `DispatchTarget::Modbus` variant; extend `classify()` port-fallback arm to recognize port 502 as Modbus; add `modbus: Option<ModbusAnalyzer>` field to `StreamDispatcher`; add `modbus_analyzer()` and `take_modbus_analyzer()` accessors; route `on_data` and `on_flow_close` to Modbus arm. The `classify()` function at line 114 currently returns `DispatchTarget::{Http,Tls,None}`; a fourth variant and a new port-502 branch must not disturb the existing TLS (0x16 0x03) and HTTP method-token content rules or the port-fallback arms for 443/8443/80/8080. |
| `src/analyzer/mod.rs` | MODIFIED (LOW RISK) | Add `pub mod modbus;` at line 14. Single-line change, no logic. |
| `src/main.rs` | MODIFIED (MEDIUM RISK) | Wire `--modbus` / `--all` flags to `ModbusAnalyzer` construction (mirror the `http_analyzer` / `tls_analyzer` pattern at lines 128-138). Add `needs_reassembly` extension to include `enable_modbus`. Collect `modbus` findings after finalize (mirror lines 199-204). Add Modbus summary to `analyzer_summaries` (mirror lines 218-223). |
| `src/cli.rs` | MODIFIED (MEDIUM RISK) | Add `#[arg(long)] modbus: bool` flag to the `Commands::Analyze` variant (mirror `http` and `tls` at lines 140-147). Add `modbus` to `run_analyze` parameter list and `*modbus \|\| *all` expansion at line 57. |
| `src/mitre.rs` | MODIFIED (CRITICAL — VP-007 integrity) | Add `"T0836" => ("Modify Parameter", MitreTactic::IcsImpairProcessControl)` arm; bump `SEEDED_TECHNIQUE_ID_COUNT` from 15 to 16; add `"T0836"` to `SEEDED_TECHNIQUE_IDS` array; add `"T0836"` and `"T0855"` to `EMITTED_IDS` in the Kani `kani_proofs` module (currently `EMITTED_IDS` only has Enterprise IDs — T0855 is seeded but not listed as emitted). The `vp007_catalog_drift_guard` test sweeps the full `T[0-9]{4}` space and will mechanically fail if `technique_info` resolves T0836 but `SEEDED_TECHNIQUE_IDS` doesn't include it. |
| `src/dispatcher.rs` (Kani proofs) | MODIFIED | The `kani_proofs` module at line 237 contains VP-004 proofs. The `classify_oracle` at line 283 mirrors the production `classify` function exactly. When `DispatchTarget::Modbus` is added, the oracle must gain the identical `port 502 → Modbus` branch. The `verify_content_first_precedence_exhaustive` proof (line 316) asserts `got == want`; if oracle and production diverge, that proof fails. |

### DEPENDENT (unchanged, must stay green)

| Component | Classification | Dependency |
|-----------|---------------|-----------|
| `src/reassembly/handler.rs` | DEPENDENT | `StreamHandler` trait is the integration surface; unchanged. `ModbusAnalyzer` implements it without modifications to handler.rs. |
| `src/reassembly/mod.rs` | DEPENDENT | `TcpReassembler::process_packet` calls `dispatcher.on_data`; the Modbus arm slots in transparently. No change needed. |
| `tests/dispatcher_tests.rs` | DEPENDENT (regression zone) | All existing VP-004 tests (`test_tls_content_wins_over_port_8080`, classify tests) must remain green. The Modbus port-502 branch must not affect the 443/8443/80/8080 fallback tests. |
| `tests/http_analyzer_tests.rs` | DEPENDENT (regression zone) | No HTTP analysis change; tests must stay green. |
| `tests/tls_analyzer_tests.rs` | DEPENDENT (regression zone) | No TLS analysis change; tests must stay green. |
| `tests/mitre_tests.rs` | DEPENDENT (regression zone) | VP-007 drift-guard test (`vp007_catalog_drift_guard`) will be exercised; it must pass after T0836 is added and counts bumped. |
| `tests/reporter_tests.rs`, `tests/reporter_terminal_tests.rs` | DEPENDENT | Reporter consumes `AnalysisSummary` from `StreamAnalyzer::summarize`; the new `BTreeMap` detail from `ModbusAnalyzer` slots in with no reporter code change. |
| `src/findings.rs` | DEPENDENT | `ThreatCategory` and `Finding` struct are unchanged. Modbus findings use the existing `ThreatCategory::Anomaly` and (if needed) `ThreatCategory::Execution` variants. No new variants required; `#[non_exhaustive]` already accommodates future growth. |

---

## 4. Regression Risk Assessment

### `src/dispatcher.rs` — RISK: HIGH

This is the most sensitive file in the feature. `classify()` and `on_data()` are CRITICAL
path components (VP-004, module-criticality.md: `Content-first dispatch` = CRITICAL tier).
Every HTTP and TLS flow touches `classify()` on first chunk.

**Specific risks:**
- Adding `DispatchTarget::Modbus` requires updating the `match target` arm in `on_data`
  (line 187). An incomplete match will generate a Rust dead-code or non-exhaustive warning
  (which CI treats as an error via `RUSTFLAGS=-Dwarnings`).
- The port-502 fallback must be inserted after the 443/8443 and 80/8080 arms to preserve
  the existing precedence ordering. Inserting it before would cause no regression in those
  exact tests (they use specific ports), but the oracle in `classify_oracle` (VP-004 Kani
  proof at line 283) must mirror the production branch order exactly or `verify_content_first_precedence_exhaustive` will fail.
- The `unclassified_flows` counter increments on `DispatchTarget::None` close. The Modbus
  arm must NOT fall through to the `None` close path; `on_flow_close` needs a
  `Some(DispatchTarget::Modbus)` match arm routing to `modbus.on_flow_close(...)`.
- The early-exit guard at line 152 (`if self.http.is_none() && self.tls.is_none()`) must
  be extended to also check `self.modbus.is_none()` or the entire guard removed in favor
  of per-arm `if let Some(ref mut x)` guards.

**Tests in regression zone:**
- `tests/dispatcher_tests.rs` — all VP-004 tests covering TLS-beats-port, HTTP content
  detection, port-fallback, and None-caching must stay green.
- Kani proofs in `dispatcher.rs` (compiled with `cargo kani`): VP-004
  `verify_content_first_precedence_exhaustive`, `verify_tls_signature_beats_port`,
  `verify_none_two_phase_caching`.

### `src/mitre.rs` — RISK: CRITICAL (VP-007 integrity)

The VP-007 catalog drift guard (`vp007_catalog_drift_guard` in `tests/mitre_tests.rs`) sweeps
all 10,000 `T[0-9]{4}` parent IDs and all 10,000,000 `T[0-9]{4}.[0-9]{3}` sub-technique IDs.
It **mechanically fails** if `technique_info` resolves an ID that is not in
`SEEDED_TECHNIQUE_IDS`. Failure mode: add `T0836` to the match arm but forget to update
`SEEDED_TECHNIQUE_IDS` or `SEEDED_TECHNIQUE_ID_COUNT`. Both must change atomically.

Additionally, the Kani `verify_all_seeded_ids_resolve` and `verify_all_emitted_ids_resolve`
proofs enumerate a `const SEEDED_IDS` and `const EMITTED_IDS` inside `kani_proofs`. T0836
must be added to `SEEDED_IDS`; T0836 and T0855 must be added to `EMITTED_IDS` (T0855 is
seeded but currently not in `EMITTED_IDS`, meaning the Kani emitted-IDs proof is
incomplete today — this feature cycle is the right moment to fix both gaps).

### `src/main.rs` — RISK: MEDIUM

The `run_analyze` function is the integration glue. The pattern is well-established
(HttpAnalyzer and TlsAnalyzer show the exact four-step pattern: construct, pass to dispatcher,
collect findings, collect summary). The risk is omitting one of the four steps:
1. Construction guard (`if enable_modbus && !skip_reassembly`).
2. `needs_reassembly` extension to include `enable_modbus`.
3. Post-finalize findings collection.
4. `analyzer_summaries.push(modbus.summarize())`.

Omitting step 2 means `--modbus` alone would not trigger reassembly, silently analyzing
no flows. Omitting step 3 drops all Modbus findings. Both are behavioral regressions
against the BC postconditions.

**Tests in regression zone:**
- `tests/cli_tests.rs`, `tests/cli_integration_tests.rs` — general CLI structure tests.
- `tests/multi_analyzer_e2e_tests.rs` — exercises multiple analyzers together.

### `src/cli.rs` — RISK: MEDIUM

Adding a new flag (`--modbus`) to an existing clap derive struct is low-risk syntactically.
Risk area: the `*modbus || *all` expansion in `main.rs`. The LESSON-P1.04 convention
("no unwired CLI flags") requires the flag to be consumed in `run_analyze`.

### `src/analyzer/modbus.rs` — RISK: LOW

New code. No existing tests exercise it. The risk is internal correctness (MBAP parse
off-by-ones, function-code classification bugs) which is the subject of F3 stories and VP-022.
No regression to existing functionality.

### `src/analyzer/mod.rs` — RISK: LOW

Single-line `pub mod modbus;` addition. The only risk is a compile error if `modbus.rs`
does not exist yet when this line is added.

---

## 5. Files-Likely-Changed and Regression Baseline

### Files that WILL change (new + modified)

```
src/analyzer/modbus.rs              [NEW]
src/analyzer/mod.rs                 [MODIFIED — add pub mod modbus]
src/dispatcher.rs                   [MODIFIED — DispatchTarget::Modbus + classify + on_data + on_flow_close + field + accessors]
src/main.rs                         [MODIFIED — --modbus wiring, findings collection, summary]
src/cli.rs                          [MODIFIED — --modbus flag]
src/mitre.rs                        [MODIFIED — T0836 arm + SEEDED/COUNT/EMITTED updates]
.factory/specs/behavioral-contracts/ss-14/*.md  [NEW — BC-2.14.NNN files]
.factory/specs/behavioral-contracts/BC-INDEX.md [MODIFIED — ss-14 section]
.factory/specs/architecture/ARCH-INDEX.md       [MODIFIED — SS-14 in subsystem registry]
.factory/specs/verification-properties/VP-022-modbus-mbap-parse-safety.md  [NEW]
.factory/specs/verification-properties/VP-INDEX.md  [MODIFIED — VP-022 addition]
.factory/specs/architecture/verification-architecture.md  [MODIFIED — VP-022 row]
.factory/specs/architecture/verification-coverage-matrix.md  [MODIFIED — VP-022 row + totals]
.factory/specs/module-criticality.md  [MODIFIED — ModbusAnalyzer classification]
```

### Files that WILL NOT change (regression baseline — must stay green)

```
src/reassembly/handler.rs           (StreamHandler trait unchanged)
src/reassembly/mod.rs               (process_packet dispatch unchanged)
src/reassembly/flow.rs              (FlowKey unchanged)
src/reassembly/segment.rs           (segment logic unchanged)
src/reassembly/config.rs            (ReassemblyConfig unchanged)
src/reassembly/stats.rs             (stats unchanged)
src/reassembly/lifecycle.rs         (lifecycle unchanged)
src/analyzer/http.rs                (HttpAnalyzer unchanged)
src/analyzer/tls.rs                 (TlsAnalyzer unchanged)
src/analyzer/dns.rs                 (DnsAnalyzer unchanged)
src/findings.rs                     (Finding/ThreatCategory unchanged)
src/decoder.rs                      (port 502 hint "Modbus" already present at line 112)
src/reporter/mod.rs                 (Reporter trait unchanged)
src/reporter/terminal.rs            (MITRE grouping loop unchanged; T0836 slots in via all_tactics_in_report_order)
src/reporter/json.rs                (unchanged)
src/reporter/csv.rs                 (unchanged)
src/summary.rs                      (unchanged)
src/reader.rs                       (unchanged)
src/lib.rs                          (pub mod re-exports; modbus module pub visibility handled in analyzer/mod.rs)
```

**Note on `src/decoder.rs`:** Port 502 is already in `app_protocol_hint` at line 112 as
`Some("Modbus")`. This is a documentation-level hint used by `Summary.ingest` for the
"services detected" display; it is NOT the dispatch path. No change needed.

---

## 6. Architecture Decision Flags for F2

### ADR-REQUIRED: ICS/OT Binary Protocol Integration Pattern

**Question:** Does adding a binary request/response protocol analyzer (Modbus, and
eventually DNP3, IEC 61850) warrant a new ADR?

**Recommendation: YES.** The current two ADRs (ADR 0001 / ADR 0002) cover HTTP-and-TLS,
which are text protocols classifiable by content prefix. Modbus differs in three
structurally distinct ways:

1. **Port-only classification.** Modbus TCP has no content-level fingerprint distinguishable
   from arbitrary binary data. The MBAP protocol-ID field (`0x0000` = Modbus) appears at
   bytes 2-3 of the payload, not at bytes 0-1. The dispatcher's content-first rule cannot
   detect Modbus from payload prefix alone without a 4-byte peek that overlaps with the
   MBAP length field. Port-502 fallback is the only reliable classification path. This is
   a documented exception to ADR 0001's content-first policy and should be recorded.

2. **PDU-oriented vs. stream-oriented parsing.** HTTP and TLS parse variable-length
   framing from a reassembled byte stream using dedicated parser crates (httparse,
   tls_parser). Modbus PDUs have a fixed 7-byte MBAP header followed by a fixed-length
   data unit. The MBAP `length` field defines the PDU boundary. `ModbusAnalyzer` will
   parse complete PDUs from the reassembled stream with a simple offset-advancing loop —
   no external parser crate required. This pattern (manual binary PDU extraction from
   reassembled TCP) is new to the codebase.

3. **Request/response correlation requirement.** Modbus uses a `transaction_id` (bytes
   0-1 of MBAP) to correlate requests and responses across directions. A write-burst
   detection strategy that operates only on per-chunk data (as HTTP and TLS do) will
   count duplicated transaction IDs if the connection is lossy or captured mid-stream.
   The ADR decision is: **stateless-per-PDU vs. request/response tracking**.

   **Recommendation for ADR: stateless-per-PDU for v1.** Tracking transaction ID
   cross-direction requires a `HashMap<u16, FunctionCode>` in per-flow state and adds
   correctness complexity (transaction ID reuse after rollover, mid-stream join). For
   the primary detection goals — write-burst anomaly, unusual function codes, exception
   responses — stateless processing of each PDU independently is sufficient and avoids
   the state management overhead. Transaction-ID matching can be added in a later cycle
   as an enhancement with its own VP.

**Subsystem recommendation: SS-14 "Modbus/ICS Analysis" (NEW, do not extend SS-07 or SS-08)**

Rationale:
- SS-07 (TLS Analysis) and SS-08 (DNS Analysis) are domain-specific; mixing ICS/OT
  protocol logic into them would violate the principle that subsystems map 1:1 to
  distinct protocols.
- The issue requests a family of ICS analyzers ("Modbus / DNP3" in mitre.rs docs,
  "ics/ot" GitHub label). A dedicated SS-14 provides a namespace for DNP3 and future
  ICS protocols without renumbering.
- SS-14 would own CAP-14 (not yet defined in domain-spec), a new capability entry
  covering ICS/OT protocol analysis. The domain-spec capabilities.md will need a
  CAP-14 entry in F2.
- BC numbering: BC-2.14.NNN. This follows the established NN = capability pattern.

### MITRE Model Decision

**Question:** Does the `ThreatCategory` enum need a new ICS variant?

**Recommendation: NO for v1.** The existing `ThreatCategory` values (`Anomaly`,
`Execution`, `Reconnaissance`) map adequately to the primary Modbus finding types:
- Exception responses → `ThreatCategory::Anomaly`
- Unusual function codes (0x08 Diagnostics) → `ThreatCategory::Anomaly`
- Suspicious write bursts (T0855/T0836) → `ThreatCategory::Execution` (matches the
  existing use for "suspicious code execution patterns" and write commands)

A future `ThreatCategory::IcsImpairProcessControl` variant could be added if ICS-specific
tactic grouping is needed in the terminal reporter, but that requires a reporter change and
is a separate decision. `#[non_exhaustive]` means adding a variant is non-breaking.

**T0836 is NOT currently seeded.** The mitre.rs catalog has T0855, T0856, T0885 (ICS)
but is missing T0836 ("Modify Parameter", ICS ImpairProcessControl). This must be added
in F2 as part of the spec work before F3 implementation. The VP-007 Kani proofs enumerate
a closed set — the seeded-list and count must update atomically with the `technique_info`
match arm.

### `IcsInhibitResponseFunction` vs. `IcsImpairProcessControl` tactic assignment

T0836 and T0855 both belong to MITRE ICS tactic "Impair Process Control" (TA0106 in the
ICS matrix). The existing `MitreTactic::IcsImpairProcessControl` variant in `mitre.rs`
is the correct assignment for both. No new tactic variant needed.

---

## 7. Recommended F2/F3 Scope

### Subsystem: SS-14 (new, "Modbus/ICS Analysis")

### New Behavioral Contracts (estimated count: 12-16 BCs)

Recommended BC groups for F2 spec:
1. **MBAP parse group (~4 BCs):** Accept well-formed 7-byte MBAP header; reject PDUs
   shorter than 8 bytes (header + function code); reject PDUs with protocol-ID != 0x0000;
   length field must be consistent with actual data available.
2. **Function code classification (~3 BCs):** Read Coils (0x01), Read Holding Registers
   (0x03), Write Single Coil (0x05), Write Multiple Registers (0x10) classified as
   write-class. Diagnostics (0x08) classified as unusual. Exception response (MSB set)
   classified separately.
3. **Finding emission (~3 BCs):** Write-burst finding emitted when write-class function
   code count exceeds threshold in a single flow (T0855). Unusual function code finding
   emitted per occurrence (T0836 or Anomaly). Exception response finding emitted per
   occurrence (Anomaly).
4. **Per-flow state lifecycle (~2 BCs):** Per-flow state created on first on_data; removed
   on on_flow_close (mirrors HTTP/TLS on_flow_close cleanup).
5. **Summary output (~2 BCs):** `summarize()` returns BTreeMap with keys
   `function_code_distribution`, `write_count`, `exception_count`, `pdu_count`.
6. **Integration (~1-2 BCs):** `--modbus` CLI flag enables analyzer; `--all` includes Modbus.

### New Verification Property: VP-022

**Proposed title:** "Modbus MBAP Parse Safety and Function-Code Boundary Classification"

**Proof strategy (Kani — recommended P1):**
- Sub-property A: MBAP header extraction never panics on any input slice shorter than 8
  bytes (the minimum valid PDU size). Kani harness: symbolic `&[u8]` of length 0..8,
  assert no panic.
- Sub-property B: Function code classification is total and covers all 256 values
  (0x00-0xFF). Kani harness: symbolic `u8` for function code, assert the classification
  function returns one of {Write, Read, Diagnostic, Exception, Unknown} with no gaps.
- Sub-property C: Exception response detection fires if and only if function code >= 0x80
  (high bit set). Kani: symbolic `u8`, assert `is_exception(fc) == (fc >= 0x80)`.

**Feasibility:** HIGH. All three sub-properties operate on small bounded inputs (byte
slices ≤8 bytes, single u8). Kani handles these with no bound explosion. The function-code
classification is a simple match on a `u8` — exhaustive over 256 values, analogous to the
VP-005 SNI single-byte arm model (which proved in milliseconds).

**Phase assignment:** P1 (recommended; the property is correctness-critical but the module
is new code without legacy verification debt, making P1 appropriate over P0).

### Stories (estimated count: 3-4 stories)

Recommended story decomposition:
- **STORY-NNN (F3):** Modbus MBAP parser and function-code classification (pure core
  of `ModbusAnalyzer` — PDU extraction, per-flow state, statistics counters). TDD.
- **STORY-NNN+1 (F3):** Modbus Finding emission (write-burst detection, unusual function
  code, exception response findings with MITRE mapping T0855/T0836). TDD.
- **STORY-NNN+2 (F3):** Dispatcher wiring + CLI flag + reporter summary integration. TDD.
- **STORY-NNN+3 (F3, optional — can merge into NNN+2):** VP-022 Kani proof harness
  for MBAP parse safety and function-code classification totality.

---

## 8. MITRE Model Detailed State

| Technique | Status in mitre.rs | Action needed |
|-----------|--------------------|---------------|
| T0855 (Unauthorized Command Message) | SEEDED (line 144), NOT in EMITTED_IDS | Add to EMITTED_IDS in Kani proofs |
| T0836 (Modify Parameter) | NOT SEEDED | Add `technique_info` arm + update SEEDED_TECHNIQUE_IDS + SEEDED_TECHNIQUE_ID_COUNT (15→16) + EMITTED_IDS |
| T0856 (Spoof Reporting Message) | SEEDED (line 148), not emitted | No action (already staged per mitre.rs design intent) |
| T0885 (Commonly Used Port) | SEEDED (line 152), not emitted | No action (staged) |
| T0846 (Remote System Discovery) | SEEDED (line 143), not emitted | No action |

**VP-007 atomic update rule:** Modifying `technique_info` requires updating five things in
the same commit: (1) the match arm, (2) `SEEDED_TECHNIQUE_IDS` array, (3)
`SEEDED_TECHNIQUE_ID_COUNT` constant, (4) `EMITTED_IDS` in `kani_proofs`, and
(5) running `cargo test mitre` to confirm `vp007_catalog_drift_guard` passes before
the implementation PR merges.

---

## 9. Purity Boundary Classification (for F2 architecture section)

`ModbusAnalyzer` follows the same purity model as `HttpAnalyzer` and `TlsAnalyzer`:

| Component | Classification |
|-----------|---------------|
| MBAP header extraction (`fn parse_mbap(data: &[u8]) -> Option<MbapHeader>`) | Pure core: deterministic, no I/O |
| Function code classification (`fn classify_function_code(fc: u8) -> FunctionCodeClass`) | Pure core: total function over u8 |
| Write-burst detection logic (count threshold comparison) | Pure core: operates on per-flow counters |
| Finding construction (`all_findings.push(...)`) | Effectful shell (mutates Vec), but referentially transparent in semantics |
| `on_data` / `on_flow_close` (HashMap mutation) | Effectful shell: modifies `flows` HashMap |

VP-022 Kani harnesses will target only the pure-core functions (`parse_mbap`,
`classify_function_code`). The effectful shell (on_data loop, HashMap ops) is covered by
integration tests.

---

## Appendix: Key Line References

| Location | Line(s) | Relevance |
|----------|---------|-----------|
| `dispatcher.rs` classify fn | 114-141 | Port-fallback arms to extend with port 502 |
| `dispatcher.rs` on_data | 143-200 | DispatchTarget match arm — add Modbus case |
| `dispatcher.rs` on_flow_close | 202-225 | Add `Some(DispatchTarget::Modbus)` close arm |
| `dispatcher.rs` StreamDispatcher fields | 42-54 | Add `modbus: Option<ModbusAnalyzer>` |
| `dispatcher.rs` new() | 57-66 | Update constructor signature |
| `dispatcher.rs` early-exit guard | 152 | Extend null check to include Modbus |
| `dispatcher.rs` Kani oracle | 283-313 | Must mirror new port-502 classify branch |
| `analyzer/mod.rs` | 14 | Add `pub mod modbus;` |
| `main.rs` run_analyze | 73-249 | Four-step Modbus wiring (construct, reassembly flag, findings, summary) |
| `cli.rs` Commands::Analyze | 131-155 | Add `modbus: bool` field |
| `mitre.rs` technique_info | 122-155 | Add T0836 arm after T0855 |
| `mitre.rs` SEEDED_TECHNIQUE_IDS | 260-278 | Add "T0836" |
| `mitre.rs` SEEDED_TECHNIQUE_ID_COUNT | 286 | Bump 15 → 16 |
| `mitre.rs` EMITTED_IDS (kani) | 191-198 | Add "T0836", "T0855" |
| `decoder.rs` app_protocol_hint | 112 | Port 502 → "Modbus" already present; no change |
| `findings.rs` ThreatCategory | 88-111 | Unchanged; `Anomaly` and `Execution` sufficient |
