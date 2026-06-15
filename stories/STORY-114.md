---
document_type: story
story_id: STORY-114
epic_id: E-16
version: "1.2"
version_note: "1.2 (2026-06-15): D-074 back-propagation — AC-006 extended with threshold-0 rejection requirement and test_cli_arp_spoof_threshold_0_rejected; EC-014 added (BC-2.16.012 EC-004 / BC-2.16.012 v1.3 PC2). 1.1 (2026-06-14): F3-convergence Pass-25 Slice-C — de-pinned 4x HS-008-*.md:75 line citations to concept anchor 'HS-008 Verification Approach step 1'; input-hash will be recomputed by orchestrator (--write)"
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: f3
points: 13
priority: P0
depends_on: [STORY-113]
blocks: [STORY-115]
behavioral_contracts:
  - BC-2.16.004
  - BC-2.16.012
  - BC-2.16.014
verification_properties: [VP-007]
tdd_mode: strict
target_module: analyzer/arp
subsystems: [SS-16]
estimated_days: 5
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: BC-2.16.004 v1.5, BC-2.16.012 v1.3, BC-2.16.014 v1.5 — authored 2026-06-12 (BC-2.16.012 updated to v1.3 per D-074 cosmetic sync 2026-06-15)
# VP-007 5-part atomic update: SEEDED 23→25 / EMITTED 15→17; vp007_catalog_drift_guard must pass
# D-069 supersedes D-067: IcsImpact Display = "Impact (ICS)" is canonical; src/mitre.rs:91 stays unchanged; HS-008 stays "Impact (ICS)". See D-069 supersession note below.
# PLANNED: BC-2.10.005 and BC-2.10.008 carry "PLANNED — implemented in STORY-114; current code 23/15" markers until this story merges
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.004.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.007.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.012.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.014.md
  - .factory/specs/verification-properties/vp-007-mitre-technique-id-format.md
input-hash: "1325d69"
---

# STORY-114: D1 ARP Spoof Escalation + GARP-that-Conflicts (D2+D1) + MITRE Attribution + VP-007 5-Part Atomic Update

## Narrative

- **As a** ICS/OT security analyst using wirerust
- **I want** `ArpAnalyzer::process_arp` to emit D1 ARP spoof findings (MEDIUM on first rebind, HIGH when rebind_count >= threshold within the flap window), implement the GARP-that-conflicts escalation rule (GARP upgrades to MEDIUM + D1 finding co-emitted), attach MITRE techniques T0830 and T1557.002 to D1/D2/D12 findings, and have the `--arp-spoof-threshold` CLI flag override the default threshold — with all of this co-committed with the VP-007 5-part atomic update to `src/mitre.rs`
- **So that** ARP cache-poisoning attacks are detected and escalated with proper severity and MITRE attribution, and the MITRE technique catalog remains consistent (vp007_catalog_drift_guard passes)

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.004 | ARP Spoof Detection — IP→MAC Rebind Emits MEDIUM then HIGH Finding |
| BC-2.16.012 | --arp-spoof-threshold Overrides SPOOF_REBIND_ESCALATION_DEFAULT |
| BC-2.16.014 | GARP-That-Conflicts Upgrades to MEDIUM and Triggers D1 Spoof Finding |

## D-069 Supersession Note (supersedes D-067)

**D-069 is the authoritative decision.** D-067 carry-forward obligations F3-OBL-STORY114-001, -002, and -003 are REVOKED as of D-069. The prior obligations (IcsImpact Display change, HS-008 update, and associated Display-string tests) are NO LONGER required. Specifically:

- **F3-OBL-STORY114-001 REVOKED:** `src/mitre.rs:91` reads `MitreTactic::IcsImpact => "Impact (ICS)"`. This is CORRECT and MUST NOT be changed. BC-2.10.002 was revised to v1.5 confirming `"Impact (ICS)"` as the canonical Display string (distinct from the Enterprise `"Impact"` tactic, preserving the ICS matrix identity).
- **F3-OBL-STORY114-002 REVOKED:** `.factory/holdout-scenarios/HS-008-*.md` (HS-008 Verification Approach step 1) already reads `"Impact (ICS)"`. This is correct and MUST NOT be changed. The F5 DNP3 tests `test_ics_impact_display_distinct_from_impact` and `test_reporter_renders_distinct_impact_sections` correctly assert `"Impact (ICS)" != "Impact"` and remain green.
- **F3-OBL-STORY114-003 REVOKED:** The Display-string-equals-"Impact" assertion tests are NOT required. Any test asserting `format!("{}", MitreTactic::IcsImpact) == "Impact"` would be WRONG under D-069.

