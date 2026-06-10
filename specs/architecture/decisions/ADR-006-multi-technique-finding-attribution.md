---
document_type: adr
adr_id: ADR-006
status: proposed
date: 2026-06-09
modified:
  - date: 2026-06-10
    actor: architect
    reason: "MITRE ATT&CK-ICS v19 remap: T0855→T1692.001 (Unauthorized Message: Command Message) in all live spec body references (issue #222)."
subsystems_affected:
  - SS-09
  - SS-10
  - SS-11
  - SS-14
supersedes: null
superseded_by: null
---

# ADR-006: Multi-Technique Finding Attribution

> **One-per-file:** Each architectural decision lives in its own file.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.
> Frontmatter `subsystems_affected` lists SS-NN identifiers from ARCH-INDEX Subsystem Registry.

## Context

wirerust's `Finding` struct (src/findings.rs) has always carried a single optional technique
attribution: `mitre_technique: Option<String>`. This was appropriate for the initial Enterprise
ATT&CK detection set, where each finding was naturally associated with one tactic and one
technique (e.g., T1083 path traversal, T1027 SNI obfuscation).

Feature #7 (Modbus TCP analyzer) introduces detections where a single observable event maps
to multiple co-applicable MITRE ATT&CK for ICS techniques simultaneously:

- A Modbus register write (FC 0x06) is simultaneously T1692.001 (Unauthorized Message: Command Message —
  any unsanctioned write) AND T0836 (Modify Parameter — specifically a holding-register write
  modifying a process setpoint or configuration value).
- A Modbus coil write (FC 0x05) is simultaneously T1692.001 AND T0835 (Manipulate I/O Image —
  specifically a digital output coil).
- A write burst exceeding the configured threshold is simultaneously T0806 (Brute Force I/O)
  AND T1692.001 (Unauthorized Message: Command Message at the burst level).

The original F2 spec (v1.0 Decision 7) handled this by a "cap to most-specific" rule: for
register writes, emit T0836 and suppress T0835; always emit T1692.001 (formerly T0855 prior to
ATT&CK-ICS v19) as a second separate finding. This approach controlled finding volume by discarding technique co-attribution — a
mechanistic tradeoff that produced 2–5 findings per PDU while losing analyst-relevant signal.

A research review of detection-engineering standards confirms the industry-standard model is
different: **one finding per observable event, carrying ALL applicable technique tags.** This
is the Sigma rule standard (`tags` carries multiple `attack.tXXXX` entries per rule), the
Elastic Common Schema design (`threat.technique` is multi-valued by design), and MITRE's own
stated intent (techniques are overlapping labels on observed behavior). Alert volume is
controlled by event-level aggregation (one finding per burst event, per Suricata `limit` /
Elastic alert suppression semantics), not by discarding technique tags.

The current single-string field `mitre_technique: Option<String>` structurally cannot express
co-attribution. A `Vec<String>` is the minimal change that aligns with the industry standard
while preserving all existing behavior (a Vec of length 0 replaces `None`; length 1 is the
prior single-technique case; length > 1 is the new co-attribution case).

## Decision

**Replace `mitre_technique: Option<String>` with `mitre_techniques: Vec<String>` on the
`Finding` struct in `src/findings.rs`.** This is a breaking type change targeting v0.3.0.

The decision encompasses four coordinated sub-decisions:

### Sub-decision 1: Type change and JSON schema

`Finding.mitre_technique: Option<String>` becomes `Finding.mitre_techniques: Vec<String>`
annotated `#[serde(skip_serializing_if = "Vec::is_empty")]`. The JSON key changes from
`"mitre_technique"` (string) to `"mitre_techniques"` (array). An empty vec produces no key
in the JSON output (same behavior as the prior `None` case). A single-element vec produces
`"mitre_techniques": ["T1027"]`. A multi-element vec produces `"mitre_techniques": ["T1692.001", "T0836"]`.

### Sub-decision 2: CSV column rename and join format

CSV column 6 is renamed from `mitre_technique` to `mitre_techniques`. Multiple techniques
are serialized as a semicolon-joined string with no spaces: `"T1692.001;T0836"`. An empty vec
produces an empty string (unchanged from the prior `None` case). Column count remains 9.

