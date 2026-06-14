---
document_type: adr
adr_id: ADR-008
status: proposed
date: 2026-06-12
modified:
  - "v1.1 (2026-06-12): F-A03 — added last_seen_ts: u32 to BindingEntry canonical struct in Decision 4 (was present in prose/Rationale/arch-delta §3.1 but omitted from the struct literal). F-B-002 — Decision 7 summarize() key set expanded to 11 canonical keys (added malformed_frames, other_opcode_count); reconciliation invariant and malformed_frames exclusion rule documented explicitly. BC-2.16.007 was never part of Decision 7 scope — no change needed there."
  - "v1.2 (2026-06-12): F-A03 adversarial Pass 3 remediation — (CRIT-01) added seven missing u64 counter fields to Decision 4 ArpAnalyzer struct literal (other_opcode_count, spoof_findings, garp_findings, storm_findings, mismatch_findings, malformed_findings, malformed_frames); removed dangling keyed-inside-StormCounter comment; (MED-01) expanded Decision 5 detection table with Confidence column and D2 LOW-base/MEDIUM-on-conflict value (arch-delta §3.3 source of truth now mirrored); (MED-03) corrected Decision intro 'six coordinated decisions' → 'seven coordinated decisions'; (LOW-01) corrected Decision 6 'module comment (lines 145-148)' → 'inline merge-by-name comment in technique_info (lines 145-148)'."
  - "v1.3 (2026-06-12): F-B4-L01 adversarial Pass 4 remediation — Decision 4 storm_rate struct comment changed from 'frames-per-second sustained threshold' to 'average-frames-per-second-since-window-start threshold (per BC-2.16.008 Invariant 2)' to eliminate contradiction with BC-2.16.008 Invariant 2 which specifies average-since-window-start, not a sustained-rate detector."
  - "v1.4 (2026-06-12): F-B6-H02 — Decision 7 key 11: replaced unconditional 'two counts are equal' claim with conditional phrasing: malformed_findings == malformed_frames only when --arp is active; when --arp is absent malformed_frames increments but malformed_findings does not, so malformed_findings <= malformed_frames. Aligns with BC-2.16.009 PC4 normative divergence. F-B6-M01 — Decision 4: added normative note on last_seen_ts write responsibility — insert_binding_lru signature unchanged (no ts parameter); process_arp (holding timestamp_secs) writes last_seen_ts on every observation; insert_binding_lru reads it only during eviction scan. PO hand-off recorded. OBS-2 — ARP_STORM_RATE_DEFAULT const doc-comment and Decision 5 D3 trigger cell aligned to 'average-frames-per-second-since-window-start (per BC-2.16.008 Invariant 2)' phrasing."
  - "v1.5 (2026-06-12): F-SA7-HIGH-01 — Decision 4 ArpAnalyzer struct: rewrote malformed_frames field doc-comment from unconditional equality ('Equal to malformed_findings (one finding per rejected frame)') to conditional form: under --arp active malformed_findings increments with each malformed_frames increment; under --arp absent malformed_frames increments but malformed_findings does not, so malformed_findings <= malformed_frames. Aligns this doc-comment sibling with Decision 7 key 11 (fixed in v1.4), BC-2.16.009 PC4, and BC-2.16.010 Invariant 5."
  - "v1.6 (2026-06-12): Pass 8 remediation — (HIGH-01) Decision 3: specified ARP early-extraction in BOTH the strict Ok(slice) arm AND the lax Err(SliceError::Len(_)) arm. The lax arm previously had LaxNetSlice::Arp(_) => unreachable!(...) which is reachable for snaplen-truncated ARP frames (etherparse 0.20 yields Some(LaxNetSlice::Arp(_)) on truncation), violating VP-008/VP-024 Sub-A no-panic. Fixed: lax arm routes Some(LaxNetSlice::Arp(arp)) to DecodedFrame::Arp via extract_arp_frame (truncated body still yields outer MAC + opcode; extract_arp_frame returns None on bad size → mapped to Err). Removed unreachable! from lax_ip_triple ARP arm; replaced with explicit handling. (HIGH-02) Confirmed-API list: Ethernet2Slice::source() confirmed to return [u8; 6] by value (docs.rs etherparse 0.20.1 fetch 2026-06-12); added to confirmed-API list; Some(eth.source()) in Decision 3 routing snippet is correct as written, no dereference needed."
  - "v1.7 (2026-06-12): F-B9-M02 — Decision 7 key 11: struck 'or etherparse parse failure' from malformed_frames definition. malformed_frames counts only frames that reach extract_arp_frame and return None (bad hw/proto sizes / non-Eth-IPv4). Etherparse parse failures (truncated/garbage ARP that never produce an ArpPacketSlice) are handled by the existing decode error path and are NOT counted in malformed_frames (see BC-2.16.009 EC-007). Aligns ADR to BC-2.16.009 EC-007 and BC-2.16.010 key 11."
  - "v1.8 (2026-06-12): F-B10-L01 — Decision 5 detection table D11 trigger cell: removed 'or etherparse rejects the frame'. D11 covers only frames that produced an ArpPacketSlice and then failed the hw_addr_size/proto_addr_size (or non-Ethernet/IPv4 hw/proto type) check inside extract_arp_frame. Etherparse-reject frames never produce an ArpPacketSlice and are NOT D11 — they are handled by the existing decode error path (BC-2.16.009 EC-007). Aligns Decision 5 D11 trigger with Decision 7 key 11 (fixed in v1.7) and BC-2.16.009 EC-007."
  - "v1.9 (2026-06-13): Pass-20 D-02 (LOW) — Decision 6 MitreTactic enum assessment paragraph: reconciled inconsistent labeling of T0830's tactic source. Both the opening sentence and the bullet now consistently state that T0830's home matrix is ICS (TA0109 Lateral Movement), and that the wirerust code maps it to the shared MitreTactic::LateralMovement variant (no separate ICS variant exists) via the merge-by-name policy. Previously the opening sentence said 'ICS matrix (TA0109)' while the bullet said 'Enterprise Lateral Movement variant', creating an apparent contradiction. No mapping change — tactic assignment is unchanged."
