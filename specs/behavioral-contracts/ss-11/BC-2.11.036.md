---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-07-08T00:00:00Z
phase: 1a
inputs: []
input-hash: "d41d8cd"
extracted_from: null
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.12.0
modified:
  - "v1.1: F-W72-P2-004 (DF-AC-TEST-NAME-SYNC-001) — VP table row 8 test name corrected: `test_BC_2_11_036_terminal_display_unchanged_uppercase` → `test_BC_2_11_036_terminal_display_unchanged`. The `_uppercase` suffix was inaccurate: the VP row covers Display for Verdict, Confidence, AND ThreatCategory; ThreatCategory::fmt returns the Debug repr (PascalCase, e.g. 'LateralMovement'), not an UPPERCASE string. The property description ('Terminal Display for Verdict and Confidence is UNCHANGED (uppercase tokens)') correctly limits the uppercase characterization to Verdict/Confidence only and is unchanged. STORY-160 AC-160-005 already uses the corrected name; this BC amendment syncs the canonical source. STORY-160 input-hash will need rebaselining (story-writer). — 2026-07-08"
  - "v1.2: F-W72-P11-M03 (MEDIUM) — VP table row 8 covered-scope extended: test_BC_2_11_036_terminal_display_unchanged now explicitly asserts Display invariance for all three enums: Verdict UPPERCASE (Verdict::Likely.to_string() == 'LIKELY'), Confidence UPPERCASE (Confidence::High.to_string() == 'HIGH'), and ThreatCategory PascalCase Debug repr (ThreatCategory::LateralMovement.to_string() == 'LateralMovement'). Description sentence clarified: Verdict/Confidence produce UPPERCASE tokens; ThreatCategory::fmt uses write!(f, \"{self:?}\") returning PascalCase. Architecture Anchor added for impl fmt::Display for ThreatCategory. BC-INDEX row v1.2 annotation added. DF-SIBLING-SWEEP-001 sweep: BC-2.11.037 and BC-2.11.001 untouched (no terminal-display assertions therein). STORY-160 input-hash needs rebaselining (story-writer). — 2026-07-08"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
breaking: true
breaking_surface: JSON output only; terminal Display tokens are UNCHANGED
breaking_target: v0.12.0
breaking_note: >
  JSON schema is a governed surface outside cargo-semver-checks scope.
  CHANGELOG entry and BC enforcement are the authoritative change-notice mechanisms.
  cargo-semver-checks does NOT detect JSON value serialization changes.
---

# BC-2.11.036: JSON Enum Values Use `lowercase` (Verdict/Confidence) and `snake_case` (ThreatCategory); Terminal Display Tokens UNCHANGED

## Description

When `JsonReporter` serializes a `Finding`, the `verdict`, `confidence`, and `category` fields
use idiomatic JSON casing rather than the Rust PascalCase variant names present in the
pre-v0.12.0 schema:

- `Verdict` variants are rendered in **lowercase** via `#[serde(rename_all = "lowercase")]`
  on the `Verdict` enum (single-word variants; `"lowercase"` and `"snake_case"` are equivalent
  for single-word identifiers): `Likely` → `"likely"`, `Unlikely` → `"unlikely"`,
  `Inconclusive` → `"inconclusive"`, `Possible` → `"possible"`.

- `Confidence` variants are rendered in **lowercase** via `#[serde(rename_all = "lowercase")]`
  on the `Confidence` enum: `High` → `"high"`, `Medium` → `"medium"`, `Low` → `"low"`.

- `ThreatCategory` variants are rendered in **snake_case** via
  `#[serde(rename_all = "snake_case")]` on the `ThreatCategory` enum (multi-word variants
  are the primary motivation): `LateralMovement` → `"lateral_movement"`,
  `CredentialAccess` → `"credential_access"`, `C2` → `"c2"`, etc.

This matches the value conventions used by Suricata EVE JSON, Elastic Common Schema (ECS),
and the Open Cybersecurity Schema Framework (OCSF) — the dominant JSON event schemas in
security tooling. These conventions use lowercase/snake_case enum values, not PascalCase.

The terminal `Display` implementation for all three enums is **UNCHANGED**: `Verdict::fmt`
and `Confidence::fmt` continue to produce UPPERCASE tokens (e.g. `"LIKELY"`, `"HIGH"`);
`ThreatCategory::fmt` continues to return the PascalCase Debug repr via
`write!(f, "{self:?}")` (e.g. `"LateralMovement"`, `"CredentialAccess"`). The two serialization surfaces
are independent:
- `serde::Serialize` governs JSON field values (this BC).
- `fmt::Display` governs terminal output (BC-2.09.003 for Verdict, BC-2.09.004 for
  Confidence). Those BCs are separately locked and are NOT modified.

