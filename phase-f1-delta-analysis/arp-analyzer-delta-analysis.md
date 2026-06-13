---
document_type: feature-delta-analysis
feature_id: issue-arp-security-analyzer
github_issue: TBD
title: "Full ARP Security Analyzer (with etherparse 0.16 → 0.20.1 migration)"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  Link-layer integration path (no IpTriple) requires new decode output types and a
  parallel dispatch lane outside the TCP reassembly pipeline. etherparse 0.20 API
  breaks two match sites in decoder.rs (NetSlice/LaxNetSlice non-exhaustive),
  renames the vlan field to link_exts across SlicedPacket/LaxSlicedPacket, replaces
  IPv4/IPv6 ECN/DSCP types, and changes the LenError taxonomy. ARP spoof detection
  requires cross-packet state (IP↔MAC binding table). New subsystem SS-16, new
  VP-024, new ADR-008. Minimum 8 source files changed across two distinct deltas
  (migration sub-delta + ARP analyzer delta). Not a trivial change.
scope_classification: standard
status: draft
producer: architect
created: 2026-06-12
base_commit: develop HEAD (post-STORY-110, post-Wave-39 close, v0.6.0)
branch: develop
prior_feature_precedent:
  - issue-007-modbus-analyzer (D-032..D-046, SS-14)
  - issue-008-dnp3-analyzer (ADR-007, SS-15, STORY-106..110)
mitre_research_status: >
  VALIDATION COMPLETE (2026-06-12). MITRE technique IDs T0830 (ICS LateralMovement)
  and T1557.002 (Enterprise CredentialAccess) validated by research agent against
  ICS ATT&CK v19.1 and Enterprise ATT&CK. Source: .factory/phase-f1-delta-analysis/
  mitre-arp-research.md (2026-06-12, Confidence HIGH). Both IDs confirmed active
  (non-revoked) in their respective ATT&CK domains. cap-10/HS-INDEX now carry them
  as confirmed. This status supersedes the prior TBD-pending placeholder.
modified:
  - "v1.1 (2026-06-13, ARP-F2 Pass-14 PO Burst 2, C-06 LOW): mitre_research_status
    updated from TBD-pending placeholder to VALIDATION COMPLETE. Research validated
    T0830 and T1557.002 in mitre-arp-research.md (2026-06-12, Confidence HIGH).
    No F1 analytical conclusions altered."
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
  - .factory/specs/dtu-assessment.md
---

# F1 Delta Analysis — ARP Security Analyzer (with etherparse 0.16 → 0.20.1 migration)

## Executive Summary

This feature has two structurally distinct sub-deltas that must be understood and
implemented in dependency order:

**Sub-delta A: etherparse 0.16 → 0.20.1 migration (required prerequisite)**

The etherparse upgrade is a *required prerequisite* to the ARP analyzer: ARP frames
are only accessible via the new `NetSlice::Arp` / `LaxNetSlice::Arp` variant that
etherparse 0.20 added. This sub-delta is primarily a compile-break repair across
`src/decoder.rs` and the Kani proofs in `src/dispatcher.rs`, with additional
non-breaking changes required for the renamed `vlan` → `link_exts` field and the
`Ipv4Ecn`/`Ipv4Dscp` → `IpEcn`/`IpDscp` rename. The migration also makes the
`NetSlice` / `LaxNetSlice` match arms non-exhaustive (previously two-arm; now
three-arm with `Arp`), which is the compile break.

**Sub-delta B: ARP Security Analyzer (the main feature)**

ARP has no IP layer and no TCP/UDP port. It bypasses the entire existing pipeline
(TCP reassembly, StreamDispatcher, `ParsedPacket`/`IpTriple`). The central
architectural question is: **where and how does an ARP frame reach an analyzer when
the existing pipeline discards it at decoder.rs line 148 ("No IP layer found")?**

This document resolves that question, enumerates every affected artifact, provides
an architectural recommendation for the link-layer integration path, estimates the
new artifact count, and proposes the F2→F7 dependency-ordered plan.

---

## 1. Intent and Scope Classification

| Field | Value |
|-------|-------|
| Intent | feature (new capability; no prior ARP analysis exists) |
| Feature type | backend |
| Trivial | NO |
| Trivial justification | See frontmatter. |
| Scope classification | standard — full F1-F7 cycle required |
| Recommended subsystem | SS-16 (new; SS-14=Modbus, SS-15=DNP3; SS-16=ARP) |
| MITRE research status | TBD-pending-research-agent (IDs T0830/T1557.002 are placeholders) |

---

## 2. The Central Architectural Question

### 2.1 Why ARP Cannot Use the Existing Pipeline

The current pipeline is IP-centric by design. The decode path in `src/decoder.rs`
(`decode_packet`) is the entry point, and it terminates with an error for any frame
that lacks an IP layer:

```
src/decoder.rs line 148: None => Err(anyhow!("No IP layer found"))
```

This is not a bug — it is the correct behavior for the existing IP-centric analyzers.
`ParsedPacket` carries `src_ip: IpAddr`, `dst_ip: IpAddr`, and `protocol: Protocol`.
None of these fields have meaning for an ARP frame. The `IpTriple` type alias
`(IpAddr, IpAddr, IpNumber)` used internally is entirely IP-specific.

