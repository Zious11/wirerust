---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.014: Per-Target Decode Errors Counted into skipped_packets

## Description

When `decode_packet` returns an `Err` for a raw packet, the error is counted but not
propagated. The first decode error prints a warning to stderr; subsequent errors are counted
silently. After the packet loop, `summary.skipped_packets = total_decode_errors`. This
design allows partial captures (with some undecoded packets) to produce results rather than
failing entirely.

## Preconditions

1. `run_analyze` or `run_summary` is processing packets from a PCAP source.
2. At least one packet fails `decode_packet`.

## Postconditions

1. First decode error: `eprintln!("Warning: failed to decode packet ({e}). Further errors
   counted silently.")` is printed to stderr.
2. Subsequent decode errors: counted silently into `total_decode_errors`.
3. After the loop: `summary.skipped_packets = total_decode_errors`.
4. Packet loop continues after each decode error; no early exit.

## Invariants

1. `total_decode_errors == 0` check gates the first-error warning print (main.rs:167-170).
2. The warning is printed at most ONCE per `run_analyze`/`run_summary` invocation,
   regardless of how many decode errors occur.
3. `skipped_packets` is a `u64` counter; overflow in normal usage is not expected.
4. Successfully decoded packets are processed normally; only failed packets are skipped.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero decode errors | skipped_packets=0; no warning |
| EC-002 | One decode error | skipped_packets=1; one warning on stderr |
| EC-003 | Many decode errors | skipped_packets=N; exactly one warning on stderr |
| EC-004 | All packets fail decode | skipped_packets=total; no findings produced |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 valid packets, 0 decode errors | skipped_packets=0 | happy-path |
| 2 valid packets, 1 decode error | skipped_packets=1 | happy-path |
| 0 valid packets, 5 decode errors | skipped_packets=5 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | skipped_packets populated from decode errors | unit: reporter tests exercising skipped_packets via Summary |
| — | Only first error warning printed | unit: suppression logic not directly tested (HIGH for counter; MEDIUM for suppression) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the decode-error handling loop (main.rs:165-173) and the summary.skipped_packets assignment (main.rs:183) are inside run_analyze / run_summary, which are CAP-12's per-target packet-processing loops; counting and suppressing decode errors is an entry-point orchestration responsibility |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-014 (pass-3 ingestion corpus, HIGH confidence for counter; MEDIUM for suppression not directly tested) |

## Related BCs

- BC-2.11.002 -- depends on (skipped_packets populated here flows into JsonReporter output)
- BC-2.11.006 -- depends on (skipped_packets populated here controls TerminalReporter warning line)

## Architecture Anchors

- `src/main.rs:166-173` -- decode error handler with eprintln and counter increment
- `src/main.rs:183` -- summary.skipped_packets = total_decode_errors assignment
- `src/main.rs:266-276` -- same pattern in run_summary

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:166-173, 183` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: reporter tests exercising skipped_packets via Summary construction
- **documentation**: code is explicit; eprintln guards on total_decode_errors == 0

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes warning to stderr (first error only) |
| **Global state access** | none |
| **Deterministic** | yes (given same packet data) |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell |

#### Refactoring Notes

No refactoring needed. The suppression design (print once, count rest) is intentional.
