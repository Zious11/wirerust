# Evidence Report — STORY-090

**Story:** Summary Data Model — ingest, Service Hints, unique_hosts, Serialization
**Story ID:** STORY-090
**Wave:** 27
**Date recorded:** 2026-05-31
**Recorder:** Demo Recorder agent
**Binary:** `~/.cargo/bin/wirerust` (installed from STORY-090 feature branch)
**Tool:** VHS 0.11.0

---

## Coverage Summary

| AC / EC | Title | Recording | Observable? | Status |
|---------|-------|-----------|-------------|--------|
| AC-001 | `Summary::ingest` increments `total_packets` by 1 | AC-001-005-summary-accumulation | Yes — `Packets: 58` visible in terminal output | COVERED |
| AC-002 | `Summary::ingest` increments `total_bytes` by `packet_len as u64` | AC-001-005-summary-accumulation | Yes — `Bytes: 7542` visible in terminal output | COVERED |
| AC-003 | Both `src_ip` and `dst_ip` inserted into `hosts` HashSet, deduplicated | AC-001-005-summary-accumulation | Yes — `Hosts: 3` (dns-remoteshell has 3 unique IPs across src+dst) | COVERED |
| AC-004 | `protocols[packet.protocol]` incremented per call | AC-001-005-summary-accumulation | Yes — `Udp: 6  Tcp: 52` breakdown visible | COVERED |
| AC-005 | `skipped_packets` NOT set by `ingest`; set by caller | AC-001-005-summary-accumulation | Yes — `Skipped: 73` shown (73 decode errors set by caller, 0 from ingest) | COVERED (unit-test-only for the invariant; CLI shows the observable result) |
| AC-006 | `app_protocol_hint` "HTTP" → `services["HTTP"]` incremented | AC-006-service-hints | Yes — `HTTP: 12` (port 80) and `DNS: 18` (port 53) visible; TLS fixture shows `TLS: 58` (port 443) | COVERED |
| AC-007 | `app_protocol_hint` returns `None` → `services` map unchanged | AC-007-AC-010-error-paths | Yes — slammer.pcap (UDP/1434) shows PROTOCOLS section but NO SERVICES section | COVERED |
| AC-008 | Service attribution is port-based; does NOT consult stream dispatcher | (unit-test-only) | No direct CLI observable — verified by unit test `test_summary_service_is_port_based_not_content_based` which asserts no dispatcher import in summary.rs | NOT DEMOED (internal constraint) |
| AC-009 | `unique_hosts()` returns sorted `Vec<IpAddr>` deduplicated | AC-009-unique-hosts | Yes — `--hosts` shows `192.168.1.1`, `192.168.1.2`, `192.168.1.3` sorted in ascending order | COVERED |
| AC-010 | Empty `Summary` returns empty `Vec` from `unique_hosts()` | AC-007-AC-010-error-paths | Yes — non-existent file path triggers early error exit; zero-packet path is also covered by unit test `test_unique_hosts_empty_when_no_packets` | COVERED (error-path demo + unit test) |
| AC-011 | `unique_hosts()` takes `&self`, non-mutating | (unit-test-only) | No CLI observable — compiler enforces via signature; unit test `test_unique_hosts_is_non_mutating` calls it twice to verify state unchanged | NOT DEMOED (compile-time guarantee) |
| AC-012 | JSON has `total_packets`, `total_bytes`, `skipped_packets` as u64 integers | AC-012-013-json-serialization | Yes — JSON output shows `"total_packets": 58`, `"total_bytes": 7542`, `"skipped_packets": 73` | COVERED |
| AC-013 | Protocol keys in JSON use Debug format (`"Tcp"`, `"Udp"`, `"Icmp"`) | AC-012-013-json-serialization | Yes — JSON shows `"protocols": {"Tcp": 52, "Udp": 6}` with Debug-format keys | COVERED |
| EC-001 | `src_ip == dst_ip` → host appears once in `unique_hosts()` | (unit-test-only) | No standard fixture for loopback-only traffic; covered by unit test | NOT DEMOED |
| EC-002 | Mix of IPv4 and IPv6 → both in `unique_hosts()`, IPv4 sorts before IPv6 | (unit-test-only) | ipv6-ripng.pcap exists but does not produce a clean side-by-side demo; covered by unit test | NOT DEMOED |
| EC-003 | Two packets same protocol → `protocols[protocol] = 2` | AC-001-005-summary-accumulation | Yes — dns-remoteshell shows `Tcp: 52` (52 TCP packets) and `Udp: 6` (6 UDP) | COVERED |
| EC-004 | `packet_len = 0` → `total_bytes` unchanged | (unit-test-only) | No fixture with zero-length packets in CLI path; covered by unit test | NOT DEMOED |
| EC-005 | `services["HTTP"] = 2` → JSON `"services": {"HTTP": 2}` | AC-012-013-json-serialization | Yes — JSON output shows `"services": {"DNS": 18, "HTTP": 12}` confirming multi-count serialization | COVERED |

