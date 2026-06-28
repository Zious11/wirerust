---
title: Edge-Case Hunt Register — All Protocol Analyzers
date: 2026-06-28
scope: develop @ b6d7a01
hunters: 6 parallel adversary agents (ENIP, DNP3, Modbus, ARP, Reassembly/Infra, TLS/HTTP/Reporting/Reader)
status: CANDIDATES — research-validation (DF-VALIDATION-001) required before any become issues
human-directive: record register only (no fixes yet except Modbus verification); verify Modbus EC-X1/EC-X2 first
design-notes:
  - cycles/feature-enip-v0.11.0/DESIGN-TIMESTAMP-MONOTONICITY.md
  - cycles/feature-enip-v0.11.0/DESIGN-CROSS-DIRECTION-STATE.md
---

# Edge-Case Hunt Register — All Protocol Analyzers (2026-06-28)

Hunt across all protocol analyzers on develop @ b6d7a01, 6 parallel adversary hunters (ENIP,
DNP3, Modbus, ARP, Reassembly/Infra, TLS/HTTP/Reporting/Reader). ~30 candidates total.

**Human directive:** record register for now (no fixes yet except Modbus verification).
All candidates are DF-VALIDATION-001-gated — no GitHub issue may be created from an
unvalidated finding. Research-validation required before filing.

**Modbus EC-X1 and EC-X2 have since been empirically confirmed** via test-vs-control repro
(scratch worktree `.worktrees/modbus-ecx-verify`, scratch/modbus-ecx-verify @ 74f2913).
See CONFIRMED tier below and D-292 in STATE.md.

---

## CONFIRMED (empirical repro — no further validation required)

### MODBUS EC-X1 — Cross-Direction Carry Splice (CRITICAL)

- **Status:** CONFIRMED (scratch worktree 74f2913, test-vs-control repro)
- **Severity:** CRITICAL
- **Source locations:** `modbus.rs:170` (single `carry: Vec<u8>` field), `modbus.rs:290`
  (`flows: HashMap<FlowKey,ModbusFlowState>` — FlowKey-only keying)
- **Pattern:** `buf = carry ++ data` with no direction check — partial C2S ADU carry is
  spliced into S2C response buffer. Real response is swallowed (fn_code_counts divergence
  confirmed in repro).
- **Root cause:** Same cross-direction carry splice bug as ENIP/DNP3 release-blockers
  (EC-X1 pattern). Modbus has per-flow threading but NOT per-direction carry.
- **RULING-EDGECASE-001 §1.6 status:** "Modbus already has direction threading and is NOT
  affected" — DISPROVEN by this confirmation. Direction-threading ≠ per-direction carry.
- **Process gap:** DF-SIBLING-SWEEP-001 miss — CRITICAL. Modbus was omitted from the §2.5
  sibling sweep that named only DNP3.
- **Release impact:** MATERIAL TO HELD v0.11.0 RELEASE — awaiting human release/scope
  decision.
- **Repro evidence:** Scratch worktree `.worktrees/modbus-ecx-verify` @ 74f2913; preserved
  to seed regression tests for a Modbus fix story.

### MODBUS EC-X2 — wrapping_sub Clock-Backwards (HIGH)

- **Status:** CONFIRMED (scratch worktree 74f2913, test-vs-control repro)
- **Severity:** HIGH
- **Source locations:** `modbus.rs:534`, `modbus.rs:595`, `modbus.rs:670`, `modbus.rs:820`
- **Pattern:** One backwards-ts packet resets burst window → T0806/T0831/T0888 suppressed
  in repro (0 findings vs control fires).
- **Root cause:** Same wrapping_sub clock-backwards bug as ENIP/DNP3 (EC-X2 pattern).
- **Spec conflict:** STORY-104 AC-006 actively mandates `wrapping_sub` (process-gap).
  Spec must change in the same fix story that updates the code.
- **RULING-EDGECASE-001 §2.5 status:** Named only DNP3 in the wrapping_sub sweep; Modbus
  was omitted. DF-SIBLING-SWEEP-001 miss.
- **Release impact:** HIGH — affects T0806/T0831/T0888 detection reliability.
- **Repro evidence:** Scratch worktree `.worktrees/modbus-ecx-verify` @ 74f2913.

