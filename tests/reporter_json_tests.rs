//! STORY-076: JsonReporter formalization tests — Wave 20
//! STORY-129: BC-2.11.035 per-finding mitre_attack enrichment tests — Wave 57
//!
//! AC↔test-name sync enforced by PG-W17-001.  Every test fn name matches its
//! AC's `**Test:**` citation exactly.
//!
//! Behavioral contracts covered:
//!   BC-2.11.001  JsonReporter Renders JSON Object with summary/findings/analyzers Keys
//!   BC-2.11.002  JsonReporter Includes skipped_packets in Summary
//!   BC-2.11.003  JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde
//!   BC-2.11.004  JsonReporter Preserves Non-ASCII Unicode in Readable Form
//!   BC-2.11.005  JsonReporter Passes C1 Codepoints Through as Raw UTF-8
//!   BC-2.11.035  JsonReporter Per-Finding mitre_attack Enrichment (STORY-129, Wave 57)

// PG-W17-001 mandates that test fn names EXACTLY match the AC `**Test:**`
// citations (e.g. `test_BC_2_11_001_top_level_keys`).  These names use
// upper-case BC identifiers which Rust flags as non-snake-case.  Suppress
// the lint for this file rather than diverge from the required naming scheme.
#![allow(non_snake_case)]

use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::Reporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::summary::Summary;

// ---------------------------------------------------------------------------
// Shared helpers — mirror the construction patterns from reporter_tests.rs
// ---------------------------------------------------------------------------

/// Minimal Finding with no optional fields set.
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

/// Render with an empty Summary and the given findings/analyzers slices.
fn render(findings: &[Finding]) -> String {
    JsonReporter.render(&Summary::new(), findings, &[])
}

/// Parse the rendered JSON — panics with the full output on failure.
fn parse(json_str: &str) -> serde_json::Value {
    serde_json::from_str(json_str).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}\nOutput was:\n{json_str}");
    })
}

// ---------------------------------------------------------------------------
// BC-2.11.001: top-level structure
// ---------------------------------------------------------------------------

/// AC-001 (BC-2.11.001 pc2 v1.9): The parsed top-level object contains exactly
/// the six keys "analyzers", "findings", "mitre_attack_version", "mitre_domain",
/// "schema_version", and "summary" — no other top-level keys exist.
/// STORY-101 / BC-2.11.001: ATT&CK envelope fields added in v0.3.0.
/// STORY-160 / BC-2.11.001 v1.9 / BC-2.11.037: schema_version added in v0.12.0.
/// (DF-SIBLING-SWEEP-001: vec updated from five-key to six-key form.)
#[test]
fn test_BC_2_11_001_top_level_keys() {
    let json_str = render(&[]);
    let value = parse(&json_str);
    let obj = value
        .as_object()
        .expect("top-level value must be a JSON object");

    // Exact key set — collect and sort for deterministic failure messages.
    let mut keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "analyzers",
            "findings",
            "mitre_attack_version",
            "mitre_domain",
            "schema_version",
            "summary"
        ],
        "BC-2.11.001 pc2 v1.9: top-level keys must be exactly \
         {{analyzers, findings, mitre_attack_version, mitre_domain, schema_version, summary}}, \
         got: {keys:?}"
    );

    // Positive: each expected key is present.
    assert!(
        obj.contains_key("summary"),
        "\"summary\" key must be present"
    );
    assert!(
        obj.contains_key("findings"),
        "\"findings\" key must be present"
    );
    assert!(
        obj.contains_key("analyzers"),
        "\"analyzers\" key must be present"
    );
    // STORY-101: ATT&CK envelope fields.
    assert!(
        obj.contains_key("mitre_domain"),
        "\"mitre_domain\" key must be present (STORY-101)"
    );
    assert!(
        obj.contains_key("mitre_attack_version"),
        "\"mitre_attack_version\" key must be present (STORY-101)"
    );
    // STORY-160 / BC-2.11.037: schema_version envelope field.
    assert!(
        obj.contains_key("schema_version"),
        "\"schema_version\" key must be present (STORY-160 / BC-2.11.037)"
    );
}

/// AC-002 (BC-2.11.001 pc3): "findings" is a JSON array with one element per
/// Finding in the input slice; an empty findings slice produces "findings": [].
#[test]
fn test_BC_2_11_001_findings_array_length() {
    // Empty slice → empty array.
    let empty_json = render(&[]);
    let empty_val = parse(&empty_json);
    let empty_arr = empty_val["findings"]
        .as_array()
        .expect("\"findings\" must be a JSON array");
    assert_eq!(
        empty_arr.len(),
        0,
        "BC-2.11.001 pc3: empty findings slice must produce findings=[], got length {}",
        empty_arr.len()
    );

    // One finding → array of length 1.
    let one_finding = [make_finding("finding A")];
    let one_json = render(&one_finding);
    let one_val = parse(&one_json);
    let one_arr = one_val["findings"]
        .as_array()
        .expect("\"findings\" must be a JSON array");
    assert_eq!(
        one_arr.len(),
        1,
        "BC-2.11.001 pc3: one finding must produce findings array of length 1, got {}",
        one_arr.len()
    );

    // Two findings → array of length 2.
    let two_findings = [make_finding("finding A"), make_finding("finding B")];
    let two_json = render(&two_findings);
    let two_val = parse(&two_json);
    let two_arr = two_val["findings"]
        .as_array()
        .expect("\"findings\" must be a JSON array");
    assert_eq!(
        two_arr.len(),
        2,
        "BC-2.11.001 pc3: two findings must produce findings array of length 2, got {}",
        two_arr.len()
    );
}

/// AC-003 (BC-2.11.001 pc5): The "summary" object contains sub-keys
/// total_packets, total_bytes, skipped_packets, unique_hosts, protocols,
/// and services.
#[test]
fn test_BC_2_11_001_summary_subkeys() {
    // BC-2.11.001 pc5: six required sub-keys must be present.
    let json_str = render(&[]);
    let value = parse(&json_str);
    let summary = value["summary"]
        .as_object()
        .expect("\"summary\" must be a JSON object");

    let required = [
        "total_packets",
        "total_bytes",
        "skipped_packets",
        "unique_hosts",
        "protocols",
        "services",
    ];
    for key in required {
        assert!(
            summary.contains_key(key),
            "BC-2.11.001 pc5: summary sub-key \"{key}\" must be present; summary keys: {:?}",
            summary.keys().collect::<Vec<_>>()
        );
    }
}

