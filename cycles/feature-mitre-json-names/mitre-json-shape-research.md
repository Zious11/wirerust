---
document_type: research-report
cycle: feature-mitre-json-names
issue: "#64"
topic: "Best JSON output design for MITRE ATT&CK technique->tactic+name data (multi-audience: LLM agents, humans, SIEMs)"
produced_by: vsdd-factory:research-agent
date: 2026-06-22
status: complete
verdict: "OVERRIDES current F1 plan (representative-first + two flat fields)"
---

# MITRE ATT&CK JSON Shape Research — Issue #64

**Question:** What is the best JSON representation for MITRE ATT&CK technique->tactic+name
data in wirerust's findings output, given three simultaneous consumers: (1) autonomous LLM
agents, (2) human analysts, (3) SIEMs (Elastic ECS, OCSF, Splunk)?

**One-line verdict:** Resolve **every** technique (not first-only) into a structured
**array of per-technique objects** carrying `id + name + tactic_id + tactic_name (+ reference)`.
This **OVERRIDES** the current F1 plan's representative-first + two flat fields
(`mitre_tactic`, `mitre_name`). The flat fields are a known agent foot-gun and do not align
with any of the three target SIEM schemas.

---

## Source-of-truth schema findings (verified 2026-06-22)

All three major schemas model MITRE as **multi-valued, per-technique, ID+name structured** —
none use a single flattened name/tactic pair.

### Elastic ECS — `threat.*` (verified against elastic.co ECS Threat fields reference)

- `threat.technique.id`, `threat.technique.name`, `threat.technique.reference` — all
  `type: keyword` with the explicit note **"this field should contain an array of values"**. [1][2]
- `threat.tactic.id`, `threat.tactic.name`, `threat.tactic.reference` — same, arrays of
  `keyword`. `threat.tactic.id` example value is **`TA0002`** (tactic IDs ARE included, not
  just names). [1][2]
- `threat.technique.subtechnique.id/.name/.reference` exist (example `T1059.001`). [1][2]
- `threat.framework` = `"MITRE ATT&CK"` (framework name). **ECS has NO dedicated ATT&CK
  *version* field** — version must go in a custom field or elsewhere. [1][2]

> ECS keeps `tactic` and `technique` as **parallel arrays** (positional alignment), not a
> single nested per-technique object. This is the one structural wrinkle: a co-occurring set
> of techniques and tactics is represented as two same-length arrays.

### OCSF — `attacks[]` / `Attack` object (verified against schema.ocsf.io 1.3.0)

Each element of the finding's `attacks[]` array is an **Attack** object with **nested**
sub-objects (this corrects an earlier, hedged quick-lookup answer): [3][4][5]

- `technique` → MITRE ATT&CK Technique object: `{ uid, name, src_url }` (uid e.g. `T1059`,
  `src_url` is the **versioned permalink**). [4]
- `tactic` → MITRE ATT&CK Tactic object `{ uid, name, src_url }` (uid e.g. `TA0002`).
  (`tactics[]` array is **deprecated since v1.1.0** in favor of singular `tactic`.) [3]
- `sub_technique` → Sub Technique object (same shape).
- `version` (**Recommended**) — String, the **ATT&CK version**. OCSF *does* model version
  at the per-Attack level. [3]
- Constraint: at least one of `technique` / `tactic` / `sub_technique` must be present. [3]

> OCSF binds **one technique to its one tactic inside a single object**, and you emit one
> Attack object per technique. This is the cleanest model of the three.

### Splunk (CIM / Enterprise Security risk annotations)

Splunk uses flat-ish fields `mitre_tactic_id` / `mitre_technique_id` (under
`All_Risk.annotations.mitre_attack.*`) and in practice accepts **comma-separated** multi-value
strings (e.g. `"T1486,T1490,T1027"`). This is the **least structured** of the three and is
widely noted as the legacy pattern the industry is moving away from. [6][7] An array of
objects de-normalizes trivially into Splunk's multi-value fields; the reverse (parsing a
flat name back to N techniques) does not.

---

## Q1 — MULTI-TECHNIQUE: first-only vs resolve-every

**RECOMMENDED: Resolve EVERY technique. First-only is not acceptable for this output.**

Rationale:

1. **It is the universal standard.** ECS `threat.technique.*` are arrays; OCSF emits one
   Attack object per technique; even Splunk carries multi-value technique lists. No major
   schema flattens to a single representative technique. [1][2][3]
