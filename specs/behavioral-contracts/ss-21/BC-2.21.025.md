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
input-hash: "cf116b5"
---

# BC-2.21.025: S7comm-plus Unencrypted Session-Setup Handshake Metadata Observation — Bounded to Message-Type/Opcode Byte and Sequence Marker, Pre-TLS Only

## Description

Per ADR-014 Decision 6, S7comm-plus's IN-scope surface extends beyond bare framing
classification (BC-2.21.024) to include limited, unencrypted session-setup handshake
metadata: prior to any TLS upgrade, the S7comm-plus session-setup exchange may expose a
message-type/opcode byte and sequence markers at the object/service-protocol envelope
level. This BC bounds exactly what is read: **only** the first byte of the object/
service envelope immediately following `protocol_id` (a message-type/opcode-shaped
byte) and any directly adjacent sequence-marker byte(s) the envelope's fixed prefix
exposes — never a parsed object ID, service ID, or payload field. This is a forensic
observation, not a dissector: the extracted bytes are surfaced as raw metadata in a
finding-adjacent observation record, not mapped to a named `S7ClassicFunction`-style
enum, since the S7comm-plus object/service protocol's semantics are explicitly out of
this feature's research scope (ADR-014 Decision 4 — no official spec, no dissection
budget allocated).

## Preconditions

1. `S7commAnalyzer` has classified the flow's `classified_protocol` as
   `S7Protocol::Plus` (BC-2.21.024).
2. No TLS ClientHello or other TLS-handshake signature (per SS-07's existing TLS
   detection) has been observed on this flow — i.e. the exchange is still in its
   unencrypted, pre-upgrade phase.
3. The DT frame's payload (immediately following `protocol_id`) contains at least 1
   byte.

## Postconditions

1. The message-type/opcode byte (the byte immediately following `protocol_id`) is
   extracted and recorded as raw metadata — not matched against any semantic table,
   not classified into an enum variant beyond "observed opcode byte: `<value>`."
2. If the fixed session-setup envelope prefix known from prose sources exposes an
   adjacent sequence-marker byte within the same bounded read window, it is extracted
   alongside the opcode byte; no additional bytes beyond this fixed, small window are
   read.
3. Once SS-07's existing TLS-handshake detection observes a TLS signature on the same
   flow (post-upgrade), this metadata observation ceases for that flow — no attempt is
   made to read further bytes from a TLS-protected S7comm-plus session (the existing
   TLS analyzer, SS-07, already covers generically what an offline analyzer without
   keys can see: endpoints, sizes, timing).
4. This observation never produces a `S7ClassicFunction`-shaped classification, never
   triggers a Group-3/Group-4-style function-code match, and never claims to identify
   a specific S7comm-plus operation (start-session, read-tag, write-tag, etc.) — the
   contract is limited to "these raw bytes were observed in the clear during
   session-setup," full stop.

## Invariants

1. **Bounded window, no growth**: this BC's read window is a small, fixed set of bytes
   (opcode + adjacent sequence marker) — it never grows into progressively deeper
   payload interpretation as a way to informally add dissection capability without
   officially calling it one.
2. **Pre-TLS only**: the observation is explicitly time-bounded to before any TLS
   upgrade is detected; this is a hard boundary, not a best-effort one — Postcondition
   3 requires the observation to *cease*, not merely to "not add new value," once TLS
   is detected.
3. **No semantic claim**: this BC's forensic-observation framing is a deliberate
   epistemic choice — recording *that* bytes were seen, without claiming to know *what
   they mean*, given no official S7comm-plus specification exists (ADR-014 Decision
   4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A `0x72` flow's very first frames are already TLS-wrapped (modern S7-1500 firmware negotiating secure PG/HMI from the start, no unencrypted setup phase at all) | No metadata observation occurs — Precondition 2 is never satisfied; the flow is classified `Plus` (BC-2.21.024) with zero session-setup metadata records |
| EC-002 | A flow exposes 3 unencrypted setup frames, then upgrades to TLS mid-session | Metadata observed for the first 3 frames only; observation ceases from the 4th frame onward per Postcondition 3 |
| EC-003 | The opcode byte happens to numerically match a classic S7comm ROSCTR or FC value (e.g. `0x01`) | No cross-interpretation occurs — the byte is recorded as an S7comm-plus opcode observation, never fed into the classic `S7ClassicFunction`/`Rosctr` classification surfaces |

## Canonical Test Vectors

| Scenario | Expected outcome | Category |
|----------|-------------------|---------|
| Unencrypted `0x72` session-setup frame, 1 byte of payload after `protocol_id` | Opcode byte recorded as raw metadata | happy-path: minimal observation |
| `0x72` flow with TLS detected on frame 2 | Frame 1 metadata recorded; frame 2 onward: no observation | happy-path: TLS-upgrade cutoff |
| `0x72` flow, TLS present from frame 1 | Zero metadata records for the entire flow | edge-case: no unencrypted phase at all |

## Verification Properties

(No independent VP-NNN — bounded-window read verified by unit test asserting exactly
the fixed window size is read and no more; TLS-cutoff behavior verified by an
integration-style test combining SS-07's TLS detection signal with this BC's
observation gate.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the "unencrypted session-setup metadata observation" surface CAP-21's description and ADR-014 Decision 6 explicitly enumerate as in-scope |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); SS-07 (TLS analyzer, upgrade-detection dependency) |
| ADR | ADR-014 Decision 6 ("Unencrypted session-setup handshake metadata observation... MAY be surfaced as a forensic observation finding") |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — a raw-byte forensic observation with no established semantic-to-technique mapping; if a future research pass establishes one, it would be a B2-scope addition, not retrofitted here) |

## Related BCs

- BC-2.21.024 — depends on (the framing-level classification this observation extends)
- BC-2.21.026 — composes with (the TLS-upgrade deferral this BC's Postcondition 3 implements)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — bounded session-setup metadata extraction, gated on `S7Protocol::Plus` and absence of a prior TLS signature
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 6` — "Unencrypted session-setup handshake metadata observation"
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — "S7comm-plus / TLS caveat"

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads TLS-detection state from SS-07 (cross-analyzer, per-flow); writes an observation record to the flow's finding-adjacent metadata store |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (bounded metadata extraction, cross-analyzer read) |
