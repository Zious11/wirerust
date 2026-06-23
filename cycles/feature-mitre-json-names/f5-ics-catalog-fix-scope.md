# F5 Change-Set Scope — ICS Tactic-ID Correctness Fix

**Date:** 2026-06-23
**Author:** architect (vsdd-factory)
**Basis:** f5-ics-tactic-id-validation.md (VALIDATED verdict — genuine defect)
**Commit baseline:** develop @ 2fa6606 (clean worktree at conversation start)
**Scope type:** Read-only scoping — no code modified.

---

## Executive Summary

The fix requires adding **four new `MitreTactic` enum variants** (IcsDiscovery, IcsCollection,
IcsCommandAndControl, and either IcsLateralMovement or reclassifying T0830 entirely to
IcsCollection), updating **seven technique-to-tactic mappings** in `technique_info`, extending
**two match tables** (`Display` impl and `technique_tactic_id`), correcting **one inline comment**,
and updating or correcting **nine test functions** across three test files — plus version-bumping
**five behavioral contracts**.

The emitted-technique count does NOT change (still 17 emitted IDs). The seeded-ID count does NOT
change (still 25). The `all_tactics_in_report_order` slice grows from 17 to 20 entries.

---

## Section 1 — Catalog Inventory

Full listing of all 25 technique arms in `technique_info` (src/mitre.rs:129–188):

| Line | Technique ID | Name | Current `MitreTactic` variant | Matrix | Correct? |
|------|-------------|------|------------------------------|--------|----------|
| 131 | T1027 | Obfuscated Files or Information | DefenseEvasion | Enterprise | CORRECT |
| 135 | T1036 | Masquerading | DefenseEvasion | Enterprise | CORRECT |
| 136 | T1040 | Network Sniffing | CredentialAccess | Enterprise | CORRECT |
| 137 | T1046 | Network Service Discovery | Discovery | Enterprise | CORRECT |
| 138 | T1071 | Application Layer Protocol | CommandAndControl | Enterprise | CORRECT |
| 139 | T1071.001 | Web Protocols | CommandAndControl | Enterprise | CORRECT |
| 140 | T1071.004 | DNS | CommandAndControl | Enterprise | CORRECT |
| 141 | T1083 | File and Directory Discovery | Discovery | Enterprise | CORRECT |
| 142 | T1499.002 | Service Exhaustion Flood | Impact | Enterprise | CORRECT |
| 143 | T1505.003 | Web Shell | Persistence | Enterprise | CORRECT |
| 144 | T1573 | Encrypted Channel | CommandAndControl | Enterprise | CORRECT |
| 149 | **T0846** | Remote System Discovery | **Discovery** | ICS | **WRONG** — should be IcsDiscovery (TA0102) |
| 150–153 | T1692.001 | Unauthorized Message: Command Message | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 154–157 | T1692.002 | Unauthorized Message: Reporting Message | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 158 | **T0885** | Commonly Used Port | **CommandAndControl** | ICS | **WRONG** — should be IcsCommandAndControl (TA0101) |
| 160 | T0836 | Modify Parameter | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 161 | T0814 | Denial of Service | IcsInhibitResponseFunction | ICS | CORRECT (TA0107) |
| 162 | T0806 | Brute Force I/O | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 163 | T0835 | Manipulate I/O Image | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 164–167 | T0831 | Manipulation of Control | IcsImpairProcessControl | ICS | CORRECT (TA0106) |
| 168–171 | **T0888** | Remote System Information Discovery | **Discovery** | ICS | **WRONG** — should be IcsDiscovery (TA0102); this ID is EMITTED by Modbus recon |
| 174–177 | T1691.001 | Block Operational Technology Message: Command Message | IcsInhibitResponseFunction | ICS | CORRECT (TA0107) |
| 178 | T0827 | Loss of Control | IcsImpact | ICS | CORRECT (TA0105) |
| 181 | **T0830** | Adversary-in-the-Middle | **LateralMovement** | ICS | **WRONG** — should be IcsCollection (TA0100); MITRE ICS matrix puts T0830 under Collection, not Lateral Movement. This ID is EMITTED by the ARP analyzer. |
| 183–185 | T1557.002 | Adversary-in-the-Middle: ARP Cache Poisoning | CredentialAccess | Enterprise | CORRECT (TA0006) |

