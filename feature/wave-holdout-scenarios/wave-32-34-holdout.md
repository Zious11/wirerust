---
document_type: holdout-scenario
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
waves: [32, 33, 34]
cycle: v0.4.0-modbus
stories: [STORY-102, STORY-103, STORY-104, STORY-105]
feature_id: issue-007-modbus-analyzer
github_issue: 7
---

# Waves 32–34 Holdout Scenarios: Modbus TCP Analyzer (v0.4.0)

> **Purpose:** End-to-end integration holdout for the Modbus TCP analyzer.
> Validates that a crafted Modbus pcap produces the correct 7-detector findings
> with multi-tag attribution, that existing analyzers are unaffected by the new
> port-502 dispatch rule, and that the dual-window burst/sustained detectors
> and T0831 coordinated-write detector fire correctly.

---

## Per-Wave Gate Summary

| Wave | Stories | Gate Criteria |
|------|---------|---------------|
| 32 | STORY-102 | Pure-core unit tests + VP-022 Kani sub-properties A and B |
| 33 | STORY-103 + STORY-104 | Transaction correlation + all 7 detection rules |
| 34 | STORY-105 | End-to-end CLI wiring + dispatcher regression |

---

## HS-W32-001: MBAP Parse — Canonical Vector

**Scope:** STORY-102 (parse_mbap_header)
**Priority:** P0
**Wave:** 32

**Input:** 12-byte slice `[0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]`

**Assertions:**
1. `parse_mbap_header` returns `Some(MbapHeader { transaction_id: 0x0001, protocol_id: 0x0000, length: 6, unit_id: 0x01, function_code: 0x03 })`.
2. `is_valid_modbus_adu` returns `true` for this header.
3. ADU offset advance = `6 + 6 = 12` bytes (entire slice consumed).

---

## HS-W32-002: MBAP Parse — Rejection Cases

**Scope:** STORY-102 (parse_mbap_header + is_valid_modbus_adu)
**Priority:** P0
**Wave:** 32

**Assertions:**
1. `parse_mbap_header` returns `None` for slices of length 0, 1, 4, 7 (all < 8).
2. `is_valid_modbus_adu` returns `false` when `protocol_id != 0x0000`.
3. `is_valid_modbus_adu` returns `false` when `length < 2` (e.g., length=0, length=1).
4. `is_valid_modbus_adu` returns `false` when `length > 253` (e.g., length=254).
5. `is_valid_modbus_adu` returns `true` at both boundary values: length=2 and length=253.
6. `parse_mbap_header` does NOT reject based on `protocol_id` or `length` — it returns `Some` with raw values parsed; only `is_valid_modbus_adu` gates validity.

---

## HS-W32-003: FC Classification — Completeness and Totality

**Scope:** STORY-102 (classify_fc)
**Priority:** P0
**Wave:** 32

**Assertions:**
1. `classify_fc` returns a valid variant for all 256 u8 values (no panic, no unreachable arm).
2. All FCs with high bit set (>= 0x80) return `Exception`. Representative: 0x80, 0x83, 0xFF.
3. Write-class FCs {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17} return `Write`.
4. Diagnostic FCs {0x08, 0x2B} return `Diagnostic`.
5. Read FCs {0x01, 0x02, 0x03, 0x04, 0x07, 0x0B, 0x0C, 0x11, 0x14} return `Read`.
6. FC 0x7F (no high bit, undefined) returns `Unknown`.
7. Exception classification takes priority: FC=0xFF returns `Exception` (not `Unknown`).

---

## HS-W33-001: Transaction Correlation — Request/Response Cycle

**Scope:** STORY-103 (pending table + STORY-104 detection)
**Priority:** P0
**Wave:** 33

**Setup:** Deliver to `ModbusAnalyzer.on_data`:
1. A ClientToServer ADU with FC=0x03 (Read Holding Registers), transaction_id=1, unit_id=1.
2. A ServerToClient ADU with FC=0x03 (echo), same transaction_id and unit_id.

**Assertions:**
1. After step 1: `flow.pending.contains_key(&(1, 1)) == true`; `pdu_count == 1`.
2. After step 2: `flow.pending.is_empty() == true`; `pdu_count == 2`.
3. No finding emitted for a Read-class request/response pair (no detection rule fires).

---

## HS-W33-002: Pending Table Bounded at 256

**Scope:** STORY-103 (MAX_PENDING_TRANSACTIONS cap)
**Priority:** P0
**Wave:** 33

**Assertions:**
1. Insert 256 requests with distinct (txn_id, unit_id) keys → `pending.len() == 256`.
2. Attempt 257th insert → `pending.len()` remains 256 (new entry dropped).
3. Existing 256 entries are unaffected by the cap-drop.
4. `pdu_count` is still incremented even when the pending insert is dropped.

---

