//! Protocol Coverage Catalog — SS-18, component C-26.
//!
//! Provides the `KNOWN_PROTOCOLS` static catalog, `SUPPORTED_PORTS` compile-time
//! constant, and three pure-core partition functions (`all_protocols`,
//! `supported_protocols`, `unsupported_protocols`).
//!
//! Architecture: pure-core leaf; no imports from any other wirerust module.
//! MUST NOT depend on `dispatcher`, `analyzer/*`, `reassembly/*`, `reporter/*`,
//! `mitre`, `findings`, or any other wirerust module (BC-2.05.010 PC-4).

/// Classification of a known protocol by domain.
///
/// Exactly two variants — no `L2` variant (ADR-012 Decision 7).
/// Layer-2 membership is expressed by `transport: Transport::LinkLayer` and
/// `port_detectable: false`, not by a third category variant.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolCategory {
    /// Industrial Control System / operational-technology protocol.
    ICS,
    /// Information Technology protocol (IT infrastructure or general-purpose).
    IT,
}

/// Transport layer used by a protocol entry.
///
/// Three variants — distinct from `dispatcher::TransportProto` (which has only
/// `Tcp` and `Udp`). MUST NOT be imported from or confused with that type.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    /// TCP transport.
    Tcp,
    /// UDP transport.
    Udp,
    /// Layer-2 / link-layer protocol; has no TCP or UDP port.
    LinkLayer,
}

/// A single entry in the `KNOWN_PROTOCOLS` coverage catalog.
///
/// All fields are `&'static` or primitive; no heap allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct KnownProtocol {
    /// Short display name, e.g. `"Modbus/TCP"`, `"DNP3"`, `"IEC 61850 GOOSE"`.
    pub name: &'static str,
    /// ICS or IT domain classification.
    pub category: ProtocolCategory,
    /// Primary transport layer for this catalog entry (single-canonical-transport model).
    pub transport: Transport,
    /// Canonical port number(s); empty slice for `LinkLayer` entries.
    pub canonical_ports: &'static [u16],
    /// IEEE EtherType for link-layer protocols; `None` for TCP/UDP entries and ARP.
    pub ethertype: Option<u16>,
    /// `true` when the protocol can be detected by TCP/UDP port matching; `false`
    /// for `transport == LinkLayer` entries (no port to key on).
    pub port_detectable: bool,
    /// Human-readable one-line description for CLI output.
    pub description: &'static str,
}

/// Compile-time constant equal to the full set of ports wirerust actively dissects by any
/// mechanism. Port → dissection path:
/// - 502   → `DispatchTarget::Modbus` in `dispatcher.rs::classify()`
/// - 20000 → `DispatchTarget::Dnp3` in `dispatcher.rs::classify()`
/// - 44818 → `DispatchTarget::Enip` in `dispatcher.rs::classify()`
/// - 2404  → `DispatchTarget::Iec104` in `dispatcher.rs::classify()` (Rule 8, ADR-013)
/// - 443, 8443 → `DispatchTarget::Tls` in `dispatcher.rs::classify()`
/// - 80, 8080 → `DispatchTarget::Http` in `dispatcher.rs::classify()`
/// - 53 → DNS decode-loop in `main.rs` (`dns_analyzer.can_decode()`); NO
///   `DispatchTarget::Dns` variant in `classify()`. DNS/53 not mirroring
///   `classify()` is PERMANENT and BY DESIGN (ADR-012 Decision 5).
///
/// ARP is NOT in this list; it is handled via `DecodedFrame::Arp` (ARP special
/// case in `supported_protocols()`).
pub const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 2404, 443, 8443, 80, 8080, 53];

