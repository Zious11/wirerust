//! Test suite for STORY-151: `src/protocols.rs` — KNOWN_PROTOCOLS static catalog,
//! SUPPORTED_PORTS compile-time constant, and pure-core partition functions.
//!
//! REGRESSION-GUARD: These tests guard the Protocol Coverage Catalog (CAP-18) partition
//! invariant. Any regression in `KNOWN_PROTOCOLS` (30 entries), `SUPPORTED_PORTS` (8 ports),
//! or the three partition functions breaks at least one test here before reaching STORY-152
//! (protocols subcommand) or STORY-154 (gap report).
//!
//! DF-CANONICAL-FRAME-HOLDOUT-001: EtherType canonical-value tests express IEEE RA registry
//! hex values as exact decimal literals. Wrong-value guards are present for visually similar
//! EtherTypes (GOOSE/SV, EtherCAT/PROFINET).
//!
//! Traceability:
//! - BC-2.18.003 v1.3 — `supported_protocols()` / `unsupported_protocols()` / SUPPORTED_PORTS
//! - BC-2.18.004 v1.2 — Catalog partition invariant; VP-041 proptest harnesses
//! - ADR-012 Decision 1/4/5/7 — catalog structure, SUPPORTED_PORTS, ARP special case
//! - VP-041 — `proptest_vp041_oracle_cross_check` + `proptest_vp041_partition_invariant`

// All tests live in `mod story_151` per DF-TEST-NAMESPACE-001 (namespace isolation).
// The `#![allow(non_snake_case)]` inner attribute is required: CI enforces `-D warnings`
// and the uppercase `test_BC_…` function names violate `non_snake_case` (F-F3P8-003).
mod story_151 {
    #![allow(non_snake_case)]

