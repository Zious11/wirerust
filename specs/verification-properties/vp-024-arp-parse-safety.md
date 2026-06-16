---
document_type: verification-property
level: L4
version: "2.3"
status: verified
producer: architect
timestamp: 2026-06-16T00:00:00Z
phase: f6
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.16.001
bcs:
  - BC-2.16.001
  - BC-2.16.002
  - BC-2.16.003
  - BC-2.16.005
  - BC-2.16.006
module: src/analyzer/arp.rs + src/decoder.rs
proof_method: kani
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-16"
proof_file_hash: null  # No canonical recomputation method defined for VP-024 proof files. Follow-up: define hash method (e.g. SHA-256 of src/decoder.rs kani_proofs + src/analyzer/arp.rs kani_proofs modules) and populate. Tracked as FU-F6-KANI-CLEANUP.
verified_at_commit: "6e9f2cc"  # develop HEAD at F6 PR #250 merge (2026-06-16)
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.0: Authored in Phase-F2 spec evolution for ARP security analyzer (SS-16). Four sub-properties: (A) extract_arp_frame parse safety / panic-freedom, (B) GARP detection totality, (C) binding-table last-write-wins determinism, (D) MAX_ARP_BINDINGS cap invariant. Kani (A, B, D) + proptest (C). P1. status=draft; harnesses authored in F4 TDD."
  - "v1.1 (2026-06-12): F-A04 — removed BC-2.16.007 from bcs: frontmatter array and Source Contract (D12 is unit-tested, not Kani; body note retained). F-A01 — Sub-D point 2 downgraded: 'least recently accessed IP' claim struck; heuristic-not-proven language added; anchor table row D corrected. F-A02 — Proof Method narrative rewritten: 'scaled HashMap' / 'Kani can explore HashMap' false claim replaced with BTreeMap-surrogate rationale; kani::any_hashmap suggestion removed. F-A05 — Sub-A negative harness comment re-anchored from BC-2.16.009 to BC-2.16.001/002 with unit-test note for D11. F-A06 — ArpFrame size unified to '≤40 bytes' in Proof Method (was '32-byte' at line ~176, '≤40 bytes' at Feasibility). F-A07 — arch-delta §4.1 note added (decoder module-doc + SliceError import comment both in scope)."
  - "v1.2 (2026-06-12): F-SA5 — Sub-A negative harness (verify_extract_arp_frame_none_on_bad_size): added explicit F4 obligation note warning of vacuous-satisfiability risk: if from_slice rejects the bad-HLEN/PLEN buffer on the Err path, the is_none() assertion is never reached; F4 must confirm Ok-arm reachability or restructure the harness to assert it explicitly via kani::cover! before F6 lock."
  - "v1.3 (2026-06-12): F-B6-M01 — Source Location anchor for insert_binding_lru updated to note that no ts parameter is present; last_seen_ts is written by process_arp on every observation and read by insert_binding_lru only during eviction scan (ADR-008 Decision 4 normative note). Sub-D harness skeleton comment updated with same rationale. Signature itself is unchanged."
  - "v1.4 (2026-06-12): Pass 8 remediation — (MED-01) Sub-A correctness harness (verify_extract_arp_frame_eth_ipv4_correctness): added same F4 vacuous-satisfiability obligation note that the negative harness carries: if from_slice rejects the fixed-header buffer (unlikely but unconfirmed), field-correctness assertions are never reached; F4 must add kani::cover! reachability assertion before F6 lock. (MED-02) Sub-C proptest sketch: declared three required test affordances in ADR-008 Decision 4 scope: new_for_test() constructor, process_arp_for_test(&frame, ts: u32) wrapper (note: ts arg is mandatory — no zero-arg form), and bindings_snapshot() test accessor; updated sketch to use process_arp_for_test(&frame, 0u32) matching the canonical two-arg process_arp signature, and bindings_snapshot() instead of direct field access; these affordances are ADR-008 Decision 4 extensions requiring PO propagation. (MED-03) Sub-D property statement: added normative specification of last_seen_ts initialization at insert time (new entries created by insert_binding_lru initialize last_seen_ts: 0; process_arp writes the real timestamp immediately after insert returns); explicitly reaffirmed that cap proof (len<=cap) is valid regardless of last_seen_ts init value and that LRU target-correctness is unit-tested in STORY-113, not Kani-proven."
  - "v1.5 (2026-06-13): F-A-01/F-A-02 remediation (Pass-1 adversarial F3) — Sub-C primary anchor corrected to BC-2.16.005 (last-write-wins) only. BC-2.16.004 (D1 ARP spoof) removed from bcs: frontmatter array and from Sub-C's anchor list; it is INDIRECTLY supported by Sub-C (the last-write-wins substrate BC-2.16.004 depends upon) but is not a VP-024 formally-verified BC. Its primary story is STORY-114 (wave 43), which runs after STORY-113 where Sub-C is implemented. Sub-C anchor table row updated: PRIMARY=BC-2.16.005, INDIRECT=BC-2.16.004 with explicit scope note. Source Contract BC-2.16.004 line changed from 'Related' to 'Indirectly supported'. VP-INDEX Verified-BCs catalog row updated (BC-2.16.004 removed); [^vp024-bc-scope] footnote rewritten to explain primary/indirect anchor split."
  - "v1.6 (2026-06-14): F-P16-A-02 remediation (Pass-16 semantic-anchor) — frontmatter module: field corrected from src/analyzer/arp.rs (singular) to src/analyzer/arp.rs + src/decoder.rs, aligning frontmatter with VP body (Sub-A target extract_arp_frame resides in src/decoder.rs) and VP-INDEX catalog row. No property or anchor content changed."
  - "v1.7 (2026-06-14, F3 ARP VP-layer audit title-sync): Source Contract 'Indirectly supported BC' BC-2.16.004 wording corrected: 'MEDIUM or HIGH finding' → 'MEDIUM then HIGH finding' to mirror BC-2.16.004 H1 v1.5 (sequential escalation). No proof-method, postcondition, or anchor content changed."
  - "v1.8 (2026-06-15): O-1 remediation (F4 re-streak finding) — Sub-A negative harness widened to cover the FULL reject contract matching decoder.rs:312-315 (hw_addr_type != ETHERNET || proto_addr_type != IPV4 || hw_addr_size != 6 || proto_addr_size != 4 → None). HTYPE/PTYPE bytes made symbolic (no longer pinned to Ethernet/IPv4). kani::assume updated to the 4-part OR condition. Harness renamed verify_extract_arp_frame_none_on_bad_size → verify_extract_arp_frame_none_on_invalid_header. Property Statement point 3 and symbolic-input summary table updated accordingly. Harness-comment prose corrected. Cross-references to BC-2.16.001 PC2-PC5 and BC-2.16.009 PC3a-3d are unchanged. Decision D-077 is the triggering change (type-rejection guard added to extract_arp_frame)."
  - "v1.9 (2026-06-15): O-1 propagation fix (adversarial F4 re-streak finding, MEDIUM) — reverted the v1.8 cosmetic rename (verify_extract_arp_frame_none_on_bad_size → verify_extract_arp_frame_none_on_invalid_header) to eliminate an 11-site cross-artifact propagation liability across src/decoder.rs, BC-2.16.009, three architecture docs, dependency-graph.md, wave-schedule.md, STORY-112, and sealed changelogs. The substantive 4-part coverage widening from v1.8 (HTYPE/PTYPE bytes made symbolic, kani::assume covering the full hw_addr_type != ETHERNET OR proto_addr_type != IPV4 OR hw_addr_size != 6 OR proto_addr_size != 4 rejection region, property-statement and symbolic-input table updated accordingly) is RETAINED intact. The harness function name reverts to verify_extract_arp_frame_none_on_bad_size; a clarifying scope note has been added to the harness comment and Property Statement point 3 explaining that despite the '_bad_size' name the harness now verifies the full type-or-size reject contract per D-077 (name retained to avoid cross-artifact churn per this decision)."
  - "v2.0 (2026-06-16): F6 LOCK — All five Kani harnesses prove VERIFICATION:- SUCCESSFUL. verification_lock set to true; status draft→verified; phase f2→f6; proof_completed_date set to 2026-06-16. Sub-D Proof Method narrative corrected: insert_binding_lru_btree reference replaced with insert_binding_lru_array (fixed-capacity array surrogate behind #[cfg(any(kani, test))]). Reason for surrogate: CBMC cannot symbolically execute std::collections::BTreeMap — runs out of memory even at 3 inserts with no resolution after 45+ minutes at any cap scale. The array surrogate reproduces the identical 3-branch eviction algorithm (update-in-place / evict-min-last_seen_ts / append) used by the production insert_binding_lru (HashMap). Branch fidelity confirmed by a branch-fidelity test asserting surrogate matches production behavior. The cap invariant proof (len <= cap) remains sound for the production HashMap path by the map-implementation-independence argument: cap invariance is a purely arithmetic property independent of map implementation. Sub-A F4 obligation notes (vacuity risk on Ok-arm reachability) resolved: kani::cover! reachability assertions were added in F4 and confirmed non-vacuous. proof_file_hash and verified_at_commit left null pending develop HEAD after F6 PR merges — do not populate from speculative values."
  - "v2.1 (2026-06-16): F6 anchor population — verified_at_commit populated with 6e9f2cc (develop HEAD at F6 PR #250 merge, 2026-06-16; all 46/46 project-wide Kani harnesses VERIFICATION:- SUCCESSFUL). proof_file_hash deferred: no canonical recomputation method defined for VP-024 proof files; follow-up recorded as FU-F6-KANI-CLEANUP (define method, e.g. SHA-256 of kani_proofs modules in src/decoder.rs + src/analyzer/arp.rs, then populate). Patch bump only — no property, proof-method, or postcondition content changed."
  - "v2.2 (2026-06-16): F7 consistency F2 — harness skeleton block for Sub-D (verify_binding_table_cap) synced to shipped insert_binding_lru_array signature: entries type corrected from [Option<([u8;4], BindingEntry)>; N] to [([u8; 4], [u8; 6], u32); CAP] (array-of-tuples, no Option wrapper); separate len: &mut usize counter added; assert updated from .filter(|e| e.is_some()).count() <= CAP to len <= CAP; loop rewritten as while i <= CAP as u8 to match shipped harness. Symbolic-input summary table Sub-D row updated to reflect new entries/len signature. Surrounding narrative (NOTE ON SUBSTRATE, soundness argument) was already correct in v2.0 — only the code block skeleton was stale. Patch bump only — verification_lock, status, and all proof properties unchanged."
  - "v2.3 (2026-06-16): F7 consistency re-audit residual — supplementary surrogate-signature prose (lines ~228/441/590) synced to shipped [([u8;4],[u8;6],u32);N] + separate len counter. Three narrative locations that still described the surrogate element type as [Option<([u8;4], BindingEntry)>; N] (Option-wrapped, BindingEntry, no separate len): (1) Proof Method 'Adopted surrogate' paragraph; (2) NOTE ON SUBSTRATE comment inside harness skeleton block; (3) Source Location forward-reference for insert_binding_lru_array (signature corrected, len: &mut usize parameter added). The code block skeleton (already corrected in v2.2), normative proof spec, BC cross-refs, and assertions are unchanged. Patch bump only — verification_lock, status, and all proof properties unchanged."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-024: ARP Frame Parse Safety and Binding-Table Invariant

