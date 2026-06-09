---
document_type: verification-delta
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "F2 Verification Delta — Modbus TCP Analyzer (SS-14)"
status: draft
producer: formal-verifier
created: 2026-06-09
base_commit: 4cfc4c4
branch: develop
modified:
  - date: 2026-06-09
    actor: formal-verifier
    note: "Reconcile against directives v2.0 + ADR-006 (Modbus F2 revision). §4: recon emitted ID T0846→T0888, SEEDED_TECHNIQUE_ID_COUNT 20→21 (T0888 new seeded; T0846 stays seeded, drops from emission), EMITTED stays 13; add Finding field-rename grep obligation (mitre_technique:Some → mitre_techniques:vec!). §7 NEW: VP-016/VP-020/VP-021 Finding field-rename harness obligations (VP-020 gains multi-tag semicolon-join CSV-cell case). §8 NEW: new-VP decision — NO VP-023 (multi-tag covered by VP-007/017/016/020); VP-INDEX stays at 22. No locked-VP lock fields edited; no index arithmetic change."
traces_to:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.001.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.005.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.006.md
---

# F2 Verification Delta — Issue #7: Modbus TCP Protocol Analyzer

This document is the formal-verifier's Phase-F2 verification delta. It records the one new
verification property (VP-022), confirms no existing VP (VP-001..021) requires modification by
this feature, and flags the verification *dependencies* (VP-004 harness extension, VP-007
catalog-drift-guard, and the VP-016/020/021 multi-tag harness updates from ADR-006 Decision 13)
that must be honored in F4 to keep already-locked proofs green.

The architect has already PRE-REGISTERED VP-022 in the three index files (counts bumped:
Kani 8→9, total 21→22, P1 7→8). This delta does not re-touch those indexes; it authors the
VP-022 spec file and documents the regression surface.

> **Reconciliation note (F2 v2 — directives v2.0 + ADR-006, 2026-06-09):** Sections §4 and §7
> below are reconciled against `f2-fix-directives.md` v2 (Decisions 11/12/13) and
> `ADR-006-multi-technique-finding-attribution.md`. Two changes supersede the prior v1
> directive content in this delta: **(1)** Decision 12 swaps the recon-path emitted technique
> from **T0846 → T0888** (T0888 "Remote System Information Discovery"; T0846 stays seeded but is
> no longer Modbus-emitted), which moves `SEEDED_TECHNIQUE_ID_COUNT` from **20 → 21** (T0888 is a
> NEW seeded entry; the earlier "20" assumed T0846-only recon and no new seeded ICS ID for recon).
> EMITTED count is unchanged at **13**. **(2)** Decision 13 (ADR-006) renames `Finding.mitre_technique:
> Option<String>` → `mitre_techniques: Vec<String>`, which forces F4 harness updates to VP-007's
> grep pattern and to the VP-016/020/021 proof harnesses (see §7). None of these edit any locked
> VP document's lock fields; they are F4 harness/source obligations recorded here, same pattern as
> the VP-004 extension.

---

## 1. New Verification Property

### VP-022 — Modbus MBAP Parse Safety and Function-Code Boundary Classification

| Field | Value |
|-------|-------|
| ID | VP-022 |
| Status | draft (`verification_lock: false`) |
| Tool | Kani |
| Phase | P1 |
| Module | `src/analyzer/modbus.rs` |
| Spec file | `.factory/specs/verification-properties/vp-022-modbus-parse-safety.md` |
| Source BC | BC-2.14.001 |
| Anchored BCs | BC-2.14.001, .002, .003, .004, .005, .006, .007, .008 |
| Sub-properties | 3 (A parse safety, B classify_fc totality, C exception biconditional) |

**Three Kani sub-properties** (all over bounded symbolic inputs; all expected `VERIFICATION:- SUCCESSFUL`):

- **Sub-property A — parse safety.** Over a symbolic `&[u8]` of bounded length (`[u8; 12]` +
  symbolic `len <= 12`): `parse_mbap_header` never panics, returns `None` for `len < 8`, returns
  `Some(well-formed MbapHeader)` for `len >= 8` (big-endian fixed-offset decode), and never
  indexes out of bounds. A companion harness proves the 3-point validity gate
  `is_valid_modbus_adu` is `true` iff `protocol_id == 0x0000 && 2 <= length <= 253`.
  **Anchors BC-2.14.001/002/003/004.**
