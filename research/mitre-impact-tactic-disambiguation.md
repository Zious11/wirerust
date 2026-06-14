# MITRE ATT&CK "Impact" Tactic Disambiguation — Enterprise (TA0040) vs. ICS (TA0105)

- **Type:** general (technology / UX convention research)
- **Date:** 2026-06-14
- **Question:** How should a security tool's grouped findings report DISPLAY and DISAMBIGUATE the
  Enterprise "Impact" tactic (TA0040) vs. the ICS "Impact" tactic (TA0105) in human-readable
  output, when both can appear as section headers in the same report?
- **Tool context:** wirerust `MitreTactic` enum, variants `Impact` (Enterprise TA0040) and
  `IcsImpact` (ICS TA0105); terminal report renders one section header per tactic.

---

## TL;DR Recommendation

**Adopt Option B (disambiguate the display string) for the ICS variant.** Render the ICS tactic
header as **`Impact (ICS)`** (or equivalently `ICS Impact`), and keep the Enterprise tactic as the
bare canonical **`Impact`**. Optionally append the TA-id to both for maximum precision
(`Impact (TA0040)` / `Impact (ICS) [TA0105]`).

Rationale in one line: MITRE keeps the *canonical name* bare for both and disambiguates by
matrix attribute + TA-id, but it **never renders both in the same view at the same time** — it
relies on a single-matrix-selection UI to supply the missing context. A grouped findings report
that mixes Enterprise and ICS tactics *does* render them together, which removes the contextual
guard MITRE depends on. Two identically-titled section headers in one document is a recognized
usability/accessibility anti-pattern (WCAG 2.4.6). Therefore the report must supply the
disambiguation that the single-matrix UI would otherwise provide. (See full rationale below.)

Confidence: **High** for the MITRE naming facts and the WCAG anti-pattern; **Medium** for the
claim that a parenthetical matrix qualifier is a broadly "recognized convention" across vendors
(see Inconclusive Findings).

---

## 1. MITRE's official naming — confirmed

Both tactics carry the **identical bare human-readable name "Impact"**; they are distinguished by
the matrix/domain and the unique TA-id, NOT by a modified name.

| Property | Enterprise | ICS |
|----------|-----------|-----|
| Canonical name | `Impact` | `Impact` |
| Tactic ID | `TA0040` | `TA0105` |
| Domain (`x_mitre_domains`) | `enterprise-attack` | `ics-attack` |
| STIX `x_mitre_shortname` | `impact` | `impact` |
| Canonical URL | attack.mitre.org/tactics/TA0040/ | attack.mitre.org/tactics/TA0105/ |
| Last modified (per page) | 25 Apr 2025 | 16 Apr 2025 |

- The page titles themselves render as **"Impact, Tactic TA0040 - Enterprise"** and
  **"Impact, Tactic TA0105 - ICS"** — i.e., MITRE *does* append the domain in the HTML `<title>`,
  but the **tactic name field is bare "Impact"** in both the matrix table and the data model.
  Source: https://attack.mitre.org/tactics/TA0040/ , https://attack.mitre.org/tactics/TA0105/
- The ICS tactics listing shows `TA0105 | Impact | The adversary is trying to manipulate,
  interrupt, or destroy your ICS systems...` — bare "Impact", no "(ICS)" qualifier in the Name
  column. Source: https://attack.mitre.org/tactics/ics/
- **Does MITRE ever qualify the ICS tactic name beyond matrix + TA-id?**
  Finding: **No, not in the canonical Name field.** MITRE's STIX data (`x-mitre-tactic` objects in
  the `mitre-attack/attack-stix-data` repo) stores `name: "Impact"` for both, with the
  disambiguator carried in `x_mitre_domains` (`enterprise-attack` vs `ics-attack`) and the
  `external_id` (`TA0040`/`TA0105`). The `x_mitre_shortname` is `impact` for both.
  Sources: https://github.com/mitre-attack/attack-stix-data/blob/master/index.json ;
  ICS Excel data `Impact (TA0105)` listing
  https://attack.mitre.org/docs/attack-excel-files/v18.1/ics-attack/ics-attack-v18.1.xlsx
  (note: the Excel/changelog references happen to write `Impact (TA0105)` for clarity, i.e. MITRE
  itself uses the TA-id suffix in prose contexts where matrix context is absent).
  Domain qualification confirmed only in: the HTML page `<title>`, the matrix/domain attribute,
  and prose references that append the TA-id.

