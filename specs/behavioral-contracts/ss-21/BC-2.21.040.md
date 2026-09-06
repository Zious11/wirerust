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

# BC-2.21.040: Command-Class Frame From an Unexpected Source Co-Tags T1692.001 Unauthorized Message

## Description

When a command-class frame (`WriteVar`, the download triad, `PlcControl` with a recognized
service, or `PlcStop` — the same set BC-2.21.033 Postcondition 2 defines) arrives at a
destination PLC from a source that DIFFERS from `expected_source_by_destination[dst_ip]`
(BC-2.21.033), `"T1692.001"` ("Unauthorized Message: Command Message," successor to revoked
T0855) is co-tagged onto whichever finding that command-class frame already produces (T0835/
T0836's Write Var finding, T0843/T0889/T0821's download-session finding, or T0858's PlcStop/
PlcControl finding). This BC adapts DNP3's established "unexpected-source" baseline pattern
(BC-2.15.010) to S7comm's per-TCP-flow model by keying the baseline on the DESTINATION (PLC)
rather than per-flow, per BC-2.21.033's design. T1692.001 is **already seeded and already
emitted** — this BC adds ONLY the S7comm emission condition.

**Design note — divergence from Modbus's convention, RESOLVED at F2 INTEGRATE
(2026-09-06).** Modbus (BC-2.14.013) co-tags T1692.001 on EVERY write-class command
unconditionally (no baseline check) — "any write is worth flagging as a potential
unauthorized command since passive analysis cannot confirm authorization." This BC instead
follows DNP3's gated, baseline-relative model (BC-2.15.010), because the source research
explicitly qualifies S7comm's T1692.001 evidence as "Bytes alone don't prove 'unauthorized'"
(a caveat the Modbus research did not carry with the same explicitness).

**INTEGRATE confirms this divergence is intentional and final**, not an inconsistency to
reconcile toward one uniform cross-protocol policy: T1692.001's emission policy is a
**per-protocol, evidence-strength-driven choice**, not a project-wide convention that all
analyzers must share identically. Modbus's blanket co-tag and DNP3/S7comm's gated,
baseline-relative co-tag are two independently-justified points on the same
evidence-strength spectrum — each protocol's research established a different confidence
posture for what "any write-class command" alone can support as unauthorized-message
evidence, and this BC's emission condition correctly reflects S7comm's (DNP3-aligned)
posture rather than being forced to match Modbus's. No further reconciliation action is
required; a future maintenance sweep MUST NOT "fix" this divergence into one uniform
policy without first re-deriving each protocol's evidence-strength justification from its
own source research.

## Preconditions