- **Sub-property B — `classify_fc` totality.** Over a symbolic `u8` (all 256 values):
  `classify_fc` returns a defined `FunctionCodeClass` variant for every input (no panic, no
  `unreachable!`, no gap — the `_ => Unknown` wildcard makes the match total). Read/Write/Diagnostic
  set membership pinned per the canonical BC map (005=classify_fc totality, 007=Write-class,
  008=Diagnostic-class). **Anchors BC-2.14.005/007/008.**
- **Sub-property C — exception-detection biconditional.** Over a symbolic `u8`:
  `classify_fc(fc) == Exception` **iff** `fc >= 0x80`, and the recovered original FC is lossless
  (`original_fc == fc & 0x7F`). **Anchors BC-2.14.006.**

**Harness/bound notes:** none of the harnesses contains a loop, so no `#[kani::unwind(N)]` is
required; sub-A bounds its state space via `kani::assume(len <= 12)`. The functions are pure
free `fn`s (no `HashMap`/`RandomState`), so there is no Kani-unsupported-FFI abort of the kind that
forced VP-004 to *model* `on_data` rather than drive it. Estimated runtime < 1 s per harness.

**SPEC-level note:** VP-022 defines *what must be proven*. The harnesses are authored in F4 TDD
against the implemented `src/analyzer/modbus.rs`; VP-022 is locked (`verification_lock: true`,
hash + date set, `vp-verified-VP-022-<date>` tag) only after the proofs pass at the F6 gate.

---

## 2. Existing VP Impact Assessment (VP-001..VP-021)

**Conclusion: NO existing VP document requires modification by this feature.** The Modbus
analyzer is new, isolated code (`src/analyzer/modbus.rs`, C-22) added under the existing
accepted L2↔L3 analyzer cycle (architecture-delta §6); it introduces no new dependency edge that
differs from `analyzer/http.rs` / `analyzer/tls.rs`. All 21 locked VPs remain semantically
unchanged. Two VPs, however, carry a **verification dependency** — their *proof harnesses / guards*
(not the VP documents) must be updated in F4 so the already-locked proofs stay green. These are
NOT new VPs and NOT VP-document edits; they are required F4 code changes.

| VP | Module | Impact | Action |
|----|--------|--------|--------|
| VP-004 | dispatcher.rs | **Extend (harness update required)** — new port-502 branch | F4 code change to `classify_oracle`; see §3 |
| VP-007 | mitre.rs | **Dependency (must stay green)** — ICS technique additions + recon T0888 + `Finding` field-rename grep | F4 atomic catalog update + grep-pattern change; see §4 |
| VP-016 | reporter/terminal.rs | **Dependency (harness update required)** — ADR-006 Decision 13 renames `Finding.mitre_technique` → `mitre_techniques` | F4 harness construction update; see §7 |
| VP-020 | reporter/csv.rs | **Dependency (harness update required)** — same field rename; multi-tag semicolon-join CSV cell | F4 harness update + new multi-tag cell case; see §7 |
| VP-021 | reassembly/mod.rs | **Dependency (test-helper update required)** — same field rename in `Finding` construction helpers | F4 test-helper update; property unaffected; see §7 |
| VP-001..003, 005, 006, 008..015, 017..019, 022 | various | **None** | No change (VP-022 authored new with `mitre_techniques: vec![...]`) |

---

## 3. VP-004 Dispatcher-Classify-Precedence — EXTENDED, not broken (REQUIRED F4 harness update)

VP-004 (`verify_content_first_precedence_exhaustive`, `src/dispatcher.rs` kani_proofs)
asserts `classify(&data, &key) == classify_oracle(&data, lower, upper)` over a symbolic 8-byte
`data` and fully symbolic 16-bit ports. The architect's design (architecture-delta §3.3) adds a
**Rule 5: port-502 → `DispatchTarget::Modbus`** arm to the production `classify` function,
placed AFTER all content rules (1–2) and after the 443/8443/80/8080 port fallbacks (3–4).

