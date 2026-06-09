# F2 Multi-Tag Schema Validation — Multiple MITRE ATT&CK Techniques per Finding

**Type:** general (schema / output-format design validation)
**Date:** 2026-06-09
**Author:** research-agent (vsdd-factory)
**Status:** complete
**Scope:** Validate wirerust's proposed JSON + CSV output schema for representing multiple MITRE ATT&CK (ICS) techniques per security finding against authoritative schema standards.

---

## 0. TL;DR — Recommendation

**The wirerust design is sound. Ship it as-is, with two small clarifications and one optional future-proofing note.**

| Design element | wirerust proposal | Verdict | Authority |
|----------------|-------------------|---------|-----------|
| **JSON: array field** | `"mitre_techniques": ["T0855","T0836"]` | **KEEP** — correct | ECS `threat.technique.id` is a flat keyword array; Suricata EVE uses JSON arrays; Sigma uses YAML lists |
| **JSON: flat IDs vs objects** | flat array of ID strings | **KEEP for primary field** — matches ECS exactly. (OCSF/STIX nest, but for different reasons — see §1.3.) | ECS = parallel primitive arrays; OCSF/STIX = object graphs for enrichment/CTI exchange |
| **Field NAME** | `mitre_techniques` | **KEEP** — acceptable; consider documenting ECS alias `threat.technique.id` for SIEM mapping | ECS `threat.technique.id`, OCSF `attacks[]`, Sigma `tags`, Suricata `metadata.mitre_technique` |
| **CSV: semicolon-joined single column** | `"T0855;T0836"` | **KEEP** — this is the de-facto security convention | PowerShell `[string]::join(";",…)`, Splunk `makemv delim=";"`, Zeek `set_separator`, European-Excel-safe |
| **Empty/absent** | absent (serde skip) | **KEEP** — acceptable and consumer-friendly. CSV must emit empty string. | ECS/OCSF omit empty; absent ≫ null for SIEM ingestion |
| **Ordering** | canonical construction order | **KEEP** — defensible. ECS/OCSF/STIX treat as set/list (no mandated order), so a *deterministic* order is strictly better, not worse. | No standard mandates order; determinism aids diffing/testing |

**One required clarification:** the matrix/domain (ICS vs enterprise) question — see §1.4. wirerust emits **only** ICS ATT&CK (`T0xxx`), and ICS IDs do not collide with enterprise (`T1xxx`) IDs, so a separate `matrix` field is **not required** for correctness. Document the domain once at the schema/report level rather than per-technique.

---

## 1. JSON Representation

### 1.1 Is an array field the right representation? — YES, unambiguously.

Every mature schema represents multiple techniques per event as a **collection**, never a delimited scalar. The question is only *flat array vs array-of-objects*, and that depends on the schema's purpose (telemetry normalization vs CTI exchange).

- **Elastic ECS** — `threat.technique.id` is a **flat array of keyword (string) values**. The ECS `threat.yml` schema definition states the field "can have multiple values to allow for the techniques used by a threat to be recorded" and the fieldset note explicitly says **"this field should contain an array of values."** ECS uses *parallel normalized arrays* (`threat.technique.id`, `threat.technique.name`, `threat.technique.subtechnique.id`, `threat.tactic.id`) rather than an array of nested `{id,name}` objects. [1][2][4]
- **Suricata EVE JSON** — multiple `metadata: mitre_technique` rule declarations are emitted as a JSON **array of strings** under `alert.metadata` (e.g. `"mitre_technique": ["T1059.001","T1086"]`). EVE deliberately uses arrays to avoid delimiter encoding. [10][11]
- **Sigma** — the `tags` field is a **YAML list** of namespaced ID strings (`- attack.t1059`), one element per technique. [12][13]
- **OCSF** — uses an `attacks[]` **array of objects**, each object carrying `technique{uid,name}`, a `tactic`, version, and optional confidence/severity. [6][7]
- **STIX 2.1** — does not embed technique IDs in the event at all; each technique is an `attack-pattern` SDO with `external_references[].external_id` (source_name `mitre-attack`), linked to an observation/`sighting` via relationship objects. [8][9]

