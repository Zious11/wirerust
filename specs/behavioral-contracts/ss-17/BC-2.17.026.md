---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.026: --enip-error-burst-threshold CLI Flag Configures T0888 Error-Burst Detection Sensitivity

## Description

The `--enip-error-burst-threshold` flag (u32, default 5) sets the value of
`EnipAnalyzer.enip_error_burst_threshold`, which is the per-10s-window count of CIP error
responses (any non-zero general_status) above which a T0888 Pattern B finding is emitted
(BC-2.17.014). Operators in environments with naturally high CIP error rates (noisy SCADA
systems, misconfigured PLCs) may increase this threshold to reduce false positives; operators
in quiet OT environments may decrease it. The flag mirrors the `--enip-write-burst-threshold`
pattern (BC-2.17.023). The default value of 5 is proposed as a conservative baseline for
typical CIP environments; it is MEDIUM-confidence (un-calibrated, ref O-03/OA-005) pending
human confirmation at F2 gate.

## Preconditions

1. `wirerust analyze [pcap] --enip --enip-error-burst-threshold N` is invoked with N in 0..=u32::MAX (0 is accepted; see EC-004 / Invariant 4).
2. `EnipAnalyzer` is constructed (i.e., `--enip` or `--all` is set).

## Postconditions

1. `EnipAnalyzer.enip_error_burst_threshold = N` (the parsed u32 value).
2. When `N` is not specified: `enip_error_burst_threshold = 5` (default).
3. The threshold is passed to `process_pdu` / the error-burst detection path (BC-2.17.014 Pattern B).
4. Changing this flag does not affect any other detection BC.

## Invariants

1. **Default = 5**: proposed default for typical CIP environments (conservative baseline for
   error-burst reconnaissance detection). [MEDIUM-confidence, un-calibrated, ref O-03/OA-005;
   human to confirm at F2 gate. ADR-010 §Open Items.]
2. **u32 type**: the threshold is a u32. Values near `u32::MAX` are accepted by the parser
   but would effectively disable error-burst detection. This is operator responsibility.
3. **Strict `>` semantics**: Pattern B fires when `total_error_count > enip_error_burst_threshold`.
   With default 5, the 6th error response within the 10s window fires the finding; 5 errors do
   NOT fire (same strict `>` convention as BC-2.17.012 and BC-2.17.014 — one comparison
   semantic throughout the analyzer).
4. **Zero-value semantics**: `--enip-error-burst-threshold 0` means every first error response
   in a 10s window immediately fires a T0888 Pattern B finding (0 > 0 is false, but the first
   error with total_error_count=1 satisfies 1 > 0). Semantically valid but high-noise; operator
   responsibility.
5. **Flag independence**: changing `--enip-error-burst-threshold` does not affect T0836, T0858,
   T0816, T0846, or T0814 detection thresholds.
6. **`--all` uses default**: when `--all` expands to include `--enip`, the threshold defaults
   to 5. Users who want a different threshold must specify `--enip-error-burst-threshold`
   explicitly alongside `--all`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--enip-error-burst-threshold 5` | threshold=5 in EnipAnalyzer |
| EC-002 | `--enip-error-burst-threshold 1` | 2nd error in 10s window triggers T0888 Pattern B |
| EC-003 | No `--enip-error-burst-threshold` flag | Default 5 used |
| EC-004 | `--enip-error-burst-threshold 0` | First error (count=1) triggers T0888 Pattern B instantly — semantically valid but high-noise; operator responsibility |
| EC-005 | `--all --enip-error-burst-threshold 10` | ENIP enabled; error-burst threshold=10 |

## Canonical Test Vectors

| CLI flags | threshold used | T0888 Pattern B fires at error count |
|-----------|---------------|--------------------------------------|
| `--enip` | 5 | 6th error in 10s window |
| `--enip --enip-error-burst-threshold 5` | 5 | 6th error |
| `--enip --enip-error-burst-threshold 1` | 1 | 2nd error |
| `--enip --enip-error-burst-threshold 10` | 10 | 11th error |
| `--all` | 5 | 6th error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | CLI flag parsing, threshold propagation: integration test | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the error-burst threshold flag provides operator control over T0888 Pattern B detection sensitivity, enabling tuning for specific OT environments (noisy SCADA vs. quiet substation automation) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-12 (cli.rs, main.rs); SS-17 (analyzer/enip.rs); ADR-010 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — CLI flag; no finding emission) |

## Related BCs

- BC-2.17.014 — configures T0888 Pattern B detection (enip_error_burst_threshold is the threshold value used by Pattern B error-burst detection)
- BC-2.17.020 — composes with (--enip-error-burst-threshold is one of the ENIP CLI flags)
- BC-2.17.023 — symmetric sibling (write-burst CLI flag for T0836; mirrors this pattern)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { enip_error_burst_threshold: u32 }` — new field with default 5
- `src/analyzer/enip.rs` — `EnipAnalyzer.enip_error_burst_threshold: u32`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 9` (flag specification)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — CLI flag; integration test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 9 (--enip-error-burst-threshold flag spec, symmetric with --enip-write-burst-threshold); ADR-010 §Open Items (OA-005 / F2 gate decision: default value human confirmation) |
| **Confidence** | high for flag structure; medium for default value (OA-005 open item; MEDIUM-confidence, un-calibrated, ref O-03) |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads CLI arg |
| **Global state access** | sets EnipAnalyzer.enip_error_burst_threshold |
| **Deterministic** | yes |
| **Thread safety** | single-threaded (setup) |
| **Overall classification** | effectful shell (CLI configuration) |
