//! VP-029, VP-030, and VP-031 Property-Based Tests for pcapng Reader
//!
//! This file contains the proptest verification properties for the pcapng reader
//! delta. VP-029 / VP-031 were mandated by STORY-126; VP-030 (multi-IDB linktype
//! agreement, whitelisted domain) is added here per BC-2.01.018 / ADR-009 rev 7
//! (VP-INDEX v2.9 line 114):
//!
//! ## VP-030: Multi-IDB Linktype Agreement (whitelisted domain)
//!
//! Property: over the WHITELISTED DataLink domain only —
//!   - all-equal whitelisted linktypes across N IDBs → Ok
//!   - first-differing whitelisted linktype → Err carrying the "(E-INP-011)" marker
//!   - the comparison unit is `DataLink` (the decoded variant), not the raw u16.
//!
//!
//! ## VP-029: Block-Walk Termination and Forward Progress
//!
//! Property: The block-walk loop terminates for any input sequence — well-formed or
//! malformed. The function returns `Ok(_)` or `Err(_)`; it never spins indefinitely.
//!
//! Implementation note (BC-2.01.015 Inv2 / ADR-009 Decision 8):
//!   - The crate's cursor does NOT advance on `Err(_)` (`read_buffer.rs:65`).
//!   - The block-walk MUST `break` on any `Err(_)` from `next_raw_block`.
//!   - An empty `Err(_) => {}` arm would spin indefinitely (CWE-835).
//!   - The forward-progress guard in `read_pcapng_crate` catches the zero-advance
//!     case defensively.
//!
//! ## VP-031: SPB Captured-Len Arithmetic
//!
//! Property: For all `(original_len: u32, body: &[u8])` where `body.len() >= 4`
//! (i.e., `body.len() >= SPB_FIXED_OVERHEAD_BYTES`):
//!
//!   `captured_len == min(original_len, (body.len() - 4) as u32)`
//!
//! and the returned value has no overflow or panic.
//!
//! The bare `body.len()` (without subtracting 4) MUST NOT be used — it is 4 bytes
//! too large because it counts the `original_len` field itself (ADR-009 Decision 22 /
//! BC-2.01.013 Inv2 / Architecture Compliance Rule 2).
//!
//! ## Implementation provenance
//!
//! - VP-029 tests: GREEN after STORY-126. `spb_captured_len` is implemented
//!   (src/reader.rs:770-781) and the SPB dispatch arm (src/reader.rs:1158-1218)
//!   handles SPB blocks directly. These proptests now hold and guard the
//!   block-walk termination invariant against regression.
//! - VP-031 tests: GREEN after STORY-126. `spb_captured_len` is implemented;
//!   these proptests guard the captured-length arithmetic invariant.
//!
//! ## Proptest configuration
//!
//! Per STORY-126 requirements: generate at least 1000 random cases per property.
//! We configure `cases = 1000` on all proptests.
//!
//! Naming convention: `proptest_BC_S_SS_NNN_xxx` or `proptest_VP_NNN_xxx` (the
//! `test_` prefix is not used for proptests to distinguish from unit tests).
//! `#![allow(non_snake_case)]` is required per factory BC-naming mandate.
#![allow(non_snake_case)]
#![allow(unused_doc_comments)]

use proptest::prelude::*;
use wirerust::reader::{PcapSource, SPB_FIXED_OVERHEAD_BYTES, spb_captured_len};

use std::io::Cursor;

// ─── pcapng canonical constants ─────────────────────────────────────────────

const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];
const DL_ETHERNET: u16 = 1;
const SPB_BLOCK_TYPE: u32 = 0x0000_0003;
const OPB_BLOCK_TYPE: u32 = 0x0000_0002;

// ─── Fixture helpers for VP-029 ─────────────────────────────────────────────

/// Build a minimal 28-byte LE SHB block.
fn le_shb() -> Vec<u8> {
    let mut v = Vec::with_capacity(28);
    v.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&28u32.to_le_bytes());
    v.extend_from_slice(&SHB_BOM_LE);
    v.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    v.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    v.extend_from_slice(&28u32.to_le_bytes());
    v
}

