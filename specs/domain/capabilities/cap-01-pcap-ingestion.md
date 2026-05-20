---
artifact: L2-cap-01
traces_to: ../domain-spec.md
cap_id: CAP-01
title: PCAP File Ingestion
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
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

## Target resolution (main.rs:340-360)

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
- smb3.pcapng exists in tests/fixtures but was added for future pcapng support, not as a
  negative-test fixture (finding from pass-0 R2 refuting prior hypothesis).
- Timestamp fields (`timestamp_secs`, `timestamp_usecs`) are read and stored in `RawPacket`
  but are NEVER threaded through to `Finding.timestamp` at any emission site.
  See domain-debt.md item O-01.

## BC references

BC-RDR-001..008 cover: link-type accept, link-type reject, Y2106 u32 timestamp, capture-end
detection, from_file delegation, error wrapping.

## NFR references

NFR-VIO-001 (eager load vs. "multi-GB" claim).