---

## CRITICAL (code-grounded, unverified — DF-VALIDATION-001-gated)

### DNP3 cand-01 — Flow-Global is_non_dnp3 Desync Latch (CRITICAL)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Source locations:** `dnp3.rs:350`, `dnp3.rs:363-370`
- **Pattern:** `is_non_dnp3` is a flow-level (not direction-level) latch. STORY-140
  carry-split exposed that the latch fires on a single direction's first-delivery but
  silences the already-established opposite direction (per-flow latch vs per-direction
  first-delivery). One-line fix: latch only when BOTH carries empty.
- **Design note:** DESIGN-CROSS-DIRECTION-STATE.md §DNP3 section.
- **Validation required before issue filing.**

### TLS cand-01 — ClientHello Fragmented Across TLS Records (CRITICAL)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Source locations:** `tls.rs:763-792`
- **Pattern:** ClientHello fragmented across TLS handshake records is not reassembled →
  SNI/JA3 fields silently absent → SNI classification evasion + JA3 fingerprint evasion.
  Full Handshake record reassembly not present in parser.
- **Validation required before issue filing.**

---

## HIGH (code-grounded, unverified — DF-VALIDATION-001-gated)

### ENIP cand-01 — 16-bit CIP Logical-Segment Evades T0888 Identity-Read

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** 16-bit CIP logical-segment width not parsed → T0888 Identity-read detection
  miss. Consequence of the documented 16-bit CIP logical-segment deferral.
- **Validation required before issue filing.**

### ENIP cand-02 — Path-Segment Desync False-Pos/Neg

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Path-segment parser desync produces false positives and false negatives in
  CIP path interpretation.
- **Validation required before issue filing.**

### DNP3 cand-03 — PRM-Bit Not Checked (BC-2.15.008 Inv-4 Contradiction)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** PRM=0 secondary frame is mis-parsed as user-data. BC-2.15.008 Inv-4 prose
  states PRM must be checked; code does not enforce it. Prose-vs-code contradiction.
- **Validation required before issue filing.**

### DNP3 cand-04 — Valid User-Data Frame frame_len<13 Drops parse_errors++ (Partial EC)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Source locations:** BC-2.15.008 EC-006 implementation
- **Pattern:** Valid user-data frame with `frame_len < 13` drops the BC-2.15.008 EC-006
  `parse_errors++` call — partial EC implementation gap.
- **Validation required before issue filing.**

### ARP cand-01 — Backwards/Jittered Timestamps → Storm Denominator=1 False Positive

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Backwards or jittered pcap timestamps cause storm denominator to collapse to
  1 → storm FALSE POSITIVE. ARP uses `saturating_sub` (not `wrapping_sub`), so the EC-X2
  fix pattern for Modbus/DNP3 does not directly apply; this is a denominator-policy issue.
- **Note:** DESIGN-TIMESTAMP-MONOTONICITY.md §ARP section recommends a separate BC decision
  for ARP timestamp handling.
- **Validation required before issue filing.**

### ARP cand-02 — D1 Flap-Window Pinned Open by Backwards Timestamps

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Backwards timestamps pin the D1 (gateway rebind) flap-window open indefinitely.
  Same non-monotonic timestamp root cause as cand-01 but distinct behavioral consequence.
- **Validation required before issue filing.**

### HTTP cand-02 — No Content-Length/Transfer-Encoding Awareness

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** No Content-Length / TE header awareness → HTTP body that parses as a new
  request is mis-counted → phantom findings and HTTP request-smuggling blindness.
- **Validation required before issue filing.**

### HTTP cand-04 — Parse Error Clears Whole Buffer

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Parse error clears the entire HTTP buffer → drops trailing valid pipelined
  requests that follow a malformed one.
- **Validation required before issue filing.**

### READER cand-05 — pcapng EPB original_len Discarded

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** pcapng EPB `original_len` is discarded after capture → downstream code cannot
  distinguish a snaplen-truncated frame from a malformed frame. Interacts with TLS cand-01
  (fragmented records from truncated captures appear identical to malformed frames).