**Conclusion for point 1:** The canonical *name* is bare "Impact" for both. MITRE disambiguates
via (a) matrix/domain attribute, (b) TA-id, and (c) in narrative prose, an appended `(TAxxxx)`.
It does **not** bake a `(ICS)` qualifier into the tactic's Name field.

---

## 2. How cross-matrix tools present the two same-named tactics

### MITRE ATT&CK Navigator (official tool)
- The Navigator is **single-domain per layer**: a layer's `domain` field is exactly one of
  `enterprise-attack`, `mobile-attack`, `ics-attack`. You select the domain when creating a layer.
  Source: attack-navigator layer format spec v4.5
  https://github.com/mitre-attack/attack-navigator/blob/master/layers/spec/v4.5/layerformat.md
  ; USAGE.md https://github.com/mitre-attack/attack-navigator/blob/master/USAGE.md
- Consequence: **the two "Impact" tactics never appear in the same matrix view simultaneously.**
  The Navigator displays the bare name "Impact" and relies on the user-selected matrix to supply
  domain context. It offers a `showID` toggle to show TA-ids in cells, and a `showName` toggle.
  This is exactly the contextual guard that a *mixed* report lacks.

### SIEM / vendor tools that span both matrices
- **Fortinet (FortiNDR / FortiAnalyzer):** spans Enterprise and ICS via an explicit **domain
  selector** ("select MITRE Domain", a dropdown to switch between Enterprise and ICS views).
  Same pattern as Navigator: separate views per domain, bare tactic names, domain supplied by the
  UI control rather than by the label.
  Sources: https://docs.fortinet.com/document/fortindr/7.6.0/administration-guide/525438/mitre-att-ck ;
  https://docs.fortinet.com/document/fortianalyzer/7.4.0/new-features/234413
- **Google SecOps / Chronicle:** MITRE matrix view; tactics shown as bare names
  (`Impact | Manipulate, interrupt, or destroy systems and data.`); platform/matrix selection
  controls which tactics appear.
  Source: https://cloud.google.com/chronicle/docs/detection/mitre-dashboard
- **Prose / documentation convention:** the de-facto community standard in narrative text is the
  **`Impact (TAxxxx)` suffix** — used by MITRE's own changelogs, vendor docs (ManageEngine,
  Palo Alto Cyberpedia), and academic papers. This is the closest thing to a "recognized
  convention," and it is **TA-id-based, not matrix-word-based**.
  Sources: https://www.manageengine.com/log-management/mitre-attack/impact.html ;
  https://www.paloaltonetworks.com/cyberpedia/what-is-mitre-attack-matrix

**Conclusion for point 2:** The dominant pattern among *interactive* cross-matrix tools is to
keep the bare name and disambiguate via a **domain/matrix selector that prevents both from showing
at once**. The dominant pattern in *prose* is the `(TAxxxx)` suffix. Appending a literal `(ICS)`
matrix word to the *name* is used by some community/internal materials but is **not** an
established MITRE/Navigator convention (see Inconclusive Findings — the deep-research model
asserted vendor-specific `(ICS)` labeling that could not be verified to primary sources).

---

## 3. Reporting/UX: is rendering two identically-titled sections an anti-pattern?

**Yes — when both appear in the same document and context does not differentiate them.**

The authoritative standard is **WCAG 2.1 Success Criterion 2.4.6 (Headings and Labels)** and its
common-practice interpretation:

> "Headings and labels must be unique **unless there is sufficient context to allow users to
> differentiate between duplicated headings or labels.**"
> — Level Access best-practice guidance, citing WCAG 2.4.6
> https://amp.levelaccess.net/public/standards/view_best_practice.php?violation_id=1248

The key phrase is the exception: duplicate headings are acceptable **only when surrounding context
disambiguates them.** This is precisely why MITRE's Navigator gets away with two bare "Impact"
headers — they're in *separate matrix views* (sufficient context). In a single grouped report that
interleaves Enterprise and ICS sections, that context is gone, so two `## Impact` headers would
fall on the **non-compliant** side of 2.4.6: a reader (especially one navigating by heading, e.g.
screen-reader users) cannot tell which "Impact" section they're in.