## HS-W33-003: Write-Class Detections — Holding Register (T0855 + T0836)

**Scope:** STORY-104 (BC-2.14.013 + BC-2.14.014)
**Priority:** P0
**Wave:** 33

**Setup:** Deliver a ClientToServer ADU with FC=0x06 (Write Single Register).

**Assertions:**
1. Exactly ONE Finding is emitted.
2. `finding.mitre_techniques == vec!["T0855", "T0836"]` (canonical order per ADR-006 sub-decision 3).
3. `finding.category == ThreatCategory::Execution`.
4. `finding.verdict == Verdict::Likely`.
5. `finding.confidence == Confidence::Medium`.

**Repeat for FC=0x10 (Write Multiple Registers) and FC=0x16 (Mask Write Register):**
Same assertions — each emits `["T0855", "T0836"]`.

---

## HS-W33-004: Write-Class Detections — Coil (T0855 + T0835)

**Scope:** STORY-104 (BC-2.14.015)
**Priority:** P0
**Wave:** 33

**Setup:** Deliver a ClientToServer ADU with FC=0x05 (Write Single Coil).

**Assertions:**
1. ONE Finding with `mitre_techniques == vec!["T0855", "T0835"]`.

**Repeat for FC=0x0F (Write Multiple Coils):** Same.

**Negative case (FC=0x15/0x17):** ONE Finding with `mitre_techniques == vec!["T0855"]` only
(not in register or coil subset).

---

## HS-W33-005: T0831 Coordinated Write — Inline Co-Tag on 2nd Write Within 5s

**Scope:** STORY-104 (BC-2.14.016)
**Priority:** P0
**Wave:** 33

**Setup:** Deliver three holding-register writes (FC=0x06) within a 5-second window.
Timestamps: t=100_000 µs, t=200_000 µs (1s later), t=300_000 µs (2s later).

**Assertions:**
1. First write (t=100_000): finding[0].mitre_techniques = `["T0855", "T0836"]` (T0831 NOT yet co-tagged; count=1).
2. Second write (t=200_000): finding[1].mitre_techniques = `["T0855", "T0836", "T0831"]` (T0831 inline, emit-once; count=2; `t0831_burst_emitted = true`).
3. Third write (t=300_000): finding[2].mitre_techniques = `["T0855", "T0836"]` (T0831 exhausted for this window; no re-emit).
4. T0831 is NOT a separate Finding object — it is co-tagged inline in the per-PDU write finding.

**Window reset case:** Deliver a 4th write at t=5_200_000 µs (> 5s after first write).
5. finding[3].mitre_techniques = `["T0855", "T0836"]` (new window started; T0831 counter reset; emit-once guard cleared).
6. A 5th write within the new window (t=5_300_000): finding[4].mitre_techniques = `["T0855", "T0836", "T0831"]` (T0831 fires again in the new window).

---

## HS-W33-006: Dual-Window Burst Detector (T0806 + T0855)

**Scope:** STORY-104 (BC-2.14.017 — 1-second burst window)
**Priority:** P0
**Wave:** 33

**Setup:** Default `write_burst_threshold = 20`. Deliver 21 ClientToServer write-class ADUs
within 1 second (e.g., timestamps from 0 to 900_000 µs).

**Assertions:**
1. 21 per-PDU write findings emitted (one per write; each `["T0855", "T0836"]` or similar).
2. Exactly ONE burst finding emitted with `mitre_techniques == vec!["T0806", "T0855"]`.
3. The burst finding is a SEPARATE Finding object, NOT merged into a per-PDU finding.
4. Total findings: 22 (21 per-PDU + 1 burst).
5. `window_burst_emitted == true` after the threshold-tipping write.

**Guard test:** Deliver a 22nd write within the same 1-second window.
6. No second burst finding emitted (`window_burst_emitted` guard prevents re-fire).
7. Total remains 23 per-PDU findings + 1 burst = 23+1 = 24 total? No — 22 per-PDU + 1 burst = 23.

---

## HS-W33-007: Sustained Rate Detector — Truncation-Free Math

**Scope:** STORY-104 (BC-2.14.017 — sustained window, truncation-free formula)
**Priority:** P0
**Wave:** 33

**The defect this test guards against (from f2-fix-directives.md §11.5a):**
Naive integer division `elapsed_secs = elapsed_us / 1_000_000` truncates 2.9s to 2s, producing
a false-positive burst at 25 writes over 2.9s (would compute rate=25/2=12.5 > threshold=10).

**Test case A — no false positive (2.9s window):**
Deliver 25 writes over 2.9 seconds (elapsed_us = 2_900_000).
Formula: `25 * 1_000_000 = 25_000_000 > 10 * 2_900_000 = 29_000_000` → FALSE.
Assertion: NO sustained burst finding emitted.