- **Validation required before issue filing.**

### MODBUS cand-02 — Direction-Shared Per-Flow Window/State Beyond Carry (Pending Intent)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Beyond the carry field (EC-X1), per-flow window state and burst-tracking is
  also shared across directions — scope of fix beyond EC-X1 carry-split pending intent
  confirmation.
- **Validation required before issue filing.**

---

## MEDIUM (code-grounded, unverified — DF-VALIDATION-001-gated)

### ENIP cand-03 — Multi-0x00B2 Items Per Frame Inflate Counters and Findings

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Multiple 0x00B2 CPF items in a single EtherNet/IP frame each independently
  increment finding counters → over-counting and inflated findings.
- **Validation required before issue filing.**

### ENIP cand-04 — Resync-DoS O(n) Byte-Walk

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Resync path performs O(n) byte-walk on malformed data; rooted in the
  proven-unreachable carry-cap latch never quarantining junk flows.
- **Validation required before issue filing.**

### ENIP cand-05 — Unbounded command_counts Cardinality

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** `command_counts` map has unbounded cardinality; attacker-controlled command
  codes produce unbounded map growth; rooted in the same latch gap as cand-04.
- **Validation required before issue filing.**

### DNP3 cand-05 — min_timedout_dest=0 Sentinel Attribution

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** `min_timedout_dest` sentinel value of 0 causes timeout events to be
  attributed to destination address 0 when no real destination has timed out.
- **Validation required before issue filing.**

### DNP3 cand-06 — First-Window elapsed=0 Display

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** First window reports elapsed=0 in display output regardless of actual elapsed
  time; cosmetic but may mislead operators.
- **Validation required before issue filing.**

### MODBUS cand-03 — Length-Over-Claim Pipelining Desync Evasion

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Attacker sends PDU with over-claimed MBAP length → subsequent pipelined
  requests consumed as data → pipelining desync → detection evasion.
- **Validation required before issue filing.**

### MODBUS cand-04 — Permanent Attacker-Triggerable is_non_modbus Latch (FN DoS)

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** `is_non_modbus` latch, once set, permanently silences all future findings
  on that flow. Attacker can trigger it with a single junk packet → false-negative DoS
  for the duration of the TCP connection.
- **Validation required before issue filing.**

### ARP cand-03 — VRRP/HSRP/Multicast-MAC D12 False Positives

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** VRRP/HSRP virtual MACs and multicast MAC addresses trigger D12 false
  positives; these are legitimate protocol behaviors, not ARP anomalies.
- **Validation required before issue filing.**

### ARP cand-04 — Per-Frame Randomized Source-MAC Defeats D3 Storm + Broadcast Tracking

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Per-frame randomized source MAC (e.g., from scanning tools) defeats D3
  ARP storm detection and broadcast-MAC address tracking.
- **Validation required before issue filing.**

### TLS cand-06 — Uncapped Per-Flow Analyzer State DoS

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Per-flow TLS analyzer state is uncapped — attacker can create unbounded
  per-flow state via long-running connections with many records.
- **Validation required before issue filing.**

### REPORT cand-03 — Analyzer Finding-Cap Asymmetry

- **Status:** CANDIDATE — PENDING INTENT confirmation + DF-VALIDATION-001-gated
- **Pattern:** Modbus caps per-flow findings; TLS, HTTP, and ARP do not. Asymmetric
  per-analyzer MAX_FINDINGS behavior. Pending confirmation of whether this is intentional
  design or a gap.
- **Validation required before issue filing.**

### HTTP cand-07 — Duplicate-Host Not Detected Despite Smuggling Claims

- **Status:** CANDIDATE — DF-VALIDATION-001-gated
- **Pattern:** Duplicate Host headers are not detected despite HTTP smuggling detection
  claims in findings. Smuggling via duplicate-Host header is undetected.
- **Validation required before issue filing.**

---

## LOW / Observations / Accepted Deferrals

### ENIP cand-06 — 0x0A MultipleServicePacket

- **Status:** DOCUMENTED DEFERRAL
- **Note:** MultipleServicePacket (0x0A) service parsing deferred to v0.12.0. Accepted.

### ARP cand-05 — Truncated-ARP Observability

