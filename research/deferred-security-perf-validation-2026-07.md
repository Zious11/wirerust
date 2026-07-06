# Deferred Security & Performance Findings — Validation Report

**Date:** 2026-07-06
**Validator:** vsdd-factory research-agent
**Policy:** DF-VALIDATION-001 (no GitHub issue may be filed from an unvalidated finding)
**Scope:** Read-only validation against develop HEAD `4a9eba3` + external research (CWE, CVEs, Rust aliasing models)
**Constraint:** Bash/`gh` unavailable to this agent; GitHub issue reads attempted via WebFetch (repo not publicly resolvable — see note under Finding 3). Issue dedup analysis is therefore based on code evidence + the finding descriptions supplied; the orchestrator must run `gh issue list --state open` to confirm issue numbers before filing.

---

## Deployment-Context Note (applies to all resource-exhaustion findings)

wirerust is an **offline / batch pcap analyzer**, not a live-capture daemon. It loads the whole
capture into memory as a `Vec<RawPacket>` before analysis, and it already ships a **global memory
cap** `--reassembly-memcap` (default 1024 MB) plus a per-file 4 GiB pcapng limit (CHANGELOG.md
lines 337, 758, 770). Per MITRE/NVD precedent for offline parsers (CVE-2026-6844 `readelf`,
CVE-2025-29484 libming), OOM from a crafted input file is a **local, user-interaction-required
DoS bounded by input file size** — materially lower severity than the same bug in a network-facing
daemon (CVE-2025-20239 Cisco IKEv2, CVE-2026-56149 Elasticsearch, scored high remote DoS).
[cwe.mitre.org/400, /401, /770; nvd CVE-2026-6844, CVE-2025-29484] This context caps the realistic
CVSS of SEC-005/SEC-006 at roughly Medium, and it means the per-flow-map growth is additionally
bounded by the existing `--reassembly-memcap` unless the analyzer flows are counted outside that cap.

---

## Summary Table

| # | Finding | Verdict | Severity (CWE / rough CVSS) | Dedup | Recommendation |
|---|---------|---------|------------------------------|-------|----------------|
| 1 | SEC-005 — ENIP `on_flow_close` unwired → per-flow state never freed | **CONFIRMED** | CWE-401 primary / CWE-770 secondary; offline-bounded MEDIUM (Local/UI-req, Avail-High, mem bounded by file) | Same root cause as #342 | **dedup-into-#342** |
| 2 | SEC-006 — DNP3 flow-map has no cap + no `on_flow_close` | **CONFIRMED** | CWE-401 primary / CWE-770 secondary; offline-bounded MEDIUM | Same root cause as #342 | **dedup-into-#342** |
| 3 | #342 relationship of SEC-005/006 | **N/A (relationship analysis)** | — | SEC-005 = ENIP arm; SEC-006 = DNP3 arm of the SAME defect | **dedup both into #342**; track as 2 checklist sub-items |
| 4 | SEC-001-ENIP — unsafe split-borrow in `enip.rs on_data` | **CONFIRMED (present) / sound-as-written** | CWE-anticipated (safe-refactor); no attacker channel; UB-risk LOW but fragile | Not in #342 | **downgrade-to-LOW** (refactor to `get_disjoint`/index; not a filed vuln) |
| 5 | SEC-001-STORY153 — `unclassified_port_counts` bound + missing doc | **PARTIALLY-CONFIRMED** | CWE-770 but bounded ~131,072 keys; INFORMATIONAL. Doc-comment promise NOT kept | Not in #342 | **file-new-issue (LOW / docs)** — add ceiling doc-comment |
| 6 | PERF-001/002 + BENCHMARK-GAP-001 — TLS carry regression + hotspots + missing fixture | **PARTIALLY-CONFIRMED** | Perf, not security. Regression figure UNVERIFIABLE from tree; benchmark gap is REAL | Not in #342 | **file-new-issue (perf/test)** for the benchmark gap; **defer** the % regression pending a reproducible bench |
| 7 | SEC-004 + SEC-007 — `+= 1` counters + MQ-003/004/005 clippy | **PARTIALLY-CONFIRMED / FALSE-POSITIVE (overflow)** | CWE-190 NOT realistically exploitable on u64 (~58,000 yr @ 1e7/s); cosmetic | Not in #342 | **downgrade-to-LOW** (hygiene only); no security issue |

