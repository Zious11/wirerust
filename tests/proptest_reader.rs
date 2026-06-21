//! VP-029 and VP-031 Property-Based Tests for pcapng Reader
//!
//! This file contains the two proptest verification properties mandated by STORY-126:
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
//! ## Red Gate status
//!
//! - VP-029 tests: FAIL because `spb_captured_len` is `todo!()` which causes panics
//!   on any proptest that exercises the SPB path. Additionally, the SPB arm doesn't
//!   exist yet so SPB-heavy sequences fall to the wildcard. The termination property
//!   itself should hold even with the wildcard, but any sequence that exercises
//!   `spb_captured_len` panics.
//! - VP-031 tests: ALL FAIL because `spb_captured_len` is `todo!()`.
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
/// RED trigger: if `spb_captured_len` is called during arbitrary byte processing,
/// the `todo!()` will cause a panic (which proptest catches as a test failure).
/// The SPB arm doesn't exist yet so this won't be called in the wildcard path.
/// However, this test should be mostly GREEN currently (arbitrary bytes rarely form
/// a valid SPB, and the wildcard arm handles unrecognized block types).
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
        // spb_captured_len todo!() IS a panic, so this will fail for any
        // sequence that reaches the (not-yet-implemented) SPB arm.
        // After implementation, all panics disappear.
        prop_assert!(
            result.is_ok(),
            "from_pcap_reader panicked on input (len={}): this violates the no-panic \
             contract (BC-2.01.017 PC3 / SEC-005). If spb_captured_len todo!() is the \
             cause, this will be RED until STORY-126 is implemented.",
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
/// RED: SPB arm calls `spb_captured_len` which is `todo!()` → panic.
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
        // Must not panic. spb_captured_len todo!() causes this to FAIL (RED Gate)
        // for any sequence containing an SPB (block_type_idx=0).
        prop_assert!(
            result.is_ok(),
            "from_pcap_reader panicked on structured block sequence (len={} bytes, {} blocks): \
             RED if SPB arm hits todo!() in spb_captured_len",
            bytes.len(),
            block_types.len()
        );
        // If no panic, verify the result is Ok or Err (not undefined).
        if let Ok(inner) = result {
            prop_assert!(inner.is_ok() || inner.is_err());
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
/// ALL CASES in this proptest FAIL until `spb_captured_len` is implemented
/// (currently `todo!()` → panics with "STORY-126: implement SPB captured_len formula").
///
/// We run 1000 cases per the STORY-126 requirement.
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
        // RED until implemented: todo!() panics with "STORY-126: implement..."
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
///
/// RED: spb_captured_len is todo!().
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_minimum_body_always_zero_captured_len(
        original_len: u32
    ) {
        // body.len() = 4 = SPB_FIXED_OVERHEAD_BYTES → spb_data_available = 0
        let body = vec![0u8; 4];

        // RED: todo!() panics
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
///
/// RED: spb_captured_len is todo!().
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_zero_original_len_always_zero_captured(
        body in prop::collection::vec(any::<u8>(), 4..1024)
    ) {
        prop_assume!(body.len() >= SPB_FIXED_OVERHEAD_BYTES);

        // RED: todo!() panics
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
///
/// RED: spb_captured_len is todo!().
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn proptest_VP_031_max_original_len_clamped_to_available(
        body in prop::collection::vec(any::<u8>(), 4..1024)
    ) {
        prop_assume!(body.len() >= SPB_FIXED_OVERHEAD_BYTES);

        let spb_data_available = (body.len() - SPB_FIXED_OVERHEAD_BYTES) as u32;

        // RED: todo!() panics
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
///
/// RED: spb_captured_len is todo!().
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

        // RED: todo!() panics
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
/// RED: no SPB arm → no packet produced; packets.len() == 0 fails the assertion.
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
            "SPB must produce exactly 1 packet (no SPB arm → 0 packets currently; RED)"
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
