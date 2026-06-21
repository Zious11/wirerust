//! VP-028 fuzz harness: pcapng FILE reader Never Panics on Arbitrary Input.
//!
//! Exercises `wirerust::reader::PcapSource::from_pcap_reader` over an in-memory
//! `&[u8]` cursor with ARBITRARY bytes. This is the pcapng/pcap FILE reader entry
//! path — the block-walk loop, SHB/IDB/EPB/SPB parsing, the skip-arm dispatch
//! (NRB/ISB/SJE/DSB/OPB/unknown), the multi-IDB E-INP-011 conflict check, and the
//! SPB captured-length arithmetic all sit behind this single entry point.
//!
//! No-panic contract (BC-2.01.017 PC3 / SEC-005): the reader must NEVER panic on
//! any input. It must return `Ok(PcapSource)` (well-formed enough to parse) or a
//! clean `Err(_)` (malformed / truncated / conflicting). A runtime panic — index
//! out of bounds, arithmetic overflow, slice OOB, infinite loop / hang — is a fuzz
//! finding. Both `Ok` and `Err` are acceptable, expected outcomes; only a panic or
//! a timeout (libFuzzer's hang detector) constitutes a finding.
//!
//! `Cursor<&[u8]>` is `Read`, satisfying the `R: Read` bound on `from_pcap_reader`.
//! Feeding raw fuzzer bytes makes the magic-number dispatch (pcap vs pcapng) and the
//! full block-walk reachable: most inputs miss the SHB magic and exit early via a
//! clean Err, while the fuzzer's coverage feedback steers a fraction toward valid
//! SHB/IDB prefixes that drive the block-walk and packet-emitter arms.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_pcapng_reader -- -max_total_time=120 -rss_limit_mb=4096

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use wirerust::reader::PcapSource;

fuzz_target!(|data: &[u8]| {
    // The reader entry path over an in-memory cursor. Ok or clean Err are both
    // acceptable; the fuzz engine flags any panic, OOB, overflow, or hang.
    let _ = PcapSource::from_pcap_reader(Cursor::new(data));
});