**What this story DOES require (D-069 aligned):** The VP-007 5-part atomic update (AC-011, AC-012) and the ARP spoof/escalation/GARP-conflict detection (AC-001 through AC-010, AC-016) are unchanged. The only obligation from the old OBL-003 that survives is the enum-variant-distinctness test: `MitreTactic::Impact != MitreTactic::IcsImpact` as enum values (AC-014) — this is still correct and harmless because it tests enum identity, not Display strings.

## VP-007 5-Part Atomic Update

All five sites below MUST be updated in a single commit. `vp007_catalog_drift_guard` (`cargo test vp007_catalog_drift_guard`) MUST pass before the PR is opened.

> **CC-003 sequencing (VP-007):** This story's VP-007 obligation is CC-003 (ARP catalog lock: SEEDED 23→25 / EMITTED 15→17). CC-003 requires CC-002 (DNP3 catalog lock, STORY-109: SEEDED 21→23 / EMITTED 13→15) to be discharged first — SEEDED=23 must be the locked baseline before this story's 23→25 lock-break/re-lock cycle. STORY-109 must be merged before STORY-114.

### (a) Functional code edits (required for vp007_catalog_drift_guard to pass)

| Site | Location | Change |
|------|----------|--------|
| `technique_info` match arms | `src/mitre.rs` lines 178–179 | Add before `_ => return None`: `"T0830" => ("Adversary-in-the-Middle", MitreTactic::LateralMovement)` and `"T1557.002" => ("Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess)` |
| `SEEDED_TECHNIQUE_IDS` array body | `src/mitre.rs` lines 305–333 | Add `"T0830"` and `"T1557.002"` as new entries |
| `SEEDED_TECHNIQUE_ID_COUNT` constant | `src/mitre.rs` line 341 | 23 → 25 |
| `EMITTED_IDS` array in `kani_proofs` | `src/mitre.rs` lines 221–240 | Add `"T0830"` and `"T1557.002"` entries |

### (b) Stale-count comment updates (do not affect test pass/fail)

| Site | Location | Change |
|------|----------|--------|
| `kani_proofs` module doc — seeded-ID count | `src/mitre.rs` line 204 ("finite (23)") | 23 → 25 |
| `kani_proofs` `SEEDED_IDS` const comment | `src/mitre.rs` line 212 ("All 23 seeded IDs") | 23 → 25 |
| `kani_proofs` `EMITTED_IDS` const comment | `src/mitre.rs` line 218 | "6 Enterprise + 7 ICS + 2 STORY-109 = 15 emitted IDs" → "6 Enterprise + 7 ICS + 2 STORY-109 + 2 ARP (STORY-114) = 17 emitted IDs" |
| `SEEDED_TECHNIQUE_IDS` doc comment line 301 | `src/mitre.rs` line 301 | "Post-F2 (STORY-100): 11 Enterprise + 10 ICS = 21 total" → "Post-F2 (STORY-100): 11 Enterprise + 10 ICS = 21 total (pre-STORY-109 subtotal)" |
| `SEEDED_TECHNIQUE_IDS` doc comment line 302 | `src/mitre.rs` line 302 | Add ARP addendum: "+ 2 ARP (STORY-114): T0830 (ICS LateralMovement) + T1557.002 (Enterprise CredentialAccess) = 25 total" |
| `SEEDED_TECHNIQUE_ID_COUNT` doc comment | `src/mitre.rs` line 339 | "currently 23: 21 post-F2/STORY-100 + 2 STORY-109 additions" → "currently 25: 21 post-F2/STORY-100 + 2 STORY-109 + 2 ARP/STORY-114 additions" |

