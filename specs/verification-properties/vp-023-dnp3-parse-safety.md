---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-06-10T00:00:00Z
phase: f2
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.15.001
bcs:
  - BC-2.15.001
  - BC-2.15.002
  - BC-2.15.003
  - BC-2.15.004
  - BC-2.15.005
  - BC-2.15.006
  - BC-2.15.007
module: src/analyzer/dnp3.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
verified_at_commit: null
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.0: Authored in Phase-F2 spec evolution for issue #8 (DNP3 TCP analyzer). Pre-registered by architect in VP-INDEX/verification-architecture/verification-coverage-matrix (Kani, P1, total→23, kani→10). Four Kani sub-properties (A header safety, B FC classification totality + correctness, C validity gate biconditional, D frame_len arithmetic). Status=draft; harnesses authored in F4 TDD."
  - "v1.1: Pass-1 adversarial remediation (issue #8): Sub-property C prose corrected from slice-based is_valid_dnp3_frame(data: &[u8]) with 4 conditions to struct-based is_valid_dnp3_frame_header(h: &Dnp3DlHeader) with 3 conditions (start1==0x05, start2==0x64, length>=5), matching BC-2.15.004 and the verify_is_valid_dnp3_frame_gate harness. No change to harness or other sub-properties."
  - "v1.2: Pass-2 adversarial remediation LOW-1 (issue #8): Related-BC note for BC-2.15.004 corrected from 'LENGTH in 5..=255' to 'LENGTH >= 5', aligning with BC-2.15.004 phrasing and harness Sub-property C biconditional (h.length >= 5). The upper bound 255 is a structural u8 constraint, not a gate condition. No change to property statement or harnesses."
  - "v1.3: Corrected introduced: field from v0.5.0-feature-008 to v0.6.0-feature-008. v0.5.0 shipped the MITRE fix; DNP3 TCP analyzer targets v0.6.0, matching all 24 SS-15 BC files. No change to property statement or harnesses."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-023: DNP3 Data-Link Frame Parse Safety and Function-Code Classification

## Property Statement

The DNP3 pure-core functions in `src/analyzer/dnp3.rs` are memory-safe, panic-free, and
total over their bounded symbolic input domains. The property decomposes into four Kani
sub-properties over the pure-core functions specified in the F2 architecture delta.

**Sub-property A — Data-link header parse safety** (anchors BC-2.15.001 / BC-2.15.002 / BC-2.15.003):

For any byte slice `data: &[u8]` of bounded length:

1. `parse_dnp3_dl_header(data)` returns `None` when `data.len() < 10` (minimum
   complete header = 8 header bytes + 2 header-CRC bytes).
2. `parse_dnp3_dl_header(data)` returns `Some(Dnp3DlHeader { .. })` when
   `data.len() >= 10`, with fields decoded from fixed offsets:
   - `start1 = data[0]`, `start2 = data[1]`
   - `length = data[2]`
   - `control = data[3]`
   - `destination = u16::from_le_bytes([data[4], data[5]])` (little-endian [SPEC])
   - `source = u16::from_le_bytes([data[6], data[7]])` (little-endian [SPEC])
   - (bytes 8–9 = header CRC, not decoded as struct fields in v1)
3. The function NEVER panics for any input (no out-of-bounds indexing). The `data.len() < 10`
   early return guarantees every index `data[0..10]` is in bounds on the `Some` path.

**Sub-property B — Application function-code classification totality and correctness**
(anchors BC-2.15.005 / BC-2.15.006):

For any `fc: u8` (all 256 values), `classify_dnp3_fc(fc)` returns exactly one defined
`Dnp3FcClass` variant from `{Read, Write, Control, Restart, Management, Response, Unknown}`.
The match is exhaustive by construction (a `_ => Unknown` wildcard arm); there is no
`unreachable!`, no gap, and no panic. Set membership holds:

- Read set: `{0x01}` → `Read`
- Write set: `{0x02}` → `Write`
- Control set: `{0x03, 0x04, 0x05, 0x06}` (SELECT, OPERATE, DIRECT_OPERATE, DIRECT_OPERATE_NR) → `Control`
- Restart set: `{0x0D, 0x0E}` (COLD_RESTART, WARM_RESTART) → `Restart`
- Management set: `{0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A}` → `Management`
- Response set: `{0x81, 0x82, 0x83}` (RESPONSE, UNSOLICITED_RESPONSE, AUTHENTICATE_RESP) → `Response`
- All other `fc` values not in the above sets → `Unknown`

**Sub-property C — Three-condition validity gate biconditional** (anchors BC-2.15.004):

The `is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` gate returns `true` if and only if
all three conditions hold on the already-parsed struct:

