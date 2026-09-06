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

# BC-2.21.041: Excluded and Deferred MITRE Techniques Are Explicit Non-Goals; ics-attack-19.1 Version Pin Retained

## Description

This BC is a negative-space contract, mirroring BC-2.21.026's explicit non-goal treatment of
TLS-wrapped S7comm-plus. It documents, as an enforceable specification rather than an implicit
omission, which candidate MITRE ATT&CK for ICS techniques this feature's classification and
emission surface (BC-2.21.001 through BC-2.21.040) deliberately does **not** produce evidence
for, and confirms the `ics-attack` catalog version pin this feature operates under. Per this
project's evidence-grounding discipline (established throughout part B1's "no force-fit,"
"no invented group IDs" invariants), an excluded technique is a documented decision, not a
silent gap an adversarial reviewer should have to rediscover.

## Preconditions

(None — this BC constrains what the feature must NOT do, applicable at all times.)

## Postconditions

1. `S7commAnalyzer` never emits `mitre_techniques` containing `"T0851"` (Rootkit) under any
   classification or state this feature produces.
2. `S7commAnalyzer` never emits `mitre_techniques` containing `"T0873"` or `"T0873.001"`
   (Project File Infection / Siemens Project File Format) under any classification or state
   this feature produces.
3. `S7commAnalyzer` never emits `mitre_techniques` containing `"T0813"` (Denial of Control)
   under any classification or state this feature produces — DEFERRED, not excluded (see
   Invariant 3 for the distinction).
4. The `ics-attack` catalog version pin (`MITRE_ATTACK_VERSION` in `src/reporter/json.rs`)
   remains `"ics-attack-19.1"` — this feature does NOT bump the pin to `"ics-attack-19.2"`.

## Invariants

1. **T0851 (Rootkit) — EXCLUDE, not network-observable** [s7comm-mitre-ics-tagging.md
   §Exclusions]: a block/firmware transfer on the wire is at most a *carrier*; the ROSCTR/
   FC/subfunction fields this feature's classification surface (`S7ClassicFunction`,
   `S7UserdataFunction`) exposes never evidence concealment/hiding behavior. Only deep
   payload-signature matching — out of a metadata-forensics analyzer's scope — could touch
   this, and this feature does not perform payload-signature matching.
2. **T0873 / T0873.001 (Project File Infection) — EXCLUDE, not network-observable**
   [s7comm-mitre-ics-tagging.md §Exclusions]: infection occurs in a STEP 7/WinCC/TIA project
   file at rest on an engineering workstation — off the wire entirely. A download
   (BC-2.21.013/030) shows that project-derived code WAS deployed, never that the SOURCE
   project file was infected. **If a future capability adds payload-signature matching against
   a downloaded block's content and finds a known-malicious signature, the correct tag is
   T0843/T0889 on the transfer (BC-2.21.030/031) — never T0873 on the file** (this project's
   own documented correction, carried forward as a standing rule for any future extension).
3. **T0813 (Denial of Control) — DEFER, not EXCLUDE**: unlike T0851/T0873, T0813 is not
   ruled out as unobservable in principle — it is only *indirectly* inferable from passive
   S7comm (connection resets, repeated failed jobs, absence of expected control responses),
   and a `PlcStop` is better attributed to T0858 (BC-2.21.036), not automatically T0813.
   DEFERRED means: no emission call-site exists this cycle, and none is precluded from being
   added in a future feature cycle if a robust control-loss temporal heuristic is designed —
   distinct from T0851/T0873's permanent, in-principle exclusion.
4. **Version pin retained at `ics-attack-19.1`, not bumped**: the live ATT&CK release as of
   this feature's research (2026-09-06) is `ics-attack-v19.2` (released 2026-08-06), but v19.2
   is confirmed [s7comm-mitre-ics-tagging.md §Version pin] to be an Agile minor touching only
   Enterprise Groups/Software with **zero** ICS technique-catalog changes — every technique
   mapping in this feature (T0843, T0889, T0821, and the 8 reused IDs) is valid under both
   v19.1 and v19.2. Retaining `19.1` is the lowest-churn choice and is explicitly NOT required
   by any technique-correctness consideration; a future feature MAY bump the pin as a pure
   currency update, independent of this feature's scope.
