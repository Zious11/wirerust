---
document_type: maintenance-sweep-output
sweep: pattern-consistency
sweep_id: maint-2026-06-22 / Sweep-3
producer: consistency-validator
timestamp: 2026-06-22T00:00:00Z
git_head: dd3b069
---

# Pattern Consistency Findings — Maintenance Sweep 3

**Date:** 2026-06-22
**HEAD:** dd3b069 (develop)
**Scope:** `src/` — all analyzer, reassembly, reader, reporter, dispatcher, decoder modules
**Method:** Read-only static analysis. No code was modified.
**Prior sweep:** `maint-2026-06-17` (pattern-consistency.md, 12 findings PC-001..PC-012)

---

## Summary

| Severity | Count | New | Already-Tracked |
|----------|-------|-----|-----------------|
| HIGH     | 3     | 0   | 3               |
| MEDIUM   | 6     | 1   | 5               |
| LOW      | 6     | 2   | 4               |
| **Total**| **15**| **3**| **12**         |

3 NEW findings (PC-013, PC-014, PC-015) identified in this sweep.
All 12 prior findings (PC-001..PC-012) remain unresolved and are re-confirmed below.

---

## Finding Table

| ID | Location | Pattern Issue | Severity | Auto-Fixable | Tech-Debt XRef |
|----|----------|---------------|----------|--------------|----------------|
| PC-001 | `src/analyzer/dnp3.rs`, `src/dispatcher.rs:315` | DNP3 does not implement `StreamHandler`/`StreamAnalyzer`; bespoke `on_data` signature; `flow_key.clone()` per packet; no `on_flow_close` forwarding | HIGH | No | TD-MAINT-PC001-DNP3-STREAMTRAIT |
| PC-002 | `src/analyzer/arp.rs`, `src/analyzer/modbus.rs`, `src/analyzer/dnp3.rs` | Inline / fully-qualified `crate::findings::` imports instead of module-level `use` | HIGH | Yes | — |
| PC-003 | `src/analyzer/dnp3.rs` | Missing `dropped_findings` counter; findings silently dropped past `MAX_FINDINGS` cap with no observable trace | HIGH | No | TD-MAINT-PC003-DNP3-DROPPED-COUNTER |
| PC-004 | `src/analyzer/dnp3.rs` | `chrono::DateTime` used via 11 fully-qualified paths; all other analyzers import at module level | MEDIUM | Yes | — |
| PC-005 | `src/analyzer/arp.rs` | MAC/IPv4 format strings duplicated ~15/8 times; no shared `fmt_mac`/`fmt_ipv4` helpers | MEDIUM | Yes | — |
| PC-006 | `src/analyzer/modbus.rs:969,1160` | `analyzer_name: "modbus"` (lowercase) while ARP/DNP3/HTTP/TLS all emit uppercase; misleading comment at line 968 | MEDIUM | Yes (+ snapshot test) | TD-MAINT-PC006-MODBUS-NAME-CASING |
| PC-007 | `src/analyzer/arp.rs:723`, `src/analyzer/modbus.rs:930`, `src/analyzer/dnp3.rs:1381` | `use std::collections::BTreeMap` inside `summarize()` method body; DNS imports at module level | MEDIUM | Yes | — |
| PC-008 | `src/analyzer/modbus.rs:929,1163` | Dual `summarize()` (inherent + `StreamAnalyzer` delegation); trait impl uses fully-qualified `crate::analyzer::AnalysisSummary` vs imported short name used elsewhere | MEDIUM | No | — |
| PC-009 | `src/analyzer/dnp3.rs:1393` | `function_code_distribution` keys use decimal strings (`"5"`) while Modbus uses hex (`"0x05"`) | LOW | Yes | — |
| PC-010 | `src/analyzer/dns.rs` | `pub fn new()` and `fn analyze()` lack doc comments present on all other analyzers | LOW | Yes | — |
| PC-011 | `src/analyzer/http.rs:600`, `src/analyzer/tls.rs:876` | `status_codes` and `tls_versions` serialized via `HashMap<String, u64>` producing non-deterministic key order; violates NFR DET-001 | LOW | Yes | — |
| PC-012 | `src/analyzer/dnp3.rs:26` | Module-level `#![allow(dead_code)]` blanket suppresses future dead-code warnings for entire module | LOW | No | — |
| PC-013 | `src/analyzer/arp.rs:555,576,642,827` | Production code calls `.expect()` on HashMap lookups with invariant-enforcing messages; pattern not mirrored in Modbus/DNP3 equivalents and not tracked in register | MEDIUM | No | NEW (see below) |
| PC-014 | `src/analyzer/dnp3.rs:1425` | `"total_parse_errors"` key name diverges from `"parse_errors"` used by HTTP, TLS, and Modbus — cross-analyzer telemetry key inconsistency | LOW | Yes | NEW (see below) |
| PC-015 | `src/analyzer/arp.rs` | ARP has no findings-output cap (`MAX_FINDINGS`) and no `dropped_findings` counter; the no-cap intent is not documented (unlike HTTP/TLS which cite BC-2.04.024 invariant 4 explicitly) | LOW | No | NEW (see below) |

