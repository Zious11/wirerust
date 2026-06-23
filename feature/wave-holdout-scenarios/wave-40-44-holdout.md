---
document_type: holdout-scenario
version: "1.1"
status: skeleton
producer: product-owner
timestamp: 2026-06-14T00:00:00Z
waves: [40, 41, 42, 43, 44]
cycle: v0.7.0-arp
stories: [STORY-111, STORY-112, STORY-113, STORY-114, STORY-115]
feature_id: issue-009-arp-security-analyzer
github_issue: 9
mitre_version: enterprise-attack-v19.1 + ics-attack-19.1
mitre_version_note: "Enterprise ATT&CK v19.1 — aligns with arp-architecture-delta.md §5 (T1557.002 technique_info arm citations Enterprise v19.1) and src/mitre.rs. ics-attack-19.1 unchanged."
confirmed_thresholds:
  spoof_rebind_escalation_default: 3
  arp_flap_window_secs: 60
  arp_storm_rate_default: 50
  max_arp_bindings: 65536
  max_storm_counters: 4096
forward_declarations:
  seeded_technique_id_count_post_story114: 25
  emitted_ids_count_post_story114: 17
  note: "BC-2.10.005 and BC-2.10.008 carry PLANNED marker until STORY-114 merges (current src 23/15)"
---

# Waves 40–44 Holdout Scenario Skeletons: ARP Security Analyzer (v0.7.0)

> **SKELETON STATUS:** This file is a Phase 3 skeleton. Concrete byte-level test vectors,
> exact PCAP byte sequences, and precise numerical postcondition assertions are authored by
> the holdout-evaluator agent during Phase 4, AFTER F3 story decomposition produces wave
> assignments and implementation code exists for evaluation.
>
> **Purpose:** Validates that the ARP Security Analyzer (SS-16) correctly detects ARP
> cache poisoning (D1, T0830/T1557.002), gratuitous ARP / GARP-conflicts (D2 + BC-2.16.014), ARP
> storm rate anomalies (D3), and malformed/L2-L3 mismatch frames (D11/D12). Verifies
> that the etherparse 0.20 decode-path changes (STORY-111) introduce no regressions and
> that the ArpAnalyzer lifecycle (binding table, per-MAC storm counters, summarize) operates
> correctly across the full v0.7.0 feature cycle.
>
> **Evaluator note:** These scenarios are BLIND — the evaluator runs the finished
> implementation against synthetic PCAP sequences or byte vectors WITHOUT reading
> implementation source code. Pass/fail is determined solely by matching the EXACT EXPECTED
> OUTPUT specified in each scenario.
>
> **MITRE discipline:** ARP findings carry T0830 (ICS Collection,
> MitreTactic::IcsCollection, TA0100) and/or T1557.002 (Enterprise Credential Access,
> MitreTactic::CredentialAccess) for D1/D2(conflicts)/D12. D3 storm carries no MITRE tag
> (T0814 deferred per DF-VALIDATION-001). D11 malformed carries no MITRE tag.
>
> **Threshold source:** ALL numeric thresholds below are wirerust engineering defaults
> (arp-architecture-delta.md §3.2). None are borrowed from external standards.

---

## Per-Wave Gate Summary

| Wave | Story | Detection / Scope | Gate Criteria |
|------|-------|-------------------|---------------|
| 40 | STORY-111 | etherparse 0.20 migration; DecodedFrame enum | BC-2.02.009 three-way postcondition; VP-008 return-type update; no regression |
| 41 | STORY-112 | extract_arp_frame; decode-vs-analysis separation; ArpAnalyzer stub; VP-024 Sub-A | ARP Request/Reply extraction correct; BC-2.16.015 decode-vs-analysis invariant; Sub-A Kani harnesses pass |
| 42 | STORY-113 | D2 GARP; D11 malformed; D12 mismatch (detection only, mitre_techniques:[]); binding table; summarize(); --arp flag; VP-024 Sub-B/C/D | All 7 STORY-113 BCs validated; Sub-B/C/D harnesses pass; D12 finding has empty MITRE at this wave |
| 43 | STORY-114 | D1 spoof escalation MEDIUM→HIGH; GARP-conflicts; MITRE T0830+T1557.002 for D1/D2/D12; VP-007 5-part atomic | rebind_count threshold correct; MITRE arms added to D1 AND D12; SEEDED=25; EMITTED=17; cargo test mitre green |
| 44 | STORY-115 | D3 storm rate; --arp-storm-rate CLI flag; storm_findings summarize key | Storm one-shot per 60s window; rate formula count/max(1,elapsed); CLI override effective; BC-2.16.010 storm_findings key (key 8 of 11, defined from STORY-113) value wired by STORY-115 |

