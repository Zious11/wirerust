---
document_type: story
story_id: STORY-113
epic_id: E-16
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: f3
points: 13
priority: P0
depends_on: [STORY-112]
blocks: [STORY-114]
behavioral_contracts:
  - BC-2.16.003
  - BC-2.16.005
  - BC-2.16.006
  - BC-2.16.007
  - BC-2.16.009
  - BC-2.16.010
  - BC-2.16.011
verification_properties: [VP-024]
tdd_mode: strict
target_module: analyzer/arp
subsystems: [SS-16]
estimated_days: 5
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: all 7 BCs authored 2026-06-12. Primary owner of BC-2.16.010 (summarize keys introduced here).
# VP-024 Sub-B (verify_classify_garp_total Kani), Sub-C (test_BC_2_16_005_binding_table_last_write_wins proptest), Sub-D (verify_binding_table_cap Kani) land here.
# NOTE: D1 spoof EMISSION (BC-2.16.004) is NOT in this story. Binding table infrastructure is built here, but D1 findings are emitted in STORY-114.
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.003.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.005.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.006.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.007.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.011.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
input-hash: "f35bcfc"
---

# STORY-113: ArpAnalyzer Full Implementation — Binding Table, GARP (D2), D11, D12, summarize(), --arp Flag, VP-024 Sub-B/C/D

## Narrative

- **As a** ICS/OT security analyst using wirerust
- **I want** `ArpAnalyzer` to maintain a bounded binding table (IP→MAC with LRU eviction), detect Gratuitous ARP (D2) via `is_gratuitous_arp`, detect D11 malformed ARP findings, detect D12 L2/L3 sender-MAC mismatch, expose a `summarize()` method returning all eleven canonical summary keys, and be activated by the `--arp` CLI flag
- **So that** all stateless and stateful ARP detection capabilities are operational (except D1 spoof escalation and MITRE attribution, which land in STORY-114), and the VP-024 Sub-B/C/D formal properties are verified by Kani and proptest

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.003 | Gratuitous ARP Detection — sender_ip == target_ip Classified as GARP |
| BC-2.16.005 | Binding-Table Update — Last-Seen MAC Wins for a Given IP |
| BC-2.16.006 | Binding-Table Cap — Table Never Exceeds MAX_ARP_BINDINGS via LRU Eviction |
| BC-2.16.007 | D12 L2/L3 Sender Mismatch — Ethernet Src MAC != ARP Sender HW Addr |
| BC-2.16.009 | D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding |
| BC-2.16.010 | ArpAnalyzer::summarize() Returns AnalysisSummary with Required Keys (11 Keys) — PRIMARY OWNER |
| BC-2.16.011 | --arp CLI Flag Gates ARP Security Analysis |

## Scope Boundary: D1 Deferred to STORY-114

