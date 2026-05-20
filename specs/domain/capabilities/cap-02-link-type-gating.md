---
artifact: L2-cap-02
traces_to: ../domain-spec.md
cap_id: CAP-02
title: Link-Type Gating
status: descriptive (brownfield)
---

# CAP-02: Link-Type Gating

## What the system does today

Before any packet decoding, reader.rs checks the pcap global header's link-type field and
rejects unsupported values. This is the L1 Ingest boundary (C-4/C-5).

**Sources:** C-4 reader.rs, C-5 decoder.rs. BC-RDR-002, BC-DEC-001..015.

## Accepted link types (5-element whitelist)

The following `pcap_file::DataLink` variants are accepted at reader.rs:22-35:

| Link type | DataLink variant | Typical use |
|---|---|---|
| Ethernet II | ETHERNET | Most pcap captures from physical networks |
| Raw IP | RAW / IPV4 / IPV6 | Loopback or tunnel captures |
| Linux cooked | LINUX_SLL | Captures via `tcpdump -i any` |

Any other link type causes `PcapSource::from_pcap_reader` to return `Err(anyhow!("Unsupported
link type: ..."))` immediately; no packets from that file are processed.

## Per-packet decoding gate

`decode_packet(data, datalink)` in decoder.rs:128-172 dispatches on the same five-element
whitelist to strip the link-layer header and hand the IP payload to etherparse. Errors:

- Unsupported link type: `anyhow!("Unsupported link type")`.
- No IP layer: `anyhow!("No IP layer found")`.
- etherparse parse error: wrapped `anyhow!("Parse error: {e}")`.

Failed per-packet decodes increment `summary.skipped_packets` (main.rs:139/216) and do not
halt the run.

## pcap_file::DataLink leakage (Smell #7)

`DataLink` from the external `pcap-file` crate leaks across the crate boundary: it appears
in `PcapSource.datalink`, passes into `decode_packet`, and is imported in some test files.
This is a low-severity smell (advisory; C-4/C-5 boundary). A future abstraction would
define a domain `LinkType` enum and map pcap-file's enum to it at the reader boundary.

## BC references

BC-RDR-002 (link-type accept/reject), BC-DEC-001..005 (per-link decoding paths).
