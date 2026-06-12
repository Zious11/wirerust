---
document_type: prd
level: L3
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/domain/domain-spec.md
  - .factory/specs/domain/domain-debt.md
  - .factory/specs/domain/invariants/inv-01-core-invariants.md
  - .factory/specs/domain/capabilities/cap-01-pcap-ingestion.md
  - .factory/specs/domain/capabilities/cap-02-link-type-gating.md
  - .factory/specs/domain/capabilities/cap-03-packet-decoding.md
  - .factory/specs/domain/capabilities/cap-04-tcp-reassembly.md
  - .factory/specs/domain/capabilities/cap-05-content-first-dispatch.md
  - .factory/specs/domain/capabilities/cap-06-http-analysis.md
  - .factory/specs/domain/capabilities/cap-07-tls-analysis.md
  - .factory/specs/domain/capabilities/cap-08-dns-analysis.md
  - .factory/specs/domain/capabilities/cap-09-finding-emission.md
  - .factory/specs/domain/capabilities/cap-10-mitre-mapping.md
  - .factory/specs/domain/capabilities/cap-11-reporting-output.md
  - .factory/semport/wirerust/wirerust-pass-3-behavioral-contracts.md
  - .factory/semport/wirerust/wirerust-pass-3-deep-behavioral-contracts-r4.md
input-hash: "ff3462e"
traces_to: .factory/specs/domain/domain-spec.md
supplements:
  - prd-supplements/interface-definitions.md
  - prd-supplements/error-taxonomy.md
  - prd-supplements/test-vectors.md
  - prd-supplements/nfr-catalog.md
---

# Product Requirements Document: wirerust

