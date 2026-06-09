//! Modbus TCP pure-core parser and function-code classifier (SS-14, CAP-14).
//!
//! This module provides the pure, formally-verified core functions for Modbus TCP
//! analysis per BC-2.14.001 through BC-2.14.008 and VP-022 (Kani).
//!
//! ## Architecture
//! - `parse_mbap_header` — pure parse, no validity gate (BC-2.14.001/002)
//! - `is_valid_modbus_adu` — 3-point validity gate (BC-2.14.003/004)
//! - `classify_fc` — total FC classification over all 256 u8 values (BC-2.14.005–008)
//! - `ModbusFlowState` — per-flow state stub (desync bail-out flag, BC-2.14.003)
//! - VP-022 Kani harnesses (sub-properties A, B, C) — gated by `#[cfg(kani)]`

/// Parsed Modbus Application Protocol (MBAP) header.
///
/// All fields decoded big-endian from fixed offsets per Modbus.org spec V1.1b3 §4.2:
/// - `transaction_id` at bytes 0–1
/// - `protocol_id`    at bytes 2–3
/// - `length`         at bytes 4–5  (covers Unit ID + PDU, NOT the 6-byte MBAP prefix)
/// - `unit_id`        at byte 6
/// - `function_code`  at byte 7
///
/// BC-2.14.001 postconditions 2–6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MbapHeader {
    pub transaction_id: u16,
    pub protocol_id: u16,
    /// Byte count of Unit ID + PDU (NOT including the 6-byte prefix TxnID+ProtoID+Length).
    /// Valid range for Modbus: [2, 254]. Full ADU byte count = 6 + length.
    pub length: u16,
    pub unit_id: u8,
    pub function_code: u8,
}

/// Function-code classification result (BC-2.14.005).
///
/// Variants:
/// - `Read`       — data-read FCs: {0x01,0x02,0x03,0x04,0x07,0x0B,0x0C,0x11,0x14,0x18}
/// - `Write`      — state-changing write FCs: {0x05,0x06,0x0F,0x10,0x15,0x16,0x17}
/// - `Diagnostic` — management/tunneling FCs: {0x08,0x2B}
/// - `Exception`  — any FC with high bit set (fc >= 0x80); biconditional (VP-022 sub-C)
/// - `Unknown`    — all remaining FC values (wildcard — guarantees totality)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCodeClass {
    Read,
    Write,
    Diagnostic,
    Exception,
    Unknown,
}

/// Per-flow Modbus analyzer state (stub — full field list in STORY-103).
///
/// `is_non_modbus`: when `true`, the flow has been identified as carrying
/// non-Modbus binary data (Protocol ID != 0x0000). All subsequent `on_data`
/// calls bail immediately without parsing (BC-2.14.003 invariant 2 / Decision 6).
pub struct ModbusFlowState {
    pub is_non_modbus: bool,
}

impl Default for ModbusFlowState {
    fn default() -> Self {
        ModbusFlowState {
            is_non_modbus: false,
        }
    }
}

/// Parse the 7-byte MBAP header from a reassembled TCP byte slice.
///
/// Returns `Some(MbapHeader)` when `data.len() >= 8` (7-byte header + 1-byte FC
/// minimum), `None` otherwise. This function is PURE — no validity gate on
/// `protocol_id` or `length` (those belong to `is_valid_modbus_adu`).
///
/// BC-2.14.001 (accept path) + BC-2.14.002 (truncation safety / reject path).
/// VP-022 sub-property A Kani target.
pub fn parse_mbap_header(_data: &[u8]) -> Option<MbapHeader> {
    todo!("BC-2.14.001/002: implement length guard + BE field decode")
}

/// 3-point Modbus ADU validity gate.
///
/// Returns `true` iff:
/// 1. `h.protocol_id == 0x0000`  (BC-2.14.003)
/// 2. `h.length >= 2`            (BC-2.14.004 lower bound)
/// 3. `h.length <= 254`          (BC-2.14.004 upper bound; PDU max = 253 bytes, Length = 1+253=254)
///
/// BC-2.14.003 + BC-2.14.004. Called by `on_data` (STORY-103) after a successful parse.
/// VP-022 sub-property A gate biconditional target.
pub fn is_valid_modbus_adu(_h: &MbapHeader) -> bool {
    todo!("BC-2.14.003/004: implement 3-point gate")
}

