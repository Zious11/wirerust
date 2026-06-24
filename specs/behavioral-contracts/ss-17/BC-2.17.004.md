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

# BC-2.17.004: classify_enip_command Total Classification with Unknown Arm Over All u16 Values

## Description

`classify_enip_command(cmd: u16) -> EnipCommandClass` is a pure-core classification function
that maps all 65,536 possible `u16` EtherNet/IP command values to a named `EnipCommandClass`
variant. The function is total — it never panics and always returns a variant. The 9 known
ODVA command codes each map to a named variant; all other values (65,527 of them) map to
`EnipCommandClass::Unknown`. The `Unknown` arm is reachable and proven non-vacuous. The
function is formally verified for totality by VP-032 Sub-B. Command classification results
are stored in `EnipFlowState.command_counts` and used by the `summarize()` output.

## Preconditions

1. `cmd` is a `u16` — all 65,536 values are valid inputs with defined behavior.
2. No preconditions on calling context: this function is called for every successfully parsed
   ENIP header, including those that fail `is_valid_enip_frame`.

## Postconditions

1. `classify_enip_command(cmd)` returns a valid `EnipCommandClass` variant for every input.
2. The mapping is:
   - `0x0004` → `EnipCommandClass::ListServices`
   - `0x0063` → `EnipCommandClass::ListIdentity`
   - `0x0064` → `EnipCommandClass::ListInterfaces`
   - `0x0065` → `EnipCommandClass::RegisterSession`
   - `0x0066` → `EnipCommandClass::UnRegisterSession`
   - `0x006F` → `EnipCommandClass::SendRRData`
   - `0x0070` → `EnipCommandClass::SendUnitData`
   - `0x0072` → `EnipCommandClass::IndicateStatus`
   - `0x0075` → `EnipCommandClass::Cancel`
   - all other values → `EnipCommandClass::Unknown`
3. The function never panics for any `u16` input.
4. The `Unknown` variant is reachable (e.g., `cmd = 0x0000` → `Unknown`).
5. The function is pure: no I/O, no state mutation.

## Invariants

1. **Totality**: `EnipCommandClass` must have exactly 10 variants (9 named + `Unknown`) and
   the match expression must be exhaustive. The Rust compiler enforces exhaustiveness; Kani
   VP-032 Sub-B proves the non-vacuity of the `Unknown` arm.
2. **Correspondence with validity gate**: the set of commands that maps to non-Unknown variants
   is exactly the known-command set used by `is_valid_enip_frame` (BC-2.17.003). These two
   functions must remain in sync — adding a new command to one requires adding it to both.
3. **Counter accumulation**: the caller increments
   `flow.command_counts.entry(cmd).or_insert(0) += 1` after each classification. Both named
   and Unknown commands are counted.
4. **Purity**: `classify_enip_command` is a pure-core Kani target (VP-032 Sub-B). No heap
   allocation, no I/O, no global state access.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `cmd = 0x006F` (SendRRData) | Returns `EnipCommandClass::SendRRData` |
| EC-002 | `cmd = 0x0063` (ListIdentity — recon) | Returns `EnipCommandClass::ListIdentity` |
| EC-003 | `cmd = 0x0000` | Returns `EnipCommandClass::Unknown` — not in ODVA table |
| EC-004 | `cmd = 0xFFFF` | Returns `EnipCommandClass::Unknown` — max u16, not ODVA-assigned |
| EC-005 | `cmd = 0x0067` (gap: between UnRegisterSession and SendRRData) | Returns `EnipCommandClass::Unknown` |
| EC-006 | `cmd = 0x0004` (ListServices) | Returns `EnipCommandClass::ListServices` — lowest named value |
| EC-007 | `cmd = 0x0075` (Cancel) | Returns `EnipCommandClass::Cancel` — highest named value |
| EC-008 | `cmd = 0x0076` | Returns `EnipCommandClass::Unknown` — one above Cancel |

## Canonical Test Vectors

| `cmd` (hex) | ODVA Name | Expected `EnipCommandClass` variant |
|-------------|-----------|-------------------------------------|
| `0x0004` | ListServices | `ListServices` |
| `0x0063` | ListIdentity | `ListIdentity` |
| `0x0064` | ListInterfaces | `ListInterfaces` |
| `0x0065` | RegisterSession | `RegisterSession` |
| `0x0066` | UnRegisterSession | `UnRegisterSession` |
| `0x006F` | SendRRData | `SendRRData` |
| `0x0070` | SendUnitData | `SendUnitData` |
| `0x0072` | IndicateStatus | `IndicateStatus` |
| `0x0075` | Cancel | `Cancel` |
| `0x0000` | (unassigned) | `Unknown` |
| `0x0001` | (unassigned) | `Unknown` |
| `0x0067` | (gap) | `Unknown` |
| `0xFFFF` | (unassigned) | `Unknown` |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-B: `classify_enip_command` is total over all 65,536 `u16` values; `Unknown` arm reachable via `cmd=0x0000` | Kani: symbolic `u16`; asserts `let _ = class` (no-panic proof); companion proof asserts `matches!(classify_enip_command(0x0000), EnipCommandClass::Unknown)` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — total ENIP command classification is required for detecting recon (ListIdentity T0846), session management abuse, and building the command-distribution summary in `summarize()`; totality ensures no u16 command value causes a panic |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — classification function; no finding emission) |

## Related BCs

- BC-2.17.003 — composes with (known-command set is identical; both must stay in sync)
- BC-2.17.010 — depends on (ListIdentity classification triggers T0846 recon detection)
- BC-2.17.021 — composes with (command_distribution in summarize() uses classify result)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn classify_enip_command(cmd: u16) -> EnipCommandClass` — pure-core free function
- `src/analyzer/enip.rs` — `enum EnipCommandClass { ListServices, ListIdentity, ListInterfaces, RegisterSession, UnRegisterSession, SendRRData, SendUnitData, IndicateStatus, Cancel, Unknown }`
- `src/analyzer/enip.rs` — `EnipFlowState.command_counts: HashMap<u16, u64>` — caller accumulates after classification
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — command enumeration
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md §Sub-B` — Kani proof skeleton

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-B — command classification totality (all 65,536 u16 values; Unknown non-vacuity)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (command enumeration); VP-032 Sub-B skeleton; ODVA EtherNet/IP Specification Table 2-3 (encapsulation command codes) |
| **Confidence** | high — command codes are normative ODVA; totality proven by VP-032 Sub-B Kani |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same u16 always produces same EnipCommandClass |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-032 Sub-B Kani target |
