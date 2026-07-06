# Pattern Consistency Scan — Maintenance Sweep 3

**run_id:** maint-2026-07-06
**Date:** 2026-07-06
**Scope:** `src/` — all analyzer, reassembly, reporter, dispatcher, decoder modules
**Method:** Read-only static analysis. No code was modified.
**Prior sweep:** 2026-06-17 (12 findings, none resolved in this run)

---

## Findings vs Prior Sweep

| Finding | Status | Notes |
|---------|--------|-------|
| PC-001 through PC-012 | PERSISTS | All 12 prior findings are unresolved |
| PC-013 through PC-020 | NEW | 8 new findings identified in this run |

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 3     |
| MEDIUM   | 11    |
| LOW      | 6     |
| **Total**| **20**|

---

## PC-001 — DNP3 does not implement `StreamHandler` / `StreamAnalyzer` traits [PERSISTS]

**Severity:** HIGH
**Category:** architecture-alignment
**Files:** `src/analyzer/dnp3.rs`, `src/dispatcher.rs:408`
**Batch-refactor candidate:** yes

`Dnp3Analyzer` has a bespoke `on_data(flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)` that differs from the `StreamHandler::on_data` signature. DNP3 neither implements `StreamHandler` nor `StreamAnalyzer`. The dispatcher at line 408 calls `dnp3.on_data(flow_key.clone(), ...)` — the `.clone()` is a per-packet heap allocation. Direction is threaded as of STORY-140, but the ADR-007 / STORY-106 design left the trait gap open. The `on_flow_close` arm at dispatcher line 452 dispatches correctly but also takes `flow_key.clone()`.

Impact: DNP3 cannot be routed polymorphically via `StreamHandler`; the dispatcher carries a permanent special case; per-packet `FlowKey` clone persists.

---

## PC-002 — Inconsistent `findings` import style: fully-qualified paths / inline `use` vs. module-level `use` [PERSISTS]

**Severity:** HIGH
**Category:** pattern-consistency
**Files:** `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`, `src/analyzer/arp.rs`
**Batch-refactor candidate:** yes

Modbus and DNP3 use `crate::findings::ThreatCategory::Anomaly`, `crate::findings::Verdict::Likely`, etc. as fully-qualified paths throughout `process_pdu` / `on_data` (~40 occurrences in modbus, ~30 in dnp3). ARP uses repeated inline `use crate::findings::{Confidence, ThreatCategory, Verdict};` blocks at lines 462, 699, 841, 931, 1015. HTTP, TLS, and reassembly all import at module level with a single `use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};`.

---

## PC-003 — DNP3 missing `dropped_findings` counter [PERSISTS]

**Severity:** HIGH
**Category:** spec-fidelity
**Files:** `src/analyzer/dnp3.rs`
**Batch-refactor candidate:** yes

`Dnp3Analyzer` has `MAX_FINDINGS = 10_000` and cap guards throughout `on_data`, but has no `dropped_findings: u64` field and does not surface the count in `summarize()`. When a finding is suppressed past the cap it is silently discarded. `ModbusAnalyzer` (`modbus.rs:287`) and `EnipAnalyzer` (`enip.rs:646`) both carry `dropped_findings` and surface it in their summaries. `TcpReassembler` likewise tracks it. DNP3 is the sole analyzer missing this counter post-v0.11.4.

---

## PC-004 — `chrono::DateTime::from_timestamp` fully-qualified at every call site in DNP3 [PERSISTS]

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/dnp3.rs` (~11 occurrences)
**Batch-refactor candidate:** yes

DNP3 uses `chrono::DateTime::from_timestamp(now_ts as i64, 0)` fully-qualified at every finding emission site (lines 1014, 1067, 1115, 1197, 1324, 1381, 1460, 1534, 1594, 1631, 1699). Modbus imports `use chrono::DateTime;` inside `process_pdu` (line 354). HTTP, TLS, and reassembly all import `chrono::DateTime` at module level. Both the fully-qualified and the local-import patterns remain in the codebase.

---

## PC-005 — MAC address formatting: inline 6-field format string duplicated ~15 times in ARP [PERSISTS]

**Severity:** MEDIUM
**Category:** code-quality
**Files:** `src/analyzer/arp.rs` (~15 MAC occurrences, ~8 IPv4 occurrences)
**Batch-refactor candidate:** yes

The ARP analyzer formats MAC addresses using an expanded 6-field format string (e.g., line 497: `eth_mac[0], eth_mac[1], ..., eth_mac[5]`) and IPv4 addresses using a 4-field format string. No `fmt_mac` or `fmt_ipv4` helper exists. Any change to the evidence format must be applied in ~15 and ~8 places respectively.

---

## PC-006 — `analyzer_name` casing inconsistency: Modbus uses lowercase `"modbus"` [PERSISTS]

**Severity:** MEDIUM
**Category:** maintainability
**Files:** `src/analyzer/modbus.rs:994-995`, `src/analyzer/modbus.rs:1218`
**Batch-refactor candidate:** yes

`ModbusAnalyzer::summarize()` emits `analyzer_name: "modbus"` (lowercase) and `fn name()` returns `"modbus"`. All other analyzers use mixed-case or all-uppercase: `"HTTP"`, `"TLS"`, `"DNS"`, `"ARP"`, `"DNP3"`, `"EtherNet/IP"`. The comment at line 994 claims `"modbus"` follows the `"http"` and `"tls"` convention, but HTTP and TLS are actually `"HTTP"` and `"TLS"` — the comment is factually wrong. This is a breaking change for any downstream consumer filtering by `analyzer_name`.

---

## PC-007 — `BTreeMap` imported inside `summarize()` method body rather than at module level [PERSISTS]

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/arp.rs:752`, `src/analyzer/modbus.rs:952`, `src/analyzer/dnp3.rs:1716`
**Batch-refactor candidate:** yes

