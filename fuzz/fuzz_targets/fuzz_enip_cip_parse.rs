//! F-P9-002 fuzz harness: ENIP/CIP pure-core parsers never panic on arbitrary input.
//!
//! The EtherNet/IP + CIP analyzer (SS-17, ADR-010) decodes attacker-controlled
//! pcap bytes through a layered set of pure-core, length-driven parsers. Three of
//! these are explicitly OUT OF SCOPE for the VP-032 Kani proof and instead carry
//! an F6 cargo-fuzz no-panic / bounds-safety obligation (F-P9-002; see the VP-032
//! "Out-of-scope note", ADR-010 Decision 8 DEFERRED list, and
//! enip-architecture-delta.md §4.3):
//!
//!   1. `parse_cpf_items(&[u8])` — walks the Common Packet Format item list. Reads a
//!      LE u16 `item_count`, then loops `item_count` times, each iteration reading a
//!      4-byte item header (type_id + transient length) followed by `length` data
//!      bytes. Every index is length-gated (`cursor + 4 > len` / `cursor + length >
//!      len` break the walk; the `Vec::with_capacity` is clamped to the slice size).
//!      An attacker controls `item_count` (up to 65 535) and every per-item length,
//!      so the loop-bound / cursor arithmetic and the capacity clamp are the fuzz
//!      surface (BC-2.17.005).
//!
//!   2. `parse_cip_header(&[u8])` — decodes an Unconnected Data Item (0x00B2). Reads
//!      a service byte and a transient `request_path_size` (u8), computes
//!      `path_byte_count = request_path_size * 2`, and slices `[2..2 + path_byte_count]`
//!      only after the `len < 2 + path_byte_count` bounds gate. `request_path_size`
//!      is a u8 so `* 2` maxes at 510 (no overflow even under release overflow-checks),
//!      but the slice bound is the fuzz surface (BC-2.17.006).
//!
//!   3. `parse_cip_request_path(&[u8])` — walks the logical-segment path 2 bytes at a
//!      time (`while cursor + 2 <= len`), exact-matching segment-type bytes
//!      (0x20/0x24/0x30) and silently skipping unrecognized types. The loop bound and
//!      the `path[cursor + 1]` read are the fuzz surface (BC-2.17.009).
//!
//! These three parsers were statically reviewed panic-free in F5 (all indexing
//! length-gated, no `unwrap`/`expect` on attacker bytes, no unchecked arithmetic);
//! this fuzz target is the FORMAL dynamic discharge of that F-P9-002 obligation,
//! analogous to VP-028 (pcapng reader fuzz). A panic, slice OOB, arithmetic
//! overflow, or hang on ANY input is a fuzz finding.
//!
//! As a fourth, unbounded cross-check (mirroring the sibling Modbus/DNP3 targets
//! that fuzz their pure-core header parser over arbitrary-length bytes), the harness
//! also feeds the raw input to `parse_enip_header` — the Kani-proven (VP-032 Sub-A)
//! fixed-offset encapsulation-header decoder — as an independent unbounded check of
//! that bounded proof.
//!
//! Fuzzer strategy: one arbitrary `&[u8]` drives ALL FOUR parsers directly. CPF item
//! lists, CIP item_data, and CIP request paths are all flat length-prefixed byte
//! runs, so raw fuzzer bytes (steered by coverage feedback) reach the loop-walk,
//! cursor-advance, and bounds-break arms of each parser without any structured
//! framing. Each parser is also driven on a few derived sub-slices so a short input
//! exercises the "header present, body truncated" boundary of each.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_enip_cip_parse -- -max_total_time=300 -rss_limit_mb=4096

#![no_main]

use libfuzzer_sys::fuzz_target;
use wirerust::analyzer::enip::{
    parse_cip_header, parse_cip_request_path, parse_cpf_items, parse_enip_header,
};

fuzz_target!(|data: &[u8]| {
    // --- F-P9-002 obligation: the three Kani-out-of-scope CIP parsers ---------
    // Each must handle arbitrary bytes gracefully (empty Vec / None / clean parse);
    // a panic, OOB, overflow, or hang is a finding.

    // 1. CPF item-list walk (attacker-controlled item_count + per-item lengths).
    let _ = parse_cpf_items(data);

    // 2. CIP header decode (service byte + transient request_path_size * 2 slice).
    let _ = parse_cip_header(data);

    // 3. CIP request-path 2-byte segment walk (0x20/0x24/0x30 exact-match + skip).
    let _ = parse_cip_request_path(data);

    // --- Sub-slice boundary coverage ------------------------------------------
    // Drive each parser on derived sub-slices so a single short input also exercises
    // the "outer header consumed, inner body truncated" boundary of each parser
    // (e.g. the body of a CPF item is itself fed to parse_cip_header /
    // parse_cip_request_path at the production call sites in process_pdu).
    if data.len() >= 2 {
        let tail = &data[2..];
        let _ = parse_cpf_items(tail);
        let _ = parse_cip_header(tail);
        let _ = parse_cip_request_path(tail);
    }

    // --- Unbounded cross-check of the VP-032 Sub-A Kani proof ------------------
    // parse_enip_header is Kani-proven panic-free for bounded slices; fuzzing it over
    // arbitrary-length attacker bytes is an independent unbounded check (mirrors the
    // sibling fuzz_modbus_parse / fuzz_dnp3_parse pure-core header cross-check).
    let _ = parse_enip_header(data);
});
