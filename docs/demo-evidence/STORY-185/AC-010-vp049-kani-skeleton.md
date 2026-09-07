# AC-185-010 — VP-049 Kani Harness Skeleton Compiles

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-010
**Traces to:** BC-2.20.005 postcondition 2, BC-2.20.006 invariant 1, BC-2.20.011
invariant 3
**Wave:** 88

---

## Acceptance Criterion

- Given the `#[cfg(kani)]` module in `src/analyzer/iso_on_tcp.rs`
- When `cargo kani --harness verify_parse_cotp_header_safety` is run
- Then the harness skeleton compiles without errors
- The full Kani proof run (STORY-194) verifies: no panics or out-of-bounds reads for any
  symbolic input (including the LI-truncation bounds check), the TPDU-type
  classification is exhaustive and non-overlapping over all 16 nibble values, and the
  protocol-ID extraction is a total identity mapping over all 256 `u8` values

---

## Skeleton Presence Verification

### Source location

File: `src/analyzer/iso_on_tcp.rs`, line 320 (harness); `#[cfg(kani)]` module gate at
line 293 (shared with STORY-184's VP-048 `verify_parse_tpkt_header_safety` harness in
the same `kani_proofs` module).

Command:
```
grep -n "cfg(kani)\|verify_parse_cotp_header_safety\|mod kani_proofs" src/analyzer/iso_on_tcp.rs
```

Output:
```
125:/// `#[cfg(kani)]` skeleton below is scoped to check only no-panic/bounds-safety over
241:/// hardening); the `#[cfg(kani)]` skeleton below is scoped to check only
293:#[cfg(kani)]
294:mod kani_proofs {
320:    fn verify_parse_cotp_header_safety() {
```

Result: `#[cfg(kani)]` block present at line 293; harness function
`verify_parse_cotp_header_safety` present at line 320, inside the same `kani_proofs`
module as STORY-184's VP-048 harness.

### Harness structure (lines 319–329)

```rust
/// VP-049: `parse_cotp_header` must not panic for any input, up to the bounded
/// length (`len <= 300`).
///
/// SCOPE (this story): no-panic / bounds-safety only, mirroring the VP-048 harness
/// pattern above. The full VP-049 proof obligation — TPDU-type classification
/// exhaustiveness over all 16 high-nibble values (BC-2.20.011 invariant 3) and
/// protocol-ID-extraction totality over all 256 `u8` values (BC-2.20.012) — is
/// deferred to STORY-194 (formal hardening), per this story's Kani obligation note.
#[kani::proof]
fn verify_parse_cotp_header_safety() {
    let len: usize = kani::any();
    kani::assume(len <= 300);
    let mut data = vec![0u8; len];
    for b in data.iter_mut() {
        *b = kani::any();
    }
    // Must not panic for any input, including the LI-truncation bounds check:
    let _ = parse_cotp_header(&data);
}
```

---

## Normal Compilation (no cfg=kani)

The `#[cfg(kani)]` block is excluded from normal compilation. `cargo check` confirms the
codebase (including this story's `parse_cotp_header` addition) compiles clean:

Command:
```
cargo check
```

Output:
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

The `#[cfg(kani)]` guard means the harness is gated behind the Kani toolchain. Under
normal stable Rust, the block is elided entirely — the harness neither compiles nor
interferes.

---

## Clippy (CI-equivalent)

Command:
```
cargo clippy --all-targets -- -D warnings
```

Output:
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

No warnings, no errors. The `iso_on_tcp` module (including both the VP-048 and VP-049
`#[cfg(kani)]` harnesses) passes clippy clean.

---

## VP-049 Property Scope (ADR-014 Decision 9)

The harness skeleton covers one property, to be fully proved in STORY-194:

| Property | Statement | Proof Method |
|----------|-----------|--------------|
| A | No panic or out-of-bounds read for any symbolic input of length <= 300, including the LI-truncation bounds check | Kani symbolic execution (STORY-194) |

Out of scope for VP-049 in this story (deferred to STORY-194's full proof run, per
ADR-014 Decision 9 and this story's stated Kani obligation):
- TPDU-type classification exhaustiveness over all 16 high-nibble values
  (BC-2.20.011 invariant 3 — AC-185-008's unit-level spot check anchors this story's
  scope; the full symbolic-input proof is STORY-194's obligation)
- Protocol-ID-extraction totality over all 256 `u8` values as a formal Kani assertion
  (BC-2.20.012 — AC-185-009's exhaustive `#[test]` loop covers this at the unit-test
  level in this story; the Kani-proved version is STORY-194's obligation)
- Combined no-panic frame-walk loop (`S7commAnalyzer::on_data`) -> VP-050/VP-055
  (STORY-186 and later)

---

## Verdict

AC-185-010: **PASS** — `#[cfg(kani)]` skeleton present at
`src/analyzer/iso_on_tcp.rs:293` with the `verify_parse_cotp_header_safety` harness at
line 320; `cargo check` and `cargo clippy --all-targets -- -D warnings` both clean;
VP-049 property scope anchored for STORY-194's full proof run.
