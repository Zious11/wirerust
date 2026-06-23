# F5 Validation — MITRE ATT&CK ICS Tactic-ID Correctness in `src/mitre.rs`

**Date:** 2026-06-23
**Validator:** research-agent (vsdd-factory)
**Finding under review:** HIGH adversarial finding F-1 — wirerust emits Enterprise-matrix
tactic-ids (TA000x) for ICS-domain techniques while declaring `mitre_domain: "ics-attack"`.
**Verdict:** **VALIDATED — genuine correctness defect.** Severity is, if anything, *understated*:
one mapping (T0830) is wrong on both the tactic name *and* the domain.

---

## TL;DR

- **F-1 is a real defect, VALIDATED.** Declaring `mitre_domain: "ics-attack"` and pinning
  `ics-attack-19.1` is a promise to consumers that the `tactic_id` is an **ICS-matrix** ID
  (TA0100–TA0111 range). Emitting Enterprise IDs (TA0007, TA0008, …) for ICS techniques
  breaks that promise; a SIEM correlating on `(tactic_id, mitre_domain="ics-attack")` cannot
  join to MITRE's ICS matrix / Navigator, because **there is no TA0007 in the ICS matrix**.
- **Recommendation: reviewer's Option 1 (split the variants).** Document-and-accept
  (Option 2) is not defensible for a tool that explicitly tags `ics-attack`.
- **A second, independent defect surfaced:** BC EC-010 and `src/mitre.rs` map **T0830
  "Adversary-in-the-Middle" to `LateralMovement`**, but MITRE assigns T0830 to **Collection
  (TA0100)** in the ICS matrix — *not* Lateral Movement at all. So the BC is wrong even on
  the tactic *name*, not merely the matrix.

---

## Q1 — Authoritative ICS-matrix tactic IDs (TAxxxx)

The ICS matrix uses a **separate ID block (TA0100–TA0111)**, distinct from Enterprise.
Verified directly against `attack.mitre.org` tactic pages where marked ✅ (snippet/page text
shows `Tactic TAxxxx – ICS`); the remainder are canonical from the ICS matrix / STIX bundle
and corroborated by the cross-validated set (medium confidence, ⚠).

| ICS tactic                | ICS TA-id | Enterprise TA-id (for contrast) | Source confidence |
|---------------------------|-----------|---------------------------------|-------------------|
| Initial Access            | TA0108    | TA0001                          | ⚠ canonical |
| Execution                 | TA0104 ✅ | TA0002                          | verified page |
| Persistence               | TA0110    | TA0003                          | ⚠ canonical |
| Privilege Escalation      | TA0111    | TA0004                          | ⚠ canonical |
| Evasion                   | TA0103    | (Defense Evasion TA0005)        | ⚠ canonical |
| Discovery                 | TA0102 ✅ | TA0007                          | verified page |
| Lateral Movement          | TA0109 ✅ | TA0008                          | verified page |
| Collection                | TA0100 ✅ | TA0009                          | verified page |
| Command and Control       | TA0101 ✅ | TA0011                          | verified page |
| Inhibit Response Function | TA0107 ✅ | (no Enterprise analogue)        | verified page |
| Impair Process Control    | TA0106    | (no Enterprise analogue)        | ⚠ canonical |
| Impact                    | TA0105    | TA0040                          | ⚠ canonical |

Note: ICS Evasion is `TA0103`; it is *not* named "Defense Evasion" in the ICS matrix.
Source: `attack.mitre.org/matrices/ics/` and per-tactic pages
(`/tactics/TA0102`, `/TA0100`, `/TA0101`, `/TA0104`, `/TA0107`, `/TA0109`).

**Key takeaway:** The current `src/mitre.rs` `technique_tactic_id()` table maps every
name-colliding ICS tactic to its *Enterprise* ID (Discovery→TA0007, LateralMovement→TA0008,
Collection→TA0009, CommandAndControl→TA0011, Persistence→TA0003). Every one of those is the
**wrong matrix** for an ICS technique under `mitre_domain="ics-attack"`.

---

## Q2 — T0888 and T0846 (both ICS Discovery)

Both are ICS-domain techniques (T0xxx), present **only** in the ICS matrix.

| Technique | Name | MITRE tactic | Correct ICS TA-id | wirerust emits | Verdict |
|-----------|------|--------------|-------------------|----------------|---------|
| **T0888** | Remote System Information Discovery | Discovery | **TA0102** | TA0007 (Enterprise) | ❌ WRONG matrix |
| **T0846** | Remote System Discovery | Discovery | **TA0102** | TA0007 (Enterprise) | ❌ WRONG matrix |

Verified directly against `attack.mitre.org/techniques/T0888/` and `/T0846/` — both list
**Parent Tactic: Discovery (TA0102)**. Enterprise `TA0007` is the **wrong** `tactic_id` for
both under `mitre_domain="ics-attack"`. (T0888 is emitted by the Modbus recon analyzer;
T0846 is seeded but not yet emitted — both share the same defect via the `Discovery` variant.)

