---
document_type: architecture-delta
feature: arp-security-analyzer
version: "1.10"
status: draft
producer: architect
timestamp: 2026-06-12T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/verification-properties/VP-INDEX.md
---

# Architecture Delta — ARP Security Analyzer (v0.7.0)

This document enumerates the concrete architecture changes introduced by the ARP Security
Analyzer feature (SS-16). It is the single authoritative reference for the decode-path
changes, new data structures, documented thresholds, and regression surface. Product-owner
should read this before authoring BC-2.16.NNN behavioral contracts.

---

## 1. New Source Module: `src/analyzer/arp.rs` (C-23, SS-16)

| Attribute | Value |
|-----------|-------|
| Component ID | C-23 (new; follows C-22 ModbusAnalyzer) |
| Subsystem | SS-16 ARP Security Analysis |
| Purity | Pure core (no I/O, no file access, no network; takes ArpFrame by value, returns Vec<Finding>) |
| Role | ArpAnalyzer struct: binding table, GARP detection, spoof detection, storm rate, D11 malformed, D12 mismatch |
| ADR reference | ADR-008 Decisions 4–5 |

The module exposes:
- `pub struct ArpAnalyzer` — the stateful analyzer instance created once per `run_analyze()` call
- `impl ArpAnalyzer { pub fn process_arp(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding> }` — called from `main.rs` for every `DecodedFrame::Arp`
- `impl ArpAnalyzer { pub fn summarize(&self) -> AnalysisSummary }` — called at end of capture
- `fn is_gratuitous_arp(frame: &ArpFrame) -> bool` — free pure-core function; VP-024 Sub-B target
- `fn insert_binding_lru(bindings: &mut HashMap<[u8;4], BindingEntry>, ip: [u8;4], mac: [u8;6], cap: usize)` — free pure-core function; VP-024 Sub-D target (production type; Kani Sub-D proof uses BTreeMap surrogate per ADR-008 Decision 4)

`ArpAnalyzer` does NOT implement `ProtocolAnalyzer` (which requires `&ParsedPacket`) or
`StreamAnalyzer` (which requires reassembled TCP bytes). It receives `&ArpFrame` directly
from `main.rs` after `decode_packet` returns `DecodedFrame::Arp`.

---

## 2. Decode-Path Changes

### 2.1 New types in `src/decoder.rs`

**`DecodedFrame` enum** (new public type):
```rust
pub enum DecodedFrame {
    Ip(ParsedPacket),
    Arp(ArpFrame),
}
```

**`ArpFrame` struct** (new public type):
```rust
pub struct ArpFrame {
    pub operation:     u16,        // 1=Request, 2=Reply
    pub sender_mac:    [u8; 6],
    pub sender_ip:     [u8; 4],
    pub target_mac:    [u8; 6],
    pub target_ip:     [u8; 4],
    pub outer_src_mac: Option<[u8; 6]>,  // Ethernet frame src MAC, for D12
    pub packet_len:    usize,
}
```

**`decode_packet` signature change**:
```
Before: pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>
After:  pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<DecodedFrame>
```

`ParsedPacket` is unchanged. All existing code that receives a `ParsedPacket` is unaffected
except the single call site in `main.rs`.

### 2.2 Match site additions in `src/decoder.rs`

`strict_ip_triple` gains one new arm that uses `unreachable!` — this is correct because
the `Ok(slice)` strict path routes `NetSlice::Arp` out before `strict_ip_triple` is
ever called:

```rust
// strict_ip_triple (NetSlice) — compile-safety arm; never reached at runtime:
NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple"),
```

`lax_ip_triple` gains one new arm that **must NOT use `unreachable!`**. The lax decode
path is taken for `Err(SliceError::Len(_))` (snaplen-truncated packets). In etherparse
0.20, a truncated ARP frame yields `Some(LaxNetSlice::Arp(_))` from the lax parser,
which then reaches `lax_ip_triple`. An `unreachable!` here would be a **reachable
runtime panic**, violating VP-008 (decode_packet no-panic) and VP-024 Sub-A.

ARP early-extraction is therefore specified in **both** the strict `Ok` arm and the lax
`Err(SliceError::Len(_))` arm of `decode_packet`. The lax arm mirrors the strict arm's
`outer_src_mac` extraction from `lax.link` and routes to `extract_arp_frame`:

```rust
// lax_ip_triple (LaxNetSlice) — explicit ARP routing arm; NOT unreachable!:
// This arm is reached for snaplen-truncated ARP frames.
// Return a sentinel / early-exit so the caller routes to extract_arp_frame.
// See decode_packet lax arm spec in ADR-008 Decision 3 (v1.6).
LaxNetSlice::Arp(arp) => /* route to extract_arp_frame — see ADR-008 Decision 3 */,
```

