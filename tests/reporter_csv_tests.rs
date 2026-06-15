//! STORY-079: CsvReporter formalization tests — Wave 21
//!
//! Formalizes 13 tests for BC-2.11.020 through BC-2.11.022 (AC-001..AC-013).
//!
//! Behavioral contracts covered:
//!   BC-2.11.020  CsvReporter Emits Exactly Nine Columns in Fixed Header Order
//!   BC-2.11.021  CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote
//!   BC-2.11.022  CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: strict
//! All tests are expected to PASS at the Green Gate because the production
//! implementation already satisfies all ACs. Any FAIL indicates a real gap.
//!
//! NOTE: `neutralize_csv_injection` is a private function in src/reporter/csv.rs
//! and cannot be called directly from integration tests. All AC-005..AC-009
//! tests that target its behavior exercise it indirectly through
//! `CsvReporter::render`, using controlled Finding inputs and asserting on the
//! rendered String. AC-005..AC-008 exercise it via the `summary` column (column 4)
//! which carries the test value verbatim; AC-009 exercises all 9 columns.
//!
//! Column counting uses the `csv` crate's reader to parse rows so that RFC 4180
//! double-quoting of comma-containing fields does NOT inflate the count — a raw
//! string split on ',' would give wrong counts for quoted fields.
//!
//! VP-020 determination: VP-020 proof method is "unit" (not proptest / symbolic
//! execution). All six trigger characters are tested parametrically in
//! test_BC_2_11_021_neutralize_all_six_trigger_chars. This is an IN-STORY
//! property assertion, NOT deferred to Phase-6 formal hardening. The VP file
//! lists proof_completed_date as null and verification_lock as false, but the
//! BC-2.11.021 Verification Properties table explicitly cites VP-020 with
//! proof_method "unit: parametric test over trigger set" — meaning the unit
//! tests here satisfy the VP-020 proof requirement. Phase-6 formal hardening
//! would only apply if the proof method were "symbolic execution" or "proptest"
//! with an unbounded input space, which is not the case here.

// PG-W17-001 / DF-AC-TEST-NAME-SYNC-001 v2 mandates that test fn names EXACTLY
// match the AC `**Test:**` citations. These names use upper-case BC identifiers
// which Rust flags as non-snake-case. Suppress the lint for this file rather
// than diverge from the required naming scheme.
#![allow(non_snake_case)]

use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::Reporter;
use wirerust::reporter::csv::CsvReporter;
use wirerust::summary::Summary;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal Finding with no optional fields set.
///
/// `category` = Anomaly, `verdict` = Likely, `confidence` = High.
/// These render as "Anomaly", "LIKELY", "HIGH" — all non-trigger chars,
/// no injection prefix needed.
fn make_finding(summary: impl Into<String>) -> Finding {
    Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: summary.into(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    }
}

/// Render a single finding through CsvReporter with a default empty Summary.
fn render_one(finding: Finding) -> String {
    CsvReporter.render(&Summary::new(), &[finding], &[])
}

/// Render a slice of findings through CsvReporter.
fn render_many(findings: &[Finding]) -> String {
    CsvReporter.render(&Summary::new(), findings, &[])
}

/// Parse CSV text with the csv crate and return all rows as Vec<Vec<String>>.
/// Includes the header row as the first element.
/// Panics with the full CSV output on parse failure.
fn parse_csv(csv_text: &str) -> Vec<Vec<String>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(false)
        .from_reader(csv_text.as_bytes());
    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap_or_else(|e| {
            panic!("CSV parse failed: {e}\nRaw CSV was:\n{csv_text}");
        });
        rows.push(record.iter().map(|f| f.to_string()).collect());
    }
    rows
}