/// Classify a Modbus function code into one of five risk/type classes.
///
/// Total function over all 256 u8 values — never panics, no unreachable arm
/// (BC-2.14.005 invariant 1). Exception pre-guard fires first (BC-2.14.006).
///
/// Classification order (matches must be checked in this order):
/// 1. `fc >= 0x80`  → `Exception`  (pre-guard, BC-2.14.006)
/// 2. Write set     → `Write`      (BC-2.14.007)
/// 3. Diagnostic    → `Diagnostic` (BC-2.14.008)
/// 4. Read set      → `Read`       (BC-2.14.005 post.2)
/// 5. `_`           → `Unknown`    (wildcard, totality guarantee)
///
/// VP-022 sub-properties B (totality + set membership) and C (exception biconditional).
pub fn classify_fc(_fc: u8) -> FunctionCodeClass {
    todo!("BC-2.14.005/006/007/008: implement total FC classification match")
}

// ---------------------------------------------------------------------------
// VP-022 Kani formal-verification harnesses (sub-properties A, B, C).
// Gated by #[cfg(kani)] — not compiled in normal builds; run via `cargo kani`.
// Harness structure from VP-022 proof skeleton (architecture-delta §2.8).
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- Sub-property A (part 1): parse_mbap_header safety (BC-2.14.001/002) ----
    // Symbolic input: [u8; 12] array + symbolic len <= 12. Proves:
    //   - no panic / no OOB for any (bytes, len) combination
    //   - None iff len < 8
    //   - Some with correct BE field decode when len >= 8
    #[kani::proof]
    fn verify_parse_mbap_header_safety() {
        const MAX_LEN: usize = 12;
        let buf: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let data = &buf[..len];

        let parsed = parse_mbap_header(data);

        if len < 8 {
            assert!(parsed.is_none());
        } else {
            let h = parsed.expect("len>=8 must parse to Some");
            assert!(h.transaction_id == u16::from_be_bytes([data[0], data[1]]));
            assert!(h.protocol_id == u16::from_be_bytes([data[2], data[3]]));
            assert!(h.length == u16::from_be_bytes([data[4], data[5]]));
            assert!(h.unit_id == data[6]);
            assert!(h.function_code == data[7]);
        }
    }

    // ---- Sub-property A (part 2): is_valid_modbus_adu gate biconditional ----
    // (BC-2.14.003/004): gate is true IFF proto==0 && 2<=len<=254.
    #[kani::proof]
    fn verify_is_valid_modbus_adu_gate() {
        let h = MbapHeader {
            transaction_id: kani::any(),
            protocol_id: kani::any(),
            length: kani::any(),
            unit_id: kani::any(),
            function_code: kani::any(),
        };
        let ok = is_valid_modbus_adu(&h);
        assert!(ok == (h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254));
    }

    // ---- Sub-property B: classify_fc totality (BC-2.14.005/007/008) ----
    // Symbolic fc: u8 (all 256 values). Proves no panic + set membership.
    #[kani::proof]
    fn verify_classify_fc_total() {
        let fc: u8 = kani::any();
        let class = classify_fc(fc);

        if matches!(fc, 0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18) {
            assert!(class == FunctionCodeClass::Read);
        }
        if matches!(fc, 0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17) {
            assert!(class == FunctionCodeClass::Write);
        }
        if matches!(fc, 0x08 | 0x2B) {
            assert!(class == FunctionCodeClass::Diagnostic);
        }
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
        assert!((classify_fc(fc) == FunctionCodeClass::Exception) == (fc >= 0x80));
        if fc >= 0x80 {
            let original_fc = fc & 0x7F;
            assert!(original_fc < 0x80);
            assert!(original_fc == fc & 0x7F);
        }
    }
}
