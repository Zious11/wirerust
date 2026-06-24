---
document_type: behavioral-contract
level: L3
version: "1.10"
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
  - "v1.4: Pass-4 remediation F-B4-H01 (BindingEntry Architecture Anchor: last_seen_ts added); F-B4-M03 (PC2 mac-update timing: Step 4 added to intra-event sequence; PC2 cross-references Step 4). — 2026-06-12"
  - "v1.5: Pass-7 remediation F-B7-H01/F-B7-H02: added tactic-anchor cross-reference to Invariant 4 — T0830 maps to MitreTactic::LateralMovement and T1557.002 to MitreTactic::CredentialAccess per ADR-008 Decision 6 (merge-by-name policy); the F3/STORY-114 implementer wires these in technique_info. — 2026-06-12"
  - "v1.6: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.7: Pass-21 HIGH remediation: requalified VP-024 Sub-C in all three loci (Verification Properties table, VP Anchors section, Purity Classification) as INDIRECT/substrate dependency only — Sub-C's primary anchor is BC-2.16.005; BC-2.16.004 is NOT a VP-024 Sub-C formally-verified BC; D1 spoof-escalation postconditions remain unit-tested per EC coverage. — 2026-06-14"
  - "v1.8: F5 ICS tactic-ID correctness fix (issue #64 follow-up). Invariant 4 tactic anchor corrected: T0830 maps to MitreTactic::IcsCollection (ICS Collection, TA0100) NOT MitreTactic::LateralMovement (TA0008). ADR-008 Decision 6 merge-by-name policy for T0830 is SUPERSEDED by f5-ics-technique-tactic-authoritative.md (VALIDATED; MITRE ATT&CK for ICS v19.1, page-verified: https://attack.mitre.org/techniques/T0830/ assigns Collection/TA0100). The F5 implementer wires IcsCollection in technique_info; BC-2.10.007 v1.9 carries the updated tactic row. — 2026-06-23"
  - "v1.9: fix-pc-013-014-015 PC-013 spec anchor — fail-safe degradation invariant added (Invariant 6). When any binding-table HashMap lookup that is expected by construction to succeed (entry present because it was just confirmed via contains_key or get in the same call frame) is unexpectedly absent, process_arp and emit_d1_spoof_finding_impl SKIP the current operation without panicking. Anchors the PC-013 code fix: replace .expect() at arp.rs lines 555/576/642/827 with if-let guards (Option (a) from scope.md §PC-013 Fix Classification). No behavioral change for well-formed analyzer state; fail-safe applies only to invariant violations that are logically unreachable in current single-threaded code but could become reachable after future refactoring. — 2026-06-23"
  - "v1.10: PC-013 spec correction — human-approved decision (2026-06-24): REMOVE Invariant 6 (fail-safe degradation / silently skip) and REVERT the .expect()→if-let-guard mandate. Research (codebase survey + external Rust best practice, .factory/cycles/fix-pc-013-014-015/research/pc-013-invariant-idiom.md) converged that all four .expect() sites at arp.rs lines 555/576/642/827 operate on state that is guaranteed present by construction — the .expect() is a loud, self-documenting tripwire consistent with the project convention (decoder.rs, arp.rs documented .expect() for proven invariants). The v1.9 'silently skip' invariant mandated a fail-open ANTI-PATTERN. Invariant 6 replaced with a by-construction panic-freedom statement (Invariant 6 — new). EC-011/EC-012 reframed from 'missing entry → skip' to by-construction characterization entries with regression test anchors. No production code change. — 2026-06-24"
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
  - .factory/phase-f1-delta-analysis/mitre-arp-research.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.004: ARP Spoof Detection — IP→MAC Rebind Emits MEDIUM then HIGH Finding

## Description

