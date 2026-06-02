//! Per-TCP-flow state and the canonical [`FlowKey`].
//!
//! A flow is identified by the unordered 4-tuple {lower_ip:port, upper_ip:port},
//! so packets in either direction map to the same key. Each [`TcpFlow`]
//! owns two [`FlowDirection`] segment buffers (client→server and
//! server→client), tracks ISN (initial sequence number) per direction, and
//! advances through the [`FlowState`] machine (`New` → `SynSent` → `Established`
//! → `Closing` → `Closed`).
//!
//! Memory accounting (`memory_used`) is consulted by the reassembler's
//! memcap eviction strategy; per-direction `overlap_count`,
//! `small_segment_run`, and `out_of_window_count` feed the
//! threshold-based Anomaly findings.

use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::reassembly::handler::Direction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowKey {
    lower_ip: IpAddr,
    lower_port: u16,
    upper_ip: IpAddr,
    upper_port: u16,
}

impl FlowKey {
    pub fn lower_ip(&self) -> IpAddr {
        self.lower_ip
    }

    pub fn lower_port(&self) -> u16 {
        self.lower_port
    }

    pub fn upper_ip(&self) -> IpAddr {
        self.upper_ip
    }

    pub fn upper_port(&self) -> u16 {
        self.upper_port
    }

    pub fn new(ip_a: IpAddr, port_a: u16, ip_b: IpAddr, port_b: u16) -> Self {
        // Canonicalize by (ip, port) tuple comparison — keeps IP+port paired together.
        // This is critical: sorting independently would merge different connections.
        if (ip_a, port_a) <= (ip_b, port_b) {
            FlowKey {
                lower_ip: ip_a,
                lower_port: port_a,
                upper_ip: ip_b,
                upper_port: port_b,
            }
        } else {
            FlowKey {
                lower_ip: ip_b,
                lower_port: port_b,
                upper_ip: ip_a,
                upper_port: port_a,
            }
        }
    }
}

impl std::fmt::Display for FlowKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} → {}:{}",
            self.lower_ip, self.lower_port, self.upper_ip, self.upper_port
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    New,
    SynSent,
    Established,
    Closing,
    Closed,
}

#[derive(Debug)]
pub struct FlowDirection {
    pub isn: Option<u32>,
    pub base_offset: u64,
    pub(super) segments: BTreeMap<u64, Vec<u8>>,
    pub(super) buffered_bytes: usize,
    pub reassembled_bytes: usize,
    pub overlap_count: u32,
    pub overlap_alert_fired: bool,
    /// Length of the *current consecutive run* of small (undersized)
    /// segments. Incremented per small segment and reset to zero by any
    /// normal-sized one — see `ReassemblyConfig::small_segment_alert_threshold`.
    /// Maintained for every flow regardless of the small-segment port
    /// exemption: the exemption is applied when the finding would be
    /// emitted, not when segments are counted, so this stays a raw
    /// measurement.
    pub small_segment_run: u32,
    pub small_segment_alert_fired: bool,
    pub out_of_window_count: u32,
    pub out_of_window_alert_fired: bool,
    pub fin_seen: bool,
    pub rst_seen: bool,
    pub depth_exceeded: bool,
}

impl Default for FlowDirection {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowDirection {
    pub fn new() -> Self {
        FlowDirection {
            isn: None,
            base_offset: 0,
            segments: BTreeMap::new(),
            buffered_bytes: 0,
            reassembled_bytes: 0,
            overlap_count: 0,
            overlap_alert_fired: false,
            small_segment_run: 0,
            small_segment_alert_fired: false,
            out_of_window_count: 0,
            out_of_window_alert_fired: false,
            fin_seen: false,
            rst_seen: false,
            depth_exceeded: false,
        }
    }

    pub fn set_isn(&mut self, isn: u32) {
        if self.isn.is_none() {
            self.isn = Some(isn);
            self.base_offset = 1; // ISN+1 is first data byte
        }
    }

