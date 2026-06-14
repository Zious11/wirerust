---
artifact: module-criticality
traces_to: .factory/specs/architecture/ARCH-INDEX.md
version: "1.3"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
frozen: true
freeze_condition: Phase 5 gate pass
frozen_at: "2026-06-02"
frozen_reason: "Phase 5 long passed (gate closed prior to Phase 6). Frozen by spec-steward at Phase-6 gate close per module-criticality lifecycle rule (MUTABLE through Phase 5)."
modified:
  - date: 2026-06-13
    actor: architect
    reason: "Corpus-wide consistency audit remediation (PR-2): C-23 ArpAnalyzer (SS-16, HIGH tier) added — same feature-cycle precedent as C-22 Modbus addition in v1.1. C-24 Dnp3Analyzer (SS-15, HIGH tier) also added — was missing from file despite shipping v0.6.0. Version bump 1.1→1.2."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-23 A-04: C-22 Modbus row harmonized with C-23/C-24 style — technique IDs now enumerated (T1692.001/T0836/T0814/T0806/T0835/T0831/T0888) per module-decomposition.md C-22 findings list. Version bump 1.2→1.3."
---

# Module Criticality Classification

> Lifecycle: MUTABLE through Phase 5. FROZEN as of 2026-06-02 (Phase-6 gate close).
> When a feature cycle adds or removes modules, update this file and
> record the change in the cycle manifest.

## Criticality Tiers

| Tier | Kill Rate Target | Meaning |
|------|-----------------|---------|
| CRITICAL | >= 95% | Forensic-correctness invariants; security properties; state-machine logic. Failures here produce incorrect forensic data or enable attacker evasion. |
| HIGH | >= 90% | Core protocol analysis logic. Failures here produce wrong findings or miss real threats. |
| MEDIUM | >= 80% | Supporting logic, output formatting, CLI parsing. Failures here degrade usability but not forensic correctness. |
| LOW | >= 70% | Test infrastructure, docs helpers, optional output formats. |

## CRITICAL Modules (>= 95% kill rate)

| Module | File | Rationale |
|--------|------|-----------|
| FlowKey + TcpFlow state | reassembly/flow.rs | INV-1 (canonical ordering) and INV-7 (finalize-once). Incorrect FlowKey merges unrelated flows, destroying evidence. State machine bugs drop data silently. |
| Segment buffer | reassembly/segment.rs | INV-3 (first-wins overlap) and INV-6 (MAX_FINDINGS). The overlap detection logic is the primary TCP-evasion-attack signal. A bug here allows attackers to inject alternate bytes without detection. |
| Content-first dispatch | dispatcher.rs | INV-2. Incorrect routing silently misdirects flows; TLS traffic analyzed as HTTP loses all TLS findings. |
| SNI classification | analyzer/tls.rs (extract_sni) | INV-5. The 4-way ordered match determines whether C0-control or non-ASCII SNI findings are emitted. A mis-classification emits no finding for a genuine attack indicator. |
| Raw-data contract enforcement | findings.rs + reporter/terminal.rs (escape_for_terminal) | INV-4 (ADR 0003). Terminal injection (CWE-117) if escape logic is missing or wrong. |

## HIGH Modules (>= 90% kill rate)

| Module | File | Rationale |
|--------|------|-----------|
| Reassembly engine hot path | reassembly/mod.rs | The core packet-processing loop. Bugs here affect every flow and every finding. Partially CRITICAL but the effectful shell (Drop/atomics) lowers the formal-verification scope. |
| TLS analyzer | analyzer/tls.rs (non-SNI logic) | JA3/JA3S correctness, weak-cipher detection, deprecated-protocol detection. Bugs produce wrong fingerprints or miss attack signals. |
| HTTP analyzer | analyzer/http.rs | Path-traversal, web-shell, and missing-host detection. Bugs produce false negatives on real attacks. |
| Packet decoder | decoder.rs | Link-type gate is a security boundary (prevents processing of unexpected frame types). Decode bugs corrupt ParsedPacket fields flowing into all downstream analysis. |
| MITRE catalog | mitre.rs | Incorrect technique ID routing produces findings that land in the wrong tactic bucket or in Uncategorized. Directly affects kill-chain analysis output. |
| Modbus TCP analyzer | analyzer/modbus.rs (C-22, SS-14) | ICS/OT threat detection. Bugs in MBAP parsing or function-code classification produce incorrect findings or miss attack signals. Pure core functions (parse_mbap_header, classify_fc) verified by VP-022. Finding-emission logic (write-burst, T0814 Diagnostics) is high-criticality. Findings: T1692.001/T0836/T0814/T0806/T0835/T0831/T0888. Target kill rate >= 90%. [NEW — feature cycle issue #7, F2 delta] |
| DNP3 TCP analyzer | analyzer/dnp3.rs (C-24, SS-15) | ICS/OT threat detection for DNP3 protocol (shipped v0.6.0). Bugs in carry-buffer parse or function-code classification produce incorrect findings or miss attack signals. Pure core functions verified by VP-023 (Kani). Findings: T1691.001/T0827/T0836/T0814. Target kill rate >= 90%. [NEW — feature cycle issue #8] |
| ARP security analyzer | analyzer/arp.rs (C-23, SS-16) | Link-layer security analysis. Binding table (HashMap, LRU-bounded MAX_ARP_BINDINGS) maintains IP→MAC state; bugs corrupt spoof detection. D1 spoof MEDIUM→HIGH escalation requires correct rebind counting. VP-024 Kani obligation covers binding-table invariant and parse safety. Findings: T0830 (ICS LateralMovement) + T1557.002 (Enterprise CredentialAccess). Target kill rate >= 90%. [NEW — feature cycle issue #9, F2 delta] |

## MEDIUM Modules (>= 80% kill rate)

| Module | File | Rationale |
|--------|------|-----------|
| PCAP reader | reader.rs | File I/O is effectful shell. The critical security property (pcapng rejection) is BC-2.01.004 -- test-sufficient. |
| JSON reporter | reporter/json.rs | Serde handles the heavy lifting. Deterministic key order (BTreeMap) is a correctness concern but not a forensic-correctness invariant. |
| Terminal reporter | reporter/terminal.rs | escape_for_terminal is CRITICAL (see above); the rest of the rendering logic (colorization, MITRE grouping) is MEDIUM. |
| Summary accumulator | summary.rs | Counts only; no security invariants. |
| Reassembly config | reassembly/config.rs | Data struct; validation (panics on invalid config per BC-2.04.001) is test-sufficient. |
| DNS analyzer | analyzer/dns.rs | Statistics-only; emits no findings; low-criticality. |

## LOW Modules (>= 70% kill rate)

| Module | File | Rationale |
|--------|------|-----------|
| CLI parser | cli.rs | clap-generated struct. CLI mutual exclusion (BC-2.12.007) is integration-tested. |
| CSV reporter | reporter/csv.rs | Optional output format; CSV-injection neutralization is unit-tested. |
| Main orchestrator | main.rs | Orchestration glue; finalize() guarantee is tested by Drop tripwire. |
| lib.rs | lib.rs | Re-export shim; no logic. |
| Absent CLI flags | cli.rs (removed flags) | BC-2.13.001..004: --threats/--beacon/--filter/--verbose do NOT exist; removed by PR #74; clap rejects them as unknown arguments. |
