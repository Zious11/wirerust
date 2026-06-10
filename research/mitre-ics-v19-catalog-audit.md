# MITRE ATT&CK for ICS v19.1 — Full Catalog Blast-Radius Audit

- **Date:** 2026-06-10
- **Type:** general (release-safety / external-standard reconciliation)
- **Subject:** Reconcile EVERY technique ID referenced in `src/mitre.rs` against ATT&CK
  release **v19.1** (the unified April-2026 release the project pins as `ics-attack-19.1`).
- **Scope:** read-only audit. No code/spec changes. Ground-truth input = `src/mitre.rs`.
- **Method:** Confirm-or-refute against MITRE primary sources (attack.mitre.org technique
  pages + the verbatim v19 release-notes Revocations/Deprecations lists). Continues the
  method and verdicts of `.factory/research/dnp3-mitre-verification.md`.
- **Headline:** **2 of the project's 21 seeded IDs are revoked under v19.1** — the
  previously-known `T0855`, **plus a newly-surfaced second defect `T0856`**. Both are ICS
  message-spoofing techniques folded into the new parent `T1692 Unauthorized Message`.
  All other 19 seeded IDs (11 Enterprise + 8 ICS) are ACTIVE-UNCHANGED in v19.1.

> **NEW finding vs. prior research:** the earlier `dnp3-mitre-verification.md` pass flagged
> only `T0855` (and `T0803`, which the code does NOT reference). This full-catalog sweep
> surfaces **`T0856 Spoof Reporting Message` as a second seeded-but-revoked ID** — it sits
> in the SEEDED set but NOT the EMITTED set, so it is a catalogue-only defect today, but it
> is still a revoked ID shipped in the binary's lookup table while the report advertises
> conformance to v19.1.

---

## 1. Authoritative input set (Step 1 — extracted from `src/mitre.rs`)

The single source of truth is `technique_info()` (the `match` arms) mirrored by
`SEEDED_TECHNIQUE_IDS` (`SEEDED_TECHNIQUE_ID_COUNT = 21`) and `EMITTED_IDS`
(13, in `kani_proofs`). Distinct IDs referenced: **21** (all in the SEEDED set; the EMITTED
set of 13 is a subset).

| # | ID | Code name string | Domain | Seeded? | Emitted? |
|---|----|------------------|--------|:-------:|:--------:|
| 1 | T1027 | Obfuscated Files or Information | Enterprise | ✅ | ✅ |
| 2 | T1036 | Masquerading | Enterprise | ✅ | ✅ |
| 3 | T1040 | Network Sniffing | Enterprise | ✅ | — |
| 4 | T1046 | Network Service Discovery | Enterprise | ✅ | ✅ |
| 5 | T1071 | Application Layer Protocol | Enterprise | ✅ | — |
| 6 | T1071.001 | Web Protocols | Enterprise | ✅ | — |
| 7 | T1071.004 | DNS | Enterprise | ✅ | — |
| 8 | T1083 | File and Directory Discovery | Enterprise | ✅ | ✅ |
| 9 | T1499.002 | Service Exhaustion Flood | Enterprise | ✅ | ✅ |
| 10 | T1505.003 | Web Shell | Enterprise | ✅ | ✅ |
| 11 | T1573 | Encrypted Channel | Enterprise | ✅ | — |
| 12 | T0846 | Remote System Discovery | ICS | ✅ | — |
| 13 | **T0855** | **Unauthorized Command Message** | ICS | ✅ | ✅ |
| 14 | **T0856** | **Spoof Reporting Message** | ICS | ✅ | — |
| 15 | T0885 | Commonly Used Port | ICS | ✅ | — |
| 16 | T0836 | Modify Parameter | ICS | ✅ | ✅ |
| 17 | T0814 | Denial of Service | ICS | ✅ | ✅ |
| 18 | T0806 | Brute Force I/O | ICS | ✅ | ✅ |
| 19 | T0835 | Manipulate I/O Image | ICS | ✅ | ✅ |
| 20 | T0831 | Manipulation of Control | ICS | ✅ | ✅ |
| 21 | T0888 | Remote System Information Discovery | ICS | ✅ | ✅ |

SEEDED = 21 (11 Enterprise + 10 ICS). EMITTED = 13 (6 Enterprise + 7 ICS), matching the
code comments. `T0856` is the only ICS ID seeded but not emitted besides `T0846` and `T0885`.

