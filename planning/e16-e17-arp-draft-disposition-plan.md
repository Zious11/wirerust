---
document_type: disposition-plan
producer: research-agent
timestamp: 2026-07-20
scope: E-16/E-17 stale-draft disposition (STORY-111/112/113/114/115/116/117)
precedent: .factory/planning/e11-stale-draft-disposition-plan.md
policy: DF-VALIDATION-001
develop_ref: 1e967bad (orchestrator-verified) + local tree read 2026-07-20
status: validated — NOTHING filed on GitHub by this agent (human-gated step)
verdict_rollup: 7/7 DELIVERED-BY-DRIFT — pure supersession bookkeeping recommended; NO wave-85
---

# E-16 / E-17 ARP Stale-Draft Disposition Plan

**Purpose:** DF-VALIDATION-001 validation pass over the 7 stale ARP story drafts
(STORY-111 … STORY-117; epics E-16 waves 40–44 and E-17 waves 45–46; all
`status: draft`). The orchestrator-verified hypothesis is "delivered-by-drift":
ARP shipped on `develop` via other work, matching the E-11 stale-draft pattern.

**Method:** every story's acceptance criteria were cross-checked against the
*actual current implementation* on `develop` — `src/analyzer/arp.rs` (5248 lines),
`src/decoder.rs`, `src/main.rs`, `src/cli.rs`, `src/mitre.rs`, the seven
`tests/bc_2_16_*` ARP test files, `Cargo.toml`, and `CHANGELOG.md`. Each disposition
claim cites concrete evidence (file:line, test name, CHANGELOG entry). Draft technical
assumptions (etherparse 0.20 API, MITRE ATT&CK tactic mappings, CLI conventions) were
re-verified for current-currency; one MCP call confirmed the MITRE ATT&CK ICS/Enterprise
tactic assignments live.

**Verdict: all 7 are DELIVERED-BY-DRIFT.** Both epics shipped in full — E-16 in
release v0.7.0 (`CHANGELOG.md:1484` "ARP Security Analyzer (issue #9, epic E-16)")
and E-17 in v0.8.0 (`CHANGELOG.md:1468–1478` "VLAN / QinQ (802.1ad double-tag) /
MACsec link-extension ARP offset handling — 10 tests … issue #253, STORY-116/117").
No survivor stories remain open. **No issue is filed; recommend a pure-supersession
bookkeeping burst (draft → superseded/OBSOLETE) with no wave-85.**

---

## Per-Story Disposition Table

| Story | Epic | Pts | Verdict | Evidence summary | Recommended action |
|-------|------|-----|---------|------------------|--------------------|
| STORY-111 | E-16 | 5 | DELIVERED-BY-DRIFT | `Cargo.toml:28` `etherparse = "0.20"`; `DecodedFrame`/`ArpFrame` + `extract_arp_frame` live in `src/decoder.rs`; 0.20 `link_exts` API in use (`decoder.rs:319`) | draft → superseded (OBSOLETE) |
| STORY-112 | E-16 | 8 | DELIVERED-BY-DRIFT | `extract_arp_frame` + strict/lax ARP routing in `decoder.rs`; `ArpAnalyzer` in `arp.rs`; `main.rs` `DecodedFrame` match (`main.rs:445`); Kani `verify_extract_arp_frame_*` present; `tests/bc_2_16_story112_arp_tests.rs` | draft → superseded (OBSOLETE) |
| STORY-113 | E-16 | 13 | DELIVERED-BY-DRIFT | `is_gratuitous_arp` (`arp.rs:161`), `insert_binding_lru` (`arp.rs:180`), 13-key `summarize()` (`arp.rs:753–806`), `--arp` flag (`cli.rs:228`), Kani `verify_classify_garp_total`/`verify_binding_table_cap` (`arp.rs:4596/4648`); `tests/bc_2_16_story113_arp_tests.rs` | draft → superseded (OBSOLETE) |
| STORY-114 | E-16 | 13 | DELIVERED-BY-DRIFT (with corrected tactic mapping) | D1 escalation + GARP-conflict + T0830/T1557.002 attach (`arp.rs:517/908/938`); `--arp-spoof-threshold` + 0-reject (`cli.rs:235`, `main.rs:218`); MITRE catalog seeds T0830/T1557.002 (`mitre.rs:233–237`, `EMITTED_IDS` `mitre.rs:359–360`); `tests/bc_2_16_story114_arp_tests.rs` | draft → superseded (OBSOLETE); log the divergences below |
| STORY-115 | E-16 | 8 | DELIVERED-BY-DRIFT | `detect_storm` (`arp.rs:1016`), `MAX_STORM_COUNTERS=4096` (`arp.rs:94`), `--arp-storm-rate` + 0-reject (`cli.rs:244`, `main.rs:215`), `storm_findings`/`storm_counters_evicted` summary keys (`arp.rs:786/806`), D3 `mitre_techniques: []` (`arp.rs:1071`); `tests/bc_2_16_story115_arp_tests.rs` | draft → superseded (OBSOLETE) |
| STORY-116 | E-17 | 3 | DELIVERED-BY-DRIFT | `tests/bc_2_16_qinq_macsec_offset_tests.rs` present with all 4 named tests; QinQ offset `14 + Σ link_exts.header_len()` at `decoder.rs:315–320`; PR #258 merged (CHANGELOG.md:1468) | draft → superseded (OBSOLETE) |
| STORY-117 | E-17 | 5 | DELIVERED-BY-DRIFT | `tests/bc_2_16_e17_macsec_offset_tests.rs` present with all 6 named tests (offset 22 / offset 30 / Modified-opaque guards); zero `todo!`/`unimplemented!`; documented-limitation shipped (CHANGELOG.md:1476–1478) | draft → superseded (OBSOLETE) |

**Points delivered-by-drift:** E-16 = 5+8+13+13+8 = **47**; E-17 = 3+5 = **8**;
**total = 55 points**, 100% delivered-by-drift.

---

## Overall Rollup

| Classification | Count | Stories | Pts |
|----------------|-------|---------|-----|
| DELIVERED-BY-DRIFT | 7 | 111, 112, 113, 114, 115, 116, 117 | 55 |
| PARTIALLY-DELIVERED | 0 | — | 0 |
| STILL-OPEN | 0 | — | 0 |
| SUPERSEDED-UPSTREAM | 0 | — | 0 |

**No survivors. No wave-85 candidate. Recommend pure-supersession bookkeeping burst.**

---

## Per-Story Validation Detail

### STORY-111 — etherparse 0.20 Migration + DecodedFrame/ArpFrame + BC-2.02.009 (5 pts) — DELIVERED-BY-DRIFT, HIGH

- **AC-010 (Cargo.toml bump):** `Cargo.toml:28` reads `etherparse = "0.20"`; the version-pin
  comment block (`Cargo.toml:21–27`) documents the 0.20 `link_exts`/`SliceError::Len` contract.
- **AC-003/005/005b/006/009 (types + dispatch + no-panic):** `DecodedFrame`, `ArpFrame`,
  `extract_arp_frame`, and the strict/lax dispatch arms are all present in `src/decoder.rs`
  and are the *fully implemented* versions — the STORY-111 "non-panicking placeholder" seam has
  been overwritten by STORY-112's real logic (expected: STORY-111's scaffolding was a transient
  seam, never a shipped end state). The 0.20 `link_exts` API is in live use at `decoder.rs:319`.