**decode_packet lax arm (complete spec — authoritative in ADR-008 Decision 3 v1.6):**
```rust
Err(SliceError::Len(_)) => {
    let lax = lax_parse(...);
    match &lax.net {
        Some(LaxNetSlice::Arp(arp)) => {
            // Truncated ARP: same extraction path as strict arm.
            // outer_src_mac extracted from lax.link (Ethernet2Slice::source()
            // returns [u8; 6] by value — confirmed docs.rs 0.20.1, 2026-06-12).
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

**VP-008 / VP-024 Sub-A guarantee:** No reachable `unreachable!` remains in the decode
path. All `LaxNetSlice::Arp` inputs are routed to `extract_arp_frame`, which is
panic-free by Sub-A postcondition. `extract_arp_frame` returns `None` on a truncated
or malformed body; `None` maps to `Err(...)`, not a panic.

### 2.3 etherparse 0.20.1 migration: `Cargo.toml` and `SlicedPacket.link_exts`

`Cargo.toml` change: `etherparse = "0.16"` → `etherparse = "0.20"`.

`SlicedPacket.vlan` has been renamed to `SlicedPacket.link_exts` in 0.20.1 (confirmed from
live docs.rs fetch, 2026-06-12). The field type is now
`ArrayVec<LinkExtSlice, LINK_EXTS_CAP>` rather than a single VLAN option. Wirerust's current
decoder code does NOT access `.vlan` on `SlicedPacket`, so this rename produces no compile
error in the existing code. If any new code (ARP tests or integration fixtures) accesses VLAN
fields, use `.link_exts` instead of `.vlan`.

**`SliceError::Len` status (CONFIRMED):** `SliceError::Len(_)` is still present and unchanged
in etherparse 0.20.1. The truncation fallback in `decoder.rs` (`Err(SliceError::Len(_)) => ...`)
requires NO change. The existing `use etherparse::err::packet::SliceError;` import and the
match arm compile unchanged under 0.20.

---

## 3. Binding-Table Data Structure and Documented Default Thresholds

### 3.1 BindingEntry and ArpAnalyzer state

```
ArpAnalyzer.bindings: HashMap<[u8; 4], BindingEntry>  — IP → binding (production type)
ArpAnalyzer.storm_counters: HashMap<[u8; 6], StormCounter>  — MAC → storm window
```

`BindingEntry` tracks: `mac: [u8; 6]`, `rebind_count: u32`, `first_rebind_ts: Option<u32>`,
`spoof_high_emitted: bool`, `last_seen_ts: u32` (used for LRU eviction — the entry with
the smallest `last_seen_ts` is evicted when the table reaches `MAX_ARP_BINDINGS`).

**LRU eviction mechanism:** `insert_binding_lru` scans all entries for the minimum
`last_seen_ts` when `bindings.len() >= cap` and removes that entry before inserting the
new one. This is O(N) on overflow only; N ≤ MAX_ARP_BINDINGS = 65,536. Accepted cost.

**Kani proof surrogate (VP-024 Sub-D):** `HashMap` with `RandomState` is incompatible
with Kani (platform RNG FFI). The Sub-D harness calls `insert_binding_lru_btree` — a
`#[cfg(any(kani, test))]`-gated wrapper over the same eviction logic parameterized on
`BTreeMap<[u8;4], BindingEntry>`. The cap invariant is arithmetic and holds for both map
types. Production type remains `HashMap`.

`StormCounter` tracks: `count_in_window: u64`, `window_start_ts: u32`, `storm_emitted: bool`.

### 3.2 Threshold constants — wirerust engineering choices

All numeric thresholds below are wirerust-chosen engineering defaults. The research in
`mitre-arp-additional-detections.md` §4 confirms that **no authoritative industry-published
numeric defaults exist** in Snort, arpwatch, Zeek, or any ICS-vendor product for these
parameters. Product-owner MUST document these values in BCs using the phrase "wirerust
engineering default" and MUST NOT cite them as borrowed from any external standard.

| Constant | Value | Description | Override mechanism |
|----------|-------|-------------|-------------------|
| `MAX_ARP_BINDINGS` | 65,536 | Maximum IP→MAC entries in binding table; LRU eviction on overflow. Sized for one /16 IPv4 subnet. | Constant only (no CLI flag) |
| `MAX_STORM_COUNTERS` | 4,096 | Maximum per-MAC storm counter entries tracked simultaneously. | Constant only |
| `SPOOF_REBIND_ESCALATION_DEFAULT` | 3 | rebind_count threshold for escalating from MEDIUM to HIGH confidence on a spoof finding. First rebind = MEDIUM/Anomaly; rebind_count >= 3 within ARP_FLAP_WINDOW_SECS = HIGH/Likely. | `--arp-spoof-threshold` CLI flag |
| `ARP_FLAP_WINDOW_SECS` | 60 | Time window in seconds for spoof escalation (flap-window). | Constant only in v0.7.0; expose as CLI flag if needed in fast-follow |
| `ARP_STORM_RATE_DEFAULT` | 50 | Average-frames-per-second-since-window-start threshold for ARP storm detection (D3) (per BC-2.16.008 Invariant 2). OT operators should lower this. | `--arp-storm-rate` CLI flag |

