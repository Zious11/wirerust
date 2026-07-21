---
document_type: maintenance-sweep-findings
sweep_id: maint-2026-07-21
sweep_number: 3
focus: code-pattern-consistency
branch: develop
commit: 1e967bad
timestamp: 2026-07-21T00:00:00Z
producer: code-reviewer
prior_sweep: maint-2026-07-11
---

# Pattern Findings — maint-2026-07-21 Sweep 3 (Pattern Consistency)

Base commit: `1e967bad` (develop). Cargo clippy (`--all-targets -D warnings`) and
`cargo fmt --check` both pass cleanly at this commit — no automated-gate violations.

---

## Prior-Sweep Item Status

### PC-NEW-001 — Spurious `#[allow(unused)]` on pub consts in `dnp3.rs` — RESOLVED

The 9 spurious `#[allow(unused)]` suppressions on actively-used `pub const` items
(previously at lines 122, 129, 139, 145, 150, 158, 167, 173, 180) have been removed
between b5e1e15 and 1e967bad. Only the 2 legitimate `#[allow(unused)]` remain at
lines 2082 (`is_broadcast_destination`) and 2099 (`is_master_frame`) — both test/doc-only
functions with no production callers. Additionally, `#[allow(dead_code)]` at line 207
on `Dnp3FlowState` was not in the prior-sweep register; it is confirmed appropriate because
the struct has extensive production use but Rust may flag some fields as dead when struct
is gated behind complex flow dispatch. No action required.

### PC-NEW-002 — `#[allow(clippy::too_many_arguments)]` in `dnp3.rs` without justification — RESOLVED

All 6 suppressions (now at lines 985, 1070, 1402, 1468, 1548, 1634 after line-number shift)
have justification comments documenting the parameter count rationale. Example at line 985:
`// 8 args is one above the default clippy limit (7); adding flow_key for BC-2.15.010 PC3...`
Pattern is now consistent with modbus.rs:343. No action required.

### PC-NEW-003 — Modbus/DNP3/ARP import-style drift not formally registered — CARRIED

No new register rows were created for the Modbus/DNP3/ARP import-style drift between sweeps.
The finding remains open. Status: CARRIED (see PAT-001 for IEC104 extension below).

---

## Known-Open Register Items — Re-verification

### PC-013 — 4 production `.expect()` in `arp.rs` — VERIFIED-STILL-PRESENT

All four invariant-enforced `.expect()` calls confirmed at lines 575, 596, 669, 866.
No change from prior sweep. Finding remains open.

### PC-015 — ArpAnalyzer no MAX_FINDINGS cap doc — VERIFIED-STILL-PRESENT

The `ArpAnalyzer` struct doc-comment (around line 354) still contains no explicit statement
that the unbounded findings behavior is intentional per BC-2.16.016. Finding remains open.

### PC-018 — HashMap-iteration serialization — VERIFIED-STILL-PRESENT

Both non-deterministic-key-order sites remain:
- `enip.rs:1539`: `for (&cmd, &count) in &enip_summary_struct.command_distribution`
- `modbus.rs:985`: `for (&fc, &count) in &self.fn_code_counts`

Both iterate a `HashMap` directly into JSON output, violating NFR DET-001. Fix: collect
into a `BTreeMap` before serializing. Finding remains open.

### PC-022 — ENIP import-style drift — VERIFIED-STILL-PRESENT

`enip.rs` continues to use fully-qualified `crate::findings::ThreatCategory/Verdict/Confidence`
inline paths (24+ occurrences), inline `chrono::DateTime::from_timestamp(...)` calls (8+),
and `BTreeMap` imported inside `summarize()` body rather than at module level. No change
from prior sweep. Finding remains open.

### SEC-001 — Unsafe split-borrow in `enip.rs` `on_data` — VERIFIED-STILL-PRESENT

`enip.rs:985–999` still uses `*mut EnipFlowState` raw pointer to work around the borrow
checker while calling `process_pdu` with `self` also borrowed. The SAFETY comment at
lines 985–991 correctly documents the invariant, but the pattern is fragile under
refactoring. Finding remains open (v0.12.0 candidate per prior register).

### HASHMAP-ENTRY-SATURATING-001 — `entry().or_insert(0) += 1` sites — VERIFIED-STILL-PRESENT

Non-compliant sites confirmed (13 total):