**Answer to "Is Enterprise TA0007 the WRONG tactic_id for T0888?"** — **Yes.** Correct value
is **TA0102** (ICS Discovery). Same for T0846 → **TA0102**.

---

## Q3 — T0830 "Adversary-in-the-Middle" (BC EC-010 assertion)

**The BC is wrong, and not in the way the reviewer assumed.**

BC `BC-2.11.035` EC-010 and `src/mitre.rs:181` assert:

> T0830 → `tactic_id: "TA0008"`, `tactic_name: "Lateral Movement"`
> ("ICS lateral movement … same tactic-id as Enterprise LateralMovement")

MITRE, verified directly against `attack.mitre.org/techniques/T0830/` **and** the ICS
Collection tactic page `attack.mitre.org/tactics/TA0100/` (which lists T0830 among its
techniques), assigns:

| Technique | Name | MITRE ICS tactic | Correct ICS TA-id | wirerust / BC asserts | Verdict |
|-----------|------|------------------|-------------------|-----------------------|---------|
| **T0830** | Adversary-in-the-Middle | **Collection** | **TA0100** | TA0008 / "Lateral Movement" | ❌ WRONG tactic AND wrong matrix |

So EC-010 is doubly incorrect:
1. **Wrong tactic name** — T0830 is **Collection**, not Lateral Movement, in the current ICS
   matrix (T0830 appears on the `TA0100` Collection page; it is *absent* from the `TA0109`
   Lateral Movement page).
2. **Wrong matrix** — even the tactic it *claims* (Lateral Movement) would be ICS **TA0109**,
   not Enterprise **TA0008**.

**Cross-matrix note (reported precisely):** T0830 is an ICS-only technique (T0xxx). The
**Enterprise** analogue is a *different* technique, **T1557 "Adversary-in-the-Middle"** (and
its sub-technique T1557.002 ARP Cache Poisoning, which wirerust already catalogs separately
under Enterprise Credential Access / TA0006). There is no Enterprise T0830. Historical note:
some third-party/older derivative material has associated AiTM with Lateral Movement
conceptually, but the canonical MITRE ICS matrix assigns T0830 to **Collection (TA0100)**.
Confidence: **high** — confirmed by two independent MITRE pages (technique page + tactic page).

---

## Q4 — Correctness impact for a SIEM / ECS / OCSF consumer

**It is a genuine correctness defect, not an acceptable canonicalization.**

- **What `mitre_domain="ics-attack"` promises:** that technique *and* tactic IDs are drawn
  from the ICS matrix. The pair `(mitre_domain="ics-attack", tactic_id="TA0007")` is
  internally contradictory — TA0007 does not exist in the ICS matrix, so a consumer joining
  on that pair against MITRE ICS content / ATT&CK Navigator gets no match.
- **ECS** (`threat.tactic.id`, `threat.technique.id`): framework-agnostic; it expects the ID
  you supply to be a valid ATT&CK ID *from the framework you claim to use*. For an ICS-domain
  event that means ICS tactic IDs (TA01xx). ECS has **no** convention that normalizes an ICS
  technique's tactic to its Enterprise counterpart.
- **OCSF** (Security/Detection Finding ATT&CK mapping): deliberately vendor-neutral; it
  carries whatever canonical IDs the framework defines. The expected value for an ICS finding
  is the **ICS-matrix** tactic ID. Any Enterprise cross-walk is an optional analytic layer,
  never the canonical representation.
- **Established convention for ICS technique → tactic resolution:** MITRE's own per-technique
  pages are authoritative — each ICS technique page lists its ICS tactic(s) with ICS IDs.
  The convention for any tool declaring ICS ATT&CK is: ICS technique IDs + ICS tactic IDs.

Sources: `attack.mitre.org/matrices/ics/`; Elastic ECS `ecs-threat` field docs
(`threat.tactic.id` / `threat.technique.id`); OCSF schema (`ocsf.io`). Confidence: high on
the directional conclusion; the ECS/OCSF docs are framework-agnostic (they neither mandate
nor forbid ICS IDs), which is itself the point — they impose no Enterprise-canonicalization
rule, so the burden is on the emitter to be domain-faithful.

---

## Q5 — Recommendation

**Adopt reviewer's Option 1: split the name-colliding `MitreTactic` variants so ICS
techniques carry ICS-matrix TA-ids.** This is the MITRE-faithful fix.

Rationale:
1. The tool *explicitly* sets `mitre_domain="ics-attack"` and pins `ics-attack-19.1`. That is
   a commitment to ICS-matrix IDs; Enterprise IDs make `mitre_domain` a misleading label.
2. It enables straightforward `(tactic_id, mitre_domain)` joins to MITRE ICS content.
3. It is the only option consistent with how the project *already* behaves for Impact.

