---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.11.021
bcs:
  - BC-2.11.021
module: src/reporter/csv.rs
proof_method: unit
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - date: 2026-05-30
    actor: product-owner
    note: "proof_method label corrected manual→unit to match body table + VP-INDEX + BC-2.11.021 (STORY-079 P1 finding)"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-020: CSV Injection Neutralization

## Property Statement

`CsvReporter` neutralizes CSV injection attacks by ensuring that no cell value
in the output causes a spreadsheet application to interpret the cell as a formula:

1. Any cell value whose first character is one of the six formula-trigger characters
   (`=`, `+`, `-`, `@`, TAB `\t`, CR `\r`) is prefixed with a single quote (`'`)
   to prevent formula interpretation by Excel, LibreOffice, and Google Sheets.
   (A leading TAB would itself be a trigger character; a tab-prefix would re-introduce
   the injection vector it claims to fix. The correct neutralization is a single quote.)
2. The `csv` crate handles field quoting and escaping of commas and double-quotes.
3. The neutralization applies to all string fields derived from attacker-controlled
   data that are actually emitted to CSV cells: per-`Finding` fields `summary`,
   `evidence` (joined), `category`, `verdict`, `confidence`, `mitre_technique`,
   `source_ip`, `direction`, and `timestamp`. The `_analyzer_summaries` parameter
   is accepted by the trait signature but is explicitly ignored by `CsvReporter::render`
   (underscore-prefixed at `csv.rs:56`); no `AnalysisSummary` detail value ever
   reaches a CSV cell, so that field is outside the neutralization scope.

This property was added when the CSV reporter was implemented in PR #84.

## Source Contract

- **Primary BC:** BC-2.11.021 -- CSV-injection neutralization (CsvReporter must prefix formula-trigger characters with a single quote)
- **Postcondition:** No CSV cell starts with a formula-injection character (=, +, -, @, TAB, CR) without a single-quote prefix

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Unit test | Rust test | N/A | All six injection characters (=, +, -, @, TAB, CR) plus safe values |

## Test Specification

```rust
#[test]
fn test_csv_injection_neutralization() {
    // All six formula-trigger characters per neutralize_csv_injection (csv.rs:42).
    // TAB and CR are themselves triggers, so using a tab prefix would re-introduce
    // the injection vector. The correct neutralization is a single-quote prefix.
    let injection_chars = ['=', '+', '-', '@', '\t', '\r'];

    for ch in injection_chars {
        let malicious_summary = format!("{}malicious_formula(A1)", ch);
        let finding = Finding {
            summary: malicious_summary.clone(),
            ..make_test_finding()
        };

        let output = render_csv_with_finding(finding); // helper: returns String

        // The cell must be prefixed with a single quote, not a tab.
        assert!(
            output.contains(&format!("'{}", ch))
                || output.contains(&format!("\"'{}", ch)),
            "injection char {:?} not single-quote-prefixed in CSV output",
            ch
        );
    }
}

#[test]
fn test_csv_safe_values_unchanged() {
    // CsvReporter is a unit struct -- constructed directly, no constructor call.
    // render() returns an owned String; there is no I/O in the reporter itself.
    let reporter = CsvReporter;
    let safe_values = ["normal text", "192.168.1.1", "GET /path", "200 OK"];
    for value in safe_values {
        let finding = Finding {
            summary: value.to_string(),
            ..make_test_finding()
        };
        let summary = Summary::default();
        // render() signature: fn render(&self, summary: &Summary, findings: &[Finding],
        //   analyzer_summaries: &[AnalysisSummary]) -> String  (Reporter trait, mod.rs:27-32)
        let csv_text = reporter.render(&summary, &[finding], &[]);
        assert!(
            csv_text.contains(value),
            "safe value '{}' was modified", value
        );
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite -- 6 injection characters (=, +, -, @, TAB, CR) + safe values | |
| Proof complexity | Very low | Simple prefix check |
| Tool support | High | Standard unit test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/csv.rs` -- `neutralize_csv_injection` function (lines 40-44).
The `csv` crate handles field quoting; wirerust adds the single-quote prefix guard.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| proof_method corrected manual→unit | 2026-05-30 | product-owner |
| Tests committed | null | formal-verifier |
| Tests passing | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