**Why this is a REQUIRED F4 change, not a VP-document edit:**

- The Kani oracle `classify_oracle` (dispatcher.rs ~line 283) currently returns `None` for a
  port-502 flow that matches no content/TLS/HTTP-port rule. The extended production `classify`
  will return `Modbus`. Without mirroring Rule 5 into the oracle, `got != want` and
  `verify_content_first_precedence_exhaustive` **FAILS** — VP-004 would break.
- The fix is to add the identical Rule-5 arm to `classify_oracle` (architecture-delta §3.6 gives
  the exact code). The oracle and production must agree byte-for-byte.

**This is a VP-004 *harness* update, not a VP-004 *property* change.** VP-004's property statement,
precedence semantics, and locked status are unchanged: content still beats port; 443/8443/80/8080
fallbacks are unchanged; port 502 is reached only when nothing earlier matched, so INV-2
(content-first precedence) for TLS/HTTP is preserved. Because the production source it verifies
changes, the VP-004 harness MUST be updated in the SAME F4 commit that adds Rule 5, and
`cargo kani --harness verify_content_first_precedence_exhaustive` MUST be re-run to confirm
`VERIFICATION:- SUCCESSFUL`. VP-004 is a locked P0 Kani VP; keeping its proof green is a gating
F4/F6 obligation.

**Optional (recommended, non-mandatory):** add `verify_modbus_port_beats_none_not_http_or_tls`
(architecture-delta §3.3) proving port-502 → `Modbus` only when no content rule and no
TLS/HTTP port apply. This strengthens coverage but is not required for VP-004 re-verification.

---

## 4. VP-007 Catalog-Drift-Guard — must STAY GREEN after mitre.rs ICS additions (verification dependency)

VP-007 (`mitre.rs` kani_proofs — the catalog-drift guard / `vp007_catalog_drift_guard`) proves
every emitted MITRE technique ID resolves in `technique_info` and matches the
`T[0-9]{4}(\.[0-9]{3})?` format. The Modbus feature performs the architect's **atomic update set**
to `mitre.rs` (architecture-delta §4.2–§4.3 + F2 directives v2 Decision 12): **six** new ICS
`technique_info` arms (T0836, T0814, T0806, T0835, T0831, **T0888**) plus the seeded-ID array and
emitted-ID set updates. The recon path (BC-2.14.020: FC 0x11 Report Server ID, FC 0x2B/0x0E Read
Device ID) emits **T0888 "Remote System Information Discovery"** (per F2 directives v2 Decision 12
— corrects the v1 T0846 misattribution). **T0846 "Remote System Discovery" stays SEEDED but is NO
LONGER Modbus-EMITTED** (kept in the catalog for completeness / future address-sweep detection).
T0888 is a NEW seeded entry AND is in `EMITTED_IDS`; it is the 7th ICS emitted entry in place of
T0846.

**This is a verification DEPENDENCY, not a new VP.** No VP-007 document edit is implied (VP-007 is
locked / `verification_lock: true`; these are F4 *harness/source* obligations). The obligation is
that the following constants/arrays are updated in the SAME F4 commit so the guard does not detect
drift (per architecture-delta §4.3 + F2 directives v2 Decision 12):

| Constant / array (mitre.rs) | Before (pre-F2) | After (post-F2, directives v2) |
|-----------------------------|-----------------|--------------------------------|
| `technique_info` ICS arms | T0846/T0855/T0856/T0885 (4) | + T0836, T0814, T0806, T0835, T0831, **T0888** (10 ICS total; T0846 kept, not emitted) |
| `SEEDED_TECHNIQUE_IDS` | 15 entries | **21** entries (add the 6 new ICS above) |
| `SEEDED_TECHNIQUE_ID_COUNT` | 15 | **21** |
| `EMITTED_IDS` (kani_proofs) | 6 Enterprise IDs | + T0855, T0836, T0814, T0806, T0835, T0831, **T0888** (13 total: 6 Enterprise + 7 ICS; **T0846 NOT emitted**) |

