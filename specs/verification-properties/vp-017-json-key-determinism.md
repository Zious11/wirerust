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
  - BC-2.11.003
module: src/reporter/json.rs
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

# VP-017: JsonReporter BTreeMap Key Determinism

## Property Statement

`JsonReporter` produces deterministic JSON output: repeated calls with
identical input data produce byte-identical JSON output.

Specifically:
1. The top-level JSON object keys appear in a fixed, deterministic order
   (`summary`, `findings`, `analyzers`) because `BTreeMap` is used instead
   of `HashMap` for the serialization root (closed by PR #76).
2. The `analyzers` field's detail sub-maps also use deterministic key order.
3. All `Option<T>` fields that are `None` are omitted from JSON output
   (`skip_serializing_if = "Option::is_none"`; BC-2.09.006).
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
#[test]
fn test_json_output_deterministic() {
    // Build identical inputs twice, verify output is byte-identical
    let findings = vec![make_test_finding()];
    let summary = make_test_summary();
    let analyzer_summaries = vec![make_test_analyzer_summary()];

    let output1 = render_json(&findings, &analyzer_summaries, &summary);
    let output2 = render_json(&findings, &analyzer_summaries, &summary);

    assert_eq!(output1, output2, "JSON output is not deterministic");
}

#[test]
fn test_json_top_level_key_order() {
    let json_str = render_json(&[], &[], &Summary::default());
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    // BTreeMap serialization: keys in alphabetical order
    // "analyzers" < "findings" < "summary"
    let obj = value.as_object().unwrap();
    let keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    assert!(keys.windows(2).all(|w| w[0] < w[1]),
        "JSON keys not in sorted (BTreeMap) order: {:?}", keys);
}

#[test]
fn test_c0_bytes_escaped_in_json() {
    let finding = Finding {
        summary: "\x1b[31mRED\x1b[0m".to_string(),
        ..make_test_finding()
    };
    let json_str = render_json(&[finding], &[], &Summary::default());
    // serde_json must escape ESC (0x1b) as 
    assert!(!json_str.contains('\x1b'),
        "raw ESC byte in JSON output");
    assert!(json_str.contains("\\u001b"),
        "ESC not escaped as \\u001b in JSON");
}

#[test]
fn test_none_fields_omitted() {
    let finding = Finding {
        mitre_technique: None,
        timestamp: None,
        direction: None,
        ..make_test_finding()
    };
    let json_str = render_json(&[finding], &[], &Summary::default());
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let first_finding = &parsed["findings"][0];
    assert!(first_finding.get("mitre_technique").is_none(),
        "None mitre_technique should be absent from JSON");
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite test inputs | Determinism is a structural guarantee from BTreeMap |
| Proof complexity | Very low | Property is guaranteed by BTreeMap + serde_json behavior |
| Tool support | High | Standard Rust integration test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/json.rs` -- `JsonReporter` using `BTreeMap` for deterministic key order (PR #76).
`src/findings.rs` -- `#[serde(skip_serializing_if = "Option::is_none")]` on Option fields.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | null | formal-verifier |
| Tests passing | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
