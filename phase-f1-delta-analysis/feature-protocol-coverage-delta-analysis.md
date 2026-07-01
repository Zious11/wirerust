---
document_type: feature-delta-analysis
feature_id: feature-protocol-coverage
cycle: feature-protocol-coverage
title: "Protocol Coverage Report — static catalog + dynamic gap detection"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  Two distinct CLI surfaces (new 'protocols' subcommand + dynamic gap report during
  analyze), a new module (src/protocols.rs), dispatcher state augmentation, a new
  subsystem (SS-18), ~9 new BCs, ~2 new VPs, and one new ADR. Fails every trivial
  criterion.
scope_classification: standard
status: draft
producer: architect
created: 2026-07-01
base_commit: 3a60317
branch: develop
spec_at_analysis:
  bc_index: v2.3
  vp_index: v2.28
  arch_index: v2.5
  prd: v1.45
  story_index: v3.10
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
---

# F1 Delta Analysis — feature-protocol-coverage

## 1. Feature Summary

The human requests two surfaces:

**Surface 1 — Static coverage report (independent of any capture)**

A CLI command (e.g. `wirerust protocols --unsupported` or `wirerust protocols --supported`)
that prints which protocols wirerust knows about from a curated reference catalog but does
NOT yet dissect, and conversely which protocols ARE supported. This requires:

- A curated `KNOWN_PROTOCOLS` catalog embedded in the binary (ICS/network protocol list).
- A mechanism to derive the "supported" set from the currently implemented dissectors.
- A new `protocols` subcommand in `cli.rs`.

**Surface 2 — Dynamic gap detection (during a capture analysis run)**

During `wirerust analyze`, surface traffic that matched `DispatchTarget::None` in the
stream dispatcher, broken down by transport and port — e.g. "undissected protocol observed
on TCP/102, 47 packets". This detects coverage gaps from actual captured traffic.

wirerust already maintains `unclassified_flows: u64` in `StreamDispatcher` (a simple
aggregate count), so the dynamic surface is an enrichment of existing infrastructure
rather than a new mechanism.

---

## 2. Intent and Scope Classification

### Intent Classification

**Classified intent:** `feature`

**Rationale:** The human says "a way to list out protocols that we don't have dissectors
for." Both surfaces are new capabilities that do not exist in the codebase. No existing
behavior is broken or corrected.

### Feature Type Classification

**Classified type:** `backend`

**Rationale:** CLI tool with no web/browser/UI surface. All changes are in Rust source
(cli.rs, main.rs, dispatcher.rs, new src/protocols.rs). No frontend, no network I/O,
no external services.

### Trivial Scope Classification

**Classified scope:** `standard`

Trivial checklist — ALL must be true; none are true here:

- [x] Single module/file? **No** — touches dispatcher.rs, cli.rs, main.rs, lib.rs, new
  src/protocols.rs; minimum 4 files with substantial changes.
- [x] No new BCs? **No** — ~9 new BCs required across SS-05, SS-12, and new SS-18.
- [x] No architecture change? **No** — new subsystem SS-18 added to ARCH-INDEX Subsystem
  Registry; new component C-26 in module-decomposition.
- [x] No new external dependencies? **True** — protocol catalog is a static Rust constant;
  no new crates needed.
- [x] Regression risk LOW? **No** — dispatcher.rs is a VP-004 Kani-verified module; any
  change requires proof re-validation.

Standard pipeline applies: full F1 → F7.

---

## 3. Impact Boundary

### 3.1 Source-Code Component Map