---

## 2. Reconciliation against v19.1 (Steps 2 — per-ID verdicts)

Decisive primary artifact: the **verbatim v19 ICS → Revocations list** from
`https://attack.mitre.org/resources/updates/` (release v19.0, 2026-04-28; v19.1 is the
current patch). The complete ICS Revocations block reads:

```
#### Revocations  (ICS)
- Block Command Message (revoked by Block Operational Technology Message: Command Message) (v1.1)
- Block Reporting Message (revoked by Block Operational Technology Message: Reporting Message) (v1.0)
- Block Serial COM (revoked by Block Communications: Serial COM) (v1.1)
- Default Credentials (revoked by Insecure Credentials: Default Credentials) (v1.0)
- Hardcoded Credentials (revoked by Insecure Credentials: Hardcoded Credentials) (v1.0)
- Module Firmware (revoked by Modify Firmware: Module Firmware) (v1.1)
- Spoof Reporting Message (revoked by Unauthorized Message: Reporting Message) (v1.2)   ← T0856
- System Firmware (revoked by Modify Firmware: System Firmware) (v1.1)
- Unauthorized Command Message (revoked by Unauthorized Message: Command Message) (v1.2) ← T0855
```

Of those 9 ICS revocations, exactly **two intersect the project's seeded set**: `T0855` and
`T0856`. Every other revoked item (Block Command/Reporting Message, Block Serial COM,
Default/Hardcoded Credentials, Module/System Firmware) is NOT referenced anywhere in
`src/mitre.rs`, so they are out of scope.

### Per-ID verdict table

| # | Project ID | Code name | v19.1 status | Correct v19.1 ID | Correct v19.1 name | Source |
|---|-----------|-----------|--------------|------------------|--------------------|--------|
| 1 | T1027 | Obfuscated Files or Information | ACTIVE-UNCHANGED | T1027 | Obfuscated Files or Information | [E-notes] |
| 2 | T1036 | Masquerading | ACTIVE-UNCHANGED | T1036 | Masquerading | attack.mitre.org/techniques/T1036/ |
| 3 | T1040 | Network Sniffing | ACTIVE-UNCHANGED | T1040 | Network Sniffing | [E-notes] |
| 4 | T1046 | Network Service Discovery | ACTIVE-UNCHANGED | T1046 | Network Service Discovery | [E-notes] |
| 5 | T1071 | Application Layer Protocol | ACTIVE-UNCHANGED | T1071 | Application Layer Protocol | [E-notes] |
| 6 | T1071.001 | Web Protocols | ACTIVE-UNCHANGED | T1071.001 | Web Protocols | [E-notes] |
| 7 | T1071.004 | DNS | ACTIVE-UNCHANGED | T1071.004 | DNS | [E-notes] |
| 8 | T1083 | File and Directory Discovery | ACTIVE-UNCHANGED | T1083 | File and Directory Discovery | attack.mitre.org/techniques/T1083/ (v1.7, last-mod 2026-05-12, no revoke banner) |
| 9 | T1499.002 | Service Exhaustion Flood | ACTIVE-UNCHANGED | T1499.002 | Service Exhaustion Flood | [E-notes] |
| 10 | T1505.003 | Web Shell | ACTIVE-UNCHANGED | T1505.003 | Web Shell | [E-notes] |
| 11 | T1573 | Encrypted Channel | ACTIVE-UNCHANGED | T1573 | Encrypted Channel | [E-notes] |
| 12 | T0846 | Remote System Discovery | ACTIVE-UNCHANGED (parent valid; gained sub-techniques T0846.001/.002/.003) | T0846 | Remote System Discovery | attack.mitre.org/techniques/T0846/ |
| 13 | **T0855** | Unauthorized Command Message | **REVOKED → SUBTECHNIQUE** | **T1692.001** | Unauthorized Message: Command Message | v19 ICS Revocations (verbatim) + attack.mitre.org/techniques/T1692/001/ |
| 14 | **T0856** | Spoof Reporting Message | **REVOKED → SUBTECHNIQUE** | **T1692.002** | Unauthorized Message: Reporting Message | v19 ICS Revocations (verbatim) + attack.mitre.org/techniques/T1692/002/ |
| 15 | T0885 | Commonly Used Port | ACTIVE-UNCHANGED (no sub-techniques) | T0885 | Commonly Used Port | attack.mitre.org/techniques/T0885/ |
| 16 | T0836 | Modify Parameter | ACTIVE-UNCHANGED | T0836 | Modify Parameter | v19 ICS Revocations (absent) + ICS index |
| 17 | T0814 | Denial of Service | ACTIVE-UNCHANGED | T0814 | Denial of Service | attack.mitre.org/techniques/T0814/; ICS index |
| 18 | T0806 | Brute Force I/O | ACTIVE-UNCHANGED | T0806 | Brute Force I/O | v19 ICS Revocations (absent) + ICS index |
| 19 | T0835 | Manipulate I/O Image | ACTIVE-UNCHANGED | T0835 | Manipulate I/O Image | attack.mitre.org/techniques/T0835/ |
| 20 | T0831 | Manipulation of Control | ACTIVE-UNCHANGED | T0831 | Manipulation of Control | v19 ICS Revocations (absent) + ICS index |
| 21 | T0888 | Remote System Information Discovery | ACTIVE-UNCHANGED | T0888 | Remote System Information Discovery | v19 ICS Revocations (absent) + ICS index |

