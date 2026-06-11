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
modified: []
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

# BC-2.15.005: classify_dnp3_fc Is Total Over All 256 FC Values (No Gap, No Panic)

## Description

`classify_dnp3_fc(fc: u8) -> Dnp3FcClass` classifies any application function code byte into
one of seven defined variants. The function is total: it returns exactly one defined variant
for every possible `u8` input value (all 256 values). There is no `unreachable!()` macro, no
gap in the match, and no panic path. Totality is guaranteed by a wildcard `_ => Unknown` arm
that covers all values not explicitly matched. VP-023 Sub-property B proves this formally over
the complete 256-value `u8` domain via Kani.

## Preconditions

1. `fc` is any `u8` value (0x00..=0xFF, all 256 possible values).
2. No precondition on the source of `fc`; the function is pure and accepts any byte.

## Postconditions

1. `classify_dnp3_fc(fc)` returns exactly one value from the set
   `{Read, Write, Control, Restart, Management, Response, Unknown}`.
2. The function NEVER panics for any `fc` value in 0x00..=0xFF.
3. The function NEVER returns an undefined/uninitialized variant.
4. No two different `Dnp3FcClass` variants are returned for the same `fc` input (deterministic).

## Invariants

1. **Totality via wildcard arm**: the `match fc { ... _ => Dnp3FcClass::Unknown }` wildcard arm
   ensures every `u8` value is covered. There is no `unreachable!()` call. [VP-023 Sub-B]
2. **Determinism**: the same `fc` byte always produces the same `Dnp3FcClass` variant. The
   function is pure (no I/O, no global state).
3. **Seven defined variants**:
   - `Dnp3FcClass::Read` — covers FC 0x01
   - `Dnp3FcClass::Write` — covers FC 0x02
   - `Dnp3FcClass::Control` — covers FCs 0x03, 0x04, 0x05, 0x06
   - `Dnp3FcClass::Restart` — covers FCs 0x0D, 0x0E
   - `Dnp3FcClass::Management` — covers known management FCs (freeze, config, time, file, auth)
   - `Dnp3FcClass::Response` — covers FCs 0x81, 0x82, 0x83
   - `Dnp3FcClass::Unknown` — covers all remaining values

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x00 (CONFIRM) | Returns `Management` (app-layer confirmation is a management frame) or `Unknown` depending on F3 implementation choice; this BC only specifies totality, not CONFIRM's exact class — see BC-2.15.006 for set membership |
| EC-002 | FC = 0xFF (reserved/undefined) | Returns `Unknown` — wildcard arm |
| EC-003 | FC = 0x80 (reserved range) | Returns `Unknown` — wildcard arm |
| EC-004 | FC = 0x7F (undefined) | Returns `Unknown` — wildcard arm |
| EC-005 | FC = 0x81 (RESPONSE) | Returns `Response` — explicit arm |
| EC-006 | FC = 0x83 (AUTHENTICATE_RESP) | Returns `Response` — explicit arm |
| EC-007 | All 256 values | Each returns a variant from the defined seven-variant set |

## Canonical Test Vectors

| FC (hex) | FC name | Expected `Dnp3FcClass` | Category |
|----------|---------|----------------------|----------|
| `0x01` | READ | `Read` | happy-path: read class |
| `0x02` | WRITE | `Write` | happy-path: write class |
| `0x05` | DIRECT_OPERATE | `Control` | happy-path: control class |
| `0x0D` | COLD_RESTART | `Restart` | happy-path: restart class |
| `0x81` | RESPONSE | `Response` | happy-path: response class |
| `0x82` | UNSOLICITED_RESPONSE | `Response` | happy-path: unsolicited response |
| `0xFF` | (reserved) | `Unknown` | edge-case: wildcard arm fires |
| `0x00` | CONFIRM | `Management` | happy-path: management |

**Totality witness**: for every value in `0x00..=0xFF`, a defined variant is returned. Kani
proves this by attempting `classify_dnp3_fc(fc: u8 = kani::any())` returns one of the seven
variants and never panics.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property B (totality): `classify_dnp3_fc(fc)` returns a defined `Dnp3FcClass` variant for all `fc: u8`; never panics; no `unreachable!` | Kani: `fc: u8 = kani::any()` (all 256 values); assertion that returned variant is one of the seven defined variants |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — `classify_dnp3_fc` totality is the foundational safety guarantee for all FC-based detection logic in the DNP3/ICS analyzer; a non-total classifier can panic on unexpected FC bytes in real ICS traffic, causing analyzer failure |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — panic freedom in the pure-core classifier ensures analyzer stability on adversarial traffic) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure classification function; no finding emission) |

## Related BCs

- BC-2.15.006 — composes with (this BC proves totality; BC-2.15.006 proves correctness of specific set membership)
- BC-2.15.010 through BC-2.15.013 — all detection BCs that branch on `Dnp3FcClass` depend on totality guarantee

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass` — wildcard `_ => Dnp3FcClass::Unknown` arm
- `src/analyzer/dnp3.rs` — `enum Dnp3FcClass { Read, Write, Control, Restart, Management, Response, Unknown }`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §3` — "Total over all 256 u8 values (VP-023 Sub-property B)"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property B: totality)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-architecture-delta.md §3; vp-023-dnp3-parse-safety.md Sub-property B; dnp3-research.md §3.2 (FC table) |
| **Confidence** | high — totality is a structural property of the `match` construct with `_ =>` wildcard; set membership values confirmed by dnp3-research.md §3.2 against CISA icsnpp-dnp3 constants and Wireshark |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same FC byte always produces same variant |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-B, totality) |