**Conclusion:** `"mitre_techniques": [...]` (a JSON array) is correct and universally expected.

### 1.2 Flat array of ID strings vs array of objects?

wirerust's flat array `["T0855","T0836"]` is **exactly the ECS model** — the single most widely-ingested telemetry schema in the SIEM space. ECS is the lingua franca for Elasticsearch/Kibana, Beats, and the OpenTelemetry security mapping; SIEMs and ingestion pipelines already know how to consume `threat.technique.id` as a string array.

OCSF and STIX use richer object structures, but **not because flat arrays are wrong** — it is because those schemas serve a different mission:

- **OCSF `attacks[]` objects** exist to carry *enrichment* (technique name, description, tactic linkage, ATT&CK version) so that a normalized event is self-describing across vendors without a lookup. The richness is an *enrichment convenience*, not a correctness requirement.
- **STIX object graph** exists for *threat-intelligence exchange* between organizations, where each technique is a first-class shareable, relatable, versioned entity.

wirerust is a **producer of detection telemetry**, not a CTI-exchange broker or a cross-vendor normalization layer. Its job is to emit the finding; downstream enrichment (technique name, tactic, description) is the SIEM's job via ATT&CK lookup tables. For that role, the **flat array of IDs is the correct, idiomatic choice** — it matches ECS, it is trivially machine-parseable, and it avoids baking volatile ATT&CK metadata (names/descriptions change between ATT&CK versions) into every emitted finding.

**Verdict: keep the flat array of ID strings.** Do *not* adopt the OCSF/STIX nested-object form for the primary field — it would couple wirerust output to a specific ATT&CK version's naming and add maintenance burden for no consumer benefit at the telemetry layer.

> If wirerust ever needs to emit human-readable names without a downstream lookup, the ECS-idiomatic move is a **second parallel array** (`mitre_technique_names`), not nested objects. Defer unless a consumer asks.

### 1.3 Field name

| Schema | Field name | Shape |
|--------|-----------|-------|
| ECS | `threat.technique.id` | flat string array |
| OCSF | `attacks[].technique.uid` | array of objects |
| Sigma | `tags` (`attack.tNNNN`) | YAML list |
| Suricata EVE | `alert.metadata.mitre_technique` | string array |
| Zeek | user-defined `set[string] &log` | TSV set |

`mitre_techniques` is **clear, self-documenting, and acceptable.** It is not a reserved name in any standard, so there is no collision risk. The plural correctly signals multiplicity. **Recommendation: keep `mitre_techniques`**, and in the schema documentation note the canonical ECS mapping (`mitre_techniques → threat.technique.id`) so anyone writing an ingest pipeline (Logstash/Vector/Cribl → Elastic) has the alias ready. This is a documentation line, not a schema change.

### 1.4 Does the matrix (enterprise vs ICS) need to be in the schema?

**No — not for correctness, given wirerust's constraints.** Key facts:

- MITRE uses **disjoint ID namespaces per matrix**: Enterprise = `T1xxx`, ICS = `T0xxx`, Mobile in its own range. IDs **do not collide across matrices**. [3][5]
- wirerust emits **only ICS ATT&CK** techniques (`T0xxx`). Every ID is unambiguously ICS by its `T0` prefix.

Therefore a per-technique `matrix` field would be **pure redundancy** — every value would be `"ics"`. The ECS guidance and OCSF both *support* a domain/matrix field for tools spanning multiple matrices, but that is precisely the case wirerust does not have today.

**Recommendation:** Do **not** add a per-technique matrix field. Instead, declare the domain **once** at the schema/report-metadata level (e.g. a top-level `"mitre_domain": "ics-attack"` and/or `"mitre_attack_version": "<vN>"` on the report envelope, not on each finding). This:
1. Removes ambiguity for any consumer that does not know `T0` ⇒ ICS.
2. Captures the ATT&CK version — the one piece OCSF/ECS both flag as genuinely important for correct historical interpretation as ATT&CK evolves. [6][7]
3. Costs one field on the envelope instead of N redundant fields on every finding.