When `ArpAnalyzer::process_arp` observes a frame whose `sender_ip` is already in the binding
table with a different `sender_mac`, it classifies the event as a potential ARP cache-poisoning
attack (D1). The first rebind emits a Finding at MEDIUM severity (Anomaly). If the rebind count
for that IP reaches or exceeds `spoof_threshold` (default `SPOOF_REBIND_ESCALATION_DEFAULT = 3`,
a wirerust engineering default) distinct MAC values within `ARP_FLAP_WINDOW_SECS = 60` seconds
(a wirerust engineering default), the finding escalates to HIGH severity (Likely). MITRE
techniques T0830 and T1557.002 are attached to all spoof findings. These thresholds are
wirerust engineering defaults; no authoritative numeric standard exists in the literature
(mitre-arp-additional-detections.md §4, CRITICAL CORRECTION).

## Preconditions

1. `frame` is a fully-populated `ArpFrame` with non-zero `sender_ip` and `sender_mac`.
2. `ArpAnalyzer.bindings` contains an entry for `frame.sender_ip` with a MAC value that
   differs from `frame.sender_mac` (byte-wise inequality).
3. `--arp` flag is active (analysis gate per BC-2.16.011).
4. `timestamp_secs` is the packet timestamp in Unix seconds (u32).

## Postconditions

**D1 finding emission rule — exactly one finding per rebind event:**

A rebind emits EXACTLY ONE D1 Finding per event. Severity is determined by the escalation
state AT THE TIME OF EMISSION. The intra-event ordering on each rebind is fixed and must
be implemented in this exact sequence:

1. For each rebind event where `frame.sender_mac != bindings[sender_ip].mac`:
   a. **Step 1 — increment**: `rebind_count` is incremented (from N to N+1).
   b. **Step 2 — set first_rebind_ts if unset**: if `first_rebind_ts` is not yet set (i.e.,
      this is the first rebind), set `first_rebind_ts = timestamp_secs`. This ensures that
      when `spoof_threshold = 1`, the elapsed time is `timestamp_secs - first_rebind_ts = 0`,
      which is <= `ARP_FLAP_WINDOW_SECS`, so the HIGH condition can fire on the very first rebind.
   c. **Step 3 — evaluate escalation using the just-set first_rebind_ts**:
      If `(rebind_count >= spoof_threshold) AND (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND (spoof_high_emitted == false)`:
      - Severity = **HIGH** (Likely). `spoof_high_emitted` is set to `true`.
   d. Otherwise:
      - Severity = **MEDIUM** (Anomaly).
   e. A single Finding is emitted:
      - `confidence: HIGH` (case c — HIGH path) or `MEDIUM` (case d — MEDIUM path)
      - `finding_type: Anomaly`
      - `description` indicating IP→MAC binding change / potential ARP cache poisoning
      - `mitre_techniques: ["T0830", "T1557.002"]`
      - Includes the conflicting IP, old MAC, and new MAC in the evidence
   f. **Step 4 — mac update:** `bindings[sender_ip].mac` is set to `frame.sender_mac`
      (last-write-wins per BC-2.16.005) AFTER escalation evaluation and finding emission.
      The mac write occurs exactly once per frame (see Postcondition 2).

Note: with `--arp-spoof-threshold 1`, `rebind_count` reaches 1 on the first rebind and
`first_rebind_ts` is set to `timestamp_secs` (Step 2), giving elapsed=0 <= window. Case (c)
fires immediately if `spoof_high_emitted == false` — HIGH is emitted on the first rebind.
There is no "first rebind is always MEDIUM" guarantee when threshold=1.

2. `bindings[sender_ip].mac` is updated to `frame.sender_mac` (last-write-wins per BC-2.16.005). This write occurs at Step 4 in the intra-event sequence (after escalation evaluation and finding emission per Postcondition 1.f).
3. `bindings[sender_ip].first_rebind_ts` is set to `timestamp_secs` on the first rebind of
   a flap window (when `first_rebind_ts` is None per Step 2); not updated on subsequent
   rebinds within the same window; re-set on the first rebind after a window reset per
   Postcondition 5.
