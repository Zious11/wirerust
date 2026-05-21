---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.002: Finding Display Renders [Category] VERDICT (CONFIDENCE) — summary

## Description

`Finding`'s `fmt::Display` implementation renders a one-line string in the format
`[{category}] {verdict} ({confidence}) — {summary}`. The category is the Debug string
of `ThreatCategory` (e.g., "Anomaly", "Reconnaissance"). The verdict and confidence are
their Display strings ("LIKELY", "MEDIUM", etc.). The separator before summary is ` — `
(U+2014 em-dash). This Display output is used for debugging and logging; terminal
rendering uses the reporter layer.

## Preconditions

1. A `Finding` value exists with any valid field values.
2. `fmt::Display` is invoked (e.g., via `format!("{finding}")` or `println!`).

## Postconditions

1. The formatted string matches: `"[{category:?}] {verdict} ({confidence}) — {summary}"`.
2. `category` renders via `{self:?}` (Debug format of ThreatCategory) -- e.g., "Anomaly".
3. `verdict` renders via Display -- "LIKELY", "UNLIKELY", or "INCONCLUSIVE".
4. `confidence` renders via Display -- "HIGH", "MEDIUM", or "LOW".
5. `summary` is included as-is (raw bytes, no escaping -- see ADR 0003).

## Invariants

1. The template is hardcoded: `[{cat}] {verdict} ({conf}) — {summary}`.
2. `ThreatCategory::fmt` uses `{self:?}` (Debug), which produces the variant name (e.g., "Anomaly").
3. The summary may contain control bytes; Display is NOT safe for direct terminal output.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | category=Anomaly, verdict=Likely, confidence=High | "[Anomaly] LIKELY (HIGH) — <summary>" |
| EC-002 | summary contains ESC byte 0x1B | ESC byte appears literally in formatted string |
| EC-003 | summary is empty string | "[Anomaly] LIKELY (HIGH) — " (trailing space after em-dash) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding { category: Anomaly, verdict: Likely, confidence: High, summary: "test" } | "[Anomaly] LIKELY (HIGH) — test" | happy-path |
| Finding { category: Reconnaissance, verdict: Inconclusive, confidence: Low, summary: "scan" } | "[Reconnaissance] INCONCLUSIVE (LOW) — scan" | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Display output matches expected format string | unit: assert_eq!(format!("{finding}"), "[Anomaly] LIKELY (HIGH) — test") |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 -- Display is the raw-text representation of a Finding, part of the CAP-09 emission contract |
| L2 Domain Invariants | INV-4 (raw bytes preserved in Display output; no escaping) |
| Architecture Module | SS-09 (findings.rs, C-14) |
| Stories | STORY-069 |
| Origin BC | BC-FND-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.003 -- composes with (Verdict::Display provides the VERDICT token)
- BC-2.09.004 -- composes with (Confidence::Display provides the CONFIDENCE token)
- BC-2.09.005 -- composes with (summary field is raw bytes per ADR 0003)

## Architecture Anchors

- `src/findings.rs:157-168` -- impl fmt::Display for Finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:157-168` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: write! macro format string is a compile-time constant

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

The Display impl is not safe for terminal rendering (raw bytes). This is intentional per
ADR 0003. The terminal reporter's `render_finding` function handles escaping.