| File | Change Type | Subsystem | Risk | Rationale |
|------|-------------|-----------|------|-----------|
| `src/protocols.rs` | **NEW** | SS-18 (new) | LOW | Pure-core catalog module; no side effects; new file, zero breakage risk |
| `src/cli.rs` | MODIFIED | SS-12 | LOW | Add `Protocols` variant to `Commands` enum + flags; clap additive; existing subcommands unaffected |
| `src/main.rs` | MODIFIED | SS-12 | LOW | Wire `Commands::Protocols` match arm; no change to existing `run_analyze`/`run_summary` paths |
| `src/lib.rs` | MODIFIED | SS-12 | LOW | Add `pub mod protocols;` declaration; one line |
| `src/dispatcher.rs` | MODIFIED | SS-05 | MEDIUM | Augment `StreamDispatcher` with per-(transport, port) unclassified-flow tracking; adds new HashMap field + `on_flow_close` population + accessor; `classify()` function and `DispatchTarget` enum NOT changed; VP-004 Kani proofs unaffected; regression zone for SS-05 BCs |

### 3.2 Architecture Components

| Component | Status |
|-----------|--------|
| C-26: `src/protocols.rs` — Protocol Coverage Catalog | **NEW** |
| C-4: `src/dispatcher.rs` (StreamDispatcher) | MODIFIED — internal state only |
| C-22: `src/cli.rs` (CLI definition) | MODIFIED — new subcommand |
| C-23: `src/main.rs` (entry point) | MODIFIED — new dispatch arm |

### 3.3 Subsystem Impact

| Subsystem | Impact | Scope |
|-----------|--------|-------|
| SS-18 (Protocol Coverage Catalog) | **NEW** — must be added to ARCH-INDEX Subsystem Registry | New `src/protocols.rs` |
| SS-12 (CLI / Entry) | MODIFIED — new subcommand | cli.rs, main.rs |
| SS-05 (Protocol Dispatch) | MODIFIED — new tracking state | dispatcher.rs |
| SS-11 (Reporting) | DEPENDENT — may need minor output additions for dynamic gap section | reporter/terminal.rs, reporter/json.rs |
| All other subsystems (SS-01/02/04/06..10/13..17) | NOT CHANGED | Regression baseline |

---

## 4. Affected Specifications

### 4.1 New Behavioral Contracts (proposed IDs, not final)

**SS-05 additions (2 new BCs, next ID = BC-2.05.010):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.05.010 | Per-(Transport,Port) Unclassified-Flow Tracker — `unclassified_port_counts` field accumulates flow counts keyed by (transport_str, port) at `on_flow_close` for None-target flows | P1 |
| BC-2.05.011 | Unclassified Port Counts Accessor — `unclassified_port_counts()` returns immutable reference to the per-port map for injection into the reporting pipeline | P1 |

**SS-12 additions (3 new BCs, next ID = BC-2.12.022):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.12.022 | `protocols` Subcommand Exists and Parses `--supported`, `--unsupported`, `--all` Flags | P0 |
| BC-2.12.023 | `protocols --supported` Prints Currently-Implemented Dissector Protocols | P0 |
| BC-2.12.024 | `protocols --unsupported` Prints Catalog Protocols Not Yet Dissected | P0 |

**SS-18 additions (4 new BCs, all new IDs = BC-2.18.001..004):**

| Proposed ID | Title | Priority |
|-------------|-------|----------|
| BC-2.18.001 | `KNOWN_PROTOCOLS` Is a Static, Compile-Time Catalog of ICS/Network Protocol Entries | P0 |
| BC-2.18.002 | Each `KnownProtocol` Entry Carries: name, transport, canonical_port(s), category (ICS/network), brief description | P1 |
| BC-2.18.003 | `supported_protocols()` Is Derived from the Dispatcher's Known DispatchTargets — Not a Separate Hand-Maintained List | P0 |
| BC-2.18.004 | `unsupported_protocols()` Is the Set Difference `KNOWN_PROTOCOLS` minus `supported_protocols()` — Deterministic Pure Function | P0 |

**Total new BCs: 9**

### 4.2 Amended Behavioral Contracts

| BC | Amendment Scope |
|----|----------------|
| BC-2.05.007 (unclassified_flows increments only at on_flow_close) | Extend postconditions to cover `unclassified_port_counts` population in the same arm; invariant now covers both counters. |
| BC-2.12.015 (dispatcher.unclassified_flows() injected into reassembly summary) | Extend to also inject `unclassified_port_counts()` map into the reporting pipeline (or as a separate analyzer summary entry). Scope TBD pending human decision on dynamic output format. |