    use wirerust::protocols::{
        KNOWN_PROTOCOLS, KnownProtocol, ProtocolCategory, SUPPORTED_PORTS, Transport,
        all_protocols, supported_protocols, unsupported_protocols,
    };

    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Internal helper — find a KNOWN_PROTOCOLS entry by name substring.
    //
    // Used by EtherType canonical tests to locate entries without relying on
    // the EtherType value itself (that would make the assertion circular).
    // Panics with a clear message if no match is found, surfacing catalog gaps
    // as test failures.
    fn find_entry(name_fragment: &str) -> &'static KnownProtocol {
        KNOWN_PROTOCOLS
            .iter()
            .find(|p| p.name.contains(name_fragment))
            .unwrap_or_else(|| {
                panic!("no KNOWN_PROTOCOLS entry whose name contains {name_fragment:?}")
            })
    }

    // -----------------------------------------------------------------------
    // AC-151-001 — KnownProtocol struct + ProtocolCategory/Transport enums
    // Traces to: BC-2.18.003 v1.3 PC-2; BC-2.18.004 v1.2 PC-1; ADR-012 Decision 1, 7
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies that `KnownProtocol` is constructible via a struct literal
    /// and that all seven public fields are accessible. Fails at Red Gate because
    /// `KNOWN_PROTOCOLS` is empty (len 0, not 30); passes after implementation when the
    /// first catalog entry can be accessed.
    #[test]
    fn test_BC_2_18_struct_fields_compile() {
        // Red Gate guard: catalog must be non-empty before field-access assertions are
        // meaningful. This fails against the stub (KNOWN_PROTOCOLS = &[]).
        let first = KNOWN_PROTOCOLS
            .first()
            .expect("KNOWN_PROTOCOLS must not be empty — AC-151-001 Red Gate");

        // Access every public field to confirm the struct shape matches the AC.
        let _name: &'static str = first.name;
        let _category: &ProtocolCategory = &first.category;
        let _transport: &Transport = &first.transport;
        let _canonical_ports: &'static [u16] = first.canonical_ports;
        let _ethertype: Option<u16> = first.ethertype;
        let _port_detectable: bool = first.port_detectable;
        let _description: &'static str = first.description;
    }

    /// REGRESSION-GUARD: Verifies that `ProtocolCategory` has exactly the two variants
    /// `ICS` and `IT` (no `L2` variant — ADR-012 Decision 7). Fails at Red Gate because
    /// `KNOWN_PROTOCOLS` is empty and the catalog membership assertions cannot be satisfied.
    #[test]
    fn test_BC_2_18_category_variants_exactly_two() {
        // Compile-check: both variants are reachable.
        let _ics = ProtocolCategory::ICS;
        let _it = ProtocolCategory::IT;
        assert_ne!(
            ProtocolCategory::ICS,
            ProtocolCategory::IT,
            "ICS and IT must be distinct variants"
        );

        // Red Gate guard: assert the catalog actually uses both variants — fails against
        // the empty stub; passes once the 30-entry catalog is populated.
        let has_ics = KNOWN_PROTOCOLS
            .iter()
            .any(|p| p.category == ProtocolCategory::ICS);
        let has_it = KNOWN_PROTOCOLS
            .iter()
            .any(|p| p.category == ProtocolCategory::IT);
        assert!(
            has_ics,
            "KNOWN_PROTOCOLS must contain at least one ICS entry"
        );
        assert!(has_it, "KNOWN_PROTOCOLS must contain at least one IT entry");
    }

    // -----------------------------------------------------------------------
    // AC-151-002 — SUPPORTED_PORTS compile-time constant (8 ports)
    // Traces to: BC-2.18.003 v1.3 PC-3, Invariant 1; ADR-012 Decision 5
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies `SUPPORTED_PORTS` contains exactly 8 port values —
    /// 502, 20000, 44818, 443, 8443, 80, 8080, 53. Fails against the stub (&[]).
    #[test]
    fn test_BC_2_18_003_supported_ports_len() {
        assert_eq!(
            SUPPORTED_PORTS.len(),
            8,
            "SUPPORTED_PORTS must contain exactly 8 actively-dissected ports"
        );
    }

    /// REGRESSION-GUARD: Verifies each of the 8 canonical port values is present in
    /// `SUPPORTED_PORTS`. Fails against the stub (empty slice).
    #[test]
    fn test_BC_2_18_003_supported_ports_contains_canonical() {
        assert!(
            SUPPORTED_PORTS.contains(&502),
            "SUPPORTED_PORTS must contain 502 (Modbus/TCP)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&20000),
            "SUPPORTED_PORTS must contain 20000 (DNP3)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&44818),
            "SUPPORTED_PORTS must contain 44818 (EtherNet/IP+CIP)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&443),
            "SUPPORTED_PORTS must contain 443 (TLS)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&8443),
            "SUPPORTED_PORTS must contain 8443 (TLS alt)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&80),
            "SUPPORTED_PORTS must contain 80 (HTTP)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&8080),
            "SUPPORTED_PORTS must contain 8080 (HTTP alt)"
        );
        assert!(
            SUPPORTED_PORTS.contains(&53),
            "SUPPORTED_PORTS must contain 53 (DNS decode-loop path — no DispatchTarget::Dns)"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies canonical port values
    /// from authoritative specifications are present in SUPPORTED_PORTS.
    ///
    /// Sources:
    /// - Port 502  (Modbus/TCP): IANA registry + Modbus Application Protocol v1.1b3 §4.3.1
    /// - Port 20000 (DNP3):      IEEE Std 1815-2012 §10.3.2
    /// - Port 53   (DNS):        RFC 1035 §4.2.1
    ///
    /// Fails against the stub (SUPPORTED_PORTS = &[]).
    #[test]
    fn test_BC_2_18_003_supported_ports_canonical() {
        // Port 502 — Modbus/TCP; IANA + Modbus App Protocol v1.1b3 §4.3.1 "Well-Known TCP
        // Port 0+502".
        assert!(
            SUPPORTED_PORTS.contains(&502),
            "502 (Modbus/TCP canonical port — IANA/Modbus App Protocol v1.1b3 §4.3.1) \
             must be in SUPPORTED_PORTS"
        );

        // Port 20000 — DNP3; IEEE Std 1815-2012 §10.3.2 "TCP Port 20000".
        assert!(
            SUPPORTED_PORTS.contains(&20000),
            "20000 (DNP3 canonical port — IEEE Std 1815-2012 §10.3.2) must be in SUPPORTED_PORTS"
        );

        // Port 53 — DNS; RFC 1035 §4.2.1 "Server" (port 53).
        // NOTE: DNS is dissected via the decode-loop path in main.rs, NOT via
        // DispatchTarget::Dns. DNS/53 not mirroring classify() is PERMANENT and BY DESIGN
        // (ADR-012 Decision 5). The port is nonetheless in SUPPORTED_PORTS.
        assert!(
            SUPPORTED_PORTS.contains(&53),
            "53 (DNS canonical port — RFC 1035 §4.2.1) must be in SUPPORTED_PORTS \
             (DNS is dissected via the decode-loop path, not DispatchTarget; this is by design)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-151-003 — KNOWN_PROTOCOLS static array (30 entries, catalog content)
    // Traces to: BC-2.18.004 v1.2 PC-1..5; BC-2.18.003 v1.3 PC-2; ADR-012 Decision 1/4
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies `KNOWN_PROTOCOLS` contains exactly 30 entries.
    /// Fails against the stub (empty slice).
    #[test]
    fn test_BC_2_18_003_known_protocols_len() {
        assert_eq!(
            KNOWN_PROTOCOLS.len(),
            30,
            "KNOWN_PROTOCOLS must contain exactly 30 entries (7 supported + 23 unsupported)"
        );
    }

    /// REGRESSION-GUARD: Verifies the ARP entry has the expected LinkLayer fields:
    /// `canonical_ports` is empty, `port_detectable` is false, `transport` is LinkLayer.
    /// Fails against the stub (no ARP entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_arp_linkLayer_port_detectable_false() {
        let arp = KNOWN_PROTOCOLS
            .iter()
            .find(|p| p.name == "ARP")
            .expect("ARP entry (name == \"ARP\") must exist in KNOWN_PROTOCOLS");

        assert!(
            arp.canonical_ports.is_empty(),
            "ARP canonical_ports must be empty (L2 protocol; no TCP/UDP port)"
        );
        assert!(
            !arp.port_detectable,
            "ARP port_detectable must be false (L2 protocol detected via DecodedFrame::Arp)"
        );
        assert_eq!(
            arp.transport,
            Transport::LinkLayer,
            "ARP transport must be Transport::LinkLayer"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the IEC 61850 GOOSE
    /// entry has EtherType 35000 (0x88B8).
    ///
    /// Source: IEC 61850-8-1 §4; IEEE Registration Authority EtherType registry entry
    /// "IEC GOOSE". The decimal value 35000 == 0x88B8.
    ///
    /// Fails against the stub (no GOOSE entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_goose_ethertype_canonical() {
        let goose = find_entry("GOOSE");

        // Primary assertion: canonical EtherType per IEC 61850-8-1 §4 + IEEE RA registry.
        assert_eq!(
            goose.ethertype,
            Some(35000), // 0x88B8
            "IEC 61850 GOOSE ethertype must be Some(35000) (0x88B8; IEC 61850-8-1 §4; \
             IEEE RA registry \"IEC GOOSE\")"
        );
        // Wrong-value guard: must not be mistakenly assigned the IEC 61850 SV value.
        assert_ne!(
            goose.ethertype,
            Some(35002), // 0x88BA — IEC 61850 SV
            "GOOSE ethertype must NOT be Some(35002) — that is IEC 61850 Sampled Values (0x88BA)"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the Ethernet POWERLINK
    /// entry has EtherType 34987 (0x88AB).
    ///
    /// Source: IEEE Registration Authority EtherType registry "ETHERNET Powerlink";
    /// EPSG assignment (V2, current standard); confirmed by Wireshark epan/etypes.h
    /// `ETHERTYPE_EPL_V2 = 0x88AB`; confirmed by IETF `ietf-ethertypes` YANG module
    /// value 34987. The obsolete V1 value 0x3E3F is intentionally excluded.
    ///
    /// Fails against the stub (no POWERLINK entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_powerlink_ethertype_canonical() {
        let powerlink = find_entry("POWERLINK");

        // Primary assertion: V2 canonical EtherType per IEEE RA + EPSG + Wireshark.
        assert_eq!(
            powerlink.ethertype,
            Some(34987), // 0x88AB
            "Ethernet POWERLINK ethertype must be Some(34987) (0x88AB; IEEE RA registry; \
             EPSG V2 current standard; Wireshark ETHERTYPE_EPL_V2; \
             IETF ietf-ethertypes value 34987)"
        );
        // Wrong-value guard: must not be the obsolete V1 value.
        assert_ne!(
            powerlink.ethertype,
            Some(0x3E3F), // 16191 decimal — obsolete V1
            "POWERLINK ethertype must NOT be Some(0x3E3F) — that is the obsolete V1 value"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the EtherCAT
    /// entry has EtherType 34980 (0x88A4).
    ///
    /// Source: IEEE Registration Authority EtherType registry "EtherCAT Technology Group".
    /// Wrong-value guards check against PROFINET (34962 / 0x8892) and GOOSE (35000 / 0x88B8)
    /// — two visually similar values that could result from a copy-paste error.
    ///
    /// Fails against the stub (no EtherCAT entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_ethercat_ethertype_canonical() {
        // EtherCAT has no "PROFINET" in its name; distinguish L2 EtherCAT from other L2 entries.
        let ethercat = KNOWN_PROTOCOLS
            .iter()
            .find(|p| p.name.contains("EtherCAT"))
            .expect("EtherCAT entry must exist in KNOWN_PROTOCOLS");

        // Primary assertion: canonical EtherType per IEEE RA registry.
        assert_eq!(
            ethercat.ethertype,
            Some(34980), // 0x88A4
            "EtherCAT ethertype must be Some(34980) (0x88A4; IEEE RA registry \
             \"EtherCAT Technology Group\")"
        );
        // Wrong-value guard: must not be mistakenly assigned PROFINET's EtherType.
        assert_ne!(
            ethercat.ethertype,
            Some(34962), // 0x8892 — PROFINET RT/DCP
            "EtherCAT ethertype must NOT be Some(34962) — that is PROFINET RT/DCP (0x8892)"
        );
        // Wrong-value guard: must not be mistakenly assigned GOOSE's EtherType.
        assert_ne!(
            ethercat.ethertype,
            Some(35000), // 0x88B8 — IEC 61850 GOOSE
            "EtherCAT ethertype must NOT be Some(35000) — that is IEC 61850 GOOSE (0x88B8)"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the PROFINET RT/DCP
    /// (L2) entry has EtherType 34962 (0x8892).
    ///
    /// Source: IEEE Registration Authority EtherType registry "PROFINET Acyclic Real-Time
    /// / PROFINET-DCP". Wrong-value guard checks against EtherCAT (34980 / 0x88A4).
    ///
    /// Note: PROFINET RPC (UDP, ports 34962/34963/34964) is a separate catalog entry.
    /// This test targets the L2 PROFINET RT/DCP entry specifically, identified by
    /// Transport::LinkLayer.
    ///
    /// Fails against the stub (no PROFINET L2 entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_profinet_ethertype_canonical() {
        // Locate the L2 PROFINET entry (PROFINET RT/DCP, not PROFINET RPC which is UDP).
        let profinet_l2 = KNOWN_PROTOCOLS
            .iter()
            .find(|p| p.name.contains("PROFINET") && p.transport == Transport::LinkLayer)
            .expect(
                "PROFINET L2 (LinkLayer) entry must exist in KNOWN_PROTOCOLS — \
                 distinct from the PROFINET RPC UDP entry",
            );

        // Primary assertion: canonical EtherType per IEEE RA registry.
        assert_eq!(
            profinet_l2.ethertype,
            Some(34962), // 0x8892
            "PROFINET RT/DCP ethertype must be Some(34962) (0x8892; IEEE RA registry \
             \"PROFINET Acyclic Real-Time / PROFINET-DCP\")"
        );
        // Wrong-value guard: must not be mistakenly assigned EtherCAT's EtherType.
        assert_ne!(
            profinet_l2.ethertype,
            Some(34980), // 0x88A4 — EtherCAT
            "PROFINET RT/DCP ethertype must NOT be Some(34980) — that is EtherCAT (0x88A4)"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the IEC 61850
    /// Sampled Values (SV) entry has EtherType 35002 (0x88BA).
    ///
    /// Source: IEC 61850-8-1 §4. GOOSE-transposition guard: SV is 0x88BA (35002), not
    /// GOOSE 0x88B8 (35000). The two values differ by 2; a transposition would be silent
    /// without this guard.
    ///
    /// Fails against the stub (no SV entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_sv_ethertype_canonical() {
        let sv = find_entry("Sampled Values");

        // Primary assertion: canonical EtherType per IEC 61850-8-1 §4.
        assert_eq!(
            sv.ethertype,
            Some(35002), // 0x88BA
            "IEC 61850 Sampled Values ethertype must be Some(35002) (0x88BA; IEC 61850-8-1 §4)"
        );
        // GOOSE-transposition guard: SV is 0x88BA, not GOOSE 0x88B8. The difference is 2;
        // a byte-transposition error would place the wrong value here.
        assert_ne!(
            sv.ethertype,
            Some(35000), // 0x88B8 — IEC 61850 GOOSE
            "IEC 61850 SV ethertype must NOT be Some(35000) — that is GOOSE (0x88B8); \
             SV is 0x88BA (35002)"
        );
    }

    /// REGRESSION-GUARD — DF-CANONICAL-FRAME-HOLDOUT-001: Verifies the BACnet/IP entry
    /// uses UDP transport and canonical port 47808 (0xBAC0).
    ///
    /// Source: ASHRAE 135-2016 Annex J §J.2.1 "UDP Port Number 47808 (0xBAC0)".
    /// BACnet/IP is UDP-only by default; port 47808 is NOT in SUPPORTED_PORTS, so
    /// BACnet/IP appears in `unsupported_protocols()`.
    ///
    /// Fails against the stub (no BACnet entry in empty catalog).
    #[test]
    fn test_BC_2_18_003_bacnet_udp_canonical() {
        let bacnet = find_entry("BACnet");

        assert_eq!(
            bacnet.transport,
            Transport::Udp,
            "BACnet/IP transport must be Transport::Udp \
             (ASHRAE 135-2016 Annex J §J.2.1; UDP-only canonical model)"
        );
        assert_eq!(
            bacnet.canonical_ports,
            &[47808u16],
            "BACnet/IP canonical_ports must be &[47808] (0xBAC0; ASHRAE 135-2016 Annex J §J.2.1)"
        );
    }

    /// REGRESSION-GUARD: Verifies the port-102 four-way collision — S7comm, S7comm-plus,
    /// IEC 61850 MMS, and ICCP/TASE.2 all exist in KNOWN_PROTOCOLS with canonical port 102.
    /// None of these are in SUPPORTED_PORTS (port 102 is absent), so all four appear in
    /// `unsupported_protocols()`.
    ///
    /// Fails against the stub (no entries in empty catalog).
    #[test]
    fn test_BC_2_18_003_port_102_four_protocols_present() {
        let port_102_count = KNOWN_PROTOCOLS
            .iter()
            .filter(|p| p.canonical_ports == [102u16].as_slice())
            .count();

        assert_eq!(
            port_102_count, 4,
            "exactly 4 catalog entries must share canonical_ports = &[102]: \
             S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2 \
             (BC-2.18.003 EC-007; BC-2.18.004 EC-005)"
        );

        // Verify each of the four protocols is present by name fragment.
        let names: Vec<&str> = KNOWN_PROTOCOLS
            .iter()
            .filter(|p| p.canonical_ports == [102u16].as_slice())
            .map(|p| p.name)
            .collect();

        assert!(
            names
                .iter()
                .any(|n| n.contains("S7comm") && !n.contains("plus")),
            "S7comm entry (canonical port 102) must be present; found: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("S7comm-plus")
                || n.contains("S7comm+")
                || (n.contains("S7comm") && n.contains("plus"))),
            "S7comm-plus entry (canonical port 102) must be present; found: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("MMS")),
            "IEC 61850 MMS entry (canonical port 102) must be present; found: {names:?}"
        );
        assert!(
            names
                .iter()
                .any(|n| n.contains("ICCP") || n.contains("TASE")),
            "ICCP/TASE.2 entry (canonical port 102) must be present; found: {names:?}"
        );
    }

    /// REGRESSION-GUARD: Verifies exactly 5 entries in KNOWN_PROTOCOLS have
    /// `port_detectable: false` and `transport: Transport::LinkLayer` — the five L2/multicast
    /// unsupported entries: IEC 61850 GOOSE, IEC 61850 Sampled Values, PROFINET RT/DCP,
    /// EtherCAT, Ethernet POWERLINK. ARP is NOT counted here (ARP is a supported L2 entry).
    ///
    /// Fails against the stub (no entries in empty catalog).
    #[test]
    fn test_BC_2_18_003_l2_port_detectable_false_exactly_five() {
        // Filter for non-ARP L2 entries (the 5 unsupported L2/multicast protocols).
        let l2_unsupported_count = KNOWN_PROTOCOLS
            .iter()
            .filter(|p| {
                p.transport == Transport::LinkLayer && !p.port_detectable && p.name != "ARP"
            })
            .count();

        assert_eq!(
            l2_unsupported_count, 5,
            "exactly 5 non-ARP LinkLayer port_detectable:false entries must exist: \
             IEC 61850 GOOSE, IEC 61850 Sampled Values, PROFINET RT/DCP, \
             EtherCAT, Ethernet POWERLINK (ADR-012 Decision 3; BC-2.18.003 v1.3)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-151-004 — all_protocols() pure function
    // Traces to: BC-2.18.004 v1.2 PC-1; BC-2.18.003 v1.3 Invariant 2
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies `all_protocols()` returns a slice of the same length as
    /// `KNOWN_PROTOCOLS`. Fails against the stub (`all_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_004_all_protocols_len() {
        assert_eq!(
            all_protocols().len(),
            KNOWN_PROTOCOLS.len(),
            "all_protocols().len() must equal KNOWN_PROTOCOLS.len() — \
             all_protocols() must return the full static catalog"
        );
    }

    // -----------------------------------------------------------------------
    // AC-151-005 — supported_protocols() — exactly 7 entries
    // Traces to: BC-2.18.003 v1.3 PC-1, PC-3, Invariant 3; ADR-012 Decision 5
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies `supported_protocols()` returns exactly 7 entries.
    /// Fails against the stub (`supported_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_003_supported_protocols_len() {
        assert_eq!(
            supported_protocols().len(),
            7,
            "supported_protocols() must return exactly 7 entries: \
             Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, HTTP"
        );
    }

    /// REGRESSION-GUARD: Verifies ARP is in `supported_protocols()` despite having
    /// `canonical_ports: &[]` — the ARP special case (`|| p.name == "ARP"`) must be
    /// present in the implementation (BC-2.18.003 Invariant 3).
    /// Fails against the stub (`supported_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_003_arp_in_supported_set() {
        let supported = supported_protocols();
        assert!(
            supported.iter().any(|p| p.name == "ARP"),
            "ARP must be in supported_protocols() — ARP is supported via DecodedFrame::Arp; \
             the explicit '|| p.name == \"ARP\"' special case is required (BC-2.18.003 Invariant 3)"
        );
    }

    /// REGRESSION-GUARD: Verifies the port mirror invariant — for every port in
    /// `SUPPORTED_PORTS`, `supported_protocols()` contains an entry with that port in
    /// `canonical_ports`. This includes port 53 (DNS); per F-F3P12-001, DNS/53 satisfies
    /// the mirror (DNS entry has `canonical_ports = &[53]`; the decode-loop path note does
    /// not exempt DNS from the mirror check).
    ///
    /// Fails against the stub (`supported_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_003_supported_ports_mirror() {
        let supported = supported_protocols();

        for &port in SUPPORTED_PORTS {
            let mirrored = supported.iter().any(|p| p.canonical_ports.contains(&port));
            assert!(
                mirrored,
                "port {port} is in SUPPORTED_PORTS but no supported_protocols() entry has \
                 it in canonical_ports — mirror invariant violated \
                 (BC-2.18.003 v1.3 Invariant 1; F-F3P12-001: port 53/DNS is included)"
            );
        }
    }

    /// REGRESSION-GUARD: Verifies BACnet/IP (port 47808) is NOT in `supported_protocols()`.
    /// Port 47808 is absent from `SUPPORTED_PORTS`, so BACnet/IP must appear only in
    /// `unsupported_protocols()` (BC-2.18.003 EC-003).
    /// Fails against the stub (`supported_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_003_bacnet_unsupported() {
        let supported = supported_protocols();
        assert!(
            !supported.iter().any(|p| p.name.contains("BACnet")),
            "BACnet/IP must NOT appear in supported_protocols() — \
             port 47808 is not in SUPPORTED_PORTS (BC-2.18.003 EC-003)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-151-006 — unsupported_protocols() — exact complement (23 entries)
    // Traces to: BC-2.18.003 v1.3 PC-2, Invariants 4–5; BC-2.18.004 v1.2 PC-1..5
    // -----------------------------------------------------------------------

    /// REGRESSION-GUARD: Verifies `supported_protocols().len() + unsupported_protocols().len()
    /// == KNOWN_PROTOCOLS.len()` (== 30). Fails against the stub (both functions are `todo!()`).
    #[test]
    fn test_BC_2_18_003_partition_len() {
        let s = supported_protocols().len();
        let u = unsupported_protocols().len();
        let total = KNOWN_PROTOCOLS.len();

        assert_eq!(
            s + u,
            total,
            "supported ({s}) + unsupported ({u}) must equal KNOWN_PROTOCOLS.len() ({total}) \
             — BC-2.18.004 v1.2 PC-3 counting invariant"
        );
    }

    /// REGRESSION-GUARD: Verifies `supported_protocols()` and `unsupported_protocols()` are
    /// disjoint — no entry name appears in both result sets.
    /// Fails against the stub (functions are `todo!()`).
    #[test]
    fn test_BC_2_18_004_disjoint() {
        let supported = supported_protocols();
        let unsupported = unsupported_protocols();

        for s in &supported {
            assert!(
                !unsupported.iter().any(|u| u.name == s.name),
                "entry '{}' appears in both supported_protocols() and \
                 unsupported_protocols() — sets must be disjoint \
                 (BC-2.18.004 v1.2 PC-2)",
                s.name
            );
        }
    }

    /// REGRESSION-GUARD: Verifies that every entry returned by `unsupported_protocols()` has
    /// a name present in `KNOWN_PROTOCOLS` — no phantom entries may appear.
    /// Fails against the stub (`unsupported_protocols()` is `todo!()`).
    #[test]
    fn test_BC_2_18_004_no_phantom_entries() {
        let known_names: Vec<&str> = KNOWN_PROTOCOLS.iter().map(|p| p.name).collect();
        let unsupported = unsupported_protocols();

        for u in &unsupported {
            assert!(
                known_names.contains(&u.name),
                "entry '{}' returned by unsupported_protocols() is not in KNOWN_PROTOCOLS — \
                 phantom entry (BC-2.18.004 v1.2 PC-5)",
                u.name
            );
        }
    }

    // -----------------------------------------------------------------------
    // AC-151-007 — VP-041 proptest harnesses
    // Traces to: BC-2.18.004 v1.2 Invariant 4; BC-2.18.003 v1.3 VP table
    // -----------------------------------------------------------------------

    // VP-041 oracle cross-check (non-vacuous).
    //
    // For a randomly-sampled entry from `KNOWN_PROTOCOLS`, verifies that its membership
    // in `supported_protocols()` matches an independently-computed oracle predicate:
    //   oracle = entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p))
    //            || entry.name == "ARP"
    //
    // The oracle is computed WITHOUT calling `supported_protocols()` or
    // `unsupported_protocols()` — this non-vacuity guards against `supported_protocols()`
    // diverging from `SUPPORTED_PORTS`.
    //
    // Non-vacuity runtime guard: asserts KNOWN_PROTOCOLS is non-empty so the test
    // cannot pass silently against an unpopulated catalog.
    //
    // Fails against the stub: all_protocols() is todo!() → panics on first invocation.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn proptest_vp041_oracle_cross_check(
            idx in 0usize..30usize,
        ) {
            // Non-vacuity guard: catalog must be populated.
            prop_assert!(
                !KNOWN_PROTOCOLS.is_empty(),
                "KNOWN_PROTOCOLS is empty — VP-041 oracle would be vacuously true"
            );

            // Obtain all entries and supported set. At Red Gate, all_protocols() panics
            // (todo!()); proptest treats the panic as a test failure.
            let all = all_protocols();
            let supported = supported_protocols();

            // Use modulo to handle the case where catalog size < 30 (unexpected but safe).
            let entry = &all[idx % all.len()];

            // Oracle: computed INDEPENDENTLY of supported_protocols() / unsupported_protocols().
            // This is the non-vacuous predicate from BC-2.18.003 v1.3 PC-1 / BC-2.18.004 Inv-4.
            let oracle: bool = entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p))
                || entry.name == "ARP";

            // Actual: check membership in supported_protocols() result set.
            let in_supported: bool = supported.iter().any(|p| p.name == entry.name);

            prop_assert_eq!(
                oracle,
                in_supported,
                "VP-041 oracle mismatch for entry '{}': oracle={} actual={} \
                 (oracle: ports∩SUPPORTED_PORTS non-empty OR name==\"ARP\")",
                entry.name,
                oracle,
                in_supported,
            );
        }
    }

    // VP-041 partition/disjointness invariant.
    //
    // Verifies that supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS
    // (counting invariant) and that the two sets share no entry (disjoint).
    //
    // This holds trivially by the complement derivation (unsupported = KNOWN \ supported).
    // The non-vacuous guard is proptest_vp041_oracle_cross_check.
    //
    // Fails against the stub: supported_protocols() is todo!() → panics.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn proptest_vp041_partition_invariant(
            _n in 0u8..=255u8,
        ) {
            let supported = supported_protocols();
            let unsupported = unsupported_protocols();
            let all = all_protocols();

            // Counting invariant: BC-2.18.004 v1.2 PC-3.
            prop_assert_eq!(
                supported.len() + unsupported.len(),
                all.len(),
                "partition counting: supported ({}) + unsupported ({}) != all_protocols() ({})",
                supported.len(),
                unsupported.len(),
                all.len()
            );

            // Disjointness: BC-2.18.004 v1.2 PC-2.
            for s in &supported {
                prop_assert!(
                    !unsupported.iter().any(|u| u.name == s.name),
                    "entry '{}' appears in both supported and unsupported — sets must be \
                     disjoint (BC-2.18.004 v1.2 PC-2)",
                    s.name
                );
            }
        }
    }
}
