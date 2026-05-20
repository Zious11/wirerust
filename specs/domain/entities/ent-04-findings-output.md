---
artifact: L2-ent-04
traces_to: ../domain-spec.md
title: Entities -- Findings and Output (L3-L4)
status: descriptive (brownfield) -- reconciled against develop HEAD aa2ece9
reconciled: 2026-05-20
---

# Entities: Findings and Output (L3-L4)

Covers E-23 through E-28, E-36 through E-39. Source: pass-2-domain-model.md + pass-3-R4.md.

## E-23: Verdict (src/findings.rs:7-22)

```
enum Verdict { Likely, Unlikely, Inconclusive }
```

`#[non_exhaustive]` (P2.10 / #76). Derives `Debug, Clone, Copy, PartialEq, Eq, Serialize`.
`Display` renders uppercase (`LIKELY`, etc.).
`verdict_rank` order (for sorting): `Likely < Inconclusive < Unlikely` (terminal.rs:223-229).

## E-24: Confidence (src/findings.rs:24-39)

```
enum Confidence { High, Medium, Low }
```

`#[non_exhaustive]` (P2.10 / #76). Derives same as Verdict.
`confidence_rank` order: `High < Medium < Low` (terminal.rs:230-236).

## E-25: ThreatCategory (src/findings.rs:41-57)

```
enum ThreatCategory {
    Reconnaissance, LateralMovement, C2, Exfiltration,
    CredentialAccess, Execution, Persistence, Anomaly
}
```

`#[non_exhaustive]` (P2.10 / #76). `LateralMovement` and `C2` are defined but never emitted
by any analyzer (grep for `::C2,` / `::LateralMovement,` in src/ returns zero hits).
`Display` uses the Debug formatter (variant name verbatim).

**Consistency with MitreTactic:** Both `ThreatCategory` and `MitreTactic` are now
`#[non_exhaustive]` (the prior inconsistency closed by P2.10).

## E-26: Finding (src/findings.rs:59-70)

The critical output type. See CAP-09 for the full schema and all 22 emission sites.

All four Option fields use `skip_serializing_if = "Option::is_none"` (symmetric JSON
serialization; fixed P1.02 / #73). No Option field ever serializes as `null`.
All 22 emission sites set `timestamp: None` (open item O-01).

`direction: Option<Direction>` was added (P2.08 / #77). HTTP and TLS analyzer findings set
it; reassembly-engine findings leave it None.

## E-27: MitreTactic (src/mitre.rs:21-42)

16-variant enum (14 Enterprise + 2 ICS). `#[non_exhaustive]` (VO-5). Derives `Debug, Clone,
Copy, PartialEq, Eq, Hash`. `Display` renders canonical English names. See CAP-10 for full
variant list.

## E-28: AnalysisSummary (src/analyzer/mod.rs:12-17)

```
struct AnalysisSummary {
    analyzer_name:    String,
    packets_analyzed: u64,
    detail:           HashMap<String, serde_json::Value>,
}
```

Derives `Debug, Serialize`. The open `detail` map is the polymorphism point: each analyzer
populates it with its own keys. No schema is enforced at the type level.

**Key example from TcpReassembler::summarize():** `packets_processed`, `flows_total`,
`bytes_reassembled`, `evictions`, `dropped_findings`, and other counters from
ReassemblyStats.

## E-36: Summary (src/summary.rs:8-16)

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

## E-38: TerminalReporter (src/reporter/terminal.rs:48-53)

```
struct TerminalReporter {
    pub use_color:            bool,
    pub show_mitre_grouping:  bool,
    pub show_hosts_breakdown: bool,
}
```

Sole owner of `escape_for_terminal`. Contains the only inline `#[test]` functions in src/
(11 tests at lines 265-341; discovered in pass-0 R2). See CAP-11 for rendering details.

## E-39: JsonReporter (src/reporter/json.rs:8)

Unit struct. `serde_json::to_string_pretty().unwrap()` is infallible by construction
(BC-RPT-007; confirmed pass-2 R2 Target 8). Statistics maps serialized via BTreeMap for
deterministic key ordering (P2.09 / #76).

## E-39b: CsvReporter (src/reporter/csv.rs)

Unit struct. Implemented P2.03 (#84). Produces 9-column CSV with CSV-injection
neutralization. See CAP-11 for full field encoding specification.