**This is a BREAKING change for the JSON surface at v0.12.0.** Any existing consumer that
pattern-matches exact enum string values in JSON output (e.g., `verdict == "Likely"`,
`category == "LateralMovement"`) must update to the new values. The JSON schema is a governed
surface **outside `cargo-semver-checks` scope** — this change must be documented in the
CHANGELOG and announced via the `schema_version` envelope field (BC-2.11.037).

## Complete Variant-to-Value Mapping

### Verdict (`rename_all = "lowercase"`)

| Rust Variant | Pre-v0.12.0 JSON | v0.12.0+ JSON |
|---|---|---|
| `Verdict::Likely` | `"Likely"` | `"likely"` |
| `Verdict::Unlikely` | `"Unlikely"` | `"unlikely"` |
| `Verdict::Inconclusive` | `"Inconclusive"` | `"inconclusive"` |
| `Verdict::Possible` | `"Possible"` | `"possible"` |

### Confidence (`rename_all = "lowercase"`)

| Rust Variant | Pre-v0.12.0 JSON | v0.12.0+ JSON |
|---|---|---|
| `Confidence::High` | `"High"` | `"high"` |
| `Confidence::Medium` | `"Medium"` | `"medium"` |
| `Confidence::Low` | `"Low"` | `"low"` |

### ThreatCategory (`rename_all = "snake_case"`)

| Rust Variant | Pre-v0.12.0 JSON | v0.12.0+ JSON |
|---|---|---|
| `ThreatCategory::Reconnaissance` | `"Reconnaissance"` | `"reconnaissance"` |
| `ThreatCategory::LateralMovement` | `"LateralMovement"` | `"lateral_movement"` |
| `ThreatCategory::C2` | `"C2"` | `"c2"` |
| `ThreatCategory::Exfiltration` | `"Exfiltration"` | `"exfiltration"` |
| `ThreatCategory::CredentialAccess` | `"CredentialAccess"` | `"credential_access"` |
| `ThreatCategory::Persistence` | `"Persistence"` | `"persistence"` |
| `ThreatCategory::Execution` | `"Execution"` | `"execution"` |
| `ThreatCategory::Anomaly` | `"Anomaly"` | `"anomaly"` |
| `ThreatCategory::Suspicious` | `"Suspicious"` | `"suspicious"` |
| `ThreatCategory::Impact` | `"Impact"` | `"impact"` |

Note: `C2` renders as `"c2"` because serde's snake_case algorithm lowercases the single
uppercase letter `C` and treats `2` as a non-alphabetic continuation (no word boundary),
producing `"c2"` rather than any hyphenated or underscored form. This is consistent with
how EVE/ECS represent abbreviated category names.

Note: Single-word `ThreatCategory` variants (Reconnaissance, Exfiltration, Persistence,
Execution, Anomaly, Suspicious, Impact) are unchanged from a visual standpoint relative to
a hypothetical `"lowercase"` rendering — snake_case and lowercase are equivalent for
single-word identifiers. The `"snake_case"` annotation on `ThreatCategory` is authoritative
and handles both current single-word and current multi-word variants uniformly.

## Preconditions

1. `JsonReporter::render` is called with a `&[Finding]` where at least one `Finding` is
   serialized.
2. The `Verdict` enum carries `#[serde(rename_all = "lowercase")]` in `src/findings.rs`.
3. The `Confidence` enum carries `#[serde(rename_all = "lowercase")]` in `src/findings.rs`.
4. The `ThreatCategory` enum carries `#[serde(rename_all = "snake_case")]` in
   `src/findings.rs`.

## Postconditions

1. Every serialized `Finding` JSON object has `"verdict": <lowercase-string>` where
   `<lowercase-string>` is the lowercase rendering of the `Verdict` variant (see mapping
   table above).
2. Every serialized `Finding` JSON object has `"confidence": <lowercase-string>` where
   `<lowercase-string>` is the lowercase rendering of the `Confidence` variant.
3. Every serialized `Finding` JSON object has `"category": <snake_case-string>` where
   `<snake_case-string>` is the snake_case rendering of the `ThreatCategory` variant.
4. No serialized `Finding` JSON object contains PascalCase enum values for `verdict`,
   `confidence`, or `category` — the pre-v0.12.0 forms (`"Likely"`, `"LateralMovement"`,
   etc.) MUST NOT appear in any JSON output produced under this BC.
