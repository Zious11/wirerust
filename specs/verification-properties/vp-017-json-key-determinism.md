---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.11.001
bcs:
  - BC-2.11.001
  - BC-2.11.003
module: src/reporter/json.rs
proof_method: integration
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: Wave-21 wave-level consistency lens — SS-11 reporter VP proof-method family harmonization (DF-SIBLING-SWEEP-001; sibling of the 2026-05-30 VP-020 correction): frontmatter proof_method corrected from manual→integration to match VP-017 body Proof-Method table and VP-INDEX (integration is authoritative; the body specifies a concrete Rust integration test, not a manual proof) — 2026-05-30"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-017: JsonReporter Key-Order Determinism

## Property Statement

`JsonReporter` produces deterministic JSON output: repeated calls with
identical input data produce byte-identical JSON output.

Specifically:
1. The top-level JSON object keys appear in a fixed, deterministic order
   (`summary`, `findings`, `analyzers`) because `serde_json::json!({...})` at
   `json.rs:47-58` preserves insertion order (serde_json's `Map` is
   indexmap-backed). The order is `summary`, `findings`, `analyzers` --
   exactly the insertion order in the source -- not alphabetical.
2. The `protocols` and `services` sub-maps inside `summary` use `BTreeMap`
   (`json.rs:36-45`), so their keys ARE in alphabetical order, making
   protocol/service key order stable across platforms.
3. All `Option<T>` fields that are `None` are omitted from JSON output
   (`skip_serializing_if = "Option::is_none"`; findings.rs:132-145).
4. C0 control bytes in string fields are escaped per RFC 8259 by serde_json
   (BC-2.11.003); no raw ESC bytes appear in the JSON output.

## Source Contract

- **Primary BC:** BC-2.11.001 -- JsonReporter Renders JSON Object with summary/findings/analyzers Keys
- **Postcondition:** JSON output is deterministic and well-formed
- **Related BC:** BC-2.11.003 -- JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test | Rust test + assert | N/A | Fixed-input round-trip: produce JSON, parse, verify structure and key order |

## Test Specification

```rust
// JsonReporter is a unit struct (json.rs:21). Use the Reporter trait's
// render() method directly -- there is no render_json() free function.
// Reporter trait signature (reporter/mod.rs:27-32):
//   fn render(&self, summary: &Summary, findings: &[Finding],
//             analyzer_summaries: &[AnalysisSummary]) -> String;

#[test]
fn test_json_output_deterministic() {
    use crate::reporter::Reporter;
    use crate::reporter::json::JsonReporter;
    use crate::summary::Summary;

    let findings = vec![make_test_finding()];
    let summary = make_test_summary();
    let analyzer_summaries = vec![make_test_analyzer_summary()];

    let reporter = JsonReporter;
    // Parameter order: summary first, then findings, then analyzer_summaries.
    let output1 = reporter.render(&summary, &findings, &analyzer_summaries);
    let output2 = reporter.render(&summary, &findings, &analyzer_summaries);

    assert_eq!(output1, output2, "JSON output is not deterministic");
}

#[test]
fn test_json_top_level_key_order() {
    use crate::reporter::Reporter;
    use crate::reporter::json::JsonReporter;
    use crate::summary::Summary;

    let json_str = JsonReporter.render(&Summary::default(), &[], &[]);
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    // json.rs:47-58 uses serde_json::json!({...}) with keys in insertion order:
    // "summary", "findings", "analyzers". serde_json preserves insertion order
    // (Map is indexmap-backed), NOT alphabetical order. Assert the exact order.
    let obj = value.as_object().unwrap();
    let keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    assert_eq!(keys, vec!["summary", "findings", "analyzers"],
        "JSON top-level key order is not the expected insertion order: {:?}", keys);
}

#[test]
fn test_c0_bytes_escaped_in_json() {
    use crate::reporter::Reporter;
    use crate::reporter::json::JsonReporter;
    use crate::summary::Summary;

    let finding = Finding {
        summary: "\x1b[31mRED\x1b[0m".to_string(),
        ..make_test_finding()
    };
    // serde_json escapes ESC (0x1b) as  per RFC 8259.
    let json_str = JsonReporter.render(&Summary::default(), &[finding], &[]);
    assert!(!json_str.contains('\x1b'),
        "raw ESC byte in JSON output");
    assert!(json_str.contains("\\u001b"),
        "ESC not escaped as \\u001b in JSON");
}

#[test]
fn test_none_fields_omitted() {
    use crate::reporter::Reporter;
    use crate::reporter::json::JsonReporter;
    use crate::summary::Summary;

    // findings.rs:132-145: mitre_technique, source_ip, timestamp, direction all
    // carry #[serde(skip_serializing_if = "Option::is_none")].
    let finding = Finding {
        mitre_technique: None,
        timestamp: None,
        direction: None,
        source_ip: None,
        ..make_test_finding()
    };
    let json_str = JsonReporter.render(&Summary::default(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let first_finding = &parsed["findings"][0];
    assert!(first_finding.get("mitre_technique").is_none(),
        "None mitre_technique should be absent from JSON");
    assert!(first_finding.get("source_ip").is_none(),
        "None source_ip should be absent from JSON");
    assert!(first_finding.get("timestamp").is_none(),
        "None timestamp should be absent from JSON");
    assert!(first_finding.get("direction").is_none(),
        "None direction should be absent from JSON");
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite test inputs | Determinism is a structural guarantee from serde_json insertion-order map |
| Proof complexity | Very low | Property is guaranteed by serde_json indexmap-backed Map + BTreeMap for sub-maps |
| Tool support | High | Standard Rust integration test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/json.rs` -- `JsonReporter::render()` using `serde_json::json!({...})` for
deterministic insertion-order top-level keys; `BTreeMap` for `protocols` and `services`
sub-maps (json.rs:36-45, 47-58).
`src/findings.rs` -- `#[serde(skip_serializing_if = "Option::is_none")]` on Option fields
(findings.rs:132-145).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | null | formal-verifier |
| Tests passing | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
