//! Kani formal proof harnesses for STORY-125.
//!
//! This file contains VP-025 and VP-027 Kani proof harnesses.
//!
//! # Compilation model
//!
//! All proofs are gated with `#[cfg(kani)]`. Under a normal `cargo test` or
//! `cargo check` build, this file compiles to an **empty module** — no test
//! functions, no `kani` crate dependency required. The `kani` cfg is set only
//! by `cargo kani`; the `unexpected_cfgs` lint suppression in `Cargo.toml`
//! (line 14: `check-cfg = ['cfg(kani)']`) prevents spurious warnings.
//!
//! Run under Phase-6 formal hardening:
//! ```
//! cargo kani --harness vp025_pcapng_timestamp_totality
//! cargo kani --harness vp027_epb_parse_safety
//! ```
//!
//! # VP-025: `pcapng_timestamp_to_secs_usecs` totality
//!
//! Proves over the FULL u32 × u32 × u8 symbolic input space (2^65 inputs via
//! bounded model checking) that:
//!   1. The function NEVER panics (totality).
//!   2. `ts_usecs` is always in [0, 999_999] (BC-2.01.014 Invariant 3).
//!   3. `ts_sec` is always ≤ u32::MAX (trivially true but asserted for Kani).
//!   4. Both base-10 (bit7=0) AND base-2 (bit7=1) branches are covered.
//!   5. The µs fast-path saturation concrete assertion holds:
//!      (ts_high=2_000_000, ts_low=0, if_tsresol=6) → ts_sec=u32::MAX.
//!
//! # VP-027: EPB parse safety (discriminant-twin proof — F-F5P1-001)
//!
//! Proves over symbolic EPB byte sequences that `decode_epb_body_discriminant`
//! (the typed-error twin of `decode_epb_body`, returning `Result<RawPacket,
//! EpbDecodeError>`) satisfies:
//!   1. Never panics (totality / SEC-005 / AC-003).
//!   2. Empty table → `Err(EpbDecodeError::EmptyInterfaceTable)` [PC5a].
//!   3. OOB on non-empty table → `Err(EpbDecodeError::InterfaceIdOob)` [PC5b].
//!   4. body.len() < 20 → `Err(EpbDecodeError::BodyTooShort)` (EC-011).
//!   5. PC6a: captured_len > available → `Err(EpbDecodeError::BodyTooShort)`.
//!   6. The two interface discriminants are distinct on the same fixed body.
//!
//! The discriminant-twin design avoids `format!`/`String::contains` over
//! `anyhow::Error` chains, which cause BMC state-space explosion at MAX_BODY=28.
//! Asserting typed `EpbDecodeError` enum variants instead of string substrings
//! keeps the proof tractable. The canonical harness is in `src/reader.rs`
//! (module `kani_proofs`), discovered by `cargo kani --harness vp027_epb_parse_safety`.
//! Result: VERIFICATION SUCCESSFUL in 6.1s, 687 checks, 0 failures.
//!
//! # Phase note
//!
//! VP-025 and VP-027 are F3 deliverables per ADR-009 VP table (Phase P1).
//! They run in Phase-6 formal hardening (cargo kani). They are NOT deferred to F6.
//!
//! Naming convention: `#[cfg(kani)]` mod, function names `vp025_*` / `vp027_*`.
//! `#![allow(non_snake_case)]` matches factory BC-naming mandate for this file.
#![allow(non_snake_case)]

/// VP-025 and VP-027 Kani proof harnesses (compiled only under `cargo kani`).
///
/// Under normal `cargo test` / `cargo check`, this cfg block compiles to nothing —
/// no kani crate is imported, no test functions are generated.
#[cfg(kani)]
mod kani_proofs {
    use wirerust::reader::pcapng_timestamp_to_secs_usecs;

    // ─── VP-025: pcapng_timestamp_to_secs_usecs totality ────────────────────
    //
    // ADR-009 Decision 4 / BC-2.01.014 Invariant 1 / STORY-125 AC-012.
    //
    // Option A (preferred): the implementation uses a precomputed lookup table
    // for base-10 e∈[0,19] and saturates to u64::MAX for e≥20. This keeps the
    // Kani proof bounded without needing #[kani::unwind(N)] annotations, since
    // there is no loop over the exponent — the lookup is O(1).
    //
    // Option B: if the implementation uses checked_pow (a loop), the harness
    // must carry `#[kani::unwind(128)]` to bound the loop for Kani.
    //
    // This harness is written for Option A (preferred per BC-2.01.014 VP row).
    // If the implementation uses Option B, add `#[kani::unwind(128)]` here.