subsystems_affected:
  - SS-02
  - SS-10
  - SS-16
supersedes: null
superseded_by: null
traces_to: .factory/specs/architecture/ARCH-INDEX.md
producer: architect
---

# ADR-008: Link-Layer Analyzer Integration via DecodedFrame Enum

> **One-per-file.** Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

wirerust's `decode_packet` function (SS-02, `src/decoder.rs`) currently returns
`Result<ParsedPacket>`, where `ParsedPacket` carries `src_ip`, `dst_ip`,
`protocol: Protocol`, and a `transport: TransportInfo`. ARP frames (EtherType 0x0806)
carry no IP layer. Under etherparse 0.16, ARP frames trigger the `None => Err(anyhow!("No
IP layer found"))` path and are silently dropped. This is correct for the existing IP-centric
analyzers but becomes a blocking limitation when adding an ARP Security Analyzer (SS-16).

The etherparse 0.20 migration (sub-delta A) introduces a third variant to both `NetSlice`
and `LaxNetSlice`: `NetSlice::Arp(ArpPacketSlice<'a>)` and `LaxNetSlice::Arp(ArpPacketSlice<'a>)`.
This makes the current two-arm `match` in `strict_ip_triple` and `lax_ip_triple` non-exhaustive
under 0.20. The migration therefore MUST address these match sites before the ARP analyzer can
land — this is a strict prerequisite dependency (sub-delta A precedes sub-delta B).

The central design question is: how does an ARP frame reach `ArpAnalyzer` given that it never
acquires a `ParsedPacket`, never enters the TCP reassembler, and never touches `StreamDispatcher`?
Three options were evaluated:

**Option 1 — `DecodedFrame` enum (SELECTED):** `decode_packet` returns `Result<DecodedFrame>`
where `DecodedFrame` is a sum type:
```
Ip(ParsedPacket)   // existing path — all code downstream of the decoder unchanged
Arp(ArpFrame)      // new — link-layer only, carries extracted ARP fields
```
`main.rs` pattern-matches on the enum: `Ip` variant routes to the existing IP pipeline;
`Arp` variant routes directly to `ArpAnalyzer::process_arp`.

**Option 2 — Pre-decode EtherType inspection in `main.rs`:** inspect the raw packet bytes for
EtherType 0x0806 before calling `decode_packet`. Rejected: requires two passes over raw bytes
(EtherType + full decode), duplicates DataLink-aware EtherType extraction logic already inside
etherparse, and makes `main.rs` the frame-classification owner rather than `decoder.rs`.

**Option 3 — `ProtocolAnalyzer` packet-level path:** add `ArpAnalyzer` to the existing
`ProtocolAnalyzer` trait flow. Rejected: `ProtocolAnalyzer::can_decode` takes `&ParsedPacket`,
which ARP frames never produce. Requires either modifying `ParsedPacket` to accommodate non-IP
frames (large structural change) or a separate pre-`ParsedPacket` hook — equivalent to Option 1
with more complexity.

ADR-007 (DNP3) established port-based classification at the `StreamDispatcher` level as the
entry point for binary ICS protocols over TCP. ARP operates below IP entirely — it bypasses
TCP reassembly, `StreamDispatcher`, and `ProtocolAnalyzer` by design. ADR-008 introduces a
second integration pattern that is link-layer-first rather than transport-layer-first.

## Decision

We integrate the ARP Security Analyzer via seven coordinated decisions:

### Decision 1: `DecodedFrame` enum as `decode_packet` return type

`decode_packet` is changed from `fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>`
to `fn decode_packet(data: &[u8], datalink: DataLink) -> Result<DecodedFrame>`.

The `DecodedFrame` enum is defined in `src/decoder.rs`:

```rust
/// Result of decoding a single captured frame.
///
/// - `Ip` — frame carried an IP layer; the existing IP-centric pipeline applies.
/// - `Arp` — frame was an Ethernet ARP packet (EtherType 0x0806); the ARP analyzer applies.
///   ARP frames have no IP layer, no TCP/UDP transport, and never enter the reassembler.
pub enum DecodedFrame {
    Ip(ParsedPacket),
    Arp(ArpFrame),
}
```

`ParsedPacket` is NOT modified. All existing analyzers, the reassembler, dispatcher, and all
BCs for SS-02 other than BC-2.02.009 remain fully valid. The change is additive at the decoder
level; callers absorb a pattern-match at the call site.

**Impact on `main.rs` packet loop:** The single `decode_packet(...)` call site pattern-matches:
```rust
match decode_packet(data, datalink) {
    Ok(DecodedFrame::Ip(p))  => { /* existing IP pipeline — unchanged */ }
    Ok(DecodedFrame::Arp(a)) => { /* new ARP path → ArpAnalyzer::process_arp(&a) */ }
    Err(e)                   => { /* existing error handling — unchanged */ }
}
```
The IP arm is structurally identical to the previous single-arm handling of `Ok(p)`.

**VP-008 obligation:** The cargo-fuzz harness for VP-008 (decode_packet no-panic) must be
updated to accept `Result<DecodedFrame>` — either variant is a non-panic outcome. The no-panic
invariant is unchanged; only the return type changes.

