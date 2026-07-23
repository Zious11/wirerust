---
artifact: L2-risk-register
document_type: domain-spec-shard
level: L2
version: "1.0"
producer: business-analyst
traces_to: ../domain/domain-spec.md
project: wirerust
status: backfill-complete
generated: 2026-07-09
inputs:
  - .factory/maintenance/risk-assumption-monitoring.md
  - .factory/tech-debt-register.md
input-hash: "865986f"
---

# wirerust Risk Register

This shard formalizes the 12 informal R-CAND items recorded across maintenance sweeps into
permanent L2 spec artifacts R-001..R-012. Each entry carries a `formalized_from` field
mapping it to its originating R-CAND identifier and preserves the exact status recorded in
`risk-assumption-monitoring.md` (maint-2026-07-09, the canonical current-status source).

This is a **backfill** document: it encodes ground truth only and introduces no new risks.
Every row traces to its R-CAND source.

**Status vocabulary:**
- `open` — unmitigated and unresolved; still requires action.
- `accepted` — formally accepted with written rationale; no further mitigation required unless trigger fires.
- `mitigated` — partial mitigation exists; item remains in registry for monitoring.
- `resolved` — fully closed; no further action required.

---

## Summary Table

| ID | formalized_from | Title | Likelihood | Impact | Status | Monitor |
|----|----------------|-------|-----------|--------|--------|---------|
| R-001 | R-CAND-001 | Unbounded weak-cipher evidence Vec | low | medium | open | yes |
| R-002 | R-CAND-002 | README multi-GB captures vs. eager Vec load | low | low | open | no |
| R-003 | R-CAND-003 | Single-platform CI (ubuntu-latest only) | medium | medium | open | yes |
| R-004 | R-CAND-004 | rayon unused direct dependency | — | — | resolved | no |
| R-005 | R-CAND-005 | Port-502 false-routing risk for non-Modbus protocols | low | low | accepted | no |
| R-006 | R-CAND-006 | DNP3 T0827 threshold misconfiguration risk | low | low | accepted | no |
| R-007 | R-CAND-007 | RUSTSEC-2026-0097 transitive rand 0.8.5 advisory | — | — | resolved | no |
| R-008 | R-CAND-008 | VLAN/QinQ/MACsec ARP offset detection limitation | low | low | accepted | no |
| R-009 | R-CAND-009 | Memory-DoS for DNP3/ENIP per-flow state | — | — | resolved | no |
| R-010 | R-CAND-010 | Unsafe split-borrow in src/analyzer/enip.rs | low | medium | open | yes |
| R-011 | R-CAND-011 | Port-44818 false-routing risk for non-ENIP protocols | low | low | accepted | no |
| R-012 | R-CAND-012 | rebind_count non-saturating arithmetic in arp.rs | — | — | resolved | no |

---

## R-001 — Unbounded Weak-Cipher Evidence Vec

**formalized_from:** R-CAND-001
**Title:** Unbounded weak-cipher evidence Vec (O-06 / NFR-RES-023)
**Source:** adversarial pass O-06; NFR-RES-023; GitHub #102
**Likelihood:** low — requires adversary-controlled PCAP with maximal cipher suite count; wirerust is an offline tool with no network-reachable attack surface
**Impact:** medium — worst-case heap ~270–500 KB per finding; approximately 9,216 cipher entries at MAX_RECORD_PAYLOAD / 2 bytes per entry
**Status:** open
**Priority:** P1 (active, 6+ months unaddressed as of maint-2026-07-09; three consecutive sweeps without action)
**Monitoring:** yes — revisit at every TLS-touching story; GitHub #102 CLOSED-COMPLETED 2026-06-08 but fix never shipped — closure appears premature, needs reconciliation (re-open or re-file)

**Description:** The ClientHello weak-cipher `Finding` in `src/analyzer/tls.rs` carries an
unbounded `Vec<String>` evidence field. No `MAX_WEAK_CIPHER_EVIDENCE` truncation cap exists.
The upper bound is approximately 9,216 cipher names (MAX_RECORD_PAYLOAD / 2 bytes per cipher);
worst-case heap footprint per finding is approximately 270–500 KB. NFR-RES-023 is OPEN.

**Mitigation (pending):** Add `MAX_WEAK_CIPHER_EVIDENCE = 64` cap with `"+N more"` annotation.
Target vehicle: STORY-150 or the next TLS-touching story.

**History:**
- Identified in adversarial pass (O-06); NFR-RES-023 filed.
- GitHub #102 CLOSED-COMPLETED 2026-06-08. Source-code grep confirms no `MAX_WEAK_CIPHER_EVIDENCE`
  cap exists in `src/` as of maint-2026-07-09 backfill validation — fix was never implemented.
  Closure appears premature or administrative. Risk remains OPEN; issue must be re-opened or
  re-filed before the next TLS-touching story. Flagged: maint-2026-07-09 backfill validation.