1. A command-class frame (per BC-2.21.033 Postcondition 2's definition) targets `dst_ip` from
   `src_ip`.
2. `expected_source_by_destination[dst_ip]` is `Some(baseline_src)` (a baseline has already
   been established for this destination — per BC-2.21.033 Postcondition 4, the very first
   command-class frame to a destination establishes the baseline and never itself triggers
   this BC).
3. `src_ip != baseline_src`.
4. This frame already produces (or is about to produce, in the same `on_data` call) a
   `Finding` via one of BC-2.21.030/031/032/034/035/036 (i.e., T1692.001 has a host finding to
   attach to — it is never emitted as a standalone `Finding` with no other technique tag).

## Postconditions

1. `"T1692.001"` is appended to the `mitre_techniques` vec of the host finding identified in
   Precondition 4. The finding's `category`/`verdict`/`confidence`/`summary`/`evidence` are NOT
   otherwise modified by this BC, except the evidence field additionally notes
   `"; source {src_ip} differs from established baseline {baseline_src} for destination
   {dst_ip} (T1692.001)"`.
2. `expected_source_by_destination[dst_ip]` is left UNCHANGED (per BC-2.21.033 Postcondition 4
   — the baseline is never overwritten by an unexpected source).
3. No one-shot guard: every subsequent command-class frame from a DIFFERENT-from-baseline
   source to the same destination re-triggers this co-tag (mirrors DNP3's "fires at count=1,
   independent of a rate threshold" per-occurrence philosophy, BC-2.15.010) — an ongoing
   campaign from the unexpected source generates repeated, forensically valuable T1692.001
   co-tags, not a single suppressed one.

## Invariants

1. **T1692.001 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `MitreTactic::IcsImpairProcessControl` (remapped from revoked T0855 per v19
   catalog remap, issue #222) — no `src/mitre.rs` catalog or enum change required.
2. **Never a standalone finding**: unlike T0843/T0889 (BC-2.21.030/031, which have their own
   dedicated `Finding` objects), T1692.001 in this BC is EXCLUSIVELY a co-tag on an existing
   command-class finding — there is no wire evidence for "unauthorized" independent of the
   command itself being observed.
3. **Baseline-relative, not allowlist-based**: per BC-2.21.033 Invariant 4, this is the
   wire-observable proxy for the ideal "allowlist / maintenance window" mechanism the source
   research names; a legitimate SECOND engineering station (redundant HMI, backup engineering
   workstation) would trigger this co-tag on every command it issues, mirroring DNP3's own
   documented accepted-limitation edge case (BC-2.15.010 EC-011, redundant-master topology) —
   this is a conscious, disclosed false-positive class, not a defect.
4. **Applies across ALL command-class finding types**: this is the one B2 BC in this feature
   whose emission surface spans every OTHER emission BC in this burst (BC-2.21.030 through
   036) rather than owning a single classification arm — its precondition is entirely about
   the CROSS-FLOW baseline state (BC-2.21.033), not about any one `S7ClassicFunction` variant.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First command-class frame to a new destination (baseline-establishing) | NO T1692.001 co-tag (Precondition 2 fails — no baseline yet to differ from); the underlying finding (e.g. T0835) still fires normally |
| EC-002 | Second command-class frame to the same destination, SAME source as baseline | NO T1692.001 co-tag (Precondition 3 fails — source matches baseline) |
| EC-003 | Command-class frame to the same destination, DIFFERENT source than baseline | T1692.001 co-tagged onto the underlying finding |
| EC-004 | A non-command-class frame (e.g. `ReadVar`) from an unexpected source | NO T1692.001 — Precondition 1 fails; `ReadVar` is not in the command-class set and produces no host finding to co-tag |
| EC-005 | Redundant-engineering-station topology: two legitimate sources both issue commands to the same PLC | The second (non-baseline) source's commands are co-tagged T1692.001 on every occurrence — accepted limitation (Invariant 3), mirroring DNP3's EC-011 |
| EC-006 | Command-class frame to a destination with NO established baseline AND this frame itself is the download-session-completion frame | The download-session finding (T0843/T0889/etc.) fires; per Precondition 2, this frame establishes the baseline (BC-2.21.033 Postcondition 4) rather than triggering T1692.001 against itself |

## Canonical Test Vectors

| Scenario | Expected `mitre_techniques` amendment | Category |
|---|---|---|
| First WriteVar to PLC X from A | No T1692.001 (baseline established) | happy-path: baseline |
| Second WriteVar to PLC X from A | No T1692.001 (matches baseline) | happy-path: expected source |
| WriteVar to PLC X from B (baseline is A) | `"T1692.001"` appended to the WriteVar finding | happy-path: unexpected source |
| Download session to PLC X completes, from B (baseline is A) | `"T1692.001"` appended to the `["T0843","T0889",...]` finding | happy-path: unexpected-source download |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Baseline-mismatch → T1692.001 co-tag on the correct host finding, baseline never overwritten: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 names T1692.001 among the 8 reused technique IDs (ADR-014 Decision 5), built on BC-2.21.033's cross-flow baseline substrate |
| L2 Domain Invariants | INV-9 (MITRE technique ID format — this BC's T1692.001 attribution follows the same convention Modbus's BC-2.14.006 already cites for INV-9) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T1692.001 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T1692.001 — Unauthorized Message: Command Message (successor to revoked T0855; `MitreTactic::IcsImpairProcessControl`; already seeded + emitted; S7comm adds an emission condition — baseline-relative, per BC-2.21.033 — rather than a new call-site tied to one classification arm) |

## Related BCs

- BC-2.21.033 — depends on (`expected_source_by_destination` baseline this BC's core condition checks)
- BC-2.21.030, BC-2.21.031, BC-2.21.032, BC-2.21.034, BC-2.21.035, BC-2.21.036 — composes with (every one of these BCs' findings is a potential T1692.001 co-tag host)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — a shared post-emission hook invoked after any command-class finding is pushed: `if src_ip != expected_source_by_destination[dst_ip] { finding.mitre_techniques.push("T1692.001") }`
- `src/mitre.rs` — `technique_info("T1692.001")` arm (existing; shared across protocols)
- `.factory/specs/behavioral-contracts/ss-15/BC-2.15.010.md` — the DNP3 "unexpected-source" baseline precedent this BC adapts
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T1692.001 row)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads cross-flow `S7commAnalyzer` state (BC-2.21.033); mutates the host finding's `mitre_techniques` vec |
| **Deterministic** | yes |
| **Thread safety** | `S7commAnalyzer` is single-threaded |
| **Overall classification** | effectful shell |