### Decision 2: `ArpFrame` struct — structured extraction from `ArpPacketSlice`

A new `ArpFrame` struct is defined in `src/decoder.rs` alongside `DecodedFrame`:

```rust
/// Structured ARP packet extracted from an Ethernet frame.
///
/// All addresses are fixed-size for Ethernet/IPv4 ARP (hardware_addr_len == 6,
/// proto_addr_len == 4). Frames with non-Ethernet hardware type or non-IPv4 protocol
/// type are extracted as-is; the detection layer validates field values.
#[derive(Debug, Clone)]
pub struct ArpFrame {
    /// ARP opcode: 1 = Request, 2 = Reply.
    pub operation: u16,
    /// Sender hardware address (MAC).
    pub sender_mac: [u8; 6],
    /// Sender protocol address (IPv4).
    pub sender_ip: [u8; 4],
    /// Target hardware address (MAC, may be zeroed in a Request).
    pub target_mac: [u8; 6],
    /// Target protocol address (IPv4).
    pub target_ip: [u8; 4],
    /// Outer Ethernet frame source MAC, for D12 L2/L3 sender mismatch detection.
    /// `Some(mac)` for ETHERNET captures (from `slice.link`); `None` for LINUX_SLL,
    /// RAW, IPV4, and IPV6 captures where no Ethernet link header is present.
    pub outer_src_mac: Option<[u8; 6]>,
    /// Total on-wire frame length in bytes.
    pub packet_len: usize,
}
```