> **Note on the SEEDED count change (20 → 21).** An earlier (v1) draft of this delta recorded
> SEEDED = 20, assuming the recon path reused the already-seeded T0846 and added only 5 new seeded
> ICS arms (T0836/T0814/T0806/T0835/T0831). Directives v2 Decision 12 instead introduces T0888 as a
> **distinct new seeded ID** for the recon path (T0846 stays seeded but is dropped from emission).
> That is 6 new seeded ICS arms, not 5, so 15 + 6 = **21**. EMITTED stays 13 (the recon emitted slot
> moves from T0846 to T0888).

**EMITTED_IDS — 7 ICS entries (the 7th is T0888 for the recon path; T0846 removed from emission):**

| # | ID | Source | Class |
|---|----|--------|-------|
| 1 | T0855 | write-class FC / unauthorized command + burst companion | ICS |
| 2 | T0836 | 0x06/0x10/0x16 parameter writes | ICS |
| 3 | T0814 | 0x08 Force Listen Only / Restart Comms | ICS |
| 4 | T0806 | write burst or sustained rate exceeded | ICS |
| 5 | T0835 | I/O image manipulation writes (coil-only) | ICS |
| 6 | T0831 | coordinated write sequence (5s window) | ICS |
| 7 | **T0888** | **recon FCs (0x11 Report Server ID, 0x2B/0x0E Read Device ID) — Remote System Information Discovery** | **ICS** |

Combined with the 6 Enterprise emitted IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003),
the runtime-computed `EMITTED_IDS` length is **13** and `SEEDED_TECHNIQUE_ID_COUNT` is **21**.

**Grep-pattern obligation (ADR-006 Decision 13 / directives v2 §13.8).** The `Finding` field is
renamed `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>`. Any VP-007 harness or
catalog-completeness scan that greps analyzer source for emitted IDs MUST change its emission-site
pattern from `mitre_technique: Some` to `mitre_techniques: vec!`. This is an F4 source/harness
change (the locked VP-007 document is not edited). The emitter-half soundness obligation
(sub-property B: every ID in `mitre_techniques` resolves in `technique_info`) is unchanged in intent
— only the field/pattern it scans changes.

**Note (pre-existing gap closed in the same commit):** T0855 is currently seeded but NOT in
`EMITTED_IDS` (architecture-delta §4.3). The Modbus commit adds "T0855" to `EMITTED_IDS`, which
keeps VP-007 sub-property B (emitter half) sound — every emitted ID is also a resolvable/seeded ID.

**Positive-coverage obligation (POL-11 positive-coverage axis — F4 REQUIRED):** the
`vp007_catalog_drift_guard` MUST NOT be allowed to pass false-green over an empty/no-op loop. The
F4 harness therefore MUST:

1. Assert a **runtime-computed** count of IDs actually validated — exactly **13 emitted**
   (`EMITTED_IDS.len() == 13`) **+ 21 seeded** (`SEEDED_TECHNIQUE_ID_COUNT == 21`,
   `SEEDED_TECHNIQUE_IDS.len() == 21`) — computed from the arrays at run time, not hardcoded as a
   literal pass. A counter incremented inside the resolve loop MUST equal that count, so a loop
   that iterates zero times (e.g., an accidentally-emptied `EMITTED_IDS`) FAILS the assertion
   rather than vacuously succeeding.
2. **Deliberately exercise at least one of the newly-added ICS IDs, including the new recon ID
   T0888** (e.g., resolve `T0888` AND one of T0836/T0814/T0806/T0835/T0831) through the full
   `technique_name` + `technique_tactic` resolve path and assert `Some(..)` on both — proving the
   new arms are reachable, not just present as dead constants. T0888 MUST be among the IDs probed
   because it is both the newest seeded arm and the recon emission target. This makes an exit-0 /
   `VERIFICATION:- SUCCESSFUL` impossible to reach without genuinely validating the new ICS catalog
   surface (incl. the T0846→T0888 recon correction).

**Gating obligation:** after the atomic update, `cargo kani` over the `mitre.rs` VP-007 harnesses
MUST report `VERIFICATION:- SUCCESSFUL` AND the positive-coverage assertions above MUST hold. If
any emitted ID lacks a `technique_info` arm or a seeded entry, OR if the runtime-computed
emitted/seeded counts are not 13/21, OR if the new-ICS-ID resolve probe (incl. T0888) returns
`None`, the guard fails. VP-007 is a locked P0 Kani VP; staying green (now with positive-coverage)
is an F4/F6 gate.

