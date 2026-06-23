---
document_type: f1-delta-analysis
cycle: feature-mitre-json-names
issue: "#64"
feature: "Add JSON DTO with computed mitre_tactic / mitre_name when SIEM consumer needs them"
produced_by: architect
date: 2026-06-23
status: complete
---

# F1 Delta Analysis — Issue #64: JSON DTO with mitre_tactic / mitre_name

## Trigger Conditions

This analysis applies the Feature Mode Phase F1 methodology to GitHub issue #64.
The feature adds two derived fields (`mitre_tactic`, `mitre_name`) to the JSON
reporter output, computing them on-the-fly from the existing `mitre_techniques`
array via lookup functions that already exist in `src/mitre.rs`.

Trigger condition from the issue: the first SIEM-ingestion or downstream-tooling
consumer of wirerust JSON output asks for `mitre_tactic` and/or `mitre_name` fields.
That trigger is now considered met by the issue author.

---

## 1. Impact Boundary

### Files Required to Change

| File | Change Type | Scope |
|------|-------------|-------|
| `src/reporter/json.rs` | MODIFY | Render `Vec<FindingJsonDto>` instead of `findings` directly; introduce/import DTO |
| `src/reporter/json_dto.rs` | CREATE (new) | `FindingJsonDto<'a>` struct with lifetime, `From<&'a Finding>` impl |
| `tests/reporter_json_tests.rs` | MODIFY | Add AC tests for new fields under new BC-2.11.035 (see §2) |

### Files Explicitly NOT Changing

| File | Rationale |
|------|-----------|
| `src/findings.rs` | `Finding` struct is untouched — DTO wraps it via `#[serde(flatten)]` |
| `src/mitre.rs` | `technique_name` (line 193) and `technique_tactic` (line 199) already exist; no changes needed |
| `src/cli.rs` | No CLI flag involved — fields emit unconditionally when `mitre_technique` is set and resolves; no flag |
| `src/reporter/terminal.rs` | Terminal reporter is out of scope; new fields are JSON-only |
| `src/reporter/csv/` | CSV reporter is out of scope (per BC-2.11.028 / BC-2.11.029: collapse and JSON-specific behavior does not bleed into CSV) |
| `src/main.rs` | No wiring change needed; `JsonReporter.render` called the same way |

### Architecture Decision: DTO vs Inline Fields

**Recommendation: separate `src/reporter/json_dto.rs` with `FindingJsonDto<'a>`.**

Rationale:

1. `Finding` derives `serde::Serialize` directly and is consumed by multiple reporters.
   Adding `#[serde(skip_serializing_if)]` fields for tactic/name on `Finding` itself
   would pollute a shared type with JSON-reporter-specific derived semantics.
2. The DTO-with-`From<&Finding>` pattern is the validated approach from the issue's
   brainstorming research (Perplexity-validated, issue body §"Suggested approach").
   It keeps `Finding` pure-data and the JSON reporter's computed view in its own module.
3. A lifetime `'a` on the DTO (`inner: &'a Finding`) avoids cloning; the DTO borrows
   `Finding` in-place for the duration of serialization, which is zero-cost.
4. `json.rs` already owns the render loop; the change is a one-liner swap from
   `findings` slice to `Vec<FindingJsonDto>` built via `.iter().map(Into::into).collect()`.

The issue's suggested skeleton is directly implementable:

```rust
// src/reporter/json_dto.rs
#[derive(Serialize)]
pub(crate) struct FindingJsonDto<'a> {
    #[serde(flatten)]
    inner: &'a Finding,
    #[serde(skip_serializing_if = "Option::is_none")]
    mitre_tactic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mitre_name: Option<&'static str>,
}

impl<'a> From<&'a Finding> for FindingJsonDto<'a> {
    fn from(f: &'a Finding) -> Self {
        // Finding.mitre_techniques is Vec<String>; for tactic/name we use
        // the first technique (primary attribution), matching existing terminal
        // rendering convention (BC-2.11.016/034 use members[0] as representative).
        let id = f.mitre_techniques.first().map(|s| s.as_str());
        Self {
            inner: f,
            mitre_tactic: id
                .and_then(crate::mitre::technique_tactic)
                .map(|t| t.to_string()),
            mitre_name: id.and_then(crate::mitre::technique_name),
        }
    }
}
```