The extraction function `extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
copies from the `ArpPacketSlice` accessors (confirmed against etherparse 0.20.1 docs):
- `arp.sender_hw_addr()` → `sender_mac` (returns `&[u8]`; copy first 6 bytes when `hw_addr_size()==6`)
- `arp.target_hw_addr()` → `target_mac`
- `arp.sender_protocol_addr()` → `sender_ip` (returns `&[u8]`; copy 4 bytes when `proto_addr_size()==4`)
- `arp.target_protocol_addr()` → `target_ip`
- `arp.operation().0` → `operation` (ArpOperation is a newtype; `.0` gives the raw u16)

Returns `None` for non-Ethernet (hw_addr_type ≠ ArpHardwareId::ETHERNET) or non-IPv4
(proto_addr_type ≠ EtherType::IPV4) frames, or if hardware/protocol address size fields
do not equal 6/4 respectively. `decode_packet` maps `None` to `Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`.

This is the pure-core extraction target for VP-024 Sub-property A (parse safety).

### Decision 3: `strict_ip_triple` / `lax_ip_triple` exhaustive match fix

Both helper functions in `src/decoder.rs` add an `Arp` arm under etherparse 0.20.

`strict_ip_triple` uses `unreachable!` because the `Ok(slice)` arm routes ARP frames
out before `strict_ip_triple` is ever called — this arm is a compile-safety net only:

```rust
// strict_ip_triple
NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple"),
```

`lax_ip_triple` MUST NOT use `unreachable!`. The lax fallback path is taken when
`from_ethernet` returns `Err(SliceError::Len(_))` — i.e., a snaplen-truncated packet.
In etherparse 0.20, a truncated ARP frame can yield `Some(LaxNetSlice::Arp(_))` from
the lax parser. That `Some(LaxNetSlice::Arp(_))` arm reaches `lax_ip_triple` before
any ARP early-exit occurs; an `unreachable!` there would panic at runtime, violating
VP-008 and VP-024 Sub-A. The arm is therefore handled explicitly:

```rust
// lax_ip_triple
LaxNetSlice::Arp(arp) => return Err(LaxNetArpSignal(arp)),
```

where `LaxNetArpSignal` is a private sentinel (or equivalent mechanism) that causes
the caller to route to `extract_arp_frame` with the truncated `arp` slice. A truncated
ARP body still yields the outer Ethernet frame source MAC and the ARP opcode in the
fixed-offset header fields; `extract_arp_frame` returns `None` if the address-size
fields are unreadable, mapping to `Err("truncated ARP frame")`.

The routing logic in `decode_packet` extracts the ARP variant in **both** the strict
`Ok(slice)` arm and the lax `Err(SliceError::Len(_))` arm:

**Strict arm (complete packet):**
```rust
Ok(slice) => match &slice.net {
    Some(NetSlice::Arp(arp)) => {
        // Extract the outer Ethernet src MAC from the link layer (for D12).
        // `slice.link` is Some(LinkSlice::Ethernet2(..)) for ETHERNET captures;
        // None for LINUX_SLL, RAW, IPV4, and IPV6 (see Decision 5 DataLink mapping).
        // Ethernet2Slice::source() confirmed to return [u8; 6] by value
        // (docs.rs etherparse 0.20.1, 2026-06-12); Some(eth.source()) is correct.
        let outer_src_mac: Option<[u8; 6]> = slice.link.as_ref().and_then(|l| {
            if let etherparse::LinkSlice::Ethernet2(eth) = l {
                Some(eth.source())
            } else {
                None
            }
        });
        match extract_arp_frame(arp, outer_src_mac, data.len()) {
            Some(frame) => Ok(DecodedFrame::Arp(frame)),
            None        => Err(anyhow!("Non-Ethernet/IPv4 ARP frame")),
        }
    }
    Some(net) => Ok(DecodedFrame::Ip(build_parsed(
        strict_ip_triple(net), &slice.transport, data.len(),
    ))),
    None => Err(anyhow!("No IP layer found")),
},
```

**Lax arm (snaplen-truncated packet):**
```rust
Err(SliceError::Len(_)) => {
    let lax = lax_parse(...);
    match &lax.net {
        Some(LaxNetSlice::Arp(arp)) => {
            // Truncated ARP: attempt extraction. Fixed-offset ARP header fields
            // (HTYPE, PTYPE, HLEN, PLEN, OPER, sender MAC, sender IP) are present
            // in a full 28-byte ARP header; extract_arp_frame returns None if the
            // slice is too short to read address fields, mapping to Err below.
            // outer_src_mac extracted from lax.link the same way as the strict arm.
            let outer_src_mac: Option<[u8; 6]> = lax.link.as_ref().and_then(|l| {
                if let etherparse::LinkSlice::Ethernet2(eth) = l {
                    Some(eth.source())
                } else {
                    None
                }
            });
            match extract_arp_frame(arp, outer_src_mac, data.len()) {
                Some(frame) => Ok(DecodedFrame::Arp(frame)),
                None        => Err(anyhow!("truncated ARP frame")),
            }
        }
        Some(net) => Ok(DecodedFrame::Ip(lax_ip_triple(net)...)),
        None      => Err(anyhow!("No IP layer found (truncated)")),
    }
}
```

**VP-008 and VP-024 Sub-A no-panic guarantee:** With this two-arm design, no reachable
`unreachable!` exists in the decode path for any `LaxNetSlice` variant. Every ARP frame
— truncated or complete — is routed to `extract_arp_frame`, which is itself panic-free
(Sub-A postcondition). The no-panic invariant of VP-008 is restored.

### Decision 4: `ArpAnalyzer` struct — binding table and detection state

`ArpAnalyzer` is defined in `src/analyzer/arp.rs` as a pure-core struct. It does not
implement `ProtocolAnalyzer` (which requires `&ParsedPacket`) or `StreamAnalyzer` (which
requires reassembled TCP bytes). It exposes a direct `process_arp(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding>` method called from `main.rs`.

**Canonical binding-table substrate decision:** The production `bindings` field uses
`HashMap<[u8; 4], BindingEntry>` (from `std::collections::HashMap`). This gives O(1)
average insert and lookup, which is appropriate for the O(65,536)-entry cap on every
packet. The `insert_binding_lru` helper has the signature:

```rust
fn insert_binding_lru(
    bindings: &mut HashMap<[u8; 4], BindingEntry>,
    ip: [u8; 4],
    mac: [u8; 6],
    cap: usize,
)
```

**`last_seen_ts` write responsibility (normative):** `insert_binding_lru` does NOT receive
a timestamp parameter. The caller — `process_arp`, which holds `timestamp_secs: u32` — is
responsible for writing `last_seen_ts` on every observation: immediately after
`insert_binding_lru` returns (for new entries) and on each access for existing entries.
`insert_binding_lru` reads `last_seen_ts` only during the eviction scan (linear min-search
when the table is at cap). This two-phase contract keeps `insert_binding_lru` a pure
structural operation on the map and places timestamp-awareness in `process_arp` where the
clock source lives. PO must apply this same split to BC-2.16.005 and BC-2.16.006 signatures.

LRU eviction in production is implemented by a `last_seen_ts: u32` field on `BindingEntry`
(updated on every access by `process_arp`) combined with a linear scan of all entries to
find the minimum `last_seen_ts` when the table is at cap. This is O(N) on overflow only; N ≤ 65,536.
This is accepted because overflow events are rare in well-sized captures and the cost is
bounded. **Kani surrogate (Sub-D VP-024):** Because `HashMap` with `RandomState` triggers
a Kani FFI incompatibility, the Sub-D proof harness uses `BTreeMap<[u8;4], BindingEntry>`
as a drop-in surrogate. The cap invariant (`len ≤ cap`) is a purely arithmetic property
independent of the map implementation; the proof is valid for `HashMap` by substitution.
This surrogate is documented in VP-024 Sub-D and does NOT affect the production type.

**Struct layout:**

```rust
pub struct ArpAnalyzer {
    /// IP address (as raw [u8; 4]) → last-seen MAC address binding.
    /// Bounded to MAX_ARP_BINDINGS entries; LRU eviction on overflow (by last_seen_ts).
    bindings: HashMap<[u8; 4], BindingEntry>,

    /// Storm-rate tracking: source MAC → per-window frame count.
    storm_counters: HashMap<[u8; 6], StormCounter>,

    /// Configurable spoof escalation threshold.
    /// First rebind → MEDIUM/Anomaly.
    /// rebind_count >= spoof_threshold within the flap window → HIGH/Likely.
    /// Default: 3. Exposed via --arp-spoof-threshold CLI flag.
    spoof_threshold: u32,

    /// Configurable storm rate: average-frames-per-second-since-window-start threshold
    /// (per BC-2.16.008 Invariant 2). Default: 50. Exposed via --arp-storm-rate CLI flag.
    storm_rate: u32,

    /// Total ARP frames processed.
    frames_analyzed: u64,

    /// Count of Request frames (operation == 1).
    request_count: u64,

    /// Count of Reply frames (operation == 2).
    reply_count: u64,

    /// Count of frames where operation is neither 1 nor 2 (any other u16 opcode).
    /// Required for the reconciliation invariant:
    ///   request_count + reply_count + other_opcode_count == frames_analyzed
    other_opcode_count: u64,

    /// Cumulative count of D1 ARP spoof findings emitted (both MEDIUM and HIGH).
    spoof_findings: u64,

    /// Cumulative count of D2 GARP findings emitted.
    garp_findings: u64,

    /// Cumulative count of D3 storm findings emitted.
    storm_findings: u64,

    /// Cumulative count of D12 L2/L3 mismatch findings emitted.
    mismatch_findings: u64,

    /// Cumulative count of D11 malformed-frame findings emitted (output-side count).
    malformed_findings: u64,

    /// Cumulative count of raw frames rejected by extract_arp_frame (input-side count).
    /// When --arp is active, one malformed_findings increment accompanies each
    /// malformed_frames increment (one finding per rejected frame); when --arp is absent,
    /// malformed_frames still increments but malformed_findings does not, so
    /// malformed_findings <= malformed_frames. Semantically distinct: malformed_frames
    /// counts inputs; malformed_findings counts outputs.
    /// Excluded from frames_analyzed — malformed frames never enter process_arp.
    malformed_frames: u64,
    // Note: bindings_tracked (summarize() key 5) is a snapshot of bindings.len()
    // at summarize() time; it requires no dedicated field.
}

struct BindingEntry {
    mac: [u8; 6],
    /// Number of distinct MACs seen for this IP (rebind count).
    rebind_count: u32,
    /// Timestamp of first rebind event for flap-window calculation.
    first_rebind_ts: Option<u32>,
    /// One-shot guard: HIGH-confidence spoof finding emitted this window.
    spoof_high_emitted: bool,
    /// LRU eviction key — entry with min last_seen_ts evicted at cap.
    last_seen_ts: u32,
}

struct StormCounter {
    count_in_window: u64,
    window_start_ts: u32,
    storm_emitted: bool,
}
```

**Bounded-resource constants (wirerust engineering choices, NOT industry-cited defaults):**

```rust
/// Maximum number of IP→MAC bindings tracked in the ArpAnalyzer binding table.
/// LRU eviction on overflow. Sized for large enterprise captures (one /16 subnet).
/// This is a wirerust engineering choice; no authoritative industry default exists.
const MAX_ARP_BINDINGS: usize = 65_536;

/// Maximum number of per-MAC storm counters tracked simultaneously.
const MAX_STORM_COUNTERS: usize = 4_096;

/// Rebind-count threshold for escalating a spoof finding from MEDIUM to HIGH.
/// First rebind: MEDIUM/Anomaly. rebind_count >= SPOOF_REBIND_ESCALATION_DEFAULT
/// within ARP_FLAP_WINDOW_SECS: HIGH/Likely.
/// Override via --arp-spoof-threshold. This is a wirerust engineering choice.
const SPOOF_REBIND_ESCALATION_DEFAULT: u32 = 3;

/// Time window in seconds for spoof flap escalation and storm rate calculation.
/// This is a wirerust engineering choice.
const ARP_FLAP_WINDOW_SECS: u32 = 60;

/// Default average-frames-per-second-since-window-start threshold for ARP storm detection (D3)
/// (per BC-2.16.008 Invariant 2). Override via --arp-storm-rate. This is a wirerust engineering choice.
const ARP_STORM_RATE_DEFAULT: u32 = 50;
```

All five constants are documented as wirerust engineering choices. The research in
`mitre-arp-additional-detections.md` §4 confirms that no authoritative numeric defaults
exist in the published literature (Snort, arpwatch, Zeek all use event/state logic without
numeric rate thresholds).

### Decision 5: Detection scope — five detections for v0.7.0

The following detections are in scope for v0.7.0. ARP scanning (D4) is explicitly deferred to
a fast-follow cycle (D4 requires a distinct-targets-per-window counter and maps to a different
MITRE technique class, T0840, creating a separate VP obligation).

| Detection | Class | Confidence | Stateful? | MITRE technique |
|-----------|-------|------------|-----------|-----------------|
| D1 ARP spoof / cache-poisoning | IP rebinds to new MAC in binding table | MEDIUM (first rebind) → HIGH (≥3 rebinds within ARP_FLAP_WINDOW_SECS) | YES (binding table) | T0830 + T1557.002 |
| D2 Gratuitous ARP (GARP) | sender_ip == target_ip | LOW base; MEDIUM when GARP also conflicts with existing binding (D2+D1 interaction) | NO (single-packet) | T0830 + T1557.002 |
| D3 ARP storm / rate anomaly | source MAC exceeds storm_rate average-frames-per-second-since-window-start (per BC-2.16.008 Invariant 2) | MEDIUM | YES (per-MAC window counter) | anomaly only (no MITRE tag — see below) |
| D11 Malformed / oversized ARP | hw_addr_size != 6 or proto_addr_size != 4 (or non-Ethernet/IPv4 hw/proto type) for ARP frames that produced an ArpPacketSlice; etherparse-reject frames are NOT D11 — handled by the existing decode error path, see BC-2.16.009 EC-007 | LOW | NO (structural check in extract_arp_frame) | anomaly only |
| D12 L2/L3 sender mismatch | Ethernet src MAC (from outer frame) != ARP sender HW addr field | MEDIUM | NO (single-packet) | T0830 + T1557.002 |

**D3 and D11 MITRE tagging rationale:** ARP storm could be argued toward T0814 Denial of
Service, but high-rate ARP from misconfigured devices is frequently benign in both IT and OT
contexts. The research (`mitre-arp-additional-detections.md` §3) notes T0814 was not
re-fetched live in the additional-detections pass. Per DF-VALIDATION-001, no technique tag
is attached to D3/D11 unless T0814 is validated live before F3 story authoring. D3 and D11
emit anomaly findings with no `mitre_techniques` tag as the conservative default. This is a
human decision point carried forward to F3.

**D12 requires the Ethernet source MAC to be surfaced.** The `ArpFrame` struct carries
`outer_src_mac: Option<[u8; 6]>` as specified in Decision 2 (canonical). The three-argument
extraction function `extract_arp_frame(arp, outer_src_mac, packet_len)` is the canonical
signature across this ADR; Decision 2 is the normative definition.

**DataLink→outer_src_mac mapping (authoritative):**

| DataLink variant | `outer_src_mac` value | Rationale |
|------------------|-----------------------|-----------|
| `ETHERNET` | `Some(eth.source())` from `slice.link` (`LinkSlice::Ethernet2`) | Ethernet frames carry a 6-byte src MAC in the link header; extracted via `etherparse::LinkSlice::Ethernet2` accessor |
| `LINUX_SLL` | `None` | Linux cooked capture has no real Ethernet header; SLL source address is a pseudo-address not suitable for D12 comparison |
| `RAW` | `None` | Raw IP capture has no link layer at all |
| `IPV4` | `None` | Raw IPv4 capture has no link layer at all |
| `IPV6` | `None` | Raw IPv6 capture has no link layer at all |

This is consistent with the `decode_packet` routing logic in Decision 3, which calls
`slice.link.as_ref().and_then(...)` to extract the MAC — `slice.link` is `None` for all
non-ETHERNET DataLink variants under etherparse 0.20 (`from_ip` and `from_linux_sll` paths
do not populate `slice.link` with an `Ethernet2` slice).

### Decision 6: MITRE catalog additions — VP-007 atomic update obligation

ARP spoofing detections emit `T0830` (ICS primary) and `T1557.002` (Enterprise secondary).
Both IDs are confirmed current and non-revoked in ATT&CK v19.1 (validated by research agent
in `.factory/phase-f1-delta-analysis/mitre-arp-research.md` §2, 2026-06-12).

**MitreTactic enum assessment (resolved — no placeholder):** T0830 belongs to tactic
"Lateral Movement" in the ICS matrix (TA0109); the wirerust `MitreTactic` enum has no
separate ICS-specific variant for this tactic and instead reuses the shared
`MitreTactic::LateralMovement` variant (the merge-by-name policy collapses ICS TA0109
"Lateral Movement" and Enterprise TA0008 "Lateral Movement" into one variant). T1557.002
belongs to "Credential Access" in Enterprise (TA0006). The inline merge-by-name comment in
`technique_info` (lines 145–148) explicitly states the merge-by-name policy: "we
intentionally merge by name so a single grouped report has one section per tactic name
regardless of source matrix." Consistent with this policy, ICS Discovery techniques (T0846,
T0888) already map to `MitreTactic::Discovery` (the shared variant), not to a separate
`IcsDiscovery`. The same rule applies here:

- **T0830 → `MitreTactic::LateralMovement`** (shared variant used for both ICS TA0109 and
  Enterprise TA0008 "Lateral Movement"; T0830's home matrix is ICS, mapped here via the
  merge-by-name policy — no separate ICS enum variant exists or is needed).
- **T1557.002 → `MitreTactic::CredentialAccess`** (Enterprise Credential Access, already
  present in the enum).

No new `MitreTactic` variant is required for either technique. The F3 implementer adds only
the `technique_info` match arms and the VP-007 5-part atomic update; no enum change needed.

**VP-007 5-part atomic update obligation** (same pattern as Modbus/ADR-005 and DNP3/ADR-007):

1. `technique_info` match arms: add `"T0830"` with name `"Adversary-in-the-Middle"` and
   tactic `MitreTactic::LateralMovement`; add `"T1557.002"` with name
   `"Adversary-in-the-Middle: ARP Cache Poisoning"` and tactic `MitreTactic::CredentialAccess`.
2. `SEEDED_TECHNIQUE_IDS` array: add `"T0830"` and `"T1557.002"`.
3. `SEEDED_TECHNIQUE_ID_COUNT` constant: bump 23 → 25 (two new IDs).
4. `EMITTED_IDS` in `kani_proofs` module: add `"T0830"` and `"T1557.002"` (both will be
   emitted by `ArpAnalyzer`).
5. `cargo test mitre` must pass before the PR merges.

**Emission summary (post-ARP v0.7.0):**

| ID | Emitted by | Status |
|----|-----------|--------|
| T0830 | ARP (D1 spoof, D2 GARP, D12 mismatch) | **NEW — add to EMITTED_IDS** |
| T1557.002 | ARP (D1 spoof, D2 GARP, D12 mismatch — Enterprise cross-tag) | **NEW — add to EMITTED_IDS** |

**VP-007 invariant (post-ARP):** SEEDED 25, EMITTED 17 (15 pre-ARP + T0830 + T1557.002).

### Decision 7: Finding flow — ARP findings reach reporters unchanged

`ArpAnalyzer::process_arp` returns `Vec<Finding>`. These findings use the existing
`crate::findings::Finding` struct with `mitre_techniques: Vec<String>` (ADR-006). No changes
to `findings.rs`, `reporter/json.rs`, `reporter/terminal.rs`, or `reporter/csv.rs` are required.
The `Finding` struct with `mitre_techniques: Vec<String>` already supports multi-technique
attribution (e.g. `["T0830", "T1557.002"]` on a single ARP spoof finding).

`main.rs` collects ARP findings alongside IP-pipeline findings and passes them to the reporter
in the same `findings` vec, maintaining the existing sort/group/report pipeline unchanged.

ARP analyzer summary is appended to `analyzer_summaries` in `main.rs`, following the
Modbus/DNP3 wiring pattern. The `ArpAnalyzer::summarize()` method returns `AnalysisSummary`
with the following **11 canonical `detail` keys** (ordered for documentation; map insertion
order does not affect correctness):

1. `frames_analyzed` — count of Ethernet/IPv4 ARP frames that passed `extract_arp_frame`
   and entered `process_arp`. Malformed frames (see key 11) are excluded from this count
   (per BC-2.16.009 / ARP-AMB-004 resolution: malformed frames are rejected by
   `extract_arp_frame` before reaching `process_arp`).
2. `request_count` — count of frames where `operation == 1` (ARP Request).
3. `reply_count` — count of frames where `operation == 2` (ARP Reply).
4. `other_opcode_count` — count of frames where `operation` is neither 1 nor 2 (any other
   u16 opcode value). Required for the reconciliation invariant (see below).
5. `bindings_tracked` — current count of IP→MAC entries in the binding table at summarize()
   time (snapshot, not cumulative).
6. `spoof_findings` — cumulative count of D1 spoof findings emitted (both MEDIUM and HIGH).
7. `garp_findings` — cumulative count of D2 GARP findings emitted.
8. `storm_findings` — cumulative count of D3 storm findings emitted.
9. `mismatch_findings` — cumulative count of D12 L2/L3 mismatch findings emitted.
10. `malformed_findings` — cumulative count of D11 malformed-frame findings emitted.
11. `malformed_frames` — cumulative count of raw frames rejected by `extract_arp_frame`
    (non-Ethernet/IPv4 ARP, wrong hw/proto addr sizes).
    This is the input-side count; `malformed_findings` (key 10) is the output-side count.
    When `--arp` is active, `malformed_findings` == `malformed_frames` (one finding per
    rejected frame). When `--arp` is absent, `malformed_frames` still increments (input-side
    count) but `malformed_findings` does not (no analysis findings emitted), so
    `malformed_findings` <= `malformed_frames`. The two counts are therefore semantically
    distinct: `malformed_frames` counts inputs; `malformed_findings` counts outputs.
    Note: etherparse parse failures (truncated/garbage ARP that never produce an
    ArpPacketSlice) are handled by the existing decode error path and are NOT counted in
    malformed_frames (see BC-2.16.009 EC-007).

**Reconciliation invariant (normative):**

```
request_count + reply_count + other_opcode_count == frames_analyzed
```

`malformed_frames` is excluded from `frames_analyzed` — malformed frames never enter
`process_arp`. The reconciliation invariant therefore holds exactly over the frames that
did pass extraction. BC-2.16.010 must document this invariant. This resolves ARP-AMB-004.

**Key count: 11.** Any BC, test vector, or HS-INDEX entry that lists a different number
of summarize() keys is inconsistent with this ADR and must be updated to align.

## Rationale

**`DecodedFrame` enum (Decision 1)** is the minimal layered change. `decoder.rs` is already the
single authority for frame-level classification; adding a second output variant extends that
authority to ARP without touching any downstream component. All existing SS-02 BCs remain valid
except BC-2.02.009, which requires a targeted text revision (Decision 1 note above and F1 delta
§5). The alternative (pre-decode EtherType inspection in `main.rs`) violates the principle that
decoder.rs owns frame-type dispatch and would require DataLink-aware byte inspection outside of
etherparse's parsing path.

**Fixed-size `ArpFrame` struct (Decision 2)** avoids lifetime parameters in the analyzer (the
`ArpPacketSlice<'a>` is borrowed from the raw packet slice which does not outlive the loop
iteration). Copying the 20 bytes of Ethernet/IPv4 ARP addresses into owned arrays is a
negligible cost, makes `ArpFrame` `Clone + 'static`, and eliminates any lifetime-propagation
complexity in the `ArpAnalyzer`.