4. `bindings[sender_ip].spoof_high_emitted` is set to `true` after the first HIGH finding
   (one-shot guard: no additional HIGH findings for this IP in the current flap window;
   MEDIUM findings continue to be emitted on each additional rebind until window resets).

**Flap window reset (Postcondition 5):**
5. After `ARP_FLAP_WINDOW_SECS` has elapsed since `first_rebind_ts`, the flap window resets:
   `rebind_count` is reset to 0, `first_rebind_ts` is cleared, `spoof_high_emitted` is reset
   to `false`. The next rebind after a reset is treated as the first rebind for this IP.

**Threshold semantics (Postconditions 6-7):**
6. `SPOOF_REBIND_ESCALATION_DEFAULT = 3` is a wirerust engineering default. It is overridable
   via `--arp-spoof-threshold` CLI flag (BC-2.16.012). Not derived from any external standard.
7. `ARP_FLAP_WINDOW_SECS = 60` is a wirerust engineering default. Not a CLI flag in v0.7.0.

## Invariants

1. **First-rebind MEDIUM rationale**: a single MAC change is common on networks with DHCP churn,
   VM migration, NIC replacement, or HA/VRRP failover. MEDIUM/Anomaly gives analysts a signal
   without forcing investigation of every DHCP lease renewal. Consistent with arpwatch
   "flip flop" semantics (borrow the concept, not a count; mitre-arp-additional-detections.md §4a).
2. **Escalation to HIGH**: ≥3 distinct MACs for one IP within 60 seconds is highly unusual in
   legitimate traffic; it indicates active ARP table manipulation. HIGH/Likely is appropriate.
   The 3/60s values are wirerust engineering choices; no published tool (Snort, arpwatch, Zeek)
   uses numeric count/rate thresholds for ARP spoof escalation (mitre-arp-additional-detections.md
   §4, CRITICAL CORRECTION).
3. **One-shot HIGH guard**: `spoof_high_emitted` prevents repeated HIGH findings for the same
   IP in the same flap window. MEDIUM findings continue to be emitted on each additional rebind
   (forensic record of all events) until the window resets.
