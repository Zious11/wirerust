---
artifact: module-criticality
traces_to: .factory/specs/architecture/ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
frozen: false
freeze_condition: Phase 5 gate pass
---

# Module Criticality Classification

> Lifecycle: MUTABLE through Phase 5. Frozen after Phase 5 gate passes.
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