**ICS techniques incorrectly mapped to Enterprise/name-merged variants: T0846, T0885, T0888, T0830.**

**Emitted ICS techniques with wrong tactic: T0888 (emitted, Modbus recon), T0830 (emitted, ARP spoof).**
**Seeded-only ICS techniques with wrong tactic: T0846, T0885.**

---

## Section 2 — Enum Change Proposal

### New variants to add to `MitreTactic` enum (src/mitre.rs:47–70)

Only variants for tactics that actually have ICS techniques in the catalog are needed:

| New Variant | Display String | `technique_tactic_id` arm | Replaces (for ICS techniques) | Notes |
|------------|---------------|--------------------------|-------------------------------|-------|
| `IcsDiscovery` | `"Discovery (ICS)"` | `"TA0102"` | Enterprise `Discovery` for T0846, T0888 | Display needs `(ICS)` parenthetical to disambiguate from Enterprise "Discovery" in co-rendered reports — same D-069 rationale as IcsImpact |
| `IcsCollection` | `"Collection (ICS)"` | `"TA0100"` | Enterprise `LateralMovement` for T0830 | T0830 is Collection in ICS, NOT Lateral Movement. Display needs `(ICS)` parenthetical. |
| `IcsCommandAndControl` | `"Command and Control (ICS)"` | `"TA0101"` | Enterprise `CommandAndControl` for T0885 | T0885 Commonly Used Port is ICS C2 tactic TA0101. Display needs `(ICS)` parenthetical. |

**Note on IcsPersistence, IcsExecution, IcsLateralMovement:** No ICS techniques in the current
25-entry catalog map to ICS Persistence (TA0110), ICS Execution (TA0104), or ICS Lateral Movement
(TA0109 — confirmed by f5-ics-tactic-id-validation.md: T0830 is Collection TA0100, not Lateral
Movement). Do NOT add variants for tactics with no catalog entries.

### Technique-to-variant remapping

| Technique ID | Old variant | New variant | Is Emitted? |
|-------------|------------|-------------|-------------|
| T0846 | `Discovery` | `IcsDiscovery` | No (seeded-only) |
| T0888 | `Discovery` | `IcsDiscovery` | YES — Modbus recon |
| T0830 | `LateralMovement` | `IcsCollection` | YES — ARP spoof |
| T0885 | `CommandAndControl` | `IcsCommandAndControl` | No (seeded-only) |

### `all_tactics_in_report_order` slice change (src/mitre.rs:100–120)

Grows from 17 to 20 entries. The three new ICS variants are appended after `IcsImpact`:

```
// Before: 17 entries, last 3 are IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact
// After: 20 entries, last 6 are IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact,
//                                  IcsDiscovery, IcsCollection, IcsCommandAndControl
```

Order within the appended set follows ICS TA-id numeric order: IcsDiscovery (TA0102) < IcsCollection
(TA0100) is already lower — or use the same append-to-end convention as existing ICS variants.
The recommended order (matches ICS matrix kill-chain phase order):
`IcsDiscovery, IcsCollection, IcsCommandAndControl`

---

## Section 3 — Affected-Site Sweep

### src/mitre.rs

