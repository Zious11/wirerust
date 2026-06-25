---
document_type: behavioral-contract
level: L3
version: "1.0"
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

# BC-2.17.023: --enip-write-burst-threshold CLI Flag Configures T0836 Write Detection Sensitivity

## Description

The `--enip-write-burst-threshold` flag (u32, default 50) sets the value of
`EnipAnalyzer.enip_write_burst_threshold`, which is the per-1s-window count of CIP
write-class services (SetAttributesAll, SetAttributeList, SetAttributeSingle) above which
a T0836 finding is emitted (BC-2.17.012). Operators in high-write CIP environments may
increase this threshold to reduce false positives; operators in quiet OT environments may
decrease it. The flag mirrors the DNP3 `--dnp3-direct-operate-threshold` pattern (BC-2.15.017).
The default value of 50 is proposed for typical CIP manufacturing environments; it is
MEDIUM-confidence (un-calibrated, ref O-03) pending human confirmation at F2 gate.

## Preconditions

1. `wirerust analyze [pcap] --enip --enip-write-burst-threshold N` is invoked with N ≥ 1.
2. `EnipAnalyzer` is constructed (i.e., `--enip` or `--all` is set).

## Postconditions

1. `EnipAnalyzer.enip_write_burst_threshold = N` (the parsed u32 value).
2. When `N` is not specified: `enip_write_burst_threshold = 50` (default).
3. The threshold is passed to `process_pdu` / the write-burst detection path (BC-2.17.012).
4. Changing this flag does not affect any other detection BC.

## Invariants

1. **Default = 50**: proposed default for typical CIP manufacturing environments
   (higher-frequency write traffic than Modbus). [MEDIUM-confidence, un-calibrated, ref O-03;
   human to confirm at F2 gate. ADR-010 §Open Items (OA-001 renamed → F2 gate decision).]
2. **u32 type**: the threshold is a u32. Values near `u32::MAX` are accepted by the parser
   but would effectively disable write-burst detection. This is operator responsibility.
3. **Flag independence**: changing `--enip-write-burst-threshold` does not affect T0858,
   T0816, T0888, T0846, or T0814 detection thresholds.
4. **`--all` uses default**: when `--all` expands to include `--enip`, the threshold defaults
   to 50. Users who want a different threshold must specify `--enip-write-burst-threshold`
   explicitly alongside `--all`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--enip-write-burst-threshold 50` | threshold=50 in EnipAnalyzer |
| EC-002 | `--enip-write-burst-threshold 1` | Any single write triggers T0836 |
| EC-003 | No `--enip-write-burst-threshold` flag | Default 50 used |
| EC-004 | `--enip-write-burst-threshold 0` | 0 means every write triggers T0836 instantly — semantically valid but high-noise; [OA-001] |
| EC-005 | `--all --enip-write-burst-threshold 30` | ENIP enabled; threshold=30 |

## Canonical Test Vectors

| CLI flags | threshold used | T0836 fires at write count |
|-----------|---------------|---------------------------|
| `--enip` | 50 | 51st write in 1s window |
| `--enip --enip-write-burst-threshold 5` | 5 | 6th write |
| `--enip --enip-write-burst-threshold 100` | 100 | 101st write |
| `--all` | 50 | 51st write |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | CLI flag parsing, threshold propagation: integration test | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the write-burst threshold flag provides operator control over T0836 detection sensitivity, enabling tuning for specific OT environments (high-write manufacturing vs. quiet substation automation) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-12 (cli.rs, main.rs); SS-17 (analyzer/enip.rs); ADR-010 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — CLI flag; no finding emission) |

## Related BCs

- BC-2.17.012 — depends on (enip_write_burst_threshold is the threshold value for T0836 detection)
- BC-2.17.020 — composes with (--enip-write-burst-threshold is one of the ENIP CLI flags)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { enip_write_burst_threshold: u32 }` — new field with default 50
- `src/analyzer/enip.rs` — `EnipAnalyzer.enip_write_burst_threshold: u32`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 9` (flag specification)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — CLI flag; integration test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 9 (--enip-write-burst-threshold flag spec, default 50); ADR-010 §Open Items (OA-001 / F2 gate decision: default value human confirmation) |
| **Confidence** | high for flag structure; medium for default value (OA-001 open item) |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads CLI arg |
| **Global state access** | sets EnipAnalyzer.enip_write_burst_threshold |
| **Deterministic** | yes |
| **Thread safety** | single-threaded (setup) |
| **Overall classification** | effectful shell (CLI configuration) |