---

## Prior Findings Re-Confirmation (PC-001..PC-012)

All 12 prior findings verified as still present in HEAD dd3b069:

**PC-001 (HIGH) — CONFIRMED:**
`src/dispatcher.rs:315` still calls `dnp3.on_data(flow_key.clone(), data, timestamp)` with value-owned `FlowKey` (clone per packet). `Dnp3Analyzer` has no `StreamHandler`/`StreamAnalyzer` impl. `dispatcher.rs:347` still reads `"Dnp3Analyzer does not implement StreamHandler; no forwarding needed."` `on_flow_close` for DNP3 is still a silent no-op (`let _ = reason;`).

**PC-002 (HIGH) — CONFIRMED:**
- `src/analyzer/arp.rs` lines 442, 670, 804, 894, 977: `use crate::findings::{Confidence, ThreatCategory, Verdict}` inside method bodies (5 occurrences).
- `src/analyzer/modbus.rs`: ~40 occurrences of `crate::findings::ThreatCategory::`, `crate::findings::Verdict::`, `crate::findings::Confidence::` with no module-level import.
- `src/analyzer/dnp3.rs`: ~20 occurrences of same fully-qualified form.
- HTTP (`src/analyzer/http.rs:19`) and TLS (`src/analyzer/tls.rs:26`) use module-level `use crate::findings::{Confidence, Finding, ThreatCategory, Verdict}` — the established pattern.

**PC-003 (HIGH) — CONFIRMED:**
`Dnp3Analyzer` has `MAX_FINDINGS = 10_000` (line 178) and cap guards across 11 call sites, but zero `dropped_findings` struct field, no increment, and no `detail.insert("dropped_findings", ...)` in `summarize()`. `ModbusAnalyzer` has `pub dropped_findings: u64` and emits it in `summarize()` (line 951). The gap is unchanged.

**PC-004 (MEDIUM) — CONFIRMED:**
`src/analyzer/dnp3.rs` has 11 occurrences of `chrono::DateTime::from_timestamp(now_ts as i64, 0)` with no module-level import. `src/analyzer/modbus.rs` has `use chrono::DateTime;` inside `process_pdu` only (one function-scoped import). HTTP and TLS import `chrono::DateTime` at module level (`http.rs:16`, `tls.rs:18`).

**PC-005 (MEDIUM) — CONFIRMED:**
`src/analyzer/arp.rs` still duplicates the 6-field MAC format string and 4-field IPv4 format string across multiple functions. No `fmt_mac` or `fmt_ipv4` helper has been extracted.

**PC-006 (MEDIUM) — CONFIRMED:**
`src/analyzer/modbus.rs:969`: `analyzer_name: "modbus".to_string()` and `fn name() -> "modbus"` (line 1160). Comment at line 968 still claims this "matches http and tls analyzer convention" which is factually incorrect (HTTP emits `"HTTP"`, TLS emits `"TLS"`). All other analyzers emit mixed-case or uppercase.

**PC-007 (MEDIUM) — CONFIRMED:**
- `src/analyzer/arp.rs:723`: `use std::collections::BTreeMap` inside `summarize()`.
- `src/analyzer/modbus.rs:930`: same.
- `src/analyzer/dnp3.rs:1381`: same.
- DNS (`src/analyzer/dns.rs:17`): module-level import — the target pattern.

**PC-008 (MEDIUM) — CONFIRMED:**
`ModbusAnalyzer` still has two `summarize()` methods: inherent at line 929 and `StreamAnalyzer` trait delegation at line 1163. Trait impl uses `crate::analyzer::AnalysisSummary` in the return type instead of the already-imported `AnalysisSummary` (`use crate::analyzer::AnalysisSummary` at line 21).