// Per DF-TEST-NAMESPACE-001: all STORY-079 tests are grouped inside a
// dedicated `mod story_079` wrapper to prevent test-function name collisions
// with other stories' BC-prefixed names (e.g. STORY-076 or STORY-077 which
// also use BC-2.11.XXX naming).
mod story_079 {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-2.11.020 — Fixed 9-column schema, header, RFC 4180 quoting
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.020 postcondition 1): The first line of CsvReporter::render
    /// output is the header row with exactly the nine expected field names in order,
    /// followed by a line terminator.
    ///
    /// BC-2.11.020 v1.3 pc1 confirms the LF (`\n`) line terminator: `csv::WriterBuilder::new()`
    /// uses LF by default; CRLF is intentionally not configured (RFC 4180 readers — including
    /// the `csv` crate reader — accept LF as a valid record terminator). This test asserts
    /// the confirmed LF behavior.
    ///
    /// Discriminating assertions:
    ///   - Positive: first logical CSV row (parsed) has exactly these 9 field values
    ///     in order: category, verdict, confidence, summary, evidence,
    ///     mitre_technique, source_ip, direction, timestamp.
    ///   - Positive: the raw output contains the exact header substring followed by \n
    ///     (LF — the csv crate WriterBuilder default).
    ///   - Negative: output does NOT start with a data row (no "Anomaly" in first line).
    #[test]
    fn test_BC_2_11_020_header_row_first_and_exact() {
        // BC-2.11.020 pc1: header is first line, exact 9 fields.
        let csv_text = render_many(&[]);

        // Parse the CSV: with zero findings the output is the header only.
        let rows = parse_csv(&csv_text);
        assert_eq!(
            rows.len(),
            1,
            "BC-2.11.020 pc1: empty findings → exactly 1 row (header); got {} rows\n{}",
            rows.len(),
            csv_text
        );

        // Exact field-by-field match for the header row.
        let header = &rows[0];
        let expected: Vec<&str> = vec![
            "category",
            "verdict",
            "confidence",
            "summary",
            "evidence",
            "mitre_techniques",
            "source_ip",
            "direction",
            "timestamp",
        ];
        assert_eq!(
            header.len(),
            9,
            "BC-2.11.020 inv1: header must have exactly 9 fields; got {}; header={header:?}",
            header.len()
        );
        for (i, (got, exp)) in header.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got.as_str(),
                *exp,
                "BC-2.11.020 pc3: header column {i} must be '{exp}'; got '{got}'"
            );
        }

        // Line terminator: BC-2.11.020 v1.3 pc1 confirms LF (\n) — csv::WriterBuilder::new()
        // default; CRLF intentionally not configured.
        let expected_header_line = "category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp\n";
        assert!(
            csv_text.starts_with(expected_header_line),
            "BC-2.11.020 v1.3 pc1: raw output must begin with the LF-terminated header line;\n\
             expected prefix: {expected_header_line:?}\n\
             actual prefix: {:?}",
            &csv_text[..csv_text.len().min(120)]
        );

        // Confirm the header line contains the full column name sequence.
        let header_fields = "category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp";
        assert!(
            csv_text.starts_with(header_fields),
            "BC-2.11.020 pc1: output must start with the exact column name sequence; \
             got: {:?}",
            &csv_text[..csv_text.len().min(120)]
        );

        // Negative: a data row (containing "Anomaly") must NOT be the first line.
        let first_line = csv_text.lines().next().unwrap_or("");
        assert!(
            !first_line.contains("Anomaly"),
            "BC-2.11.020 pc1: first line must be the header row, not a data row; \
             got: {first_line:?}"
        );
    }

    /// AC-002 (BC-2.11.020 postcondition 2): Every data row contains exactly 9
    /// comma-separated fields in the same column order as the header.
    ///
    /// Discriminating assertions:
    ///   - Header row: 9 fields.
    ///   - Three data rows: each exactly 9 fields.
    ///   - Column values in header and data rows match the spec order (spot-check
    ///     columns 1-3 of first data row against known Finding values).
    #[test]
    fn test_BC_2_11_020_every_row_has_nine_columns() {
        // BC-2.11.020 pc2 + inv1: parse every row; each must have exactly 9 fields.
        // Use 3 findings so the loop in render() is exercised multiple times.
        let findings = vec![
            make_finding("summary one"),
            make_finding("summary two"),
            make_finding("summary three"),
        ];
        let csv_text = render_many(&findings);
        let rows = parse_csv(&csv_text);

        // Header + 3 data rows = 4 total.
        assert_eq!(
            rows.len(),
            4,
            "BC-2.11.020: expected 4 rows (1 header + 3 data); got {};\n{}",
            rows.len(),
            csv_text
        );

        // Every row — header and data — must have exactly 9 fields.
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(
                row.len(),
                9,
                "BC-2.11.020 inv1: row {i} must have exactly 9 fields; got {}; row={row:?}",
                row.len()
            );
        }

        // Spot-check: columns 1-3 of the first data row (row index 1) must match
        // the Finding we passed in (Anomaly, LIKELY, HIGH).
        let data_row = &rows[1];
        assert_eq!(
            data_row[0], "Anomaly",
            "BC-2.11.020 pc3: column 1 (category) must be 'Anomaly'; got '{}'",
            data_row[0]
        );
        assert_eq!(
            data_row[1], "LIKELY",
            "BC-2.11.020 pc3: column 2 (verdict) must be 'LIKELY'; got '{}'",
            data_row[1]
        );
        assert_eq!(
            data_row[2], "HIGH",
            "BC-2.11.020 pc3: column 3 (confidence) must be 'HIGH'; got '{}'",
            data_row[2]
        );
        // Column 4 (summary) must be our test string.
        assert_eq!(
            data_row[3], "summary one",
            "BC-2.11.020 pc3: column 4 (summary) must be 'summary one'; got '{}'",
            data_row[3]
        );
    }

    /// AC-003 (BC-2.11.020 invariant 1): Column count is exactly 9 even when a
    /// field value contains a comma. The csv crate wraps the field in double-quotes
    /// per RFC 4180, but parsing via the csv crate recovers the original value and
    /// the column count stays exactly 9.
    ///
    /// Discriminating assertions:
    ///   - Data row parsed by csv crate: exactly 9 fields (not 10+).
    ///   - The comma-containing field value is recovered correctly.
    ///   - The raw CSV output contains a double-quoted field (RFC 4180 proof).
    #[test]
    fn test_BC_2_11_020_comma_in_field_does_not_change_column_count() {
        // BC-2.11.020 inv1 / EC-004: a summary containing a comma must be
        // RFC 4180 quoted by the csv crate; the parsed column count must still be 9.
        // Canonical BC test vector: field value "a,b" → quoted in CSV output.
        let finding = make_finding("attack,payload,here");
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        // Must be 2 rows: header + 1 data row.
        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.020 inv1: expected 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        // Data row must have exactly 9 fields despite the commas in the summary.
        let data_row = &rows[1];
        assert_eq!(
            data_row.len(),
            9,
            "BC-2.11.020 inv1: comma-containing field must not inflate column count; \
             got {} columns; data_row={data_row:?}",
            data_row.len()
        );

        // The summary field (column 4) must be recovered as the original value
        // (comma preserved, no spurious splitting).
        assert_eq!(
            data_row[3], "attack,payload,here",
            "BC-2.11.020 inv1: csv parser must recover comma-containing summary \
             exactly; got '{}'",
            data_row[3]
        );

        // Raw CSV must contain a double-quoted field (RFC 4180 evidence).
        // The csv crate emits "attack,payload,here" in the raw bytes.
        assert!(
            csv_text.contains("\"attack,payload,here\""),
            "BC-2.11.020 pc4: comma-containing field must be double-quoted in raw CSV; \
             raw output:\n{csv_text}"
        );
    }

    /// AC-004 (BC-2.11.020 postcondition 4): The output is valid RFC 4180 CSV.
    /// A field containing commas appears double-quoted. A field containing
    /// double-quotes uses `""` escaping per RFC 4180.
    ///
    /// Discriminating assertions:
    ///   - Raw CSV contains double-quoted field when value has commas.
    ///   - Raw CSV contains `""` when value has a literal double-quote.
    ///   - Parsed value (after csv crate unescaping) recovers the original string.
    #[test]
    fn test_BC_2_11_020_rfc4180_quoting() {
        // BC-2.11.020 pc4 / EC-005: canonical BC test vector: field value contains `"`.
        // RFC 4180: a field containing a double-quote is wrapped in double-quotes
        // and the inner quote is doubled: abc"def → "abc""def"
        let finding = make_finding("has a \"quote\" inside");
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        let data_row = &rows[1];
        assert_eq!(
            data_row.len(),
            9,
            "BC-2.11.020 inv1: double-quote-containing field must not change column count; \
             got {}; row={data_row:?}",
            data_row.len()
        );

        // The parsed value must recover the original string exactly.
        assert_eq!(
            data_row[3], "has a \"quote\" inside",
            "BC-2.11.020 pc4: csv crate must recover double-quote-containing field exactly; \
             got '{}'",
            data_row[3]
        );

        // The raw CSV must use `""` escaping for the inner double-quotes.
        // RFC 4180: "has a ""quote"" inside" in the raw bytes.
        assert!(
            csv_text.contains("\"\""),
            "BC-2.11.020 pc4: raw CSV must use RFC 4180 `\"\"` escaping for double-quotes; \
             raw output:\n{csv_text}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.021 — CSV-injection neutralization
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.11.021 postcondition 1): `neutralize_csv_injection` prepends
    /// `'` to any cell value whose first character is `=`, `+`, `-`, `@`, TAB
    /// (U+0009), or CR (U+000D). All six trigger chars are tested.
    ///
    /// Exercised via the `summary` column (column 4, index 3) which carries the
    /// test value verbatim (no other transformation applied to that field).
    ///
    /// Discriminating assertions for each of the 6 trigger chars:
    ///   - Positive: parsed summary cell starts with `'` followed by the trigger char.
    ///   - Negative: the value does NOT appear without the `'` prefix in that column.
    #[test]
    fn test_BC_2_11_021_neutralize_all_six_trigger_chars() {
        // BC-2.11.021 pc1 / VP-020: canonical test vectors from the BC.
        // All six trigger characters must receive the single-quote prefix.
        let trigger_cases: &[(&str, &str)] = &[
            ("=SUM(A1:A2)", "'=SUM(A1:A2)"),
            ("+payload", "'+payload"),
            ("-1", "'-1"),
            ("@admin", "'@admin"),
            ("\tindented", "'\tindented"),
            ("\rcarriage", "'\rcarriage"),
        ];

        for (input, expected_cell) in trigger_cases {
            let finding = make_finding(*input);
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);

            assert_eq!(
                rows.len(),
                2,
                "BC-2.11.021 pc1: trigger={input:?} — expected 2 rows; got {};\n{}",
                rows.len(),
                csv_text
            );

            let summary_cell = &rows[1][3]; // column 4 (index 3) = summary
            assert_eq!(
                summary_cell.as_str(),
                *expected_cell,
                "BC-2.11.021 pc1: trigger char at start of input {:?} must be \
                 neutralized with leading quote; expected cell {:?}, got {:?}",
                input,
                expected_cell,
                summary_cell
            );

            // Negative: the cell must NOT equal the raw input (no prefix would be a bug).
            assert_ne!(
                summary_cell.as_str(),
                *input,
                "BC-2.11.021 pc1: neutralized cell must differ from raw input {:?}; \
                 got unchanged value {:?}",
                input,
                summary_cell
            );
        }
    }

    /// AC-006 (BC-2.11.021 postcondition 2): `neutralize_csv_injection` returns
    /// the input unchanged when the first character is not in the trigger set.
    ///
    /// Discriminating assertions:
    ///   - Positive: cell value equals the input exactly (no prefix added).
    ///   - Negative: no spurious `'` prefix present for non-trigger input.
    #[test]
    fn test_BC_2_11_021_no_trigger_no_change() {
        // BC-2.11.021 pc2 / BC EC-009/EC-010: canonical BC test vector:
        //   "normal text" → "normal text" (unchanged).
        //   Also tests: digits, letters, printable punctuation (not a trigger).
        let safe_cases: &[&str] = &[
            "normal text",
            "192.168.1.1",
            "'already-prefixed",
            "100",
            "A-leading-letter",
            "!bang",
            "#hashtag",
        ];

        for input in safe_cases {
            let finding = make_finding(*input);
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);

            let summary_cell = &rows[1][3];
            assert_eq!(
                summary_cell.as_str(),
                *input,
                "BC-2.11.021 pc2: non-trigger input {:?} must be unchanged; got {:?}",
                input,
                summary_cell
            );

            // Negative: no spurious quote prefix on a non-trigger value.
            if !input.starts_with('\'') {
                assert!(
                    !summary_cell.starts_with('\''),
                    "BC-2.11.021 pc2: non-trigger input {:?} must NOT receive a quote prefix; \
                     got {:?}",
                    input,
                    summary_cell
                );
            }
        }
    }

    /// AC-007 (BC-2.11.021 postcondition 4): An empty string input to
    /// `neutralize_csv_injection` is returned unchanged (no prefix added, no crash).
    ///
    /// Discriminating assertions:
    ///   - Positive: the summary column is exactly empty string "".
    ///   - Negative: no single-quote prefix on the empty cell.
    ///   - Positive: the row still has exactly 9 columns (invariant holds for empty fields).
    #[test]
    fn test_BC_2_11_021_empty_string_unchanged() {
        // BC-2.11.021 pc4 / EC-008: canonical BC test vector: "" → "" (unchanged).
        // `chars().next()` returns None for an empty string — no match, no prefix.
        let finding = make_finding("");
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.021 pc4: expected 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        // Row must still have 9 columns.
        let data_row = &rows[1];
        assert_eq!(
            data_row.len(),
            9,
            "BC-2.11.020 inv1: empty-summary row must still have 9 columns; \
             got {}; row={data_row:?}",
            data_row.len()
        );

        // Summary cell (column 4) must be exactly "".
        let summary_cell = &data_row[3];
        assert_eq!(
            summary_cell.as_str(),
            "",
            "BC-2.11.021 pc4: empty string must be returned unchanged; got {:?}",
            summary_cell
        );

        // Negative: no single-quote prefix on empty cell.
        assert!(
            !summary_cell.starts_with('\''),
            "BC-2.11.021 pc4: empty string must NOT receive a quote prefix; \
             got {:?}",
            summary_cell
        );
    }

    /// AC-008 (BC-2.11.021 invariant 2): Only the FIRST character is inspected;
    /// a trigger character at position 2+ (not the first char) does NOT cause a
    /// prefix to be added.
    ///
    /// Discriminating assertions:
    ///   - Positive: cell value equals the input exactly (no prefix).
    ///   - Negative: no single-quote prefix when trigger is at position 2+.
    #[test]
    fn test_BC_2_11_021_trigger_at_position_2_no_prefix() {
        // BC-2.11.021 inv2 / EC-011: canonical BC test vector: "a=formula" → "a=formula".
        // Trigger character `=` at index 1 — must NOT cause a prefix.
        let trigger_at_pos2_cases: &[&str] = &[
            "a=formula",   // = at position 1
            "b+inject",    // + at position 1
            "c-dash",      // - at position 1
            "d@user",      // @ at position 1
            "e\ttab",      // TAB at position 1
            "f\rcarriage", // CR at position 1
        ];

        for input in trigger_at_pos2_cases {
            let finding = make_finding(*input);
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);

            let summary_cell = &rows[1][3];
            assert_eq!(
                summary_cell.as_str(),
                *input,
                "BC-2.11.021 inv2: trigger at position 2+ must not cause prefix; \
                 input={input:?}, got cell={summary_cell:?}"
            );

            assert!(
                !summary_cell.starts_with('\''),
                "BC-2.11.021 inv2: no quote prefix when trigger char is at position 2+; \
                 input={input:?}, got cell={summary_cell:?}"
            );
        }
    }

    /// AC-009 (BC-2.11.021 invariant 1): `neutralize_csv_injection` is applied to
    /// ALL nine column values for every data row without exception.
    ///
    /// Strategy: construct a Finding where each field that maps to a CSV column
    /// starts with a trigger character. Verify that the corresponding parsed CSV
    /// cell is prefixed with `'` for all columns where that is possible given the
    /// field types.
    ///
    /// Columns exercised:
    ///   col 1 (category): enum Display — "Anomaly" starts with 'A', non-trigger.
    ///   col 2 (verdict): enum Display — "LIKELY" starts with 'L', non-trigger.
    ///   col 3 (confidence): enum Display — "HIGH" starts with 'H', non-trigger.
    ///   col 4 (summary): string — set to "=trigger" → must become "'=trigger".
    ///   col 5 (evidence joined): string — set to "=ev1" → must become "'=ev1".
    ///   col 6 (mitre_technique): Option<String> — set to "=T1234" → must become "'=T1234".
    ///   col 7 (source_ip): None → empty string, non-trigger.
    ///   col 8 (direction): None → empty string, non-trigger.
    ///   col 9 (timestamp): None → empty string, non-trigger.
    ///
    /// Discriminating assertions: columns 4, 5, and 6 receive the `'` prefix.
    #[test]
    fn test_BC_2_11_021_applied_to_all_nine_columns() {
        // BC-2.11.021 inv1: neutralization applied to ALL columns.
        // We control the string-derived columns (4, 5, 6) with trigger-char values
        // and verify each gets the prefix. The enum-derived columns (1-3) and
        // None-derived columns (7-9) produce non-trigger strings so they pass through.
        let finding = Finding {
            category: ThreatCategory::Anomaly, // col 1: "Anomaly" — non-trigger
            verdict: Verdict::Likely,          // col 2: "LIKELY" — non-trigger
            confidence: Confidence::High,      // col 3: "HIGH" — non-trigger
            summary: "=trigger_summary".to_string(), // col 4: trigger '='
            evidence: vec!["=trigger_evidence".to_string()], // col 5: joined → "=trigger_evidence"
            mitre_techniques: vec!["=T1234".to_string()], // col 6: trigger '='
            source_ip: None,                   // col 7: "" — non-trigger
            direction: None,                   // col 8: "" — non-trigger
            timestamp: None,                   // col 9: "" — non-trigger
        };

        let csv_text = CsvReporter.render(&Summary::new(), &[finding], &[]);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.021 inv1: expected 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        let data_row = &rows[1];
        assert_eq!(
            data_row.len(),
            9,
            "BC-2.11.020 inv1: row must have 9 columns; got {}",
            data_row.len()
        );

        // col 1 (index 0): "Anomaly" — no trigger, must be unchanged.
        assert_eq!(
            data_row[0], "Anomaly",
            "BC-2.11.021 inv1: col 1 (category) must be 'Anomaly'; got '{}'",
            data_row[0]
        );

        // col 2 (index 1): "LIKELY" — no trigger.
        assert_eq!(
            data_row[1], "LIKELY",
            "BC-2.11.021 inv1: col 2 (verdict) must be 'LIKELY'; got '{}'",
            data_row[1]
        );

        // col 3 (index 2): "HIGH" — no trigger.
        assert_eq!(
            data_row[2], "HIGH",
            "BC-2.11.021 inv1: col 3 (confidence) must be 'HIGH'; got '{}'",
            data_row[2]
        );

        // col 4 (index 3): "=trigger_summary" → must be "'=trigger_summary" (neutralized).
        assert_eq!(
            data_row[3], "'=trigger_summary",
            "BC-2.11.021 inv1: col 4 (summary) must be neutralized; \
             expected \"'=trigger_summary\", got {:?}",
            data_row[3]
        );

        // col 5 (index 4): joined evidence "=trigger_evidence" → must be "'=trigger_evidence".
        assert_eq!(
            data_row[4], "'=trigger_evidence",
            "BC-2.11.021 inv1: col 5 (evidence) must be neutralized after join; \
             expected \"'=trigger_evidence\", got {:?}",
            data_row[4]
        );

        // col 6 (index 5): "=T1234" → must be "'=T1234".
        assert_eq!(
            data_row[5], "'=T1234",
            "BC-2.11.021 inv1: col 6 (mitre_technique) must be neutralized; \
             expected \"'=T1234\", got {:?}",
            data_row[5]
        );

        // col 7 (index 6): None → "" — non-trigger, must remain empty.
        assert_eq!(
            data_row[6], "",
            "BC-2.11.021 inv1: col 7 (source_ip) with None must be empty string; got {:?}",
            data_row[6]
        );

        // col 8 (index 7): None → "" — non-trigger.
        assert_eq!(
            data_row[7], "",
            "BC-2.11.021 inv1: col 8 (direction) with None must be empty string; got {:?}",
            data_row[7]
        );

        // col 9 (index 8): None → "" — non-trigger.
        assert_eq!(
            data_row[8], "",
            "BC-2.11.021 inv1: col 9 (timestamp) with None must be empty string; got {:?}",
            data_row[8]
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.022 — Evidence Vec join with "; "
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.11.022 postcondition 1): For a Finding with
    /// `evidence = ["first", "second", "third"]`, the evidence CSV cell value
    /// is `"first; second; third"` (joined with `"; "` — semicolon then space).
    ///
    /// Discriminating assertions:
    ///   - Positive: parsed evidence cell (column 5) equals "first; second; third".
    ///   - Positive: separator is exactly "; " (two chars: semicolon + space).
    ///   - Negative: separator is NOT ";" alone (no space) or "; " with extra space.
    #[test]
    fn test_BC_2_11_022_evidence_joined_with_semicolon_space() {
        // BC-2.11.022 pc1 / inv1: test vector derived from BC-2.11.022 pc1/inv1
        // (3-element join → 2 separators; exercises the general N-element case).
        let mut finding = make_finding("summary");
        finding.evidence = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        let evidence_cell = &rows[1][4]; // column 5 = evidence (index 4)
        assert_eq!(
            evidence_cell.as_str(),
            "first; second; third",
            "BC-2.11.022 pc1: evidence join must use '; ' separator; \
             expected 'first; second; third', got {:?}",
            evidence_cell
        );

        // Positive: verify the separator is exactly "; " not ";" (no space).
        // "first; second" contains the two-char separator "; ".
        assert!(
            evidence_cell.contains("; "),
            "BC-2.11.022 inv1: separator must be '; ' (semicolon + space); \
             got: {evidence_cell:?}"
        );

        // Negative: separator must NOT be just ";" without a space following.
        // Split by "; " should give exactly 3 elements.
        let parts: Vec<&str> = evidence_cell.split("; ").collect();
        assert_eq!(
            parts.len(),
            3,
            "BC-2.11.022 pc1: split by '; ' must yield exactly 3 parts; \
             got {}: {parts:?}",
            parts.len()
        );
    }

    /// AC-011 (BC-2.11.022 postcondition 2): For a Finding with `evidence = []`
    /// (empty), the evidence cell is an empty string `""`.
    ///
    /// Discriminating assertions:
    ///   - Positive: parsed evidence cell is exactly "".
    ///   - Positive: row still has 9 columns (no column omission for empty field).
    ///   - Negative: evidence cell is NOT absent (null/missing column).
    #[test]
    fn test_BC_2_11_022_empty_evidence_is_empty_cell() {
        // BC-2.11.022 pc2 / EC-001: empty Vec → empty cell (join of empty = "").
        let finding = make_finding("summary");
        // evidence defaults to vec![] from make_finding.
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.022 pc2: expected 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        let data_row = &rows[1];
        assert_eq!(
            data_row.len(),
            9,
            "BC-2.11.020 inv1: row must have 9 columns; got {}; row={data_row:?}",
            data_row.len()
        );

        let evidence_cell = &data_row[4]; // column 5 = evidence
        assert_eq!(
            evidence_cell.as_str(),
            "",
            "BC-2.11.022 pc2: empty evidence Vec must produce empty cell; \
             got {:?}",
            evidence_cell
        );
    }

    /// AC-012 (BC-2.11.022 postcondition 3): For a Finding with
    /// `evidence = ["single item"]`, the evidence cell is `"single item"` with
    /// no separator.
    ///
    /// Discriminating assertions:
    ///   - Positive: parsed evidence cell equals "single item" exactly.
    ///   - Negative: no "; " separator in the cell.
    ///   - Negative: no leading or trailing "; " (no sentinel added for single element).
    #[test]
    fn test_BC_2_11_022_single_evidence_no_separator() {
        // BC-2.11.022 pc3 / EC-002: single element → cell value = that element, no separator.
        let mut finding = make_finding("summary");
        finding.evidence = vec!["single item".to_string()];
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        let evidence_cell = &rows[1][4];
        assert_eq!(
            evidence_cell.as_str(),
            "single item",
            "BC-2.11.022 pc3: single evidence element must appear with no separator; \
             got {:?}",
            evidence_cell
        );

        // Negative: no "; " separator should appear in the cell.
        assert!(
            !evidence_cell.contains("; "),
            "BC-2.11.022 pc3: single-element evidence must contain no '; ' separator; \
             got: {evidence_cell:?}"
        );
    }

    /// AC-013 (BC-2.11.022 postcondition 4): The joined evidence string is
    /// subsequently processed by `neutralize_csv_injection`. If the first element
    /// starts with `=`, the entire joined string (starting with `=`) receives a
    /// `'` prefix.
    ///
    /// The join happens BEFORE neutralization; the prefix is applied to the
    /// joined result (not the individual elements).
    ///
    /// Discriminating assertions:
    ///   - Positive: evidence cell starts with `'=` (prefix on joined string).
    ///   - Positive: the rest of the cell (after `'`) equals the joined string.
    ///   - Negative: the second evidence element is NOT separately prefixed.
    #[test]
    fn test_BC_2_11_022_evidence_join_then_neutralize() {
        // BC-2.11.022 pc4 / EC-006: canonical BC test vector:
        //   evidence=["=formula", "other"] → joined "=formula; other"
        //   → neutralized "'=formula; other".
        let mut finding = make_finding("summary");
        finding.evidence = vec!["=formula".to_string(), "other".to_string()];
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.022 pc4: expected 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        let evidence_cell = &rows[1][4];

        // Positive: the prefix `'` is applied to the JOINED string.
        assert_eq!(
            evidence_cell.as_str(),
            "'=formula; other",
            "BC-2.11.022 pc4: join-then-neutralize — joined '=formula; other' must be \
             neutralized to \"'=formula; other\"; got {:?}",
            evidence_cell
        );

        // Verify join ordering: the cell starts with `'=` (prefix on the joined string).
        assert!(
            evidence_cell.starts_with("'="),
            "BC-2.11.022 pc4: evidence cell must start with \"'=\" after join+neutralize; \
             got: {evidence_cell:?}"
        );

        // Negative: the second element "other" does NOT get its own prefix.
        // The string after the '; ' separator must be "other" (not "'other").
        let after_separator = evidence_cell.split("; ").nth(1).unwrap_or("");
        assert_eq!(
            after_separator, "other",
            "BC-2.11.022 pc4: second evidence element must NOT be separately neutralized; \
             expected 'other' after '; ' separator, got {:?}",
            after_separator
        );
    }
}

