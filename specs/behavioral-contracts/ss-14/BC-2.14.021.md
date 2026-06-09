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

# BC-2.14.021: summarize() Returns AnalysisSummary with Specified Per-Analyzer Summary Keys

## Description

`ModbusAnalyzer::summarize()` implements the `StreamAnalyzer` trait method and returns an
`AnalysisSummary` value containing aggregate statistics across all flows processed during
the capture. The summary is consumed by the terminal reporter, JSON reporter, and CSV
reporter — all three reporter surfaces use the same `AnalysisSummary` struct, which carries
a `detail: HashMap<String, Value>` map of string-keyed detail entries. This BC specifies the
exact key names, value types, and semantics for every key produced by the Modbus analyzer,
ensuring consistency with how HTTP and TLS analyzer summaries are structured (same
`AnalysisSummary` type, same key-value contract pattern). The summary is produced only once,
called by `main.rs` after `dispatcher.take_modbus_analyzer()` returns the finalized analyzer.

## Preconditions

1. `ModbusAnalyzer::summarize()` is called after all `on_data` and `on_flow_close` calls have
   completed (i.e., at post-pcap-processing finalization time).
2. The `ModbusAnalyzer` struct fields are in their final state: `total_pdu_count`,
   `total_write_count`, `total_exception_count`, `fn_code_counts`, `parse_errors` are
   the accumulated values across the entire capture.

## Postconditions

1. The returned `AnalysisSummary` has the following `detail` map entries (SIX keys — the
   complete and authoritative set for v1; none may be omitted):

   | Key | Value Type | Semantics |
   |-----|-----------|-----------|
   | `"pdu_count"` | `Value::Number(u64)` | Total PDUs processed across all flows (valid ADUs past the three-point gate) |
   | `"write_count"` | `Value::Number(u64)` | Total write-class FC PDUs across all flows |
   | `"exception_count"` | `Value::Number(u64)` | Total exception-response PDUs (FC >= 0x80) across all flows |
   | `"parse_errors"` | `Value::Number(u64)` | Total ADUs that failed the three-point validity gate or were malformed |
   | `"function_code_distribution"` | `Value::Object(HashMap<String, Value::Number(u64)>)` | FC → count map, hex-string keys (see Invariant 3) |
   | `"dropped_findings"` | `Value::Number(u64)` | Findings silently dropped due to MAX_FINDINGS cap (ALWAYS present, even when 0) |

2. The `function_code_distribution` object contains ONLY FC bytes for which `count > 0`.
   An FC that was never observed is absent from the map (no zero-count entries).

3. The `AnalysisSummary` struct fields (beyond `detail`) are set as follows:
   - `analyzer_name: "modbus"` — lowercase, consistent with `"http"` and `"tls"` names.
   - `findings_count: self.all_findings.len() as u64`
   - `flows_analyzed: self.total_flows_analyzed` — the monotonic counter incremented once
     per flow on first PDU insertion (NOT derived from `self.flows.len()`; `on_flow_close`
     removes entries from `self.flows` so `flows.len()` → 0 after all flows close; per
     architecture-delta.md §2.7 Decision 4, `total_flows_analyzed` is the correct source).
   - `protocol: "Modbus/TCP"` — human-readable protocol name for display.

4. `summarize()` does NOT consume `self` (takes `&self`); the analyzer is still usable
   afterward (though in practice `take_modbus_analyzer()` moves it out of the dispatcher
   immediately before `summarize()` is called, so this is a type-system constraint only).

## Invariants

1. **Key name exactness** (authoritative for all downstream consumers):
   The six key names above are the complete and authoritative set of Modbus summary keys for
   v1. These six keys must not be omitted. Terminal reporter, JSON reporter, and CSV reporter
   MUST use these exact string keys. `"dropped_findings"` MUST always be present (value 0
   when the MAX_FINDINGS cap was never reached). Any additional future keys must be added
   via a new BC revision.

2. **Zero-value suppression**: `function_code_distribution` omits FCs with count == 0.
   Implementation: build the map from `self.fn_code_counts.iter().filter(|(_, &v)| v > 0)`.
   This mirrors the HTTP analyzer's URL-pattern summary (which omits zero-count patterns)
   and the TLS cipher summary (which only lists observed ciphers).

3. **Hex-string key format for `function_code_distribution`**: each key is formatted as
   `format!("0x{:02X}", fc_byte)` — uppercase hex, zero-padded to 2 digits, with `0x` prefix.
   Examples: FC 0x03 → key `"0x03"`, FC 0x10 → key `"0x10"`. This format is consistent
   with the evidence string format used in individual BC finding emissions.

4. **Counter semantics**:
   - `pdu_count` counts ONLY valid PDUs (those that passed the three-point gate and were
     dispatched to the FC classification path). ADUs that failed the gate are counted in
     `parse_errors` only.
   - `write_count` counts write-class FCs in request direction only (not echoed responses).
     Exception responses on write FCs are counted in `exception_count`, not `write_count`.
   - `exception_count` counts all exception-response PDUs (FC >= 0x80, direction ServerToClient),
     regardless of whether they triggered an Anomaly finding.

