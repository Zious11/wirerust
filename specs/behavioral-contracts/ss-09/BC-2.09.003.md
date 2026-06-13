---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/findings.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Pass-19 B-03 — re-anchor Architecture Anchor and Source Evidence: src/findings.rs:43-50 → :48-57 (impl fmt::Display for Verdict shifted after #[non_exhaustive] enum expansion). Content gap B-03 resolved: Verdict::Possible variant (line 45, renders POSSIBLE, line 54) added to Description, Postconditions, Invariants, Edge Cases, and Canonical Test Vectors. Verified against HEAD findings.rs. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.003: Verdict Display: Uppercase Tokens

## Description

`Verdict` implements `fmt::Display` with uppercase string representations: `Likely` => "LIKELY",
`Unlikely` => "UNLIKELY", `Inconclusive` => "INCONCLUSIVE", `Possible` => "POSSIBLE". These
tokens appear in both `Finding::Display` output and in the terminal reporter's colorized
rendering. The uppercase convention is part of the wire-visible output contract. `Verdict` is
`#[non_exhaustive]` with four current variants; `Possible` was added in STORY-109 for
inferred/anomaly detections (BC-2.15.014, BC-2.15.018, BC-2.15.019, BC-2.15.023, BC-2.15.024).

## Preconditions

1. A `Verdict` value is formatted via Display.

## Postconditions

1. `Verdict::Likely` displays as "LIKELY".
2. `Verdict::Unlikely` displays as "UNLIKELY".
3. `Verdict::Inconclusive` displays as "INCONCLUSIVE".
4. `Verdict::Possible` displays as "POSSIBLE".
5. No other strings are produced by the four current variants.

## Invariants

1. The strings are hardcoded in the match arms; they do not depend on the `Debug` derive.
2. `Verdict` is `#[non_exhaustive]` with four current variants: `Likely`, `Unlikely`, `Inconclusive`, `Possible`; future variants must add Display arms.
3. Lowercase input to constructors does not affect Display (the enum is not string-parsed here).
4. `Verdict::Possible` is a lower-confidence signal than `Likely`; it renders as "POSSIBLE" (src/findings.rs:54). The terminal reporter sorts `Possible` between `Likely` (rank 0) and `Inconclusive` (rank 2) in tactic-grouped view (terminal.rs:287-293, `verdict_rank` fn; Possible → rank 1 at terminal.rs:290).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Verdict::Likely | "LIKELY" |
| EC-002 | Verdict::Unlikely | "UNLIKELY" |
| EC-003 | Verdict::Inconclusive | "INCONCLUSIVE" |
| EC-004 | Verdict::Possible | "POSSIBLE" — lower-confidence signal used for inferred/anomaly detections (STORY-109; DNP3 BC-2.15.014, BC-2.15.018, BC-2.15.019, BC-2.15.023, BC-2.15.024) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", Verdict::Likely) | "LIKELY" | happy-path |
| format!("{}", Verdict::Unlikely) | "UNLIKELY" | happy-path |
| format!("{}", Verdict::Inconclusive) | "INCONCLUSIVE" | happy-path |
| format!("{}", Verdict::Possible) | "POSSIBLE" | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All Verdict variants produce expected uppercase strings | unit: exhaustive assert on each variant |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md -- Verdict display is part of the Finding output vocabulary defined in CAP-09 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-09 (findings.rs, C-14) |
| Stories | STORY-069 |
| Origin BC | BC-FND-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.002 -- composes with (Verdict Display is used in Finding Display template)

## Architecture Anchors

- `src/findings.rs:32-46` -- `enum Verdict` with four variants (Likely, Unlikely, Inconclusive, Possible)
- `src/findings.rs:48-57` -- impl fmt::Display for Verdict

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:48-57` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: hardcoded string literals in match arms

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