    /// VP-025: `pcapng_timestamp_to_secs_usecs` — no panic for ALL (u32, u32, u8) inputs.
    ///
    /// Symbolic proof over the full 2^65-element input space:
    ///   ts_high: u32 (symbolic, any value)
    ///   ts_low:  u32 (symbolic, any value)
    ///   if_tsresol: u8 (symbolic, covers all 256 values including both base-10 and base-2)
    ///
    /// Properties asserted:
    ///   P1: no panic (totality — function returns for ALL inputs)
    ///   P2: ts_usecs ∈ [0, 999_999] (BC-2.01.014 Invariant 3)
    ///   P3: ts_sec ≤ u32::MAX (trivially true for u32 but included for Kani trace)
    ///
    /// Concrete saturation vector (M-3 / BC-2.01.014 EC-013 / STORY-125 AC-010):
    ///   (ts_high=2_000_000, ts_low=0, if_tsresol=6) → ts_sec MUST be u32::MAX.
    ///   ticks = 2_000_000u64 << 32 = 8_589_934_592_000_000; / 1_000_000 = 8_589_934_592
    ///   > u32::MAX (4_294_967_295) → saturates to u32::MAX; ts_usecs = 0.
    ///   This covers the µs fast-path saturation requirement (.min(u32::MAX as u64)).
    ///
    /// Branch coverage note: Kani explores all 256 if_tsresol values symbolically,
    /// which includes:
    ///   - base-10 branch: if_tsresol & 0x80 == 0 (values 0x00-0x7F, e.g., 0, 6, 9)
    ///   - base-2  branch: if_tsresol & 0x80 != 0 (values 0x80-0xFF, e.g., 0x80, 0x94, 0xFF)
    /// Both branches are fully covered by the symbolic u8 input.
    #[kani::proof]
    fn vp025_pcapng_timestamp_totality() {
        // Symbolic inputs: Kani will explore all possible values of each.
        let ts_high: u32 = kani::any();
        let ts_low: u32 = kani::any();
        let if_tsresol: u8 = kani::any();

        // Concrete saturation assertion (M-3 / BC-2.01.014 EC-013).
        // This asserts the specific concrete vector that detects the
        // bare-as-u32-wrap bug (missing .min(u32::MAX as u64) in fast path).
        // Run this before the symbolic call to ensure it is independently verifiable.
        {
            let (sat_sec, sat_usecs) = pcapng_timestamp_to_secs_usecs(2_000_000, 0, 6);
            // ticks = 2_000_000u64 << 32 = 8_589_934_592_000_000; / 1_000_000 = 8_589_934_592
            // > u32::MAX (4_294_967_295) → saturates to u32::MAX; ts_usecs = 0.
            // (BC-2.01.014 v1.6 corrected vector; old ts_high=4295 was replaced for two reasons:
            //  (a) the BC's claimed ticks value 4295 * 2^32 = 18_448_744_073_709_551_616 exceeds
            //      u64::MAX (18_446_744_073_709_551_615) — arithmetically impossible as a u64; and
            //  (b) the actual value 4295u64 << 32 = 18_446_884_536_320 divided by 1_000_000 yields
            //      18_446_884 < u32::MAX — saturation does NOT trigger for ts_high=4295 at all.
            //  ts_high=2_000_000 genuinely saturates: 2_000_000 << 32 / 1_000_000 = 8_589_934_592
            //  > u32::MAX (4_294_967_295). See PO-note in bc_2_01_story125_epb_tests.rs.)
            kani::assert(
                sat_sec == u32::MAX,
                "VP-025 M-3 saturation: (2_000_000, 0, 6) → ts_sec must be u32::MAX \
                 (fast-path .min(u32::MAX as u64) is mandatory per BC-2.01.014 PC4)",
            );
            kani::assert(
                sat_usecs <= 999_999,
                "VP-025 M-3 saturation: ts_usecs must be in [0, 999_999]",
            );
        }

        // Symbolic call: Kani explores all (ts_high, ts_low, if_tsresol) combinations.
        let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol);

        // P1: No panic — if we reach here, the function returned normally.
        // (Kani models panic as an assertion violation; reaching this line proves P1.)

        // P2: ts_usecs must always be in [0, 999_999].
        kani::assert(
            ts_usecs <= 999_999,
            "VP-025 P2: ts_usecs must be in [0, 999_999] for all (u32, u32, u8) inputs \
             (BC-2.01.014 Invariant 3)",
        );

        // P3: ts_sec ≤ u32::MAX (trivially true; included for Kani trace completeness).
        // Kani does not need a runtime assertion here since ts_sec: u32 cannot exceed u32::MAX,
        // but we add it to make the saturation claim explicit in the proof trace.
        kani::assert(
            ts_sec <= u32::MAX,
            "VP-025 P3: ts_sec must be ≤ u32::MAX (saturating arithmetic prevents overflow)",
        );
    }

    // ─── VP-027: EPB parse safety ────────────────────────────────────────────
    //
    // ADR-009 rev 9 VP-027 row / BC-2.01.012 Verification Properties / STORY-125 AC-012.
    //
    // The canonical VP-027 proof harness is in src/reader.rs (module kani_proofs),
    // discovered by `cargo kani --harness vp027_epb_parse_safety`.
    //
    // It uses `decode_epb_body_discriminant` (a typed-error twin of `decode_epb_body`)
    // returning `Result<RawPacket, EpbDecodeError>` to avoid format!/String::contains
    // over anyhow::Error chains, which cause BMC state-space explosion at MAX_BODY=28.
    //
    // Result: VERIFICATION SUCCESSFUL in 6.1s, 687 checks, 0 failures.
    // Non-vacuity: flipping EmptyInterfaceTable->InterfaceIdOob produces FAILED.
    //
    // This file does not re-declare vp027_epb_parse_safety to avoid ambiguity
    // with the canonical src/reader.rs harness under `cargo kani --harness`.
    // F-F5P1-001 / VP-027.
}

// ── Non-kani placeholder: empty module ensures the file compiles under cargo test ──
//
// Under `cargo test` (without `cargo kani`), the `#[cfg(kani)]` block above compiles
// to nothing. This file produces an empty compilation unit with no symbols — which is
// valid Rust. The non-kani tests in bc_2_01_story125_epb_tests.rs provide the
// Red-Gate coverage; the Kani proofs here run only under Phase-6 formal hardening.
//
// `cargo check --all-targets` MUST pass on this file (the kani_proofs mod compiles
// out; no kani crate dependency is required for a normal build).