/// Static catalog of all known ICS/IT protocols that wirerust is aware of.
///
/// Exactly 30 entries in catalog-declaration order: 7 entries in the supported block
/// (Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, HTTP), plus 1 additional
/// supported via port-filter (IEC 60870-5-104), then 22 unsupported entries.
/// IEC 60870-5-104 is functionally supported
/// (port 2404 in `SUPPORTED_PORTS` since STORY-173; BC-2.18.003 PC-1) but is
/// physically still in the ICS Tier-1 block below — membership-by-port-filter
/// pattern: `supported_protocols()` returns it via the port intersection, not by
/// physical placement. Total supported: 8; total unsupported: 22.
///
/// Canonical EtherType values (IEEE RA registry):
/// - GOOSE    = 0x88B8 (35000 decimal) — IEC 61850-8-1 §4
/// - SV       = 0x88BA (35002 decimal) — IEC 61850-8-1 §4
/// - PROFINET = 0x8892 (34962 decimal) — PROFINET Acyclic Real-Time / DCP
/// - EtherCAT = 0x88A4 (34980 decimal) — EtherCAT Technology Group
/// - POWERLINK= 0x88AB (34987 decimal) — EPSG V2 current standard
pub const KNOWN_PROTOCOLS: &[KnownProtocol] = &[
    // -----------------------------------------------------------------------
    // Supported (7 physically here) — catalog-declaration order per BC-2.18.003 v1.3 PC-2
    // IEC 60870-5-104 is the 8th supported protocol; it is promoted-in-place
    // (physically in the Tier-1 block below; membership via port-filter on port 2404).
    // -----------------------------------------------------------------------
    KnownProtocol {
        name: "Modbus/TCP",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[502],
        ethertype: None,
        port_detectable: true,
        description: "Modbus over TCP — IANA/Modbus App Protocol v1.1b3; \
                      ICS field-device register read/write",
    },
    KnownProtocol {
        name: "DNP3",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[20000],
        ethertype: None,
        port_detectable: true,
        description: "Distributed Network Protocol 3 over TCP — IEEE Std 1815-2012; \
                      SCADA/substation master-to-outstation",
    },
    KnownProtocol {
        name: "EtherNet/IP + CIP",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[44818],
        ethertype: None,
        port_detectable: true,
        description: "EtherNet/IP with Common Industrial Protocol — ODVA; \
                      TCP port 44818 (explicit messaging)",
    },
    KnownProtocol {
        name: "TLS",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[443, 8443],
        ethertype: None,
        port_detectable: true,
        description: "Transport Layer Security — RFC 8446; \
                      encrypted tunnel for HTTPS and OT-over-TLS",
    },
    KnownProtocol {
        name: "ARP",
        category: ProtocolCategory::IT,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: None,
        port_detectable: false,
        description: "Address Resolution Protocol — RFC 826; \
                      detected via DecodedFrame::Arp (EtherType 0x0806)",
    },
    KnownProtocol {
        name: "DNS",
        category: ProtocolCategory::IT,
        transport: Transport::Udp,
        canonical_ports: &[53],
        ethertype: None,
        port_detectable: true,
        description: "Domain Name System — RFC 1035; \
                      dissected via decode-loop path (no DispatchTarget::Dns variant — by design)",
    },
    KnownProtocol {
        name: "HTTP",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[80, 8080],
        ethertype: None,
        port_detectable: true,
        description: "Hypertext Transfer Protocol — RFC 9110; \
                      HMI web interfaces, historian REST APIs",
    },
    // -----------------------------------------------------------------------
    // ICS Tier-1, Port-Detectable (9)
    // IEC 60870-5-104 (the first SUPPORTED entry in this block) is functionally
    // SUPPORTED via port 2404; the remaining 8 entries are unsupported. IEC-104
    // remains physically here — promoted in place, membership determined by
    // port-filter (STORY-173).
    // -----------------------------------------------------------------------
    KnownProtocol {
        name: "S7comm",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[102],
        ethertype: None,
        port_detectable: true,
        description: "Siemens S7 communication protocol — ISO-on-TCP/TPKT (RFC 1006); \
                      Siemens S7-300/400 PLCs; port 102 collision (S7comm/S7comm-plus/MMS/ICCP)",
    },
    KnownProtocol {
        name: "S7comm-plus",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[102],
        ethertype: None,
        port_detectable: true,
        description: "Siemens S7comm+ (TIA Portal) — ISO-on-TCP/TPKT (RFC 1006); \
                      S7-1200/1500 PLCs; port 102 collision (S7comm/S7comm-plus/MMS/ICCP)",
    },
    KnownProtocol {
        name: "IEC 60870-5-104",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[2404],
        ethertype: None,
        port_detectable: true,
        description: "IEC 60870-5-104 (IEC-104) — TCP-mapped IEC 60870-5-101; \
                      SCADA telecontrol; IANA-registered TCP port 2404",
    },
    KnownProtocol {
        name: "IEC 61850 MMS",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[102],
        ethertype: None,
        port_detectable: true,
        description: "IEC 61850 Manufacturing Message Specification — ISO-on-TCP/TPKT \
                      (RFC 1006); substation automation; port 102 collision",
    },
    KnownProtocol {
        name: "BACnet/IP",
        category: ProtocolCategory::ICS,
        transport: Transport::Udp,
        canonical_ports: &[47808],
        ethertype: None,
        port_detectable: true,
        description: "BACnet over IP — ASHRAE 135-2016 Annex J; \
                      UDP port 47808 (0xBAC0); building automation and control",
    },
    KnownProtocol {
        name: "OPC-UA",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[4840],
        ethertype: None,
        port_detectable: true,
        description: "OPC Unified Architecture binary — OPC Foundation; \
                      IANA-registered TCP port 4840; ICS data exchange",
    },
    KnownProtocol {
        name: "PROFINET RPC",
        category: ProtocolCategory::ICS,
        transport: Transport::Udp,
        canonical_ports: &[34962, 34963, 34964],
        ethertype: None,
        port_detectable: true,
        description: "PROFINET RPC (cyclic data / IO) — UDP ports 34962–34964; \
                      distinct from PROFINET RT/DCP (L2/EtherType 0x8892)",
    },
    KnownProtocol {
        name: "ICCP/TASE.2",
        category: ProtocolCategory::ICS,
        transport: Transport::Tcp,
        canonical_ports: &[102],
        ethertype: None,
        port_detectable: true,
        description: "Inter-Control Center Communications Protocol / TASE.2 — \
                      IEC 60870-6; ISO-on-TCP/TPKT (RFC 1006); port 102 collision",
    },
    KnownProtocol {
        name: "HART-IP",
        category: ProtocolCategory::ICS,
        transport: Transport::Udp,
        canonical_ports: &[5094],
        ethertype: None,
        port_detectable: true,
        description: "HART-IP — FieldComm Group; UDP port 5094 (canonical); \
                      TCP also supported per HART-IP specification",
    },
    // -----------------------------------------------------------------------
    // L2/Multicast — NOT Port-Detectable (5)
    // -----------------------------------------------------------------------
    KnownProtocol {
        name: "IEC 61850 GOOSE",
        category: ProtocolCategory::ICS,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: Some(0x88B8), // 35000 decimal — IEC 61850-8-1 §4; IEEE RA "IEC GOOSE"
        port_detectable: false,
        description: "IEC 61850 Generic Object Oriented Substation Event — \
                      EtherType 0x88B8 (35000); L2 multicast; substation protection",
    },
    KnownProtocol {
        name: "IEC 61850 Sampled Values",
        category: ProtocolCategory::ICS,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: Some(0x88BA), // 35002 decimal — IEC 61850-8-1 §4
        port_detectable: false,
        description: "IEC 61850 Sampled Values — EtherType 0x88BA (35002); \
                      L2 multicast; merging unit current/voltage samples",
    },
    KnownProtocol {
        name: "PROFINET RT/DCP",
        category: ProtocolCategory::ICS,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: Some(0x8892), // 34962 decimal — IEEE RA "PROFINET Acyclic Real-Time / DCP"
        port_detectable: false,
        description: "PROFINET Real-Time / Device Configuration Protocol — \
                      EtherType 0x8892 (34962); L2; Siemens/Profibus device discovery",
    },
    KnownProtocol {
        name: "EtherCAT",
        category: ProtocolCategory::ICS,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: Some(0x88A4), // 34980 decimal — IEEE RA "EtherCAT Technology Group"
        port_detectable: false,
        description: "EtherCAT — EtherType 0x88A4 (34980); \
                      L2; Beckhoff real-time fieldbus for motion control",
    },
    KnownProtocol {
        name: "Ethernet POWERLINK",
        category: ProtocolCategory::ICS,
        transport: Transport::LinkLayer,
        canonical_ports: &[],
        ethertype: Some(0x88AB), // 34987 decimal — IEEE RA; EPSG V2 current standard
        port_detectable: false,
        description: "Ethernet POWERLINK — EtherType 0x88AB (34987); \
                      L2; EPSG V2 (current); obsolete V1 value 0x3E3F intentionally excluded",
    },
    // -----------------------------------------------------------------------
    // IT Core Unsupported (9)
    // -----------------------------------------------------------------------
    KnownProtocol {
        name: "SSH",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[22],
        ethertype: None,
        port_detectable: true,
        description: "Secure Shell — RFC 4253; remote admin of PLCs/gateways; \
                      lateral movement vector in OT environments",
    },
    KnownProtocol {
        name: "SMB",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[445],
        ethertype: None,
        port_detectable: true,
        description: "Server Message Block — MS-SMB2; engineering workstations; \
                      WannaCry/Industroyer attack vector",
    },
    KnownProtocol {
        name: "RDP",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[3389],
        ethertype: None,
        port_detectable: true,
        description: "Remote Desktop Protocol — MS-RDPBCGR; HMI/EWS remote access; \
                      top OT intrusion vector",
    },
    KnownProtocol {
        name: "FTP",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[21],
        ethertype: None,
        port_detectable: true,
        description: "File Transfer Protocol — RFC 959; \
                      firmware and configuration transfer to/from field devices",
    },
    KnownProtocol {
        name: "Telnet",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[23],
        ethertype: None,
        port_detectable: true,
        description: "Telnet — RFC 854; legacy cleartext device CLI; \
                      common on older PLCs and network equipment",
    },
    KnownProtocol {
        name: "SNMP",
        category: ProtocolCategory::IT,
        transport: Transport::Udp,
        canonical_ports: &[161, 162],
        ethertype: None,
        port_detectable: true,
        description: "Simple Network Management Protocol — RFC 3411; \
                      device management and monitoring (traps on UDP/162)",
    },
    KnownProtocol {
        name: "NTP",
        category: ProtocolCategory::IT,
        transport: Transport::Udp,
        canonical_ports: &[123],
        ethertype: None,
        port_detectable: true,
        description: "Network Time Protocol — RFC 5905; \
                      time synchronisation critical for SV/GOOSE/SCADA timestamps",
    },
    KnownProtocol {
        name: "SMTP",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[25],
        ethertype: None,
        port_detectable: true,
        description: "Simple Mail Transfer Protocol — RFC 5321; \
                      alarm email from historians and RTUs",
    },
    KnownProtocol {
        name: "LDAP",
        category: ProtocolCategory::IT,
        transport: Transport::Tcp,
        canonical_ports: &[389],
        ethertype: None,
        port_detectable: true,
        description: "Lightweight Directory Access Protocol — RFC 4511; \
                      Active Directory authentication in IT/OT DMZ environments",
    },
];