| File:Line | Current assertion / content | Must become |
|-----------|----------------------------|-------------|
| `src/mitre.rs:47–70` | `MitreTactic` enum — 17 variants | Add 3 variants: `IcsDiscovery`, `IcsCollection`, `IcsCommandAndControl` |
| `src/mitre.rs:72–95` | `Display` impl — 17 arms | Add 3 arms: `IcsDiscovery => "Discovery (ICS)"`, `IcsCollection => "Collection (ICS)"`, `IcsCommandAndControl => "Command and Control (ICS)"` |
| `src/mitre.rs:100–120` | `all_tactics_in_report_order` — 17-element slice | Extend to 20 elements; append `IcsDiscovery`, `IcsCollection`, `IcsCommandAndControl` after `IcsImpact` |
| `src/mitre.rs:145–148` | Comment: "we intentionally merge by name so a single grouped report has one section per tactic name regardless of source matrix" — also cites "Enterprise Discovery TA0007 vs ICS Discovery **TA0111**" (wrong number; ICS Discovery is TA0102) | Comment must be removed or rewritten to reflect the split-by-variant policy now applied consistently. The TA0111 number is incorrect (TA0111 = ICS Privilege Escalation; ICS Discovery = TA0102). |
| `src/mitre.rs:149` | `"T0846" => ("Remote System Discovery", MitreTactic::Discovery)` | `MitreTactic::IcsDiscovery` |
| `src/mitre.rs:158` | `"T0885" => ("Commonly Used Port", MitreTactic::CommandAndControl)` | `MitreTactic::IcsCommandAndControl` |
| `src/mitre.rs:168–171` | `"T0888" => ("Remote System Information Discovery", MitreTactic::Discovery)` | `MitreTactic::IcsDiscovery` |
| `src/mitre.rs:181` | `"T0830" => ("Adversary-in-the-Middle", MitreTactic::LateralMovement)` | `MitreTactic::IcsCollection` |
| `src/mitre.rs:219–238` | `technique_tactic_id` match — 17 arms | Add 3 arms: `IcsDiscovery => "TA0102"`, `IcsCollection => "TA0100"`, `IcsCommandAndControl => "TA0101"` |
| `src/mitre.rs:252` | Kani comment: "The catalogue is a closed-world static match; the seeded set is finite (25)" | No change to count — still 25 seeded IDs. But the enum-variant count cited in adjacent comments changes. |
| `src/mitre.rs:263` | Comment in `EMITTED_IDS`: `"T0830",  // Adversary-in-the-Middle (BC-2.16.004; LateralMovement)` | Must say `IcsCollection` |
| `src/mitre.rs:286–287` | `EMITTED_IDS` line for T0830 comment | Update from `LateralMovement` to `IcsCollection` |
| `src/mitre.rs:350–353` | `SEEDED_TECHNIQUE_IDS` comment block — note the "13 ICS" count and the T0830 note citing `ICS LateralMovement` | Update T0830 comment from `ICS LateralMovement` to `ICS Collection (IcsCollection)` |

### tests/mitre_tests.rs

These tests will BREAK after the enum/mapping change:

| Test function | File:Lines (approx) | Current assertion | Must become | Break? |
|--------------|--------------------|--------------------|-------------|--------|
| `test_all_tactics_length_is_16` | mitre_tests.rs:105–113 | `assert_eq!(...len(), 17)` with comment "14 Enterprise + 3 ICS-unique = 17" | `assert_eq!(...len(), 20)` with comment "14 Enterprise + 6 ICS-unique = 20" | YES |
| `test_all_tactics_enterprise_kill_chain_order` | mitre_tests.rs:120–142 | Asserts first 14 elements — no change to those 14 | No change to the first-14 assertion, but test name / doc comment mentions "17" | Minor doc update only |
| `test_all_tactics_ics_at_end` | mitre_tests.rs:149–163 | Asserts `tactics[14] == IcsInhibitResponseFunction` and `tactics[15] == IcsImpairProcessControl` — NO assertion on [16], [17], [18], [19] | Add assertions for `tactics[16] == IcsImpact`, `tactics[17] == IcsDiscovery`, `tactics[18] == IcsCollection`, `tactics[19] == IcsCommandAndControl` | YES (extend to cover new variants) |
| `test_all_tactics_no_duplicates` | mitre_tests.rs:171–184 | `assert_eq!(unique.len(), 17)` | `assert_eq!(unique.len(), 20)` | YES |
| `test_all_tactics_all_variants_present` | mitre_tests.rs:192–224 | Expected set has 17 variants — does NOT include `IcsDiscovery`, `IcsCollection`, `IcsCommandAndControl` | Add 3 new variants to `expected` HashSet | YES |
| `test_technique_tactic_correct_assignments` | mitre_tests.rs:322–367 | `("T0846", MitreTactic::Discovery)` at line ~347; `("T0888", MitreTactic::Discovery)` at line ~357 | T0846 → `IcsDiscovery`; T0888 → `IcsDiscovery`; T0885 and T0830 are NOT in this table (but T0885 is missing; see below) | YES — T0846 and T0888 lines break |
| `test_technique_name_resolves_all_21_seeded_ids` | mitre_tests.rs:232–296 | Title says "21 seeded IDs"; the loop only checks 21 entries and does NOT assert tactic | Title/count mismatch with reality (25 seeded), but loop does NOT check tactic assignment — does NOT break from tactic change alone | No break from tactic change, but count comment is stale |

