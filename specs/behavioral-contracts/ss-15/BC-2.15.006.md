---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.1: Pass-1 adversarial fix I-3: corrected stale cross-reference from BC-2.15.016 to BC-2.15.014 in Postcondition 9 header, Invariant 5, and Related BCs. The Response-class request/response correlation for T1691.001 inference lives in BC-2.15.014 (block-command inference), not BC-2.15.016 (per-flow state and carry buffer). — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/phase-f2-spec-evolution/dnp3-verification-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: TBD
---

# BC-2.15.006: FC Classification Correctness — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01}

## Description

`classify_dnp3_fc(fc: u8) -> Dnp3FcClass` correctly maps the four detection-critical FC sets
to their exact `Dnp3FcClass` variants. The Control set {SELECT=0x03, OPERATE=0x04,
DIRECT_OPERATE=0x05, DIRECT_OPERATE_NR=0x06} maps to `Dnp3FcClass::Control`; the Restart set
{COLD_RESTART=0x0D, WARM_RESTART=0x0E} maps to `Dnp3FcClass::Restart`; WRITE=0x02 maps to
`Dnp3FcClass::Write`; READ=0x01 maps to `Dnp3FcClass::Read`. The Response set {0x81, 0x82,
0x83} maps to `Dnp3FcClass::Response`. These mappings are verified by VP-023 Sub-property B
set-membership assertions over all 256 FC values.

## Preconditions

1. `fc` is any `u8` value.
2. `classify_dnp3_fc` is the pure core function from BC-2.15.005 (total, no panic).

## Postconditions

**Control set** (FCs triggering T1692.001 unauthorized control detection):
1. `classify_dnp3_fc(0x03)` returns `Dnp3FcClass::Control` (SELECT) [SPEC: dnp3-research.md §3.2]
2. `classify_dnp3_fc(0x04)` returns `Dnp3FcClass::Control` (OPERATE) [SPEC]
3. `classify_dnp3_fc(0x05)` returns `Dnp3FcClass::Control` (DIRECT_OPERATE) [SPEC]
4. `classify_dnp3_fc(0x06)` returns `Dnp3FcClass::Control` (DIRECT_OPERATE_NR) [SPEC]

**Restart set** (FCs triggering T0814 denial-of-service detection):
5. `classify_dnp3_fc(0x0D)` returns `Dnp3FcClass::Restart` (COLD_RESTART) [SPEC: dnp3-research.md §3.2]
6. `classify_dnp3_fc(0x0E)` returns `Dnp3FcClass::Restart` (WARM_RESTART) [SPEC]

**Write set** (FC triggering T0836 modify-parameter detection):
7. `classify_dnp3_fc(0x02)` returns `Dnp3FcClass::Write` (WRITE) [SPEC: dnp3-research.md §3.2]

**Read set** (non-detection; used for FC distribution statistics):
8. `classify_dnp3_fc(0x01)` returns `Dnp3FcClass::Read` (READ) [SPEC]

**Response set** (used for request/response correlation in BC-2.15.014):
9. `classify_dnp3_fc(0x81)` returns `Dnp3FcClass::Response` (RESPONSE) [SPEC]
10. `classify_dnp3_fc(0x82)` returns `Dnp3FcClass::Response` (UNSOLICITED_RESPONSE) [SPEC]
11. `classify_dnp3_fc(0x83)` returns `Dnp3FcClass::Response` (AUTHENTICATE_RESP) [SPEC]

## Invariants

1. **Control set is contiguous** {0x03, 0x04, 0x05, 0x06} — SELECT through DIRECT_OPERATE_NR.
   A `match` arm `0x03..=0x06 => Control` implements this cleanly. No value in 0x03..=0x06
   should map to any class other than `Control`. [SPEC: dnp3-research.md §3.2, all four confirmed]
2. **Restart set is {0x0D, 0x0E}** — two non-contiguous values in the lower FC range.
   COLD_RESTART and WARM_RESTART. No other FC triggers T0814. [SPEC]
3. **Write is 0x02 only** — WRITE is a single FC. All other write-related FCs (FREEZE, etc.) map
   to `Management`, not `Write`. The separation is intentional: T0836 maps to FC 0x02 only.
4. **DIRECT_OPERATE_NR (0x06)** maps to `Control` even though it expects no response. This is
   correct: it still triggers T1692.001 if unauthorized. It does NOT trigger T1691.001
   request-without-response inference (see BC-2.15.014 edge case EC-001).
5. **Response set**: 0x81/0x82/0x83 are used for request/response correlation (BC-2.15.014)
   and unsolicited-response anomaly (BC-2.15.019). All three map to `Dnp3FcClass::Response`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x03 (SELECT) | `Control` — first element of the SBO control pair |
