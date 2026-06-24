---
document_type: fix-bundle-scope
bundle_id: fix-pc-013-014-015
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-23T00:00:00Z
pipeline_state: QUIESCED-at-v0.9.4 (develop=0115d0e)
source_items:
  - STATE.md PC-013
  - STATE.md PC-014
  - STATE.md PC-015
  - tech-debt-register.md PC-013
  - tech-debt-register.md PC-014
  - tech-debt-register.md PC-015
---

# F1 Delta Analysis — Post-v0.9.4 Defect Fix Bundle Scope

## Summary Table

| ID | Title | Classification | Spec Change? | Delivery Priority |
|----|-------|---------------|--------------|-------------------|
| PC-013 | ARP production `.expect()` panic sites | **(b) Code → existing BC** | BC update only (no new behavior) | 2nd (after BC update) |
| PC-014 | DNP3 `total_parse_errors` key name divergence | **(b) Code → existing BC** | BC update (key rename + snapshot update) | 3rd |
| PC-015 | ARP findings cap not documented in CLI help | **(b or c)** — doc/spec hybrid | NEW BC or BC update for ARP no-cap invariant | 1st (spec-only; unblocks other doc work) |

---

## PC-013: ARP Production `.expect()` Sites — Panic-on-Malformed Risk

### 1. Exact Code Sites

File: `src/analyzer/arp.rs`

| Line | Call Site | Context |
|------|-----------|---------|
| 555 | `.expect("has_conflict implies entry exists")` | `process_arp` — GARP-conflict path. After `has_conflict = bindings.get(&sender_ip).map(...).unwrap_or(false)` returns `true`, code calls `bindings.get_mut(&sender_ip).expect(...)`. The implication holds as long as no concurrent mutation removes the entry between the `get` and `get_mut` calls. In single-threaded `process_arp` this is logically sound but `.expect()` panics if the invariant is ever violated by future refactoring. |
| 576 | `.expect("entry must still exist")` | `process_arp` — GARP-conflict Step 4 MAC update. A second `bindings.get_mut(&sender_ip)` after `apply_garp_conflict_escalation_impl`. Invariant holds for same single-threaded reason. |
| 642 | `.expect("entry must still exist")` | `process_arp` — non-GARP rebind Step 4 MAC update. After emitting a D1 finding via `emit_d1_spoof_finding_impl`, the entry is re-borrowed for MAC update. Invariant holds for same reason. |
| 827 | `entry.first_rebind_ts.expect("set in Step 2")` | `emit_d1_spoof_finding_impl` — Step 3 evaluates `first_rebind_ts` which was unconditionally set to `Some(timestamp_secs)` in Step 2 immediately above. Logically sound; panic is unreachable absent compiler/optimizer UB. |

**Risk characterization:** None of these four `.expect()` calls are directly reachable from malformed packet input in the current implementation. Lines 555/576/642 panic only if the HashMap invariant (entry present after `contains_key` confirmed it) is violated — which cannot happen in single-threaded code without a compiler bug. Line 827 panics only if `first_rebind_ts` is `None` at Step 3, which is impossible because Step 2 sets it unconditionally.

**The real risk is internal invariant fragility:** future refactoring that changes `process_arp` to be async, removes an intermediate step, or reorders code could silently break the invariant while the `.expect()` panic message misleads debugging. This is NOT a current CVE-class panic-on-packet risk, but rather a code health / adversarial-robustness debt that maintenance correctly flagged.

**Verdict:** The tech-debt register entry (PC-013) and STATE.md description ("panic-on-malformed risk") are slightly overstated: the `.expect()` sites are on invariants derived from **analyzer state logic**, not directly from **untrusted packet bytes**. The actual packet input safety is guaranteed by `extract_arp_frame` (VP-024 Sub-A, formally verified). However the remediation is still correct: replace with `ok_or_else(...)` / graceful error return or `debug_assert!`+`?` pattern to make future refactors safe.

**Test suite cross-check:** Lines 1246, 1352, 1401, 1755, 1985, 1996, 2007 in `arp.rs` are ALL within `#[cfg(test)]` blocks (post line 1094) and are NOT production panics. The four production `.expect()` sites are exclusively at lines 555, 576, 642, and 827.