| File | Line | Field |
|------|------|-------|
| `dnp3.rs` | 790 | `flow.fc_counts` |
| `dnp3.rs` | 791 | `self.fn_code_counts` |
| `modbus.rs` | 409 | `self.fn_code_counts` |
| `modbus.rs` | 872 | `flow.exception_window_counts` (two-liner) |
| `enip.rs` | 866 | `flow.command_counts` |
| `enip.rs` | 892 | `flow.command_counts` |
| `enip.rs` | 923 | `flow.command_counts` |
| `enip.rs` | ~1165 | `flow.error_counts_in_window` (multi-line form) |
| `tls.rs` | 470 | `map` (cipher-suite distribution) |
| `http.rs` | 399 | `self.methods` |
| `http.rs` | 405 | `self.hosts` |
| `http.rs` | 414 | `self.user_agents` |
| `http.rs` | 486 | `self.status_codes` |

Note: `enip.rs:703–705` and `enip.rs:1513–1515` (`.saturating_add` two-liners) remain
compliant — do not convert. IEC-104 (`iec104.rs`) is clean: all counter increments use
`.saturating_add(1)` at the call site. Finding remains open.

---

## New Findings

### PAT-001 — IEC-104 import-style drift (extends PC-NEW-003 to newest analyzer)
**Severity:** LOW | **Auto-fixable:** Yes | **Effort:** < 30 min | **Status:** NEW

`iec104.rs` repeats the inner-function import pattern already noted for `arp.rs` in
PC-NEW-003. Specifically:

- `use crate::findings::{Confidence, ThreatCategory, Verdict};` appears inside two function
  bodies at lines 343 (`process_u_frame`) and 737 (`detect_iec104_threats`) rather than at
  module level.
- `use std::collections::BTreeMap;` appears inside `summarize()` at line 1384 rather than
  at module level.

The module-level import block (`iec104.rs:47–53`) imports only `Finding`, not the three
verdict/category/confidence types. This is the same deficit as `modbus.rs` (PC-NEW-003 class)
and is inconsistent with the clean module-level pattern in `http.rs` and `tls.rs`.

Fix: add `use crate::findings::{Confidence, ThreatCategory, Verdict};` and
`use std::collections::BTreeMap;` to the module-level import block; remove the
inner-function copies.

---

### PAT-002 — Bare `.unwrap()` without invariant messages in `reassembly/mod.rs`
**Severity:** LOW | **Auto-fixable:** Yes | **Effort:** < 30 min | **Status:** NEW

Five production calls to `self.flows.get_mut(key).unwrap()` in `reassembly/mod.rs` carry
no `.expect(...)` message documenting the guaranteed-present invariant:

| Line | Calling function |
|------|-----------------|
| 299 | `on_syn_or_mid_stream_join` — flow was just inserted |
| 318 | `apply_handshake_flags` — called after flow admission |
| 372 | `insert_payload_segment` — called after flow admission |
| 513 | `check_anomaly_thresholds` — called after flow admission |
| 620 | `flush_contiguous_data` — called after flow admission |

The invariant in all five cases is that these private methods are only reachable after the
flow has been inserted into `self.flows` by the public `process_packet` path. The codebase
has a documented convention for invariant-enforced `.expect()` messages: `enip.rs:995` uses
`.expect("flow exists: inserted above and not removed")` and `enip.rs:798` uses
`.expect("just inserted")`. The bare `.unwrap()` style offers no diagnostic on panic and
makes the invariant invisible to code reviewers.

Fix: replace each `.unwrap()` with `.expect("<brief invariant description>")`.

---

### PAT-003 — Inconsistent exec bit on `bin/test_*.py` files
**Severity:** NIT | **Auto-fixable:** Yes (`chmod +x`) | **Effort:** < 5 min | **Status:** NEW

`bin/test_compute_input_hash.py` has the exec bit set (`-rwxr-xr-x`) while all other test
files do not:

| File | Exec bit |
|------|----------|
| `test_compute_input_hash.py` | set |
| `test_changelog_gate_content.py` | not set |
| `test_check_green_doc_tense.py` | not set |
| `test_gitignore_mutants_glob.py` | not set |
| `test_lint_cycle_artifact.py` | not set |
| `test_validate_citations.py` | not set |

All test files are invoked as `python3 bin/test_xxx.py` per CLAUDE.md, so the exec bit is not
functionally required. The inconsistency is a scaffolding artifact. Fix: either add exec bit
to all test files (for uniformity with their corresponding main scripts) or remove it from
`test_compute_input_hash.py` (minimal-change).