**Top-priority recommendation:** Treat **#342 as the single tracking issue for SEC-005 + SEC-006** (they are the same defect: the dispatcher's `on_flow_close` never forwards to the DNP3/ENIP analyzers, so their unbounded `flows` maps are never pruned). The concrete fix is one code site — wire the two `DispatchTarget::Dnp3`/`Dnp3Analyzer` and `DispatchTarget::Enip`/`EnipAnalyzer::on_flow_close` arms in `src/dispatcher.rs`. Do NOT file SEC-005 and SEC-006 as separate issues.

---

## Finding 1 — SEC-005: ENIP `on_flow_close` unwired (CONFIRMED)

**Claim:** ENIP analyzer `on_flow_close` is unwired, so per-flow state is never cleaned up → unbounded memory growth (CWE-400 DoS).

**Code evidence:**
- `EnipAnalyzer::on_flow_close` **exists and is correct** — `src/analyzer/enip.rs:693-712`. It calls `self.flows.remove(&flow_key)` (line 696) and folds counters into aggregates. So the analyzer *can* free per-flow state.
- **BUT the dispatcher never calls it.** In `src/dispatcher.rs:453-457`, the `Some(DispatchTarget::Enip)` arm of `StreamDispatcher::on_flow_close` is a no-op:
  ```rust
  Some(DispatchTarget::Enip) => {
      // EnipAnalyzer does not implement StreamHandler; no forwarding needed.
      let _ = reason;
  }
  ```
  The comment "no forwarding needed" is the defect: `EnipAnalyzer::on_flow_close(flow_key)` is a plain method (not the `StreamHandler` trait method), and it is simply never invoked from the dispatcher's close path.
- `EnipAnalyzer::on_data` unconditionally does `self.flows.entry(flow_key.clone()).or_default()` (`src/analyzer/enip.rs:783`) with **no `flows.len()` cap and no eviction** (confirmed by grep: no `MAX_FLOWS`, `flows.retain`, or `flows.len()` guard anywhere in enip.rs).
- End-of-capture drain exists (`src/main.rs:509 take_enip_analyzer()`), so state is freed at process exit — but it accumulates for the **entire capture** until then.

**Verdict:** **CONFIRMED.** Per-flow `EnipFlowState` (which itself contains `HashMap`s and `Vec` carry buffers) is retained for every distinct port-44818 flow until the analyzer is dropped at end-of-capture. A crafted pcap with many distinct 5-tuples to port 44818 grows `flows` monotonically.

**Severity:** CWE-401 (Missing Release after Effective Lifetime) is the most precise root-cause classification — the flow's `FlowState` is logically dead once the TCP flow closes (FIN/RST) but is never released; CWE-770 (Allocation Without Limits) describes the attacker-driven manifestation, with CWE-400 as the umbrella. The originally-claimed bare **CWE-400 is acceptable but imprecise** — prefer CWE-401 (primary) + CWE-770 (secondary). Attacker-controllable via untrusted pcap: yes. Offline-bounded: memory bounded by file size AND by `--reassembly-memcap`; **realistic severity MEDIUM** (Local, UI-required, Availability-High), not High. [cwe.mitre.org/401, /770; nvd CVE-2026-6844]

**Dedup:** Same root cause as #342 (see Finding 3). **dedup-into-#342.**

---

## Finding 2 — SEC-006: DNP3 flow-map has no cap (CONFIRMED)

**Claim:** DNP3 flow-map has no cap → unbounded growth (CWE-400).