1. `h.start1 == 0x05` (first sync byte)
2. `h.start2 == 0x64` (second sync byte; together the pair is the 0x0564 sync word)
3. `h.length >= 5` (LENGTH minimum per DNP3 spec)

The function operates on a `Dnp3DlHeader` value (output of `parse_dnp3_dl_header`), not on a
raw byte slice; there is no length check and no indexing inside this function. It reads only
struct fields and never panics. The biconditional holds: `is_valid_dnp3_frame_header(h)` is
`true` if and only if `h.start1 == 0x05 && h.start2 == 0x64 && h.length >= 5`.

**Sub-property D — frame_len arithmetic correctness** (anchors BC-2.15.007):

The `compute_dnp3_frame_len(length: u8) -> Option<usize>` function:

1. Returns `None` when `length < 5` (invalid frame — LENGTH below minimum).
2. For all `length` in `5..=255`, computes and returns:
   ```
   num_user_octets = (length as usize) - 5
   num_data_blocks = (num_user_octets + 15) / 16   // integer ceil(U/16)
   frame_len       = 5 + (length as usize) + 2 * num_data_blocks
   ```
3. The result satisfies:
   - `frame_len >= 10` for all `length >= 5` (minimum frame is LENGTH=5 → frame_len=10)
   - `frame_len <= 292` for all `length <= 255` (maximum frame is LENGTH=255 → frame_len=292)
4. The arithmetic never overflows `usize` (given `usize >= 16 bits` which is a Rust
   platform guarantee; the maximum value 292 fits in any realistic `usize`).
5. `compute_dnp3_frame_len` NEVER panics on any `u8` input.

This sub-property ensures the frame-consumption loop in `Dnp3Analyzer::on_data` never
over-reads (indexes beyond the carry buffer) or under-reads (leaves part of a frame
unconsumed) by proving the formula is correct over the full LENGTH domain.

### Sub-property → BC anchor table (canonical map)

| Sub-property | Concern | Anchored BCs |
|--------------|---------|--------------|
| A — DL header parse safety | `parse_dnp3_dl_header` panic/OOB-freedom, `None` iff `len<10`, LE field decode | BC-2.15.001, BC-2.15.002, BC-2.15.003 |
| B — FC classification totality + correctness | match totality (no gap/panic), Read/Write/Control/Restart set membership | BC-2.15.005, BC-2.15.006 |
| C — validity gate biconditional | gate true iff sync==0x0564 && LENGTH>=5 | BC-2.15.004 |
| D — frame_len arithmetic | formula correctness, 10..=292 range, no overflow/panic | BC-2.15.007 |

Union of anchored BCs = {001, 002, 003, 004, 005, 006, 007} = the 7-BC frontmatter `bcs:`
set (7 BCs; contrast VP-022's 8 BCs — DNP3 collapses exception detection into the
Response class rather than a biconditional, so no separate exception sub-property).

> **SPEC-level document.** This VP defines *what must be proven*. The Kani harnesses are
> authored in F4 TDD against the implemented `src/analyzer/dnp3.rs`. At F4/F6 lock time
> the formal-verifier sets `verification_lock: true`, `proof_completed_date`,
> `proof_file_hash`, and `status: verified`, and creates the `vp-verified-VP-023-<YYYY-MM-DD>`
> tag. Until then this document is mutable (`verification_lock: false`).

## Source Contract

- **Primary BC:** BC-2.15.001 — DNP3 data-link header accepted for well-formed 10-byte-minimum frame
- **Postcondition:** `parse_dnp3_dl_header` never panics; `None` iff `len < 10`; `Some(well-formed)` otherwise
- **Related BC:** BC-2.15.002 — DL header rejected for frame shorter than 10 bytes (truncation safety)
- **Related BC:** BC-2.15.003 — DEST/SOURCE addresses decoded little-endian from fixed offsets 4–7
- **Related BC:** BC-2.15.004 — Three-point validity gate: true iff sync==0x0564 and LENGTH >= 5
- **Related BC:** BC-2.15.005 — `classify_dnp3_fc` is total over all 256 FC values
- **Related BC:** BC-2.15.006 — FC classification correctness: Control set {0x03,0x04,0x05,0x06}, Restart set {0x0D,0x0E}, Write set {0x02}, Read set {0x01}
- **Related BC:** BC-2.15.007 — `compute_dnp3_frame_len` arithmetic correct over LENGTH domain 5..=255; result in [10, 292]; no overflow/panic
- **ADR:** ADR-007 (DNP3 TCP integration), Decision 2 (frame_len formula), Decision 3 (CRC-block-walk), Decision 4 (FIR=1 parse)
- **Architecture:** `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md` §2 (purity boundary), §3 (frame math)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes — sub-A: symbolic `[u8; N]` + symbolic `len <= 12`; sub-B/C: symbolic `u8` (all 256 values); sub-D: symbolic `u8` length (all 256 values) | All parse outcomes for `data.len()` 0..=12; full FC domain 0x00..=0xFF; full LENGTH domain 0x00..=0xFF |

