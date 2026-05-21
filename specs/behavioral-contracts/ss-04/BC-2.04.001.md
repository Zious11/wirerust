---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/mod.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.001: TcpReassembler::new Panics on Invalid Config

## Description

`TcpReassembler::new(config)` validates five fields of `ReassemblyConfig` at construction
time via `assert!`. If any field is zero -- `max_depth`, `memcap`, `max_flows`,
`max_segments_per_direction`, or `max_receive_window` -- the constructor panics immediately
with a descriptive message. A zero value in any of these fields would cause silent incorrect
behavior downstream (division or subtraction from zero), so early panicking is the intended
policy. Valid configs always produce a zero-finding, zero-flow, un-finalized instance.

## Preconditions

1. A `ReassemblyConfig` value is supplied (may be default or custom).
2. The caller is on the happy path (expected to provide valid config derived from CLI defaults
   or explicit construction).

## Postconditions

1. If `config.max_depth == 0`: panics with message containing "max_depth must be > 0".
2. If `config.memcap == 0`: panics with message containing "memcap must be > 0".
3. If `config.max_flows == 0`: panics with message containing "max_flows must be > 0".
4. If `config.max_segments_per_direction == 0`: panics with message containing
   "max_segments_per_direction must be > 0".
5. If `config.max_receive_window == 0`: panics with message containing
   "max_receive_window must be > 0".
6. If all five fields are > 0: returns `TcpReassembler` with empty flows, empty findings,
   `total_memory == 0`, and `finalized == false`.

## Invariants

1. The panic messages are fixed string literals; the check is a Rust `assert!`, not a
   recoverable `Result`.
2. Other config fields (`flow_timeout_secs`, threshold fields) are NOT validated at
   construction; zero values in those fields are legal.
3. The five validated fields map directly to the bounds arithmetic in `insert_segment` and
   `evict_flows` -- a zero value would cause saturating-sub underflow or infinite loops.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | max_depth = 0, all others valid | panic: "max_depth must be > 0" |
| EC-002 | memcap = 0, all others valid | panic: "memcap must be > 0" |
| EC-003 | max_flows = 0 | panic: "max_flows must be > 0" |
| EC-004 | max_segments_per_direction = 0 | panic: "max_segments_per_direction must be > 0" |
| EC-005 | max_receive_window = 0 | panic: "max_receive_window must be > 0" |
| EC-006 | ReassemblyConfig::default() (all > 0) | Returns valid TcpReassembler; no panic |
| EC-007 | flow_timeout_secs = 0 | No panic; legal value |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ReassemblyConfig::default() | Valid TcpReassembler; flows.is_empty(), findings.is_empty() | happy-path |
| Config with max_depth=0 | panic("max_depth must be > 0") | error |
| Config with max_flows=0 | panic("max_flows must be > 0") | error |
| Config with max_receive_window=0 | panic("max_receive_window must be > 0") | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All five invalid configs cause panic | unit: #[should_panic(expected = "...")] for each |
| — | Valid config (all > 0) never panics | unit: new with min valid config (all fields = 1) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- constructor validation is the entry invariant for the reassembly engine |
| L2 Domain Invariants | None directly (pre-invariant guard) |
| Architecture Module | SS-04 (reassembly/mod.rs:107-127, C-6) |
| Stories | S-TBD |
| Origin BC | BC-RAS-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.015 -- depends on (max_flows validated here; eviction uses it)
- BC-2.04.016 -- depends on (memcap validated here; eviction uses it)
- BC-2.04.041 -- depends on (max_depth validated here; depth truncation uses it)

## Architecture Anchors

- `src/reassembly/mod.rs:107-127` -- five assert! calls in TcpReassembler::new
- `src/reassembly/config.rs` -- ReassemblyConfig field definitions

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:107-127` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: five `assert!` macro calls with literal messages

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (construction; no shared state) |
| **Overall classification** | pure (until panic path) |

## Refactoring Notes

No refactoring needed. The panic-on-invalid-config pattern is idiomatic Rust for programming
errors that should never occur in production (CLI validation upstream prevents zero values).