**Multi-technique note:** When `mitre_techniques` has more than one element (co-attributed
findings, e.g. `["T1692.001","T0836"]`), only the first element is used for
`mitre_tactic`/`mitre_name`. This matches the terminal reporter convention from
BC-2.11.016/034. F2 spec work can extend this to an array form if SIEM consumers
require all tactic/name pairs. The single-element form is simpler, non-breaking,
and consistent with existing terminal behavior. This design choice MUST be documented
in the BC postconditions and the DTO module doc.

---

## 2. Spec Delta

### Subsystem

SS-11 (Reporting and Output, CAP-11). Current SS-11 BC count: **34** (BC-2.11.001
through BC-2.11.034, all `[WRITTEN]`). Next free ID: **BC-2.11.035**.

### New BC Required

**BC-2.11.035** — "JSON DTO Computes `mitre_tactic` and `mitre_name` from
`mitre_techniques[0]` Lookup; Fields Absent When Unresolved"

This BC is required because the per-finding field contract is observable behavior
with explicit postconditions:

- When `mitre_techniques` is non-empty and `mitre_techniques[0]` resolves in
  `technique_info`, both `mitre_tactic` (String) and `mitre_name` (string) are
  present in the finding's JSON object.
- When `mitre_techniques` is empty, both fields are absent (via
  `skip_serializing_if = "Option::is_none"`).
- When `mitre_techniques[0]` does not resolve (unknown ID), both fields are absent.
- `mitre_technique` (the legacy `Finding` field) is unaffected and retains its
  current serialization behavior per BC-2.09.006.

### Existing BCs Affected

| BC | Change |
|----|--------|
| BC-2.11.001 | Postcondition 2 currently states "exactly five top-level keys" and test `test_BC_2_11_001_top_level_keys` asserts an exact keyset. The new fields appear per-finding (not at the envelope level), so the envelope-level BC-2.11.001 is **not affected**. However, the BC description and the test's "findings array element" coverage implicitly describe `Finding`'s serialized shape. A version bump with a clarifying note pointing to BC-2.11.035 for per-finding computed fields is advisable but not strictly required. |
| BC-2.11.029 | Explicit scope note: "JSON/CSV Reporters Receive Unmodified `findings` Slice." After this change, `JsonReporter` wraps each `&Finding` in `FindingJsonDto` at serialization time; the underlying `findings` slice is logically unmodified. BC-2.11.029 covers the concern that collapse behavior stays display-layer only — this change is similarly display-layer (render-time computation). A clarifying note that the DTO wrapping is also render-time may be warranted but is advisory, not blocking. |

### Error-Taxonomy Impact

None. The DTO computes fields from the in-memory lookup (`technique_info`) which
returns `Option`. No new error paths, no new error codes, no I/O.

### NFR Impact

None. `technique_info` is a static `match` expression — O(1), infallible, zero
allocation for `mitre_name` (returns `&'static str`), one `String::from` for
`mitre_tactic`. Negligible serialization cost.

### Verification Property Assessment

A formal VP is **not warranted** for BC-2.11.035. Rationale:

1. The DTO logic is pure and simple: `Option::and_then` chained over `technique_info`,
   which already has VP-007 (MITRE catalog completeness and format invariants).
2. The property is fully expressible as a unit test suite in `reporter_json_tests.rs`
   without formal proof — all lookup behavior is already covered by VP-007 Kani proofs
   on `mitre.rs`. There is no new arithmetic, state machine, or security boundary.
3. Classification per the VP decision rubric: **"Test sufficient"** — derived-field
   computation on top of an already-proven lookup function.

Current VP total: 31 (VP-001 through VP-031, all verified, locked). No VP addition
in this feature.

---

## 3. ATT&CK Version

**Current mapping corresponds to MITRE ATT&CK for ICS v19.1.**

Evidence (source of truth, three independent anchors):

