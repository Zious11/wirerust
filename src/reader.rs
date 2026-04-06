use std::io::Read;

use anyhow::{Context, Result};
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
}

impl PcapSource {
    pub fn from_pcap_reader<R: Read>(reader: R) -> Result<Self> {
        let mut pcap_reader = PcapReader::new(reader).context("Failed to parse pcap header")?;

        let mut packets = Vec::new();

        while let Some(raw_packet) = pcap_reader.next_packet() {
            let raw_packet = raw_packet.context("Failed to read packet")?;
            packets.push(RawPacket {
                timestamp_secs: raw_packet.timestamp.as_secs() as u32,
                timestamp_usecs: raw_packet.timestamp.subsec_micros(),
                data: raw_packet.data.into_owned(),
            });
        }

        Ok(PcapSource { packets })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        Self::from_pcap_reader(reader)
    }
}
