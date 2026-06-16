//! VP-008 fuzz harness: decode_packet Never Panics on Arbitrary Input.
//!
//! Exercises `wirerust::decoder::decode_packet` with arbitrary byte inputs
//! across BOTH the whitelisted DataLink variants (ETHERNET, RAW, IPV4, IPV6,
//! LINUX_SLL) AND unsupported variants (IEEE802_11, NULL, LOOP). The
//! no-panic guarantee from VP-008 must hold for every code path, including
//! the `other => return Err(...)` rejection arm in `decode_packet` and the
//! matching arm in `lax_parse` — paths that are unreachable when only
//! whitelisted variants are exercised.
//!
//! Return type: `decode_packet` returns `Result<DecodedFrame>` (updated from
//! `Result<ParsedPacket>` in STORY-111, etherparse 0.20 migration). Both
//! `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` are non-panic
//! outcomes and are acceptable to this harness (both discarded via `let _`).
//! Only a runtime panic constitutes a fuzz finding. (VP-008 / BC-2.02.009
//! Invariant 5 / arp-architecture-delta §4.3.)
//!
//! BC-2.02.008 ("Reject Unsupported Link Types") is a source contract for
//! VP-008; BC-2.02.007 invariant 1 lists "Unsupported link type:" as a
//! reachable error prefix. Both are fuzzed here.
//!
//! A panic inside `decode_packet` is a fuzz finding. `Err(_)` is the correct,
//! expected outcome for unsupported variants and for most arbitrary inputs on
//! supported variants; `Ok(_)` is valid for well-formed inputs on supported
//! variants.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_decode_packet

#![no_main]

use libfuzzer_sys::fuzz_target;
use pcap_file::DataLink;
use wirerust::decoder::decode_packet;

fuzz_target!(|data: &[u8]| {
    // --- Whitelisted variants (parse paths) ---
    // These reach SlicedPacket / LaxSlicedPacket and the full decode logic.
    let _ = decode_packet(data, DataLink::ETHERNET);
    let _ = decode_packet(data, DataLink::RAW);
    let _ = decode_packet(data, DataLink::IPV4);
    let _ = decode_packet(data, DataLink::IPV6);
    let _ = decode_packet(data, DataLink::LINUX_SLL);

    // --- Unsupported variants (rejection paths) ---
    // These must reach the `other => return Err("Unsupported link type: ...")` arm
    // in both decode_packet and lax_parse without panicking. Err(_) is the only
    // valid outcome; the fuzz engine detects any panic as a finding.
    let _ = decode_packet(data, DataLink::IEEE802_11);
    let _ = decode_packet(data, DataLink::NULL);
    let _ = decode_packet(data, DataLink::LOOP);
});