All three analyzers import `use std::collections::BTreeMap;` inside their `summarize()` method body. The DNS analyzer imports it at module level (line 17). Per-function inline `use` is noisier and inconsistent with the module-level convention used by the newer analyzers and DNS.

---

## PC-008 — Modbus `summarize()` dual-method pattern: inherent method + trait delegation [PERSISTS]

**Severity:** MEDIUM
**Category:** code-quality
**Files:** `src/analyzer/modbus.rs:951`, `src/analyzer/modbus.rs:1221-1223`
**Batch-refactor candidate:** yes

`ModbusAnalyzer` has a `pub fn summarize(&self)` inherent method (line 951) and a `StreamAnalyzer` trait impl that delegates to it (line 1221). The trait impl also uses the fully-qualified return type `crate::analyzer::AnalysisSummary` rather than the imported `AnalysisSummary` (imported at line 21). `HttpAnalyzer` and `TlsAnalyzer` implement `StreamAnalyzer::summarize()` directly without a separate inherent method.

---

## PC-009 — DNP3 `function_code_distribution` keys formatted as decimal strings vs. hex in Modbus [PERSISTS]

**Severity:** LOW
**Category:** maintainability
**Files:** `src/analyzer/dnp3.rs:1737`
**Batch-refactor candidate:** yes

`Dnp3Analyzer::summarize()` formats FC keys as decimal strings: `.map(|(&fc, &count)| (fc.to_string(), count))` (line 1737). Modbus uses `format!("0x{fc:02X}")` (line 985). Consumers correlating FC distributions across Modbus and DNP3 must parse two different formats. A decimal `"5"` for DNP3's FC=0x05 (DIRECT_OPERATE) is less readable than `"0x05"`.

---

## PC-010 — DNS analyzer lacks doc comments on `new()`, `analyze()`, and `Default` impl [PERSISTS]

**Severity:** LOW
**Category:** maintainability
**Files:** `src/analyzer/dns.rs:35`, `src/analyzer/dns.rs:28`, `src/analyzer/dns.rs:70`
**Batch-refactor candidate:** no

`DnsAnalyzer::new()` has no doc comment (line 35). `analyze()` returns `Vec::new()` with no inline doc explaining why (line 70). The `Default` impl (line 28) has no delegation rationale note. All other analyzers that implement `Default` and `new()` have doc comments explaining the delegation and construction rationale.

---

## PC-011 — HTTP and TLS serialize distribution maps via `HashMap` into `serde_json::json!()`, producing non-deterministic key order [PERSISTS]

**Severity:** LOW
**Category:** pattern-consistency
**Files:** `src/analyzer/http.rs:613`, `src/analyzer/tls.rs:1255`
**Batch-refactor candidate:** yes

Both files use `.collect::<HashMap<String, u64>>()` inside `serde_json::json!()` for their distribution maps. This yields a `serde_json::Value::Object` with non-deterministic key ordering because `HashMap` iteration order is undefined. This violates NFR DET-001 deterministic output for those summary fields. DNP3's `function_code_distribution` (line 1733) correctly collects into a `BTreeMap`.

---

## PC-012 — DNP3 module-level `#![allow(dead_code)]` blanket suppression [PERSISTS]

**Severity:** LOW
**Category:** code-quality
**Files:** `src/analyzer/dnp3.rs:26`
**Batch-refactor candidate:** yes

`dnp3.rs` opens with `#![allow(dead_code)]` at line 26, suppressing unused-code warnings for the entire module. Per-item `#[allow(unused)]` attributes exist on specific constants (lines 122, 129, 139, 145, 151, etc.) but the module-level blanket is also present. Future additions of dead constants, fields, or functions will compile silently without any compiler signal. `arp.rs` and `modbus.rs` use no module-level suppress.