---

## 5. Index Consistency Confirmation (architect pre-registration)

Verified against the architect's pre-registration — all three index files are mutually consistent
and match the VP-022 spec authored here:

| Index file | VP-022 row | Counts after pre-registration |
|-----------|-----------|-------------------------------|
| `VP-INDEX.md` | present (Kani, P1, draft, analyzer/modbus.rs) | total 21→22, Kani 8→9, P1 7→8, draft 0→1 |
| `verification-architecture.md` | present in "Should Prove" table + P1 list | P1 7→8, total 21→22, Kani list includes VP-022 |
| `verification-coverage-matrix.md` | present + new analyzer/modbus.rs module row | Kani 8→9, Total 21→22; Totals row Kani 9 / proptest 7 / fuzz 1 / integ-unit 5 = 22 |

**Counts confirmed:** Kani **9**, total **22**, P1 **8**, P0 8, test-sufficient 6, draft 1
(VP-022), verified 21. `9 + 7 + 1 + 5 = 22` (tool totals). `8 + 8 + 6 = 22` (phase totals).
**No inconsistency found** between the architect's pre-registration and the VP-022 spec file.

### BCs reconciliation (RESOLVED — consistency BLOCKING-1 / adversary F-MED-006)

Originally the VP-INDEX `Verified BCs` column for VP-022 enumerated six BCs
(001, 002, 003, 004, 006, 007) while the VP-022 spec file `bcs:` frontmatter listed eight
(adding 005 and 008). The 8-BC set is the correct one: sub-property B's set-membership
obligations anchor to BC-2.14.005 (`classify_fc` totality), BC-2.14.007 (Write set), and
BC-2.14.008 (Diagnostic set). In this F2 fix burst the VP-INDEX VP-022 row was widened to the
full 8-BC set — `BC-2.14.001, .002, .003, .004, .005, .006, .007, .008` — so the VP-INDEX
`Verified BCs` column, the VP-022 frontmatter `bcs:`, and the VP-022 Sub-property→BC anchor table
are now mutually consistent under the architect's canonical BC map (005=totality, 006=exception,
007=Write, 008=Diagnostic). No VP-count change resulted (still total 22, Kani 9, P1 8).

---

## 6. F4 Action Checklist (verification obligations carried forward)

1. Author the four VP-022 Kani harnesses in `src/analyzer/modbus.rs` `#[cfg(kani)] mod kani_proofs`;
   run `cargo kani` → all `SUCCESSFUL`; then lock VP-022 (set `verification_lock: true`,
   `proof_completed_date`, `proof_file_hash`, `status: verified`; create
   `vp-verified-VP-022-<date>` tag).
2. **REQUIRED:** extend `classify_oracle` in dispatcher.rs with the Rule-5 port-502 arm in the
   same commit that adds Rule 5 to production `classify`; re-run VP-004 → `SUCCESSFUL`.
3. **REQUIRED:** perform the mitre.rs atomic update set (6 ICS arms incl. **T0888** + 2 arrays +
   `EMITTED_IDS` incl. T0855 and **T0888** with **T0846 removed from emission** → 13 emitted /
   21 seeded); change the emitted-ID grep pattern from `mitre_technique: Some` to
   `mitre_techniques: vec!`; add the §4 POL-11 positive-coverage assertions (runtime-computed
   13-emitted + 21-seeded count, plus deliberate resolve of ≥1 newly-added ICS ID **including
   T0888**); re-run VP-007 → `SUCCESSFUL` with positive-coverage holding.
4. **REQUIRED (ADR-006 Decision 13):** update the VP-016, VP-020, and VP-021 proof harnesses /
   test helpers to construct `Finding { mitre_techniques: vec![...] }` instead of
   `Finding { mitre_technique: ... }`; for VP-020 add a multi-tag CSV-cell case
   (`vec!["T0855","T0836"]` → `"T0855;T0836"`) and assert it remains injection-safe; re-run each →
   green. See §7.
