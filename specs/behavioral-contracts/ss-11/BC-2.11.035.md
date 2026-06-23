---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-22T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.11.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.035: Per-Finding `mitre_attack` Array Enriches JSON Output with Resolved Technique Objects; Order-Preserving; Unknown IDs Emit Partial Objects; Empty Vec Omits Field

## Description

When `JsonReporter` serializes a `Finding`, any non-empty `mitre_techniques` vec causes a
companion `"mitre_attack"` array to appear in the JSON object. Each element corresponds to one
technique ID in `mitre_techniques`, in the same positional order, and carries five fields:
`id`, `name`, `tactic_id`, `tactic_name`, and `reference`. For IDs that resolve in the
catalog, all five fields are present. For IDs that do NOT resolve, `id` is always emitted
(agent-safety invariant) while `name`, `tactic_id`, and `tactic_name` are omitted via
`skip_serializing_if`; `reference` is always synthesized from the ID regardless of resolution
status. When `mitre_techniques` is empty, `mitre_attack` is omitted entirely, preserving the
`skip_serializing_if = Vec::is_empty` additive-non-breaking contract of BC-2.09.006. The raw
`mitre_techniques` array is unchanged (additive, non-breaking). The envelope field
`mitre_attack_version` (BC-2.11.001) communicates the catalog version; no per-element version
field is emitted.

## Catalog Extension Required

The mitre module (`src/mitre.rs`) currently exposes `technique_info(id) -> Option<(&'static str, MitreTactic)>`.
The `MitreTactic` enum has no associated TA-prefix ID string; the catalog merges by tactic name
across matrices (Enterprise vs ICS). A new accessor `technique_tactic_id(id: &str) -> Option<&'static str>`
must be added to `src/mitre.rs` as part of this feature's implementation scope. This accessor
maps each `MitreTactic` variant to its canonical MITRE ATT&CK tactic ID string:

| `MitreTactic` variant          | Canonical tactic_id |
|-------------------------------|---------------------|
| Reconnaissance                 | TA0043              |
| ResourceDevelopment            | TA0042              |
| InitialAccess                  | TA0001              |
| Execution                      | TA0002              |
| Persistence                    | TA0003              |
| PrivilegeEscalation            | TA0004              |
| DefenseEvasion                 | TA0005              |
| CredentialAccess               | TA0006              |
| Discovery                      | TA0007              |
| LateralMovement                | TA0008              |
| Collection                     | TA0009              |
| CommandAndControl              | TA0011              |
| Exfiltration                   | TA0010              |
| Impact                         | TA0040              |
| IcsInhibitResponseFunction     | TA0107              |
| IcsImpairProcessControl        | TA0106              |
| IcsImpact                      | TA0105              |

The `tactic_id` field on a technique object is `skip_serializing_if = "Option::is_none"` — it
is omitted only when `technique_tactic_id` returns `None` (which cannot happen for any
currently-seeded ID but is possible for future `MitreTactic` variants added before this
mapping is updated).

The `reference` URL is NOT sourced from the catalog but synthesized at serialization time:
`format!("https://attack.mitre.org/techniques/{}/", id)`. This synthesis is performed for ALL
technique IDs — both resolved and unresolved — because the URL is deterministically derivable
from the ID alone and is always useful to consumers regardless of catalog completeness.

## Preconditions

1. `JsonReporter::render` is called with a `&[Finding]` where at least one `Finding` has a
   non-empty `mitre_techniques: Vec<String>`.
2. The `technique_info(id)` catalog function (existing) is available.
3. A new `technique_tactic_id(id: &str) -> Option<&'static str>` function (catalog extension,
   in-scope for this feature) is available in `src/mitre.rs`.
4. The `Finding` struct exposes the `mitre_techniques` field for serialization.
5. The `MitreTactic` enum's `Display` impl returns the canonical tactic name string.

## Postconditions

1. For each `Finding` with a non-empty `mitre_techniques` vec, the JSON object for that
   finding includes a `"mitre_attack"` key whose value is a JSON array.
2. The `"mitre_attack"` array has exactly `mitre_techniques.len()` elements, in the same
   positional order as `mitre_techniques`.
3. For each element at index `i` (corresponding to `mitre_techniques[i]` = `id`):
   a. `"id"`: always present. Value is `id` verbatim (the technique ID string as-is from the
      `mitre_techniques` vec — never transformed).
   b. `"name"`: present if and only if `technique_info(id)` returns `Some((name, _))`. Value
      is the `name` `&'static str`. Omitted (via `skip_serializing_if`) when `None`.
   c. `"tactic_id"`: present if and only if `technique_tactic_id(id)` returns `Some(ta_id)`.
      Value is the TA-prefix ID string (e.g., `"TA0007"`). Omitted when `None`.
   d. `"tactic_name"`: present if and only if `technique_info(id)` returns `Some((_, tactic))`.
      Value is `tactic.to_string()` — the `MitreTactic::Display` string (e.g.,
      `"Discovery"`, `"Impact (ICS)"`). Omitted when `None`.
   e. `"reference"`: always present. Value is
      `format!("https://attack.mitre.org/techniques/{}/", id)` where any `.` separator in
      sub-technique IDs is preserved verbatim (e.g., `T1071.001` →
      `"https://attack.mitre.org/techniques/T1071.001/"`).