---

## HS-W43-ARP-D1: ARP Cache Poisoning (D1) — MEDIUM→HIGH Spoof Escalation

**Detection:** D1 ARP Spoof / Cache Poisoning
**Scope:** STORY-114 (BC-2.16.004, BC-2.16.012)
**Priority:** P0 (must-pass)
**Wave:** 43
**MITRE:** T0830 (IcsCollection, TA0100), T1557.002 (CredentialAccess)
**assumption_source:** null
**risk_source:** null

### Scenario A — Default threshold escalation (MEDIUM then HIGH within 60s)

**Setup:**
A fresh `ArpAnalyzer` instance (default threshold: SPOOF_REBIND_ESCALATION_DEFAULT = 3,
ARP_FLAP_WINDOW_SECS = 60). The binding table is empty at start (ts = T0).

**Frame sequence (all timestamps within 60s of T0):**

1. Frame 1 (ts=T0): ARP Reply — sender_ip=192.168.1.10, sender_mac=AA:BB:CC:DD:EE:01
   (legitimate; establishes binding entry).
2. Frame 2 (ts=T0+1): ARP Reply — sender_ip=192.168.1.10, sender_mac=AA:BB:CC:DD:EE:02
   (first rebind — different MAC for same IP; rebind_count=1).
3. Frame 3 (ts=T0+2): ARP Reply — sender_ip=192.168.1.10, sender_mac=AA:BB:CC:DD:EE:03
   (second rebind; rebind_count=2).
4. Frame 4 (ts=T0+3): ARP Reply — sender_ip=192.168.1.10, sender_mac=AA:BB:CC:DD:EE:04
   (third rebind; rebind_count=3 >= threshold).

**Expected findings (Phase 4 evaluator completes exact fields):**

- Frame 2: one Finding emitted with:
  - verdict: Anomaly (MEDIUM confidence)
  - techniques: [T0830, T1557.002]
  - summary contains "192.168.1.10" and "ARP" and "rebind" or "spoof"
- Frame 3: one Finding emitted with:
  - verdict: Anomaly (MEDIUM confidence)
  - techniques: [T0830, T1557.002]
- Frame 4: one Finding emitted with:
  - verdict: Likely (HIGH confidence) — escalation triggered
  - techniques: [T0830, T1557.002]
  - summary contains escalation signal ("high confidence" or "repeated rebind")

**Assertions:**
1. Frame 1 produces zero findings (initial binding, no conflict).
2. Frame 2 produces exactly one finding with confidence MEDIUM.
3. Frame 4 produces exactly one finding with confidence HIGH (rebind_count >= 3).
4. All D1 findings carry techniques array containing both "T0830" and "T1557.002".
5. `spoof_high_emitted` flag set to `true` after Frame 4; subsequent same-IP rebinds within window do NOT re-emit HIGH (one-shot per window).
6. No panic. No crash. No output on stdout before findings.

### Scenario B — --arp-spoof-threshold override (threshold=1, HIGH on first rebind)

**Setup:** Same as A but with `--arp-spoof-threshold 1` (SPOOF_REBIND_ESCALATION_DEFAULT overridden to 1).

**Frame sequence:**
1. Frame 1: sender_ip=10.0.0.5, sender_mac=DE:AD:BE:EF:00:01 (establishes binding).
2. Frame 2: sender_ip=10.0.0.5, sender_mac=DE:AD:BE:EF:00:02 (first rebind; threshold=1).

**Assertions:**
1. Frame 2 produces exactly one finding with confidence HIGH (not MEDIUM first).
2. Techniques: [T0830, T1557.002].
3. `spoof_high_emitted == true` after Frame 2.

### Evaluation Rubric