**LRU-bounded binding table (Decision 4)** bounds memory usage in long-running captures with
many hosts. The cap of 65,536 entries is sufficient for a /16 IPv4 subnet's worth of distinct
IP addresses. The LRU policy (evict entry with minimum `last_seen_ts` on overflow) ensures
that long-tail IPs observed once early in a capture are evicted before fresh IPs trigger false
eviction of recently-active bindings. `HashMap<[u8;4], BindingEntry>` is the production
substrate; `BTreeMap` is used only as a Kani proof surrogate for Sub-D (see Decision 4).
This is documented as a wirerust engineering choice per `mitre-arp-additional-detections.md` §4b.

**Spoof escalation model (Decision 5, D1)** avoids high-confidence findings on the first
MAC rebind, which is a legitimate event on networks with DHCP churn, VM migration, or
NIC replacement. First-rebind MEDIUM gives analysts a signal without forcing them to
investigate every DHCP lease renewal as a security incident. Escalation to HIGH after
≥3 distinct MACs for one IP within 60 seconds mirrors the DNP3 burst-escalation pattern
(ADR-007 Decision 4) and is a wirerust engineering choice.

**No MITRE tag on D3/D11 (Decision 5)** follows the research's explicit recommendation
(`mitre-arp-additional-detections.md` §3 guidance note on D3/D11) and DF-VALIDATION-001.
Forcing T0814 without a live validation would repeat the T0803/T0855 correction pattern from
DNP3 (ADR-007 Decision 5 context note).

