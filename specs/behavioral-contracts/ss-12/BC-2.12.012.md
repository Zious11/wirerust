---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.012: Non-Existent Target Yields bail! with Target Not Found

## Description

`resolve_targets` returns an `anyhow::Error` via `bail!("Target not found: {}", target.display())`
when a target path is neither a regular file nor a directory. This is the final else-branch
of the `is_file()` / `is_dir()` check. The error propagates through `?` back to the
outer pipeline in `run_analyze` / `run_summary`.

## Preconditions

1. `resolve_targets` is called with a `target` PathBuf that does not exist as a file or
   directory (e.g., the path does not exist at all).

## Postconditions

1. Returns `Err(anyhow::Error)` with message matching
   `"Target not found: <path-display>"`.
2. The error propagates via `?` up to `run_analyze` / `run_summary`.
3. In `run_analyze`, the error is captured in the IIFE `capture_result` and eventually
   returned after `finalize()` completes (main.rs:193).

## Invariants

1. The bail message format is `"Target not found: {}"` using `target.display()`.
2. The error is NOT printed to stderr directly by `resolve_targets`; it is an `Err` return.
3. Callers use `?` to propagate; printing (if any) happens at the top-level error handler.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Path to a non-existent file | Err: "Target not found: /path/to/nonexistent.pcap" |
| EC-002 | Dangling symlink (neither file nor dir) | Err: "Target not found: ..." |
| EC-003 | Valid file | Ok(vec![file]) |
| EC-004 | Valid directory | Ok(expanded list) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| target = "/nonexistent/path.pcap" | Err with "Target not found" message | error |
| target = "existing.pcap" | Ok([path]) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Missing target returns bail! error | unit: resolve_targets test with tempdir (MEDIUM -- not directly tested) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the bail! on an invalid target path (main.rs:359) is part of resolve_targets, which is CAP-12's target-resolution step; input validation at the entry point before any packet reading is precisely the orchestration concern CAP-12 owns |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-012 (pass-3 ingestion corpus, MEDIUM confidence -- behavior is explicit in code; no unit test) |

## Related BCs

- BC-2.12.011 -- composes with (directory expansion is the other branch of resolve_targets)

## Architecture Anchors

- `src/main.rs:359` -- `anyhow::bail!("Target not found: {}", target.display())`

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:359` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: bail! macro at line 359 is explicit

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads filesystem metadata |
| **Global state access** | none |
| **Deterministic** | yes (given same filesystem state) |
| **Thread safety** | N/A |
| **Overall classification** | effectful shell |

#### Refactoring Notes

To upgrade to HIGH: use `tempdir` in a test to verify that a non-existent path produces
an Err with the expected message prefix.
