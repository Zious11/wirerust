# Pattern Consistency Scan — Maintenance Sweep 3

**Date:** 2026-06-17
**Scope:** `src/` — all analyzer, reassembly, reporter, dispatcher, decoder modules
**Method:** Read-only static analysis. No code was modified.

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 3     |
| MEDIUM   | 5     |
| LOW      | 4     |
| **Total**| **12**|

---

## PC-001 — DNP3 does not implement `StreamHandler` / `StreamAnalyzer` traits

**Severity:** HIGH
**Category:** architecture-alignment
**Files:** `src/analyzer/dnp3.rs`, `src/dispatcher.rs`

### Legacy pattern (DNP3)
`Dnp3Analyzer` has a bespoke `pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32)` method with a **different signature** from `StreamHandler::on_data` (which takes `&FlowKey`, `Direction`, `&[u8]`, `u64`, `u32`). DNP3 neither implements `StreamHandler` nor `StreamAnalyzer`.

The dispatcher line 347 even documents this explicitly:
```
// Dnp3Analyzer does not implement StreamHandler; no forwarding needed.
```
The `on_flow_close` arm for DNP3 is a silent no-op (line 348: `let _ = reason;`).

### New pattern (Modbus, HTTP, TLS)
`ModbusAnalyzer`, `HttpAnalyzer`, `TlsAnalyzer` all implement `StreamHandler` + `StreamAnalyzer`, route through the trait interface, and receive `on_flow_close` calls.

### Impact
- DNP3 flows are never notified of flow closure — any close-triggered detection logic that requires a flush step cannot be wired without refactoring.
- `Direction` is silently dropped on all DNP3 data (noted in `dnp3.rs:1454` as "DRIFT-DNP3-DIRECTION-001").
- The dispatcher comment at line 315 calls `dnp3.on_data(flow_key.clone(), data, timestamp)` — the `flow_key` is **cloned** (heap allocation per packet) because the signature takes `FlowKey` by value rather than `&FlowKey`. All other analyzers take `&FlowKey`.

### Refactor effort: MEDIUM (2–4 days)
Implement `StreamHandler` / `StreamAnalyzer` for `Dnp3Analyzer`, migrate the internal `on_data` signature, thread `Direction` through, and wire `on_flow_close`.

---

## PC-002 — Inconsistent `findings` import style: module-level `use` vs. inline `use` inside method bodies

**Severity:** HIGH
**Category:** pattern-consistency
**Files:** `src/analyzer/arp.rs`, `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs`

### Legacy pattern (ARP, Modbus, DNP3)
All three newer ICS analyzers import `crate::findings` items **inside method bodies** with repeated inline `use` statements:

```rust
// arp.rs — repeated at lines 442, 670, 804, 894, 977
use crate::findings::{Confidence, ThreatCategory, Verdict};

// modbus.rs — uses fully-qualified paths throughout process_pdu (~40 occurrences):
category: crate::findings::ThreatCategory::Anomaly,
verdict: crate::findings::Verdict::Inconclusive,
confidence: crate::findings::Confidence::Medium,

// dnp3.rs — same fully-qualified path pattern (~20 occurrences)
category: crate::findings::ThreatCategory::Execution,
```

### New pattern (HTTP, TLS, reassembly)
`src/analyzer/http.rs`, `src/analyzer/tls.rs`, `src/reassembly/mod.rs`, `src/reassembly/lifecycle.rs` all import findings types at the **module level**:

```rust
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
```

### Impact
- `modbus.rs` and `dnp3.rs` have no module-level `findings` import beyond `Finding` itself, causing ~60 verbose `crate::findings::ThreatCategory::X` paths. This is noise in code review and inconsistent with the established pattern.
- ARP duplicates the inline `use` five times.

### Refactor effort: LOW (< 1 day)
Add module-level `use crate::findings::{Confidence, ThreatCategory, Verdict};` to `modbus.rs`, `dnp3.rs`, `arp.rs`; replace all inline `use` and fully-qualified paths.

