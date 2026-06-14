---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-06-12T02:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.4: Pass-8 remediation F-B8-M01: PC1 tightened to exclude broadcast (255.255.255.255) in addition to all-zero (0.0.0.0) sender IPs, consistent with Invariant 5. Test-infrastructure note added referencing ADR-008 Decision 4 affordances (new_for_test(), process_arp_for_test(), bindings_snapshot()) for VP-024 Sub-C proptest use by F3/F4 implementers. — 2026-06-12"
  - "v1.5: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.005: Binding-Table Update — Last-Seen MAC Wins for a Given IP

## Description

The `ArpAnalyzer` binding table maps `[u8; 4]` IP addresses to `BindingEntry` structs. Every
time `process_arp` is called with a frame carrying a `sender_ip` and `sender_mac`, the binding
table is updated so that `bindings[sender_ip].mac` always holds the MAC from the most recently
processed frame for that IP (last-write-wins). This deterministic update semantic is the
foundational correctness property for spoof detection (BC-2.16.004): if last-write-wins is
violated, the binding table can represent a stale MAC as current, causing missed detections.
VP-024 Sub-property C (proptest) verifies this invariant over arbitrary frame sequences.

## Preconditions

1. `frame` is a fully-populated `ArpFrame` with a `sender_ip` that is neither all-zero
   (`0.0.0.0` / `[0,0,0,0]`) nor broadcast (`255.255.255.255` / `[255,255,255,255]`). These
   inadmissible values are filtered at analysis entry per Invariant 5; PC1 applies only to
   admissible sender IPs.
2. `--arp` flag is active (analysis gate per BC-2.16.011).
3. The binding table has not exceeded `MAX_ARP_BINDINGS = 65,536` entries for the current
   sender_ip key (existing keys are always updatable regardless of table size).

## Postconditions

1. After `process_arp(frame, ts)` completes, `bindings[frame.sender_ip].mac == frame.sender_mac`.
2. No other entry in the binding table is modified by this update (isolation: updating IP A's
   binding does not affect IP B's binding).
3. The binding table contains exactly one entry per IP key at any point in time; there are no
   duplicate entries for the same IP with different MACs simultaneously held.
4. The `rebind_count` field is incremented only when the incoming MAC differs from the existing
   MAC (i.e., only on a genuine rebind event, not on repeated same-MAC observations).
5. The update is unconditional for the `mac` field: even if the same MAC is repeated, the entry
   is touched (timestamp of last-seen may be updated for LRU bookkeeping per BC-2.16.006).

## Invariants

1. **Last-write-wins**: for any IP, after processing N frames, `bindings[ip].mac` equals the
   `sender_mac` from the Nth (most recent) frame with that `sender_ip`. This is the
   `prop_assert_eq!(&entry.mac, expected_mac)` assertion in VP-024 Sub-C proptest.
2. **No-duplicate-key**: the HashMap structure guarantees this at the data-structure level.
   The proptest explicitly checks `unique_ips.len() == bindings.len()` as a documentation
   witness.
3. **New-entry initialization**: when an IP is seen for the first time, `bindings` is inserted
   with `rebind_count = 0`, `first_rebind_ts = None`, `spoof_high_emitted = false`. No spoof
   finding is emitted on first observation.
4. **Eviction does not affect correctness**: LRU eviction (BC-2.16.006) may remove entries for
   IPs not recently seen. Once an entry is evicted and the same IP reappears, it is treated as
   a new first-time observation (rebind_count reset to 0). This is an accepted trade-off for
   memory bounding in long captures (ADR-008 §Rationale).
5. **Zero/broadcast sender IP admissibility**: `sender_ip = 0.0.0.0` (all-zero) and
   `sender_ip = 255.255.255.255` (broadcast) are FILTERED at analysis entry — `process_arp`
   MUST NOT insert bindings for these values. Zero IPs occur in RFC 5227 ACD probes and are
   not valid host addresses. Broadcast sender IPs are malformed ARP. Neither value produces a
   spoof finding. This rule is the canonical definition that BC-2.16.004 EC-010 cross-references.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IP seen for the first time | New binding inserted; mac=sender_mac; rebind_count=0 |
| EC-002 | Same IP, same MAC (repeated) | Binding updated (LRU touch); mac unchanged; rebind_count unchanged |
| EC-003 | Same IP, different MAC | Binding updated (mac=new_mac); rebind_count incremented; spoof detection triggered (BC-2.16.004) |
| EC-004 | Two distinct IPs updated in sequence | Each binding updated independently; neither affects the other |
| EC-005 | IP evicted from table (cap reached) then reappears | Treated as first-time observation; rebind_count=0; spoof_high_emitted=false |
| EC-006 | Sender IP is all-zero (0.0.0.0) | Zero sender IPs are filtered at analysis entry: `process_arp` MUST NOT insert a binding for `sender_ip = [0,0,0,0]`. Zero sender IPs occur in RFC 5227 ACD probes and are not valid host addresses to track. No spoof detection is triggered (per BC-2.16.004 EC-010 cross-reference). |
| EC-007 | Sender IP is broadcast (255.255.255.255) | Broadcast sender IPs are filtered at analysis entry: `process_arp` MUST NOT insert a binding for `sender_ip = [255,255,255,255]`. ARP frames with broadcast sender IPs are malformed (RFC 826 does not assign broadcast as a valid sender address); inserting them into the binding table would produce spurious spoof findings. No binding inserted; no spoof detection triggered. |

## Canonical Test Vectors

| Sequence of frames (sender_ip, sender_mac) | Final `bindings[ip].mac` | Notes |
|---|---|---|
| [(10.0.0.1, AA:AA:AA:AA:AA:AA)] | AA:AA:AA:AA:AA:AA | First observation |
| [(10.0.0.1, AA:AA), (10.0.0.1, AA:AA)] | AA:AA:AA:AA:AA:AA | Repeated same MAC — no rebind |
| [(10.0.0.1, AA:AA), (10.0.0.1, BB:BB)] | BB:BB:BB:BB:BB:BB | Last-write-wins; rebind detected |
| [(10.0.0.1, AA:AA), (10.0.0.2, CC:CC), (10.0.0.1, DD:DD)] | bindings[10.0.0.1]=DD:DD; bindings[10.0.0.2]=CC:CC | Two IPs independent; last writer for each IP wins |
| 65537 distinct IPs, then first IP again | bindings[first_ip].mac = mac_from_last_frame OR entry absent (evicted) | LRU eviction at cap; if evicted, first IP is re-initialized on re-appearance |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property C (binding-table last-write-wins determinism): for any sequence of ArpFrames, bindings[ip].mac == MAC from last frame with that IP; no duplicate keys | proptest: arbitrary Vec<(IP, MAC, opcode)> up to 1000 entries; last-write assertion and unique-keys check |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the binding-table last-write-wins semantic is the foundational correctness property for D1 ARP spoof detection; incorrect binding semantics would produce silent false negatives (stale MAC treated as current) or false positives (current MAC treated as new) |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer, C-23); ADR-008 Decision 4 |
| Stories | STORY-113 |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — binding table update is a data structure operation; findings are emitted by BC-2.16.004) |