**Code evidence:**
- `Dnp3Analyzer.flows: HashMap<FlowKey, Dnp3FlowState>` (`src/analyzer/dnp3.rs:303`).
- `Dnp3Analyzer::on_data` does `self.flows.entry(flow_key.clone()).or_default()` (`src/analyzer/dnp3.rs:345`) with **no cap on the number of flows**.
- **Dnp3Analyzer has NO `on_flow_close` method at all** (grep for `fn on_flow_close` in dnp3.rs returns nothing). The dispatcher's `Some(DispatchTarget::Dnp3)` arm is likewise a no-op (`src/dispatcher.rs:448-452`, `let _ = reason;`).
- Intra-flow state IS bounded: `pending_requests` capped at `MAX_PENDING_REQUESTS=256` with LRU eviction (dnp3.rs:1729-1745), `master_addrs_seen` capped at `MAX_MASTER_ADDRS=64` (dnp3.rs:696-701), carry buffers capped at `MAX_DNP3_FRAME_LEN=292` per direction. **Only the outer `flows` map is uncapped.**
- `summarize()` iterates `self.flows.values()` (dnp3.rs:1664-1666), so all flows are retained until end-of-capture drain (`src/main.rs:502 take_dnp3_analyzer()`).

**Verdict:** **CONFIRMED.** Identical shape to SEC-005: the outer per-flow map has no cap and no cleanup-on-close. Contrast with ARP (LRU eviction, MAX_ARP_BINDINGS) and Modbus/HTTP/TLS (which `flows.remove` in their `on_flow_close`).

**Severity:** Same as Finding 1 — CWE-401 primary / CWE-770 secondary; offline-bounded MEDIUM. `Dnp3FlowState` is heavier than ENIP's (more `HashMap`/`Vec` fields), so per-entry cost is higher, but the bound is still file-size × per-flow-state.

**Dedup:** Same root cause as #342 (see Finding 3). **dedup-into-#342.**

---

## Finding 3 — Relationship of SEC-005 / SEC-006 to Issue #342

**Issue #342 (as described):** "Fuzzing & robustness report: DNP3/ENIP analyzers leak per-flow state (unbounded memory)."

**Note on issue read:** WebFetch of the issue URL returned HTTP 404 (repo not publicly resolvable; `gh` unavailable to this agent). Analysis below is from the finding title supplied plus code evidence. **Orchestrator must confirm the live #342 body and number via `gh issue view 342` before filing/deduping.**

