# ADR 0006: Multi-Technique Finding Attribution Model

**Status:** Accepted
**Date:** 2026-06-09
**Context:** v0.3.0 / STORY-100 (PR #209). The `Finding` struct carried a single
`mitre_technique: Option<String>` field. Adding the Modbus analyzer (v0.4.0) required
emitting findings that map to multiple simultaneous MITRE ATT&CK techniques (e.g., a
write-register PDU is simultaneously T1692.001 + T0836 + optionally T0831). A scalar
`Option<String>` cannot express co-attribution.

## Problem

ICS/OT threat findings frequently involve multiple concurrent MITRE ATT&CK techniques.
For example, a Modbus write-register PDU that is part of a burst pattern simultaneously:

- Constitutes an **unauthorized command** (T1692.001 — Unauthorized Message: Command Message)
- **Modifies a parameter** (T0836 — Modify Parameter)
- May also constitute **manipulation of control** (T0831 — Manipulation of Control) if a
  rolling window threshold is exceeded.

A scalar `Option<String>` forces a choice of one technique, which either loses attribution
or requires emitting duplicate findings for the same event. Either outcome harms analyst
usability and downstream SIEM correlation.

Additionally, the existing JSON field name `mitre_technique` (singular) does not align with
the Elastic Common Schema (ECS) field `threat.technique.id`, which is an array.

## Decision

**Replace `mitre_technique: Option<String>` with `mitre_techniques: Vec<String>` on the
`Finding` struct.**

Key sub-decisions:

1. **`Vec<String>`, not `Vec<MitreTechniqueId>`** — technique IDs are validated at emission
   sites, not by the type. This avoids a large enum of all technique IDs while keeping the
   type ergonomic for new analyzers.

2. **Empty vec serializes as absent key in JSON** — `#[serde(skip_serializing_if = "Vec::is_empty")]`
   keeps the JSON compact: findings with no MITRE attribution produce no `mitre_techniques` key.
   Findings with one technique produce `"mitre_techniques": ["T1027"]` (an array, not a scalar).

3. **Canonical emission order per analyzer** — each analyzer defines a stable ordering for its
   multi-technique vecs. For Modbus write findings the canonical order is:
   T0806 > T1692.001 > T0836 > T0835 > T0831 > T0814 > T0888 (threat severity descending,
   with T1692.001 always first for per-PDU write findings when T0806 is not co-emitted).
   This ordering is documented in inline comments at each emission site (ADR-006 §13.7 sub-decision 3).

4. **CSV: semicolon-join for multi-technique cells** — the CSV reporter joins the vec with `";"`,
   e.g. `T1692.001;T0836`. An empty vec produces an empty string (not `"null"` or `"[]"`).
   Downstream consumers must guard `if cell.is_empty() { return vec![] }` before splitting on `";"`.

5. **Terminal reporter: comma-space join** — the terminal reporter renders `MITRE: T1692.001, T0836`
   for multi-technique findings and groups by the first technique's tactic.

6. **JSON envelope fields** — every JSON report carries `"mitre_domain": "ics-attack"` and
   `"mitre_attack_version": "ics-attack-19.1"` in the top-level envelope to declare the ATT&CK
   matrix domain and pinned version.

## Alternatives Considered

### Keep `Option<String>`, emit multiple findings per event

Emit one `Finding` per technique for a multi-technique event.

- **Pro:** No schema change.
- **Con:** Duplicate findings for the same network event (same timestamp, same source IP, same
  summary) clutter analyst output. SIEM correlation must de-duplicate them. Analyst counts are
  inflated.
- **Rejected:** Multiple findings per event is worse for usability than a co-attribution vec.

### `mitre_technique: Option<String>` + `mitre_techniques_extra: Vec<String>`

Add an overflow field for additional techniques while keeping the primary scalar.

- **Con:** Two fields for the same concept. A finding with three techniques requires deciding
  which is "primary". The result is inconsistent.
- **Rejected:** Clean break to a single `Vec<String>` is simpler.

### A newtype `MitreTechniqueId(String)` for compile-time validation

Define a newtype that validates the `T\d+(\.\d+)?` pattern at construction.

- **Pro:** Catches typos at compile time.
- **Con:** Every new technique ID requires registering in the newtype's allowlist or using an
  unchecked constructor. The analyzer code already validates against the static catalog at the
  emission sites. The additional compile-time gate adds boilerplate without preventing runtime
  errors in the catalog.
- **Rejected:** Not justified for the current scale. Can be added if the technique-ID surface grows.

## Rationale

- **ECS alignment.** `threat.technique.id` in Elastic Common Schema is an array. Using `Vec<String>`
  makes wirerust JSON output compatible with ECS-based SIEM ingest without field remapping.
- **Forensic completeness.** ICS/OT attacks frequently span multiple ATT&CK techniques
  simultaneously. Losing attribution obscures the full attack picture for analysts.
- **Minimal migration.** The `serde` rename from `mitre_technique` to `mitre_techniques` and the
  type change from `Option<String>` to `Vec<String>` is the only API-breaking change; all other
  reporter and pipeline code adapts mechanically.

## Consequences

### Breaking changes (v0.3.0)

- **JSON:** `"mitre_technique": "T1027"` (string, may be absent) becomes `"mitre_techniques": ["T1027"]`
  (array, omitted when empty). Downstream JSON consumers must update field reads.
- **CSV:** Column 6 renamed from `mitre_technique` to `mitre_techniques`. Multi-value cells are
  semicolon-joined. Downstream consumers must split on `";"`.

### File-level changes

| File | Change |
|------|--------|
| `src/findings.rs` | `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` with `skip_serializing_if = "Vec::is_empty"`. See inline comment referencing this ADR (Decision 13). |
| `src/reporter/csv.rs` | `f.mitre_techniques.join(";")` with empty-vec guard. See inline comment referencing this ADR (Decision 13 §13.3). |
| `src/reporter/terminal.rs` | Render `MITRE: T1692.001, T0836` for multi-technique findings; group by first technique's tactic. |
| `src/analyzer/modbus.rs` | All emission sites use `mitre_techniques: vec![...]` with the canonical ordering documented inline. |
| All other analyzers | Migrate from `mitre_technique: Some("T1027")` to `mitre_techniques: vec!["T1027".to_string()]` at each emission site. |
