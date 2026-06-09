---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: formal-verifier
timestamp: 2026-06-09T00:00:00Z
phase: f2
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.14.001
bcs:
  - BC-2.14.001
  - BC-2.14.002
  - BC-2.14.003
  - BC-2.14.004
  - BC-2.14.005
  - BC-2.14.006
  - BC-2.14.007
  - BC-2.14.008
module: src/analyzer/modbus.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
verified_at_commit: null
lifecycle_status: active
introduced: v0.3.0-feature-007
modified:
  - "v1.0: Authored in Phase-F2 spec evolution for issue #7 (Modbus TCP analyzer). Pre-registered by architect in VP-INDEX/verification-architecture/verification-coverage-matrix (Kani, P1, total→22, kani→9). Three Kani sub-properties (A parse safety, B classify_fc totality, C exception biconditional). status=draft; harnesses authored in F4 TDD."
  - "v1.0 (F2 fix, consistency BLOCKING-1 / adversary F-MED-006): re-anchored sub-properties to the architect's canonical BC map (005=classify_fc totality, 006=exception detection, 007=Write-class, 008=Diagnostic-class). Sub-property B anchor corrected to BC-2.14.005/007/008; added explicit Sub-property→BC anchor table. Body unchanged in proof content; verification_lock stays false (draft)."
  - "v1.1 (F4/STORY-102 adversarial finding reconciliation): length upper bound corrected from 253 to 254 throughout (Sub-property A gate description, Source Contract BC-2.14.004 label, Kani harness assertion, and summary table) to match BC-2.14.004 authoritative value and the implemented is_valid_modbus_adu gate. Earlier V1 stale value of 253 was a pre-F2-fix residual. No structural change to proof logic."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-022: Modbus MBAP Parse Safety and Function-Code Boundary Classification

## Property Statement

The Modbus pure-core functions in `src/analyzer/modbus.rs` are memory-safe, panic-free,
and total over their bounded symbolic input domains. The property decomposes into three
Kani sub-properties over the architecture-delta §2.4–§2.5 pure-core functions:

**Sub-property A — MBAP parse safety** (`parse_mbap_header`, anchors BC-2.14.001/002/003/004):

For any byte slice `data: &[u8]` of bounded length:

1. `parse_mbap_header(data)` returns `None` when `data.len() < 8`.
2. `parse_mbap_header(data)` returns `Some(MbapHeader { .. })` when `data.len() >= 8`,
   with fields decoded big-endian from fixed offsets:
   `transaction_id = u16::from_be_bytes([data[0], data[1]])`,
   `protocol_id    = u16::from_be_bytes([data[2], data[3]])`,
   `length         = u16::from_be_bytes([data[4], data[5]])`,
   `unit_id        = data[6]`, `function_code = data[7]`.
3. The function NEVER panics for any input (no out-of-bounds indexing). The `data.len() < 8`
   early return guarantees every index `data[0..8]` is in bounds on the `Some` path.
4. The three-point validity gate `is_valid_modbus_adu(&h)` returns `true` if and only if
   `h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254`. It never panics; it reads
   only struct fields. (BC-2.14.003 = Protocol ID arm; BC-2.14.004 = Length arm.)

**Sub-property B — `classify_fc` totality + read/write/diagnostic membership**
(anchors BC-2.14.005/007/008 — canonical map: 005=`classify_fc` totality, 007=Write-class,
008=Diagnostic-class):

For any `fc: u8` (all 256 values), `classify_fc(fc)` returns exactly one defined
`FunctionCodeClass` variant from `{Read, Write, Diagnostic, Exception, Unknown}`. The match is
exhaustive by construction (the `_ => Unknown` wildcard arm); there is no `unreachable!`,
no gap, and no panic. Set membership holds:
- Read set `{0x01,0x02,0x03,0x04,0x07,0x0B,0x0C,0x11,0x14,0x18}` → `Read` (BC-2.14.005 post.2).
- Write set `{0x05,0x06,0x0F,0x10,0x15,0x16,0x17}` → `Write` (BC-2.14.005 post.3 / BC-2.14.007).
- Diagnostic set `{0x08,0x2B}` → `Diagnostic` (BC-2.14.005 post.4 / BC-2.14.008).
- All other `fc < 0x80` not in the above sets → `Unknown` (BC-2.14.005 post.6).