- **Correctness** (weight: 0.5): MEDIUM on rebind_count < threshold; HIGH on rebind_count >= threshold.
- **MITRE fidelity** (weight: 0.3): Both T0830 and T1557.002 appear in finding techniques array.
- **One-shot guard** (weight: 0.2): HIGH finding emitted at most once per flap window per IP.

---

## HS-W42-ARP-D2: GARP Detection and GARP-Conflicts Escalation (D2 + BC-2.16.014)

**Detection:** D2 Gratuitous ARP (BC-2.16.003) + GARP-that-conflicts escalation (BC-2.16.014)
**Scope:** STORY-113 for D2 baseline (wave 42); STORY-114 for D2+D1 interaction (wave 43)
**Priority:** P0 (must-pass)
**Wave:** 42 (D2 baseline), 43 (GARP-conflicts escalation)
**MITRE for D2 conflicts:** T0830, T1557.002 (when GARP conflicts with existing binding)
**assumption_source:** null
**risk_source:** null

### Scenario A — Benign GARP (sender_ip == target_ip, no binding conflict) — Wave 42

**Setup:** Fresh ArpAnalyzer. Binding table empty at start.

**Input:** ARP Request frame — sender_ip=192.168.1.1, target_ip=192.168.1.1,
sender_mac=AA:BB:CC:DD:EE:FF, target_mac=00:00:00:00:00:00 (standard GARP probing form).

**Assertions:**
1. `is_gratuitous_arp(frame)` returns `true` (sender_ip == target_ip; opcode-agnostic).
2. Exactly one finding emitted with confidence LOW.
3. Finding verdict: Anomaly (LOW / Inconclusive class).
4. No D1 spoof finding emitted (no binding conflict — table was empty).
5. No MITRE techniques on this finding (LOW GARP with no conflict has no MITRE tag per §3.3).
6. Binding table entry for 192.168.1.1 → AA:BB:CC:DD:EE:FF created (insert_binding_lru called).

### Scenario B — GARP-that-Conflicts (D2 + D1 co-emission) — Wave 43

**Setup:** ArpAnalyzer with binding table pre-populated:
  192.168.1.1 → 11:22:33:44:55:66 (legitimate owner)

**Input:** ARP Reply — sender_ip=192.168.1.1, target_ip=192.168.1.1,
sender_mac=AA:BB:CC:DD:EE:FF (attacker impersonating 192.168.1.1).

**Assertions (per BC-2.16.014 GARP escalation rule and arp-architecture-delta.md §3.3):**
1. `is_gratuitous_arp(frame)` returns `true`.
2. Two findings emitted on this single frame:
   a. GARP finding — confidence MEDIUM (upgraded from LOW because of binding conflict).
      techniques: [T0830, T1557.002].
   b. D1 spoof finding — confidence MEDIUM (first rebind; rebind_count=1 < 3).
      techniques: [T0830, T1557.002].
3. GARP finding confidence is MEDIUM, NOT LOW (conflict upgrade applied).
4. Both findings carry techniques [T0830, T1557.002].

### Evaluation Rubric

- **Correctness** (weight: 0.4): LOW for benign GARP; MEDIUM + co-emission for conflicting GARP.
- **Opcode agnosticism** (weight: 0.2): sender_ip==target_ip triggers GARP detection for both op=1 and op=2.
- **MITRE fidelity** (weight: 0.3): T0830 + T1557.002 present on conflicting GARP only.
- **No false positive** (weight: 0.1): Benign GARP does not emit D1 spoof finding.

---

## HS-W44-ARP-D3: ARP Storm Rate Detection (D3)

**Detection:** D3 ARP Storm (BC-2.16.008, BC-2.16.013)
**Scope:** STORY-115 (wave 44)
**Priority:** P0 (must-pass) for Scenarios A and B (core D3 storm detection, BC-2.16.008); P1 (should-pass) for Scenario C (--arp-storm-rate CLI override, BC-2.16.013)
**Wave:** 44
**MITRE:** None (T0814 deferred per DF-VALIDATION-001)
**assumption_source:** ARP-AMB-003 RESOLVED (storm-rate formula: count/max(1,elapsed))
**risk_source:** null

### Scenario A — Storm threshold exceeded (default rate 50 fps)

**Setup:** Fresh ArpAnalyzer (ARP_STORM_RATE_DEFAULT = 50).