Kani is the primary and sole counted tool for VP-023 (per VP-INDEX: Kani 10 after this
addition). All four pure-core functions have no heap allocation, no I/O, and no
`HashMap`, so they are ideal Kani targets with trivially bounded state spaces. The
`HashMap` in `Dnp3FlowState` is in the effectful `on_data` shell and is NOT targeted by
these harnesses (same pattern as VP-022 for Modbus).

## Proof Harness Skeleton

> Harnesses live in `src/analyzer/dnp3.rs` under
> `#[cfg(kani)] mod kani_proofs { use super::*; }`, mirroring the convention established
> by VP-022 (`src/analyzer/modbus.rs`). Pure-core functions are free `fn`s (not
> `impl Dnp3Analyzer` methods), so harnesses call them directly without constructing the
> analyzer struct. The harnesses below are SPEC skeletons; exact wiring is finalized in
> F4 against the real signatures.

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- Sub-property A: parse_dnp3_dl_header safety (BC-2.15.001/002/003) ----
    //
    // MAX_LEN = 12 covers: the len<10 reject band (0..=9), the minimum accept
    // (len==10), and lengths with a couple of user bytes visible (11..=12) to
    // ensure sub-B/C paths remain representable. No allocation, no loop.
    const MAX_LEN: usize = 12;

    #[kani::proof]
    fn verify_parse_dnp3_dl_header_safety() {
        let buf: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let data = &buf[..len];

        // (A.3) No panic / no OOB: calling over the symbolic slice proves
        // absence of out-of-bounds indexing for every length 0..=12.
        let parsed = parse_dnp3_dl_header(data);

        // (A.1) len<10 => None ; (A.2) len>=10 => Some.
        if len < 10 {
            assert!(parsed.is_none());
        } else {
            let h = parsed.expect("len>=10 must parse to Some");
            // (A.2) field decode correctness.
            assert!(h.start1       == data[0]);
            assert!(h.start2       == data[1]);
            assert!(h.length       == data[2]);
            assert!(h.control      == data[3]);
            // Little-endian DEST/SOURCE (BC-2.15.003).
            assert!(h.destination  == u16::from_le_bytes([data[4], data[5]]));
            assert!(h.source       == u16::from_le_bytes([data[6], data[7]]));
        }
    }

    // ---- Sub-property C: validity gate biconditional (BC-2.15.004) ----
    #[kani::proof]
    fn verify_is_valid_dnp3_frame_gate() {
        let h = Dnp3DlHeader {
            start1:      kani::any(),
            start2:      kani::any(),
            length:      kani::any(),
            control:     kani::any(),
            destination: kani::any(),
            source:      kani::any(),
        };
        let ok = is_valid_dnp3_frame_header(&h);
        // Gate is true IFF sync matches AND LENGTH >= 5.
        assert!(ok == (h.start1 == 0x05 && h.start2 == 0x64 && h.length >= 5));
    }

    // ---- Sub-property B: classify_dnp3_fc totality + set membership (BC-2.15.005/006) ----
    //
    // Symbolic input: a single u8 (all 256 values). The match is exhaustive by
    // construction; "no panic" + a returned variant proves totality.
    #[kani::proof]
    fn verify_classify_dnp3_fc_total() {
        let fc: u8 = kani::any();
        let class = classify_dnp3_fc(fc); // must return for every u8

        // Read set (BC-2.15.006).
        if matches!(fc, 0x01) {
            assert!(class == Dnp3FcClass::Read);
        }
        // Write set (BC-2.15.006).
        if matches!(fc, 0x02) {
            assert!(class == Dnp3FcClass::Write);
        }
        // Control set (BC-2.15.006 — SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR).
        if matches!(fc, 0x03 | 0x04 | 0x05 | 0x06) {
            assert!(class == Dnp3FcClass::Control);
        }
        // Restart set (BC-2.15.006 — COLD_RESTART/WARM_RESTART).
        if matches!(fc, 0x0D | 0x0E) {
            assert!(class == Dnp3FcClass::Restart);
        }
        // Response set (BC-2.15.006).
        if matches!(fc, 0x81 | 0x82 | 0x83) {
            assert!(class == Dnp3FcClass::Response);
        }
        // Totality witness: returned value is one of the defined variants.
        assert!(matches!(
            class,
            Dnp3FcClass::Read
                | Dnp3FcClass::Write
                | Dnp3FcClass::Control
                | Dnp3FcClass::Restart
                | Dnp3FcClass::Management
                | Dnp3FcClass::Response
                | Dnp3FcClass::Unknown
        ));
    }

    // ---- Sub-property D: frame_len arithmetic (BC-2.15.007) ----
    //
    // Symbolic input: a single u8 (all 256 LENGTH values).
    // Proves: None for length<5; correct formula for length>=5; result in [10,292].
    #[kani::proof]
    fn verify_compute_dnp3_frame_len() {
        let length: u8 = kani::any();
        let result = compute_dnp3_frame_len(length);

        if length < 5 {
            // (D.1) Below minimum: must return None.
            assert!(result.is_none());
        } else {
            // (D.2) Valid range: formula must hold and result in [10, 292].
            let fl = result.expect("length>=5 must return Some");
            let u = (length as usize) - 5;
            let blocks = (u + 15) / 16; // ceil(u / 16)
            let expected = 5 + (length as usize) + 2 * blocks;
            assert!(fl == expected);
            // (D.3) Bounds invariant.
            assert!(fl >= 10);
            assert!(fl <= 292);
        }
    }
}
```

### Symbolic input construction summary

| Sub-property | Harness | Symbolic input | Bound / unwind | Assertions |
|--------------|---------|----------------|----------------|------------|
| A (parse) | `verify_parse_dnp3_dl_header_safety` | `[u8; 12]` + symbolic `len <= 12`, sliced `&buf[..len]` | `kani::assume(len <= 12)`; no loop → no `#[kani::unwind]` | no panic; `None` iff `len<10`; LE field decode on `Some` |
| C (gate) | `verify_is_valid_dnp3_frame_gate` | symbolic `Dnp3DlHeader` fields | none (straight-line) | gate true iff `start1==0x05 && start2==0x64 && length>=5` |
| B (totality) | `verify_classify_dnp3_fc_total` | `fc: u8 = kani::any()` (256 values) | none (straight-line `match`) | no panic; Read/Write/Control/Restart/Response set membership; returns a defined variant |
| D (frame_len) | `verify_compute_dnp3_frame_len` | `length: u8 = kani::any()` (256 values) | none (straight-line arithmetic) | `None` iff `length<5`; formula correct; result in `[10, 292]` |

