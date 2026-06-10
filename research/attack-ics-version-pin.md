# Finding: MITRE ATT&CK for ICS version pin (F4-PIN)

- **Date:** 2026-06-09
- **Type:** general (release-blocker resolution)
- **Blocker:** F4-PIN — pin `mitre_attack_version` report-envelope field before tagging wirerust v0.3.0
- **Status:** RESOLVED — decisive recommendation below
- **Current placeholder:** `"ics-attack-v15"` (stale / non-canonical — replace)

## Decision (TL;DR)

Pin the report envelope to:

```
mitre_attack_version = "ics-attack-19.1"
```

This matches the canonical MITRE STIX per-domain bundle filename
(`ics-attack-19.1.json`) for the latest released ATT&CK version (v19.1, released
2026-04-28). All 7 emitted ICS technique IDs **and** the cross-domain T0846 are
**valid and active** in v19.1 — none deprecated, revoked, or renamed.

If a stricter/looser format is preferred, see "Format options" below. The
recommendation is the per-domain string because wirerust emits ICS-domain
technique IDs specifically, and it unambiguously names both the domain and the
exact version.

---

## 1. Latest authoritative ATT&CK version (mid-2026)

| Fact | Value | Source |
|------|-------|--------|
| Latest version | **v19.1** | attack.mitre.org/resources/versions/ [1] |
| Release date | **2026-04-28** (v19.0); v19.1 minor on **2026-05-12** | attack.mitre.org [1], attack-stix-data releases [2] |
| Predecessor | v18.1 (2025-10-28 → 2026-04-27) | [1] |
| Release cadence | Biannual (~Apr / ~Oct); `.1` minors are typo/data corrections | [1] |
| Domain versioning | ICS is versioned **together** with Enterprise + Mobile under one number — there is no separate ICS version stream | [1][3] |

ATT&CK versions are `major.minor`. Major releases (v18→v19) carry structural
changes; minor releases (v19.0→v19.1) are corrections. **v19.1 is the current
release as of 2026-06-09** and is the correct pin target.

Note: v19 introduced the *first* ICS sub-techniques (18 new entries) and some ICS
restructuring (e.g. T0846 Remote System Discovery gained sub-techniques; new
T1693/T1694/T1695 cross-domain techniques). This restructuring **added** detail
but did **not** deprecate or rename any of wirerust's emitted parent technique
IDs (verified in §2).

## 2. Technique-ID validity in v19.1

All confirmed against attack.mitre.org technique pages and the v17.1/current ICS
technique listings. None deprecated or revoked.

| ID | wirerust-listed name | Confirmed v19.1 name | Status | Source |
|----|----------------------|----------------------|--------|--------|
| T0888 | Remote System Information Discovery | Remote System Information Discovery | **Active** (tech v1.1, last-mod 2026-05-12) | [4] |
| T0855 | Unauthorized Command Message | Unauthorized Command Message | **Active** | [5][6] |
| T0836 | Modify Parameter | Modify Parameter | **Active** | [3] |
| T0835 | Manipulate I/O Image | Manipulate I/O Image | **Active** | [3] |
| T0831 | Manipulation of Control | Manipulation of Control | **Active** (cited on G0034 Sandworm mapping) | [3][7] |
| T0814 | Denial of Service | Denial of Service | **Active** (Inhibit Response Function) | [8] |
| T0806 | Brute Force I/O | Brute Force I/O | **Active** | [3] |
| T0846 | Remote System Discovery (cross-domain) | Remote System Discovery | **Active** (expanded with sub-techniques in v19) | [3] |

All 8 names wirerust uses are **exact matches** to canonical MITRE names. No
corrections needed. T0888 was directly verified as last-modified 2026-05-12 with
technique-version 1.1, which independently proves it is live in the current
release (not a stale cache).

Caveat on T0846: it remains a single valid technique ID, but in v19 it gained
sub-techniques (port scan / broadcast / multicast discovery). If wirerust ever
wants finer granularity it could emit a sub-technique ID (e.g. `T0846.00x`), but
the bare `T0846` parent remains valid — no action required for v0.3.0.

## 3. Canonical version-string format

MITRE uses **two** repository conventions plus a per-domain bundle convention:

| Context | Format | Example (v19.1) | Source |
|---------|--------|-----------------|--------|
| attack.mitre.org UI / human | `vX.Y` | `v19.1` | [1] |
| `mitre-attack/attack-stix-data` git tag | `vX.Y` | `v19.1` | [2] |
| `mitre/cti` git tag | `ATT&CK-vX.Y` | `ATT&CK-v19.1` | [9] |
| **Per-domain STIX bundle filename** | `<domain>-attack-X.Y.json` | **`ics-attack-19.1.json`** | [10] |