**PC-009 (LOW) — CONFIRMED:**
`src/analyzer/dnp3.rs:1393`: `.map(|(&fc, &count)| (fc.to_string(), count))` produces decimal keys (`"5"` for FC=0x05). Modbus (line 959): `format!("0x{fc:02X}")` produces hex keys. Comment at line 1388 documents decimal as the intended format for DNP3.

**PC-010 (LOW) — CONFIRMED:**
`src/analyzer/dns.rs:35` `pub fn new()` has no doc comment. `fn analyze()` (line 70) has no doc comment explaining why it returns `Vec::new()`. All other analyzers document their `new()` methods.

**PC-011 (LOW) — CONFIRMED:**
`src/analyzer/http.rs:600`: `.collect::<HashMap<String, u64>>()` for `status_codes` serialization.
`src/analyzer/tls.rs:876`: `.collect::<HashMap<String, u64>>()` for `version_counts` serialization.
Both produce JSON with non-deterministic key order, inconsistent with the `BTreeMap`-based `detail` map used for top-level summary determinism.

**PC-012 (LOW) — CONFIRMED:**
`src/analyzer/dnp3.rs:26`: `#![allow(dead_code)]` still present as module-level attribute. Line 193 has a per-item `#[allow(dead_code)]` as well, confirming the module-level blanket is redundant and over-broad.

---

## New Findings Detail

### PC-013 — Production `.expect()` calls in `ArpAnalyzer` without parallel tracking

**Severity:** MEDIUM
**Category:** error-handling
**Files:** `src/analyzer/arp.rs:555,576,642,827`
**Auto-fixable:** No

Lines 555, 576, 642, and 827 are in production code (before the test block at line 1054) and call `.expect()` on `HashMap::get_mut()` results with invariant-enforcing messages:

```
// Line 555 — inside process_arp
.expect("has_conflict implies entry exists")

// Line 576 — inside process_arp
.expect("entry must still exist")

// Line 642 — inside process_arp
.expect("entry must still exist")

// Line 827 — inside apply_garp_conflict_escalation_impl (pub free fn)
let first_ts = entry.first_rebind_ts.expect("set in Step 2");
```

Lines 555 and 576 access the same `bindings` entry within the same branch that checks `has_conflict` (which requires `bindings.get().is_some()`). Line 827 accesses `first_rebind_ts.expect()` after `Some`-assignment on line 823. These are logically sound invariant assertions but they produce a panic rather than a structured error if the invariant is violated (e.g. by a future refactor that violates the ordering guarantee). The equivalent code in `src/reassembly/mod.rs:298` is tracked as CR-006 in the tech-debt register. The ARP calls are structurally identical but not tracked.

**Distinction from test expects:** Lines 1985, 1996, 2007, 2043, 2185 are inside `#[cfg(test)]` blocks (test starts at line 1054) — those are expected and appropriate.

**Refactor effort:** LOW (convert to `ok_or` or restructure to avoid double-lookup via `entry()` API)

**Register recommendation:** Add as TD-MAINT-PC013-ARP-PROD-EXPECT (P3); cross-reference CR-006.

---

### PC-014 — Telemetry key `"total_parse_errors"` (DNP3) diverges from `"parse_errors"` (HTTP, TLS, Modbus)

**Severity:** LOW
**Category:** telemetry-naming
**Files:** `src/analyzer/dnp3.rs:1425`
**Auto-fixable:** Yes

`Dnp3Analyzer::summarize()` inserts `"total_parse_errors"` while HTTP (line 617), TLS (line 884), and Modbus (line 947) all emit `"parse_errors"` for equivalent semantics. The `total_` prefix in the DNP3 key is unique across the codebase:

```rust
// dnp3.rs:1425 — outlier
detail.insert("total_parse_errors".to_string(), ...);

// http.rs:617, tls.rs:884, modbus.rs:947 — established pattern
detail.insert("parse_errors".to_string(), ...);
```

JSON consumers that aggregate parse-error counts across all analyzers must handle two key names. This is the same class of inconsistency as the `"total_frames"` / `"frames_analyzed"` split (those are different semantic concepts — frames vs analysis events — so they are intentionally distinct). The `"parse_errors"` vs `"total_parse_errors"` divergence, however, is purely cosmetic: both measure malformed protocol units that failed parsing.