---

## PC-003 — DNP3 missing `dropped_findings` counter

**Severity:** HIGH
**Category:** spec-fidelity
**Files:** `src/analyzer/dnp3.rs`

### Legacy pattern (DNP3)
`Dnp3Analyzer` has `MAX_FINDINGS = 10_000` and a cap guard in detection methods, but has **no `dropped_findings` counter** and does not surface dropped-finding counts in `summarize()`. When a finding is suppressed past the cap, it is silently discarded with no observable trace.

### New pattern (Modbus, reassembly)
`ModbusAnalyzer` (`modbus.rs:284`, `modbus.rs:953`):
```rust
pub dropped_findings: u64,
// ...
detail.insert("dropped_findings".to_string(), serde_json::json!(self.dropped_findings));
```
`TcpReassembler` likewise increments `stats.dropped_findings` and includes it in `summarize()`.

### Impact
Operators and downstream tooling cannot distinguish "no threats found" from "threats found but silently dropped past the cap" for DNP3 streams. This is an observability gap relative to the explicitly stated contract for Modbus (BC-2.14.022) and reassembly (BC-2.04.024). DNP3's own BC-2.15.022 notes the `MAX_FINDINGS` cap but does not address observability.

### Refactor effort: LOW (< 1 day)
Add `dropped_findings: u64` field to `Dnp3Analyzer`, increment it in the cap guard arms, surface in `summarize()`.

---

## PC-004 — `chrono::DateTime::from_timestamp` call-site import inconsistency

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/dnp3.rs`, `src/analyzer/modbus.rs`

### Legacy pattern
`dnp3.rs` uses the fully-qualified path at every call site (~14 occurrences):
```rust
timestamp: chrono::DateTime::from_timestamp(now_ts as i64, 0),
```
`modbus.rs` imports `DateTime` with an inline `use` inside `process_pdu` only:
```rust
use chrono::DateTime;
let finding_ts = DateTime::from_timestamp(timestamp as i64, 0);
```

### New pattern (HTTP, TLS, reassembly)
`http.rs`, `tls.rs`, `reassembly/mod.rs`, `reassembly/lifecycle.rs` all import `chrono::DateTime` at the **module level** and use the short form at call sites.

### Impact
Verbose repeated paths in dnp3.rs obscure the call sites; the inconsistency across files makes grep/refactor operations harder. No semantic difference.

### Refactor effort: LOW (< 1 day)
Add `use chrono::DateTime;` at module-level in `dnp3.rs`; replace all `chrono::DateTime::from_timestamp(...)` with `DateTime::from_timestamp(...)`.

---

## PC-005 — MAC address formatting: inline 6-field format string duplicated ~15 times

**Severity:** MEDIUM
**Category:** code-quality
**Files:** `src/analyzer/arp.rs`

### Pattern
The ARP analyzer formats MAC addresses using an expanded 6-field format string. There is no shared helper function. The pattern appears ~15 times across `process_arp`, `emit_d1_spoof_finding_impl`, `apply_garp_conflict_escalation_impl`, and `detect_storm`:

```rust
format!(
    "eth_src_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
    eth_mac[0], eth_mac[1], eth_mac[2], eth_mac[3], eth_mac[4], eth_mac[5]
),
format!(
    "arp_sender_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
    frame.sender_mac[0], ..., frame.sender_mac[5]
),
```

Similarly, IPv4 addresses are formatted inline with a 4-field pattern that appears ~8 times in the same file:
```rust
format!("sender_ip={}.{}.{}.{}", sender_ip[0], sender_ip[1], sender_ip[2], sender_ip[3])
```

### Comparable pattern elsewhere
The reassembly, Modbus, and DNP3 modules use `IpAddr` (already implements `Display`) and never need the split-indexing form for IP addresses. ARP works with raw `[u8; 4]` / `[u8; 6]` arrays; there is no standard `Display` for these.

### Impact
Any change to the MAC/IP evidence format must be applied in ~15/8 places. A typo in one instance would produce inconsistent evidence strings. A one-line helper (`fn fmt_mac(m: &[u8; 6]) -> String`) would eliminate the duplication.

### Refactor effort: LOW (< 1 day)
Extract `fn fmt_mac(m: &[u8; 6]) -> String` and `fn fmt_ipv4(ip: &[u8; 4]) -> String` as module-level free functions in `arp.rs`; replace all instances.

---

## PC-006 — `analyzer_name` casing inconsistency in `summarize()`

**Severity:** MEDIUM
**Category:** maintainability
**Files:** `src/analyzer/modbus.rs:971`, `src/analyzer/dnp3.rs:1430`

### Pattern
`analyzer_name` values across analyzers are inconsistently cased:
- `"HTTP"` — from `fn name()` in StreamAnalyzer, used via `self.name().to_string()`
- `"TLS"` — same pattern
- `"DNS"` — DnsAnalyzer `fn name()` returns `"DNS"`
- `"TCP Reassembly"` — reassembly `summarize()`
- `"ARP"` — arp `summarize()`
- `"DNP3"` — dnp3 `summarize()`
- `"modbus"` — **lowercase** in `ModbusAnalyzer::summarize()` and `fn name()`

The Modbus analyzer is the outlier: `analyzer_name: "modbus".to_string()` (line 971) with `fn name() -> "modbus"` (line 1162). All other analyzers use mixed-case or all-uppercase. The comment at line 970 says `// BC-2.14.021 post.3: lowercase "modbus" matches "http" and "tls" analyzer convention` — but HTTP and TLS are actually uppercase (`"HTTP"`, `"TLS"`), so this comment is factually incorrect.