2. **First-only is a documented agent foot-gun.** A lone `mitre_name` that silently
   corresponds to only one of N techniques is *lossy and ambiguous*: an agent reading
   `mitre_techniques: ["T1071","T1573"]` alongside a single `mitre_name: "Application Layer
   Protocol"` cannot tell whether the name applies to the set, to T1071 only, or is
   authoritative at all. This is precisely the "technique shadowing" failure the research
   surfaced — one technique masks the others and produces false coverage assessments. [8][9]
3. **SIEM mapping breaks.** To populate ECS `threat.technique.name[]` or OCSF `attacks[]`,
   a consumer needs *every* name, not the first. First-only forces every SIEM integrator to
   re-derive the missing names themselves, defeating the entire purpose of emitting names.

**The standard:** all techniques resolved, each paired with its own tactic. Where a finding's
techniques span multiple tactics, each technique carries its *own* tactic — do not assume one
tactic for the finding.

---

## Q2 — SHAPE: flat fields vs nested object vs array of objects

**RECOMMENDED: ARRAY of per-technique objects** —
`mitre_techniques: [{ id, name, tactic_id, tactic_name, reference }]`.

| Shape | Agent | Human | SIEM | Verdict |
|-------|-------|-------|------|---------|
| Two flat fields (`mitre_tactic`, `mitre_name`) | Poor — ambiguous which technique; lossy | OK for single-technique only | Poor — maps to no schema | **Reject** |
| Nested object (`mitre: { tactic, name }`) | Same ambiguity; no multi-value path | Slightly tidier namespace | Poor — still single-valued | **Reject** |
| **Array of per-technique objects** | **Best** — self-contained, unambiguous, predictable | **Good** — each line reads "ID — name (tactic)" | **Best** — de-normalizes to ECS parallel arrays AND OCSF `attacks[]` 1:1 | **RECOMMEND** |

Rationale:

- The array-of-objects is the **only** shape that survives the multi-technique requirement
  (Q1) without ambiguity, and it is the shape the research identifies as the converged
  industry standard across ECS and OCSF. [1][2][3]
- **It maps cleanly to both SIEM targets**, which have *different* structural conventions:
  - OCSF wants **one object per technique with its tactic nested inside** — a 1:1 map from
    our array elements. This is why we bind `tactic_id`/`tactic_name` *inside each technique
    object* rather than as a sibling parallel array.
  - ECS wants **parallel `technique[]` and `tactic[]` arrays** — a trivial projection
    (`map(.id)`, `map(.tactic_id)`) from our array.
  An object-per-technique is the lossless superset; both SIEM shapes are cheap projections of
  it. The reverse is not true.
- **Keep the existing `mitre_techniques: ["T1071","T1573"]` raw ID array unchanged** for
  backward compatibility (F1 delta confirms it is `Vec<String>` and additive-safe). The new
  structured array is an *additional* field so existing ID-only consumers are unbroken.

**Naming:** Avoid colliding with the existing raw-ID field `mitre_techniques`. Use a distinct
key for the resolved structured array (recommended: `mitre_attack` or
`mitre_techniques_resolved`; see snippet — chosen `mitre_attack` to read as the namespaced,
resolved view, leaving the raw `mitre_techniques` ID list intact).

---

## Q3 — AGENT-FIRST considerations

**RECOMMENDED design properties for the agent consumer:**

1. **ID is the durable key; always pair ID + human name.** Keep `id: "T1071"` as the stable,
   version-independent reference *alongside* `name`. Names drift across ATT&CK versions and
   vary by phrasing; the ID does not. This dual representation is the single most-cited
   agent best practice — it removes natural-language ambiguity while still letting the agent
   emit human-readable output without a second lookup. [1][8]
2. **Include the tactic ID `TAxxxx`, not just the tactic name.** ECS does this
   (`threat.tactic.id` example `TA0002`); OCSF does this (`tactic.uid`). The tactic ID lets an
   agent reconstruct the tactic hierarchy and group co-tactic techniques deterministically
   without string-matching tactic names. [1][3]
3. **Stable, predictable, snake_case key names.** Consistent field names
   (`id`, `name`, `tactic_id`, `tactic_name`) reduce agent parse branches and field
   misidentification. Mirror ECS/OCSF semantics so downstream agents trained on those schemas
   transfer directly. [8]
4. **No lossy first-only flattening** (the Q1 result) — arrays remove the "which technique
   does this name belong to?" parse hazard entirely.
5. **Reference URL: optional, recommended.** A `reference` permalink gives the agent
   authoritative context without an external lookup and gives humans a click-through. OCSF
   makes `src_url` the *versioned* permalink, which doubles as a version anchor. Modest token
   cost; high utility. Make it optional so token-budgeted callers can omit.
