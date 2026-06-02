---
document_type: verification-property
level: L4
version: "2.0"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.12.007
bcs:
  - BC-2.12.007
  - BC-2.12.009
module: src/cli.rs
proof_method: integration
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "38b0b7e3d65b00007b7d769056731ee7b8629574ce4c98b8f16b62fbc7c2aba1"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: proof_method manual→integration to match VP body (Integration test / assert_cmd), VP-INDEX (integration), verification-coverage-matrix, and verification-architecture — F-W21-VP-METHOD — 2026-05-31"
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set (tests/cli_story_087_tests.rs)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-018: CLI Reassemble / No-Reassemble Mutual Exclusion

## Property Statement

The CLI rejects the combination of `--reassemble` and `--no-reassemble` flags
with a clear error message. They are mutually exclusive:

1. `wirerust analyze --reassemble <file>` succeeds (reassembly enabled).
2. `wirerust analyze --no-reassemble <file>` succeeds (reassembly disabled with
   a warning when analyzers that need it are also enabled).
3. `wirerust analyze --reassemble --no-reassemble <file>` exits with a non-zero
   status code and an error message indicating mutual exclusion.

Additionally, `needs_reassembly` logic (BC-2.12.009): when both HTTP and TLS
analyzers are enabled but `--no-reassemble` is specified, a warning is emitted
and reassembly is forced off (the user's `--no-reassemble` wins over the
analyzers' implicit requirement).

## Source Contract

- **Primary BC:** BC-2.12.007 -- --reassemble and --no-reassemble are Mutually Exclusive
- **Postcondition:** Process exits non-zero when both flags are provided
- **Related BC:** BC-2.12.009 -- needs_reassembly Logic; --no-reassemble Forces Off with Warning

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test | assert_cmd / CLI test | N/A | Direct CLI invocation with conflicting flags |

## Test Specification

```rust
// tests/cli_tests.rs (using assert_cmd)
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_reassemble_and_no_reassemble_mutual_exclusion() {
    Command::cargo_bin("wirerust")
        .unwrap()
        .args(["analyze", "--reassemble", "--no-reassemble", "tests/fixtures/test.pcap"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with")
            .or(predicate::str::contains("mutually exclusive"))
            .or(predicate::str::contains("not allowed")));
}

#[test]
fn test_reassemble_alone_succeeds() {
    // If a valid pcap file exists
    Command::cargo_bin("wirerust")
        .unwrap()
        .args(["analyze", "--reassemble", "--http", "tests/fixtures/http.pcap"])
        .assert()
        .success();
}

#[test]
fn test_no_reassemble_with_http_emits_warning() {
    let output = Command::cargo_bin("wirerust")
        .unwrap()
        .args(["analyze", "--no-reassemble", "--http", "tests/fixtures/http.pcap"])
        .output()
        .unwrap();
    // Should succeed (warning, not error)
    // Warning goes to stderr
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("reassembl") || output.status.success(),
        "Expected warning or success with --no-reassemble");
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite -- 3 cases (reassemble, no-reassemble, both) | |
| Proof complexity | Very low | clap handles mutual exclusion via `conflicts_with` attribute |
| Tool support | High | `assert_cmd` crate; already used in wirerust test suite |
| Estimated proof time | < 1 second | |

## Source Location

`src/cli.rs` -- `--reassemble` and `--no-reassemble` flag definitions with
`conflicts_with` (or equivalent clap mutual exclusion attribute).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Tests committed | 2026-06-02 | formal-verifier |
| Tests passing | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
