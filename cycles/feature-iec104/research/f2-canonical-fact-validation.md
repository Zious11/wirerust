# F2 Canonical Fact-Validation — IEC 60870-5-104 Feature

**Date:** 2026-07-13
**Agent:** vsdd-factory:research-agent
**Policy:** DF-VALIDATION-001 (every verdict sourced; inconclusive points flagged, not guessed)
**Scope:** Two canonical-frame disputes from the F2 adversarial spec review.
**Matrix version context:** repo pins `ics-attack-19.1`. See Version Caveat below.

---

## VERDICT TABLE (concise)

### Dispute 1 — MITRE ATT&CK for ICS technique identity

| # | Disputed fact | Verdict | Authoritative value | Source |
|---|---------------|---------|---------------------|--------|
| 1a | Feature seeds **T0809 = "Service Stop" / Inhibit Response Function** | **REFUTED (name wrong; tactic right)** | T0809 = **"Data Destruction"**, tactic **Inhibit Response Function**. The *name* "Service Stop" is wrong; the *tactic* string "Inhibit Response Function" is coincidentally correct for T0809. | attack.mitre.org/techniques/T0809/ |
| 1b | Adversary: "Service Stop is **T0881**" | **CONFIRMED** | T0881 = **"Service Stop"**, tactic **Inhibit Response Function**. | attack.mitre.org/techniques/T0881/ |
| 1c | Adversary: "**T0829** is Loss of View" | **CONFIRMED** | T0829 = **"Loss of View"**, tactic **Impact** (ICS Impact, TA0105). | attack.mitre.org/techniques/T0829/ |
| 1d | Is T0809 a NEW technique the feature must ADD? Which tactic enum? | **CONFIRMED NEW / tactic = IcsInhibitResponseFunction** | T0809 is NOT in `src/mitre.rs` today → the feature ADDS it. Required seed: `"T0809" => ("Data Destruction", MitreTactic::IcsInhibitResponseFunction)` (TA0107). **NOT IcsImpact.** | src/mitre.rs (read) + attack.mitre.org/techniques/T0809/ |
| 1e-i | T1692.001 = "Unauthorized Message: Command Message" (adversary: NOT "Exploitation of Remote Services") | **CONFIRMED** | MITRE page title is **"Command Message"** under parent T1692 **"Unauthorized Message"**; repo's composed name "Unauthorized Message: Command Message" follows MITRE's parent:sub display convention. Tactics: **Evasion + Impair Process Control**. Repo's `IcsImpairProcessControl` is one of the two valid tactics. Repo is CORRECT. | attack.mitre.org/techniques/T1692/001/ |
| 1e-ii | T0831 = "Manipulation of Control" / Impact | **CONFIRMED** | T0831 = "Manipulation of Control", tactic **Impact** (TA0105). Repo `IcsImpact` correct. | attack.mitre.org/techniques/T0831/ |
| 1e-iii | T0836 = "Modify Parameter" / Impair Process Control | **CONFIRMED** | T0836 = "Modify Parameter", tactic **Impair Process Control** (TA0106). Repo `IcsImpairProcessControl` correct. | attack.mitre.org/techniques/T0836/ |
| 1e-iv | T0827 = "Loss of Control" / Impact | **CONFIRMED** | T0827 = "Loss of Control", tactic **Impact** (TA0105). Repo `IcsImpact` correct. | attack.mitre.org/techniques/T0827/ |

### Dispute 2 — IEC-104 control-command TypeIDs