## Property Statement

The ARP pure-core functions in `src/analyzer/arp.rs` are memory-safe, panic-free,
and total over their bounded symbolic input domains. The property decomposes into four
sub-properties targeting the two pure-core functions `extract_arp_frame` (in `src/decoder.rs`)
and the detection-classification logic in `src/analyzer/arp.rs`:

**Sub-property A — ARP frame extraction parse safety** (`extract_arp_frame`, anchors
BC-2.16.001/BC-2.16.002):

For any `ArpPacketSlice<'_>` that etherparse 0.20 successfully constructs (i.e., the slice
is well-formed per etherparse's own validation):

1. `extract_arp_frame(arp, outer_src_mac, packet_len)` NEVER panics for any valid
   `ArpPacketSlice` input. No out-of-bounds access, no unwrap panic.
2. When `hw_addr_size() == 6` and `proto_addr_size() == 4` and `hw_addr_type() == ETHERNET`
   and `proto_addr_type() == IPV4`, the function returns `Some(ArpFrame { ... })` with fields
   copied exactly from the slice accessors:
   - `sender_mac` = first 6 bytes of `sender_hw_addr()`
   - `target_mac` = first 6 bytes of `target_hw_addr()`
   - `sender_ip`  = first 4 bytes of `sender_protocol_addr()`
   - `target_ip`  = first 4 bytes of `target_protocol_addr()`
   - `operation`  = raw u16 from `operation().0`
3. When `hw_addr_type() != ETHERNET` or `proto_addr_type() != IPV4` or `hw_addr_size() != 6`
   or `proto_addr_size() != 4` (any of the four conditions in the combined guard at
   decoder.rs:312-315), the function returns `None` (no panic — graceful rejection). This is
   explicitly verified by the `verify_extract_arp_frame_none_on_bad_size` Kani harness
   below, which makes HTYPE, PTYPE, HLEN, and PLEN fully symbolic and constrains the symbolic
   domain to the rejection region via `kani::assume(htype != ETHERNET || ptype != IPV4 || hlen != 6 || plen != 4)`.
   Note: the harness name predates decision D-077; despite '_bad_size', it now verifies the
   FULL type-or-size reject contract (hw/proto type AND size) per D-077 — the name is retained
   to avoid cross-artifact churn across delivered stories and architecture docs.

**Sub-property B — GARP detection totality** (`is_gratuitous_arp` or equivalent detection
function, anchors BC-2.16.003):

For ANY `ArpFrame` input, regardless of `operation` value:

1. The gratuitous-ARP classifier returns `true` if and only if `sender_ip == target_ip`.
   This biconditional holds for ALL operation values (1=Request, 2=Reply, and any other u16).
2. The classifier NEVER panics on any `ArpFrame` input.
3. For any `ArpFrame` where `sender_ip != target_ip`, the GARP classifier returns `false`
   regardless of `operation`.

The GARP definition is opcode-agnostic: a GARP is any ARP frame where the sender and target
protocol addresses are equal. Both Request-form GARPs (gratuitous ARP Request, operation==1)
and Reply-form GARPs (gratuitous ARP Reply, operation==2) satisfy this condition. The Kani
harness uses `operation: kani::any()` to cover all 65,536 possible u16 operation values
simultaneously; no `operation == 2` precondition is applied.

**Sub-property C — Binding-table last-write-wins determinism** (proptest, primary anchor
BC-2.16.005; indirectly supports BC-2.16.004 — see Sub-C anchor note):

For any sequence of `ArpFrame` values processed by `ArpAnalyzer::process_arp`:

1. The binding table maps each `[u8; 4]` IP address to exactly one `BindingEntry` at any
   point in time. There are no duplicate IP keys with different MAC values simultaneously
   held in the table (this would be a data structure invariant violation).
2. After processing a sequence ending with frame `f` for IP `ip`, `bindings[ip].mac` equals
   the MAC from `f` (last-write-wins determinism). This is the foundational correctness
   property for spoof detection: the binding must represent the most recent state.
3. Table size is monotonically non-decreasing up to MAX_ARP_BINDINGS, after which LRU
   eviction prevents growth beyond the cap (see Sub-property D).

proptest generates arbitrary sequences of `ArpFrame` values (operation, sender_ip, sender_mac,
target_ip, target_mac) and asserts the invariants above after each frame.

**Sub-property D — MAX_ARP_BINDINGS cap** (Kani, anchors BC-2.16.006):

For any bounded sequence of `ArpFrame` values where all `sender_ip` addresses are distinct:

1. `bindings.len()` NEVER exceeds `MAX_ARP_BINDINGS` at any point during processing.
2. LRU eviction fires exactly when the table would exceed the cap; the eviction removes
   exactly one entry. The selection heuristic targets the entry with minimum `last_seen_ts`,
   but this selection is NOT formally verified — only the `len <= cap` invariant is proven
   by the Kani harness. The `last_seen_ts` field is updated on each access; the minimum-scan
   eviction logic is unit-tested in STORY-113, not Kani-proven.

**`last_seen_ts` initialization at insert time (normative — Sub-D harness implication):**
`insert_binding_lru` does NOT receive a `ts` parameter (see ADR-008 Decision 4 normative
note). When `insert_binding_lru` creates a new `BindingEntry`, it initializes
`last_seen_ts: 0` (the default/zero value). The caller — `process_arp`, which holds
`timestamp_secs: u32` — is responsible for writing `last_seen_ts` on the entry
immediately after `insert_binding_lru` returns. The cap proof (`len <= cap`) is a
purely arithmetic invariant over the number of entries; it holds regardless of the
`last_seen_ts` initialization value or the order in which `process_arp` writes it.
The LRU target-correctness (evicting the least-recently-used entry) is explicitly
**out of Kani scope for VP-024 Sub-D**: the `last_seen_ts: 0` initialization in new
entries and the correctness of the min-scan eviction are unit-tested in STORY-113, not
formally verified. The Kani harness proves only `bindings.len() <= cap` — this is
valid regardless of `last_seen_ts` values.

Kani unwind bound covers sequences up to MAX_ARP_BINDINGS + 2 (to exercise the eviction
boundary at cap and cap+1). Because MAX_ARP_BINDINGS = 65,536 is too large for direct Kani
exhaustion, the harness uses a scaled-down constant `TEST_MAX_ARP_BINDINGS = 8` and verifies
the invariant holds at that scale. The proof is parameterized so the production constant can
be substituted; the cap invariant is an arithmetic property independent of the specific value.

**VP-007 note — mitre.rs atomic update for T0830 and T1557.002:**

The addition of T0830 ("Adversary-in-the-Middle", ICS) and T1557.002 ("Adversary-in-the-Middle:
ARP Cache Poisoning", Enterprise) to `mitre.rs` carries the same VP-007 5-part atomic update
obligation documented in ADR-007 Decision 5 and ADR-005 Decision 4:

1. `technique_info` match arms for `"T0830"` and `"T1557.002"`
2. `SEEDED_TECHNIQUE_IDS` array (23 → 25)
3. `SEEDED_TECHNIQUE_ID_COUNT` constant (23 → 25)
4. `EMITTED_IDS` in `kani_proofs` module (add both IDs)
5. `cargo test mitre` green before PR merges

This VP does NOT verify the `mitre.rs` atomic update — that is VP-007's scope. VP-024's
scope is the ARP pure-core parse functions and the binding-table cap invariant. The
VP-007 obligation is documented here as a cross-reference because the stories that implement
Sub-property A (STORY-111/112 migration + decoder extension) and Sub-property B (STORY-113/114
ArpAnalyzer detections) both touch `mitre.rs` and must not break VP-007's drift guard.

### Sub-property → BC anchor table

| Sub-property | Concern | Anchored BCs |
|---|---|---|
| A — ARP frame extraction parse safety | `extract_arp_frame` panic/OOB-freedom; `None` for any frame failing the combined type-or-size guard (hw_addr_type != ETHERNET OR proto_addr_type != IPV4 OR hw_addr_size != 6 OR proto_addr_size != 4); `Some(correctly-decoded)` for Eth/IPv4 | BC-2.16.001 (ARP Request parse), BC-2.16.002 (ARP Reply parse) |
| B — GARP detection totality | GARP iff sender_ip==target_ip; both op==1 and op==2 forms; no panic | BC-2.16.003 (GARP detection) |
| C — Binding-table determinism | last-write-wins; no duplicate keys; proptest sequence | **Primary:** BC-2.16.005 (binding-table update semantics — last-seen MAC wins for a given IP). **Indirect:** BC-2.16.004 (D1 ARP spoof/rebind escalation, primary STORY-114) depends on the last-write-wins property as its substrate; Sub-C discharges BC-2.16.005 directly and supports BC-2.16.004 indirectly. BC-2.16.004 is NOT in VP-024's formal verified-BC scope. |
| D — MAX_ARP_BINDINGS cap | table.len() never > cap; eviction removes exactly one entry on overflow (min-last_seen_ts heuristic NOT proven — only len<=cap is Kani-proven) | BC-2.16.006 (binding-table bounded resource) |

Additional BC-2.16.007 (D12 L2/L3 sender mismatch detection) is verified by unit test
(stateless single-packet comparison), not by Kani, and is not part of VP-024's formal scope.

> **VERIFIED — IMMUTABLE.** All five Kani harnesses (Sub-A x3, Sub-B x1, Sub-D x1) proved
> `VERIFICATION:- SUCCESSFUL` at Phase F6 (2026-06-16). `verification_lock: true` is set.
> This document is now append-only per VSDD L4 immutability rules. Future changes require
> VP withdrawal and replacement with a new VP-NNN identifier. `proof_file_hash` and
> `verified_at_commit` are pending population against the develop HEAD SHA after the F6
> PR merges — do not guess or backfill speculatively. A `vp-verified-VP-024-2026-06-16`
> tag should be created on the factory-artifacts branch after those fields are populated.

## Source Contract

- **Primary BC:** BC-2.16.001 — ARP Request frame correctly parsed from ArpPacketSlice
- **Primary BC:** BC-2.16.002 — ARP Reply frame correctly parsed from ArpPacketSlice
- **Related BC:** BC-2.16.003 — Gratuitous ARP detection: sender_ip == target_ip classified as GARP
- **Indirectly supported BC:** BC-2.16.004 — ARP spoof detection: IP→MAC rebind emits MEDIUM then HIGH finding (primary STORY-114). Sub-C's last-write-wins proof (BC-2.16.005) is the substrate BC-2.16.004 depends upon; Sub-C discharges BC-2.16.005 directly and supports BC-2.16.004 indirectly. BC-2.16.004 is NOT a VP-024 formally-verified BC.
- **Related BC:** BC-2.16.005 — Binding-table update: last-seen MAC wins for a given IP
- **Related BC:** BC-2.16.006 — Binding-table cap: table never exceeds MAX_ARP_BINDINGS entries
- **Note:** BC-2.16.007 (D12 L2/L3 sender mismatch) is verified by unit test in STORY-113
  (stateless single-packet comparison), not by Kani, and is NOT part of VP-024's formal scope.
- **ADR:** ADR-008, Decision 2 (ArpFrame struct + extract_arp_frame), Decision 4 (ArpAnalyzer
  binding table + detection state layout)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|---|---|---|---|
| Model checking | Kani | Yes — Sub-A: three harnesses: (1) fully-symbolic `[u8;28]` for no-panic; (2) Eth/IPv4-fixed buffer with symbolic OPER+addrs for field correctness; (3) fully-symbolic HTYPE/PTYPE/HLEN/PLEN buffer constrained to the full rejection region (hw_addr_type != ETHERNET OR proto_addr_type != IPV4 OR hw_addr_size != 6 OR proto_addr_size != 4) for None-on-invalid-header negative assertion. Sub-B: symbolic `ArpFrame` with `operation: kani::any()` — covers all 65,536 u16 operation values, opcode-agnostic biconditional. Sub-D: fixed-capacity array surrogate (`insert_binding_lru_array`, `#[cfg(any(kani, test))]`), `#[kani::unwind(12)]`, 9-iteration sequence (cap=TEST_MAX_ARP_BINDINGS=8), scaled cap proof. | All parse outcomes; full GARP domain (opcode-agnostic, 4B×4B IP space); None-on-invalid-header negative path (type AND size rejection); cap-boundary transitions |
| Property-based testing | proptest | Yes — Sub-C: arbitrary `Vec<ArpFrame>` sequences up to 1000 entries; 1000 test cases | Binding-table determinism and no-duplicate-key invariant across arbitrary frame sequences |

Kani is the primary counted tool for VP-024 (per VP-INDEX: Kani). proptest for Sub-C is
counted under proptest. Each VP is counted once; VP-024 is counted under Kani per VP-INDEX
convention (primary/counted tool is Kani).

The ARP pure-core functions have no heap allocation in their hot paths (ArpFrame is a
≤40-byte struct on the stack), no I/O, and no HashMap (the binding table lives in
ArpAnalyzer which is not the target of Sub-A/B).

**Sub-D tool-limitation and surrogate rationale (F6 finding):** The production
`insert_binding_lru` uses `HashMap<[u8;4], BindingEntry>`. Two alternative substrates were
evaluated for the Kani harness:

- `HashMap` with `RandomState` is Kani-incompatible regardless of map size: `RandomState::new()`
  invokes platform RNG, triggering an FFI incompatibility. This is not a scale issue.
- `BTreeMap<[u8;4], BindingEntry>` was the surrogate nominated in Sub-D's draft skeleton
  (`insert_binding_lru_btree`). However, CBMC cannot symbolically execute `std::collections::BTreeMap`:
  even at 3 inserts the model checker exhausts memory with no resolution after 45+ minutes
  across all tested cap scales. The `BTreeMap` surrogate was therefore infeasible.

**Adopted surrogate:** `insert_binding_lru_array` — a fixed-capacity array surrogate
(`#[cfg(any(kani, test))]`) that stores entries in a `[([u8; 4], [u8; 6], u32); N]`
array (plain tuples: IP, MAC, last_seen_ts — no Option wrapper) with a separate
`len: &mut usize` counter tracking occupied entry count, and implements the IDENTICAL
3-branch eviction algorithm used by the production `insert_binding_lru` (HashMap):
1. Update-in-place: if the IP already exists, overwrite MAC and return.
2. Evict-min-last_seen_ts: if the array is at capacity, scan for the entry with the
   minimum `last_seen_ts` value and overwrite it.
3. Append: if capacity is not yet reached, insert into the first vacant slot.

Branch fidelity is confirmed by a `#[cfg(test)]` branch-fidelity test that asserts the
array surrogate matches the production `insert_binding_lru` (HashMap) output for all
three branches across representative inputs.

The cap invariant (`len <= TEST_MAX_ARP_BINDINGS`) is a purely arithmetic property over the
count of occupied entries, independent of the underlying data structure; the proof is valid
for the production `HashMap` by map-implementation-independence: the invariant cannot be
violated unless the eviction logic fails to remove an entry before inserting a new one when
at capacity, and this branch is identical between the surrogate and production functions.

## Proof Harness Skeleton

> Harnesses live in `src/analyzer/arp.rs` under `#[cfg(kani)] mod kani_proofs { use super::*; }`,
> and `src/decoder.rs` for Sub-A (since `extract_arp_frame` lives in the decoder module).
> The pure-core functions are free `fn`s, so harnesses call them directly. These are SPEC
> skeletons; exact wiring is finalized in F4 against the real signatures.

```rust
// ---- In src/decoder.rs #[cfg(kani)] block ----

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // Sub-property A: extract_arp_frame parse safety (BC-2.16.001, BC-2.16.002)
    //
    // Strategy: construct symbolic ArpPacketSlice-like inputs. Because ArpPacketSlice
    // is a slice wrapper (not directly constructible with symbolic bytes via kani::any()),
    // the harness instead calls extract_arp_frame with a symbolic [u8; 28] (min Eth/IPv4
    // ARP length) and verifies no panic occurs. The test coverage is:
    //   - all hw_addr_size / proto_addr_size / hw_addr_type / proto_addr_type combinations
    //   - symbolic MAC (6-byte) and IP (4-byte) address fields
    //
    // NOTE: Exact harness form depends on whether ArpPacketSlice can be constructed
    // in tests (F4 obligation to verify). Alternate approach: test extract_arp_frame via
    // ArpPacketSlice::from_slice on a symbolic 28-byte buffer.

    const ARP_ETH_IPV4_LEN: usize = 28; // fixed size for Ethernet/IPv4 ARP

    #[kani::proof]
    fn verify_extract_arp_frame_safety() {
        let buf: [u8; ARP_ETH_IPV4_LEN] = kani::any();
        // ArpPacketSlice::from_slice returns Result<ArpPacketSlice, LenError>.
        // We only prove the no-panic property for valid slices (Ok path):
        if let Ok(arp_slice) = etherparse::ArpPacketSlice::from_slice(&buf) {
            let outer_mac: Option<[u8; 6]> = kani::any();
            // No panic for any valid ArpPacketSlice and any outer_mac:
            let _ = extract_arp_frame(&arp_slice, outer_mac, ARP_ETH_IPV4_LEN);
        }
        // If from_slice returns Err (buf too short), the harness terminates — no panic path.
        // The assert is the absence of panic (implicit: if we reach here, no unwind occurred).
    }

    // Sub-property A correctness: for well-formed Ethernet/IPv4 ARP, returns Some with correct fields.
    //
    // F4 REACHABILITY OBLIGATION: The `if let Ok(arp_slice)` guard means the field-correctness
    // assertions are only exercised when etherparse::ArpPacketSlice::from_slice accepts the buffer.
    // Because HTYPE/PTYPE/HLEN/PLEN are fixed to valid Ethernet/IPv4 values (0x0001, 0x0800, 6, 4)
    // and the buffer is exactly 28 bytes (the minimum ARP Ethernet/IPv4 wire length), from_slice
    // should always succeed — the Ok arm should always be reachable for this fixed-header buffer.
    // F4 must confirm this via kani::cover! before F6 lock:
    //   kani::cover!(from_slice_result.is_ok(), "Eth/IPv4 ARP buffer must reach Ok arm");
    // If from_slice ever returns Err on a 28-byte well-formed buffer, the field-correctness
    // assertions are vacuously satisfied and the harness provides no coverage. Resolve before lock.
    #[kani::proof]
    fn verify_extract_arp_frame_eth_ipv4_correctness() {
        // Construct a canonical 28-byte Ethernet/IPv4 ARP buffer:
        // Bytes 0-1: HTYPE = 0x0001 (Ethernet)
        // Bytes 2-3: PTYPE = 0x0800 (IPv4)
        // Byte  4:   HLEN  = 6
        // Byte  5:   PLEN  = 4
        // Bytes 6-7: OPER  = symbolic (1=Request, 2=Reply)
        // Bytes 8-13:  sender MAC (symbolic)
        // Bytes 14-17: sender IP  (symbolic)
        // Bytes 18-23: target MAC (symbolic)
        // Bytes 24-27: target IP  (symbolic)
        let mut buf: [u8; 28] = kani::any();
        buf[0] = 0x00; buf[1] = 0x01; // HTYPE = Ethernet
        buf[2] = 0x08; buf[3] = 0x00; // PTYPE = IPv4
        buf[4] = 6;                    // HLEN
        buf[5] = 4;                    // PLEN
        // OPER (bytes 6-7) remains symbolic: covers both Request (1) and Reply (2)

        if let Ok(arp_slice) = etherparse::ArpPacketSlice::from_slice(&buf) {
            let outer_mac: Option<[u8; 6]> = kani::any();
            let result = extract_arp_frame(&arp_slice, outer_mac, 28);
            // For a properly-formed Ethernet/IPv4 ARP, Some must be returned:
            let frame = result.expect("Ethernet/IPv4 ARP must produce Some(ArpFrame)");
            // Field correctness:
            assert!(frame.sender_mac == <[u8; 6]>::try_from(&buf[8..14]).unwrap());
            assert!(frame.sender_ip  == <[u8; 4]>::try_from(&buf[14..18]).unwrap());
            assert!(frame.target_mac == <[u8; 6]>::try_from(&buf[18..24]).unwrap());
            assert!(frame.target_ip  == <[u8; 4]>::try_from(&buf[24..28]).unwrap());
            let oper = u16::from_be_bytes([buf[6], buf[7]]);
            assert!(frame.operation == oper);
        }
        // Do NOT ship to F6 lock without resolving the F4 reachability obligation above.
    }

    // Sub-property A negative assertion: None returned when the combined reject guard fires —
    // i.e., when ANY of the four conditions at decoder.rs:312-315 is true:
    //   hw_addr_type() != ETHERNET  OR  proto_addr_type() != IPV4
    //   OR  hw_addr_size() != 6     OR  proto_addr_size() != 4
    // (BC-2.16.001/BC-2.16.002 — the parse-safety postcondition requires graceful None
    // for any frame failing type or size validation; panic is forbidden.)
    //
    // Note: D11 finding-emission (BC-2.16.009 PC3a-3d) is separately unit-tested in
    // STORY-113; this harness only proves extract_arp_frame returns None (no panic) when
    // the reject guard fires. The finding-emission path is out of scope for VP-024 formal
    // proofs. Decision D-077 added the type-rejection branches (HTYPE != ETHERNET,
    // PTYPE != IPV4); this harness covers them alongside the pre-existing size branches.
    //
    // Note: the harness name predates decision D-077; despite '_bad_size', it now verifies
    // the FULL type-or-size reject contract (hw/proto type AND size) per D-077 — the name
    // is retained to avoid cross-artifact churn across delivered stories and architecture docs.
    //
    // Strategy: construct a 28-byte buffer with ALL of HTYPE, PTYPE, HLEN, PLEN left
    // fully symbolic (kani::any()). Use kani::assume to restrict the symbolic domain to
    // the rejection region:
    //   HTYPE != ETHERNET  OR  PTYPE != IPV4  OR  HLEN != 6  OR  PLEN != 4
    // Assert that extract_arp_frame returns None (graceful rejection, no panic).
    // This covers type-only rejection (e.g. HTYPE=0x0006/Token Ring, valid HLEN/PLEN),
    // size-only rejection (HTYPE=Ethernet, PTYPE=IPv4, bad HLEN/PLEN), and mixed cases.
    #[kani::proof]
    fn verify_extract_arp_frame_none_on_bad_size() {
        let mut buf: [u8; 28] = kani::any();
        // HTYPE bytes (0-1), PTYPE bytes (2-3), HLEN byte (4), PLEN byte (5) are all
        // left symbolic (kani::any() applies to the whole buf). Only the reject-region
        // constraint is applied via kani::assume below.
        let htype = u16::from_be_bytes([buf[0], buf[1]]);
        let ptype = u16::from_be_bytes([buf[2], buf[3]]);
        let hlen = buf[4];
        let plen = buf[5];
        // Restrict to the full rejection region — mirrors the guard at decoder.rs:312-315:
        //   hw_addr_type != ETHERNET (0x0001)  OR  proto_addr_type != IPV4 (0x0800)
        //   OR  hw_addr_size != 6              OR  proto_addr_size != 4
        kani::assume(htype != 0x0001 || ptype != 0x0800 || hlen != 6 || plen != 4);

        // F4 OBLIGATION: Confirm that a buffer satisfying the rejection-region assume
        // actually reaches the Ok(arp_slice) arm so the is_none() assertion is exercised
        // (not vacuously satisfied). If etherparse::ArpPacketSlice::from_slice rejects the
        // buffer with a length error on the Err path, assert!(result.is_none()) is never
        // reached and the harness passes vacuously without testing the None postcondition.
        // F4 must either:
        //   (a) confirm that from_slice accepts 28-byte buffers with arbitrary HTYPE/PTYPE/
        //       HLEN/PLEN (i.e., from_slice does not validate header fields against actual
        //       payload length for a fixed 28-byte packet), making the Ok arm reachable; OR
        //   (b) restructure the harness to assert reachability explicitly, e.g.:
        //       kani::cover!(from_slice_result.is_ok(), "reject-region buffer reaches Ok arm");
        //       and then assert is_none() unconditionally inside the Ok branch.
        // Do NOT ship this harness to F6 without resolving the vacuous-satisfiability risk.
        if let Ok(arp_slice) = etherparse::ArpPacketSlice::from_slice(&buf) {
            let outer_mac: Option<[u8; 6]> = kani::any();
            let result = extract_arp_frame(&arp_slice, outer_mac, 28);
            // When the combined reject guard fires (type OR size mismatch), must return None.
            assert!(result.is_none(),
                "extract_arp_frame must return None when hw_addr_type != ETHERNET \
                 or proto_addr_type != IPV4 or HLEN != 6 or PLEN != 4");
        }
        // If from_slice rejects the buffer, no-panic is satisfied implicitly.
    }
}

// ---- In src/analyzer/arp.rs #[cfg(kani)] block ----

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // Sub-property B: GARP detection totality (BC-2.16.003)
    //
    // For any ArpFrame: is_gratuitous_arp(&frame) == (frame.sender_ip == frame.target_ip)
    // No panic for any input.

    #[kani::proof]
    fn verify_classify_garp_total() {
        let frame = ArpFrame {
            operation:       kani::any(),
            sender_mac:      kani::any(),
            sender_ip:       kani::any(),
            target_mac:      kani::any(),
            target_ip:       kani::any(),
            outer_src_mac:   kani::any(),
            packet_len:      kani::any(),
        };
        // No panic: pure boolean function over ArpFrame fields.
        let is_garp = is_gratuitous_arp(&frame);
        // Biconditional: GARP iff sender_ip == target_ip.
        assert!(is_garp == (frame.sender_ip == frame.target_ip));
    }

    // Sub-property D: MAX_ARP_BINDINGS cap (BC-2.16.006)
    //
    // Scaled test: use TEST_MAX_ARP_BINDINGS = 8 as a surrogate cap.
    // Process 9 frames (cap + 1) with distinct IPs; assert table.len() <= 8 after each.
    //
    // NOTE ON SUBSTRATE (F6 finding — array surrogate replaces BTreeMap skeleton):
    // The production `insert_binding_lru` uses `HashMap<[u8;4], BindingEntry>`.
    // HashMap with RandomState is Kani-incompatible (RandomState::new() calls platform RNG,
    // FFI incompatibility regardless of scale). The draft skeleton nominated BTreeMap as
    // surrogate (`insert_binding_lru_btree`), but CBMC cannot symbolically execute
    // std::collections::BTreeMap — exhausts memory at 3 inserts after 45+ minutes with no
    // resolution at any cap scale.
    //
    // ADOPTED SURROGATE: insert_binding_lru_array — a fixed-capacity array surrogate
    // `#[cfg(any(kani, test))]` that stores entries in [([u8; 4], [u8; 6], u32); N]
    // (plain tuples: IP, MAC, last_seen_ts — no Option wrapper) with a separate
    // len: &mut usize counter tracking occupied entry count, and implements the
    // IDENTICAL 3-branch eviction algorithm:
    //   1. Update-in-place (IP exists → overwrite MAC).
    //   2. Evict-min-last_seen_ts (at capacity → scan for min last_seen_ts, overwrite).
    //   3. Append (capacity not reached → insert in first vacant slot).
    // Branch fidelity confirmed by a #[cfg(test)] branch-fidelity test asserting
    // surrogate matches production insert_binding_lru (HashMap) on all three branches.
    //
    // SOUNDNESS: The cap invariant (len <= N) is a purely arithmetic property over occupied
    // entry count, independent of the data structure. The proof is valid for the production
    // HashMap by map-implementation-independence: the invariant can only be violated if
    // eviction fails to remove an entry before inserting when at capacity — this branch is
    // identical in surrogate and production. VP-024's own proof-method acknowledges this:
    // "the cap invariant is a purely arithmetic property independent of map implementation."
    //
    // NOTE: insert_binding_lru has no ts parameter. process_arp (the caller, holding
    // timestamp_secs: u32) writes last_seen_ts on every observation; insert_binding_lru
    // only reads last_seen_ts during the eviction scan. See ADR-008 Decision 4 normative note.

    const TEST_MAX_ARP_BINDINGS: usize = 8;

    #[kani::proof]
    #[kani::unwind(12)] // TEST_MAX_ARP_BINDINGS + 3 overhead; covers 0..=8 (9 iterations)
    fn verify_binding_table_cap() {
        const CAP: usize = TEST_MAX_ARP_BINDINGS; // 8
        // insert_binding_lru_array is a #[cfg(any(kani, test))] fixed-capacity array
        // surrogate implementing the identical 3-branch eviction algorithm as production
        // insert_binding_lru (HashMap). See surrogate rationale in NOTE ON SUBSTRATE above.
        // Signature: entries: &mut [([u8; 4], [u8; 6], u32); N], len: &mut usize,
        //            ip: [u8; 4], mac: [u8; 6], cap: usize
        // Entries are (ip, mac, last_seen_ts) tuples; len tracks occupied count separately.
        let mut entries: [([u8; 4], [u8; 6], u32); CAP] = [([0u8; 4], [0u8; 6], 0u32); CAP];
        let mut len: usize = 0;
        // Process TEST_MAX_ARP_BINDINGS + 1 frames with distinct IPs.
        // After each insertion, assert the cap holds.
        let mut i = 0u8;
        while i <= (CAP as u8) {
            let ip: [u8; 4] = [0, 0, 0, i]; // distinct IP per iteration
            let mac: [u8; 6] = kani::any();
            insert_binding_lru_array(&mut entries, &mut len, ip, mac, CAP);
            assert!(len <= CAP);
            i += 1;
        }
    }
}
```

### Symbolic input construction summary

| Sub-property | Harness | Symbolic input | Bound / unwind | Assertions |
|---|---|---|---|---|
| A (safety) | `verify_extract_arp_frame_safety` | `[u8; 28]` fully symbolic; only valid slices tested (from_slice Ok path) | none (no loop) | no panic |
| A (correctness) | `verify_extract_arp_frame_eth_ipv4_correctness` | `[u8; 28]` with HTYPE/PTYPE/HLEN/PLEN fixed; OPER+addrs symbolic | none | Some returned; all field values exact |
| A (negative) | `verify_extract_arp_frame_none_on_bad_size` | `[u8; 28]` fully symbolic (HTYPE, PTYPE, HLEN, PLEN all symbolic); constrained to rejection region via `kani::assume(htype != 0x0001 \|\| ptype != 0x0800 \|\| hlen != 6 \|\| plen != 4)` | none | `result.is_none()` — no panic; graceful None for type OR size mismatch (name predates D-077; covers full type+size reject contract per v1.9 clarification) |
| B (totality) | `verify_classify_garp_total` | symbolic `ArpFrame` (all fields symbolic, `operation: kani::any()`) | none (straight-line) | `is_garp == (sender_ip == target_ip)` for ALL operation values |
| D (cap) | `verify_binding_table_cap` | deterministic IPs (0..=8); symbolic MACs; array surrogate `insert_binding_lru_array` (`#[cfg(any(kani, test))]`; BTreeMap infeasible — CBMC OOM; see Proof Method); entries: `[([u8; 4], [u8; 6], u32); CAP]`, separate `len: usize` counter | `#[kani::unwind(12)]` | `len <= CAP` (=8) after every insert |

**Sub-property C proptest sketch:**

The sketch uses three test affordances that MUST be declared in `src/analyzer/arp.rs`
under `#[cfg(test)]` (or `#[cfg(any(test, proptest))]`) before F4 implementation:

1. `ArpAnalyzer::new_for_test() -> ArpAnalyzer` — constructs an `ArpAnalyzer` with
   default thresholds and empty tables, without requiring CLI arguments. This is a
   test-only constructor; the production path uses `ArpAnalyzer::new(cfg)` or equivalent.
2. `ArpAnalyzer::process_arp_for_test(&mut self, frame: &ArpFrame, ts: u32) -> Vec<Finding>` —
   a `#[cfg(test)]`-gated thin wrapper over `process_arp` that supplies a default
   timestamp (e.g., `0u32`) so proptest callers do not need a clock source. Note:
   the canonical `process_arp` signature is `process_arp(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding>` — `ts` is mandatory; `process_arp_for_test` wraps
   it with a fixed `ts = 0`.
3. `bindings` field test-visibility — `#[cfg(test)] pub(crate) bindings: HashMap<[u8;4], BindingEntry>` OR a `#[cfg(test)]`-gated `fn bindings_snapshot(&self) -> &HashMap<[u8;4], BindingEntry>` accessor. Direct field access (`analyzer.bindings.get(ip)`) requires test-visibility; a snapshot accessor is the preferred non-invasive approach.

These test affordances are ADR-008 Decision 4 scope extensions. PO must carry them in
BC-2.16.005 test notes or a supplemental test-infrastructure BC if needed.

```rust
// proptest in src/analyzer/arp.rs tests module
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_binding_table_last_write_wins(
        frames in proptest::collection::vec(
            (any::<[u8; 4]>(), any::<[u8; 6]>(), any::<u16>()),
            0..1000usize
        )
    ) {
        // new_for_test() is a #[cfg(test)] constructor — see test affordances above.
        let mut analyzer = ArpAnalyzer::new_for_test();
        let mut last_mac_for_ip: std::collections::HashMap<[u8; 4], [u8; 6]> =
            std::collections::HashMap::new();

        for (ip, mac, oper) in &frames {
            let frame = ArpFrame {
                operation: *oper, sender_ip: *ip, sender_mac: *mac,
                target_ip: [0u8; 4], target_mac: [0u8; 6],
                outer_src_mac: None, packet_len: 42,
            };
            // process_arp_for_test supplies ts=0; wraps process_arp(&frame, 0u32).
            // See test affordances above — arity is (frame, ts); no zero-arg form exists.
            let _ = analyzer.process_arp_for_test(&frame, 0u32);
            last_mac_for_ip.insert(*ip, *mac);
        }

        // Last-write-wins: for every IP in last_mac_for_ip that is still in the binding
        // table (not evicted), its MAC equals the last MAC we sent.
        // bindings_snapshot() is a #[cfg(test)] accessor — see test affordances above.
        for (ip, expected_mac) in &last_mac_for_ip {
            if let Some(entry) = analyzer.bindings_snapshot().get(ip) {
                prop_assert_eq!(&entry.mac, expected_mac,
                    "last-write-wins violation for ip {:?}", ip);
            }
            // If evicted (table was at cap), the absence is acceptable.
        }

        // No-duplicate-key invariant: HashMap guarantees this structurally.
        // Explicit check for documentation:
        let snap = analyzer.bindings_snapshot();
        let unique_ips: std::collections::HashSet<[u8; 4]> =
            snap.keys().cloned().collect();
        prop_assert_eq!(unique_ips.len(), snap.len(),
            "duplicate IP key detected in binding table");
    }
}
```

**Loop / unwind bounds:** Sub-A/B have no loops. Sub-D uses `#[kani::unwind(12)]` covering
the 9-iteration sequence (0..=8) plus overhead. Sub-C is proptest (no Kani unwind concern).

**Expected result for Sub-A/B/D Kani harnesses:** `VERIFICATION:- SUCCESSFUL`.
**Expected result for Sub-C proptest:** all 1000 property-test cases pass.

## Feasibility Assessment

| Factor | Assessment | Notes |
|---|---|---|
| Input space size | Bounded | Sub-A: `[u8; 28]` fully symbolic (fast — 28 bytes, straight-line); Sub-B: symbolic `ArpFrame` (≤40 bytes, straight-line); Sub-D: 9-iteration loop, BTreeMap with 8 entries maximum |
| Proof complexity | Low | Sub-A/B: pure field-extraction and boolean comparisons; Sub-D: bounded loop with a simple len≤N assertion |
| Tool support | High | Sub-A/B: no HashMap/RandomState → no Kani FFI issue; Sub-D uses fixed-capacity array surrogate `insert_binding_lru_array` (HashMap/RandomState Kani FFI incompatible; BTreeMap OOM in CBMC — see Proof Method surrogate rationale); Sub-C is proptest (no Kani constraint) |
| Estimated proof time | < 2 seconds per harness | Analogous to VP-022/VP-023 harnesses; no recursion, no unbounded loops |
| Precedent | VP-022 (Modbus), VP-023 (DNP3) | Both ran SUCCESSFUL in < 1 second each; ARP harnesses are structurally simpler |

## Source Location (forward references — implemented in F4 TDD)

- `src/decoder.rs` — `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
- `src/decoder.rs` — `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }`
- `src/decoder.rs` — `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`
- `src/analyzer/arp.rs` — `fn is_gratuitous_arp(frame: &ArpFrame) -> bool`
- `src/analyzer/arp.rs` — `fn insert_binding_lru(bindings: &mut HashMap<[u8;4], BindingEntry>, ip: [u8;4], mac: [u8;6], cap: usize)` (production type; no `ts` parameter — `last_seen_ts` is written by `process_arp` on every observation; `insert_binding_lru` reads it only during eviction scan; see ADR-008 Decision 4 normative note)
- `src/analyzer/arp.rs` — `fn insert_binding_lru_array(entries: &mut [([u8; 4], [u8; 6], u32); N], len: &mut usize, ip: [u8;4], mac: [u8;6], cap: usize)` (`#[cfg(any(kani, test))]` fixed-capacity array surrogate used by Sub-D Kani harness; entries are plain (IP, MAC, last_seen_ts) tuples — no Option wrapper; separate `len` counter tracks occupied entry count; BTreeMap surrogate from draft skeleton proved CBMC-infeasible — OOM at 3 inserts; see Proof Method surrogate rationale. Branch-fidelity test confirms identical eviction algorithm.)
- `src/analyzer/arp.rs` — `pub struct ArpAnalyzer { bindings, storm_counters, spoof_threshold, storm_rate, ... }`

## Lifecycle

| Event | Date | Actor |
|---|---|---|
| Created (draft, F2 spec evolution) | 2026-06-12 | architect |
| Proof harness committed | F4 (develop branch) | formal-verifier |
| Proof first passed | 2026-06-16 (F6) | formal-verifier |
| Locked (VERIFIED) | 2026-06-16 (F6 gate) | spec-steward |
