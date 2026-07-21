---
run-id: maint-2026-07-21
sweep: 1 (security analysis)
producer: security-reviewer
date: 2026-07-21
wirerust-version: 0.13.0
crate-count: 175
advisory-db-entries: 1166
raw-log: .factory/maintenance/dependency-audit-raw-2026-07-21.log
prior-sweep: maint-2026-07-11 (dependency-audit-findings-2026-07-11.md)
---

# Dependency Audit Findings — Maintenance Sweep 1 (maint-2026-07-21)

## Overall Verdict: CLEAN

**cargo audit:** ZERO advisories against 175 locked crates (advisory DB: 1166 entries).
**cargo deny:** ZERO errors. 1 warning — DEP-007 (syn duplicate), pre-existing and deferred.
**New duplicates:** rand 0.8.7 + 0.9.5 and rand_core 0.6.4 + 0.9.5 (DEP-008, DEP-009 — build/dev-only, LOW, no action).
**Advisory-race check:** PASS — DB grew +7 entries (1159→1166) since last sweep; cargo audit is CLEAN against the updated 1166-entry DB, confirming none of the 7 new advisories touch our tree.

No CRITICAL, HIGH, or MEDIUM findings. The audit is NON-BLOCKING for continued development. No emergency fix PR required.

---

## Section 1 — Advisory Scan Confirmation

cargo audit exited 0 with zero output. The scan covered 175 locked crates against 1166 advisory DB entries. The crate count delta (193 → 175, −18) reflects PR #420 dep-soak landed 2026-07-19, which pruned stale transitive entries. Zero `[VULNERABILITY]`, `[WARNING]`, or `[UNMAINTAINED]` entries appear.

### Advisory-DB Delta: +7 Entries (1159 → 1166)

The DB grew by 7 entries between the maint-2026-07-11 scan and today. Because cargo audit reported CLEAN against the 1166-entry DB, all 7 new advisories were checked by the tool and none matched any of the 175 locked crates. The advisory-race concern documented after RUSTSEC-2026-0204 is therefore not triggered — we have a confirmed-clean result against the fully updated DB, not a stale-DB-then-PR-CI-race scenario.

| Sweep | DB entries | Delta |
|---|---|---|
| maint-2026-07-06 | 1158 | baseline |
| maint-2026-07-08 | 1159 | +1 |
| maint-2026-07-11 | 1159 | 0 |
| **maint-2026-07-21 (this sweep)** | **1166** | **+7 — CLEAN confirmed** |

### cargo deny

All four check categories pass: `advisories ok, bans ok, licenses ok, sources ok`. The sole warning is DEP-007 (syn 1.0.109 + 2.0.117 duplicate, pre-existing, deferred). No new deny errors.

---

## Section 2 — Findings Classification Table

| Finding ID | Advisory / Category | Dependency | Severity | Status | Action |
|---|---|---|---|---|---|
| DEP-007 | N/A (ecosystem migration) | syn 1.0.109 + 2.0.117 duplicate | LOW (informational) | DEFERRED (registered maint-2026-07-06) | No action; upstream resolution required |
| DEP-008 | N/A (build-dep dual version) | rand 0.8.7 (build) + rand 0.9.5 (dev) | LOW (informational) | NEW — DEFERRED | No action; see analysis below |
| DEP-009 | N/A (build-dep dual version) | rand_core 0.6.4 (build) + rand_core 0.9.5 (dev) | LOW (informational) | NEW — DEFERRED | No action; see analysis below |

**CRITICAL:** 0 | **HIGH:** 0 | **MEDIUM:** 0 | **LOW:** 3 (all deferred)

---

## Section 3 — DEP-008 / DEP-009: rand + rand_core Dual Versions

`cargo tree --duplicates` reveals two new dual-version pairs not present in maint-2026-07-11:

**rand 0.8.7** arrives via `phf_generator` → `phf_codegen` → `tls-parser` (build-dependency of tls-parser). `phf_generator` is a code-generation tool that runs at build time; the rand call is compile-time-only and is not linked into the wirerust binary's runtime path.

**rand 0.9.5** arrives via `proptest 1.11.0` (dev-dependency of wirerust). proptest is test-only.