/// AC-004 (BC-2.11.001 pc6): The output is pretty-printed — indented with
/// spaces, one key per line (serde_json::to_string_pretty).
#[test]
fn test_BC_2_11_001_output_is_pretty_printed() {
    // BC-2.11.001 pc6: pretty-printed output contains newlines and indentation.
    //
    // serde_json::to_string_pretty uses "  " (two-space) indentation.
    // A compact serializer (to_string) would produce a single-line blob
    // with no leading whitespace.  We verify:
    //   1. The output contains at least one newline.
    //   2. At least one line begins with one or more space characters
    //      (indentation evidence).
    //   3. A known top-level key ("summary") appears on its own indented
    //      line, proving two-space indentation — not just any whitespace.
    //      serde_json::to_string_pretty emits "\n  \"key\"" for top-level
    //      object members.  This discriminates "pretty" (newline + two
    //      spaces + key) from compact (no newline) and tab-indented output.
    let json_str = render(&[]);

    assert!(
        json_str.contains('\n'),
        "BC-2.11.001 pc6: pretty-printed JSON must contain newlines; got single-line output"
    );

    let has_indented_line = json_str.lines().any(|line| line.starts_with(' '));
    assert!(
        has_indented_line,
        "BC-2.11.001 pc6: pretty-printed JSON must have at least one indented line; \
         got:\n{json_str}"
    );

    // F-002 remediation: structural indentation proof — at least one line begins
    // with exactly two spaces followed by a double-quote (a 2-space-indented JSON
    // key).  This discriminates `to_string_pretty` from both compact output (no
    // leading space) and tab-indented output, without coupling to a specific key name.
    assert!(
        json_str.lines().any(|l| l.starts_with("  \"")),
        "BC-2.11.001 pc6: serde_json::to_string_pretty must produce lines beginning \
         with '  \"' (two-space-indented quoted key); got:\n{json_str}"
    );

    // Additionally verify the known top-level key "summary" is indented as expected
    // (two spaces), proving the structural assertion is not satisfied by nested keys only.
    assert!(
        json_str.contains("\n  \"summary\""),
        "BC-2.11.001 pc6: serde_json::to_string_pretty must indent top-level keys \
         with exactly two spaces — expected the literal '\\n  \"summary\"' substring \
         in output; got:\n{json_str}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.002: skipped_packets always present in summary
// ---------------------------------------------------------------------------

/// AC-005 (BC-2.11.002 pc2): When Summary.skipped_packets = 0 the JSON output
/// contains "skipped_packets": 0 — the key is present with value 0, not absent.
///
/// BC-2.11.002 inv1: skipped_packets is ALWAYS present regardless of value.
#[test]
fn test_BC_2_11_002_skipped_packets_zero_present() {
    // BC-2.11.002 pc2 + inv1: zero value must produce the key, not suppress it.
    let mut summary = Summary::new();
    summary.skipped_packets = 0;
    let json_str = JsonReporter.render(&summary, &[], &[]);
    let value = parse(&json_str);

    let skipped = value["summary"]
        .as_object()
        .expect("summary must be an object")
        .get("skipped_packets")
        .expect("BC-2.11.002 inv1: \"skipped_packets\" key must be present even when value is 0");

    assert_eq!(
        skipped.as_u64(),
        Some(0),
        "BC-2.11.002 pc2: skipped_packets value must be 0, got: {skipped}"
    );
}

/// AC-006 (BC-2.11.002 pc3): When Summary.skipped_packets = 3 the JSON output
/// contains "skipped_packets": 3.
#[test]
fn test_BC_2_11_002_skipped_packets_nonzero() {
    // BC-2.11.002 pc3: non-zero value must be serialized as JSON integer.
    let mut summary = Summary::new();
    summary.skipped_packets = 3;
    let json_str = JsonReporter.render(&summary, &[], &[]);
    let value = parse(&json_str);

    let skipped = value["summary"]["skipped_packets"].as_u64();
    assert_eq!(
        skipped,
        Some(3),
        "BC-2.11.002 pc3: skipped_packets must be 3, got: {:?}",
        value["summary"]["skipped_packets"]
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.003: C0 bytes escaped, DEL not escaped, round-trip
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.11.003 pc1): A Finding with ESC (0x1B) in its summary field
/// produces JSON where the ESC byte appears as the six-character sequence
/// , not as a raw 0x1B byte.
///
/// BC-2.11.003 pc1: C0 bytes → \uNNNN in JSON text.
/// BC-2.11.003 inv1: JsonReporter NEVER calls escape_for_terminal.
#[test]
fn test_BC_2_11_003_c0_esc_escaped_in_json() {
    // BC-2.11.003 pc1: ESC (0x1B) in a finding summary must appear as 
    // in the serialized JSON string, not as the raw 0x1B byte.
    let finding = make_finding("\x1b[31mRED\x1b[0m");
    let json_str = render(&[finding]);

    // Raw ESC byte must not be present.
    assert!(
        !json_str.as_bytes().contains(&0x1b),
        "BC-2.11.003 pc1: raw ESC (0x1B) must not appear in JSON output; \
         serde_json must have escaped it as \\u001b"
    );

    // The six-character escape sequence must be present.
    assert!(
        json_str.contains("\\u001b"),
        "BC-2.11.003 pc1: ESC must appear as \\u001b in JSON output; got:\n{json_str}"
    );
}

/// AC-008 (BC-2.11.003 pc2): DEL (0x7F) is NOT escaped by serde_json; it
/// passes through as a raw 0x7F byte in the JSON output string.
///
/// BC-2.11.003 pc2: DEL (0x7F) is above the C0 range and is NOT escaped.
/// BC-2.11.003 inv2: serde_json escapes C0 (0x00-0x1F) but passes DEL and
/// C1 through as raw UTF-8.
#[test]
fn test_BC_2_11_003_del_not_escaped_in_json() {
    // BC-2.11.003 pc2: DEL (0x7F) must NOT be converted to a \uNNNN sequence;
    // it must appear as the literal 0x7F byte in the output.
    let finding = make_finding("before\x7fafter");
    let json_str = render(&[finding]);

    // Raw DEL byte must be present (serde_json does not escape it).
    assert!(
        json_str.as_bytes().contains(&0x7f),
        "BC-2.11.003 pc2: DEL (0x7F) must pass through as raw byte in JSON output; \
         got output where 0x7F is absent"
    );

    // F-001 remediation: confirm DEL did not become either lowercase or uppercase
    // \u escape.  serde_json emits lowercase hex, but we also guard uppercase to
    // prove the postcondition "NOT escaped" rather than "not escaped as lowercase."
    assert!(
        !json_str.contains("\\u007f"),
        "BC-2.11.003 pc2: DEL must NOT be escaped as \\u007f (lowercase); \
         serde_json's contract is C0-only escaping"
    );
    assert!(
        !json_str.contains("\\u007F"),
        "BC-2.11.003 pc2: DEL must NOT be escaped as \\u007F (uppercase); \
         any \\u007F/\\u007f form proves incorrect escaping of DEL"
    );
}

/// AC-009 (BC-2.11.003 pc4): A round-trip (serialize Finding with C0 bytes,
/// then deserialize the JSON) recovers the original byte sequence exactly.
///
/// BC-2.11.003 pc4: round-trip recovers original bytes.
/// BC-2.11.003 inv3: behavior is deterministic.
///
/// Pass-1 remediation: added discriminating escaped-form-absence assertions on
/// the intermediate JSON wire format so a test cannot pass by accident when the
/// JSON parser normalises an incorrectly-unescaped value.
#[test]
fn test_BC_2_11_003_c0_roundtrip() {
    // BC-2.11.003 pc4: serialize a Finding that contains several C0 bytes
    // (NUL, BEL, ESC) and verify that deserializing the resulting JSON
    // recovers the original summary string byte-for-byte.
    let original_summary = "\x00null\x07bel\x1b[31mesc-seq\x1b[0m";
    let finding = make_finding(original_summary);

    let json_str = render(&[finding]);

    // --- Discriminating wire-format assertions (pass-1 remediation) ----------
    // Each C0 byte must appear as its \uNNNN escape on the wire; raw bytes must
    // be absent.  These checks ensure the round-trip cannot silently pass when
    // the serializer emits raw control bytes that a lenient parser re-normalises.

    // NUL (0x00) → must be escaped, raw byte must be absent.
    assert!(
        !json_str.as_bytes().contains(&0x00),
        "BC-2.11.003 pc4 wire: raw NUL (0x00) must not appear in JSON output; \
         serde_json must have escaped it as \\u0000"
    );
    assert!(
        json_str.contains("\\u0000"),
        "BC-2.11.003 pc4 wire: NUL must appear as \\u0000 in JSON output; got:\n{json_str}"
    );

    // BEL (0x07) → must be escaped, raw byte must be absent.
    assert!(
        !json_str.as_bytes().contains(&0x07),
        "BC-2.11.003 pc4 wire: raw BEL (0x07) must not appear in JSON output; \
         serde_json must have escaped it as \\u0007"
    );
    assert!(
        json_str.contains("\\u0007"),
        "BC-2.11.003 pc4 wire: BEL must appear as \\u0007 in JSON output; got:\n{json_str}"
    );

    // ESC (0x1B) → must be escaped, raw byte must be absent.
    assert!(
        !json_str.as_bytes().contains(&0x1b),
        "BC-2.11.003 pc4 wire: raw ESC (0x1B) must not appear in JSON output; \
         serde_json must have escaped it as \\u001b"
    );
    assert!(
        json_str.contains("\\u001b"),
        "BC-2.11.003 pc4 wire: ESC must appear as \\u001b in JSON output; got:\n{json_str}"
    );
    // -------------------------------------------------------------------------

    // The JSON must be valid and parseable.
    let parsed = parse(&json_str);

    // Extract the round-tripped summary value.
    let recovered = parsed["findings"][0]["summary"]
        .as_str()
        .expect("findings[0].summary must be a JSON string");

    assert_eq!(
        recovered, original_summary,
        "BC-2.11.003 pc4: round-trip must recover original bytes exactly; \
         original={original_summary:?} recovered={recovered:?}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.004: non-ASCII Unicode preserved as readable UTF-8
// ---------------------------------------------------------------------------

/// AC-010 (BC-2.11.004 pc1): A Finding with a Cyrillic hostname in summary
/// produces JSON where the Cyrillic characters appear as raw UTF-8 bytes, NOT
/// as \u escape sequences.
///
/// BC-2.11.004 pc1: Cyrillic → raw UTF-8 in JSON, not \uNNNN.
/// BC-2.11.004 inv1: serde_json's default serializer does not escape printable
/// non-ASCII Unicode.
#[test]
fn test_BC_2_11_004_cyrillic_preserved_readable() {
    // BC-2.11.004 pc1: "пример.рф" (Cyrillic) must appear literally in the
    // JSON output, not as при... escape sequences.
    let cyrillic_summary = "TLS SNI: пример.рф";
    let finding = make_finding(cyrillic_summary);
    let json_str = render(&[finding]);

    // The Cyrillic string must be present literally (raw UTF-8).
    assert!(
        json_str.contains("пример.рф"),
        "BC-2.11.004 pc1: Cyrillic must appear as readable UTF-8 in JSON output, \
         not as escape sequences; got:\n{json_str}"
    );

    // No Debug-format \u{NNNN} sequences (the old regression form).
    assert!(
        !json_str.contains("\\u{43f}"),
        "BC-2.11.004 pc1: Cyrillic must not appear as Debug-formatted \\u{{NNN}} \
         escapes (construction-site regression); got:\n{json_str}"
    );

    // No RFC 8259 \uNNNN escapes for the Cyrillic code points.
    // U+043F (п) → would appear as п if incorrectly escaped.
    assert!(
        !json_str.contains("\\u043f"),
        "BC-2.11.004 pc1: Cyrillic 'п' must not appear as \\u043f RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );

    // F-001 remediation: per-character exact-escape-absence assertions for every
    // non-ASCII codepoint in the fixture string "пример.рф".  Asserting the
    // incomplete prefix "\\u04" would be over-broad and fragile (JSON \u escapes
    // are exactly 4 hex digits; a prefix match could collide with unrelated output).
    // Instead we assert the exact 6-character \uXXXX sequence for each codepoint.
    // Codepoints in fixture (serde_json emits lowercase hex):
    //   п = U+043F → п
    //   р = U+0440 → р
    //   и = U+0438 → и
    //   м = U+043C → м
    //   е = U+0435 → е
    //   ф = U+0444 → ф
    assert!(
        !json_str.contains("\\u043f"),
        "BC-2.11.004 pc1: 'п' (U+043F) must not appear as \\u043f RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );
    assert!(
        !json_str.contains("\\u0440"),
        "BC-2.11.004 pc1: 'р' (U+0440) must not appear as \\u0440 RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );
    assert!(
        !json_str.contains("\\u0438"),
        "BC-2.11.004 pc1: 'и' (U+0438) must not appear as \\u0438 RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );
    assert!(
        !json_str.contains("\\u043c"),
        "BC-2.11.004 pc1: 'м' (U+043C) must not appear as \\u043c RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );
    assert!(
        !json_str.contains("\\u0435"),
        "BC-2.11.004 pc1: 'е' (U+0435) must not appear as \\u0435 RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );
    assert!(
        !json_str.contains("\\u0444"),
        "BC-2.11.004 pc1: 'ф' (U+0444) must not appear as \\u0444 RFC-escape; \
         serde_json must emit raw UTF-8 for printable non-ASCII; got:\n{json_str}"
    );

    // Round-trip: deserializing must recover the original Cyrillic string.
    let parsed = parse(&json_str);
    let recovered = parsed["findings"][0]["summary"]
        .as_str()
        .expect("findings[0].summary must be a JSON string");
    assert_eq!(
        recovered, cyrillic_summary,
        "BC-2.11.004 pc1: round-trip must recover original Cyrillic string exactly"
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.005: C1 codepoints pass through as raw UTF-8
// ---------------------------------------------------------------------------

/// AC-011 (BC-2.11.005 pc1): A Finding with U+009B (C1 CSI) in summary
/// produces JSON where the CSI appears as the raw two-byte UTF-8 sequence
/// 0xC2 0x9B, NOT as the text .
///
/// BC-2.11.005 pc1: C1 codepoints appear as raw UTF-8 in JSON output.
/// BC-2.11.005 inv1: serde_json does NOT escape codepoints above U+001F.
#[test]
fn test_BC_2_11_005_c1_passthrough_raw_utf8() {
    // BC-2.11.005 pc1: U+009B (C1 CSI) encoded as 0xC2 0x9B in UTF-8 must
    // pass through serde_json as-is.  The  escape sequence must NOT
    // appear in the JSON output bytes.
    let c1_csi = "\u{009b}"; // encodes as 0xC2 0x9B in UTF-8
    let finding = make_finding(format!("payload: {c1_csi}31mINJECTED"));
    let json_str = render(&[finding]);

    // The raw 0xC2 0x9B byte pair must be present in the output.
    let bytes = json_str.as_bytes();
    let has_raw_c1 = bytes.windows(2).any(|w| w == [0xC2, 0x9B]);
    assert!(
        has_raw_c1,
        "BC-2.11.005 pc1: C1 CSI (U+009B) must appear as raw 0xC2 0x9B in JSON output; \
         serde_json must not escape it"
    );

    // F-003 remediation: guard both lowercase and uppercase forms of the \u escape
    // for U+009B.  serde_json emits lowercase hex, but the negative postcondition
    // is "NOT escaped at all" — both case variants must be absent to prove it.
    assert!(
        !json_str.contains("\\u009b"),
        "BC-2.11.005 pc1: C1 CSI must NOT appear as \\u009b (lowercase) in JSON output; \
         RFC 8259 only mandates escaping of C0 (U+0000-U+001F)"
    );
    assert!(
        !json_str.contains("\\u009B"),
        "BC-2.11.005 pc1: C1 CSI must NOT appear as \\u009B (uppercase) in JSON output; \
         any \\u009b/\\u009B form proves incorrect escaping of U+009B"
    );
}

/// AC-012 (BC-2.11.005 inv2): A Finding with both ESC (C0, 0x1B) and U+009B
/// (C1) in summary produces JSON where ESC is  and C1 is raw 0xC2 0x9B —
/// the two characters are treated differently.
///
/// BC-2.11.005 inv2: asymmetry — C0 is escaped, C1 is not.
/// BC-2.11.003 pc1: C0 → \uNNNN.
/// BC-2.11.005 pc1: C1 → raw UTF-8.
#[test]
fn test_BC_2_11_005_c0_escaped_c1_passthrough_in_same_string() {
    // BC-2.11.005 inv2: same string, different treatment.
    //   ESC (0x1B, C0) →  in JSON text (escaped per RFC 8259)
    //   U+009B (C1 CSI) → raw 0xC2 0x9B bytes in JSON text (NOT escaped)
    let mixed = format!("\x1b[31m{}\x1b[0m", "\u{009b}INJECTED");
    let finding = make_finding(&mixed);
    let json_str = render(&[finding]);
    let bytes = json_str.as_bytes();

    // C0 ESC must be escaped as  — no raw 0x1B byte.
    assert!(
        !bytes.contains(&0x1b),
        "BC-2.11.005 inv2 / BC-2.11.003 pc1: raw ESC (0x1B, C0) must NOT appear in \
         JSON output; serde_json must have escaped it as \\u001b"
    );
    assert!(
        json_str.contains("\\u001b"),
        "BC-2.11.005 inv2 / BC-2.11.003 pc1: ESC must appear as \\u001b in JSON output; \
         got:\n{json_str}"
    );

    // C1 CSI must be present as raw 0xC2 0x9B — NOT escaped.
    let has_raw_c1 = bytes.windows(2).any(|w| w == [0xC2, 0x9B]);
    assert!(
        has_raw_c1,
        "BC-2.11.005 inv2 / BC-2.11.005 pc1: C1 CSI (U+009B) must remain as raw \
         0xC2 0x9B in JSON output alongside the escaped C0 ESC byte"
    );
    // AC-012 remediation: guard both case variants for C1 escape absence.
    // The postcondition is "C1 NOT escaped" — both \\u009b and \\u009B must
    // be absent to fully discriminate raw-UTF-8 from escaped form.
    assert!(
        !json_str.contains("\\u009b"),
        "BC-2.11.005 inv2: C1 CSI must NOT appear as \\u009b (lowercase); \
         only C0 bytes are escaped by serde_json"
    );
    assert!(
        !json_str.contains("\\u009B"),
        "BC-2.11.005 inv2: C1 CSI must NOT appear as \\u009B (uppercase); \
         any \\u009b/\\u009B form proves incorrect escaping of U+009B"
    );
}

/// AC-013 (BC-2.11.005 pc1): A Finding with U+0080 (lower boundary of the C1
/// range) in summary produces JSON where U+0080 appears as the raw two-byte
/// UTF-8 sequence 0xC2 0x80, NOT as the text .
///
/// Per RFC 8259 §7, only U+0000–U+001F (plus `"` and `\`) require escaping.
/// U+0080, the first codepoint above the ASCII range, is in the C1 block and
/// must pass through serde_json unescaped.
#[test]
fn test_BC_2_11_005_c1_lower_boundary_u0080_passthrough_raw_utf8() {
    // BC-2.11.005 pc1: U+0080 (C1 PAD, bottom of the C1 range) encoded as
    // 0xC2 0x80 in UTF-8 must pass through serde_json as-is.  The 
    // escape sequence must NOT appear in the JSON output bytes.
    let c1_pad = "\u{0080}"; // encodes as 0xC2 0x80 in UTF-8
    let finding = make_finding(format!("payload: {c1_pad}boundary"));
    let json_str = render(&[finding]);

    // The raw 0xC2 0x80 byte pair must be present in the output.
    let bytes = json_str.as_bytes();
    let has_raw_c1 = bytes.windows(2).any(|w| w == [0xC2, 0x80]);
    assert!(
        has_raw_c1,
        "BC-2.11.005 pc1: C1 U+0080 must appear as raw 0xC2 0x80 in JSON output; \
         serde_json must not escape it"
    );

    // Guard both lowercase and uppercase forms of the \u escape for U+0080.
    assert!(
        !json_str.contains("\\u0080"),
        "BC-2.11.005 pc1: C1 U+0080 must NOT appear as \\u0080 (lowercase) in JSON output; \
         RFC 8259 only mandates escaping of C0 (U+0000-U+001F)"
    );
    assert!(
        !json_str.contains("\\u0080"),
        "BC-2.11.005 pc1: C1 U+0080 must NOT appear as \\u0080 (uppercase) in JSON output; \
         any \\u0080 form proves incorrect escaping of U+0080"
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.035: per-finding mitre_attack enrichment (STORY-129, Wave 57)
//
// AC-1 through AC-8 verify that JsonReporter emits the correct mitre_attack
// array for each finding by routing through FindingJsonDto, which enriches
// each technique ID with name, tactic_id, tactic_name, and reference.
//
// AC-9 and AC-10 verify CsvReporter and TerminalReporter are unaffected:
// neither reporter emits JSON objects, so mitre_attack is structurally absent
// from their output space.
// ---------------------------------------------------------------------------

/// AC-1 (BC-2.11.035 pc1, pc3a-3e, EC-002): Known single technique T1046
/// produces a fully-resolved 5-field mitre_attack object in the JSON output.
///
/// Verifies that JsonReporter emits mitre_attack[0] with id, name, tactic_id,
/// tactic_name, and reference all correctly populated for T1046.
#[test]
fn test_BC_2_11_035_known_technique_all_five_fields() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1046".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"].as_array().expect(
        "BC-2.11.035 pc1: mitre_attack must be a JSON array for non-empty mitre_techniques",
    );

    assert_eq!(
        attack_arr.len(),
        1,
        "BC-2.11.035 pc2: mitre_attack must have exactly 1 element for 1 technique"
    );
    let entry = &attack_arr[0];
    assert_eq!(entry["id"], "T1046", "BC-2.11.035 pc3a: id must be T1046");
    assert_eq!(
        entry["name"], "Network Service Discovery",
        "BC-2.11.035 pc3b: name must be Network Service Discovery"
    );
    assert_eq!(
        entry["tactic_id"], "TA0007",
        "BC-2.11.035 pc3c: tactic_id must be TA0007 (Discovery)"
    );
    assert_eq!(
        entry["tactic_name"], "Discovery",
        "BC-2.11.035 pc3d: tactic_name must be Discovery"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T1046/",
        "BC-2.11.035 pc3e: reference must be synthesized URL"
    );
}

/// AC-2 (BC-2.11.035 pc4, inv1, EC-001): Unknown technique T9999 produces a
/// partial object: id and reference only; name/tactic_id/tactic_name absent.
///
/// Verifies that FindingJsonDto preserves unrecognized technique IDs rather
/// than dropping them, and that skip_serializing_if suppresses the optional
/// fields entirely (not even null) when the catalog lookup returns None.
#[test]
fn test_BC_2_11_035_unknown_technique_id_never_lost() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T9999".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 inv1: mitre_attack must be present even for unknown IDs");

    assert_eq!(
        attack_arr.len(),
        1,
        "BC-2.11.035 pc2: one element for one technique"
    );
    let entry = &attack_arr[0];
    assert_eq!(
        entry["id"], "T9999",
        "BC-2.11.035 inv1: id must be present for unknown technique"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T9999/",
        "BC-2.11.035 pc3e: reference must be synthesized even for unknown IDs"
    );
    // BC-2.11.035 pc3b/3c/3d: skip_serializing_if = Option::is_none means the JSON
    // key must be ABSENT entirely — not present as null. A null value would mean
    // the serializer emitted the key, violating the skip contract.
    assert!(
        entry.get("name").is_none(),
        "BC-2.11.035 pc3b: name must be absent (not even null) for unknown technique; \
         skip_serializing_if must suppress the key entirely, got: {:?}",
        entry.get("name")
    );
    assert!(
        entry.get("tactic_id").is_none(),
        "BC-2.11.035 pc3c: tactic_id must be absent (not even null) for unknown technique; \
         skip_serializing_if must suppress the key entirely, got: {:?}",
        entry.get("tactic_id")
    );
    assert!(
        entry.get("tactic_name").is_none(),
        "BC-2.11.035 pc3d: tactic_name must be absent (not even null) for unknown technique; \
         skip_serializing_if must suppress the key entirely, got: {:?}",
        entry.get("tactic_name")
    );
}

/// AC-3 (BC-2.11.035 pc4, EC-001): Empty mitre_techniques vec omits the
/// mitre_attack key entirely from the finding JSON object.
///
/// Verifies that when a Finding has no mitre_techniques, JsonReporter omits
/// both mitre_techniques and mitre_attack from the serialized finding object
/// (skip_serializing_if = is_empty / is_none respectively).
#[test]
fn test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack() {
    let finding = make_finding("test finding"); // mitre_techniques: vec![] by default
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let finding_obj = value["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    assert!(
        !finding_obj.contains_key("mitre_attack"),
        "BC-2.11.035 pc4: mitre_attack must be absent when mitre_techniques is empty"
    );
    assert!(
        !finding_obj.contains_key("mitre_techniques"),
        "BC-2.09.006: mitre_techniques must be absent when vec is empty (skip_serializing_if)"
    );
}

/// AC-4 (BC-2.11.035 pc2, inv2, EC-006): Multi-tag finding: mitre_attack array
/// order matches mitre_techniques order exactly.
///
/// Verifies that FindingJsonDto maps techniques in declaration order and that
/// multi-technique ICS findings (T1692.001, T0836) each resolve to TA0106.
#[test]
fn test_BC_2_11_035_multitag_order_preserved() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1692.001".to_string(), "T0836".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 pc1: mitre_attack must be present for non-empty mitre_techniques");

    assert_eq!(
        attack_arr.len(),
        2,
        "BC-2.11.035 pc2: exactly 2 elements for 2 techniques"
    );
    assert_eq!(
        attack_arr[0]["id"], "T1692.001",
        "BC-2.11.035 inv2: index 0 must be T1692.001 (declaration order)"
    );
    assert_eq!(
        attack_arr[1]["id"], "T0836",
        "BC-2.11.035 inv2: index 1 must be T0836 (declaration order)"
    );
    assert_eq!(
        attack_arr[0]["tactic_id"], "TA0106",
        "BC-2.11.035 EC-006: T1692.001 tactic_id must be TA0106 (IcsImpairProcessControl)"
    );
    assert_eq!(
        attack_arr[1]["tactic_id"], "TA0106",
        "BC-2.11.035 EC-006: T0836 tactic_id must be TA0106 (IcsImpairProcessControl)"
    );
}

/// AC-5 (BC-2.11.035 inv3, EC-007): Duplicate technique IDs produce duplicate
/// (non-deduplicated) elements. mitre_attack.len() == mitre_techniques.len().
///
/// Verifies that FindingJsonDto maps each element of mitre_techniques
/// independently — no deduplication occurs — preserving the 1:1 cardinality
/// invariant between mitre_techniques and mitre_attack.
#[test]
fn test_BC_2_11_035_duplicate_ids_not_deduplicated() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec![
        "T1046".to_string(),
        "T9999".to_string(),
        "T1046".to_string(),
    ];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 pc1: mitre_attack must be present");

    assert_eq!(
        attack_arr.len(),
        3,
        "BC-2.11.035 inv3: must have 3 elements (no deduplication); \
         got {}",
        attack_arr.len()
    );
    assert_eq!(attack_arr[0]["id"], "T1046", "index 0 must be T1046");
    assert_eq!(attack_arr[1]["id"], "T9999", "index 1 must be T9999");
    assert_eq!(
        attack_arr[2]["id"], "T1046",
        "index 2 must be T1046 (duplicate)"
    );
}

/// AC-6 (BC-2.11.035 pc3e, inv4, EC-005): Sub-technique dot separator is
/// preserved verbatim in id and in the reference URL.
///
/// Verifies that T1071.001 serializes with the literal dot in the id field
/// and in the synthesized reference URL, and resolves to the correct name
/// and tactic (TA0011, Command and Control).
#[test]
fn test_BC_2_11_035_sub_technique_dot_preserved() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1071.001".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 pc1: mitre_attack must be present");

    assert_eq!(attack_arr.len(), 1, "one element for one technique");
    let entry = &attack_arr[0];
    assert_eq!(
        entry["id"], "T1071.001",
        "BC-2.11.035 pc3a + inv4: dot separator must be preserved verbatim in id"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T1071.001/",
        "BC-2.11.035 pc3e + inv4: dot separator must be preserved in reference URL"
    );
    assert_eq!(entry["name"], "Web Protocols", "name must be Web Protocols");
    assert_eq!(
        entry["tactic_id"], "TA0011",
        "BC-2.11.035: T1071.001 tactic_id must be TA0011 (CommandAndControl)"
    );
    assert_eq!(
        entry["tactic_name"], "Command and Control",
        "BC-2.11.035: tactic_name must be Command and Control"
    );
}

/// AC-7 (BC-2.11.035 Catalog Extension, EC-003): ICS technique T0827 resolves
/// tactic_id to TA0105 (ICS-matrix ID for IcsImpact), not TA0040.
///
/// Verifies that the ICS-specific tactic ID (TA0105) is emitted rather than
/// the Enterprise Impact tactic ID (TA0040), and that tactic_name is the
/// ICS-qualified display string "Impact (ICS)".
#[test]
fn test_BC_2_11_035_ics_tactic_id_resolved() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T0827".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 pc1: mitre_attack must be present");

    let entry = &attack_arr[0];
    assert_eq!(entry["id"], "T0827", "id must be T0827");
    assert_eq!(
        entry["name"], "Loss of Control",
        "name must be Loss of Control"
    );
    assert_eq!(
        entry["tactic_id"], "TA0105",
        "BC-2.11.035 EC-003: ICS IcsImpact tactic_id must be TA0105, NOT TA0040"
    );
    assert_eq!(
        entry["tactic_name"], "Impact (ICS)",
        "BC-2.11.035 EC-003: tactic_name must be Impact (ICS)"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T0827/",
        "reference must be synthesized URL"
    );
}

/// AC-8 (BC-2.11.035 pc5, inv5, EC-002): mitre_techniques array is unchanged
/// (additive non-breaking) when mitre_attack is also present.
///
/// Verifies that JsonReporter preserves the raw mitre_techniques array
/// alongside the enriched mitre_attack array — the enrichment is additive
/// and does not replace or remove the original technique IDs.
#[test]
fn test_BC_2_11_035_mitre_techniques_unchanged() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1046".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let finding_obj = &value["findings"][0];

    // mitre_techniques must still be present unchanged.
    let techniques = finding_obj["mitre_techniques"]
        .as_array()
        .expect("BC-2.11.035 pc5: mitre_techniques must still be present");
    assert_eq!(techniques.len(), 1, "mitre_techniques must have 1 element");
    assert_eq!(
        techniques[0], "T1046",
        "BC-2.11.035 pc5: mitre_techniques[0] must be T1046 unchanged"
    );

    // mitre_attack must also be present (additive).
    assert!(
        finding_obj.get("mitre_attack").is_some(),
        "BC-2.11.035 pc5 / inv5: mitre_attack must be present alongside mitre_techniques"
    );
}

/// AC-9 (BC-2.11.035 pc6): mitre_attack is absent from CSV output.
/// CsvReporter is unmodified by STORY-129.
///
/// Verifies that CsvReporter, which produces flat delimited text rather than
/// JSON, does not emit a "mitre_attack" key anywhere in its output even when
/// the finding carries populated mitre_techniques.
#[test]
fn test_BC_2_11_035_csv_unaffected() {
    use wirerust::reporter::csv::CsvReporter;

    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1046".to_string()];
    let csv_output = CsvReporter.render(&wirerust::summary::Summary::new(), &[finding], &[]);

    assert!(
        !csv_output.contains("mitre_attack"),
        "BC-2.11.035 pc6: mitre_attack must NOT appear in CSV output; \
         CsvReporter is unmodified by STORY-129"
    );
}

/// AC-10 (BC-2.11.035 pc7): mitre_attack is absent from terminal output.
/// TerminalReporter is unmodified by STORY-129.
///
/// Verifies that TerminalReporter, which produces human-readable colored text
/// rather than JSON, does not emit a "mitre_attack" key anywhere in its output
/// even when the finding carries populated mitre_techniques.
#[test]
fn test_BC_2_11_035_terminal_unaffected() {
    use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};

    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1046".to_string()];
    let reporter = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
    };
    let terminal_output = reporter.render(&wirerust::summary::Summary::new(), &[finding], &[]);

    assert!(
        !terminal_output.contains("mitre_attack"),
        "BC-2.11.035 pc7: mitre_attack must NOT appear in terminal output; \
         TerminalReporter is unmodified by STORY-129"
    );
}

/// BC-2.11.035 EC-009: Enterprise sub-technique T1557.002 (ARP Cache Poisoning)
/// resolves to tactic Credential Access (TA0006).
///
/// Verifies that an Enterprise sub-technique with a dot separator maps correctly
/// through FindingJsonDto: id and reference preserve the dot, tactic_id is TA0006,
/// and tactic_name is the exact Display string for MitreTactic::CredentialAccess.
/// Catalog confirmed: T1557.002 → "Adversary-in-the-Middle: ARP Cache Poisoning",
/// MitreTactic::CredentialAccess → TA0006 → "Credential Access" (STORY-114).
#[test]
fn test_BC_2_11_035_ec009_enterprise_subtechnique() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T1557.002".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 EC-009: mitre_attack must be present for T1557.002");

    assert_eq!(
        attack_arr.len(),
        1,
        "BC-2.11.035 EC-009: one element for one technique"
    );
    let entry = &attack_arr[0];
    assert_eq!(
        entry["id"], "T1557.002",
        "BC-2.11.035 EC-009: id must be T1557.002 with dot separator preserved"
    );
    assert_eq!(
        entry["name"], "Adversary-in-the-Middle: ARP Cache Poisoning",
        "BC-2.11.035 EC-009: name must match catalog entry for T1557.002"
    );
    assert_eq!(
        entry["tactic_id"], "TA0006",
        "BC-2.11.035 EC-009: tactic_id must be TA0006 (Credential Access)"
    );
    assert_eq!(
        entry["tactic_name"], "Credential Access",
        "BC-2.11.035 EC-009: tactic_name must be Credential Access"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T1557.002/",
        "BC-2.11.035 EC-009: reference must be synthesized URL with dot separator"
    );
}

