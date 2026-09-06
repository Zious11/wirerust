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

# BC-2.21.037: Decoded Restart Sub-Operation Within PLC Control Program-Start Co-Tags T0816 Device Restart/Shutdown

## Description

**This BC discloses a gap between the source research and part B1's classification surface,
and contracts the narrowest defensible resolution.** The source research's T0816 detection
pattern is "`0x28` restart PI-service string" — phrased as if a distinct, dedicated service
string exists. Part B1's `PlcControlService` enum (BC-2.21.015), however, recognizes exactly
five service strings (`P_PROGRAM`, `_INSE`, `_DELE`, `_GARB`, `_MODU`); there is no sixth
"restart" string. Cold/warm restart in classic S7comm is understood (per general S7 protocol
structure, not independently re-verified via a fresh live-source pass in this research burst)
to be encoded as a **sub-operation parameter following the `P_PROGRAM` service-name string**,
not as a separate service name — exactly the "additional PI-service parameter bytes beyond the
service-name string" BC-2.21.015 Invariant 3 already flags as undecoded and explicitly assigns
to "B2's T0858 emission call-site... any further sub-operation disambiguation it needs." This BC
is that further disambiguation, scoped conservatively: T0816 is co-tagged onto BC-2.21.036 path
(b)'s finding ONLY when a NEW, B2-authored decode of the `P_PROGRAM` parameter's trailing bytes
positively identifies a restart operation; when that decode is not attempted, not possible, or
inconclusive, NO T0816 is emitted (unlike T0821's low-confidence fallback, BC-2.21.032 — a
speculative restart tag on every `P_PROGRAM` occurrence would create ambiguous double-tagging
with T0858 for the common start/run case, which this BC deliberately avoids).

## Preconditions

1. BC-2.21.036 path (b)'s Precondition 2 holds
   (`S7ClassicFunction::PlcControl(PlcControlService::ProgramStart)` classified).
2. The PI-service parameter block contains bytes beyond the `"P_PROGRAM"` service-name string
   (i.e. the parameter block is long enough to carry a sub-operation indicator).
3. Those trailing bytes are decodable (per an implementer-assigned, byte-level convention this
   BC does not pin — see Invariant 2) as a cold-restart or warm-restart indicator, distinct
   from a plain start/run indicator.

## Postconditions

1. If Preconditions 1-3 all hold: `"T0816"` is appended to the SAME `Finding` BC-2.21.036 path
   (b) pushes for this `P_PROGRAM` frame — no separate `Finding` object. The evidence field
   additionally notes `"; sub-operation decoded as restart (cold/warm)"`.
2. If Precondition 2 or 3 fails (no trailing bytes, or trailing bytes present but not decodable
   as restart-specific): NO `"T0816"` is appended — BC-2.21.036's `["T0858"]`-only finding
   stands unmodified. This is a strict gate, not a confidence downgrade (contrast BC-2.21.032's
   T0821 low-confidence fallback).
3. `PlcStop` (BC-2.21.036 path a) NEVER co-tags T0816 — `PlcStop` and a restart operation are
   distinct S7 operations (stop halts execution; restart is a specific `P_PROGRAM` sub-mode);
   this BC's scope is exclusively the `P_PROGRAM` path.

## Invariants

1. **T0816 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `MitreTactic::IcsInhibitResponseFunction` (`TA0107`) — no `src/mitre.rs`
   catalog or enum change required for the ID itself; this BC only adds the S7comm-specific,
   gated emission condition.
2. **Sub-operation decode mechanics are unspecified by design, not deferred-implementation
   detail**: unlike BC-2.21.012/015/029's "values pinned, byte offset deferred" pattern, this
   BC does NOT pin even the semantic decode values for the restart sub-operation, because no
   research pass in this feature independently verified the byte-level convention for
   `P_PROGRAM`'s trailing parameter bytes.
   **RESOLVED at F2 INTEGRATE (2026-09-06).** No follow-up research pass verifying the
   sub-operation byte convention was commissioned in the INTEGRATE sub-burst (option (a)
   below remains open for a future cycle, but is not exercised now). This BC is therefore
   finalized, for this feature cycle, as **outcome (b): a gated contract that yields ZERO
   S7comm emission call-sites this cycle.** This is an explicit, deliberate, non-blocking
   resolution, not an unresolved gap: BC-2.21.036's T0858-only treatment of `P_PROGRAM`
   stands as the COMPLETE and CORRECT S7comm behavior for this feature; T0816 remains
   seeded-and-emitted via ENIP (`EMITTED_IDS` already contains it) with no S7comm call-site
   added. The two options this BC originally posed for INTEGRATE to choose between were:
   - (a) a follow-up research pass verifies the exact sub-operation byte convention
     (cold-restart vs. warm-restart vs. plain-start indicator values) — NOT exercised this
     cycle;
   - (b) T0816 has NO S7comm emission call-site this cycle, and BC-2.21.036's T0858-only
     treatment stands as complete — **this is the finalized outcome.**
   Adding zero call-sites is a valid, non-blocking degenerate case of ADR-014 Decision 5's
   "add S7comm emission call-sites only — no catalog change" framing, not a violation of it.
   A future feature cycle MAY reopen this BC via option (a) if the byte-level restart
   convention is independently verified; until then, this BC's Preconditions/Postconditions
   remain specified (so the gate is ready to implement the moment the convention is
   verified) but are expected to never fire in this feature's delivered scope.