// Per DF-TEST-NAMESPACE-001: all STORY-080 tests are grouped inside a
// dedicated `mod story_080` wrapper to prevent test-function name collisions
// with story_079's BC-2.11.020..022 names.
//
// STORY-080 formalizes BC-2.11.023 (Reporter trait compliance, row count, ignored
// parameters) and BC-2.11.024 (None optional field encoding, Direction Debug format).
//
// implementation_strategy: brownfield-formalization
// tdd_mode: strict
// RED GATE stub phase completed: all 12 stubs confirmed FAIL (see red-gate-log.md).
// This block contains the final discriminating assertions against the existing
// CsvReporter implementation.
mod story_080 {
    use super::*;
    use std::net::IpAddr;

    use chrono::{TimeZone, Utc};
    use wirerust::analyzer::AnalysisSummary;
    use wirerust::reassembly::handler::Direction;

    // -----------------------------------------------------------------------
    // Helpers local to story_080
    // -----------------------------------------------------------------------

    /// Build a minimal AnalysisSummary for use in analyzer_summaries arguments.
    fn make_analysis_summary(name: &str) -> AnalysisSummary {
        AnalysisSummary {
            analyzer_name: name.to_string(),
            packets_analyzed: 42,
            detail: std::collections::BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // BC-2.11.023 — Reporter trait compliance, row count, ignored parameters
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.023 postcondition 1, invariant 1):
    /// Total row count in `CsvReporter::render` output is exactly `1 + findings.len()`
    /// for multiple finding-slice sizes: 0, 1, 3.
    ///
    /// BC-2.11.023 pc1 / inv1: "contains exactly one header row followed by exactly
    /// findings.len() data rows".
    ///
    /// Discriminating assertions:
    ///   - Empty slice → 1 row (header only).
    ///   - Single finding → 2 rows.
    ///   - Three findings → 4 rows.
    ///   - Counting via csv-crate parse (not naive split) so RFC 4180 quoting
    ///     does not inflate the count.
    #[test]
    fn test_BC_2_11_023_row_count_equals_one_plus_findings_len() {
        // BC-2.11.023 pc1 / inv1 canonical test vectors:
        //   findings=[] → 1 row
        //   findings=[f1] → 2 rows
        //   findings=[f1, f2, f3] → 4 rows
        let cases: &[usize] = &[0, 1, 3];
        for &n in cases {
            let findings: Vec<Finding> = (0..n).map(|i| make_finding(format!("f{i}"))).collect();
            let csv_text = render_many(&findings);
            let rows = parse_csv(&csv_text);
            assert_eq!(
                rows.len(),
                1 + n,
                "BC-2.11.023 pc1/inv1: n={n} findings → expected {} rows, got {};\n{}",
                1 + n,
                rows.len(),
                csv_text
            );
        }
    }

    /// AC-005 (BC-2.11.023 invariant 1, EC-001):
    /// An empty findings slice produces exactly the header row — a valid 1-line CSV
    /// with zero data rows.
    ///
    /// BC-2.11.023 EC-001: "Output is the header row only; no data rows; valid 1-line CSV".
    ///
    /// Discriminating assertions:
    ///   - parse_csv yields exactly 1 row.
    ///   - That 1 row is the header (first field = "category").
    ///   - No second row present.
    ///   - Raw output is non-empty (valid CSV string, not "").
    #[test]
    fn test_BC_2_11_023_empty_findings_header_only() {
        // BC-2.11.023 inv1 / EC-001 canonical test vector:
        //   findings=[] → single header line only.
        let csv_text = render_many(&[]);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            1,
            "BC-2.11.023 inv1/EC-001: empty findings → exactly 1 row (header); got {};\n{}",
            rows.len(),
            csv_text
        );

        // The one row must be the header, not a data row.
        assert_eq!(
            rows[0][0], "category",
            "BC-2.11.023 inv1: sole row must be header; col 1 must be 'category', got '{}'",
            rows[0][0]
        );

        // Negative: no second row.
        assert!(
            rows.get(1).is_none(),
            "BC-2.11.023 inv1: no data row should exist for empty findings slice"
        );

        // Raw output is non-empty.
        assert!(
            !csv_text.is_empty(),
            "BC-2.11.023 inv1: output must be a non-empty valid CSV string"
        );
    }