**Test case B — correct fire (2.0s window):**
Deliver 25 writes over 2.0 seconds (elapsed_us = 2_000_000).
Formula: `25 * 1_000_000 = 25_000_000 > 10 * 2_000_000 = 20_000_000` → TRUE.
Assertion: ONE sustained burst finding emitted with `["T0806", "T0855"]`.

**Timestamp wrap test:**
Deliver a write at `ts = 0xFFFFFF00` followed by one at `ts = 0x00000100`.
`wrapping_sub(0x00000100, 0xFFFFFF00) = 0x00000200 = 512 µs`.
Assertion: No panic (overflow-checks=true in debug); window elapsed = 512 µs (< 2s minimum).

---

## HS-W33-008: Diagnostics Detector — T0814 DoS Finding

**Scope:** STORY-104 (BC-2.14.018)
**Priority:** P0
**Wave:** 33

**Setup:** Deliver a ClientToServer ADU with FC=0x08 and sub-function bytes `0x00 0x04`
(Force Listen Only Mode).

**Assertions:**
1. ONE Finding with `mitre_techniques == vec!["T0814"]`.
2. `category == ThreatCategory::IcsInhibitResponseFunction`.

**Repeat for sub-function `0x00 0x01` (Restart Communications Option):** Same.

**Negative case:** FC=0x08 with sub-function `0x00 0x00` (Return Query Data): NO T0814 finding.

---

## HS-W33-009: Exception-Burst Anomaly Detector

**Scope:** STORY-104 (BC-2.14.019)
**Priority:** P1
**Wave:** 33

**Setup:** Deliver 11+ exception responses for FC=0x83 (exception for FC=0x03) within 10 seconds.

**Assertions:**
1. ONE `Anomaly` finding emitted (no MITRE technique: `mitre_techniques == vec![]`).
2. `exception_burst_emitted[0x83] == true` (guard prevents re-emission).
3. No second Anomaly finding on the 12th or subsequent exception.

---

## HS-W33-010: Recon Detector — T0888 Remote System Information Discovery

**Scope:** STORY-104 (BC-2.14.020)
**Priority:** P0
**Wave:** 33

**Assertions:**
1. FC=0x11 (Report Server ID) in ClientToServer → ONE Finding with `["T0888"]`.
2. FC=0x2B with MEI type=0x0E (Read Device Identification) in ClientToServer → ONE Finding with `["T0888"]`.
3. FC=0x07 (Read Exception Status) in ClientToServer → NO finding (Decision 12; FC=0x07 is not a recon indicator).
4. FC=0x2B with MEI type != 0x0E → NO T0888 finding.

---

## HS-W33-011: MAX_FINDINGS Cap — Poison-Skip Behavior

**Scope:** STORY-104 (BC-2.14.022)
**Priority:** P0
**Wave:** 33

**Setup:** Pre-fill `all_findings` to exactly 10,000 entries. Deliver one write-class PDU.

**Assertions:**
1. `all_findings.len() == 10_000` (no growth past cap).
2. `dropped_findings == 1` (one push skipped).
3. `write_count` is still incremented (counter update is NOT skipped).
4. No panic.

---

## HS-W33-012: summarize() — Six Keys with Correct Values

**Scope:** STORY-104 (BC-2.14.021)
**Priority:** P0
**Wave:** 33

**Setup:** Process a mix of ADUs: 5 reads, 3 writes, 1 exception, 1 invalid (parse error).

**Assertions:**
1. `summarize()` returns an `AnalysisSummary` with exactly 6 keys.
2. `pdu_count` reflects all processed PDUs.
3. `write_count` reflects write-class PDUs.
4. `exception_count` reflects exception responses.
5. `parse_errors` reflects ADUs that failed the 3-point validity gate.
6. `findings_emitted` = `all_findings.len()`.
7. `dropped_findings` = count of finding-push skips due to MAX_FINDINGS cap.
8. The key `duplicate_inflight_txn` does NOT appear in the summary (it is an internal counter only).

---

## HS-W34-001: End-to-End — Crafted Modbus pcap with 7 Detector Fires

**Scope:** STORY-105 (dispatcher integration + CLI wiring)
**Priority:** P0
**Wave:** 34

**Setup:** Craft a synthetic Modbus TCP pcap containing:
- 1 holding-register write (FC=0x06): triggers T0855+T0836 per-PDU finding
- 1 coil write (FC=0x05): triggers T0855+T0835 per-PDU finding
- 2 holding-register writes within 5s: triggers T0831 co-tag on 2nd write
- 21 holding-register writes within 1s: triggers T0806+T0855 burst finding
- FC=0x08 with sub-function 0x0004: triggers T0814 finding
- FC=0x11 (recon): triggers T0888 finding

Run: `wirerust analyze <modbus.pcap> --modbus --format json`

