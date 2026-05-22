---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.02.007
bcs:
  - BC-2.02.007
  - BC-2.02.008
  - BC-2.02.009
module: src/decoder.rs
proof_method: fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: Correct fuzz target filename from decode_packet.rs to fuzz_decode_packet.rs to match delivered harness and STORY-003 AC-011; annotate pcap_source harness as unowned obligation — 2026-05-22"
  - "v1.2: Note that delivered harness is wider than skeleton (also fuzzes unsupported variants IEEE802_11, NULL, LOOP); update skeleton import to pcap_file::DataLink (STORY-003 pass-3 Nit-1) — 2026-05-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-008: decode_packet Never Panics on Arbitrary Input

## Property Statement

For any byte slice `data` of any length (including empty) and any supported
`DataLink` variant, `decode_packet(data, datalink)` never panics (no
`unwrap()`, no index out of bounds, no stack overflow on any valid input). It
either returns `Ok(ParsedPacket)` or `Err(anyhow::Error)`.

This property must hold for all five supported link types:
- `DataLink::ETHERNET` (0x01)
- `DataLink::RAW` (raw IP)
- `DataLink::IPV4` (raw IPv4)
- `DataLink::IPV6` (raw IPv6)
- `DataLink::LINUX_SLL` (Linux cooked capture)

And for all inputs including:
- Empty slices (`&[]`)
- Single-byte slices
- Truncated headers (too short for any protocol)
- Randomly mutated valid packets
- Crafted adversarial patterns (oversized length fields, etc.)

## Source Contract

- **Primary BC:** BC-2.02.007 -- Reject Malformed Input Bytes with anyhow Error (No Panic)
- **Postcondition:** `decode_packet` returns `Ok` or `Err`; never panics
- **Related BC:** BC-2.02.008 -- Reject Unsupported Link Types in decode_packet
- **Related BC:** BC-2.02.009 -- Surface No IP Layer Found Error

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Fuzzing | cargo-fuzz (libFuzzer) | No -- coverage-guided; runs until timeout | Guided mutation of valid pcap frames; random byte sequences |

Rationale: cargo-fuzz is the correct tool for no-panic properties at parser entry
points. libFuzzer's coverage-guided mutation explores etherparse's internal parsing
branches far more effectively than Kani's bounded model checking on a large external
library. The `etherparse` crate is a dependency, not verified source; fuzzing
exercises the integration boundary.

## Proof Harness Skeleton

> **Note (v1.2):** The skeleton below is illustrative/minimal. The delivered harness
> (`fuzz/fuzz_targets/fuzz_decode_packet.rs`, committed in STORY-003 pass-2) is wider:
> it additionally fuzzes unsupported `DataLink` variants (`IEEE802_11`, `NULL`, `LOOP`)
> to verify that the `Err("Unsupported link type")` arm never panics either. The skeleton
> is retained here as a readable baseline; the delivered harness is the authoritative source.

```rust
// File: fuzz/fuzz_targets/fuzz_decode_packet.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use wirerust::decoder::decode_packet;
use pcap_file::DataLink;

fuzz_target!(|data: &[u8]| {
    // Test all supported link types for every input
    let link_types = [
        DataLink::ETHERNET,
        DataLink::RAW,
        DataLink::IPV4,
        DataLink::IPV6,
        DataLink::LINUX_SLL,
    ];
    for link_type in link_types {
        // Must never panic -- Ok or Err both acceptable
        let _ = decode_packet(data, link_type);
    }
    // Delivered harness also covers unsupported variants (IEEE802_11, NULL, LOOP)
    // to confirm the Err arm is also panic-free.
});
```

Secondary fuzz target for the full pcap read path (UNOWNED — NOT scoped to any current story):

> **Note (added v1.1):** The harness below (`fuzz/fuzz_targets/pcap_source.rs`) was
> sketched here at spec-creation time but has NOT been delivered. STORY-003 scopes only
> the `fuzz_decode_packet.rs` harness above. The `pcap_source` harness is a separate,
> not-yet-scoped obligation that must be assigned to a future story before it can be
> considered part of VP-008's verification evidence. Until then it remains aspirational
> skeleton only.

```rust
// File: fuzz/fuzz_targets/pcap_source.rs
// STATUS: unowned — not delivered, not scoped to any story as of 2026-05-22
fuzz_target!(|data: &[u8]| {
    use std::io::Cursor;
    use wirerust::reader::PcapSource;
    // Must not panic on any byte sequence presented as a pcap file
    let cursor = Cursor::new(data);
    let _ = PcapSource::from_pcap_reader(cursor);
});
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded -- coverage-guided | libFuzzer explores the space guided by code coverage |
| Proof complexity | Low setup, high runtime | Fuzz target is ~10 lines; runs continuously |
| Tool support | High | cargo-fuzz is stable; wirerust has no unsafe code to complicate the harness |
| Estimated proof time | 24+ hours continuous fuzzing recommended; first runs in minutes | Coverage plateau typically reached in hours for a parser of this complexity |

## Source Location

`src/decoder.rs:128` -- `decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>`.

The etherparse parsing chain is the primary source of potential panics. wirerust has
no `unsafe` blocks. The `unwrap()` audit from pass-1 ingestion found no panicking
unwraps in the production path; fuzzing validates this empirically.

## Corpus Seeding

Seed the fuzz corpus with:
- `tests/fixtures/*.pcap` contents (valid pcap frames from each link type)
- Truncated versions of each fixture at every byte boundary
- Synthetic malformed frames: zero-length, oversized length fields, wrong ethertype

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Fuzz target committed | null | formal-verifier |
| Initial fuzzing run (8h) completed | null | formal-verifier |
| Corpus frozen | null | formal-verifier |