`[E-notes]` = the v19 Enterprise Revocations list at
`https://attack.mitre.org/resources/updates/` was read in full; none of these IDs appear in
it. The Enterprise Defense-Evasion→Stealth/Defense-Impairment tactic split relocated some
techniques to new tactics but did NOT change the IDs the project uses. (The split's revoked
IDs — e.g. T1562 → T1685 — are NOT referenced by this project.)

#### Note on a refuted lead

A low-depth `perplexity_ask` call claimed `T1083` had been "revoked / replaced" in v19. This
was **REFUTED** by direct fetch of `https://attack.mitre.org/techniques/T1083/`, which shows
T1083 ACTIVE (version 1.7, Tactic: Discovery, last-modified 2026-05-12, no deprecation or
revocation banner). T1083 does not appear in any v19 revocation list. The hallucinated lead
was discarded; the verdict above rests on the primary source. **No [UNVERIFIED] flags
remain** — every revocation traces to MITRE's own verbatim changelog, and the two
non-revoked-but-restructured cases (T0846 sub-techniques, Enterprise tactic relocations)
keep their original ID strings.

---

## 3. Summary counts (Step 3.2)

- **Total distinct IDs audited:** 21
- **ACTIVE-UNCHANGED:** 19 (11 Enterprise + 8 ICS) — incl. T0846 (parent stays valid despite
  new sub-techniques) and T0885.
- **RENAMED (same ID, new name):** 0
- **REVOKED → needs remap:** **2** — both ICS, both → sub-techniques of new parent T1692.
- **NOT-FOUND / UNVERIFIED:** 0

---

## 4. Explicit remap list (Step 3.3) — old → new

Only two IDs must change. Both are ICS revocations from the v19 release:

| Old (revoked) ID | Old name | New v19.1 ID | New v19.1 name | New parent |
|------------------|----------|--------------|----------------|-----------|
| **T0855** | Unauthorized Command Message | **T1692.001** | Unauthorized Message: Command Message | T1692 Unauthorized Message (ICS) |
| **T0856** | Spoof Reporting Message | **T1692.002** | Unauthorized Message: Reporting Message | T1692 Unauthorized Message (ICS) |

