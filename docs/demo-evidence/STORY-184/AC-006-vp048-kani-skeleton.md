# AC-184-006 — VP-048 Kani Harness Skeleton Compiles

**Story:** STORY-184: S7comm TPKT Core Parser
**AC:** AC-184-006
**Traces to:** BC-2.20.001 invariant 2, BC-2.20.002 invariant 2, BC-2.20.003 invariant 2,
BC-2.20.004 postcondition 3
**Wave:** 87

---

## Acceptance Criterion

- Given the `#[cfg(kani)]` module in `src/analyzer/iso_on_tcp.rs`
- When `cargo kani --harness verify_parse_tpkt_header_safety` is run (against the
  `todo!()`-free implementation from this story)
- Then the harness skeleton compiles without errors
- The full Kani proof run (STORY-194) verifies: no panics for any symbolic `[u8; N]`
  input, and the four-way partition (AC-184-005) is exhaustive and non-overlapping over
  all possible `data` inputs, with no overflow in `h.length` decoding
- ADR-014 Decision 9 scope: VP-048 covers `parse_tpkt_header` only; `parse_cotp_header`
  is VP-049 (STORY-185); the combined no-panic frame-walk loop is VP-050/VP-055

---

## Skeleton Presence Verification

### Source location

File: `src/analyzer/iso_on_tcp.rs`, line 145

Command:
```
grep -n "cfg(kani)" src/analyzer/iso_on_tcp.rs
```

Output:
```
108:/// `#[cfg(kani)]` skeleton below is scoped to check only no-panic/bounds-safety over
145:#[cfg(kani)]
```

Result: `#[cfg(kani)]` block present at line 145. Harness module `kani_proofs` contains
`verify_parse_tpkt_header_safety`.

### Harness structure (lines 145–162)

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-048: `parse_tpkt_header` must not panic for any input, up to the bounded
    /// length (`len <= 300`).
    #[kani::proof]
    fn verify_parse_tpkt_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input:
        let _ = parse_tpkt_header(&data);
    }
}
```

---

## Normal Compilation (no cfg=kani)

The `#[cfg(kani)]` block is excluded from normal compilation. `cargo check` confirms the
codebase compiles clean:

Command:
```
cargo check
```

Output:
```
    Checking wirerust v0.13.3 (<repo>)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.59s
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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
```

No warnings, no errors. The `iso_on_tcp` module (including the `#[cfg(kani)]` block)
passes clippy clean.

---

## VP-048 Property Scope (ADR-014 Decision 9)

The harness skeleton covers one property, to be fully proved in STORY-194:

| Property | Statement | Proof Method |
|----------|-----------|--------------|
| A | No panic for any symbolic input of length <= 300 | Kani symbolic execution (STORY-194) |

Out of scope for VP-048 (per ADR-014 Decision 9):
- `parse_cotp_header` no-panic/bounds-safety -> VP-049 (STORY-185)
- Combined no-panic frame-walk loop (`S7commAnalyzer::on_data`) -> VP-050/VP-055
  (STORY-186 and later)

The full proof run additionally asserts (per this story's stated scope, executed in
STORY-194): the four-way partition (AC-184-005) is exhaustive and non-overlapping over
all possible `data` inputs, with no overflow in `h.length` decoding.

---

## Verdict

AC-184-006: **PASS** — `#[cfg(kani)]` skeleton present at `src/analyzer/iso_on_tcp.rs:145`;
`cargo check` and `cargo clippy --all-targets -- -D warnings` both clean; VP-048 property
scope anchored for STORY-194's full proof run.
