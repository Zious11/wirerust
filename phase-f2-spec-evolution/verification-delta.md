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
traces_to:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.001.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.005.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.006.md
---

# F2 Verification Delta — Issue #7: Modbus TCP Protocol Analyzer

This document is the formal-verifier's Phase-F2 verification delta. It records the one new
verification property (VP-022), confirms no existing VP (VP-001..021) requires modification by
this feature, and flags two verification *dependencies* (VP-004 harness extension, VP-007
catalog-drift-guard) that must be honored in F4 to keep already-locked proofs green.

The architect has already PRE-REGISTERED VP-022 in the three index files (counts bumped:
Kani 8→9, total 21→22, P1 7→8). This delta does not re-touch those indexes; it authors the
VP-022 spec file and documents the regression surface.

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
| VP-007 | mitre.rs | **Dependency (must stay green)** — ICS technique additions | F4 atomic catalog update; see §4 |
| VP-001..003, 005, 006, 008..021 | various | **None** | No change |

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
to `mitre.rs` (architecture-delta §4.2–§4.3): five new ICS `technique_info` arms (T0836, T0814,
T0806, T0835, T0831) plus the seeded-ID array and emitted-ID set updates. The recon path
(BC-2.14.020: FC 0x11 Report Server ID, FC 0x2B/0x0E Read Device ID) emits **T0846**
(per F2 directives §8); T0846 is already SEEDED, so it adds no seeded entry but DOES add a 7th
ICS entry to `EMITTED_IDS`.

**This is a verification DEPENDENCY, not a new VP.** No VP-007 document edit is implied (VP-007 is
locked / `verification_lock: true`; these are F4 *harness/source* obligations). The obligation is
that the following constants/arrays are updated in the SAME F4 commit so the guard does not detect
drift (per architecture-delta §4.3 + F2 directives §8):

| Constant / array (mitre.rs) | Before | After |
|-----------------------------|--------|-------|
| `technique_info` ICS arms | T0846/T0855/T0856/T0885 | + T0836, T0814, T0806, T0835, T0831 |
| `SEEDED_TECHNIQUE_IDS` | 15 entries | 20 entries (add the 5 above) |
| `SEEDED_TECHNIQUE_ID_COUNT` | 15 | 20 |
| `EMITTED_IDS` (kani_proofs) | 6 Enterprise IDs | + T0855, T0836, T0814, T0806, T0835, T0831, **T0846** (13 total: 6 Enterprise + 7 ICS) |

**EMITTED_IDS — 7 ICS entries (the 7th is T0846 for the recon path):**

| # | ID | Source | Class |
|---|----|--------|-------|
| 1 | T0855 | write-class FC / unauthorized command + burst companion | ICS |
| 2 | T0836 | 0x06/0x10/0x16 parameter writes | ICS |
| 3 | T0814 | 0x08 Force Listen Only / Restart Comms | ICS |
| 4 | T0806 | write burst rate exceeded | ICS |
| 5 | T0835 | I/O image manipulation writes (coil-only) | ICS |
| 6 | T0831 | coordinated write sequence (5s window) | ICS |
| 7 | **T0846** | **recon FCs (0x11 Report Server ID, 0x2B/0x0E Read Device ID)** | **ICS** |

Combined with the 6 Enterprise emitted IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003),
the runtime-computed `EMITTED_IDS` length is **13** and `SEEDED_TECHNIQUE_ID_COUNT` is **20**.

**Note (pre-existing gap closed in the same commit):** T0855 is currently seeded but NOT in
`EMITTED_IDS` (architecture-delta §4.3). The Modbus commit adds "T0855" to `EMITTED_IDS`, which
keeps VP-007 sub-property B (emitter half) sound — every emitted ID is also a resolvable/seeded ID.

**Positive-coverage obligation (POL-11 positive-coverage axis — F4 REQUIRED):** the
`vp007_catalog_drift_guard` MUST NOT be allowed to pass false-green over an empty/no-op loop. The
F4 harness therefore MUST:

1. Assert a **runtime-computed** count of IDs actually validated — exactly **13 emitted**
   (`EMITTED_IDS.len() == 13`) **+ 20 seeded** (`SEEDED_TECHNIQUE_ID_COUNT == 20`,
   `SEEDED_TECHNIQUE_IDS.len() == 20`) — computed from the arrays at run time, not hardcoded as a
   literal pass. A counter incremented inside the resolve loop MUST equal that count, so a loop
   that iterates zero times (e.g., an accidentally-emptied `EMITTED_IDS`) FAILS the assertion
   rather than vacuously succeeding.
2. **Deliberately exercise at least one of the newly-added ICS IDs** (e.g., resolve `T0846` AND
   one of T0836/T0814/T0806/T0835/T0831) through the full `technique_name` + `technique_tactic`
   resolve path and assert `Some(..)` on both — proving the new arms are reachable, not just
   present as dead constants. This makes an exit-0 / `VERIFICATION:- SUCCESSFUL` impossible to
   reach without genuinely validating the new ICS catalog surface.

**Gating obligation:** after the atomic update, `cargo kani` over the `mitre.rs` VP-007 harnesses
MUST report `VERIFICATION:- SUCCESSFUL` AND the positive-coverage assertions above MUST hold. If
any emitted ID lacks a `technique_info` arm or a seeded entry, OR if the runtime-computed
emitted/seeded counts are not 13/20, OR if the new-ICS-ID resolve probe returns `None`, the guard
fails. VP-007 is a locked P0 Kani VP; staying green (now with positive-coverage) is an F4/F6 gate.

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
3. **REQUIRED:** perform the mitre.rs atomic update set (5 arms + 2 arrays + `EMITTED_IDS`
   incl. T0855 and **T0846** → 13 emitted / 20 seeded); add the §4 POL-11 positive-coverage
   assertions (runtime-computed 13-emitted + 20-seeded count, plus deliberate resolve of ≥1
   newly-added ICS ID incl. T0846); re-run VP-007 → `SUCCESSFUL` with positive-coverage holding.
4. Confirm all other locked VPs (VP-001..003, 005, 006, 008..021) remain `SUCCESSFUL` / green in
   the full regression run (no expected change; new module is isolated).