### Impact
JSON consumers comparing `analyzers[].analyzer_name` must handle a mixed-case set. The incorrect comment may mislead future contributors about the convention.

### Refactor effort: LOW (trivial string change + test update)
Change `"modbus"` → `"Modbus"` (or `"Modbus TCP"` for clarity) in `ModbusAnalyzer::summarize()` and `fn name()`. Update the BC-2.14.021 snapshot test. Correct or remove the misleading comment.

**Note:** This is a breaking change for any downstream consumer filtering by `analyzer_name`. Needs a version note.

---

## PC-007 — `BTreeMap` in `summarize()` imported locally vs. module-level

**Severity:** MEDIUM
**Category:** pattern-consistency
**Files:** `src/analyzer/arp.rs:723`, `src/analyzer/modbus.rs:932`, `src/analyzer/dnp3.rs:1381`

### Pattern
ARP, Modbus, and DNP3 all import `std::collections::BTreeMap` inside their `summarize()` method body:
```rust
pub fn summarize(&self) -> AnalysisSummary {
    use std::collections::BTreeMap;
    // ...
}
```

DNS, on the other hand, imports it at module level:
```rust
use std::collections::BTreeMap;
```

HTTP and TLS use `std::collections::BTreeMap` fully qualified inline. The reassembly `summarize()` uses `std::collections::BTreeMap::new()` inline.

### Impact
Minor inconsistency; no semantic difference. The module-level pattern (DNS) is cleanest and matches the general project style. The inline `use` per-function pattern is more verbose but acceptable. The mixed `std::collections::BTreeMap` and `use BTreeMap` pattern within the same codebase is the friction point.

### Refactor effort: LOW (< 30 minutes)
Standardize on module-level `use std::collections::BTreeMap` in the three files that currently import it locally.

---

## PC-008 — `StreamAnalyzer::summarize()` delegation pattern in Modbus

**Severity:** MEDIUM
**Category:** code-quality
**Files:** `src/analyzer/modbus.rs:1165-1168`

### Pattern
`ModbusAnalyzer` has a `pub fn summarize(&self)` inherent method (line 931) **and** a `StreamAnalyzer` trait impl that delegates to it (line 1165):

