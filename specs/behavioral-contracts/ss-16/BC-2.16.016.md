---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-23T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: fix-pc-013-014-015
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
  - .factory/code-delivery/fix-pc-013-014-015/scope.md
input-hash: TBD
---

# BC-2.16.016: ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on process_arp Return Vec

## Description

`ArpAnalyzer::process_arp` returns a `Vec<Finding>` with NO upper bound on the number of
findings it may contain. Unlike the stream-reassembly analyzers (HTTP, TLS, Modbus, DNP3)
which bound their findings output via the reassembly layer `MAX_FINDINGS = 10,000` cap,
`process_arp` operates at the Ethernet link layer and bypasses the reassembly path entirely
(BC-2.16.015 Invariant 2). The binding table is separately bounded at `MAX_ARP_BINDINGS =
65,536` entries (BC-2.16.006) and storm counters at `MAX_STORM_COUNTERS = 4,096` entries;
these are MEMORY bounds on internal analyzer state, NOT output bounds on the findings Vec.
This absence of a findings cap is intentional design: ARP is a link-layer protocol and the
reassembly-layer MAX_FINDINGS gate does not apply to it.

## Preconditions

1. `ArpAnalyzer::process_arp` is called for each ARP frame in the capture.
2. `--arp` flag is active (analysis gate per BC-2.16.011).
3. The caller may supply any number of ARP frames, including counts exceeding 10,000.

## Postconditions

1. The `Vec<Finding>` returned across all `process_arp` calls is NOT truncated by any
   `MAX_FINDINGS` constant. If the capture contains N ARP events that each produce a
   finding (e.g., N D1 spoof events), the cumulative findings count equals N.
2. No `dropped_findings` counter is maintained by `ArpAnalyzer`. There is no mechanism
   to count or surface capped/dropped ARP findings (unlike Modbus and DNP3 analyzers
   which maintain such counters).
3. `ArpAnalyzer::summarize()` NEVER emits a `dropped_findings` key (BC-2.16.010 Invariant 1
   specifies exactly 11 required keys; adding a `dropped_findings` key would be a
   BC-2.16.010 breaking change).
4. The CLI `--help` text for `--arp` MUST document the absence of a findings cap. Operators
   analyzing adversarial captures with massive ARP-storm or ARP-spoof events must be informed
   that findings output can grow proportionally to the number of triggering frames, without
   any platform-imposed bound.

## Invariants

1. **No MAX_FINDINGS on ARP path**: `ArpAnalyzer` does NOT define or use a `MAX_FINDINGS`
   constant. The binding table cap (`MAX_ARP_BINDINGS = 65,536`, BC-2.16.006) and storm
   counter cap (`MAX_STORM_COUNTERS = 4,096`, BC-2.16.008) are MEMORY bounds on internal
   HashMaps; they do not cap the findings Vec.
2. **Contrast with stream-reassembly analyzers**: HTTP (`src/analyzer/http.rs`), TLS
   (`src/analyzer/tls.rs`), Modbus (`src/analyzer/modbus.rs`), and DNP3
   (`src/analyzer/dnp3.rs`) all operate via `TcpReassembler` which applies
   `MAX_FINDINGS = 10,000` from `src/reassembly/mod.rs:57`. ARP bypasses TCP reassembly
   entirely because ARP is an Ethernet-layer protocol, not a TCP/IP application protocol.
3. **Intentional design — not a missing implementation**: The absence of a cap is a
   deliberate choice for a forensics tool where users own their pcap files and need complete
   finding records. Adding a cap would be a behavioral change requiring a new BC, a
   `dropped_findings` counter in summarize (BC-2.16.010 breaking change), and a semver bump.
4. **Security awareness**: A malicious pcap with millions of ARP spoof or storm events can
   cause unbounded `Vec<Finding>` growth proportional to the number of triggering events.
   This is an accepted design trade-off for a CLI forensics tool. Operators analyzing
   untrusted capture files from adversarial sources should be aware of this behavior.
