---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-21
capability: CAP-21
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "8f268fc"
---

# BC-2.21.026: TLS-Wrapped S7comm-plus Defers Entirely to SS-07 — No Decryption, No Interpretation Attempt

## Description

Once a flow classified `S7Protocol::Plus` (BC-2.21.024) is observed to negotiate TLS
(secure PG/HMI: S7-1500 firmware ≥ 2.9, S7-1200 firmware ≥ 4.5), `S7commAnalyzer` makes
**zero** attempt to decrypt, fingerprint-beyond-existing-capability, or semantically
interpret the encrypted traffic. Per ADR-014 Decision 6, this is explicitly out of
scope: "an offline analyzer without keys sees only endpoints, sizes, and timing for
TLS-protected flows, which the existing TLS analyzer (SS-07) already covers
generically." This BC formalizes the handoff boundary: SS-21 stops observing the flow
functionally once TLS is detected (BC-2.21.025 Postcondition 3), and SS-07's existing,
protocol-agnostic TLS analysis is the sole source of any further finding for that
traffic.

## Preconditions

1. A flow classified `S7Protocol::Plus` is observed to negotiate TLS (a TLS
   ClientHello or other TLS-handshake signature is detected per SS-07's existing
   logic).

## Postconditions

1. `S7commAnalyzer` performs no further byte-level reads, metadata extraction, or
   classification attempts on this flow's payload once TLS is detected — the
   `classified_protocol: Some(Plus)` tag and any pre-TLS session-setup metadata
   (BC-2.21.025) already recorded remain, but no new S7comm-plus-specific observation
   is added.
2. No integrity or anti-replay material (present in S7comm-plus's proprietary
   per-packet fields) is interpreted at any point, TLS-wrapped or not — this is an
   explicit non-goal independent of the TLS question (ADR-014 Decision 6 "OUT of
   scope" list, item 3).
3. SS-07's TLS analyzer continues to observe and report on the flow using its
   existing, protocol-agnostic capability (endpoints, sizes, timing, certificate
   metadata if applicable) — this BC does not modify or extend SS-07's behavior in any
   way; it only confirms SS-21 does not duplicate or interfere with it.

## Invariants

1. **No decryption capability, ever, this cycle**: stated as a hard non-goal, not a
   deferred nice-to-have — wirerust is an offline analyzer with no key-material
   access.
2. **Single source of truth for TLS-wrapped S7comm-plus findings**: SS-07 is the only
   analyzer producing findings for this traffic once TLS is active; SS-21 never
   produces a competing or duplicate finding for the same encrypted bytes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A TLS-wrapped `0x72` flow's certificate metadata (via SS-07) reveals a Siemens-specific certificate subject/OID | This is SS-07's existing capability surfacing incidentally useful context; SS-21 takes no action on it and claims no S7comm-plus-specific interpretation credit |
| EC-002 | A flow's TLS handshake fails or is reset mid-negotiation | SS-07's existing TLS-analysis error/anomaly handling applies unchanged; SS-21 does not add S7comm-plus-specific handling for this case |

## Canonical Test Vectors

| Scenario | Expected outcome | Category |
|----------|-------------------|---------|
| `0x72` flow negotiates TLS successfully | SS-21 stops per-flow S7comm-plus-specific observation; SS-07 continues its generic TLS analysis independently | happy-path: clean handoff |
| `0x72` flow never negotiates TLS (legacy or fully plaintext deployment) | BC-2.21.024/025 continue to apply for the flow's entire lifetime; this BC's boundary is never invoked | edge-case: no handoff needed |

## Verification Properties

(No independent VP-NNN — boundary contract verified by an integration-style regression
guard asserting no `S7commFlowState` mutation occurs for a flow after its TLS-detected
timestamp, mirroring BC-2.21.024's regression-guard style.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the SS-07 handoff boundary is the explicit "OUT of scope" complement to CAP-21's S7comm-plus IN-scope surface (BC-2.21.024/025) |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); SS-07 (existing TLS analyzer, unmodified) |
| ADR | ADR-014 Decision 6 ("No attempt to decrypt or interpret TLS-wrapped S7comm-plus... the existing TLS analyzer (SS-07) already covers generically") |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — explicit non-goal boundary contract) |

## Related BCs

- BC-2.21.024 — composes with (the framing-only classification this boundary bounds)
- BC-2.21.025 — depends on (the pre-TLS observation window this BC's Postcondition 1 terminates)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — TLS-detected early-return/no-op branch for `S7Protocol::Plus` flows
- `src/analyzer/tls.rs` (SS-07, existing, unmodified) — the sole finding source for TLS-wrapped traffic
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 6` — "No attempt to decrypt or interpret TLS-wrapped S7comm-plus"

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads TLS-detection state from SS-07 (cross-analyzer, per-flow) |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (no-op boundary enforcement) |
