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

# BC-2.12.010: NO_COLOR Env Var Disables Color

## Description

The `use_color` variable in `main` is computed as `!cli.no_color && std::env::var("NO_COLOR").is_err()`.
When the `NO_COLOR` environment variable is set to ANY value (even empty string), `std::env::var`
returns `Ok(...)` and `is_err()` returns false, so `use_color = false`. This honors the
`NO_COLOR` convention (https://no-color.org/) independently of the `--no-color` flag.

## Preconditions

1. `main()` is entered.
2. `NO_COLOR` may or may not be set in the process environment.

## Postconditions

1. When `NO_COLOR` is set (any value): `use_color = false` regardless of `--no-color`.
2. When `NO_COLOR` is not set AND `--no-color` is absent: `use_color = true`.
3. When `NO_COLOR` is not set AND `--no-color` is present: `use_color = false`.
4. `use_color` is passed to `TerminalReporter { use_color, ... }`.

## Invariants

1. The check is `std::env::var("NO_COLOR").is_err()` at main.rs:43.
2. Any non-empty or even empty value of `NO_COLOR` satisfies the convention (set vs. not-set).
3. `NO_COLOR` and `--no-color` are INDEPENDENT disabling mechanisms; either alone is
   sufficient to disable color.
4. This check runs once per process at startup; changing `NO_COLOR` after `main()` starts
   has no effect.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | NO_COLOR="" (empty) | use_color=false (Ok("") is not an Err) |
| EC-002 | NO_COLOR="1" | use_color=false |
| EC-003 | NO_COLOR not set | use_color depends on --no-color flag only |
| EC-004 | NO_COLOR set AND --no-color present | use_color=false (both disable) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| NO_COLOR="" in env, no --no-color flag | use_color=false | happy-path |
| NO_COLOR not set, no --no-color flag | use_color=true | happy-path |
| NO_COLOR="0", no --no-color | use_color=false (any set value counts) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | NO_COLOR env disables color | unit: serial env-var test (MEDIUM -- env var tests need serial annotation) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the NO_COLOR environment variable check (main.rs:43) is evaluated once at main() entry before subcommand dispatch; resolving use_color is an entry-point startup concern, not a reporter rendering concern |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-010 (pass-3 ingestion corpus, MEDIUM confidence -- env var tests require serial test infrastructure not yet in place) |

## Related BCs

- BC-2.12.003 -- composes with (--no-color flag is the complementary disable mechanism)

## Architecture Anchors

- `src/main.rs:43` -- use_color = !cli.no_color && std::env::var("NO_COLOR").is_err()

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:43` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code is explicit one-liner
- **inferred**: no direct test for env-var behavior; env var tests require serial annotation

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads environment variable |
| **Global state access** | reads global process environment |
| **Deterministic** | no (depends on external env) |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell |

#### Refactoring Notes

To upgrade to HIGH: add a `#[serial]` test that sets `NO_COLOR` in the env and asserts
`use_color = false`. Requires adding the `serial_test` crate as a dev-dependency to prevent
env-var cross-contamination between tests.