5. **Test accountability**: The behavioral test `test_BC_2_16_016_arp_findings_vec_has_no_cap`
   (described in the Canonical Test Vectors section below) serves as the regression guard.
   If a `MAX_FINDINGS` cap is accidentally added to the ARP path in the future, this test
   will fail, alerting maintainers to the behavioral change.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 0 ARP frames processed | `Vec<Finding>` is empty; no findings; compliant with Postcondition 1 (N=0) |
| EC-002 | 10,001 ARP spoof events (each causing a D1 MEDIUM finding) | findings.len() == 10,001; no cap applied; no dropped_findings |
| EC-003 | 65,536 ARP frames all from different sender_IPs, each with MAC rebind | Binding table evicts via LRU (BC-2.16.006); BUT findings.len() == 65,536 (one MEDIUM D1 per rebind); the cap on the binding TABLE does not cap FINDINGS |
| EC-004 | Capture with MAX_ARP_BINDINGS+1 distinct sender IPs, each rebinding once | findings.len() == MAX_ARP_BINDINGS+1; the LRU eviction of the oldest binding entry may cause the (MAX_ARP_BINDINGS+1)th IP to be treated as a new binding on re-arrival (no existing entry); this is a BC-2.16.006 behavior, not a findings-cap behavior |
| EC-005 | 1,000,000 identical ARP storm frames from one source MAC | D3 storm finding emitted once per storm-window trigger (BC-2.16.008 one-shot guard per window); total D3 findings << 1,000,000; storm counter cap (MAX_STORM_COUNTERS) limits unique MAC tracking, not findings count |

## Canonical Test Vectors

### Red Gate Test: `test_BC_2_16_016_arp_findings_vec_has_no_cap`

**Purpose:** Prove that no MAX_FINDINGS cap is applied to the ARP findings path. This test
PASSES after the cap-absence invariant is established and FAILS if a cap is accidentally
introduced.

**Location:** `src/analyzer/arp.rs` `#[cfg(test)] mod tests` OR
`tests/bc_2_16_story113_arp_tests.rs`

**Protocol:**
1. Create `ArpAnalyzer::new(spoof_threshold=1, storm_rate=u32::MAX)`.
   - `spoof_threshold=1` ensures the first rebind of any IP triggers a HIGH D1 finding
     immediately (per BC-2.16.004 EC-008).
   - `storm_rate=u32::MAX` suppresses D3 storm findings so findings are purely D1.
2. Synthesize N = 10,001 distinct ARP reply frames (`op=2`), each with a unique
   `sender_ip` (e.g., `10.0.0.1` through `10.0.39.17`) and alternating `sender_mac`
   values (e.g., even index → `AA:AA:AA:AA:AA:AA`, odd index → `BB:BB:BB:BB:BB:BB`).
   The alternating MAC pattern ensures each second frame for an IP triggers a D1 rebind.
3. Process all frames by calling `process_arp` in sequence. Accumulate all returned
   `Vec<Finding>` items into a single `all_findings: Vec<Finding>`.
4. **Assert:** `all_findings.len() == expected_d1_count` where `expected_d1_count` is
   the number of MAC-rebind events (approximately N/2 for the alternating pattern).
   The assertion MUST NOT be `all_findings.len() <= 10_000`.
5. Verify NO finding has been silently dropped by checking the count equals exactly the
   number of rebind events observed, with no plateau at 10,000.

| Analyzer config | Frame sequence | Expected findings.len() |
|---|---|---|
| spoof_threshold=1, storm_rate=MAX | 10,001 alternating-MAC pairs (same sender_ip, alternating MACs) | >= 10,001 D1 findings (no cap); exact count depends on rebind detection per BC-2.16.004 |
| spoof_threshold=3, storm_rate=MAX | 12,000 frames: 3 distinct MACs cycling on 4,000 distinct IPs | 4,000 MEDIUM + 4,000 HIGH findings == 8,000 total (no cap); findings.len() > 10,000 confirms unbounded path |
| spoof_threshold=1, storm_rate=MAX | 0 frames | findings.len() == 0 |

