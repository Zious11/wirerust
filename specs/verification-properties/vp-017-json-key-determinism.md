---
document_type: verification-property
level: L4
version: "2.0"
status: verified
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
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "523e63626df9728e13b8f23346d6877aaf929865fe3e580ade0542e87183a66d"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: Wave-21 wave-level consistency lens — SS-11 reporter VP proof-method family harmonization (DF-SIBLING-SWEEP-001; sibling of the 2026-05-30 VP-020 correction): frontmatter proof_method corrected from manual→integration to match VP-017 body Proof-Method table and VP-INDEX (integration is authoritative; the body specifies a concrete Rust integration test, not a manual proof) — 2026-05-30"
  - "v1.2: ADV-IMPL-P07-MED-001 correction — mechanism prose rewritten: serde_json Map is BTreeMap (preserve_order OFF, no indexmap dep in Cargo.toml/Cargo.lock), so top-level key order is ALPHABETICAL (analyzers, findings, summary), NOT insertion order; phantom test_json_top_level_key_order replaced with real shipped tests from tests/reporter_json_tests.rs; Feasibility Assessment and Source Location updated to match reality — 2026-06-01"
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set (tests/reporter_json_tests.rs)."
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
1. The top-level JSON object keys appear in ALPHABETICAL order
   (`analyzers`, `findings`, `summary`) because `serde_json::json!({...})` at
   `json.rs:48-59` produces a `serde_json::Map<String, Value>`, which is a
   `BTreeMap` when the `preserve_order` feature is OFF (the default). `Cargo.toml`
   declares `serde_json = "1"` with no features; the `indexmap` crate is NOT a
   transitive dependency of serde_json in `Cargo.lock`. Therefore all
   `serde_json::Map` objects -- including the top-level object -- serialize keys in
   lexicographic (alphabetical) order. The actual serialized top-level key order is
   `analyzers`, `findings`, `summary` (alphabetical) -- NOT the insertion order
   (`summary`, `findings`, `analyzers`) written in source.
2. The `protocols` and `services` sub-maps inside `summary` use `BTreeMap`
   (`json.rs:36-45`), so their keys are also in alphabetical order. Both the
   top-level map and the sub-maps share the same BTreeMap-based determinism
   guarantee.
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

The shipped integration tests live in `tests/reporter_json_tests.rs` (STORY-076 / Wave 20,
per DF-AC-TEST-NAME-SYNC-001). Test function names follow PG-W17-001 AC-name sync; they are
cited below by their exact `fn` names as they appear in the source file.

**`test_BC_2_11_001_top_level_keys`** (reporter_json_tests.rs:63-91, BC-2.11.001 pc2)
Verifies that the top-level object contains exactly the keys `summary`, `findings`, and
`analyzers` and no others. The test collects keys from the parsed `serde_json::Map`, sorts
them with `keys.sort_unstable()`, then asserts the sorted slice equals
`["analyzers", "findings", "summary"]` (alphabetical order). The sort-before-assert
pattern is consistent with BTreeMap serialization: the asserted value IS the alphabetical
BTreeMap key order. A set-equality negative assertion also confirms no extra top-level keys exist.

**`test_BC_2_11_001_findings_array_length`** (reporter_json_tests.rs:97-138, BC-2.11.001 pc3)
Verifies that empty, single, and two-Finding slices produce `findings` arrays of the
correct length (0, 1, and 2 respectively).

**`test_BC_2_11_001_summary_subkeys`** (reporter_json_tests.rs:144-167, BC-2.11.001 pc5)
Verifies that all six required sub-keys are present in the `summary` object:
`total_packets`, `total_bytes`, `skipped_packets`, `unique_hosts`, `protocols`, `services`.

**`test_BC_2_11_001_output_is_pretty_printed`** (reporter_json_tests.rs:172-218, BC-2.11.001 pc6)
Verifies pretty-printed output: contains newlines, indented lines starting with spaces,
and the literal substring `\n  "summary"` proving serde_json two-space indentation.

**`test_BC_2_11_003_c0_esc_escaped_in_json`** (reporter_json_tests.rs:279-297, BC-2.11.003 pc1)
Verifies that ESC (0x1B) in a finding summary is serialized as `` in the JSON
wire format, with no raw 0x1B byte in the output.

**`test_BC_2_11_003_c0_roundtrip`** (reporter_json_tests.rs:344-405, BC-2.11.003 pc4)
Verifies that a round-trip (serialize Finding with C0 bytes NUL/BEL/ESC, then parse the
JSON) recovers the original string byte-for-byte. Wire-format assertions confirm each C0
byte is present as its `\uNNNN` escape before parsing, preventing a lenient parser from
masking an incorrectly-unescaped value.

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite test inputs | Determinism is a structural guarantee from serde_json BTreeMap-backed Map |
| Proof complexity | Very low | Property is guaranteed by serde_json BTreeMap (default, no preserve_order feature) for the top-level object, and explicit BTreeMap conversion for protocol/service sub-maps |
| Tool support | High | Standard Rust integration test |
| Estimated proof time | < 1 second | |

## Source Location

`src/reporter/json.rs` -- `JsonReporter::render()` using `serde_json::json!({...})` for
the output value (json.rs:48-59); top-level key order is alphabetical (BTreeMap) because
`preserve_order` is OFF. `BTreeMap` explicitly used for `protocols` and `services`
sub-maps (json.rs:36-45).
`src/findings.rs` -- `#[serde(skip_serializing_if = "Option::is_none")]` on Option fields
(findings.rs:132-145).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | 2026-06-02 | formal-verifier |
| Tests passing | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