- **Currency check:** the draft's central premise (migrate 0.16 → 0.20) is satisfied and stable;
  Cargo pins the 0.20.x minor. No open work.

### STORY-112 — extract_arp_frame + decode_packet routing + ArpAnalyzer stub + VP-024 Sub-A (8 pts) — DELIVERED-BY-DRIFT, HIGH

- **AC-001…005 (extract_arp_frame):** implemented in `src/decoder.rs`; unit coverage in
  `tests/bc_2_16_story112_arp_tests.rs`.
- **AC-006/007/012 (strict + lax ARP routing, error strings):** both arms present in
  `decode_packet`; the D-078 lax None-arm fixed-header peek is realized (the QinQ/MACsec offset
  logic at `decoder.rs:290–337` is the descendant of this arm).
- **AC-008/010 (main.rs wiring + ArpAnalyzer):** `main.rs:445` routes `DecodedFrame::Arp`;
  `ArpAnalyzer` exists (far beyond the stub).
- **AC-011 (VP-024 Sub-A Kani):** `verify_extract_arp_frame_safety` / `_eth_ipv4_correctness` /
  `_none_on_bad_size` harnesses present (module doc `arp.rs:18–19` records F6 formal proof).

### STORY-113 — ArpAnalyzer full impl: binding table, GARP, summarize(), --arp, VP-024 Sub-B/C/D (13 pts) — DELIVERED-BY-DRIFT, HIGH

- **is_gratuitous_arp** `arp.rs:161`; **insert_binding_lru** `arp.rs:180`; **MAX_ARP_BINDINGS = 65_536**
  `arp.rs:57`.
