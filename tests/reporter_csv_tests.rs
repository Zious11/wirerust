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
        mitre_technique: None,
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
            "mitre_technique",
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
        let expected_header_line = "category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp\n";
        assert!(
            csv_text.starts_with(expected_header_line),
            "BC-2.11.020 v1.3 pc1: raw output must begin with the LF-terminated header line;\n\
             expected prefix: {expected_header_line:?}\n\
             actual prefix: {:?}",
            &csv_text[..csv_text.len().min(120)]
        );

        // Confirm the header line contains the full column name sequence.
        let header_fields = "category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp";
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
            mitre_technique: Some("=T1234".to_string()), // col 6: trigger '='
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
        // BC-2.11.022 pc1 / inv1: canonical BC test vector: ["first","second","third"]
        // → "first; second; third".
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