### 2. Governing Behavioral Contract

- **BC-2.16.003** (GARP detection) and **BC-2.16.004** (D1 ARP Spoof escalation) govern the detection logic paths where lines 555/576 and 642 respectively sit.
- **BC-2.16.004** governs `emit_d1_spoof_finding_impl` where line 827 sits.
- **VP-024** (ARP Parse Safety) is the formal verification property — its Sub-A harness proves `extract_arp_frame` panic-freedom, but does NOT cover the `.expect()` sites in `process_arp` and `emit_d1_spoof_finding_impl`.

No existing BC currently specifies the graceful-error behavior for analyzer state invariant failures. VP-024's scope is decoder-level parse safety, not mid-analysis invariant robustness.

### 3. Fix Classification

**(b) Code brought into line with an existing-but-underspecified invariant.** No behavior change is needed: the invariants ARE logically correct and should remain. The fix is defensive coding style: replace `.expect()` with either:

- Option (a): `if let Some(entry) = bindings.get_mut(&sender_ip)` guards, silently continuing if the invariant is violated (fail-safe, no panic).
- Option (b): `debug_assert!(bindings.contains_key(&sender_ip), "..."); bindings.get_mut(&sender_ip)?` returning an error up the call stack.

Option (a) is preferred: ARP finding emission is best-effort; silently dropping a finding on a broken invariant is safer than propagating an error that aborts the capture. This matches the pattern established for other graceful-degradation paths (e.g., the `decode_packet` lax-parse fallthrough).

**BC change required:** BC-2.16.004 (and by extension BC-2.16.003/BC-2.16.014 which drive lines 555/576) need an **invariant addendum** (not a new postcondition) stating: *"If the binding-table entry for `sender_ip` is absent at any point where it is expected to exist, `process_arp` silently skips the current operation without panicking (fail-safe degradation)."* This is a **BC version bump** (clarification), NOT a behavioral change. No new BC ID is required.

### 4. Per-Item Test Strategy (Red Gate)

**Failing tests before fix:**

Since the `.expect()` calls are on logically-unbreachable invariants, there is no test that currently triggers a panic. The Red Gate test must assert the robustness guarantee by **simulating the invariant violation**:

```rust
// test_BC_2_16_004_expect_site_no_panic_on_missing_entry (Red Gate)
// Construct an ArpAnalyzer, manually remove an entry from bindings
// after setting has_conflict=true via a first frame, then send a
// second conflicting frame. The test should NOT panic — with the
// current code using .expect(), this test would trigger a panic
// if the entry were absent.
//
// Implementation: Use ArpAnalyzer::new_for_test(), process two frames
// establishing a binding, then directly mutate bindings (via pub field
// accessible in tests) to remove the entry, then process a third
// conflicting GARP frame. Assert: no panic, zero findings returned.
```

For line 827 (`first_rebind_ts.expect("set in Step 2")`), a direct unit test of `emit_d1_spoof_finding_impl` with a zero-initialized entry (first_rebind_ts = None, rebind_count = 0) would panic with current code. After the fix (using `?` or `if let`), it should return gracefully.

**Test location:** `src/analyzer/arp.rs` `#[cfg(test)] mod tests` (matching existing conventions for unit-level tests). Integration-level AC tests for the graceful-skip behavior should go in `tests/bc_2_16_story113_arp_tests.rs`.

### 5. Regression Risk & Blast Radius

- **Blast radius: LOW.** Changes are confined to `src/analyzer/arp.rs` (4 call sites). No public API changes.
- **Tests at risk:** All tests in `src/analyzer/arp.rs` `mod tests` (lines 1094–4439) and `tests/bc_2_16_story113_arp_tests.rs`. None should change behavior since the invariants hold in all test scenarios.
- **VP-024 at risk:** VP-024 Sub-A/B/C/D Kani proofs are in `src/decoder.rs` and `src/analyzer/arp.rs`. Replacing `.expect()` with `if let Some(...)` in production functions does NOT change the Kani-proved pure-core functions (`is_gratuitous_arp`, `insert_binding_lru`, `insert_binding_lru_array`). VP-024 re-verification is NOT required.
- **Kani unwind budgets:** No Kani harnesses are inside `process_arp` or `emit_d1_spoof_finding_impl` — confirmed safe.

