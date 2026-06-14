---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-12T01:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.3: F3 story-anchor back-fill. — 2026-06-14"
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

# BC-2.16.006: Binding-Table Cap — Table Never Exceeds MAX_ARP_BINDINGS via LRU Eviction

## Description

The `ArpAnalyzer` binding table (`HashMap<[u8; 4], BindingEntry>`, production substrate) is
bounded by `MAX_ARP_BINDINGS = 65,536` entries (a wirerust engineering default sized for one
/16 IPv4 subnet). When a new IP would cause the table to exceed this cap,
`insert_binding_lru` evicts the entry with the minimum `last_seen_ts` timestamp (heuristic
LRU approximation); exactly one entry is evicted; table length never exceeds MAX_ARP_BINDINGS.
`bindings.len()` NEVER exceeds `MAX_ARP_BINDINGS` at any point during processing. This is
verified by VP-024 Sub-property D via a scaled Kani proof (`TEST_MAX_ARP_BINDINGS = 8`).
**Scope of proof:** VP-024 Sub-D proves only `len <= cap`; the Kani harness does not prove a
formal LRU ordering invariant (the proof uses a BTreeMap surrogate for the Kani bounded model).
The cap prevents unbounded memory growth in long-running captures with many hosts.
`MAX_ARP_BINDINGS = 65,536` is a wirerust engineering default; no authoritative industry
standard for this value exists.

## Preconditions

1. `ArpAnalyzer.bindings` is a `HashMap<[u8; 4], BindingEntry>` (production substrate; BTreeMap
   is used only as Kani surrogate in VP-024 Sub-D scaled proof, not in production code).
2. The current table size equals `MAX_ARP_BINDINGS = 65,536` (cap has been reached).
3. A new frame arrives with a `sender_ip` NOT currently in the table.
4. `--arp` flag is active.

## Postconditions

1. `insert_binding_lru` evicts the entry with the minimum `last_seen_ts` timestamp (heuristic
   LRU approximation) before inserting the new entry; exactly one entry is evicted.
2. `bindings.len() <= MAX_ARP_BINDINGS` after every call to `insert_binding_lru`, for all
   table sizes from 0 to MAX_ARP_BINDINGS + N (for any N ≥ 0).
3. The newly inserted entry is accessible via `bindings[new_sender_ip]` immediately after
   insertion.
4. The evicted entry (the LRU IP) is no longer accessible in `bindings`. Any subsequent
   frames for the evicted IP are treated as a new first-time observation.
5. No more than one entry is evicted per insertion (one-in-one-out at the cap boundary).

## Invariants

1. **Hard cap**: `bindings.len()` NEVER exceeds `MAX_ARP_BINDINGS`. This is not a soft
   limit or a "try to evict" policy — it is a strict upper bound enforced on every insert.
2. **LRU eviction policy (heuristic)**: the evicted entry is the one with the minimum
   `last_seen_ts` timestamp — the IP for which the most recently observed ARP frame has
   the earliest Unix timestamp. For an offline PCAP analyzer, this is equivalent to
   processing order. VP-024 Sub-D (Kani) proves only the cap invariant (`len <= cap`);
   the strict LRU ordering is NOT formally proven (the Kani harness uses a BTreeMap
   surrogate and does not assert eviction order beyond cap compliance).
3. **Eviction does not emit a Finding**: LRU eviction is a memory-management operation.
   No security Finding is emitted when an entry is evicted.
4. **Eviction and missed detection trade-off (documented)**: an attacker who can inject
   ≥65,536 distinct source IPs CAN evict legitimate bindings and cause missed spoof
   detections for those IPs. This is an accepted trade-off for memory bounding in an offline
   forensic tool. wirerust is NOT a live inline defense system; this is not a safety risk
   for the analyzed network (ADR-008 §Negative / Trade-offs).
5. **MAX_ARP_BINDINGS = 65,536 is a wirerust engineering default**: sized for a /16 subnet
   (65,534 host addresses). No authoritative industry default exists
   (mitre-arp-additional-detections.md §4b). Not a CLI flag in v0.7.0.
