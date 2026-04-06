# Multi-Link-Type Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support Raw IP (101), IPv4 (228), IPv6 (229), and Linux Cooked SLL (113) pcap link types in addition to Ethernet, with clear errors for unsupported types.

**Architecture:** Read link type from pcap global header, dispatch to the correct etherparse parser (`from_ethernet`, `from_ip`, `from_linux_sll`), and surface decode errors to the user instead of silently dropping packets.

**Tech Stack:** etherparse 0.16 (`SlicedPacket::from_ip`, `from_linux_sll`), pcap-file 2.0 (`DataLink` enum, `PcapReader::header().datalink`)

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `src/reader.rs` | Modify | Store `DataLink` from pcap header, reject unsupported types |
| `src/decoder.rs` | Modify | Accept `DataLink` param, dispatch to correct parser |
| `src/main.rs` | Modify | Pass `datalink` through, count and surface decode errors |
| `src/summary.rs` | Modify | Add `skipped_packets` field |
| `src/reporter/terminal.rs` | Modify | Show skipped packets in output |
| `src/reporter/json.rs` | Modify | Include skipped packets in JSON |
| `tests/reader_tests.rs` | Modify | Add unsupported link type test, verify datalink field |
| `tests/decoder_tests.rs` | Modify | Update existing tests with `DataLink::ETHERNET`, add Raw IP test |
| `tests/integration_test.rs` | Modify | Update `decode_packet` call to pass datalink |
| `tests/linktype_integration_tests.rs` | Create | Integration tests with real pcap fixtures |

---

### Task 1: Reader — Store DataLink and Reject Unsupported Types

**Files:**
- Modify: `src/reader.rs`
- Test: `tests/reader_tests.rs`

- [ ] **Step 1: Write failing test for unsupported link type**

Add this test to `tests/reader_tests.rs`:

```rust
#[test]
fn test_unsupported_link_type_rejected() {
    let mut buf = Vec::new();
    // Global header with link type 105 (IEEE802_11)
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes()); // magic
    buf.extend_from_slice(&2u16.to_le_bytes()); // version major
    buf.extend_from_slice(&4u16.to_le_bytes()); // version minor
    buf.extend_from_slice(&0i32.to_le_bytes()); // thiszone
    buf.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&105u32.to_le_bytes()); // network: IEEE802_11 (unsupported)

    let cursor = Cursor::new(buf);
    let result = PcapSource::from_pcap_reader(cursor);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unsupported"),
        "Error should mention 'Unsupported', got: {err_msg}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reader_tests test_unsupported_link_type_rejected`
Expected: FAIL — `PcapSource::from_pcap_reader` currently succeeds for any link type.

- [ ] **Step 3: Write failing test for datalink field on PcapSource**

Add this test to `tests/reader_tests.rs`:

```rust
use pcap_file::DataLink;

#[test]
fn test_pcap_source_stores_datalink() {
    let data = minimal_pcap_bytes(); // Uses network=1 (Ethernet)
    let cursor = Cursor::new(data);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    assert_eq!(source.datalink, DataLink::ETHERNET);
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test --test reader_tests test_pcap_source_stores_datalink`
Expected: FAIL — `PcapSource` has no `datalink` field.

- [ ] **Step 5: Implement reader changes**

In `src/reader.rs`, add the `DataLink` import and modify `PcapSource`:

```rust
use std::io::Read;

use anyhow::{Context, Result, anyhow};
use pcap_file::DataLink;
use pcap_file::pcap::PcapReader;

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub timestamp_secs: u32,
    pub timestamp_usecs: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct PcapSource {
    pub packets: Vec<RawPacket>,
    pub datalink: DataLink,
}

impl PcapSource {
    pub fn from_pcap_reader<R: Read>(reader: R) -> Result<Self> {
        let mut pcap_reader = PcapReader::new(reader).context("Failed to parse pcap header")?;

        let datalink = pcap_reader.header().datalink;
        match datalink {
            DataLink::ETHERNET
            | DataLink::RAW
            | DataLink::IPV4
            | DataLink::IPV6
            | DataLink::LINUX_SLL => {}
            other => {
                return Err(anyhow!(
                    "Unsupported pcap link type: {other:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"
                ));
            }
        }

        let mut packets = Vec::new();

        while let Some(raw_packet) = pcap_reader.next_packet() {
            let raw_packet = raw_packet.context("Failed to read packet")?;
            packets.push(RawPacket {
                timestamp_secs: raw_packet.timestamp.as_secs() as u32,
                timestamp_usecs: raw_packet.timestamp.subsec_micros(),
                data: raw_packet.data.into_owned(),
            });
        }

        Ok(PcapSource { packets, datalink })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        Self::from_pcap_reader(reader)
    }
}
```