## Related BCs

- BC-2.16.004 — depends on (spoof detection correctness depends on last-write-wins semantics)
- BC-2.16.006 — composes with (LRU eviction operates on the binding table this BC defines)

## Architecture Anchors

- `src/analyzer/arp.rs` — `ArpAnalyzer.bindings: HashMap<[u8; 4], BindingEntry>` — the binding table
- `src/analyzer/arp.rs` — `struct BindingEntry { mac: [u8; 6], rebind_count: u32, first_rebind_ts: Option<u32>, spoof_high_emitted: bool, last_seen_ts: u32 }`
- `src/analyzer/arp.rs` — `fn insert_binding_lru(bindings: &mut HashMap<[u8;4], BindingEntry>, ip: [u8;4], mac: [u8;6], cap: usize)` — free pure-core update function (production substrate: HashMap; BTreeMap used only as Kani surrogate in VP-024 Sub-D scaled proof). **Normative note (ADR-008 Decision 4):** `insert_binding_lru` has no `ts` parameter; `last_seen_ts` is written by `process_arp` on every observation and read by `insert_binding_lru` only during the eviction scan (per ADR-008 Decision 4).
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 4`
- `.factory/specs/architecture/arp-architecture-delta.md §3.1`

**Test infrastructure (ADR-008 Decision 4 extensions, used by VP-024 Sub-C proptest):**
- `new_for_test()` — constructs an `ArpAnalyzer` instance for test contexts (bypasses
  any production-init side effects).
- `process_arp_for_test(&frame, ts)` — thin test-only wrapper around `process_arp` that
  exposes the method under `#[cfg(test)]` without altering production call paths.
- `bindings_snapshot()` — returns a clone of the internal `bindings` HashMap for
  assertion in the proptest (`prop_assert_eq!(&entry.mac, expected_mac)`). These
  affordances are `#[cfg(test)]` only; they do not affect the production binary.

## Story Anchor

STORY-113

## VP Anchors

- VP-024 — ARP Frame Parse Safety and Binding-Table Invariant (Sub-property C: proptest last-write-wins determinism)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 4; arp-architecture-delta.md §3.1; vp-024 Sub-C proptest sketch |
| **Confidence** | high — last-write-wins is a structural property of HashMap insert semantics; proptest provides exhaustive coverage of arbitrary sequences |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same sequence always produces same final binding state |
| **Thread safety** | ArpAnalyzer is single-threaded |
| **Overall classification** | stateful pure core — VP-024 Sub-C (proptest) |