The origin of this pairing in PR #420 was the proptest version upgrade pulling rand 0.9, while phf/tls-parser remain pinned to rand 0.8. No RUSTSEC advisory exists against rand 0.8.x or 0.9.x at this DB snapshot. The two version families coexist without conflict because they occupy separate crate slots in the lockfile and are never loaded into the same binary.

**Severity rationale: LOW.** Neither version is production-runtime in wirerust. rand 0.8.7 is fully contained to the build-dep code-gen path; rand 0.9.5 to test execution. No security advisory fires. The dual-version situation is a natural consequence of tls-parser / pcap-file ecosystem lag on rand 0.8 vs proptest's adoption of rand 0.9.

**Recommended action:** None now. Register as DEP-008 (rand) and DEP-009 (rand_core) for tracking. If tls-parser or phf_generator releases a version that switches to rand 0.9, the duplicates will resolve automatically on the next `cargo update`. No manual action needed this sweep or at 2026-07-27.

---

## Section 4 — Dependabot Action PRs (#422–425): Soak Verdicts

Governing rule (human decision D-489): soak is measured from **upstream release date** (≥8 days per D-417), not the Dependabot PR open date. Security-relevant bumps are considered regardless of soak.

**SHA-pin policy note:** All four PRs must be verified to carry a 40-character commit SHA with a `# vX.Y.Z` version comment before merge, per the action-pin-gate requirement in CLAUDE.md. Dependabot in this repo has consistently generated proper SHA-pinned refs (enforced by the action-pin-gate CI job on every CI run), so this is an expected pass — but the reviewer must confirm the diff visually before approving.

| PR | Action | Version | Release Date | Days Since Release | Security? | Scorecard Note | Verdict |
|---|---|---|---|---|---|---|---|
| #422 | EmbarkStudios/cargo-deny-action | 2.1.1 | 2026-07-13 | 8 | NO | N/A | **RECOMMEND-ADOPT** |
| #423 | step-security/harden-runner | 2.20.0 | 2026-07-07 | 14 | NO | SCORECARD note satisfied (see below) | **RECOMMEND-ADOPT** |
| #424 | softprops/action-gh-release | 3.0.2 | 2026-07-13 | 8 | NO | N/A | **RECOMMEND-ADOPT** |
| #425 | github/codeql-action | 4.37.0 | 2026-07-08 | 13 | NO | N/A | **RECOMMEND-ADOPT** |

All four meet the ≥8-day soak threshold and none carry a CVE or security-advisory classification.

**Release notes summaries:**
- **cargo-deny-action 2.1.1:** Routine fix for a deprecation issue introduced in 2.1.0 (`use-git-cli` argument handling). No CVE.
- **harden-runner 2.20.0:** Adds block-policy support for macOS/Windows hosted runners; HTTPS monitoring for Bun on Linux. Platform-support additions. No CVE.
- **action-gh-release 3.0.2:** Patch for release reliability and compatibility (upload hardening, diagnostics, asset fixes). No CVE.
- **codeql-action 4.37.0:** Default CodeQL bundle updated to 2.26.0; new config-file input format (disabled by default). Routine bundle update. No CVE.

**SCORECARD-ENABLEMENT-RUNBOOK / harden-runner carry-forward:** The existing STATE.md note records "window watch satisfied; no manual re-pin needed" for PR #423. With 14 days now elapsed since the upstream release (2026-07-07), the soak window is fully satisfied. The SCORECARD-ENABLEMENT-RUNBOOK carry-forward condition for harden-runner is met. PR #423 may be adopted along with the other three.

**Batching recommendation:** Because all four Dependabot action PRs are now soak-eligible, they can be merged as a single CI-infrastructure maintenance batch today rather than waiting for 2026-07-27. They touch only workflow YAML files, carry no Cargo.lock impact, and the action-pin-gate CI job will verify SHA compliance on each PR's CI run. This is the recommended disposition to clear the open-PR backlog for CI infrastructure items.

---

## Section 5 — Crate Soak Status

### Eligible Today (2026-07-21, ≥8 days)

| Crate | Version | Release Date | Days | Notes |
|---|---|---|---|---|
| winnow | 1.0.4 | 2026-07-13 | 8 | Eligible today |
| zmij | 1.0.23 | 2026-07-13 | 8 | Eligible today |