**Note on ARP:** ARP uses `"malformed_frames"` — this refers to malformed ARP frames (protocol-level malformed, not parse failures), which is a distinct concept. The key name `"malformed_frames"` is appropriate for ARP.

**Refactor effort:** LOW (rename key + update snapshot test if one targets this field)

**Register recommendation:** Add as TD-MAINT-PC014-DNP3-PARSE-KEY (P3).

---

### PC-015 — ARP findings-output cap intent undocumented relative to HTTP/TLS precedent

**Severity:** LOW
**Category:** spec-fidelity / documentation
**Files:** `src/analyzer/arp.rs`
**Auto-fixable:** No

`ArpAnalyzer` has `MAX_ARP_BINDINGS` (line 57) and `MAX_STORM_COUNTERS` (line 94) for data-structure caps, but has no `MAX_FINDINGS` output cap and no `dropped_findings` counter. This is observably different from `ModbusAnalyzer` and `Dnp3Analyzer` which each have `MAX_FINDINGS = 10_000` output caps.

`HttpAnalyzer` and `TlsAnalyzer` also have no output cap, but they explicitly document this via `BC-2.04.024 invariant 4` citations in `http.rs:644-665` and `tls.rs:907-928`. `ArpAnalyzer` has no equivalent documentation explaining whether the absence of an output cap is intentional or an oversight.

The distinction matters for DoS analysis: ARP storm detection could in theory produce O(N) findings per capture where N is the number of unique storm-triggering MACs. Without a cap or documented reasoning, a security auditor cannot distinguish intentional behavior from missing implementation.

**Refactor effort:** LOW (document intent in module-level comment per BC citation, or add cap + dropped_findings counter like Modbus if per-BC-2.16 requirement exists)

**Register recommendation:** Add as TD-MAINT-PC015-ARP-NOCAP-DOC (P3); pending DF-VALIDATION-001 check before GitHub issue.

---

## Architecture Layer Check — `src/reader.rs` (pcapng reader)

`src/reader.rs` was examined as the primary new module added since the prior sweep.

**Layer compliance: PASS.** `reader.rs` imports only `std::io`, `anyhow`, and `pcap_file` — no `crate::` imports. It sits correctly below the decoder layer with no upward dependencies. The file-size gate (`MAX_PCAPNG_FILE_BYTES: u64 = 4_294_967_296`) uses `fstat` on the already-open fd (not a path-based TOCTOU call — note ADR-009 rev 9 Decision 27 / CWE-367).

**Constant naming: CONSISTENT.** All constants follow `SCREAMING_SNAKE_CASE`, consistent with the rest of the codebase.

**Error handling: CONSISTENT.** All production paths use `anyhow::Result` with `.context()` / `anyhow!()`. The one `unwrap_or()` at line 1379 (`DataLink::from(0)` fallback) is a documented sentinel per Architecture Compliance Rule M-3, using `unwrap_or` (safe fallback), not `unwrap()` (panic). No `unwrap()` or `expect()` in production paths.

**Duplicate timestamp path: NONE.** The `vp025_check` helper in the Kani harness (line 1576) re-implements the timestamp formula for proof-checking purposes. This is inside `#[cfg(kani)]` and intentional (per inline comment explaining the BMC tractability constraint). No unintentional duplication exists.

---

## Import Ordering

Import ordering across all analyzer files follows the canonical pattern `std::` → blank line → external crates → blank line → `crate::`. The pattern is consistent and auto-enforced by `rustfmt` with `edition = "2024"`. No violations detected.

---

## Error-Handling Survey

| Module | Pattern | Issues |
|--------|---------|--------|
| `src/analyzer/arp.rs` | `Vec::new()` returns; production `.expect()` on invariant asserts | PC-013 (4 production expect calls) |
| `src/analyzer/modbus.rs` | `anyhow::Result` at stream boundary; `unwrap_or` fallbacks | CR-006-adjacent (within test/Kani only) |
| `src/analyzer/dnp3.rs` | `anyhow::Result` for parse; silent cap drop (no counter) | PC-003 |
| `src/analyzer/http.rs` | Clean; no production `unwrap()`/`expect()` | None new |
| `src/analyzer/tls.rs` | Clean; no production `unwrap()`/`expect()` | None new |
| `src/analyzer/dns.rs` | Clean; returns `Vec::new()` always | None new |
| `src/reader.rs` | `anyhow::Result` throughout; `unwrap_or` sentinel only | None |
| `src/reassembly/mod.rs` | `get_mut().unwrap()` after guaranteed-insert pattern | CR-006 (existing register entry) |
| `src/reporter/json.rs:69` | `serde_json::to_string_pretty().unwrap()` (infallible, low risk) | CR-007 (existing register entry) |
| `src/dispatcher.rs` | Clean; trait-based dispatch | None new |

