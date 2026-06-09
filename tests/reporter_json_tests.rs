//! STORY-076: JsonReporter formalization tests — Wave 20
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

/// AC-001 (BC-2.11.001 pc2): The parsed top-level object contains exactly the
/// keys "summary", "findings", "analyzers", "mitre_domain", and
/// "mitre_attack_version" — no other top-level keys exist.
/// STORY-101 / BC-2.11.001: two ATT&CK envelope fields added in v0.3.0.
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
            "summary"
        ],
        "BC-2.11.001 pc2: top-level keys must be exactly \
         {{summary, findings, analyzers, mitre_domain, mitre_attack_version}}, got: {keys:?}"
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