/// Build a minimal LE IDB block with Ethernet linktype.
fn le_idb_ethernet() -> Vec<u8> {
    let btl: u32 = 20;
    let mut v = Vec::with_capacity(20);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&btl.to_le_bytes());
    v.extend_from_slice(&DL_ETHERNET.to_le_bytes()); // linktype = Ethernet
    v.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
    v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    v.extend_from_slice(&btl.to_le_bytes());
    v
}

/// Build a valid skip block of the given 4-byte-aligned type with body length `body_len`.
///
/// `body_len` must be 4-aligned. If not 4-aligned, it will be rounded up.
fn le_block_aligned(block_type: u32, body_len: usize) -> Vec<u8> {
    let aligned = (body_len + 3) & !3;
    let btl = 12 + aligned;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&block_type.to_le_bytes());
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    v.extend_from_slice(&vec![0u8; aligned]); // body bytes (zeroed)
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    v
}

/// Build a LE SPB block with a body containing `original_len` + `data_len` data bytes.
/// `original_len` is set to `data_len` for well-formed SPBs.
/// `data_len` is 4-aligned.
fn le_spb(data_len: usize) -> Vec<u8> {
    let aligned = (data_len + 3) & !3;
    let body_len = 4 + aligned; // original_len(4) + data
    let btl = 12 + body_len;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    v.extend_from_slice(&(data_len as u32).to_le_bytes()); // original_len = data_len
    v.extend_from_slice(&vec![0xABu8; aligned]); // data + padding
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    v
}

/// Build a LE OPB block (empty body).
fn le_opb() -> Vec<u8> {
    le_block_aligned(OPB_BLOCK_TYPE, 0)
}

// ─── VP-029: Block-Walk Termination ─────────────────────────────────────────

/// VP-029 — Block-walk termination: from_pcap_reader always returns Ok or Err.
///
/// Strategy: Generate arbitrary byte sequences of length 0..4096 and feed them
/// to from_pcap_reader. The function MUST NOT spin or panic. It must return Ok or Err.
///
/// This tests the forward-progress invariant (CWE-835 / ADR-009 Decision 8):
///   - The loop breaks on Err from next_raw_block.
///   - The zero-advance guard fires if a future crate regression produces zero-advance Ok.
///
/// GREEN: `spb_captured_len` (src/reader.rs:770-781) and the SPB dispatch arm are
/// implemented. Arbitrary byte sequences rarely form a valid SPB header, but even
/// when they do, the SPB arm returns Ok — never panics. The wildcard arm handles
/// unrecognized block types. This test guards the totality invariant (BC-2.01.017 PC3).
///
/// Note: proptest runs this as a regular test function; we configure 1000 cases.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_029_block_walk_terminates_arbitrary_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..4096)
    ) {
        // Feed arbitrary bytes — function must not panic or spin.
        // We don't care about Ok vs Err; only that it terminates.
        let result = std::panic::catch_unwind(|| {
            PcapSource::from_pcap_reader(Cursor::new(&bytes))
        });
        // The function must not panic (catch_unwind Ok = no panic).
        // Regression guard: any panic here indicates a totality violation in
        // the SPB path or another block-walk branch (BC-2.01.017 PC3 / SEC-005).
        prop_assert!(
            result.is_ok(),
            "from_pcap_reader panicked on input (len={}): totality invariant violated — \
             the reader must never panic regardless of input (BC-2.01.017 PC3 / SEC-005).",
            bytes.len()
        );
    }
}

/// VP-029 — Block-walk termination with structured valid pcapng + arbitrary trailing bytes.
///
/// Start with a valid SHB+IDB header, then append random bytes for the block region.
/// This exercises the block-walk loop more directly (the SHB parse succeeds, then
/// arbitrary bytes form "blocks" of various types/lengths/truncations).
///
/// The function must return Ok or Err — not spin or panic.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_029_block_walk_terminates_with_valid_shb_prefix(
        tail in prop::collection::vec(any::<u8>(), 0..2048)
    ) {
        let mut bytes = le_shb();
        bytes.extend_from_slice(&tail);

        let result = std::panic::catch_unwind(|| {
            PcapSource::from_pcap_reader(Cursor::new(&bytes))
        });
        prop_assert!(
            result.is_ok(),
            "from_pcap_reader panicked on SHB + {} random tail bytes: \
             this violates the no-panic contract",
            tail.len()
        );
    }
}