3. **No double-tagging ambiguity**: because Postcondition 2 is a strict gate, a `P_PROGRAM`
   frame never carries BOTH an unqualified T0858-start meaning and a T0816-restart meaning
   without positive restart evidence — avoiding the "is this a start or a restart" ambiguity
   the source research's "flagged/unverifiable" section warns about generally for `0x28`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `P_PROGRAM` with no trailing parameter bytes | NO T0816 (Precondition 2 fails); T0858 finding stands alone |
| EC-002 | `P_PROGRAM` with trailing bytes that decode to a plain start/run indicator | NO T0816 (Precondition 3 fails — decoded, but not restart-specific) |
| EC-003 | `P_PROGRAM` with trailing bytes that decode to a cold-restart indicator (per an implementer-verified convention) | T0816 co-tagged onto the T0858 finding |
| EC-004 | `PlcStop` (`0x29`) | NO T0816 regardless of any other state (Postcondition 3) |
| EC-005 | Sub-operation decode is not implemented at all in a given release (per Invariant 2 outcome (b)) | NO T0816 ever emitted for S7comm; T0858-only treatment is the complete, valid behavior — not a defect |

## Canonical Test Vectors

| Trigger | Expected `mitre_techniques` | Category |
|---|---|---|
| `P_PROGRAM`, no sub-op decode attempted | `["T0858"]` only | happy-path: conservative default |
| `P_PROGRAM`, any trailing sub-operation bytes (including a restart-shaped indicator) | `["T0858"]` only, never `T0816` | happy-path: RESOLVED zero-call-site outcome — no decode helper exists this cycle (Invariant 2 outcome (b)) |
| `PlcStop` | `["T0858"]` only, never `T0816` | regression-guard |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | No-emission negative assertion: the `P_PROGRAM` / `PlcControlService::ProgramStart` path never appends `"T0816"` this cycle under any input, because no S7comm decode call-site exists (Invariant 2 outcome (b), RESOLVED — zero call-sites); effectful shell | unit test (regression guard) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 names T0816 among the 8 reused technique IDs (ADR-014 Decision 5); this BC discharges that obligation conservatively given the disclosed B1/research gap |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0816 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0816 — Device Restart/Shutdown (ICS Inhibit Response Function, TA0107; already seeded + emitted via ENIP; S7comm emission call-site is GATED and RESOLVED at F2 INTEGRATE to ZERO call-sites this cycle — see Invariant 2's finalized resolution — pending a future cycle's independent verification of the restart-byte convention) |

## Related BCs

- BC-2.21.015 — depends on (`PlcControlService::ProgramStart` classification; Invariant 3's explicit deferral of sub-operation decode to B2)
- BC-2.21.036 — composes with (T0816 conditionally appends to BC-2.21.036 path (b)'s finding)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `PlcControl(ProgramStart)` handler: NO sub-operation decode helper is added this cycle (Invariant 2 outcome (b), RESOLVED — zero S7comm emission call-sites); a future cycle MAY introduce one, contingent on independently verifying the `P_PROGRAM` trailing-byte convention (Invariant 2 option (a), not exercised now)
- `src/mitre.rs` — `technique_info("T0816")` arm (existing; shared with ENIP)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0816 row: "decoded `0x28` restart PI-service string" — the gap this BC discloses)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5` ("`0x28` PI-Service ambiguity... MUST decode the service name before mapping")

## Story Anchor

(TBD — assigned during F3 story decomposition; blocked on INTEGRATE's Invariant 2 resolution)

## VP Anchors

None. Invariant 2 is RESOLVED (F2 INTEGRATE, 2026-09-06) as outcome (b): zero S7comm emission call-sites this cycle, so no VP is anchored to this BC now. A future cycle that reopens Invariant 2 via option (a) (independent byte-convention research) would introduce a VP at that time.

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings` (when implemented) |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (conditional on INTEGRATE resolving Invariant 2) |