**CSV writer delimiter requirement:** The CSV writer MUST be explicitly configured with a
**comma** (`,`) as the field delimiter — not locale-dependent, not a platform default. The
implementation must not rely on any global or locale setting that might produce a different
delimiter. The semicolon join character is an intra-cell separator within column 6; it MUST
NOT conflict with the row-level field delimiter. Using a comma field delimiter ensures that
a cell value of `"T1692.001;T0836"` is emitted as a single quoted or unquoted column without
the semicolons being misinterpreted as field boundaries.

### Sub-decision 3: One-finding-per-event with co-attribution (volume control via aggregation)

For Modbus write-class events:
- One finding per write-class PDU carrying ALL applicable technique tags (e.g.,
  `["T1692.001", "T0836"]` for register writes, `["T1692.001", "T0835"]` for coil writes).
- One aggregated burst finding per burst event (`["T0806", "T1692.001"]`) — fired at most once
  per 1-second burst-window overflow and at most once per >=2-second sustained-window overflow.
  This is the volume-control mechanism (Elastic/Suricata/Splunk "one representative alert per
  time period" pattern) that replaces the prior tag-suppression approach.
- Total findings per write PDU: 1 (mid-burst) to 4 (tip-of-burst + T0831 co-occurring).
  This is fewer than v1.0 Decision 7's 2–5 per PDU, while preserving full technique signal.

**Canonical `mitre_techniques` construction order (determinism mandate):** Every emission
site MUST construct `mitre_techniques` in the following fixed precedence order, regardless
of runtime evaluation order:

1. **T0806** — Brute Force I/O (burst/rate-level technique; only present on burst findings)
2. **T1692.001** — Unauthorized Message: Command Message (always present on write-class and burst findings)
3. **T0836** — Modify Parameter (register writes: FC {0x06, 0x10, 0x16})
4. **T0835** — Manipulate I/O Image (coil writes: FC {0x05, 0x0F})
5. **T0831** — Manipulation of Control (inline co-tag on the triggering holding-register write)
6. **T0814** — Denial of Service (diagnostic sub-function findings)
7. **T0888** — Remote System Information Discovery (recon-FC findings)

**Canonical per-event vectors (authoritative):**

| Event | `mitre_techniques` |
|-------|--------------------|
| Register write (normal) | `["T1692.001", "T0836"]` |
| Register write (T0831 co-tag fires) | `["T1692.001", "T0836", "T0831"]` |
| Coil write | `["T1692.001", "T0835"]` |
| Other write FC (0x15, 0x17) | `["T1692.001"]` |
| Burst or sustained rate exceeded | `["T0806", "T1692.001"]` |
| Diagnostic sub-func 0x0001 or 0x0004 | `["T0814"]` |
| Recon FC (0x11, 0x2B/0x0E) | `["T0888"]` |

The ordering rule ensures that `mitre_techniques[0]` — used by the terminal reporter for
MITRE tactic-bucket grouping (BC-2.11.013) — is deterministic and consistent regardless of
which emission site produced the finding. Emission sites MUST use `vec![...]` literals in
the canonical order above, NOT dynamic insertion or set-to-vec conversion. Any deviation
from this order is a bug.

### Sub-decision 4: Migration of all existing emission sites

All existing emission sites in `src/analyzer/http.rs`, `src/analyzer/tls.rs`, and
`src/reassembly/` migrate from `mitre_technique: Some("TXXXX")` / `mitre_technique: None`
to `mitre_techniques: vec!["TXXXX"]` / `mitre_techniques: vec![]` respectively. No
existing detection behavior changes; only the field name and type changes.

## Rationale

### Why Vec<String> rather than a newtype or enum?

A `Vec<String>` is the minimal structural change consistent with how MITRE technique IDs
are already handled in this codebase: as plain strings verified at the boundary by the
`technique_info` lookup (VP-007). Introducing a `MitreId` newtype or an enum would require
either a large enum (intractable for hundreds of ATT&CK IDs) or a validated-newtype with
parse/display cost. The existing pattern — store the string, verify at boundary — is
already validated by `vp007_catalog_drift_guard`. A Vec of strings is the least-disruptive
generalisation of the existing pattern.

### Why not keep multiple separate single-technique findings?

The industry consensus surveyed in `.factory/research/modbus-f2-design-decisions.md`
(Decision 3) is unambiguous: one observable → one alert → N technique tags is the
Sigma/Elastic-aligned standard. Separate single-technique findings from the same event
produce correlated alerts that an analyst must manually deduplicate to understand the
scope of the event. Multi-tag single findings preserve provenance (both T1692.001 and T0836
are visible on the same event) without requiring deduplication logic downstream.

Additionally, separate findings amplify finding count proportional to the number of
co-applicable techniques. In a Modbus write flood (e.g., 500 writes in 5 seconds), the v1.0
model would produce 1000+ findings (2 per write), of which the T1692.001 entries are redundant
with the T0836 entries. The multi-tag model produces 500 findings (one per write), reducing
MAX_FINDINGS pressure while preserving full attribution.

### Why not cap to most-specific technique?

Capping to the most specific technique (T0836 > T0835 > T1692.001) discards the information
that T1692.001 applies. An analyst investigating an ICS incident genuinely needs to see both
"this is an unauthorized command message" (T1692.001, ICS Impair Process Control tactic) AND
"this specifically modifies a parameter" (T0836). Suppressing T1692.001 removes the broader
context; suppressing T0836 removes the specificity. Neither form of suppression is
information-preserving. The multi-tag model retains both.

### Why is this a breaking change (v0.3.0)?

Because:
1. The JSON key name changes: `"mitre_technique"` → `"mitre_techniques"`.
2. The JSON value type changes: scalar string → array.
3. The CSV column 6 header changes.
4. The Rust type changes: `Option<String>` → `Vec<String>`.

Any downstream consumer that parses wirerust JSON output, processes its CSV output, or
imports the `Finding` struct directly will observe a breaking change and must update.
This meets the SemVer definition of a breaking change. Per the project's gitflow versioning
convention (see CLAUDE.md), this warrants a v0.3.0 release bump. The CHANGELOG MUST document
this as a breaking schema change.

### Why bundle this into Feature #7 rather than a separate feature?

The multi-tag model is motivated directly by the Modbus co-attribution requirement: without
it, Feature #7 must either suppress valid technique tags (wrong) or emit 2–5 separate
single-technique findings per write PDU (amplification risk). Feature #7 is the first and
forcing case for multi-tag attribution. Bundling the Finding type change with Feature #7
avoids an intermediate state where the Modbus analyzer must work around a structural
limitation of the Finding type.

## Consequences

### Positive

- `Finding` can express co-occurring technique attribution without information loss.
- Finding count per Modbus write PDU decreases (1 multi-tag finding vs 2–5 single-tag
  findings in the v1.0 model), reducing MAX_FINDINGS pressure.
- JSON and CSV outputs are aligned with the Sigma/Elastic multi-technique convention,
  making them easier to ingest into standard SIEM platforms.
- VP-007's drift guard (`vp007_catalog_drift_guard`) continues to enforce that every ID in
  `mitre_techniques` resolves in `technique_info`; the guard logic changes only in that the
  grep pattern for emitted IDs shifts from `mitre_technique: Some` to `mitre_techniques: vec!`.
- `Vec<String>` is structurally backward-compatible with the single-technique case: a length-1
  vec is semantically equivalent to the prior `Option::Some` case; a length-0 vec is
  semantically equivalent to `Option::None`.

### Negative / Trade-offs

- **Breaking change** to the `Finding` public type, JSON schema, and CSV schema. All
  downstream consumers must update. No silent migration path exists.
- Every emission site in `src/analyzer/http.rs`, `src/analyzer/tls.rs`, `src/reassembly/`,
  and (new) `src/analyzer/modbus.rs` must be updated in the Feature #7 implementation commit.
  This is a large but mechanical change (~25–30 emission sites total).
- All test code constructing `Finding` values must be updated.
- VP harnesses that construct `Finding` values (VP-016, VP-020, VP-021) must be updated.
- The terminal reporter's MITRE grouping logic must be updated to work with multi-technique
  findings (use `mitre_techniques[0]` as the primary tactic-bucket key; secondary techniques
  appear as additional tags in the finding display). **Bucket-order determinism is guaranteed
  by the canonical construction order mandate in Sub-decision 3**: `mitre_techniques[0]` is
  always the same technique for the same event type because emission sites use fixed `vec![...]`
  literals in canonical order, not dynamic insertion. BC-2.11.013 mirroring obligation: the
  product-owner MUST specify the multi-techniques tactic-grouping rule as "`mitre_techniques[0]`
  is the bucket key; empty vec → Uncategorized."
- The `technique_info` display for multi-tag findings requires a renderer decision: show all
  IDs (`"MITRE: T1692.001, T0836"`), show first only, or show all with names. Decision: show all
  IDs separated by `, ` in the default inline display; the MITRE grouping expanded view
  (BC-2.11.016) shows name+em-dash for each. This is a cosmetic change to terminal output.
- The CSV writer MUST be explicitly configured with a comma field delimiter (see Sub-decision 2);
  relying on a locale-default or implicit delimiter risks semicolons in multi-tag cells being
  misread as field separators in non-RFC-4180-compliant consumers. (Example cell value:
  `"T1692.001;T0836"` — the semicolon is an intra-cell separator, not a field delimiter.)

### Impact Boundary vs F1

F1 (Feature-100, timestamp) documented `Finding` as **DEPENDENT/unchanged** — the timestamp
field was added but the core type structure was stable. ADR-006 changes that classification:
`Finding` is now **MODIFIED** in Feature #7. The F2 spec delta boundary declaration
(`architecture-delta.md §1`) MUST be updated to reflect that `Finding` transitions from
DEPENDENT to MODIFIED in this feature cycle.

## Alternatives Considered

- **`mitre_technique: Option<Vec<String>>`:** Using an `Option<Vec<String>>` wrapping would
  distinguish "explicitly no techniques" from "not applicable." Rejected as over-engineering:
  the `Vec::is_empty()` predicate already provides this distinction without an extra layer of
  Option wrapping. An empty Vec is unambiguously "no technique attributed."

- **`mitre_technique_primary: Option<String>` + `mitre_technique_secondary: Vec<String>`:**
  A primary/secondary split to preserve the common single-technique rendering path. Rejected
  because it re-introduces the priority-ranking logic that this ADR explicitly removes, and
  because it makes the JSON schema asymmetric in a confusing way.

- **Keep single-technique field; add a separate `co_techniques: Vec<String>` field:**
  Preserves backward compatibility for the common case. Rejected because it splits attribution
  across two fields with unclear semantics (is the primary the "most specific" or the
  "authoritative"?), and because downstream consumers would need to merge the two fields to
  get the full technique set.

- **Emit separate single-technique findings per co-occurring technique (v1.0 Decision 7):**
  Preserve the `Option<String>` field and cap to most-specific. Rejected per the Rationale
  above: discards analyst-relevant signal and amplifies finding count.

- **Defer multi-technique support to v0.4.0; use cap-to-most-specific for v0.3.0:**
  Acceptable if the breaking change is too disruptive for this feature cycle. Not recommended:
  Feature #7 is the forcing case. Deferral means implementing cap-to-most-specific now and
  then migrating all emission sites AGAIN in v0.4.0 — double the churn for no user benefit.

## Source / Origin

- **Sigma multi-technique tags:** SigmaHQ specification; `tags` field carries multiple
  `attack.tXXXX` entries per detection rule by design.
- **Elastic Common Schema:** `threat.technique` field is multi-valued by design for
  ATT&CK classification across events spanning multiple techniques.
- **Elastic Security alert suppression:** "one representative alert per time period" pattern
  for volume control without attribution loss.
- **Suricata `threshold` / `limit` / `both` / `backoff` modes:** Volume-control mechanism
  orthogonal to attribution (alert count ≠ technique count).
- **Splunk alert suppression groups / event correlation:** Same pattern.
- **Research:** `.factory/research/modbus-f2-design-decisions.md` Decision 3 (verified,
  cross-sourced across Sigma/Elastic/Suricata/Snort/Splunk, all pointing to one-alert/N-tags
  + aggregation as the industry norm).
- **F2 directives (v2):** `.factory/phase-f2-spec-evolution/f2-fix-directives.md` Decision 13,
  which contains the full product-owner and formal-verifier obligation list.

## Status as of 2026-06-09

Proposed. The `Finding` type change, reporter updates, and analyzer emission-site updates are
part of the Feature #7 F3 implementation stories. All VP harness updates (VP-016, VP-020,
VP-021) are gated to the same implementation story that touches `src/findings.rs`. The
`vp007_catalog_drift_guard` test will remain green as long as the atomic update of
`SEEDED_TECHNIQUE_IDS`, `EMITTED_IDS`, and `technique_info` arms is performed in the same
commit as the `mitre_techniques: vec![...]` emission sites are introduced.
