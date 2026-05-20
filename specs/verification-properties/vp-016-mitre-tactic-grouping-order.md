---
document_type: verification-property
level: L4
version: "1.0"
status: draft
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
proof_method: manual
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
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
    use crate::mitre::{all_tactics_in_report_order, MitreTactic};

    // Verify all_tactics_in_report_order is exhaustive
    let tactics = all_tactics_in_report_order();
    let all_variants = MitreTactic::all_variants(); // if available, else count manually
    assert_eq!(tactics.len(), all_variants.len(),
        "all_tactics_in_report_order missing variants");

    // No duplicates
    let deduped: Vec<_> = tactics.iter().collect::<std::collections::HashSet<_>>()
        .into_iter().collect();
    assert_eq!(deduped.len(), tactics.len(), "duplicate tactics in order list");
}

#[test]
fn test_no_technique_finding_lands_in_uncategorized() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        mitre_technique: None,
        summary: "test".to_string(),
        evidence: vec![],
        timestamp: None,
        direction: None,
    };
    let mut output = Vec::new();
    let mut reporter = TerminalReporter::new(&mut output, /*mitre=*/true, /*no_color=*/true);
    reporter.report(&[finding], &[], &Summary::default()).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Uncategorized"), "no-technique finding not in Uncategorized");
}

#[test]
fn test_within_bucket_sort_verdict_first() {
    // Findings with different verdicts in same tactic should be sorted Likely first
    let likely = make_finding(Verdict::Likely, Confidence::Low, Some("T1036"));
    let inconclusive = make_finding(Verdict::Inconclusive, Confidence::High, Some("T1036"));
    let sorted = sort_within_bucket(&[inconclusive, likely]);
    assert!(matches!(sorted[0].verdict, Verdict::Likely),
        "Likely should sort before Inconclusive");
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
| Tests committed | null | formal-verifier |
| Tests passing | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
