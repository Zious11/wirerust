---
document_type: story
story_id: STORY-159
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-08T00:00:00Z
phase: f7
level: feature
cycle: maint-2026-07-08
points: 3
priority: P3
depends_on: []
blocks: []
# BC status: none — E-11 convention (no BCs authored; status: draft, pending PO authorship)
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: docs/adr/0012-protocols-catalog-and-coverage-gaps.md
subsystems: []
estimated_days: 1
wave: "72"
traces_to:
  - .factory/maintenance/doc-drift-findings.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
input-hash: "f4a8f03"
inputs:
  - .factory/maintenance/doc-drift-findings.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
---

# STORY-159: Author Public ADR-012 — Protocols Catalog and Coverage-Gaps System

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3
**Priority:** P3

## Narrative

- **As a** contributor or maintainer reading wirerust source code
- **I want** `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` to exist with a full
  account of the ten design decisions it documents
- **So that** any of the 38 inline `ADR-012 Decision N` citations found across
  `src/protocols.rs`, `src/dispatcher.rs`, `src/main.rs`, `tests/protocols_tests.rs`,
  and `tests/dispatcher_tests.rs` can be resolved by a reader without access to the
  factory specification layer

## Behavioral Contracts

_(none — E-11 convention: no BCs authored yet; status: draft, pending PO authorship)_

## Background

Maintenance sweep `maint-2026-07-08` (finding NEW-001, HIGH) identified that ADR-012 is
cited 38 times across five source and test files but no corresponding public document
exists in `docs/adr/`. The `docs/adr/` directory contains `0001`–`0007`, `0009`–`0011`;
ADR-012 is the current missing entry (ADR-008 was intentionally skipped in sequence).

The authoritative factory-side record is
`.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`,
accepted 2026-07-01 with ten decisions (status: accepted, feature cycle
`feature-protocol-coverage`, issue D-320). The ARCH-INDEX v2.12 ADR table already
carries a summary row for ADR-012 listing affected subsystems SS-18, SS-05, SS-12.