## Verification Properties

| VP-NNN | Property | Proof Method | Relationship |
|--------|----------|-------------|-------------|
| (none — no formal proof target; behavioral test is sufficient) | ARP findings Vec is not bounded by MAX_FINDINGS; findings.len() == number of triggering events processed | Unit test: `test_BC_2_16_016_arp_findings_vec_has_no_cap` (process >10,000 spoof events; assert findings.len() > 10,000 and == exact expected count) | primary |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per capabilities.md §CAP-16 — the unbounded findings output is a fundamental characteristic of the ARP Security Analysis capability's forensic design; documenting and contractually specifying this behavior is a direct property of how the ARP analyzer exposes its security findings to operators |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::process_arp); BC-2.16.015 (link-layer bypass of reassembly); BC-2.16.006 (binding table cap — distinct from findings cap) |
| Stories | (none yet — new BC authored for fix-pc-013-014-015; story decomposition TBD) |
| Feature | fix-pc-013-014-015 (PC-015 spec/doc fix) |
| MITRE Techniques | (none — design-invariant BC; no finding emission) |

## Related BCs

- BC-2.16.010 — depends on (summarize() 11-key contract; adding dropped_findings would be a BC-2.16.010 breaking change requiring its own version bump)
- BC-2.16.015 — composes with (link-layer bypass architectural invariant — the reason ARP does not use the reassembly MAX_FINDINGS path)
- BC-2.16.006 — contrasts with (binding table cap MAX_ARP_BINDINGS=65,536 is a MEMORY bound, not a findings-output bound; must not be confused)
- BC-2.16.008 — contrasts with (storm counter cap MAX_STORM_COUNTERS=4,096 is a MEMORY bound, not a findings-output bound)

## Architecture Anchors

- `src/analyzer/arp.rs` — `impl ArpAnalyzer { pub fn process_arp(...) -> Vec<Finding> }` — returns unbounded Vec
- `src/reassembly/mod.rs:57` — `const MAX_FINDINGS: usize = 10_000` — applies to HTTP/TLS/Modbus/DNP3 ONLY; NOT to ARP
- `src/analyzer/arp.rs` — `const MAX_ARP_BINDINGS: usize = 65_536` — binding TABLE cap; distinct from findings cap
- `src/analyzer/arp.rs` — `const MAX_STORM_COUNTERS: usize = 4_096` — storm counter TABLE cap; distinct from findings cap
- `src/cli.rs` lines 194–213 — `--arp` flag definition; `long_help` MUST document unbounded findings behavior (PC-015 doc fix)
- `.factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md §Invariant 2` — link-layer bypass rationale

## Story Anchor

(TBD — fix-pc-013-014-015 story decomposition; story-writer will populate)

## VP Anchors

- (none)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | fix-pc-013-014-015/scope.md §PC-015 (ARP Findings Cap Not Documented — clarification that NO cap exists); src/analyzer/arp.rs (no MAX_FINDINGS constant defined); src/reassembly/mod.rs:57 (MAX_FINDINGS=10,000 applies to HTTP/TLS/Modbus/DNP3 only) |
| **Confidence** | high — code confirms absence of MAX_FINDINGS on ARP path; intentional-design status confirmed by scope.md §PC-015 Fix Classification |
| **Extraction Date** | 2026-06-23 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (process_arp is a pure state-mutating computation) |
| **Global state access** | none (ArpAnalyzer is a pure-core struct) |
| **Deterministic** | yes — same sequence of frames always produces same findings count |
| **Thread safety** | single-threaded (consistent with wirerust single-threaded pipeline) |
| **Overall classification** | design-invariant BC — specifies the ABSENCE of a behavioral bound; verified by behavioral test rather than formal proof |