    /// AC-004 (BC-2.11.023 postcondition 5, invariant 3):
    /// Row order in the CSV output matches the iteration order of the findings slice.
    /// First finding → data row 2 (index 1), second → data row 3, etc.
    /// No sorting or deduplication.
    ///
    /// BC-2.11.023 pc5 / inv3: "Row order is identical to the iteration order of the
    /// findings slice (no sorting, no deduplication)".
    ///
    /// Discriminating assertions:
    ///   - rows[1][3] == "alpha" (first finding summary).
    ///   - rows[2][3] == "beta"  (second finding summary).
    ///   - rows[3][3] == "gamma" (third finding summary).
    ///   - EC-005: duplicate finding emitted twice — no deduplication.
    #[test]
    fn test_BC_2_11_023_row_order_matches_findings_slice() {
        // BC-2.11.023 pc5 / inv3: order-preservation test.
        let findings = vec![
            make_finding("alpha"),
            make_finding("beta"),
            make_finding("gamma"),
        ];
        let csv_text = render_many(&findings);
        let rows = parse_csv(&csv_text);

        assert_eq!(
            rows.len(),
            4,
            "BC-2.11.023 pc5: expected 4 rows (header + 3); got {};\n{}",
            rows.len(),
            csv_text
        );
        assert_eq!(
            rows[1][3], "alpha",
            "BC-2.11.023 pc5: data row 1 summary must be 'alpha'; got '{}'",
            rows[1][3]
        );
        assert_eq!(
            rows[2][3], "beta",
            "BC-2.11.023 pc5: data row 2 summary must be 'beta'; got '{}'",
            rows[2][3]
        );
        assert_eq!(
            rows[3][3], "gamma",
            "BC-2.11.023 pc5: data row 3 summary must be 'gamma'; got '{}'",
            rows[3][3]
        );

        // BC-2.11.023 EC-005: duplicate entries — no deduplication.
        let dup_findings = vec![make_finding("same"), make_finding("same")];
        let csv_dup = render_many(&dup_findings);
        let rows_dup = parse_csv(&csv_dup);
        assert_eq!(
            rows_dup.len(),
            3,
            "BC-2.11.023 EC-005: duplicate findings → 3 rows (header + 2); got {};\n{}",
            rows_dup.len(),
            csv_dup
        );
        assert_eq!(
            rows_dup[1][3], "same",
            "BC-2.11.023 EC-005: first duplicate must appear; got '{}'",
            rows_dup[1][3]
        );
        assert_eq!(
            rows_dup[2][3], "same",
            "BC-2.11.023 EC-005: second duplicate must also appear (no dedup); got '{}'",
            rows_dup[2][3]
        );
    }