5. **Summary is independent of MAX_FINDINGS cap**: `pdu_count`, `write_count`, and
   `exception_count` are incremented for every valid PDU regardless of whether the findings
   cap prevented a finding from being pushed. The cap only limits `findings_count`.

6. **JSON output format** for `function_code_distribution`:
   ```json
   "function_code_distribution": {
     "0x01": 42,
     "0x03": 1200,
     "0x06": 3,
     "0x10": 15
   }
   ```
   Only observed FCs appear. The map may be empty (`{}`) if no valid PDUs were processed.

7. **Terminal reporter display**: the terminal reporter renders summary keys as a
   labeled table or structured block, using the same format as HTTP/TLS summaries.
   The `function_code_distribution` map is rendered as a two-column sub-table:
   `FC code | count`. The exact rendering is the terminal reporter's concern (SS-11);
   this BC only mandates the data shape.

8. **CSV reporter**: the `function_code_distribution` object serializes as a JSON blob
   within the CSV cell (consistent with how the HTTP URL-distribution is serialized in
   CSV mode). Flat expansion of the FC map into CSV columns is not required in v1.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Capture with zero valid Modbus PDUs (all parse errors) | `pdu_count=0`, `write_count=0`, `exception_count=0`, `parse_errors=N`, `function_code_distribution={}` |
| EC-002 | Capture with 1000 read polls (FC=0x03 only) | `pdu_count=1000`, `write_count=0`, `exception_count=0`, `function_code_distribution={"0x03": 1000}` |
| EC-003 | Capture with mixed FCs: 500 reads (0x03), 10 writes (0x06), 3 exceptions (0x86) | `pdu_count=513`, `write_count=10`, `exception_count=3`, `function_code_distribution={"0x03": 500, "0x06": 10, "0x86": 3}` |
| EC-004 | All 256 FC byte values observed at least once | `function_code_distribution` has 256 entries. No zero-count suppression needed since all are > 0. |
| EC-005 | `MAX_FINDINGS` cap was hit mid-capture | `findings_count = 10_000`; counters (`pdu_count`, etc.) reflect actual totals, not capped values. `pdu_count` may be > 10_000. |

## Canonical Test Vectors

| Setup | Expected summarize() output | Category |
|-------|----------------------------|----------|
| 5 valid ADUs processed: 3x FC=0x03 (reads), 2x FC=0x06 (writes); 0 exceptions; 1 parse error; cap not hit | `{pdu_count: 5, write_count: 2, exception_count: 0, parse_errors: 1, function_code_distribution: {"0x03": 3, "0x06": 2}, dropped_findings: 0}` | happy-path |
| 0 valid ADUs; 3 parse errors; no findings dropped | `{pdu_count: 0, write_count: 0, exception_count: 0, parse_errors: 3, function_code_distribution: {}, dropped_findings: 0}` | edge-case (all invalid) |
| 1000 FC=0x03 reads; no writes; no exceptions; no cap hit | `pdu_count: 1000, function_code_distribution: {"0x03": 1000}, dropped_findings: 0` (dropped_findings always present) | happy-path (read-only) |
| FC=0x86 exception: 5 occurrences; MAX_FINDINGS cap hit (7 findings dropped) | `exception_count: 5`, `dropped_findings: 7`, `function_code_distribution: {"0x86": 5}` | happy-path (dropped_findings non-zero) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | (indirect) pdu_count and fn_code_counts are populated by on_data which relies on classify_fc | Kani (sub-property B via classify_fc) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC specifies the per-analyzer summary output that surfaces aggregate ICS protocol statistics to all reporter surfaces, completing the analysis capability's reporting path |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; `summarize()` method) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | N/A (summary statistics, not a finding) |

## Related BCs

- BC-2.14.013 through BC-2.14.020 — all feeding (write_count, exception_count, fn_code_counts increments per PDU path)
- BC-2.14.022 — related to (MAX_FINDINGS cap affects findings_count but not pdu_count/write_count)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusAnalyzer::summarize()` method (implements `StreamAnalyzer` trait)
- `src/analyzer/mod.rs` — `AnalysisSummary` struct definition; `detail: HashMap<String, Value>`
- `src/main.rs` — post-finalize block: `analyzer_summaries.push(modbus.summarize())`
- `src/reporting/terminal.rs` — summary rendering (SS-11 consumer)
- `src/reporting/json.rs` — JSON serialization (SS-11 consumer)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani (indirect via classify_fc correctness feeding fn_code_counts)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.7 (summarize() output: exact key names and value types); architecture-delta.md §2.2 (ModbusAnalyzer struct fields: fn_code_counts, total_write_count, total_exception_count, total_pdu_count, parse_errors) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (reads self state only) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (AnalysisSummary is owned) |
| **Overall classification** | effectful shell (reads mutable self fields) |