6. **MAX_STORM_COUNTERS = 4,096 analogous cap**: the storm_counters HashMap is bounded by
   a separate constant `MAX_STORM_COUNTERS = 4,096` (also a wirerust engineering default),
   also subject to LRU eviction. This BC covers the bindings table; storm counter cap
   behavior is analogous but tracked in BC-2.16.008.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Table at MAX_ARP_BINDINGS - 1: new IP arrives | Inserted without eviction; len → MAX_ARP_BINDINGS |
| EC-002 | Table at MAX_ARP_BINDINGS: new IP arrives | LRU entry evicted; new IP inserted; len remains MAX_ARP_BINDINGS |
| EC-003 | Table at MAX_ARP_BINDINGS: already-tracked IP arrives (update, not insert) | No eviction; existing entry updated; len unchanged |
| EC-004 | 65,537 distinct IPs processed in sequence | After each insertion past 65,536, one LRU entry is evicted; len never exceeds 65,536 |
| EC-005 | LRU entry evicted; same IP reappears later | Treated as first-time observation; rebind_count=0; no spoof finding on first re-appearance |
| EC-006 | Kani scaled proof: TEST_MAX_ARP_BINDINGS=8; 9 distinct IPs processed | Table len never exceeds 8; 9th IP inserted after LRU eviction of IP 1 |

## Canonical Test Vectors

| Sequence | Expected `bindings.len()` | Notes |
|---|---|---|
| 65,535 distinct IPs → len=65,535 | 65,535 | One below cap — no eviction |
| 65,536th distinct IP | 65,536 | Cap reached; no eviction yet |
| 65,537th distinct IP | 65,536 | LRU evicted; len stays at 65,536 |
| Kani: 0 through 8 distinct IPs (TEST_MAX_ARP_BINDINGS=8) | ≤ 8 after each step | Scaled proof |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property D (MAX_ARP_BINDINGS cap): `bindings.len()` NEVER exceeds MAX_ARP_BINDINGS; LRU evicts exactly one entry on overflow | Kani: scaled cap TEST_MAX_ARP_BINDINGS=8; 9-iteration loop (cap+1); assert len ≤ 8 after each insert; `#[kani::unwind(12)]` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the binding-table cap is a bounded-resource invariant for the ARP Security Analysis capability; without it, a long-running capture against a large network would cause unbounded memory growth in the analyzer |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs `insert_binding_lru`, C-23); ADR-008 Decision 4 |
| Stories | STORY-113 |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — capacity management operation; no finding emission) |

## Related BCs

- BC-2.16.005 — composes with (LRU eviction operates on the binding table whose semantics BC-2.16.005 defines)
- BC-2.16.004 — depends on (eviction affects spoof detection: evicted IPs re-initialize)

## Architecture Anchors

- `src/analyzer/arp.rs` — `fn insert_binding_lru(bindings: &mut HashMap<[u8;4], BindingEntry>, ip: [u8;4], mac: [u8;6], cap: usize)` (free pure-core function; production substrate HashMap; VP-024 Sub-D Kani harness uses BTreeMap surrogate for bounded model only). **Normative note (ADR-008 Decision 4):** `insert_binding_lru` has no `ts` parameter; `last_seen_ts` is written by `process_arp` on every observation and read by `insert_binding_lru` only during the eviction scan (per ADR-008 Decision 4).
- `src/analyzer/arp.rs` — `const MAX_ARP_BINDINGS: usize = 65_536` (wirerust engineering default; NOT an industry standard)
- `src/analyzer/arp.rs` — `const MAX_STORM_COUNTERS: usize = 4_096` (wirerust engineering default; analogous cap for storm_counters)
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 4` — LRU rationale and constant documentation
- `.factory/specs/architecture/arp-architecture-delta.md §3.2`

## Story Anchor

STORY-113

## VP Anchors

- VP-024 — ARP Frame Parse Safety and Binding-Table Invariant (Sub-property D: MAX_ARP_BINDINGS cap, scaled Kani proof)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 4 (LRU rationale and cap constant); arp-architecture-delta.md §3.2; mitre-arp-additional-detections.md §4b (MAX_ARP_BINDINGS is a wirerust engineering choice; no industry default) |
| **Confidence** | high — cap constant and LRU policy are explicit engineering decisions in the architecture; Kani scaled proof is specified in VP-024 Sub-D |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same cap, same sequence always produces same table state |
| **Thread safety** | single-threaded |
| **Overall classification** | stateful pure core — VP-024 Sub-D (Kani, scaled) |