- **Status:** OBSERVATION
- **Note:** Truncated ARP packets (<28 bytes) are silently discarded with no finding;
  observability gap (no parse_errors increment).

### ARP cand-06 — rebind_count u32 Overflow Panic

- **Status:** LOW — theoretical
- **Note:** `rebind_count` is u32 and could overflow to panic; requires ~4.3 billion
  rebinds — effectively unreachable in practice.

### INFRA cand-02 — ENIP/DNP3 Shared Malformed-Window

- **Status:** ADJUDICATED — no action
- **Note:** ENIP and DNP3 share a single `malformed_window` instance; adjudicated
  acceptable at §1.3 (separate per-protocol analysis contexts).

### INFRA cand-03 — Backwards last_seen Idle Measure

- **Status:** BACKSTOPPED
- **Note:** Backwards `last_seen` timestamps for idle measurement are backstopped by R4
  (flow-state max_flows eviction guard). No independent fix required.

---

## CONFIRMED-CLEAN (probed, no novel gap)

The following areas were specifically probed during the hunt and found to have no novel gaps:

| Area | Verdict | Basis |
|------|---------|-------|
| Shared reassembly engine | CLEAN | VP-001/002/015, R1/R4 guards, memcap/max_flows all correct |
| pcapng reader (total correctness) | CLEAN | Kani-proven total; all EPB/IDB/SHB paths covered |
| TLS SNI 4-way classification | CLEAN | VP-005; all 4 classification states correct |
| MAX_FINDINGS cap | CLEAN | No off-by-one; F6 'impractical' mutants are behaviorally equivalent |
| Modbus exception-window arithmetic | CLEAN | Arithmetic correct; root causes are carry and timestamp |
| DNP3 port-20000 heuristic | NOT-A-DEFECT | Correct Modbus-mirroring implementation; adjudicated F5/F7 AC-140-002b |
| Dispatcher tiny-first-chunk port fallback | NOT-A-DEFECT | Accepted behavior |

---

## Cross-Cutting Themes

Two cross-cutting design-scope notes were written to `.factory/` (commit afd7dbb):

### Theme 1: Timestamp Monotonicity

**File:** `cycles/feature-enip-v0.11.0/DESIGN-TIMESTAMP-MONOTONICITY.md`

Neither `wrapping_sub` nor `saturating_sub` detects or handles out-of-order (non-monotonic)
timestamps. The two patterns have different failure modes:
- `wrapping_sub`: backwards timestamp resets window (Modbus EC-X2 pattern)
- `saturating_sub`: backwards timestamp collapses denominator to 0 or 1 (ARP storm gap)

**Recommendation:** Two-phase approach:
1. Modbus fix uses `saturating_sub` (consistent with ENIP/DNP3 precedent)
2. ARP denominator-policy requires a separate BC decision (distinct issue)
3. `WindowClock` abstraction deferred — not required for either fix

**4 un-swept wrapping_sub sites in Modbus:** lines 534, 595, 670, 820.

### Theme 2: Cross-Direction Shared Per-Flow State

**File:** `cycles/feature-enip-v0.11.0/DESIGN-CROSS-DIRECTION-STATE.md`

**ENIP:** §1.3 adjudicated correct — shared malformed-window is intentional.
**DNP3:** `is_non_dnp3` desync-latch is a one-line fix — latch only when BOTH carries empty.
**Modbus:** Carry is the sole structural gap; confirmed CRITICAL (EC-X1).

**NOT recommended:** Full `(FlowKey, Direction)` keying for counters — counters are
flow-level aggregates and direction-splitting them would break finding semantics.

These notes inform v0.12.0 planning and the scope of the Modbus fix story.

---

## Summary Counts

| Tier | Count |
|------|-------|
| CONFIRMED (empirical) | 2 |
| CRITICAL (code-grounded, unverified) | 2 |
| HIGH (code-grounded, unverified) | 7 |
| MEDIUM | 7 |
| LOW / observations / deferrals | 6 |
| CONFIRMED-CLEAN | 8 |
| **Total candidates** | **~30** |

All unconfirmed candidates require DF-VALIDATION-001 research-validation before issue filing.