**Loop / unwind bounds:** none of the four harnesses contains a user-visible loop
(the integer arithmetic in Sub-D is straight-line), so no `#[kani::unwind(N)]` attribute
is required. Sub-A bounds its state space via `kani::assume(len <= 12)`. Estimated total
Kani runtime: **< 1 second per harness** (analogous to VP-022, VP-005, VP-007).

**Expected result for all four harnesses:** `VERIFICATION:- SUCCESSFUL` (proof SUCCESSFUL).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Sub-A: slices of length 0..=12 (≤12 symbolic bytes); Sub-B/C/D: a single `u8` (256 values) |
| Proof complexity | Low | Four pure functions: a fixed-offset LE decode, a 3-clause boolean predicate, a `match` with wildcard, and straight-line integer arithmetic; no allocation, no FFI |
| Tool support | High | No `HashMap`/`RandomState` in pure core → no Kani-unsupported-FFI abort; ideal Kani targets per architecture-delta §2 purity boundary |
| Estimated proof time | < 1 second per harness | Comparable to VP-022 (Modbus, 4/4 SUCCESSFUL) and VP-005 (SNI) |
| Notable difference from VP-022 | frame_len arithmetic (Sub-D) | VP-022 had no arithmetic property; DNP3 adds the CRC-block formula proof. Straight-line integer arithmetic over a `u8` domain is a canonical Kani strength. |

## Source Location

- `src/analyzer/dnp3.rs` — `fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` (architecture-delta §2)
- `src/analyzer/dnp3.rs` — `fn is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` (architecture-delta §2, 3-point gate)
- `src/analyzer/dnp3.rs` — `fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass` (architecture-delta §3)
- `src/analyzer/dnp3.rs` — `fn compute_dnp3_frame_len(length: u8) -> Option<usize>` (architecture-delta §2, frame_len formula)
- `src/analyzer/dnp3.rs` — `struct Dnp3DlHeader { start1, start2, length, control, destination, source }`
- `src/analyzer/dnp3.rs` — `enum Dnp3FcClass { Read, Write, Control, Restart, Management, Response, Unknown }`

(These source paths are forward references — the module is implemented in F4 TDD. Exact
line numbers are filled in when harnesses are authored and the VP is locked.)

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created (draft, F2 spec evolution) | 2026-06-10 | architect |
| Proof harness to be committed | F4 TDD | formal-verifier |
| Proof to be verified | F6 formal hardening | formal-verifier |
| Lock (VERIFIED) | F6 gate | formal-verifier |