5. The terminal Display output (`fmt::Display`) for `Verdict`, `Confidence`, and
   `ThreatCategory` is UNCHANGED. `TerminalReporter` output continues to render uppercase
   tokens per BC-2.09.003 (Verdict) and BC-2.09.004 (Confidence). `ThreatCategory::fmt`
   uses the Debug repr, which continues to return PascalCase (e.g. `"LateralMovement"`).
6. `CsvReporter` output is UNCHANGED by this BC. The CSV reporter uses `Display`/`Debug`
   formatting for its columns, not `Serialize`. The nine CSV columns defined by BC-2.11.020
   are unaffected.
7. The `serde(rename_all)` attributes apply to all current variants AND any future variants
   added to these enums without a per-variant override.

## Invariants

1. **Casing is annotation-driven, not hardcoded.** The rename is applied via serde
   `rename_all` on the enum declaration, not via custom `Serialize` impls or per-variant
   `#[serde(rename = "...")]` attributes. New variants inherit the casing rule automatically.
2. **JSON and terminal are independent surfaces.** `serde::Serialize` and `fmt::Display`
   are separate trait implementations. Changing `rename_all` on the derive attribute does
   NOT affect `fmt::Display` output. The terminal reporter reads the Display representation;
   it does NOT read the serde-serialized form. These surfaces cannot interfere.
3. **Exhaustive mapping.** The variant mapping tables above are exhaustive for the currently
   defined variants. If new variants are added to these enums, they automatically inherit
   the `rename_all` rule. The tables must be updated in a new BC version when new variants
   are added (to maintain BC-as-documentation accuracy), but no code change beyond the enum
   declaration is required.
4. **Breaking at v0.12.0 only.** This change occurs in a single hard-cutover release. There
   is no dual-output mode, no opt-in flag, and no deprecation period. The `schema_version`
   field (BC-2.11.037) serves as the machine-readable signal that consumers can use to
   distinguish the old format from the new.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `Verdict::Likely` serialized to JSON | `"verdict": "likely"` (not `"Likely"`) |