4. **MITRE tagging**: T0830 (ICS AiTM, v2.0, ATT&CK v19.1, current) and T1557.002 (Enterprise
   ARP Cache Poisoning, current). Dual-tagging is the wirerust convention for techniques that
   appear in both ICS and Enterprise matrices.
   **Tactic anchors (F5 correction — ADR-008 Decision 6 superseded for T0830):** T0830 maps to
   `MitreTactic::IcsCollection` (ICS Collection, TA0100) per MITRE ATT&CK for ICS v19.1
   (page-verified: https://attack.mitre.org/techniques/T0830/ — single tactic: Collection/TA0100).
   ADR-008 Decision 6 merge-by-name policy previously assigned T0830→LateralMovement; that
   assignment is WRONG and is superseded by the validated F5 research
   (f5-ics-technique-tactic-authoritative.md). T1557.002 maps to `MitreTactic::CredentialAccess`
   (TA0006, Enterprise) — unchanged. The F5 implementer must update the T0830 arm in
   `technique_info` from `LateralMovement` to `IcsCollection`. Normative source: MITRE ATT&CK
   for ICS v19.1; f5-ics-technique-tactic-authoritative.md (VALIDATED); BC-2.10.007 v1.9.
5. **Not a stateless detection**: D1 is stateful — it requires the binding table. Frames before
   the first binding for an IP do not emit a spoof finding (the first observation initializes
   the binding).
6. **By-construction panic-freedom at the four `.expect()` sites (PC-013 resolution)**: The
   four `.expect()` calls in `arp.rs` are provably unreachable under invariant-preserving
   execution and are the correct Rust idiom for this class of guarantee:
   - **Line 555**: `bindings.get_mut(&sender_ip)` on the GARP-conflict path — entry is
     guaranteed present because the `has_conflict` guard (which confirms `sender_ip` is in
     `bindings`) precedes this call unconditionally within the same `process_arp` invocation.
   - **Line 576**: `bindings.get_mut(&sender_ip)` on the GARP-conflict path after
     `apply_garp_conflict_escalation_impl` — same guard as line 555; the entry cannot be
     removed between the guard and this call in single-threaded code.
   - **Line 642**: `bindings.get_mut(&sender_ip)` on the non-GARP rebind path after emitting
     the D1 finding via `emit_d1_spoof_finding_impl` — entry was confirmed present by the
     rebind detection branch that reached this code.
   - **Line 827**: `entry.first_rebind_ts.expect(...)` in `emit_d1_spoof_finding_impl` Step 3
     — `first_rebind_ts` is set unconditionally in Step 2 of the same call before Step 3
     executes; the `None` branch is structurally unreachable.
   Each `.expect()` is a **loud, self-documenting tripwire**: if a future refactor breaks the
   invariant, the panic fires immediately with a descriptive message instead of silently
   corrupting state or dropping a finding. This is the established project convention —
   `decoder.rs` and `arp.rs` both use documented `.expect()` for proven invariants (e.g.,
   VP-024 Sub-C formally proves the binding-table last-write-wins substrate; the `.expect()`
   sites rely on that substrate).
   **Cross-reference:** VP-024 Sub-C (decoder panic-freedom, formally proven) establishes the
   substrate on which these sites depend. The five white-box regression tests in
   `mod bc_2_16_004_inv6` (see EC-011/EC-012) verify that the production paths emit correct
   finding counts and severities and serve as regression guards against any future
   drop-in-findings or severity regression that would indicate the invariant has been broken.
   **No production code change is mandated by this BC.** The `.expect()` calls remain as-is.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IP seen for the first time (no existing binding) | No spoof finding; binding initialized: `bindings[sender_ip] = BindingEntry { mac: sender_mac, rebind_count: 0, ... }` |
| EC-002 | IP seen again with the SAME MAC as existing binding | No spoof finding; binding remains unchanged (no rebind event) |
| EC-003 | First rebind: MAC changes from A to B | MEDIUM finding emitted; rebind_count → 1; binding updated to MAC B |
| EC-004 | Second rebind within 60s: MAC changes from B to C | MEDIUM finding emitted; rebind_count → 2; binding updated to MAC C |
| EC-005 | Third rebind within 60s: MAC changes from C to D (rebind_count reaches 3) | HIGH finding emitted (rebind_count >= SPOOF_REBIND_ESCALATION_DEFAULT=3); spoof_high_emitted → true; binding updated to MAC D |
| EC-006 | Fourth rebind within 60s: MAC changes again (spoof_high_emitted == true) | MEDIUM finding emitted (not HIGH — one-shot guard active); binding updated |
| EC-007 | Rebind after flap window expires (>60s since first_rebind_ts) | Window resets; rebind treated as first rebind; MEDIUM finding; rebind_count → 1 |
| EC-008 | `--arp-spoof-threshold 1` set: first rebind — Step 1: rebind_count→1; Step 2: first_rebind_ts=timestamp_secs (elapsed=0); Step 3: rebind_count=1 >= threshold=1 AND elapsed=0 <= 60 AND spoof_high_emitted=false → HIGH | HIGH Finding emitted on the very first rebind; elapsed=0 satisfies the window condition because first_rebind_ts is set in Step 2 before the condition is evaluated in Step 3 |
| EC-009 | `--arp-spoof-threshold 100` set: escalate only after 100 rebinds within 60s | MEDIUM on all rebinds until count reaches 100; HIGH only if 100 rebinds in 60s |
| EC-010 | sender_ip is all-zero (0.0.0.0) or broadcast (255.255.255.255) | Admissibility determined by BC-2.16.005 binding-table update rule (see BC-2.16.005 EC-006 and Invariant notes for zero/broadcast sender IP policy). If BC-2.16.005 filters these IPs at insertion, no binding is created and no spoof detection is triggered. If inserted, spoof detection applies normally. |
| EC-011 | By-construction characterization: GARP-conflict path — `has_conflict == true` therefore `bindings.get_mut(&sender_ip)` is **always** `Some(...)` (entry present by guard at arp.rs:555/576). The `.expect()` at these sites cannot fire in invariant-preserving execution. Regression tests: `test_BC_2_16_004_expect_site_no_panic_on_missing_entry` (verifies no-panic on production GARP-conflict path), `test_BC_2_16_004_garp_conflict_high_escalation` (verifies correct HIGH finding count and severity on GARP-conflict escalation path). Note for test-writer: `test_BC_2_16_004_expect_site_no_panic_on_missing_entry` — the name implies a "missing entry" scenario; in the corrected spec the test exercises the production path where entry IS present and confirms no panic and correct emission. A rename to `test_BC_2_16_004_garp_conflict_expect_site_invariant_holds` would be more precise, but the existing anchor name is kept for continuity; apply rename optionally. |
| EC-012 | By-construction characterization: non-GARP rebind path — `entry.first_rebind_ts` is **always** `Some(...)` when `emit_d1_spoof_finding_impl` Step 3 executes, because Step 2 sets it unconditionally before Step 3 in the same call frame (arp.rs:827). The `.expect()` cannot fire in invariant-preserving execution. Regression tests: `test_BC_2_16_004_emit_d1_first_rebind_ts_none` (verifies Step 2 sets `first_rebind_ts` and Step 3 proceeds correctly; name retained for continuity — the production path always has `Some`; a rename to `test_BC_2_16_004_emit_d1_first_rebind_ts_always_set` would be more precise), `test_BC_2_16_004_emit_d1_after_flap_window_reset` (verifies correct finding count/severity after a flap-window reset resets `first_rebind_ts`), `test_BC_2_16_004_non_garp_rebind_step4_reborrow` (verifies Step 4 mac-update at arp.rs:642 proceeds correctly and emits correct finding on non-GARP rebind path). All five tests in `mod bc_2_16_004_inv6` serve as regression guards: if a future refactor causes a drop-in-findings or severity regression on any of these paths, the tests catch it before the `.expect()` would need to fire. |

## Canonical Test Vectors

| Binding table state | Frame | Expected outcome |
|---|---|---|
| Empty | op=2, sender_ip=10.0.0.1, sender_mac=AA:AA:AA:AA:AA:AA, ts=0 | Binding initialized, no finding |
| {10.0.0.1 → AA:AA:AA:AA:AA:AA, rebind=0} | op=2, sender_ip=10.0.0.1, sender_mac=AA:AA:AA:AA:AA:AA, ts=1 | Same MAC — no finding; no state change |
| {10.0.0.1 → AA:AA:AA:AA:AA:AA, rebind=0} | op=2, sender_ip=10.0.0.1, sender_mac=BB:BB:BB:BB:BB:BB, ts=2 | MEDIUM Finding, T0830+T1557.002; rebind=1; binding → BB:BB |
| {10.0.0.1 → BB:BB, rebind=1, first_rebind_ts=2} | op=2, sender_ip=10.0.0.1, sender_mac=CC:CC:CC:CC:CC:CC, ts=5 | MEDIUM Finding; rebind=2; binding → CC:CC |
| {10.0.0.1 → CC:CC, rebind=2, first_rebind_ts=2} | op=2, sender_ip=10.0.0.1, sender_mac=DD:DD:DD:DD:DD:DD, ts=8 (within 60s) | HIGH Finding, T0830+T1557.002; spoof_high_emitted=true; binding → DD:DD |
| {10.0.0.1 → DD:DD, rebind=3, spoof_high_emitted=true} | op=2, sender_ip=10.0.0.1, sender_mac=EE:EE:EE:EE:EE:EE, ts=10 | MEDIUM Finding (high guard active); binding → EE:EE |

## Verification Properties

| VP-NNN | Property | Proof Method | Relationship |
|--------|----------|-------------|-------------|
| VP-024 Sub-C | Binding-table last-write-wins determinism: after processing any sequence of frames for an IP, `bindings[ip].mac` equals the MAC from the last frame | INDIRECT — BC-2.16.004 depends on the last-write-wins binding substrate that Sub-C proves (primary anchor is BC-2.16.005); Sub-C does NOT formally discharge BC-2.16.004's spoof-escalation postconditions (D1 severity logic is unit-tested per EC coverage). | substrate dependency |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — ARP cache poisoning / spoof detection (D1) is the core security detection of the ARP Security Analysis capability, directly mapping to T0830 (AiTM, ICS) and T1557.002 (ARP Cache Poisoning, Enterprise) |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::process_arp, C-23); ADR-008 Decisions 4–5 |
| Stories | STORY-114 |
| Feature | arp-security-analyzer |
| MITRE Techniques | T0830 (Adversary-in-the-Middle, ICS, ATT&CK v19.1 — current); T1557.002 (ARP Cache Poisoning, Enterprise, ATT&CK v19.1 — current) |

## Related BCs

- BC-2.16.005 — composes with (binding-table last-write-wins is the update mechanism this BC depends on)
- BC-2.16.006 — depends on (binding table cap; eviction affects which bindings are available for spoof detection)
- BC-2.16.012 — depends on (--arp-spoof-threshold overrides SPOOF_REBIND_ESCALATION_DEFAULT)
- BC-2.16.014 — composes with (GARP-that-conflicts: GARP frame triggers both D2 and D1)

## Architecture Anchors

- `src/analyzer/arp.rs` — `impl ArpAnalyzer { fn process_arp(...) }` — spoof detection logic
- `src/analyzer/arp.rs` — `struct BindingEntry { mac: [u8; 6], rebind_count: u32, first_rebind_ts: Option<u32>, spoof_high_emitted: bool, last_seen_ts: u32 }` (per ADR-008 Decision 4; `last_seen_ts` used for LRU eviction heuristic by BC-2.16.006)
- `src/analyzer/arp.rs` — `const SPOOF_REBIND_ESCALATION_DEFAULT: u32 = 3` (wirerust engineering default)
- `src/analyzer/arp.rs` — `const ARP_FLAP_WINDOW_SECS: u32 = 60` (wirerust engineering default)
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 4–5`
- `.factory/specs/architecture/arp-architecture-delta.md §3.2, §3.3 D1`

## Story Anchor

STORY-114

## VP Anchors

- VP-024 Sub-C — INDIRECT: BC-2.16.004 depends on the last-write-wins binding substrate that Sub-C proves (primary anchor BC-2.16.005); Sub-C does NOT formally discharge BC-2.16.004's spoof-escalation postconditions (those are unit-tested per the EC coverage).

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 4–5; arp-architecture-delta.md §3.2/§3.3; mitre-arp-additional-detections.md §4 (wirerust engineering choice defaults; fabricated industry thresholds explicitly REJECTED) |
| **Confidence** | high — D1 logic (rebind detection) is well-established; threshold values are intentionally our own engineering choices per §4 CRITICAL CORRECTION |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (ArpAnalyzer is a pure-core struct; process_arp mutates only the ArpAnalyzer instance fields) |
| **Deterministic** | yes — same sequence of frames always produces same findings |
| **Thread safety** | ArpAnalyzer is single-threaded (consistent with wirerust single-threaded pipeline) |
| **Overall classification** | stateful pure core — D1 spoof-escalation logic verified by unit tests (EC-001 through EC-012); VP-024 Sub-C substrate dependency is indirect only (primary anchor BC-2.16.005; BC-2.16.004 is NOT a VP-024 Sub-C formally-verified target); by-construction panic-freedom at four `.expect()` sites (Invariant 6) verified by regression tests in `mod bc_2_16_004_inv6` |
