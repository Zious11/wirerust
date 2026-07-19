---
document_type: story
story_id: STORY-175
epic_id: E-11
version: "1.0"
status: superseded
producer: story-writer
timestamp: 2026-07-18T00:00:00Z
phase: f7
level: feature
cycle: feature-iec104
points: 2
priority: P3
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: .factory/maintenance/
subsystems: []
estimated_days: 1
wave: "TBD"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/demo-evidence-scrub-gate.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "62d13e0"
---

# STORY-175: Feature-IEC104 Cycle-Close: Demo Evidence JSON Accuracy Protocol

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** superseded
**Wave:** TBD
**Points:** 2
**Priority:** P3

## Narrative

- **As a** spec-steward, adversary reviewer, and future contributor on the wirerust project
- **I want** the demo-evidence production discipline tightened so that JSON evidence files
  contain only real enum variants and field values derived from actual `cargo run`/`cargo test`
  serialized output
- **So that** fabricated illustrative JSON in demo-evidence artifacts no longer passes the
  scrub gate silently, preventing a class of adversarial HIGH findings that slowed the
  feature-iec104 F5 convergence (3 confirmed occurrences: FIX-F5-001 report R2 F5R2-02,
  FIX-P4-001 ×3 artifacts R3 F-B1; root cause: demo-recorder hand-writing JSON values
  without deriving them from real serde output)

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

### PG-DEMO-JSON-FABRICATION — demo evidence JSON fabricated by hand

Feature-iec104 F5 Round 2 finding F5R2-02 (MEDIUM): FIX-F5-001 demo-evidence report
contained a fabricated JSON object with `"category": "Protocol"` — a variant that does not
exist in the `ThreatCategory` enum. The real variants are `Anomaly`, `Reconnaissance`, etc.

F5 Round 3 finding F-B1 (HIGH, `cycles/feature-iec104/convergence-trajectory.md`): three
FIX-P4-001 demo-evidence artifacts retained fabricated JSON with:
- `"category": "Protocol"` — non-existent variant
- `"verdict": "Anomaly"` — non-existent variant (`Verdict` variants: `Possible`, `Likely`,
  `Confirmed`)
- `"confidence": "High"` — wrong casing (serialized form is `"high"` per
  `rename_all = "lowercase"`)
- Wrong MITRE technique cited in the accompanying prose

Root cause: the demo-recorder agent was hand-writing JSON evidence fields by reasoning about
what "looks right" rather than capturing real `cargo run`/`cargo test` output. This produces
values that compile in prose but do not match the actual Rust enum serialization. The
adversarial reviewer correctly flags these as HIGH defects because fabricated evidence
undermines the traceability chain.

The feature-code was CONVERGED since Round 2 — all three F5 tail rounds (R3/R4/R5) were
consumed on doc-accuracy issues rooted in this fabrication pattern.

This is a feature-iec104 cycle-execution finding — DF-VALIDATION-001-exempt per the
in-process exemption (same pattern as STORY-165 Notes, STORY-166 Notes).

## Acceptance Criteria

### AC-175-001 (traces to PG-DEMO-JSON-FABRICATION — demo-evidence scrub gate extension)

`.factory/maintenance/demo-evidence-scrub-gate.md` is extended with a new "JSON Accuracy
Mandate" subsection. The subsection MUST state:

(a) **Source requirement:** All JSON blocks in demo-evidence files (`docs/demo-evidence/`
    and `.factory/demo-evidence/`) MUST be produced by capturing actual program output —
    `cargo run -- ... --format json`, `cargo test -- --nocapture`, or equivalent. Hand-written
    illustrative JSON is prohibited.

(b) **Enum variant accuracy:** JSON fields that carry enum-serialized values (e.g.
    `category`, `verdict`, `confidence`, `mitre_techniques` array entries, tactic strings)
    MUST use the exact serialized forms produced by serde. The source of truth is the running
    binary's output, not prose reasoning or memory.

(c) **Gate check:** Before committing demo-evidence JSON, the author MUST verify at least
    one enum-carrying field against actual program output. A one-line spot-check is
    sufficient (e.g. `cargo run -- ... --format json | jq '.findings[0].verdict'`).

(d) **Failure class:** An adversarial pass that finds a non-existent enum variant in
    committed demo-evidence JSON is a HIGH defect and requires a fix-PR before convergence
    can be declared.

Verification:
```bash
grep -n "JSON Accuracy\|enum variant\|hand-written\|serde" \
  .factory/maintenance/demo-evidence-scrub-gate.md
# Must emit non-empty output containing the new subsection
```

### AC-175-002 (traces to PG-DEMO-JSON-FABRICATION — delivery-doc currency protocol)

`.factory/maintenance/delivery-doc-currency-protocol.md` Step 3 (demo-evidence currency
sweep) is extended with a JSON accuracy check note. The note MUST state:

- For any demo-evidence JSON committed during the wave's delivery, verify at least one
  enum-carrying field against actual program output before the pre-adversarial currency
  sweep is declared complete.