The TCP reassembly pipeline (`TcpReassembler`, `StreamDispatcher`) operates exclusively
on `ParsedPacket` structs that already have a valid IP layer. An ARP analyzer cannot
be wired in at the `StreamDispatcher` level because no ARP flow ever reaches the
dispatcher.

In etherparse 0.20, when `SlicedPacket::from_ethernet` successfully parses an ARP
Ethernet frame, it produces `SlicedPacket { net: Some(NetSlice::Arp(...)), transport: None, ... }`.
The `strict_ip_triple` and `lax_ip_triple` functions in `decoder.rs` currently match
only `NetSlice::Ipv4` and `NetSlice::Ipv6` — adding the `Arp` variant would be a
non-exhaustive match compile error under 0.20.

### 2.2 Integration Path Options

**Option 1: New `LinkLayerFrame` output type from `decode_packet` (RECOMMENDED)**

Extend `decode_packet` to return a sum type instead of `ParsedPacket`:

```rust
pub enum DecodedFrame {
    Ip(ParsedPacket),       // existing path — all existing code unchanged
    Arp(ArpFrame),          // new — link-layer only, no IP triple
}
```

Where `ArpFrame` carries:
- `operation: ArpOperation` (Request / Reply)
- `sender_mac: [u8; 6]`
- `sender_ip: Ipv4Addr`
- `target_mac: [u8; 6]`
- `target_ip: Ipv4Addr`
- `packet_len: usize`

**Advantages:**
- `decode_packet` is the natural single point where the Ethernet type is known.
  In etherparse 0.20, the `NetSlice::Arp` variant surfaces here first.
- Existing callers (`main.rs` packet loop) change minimally: they pattern-match on
  the enum and route `DecodedFrame::Ip(p)` to the existing IP pipeline (unchanged)
  and `DecodedFrame::Arp(a)` to the new ARP path.
- The `ParsedPacket` type is **not modified**. All existing analyzers, the
  reassembler, and all BCs for SS-02 remain valid.
- No change to `StreamDispatcher` or `TcpReassembler`.
- `ArpAnalyzer` receives individual `ArpFrame` values directly — it does not need
  TCP flow state or a carry buffer (ARP frames are self-contained in a single Ethernet
  frame; no fragmentation).

**Option 2: Separate packet-loop pass in `main.rs`**

Keep `decode_packet` returning `Result<ParsedPacket>` (returning `Err` for ARP as
before). Add a pre-decode pass in `main.rs` that inspects each raw packet's EtherType
before calling `decode_packet`. If EtherType is 0x0806 (ARP), route directly to
`ArpAnalyzer`. Otherwise proceed as today.

**Disadvantages vs Option 1:**
- Requires two passes over the raw packet bytes (EtherType inspection + etherparse decode).
- The EtherType check needs to be DataLink-aware (SLL vs Ethernet have different
  EtherType positions), duplicating logic already inside etherparse.
- `main.rs` becomes the routing owner rather than `decode_packet`, violating the layering
  principle that `decoder.rs` is the single frame-type dispatch authority.

**Option 3: ProtocolAnalyzer packet-level path (DNS precedent)**

Add `ArpAnalyzer` implementing `ProtocolAnalyzer::can_decode` keyed on a new
`Protocol::Arp` variant, invoked from the main packet loop before the reassembler.

**Disadvantages vs Option 1:**
- `ProtocolAnalyzer::can_decode` takes a `&ParsedPacket`, but ARP frames never produce
  a `ParsedPacket` (they fail at `decode_packet`). This path requires either modifying
  `ParsedPacket` to accommodate non-IP frames (a large structural change) or adding a
  separate pre-`ParsedPacket` hook — which is functionally equivalent to Option 1 with
  more complexity.

**RECOMMENDATION: Option 1 — `DecodedFrame` enum output from `decode_packet`.**

This is the minimal, layered change. The decoder becomes the single authority for
frame classification at the link layer. Existing IP-path code is untouched. The ARP
analyzer is a pure-core function receiving a structured `ArpFrame` value.

### 2.3 ARP Spoof Detection State Architecture

Gratuitous ARP (GARP) and ARP cache poisoning require **cross-packet state**: a
binding table that maps `IpAddr → MacAddr` and detects when the mapping changes.

The `ArpAnalyzer` must own an `ArpBindingTable`:

```rust
pub struct ArpAnalyzer {
    /// IP → last-seen MAC binding table.
    /// Bounded to MAX_ARP_BINDINGS entries (LRU eviction on overflow).
    bindings: HashMap<Ipv4Addr, [u8; 6]>,

    /// Total ARP frames analyzed.
    frames_analyzed: u64,

    /// Counts by operation (Request / Reply).
    request_count: u64,
    reply_count: u64,
}
```

Detection rules (to be confirmed by research agent in F2):
1. **Gratuitous ARP**: sender IP == target IP in an ARP Reply, OR sender IP == target IP
   in an ARP Request (less common GARP form). May be legitimate (IP address announcement
   on link up) or malicious. Emit low/medium confidence anomaly finding.
