---
document_type: maintenance-sweep-findings
sweep_id: maint-2026-07-11
sweep_number: 3
focus: code-pattern-consistency
branch: develop
commit: b5e1e15
timestamp: 2026-07-11T00:00:00Z
producer: sweep3-pattern-consistency
---

# Pattern Findings — maint-2026-07-11 Sweep 3 (Code Pattern Consistency)

Base commit: `b5e1e15` (develop). No uncommitted changes in `src/`.
Commits scanned since PF-001 resolution (`c4eb1f4`): `75c5ba5` (CI only), `704fd2e` (reporter/findings only).

---

## Known-Open Register Items — Re-verification

### PC-013 — 4 production `.expect()` in `arp.rs` — VERIFIED-STILL-PRESENT

All four `.expect()` calls confirmed in production code (before `#[cfg(test)]` at line 1092).
Line numbers have shifted from the registered values by approximately +20 lines:

| Registered | Current | Message |
|-----------|---------|---------|
| arp.rs:555 | arp.rs:575 | `"has_conflict implies entry exists"` |
| arp.rs:576 | arp.rs:596 | `"entry must still exist"` |
| arp.rs:642 | arp.rs:669 | `"entry must still exist"` |
| arp.rs:827 | arp.rs:864 | `"set in Step 2"` |

All four are `HashMap::get_mut()` / `Option::expect()` calls on invariant-enforced entries. Register
line numbers should be updated; finding remains open.

---

### PC-014 — DNP3 `"total_parse_errors"` key — NO-LONGER-PRESENT (RESOLVED IN CODE)

The string `"total_parse_errors"` does not appear anywhere in `src/`. `dnp3.rs:1870` now emits
`"parse_errors"`, consistent with HTTP (`http.rs:630`), TLS (`tls.rs:1139`), Modbus
(`modbus.rs:971`), and ENIP (`enip.rs:1558`). The register still shows PC-014 as OPEN — the
register row should be updated to RESOLVED.

---

### PC-015 — ArpAnalyzer no MAX_FINDINGS cap doc — VERIFIED-STILL-PRESENT

The `ArpAnalyzer` struct doc-comment (`arp.rs:344–354`) does not mention the intentional
no-MAX_FINDINGS-cap design. The characterization test `test_BC_2_16_016_arp_findings_vec_has_no_cap`
(line 4449) documents this at the test level (BC-2.16.016), but the implementation-level struct
and `process_arp` function comments contain no explicit statement that the unbounded behavior is
by design per BC-2.16.016. Security auditors reading the struct cannot distinguish intentional
from missing without reading the test module. Finding remains open per register.

---

### PC-018 — HashMap-iteration serialization — VERIFIED-STILL-PRESENT

Both registered sites still iterate a `HashMap` directly into the JSON output, producing
non-deterministic key ordering (violates NFR DET-001):

- `enip.rs:1539`: `for (&cmd, &count) in &enip_summary_struct.command_distribution` —
  `command_distribution` is `HashMap<u16, u64>`; keys format as hex strings.
- `modbus.rs:985`: `for (&fc, &count) in &self.fn_code_counts` —
  `fn_code_counts` is `HashMap<u8, u64>`; keys format as hex strings.

Fix: collect into a `BTreeMap` before serializing (same class as PC-011 HTTP/TLS). Finding
remains open per register.

---

### PC-022 — ENIP import-style drift — VERIFIED-STILL-PRESENT

Counts unchanged from last sweep:

- Fully-qualified `crate::findings::ThreatCategory/Verdict/Confidence` inline paths: **24**
  (neither imported at module level nor via inner `use`)
- Fully-qualified `chrono::DateTime::from_timestamp(...)` inline calls: **8**
- `BTreeMap` imported inside `summarize()` body at `enip.rs:1487` instead of module level

All three sub-items confirmed. Finding remains open per register.

---

### SEC-001 — unsafe split-borrow in `enip.rs` `on_data` — VERIFIED-STILL-PRESENT

`enip.rs:985–999` (for-loop over `pdu_queue`):

```text
enip.rs:992  let flow_ptr: *mut EnipFlowState = self
enip.rs:993      .flows
enip.rs:994      .get_mut(&flow_key)
enip.rs:995      .expect("flow exists: inserted above and not removed");
enip.rs:998  #[allow(clippy::ptr_as_ptr)]
enip.rs:999  self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, timestamp, src_ip);
```

The `&mut` from `get_mut` is transmuted to a `*mut EnipFlowState` and then back to `&mut` inside
`unsafe` while `self` is also borrowed by `process_pdu`. Safety invariant is documented in the
SAFETY comment (lines 986–991) and holds under current code, but is fragile under refactoring.
Finding remains open per register (v0.12.0 candidate).

---

### HASHMAP-ENTRY-SATURATING-001 — `entry().or_insert(0) += 1` sites — VERIFIED-STILL-PRESENT

14 sites confirmed across 5 analyzer files. The register estimated ~15; exact enumeration:

