---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/segment.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
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

# BC-2.04.032: insert_segment With No ISN Returns IsnMissing; Inserts Nothing

## Description

When `FlowDirection::insert_segment` is called on a direction whose `isn` field is `None`,
it emits a one-shot process-wide `eprintln!` (via `ISN_MISSING_WARNED` atomic), then returns
`InsertResult::IsnMissing` without inserting any data, modifying any counters, or panicking.
This is classified as a programming error: callers (the engine) should always ensure the ISN
is set before invoking `insert_segment`.

## Preconditions

1. `self.isn == None` (no SYN or infer_isn call has occurred for this direction).
2. `data` is non-empty (empty segments return `Inserted` immediately before the ISN check).

## Postconditions

1. Returns `InsertResult::IsnMissing`.
2. `self.segments` is unchanged (nothing inserted).
3. `self.buffered_bytes` is unchanged.
4. `self.overlap_count`, `self.out_of_window_count`, and all other counters are unchanged.
5. If `ISN_MISSING_WARNED` was `false`: set to `true`; `eprintln!("wirerust: insert_segment called with no ISN set")` fires.
6. If `ISN_MISSING_WARNED` was already `true`: silent return with no eprintln.

## Invariants

1. `ISN_MISSING_WARNED` is a `static AtomicBool` (segment.rs:16); once set to `true` it is
   never reset within the process lifetime (per ADR 0004 one-shot warning pattern).
2. The IsnMissing result is a programming-error signal, not an expected runtime condition.
   The engine in `insert_payload_segment` (mod.rs:306-319) should have set the ISN before
   calling insert_segment.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First IsnMissing encounter | eprintln fires; ISN_MISSING_WARNED set to true |
| EC-002 | Second IsnMissing encounter in same process | Silent return; no second eprintln |
| EC-003 | Empty data slice with no ISN | Returns Inserted early (before ISN check); no warning |
| EC-004 | ISN set to Some(x) | Normal insertion path; IsnMissing not returned |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| insert_segment with isn=None, data=b"hello" | IsnMissing; segments empty; warned once | happy-path |
| insert_segment with isn=None, data=b"" | Inserted (empty early return, no ISN check triggered) | edge-case |
| insert_segment with isn=Some(0), data=b"hello" | Inserted (normal path) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IsnMissing inserts nothing and leaves buffered_bytes unchanged | unit: test_isn_missing_returns_isn_missing |
| VP-TBD | IsnMissing never panics | unit + manual review |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- IsnMissing is the segment insertion contract's programming-error defensive guard |
| L2 Domain Invariants | INV-6 (IsnMissing does not consume findings capacity; it is not a forensic finding) |
| Architecture Module | SS-04 (reassembly/segment.rs:51-58, C-8) |
| Stories | S-TBD |
| Origin BC | BC-RAS-032 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.031 -- depends on (ISN must be set by set_isn or infer_isn before insert_segment)
- BC-2.04.048 -- composes with (ISN_MISSING_WARNED atomic -- this BC describes its triggering condition)

## Architecture Anchors

- `src/reassembly/segment.rs:16` -- ISN_MISSING_WARNED AtomicBool
- `src/reassembly/segment.rs:51-58` -- IsnMissing guard in insert_segment

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:51-58` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_isn_missing_returns_isn_missing verifies the return value
- **guard clause**: `let isn = match self.isn { Some(isn) => isn, None => { ... return IsnMissing } }`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to stderr on first call |
| **Global state access** | reads + writes ISN_MISSING_WARNED (AtomicBool) |
| **Deterministic** | no -- depends on prior process state (atomic) |
| **Thread safety** | atomic access is thread-safe |
| **Overall classification** | effectful shell (side-effectful stderr write on first encounter) |