    pub fn infer_isn(&mut self, first_seq: u32) {
        if self.isn.is_none() {
            self.isn = Some(first_seq.wrapping_sub(1));
            self.base_offset = 1;
        }
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn buffered_bytes(&self) -> usize {
        self.buffered_bytes
    }

    pub fn segments_is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segment_at(&self, offset: u64) -> Option<&[u8]> {
        self.segments.get(&offset).map(|v| v.as_slice())
    }

    pub fn has_segment_at(&self, offset: u64) -> bool {
        self.segments.contains_key(&offset)
    }

    pub fn memory_used(&self) -> usize {
        debug_assert_eq!(
            self.buffered_bytes,
            self.segments.values().map(|v| v.len()).sum::<usize>(),
            "buffered_bytes counter drifted from actual segment sizes"
        );
        self.buffered_bytes
    }
}

#[derive(Debug)]
pub struct TcpFlow {
    pub key: FlowKey,
    pub client_to_server: FlowDirection,
    pub server_to_client: FlowDirection,
    pub state: FlowState,
    pub partial: bool,
    pub first_seen: u32,
    pub last_seen: u32,
    initiator: Option<(IpAddr, u16)>,
    fin_count: u8,
}

impl TcpFlow {
    pub fn new(key: FlowKey, timestamp: u32) -> Self {
        TcpFlow {
            key,
            client_to_server: FlowDirection::new(),
            server_to_client: FlowDirection::new(),
            state: FlowState::New,
            partial: false,
            first_seen: timestamp,
            last_seen: timestamp,
            initiator: None,
            fin_count: 0,
        }
    }

    pub fn set_initiator(&mut self, ip: IpAddr, port: u16) {
        if self.initiator.is_none() {
            self.initiator = Some((ip, port));
        }
    }

    pub fn direction(&self, src_ip: IpAddr, src_port: u16) -> Direction {
        if self.initiator == Some((src_ip, src_port)) {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        }
    }

    /// Returns the count of FIN flags observed on this flow.
    ///
    /// Saturates at u8::MAX (255) — see BC-2.04.050 invariant 4.
    pub fn fin_count(&self) -> u8 {
        self.fin_count
    }

    pub fn get_direction_mut(&mut self, dir: Direction) -> &mut FlowDirection {
        match dir {
            Direction::ClientToServer => &mut self.client_to_server,
            Direction::ServerToClient => &mut self.server_to_client,
        }
    }

    pub fn on_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::SynSent;
        }
    }

    pub fn on_syn_ack(&mut self) {
        if self.state == FlowState::SynSent || self.state == FlowState::New {
            self.state = FlowState::Established;
        }
    }

    pub fn on_data_without_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::Established;
            self.partial = true;
        }
    }

    pub fn on_fin(&mut self) {
        self.fin_count = self.fin_count.saturating_add(1);
        if self.fin_count >= 2 {
            self.state = FlowState::Closed;
        } else if self.state == FlowState::Established || self.state == FlowState::SynSent {
            self.state = FlowState::Closing;
        }
    }

    pub fn on_rst(&mut self) {
        self.state = FlowState::Closed;
    }

    pub fn memory_used(&self) -> usize {
        self.client_to_server.memory_used() + self.server_to_client.memory_used()
    }
}