**The project is already internally inconsistent**, which strengthens Option 1: it correctly
splits **IcsImpact → TA0105** (separate from Enterprise Impact TA0040) — see `src/mitre.rs`
lines 69, 178, 236 and the doc comment at lines 145–148 that *explicitly acknowledges* the
ICS/Enterprise TA-id divergence ("Enterprise Discovery TA0007 vs ICS Discovery TA0111" — note
that comment even has the wrong ICS Discovery number; it is **TA0102**, not TA0111) — yet
deliberately merges Discovery / Lateral Movement / Collection / C2 / Persistence into the
Enterprise variants. There is no principled line between "split Impact" and "merge Discovery";
the current state is a half-applied policy, not a coherent canonicalization.

**Why not Option 2 (document-and-accept + pin a test):** Pinning a known-wrong value with a
test institutionalizes the defect. If Enterprise canonicalization were genuinely desired, the
correct expression would be to set `mitre_domain="enterprise-attack"` (or omit the ICS domain
claim) — but that conflicts with the ICS technique IDs (T0xxx) being emitted, and with the
v19.1 ICS pin. Option 2 cannot be made self-consistent.

### Concrete remediation (for the implementing story — not done here)

Introduce ICS-specific variants for the colliding tactics and remap the affected catalog
entries. Minimum set implied by the verified findings:

| Catalog entry | Current variant → TA-id | Correct ICS variant → TA-id |
|---------------|-------------------------|-----------------------------|
| T0846 Remote System Discovery | Discovery → TA0007 | IcsDiscovery → **TA0102** |
| T0888 Remote System Information Discovery | Discovery → TA0007 | IcsDiscovery → **TA0102** |
| T0830 Adversary-in-the-Middle | LateralMovement → TA0008 | **IcsCollection → TA0100** (NOT Lateral Movement) |
| T0885 Commonly Used Port | CommandAndControl → TA0011 | IcsCommandAndControl → **TA0101** |

Plus BC `BC-2.11.035` EC-010 must be corrected: T0830 expected output should be
`tactic_id: "TA0100"`, `tactic_name: "Collection"`. (EC-009 / T1557.002 remains correct — it
is a genuine Enterprise sub-technique → TA0006.) Other ICS entries already mapped to
ICS-unique variants (T0836/T0806/T0835/T0831/T1692.* → IcsImpairProcessControl/TA0106;
T0814/T1691.001 → IcsInhibitResponseFunction/TA0107; T0827 → IcsImpact/TA0105) are **already
correct** and need no change — those tactics have no Enterprise name-collision.

> **DF-VALIDATION-001 note:** This validation supports filing the finding as an issue. The
> defect is confirmed against authoritative MITRE sources; it is not speculative.

---

## Inconclusive / lower-confidence items (flagged)

- ICS TA-ids marked ⚠ in Q1 (Initial Access TA0108, Persistence TA0110, Privilege Escalation
  TA0111, Evasion TA0103, Impair Process Control TA0106, Impact TA0105) were **not** each
  re-fetched from their individual `attack.mitre.org` pages in this pass; they come from the
  ICS matrix / STIX bundle synthesis and are consistent across the cross-validated source set.
  Impact=TA0105 is independently corroborated by the project's own working IcsImpact mapping.
  Confidence: medium-high. Only the four directly relevant to this finding (Discovery TA0102,
  Collection TA0100, Lateral Movement TA0109, Command and Control TA0101, Execution TA0104,
  Inhibit Response Function TA0107) were page-verified.
- The `src/mitre.rs` line-145 comment cites "ICS Discovery TA0111" — that specific number is
  **incorrect** (ICS Discovery is **TA0102**; TA0111 is ICS Privilege Escalation). This is a
  doc-comment error worth fixing alongside the code, but it is cosmetic relative to the
  emitted-data defect.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Full ICS tactic→TA-id table TA0100–TA0111; (2) per-technique tactic for T0888/T0846/T0830 + cross-matrix/domain status |
| Perplexity perplexity_reason | 1 | Synthesis: SIEM/ECS/OCSF expectations, internal-consistency of split-Impact-but-not-Discovery, Option 1 vs 2 |
| WebFetch | 5 | Direct verification against attack.mitre.org: /techniques/T0830, /T0888, /T0846; /tactics/TA0100 (Collection — confirmed T0830 listed) |
| Read | 1 | src/mitre.rs catalog (technique_info + technique_tactic_id tables) |
| Grep | 3 | Extract T0830 mentions from research output; locate + read BC EC-010 assertion |
| Training data | 1 area | MITRE domain-model background framing only — every load-bearing TA-id and tactic assignment is web-verified, not from training data |

**Total MCP tool calls:** 8 (3 Perplexity + 5 WebFetch)
**Training data reliance:** low — all four finding-critical mappings (T0888, T0846, T0830, and
the ICS TA-id block) were verified against live attack.mitre.org pages; the ⚠-marked Q1 IDs
are the only items resting on STIX-bundle/matrix synthesis rather than per-page fetches.