/// VP-029 — Block-walk with known valid block types in arbitrary order and count.
///
/// Strategy: Generate a sequence of block-type identifiers (0=SPB, 1=OPB, 2=ISB,
/// 3=NRB, 4=unknown) and build a valid pcapng with SHB+IDB followed by those blocks.
/// Each block must be skipped or parsed correctly; the loop must terminate.
///
/// This exercises named skip arms, OPB dual-counter, SPB parse, and unknown catch-all.
///
/// GREEN: The SPB arm calls `spb_captured_len` (src/reader.rs:770-781), which is
/// implemented and returns the correct captured length — no panic possible.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_029_block_walk_terminates_known_block_sequence(
        block_types in prop::collection::vec(0u8..6u8, 0..20)
    ) {
        let mut bytes = le_shb();
        bytes.extend_from_slice(&le_idb_ethernet());

        for block_type_idx in &block_types {
            match block_type_idx {
                0 => bytes.extend_from_slice(&le_spb(4)), // SPB (4 bytes payload)
                1 => bytes.extend_from_slice(&le_opb()),   // OPB
                2 => bytes.extend_from_slice(&le_block_aligned(0x0000_0005, 0)), // ISB
                3 => bytes.extend_from_slice(&le_block_aligned(0x0000_0004, 0)), // NRB
                4 => bytes.extend_from_slice(&le_block_aligned(0xDEAD_BEEF, 0)), // unknown
                5 => bytes.extend_from_slice(&le_block_aligned(0x0000_000A, 8)), // DSB (8 bytes body)
                _ => {}
            }
        }

        let result = std::panic::catch_unwind(|| {
            PcapSource::from_pcap_reader(Cursor::new(&bytes))
        });
        // Regression guard: the function must not panic for any structured block sequence.
        // A panic here indicates a totality violation in the SPB arm or another dispatch
        // path (BC-2.01.017 PC3 / SEC-005 / ADR-009 Decision 8).
        prop_assert!(
            result.is_ok(),
            "from_pcap_reader panicked on structured block sequence (len={} bytes, {} blocks): \
             totality invariant violated — no block sequence may cause a panic",
            bytes.len(),
            block_types.len()
        );
        // If no panic: verify forward progress — the function must have consumed the input
        // (returned a result) rather than spinning indefinitely. The catch_unwind completing
        // above already guarantees termination; here we assert the result is a meaningful
        // outcome (Ok with packets/counters, or Err with a non-empty message).
        if let Ok(inner) = result {
            match inner {
                Ok(ref src) => {
                    // Forward progress: at minimum, the SHB was processed.
                    // skipped_blocks + packets.len() + opb_skipped are all ≥ 0 (trivially).
                    // Meaningful check: the source struct is coherent (opb_skipped ≤ skipped_blocks).
                    prop_assert!(
                        src.opb_skipped <= src.skipped_blocks,
                        "opb_skipped={} must never exceed skipped_blocks={} \
                         (OPB increments both; non-OPB increments only skipped_blocks)",
                        src.opb_skipped,
                        src.skipped_blocks
                    );
                }
                Err(ref e) => {
                    // Forward progress on error path: the error message must be non-empty.
                    prop_assert!(
                        !e.to_string().is_empty(),
                        "error message must be non-empty (error without context is a silent failure)"
                    );
                }
            }
        }
    }
}

