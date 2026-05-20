---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.11.001
bcs:
  - BC-2.11.001
module: src/reporter/csv.rs
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

# VP-020: CSV Injection Neutralization

## Property Statement

`CsvReporter` neutralizes CSV injection attacks by ensuring that no cell value
in the output causes a spreadsheet application to interpret the cell as a formula:

1. Any cell value starting with `=`, `+`, `-`, or `@` is prefixed with a tab
   character (`\t`) to prevent formula interpretation by Excel, LibreOffice, and
   Google Sheets.
2. The `csv` crate handles field quoting and escaping of commas and double-quotes.
3. The neutralization applies to all string fields derived from attacker-controlled
   data: `summary`, `evidence`, and any `detail` values from `AnalysisSummary`.

This property was added when the CSV reporter was implemented in PR #84.

## Source Contract

- **Primary BC:** BC-2.11.001 -- Reporters produce structured output (covers all reporters including CSV)
- **Postcondition:** No CSV cell starts with a formula-injection character without a tab prefix

Note: The BC index does not have a dedicated CSV-reporter BC (the CsvReporter was
an absent behavior closed by PR #84; BC-ABS-007 was retired). This VP is traced to
BC-2.11.001 as the nearest applicable reporting contract. A future BC sweep may
add a dedicated BC-2.11.020 for CSV-specific properties.

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Unit test | Rust test | N/A | All four injection characters (=, +, -, @) plus safe values |

## Test Specification

```rust
#[test]
fn test_csv_injection_neutralization() {
    let injection_chars = ['=', '+', '-', '@'];

    for ch in injection_chars {
        let malicious_summary = format!("{}malicious_formula(A1)", ch);
        let finding = Finding {
            summary: malicious_summary.clone(),
            ..make_test_finding()
        };

        let mut output = Vec::new();
        let mut reporter = CsvReporter::new(&mut output);
        reporter.report(&[finding], &[], &Summary::default()).unwrap();

        let csv_text = String::from_utf8(output).unwrap();
        // The cell value must not start with the injection character in the CSV
        // (it should be prefixed with tab)
        assert!(!csv_text.contains(&format!(",{}", ch)),
            "injection char '{}' not neutralized in CSV", ch);
        assert!(csv_text.contains(&format!("\t{}", ch))
            || csv_text.contains(&format!("\"\t{}", ch)),
            "injection char '{}' not tab-prefixed", ch);
    }
}

#[test]
fn test_csv_safe_values_unchanged() {
    let safe_values = ["normal text", "192.168.1.1", "GET /path", "200 OK"];
    for value in safe_values {
        let finding = Finding {
            summary: value.to_string(),
            ..make_test_finding()
        };
        let mut output = Vec::new();
        let mut reporter = CsvReporter::new(&mut output);
        reporter.report(&[finding], &[], &Summary::default()).unwrap();
        let csv_text = String::from_utf8(output).unwrap();
        assert!(csv_text.contains(value),
            "safe value '{}' was modified", value);
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite -- 4 injection characters + safe values | |
| Proof complexity | Very low | Simple prefix check |
| Tool support | High | Standard unit test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/csv.rs` -- CSV-injection neutralization logic (PR #84).
The `csv` crate handles field quoting; wirerust adds the tab-prefix guard.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | null | formal-verifier |
| Tests passing | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