This is **optional** for correctness but **recommended** as cheap future-proofing. If wirerust ever adds enterprise-matrix mappings, the `T0`/`T1` prefix still disambiguates per-value, so no schema break is forced.

---

## 2. CSV Representation

### 2.1 Is semicolon-joining in ONE column the accepted convention? — YES.

The semicolon-as-intra-field-delimiter is the **de-facto standard** for multi-valued security data in CSV, and it is what every downstream tool expects to be able to split:

- **PowerShell** (the workhorse of security CSV export): the established idiom is `[string]::join(";", $values)` — semicolon is the documented community convention precisely because it avoids collision with the comma field delimiter. [16]
- **Splunk**: `| makemv delim=";" mitre_techniques` is the standard incantation to turn a semicolon-joined column into a true multivalue field; `mvexpand` then explodes it. Splunk's *default* `makemv` delimiter is a single space, so a semicolon is an explicit, deliberate, and well-trodden choice for analysts. [17][18][19]
- **pandas**: `df['mitre_techniques'].str.split(';')` (or a `converters={'mitre_techniques': lambda x: x.split(';')}` on `read_csv`) is the canonical transform. [20]
- **Zeek**: uses a *two-layer* separator model (tab between fields, `set_separator` — default comma, redefinable to `;` — within a field). This is the same principle: a distinct intra-cell separator that cannot collide with the field separator. [21][22]
- **Excel**: a cell containing `T0855;T0836` is read as a single string in US locales (comma field-delimiter), and the analyst splits via Text-to-Columns on `;`. Bonus: semicolon is *also* European-Excel's field delimiter, but since the values contain no commas and the file is comma-delimited, the multi-valued cell survives intact and is split intentionally.

### 2.2 Is comma-delimiter + semicolon-intra-cell safe and standard? — YES, and it is unambiguous for ATT&CK IDs.

The safety argument is decisive for this specific data type:

- MITRE ATT&CK IDs match `^T[0-9]{4}(\.[0-9]{3})?$` for enterprise and `^T0[0-9]{3}$` for ICS — i.e. **`[A-Z0-9.]` only**. They contain **no commas, no semicolons, no quotes, no whitespace.**
- Because the values can never contain the field delimiter (`,`) or the intra-cell delimiter (`;`), there is **zero quoting/escaping ambiguity** under RFC 4180. The cell does not even need to be quoted (though quoting it is harmless and slightly safer for naive parsers). [23]

This is the *ideal* case for delimiter-joining: a closed, restricted alphabet with no delimiter characters. The semicolon choice is strictly safer than pipe (`|`, which some parsers treat specially) and far simpler than the alternatives.

### 2.3 Why not the alternatives?

| Alternative | Verdict | Reason |
|-------------|---------|--------|
| **Repeated columns** (`technique_1`, `technique_2`, …) | Reject | Requires a fixed max-arity; sparse; breaks uniform analytics ("count by technique" becomes painful); schema churns when arity grows. |
| **Pipe-joined** (`T0855\|T0836`) | Avoid | Works, but `\|` is a regex/SPL metacharacter and is non-idiomatic vs semicolon in PowerShell/Splunk security tooling. No advantage over `;` here. |
| **JSON-in-cell** (`["T0855","T0836"]`) | Reject for CSV | Defeats the point of CSV (flat, spreadsheet-friendly); Excel shows raw brackets/quotes; requires JSON parsing per-cell. Reserve structured arrays for the JSON output. |
| **Exploded rows** (one row per technique) | Reject as default | Duplicates every other column; inflates row count; corrupts per-finding counts and "1 finding = 1 row" invariants. (It is what `mvexpand` produces *on demand* — a consumer choice, not a wire format.) |
| **Semicolon-joined single column** | **CHOSEN** | Idiomatic, safe for ATT&CK alphabet, splittable by every major tool, preserves "1 finding = 1 row." |

**Recommendation: keep `T0855;T0836` semicolon-joining.** Optionally emit a literal `sep=,` first line *only if* you find European-locale Excel double-clicking is a real consumer workflow — but for programmatic consumers (Splunk/pandas) it is unnecessary, and most pipelines specify the delimiter explicitly. Treat `sep=,` as a defer-unless-needed nicety.

