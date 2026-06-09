# Release Sequencing: Bundle vs. Split the MITRE Multi-Tag Schema Break and the Modbus Feature

**Type:** general (release engineering)
**Date:** 2026-06-09
**Status:** complete
**Decision owner:** F2 spec-gate / release maintainer
**Scope:** wirerust v0.2.0 → next release. Adds a Modbus TCP analyzer (additive) **and** changes the core `Finding` output field `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` (breaking to JSON + CSV consumers across all analyzers).
**Related prior research:** `.factory/research/modbus-f2-design-decisions.md` (Decision 3 motivates the multi-tag change), `.factory/research/modbus-tcp-research.md`.

> **Confidence legend:** [VERIFIED] = stated in an authoritative source cited below. [INFERRED] = reasoned synthesis across sources. [JUDGMENT] = design judgment where evidence under-determines the answer.

---

## TL;DR — Recommendation

**Choose a SPLIT — specifically variant B2: ship the multi-tag schema break FIRST as a focused breaking release, then Modbus as an additive release.** And do it the *mature* way Trivy and Zeek do it, not as a hard cutover.

Concretely:

- **v0.3.0 — schema release (the break, isolated).** Migrate all existing analyzers (HTTP/TLS/DNS/lifecycle) to multi-tag output. Use the **additive dual-emit** pattern: keep the old single-value key working through a deprecation window, introduce the new list field, default to the new shape with an opt-out compat flag. No new protocol analyzer in this release.
- **v0.4.0 — Modbus release (purely additive).** Add the Modbus TCP analyzer on top of the now-stable multi-tag schema. No schema break. Consumers who migrated at v0.3.0 adopt Modbus at their leisure; the version number truthfully signals "additive feature."

This sequencing is decisively supported by the evidence on three of the five sub-questions (semver signalling, blast-radius/clean-bisection, real-world OSS practice) and is *enabled* — not blocked — by a key codebase fact below.

**The single most important finding for the F2 gate:** the multi-tag change is **NOT required for Modbus to function**. Per `modbus-f2-design-decisions.md` Decision 3, multi-tagging is *motivated* by Modbus's multi-technique findings (a single write PDU matching T0855 + T0836/T0835), but the recommended volume-control mechanism there is **burst aggregation**, and a finding can carry its "most important" technique in a single field if forced. The multi-tag schema is an **independent, cross-cutting improvement** that happens to be surfaced by Modbus — it benefits all four existing analyzers equally. That decoupling is exactly what makes the split clean and removes the only real argument for bundling.

---

## Codebase grounding (verified against the tree)

[VERIFIED] `Cargo.toml` → `version = "0.2.0"`.

[VERIFIED] `mitre_technique: Option<String>` is consumed by **all** analyzers, not just one:
- `src/analyzer/tls.rs` (7 sites), `src/analyzer/http.rs` (9 sites), `src/analyzer/dns.rs`, `src/reassembly/lifecycle.rs` — so the break is genuinely cross-cutting, confirming the "affects HTTP/TLS/DNS + downstream" premise.