**Dual ICS+Enterprise tagging (Decision 6)** is consistent with wirerust's existing convention
of carrying both matrices on multi-matrix techniques (established by Modbus: T1692.001 Enterprise
sub-technique emitted alongside ICS T0836/T0814). ARP spoofing is relevant in both OT and IT
networks; dual-tagging maximizes utility for analysts using either MITRE matrix.

## Consequences

### Positive

- ARP frames are correctly routed and analyzed, enabling ICS/OT AiTM detection (T0830)
  and Enterprise ARP cache-poisoning detection (T1557.002).
- `decode_packet`'s return type change is additive: no existing IP-path code changes
  except the call site in `main.rs` (which gains one `match` level).
- `ParsedPacket` is unchanged; all SS-02 BCs except BC-2.02.009 remain valid without
  modification.
- The ARP analyzer is a pure-core module with no I/O, enabling formal verification (VP-024).
- The binding table is bounded (MAX_ARP_BINDINGS = 65,536, LRU) and prevents unbounded
  memory growth across large captures.
- D12 (Ethernet/ARP sender mismatch) is a near-zero FP stateless check that provides
  coverage identical to Snort GID 112 SID 2/3 — the most reliable stateless ARP signal.

### Negative / Trade-offs

- **BC-2.02.009 text revision required.** The BC currently states that ARP frames produce
  `Err("No IP layer found")`. After this ADR, ARP frames produce `Ok(DecodedFrame::Arp(...))`.
  The revised postcondition: "Non-IP frames that are not ARP return `Err("No IP layer found")`.
  ARP frames (EtherType 0x0806) return `Ok(DecodedFrame::Arp(...))`."
