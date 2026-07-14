---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-10
capability: CAP-10
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "8b69772"
---

# BC-2.10.010: T0881 "Service Stop" Registered in SEEDED_TECHNIQUE_IDS, technique_info(), and EMITTED_IDS

## Description

The MITRE ATT&CK for ICS technique T0881 "Service Stop" must be registered
in all three cataloging structures in `mitre.rs` atomically (ADR-013 Decision 10, which
extends the VP-007 six-part atomic obligation from prior protocols). This BC defines the
three-part registration: (1) `SEEDED_TECHNIQUE_IDS` array must include `"T0881"` (bumping
count from 28 to 29), (2) `technique_info("T0881")` must return a non-None entry with
correct name/tactic, and (3) `EMITTED_IDS` must include `"T0881"`. All three
registrations must occur in the same commit to avoid partial registration gaps.

## Preconditions

1. The IEC-104 passive analyzer feature is being integrated (feature-iec104).
2. `mitre.rs` contains `SEEDED_TECHNIQUE_IDS`, `technique_info()`, and `EMITTED_IDS` structures.
3. T0881 is NOT yet registered in any of the three structures prior to this feature.

## Postconditions

1. `SEEDED_TECHNIQUE_IDS.contains("T0881")` == true (count: 28 → 29).
2. `technique_info("T0881")` returns `Some(("Service Stop", MitreTactic::IcsInhibitResponseFunction))`.
3. `EMITTED_IDS.contains("T0881")` == true.
4. VP-007 Kani harness `verify_all_seeded_ids_resolve` passes with SEEDED count=29.

## Invariants

1. **Atomic six-part registration**: all six parts of the VP-007 obligation (SEEDED array, COUNT, technique_info arm, EMITTED array, Kani count constant, VP file) must be updated in one commit. ADR-013 Decision 10 extends ADR-007 Decision 4.
2. **T0881 tactic**: `MitreTactic::IcsInhibitResponseFunction` (tactic TA0107 — T0881 "Service Stop" is classified under Inhibit Response Function in ATT&CK for ICS).
3. **SEEDED count bump**: `SEEDED_TECHNIQUE_ID_COUNT` constant must be bumped from 28 to 29 atomically with the array entry.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `technique_info("T0881")` called | Returns Some(("Service Stop", IcsInhibitResponseFunction)) |
| EC-002 | `technique_info("T0881 ")` (trailing space) | Returns None (exact match) |
| EC-003 | SEEDED_TECHNIQUE_IDS count check | len == 29 |

## Canonical Test Vectors

| Query | Expected |
|-------|----------|
| `technique_info("T0881")` | `Some(("Service Stop", MitreTactic::IcsInhibitResponseFunction))` |
| `SEEDED_TECHNIQUE_ID_COUNT` | 29 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All SEEDED_TECHNIQUE_IDS have corresponding technique_info entries; SEEDED count == SEEDED_TECHNIQUE_ID_COUNT == 29; T0881 resolves | Kani: `verify_all_seeded_ids_match_format` + `verify_all_seeded_ids_resolve` (src/mitre.rs) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK ICS Technique Mapping") per ARCH-INDEX.md §SS-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK ICS Technique Mapping") per ARCH-INDEX.md §SS-10 — T0881 registration extends the MITRE technique catalog that underpins all ICS finding attribution in the analyzer |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-10 (src/mitre.rs); ADR-013 Decision 10 |
| Feature | feature-iec104 |
| MITRE Techniques | T0881 "Service Stop" (IcsInhibitResponseFunction / TA0107 — the technique being registered) |

## Related BCs

- BC-2.19.011 — depends on (T0881 Possible finding requires this registration)
- BC-2.19.012 — depends on (T0881 Likely finding requires this registration)

## Architecture Anchors

- `src/mitre.rs` — `SEEDED_TECHNIQUE_IDS`: add `"T0881"`; `SEEDED_TECHNIQUE_ID_COUNT`: 28 → 29
- `src/mitre.rs` — `technique_info("T0881")` arm: `"T0881" => ("Service Stop", MitreTactic::IcsInhibitResponseFunction)`
- `src/mitre.rs` — `EMITTED_IDS`: add `"T0881"`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 10` — VP-007 T0881 six-part atomic obligation

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-007 — `verify_all_seeded_ids_match_format` + `verify_all_seeded_ids_resolve` (SEEDED count=29, T0881 in catalog)
