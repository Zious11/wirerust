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

## Trajectory Shorthand

`17→13→7→19→...`

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