- **VP-008 fuzz harness update required.** The fuzz target for `decode_packet` must accept
  `Result<DecodedFrame>` rather than `Result<ParsedPacket>`. The no-panic invariant is
  unchanged.
- **D12 requires outer Ethernet MAC extraction.** The `ArpFrame` struct carries
  `outer_src_mac: Option<[u8; 6]>` (Decision 2, canonical) and `extract_arp_frame` receives
  it as a parameter (Decision 3 routing extracts it from `slice.link`). The DataLink→value
  mapping is specified in Decision 5. This is a two-line addition to the decoder at the
  call site in `decode_packet`.
- **LRU eviction is not cryptographically safe.** An attacker who can inject ≥65,536 distinct
  source IPs can evict legitimate bindings from the table, causing missed detections.
  wirerust is an offline PCAP forensics tool; this is an accepted trade-off for memory
  bounding. The cap is configurable if needed.
- **SEEDED_TECHNIQUE_ID_COUNT bump to 25** requires the VP-007 5-part atomic update in the
  same commit as technique_info arms for T0830 and T1557.002. The `vp007_catalog_drift_guard`
  test will mechanically fail if the five parts are not co-committed.

### BC-2.02.009 revised postcondition (normative)

> **Previous:** An ARP frame (EtherType 0x0806, no IP layer) causes `decode_packet` to return
> `Err("No IP layer found")`.
>
> **Revised:** An ARP frame (EtherType 0x0806) with Ethernet/IPv4 addresses causes `decode_packet`
> to return `Ok(DecodedFrame::Arp(ArpFrame { ... }))`. Non-Ethernet/IPv4 ARP frames return
> `Err("Non-Ethernet/IPv4 ARP frame")`. Non-IP frames that are not ARP (e.g. LLDP, AppleTalk)
> continue to return `Err("No IP layer found")`.

