---
document_type: behavioral-contract
level: L3
version: "1.7"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/json.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave-21 wave-level consistency lens — SS-11 reporter VP proof-method family harmonization (DF-SIBLING-SWEEP-001; sibling of the 2026-05-30 VP-020 correction): VP-017 VP-table Proof Method cells corrected unit→integration to match VP-017 authoritative method — 2026-05-30"
  - "v1.4: ADV-IMPL-P07-LOW-001 correction — Architecture Anchor and Invariant line citations json.rs:59 corrected to json.rs:60 (verified: serde_json::to_string_pretty(&output).unwrap() is at line 60; line 59 is the closing `});` of the json! macro) — 2026-06-01"
  - "v1.5: ADD-ON 1 (research-backed, f2-multitag-schema.md §1.4) — add mitre_domain and mitre_attack_version to JSON report envelope; both fields are top-level envelope fields (not per-finding); CSV reporters carry no envelope fields. mitre_attack_version value flagged for F4 to pin. — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. F4 FLAG updated: T0855 replaced by T1692.001 in the list of ICS technique IDs to pin the catalog version against. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.7: Advisory pointer — BC-2.11.035 (F2 issue #64) defines the per-finding `mitre_attack` array that extends each `findings[*]` object with resolved technique objects. This BC (BC-2.11.001) governs the JSON envelope shape and `mitre_attack_version` field; BC-2.11.035 governs the per-finding enrichment. The two contracts compose: BC-2.11.001 PC-3 (`findings` is an array; one element per Finding) is the entry point; BC-2.11.035 specifies the additive `mitre_attack` field within each element. No normative change to this BC. — 2026-06-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.001: JsonReporter Renders JSON Object with summary/findings/analyzers/mitre_domain/mitre_attack_version Keys

## Description

`JsonReporter::render` produces a single JSON object with exactly five top-level keys:
`summary`, `findings`, `analyzers`, `mitre_domain`, and `mitre_attack_version`. The output
is valid, pretty-printed JSON produced by `serde_json::to_string_pretty`. The `unwrap()`
call is infallible by construction because `serde_json::Value` serialization cannot fail.

The two MITRE envelope fields appear once at the top level of the report — never per-finding
— following the ECS/OCSF recommendation to document the ATT&CK matrix domain and version
once per report rather than redundantly on every finding (research basis:
`f2-multitag-schema.md §1.4`; ICS technique IDs use the `T0xxx` namespace which unambiguously
identifies the ICS-ATT&CK matrix, but the envelope fields remove any consumer ambiguity and
capture the catalog version for historical interpretation). CSV reporters carry no envelope
fields; these are JSON-only.

## Preconditions

1. `JsonReporter::render` is called with a `Summary`, a `&[Finding]`, and a
   `&[AnalysisSummary]`.
2. The `Summary` struct implements `serde::Serialize`.
3. All `Finding` fields implement `serde::Serialize`.

## Postconditions

1. The returned `String` is valid JSON (parseable by any RFC 8259 compliant parser).
2. The top-level object contains exactly the keys `"summary"`, `"findings"`, `"analyzers"`,
   `"mitre_domain"`, and `"mitre_attack_version"`.
3. `"findings"` is a JSON array; one element per `Finding` in the input slice.
4. `"analyzers"` is a JSON array; one element per `AnalysisSummary` in the input slice.
5. `"summary"` contains `total_packets`, `total_bytes`, `skipped_packets`,
   `unique_hosts`, `protocols`, and `services` sub-keys.
6. Output is pretty-printed (indented with spaces, one key per line).
7. `"mitre_domain"` is the JSON string `"ics-attack"` (the MITRE ATT&CK matrix that covers
   all `T0xxx` ICS technique IDs emitted by wirerust analyzers).
8. `"mitre_attack_version"` is a JSON string encoding the ATT&CK for ICS catalog version
   targeted by this release (e.g., `"ics-attack-v15"`). The exact version string is a
   placeholder (`"ics-attack-v15"`) until F4 pins the authoritative value against the
   deployed catalog. See FLAG below.

> **FLAG for F4 (pin mitre_attack_version):** The exact ATT&CK for ICS version string must
> be pinned against the catalog version that defines T0888 (Remote System Information
> Discovery), T1692.001, T0836, T0835, T0831, T0814, T0806 as used by wirerust's Modbus
> analyzer. The current placeholder is `"ics-attack-v15"`. F4 implementers must verify the
> actual version at attack.mitre.org/resources/attack-data-and-tools/ and update the
> constant in `src/reporter/json.rs` before the v0.3.0 release tag.

## Invariants