2. **ARP cache poisoning / Adversary-in-the-Middle**: An IP address is observed binding
   to a new MAC address after a binding already exists in the table. High confidence
   finding. Maps to MITRE technique TBD-pending-research (T0830 or T1557.002 — research
   agent must confirm correct IDs for ICS-ATT&CK and/or Enterprise ATT&CK).
3. **ARP storm / scanning**: Anomalously high rate of ARP requests from a single MAC.
   Rate-based, similar to Modbus burst detection. Emit low/medium confidence anomaly.

### 2.4 Finding Flow Integration

`ArpAnalyzer` emits `Finding` values using the existing `crate::findings::Finding`
struct. These findings flow into the same reporting pipeline (JSON/Terminal/CSV
reporters) without modification. The `Finding` struct with `mitre_techniques: Vec<String>`
(post-ADR-006) already accommodates ARP findings. No change to `findings.rs` or
reporters is required.

MITRE technique IDs must be seeded in `src/mitre.rs` via the VP-007 atomic update
obligation (same 5-part pattern as Modbus and DNP3). The exact IDs are
TBD-pending-research-agent validation.

---

## 3. etherparse 0.16 → 0.20.1: Concrete API Break Inventory

### 3.1 Compile-Breaking Changes

The following changes WILL prevent `cargo build` from succeeding after a version bump
to 0.20.1. Each must be addressed in the migration sub-delta.

**Break 1: Non-exhaustive `NetSlice` match in `strict_ip_triple` (decoder.rs:209-228)**

```rust
// CURRENT (0.16 — two-arm match, exhaustive)
fn strict_ip_triple(net: &NetSlice<'_>) -> IpTriple {
    match net {
        NetSlice::Ipv4(ipv4) => { ... }
        NetSlice::Ipv6(ipv6) => { ... }
    }
}
```

In etherparse 0.20, `NetSlice` gains `NetSlice::Arp(ArpPacketSlice<'_>)`. The existing
two-arm match becomes non-exhaustive and will fail to compile.

**Fix:** Add `NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple")`.

OR (preferred, per the Option 1 architecture): `strict_ip_triple` is no longer called
for ARP frames because `decode_packet` routes them to `DecodedFrame::Arp` before
attempting `strict_ip_triple`. The match arm becomes unreachable in practice but must
still compile. Adding the `Arp` arm with `unreachable!` or a panic is the safe approach.

**Break 2: Non-exhaustive `LaxNetSlice` match in `lax_ip_triple` (decoder.rs:231-250)**

Same issue: `LaxNetSlice` gains `LaxNetSlice::Arp(ArpPacketSlice<'_>)` in 0.20.

```rust
// CURRENT (0.16 — two-arm match, exhaustive)
fn lax_ip_triple(net: &LaxNetSlice<'_>) -> IpTriple {
    match net {
        LaxNetSlice::Ipv4(ipv4) => { ... }
        LaxNetSlice::Ipv6(ipv6) => { ... }
    }
}
```

**Fix:** Same as Break 1 — add `LaxNetSlice::Arp(_) => unreachable!(...)` arm.

**Break 3: `SlicedPacket::net` type change / `vlan` field renamed to `link_exts`**

In etherparse 0.20.1, the `vlan` field in `SlicedPacket`, `LaxSlicedPacket`,
`PacketHeaders`, and `LaxPacketHeaders` was replaced by a `link_exts` field. If any
code accesses `.vlan` on these types it will fail to compile.

**Assessment for wirerust:** Searching `src/decoder.rs` — the code accesses
`slice.net` and `lax.net` but does NOT access `.vlan`. This change is
**non-breaking for wirerust's current code**. It will be breaking only if new ARP
analysis code or ARP tests access VLAN fields — which is unlikely.

**Break 4: `Ipv4Ecn` / `Ipv4Dscp` renamed to `IpEcn` / `IpDscp`**

etherparse 0.20 renamed these to be IP-version-agnostic types (IPv6 now uses them too).

**Assessment for wirerust:** `src/decoder.rs` reads `header.protocol()`,
`header.source_addr()`, `header.destination_addr()` on both `Ipv4HeaderSlice` and
`Ipv6HeaderSlice` — none of these use `Ipv4Ecn` or `Ipv4Dscp`. This change is
**non-breaking for wirerust's current code** unless tests or the ARP module explicitly
reference these types.

**Break 5: `SliceError::Len` — error taxonomy change**

In etherparse 0.16, the truncation fallback in `decode_packet` (decoder.rs:158) keys
on `SliceError::Len(_)`:

```rust
Err(SliceError::Len(_)) => {
    let lax = lax_parse(data, datalink)?;
    ...
}
```

**In etherparse 0.20, `SliceError::Len` no longer exists as a variant.** The error
type for length violations is now `LenError` exposed differently from the slice error
hierarchy. The comment in decoder.rs (line 44-48) already anticipates this:

```
// Note: ... This is part of the etherparse 0.16 API contract; `Cargo.toml` constrains
// the dependency to `0.16.x` accordingly. A future 0.17 bump must re-verify the error
// taxonomy — test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing and
// test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered
// act as the contract tests for it.
```