**Frame sequence:** 101 ARP frames from source MAC AA:BB:CC:DD:EE:01.
- Frames 1-101 all have timestamp ts=T0 (same second, elapsed=0).
- Rate = 101 / max(1, 0) = 101 / 1 = 101 fps (exceeds threshold 50).

**Assertions:**
1. At frame 50 (count=50 >= threshold=50): exactly one D3 storm finding emitted (one-shot per window fires at first breach of threshold).
2. Finding confidence: MEDIUM.
3. Finding summary contains sender MAC "AA:BB:CC:DD:EE:01" and rate/threshold reference.
4. Finding carries NO MITRE techniques (T0814 deferred).
5. `storm_emitted == true` for MAC AA:BB:CC:DD:EE:01 after frame 50 emission.
6. Frames 51-101 (same MAC, same window): zero additional findings emitted (one-shot guard active after frame 50).

### Scenario B — Same-second denominator safety (no divide-by-zero)

**Setup:** Fresh ArpAnalyzer. All frames from same MAC at ts=N (elapsed = N - N = 0).

**Input:** 51 ARP frames from source MAC BB:CC:DD:EE:FF:00, all with timestamp=0.

**Assertions (ARP-AMB-003 RESOLVED — rate = count/max(1,0) = 51/1 = 51):**
1. Rate computed as 51 (not divide-by-zero; max(1,0) = 1).
2. 51 > 50 threshold: one finding emitted.
3. No panic. No NaN. No infinity.

### Scenario C — --arp-storm-rate override (P1 should-pass)

**Priority note:** This scenario is P1 (should-pass). The core D3 storm detection (Scenarios A and
B, BC-2.16.008) is P0 must-pass because it is a primary BC for STORY-115. The CLI override
(BC-2.16.013) is a configuration affordance; it is tested here as P1 consistent with HS-W44-003
in HS-INDEX.

**Setup:** ArpAnalyzer with `--arp-storm-rate 10` (threshold lowered to 10 fps).

**Frame sequence:** 11 ARP frames from source MAC CC:DD:EE:FF:00:11, timestamps T0 to T0+1
(elapsed = 1 second; rate = 11 / max(1,1) = 11 fps > threshold 10).

**Assertions:**
1. Storm finding emitted after 11 frames (rate 11 > custom threshold 10).
2. With default threshold (50), 11 frames in 1 second would NOT trigger storm (11 < 50).
   This scenario confirms the CLI override is effective.

### Scenario D — storm_findings key in summarize() output (BC-2.16.010, key 8 of 11)

**Setup:** ArpAnalyzer after processing Scenario A (storm detected for one MAC).

**Assertions for summarize() output:**
1. Key `storm_findings` present in AnalysisSummary. `storm_findings` is canonical key 8 of
   the 11-key set defined by BC-2.16.010 (defined from STORY-113); STORY-115 wires its value
   by populating storm detection counts — the key itself is not a new addition.
2. `storm_findings >= 1` (at least one storm finding was emitted).
3. The other 10 canonical BC-2.16.010 keys also present:
   `frames_analyzed`, `request_count`, `reply_count`, `other_opcode_count`,
   `bindings_tracked`, `spoof_findings`, `garp_findings`, `mismatch_findings`,
   `malformed_findings`, `malformed_frames`.

### Evaluation Rubric

- **Rate formula** (weight: 0.3): count/max(1,elapsed) — no divide-by-zero, correct fps.
- **One-shot guard** (weight: 0.25): one finding per MAC per 60s window; no duplicates.
- **CLI override** (weight: 0.2): --arp-storm-rate changes threshold correctly.
- **No MITRE** (weight: 0.1): T0814 absent from storm findings.
- **summarize() completeness** (weight: 0.15): storm_findings key present and non-zero.

---

## HS-W42-ARP-D11D12: Malformed ARP (D11) and L2/L3 Mismatch (D12)

**Detection:** D11 Malformed (BC-2.16.009) + D12 L2/L3 Mismatch (BC-2.16.007)
**Scope:** STORY-113 (wave 42) — detection only; STORY-114 (wave 43) — MITRE attachment
**Priority:** P0 (must-pass)
**Wave:** 42 (detection), 43 (MITRE attachment — see Scenario C)
**MITRE for D12 at wave 42:** [] (empty — catalog not seeded until STORY-114)
**MITRE for D12 at wave 43:** T0830, T1557.002 (attached co-committed with src/mitre.rs catalog seeding)
**assumption_source:** null
**risk_source:** null