4. For each `Finding` with `mitre_techniques = vec![]` (empty), the `"mitre_attack"` key is
   entirely absent from the JSON object (`skip_serializing_if = Vec::is_empty` semantics,
   mirroring the existing `mitre_techniques` omission behavior of BC-2.09.006).
5. The existing `"mitre_techniques"` JSON array is UNCHANGED — it continues to be
   `skip_serializing_if = Vec::is_empty`, emitting the raw ID strings exactly as before.
   BC-2.11.035 is purely additive.
6. The `"mitre_attack"` field does NOT appear in CSV output. The `CsvReporter` is unaffected
   by this BC (CSV has no envelope or nested object concept).
7. The `"mitre_attack"` field does NOT appear in terminal output. The `TerminalReporter` is
   unaffected by this BC.

## Invariants

1. **`id` is never lost.** For any technique ID in `mitre_techniques`, the corresponding
   `mitre_attack` element always emits `"id": "<technique_id>"`, regardless of whether the
   ID resolves in the catalog. An unknown ID produces `{"id": "<id>", "reference": "<url>"}`
   — never an empty object and never a missing element.
2. **Order is preserved.** `mitre_attack[i].id == mitre_techniques[i]` for all `i` in
   `0..mitre_techniques.len()`. The serialization MUST NOT reorder, deduplicate, or filter
   elements.
3. **Duplicates are mirrored.** If `mitre_techniques` contains duplicate IDs (e.g.,
   `["T1046", "T1046"]`), `mitre_attack` contains two elements, each with `id: "T1046"`.
   The implementer does not deduplicate.
4. **`reference` is synthesized, not cataloged.** The URL is always
   `"https://attack.mitre.org/techniques/<id>/"` derived algorithmically. No catalog lookup
   is performed for `reference`.
5. **Additive non-breaking.** The `mitre_techniques` field and all other `Finding` fields
   are serialized identically to their pre-BC-2.11.035 form. No existing key is renamed,
   removed, or reordered.
6. **Catalog extension is atomic with this feature.** `technique_tactic_id` MUST be added to
   `src/mitre.rs` in the same story that implements this BC. The Kani proof set (VP-007) and
   the `vp007_catalog_drift_guard` test are updated to cover the new accessor.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `mitre_techniques = []` (empty vec) | `"mitre_attack"` key is ABSENT from the finding JSON object; `"mitre_techniques"` key is also absent (BC-2.09.006 existing behavior). Both omissions use `skip_serializing_if`. |