> **Brownfield Mode:** This PRD is DESCRIPTIVE of the shipped system as of develop HEAD (post
> remediation-cycle PRs #69-#98, reconciled against 0082a0c). Every requirement is grounded in
> verified source evidence. Known gaps are recorded as debt (O-01..O-08), not silently omitted.
> Do NOT treat this document as aspirational -- it specifies what the system does today.

> **BC Index Model:** This PRD is an index document. Each Behavioral Contract (BC) lives in its
> own file under `behavioral-contracts/ss-NN/`. The tables below provide one-line summaries
> linking to individual BC files. Full contract details are NOT inlined here.
>
> **Version 1.1 delta:** Added Section 2.14 (SS-14 Modbus/ICS Analysis, 25 BCs, Feature #7,
> ADR-005). Updated Section 1.5 Out of Scope (T0855/T1692.001 and 5 other ICS techniques now emitted).
> Updated Section 6 KD-005 and KD-003 with Modbus-specific BC references. Added SS-14 rows to
> Section 7 RTM. Total BC count: 244 (was 219).
> **→ Current total after all deltas: 268 BCs.**
>
> **Version 1.2 delta (2026-06-09 — F2 Modbus revision):** Adopts three approved decisions from
> `f2-fix-directives.md` v2 (Decisions 11, 12, 13). **BREAKING CHANGE targeting v0.3.0:**
> Decision 13 changes `Finding.mitre_technique: Option<String>` to
> `Finding.mitre_techniques: Vec<String>` — JSON key renames to `"mitre_techniques"` (array),
> CSV column-6 header renames to `mitre_techniques` with semicolon-join encoding. Existing BCs
> revised: BC-2.09.001 v1.4, BC-2.09.006 v1.5, BC-2.10.005 v1.4, BC-2.10.007 v1.3,
> BC-2.10.008 v1.5, BC-2.11.013 v1.6, BC-2.11.015 v1.6, BC-2.11.017 v1.5, BC-2.11.020 v1.5,
> BC-2.11.024 v1.4. SS-14 revised BCs: BC-2.14.013/014/015/016/017/020/022/024 (all v2.0).
> ADR-006 registered. See `spec-changelog.md` §[1.2] for full entry.
>
> **Version 1.3 delta (2026-06-09 — F2 schema add-ons + release split):** Two research-backed
> schema add-ons (f2-multitag-schema.md) and release sequencing decision (f2-bundle-vs-split.md).
> ADD-ON 1: BC-2.11.001 v1.5 — JSON report envelope adds `mitre_domain: "ics-attack"` +
> `mitre_attack_version: "ics-attack-v15"` (F4 must pin). ADD-ON 2: BC-2.11.024 v1.5 — empty
> CSV cell clarification: EMPTY STRING not null; EC-015 added for consumer split guard.
> Release split: v0.3.0 = schema-only break (SS-09/10/11 + add-ons); v0.4.0 = Modbus additive
> (SS-14). See RELEASE SEQUENCING box in Section 2 and `spec-changelog.md` §[1.3].
>
> **Version 1.4 delta (2026-06-10 — MITRE ATT&CK for ICS v19 remap, issue #222):** 1:1 technique-ID
> remap driven by DF-VALIDATION-001-validated defect. T0855 "Unauthorized Command Message"
> (revoked v19) → T1692.001 "Unauthorized Message: Command Message" (ICS sub-technique, v19).
> T0856 "Spoof Reporting Message" (revoked v19) → T1692.002 "Unauthorized Message: Reporting
> Message" (ICS sub-technique, v19). Tactic unchanged: IcsImpairProcessControl for both.
> All T0855/T0856 references in live spec body updated. Audited via
> `mitre-ics-v19-catalog-audit.md` and `dnp3-mitre-verification.md`. Updated BCs: SS-14
> BC-2.14.006/007/008/011/013/014/015/016/017/018/019/020/022/024; SS-11
> BC-2.11.001/013/017/020/024; SS-10 BC-2.10.008; SS-09 BC-2.09.001/006.
> See `spec-changelog.md` §[v19-remap-2026-06-10].
>
> **Version 1.5 delta (2026-06-10 — Feature #8 DNP3/ICS analyzer, issue #8):** Added Section
> 2.15 (SS-15 DNP3/ICS Analysis, 22 BCs, ADR-007). Updated Section 2.10 O-04 domain debt
> note: SEEDED 21→23 (added T1691.001 + T0827), EMITTED 13→15. New ICS-unique MitreTactic
> variant `IcsImpact` (Display "Impact", ICS TA0105) added; `all_tactics_in_report_order`
> grows 16→17 elements. Updated BCs: BC-2.10.002/003/004/005/007/008 (v1.3–v1.7 per BC).
> Added SS-15 rows to Section 7 RTM. KD-005 and KD-007 extended with DNP3 BCs.
> Total BC count: 266 (was 244). See `spec-changelog.md` §[dnp3-f2-2026-06-10].
>
> **Version 1.6 delta (2026-06-10 — Feature #8 DNP3 research must-adds, issue #8 post-gate):**
> Added 2 research-validated must-add detections from `dnp3-f2-scope-threshold-validation.md`:
> BC-2.15.023 (DISABLE_UNSOLICITED/ENABLE_UNSOLICITED abuse → T0814) and BC-2.15.024
> (malformed/structural DNP3 anomaly from parse_errors threshold → T0814, Crain-Sistrunk
> coverage). Both map to existing T0814 — MITRE catalog counts unchanged (23/15/8). Applied
> threshold clarifications: BC-2.15.010 v1.2 (10/60s is flood guard; unauthorized-source
> fires at count=1; ~5/60s option for quiet profiles); BC-2.15.014 v1.4 (DIRECT_OPERATE_NR
> exclusion research-confirmed); BC-2.15.015 v1.4 (≥3 must be distinct impact events, not
> double-counted). SS-15 now 24 BCs. Total BC count: 268 (was 266).
> See `spec-changelog.md` §[dnp3-f2-mustadds-2026-06-10].

> **Version 1.7 delta (2026-06-10 — Adversarial finding C-2 fix, issue #8 blocking):**
> Fixed BC-2.15.024 (v1.1): replaced the erroneous windowed use of `parse_errors` with a
> separate windowed counter `malformed_in_window`. `parse_errors` is now correctly specified
> as a LIFETIME/monotonic counter (NEVER reset at window expiry; consumed by BC-2.15.020
> summarize()). `malformed_in_window` is the new windowed counter used for all threshold
> checks; resets to 0 at 300s window expiry. Extended BC-2.15.015 (v1.5) to reset the two
> new BC-2.15.024 windowed fields at window expiry (malformed_in_window, malformed_anomaly_emitted);
> Invariant 6 updated from "four fields" to six. PRD prose updated from "BC-2.15.001..022"
> to "BC-2.15.001..024", "22 BCs" to "24 BCs", and RTM entry for BC-2.15.024 corrected to
> name `malformed_in_window`. No new BCs; no MITRE catalog change; counts 23/15/8 unchanged.

> **Supplement Model:** Sections 3-5 reference extracted supplement files under
> `prd-supplements/`. These supplements are produced in a SEPARATE burst (Phase 1b).
> Entries in those sections are summary stubs until the supplement burst completes.


## 1. Product Overview

### 1.1 Problem Statement

Network security analysts and incident responders must triage captured network traffic for
indicators of compromise. Existing tools (Wireshark, Zeek, Suricata) require network
connectivity, complex configuration, or ongoing daemon processes. Analysts working on isolated
forensic workstations need a single-binary tool that produces structured, machine-readable
findings from pcap captures without any runtime infrastructure.

Additionally, existing tools often sanitize or alter attacker-controlled data during analysis,
destroying forensic fidelity. A raw HTTP URI containing C0 control bytes looks different after
being processed by a display-layer renderer -- yet the raw bytes are the evidence.

### 1.2 Solution Vision

wirerust is an offline, single-binary, single-pass forensic triage CLI that ingests classic-pcap
captures and emits structured findings about HTTP, TLS, and DNS traffic plus TCP stream-reassembly
anomalies. It has no network I/O, no async runtime, no unsafe blocks, and no process-to-process
state. The binary is the complete deployment unit.

The core design principle is "trustworthy forensic data preservation plus display-layer safety":
raw attacker-controlled bytes survive intact through every layer to JSON output; the terminal
renderer is the sole owner of escape logic. This ensures SIEM consumers see unaltered forensic
data while terminal operators are protected from terminal injection attacks.

Architecture: 5-layer synchronous pipeline (Entry -> Ingest -> Stream -> Domain -> Output), 24
Rust source files, 3,868 source LOC, 282 tests, single crate, Rust 2024 edition, MSRV 1.91.

### 1.3 Key Differentiators

| ID | Differentiator | Description |
|----|---------------|-------------|
| KD-001 | Offline single-binary deployment | No daemon, no network I/O, no runtime dependencies. Suitable for air-gapped forensic workstations. |
| KD-002 | Forensic-fidelity raw-data contract | Attacker-controlled bytes (URIs, SNI hostnames, payloads) pass through unmodified to JSON output; escape runs only at terminal display (ADR 0003). |
| KD-003 | Content-first protocol identification | Protocol dispatch inspects TCP payload bytes before port numbers, defeating port-obfuscation attacks (ADR 0001). |
| KD-004 | First-wins TCP overlap forensics | Conflicting retransmissions are detected and emitted as findings; attackers cannot silently insert alternate bytes (INV-3). |
| KD-005 | MITRE ATT&CK tactic-grouped output | Findings carry structured MITRE technique IDs; terminal output can group by tactic for kill-chain analysis. |
| KD-006 | SNI anomaly detection with 4-way classification | TLS SNI hostnames are classified into four categories (clean ASCII, C0/DEL-containing, non-ASCII UTF-8, non-UTF-8 bytes) each triggering distinct findings. |
| KD-007 | Bounded-resource design | MAX_FINDINGS cap (10,000), per-direction buffer caps (65 KB), configurable reassembly thresholds with CLI override, no unbounded accumulation paths (except O-06). |

### 1.4 Target Users

| Persona | Description | Volume | Pain Level |
|---------|-------------|--------|------------|
| Forensic analyst | Processes pcap captures from incident response collections on isolated workstations | Low volume, high frequency during IR | High -- needs structured output fast, cannot install complex tooling |
| SOC operator | Bulk-processes pcap archives for indicator extraction, feeds output into SIEM | Medium volume, batch mode | High -- JSON output must be machine-parseable, not display-oriented |
| Malware researcher | Analyzes C2 traffic patterns, TLS fingerprinting, HTTP evasion techniques | Low volume, deep analysis | Medium -- needs JA3/JA3S and SNI anomaly details |
| Security toolchain integrator | Uses wirerust as a preprocessing stage in a pipeline (jq, grep, awk on JSON output) | High volume, automated | Medium -- needs deterministic JSON key order, stable exit codes |

### 1.5 Out of Scope

> Machine-consumed constraint list. The adversary and consistency-validator check that no story
> AC implements any feature listed here. Be explicit and unambiguous.

- pcapng format support (wirerust reads classic pcap ONLY; pcapng files are rejected at the reader boundary)
- Live network capture / sniffing (no network I/O of any kind; offline pcap files only)
- HTTP/2 or HTTP/3 analysis (HTTP/1.x only; H2 frames will be parsed as unknown bytes)
- DNS-based detection findings (DNS is statistics-only: query/response counts only; no NXDOMAIN flood, no tunneling detection)
- TLS decryption or certificate validation (SNI and cipher fingerprinting only; no key material involved)
- BPF filtering (--filter flag removed by PR #74; clap rejects --filter as unknown argument; out of scope for current release)
- C2 beacon detection (--beacon flag removed by PR #74; clap rejects --beacon as unknown argument; no beacon analyzer exists)
- --threats flag behavior (flag removed by PR #74; clap rejects --threats as unknown argument; no corresponding analyzer)
- --verbose flag (removed by PR #74 alongside --filter/--beacon/--threats; clap rejects --verbose as unknown argument; no verbosity levels defined)
- --services flag on summary subcommand (removed by PR #74; clap rejects --services as unknown argument; per-service breakdown is out of scope for current release)
- Parallel file processing (rayon = "1" is a declared production dependency but is entirely unused in src/; single-threaded only)
- Streaming / lazy-read pcap processing (entire file loaded into RAM before processing)
- Per-packet timestamp in findings (Finding.timestamp is always None; O-01)
- Empirically-calibrated anomaly thresholds (defaults are research-documented but not validated against labelled traffic; O-03)
- MITRE techniques T1040, T1071, T1071.001, T1071.004, T1573, T1692.002, T0885 (catalogued but never emitted; O-04; note: T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 are now emitted by the Modbus/ICS analyzer — see Section 2.14; T0846 is seeded in the catalog but NOT emitted — see ADR-006 Decision 12; T1692.002 replaces revoked T0856 per ATT&CK-ICS v19 remap)


## 2. Behavioral Contracts Index

> BCs are organized by L2 domain capability (CAP-NN). BC numbering: BC-2.NN.NNN where
> 2 = PRD section, NN = capability number, NNN = sequential within capability.
> Files live in `behavioral-contracts/ss-NN/BC-2.NN.NNN.md`.

> **BREAKING OUTPUT SCHEMA CHANGE — v0.3.0 (ADR-006):**
> `Finding.mitre_technique: Option<String>` is renamed and retyped to
> `Finding.mitre_techniques: Vec<String>`. This affects ALL analyzers and ALL reporters:
> - **JSON:** key `"mitre_technique"` (scalar string) → `"mitre_techniques"` (array);
>   field absent when empty (same policy as prior `None` via `skip_serializing_if`).
> - **JSON envelope:** two new top-level fields added: `mitre_domain: "ics-attack"` and
>   `mitre_attack_version: "ics-attack-v15"` (placeholder; F4 must pin). See BC-2.11.001 v1.5.
> - **CSV:** column-6 header renamed `mitre_technique` → `mitre_techniques`; multiple
>   values semicolon-joined (`"T1692.001;T0836"`); single value unchanged; empty cell is `""`
>   (not `"null"`, not `"[]"`); consumers splitting on `;` must guard the empty-cell case
>   (see BC-2.11.024 v1.5 EC-015). CSV carries no envelope fields.
> - **Rust type:** `Option<String>` → `Vec<String>`; all emission sites updated.
>   All downstream JSON consumers, CSV pipelines, and Rust code using `Finding` must update.
> See ADR-006, BC-2.09.001, BC-2.09.006, BC-2.11.001, BC-2.11.020, BC-2.11.024.
> Affected stories: STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080.

> **RELEASE SEQUENCING — Feature #7 split: v0.3.0 (schema) + v0.4.0 (Modbus) (f2-bundle-vs-split.md B2):**
> Feature #7 is split into two releases per research recommendation (B2 — Trivy/Zeek pattern):
>
> **v0.3.0 — "Multi-technique findings" (schema migration only; breaking):**
> All existing analyzers (HTTP/TLS/DNS/lifecycle) migrated to `mitre_techniques: Vec<String>`.
> JSON envelope fields added. No new protocol analyzer. This is a **semver-honest breaking
> release**: one signal, one break, focused migration note.
> BCs in scope for v0.3.0:
> - SS-09 (findings.rs): BC-2.09.001, BC-2.09.006
> - SS-10 (mitre.rs): BC-2.10.005, BC-2.10.007, BC-2.10.008
> - SS-11 (reporters): BC-2.11.013, BC-2.11.015, BC-2.11.017, BC-2.11.020, BC-2.11.024
>   (+ BC-2.11.001 for envelope ADD-ON 1)
> - Existing stories: STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080
>
> **v0.4.0 — "Modbus TCP analyzer" (purely additive; no schema break):**
> Adds the Modbus TCP protocol analyzer on top of the stabilized multi-tag contract.
> Multi-tag type ships in v0.3.0; Modbus emits multi-tag findings natively but the *type
> itself* is already stable. No `**Breaking:**` entry in v0.4.0 changelog.
> BCs in scope for v0.4.0: all SS-14 BCs (BC-2.14.001 through BC-2.14.025).
> T0888/dual-window/co-emission detection are v0.4.0 (Modbus analyzer emits these;
> the multi-tag Vec<String> type that enables them ships in v0.3.0).
>
> Rationale: f2-bundle-vs-split.md establishes that multi-tag is independent of Modbus
> (shared `Finding` struct in `findings.rs`), bundling couples a cross-cutting refactor
> with a new stateful analyzer (worst pairing for bisection/rollback), and the Trivy
> two-phase flag model is the closest OSS precedent. Compat softening: `--compat-mitre-scalar`
> flag (default on in v0.3.x) emits the old scalar `mitre_technique` key alongside the new
> array for a deprecation window, following the Zeek dual-field approach.

### 2.1 PCAP File Ingestion (CAP-01)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.01.001 | Accept supported link types and reject unsupported at file open | P0 | BC-RDR-001 |
| BC-2.01.002 | Read all packets from pcap as Vec<RawPacket> preserving timestamps | P0 | BC-RDR-002 |
| BC-2.01.003 | Accept pcap with zero packets (header-only) without error | P1 | BC-RDR-003 |
| BC-2.01.004 | Reject pcapng-format input at reader level | P0 | BC-RDR-004 |
| BC-2.01.005 | Convert pcap record timestamp to (timestamp_secs: u32, timestamp_usecs: u32) | P1 | BC-RDR-005 |
| BC-2.01.006 | Surface pcap header parse errors with anyhow context | P1 | BC-RDR-006 |
| BC-2.01.007 | Surface per-packet read errors with anyhow context | P1 | BC-RDR-007 |
| BC-2.01.008 | from_file opens via BufReader and delegates to from_pcap_reader | P2 | BC-RDR-008 |

> Full contracts: `behavioral-contracts/ss-01/BC-2.01.001.md` through `BC-2.01.008.md`

### 2.2 Link-Type Gating (CAP-02)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.02.001 | Decode Ethernet-framed IPv4 TCP packet to ParsedPacket | P0 | BC-DEC-001 |
| BC-2.02.002 | Decode Ethernet-framed IPv4 UDP packet with DNS hint | P0 | BC-DEC-002 |
| BC-2.02.003 | Decode RAW link-layer IPv4 TCP packet via from_ip | P0 | BC-DEC-003 |
| BC-2.02.004 | DataLink::IPV4 decodes identically to DataLink::RAW | P1 | BC-DEC-004 |
| BC-2.02.005 | Decode RAW IPv6 TCP packet surfacing IPv6 addresses | P0 | BC-DEC-005 |
| BC-2.02.006 | Decode Linux SLL (cooked) TCP packets | P0 | BC-DEC-006 |
| BC-2.02.007 | Reject malformed input bytes with anyhow error (no panic) | P0 | BC-DEC-007 |
| BC-2.02.008 | Reject unsupported link types in decode_packet | P1 | BC-DEC-008 |
| BC-2.02.009 | Surface No IP layer found error | P1 | BC-DEC-009 |
| BC-2.02.010 | Classify ICMP as Protocol::Icmp with TransportInfo::None | P1 | BC-DEC-010 |
| BC-2.02.011 | Classify other IP protocols as Protocol::Other(byte) | P1 | BC-DEC-011 |
| BC-2.02.012 | app_protocol_hint returns service strings from port number | P1 | BC-DEC-012 |
| BC-2.02.013 | app_protocol_hint returns None when TransportInfo is None | P2 | BC-DEC-013 |
| BC-2.02.014 | packet_len is set to total frame length not just payload | P1 | BC-DEC-014 |
| BC-2.02.015 | Extract TCP control flags and sequence number into TransportInfo::Tcp | P0 | BC-DEC-015 |

> Full contracts: `behavioral-contracts/ss-02/BC-2.02.001.md` through `BC-2.02.015.md`

### 2.3 Packet Decoding (CAP-03)

> CAP-03 BCs are co-located with CAP-02 in ss-02 because the decoder is the single component
> (C-5) implementing both capabilities. The BC-DEC-NNN ingestion IDs map to BC-2.02.NNN above.
> No separate ss-03 directory is required for this capability.

### 2.4 TCP Stream Reassembly (CAP-04)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.04.001 | TcpReassembler::new panics on invalid config (defensive assert) | P1 | BC-RAS-001 |
| BC-2.04.002 | Non-TCP packets are skipped and packets_skipped_non_tcp increments | P1 | BC-RAS-002 |
| BC-2.04.003 | Canonical FlowKey ordering ensures A->B and B->A produce identical key | P0 | BC-RAS-003 |
| BC-2.04.004 | First SYN sets client ISN and initiator | P0 | BC-RAS-004 |
| BC-2.04.005 | SYN+ACK marks server as responder and transitions state to Established | P0 | BC-RAS-005 |
| BC-2.04.006 | Bidirectional data delivered with correct Direction tag | P0 | BC-RAS-006 |
| BC-2.04.007 | In-order data flushes contiguously to handler in segment order | P0 | BC-RAS-007 |
| BC-2.04.008 | Out-of-order segments buffer until gap filled then flush contiguously | P0 | BC-RAS-008 |
| BC-2.04.009 | Mid-stream join infers ISN from first-data seq-1 and marks flow partial | P0 | BC-RAS-009 |
| BC-2.04.010 | RST closes flow immediately with CloseReason::Rst and zeroes total_memory | P0 | BC-RAS-010 |
| BC-2.04.011 | Both FINs close flow with CloseReason::Fin and remove from table | P0 | BC-RAS-011 |
| BC-2.04.012 | finalize flushes all remaining flows with Timeout and is idempotent | P0 | BC-RAS-012 |
| BC-2.04.013 | expire_idle_by_timeout / expire_flows closes flows idle past flow_timeout_secs | P1 | BC-RAS-013 |
| BC-2.04.014 | total_memory tracks buffered bytes and decrements on flush and close | P1 | BC-RAS-014 |
| BC-2.04.015 | Flow eviction on max_flows hit uses LRU non-established-first policy | P1 | BC-RAS-015 |
| BC-2.04.016 | Memory pressure eviction when total_memory exceeds memcap | P1 | BC-RAS-016 |
| BC-2.04.017 | Eviction sort: non-established first, then oldest-last-seen within band | P1 | BC-RAS-017 |
| BC-2.04.018 | Conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | P0 | BC-RAS-018 |
| BC-2.04.019 | Excessive overlaps (>threshold) emit one-shot T1036 finding | P0 | BC-RAS-019 |
| BC-2.04.020 | Excessive small segments (>threshold) emit one-shot finding | P1 | BC-RAS-020 |
| BC-2.04.021 | Excessive out-of-window segments (>threshold) emit one-shot Low finding | P1 | BC-RAS-021 |
| BC-2.04.022 | Per-direction alert fires at most once per flow (sticky latch) | P0 | BC-RAS-022 |
| BC-2.04.023 | Truncated segment emits Anomaly/Inconclusive/Low finding (no MITRE) | P1 | BC-RAS-023 |
| BC-2.04.024 | Total findings capped at MAX_FINDINGS=10000; excess silently dropped | P0 | BC-RAS-024 |
| BC-2.04.025 | finalize emits segment-limit summary finding when segments dropped (with pluralization) | P0 | BC-RAS-025 |
| BC-2.04.026 | finalize does NOT emit segment-limit finding when counter is zero | P0 | BC-RAS-026 |
| BC-2.04.027 | segments_depth_exceeded tracks fully-rejected segments after depth hit | P1 | BC-RAS-027 |
| BC-2.04.028 | summarize returns AnalysisSummary with reassembly stats detail map | P1 | BC-RAS-028 |
| BC-2.04.029 | close_flow for missing key logs one-shot process-wide warning | P2 | BC-RAS-029 |
| BC-2.04.030 | bytes_reassembled equals total bytes delivered to handler at end | P1 | BC-RAS-030 |
| BC-2.04.031 | ISN set on first SYN; inferred as seq-1 on data-without-SYN | P0 | BC-RAS-031 |
| BC-2.04.032 | insert_segment with no ISN returns IsnMissing and inserts nothing | P0 | BC-RAS-032 |
| BC-2.04.033 | Single segment insertion returns Inserted and stores under offset key | P0 | BC-RAS-033 |
| BC-2.04.034 | flush_contiguous consumes segments from base_offset in order | P0 | BC-RAS-034 |
| BC-2.04.035 | Identical retransmission returns Duplicate and does not double-count bytes | P0 | BC-RAS-035 |
| BC-2.04.036 | First-wins overlap: gap bytes added, existing bytes preserved | P0 | BC-RAS-036 |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap, preserves original | P0 | BC-RAS-037 |
| BC-2.04.038 | Multi-segment full coverage returns Duplicate or ConflictingOverlap as appropriate | P0 | BC-RAS-038 |
| BC-2.04.039 | TCP sequence wraparound across 32-bit boundary reassembles correctly | P0 | BC-RAS-039 |
| BC-2.04.040 | Small-segment counter increments per direction for segments under threshold | P1 | BC-RAS-040 |
| BC-2.04.041 | Depth truncation: segment crossing max_depth is truncated to remaining capacity | P0 | BC-RAS-041 |
| BC-2.04.042 | Segment beyond max_receive_window returns OutOfWindow; boundary segment accepted | P1 | BC-RAS-042 |
| BC-2.04.043 | Adjacent segments at exact boundary do not count as overlap | P0 | BC-RAS-043 |
| BC-2.04.044 | Segments map full: non-overlapping insert returns SegmentLimitReached | P0 | BC-RAS-044 |
| BC-2.04.045 | Segments map full: overlapping insert needing gap insertion returns SegmentLimitReached | P0 | BC-RAS-045 |
| BC-2.04.046 | Segments map fills mid-loop: partial insertion with later gaps dropped | P0 | BC-RAS-046 |
| BC-2.04.047 | buffered_bytes mirrors segment size sum after all insert/overlap/flush ops | P0 | BC-RAS-047 |
| BC-2.04.048 | ISN_MISSING_WARNED atomic prevents repeated eprintln on missing-ISN errors | P2 | BC-RAS-048 |
| BC-2.04.049 | FlowKey::Display formats as lower_ip:lower_port -> upper_ip:upper_port with U+2192 | P1 | BC-RAS-049 |
| BC-2.04.050 | Flow state machine: New->SynSent->Established->Closing->Closed transitions | P0 | BC-RAS-050 |
| BC-2.04.051 | RST transitions state to Closed from any prior state | P0 | BC-RAS-051 |
| BC-2.04.052 | on_data_without_syn transitions New->Established and sets partial=true | P0 | BC-RAS-052 |
| BC-2.04.053 | TcpFlow::direction returns ClientToServer when src matches initiator | P0 | BC-RAS-053 |
| BC-2.04.054 | finalize unconditionally bypasses MAX_FINDINGS cap for segment-limit finding | P0 | BC-RAS-054 |

> Full contracts: `behavioral-contracts/ss-04/BC-2.04.001.md` through `BC-2.04.055.md`
> (BC-2.04.055 added Feature Mode F2 issue #100: StreamHandler::on_data timestamp parameter)

### 2.5 Content-First Protocol Dispatch (CAP-05)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.05.001 | TLS content signature routes flow to TLS regardless of port | P0 | BC-DSP-001 |
| BC-2.05.002 | HTTP method prefix routes flow to HTTP | P0 | BC-DSP-002 |
| BC-2.05.003 | Port fallback: 443/8443->TLS, 80/8080->HTTP when content insufficient | P0 | BC-DSP-003 |
| BC-2.05.004 | Unknown content and unknown port returns DispatchTarget::None | P1 | BC-DSP-004 |
| BC-2.05.005 | Classification cached per FlowKey after first non-None result | P0 | BC-DSP-005 |
| BC-2.05.006 | DispatchTarget::None NOT cached until retry cap (default 8); reclassification retried per on_data until cap, then None cached permanently | P0 | BC-DSP-006 |
| BC-2.05.007 | unclassified_flows increments only at on_flow_close for never-classified flows | P1 | BC-DSP-007 |
| BC-2.05.008 | No analyzer configured: dispatcher early-returns from on_data | P1 | BC-DSP-008 |
| BC-2.05.009 | on_flow_close removes route entry and forwards close to analyzer | P0 | BC-DSP-009 |

> Full contracts: `behavioral-contracts/ss-05/BC-2.05.001.md` through `BC-2.05.009.md`

### 2.6 HTTP Traffic Analysis (CAP-06)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.06.001 | Parse complete HTTP/1.1 request extracting method, URI, version, Host, User-Agent | P0 | BC-HTTP-001 |
| BC-2.06.002 | Parse pipelined requests with independent per-request method/uri counting | P0 | BC-HTTP-002 |
| BC-2.06.003 | Partial requests buffered until complete; not counted until full | P0 | BC-HTTP-003 |
| BC-2.06.004 | Parse HTTP/1.1 responses with status code counting and transaction advance | P0 | BC-HTTP-004 |
| BC-2.06.005 | Path traversal in URI emits Reconnaissance/Likely/High finding mapped to T1083 | P0 | BC-HTTP-005 |
| BC-2.06.006 | Web-shell URI patterns emit Execution/Likely/Medium finding mapped to T1505.003 | P0 | BC-HTTP-006 |
| BC-2.06.007 | Admin panel paths emit Reconnaissance/Inconclusive/Low finding mapped to T1046 | P1 | BC-HTTP-007 |
| BC-2.06.008 | Unusual HTTP methods emit Reconnaissance/Inconclusive/Medium finding (no MITRE) | P1 | BC-HTTP-008 |
| BC-2.06.009 | HTTP/1.1 request without Host header emits Anomaly/Inconclusive/Medium finding | P0 | BC-HTTP-009 |
| BC-2.06.010 | URI longer than 2048 chars emits Execution/Likely/Medium finding with char count | P1 | BC-HTTP-010 |
| BC-2.06.011 | Empty (present-but-blank) User-Agent emits Anomaly/Inconclusive/Low finding; absent UA does NOT | P1 | BC-HTTP-011 |
| BC-2.06.012 | Well-formed HTTP request produces zero findings | P0 | BC-HTTP-012 |
| BC-2.06.013 | Non-HTTP bytes increment parse_errors but do not emit Token-error findings | P0 | BC-HTTP-013 |
| BC-2.06.014 | Too many headers (>96) emits Anomaly/Inconclusive/Medium finding mapped to T1499.002 | P0 | BC-HTTP-014 |
| BC-2.06.015 | After 3 consecutive parse errors a direction is poisoned; subsequent bytes skipped | P0 | BC-HTTP-015 |
| BC-2.06.016 | Single parse error does not poison; next valid request parses normally | P0 | BC-HTTP-016 |
| BC-2.06.017 | Poisoning is per-direction: poisoned request does not affect response | P0 | BC-HTTP-017 |
| BC-2.06.018 | non_http_flows counts a flow once even if both directions get poisoned | P1 | BC-HTTP-018 |
| BC-2.06.019 | on_flow_close removes per-flow state; reopening same FlowKey starts fresh | P0 | BC-HTTP-019 |
| BC-2.06.020 | HTTP body bytes after header completion do not inflate parse_errors | P1 | BC-HTTP-020 |
| BC-2.06.021 | Cross-flow isolation: parse errors and poisoning in one flow do not leak | P0 | BC-HTTP-021 |
| BC-2.06.022 | Per-direction header buffer capped at MAX_HEADER_BUF (65536 bytes) | P1 | BC-HTTP-022 |
| BC-2.06.023 | summarize emits AnalysisSummary with HTTP stats detail map | P1 | BC-HTTP-023 |
| BC-2.06.024 | Per-map cardinality cap: new keys dropped past MAX_MAP_ENTRIES (50000) | P2 | BC-HTTP-024 |
| BC-2.06.025 | uris list capped at MAX_URIS=10000; further URIs silently dropped | P2 | BC-HTTP-025 |
| BC-2.06.026 | Header value extraction uses from_utf8_lossy.trim(); raw bytes preserved per ADR 0003 | P0 | BC-HTTP-026 |

> Full contracts: `behavioral-contracts/ss-06/BC-2.06.001.md` through `BC-2.06.026.md`

### 2.7 TLS Traffic Analysis (CAP-07)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.07.001 | Parse complete TLS ClientHello: version, ciphers, extensions, SNI, JA3 | P0 | BC-TLS-001 |
| BC-2.07.002 | Parse complete TLS ServerHello: JA3S fingerprint computed | P0 | BC-TLS-002 |
| BC-2.07.003 | After both hellos seen, subsequent records silently skipped | P0 | BC-TLS-003 |
| BC-2.07.004 | TLS record payload > MAX_RECORD_PAYLOAD (18432) increments parse_errors and truncated_records | P0 | BC-TLS-004 |
| BC-2.07.005 | Per-direction buffer capped at MAX_BUF=65536 bytes | P1 | BC-TLS-005 |
| BC-2.07.006 | JA3 computation filters GREASE values per RFC 8701 | P0 | BC-TLS-006 |
| BC-2.07.007 | JA3 string format: version,ciphers,extensions,curves,pointfmts hyphen-joined; MD5 hex | P0 | BC-TLS-007 |
| BC-2.07.008 | JA3S string format: version,cipher,extensions hyphen-joined; MD5 hex | P0 | BC-TLS-008 |
| BC-2.07.009 | Weak client cipher (NULL/ANON/EXPORT in ClientHello) emits Anomaly/Likely/High finding | P0 | BC-TLS-009 |
| BC-2.07.010 | Weak server cipher selected (NULL/ANON/EXPORT/RC4) emits Anomaly/Likely/Medium finding | P0 | BC-TLS-010 |
| BC-2.07.011 | Deprecated client protocol (<=SSLv3) emits Anomaly/Likely/High finding citing RFC 7568 | P0 | BC-TLS-011 |
| BC-2.07.012 | Deprecated server protocol (<=SSLv3) emits Anomaly/Likely/High finding | P0 | BC-TLS-012 |
| BC-2.07.013 | Clean ASCII SNI without C0/DEL bytes produces no SNI-related finding | P0 | BC-TLS-013 |
| BC-2.07.014 | SNI containing C0/DEL byte emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-014 |
| BC-2.07.015 | Multiple control bytes in one SNI produce exactly ONE finding | P0 | BC-TLS-015 |
| BC-2.07.016 | C0 boundary: 0x1F trips the finding; 0x20 (space) does not | P0 | BC-TLS-016 |
| BC-2.07.017 | Non-ASCII but valid UTF-8 SNI emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-017 |
| BC-2.07.018 | Punycode A-label (xn--...) is pure ASCII and emits no SNI finding | P1 | BC-TLS-018 |
| BC-2.07.019 | Non-UTF-8 SNI bytes emit Anomaly/Inconclusive/Low finding mapped to T1027; count key tagged | P0 | BC-TLS-019 |
| BC-2.07.020 | Non-UTF-8 SNI summary preserves raw bytes (no Debug-format escaping per ADR 0003) | P0 | BC-TLS-020 |
| BC-2.07.021 | Non-ASCII UTF-8 SNI summary preserves raw bytes per ADR 0003 | P0 | BC-TLS-021 |
| BC-2.07.022 | SNI extension with empty ServerNameList: no count, no finding, handshake still counted | P1 | BC-TLS-022 |
| BC-2.07.023 | SNI with empty hostname bytes counts under "" key; no non-UTF-8 finding | P2 | BC-TLS-023 |
| BC-2.07.024 | Only FIRST ServerName entry in multi-name SNI list is processed | P1 | BC-TLS-024 |
| BC-2.07.025 | Non-zero NameType entries are passed through as hostnames (current tls-parser behavior) | P2 | BC-TLS-025 |
| BC-2.07.026 | Trailing bytes in ServerNameList tolerated; first hostname still extracted | P2 | BC-TLS-026 |
| BC-2.07.027 | Large SNI (16 KB) under MAX_RECORD_PAYLOAD parses successfully | P1 | BC-TLS-027 |
| BC-2.07.028 | sni_counts cap at MAX_MAP_ENTRIES silently drops keys; SNI anomaly finding still fires | P0 | BC-TLS-028 |
| BC-2.07.029 | Bad TLS record body increments parse_errors and does not panic | P0 | BC-TLS-029 |
| BC-2.07.030 | Normal handshake (strong cipher) produces zero findings | P0 | BC-TLS-030 |
| BC-2.07.031 | summarize emits AnalysisSummary with TLS stats detail map | P1 | BC-TLS-031 |
| BC-2.07.032 | TLS 1.3 ClientHello legacy_version recorded as 0x0303 per JA3 spec | P1 | BC-TLS-032 |
| BC-2.07.033 | TLS analyzer ignores non-handshake records (record_type != 0x16) | P1 | BC-TLS-033 |
| BC-2.07.034 | After both hellos seen for flow, on_data short-circuits without buffering | P0 | BC-TLS-034 |
| BC-2.07.035 | on_flow_close drops per-flow TlsFlowState | P1 | BC-TLS-035 |
| BC-2.07.036 | Unknown cipher IDs render as hex 0xNNNN lowercase | P2 | BC-TLS-036 |
| BC-2.07.037 | SNI with both non-ASCII and C0 control bytes fires arm 3 (NonAsciiUtf8), not arm 2 | P0 | BC-TLS-037 |

> Full contracts: `behavioral-contracts/ss-07/BC-2.07.001.md` through `BC-2.07.037.md`

### 2.8 DNS Traffic Analysis (CAP-08)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.08.001 | DnsAnalyzer matches packets where src or dst port == 53 (TCP or UDP) | P0 | BC-DNS-001 |
| BC-2.08.002 | DNS QR-bit dispatch: response_count++ if bit set; query_count++ otherwise; returns empty findings | P0 | BC-DNS-002 |
| BC-2.08.003 | summarize emits AnalysisSummary with dns_queries and dns_responses counts | P1 | BC-DNS-003 |
| BC-2.08.004 | DnsAnalyzer NEVER emits findings (statistics-only by design) | P0 | BC-DNS-004 |

> Full contracts: `behavioral-contracts/ss-08/BC-2.08.001.md` through `BC-2.08.004.md`

### 2.9 Forensic Finding Emission (CAP-09)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.09.001 | Finding is constructed with required and optional fields as specified | P0 | BC-FND-001 |
| BC-2.09.002 | Finding Display renders [Category] VERDICT (CONFIDENCE) -- summary (raw text) | P1 | BC-FND-002 |
| BC-2.09.003 | Verdict Display: Likely/Unlikely/Inconclusive render as uppercase tokens | P1 | BC-FND-003 |
| BC-2.09.004 | Confidence Display: High/Medium/Low render as uppercase tokens | P1 | BC-FND-004 |
| BC-2.09.005 | Finding.summary and evidence store RAW post-from_utf8_lossy bytes per ADR 0003 | P0 | BC-FND-005 |
| BC-2.09.006 | Finding JSON serialization: empty Vec fields omitted (skip_serializing_if Vec::is_empty); mitre_techniques serialized as array | P0 | BC-FND-006 |

> Full contracts: `behavioral-contracts/ss-09/BC-2.09.001.md` through `BC-2.09.006.md`
>
> Known limitation: All 22 emission sites set timestamp: None (domain-debt O-01). This is
> described by BC-2.09.001 as current behavior. Finding.timestamp field exists but is never populated.

### 2.10 MITRE ATT&CK Mapping (CAP-10)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.10.001 | MitreTactic Display renders Enterprise tactics with canonical spacing | P0 | BC-MIT-001 |
| BC-2.10.002 | ICS tactics render unprefixed (no ICS: prefix) | P1 | BC-MIT-002 |
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order first then ICS-unique | P0 | BC-MIT-003 |
| BC-2.10.004 | all_tactics_in_report_order contains every variant exactly once (17 total) | P0 | BC-MIT-004 |
| BC-2.10.005 | technique_name returns Some for every seeded ID (23 Total) | P0 | BC-MIT-005 |
| BC-2.10.006 | technique_name returns None for unknown IDs | P0 | BC-MIT-006 |
| BC-2.10.007 | technique_tactic returns correct tactic for every seeded ID | P0 | BC-MIT-007 |
| BC-2.10.008 | All technique IDs currently emitted by analyzers resolve in lookup | P0 | BC-MIT-008 |
| BC-2.10.009 | MitreTactic is #[non_exhaustive] (adding variants is non-breaking) | P2 | BC-MIT-009 |

> Full contracts: `behavioral-contracts/ss-10/BC-2.10.001.md` through `BC-2.10.009.md`
>
> Domain debt O-04 (revised v1.5): 23 techniques seeded (11 Enterprise + 12 ICS); 15 emitted
> (6 Enterprise + 9 ICS). Catalogued-but-never-emitted (8): T1040, T1071, T1071.001, T1071.004,
> T1573, T1692.002, T0885 (Enterprise), T0846 (ICS — seeded but not emitted per Decision 12;
> T1692.002 replaces revoked T0856 per ATT&CK-ICS v19 remap).
> T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 are emitted by the Modbus analyzer.
> T1691.001, T0827 are emitted by the DNP3 analyzer (Feature #8).
> Arithmetic: SEEDED=23, EMITTED=15, CATALOGUE-ONLY=23−15=8.
> BC-2.10.005 documents all 23 seeded IDs; BC-2.10.008 documents 15 emitted IDs.

### 2.11 Reporting and Output (CAP-11)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.11.001 | JsonReporter renders JSON object with summary, findings, analyzers keys | P0 | BC-RPT-001 |
| BC-2.11.002 | JsonReporter includes skipped_packets in summary (zero when unset) | P1 | BC-RPT-002 |
| BC-2.11.003 | JsonReporter escapes C0 control bytes per RFC 8259 via serde | P0 | BC-RPT-003 |
| BC-2.11.004 | JsonReporter preserves non-ASCII Unicode in readable form (no unnecessary \uNNNN) | P1 | BC-RPT-004 |
| BC-2.11.005 | JsonReporter passes C1 codepoints through as raw UTF-8 (serde_json does not escape them) | P1 | BC-RPT-005 |
| BC-2.11.006 | TerminalReporter shows Skipped: N packets only when N > 0 | P1 | BC-RPT-006 |
| BC-2.11.007 | TerminalReporter escapes C0+DEL+C1+backslash in finding summary and evidence | P0 | BC-RPT-007 |
| BC-2.11.008 | TerminalReporter escape preserves printable ASCII, Cyrillic, emoji, mixed Unicode | P0 | BC-RPT-008 |
| BC-2.11.009 | TerminalReporter escapes C1 codepoints U+0080-U+009F; U+00A0 is preserved | P0 | BC-RPT-009 |
| BC-2.11.010 | TerminalReporter escapes both Finding.summary AND each evidence line | P0 | BC-RPT-010 |
| BC-2.11.011 | TerminalReporter escapes analyzer-summary detail values (closes C1 gap) | P0 | BC-RPT-011 |
| BC-2.11.012 | TerminalReporter end-to-end: C1 CSI in path-traversal finding is escaped | P0 | BC-RPT-012 |
| BC-2.11.013 | MITRE grouping emits tactic headers in all_tactics_in_report_order; Uncategorized last | P0 | BC-RPT-013 |
| BC-2.11.014 | Within tactic bucket findings sort by verdict then confidence then emission order | P1 | BC-RPT-014 |
| BC-2.11.015 | No-technique or unknown-ID findings land in Uncategorized; unknown IDs get (unknown) label | P0 | BC-RPT-015 |
| BC-2.11.016 | MITRE grouping expands per-finding line with em-dash and technique name for known IDs | P1 | BC-RPT-016 |
| BC-2.11.017 | Default (flag-off) rendering emits MITRE: <id(s)> only with no em-dash; multi-ID rendered "MITRE: T1692.001, T0836" | P1 | BC-RPT-017 |
| BC-2.11.018 | TerminalReporter colorization: Likely/High=red bold, Likely/other=yellow, Inconclusive=cyan, Unlikely=dimmed | P2 | BC-RPT-018 |
| BC-2.11.019 | TerminalReporter renders sections in order: header, PROTOCOLS, SERVICES, FINDINGS, ANALYZER summaries | P1 | BC-RPT-019 |
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns in Fixed Header Order | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.021 | CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.022 | CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell | P1 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.023 | CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.024 | CsvReporter Encodes Optional Fields as Empty Strings and mitre_techniques as Semicolon-Joined String | P1 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

> Full contracts: `behavioral-contracts/ss-11/BC-2.11.001.md` through `BC-2.11.024.md`
> (BC-2.11.020–024 added adversarial-review pass-4: CsvReporter coverage gap H-1)

### 2.12 CLI and Entry Point (CAP-12 / CLI Orchestration)

> CLI BCs are cross-cutting: they describe the entry point (C-1..C-3) that wires all capabilities
> together. Numbered under ss-12 for organizational clarity.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.12.001 | analyze subcommand parses positional targets and all analysis flags | P0 | BC-CLI-001 |
| BC-2.12.002 | summary subcommand parses positional targets and --hosts flag | P1 | BC-CLI-002 |
| BC-2.12.003 | Global flag --no-color is parsed and stored | P1 | BC-CLI-003 |
| BC-2.12.004 | Global flag --output-format json parses to Some(OutputFormat::Json); default is None | P0 | BC-CLI-004 |
| BC-2.12.005 | Reassembly CLI flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags | P0 | BC-CLI-005 |
| BC-2.12.006 | Multiple positional targets accepted in analyze | P1 | BC-CLI-006 |
| BC-2.12.007 | --reassemble and --no-reassemble are mutually exclusive (clap conflicts_with) | P0 | BC-CLI-007 |
| BC-2.12.008 | --all enables dns/http/tls together (boolean OR semantics) | P1 | BC-CLI-008 |
| BC-2.12.009 | needs_reassembly = (--reassemble OR --http OR --tls); --no-reassemble forces off with warning | P0 | BC-CLI-009 |
| BC-2.12.010 | NO_COLOR env var disables color even without --no-color flag | P2 | BC-CLI-010 |
| BC-2.12.011 | Directory target expands to all *.pcap files sorted; *.pcapng excluded from glob | P1 | BC-CLI-011 |
| BC-2.12.012 | Non-existent target yields bail! with Target not found message | P1 | BC-CLI-012 |
| BC-2.12.013 | Per-target progress bar on stderr using indicatif | P2 | BC-CLI-013 |
| BC-2.12.014 | Per-target decode errors counted into skipped_packets; only first error printed to stderr | P1 | BC-CLI-014 |
| BC-2.12.015 | dispatcher.unclassified_flows() injected into reassembly AnalysisSummary detail | P1 | BC-CLI-015 |
| BC-2.12.016 | --output-format json picks JsonReporter; --output-format csv picks CsvReporter; default terminal | P0 | BC-CLI-016 |
| BC-2.12.017 | Output routed: file path if --json <FILE> or --csv <FILE> given; stdout otherwise | P0 | BC-CLI-017 |
| BC-2.12.018 | Summary::ingest increments total_packets, total_bytes, hosts, protocol counters | P0 | BC-SUM-001 |
| BC-2.12.019 | Summary::ingest derives service name from app_protocol_hint and increments service counter | P1 | BC-SUM-002 |
| BC-2.12.020 | Summary::unique_hosts returns sorted deduplicated Vec<IpAddr> | P1 | BC-SUM-003 |
| BC-2.12.021 | Summary serializes with total_packets, total_bytes, skipped_packets fields | P1 | BC-SUM-004 |

> Full contracts: `behavioral-contracts/ss-12/BC-2.12.001.md` through `BC-2.12.021.md`

### 2.13 Absent / Unwired Feature Contracts (Documented Current Behavior)

> These BCs document flags or behaviors that do not exist in the current codebase (removed by
> PR #74). clap rejects all four as unknown arguments; there is no runtime behavior for any of
> them. They are HIGH-confidence absent contracts verified against src/cli.rs.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.13.001 | --threats flag does not exist; clap rejects it as unknown argument | P0 (absent) | BC-ABS-001 |
| BC-2.13.002 | --beacon flag does not exist; no C2 beacon analyzer exists | P0 (absent) | BC-ABS-002 |
| BC-2.13.003 | --filter <BPF> flag does not exist; no BPF filter applied | P0 (absent) | BC-ABS-003 |
| BC-2.13.004 | --verbose flag does not exist; no verbose logging mode | P2 (absent) | BC-ABS-010 |

> Full contracts: `behavioral-contracts/ss-13/BC-2.13.001.md` through `BC-2.13.004.md`

### 2.14 Modbus/ICS Analysis (CAP-14) [Feature #7 — ADR-005, ADR-006]

> **Release target: v0.4.0 (additive — no schema break).**
> All SS-14 BCs (BC-2.14.001..025) ship in v0.4.0. The `mitre_techniques: Vec<String>` type
> they depend on ships in **v0.3.0** (schema migration of existing analyzers). Modbus is built
> on top of the stable v0.3.0 contract and is purely additive at v0.4.0. See RELEASE SEQUENCING
> box in Section 2 for the full v0.3.0/v0.4.0 split rationale (f2-bundle-vs-split.md).

> **Feature Mode F2 addition (v1.1) + v2 revision (v1.2).** 25 BCs covering the Modbus TCP
> protocol analyzer (SS-14, C-22 ModbusAnalyzer). Analyzer detects 7 MITRE ATT&CK for ICS
> techniques: T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 (Remote System Information
> Discovery — recon FCs 0x11/0x2B/0x0E; **T0888 replaces prior T0846 per Decision 12**).
> Matrix discriminator: ICS technique IDs use T0xxx namespace (second char '0'),
> Enterprise use T1xxx-T9xxx. See ADR-005 for binary ICS protocol integration rationale;
> ADR-006 for multi-technique Finding attribution.
>
> **v2 co-emission model (Decision 13, ADR-006):** One finding per write-class PDU carrying
> ALL applicable technique tags (`mitre_techniques: Vec<String>`). No tag-suppression.
> Write FCs 0x06/0x10/0x16 → `["T1692.001","T0836"]`; coil FCs 0x05/0x0F → `["T1692.001","T0835"]`;
> burst/sustained rate findings → `["T0806","T1692.001"]`; T0831 co-tagged inline on per-PDU write finding → `["T1692.001","T0836","T0831"]` (no separate T0831 Finding object).
>
> **v2 dual-window burst detection (Decision 11):** Two independent CLI-configurable windows:
> `--modbus-write-burst-threshold` (default 20, 1-second burst) and
> `--modbus-write-sustained-threshold` (default 10, >=2-second sustained rolling window).
> Old `--modbus-write-threshold` flag is **REMOVED**.
>
> **CLI flags added:** `--modbus` (enable analyzer), `--modbus-write-burst-threshold N`
> (default 20; zero rejected), `--modbus-write-sustained-threshold N` (default 10; zero
> rejected). `--all` includes Modbus. Modbus analysis requires stream reassembly
> (`--no-reassemble` disables it with a warning). Dispatcher Rule 5: port-502 flows →
> `DispatchTarget::Modbus`, checked AFTER content rules (Rules 1-2) and TLS/HTTP port
> fallbacks (Rules 3-4).
>
> **Formal verification:** VP-022 covers `parse_mbap_header` (None for < 8 bytes),
> `classify_fc` (total over all 256 values), and the exception biconditional (fc >= 0x80).
> VP-004 extended: `classify_oracle` must mirror Rule 5 for port 502.

#### 2.14.A MBAP Parse and Validity Gate

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.001 | MBAP header accepted for well-formed 8-byte-minimum ADU | P0 | feature-007-F2 |
| BC-2.14.002 | MBAP header rejected for ADU shorter than 8 bytes | P0 | feature-007-F2 |
| BC-2.14.003 | MBAP header rejected when Protocol ID is not 0x0000 | P0 | feature-007-F2 |
| BC-2.14.004 | MBAP header rejected when Length is outside [2, 253] | P0 | feature-007-F2 |

#### 2.14.B Function-Code Classification

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.005 | classify_fc is total over all 256 FC values — covers Read, Write, Diagnostic, Exception, and Unknown classes | P0 | feature-007-F2 |
| BC-2.14.006 | Exception response detection — FC high bit set identifies exception and recovers original FC | P0 | feature-007-F2 |
| BC-2.14.007 | Write-class FC classification — state-changing function codes identified as elevated-risk | P0 | feature-007-F2 |
| BC-2.14.008 | Diagnostic-class FC classification and sub-function dispatch (0x08 and 0x2B) | P1 | feature-007-F2 |

#### 2.14.C Transaction Correlation

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.009 | Request PDU (client-to-server) inserted into per-flow pending table keyed on (Transaction ID, Unit ID) | P0 | feature-007-F2 |
| BC-2.14.010 | Response PDU (server-to-client) matched against pending table; entry removed on FC echo match | P0 | feature-007-F2 |
| BC-2.14.011 | Exception response PDU attributed to originating request FC via pending table lookup | P0 | feature-007-F2 |
| BC-2.14.012 | Pending table bounded to MAX_PENDING_TRANSACTIONS=256; new requests dropped (not evicting) when full | P0 | feature-007-F2 |

#### 2.14.D Finding Emission: Write-Class Events

> **v2 co-emission model (ADR-006, Decision 13):** One finding per write-class PDU carrying
> ALL applicable technique tags. No tag-suppression. Holding-register FCs (0x06/0x10/0x16) →
> `["T1692.001","T0836"]`; coil FCs (0x05/0x0F) → `["T1692.001","T0835"]`; other write FCs →
> `["T1692.001"]`. Volume control via burst aggregation (BC-2.14.017), not tag-suppression.

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.013 | Write-class FC in request direction emits multi-tag finding carrying T1692.001 and applicable technique tags; one finding per write PDU | P0 | feature-007-F2 |
| BC-2.14.014 | Write FC 0x06/0x10/0x16 in request direction emits finding tagged ["T1692.001","T0836"]; single multi-tag finding per PDU | P0 | feature-007-F2 |
| BC-2.14.015 | Write FC to coil output only (0x05/0x0F) emits finding tagged ["T1692.001","T0835"]; single multi-tag finding per PDU | P0 | feature-007-F2 |

#### 2.14.E Finding Emission: Coordinated Write (T0831) and Dual-Window Write-Burst Detection (T0806/T1692.001)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.016 | Coordinated write sequence to holding registers within 5-second window co-tags the per-PDU finding with T0831 inline (`["T1692.001","T0836","T0831"]`); no separate T0831 Finding object | P0 | feature-007-F2 |
| BC-2.14.017 | Write-rate exceeding either burst threshold (>N in 1s) or sustained threshold (>M avg over >=2s) emits `["T0806","T1692.001"]` finding; each window fires at most once per overflow | P0 | feature-007-F2 |

#### 2.14.F Finding Emission: Diagnostic/DoS (T0814) and Exception Burst Anomaly

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.018 | Diagnostics FC 0x08 sub-function 0x0004 or 0x0001 emits T0814 (Denial of Service) finding; sub-func guard h.length >= 4 | P0 | feature-007-F2 |
| BC-2.14.019 | Exception response anomaly — burst of exception codes (> 5 in 10s) emits Anomaly finding for recon/scanning | P0 | feature-007-F2 |

#### 2.14.G Anomaly/Recon, Summary, Statistics, and Bounded Resource

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.020 | Reconnaissance FCs (0x11, 0x2B/0x0E) emit T0888 (Remote System Information Discovery) finding; 0x07 not a standalone finding; unusual unknown FCs emit generic Anomaly | P1 | feature-007-F2 |
| BC-2.14.021 | summarize() returns AnalysisSummary with SIX keys: pdu_count, write_count, exception_count, function_code_distribution, parse_errors, dropped_findings (always present) | P1 | feature-007-F2 |
| BC-2.14.022 | MAX_FINDINGS cap (10,000) and poison-skip behavior for ModbusAnalyzer | P0 | feature-007-F2 |

#### 2.14.H Dispatcher and CLI Integration

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.023 | --modbus CLI flag enables ModbusAnalyzer; --all includes Modbus; default-off; requires stream reassembly | P0 | feature-007-F2 |
| BC-2.14.024 | --modbus-write-burst-threshold (default 20) and --modbus-write-sustained-threshold (default 10) configure dual-window burst detection; old --modbus-write-threshold removed | P0 | feature-007-F2 |
| BC-2.14.025 | StreamDispatcher classifies port-502 flows to DispatchTarget::Modbus as Rule 5 (after content and TLS/HTTP port rules); routes on_data and on_flow_close to ModbusAnalyzer; VP-004 oracle must mirror this rule | P0 | feature-007-F2 |

> Full contracts: `behavioral-contracts/ss-14/BC-2.14.001.md` through `BC-2.14.025.md`


### 2.15 DNP3/ICS Analysis (CAP-15) [Feature #8 — ADR-007]

> **Release target: v0.6.0 (additive — no schema break).**
> All SS-15 BCs (BC-2.15.001..024) ship in v0.6.0. The `mitre_techniques: Vec<String>` type
> and multi-tag finding model established by v0.3.0 are reused without modification. DNP3 is
> purely additive at v0.6.0.

> **Feature Mode F2 addition (v1.5).** 24 BCs covering the DNP3 TCP protocol analyzer (SS-15,
> C-26 Dnp3Analyzer). Analyzer detects 5 MITRE ATT&CK for ICS techniques directly and 2 via
> correlation: T1692.001 (unauthorized control command — direct), T0814 (restart/DoS — direct),
> T0836 (write FC — direct), T1691.001 (inferred block-command, ICS sub-technique — per-flow
> inference), T0827 (derived loss-of-control — correlated across events).
>
> **New ICS tactic variant:** `IcsImpact` (Display "Impact", TA0105) added to `MitreTactic`
> enum for T0827. `all_tactics_in_report_order` grows from 16 to 17 elements (element [16]).
> See BC-2.10.002/003/004 for the tactic enum update.
>
> **DNP3 frame model:** Link-layer header (10 bytes minimum: 8 header + 2 CRC). Validity gate:
> sync==0x0564 and LENGTH>=5. DEST/SOURCE addresses little-endian at offsets 4–7. Maximum
> frame size 292 bytes (BC-2.15.007). Carry buffer per-flow bounded to 292 bytes.
>
> **FC classification:** `classify_dnp3_fc` is total over all 256 values — Control class
> {0x03,0x04,0x05,0x06}, Restart class {0x0D,0x0E}, Write class {0x02}, Read class {0x01},
> Unknown otherwise. Transport FIR=1 gates application-layer FC extraction (BC-2.15.008).
>
> **Desync safety:** `is_non_dnp3` check — if no valid sync bytes in first 16 bytes, flow is
> silenced permanently (BC-2.15.009). Prevents false-positive finding spam on non-DNP3 flows.
>
> **Correlated findings (F2 novel):** T1691.001 (BC-2.15.014) requires a control request
> without response within a configurable window — per-flow request/response correlation.
> T0827 (BC-2.15.015) requires N restart/block events within a detection window — cross-event
> aggregation producing a single derived impact finding.
>
> **CLI flags added:** `--dnp3` (enable analyzer), `--dnp3-direct-operate-threshold N`
> (default 5; zero rejected). `--all` includes DNP3. DNP3 analysis requires stream reassembly
> (`--no-reassemble` disables it with a warning). Dispatcher Rule 6: port-20000 flows →
> `DispatchTarget::Dnp3`, checked AFTER content rules (Rules 1-2), TLS/HTTP port fallbacks
> (Rules 3-4), and Modbus Rule 5.
>
> **Formal verification:** VP-023 covers `parse_dnp3_dl_header` (None for < 10 bytes),
> `classify_dnp3_fc` (total over all 256 values), `is_valid_dnp3_frame_header` (biconditional),
> and `compute_dnp3_frame_len` (arithmetic correctness, result in [10,292]).

#### 2.15.A DL Header Parse and Validity Gate

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.001 | DNP3 DL header accepted for well-formed 10-byte-minimum frame | P0 | feature-008-F2 |
| BC-2.15.002 | DNP3 DL header rejected for frame shorter than 10 bytes (truncation safety) | P0 | feature-008-F2 |
| BC-2.15.003 | DEST/SOURCE addresses decoded little-endian from fixed offsets 4–7 | P0 | feature-008-F2 |
| BC-2.15.004 | Three-point validity gate returns true iff sync==0x0564 and LENGTH>=5 | P0 | feature-008-F2 |

#### 2.15.B Function-Code Classification

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.005 | classify_dnp3_fc is total over all 256 FC values (no gap, no panic) | P0 | feature-008-F2 |
| BC-2.15.006 | FC classification correctness — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01} | P0 | feature-008-F2 |
| BC-2.15.007 | compute_dnp3_frame_len arithmetic correct; result in [10,292]; no overflow | P0 | feature-008-F2 |

#### 2.15.C Transport Layer and Desync Safety

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.008 | Transport FIR=1 first-fragment gates application-layer FC extraction | P0 | feature-008-F2 |
| BC-2.15.009 | is_non_dnp3 desync-safe bail — flow silenced after no valid sync in first 16 bytes | P0 | feature-008-F2 |

#### 2.15.D Finding Emission: Detection (Direct Techniques)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.010 | Unauthorized control command — Unexpected source (count=1) or Control-class FC exceeding threshold emits T1692.001 | P0 | feature-008-F2 |
| BC-2.15.011 | COLD_RESTART/WARM_RESTART observed — emits T0814 per-occurrence finding | P0 | feature-008-F2 |
| BC-2.15.012 | WRITE FC observed — emits T0836 Modify-Parameter finding per-occurrence | P0 | feature-008-F2 |
| BC-2.15.013 | Co-emission ordering — direct finding (T0814/T1692.001) precedes derived T0827 | P0 | feature-008-F2 |

#### 2.15.E Finding Emission: Inferred and Correlated (T1691.001 and T0827)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.014 | Inferred block-command — control request without response within window emits T1691.001 | P0 | feature-008-F2 |
| BC-2.15.015 | Derived loss-of-control — N restart/block events in window emits T0827 as correlated finding | P0 | feature-008-F2 |

#### 2.15.F Bounded Resource and CLI Integration

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.016 | Per-flow state and carry buffer — ≤292 bytes, bounded across all flows | P0 | feature-008-F2 |
| BC-2.15.017 | --dnp3-direct-operate-threshold CLI flag controls control-command detection window | P0 | feature-008-F2 |

#### 2.15.G Anomaly Detection

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.018 | Broadcast destination anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF emits anomaly finding | P1 | feature-008-F2 |
| BC-2.15.019 | Unsolicited response anomaly — UNS bit set or FC 0x82 from unexpected pattern | P1 | feature-008-F2 |

#### 2.15.H Summary, Dispatcher, and DoS Bound

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.020 | summarize() emits function-code distribution and control-operation counts | P1 | feature-008-F2 |
| BC-2.15.021 | Port-20000 flow dispatched to Dnp3Analyzer (DispatchTarget::Dnp3, Rule 6) | P0 | feature-008-F2 |
| BC-2.15.022 | MAX_FINDINGS DoS bound — finding cap prevents unbounded all_findings growth | P0 | feature-008-F2 |

#### 2.15.I Research Must-Add Detections (Post-Gate F2, issue #8)

> Added 2026-06-10 based on `dnp3-f2-scope-threshold-validation.md` scope validation.
> Both detections map to existing T0814 — no MITRE catalog change; counts remain 23/15/8.

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.023 | Unsolicited-response enable/disable abuse — FC 0x15/0x14 observed emits T0814 | P1 | feature-008-F2 |
| BC-2.15.024 | Malformed/structural DNP3 anomaly — malformed_in_window threshold emits T0814 | P1 | feature-008-F2 |

> Full contracts: `behavioral-contracts/ss-15/BC-2.15.001.md` through `BC-2.15.024.md`


## 3. Interface Definition

> **Supplement:** Full interface definitions are in `prd-supplements/interface-definitions.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Summary: wirerust exposes a single CLI binary. Subcommands: `analyze` (produces findings),
`summary` (produces protocol/host overview). Global flags include `--output-format`,
`--no-color`, `--reassemble`, `--no-reassemble`, reassembly threshold overrides, and file
output paths (`--json <FILE>`, `--csv <FILE>`). Exit codes: 0=success, 1=fatal error.
See `prd-supplements/interface-definitions.md` for the complete flag reference, exit code
semantics, JSON output schema, and flag interaction rules.


## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog is in `prd-supplements/nfr-catalog.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

The NFR catalog (79 entries from pass-4) covers categories: PERF (throughput and latency),
SEC (memory safety, no unsafe, injection prevention), REL (overflow checks, saturating
arithmetic), OBS (counters for dropped findings, truncated records, poisoned bytes),
RES (MAX_FINDINGS cap, buffer caps, map cardinality caps), MNT (MSRV, test coverage),
PORT (Rust 2024 edition), SUP (MITRE version), COMPAT (pcap classic only).
See `prd-supplements/nfr-catalog.md` for NFR-NNN entries with numerical targets.

Known NFR violation: NFR-VIO-001 -- README's "multi-GB captures" claim is only accurate
under matching RAM constraints (eager full-file load; O-01 context).


## 5. Error Taxonomy

> **Supplement:** Full error taxonomy is in `prd-supplements/error-taxonomy.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Errors follow anyhow chaining patterns. Key categories:
- E-INP-NNN: Input / File errors (header parse failure, unsupported link type, file open failure, packet read failure)
- E-DEC-NNN: Decoder errors (unsupported link type, no IP layer, etherparse parse failure)
- E-RAS-NNN: Reassembly errors (lifecycle state-machine edge cases and resource limits)
- E-ANA-NNN: Analyzer errors (HTTP, TLS, DNS protocol-level parse failures)
- E-OUT-NNN: Output errors (file write failures for --json/--csv paths)
- E-CFG-NNN: Configuration errors (mutually exclusive flag combinations rejected by clap)
See `prd-supplements/error-taxonomy.md` for the complete E-xxx-NNN catalog.


## 6. Competitive Differentiator Traceability

> Maps each key differentiator (Section 1.3) to the behavioral contracts that implement it.

### 6.1 KD-001: Offline Single-Binary Deployment

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.001 | Link-type gating at read time: no network call needed |
| BC-2.01.002 | Eager full-file load into memory: no streaming or daemon state |
| BC-2.12.016 | All three output reporters (terminal, JSON, CSV) are self-contained |

### 6.2 KD-002: Forensic-Fidelity Raw-Data Contract

| BC ID | Contribution |
|-------|-------------|
| BC-2.09.005 | Finding.summary and evidence carry RAW post-from_utf8_lossy bytes (ADR 0003) |
| BC-2.11.003 | JsonReporter uses serde RFC 8259 escaping; does NOT call escape_for_terminal |
| BC-2.11.007 | TerminalReporter is the SOLE caller of escape_for_terminal |
| BC-2.07.020 | TLS SNI non-UTF-8 bytes preserved raw in Finding.summary |
| BC-2.07.021 | TLS SNI non-ASCII UTF-8 bytes preserved raw in Finding.summary |
| BC-2.06.026 | HTTP header bytes preserved raw at analyzer layer |

### 6.3 KD-003: Content-First Protocol Identification

| BC ID | Contribution |
|-------|-------------|
| BC-2.05.001 | 0x16 0x03 content signature routes to TLS regardless of port |
| BC-2.05.002 | HTTP method prefix routes to HTTP regardless of port |
| BC-2.05.003 | Port fallback only when content is insufficient (5 bytes minimum) |
| BC-2.05.005 | Classification cached per flow for efficiency |
| BC-2.05.006 | DispatchTarget::None not cached until retry cap (default 8); late protocol identification retried until cap, then permanently cached as None |
| BC-2.14.025 | Modbus port-502 Rule 5 checked AFTER content rules (1-2) and TLS/HTTP port fallbacks (3-4); TLS/HTTP traffic on port 502 is never stolen by Modbus rule |
| BC-2.15.021 | DNP3 port-20000 Rule 6 checked AFTER all prior rules (1-5); TLS/HTTP/Modbus traffic on port 20000 is never stolen by DNP3 rule |

### 6.4 KD-004: First-Wins TCP Overlap Forensics

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.036 | First-wins: gap bytes added; existing bytes preserved on partial overlap |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap; original data wins |
| BC-2.04.018 | ConflictingOverlap emits Anomaly/Likely/High finding with T1036 (Masquerading) |
| BC-2.04.019 | Excessive overlap threshold emits one-shot T1036 alert finding |

### 6.5 KD-005: MITRE ATT&CK Tactic-Grouped Output

| BC ID | Contribution |
|-------|-------------|
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order for deterministic grouping |
| BC-2.10.005 | technique_name lookup for all 23 seeded IDs (11 Enterprise + 12 ICS: T0846 seeded-not-emitted; T1692.001/T1692.002/T0885 existing; T0836/T0814/T0806/T0835/T0831/T0888 new Modbus; T1691.001/T0827 new DNP3 F2) |
| BC-2.11.013 | TerminalReporter MITRE grouping with tactic headers in canonical order; groups by `mitre_techniques[0]`; multi-tag findings display all IDs |
| BC-2.11.015 | Uncategorized bucket for empty `mitre_techniques` vec or all-unknown IDs |
| BC-2.11.016 | Per-finding MITRE expansion with em-dash and name |
| BC-2.14.013 | T1692.001 co-included in multi-tag finding vec for every write-class FC (ADR-006); not standalone |
| BC-2.14.014 | Holding-register writes (0x06/0x10/0x16) emit `["T1692.001","T0836"]` single multi-tag finding |
| BC-2.14.015 | Coil-only writes (0x05/0x0F) emit `["T1692.001","T0835"]` single multi-tag finding |
| BC-2.14.016 | T0831 co-tagged inline on per-PDU write finding as `["T1692.001","T0836","T0831"]`; no separate T0831 Finding object (per-PDU write finding already carries T1692.001+T0836) |
| BC-2.14.017 | Burst/sustained rate detection emits `["T0806","T1692.001"]` — dual-window model (1s burst + >=2s sustained) |
| BC-2.14.018 | T0814 (Denial of Service) emitted for Force-Listen-Only (0x0004) and Restart-Comms (0x0001) Diagnostics sub-functions |
| BC-2.14.020 | T0888 (Remote System Information Discovery) emitted for recon FCs 0x11 and 0x2B/0x0E (correctness fix; T0846 not emitted) |
| BC-2.15.010 | T1692.001 emitted for unexpected source (count=1) or Control-class FC exceeding threshold per flow (DNP3) |
| BC-2.15.011 | T0814 (Denial of Service) emitted for COLD_RESTART/WARM_RESTART FCs (DNP3) |
| BC-2.15.012 | T0836 (Modify Parameter) emitted for WRITE FC (DNP3) |
| BC-2.15.013 | Co-emission ordering — direct finding (T0814/T1692.001) precedes derived T0827; broadcast-anomaly (018↔010) dedup rule |
| BC-2.15.014 | T1691.001 (Block Operational Technology Message: Command Message) emitted via per-flow request/response correlation — control request without response within window |
| BC-2.15.015 | T0827 (Loss of Control) emitted as derived correlated finding — N restart/block events in detection window |
| BC-2.15.023 | T0814 emitted per-occurrence for DISABLE_UNSOLICITED (0x15, Likely/Medium) and ENABLE_UNSOLICITED (0x14, Possible/Low) — alarm-suppression / event-blinding primitive detection |
| BC-2.15.024 | T0814 emitted as low-confidence anomaly when malformed_in_window ≥ MALFORMED_ANOMALY_THRESHOLD [F2-GATE-DEFAULT: 3] in 300s window — Crain-Sistrunk malformed-frame crash-class coverage (parse_errors is lifetime/monotonic; malformed_in_window is the windowed threshold counter) |

### 6.6 KD-006: SNI Anomaly Detection with 4-Way Classification

| BC ID | Contribution |
|-------|-------------|
| BC-2.07.013 | Clean ASCII SNI: silent, no finding |
| BC-2.07.014 | AsciiWithControl SNI: C0/DEL bytes detected, T1027 finding |
| BC-2.07.017 | NonAsciiUtf8 SNI: non-ASCII chars detected, T1027 finding |
| BC-2.07.019 | NonUtf8 SNI: invalid UTF-8 bytes detected, T1027 finding |
| BC-2.07.037 | Disambiguation: mixed non-ASCII+control fires arm 3 (NonAsciiUtf8) not arm 2 |

### 6.7 KD-007: Bounded-Resource Design

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.024 | MAX_FINDINGS=10000 cap on reassembly engine findings |
| BC-2.04.025 | finalize bypass is the ONLY unconditional push past MAX_FINDINGS |
| BC-2.07.004 | MAX_RECORD_PAYLOAD=18432 cap on TLS record parsing |
| BC-2.07.005 | MAX_BUF=65536 per-direction buffer cap in TLS |
| BC-2.06.022 | MAX_HEADER_BUF=65536 per-direction buffer cap in HTTP |
| BC-2.04.041 | max_depth truncation prevents unbounded stream accumulation |
| BC-2.04.042 | max_receive_window rejects out-of-window segments |
| BC-2.15.016 | Per-flow DNP3 carry buffer bounded to MAX_DNP3_FRAME_LEN=292 bytes; master_addrs_seen bounded to 64 entries |
| BC-2.15.022 | MAX_FINDINGS cap prevents unbounded all_findings growth in Dnp3Analyzer |


## 7. Requirements Traceability Matrix

> Module column reflects subsystem IDs from ARCH-INDEX (ARCH-INDEX.md Subsystem Registry, Phase 1c). Priority is from Section 2.
> Test type is from BC source evidence (HIGH confidence = test exists; MEDIUM = code-only;
> LOW = ADR/comment-only).

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test Type |
|-------|----------------|-----------|----------|-----------|
| BC-2.01.001 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.002 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.003 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.004 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.005 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.006 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.007 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.008 | CAP-01 | SS-01 (reader.rs) | P2 | inferred |
| BC-2.02.001 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.002 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.003 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.004 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.005 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.006 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.007 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.008 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.009 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.010 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.011 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.012 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.013 | CAP-02 | SS-02 (decoder.rs) | P2 | inferred |
| BC-2.02.014 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.015 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.04.001 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.002 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.003 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.004 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.005 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.006 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.007 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.008 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.009 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.010 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.011 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.012 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.013 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.014 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.015 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.016 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.017 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.018 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.019 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.020 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.021 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.022 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.023 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.024 | CAP-04 | SS-04 (reassembly/) | P0 | inferred |
| BC-2.04.025 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.026 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.027 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.028 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.029 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.030 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.031 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.032 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.033 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.034 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.035 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.036 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.037 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.038 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.039 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.040 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.041 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.042 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.043 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.044 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.045 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.046 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.047 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.048 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.049 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.050 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.051 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.052 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.053 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.054 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.05.001 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.002 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.003 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.004 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.005 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.006 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.007 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.008 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.009 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.06.001 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.002 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.003 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.004 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.005 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.006 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.007 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.008 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.009 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.010 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.011 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.012 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.013 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.014 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.015 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.016 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.017 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.018 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.019 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.020 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.021 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.022 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.023 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.024 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.025 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.026 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.07.001 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.002 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.003 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.004 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.005 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.006 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.007 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.008 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.009 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit+integration |
| BC-2.07.010 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.011 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | integration |
| BC-2.07.012 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.013 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.014 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.015 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.016 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.017 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.018 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.019 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.020 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.021 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.022 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.023 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.024 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.025 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.026 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.027 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.028 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.029 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.030 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.031 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit+integration |
| BC-2.07.032 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | integration |
| BC-2.07.033 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.034 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.035 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.036 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | inferred |
| BC-2.07.037 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.08.001 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.002 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.003 | CAP-08 | SS-08 (analyzer/dns.rs) | P1 | unit |
| BC-2.08.004 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.09.001 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.09.002 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.003 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.004 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.005 | CAP-09 | SS-09 (findings.rs) | P0 | unit+integration |
| BC-2.09.006 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.10.001 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.002 | CAP-10 | SS-10 (mitre.rs) | P1 | unit |
| BC-2.10.003 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.004 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.005 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.006 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.007 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.008 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.009 | CAP-10 | SS-10 (mitre.rs) | P2 | low |
| BC-2.11.001 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.002 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.003 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.004 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.005 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.006 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.007 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.008 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.009 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.010 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.011 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.012 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.013 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.014 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.015 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.016 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.017 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.018 | CAP-11 | SS-11 (reporter/) | P2 | inferred |
| BC-2.11.019 | CAP-11 | SS-11 (reporter/) | P1 | inferred |
| BC-2.11.020 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.021 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.022 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.023 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.024 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.12.001 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.002 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.003 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.004 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.005 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.006 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.007 | CAP-12 | SS-12 (cli.rs) | P0 | inferred |
| BC-2.12.008 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.009 | CAP-12 | SS-12 (main.rs) | P0 | inferred |
| BC-2.12.010 | CAP-12 | SS-12 (main.rs) | P2 | inferred |
| BC-2.12.011 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.012 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.013 | CAP-12 | SS-12 (main.rs) | P2 | low |
| BC-2.12.014 | CAP-12 | SS-12 (main.rs) | P1 | unit |
| BC-2.12.015 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.016 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.017 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.018 | CAP-12 | SS-12 (summary.rs) | P0 | unit |
| BC-2.12.019 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.020 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.021 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.13.001 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.002 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.003 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.004 | CAP-12 | SS-13 (cli.rs) | P2 | unit |
| BC-2.14.001 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.002 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.003 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.004 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.005 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.006 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.007 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.008 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.009 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.010 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.011 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.012 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.013 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.014 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.015 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.016 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.017 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.018 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.019 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.020 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.021 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.022 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.023 | CAP-14 | SS-12 (cli.rs, main.rs) + SS-14 | P0 | unit+integration |
| BC-2.14.024 | CAP-14 | SS-12 (cli.rs, main.rs) + SS-14 | P0 | unit+integration |
| BC-2.14.025 | CAP-14 | SS-05 (dispatcher.rs) + SS-14 | P0 | unit+kani |
| BC-2.15.001 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.002 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.003 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.004 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.005 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.006 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.007 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.008 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.009 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.010 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.011 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.012 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.013 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.014 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.015 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.016 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.017 | CAP-15 | SS-12 (cli.rs, main.rs) + SS-15 | P0 | unit+integration |
| BC-2.15.018 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.019 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.020 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.021 | CAP-15 | SS-05 (dispatcher.rs) + SS-15 | P0 | unit+kani |
| BC-2.15.022 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.023 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.024 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |


## 8. Domain Debt Index

> These open items from domain-debt.md are cross-referenced here for quick lookup.
> They describe CURRENT BEHAVIOR as of develop HEAD, not future requirements.

| Item | Description | Affected BCs |
|------|-------------|--------------|
| O-01 | Finding.timestamp always None; RawPacket timestamps never threaded to Finding constructors | BC-2.09.001, BC-2.09.006 |
| O-02 | Absent User-Agent (None) intentionally not detected; only Some("") fires | BC-2.06.011 |
| O-03 | Anomaly thresholds not empirically calibrated against labelled traffic | BC-2.04.019, BC-2.04.020, BC-2.04.021 |
| O-04 | 8 MITRE techniques catalogued but never emitted (T1040, T1071, T1071.001, T1071.004, T1573, T1692.002, T0885, T0846; T1692.002 replaces revoked T0856 per ATT&CK-ICS v19 remap; T0846 seeded-not-emitted per Decision 12); T1692.001/T0836/T0814/T0806/T0835/T0831/T0888 now emitted by Modbus analyzer (Feature #7); T1691.001/T0827 now emitted by DNP3 analyzer (Feature #8); SEEDED=23, EMITTED=15, CATALOGUE-ONLY=8 | BC-2.10.005 |
| O-05 | reassembly/mod.rs still 691 LOC after partial split (#85) | BC-2.04.* (reassembly module group) |
| O-06 | Weak-cipher Finding evidence Vec has unbounded cardinality (up to ~9216 cipher names) | BC-2.07.009 |
| O-07 | rayon declared in Cargo.toml but never imported; unused transitive dependency | (none -- build/dep debt only) |
| O-08 | dns.rs module doc-comment (lines 1-7) describes DGA/entropy/NXDOMAIN/rare-TLD detection not implemented; DnsAnalyzer is statistics-only (QR-bit counters, always returns empty findings Vec) | BC-2.08.001-004 |