- **13-key summarize()** — exact key set present at `arp.rs:753–806` and the drift-guard test list
  at `arp.rs:2072–2082`: `frames_analyzed, request_count, reply_count, other_opcode_count,
  bindings_tracked, spoof_findings, garp_findings, storm_findings, mismatch_findings,
  malformed_findings, malformed_frames, bindings_evicted, storm_counters_evicted`. Matches
  BC-2.16.010 v1.9 (13 keys).
- **--arp flag** `cli.rs:228`; **record_malformed** `arp.rs:698`.
- **VP-024 Sub-B/C/D** — `verify_classify_garp_total` (`arp.rs:4596`), `verify_binding_table_cap`
  (`arp.rs:4648`) using `insert_binding_lru_array` (`arp.rs:299`); Sub-C last-write-wins proptest
  present. Note: the array surrogate shipped as a generic `insert_binding_lru_array<const N>`;
  a `insert_binding_lru_btree` also exists (`arp.rs:232`) — a superset of the draft's plan, not a gap.

### STORY-114 — D1 spoof escalation + GARP-conflict + MITRE + VP-007 5-part atomic update (13 pts) — DELIVERED-BY-DRIFT, HIGH — WITH TWO NOTED DIVERGENCES

Functionally delivered:
- **D1 escalation (AC-001…005)** and **GARP-that-conflicts (AC-007…010)** implemented in
  `process_arp`; D1/D12/GARP-conflict findings carry `mitre_techniques: ["T0830","T1557.002"]`
  (`arp.rs:517`, `:908`, `:938`).
- **--arp-spoof-threshold (AC-006)** `cli.rs:235` (`default_value_t = 3`); the `0`-rejection
  fail-fast `anyhow::bail!` is at `main.rs:218–219` ("must be >= 1 (got 0)") — matches D-074.
- **MITRE catalog (AC-011/012):** T0830 and T1557.002 are seeded (`mitre.rs:233–237`) and emitted
  (`EMITTED_IDS` `mitre.rs:359–360`).
- **IcsImpact Display (AC-013):** `mitre.rs:107` reads `"Impact (ICS)"` — unchanged, per D-069.

Two divergences from the draft's *assumed* specifics (the shipped code is authoritative and, in
one case, MORE correct than the draft — evidence that re-opening the draft as-written would be wrong):

1. **VP-007 counts drifted well past the draft's 25/17.** The draft assumed
   `SEEDED_TECHNIQUE_ID_COUNT: 23 → 25`, `EMITTED: 15 → 17`. Actual `mitre.rs:485` reads
   `SEEDED_TECHNIQUE_ID_COUNT = 29`, because later stories seeded further techniques (T0858/T0816/
   T1693.001 via STORY-133; T0881 via STORY-173 — `mitre.rs:238–251`). The ARP-specific pair
   (T0830 + T1557.002) is present and emitted, so the STORY-114 obligation is satisfied; the count
   is simply higher due to superseding work. **A re-scoped survivor asserting "== 25/== 17" would
   FAIL the current tree — do not resurrect the numeric ACs verbatim.**
2. **T0830 tactic mapping corrected from the draft.** STORY-114 AC-011 asserted
   `technique_info("T0830") == (…, MitreTactic::LateralMovement)`. Actual code maps it to
   `MitreTactic::IcsCollection` (TA0100) at `mitre.rs:233`. Live MITRE ATT&CK verification (see
   Research Methods) confirms **T0830 Adversary-in-the-Middle is a Collection (TA0100) technique in
   the ICS matrix**, NOT Lateral Movement — so the shipped code is correct and the draft's
   assumption was stale. `CHANGELOG.md:1239` records the deliberate reclassification
   ("T0830 … reclassified … to Collection"). T1557.002 → `CredentialAccess` (TA0006) matches both
   draft and MITRE.

These divergences do not change the verdict (DELIVERED-BY-DRIFT); they are recorded so the
supersession note documents that the draft's numeric/mapping assumptions are obsolete.

### STORY-115 — D3 ARP storm detection + --arp-storm-rate + storm_findings (8 pts) — DELIVERED-BY-DRIFT, HIGH

- **detect_storm** `arp.rs:1016`; rate formula `count / max(1, elapsed)` and 60s window shared via
  `ARP_FLAP_WINDOW_SECS` (`arp.rs:79`); **ARP_STORM_RATE_DEFAULT = 50** (`arp.rs:86`);
  **MAX_STORM_COUNTERS = 4096** (`arp.rs:94`) with LRU eviction and `storm_counters_evicted`
  observability (`arp.rs:806`).
- **--arp-storm-rate (AC-011)** `cli.rs:244` (`default_value_t = 50`); `0`-rejection at `main.rs:215–216`.
- **AC-012 (flag accepted without --arp)** and **AC-014 (D3 `mitre_techniques: []`, T0814 withheld)**
  — `arp.rs:1071` emits empty MITRE; `CHANGELOG.md:702–703` and `:1493` record the DF-VALIDATION-001
  T0814 withholding (D3 T0830 drift errata). Tests in `tests/bc_2_16_story115_arp_tests.rs`.