| EC-002 | `mitre_techniques = ["T1046"]` — known Enterprise technique | `"mitre_attack": [{"id": "T1046", "name": "Network Service Discovery", "tactic_id": "TA0007", "tactic_name": "Discovery", "reference": "https://attack.mitre.org/techniques/T1046/"}]` |
| EC-003 | `mitre_techniques = ["T0827"]` — known ICS technique (IcsImpact) | `"mitre_attack": [{"id": "T0827", "name": "Loss of Control", "tactic_id": "TA0105", "tactic_name": "Impact (ICS)", "reference": "https://attack.mitre.org/techniques/T0827/"}]` |
| EC-004 | `mitre_techniques = ["T9999"]` — unknown/unmapped ID | `"mitre_attack": [{"id": "T9999", "reference": "https://attack.mitre.org/techniques/T9999/"}]` — `name`, `tactic_id`, `tactic_name` are absent; `id` and `reference` are present. |
| EC-005 | `mitre_techniques = ["T1071.001"]` — sub-technique with dot separator | `"mitre_attack": [{"id": "T1071.001", "name": "Web Protocols", "tactic_id": "TA0011", "tactic_name": "Command and Control", "reference": "https://attack.mitre.org/techniques/T1071.001/"}]` — dot preserved verbatim in both `id` and the reference URL. |
| EC-006 | `mitre_techniques = ["T1692.001", "T0836"]` — multi-tag, two known ICS techniques | `"mitre_attack": [{"id": "T1692.001", "name": "Unauthorized Message: Command Message", "tactic_id": "TA0106", "tactic_name": "Impair Process Control", "reference": "https://attack.mitre.org/techniques/T1692.001/"}, {"id": "T0836", "name": "Modify Parameter", "tactic_id": "TA0106", "tactic_name": "Impair Process Control", "reference": "https://attack.mitre.org/techniques/T0836/"}]` — two elements in declaration order; same tactic for both. |
| EC-007 | `mitre_techniques = ["T1046", "T9999", "T1046"]` — mix of known, unknown, and duplicate | Three elements in order: index 0 fully resolved, index 1 partial (id+reference only), index 2 fully resolved (duplicate of index 0). No deduplication. |
| EC-008 | Finding with no `mitre_techniques` key in JSON output (EC-001 situation) in a report with other findings that DO have `mitre_attack` arrays | `"mitre_attack"` absent only on the empty-vec finding; other findings unaffected. Batch serialization is per-finding independent. |
| EC-009 | `mitre_techniques = ["T1557.002"]` — Enterprise sub-technique (ARP, CredentialAccess) | `"mitre_attack": [{"id": "T1557.002", "name": "Adversary-in-the-Middle: ARP Cache Poisoning", "tactic_id": "TA0006", "tactic_name": "Credential Access", "reference": "https://attack.mitre.org/techniques/T1557.002/"}]` |
| EC-010 | `mitre_techniques = ["T0830"]` — ICS lateral movement (LateralMovement, same tactic-id as Enterprise LateralMovement) | `"mitre_attack": [{"id": "T0830", "name": "Adversary-in-the-Middle", "tactic_id": "TA0008", "tactic_name": "Lateral Movement", "reference": "https://attack.mitre.org/techniques/T0830/"}]` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with `mitre_techniques: vec!["T1046".to_string()]` | JSON finding object contains `"mitre_techniques": ["T1046"]` and `"mitre_attack": [{"id": "T1046", "name": "Network Service Discovery", "tactic_id": "TA0007", "tactic_name": "Discovery", "reference": "https://attack.mitre.org/techniques/T1046/"}]` | happy-path (known single technique) |
| Finding with `mitre_techniques: vec![]` (empty) | JSON finding object lacks both `"mitre_techniques"` key and `"mitre_attack"` key | happy-path (empty vec, both keys absent) |
| Finding with `mitre_techniques: vec!["T9999".to_string()]` (unknown) | JSON finding contains `"mitre_techniques": ["T9999"]` and `"mitre_attack": [{"id": "T9999", "reference": "https://attack.mitre.org/techniques/T9999/"}]`; no `name`, `tactic_id`, `tactic_name` keys in the element | edge-case (unknown ID, partial object, id never lost) |
| Finding with `mitre_techniques: vec!["T1692.001".to_string(), "T0836".to_string()]` (multi-tag) | `"mitre_attack"` has 2 elements in order: T1692.001 fully resolved (IcsImpairProcessControl / TA0106), T0836 fully resolved (IcsImpairProcessControl / TA0106) | edge-case (multi-tag, order preserved) |
| Report with 3 findings: first has `mitre_techniques=["T1046"]`, second has `mitre_techniques=[]`, third has `mitre_techniques=["T0827"]` | First finding: `mitre_attack` present with T1046 object. Second finding: `mitre_attack` absent. Third finding: `mitre_attack` present with T0827 object (IcsImpact / TA0105). `mitre_techniques` key follows BC-2.09.006 rules independently. | happy-path (mixed batch, per-finding independence) |
| Finding with `mitre_techniques: vec!["T1046".to_string(), "T9999".to_string(), "T1046".to_string()]` (known + unknown + duplicate) | `"mitre_attack"` has 3 elements: index 0 fully resolved T1046, index 1 partial T9999 (id+reference only), index 2 fully resolved T1046 — no deduplication, order preserved | edge-case (EC-007: duplicates and unknown interleaved) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Known single technique produces fully-resolved 5-field object in `mitre_attack[0]` | unit: test_BC_2_11_035_known_technique_all_five_fields |
| — | Unknown technique ID produces partial object with `id` and `reference` only; `name`/`tactic_id`/`tactic_name` absent | unit: test_BC_2_11_035_unknown_technique_id_never_lost |
| — | Empty `mitre_techniques` produces no `mitre_attack` key in JSON output | unit: test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack |
| — | Multi-tag: `mitre_attack` array order matches `mitre_techniques` order exactly | unit: test_BC_2_11_035_multitag_order_preserved |
| — | Duplicate technique IDs in `mitre_techniques` produce duplicate (not deduplicated) elements in `mitre_attack` | unit: test_BC_2_11_035_duplicate_ids_not_deduplicated |
| — | Sub-technique dot separator preserved verbatim in `id` and `reference` URL | unit: test_BC_2_11_035_sub_technique_dot_preserved |
| — | ICS technique (T0xxx) resolves `tactic_id` to ICS-matrix TA-id (e.g. TA0105 / TA0106 / TA0107) | unit: test_BC_2_11_035_ics_tactic_id_resolved |
| — | `mitre_techniques` array is unchanged (additive non-breaking) when `mitre_attack` is also present | unit: test_BC_2_11_035_mitre_techniques_unchanged |
| — | `mitre_attack` absent from CSV output (CsvReporter unaffected) | unit: test_BC_2_11_035_csv_unaffected |
| — | `mitre_attack` absent from terminal output (TerminalReporter unaffected) | unit: test_BC_2_11_035_terminal_unaffected |