| EC-002 | `Verdict::Inconclusive` serialized to JSON | `"verdict": "inconclusive"` |
| EC-003 | `Confidence::High` serialized to JSON | `"confidence": "high"` |
| EC-004 | `Confidence::Low` serialized to JSON | `"confidence": "low"` |
| EC-005 | `ThreatCategory::LateralMovement` serialized to JSON | `"category": "lateral_movement"` |
| EC-006 | `ThreatCategory::CredentialAccess` serialized to JSON | `"category": "credential_access"` |
| EC-007 | `ThreatCategory::C2` serialized to JSON | `"category": "c2"` (single uppercase letter + digit) |
| EC-008 | Terminal rendering of `Verdict::Likely` | `"LIKELY"` (unchanged per BC-2.09.003) |
| EC-009 | Terminal rendering of `Confidence::High` | `"HIGH"` (unchanged per BC-2.09.004) |
| EC-010 | CSV output containing a `ThreatCategory::LateralMovement` finding | CSV column renders `"LateralMovement"` (Debug repr via Display) — CSV is UNCHANGED |
| EC-011 | All `Verdict` variants serialized to JSON | `["likely", "unlikely", "inconclusive", "possible"]` — no PascalCase forms present |
| EC-012 | All `Confidence` variants serialized to JSON | `["high", "medium", "low"]` — no PascalCase forms present |
| EC-013 | All `ThreatCategory` variants serialized to JSON | See full mapping table — no PascalCase forms present |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with `verdict: Verdict::Likely` serialized via JsonReporter | `"verdict": "likely"` in JSON output | happy-path (breaking change confirmed) |
| Finding with `verdict: Verdict::Possible` serialized via JsonReporter | `"verdict": "possible"` | happy-path |
| Finding with `confidence: Confidence::High` serialized via JsonReporter | `"confidence": "high"` | happy-path |
| Finding with `category: ThreatCategory::LateralMovement` serialized via JsonReporter | `"category": "lateral_movement"` | happy-path (multi-word snake_case confirmed) |
| Finding with `category: ThreatCategory::C2` serialized via JsonReporter | `"category": "c2"` | edge-case (abbreviation + digit, no underscore) |
| Finding with `category: ThreatCategory::CredentialAccess` serialized via JsonReporter | `"category": "credential_access"` | happy-path (multi-word snake_case confirmed) |
| `Verdict::Likely` formatted via `fmt::Display` | `"LIKELY"` | surface-independence (terminal unchanged) |
| `Confidence::High` formatted via `fmt::Display` | `"HIGH"` | surface-independence (terminal unchanged) |
| CsvReporter with `ThreatCategory::LateralMovement` finding | CSV column contains `"LateralMovement"` (Debug repr) | surface-independence (CSV unchanged) |
| All 4 Verdict variants serialized to JSON in one report | `["likely", "unlikely", "inconclusive", "possible"]` (no PascalCase) | exhaustive-coverage |
| All 7 Confidence+Verdict combinations — no PascalCase remains | Full report has no `"Likely"`, no `"High"`, no `"LateralMovement"` in JSON | regression guard |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `Verdict::Likely` serializes to `"likely"` (not `"Likely"`) in JSON output | unit: test_BC_2_11_036_verdict_likely_serializes_lowercase |
| — | All four `Verdict` variants serialize to their lowercase form; no PascalCase variant present | unit: test_BC_2_11_036_verdict_all_variants_lowercase |
| — | `Confidence::High` serializes to `"high"` (not `"High"`) in JSON output | unit: test_BC_2_11_036_confidence_high_serializes_lowercase |
| — | All three `Confidence` variants serialize to their lowercase form | unit: test_BC_2_11_036_confidence_all_variants_lowercase |
| — | `ThreatCategory::LateralMovement` serializes to `"lateral_movement"` in JSON output | unit: test_BC_2_11_036_threat_category_lateral_movement_snake_case |
| — | `ThreatCategory::C2` serializes to `"c2"` (not `"C2"`) in JSON output | unit: test_BC_2_11_036_threat_category_c2_snake_case |
| — | All ten `ThreatCategory` variants serialize to their snake_case form; no PascalCase variant present | unit: test_BC_2_11_036_threat_category_all_variants_snake_case |
| — | Terminal Display for all three enums is UNCHANGED: `Verdict`/`Confidence` produce UPPERCASE tokens (e.g. `Verdict::Likely.to_string() == "LIKELY"`); `ThreatCategory` produces PascalCase Debug repr (e.g. `ThreatCategory::LateralMovement.to_string() == "LateralMovement"`) | unit: test_BC_2_11_036_terminal_display_unchanged |
| — | CSV output for `ThreatCategory` is UNCHANGED (Debug repr PascalCase) | unit: test_BC_2_11_036_csv_category_unchanged |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 — this BC governs the machine-readable JSON value format for three core finding fields (`verdict`, `confidence`, `category`), which is a direct output-quality and interoperability contract of the Reporting capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — serde Serialize and fmt::Display are independent surfaces; this BC amends the Serialize surface only; Display is unchanged) |
| Architecture Module | SS-11 (reporter/json.rs via serde derive on enums in src/findings.rs) |
| Stories | TBD (story-writer assigns) |
| Issue | #255 (snake_case JSON enums) |
| ADR | None required (additive serde annotation change; consistent with ADR-0003 serde delegation pattern) |

## Related BCs

- BC-2.09.003 — terminal Verdict Display contract (uppercase tokens, UNCHANGED by this BC)
- BC-2.09.004 — terminal Confidence Display contract (uppercase tokens, UNCHANGED by this BC)
- BC-2.11.001 — JSON envelope shape (advisory pointer in v1.8; schema_version envelope field added in BC-2.11.037)
- BC-2.11.037 — schema_version envelope field; provides consumers with a machine-readable signal to distinguish pre-v0.12.0 (PascalCase) from v0.12.0+ (lowercase/snake_case) JSON output

## Architecture Anchors

- `src/findings.rs` — `Verdict` enum declaration: add `#[serde(rename_all = "lowercase")]`
  to the `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]` attribute group
- `src/findings.rs` — `Confidence` enum declaration: add `#[serde(rename_all = "lowercase")]`
  to the `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]` attribute group
- `src/findings.rs` — `ThreatCategory` enum declaration: add
  `#[serde(rename_all = "snake_case")]` to the
  `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]` attribute group
- `src/findings.rs` — `impl fmt::Display for Verdict`: UNCHANGED (UPPERCASE tokens)
- `src/findings.rs` — `impl fmt::Display for Confidence`: UNCHANGED (UPPERCASE tokens)
- `src/findings.rs` — `impl fmt::Display for ThreatCategory`: UNCHANGED (PascalCase Debug repr via `write!(f, "{self:?}")`, e.g. `"LateralMovement"`)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (serde rename_all is a compile-time annotation; serialization is pure) |
| **Global state access** | none |
| **Deterministic** | yes — serde rename_all produces a fixed compile-time mapping; the output for a given variant is always identical |
| **Thread safety** | Send + Sync (enum values are Copy; no mutable state) |
| **Overall classification** | pure |