    /// AC-002 (BC-2.11.023 postcondition 2, EC-003):
    /// The `summary` parameter (Summary struct) is NOT reflected anywhere in the CSV output.
    /// A Summary with `total_packets = 9999` produces no row, column, or value
    /// containing 9999.
    ///
    /// BC-2.11.023 pc2 / EC-003 canonical test vector:
    ///   summary.total_packets=9999, findings=[] → header line only; 9999 not in output.
    ///
    /// Discriminating assertions:
    ///   - raw CSV does not contain "9999".
    ///   - raw CSV does not contain "888888".
    ///   - row count is 1 (header only).
    ///   - "total_packets" does not appear as a column name.
    #[test]
    fn test_BC_2_11_023_summary_not_in_output() {
        // BC-2.11.023 pc2 / EC-003.
        let mut summary = Summary::new();
        summary.total_packets = 9999;
        summary.total_bytes = 888_888;

        let csv_text = CsvReporter.render(&summary, &[], &[]);
        let rows = parse_csv(&csv_text);

        // Row count: header only (findings=[]).
        assert_eq!(
            rows.len(),
            1,
            "BC-2.11.023 pc2: summary ignored → 1 row (header only); got {};\n{}",
            rows.len(),
            csv_text
        );

        // Negative: 9999 must not appear in the CSV.
        assert!(
            !csv_text.contains("9999"),
            "BC-2.11.023 pc2: total_packets=9999 must NOT appear in CSV; got:\n{csv_text}"
        );

        // Negative: 888888 must not appear either.
        assert!(
            !csv_text.contains("888888"),
            "BC-2.11.023 pc2: total_bytes=888888 must NOT appear in CSV; got:\n{csv_text}"
        );

        // Negative: "total_packets" must not appear as a column name.
        assert!(
            !csv_text.contains("total_packets"),
            "BC-2.11.023 pc2: 'total_packets' must not appear in CSV; got:\n{csv_text}"
        );
    }