- PF-001 sweep (PR #384, 2026-07-08): converted counter `+=` sites to `saturating_add`; the
  evidence Vec collection structure was explicitly out of scope.
- maint-2026-07-09: still OPEN P1, three consecutive sweeps without action.

---

## R-002 — README Multi-GB Captures vs. Eager Vec Load

**formalized_from:** R-CAND-002
**Title:** README "multi-GB captures" claim contradicts eager Vec<RawPacket> load (NFR-VIO-001)
**Source:** maint-2026-07-06 risk-assumption-monitoring; NFR-VIO-001
**Likelihood:** low — most forensic PCAP captures in ICS environments are small; multi-GB captures are rare
**Impact:** low — OOM crash on oversized captures; no data corruption or security impact
**Status:** open
**Priority:** P3
**Monitoring:** no

**Description:** The README claims wirerust handles "multi-GB captures" but the ingestion layer
eagerly loads all packets into a `Vec<RawPacket>` before any analysis begins. NFR-VIO-001 is
OPEN-DEBT. The documentation claim is inaccurate for captures larger than available RAM.
No streaming refactor has been scoped (see ASM-004).

**Mitigation (pending):** Update README to accurately describe the eager-load constraint, or
implement streaming ingestion. Documentation correction is trivial; streaming refactor requires
significant effort.

**History:**
- NFR-VIO-001 opened during brownfield ingestion.
- Wave-72 doc PRs (#388, #390) did not address this README language gap.
- maint-2026-07-09: no change.

---

## R-003 — Single-Platform CI (ubuntu-latest only)

**formalized_from:** R-CAND-003
**Title:** Single-platform CI / ubuntu-latest only (NFR-VIO-010 / NFR-PORT-001)
**Source:** maint-2026-07-06 risk-assumption-monitoring; NFR-VIO-010; NFR-PORT-001
**Likelihood:** medium — cross-platform regressions can accumulate silently across releases
**Impact:** medium — latent build breakage or behavioral differences on macOS or Windows go
  undetected until a user reports them
**Status:** open
**Priority:** P2
**Monitoring:** yes

**Description:** The `ci.yml` test and clippy jobs run only on `ubuntu-latest`. The cross-compile
release matrix covers 4 platforms for build-time failures only, not test execution.
Platform-specific behavior differences (path separators, OS API behavior, endianness) are
undetected. NFR-PORT-001 is unmet.

**Mitigation (pending):** Add macOS and/or Windows test matrix jobs to `ci.yml`. Wave-72 CI
hardening (PR #391) improved supply-chain security but did not add cross-platform test execution.

**History:**
- Opened maint-2026-07-06; maint-2026-07-09 no change.

---

## R-004 — rayon Unused Direct Dependency

**formalized_from:** R-CAND-004
**Title:** rayon declared in Cargo.toml but never imported
**Source:** adversarial pass (LOW finding); maint-2026-06-22 dependency sweep
**Likelihood:** — (resolved)
**Impact:** — (resolved)
**Status:** resolved

**Description:** `rayon = "1"` was declared in `[dependencies]` but never imported in `src/`.
The dead dependency contributed unnecessary transitive crates to the build graph without
providing any functionality.

**Resolution:** Removed in PR #304 (e458ce2), maint-2026-06-22.

---

## R-005 — Port-502 False-Routing Risk for Non-Modbus Protocols

**formalized_from:** R-CAND-005
**Title:** Port-502 false-routing risk for non-Modbus binary protocols
**Source:** ADR-005; VP-022; maint-2026-07-06 risk-assumption-monitoring
**Likelihood:** low — port 502 is IANA-registered exclusively for Modbus; non-Modbus occupancy
  is uncommon in ICS environments
**Impact:** low — misrouted traffic produces parse errors and dropped findings; no silent data
  corruption or false findings
**Status:** accepted
**Monitoring:** no

**Description:** Any non-Modbus TCP traffic on port 502 will be routed to the Modbus analyzer.
The Modbus analyzer will emit parse errors for non-conformant data but will not produce false
security findings. VP-022 Kani proof verifies the three-point dispatch gate is sound. The risk
is inherent to port-only classification (see ASM-001).

**Acceptance rationale:** Port 502 is exclusively IANA-registered for Modbus. VP-022 is in
force. Accepted as a known design characteristic.

**History:**
- Accepted as known risk; no change through maint-2026-07-09.

---

## R-006 — DNP3 T0827 Threshold Misconfiguration Risk

**formalized_from:** R-CAND-006
**Title:** DNP3 T0827 direct-operate burst threshold misconfiguration risk (ADR-007)
**Source:** ADR-007; maint-2026-07-06 risk-assumption-monitoring; dnp3-research.md §5.1
**Likelihood:** low — misconfiguration requires a deliberate user action; the default is
  research-bounded within the 5–20 range per dnp3-research.md §5.1
**Impact:** low — false negatives (bursts below threshold undetected) or nuisance alerts
  (threshold too low); no false positives or security information disclosure
**Status:** accepted
**Monitoring:** no

**Description:** The DNP3 T0827 direct-operate burst threshold defaults to 10
(research-bounded: 5–20 range per dnp3-research.md §5.1). A misconfigured threshold could
cause false negatives or nuisance alerts. The default is an engineering choice, not an
empirically validated value.

**Acceptance rationale (PR #382, 2026-07-08):** README § Known Limitations "DNP3
direct-operate burst threshold" subsection documents the default as a research-bounded
engineering choice with CLI overridability noted. TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED.

**History:**
- maint-2026-07-06: MEDIUM (P2), AGING.
- PR #382 (624bae3), 2026-07-08: ACCEPTED-FORMALLY.
- maint-2026-07-09: ACCEPTED-FORMALLY confirmed, no further action required.

---

## R-007 — RUSTSEC-2026-0097 Transitive rand 0.8.5

**formalized_from:** R-CAND-007
**Title:** RUSTSEC-2026-0097 transitive rand 0.8.5 advisory
**Source:** maint-2026-06-22 dependency sweep (DEP-001); tech-debt-register DEP-001
**Likelihood:** — (resolved)
**Impact:** — (resolved)
**Status:** resolved

**Description:** rand 0.8.5 carried RUSTSEC-2026-0097. CI was failing with a temporary
`--ignore RUSTSEC-2026-0097` flag as a workaround. The advisory affected rand transitively
via tls-parser → phf.

**Resolution:** rand bumped to 0.8.6; CI `--ignore RUSTSEC-2026-0097` removed; `cargo audit`
clean. PR #304 (e458ce2), maint-2026-06-22.

---

## R-008 — VLAN/QinQ/MACsec ARP Offset Detection Limitation

**formalized_from:** R-CAND-008
**Title:** VLAN/QinQ/MACsec ARP offset detection limitation (CWE-693)
**Source:** STORY-117, E-17; maint-2026-07-06 risk-assumption-monitoring
**Likelihood:** low — VLAN/MACsec encapsulation is uncommon in most ICS PCAP captures
**Impact:** low — ARP findings missed for encapsulated frames; not a false positive risk; no
  security information disclosure
**Status:** accepted
**Monitoring:** no

**Description:** The ARP analyzer parses Ethernet frames at a fixed Ethernet II offset.
VLAN-tagged (802.1Q), QinQ double-tagged, or MACsec-encapsulated frames carry the ARP
payload at a different byte offset. Such frames will not produce ARP findings. CWE-693
is correctly recorded. This is a documented design-time decision (STORY-117, E-17).

**Acceptance rationale:** Accepted as a known design characteristic for the target ICS network
capture use case. README § Known Limitations entry for the MACsec ARP limitation is recommended
(REC-006, unactioned for three sweeps); file alongside the next documentation PR.

**History:**
- Accepted since STORY-117 (E-17).
- REC-006 (one sentence to README) has been unactioned for three consecutive sweeps
  (maint-2026-06-17, maint-2026-07-06, maint-2026-07-09).

---

## R-009 — Memory-DoS for DNP3/ENIP Per-Flow State

**formalized_from:** R-CAND-009
**Title:** Memory-DoS for DNP3/ENIP per-flow state (CWE-401 + CWE-770)
**Source:** maint-2026-07-06 risk-assumption-monitoring; GitHub #342
**Likelihood:** — (resolved)
**Impact:** — (resolved)
**Status:** resolved

**Description:** DNP3 and ENIP per-flow state maps previously had unbounded growth.
An adversary-controlled PCAP with many unique 5-tuples could exhaust process memory (CWE-770).
Resolved by introducing LRU eviction with configurable capacity bounds and observability
counters.

**Resolution:** Resolved in v0.11.3 (PR #362). GitHub #342 CLOSED.

---

## R-010 — Unsafe Split-Borrow in src/analyzer/enip.rs

**formalized_from:** R-CAND-010
**Title:** Unsafe split-borrow pattern in EnipAnalyzer::on_data (SEC-001)
**Source:** PR #334 security review; tech-debt-register SEC-001
**Likelihood:** low — the borrow is sound as written; risk materializes only if
  EnipFlowState struct is refactored without preserving the disjointness invariant
**Impact:** medium — if the disjointness invariant is violated by future refactoring, the
  resulting aliasing could produce undefined behavior in a production binary
**Status:** open
**Priority:** P2 (target: wave-85 / STORY-181)
**Monitoring:** yes

**Description:** The `EnipAnalyzer::on_data` function in `src/analyzer/enip.rs` contains an
unsafe self/flows split-borrow in the PDU dispatch loop (lines 992–999). The `for pdu in
pdu_queue` loop derives a `*mut EnipFlowState` raw pointer via `self.flows.get_mut(&flow_key)`,
then calls `self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, ...)`, creating simultaneous
aliasing between the `&mut self` borrow required by `process_pdu` and the raw pointer into
`self.flows[flow_key]`. Sound under the inline SAFETY comment invariant that `process_pdu`
never accesses `self.flows`, but fragile under refactoring. Note: the carry-buffer select at
lines 825–829 uses `std::mem::take` and is already safe — SEC-001 is exclusively the PDU
dispatch loop. This is tech-debt-register item SEC-001.

**Mitigation:** Take-remove-reinsert pattern: `self.flows.remove(&flow_key)` before the PDU
dispatch loop produces an owned local `EnipFlowState`, eliminating aliasing with `self.flows`;
`self.process_pdu(&mut flow, &pdu, ...)` called on the local variable; `self.flows.insert(flow_key,
flow)` re-inserts after the loop. `process_pdu`'s signature unchanged; behavior identical.
Absorbed into STORY-181 (wave-85, drafted D-494). Target: wave-85 delivery.

**History:**
- Surfaced in PR #334 security review as a pre-existing finding.
- DF-VALIDATION-001 confirmed pattern still present at commit c4eb1f4 (maint-2026-07-08).
- PF-001 (PR #384): converted counter increments to `saturating_add` but did not touch the
  PDU dispatch borrow pattern, which was out of scope for a counter discipline sweep.
- maint-2026-07-09: OPEN P2, no change.
- Wave-85 adversarial pass-1 remediation (2026-07-23): description corrected from stale
  carry-field split-borrow framing to the actual *mut EnipFlowState PDU-dispatch site;
  absorbed into STORY-181 (D-494).

---

## R-011 — Port-44818 False-Routing Risk for Non-EtherNet/IP Protocols

**formalized_from:** R-CAND-011
**Title:** Port-44818 false-routing risk for non-EtherNet/IP binary protocols (ADR-010)
**Source:** ADR-010; maint-2026-07-06 risk-assumption-monitoring
**Likelihood:** low — port 44818 is IANA-registered exclusively for EtherNet/IP; occupancy
  by other protocols is rare
**Impact:** low — misrouted traffic produces parse errors and dropped findings; no silent data
  corruption or false findings
**Status:** accepted
**Monitoring:** no

**Description:** Any non-EtherNet/IP TCP traffic on port 44818 will be routed to the ENIP
analyzer (see ASM-010). The ENIP analyzer will emit parse errors for non-conformant data.
IANA-exclusive port registration is the practical discriminator. No dedicated Kani gate proof
exists for port-44818 dispatch; REC-007 flags a `classify_enip_gate` harness (analogous to
VP-022/VP-023) for the next ENIP hardening cycle.

**Acceptance rationale:** IANA-exclusive port 44818 registration for EtherNet/IP. Accepted
analogously to R-005 (port-502 Modbus). Dedicated Kani proof is pending (REC-007).

**History:**
- Accepted; maint-2026-07-09 no change. REC-007 flagged for next ENIP hardening cycle.

---

## R-012 — rebind_count Non-Saturating Arithmetic in arp.rs

**formalized_from:** R-CAND-012
**Title:** rebind_count non-saturating arithmetic in ArpAnalyzer
**Source:** PR #366 security review; tech-debt-register REBIND-COUNT-SATURATING-001
**Likelihood:** — (resolved)
**Impact:** — (resolved)
**Status:** resolved

**Description:** `ArpEntry::rebind_count` (type u32) in `src/analyzer/arp.rs` used plain
`+= 1` instead of `saturating_add(1)`, inconsistent with the house saturating arithmetic
convention established by SEC-003/SEC-004/SEC-007. Overflow was realistically unreachable
in practice but represented a disciplinary inconsistency.

**Resolution:** Folded into PF-001 sweep PR #384 (c4eb1f4, 2026-07-08). All 14 arp.rs
counter sites (5 u64 standard counters + 1 u32 `rebind_count` + 8 finding-class u64 counters)
converted to `saturating_add`. NFR-REL-003 saturating arithmetic convention now uniformly
applied across all arp.rs counter sites.