/// BC-2.11.035 EC-010: ICS technique T0830 (Adversary-in-the-Middle) resolves
/// to tactic Collection (ICS) (TA0100), not Lateral Movement.
///
/// F5 correctness fix: T0830 maps to MitreTactic::IcsCollection (ICS TA0100),
/// not MitreTactic::LateralMovement (Enterprise TA0008). The ICS ATT&CK matrix
/// places "Adversary-in-the-Middle" under the Collection tactic (TA0100).
/// Verifies that FindingJsonDto emits tactic_id "TA0100" and tactic_name
/// "Collection (ICS)" for T0830.
#[test]
fn test_BC_2_11_035_ec010_ics_collection() {
    let mut finding = make_finding("test finding");
    finding.mitre_techniques = vec!["T0830".to_string()];
    let json_str = render(&[finding]);
    let value = parse(&json_str);

    let attack_arr = value["findings"][0]["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 EC-010: mitre_attack must be present for T0830");

    assert_eq!(
        attack_arr.len(),
        1,
        "BC-2.11.035 EC-010: one element for one technique"
    );
    let entry = &attack_arr[0];
    assert_eq!(entry["id"], "T0830", "BC-2.11.035 EC-010: id must be T0830");
    assert_eq!(
        entry["name"], "Adversary-in-the-Middle",
        "BC-2.11.035 EC-010: name must match catalog entry for T0830"
    );
    assert_eq!(
        entry["tactic_id"], "TA0100",
        "BC-2.11.035 EC-010: tactic_id must be TA0100 (IcsCollection / Collection ICS), \
         NOT TA0008 (Enterprise Lateral Movement) — F5 correctness fix"
    );
    assert_eq!(
        entry["tactic_name"], "Collection (ICS)",
        "BC-2.11.035 EC-010: tactic_name must be Collection (ICS), \
         NOT Lateral Movement — F5 correctness fix"
    );
    assert_eq!(
        entry["reference"], "https://attack.mitre.org/techniques/T0830/",
        "BC-2.11.035 EC-010: reference must be synthesized URL"
    );
}