**Assertions:**
1. `cargo run` exits 0.
2. JSON output includes a non-empty `"findings"` array.
3. At least one finding with `mitre_techniques` containing `"T0855"`.
4. At least one finding with `mitre_techniques` containing `"T0806"` (burst detector).
5. At least one finding with `mitre_techniques` containing `"T0814"`.
6. At least one finding with `mitre_techniques` containing `"T0888"`.
7. At least one finding with `mitre_techniques` containing `"T0831"` (co-tagged inline).
8. `"analyzers"` array includes a Modbus summary entry with `pdu_count > 0`.
9. No crash, no stack overflow, no panic.

---

## HS-W34-002: Dispatcher Port-502 Rule Does NOT Steal HTTP or TLS Traffic

**Scope:** STORY-105 (BC-2.14.025 — Rule 5 fires AFTER content rules)
**Priority:** P0
**Wave:** 34

**Assertions:**
1. A port-502 flow carrying TLS ClientHello bytes → `DispatchTarget::Tls` (content-first Rule 1 wins).
2. A port-502 flow carrying HTTP GET bytes → `DispatchTarget::Http` (content-first Rule 2 wins).
3. A port-443 flow → `DispatchTarget::Tls` (Rule 3; NOT affected by port-502 addition).
4. A port-80 flow → `DispatchTarget::Http` (Rule 4; unchanged).
5. A port-502 flow with non-TLS/HTTP binary → `DispatchTarget::Modbus` (Rule 5 fires correctly).

---

## HS-W34-003: --modbus Default-Off — No Breaking Change to Existing Invocations

**Scope:** STORY-105 (BC-2.14.023 invariant 1)
**Priority:** P0
**Wave:** 34

**Assertions:**
1. `wirerust analyze <any.pcap> --http --format json` — no `--modbus` flag — produces output with no Modbus section.
2. Port-502 flows in the pcap receive `DispatchTarget::None` (no Modbus analysis).
3. Existing test suite passes without any `--modbus` flag (regression guard: no existing test should break).

---

## HS-W34-004: --modbus + --no-reassemble Warning Path

**Scope:** STORY-105 (BC-2.14.023 postcondition P2 sub-case)
**Priority:** P1
**Wave:** 34

**Assertions:**
1. `wirerust analyze <any.pcap> --modbus --no-reassemble` exits 0.
2. stderr contains `"WARNING: --modbus requires stream reassembly"`.
3. No Modbus section in output (analyzer not constructed).
4. No crash.

---

## HS-W34-005: Threshold CLI Flags Parsed and Forwarded

**Scope:** STORY-105 (BC-2.14.024)
**Priority:** P1
**Wave:** 34

**Assertions:**
1. `--modbus-write-burst-threshold 5` with 6 writes within 1s → burst finding emitted (threshold honored).
2. `--modbus-write-sustained-threshold 3` with correct math → sustained finding emitted at lower threshold.
3. `--modbus-write-burst-threshold 0` → non-zero exit with error message `"must be >= 1"`.
4. `--modbus-write-sustained-threshold 0` → non-zero exit with error message `"must be >= 1"`.

---

## HS-W34-006: Regression on Existing Analyzers After Waves 32–34

**Scope:** All existing analyzers (HTTP, TLS, DNS, Reassembly, Reporters)
**Priority:** P0
**Wave:** 34

**Assertions:**
1. `cargo test --all-targets` exits 0 after all 4 Modbus stories are merged.
2. All VP assertions pass: VP-004 (classify-oracle, extended), VP-007 (catalog-drift-guard),
   VP-016 (mitre-tactic-grouping), VP-020 (csv-injection), VP-021 (timestamp-provenance),
   VP-022 (Modbus Kani proofs: parse_mbap no-panic, classify_fc total, pending-table bound).
3. No existing test function that was green pre-Wave-32 has turned red.
4. `cargo clippy --all-targets -- -D warnings` clean.
5. `cargo fmt --check` clean.

---

## Waves 32–34 Release Gate (v0.4.0)

Before creating the v0.4.0 release tag:

1. All HS-W32-001 through HS-W34-006 pass.
2. `cargo test --all-targets` exits 0 on a clean checkout.
3. `cargo clippy --all-targets -- -D warnings` clean.
4. `cargo fmt --check` clean.
5. VP-022 Kani proofs verified (requires nightly toolchain): `verify_parse_mbap_no_panic`,
   `verify_classify_fc_no_panic`, and the pending-table bound integration test pass.
6. VP-004 Kani oracle extension covers `DispatchTarget::Modbus` for port-502 flows.
7. `wirerust analyze --help` shows `--modbus`, `--modbus-write-burst-threshold`,
   `--modbus-write-sustained-threshold` flags with correct default values.
8. Verify v0.3.0 is already tagged (v0.4.0 must be a forward increment from v0.3.0).
