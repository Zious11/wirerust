---
artifact: L2-cap-01
traces_to: ../domain-spec.md
cap_id: CAP-01
title: PCAP File Ingestion
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.1"
changelog:
  - 2026-05-21: Phase 3 per-story adversarial review — corrected Scope/limitations: smb3.pcapng IS now used as an active negative-test fixture (test_BC_2_01_004_rejects_pcapng, STORY-001)
  - "v1.1: Burst-10 (O-01-closure) — Scope/limitations timestamp note updated: O-01 CLOSED; timestamp_secs now threaded to Finding.timestamp (STORY-097/098/099; STORY-102..110); BC-2.04.054 sole exception. — 2026-06-13"
---

# CAP-01: PCAP File Ingestion

## What the system does today

wirerust accepts one or more targets on the CLI (files or directories). For each target, it
reads a classic-pcap file entirely into memory as a `Vec<RawPacket>` before any processing
begins. No streaming/lazy-read mode exists.

**Sources:** C-4 (reader.rs), C-3 (cli.rs), C-1 (main.rs). BC-RDR-001..008.

## Inputs

- File path (`PathBuf`) or directory (`Vec<PathBuf>` after glob expansion).
- CLI `analyze` subcommand: `targets: Vec<PathBuf>` (E-2 Commands::Analyze).

## Target resolution (main.rs:344-364)

- **Single file:** any extension accepted (no extension filter).
- **Directory:** expands `*.pcap` only. `*.pcapng` was removed from the glob by
  LESSON-P0.02 (#69) because reader.rs rejects pcapng at the format-header level.

## In-memory load (C-4 PcapSource::from_file)

`PcapSource::from_file(path)` opens the file with `BufReader`, delegates to
`PcapSource::from_pcap_reader`, and stores all packets in `PcapSource { packets:
Vec<RawPacket>, datalink: DataLink }`.

`RawPacket` (E-4) carries:
- `timestamp_secs: u32` and `timestamp_usecs: u32` -- read from pcap record header.
- `data: Vec<u8>` -- raw frame bytes.

**Implication:** for multi-GB captures, the entire file must fit in RAM. README's "multi-GB"
claim is accurate only under matching RAM constraints (NFR-VIO-001).

## Snaplen-truncated captures

`reader.rs` accepts captures where `orig_len > snap_len` (the case produced by `tcpdump -s`).
This was a genuine reader bug discovered in #87 (pcap-file 2.0.0's validated path wrongly
rejected these). The decoder also handles the resulting truncated IP/TCP frames via a
strict-first then lax-fallback strategy (see CAP-03).

## Scope / limitations

- Classic pcap ONLY. pcapng is NOT supported; reader.rs returns an error for pcapng files.
- smb3.pcapng is used as an active negative-test fixture: `test_BC_2_01_004_rejects_pcapng`
  (delivered in STORY-001) asserts that passing it to `from_file` returns Err containing
  "Failed to parse pcap header". It is no longer merely a future-coverage placeholder.
- Timestamp fields (`timestamp_secs`, `timestamp_usecs`) are read and stored in `RawPacket`
  and are threaded through to `Finding.timestamp` at 21 of 22 emission sites (STORY-097/098/099
  for http/tls/reassembly; STORY-102..110 for modbus/dnp3). Domain-debt O-01 is CLOSED.
  BC-2.04.054 (segment-limit summary finding) retains timestamp:None by design as the sole
  exception — see domain-debt.md RETIRED entry and BC-2.09.007.

## BC references

BC-RDR-001..008 cover: link-type accept, link-type reject, Y2106 u32 timestamp, capture-end
detection, from_file delegation, error wrapping.

## NFR references

NFR-VIO-001 (eager load vs. "multi-GB" claim).