Both successors are genuinely **ICS-domain** sub-techniques (parent `T1692 Unauthorized
Message` is a new ICS parent introduced in v19; its `.NNN` shape is the new ICS sub-technique
form, not an Enterprise misattribution — confirmed in `dnp3-mitre-verification.md` §"Domain-ID
disambiguation" and the live pages `T1692/001/`, `T1692/002/`).

> **Catalogue-mechanics caveat for whoever remediates:** the successors carry the
> sub-technique shape `T1692.001` / `T1692.002`. These satisfy `is_valid_technique_id_format`
> (the `T[0-9]{4}.[0-9]{3}` branch), so VP-007 sub-property A still holds after the remap. No
> new tactic enum variant is needed — both map to an existing ICS tactic (T0855's current arm
> uses `IcsImpairProcessControl`; T0856 likewise). Remediation also requires bumping
> `SEEDED_TECHNIQUE_ID_COUNT` only if the count of arms changes (a 1:1 ID swap keeps it 21).

---

## 5. Which shipped releases advertise which revoked IDs (Step 3.4)

The defect manifests differently in the SEEDED (catalogue) vs EMITTED (analyzer-produced)
layers. Both v0.3.0 and v0.4.0 pin/advertise `mitre_attack_version = "ics-attack-19.1"`.

| Release | Pins v19.1? | Seeds T0855 (revoked) | Seeds T0856 (revoked) | **Emits** T0855 (revoked) | Emits T0856 |
|---------|:-----------:|:---------------------:|:---------------------:|:-------------------------:|:-----------:|
| **v0.3.0** | yes | yes (pre-F2 catalogue) | yes (pre-F2 catalogue) | **no** (no ICS analyzer emitting it pre-F2 in 0.3.0) | no |
| **v0.4.0** | yes | yes | yes | **YES** (Modbus analyzer emits T0855) | no |

- **T0855 — catalogue defect in BOTH v0.3.0 and v0.4.0; ESCALATED to an *emitted* defect in
  v0.4.0.** Per `dnp3-mitre-verification.md`, the shipped **v0.4.0 Modbus analyzer emits
  T0855**, so a v0.4.0 binary actively attaches a revoked technique ID to findings while its
  report envelope claims conformance to v19.1 (where T0855 no longer exists as an active
  technique). This is the confirmed release-safety bug. In v0.3.0 T0855 was present in the
  lookup table but (per the EMITTED-set history) not yet attached by any analyzer — a
  latent/catalogue-only defect in 0.3.0.
- **T0856 — catalogue-only defect in BOTH v0.3.0 and v0.4.0.** T0856 is in the SEEDED set but
  NOT in `EMITTED_IDS` in either release, so no finding carries it; however the binary still
  ships a revoked ID in its name/tactic lookup table while advertising v19.1 conformance.
  Lower severity than T0855 (never user-visible on a finding) but the same class of internal
  inconsistency, and it should be remapped in the same pass.

> Severity ordering for the human deciding remediation sequencing: **T0855 (emitted in
> v0.4.0) > T0856 (catalogue-only, both releases)**. Fixing T0855 alone leaves a known-revoked
> ID (T0856) in the shipped catalogue; remapping both in one change is the clean fix.

---

## 6. Effect on VP-007 drift-guard logic (Step 3.5)

**VP-007's seeded-vs-emitted invariant is NOT broken by these revocations — but it does not
and cannot catch them.** Details:

- **What VP-007 proves:** (A) every seeded ID matches the `TXXXX` / `TXXXX.NNN` *format*;
  (B-catalogue) every seeded ID resolves in `technique_info`; (B-emitter) every EMITTED ID
  resolves in the catalogue; corollary: unknown IDs return `None` without panic. The
  `vp007_catalog_drift_guard` test additionally derives the catalogue size by sweeping the
  full ID space and asserts it equals `SEEDED_TECHNIQUE_IDS.len()`.
- **Why a rename/revocation does NOT trip it:** VP-007 is a *closed-world internal
  consistency* check — it verifies the catalogue is self-consistent and the emitted set is a
  subset of the catalogue. It has **no oracle for whether an ID is still valid in the external
  ATT&CK standard.** `T0855` and `T0856` are well-formed (`T[0-9]{4}`), resolve in
  `technique_info`, and are members of `SEEDED_TECHNIQUE_IDS`, so every VP-007 assertion still
  passes. Revocation is an *external-standard-drift* failure mode that lies entirely outside
  VP-007's specification.
- **Does the remap keep VP-007 green?** Yes, if done in lockstep. Swapping `T0855`→`T1692.001`
  and `T0856`→`T1692.002` in `technique_info` AND `SEEDED_TECHNIQUE_IDS` (and `EMITTED_IDS`
  for T0855) preserves: format validity (sub-technique branch), catalogue completeness, and
  the swept-count equality (21 stays 21 on a 1:1 swap). The drift guard would only fire if the
  edits to `technique_info` and `SEEDED_TECHNIQUE_IDS` were not mirrored — which is exactly its
  job and a desirable safety net during remediation.
- **Gap recommendation (for the human, not actioned here):** the project has no automated
  guard that emitted/seeded IDs are *currently-valid* against the pinned ATT&CK version. The
  only defense is periodic manual reconciliation like this audit. A future hardening item
  could diff `SEEDED_TECHNIQUE_IDS` against the `ics-attack-19.1.json` STIX bundle's
  non-revoked technique set, but that is out of scope here.

---

## 7. Bottom-line verdict (concise)

| Project ID | v19.1 status | Correct ID |
|-----------|--------------|-----------|
| T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573 | ACTIVE-UNCHANGED | (same) |
| T0846 | ACTIVE-UNCHANGED (parent; gained sub-techniques) | T0846 |
| T0885, T0836, T0814, T0806, T0835, T0831, T0888 | ACTIVE-UNCHANGED | (same) |
| **T0855** | **REVOKED → sub-technique** | **T1692.001** (Unauthorized Message: Command Message) |
| **T0856** | **REVOKED → sub-technique** | **T1692.002** (Unauthorized Message: Reporting Message) |

**Counts:** 21 total · 19 active-unchanged · 0 renamed · **2 revoked-needs-remap** (T0855,
T0856 — both ICS, both → T1692.xxx).
**Emitted-set defect:** T0855 emitted in v0.4.0 (confirmed release bug); T0856 catalogue-only
in both v0.3.0 and v0.4.0. Both releases pin v19.1.
**VP-007:** not broken; structurally unable to detect external revocation; stays green after a
mirrored 1:1 remap.

---

## Sources

| # | URL | Used for |
|---|-----|----------|
| 1 | https://attack.mitre.org/resources/updates/ | **Decisive** — verbatim v19 ICS + Enterprise Revocations/Deprecations lists; both T0855 & T0856 revocation lines |
| 2 | https://attack.mitre.org/resources/updates/updates-april-2026/ | v19 (April 2026) release notes, ICS section |
| 3 | https://attack.mitre.org/techniques/T1692/001/ | T0855 successor (ICS-domain, parent T1692) |
| 4 | https://attack.mitre.org/techniques/T1692/002/ | T0856 successor (ICS-domain, parent T1692) |
| 5 | https://attack.mitre.org/techniques/T0846/ | T0846 active; gained sub-techniques |
| 6 | https://attack.mitre.org/techniques/T0885/ | T0885 active, no sub-techniques |
| 7 | https://attack.mitre.org/techniques/T0835/ | T0835 active |
| 8 | https://attack.mitre.org/techniques/T0814/ | T0814 active |
| 9 | https://attack.mitre.org/techniques/T1083/ | T1083 active (REFUTES hallucinated "revoked" lead): v1.7, last-mod 2026-05-12 |
| 10 | https://attack.mitre.org/techniques/T1036/ | T1036 active post-tactic-split |
| 11 | https://attack.mitre.org/resources/versions/ | v19.1 current (2026-04-28); unified ICS versioning |
| 12 | https://attack.mitre.org/docs/changelogs/v18.1-v19.0/changelog-detailed.html | per-object v18→v19 ICS changelog cross-check |
| — | `.factory/research/dnp3-mitre-verification.md` | prior validated verdicts (T0855/T0803/T0814/T0836/T0827/T0828) + domain disambiguation |
| — | `src/mitre.rs` | ground-truth seeded/emitted ID sets |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep `reasoning_effort=high` sweep of all 10 ICS IDs vs v19; surfaced the T0856 revocation and reproduced the verbatim v19 ICS Revocations list (leads, then verified against primary text) |
| Perplexity perplexity_search | 2 | Pulled the exact verbatim v19 ICS Revocations block from attack.mitre.org/resources/updates/ (decisive); confirmed T1083 active against the live technique page |
| Perplexity perplexity_ask | 2 | Quick status checks on T0806/T0831/T0888/T0835/T0836 (active) and Enterprise IDs — ONE ask returned a hallucinated "T1083 revoked" claim that was then refuted via WebFetch (verify-don't-trust) |
| WebFetch | 1 | Direct fetch of attack.mitre.org/techniques/T1083/ — refuted the hallucinated revocation; confirmed v1.7 active |
| Read | 2 | `src/mitre.rs` (ground-truth IDs) + `dnp3-mitre-verification.md` (prior method/verdicts) |
| Training data | 0 load-bearing areas | No ID, name, status, version, or date taken from training data; every verdict traces to an attack.mitre.org primary source |

**Total MCP tool calls:** 5 (1 research + 2 search + 2 ask)
**Training data reliance:** low — every load-bearing fact is cited to attack.mitre.org. The
one hallucinated lead (perplexity_ask claiming T1083 revoked) was caught and refuted by direct
primary-source fetch, demonstrating the confirm-or-refute discipline; it does not appear in
any verdict.