## Acceptance Criteria

### AC-001 (traces to BC-2.16.004 postcondition 1.a–1.e — D1 first rebind emits MEDIUM)
When `frame.sender_ip` is already in the binding table with a different MAC (first rebind),
`process_arp` emits one `Finding` with `confidence: MEDIUM`, `finding_type: Anomaly`,
`mitre_techniques: ["T0830", "T1557.002"]`. `rebind_count` is incremented (Step 1).
`first_rebind_ts` is set to `timestamp_secs` if unset (Step 2). Escalation condition fails
(rebind_count=1 < default threshold=3) so MEDIUM is emitted (Step 3, case d). MAC is updated
to frame.sender_mac (Step 4).
- **Test:** `test_d1_first_rebind_emits_medium()`

### AC-002 (traces to BC-2.16.004 postcondition 1.c — D1 escalates to HIGH at threshold)
When `rebind_count >= SPOOF_REBIND_ESCALATION_DEFAULT = 3` AND
`timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS = 60` AND
`spoof_high_emitted == false`, the emitted finding has `confidence: HIGH` (Likely).
`spoof_high_emitted` is set to `true`. Subsequent rebinds in the same window emit MEDIUM
(one-shot guard, AC-003).
- **Test:** `test_d1_escalates_to_high_at_threshold()` (3 rebinds within 60s → HIGH on 3rd)

### AC-003 (traces to BC-2.16.004 postcondition 4 — one-shot HIGH guard)
After `spoof_high_emitted == true`, subsequent rebinds within the same flap window emit
MEDIUM findings (not HIGH). The one-shot guard prevents repeated HIGH findings per window.
- **Test:** `test_d1_high_guard_prevents_second_high()`

### AC-004 (traces to BC-2.16.004 postcondition 5 — flap window reset)
After `ARP_FLAP_WINDOW_SECS = 60` seconds have elapsed since `first_rebind_ts`, the window
resets: `rebind_count → 0`, `first_rebind_ts → None`, `spoof_high_emitted → false`. The next
rebind after reset is treated as the first rebind (MEDIUM).
- **Test:** `test_d1_flap_window_reset()`

### AC-005 (traces to BC-2.16.004 EC-008 — threshold=1: HIGH on first rebind)
With `--arp-spoof-threshold 1`, the first rebind emits HIGH immediately: Step 1 sets
rebind_count=1, Step 2 sets first_rebind_ts=timestamp_secs (elapsed=0), Step 3 evaluates
`rebind_count=1 >= threshold=1 AND elapsed=0 <= 60 AND !spoof_high_emitted` → HIGH.
- **Test:** `test_d1_threshold_1_high_on_first_rebind()`