**Note:** `test_all_emitted_ids_resolve` (mitre_tests.rs:374–421) does NOT assert tactic
assignments, only that `technique_name` and `technique_tactic` return `Some`. It will NOT break
from the tactic change — the new variants resolve correctly.

### tests/reporter_json_tests.rs

| Test function | File:Lines (approx) | Current assertion | Must become | Break? |
|--------------|--------------------|--------------------|-------------|--------|
| `test_BC_2_11_035_ec010_ics_lateral_movement` | reporter_json_tests.rs:1071–1104 | `entry["tactic_id"] == "TA0008"` (line 1094); `entry["tactic_name"] == "Lateral Movement"` (line 1097) | `tactic_id` must be `"TA0100"`; `tactic_name` must be `"Collection (ICS)"` | YES — both assertions break |

### tests/bc_2_16_story114_arp_tests.rs

| Test function | File:Lines (approx) | Current assertion | Must become | Break? |
|--------------|--------------------|--------------------|-------------|--------|
| `test_t0830_t1557002_resolve_in_mitre_catalog` (AC-011) | bc_2_16_story114_arp_tests.rs:33–62 | `Some(MitreTactic::LateralMovement)` for T0830 at line ~59 | `Some(MitreTactic::IcsCollection)` | YES — tactic assertion breaks |

### src/reporter/terminal.rs

No direct tactic-ID or variant assertions. The terminal reporter uses `technique_name` and
`all_tactics_in_report_order` for grouped output headers. Behavioral impact:

- In `--mitre` grouped mode, ICS techniques that previously appeared under "Discovery" (T0888, T0846)
  will now appear under "Discovery (ICS)". T0830 will appear under "Collection (ICS)" instead of
  "Lateral Movement". T0885 will appear under "Command and Control (ICS)" instead of
  "Command and Control".
- `src/reporter/terminal.rs:335` — comment cites `T1692.001, T0836` as the multi-ID example.
  No assertion about tactic names at this line. No change required.
- This is a user-visible grouped-output change for any finding carrying T0888 (Modbus recon),
  T0830 (ARP spoof), T0846 (staged), or T0885 (staged). T0888 and T0830 are the emitted cases.

### src/reporter/json.rs

| File:Line | Current content | Must change |
|-----------|----------------|-------------|
| `src/reporter/json.rs:26` | Comment: "(T0888, T1692.001, T0836, T0835, T0831, T0814, T0806) confirmed valid and active" | No functional change needed — this is a comment listing IDs, not asserting tactic. Cosmetic update optional. |

No functional changes required in `json.rs` — the enrichment logic calls `technique_tactic_id`
transparently; the corrected tactic IDs flow through automatically.

### .factory/specs/behavioral-contracts — BCs requiring version bump