---

## PC-013 — ENIP uses fully-qualified `crate::findings::ThreatCategory/Verdict/Confidence` paths throughout [NEW]

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/enip.rs` (~24 occurrences across `check_t0814`, `process_pdu`)
**Batch-refactor candidate:** yes

`EnipAnalyzer` imports `crate::findings::Finding` at module level (line 206) but uses fully-qualified paths for `ThreatCategory`, `Verdict`, and `Confidence` at every call site: `crate::findings::ThreatCategory::Anomaly`, `crate::findings::Verdict::Possible`, etc. (lines 475–477, 1073–1075, 1179–1181, 1224–1226, 1258–1260, 1293–1295, 1363–1365, 1451–1453). This is the same pattern as PC-002 in Modbus and DNP3. ENIP was written after the module-level import convention was established and should have followed it. Bundle fix with PC-002.

---

## PC-014 — ENIP uses `chrono::DateTime::from_timestamp` fully-qualified at every call site [NEW]

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/enip.rs` (~8 occurrences: lines 485, 1086, 1192, 1238, 1273, 1308, 1380, 1458)
**Batch-refactor candidate:** yes

Same pattern as PC-004 in DNP3: no module-level `use chrono::DateTime;` import; `chrono::DateTime::from_timestamp(...)` used fully-qualified at every finding emission site. ENIP was written after the module-level import convention was established by HTTP, TLS, and reassembly. Bundle fix with PC-004.

---

