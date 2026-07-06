# Maintenance Sweep 4 — Holdout Scenario Freshness (run_id maint-2026-07-06)

Date: 2026-07-06
Prior baseline: maint-2026-06-22 (wirerust v0.9.3)
Releases in window since last freshness check (maint-2026-07-01, v0.11.1):
v0.11.2 (2026-07-05), v0.11.3 (2026-07-06), v0.11.4 (2026-07-06)

Scope: `.factory/holdout-scenarios/HS-001..HS-132` (concrete files). Feature-holdout
seeds (DNP3 wave 35–39, ARP wave 40–44, finding-collapse wave 47) are BC seed rows
only, not concrete HS files, and are outside this sweep's scope.

## Totals

| Metric | Count |
|--------|------:|
| Concrete HS files on disk (HS-001..HS-132) | 132 |
| Files with `lifecycle_status` frontmatter field | 131 (HS-018 missing field — pre-existing) |
| **active** (post-sweep) | 127 |
| **stale** (post-sweep) | 4 (HS-061, HS-064, HS-066, HS-075) |
| **retired** | 0 |
| Status transitions this sweep | 4 (all `active` → `stale`) |
| HS-INDEX.md all-namespace total (seeds included) | 205 (unchanged) |

## Status Changes (active → stale)

| HS ID | BCs | Reason | Introduced by |
|-------|-----|--------|---------------|
| HS-061 | BC-2.06.023 | Asserts "Exactly 9 keys" in HTTP analyzer `detail` BTreeMap. Product now emits 10 keys — `dropped_map_entries` added as 10th key. | v0.11.4 (BC-2.06.023 v1.6, 2026-07-06) — silent-limit audit / observability counters (PR #365) |
| HS-064 | BC-2.11.001..005 | Asserts "Exactly 3 top-level JSON keys" (`summary`/`findings`/`analyzers`). Product emits 5 (adds `mitre_attack_version`, `mitre_domain`). | PR #209 ATT&CK-for-ICS v19.1 envelope (pre-dates window; flagged FAIL-STALE in maint-2026-06-22, still unfixed at v0.11.4) |
| HS-066 | BC-2.07.031 | Asserts "exactly 7 keys" in TLS analyzer `detail` BTreeMap. Product now emits 10 keys — `handshake_reassembly_overflows` (BC-2.07.039), `buffer_saturation_drops` (BC-2.07.043), and `dropped_map_entries` (BC-2.07.031 v1.5). | `dropped_map_entries` added in v0.11.4 (BC-2.07.031 v1.5, 2026-07-06). Prior overflows/buffer-saturation counters landed in v0.11.1 window (STORY-144/146). |
| HS-075 | BC-2.11.001..002 | Same "Exactly 3 top-level keys" assertion as HS-064. | Same as HS-064 |

Remediation (out of scope for this sweep; belongs to product-owner):
- HS-061 / HS-066: relax "exactly N keys" to "N core stats keys + observability counters
  {`dropped_map_entries`, `handshake_reassembly_overflows`, `buffer_saturation_drops`}"
  and enumerate the current authoritative key set from BC-2.06.023 v1.6 / BC-2.07.031 v1.5.
- HS-064 / HS-075: relax "Exactly 3 top-level keys" to "3 core + 2 MITRE envelope
  keys (`mitre_attack_version`, `mitre_domain`)" per prior sweep's recommendation.

## Not Stale (deliberately kept `active`)

Behavior-change signals inspected against the release window (v0.11.2/3/4):

- **v0.11.3 flow-purge fix (#342)** — `fix(dispatcher): purge DNP3/ENIP per-flow state on
  flow close`. Restores the correct invariant (no cross-flow state carry). No HS file
  asserted the pre-fix (buggy) behavior. HS-116 (`enip-forwardopen-close-empty-mitre`)
  concerns the CIP ForwardClose *service* detection, not TCP flow close, and is
  unrelated. **No stale.**
- **v0.11.4 observability counters (#365 / cc2a87c)** — surfaces silently-dropped/evicted
  state via new counters. Impact is confined to summarize() detail-map shape (see stale
  set above). ARP summarize (BC-2.16.010 v1.9, 11→13 keys) also grew; no concrete ARP
  holdout files exist (still a coverage gap noted in prior sweeps), so no stale marker
  is applicable.
- **`last_evaluated: null` on many scenarios** — this reflects the baseline holdout set
  which has not yet been executed by holdout-evaluator in a numbered evaluation. It is
  not a per-release staleness signal on its own; retention until evaluated is the
  documented lifecycle.

## Sanity

- Public CLI surface (`cargo run -- --help`) inspected — no removed/renamed flags in the
  window that would invalidate holdout invocation forms. The `protocols` subcommand
  (v0.11.0) and `--coverage-gaps` flag (v0.11.2) are additive and already covered by
  HS-123..HS-132.
- Prior-sweep minor findings (HS-108 "stdout empty under --json" wording; HS-090/HS-098
  invocation form) unchanged from maint-2026-06-22; not escalated to stale — remain as
  product-owner wording tickets.

## Index Update

`HS-INDEX.md` bumped v2.10 → v2.11 with a changelog line summarizing the four stale
transitions and an "Anomalies" section entry documenting each. All-namespace total
(205) and per-category/per-epic counts unchanged — stale scenarios still exist and
count against the same buckets.