**This story does NOT emit D1 spoof findings.** The binding table is built and updated (BC-2.16.005, BC-2.16.006) — IPs and MACs are tracked — but when a rebind is detected, STORY-113 only records the state update (increments `rebind_count`). D1 finding emission, the MEDIUM→HIGH escalation logic (BC-2.16.004), the GARP-that-conflicts escalation (BC-2.16.014), and MITRE T0830/T1557.002 attribution (STORY-114's VP-007 atomic update) all land in STORY-114. The `spoof_findings` summary key will be 0 after STORY-113; it becomes non-zero after STORY-114.

## VP-024 Sub-B, Sub-C, Sub-D

**verify_classify_garp_total (Sub-B):** Kani harness. Symbolic `ArpFrame` with all fields symbolic; assert `is_gratuitous_arp(frame) == (frame.sender_ip == frame.target_ip)` for all symbolic inputs. Never panics. Targets the biconditional invariant in BC-2.16.003 Invariant 1.

**test_BC_2_16_005_binding_table_last_write_wins (Sub-C):** proptest. Arbitrary `Vec<(IP, MAC, opcode)>` sequences up to 1000 entries; after processing all frames, assert `bindings[ip].mac == mac_from_last_frame_with_that_ip` for every IP. Uses `new_for_test()`, `process_arp_for_test()`, `bindings_snapshot()` test affordances (BC-2.16.005 Architecture Anchors, ADR-008 Decision 4 extensions).

> **Sub-C anchor adjudication (PO, 2026-06-13):** VP-024 Sub-C (test_BC_2_16_005_binding_table_last_write_wins proptest) has BC-2.16.005 as its primary anchor. BC-2.16.004 (D1 spoof escalation, STORY-114) indirectly depends on this last-write-wins substrate; Sub-C does not formally discharge BC-2.16.004.

**verify_binding_table_cap (Sub-D):** Kani harness using `insert_binding_lru_array` (array surrogate gated under `#[cfg(any(kani, test))]`; signature: `entries: &mut [([u8; 4], [u8; 6], u32); CAP], len: &mut usize, ip, mac, cap`). The array surrogate is required because CBMC/Kani cannot model `HashMap` symbolically; it reproduces the production 3-branch eviction logic over a fixed-size array. `TEST_MAX_ARP_BINDINGS = 8`; 9-iteration loop (cap+1); `#[kani::unwind(12)]`; assert `len <= 8` after each insert. Sanctioned by VP-024 map-implementation-independence (VP-024 v2.2 + arp-architecture-delta v1.17). Production type remains `HashMap`.

## Acceptance Criteria

### AC-001 (traces to BC-2.16.003 postcondition 1/2 — is_gratuitous_arp biconditional)
`fn is_gratuitous_arp(frame: &ArpFrame) -> bool` returns `true` if and only if
`frame.sender_ip == frame.target_ip` (byte-wise). Returns `false` if and only if
`frame.sender_ip != frame.target_ip`. No other field affects the return value.
- **Test:** `test_BC_2_16_003_is_gratuitous_arp_true_when_sender_eq_target_ip`, `test_BC_2_16_003_is_gratuitous_arp_false_when_sender_ne_target_ip`

### AC-002 (traces to BC-2.16.003 postcondition 3 — opcode agnosticism)
`is_gratuitous_arp` returns `true` for both op=1 (Request GARP) and op=2 (Reply GARP) when
sender_ip == target_ip. The function does not inspect the operation field.
- **Test:** `test_BC_2_16_003_is_gratuitous_arp_opcode_agnostic`

### AC-003 (traces to BC-2.16.003 postcondition 5 — GARP finding at LOW/Anomaly)
When `is_gratuitous_arp` returns `true` AND there is no binding conflict (benign GARP),
`process_arp` emits one `Finding` with `confidence: LOW`, `finding_type: Anomaly`,
description indicating Gratuitous ARP, and `mitre_techniques: []` (empty). Per the
clarified §3.3 D2 rule: benign GARP emits no MITRE techniques; T0830 and T1557.002 are
attached only on GARP-conflicts (which escalate to D1 and are handled by STORY-114 via
BC-2.16.014). This is consistent with the Scope Boundary note above and Architecture
Compliance Rule 4.
- **Test:** `test_BC_2_16_003_process_arp_garp_emits_low_anomaly_finding` (asserts `mitre_techniques` is empty)

### AC-004 (traces to BC-2.16.003 postcondition 6 — one GARP finding per GARP frame)
Exactly one GARP finding is emitted per frame where `is_gratuitous_arp` returns `true`.
There is no cross-frame one-shot guard for GARP (unlike D1 and D3). Each GARP frame
produces its own finding.
- **Test:** `test_BC_2_16_003_process_arp_garp_emits_per_frame` (10 consecutive GARP frames → 10 findings)

### AC-005 (traces to BC-2.16.005 postcondition 1 — last-write-wins binding update)
After `process_arp(frame, ts)` completes, `bindings[frame.sender_ip].mac == frame.sender_mac`.
For non-zero, non-broadcast sender IPs, the binding table holds the MAC from the most recently
processed frame with that sender IP.
- **Test:** `test_BC_2_16_005_binding_table_last_write_wins_basic`

### AC-006 (traces to BC-2.16.005 postcondition 4/invariant 3 — first-time observation initializes entry)
When a sender_ip is seen for the first time, a new binding is inserted with `rebind_count = 0`,
`first_rebind_ts = None`, `spoof_high_emitted = false`. No finding is emitted on first observation.
- **Test:** `test_BC_2_16_005_binding_first_observation_no_finding`

### AC-007 (traces to BC-2.16.005 invariant 5 — zero and broadcast sender IPs filtered)
`process_arp` does NOT insert a binding entry for `sender_ip = [0, 0, 0, 0]` (zero) or
`sender_ip = [255, 255, 255, 255]` (broadcast). No spoof finding is emitted for these values.
- **Test:** `test_BC_2_16_005_binding_zero_sender_ip_filtered`, `test_BC_2_16_005_binding_broadcast_sender_ip_filtered`

### AC-008 (traces to BC-2.16.006 postcondition 2 — binding table cap enforced)
`bindings.len()` NEVER exceeds `MAX_ARP_BINDINGS = 65_536` at any point during processing.
When a new IP would cause overflow, `insert_binding_lru` evicts the entry with the minimum
`last_seen_ts` before inserting.
- **Test:** `test_BC_2_16_006_binding_table_cap_enforced` (insert 65,537 distinct IPs; assert `bindings.len() == 65_536` after each insert past cap)

### AC-009 (traces to BC-2.16.007 postcondition 1 — D12 mismatch finding emitted)
When `frame.outer_src_mac == Some(eth_mac)` and `eth_mac != frame.sender_mac`, `process_arp`
emits one `Finding` with `confidence: MEDIUM`, `finding_type: Anomaly`, description indicating
L2/L3 sender MAC mismatch, `mitre_techniques: []` (catalog not seeded yet; T0830 and T1557.002
are attached in STORY-114 wave 43, co-committed with the VP-007 5-part atomic update per
BC-2.16.007's cross-story delivery note — analogous to the D1 deferral in STORY-113's 'Crucial boundary' note and
the BC-2.16.010 storm_findings value-wiring). Evidence includes eth_mac, arp_sender_mac, and
sender_ip.
- **Test:** `test_BC_2_16_007_d12_mismatch_emits_medium_finding` (asserts `mitre_techniques` is empty at wave 42)

### AC-010 (traces to BC-2.16.007 postcondition 4/5 — D12 skipped for None or matching MACs)
When `frame.outer_src_mac == None`, no D12 finding is emitted. When
`frame.outer_src_mac == Some(mac)` and `mac == frame.sender_mac`, no D12 finding is emitted.
D12 is stateless: no binding table state is updated.
- **Test:** `test_BC_2_16_007_d12_skipped_when_outer_src_mac_none`, `test_BC_2_16_007_d12_skipped_when_macs_match`

### AC-011 (traces to BC-2.16.009 postcondition 3 — D11 malformed ARP finding emitted)
When `main.rs` calls `arp_analyzer.record_malformed(packet_len)` (or equivalent mechanism)
after receiving `Err("Non-Ethernet/IPv4 ARP frame")` from `decode_packet`, `ArpAnalyzer`
emits one `Finding` with `confidence: LOW`, `finding_type: Anomaly`, description indicating
malformed ARP frame, `mitre_techniques: []` (empty — T0814 withheld per DF-VALIDATION-001),
and evidence including the packet_len.
- **Test:** `test_BC_2_16_009_d11_malformed_arp_emits_low_finding`

### AC-012 (traces to BC-2.16.009 postcondition 4 — malformed counters)
`frames_analyzed` is NOT incremented for malformed frames. `malformed_frames` increments
unconditionally on every malformed frame event (even when `--arp` is absent). `malformed_findings`
increments only when `--arp` is active (one-shot with each `record_malformed` call under the `--arp` gate).
- **Test:** `test_BC_2_16_009_d11_malformed_counter_semantics`

### AC-013 (traces to BC-2.16.010 postcondition 1 — all eleven summary keys present)
`ArpAnalyzer::summarize()` returns an `AnalysisSummary` containing exactly these eleven keys
(exact string names): `"frames_analyzed"`, `"request_count"`, `"reply_count"`, `"other_opcode_count"`, `"bindings_tracked"`, `"spoof_findings"`, `"garp_findings"`, `"storm_findings"`, `"mismatch_findings"`, `"malformed_findings"`, `"malformed_frames"`. All values are `u64` (or compatible numeric). All keys present with value 0 when no frames processed (BC-2.16.010 EC-001).
- **Test:** `test_BC_2_16_010_summarize_zero_frames_all_eleven_keys_zero`, `test_BC_2_16_010_summarize_key_names_exact`

### AC-014 (traces to BC-2.16.010 invariant 3 — reconciliation invariant)
`request_count + reply_count + other_opcode_count == frames_analyzed` holds after every
`process_arp` call. Malformed frames (incrementing `malformed_frames`) do NOT contribute to
`frames_analyzed`.
- **Test:** `test_BC_2_16_010_summarize_reconciliation_invariant`

### AC-015 (traces to BC-2.16.011 postconditions 1–4 — --arp absent: no analysis)
When `args.arp` is false (flag absent), `process_arp` is NOT called on any ARP frame in
`main.rs`. No ARP findings are emitted. No ARP summary is appended to `analyzer_summaries`.
- **Test:** `test_BC_2_16_011_main_arp_flag_absent_no_findings_no_summary` (integration, tests/bc_2_16_story113_arp_tests.rs)

### AC-016 (traces to BC-2.16.011 postconditions 5–8 — --arp present: analysis active)
When `args.arp` is true, `process_arp` is called for every `DecodedFrame::Arp` frame.
`ArpAnalyzer::summarize()` is called at end of capture and appended to `analyzer_summaries`
(following the Modbus/DNP3 pattern in `main.rs`). The `--arp` flag is declared as
`#[arg(long)] arp: bool` on `Commands::Analyze` in `src/cli.rs`.
- **Test:** `test_BC_2_16_011_main_arp_flag_present_summarize_appended` (integration, tests/bc_2_16_story113_arp_tests.rs)

### AC-017 (traces to BC-2.16.003/VP-024 Sub-B — verify_classify_garp_total Kani harness)
`verify_classify_garp_total` Kani harness asserts the biconditional `is_gratuitous_arp(frame) == (frame.sender_ip == frame.target_ip)` for all symbolic `ArpFrame` inputs. Reports `VERIFICATION:- SUCCESSFUL`.
- **Kani:** Run at F6 formal-hardening gate.

### AC-018 (traces to BC-2.16.005/VP-024 Sub-C — test_BC_2_16_005_binding_table_last_write_wins proptest)
`test_BC_2_16_005_binding_table_last_write_wins` proptest verifies that for any arbitrary sequence of
`Vec<ArpFrame>` up to 1000 entries, `bindings[ip].mac == mac_from_last_frame_with_that_ip`
for every IP in the sequence. Uses `new_for_test()`, `process_arp_for_test()`, `bindings_snapshot()`.
- **Test:** `test_BC_2_16_005_binding_table_last_write_wins` proptest in `src/analyzer/arp.rs` tests module; runs at `cargo test`.

### AC-019 (traces to BC-2.16.006/VP-024 Sub-D — verify_binding_table_cap Kani harness)
`verify_binding_table_cap` Kani harness using `insert_binding_lru_array` (array surrogate,
`#[cfg(any(kani, test))]`-gated; signature: `entries: &mut [([u8; 4], [u8; 6], u32); CAP], len: &mut usize, ip, mac, cap`); array surrogate used because CBMC/Kani cannot model `HashMap` symbolically; reproduces the production 3-branch eviction logic; sanctioned by VP-024 map-implementation-independence. `TEST_MAX_ARP_BINDINGS = 8`; 9-iteration loop; `#[kani::unwind(12)]`; assert `len <= 8` after each call. Production type remains `HashMap`.
- **Kani:** Run at F6 formal-hardening gate.

### AC-020 (traces to BC-2.16.007 invariant 3 / EC-004 — D12 and D2 co-emit on a single frame)
A single ARP frame where `sender_ip == target_ip` (GARP condition) AND `outer_src_mac != sender_mac` (D12 mismatch condition) causes `process_arp` to emit exactly two findings in the same call: one D12 `Finding` (MEDIUM/Anomaly, `mitre_techniques: []`) and one D2 GARP `Finding` (LOW/Anomaly, `mitre_techniques: []`). The two detections are independent and both fire. D1 co-emit (GARP-that-conflicts) is out of scope for this story — that escalation requires the binding conflict check and is STORY-114's responsibility.
- **Test:** `test_BC_2_16_007_d12_and_garp_coemit_on_single_frame`

### AC-021 (traces to BC-2.16.005 postcondition 5 — same-MAC re-observation advances last_seen_ts)
When `process_arp` processes a frame where `sender_ip` already has a binding entry AND `frame.sender_mac == bindings[sender_ip].mac` (no MAC change — no rebind), the binding entry's `last_seen_ts` is still updated to the current `timestamp_secs`. `rebind_count` remains unchanged. This ensures LRU eviction correctly identifies the most-recently-seen entry regardless of whether a rebind occurred.
- **Test:** `test_BC_2_16_005_binding_same_mac_touches_last_seen_ts`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `pub struct ArpAnalyzer` (full impl) | `src/analyzer/arp.rs` | Pure core (stateful) |
| `struct BindingEntry { mac, rebind_count, first_rebind_ts, spoof_high_emitted, last_seen_ts }` | `src/analyzer/arp.rs` | Data type |
| `struct StormCounter { count_in_window, window_start_ts, storm_emitted }` | `src/analyzer/arp.rs` | Data type (stub; fields only — D3 logic in STORY-115) |
| `fn is_gratuitous_arp(frame: &ArpFrame) -> bool` | `src/analyzer/arp.rs` | Pure core (VP-024 Sub-B Kani target) |
| `fn insert_binding_lru(bindings: &mut HashMap<[u8;4], BindingEntry>, ip, mac, cap)` | `src/analyzer/arp.rs` | Pure core (VP-024 Sub-D production substrate) |
| `fn insert_binding_lru_array(entries: &mut [([u8;4], [u8;6], u32); CAP], len: &mut usize, ip, mac, cap)` | `src/analyzer/arp.rs` `#[cfg(any(kani, test))]` | Array surrogate for Sub-D (CBMC/Kani cannot model HashMap symbolically; reproduces production 3-branch eviction; sanctioned by VP-024 map-implementation-independence) |
| `impl ArpAnalyzer::new()` (parameterless — `spoof_threshold` added in STORY-114; `storm_rate` added in STORY-115) | `src/analyzer/arp.rs` | Constructor |
| `impl ArpAnalyzer::process_arp` (full: D2/D11/D12 + binding update) | `src/analyzer/arp.rs` | Pure core (stateful) |
| `impl ArpAnalyzer::record_malformed(packet_len)` | `src/analyzer/arp.rs` | Notification method |
| `impl ArpAnalyzer::summarize(&self)` | `src/analyzer/arp.rs` | Pure read-only aggregation |
| `new_for_test()`, `process_arp_for_test()`, `bindings_snapshot()` | `src/analyzer/arp.rs` `#[cfg(test)]` | Test affordances (ADR-008 Decision 4) |
| `--arp` CLI flag | `src/cli.rs` | Effectful shell (CLI) |
| `src/main.rs` summarize + append | `src/main.rs` | Effectful shell |
| VP-024 Sub-B harness | `src/analyzer/arp.rs` `#[cfg(kani)]` | Kani |
| VP-024 Sub-C proptest | `src/analyzer/arp.rs` tests | proptest |
| VP-024 Sub-D harness | `src/analyzer/arp.rs` `#[cfg(kani)]` | Kani |

Architecture section references: `architecture/module-decomposition.md` (SS-16 C-23 ArpAnalyzer), `architecture/dependency-graph.md`.

## Forbidden Dependencies

- `src/analyzer/arp.rs` MUST NOT import from `src/dispatcher.rs`. ArpAnalyzer is not a StreamAnalyzer.
- `src/analyzer/arp.rs` MUST NOT import from `src/analyzer/modbus.rs` or `src/analyzer/dnp3.rs`.
- `src/mitre.rs` MUST NOT be modified in this story. SEEDED/EMITTED counts remain at 23/15 until STORY-114's VP-007 atomic update.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | op=2, sender_ip==target_ip, no binding conflict | GARP finding LOW; binding initialized |
| EC-002 | op=1, sender_ip==target_ip (GARP Request, RFC 5227 announce) | GARP finding LOW (opcode-agnostic) |
| EC-003 | op=2, sender_ip==target_ip, binding EXISTS with same MAC | GARP finding LOW; no rebind (MAC unchanged); no D1 |
| EC-004 | op=2, sender_ip==target_ip, binding EXISTS with DIFFERENT MAC | GARP finding LOW (STORY-113); D1 escalation deferred to STORY-114 (BC-2.16.014) |
| EC-005 | outer_src_mac=None | D12 skipped; no mismatch finding |
| EC-006 | outer_src_mac=Some, matches sender_mac | No D12 finding |
| EC-007 | outer_src_mac=Some, differs from sender_mac | D12 MEDIUM finding emitted |
| EC-008 | hw_addr_size=8 ARP frame notification | D11 LOW finding via record_malformed; malformed_frames incremented |
| EC-009 | 65,537 distinct IP → binding table cap triggered | LRU eviction; len remains 65,536 |
| EC-010 | sender_ip=0.0.0.0 | Filtered; no binding; no finding |
| EC-011 | sender_ip=255.255.255.255 | Filtered; no binding; no finding |
| EC-012 | Zero frames processed | All eleven summarize keys = 0 |

## Tasks

1. **Replace `ArpAnalyzer` stub** with full implementation: add `bindings: HashMap<[u8;4], BindingEntry>`, `storm_counters: HashMap<[u8;6], StormCounter>` (stub: empty map; D3 logic in STORY-115), and all counter fields (`frames_analyzed: u64`, `request_count: u64`, `reply_count: u64`, `other_opcode_count: u64`, `spoof_findings: u64` [remains 0 here], `garp_findings: u64`, `storm_findings: u64` [remains 0], `mismatch_findings: u64`, `malformed_findings: u64`, `malformed_frames: u64`).
2. **Define `BindingEntry` struct**: `mac: [u8; 6]`, `rebind_count: u32`, `first_rebind_ts: Option<u32>`, `spoof_high_emitted: bool`, `last_seen_ts: u32`.
3. **Define `StormCounter` struct**: `count_in_window: u64`, `window_start_ts: u32`, `storm_emitted: bool` (stub for STORY-115).
4. **Implement `is_gratuitous_arp(frame: &ArpFrame) -> bool`**: single expression `frame.sender_ip == frame.target_ip`.
5. **Implement `insert_binding_lru`** (production, HashMap): scan for min `last_seen_ts` entry when `bindings.len() >= cap`; evict; insert new entry.
6. **Implement `insert_binding_lru_array`** (`#[cfg(any(kani, test))]`, array surrogate; signature: `entries: &mut [([u8; 4], [u8; 6], u32); CAP], len: &mut usize, ip, mac, cap`): same 3-branch eviction logic over a fixed-size array for VP-024 Sub-D Kani harness. Array surrogate is required because CBMC/Kani cannot model `HashMap` symbolically; sanctioned by VP-024 map-implementation-independence.
7. **Implement `process_arp`** full logic: (a) filter zero/broadcast sender_ip; (b) count frame (frames_analyzed, request_count/reply_count/other_opcode_count); (c) check D12 mismatch (outer_src_mac vs sender_mac); (d) check GARP (is_gratuitous_arp); (e) update binding table via `insert_binding_lru` (updating `last_seen_ts`); (f) detect rebind (MAC change) — update `rebind_count` and `first_rebind_ts`, but do NOT emit D1 finding (that is STORY-114); (g) return Vec of findings (D2/D12 only in this story).
8. **Implement `record_malformed(packet_len: usize)`**: increment `malformed_frames`; if `--arp` active, emit D11 LOW finding and increment `malformed_findings`. The mechanism for routing malformed notification to ArpAnalyzer from `main.rs` is an F3 implementation decision per BC-2.16.009 PC6 (e.g., a `process_malformed_arp` method or equivalent).
9. **Implement `summarize(&self)`**: return `AnalysisSummary` with all eleven canonical keys in their exact string names.
10. **Add `--arp: bool` flag** to `src/cli.rs` `Commands::Analyze`: `#[arg(long)] arp: bool`.
11. **Update `src/main.rs`**: call `arp_analyzer.summarize()` and append to `analyzer_summaries` when `args.arp` is true; wire `record_malformed` for `Err("Non-Ethernet/IPv4 ARP frame")` catch.
12. **Implement test affordances** (`#[cfg(test)]`): `new_for_test()`, `process_arp_for_test()`, `bindings_snapshot()`.
13. **Write VP-024 Sub-B Kani harness** (`verify_classify_garp_total`) in `src/analyzer/arp.rs` `#[cfg(kani)]` mod.
14. **Write VP-024 Sub-C proptest** (`test_BC_2_16_005_binding_table_last_write_wins`) in `src/analyzer/arp.rs` tests module.
15. **Write VP-024 Sub-D Kani harness** (`verify_binding_table_cap`) using `insert_binding_lru_array`.
16. **Run `cargo test --all-targets`**: all tests green (including proptest Sub-C).
17. **Run `cargo clippy --all-targets -- -D warnings`**: clean.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_16_003_is_gratuitous_arp_true_when_sender_eq_target_ip`, `test_BC_2_16_003_is_gratuitous_arp_false_when_sender_ne_target_ip` | Unit |
| AC-002 | `test_BC_2_16_003_is_gratuitous_arp_opcode_agnostic` | Unit |
| AC-003 | `test_BC_2_16_003_process_arp_garp_emits_low_anomaly_finding` | Unit |
| AC-004 | `test_BC_2_16_003_process_arp_garp_emits_per_frame` | Unit |
| AC-005 | `test_BC_2_16_005_binding_table_last_write_wins_basic` | Unit |
| AC-006 | `test_BC_2_16_005_binding_first_observation_no_finding` | Unit |
| AC-007 | `test_BC_2_16_005_binding_zero_sender_ip_filtered`, `test_BC_2_16_005_binding_broadcast_sender_ip_filtered` | Unit |
| AC-008 | `test_BC_2_16_006_binding_table_cap_enforced` | Unit |
| AC-009 | `test_BC_2_16_007_d12_mismatch_emits_medium_finding` | Unit |
| AC-010 | `test_BC_2_16_007_d12_skipped_when_outer_src_mac_none`, `test_BC_2_16_007_d12_skipped_when_macs_match` | Unit |
| AC-011 | `test_BC_2_16_009_d11_malformed_arp_emits_low_finding` | Unit |
| AC-012 | `test_BC_2_16_009_d11_malformed_counter_semantics` | Unit |
| AC-013 | `test_BC_2_16_010_summarize_zero_frames_all_eleven_keys_zero`, `test_BC_2_16_010_summarize_key_names_exact` | Unit |
| AC-014 | `test_BC_2_16_010_summarize_reconciliation_invariant` | Unit |
| AC-015 | `test_BC_2_16_011_main_arp_flag_absent_no_findings_no_summary` | Integration (tests/bc_2_16_story113_arp_tests.rs) |
| AC-016 | `test_BC_2_16_011_main_arp_flag_present_summarize_appended` | Integration (tests/bc_2_16_story113_arp_tests.rs) |
| AC-017 | `verify_classify_garp_total` | Kani (F6) |
| AC-018 | `test_BC_2_16_005_binding_table_last_write_wins` | proptest (`cargo test`) |
| AC-019 | `verify_binding_table_cap` | Kani (F6) |
| AC-020 | `test_BC_2_16_007_d12_and_garp_coemit_on_single_frame` | Unit |
| AC-021 | `test_BC_2_16_005_binding_same_mac_touches_last_seen_ts` | Unit |

## Previous Story Intelligence

STORY-112 (this epic's predecessor) established:
- `extract_arp_frame` fully implemented and tested.
- `ArpAnalyzer` stub (`new`, `process_arp` no-op) exists in `src/analyzer/arp.rs`.
- `main.rs` has the `DecodedFrame` pattern-match wired with the stub.
- `--arp` flag is NOT yet in `src/cli.rs` (STORY-112 only wired the main.rs match; the flag is added here in STORY-113).

**Crucial boundary**: STORY-113's `process_arp` detects rebinds (updates `rebind_count`) but does NOT emit D1 spoof findings. Adding any `spoof_findings` emission to `process_arp` in STORY-113 is out-of-scope and will break the F3 adversarial review (D1 emission carries a MITRE dependency on T0830/T1557.002 which are not seeded in `src/mitre.rs` until STORY-114).

Modbus/DNP3 pattern precedent (STORY-096, STORY-105, STORY-110): the `summarize()` → `analyzer_summaries.push(...)` wiring in `main.rs` mirrors the existing pattern. The `AnalysisSummary` type is shared; no reporter changes are needed per BC-2.16.010 Invariant 4.

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §1, §3.1–§3.3, ADR-008 Decisions 4–5, BC-2.16.010 Invariant 4:

1. **`insert_binding_lru` has NO `ts` parameter** — `last_seen_ts` is written by `process_arp` on every observation BEFORE calling `insert_binding_lru`; the eviction function reads it during the scan (ADR-008 Decision 4 normative note). Do not add `ts` as a parameter to `insert_binding_lru`.
2. **Array surrogate is `#[cfg(any(kani, test))]` only** — the `insert_binding_lru_array` function is not in the production binary. It uses a fixed-size array (`[([u8; 4], [u8; 6], u32); CAP]`) because CBMC/Kani cannot model `HashMap` symbolically; it reproduces the production 3-branch eviction logic and is sanctioned by VP-024 map-implementation-independence. Production substrate is `HashMap`.
3. **`ArpAnalyzer::new()` is parameterless in this story** — neither `spoof_threshold` nor `storm_rate` is consumed by STORY-113's detections (D2/D11/D12 have no configurable threshold). `spoof_threshold` is introduced in STORY-114 when D1 escalation lands (BC-2.16.012); `storm_rate` is introduced in STORY-115 when D3 detection lands (BC-2.16.013). STORY-113 adds ONLY the `--arp` bool flag (BC-2.16.011). Test affordances `new_for_test()`, `process_arp_for_test()`, and `bindings_snapshot()` are used for VP-024 Sub-C/D — they do not require constructor params.
4. **GARP-that-conflicts escalation (BC-2.16.014) is STORY-114** — when GARP + binding conflict is detected in STORY-113, emit GARP LOW only. The escalation to MEDIUM and the D1 finding emission are STORY-114's responsibility.
5. **Eleven keys exactly** — the exact string key names from BC-2.16.010 postcondition 1 are the contract. Any deviation (e.g., `"garp_count"` instead of `"garp_findings"`) fails the summary contract test.
6. **`storm_findings` key is 0** — D3 storm detection (STORY-115) populates this key. STORY-113's summarize() always returns 0 for `storm_findings`.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::collections::HashMap` | std | Production binding table and storm_counters substrate |
| Fixed-size array `[([u8; 4], [u8; 6], u32); CAP]` | std | Array surrogate only (`#[cfg(any(kani, test))]`) — no BTreeMap dependency |
| `proptest` | same as existing (check Cargo.toml) | VP-024 Sub-C proptest; `proptest::prelude::*` |
| `kani` | via cargo-kani | VP-024 Sub-B and Sub-D harnesses |
| `clap` | same as existing | `#[arg(long)] arp: bool` only — `--arp-spoof-threshold` is added in STORY-114; `--arp-storm-rate` is added in STORY-115 |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/arp.rs` | Replace stub with full impl | All types, functions, methods, test affordances, Kani/proptest harnesses |
| `src/cli.rs` | Modify | Add `#[arg(long)] arp: bool` only to `Commands::Analyze`; `--arp-spoof-threshold` belongs to STORY-114, `--arp-storm-rate` belongs to STORY-115 |
| `src/main.rs` | Modify | Wire `--arp` flag to analysis gate; wire `record_malformed`; call `summarize()` and append |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5,500 |
| BC files (7 BCs) | ~14,000 |
| arp-architecture-delta.md §1, §3.1–§3.3 | ~3,000 |
| VP-024 file (Sub-B/C/D sections) | ~2,000 |
| STORY-112 (for ArpAnalyzer stub context) | ~2,000 |
| Existing `src/main.rs`, `src/cli.rs` | ~2,500 |
| Tool outputs (cargo test, proptest, kani) | ~2,000 |
| **Total estimated** | **~31,000** |

This is the largest story in E-16. At ~31k tokens it approaches the 20–30% context window limit. If the implementing agent reports context pressure, split at the Sub-D Kani harness boundary: deliver tasks 1–12 + AC-001..AC-016 in sub-burst A, then tasks 13–17 + AC-017..AC-019 in sub-burst B.

## Dependency Rationale

- `depends_on: [STORY-112]` — STORY-112 delivers working `extract_arp_frame`, the `ArpAnalyzer` stub, and the `main.rs` `DecodedFrame` pattern-match. The full implementation in STORY-113 cannot proceed without the stub's method signatures and the working extraction function.
- `blocks: [STORY-114]` — STORY-114 implements D1 spoof escalation, GARP-that-conflicts (BC-2.16.014), and the VP-007 MITRE atomic update. All of these depend on the binding table (BC-2.16.005/BC-2.16.006) and GARP detection (BC-2.16.003) from this story.

## Changelog

- v1.2: F7 consistency F2 — Sub-D surrogate renamed from `insert_binding_lru_btree` (BTreeMap) to `insert_binding_lru_array` (array; signature `entries: &mut [([u8; 4], [u8; 6], u32); CAP], len: &mut usize, ip, mac, cap`). All six loci updated: Sub-D description (§VP-024 Sub-B/C/D), AC-019, Architecture Mapping table, Task 6, Task 15, Architecture Compliance Rule 2, Library table. Matches F6 implementation; sanctioned by VP-024 v2.2 + arp-architecture-delta v1.17.
- v1.1: F-3 pre-empt — AC **Test:** + Test Plan citations synced to exact BC-prefixed test fn names (DF-AC-TEST-NAME-SYNC-001). Source: test-writer commit 01a67c0. input-hash unchanged at 7c61bae.