### Open Items for F3 / Human Decision

- **D3/D11 MITRE tagging:** validate T0814 live per DF-VALIDATION-001 before attaching to
  storm/malformed findings in F3 BCs. No technique tag shipped until validated.
- **`MitreTactic` lateral-movement variant:** RESOLVED in Decision 6 — T0830 uses the
  existing `MitreTactic::LateralMovement` variant (merge-by-name policy). No new enum
  variant is required. T1557.002 uses `MitreTactic::CredentialAccess` (already present).
- **D4 ARP scanning:** deferred to fast-follow cycle. Requires T0840 live validation and a
  `scan_threshold` counter. Not in v0.7.0 scope.
- **Storm rate default (50/s):** a conservative engineering choice. OT operators with ICS
  field devices should lower this; expose via `--arp-storm-rate`.

## Source / Origin

- **ARP frame format:** RFC 826; confirmed via etherparse 0.20.1 ArpPacketSlice accessor docs
  fetched live from docs.rs on 2026-06-12.
- **etherparse 0.20.1 API confirmation:** `NetSlice::Arp(ArpPacketSlice)` and
  `LaxNetSlice::Arp(ArpPacketSlice)` confirmed current from live docs.rs fetch (2026-06-12).
  `SliceError::Len` variant confirmed still present in 0.20.1 — no error-taxonomy change.
  `SlicedPacket.vlan` confirmed renamed to `SlicedPacket.link_exts` (type:
  `ArrayVec<LinkExtSlice, LINK_EXTS_CAP>`). `ArpPacketSlice` accessor method names confirmed:
  `sender_hw_addr()`, `target_hw_addr()`, `sender_protocol_addr()`, `target_protocol_addr()`,
  `operation()`, `hw_addr_type()`, `proto_addr_type()`.
  **`Ethernet2Slice::source()` return type CONFIRMED:** `pub fn source(&self) -> [u8; 6]`
  returns `[u8; 6]` **by value** (not `&[u8; 6]` or `&[u8]`). Source: docs.rs
  etherparse 0.20.1 `Ethernet2Slice` struct page, fetched live 2026-06-12. The
  Decision 3 routing snippet `Some(eth.source())` writing into `Option<[u8; 6]>` is
  correct as written; no dereference (`*eth.source()`) or `.to_owned()` is required.
  `Ethernet2Slice::destination()` also returns `[u8; 6]` by value (same struct, same pattern).
- **MITRE technique IDs:** T0830, T1557.002 confirmed current in ATT&CK v19.1 per
  `.factory/phase-f1-delta-analysis/mitre-arp-research.md` §2 (live page fetches 2026-06-12).
- **Detection scope / threshold defensible-defaults:** `.factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md`
  §2 (detection catalogue) and §4b (wirerust engineering-choice defaults table).
- **F1 delta integration-path analysis:** `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
  §2 (option evaluation) and §4 (affected artifacts).
- **Modbus/DNP3 integration precedents:** ADR-005, ADR-007.
