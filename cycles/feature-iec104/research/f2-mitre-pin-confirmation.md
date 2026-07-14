# F2 MITRE ATT&CK for ICS v19.1 Pin Confirmation — IEC 60870-5-104 Feature

**Date:** 2026-07-14
**Agent:** vsdd-factory:research-agent
**Policy:** DF-VALIDATION-001 (every verdict version-anchored; unpinnable points flagged, not guessed)
**Scope:** Close the residual version-pin gap from `f2-canonical-fact-validation.md` — pin the eight technique identities to MITRE ATT&CK for ICS **v19.1** specifically (the `ics-attack-19.1` pin in ADR-0013), and validate the pin-string nomenclature itself.
**Predecessor:** `.factory/cycles/feature-iec104/research/f2-canonical-fact-validation.md` (resolved identities against *current* pages; flagged it could not surface a v19.1-labelled artifact).

---

## HEADLINE VERDICTS

1. **`ics-attack-19.1` IS valid MITRE release nomenclature.** v19.1 is the *current* published ATT&CK version (website + CTI data), released 2026-05-12. Point releases (`.1`) are a documented, systematic part of MITRE's `major.minor` schema (v15.1, v16.1, v17.1, v18.1, v19.1 all on the version-history page). The `attack-stix-data` repo names versioned bundles `<domain>-attack-<version>.json` (e.g. `enterprise-attack-9.0.json`), so `ics-attack-19.1` is the correct filename stem for the ICS STIX bundle `ics-attack-19.1.json`. **No pin correction needed. Do NOT downgrade to `v19`/`ics-attack-19.0`** — v19.1 is a real, more-precise, and current minor.

2. **All 8 technique identities are CONFIRMED at v19.1.** The live attack.mitre.org pages serve v19.1 content (site "Current Version" banner = ATT&CK v19.1; content-release date 2026-05-12 matches the "Last Modified: 12 May 2026" stamp on the v19-touched techniques). Every name+tactic pair the feature relies on holds at the pinned version.

3. **ONE DISCREPANCY — in the *task statement*, not the repo.** Task item 7 asserts **T0831 "Manipulation of Control" = tactic "Impair Process Control."** This is **wrong.** T0831's authoritative tactic is **Impact (TA0105)**. The repo (`src/mitre.rs`) already records `T0831 => (…, MitreTactic::IcsImpact)`, which is **correct**. The error is only in the task's phrasing; no code change is implied. Flagged so the orchestrator does not "fix" a correct entry.

---

## VERDICT TABLE (concise)

Legend: **CONFIRMED-AT-v19.1** = identity holds in the v19.1 matrix, verified against the site serving v19.1 content. All T0xxx dates below are the technique-object's own `Last Modified` field; the *matrix* version served is v19.1 in every case.

| # | Technique | Name (v19.1) | Tactic (v19.1) | Verdict | Versioned anchor |
|---|-----------|--------------|----------------|---------|------------------|
| 1 | **T0881** | Service Stop | Inhibit Response Function (TA0107) | **CONFIRMED-AT-v19.1** | techniques/T0881/ · obj v1.1, LastMod 15 Apr 2025, served under site v19.1 |
| 2 | **T1692.001** | Unauthorized Message: Command Message | **Evasion (TA0103) + Impair Process Control (TA0106)** — dual | **CONFIRMED-AT-v19.1** | techniques/T1692/001/ · obj v1.0, Created 20 Apr 2026, LastMod 12 May 2026 (= v19.1 release) |
| 2p | **T1692** (parent) | Unauthorized Message | Evasion + Impair Process Control (dual) | **CONFIRMED-AT-v19.1** (parent/sub structure exists; new in v19) | techniques/T1692/ · Created 20 Apr 2026, LastMod 12 May 2026; v19 Updates "New Techniques" list |
| 3 | **T1692.002** | Unauthorized Message: Reporting Message | **Evasion + Impair Process Control** (dual) — NOT Impact | **CONFIRMED-AT-v19.1** | techniques/T1692/ (sub-tech table) + v19 Updates "New Techniques" list; inherits parent dual-tactic |
| 4 | **T0836** | Modify Parameter | Impair Process Control (TA0106) | **CONFIRMED-AT-v19.1** | techniques/T0836/ · obj v1.3, LastMod 16 Apr 2025, served under site v19.1 |
| 5 | **T0814** | Denial of Service | Inhibit Response Function (TA0107) | **CONFIRMED-AT-v19.1** | techniques/T0814/ · obj v1.1, LastMod 15 Apr 2025, served under site v19.1 |
| 6 | **T0827** | Loss of Control | Impact (TA0105) | **CONFIRMED-AT-v19.1** | techniques/T0827/ · obj v1.0, LastMod 12 May 2026 (= v19.1 release) |
| 7 | **T0831** | Manipulation of Control | **Impact (TA0105)** | **DISCREPANCY vs task text** (task said "Impair Process Control"). Identity itself **CONFIRMED-AT-v19.1**; repo is correct (`IcsImpact`). | techniques/T0831/ · obj v1.0, LastMod 16 Apr 2025, served under site v19.1 |
| 8a | **T0809** | Data Destruction (NOT "Service Stop") | Inhibit Response Function (TA0107) | **CONFIRMED-AT-v19.1** — earlier correction stands | techniques/T0809/ · obj v1.0, LastMod 12 May 2026 (= v19.1 release) |
| 8b | **T0829** | Loss of View | Impact (TA0105) | **CONFIRMED-AT-v19.1** | techniques/T0829/ · obj v1.0, LastMod 12 May 2026 (= v19.1 release) |

