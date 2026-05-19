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

Both are exercised by `tests/fixture_reassembly_tests.rs` as benign
calibration baselines: they drive heavy reassembly while producing zero
anomaly findings, confirming the overlap / small-segment /
out-of-window thresholds do not false-positive on normal traffic.

The remaining fixtures in this directory predate this README; their
individual provenance is not recorded here. Documenting them is tracked
as a separate backlog item (the "dead-fixtures README or removal"
lesson).

## Known reader limitation

`wirerust` (via the `pcap-file` crate) currently **cannot read
snaplen-truncated captures** — files where a packet's original on-wire
length exceeds the capture's `snaplen` (e.g. produced by
`tcpdump -s 96`). Such files fail with
`Invalid field value: PacketHeader orig_len > snap_len`.

This blocked the addition of the wiki's `nfs_bad_stalls.cap` (a
snaplen-96 retransmission-stall capture that would otherwise have been
a good benign reassembly baseline). Snaplen-truncated captures are
common in real-world forensics, so this is a genuine functional gap
worth a future fix — tracked as a follow-up, not addressed here.