### 3.3 Detection-to-finding mapping

| Detection | Trigger condition | Confidence | MITRE techniques | Stateful? |
|-----------|-----------------|------------|-----------------|-----------|
| D1 ARP spoof | Binding table: IP seen with new MAC | MEDIUM (first rebind) → HIGH (≥3 rebinds / 60s) | T0830, T1557.002 | YES |
| D2 GARP | sender_ip == target_ip (any operation) | LOW base; MEDIUM when GARP also conflicts with existing binding (D2+D1 interaction) | T0830, T1557.002 | NO |
| D3 Storm | Source MAC average-frames-per-second-since-window-start exceeds ARP_STORM_RATE_DEFAULT (per BC-2.16.008 Invariant 2) | MEDIUM | None (T0814 deferred pending DF-VALIDATION-001 live check) | YES |
| D11 Malformed | extract_arp_frame returns None for non-Eth/IPv4 field sizes, or hw/proto type mismatch | LOW | None | NO |
| D12 L2/L3 mismatch | outer_src_mac != sender_mac (when outer_src_mac is Some) | MEDIUM | T0830, T1557.002 | NO |

**GARP escalation rule:** if D2 (GARP) also triggers D1 (binding conflict — the GARP claims an
IP already bound to a different MAC), the GARP finding is upgraded to MEDIUM and the D1 spoof
finding is also emitted. The two findings may be emitted on the same frame for the same IP.

---

## 4. Regression Surface

### 4.1 Changes requiring implementation updates

| Component | Change type | Risk | Notes |
|-----------|-------------|------|-------|
| `src/decoder.rs` | MODIFIED (significant) | HIGH — CRITICAL PATH | Add `DecodedFrame` enum, `ArpFrame` struct, `extract_arp_frame` fn, `NetSlice::Arp` / `LaxNetSlice::Arp` match arms, update `decode_packet` return type, update decoder module-doc comment (top-of-file `//!` doc, src lines ~1-10) AND the `SliceError` import comment block (src lines ~42-48, which still reference "etherparse 0.16 API contract" / "0.17 bump") — both prose blocks must be updated to reference 0.20 in the same prose-sweep pass |
| `Cargo.toml` | MODIFIED | LOW | `etherparse = "0.16"` → `etherparse = "0.20"`. Bump the version pin comment on ~lines 21–26 to reference 0.20 API contract |
| `src/main.rs` | MODIFIED | MEDIUM | Pattern-match on `DecodedFrame`; wire `ArpAnalyzer::process_arp` on `Arp` variant; add `--arp` flag wiring; push summary to `analyzer_summaries` |
| `src/cli.rs` | MODIFIED | LOW | Add `#[arg(long)] arp: bool`, `--arp-spoof-threshold: u32`, `--arp-storm-rate: u32` to `Commands::Analyze` |
| `src/analyzer/mod.rs` | MODIFIED | LOW | Add `pub mod arp;` |
| `src/mitre.rs` | MODIFIED (CRITICAL) | CRITICAL | VP-007 5-part atomic update: add T0830 + T1557.002 arms; SEEDED 23→25; EMITTED +2; `cargo test mitre` green |
| `src/analyzer/arp.rs` | NEW | LOW | New module; no existing tests to break |

### 4.2 BC revision required

**BC-2.02.009** "Surface No IP Layer Found Error for Non-IP Frames" must be revised.

Previous postcondition: ARP frames return `Err("No IP layer found")`.

Revised postcondition: ARP frames (EtherType 0x0806) with Ethernet/IPv4 format return
`Ok(DecodedFrame::Arp(...))`. Non-Ethernet/IPv4 ARP frames return
`Err("Non-Ethernet/IPv4 ARP frame")`. Non-IP, non-ARP frames continue to return
`Err("No IP layer found")`.

The test `test_decode_non_ip_frame_returns_error` (or equivalent) for BC-2.02.009 must
be updated to reflect this three-way postcondition.

### 4.3 VP obligations created or updated