6. **Token efficiency.** Array-of-objects has some field-name repetition, but the redundancy
   is bounded (findings carry few techniques) and the alternative — agents doing ID->name
   resolution themselves or mis-parsing comma-joined strings — is strictly worse. Keep keys
   short; do not duplicate the raw ID list's content beyond the structured array.

There is no formal RFC for "agent-readable security JSON," but the converged practice the
sources describe is: **structured arrays + durable IDs + co-located human names + stable
field names + explicit per-item tactic context.** Flag: this is an emergent best-practice
synthesis from ECS/OCSF design and vendor guidance, **not** a single normative standard.

---

## Q4 — Reconcile with prior triage note on #64 (ID+name inline, pinned to ATT&CK version)

The prior triage note (ID + name inline, pinned to a MITRE ATT&CK version) is **correct in
spirit and REFINED, not contradicted**, by this research:

- **Confirmed:** keep both ID and name inline (Q3.1). The triage note got the core right.
- **Confirmed:** pin to an ATT&CK version. wirerust already pins `ics-attack-19.1`
  (`src/reporter/json.rs:27`, `mitre_attack_version` envelope field). This research adds that
  **OCSF models version per-Attack** (`version: "19.1"`) and uses *versioned* permalinks —
  so the envelope-level pin is sufficient for now, but if a consumer maps to OCSF they may
  want the version echoed per technique or via the permalink. The envelope `mitre_attack_version`
  field satisfies this; no per-element version field is required.
- **Refined / extended:** the triage note did not settle multi-technique handling or the
  flat-vs-array shape. This research resolves both: **every** technique, as an **array of
  objects**, each carrying its **own tactic (ID + name)**. This is the substantive delta over
  the F1 plan.

---

## Verdict vs current F1 plan