## PC-015 — ENIP imports `BTreeMap` inside `summarize()` method body [NEW]

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/enip.rs:1487`
**Batch-refactor candidate:** yes

`EnipAnalyzer::summarize()` at line 1487 opens with `use std::collections::BTreeMap;` inside the method body. Same pattern as PC-007 in ARP, Modbus, and DNP3. ENIP was written after the DNS module-level import convention was established. Bundle fix with PC-007.

---

## PC-016 — DNP3 `master_addrs_seen` silent ignore has no observable counter [NEW]

**Severity:** MEDIUM
**Category:** observability
**Files:** `src/analyzer/dnp3.rs:751-754`, `src/analyzer/dnp3.rs:146`
**Batch-refactor candidate:** yes

`Dnp3FlowState.master_addrs_seen: Vec<u16>` is capped at `MAX_MASTER_ADDRS = 64` (line 146). When the cap is reached, new master source addresses are silently ignored: `&& flow.master_addrs_seen.len() < MAX_MASTER_ADDRS` (line 752). No `master_addrs_dropped` or `dropped_map_entries` counter exists on `Dnp3Analyzer` or `Dnp3FlowState`, and no count is surfaced in `summarize()`.

The v0.11.4 observability counter pattern (ARP `bindings_evicted`/`storm_counters_evicted`, Modbus `dropped_transactions`, HTTP/TLS `dropped_map_entries`) mandates surfacing all silent resource-cap events. Detection correctness is affected: a full `master_addrs_seen` causes `unexpected_source_emitted` logic (line 812) to behave as if all future sources are "known", potentially silencing T1692.001 findings without any observable signal.

---

## PC-017 — DNP3 `pending_requests` LRU eviction has no observable counter [NEW]

**Severity:** MEDIUM
**Category:** observability
**Files:** `src/analyzer/dnp3.rs:1796-1814`, `src/analyzer/dnp3.rs:123`
**Batch-refactor candidate:** yes

`Dnp3FlowState.pending_requests: HashMap<(u16, u8), u32>` is capped at `MAX_PENDING_REQUESTS = 256` (line 123). The `insert_pending_request` helper at lines 1796–1814 evicts the oldest entry (by minimum timestamp) when the map is full: "the evicted entry is silently dropped — it generates NO T1691.001 finding" (line 1797). No `pending_requests_evicted` counter exists on `Dnp3Analyzer`, and no eviction count is surfaced in `summarize()`.

The v0.11.4 pattern (ARP `bindings_evicted`, Modbus `dropped_transactions`) requires surfacing such eviction events. Under high-rate DNP3 Control-class traffic, LRU pressure on `pending_requests` can silently degrade T1691.001 (block-command inference) accuracy without any observable signal.

---

## PC-018 — ENIP `command_distribution` serialized via `HashMap` iteration into `serde_json::Map`, non-deterministic key order [NEW]

**Severity:** LOW
**Category:** pattern-consistency
**Files:** `src/analyzer/enip.rs:1538-1542`
**Batch-refactor candidate:** yes

`EnipAnalyzer::summarize()` builds `cmd_dist: serde_json::Map<String, serde_json::Value>` by iterating `enip_summary_struct.command_distribution: HashMap<u16, u64>` (line 1539). Since `HashMap` iteration order is non-deterministic, the resulting `command_distribution` JSON object has non-deterministic key ordering. This violates NFR DET-001 for that field. The fix is to iterate a `BTreeMap` or collect to `BTreeMap` first (same fix as PC-011). Bundle fix with PC-011 and PC-019.

---

## PC-019 — Modbus `function_code_distribution` serialized via `HashMap` iteration into `serde_json::Map`, non-deterministic key order [NEW]

**Severity:** LOW
**Category:** pattern-consistency
**Files:** `src/analyzer/modbus.rs:982-990`
**Batch-refactor candidate:** yes

`ModbusAnalyzer::summarize()` builds `dist: serde_json::Map<String, serde_json::Value>` by iterating `self.fn_code_counts: HashMap<u8, u64>` (line 983). Since `HashMap` iteration order is non-deterministic, the `function_code_distribution` JSON object has non-deterministic key ordering. PC-011 noted the same issue in HTTP and TLS but missed Modbus. DNP3 (`dnp3.rs:1733`) correctly collects into a `BTreeMap<String, u64>` first. Bundle fix with PC-011 and PC-018.

---

## PC-020 — ENIP does not implement `StreamHandler` / `StreamAnalyzer` traits [NEW]

**Severity:** MEDIUM
**Category:** architecture-alignment
**Files:** `src/analyzer/enip.rs`, `src/dispatcher.rs:418,461`
**Batch-refactor candidate:** yes

`EnipAnalyzer` has a bespoke `on_data(flow_key: FlowKey, data: &[u8], timestamp: u32, direction: Direction)` that differs from `StreamHandler::on_data` (which takes `&FlowKey` and has a different parameter order). ENIP neither implements `StreamHandler` nor `StreamAnalyzer`. The dispatcher at line 418 calls `enip.on_data(flow_key.clone(), ...)` and at line 461 calls `enip.on_flow_close(flow_key.clone())` — both incur per-call `FlowKey` heap allocations.

Unlike PC-001 (DNP3), ENIP does correctly thread `direction` and correctly implements `on_flow_close` semantics. The gap is therefore narrower: it is primarily an architectural inconsistency and the per-packet clone. ENIP was written after the Modbus `StreamHandler` pattern was established (STORY-105) and should have followed it. The comment at dispatcher line 406 explicitly references the "Modbus/ENIP pattern" for direction threading, suggesting ENIP was modeled on DNP3 rather than Modbus at design time.

---

## Batch Refactoring Candidates (Prioritized)

| Priority | Findings | Description | Effort |
|----------|----------|-------------|--------|
| 1 (HIGH)  | PC-001 | DNP3 `StreamHandler`/`StreamAnalyzer` conformance | 2–4 days |
| 2 (HIGH)  | PC-002, PC-013 | Migrate inline `findings` imports to module level (modbus, dnp3, arp, enip) | < 1 day |
| 3 (HIGH)  | PC-003 | Add `dropped_findings` counter to DNP3 | < 1 day |
| 4 (MEDIUM)| PC-016, PC-017 | Add DNP3 observability counters: `master_addrs_dropped`, `pending_requests_evicted` | < 1 day |
| 5 (MEDIUM)| PC-020 | ENIP `StreamHandler`/`StreamAnalyzer` conformance (narrower gap than PC-001) | 1–2 days |
| 6 (MEDIUM)| PC-004, PC-014 | Normalize `chrono::DateTime` import in DNP3 and ENIP | < 1 day |
| 7 (MEDIUM)| PC-005 | Extract `fmt_mac` / `fmt_ipv4` helpers in ARP | < 1 day |
| 8 (MEDIUM)| PC-006 | Fix `analyzer_name` casing (`"modbus"` → `"Modbus"`) | trivial + snapshot test |
| 9 (MEDIUM)| PC-007, PC-015 | Move `BTreeMap` imports to module level (arp, modbus, dnp3, enip) | < 30 min |
| 10 (MEDIUM)| PC-008 | Resolve Modbus `summarize()` dual-method pattern | < 1 day |
| 11 (LOW)  | PC-009 | Align DNP3 FC key format to hex (`"0x{fc:02X}"`) | < 1 hour |
| 12 (LOW)  | PC-011, PC-018, PC-019 | Fix non-deterministic distribution map key order (http, tls, enip, modbus) | < 1 hour |
| 13 (LOW)  | PC-012 | Remove module-level `#![allow(dead_code)]` from DNP3 | < 1 day |
| 14 (LOW)  | PC-010 | Add missing doc comments to DNS analyzer | < 30 min (no batch) |