| # | Disputed fact | Verdict | Authoritative value | Source |
|---|---------------|---------|---------------------|--------|
| 2a | C_SE has exactly three variants (NA_1=48, NB_1=49, NC_1=50); **C_SE_ND_1 does NOT exist** | **CONFIRMED** | Non-timestamped C_SE family is exactly NA_1(48)/NB_1(49)/NC_1(50). No standardized C_SE_ND_1. (Timestamped set: C_SE_TA_1/TB_1/TC_1 at 61–63.) | Wireshark packet-iec104.c; OpenMUC ASduType; FreyrSCADA IEC-104 |
| 2b | Correct TypeID for **C_BO_NA_1** | **CONFIRMED = 51** | TypeID **51 = C_BO_NA_1** (bitstring of 32 bits command). C_BO_TA_1 (timestamped) = 64. | OpenMUC ASduType; Wireshark; FreyrSCADA |
| 2c | Is TypeID 52 reserved? Correct control range? | **CONFIRMED (spec's 45–52 is wrong)** | **TypeIDs 52–57 are "Reserved (standard area)"** — no ASDU assigned. Non-timestamped control-process range is **45–51**, not 45–52. (Full control-direction band 45–64; 58–64 = timestamped variants.) | FreyrSCADA (explicit "Reserved"); OpenMUC (gap 52–57); Wireshark |
| 2d | System commands 100/101/103/105 | **CONFIRMED** | C_IC_NA_1=100, C_CI_NA_1=101, C_CS_NA_1=103, C_RP_NA_1=105 (also C_RD_NA_1=102, C_TS_NA_1=104, C_CD_NA_1=106, C_TS_TA_1=107). | Wireshark; OpenMUC; Beckhoff; Pro-face; SCADAprotocols; FreyrSCADA |

**Spec's TypeID map {45:C_SC_NA_1, 46:C_DC_NA_1, 47:C_RC_NA_1, 48:C_SE_NA_1, 49:C_SE_NB_1, 50:C_SE_NC_1, 51:C_BO_NA_1} is CORRECT for 45–51.** The 8th entry **52 → C_BO_NA_1 (paired with a phantom "C_SE_ND_1") is REFUTED**: C_BO_NA_1 is already 51, 52 is reserved, and C_SE_ND_1 is not a standardized type. **Adversary is fully correct on Dispute 2.**

---

## DETAILED FINDINGS

### Dispute 1 — technique identity

The adversary's three assertions are all correct against the current official ATT&CK for ICS technique pages:

- **T0809 = "Data Destruction" / Inhibit Response Function.** The feature's seed name "Service Stop" is wrong. Notably, the *tactic* the feature paired with it ("Inhibit Response Function") is the correct tactic for T0809 — so only the name is defective. Both T0809 (Data Destruction) and T0881 (Service Stop) live under Inhibit Response Function, which is why the tactic string "looked right" even with the wrong name. (attack.mitre.org/techniques/T0809/)
- **T0881 = "Service Stop" / Inhibit Response Function.** If the feature's detection intent is genuinely "a service/process was stopped," the correct ID is **T0881**, not T0809. (attack.mitre.org/techniques/T0881/)
- **T0829 = "Loss of View" / Impact.** Confirmed. (attack.mitre.org/techniques/T0829/)

**Repo cross-check (`src/mitre.rs`, read in full):**

- T0809, T0881, T0829 are **NOT currently in the catalogue** (`technique_info` match / `SEEDED_TECHNIQUE_IDS`). Whichever the feature adopts is a NEW seeded entry, and `SEEDED_TECHNIQUE_ID_COUNT` (currently 28) plus `SEEDED_TECHNIQUE_IDS` must be bumped in lockstep (the `vp007_catalog_drift_guard` test enforces this).
- **Required seed for T0809 (if kept):** `"T0809" => ("Data Destruction", MitreTactic::IcsInhibitResponseFunction)`. The tactic enum is **`IcsInhibitResponseFunction`** (→ `TA0107`), **not `IcsImpact`**. This matches the repo's existing convention for the same tactic, e.g. `T0814` Denial of Service and `T0816` Device Restart/Shutdown are both `IcsInhibitResponseFunction`.
- The repo's already-seeded IDs the feature reuses are all correct as-is: **T0831** (`"Manipulation of Control"`, `IcsImpact`), **T0836** (`"Modify Parameter"`, `IcsImpairProcessControl`), **T0827** (`"Loss of Control"`, `IcsImpact`), **T1692.001** (`"Unauthorized Message: Command Message"`, `IcsImpairProcessControl`). No change needed to any of these four.

**Note on T1692.001 tactic:** MITRE v19 lists T1692 / T1692.001 under **two** tactics — **Evasion** (ICS Evasion, TA0103) *and* **Impair Process Control** (TA0106). The repo's single-tactic data model records `IcsImpairProcessControl`, which is one of the two authoritative tactics and is a defensible choice. Flagging as a design nuance, not an error. If the F2 spec wants to note the alternative tactic it may, but no correction is required.

### Dispute 2 — IEC-104 TypeIDs

Multiple independent authoritative sources converge (Wireshark `packet-iec104.c`, OpenMUC `ASduType`, DIN-DKE/FreyrSCADA IEC-104 summary, Beckhoff and Pro-face vendor tables, SCADAprotocols):

- **45–51 canonical map is exactly as the spec states:** 45 C_SC_NA_1, 46 C_DC_NA_1, 47 C_RC_NA_1, 48 C_SE_NA_1 (normalized), 49 C_SE_NB_1 (scaled), 50 C_SE_NC_1 (short float), **51 C_BO_NA_1** (bitstring of 32 bits).
- **C_SE_ND_1 does not exist** in the base IEC 60870-5-101/104 standard. The only mention found anywhere is a single non-authoritative Chinese-language study note; it appears in zero conforming implementations. The set-point family is three non-timestamped variants (NA_1/NB_1/NC_1) plus three timestamped (TA_1/TB_1/TC_1 at 61–63).
- **TypeIDs 52–57 are reserved** ("Reserved (standard area)" per FreyrSCADA; a definitional gap in OpenMUC/Wireshark). So the control-process (non-timestamped) range is **45–51**, and the spec's implied "45–52" with an 8th type at 52 is wrong.
- **System commands confirmed:** C_IC_NA_1=100, C_CI_NA_1=101, C_CS_NA_1=103, C_RP_NA_1=105 (plus C_RD_NA_1=102, C_TS_NA_1=104, C_CD_NA_1=106, C_TS_TA_1=107).

**Fix direction for the spec:** the 8-element control-command set should be the **7-element set 45–51** ending at C_BO_NA_1 (51). Drop the phantom TypeID 52 / C_SE_ND_1 entry entirely. If a bitstring command is desired it is already present as C_BO_NA_1 = 51.

---

## Version Caveat (DF-VALIDATION-001 transparency)

The deep-research pass could not surface a downloadable artifact explicitly labelled `ics-attack-v19.1` (the newest versioned ICS spreadsheet exposed in results was v17.1). The verdicts instead rely on the **current official technique pages** at attack.mitre.org, which are the canonical source MITRE itself treats as authoritative for name+tactic. Corroborating signals that the pages reflect the v19 line: T0829 "Loss of View" last-modified 2026-05-12; T0836 last-modified 2025-04-16; T1692/T1692.001 created 2026-04-20 (consistent with the ATT&CK v19 release that introduced the T1692 "Unauthorized Message" restructuring — the same restructuring the repo tracked in its v19 remap, issue #222, T0855→T1692.001 / T0856→T1692.002). Confidence: **high** that these identities match `ics-attack-19.1`. Residual risk: **low** — none of the six T0xxx identities in question have changed name or tactic across recent ICS versions.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Authoritative name+tactic for T0809/T0881/T0829/T0831/T0836/T0827 against official attack.mitre.org ICS pages; (2) IEC 60870-5-101/104 control-command TypeID table 45–51/52–57/100–107 against Wireshark dissector + OpenMUC + FreyrSCADA + vendor docs. Both run at `reasoning_effort: high`. |
| Perplexity perplexity_ask | 1 | Targeted lookup of T1692 / T1692.001 exact name + tactic (the sub-technique that replaced T0855). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read | 3 | `src/mitre.rs` (repo mapping cross-check) + 2 persisted research result files. |
| Training data | 0 areas | All verdicts sourced to web/MCP findings + the repo file; no version numbers or TypeIDs asserted from model memory. |

**Total MCP tool calls:** 3 (2 `perplexity_research` + 1 `perplexity_ask`)
**Training data reliance:** low — every disputed fact is backed by attack.mitre.org technique pages (Dispute 1) or by ≥3 converging IEC-104 implementations/standards (Dispute 2), cross-checked against the actual `src/mitre.rs` source.

### Key sources
- MITRE ATT&CK for ICS technique pages: attack.mitre.org/techniques/{T0809,T0881,T0829,T0831,T0836,T0827,T1692,T1692/001}/
- Wireshark IEC-104 dissector: github.com/boundary/wireshark/blob/master/epan/dissectors/packet-iec104.c
- OpenMUC IEC 60870-5-104 `ASduType`: openmuc.org/iec-60870-5-104/javadoc/org/openmuc/j60870/ASduType.html
- DIN-DKE / FreyrSCADA IEC-60870-5-104 type summary (explicit "Reserved 52–57"): github.com/DIN-DKE/IEC_60870_5_104__FreyrSCADA_IEC-60870-5-104-
- Beckhoff TF6500 IEC 60870-5-10x data types; Pro-face IEC 60870-5-101 driver manual; SCADAprotocols IEC-104 Type IDs reference
- Repo file: `/Users/zious/Documents/GITHUB/wirerust/src/mitre.rs`