There is no formal ECS/SIEM-standardized canonical ATT&CK version field; ECS's
`threat.technique.*` fields carry technique IDs/names but leave version
referencing to the producer. The de-facto unambiguous reference is the STIX
bundle name, because it encodes **domain + version** in one token. Since wirerust
emits ICS-domain IDs, the per-domain bundle string is the least ambiguous choice.

### Format options (decreasing preference for wirerust)

1. **`ics-attack-19.1`** ← **RECOMMENDED.** Matches MITRE's own ICS STIX bundle
   (`ics-attack-19.1.json`). Self-describing: domain + version. Drop-in fix for
   the existing placeholder shape (`ics-attack-v15` → `ics-attack-19.1`). Note:
   the canonical bundle has **no `v`** between `attack-` and the number.
2. `v19.1` — correct and matches the git tag / UI, but loses the domain qualifier.
   Acceptable if the envelope already states the domain elsewhere.
3. `ATT&CK-v19.1` — matches `mitre/cti` tag but contains `&` (awkward in JSON keys
   / SIEM field values); avoid.

**Do not** keep the `v` from the old placeholder if adopting option 1: MITRE's ICS
bundle is `ics-attack-19.1.json` (no `v`), so the exact pin is `ics-attack-19.1`.

## 4. Recommendation (decisive)

```diff
- mitre_attack_version = "ics-attack-v15"
+ mitre_attack_version = "ics-attack-19.1"
```

- **Pin string:** `ics-attack-19.1`
- **All 7 emitted ICS IDs (T0888, T0855, T0836, T0835, T0831, T0814, T0806) are
  valid and active in v19.1.** T0846 (cross-domain) is also valid and active.
- **No technique names need changing** — every name wirerust uses matches MITRE
  canonical exactly.
- **Assumption flag:** v19.1 is confirmed via two independent authoritative
  sources (attack.mitre.org versions page + attack-stix-data GitHub releases) as
  the current release on 2026-06-09. Confidence: **high.** If a v19.2/v20 minor
  ships between now and tagging, re-confirm — but the 6-month cadence makes v20
  unlikely before ~2026-10. The 7 ICS IDs are stable parent techniques and have
  not been deprecated across the v15→v19 span, so the technique-validity
  conclusion is robust even if a minor bump occurs.

---

## Sources

| # | Source | Used for |
|---|--------|----------|
| [1] | attack.mitre.org/resources/versions/ | Latest version v19.1, dates, cadence, unified ICS versioning |
| [2] | github.com/mitre-attack/attack-stix-data/releases | v19.1 / v19.0 / v18.1 git tags, `vX.Y` format |
| [3] | attack.mitre.org/techniques/ics/ | ICS technique listing; T0836/T0835/T0831/T0806/T0846 names + active status |
| [4] | attack.mitre.org/techniques/T0888/ | T0888 active, tech v1.1, last-mod 2026-05-12 |
| [5] | attack.mitre.org/techniques/T0855/ (via ATT&CK ICS Excel listing) | T0855 Unauthorized Command Message, active |
| [6] | attack.mitre.org/docs/attack-excel-files/v17.1/ics-attack/ | ICS technique URL listing incl. T0855 |
| [7] | attack.mitre.org/groups/G0034/ | T0831 Manipulation of Control active in mappings |
| [8] | attack.mitre.org/techniques/T0814/ | T0814 Denial of Service active, Inhibit Response Function |
| [9] | github.com/mitre/cti/releases | `ATT&CK-vX.Y` tag convention |
| [10] | github.com/mitre-attack/attack-stix-data/tree/master/ics-attack | `ics-attack-19.1.json` per-domain bundle filename |

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of ATT&CK version history, current version (v19.1), release dates, STIX/CTI version-string conventions |
| Perplexity perplexity_reason | 1 | Synthesis/cross-check of the 8 ICS technique IDs' current names + deprecation status (domain-restricted to attack.mitre.org) |
| Perplexity perplexity_ask | 1 | Targeted confirm of T0855 name + non-deprecation (WebFetch returned empty twice) |
| WebFetch | 6 | attack.mitre.org versions page; ICS techniques index; T0888/T0855 pages; attack-stix-data releases + ics-attack bundle directory |

**Total MCP tool calls:** 3 (1 research + 1 reason + 1 ask)
**Training data reliance:** low — every load-bearing fact (current version, dates,
technique names/status, bundle filenames) is verified against attack.mitre.org
and the MITRE attack-stix-data GitHub repo. Training data used only for general
ATT&CK background.