/// Returns the full `KNOWN_PROTOCOLS` slice.
///
/// Pure function; no I/O; same call always returns the same result.
///
/// (BC-2.18.004 v1.2 PC-1; BC-2.18.003 v1.3 Invariant 2)
pub fn all_protocols() -> &'static [KnownProtocol] {
    KNOWN_PROTOCOLS
}

/// Returns entries from `KNOWN_PROTOCOLS` whose `canonical_ports` intersect
/// `SUPPORTED_PORTS`, plus the ARP entry (ARP special case, BC-2.18.003
/// Invariant 3 — `|| p.name == "ARP"` is explicit).
///
/// Returns exactly 8 entries: Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP,
/// DNS, HTTP, IEC 60870-5-104. IEC 60870-5-104 is included because port 2404
/// was added to `SUPPORTED_PORTS` in STORY-173 (BC-2.18.003 PC-1).
///
/// Pure function; no I/O.
///
/// (BC-2.18.003 v1.3 PC-1, PC-3, Invariants 2–3; BC-2.18.004 v1.2 PC-1..5)
pub fn supported_protocols() -> Vec<&'static KnownProtocol> {
    KNOWN_PROTOCOLS
        .iter()
        .filter(|p| {
            p.canonical_ports
                .iter()
                .any(|port| SUPPORTED_PORTS.contains(port))
                || p.name == "ARP"
        })
        .collect()
}

