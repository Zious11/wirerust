---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v1.1: F-P3-H1 — VP-044 over-scope: is_valid_iec104_frame is not parse_apci_header; re-anchored to VP-047 per ADR-013 Decision 8. 2026-07-14"
  - "v1.2: A-173-A-01 — gate framing corrected per adversarial advisory: is_valid_iec104_frame reframed as standalone pure predicate (not a wired production dispatch gate); Description/PC-2/PC-3/Invariant-1/Invariant-3/Traceability corrected to reflect DELIVERED design; SEC-001 + F-172-001 + ADR-013 Decisions 1-3 cited. 2026-07-15"
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
input-hash: "0f692ba"
---

# BC-2.19.006: `is_valid_iec104_frame` Standalone Pure Frame-Validity Predicate

## Description

`is_valid_iec104_frame(data: &[u8]) -> bool` is a standalone pure predicate that checks
whether a byte slice begins with a valid IEC-104 APCI start byte (`0x68`) and a LEN byte
in [4, 253]. It is NOT invoked as a production dispatch or caller gate — the equivalent
validation is performed inline in `Iec104Analyzer::on_data`'s frame-walk loop (start-byte
check and LEN-range check), as required by the walk-first residual-bound anti-evasion
semantics (SEC-001, F-172-001, ADR-013 Decisions 2-3). Flows reach `on_data` via
port-2404-based classification (ADR-013 Decision 1), not content-level gating. Wiring this
function as a delivery-level pre-gate would re-open the Ptacek/Newsham evasion hole and
break cross-segment carry. This predicate is unit-tested and serves as the reference
specification for the inline frame-walk validation logic.

## Preconditions

1. `data.len() >= 2` (minimum: start byte + LEN byte visible).

## Postconditions

1. Returns `true` iff `data[0] == 0x68` AND `4 <= data[1] <= 253`.
2. Returns `false` for any other first two bytes; no side effects.
3. No production caller: the equivalent start-byte and LEN validation is performed inline
   in `on_data`'s frame-walk (silent 1-byte resync on bad start byte per ADR-013 Decision 3;
   T0814 emit-then-dedup on malformed LEN per BC-2.19.026). This predicate is unit-tested
   and VP-047-fuzz-covered as a reference specification for those inline checks.

## Invariants

1. **Predicate scope**: validates only bytes 0 and 1; does not fully parse the APCI header.
2. **Consistency with parse_apci_header**: any input where `is_valid_iec104_frame` returns `true`
   and `data.len() >= 6` will cause `parse_apci_header` to succeed (return `Some`).
3. **Not a production gate**: this predicate is NOT invoked in the `on_data` production
   frame-walk path (SEC-001; F-172-001; ADR-013 Decision 1). Its validation logic
   (len>=2, data[0]==0x68, 4<=data[1]<=253) is mirrored inline in `on_data`. Testing this
   function provides reference-specification coverage of those inline checks.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[0] == 0x68`, `data[1] == 4` | Returns `true` |
| EC-002 | `data[0] == 0x68`, `data[1] == 253` | Returns `true` |
| EC-003 | `data[0] != 0x68` | Returns `false` |
| EC-004 | `data[0] == 0x68`, `data[1] == 3` | Returns `false` (LEN < 4) |
| EC-005 | `data[0] == 0x68`, `data[1] == 254` | Returns `false` (LEN > 253) |
| EC-006 | `data.len() == 1` | Returns `false` (cannot read LEN) |

## Canonical Test Vectors

| Input | Expected | Category |
|-------|----------|---------|
| `[0x68, 0x04, ...]` | `true` | valid IEC-104 frame start |
| `[0x48, 0x04, ...]` | `false` | wrong start byte |
| `[0x68, 0xFF, ...]` | `false` | LEN out of range |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | `is_valid_iec104_frame` never panics for any input; returns `true` iff `data[0] == 0x68` and `4 <= data[1] <= 253`; `is_valid_iec104_frame` is not `parse_apci_header` and is outside VP-044 Kani scope per ADR-013 Decision 8 | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — the validity predicate mirrors the inline frame-walk checks in `on_data` and provides reference-specification coverage for IEC-104 start-byte and LEN validation |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decisions 1-3 (port-based classification; walk-first anti-evasion; predicate mirrors inline frame-walk checks — not a production dispatch gate) |
| Feature | feature-iec104 |
| MITRE Techniques | (none — standalone pure predicate; no findings emitted) |

## Related BCs

- BC-2.19.001..005 — composes with (is_valid_iec104_frame summarizes the same two-byte checks as parse_apci_header's first two guards)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn is_valid_iec104_frame(data: &[u8]) -> bool`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 1` — port-based classification; predicate mirrors inline frame-walk checks (not a production dispatch gate)

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all `is_valid_iec104_frame` paths; predicate is outside VP-044 Kani scope per ADR-013 Decision 8)