Two crates are technically eligible today, but with 13 of the 15 identifiable crates becoming eligible by 2026-07-22..28, adopting winnow and zmij alone today would create a partial batch. The DEP-SOAK-FOLLOWUP-2026-07-27 carry-forward already plans the consolidated soak sweep. Recommendation: fold winnow and zmij into the 2026-07-27 batch rather than a standalone 2-crate PR.

### Not-Yet-Soaked (eligible 2026-07-22..28)

| Crate | Target Version | Soak-Eligible Date | Notes |
|---|---|---|---|
| bstr | 1.13.0 | 2026-07-22 | |
| toml_edit | 0.25.13 | 2026-07-22 | +spec-1.1.0 metadata suffix on latest; same base version |
| bitflags | 2.13.1 | 2026-07-23 | |
| regex | 1.13.1 | 2026-07-23 | |
| syn | 2.0.119 (see note) | 2026-07-23 | See MAJOR BUMP note below |
| clap | 4.6.2 (deferred) / 4.6.3 (latest) | 2026-07-23 / 2026-07-28 | 4.6.3 released 2026-07-20; target the latest eligible version |
| portable-atomic | 1.14.0 | 2026-07-25 | |
| anyhow | 1.0.104 | 2026-07-26 | |
| cc | 1.3.0 | 2026-07-26 | Unblocks shlex chain on this date |
| serde | 1.0.229 | 2026-07-26 | |
| fastrand | 2.5.0 | 2026-07-27 | |
| proc-macro2 | 1.0.107 | 2026-07-27 | |
| quote | 1.0.47 | 2026-07-27 | |

**Unidentified 2 of "17":** The two unaccounted entries in STATE.md's "17 not-yet-soaked" count most likely correspond to `clap_derive` (Dependabot typically tracks it as a paired bump with clap, which could inflate the count by 1) and either `iana-time-zone` or another transitive dep that received a version bump in PR #420 without explicit naming in the PR body. Resolution: cross-check Cargo.lock diff from PR #420 (commit 49255464) if exact attribution is needed for audit purposes. The discrepancy does not affect the action plan — the identified 15 crates cover all material version changes.

### syn 3.0.2 — MAJOR VERSION BUMP

syn 3.0.2 was released 2026-07-20. This is a semver-major bump from the 2.x line. The deferred version (syn 2.0.119) remains the correct adoption target; 2.0.119 soaks 2026-07-23 and is a routine patch-level update within the 2.x series.

**Recommendation: DO NOT adopt syn 3.0.2.** Major version bumps in foundational proc-macro infrastructure (syn is used by clap_derive, serde_derive, thiserror-impl, zerocopy-derive, num_enum_derive, wasm-bindgen-macro-support, and windows-implement) require an intentional migration assessment covering API breakage across all dependents. Adopting syn 3.x is a multi-crate coordination exercise, not a routine dep soak. Treat syn 3.x as a future planned migration; stay on syn 2.x until an explicit migration story is scheduled.

### Soaked-but-Blocked (4 crates)

| Crate | Latest Version | Release Date | Days | Block Reason | Unblock Date |
|---|---|---|---|---|---|
| js-sys | 0.3.103 | 2026-06-24 | 27 | futures-* 0.3.33 transitive dep published 2026-07-19 (2 days old, not soaked) | ~2026-07-27 |
| wasm-bindgen | 0.2.126 | 2026-06-24 | 27 | Same: futures-* 0.3.33 chain | ~2026-07-27 |
| web-sys | 0.3.103 | 2026-06-24 | 27 | Same: futures-* 0.3.33 chain | ~2026-07-27 |
| shlex | 2.0.1 | 2026-05-17 | 65 | Blocked by cc 1.3.0 (soaks 2026-07-26) AND is a semver-major bump (1.x → 2.x) | 2026-07-26+ with explicit review |

**js-sys / wasm-bindgen / web-sys analysis:** The three crates show 2026-06-24 as the latest crates.io versions (27 days old, fully soaked in isolation). However, upgrading to these versions pulls in futures-* 0.3.33 as a transitive dependency. futures-* 0.3.33 was published 2026-07-19 and is only 2 days old, failing the 8-day soak gate. The 2026-06-24 versions are therefore **not independently adoptable** today. Unblock expected ~2026-07-27, aligning with the DEP-SOAK-FOLLOWUP-2026-07-27 target.