This is the **most architecturally significant compile break**. The exact replacement
in 0.20 must be confirmed during F2 spec work by reading the etherparse 0.20 changelog
and docs for `SlicedPacket::from_ethernet`'s error type. The two contract tests
named in the comment serve as the verification oracle — they must be re-confirmed
green after the migration.

**Fix skeleton:** Replace `Err(SliceError::Len(_)) =>` with the appropriate 0.20
error variant. Likely involves matching on a `LenError` variant from the restructured
error type. The exact match pattern is an F2 implementation task requiring etherparse
0.20 error docs.

**Break 6: `LaxSlicedPacket::from_linux_sll` — still absent in 0.20**

The existing workaround in `lax_parse` (decoder.rs:193-205) manually extracts the
EtherType from the SLL header and calls `LaxSlicedPacket::from_ether_type` because
etherparse 0.16 has no `LaxSlicedPacket::from_linux_sll`. Research confirms this
method **does not exist in etherparse 0.20 either**. The existing workaround code
should continue to compile in 0.20 (it does not use removed APIs), but must be
verified. This is a **carry-forward concern**, not a new break.

### 3.2 Non-Breaking Changes Requiring Attention

- The `NetSlice::Arp` variant is new but is only reached if decoder logic explicitly
  handles it. The migration fix adds the arm; the ARP analyzer consumes it.
- `ArpPacketSlice<'a>` is the type contained in `NetSlice::Arp`. It exposes:
  `operation()` (request/reply), `sender_hardware_addr()`, `sender_protocol_addr()`,
  `target_hardware_addr()`, `target_protocol_addr()`, `hardware_type()`,
  `protocol_type()`. Exact method names must be verified against the etherparse 0.20
  docs during F2.
- `EtherType` constants are unchanged between 0.16 and 0.20. `EtherType::ARP`
  (0x0806) exists in both versions.

### 3.3 Migration Sub-Delta: Files Requiring Changes

| File | Change Required | Risk |
|------|----------------|------|
| `Cargo.toml` | `etherparse = "0.16"` → `etherparse = "0.20"` | LOW (mechanical) |
| `src/decoder.rs` | Fix Break 1 (`strict_ip_triple` Arp arm), Break 2 (`lax_ip_triple` Arp arm), Break 5 (`SliceError::Len` replacement), Break 6 verification; add `DecodedFrame` enum and ARP extraction path | HIGH |
| `src/main.rs` | Update packet loop to match on `DecodedFrame::Ip` vs `DecodedFrame::Arp` | MEDIUM |
| `src/decoder.rs` tests | The two contract tests named in the line-44 comment (`test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`, `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered`) must be re-verified green under 0.20 | MEDIUM |
| `src/dispatcher.rs` (Kani) | VP-008 (`decode_packet` no-panic, cargo-fuzz) needs re-run under 0.20 since the error taxonomy changed | MEDIUM |

---

## 4. Affected Artifacts Inventory

### 4.1 NEW Source Files

| File | Role |
|------|------|
| `src/analyzer/arp.rs` | `ArpAnalyzer`: binding table, GARP detection, ARP spoofing/poisoning detection, rate anomaly, Finding emission. Pure core. |

### 4.2 MODIFIED Source Files

| File | Change Type | Change Description | Risk |
|------|------------|-------------------|------|
| `Cargo.toml` | MODIFIED | `etherparse = "0.16"` → `etherparse = "0.20"` (or `"0.20.1"`) | LOW |
| `src/decoder.rs` | MODIFIED (significant) | (1) Add `DecodedFrame` enum; (2) extend `decode_packet` to return `Result<DecodedFrame>`; (3) add `NetSlice::Arp` / `LaxNetSlice::Arp` match arms to `strict_ip_triple`/`lax_ip_triple`; (4) replace `SliceError::Len` with 0.20 equivalent; (5) add `ArpFrame` struct and extraction from `ArpPacketSlice`. Comment on line 22 ("etherparse 0.16 API contract") must be updated. | HIGH — CRITICAL path |
| `src/main.rs` | MODIFIED | (1) Update packet loop: `decode_packet` returns `DecodedFrame`; route `Ip` variant to existing IP pipeline (unchanged), `Arp` variant to `ArpAnalyzer::process_arp`; (2) wire `--arp` CLI flag → `ArpAnalyzer` construction; (3) collect ARP findings post-loop; (4) push ARP summary to `analyzer_summaries`. Mirror 4-step Modbus/DNP3 wiring pattern. | MEDIUM |
| `src/cli.rs` | MODIFIED | Add `#[arg(long)] arp: bool` flag to `Commands::Analyze`; add `*arp \|\| *all` expansion; add `--arp-spoof-threshold` (max IP-MAC rebinds per window before spoof finding fires) analogous to Modbus write burst threshold. | MEDIUM |
| `src/analyzer/mod.rs` | MODIFIED | Add `pub mod arp;`. Single line. | LOW |
| `src/mitre.rs` | MODIFIED (CRITICAL) | Add new MITRE technique IDs for ARP (TBD-pending-research — T0830 "Adversary-in-the-Middle" for ICS, T1557.002 "ARP Cache Poisoning" for Enterprise). VP-007 5-part atomic update obligation applies: `technique_info` arm + `SEEDED_TECHNIQUE_IDS` + `SEEDED_TECHNIQUE_ID_COUNT` + `EMITTED_IDS` + `cargo test mitre`. Exact IDs must be confirmed by research agent before F2 BCs are written. | CRITICAL |
| `src/decoder.rs` line 44-48 comment | MODIFIED | Update the doc comment that references "etherparse 0.16 API contract" to reference the 0.20 contract and the new error type. | LOW |