### Scenario A — D11 Malformed ARP (non-Ethernet/IPv4 hw/proto sizes)

**Setup:** Call `extract_arp_frame` with an ARP payload where:
- Hardware address length (hlen) != 6 (not Ethernet MAC size), OR
- Protocol address length (plen) != 4 (not IPv4 size).

**Input example (Phase 4 evaluator provides concrete bytes):**
ARP payload with hlen=8, plen=16 (not Ethernet/IPv4 format).

**Assertions (per BC-2.16.009 and STORY-113 AC-011):**
1. `extract_arp_frame(...)` returns `None` (malformed; cannot extract valid ArpFrame fields).
2. On receiving `None`, `ArpAnalyzer.process_arp` route (or the main.rs dispatch) emits
   exactly one finding with confidence LOW and category Malformed.
3. The D11 malformed finding carries `mitre_techniques: []` (empty list). No MITRE technique
   tags — specifically: T0830, T1557.002, and T0814 are ALL absent from this finding.
   A blind evaluator MUST assert that the techniques array is empty, not merely that specific
   tags are missing.
4. `malformed_frames` counter incremented by 1 in ArpAnalyzer state.
5. `frames_analyzed` counter NOT incremented for this frame (per ARP-AMB-004 RESOLVED:
   malformed frames excluded from frames_analyzed).
6. No D1/D2/D12 finding emitted for this frame (malformed frame cannot trigger behavioral detections).
7. No panic.

### Scenario B — D12 L2/L3 Mismatch (outer Ethernet MAC differs from ARP sender hardware address) — Wave 42 (STORY-113)

**Setup:** ArpAnalyzer with empty binding table. Frame constructed as:
- Ethernet frame with outer source MAC = EE:FF:00:11:22:33
- ARP sender hardware address (sender_mac) = AA:BB:CC:DD:EE:FF
- (outer_src_mac != sender_mac → D12 mismatch condition)