| VP | Status | Change |
|----|--------|--------|
| VP-008 (decode_packet no-panic, cargo-fuzz) | EXISTING — MUST UPDATE | Fuzz harness return type changes from `Result<ParsedPacket>` to `Result<DecodedFrame>`; no-panic invariant unchanged; both `Ip` and `Arp` variants are non-panic outcomes |
| VP-024 (ARP parse safety + binding-table invariant) | NEW — draft | 4 sub-properties: Kani (A/B/D) + proptest (C); P1 |

### 4.4 Components with NO regression risk

- `src/dispatcher.rs` / VP-004: ARP frames are routed before the reassembler; `StreamDispatcher` is not reached
- `src/reassembly/` (all files): reassembler operates on `ParsedPacket`; `DecodedFrame::Ip(p)` still delivers `ParsedPacket` unchanged
- All HTTP / TLS / DNS / Modbus / DNP3 tests: IP pipeline is structurally identical; call sites change only at the outer `match` in `main.rs`
- `src/findings.rs`, all reporters: `Finding` struct and reporter pipeline are unchanged

### 4.5 Two decoder contract tests (oracle for migration correctness)

Per the comment in `src/decoder.rs` lines 44–48, these two tests are the contract oracle
for the `SliceError::Len` truncation fallback:
- `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`
- `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered`

Both tests must remain green after the etherparse 0.20 migration. Since `SliceError::Len` is
confirmed present in 0.20.1, these tests should compile and pass without modification.
Running them is the FIRST verification step after `Cargo.toml` bump.

---

## 5. mitre.rs VP-007 Atomic Update — Detail

### 5.0 Forward-Declaration Convention (normative)

**BC-2.10.005 and BC-2.10.008 are FORWARD-DECLARED to the post-STORY-114 state
(SEEDED=25, EMITTED=17).** `src/mitre.rs` remains at 23/15 until STORY-114 lands the
5-part atomic update; the `vp007_catalog_drift_guard` test enforces consistency at
implementation time. Both catalogue BCs MUST carry a uniform marker:

> `PLANNED — implemented in STORY-114; current code 23/15`

This marker must appear in both BC-2.10.005 and BC-2.10.008 until STORY-114 merges.
The marker is the product-owner's responsibility (see PO hand-off below).

**STORY-114 F4 obligation (explicit — do not miss):** The 5-part atomic update covers
four functional code edits and a set of stale-comment updates. All sites must be updated
in a single commit; `vp007_catalog_drift_guard` will mechanically fail if any functional
site is missed. Do NOT touch src/mitre.rs before STORY-114.

**Verified line numbers from src/mitre.rs as of pre-ARP baseline (sibling-sweep
DF-SIBLING-SWEEP-001 confirmed 2026-06-12).**

**(a) Functional code edits required for vp007_catalog_drift_guard to pass:**

| Site | Verified location | Change |
|------|-------------------|--------|
| `technique_info` match arms | src/mitre.rs lines 178-179 (last arm: T0827 `=>` at line 178; `_ => return None` at line 179) | Add two new arms before `_ => return None`: `"T0830" => ("Adversary-in-the-Middle", MitreTactic::LateralMovement)` and `"T1557.002" => ("Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess)` |
| `SEEDED_TECHNIQUE_IDS` array body | src/mitre.rs lines 305-333 (array literal; `"T0827"` is the last entry at line 332) | Add `"T0830"` and `"T1557.002"` as new entries |
| `SEEDED_TECHNIQUE_ID_COUNT` constant | src/mitre.rs line 341 | 23 → 25 |
| `EMITTED_IDS` array in `kani_proofs` | src/mitre.rs lines 221-240 (array literal starts line 221, closing `];` at line 240) | Add `"T0830"` and `"T1557.002"` entries (both will be emitted by ArpAnalyzer D1/D2/D12) |

**(b) Stale-count comment updates (do not affect test pass/fail, but must be correct
for future readers and for vp007_catalog_drift_guard narrative accuracy):**

