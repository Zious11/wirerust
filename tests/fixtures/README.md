# Test fixtures

PCAP capture files used by the integration test suite.

## Licensing notice — read before adding fixtures

These captures are **third-party packet captures**, predominantly sourced
from the [Wireshark SampleCaptures wiki](https://wiki.wireshark.org/SampleCaptures).

That wiki carries **no per-file license**. Its site-wide content license
is the GNU GPL, and individual capture files carry no explicit
permissive statement. They are therefore **not** public-domain or
permissively licensed, and cannot honestly be labelled as such.

They are included here on a **de-facto basis**: the captures are tiny,
anonymized protocol demonstrations that are redistributed extremely
widely (including inside the Wireshark source tree itself), and the
repository has shipped wiki-sourced fixtures since before this notice
existed. This is a pragmatic accommodation, not a clean licensing
position. If strict licensing hygiene is later required, this directory
should be re-sourced from explicitly-permissively-licensed captures or
from a first-party synthetic generator.

**Do not** add captures containing live malware command-and-control or
exploit traffic — they trip GitHub's malware policy and cause antivirus
false-positives for anyone cloning the repository.

## Fixtures with recorded provenance

| File | Source | Notes |
|------|--------|-------|
| `tcp-ecn-sample.pcap` | [Wireshark SampleCaptures wiki](https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/tcp-ecn-sample.pcap) | TCP transfer with ECN (RFC 3168). 479 TCP packets, 2 flows. Reassembly-heavy benign baseline (P2.05). |
| `tcp-ethereal-file1.trace` | [Wireshark SampleCaptures wiki](https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/tcp-ethereal-file1.trace) | Large multi-segment TCP/HTTP transfer. 218 TCP packets, 1 flow, ~150 KB reassembled. Reassembly-heavy benign baseline (P2.05). |
| `nfs_bad_stalls.cap` | [Wireshark SampleCaptures wiki](https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/nfs_bad_stalls.cap) | Snaplen-96 NFS-over-TCP capture (7038 packets). Exercises the snaplen-truncated reader + decoder paths end-to-end. A genuine "bad stalls" capture whose NFS flow trips the out-of-window anomaly — a positive detection fixture, **not** a benign baseline. See below. |

The first two are exercised by `tests/fixture_reassembly_tests.rs` as
benign calibration baselines: they drive heavy reassembly while
producing zero anomaly findings, confirming the overlap / small-segment
/ out-of-window thresholds do not false-positive on normal traffic.

The remaining fixtures in this directory predate this README; their
individual provenance is not recorded here. Documenting them is tracked
as a separate backlog item (the "dead-fixtures README or removal"
lesson).

## Snaplen-truncated captures (`nfs_bad_stalls.cap`)

Snaplen-truncated captures — files where a packet's original on-wire
length exceeds the capture's `snaplen` (e.g. produced by
`tcpdump -s 96`) — are common in real-world forensics. wirerust handles
them end-to-end:

- **Reader.** `pcap-file` 2.0.0's validated read path wrongly rejects
  snaplen-truncated records with
  `Invalid field value: PacketHeader orig_len > snap_len`, so the reader
  takes the unvalidated raw-record path instead.
- **Decoder.** `etherparse`'s strict parser rejects an IP header whose
  `total_length` / `payload_length` overshoots the captured bytes — a
  *length* error. On that error class only, the decoder falls back to
  `etherparse`'s lax parser, which clamps lengths to the captured slice
  (matching how Wireshark and tcpdump dissect truncated captures rather
  than dropping them). A structural error — bad header version, bad
  IHL, bad TCP data-offset — is genuine corruption and is still
  rejected; lax recovery never admits a malformed packet.

`nfs_bad_stalls.cap` is the end-to-end regression fixture for both. All
7037 of its IPv4 packets decode (the single non-IP ARP frame is dropped,
as expected); only the application-layer *payload* bytes beyond the
96-byte snaplen are unavoidably absent. Because the TCP/IP headers
survive intact, the reassembly engine sees this capture's true sequence
behaviour — and its NFS flow legitimately trips the out-of-window
anomaly threshold, which is what makes it a positive detection fixture.