**Pin-string verdict:** `mitre_pin: ics-attack-19.1` in ADR-0013 is **VALID MITRE release nomenclature. Keep as-is.**

---

## HOW v19.1 WAS PINNED (methodology — DF-VALIDATION-001)

The residual gap in the predecessor report was "could not surface a v19.1-labelled artifact." This pass closes it via three converging versioned anchors:

1. **Version-history page** (`attack.mitre.org/resources/versions/`) — explicitly lists **"Current Version: ATT&CK v19.1, April 28, 2026 – current."** This establishes that the live site *is* v19.1: every technique page fetched below is therefore v19.1 content by definition, not "current of unknown version."

2. **Changelog** (`attack.mitre.org/resources/changelog.html`) — "v4.4.3 (2026-05-12) … Release ATT&CK content version 19.1." This dates the v19.1 *content* release to **12 May 2026**. Four of the feature's techniques (T0827, T0809, T0829, and the T1692 family) carry `Last Modified: 12 May 2026` — i.e. they were touched *in the v19.1 content release itself*, giving a direct object-level pin to v19.1 for those five entries.

3. **v19.0→v19.1 detailed changelog** exists at `attack.mitre.org/docs/changelogs/v19.0-v19.1/changelog-detailed.html` (and v18.1→v19.0 at the analogous path), confirming MITRE formally versions the ICS delta between these minors.

The four techniques stamped `Last Modified` 15–16 April 2025 (T0881, T0836, T0814, T0831) were last edited in the v17.x line and **carried forward unchanged** into the v19.1 matrix. They are confirmed at v19.1 because the site currently serving them is v19.1, and the v18.1→v19.0 / v19.0→v19.1 ICS changelogs record no rename or tactic-reassignment for them (the v19 ICS churn was the *addition* of sub-techniques — T1691/T1692/T1693/T1694/T1695 and T0846/T0843 decompositions — not edits to these stable Impact/Inhibit-Response/Impair-Process-Control techniques). Confidence: **high**.

### STIX bundle / filename convention (closes the "no versioned artifact" gap)

- The `attack-stix-data` repo README documents that each domain folder holds **version-marked collection bundles named `<domain>-attack-<version>.json`** (worked example in the repo listing: `enterprise-attack-9.0.json`), plus an unmarked `<domain>-attack.json` that always mirrors the most recent release. Applied to ICS: the versioned bundle is **`ics-attack/ics-attack-19.1.json`**. This is exactly the stem in ADR-0013's pin (`ics-attack-19.1`).
- `mitre/cti` repo labels its ICS domain folder **"ATT&CK v19.1 ICS,"** and the ATT&CK Data & Tools page publishes **Version 19.1** spreadsheets — both confirm a v19.1 ICS dataset exists.
- **Residual (flagged, not guessed):** I could not open a live directory listing of `attack-stix-data/ics-attack/` to *visually* confirm the file `ics-attack-19.1.json` is physically committed — one cached snapshot of `ics-attack.json` still showed a May-2025 v17.1 commit (stale mirror). This does not weaken the pin: v19.1 ICS content demonstrably exists (Data & Tools spreadsheets + mitre/cti "v19.1 ICS" label + version-history current=v19.1), and `ics-attack-19.1` is the documented naming stem. Confidence that `ics-attack-19.1.json` is the correct/available bundle name: **high**; direct file-existence observation: **not obtained** (low-effort follow-up: open `github.com/mitre-attack/attack-stix-data/tree/master/ics-attack`).

---

## POINT-RELEASE NOMENCLATURE (task sub-question: does ".1" exist for v19?)

**Yes — unambiguously.** MITRE's version-history page uses a `major.minor` schema and lists point releases across the board: v15.1, v16.1, v17.1, v18.1, and v19.1, each with explicit date ranges. The April-2026 Updates page ties the v19 major release to **two CTI data minors: v19.0 and v19.1**, with formal diff files for 18.1→19.0 and 19.0→19.1. So:

- `ics-attack-19.1` is **correct and current**. It is the *published minor* as of this report.
- The alternative pins the task floated — `v19` (ambiguous: could mean 19.0 or 19.1) or `ics-attack-19.0` (an older, superseded minor) — would be **less precise or stale**. The ADR's choice of the exact current minor is the right call.

---

## REPO CROSS-CHECK (`src/mitre.rs`, read in full)