**Sub-property C — exception-detection biconditional** (anchors BC-2.14.006):

For any `fc: u8`, `classify_fc(fc) == FunctionCodeClass::Exception` **if and only if**
`fc >= 0x80`. Bidirectional: (a) every `fc >= 0x80` yields `Exception`; (b) no `fc < 0x80`
yields `Exception`. Additionally, the recovered original request FC is lossless:
`original_fc == fc & 0x7F` for all `fc` in `0x80..=0xFF` (the server sets the high bit of the
original FC byte, so masking reverses it exactly).

### Sub-property → BC anchor table (canonical map)

The canonical concept→BC map (per architect F2 directives §1, BC body H1s authoritative) is:
BC-2.14.005 = `classify_fc` totality over all 256 FC values; BC-2.14.006 = exception-response
detection (high-bit); BC-2.14.007 = Write-class FC classification; BC-2.14.008 = Diagnostic-class
FC classification. VP-022 sub-properties anchor under this map as follows:

| Sub-property | Concern | Anchored BCs |
|--------------|---------|--------------|
| A — MBAP parse safety + 3-point validity gate | `parse_mbap_header` panic/OOB-freedom, `None` iff `len<8`, BE field decode, gate biconditional | BC-2.14.001, BC-2.14.002, BC-2.14.003, BC-2.14.004 |
| B — `classify_fc` totality + read/write/diagnostic membership | match totality (no gap/panic), Read/Write/Diagnostic set membership | BC-2.14.005, BC-2.14.007, BC-2.14.008 |
| C — exception-detection biconditional + lossless FC recovery | `Exception` iff `fc >= 0x80`; `original_fc == fc & 0x7F` | BC-2.14.006 |

Union of anchored BCs = {001, 002, 003, 004, 005, 006, 007, 008} = the 8-BC frontmatter `bcs:`
set and the VP-INDEX VP-022 `Verified BCs` row (reconciled in the same F2 fix burst).

> **SPEC-level document.** This VP defines *what must be proven*. The Kani harnesses are
> authored in F4 TDD against the implemented `src/analyzer/modbus.rs`. At F4/F6 lock time the
> formal-verifier sets `verification_lock: true`, `proof_completed_date`, `proof_file_hash`,
> and `status: verified`, and creates the `vp-verified-VP-022-<YYYY-MM-DD>` tag. Until then this
> document is mutable (`verification_lock: false`).

## Source Contract

- **Primary BC:** BC-2.14.001 — MBAP header accepted for well-formed 8-byte-minimum ADU
- **Postcondition:** `parse_mbap_header` never panics; `None` iff `len < 8`; `Some(well-formed)` otherwise
- **Related BC:** BC-2.14.002 — MBAP header rejected for ADU shorter than 8 bytes (truncation safety)
- **Related BC:** BC-2.14.003 — MBAP ADU rejected when Protocol ID is not 0x0000 (gate arm 1)
- **Related BC:** BC-2.14.004 — MBAP ADU rejected when Length is outside [2, 254] (gate arm 2)
- **Related BC:** BC-2.14.005 — `classify_fc` is total over all 256 FC values (sub-property B)
- **Related BC:** BC-2.14.006 — Exception response detection: `Exception` iff `fc >= 0x80` (sub-property C)
- **Related BC:** BC-2.14.007 — Write-class FC classification (Write-set membership, sub-property B)
- **Related BC:** BC-2.14.008 — Diagnostic-class FC classification (Diagnostic-set membership, sub-property B)
- **ADR:** ADR-005 (Binary ICS Protocol Integration — Modbus TCP), §2.4 (MBAP parse model), §2.5 (FC classification)
- **Architecture:** `.factory/phase-f2-spec-evolution/architecture-delta.md` §2.4, §2.5, §2.8 (purity boundary)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes — sub-A: symbolic `[u8; N]` + symbolic `len <= 12`; sub-B/C: symbolic `u8` (all 256 values) | All parse outcomes for `data.len()` 0..=12; full FC domain 0x00..=0xFF |