> **Sequencing note (Pass-12 D12-MITRE fix, per BC-2.16.007's cross-story delivery note):**
> At wave 42, STORY-113 DETECTS D12 and emits the finding, but the src/mitre.rs catalog is
> not yet seeded. The MITRE attachment (T0830, T1557.002) is applied in STORY-114 (wave 43),
> co-committed with catalog seeding. See Scenario C below for the wave-43 MITRE assertion.
> This mirrors the D1 deferral pattern already documented in STORY-113 'Crucial boundary' D1-deferral note.

**Assertions (per BC-2.16.007, wave-42 intermediate state):**
1. `frame.outer_src_mac == Some([0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33])`.
2. `frame.sender_mac == [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]`.
3. Exactly one D12 finding emitted with confidence MEDIUM.
4. Finding techniques: [] (empty — MITRE attachment deferred to STORY-114, wave 43).
5. Summary contains both MAC addresses (outer and sender).

### Scenario C — SLL capture: outer_src_mac=None suppresses D12 (no false positive) — Wave 42

**Setup:** ArpFrame where outer_src_mac = None (Linux SLL cooked capture has no Ethernet header).

**Assertions:**
1. No D12 finding emitted (outer_src_mac is None; comparison not possible).
2. No panic. Zero findings if no other conditions triggered.
3. This is the no-false-positive guard for captures without Ethernet framing.

### Scenario D — D12 MITRE Attachment (wave 43, STORY-114)

**Wave:** 43
**Scope:** STORY-114 — co-committed with src/mitre.rs catalog seeding and VP-007 5-part atomic update

**Setup:** Same frame as Scenario B (outer_src_mac=Some(EE:FF:00:11:22:33), sender_mac=AA:BB:CC:DD:EE:FF).
After STORY-114 merges, re-run the Scenario B input against the STORY-114 implementation (catalog seeded).

> This scenario asserts that the MITRE attachment added in STORY-114 is correctly applied to
> D12 findings. It is the complement of Scenario B: Scenario B validates the wave-42 detection
> (finding emitted, no MITRE); Scenario D validates the wave-43 MITRE attachment (same detection
> conditions, now with full technique array). Together they verify the two-wave delivery chain
> described in BC-2.16.007's cross-story delivery note.

**Assertions (per BC-2.16.007 final-state postconditions, post-STORY-114):**
1. Exactly one D12 finding emitted with confidence MEDIUM.
2. Finding techniques: ["T0830", "T1557.002"] (catalog now seeded; attachment applied).
3. `technique_info` for T0830 resolves to a non-empty entry (IcsCollection / TA0100 tactic arm per ADR-008 Decision 6 — corrected from LateralMovement).
4. `technique_info` for T1557.002 resolves to a non-empty entry (CredentialAccess tactic arm per ADR-008 Decision 6).
5. `cargo test mitre` green — EMITTED_IDS includes T0830 and T1557.002 (VP-007 5-part atomic assertion).
6. Summary contains both MAC addresses (outer and sender) — unchanged from Scenario B.

### Evaluation Rubric

- **D11 correctness** (weight: 0.25): extract_arp_frame returns None; malformed_frames incremented; frames_analyzed not incremented.
- **D12 wave-42 detection** (weight: 0.25): outer_src_mac != sender_mac triggers MEDIUM finding with mitre_techniques: [] (detection only; no MITRE at wave 42).
- **D12 wave-43 MITRE attachment** (weight: 0.2): after STORY-114, same D12 conditions produce MEDIUM finding with T0830+T1557.002 and resolved technique_info arms.
- **No false positive** (weight: 0.15): outer_src_mac=None produces zero D12 findings.
- **Independence** (weight: 0.15): D11 and D12 are independent detections; each fires only on its specific condition.

---

## HS-W44-ARP-CORPUS: Real-World ARP Corpus Validation

**Detection:** All ARP detections (D1, D2, D3, D11, D12) end-to-end
**Scope:** STORY-115 (wave 44) — final integration gate
**Priority:** P0 (must-pass)
**Wave:** 44
**MITRE:** T0830, T1557.002 (for D1/D2-conflicts/D12); none for D3/D11
**assumption_source:** null
**risk_source:** null

### Scenario A — Known-Good Corpus (Zero False-Positive Rate)

**Corpus source:** Wireshark wiki sample captures or equivalent public LAN traffic trace
containing normal ARP resolution traffic (ARP Requests followed by expected ARP Replies;
stable IP→MAC bindings; no rebinds; no rate anomalies).

**Suggested source (Phase 4 evaluator confirms availability):**
Wireshark wiki `sample-captures/arp-storm.pcap` filtered to stable-LAN-only portion, OR
any public /24 LAN trace from Wireshark sample captures. The source must be publicly
available and reproducible (not a private capture).

**Command:**
```
wirerust analyze --arp <corpus-pcap>
```

**Expected outcomes:**
1. Exit code 0.
2. Zero D1 spoof findings (no IP→MAC rebinds in a stable LAN trace).
3. Zero D12 mismatch findings (all Ethernet frames have matching outer/sender MACs).
4. Zero or minimal D3 storm findings (normal LAN ARP does not exceed 50 fps per-MAC).
5. Zero D11 malformed findings (well-formed captures use standard Ethernet/IPv4 ARP).
6. D2 GARP findings may be present (GARP is a normal network event; LOW confidence expected).
7. summarize() output is valid JSON with all 11 required keys.
8. `frames_analyzed + malformed_frames == total_arp_frames_in_pcap`.

**False-positive threshold:** Zero D1, D12 findings. D3: zero or ≤ 1 (only if PCAP contains
a legitimate storm event). D11: zero.

### Scenario B — Known-Problematic Corpus (Expected Detection)

**Corpus source:** A crafted synthetic PCAP or publicly available CTF/research pcap
demonstrating explicit ARP cache poisoning. The pcap must contain:
- An attacker sending unsolicited ARP Reply frames rebinding a victim IP to an attacker MAC.
- At least 3 rebinds within 60 seconds (to trigger HIGH confidence escalation).
- Ideally: attacker also sends GARPs to pre-populate victim caches.

**Suggested construction (Phase 4 evaluator creates or locates):**
Craft a minimal synthetic PCAP (using scapy or equivalent) with the sequence:
1. ARP Reply: 192.168.1.100 → 11:22:33:44:55:66 (legitimate binding established)
2. ARP Reply: 192.168.1.100 → AA:BB:CC:DD:EE:FF (attacker rebinds; MEDIUM finding)
3. ARP Reply: 192.168.1.100 → AA:BB:CC:DD:EE:FF (second rebind; MEDIUM finding)
4. ARP Reply: 192.168.1.100 → AA:BB:CC:DD:EE:FF (third rebind; HIGH finding emitted)

**Command:**
```
wirerust analyze --arp <poisoning-pcap> --output-format json
```

**Expected outcomes:**
1. Exit code 0 (findings do not cause non-zero exit; only decode errors do).
2. At least one D1 finding with confidence HIGH (T0830, T1557.002).
3. At least two D1 findings with confidence MEDIUM (first and second rebinds).
4. JSON output contains `techniques: ["T0830", "T1557.002"]` on D1 findings.
5. `spoof_findings >= 3` in summarize() output.
6. `spoof_high_emitted == true` for 192.168.1.100 after third rebind.

**False-negative threshold:** At least one HIGH-confidence D1 finding MUST be present.
A run producing zero D1 findings constitutes a false-negative failure.

### Scenario C — Regression on Existing Analyzers (Post-Waves-40-44)

**Scope:** Verify no regression introduced by etherparse 0.20 migration and DecodedFrame
enum changes on all pre-ARP analyzers.

**Command:**
```
wirerust analyze --http --tls --dns <known-good-ip-traffic-pcap>
```

**Assertions:**
1. Same findings produced as pre-ARP baseline run (zero new false positives; zero missing findings).
2. `decode_packet` returning `Result<DecodedFrame>` does not affect IP-path analysis correctness.
3. VP-008 fuzz harness remains green (`cargo fuzz run decode_packet` completes with no panics).
4. VP-004 dispatcher oracle: TLS/HTTP on port 20000 is not stolen by ARP routing.
5. `cargo test mitre` green (VP-007 5-part atomic update verified — SEEDED=25, EMITTED=17).

### Evaluation Rubric

- **False positive rate** (weight: 0.3): Known-good corpus produces zero D1/D12; ≤1 D3.
- **Detection fidelity** (weight: 0.35): Known-problematic corpus produces HIGH D1 findings with T0830+T1557.002.
- **Regression guard** (weight: 0.25): Existing analyzers unaffected; VP-008/VP-004 green.
- **summarize() completeness** (weight: 0.1): All 11 keys present in both corpus runs.

---

## ARP Feature Holdout Skeleton Summary

| Scenario ID | Detection | Wave | Story | BCs | Priority |
|-------------|-----------|------|-------|-----|----------|
| HS-W43-ARP-D1 | D1 ARP Spoof / Cache Poisoning (MEDIUM→HIGH escalation) | 43 | STORY-114 | BC-2.16.004, BC-2.16.012 | P0 |
| HS-W42-ARP-D2 | D2 GARP (benign + GARP-conflicts escalation BC-2.16.014) | 42, 43 | STORY-113, STORY-114 | BC-2.16.003, BC-2.16.014, BC-2.16.004 | P0 |
| HS-W44-ARP-D3 | D3 ARP Storm Rate Detection (Scenarios A+B: core storm, P0; Scenario C: --arp-storm-rate override, P1) | 44 | STORY-115 | BC-2.16.008 (P0), BC-2.16.013 (P1), BC-2.16.010 | P0/P1 (split — see scenario priorities) |
| HS-W42-ARP-D11D12 | D11 Malformed + D12 L2/L3 Mismatch | 42 | STORY-113 | BC-2.16.009, BC-2.16.007 | P0 |
| HS-W44-ARP-CORPUS | Real-world corpus (known-good + known-problematic + regression) | 44 | STORY-115 | BC-2.16.004, BC-2.16.010, VP-008, VP-004, VP-007 | P0 |

Total skeleton scenarios: 5 (4 fully P0 must-pass; 1 split P0/P1 — HS-W44-ARP-D3 Scenarios A+B are P0, Scenario C is P1).
Phase 4 evaluator completes: concrete byte sequences, exact finding field values, PCAP sources.
