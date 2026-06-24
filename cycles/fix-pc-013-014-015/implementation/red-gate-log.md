---
document_type: red-gate-log
story: PC-013 (fix-pc-013-014-015)
bc: BC-2.16.004 v1.9 Invariant 6
phase: test-writer
timestamp: 2026-06-23
branch: fix/arp-expect-failsafe-guards
worktree: .worktrees/fix-pc-013
---

# Red Gate Log — PC-013 / BC-2.16.004 v1.9 Invariant 6

## Test File

`src/analyzer/arp.rs` — `mod bc_2_16_004_inv6` (appended after the existing
`mod bc_2_16_016` module at the bottom of the file, line ~4562 onward).

## Tests Written

| Test Function | `.expect()` Site(s) Covered | EC | Status |
|---|---|---|---|
| `test_BC_2_16_004_expect_site_no_panic_on_missing_entry` | Lines 555, 576 (GARP-conflict path, primary test) | EC-011 | PASS |
| `test_BC_2_16_004_expect_site_no_panic_garp_conflict_high_escalation` | Lines 555, 576 (GARP-conflict path, HIGH escalation variant) | EC-011 | PASS |
| `test_BC_2_16_004_expect_site_no_panic_emit_d1_first_rebind_ts_none` | Line 827 (`first_rebind_ts.expect`) — direct call | EC-012 | PASS |
| `test_BC_2_16_004_expect_site_no_panic_emit_d1_after_flap_window_reset` | Line 827 (via flap-window reset clearing first_rebind_ts) | EC-012 | PASS |
| `test_BC_2_16_004_expect_site_no_panic_non_garp_rebind_step4_reborrow` | Line 642 (non-GARP rebind Step 4 re-borrow) | EC-011 | PASS |

`cargo test bc_2_16_004_inv6` result: **5 passed; 0 failed**.

## Red Gate Status: NOT ACHIEVED (by design — invariants unbreachable)

### Why the `.expect()` sites cannot be made to panic in safe Rust

All four `.expect()` sites in `process_arp` and `emit_d1_spoof_finding_impl`
protect invariants that hold in single-threaded safe Rust code and **cannot be
violated by any external or white-box input**:

**Lines 555 and 576 (GARP-conflict path):**

The pattern is:

```rust
let has_conflict = self.bindings.get(&sender_ip)
    .map(|e| e.mac != sender_mac)
    .unwrap_or(false);

if has_conflict {
    let entry = self.bindings.get_mut(&sender_ip)
        .expect("has_conflict implies entry exists");  // line 555
    ...
    let entry = self.bindings.get_mut(&sender_ip)
        .expect("entry must still exist");  // line 576
}
```

Both `get` (for `has_conflict`) and `get_mut` (line 555) execute within the
same `process_arp` invocation.  There is no interleaving point at which a test
body could remove the entry from the HashMap between the two accesses.
Unreachable in single-threaded safe Rust.

**Line 642 (non-GARP rebind Step 4):**

```rust
if let Some(entry) = self.bindings.get_mut(&sender_ip) {
    ...
    let entry = self.bindings.get_mut(&sender_ip)
        .expect("entry must still exist");  // line 642
}
```

The entry is confirmed present by the `if let Some(entry)` guard at the top of
the block.  The second `get_mut` executes in the same invocation with no removal
opportunity.  Unreachable.

**Line 827 (`first_rebind_ts.expect`):**

```rust
// Step 2: set first_rebind_ts if currently None
if entry.first_rebind_ts.is_none() {
    entry.first_rebind_ts = Some(timestamp_secs);
}
// Step 3: evaluate HIGH vs MEDIUM
let first_ts = entry.first_rebind_ts.expect("set in Step 2");  // line 827
```

Step 2 unconditionally sets `first_rebind_ts` if it is `None` before Step 3's
`.expect()` runs.  Even after a flap-window reset (which clears the field to
`None`), Step 2 immediately sets it again.  There is no input state that makes
`first_rebind_ts` `None` at the point Step 3 runs.  Unreachable.

### Scope.md acknowledgment

`fix-pc-013-014-015/scope.md §4` states: *"Since the `.expect()` calls are on
logically-unbreachable invariants, there is no test that currently triggers a
panic."*  This log confirms that conclusion from first principles.

### What the tests DO verify (behavioral characterization)

The 5 tests in `mod bc_2_16_004_inv6` verify the **current behavior** around
each `.expect()` site:

- **EC-011 (lines 555, 576):** GARP-conflict path emits exactly 2 findings
  (D2 MEDIUM + D1 MEDIUM or HIGH), MAC is updated via Step 4 (line 576 path),
  no panic.
- **EC-012 (line 827):** `emit_d1_spoof_finding_impl` called directly with
  `first_rebind_ts = None` returns a valid MEDIUM Finding; called with a
  flap-window-expired entry also returns a valid MEDIUM Finding with correct
  `rebind_count = 1` and `first_rebind_ts = Some(current_ts)`.
- **Line 642:** Non-GARP rebind path emits 1 D1 Finding, MAC updated, no panic.

After the fix (`.expect()` → `if let` guards), all 5 tests remain GREEN.  If
the fix accidentally drops a finding or changes severity, these tests will fail.
If a future refactor breaks the invariant (making the `None` reachable), the
tests will catch it as a behavior change (missing finding or wrong severity).

## `cargo fmt --check` and `cargo clippy -- -D warnings`

Both clean.  No lint warnings, no format diffs.

## Decision: No separate `.factory/cycles` path entry for Red Gate log

The `fix-pc-013-014-015` defect fix bundle does not have a dedicated Phase 3
cycle path in `.factory/cycles/` (there is no STORY-NNN.md for this fix — it is
a direct-to-code defect fix).  This log is placed at:

```
.factory/cycles/fix-pc-013-014-015/implementation/red-gate-log.md
```

analogous to the STORY-114 pattern in `feature-arp-v0.7.0/implementation/`.