Kani is the primary and sole counted tool for VP-022 (per VP-INDEX: Kani 9). The three pure-core
functions have no heap allocation, no I/O, and no `HashMap` (unlike the effectful `on_data`
shell), so they are ideal Kani targets with trivially bounded state spaces. No FFI / `RandomState`
issue arises (contrast VP-004's `on_data` state-machine modelling note), because these functions
take a slice / `u8` by value and construct only a stack `MbapHeader`.

## Proof Harness Skeleton

> Harnesses live in `src/analyzer/modbus.rs` under `#[cfg(kani)] mod kani_proofs { use super::*; }`,
> mirroring the `dispatcher.rs` / `mitre.rs` convention. The pure-core functions are free `fn`s
> (not `impl ModbusAnalyzer` methods, per architecture-delta §2.8), so harnesses call them
> directly without constructing the analyzer struct. The harnesses below are SPEC skeletons;
> exact wiring is finalized in F4 against the real signatures.

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- Sub-property A: parse_mbap_header safety (BC-2.14.001/002/003/004) ----
    //
    // Symbolic input construction: a fixed-capacity stack array of MAX_LEN bytes
    // plus a symbolic length `len <= MAX_LEN`. We slice `&buf[..len]` so Kani
    // explores every length 0..=12 with every byte symbolic. MAX_LEN = 12 covers:
    //   - the len<8 reject band (0..=7),
    //   - the minimum accept (len==8),
    //   - lengths with a few PDU bytes (9..=12) so a downstream data[8] read in
    //     the exception path (BC-2.14.006 EC-010 guard) is representable.
    // No allocation, no loop over data => no unwind bound needed for the array
    // form. `len` is symbolic-bounded by kani::assume, so the slice length is
    // bounded and Kani terminates quickly (<1 s expected).
    const MAX_LEN: usize = 12;

    #[kani::proof]
    fn verify_parse_mbap_header_safety() {
        let buf: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let data = &buf[..len];

        // (A.3) No panic / no OOB: simply calling over the symbolic slice proves
        // the absence of out-of-bounds indexing for every length 0..=12.
        let parsed = parse_mbap_header(data);

        // (A.1) len<8 => None ;  (A.2) len>=8 => Some.
        if len < 8 {
            assert!(parsed.is_none());
        } else {
            let h = parsed.expect("len>=8 must parse to Some");
            // (A.2) field decode correctness — big-endian, fixed offsets.
            assert!(h.transaction_id == u16::from_be_bytes([data[0], data[1]]));
            assert!(h.protocol_id    == u16::from_be_bytes([data[2], data[3]]));
            assert!(h.length         == u16::from_be_bytes([data[4], data[5]]));
            assert!(h.unit_id        == data[6]);
            assert!(h.function_code  == data[7]);
        }
    }

    // (A.4) Three-point validity gate biconditional. Symbolic header fields.
    #[kani::proof]
    fn verify_is_valid_modbus_adu_gate() {
        let h = MbapHeader {
            transaction_id: kani::any(),
            protocol_id:    kani::any(),
            length:         kani::any(),
            unit_id:        kani::any(),
            function_code:  kani::any(),
        };
        let ok = is_valid_modbus_adu(&h);
        // Gate is true IFF all three points hold (BC-2.14.003 + BC-2.14.004).
        assert!(ok == (h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254));
    }

    // ---- Sub-property B: classify_fc totality (BC-2.14.005/007/008) ----
    //
    // Symbolic input: a single u8 (all 256 values). The match is exhaustive by
    // construction, so "no panic" + a returned variant proves totality. We also
    // pin the set-membership obligations for Read/Write/Diagnostic.
    // No loop => no unwind bound required.
    #[kani::proof]
    fn verify_classify_fc_total() {
        let fc: u8 = kani::any();
        let class = classify_fc(fc); // must return for every u8 (no panic, no unreachable)

        // Read-set membership (BC-2.14.005 post.2).
        if matches!(fc, 0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18) {
            assert!(class == FunctionCodeClass::Read);
        }
        // Write-set membership (BC-2.14.005 post.3 / BC-2.14.007).
        if matches!(fc, 0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17) {
            assert!(class == FunctionCodeClass::Write);
        }
        // Diagnostic-set membership (BC-2.14.005 post.4 / BC-2.14.008).
        if matches!(fc, 0x08 | 0x2B) {
            assert!(class == FunctionCodeClass::Diagnostic);
        }
        // Totality witness: the returned value is one of the five variants
        // (trivially true by type, asserted for documentation/robustness).
        assert!(matches!(
            class,
            FunctionCodeClass::Read
                | FunctionCodeClass::Write
                | FunctionCodeClass::Diagnostic
                | FunctionCodeClass::Exception
                | FunctionCodeClass::Unknown
        ));
    }

    // ---- Sub-property C: exception biconditional + lossless recovery (BC-2.14.006) ----
    #[kani::proof]
    fn verify_classify_fc_exception_iff_high_bit() {
        let fc: u8 = kani::any();
        // Biconditional: Exception IFF fc >= 0x80.
        assert!((classify_fc(fc) == FunctionCodeClass::Exception) == (fc >= 0x80));
        // Lossless original-FC recovery on the exception band.
        if fc >= 0x80 {
            let original_fc = fc & 0x7F;
            assert!(original_fc < 0x80);
            assert!(original_fc == fc & 0x7F);
        }
    }
}
```

### Symbolic input construction summary

| Sub-property | Harness | Symbolic input | Bound / unwind | Assertions |
|--------------|---------|----------------|----------------|------------|
| A (parse) | `verify_parse_mbap_header_safety` | `[u8; 12]` + symbolic `len <= 12`, sliced `&buf[..len]` | `kani::assume(len <= 12)`; no loop ⇒ no `#[kani::unwind]` | no panic; `None` iff `len<8`; BE field decode on `Some` |
| A (gate) | `verify_is_valid_modbus_adu_gate` | symbolic `MbapHeader` fields | none (straight-line) | gate true iff `proto==0 && 2<=len<=254` |
| B (totality) | `verify_classify_fc_total` | `fc: u8 = kani::any()` (256 values) | none (straight-line `match`) | no panic; Read/Write/Diagnostic set membership; returns a defined variant |
| C (exception) | `verify_classify_fc_exception_iff_high_bit` | `fc: u8 = kani::any()` (256 values) | none (straight-line) | `Exception` iff `fc>=0x80`; `original_fc == fc & 0x7F` |

