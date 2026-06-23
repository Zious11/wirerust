---
document_type: verification-property
level: L4
version: "2.6"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.11.013
bcs:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.015
  - BC-2.10.003
  - BC-2.10.004
module: src/reporter/terminal.rs
proof_method: integration
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "14019033a379d7fe8646b05e5e170d84a537fc5f9c84e6a667073054456a9fc0"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: proof_method manual→integration to match body table + VP-INDEX (Wave-21 wave-level consistency lens; SS-11 reporter VP family harmonization completion — sibling of VP-017 fix in 86113c2; DF-SIBLING-SWEEP-001)"
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set (tests/reporter_terminal_tests.rs)."
  - "v2.1 (2026-06-12): F-D10-L02 — corrected stale variant count 16 → 17. IcsImpact was added in the DNP3/Feature-8 cycle (src/mitre.rs, STORY-109). Canonical count: 14 Enterprise + 3 ICS-unique (IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact) = 17. Updated test assertion comment and assert_eq value."
  - "v2.2 (2026-06-13, ARP-F2 Pass-14 PO Burst 2): Two stale Finding field references in Test Specification corrected: 'mitre_technique: None' → 'mitre_techniques: vec![]' and 'mitre_technique: technique.map(|s| s.to_string())' → 'mitre_techniques: technique.map(|s| vec![s.to_string()]).unwrap_or_default()'. These were STALE singular field uses; shipped struct is Vec<String> per ADR-006 Decision 13. Lock fields unchanged."
  - "v2.3 (2026-06-14, F3-convergence FIX-4): De-pinned stale line anchor '(mitre.rs:95)' → '(src/mitre.rs `all_tactics_in_report_order`)'. Live src verified: all_tactics_in_report_order at mitre.rs:100 (was off by 5). DF-SIBLING-SWEEP-001: no other stale line pins found in this file."
  - "v2.4: mechanical API-vocabulary update — TerminalReporter test-spec snippets re-expressed in FindingsRender enum vocabulary (show_mitre_grouping: true → render: FindingsRender::Grouped) per STORY-120 / issue #62; no normative change to the property or proof obligation; verification_lock preserved."
  - "v2.5: mechanical API-vocabulary update — FindingsRender enum→struct (STORY-119 / D-110); render: FindingsRender::Grouped → FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }; no normative change; verification_lock preserved."
  - "v2.6 (2026-06-23, F5 ICS tactic-ID correctness fix, D-209): 17→20 variants. Three new ICS MitreTactic variants added (IcsDiscovery TA0102, IcsCollection TA0100, IcsCommandAndControl TA0101). Test assertion comment updated: '14 Enterprise + 3 ICS-unique' → '14 Enterprise + 6 ICS'; assert_eq value 17→20; message text updated to match. DF-SIBLING-SWEEP-001."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-016: MITRE Tactic Grouping Order

## Property Statement

When `--mitre` output mode is active, `TerminalReporter` groups findings by MITRE
tactic and emits tactic headers in the canonical order returned by
`all_tactics_in_report_order()`:

1. Tactic headers appear in the order defined by `MitreTactic::all_tactics_in_report_order`:
   Enterprise tactics first (in kill-chain order), then ICS tactics, then an
   "Uncategorized" bucket last.

2. Findings with no MITRE technique or with an unrecognized technique ID land in
   "Uncategorized" (BC-2.11.015).

3. Within each tactic bucket, findings are sorted by: Verdict (Likely before
   Inconclusive before Unlikely), then Confidence (High before Medium before Low),
   then original emission order as a tiebreaker (BC-2.11.014).

4. `all_tactics_in_report_order()` contains every `MitreTactic` variant exactly
   once (BC-2.10.004).

## Source Contract

- **Primary BC:** BC-2.11.013 -- MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last
- **Postcondition:** Tactic headers appear in kill-chain order from all_tactics_in_report_order
- **Related BC:** BC-2.11.014 -- Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order
- **Related BC:** BC-2.11.015 -- No-Technique or Unknown-ID Findings Land in Uncategorized
- **Related BC:** BC-2.10.003 -- all_tactics_in_report_order Returns Kill-Chain Order First Then ICS
- **Related BC:** BC-2.10.004 -- all_tactics_in_report_order Contains Every Variant Exactly Once

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test | Rust test + assert | N/A | Fixed-input finding sets with known expected output order |