### 4.3 NEW Factory Spec Artifacts

| Artifact | Notes |
|----------|-------|
| Subsystem SS-16 "ARP Security Analysis" | New BC namespace BC-2.16.NNN |
| `.factory/specs/behavioral-contracts/ss-16/` | New directory for BC-2.16.NNN files |
| `VP-024` (proposed) | ARP frame parse safety + binding-table invariants. Kani candidate for pure-core functions. See §5. |
| `vp-024-arp-parse-safety.md` | VP-024 file in `verification-properties/` |
| ADR-008 (proposed) | ARP link-layer integration decisions: `DecodedFrame` output type, binding-table design, GARP vs poisoning detection semantics, MITRE technique selection. |
| `.factory/specs/architecture/decisions/ADR-008-arp-security-analyzer.md` | ADR file |

### 4.4 MODIFIED Factory Spec Artifacts

| Artifact | Change |
|----------|--------|
| `BC-INDEX.md` | Add SS-16 section (BC-2.16.001..NNN rows) |
| `ARCH-INDEX.md` | Add SS-16 to Subsystem Registry; update module-decomposition.md to add C-23 `src/analyzer/arp.rs` |
| `VP-INDEX.md` | Add VP-024; bump total 23→24; bump p1 9→10; bump kani 10→11 |
| `verification-architecture.md` | Add VP-024 row to Provable Properties Catalog; update P1 list; update summary counts |
| `verification-coverage-matrix.md` | Add VP-024 row; update per-module Kani count; update Totals row (Kani 10→11, total 23→24) |
| `module-criticality.md` | Add `ArpAnalyzer` (HIGH criticality — ARP spoof detection is a security-relevant finding; binding table correctness is forensic-fidelity concern) |
| `module-decomposition.md` | Add C-23 `src/analyzer/arp.rs` (SS-16, pure core); update C-5 `src/decoder.rs` description (returns `DecodedFrame` enum post-migration) |
| `dtu-assessment.md` | Update to note that ARP sub-delta confirms `DTU_REQUIRED: false` (no external services introduced) |

### 4.5 DEPENDENT (Regression Zone — must stay green)

| Component | Risk | Notes |
|-----------|------|-------|
| All existing BCs for SS-02 (decoder.rs) | HIGH | BC-2.02.001..015 describe current `decode_packet` behavior. BC-2.02.009 ("Surface No IP Layer Found Error for Non-IP Frames") is directly affected: ARP frames will no longer return `Err("No IP layer found")` — they return `DecodedFrame::Arp`. BC-2.02.009 must be revised to "Non-IP frames other than ARP return Err". This is a BC update, not a regression. |
| `tests/decoder_tests.rs` | HIGH | Tests for non-IP frame rejection (BC-2.02.009) must be updated. Tests for the snaplen truncation fallback must be re-verified under etherparse 0.20 error types. |
| VP-008 (decode_packet no-panic, cargo-fuzz) | HIGH | The fuzz harness runs against `decode_packet`. After the return type changes to `DecodedFrame`, the fuzz harness must be updated to accept either variant. No-panic property still holds. |
| `src/dispatcher.rs` / VP-004 | NONE | `StreamDispatcher` is unchanged. ARP frames never reach the dispatcher (they are routed at the decoder level, before the reassembler). VP-004 is unaffected. |
| `src/reassembly/` (all files) | NONE | The reassembler operates on `ParsedPacket` values. These still come from `DecodedFrame::Ip(p)`. No change. |
| All HTTP/TLS/DNS/Modbus/DNP3 tests | NONE | The IP path in `decode_packet` is structurally unchanged; it just requires pattern-matching `DecodedFrame::Ip(p)` at the call site instead of the direct return. |
| `src/mitre.rs` / VP-007 | CRITICAL | The `vp007_catalog_drift_guard` test WILL fail if new MITRE IDs are added to `technique_info` without updating `SEEDED_TECHNIQUE_IDS` + count. This is the same mechanical tripwire that fired in Modbus and DNP3 cycles. |
| `tests/integration_*` (e2e pcap tests) | MEDIUM | If any integration test feeds a raw packet directly to `decode_packet` and pattern-matches the return value, it must be updated to match on `DecodedFrame::Ip`. |

---

## 5. BC-2.02.009 Revision Required (Non-Trivial Spec Impact)

**BC-2.02.009 current text:** "Surface No IP Layer Found Error for Non-IP Frames"

**Problem:** After the migration, ARP frames produce `DecodedFrame::Arp(...)`, not
`Err("No IP layer found")`. The BC-2.02.009 postcondition that an ARP frame yields an
error is no longer true.