**Loop / unwind bounds:** none of the four harnesses contains a loop, so no `#[kani::unwind(N)]`
attribute is required (contrast VP-005 `#[kani::unwind(33)]` and VP-004 `#[kani::unwind(11)]`,
which iterate over byte slices / attempt counters). Sub-A bounds its state space via the
`kani::assume(len <= 12)` on the slice length; the array form avoids a `Vec` and therefore avoids
any allocation loop. Estimated total Kani runtime: **< 1 second** per harness (analogous to the
VP-005 SNI and VP-007 format harnesses).

**Expected result for all four harnesses:** `VERIFICATION:- SUCCESSFUL` (proof SUCCESSFUL).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Sub-A: slices of length 0..=12 (≤ 12 symbolic bytes); Sub-B/C: a single `u8` (256 values) |
| Proof complexity | Low | Three pure functions: a fixed-offset BE decode, a 3-clause boolean predicate, and a `match` with a wildcard; no allocation, no FFI |
| Tool support | High | No `HashMap`/`RandomState` in the pure core ⇒ no Kani-unsupported-FFI abort (unlike `on_data`); ideal Kani targets per architecture-delta §2.8 |
| Estimated proof time | < 1 second per harness | Comparable to VP-005 SNI and VP-007 catalog-format proofs |

## Source Location

- `src/analyzer/modbus.rs` — `fn parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` (architecture-delta §2.4)
- `src/analyzer/modbus.rs` — `fn is_valid_modbus_adu(h: &MbapHeader) -> bool` (architecture-delta §2.4, 3-point gate)
- `src/analyzer/modbus.rs` — `fn classify_fc(fc: u8) -> FunctionCodeClass` (architecture-delta §2.5)
- `src/analyzer/modbus.rs` — `struct MbapHeader` (transaction_id, protocol_id, length, unit_id, function_code)
- `src/analyzer/modbus.rs` — `enum FunctionCodeClass { Read, Write, Diagnostic, Exception, Unknown }`

(These source paths are forward references — the module is implemented in F4 TDD. The exact line
numbers are filled in when the harnesses are authored and the VP is locked.)

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created (draft, F2 spec evolution) | 2026-06-09 | formal-verifier |
| Proof harness committed | TBD (F4) | formal-verifier |
| Proof first passed | TBD (F4/F6) | formal-verifier |
| Locked (VERIFIED) | TBD (F6 gate) | spec-steward / formal-verifier |