- Reference: PG-DEMO-JSON-FABRICATION; see demo-evidence-scrub-gate.md §JSON Accuracy
  Mandate for the gate check procedure.

Verification:
```bash
grep -n "JSON\|enum\|PG-DEMO-JSON" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the new note
```

**Engine cross-reference:** The demo-recording skill update (automatic JSON capture from
real binary output rather than hand-writing) is an engine-level improvement tracked
separately in the vsdd-factory plugin. wirerust takes no engine action here beyond the
project-side mandates in AC-175-001 and AC-175-002.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Demo-evidence scrub gate JSON accuracy | `.factory/maintenance/demo-evidence-scrub-gate.md` (amend) | Documentation |
| Delivery-doc currency protocol Step 3 note | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | Documentation |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No `bin/` changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Demo evidence contains only prose descriptions, no JSON blocks | No spot-check required; JSON accuracy mandate applies only to JSON blocks |
| EC-002 | Demo evidence JSON contains only string/integer fields, no enum-carrying fields | Spot-check not required for those fields; enum-carrying fields (category, verdict, confidence, technique IDs) are the focus |
| EC-003 | Demo-recorder captures real output but output format changes in a later release | The captured output was correct at time of recording; CHANGELOG documents format changes; re-recording is required if format changes make prior evidence inaccurate |

## Tasks

1. **Extend demo-evidence-scrub-gate.md (AC-175-001):** Add "JSON Accuracy Mandate"
   subsection with (a) source requirement, (b) enum variant accuracy, (c) gate check,
   (d) failure class. Factory-artifacts branch commit.

2. **Extend delivery-doc-currency-protocol.md Step 3 (AC-175-002):** Add JSON accuracy
   check note referencing PG-DEMO-JSON-FABRICATION and the new scrub-gate subsection.
   Factory-artifacts branch commit.

3. **Register in STORY-INDEX.md:** Add STORY-175 row (draft, E-11, wave-TBD).
   Factory-artifacts branch commit.

> **Note for implementer:** Both ACs are factory-artifacts branch commits only — no develop
> PR, no CHANGELOG entry (no `src/`, `Cargo.toml`, or `bin/` changes). The STORY-INDEX
> registration is also factory-artifacts.

## Previous Story Intelligence

Lessons from STORY-166 (wave-75, E-11 governance, demo-evidence scrub scope extension):
STORY-166 AC-166-003 extended the scrub gate to cover `.factory/demo-evidence/` for new
captures. STORY-175 extends the same gate in a different dimension: JSON accuracy (content
correctness) rather than path scrub (host-path removal). The two mandates are complementary.

## Architecture Compliance Rules

- **No Rust source changes:** This story adds no files in `src/`, `tests/`, or `Cargo.toml`.
- **No bin/ changes:** No Python tools modified; no new CI gates.
- **factory-artifacts branch only:** All changes are documentation amendments to the
  `.factory/maintenance/` tree. No develop PR required.

## Notes

- **S-7.02 disposition:** Creating this story at draft status codifies a feature-iec104
  cycle-execution process gap (PG-DEMO-JSON-FABRICATION). Three confirmed occurrences
  across F5 rounds (F5R2-02 MEDIUM + F-B1 HIGH × 3 artifacts). Root cause documented
  in `cycles/feature-iec104/convergence-trajectory.md` Pass F5-R3.
- **DF-VALIDATION-001 gate:** PG-DEMO-JSON-FABRICATION is a feature-iec104 in-process
  execution finding. DF-VALIDATION-001-exempt per the in-process exemption (same pattern
  as STORY-165/166 Notes).
- **No behavioral contract required:** E-11 convention (no BCs authored; pending PO
  authorship per epics.md E-11).
- **Predecessor chain:** STORY-175 follows the E-11 S-7.02 pattern: STORY-163 → wave-73;
  STORY-164 → wave-74; STORY-165 → wave-75; STORY-166 → wave-75; STORY-175 →
  feature-iec104 cycle-close.

## Disposition

**Status:** superseded — routed upstream 2026-07-19

All ACs in this story address the demo-evidence JSON fabrication process gap
(PG-DEMO-JSON-FABRICATION). The root cause is in the vsdd-factory engine's demo-recorder
agent, which produces JSON by hand-reasoning rather than from real program output. Codifying
project-side mandates in wirerust `.factory/maintenance/` files is superseded by the
upstream engine fix.

| AC | Upstream Disposition |
|----|---------------------|
| AC-175-001 (demo-evidence-scrub-gate JSON Accuracy Mandate) | drbothen/vsdd-factory#494 evidence comment, 2026-07-19 |
| AC-175-002 (delivery-doc-currency Step 3 note) | drbothen/vsdd-factory#494 evidence comment, 2026-07-19 |

This story file is retained on disk for traceability. No further wirerust delivery expected.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-DEMO-JSON-FABRICATION (3 confirmed occurrences F5R2-02 + F-B1×3; AC-175-001 demo-evidence-scrub-gate JSON Accuracy Mandate + AC-175-002 delivery-doc-currency Step 3 note). |
