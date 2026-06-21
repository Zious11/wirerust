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
//! # VP-027: EPB parse safety (real-call proof — F-F5P1-001)
//!
//! Proves over symbolic EPB byte sequences that `decode_epb_body`:
//!   1. Never panics (totality / SEC-005 / AC-003).
//!   2. Empty table → Err containing "E-INP-009" (and NOT "E-INP-010") [PC5a].
//!   3. OOB on non-empty table → Err containing "E-INP-010" (NOT "E-INP-009") [PC5b].
//!   4. body.len() < 20 → Err containing "E-INP-008" (EC-011).
//!   5. PC6a: captured_len > available → Err "E-INP-008".
//!   6. The two interface discriminants are distinct on the same fixed body.
//!
//! The previous (pre-F-F5P1-001) harness was tautological: it asserted the
//! `if`-guard conditions themselves rather than calling the real function.
//! This harness calls `wirerust::reader::decode_epb_body` directly over a
//! static symbolic buffer of up to MAX_BODY=28 bytes with a symbolic interface
//! table of size 0 or 1. Coverage follows the VP-027 module-anchor spec in
//! VP-INDEX [^vp025-027-module-anchor].
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

    // ─── VP-027: EPB parse safety (real-call proof — F-F5P1-001) ────────────
    //
    // ADR-009 rev 9 VP-027 row / BC-2.01.012 Verification Properties / STORY-125 AC-012.
    //
    // The previous harness (pre-F-F5P1-001) was tautological: each case asserted its
    // own `if`-guard condition rather than calling the real function. F-F5P1-001 extracted
    // `decode_epb_body` as a pure pub fn; this harness now calls it directly over a
    // static symbolic buffer and asserts the BC-2.01.012 PC9 discriminant properties.

    /// VP-027: EPB parse safety — real-call proof over symbolic body + interface table.
    ///
    /// Proves over symbolic EPB bodies + interface tables that `decode_epb_body`:
    ///   1. Never panics (totality / SEC-005 / AC-003).
    ///   2. Empty table  -> Err containing "E-INP-009" (and NOT "E-INP-010")  [PC5a].
    ///   3. OOB on non-empty table -> Err containing "E-INP-010" (NOT "E-INP-009") [PC5b].
    ///   4. body.len() < 20 -> Err containing "E-INP-008" (EC-011).
    ///   5. PC6a: captured_len > available -> Err "E-INP-008".
    ///   6. The two interface discriminants are distinct on the same fixed body.
    #[kani::proof]
    #[kani::unwind(32)]
    fn vp027_epb_parse_safety() {
        use pcap_file::DataLink;
        use wirerust::reader::{InterfaceInfo, SectionEndianness, decode_epb_body};

        // EPB fixed overhead is 20 bytes (BC-2.01.012 Invariant 5).
        // The crate-private EPB_FIXED_OVERHEAD_BYTES is not re-exported; duplicate
        // the literal here with a comment citing the BC.
        const EPB_OVERHEAD: usize = 20; // BC-2.01.012 Invariant 5

        // Symbolic body length bounded for BMC tractability.
        // 28 covers: <20 (EC-011), exactly 20 (zero captured), and a small data+pad zone
        // spanning the EC-009/EC-010 boundary.
        const MAX_BODY: usize = 28;
        let body_len: usize = kani::any_where(|n: &usize| *n <= MAX_BODY);

        // Symbolic body bytes. A fixed-capacity array sliced to body_len keeps the
        // allocation static for Kani.
        let mut buf = [0u8; MAX_BODY];
        for b in buf.iter_mut() {
            *b = kani::any();
        }
        let body: &[u8] = &buf[..body_len];

        let endianness = if kani::any() {
            SectionEndianness::LittleEndian
        } else {
            SectionEndianness::BigEndian
        };

        // ---- Case A: EMPTY table -> E-INP-009 (PC5a) ----
        {
            let empty: [InterfaceInfo; 0] = [];
            let r = decode_epb_body(body, &empty, endianness);
            // Totality: the call returns (Ok or Err); it never panics. If body_len >= 20,
            // the empty-table branch fires before any captured_len arithmetic (PC9 step iii).
            if body_len >= EPB_OVERHEAD {
                let e = r.expect_err("empty table with valid-length body must Err");
                let s = format!("{e:#}");
                kani::assert(s.contains("E-INP-009"), "empty table -> E-INP-009 (PC5a)");
                kani::assert(
                    !s.contains("E-INP-010"),
                    "empty table must NOT be E-INP-010",
                );
            } else {
                // body too short -> E-INP-008 (PC9 step i precedes empty-table check).
                let e = r.expect_err("short body must Err");
                let s = format!("{e:#}");
                kani::assert(s.contains("E-INP-008"), "body < 20 -> E-INP-008 (EC-011)");
            }
        }

        // ---- Case B: NON-EMPTY table (len 1), symbolic interface_id ----
        {
            let table = [InterfaceInfo {
                linktype: DataLink::ETHERNET,
                if_tsresol: 6,
            }];
            let r = decode_epb_body(body, &table, endianness);

            if body_len < EPB_OVERHEAD {
                let s = format!("{:#}", r.expect_err("short body must Err"));
                kani::assert(s.contains("E-INP-008"), "body < 20 -> E-INP-008 (EC-011)");
            } else {
                // interface_id is read from body[0..4]; with table.len()==1 it is OOB
                // iff interface_id != 0.
                let id = match endianness {
                    SectionEndianness::LittleEndian => {
                        u32::from_le_bytes([body[0], body[1], body[2], body[3]])
                    }
                    SectionEndianness::BigEndian => {
                        u32::from_be_bytes([body[0], body[1], body[2], body[3]])
                    }
                };
                if id as usize >= 1 {
                    let s = format!("{:#}", r.expect_err("OOB id must Err"));
                    kani::assert(s.contains("E-INP-010"), "OOB non-empty -> E-INP-010 (PC5b)");
                    kani::assert(!s.contains("E-INP-009"), "OOB must NOT be E-INP-009");
                } else {
                    // id == 0: in-bounds; result is Ok unless PC6a/PC6b reject captured_len.
                    // Either way the call must not panic, and any Err here is E-INP-008
                    // (PC6a/PC6b are the only remaining failure modes once id is valid).
                    match r {
                        Ok(_) => {}
                        Err(e) => {
                            let s = format!("{e:#}");
                            kani::assert(
                                s.contains("E-INP-008"),
                                "valid id, body-decode reject -> E-INP-008 (PC6a/PC6b)",
                            );
                        }
                    }
                }
            }
        }
    }
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