This is the same class as the resolved Sweep-2 finding DOC-002 (ADR-009 missing —
fixed by PR #305, STORY delivered at that time). The fix pattern is identical: author
the public ADR doc from the factory-side source, clean of internal factory IDs, then
add the CLAUDE.md Project References row.

### Finding NEW-001 — source citations recovered

Representative inline citations from the five affected files:

| File | Line | Citation |
|------|------|---------|
| `src/protocols.rs` | ~13 | `(ADR-012 Decision 7)` |
| `src/protocols.rs` | ~69 | `(ADR-012 Decision 5)` |
| `src/dispatcher.rs` | ~44 | `(ADR-012 Decision 6)` |
| `src/dispatcher.rs` | ~98 | `(ADR-012 Decision 6 Clarification)` |
| `src/main.rs` | ~195 | `(ADR-012 Decision 9)` |

Full sweep: `grep -rn "ADR-012" src/ tests/` — 38 occurrences total.
Decision numbers cited in source: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 (all ten decisions
from the factory ADR are referenced somewhere in the codebase).

### Factory ADR-012 Decisions Summary (authoritative content)

The factory ADR-012 records ten non-obvious design choices made during the
`feature-protocol-coverage` cycle (D-320 OQ-1..OQ-5):

| Decision | Title |
|----------|-------|
| 1 | Hand-Curated Static Compile-Time Array |
| 2 | Tri-State Vocabulary (Suricata-Derived) |
| 3 | Port-Based Detection Caveats (four sub-items: 3a transport scope, 3b port-102 collision, 3c L2/multicast no-port, 3d heuristic caveat) |
| 4 | Catalog Scope — ICS + Core-IT |
| 5 | Supported-Set Derivation — Static `SUPPORTED_PORTS` |
| 6 | TCP+UDP Dynamic Detection (D-320 OQ-5 Approved Scope) with Decision 6 Clarification on increment-site semantics |
| 7 | Category Tagging (`ProtocolCategory` = `{ICS, IT}` only; no `L2` variant) |
| 8 | `--coverage-gaps` Explicit Flag (not auto-enabled under `--all`) |
| 9 | `CoverageGapsSummary` as New Report Section (not individual Finding entries) |
| 10 | UDP Gap Classification Decoupled from `enable_dns` |

## Acceptance Criteria

### AC-159-001 (public ADR file exists)

`docs/adr/0012-protocols-catalog-and-coverage-gaps.md` exists at HEAD and follows
the public ADR format established by `docs/adr/0009-pcapng-reader-design.md`:
- Markdown headings, no YAML frontmatter
- Human-readable **Status**, **Date**, and **Context** preamble
- One subsection per decision with a brief rationale
- No internal factory IDs (no `BC-2.NN.NNN`, `VP-NNN`, `STORY-NNN`, `F-*`, `D-NNN`,
  or `.factory/` paths) — ADR cross-references like "ADR-0001" or "ADR-0009" are
  acceptable, matching the ADR-0009 precedent

### AC-159-002 (all ten decisions covered)

The public doc contains a labelled section (or subsection) for every decision in the
factory ADR-012: Decision 1 through Decision 10, including the Decision 6
Clarification on increment-site semantics. The grep command:

```bash
for n in 1 2 3 4 5 6 7 8 9 10; do
  grep -q "Decision $n" docs/adr/0012-protocols-catalog-and-coverage-gaps.md \
    || { echo "MISSING: Decision $n"; exit 1; }
done
echo "All ten decisions present"
```

must exit 0.

### AC-159-003 (all 38 inline source citations resolvable)

Every decision number referenced in the source files (`ADR-012 Decision N` where
`N` ∈ {1..10}) resolves to a section in the authored public doc. Verification:

```bash
# Extract unique decision numbers cited in source
CITED=$(grep -roh "ADR-012 Decision [0-9]*" src/ tests/ \
  | grep -oP '\d+$' | sort -nu)
# Verify each is in the public doc
for n in $CITED; do
  grep -q "Decision $n" docs/adr/0012-protocols-catalog-and-coverage-gaps.md \
    || { echo "UNRESOLVED: Decision $n"; exit 1; }
done
echo "All cited decisions resolvable"
```

### AC-159-004 (CLAUDE.md Project References row added)

`CLAUDE.md` is updated to add `0012 protocols catalog and coverage-gaps system` to
the `docs/adr/` row in the Project References table. After the fix the row reads:

```
| `docs/adr/` | Architecture Decision Records (0001 stream dispatch, 0002 modular analyzers, 0003 reporting pipeline, 0004 process-wide warning atomics, 0005 binary ICS protocol integration, 0006 multi-technique finding attribution, 0007 DNP3 stream dispatch and parser design, 0009 pcapng reader design, 0010 EtherNet/IP CIP stream dispatch, 0011 TLS handshake reassembly, 0012 protocols catalog and coverage-gaps system) |
```

### AC-159-005 (PR type)

The pull request title uses the `docs:` semantic prefix (e.g.,
`docs: add ADR-012 protocols catalog and coverage-gaps`), consistent with the
finding's suggested action and the `docs:` type used for prior ADR authoring work.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Public ADR document | `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` (create) | Documentation artifact |
| CLAUDE.md ADR row | `CLAUDE.md` (amend Project References table) | Documentation artifact |

No production Rust source files are modified. No tests are added or changed.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` | Documentation artifact | Markdown only |
| `CLAUDE.md` | Documentation artifact | One-line amendment to Project References table |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Factory ADR-012 cites `VP-042(d)` precondition — an internal VP reference | Omit; document the architectural consequence (dual-gate co-increment) in plain language, no `VP-*` ID |
| EC-002 | Factory ADR-012 references `BC-2.05.010 Architecture Anchor wording` | Omit; document the architectural decision (analyzer-present guard, increment semantics) without BC IDs |
| EC-003 | Factory ADR Decision 6 Clarification has a detailed code block | Include the implementation sketch (code block) as-is — it is architecture-level content, not a BC or test reference |
| EC-004 | `docs/adr/0008-*.md` gap in sequence | Do not fill in ADR-008; the sequence jumps from 0007 to 0009 intentionally; 0012 is simply the next authored file after 0011 |

## Tasks

1. **Read factory ADR-012** at
   `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
   in its entirety. It is the canonical source for all decision content.

2. **Author the public doc** `docs/adr/0012-protocols-catalog-and-coverage-gaps.md`
   following the format of `docs/adr/0009-pcapng-reader-design.md`. Cover all ten
   decisions plus the Decision 6 Clarification. Strip all internal factory IDs
   (`BC-*`, `VP-*`, `STORY-*`, `F-F*`, `D-NNN`, `.factory/` paths). The
   Consequences section of the factory ADR is implementation detail, not a decision —
   summarize relevant architecture-level consequences inline within each decision
   rather than as a standalone Consequences section.

3. **Run the AC-159-002 and AC-159-003 verification scripts** and fix any missing
   sections before proceeding.

4. **Update `CLAUDE.md`** Project References table: append
   `, 0012 protocols catalog and coverage-gaps system` to the `docs/adr/` row
   description (inside the existing parenthesized list, after `0011 TLS handshake
   reassembly`).

5. **Open a `docs:` pull request** targeting `develop` with both file changes.

## Previous Story Intelligence

Lessons from closest analogues:

- **DOC-002 / ADR-009 gap (Sweep 2, PR #305):** Same class of finding — public ADR
  cited in reader.rs but file absent. Fix: author from factory spec, PR title
  `docs: add ADR-0009 pcapng reader design`, merged. Zero code changes.
- **STORY-157 (wave-70 process-gap codifications, 5 pts):** Input-hash workflow
  established; `inputs:` declares real spec evidence files. Follow the same pattern.
- **STORY-158 (wave-71 process-gap codifications, 3 pts):** Same E-11 wave-TBD
  pattern; `cycle: maint-*` naming.

## Architecture Compliance Rules

- This story modifies ONLY: `docs/adr/0012-protocols-catalog-and-coverage-gaps.md`
  (create) and `CLAUDE.md` (one-line amendment). No Rust source, no tests,
  no CI configuration.
- The public ADR content is derived from the factory ADR-012 — do not invent or
  paraphrase decisions. Transcribe accurately, then strip internal IDs.
- ADR-012 sequence number (0012) follows 0011. ADR-008 was intentionally skipped;
  do not insert a 0008 file.

## Library & Framework Requirements

None. Markdown authoring only; no third-party tools required beyond a text editor.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` | Create | From factory ADR-012; internal IDs stripped; ten decisions + Decision 6 Clarification |
| `CLAUDE.md` | Modify | Append `, 0012 protocols catalog and coverage-gaps system` to `docs/adr/` row |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~3 k |
| `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` (~350 lines) | ~5 k |
| `CLAUDE.md` (one-line amendment) | ~0.1 k |
| **Total** | **~8.1 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** Finding NEW-001 originates from a maint-2026-07-08
  technical-writer sweep (not an open deferred finding from a prior session).
  It is a directly-observed, in-process sweep finding. Per DF-VALIDATION-001 scope
  ("deferred or open findings"), a dedicated research-agent validation pass is not
  required before creating this story. Findings from sweep reports are validated by
  the sweep process itself.
- **Factory ADR location:** The factory ADR-012 resides at
  `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
  (confirmed in ARCH-INDEX v2.12 ADR table). It is NOT in `docs/adr/` — factory-side
  ADRs (ADR-005 onwards) live in `.factory/specs/architecture/decisions/`; only
  ADRs 0001–0004 plus public-facing summaries live in `docs/adr/`.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet — status: draft; pending PO authorship").
- **PR title example:** `docs: add ADR-012 protocols catalog and coverage-gaps system`
- Source: finding NEW-001 (HIGH, MANUAL) from `.factory/maintenance/doc-drift-findings.md`,
  maint-2026-07-08 Sweep 3.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-08 | story-writer | Initial authorship — maint-2026-07-08 NEW-001 follow-up: author public docs/adr/0012-protocols-catalog-and-coverage-gaps.md from factory ADR-012; add CLAUDE.md Project References row; verify all 38 inline ADR-012 citations resolve. |