## Formal Verification Note

No new Verification Property (VP-NNN) is required. The enrichment logic is pure
`Option`-chaining over `technique_info()` (already Kani-verified as VP-007 sub-properties A
and B). The `mitre_attack` array serialization is structurally identical to the existing
`mitre_techniques` array serialization path — `serde` derive handles it deterministically
without loops over symbolic inputs. Test coverage (unit tests per the table above) is
sufficient; Kani would add no soundness value over an exhaustive closed-set catalog.

The `technique_tactic_id` extension IS in VP-007 scope: its `MitreTactic`-to-TA-ID mapping
must be added to the `vp007_catalog_drift_guard` test (a count/membership cross-check, not a
Kani proof), ensuring the mapping table does not drift from the `MitreTactic` enum variants.

## Error Taxonomy Note

No new error codes are required. Unknown technique IDs produce partial objects (EC-004) rather
than errors. This is a deliberate agent-safety decision: the `id` field is always emitted so
consumers can still act on unresolved IDs (e.g., forward them to external MITRE lookups). The
`skip_serializing_if` mechanism for optional resolved fields is infallible by construction
(same pattern as BC-2.09.006).

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — this BC extends the JSON finding shape with a human-readable enriched MITRE technique object, which is a direct output-quality contract of the Reporting capability enabling consumers to correlate findings with the ATT&CK knowledge base without secondary catalog lookups |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — `mitre_techniques` raw vec is never mutated; `mitre_attack` is a derived read-only projection; CSV/terminal reporters are unaffected) |
| Architecture Module | SS-11 (reporter/json.rs — new `MitreAttackEntry` struct with serde derive; new `technique_tactic_id` accessor in src/mitre.rs) |
| Stories | TBD (story-writer assigns in F3) |
| Issue | #64 (mitre_attack enriched JSON field) |
| ADR | TBD (no new ADR required; additive change; consistent with ADR-0003 serde delegation pattern and ADR-006 Decision 13 mitre_techniques schema) |

## Related BCs

- BC-2.09.006 — composes with (`mitre_techniques` Vec serialization pattern; `skip_serializing_if = Vec::is_empty` behavior; this BC adds a parallel field that mirrors the same omission logic)
- BC-2.11.001 — composes with (JSON envelope shape; `mitre_attack_version` envelope field is the catalog-version communication channel; this BC adds per-finding enrichment that is version-indexed by the envelope)
- BC-2.10.005 — depends on (MITRE technique catalog seeding; `technique_info` is the shared lookup function; this BC's resolution semantics depend on the catalog's `Some`/`None` guarantees)
- BC-2.10.006 — composes with (unknown IDs return `None` without panicking; this BC's EC-004 partial-object behavior relies on BC-2.10.006's `None`-safety guarantee)

## Architecture Anchors

- `src/reporter/json.rs` — new `MitreAttackEntry` struct (serde Serialize derive; fields: `id: String`, `name: Option<&'static str>`, `tactic_id: Option<&'static str>`, `tactic_name: Option<String>`, `reference: String`; `skip_serializing_if` on `name`/`tactic_id`/`tactic_name`)
- `src/reporter/json.rs` — `Finding` serialization: add `mitre_attack` field derived from `mitre_techniques` via a `#[serde(skip_serializing_if)]` computed field or a custom `Serialize` impl; alternatively a wrapper type
- `src/mitre.rs` — new `technique_tactic_id(id: &str) -> Option<&'static str>` public function: calls `technique_info(id)`, maps the `MitreTactic` to its canonical TA-ID string via a `match`; returns `None` for unresolved IDs

## Story Anchor

TBD (story-writer assigns in F3)

## VP Anchors

- — (VP assignment deferred; see Formal Verification Note and Verification Properties table above; test-writer authors unit tests per DF-AC-TEST-NAME-SYNC-001)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (catalog lookup is a static match; URL synthesis is string formatting) |
| **Global state access** | none |
| **Deterministic** | yes — catalog lookup is deterministic; URL synthesis is deterministic; serde output is deterministic for a given input |
| **Thread safety** | Send + Sync (no mutable state; all inputs are borrowed or owned value types) |
| **Overall classification** | pure |