| BC file | Path | Current version | What changes | Bump to |
|---------|------|----------------|--------------|---------|
| **BC-2.11.035** | `ss-11/BC-2.11.035.md` | v1.0 | EC-010 must change: T0830 → `tactic_id: "TA0100"`, `tactic_name: "Collection (ICS)"`. The Catalog Extension table (lines 57–67) must add 3 new rows: IcsDiscovery→TA0102, IcsCollection→TA0100, IcsCommandAndControl→TA0101. | v1.1 |
| **BC-2.10.007** | `ss-10/BC-2.10.007.md` | v1.8 | Postcondition 2 tactic assignments for T0846→IcsDiscovery, T0888→IcsDiscovery, T0830→IcsCollection, T0885→IcsCommandAndControl. EC-002, EC-004, EC-009 tactic names update. Invariant 3 text update (no longer "merge by name"). Description text update. | v1.9 |
| **BC-2.10.003** | `ss-10/BC-2.10.003.md` | v1.4 | Slice length 17→20. Postcondition 1 updates. Element [16] was IcsImpact; now elements [16], [17], [18], [19] must be specified. Edge cases EC-001 and EC-006 update. | v1.5 |
| **BC-2.10.002** | `ss-10/BC-2.10.002.md` | v1.5 | New ICS Display strings for the 3 new variants (Discovery (ICS), Collection (ICS), Command and Control (ICS)). Postconditions extended. Invariant 4 (length 17→20). | v1.6 |
| **BC-2.16.004** | `ss-16/BC-2.16.004.md` | v1.7 | Invariant 4 tactic anchor: T0830 maps to `MitreTactic::IcsCollection` (NOT `MitreTactic::LateralMovement`). ADR-008 Decision 6 reference must be noted as superseded for the T0830 tactic assignment. | v1.8 |

### docs/ — documentation changes

| File:Line | Current content | Must change |
|-----------|----------------|-------------|
| `docs/demo-evidence/STORY-129/evidence-report.md:35` | "T0888 → `tactic_id: "TA0007"`, `tactic_name: "Discovery"`" | `tactic_id: "TA0102"`, `tactic_name: "Discovery (ICS)"` — this is captured demo evidence; update the description to reflect the corrected values |
| `docs/demo-evidence/STORY-129/evidence-report.md` (overall) | References TA0007 for T0888 | Update description |

The ADR files (`docs/adr/0006-multi-technique-finding-attribution.md`) reference T0888 only for
technique-ID assignment (not tactic). No tactic assertions in those files. No change required.

`docs/superpowers/plans/2026-04-13-mitre-attack-mapping.md` is a historical design plan, not
a normative spec. It predates the ICS split. No change required (it is not consumed by any test).

### .factory/specs/architecture/decisions/ADR-008

| File:Lines | Current content | Required action |
|-----------|----------------|-----------------|
| `ADR-008-arp-link-layer-integration.md:472–476` | Decision 6 states merge-by-name policy; T0830 maps to `MitreTactic::LateralMovement` via merge-by-name; "no separate ICS enum variant exists or is needed" | Decision 6 must be superseded or annotated. T0830 is Collection (TA0100) per MITRE, not Lateral Movement. ADR-008 v2.0 must record: Decision 6 tactic anchor for T0830 is CORRECTED from LateralMovement to IcsCollection per f5-ics-tactic-id-validation.md findings. |

### .factory/holdout-scenarios/

| File:Lines | Current content | Impact |
|-----------|----------------|--------|
| `HS-INDEX.md:507–510` | Cites T0830 → `MitreTactic::LateralMovement` and "merge-by-name per mitre.rs convention" | Must update to IcsCollection |
| `HS-INDEX.md:563–567` | HS-W43-001 through HS-W43-005 cite T0830 as LateralMovement in rubrics/expected outputs | Update LateralMovement references to IcsCollection |
| `HS-INDEX.md:634` | "T0830 + T1557.002 findings" — does not assert tactic name directly | No change needed |
| `wave-40-44-holdout.md:73` | "**MITRE:** T0830 (LateralMovement), T1557.002 (CredentialAccess)" | Must become: T0830 (IcsCollection/TA0100) |
| `wave-40-44-holdout.md:347` | "`technique_info` for T0830 resolves to a non-empty entry (LateralMovement tactic arm per ADR-008 Decision 6)" | Must change to IcsCollection |

