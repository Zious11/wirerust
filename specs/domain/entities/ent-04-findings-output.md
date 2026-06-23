---
artifact: L2-ent-04
traces_to: ../domain-spec.md
title: Entities -- Findings and Output (L3-L4)
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.4"
modified:
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-10 sibling-sweep (DF-SIBLING-SWEEP-001) F-D10-L02: stale 'E-27 MitreTactic 16-variant enum (14 Enterprise + 2 ICS)' corrected to '17-variant enum (14 Enterprise + 3 ICS-unique incl. IcsImpact)' — IcsImpact added in Feature #8 (issue #8, ADR-007)."
  - date: 2026-06-13
    actor: product-owner
    reason: "ARP-F2 Pass-14 remediation C-03: E-26 'all four Option fields' corrected to 'three remaining Option fields' — mitre_techniques is Vec<String> with skip_serializing_if=Vec::is_empty (not an Option; STORY-100 AC-008/ADR-006 Decision 13). O-01 closed: timestamp wired in STORY-097/098/099. Version 1.1→1.2."
  - date: 2026-06-13
    actor: product-owner
    reason: "P19 straggler anchor sweep: E-23 Verdict :30-40 → :32-46; E-24 Confidence :57-66 → :66-73; E-27 MitreTactic :45-66 → :47-70. Verified against src/findings.rs and src/mitre.rs. Version 1.2→1.3."
  - date: 2026-06-23
    actor: product-owner
    reason: "F5 ICS tactic-ID correctness fix (D-209, DF-SIBLING-SWEEP-001): E-27 MitreTactic variant count 17→20 (14 Enterprise + 6 ICS). Three new ICS-unique variants: IcsDiscovery (TA0102), IcsCollection (TA0100), IcsCommandAndControl (TA0101). Version 1.3→1.4."
---

# Entities: Findings and Output (L3-L4)

Covers E-23 through E-28, E-36 through E-39. Source: pass-2-domain-model.md + pass-3-R4.md.

## E-23: Verdict (src/findings.rs:32-46)

```
enum Verdict { Likely, Unlikely, Inconclusive }
```

`#[non_exhaustive]` (P2.10 / #76). Derives `Debug, Clone, Copy, PartialEq, Eq, Serialize`.
`Display` renders uppercase (`LIKELY`, etc.).
`verdict_rank` order (for sorting): `Likely < Inconclusive < Unlikely` (terminal.rs:269-275).

## E-24: Confidence (src/findings.rs:66-73)

```
enum Confidence { High, Medium, Low }
```

`#[non_exhaustive]` (P2.10 / #76). Derives same as Verdict.
`confidence_rank` order: `High < Medium < Low` (terminal.rs:276-282).

## E-25: ThreatCategory (src/findings.rs:88-111)

```
enum ThreatCategory {
    Reconnaissance, LateralMovement, C2, Exfiltration,
    CredentialAccess, Persistence, Execution, Anomaly
}
```

`#[non_exhaustive]` (P2.10 / #76). `LateralMovement` and `C2` are defined but never emitted
by any analyzer (grep for `::C2,` / `::LateralMovement,` in src/ returns zero hits).
`Display` uses the Debug formatter (variant name verbatim).

**Consistency with MitreTactic:** Both `ThreatCategory` and `MitreTactic` are now
`#[non_exhaustive]` (the prior inconsistency closed by P2.10).

## E-26: Finding (src/findings.rs:135-162)

The critical output type. See CAP-09 for the full schema and all emission sites.

`mitre_techniques: Vec<String>` (STORY-100 / ADR-006 Decision 13) uses
`skip_serializing_if = "Vec::is_empty"` — the key is absent from JSON when no technique is
attributed. The old scalar `mitre_technique: Option<String>` field was removed (STORY-100
AC-008).

Three remaining Option fields (`source_ip`, `timestamp`, `direction`) each use
`skip_serializing_if = "Option::is_none"`. No Option field ever serializes as `null`.

`timestamp: Option<DateTime<Utc>>` is now wired at all emission sites (STORY-097/098/099
for http/tls/reassembly; STORY-102..110 for modbus/dnp3). O-01 is closed.

`direction: Option<Direction>` was added (P2.08 / #77). HTTP and TLS analyzer findings set
it; reassembly-engine findings leave it None.

## E-27: MitreTactic (src/mitre.rs:47-70)

20-variant enum (14 Enterprise + 6 ICS-unique: IcsInhibitResponseFunction, IcsImpairProcessControl,
IcsImpact, IcsDiscovery, IcsCollection, IcsCommandAndControl). `#[non_exhaustive]` (VO-5). Derives `Debug, Clone,
Copy, PartialEq, Eq, Hash`. `Display` renders canonical English names. See CAP-10 for full
variant list. Three ICS variants added in F5 D-209: IcsDiscovery (TA0102), IcsCollection (TA0100),
IcsCommandAndControl (TA0101).

## E-28: AnalysisSummary (src/analyzer/mod.rs:38-50)

```
struct AnalysisSummary {
    analyzer_name:    String,
    packets_analyzed: u64,
    detail:           BTreeMap<String, serde_json::Value>,
}
```

Derives `Debug, Serialize`. The open `detail` map is the polymorphism point: each analyzer
populates it with its own keys. No schema is enforced at the type level.

**Key example from TcpReassembler::summarize():** `packets_processed`, `flows_total`,
`bytes_reassembled`, `evictions`, `dropped_findings`, and other counters from
ReassemblyStats.

## E-36: Summary (src/summary.rs:18-38)

```
struct Summary {
    total_packets:  u64,
    total_bytes:    u64,
    skipped_packets:u64,
    hosts:          HashSet<IpAddr>,           // private
    protocols:      HashMap<Protocol, u64>,    // private
    services:       HashMap<String, u64>,      // private
}
```

Derives `Debug, Serialize`. Populated once per decoded packet via `ingest()`.

`services` uses port-based `app_protocol_hint()`. This attribution differs from the
content-first `StreamDispatcher` routing for the same flows (LESSON-P3.01).

`--hosts` flag is wired via `show_hosts_breakdown` on TerminalReporter (P1.03). The hosts
HashSet is now rendered when `--hosts` is passed.

## E-37: Reporter (src/reporter/mod.rs:8-15) [trait]

```rust
pub trait Reporter {
    fn render(&self, summary: &Summary, findings: &[Finding],
              analyzer_summaries: &[AnalysisSummary]) -> String;
}
```

Single immutable-inputs method. ADR 0003: each implementor owns its own escaping.

## E-38: TerminalReporter (src/reporter/terminal.rs:63-75)

```
struct TerminalReporter {
    pub use_color:            bool,
    pub show_mitre_grouping:  bool,
    pub show_hosts_breakdown: bool,
}
```

Sole owner of `escape_for_terminal`. Contains 11 inline `#[test]` functions
(terminal.rs:300-389); `src/analyzer/tls.rs` holds the other 7 inline tests -- total 18
inline across src/ (discovered in pass-0 R2). See CAP-11 for rendering details.

## E-39: JsonReporter (src/reporter/json.rs:21)

Unit struct. `serde_json::to_string_pretty().unwrap()` is infallible by construction
(BC-RPT-001 / BC-2.11.001; confirmed pass-2 R2 Target 8). Statistics maps serialized via BTreeMap for
deterministic key ordering (P2.09 / #76).

## E-39b: CsvReporter (src/reporter/csv.rs)

Unit struct. Implemented P2.03 (#84). Produces 9-column CSV with CSV-injection
neutralization. See CAP-11 for full field encoding specification.