| File | Line | Field |
|------|------|-------|
| `dnp3.rs` | 799 | `flow.fc_counts` |
| `dnp3.rs` | 800 | `self.fn_code_counts` |
| `modbus.rs` | 409 | `self.fn_code_counts` |
| `modbus.rs` | 873 | `flow.exception_window_counts` (two-liner: `let count = entry; *count += 1`) |
| `enip.rs` | 866 | `flow.command_counts` |
| `enip.rs` | 892 | `flow.command_counts` |
| `enip.rs` | 923 | `flow.command_counts` |
| `enip.rs` | 1165 | `flow.error_counts_in_window` |
| `tls.rs` | 470 | `map` (cipher-suite distribution) |
| `http.rs` | 399 | `self.methods` |
| `http.rs` | 405 | `self.hosts` |
| `http.rs` | 414 | `self.user_agents` |
| `http.rs` | 486 | `self.status_codes` |

Note: `enip.rs:703–705` and `enip.rs:1513–1515` use `let e = entry.or_insert(0); *e = e.saturating_add(count)` — these ARE already compliant and should not be converted.

The `modbus.rs:872–873` two-liner form was not captured by the one-liner grep pattern used in
prior sweeps. It is the same logical class.

---

### PF-001 Saturating-Counter Discipline — HOLDS