5. Confirm all other locked VPs (VP-001..003, 005, 006, 008..015, 017..019, 022) remain
   `SUCCESSFUL` / green in the full regression run (no expected change; new module is isolated).

---

## 7. VP-016 / VP-020 / VP-021 — `Finding` field-rename harness obligations (ADR-006 Decision 13)

ADR-006 Decision 13 renames the `Finding` field `mitre_technique: Option<String>` →
`mitre_techniques: Vec<String>` (`#[serde(skip_serializing_if = "Vec::is_empty")]`). Three locked,
test-sufficient VP harnesses construct `Finding` values directly and therefore will **fail to
compile** until their construction sites are migrated. For each, the underlying **property is
unchanged** — only the harness construction syntax (and, for VP-020, one added test case) changes.
These are F4 harness obligations; **no locked VP-document lock field is edited**. The
`proof_file_hash` of each affected VP will be recomputed and the lifecycle row appended in the F4
commit that touches the harness, exactly as the lock protocol prescribes (lock fields themselves —
`verification_lock`, `proof_completed_date` — are NOT changed here; the F4 harness change is a
re-verification of the same locked property against the renamed type).

| VP | Harness file (evidence) | Required F4 change | Property impact |
|----|-------------------------|--------------------|-----------------|
| VP-016 | `tests/reporter_terminal_tests.rs` | Replace every `mitre_technique: None` / `mitre_technique: technique.map(...)` / `Some("T1036")` construction with `mitre_techniques: vec![]` / `mitre_techniques: vec!["T1036"]`. Tactic-grouping uses `mitre_techniques[0]` as the bucket key; the single-technique tests are unchanged in expected output. | **None** — grouping order, Uncategorized-last, and within-bucket sort are all preserved. Empty `vec![]` lands in Uncategorized exactly as the prior `None` did. |
| VP-020 | `tests/reporter_csv_tests.rs` | Replace `Finding { mitre_technique: ... }` constructions with `mitre_techniques: vec![...]`. **ADD a multi-tag CSV-cell case**: a finding with `mitre_techniques: vec!["T0855","T0836"]` serializes column 6 as `"T0855;T0836"` (semicolon-join, no space) and MUST still pass the injection-neutralization assertion (semicolons are neutral, non-formula-trigger characters — no leading `=`/`+`/`-`/`@`/TAB/CR is introduced by the join). Also keep the empty-vec → `""` case (was `None` → `""`). | **None** — `neutralize_csv_injection` is unchanged; the property "no CSV cell starts with a formula-trigger char without a single-quote prefix" still holds for joined multi-value cells. The new case strengthens coverage to the multi-tag column-6 surface. |
| VP-021 | `tests/timestamp_threading_tests.rs` | Update any `Finding { mitre_technique: ... }` constructed in test helpers to `mitre_techniques: vec![...]`. The timestamp-provenance property never reads the technique field. | **None / confirmed unaffected** — VP-021 asserts on `Finding.timestamp` only; the technique field is orthogonal. The change is a mechanical compile-fix of helper construction, not a behavioral change. If the F4 helpers happen not to set the technique field at all (relying on `..make_finding()` defaults that move to `vec![]`), VP-021 may need **no** edit beyond the shared helper. |

**VP-020 multi-tag injection-safety reasoning (explicit, per task):** the semicolon-joined cell
`"T0855;T0836"` begins with `'T'`, which is not in the formula-trigger set
{`=`,`+`,`-`,`@`,`\t`,`\r`}. The `join(";")` operation cannot introduce a leading trigger character
because (a) MITRE IDs always begin with `'T'`, and (b) the separator `;` is interior, never leading.
Therefore the existing single-quote-prefix guard remains sufficient and the BC-2.11.021 property is
preserved for multi-value cells. The added test case pins this so a future change to the join format
(e.g., a leading-separator bug) would be caught.

**Re-verification gate:** after the F4 harness migration, `cargo test` over
`reporter_terminal_tests`, `reporter_csv_tests`, and `timestamp_threading_tests` MUST be green, and
each affected VP's `proof_file_hash` recomputed in the F4 lifecycle row. No property statement, BC
anchor, or VP-INDEX row for VP-016/020/021 changes.

---