```rust
impl StreamAnalyzer for ModbusAnalyzer {
    fn summarize(&self) -> crate::analyzer::AnalysisSummary {
        // Delegate to the inherent method (same logic).
        ModbusAnalyzer::summarize(self)
    }
}
```

The trait `summarize` also uses a fully-qualified return type `crate::analyzer::AnalysisSummary` rather than the imported `AnalysisSummary`, inconsistent with the module's top-level import at line 22 (`use crate::analyzer::AnalysisSummary`).

### Comparable pattern
`HttpAnalyzer` and `TlsAnalyzer` implement `StreamAnalyzer::summarize()` directly with the logic inline — there is no separate inherent `summarize` method.

### Impact
The two-method pattern (inherent + trait delegation) is unusual and slightly confusing. Having both means callers can call `analyzer.summarize()` without going through the trait (potentially bypassing the trait's `AnalysisSummary` return type annotation). The use of `crate::analyzer::AnalysisSummary` in the trait impl but `AnalysisSummary` everywhere else is inconsistent.

### Refactor effort: LOW (< 1 day)
Either (a) remove the inherent `summarize()` and move the logic into the `StreamAnalyzer` impl, or (b) keep the inherent method and use the already-imported short name `AnalysisSummary` in the trait impl signature.

---

## PC-009 — DNP3 `function_code_distribution` key format diverges from Modbus

**Severity:** LOW
**Category:** maintainability
**Files:** `src/analyzer/dnp3.rs:1393`, `src/analyzer/modbus.rs:961`

### Pattern
`ModbusAnalyzer::summarize()` formats FC keys as hex strings:
```rust
dist.insert(format!("0x{fc:02X}"), serde_json::json!(count));
```

`Dnp3Analyzer::summarize()` formats FC keys as decimal strings:
```rust
.map(|(&fc, &count)| (fc.to_string(), count))
// → "5" for 0x05, "3" for 0x03
```

### Impact
JSON consumers that need to correlate FC distributions across Modbus and DNP3 must parse two different formats. A decimal `"5"` for DNP3's FC=0x05 (DIRECT_OPERATE) is less readable than `"0x05"`. This is a low-stakes documentation/usability issue but will become a bigger pain once a unified report view is built.

### Refactor effort: LOW (< 1 hour)
Change the DNP3 key format to `format!("0x{fc:02X}")` to match Modbus.

---

## PC-010 — DNS analyzer lacks module-level doc comment / `ProtocolAnalyzer` impl note

**Severity:** LOW
**Category:** maintainability
**Files:** `src/analyzer/dns.rs`

### Pattern
`DnsAnalyzer` is the simplest analyzer (97 lines) and implements `ProtocolAnalyzer`. Its file-level doc comment documents behavior, but:
- There is no `Default` impl comment explaining the delegation rationale (the other analyzers that implement `Default` have these).
- The `analyze()` method just returns `Vec::new()` with no doc comment explaining why (only the module-level comment covers this).
- Unlike all other analyzers that have a `pub fn new()` with a doc comment, `DnsAnalyzer::new()` has none.

### Impact
Low: DNS is simple enough that this isn't confusing, but it diverges from the project's convention of documenting every `new()` and `analyze()`.

### Refactor effort: LOW (< 30 minutes)

---

## PC-011 — HTTP `summarize()` uses `HashMap<String, u64>` inside `serde_json::json!()` for `status_codes` and `tls_versions`

**Severity:** LOW
**Category:** pattern-consistency
**Files:** `src/analyzer/http.rs:594-601`, `src/analyzer/tls.rs:873-878`

### Pattern
Both HTTP and TLS serialize a `HashMap<u16, u64>` (or `HashMap<u16, u64>`) by converting to `HashMap<String, u64>` inside the `serde_json::json!()` macro:

```rust
// http.rs
serde_json::json!(
    self.status_codes
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect::<HashMap<String, u64>>()
)
// tls.rs — same pattern for version_counts
```

This produces JSON objects with **non-deterministic key order** (HashMap iteration order is undefined). The module-level comment (`LESSON-P2.09`) explains that `BTreeMap` was adopted for deterministic JSON output, but these two map conversions still use `HashMap`.

### Impact
`status_codes` and `tls_versions` keys in `analyzer_summaries` will have non-deterministic order in JSON output, violating the NFR DET-001 determinism goal for those fields. This is inconsistent with the `BTreeMap` top-level `detail` map.

### Refactor effort: LOW (< 1 hour)
Change `.collect::<HashMap<String, u64>>()` to `.collect::<std::collections::BTreeMap<String, u64>>()` in both files.

---

## PC-012 — `#[allow(dead_code)]` module-level in DNP3 masks unused items silently

**Severity:** LOW
**Category:** code-quality
**Files:** `src/analyzer/dnp3.rs:26`

### Pattern
`dnp3.rs` opens with `#![allow(dead_code)]` at line 26, suppressing all unused-code warnings for the entire module. Many constants and fields are gated with per-item `#[allow(unused)]`, but the module-level suppress is also present.

In contrast, `arp.rs` and `modbus.rs` use **per-item** `#[allow(unused)]` only where specifically justified (e.g., `modbus.rs` has none; `arp.rs` has none; `dnp3.rs` uses `#[allow(unused)]` on specific constants at lines 101, 107, 117, 123 etc).

### Impact
The `#![allow(dead_code)]` blanket prevents the compiler from flagging any dead code introduced in future changes to this module. It is a maintenance liability: added constants, fields, or functions that are never used will compile silently.

### Refactor effort: LOW (< 1 day)
Remove the module-level `#![allow(dead_code)]` and convert to per-item `#[allow(unused)]` or `#[allow(dead_code)]` where the items are legitimately staged/deferred.

---

## Batch Refactoring Candidates (Prioritized)

| Priority | Findings | Description | Effort |
|----------|----------|-------------|--------|
| 1 (HIGH)  | PC-001 | DNP3 `StreamHandler`/`StreamAnalyzer` conformance | 2–4 days |
| 2 (HIGH)  | PC-002 | Migrate inline `findings` imports to module level | < 1 day |
| 3 (HIGH)  | PC-003 | Add `dropped_findings` counter to DNP3 | < 1 day |
| 4 (MEDIUM)| PC-004 | Normalize `chrono::DateTime` import in DNP3 | < 1 day |
| 5 (MEDIUM)| PC-005 | Extract `fmt_mac` / `fmt_ipv4` helpers in ARP | < 1 day |
| 6 (MEDIUM)| PC-006 | Fix `analyzer_name` casing (`"modbus"` → `"Modbus"`) | trivial + snapshot test |
| 7 (MEDIUM)| PC-007 | Move `BTreeMap` imports to module level | < 30 min |
| 8 (MEDIUM)| PC-008 | Resolve Modbus `summarize()` dual-method pattern | < 1 day |
| 9 (LOW)   | PC-009 | Align DNP3 FC key format to Modbus (`"0x{fc:02X}"`) | < 1 hour |
| 10 (LOW)  | PC-011 | Fix non-deterministic `status_codes`/`tls_versions` in summaries | < 1 hour |
| 11 (LOW)  | PC-012 | Remove module-level `#![allow(dead_code)]` from DNP3 | < 1 day |
| 12 (LOW)  | PC-010 | Add missing doc comments to DNS analyzer | < 30 min |

PC-001 (DNP3 trait conformance) is the highest-value refactor: it closes an architecture gap, enables direction-aware source resolution, enables on_flow_close hooks, and eliminates the per-packet `flow_key.clone()` allocation. PC-002, PC-003, and PC-004 are low-effort improvements with immediate code-quality benefit and should be bundled into a single chore PR.