No plain `+=` on diagnostic counters introduced in `src/` since `c4eb1f4` (PR #384).
`git diff c4eb1f4..HEAD -- src/` shows only:
- `findings.rs`: `#[serde(rename_all)]` attribute additions (not counter increments)
- `reporter/json.rs`: `SCHEMA_VERSION` constant + output field (not counter increments)
- `arp.rs`: a test comment change

Discipline holds. No new violations.

---

## New Findings

### PC-NEW-001 — Spurious `#[allow(unused)]` on actively-used public constants in `dnp3.rs`
**Severity:** NIT | **Auto-fixable:** Yes | **Effort:** < 30 min

Nine `#[allow(unused)]` suppressions appear on `pub const` items in `dnp3.rs` that are actively
used in non-test, non-kani production code:

| Line | Constant | Used at |
|------|----------|---------|
| 122 | `MAX_PENDING_REQUESTS` | `dnp3.rs:1912` |
| 129 | `MAX_DNP3_FRAME_LEN` | `dnp3.rs:492` |
| 139 | `MAX_DNP3_CARRY_BYTES` (deprecated alias) | via alias chain |
| 145 | `MAX_MASTER_ADDRS` | `dnp3.rs:783` |
| 150 | `MALFORMED_ANOMALY_THRESHOLD` | `dnp3.rs:1752/1784` |
| 158 | `CORRELATION_WINDOW_SECS` | `dnp3.rs:1316/1751` |
| 167 | `BLOCK_CMD_TIMEOUT_SECS` | `dnp3.rs:1206/1246` |
| 173 | `BLOCK_CMD_THRESHOLD` | `dnp3.rs:1227/1250` |
| 180 | `T0827_THRESHOLD` | `dnp3.rs:1354/1382` |

Since `analyzer` is a `pub` module in the library crate and these are `pub const` items, Rust
would not warn about them as dead code without the attribute. These suppressions are scaffolding
artifacts from the initial stub phase and can be removed without any risk.

Two `#[allow(unused)]` instances on functions at `dnp3.rs:2085` (`is_broadcast_destination`) and
`dnp3.rs:2102` (`is_master_frame`) are LEGITIMATE — both functions are only called from test or
doc contexts and would otherwise generate unused warnings. Retain those two.

Fix: remove the 9 spurious `#[allow(unused)]` attributes from lines 122, 129, 139, 145, 150,
158, 167, 173, 180. Retain lines 2085 and 2102.

---

### PC-NEW-002 — Clippy `too_many_arguments` suppressions in `dnp3.rs` lack justification comments
**Severity:** NIT | **Auto-fixable:** Partially | **Effort:** < 1 hour

`dnp3.rs` has 6 `#[allow(clippy::too_many_arguments)]` suppressions (lines 994, 1079, 1409,
1475, 1553, 1637) with no rationale comment. Compare with `modbus.rs:343` which documents its
reason inline:

```rust
// 8 params: interface dictated by STORY-105 wiring (FlowKey, flow state,
// direction, header, fc, raw data, timestamp)
```

and `main.rs:173` which is self-evident from context. The `dnp3.rs` suppressions are silent,
making it impossible to verify at review time whether the argument count is truly necessary or
whether refactoring is due.

Fix: add a `// N params: <reason>` comment immediately before each of the 6 suppressions, or
alternatively replace with `#[expect(clippy::too_many_arguments, reason = "...")]` (Rust 1.81+,
available since this project requires 1.91+).

---

### PC-NEW-003 — Modbus/DNP3 import-style drift not formally registered
**Severity:** LOW | **Auto-fixable:** Yes | **Effort:** ~30–60 min per file

PC-022 references "existing PC-002/PC-004/PC-007 (Modbus/DNP3)" but no register rows exist for
those IDs. The same class of import-style drift is present in `modbus.rs` and `dnp3.rs` and
is untracked as an explicit open item:

- `modbus.rs`: 28 inline `crate::findings::ThreatCategory/Verdict/Confidence` fully-qualified
  path usages (module-level `use crate::findings::Finding` but not the other three types).
  Also uses `use chrono::DateTime;` inside a single function body at line 354 rather than at
  module level.
- `dnp3.rs`: 34 inline `crate::findings::*` fully-qualified path usages + 11 inline
  `chrono::DateTime::from_timestamp(...)` calls. Module-level import (`dnp3.rs:31`) imports only
  `Finding`, not `ThreatCategory`, `Verdict`, or `Confidence`.

`arp.rs` uses a third style: inner-function `use crate::findings::{Confidence, ThreatCategory, Verdict};`
at 10+ function-body locations (lines 462, 699, 752, 841, 931, 1015, etc.) rather than a single
module-level import.

By contrast, `http.rs` and `tls.rs` import all four findings types at module level, which is the
cleanest pattern. This registration gap means the Modbus/DNP3/ARP import drift has no formal
register row to track remediation against.

Recommended action: create explicit register rows (reuse IDs PC-002/PC-004/PC-007 or assign
new IDs) to formally track the Modbus, DNP3, and ARP import-style drift as peers to PC-022.
These can be batch-fixed in the same PR as PC-022.

---

## Checks with No New Findings

**Check 1 — Error-handling style consistency:** Uniform across all stream analyzers. Parse errors
are counted via saturating arithmetic; no `Result`/`?` usage in `on_data` paths. The architectural
gap (DNP3/ENIP bespoke methods vs StreamHandler/StreamAnalyzer trait) is already tracked as
TD-MAINT-PC001-DNP3-STREAMTRAIT (DEFERRED) and PC-020 (DEFERRED). No new violations.

**Check 4 (import ordering) — ordering within compliant files:** `http.rs`, `tls.rs`, and
`modbus.rs` module-level imports follow the pattern `std::` → external crates → `crate::`. No
violation within those files beyond the inline-path issue noted in PC-NEW-003.

**Check 5 — Architecture layer rules (pure-core vs effectful-I/O boundary):**
- No `eprintln!`, `println!`, `File::open`, `std::fs`, or any I/O in analyzer pure-core
  function bodies
- `arp.rs` enforces its forbidden-imports list (lines 38–40: no `crate::dispatcher`,
  `crate::analyzer::modbus`, or `crate::analyzer::dnp3`)
- No new boundary violations found

---

## Summary Table

### Known-Open Items (register re-verification)

| ID | Status | Current Location |
|----|--------|-----------------|
| PC-013 | VERIFIED-STILL-PRESENT | arp.rs:575, 596, 669, 864 (line numbers shifted +~20) |
| PC-014 | NO-LONGER-PRESENT — resolved in code; register needs update | dnp3.rs:1870 now uses `"parse_errors"` |
| PC-015 | VERIFIED-STILL-PRESENT | ArpAnalyzer struct doc-comment missing no-cap design note |
| PC-018 | VERIFIED-STILL-PRESENT | enip.rs:1539, modbus.rs:985 |
| PC-022 | VERIFIED-STILL-PRESENT | enip.rs (24+8+1 drift sites) |
| SEC-001 | VERIFIED-STILL-PRESENT | enip.rs:992–999 |
| HASHMAP-ENTRY-SATURATING-001 | VERIFIED-STILL-PRESENT | 14 sites across 5 files (enumerated above) |
| PF-001 discipline | HOLDS — no new plain `+=` violations since c4eb1f4 | — |

### New Findings

| ID | Severity | Description | Auto-fixable? | Effort |
|----|----------|-------------|---------------|--------|
| PC-NEW-001 | NIT | 9 spurious `#[allow(unused)]` on pub consts in `dnp3.rs` that are actively used in production | Yes | < 30 min |
| PC-NEW-002 | NIT | 6 `#[allow(clippy::too_many_arguments)]` in `dnp3.rs` lack rationale comments (cf. modbus.rs:343) | Partially | < 1 hour |
| PC-NEW-003 | LOW | Modbus/DNP3/ARP import-style drift not formally registered; PC-022 references "PC-002/004/007" but no register rows exist | Yes (with PC-022 batch) | Register row addition only |

**NEW-finding count: 3**

---

## Notes for Register Maintainer

1. **PC-014** should be marked RESOLVED in the register. The fix landed before this sweep in an
   unspecified commit — the key `"total_parse_errors"` no longer exists in `src/`.

2. **PC-013** line numbers should be updated from `arp.rs:555/576/642/827` to `arp.rs:575/596/669/864`.

3. **PC-NEW-003** recommendation: assign formal register IDs to the Modbus/DNP3/ARP import-style
   drift (suggested: reuse informal PC-002/004/007 references or assign PC-023/024/025). Batch
   with PC-022 remediation PR.

4. **HASHMAP-ENTRY-SATURATING-001** exact site count is 14, not the estimated ~15. The `modbus.rs:873`
   two-liner variant (split across two lines) was not captured by the original one-liner grep.
   Register description should be updated to note the two-liner form.
