# Independent Adversarial Verification: MITRE ATT&CK for ICS Technique Status

- **Date:** 2026-06-10
- **Type:** general (adversarial verification / release-safety)
- **Method:** Independent confirm-or-refute against MITRE primary sources. Default verdict was "could NOT confirm revocation" unless authoritative evidence forced otherwise.
- **Subject:** Prior research pass claimed several ATT&CK-for-ICS techniques were revoked; this contradicts the project's own pinned research (`attack-ics-version-pin.md`, F4-PIN) that shipped in v0.3.0 and v0.4.0.
- **Outcome:** The prior pass is **CORRECT**. The project's own pinned research (F4-PIN) is **WRONG** on T0855. **The shipped v0.4.0 Modbus analyzer emits at least one REVOKED technique ID (T0855).** This is a release-safety finding.

> Note on subject naming: the task and output filename reference "dnp3" but the
> affected analyzer is the **Modbus** analyzer (per the task context: "shipped
> v0.4.0 Modbus analyzer EMITS T0855"). Findings below concern the ICS technique
> IDs regardless of which protocol analyzer emits them.

---

## Decisive primary source

The single most authoritative artifact is the **official ATT&CK v19 release notes**
(`https://attack.mitre.org/resources/updates/`, v19.0 release dated 2026-04-28).
Its **ICS → Revocations** section lists, **verbatim**:

```
#### Revocations
- Block Command Message (revoked by Block Operational Technology Message: Command Message) (v1.1)
- Block Reporting Message (revoked by Block Operational Technology Message: Reporting Message) (v1.0)
- Block Serial COM (revoked by Block Communications: Serial COM) (v1.1)
- Default Credentials (revoked by Insecure Credentials: Default Credentials) (v1.0)
- Hardcoded Credentials (revoked by Insecure Credentials: Hardcoded Credentials) (v1.0)
- Module Firmware (revoked by Modify Firmware: Module Firmware) (v1.1)
- Spoof Reporting Message (revoked by Unauthorized Message: Reporting Message) (v1.2)
- System Firmware (revoked by Modify Firmware: System Firmware) (v1.1)
- Unauthorized Command Message (revoked by Unauthorized Message: Command Message) (v1.2)
```

This is MITRE's own machine-generated changelog for the release the project pins
(`ics-attack-19.1`). It explicitly revokes both T0855 and T0803. There is no
ambiguity to resolve in the project's favor.

---

## Verdict table

| # | Item | Prior-pass claim | Verdict | Correct status (v19.1) |
|---|------|------------------|---------|------------------------|
| 1 | **T0855** Unauthorized Command Message | REVOKED → T1692.001 | **CONFIRMED** (prior pass right) | **REVOKED** in v19.0 by **T1692.001** "Unauthorized Message: Command Message" |
| 2 | **T0803** Block Command Message | REVOKED → T1691.001 | **CONFIRMED** (prior pass right) | **REVOKED** in v19.0 by **T1691.001** "Block Operational Technology Message: Command Message" |
| 3 | **T0828** name | = "Loss of Productivity and Revenue", NOT "Loss of Control" | **CONFIRMED** (prior pass right) | **Active**, name = "Loss of Productivity and Revenue" |
| 4 | **T0827** name | = "Loss of Control" | **CONFIRMED** (prior pass right) | **Active**, name = "Loss of Control" |
| 5 | **T0814** + **T0836** | both still active/unchanged | **CONFIRMED** | Both **active**, bare T0xxx IDs, not revoked/deprecated |
| 6 | **ics-attack-19.1** version reality | (project's own pin) | **VALID** | v19.1 is the current unified release; ICS shares the number; `ics-attack-19.1` bundle is real |

**Bottom line: the prior pass that claimed the revocations was correct on every
checkable point (6/6). The project's F4-PIN research was wrong specifically about
T0855 (and is now stale w.r.t. T0803 naming convention too).**

---

## Per-item evidence

### 1. T0855 "Unauthorized Command Message" — CONFIRMED REVOKED
- **Verdict:** CONFIRMED (prior pass right). REVOKED in ATT&CK v19.0.
- **Successor:** **T1692.001** "Command Message" (sub-technique of **T1692** "Unauthorized Message"). ICS-domain.
- **Version landed:** v19.0 (2026-04-28); successor sub-techniques created 2026-04-20.
- **Primary source:** `https://attack.mitre.org/resources/updates/` (v19 release notes, ICS → Revocations)
  - Verbatim: *"Unauthorized Command Message (revoked by Unauthorized Message: Command Message) (v1.2)"*
- **Successor page verified live:** `https://attack.mitre.org/techniques/T1692/001/` — "Command Message (T1692.001)", parent T1692 "Unauthorized Message", ICS-domain, real content, last-modified 2026-05-12.
- **Caveat (why F4-PIN got fooled):** The legacy page `https://attack.mitre.org/techniques/T0855/` still renders (Version 1.2, last-modified 16 April 2025, "No sub-techniques") because MITRE keeps revoked technique pages online with a redirect banner. A reader who only looked at the still-serving T0855 page — and missed the v19 changelog — would wrongly conclude it is active. That is exactly the F4-PIN failure mode.

### 2. T0803 "Block Command Message" — CONFIRMED REVOKED
- **Verdict:** CONFIRMED (prior pass right). REVOKED in ATT&CK v19.0.
- **Successor:** **T1691.001** "Block Operational Technology Message: Command Message" (sub-technique of **T1691** "Block Operational Technology Message"). ICS-domain.
- **Version landed:** v19.0 (2026-04-28); successor created 2026-04-20.
- **Primary source:** `https://attack.mitre.org/resources/updates/` (v19 release notes, ICS → Revocations)
  - Verbatim: *"Block Command Message (revoked by Block Operational Technology Message: Command Message) (v1.1)"*
- **Successor page verified live:** `https://attack.mitre.org/techniques/T1691/001/` — confirmed ICS-domain, parent T1691, last-modified 2026-05-12.

### 3. T0828 — CONFIRMED "Loss of Productivity and Revenue", active
- **Verdict:** CONFIRMED (prior pass right).
- **Primary source:** `https://attack.mitre.org/techniques/T0828/`
  - Name on page: *"Loss of Productivity and Revenue"*; ID T0828; no deprecation/revocation banner; created 2020-05-21, last-modified 2025-04-16; Impact tactic.
- It is **not** "Loss of Control" — confirming the prior pass's correction.

### 4. T0827 — CONFIRMED "Loss of Control", active
- **Verdict:** CONFIRMED (prior pass right).
- **Primary source:** `https://attack.mitre.org/techniques/T0827/`
  - Name on page: *"Loss of Control"*; active; last-modified 2026-05-12; no deprecation/revocation banner.

### 5. T0814 + T0836 — CONFIRMED both active, unchanged
- **Verdict:** CONFIRMED.
- **Primary source:** ICS technique index `https://attack.mitre.org/techniques/ics/`
  - *"T0814 | Denial of Service | Adversaries may perform Denial-of-Service (DoS) attacks…"* — present with bare T0814 ID, full active description.
  - *"T0836 | Modify Parameter | Adversaries may modify parameters used to instruct industrial control system devices…"* — present with bare T0836 ID, full active description.
  - Neither appears in the v19 ICS Revocations or Deprecations list (which was read in full — see item-1 source block; T0814/T0836 are absent from it).

### 6. Version reality check — ics-attack-19.1 is VALID
- **Verdict:** VALID (no concern). v19.x for ICS is real.
- **Primary source:** `https://attack.mitre.org/resources/versions/`
  - Current release: **ATT&CK v19.1, released 2026-04-28**. *"The overall ATT&CK catalog is versioned using a `major.minor` version schema."*
  - **ICS is versioned together with Enterprise and Mobile under one unified number.** There is no separate, lower ICS version stream. So an ICS matrix "v19.x" absolutely exists — the old assumption that ICS uses low version numbers is stale (that was true years ago; ICS folded into the unified release cadence).
  - The per-domain STIX bundle `ics-attack-19.1.json` is the canonical name, so the pin string `ics-attack-19.1` is well-formed and points at a real, current artifact.
- **Important nuance:** the *version pin* is valid, but pinning to v19.1 is precisely what makes the T0855 emission a defect — under v19.1, T0855 is revoked. The pin is correct; the emitted ID set was not updated to match it.

---

## Domain-ID disambiguation (the "common confusion point")

The task flagged a risk that claimed successors T1691.001 / T1692.001 might be
Enterprise-domain IDs misattributed to ICS. **Verified they are genuinely
ICS-domain:**

- **T1692** "Unauthorized Message" and **T1691** "Block Operational Technology
  Message" are **new ICS-domain parent techniques** introduced in v19 (listed
  under "ICS → New Techniques" in the v19 release notes, and present on the live
  ICS techniques index `https://attack.mitre.org/techniques/ics/`).
- Their sub-technique pages (`/techniques/T1692/001/`, `/techniques/T1691/001/`)
  self-identify as **ICS** and were independently fetched.
- v19 is the release that **introduced sub-techniques to the ICS matrix for the
  first time** (18 new ICS sub-techniques), which is why the successors carry the
  `Txxxx.NNN` shape previously associated only with Enterprise. The numbering
  collides visually with Enterprise IDs but the objects are ICS-domain. No
  misattribution.

---

## Bottom line

**(a) Is the project's `ics-attack-19.1` pin valid?**
YES — v19.1 is the current unified ATT&CK release (2026-04-28) and the
`ics-attack-19.1` bundle is the canonical ICS STIX bundle. The pin string is
correct. The defect is not the pin; it is that the emitted technique-ID set was
not reconciled against what v19.1 actually contains.

**(b) Does shipped v0.4.0 emit any revoked/deprecated ID?**
**YES — this is a confirmed release defect.** v0.4.0's Modbus analyzer emits
**T0855**, which is **REVOKED** under the very version (v19.1) the report envelope
pins. Any other emitter of **T0803** (Block Command Message) would be in the same
state. A released binary advertising `mitre_attack_version = "ics-attack-19.1"`
while emitting `T0855` is internally inconsistent: it claims conformance to a
matrix version in which that ID no longer exists as an active technique.

> Also note: the project's F4-PIN doc (`attack-ics-version-pin.md`) itself lists
> T0855 and several others as "valid and active in v19.1." That conclusion is
> **incorrect for T0855** and should be treated as superseded by this
> verification. (T0803 was not in F4-PIN's emitted-ID list, but the same v19
> revocation applies to it.)

**(c) Correct, currently-valid technique ID for each:**

| Concept | Legacy ID (v0.4.0 emits / referenced) | Correct v19.1 ID | Correct v19.1 name |
|---------|----------------------------------------|------------------|--------------------|
| Unauthorized command | T0855 (REVOKED) | **T1692.001** | Unauthorized Message: Command Message |
| Block command | T0803 (REVOKED) | **T1691.001** | Block Operational Technology Message: Command Message |
| Loss of control | — | **T0827** | Loss of Control (active, unchanged — T0828 is the *different* "Loss of Productivity and Revenue") |

---

## Recommended follow-up (for the orchestrator — not actioned here)

1. **Treat as a release-safety bug, candidate for GitHub issue** (subject to the
   `DF-VALIDATION-001` policy gate in `.factory/policies.yaml` — this report is
   the research-agent validation that policy requires before filing).
   - Map T0855 → T1692.001 and any T0803 → T1691.001 in the analyzer's emitted-ID
     table; bump/patch and note in CHANGELOG that the prior IDs were revoked
     under ATT&CK v19.
2. **Correct the stale F4-PIN doc** `attack-ics-version-pin.md`: its §2 table
   asserting T0855 active in v19.1 is wrong. The version-string recommendation
   (`ics-attack-19.1`) remains correct.
3. **Re-audit the full emitted-ID set** against the v19 ICS Revocations list
   (quoted above) — the project also referenced T0888, T0836, T0835, T0831,
   T0814, T0806, T0846. Of these, none appear in the v19 revocation list per the
   sources read, but a one-pass reconciliation against
   `https://attack.mitre.org/techniques/ics/` is cheap insurance before the next
   release. (T0846 Remote System Discovery gained sub-techniques in v19 but the
   bare parent remains valid.)

---

## Sources (all primary, attack.mitre.org)

| # | URL | Used for |
|---|-----|----------|
| 1 | https://attack.mitre.org/resources/updates/ | **Decisive** v19 ICS Revocations list (verbatim T0855 + T0803 revocation lines) |
| 2 | https://attack.mitre.org/resources/updates/updates-april-2026/ | v19 (April 2026) release notes ICS section — full revocation enumeration |
| 3 | https://attack.mitre.org/techniques/T1692/001/ | Successor for T0855 verified live, ICS-domain, parent T1692 |
| 4 | https://attack.mitre.org/techniques/T1691/001/ | Successor for T0803 verified live, ICS-domain, parent T1691 |
| 5 | https://attack.mitre.org/techniques/T0827/ | T0827 = "Loss of Control", active |
| 6 | https://attack.mitre.org/techniques/T0828/ | T0828 = "Loss of Productivity and Revenue", active |
| 7 | https://attack.mitre.org/techniques/ics/ | T0814, T0836 active in current ICS index; T1691/T1692 parents listed as ICS |
| 8 | https://attack.mitre.org/resources/versions/ | v19.1 current (2026-04-28); unified ICS+Enterprise+Mobile versioning |
| 9 | https://attack.mitre.org/techniques/T0855/ | Legacy T0855 page still renders (explains F4-PIN error); Version 1.2, last-mod 2025-04-16 |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of all 6 items; surfaced the candidate revocations and successor IDs (treated as leads, then independently verified against primary sources — NOT taken at face value) |
| Perplexity perplexity_search | 4 | Pulled exact verbatim text from attack.mitre.org pages that WebFetch could not render: the v19 Revocations list (decisive), T0855 page state, T0814/T0836 active index entries |
| WebFetch | 10 (6 empty / 4 substantive) | Direct fetch of T0827, T0828, T1692/001, T1691/001, v19 April-2026 updates, versions page. NOTE: 6 fetches of JS-rendered MITRE technique pages (T0855, T0803, and retries) returned EMPTY content — these were backfilled via perplexity_search of the same canonical URLs. |
| Training data | 0 load-bearing areas | No version numbers, names, or statuses taken from training data. Every verdict traces to a cited attack.mitre.org URL. |

**Total MCP tool calls:** 5 (1 research + 4 search)
**Training data reliance:** low — every load-bearing fact (revocation status, successor IDs, technique names, version/date) is cited to an attack.mitre.org primary source. The deep-research synthesis was used only to generate leads, which were then independently confirmed against MITRE's own changelog and technique pages.

**Tooling caveat:** WebFetch consistently returned empty bodies for MITRE's
JavaScript-rendered `/techniques/TXXXX/` pages (6 empty results). The decisive
evidence (the verbatim v19 ICS Revocations list) was instead obtained via
`perplexity_search` against the canonical `attack.mitre.org/resources/updates/`
URL, and cross-checked against the `updates-april-2026/` page via WebFetch which
DID render. The two independent paths agree exactly, so the empty WebFetch
results did not weaken the conclusion.