**Total amended BCs: 2**

### 4.3 Verification Properties

**New VPs (proposed IDs, next available = VP-041):**

| Proposed ID | Tool | Phase | Description |
|-------------|------|-------|-------------|
| VP-041 | proptest | P1 | `unsupported_protocols()` is the exact set difference of `KNOWN_PROTOCOLS` minus `supported_protocols()`: for any supported-set S and catalog C, every entry in `unsupported_protocols()` is in C and NOT in S, and every entry in C not in S is in `unsupported_protocols()`. Pure-core function; proptest strategies over subsets. |
| VP-042 | proptest | P1 | `unclassified_port_counts` accumulates correctly: after N `on_flow_close` calls on None-target flows with varying ports, the map totals equal N and the per-port counts equal the call frequency per port. Dispatcher state machine property. |

**Amended VPs:** None. VP-004 (dispatch precedence Kani proof) is NOT affected — `classify()` and `DispatchTarget` are unchanged; the new HashMap field is in `StreamDispatcher` state, not in the classification logic. The Kani oracle model (`step_none_path`, `classify_oracle`) remains valid.

**Total new VPs: 2**

### 4.4 Architecture Documents

| Document | Change Required |
|----------|----------------|
| `ARCH-INDEX.md` | Add SS-18 row to Subsystem Registry; update Document Map (C-26, component count 25→26); Bounded-Resource Design note: `unclassified_port_counts` HashMap capacity |
| `module-decomposition.md` | Add C-26 entry for `src/protocols.rs` (SS-18) |
| `api-surface.md` | Add `protocols` subcommand surface |
| `system-overview.md` | Add static catalog pipeline path (protocols subcommand); add dynamic gap report as output of analyze pipeline |
| `verification-architecture.md` | Add VP-041 + VP-042 to Provable Properties Catalog; P1 list |
| `verification-coverage-matrix.md` | Add VP-041 + VP-042 rows; update totals (40→42) |

### 4.5 ADRs

| ADR | Status |
|-----|--------|
| ADR-012 (new) | Protocol Coverage Catalog design: (1) static compile-time array vs. external data file; (2) supported-set derivation — dispatcher introspection vs. hand-maintained list; (3) catalog scope (ICS-only vs. ICS + common IT protocols); (4) output format for `protocols` subcommand (plain terminal vs. JSON-flag-aware); (5) dynamic gap output surface (Finding vs. analyzer summary entry vs. both) |

### 4.6 PRD Sections

| Section | Change |
|---------|--------|
| PRD §2.5 (Protocol Dispatch) | Amend to add BC-2.05.010/011 |
| PRD §2.12 (CLI / Entry) | Add BC-2.12.022..024 |
| PRD §2.18 (new — Protocol Coverage Catalog) | Create; 4 new BCs |
| PRD §7 (RTM) | Add new rows for all 9 new BCs |

---

## 5. Files Changed

### New Files

| File Path | Purpose |
|-----------|---------|
| `src/protocols.rs` | Static `KNOWN_PROTOCOLS` catalog + `KnownProtocol` struct + `supported_protocols()` + `unsupported_protocols()` pure functions |

### Modified Files

| File Path | Change Type | Risk |
|-----------|------------|------|
| `src/dispatcher.rs` | Internal state: new `unclassified_port_counts: HashMap<(u16, u16), u64>` field; new `on_flow_close` arm population; new accessor method | MEDIUM |
| `src/cli.rs` | Additive: new `Protocols { supported, unsupported, all }` variant added to `Commands` enum | LOW |
| `src/main.rs` | Additive: new `Commands::Protocols` match arm in `main()`; new `run_protocols()` function | LOW |
| `src/lib.rs` | One line: `pub mod protocols;` declaration | LOW |

### Dependent Files (unchanged unless dynamic output format requires reporter changes)