/// VP-029 (strengthened) — Skip-arm dispatch counter exactness + DSB no-log + forward progress.
///
/// BC-2.01.015 AC-001 F-07 / AC-003 / AC-007 / AC-008 / SEC-007.
///
/// Builds a valid SHB+IDB header followed by an arbitrary sequence of skip-eligible
/// blocks (NRB / ISB / SJE / DSB / OPB / unknown). The reader exposes exactly two
/// counters (`skipped_blocks` total and `opb_skipped` OPB sub-count); there are no
/// per-arm counters. This proptest asserts the EXACT counter arithmetic:
///
///   - `skipped_blocks` == total number of skip-eligible blocks emitted, AND
///   - `opb_skipped`    == number of OPB blocks emitted (and only OPB increments it).
///
/// It also exercises the "right counter per arm" requirement: each named arm
/// (NRB/ISB/SJE/DSB/unknown) increments `skipped_blocks` by exactly 1 and leaves
/// `opb_skipped` untouched, while OPB increments BOTH. Since every block in this
/// strategy is skip-eligible (no EPB/SPB packet emitters), `packets.len()` must be 0.
///
/// DSB no-log (SEC-007): DSB carries TLS key material and MUST NOT be surfaced.
/// Structurally, the DSB arm only bumps `skipped_blocks` and never ingests the body
/// into `packets`. We assert `packets.is_empty()` even when DSB blocks carry non-zero
/// body bytes — the body never escapes into the packet stream.
///
/// Forward progress: every block advances the cursor; the loop terminates and the
/// counters equal the emitted-block counts (no missed block, no double-count, no spin).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_029_skip_arm_counter_exactness_and_dsb_no_log(
        block_types in prop::collection::vec(0u8..6u8, 0..40)
    ) {
        // Map: 0=NRB, 1=ISB, 2=SJE, 3=DSB(non-zero body), 4=OPB, 5=unknown.
        let mut bytes = le_shb();
        bytes.extend_from_slice(&le_idb_ethernet());

        let mut expected_skipped: u32 = 0;
        let mut expected_opb: u32 = 0;

        for t in &block_types {
            match t {
                0 => bytes.extend_from_slice(&le_block_aligned(0x0000_0004, 0)), // NRB
                1 => bytes.extend_from_slice(&le_block_aligned(0x0000_0005, 0)), // ISB
                2 => bytes.extend_from_slice(&le_block_aligned(0x0000_0009, 0)), // SJE
                3 => {
                    // DSB with NON-ZERO body (simulated "secret" bytes 0xDE) — these
                    // bytes MUST NOT escape into packets (SEC-007 no-log/no-surface).
                    let mut dsb = Vec::new();
                    let body_len = 8usize;
                    let btl = 12 + body_len;
                    dsb.extend_from_slice(&0x0000_000Au32.to_le_bytes()); // DSB type
                    dsb.extend_from_slice(&(btl as u32).to_le_bytes());
                    dsb.extend_from_slice(&vec![0xDEu8; body_len]); // "secret" body
                    dsb.extend_from_slice(&(btl as u32).to_le_bytes());
                    bytes.extend_from_slice(&dsb);
                }
                4 => bytes.extend_from_slice(&le_opb()), // OPB
                5 => bytes.extend_from_slice(&le_block_aligned(0xDEAD_BEEF, 0)), // unknown
                _ => continue,
            }
            expected_skipped += 1;
            if *t == 4 {
                expected_opb += 1;
            }
        }

        let src = PcapSource::from_pcap_reader(Cursor::new(&bytes))
            .expect("valid SHB+IDB + skip-only blocks must parse to Ok");

        // Counter exactness: every skip-eligible block was counted exactly once.
        prop_assert_eq!(
            src.skipped_blocks,
            expected_skipped,
            "skipped_blocks={} must equal the number of skip-eligible blocks emitted={} \
             (forward progress: no missed block, no double-count)",
            src.skipped_blocks,
            expected_skipped
        );
        // OPB sub-counter: only OPB increments opb_skipped.
        prop_assert_eq!(
            src.opb_skipped,
            expected_opb,
            "opb_skipped={} must equal the number of OPB blocks emitted={} \
             (only OPB increments the sub-counter; NRB/ISB/SJE/DSB/unknown must not)",
            src.opb_skipped,
            expected_opb
        );
        // Dual-counter invariant always holds.
        prop_assert!(
            src.opb_skipped <= src.skipped_blocks,
            "opb_skipped={} must never exceed skipped_blocks={}",
            src.opb_skipped,
            src.skipped_blocks
        );
        // DSB no-log / no-surface (SEC-007): no skip block ever produces a packet,
        // so the DSB "secret" body bytes never escape into the packet stream.
        prop_assert!(
            src.packets.is_empty(),
            "skip-only block stream must yield zero packets; got {} \
             (DSB body bytes must never surface — SEC-007)",
            src.packets.len()
        );
    }
}