---

## Telemetry Key Naming Map

| Analyzer | Parse-Error Key | Frames/PDUs Key | Findings-Drop Key |
|----------|----------------|-----------------|-------------------|
| HTTP | `"parse_errors"` | `"transactions"` | (no cap per BC-2.04.024 inv4) |
| TLS | `"parse_errors"` | (SNI/JA3 focused) | (no cap per BC-2.04.024 inv4) |
| Modbus | `"parse_errors"` | `"pdu_count"` | `"dropped_findings"` |
| DNP3 | `"total_parse_errors"` (DIVERGES) | `"total_frames"` | (absent — PC-003) |
| ARP | `"malformed_frames"` (distinct concept) | `"frames_analyzed"` | (no cap — PC-015 doc gap) |

---

## Batch Refactoring Priority (Updated)

| Priority | Finding | Description | Effort |
|----------|---------|-------------|--------|
| 1 (HIGH)  | PC-001 | DNP3 `StreamHandler`/`StreamAnalyzer` conformance | 2–4 days |
| 2 (HIGH)  | PC-002 | Migrate inline `findings` imports to module level | < 1 day |
| 3 (HIGH)  | PC-003 | Add `dropped_findings` counter to DNP3 | < 1 day |
| 4 (MEDIUM)| PC-004 | Normalize `chrono::DateTime` import in DNP3 | < 1 day |
| 5 (MEDIUM)| PC-005 | Extract `fmt_mac`/`fmt_ipv4` helpers in ARP | < 1 day |
| 6 (MEDIUM)| PC-006 | Fix `analyzer_name` casing (`"modbus"` → `"Modbus"`) | trivial + snapshot test |
| 7 (MEDIUM)| PC-007 | Move `BTreeMap` imports to module level | < 30 min |
| 8 (MEDIUM)| PC-008 | Resolve Modbus `summarize()` dual-method pattern | < 1 day |
| 9 (MEDIUM)| PC-013 | Convert ARP production `.expect()` to `ok_or` or `entry()` | < 1 day |
| 10 (LOW)  | PC-009 | Align DNP3 FC key format to Modbus hex | < 1 hour |
| 11 (LOW)  | PC-011 | Fix non-deterministic `status_codes`/`tls_versions` in summaries | < 1 hour |
| 12 (LOW)  | PC-014 | Rename DNP3 `"total_parse_errors"` → `"parse_errors"` | < 30 min + snapshot |
| 13 (LOW)  | PC-012 | Remove module-level `#![allow(dead_code)]` from DNP3 | < 1 day |
| 14 (LOW)  | PC-015 | Document ARP no-cap intent per BC-2.04.024 pattern | < 30 min |
| 15 (LOW)  | PC-010 | Add missing doc comments to DNS analyzer | < 30 min |

---

## Tech-Debt Register Cross-Reference

| Finding | Register Status |
|---------|----------------|
| PC-001 | TD-MAINT-PC001-DNP3-STREAMTRAIT (DEFERRED P2) |
| PC-002 | NOT in register |
| PC-003 | TD-MAINT-PC003-DNP3-DROPPED-COUNTER (DEFERRED P3) |
| PC-004 | NOT in register |
| PC-005 | NOT in register |
| PC-006 | TD-MAINT-PC006-MODBUS-NAME-CASING (DEFERRED P3) |
| PC-007 | NOT in register |
| PC-008 | NOT in register |
| PC-009 | NOT in register |
| PC-010 | NOT in register (related: O-08 — DNS doc stale) |
| PC-011 | NOT in register |
| PC-012 | NOT in register |
| PC-013 | NEW — not in register; related to CR-006 |
| PC-014 | NEW — not in register |
| PC-015 | NEW — not in register |

DF-VALIDATION-001 applies to any finding recommended for a GitHub issue:
PC-013, PC-014, PC-015 are tagged above with register recommendations pending validation.