---

## PC-014: DNP3 `total_parse_errors` Key Name Divergence

### 1. Exact Code Sites

File: `src/analyzer/dnp3.rs`

| Line | Code | Issue |
|------|------|-------|
| 1425–1426 | `detail.insert("total_parse_errors".to_string(), ...)` | Key name `"total_parse_errors"` |

**Comparison with sibling analyzers:**

| Analyzer | Key name in `summarize()` | File |
|----------|---------------------------|------|
| HTTP | `"parse_errors"` | `src/analyzer/http.rs:617` |
| TLS | `"parse_errors"` | `src/analyzer/tls.rs:884` |
| Modbus | `"parse_errors"` | `src/analyzer/modbus.rs:947` |
| **DNP3** | **`"total_parse_errors"`** | `src/analyzer/dnp3.rs:1425` ← diverges |

**Critical clarification on STATE.md description:** The STATE.md entry says "total_parse_errors key missing from output map." This is **inaccurate**. The key IS present in the code at line 1425. The actual defect is a **naming inconsistency** — the key is named `"total_parse_errors"` instead of `"parse_errors"`. Cross-analyzer JSON consumers (scripts, SIEM connectors, integration tests) that lookup `"parse_errors"` on DNP3 output will receive `None`/missing.

**Evidence the key IS present:**
- `dnp3.rs:1385`: `let total_parse_errors: u64 = self.flows.values().map(|f| f.parse_errors).sum();`
- `dnp3.rs:1425–1426`: `detail.insert("total_parse_errors".to_string(), ...)`
- `tests/dnp3_detection_tests.rs:995`: test uses `.get("total_parse_errors")` and passes — consistent with implementation.

**BC-2.15.020 alignment:** The BC at `.factory/specs/behavioral-contracts/ss-15/BC-2.15.020.md` (line 57) specifies: `"total_parse_errors": sum of flow.parse_errors across all flows`. The BC was written to match the implementation, NOT to match the cross-analyzer convention. The BC itself is therefore consistent with the code, but inconsistent with the cross-analyzer naming pattern.

### 2. Governing Behavioral Contract

**BC-2.15.020** ("summarize() Emits Function-Code Distribution and Control-Operation Counts") at v1.3 — Postcondition 1 line 57 specifies `total_parse_errors` (verbatim). This BC currently documents the divergent key name.

### 3. Fix Classification

**(b) Code rename to align with cross-analyzer convention.** Two sub-changes:

1. Rename `"total_parse_errors"` → `"parse_errors"` in `src/analyzer/dnp3.rs:1425`.
2. Update all test references from `"total_parse_errors"` to `"parse_errors"`:
   - `tests/dnp3_detection_tests.rs:995` (`.get("total_parse_errors")`)
   - `tests/dnp3_detection_tests.rs:1378` (`.contains_key("total_parse_errors")`)
   - `tests/bc_2_15_110_dnp3_dispatcher_tests.rs:959` (reference in bc_2_15 test; verify this is a local variable assignment, not a key lookup — if a key lookup, also update)

**BC change required:** BC-2.15.020 v1.3 requires a **version bump** to update Postcondition 1 line 57 from `"total_parse_errors"` to `"parse_errors"`. This is the spec source of truth for the key name. The BC version bump is required BEFORE the code rename so the Red Gate test (asserting `"parse_errors"` present, `"total_parse_errors"` absent) can be written against the updated spec.

**Breaking change:** This IS a breaking change for any caller that currently reads `"total_parse_errors"` from DNP3 JSON output. However, since the product is pre-1.0 and no external consumers are documented, this is acceptable. The public `AnalysisSummary.detail` field is not stability-guaranteed. Document in CHANGELOG.

### 4. Per-Item Test Strategy (Red Gate)