| F1 plan element | This research | Relationship |
|-----------------|---------------|--------------|
| Representative-first (only `mitre_techniques[0]` resolved) | Resolve **every** technique | **OVERRIDE** |
| Two flat fields `mitre_tactic`, `mitre_name` | Array of per-technique objects | **OVERRIDE** |
| Pin to ATT&CK version (envelope `mitre_attack_version`) | Keep; sufficient | **MATCH** |
| Keep raw `mitre_techniques: ["T1071",...]` ID array | Keep unchanged, additive | **MATCH** |
| DTO pattern (`FindingJsonDto<'a>` wrapping `&Finding`) | Still the right mechanism | **MATCH** (only the DTO's *fields* change) |
| Additive / non-breaking schema change | Still additive (new field, raw array untouched) | **MATCH** |

The F1 DTO architecture (a `FindingJsonDto<'a>` computing fields at render time from
`src/mitre.rs` lookups) is **unchanged and still correct**. What changes is *what the DTO
emits*: instead of two `Option<String>` scalars derived from `techniques[0]`, the DTO emits a
`Vec` built by mapping **every** technique through `technique_name` + `technique_tactic`.

> Implementation note (not a code change — for the architect): the existing
> `crate::mitre::technique_name` / `technique_tactic` lookups already return per-ID values, so
> resolving the full array is the same per-element call in a `.iter().filter_map(...)` instead
> of a `.first().and_then(...)`. The cost delta is negligible (few techniques per finding).
> The `tactic_id` (`TAxxxx`) form should be confirmed available from `src/mitre.rs`; if the
> catalog currently stores only tactic *names*, adding the tactic ID is a small catalog
> extension worth doing for ECS/OCSF alignment (flag for architect — **inconclusive without
> reading `src/mitre.rs`, which is out of research scope**).

---

## Proposed JSON snippet (recommended `findings[*]` MITRE representation)

```jsonc
{
  "findings": [
    {
      // ... existing finding fields (analyzer, summary, evidence, timestamp, ...) ...

      // UNCHANGED: raw durable ID list, backward-compatible (existing Vec<String>)
      "mitre_techniques": ["T1071", "T1573"],

      // NEW: resolved, structured, per-technique array. Override of first-only flat fields.
      // One object per technique; each carries its OWN tactic (ID + name).
      // Maps 1:1 to OCSF attacks[] and projects to ECS threat.technique[]/threat.tactic[].
      "mitre_attack": [
        {
          "id": "T1071",
          "name": "Application Layer Protocol",
          "tactic_id": "TA0011",
          "tactic_name": "Command and Control",
          "reference": "https://attack.mitre.org/techniques/T1071/"
        },
        {
          "id": "T1573",
          "name": "Encrypted Channel",
          "tactic_id": "TA0011",
          "tactic_name": "Command and Control",
          "reference": "https://attack.mitre.org/techniques/T1573/"
        }
      ]
    }
  ],

  // UNCHANGED envelope-level version pin (satisfies ATT&CK version requirement for all consumers)
  "mitre_attack_version": "ics-attack-19.1"
}
```

Notes on the snippet:

- `id` first in each object = durable key (Q3.1).
- `tactic_id` in `TAxxxx` form included, not just the name (Q3.2), matching ECS
  `threat.tactic.id` and OCSF `tactic.uid`.
- `reference` optional — include for human click-through + agent context; omit under a tight
  token budget. (Values shown are illustrative *enterprise* IDs for clarity; wirerust emits
  **ICS** ATT&CK IDs per its `ics-attack-19.1` pin — the *shape* is what matters here.)
- `mitre_techniques` (raw IDs) retained so existing ID-only consumers are unbroken; the
  structured `mitre_attack` is purely additive.
- A consumer targeting **ECS** projects: `threat.technique.id = mitre_attack[].id`,
  `threat.technique.name = mitre_attack[].name`, `threat.tactic.id = mitre_attack[].tactic_id`
  (parallel arrays). A consumer targeting **OCSF** maps each `mitre_attack[i]` to one
  `attacks[i] = { technique:{uid,name,src_url}, tactic:{uid,name}, version }`.

---

## Open / inconclusive items (flagged)

1. **Field name `mitre_attack` vs `mitre_techniques_resolved`** — naming is a project
   convention call, not a research finding. Recommended `mitre_attack` (namespaced, reads as
   "the ATT&CK view"), but either is defensible. **Inconclusive — project decision.**
2. **Availability of `tactic_id` (`TAxxxx`) in `src/mitre.rs`** — this research did not read
   `src/mitre.rs` (out of scope). If the catalog stores only tactic names, emitting
   `tactic_id` needs a small catalog addition. **Inconclusive without source read — flag for
   architect.**
3. **"Agent-readable security JSON" normative standard** — none exists as a single RFC; the
   recommendation is a synthesis of ECS/OCSF design + vendor guidance. **Confidence: high on
   direction, no single citable standard.**

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis on multi-audience MITRE JSON design (ECS/OCSF/Splunk/STIX), multi-technique standard, shape comparison, agent-first practices — reasoning_effort: high |
| Perplexity perplexity_ask | 2 | Exact ECS `threat.technique`/`threat.tactic` array/type/format confirmation; OCSF `attacks[]` structure (the OCSF answer was hedged — verified below) |
| WebFetch | 2 | Canonical OCSF schema.ocsf.io 1.3.0 `Attack` object (nested technique/tactic/sub_technique + version) and `Technique` object (uid/name/src_url) — corrected the hedged ask answer |
| Training data | 1 area | General MITRE ATT&CK tactic/technique hierarchy concepts — all schema-specific claims grounded in sources |

**Total MCP tool calls:** 3 (1 perplexity_research + 2 perplexity_ask) + 2 WebFetch verifications.
**Training data reliance:** low — every schema-specific claim (ECS array types, `TA0002`
format, OCSF nested objects, OCSF `version` field) is verified against elastic.co and
schema.ocsf.io. The OCSF nested-object structure was explicitly cross-checked via WebFetch
after the quick-lookup answer hedged, and the canonical schema confirmed nested
`technique`/`tactic`/`sub_technique` objects plus a `version` field.

### Sources

- [1] Elastic ECS Threat fields reference — https://www.elastic.co/docs/reference/ecs/ecs-threat
- [2] Elastic ECS Threat fields (1.12) — https://www.elastic.co/guide/en/ecs/1.12/ecs-threat.html
- [3] OCSF Attack object (1.3.0) — https://schema.ocsf.io/1.3.0/objects/attack
- [4] OCSF Technique object (1.3.0) — https://schema.ocsf.io/1.3.0/objects/technique
- [5] OCSF schema CHANGELOG — https://github.com/ocsf/ocsf-schema/blob/main/CHANGELOG.md
- [6] Splunk ES MITRE tactic/technique fields — https://splunk.my.site.com/customer/s/article/mitre-tactic-id-mitre-technique-id-Field-Values-Missing-in-All-Risk-Data-Model
- [7] Splunk ES default risk incident rules — https://help.splunk.com/en/splunk-enterprise-security-7/risk-based-alerting/7.3/identify-threat/default-risk-incident-rules-in-splunk-enterprise-security
- [8] Elastic ECS GitHub threat.yml — https://github.com/elastic/ecs/blob/master/schemas/threat.yml
- [9] Elastic Security MITRE ATT&CK coverage — https://www.elastic.co/docs/solutions/security/detect-and-alert/mitre-attack-coverage