**Analysis:** SEC-005 (ENIP) and SEC-006 (DNP3) are **not distinct root causes** — they are the **two protocol-specific arms of a single defect**: `StreamDispatcher::on_flow_close` (`src/dispatcher.rs:425-493`) forwards close events to HTTP, TLS, and Modbus analyzers (arms at lines 432-447, all of which call the analyzer's `on_flow_close` and free per-flow state) but **deliberately stubs out the DNP3 and ENIP arms** (lines 448-457) with `let _ = reason;` and the (incorrect) comment "does not implement StreamHandler; no forwarding needed."

The issue #342 title names **exactly these two analyzers** ("DNP3/ENIP") and the same symptom ("leak per-flow state (unbounded memory)"). SEC-005 and SEC-006 are therefore the ENIP-half and DNP3-half of #342, respectively.

**Recommendation:** **dedup BOTH SEC-005 and SEC-006 into #342.** Track them as two checklist sub-items under #342 (one code fix touches both):
1. Wire `Some(DispatchTarget::Dnp3)` arm → call `dnp3.on_flow_close(flow_key)` (requires adding an `on_flow_close` method to `Dnp3Analyzer`, mirroring `EnipAnalyzer::on_flow_close`).
2. Wire `Some(DispatchTarget::Enip)` arm → call `enip.on_flow_close(flow_key.clone())` (method already exists at enip.rs:693).

Do **not** open new issues for SEC-005 or SEC-006.

---

## Finding 4 — SEC-001-ENIP: unsafe split-borrow in `enip.rs on_data` (CONFIRMED present / sound-as-written)

**Claim:** unsafe split-borrow in `enip.rs on_data`, potential memory-safety/UB, MEDIUM.

**Code evidence:** `src/analyzer/enip.rs:985-1000`, PDU-dispatch phase:
```rust
for pdu in pdu_queue {
    let flow_ptr: *mut EnipFlowState = self
        .flows
        .get_mut(&flow_key)
        .expect("flow exists: inserted above and not removed");
    #[allow(clippy::ptr_as_ptr)]
    self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, timestamp, src_ip);
}
```
This is exactly the "obtain `*mut T` from `HashMap::get_mut`, then `unsafe { &mut *ptr }`" pattern. It exists to sidestep the borrow conflict between `flow` (borrowed from `self.flows`) and `process_pdu` (needs `&mut self` for `self.all_findings`, `self.error_count`, etc.). The SAFETY comment (lines 986-997) asserts `process_pdu` does not touch `self.flows` or alias `flow_ptr`.

**Soundness assessment:** Sound **as written**, conditional on invariants that the code currently upholds:
- `flow_ptr` is re-acquired **inside each loop iteration** via a fresh `get_mut` — so it cannot dangle across an intervening map mutation within the loop.
- `process_pdu` must not access `self.flows` (the aliased field) while the `&mut *flow_ptr` reference is live. I verified by inspection of the `process_pdu` signature (enip.rs:1029) that it receives `flow: &mut EnipFlowState` and mutates `self.all_findings`/`error_count`/`write_count`/`dropped_findings`/threshold fields — none of which is `self.flows`. Under Stacked/Tree Borrows this reborrow is permitted as long as no other reference to that `EnipFlowState` is created in the interim and the map is not re-hashed/reallocated while the pointer is live. [ralfj.de/blog Tree Borrows; internals.rust-lang.org Stacked Borrows]
- **Fragility:** if a future edit makes `process_pdu` call any `self.flows` method (get/get_mut/insert/remove/iterate), the pattern becomes UB (aliasing violation or dangling-on-realloc). The `#[allow(clippy::ptr_as_ptr)]` and hand-written SAFETY comment are the only guardrails — there is no compile-time enforcement.

**Verdict:** **CONFIRMED** the unsafe split-borrow exists; **NOT a live memory-safety bug** on HEAD. There is **no attacker-controllable path to UB** — pcap contents cannot cause `process_pdu` to touch `self.flows`. So this is not a filable security vulnerability. It IS a maintainability/latent-UB hazard.

**Severity:** Not a security finding (no exploit channel). Latent-UB-risk LOW. Originally claimed MEDIUM is too high.

**Recommendation:** **downgrade-to-LOW.** Suggest a safe refactor (collect PDUs then process without holding the flow borrow — which the code already partly does via `pdu_queue`; the remaining `unsafe` could be replaced by passing the disjoint `self` sub-fields into a free function, exactly as `Dnp3Analyzer` does with its `detect_*_split` associated functions at dnp3.rs:897+). Track as tech-debt, not a vuln. Not part of #342.

---

## Finding 5 — SEC-001-STORY153: `unclassified_port_counts` bound + missing doc (PARTIALLY-CONFIRMED)

**Claim:** `per_port_counts` HashMap in the coverage-gaps path is bounded at ~130k keys (one per distinct `(TransportProto, port)`; port space ~65k×2). CWE-400 but bounded. Validate the bound is real and whether the promised doc-comment on the ceiling was added.

**Code evidence:**
- Field: `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` (`src/dispatcher.rs:100`).
- `TransportProto` has exactly 2 variants (`Tcp`, `Udp` — dispatcher.rs:46-49). Key second component is `u16` (65,536 values). **Maximum distinct keys = 2 × 65,536 = 131,072.** Bound is REAL and tight.
- TCP keys inserted in `on_flow_close` only when `coverage_gaps_enabled` (dispatcher.rs:477-489), counter uses `saturating_add` (line 488). UDP keys accumulated in `main.rs` via the `udp_gap_key` seam (dispatcher.rs:520-538, main.rs:415). Merged in `collect_all_gaps` (main.rs:1082-1092).
- **Missing doc-comment:** The field doc (dispatcher.rs:97-104) and accessor doc (dispatcher.rs:167-175) describe the key semantics and the `coverage_gaps_enabled` gate, but contain **NO mention of the numeric ceiling** (131,072). Grep for `130` / `131072` / `65536` / `ceiling` / `upper bound` in dispatcher.rs finds only the unrelated retry-cap doc (line 64) and the Kani port-symbol comment (line 593). **The promised ceiling doc-comment was NOT added.**

**Verdict:** **PARTIALLY-CONFIRMED.** The bound is real and correctly ~131,072 keys (the "~130k" estimate is accurate). The map is not a DoS vector (bounded, small, u64 values saturating). **The documentation promise is unfulfilled** — this is the actionable part.

**Severity:** CWE-770 in the abstract but **bounded and tiny** (131,072 × ~24 bytes ≈ 3 MB worst case) — INFORMATIONAL, not a security risk. Not attacker-amplifiable beyond the fixed ceiling.

**Recommendation:** **file-new-issue (LOW / docs).** Add a doc-comment to `unclassified_port_counts` (and/or `with_coverage_gaps`) stating the hard ceiling of `2 × u16::MAX+1 = 131,072` entries and why it is safe (one entry per distinct `(TransportProto, port)`, values saturating). This closes the STORY-153 documentation debt. Not part of #342. Very low priority.

---

## Finding 6 — PERF-001/002 + BENCHMARK-GAP-001 (PARTIALLY-CONFIRMED)

**Claim:** TLS carry-path +10.3% throughput regression; HashMap + Vec allocation hotspots in the reassembly path; no fragmented-handshake benchmark fixture exists.

**Code evidence:**
- **Allocation hotspots are real and visible.** The TLS carry path clones per-record and per-message: `record_bytes ... .to_vec()` (`src/analyzer/tls.rs:785,788`) and `msg_bytes ... .to_vec()` (tls.rs:933, 1063) on every dispatched handshake message, plus repeated `self.flows.get`/`get_mut` lookups inside the drain loop (tls.rs:882-895, 929-935, 1019-1033, 1059-1065). The code itself documents a prior O(N²) `drain`-per-message hazard that was fixed to cursor-based single-drain (SEC-001 note, tls.rs:846-874) — evidence the carry path is a known perf-sensitive area.
- **The +10.3% figure is UNVERIFIABLE from the tree.** The regression is a runtime measurement; no committed criterion baseline or regression record exists to confirm it. I can confirm the *mechanism* (extra clones/lookups) is plausible but cannot validate the specific percentage.
- **BENCHMARK-GAP-001 is REAL.** `benches/pipeline.rs` is the only bench file. Its `bench_reassembly` group loads only `segmented.pcap` and `tls.pcap` (benches/pipeline.rs:92) and constructs the dispatcher with only HTTP+TLS analyzers (lines 108-114). **There is no fragmented / multi-record-split TLS-handshake fixture** exercising the `client_hs_carry`/`server_hs_carry` drain loop across multiple `on_data` calls — the exact path PERF-001/002 concern. The benchmark cannot detect a carry-path regression because no bench drives the carry path with fragmentation.

**Verdict:** **PARTIALLY-CONFIRMED.** Hotspots (clones + repeated map lookups) are evidenced in source. The benchmark gap is definitively real. The specific throughput-regression percentage is not reproducible from the repository state.

**Severity:** Performance, not security. No CWE. No attacker relevance beyond the general (already-covered) DoS surface.

**Recommendation:**
- **file-new-issue (test/perf):** Add a fragmented-TLS-handshake benchmark fixture + a `bench_reassembly` case that splits a ClientHello/ServerHello across many small `on_data` chunks, so the carry-path is measured and future regressions are caught. This is the concrete, verifiable gap.
- **defer** the "+10.3% regression" claim until the above benchmark exists to reproduce it. Do not file a regression issue against an unreproducible number.
- Not part of #342.

---

## Finding 7 — SEC-004 + SEC-007: `+= 1` counters + MQ-003/004/005 (PARTIALLY-CONFIRMED / overflow FALSE-POSITIVE)

**Claim:** several counter increments use `+= 1` instead of `saturating_add` (overflow risk); plus clippy hygiene MQ-003/004/005.

**Code evidence:**
- Non-saturating `+= 1` sites exist and operate on `u64`/`usize` counters. Examples: `flow.malformed_in_window += 1` (dnp3.rs:424,425,483,484,515,516,580,581,621,622,667,668; enip.rs:868,894,957), `flow.parse_errors += 1` (dnp3.rs multiple), `flow.restart_event_count += 1` (dnp3.rs:1020), `flow.direct_operate_count += 1` (dnp3.rs:922), `self.unclassified_flows += 1` (dispatcher.rs:471), `*flow.command_counts.entry(...).or_insert(0) += 1` (enip.rs:866,892,923), `self.handshakes_seen += 1` (tls.rs:437). Note the release profile sets `overflow-checks = true` (per CLAUDE.md), so a genuine overflow would **panic** (a DoS), not wraparound — but see exploitability below.
- The codebase is **inconsistent**: some hot counters already use `saturating_add` (enip.rs:494,698,700,704,708,761; dispatcher.rs:371,488; tls.rs:837,1171) with explicit "SEC-003" comments, while sibling counters use `+= 1`.

**Exploitability analysis (external research):** All affected counters are `u64` (or `usize`, 64-bit on target platforms). Overflowing a `u64` via `+= 1` requires 2^64 ≈ 1.84×10^19 increments. At an unrealistically high 10^7 increments/second that is ≈ **58,000 years**; at 10^8/s ≈ 5,800 years. [cwe.mitre.org/190; arithmetic]. These counters increment at most once per frame/PDU/flow, and the entire input is bounded by a ≤4 GiB file. **There is no attacker-controllable path to overflow.** The `u64` type choice is itself the CWE-190 mitigation MITRE recommends.

**Verdict:**
- **Overflow risk: FALSE-POSITIVE / cosmetic.** `saturating_add` here is defensive hardening (and consistency with the SEC-003 siblings), not a genuine CWE-190 fix. Not a security finding.
- **MQ-003/004/005 clippy hygiene: PARTIALLY-CONFIRMED** as legitimate but purely cosmetic lint items. (I could not map the exact MQ-003/004/005 identifiers to specific lints from the tree — the finding did not enumerate them and no MQ-* reference exists in the source. Orchestrator should confirm which lints these are; CI already runs `clippy --all-targets -D warnings`, so any real clippy warning would already fail CI, implying these are either allow-listed or already-clean.)

**Severity:** No CWE that is realistically exploitable. CWE-190 does not apply in practice. Cosmetic / consistency only.

**Recommendation:** **downgrade-to-LOW.** Optionally fold into a single "counter-hygiene: use `saturating_add` consistently for aggregate counters" chore issue for code consistency with the existing SEC-003 sites — but flag explicitly that this is **hygiene, not a vulnerability**. Do NOT file as a security issue. Not part of #342.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 (reasoning_effort=high) | CWE-400/401/770/774 classification for unbounded per-flow map; offline-vs-daemon DoS severity & file-size boundedness; u64 `+= 1` CWE-190 exploitability math; Rust unsafe `*mut` split-borrow soundness under Stacked/Tree Borrows. Grounded in MITRE CWE, NVD CVEs (CVE-2026-6844, CVE-2025-29484, CVE-2026-56149, CVE-2025-20239), RustSec RUSTSEC-2025-0125, CVSS v3.1 spec, ralfj.de. |
| WebFetch | 1 | Attempted read of GitHub issue #342 — returned 404 (repo not publicly resolvable; `gh` unavailable to agent). Escalated to orchestrator. |
| Read | 8 | dispatcher.rs (full), enip.rs (1-1042), dnp3.rs (1-1109, 1640-1746), tls.rs (1-1177), benches/pipeline.rs, main.rs (450-530, 1440-1510). |
| Grep | 8 | on_flow_close wiring; flows-map cap/eviction search across analyzers; unsafe/split-borrow in enip; STORY-153 ceiling-doc search; counter `+= 1` sites; CHANGELOG memcap context. |
| Glob | 2 | Locate src/**/*.rs and benches layout. |
| Training data | 0 areas | All external claims sourced to Perplexity/MITRE/NVD; all code claims sourced to file+line. |

**Total MCP tool calls:** 1 (`perplexity_research`, high effort). WebFetch (1) is a built-in tool, not MCP.
**Training data reliance:** low — CWE/CVE/aliasing claims are web-sourced with citations; code verdicts are file+line grounded.

### MCP-UNAVAILABLE note (partial)
MCP Perplexity was available and used. The only unavailable capability was authenticated GitHub issue reading (`gh`/Bash denied to this agent; WebFetch 404 on the repo). This does not affect the code-level verdicts; it only means the **issue-number dedup must be reconfirmed by the orchestrator** via `gh issue list --state open` and `gh issue view 342` before any issue is filed. The finding-to-#342 relationship (Finding 3) is established on code + title evidence and is high-confidence regardless.
