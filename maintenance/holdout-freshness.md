---
run: maint-2026-07-09
producer: holdout-evaluator (sweep4 subagent)
sweep: 4
binary: cargo run --release (develop @ 716054a)
hs-index-version: "2.13"
scope: full holdout inventory (132 files); PR #389 JSON-envelope + enum-casing impact re-check
prior-sweep: maint-2026-07-08 (21/21 PASS, 0 stale)
---

# Holdout Freshness Check — maint-2026-07-09

## Summary Counts

| Metric | Count |
|--------|-------|
| Total holdout scenarios on disk | 132 (HS-001..HS-109 greenfield + 23 feature: DNP3, ARP, collapse, ENIP, protocol-coverage) |
| `lifecycle_status: active` | 132 |
| `lifecycle_status: stale` | 0 |
| `lifecycle_status: retired` | 0 |
| Scenarios needing re-evaluation from PR #389 | 0 new — all 13 known JSON-touching scenarios already repaired in wave-72-repair-2026-07-09 (this cycle, prior burst) |
| Scenarios modified this sweep | 0 (frontmatter left untouched per task rule) |

Prior sweep (maint-2026-07-08) recorded 21/21 PASS on the sampled subset. This sweep
confirms all 132 scenarios remain `active` and the 13 wave-72 JSON repairs are landed
and consistent with the shipped binary at develop `716054a`.

## PR #389 (Wave-72) — Reporter JSON Envelope & Enum Casing

**PR #389** merged the BC-2.11.036 (v1.2) + BC-2.11.037 changes: verdict/confidence
lowercased, ThreatCategory snake_cased, and the JSON envelope expanded from 5 to 6
top-level keys (adds `schema_version`).

**Repaired scenarios (already landed in this same maintenance cycle, HS-INDEX v2.13):**

| HS ID | Repair Type | Verified Against Binary |
|-------|-------------|-------------------------|
| HS-021, HS-032, HS-034, HS-059, HS-065, HS-074 | PascalCase → lowercase verdict/confidence + snake_case category (BC-2.11.036 v1.2) | Source `src/findings.rs:31,66,98` confirms `rename_all = "lowercase"` (Verdict, Confidence) and `rename_all = "snake_case"` (ThreatCategory) |
| HS-064, HS-075 | 5-key → 6-key envelope (adds `schema_version`, BC-2.11.037) | `cargo run -- analyze --output-format json tests/fixtures/tcp-ecn-sample.pcap \| jq 'keys'` returns exactly `["analyzers","findings","mitre_attack_version","mitre_domain","schema_version","summary"]` — matches HS-075 v1.2 assertion byte-for-byte |
| HS-024, HS-033, HS-035, HS-050, HS-054 | DF-SIBLING-SWEEP-001 category/confidence literals lowercased | No PascalCase enum literals remain in JSON assertion contexts (regex sweep of `"Likely|Unlikely|Inconclusive|Possible|High|Medium|Low|Reconnaissance|LateralMovement|C2|Exfiltration|CredentialAccess|Persistence|Execution|Anomaly|Suspicious|Impact"` in double-quoted JSON contexts: 0 hits across all 132 scenarios) |

All 13 repaired scenarios were version-bumped and carry `modified:` changelog entries
per FIX-C precedent. `lifecycle_status` on each is `active`. No further JSON-envelope
or enum-casing drift detected in the remaining 119 scenarios.

## Deferred Item — HS-INDEX-ENIP-WAVE-DRIFT-001

Wave/story column drift for waves 58-61 (STORY-130..138) in the ENIP feature holdout
section: **CONFIRMED and DEFERRED Route C** by human on 2026-07-08. State unchanged
this sweep — no fix attempted per task instruction. HS-INDEX line 777 currently reads
"Stories: STORY-131..STORY-141 (waves 63-68)" for the ENIP feature holdout block;
reconciling the historical drift entry against the current story range is deferred to
the human-scoped follow-up already logged 2026-07-08.

## Advisory (Not Marked Stale)

**HS-082:76** (`terminal-color-disabled-no-ansi-codes.md`): the "Optionally verify"
step suggests looking for the substrings `"Likely"` and `"Inconclusive"` in
`--no-color` terminal output. `Verdict::fmt` in `src/findings.rs:49-58` writes
`"LIKELY"` / `"INCONCLUSIVE"` (all-caps) — mixed-case substrings would not match
literally. This pre-dates PR #389 (terminal Display was uppercase before wave 72),
and the assertion is worded as "optional" / "recognizable text." **Not marked stale**
per the task's "genuinely stale only" rule; flagged here as a candidate for a future
worded-precision cleanup pass.

## Verification Commands Run

```
grep -a -h "^lifecycle_status:" .factory/holdout-scenarios/HS-*.md | sort | uniq -c
    → 132 lifecycle_status: active

cargo run --quiet --release -- analyze --output-format json \
    tests/fixtures/tcp-ecn-sample.pcap | jq 'keys'
    → ["analyzers","findings","mitre_attack_version","mitre_domain","schema_version","summary"]

grep -a -n -E '"(Likely|Unlikely|Inconclusive|Possible|High|Medium|Low|Reconnaissance|LateralMovement|Exfiltration|CredentialAccess|Anomaly|Suspicious|Impact)"' \
    .factory/holdout-scenarios/HS-*.md | grep -v INDEX
    → 4 hits, all in terminal-display/BC-shorthand contexts (HS-006, HS-008, HS-025,
      HS-082); none in JSON assertion contexts.
```

## Result

**FRESHNESS: CLEAN.** All 132 scenarios remain `active`; zero new staleness introduced
by PR #389 (13 JSON-shape scenarios already repaired in this same maintenance cycle,
landings verified against binary at develop 716054a). No lifecycle_status frontmatter
edits made this sweep.
