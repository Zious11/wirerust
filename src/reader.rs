//! Pcap-format capture-file reader.
//!
//! [`PcapSource::from_file`] reads a `.pcap` (libpcap) file into memory and
//! exposes the link-layer type plus a `Vec<RawPacket>` of frame timestamps
//! and bytes. `pcapng` is intentionally NOT supported here (see LESSON-P0.02
//! in the brownfield-ingest synthesis); the directory-glob path in
//! `src/main.rs` enforces that.
//!
//! ## Snaplen-truncated captures
//!
//! Captures taken with a snap length (`tcpdump -s 96`, etc.) record packets
//! whose on-wire `orig_len` legitimately exceeds the file's `snaplen` — only
//! the first `snaplen` bytes are actually stored. `pcap-file` 2.0.0's
//! validated `next_packet()` path wrongly rejects every such record with
//! `PacketHeader orig_len > snap_len` (the real invariant is only
//! `incl_len <= snaplen`). Because snaplen-truncated captures are common in
//! real-world forensics, this reader uses the *unvalidated*
//! `next_raw_packet()` path and converts the timestamp itself. Buffer safety
//! is unaffected: `RawPcapPacket` parsing still bounds `data` to exactly the
//! captured `incl_len` bytes (an over-long `incl_len` yields a hard error).
//!
//! For very large captures the all-in-memory model is a known limitation;
//! see the technical-debt register for a streaming-reader follow-up.

use std::io::Read;

use anyhow::{Context, Result, anyhow};
use pcap_file::pcap::PcapReader;
use pcap_file::{DataLink, TsResolution};

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

        let header = pcap_reader.header();
        let datalink = header.datalink;
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

        // Whether the per-packet `ts_frac` field is microseconds or
        // nanoseconds is set by the file's magic number (see module docs
        // on why the raw-record path is used).
        let ts_resolution = header.ts_resolution;
        let mut packets = Vec::new();

        while let Some(raw_packet) = pcap_reader.next_raw_packet() {
            let raw_packet = raw_packet.context("Failed to read packet")?;
            let timestamp_usecs = match ts_resolution {
                TsResolution::MicroSecond => raw_packet.ts_frac,
                TsResolution::NanoSecond => raw_packet.ts_frac / 1_000,
            };
            packets.push(RawPacket {
                timestamp_secs: raw_packet.ts_sec,
                timestamp_usecs,
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
