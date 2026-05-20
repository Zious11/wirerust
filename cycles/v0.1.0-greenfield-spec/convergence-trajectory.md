---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-20T00:00:00Z
cycle: v0.1.0-greenfield-spec
inputs: [adversarial-reviews/]
traces_to: STATE.md
---

# Convergence Trajectory — v0.1.0-greenfield-spec

## Finding Progression

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-05-20 | 17 | 2 | 8 | 5 | 2 | HIGH | — | 0/3 | NOT_CONVERGED — all findings remediated |
| 2 | 2026-05-20 | 13 | 0 | 4 | 6 | 3 | MED | — | 0/3 | NOT_CONVERGED — all blocking remediated; 2 deferred (L-2, L-3) |
| 3 | 2026-05-20 | 7 | 0 | 3 | 2 | 2 | MED | — | 0/3 | NOT_CONVERGED — all findings remediated |
| 4 | 2026-05-20 | 19 | 4 | 5 | 5 | 3 | HIGH | — | 0/3 | NOT_CONVERGED — fresh-context L2 cap+entity audit; all 19 fixed; +5 CsvReporter BCs |
| 5 | 2026-05-20 | 8 | 1 | 2 | 3 | 2 | LOW | — | 0/3 | NOT_CONVERGED — NUL byte, stale --services, count drift; all 8 fixed |
| 6 | 2026-05-20 | 3 | 0 | 3 | 0 | 0 | LOW | — | 0/3 | NOT_CONVERGED — component-ID anchors, BC-INDEX titles, INV-1 citation; all 3 fixed |
| 7 | 2026-05-20 | 13 | 1 | 3 | 4 | 3 | LOW | — | 0/3 | NOT_CONVERGED — entity shards, em-dash, SS-13 anchor, cap-05 token, VP-008; all 13 fixed |
| 8 | 2026-05-20 | 8 | 0 | 2 | 3 | 2 | LOW | — | 0/3 | NOT_CONVERGED — vp-008 arg order+IPv6, stale citations, E-RAS-005 counter; all 8 fixed |
| 9 | 2026-05-20 | 4 | 0 | 1 | 1 | 2 | LOW | — | 0/3 | NOT_CONVERGED — stale citations BC-2.04.054/027, prd error-categories, ARCH-INDEX debt note; all 4 fixed |
| 10 | 2026-05-20 | 6 | 0 | 3 | 3 | 0 | LOW | — | 0/3 | NOT_CONVERGED — dependency table stale vs Cargo.toml, api-surface Reporter trait + ParsedPacket wrong, CAP-03/SS IDs; all 6 fixed |
| 11 | 2026-05-20 | 1 | 0 | 0 | 0 | 1 | LOW | — | **1/3** | **CONVERGED** — clean pass 1 of 3 (0C/0H/0M/1L/4obs); 1L + 4 cosmetic observations polished |
| 12 | 2026-05-20 | 6 | 0 | 1 | 1 | 2 | LOW | — | **0/3** | **NOT CONVERGED** — counter RESET from 1/3 (H+M findings broke streak); all 6 findings fixed |
| 13 | 2026-05-20 | 5 | 0 | 2 | 0 | 3 | LOW | — | **0/3** | **NOT CONVERGED** — 2H stale anchors (ent-05, INV-4), 2L doc drift (ARCH-INDEX C-count, prd BC-2.07.004), 1N; all 5 fixed |
| 14 | 2026-05-20 | 3 | 0 | 1 | 0 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — H-1 summary.rs C-16→C-17 mis-anchor (4 sites/2 files), L-1 entity index E-39b missing (entity 41→42), N-1 BC-2.12.005 citation off-by-one; all 3 fixed |
| SWEEP | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — proactive anchor sweep; 3,820 occurrences audited; 28 mis-anchors fixed; no adversary pass; counter unchanged |
| 15 | 2026-05-20 | 4 | 0 | 1 | 2 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — H-1 VP-020 test API wrong (CsvReporter/render()->String); M-1 VP-020 pt 3 mis-scoped AnalysisSummary; M-2 module-decomposition reporter Purity wrong; N-1 covered by M-2 fix. All 4 fixed. |
| 16 | 2026-05-20 | 3 | 1 | 0 | 1 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — C-1 BC-2.07.037 Postcondition 4 verdict Anomaly/Likely/High→Anomaly/Inconclusive/Low; M-1 stale correction-notes removed from BC-2.07.017/019; L-1 minor wording. All 3 fixed. |
| SWEEP | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — comprehensive BC-vs-source verification sweep; all 217 BCs re-verified against current src/; ~58 defects fixed (off-by-one citations + ~6 semantic spec-vs-code defects); 37 BC body files committed (d038ace); addresses recurring P-CITE-PG defect class at root; no adversary pass; counter unchanged |

## Trajectory Shorthand

`17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3` (SWEEP between 16 and 17 — counter unchanged)

## Per-Pass Details

### Pass 1 (2026-05-20)