    /// AC-003 (BC-2.11.023 postcondition 3, EC-004):
    /// The `analyzer_summaries` parameter is NOT reflected anywhere in the CSV output.
    /// Non-empty analyzer_summaries produce no additional rows or columns.
    ///
    /// BC-2.11.023 pc3 / EC-004 canonical test vector:
    ///   analyzer_summaries=[tls_summary], findings=[f1]
    ///   → Header + 1 data row; tls_summary not in output.
    ///
    /// Discriminating assertions:
    ///   - Row count 2 (header + 1 data) despite non-empty analyzer_summaries.
    ///   - analyzer_name "TLS-Sentinel" does not appear in the raw CSV.
    ///   - "packets_analyzed" does not appear as a column header.
    ///   - Multiple analyzer_summaries still produce only 2 rows.
    #[test]
    fn test_BC_2_11_023_analyzer_summaries_not_in_output() {
        // BC-2.11.023 pc3 / EC-004.
        let finding = make_finding("finding-summary");
        let tls_summary = make_analysis_summary("TLS-Sentinel");
        let csv_text = CsvReporter.render(&Summary::new(), &[finding], &[tls_summary]);
        let rows = parse_csv(&csv_text);

        // Row count: header + 1 data row; analyzer_summaries do NOT add rows.
        assert_eq!(
            rows.len(),
            2,
            "BC-2.11.023 pc3: analyzer_summaries silently ignored → 2 rows; got {};\n{}",
            rows.len(),
            csv_text
        );

        // Negative: analyzer name must not appear in the CSV.
        assert!(
            !csv_text.contains("TLS-Sentinel"),
            "BC-2.11.023 pc3: analyzer_name 'TLS-Sentinel' must NOT appear; got:\n{csv_text}"
        );

        // Negative: "packets_analyzed" must not appear.
        assert!(
            !csv_text.contains("packets_analyzed"),
            "BC-2.11.023 pc3: 'packets_analyzed' must NOT appear as a column; got:\n{csv_text}"
        );

        // Multiple analyzer summaries still produce only 2 rows.
        let finding2 = make_finding("finding-2");
        let summ_a = make_analysis_summary("DNS");
        let summ_b = make_analysis_summary("HTTP");
        let csv_multi = CsvReporter.render(&Summary::new(), &[finding2], &[summ_a, summ_b]);
        let rows_multi = parse_csv(&csv_multi);
        assert_eq!(
            rows_multi.len(),
            2,
            "BC-2.11.023 pc3: 2 analyzer_summaries → still only 2 rows; got {}",
            rows_multi.len()
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.024 — None optional field encoding, Direction Debug format
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.11.024 postcondition 3, EC-006):
    /// When `Finding.direction = None`, column 8 (index 7) is an empty string `""`.
    ///
    /// BC-2.11.024 pc3: "direction cell: if None, empty string ''".
    ///
    /// Discriminating assertions:
    ///   - Parsed column 8 (index 7) == "".
    ///   - Does not equal "null", "None", "N/A", "-", or any other sentinel.
    #[test]
    fn test_BC_2_11_024_none_direction_is_empty() {
        // BC-2.11.024 pc3 / EC-006: None direction → empty string.
        // make_finding sets direction = None.
        let finding = make_finding("test");
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);
        let direction_cell = &rows[1][7]; // col 8, index 7

        assert_eq!(
            direction_cell.as_str(),
            "",
            "BC-2.11.024 pc3: None direction must be empty string ''; got {:?}",
            direction_cell
        );

        // Negative: must not be any sentinel value.
        let sentinels = ["null", "None", "N/A", "-", "undefined"];
        for sentinel in sentinels {
            assert_ne!(
                direction_cell.as_str(),
                sentinel,
                "BC-2.11.024 inv4: None direction must not produce sentinel {:?}; got {:?}",
                sentinel,
                direction_cell
            );
        }
    }