// ─── VP-030: Multi-IDB Linktype Agreement (whitelisted domain) ──────────────

/// Whitelisted DataLink raw u16 linktypes (pcap-file 2.x encoding).
/// Per ADR-009 rev 7 / VP-INDEX v2.9 line 114, VP-030 operates over the
/// WHITELISTED domain only: ETHERNET=1, RAW=101, LINUX_SLL=113, IPV4=228, IPV6=229.
const WHITELISTED_LINKTYPES: [u16; 5] = [1, 101, 113, 228, 229];

/// Build a minimal LE IDB block with the given raw u16 linktype.
fn le_idb_with_linktype(linktype: u16) -> Vec<u8> {
    let btl: u32 = 20;
    let mut v = Vec::with_capacity(20);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&btl.to_le_bytes());
    v.extend_from_slice(&linktype.to_le_bytes()); // linktype (u16 LE)
    v.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
    v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    v.extend_from_slice(&btl.to_le_bytes());
    v
}

/// VP-030 — All-equal whitelisted linktypes across N IDBs → Ok (no E-INP-011).
///
/// BC-2.01.018 / ADR-009 Decision 17 check 3. When every IDB carries the SAME
/// whitelisted linktype, there is no conflict and the reader returns Ok. The
/// comparison unit is `DataLink` (not the raw u16) — but for a single whitelisted
/// value, raw equality and DataLink equality coincide, so all-equal is Ok.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_030_all_equal_whitelisted_idbs_ok(
        lt_idx in 0usize..WHITELISTED_LINKTYPES.len(),
        n_extra in 0usize..8usize
    ) {
        let lt = WHITELISTED_LINKTYPES[lt_idx];

        let mut bytes = le_shb();
        // First IDB + n_extra additional IDBs, all with the SAME whitelisted linktype.
        for _ in 0..(1 + n_extra) {
            bytes.extend_from_slice(&le_idb_with_linktype(lt));
        }

        let result = PcapSource::from_pcap_reader(Cursor::new(&bytes));
        prop_assert!(
            result.is_ok(),
            "all-equal whitelisted linktype {} across {} IDBs must return Ok \
             (no E-INP-011 conflict); got: {:?}",
            lt,
            1 + n_extra,
            result.err()
        );
        let src = result.unwrap();
        // The resolved datalink must equal the (single) whitelisted linktype.
        prop_assert_eq!(
            u32::from(src.datalink),
            u32::from(lt),
            "resolved datalink must equal the agreed whitelisted linktype {}",
            lt
        );
    }
}