| Site | Verified location | Change |
|------|-------------------|--------|
| `kani_proofs` module doc — seeded-ID count | src/mitre.rs line 204 ("finite (23)") | 23 → 25 |
| `kani_proofs` `SEEDED_IDS` const comment | src/mitre.rs line 212 ("All 23 seeded IDs") | 23 → 25 |
| `kani_proofs` `EMITTED_IDS` const comment | src/mitre.rs line 218 ("6 Enterprise + 7 ICS + 2 STORY-109 = 15 emitted IDs") | Update to "6 Enterprise + 7 ICS + 2 STORY-109 + 2 ARP (STORY-114) = 17 emitted IDs" |
| `SEEDED_TECHNIQUE_IDS` doc comment — history line (Post-F2) | src/mitre.rs line 301 ("Post-F2 (STORY-100): 11 Enterprise + 10 ICS = 21 total") | Correct stale split first: the actual post-F2/STORY-100 base before STORY-109 is 11 Enterprise + 10 ICS = 21; then STORY-109 added +2 ICS (T1691.001, T0827) → 12 ICS = 23 total. Line 301 incorrectly states "11 Enterprise + 10 ICS = 21 total" as the full post-F2 count — this is the pre-STORY-109 subtotal. Correct to: "Post-F2 (STORY-100): 11 Enterprise + 10 ICS = 21 total (pre-STORY-109 subtotal)" |
| `SEEDED_TECHNIQUE_IDS` doc comment — STORY-109 line | src/mitre.rs line 302 ("STORY-109 (VP-007 atomic obligation): +2 ICS (T1691.001, T0827) = 23 total") | Add ARP addendum: "+ 2 ARP (STORY-114): T0830 (ICS LateralMovement) + T1557.002 (Enterprise CredentialAccess) = 25 total" |
| `SEEDED_TECHNIQUE_ID_COUNT` doc comment | src/mitre.rs line 339 ("currently 23: 21 post-F2/STORY-100 + 2 STORY-109 additions") | Update to "currently 25: 21 post-F2/STORY-100 + 2 STORY-109 + 2 ARP/STORY-114 additions" |

All five functional sites and all stale-comment sites must be updated in one commit.

**Process-gap record (F-A03-LOW-02 / Slice C — S-7.02 candidates):**
- No automated "ADR struct-literal ↔ prose-mandated-field-set" coherence check exists.
  The missing seven counter fields in the Decision 4 struct literal were caught by
  adversarial review, not tooling. Candidate for S-7.02 codification.
- No engine convention for forward-declared BC postconditions (BCs that describe
  post-implementation state while code is at an older state). This convention is now
  established here as a normative pattern. Candidate for S-7.02 codification.

**Pre-existing brownfield debt record — F4 obligation (do NOT touch src/mitre.rs before STORY-114):**

| Item | Location | Contradiction | Recommended resolution | Story |
|------|----------|---------------|------------------------|-------|
| `IcsImpact` Display string | `src/mitre.rs:91` — `MitreTactic::IcsImpact => "Impact (ICS)"` | BC-2.10.002 PC3 canonical Display is `"Impact"` (no matrix suffix); PRD §85/823 and the merge-by-name design both specify `"Impact"`. The `" (ICS)"` suffix causes the merge-by-name grouping to produce a separate `"Impact (ICS)"` bucket rather than merging with the `"Impact"` tactic name from other findings, breaking the merge-by-name invariant. | Change `src/mitre.rs:91` from `"Impact (ICS)"` to `"Impact"` to match BC-2.10.002 PC3 and the merge-by-name policy. If this would silently drop a user-visible matrix label, discuss with PO before STORY-114; a separate brownfield-debt story is acceptable if out of ARP scope. | STORY-114 (adjudicate); or separate brownfield story if out of ARP scope |

This is a **pre-existing src-vs-spec contradiction, NOT introduced by the ARP analyzer**.
F2 is spec-only; `src/mitre.rs` is not touched in F2. STORY-114 implementer must
adjudicate: change source to `"Impact"`, OR record as a separate post-ARP debt story
with explicit BC-2.10.002 PC3 exemption. Do not silently retain `"Impact (ICS)"` in
code after STORY-114 merges without a documented decision.

After the ARP feature is merged (STORY-115), `src/mitre.rs` must contain:

```rust
// New arms in technique_info():
"T0830" => ("Adversary-in-the-Middle", MitreTactic::LateralMovement),
"T1557.002" => ("Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess),

// SEEDED_TECHNIQUE_IDS: add "T0830", "T1557.002"
// SEEDED_TECHNIQUE_ID_COUNT: 23 → 25
// EMITTED_IDS in kani_proofs: add "T0830", "T1557.002"
```

**MitreTactic decision (resolved — ADR-008 Decision 6):**

- T0830 (ICS Lateral Movement, TA0109) → `MitreTactic::LateralMovement`. The `mitre.rs`
  merge-by-name policy (confirmed from source, lines 145–148) maps ICS tactics to their
  Enterprise equivalent when the tactic name matches. ICS "Lateral Movement" (TA0109) merges
  with Enterprise "Lateral Movement" (TA0008) — the `LateralMovement` variant already exists.
  No new enum variant is required.