Supporting evidence:
- WCAG 2.4.6 rationale: non-unique headings cause users with cognitive/visual impairments to
  struggle to find/predict the correct section; skimming by heading becomes unreliable.
  Source: same Level Access page above.
- Accessibility scanning tools treat non-unique page titles/headings as a defect:
  "a number of the pages... contain the same page title; this does not allow screen reader users
  to quickly identify the page they are navigating."
  Source: CQC/Digital Accessibility Centre audit
  https://www.cqc.org.uk/sites/default/files/20190710-DAC-accesssibility-report-give-feedback-on-care.pdf
- General reporting tools that group findings by a key make the **grouping key unambiguous**
  (rule name/ID is the section header, and it is unique). Level Access "view by rule" and
  BrowserStack "issue groups" both key on a unique identifier, never on a name that can collide.
  Sources: https://client.levelaccess.com/hc/en-us/articles/360046279633 ;
  https://www.browserstack.com/docs/accessibility/accessibility-testing-dashboard/issue-groups

**Conclusion for point 3:** Two identically-titled section headers in one report **is** an
anti-pattern under WCAG 2.4.6 absent differentiating context. Reputable tools disambiguate the
section key. For wirerust's terminal report (no separate "matrix view" to supply context), the two
Impact sections WILL be co-present and adjacent in the grouping, so the headers must be made
distinct.

---

## Recommendation (detailed)

**Option B — disambiguate the display string for the ICS variant.**

Concrete proposal for wirerust:

- `MitreTactic::Impact` (Enterprise TA0040) → Display: **`Impact`** (canonical, unchanged)
- `MitreTactic::IcsImpact` (ICS TA0105) → Display: **`Impact (ICS)`** (or `ICS Impact`)