```rust
// test_BC_2_15_020_parse_errors_key_name_is_parse_errors (Red Gate — FAILS before fix)
// Assert: summary.detail.contains_key("parse_errors") == true
// Assert: summary.detail.contains_key("total_parse_errors") == false
// This test FAILS on current develop (key is "total_parse_errors") and PASSES after rename.
```

Existing test `test_BC_2_15_020_summarize_includes_parse_errors` at `tests/dnp3_detection_tests.rs:1362` already checks `.contains_key("total_parse_errors")` — this becomes the test to UPDATE (rename the assertion key), turning it red on current code and green after the fix.

Also add a sibling-consistency test:

```rust
// test_summarize_cross_analyzer_parse_errors_key_consistency
// Run all four analyzers (HTTP, TLS, Modbus, DNP3) summarize() and assert
// each detail map contains "parse_errors" (uniform key name).
// Location: tests/ (integration-level, cross-analyzer)
```

### 5. Regression Risk & Blast Radius

- **Blast radius: LOW-MEDIUM.** Single rename in `src/analyzer/dnp3.rs`. All callers that reference `"total_parse_errors"` by string must be updated atomically.
- **Tests that will flip red → green after fix:** 2 test assertions in `dnp3_detection_tests.rs`. Any test that asserts `total_parse_errors` absent will flip green without code change.
- **Snapshot tests:** No JSON golden-file snapshots found that contain the `total_parse_errors` string literal. Verify with `grep -r total_parse_errors tests/` before PR.
- **VP-023 (DNP3 parse safety):** Not affected — VP-023 is about the Kani proof of `parse_dn3_frame`. Key name changes have no impact on formal proofs.
- **BC-2.15.020 dependent stories:** STORY-106/107/108/109/110 cite BC-2.15.020. Their AC test names that reference `total_parse_errors` as a BC postcondition label (not a code string) should be noted in story AC columns, but do NOT require test name changes (test names reference BC IDs, not key strings).

---

## PC-015: ARP Findings Cap Not Documented in Public CLI Help

### 1. Exact Code Sites and Documentation Gap

**The actual behavior (from `src/analyzer/arp.rs` and `src/reassembly/mod.rs`):**

| Analyzer | Findings cap | Dropped counter | Cap in --help |
|----------|-------------|-----------------|---------------|
| HTTP | ~10,000 (MAX_FINDINGS, reassembly) | yes (`dropped_findings`) | not surfaced |
| TLS | ~10,000 (MAX_FINDINGS, reassembly) | yes (`dropped_findings`) | not surfaced |
| Modbus | 10,000 (local MAX_FINDINGS, `src/analyzer/modbus.rs`) | yes (`dropped_findings`) | not surfaced |
| DNP3 | 10,000 (local MAX_FINDINGS, `src/analyzer/dnp3.rs`) | NO (TD-MAINT-PC003) | not surfaced |
| **ARP** | **NONE — no cap** | **NONE** | **no cap, no mention** |

**Critical clarification on STATE.md and tech-debt-register PC-015:** The STATE.md entry says "ARP findings cap behavior exists but is not surfaced in --help / CLI documentation." This is **inaccurate**. The ARP analyzer has **NO findings output cap** at all. The binding table has `MAX_ARP_BINDINGS = 65,536` as a memory cap, and the storm counters have `MAX_STORM_COUNTERS = 4,096`, but there is no equivalent `MAX_FINDINGS` cap on the findings `Vec` returned by `process_arp`. The missing documentation is therefore not about "a cap that exists but is undocumented" — it is about **the intentional absence of a cap**, which is also undocumented.

**File: `src/cli.rs`**
- Lines 194–213: ARP CLI flag definitions. No mention of findings cap behavior (present or absent) in any `///` doc comment.
- The `--help` output will therefore show only the three ARP flags (`--arp`, `--arp-spoof-threshold`, `--arp-storm-rate`) with no information about memory bounds on findings.

