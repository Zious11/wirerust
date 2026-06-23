# F5 — Authoritative ICS Technique → Tactic Mapping (MITRE ATT&CK for ICS v19 / ics-attack-19.1)

**Date:** 2026-06-23
**Agent:** vsdd-factory:research-agent
**Scope:** Page-verified mapping of every ICS MITRE ATT&CK technique in wirerust's catalog (`src/mitre.rs::technique_info`) to its correct ICS-matrix tactic and tactic TA-id.
**Pinned version:** MITRE ATT&CK for ICS v19 (ics-attack-19.1, released 2026-04-20).
**Authoritative source:** attack.mitre.org technique/tactic pages (each cited inline).

> This report does NOT modify code. It is the authoritative reference the F5 fix
> should apply against `src/mitre.rs`.

---

## Executive Summary

Two catalog entries are **WRONG** against authoritative MITRE ICS v19 pages and must be changed:

| ID | Name | Catalog tactic | CORRECT ICS tactic | Severity |
|----|------|----------------|--------------------|----------|
| **T0830** | Adversary-in-the-Middle | Lateral Movement (TA0008) | **Collection (TA0100)** | WRONG — fix required |
| **T0831** | Manipulation of Control | Impair Process Control (TA0106) | **Impact (TA0105)** | WRONG — fix required |

All other ICS techniques in the catalog have a tactic *name* consistent with MITRE, BUT note the
**TA-id layer is systematically wrong for ICS techniques that reuse Enterprise tactic names**
(Discovery, Command and Control, Lateral Movement, Collection). The catalog merges ICS techniques
into Enterprise `MitreTactic` variants, so `technique_tactic_id()` emits the **Enterprise** TA-id
(e.g. Discovery→`TA0007`) instead of the **ICS** TA-id (Discovery→`TA0102`). Whether that is a
defect depends on F5's intent (see "TA-id Layer Caveat" below). The two rows above are wrong at the
*tactic-name* level, which is unambiguous regardless of that design question.