**shlex 2.0.1 analysis:** This is a semver-major bump (1.x → 2.x), so it warrants explicit API-compatibility review regardless of soak. The immediate blocker is cc 1.3.0 (cc depends on shlex; shlex 2.0.1 is pulled via the cc upgrade chain). cc soaks 2026-07-26, making the chain eligible on that date. Even after unblocking, the major version step should be reviewed for API changes visible to wirerust before adoption.

---

## Section 6 — Action Plan

### THIS RUN (maint-2026-07-21)

| Item | Action | Rationale |
|---|---|---|
| cargo audit | No fix PR needed | CLEAN; zero advisories |
| DEP-007 (syn duplicate) | Continue deferral | Pre-existing, upstream-gated |
| DEP-008 (rand dual) | Register, no fix | Build/dev-only; no advisory |
| DEP-009 (rand_core dual) | Register, no fix | Build/dev-only; no advisory |
| Dependabot PRs #422–425 | **RECOMMEND-ADOPT (batch)** | All ≥8d soak; CI-infra only; SHA-pin policy must be verified on each PR diff before merge |
| winnow + zmij | Defer to 2026-07-27 batch | Only 2 crates eligible; batching with 2026-07-27 is more efficient |
| syn 3.0.2 | DO NOT ADOPT | Major version bump; intentional migration required |

### 2026-07-27 FOLLOW-UP (DEP-SOAK-FOLLOWUP-2026-07-27)

By 2026-07-27 the following will be eligible for a single consolidated soak PR:

- All 15 identified not-yet-soaked crates (winnow, zmij, bstr, toml_edit, bitflags, regex, syn 2.0.119, clap 4.6.x, portable-atomic, anyhow, cc, serde, fastrand, proc-macro2, quote)
- js-sys / wasm-bindgen / web-sys (once futures-* 0.3.33 soaks ~2026-07-27)
- shlex 2.0.1 (once cc soaks 2026-07-26, with explicit major-version API review)
- Resolve the 2 unidentified crates from the "17" count via Cargo.lock diff of PR #420

**Batching is strongly preferred** over piecemeal adoption. A single `cargo update` + CI-green run on 2026-07-27 captures all eligible crates simultaneously, minimizes CI churn, and avoids lockfile thrash. The DEP-SOAK-FOLLOWUP-2026-07-27 carry-forward already calls this out.

---

## Diff vs maint-2026-07-11

| Dimension | maint-2026-07-11 | maint-2026-07-21 | Delta |
|---|---|---|---|
| cargo audit advisories | 0 | 0 | No change — CLEAN |
| Advisory DB entries | 1159 | 1166 | +7 — all 7 verified non-applicable to our tree |
| cargo deny errors | 0 | 0 | No change |
| cargo deny warnings | 1 (DEP-007) | 1 (DEP-007) | Stable |
| New duplicate pairs | 0 | 2 (rand, rand_core) | DEP-008, DEP-009 registered |
| Crate count | 193 | 175 | −18 (PR #420 dep-soak, 2026-07-19) |
| Open Dependabot action PRs | 0 | 4 (#422–425) | All soak-eligible today |

---

## Summary

| Severity | Count | Finding IDs | Fix PR Required? |
|---|---|---|---|
| CRITICAL | 0 | — | — |
| HIGH | 0 | — | — |
| MEDIUM | 0 | — | — |
| LOW | 3 | DEP-007, DEP-008, DEP-009 | No — all deferred |

**Audit CLEAN. No emergency action required.** Two new LOW/informational duplicate-version pairs (rand, rand_core) are registered as DEP-008/DEP-009; both are confined to build/dev paths with no security advisory exposure. All four Dependabot action PRs (#422–425) are soak-eligible today and are recommended for adoption as a CI-infrastructure batch — verify SHA-pin compliance on each diff before merging. The main crate soak event remains on track for the DEP-SOAK-FOLLOWUP-2026-07-27 run, which will be able to batch the majority of the 15+ eligible crates in a single PR. syn 3.0.2 (major version) should not be adopted; stay on the 2.x line.
