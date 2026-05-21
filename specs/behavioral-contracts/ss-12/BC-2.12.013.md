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

# BC-2.12.013: Per-Target Progress Bar on stderr via indicatif

## Description

During `run_analyze`, each PCAP file processed gets an `indicatif::ProgressBar` showing
elapsed time, a 40-character bar, and the current/total packet position. The bar is configured
with the template `"[{elapsed_precise}] {bar:40} {pos}/{len} packets"`. The bar is rendered
on stderr (indicatif default). It is finished with `pb.finish_and_clear()` after the file is
processed, leaving no artifacts on stderr.

## Preconditions

1. `run_analyze` is processing a PCAP file.
2. `indicatif::ProgressBar` is successfully constructed with the file's packet count.

## Postconditions

1. A progress bar appears on stderr for each PCAP file during packet processing.
2. The bar increments by 1 for each packet processed (`pb.inc(1)`).
3. The bar is finished and cleared after the file loop completes.
4. Progress bars do NOT appear in the final output string (they go to stderr, not stdout).

## Invariants

1. The progress bar template string is hardcoded at main.rs:150-152.
2. `pb.finish_and_clear()` is always called after the inner loop, even if errors occur.
3. The ProgressBar constructor `ProgressStyle::with_template(...)` is fallible; the `?`
   propagates inside the IIFE closure.
4. `run_summary` has NO progress bar.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PCAP with 0 packets | ProgressBar created with len=0; finish_and_clear immediately |
| EC-002 | Multiple PCAP files | One ProgressBar per file |
| EC-003 | Invalid template string (impossible with hardcoded template) | `?` would propagate |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| (visual only -- progress bar on stderr) | No assertion; cosmetic behavior | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Progress bar does not appear in stdout/final output | manual / visual (LOW confidence -- cosmetic UI, no assertion) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the indicatif ProgressBar is constructed and driven in run_analyze (main.rs:149-177) as part of the per-target packet loop; this is the entry-layer packet-loop orchestration owned by CAP-12, not a reporter rendering concern |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-013 (pass-3 ingestion corpus, LOW confidence -- cosmetic UI, not asserted) |

## Related BCs

- BC-2.12.001 -- composes with (analyze subcommand is where progress bars appear)

## Architecture Anchors

- `src/main.rs:149-177` -- ProgressBar construction and use in packet loop

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:149-177` |
| **Confidence** | low |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code at lines 149-177 is explicit
- **inferred**: no test asserts progress bar behavior; cosmetic UI

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to stderr (indicatif default) |
| **Global state access** | none |
| **Deterministic** | no (elapsed time is non-deterministic) |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell |

#### Refactoring Notes

No change needed. Progress bar behavior is intentionally LOW confidence; asserting specific
ANSI cursor-movement bytes would be fragile and of low value. Recommend keeping as LOW.