The holdout expected-output changes **affect rubric descriptions, not pass/fail binary outcomes**
(holdouts check for technique-ID presence, not tactic-variant names in most cases). Review each
holdout carefully; those that explicitly assert `tactic_name: "Lateral Movement"` on T0830 will
fail the holdout if the holdout runs JSON output.

### .factory/convergence/ and .factory/code-delivery/

The convergence files for STORY-114 (`STORY-114-step45.md`) and code-delivery PR descriptions
reference T0830 as LateralMovement. These are historical records — update for accuracy but they
do not gate tests.

---

## Section 4 — Ripple Assessment

### Tests that WILL BREAK (currently asserting wrong values):

1. **`tests/reporter_json_tests.rs:test_BC_2_11_035_ec010_ics_lateral_movement`** (lines ~1071–1104)
   - Currently asserts `tactic_id: "TA0008"` and `tactic_name: "Lateral Movement"` for T0830.
   - After fix: must assert `tactic_id: "TA0100"` and `tactic_name: "Collection (ICS)"`.
   - This test encodes the known-wrong value. It WILL fail as soon as the enum is changed.

2. **`tests/bc_2_16_story114_arp_tests.rs` (AC-011 function, ~line 59)**
   - Currently asserts `Some(MitreTactic::LateralMovement)` for T0830.
   - After fix: must assert `Some(MitreTactic::IcsCollection)`.

3. **`tests/mitre_tests.rs:test_all_tactics_length_is_16`** (line 108)
   - Currently asserts length 17.
   - After fix: must assert length 20.

4. **`tests/mitre_tests.rs:test_all_tactics_no_duplicates`** (line 183)
   - Currently asserts `unique.len() == 17`.
   - After fix: must assert `unique.len() == 20`.

5. **`tests/mitre_tests.rs:test_all_tactics_all_variants_present`** (lines ~197–218)
   - Expected HashSet does not include `IcsDiscovery`, `IcsCollection`, `IcsCommandAndControl`.
   - After fix: these three variants must be added to the expected set.

6. **`tests/mitre_tests.rs:test_technique_tactic_correct_assignments`** (lines ~347, ~357)
   - T0846 asserts `MitreTactic::Discovery`; T0888 asserts `MitreTactic::Discovery`.
   - After fix: both must assert `MitreTactic::IcsDiscovery`.

7. **`src/mitre.rs` — compile-time breakage from `#[non_exhaustive]` within-crate exhaustive match**
   - Adding new variants without updating the `technique_tactic_id` match (line 219) produces a
     compile error. The exhaustive match enforces VP-007 coverage. This is intentional and desirable.
   - Similarly, `Display` impl at line 72 is an exhaustive match — adding variants without arms
     produces a compile error. Good: the compiler enforces completeness.

### Tests that will NOT break (but are related):

- `vp007_catalog_drift_guard` (src/mitre.rs:461–558): This test sweeps all 25 seeded IDs and
  checks that `technique_tactic_id` returns `Some` for each. After adding the 3 new variants and
  their `technique_tactic_id` arms, the test continues to pass. The SEEDED_TECHNIQUE_IDS list and
  SEEDED_TECHNIQUE_ID_COUNT do NOT change (the tactic-variant change does not add or remove catalog
  entries). The test also checks that every arm in `technique_info` is mirrored in SEEDED_TECHNIQUE_IDS
  — this invariant is unaffected.

- `test_all_emitted_ids_resolve` (mitre_tests.rs:374–421): Checks that `technique_name` and
  `technique_tactic` both return `Some` for the 13 emitted IDs. After fix, they still return Some
  (new variants are valid). Does not assert specific tactic values. Will not break.

- `test_ics_v19_remap_t1692_sub_techniques_are_pinned` (mitre_tests.rs:505–541): Asserts
  IcsImpairProcessControl for T1692.001 and T1692.002. Not affected. Will not break.