5. **This BC is a constraint, not an emission**: no `Finding` is ever produced BY this BC —
   its postconditions are universally-quantified absence statements over all OTHER BCs'
   emission surfaces (BC-2.21.030 through 040).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A download session transfers a block whose content happens to match a known rootkit signature (hypothetically, if payload-signature matching existed) | Even hypothetically, the correct tag would be T0843/T0889 on the transfer, never T0851 (Invariant 1) |
| EC-002 | A downloaded block's content matches a known-malicious signature (hypothetically) | T0843/T0889 tag the transfer; T0873/T0873.001 is never emitted (Invariant 2's standing rule) |
| EC-003 | Repeated connection resets or failed jobs suggest a possible denial-of-control pattern | No T0813 finding in this feature's scope; such evidence, if acted on, routes to T0858 (PlcStop) or T0814 (malformed/flood, already covered by BC-2.21.004/007/008/009 and BC-2.20.014) instead |
| EC-004 | A future feature cycle proposes adding T0813 emission | Not blocked by this BC (deferred, not excluded) — requires a new research pass establishing a defensible emission predicate, per Invariant 3 |

## Canonical Test Vectors

(Negative-space contract — canonical test vectors are regression guards asserting absence.)

| Scenario | Expected `mitre_techniques` | Category |
|---|---|---|
| Any S7comm traffic this feature's test corpus exercises | Never contains `"T0851"`, `"T0873"`, `"T0873.001"`, or `"T0813"` | regression-guard: exclusion/deferral enforcement |
| Version pin check on any emitted `Finding`/report envelope | `ics_attack_version == "ics-attack-19.1"` | regression-guard: version pin retained |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| No emission path in `S7commAnalyzer` ever produces `"T0851"`, `"T0873"`, `"T0873.001"`, or `"T0813"` in `mitre_techniques` | Exhaustive code-review / grep-based static check at implementation time (not a Kani/proptest target — this is an absence-of-code-path property, not a data-flow property); the existing `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS` drift guard (VP-007) already structurally prevents emitting an ID that was never seeded, which covers T0851/T0873/T0873.001/T0813 by construction since none of the four are added to `SEEDED_TECHNIQUE_IDS` by this feature |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC documents the boundary of CAP-21's MITRE-emission scope, mirroring BC-2.21.026's S7comm-plus TLS non-goal treatment for the classification layer |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned — absence of code paths); `src/reporter/json.rs` (version pin, unchanged) |
| ADR | ADR-014 Decision 5 ("Excluded (not seeded): T0851 Rootkit, T0873/T0873.001... **Deferred:** T0813..."; "Version pin: retain `ics-attack-19.1`...") |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0851, T0873, T0873.001 (EXCLUDED — never seeded, never emitted); T0813 (DEFERRED — never seeded, never emitted this cycle, not precluded from a future cycle) |

## Related BCs

- BC-2.21.026 — composes with (the precedent explicit non-goal pattern this BC mirrors, applied to the MITRE-emission layer instead of the S7comm-plus classification layer)
- BC-2.21.030, BC-2.21.031 — composes with (Invariant 2's standing rule for how a hypothetical future payload-signature match must be tagged)
- BC-2.21.004, BC-2.21.007, BC-2.21.008, BC-2.21.009, BC-2.20.014 — composes with (the existing T0814 emission surface EC-003's routing alternative points to, in lieu of T0813)

## Architecture Anchors

- `src/mitre.rs` — `SEEDED_TECHNIQUE_IDS` (this feature's additions: `"T0843"`, `"T0889"`, `"T0821"` only — `"T0851"`, `"T0873"`, `"T0873.001"`, `"T0813"` are never added)
- `src/reporter/json.rs` — `MITRE_ATTACK_VERSION` constant, `ics-attack-19.1` pin, unchanged by this feature
- `.factory/research/s7comm-mitre-ics-tagging.md` §Exclusions, §Deferred, §Version pin

## Story Anchor

(TBD — assigned during F3 story decomposition; likely satisfied by absence rather than a dedicated implementation story)

## VP Anchors

(None dedicated — covered structurally by VP-007's `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS` drift-guard harness, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (constrains absence of state/emission, not a stateful behavior itself) |
| **Deterministic** | yes (vacuously — no code path to be non-deterministic) |
| **Thread safety** | n/a |
| **Overall classification** | negative-space specification contract |