1. The `unwrap()` at `json.rs:60` is infallible; `serde_json::to_string_pretty` cannot fail
   on a `serde_json::Value`.
2. No manual escaping is performed; ADR 0003 delegates escaping to serde_json's RFC 8259
   path (C0+DEL escaped as `\uNNNN`; C1 passed through as raw UTF-8).
3. Protocol keys are converted via `{k:?}` (Debug) format to produce string keys in the
   JSON map (e.g., `"Tcp"`, `"Udp"`).
4. `mitre_domain` is a **constant** string `"ics-attack"`. It is NOT derived from the
   findings array and is never absent or null. Every JSON output carries this field
   unconditionally.
5. `mitre_attack_version` is a **constant** string determined at compile time (or sourced
   from a build-time constant). It is NOT inferred from the findings data. The value must
   be pinned by F4 to the authoritative MITRE ATT&CK for ICS version that defines all
   ICS techniques emitted by wirerust.
6. Both `mitre_domain` and `mitre_attack_version` appear in the JSON output ONLY (not in
   CSV, not in terminal output). CSV has no envelope concept; terminal output renders
   per-finding and per-summary data only.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty findings slice | `"findings": []` |
| EC-002 | Empty analyzers slice | `"analyzers": []` |
| EC-003 | skipped_packets = 0 | `"skipped_packets": 0` is present |
| EC-004 | No hosts seen | `"unique_hosts": []` |
| EC-005 | Multiple protocol types | Protocol map has one key per distinct Protocol variant observed |
| EC-006 | mitre_domain field is always present | `"mitre_domain": "ics-attack"` appears in every JSON report regardless of whether any ICS findings were emitted |
| EC-007 | mitre_attack_version field is always present | `"mitre_attack_version": "ics-attack-v15"` (or pinned value) appears in every JSON report; value is a constant, not inferred from findings |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary with 1 packet, 0 findings, 0 analyzers | JSON with summary.total_packets=1, findings=[], analyzers=[], mitre_domain="ics-attack", mitre_attack_version present | happy-path |
| Summary with skipped_packets=3 | "skipped_packets": 3 appears in summary | happy-path |
| findings=[one Finding] | "findings" array has length 1; mitre_domain and mitre_attack_version always present | happy-path |
| Empty everything | All five top-level keys present: summary, findings=[], analyzers=[], mitre_domain, mitre_attack_version | edge-case |
| JSON report with only HTTP findings (no ICS techniques) | mitre_domain="ics-attack" and mitre_attack_version still present (envelope fields are unconditional) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-017 | Output is valid JSON | integration: test_json_reporter_produces_valid_json |
| VP-017 | Top-level keys are exactly summary, findings, analyzers | integration: assert key presence |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC defines the machine-readable JSON output shape that is the primary API surface of the reporting capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- JsonReporter does not escape, delegates to serde_json per ADR 0003) |
| Architecture Module | SS-11 (reporter/json.rs, C-19) |
| Stories | STORY-076 |
| Origin BC | BC-RPT-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.002 -- composes with (skipped_packets detail in summary)
- BC-2.11.003 -- composes with (RFC 8259 C0 escaping behavior)
- BC-2.11.004 -- composes with (Unicode preservation behavior)
- BC-2.09.006 -- depends on (Finding Option fields serialization)
- BC-2.11.035 -- composes with (per-finding `mitre_attack` array — additive enrichment within each `findings[*]` element; BC-2.11.001 governs the envelope, BC-2.11.035 governs the per-finding MITRE object array)

## Architecture Anchors

- `src/reporter/json.rs:23-60` -- JsonReporter::render implementation
- `src/reporter/json.rs:60` -- infallible unwrap on serde_json::to_string_pretty
- `src/reporter/json.rs` -- MITRE_DOMAIN constant (`"ics-attack"`) and MITRE_ATTACK_VERSION
  constant (placeholder `"ics-attack-v15"`; F4 must pin) to be added in v0.3.0

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/json.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: serde::Serialize derived on Summary and Finding
- **assertion**: test_json_reporter_produces_valid_json verifies parseable output
- **structural guarantee**: `serde_json::to_string_pretty` on a `serde_json::Value` cannot fail
  (Value implements Serialize and contains only JSON-representable types); the `unwrap()` at
  `json.rs:60` is therefore infallible by construction, not by documentation

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (returns owned String) |
| **Global state access** | none |
| **Deterministic** | yes (BTreeMap used for protocol/service maps; alphabetical key order) |
| **Thread safety** | Send + Sync (JsonReporter is a unit struct) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- pure string transformation, suitable for formal verification.