| File Path | Depends On | Regression Risk | Notes |
|-----------|-----------|----------------|-------|
| `src/reporter/terminal.rs` | `dispatcher.rs` (AnalysisSummary) | LOW | May need minor addition if dynamic gap section rendered in terminal output; pending human decision |
| `src/reporter/json.rs` | `dispatcher.rs` (AnalysisSummary) | LOW | Same; BTreeMap key order preserved by existing serialization |
| `src/summary.rs` | `decoder.rs` only | NONE | Summary struct and `ingest()` not touched |

---

## 6. Files NOT Changed (Regression Baseline)

The following files must not be modified and all their tests must continue to pass:

- `src/analyzer/arp.rs` — ARP analyzer; no dependency on catalog or dispatch tracking
- `src/analyzer/dnp3.rs` — DNP3 analyzer
- `src/analyzer/dns.rs` — DNS analyzer
- `src/analyzer/enip.rs` — EtherNet/IP analyzer
- `src/analyzer/http.rs` — HTTP analyzer
- `src/analyzer/modbus.rs` — Modbus analyzer
- `src/analyzer/tls.rs` — TLS analyzer (PERF-001/002 sensitivity zone — must not be touched)
- `src/decoder.rs` — packet decoder (note: `app_protocol_hint` already maps SSH/SMB/Modbus/DNP3 etc.; this is a separate mechanism from the protocol catalog)
- `src/reader.rs` — pcap/pcapng reader
- `src/findings.rs` — Finding struct
- `src/mitre.rs` — MITRE catalog
- `src/summary.rs` — Summary struct
- `src/reassembly/` (all files) — TCP reassembly engine
- `src/analyzer/mod.rs` — ProtocolAnalyzer trait definition

---

## 7. Regression Risk Assessment

| Risk Type | Level | Rationale |
|-----------|-------|-----------|
| Regression — static surface (protocols subcommand) | LOW | Entirely additive (new `Commands` variant + new `run_protocols()` function); existing `Commands::Analyze` and `Commands::Summary` arms are not changed; clap additive extension is safe |
| Regression — dynamic surface (dispatcher state) | MEDIUM | `src/dispatcher.rs` carries VP-004 Kani proofs (3 harnesses) and BC-2.05.001..009 (9 BCs, all with existing tests). New HashMap field and `on_flow_close` population are additive to the `None` arm, but: (a) any accidental mutation of the `routes` HashMap or `classification_attempts` logic would break proofs; (b) new HashMap could interfere with the `on_flow_close` `None` guard that checks `is_some()` for all analyzers. Requires careful implementation. |
| Regression — TLS carry-path (PERF-001/002) | LOW | `src/analyzer/tls.rs` is NOT touched by this feature. The +10.3% TLS carry-path sensitivity (STORY-149 backlog item) is fully isolated to tls.rs; the dispatcher change is off the TLS hot path. |
| Architecture | LOW | New subsystem SS-18 is a pure-core, no-dependency module. No circular dependencies introduced. |
| Security | LOW | Protocol catalog is a compile-time constant array; no external input parsing; no user-controlled data enters the catalog comparison logic. |
| Performance | LOW | `unclassified_port_counts` HashMap is populated only at `on_flow_close` (not on every `on_data` call). None-target flows that hit the retry cap are already cached in `routes`, so `on_flow_close` for those is a single HashMap entry operation. For None flows that close before hitting the cap, the work is identical to the current `unclassified_flows += 1` plus one HashMap entry op. Expected overhead: negligible on typical captures (unclassified flows are rare when analyzers match well). |

### 7.1 Dispatcher Regression Detail

`classify()` is NOT changed. `DispatchTarget` enum is NOT changed. The Kani oracle
function `classify_oracle` is NOT changed. VP-004's three proof harnesses remain
sound: `verify_tls_signature_beats_port`, `verify_content_first_precedence_exhaustive`,
and `verify_none_two_phase_caching`.

The only dispatcher change is in `on_flow_close` — the existing `None` arm:

