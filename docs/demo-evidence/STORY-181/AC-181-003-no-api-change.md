# AC-181-003: No Public API Surface Change

**AC:** AC-181-003  
**Story:** STORY-181 (SEC-001 ENIP unsafe split-borrow refactor)  
**Date:** 2026-07-24  
**Branch:** feature/STORY-181-enip-sec001-split-borrow

---

## Verdict: PASS

---

## process_pdu Signature Unchanged

Command:
```
grep -n "fn process_pdu" src/analyzer/enip.rs
```

Output:
```
1032:    pub fn process_pdu(
```

Current signature at `src/analyzer/enip.rs` lines 1032–1037:
```rust
    pub fn process_pdu(
        &mut self,
        flow: &mut EnipFlowState,
        pdu: &[u8],
        timestamp: u32,
        src_ip: IpAddr,
    ) {
```

This is identical to the pre-refactor signature. The fix is internal to the `on_data`
function body; no `pub` or `pub(crate)` signatures were changed.

---

## git diff Stat: Only enip.rs, bin/, CHANGELOG Touched

Command:
```
git diff 421bf572..HEAD --stat
```

Output:
```
 CHANGELOG.md           | 23 +++++++++++++++++++++++
 bin/validate-citations |  8 ++++----
 src/analyzer/enip.rs   | 45 ++++++++++++++++++++++++---------------------
 3 files changed, 51 insertions(+), 25 deletions(-)
```

Exactly three files:
- `src/analyzer/enip.rs` — the refactored implementation (SEC-001 fix)
- `bin/validate-citations` — the AC-181-004 docstring update (advisory)
- `CHANGELOG.md` — required [Unreleased] entry (PG-W71-CHANGELOG)

`Cargo.toml` is **not** in the diff. No new crate dependencies. No Cargo.toml changes.
The refactor scope is implementation-internal to `on_data` only.