---

### PAT-004 — `bin/compute-input-hash` uses manual `sys.argv` parsing while other Python tools use `argparse`
**Severity:** NIT | **Auto-fixable:** Partially | **Effort:** ~1 hour | **Status:** NEW

`bin/validate-citations` uses `argparse.ArgumentParser` with `add_argument` declarations.
`bin/compute-input-hash` (which has non-trivial flag combinations: `--write`, `--scan`,
optional path argument) uses manual `sys.argv[1:]` processing with a hand-rolled `--help`
string at lines 309–329. `bin/lint-cycle-artifact` uses a `main(argv: list[str] | None = None)`
parameterization pattern that aids testability.

The manual parsing in `compute-input-hash` works correctly but provides no auto-generated
`--help` output and is harder to extend. Migration to `argparse` would align it with
`validate-citations` and make the flag matrix (`--write --scan <glob>` combinations)
machine-readable.

Note: `bin/check-green-doc-tense` takes no arguments at all (auto-discovers repo root)
and is exempt from this finding.

---

## Automated Check Results

| Check | Result |
|-------|--------|
| `cargo clippy --all-targets -- -D warnings` | PASS — clean, no warnings suppressed by new `#[allow]` |
| `cargo fmt --check` | PASS — no formatting drift |
| `#[allow(...)]` inventory | No new unjustified suppressions since b5e1e15 |
| bin/ shebangs | All 6 executable scripts have `#!/usr/bin/env python3` or `#!/usr/bin/env bash` |
| bin/ exec bits (scripts) | All 6 main scripts have exec bit set |
| bin/ test coverage | `fetch-e2e-pcaps` (bash download script) has no `test_*.py` — acceptable for network-dependent bash scripts |

---

## Summary Table

### Carried Items

| ID | Severity | Location | Description | Auto-fix? | Status |
|----|----------|----------|-------------|-----------|--------|
| PC-013 | LOW | `arp.rs:575,596,669,866` | 4 invariant-enforced `.expect()` in production code | No | CARRIED |
| PC-015 | NIT | `arp.rs` struct doc | ArpAnalyzer missing no-MAX_FINDINGS-cap design note | Yes | CARRIED |
| PC-018 | LOW | `enip.rs:1539`, `modbus.rs:985` | HashMap iteration in serialization (non-deterministic JSON order) | Yes | CARRIED |
| PC-022 | LOW | `enip.rs` (24+ sites) | ENIP import-style drift: inline `crate::findings::*` paths, inline chrono, BTreeMap in function body | Yes | CARRIED |
| SEC-001 | MEDIUM | `enip.rs:985–999` | Unsafe split-borrow via raw pointer in `on_data`; sound but fragile | No | CARRIED |
| HASHMAP-ENTRY-SATURATING-001 | LOW | 5 files, 13 sites | `entry().or_insert(0) += 1` without `.saturating_add` | Yes | CARRIED |
| PC-NEW-003 | LOW | `modbus.rs`, `dnp3.rs`, `arp.rs` | Import-style drift not formally registered with dedicated IDs | No (register work) | CARRIED |

### Resolved Items

| ID | Resolution |
|----|------------|
| PC-NEW-001 | 9 spurious `#[allow(unused)]` on pub consts in `dnp3.rs` removed |
| PC-NEW-002 | All 6 `#[allow(clippy::too_many_arguments)]` in `dnp3.rs` now have justification comments |

### New Items

| ID | Severity | Location | Description | Auto-fix? | Status |
|----|----------|----------|-------------|-----------|--------|
| PAT-001 | LOW | `iec104.rs:343,737,1384` | IEC-104 import-style drift: inner-function use of findings types + BTreeMap (extends PC-NEW-003) | Yes | NEW |
| PAT-002 | LOW | `reassembly/mod.rs:299,318,372,513,620` | Bare `.unwrap()` on `flows.get_mut(key)` without invariant messages | Yes | NEW |
| PAT-003 | NIT | `bin/test_compute_input_hash.py` | Inconsistent exec bit on test files | Yes | NEW |
| PAT-004 | NIT | `bin/compute-input-hash:309–329` | Manual `sys.argv` parsing vs `argparse` in other Python tools | Partially | NEW |

**NEW finding count: 4 (2 LOW, 2 NIT)**
**RESOLVED since prior sweep: 2 (PC-NEW-001, PC-NEW-002)**