### STORY-116 — ARP QinQ (double-tag) decoder offset coverage (3 pts) — DELIVERED-BY-DRIFT, HIGH

- All 4 named tests exist in `tests/bc_2_16_qinq_macsec_offset_tests.rs`:
  `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11` (:249),
  `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11` (:433),
  `test_BC_2_16_015_qinq_link_exts_offset_formula_pin` (:546),
  `test_BC_2_16_015_macsec_arp_lax_parse_probe` (:671).
- The offset formula `14 + Σ link_exts.header_len()` is implemented at `decoder.rs:315–320`
  (`lax.link_exts.iter().map(|ext| ext.header_len()).sum()`), exactly as the draft's AC-003 pins.
- **Currency check:** draft assumes etherparse 0.20.2 "two `Vlan` entries, no `VlanDouble`". Cargo
  pins `0.20` (0.20.x); no `VlanDouble` reference exists in `src/`. Assumption still holds.
- PR #258 is merged (the facade delivery condition) — `CHANGELOG.md:1468–1471`.

### STORY-117 — ARP MACsec offset documented-limitation coverage (5 pts) — DELIVERED-BY-DRIFT, HIGH

- All 6 named tests exist in `tests/bc_2_16_e17_macsec_offset_tests.rs` (offset-22 no-SCI,
  offset-30 SCI-present, hlen=8 D11 routing for both, and the two Modified-opaque
  `stop_err != Layer::Arp` security guards at :885/:969). Zero `todo!`/`unimplemented!`/`unreachable!`
  in the file (facade requirement satisfied).
- The off-by-8 SCI-accounting guard and the DOCUMENTED-UNVERIFIED real-traffic limitation shipped —
  `CHANGELOG.md:1470–1478`.