---

## 3. Empty / Absent Technique

**wirerust choice: absent in JSON via serde skip. This is correct and consumer-friendly.**

- **ECS and OCSF both omit empty threat fields** rather than emitting `null` or `[]`. Absent-when-empty is the dominant convention because:
  - SIEM ingestion (Elastic, Splunk) treats a missing field and an empty array nearly identically for search, but a present `null` can trigger mapping/type-inference noise and wastes index space at scale.
  - `null` is the worst option: it forces consumers to handle a third state (present-but-null) distinct from absent and empty.
- A finding with **no** technique is a legitimately common case (many protocol anomalies have no ATT&CK mapping), so making the field optional via `skip_serializing_if` is the right model.

**One caveat — CSV cannot omit a column.** In a fixed-schema CSV, the `mitre_techniques` column must exist on every row. For a finding with no technique, emit an **empty string** (`,,`), **not** the literal `null`, `[]`, or `none`. Empty cell ⇒ `makemv` yields an empty multivalue, `str.split(';')` on empty yields `['']` (guard with `if x else []` in the converter), and Excel shows a blank. This is the standard and least-surprising behavior.

**Recommendation:**
- JSON: **absent when empty** (keep the serde skip). ✔
- CSV: **empty string in the column** when no technique. Verify the converter/`makemv` empty-cell path so a no-technique row does not produce a spurious `[""]` value in analytics.

---

## 4. Ordering — Set vs Ordered List

**The standards treat the technique collection as a set/list with no mandated canonical order:**

- ECS `threat.technique.id` is a normalized array — order is not semantically significant in the spec.
- OCSF `attacks[]` and Sigma `tags` are lists with no required ordering.
- STIX models techniques as independently-related objects — inherently unordered.

Because **no standard mandates an order**, wirerust imposing a **deterministic canonical construction order** (for primary-tactic bucketing) is:

1. **Fully compliant** — you are allowed to choose any order; a set is a subset of "any order."
2. **Strictly better than nondeterministic order** for *your* needs: deterministic output makes findings byte-for-byte reproducible, which is essential for golden-file/snapshot tests, diffing two runs over the same pcap, and the input-hash drift detection this project relies on.
3. **Harmless to consumers** — every consumer that cares (Splunk `mvexpand`, set operations in pandas, Elastic terms aggregations) treats the values as a set and is order-insensitive. None will break if the order is fixed; none requires the order to be fixed.

**Recommendation: keep the canonical construction order.** Document it as an *implementation guarantee* (deterministic output) rather than a *consumer contract* (do not promise consumers any semantic meaning to the order — e.g. do not let a downstream infer "first = primary technique" unless you explicitly specify that, which would then become a maintained contract). If primary-tactic bucketing means the first element *is* meaningful, state that explicitly in the schema doc so it becomes an intentional, tested contract rather than an accident.

---

## 5. Final Recommendation Summary

wirerust's design — **`mitre_techniques` JSON array of flat ICS ID strings + semicolon-joined CSV column + absent-when-empty (serde skip) + canonical deterministic order** — is **sound and aligns with authoritative schema conventions.** Specifically:

1. **JSON array of flat ID strings** — matches ECS `threat.technique.id` exactly, the most SIEM-ingestible model. Do not adopt OCSF/STIX nesting at the telemetry layer. ✔ **No change.**
2. **`mitre_techniques` field name** — fine; document the `→ threat.technique.id` ECS mapping for ingest pipelines. ✔ **No change; add doc note.**
3. **Matrix/domain** — not needed per-technique (`T0` prefix disambiguates ICS, no cross-matrix collisions). **Recommended (optional):** declare `mitre_domain: "ics-attack"` + `mitre_attack_version` once on the report envelope. ⚙ **Optional add at envelope level.**
4. **Semicolon-joined CSV** — the de-facto security convention (PowerShell/Splunk/Zeek/pandas all expect splittable semicolons); ATT&CK's `[A-Z0-9.]` alphabet makes it escape-safe. ✔ **No change.**
5. **Empty handling** — absent in JSON ✔; **must be empty string (not null) in CSV** — verify the empty-cell split path. ✔ **No change to JSON; confirm CSV empty path.**
6. **Ordering** — canonical deterministic order is compliant and beneficial for reproducibility/testing; consumers are order-insensitive. ✔ **No change; document whether element-0 carries "primary" meaning.**