- [ ] **Step 6: Run both new tests to verify they pass**

Run: `cargo test --test reader_tests`
Expected: All 4 tests pass (2 existing + 2 new).

- [ ] **Step 7: Commit**

```bash
git add src/reader.rs tests/reader_tests.rs
git commit -m "feat: store DataLink from pcap header, reject unsupported link types"
```

---

### Task 2: Decoder — Multi-Link-Type Dispatch

**Files:**
- Modify: `src/decoder.rs`
- Modify: `tests/decoder_tests.rs`

- [ ] **Step 1: Write failing test for Raw IP decode**

Add this test to `tests/decoder_tests.rs`:

```rust
use pcap_file::DataLink;

fn make_raw_ip_tcp_packet() -> Vec<u8> {
    vec![
        // IPv4 header (20 bytes) — no Ethernet header
        0x45, 0x00, 0x00, 0x28, // version/IHL, DSCP, total length=40
        0x00, 0x01, 0x00, 0x00, // identification, flags/fragment
        0x40, 0x06, 0x00, 0x00, // TTL=64, protocol=TCP, checksum
        0xc0, 0xa8, 0x01, 0x0a, // src: 192.168.1.10
        0xc0, 0xa8, 0x01, 0x01, // dst: 192.168.1.1
        // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ]
}

#[test]
fn test_decode_raw_ip_tcp_packet() {
    let data = make_raw_ip_tcp_packet();
    let parsed = decode_packet(&data, DataLink::RAW).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test decoder_tests test_decode_raw_ip_tcp_packet`
Expected: FAIL — `decode_packet` doesn't accept a `DataLink` parameter.

- [ ] **Step 3: Implement decoder changes**

In `src/decoder.rs`, add the `DataLink` import and modify `decode_packet`:

```rust
use std::net::IpAddr;

use anyhow::{Result, anyhow};
use etherparse::SlicedPacket;
use pcap_file::DataLink;
use serde::Serialize;

// ... (Protocol, TransportInfo, ParsedPacket, app_protocol_hint unchanged) ...

pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket> {
    let sliced = match datalink {
        DataLink::ETHERNET => SlicedPacket::from_ethernet(data),
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data),
        DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data),
        other => return Err(anyhow!("Unsupported link type: {other:?}")),
    }
    .map_err(|e| anyhow!("Parse error: {e}"))?;

    // ... (rest of function unchanged — net/transport/payload extraction) ...
```

Only the first two lines of the function body change. Everything from `let (src_ip, dst_ip, ip_protocol) = match &sliced.net {` onward stays exactly the same.

- [ ] **Step 4: Update existing decoder tests to pass DataLink::ETHERNET**

In `tests/decoder_tests.rs`, add the import and update the three existing tests:

Add at the top (if not already there):
```rust
use pcap_file::DataLink;
```

Update `test_decode_tcp_packet`:
```rust
let parsed = decode_packet(&data, DataLink::ETHERNET).unwrap();
```

Update `test_decode_udp_dns_packet`:
```rust
let parsed = decode_packet(&data, DataLink::ETHERNET).unwrap();
```

Update `test_decode_invalid_packet`:
```rust
assert!(decode_packet(&garbage, DataLink::ETHERNET).is_err());
```

- [ ] **Step 5: Run all decoder tests to verify they pass**

Run: `cargo test --test decoder_tests`
Expected: All 4 tests pass (3 existing updated + 1 new).