| EC-002 | FC = 0x04 (OPERATE) | `Control` — second element of the SBO control pair |
| EC-003 | FC = 0x05 (DIRECT_OPERATE) | `Control` — one-shot control with response expected |
| EC-004 | FC = 0x06 (DIRECT_OPERATE_NR) | `Control` — one-shot control, NO response expected; still `Control` class |
| EC-005 | FC = 0x07 (IMMED_FREEZE) | `Management` — freeze is not a Control-class FC for detection purposes |
| EC-006 | FC = 0x0C (FREEZE_AT_TIME_NR) | `Management` — freeze variant, not Control or Restart |
| EC-007 | FC = 0x0D (COLD_RESTART) | `Restart` — full device restart; triggers T0814 |
| EC-008 | FC = 0x0E (WARM_RESTART) | `Restart` — partial restart; triggers T0814 |
| EC-009 | FC = 0x0F (INITIALIZE_DATA) | `Management` — NOT Restart (INITIALIZE_DATA is a separate operation) |
| EC-010 | FC = 0x02 (WRITE) | `Write` — triggers T0836; single FC only |
| EC-011 | FC = 0x82 (UNSOLICITED_RESPONSE) | `Response` — triggers unsolicited-anomaly check (BC-2.15.019) |

## Canonical Test Vectors

| FC (hex) | FC name | Expected `Dnp3FcClass` | MITRE technique | Category |
|----------|---------|----------------------|----------------|----------|
| `0x01` | READ | `Read` | (none) | happy-path: polling |
| `0x02` | WRITE | `Write` | T0836 (see BC-2.15.012) | happy-path: modify parameter |
| `0x03` | SELECT | `Control` | T1692.001 (see BC-2.15.010) | happy-path: SBO select |
| `0x04` | OPERATE | `Control` | T1692.001 | happy-path: SBO operate |
| `0x05` | DIRECT_OPERATE | `Control` | T1692.001 | happy-path: direct control |
| `0x06` | DIRECT_OPERATE_NR | `Control` | T1692.001 | happy-path: direct control NR |
| `0x07` | IMMED_FREEZE | `Management` | (none) | happy-path: management |
| `0x0D` | COLD_RESTART | `Restart` | T0814 (see BC-2.15.011) | happy-path: cold restart |
| `0x0E` | WARM_RESTART | `Restart` | T0814 | happy-path: warm restart |
| `0x81` | RESPONSE | `Response` | (correlation only) | happy-path: solicited response |
| `0x82` | UNSOLICITED_RESPONSE | `Response` | T0814 / anomaly check | happy-path: unsolicited |
| `0x83` | AUTHENTICATE_RESP | `Response` | (none) | happy-path: auth response |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property B (set membership): Control set {0x03-0x06} → `Control`; Restart set {0x0D,0x0E} → `Restart`; Write {0x02} → `Write`; Read {0x01} → `Read`; Response {0x81,0x82,0x83} → `Response` — verified for all 256 FC values | Kani: `fc: u8 = kani::any()`; per-set `if matches!(fc, N) { assert!(class == Variant); }` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — correct FC-to-class mapping is the direct prerequisite for all MITRE technique detection in the DNP3/ICS analyzer; a misclassified Control FC produces no T1692.001 finding (false negative); a misclassified Management FC produces a spurious T1692.001 finding (false positive) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — correct FC classification ensures findings are emitted only for the correct protocol operations) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure classification function; detection BCs BC-2.15.010–019 are the emitters) |

## Related BCs

- BC-2.15.005 — composes with (this BC proves correctness; BC-2.15.005 proves totality — both together fully specify the classifier)
- BC-2.15.010 — depends on (T1692.001 unauthorized control detection uses `classify_dnp3_fc` returning `Control`)
- BC-2.15.011 — depends on (T0814 restart detection uses `classify_dnp3_fc` returning `Restart`)
- BC-2.15.012 — depends on (T0836 write detection uses `classify_dnp3_fc` returning `Write`)
- BC-2.15.014 — depends on (T1691.001 block-command inference uses `Response` class for correlation)
- BC-2.15.019 — depends on (unsolicited anomaly uses `Response` class for FC 0x82)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass` — explicit arms for 0x01..=0x06, 0x0D, 0x0E, 0x81..=0x83; management FCs; wildcard
- `src/analyzer/dnp3.rs` — `enum Dnp3FcClass` — seven variants
- `.factory/research/dnp3-research.md §3.2` — confirmed FC table: READ=0x01, WRITE=0x02, SELECT=0x03..DIRECT_OPERATE_NR=0x06, COLD_RESTART=0x0D, WARM_RESTART=0x0E, RESPONSE=0x81, UNSOLICITED_RESPONSE=0x82, AUTHENTICATE_RESP=0x83 [SPEC]

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property B: set-membership correctness)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §3.2 (FC table, all values confirmed against CISA icsnpp-dnp3, Suricata, Wireshark); dnp3-architecture-delta.md §3 (Dnp3FcClass enum); ADR-007 Decision 5 (technique-to-FC mapping) |
| **Confidence** | high — all FC hex values confirmed [SPEC] against CISA icsnpp-dnp3 constants.zeek, Suricata dnp3_func keyword list, and Wireshark dissector |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same FC byte always produces same variant |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-B, set membership) |