/// VP-030 — First-differing whitelisted linktype → Err(E-INP-011) at that IDB.
///
/// BC-2.01.018 AC-001 / ADR-009 Decision 17 check 3. Given a sequence of whitelisted
/// IDBs where exactly one (at a chosen position ≥ 2nd) differs from the first, the
/// reader must return Err whose message carries the "(E-INP-011)" marker. The
/// comparison unit is DataLink: two DISTINCT whitelisted raw values map to distinct
/// DataLink variants, so the first differing IDB triggers the conflict.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_030_first_differing_whitelisted_idb_errs_e_inp_011(
        first_idx in 0usize..WHITELISTED_LINKTYPES.len(),
        diff_idx in 0usize..WHITELISTED_LINKTYPES.len(),
        leading_same in 0usize..4usize,
    ) {
        let first_lt = WHITELISTED_LINKTYPES[first_idx];
        let diff_lt = WHITELISTED_LINKTYPES[diff_idx];
        // Require an actual difference (distinct whitelisted DataLinks).
        prop_assume!(first_lt != diff_lt);

        let mut bytes = le_shb();
        // First IDB establishes the registered linktype.
        bytes.extend_from_slice(&le_idb_with_linktype(first_lt));
        // `leading_same` more IDBs that AGREE (must not trigger conflict).
        for _ in 0..leading_same {
            bytes.extend_from_slice(&le_idb_with_linktype(first_lt));
        }
        // The first DIFFERING IDB — this must trip E-INP-011.
        bytes.extend_from_slice(&le_idb_with_linktype(diff_lt));

        let result = PcapSource::from_pcap_reader(Cursor::new(&bytes));
        prop_assert!(
            result.is_err(),
            "a differing whitelisted linktype (first={}, differing={}) must return \
             Err(E-INP-011); got Ok",
            first_lt,
            diff_lt
        );
        let msg = result.unwrap_err().to_string();
        prop_assert!(
            msg.contains("E-INP-011"),
            "error message must carry the (E-INP-011) marker; got: {:?}",
            msg
        );
    }
}

/// VP-030 — Comparison unit is DataLink, not raw u16 (regression guard).
///
/// This pins the BC-2.01.018 requirement that agreement is decided on the DECODED
/// DataLink, not the raw u16. Two IDBs with the SAME whitelisted raw value decode to
/// the SAME DataLink → Ok. Two IDBs with DISTINCT whitelisted raw values decode to
/// DISTINCT DataLinks → Err(E-INP-011). We assert both directions over the
/// whitelisted cross-product, which is exactly the DataLink-equality semantics.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_030_comparison_unit_is_datalink(
        a_idx in 0usize..WHITELISTED_LINKTYPES.len(),
        b_idx in 0usize..WHITELISTED_LINKTYPES.len(),
    ) {
        let a = WHITELISTED_LINKTYPES[a_idx];
        let b = WHITELISTED_LINKTYPES[b_idx];

        let mut bytes = le_shb();
        bytes.extend_from_slice(&le_idb_with_linktype(a));
        bytes.extend_from_slice(&le_idb_with_linktype(b));

        let result = PcapSource::from_pcap_reader(Cursor::new(&bytes));

        if a == b {
            // Same raw → same DataLink → agreement → Ok.
            prop_assert!(
                result.is_ok(),
                "equal whitelisted DataLinks (raw {}) must agree → Ok; got {:?}",
                a,
                result.err()
            );
        } else {
            // Distinct whitelisted raw values → distinct DataLink variants → Err.
            prop_assert!(
                result.is_err(),
                "distinct whitelisted DataLinks (raw {} vs {}) must conflict → Err(E-INP-011)",
                a,
                b
            );
            prop_assert!(
                result.unwrap_err().to_string().contains("E-INP-011"),
                "conflict must be reported as E-INP-011"
            );
        }
    }
}

// ─── VP-031: SPB Captured-Len Arithmetic ────────────────────────────────────