ICS tactic TA-id reference (verified against https://attack.mitre.org/tactics/ics/):
Initial Access **TA0108** · Execution **TA0104** · Persistence **TA0110** · Privilege Escalation **TA0111** · Evasion **TA0103** · Discovery **TA0102** · Lateral Movement **TA0109** · Collection **TA0100** · Command and Control **TA0101** · Inhibit Response Function **TA0107** · Impair Process Control **TA0106** · Impact **TA0105**.
(This matches every anchor provided in the task brief exactly.)

---

## Step 1 — ICS techniques extracted from `src/mitre.rs::technique_info`

The catalog has 25 seeded IDs (12 Enterprise + 13 ICS-context). The ICS-context techniques —
all `T0xxx` plus the ICS sub-techniques `T1692.001/.002`, `T1691.001`, and the ICS-used
`T1557.002` — are:

| # | ID | Catalog name | Catalog `MitreTactic` variant | Catalog TA-id (`technique_tactic_id`) |
|---|----|--------------|-------------------------------|----------------------------------------|
| 1 | T0846 | Remote System Discovery | `Discovery` | TA0007 |
| 2 | T1692.001 | Unauthorized Message: Command Message | `IcsImpairProcessControl` | TA0106 |
| 3 | T1692.002 | Unauthorized Message: Reporting Message | `IcsImpairProcessControl` | TA0106 |
| 4 | T0885 | Commonly Used Port | `CommandAndControl` | TA0011 |
| 5 | T0836 | Modify Parameter | `IcsImpairProcessControl` | TA0106 |
| 6 | T0814 | Denial of Service | `IcsInhibitResponseFunction` | TA0107 |
| 7 | T0806 | Brute Force I/O | `IcsImpairProcessControl` | TA0106 |
| 8 | T0835 | Manipulate I/O Image | `IcsImpairProcessControl` | TA0106 |
| 9 | T0831 | Manipulation of Control | `IcsImpairProcessControl` | TA0106 |
| 10 | T0888 | Remote System Information Discovery | `Discovery` | TA0007 |
| 11 | T1691.001 | Block Operational Technology Message: Command Message | `IcsInhibitResponseFunction` | TA0107 |
| 12 | T0827 | Loss of Control | `IcsImpact` | TA0105 |
| 13 | T0830 | Adversary-in-the-Middle | `LateralMovement` | TA0008 |
| 14 | T1557.002 | Adversary-in-the-Middle: ARP Cache Poisoning | `CredentialAccess` | TA0006 |

Note: T1557.002 is an **Enterprise** sub-technique (Credential Access; Collection), seeded in the
ICS section because wirerust's ARP-spoof analyzer emits it alongside the ICS T0830. It is NOT an
ICS-matrix technique — see row 14 analysis.

---

## Step 2 — Authoritative ICS-matrix tactic for each technique

### MASTER TABLE

| ID | Name (catalog) | Current catalog tactic (name / TA-id) | CORRECT ICS tactic | CORRECT ICS TA-id | Source URL | CHANGE? |
|----|----------------|----------------------------------------|--------------------|-------------------|------------|---------|
| **T0846** | Remote System Discovery | Discovery / TA0007 | **Discovery** | **TA0102** | https://attack.mitre.org/techniques/T0846/ | name: NO · TA-id: see caveat |
| **T0888** | Remote System Information Discovery | Discovery / TA0007 | **Discovery** | **TA0102** | https://attack.mitre.org/techniques/T0888/ | name: NO · TA-id: see caveat |
| **T0885** | Commonly Used Port | Command and Control / TA0011 | **Command and Control** | **TA0101** | https://attack.mitre.org/techniques/T0885/ | name: NO · TA-id: see caveat |
| **T0836** | Modify Parameter | Impair Process Control / TA0106 | **Impair Process Control** | **TA0106** | https://attack.mitre.org/techniques/T0836/ | NO |
| **T0814** | Denial of Service | Inhibit Response Function / TA0107 | **Inhibit Response Function** | **TA0107** | https://attack.mitre.org/techniques/T0814/ | NO |
| **T0806** | Brute Force I/O | Impair Process Control / TA0106 | **Impair Process Control** | **TA0106** | https://attack.mitre.org/techniques/T0806/ | NO |
| **T0835** | Manipulate I/O Image | Impair Process Control / TA0106 | **Impair Process Control** | **TA0106** | https://attack.mitre.org/techniques/T0835/ | NO |
| **T0831** | Manipulation of Control | Impair Process Control / TA0106 | **Impact** | **TA0105** | https://attack.mitre.org/techniques/T0831/ | **YES — WRONG** |
| **T0827** | Loss of Control | Impact (ICS) / TA0105 | **Impact** | **TA0105** | https://attack.mitre.org/techniques/T0827/ | NO |
| **T0830** | Adversary-in-the-Middle | Lateral Movement / TA0008 | **Collection** | **TA0100** | https://attack.mitre.org/techniques/T0830/ | **YES — WRONG** |
| **T1692.001** | Unauthorized Message: Command Message | Impair Process Control / TA0106 | **Impair Process Control** (also Evasion) | **TA0106** (Evasion = TA0103) | https://attack.mitre.org/techniques/T1692/001/ | NO (catalog picks valid one) |
| **T1692.002** | Unauthorized Message: Reporting Message | Impair Process Control / TA0106 | **Impair Process Control** (also Evasion) | **TA0106** (Evasion = TA0103) | https://attack.mitre.org/techniques/T1692/002/ | NO (catalog picks valid one) |
| **T1691.001** | Block Operational Technology Message: Command Message | Inhibit Response Function / TA0107 | **Inhibit Response Function** | **TA0107** | https://attack.mitre.org/techniques/T1691/001/ | NO |
| **T1557.002** | Adversary-in-the-Middle: ARP Cache Poisoning | Credential Access / TA0006 | **(Enterprise only)** Credential Access + Collection | TA0006 / TA0009 (Enterprise) | https://attack.mitre.org/techniques/T1557/002/ | NO (Enterprise technique; not in ICS matrix) |

---

## Detailed findings per technique

### T0846 Remote System Discovery → Discovery / **TA0102**
Page directly verified (WebFetch): "Tactic: [Discovery](/tactics/TA0102)". Single tactic.
Catalog tactic NAME is correct; the canonical ICS TA-id is **TA0102** (catalog emits Enterprise
TA0007 via the merged `Discovery` variant — see caveat).
Source: https://attack.mitre.org/techniques/T0846/

### T0888 Remote System Information Discovery → Discovery / **TA0102**
Page directly verified (WebFetch): "Tactic: [Discovery](/tactics/TA0102)". Single tactic.
Confirms the task anchor (T0888→Discovery). Catalog NAME correct; ICS TA-id is TA0102.
Source: https://attack.mitre.org/techniques/T0888/

### T0885 Commonly Used Port → Command and Control / **TA0101**
Perplexity deep-research, page snippet "Tactic: Command and Control". Single tactic. Catalog NAME
correct; ICS TA-id is TA0101 (catalog emits Enterprise TA0011).
Source: https://attack.mitre.org/techniques/T0885/

### T0836 Modify Parameter → Impair Process Control / **TA0106** — CORRECT
Page snippet "Tactic: Impair Process Control". Catalog fully correct (name + TA-id both TA0106;
ICS Impair Process Control has no Enterprise namesake, so the merged variant happens to carry the
ICS TA-id).
Source: https://attack.mitre.org/techniques/T0836/

### T0814 Denial of Service → Inhibit Response Function / **TA0107** — CORRECT
Page snippet "Tactic: Inhibit Response Function". Catalog fully correct (TA0107).
Source: https://attack.mitre.org/techniques/T0814/

### T0806 Brute Force I/O → Impair Process Control / **TA0106** — CORRECT
Page snippet "Tactic: Impair Process Control". Catalog fully correct (TA0106).
Source: https://attack.mitre.org/techniques/T0806/

### T0835 Manipulate I/O Image → Impair Process Control / **TA0106** — CORRECT
Tactic field was truncated in the deep-research snippet, but the technique is listed under the
Impair Process Control column of the ICS matrix (https://attack.mitre.org/matrices/ics/), and the
description (altering the PLC process image) is a canonical Impair-Process-Control behavior.
Confidence: HIGH (matrix-confirmed; not a verbatim single-page tactic line). Catalog correct.
Source: https://attack.mitre.org/techniques/T0835/ · https://attack.mitre.org/matrices/ics/

### T0831 Manipulation of Control → **Impact** / **TA0105** — ❗ WRONG IN CATALOG
Page directly verified (WebFetch): "Tactic: [Impact](/tactics/TA0105)". Single tactic.
**Catalog currently assigns `IcsImpairProcessControl` (TA0106) — this is WRONG.** Correct ICS
tactic is **Impact / TA0105**. Note: in the catalog this maps to the existing `IcsImpact` variant
(already TA0105), the same variant T0827 uses.
Source: https://attack.mitre.org/techniques/T0831/

### T0827 Loss of Control → Impact / **TA0105** — CORRECT
Page snippet + cross-confirmed via CISA/DOE/NSA/FBI advisory. Catalog `IcsImpact` → TA0105 is
correct.
Source: https://attack.mitre.org/techniques/T0827/

### T0830 Adversary-in-the-Middle → **Collection** / **TA0100** — ❗ WRONG IN CATALOG
Page directly verified (WebFetch): "Tactic: [Collection](/tactics/TA0100)". Single tactic.
**Catalog currently assigns `LateralMovement` (TA0008) — this is WRONG.** Correct ICS tactic is
**Collection / TA0100**. This confirms the specific defect flagged in the task brief. The catalog
has no `Collection`-for-ICS variant distinct from Enterprise `Collection` (TA0009); the fix needs a
variant/mapping that emits the ICS Collection TA-id **TA0100** (see caveat — the same merge issue as
Discovery/C2).
Source: https://attack.mitre.org/techniques/T0830/

### T1692.001 / T1692.002 Unauthorized Message (Command / Reporting) → Evasion **+** Impair Process Control
MITRE assigns BOTH sub-techniques to **two** ICS tactics: **Evasion (TA0103)** and
**Impair Process Control (TA0106)**. The catalog stores one tactic per technique and uses
`IcsImpairProcessControl` (TA0106) — a **legitimate** one of the two MITRE tactics, so no change is
strictly required. If F5 prefers Evasion as the primary, that is also defensible; recommendation:
**keep Impair Process Control** (it is the process-effect tactic and matches the analyzer's intent).
These are the v19 replacements for the deprecated T0855/T0856 (issue #222 remap already reflected in
the catalog).
Sources: https://attack.mitre.org/techniques/T1692/001/ · https://attack.mitre.org/techniques/T1692/002/

### T1691.001 Block OT Message: Command Message → Inhibit Response Function / **TA0107** — CORRECT
Page verified via deep research: "Tactic: Inhibit Response Function" (single tactic). v19 replacement
for deprecated T0803. Catalog `IcsInhibitResponseFunction` → TA0107 is correct.
Source: https://attack.mitre.org/techniques/T1691/001/

### T1557.002 ARP Cache Poisoning → Enterprise (Credential Access + Collection) — NOT an ICS technique
This is an **Enterprise** sub-technique. Its page lists tactics **Credential Access** and
**Collection**, platforms Linux/Windows/macOS. It does NOT appear in the ICS matrix; there is no ICS
technique page for it. ICS AiTM is modeled by the separate ICS technique T0830. The catalog's
assignment of `CredentialAccess` (TA0006) is a valid Enterprise tactic for it. No change required at
the ICS layer. (If F5 wants the second Enterprise tactic, Collection/TA0009 is also valid — but the
catalog stores one tactic, and Credential Access is the primary listed.)
Source: https://attack.mitre.org/techniques/T1557/002/

---

## TA-id Layer Caveat (design decision for F5 — not a verified-fact question)

The catalog deliberately **merges ICS techniques into Enterprise `MitreTactic` variants by name**
(see the comment at `src/mitre.rs:145-148`: "we intentionally merge by name so a single grouped
report has one section per tactic name regardless of source matrix"). Consequence: for the four ICS
tactics whose NAMES collide with Enterprise tactics, `technique_tactic_id()` returns the
**Enterprise** TA-id, not the **ICS** TA-id:

| Tactic name | ICS TA-id (correct for ICS techniques) | Enterprise TA-id (what catalog emits) | Affected ICS techniques |
|-------------|----------------------------------------|----------------------------------------|--------------------------|
| Discovery | **TA0102** | TA0007 | T0846, T0888 |
| Command and Control | **TA0101** | TA0011 | T0885 |
| Lateral Movement | **TA0109** | TA0008 | (none after T0830 fix) |
| Collection | **TA0100** | TA0009 | T0830 (after fix) |

The ICS-unique tactics (Inhibit Response Function TA0107, Impair Process Control TA0106, ICS Impact
TA0105) have no Enterprise namesake, so their TA-ids are already correct in the catalog.

**This is a JUDGMENT CALL for F5, not a correctness bug per se:** if the design intent is "one
section per tactic NAME regardless of matrix," the Enterprise TA-id is an arbitrary-but-consistent
choice. If the intent of the F5 "MITRE JSON names" work is to emit the *authoritative ICS-matrix
TA-id* for ICS techniques, then T0846/T0888/T0885 (and post-fix T0830) need ICS-specific TA-ids,
which requires new `MitreTactic` variants (e.g. `IcsDiscovery`, `IcsCommandAndControl`,
`IcsCollection`) or a per-technique TA-id override. **Recommendation:** confirm F5's intent before
touching the TA-id layer; the two tactic-NAME errors (T0830, T0831) are unconditional fixes
regardless.

---

## Required code changes (for the F5 implementer — informational only)

1. **T0830** — change `MitreTactic::LateralMovement` → a Collection-for-ICS assignment that yields
   ICS TA0100 (NOT Enterprise `Collection`/TA0009). Likely needs a new `IcsCollection` variant.
2. **T0831** — change `MitreTactic::IcsImpairProcessControl` → `MitreTactic::IcsImpact` (already
   TA0105). One-line variant swap; reuses existing `IcsImpact`.
3. (Conditional, pending F5 intent) — ICS TA-id correctness for T0846/T0888 (Discovery→TA0102),
   T0885 (C2→TA0101).

These changes will require updating the Kani proofs / `SEEDED_TECHNIQUE_IDS` only if variants are
added (count stays 25; IDs unchanged). The `vp007_catalog_drift_guard` test enforces the
`MitreTactic`→TA-id table stays exhaustive, so any new variant must be added to
`technique_tactic_id()`.

---

## Inconclusive / lower-confidence items

| Item | Status | Note |
|------|--------|------|
| T0835 verbatim tactic line | MEDIUM-HIGH | Tactic field truncated in research snippet; confirmed via ICS matrix column (Impair Process Control) + description. Not a single-page verbatim quote. To make HIGH: fetch https://attack.mitre.org/techniques/T0835/ directly. |

Everything else in the master table is HIGH confidence (verbatim tactic line read from the MITRE
technique page, either via direct WebFetch or deep-research snippet quoting the page's "Tactic:" field).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) ICS tactic assignment for T0846/T0885/T0836/T0814/T0806/T0835/T0831/T0888/T0827/T0830 against attack.mitre.org v19 pages; (2) tactic assignment for T1692/.001/.002, T1691/.001, and T1557.002 (Enterprise vs ICS) — both at `reasoning_effort: high`. |
| WebFetch | 5 | Direct verbatim verification of T0830 (→Collection/TA0100), T0831 (→Impact/TA0105), T0888 + T0846 (→Discovery/TA0102), and the full ICS tactics→TA-id table (https://attack.mitre.org/tactics/ics/). |
| Read | 1 | `src/mitre.rs` — extracted the full `technique_info` catalog and current tactic/TA-id assignments. |
| Training data | 0 areas | All tactic assignments sourced from MITRE pages; none from model knowledge. |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated primary tool) + 5 WebFetch verifications.
**Training data reliance:** low — every tactic-name and TA-id claim is grounded in an attack.mitre.org page cited inline; the two discrepancy findings (T0830, T0831) and all TA-id anchors were independently re-verified with direct WebFetch against the live MITRE pages.