/// BC-2.11.035 EC-008: Mixed-batch per-finding independence.
///
/// In a multi-finding report, each finding's `mitre_attack` is computed
/// independently.  A finding with empty `mitre_techniques` omits `mitre_attack`
/// entirely, while sibling findings in the same report still emit theirs.
///
/// Exercises the BC's canonical "Report with 3 findings" test vector:
///   findings[0]: mitre_techniques = ["T1046"]  → mitre_attack present (1 entry, id T1046)
///   findings[1]: mitre_techniques = []          → mitre_attack absent
///   findings[2]: mitre_techniques = ["T0827"]  → mitre_attack present (1 entry, id T0827)
///
/// Also verifies that the raw `mitre_techniques` field is preserved on the
/// findings that carry it (additive non-breaking, BC-2.11.035 pc5).
#[test]
fn test_BC_2_11_035_mixed_batch_per_finding_independence() {
    // finding A — has one technique (T1046, Network Service Discovery)
    let mut finding_a = make_finding("finding with T1046");
    finding_a.mitre_techniques = vec!["T1046".to_string()];

    // finding B — empty mitre_techniques; mitre_attack must be absent
    let finding_b = make_finding("finding with no techniques");
    // mitre_techniques defaults to vec![] via make_finding

    // finding C — has one ICS technique (T0827, Loss of Control)
    let mut finding_c = make_finding("finding with T0827");
    finding_c.mitre_techniques = vec!["T0827".to_string()];

    // Render all three in a single call.
    let json_str = render(&[finding_a, finding_b, finding_c]);
    let value = parse(&json_str);

    let findings = value["findings"]
        .as_array()
        .expect("BC-2.11.035 EC-008: findings must be a JSON array");

    assert_eq!(
        findings.len(),
        3,
        "BC-2.11.035 EC-008: three findings must produce a findings array of length 3"
    );

    // --- findings[0]: T1046 must produce mitre_attack with one fully-resolved entry ---
    let f0 = &findings[0];
    let attack_0 = f0["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 EC-008: findings[0] must have mitre_attack (T1046 is non-empty)");
    assert_eq!(
        attack_0.len(),
        1,
        "BC-2.11.035 EC-008: findings[0].mitre_attack must have exactly 1 entry"
    );
    assert_eq!(
        attack_0[0]["id"], "T1046",
        "BC-2.11.035 EC-008: findings[0].mitre_attack[0].id must be T1046"
    );
    // mitre_techniques raw field must also be present on findings[0].
    let techniques_0 = f0["mitre_techniques"]
        .as_array()
        .expect("BC-2.11.035 EC-008 / pc5: findings[0].mitre_techniques must be present");
    assert_eq!(
        techniques_0.len(),
        1,
        "BC-2.11.035 EC-008 / pc5: findings[0].mitre_techniques must have 1 element"
    );
    assert_eq!(
        techniques_0[0], "T1046",
        "BC-2.11.035 EC-008 / pc5: findings[0].mitre_techniques[0] must be T1046"
    );

    // --- findings[1]: empty mitre_techniques → mitre_attack key must be absent entirely ---
    let f1 = f1_obj(findings);
    assert!(
        f1.get("mitre_attack").is_none(),
        "BC-2.11.035 EC-008 / pc4: findings[1].mitre_attack must be absent when \
         mitre_techniques is empty; skip_serializing_if must suppress the key"
    );
    assert!(
        f1.get("mitre_techniques").is_none(),
        "BC-2.11.035 EC-008: findings[1].mitre_techniques must be absent when vec is empty \
         (skip_serializing_if)"
    );

    // --- findings[2]: T0827 must produce mitre_attack with one fully-resolved ICS entry ---
    let f2 = &findings[2];
    let attack_2 = f2["mitre_attack"]
        .as_array()
        .expect("BC-2.11.035 EC-008: findings[2] must have mitre_attack (T0827 is non-empty)");
    assert_eq!(
        attack_2.len(),
        1,
        "BC-2.11.035 EC-008: findings[2].mitre_attack must have exactly 1 entry"
    );
    assert_eq!(
        attack_2[0]["id"], "T0827",
        "BC-2.11.035 EC-008: findings[2].mitre_attack[0].id must be T0827"
    );
    // mitre_techniques raw field must also be present on findings[2].
    let techniques_2 = f2["mitre_techniques"]
        .as_array()
        .expect("BC-2.11.035 EC-008 / pc5: findings[2].mitre_techniques must be present");
    assert_eq!(
        techniques_2.len(),
        1,
        "BC-2.11.035 EC-008 / pc5: findings[2].mitre_techniques must have 1 element"
    );
    assert_eq!(
        techniques_2[0], "T0827",
        "BC-2.11.035 EC-008 / pc5: findings[2].mitre_techniques[0] must be T0827"
    );
}