---

## BC Coverage

| BC | Title | Recording | Status |
|----|-------|-----------|--------|
| BC-2.12.018 | Summary::ingest Increments total_packets, total_bytes, hosts, protocols | AC-001-005-summary-accumulation | COVERED — Packets: 58, Bytes: 7542, Hosts: 3, Tcp: 52, Udp: 6 |
| BC-2.12.019 | Summary::ingest Derives Service Name from app_protocol_hint | AC-006-service-hints | COVERED — DNS:18, HTTP:12 (dns-remoteshell), TLS:58 (tls.pcap), no service (slammer.pcap) |
| BC-2.12.020 | Summary::unique_hosts Returns Sorted Deduplicated Vec<IpAddr> | AC-009-unique-hosts | COVERED — 192.168.1.1, 192.168.1.2, 192.168.1.3 in sorted order |
| BC-2.12.021 | Summary Serializes with total_packets/total_bytes/skipped_packets Fields | AC-012-013-json-serialization | COVERED — JSON shows all three u64 fields plus "Tcp"/"Udp" Debug-format protocol keys |

---

## Not-Demo-able ACs (unit-test-only)

The following ACs cover internal invariants or architectural constraints that have no
distinct CLI-observable signal separate from the unit tests:

| AC | Reason not demoed | Unit test |
|----|-------------------|-----------|
| AC-008 | Port-based vs dispatcher is a code-structure invariant; no observable CLI difference exists | `test_summary_service_is_port_based_not_content_based` |
| AC-011 | `&self` non-mutating is a compile-time guarantee enforced by the Rust borrow checker | `test_unique_hosts_is_non_mutating` |
| EC-001 | Requires loopback-only fixture; no standard fixture in tests/fixtures | `test_unique_hosts_sorted_and_deduplicated` (covers dedup) |
| EC-002 | IPv4/IPv6 mixed sort; ipv6-ripng.pcap decodes but display is noisy; unit test is cleaner | Dedicated unit test |
| EC-004 | Zero-length packets not in any CLI fixture | Dedicated unit test |

---

## Recordings

| File | Format | ACs Covered |
|------|--------|-------------|
| `AC-001-005-summary-accumulation.gif` | GIF | AC-001, AC-002, AC-003, AC-004, AC-005, EC-003 |
| `AC-001-005-summary-accumulation.webm` | WebM | AC-001, AC-002, AC-003, AC-004, AC-005, EC-003 |
| `AC-001-005-summary-accumulation.tape` | VHS script | — |
| `AC-009-unique-hosts.gif` | GIF | AC-009 (BC-2.12.020) |
| `AC-009-unique-hosts.webm` | WebM | AC-009 (BC-2.12.020) |
| `AC-009-unique-hosts.tape` | VHS script | — |
| `AC-006-service-hints.gif` | GIF | AC-006 (BC-2.12.019) |
| `AC-006-service-hints.webm` | WebM | AC-006 (BC-2.12.019) |
| `AC-006-service-hints.tape` | VHS script | — |
| `AC-012-013-json-serialization.gif` | GIF | AC-012, AC-013, EC-005 (BC-2.12.021) |
| `AC-012-013-json-serialization.webm` | WebM | AC-012, AC-013, EC-005 (BC-2.12.021) |
| `AC-012-013-json-serialization.tape` | VHS script | — |
| `AC-007-AC-010-error-paths.gif` | GIF | AC-007, AC-010 |
| `AC-007-AC-010-error-paths.webm` | WebM | AC-007, AC-010 |
| `AC-007-AC-010-error-paths.tape` | VHS script | — |

---

## Fixtures Used

| Fixture | What it demonstrates |
|---------|---------------------|
| `tests/fixtures/dns-remoteshell.pcap` | 58 packets, 7542 bytes, 3 hosts, Tcp:52 Udp:6, DNS:18 HTTP:12, Skipped:73 |
| `tests/fixtures/tls.pcap` | 58 packets, 24105 bytes, 1 host (127.0.0.1), Tcp:58, TLS:58 |
| `tests/fixtures/slammer.pcap` | 1 packet, UDP/1434, no recognized service (AC-007 error path) |
| `tests/fixtures/nonexistent.pcap` | Non-existent file → error exit (AC-010 error path) |

---

_Note: This report covers STORY-090 only (13 ACs + 5 ECs). No cross-story rollups are included (per lesson F-W22-T1)._