## 8. New-VP assessment: does multi-tag attribution warrant VP-023? — NO (extend-existing)

**Decision: NO new VP. The next free VP id (VP-023) is NOT allocated.** VP-INDEX stays at **22**
(VP-022 remains the only F2-new VP). The candidate multi-tag properties are already covered by
existing VPs once the §7 harness extensions land; the gaps are closed by *anchors/extensions*, not
a new property. Per the "prefer extending existing over new VP unless there is a real gap" rule:

| Candidate multi-tag property | Covered by | Verdict |
|------------------------------|------------|---------|
| **No technique tag lost across emission → serialization** (every ID placed in `mitre_techniques` survives into JSON/CSV) | **VP-007** sub-property B (emitter-catalog completeness) proves every emitted ID resolves; **VP-017** (JsonReporter key-order determinism / round-trip) and **VP-020** (CSV cell content, now multi-tag) prove the field's values are emitted faithfully. Serde `#[derive(Serialize)]` over `Vec<String>` emits the vec verbatim — no element-dropping path exists. | **Covered** — VP-017 (JSON) + VP-020 (CSV, §7 multi-tag case) + VP-007 (resolvability). No round-trip element-loss path to prove separately. |
| **Vec ordering determinism** (the order of tags within `mitre_techniques` is stable across runs) | The order is fixed by the emission site (a literal `vec!["T0855","T0836"]` constructed in source); `serde` preserves `Vec` order; there is no sort or `HashSet` in the path. **VP-017**'s determinism property (deterministic JSON output for fixed input) already covers stable field-value ordering. | **Covered** — determinism is structural (literal vec, order-preserving serialize); VP-017 anchors the JSON-output-determinism axis. No nondeterministic source (no HashSet/HashMap iteration) feeds tag order. |
| **CSV multi-value cell stays injection-safe** | **VP-020** (extended in §7 with the `"T0855;T0836"` case). | **Covered** — VP-020 extension. |
| **Empty vec == prior None semantics** (no key in JSON / empty CSV cell / Uncategorized bucket) | **VP-016** (Uncategorized), **VP-017** (skip-when-empty JSON), **VP-020** (empty CSV cell). | **Covered** — VP-016/017/020. |

**Anchors added instead of a new VP:** VP-020's anchor set is conceptually widened to include the
multi-tag CSV-cell case (recorded in §7; no BC-anchor or VP-INDEX `Verified BCs` change because
BC-2.11.021 already governs the whole CSV-injection surface, and BC-2.11.024 — semicolon-join — is
a product-owner-owned BC revision, not a VP anchor). No VP-document `bcs:` frontmatter changes.

**Conclusion:** the multi-tag change is fully covered by VP-007 (resolvability/no-loss-of-validity),
VP-017 (JSON determinism + skip-empty), VP-016 (terminal grouping / Uncategorized), and VP-020
(CSV injection + multi-value cell). **VP-INDEX remains at total 22.** No Kani/total arithmetic change.

---

## 9. Index consistency re-confirmation after §4–§8 edits

These edits touch ONLY this `verification-delta.md` (and do not edit any locked VP document, BC, or
index file). The three index files are therefore unchanged and remain mutually consistent at the
values the architect pre-registered for VP-022:

- **VP-INDEX.md:** total **22**, Kani **9**, P0 8, P1 **8**, test-sufficient 6, draft 1 (VP-022),
  verified 21. Tool totals `9 + 7 + 1 + 5 = 22`. Phase totals `8 + 8 + 6 = 22`.
- **verification-architecture.md:** 22 VP rows; VP-022 in Should-Prove + P1 list; Kani list of 9.
- **verification-coverage-matrix.md:** 22 VP rows; Totals row Kani 9 / proptest 7 / fuzz 1 /
  integration-unit 5 = 22.

No new VP (no VP-023) ⇒ **no Kani-count or total-count change** from this reconciliation. The only
substantive numeric change in this delta is internal to the VP-007 *catalog* dependency:
`SEEDED_TECHNIQUE_ID_COUNT` 20 → **21** and the recon emitted ID T0846 → **T0888** (EMITTED stays
**13**). These are F4 source/harness obligations, not VP-index counts, so no index arithmetic is
affected.
