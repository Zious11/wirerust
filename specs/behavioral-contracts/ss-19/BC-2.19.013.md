---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "f5a97d3"
---

# BC-2.19.013: TESTFR Keepalive Frames (CF1=0x43/0x83) Produce No Finding

## Description

U-format frames with CF1=0x43 (TESTFR-act) or CF1=0x83 (TESTFR-con) are IEC-104
test-frame keepalives used to verify link-layer connectivity. The passive analyzer
recognizes both values as normal operational frames and produces no security finding.
TESTFR frames contain no ASDU payload. Recognizing TESTFR prevents false positives
where a non-canonical CF1 path (BC-2.19.014) would otherwise be triggered.

## Preconditions

1. A valid U-format APCI frame has been parsed.
2. CF1 is one of: `0x43` (TESTFR-act), `0x83` (TESTFR-con).

## Postconditions

1. No finding is emitted.
2. `Iec104FlowState` is not modified by TESTFR processing.
3. Frame is logged for diagnostics only (at trace level, not as a finding).

## Invariants

1. **No-op for findings**: TESTFR is standard keepalive behavior; emitting a finding for it would produce noise.
2. **LEN constraint**: TESTFR frames have LEN=4; LEN≠4 → T0814 before reaching this path.
3. **Both directions**: TESTFR-act and TESTFR-con may appear on either direction of the TCP flow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TESTFR-act (0x43) on any flow | No finding |
| EC-002 | TESTFR-con (0x83) on any flow | No finding |
| EC-003 | High-frequency TESTFR (keepalive flood) | No finding per frame; aggregate rate anomaly is out of scope for MVP |

## Canonical Test Vectors

| CF1 | Expected Finding | Notes |
|-----|-----------------|-------|
| `0x43` | none | TESTFR-act |
| `0x83` | none | TESTFR-con |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on TESTFR input | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — TESTFR no-op handling prevents false positives in the IEC-104 session state machine |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 5 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — TESTFR is not a threat indicator) |

## Related BCs

- BC-2.19.010 — composes with (STARTDT: other canonical U-frame)
- BC-2.19.011 — composes with (STOPDT: other canonical U-frame)
- BC-2.19.014 — composes with (non-canonical U CF1 → T0814)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if cf1 == 0x43 || cf1 == 0x83 { /* keepalive: no finding */ }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 5`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