**Comparison with HTTP/TLS:** `src/analyzer/http.rs` and `src/analyzer/tls.rs` use the reassembly `MAX_FINDINGS = 10_000` cap (in `src/reassembly/mod.rs:57`) via the `TcpReassembler` lifecycle. ARP bypasses reassembly entirely (it is a link-layer protocol). The ARP findings Vec is unbounded.

**The security implication:** A malicious pcap with millions of ARP spoof events could cause unbounded findings Vec growth. For a CLI forensics tool this is an accepted design choice (users own their pcap files), but it must be documented.

### 2. Governing Behavioral Contract

**BC-2.16.010** (ArpAnalyzer::summarize 11 Keys) and **BC-2.16.011** (--arp CLI flag gates ARP analysis) are the most directly relevant. Neither currently specifies findings-cap behavior.

**BC-2.16.006** specifies the binding table cap (MAX_ARP_BINDINGS). There is NO equivalent BC for an output findings cap on the ARP path — because no such cap exists.

### 3. Fix Classification

**(c) Documentation + spec clarification (no code change).** The fix is twofold:

1. **Add `///` doc-comment to `--arp` in `src/cli.rs`** clarifying that ARP findings are unbounded (unlike HTTP/TLS/Modbus which apply a 10,000-finding cap via the reassembly layer). Operator note: on adversarial pcaps with massive ARP storms, findings Vec may grow without bound.

2. **Add invariant to BC-2.16.010 or BC-2.16.011** (or a new BC-2.16.016) explicitly stating: *"ARP findings are NOT capped by `MAX_FINDINGS`. Unlike stream-reassembly analyzers (HTTP, TLS, Modbus, DNP3) which bound output via the reassembly `MAX_FINDINGS = 10,000` cap, `ArpAnalyzer::process_arp` appends all findings to the return Vec without bound. The binding table is capped at MAX_ARP_BINDINGS (65,536); storm counters are capped at MAX_STORM_COUNTERS (4,096); but the findings output is unbounded. This is intentional: ARP operates at link-layer and bypasses the reassembly path entirely (BC-2.16.015 Invariant 2). Operators analyzing adversarial captures should be aware that ARP finding counts can equal the number of ARP frames in the capture."*

**Option evaluation — add a cap vs. document no-cap:**
The maintenance sweep note (PC-015 in tech-debt-register) flagged that "security auditors cannot distinguish intentional behavior from missing implementation." The correct fix is documentation, NOT silently adding a MAX_FINDINGS cap — adding a cap would be a behavioral change requiring a new BC, new tests, and potentially a semver bump (since users may rely on full findings). Documentation is the minimal-change path consistent with the QUIESCED pipeline state.

**Proposed BC change:** Version bump BC-2.16.010 or BC-2.16.011 to add an explicit "no findings cap" invariant. Alternatively, create **BC-2.16.016** ("ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on process_arp Return Vec") as a greenfield BC to keep concerns separated. A new BC is preferred because: (a) it preserves append-only numbering; (b) it provides a dedicated test target for the absence of a cap; (c) it is directly testable.

### 4. Per-Item Test Strategy (Red Gate)

For a doc-only fix, the "test" is a doc/help-text assertion:

```rust
// test_BC_2_16_016_arp_help_text_mentions_unbounded_findings (or doc-test)
// Verify the --arp flag long_help string contains the phrase "unbounded" or
// "no cap" or similar. This is a Clap #[arg(long_help = "...")] string test.
// Currently FAILS because the help text has no such mention.
//
// Alternative: An integration test that processes N ARP frames (N > 10_000)
// and asserts that findings.len() == N (no cap applied). This is the strongest
// Red Gate — it would fail if a cap were accidentally added.
```

The second form (behavioral assertion that no cap is applied) is the stronger Red Gate and is recommended. It doubles as a DoS-awareness regression test.

**Proposed test name:** `test_BC_2_16_016_arp_findings_vec_has_no_cap`

Location: `src/analyzer/arp.rs` tests block or `tests/bc_2_16_story113_arp_tests.rs`.

### 5. Regression Risk & Blast Radius