    /// AC-006 (BC-2.11.024 postcondition 1, EC-001):
    /// When `Finding.mitre_technique = None`, column 6 (index 5) is an empty string `""`.
    ///
    /// BC-2.11.024 pc1: "mitre_technique cell: if None, empty string ''".
    ///
    /// Discriminating assertions:
    ///   - Parsed column 6 (index 5) == "".
    ///   - Does not equal "null", "None", "N/A", or "-".
    #[test]
    fn test_BC_2_11_024_none_mitre_technique_is_empty() {
        // BC-2.11.024 pc1 / EC-001: None mitre_technique → empty string.
        // make_finding sets mitre_technique = None.
        let finding = make_finding("test");
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);
        let mitre_cell = &rows[1][5]; // col 6, index 5

        assert_eq!(
            mitre_cell.as_str(),
            "",
            "BC-2.11.024 pc1: None mitre_technique must be empty string ''; got {:?}",
            mitre_cell
        );

        // Negative: must not be any sentinel.
        let sentinels = ["null", "None", "N/A", "-", "undefined"];
        for sentinel in sentinels {
            assert_ne!(
                mitre_cell.as_str(),
                sentinel,
                "BC-2.11.024 inv4: None mitre_technique must not produce sentinel {:?}",
                sentinel
            );
        }
    }

    /// AC-011 (BC-2.11.024 invariant 4, EC-012):
    /// No sentinel values appear for any None optional field across all four optional
    /// columns (6, 7, 8, 9) when all are None simultaneously.
    ///
    /// BC-2.11.024 inv4: "None-to-empty-string conversion via unwrap_or('') /
    /// unwrap_or_default(); no sentinel values like 'null', 'N/A', or '-' are used."
    ///
    /// BC-2.11.024 EC-012: "All four Option fields None → all four cells empty string."
    ///
    /// Discriminating assertions:
    ///   - All four optional cells (indices 5, 6, 7, 8) == "".
    ///   - None of those cells is "null", "None", "N/A", "-", "undefined", or "nil".
    #[test]
    fn test_BC_2_11_024_no_sentinel_values_for_none() {
        // BC-2.11.024 inv4 / EC-012 canonical test vector:
        //   all four Option fields None → columns 6-9 are all empty strings.
        let finding = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "all-none".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        let csv_text = render_one(finding);
        let rows = parse_csv(&csv_text);
        let data_row = &rows[1];

        // All four optional columns must be exactly "".
        let optional_indices: &[(usize, &str)] = &[
            (5, "mitre_techniques"),
            (6, "source_ip"),
            (7, "direction"),
            (8, "timestamp"),
        ];
        for &(idx, col_name) in optional_indices {
            assert_eq!(
                data_row[idx].as_str(),
                "",
                "BC-2.11.024 inv4: None {col_name} (col {}) must be empty string ''; got {:?}",
                idx + 1,
                data_row[idx]
            );
        }

        // Negative: sentinel values must NOT appear in any of the four columns.
        let sentinels = ["null", "None", "N/A", "-", "undefined", "nil"];
        for &(idx, col_name) in optional_indices {
            for sentinel in sentinels {
                assert_ne!(
                    data_row[idx].as_str(),
                    sentinel,
                    "BC-2.11.024 inv4: None {col_name} must not produce sentinel {:?}; \
                     got {:?}",
                    sentinel,
                    data_row[idx]
                );
            }
        }
    }

    /// AC-008 (BC-2.11.024 postcondition 3, invariant 2, EC-007/EC-008):
    /// When `Finding.direction = Some(ClientToServer)`, column 8 is `"ClientToServer"`.
    /// When `Some(ServerToClient)`, column 8 is `"ServerToClient"`.
    /// Format is Debug (`{:?}`) — CamelCase, not lowercase.
    ///
    /// BC-2.11.024 pc3 / inv2: "direction via format!('{d:?}') which produces the
    /// variant name 'ClientToServer' or 'ServerToClient'".
    ///
    /// Discriminating assertions (both variants):
    ///   - ClientToServer → "ClientToServer" (not "clienttoserver").
    ///   - ServerToClient → "ServerToClient" (not "servertoClient").
    #[test]
    fn test_BC_2_11_024_direction_debug_camelcase() {
        // BC-2.11.024 pc3 / inv2 canonical test vectors: both Direction variants.
        let cases: &[(Direction, &str)] = &[
            (Direction::ClientToServer, "ClientToServer"),
            (Direction::ServerToClient, "ServerToClient"),
        ];
        for &(dir, expected) in cases {
            let mut finding = make_finding("test");
            finding.direction = Some(dir);
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);
            let direction_cell = &rows[1][7]; // col 8, index 7

            assert_eq!(
                direction_cell.as_str(),
                expected,
                "BC-2.11.024 pc3/inv2: direction {:?} must render as {:?}; got {:?}",
                dir,
                expected,
                direction_cell
            );

            // Negative: must NOT be lowercase (Debug format mandated, not Display).
            assert_ne!(
                direction_cell.as_str(),
                expected.to_lowercase().as_str(),
                "BC-2.11.024 inv2: direction must be CamelCase Debug format, not lowercase; \
                 got {:?}",
                direction_cell
            );
        }
    }

    /// AC-007 (BC-2.11.024 postcondition 2, EC-003/EC-004/EC-005):
    /// `source_ip` encoding:
    ///   - `None` → col 7 == `""`.
    ///   - `Some(192.168.1.1)` → `"192.168.1.1"`.
    ///   - `Some(::1)` → `"::1"` (IpAddr::to_string compact form).
    ///   - `Some(2001:db8::1)` → `"2001:db8::1"`.
    ///
    /// BC-2.11.024 pc2: "source_ip cell: if None, empty string ''; if Some(ip),
    /// ip.to_string()".
    ///
    /// Discriminating assertions across four cases (None, IPv4, two IPv6).
    #[test]
    fn test_BC_2_11_024_source_ip_encoding() {
        // BC-2.11.024 pc2 canonical test vectors.
        let cases: &[(Option<IpAddr>, &str)] = &[
            (None, ""),
            (Some("192.168.1.1".parse().unwrap()), "192.168.1.1"),
            (Some("::1".parse().unwrap()), "::1"),
            (Some("2001:db8::1".parse().unwrap()), "2001:db8::1"),
            (Some("10.0.0.1".parse().unwrap()), "10.0.0.1"),
        ];
        for &(ip, expected) in cases {
            let mut finding = make_finding("test");
            finding.source_ip = ip;
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);
            let source_ip_cell = &rows[1][6]; // col 7, index 6

            assert_eq!(
                source_ip_cell.as_str(),
                expected,
                "BC-2.11.024 pc2: source_ip={ip:?} must render as {:?}; got {:?}",
                expected,
                source_ip_cell
            );
        }
    }

    /// AC-010 (BC-2.11.024 postcondition 4, invariant 3, EC-009/EC-010):
    /// `timestamp` encoding:
    ///   - `None` → col 9 == `""`.
    ///   - `Some(datetime)` → col 9 == `to_rfc3339()` (e.g., `"2024-01-15T12:34:56+00:00"`).
    ///
    /// BC-2.11.024 pc4 / inv3: "timestamp cell: if None, empty string ''; if Some(t),
    /// t.to_rfc3339() — an ISO 8601 / RFC 3339 string".
    ///
    /// Discriminating assertions:
    ///   - None → col 9 == "".
    ///   - Some(2024-01-15T12:34:56Z) → contains "2024-01-15", "12:34:56", and 'T'.
    ///   - Non-empty when Some.
    #[test]
    fn test_BC_2_11_024_timestamp_rfc3339_encoding() {
        // BC-2.11.024 pc4 / inv3 / EC-009 canonical test vector: None → "".
        let finding_none = make_finding("test");
        let csv_none = render_one(finding_none);
        let rows_none = parse_csv(&csv_none);
        let ts_none_cell = &rows_none[1][8]; // col 9, index 8

        assert_eq!(
            ts_none_cell.as_str(),
            "",
            "BC-2.11.024 pc4: None timestamp must be empty string ''; got {:?}",
            ts_none_cell
        );

        // BC-2.11.024 pc4 / inv3 / EC-010 canonical test vector:
        //   Some(2024-01-15 12:34:56 UTC) → RFC3339 string.
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 12, 34, 56).unwrap();
        let mut finding_some = make_finding("test");
        finding_some.timestamp = Some(dt);
        let csv_some = render_one(finding_some);
        let rows_some = parse_csv(&csv_some);
        let ts_some_cell = &rows_some[1][8]; // col 9, index 8

        // Must be non-empty.
        assert!(
            !ts_some_cell.is_empty(),
            "BC-2.11.024 pc4: Some(timestamp) must produce non-empty cell"
        );

        // Must contain the date portion.
        assert!(
            ts_some_cell.contains("2024-01-15"),
            "BC-2.11.024 pc4: RFC3339 cell must contain '2024-01-15'; got {:?}",
            ts_some_cell
        );

        // Must contain the time portion.
        assert!(
            ts_some_cell.contains("12:34:56"),
            "BC-2.11.024 pc4: RFC3339 cell must contain '12:34:56'; got {:?}",
            ts_some_cell
        );

        // Must contain the 'T' date-time separator (RFC 3339 / ISO 8601).
        assert!(
            ts_some_cell.contains('T'),
            "BC-2.11.024 inv3: RFC3339 string must contain 'T' separator; got {:?}",
            ts_some_cell
        );

        // BC-2.11.024 EC-010 / pc4 (v1.3 lock): chrono::DateTime<Utc>::to_rfc3339()
        // always emits the +00:00 offset form, never bare 'Z'.
        // Exact canonical vector: fixture is 2024-01-15 12:34:56 UTC.
        assert_eq!(
            ts_some_cell.as_str(),
            "2024-01-15T12:34:56+00:00",
            "BC-2.11.024 EC-010/pc4 v1.3: DateTime<Utc>::to_rfc3339() must emit \
             '+00:00' offset form (not bare 'Z'); expected exact canonical vector \
             \"2024-01-15T12:34:56+00:00\", got {:?}",
            ts_some_cell
        );

        // Discriminating guards: +00:00 suffix required, bare Z suffix forbidden.
        assert!(
            ts_some_cell.ends_with("+00:00"),
            "BC-2.11.024 EC-010 v1.3: RFC3339 UTC timestamp must end with '+00:00'; \
             got {:?}",
            ts_some_cell
        );
        assert!(
            !ts_some_cell.ends_with('Z'),
            "BC-2.11.024 EC-010 v1.3: RFC3339 UTC timestamp must NOT use bare 'Z' suffix; \
             got {:?}",
            ts_some_cell
        );

        // Negative: must not be a sentinel.
        let sentinels = ["null", "None", "N/A", "-"];
        for sentinel in sentinels {
            assert_ne!(
                ts_some_cell.as_str(),
                sentinel,
                "BC-2.11.024 inv4: Some(timestamp) must not produce sentinel {:?}; got {:?}",
                sentinel,
                ts_some_cell
            );
        }
    }

    /// AC-012 (BC-2.11.024 postcondition 5, EC-011):
    /// All four optional-field-derived strings are individually passed through
    /// `neutralize_csv_injection` before the csv write.
    /// `mitre_technique = Some("=HYPERLINK(...)")` → `"'=HYPERLINK(...)"` in col 6.
    ///
    /// BC-2.11.024 pc5: "All four derived strings are individually passed through
    /// neutralize_csv_injection at csv.rs:94-97 before being written."
    ///
    /// Strategy: set `mitre_technique` to trigger-char values and verify the leading
    /// `'` prefix appears in column 6. (source_ip, direction, and timestamp produce
    /// non-trigger strings from their type-specific conversions, so mitre_technique
    /// is the canonical attacker-controlled optional field for this assertion.)
    ///
    /// Discriminating assertions:
    ///   - "=HYPERLINK(...)" → "'=HYPERLINK(...)".
    ///   - "+T1059" → "'+T1059".
    ///   - "-T9999" → "'-T9999".
    ///   - "@tag" → "'@tag".
    #[test]
    fn test_BC_2_11_024_optional_fields_neutralized() {
        // BC-2.11.024 pc5 / EC-011 canonical test vector.
        let trigger_mitres: &[(&str, &str)] = &[
            (
                "=HYPERLINK(http://evil.example)",
                "'=HYPERLINK(http://evil.example)",
            ),
            ("+T1059", "'+T1059"),
            ("-T9999", "'-T9999"),
            ("@tag", "'@tag"),
        ];
        for &(raw, expected) in trigger_mitres {
            let mut finding = make_finding("test");
            finding.mitre_techniques = vec![raw.to_string()];
            let csv_text = render_one(finding);
            let rows = parse_csv(&csv_text);
            let mitre_cell = &rows[1][5]; // col 6, index 5

            assert_eq!(
                mitre_cell.as_str(),
                expected,
                "BC-2.11.024 pc5: trigger mitre_technique {:?} must be neutralized to {:?}; \
                 got {:?}",
                raw,
                expected,
                mitre_cell
            );

            // Positive: must start with the single-quote prefix.
            assert!(
                mitre_cell.starts_with('\''),
                "BC-2.11.024 pc5: neutralized optional field must start with \"'\"; \
                 got {:?}",
                mitre_cell
            );
        }
    }
}
