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

# BC-2.17.003: is_valid_enip_frame Validity Gate Biconditional — Known-Command Set

## Description

`is_valid_enip_frame(h: &EnipHeader) -> bool` is the post-classification validity gate for
EtherNet/IP frames. It returns `true` if and only if `h.command` is a member of the
9-value known-command set: {0x0004 ListServices, 0x0063 ListIdentity, 0x0064 ListInterfaces,
0x0065 RegisterSession, 0x0066 UnRegisterSession, 0x006F SendRRData, 0x0070 SendUnitData,
0x0072 IndicateStatus, 0x0075 Cancel}. For any command value outside this set, the function
returns `false`. The biconditional holds for all 65,536 possible `u16` command values and is
formally verified by VP-032 Sub-C. Frames that fail this gate increment `parse_errors` and
`malformed_in_window` in `EnipFlowState` (see BC-2.17.018).

## Preconditions

1. `h` is an `EnipHeader` produced by `parse_enip_header` (BC-2.17.002).
2. `h.command` is a `u16` — all 65,536 values are valid inputs.

## Postconditions

1. `is_valid_enip_frame(h)` returns `true` if and only if `h.command ∈ {0x0004, 0x0063,
   0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075}`.
2. For any `h.command` not in the known set, returns `false`.
3. The function does not inspect any field other than `h.command` (length, status, options
   are not gate criteria at this level).
4. The function is pure: no I/O, no state mutation, terminates for all inputs.

## Invariants

1. **Biconditional**: the gate is an exact equivalence — neither a superset nor a subset.
   A command is valid if and only if ODVA has assigned it a normative meaning in the
   encapsulation layer. [SPEC: ODVA EtherNet/IP Specification Table 2-3; ADR-010 Decision 2]
2. **Totality**: every possible `u16` command value produces a defined `bool` return. There
   is no panic path. The `Unknown` arm in `classify_enip_command` (BC-2.17.004) captures
   the values for which `is_valid_enip_frame` returns `false`.
3. **Command-only gate**: the validity gate does not check `h.status` or `h.options`. Status
   field validation (non-zero status in responses) is a separate detection concern.
4. **Post-classification purpose**: this gate is the compensating control for port-only
   classification (ADR-010 Decision 1). Any non-ENIP binary protocol on port 44818 that
   happens to produce 24 bytes where bytes 0–1 are not in the known-command set is rejected
   here. This prevents false positive findings on mis-routed traffic.
5. **Known-command set is v0.11.0-fixed**: the set of 9 commands is normative ODVA and does
   not change at runtime. No configuration or runtime update modifies this set.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `command = 0x006F` (SendRRData) | Returns `true` — primary explicit messaging command |
| EC-002 | `command = 0x0063` (ListIdentity) | Returns `true` — recon command (T0846 detection path) |
| EC-003 | `command = 0x0000` (not in known set) | Returns `false` — unknown; frame rejected |
| EC-004 | `command = 0xFFFF` (not in known set) | Returns `false` — unknown; frame rejected |
| EC-005 | `command = 0x0071` (between 0x0070 and 0x0072, not assigned) | Returns `false` — gap in ODVA command table |
| EC-006 | `command = 0x0075` (Cancel) | Returns `true` — boundary of known set; included |
| EC-007 | `command = 0x0076` (above 0x0075, not assigned) | Returns `false` — above highest known command |
| EC-008 | All other fields zeroed, `command = 0x0065` (RegisterSession) | Returns `true` — command-only check |

## Canonical Test Vectors

| `h.command` (hex) | Name | `is_valid_enip_frame` return | Category |
|-------------------|------|------------------------------|---------|
| `0x0004` | ListServices | `true` | known — boundary low |
| `0x0063` | ListIdentity | `true` | known — recon |
| `0x0064` | ListInterfaces | `true` | known |
| `0x0065` | RegisterSession | `true` | known |
| `0x0066` | UnRegisterSession | `true` | known |
| `0x006F` | SendRRData | `true` | known — primary explicit-messaging |
| `0x0070` | SendUnitData | `true` | known |
| `0x0072` | IndicateStatus | `true` | known |
| `0x0075` | Cancel | `true` | known — boundary high |
| `0x0000` | (none) | `false` | unknown |
| `0x0062` | (none) | `false` | unknown — gap below ListIdentity |
| `0x0067` | (none) | `false` | unknown — gap between UnRegister/SendRRData |
| `0xFFFF` | (none) | `false` | unknown — max u16 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-C: `is_valid_enip_frame(&h)` returns `true` iff `h.command ∈ known_set`; biconditional holds for all 65,536 `u16` command values | Kani: symbolic `u16` command → fabricated `EnipHeader`; asserts `gate_result == is_known` for all inputs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the validity gate is the compensating control for port-44818 port-only classification (ADR-010 Decision 1); it prevents false-positive ENIP findings on mis-routed non-ENIP binary traffic and is a required safety property of the EtherNet/IP analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — ENIP flows are only routed after TLS/HTTP content rules fail; this gate is the second-line check) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 1 (port-fallback + validity gate), Decision 2 (parse path) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — validity gate; no finding emission directly) |

## Related BCs

- BC-2.17.001 — depends on (parse returns None for short input; gate is never called on None)
- BC-2.17.002 — depends on (gate operates on EnipHeader produced by parse_enip_header)
- BC-2.17.004 — composes with (classify_enip_command produces Unknown for same invalid commands)
- BC-2.17.016 — composes with (gate failure triggers `parse_errors++` and desync handling)
- BC-2.17.018 — depends on (gate failure feeds `malformed_in_window` counter)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn is_valid_enip_frame(h: &EnipHeader) -> bool` — pure-core free function
- `src/analyzer/enip.rs` — frame-walk loop: `if !is_valid_enip_frame(&header) { flow.parse_errors++; flow.malformed_in_window++; cursor += 1; continue; }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 1` — port-fallback + validity gate compensating control
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md §Sub-C` — Kani proof skeleton (biconditional assertion)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-C — validity gate biconditional (all 65,536 u16 command values)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 1 (port-only classification + validity gate); ADR-010 Decision 2 (is_valid_enip_frame pseudocode and known-command set); VP-032 Sub-C skeleton; ODVA EtherNet/IP Specification Table 2-3 (command codes) |
| **Confidence** | high — known-command set is normative ODVA; VP-032 Sub-C Kani proof verifies biconditional for all 65,536 inputs |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same command always produces same bool |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-032 Sub-C Kani target |