### User-visible output changes:

- **JSON output (`--json`):** For findings carrying T0888, `tactic_id` changes from `"TA0007"` to
  `"TA0102"` and `tactic_name` changes from `"Discovery"` to `"Discovery (ICS)"`. For findings
  carrying T0830, `tactic_id` changes from `"TA0008"` to `"TA0100"` and `tactic_name` from
  `"Lateral Movement"` to `"Collection (ICS)"`. Both are breaking changes to the JSON shape for
  consumers of the per-finding `mitre_attack` array.

- **Terminal `--mitre` grouped output:** Findings with T0888 move from the "Discovery" group header
  to the "Discovery (ICS)" group header. Findings with T0830 move from "Lateral Movement" to
  "Collection (ICS)". These are new tactic section headers in the grouped terminal output — they
  will appear only if a finding carries the relevant technique.

- **Canonical-emitted-IDs count:** Unchanged. Still 17 emitted IDs. No count changes.

### Holdout expected-output changes:

- HS-W43-001 through HS-W43-005 and wave-40-44-holdout.md assertions on T0830's tactic name must
  be updated. Any holdout that exercises `--json` output and checks `tactic_name` or `tactic_id` for
  T0830 will fail if the expected value is not updated from "Lateral Movement" / "TA0008" to
  "Collection (ICS)" / "TA0100".
- HS-009 does not assert tactic IDs explicitly. No change needed for HS-009 pass/fail rubric.

---

## Section 5 — BC Impact and Input-Hash Recompute

### BCs requiring version bump:

| BC | Path | Version bump | Rationale |
|----|------|-------------|-----------|
| BC-2.11.035 | `.factory/specs/behavioral-contracts/ss-11/BC-2.11.035.md` | 1.0 → 1.1 | EC-010 tactic correction; Catalog Extension table gains 3 rows |
| BC-2.10.007 | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md` | 1.8 → 1.9 | Postcondition 2 tactic assignments for T0846/T0888/T0830/T0885 |
| BC-2.10.003 | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md` | 1.4 → 1.5 | Slice length 17→20; new ICS elements |
| BC-2.10.002 | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md` | 1.5 → 1.6 | New ICS Display strings for 3 new variants |
| BC-2.16.004 | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.7 → 1.8 | Invariant 4 tactic anchor: T0830 → IcsCollection |

### Stories whose `inputs:` include a changed BC (input-hash recompute required):

BC-2.16.004 is in the `inputs:` list of STORY-114
(`.factory/stories/STORY-114.md` — trace: BC-2.16.004 is the D1 spoof-detection BC that STORY-114
implements). After bumping BC-2.16.004 to v1.8, run:
```
bin/compute-input-hash --write .factory/stories/STORY-114.md
```

BC-2.10.007 and BC-2.10.005 are inputs to HS-009 (holdout scenario) which traces to
`STORY-071.md`. Check whether `STORY-071.md` has an `inputs:` frontmatter that includes these BCs:
```
grep -n "inputs:" .factory/stories/STORY-071.md
```

BC-2.11.035 is traced by STORY-129. Check:
```
grep -n "inputs:" .factory/stories/STORY-129.md
```

Run `bin/compute-input-hash --scan` from the repo root (with `.factory/` mounted) after all BC
updates to identify all stale story hashes.

---

## Section 6 — Correction to ADR-008 Decision 6

ADR-008 Decision 6 (`.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md`,
lines ~459–485) enshrined the merge-by-name policy for T0830. That decision is now superseded by
the validated correctness finding: T0830 is ICS Collection (TA0100), not Lateral Movement.

The ADR version must be bumped (to v2.1 or higher) with a Decision 6 amendment:

> **Amendment (2026-06-23):** T0830's tactic is corrected from `MitreTactic::LateralMovement`
> (merge-by-name policy, TA0008 Enterprise) to `MitreTactic::IcsCollection` (ICS matrix,
> TA0100). The merge-by-name policy is superseded for all ICS techniques that collide with
> Enterprise tactic names. f5-ics-tactic-id-validation.md (VALIDATED) is the authoritative
> source. The tactic name for T0830 in the ICS matrix is "Collection" (TA0100), not
> "Lateral Movement". BC-2.16.004 Invariant 4 is updated in v1.8.

---

## Section 7 — Summary of All Files Requiring Changes

### Code changes (src/):

| File | Change summary |
|------|----------------|
| `src/mitre.rs` | Add 3 enum variants; add 3 Display arms; extend `all_tactics_in_report_order` by 3; remap 4 technique entries; add 3 `technique_tactic_id` arms; correct wrong comment at lines 145–148 and update EMITTED_IDS comment for T0830 |

### Test changes (tests/):

| File | Test functions requiring update |
|------|--------------------------------|
| `tests/mitre_tests.rs` | `test_all_tactics_length_is_16`, `test_all_tactics_ics_at_end`, `test_all_tactics_no_duplicates`, `test_all_tactics_all_variants_present`, `test_technique_tactic_correct_assignments` |
| `tests/reporter_json_tests.rs` | `test_BC_2_11_035_ec010_ics_lateral_movement` |
| `tests/bc_2_16_story114_arp_tests.rs` | AC-011 function (T0830 tactic assertion) |

### Spec changes (.factory/specs/):

| File | Change summary |
|------|----------------|
| `.factory/specs/behavioral-contracts/ss-11/BC-2.11.035.md` | EC-010 correction; Catalog Extension table adds 3 rows; version 1.0→1.1 |
| `.factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md` | Postcondition 2 tactic assignments; EC-002/004/009; version 1.8→1.9 |
| `.factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md` | Slice length 17→20; new ICS elements; version 1.4→1.5 |
| `.factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md` | New ICS Display strings; version 1.5→1.6 |
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.004.md` | Invariant 4 tactic anchor correction; version 1.7→1.8 |
| `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | Decision 6 amendment; version bump |

### Documentation changes (docs/):

| File | Change summary |
|------|----------------|
| `docs/demo-evidence/STORY-129/evidence-report.md:35` | T0888 tactic_id/tactic_name description updated |

### Factory artifact changes (.factory/):

| File | Change summary |
|------|----------------|
| `.factory/holdout-scenarios/HS-INDEX.md:507–510, 563–567` | T0830 tactic label updated to IcsCollection |
| `.factory/feature/wave-holdout-scenarios/wave-40-44-holdout.md:73, 347` | T0830 tactic label updated |
| Input-hash recompute for stories whose `inputs:` include changed BCs (STORY-114 minimum; run `--scan` to catch all) |

---

## Appendix — Wrong TA-IDs Currently Emitted

This table shows the exact wrong-vs-correct tactic_id values that wirerust currently emits in
JSON output for ICS techniques mapped to Enterprise variants:

| Technique | Name | Currently emitted `tactic_id` | Correct `tactic_id` | Currently emitted `tactic_name` | Correct `tactic_name` | Emitted? |
|-----------|------|------------------------------|---------------------|--------------------------------|-----------------------|----------|
| T0846 | Remote System Discovery | TA0007 (Enterprise Discovery) | **TA0102** (ICS Discovery) | "Discovery" | "Discovery (ICS)" | No (seeded-only) |
| T0888 | Remote System Information Discovery | TA0007 (Enterprise Discovery) | **TA0102** (ICS Discovery) | "Discovery" | "Discovery (ICS)" | **YES** |
| T0885 | Commonly Used Port | TA0011 (Enterprise C2) | **TA0101** (ICS C2) | "Command and Control" | "Command and Control (ICS)" | No (seeded-only) |
| T0830 | Adversary-in-the-Middle | TA0008 (Enterprise LateralMovement) | **TA0100** (ICS Collection) | "Lateral Movement" | "Collection (ICS)" | **YES** |