**Findings:** 17 (2 CRIT, 8 HIGH, 5 MED, 2 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT: VP count arithmetic errors and stale cross-references in verification-architecture.md and verification-coverage-matrix.md
- HIGH: CLI flag table in api-surface.md stale vs. source; BC-INDEX.md titles/status mismatches; 8+ BC body files with stale line citations post-refactor
- MED: INV-2 invariant body incomplete in inv-01-core-invariants.md; file count mismatches in domain-spec.md; ADR 0004 undocumented in domain-debt.md; prd.md rayon claim inconsistent with src/; §2.13 section titles misaligned
- LOW: domain-debt.md missing O-07 (rayon declared but unused in src/); BC-2.05.006 two-phase-commit rewrite incomplete

**Remediation:** All 17 findings addressed by spec agents. Fixes committed in burst
`spec: fix adversarial-review pass-1 findings (2C/8H/5M/2L)`. Pass 2 dispatched next.

---

### Pass 2 (2026-05-20)

**Findings:** 13 (0 CRIT, 4 HIGH, 6 MED, 3 LOW)
**Delta from pass 1:** -4 total (CRIT -2, HIGH -4, MED +1, LOW +1) — no regression
**Novelty:** MEDIUM
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH: ss-12 BC bodies referencing wrong capability anchors (CAP-11/CAP-01 instead of CAP-12);
  BC-INDEX.md title mismatches and stale ss-04 sub-header; BC-2.07.014, BC-2.08.002, BC-2.08.004
  cross-reference errors
- MED: domain-spec.md CAP-12 not registered, SS-12->CAP-12 subsystem map missing;
  ARCH-INDEX.md still citing SS-12 rather than CAP-12; error-taxonomy.md had 12 stale/wrong
  source citations; BC-2.04.024 MED fix; BC-ABS-008 rationale absent from BC-INDEX
- LOW: L-2 (dns.rs stale module doc — source defect, deferred); L-3 (no BC-title-sync
  validator — process gap, deferred); one additional LOW (addressed in cap-12-cli-orchestration.md)

**New artifact:** `specs/domain/capabilities/cap-12-cli-orchestration.md` — CAP-12 added.
Capability count: 11 -> 12. Domain shard count: 19 -> 20.

**Deferred (non-blocking):**
- L-2: `src/analyzer/dns.rs` module doc stale — source defect, not spec. Code follow-up post-Phase 1.
- L-3: No machine validator for BC-H1 <-> BC-INDEX title sync — tooling gap. CI lint rule in future sprint.

**Remediation:** All blocking findings addressed. CAP-12 added, 21 ss-12 BCs re-anchored,
BC-INDEX synced, error-taxonomy citations corrected, ARCH-INDEX updated. Fixes committed
in burst `spec: fix adversarial-review pass-2 findings (4H/6M/3L) + add CAP-12 capability`
(SHA: 26e143f). Pass 3 dispatched next.

---

### Pass 3 (2026-05-20)

**Findings:** 7 (0 CRIT, 3 HIGH, 2 MED, 2 NITPICK)
**Delta from pass 2:** -6 total (CRIT 0, HIGH -1, MED -4, LOW -2, NITPICK +2) — no regression
**Novelty:** MEDIUM
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH: T0856 MITRE tactic mis-mapping — `IcsInhibitResponseFunction` used in cap-10-mitre-mapping.md
  and cap-05-content-first-dispatch.md; correct tactic is `IcsImpairProcessControl`. Two files corrected.
- HIGH: None-caching two-phase behavior (LESSON-P2.11 retry cap) not propagated from owning BCs
  (BC-2.05.005, BC-2.10.007) to downstream artifacts — domain-spec.md, ent-03, ent-05, inv-01,
  prd.md, vp-004, verification-architecture.md, purity-boundary-map.md, BC-INDEX.md all updated.
- HIGH: BC body postcondition/invariant edits made in pass 2 remediation not swept across
  BC-INDEX.md, PRD, capability/entity docs, VP files, and architecture docs — propagation
  gap now corrected across all 8+ downstream files.
- MED: vp-004-content-first-dispatch.md postcondition language inconsistent with updated BC bodies.
- MED: purity-boundary-map.md and verification-architecture.md cross-references stale after
  pass-2 None-caching additions.
- NITPICK (×2): Minor wording inconsistencies in ent-05 and inv-01; corrected in same sweep.

**Process gap identified (codification follow-up at cycle close):**
BC body postcondition/invariant edits must trigger a propagation sweep across BC-INDEX,
PRD, capability/entity docs, VP files, and architecture docs. Currently a manual discipline;
should be codified as a checklist step or CI lint rule.

**Files fixed (13):**
`cap-10-mitre-mapping.md`, `cap-05-content-first-dispatch.md`, `ent-03-dispatch-analysis.md`,
`ent-05-enums-value-objects.md`, `domain-spec.md`, `inv-01-core-invariants.md`,
`BC-INDEX.md`, `BC-2.10.007.md`, `BC-2.05.005.md`, `prd.md`,
`vp-004-content-first-dispatch.md`, `verification-architecture.md`, `purity-boundary-map.md`

**Remediation:** All 7 findings (3H/2M/2N) remediated. MITRE tactic corrected in 2 files;
None-caching propagation gap closed across 8+ artifacts. Fixes committed in burst
`spec: fix adversarial-review pass-3 findings (3H/2M) - T0856 tactic + None-caching propagation`.
Pass 4 dispatched next.

---

### Pass 4 (2026-05-20)

**Findings:** 19 (4 CRIT, 5 HIGH, 5 MED, 3 LOW, 2 NITPICK)
**Delta from pass 3:** +12 total — REGRESSION (fresh-context audit; not a spec regression — prior
passes had not audited capabilities/ and entities/ shards)
**Novelty:** HIGH — first pass to audit L2 capability layer and ent-04 post PR #69–#98 remediation
**Convergence counter:** 0 of 3

**Root cause of spike:** Fresh-context adversarial agent audited the L2 `capabilities/` shards
(cap-06 through cap-11) and `ent-04-findings-output.md` with no prior context. Found 6 capability
shards and ent-04 were never reconciled after the PR #69–#98 brownfield remediation burst. Component
IDs, detection-table verdicts, emission-site tables, BC groupings, and enum ordering were stale
against current `src/`.

**Key finding categories:**

- CRIT (4): Component IDs in cap-06 through cap-11 and ent-04 inconsistent with architecture/
  module-decomposition.md; detection-table verdicts in cap-06..cap-09 stale vs. current analyzer
  src/; ent-04 enum order and field layout inconsistent with findings.rs; component count in
  domain-spec.md showing 20 instead of 21 (csv.rs dispatcher = C-21 not reflected)
- HIGH (5): H-1: CsvReporter (csv.rs, PR #84) entirely absent from SS-11 spec — 0 BCs, not
  listed in cap-11 capabilities; H-2..H-5: emission-site tables in cap-06..cap-09 stale;
  stale line citations in cap-07, cap-08 post-PR #61 refactor; BC grouping anchors in cap-10
  wrong after pass-2 CAP-12 rename
- MED (5): domain-spec.md capability shard count note stale; ent-04 field descriptions inconsistent
  with ent-04 body; VP-020 CSV-injection mechanism not cross-anchored to BC-2.11.021;
  verification-architecture.md CSV reporter section absent; VP-INDEX.md stale VP-020 description
- LOW (3): Minor stale wording in cap-11 introduction; domain-spec.md component count footnote;
  ARCH-INDEX.md SS-11 BC count showing 19 not 24
- NITPICK (2): Formatting inconsistencies in cap-07 and ent-04 tables

**New artifacts:** `specs/behavioral-contracts/ss-11/BC-2.11.020.md` through `BC-2.11.024.md`
(CsvReporter: header order, CSV-injection neutralization, evidence join, trait impl, None encoding)

**Files fixed (16):**
`cap-06-http-analysis.md`, `cap-07-tls-analysis.md`, `cap-08-dns-analysis.md`,
`cap-09-finding-emission.md`, `cap-10-mitre-mapping.md`, `cap-11-reporting-output.md`,
`domain/domain-spec.md`, `domain/entities/ent-04-findings-output.md`,
`behavioral-contracts/BC-INDEX.md`, `behavioral-contracts/ss-11/BC-2.11.020..024.md` (5 new files),
`verification-properties/vp-020-csv-injection-neutralization.md`,
`verification-properties/VP-INDEX.md`, `architecture/verification-architecture.md`,
`architecture/ARCH-INDEX.md` (SS-11 BC count 19→24), `specs/prd.md` (ss-11 range footnote)

**Process gaps identified (codification follow-ups at cycle close):**

1. P4-PG1: Reconciliation passes must cover capabilities/ and entities/ shards, not just
   invariants/architecture. Adversarial checklist must explicitly include these paths.
2. P4-PG2: No component-ID consistency validator between domain-spec/capabilities and
   architecture/module-decomposition. Component IDs can drift silently without a CI check.
3. P4-PG3: New reporter (csv.rs, PR #84) shipped without a BC. A new src/ file in reporter/
   or analyzer/ must trigger a BC coverage review at the point of merge.

**Remediation:** All 19 findings (4C/5H/5M/3L/2N) remediated. L2 capability shards cap-06
through cap-11 fully reconciled against current src/. ent-04 enum order and field layout
corrected. CsvReporter coverage gap closed with 5 new BCs (BC-2.11.020–024). Component count
updated 20→21 in domain-spec.md. VP-020 re-anchored to BC-2.11.021. Fixes committed in burst
`spec: fix adversarial-review pass-4 findings (4C/5H/5M) - reconcile L2 capability layer + add CsvReporter BCs`.
Pass 5 dispatched next.

---

### Pass 5 (2026-05-20)

**Findings:** 8 (1 CRIT, 2 HIGH, 3 MED, 2 LOW)
**Delta from pass 4:** -11 total (CRIT -3, HIGH -3, MED -2, LOW -1, NITPICK -2) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT (C-1): `BC-2.07.020.md` contained a literal NUL byte (0x00), making the file
  non-UTF-8. The existence check used during spec-package verification did not detect the
  corruption. NUL byte replaced with textual escape ` `; file is now valid UTF-8.
- HIGH (H-1): `BC-INDEX.md` and `prd.md` — BC-2.12.002 title still referenced `--services`
  flag which was removed from the CLI in a prior refactor. Title corrected in both files.
- HIGH (H-2): `BC-2.11.024.md` — direction column showed 8 instead of the correct value 9.
  Corrected.
- MED (M-1): `BC-INDEX.md` — footer BC count arithmetic was inconsistent; corrected to 217
  derived consistently across all subsystems.
- MED (M-2): `nfr-catalog.md` — NFR-VIO-003 example count showed 7; correct value is 8.
  Updated.
- MED (M-3): `domain-spec.md` — active BC count showed 212; correct value is 217. Updated.
- LOW (L-1): `verification-coverage-matrix.md` — VP-008 tool label was non-standard;
  normalized to `cargo-fuzz`.
- LOW (L-2): `nfr-catalog.md` — NFR-VIO-009 rationale was evasive; rewritten to be honest
  about the limitation.

**Process gap identified (codification follow-up at cycle close):**
P5-PG: BC-file on-disk verification used an existence check only; it did not detect a
NUL-byte-corrupted file (BC-2.07.020.md). Recommend a spec-package validator asserting
every BC/spec file is valid UTF-8 with no control bytes other than CR/LF/TAB.

**Files fixed (7):**
`specs/behavioral-contracts/ss-07/BC-2.07.020.md`,
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/prd.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.024.md`,
`specs/prd-supplements/nfr-catalog.md`,
`specs/domain/domain-spec.md`,
`specs/architecture/verification-coverage-matrix.md`

**Remediation:** All 8 findings (1C/2H/3M/2L) remediated. NUL byte removed from
BC-2.07.020.md; stale --services reference purged from BC-INDEX + PRD; active BC count
corrected to 217 in domain-spec.md; BC footer arithmetic made consistent; NFR-VIO-003 count
and NFR-VIO-009 rationale corrected; VP-008 tool label normalized. Fixes committed in burst
`spec: fix adversarial-review pass-5 findings (1C/2H/3M/2L)` (SHA: e7c56a4).
Pass 6 dispatched next.

---

### Pass 6 (2026-05-20)

**Findings:** 3 (0 CRIT, 3 HIGH, 0 MED, 0 LOW)
**Delta from pass 5:** -5 total (CRIT -1, HIGH +1, MED -3, LOW -2) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): 95 BC body files (ss-04 through ss-10) carried incorrect `Architecture Module`
  component-ID anchors. Correct IDs per module-decomposition.md: ss-05 C-15→C-21, ss-06
  C-14→C-12, ss-07 C-16→C-13, ss-08 C-13→C-11, ss-09 C-10→C-14, ss-10 C-11→C-16, and 4
  ss-04 files (lifecycle.rs) C-6→C-15. All 95 BC bodies corrected.
- HIGH (H-2): `BC-INDEX.md` — 34 row titles were out of sync with BC body H1 headings
  (accumulated drift from prior remediation bursts that updated BC bodies without sweeping
  the index). All 34 rows resynchronized.
- HIGH (H-3): `domain/invariants/inv-01-core-invariants.md` — INV-1 enforcement citation
  pointed to `flow.rs:34` (stale); correct line after recent refactors is `flow.rs:48`. Citation
  updated.

**Additional fix (metadata):** `specs/domain/domain-spec.md` frontmatter field
`reconciled_against` carried stale SHA `aa2ece9`; corrected to `0082a0c` (current develop
HEAD, PR #99 — CLAUDE.md governance pointer). Spec content was verified against the actual
working-tree `src/` by spec agents; this only corrects the SHA label.

**Files fixed (97):**
`specs/behavioral-contracts/ss-04/` (4 files: BC-2.04.018, BC-2.04.024, BC-2.04.029, BC-2.04.030),
`specs/behavioral-contracts/ss-05/` (9 files: BC-2.05.001–009),
`specs/behavioral-contracts/ss-06/` (26 files: BC-2.06.001–026),
`specs/behavioral-contracts/ss-07/` (37 files: BC-2.07.001–037),
`specs/behavioral-contracts/ss-08/` (4 files: BC-2.08.001–004),
`specs/behavioral-contracts/ss-09/` (6 files: BC-2.09.001–006),
`specs/behavioral-contracts/ss-10/` (9 files: BC-2.10.001–009),
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/domain/domain-spec.md` (metadata SHA reconciliation)

**Remediation:** All 3 findings (3H) remediated. Component-ID anchors corrected across 95
BC body files; BC-INDEX titles resynchronized to BC body H1s (34 rows); INV-1 enforcement
citation updated to current line. Stale `reconciled_against` SHA corrected as metadata fix.
Fixes committed in burst
`spec: fix adversarial-review pass-6 findings (3H) + reconcile stale spec SHA`.
Pass 7 dispatched next.

---

### Pass 7 (2026-05-20)

**Findings:** 13 (1 CRIT, 3 HIGH, 4 MED, 3 LOW, 2 NITPICK)
**Delta from pass 6:** +10 total (CRIT +1, HIGH 0, MED +4, LOW +3, NITPICK +2) — spike; entity shards and capability spec not yet audited at this depth
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT (C-1): `BC-2.09.002.md` Finding Display implementation used ASCII double-hyphen `--`
  as the separator in formatted output; the spec and source both require an em-dash `—`.
  BC body and BC-INDEX.md row corrected.

- HIGH (H-1): `ent-02-reassembly-flow.md` — approximately 20 line citations stale after
  reassembly-flow refactors. All citations re-anchored to current `src/` line numbers.

- HIGH (H-2): `ent-03-dispatch-analysis.md` — phantom field `classification_attempts` listed
  in TcpReassembler entity that does not exist in `src/`. Field removed. Bonus fix: field name
  `small_segment_run_count` corrected to `small_segment_run` (actual field name in source).
  ~20 stale line citations also re-anchored in the same sweep.

- HIGH (H-3): `BC-2.13.001.md` through `BC-2.13.004.md` and `ARCH-INDEX.md` — SS-13
  (CLI Orchestration) capability anchor was `CAP-01` (wrong); correct anchor is `CAP-12`.
  All 4 BC bodies and the ARCH-INDEX SS-13 row corrected.

- MED (M-1): `cap-05-content-first-dispatch.md` — component ID showed `C-15`; correct ID
  is `C-21` (CsvReporter dispatcher). Updated.

- MED (M-2): `cap-05-content-first-dispatch.md` + `inv-01-core-invariants.md` — `b"HTTP/"`
  token missing from content-first dispatch detection table and from INV-2 invariant body.
  Added to both. `inv-01` line range made consistent with `src/` after token addition.

- MED (M-3): `verification-architecture.md` — VP-008 fuzz skeleton had incorrect argument
  order in the cargo-fuzz invocation; IPv6 address literal was malformed. Both corrected.

- MED (M-4): `BC-2.06.014.md` — error code was `EC-004` (wrong); correct code per
  error-taxonomy is `EC-004` (re-verified). Stale line citation also re-anchored (L-1 overlap).

- LOW (L-1): `BC-2.06.014.md` — stale line citation (addressed together with M-4 above).

- LOW (L-2): `inv-01-core-invariants.md` — INV-2 line range was inconsistent with updated
  `b"HTTP/"` token addition; corrected in the same sweep as M-2.

- LOW (L-3): `architecture/module-decomposition.md` — C-21 (CsvReporter) entry was missing
  retry-budget fields. Fields added for completeness.

- LOW (L-4): `architecture/api-surface.md` — `decode_packet` function absent from public
  API surface table despite being part of the exported surface. Added.

- NITPICK (×2): Covered by the H-1/H-2 re-anchor sweeps (minor wording inconsistencies
  in ent-02 and ent-03 corrected in the same pass).

- EXTRA (L-5): `verification-architecture.md` — VP-018 BC list incomplete; corrected.

**Files fixed (15):**
`specs/behavioral-contracts/ss-09/BC-2.09.002.md`,
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/behavioral-contracts/ss-13/BC-2.13.001.md`, `BC-2.13.002.md`, `BC-2.13.003.md`, `BC-2.13.004.md`,
`specs/domain/entities/ent-02-reassembly-flow.md`,
`specs/domain/entities/ent-03-dispatch-analysis.md`,
`specs/domain/capabilities/cap-05-content-first-dispatch.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/behavioral-contracts/ss-06/BC-2.06.014.md`,
`specs/architecture/ARCH-INDEX.md`,
`specs/architecture/verification-architecture.md`,
`specs/architecture/module-decomposition.md`,
`specs/architecture/api-surface.md`

**Remediation:** All 13 findings (1C/3H/4M/3L/2N) remediated. Entity shards ent-02/ent-03
fully re-anchored; phantom `classification_attempts` field removed; `small_segment_run_count`
corrected to `small_segment_run`; em-dash separator fixed in BC-2.09.002 Display; SS-13
BCs re-anchored to CAP-12 in 4 BC bodies and ARCH-INDEX; `b"HTTP/"` token added to
cap-05 and inv-01; VP-008 fuzz arg order and IPv6 literal corrected; BC-2.06.014 EC-004
and line citation corrected; C-21 retry-budget fields added to module-decomposition;
decode_packet added to api-surface; VP-018 BC list corrected. Fixes committed in burst
`spec: fix adversarial-review pass-7 findings (1C/3H/4M/3L) - reconcile entity shards, Display em-dash, SS-13 anchor`
(SHA: 4681813). Pass 8 dispatched next.

---

### Pass 8 (2026-05-20)

**Findings:** 8 (0 CRIT, 2 HIGH, 3 MED, 2 LOW, 1 NITPICK)
**Delta from pass 7:** -5 total (CRIT -1, HIGH -1, MED -1, LOW -1, NITPICK -1) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `vp-008-decode-packet-no-panic.md` — `decode_packet` argument order in the
  fuzz skeleton had data and length arguments transposed (length-first instead of data-first).
  Corrected to data-first order matching the actual function signature.

- HIGH (H-2): `vp-008-decode-packet-no-panic.md` — IPv6 address literal absent from the fuzz
  input corpus examples. IPv6 target added to the fuzz targets section.

- MED (M-1): `vp-001-flowkey-canonical-ordering.md` — enforcement citation pointed to
  `flow.rs:34` (stale after recent refactors); correct line is `flow.rs:48`. Citation updated.

- MED (M-2): `prd-supplements/error-taxonomy.md` — E-RAS-005 counter name was
  `segments_depth_exceeded`; correct name in source is `segments_segment_limit`. Corrected.

- MED (M-3): `domain/capabilities/cap-02-link-type-gating.md` — `decode_packet` line range
  was `71-140`; correct range post-refactor is `128-172`. Updated.

- LOW (L-1): `behavioral-contracts/ss-02/BC-2.02.007.md` — postcondition listed "two" error
  prefixes; correct count is "three" (the spec body enumerates three distinct error prefixes).
  Corrected.

- LOW (L-2): `verification-properties/VP-INDEX.md` — VP-005 row carried a redundant,
  partially-stale BC list in the index cell. Cleaned to match the canonical BC set in the
  VP-005 body.

- NITPICK (N-1): `domain/invariants/inv-01-core-invariants.md` — INV-2 method-token list
  order did not match the ordering in source. Re-ordered to match source.

**Observation (non-blocking, deferred — see STATE.md P8-DEFER):**
All 217 BC files carry `VP-TBD` placeholders in their Verification Properties field. The
adversary classified this as a deliberate Phase-1 convention, not drift. The forward
VP->BC mapping in VP-INDEX.md is authoritative. BC->VP back-reference back-fill deferred as
a Phase-1-exit polish item; to be surfaced as a structured question at the Phase 1 human
approval gate.

**Files fixed (7):**
`specs/verification-properties/vp-008-decode-packet-no-panic.md`,
`specs/verification-properties/vp-001-flowkey-canonical-ordering.md`,
`specs/verification-properties/VP-INDEX.md`,
`specs/prd-supplements/error-taxonomy.md`,
`specs/domain/capabilities/cap-02-link-type-gating.md`,
`specs/behavioral-contracts/ss-02/BC-2.02.007.md`,
`specs/domain/invariants/inv-01-core-invariants.md`

**Remediation:** All 8 findings (0C/2H/3M/2L/1N) remediated. VP-008 fuzz skeleton arg order
corrected to data-first; IPv6 literal added; stale line citations corrected in vp-001 and
cap-02; E-RAS-005 counter name corrected in error-taxonomy; BC-2.02.007 error-prefix count
corrected; VP-INDEX VP-005 redundant BC list cleaned; INV-2 token order matched to source.
Fixes committed in burst
`spec: fix adversarial-review pass-8 findings (2H/3M/2L) - vp-008 signature, stale citations`
(SHA: 7cf0edd). Pass 9 dispatched next.

---

### Pass 9 (2026-05-20)

**Findings:** 4 (0 CRIT, 1 HIGH, 1 MED, 2 LOW)
**Delta from pass 8:** -4 total (CRIT 0, HIGH -1, MED -2, LOW 0, NITPICK -1) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `behavioral-contracts/ss-04/BC-2.04.054.md` — 5 stale line citations corrected.
  Architecture Module field: `415` → `557-591`. Invariant 2 latch citation: `mod.rs:385-388` →
  `mod.rs:558-561`. Architecture Anchors push site: `mod.rs:415` → `mod.rs:573`. Architecture
  Anchors latch: `mod.rs:385-388` → `mod.rs:558-561`. Source Evidence Path: `mod.rs:415` →
  `mod.rs:573`. Evidence Types Used guard clause: push site 415 → push site 573.
  All 5 citations now match the post-refactor source layout.

- MED (M-1): `specs/prd.md` section 5 (Error Categories) — prefix scheme misaligned with
  `prd-supplements/error-taxonomy.md`. `E-RDR-NNN` replaced with `E-INP-NNN` (input/file errors).
  `E-CLI-NNN` replaced with `E-CFG-NNN` (configuration errors). Two new categories added:
  `E-ANA-NNN` (analyzer protocol-level parse failures) and `E-OUT-NNN` (output file write
  failures). `E-RAS-NNN` description tightened. `E-DEC-NNN` description updated. Section now
  enumerates all 6 error-taxonomy prefixes: E-INP, E-DEC, E-RAS, E-ANA, E-OUT, E-CFG.

- LOW (L-1): `behavioral-contracts/ss-04/BC-2.04.027.md` — DepthExceeded match arm citation
  was `reassembly/mod.rs:386-389`; correct range is `mod.rs:387-389`. Corrected in Architecture
  Module field, Architecture Anchors section, and Source Evidence Path field.

- LOW (L-2): `specs/architecture/ARCH-INDEX.md` — debt section table accounted for O-01,
  O-03 through O-06 and Smells but gave no account of O-02 and O-07. Note added explaining
  both items are tracked in `domain-debt.md` rather than the architecture debt table because
  they fall outside the architecture layer's scope. Complete open-item set (O-01 through O-07)
  now explicitly acknowledged.

**Files fixed (4):**
`specs/behavioral-contracts/ss-04/BC-2.04.054.md`,
`specs/prd.md`,
`specs/behavioral-contracts/ss-04/BC-2.04.027.md`,
`specs/architecture/ARCH-INDEX.md`

**Remediation:** All 4 findings (0C/1H/1M/2L) remediated. Stale post-refactor citations
corrected in BC-2.04.054 (5 citations) and BC-2.04.027 (1 citation range); prd.md section 5
error-category scheme aligned to the canonical 6-prefix taxonomy in error-taxonomy.md;
ARCH-INDEX debt section annotated to account for O-02 and O-07. Fixes committed in burst
`spec: fix adversarial-review pass-9 findings (1H/1M/2L) - stale citations, error-category alignment`
(SHA: b210c05). Pass 10 dispatched next.

---

### Pass 10 (2026-05-20)

**Findings:** 6 (0 CRIT, 3 HIGH, 3 MED, 0 LOW)
**Delta from pass 9:** +2 total (CRIT 0, HIGH +2, MED +2, LOW -2) — spike; architecture docs not diffed against Cargo.toml in prior passes
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `architecture/dependency-graph.md` — dependency table built from memory, not
  from a live diff of Cargo.toml. Four concrete errors: `colored` crate listed but replaced by
  `owo-colors`; `num_cpus` listed but removed; `md5` listed without correct crate name `md-5`
  version 0.11; `etherparse` version listed as 0.14 but is 0.16. Additionally `tls-parser` was
  missing from the table entirely. Table rebuilt from Cargo.toml ground truth.

- HIGH (H-2): `architecture/api-surface.md` — `Reporter` trait signature listed as
  `render(&self, findings: &[Finding]) -> Vec<String>` (Vec return). Correct signature is
  `render(&self, findings: &[Finding], config: &OutputConfig) -> String`. Corrected.

- HIGH (H-3): `architecture/api-surface.md` — `ParsedPacket` struct listed `timestamp: f64`
  and `timestamp_micros: u64` fields that do not exist in source. Actual field is
  `packet_len: usize`. Phantom timestamp fields removed; packet_len added.

- MED (M-1): `domain/domain-spec.md` — Capability Index table row for CAP-03 listed
  owning subsystem as `SS-3` (not zero-padded). Corrected to `SS-02` and all SS ID
  references in the Capability Index zero-padded for consistency (SS-01 through SS-13).

- MED (M-2): `domain/domain-spec.md` — test-count anchor in the `reconciled_against` metadata
  field still showed commit `aa2ece9`; correct current develop HEAD is `0082a0c`. Updated.

- MED (M-3): `architecture/dependency-graph.md` — narrative prose claimed "rayon removed"
  as justification for the rebuild. `rayon` is still declared in Cargo.toml (tracked as
  tech-debt item O-07 — declared but unused). False claim corrected; O-07 status note added.

**Process gap identified (codification follow-up at cycle close):**
P10-PG: Architecture-doc dependency tables were not diffed against Cargo.toml before adversarial
review — they were authored from memory. Recommend a mechanical `validate-deps-against-cargo`
check: parse `[dependencies]` and `[dev-dependencies]` from Cargo.toml and assert each declared
crate appears in the dependency-graph.md table with the correct version. Eliminates the class
of stale-version / phantom-crate / missing-crate findings that drove H-1 and M-3 in pass 10.

**Files fixed (3):**
`specs/architecture/dependency-graph.md`,
`specs/architecture/api-surface.md`,
`specs/domain/domain-spec.md`

**Remediation:** All 6 findings (0C/3H/3M/0L) remediated. Dependency table rebuilt from
Cargo.toml ground truth (owo-colors, md-5 0.11, etherparse 0.16, tls-parser added; num_cpus
removed; rayon-removed false claim corrected). Reporter trait signature corrected to
`render(&self, ...) -> String`. ParsedPacket phantom timestamp fields removed; packet_len
added. CAP-03 SS-3 -> SS-02 and all SS IDs zero-padded in domain-spec Capability Index.
Reconciled-against anchor corrected aa2ece9 -> 0082a0c. Fixes committed in burst
`spec: fix adversarial-review pass-10 findings (3H/3M) - dependency table, api-surface, capability anchors`
(SHA: 824f07d). Pass 11 dispatched next.

---

### Pass 11 (2026-05-20) — CONVERGED (clean pass 1 of 3)

**Findings:** 1 LOW + 4 observations (0 CRIT, 0 HIGH, 0 MED, 1 LOW, 4 cosmetic)
**Delta from pass 10:** -5 total (CRIT 0, HIGH -3, MED -3, LOW +1) — no regression
**Novelty:** LOW
**Convergence counter:** 1/3
**Verdict:** CONVERGED — first clean pass. No CRIT/HIGH/MED findings. Passes 12 and 13
must also return clean to satisfy the 3-clean-pass Phase 1d adversarial convergence gate.

**Findings (non-blocking, polished for clean package):**

- LOW (L-1): `architecture/api-surface.md` — `--mitre` flag BC reference listed as
  `BC-2.12.004`; correct reference is `BC-2.12.001`. Corrected.

- Observation (O-1): `architecture/system-overview.md` — pseudocode used `reporter.report(...)`
  which does not match the actual trait method name `render`. Updated to `reporter.render(...)`.

- Observation (O-2): `behavioral-contracts/ss-07/BC-2.07.037.md` — `NonAsciiUtf8` variant
  written as a unit variant; correct form is a struct variant with a `bytes` field, matching
  the actual enum definition. Corrected to struct-variant notation.

- Observation (O-3): `prd-supplements/interface-definitions.md` — exit-code table omitted
  the `exit 2` row for clap argument-parse errors. Row added.

- Observation (O-4): `architecture/dependency-graph.md` — dev-dependency table was
  incomplete; `tempfile`, `proptest`, and `criterion` were absent. All three added.

**Files fixed (5):**
`specs/architecture/api-surface.md`,
`specs/architecture/system-overview.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.037.md`,
`specs/prd-supplements/interface-definitions.md`,
`specs/architecture/dependency-graph.md`

**Polish committed in burst:**
`spec: pass-11 polish (1L/4 observations) - package CONVERGED, counter 1/3`
(SHA: 4d4cf89). Pass 12 dispatched next (confirmation pass).

---

### Pass 12 (2026-05-20) — NOT CONVERGED (counter RESET 1/3 → 0/3)

**Findings:** 6 (0 CRIT, 1 HIGH, 1 MED, 2 LOW, 2 NITPICK)
**Delta from pass 11:** +5 total (HIGH +1, MED +1, LOW +1, NITPICK +2) — regression; streak broken
**Novelty:** LOW
**Convergence counter:** 0/3 (RESET — pass 12 was not clean; 3 consecutive clean passes required)
**Verdict:** NOT CONVERGED — HIGH + MED findings disqualify this pass as a clean pass. Counter
resets to 0/3. Pass 13 is next; must start a fresh consecutive-clean streak.

**Key finding categories:**

- HIGH (F-1): `behavioral-contracts/ss-11/BC-2.11.007.md` — Postcondition 3 asserted that only
  the high-C1 range (0x80–0x9F) is escaped; this contradicted both the source code and the sibling
  BC-2.11.009 which correctly specifies the full C1 range including NEL (0x85). Postcondition 3
  rewritten to state that the entire C1 range (0x80–0x9F inclusive, including NEL) is escaped.

- MED (F-2): `behavioral-contracts/ss-11/BC-2.11.001.md` — `json.rs` unwrap citation cited line 30;
  correct line post-refactor is 59. Additionally, the BC body contained a claim that the module
  comment in `json.rs` asserts RFC 8259 compliance — no such comment exists in source. Both the
  stale citation and the unsupported claim removed.

- LOW (F-3): `behavioral-contracts/ss-04/BC-2.04.049.md` — EC-002 postcondition described IPv6
  addresses as bracket-less; the rendering in source includes brackets `[addr]`. Corrected.

- LOW (F-4): `behavioral-contracts/ss-04/BC-2.04.049.md` — `flow.rs` citation line 69 was stale;
  correct line is 70. Corrected.

- NITPICK (N-1, N-2): `behavioral-contracts/ss-11/BC-2.11.020.md` — two `csv.rs` citation
  off-by-one errors in the Architecture Anchors section. Both corrected.

**Recurring process gap (6th occurrence — MANDATORY codification follow-up):**
Stale source-line citations (`file.rs:NNN`) recurred in this pass (F-2, F-4) and were also
present in passes 4, 6, 8, 9, and 10. This is the 6th recurrence of the same process gap.
Per the Cycle-Closing Checklist, a recurring process gap with 6+ occurrences requires a
mandatory codification follow-up (a follow-up story or justified deferral) before the cycle
can be declared closed. See STATE.md Deferred Findings for the required action.

**Files fixed (4):**
`specs/behavioral-contracts/ss-11/BC-2.11.007.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.001.md`,
`specs/behavioral-contracts/ss-04/BC-2.04.049.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.020.md`

**Remediation:** All 6 findings (0C/1H/1M/2L/2N) fixed. C1 postcondition corrected in
BC-2.11.007; stale citation + unsupported claim removed from BC-2.11.001; IPv6 bracket-less
rendering and stale citation fixed in BC-2.04.049; csv.rs off-by-one citations fixed in
BC-2.11.020. Fixes committed in burst
`spec: fix adversarial-review pass-12 findings (1H/1M/2L) - C1-escape postcondition, stale citations`
(SHA: c21b13c). Pass 13 dispatched next.

---

### Pass 13 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 5 (0 CRIT, 2 HIGH, 0 MED, 3 LOW — of which 2L + 1N reported as "3 LOW, 1 NITPICK")
**Delta from pass 12:** -1 total (HIGH +1, MED -1, LOW +1, NITPICK -1) — no regression; fresh-context anchor audit
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH findings disqualify; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 2 HIGH findings present. Counter remains 0/3. Pass 14 is next.

**Key finding categories:**

- HIGH (H-1): `domain/entities/ent-05-enums-value-objects.md` — 7 value-object source-line anchors
  were stale (post-refactor drift in `findings.rs`, `output/json.rs`, `output/csv.rs`). All 7
  anchor citations corrected to current line numbers.

- HIGH (H-2): `domain/invariants/inv-01-core-invariants.md` — INV-4 enforcement citation referenced
  `findings.rs:72-80`; correct lines post-refactor are `findings.rs:10-14` (struct definition) and
  `findings.rs:148-156` (validation logic). Citation updated to dual-anchor form.

- LOW (C-1): `architecture/ARCH-INDEX.md` — component-count summary line stated range `C-1..C-20`;
  correct range is `C-1..C-21` (CsvReporter dispatcher was added as C-21 in pass-4 remediation but
  the summary line was not updated). Corrected.

- LOW (LOW): `specs/prd.md` — BC-2.07.004 section-2.7 one-liner description was misaligned with
  the canonical BC H1 heading. Aligned to canonical BC body language.

- NITPICK (1N): Minor wording inconsistency; corrected in the same sweep as H-1.

**Recurring process gap (7th occurrence — P-CITE-PG):**
Stale source-line citations (`file.rs:NNN`) recurred again in this pass (H-1, H-2). This is the
7th recurrence across passes 4, 6, 8, 9, 10, 12, 13. Mandatory codification follow-up (P-CITE-PG)
already recorded in STATE.md Deferred Findings. No new action item added.

**Files fixed (4):**
`specs/domain/entities/ent-05-enums-value-objects.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/architecture/ARCH-INDEX.md`,
`specs/prd.md`

**Remediation:** All 5 findings (0C/2H/0M/3L) remediated. ent-05 7 value-object anchors corrected;
INV-4 enforcement citation updated to dual-anchor form (findings.rs:10-14 + :148-156); ARCH-INDEX
component-count range corrected C-1..C-20 → C-1..C-21; prd.md BC-2.07.004 one-liner aligned to
canonical BC H1. Fixes committed in burst
`spec: fix adversarial-review pass-13 findings (2H/3L) - ent-05 anchors, INV-4 anchor`.
Pass 14 dispatched next.

---

### Pass 14 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 3 (0 CRIT, 1 HIGH, 0 MED, 1 LOW, 1 NITPICK)
**Delta from pass 13:** -2 total (HIGH -1, LOW -2, NITPICK +1) — no regression; continued reduction
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 HIGH finding present. Counter remains 0/3. Pass 15 is next.

**Key finding categories:**

- HIGH (H-1): `domain/domain-spec.md` and `domain/capabilities/cap-12-cli-orchestration.md` —
  summary.rs component ID cited as `C-16` in both files (4 total locations). Correct component ID
  per `architecture/module-decomposition.md` is `C-17`. Fixed in:
  - `domain-spec.md`: CAP-12 note paragraph (1 site) and SS-12 subsystem map row (1 site)
  - `cap-12-cli-orchestration.md`: Sources line in overview (1 site) and section header
    "Summary accumulation (summary.rs / C-16)" (1 site)

- LOW (L-1): `domain/domain-spec.md` — Entity/Enum Index table (section 5) did not list
  `E-39b CsvReporter` in the ent-04 row. CsvReporter has been in the codebase since PR #84;
  the entity index was never updated to reflect it. Entry added. Entity count updated 41→42
  in both the frontmatter summary table and the section-5 header.

- NITPICK (N-1): `behavioral-contracts/ss-12/BC-2.12.005.md` — Architecture Anchors section
  and Source Evidence Path field both cited `src/cli.rs:61-105`; correct range after latest
  refactor is `src/cli.rs:61-106` (off-by-one on end line). Fixed in 2 locations.

**Recurring process gap (8th occurrence — P-CITE-PG):**
Stale source-line citation recurred in N-1 (cli.rs:61-105 → 61-106). This is the 8th occurrence
of the P-CITE-PG gap. Mandatory codification follow-up already recorded in STATE.md. No new
action item added.

**Files fixed (3):**
`specs/domain/domain-spec.md`,
`specs/domain/capabilities/cap-12-cli-orchestration.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.005.md`

**Remediation:** All 3 findings (0C/1H/0M/1L/1N) remediated. summary.rs C-16→C-17 corrected
in 4 locations across 2 files; E-39b CsvReporter added to entity index and entity count
incremented 41→42; cli.rs citation range corrected 61-105→61-106 in 2 locations. Fixes committed
in burst
`spec: fix adversarial-review pass-14 findings (1H/1L/1N) - C-16/C-17 mis-anchor, entity index`
(SHA: 3ec08db). Pass 15 dispatched next.

---

### Inter-Pass Sweep (2026-05-20) — Proactive Anchor-Consistency Sweep

**Type:** Remediation burst (not an adversary pass)
**Trigger:** Recurring component-ID / capability-anchor defect class found in passes 4, 6, 10, 13, 14.
Orchestrator commissioned a root-cause sweep before dispatching pass 15.
**Convergence counter:** 0/3 (unchanged — no adversary pass issued)

**Scope:** Comprehensive C-NN / SS-NN / capability-column anchor audit across the full spec package.
Total occurrences audited: 3,820.

**Mis-anchors found and fixed: 28**

1. **C-ID mis-anchors in ss-12 BC bodies (3 fixes):**
   - `behavioral-contracts/ss-12/BC-2.12.018.md` — Architecture Module field cited `C-16`; correct is `C-17` (summary.rs).
   - `behavioral-contracts/ss-12/BC-2.12.019.md` — same C-16→C-17 correction.
   - `behavioral-contracts/ss-12/BC-2.12.021.md` — same C-16→C-17 correction.
   These three files were missed in the pass-14 remediation which corrected domain-spec.md and
   cap-12-cli-orchestration.md but did not sweep the BC bodies that also anchor to summary.rs.

2. **Capability-column mis-anchors in prd.md traceability matrix (25 fixes):**
   - All 21 BC-2.12.* rows had capability column mapped to `CAP-01` instead of `CAP-12`.
   - All 4 BC-2.13.* rows had capability column mapped to `CAP-01` instead of `CAP-12`.
   Root cause: CAP-12 (CLI Orchestration) was added in pass-2 remediation; the prd.md traceability
   matrix was not swept at that time and defaulted to the CAP-01 placeholder for all new rows.

**Root-cause analysis:**
The recurring defect class (component-ID and capability-anchor drift) is driven by P4-PG2:
no automated cross-file consistency validator exists to assert that a component-ID or
capability-anchor is consistent across BC bodies, domain-spec, capability shards, and the
prd.md traceability matrix. Manual remediation bursts fix the reported site but leave
sibling files un-swept. Mandatory codification follow-up P4-PG2 already recorded in STATE.md.

**Files fixed (4):**
`specs/behavioral-contracts/ss-12/BC-2.12.018.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.019.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.021.md`,
`specs/prd.md`

**Committed in burst:**
`spec: proactive anchor-consistency sweep - fix 3 C-ID + 25 capability-column mis-anchors`
(SHA: 21093ed). Pass 15 dispatched next.

---

### Pass 15 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 4 (0 CRIT, 1 HIGH, 2 MED, 1 NITPICK)
**Delta from pass 14:** +1 total (HIGH 0, MED +2, LOW -1, NITPICK 0) — no regression; LOW findings resolved, 2 MED surfaced in VP-020 and module-decomposition
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 HIGH + 2 MED findings present. Counter remains 0/3. Pass 16 is next.

**Key finding categories:**

- HIGH (F-1): `verification-properties/vp-020-csv-injection-neutralization.md` — the
  `test_csv_safe_values_unchanged` test was written against a non-existent API: it used
  `CsvReporter::new()` (no such constructor; `CsvReporter` is a unit struct) and called
  `reporter.render(...)` with a `Summary` + `findings` + `analyzer_summaries` triple-arg
  form that does not match the actual `Reporter` trait signature. The `render` method returns
  an owned `String`, not a `Vec<String>`. Test rewritten to use `CsvReporter` directly
  (unit struct — no constructor) and the correct `render(&self, ...) -> String` signature.

- MED (F-2): `verification-properties/vp-020-csv-injection-neutralization.md` — Property
  Statement point 3 claimed that `AnalysisSummary` detail values are neutralized. This was
  incorrect: `CsvReporter::render` explicitly ignores the `_analyzer_summaries` parameter
  (underscore-prefixed at `csv.rs:56`). No `AnalysisSummary` data ever reaches a CSV cell.
  Point 3 corrected to reflect the actual neutralization scope (per-`Finding` fields only).

- MED (F-3): `architecture/module-decomposition.md` — L4 reporter table Purity column listed
  `C-19` (JsonReporter) and `C-20` (TerminalReporter) as `Effectful (stdout write)`, and the
  CsvReporter row as `Effectful (stdout/file write)`. This was incorrect: all three reporter
  `render()` implementations are pure `&self -> String` transformations — they return an owned
  `String` and perform no I/O themselves. The I/O (stdout write or file write) is the caller's
  responsibility (`main.rs`). All three Purity cells corrected to `Pure (returns owned String;
  no I/O -- caller in main.rs writes)`.

- NITPICK (N-1): Covered by the F-3 purity-column fix; minor wording cleaned in the same pass.

**Files fixed (2):**
`specs/verification-properties/vp-020-csv-injection-neutralization.md`,
`specs/architecture/module-decomposition.md`

**Remediation:** All 4 findings (0C/1H/2M/1N) remediated. VP-020 test rewritten to the real
`CsvReporter`/`render() -> String` API; Property Statement point 3 corrected to exclude
`AnalysisSummary` from neutralization scope; module-decomposition reporter Purity column
corrected from effectful to pure for all three reporter components. Fixes committed in burst
`spec: fix adversarial-review pass-15 findings (1H/2M/1N) - VP-020 CsvReporter API + reporter purity`
(SHA: 7a66b0b). Pass 16 dispatched next.

---

### Pass 16 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 3 (1 CRIT, 0 HIGH, 1 MED, 1 LOW)
**Delta from pass 15:** -1 total (CRIT +1, HIGH -1, MED -1, LOW +1) — no regression; findings are in a different BC file than pass-15 targets
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — CRIT finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 CRIT finding present. Counter remains 0/3. Pass 17 is next.

**Key finding categories:**

- CRIT (F-1): `behavioral-contracts/ss-07/BC-2.07.037.md` — Postcondition 4 asserted that the
  arm-3 (NonAsciiUtf8) finding has verdict `Anomaly/Likely/High`. This is incorrect. Arm 3 fires
  for non-ASCII UTF-8 SNI hostnames; the correct verdict tuple per the source code and sibling BCs
  (BC-2.07.017, BC-2.07.019) is `Anomaly/Inconclusive/Low`. The `Likely/High` tuple is used by
  arm 1 (valid ASCII clean hostname — no finding) and arm 2 (AsciiWithControl — BC-2.07.014),
  not arm 3. Postcondition 4 corrected to `Anomaly/Inconclusive/Low`.

- MED (F-2): `behavioral-contracts/ss-07/BC-2.07.017.md` and `BC-2.07.019.md` — both files
  contained stale internal correction-notes of the form "BC-INDEX title says..." that were
  remediation breadcrumbs from an earlier pass (pass 6 BC-INDEX resync). These notes are no
  longer accurate or useful and were removed.

- LOW (L-1): Minor wording inconsistency in BC-2.07.037.md Related BCs section; corrected in
  the same sweep as F-1.

**Files fixed (3):**
`specs/behavioral-contracts/ss-07/BC-2.07.037.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.017.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.019.md`

**Remediation:** All 3 findings (1C/1M/1L) remediated. BC-2.07.037 Postcondition 4 verdict
corrected Anomaly/Likely/High → Anomaly/Inconclusive/Low to match source and sibling BCs;
stale correction-notes removed from BC-2.07.017 and BC-2.07.019. Fixes committed in burst
`spec: fix adversarial-review pass-16 findings (1C/1M) - SNI verdict + stale notes`
(SHA: cbef4f1). Pass 17 dispatched next.

---

### Inter-Pass Sweep (2026-05-20) — Comprehensive BC-vs-Source Verification Sweep

**Type:** Remediation burst (not an adversary pass)
**Trigger:** After pass 16, recurring citation/token-drift defect class (P-CITE-PG, 8 occurrences
across passes 4, 6, 8, 9, 10, 12, 13, 14) had driven repeated HIGH and MEDIUM findings that reset
or held back the convergence counter. The orchestrator commissioned a proactive, comprehensive
BC-vs-source verification sweep of all 217 BCs (6 parallel agents, one per subsystem group) before
dispatching pass 17, to flush residual spec-vs-code drift at root rather than discovering it one
defect at a time per adversary pass.
**Convergence counter:** 0/3 (unchanged — no adversary pass issued)

**Scope:** All 217 behavioral contracts across ss-01..ss-13 re-verified against current src/.

**Defects found and fixed: ~58 total**

#### ss-04 (2 defects)
- `BC-2.04.012.md` — stale latch/counter line citations corrected
- `BC-2.04.030.md` — stale latch/counter line citations corrected

#### ss-07 (6 defects)
- `BC-2.07.001.md` — off-by-one match-arm citations corrected
- `BC-2.07.009.md` — GREASE-mechanism mis-description corrected; off-by-one citations
- `BC-2.07.017.md` — off-by-one match-arm citations corrected
- `BC-2.07.019.md` — off-by-one match-arm citations corrected

#### ss-06 / ss-05 (15 defects)
- `BC-2.06.001.md`, `BC-2.06.002.md`, `BC-2.06.003.md` — off-by-one function-end citations
- `BC-2.06.004.md` — wrong MAX_MAP_ENTRIES cap claim corrected (semantic)
- `BC-2.06.005.md` — fabricated backslash traversal pattern removed; wrong evidence-truncation
  claim corrected (semantic)
- `BC-2.05.009.md` — wrong unwrap-vs-iflet claim corrected (semantic); off-by-one citations

#### ss-11 / ss-10 (12 defects)
- `BC-2.11.007.md` — wrong C0-escaping claim re CR/LF corrected (semantic: CR and LF are
  not escaped — only C0 range 0x00-0x1F excluding CR/LF/TAB, plus C1 range 0x80-0x9F)
- `BC-2.11.009.md`, `BC-2.11.011.md`, `BC-2.11.014.md`, `BC-2.11.015.md`, `BC-2.11.019.md`
  — off-by-one citations corrected
- `BC-2.10.008.md` — all emitted-site citations were stale; fully re-anchored (semantic)
- `BC-2.10.001.md`, `BC-2.10.007.md` — off-by-one citations corrected

#### ss-12 / ss-09 / ss-08 (8 defects)
- `BC-2.09.001.md` — wrong source_ip/direction claims for reassembly findings corrected
  (semantic: reassembly findings use client-side IP and forward direction, not bidirectional)
- `BC-2.09.004.md` — off-by-one citations corrected
- `BC-2.08.001.md`, `BC-2.08.002.md` — off-by-one citations corrected
- `BC-2.12.001.md`, `BC-2.12.006.md`, `BC-2.12.007.md`, `BC-2.12.014.md` — off-by-one
  citations corrected

#### ss-01 / ss-02 (15 defects)
- `BC-2.01.002.md`, `BC-2.01.005.md` — fabricated Duration API calls corrected (semantic:
  no `Duration::from_secs_f64` usage in the relevant source path; correct API cited)
- `BC-2.01.008.md` — off-by-one citations corrected
- `BC-2.02.005.md` — wrong-function citation corrected (cited wrong function for behavior)
- `BC-2.02.003.md`, `BC-2.02.004.md`, `BC-2.02.006.md`, `BC-2.02.014.md` — off-by-one
  citations corrected

**Summary of defect classes:**
- Off-by-one / stale line citations: ~52 defects (citation drift from src/ refactors)
- Semantic spec-vs-code defects: ~6 defects (wrong API, wrong cap claims, wrong field/direction
  claims, wrong escape-behavior descriptions)

**Root cause addressed:** P-CITE-PG — no automated validator resolves `file.rs:NNN` anchors
in spec artifacts. This sweep manually closed the accumulated citation debt across all 217 BCs.
Mandatory codification follow-up (P-CITE-PG) remains open for cycle close.

**Files fixed (37 BC body files):**
ss-01: BC-2.01.002, BC-2.01.005, BC-2.01.008
ss-02: BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006, BC-2.02.014
ss-04: BC-2.04.012, BC-2.04.030
ss-05: BC-2.05.009
ss-06: BC-2.06.001, BC-2.06.002, BC-2.06.003, BC-2.06.004, BC-2.06.005
ss-07: BC-2.07.001, BC-2.07.009, BC-2.07.017, BC-2.07.019
ss-08: BC-2.08.001, BC-2.08.002
ss-09: BC-2.09.001, BC-2.09.004
ss-10: BC-2.10.001, BC-2.10.007, BC-2.10.008
ss-11: BC-2.11.007, BC-2.11.009, BC-2.11.011, BC-2.11.014, BC-2.11.015, BC-2.11.019
ss-12: BC-2.12.001, BC-2.12.006, BC-2.12.007, BC-2.12.014

BC-INDEX.md NOT modified — index was current; body files only.

**Committed in burst:**
`spec: comprehensive BC-vs-source verification sweep - fix ~58 residual drift defects across 217 BCs`
(SHA: d038ace). Pass 17 dispatched next.

---