- The four feature-relevant IDs already seeded resolve **exactly** to the v19.1-authoritative identities:
  - `T0836 => ("Modify Parameter", IcsImpairProcessControl)` ✓ (Impair Process Control / TA0106)
  - `T0814 => ("Denial of Service", IcsInhibitResponseFunction)` ✓ (TA0107)
  - `T0827 => ("Loss of Control", IcsImpact)` ✓ (Impact / TA0105)
  - `T0831 => ("Manipulation of Control", IcsImpact)` ✓ (Impact / TA0105) — **repo is right; the task text's "Impair Process Control" is the error.**
  - `T1692.001 => ("Unauthorized Message: Command Message", IcsImpairProcessControl)` ✓ (one of the two authoritative tactics; MITRE lists dual Evasion + Impair Process Control)
  - `T1692.002 => ("Unauthorized Message: Reporting Message", IcsImpairProcessControl)` ✓ (same dual-tactic note)
- **T0881 (Service Stop) is NOT in the catalogue.** If this feature seeds STOPDT-abuse detection to T0881, it is a **new** entry: `"T0881" => ("Service Stop", MitreTactic::IcsInhibitResponseFunction)` (TA0107), and `SEEDED_TECHNIQUE_ID_COUNT` (currently 28) + `SEEDED_TECHNIQUE_IDS` must bump in lockstep (the `vp007_catalog_drift_guard` test enforces this).
- **No version string is embedded in `src/mitre.rs`.** The module has no `mitre_pin`/matrix-version constant — only prose comments referencing "v18 added Resource Development" and the "v19 remap issue #222 (T0855→T1692.001, T0856→T1692.002)." **The `ics-attack-19.1` pin lives only in ADR-0013 documentation, with no code-level assertion tying the catalogue to that version.** Flagged for the orchestrator: if version provenance needs to be machine-checkable, that is a gap (no test asserts the catalogue matches v19.1); today it is documentation-only.

### Dual-tactic note (T1692 family)
MITRE v19.1 lists T1692 and both sub-techniques under **two** tactics: **Evasion (TA0103)** *and* **Impair Process Control (TA0106)**. The repo's single-tactic model records `IcsImpairProcessControl` — a defensible, authoritative choice (one of the two). This is a data-model nuance, not an error. Note: the task's item-3 alternative "or Evasion" is correct that a second tactic exists, but it is **Evasion, not Impact**; neither T1692 sub-technique is an Impact-tactic technique.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep sweep (reasoning_effort=high) on ATT&CK `major.minor` versioning, whether `v19.1`/`ics-attack-19.1` is a real identifier, v18/v19 release dates, point-release history, and STIX-bundle naming in `attack-stix-data`/`mitre/cti`. |
| Perplexity perplexity_search | 2 | (1) `attack-stix-data ics-attack` filenames + v19.1 release-notes URLs (surfaced the `<domain>-attack-<version>.json` convention, version-history current=v19.1, changelog v19.1@2026-05-12, v19.0→v19.1 changelog); (2) v19 ICS "New Techniques" list + T1692 parent/sub structure + T0855/T0856 remap. |
| WebFetch | 7 | Live v19.1 technique pages T0831, T0881, T0836, T0814, T0827, T0809, T0829 — exact name, tactic(s), object version, created/last-modified dates. |
| Perplexity perplexity_ask | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebSearch | 0 | — |
| Read | 2 | `src/mitre.rs` (catalogue cross-check) + predecessor `f2-canonical-fact-validation.md`. |
| Training data | 0 areas | No identity, tactic, TA-id, or version asserted from model memory; all pinned to attack.mitre.org v19.1 pages + MITRE version-history/changelog + repo file. |

**Total MCP tool calls:** 3 (1 `perplexity_research` + 2 `perplexity_search`) + 7 WebFetch = 10 external retrievals.
**Training data reliance:** low — every technique identity is anchored to the live v19.1 technique page (with object version/date), the version-pin to MITRE's version-history + changelog, and the filename convention to the `attack-stix-data` README.

### Key sources (version-anchored)
- Version history (Current = ATT&CK v19.1): https://attack.mitre.org/resources/versions/
- Changelog (content v19.1 released 2026-05-12): https://attack.mitre.org/resources/changelog.html
- v19.0→v19.1 detailed changelog: https://attack.mitre.org/docs/changelogs/v19.0-v19.1/changelog-detailed.html
- Updates – April 2026 (v19; CTI v19.0 + v19.1; ICS sub-techniques added): https://attack.mitre.org/resources/updates/
- Data & Tools (Version 19.1 / 19.0 spreadsheets): https://attack.mitre.org/resources/attack-data-and-tools/
- attack-stix-data (bundle naming `<domain>-attack-<version>.json`): https://github.com/mitre-attack/attack-stix-data
- mitre/cti (ICS folder labelled "ATT&CK v19.1 ICS"): https://github.com/mitre/cti
- Technique pages (v19.1): attack.mitre.org/techniques/{T0881,T1692,T1692/001,T0836,T0814,T0827,T0831,T0809,T0829}/
- Repo: `/Users/zious/Documents/GITHUB/wirerust/src/mitre.rs`