1. `src/reporter/json.rs:27`: `const MITRE_ATTACK_VERSION: &str = "ics-attack-19.1";`
   with comment at lines 23-26:
   > "F4: RESOLVED — pinned to ATT&CK for ICS v19.1 (released 2026-04-28).
   > Canonical STIX bundle: ics-attack-19.1.json."

2. `.factory/research/attack-ics-version-pin.md`: documents the F4-PIN resolution.
   Decision: pin to `ics-attack-19.1`. All emitted ICS technique IDs confirmed valid
   and active in v19.1 (after the T0855→T1692.001 v19 remap, issue #222).

3. `src/mitre.rs:316`: changelog comment:
   > "ICS v19 remap (issue #222): T0855→T1692.001, T0856→T1692.002."

**How the feature must handle versioning:**

The `mitre_name` and `mitre_tactic` values returned by `technique_info` are
hardcoded string literals that were authored against v19.1. They do not need a
version parameter — they are the v19.1 values by construction. The
`mitre_attack_version: "ics-attack-19.1"` envelope field already signals this to
consumers. No additional version recording is required for this feature.

**Important constraint:** If a future feature seeds new technique IDs from a later
ATT&CK version, both `technique_info` and the `MITRE_ATTACK_VERSION` constant
must be updated atomically. VP-007 (catalog drift guard, `vp007_catalog_drift_guard`
test) enforces completeness; the version constant is a human update obligation that
must be captured in the implementing story's acceptance criteria.

---

## 4. Story Scope

**One story: STORY-129.**

The change is self-contained:
- New file `src/reporter/json_dto.rs` (the DTO)
- Modification to `src/reporter/json.rs` (swap `findings` to `Vec<FindingJsonDto>`)
- New tests in `tests/reporter_json_tests.rs` (BC-2.11.035 ACs)
- New BC file `.factory/specs/behavioral-contracts/ss-11/BC-2.11.035.md`
- BC-INDEX update (ss-11 section, new row for BC-2.11.035, total 34→35)

No second story is warranted. The dependency chain is trivial: BC spec → DTO
implementation → tests. All affected files are in the reporter module.

**STORY-INDEX slot:** Wave 57 (next after Wave 56, the last wave assigned to
STORY-128). The story has no predecessor in the existing DAG; it depends only on
`src/mitre.rs` and `src/findings.rs` both of which are stable.

**Points estimate:** 5 points (small: one new file, one edit, new test suite, one BC).

---

## 5. Regression Risk

### Existing Tests Covering JSON Output

| Test File | BCs Covered | Risk from This Change |
|-----------|-------------|----------------------|
| `tests/reporter_json_tests.rs` | BC-2.11.001–005 (top-level keys, pretty-print, C0/C1 escaping) | `test_BC_2_11_001_top_level_keys` asserts **exactly** `["analyzers","findings","mitre_attack_version","mitre_domain","summary"]` as top-level keys. The new fields appear **inside** `findings[*]` objects, not at the top level — this test is **not affected**. |
| `tests/timestamp_threading_tests.rs:674` | `test_segment_limit_summary_timestamp_is_none_and_absent_from_json` | Tests `timestamp` key absence in JSON finding when None. The DTO `#[serde(flatten)]` preserves all `Finding` skip-if rules. **Not affected.** |
| `tests/timestamp_threading_tests.rs:786` | `test_finding_timestamp_json_serialization` | Tests that `timestamp` is present when Some. DTO flatten preserves this. **Not affected.** |
| `tests/bc_2_09_100_multitag_tests.rs` | BC-2.09.006 multi-tag schema | Tests `mitre_techniques` array shape. DTO flatten passes through all `Finding` fields including `mitre_techniques`. **Not affected.** |

### Key Invariants That Must Keep Passing

1. `test_BC_2_11_001_top_level_keys` — envelope key set is unchanged (new fields are
   per-finding, not per-envelope).
2. All `mitre_techniques` field tests — the DTO `#[serde(flatten)]` on `inner: &'a Finding`
   passes `Finding`'s existing serialization through unmodified. `mitre_techniques` retains
   its `skip_serializing_if = "Vec::is_empty"` behavior.
3. All C0/C1 escape tests — `summary` and `evidence` pass through the `Finding` flatten;
   serde_json's RFC 8259 path is unchanged. No new escaping is introduced.
4. `vp007_catalog_drift_guard` — this is a `#[cfg(test)]` test in `src/mitre.rs` that
   sweeps the entire ID space. The DTO calls the same `technique_info` function; no catalog
   changes occur. **Not affected.**

### Snapshot / Exact-Match Test Risk

**LOW.** There are no golden-file snapshot tests for JSON output in the test suite.
All JSON tests parse and assert specific keys/values — they do not compare full output
strings. The new `mitre_tactic`/`mitre_name` keys on finding objects will appear in
rendered output, but no test performs a full-string equality assertion on a multi-field
finding JSON blob that would fail on unexpected new keys.

**One potential trap to watch:** `test_BC_2_11_001_top_level_keys` asserts that the
top-level object has **exactly** the five known keys. If any test similarly asserts the
exact keyset of a finding object, it would fail. A search of `reporter_json_tests.rs`
shows no such per-finding keyset exhaustiveness assertion. New story tests for BC-2.11.035
must not introduce an accidental over-specification in existing vectors.

---

## 6. Schema-Compat Verdict

**ADDITIVE / NON-BREAKING — CONFIRMED.**

Analysis:

1. **New optional fields only.** `mitre_tactic` and `mitre_name` are introduced with
   `skip_serializing_if = "Option::is_none"`. They are absent for findings with no
   technique, and present only when the technique ID resolves. No existing field is
   renamed, removed, or type-changed.

2. **`mitre_techniques` is unchanged.** The existing `Vec<String>` field retains
   its current serialization behavior (absent when empty, JSON array when non-empty).
   Consumers already doing their own ID→name lookup see no regression.

3. **Top-level envelope unchanged.** The `summary`, `findings`, `analyzers`,
   `mitre_domain`, `mitre_attack_version` structure is preserved. `"findings"` remains
   a JSON array; each element gains optional fields.

4. **"No consumers today" mitigates compatibility risk.** The issue explicitly states
   "there is no SIEM-ingestion consumer of wirerust's JSON output" today. Any future
   consumer will be designed against the new schema. Strict-mode JSON parsers that
   reject unknown keys would be an issue for the consumer, not wirerust; this is a
   standard additive-schema concern and the documented mitigation is the issue's own
   "Existing JSON consumers (none today) are unbroken" statement.

5. **CSV reporter unaffected.** BC-2.11.028/029 confirm JSON-specific behavior does not
   bleed to CSV. The DTO is used only in `JsonReporter::render`.

**Conclusion:** Proceeding with this feature does not break any existing consumer.
The schema change is purely additive. SemVer classification: the change would be
captured in a minor version increment (or patch, given no consumer exists to break).

---

## Summary

| Deliverable | Answer |
|-------------|--------|
| Files to change | `src/reporter/json.rs` (modify), `src/reporter/json_dto.rs` (create), `tests/reporter_json_tests.rs` (new ACs) |
| DTO vs inline | DTO (`FindingJsonDto<'a>` in `json_dto.rs`) — recommended per issue research validation |
| New BCs | BC-2.11.035 (new, SS-11, next free after BC-2.11.034) |
| Existing BCs affected | BC-2.11.001 (advisory version bump with pointer to BC-2.11.035 for per-finding shape); BC-2.11.029 (optional clarification note) |
| New VP | None — "test sufficient" classification |
| ATT&CK version | `ics-attack-19.1` (pinned in `src/reporter/json.rs:27`; validated in `.factory/research/attack-ics-version-pin.md`). Names/tactics in `technique_info` are v19.1 values by construction. |
| Story count | 1 (STORY-129, Wave 57, ~5 points) |
| Regression risk | LOW — no exact-match JSON snapshot tests; envelope key tests unaffected; flatten preserves all existing Finding fields |
| Schema-compat | ADDITIVE / NON-BREAKING confirmed |