/// VP-031 — SPB captured_len arithmetic: for all (original_len, body) with body.len()>=4:
///   captured_len == min(original_len, (body.len() - 4) as u32)
///
/// This is the canonical formula from ADR-009 Decision 22 / BC-2.01.013 Inv2 / AC-002.
/// The bare `body.len()` (without -4) is WRONG — 4 bytes too large.
///
/// All 1000 cases are GREEN: `spb_captured_len` (src/reader.rs:770-781) is implemented
/// and returns `min(original_len, (body.len() - 4) as u32)` per ADR-009 Decision 22.
/// This proptest guards against regressions in the arithmetic (e.g. off-by-4, saturation).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_spb_captured_len_arithmetic(
        original_len: u32,
        body in prop::collection::vec(any::<u8>(), 4..1024) // body.len() >= 4
    ) {
        // Pre-condition: body.len() >= SPB_FIXED_OVERHEAD_BYTES (4).
        // (Shorter bodies are the E-INP-008 body-too-short path, not VP-031 domain.)
        prop_assume!(body.len() >= SPB_FIXED_OVERHEAD_BYTES);

        // Canonical formula (ADR-009 Decision 22):
        let spb_data_available = (body.len() - SPB_FIXED_OVERHEAD_BYTES) as u32;
        let expected_captured_len = original_len.min(spb_data_available);

        // Call the pure-core helper.
        // GREEN: spb_captured_len (src/reader.rs:770-781) returns the correct value.
        let actual = spb_captured_len(original_len, &body);

        prop_assert_eq!(
            actual,
            expected_captured_len,
            "spb_captured_len({}, body.len()={}) must equal min({}, {}) = {}; \
             bare body.len()={} (4 too large) is the wrong bound (ADR-009 Decision 22)",
            original_len,
            body.len(),
            original_len,
            spb_data_available,
            expected_captured_len,
            body.len()
        );

        // Secondary: verify captured_len never exceeds spb_data_available (no OOB).
        prop_assert!(
            actual <= spb_data_available,
            "captured_len={} must never exceed spb_data_available={} \
             (no out-of-bounds slice possible)",
            actual,
            spb_data_available
        );

        // Secondary: verify captured_len never exceeds original_len.
        prop_assert!(
            actual <= original_len,
            "captured_len={} must never exceed original_len={}",
            actual,
            original_len
        );
    }
}

/// VP-031 (boundary) — body.len() exactly 4 (minimum legal): spb_data_available=0.
///
/// For any original_len, captured_len must be 0 when body.len()==4.
/// GREEN: `spb_captured_len` (src/reader.rs:770-781) returns 0 when body.len()-4=0.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_minimum_body_always_zero_captured_len(
        original_len: u32
    ) {
        // body.len() = 4 = SPB_FIXED_OVERHEAD_BYTES → spb_data_available = 0
        let body = vec![0u8; 4];

        // GREEN: spb_captured_len returns 0 for body.len()=4 (spb_data_available=0).
        let actual = spb_captured_len(original_len, &body);

        prop_assert_eq!(
            actual,
            0,
            "when body.len()=4 (spb_data_available=0), captured_len must be 0 \
             for any original_len={}",
            original_len
        );
    }
}

/// VP-031 (saturation) — original_len=0 always yields captured_len=0 regardless of body.
///
/// min(0, spb_data_available) = 0 for any spb_data_available.
/// GREEN: `spb_captured_len` (src/reader.rs:770-781) returns 0 when original_len=0.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_zero_original_len_always_zero_captured(
        body in prop::collection::vec(any::<u8>(), 4..1024)
    ) {
        prop_assume!(body.len() >= SPB_FIXED_OVERHEAD_BYTES);

        // GREEN: spb_captured_len returns 0 when original_len=0 (min(0, anything)=0).
        let actual = spb_captured_len(0, &body);

        prop_assert_eq!(
            actual,
            0,
            "captured_len must be 0 when original_len=0 (min(0, anything) = 0)"
        );
    }
}

/// VP-031 (max original_len) — original_len=u32::MAX clamped to spb_data_available.
///
/// For any body, captured_len = spb_data_available = body.len() - 4.
/// GREEN: `spb_captured_len` (src/reader.rs:770-781) returns spb_data_available when
/// original_len=u32::MAX (clamped by min()).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_max_original_len_clamped_to_available(
        body in prop::collection::vec(any::<u8>(), 4..1024)
    ) {
        prop_assume!(body.len() >= SPB_FIXED_OVERHEAD_BYTES);

        let spb_data_available = (body.len() - SPB_FIXED_OVERHEAD_BYTES) as u32;

        // GREEN: spb_captured_len returns spb_data_available when original_len=u32::MAX.
        let actual = spb_captured_len(u32::MAX, &body);

        prop_assert_eq!(
            actual,
            spb_data_available,
            "when original_len=u32::MAX, captured_len must be clamped to \
             spb_data_available={} (body.len()-4={})",
            spb_data_available,
            body.len() - SPB_FIXED_OVERHEAD_BYTES
        );
    }
}