This is "test sufficient" -- the tactic ordering is derived from a simple
`Vec` returned by a pure function. A deterministic test with known findings
exercises the full grouping and sorting logic without formal verification overhead.

## Test Specification

```rust
#[test]
fn test_mitre_grouping_order_canonical() {
    use crate::mitre::all_tactics_in_report_order;

    // all_tactics_in_report_order returns a &'static [MitreTactic] (src/mitre.rs `all_tactics_in_report_order`).
    // MitreTactic has no all_variants() method; count the variants manually:
    // 14 Enterprise + 6 ICS (IcsInhibitResponseFunction, IcsImpairProcessControl,
    // IcsImpact, IcsDiscovery, IcsCollection, IcsCommandAndControl) = 20 total
    // (mitre.rs enum; IcsImpact added STORY-109 DNP3/Feature-8 cycle; IcsDiscovery/
    // IcsCollection/IcsCommandAndControl added F5 ICS tactic-ID correctness fix D-209).
    let tactics = all_tactics_in_report_order();
    assert_eq!(tactics.len(), 20,
        "all_tactics_in_report_order must list all 20 MitreTactic variants (14 Enterprise + 6 ICS)");

    // No duplicates
    let unique: std::collections::HashSet<_> = tactics.iter().collect();
    assert_eq!(unique.len(), tactics.len(), "duplicate tactics in order list");
}

#[test]
fn test_no_technique_finding_lands_in_uncategorized() {
    use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use crate::reporter::Reporter;
    use crate::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
    use crate::summary::Summary;

    // TerminalReporter is a plain struct with public fields; no new() constructor
    // (terminal.rs:63-75). Construct directly.
    let reporter = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded },
    };
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        mitre_techniques: vec![],
        summary: "test".to_string(),
        evidence: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    // Reporter trait method is render(), not report() (reporter/mod.rs:27-32).
    let text = reporter.render(&Summary::default(), &[finding], &[]);
    assert!(text.contains("Uncategorized"), "no-technique finding not in Uncategorized");
}

#[test]
fn test_within_bucket_sort_verdict_first() {
    use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use crate::reporter::Reporter;
    use crate::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
    use crate::summary::Summary;

    // sort_within_bucket is an internal detail of render_findings_grouped;
    // it is not a public function. Verify the sort property end-to-end via
    // render(), checking that Likely appears before Inconclusive in the output.
    let reporter = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded },
    };
    fn make_finding(verdict: Verdict, confidence: Confidence, technique: Option<&str>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict,
            confidence,
            summary: format!("{verdict:?}"),
            evidence: vec![],
            mitre_techniques: technique.map(|s| vec![s.to_string()]).unwrap_or_default(),
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }
    // Both findings map to DefenseEvasion (T1036 -> MitreTactic::DefenseEvasion).
    let inconclusive = make_finding(Verdict::Inconclusive, Confidence::High, Some("T1036"));
    let likely      = make_finding(Verdict::Likely,       Confidence::Low,  Some("T1036"));
    let text = reporter.render(&Summary::default(), &[inconclusive, likely], &[]);
    // "Likely" in the summary line must appear before "Inconclusive"
    let pos_likely      = text.find("Likely").unwrap_or(usize::MAX);
    let pos_inconclusive = text.find("Inconclusive").unwrap_or(usize::MAX);
    assert!(pos_likely < pos_inconclusive,
        "Likely should sort before Inconclusive within tactic bucket");
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite | Fixed finding sets with known expected order |
| Proof complexity | Very low | Pure list sorting and output rendering |
| Tool support | High | Standard Rust integration test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/terminal.rs` -- MITRE grouping logic in `report()` method.
`src/mitre.rs` -- `all_tactics_in_report_order()`.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | 2026-06-02 | formal-verifier |
| Tests passing | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