[VERIFIED — and decisive for the migration shape] `src/findings.rs:134-135`:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub mitre_technique: Option<String>,
```
The field is **already omitted from JSON when `None`** (the P1.02 symmetry fix). This matters enormously: a consumer that reads `mitre_technique` and a consumer that reads `mitre_techniques` can be served **simultaneously** from one struct with two serialized fields, because absent keys are simply skipped. The Zeek "introduce new `&optional` field, deprecate the old one" pattern (below) maps directly onto this serde setup with near-zero added complexity. CSV is the harder surface (positional/header columns), and is the main reason the break is real rather than cosmetic — see Risk section.

---

## Sub-question 1 — Semver: how do mature 0.x projects sequence a break vs. a feature?

[VERIFIED] For `0.y.z`, SemVer and the Cargo Book treat a bump in `y` as the **major-equivalent**: "for `0.y.z`, changes in `y` should be treated as major releases" (doc.rust-lang.org/cargo/reference/semver.html; semver.org §4). RFC 1105 ("API Evolution") establishes the conservative default: *"breaking changes are assumed to be major changes unless otherwise stated"* (rust-lang.github.io/rfcs/1105-api-evolution.html).

The practical consequence for sequencing: the version number for a 0.x tool can only carry **one** signal per release — `0.2.0 → 0.3.0` says "major-equivalent / expect breakage." If you bundle, that single signal is *overloaded*: a consumer scanning the changelog sees a feature headline (Modbus) and a breaking schema change competing for the same version bump, and the SemVer literature (semver.org issue #333: "in a pre-1.0, the minor acts as both the major and the minor") notes this compression is exactly what makes pre-1.0 version signalling ambiguous. Splitting restores one-signal-per-release:
- **v0.3.0** = the major-equivalent break (truthful, loud).
- **v0.4.0** = a clean additive minor (Modbus), which a consumer can adopt with zero migration anxiety because the version truthfully says "no break here."

[VERIFIED] The research on breaking changes in ecosystems (arxiv.org/html/2605.24397v1; the ACM systematic review 10.1145/3447245) finds that *"non-major releases routinely carry breaking changes that the version number does not advertise, which leads downstream consumers to delay updates."* Bundling a break under a release whose headline is a feature is precisely this anti-pattern. **[VERIFIED] → split is the semver-honest sequencing.**

---

## Sub-question 2 — Consumer blast radius: migrate once vs. twice

The naive intuition is "bundling = consumers migrate once, splitting = twice." **This intuition is wrong for this case**, for two reasons:

1. **The break is migrate-once either way.** The schema change is a single parser edit (`mitre_technique` scalar → `mitre_techniques` array). Modbus is purely additive output — it does **not** force any parser change for existing consumers. So splitting does **not** create a second migration; it creates one migration (v0.3.0) plus one zero-migration feature adoption (v0.4.0). Bundling does **not** save a migration — it just couples the migration to a feature the consumer may not want yet. [INFERRED from the additive nature of Modbus + the cross-cutting nature of the schema field, both verified above.]

2. **Isolation shrinks the *perceived* blast radius and the migration's cognitive load.** [VERIFIED] Keep a Changelog and Common Changelog both stress changelogs are "for humans" and that breaking entries must be prefixed `**Breaking:**` and listed first (keepachangelog.com/en/1.1.0; github.com/vweevers/common-changelog). When a release contains *only* the schema break, the migration note is focused: "all findings now emit `mitre_techniques: []`; update your parser; here is the before/after JSON and CSV." When it is bundled, the consumer must disentangle "does the Modbus analyzer require the new schema? does it affect my existing HTTP parsing?" — Microsoft's FluidFramework breaking-change guidance (github.com/microsoft/FluidFramework/wiki) explicitly calls out that a concise, single-concern breaking note with clear "why" and replacement guidance is what motivates consumers to actually do the migration.

**Net blast-radius verdict [VERIFIED-leaning]:** Splitting is *strictly better or equal* on blast radius — it never forces a second migration (Modbus is additive), and it materially improves migration clarity by isolating the break in a dedicated, focused release.

---

## Sub-question 3 — Risk: cross-cutting refactor + new analyzer in one release

[VERIFIED] The release-engineering principle is "one logical change per release" → clean bisection and clean rollback (planview.com release-management best practices; the "control the variables" framing of releases-as-experiments). Two independent, simultaneously-shipped changes make causal attribution of any regression impossible: if a consumer reports "parsing broke / a finding looks wrong after upgrade," a bundled v0.3.0 leaves you unable to tell whether the multi-tag refactor (touching all four analyzers + JSON + CSV serialization) or the brand-new Modbus analyzer is at fault.

For *this* change the bisection argument is unusually strong because **both** halves are higher-than-average risk:
- The schema change is a **cross-cutting serialization refactor** touching 4 analyzers, the `Finding` struct, the JSON reporter, and the CSV reporter (positional columns — the brittle surface).
- Modbus is a **new stateful analyzer** with its own burst-window and multi-tag emission logic (per `modbus-f2-design-decisions.md`, itself non-trivial: dual-window detection, T0888 remapping, burst aggregation).

Coupling a wide-but-shallow refactor with a narrow-but-deep new feature is the worst pairing for rollback: you cannot revert one without the other. [JUDGMENT, evidence-backed] Splitting gives two independently-revertable, independently-bisectable releases.

---

## Sub-question 4 — Real-world OSS: do comparable tools batch breaks into dedicated releases?

This is where the deep research is most decisive. Findings, tool by tool (all [VERIFIED] against the cited docs/changelogs):

| Tool | How it handles output-schema breaks vs. features | Source |
|---|---|---|
| **Trivy** (Aqua) | **Gold standard, directly analogous.** Two-phase, schema-isolated migration: new JSON schema shipped behind an opt-in flag `TRIVY_NEW_JSON_SCHEMA=true` in **v0.19.0**, made default in **v0.20.0**, old-schema env removed later. The schema migration is treated as its *own* tracked concern with a documented timeline, *not* bundled silently under a feature headline. | github.com/aquasecurity/trivy/discussions/1050; issue #7553 |
| **Zeek** | **Formal schema-evolution policy.** Rules: required field can't be removed without a deprecation cycle; column data type can't be changed in place; new columns are always `&optional`/`&default`; type changes are done by *adding* a new `&optional` field and `&deprecated`-ing the old one across a deprecation cycle tied to the LTS schedule. Breaks are isolated, announced in release notes, and time-boxed — never bundled silently. | zeek.org/2026/02/why-zeek-keeps-breaking-your-test-baselines/ |
| **Suricata** (eve.json) | Maintains a formal `etc/schema.json` and a **dedicated Upgrade Guide** that calls out config/output changes per release, but lacks an in-band schema-version field; breaks tend to land at **major** boundaries with upgrade-guide documentation rather than mid-stream. | docs.suricata.io …/upgrade.html; appendix/eve-schema.html |
| **osquery** | Less formalized; breaking table/column changes appear bundled with feature releases and flagged only in CHANGELOG (e.g. augeas search-semantics break) — explicitly the **anti-pattern** the literature warns against. | github.com/osquery/osquery CHANGELOG.md |
| **Falco** | Least formalized; schema changes largely bundled with feature releases, not categorized in the changelog. | falco.org/docs/reference/changelog/ |

**Convergent pattern [VERIFIED]:** The *more mature and more stability-respected* the tool, the more it (a) **isolates** output-schema breaks from feature work and (b) softens them with a **deprecation/dual-emit window** (Zeek) or an **opt-in-then-default schema-version toggle** (Trivy). The tools that bundle breaks with features (osquery, Falco) are the ones the comparative analysis flags as imposing the highest integration burden on consumers. For a network-forensics tool whose downstream is SIEMs and scripts — the same audience as Zeek/Suricata/Trivy — the right peer group to imitate is Zeek and Trivy, both of which **split and soften**.

Trivy is the closest analogue to wirerust's exact situation (a CLI security scanner changing its JSON report schema) and it is unambiguously a **split + two-phase** model. **[VERIFIED] → real-world evidence favors B (split).**

---

## Sub-question 5 — Is the multi-tag change actually NEEDED for Modbus? (the crux)

[VERIFIED against `modbus-f2-design-decisions.md` Decision 3]: A single Modbus write PDU can legitimately match **T0855 (Unauthorized Command Message) + T0836 (Modify Parameter) + T0835 (Manipulate I/O Image)**, and the recommended detection-engineering norm (Sigma `tags`, Elastic `threat.technique` multi-valued) is **one finding, many technique tags**. So Modbus is the *motivating use case* for multi-tag.

**But "motivating" ≠ "required."** Two independent facts establish that the schema change is severable from Modbus:

1. **The volume/correctness story for Modbus is solved by burst aggregation, not by the schema shape.** Decision 3's recommendation pairs multi-tagging with per-burst event aggregation; the *minimum-change* fallback it offers ("keep T0855 once per burst; emit one finding per burst carrying both T0836 and T0835") could, in principle, still be expressed with a single most-specific technique per finding for a first Modbus cut, deferring multi-tag. Modbus is *useful* (recon detection via T0888, write-burst detection) with single-tag output. [INFERRED from Decision 3.]

2. **The change benefits all four *existing* analyzers, not just Modbus.** HTTP/TLS/DNS findings can also legitimately implicate multiple techniques; the `Vec<String>` is a general expressiveness upgrade to the core `Finding` contract. It is an **independent improvement surfaced by, but not owned by, Modbus.** [VERIFIED — the field lives in `src/findings.rs`, shared by every analyzer.]

**Conclusion:** Because multi-tag is independent of Modbus, there is **no engineering coupling that forces bundling.** The only thing bundling buys is "one fewer release tag," which the semver/blast-radius/bisection evidence shows is a *cost*, not a benefit. This removes the last argument for option A. **[VERIFIED-leaning] → the schema change should be its own release.**

---

## Decision matrix

| Option | Semver honesty | Blast radius | Rollback/bisection | OSS precedent | Verdict |
|---|---|---|---|---|---|
| **A — Bundle both in v0.3.0** | Poor (one bump, two signals; break hidden under feature headline) | Couples break to an unwanted-yet feature; complex migration note | Worst: cannot revert one without the other | osquery/Falco (anti-pattern) | ✗ |
| **B1 — Modbus single-tag first (v0.3.0 additive), multi-tag break later (v0.4.0)** | OK, but defers the break and ships Modbus on a schema you *know* you'll break, creating a second Modbus-output migration | Modbus consumers migrate when multi-tag lands → arguably *adds* a migration for Modbus | Good | — | ◐ acceptable, not preferred |
| **B2 — Schema break first (v0.3.0), Modbus additive (v0.4.0)** | Best: v0.3.0 = honest major-equiv break; v0.4.0 = honest additive minor | Migrate-once for the break; Modbus is zero-migration additive on a now-stable schema | Best: two independently revertable releases | **Trivy, Zeek** (closest analogues) | ✓ **recommended** |

**Why B2 over B1:** B1 ships Modbus on the *old* single-tag schema, then breaks it later — meaning Modbus output consumers eat the migration too, and you ship a brand-new analyzer on a contract you have already decided to abandon. B2 stabilizes the core contract first, so Modbus is built once, on the final schema, and lands as a clean additive feature. B2 also front-loads the riskier, cross-cutting change while the surface area is smaller (no Modbus code to disturb the refactor).

---

## Recommended execution plan (B2, Trivy/Zeek-softened)

**v0.3.0 — "Multi-technique findings" (the isolated break)**
1. Change `Finding` to carry `mitre_techniques: Vec<String>` (empty vec = no technique, replacing today's `None`). Migrate all emission sites in `tls.rs`, `http.rs`, `dns.rs`, `reassembly/lifecycle.rs`.
2. **Soften per Zeek/Trivy.** Leverage the existing `skip_serializing_if` serde behavior: during the deprecation window, emit **both** `mitre_technique` (scalar = first/most-specific, omitted when empty — preserves today's exact JSON for legacy parsers) **and** `mitre_techniques` (the array). Gate the old field behind a `--compat-mitre-scalar` (default on for v0.3.x), to be removed in a later release. This is the Trivy "new-schema-available-before-default" / Zeek "new `&optional` field + deprecate old" pattern applied to wirerust's serde setup. [VERIFIED patterns; mapping is [JUDGMENT].]
3. **CSV** is the genuine break (positional columns can't dual-emit cleanly). Document the column change explicitly: either widen the `mitre_technique` column to a delimited list (e.g. `;`-joined) — lowest-friction for scripts doing substring matches — or add a `mitre_techniques` column. Pick one and put a `**Breaking:**` before/after example in the changelog.
4. Changelog: `**Breaking:**`-prefixed entry, first in the Changed section, with before/after JSON + CSV, the "why" (multi-technique findings are more accurate and are required to represent ICS write events that span tactics), and the compat-flag/removal timeline. (keepachangelog + common-changelog + FluidFramework conventions, all [VERIFIED].)

**v0.4.0 — "Modbus TCP analyzer" (purely additive)**
5. Add Modbus on top of the stabilized `Vec<String>` contract, emitting multi-tag findings natively (T0888 recon, T0855+T0836/T0835 write events, per `modbus-f2-design-decisions.md`).
6. Changelog: `Added` section only. No `**Breaking:**`. Version truthfully signals "safe additive upgrade."

**Later — remove the compat scalar** in a subsequent minor (announce target version when you deprecate, Zeek-style), or fold it into the eventual 1.0.

> Gitflow note: each release is a normal `release/0.3.0` then `release/0.4.0` branch off `develop` → PR into `main`, per CLAUDE.md. The split costs one extra release branch — cheap relative to the bisection/rollback insurance it buys.

---

## Honest counter-arguments (where A could win)

- **Pre-1.0 license to break freely.** [VERIFIED] At 0.x, consumers already expect volatility every minor bump (semver.org issue #442's "risk buckets"). A maintainer could argue "we're 0.x, just break once and move on." This is the strongest case for A. The rebuttal: forensics output feeds SIEMs/scripts in operational security contexts where even 0.x consumers are migration-sensitive, and the cost of splitting here is *one extra release tag* — trivially cheap insurance. The pre-1.0 freedom argues for being *allowed* to break, not for *bundling* the break with a feature.
- **Release overhead / velocity optics.** Two releases can read as slower cadence. Mitigated by clear changelog rationale; and the two releases can land close together (the schema PR and the Modbus PR are already separable work).
- **If the team is firmly single-release-only**, the least-bad bundling is still: do the schema migration as its own *commit/PR* inside v0.3.0 (so `git bisect` retains value even if the *release* is bundled), and write the changelog with two clearly separated sections, break first. But this is a fallback, not the recommendation.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Bundle-vs-split release engineering: semver/Cargo/RFC-1105 sequencing, Keep-a-Changelog/Common-Changelog/FluidFramework break-documentation, one-logical-change/blast-radius/bisection literature, OSS examples. (2) Targeted real-world: how Suricata/Zeek/osquery/Trivy/Falco sequence output-schema breaks vs. features (verified Trivy two-phase flag, Zeek deprecation policy, osquery/Falco bundling anti-pattern). Both run at `reasoning_effort: high`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Grep / Read (local codebase) | 4 | Verified v0.2.0, cross-cutting `mitre_technique: Option<String>` usage (tls/http/dns/lifecycle), and the `skip_serializing_if = "Option::is_none"` serde behavior that enables clean dual-emit migration. Cross-referenced `modbus-f2-design-decisions.md` Decision 3 for the "multi-tag is motivated-but-not-required by Modbus" crux. |
| Training data | 1 area | SemVer/gitflow general knowledge cross-checked against the Perplexity-cited authoritative pages; all load-bearing claims sourced to citations below. |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated primary tool), `reasoning_effort: high`.
**Training data reliance:** low — every recommendation is anchored to a cited source (semver.org, Cargo Book, RFC 1105, Keep a Changelog, Common Changelog, FluidFramework, Trivy discussions/issues, Zeek schema-evolution post, Suricata upgrade docs, osquery/Falco changelogs, breaking-change ecosystem literature) or to a directly-verified codebase fact.

### Key sources (verified)
- SemVer: semver.org (§4 pre-1.0; issues #333, #442); doc.rust-lang.org/cargo/reference/semver.html; rust-lang.github.io/rfcs/1105-api-evolution.html
- Changelog/break docs: keepachangelog.com/en/1.1.0; github.com/vweevers/common-changelog; github.com/microsoft/FluidFramework/wiki (Communicating breaking changes)
- Release engineering: planview.com software-release-management best practices; breaking-change ecosystem studies (arxiv.org/html/2605.24397v1; dl.acm.org/doi/fullHtml/10.1145/3447245)
- OSS schema evolution: **Trivy** github.com/aquasecurity/trivy/discussions/1050 & issue #7553 (two-phase `TRIVY_NEW_JSON_SCHEMA` flag); **Zeek** zeek.org/2026/02/why-zeek-keeps-breaking-your-test-baselines/ (deprecation policy); **Suricata** docs.suricata.io upgrade & eve-schema; **osquery** github.com/osquery/osquery CHANGELOG.md; **Falco** falco.org/docs/reference/changelog
- Codebase: `Cargo.toml` (v0.2.0); `src/findings.rs:119-148`; `src/analyzer/{tls,http,dns}.rs`, `src/reassembly/lifecycle.rs`; `.factory/research/modbus-f2-design-decisions.md` (Decision 3)