```rust
// BEFORE:
Some(DispatchTarget::None) | None => {
    if self.http.is_some() || ... {
        self.unclassified_flows += 1;
    }
}

// AFTER (proposed):
Some(DispatchTarget::None) | None => {
    if self.http.is_some() || ... {
        self.unclassified_flows += 1;
        // NEW: per-port tracking (direction-independent, keyed by flow_key ports)
        let entry = self.unclassified_port_counts.entry((lower, upper)).or_insert(0);
        *entry = entry.saturating_add(1);
    }
}
```

The `flow_key` is already passed to `on_flow_close` — no signature change needed.

---

## 8. Existing Tests in Regression Risk Zone

Tests that exercise the affected modules and must remain passing:

| Test Location | BC(s) Covered | Risk |
|---------------|--------------|------|
| `src/dispatcher.rs` Kani harnesses (`#[cfg(kani)]`) | VP-004 | MEDIUM — must run `cargo kani` post-change |
| Tests for BC-2.05.001..009 (in dispatcher tests or `tests/`) | SS-05 all | MEDIUM |
| Test for BC-2.12.015 (`unclassified_flows()` injection into reassembly summary) | SS-12 | LOW — accessor name unchanged |
| End-to-end `run_analyze()` integration tests | SS-12 | LOW — no change to analyze path logic |
| Any tests in `tests/` calling `StreamDispatcher::new()` | SS-05 | LOW — constructor signature unchanged |

---

## 9. New Artifacts Required

### 9.1 Protocol Catalog — Design Decision Required

The feature requires a source-of-truth catalog of "known protocols." Design options:

**Option A (recommended): Static compile-time array in `src/protocols.rs`**

```rust
pub struct KnownProtocol {
    pub name: &'static str,
    pub transport: &'static str,   // "TCP", "UDP", "Link-Layer"
    pub canonical_ports: &'static [u16],
    pub category: ProtocolCategory, // ICS | Network | Both
    pub description: &'static str,
}

pub const KNOWN_PROTOCOLS: &[KnownProtocol] = &[
    KnownProtocol { name: "Modbus TCP", transport: "TCP", canonical_ports: &[502], category: ProtocolCategory::ICS, description: "SCADA/ICS Modbus over TCP" },
    KnownProtocol { name: "DNP3", transport: "TCP", canonical_ports: &[20000], category: ProtocolCategory::ICS, description: "DNP3 distributed automation" },
    KnownProtocol { name: "EtherNet/IP", transport: "TCP", canonical_ports: &[44818], category: ProtocolCategory::ICS, description: "CIP over EtherNet/IP (Allen-Bradley)" },
    KnownProtocol { name: "BACnet/IP", transport: "UDP", canonical_ports: &[47808], category: ProtocolCategory::ICS, description: "Building automation" },
    KnownProtocol { name: "OPC-UA", transport: "TCP", canonical_ports: &[4840], category: ProtocolCategory::ICS, description: "OPC Unified Architecture" },
    KnownProtocol { name: "Profinet", transport: "UDP", canonical_ports: &[34964], category: ProtocolCategory::ICS, description: "Siemens industrial Ethernet" },
    KnownProtocol { name: "S7comm", transport: "TCP", canonical_ports: &[102], category: ProtocolCategory::ICS, description: "Siemens S7 PLC protocol" },
    KnownProtocol { name: "IEC 61850 MMS", transport: "TCP", canonical_ports: &[102], category: ProtocolCategory::ICS, description: "IEC 61850 MMS (shares port 102 with S7)" },
    KnownProtocol { name: "TLS", transport: "TCP", canonical_ports: &[443, 8443], category: ProtocolCategory::Network, description: "TLS handshake analysis" },
    KnownProtocol { name: "HTTP", transport: "TCP", canonical_ports: &[80, 8080], category: ProtocolCategory::Network, description: "HTTP/1.x traffic" },
    KnownProtocol { name: "DNS", transport: "UDP", canonical_ports: &[53], category: ProtocolCategory::Network, description: "DNS query/response" },
    // ... ARP, SSH, SMB, RDP, FTP, SMTP, etc. — scope TBD by human
];
```

