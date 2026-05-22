//! VP-008 fuzz harness: decode_packet Never Panics on Arbitrary Input.
//!
//! Exercises `wirerust::decoder::decode_packet` with the full set of supported
//! [`pcap_file::DataLink`] variants under every possible byte sequence the
//! fuzzer can generate. A panic inside `decode_packet` constitutes a fuzz
//! finding. Returning `Err(_)` is fully expected and is not a finding.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_decode_packet

#![no_main]

use libfuzzer_sys::fuzz_target;
use pcap_file::DataLink;
use wirerust::decoder::decode_packet;

fuzz_target!(|data: &[u8]| {
    // Exercise every supported DataLink variant with the same arbitrary input.
    // The contract under test (VP-008) is: decode_packet NEVER panics.
    // Err(_) is the expected outcome for most inputs; Ok(_) is also valid.
    let _ = decode_packet(data, DataLink::ETHERNET);
    let _ = decode_packet(data, DataLink::RAW);
    let _ = decode_packet(data, DataLink::IPV4);
    let _ = decode_packet(data, DataLink::IPV6);
    let _ = decode_packet(data, DataLink::LINUX_SLL);
});