// Kani formal-verification harnesses. Gated behind `#[cfg(kani)]` so they are
// invisible to the normal `cargo build`/`cargo test`/`cargo clippy` pipeline
// (the `kani` cfg is only set by `cargo kani`). These prove VP-001 (FlowKey
// canonical ordering) and VP-009 (FlowState machine validity).
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    // ---- VP-001: FlowKey Canonical Ordering (BC-2.04.003, INV-1) -------------

    /// Bounded domain: all 32-bit IPv4 addresses and 16-bit ports are symbolic.
    /// This is the *full IPv4* endpoint space (no `assume` narrowing), so the
    /// proof covers every valid (IPv4, port) pair. Kani models the two `u32` +
    /// two `u16` symbolic words exactly; the harness is loop-free and
    /// allocation-free, so this is sound and complete over IPv4.
    ///
    /// SCOPE: production `FlowKey` accepts `IpAddr` (IPv4 *and* IPv6); this
    /// harness symbolically proves the IPv4 family only. IPv6 is independently
    /// and fully proven by `verify_flowkey_canonical_ordering_ipv6` below over
    /// the full symbolic 128-bit address space (tractable under CBMC, ~3s).
    #[kani::proof]
    fn verify_flowkey_canonical_ordering_ipv4() {
        let raw_a: u32 = kani::any();
        let port_a: u16 = kani::any();
        let raw_b: u32 = kani::any();
        let port_b: u16 = kani::any();

        let ip_a = IpAddr::V4(Ipv4Addr::from(raw_a));
        let ip_b = IpAddr::V4(Ipv4Addr::from(raw_b));

        let key_ab = FlowKey::new(ip_a, port_a, ip_b, port_b);
        let key_ba = FlowKey::new(ip_b, port_b, ip_a, port_a);

        // Commutativity: argument order must not change the canonical key.
        assert!(key_ab == key_ba);

        // Ordering invariant: the stored "lower" tuple is <= the "upper" tuple
        // under tuple-pair comparison.
        assert!(
            (key_ab.lower_ip(), key_ab.lower_port()) <= (key_ab.upper_ip(), key_ab.upper_port())
        );
    }

    /// VP-001 over IPv6. `FlowKey::new` canonicalizes by a single total-order
    /// `(ip, port) <= (other_ip, other_port)` tuple comparison: it does NOT
    /// branch on address family, and `IpAddr`/`Ipv6Addr` implement a *total*
    /// `Ord`, so the same commutativity + ordering argument holds for IPv6 as
    /// for IPv4. This harness discharges that argument formally over the full
    /// symbolic 128-bit address space (two `u128` + two `u16` symbolic words).
    /// Loop-free and allocation-free, so it is sound and complete over IPv6.
    #[kani::proof]
    fn verify_flowkey_canonical_ordering_ipv6() {
        let raw_a: u128 = kani::any();
        let port_a: u16 = kani::any();
        let raw_b: u128 = kani::any();
        let port_b: u16 = kani::any();

        let ip_a = IpAddr::V6(Ipv6Addr::from(raw_a));
        let ip_b = IpAddr::V6(Ipv6Addr::from(raw_b));

        let key_ab = FlowKey::new(ip_a, port_a, ip_b, port_b);
        let key_ba = FlowKey::new(ip_b, port_b, ip_a, port_a);

        // Commutativity: argument order must not change the canonical key.
        assert!(key_ab == key_ba);

        // Ordering invariant: the stored "lower" tuple is <= the "upper" tuple
        // under tuple-pair comparison (total order over IPv6 addresses).
        assert!(
            (key_ab.lower_ip(), key_ab.lower_port()) <= (key_ab.upper_ip(), key_ab.upper_port())
        );
    }

    /// Proves canonicalization is TUPLE-PAIR, not independent per-field sorting.
    /// Same IP, descending ports: the (ip, 80) tuple must win as the lower slot
    /// even though it is passed second.
    #[kani::proof]
    fn verify_flowkey_tuple_pair_not_independent_field() {
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 0, 0, 0));
        let key = FlowKey::new(ip, 9000, ip, 80);
        assert!(key.lower_port() == 80);
        assert!(key.upper_port() == 9000);
    }

    // ---- VP-009: FlowState Machine Validity (BC-2.04.050/051/052) ------------

    /// Build a `TcpFlow` pre-seeded to one of the 5 reachable states by driving
    /// the real transition methods. `discriminant % 5` selects the state.
    fn make_flow_in_state(discriminant: u8) -> TcpFlow {
        let key = FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
            1000,
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 2)),
            80,
        );
        let mut flow = TcpFlow::new(key, 0);
        match discriminant % 5 {
            0 => {}             // New
            1 => flow.on_syn(), // SynSent
            2 => {
                flow.on_syn();
                flow.on_syn_ack();
            } // Established
            3 => {
                flow.on_syn();
                flow.on_syn_ack();
                flow.on_fin();
            } // Closing
            _ => flow.on_rst(), // Closed
        }
        flow
    }

    /// Apply one of the 5 driving events. Test-only; not in production code.
    fn apply_event(flow: &mut TcpFlow, event: u8) {
        match event % 5 {
            0 => flow.on_syn(),
            1 => flow.on_syn_ack(),
            2 => flow.on_rst(),
            3 => flow.on_fin(),
            _ => flow.on_data_without_syn(),
        }
    }

    /// VP-009 invariant 3: RST drives ANY prior state to `Closed`.
    /// Bounded domain: symbolic `discriminant` (5 reachable start states). Sound
    /// because `make_flow_in_state` exhausts every reachable state.
    #[kani::proof]
    fn verify_rst_closes_from_any_state() {
        let discriminant: u8 = kani::any();
        let mut flow = make_flow_in_state(discriminant);
        flow.on_rst();
        assert!(matches!(flow.state, FlowState::Closed));
    }

    /// VP-009 invariant 5: `Closed` is terminal. From a Closed flow, RST/SYN/FIN
    /// must all leave the state Closed. Note: `on_fin` saturates fin_count and,
    /// at >=2 FINs, forces Closed regardless — so a Closed flow stays Closed.
    #[kani::proof]
    fn verify_closed_is_terminal() {
        let mut f1 = make_flow_in_state(4);
        f1.on_rst();
        assert!(matches!(f1.state, FlowState::Closed));

        let mut f2 = make_flow_in_state(4);
        f2.on_syn();
        assert!(matches!(f2.state, FlowState::Closed));

        let mut f3 = make_flow_in_state(4);
        f3.on_fin();
        assert!(matches!(f3.state, FlowState::Closed));

        let mut f4 = make_flow_in_state(4);
        f4.on_syn_ack();
        assert!(matches!(f4.state, FlowState::Closed));

        let mut f5 = make_flow_in_state(4);
        f5.on_data_without_syn();
        assert!(matches!(f5.state, FlowState::Closed));
    }

    /// VP-009 invariant 4: `on_data_without_syn` moves New -> Established and
    /// sets `partial = true`. Concrete (no symbolic inputs needed).
    #[kani::proof]
    fn verify_data_without_syn_sets_partial() {
        let key = FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
            1000,
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 2)),
            80,
        );
        let mut flow = TcpFlow::new(key, 0);
        assert!(matches!(flow.state, FlowState::New));
        flow.on_data_without_syn();
        assert!(matches!(flow.state, FlowState::Established));
        assert!(flow.partial);
    }

    /// VP-009 invariant 2: no state outside the 5 valid variants is reachable.
    /// Bounded domain: symbolic start state x 2 symbolic events. Two events
    /// suffice to expose any single-step transition out of every reachable
    /// state (the first event can land in any reachable state, the second
    /// exercises every outgoing edge from it). The `FlowState` enum has exactly
    /// 5 variants, so `matches!` against all 5 is total — the assertion would
    /// only fail if memory corruption produced an invalid discriminant.
    #[kani::proof]
    fn verify_no_invalid_state_reachable() {
        let disc: u8 = kani::any();
        let event1: u8 = kani::any();
        let event2: u8 = kani::any();

        let mut flow = make_flow_in_state(disc);
        apply_event(&mut flow, event1);
        apply_event(&mut flow, event2);

        assert!(matches!(
            flow.state,
            FlowState::New
                | FlowState::SynSent
                | FlowState::Established
                | FlowState::Closing
                | FlowState::Closed
        ));
    }
}