/// Helper: extract findings[1] as an object for the mixed-batch test.
/// Using a named function keeps the borrow checker happy without a let-binding
/// that outlives the `findings` temporary in the caller.
fn f1_obj(findings: &[serde_json::Value]) -> &serde_json::Map<String, serde_json::Value> {
    findings[1]
        .as_object()
        .expect("BC-2.11.035 EC-008: findings[1] must be a JSON object")
}

// ---------------------------------------------------------------------------
// BC-2.11.036: JSON enum-value casing + surface-independence (STORY-160)
// ---------------------------------------------------------------------------

/// BC-2.11.036 pc1 + ec001: Verdict::Likely serializes to "likely" (lowercase)
/// in JSON output. The pre-v0.12.0 form "Likely" must not appear.
#[test]
fn test_BC_2_11_036_verdict_likely_serializes_lowercase() {
    let json = serde_json::to_value(Verdict::Likely)
        .expect("Verdict::Likely must serialize without error");
    assert_eq!(
        json,
        serde_json::Value::String("likely".to_string()),
        "BC-2.11.036 pc1: Verdict::Likely must serialize to JSON string \"likely\" \
         (not \"Likely\"); got: {json}"
    );
}

/// BC-2.11.036 pc1 + ec011: All four Verdict variants serialize to their
/// lowercase form; zero PascalCase occurrences in the serialized JSON.
#[test]
fn test_BC_2_11_036_verdict_all_variants_lowercase() {
    // Per-variant assertion for clear failure messages.
    let cases: &[(Verdict, &str)] = &[
        (Verdict::Likely, "likely"),
        (Verdict::Unlikely, "unlikely"),
        (Verdict::Inconclusive, "inconclusive"),
        (Verdict::Possible, "possible"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_value(variant)
            .expect("Verdict variant must serialize without error");
        assert_eq!(
            json,
            serde_json::Value::String(expected.to_string()),
            "BC-2.11.036 pc1: Verdict::{variant:?} must serialize to \
             \"{expected}\"; got: {json}"
        );
    }
    // Exhaustive PascalCase-absence guard (bc pc4).
    let arr_json = serde_json::to_string(&[
        Verdict::Likely,
        Verdict::Unlikely,
        Verdict::Inconclusive,
        Verdict::Possible,
    ])
    .expect("serializing Verdict array must not fail");
    for pascal in &["\"Likely\"", "\"Unlikely\"", "\"Inconclusive\"", "\"Possible\""] {
        assert!(
            !arr_json.contains(pascal),
            "BC-2.11.036 pc4: PascalCase form {pascal} must not appear in \
             serialized Verdict array; got: {arr_json}"
        );
    }
}

/// BC-2.11.036 pc2 + ec003: Confidence::High serializes to "high" (lowercase).
#[test]
fn test_BC_2_11_036_confidence_high_serializes_lowercase() {
    let json = serde_json::to_value(Confidence::High)
        .expect("Confidence::High must serialize without error");
    assert_eq!(
        json,
        serde_json::Value::String("high".to_string()),
        "BC-2.11.036 pc2: Confidence::High must serialize to JSON string \"high\" \
         (not \"High\"); got: {json}"
    );
}

/// BC-2.11.036 pc2 + ec012: All three Confidence variants serialize to their
/// lowercase form; zero PascalCase occurrences in the serialized JSON.
#[test]
fn test_BC_2_11_036_confidence_all_variants_lowercase() {
    let cases: &[(Confidence, &str)] = &[
        (Confidence::High, "high"),
        (Confidence::Medium, "medium"),
        (Confidence::Low, "low"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_value(variant)
            .expect("Confidence variant must serialize without error");
        assert_eq!(
            json,
            serde_json::Value::String(expected.to_string()),
            "BC-2.11.036 pc2: Confidence::{variant:?} must serialize to \
             \"{expected}\"; got: {json}"
        );
    }
    // PascalCase-absence guard.
    let arr_json =
        serde_json::to_string(&[Confidence::High, Confidence::Medium, Confidence::Low])
            .expect("serializing Confidence array must not fail");
    for pascal in &["\"High\"", "\"Medium\"", "\"Low\""] {
        assert!(
            !arr_json.contains(pascal),
            "BC-2.11.036 pc4: PascalCase form {pascal} must not appear in \
             serialized Confidence array; got: {arr_json}"
        );
    }
}

/// BC-2.11.036 pc3 + ec005: ThreatCategory::LateralMovement serializes to
/// "lateral_movement" (snake_case with underscore at word boundary).
#[test]
fn test_BC_2_11_036_threat_category_lateral_movement_snake_case() {
    let json = serde_json::to_value(ThreatCategory::LateralMovement)
        .expect("ThreatCategory::LateralMovement must serialize without error");
    assert_eq!(
        json,
        serde_json::Value::String("lateral_movement".to_string()),
        "BC-2.11.036 pc3: ThreatCategory::LateralMovement must serialize to \
         \"lateral_movement\"; got: {json}"
    );
}

/// BC-2.11.036 pc3 + ec007 (EC-001 story): ThreatCategory::C2 serializes to
/// "c2" — single uppercase letter lowercased; digit '2' is treated as a
/// non-alphabetic continuation by serde's snake_case algorithm (no underscore).
#[test]
fn test_BC_2_11_036_threat_category_c2_snake_case() {
    let json = serde_json::to_value(ThreatCategory::C2)
        .expect("ThreatCategory::C2 must serialize without error");
    assert_eq!(
        json,
        serde_json::Value::String("c2".to_string()),
        "BC-2.11.036 pc3 + EC-001: ThreatCategory::C2 must serialize to \"c2\" \
         (no underscore; serde snake_case lowercases 'C' and treats '2' as \
         non-alpha continuation); got: {json}"
    );
}

/// BC-2.11.036 pc3 + ec013: All ten ThreatCategory variants serialize to their
/// snake_case form; zero PascalCase occurrences.
#[test]
fn test_BC_2_11_036_threat_category_all_variants_snake_case() {
    let cases: &[(ThreatCategory, &str)] = &[
        (ThreatCategory::Reconnaissance, "reconnaissance"),
        (ThreatCategory::LateralMovement, "lateral_movement"),
        (ThreatCategory::C2, "c2"),
        (ThreatCategory::Exfiltration, "exfiltration"),
        (ThreatCategory::CredentialAccess, "credential_access"),
        (ThreatCategory::Persistence, "persistence"),
        (ThreatCategory::Execution, "execution"),
        (ThreatCategory::Anomaly, "anomaly"),
        (ThreatCategory::Suspicious, "suspicious"),
        (ThreatCategory::Impact, "impact"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_value(variant)
            .expect("ThreatCategory variant must serialize without error");
        assert_eq!(
            json,
            serde_json::Value::String(expected.to_string()),
            "BC-2.11.036 pc3: ThreatCategory::{variant:?} must serialize to \
             \"{expected}\"; got: {json}"
        );
    }
    // PascalCase-absence guard over all variants (bc pc4).
    let arr_json = serde_json::to_string(&[
        ThreatCategory::Reconnaissance,
        ThreatCategory::LateralMovement,
        ThreatCategory::C2,
        ThreatCategory::Exfiltration,
        ThreatCategory::CredentialAccess,
        ThreatCategory::Persistence,
        ThreatCategory::Execution,
        ThreatCategory::Anomaly,
        ThreatCategory::Suspicious,
        ThreatCategory::Impact,
    ])
    .expect("serializing ThreatCategory array must not fail");
    for pascal in &[
        "\"Reconnaissance\"",
        "\"LateralMovement\"",
        "\"C2\"",
        "\"Exfiltration\"",
        "\"CredentialAccess\"",
        "\"Persistence\"",
        "\"Execution\"",
        "\"Anomaly\"",
        "\"Suspicious\"",
        "\"Impact\"",
    ] {
        assert!(
            !arr_json.contains(pascal),
            "BC-2.11.036 pc4: PascalCase form {pascal} must not appear in \
             serialized ThreatCategory array; got: {arr_json}"
        );
    }
}

/// BC-2.11.036 pc5 / inv2: Terminal Display for all three enums is UNCHANGED.
/// Verdict/Confidence produce UPPERCASE tokens; ThreatCategory uses Debug repr
/// (PascalCase). Regression guard — remains green before and after the serde
/// rename_all annotations are applied (serde and Display are independent surfaces).
#[test]
fn test_BC_2_11_036_terminal_display_unchanged() {
    // Verdict: fmt::Display produces UPPERCASE tokens per BC-2.09.003.
    assert_eq!(
        Verdict::Likely.to_string(),
        "LIKELY",
        "BC-2.11.036 pc5: Verdict::Likely.to_string() must be \"LIKELY\"; \
         serde rename_all must not affect fmt::Display"
    );
    assert_eq!(
        Verdict::Inconclusive.to_string(),
        "INCONCLUSIVE",
        "BC-2.11.036 pc5: Verdict::Inconclusive.to_string() must be \"INCONCLUSIVE\""
    );
    // Confidence: fmt::Display produces UPPERCASE tokens per BC-2.09.004.
    assert_eq!(
        Confidence::High.to_string(),
        "HIGH",
        "BC-2.11.036 pc5: Confidence::High.to_string() must be \"HIGH\"; \
         serde rename_all must not affect fmt::Display"
    );
    // ThreatCategory: fmt::Display delegates to Debug repr via write!(f, "{self:?}").
    // Produces PascalCase (e.g. "LateralMovement"), NOT snake_case.
    assert_eq!(
        ThreatCategory::LateralMovement.to_string(),
        "LateralMovement",
        "BC-2.11.036 pc5 + v1.2: ThreatCategory::LateralMovement.to_string() must be \
         \"LateralMovement\" (PascalCase Debug repr); serde rename_all must not affect \
         fmt::Display (which uses write!(f, \"{{self:?}}\"))"
    );
}

/// BC-2.11.036 pc6 + ec010: CSV output for ThreatCategory is UNCHANGED.
/// The CSV reporter uses f.category.to_string() (Display → Debug repr),
/// so the PascalCase form "LateralMovement" is preserved in the CSV cell.
/// Regression guard — remains green before and after the serde annotation.
#[test]
fn test_BC_2_11_036_csv_category_unchanged() {
    use wirerust::reporter::csv::CsvReporter;

    let mut finding = make_finding("csv regression for ThreatCategory casing");
    finding.category = ThreatCategory::LateralMovement;
    let csv_output =
        CsvReporter.render(&wirerust::summary::Summary::new(), &[finding], &[]);

    assert!(
        csv_output.contains("LateralMovement"),
        "BC-2.11.036 pc6 + EC-010: CSV output must contain \"LateralMovement\" \
         (Debug repr / PascalCase); the serde rename_all annotation must not affect \
         CSV output (which uses Display/Debug, not Serialize); got:\n{csv_output}"
    );
    // Confirm the snake_case form does NOT bleed into CSV.
    assert!(
        !csv_output.contains("lateral_movement"),
        "BC-2.11.036 pc6: CSV output must NOT contain snake_case \"lateral_movement\"; \
         CSV path uses Display/Debug — not Serialize; got:\n{csv_output}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.11.037: schema_version envelope field (STORY-160)
// ---------------------------------------------------------------------------

/// BC-2.11.037 pc1: The JSON report output contains a "schema_version" key
/// at the top level of the envelope.
#[test]
fn test_BC_2_11_037_schema_version_present_in_json() {
    let json_str = render(&[make_finding("test finding for schema_version check")]);
    let value = parse(&json_str);
    let obj = value
        .as_object()
        .expect("top-level JSON must be an object");
    assert!(
        obj.contains_key("schema_version"),
        "BC-2.11.037 pc1: JSON envelope must contain \"schema_version\" key; \
         top-level keys found: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

/// BC-2.11.037 pc2: The value of "schema_version" is the JSON string "2"
/// (not integer 2, not null, not absent).
#[test]
fn test_BC_2_11_037_schema_version_value_is_two() {
    let json_str = render(&[make_finding("test finding for schema_version value")]);
    let value = parse(&json_str);
    assert_eq!(
        value["schema_version"],
        serde_json::Value::String("2".to_string()),
        "BC-2.11.037 pc2: schema_version must be the JSON string \"2\" \
         (not integer 2, not null, not absent); got: {}",
        value["schema_version"]
    );
}

/// BC-2.11.037 pc3: "schema_version" is present even when findings slice is
/// empty — the field is unconditional (a constant, not derived from input).
#[test]
fn test_BC_2_11_037_schema_version_unconditional_empty_findings() {
    let json_str = render(&[]);
    let value = parse(&json_str);
    let obj = value.as_object().expect("top-level JSON must be an object");
    assert!(
        obj.contains_key("schema_version"),
        "BC-2.11.037 pc3: schema_version must be present even with empty findings slice; \
         top-level keys: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        value["schema_version"],
        serde_json::Value::String("2".to_string()),
        "BC-2.11.037 pc3: schema_version value must be \"2\" even when findings is empty"
    );
    // Belt-and-braces: confirm the findings array really is empty.
    assert_eq!(
        value["findings"].as_array().map(|a| a.len()),
        Some(0),
        "BC-2.11.037 pc3: findings must be an empty array in this test path"
    );
}

/// BC-2.11.037 pc4: "schema_version" is absent from CSV output.
/// CsvReporter has no envelope concept and is unaffected by BC-2.11.037.
/// Regression guard — remains green before and after implementation.
#[test]
fn test_BC_2_11_037_schema_version_absent_from_csv() {
    use wirerust::reporter::csv::CsvReporter;

    let finding = make_finding("csv surface-independence check");
    let csv_output =
        CsvReporter.render(&wirerust::summary::Summary::new(), &[finding], &[]);
    assert!(
        !csv_output.contains("schema_version"),
        "BC-2.11.037 pc4: schema_version must NOT appear in CSV output; \
         the CsvReporter emits no envelope fields; got:\n{csv_output}"
    );
}

/// BC-2.11.037 pc5: "schema_version" is absent from terminal output.
/// TerminalReporter has no envelope concept and is unaffected by BC-2.11.037.
/// Regression guard — remains green before and after implementation.
#[test]
fn test_BC_2_11_037_schema_version_absent_from_terminal() {
    use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};

    let finding = make_finding("terminal surface-independence check");
    let reporter = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
    };
    let terminal_output =
        reporter.render(&wirerust::summary::Summary::new(), &[finding], &[]);
    assert!(
        !terminal_output.contains("schema_version"),
        "BC-2.11.037 pc5: schema_version must NOT appear in terminal output; \
         the TerminalReporter emits no envelope fields; got:\n{terminal_output}"
    );
}