- **Currency check:** MACsec `header_len()` values (8 / 16 / 6 / 14) are asserted at runtime by the
  tests rather than by volatile source-line citations (per the draft's own Compliance Rule 5), so
  they are self-guarding against etherparse drift.

---

## Points Accounting / Epic Impact

- **E-16 (STORY-111…115): 47 pts, all DELIVERED-BY-DRIFT.** Epic shipped as the "ARP Security
  Analyzer" feature in release v0.7.0 (`CHANGELOG.md:1484`, issue #9). Moving these 5 drafts to
  `superseded` removes 47 points of phantom open backlog from E-16; the epic's *delivered* scope is
  unchanged (it was already delivered in code and CHANGELOG).
- **E-17 (STORY-116/117): 8 pts, all DELIVERED-BY-DRIFT.** Epic shipped as the QinQ/MACsec
  offset-hardening test suite in v0.8.0 (`CHANGELOG.md:1468`, issue #253). Superseding removes 8
  points of phantom open backlog from E-17.
- **Net:** 55 points move draft → superseded. Zero points survive to any future wave. Both epics
  should be marked COMPLETE/DELIVERED in STORY-INDEX / STATE if not already (a status-flip
  bookkeeping item — this is exactly the STORY-155 / upstream #290 "post-merge status not flipped"
  class, which is why these drafts still read `status: draft` despite shipping).

---

## Wave-85 Composition

**Not applicable.** No STILL-OPEN or PARTIAL survivors exist. There is nothing to schedule.

The only follow-through is a **pure-supersession bookkeeping burst** (human-gated), no code, no CI:

1. Flip `status: draft → superseded` (disposition OBSOLETE / delivered-by-drift) on
   STORY-111/112/113/114/115/116/117, each carrying a one-line supersession pointer to the shipping
   evidence in the table above (release + CHANGELOG anchor + primary file:line).
2. In the STORY-114 supersession note specifically, record the two divergences (VP-007 count now 29
   not 25; T0830 → IcsCollection/TA0100 not LateralMovement, corrected and MITRE-verified) so no
   future reader mistakes the numeric ACs for current truth.
3. Confirm E-16 and E-17 are marked delivered in STORY-INDEX/STATE (status-flip only).
4. Re-run `bin/compute-input-hash --scan` after the status edits (frontmatter-only change does not
   alter `input-hash`, which hashes `inputs:` file contents, but scan-clean is the gate discipline).

No GitHub issue is filed by this agent (DF-VALIDATION-001 human-gated step). Nothing here is engine/
upstream-shaped — all seven are wirerust product code, so no drbothen/vsdd-factory routing applies.

---

## Inconclusive / Flags for the Team Lead

- **None of the seven are inconclusive** — every AC maps to shipped code/tests with a file:line or
  test-name anchor, and the release CHANGELOG independently corroborates both epics.
- **STORY-114 numeric ACs are actively stale**, not merely satisfied: an agent that resurrected the
  draft and re-asserted `SEEDED == 25` would fail the current tree (`= 29`). The supersession note
  MUST flag this so the draft is retired, not "re-validated against 25".
- **`bindings_tracked` key ordering:** the doc-comment key list (`arp.rs:734–746`) and the emitted
  order (`arp.rs:758–806`) differ slightly in ordering, but all 13 keys are present and the exact
  name set matches BC-2.16.010 v1.9. Non-issue for supersession; noted for completeness.
- **Why these read `draft` despite shipping:** classic post-merge index-status-not-flipped (E-11
  STORY-155 / upstream #290 class). Recommend the bookkeeping burst also verify no *other* delivered
  ARP-adjacent stories (e.g. STORY-156 BC-2.16.016, PR #378, `CHANGELOG.md:834`) are stuck at draft.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 0 | Not required — disposition is a source-tree/AC cross-check, not multi-source synthesis; the single external-fact question (MITRE tactic mapping) is a ≤2-sentence lookup, so `perplexity_ask` is the correct variant per the agent's bias rule. |
| Perplexity perplexity_ask | 1 | Live-verified MITRE ATT&CK tactic assignment for T0830 (ICS Collection/TA0100) and T1557.002 (Enterprise Credential Access/TA0006) — confirmed the shipped `IcsCollection` mapping is correct and the STORY-114 draft's `LateralMovement` assumption is stale. |
| Read | 9 | e11 precedent plan; 7 story drafts; Cargo.toml; targeted mitre.rs slice |
| Grep | 10 | arp.rs function/constant/summary-key inventory; mitre.rs counts + T0830/T1557.002 mappings; decoder QinQ/MACsec offset; cli.rs flags; main.rs wiring + 0-reject; test-file name inventories; todo!/unimplemented! facade check; CHANGELOG delivery evidence |
| Glob | 3 | ARP / QinQ / MACsec test-file discovery in tests/ |
| Training data | 0 areas | MITRE mapping verified live rather than from training data; all impl claims verified against the tree |

**Total MCP tool calls:** 1 (perplexity_ask).
**Training data reliance:** low — every disposition claim is anchored to a file:line, test name, or
CHANGELOG entry read from the current `develop` tree; the one external fact (MITRE tactic) was
web-verified live.

**Deviation note (perplexity_research = 0):** the mandate biases toward `perplexity_research` for
non-trivial topics. This task is a delivered-by-drift AC-vs-code audit whose evidence lives entirely
in the local repository, not on the web; the only web-checkable claim was a single MITRE tactic
mapping, appropriately handled with `perplexity_ask`. MCP tooling was available and exercised (1
call), satisfying the ≥1-MCP-call gate; no MCP-UNAVAILABLE escalation is warranted.

---

## Independent Validation Pass (2026-07-21)

**Reviewer role:** Fresh, skeptical second reviewer under DF-VALIDATION-001. Mandate was to
independently CONFIRM or REFUTE the "7/7 delivered-by-drift" disposition above against the live
`develop` tree — not to rubber-stamp it. Every claim below is anchored to evidence I gathered
myself (file:line, test name, grep result, or an independent MITRE web lookup), read from the
working tree at the time of review (repo HEAD family `1e967bad`; `Cargo.toml` version now
`0.13.0`). Where my findings diverge from the first report, I say so explicitly.

### VERDICT: CONFIRM (with two mandatory corrections carried into the supersession note)

All 7 drafts (STORY-111…117) are **DELIVERED-BY-DRIFT**. Every acceptance-criterion I spot-checked
maps to shipped code, a shipped test, or a shipped CHANGELOG entry. **The negative check found ZERO
genuinely-open ACs** — there is no real still-open work the first report missed. The disposition is
sound and safe to action as a pure-supersession bookkeeping burst.

Two corrections must ride along (details below): (1) the first report's release-version label for
E-17 is **wrong** — STORY-116/117 shipped in **v0.7.1**, not v0.8.0; and (2) the STORY-114
stale-assumption caveat is independently reconfirmed and MUST survive into the supersession note.

### Claim 1 — Release provenance (E-16 = v0.7.0, E-17 = v0.8.0)

- **E-16 CONFIRMED in v0.7.0.** `CHANGELOG.md:1480` `## [0.7.0] - 2026-06-16`; `:1484`
  "**ARP Security Analyzer** (issue #9, epic E-16)", enumerating D1/D2/D3/D11/D12 and the three CLI
  flags, "Implemented across STORY-111..115 (PRs #236, #238, #239, #240, #241)" (`:1505`).
- **E-17 release label is WRONG in the first report — it shipped in v0.7.1, not v0.8.0.**
  `CHANGELOG.md:1464` `## [0.7.1] - 2026-06-17`; `:1468-1471` "Regression test coverage for VLAN /
  QinQ (802.1ad double-tag) / MACsec link-extension ARP offset handling — 10 tests across
  `tests/bc_2_16_qinq_macsec_offset_tests.rs` and `tests/bc_2_16_e17_macsec_offset_tests.rs`
  (issue #253, STORY-116/117)". The `## [0.8.0] - 2026-06-17` section (`:1448-1462`) is `--no-collapse`
  / finding-collapse (STORY-118, #259) — unrelated to ARP. Note also `CHANGELOG.md:1475` states the
  offset *handling itself* shipped in **0.7.0**; v0.7.1 added the regression guards. So E-17's code
  landed 0.7.0 and its test suite landed 0.7.1 — **v0.8.0 is not involved at all.** The first
  report's line citations (1468–1478) actually point at v0.7.1 content, so its *evidence* is right;
  only the *version label* ("v0.8.0") is mislabeled. This does not change the delivered-by-drift
  verdict but the supersession note for STORY-116/117 MUST cite v0.7.1 (+ v0.7.0 for the handling),
  not v0.8.0.
- **Code corroboration CONFIRMED:** `Cargo.toml:28` `etherparse = "0.20"` (with the 0.20 `link_exts`
  pin-comment block at `:21-27`); `src/decoder.rs` defines `struct ArpFrame` (`:141`), `enum
  DecodedFrame` (`:162`), `pub fn extract_arp_frame` (`:395`), and uses the 0.20 `link_exts` API live
  (`:319`); `src/main.rs` routes `Ok(DecodedFrame::Arp(arp_frame))` (`:446`) and constructs
  `ArpAnalyzer::new(arp_spoof_threshold, arp_storm_rate)` (`:229`).

### Claim 2 — Per-story AC spot-checks (STORY-113 / 115 / 116 / 117 deep-checked)

- **STORY-113 (full ArpAnalyzer) CONFIRMED.** Independently read the draft's ACs and matched them to
  code: `is_gratuitous_arp` (`arp.rs:161`), `insert_binding_lru` (`:180`), `record_malformed`
  (`:698`), `MAX_ARP_BINDINGS = 65_536` (`:57`). The 13-key `summarize()` (`:753-808`) emits exactly
  the AC-013 key set — I counted all 13 in source: `frames_analyzed, request_count, reply_count,
  other_opcode_count, bindings_tracked, spoof_findings, garp_findings, storm_findings,
  mismatch_findings, malformed_findings, malformed_frames, bindings_evicted, storm_counters_evicted`.
  `--arp` flag `arp: bool` at `cli.rs:228`. VP-024 Sub-B/C/D Kani: `verify_classify_garp_total`
  (`arp.rs:4596`) and `verify_binding_table_cap` (`:4648`). Both `insert_binding_lru_array` (`:299`,
  the current Sub-D surrogate per the draft's own v1.2 rename) and a legacy `insert_binding_lru_btree`
  (`:232`) exist — a superset, not a gap. VP-024 Sub-A `extract_arp_frame` Kani harnesses live in
  `decoder.rs`: `verify_extract_arp_frame_safety` (`:616`), `_eth_ipv4_correctness` (`:643`),
  `_none_on_bad_size` (`:705`). `tests/bc_2_16_story113_arp_tests.rs` present.
- **STORY-115 (D3 storm) CONFIRMED.** `detect_storm` (`arp.rs:1016`); rate formula
  `count_in_window / elapsed.max(1)` (`:1041-1042`); 60 s window via `ARP_FLAP_WINDOW_SECS = 60`
  (`:79`); `ARP_STORM_RATE_DEFAULT = 50` (`:86`); `MAX_STORM_COUNTERS = 4_096` (`:94`);
  `--arp-storm-rate` `default_value_t = 50` (`cli.rs:244`) with 0-reject `anyhow::bail!("--arp-storm-rate
  must be >= 1 (got 0)")` (`main.rs:215-216`). AC-014 (D3 emits empty MITRE, T0814 withheld)
  CONFIRMED by reading the emit site: `detect_storm` returns `mitre_techniques: vec![]` with the
  literal comment "T0814 withheld per DF-VALIDATION-001 / BC-2.16.008 Invariant 3" (`arp.rs:1070-1071`).
  `storm_findings` / `storm_counters_evicted` summary keys present. `tests/bc_2_16_story115_arp_tests.rs`
  present.
- **STORY-116 (QinQ double-tag) CONFIRMED.** `tests/bc_2_16_qinq_macsec_offset_tests.rs` exists with
  the 4 named tests: `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11` (`:249`),
  `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11` (`:433`),
  `test_BC_2_16_015_qinq_link_exts_offset_formula_pin` (`:546`),
  `test_BC_2_16_015_macsec_arp_lax_parse_probe` (`:671`). Offset formula
  `Some(14 + Σ ext.header_len())` for `LinkSlice::Ethernet2` at `decoder.rs:315-321`.
- **STORY-117 (MACsec offset) CONFIRMED.** `tests/bc_2_16_e17_macsec_offset_tests.rs` exists with the
  6 named tests, including offset-22 no-SCI (`:270`), offset-30 SCI-present (`:575`), hlen=8 D11
  routing for both (`:442`, `:754`), and the two Modified-opaque guards (`:885`, `:969`). Zero
  `todo!`/`unimplemented!` in the file (grep clean).

### Claim 3 — STORY-114 divergences (SEEDED count, T0830 tactic)

- **(a) SEEDED count is 29, not 25 — CONFIRMED.** `mitre.rs:485` `const SEEDED_TECHNIQUE_ID_COUNT:
  usize = 29;`. The draft's AC-012 asserts `SEEDED_TECHNIQUE_ID_COUNT == 25` and
  `kani_proofs::EMITTED_IDS.len() == 17` (STORY-114.md:170-171), and its Architecture Mapping pins
  "Constant = 25" / "17 entries" (`:237-238`). **Those numeric ACs would FAIL the current tree
  verbatim** — the count drifted upward via later seeding (STORY-133/173, per the arp.rs/mitre.rs
  provenance comments). The ARP pair itself is present: `SEEDED_TECHNIQUE_IDS` contains `"T0830"`
  (`mitre.rs:468`) and `"T1557.002"` (`:469`), and `EMITTED_IDS` contains both (`:359-360`). So the
  functional obligation is satisfied; only the frozen count is stale.
- **(b) T0830 maps to IcsCollection, not LateralMovement — CONFIRMED.** `mitre.rs:233`
  `"T0830" => ("Adversary-in-the-Middle", MitreTactic::IcsCollection)`, with `IcsCollection` →
  Display `"Collection (ICS)"` (`:109`) and ID `"TA0100"` (`:140`). The draft's AC-011 explicitly
  asserts `MitreTactic::LateralMovement` (STORY-114.md:166-167) and its Detailed-Design/Task/Compliance
  sections repeat the `LateralMovement` mapping (`:76`, `:89`, `:325`). **That assertion would FAIL
  the current tree.** T1557.002 → `CredentialAccess` (`mitre.rs:236`) matches both draft and code.
- **Independent MITRE verification (my own web lookup, not the first report's):** I ran a fresh
  `perplexity_ask` query. Result: **T0830 Adversary-in-the-Middle is a Collection tactic (TA0100)
  in the ICS matrix** (cites attack.mitre.org/techniques/T0830 and /tactics/ics/), and
  **T1557.002 ARP Cache Poisoning is Credential Access (TA0006)** in Enterprise. Therefore the
  shipped code is authoritative and CORRECT, and the STORY-114 draft's `LateralMovement` assumption
  is objectively stale. This independently reproduces (does not merely echo) the first report's
  Claim-3 conclusion.

### Claim 4 — NEGATIVE CHECK (the decisive test): any genuinely-open AC?

**Result: NO open AC found. The 7/7 verdict is NOT wrong.** Evidence:

- **No stubs anywhere in the ARP path.** `grep todo!|unimplemented!|unreachable!` over
  `src/analyzer/arp.rs` → no matches; over `src/decoder.rs` → no matches; over `src/main.rs` → no
  matches. (`unreachable` appears only in test *names* like `..._modified_opaque_payload_unreachable`,
  not as a panic macro.)
- **Every CLI flag named across the 7 drafts exists and is wired with 0-reject fail-fast:** `--arp`
  (`cli.rs:228`), `--arp-spoof-threshold` (`:234-235`, default 3, reject at `main.rs:218-219`),
  `--arp-storm-rate` (`:243-244`, default 50, reject at `main.rs:215-216`).
- **Every named test file exists:** `tests/bc_2_16_story112_arp_tests.rs`, `..._story113_...`,
  `..._story114_...`, `..._story115_...`, `bc_2_16_qinq_macsec_offset_tests.rs`,
  `bc_2_16_e17_macsec_offset_tests.rs` (plus D-074/D-077/D-078/016 hardening suites) — all present in
  `tests/`.
- **The only "would-fail-if-resurrected" cases (STORY-114 AC-011/AC-012) are stale-assertion, not
  open-work.** The underlying behavior (T0830+T1557.002 seeded and emitted; catalog drift-guard
  passing) is delivered; the draft merely froze now-superseded constants/tactic. That is the textbook
  delivered-by-drift-with-corrected-specifics case, not a survivor.
- **All 7 drafts confirmed `status: draft`** (grep over STORY-111…117 frontmatter), consistent with
  the "post-merge status-not-flipped" root cause the first report cites.

### Corrections the supersession note MUST carry

1. **STORY-116/117 release label:** cite **v0.7.1** (regression suite, `CHANGELOG.md:1464-1471`) and
   **v0.7.0** (offset handling itself, `:1475`) — **NOT v0.8.0**. The first report's "v0.8.0" label is
   a factual error (v0.8.0 is `--no-collapse`/STORY-118); its line citations point at v0.7.1 content.
2. **STORY-114 stale assumptions (unchanged mandate — must survive regardless of verdict):** record
   that (a) `SEEDED_TECHNIQUE_ID_COUNT` is now **29**, not the draft's 25 (and `EMITTED` likewise
   drifted past 17); and (b) T0830 is mapped to **IcsCollection / TA0100**, MITRE-correct, not the
   draft's `LateralMovement`. Any future reader must not treat the STORY-114 numeric/tactic ACs as
   current truth or attempt to "re-validate against 25/LateralMovement".

### Inconclusive / flags

- **INCONCLUSIVE (immaterial):** exact PR numbers cited for the facade deliveries (e.g. "PR #258"
  for STORY-116) were not re-verified — PR metadata is not in the `develop` tree and is not required
  for the delivered-by-drift determination, which rests on shipped code + tests + CHANGELOG.
- **Minor citation drift (immaterial):** a few of the first report's `CHANGELOG.md` line anchors are
  off by ~2 lines versus the current HEAD (e.g. it cites `:1468` for the E-17 Added entry; current is
  `:1470`). The referenced content is correct; only the line numbers have shifted slightly as the file
  grew. No effect on the verdict.
- No story is inconclusive on substance; every functional AC maps to shipped code/test/CHANGELOG.

### Bottom line

**CONFIRM.** The disposition is sound: all 7 ARP drafts are delivered-by-drift and safe to move
`draft → superseded`. Fix the E-17 version label (v0.7.1/v0.7.0, not v0.8.0) and preserve the
STORY-114 stale-assumption caveat (SEEDED=29 not 25; T0830→IcsCollection/TA0100 not LateralMovement)
in the supersession notes. No issue is filed by this agent (DF-VALIDATION-001 human-gated step).

### Independent Validation — Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 0 | Not used — validation is a local-tree AC-vs-code audit; the sole external fact (MITRE tactic) is a ≤2-sentence lookup, correctly handled by `perplexity_ask` per the bias rule. |
| Perplexity perplexity_ask | 1 | Fresh, independent MITRE ATT&CK verification: T0830 = Collection/TA0100 (ICS), T1557.002 = Credential Access/TA0006 (Enterprise) — reproduced, not echoed, the first report's mapping conclusion. |
| Read | 5 | Disposition plan under validation; Cargo.toml; CHANGELOG release sections; STORY-113 + STORY-114 drafts in full; detect_storm emit block; summarize() body. |
| Grep | 9 | arp.rs function/constant inventory; mitre.rs SEEDED count + T0830/T1557.002 mappings + EMITTED_IDS; cli.rs flags; main.rs wiring + 0-reject; decoder.rs types/offset/stubs; todo!/unimplemented! sweeps (arp.rs, decoder.rs, main.rs); STORY-114 numeric/tactic assertions; all-7 status frontmatter; CHANGELOG release-provenance. |
| Glob | 2 | STORY-111…117 draft discovery; tests/bc_2_16_*.rs inventory. |
| Training data | 0 areas | MITRE fact verified live; all impl claims verified against the tree. |

**Total MCP tool calls (this pass):** 1 (perplexity_ask).
**Training data reliance:** low — every validation claim is anchored to a file:line, test name, or
CHANGELOG entry read from the current tree; the one external fact was web-verified live and
independently of the first report.

**Deviation note (perplexity_research = 0):** identical rationale to the first pass — this is a
local-repository AC-vs-code audit, not multi-source web synthesis, so `perplexity_research` is not
the right instrument; the ≥1-MCP-call gate is satisfied via the independent MITRE `perplexity_ask`.
MCP tooling was available and exercised; no MCP-UNAVAILABLE escalation warranted.