- **Blast radius: MINIMAL.** CLI doc comment additions only + BC text update. No code logic changes.
- **Tests at risk:** None (no behavioral change).
- **VP-024 at risk:** Not affected.
- **If a cap IS added (not recommended for this bundle):** would be a breaking change requiring Modbus-pattern implementation, a `dropped_findings` counter in ARP summarize (11 keys → 12 keys, BC-2.16.010 breaking change), new VP, and new holdout scenarios. Out of scope for this defect-fix bundle.

---

## Recommended Delivery Order

### Rationale

PC-015 must be spec-authored first because it establishes the "no-cap is intentional" invariant that protects PC-013's graceful-skip fix from being misread as an accidental gap. PC-013 requires a BC version bump before the Red Gate test is written. PC-014 is independent but is last because it is the most breaking (key rename affecting JSON consumers).

```
ORDER 1: PC-015 (spec/doc) — author BC-2.16.016 + update BC-2.16.010 or BC-2.16.011
ORDER 2: PC-013 (code) — replace .expect() sites, version-bump BC-2.16.004
ORDER 3: PC-014 (code + snapshot) — rename key, version-bump BC-2.15.020
```

All three items are small enough to deliver in a single PR if the team prefers, since they touch disjoint files. However, sequential delivery is safer because each has an independent Red Gate test.

### Suggested Branch Names

```
fix/arp-expect-panic-sites      # PC-013
fix/dnp3-parse-errors-key-name  # PC-014
docs/arp-no-findings-cap        # PC-015
```

---

## Required BC / AC Additions or Edits

### New BC

| Proposed ID | Title | Action |
|-------------|-------|--------|
| BC-2.16.016 | ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on `process_arp` Return Vec | NEW (greenfield). Specifies: no cap on findings Vec, no `dropped_findings` counter, intentional design (link-layer bypasses reassembly MAX_FINDINGS). Test: `test_BC_2_16_016_arp_findings_vec_has_no_cap`. |

### BC Version Bumps (no new behavior — clarification / rename only)

| BC | Current Version | Required Change |
|----|----------------|-----------------|
| BC-2.16.004 (D1 ARP Spoof) | v1.8 | Add invariant: "If the binding-table entry for `sender_ip` is absent at any invariant-enforcing site in `process_arp`, the operation is silently skipped (fail-safe degradation, no panic)." → v1.9 |
| BC-2.16.010 (summarize 11 Keys) | v1.7 | Cross-reference BC-2.16.016 in Related BCs. Add note: "findings output (from process_arp) is unbounded — BC-2.16.016". → v1.8 |
| BC-2.15.020 (DNP3 summarize) | v1.3 | Rename `total_parse_errors` → `parse_errors` in Postcondition 1 line 57 and all test vectors. → v1.4 |

### No New VPs Required

- PC-013: The graceful-skip is a code pattern change, not a new formally-verifiable pure-core property. The `.expect()` sites are in `process_arp` (effectful), not in the Kani-verified pure-core functions. No VP needed.
- PC-014: Key rename only. No VP.
- PC-015: Documentation. VP would be overkill. The behavioral test (`test_BC_2_16_016_arp_findings_vec_has_no_cap`) is sufficient.

---

## Appendix: Key Files Referenced

| Path | Relevance |
|------|-----------|
| `src/analyzer/arp.rs` | PC-013 fix targets: lines 555, 576, 642, 827 |
| `src/analyzer/dnp3.rs` | PC-014 fix target: line 1425 |
| `src/cli.rs` | PC-015 doc target: lines 194–198 (`--arp` arg doc comment) |
| `tests/dnp3_detection_tests.rs` | PC-014 test updates: lines 995, 1378 |
| `tests/bc_2_15_110_dnp3_dispatcher_tests.rs` | PC-014 test review: line 959 (local variable, not key string — no update needed if it is a variable assignment) |
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.004.md` | PC-013 BC update |
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md` | PC-015 BC cross-ref |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.020.md` | PC-014 BC update |
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.016.md` | PC-015 NEW BC (to be created) |
| `.factory/tech-debt-register.md` | Source register: close PC-013, PC-014, PC-015 on delivery |
| `.factory/STATE.md` | Close open items PC-013, PC-014, PC-015 on delivery |