/// Returns the exact complement of `supported_protocols()` within
/// `KNOWN_PROTOCOLS` — i.e., every entry NOT in `supported_protocols()`.
///
/// Derived as the complement; not a hand-maintained list
/// (BC-2.18.003 Invariant 4). Returns exactly 22 entries (after IEC-104 promoted in STORY-173).
///
/// Pure function; no I/O.
///
/// (BC-2.18.003 v1.3 PC-2, Invariants 4–5; BC-2.18.004 v1.2 PC-1..5)
pub fn unsupported_protocols() -> Vec<&'static KnownProtocol> {
    let supported: Vec<_> = supported_protocols().iter().map(|p| p.name).collect();
    KNOWN_PROTOCOLS
        .iter()
        .filter(|p| !supported.contains(&p.name))
        .collect()
}

// ── VP-041: Protocol Coverage Catalog partition — Kani JUSTIFIED-DEFERRED ──────
//
// F6 targeted-hardening disposition for VP-041. The verification-architecture
// designates VP-041 as a `proptest` property (harnesses
// `proptest_vp041_oracle_cross_check` + `proptest_vp041_partition_invariant` in
// `tests/protocols_tests.rs`; both green). Kani/CBMC is NOT applied here, by design:
//
//  1. NO SYMBOLIC INPUT. `KNOWN_PROTOCOLS` and `SUPPORTED_PORTS` are compile-time
//     constants; the partition functions take no arguments. A Kani harness would
//     have zero `kani::any()` inputs, so bounded model checking degenerates to a
//     single concrete execution — exactly what the deterministic proptest harnesses
//     (and the `fuzz_coverage_gap_classify` completeness oracle) already cover.
//     BMC adds no additional state-space coverage over a constant.
//
//  2. CBMC INTRACTABILITY. The partition is expressed over `Vec<&KnownProtocol>`
//     with `&'static str` name equality (`supported.contains(&p.name)`, nested
//     30x8 / 8x22 string comparisons). Modeling heap `Vec` growth plus byte-wise
//     `str` memcmp exploded the SAT formula: a trial harness ran CBMC (cadical,
//     --unwind 64, --object-bits 16) for >12 min of solver time without converging.
//     Shipping a non-terminating proof harness would violate the repo's
//     no-flaky/non-gating-stub rule (CLAUDE.md W7.1).
//
// Assurance for VP-041 is therefore provided by: (a) the two designated proptest
// harnesses, and (b) the new `fuzz_coverage_gap_classify` target, which asserts
// `|supported| + |unsupported| == |KNOWN_PROTOCOLS|` on every iteration under the
// ASan/UBSan-instrumented libFuzzer build.