**Option B: External JSON/TOML file**

Pro: editable without recompilation. Con: adds file I/O to what should be a pure-core
query; contradicts wirerust's "no network I/O" principle for a CLI tool.

**Recommendation:** Option A. Compile-time constant is verifiable (VP-041), has zero
I/O, is suitable for Kani/proptest, and aligns with wirerust's existing static-catalog
patterns (MITRE catalog in `src/mitre.rs`).

### 9.2 Supported-Set Derivation

The `supported_protocols()` function must derive the supported set from the dispatcher's
known `DispatchTarget` variants — NOT from a separate hand-maintained list. This prevents
drift: when a new analyzer is added and a new `DispatchTarget` variant is created, the
`supported_protocols()` output updates automatically.

Concretely: `supported_protocols()` returns the subset of `KNOWN_PROTOCOLS` entries whose
`canonical_ports` overlap with the dispatcher's port-classification rules (502, 20000,
44818, 80, 8080, 443, 8443, 53 — derived from `classify()` source) PLUS link-layer
protocols handled outside the dispatcher (ARP via `DecodedFrame::Arp`).

This can be a pure compile-time constant slice (since the dispatcher's ports are known
statically) — no runtime introspection needed. The connection between catalog and
dispatcher is maintained by a compile-time assertion or documented invariant in ADR-012.

---

## 10. Open Questions for Human Gate

The following decisions must be resolved before F2 spec evolution can begin. F2 is
blocked on items marked **BLOCKING**.

| # | Question | Blocking? | Options |
|---|----------|-----------|---------|
| OQ-1 | **Catalog scope**: ICS protocols only (Modbus, DNP3, EtherNet/IP, BACnet, OPC-UA, Profinet, S7comm, IEC 61850) or also common IT/network protocols (SSH, FTP, SMTP, RDP, SMB, RTP, SIP, etc.)? `decoder.rs` already hints SSH (port 22) and SMB (port 445). | **BLOCKING** | ICS-only (narrower, more focused); ICS + core IT (SSH/SMB/FTP/SMTP/RDP/SIP); All known IANA (very broad, less useful) |
| OQ-2 | **Dynamic detection output surface**: Should unclassified traffic surface as (a) entries in the existing `AnalysisSummary.detail` map (like `unclassified_flows` today — aggregate in the reassembly summary); (b) a new `CoverageGapsSummary` analyzer summary; or (c) individual `Finding` entries (one per port with N-packet count — these enter the finding pipeline with severity + potential MITRE)? | **BLOCKING** | Option (a) is lowest-risk; option (c) requires MITRE mapping decisions |
| OQ-3 | **`protocols` subcommand output format**: Should `wirerust protocols --unsupported` produce (a) plain terminal table only; (b) terminal table + `--json` flag awareness (structured JSON output); or (c) terminal table + JSON + CSV? LESSON-P1.04 ("no unwired flags") means any global flag accepted must be wired — global `--json` / `--csv` flags exist today and clap will pass them to all subcommands. | MEDIUM | Option (b) is recommended: wire `--json` for the protocols subcommand; `--csv` is less natural for a catalog listing |
| OQ-4 | **`--all` interaction during analyze**: Should `wirerust analyze --all <file>` automatically append a "Coverage Gaps" section showing unclassified flows by port (making it easier to see gaps without a separate command), or is the dynamic gap output always opt-in via a new flag (e.g. `--coverage-gaps`)? | MEDIUM | Auto-in with `--all` is convenient; explicit flag is more deliberate |
| OQ-5 | **UDP scope for dynamic detection**: `StreamDispatcher` handles only TCP flows. UDP protocols (DNS, BACnet, SNMP, etc.) are NOT routed through the dispatcher. If a UDP packet carries BACnet on port 47808 it will only appear in `Summary.services` (via `app_protocol_hint`) but NOT in the dispatcher's unclassified-flow tracking. Do you want: (a) TCP-only dynamic detection (simpler, no new code paths); or (b) UDP gap detection too (requires new UDP-flow tracking outside the dispatcher, likely in the decode loop in main.rs)? | MEDIUM | Option (a) recommended for this cycle; UDP can be a follow-on |
| OQ-6 | **`protocols --all` flag behavior**: Should `protocols --all` list ALL catalog entries (both supported and unsupported), or should `--all` be spelled differently to avoid confusion with `analyze --all`? | LOW | `--all` or `--list` or omit (default behavior = show all) |