- [ ] **Step 6: Commit**

```bash
git add src/decoder.rs tests/decoder_tests.rs
git commit -m "feat: decode_packet dispatches by link type (Ethernet, Raw IP, Linux SLL)"
```

---

### Task 3: Wire Up Callers — main.rs and Integration Test

**Files:**
- Modify: `src/main.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Update main.rs to pass datalink to decode_packet**

In `src/main.rs`, in the `run_analyze` function, change the decode call at line 83 from:

```rust
if let Ok(parsed) = decode_packet(&raw.data) {
```

to:

```rust
if let Ok(parsed) = decode_packet(&raw.data, source.datalink) {
```

Make the same change in `run_summary` at line 133, from:

```rust
if let Ok(parsed) = decode_packet(&raw.data) {
```

to:

```rust
if let Ok(parsed) = decode_packet(&raw.data, source.datalink) {
```

- [ ] **Step 2: Update integration_test.rs**

In `tests/integration_test.rs`, add the import:

```rust
use pcap_file::DataLink;
```

Change line 51 from:

```rust
if let Ok(parsed) = decode_packet(&raw.data) {
```

to:

```rust
if let Ok(parsed) = decode_packet(&raw.data, DataLink::ETHERNET) {
```

The integration test constructs an Ethernet pcap, so `DataLink::ETHERNET` is correct.

- [ ] **Step 3: Verify everything compiles and all tests pass**

Run: `cargo test`
Expected: All tests pass. The project now compiles with the new API.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs tests/integration_test.rs
git commit -m "fix: pass DataLink to decode_packet in main pipeline and integration test"
```

---

### Task 4: Error Surfacing — Decode Error Counting

**Files:**
- Modify: `src/summary.rs`
- Modify: `src/reporter/terminal.rs`
- Modify: `src/reporter/json.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add skipped_packets field to Summary**

In `src/summary.rs`, add a `skipped_packets` field and a method to report it:

Add the field to the `Summary` struct:

```rust
#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub skipped_packets: u64,
    hosts: HashSet<IpAddr>,
    protocols: HashMap<Protocol, u64>,
    services: HashMap<String, u64>,
}
```

Update `Summary::new()` to initialize it:

```rust
pub fn new() -> Self {
    Summary {
        total_packets: 0,
        total_bytes: 0,
        skipped_packets: 0,
        hosts: HashSet::new(),
        protocols: HashMap::new(),
        services: HashMap::new(),
    }
}
```

- [ ] **Step 2: Update terminal reporter to show skipped packets**

In `src/reporter/terminal.rs`, update the header section (around line 23) to show skipped packets when non-zero. Change:

```rust
out.push_str(&format!(
    "  Packets: {}  Bytes: {}  Hosts: {}\n\n",
    summary.total_packets,
    summary.total_bytes,
    summary.unique_hosts().len(),
));
```

to:

```rust
out.push_str(&format!(
    "  Packets: {}  Bytes: {}  Hosts: {}\n",
    summary.total_packets,
    summary.total_bytes,
    summary.unique_hosts().len(),
));
if summary.skipped_packets > 0 {
    let warning = format!("  Skipped: {} packets (decode errors)\n", summary.skipped_packets);
    if self.use_color {
        out.push_str(&warning.yellow().to_string());
    } else {
        out.push_str(&warning);
    }
}
out.push('\n');
```

- [ ] **Step 3: Update JSON reporter to include skipped_packets**

In `src/reporter/json.rs`, add `skipped_packets` to the JSON output. Change the summary object in `render()`:

```rust
let output = json!({
    "summary": {
        "total_packets": summary.total_packets,
        "total_bytes": summary.total_bytes,
        "skipped_packets": summary.skipped_packets,
        "unique_hosts": summary.unique_hosts(),
        "protocols": protocols,
        "services": summary.service_counts(),
    },
    "findings": findings,
    "analyzers": analyzer_summaries,
});
```

- [ ] **Step 4: Add decode error counting and stderr warning to main.rs**

In `src/main.rs`, update `run_analyze` to count errors and warn. Replace the packet processing loop (around line 82-96):

```rust
let mut decode_errors: u64 = 0;

for raw in &source.packets {
    match decode_packet(&raw.data, source.datalink) {
        Ok(parsed) => {
            summary.ingest(&parsed);
            if enable_dns && dns_analyzer.can_decode(&parsed) {
                let findings = dns_analyzer.analyze(&parsed);
                all_findings.extend(findings);
            }
            if let Some(ref mut reasm) = reassembler {
                reasm.process_packet(&parsed, raw.timestamp_secs, &mut stream_handler);
            }
        }
        Err(e) => {
            if decode_errors == 0 {
                eprintln!(
                    "Warning: failed to decode packet ({e}). Further errors counted silently."
                );
            }
            decode_errors += 1;
        }
    }
    pb.inc(1);
}
```

After the file loop closes (before the reporter section), add the skipped count to summary:

```rust
summary.skipped_packets += decode_errors;
```

Wait — `decode_errors` is per-file inside the loop. We need to accumulate across files. Move the counter outside the file loop. The structure is:

In `run_analyze`, before `for target in targets {`:
```rust
let mut total_decode_errors: u64 = 0;
```

Inside the per-file packet loop, replace `decode_errors` with `total_decode_errors`.

After the `for target in targets` loop closes (around line 97), add:
```rust
summary.skipped_packets = total_decode_errors;
```

Apply the same pattern to `run_summary`. Replace the packet loop (around line 132-134):

```rust
let mut total_decode_errors: u64 = 0;

for target in targets {
    let pcap_files = resolve_targets(target)?;
    for path in &pcap_files {
        let source = PcapSource::from_file(path)?;
        for raw in &source.packets {
            match decode_packet(&raw.data, source.datalink) {
                Ok(parsed) => {
                    summary.ingest(&parsed);
                }
                Err(e) => {
                    if total_decode_errors == 0 {
                        eprintln!(
                            "Warning: failed to decode packet ({e}). Further errors counted silently."
                        );
                    }
                    total_decode_errors += 1;
                }
            }
        }
    }
}
summary.skipped_packets = total_decode_errors;
```

- [ ] **Step 5: Run all tests to verify nothing broke**

Run: `cargo test`
Expected: All tests pass. The `skipped_packets` field defaults to 0, so existing tests are unaffected.

- [ ] **Step 6: Commit**

```bash
git add src/summary.rs src/reporter/terminal.rs src/reporter/json.rs src/main.rs
git commit -m "feat: count and surface decode errors instead of silently dropping packets"
```

---

### Task 5: Integration Tests with Real Pcap Fixtures

**Files:**
- Create: `tests/linktype_integration_tests.rs`

- [ ] **Step 1: Write integration tests for all three fixture link types**

Create `tests/linktype_integration_tests.rs`:

```rust
use pcap_file::DataLink;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;

#[test]
fn test_ethernet_pcap_tls() {
    let source = PcapSource::from_file(std::path::Path::new("tests/fixtures/tls.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::ETHERNET);
    assert!(
        source.packets.len() > 0,
        "tls.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from tls.pcap, got 0"
    );
}

#[test]
fn test_raw_ip_pcap_segmented() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/segmented.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::RAW);
    assert!(
        source.packets.len() > 0,
        "segmented.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from segmented.pcap, got 0"
    );
}

#[test]
fn test_ipv4_pcap_http_ooo() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/http-ooo.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::IPV4);
    assert!(
        source.packets.len() > 0,
        "http-ooo.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from http-ooo.pcap, got 0"
    );
}
```

- [ ] **Step 2: Run the integration tests**

Run: `cargo test --test linktype_integration_tests`
Expected: All 3 tests pass — each fixture loads, has packets, and decodes successfully.

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: All tests pass (existing + new).

Run: `cargo clippy -- -D warnings`
Expected: Clean.

- [ ] **Step 4: Commit**

```bash
git add tests/linktype_integration_tests.rs
git commit -m "test: integration tests for Ethernet, Raw IP, and IPv4 pcap fixtures"
```