- T1557.002 (Enterprise Credential Access, TA0006) → `MitreTactic::CredentialAccess`.
  The `CredentialAccess` variant already exists in the enum. Confirmed: T1557 ("Adversary-in-
  the-Middle") has tactic "Credential Access" in Enterprise ATT&CK v19.1 — consistent with
  `T1040` ("Network Sniffing") already mapped to `MitreTactic::CredentialAccess` in mitre.rs.

The F3 implementer adds ONLY the `technique_info` match arms and VP-007 5-part atomic update.
No `MitreTactic` enum change is needed.

---

## 6. Canonical Story Decomposition (authoritative — F3 consumes this table)

This is the single authoritative story→BC/detection/VP decomposition. HS-INDEX waves 40–44
MUST be rewritten by product-owner to match this table exactly (see product-owner hand-off
below). The dependency chain runs STORY-111 → 112 → 113 → 114 → 115 strictly; no story
may begin until its predecessor's PR has merged.

| Story | Scope | BCs covered | VPs touched | Dependencies |
|-------|-------|-------------|-------------|--------------|
| STORY-111 | etherparse 0.20 migration (`Cargo.toml` bump); `DecodedFrame` enum + `ArpFrame` struct (with `outer_src_mac`) in `src/decoder.rs`; `strict_ip_triple` `NetSlice::Arp` unreachable arm (compile-safety only — strict path routes ARP out before this function is called); `lax_ip_triple` `LaxNetSlice::Arp` explicit-routing arm (NOT unreachable! — truncated ARP frames reach lax_ip_triple via the Err(SliceError::Len) path; see ADR-008 Decision 3 v1.6 and §2.2); BC-2.02.009 postcondition revision; `SliceError::Len` contract tests green | BC-2.02.009 (decode_packet three-way postcondition — revised) | VP-008 fuzz harness return-type update (no-panic invariant unchanged) | STORY-110 (post-Wave-39) |
| STORY-112 | `extract_arp_frame(arp, outer_src_mac, packet_len)` implementation in `src/decoder.rs`; `decode_packet` routing: ARP early-extraction in **both** the strict `Ok(slice)` arm (`NetSlice::Arp` early-exit with `outer_src_mac` from `slice.link`) and the lax `Err(SliceError::Len(_))` arm (`LaxNetSlice::Arp` explicit routing with `outer_src_mac` from `lax.link`); `main.rs` `DecodedFrame` pattern-match wiring; `ArpAnalyzer` stub (`new`, `process_arp` no-op); VP-024 Sub-A Kani harnesses (safety, correctness, none-on-bad-size) | BC-2.16.001 (ARP Request frame extraction), BC-2.16.002 (ARP Reply frame extraction), BC-2.16.015 (decode-vs-analysis separation — DecodedFrame::Arp always produced) | VP-024 Sub-A (Kani: verify_extract_arp_frame_safety, verify_extract_arp_frame_eth_ipv4_correctness, verify_extract_arp_frame_none_on_bad_size) | STORY-111 |
| STORY-113 | `ArpAnalyzer` full implementation: binding table (`HashMap<[u8;4], BindingEntry>` + `insert_binding_lru`), GARP detection D2 (`is_gratuitous_arp`), D11 malformed finding emission, D12 mismatch detection; `summarize()` method (keys introduced here); `--arp` CLI flag; VP-024 Sub-B and Sub-D Kani harnesses; VP-024 Sub-C proptest | BC-2.16.003 (D2 Gratuitous ARP detection — opcode-agnostic), BC-2.16.005 (binding-table last-write-wins), BC-2.16.006 (binding-table cap — MAX_ARP_BINDINGS), BC-2.16.007 (D12 L2/L3 sender-MAC mismatch), BC-2.16.009 (D11 malformed ARP), BC-2.16.010 (ArpAnalyzer::summarize() AnalysisSummary keys — primary owner, keys introduced here), BC-2.16.011 (--arp CLI flag gates analysis) | VP-024 Sub-B (Kani: verify_classify_garp_total), VP-024 Sub-C (proptest: test_binding_table_last_write_wins), VP-024 Sub-D (Kani: verify_binding_table_cap) | STORY-112 |
| STORY-114 | D1 ARP spoof escalation (MEDIUM→HIGH on rebind_count >= SPOOF_REBIND_ESCALATION_DEFAULT within ARP_FLAP_WINDOW_SECS); GARP-that-conflicts escalation rule (D2+D1 interaction); MITRE emission (T0830, T1557.002) on D1/D2/D12 findings; VP-007 5-part atomic update (`technique_info` arms + SEEDED 23→25 + SEEDED_TECHNIQUE_ID_COUNT 23→25 + EMITTED_IDS +2 + `cargo test mitre` green) | BC-2.16.004 (D1 ARP spoof detection / rebind escalation), BC-2.16.014 (GARP-that-conflicts upgrade — triggers D1), BC-2.16.012 (--arp-spoof-threshold override) | VP-007 (5-part atomic update must be co-committed; `vp007_catalog_drift_guard` test must pass) | STORY-113 |
| STORY-115 | D3 ARP storm detection (`storm_counters: HashMap<[u8;6], StormCounter>`; per-MAC rate window); `--arp-storm-rate` CLI flag; `summarize()` storm key wiring (cross-story extension of BC-2.16.010 — adds `storm_findings` key; primary owner remains STORY-113); `src/cli.rs` additions; integration test for storm rate | BC-2.16.008 (D3 ARP storm rate detection), BC-2.16.013 (--arp-storm-rate override); extends BC-2.16.010 (adds storm_findings key to AnalysisSummary — cross-story extension, primary owner STORY-113) | none (D3 is unit-tested, not Kani-verified; T0814 MITRE tag deferred per DF-VALIDATION-001) | STORY-114 |

**Product-owner hand-off:** HS-INDEX waves 40–44 must be rewritten to match the BC/VP
columns above exactly. The previous HS-INDEX decomposition placed D1/D3/D11/D12/summarize
and VP-024 sub-properties across waves 40–44 inconsistently with this dependency chain.
The authoritative order is: migration (STORY-111) → extraction/Sub-A (STORY-112) →
analyzer/B/C/D (STORY-113) → spoof escalation/MITRE/VP-007 (STORY-114) → storm/CLI
(STORY-115).

---

## 7. Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-06-12 | Initial architecture delta for ARP Security Analyzer (v0.7.0) |
| 1.1 | 2026-06-12 | §6 BC mis-citation corrections in STORY-114 and STORY-115 rows (product-owner catch during HS-INDEX propagation): (1) STORY-115 `BC-2.16.016` (non-existent) replaced with `BC-2.16.010` (summarize AnalysisSummary keys, which includes `storm_findings`). (2) STORY-115 `BC-2.16.014` (GARP-that-conflicts escalation — incorrect placement) replaced with `BC-2.16.013` (`--arp-storm-rate` CLI flag). BC-2.16.014 moved to STORY-114 where it belongs: GARP-that-conflicts is a D2+D1 interaction rule, co-located with the full D1 spoof escalation logic and MITRE attribution work. STORY-115 §6 BCs are now: BC-2.16.008 (D3 storm detection), BC-2.16.013 (--arp-storm-rate flag), BC-2.16.010 (storm summary key). |
| 1.2 | 2026-06-12 | §6 definitive BC-intent label and primary-owner correction. Root causes fixed: (1) STORY-112 previously cited BC-2.16.009 (D11 malformed) as a primary owner — wrong; D11 finding emission belongs in STORY-113 with the full ArpAnalyzer implementation; BC-2.16.015 (decode-vs-analysis separation) is the correct third primary for STORY-112. (2) STORY-113 previously omitted BC-2.16.011 (--arp flag) and incorrectly carried BC-2.16.004 (D1 spoof) as a partial primary — D1 full escalation path belongs entirely to STORY-114; STORY-113's binding table work only creates the binding entry infrastructure without emitting D1 findings. (3) STORY-114 previously cited BC-2.16.008 with wrong intent label "MITRE attribution" (BC-2.16.008 is D3 ARP storm rate detection, owned by STORY-115), and double-cited it; removed. BC-2.16.011 and BC-2.16.012 were wrongly split across STORY-114 arms — BC-2.16.011 (--arp flag) is primary in STORY-113; BC-2.16.012 (--arp-spoof-threshold) is primary in STORY-114. All 15 SS-16 BCs now primary-assigned exactly once across STORY-112..115; BC-2.02.009 primary in STORY-111. |
| 1.3 | 2026-06-12 | F-A07 — §4.1 decoder.rs change note expanded: "update decoder module-doc comment" now explicitly covers BOTH the top-of-file `//!` module doc AND the `SliceError` import comment block (src lines ~42-48, etherparse 0.16/0.17 references) — both must be updated to 0.20 in the same prose-sweep. |
| 1.4 | 2026-06-12 | F-A03 Pass 3 adversarial remediation — (MED-02) §3.3 D1 typo "rekinds" → "rebinds". §5.0 added: forward-declaration convention (normative): BC-2.10.005/008 carry PLANNED marker until STORY-114 merges; explicit STORY-114 F4 obligation table listing all five src/mitre.rs hardcoded 23/15 sites (~lines 205, 212, 218, 301-302, 339); process-gap record for F-A03-LOW-02 / Slice C (S-7.02 candidates). ADR-008 v1.2 simultaneously: struct fields added, Decision 5 confidence column, seven-coordinated fix, LOW-01 comment rewording. |
| 1.5 | 2026-06-12 | F-SA04 adversarial Pass 4 remediation — (F-SA04-MED-01) §5.0 STORY-114 F4 obligation table line numbers corrected with verified actuals from src/mitre.rs: EMITTED_IDS array at lines 221-240 (not 301-302); technique_info last arm T0827 at line 178 / `_ => return None` at line 179 (not 339); SEEDED_TECHNIQUE_ID_COUNT constant at line 341; SEEDED_TECHNIQUE_IDS array body at lines 305-333. (F-SA04-MED-02) Table reframed into two groups: (a) functional code edits required for vp007_catalog_drift_guard to pass; (b) stale-count comment updates. (F-SA04-MED-03) Missing SEEDED_TECHNIQUE_IDS array body edit (@305-333) added as a separate functional row (required by ADR-008 Decision 6 step 2). (F-C-P4-HIGH-001) Pre-existing stale "11 Enterprise + 10 ICS = 21 total" comment added as an additional stale-comment fix row: correct post-STORY-109 split is 11 Enterprise + 12 ICS = 23; post-ARP (STORY-114) split is 12 Enterprise + 13 ICS = 25. (F-B4-L01) §3.2 ARP_STORM_RATE_DEFAULT description and §3.3 D3 Storm trigger description changed from "sustained" framing to "average-frames-per-second-since-window-start (per BC-2.16.008 Invariant 2)". ADR-008 v1.3 simultaneously: Decision 4 storm_rate struct comment changed from "frames-per-second sustained threshold" to "average-frames-per-second-since-window-start threshold (per BC-2.16.008 Invariant 2)". |
| 1.6 | 2026-06-12 | F-SA5 adversarial Pass 5 remediation — (F-SA5-HIGH-01) §5.0(b) stale-comment table: deleted duplicate row targeting src/mitre.rs line 301 (the F-C-P4-HIGH-001 row added in v1.5 was contradictory — it would have overwritten line 301 with post-STORY-109/post-ARP totals that belong on lines 302/339). The surviving row correctly annotates line 301 as "(pre-STORY-109 subtotal)"; subsequent rows handle the incremental totals at src/mitre.rs lines 302 and 339. Mandatory sibling sweep confirms only one §5.0 row now targets line 301. (F-SA5-D2) §3.3 D2 GARP confidence cell updated from "LOW/Inconclusive" to "LOW base; MEDIUM when GARP also conflicts with existing binding (D2+D1 interaction)" to mirror ADR-008 Decision 5 two-state phrasing (cosmetic symmetry fix). |
| 1.7 | 2026-06-12 | F-B6 adversarial Pass 6 remediation — OBS-1: §4.1 Cargo.toml line reference corrected from "lines 22–27" to "~lines 21–26". |
| 1.8 | 2026-06-12 | Pass 8 remediation — (HIGH-01) §2.2: replaced `lax_ip_triple` `LaxNetSlice::Arp(_) => unreachable!(...)` arm with explicit ARP routing. Snaplen-truncated ARP frames yield `Some(LaxNetSlice::Arp(_))` from the lax parser; that arm is reachable at runtime — unreachable! would panic, violating VP-008/VP-024 Sub-A. Full lax decode_packet arm spec added (mirrors strict arm; extract_arp_frame used for both). STORY-111 and STORY-112 scope rows in §6 updated accordingly: STORY-111 now explicitly notes the lax_ip_triple arm is NOT unreachable; STORY-112 now covers ARP early-extraction in both strict and lax decode paths. (HIGH-02) §5.0 pre-existing brownfield debt table added: src/mitre.rs:91 IcsImpact Display "Impact (ICS)" contradicts BC-2.10.002 PC3 canonical "Impact"; STORY-114 must adjudicate. (Ethernet2Slice::source() return type [u8; 6] by value confirmed — see ADR-008 v1.6 Source/Origin; no change needed to snippet.) |
| 1.9 | 2026-06-12 | F-B9-M02 — §7 v1.6 row: dropped stale doc-internal line number parentheticals ("line 265", "rows 266/267") that no longer resolve after §5.0 table moved; stable src/mitre.rs target line numbers 301/302/339 retained in the body text where applicable. F-SA9-LOW-01 — reordered §7 rows to strict ascending 1.0→1.9 (1.2 was misplaced after 1.5); row content unchanged. F-SA9-LOW-02 — corrected typo "Ethereum2Slice" → "Ethernet2Slice" in v1.8 changelog row. |
| 1.10 | 2026-06-13 | F-SA11-MED-01 (Pass-12 corpus debt cleanup — F-4) — §7 changelog row added for the 1.9→1.10 promotion. The v1.10 content change itself was the Some()-double-wrap fix in §5.0 stale-count comment table (Pass-11 remediation): the incorrect `Some(eth.source())` reference in the v1.8 migration snippet was verified correct (Ethernet2Slice::source() returns [u8; 6] by value in etherparse 0.20.1 — no dereference needed, Some() wraps a [u8; 6] value); frontmatter was bumped to 1.10 at that pass. This row-add completes the §7 audit trail without further version change. |