/// VP-031 (exact match) — when original_len == spb_data_available: no truncation.
///
/// min(n, n) = n for any n. Captured_len == original_len == spb_data_available.
/// GREEN: `spb_captured_len` (src/reader.rs:770-781) returns original_len when
/// original_len == spb_data_available (identity case, no clamping).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_exact_match_no_truncation(
        extra_body_len in 0u32..1020u32 // controls spb_data_available
    ) {
        // body.len() = 4 + extra_body_len → spb_data_available = extra_body_len
        let body_len = 4 + extra_body_len as usize;
        let body = vec![0xFFu8; body_len];
        let spb_data_available = extra_body_len;
        let original_len = spb_data_available; // exact match

        // GREEN: spb_captured_len returns original_len unchanged when equal to spb_data_available.
        let actual = spb_captured_len(original_len, &body);

        prop_assert_eq!(
            actual,
            original_len,
            "when original_len={} == spb_data_available={}, captured_len must equal both \
             (no truncation)",
            original_len,
            spb_data_available
        );
    }
}

/// VP-031 end-to-end integration: SPB packets have data.len() == min(original_len, body.len()-4).
///
/// This exercises the full pcapng reader path: SHB + IDB + SPB with various
/// (original_len, payload) combinations. For each: data.len() in the resulting
/// RawPacket must match the canonical formula.
///
/// GREEN: SPB arm (src/reader.rs:1158-1218) is implemented; each SPB produces exactly
/// one RawPacket with data.len() == min(original_len, spb_data_available).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_spb_e2e_packet_data_length_matches_formula(
        payload in prop::collection::vec(any::<u8>(), 0..256),
        original_len_delta in -128i32..128i32
    ) {
        // Compute the in-file payload length (4-byte aligned for pcapng format).
        let payload_len = payload.len();

        // original_len can be less, equal, or greater than payload_len.
        let original_len = (payload_len as i32 + original_len_delta)
            .max(0) as u32;

        // Build the SPB block with the given payload and original_len.
        let pad_len = (4usize.wrapping_sub(payload_len % 4)) % 4;
        let padded_data_len = payload_len + pad_len;
        let body_len = 4 + padded_data_len;
        let btl = 12 + body_len;

        let mut block = Vec::with_capacity(btl);
        block.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
        block.extend_from_slice(&(btl as u32).to_le_bytes());
        block.extend_from_slice(&original_len.to_le_bytes());
        block.extend_from_slice(&payload);
        block.extend_from_slice(&vec![0u8; pad_len]);
        block.extend_from_slice(&(btl as u32).to_le_bytes());

        let mut bytes = le_shb();
        bytes.extend_from_slice(&le_idb_ethernet());
        bytes.extend_from_slice(&block);

        let result = PcapSource::from_pcap_reader(Cursor::new(&bytes));

        prop_assert!(
            result.is_ok(),
            "SPB parse should succeed; got: {:?}",
            result.err()
        );

        let source = result.unwrap();
        prop_assert_eq!(
            source.packets.len(),
            1,
            "regression: SPB arm must produce exactly 1 packet (src/reader.rs:1158-1218)"
        );

        let pkt = &source.packets[0];

        // Canonical formula (ADR-009 Decision 22):
        // spb_data_available = body.len() - 4 = padded_data_len
        let spb_data_available = padded_data_len as u32;
        let expected_data_len = original_len.min(spb_data_available) as usize;

        prop_assert_eq!(
            pkt.data.len(),
            expected_data_len,
            "packet data.len()={} must equal min(original_len={}, \
             spb_data_available={})={}; \
             bare body.len()-4={} is the correct available-bytes bound",
            pkt.data.len(),
            original_len,
            spb_data_available,
            expected_data_len,
            padded_data_len
        );

        // Timestamps must always be zero for SPB (BC-2.01.013 Invariant 1).
        prop_assert_eq!(
            pkt.timestamp_secs,
            0,
            "SPB timestamp_secs must always be 0"
        );
        prop_assert_eq!(
            pkt.timestamp_usecs,
            0,
            "SPB timestamp_usecs must always be 0"
        );
    }
}
