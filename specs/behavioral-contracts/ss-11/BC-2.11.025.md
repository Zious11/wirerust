---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-06-17T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.8.0
modified: ["v1.1 2026-06-17: F2 adversarial pass-1 — add singleton immutability invariant (F-259-03), severity-agnostic postcondition (F-259-04), input-order determinism assumption (F-259-06), raw-key vs escaped-display postcondition and test vector (F-259-09)", "v1.2 2026-06-17: F2 adversarial pass-2 — Vec-accumulator canonical (F-A01); strengthen primary flood test vector (F-A04); fix dispatch anchor 149-160→149-162 (F-A05)", "v1.3 2026-06-17: F2 adversarial pass-4 — F-F2-A01: Invariant 6 singleton claim converted from 'byte-identical to calling render_finding_flat directly' to observable-behavior form; F-F2-O01: anchor :203-226 → :203-227; F-F2-O02: flood vector timestamp updated to 'differing per-request timestamps (non-key field)' to reflect real empty-UA emission pattern", "v1.4 2026-06-17: F2 adversarial pass-9 — F-PA-02: soften flood-vector timestamp claim: 'DIFFERING per-request timestamps' → 'timestamps MAY differ across requests/flows (timestamp is a NON-KEY field; collapse is invariant to it regardless)' to avoid implying timestamps always differ"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.025: Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic

## Description

When `TerminalReporter.collapse_findings = true` and `show_mitre_grouping = false`, the
flat-mode FINDINGS section collapses the raw `findings` slice into display groups before
rendering. Two findings are members of the same group if and only if all four key fields
are equal: `category`, `verdict`, `confidence`, and `summary`. Any difference in any of
these four fields — even one character in `summary` — produces a distinct group. Fields
outside the key (`evidence`, `mitre_techniques`, `source_ip`, `timestamp`, `direction`)
are NOT part of the key; they vary per instance and are retained as evidence samples
(governed by BC-2.11.027). The collapsed output preserves first-occurrence order: each
group's position in the output is determined by the index of the first finding matching
that key in the input slice. Group order is stable and deterministic across all runs given
the same input slice.

This BC is scoped to flat mode only (`show_mitre_grouping = false`). Grouped (`--mitre`)
mode renders findings individually regardless of `collapse_findings`. Grouped-mode collapse
is deferred to a future cycle (see STORY-119).

## Preconditions

1. `TerminalReporter.collapse_findings = true`.
2. `TerminalReporter.show_mitre_grouping = false` (flat mode).
3. `findings` is a non-empty `&[Finding]` slice passed to `render()`.
4. The `Finding` struct fields used as the aggregation key (`category: ThreatCategory`,
   `verdict: Verdict`, `confidence: Confidence`, `summary: String`) are populated on every
   element (they are non-optional by type).

## Postconditions

1. The rendered FINDINGS section contains at most one display group per unique
   `(category, verdict, confidence, summary)` tuple.
2. Display groups appear in first-occurrence order: the position of each group in the output
   corresponds to the position of the first finding with that key in the input `findings` slice.
3. Findings whose `(category, verdict, confidence, summary)` tuple differs from all other
   findings form a singleton group (group count N=1) and are rendered individually (governed
   by BC-2.11.026 for count suffix rules).
4. Two findings that differ only in `evidence`, `mitre_techniques`, `source_ip`, `timestamp`,
   or `direction` — but share the same four-field key — are collapsed into one display group.
5. The total number of display groups equals the number of distinct
   `(category, verdict, confidence, summary)` tuples present in the input slice.
6. The collapse pass is applied strictly inside `TerminalReporter::render`; the `findings`
   slice passed to `render()` is never modified or pre-filtered upstream of the multi-reporter
   dispatch.
7. Collapse is SEVERITY-AGNOSTIC. It applies to every group sharing the four-tuple key
   regardless of `verdict`, `confidence`, or `category`. There is NO "low-value" filter; the
   issue-#259 title "Collapse repeated low-value findings" describes the motivating case
   (empty-UA flood), not a gating condition on verdict or confidence. Two `Likely/High`
   identical findings collapse into a group of N=2 with a ` (x2)` suffix just as two
   `Inconclusive/Low` findings would.
8. Because the key uses RAW-byte `summary` equality (byte-exact string comparison) while the
   terminal display applies `escape_for_terminal`, two findings whose escaped summaries are
   visually identical on the terminal but whose raw `summary` fields differ form two DISTINCT
   groups, each with its own count. This is intentional: forensic fidelity takes precedence
   over visual deduplication. Example: `summary = "x\x1b"` and `summary = "x"` have different
   raw bytes; they form two groups even though both render visually as `"x"` after escaping
   (the first displays as `"x\u{1b}"`).
9. Collapse determinism is CONDITIONAL on the input `findings` slice order being stable. The
   collapse pass itself introduces NO additional non-determinism: it MUST use a
   `Vec<(CollapseKey, Vec<&Finding>)>` insertion-ordered accumulator with linear-scan
   `PartialEq` matching (since `ThreatCategory`, `Verdict`, and `Confidence` derive `PartialEq`
   but NOT `Hash`, and `Cargo.toml` does not add `indexmap` in v0.8.0). An `IndexMap` is only
   viable if `Hash` is derived on all three key enums AND the `indexmap` crate is added — NOT
   done in v0.8.0; the Vec accumulator is canonical. Using a `HashMap` whose iteration order
   is non-deterministic is prohibited. Input slice-order stability is upstream's responsibility
   and is out of scope for this BC.

## Invariants

1. The aggregation key is the four-tuple `(category: ThreatCategory, verdict: Verdict,
   confidence: Confidence, summary: String)`. Key equality uses `PartialEq` on the respective
   types; `ThreatCategory`, `Verdict`, and `Confidence` are `#[derive(PartialEq)]` enums;
   `summary` equality is byte-exact string equality.
2. Fields NOT in the key: `evidence: Vec<String>`, `mitre_techniques: Vec<String>`,
   `source_ip: Option<IpAddr>`, `timestamp: Option<DateTime<Utc>>`,
   `direction: Option<Direction>`. Differences in these fields do NOT prevent collapsing.
3. Group order is first-occurrence: equivalent to a stable, order-preserving deduplication
   of the slice by key. The implementation must not sort, shuffle, or otherwise reorder the
   output relative to input slice order.
4. The collapse pass is a private implementation detail of `TerminalReporter`. `JsonReporter`
   and `CsvReporter` always receive and render the original unmodified `findings` slice
   (BC-2.11.029).
5. This invariant is scoped to flat mode. When `show_mitre_grouping = true`, the collapse
   pass is not applied regardless of the `collapse_findings` field value.
6. For a singleton group (N=1), the representative rendered is the ORIGINAL `&Finding`
   element from the input slice (same field values, full untruncated `evidence` vec). The
   collapse pass MUST NOT clone, reorder, or modify a singleton's fields. The N=1 output is
   byte-identical to the pre-v0.8.0 terminal output for the same finding — no ` (x1)` suffix,
   no evidence truncation, no reordering. This invariant is referenced by BC-2.11.026 (singleton
   has no suffix) and BC-2.11.029 (singleton renders identically to pre-v0.8.0). Any future
   refactor that handles the N=1 path differently MUST verify byte-identity against the
   pre-v0.8.0 output.
7. The collapse grouping structure MUST be a `Vec<(CollapseKey, Vec<&Finding>)>` accumulator
   with linear-scan `PartialEq` matching. This is the CANONICAL implementation structure for
   v0.8.0 because `ThreatCategory`, `Verdict`, and `Confidence` derive only `PartialEq` (not
   `Hash`), and the `indexmap` crate is not a dependency. (`IndexMap` would be viable only if
   `Hash` were derived on all three key enums AND `indexmap` added to `Cargo.toml` — neither
   is done in v0.8.0.) Using a `HashMap` (non-deterministic iteration order) is prohibited and
   would violate Postcondition 2 (first-occurrence order) and Postcondition 9 (determinism).
   This is an implementation constraint enforced at the spec level.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All N findings identical in all four key fields | One display group with count N |
| EC-002 | All findings have distinct keys | N display groups, all singleton (N=1 each), rendered in input order |
| EC-003 | Two findings share key but differ only in evidence | Collapsed into one group; evidence sampling per BC-2.11.027 |
| EC-004 | Two findings share key but differ only in source_ip | Collapsed into one group |
| EC-005 | Two findings share key but differ only in mitre_techniques | Collapsed into one group |
| EC-006 | Two findings differ in category only | Two distinct groups |
| EC-007 | Two findings differ in verdict only | Two distinct groups |
| EC-008 | Two findings differ in confidence only | Two distinct groups |
| EC-009 | Two findings differ in summary by one character | Two distinct groups |
| EC-010 | Input slice has a single finding | One singleton group; rendered as current (no count suffix per BC-2.11.026) |
| EC-011 | show_mitre_grouping=true, collapse_findings=true | Collapse pass NOT applied; findings rendered individually via grouped path (scoping boundary) |
| EC-012 | Summary contains attacker-controlled bytes (C0/ESC) | Key comparison operates on raw bytes; escape_for_terminal applied at render time per BC-2.11.010, not during key construction |
| EC-013 | Mixed: 3 findings with key A, 2 with key B, 1 with key C; input order A,B,A,C,B,A | Output groups in order: A (count 3), B (count 2), C (count 1) — first-occurrence position of A is index 0, B is index 1, C is index 3 |
| EC-014 | Two findings identical except verdict=Likely and confidence=High (high-severity pair), same summary | Collapsed into one group with ` (x2)` suffix — collapse is severity-agnostic; no "low-value" gating condition |
| EC-015 | summary="x\x1b" (raw ESC byte) vs summary="x" (no ESC) | Two distinct groups — raw-byte key comparison distinguishes them even though both render visually similar after escape_for_terminal; key operates on raw bytes before any display escaping |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 findings all `(Anomaly, Inconclusive, Low, "Empty User-Agent header")`, IDENTICAL 4-tuple key, each with a DISTINCT evidence URI (e.g., `["GET /a HTTP/1.1"]`, `["GET /b HTTP/1.1"]`, ..., `["GET /e HTTP/1.1"]`), timestamps MAY differ across requests/flows (timestamp is a NON-KEY field; collapse is invariant to it regardless) — mirroring `src/analyzer/http.rs:359-371` empty-UA emission pattern | FINDINGS section contains exactly 1 display group for that key; group count = 5; evidence sampled per BC-2.11.027 positional first-K-members (first min(5,3)=3 evidence lines rendered); timestamp variance does NOT prevent collapse because timestamp is not a key field | happy-path (flood collapse — canonical empty-UA case) |
| 1 finding `(Anomaly, Inconclusive, Low, "Empty UA")` + 1 finding `(Reconnaissance, Likely, High, "Port scan")` | 2 display groups in input order | happy-path (distinct keys) |
| 3 findings: key A at index 0, key B at index 1, key A at index 2 | Output: group A first, group B second (first-occurrence order) | happy-path (ordering) |
| 2 findings same key but evidence=["req1"] vs evidence=["req2"] | 1 collapsed group (evidence differs; key matches) | edge-case (EC-003) |
| 2 findings differ in verdict (Likely vs Unlikely), same other fields | 2 distinct groups | edge-case (EC-007) |
| 2 findings both `(Reconnaissance, Likely, High, "Port scan")` (identical key, high severity) | 1 collapsed group with ` (x2)` suffix — severity-agnostic collapse (EC-014) | edge-case (F-259-04) |
| 2 findings: summary="x\x1b" (ESC byte) vs summary="x" | 2 distinct groups (raw-byte key; escape_for_terminal applied at render time only) | edge-case (EC-015, F-259-09) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | N identical-key findings collapse to 1 display group | unit: test_BC_2_11_025_identical_findings_collapse_to_one_group |
| — | Key discriminator: category difference prevents collapse | unit: test_BC_2_11_025_key_discriminator_category |
| — | Key discriminator: evidence difference does NOT prevent collapse | unit: test_BC_2_11_025_key_discriminator_evidence_nondiscriminating |
| — | First-occurrence group order preserved | unit: test_BC_2_11_025_first_occurrence_order |
| — | show_mitre_grouping=true suppresses collapse | unit: test_BC_2_11_025_grouped_mode_bypasses_collapse |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- this BC defines the aggregation-key contract that determines how repeated terminal findings are collapsed for human readability, which is a core output formatting responsibility of the Reporting and Output capability |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- collapse is a display-layer transform; the Finding slice is never mutated) |
| Architecture Module | SS-11 (reporter/terminal.rs) |
| Stories | STORY-118 |
| Issue | #259 (Collapse repeated low-value findings) |
| ADR | ADR-0003 (display-layer aggregation subsection) |

## Related BCs

- BC-2.11.026 -- composes with (count display rules for collapsed groups and singletons)
- BC-2.11.027 -- composes with (evidence sampling within a collapsed group)
- BC-2.11.028 -- depends on (opt-out flag that disables this collapse pass)
- BC-2.11.029 -- depends on (raw-stream invariant: JSON/CSV reporters are unaffected)
- BC-2.11.019 -- composes with (FINDINGS section structure; collapse changes the content of the flat path only)

## Architecture Anchors

- `src/reporter/terminal.rs:149-162` -- FINDINGS dispatch block (flat vs grouped branch) where the collapse pass will be inserted (includes `out.push('\n')` at :161 and block close at :162)
- `src/reporter/terminal.rs:203-227` -- render_finding_prefix (escape applied; called by the non-collapse and grouped paths)
- `src/reporter/terminal.rs:232-238` -- render_finding_flat (flat rendering helper)
- `src/findings.rs:136-162` -- Finding struct: category, verdict, confidence, summary fields (key fields); evidence, mitre_techniques, source_ip, timestamp, direction (non-key fields)

## Story Anchor

STORY-118

## VP Anchors

- — (new VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — first-occurrence order is determined by input slice position; no HashMap or random ordering |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