---

## 11. Recommended F2/F3 Sequencing

### F2 Spec Evolution (after human resolves OQ-1 through OQ-3)

1. **ADR-012**: Capture catalog design decisions (OQ-1 catalog scope, OQ-2 output
   surface, OQ-3 output format, supported-set derivation approach).
2. **9 new BCs**: BC-2.05.010..011 (SS-05), BC-2.12.022..024 (SS-12),
   BC-2.18.001..004 (SS-18 new).
3. **2 amended BCs**: BC-2.05.007 v2.x (extend postconditions), BC-2.12.015 v1.x
   (extend injection scope).
4. **2 new VPs**: VP-041 (proptest; pure catalog set-difference), VP-042 (proptest;
   dispatcher unclassified-port-counts accumulation).
5. **ARCH-INDEX update**: SS-18 entry, C-26, component count 25→26.

### F3 Story Decomposition (after F2 approval)

Recommended wave structure (rough estimate, ~5 stories):

| Wave | Story | Surface | Est. Pts |
|------|-------|---------|---------|
| Wave 67 | STORY-151: `src/protocols.rs` — KNOWN_PROTOCOLS catalog + KnownProtocol struct + supported_protocols() + unsupported_protocols() pure functions + proptest VP-041 | Static | 5 |
| Wave 67 | STORY-152: `protocols` CLI subcommand — cli.rs + main.rs `run_protocols()` + terminal output | Static | 5 |
| Wave 68 | STORY-153: Dispatcher dynamic gap tracking — `unclassified_port_counts` field + on_flow_close population + accessor + BC-2.05.010/011 tests | Dynamic | 5 |
| Wave 68 | STORY-154: Gap report integration — inject `unclassified_port_counts()` into analysis output (reporter extension per OQ-2 decision) + BC-2.12.015 amendment | Dynamic | 3 |
| Wave 69 | STORY-155: Formal hardening — VP-042 proptest harnesses; VP-041 proptest confirm; cargo-mutants on protocols.rs; re-run Kani VP-004 to confirm no regression | Hardening | 5 |

STORY-151 and STORY-152 are independent (static surface); can be decomposed in parallel.
STORY-153 is a prerequisite for STORY-154. STORY-155 follows all implementation stories.

**Total estimated stories: 5, ~23 story points**

---

## 12. Impact Summary Table

| Dimension | Count | Details |
|-----------|-------|---------|
| New BCs | 9 | BC-2.05.010..011, BC-2.12.022..024, BC-2.18.001..004 |
| Amended BCs | 2 | BC-2.05.007, BC-2.12.015 |
| New VPs | 2 | VP-041 (proptest), VP-042 (proptest) |
| Amended VPs | 0 | VP-004 Kani proofs unaffected |
| New ADRs | 1 | ADR-012 (catalog design) |
| New subsystems | 1 | SS-18 Protocol Coverage Catalog |
| New source files | 1 | `src/protocols.rs` |
| Modified source files | 4 | dispatcher.rs, cli.rs, main.rs, lib.rs |
| Estimated stories | 5 | 2 static + 2 dynamic + 1 hardening |
| Regression risk | MEDIUM | Dispatcher is a VP-004 Kani zone; TLS carry-path NOT touched |
| Feature type | backend | Pure Rust CLI; no UI/network/external services |
| Intent | feature | New capability; no existing behavior corrected |
| Scope | standard | Full F1-F7 pipeline |