**Required revision:** BC-2.02.009 must be revised to:
"Non-IP frames that are not ARP return `Err("No IP layer found")`. ARP frames
(EtherType 0x0806) return `Ok(DecodedFrame::Arp(...))`."

This is a **BC modification** in ss-02, not a new BC. The F2 spec-evolution phase
must produce this revision. The story that implements the decoder migration must
reference the revised BC-2.02.009.

Additionally, the doc comment on `src/decoder.rs` line 1 ("Non-IP frames (e.g. ARP).
Lax parsing cannot conjure an IP layer that is not present, so reject directly.") must
be updated to reflect the new ARP routing.

---

## 6. Verification Property: VP-024 (Proposed)

**Title:** "ARP Frame Parse Safety and Binding-Table Invariant"

**Module:** `src/analyzer/arp.rs`

**Phase:** P1 (consistent with VP-022/VP-023 for Modbus/DNP3)

**Tool:** Kani (primary) + proptest (binding-table properties)

**Sub-properties (draft):**
- Sub-A: ARP frame extraction from `ArpPacketSlice` never panics on any valid
  `ArpPacketSlice` input. Kani harness: symbolic hardware/protocol address fields,
  assert no panic on `extract_arp_frame`.
- Sub-B: GARP detection is total — for any `ArpFrame` where `sender_ip == target_ip`
  and operation is `Reply`, the analyzer classifies it as gratuitous. Kani harness:
  symbolic ARP frame with symbolic IPs, assert classification correctness.
- Sub-C: Binding-table invariant — `ArpAnalyzer.bindings` maps each `Ipv4Addr` to
  exactly one `[u8; 6]` at any point in time. proptest: sequence of arbitrary
  `ArpFrame` values, assert no duplicate IP entries with different MACs exist
  simultaneously (deterministic last-write-wins update). This is the foundational
  correctness property for spoof detection.
- Sub-D: MAX_ARP_BINDINGS cap — the binding table never exceeds the bounded size.
  proptest or Kani: symbolic sequence of frames with distinct IPs, assert table size
  ≤ MAX_ARP_BINDINGS.

**Feasibility:** HIGH. All sub-properties operate on small bounded inputs. Sub-A/B
are straightforward Kani proofs. Sub-C/D are proptest sequences with simple
invariant checks. Analogous to VP-022/VP-023 which both ran successfully.

**Verified BCs (draft):** BC-2.16.001 (ARP request parse), BC-2.16.002 (ARP reply
parse), BC-2.16.003 (GARP detection), BC-2.16.004 (ARP spoofing detection via binding
table), BC-2.16.005 (binding table bounded resource). Exact BC numbers assigned in F2.

---

## 7. MITRE Delta (TBD-Pending-Research)

### Current State (post v0.6.0, STORY-110)

- Seeded IDs: 23 (11 Enterprise + 12 ICS)
- Emitted IDs: 15 (6 Enterprise + 9 ICS)
- `SEEDED_TECHNIQUE_ID_COUNT`: 23
- VP-007 status: verified/locked

### ARP MITRE Requirements (PRELIMINARY — research agent must validate all IDs)

| Technique | Proposed (unvalidated) | Status | Action Required |
|-----------|----------------------|--------|----------------|
| T0830 (ARP Spoofing — ICS) | "Adversary-in-the-Middle" | NOT CONFIRMED | Research agent must validate T0830 exists in ICS ATT&CK v19.1 and confirm tactic |
| T1557.002 (ARP Cache Poisoning — Enterprise) | "Adversary-in-the-Middle: ARP Cache Poisoning" | NOT CONFIRMED | Research agent must validate T1557.002 exists in Enterprise ATT&CK and confirm tactic |

**CRITICAL: DF-VALIDATION-001 applies.** No BC in ss-16 may reference a MITRE technique
ID that has not been validated by the research agent against the pinned ATT&CK versions.
The Modbus cycle's T0846→T0888 correction (D-032 Decision-12) and the DNP3 cycle's
T0803-revoked → T1691.001 correction (ADR-007 Decision 5) demonstrate that MITRE IDs
require external validation. Do not write BCs with placeholder IDs.

**VP-007 atomic update obligation** (when IDs are confirmed):
Same 5-part pattern as Modbus/DNP3:
1. `technique_info` match arm(s) — one per confirmed ID
2. `SEEDED_TECHNIQUE_IDS` array — add confirmed IDs
3. `SEEDED_TECHNIQUE_ID_COUNT` constant — bump by number of new IDs
4. `EMITTED_IDS` in `kani_proofs` module — add IDs that ArpAnalyzer will emit
5. `cargo test mitre` — must pass before PR merges

**MitreTactic enum assessment:** ARP-related techniques may map to existing tactic
variants (`LateralMovement`, `CredentialAccess`, `IcsImpairProcessControl`) or may
require a new variant. Research agent must confirm tactic assignments. If a new
`MitreTactic` variant is required, it must be added to `all_tactics_in_report_order()`
and its `fmt::Display` arm — the same obligation as `IcsImpact` in ADR-007 Decision 5.

---

## 8. Regression Risk Assessment

| Module | Risk Level | Rationale |
|--------|-----------|-----------|
| `src/decoder.rs` | HIGH | Return type changes from `Result<ParsedPacket>` to `Result<DecodedFrame>`. Two match sites (strict_ip_triple, lax_ip_triple) become non-exhaustive. SliceError::Len replacement is the highest-risk compile break. |
| `src/mitre.rs` | CRITICAL | VP-007 drift guard will mechanically fail if atomic update obligation is not met. Same risk as Modbus/DNP3. |
| `src/main.rs` | MEDIUM | Packet loop call site must pattern-match on DecodedFrame. Risk of accidentally dropping Arp variant or duplicating the Ip variant route. |
| `src/cli.rs` | LOW | Additive only. |
| `src/analyzer/arp.rs` (new) | LOW | New module, no existing tests to break. |
| All existing SS-02 BCs | MEDIUM | BC-2.02.009 requires text revision. No other SS-02 BCs are semantically changed. |
| VP-008 (fuzz) | MEDIUM | Fuzz harness must be updated for DecodedFrame return type. No-panic property still holds. |
| Dispatcher/reassembly/HTTP/TLS/DNS/Modbus/DNP3 | NONE | ARP frames are routed before the reassembler. IP pipeline is structurally unchanged. |

**Regression baseline:** 63 stories completed (STORY-001..STORY-110, all waves 1-39
closed). All existing tests must stay green. The two decoder contract tests
(`test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`,
`test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered`) are the
oracle for the SliceError::Len migration correctness.

---

## 9. DTU Assessment Delta

**DTU_REQUIRED: false (confirmed, no change)**

The ARP security analyzer:
- Has no external service dependencies.
- Processes ARP frames from existing local PCAP files.
- Binding table is purely in-memory state.
- MITRE technique lookups use the existing local `technique_info` function.
- No network I/O, no external API calls, no webhooks, no cloud services.

The `dtu-assessment.md` rationale is unchanged. The ARP feature does not introduce any
external service boundary that would require a Digital Twin Universe clone. Update
`dtu-assessment.md` version field to reflect the ARP review confirmation.

---

## 10. New Artifact Estimate

| Artifact Type | Count | Basis |
|--------------|-------|-------|
| New BCs (SS-16) | 18–24 | Modbus: 25 BCs; DNP3: 24 BCs. ARP is simpler (no carry buffer, no CRC, no transaction correlation). Estimated: 18–24 BCs covering parse correctness, GARP detection, ARP spoof detection, binding-table bounds, CLI integration, dispatcher bypass, summary output. |
| Revised BCs (SS-02) | 1 | BC-2.02.009 text revision |
| New VPs | 1 | VP-024 (Kani + proptest, P1) |
| New ADRs | 1 | ADR-008 (ARP link-layer integration) |
| New stories | 5–6 | See §11 story breakdown |
| New holdout scenarios | 3–5 | GARP detection, ARP spoofing, benign-ARP baseline, poisoning with prior binding, MAC reuse |

---

## 11. Recommended F2→F7 Phase Plan

### Dependency Order

The migration sub-delta (etherparse 0.16→0.20 + DecodedFrame) is a **strict dependency**
of the ARP analyzer. A story that introduces `src/analyzer/arp.rs` cannot land until
`decode_packet` returns `DecodedFrame` and `ArpFrame` is available.

### Proposed Story Breakdown (F3 estimation)

| Story | Title | Epic | Dependencies |
|-------|-------|------|-------------|
| STORY-111 | etherparse 0.20 Migration — DecodedFrame Enum + Break Repairs | E-16 | STORY-110 |
| STORY-112 | ArpFrame Extraction from NetSlice::Arp + decode_packet Routing | E-16 | STORY-111 |
| STORY-113 | ArpAnalyzer: Binding Table, GARP Detection, Frame Statistics | E-16 | STORY-112 |
| STORY-114 | ArpAnalyzer: ARP Spoof/Poisoning Detection + MITRE Emission (VP-024 Kani) | E-16 | STORY-113 |
| STORY-115 | ARP CLI Integration (--arp flag), Summary, VP-007 MITRE Atomic Update | E-16 | STORY-114 |
| STORY-116 (optional) | ARP Rate/Storm Anomaly Detection (if human approves threshold feature) | E-16 | STORY-113 |

**Wave assignment:** Stories 111–115 form a strict linear dependency chain.
STORY-116 depends on STORY-113 but not STORY-114, so it could run in parallel with
STORY-114 if approved.

### F2→F7 Phase Plan

| Phase | Scope |
|-------|-------|
| F2 | Spec evolution: BC-2.16.NNN (18–24 BCs), VP-024 spec, ADR-008. Research agent: (a) confirm MITRE IDs T0830/T1557.002 against ICS/Enterprise ATT&CK current versions; (b) confirm ArpPacketSlice exact method names in etherparse 0.20 docs; (c) confirm SliceError::Len replacement pattern in 0.20 error hierarchy. Revise BC-2.02.009. Adversarial spec review (cross-model per DF-ADVERSARY-TOOLCHAIN-PAIRING-001 policy). |
| F3 | Story decomposition: 5–6 stories (§11), wave schedule, dependency graph update. Sub-epic E-16. |
| F4 | TDD implementation per story. STORY-111 first (migration prerequisite). Per-story convergence with adversarial round. Special attention: (a) two decoder contract tests re-green under etherparse 0.20; (b) VP-008 fuzz harness updated for DecodedFrame. |
| F5 | Combined-delta adversarial review of full ARP analyzer + etherparse migration. Focus on: (a) ARP spoof false-positive/negative balance; (b) binding table LRU eviction correctness; (c) GARP vs poisoning classification boundary. |
| F6 | Kani VP-024 (sub-A/B — parse safety, GARP totality), proptest (sub-C — binding table invariant, sub-D — cap). Re-run VP-008 cargo-fuzz against updated decode_packet. Run mutation testing on ArpAnalyzer classifier. cargo audit/deny clean under etherparse 0.20. |
| F7 | Convergence: holdout evaluation (crafted ARP PCAP fixture with known GARP and poisoning events), e2e acceptance test, consistency audit (ARCH-INDEX.md, VP-INDEX.md, module-criticality.md all updated). |

**Release target:** v0.7.0 (following the v0.4.0 Modbus / v0.6.0 DNP3 cadence).

---

## 12. Architecture Decision Points Requiring Human Gate Confirmation

The following decisions must be confirmed before F2 spec work begins.

### DECISION 1 — DecodedFrame integration pattern

**Recommendation:** Option 1 — `DecodedFrame` enum returned by `decode_packet`.
Alternatives: Option 2 (pre-decode EtherType inspection in main.rs) or Option 3
(ProtocolAnalyzer packet-level path).

**Human must confirm:** Option 1 is the recommended approach. Confirm or override.

### DECISION 2 — ARP spoof detection confidence levels

**Context:** ARP binding table conflict (IP seen with new MAC) could be a legitimate
event (host NIC replacement, DHCP lease change, VM migration) or a genuine attack.

**Options:**
1. Always emit HIGH confidence (maximize detection, accept false positives).
2. Emit MEDIUM confidence for first re-binding, HIGH after N re-bindings in a time window.
3. Emit LOW/MEDIUM only (ARP rebinding is too common to rate as HIGH without
   additional context).

**Recommendation:** Option 2. First rebinding emits MEDIUM/Anomaly. Rapid repeated
rebinding (e.g., 3+ times in 60s) escalates to HIGH/Likely. Analogous to DNP3
DIRECT_OPERATE threshold pattern. Threshold configurable via `--arp-spoof-threshold`.

### DECISION 3 — GARP treatment

**Options:**
1. Always emit a finding for GARP (sender IP == target IP). Low/Inconclusive.
2. Emit only if the GARP conflicts with an existing binding.
3. Do not emit for GARP at all; track only.

**Recommendation:** Option 1 with low confidence. GARP is a common legitimate
operation on healthy networks, so over-emission is a real risk. A low/inconclusive
finding allows analysts to filter. The binding table update proceeds normally.

### DECISION 4 — MITRE technique scope

**Pending research agent validation.** Human must confirm the final set of technique
IDs after research agent produces validated mappings. Options include:
- Enterprise only (T1557.002 ARP Cache Poisoning)
- ICS only (T0830 if it exists in ICS ATT&CK)
- Both (ARP spoofing is relevant in both OT and IT contexts)
- Additional techniques if research agent identifies relevant mappings

### DECISION 5 — Binding table capacity (MAX_ARP_BINDINGS)

**Context:** Modbus uses MAX_PENDING_TRANSACTIONS=256; DNP3 uses MAX_MASTER_ADDRS=64.
An ARP binding table tracks IP-MAC mappings for the entire capture. A large enterprise
capture could observe thousands of hosts.

**Recommendation:** MAX_ARP_BINDINGS = 65,536 (same order of magnitude as IPv4
subnets, sufficient for enterprise captures, bounded to prevent unbounded growth).
LRU eviction on overflow. Exact value is an F2 decision based on memory impact analysis.

---

## 13. Consistency with Prior Cycle Precedents

| Precedent | ARP Alignment |
|-----------|--------------|
| D-032: StreamHandler integration (Modbus) | ARP does NOT use StreamHandler — it is link-layer, not TCP flow. New integration pattern via DecodedFrame. |
| ADR-007 Decision 1: port-fallback classification | ARP does NOT use port-fallback or StreamDispatcher — it is classified at the decoder level by EtherType, before TCP reassembly. This requires ADR-008 to document the link-layer decode path as the integration point. |
| D-032: VP-007 atomic update obligation | Same 5-part obligation for confirmed ARP MITRE IDs. |
| D-043: F5 combined-delta adversarial review | ARP must have F5 adversarial focus on binding table false-positive/negative rates. |
| DNP3 ADR-007 Decision 2: manual binary parsing | ARP uses etherparse's ArpPacketSlice directly — no manual binary parsing needed (etherparse 0.20 provides it). This is simpler than Modbus/DNP3. |
| DF-VALIDATION-001 (policies.yaml) | MITRE IDs T0830/T1557.002 are unvalidated placeholders. Research agent validation required before F2 BCs reference them. |