### AC-006 (traces to BC-2.16.012 postcondition 1/2/EC-004 — --arp-spoof-threshold wiring; 0 rejected)
`ArpAnalyzer::new(spoof_threshold, storm_rate)` uses the `spoof_threshold` parameter in
D1 escalation logic. `src/cli.rs` declares `#[arg(long, default_value_t = 3)] arp_spoof_threshold: u32`
on `Commands::Analyze`. `src/main.rs` passes `args.arp_spoof_threshold` to `ArpAnalyzer::new`.
When flag is absent, default 3 applies. `--arp-spoof-threshold 0` MUST be rejected at CLI
parse time with a fail-fast error (`--arp-spoof-threshold must be >= 1 (got 0)`); 0 is not
clamped (D-074 / BC-2.16.012 EC-004). ARP comparisons are inclusive (`>=`), so a threshold of
0 would trigger HIGH escalation on the very first rebind unconditionally.
**Standalone-compile note:** `--arp-storm-rate` does not
exist in STORY-114 (that flag is STORY-115's deliverable); the `storm_rate` argument at the
`src/main.rs` call site MUST be `ARP_STORM_RATE_DEFAULT` (= 50) until STORY-115 wires
`args.arp_storm_rate`. STORY-115 replaces this constant with `args.arp_storm_rate`.
- **Test:** `test_cli_arp_spoof_threshold_parsed()`, `test_cli_arp_spoof_threshold_default_3()`, `test_cli_arp_spoof_threshold_0_rejected()`

### AC-007 (traces to BC-2.16.014 postcondition 1 — GARP-that-conflicts: GARP upgrades to MEDIUM)
When `is_gratuitous_arp(frame) == true` AND `bindings[sender_ip].mac != frame.sender_mac`
(binding conflict), the GARP finding severity is upgraded from LOW to MEDIUM. The GARP
finding carries `mitre_techniques: ["T0830", "T1557.002"]`.
- **Test:** `test_garp_conflicts_garp_finding_upgrades_to_medium()`

### AC-008 (traces to BC-2.16.014 postcondition 2 — GARP-that-conflicts: D1 finding also emitted)
For the same frame satisfying GARP-that-conflicts, a separate D1 ARP Spoof finding is ALSO
emitted. Its severity is determined by BC-2.16.004 Steps 1–3 (rebind_count incremented first;
HIGH if conditions met, MEDIUM otherwise). Two distinct findings are returned from `process_arp`
for this single frame.
- **Test:** `test_garp_conflicts_d1_also_emitted()`

### AC-009 (traces to BC-2.16.014 EC-004 — GARP-that-conflicts at HIGH threshold)
When the GARP-that-conflicts is the 3rd rebind within 60s (default threshold=3), the D1
finding is HIGH, while the GARP finding is MEDIUM. Two findings total.
- **Test:** `test_garp_conflicts_d1_high_at_threshold()`

### AC-010 (traces to BC-2.16.014 postcondition 6 — GARP without conflict: LOW only)
A GARP frame where `sender_ip` is NOT in the binding table (no conflict) produces one GARP
finding at LOW confidence. No D1 finding. This is unchanged from STORY-113 behavior.
- **Test:** `test_garp_no_conflict_low_only()` (regression)

### AC-011 (traces to BC-2.16.004 — MITRE: T0830 mapped to LateralMovement, T1557.002 to CredentialAccess)
After the VP-007 atomic update, `technique_info("T0830")` returns `("Adversary-in-the-Middle", MitreTactic::LateralMovement)` and `technique_info("T1557.002")` returns `("Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess)`. Both are resolvable via the public `technique_name` and `technique_tactic` functions.
- **Test:** `test_t0830_and_t1557_002_resolves_in_catalog()`

### AC-012 (traces to VP-007 — vp007_catalog_drift_guard passes: SEEDED=25, EMITTED=17)
After the 5-part atomic update, `SEEDED_TECHNIQUE_ID_COUNT == 25` and `SEEDED_TECHNIQUE_IDS.len() == 25`. `kani_proofs::EMITTED_IDS.len() == 17`. `vp007_catalog_drift_guard` unit test passes (cargo test vp007_catalog_drift_guard). Derived arithmetic: 25 − 17 == 8 catalogue-only IDs (unchanged).
- **Test (layered):**
  - `vp007_catalog_drift_guard` (in-crate unit test in `src/mitre.rs`)
  - `test_vp007_seeded_25_emitted_17` (integration test via public API: `technique_name` and `technique_tactic` resolve for all 25 seeded IDs, non-empty, non-"Unknown")

### AC-013 (traces to BC-2.10.002 v1.5 — IcsImpact Display is "Impact (ICS)" — VERIFY ONLY)
**D-069 canonical:** `MitreTactic::IcsImpact` Display implementation returns `"Impact (ICS)"` (NOT bare `"Impact"`). `src/mitre.rs:91` is already correct. No code change required. If a test for this exists, it MUST assert `format!("{}", MitreTactic::IcsImpact) == "Impact (ICS)"`.
The F5 DNP3 tests `test_ics_impact_display_distinct_from_impact` and `test_reporter_renders_distinct_impact_sections` already assert the correct behavior and MUST NOT be touched or contradicted by STORY-114.
- **No new test required for this AC** — existing F5 DNP3 tests cover it. STORY-114 must not introduce a conflicting test.

### AC-014 (traces to BC-2.10.002 v1.5 — enum-variant distinctness)
`MitreTactic::Impact` and `MitreTactic::IcsImpact` are distinct enum variants with distinct Display strings (`"Impact"` vs `"Impact (ICS)"` respectively). A test verifies that the two variants are not equal as enum values (`MitreTactic::Impact != MitreTactic::IcsImpact`). Under D-069 they also have distinct Display strings, so no merge-by-name collision occurs.
- **Test:** `test_impact_vs_ics_impact_variants_distinct()`

### AC-015 (traces to BC-2.10.002 v1.5 / D-069 — HS-008 already correct)
`.factory/holdout-scenarios/HS-008-*.md` (HS-008 Verification Approach step 1) already reads `"Impact (ICS)"`. This is canonical under D-069 and MUST NOT be changed by STORY-114. No HS-008 file modification is required in this story.
- **No new test required for this AC** — HS-008 (HS-008 Verification Approach step 1) holdout evaluation at Phase 4 exercises it directly.

> **Consumer References (verify-only, non-owned):** AC-013, AC-014, and AC-015 trace to BC-2.10.002 v1.5 (SS-10 MITRE tactic Display and enum-variant contracts). BC-2.10.002 is owned by STORY-071 and does NOT appear in this story's `behavioral_contracts:` frontmatter or `inputs:` list — STORY-114 is a verify-only consumer under D-069 adjudication, not the contract owner. No code change to `src/mitre.rs:91` is required or permitted. This verify-only consumer relationship does not affect the input-hash computation.

### AC-016 (traces to BC-2.16.004 — D1 finding evidence includes old MAC and new MAC)
The D1 spoof finding's evidence includes: the conflicting IP, the old MAC (from binding table before update), and the new MAC (from `frame.sender_mac`). These are present in the finding's evidence field regardless of MEDIUM or HIGH severity.
- **Test:** `test_d1_finding_evidence_contains_ips_and_macs()`

## BC-2.16.007 Cross-Story Extension (D12 MITRE)

STORY-113 (wave 42) emits D12 mismatch findings with `mitre_techniques: []` (catalog not seeded yet). STORY-114 (wave 43) back-fills `mitre_techniques: ["T0830", "T1557.002"]` on D12 findings, co-committed with the VP-007 5-part atomic update — this is the only wave at which T0830 and T1557.002 enter `src/mitre.rs`. BC-2.16.007 contains an explicit cross-story delivery note authorising this pattern (analogous to BC-2.16.010's storm_findings wiring deferred to STORY-115).

STORY-114 is the primary owner of D12 MITRE attribution. BC-2.16.007 is therefore a declared input of this story (see `inputs:` frontmatter).

### AC-017 (traces to BC-2.16.007 postcondition 1 — D12 mismatch carrying MITRE after catalog seeding)
After the VP-007 5-part atomic update is applied, a D12 mismatch finding (outer `eth_src_mac` ≠ ARP `sender_mac`) carries `mitre_techniques: ["T0830", "T1557.002"]` AND retains `confidence: MEDIUM`, `finding_type: Anomaly`, and evidence fields `eth_mac`, `arp_sender_mac`, `sender_ip` per BC-2.16.007 postcondition 1.
- **Test:** `test_d12_mismatch_carries_mitre_after_catalog()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `process_arp` D1 emission + escalation logic | `src/analyzer/arp.rs` | Pure core (stateful) |
| `process_arp` GARP-that-conflicts escalation | `src/analyzer/arp.rs` | Pure core (stateful) |
| `SPOOF_REBIND_ESCALATION_DEFAULT: u32 = 3` | `src/analyzer/arp.rs` | Constant |
| `ARP_FLAP_WINDOW_SECS: u32 = 60` | `src/analyzer/arp.rs` | Constant (shared with D3 storm window) |
| `technique_info` match arms (T0830, T1557.002) | `src/mitre.rs` | Pure core |
| `SEEDED_TECHNIQUE_IDS` array | `src/mitre.rs` | Constant (25 entries) |
| `SEEDED_TECHNIQUE_ID_COUNT` | `src/mitre.rs` | Constant = 25 |
| `EMITTED_IDS` in `kani_proofs` | `src/mitre.rs` `#[cfg(kani)]` | Kani local constant (17 entries) |
| `MitreTactic::IcsImpact` Display = "Impact (ICS)" | `src/mitre.rs:91` | Display impl — already correct; DO NOT change (D-069) |
| `--arp-spoof-threshold` CLI flag | `src/cli.rs` | Effectful shell (CLI) |
| `src/main.rs` — `ArpAnalyzer::new(args.arp_spoof_threshold, ARP_STORM_RATE_DEFAULT)` | `src/main.rs` | Effectful shell (standalone-compile: `storm_rate` = const until STORY-115 wires `args.arp_storm_rate`) |

Architecture section references: `architecture/module-decomposition.md` (SS-16 C-23 ArpAnalyzer), arp-architecture-delta.md §5.

## Forbidden Dependencies

- `src/mitre.rs` MUST NOT be modified before this story (per ADR-008 §5.0: "Do NOT touch src/mitre.rs before STORY-114"). The pre-ARP baseline has SEEDED=23, EMITTED=15; all five functional sites are updated atomically in this story.
- The `MitreTactic` enum itself requires NO new variants for T0830 or T1557.002. `LateralMovement` and `CredentialAccess` variants already exist (ADR-008 Decision 6).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IP seen for first time (no binding) | No D1 finding; binding initialized |
| EC-002 | Same IP, same MAC (repeated) | No D1 finding; no rebind |
| EC-003 | First rebind (MAC A → B) | MEDIUM D1 finding; rebind_count=1 |
| EC-004 | Second rebind within 60s (MAC B → C) | MEDIUM D1 finding; rebind_count=2 |
| EC-005 | Third rebind within 60s (reaches threshold=3) | HIGH D1 finding; spoof_high_emitted=true |
| EC-006 | Fourth rebind within 60s (high guard active) | MEDIUM D1 finding (not HIGH) |
| EC-007 | Rebind after 61s (window expired) | Window reset; MEDIUM finding; rebind_count=1 |
| EC-008 | threshold=1: first rebind | HIGH immediately (elapsed=0 ≤ 60; rebind_count=1 ≥ threshold=1) |
| EC-009 | GARP, no binding conflict | GARP LOW only (regression from STORY-113) |
| EC-010 | GARP + binding conflict, first rebind | GARP MEDIUM + D1 MEDIUM |
| EC-011 | GARP + binding conflict, 3rd rebind within 60s | GARP MEDIUM + D1 HIGH |
| EC-012 | T0830 not in catalog before STORY-114 | technique_info("T0830") returned None before; returns Some(...) after 5-part update |
| EC-013 | IcsImpact Display (D-069 canonical) | "Impact (ICS)" — correct as-is; src/mitre.rs:91 MUST NOT be changed |
| EC-014 | `--arp-spoof-threshold 0` | Rejected at CLI parse time with error: `--arp-spoof-threshold must be >= 1 (got 0)` (D-074 / BC-2.16.012 EC-004) |

## Tasks

1. **Implement D1 spoof detection** in `process_arp`: on rebind detection, follow the exact 4-step intra-event sequence from BC-2.16.004 postcondition 1 (Step 1: increment `rebind_count`; Step 2: set `first_rebind_ts` if None; Step 3: evaluate HIGH vs MEDIUM; Step 4: update MAC). Emit exactly one D1 finding per rebind event.
2. **Implement GARP-that-conflicts escalation** in `process_arp`: when `is_gratuitous_arp(frame) && binding_conflict`, upgrade GARP finding from LOW to MEDIUM AND emit D1 finding (per BC-2.16.014 postcondition 1/2). The D1 finding's severity uses the same Steps 1–3 as the normal D1 path.
3. **Attach MITRE T0830 + T1557.002** to D1/D2/D12 findings: `mitre_techniques: ["T0830", "T1557.002"]`. D12 MITRE back-fill is co-committed here per the BC-2.16.007 cross-story delivery note (see AC-017).
4. **Apply VP-007 5-part functional atomic update to `src/mitre.rs`** (all four functional sites in §(a) above; verified line numbers from arp-architecture-delta.md §5.0).
5. **Apply all stale-comment updates** to `src/mitre.rs` (all six sites in §(b) above).
6. **Verify IcsImpact Display is "Impact (ICS)"** (D-069): confirm `src/mitre.rs:91` reads `MitreTactic::IcsImpact => "Impact (ICS)"` — no change required. Do NOT revert to `"Impact"`.
7. **Verify HS-008 holdout scenario is correct** (D-069): confirm `.factory/holdout-scenarios/HS-008-*.md` (HS-008 Verification Approach step 1) reads `"Impact (ICS)"` — no change required.
8. **Run `cargo test vp007_catalog_drift_guard`**: MUST pass before PR is opened.
9. **Add `--arp-spoof-threshold` CLI flag**: `#[arg(long, default_value_t = 3)] arp_spoof_threshold: u32` in `src/cli.rs`. This flag is STORY-114's primary deliverable per BC-2.16.012 — it is NOT added in STORY-113. Wire it to `ArpAnalyzer::new(spoof_threshold, storm_rate)` in `src/main.rs`, passing `ARP_STORM_RATE_DEFAULT` (= 50) as `storm_rate` — `--arp-storm-rate` does not exist until STORY-115; using the constant keeps STORY-114 standalone-compilable.
10. **Write unit tests** for AC-001 through AC-016.
11. **Run `cargo test --all-targets`**: all tests green, including vp007_catalog_drift_guard, IcsImpact Display test, and D1 escalation tests.
12. **Run `cargo clippy --all-targets -- -D warnings`**: clean.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_d1_first_rebind_emits_medium` | Unit |
| AC-002 | `test_d1_escalates_to_high_at_threshold` | Unit |
| AC-003 | `test_d1_high_guard_prevents_second_high` | Unit |
| AC-004 | `test_d1_flap_window_reset` | Unit |
| AC-005 | `test_d1_threshold_1_high_on_first_rebind` | Unit |
| AC-006 | `test_cli_arp_spoof_threshold_parsed`, `test_cli_arp_spoof_threshold_default_3`, `test_cli_arp_spoof_threshold_0_rejected` | Unit |
| AC-007 | `test_garp_conflicts_garp_finding_upgrades_to_medium` | Unit |
| AC-008 | `test_garp_conflicts_d1_also_emitted` | Unit |
| AC-009 | `test_garp_conflicts_d1_high_at_threshold` | Unit |
| AC-010 | `test_garp_no_conflict_low_only` | Unit (regression) |
| AC-011 | `test_t0830_and_t1557_002_resolves_in_catalog` | Unit |
| AC-012 | `vp007_catalog_drift_guard`, `test_vp007_seeded_25_emitted_17` | In-crate unit + integration |
| AC-013 | No new test — existing F5 DNP3 tests cover; STORY-114 must not contradict | Verify only |
| AC-014 | `test_impact_vs_ics_impact_variants_distinct` | In-crate unit |
| AC-015 | No new test — HS-008 already correct; Phase 4 holdout evaluation covers | Verify only |
| AC-016 | `test_d1_finding_evidence_contains_ips_and_macs` | Unit |
| AC-017 | `test_d12_mismatch_carries_mitre_after_catalog` | Unit |

## Previous Story Intelligence

STORY-113 (this epic's predecessor) established:
- `BindingEntry` struct with all five fields including `rebind_count`, `first_rebind_ts`, `spoof_high_emitted`.
- `process_arp` detects rebinds and updates binding state (rebind_count, first_rebind_ts) but does NOT emit D1 findings.
- `is_gratuitous_arp` implemented and tested; GARP finding emitted at LOW.
- `src/mitre.rs` is at SEEDED=23, EMITTED=15 (the pre-ARP baseline). DO NOT touch `src/mitre.rs` before this story.

**Critical from STORY-114 perspective**: the "PLANNED — implemented in STORY-114; current code 23/15" marker must be present in BC-2.10.005 and BC-2.10.008 until this story merges. This story's implementer is responsible for removing those markers after the 5-part atomic update is applied.

**STORY-109 precedent** (VP-007 atomic update for T1691.001 + T0827): the exact same pattern was used in E-15. See STORY-109's VP-007 obligation for the reference implementation of the 5-part atomic update pattern.

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §5, ADR-008 Decision 6, BC-2.16.004 postcondition 1:

1. **Intra-event ordering is fixed** — Steps 1/2/3/4 in BC-2.16.004 PC1 must execute in this exact order. Step 4 (MAC update) occurs AFTER escalation evaluation and finding emission. The mac write happens exactly once per frame.
2. **All five `src/mitre.rs` functional sites must be updated in one commit** — partial updates cause `vp007_catalog_drift_guard` to fail. The drift guard mechanically enforces consistency.
3. **MitreTactic enum requires NO new variants** — T0830 → `LateralMovement` (existing variant), T1557.002 → `CredentialAccess` (existing variant). Confirmed per ADR-008 Decision 6.
4. **IcsImpact Display is "Impact (ICS)" — DO NOT change** — `src/mitre.rs:91` already reads `MitreTactic::IcsImpact => "Impact (ICS)"`. This is canonical under D-069 (BC-2.10.002 v1.5). The ICS matrix suffix preserves the distinct tactic identity and prevents erroneous merge-by-name grouping with the Enterprise `"Impact"` tactic. Any test or code change asserting `"Impact"` for IcsImpact is WRONG under D-069.
5. **HS-008 is already correct at "Impact (ICS)"** — `.factory/holdout-scenarios/HS-008-*.md` (HS-008 Verification Approach step 1) must not be changed. The F5 DNP3 tests asserting `"Impact (ICS)" != "Impact"` are correct and remain green. STORY-114 must not contradict them.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `src/mitre.rs` constants | as-is | SEEDED_TECHNIQUE_ID_COUNT: 23 → 25; EMITTED_IDS: 15 → 17 entries |
| `clap` | same as existing | `--arp-spoof-threshold` flag — added here in STORY-114 (BC-2.16.012 primary deliverable; NOT present from STORY-113) |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/arp.rs` | Modify | Add D1 escalation logic + GARP-that-conflicts + T0830/T1557.002 attribution |
| `src/mitre.rs` | Modify | VP-007 5-part atomic update (4 functional sites + 6 comment sites) — IcsImpact Display stays "Impact (ICS)" (D-069); DO NOT change line 91 |
| `src/cli.rs` | Modify | Add `#[arg(long, default_value_t = 3)] arp_spoof_threshold: u32` — STORY-114 is the owner of this flag per BC-2.16.012; it is not carried forward from STORY-113 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5,500 |
| BC files (3 BCs) | ~8,000 |
| arp-architecture-delta.md §5 (VP-007 detail) | ~3,000 |
| VP-007 file | ~2,000 |
| Existing `src/mitre.rs` (relevant sections) | ~4,000 |
| STORY-113 (for ArpAnalyzer context) | ~2,000 |
| Tool outputs (cargo test) | ~1,500 |
| **Total estimated** | **~26,000** |

Within 20–30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-113]` — D1 spoof detection requires the binding table (BC-2.16.005/BC-2.16.006) and rebind state (`rebind_count`, `first_rebind_ts`, `spoof_high_emitted` in `BindingEntry`) from STORY-113. GARP-that-conflicts (BC-2.16.014) requires `is_gratuitous_arp` from STORY-113. The VP-007 atomic update adds T0830 and T1557.002 to the MITRE catalog; these IDs are tagged on D1/D2/D12 findings from STORY-113 and STORY-114 — without the catalog entry, `technique_name("T0830")` would return "Unknown".
- `blocks: [STORY-115]` — STORY-115 implements D3 ARP storm detection. D3 shares `ARP_FLAP_WINDOW_SECS` (defined in BC-2.16.004 and shared) and the `summarize()` storm key. STORY-115 also adds the `--arp-storm-rate` CLI flag, which is the sister flag to `--arp-spoof-threshold` from this story. STORY-115 cannot build cleanly until the full MITRE catalog (SEEDED=25, EMITTED=17) is in place, because its integration tests exercise the full detection + reporting pipeline.