Stronger variant (recommended if the report already shows TA-ids elsewhere, for full precision and
to honor MITRE's own prose convention):

- Enterprise → **`Impact (TA0040)`**
- ICS → **`Impact (ICS) (TA0105)`** or **`Impact [TA0105]`**

### Why Option B over Option A here

1. **The contextual guard MITRE relies on is absent in this report.** MITRE/Navigator/Fortinet all
   keep the bare name *only because the matrix is selected separately and the two never co-render*.
   A grouped findings report that mixes domains removes that guard; you must reintroduce it in the
   label. (Points 1 & 2.)
2. **WCAG 2.4.6 makes co-located duplicate headers a defect** unless context differentiates them.
   For a security tool, accessibility and at-a-glance scannability of a report are real
   requirements. (Point 3.)
3. **Reader-confusion blast radius is high in this exact domain.** Enterprise Impact (ransomware,
   data destruction) and ICS Impact (physical-process manipulation, safety consequences) are
   *conceptually different and operationally critical to tell apart*. A reader who misreads which
   "Impact" they're looking at can mis-triage. wirerust analyzes ICS protocols (Modbus, DNP3),
   so ICS findings are first-class — disambiguation is not hypothetical.
4. **It does not violate MITRE fidelity.** MITRE's *canonical name* is "Impact", but MITRE itself
   appends `(TA0105)` / "ICS" in prose and in page titles when matrix context is missing. A
   parenthetical `(ICS)` is an additive *qualifier*, not a renaming — the canonical name is still
   present. Keep the TA-id available (in a column, tooltip, or suffix) as the machine-precise key.

### Caveats / how to keep MITRE-faithful

- Do **not** silently change Enterprise "Impact" to something non-canonical; only the ICS one needs
  the qualifier (or qualify both symmetrically as `Impact (Enterprise)` / `Impact (ICS)` if you
  prefer parallel structure — also acceptable and arguably clearer).
- If wirerust emits a **machine-readable** field (JSON/CSV) for the tactic, that field should carry
  the **canonical bare name + the TA-id + the domain** as separate fields (matching MITRE's STIX
  model), and reserve the `(ICS)` qualifier for the **human-readable Display string only**. Don't
  pollute the canonical name in structured output — downstream consumers expect MITRE's exact
  string keyed by TA-id.
- `ICS Impact` vs `Impact (ICS)`: both are fine. `Impact (ICS)` sorts/groups next to Enterprise
  `Impact` (helpful if sections are alphabetized) and keeps the canonical word first; `ICS Impact`
  reads more naturally as a header. Minor preference for `Impact (ICS)` to keep the canonical token
  leading and adjacent to its Enterprise sibling.

### Strongest case for Option A (for completeness)

- **MITRE fidelity purism:** the canonical name *is* bare "Impact"; some teams insist tool output
  mirror MITRE strings verbatim and disambiguate purely by an adjacent TA-id column. If wirerust
  already prints `TA0040`/`TA0105` prominently in every section header, the bare-name collision is
  arguably "differentiated by sufficient context" (the visible TA-id) and could satisfy WCAG 2.4.6.
- This is the *only* condition under which Option A is defensible: **the TA-id must be rendered in
  or immediately beside the header**, every time, with no toggle that can hide it. If the TA-id is
  optional/hidden by default (as `showID` is in Navigator), Option A fails and Option B is required.

Given wirerust is a terminal report (limited, glanceable, no interactive matrix selector), Option B
is the lower-risk, more reader-robust choice.

---

## Inconclusive / flagged findings

- **"Vendors append `(ICS)` as a recognized convention":** The deep-research (sonar-deep-research)
  pass asserted that Splunk ES, IBM QRadar, and Microsoft Sentinel display `Impact (TA0040)` /
  `Impact (TA0105)` and that some SOCs use `Impact (ICS)` in runbooks. These specific
  vendor-UI behaviors and "SOCs interviewed" claims **could not be confirmed against primary
  sources** in this pass and read as model synthesis. Treat as **low confidence**. What IS
  verified: (a) the `(TAxxxx)` suffix is widely used in *prose/docs*; (b) interactive tools use a
  domain *selector* rather than a `(ICS)` *label*. The literal `(ICS)` *name* qualifier is best
  characterized as a reasonable, defensible choice rather than an industry-standard one.
- **Enterprise tactic count drift:** sources variously say the Enterprise matrix has 14 vs 16
  tactics (Fortinet doc says 16; older F5 doc says 14). Not material to this question, but noted —
  do not rely on a fixed Enterprise tactic count.
- **ATT&CK version:** the TA-ids and names confirmed here are stable across recent versions; the
  attack.mitre.org pages cited were last modified Apr 2025. wirerust's pinned ICS version is
  v19.1 per prior research (`attack-ics-version-pin.md`); TA0105 = Impact is valid in v19.1.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Deep multi-source synthesis on MITRE dual-Impact naming, STIX data model, Navigator/vendor presentation. (1st call returned empty; 2nd succeeded, reasoning_effort=medium.) |
| Perplexity perplexity_search | 3 | Primary-source URLs: attack.mitre.org tactic pages, Navigator layer spec, WCAG 2.4.6 / duplicate-heading accessibility guidance. |
| Read | 2 | Read RESEARCH-INDEX.md; read persisted deep-research output. |
| Training data | 1 area | WCAG 2.4.6 framing (verified against Level Access source). |

**Total MCP tool calls:** 5 (2 research + 3 search)
**Training data reliance:** low — every load-bearing claim is tied to a cited URL; the one
training-data area (WCAG 2.4.6) is corroborated by a cited source.

### Key citations
- https://attack.mitre.org/tactics/TA0040/ (Enterprise Impact, bare name)
- https://attack.mitre.org/tactics/TA0105/ (ICS Impact, bare name)
- https://attack.mitre.org/tactics/ics/ (ICS tactics list — TA0105 Name column = "Impact")
- https://github.com/mitre-attack/attack-stix-data/blob/master/index.json (STIX data model)
- https://github.com/mitre-attack/attack-navigator/blob/master/layers/spec/v4.5/layerformat.md (single-domain layers)
- https://amp.levelaccess.net/public/standards/view_best_practice.php?violation_id=1248 (WCAG 2.4.6: unique headings unless sufficient context)
- https://cloud.google.com/chronicle/docs/detection/mitre-dashboard (SecOps bare-name matrix)
- https://docs.fortinet.com/document/fortianalyzer/7.4.0/new-features/234413 (Fortinet domain selector)