**Net:** zero blocking changes. Two documentation clarifications (ECS field alias; whether order is semantically meaningful), one CSV empty-cell verification, and one optional envelope-level `mitre_domain`/`mitre_attack_version` addition for future-proofing.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Deep multi-source synthesis of ECS / OCSF / STIX 2.1 / MITRE ATT&CK data-model representation of multiple techniques per event (flat array vs nested objects, ID namespaces, matrix collision). (2) Deep synthesis of CSV multi-value conventions + Suricata EVE / Zeek log format / Sigma spec + Excel/pandas/Splunk ingestion behavior. Both run at `reasoning_effort: high`. |
| Perplexity perplexity_ask | 1 | Load-bearing verification: confirm ECS `threat.technique.id` is a flat string array (not nested objects) and confirm OCSF field name is `attacks[]`. Domain-filtered to elastic.co / schema.ocsf.io / github.com. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | — |
| Tavily | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area | ATT&CK ID regex shapes (`T0xxx` / `T1xxx`) and RFC 4180 quoting basics — corroborated by the research outputs, flagged here for transparency. |

**Total MCP tool calls:** 3 (2 × `perplexity_research` high-effort + 1 × `perplexity_ask` domain-filtered)
**Training data reliance:** low — every load-bearing claim (ECS flat-array note, OCSF `attacks[]`, Suricata EVE arrays, Sigma YAML list, Zeek `set_separator`, Splunk `makemv delim=";"`, PowerShell `join(";")`, ATT&CK ID namespace disjointness) is sourced from web-grounded MCP research with citations below. ATT&CK ID regex and RFC 4180 escaping are general knowledge cross-confirmed by the research.

### Key Sources

ECS threat fields: elastic.co/guide/en/ecs/8.17/ecs-threat.html [1], elastic.co/docs/reference/ecs/ecs-threat [2], github.com/elastic/ecs/blob/master/schemas/threat.yml [4], elastic.co/docs/reference/ecs/ecs-threat-usage.
MITRE ATT&CK matrices / ID namespaces: attack.mitre.org [3], attack.mitre.org/techniques/enterprise/ [5], attack.mitre.org/resources/attack-data-and-tools/.
OCSF: github.com/ocsf [6], github.com/ocsf/ocsf-schema/releases [7], ocsf.io.
STIX 2.1: docs.oasis-open.org/cti/stix/v2.1/cs02/stix-v2.1-cs02.html [8], oasis-open.github.io/cti-documentation/stix/intro.html [9], github.com/mitre-attack/attack-data-model (stix-external-references).
Suricata EVE: docs.suricata.io/en/latest/output/eve/eve-json-output.html [10], docs.suricata.io/en/latest/rules/meta.html [11].
Sigma: sigmahq.io/docs/basics/rules.html [12], sigmahq.io/sigma-specification/specification/sigma-appendix-tags.html [13].
PowerShell CSV join: millersystems.com/powershell-exporting-multi-valued-attributes-via-export-csv-cmdlet/ [16].
Splunk makemv/mvexpand: help.splunk.com (Makemv) [17], help.splunk.com (makemv overview/syntax) [18], help.splunk.com (Mvexpand) [19].
pandas: pandas.pydata.org/docs/reference/api/pandas.Series.str.split.html [20].
Zeek logs / set_separator: docs.zeek.org/en/master/tutorial/logs.html [21], github.com/zeek/zeek/blob/master/scripts/base/frameworks/logging/main.zeek [22].
CSV / RFC 4180: ietf.org/rfc/rfc4180.txt [23], en.wikipedia.org/wiki/Comma-separated_values.
