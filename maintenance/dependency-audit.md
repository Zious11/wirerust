# Dependency Audit — maint-2026-07-09

- **Run ID:** maint-2026-07-09
- **Date:** 2026-07-09
- **Project:** wirerust
- **Branch:** develop @ 716054a (v0.11.5)
- **Raw scan source:** `.factory/maintenance/dependency-audit-raw.log`
- **Crates scanned:** 193 (Cargo.lock)
- **Advisory DB loaded:** 1159 advisories (cargo-audit)
- **cargo-deny:** exit 0, all 4 checks passed
- **cargo-outdated:** NOT INSTALLED (skipped)
- **Prior sweep reference:** maint-2026-07-08 (`dependency-audit-findings.md`)

---

## Overall Verdict: CLEAN

**cargo audit:** ZERO advisories against 193 locked crates.
**cargo deny:** ZERO errors. 9 warnings — all known, all previously registered (DEP-006 × 8, DEP-007 × 1).
**cargo outdated:** SKIPPED — tool not installed.

No new findings introduced since maint-2026-07-08. Two pre-existing deferred LOW items (DEP-006, DEP-007) remain valid deferrals with no change in status or upstream resolution. The only delta vs the prior sweep is indicatif 0.18.5 → 0.18.6 (Dependabot #386, already merged), which does not affect the security or compliance posture.

**Finding counts by severity: CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 2**

---

## Findings Table

| Finding ID | Advisory / CWE | Dependency | Severity | Status | Action |
|---|---|---|---|---|---|
| DEP-006 | N/A (hygiene) | deny.toml — 8 unused `license-not-encountered` entries | LOW (informational) | DEFERRED (registered maint-2026-07-06, re-validated maint-2026-07-09) | Remain deferred — see disposition below |
| DEP-007 | N/A (ecosystem migration) | syn 1.0.109 + syn 2.0.117 duplicate | LOW (informational) | DEFERRED (registered maint-2026-07-06, re-validated maint-2026-07-09) | No action; upstream resolution required |

---

## DEP-006 — deny.toml license allowlist, 8 unused entries (PERSISTS, DEFERRED)

**Severity:** LOW (informational, no security implication)
**CWE:** N/A
**RUSTSEC:** N/A

Eight entries in `deny.toml` `[licenses] allow` continue to match no crate in the current dependency graph. The set is identical to prior sweeps: `0BSD`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `CC0-1.0`, `ISC`, `MPL-2.0`, `Unicode-DFS-2016`, `Zlib`. No new entries appeared; no previously-flagged entries were resolved.

**Upstream status:** No change. These are permissive-license allowances that the current 193-crate graph simply does not exercise. All eight are permissive and MIT-compatible; no disallowed or copyleft license is present in the graph.

**Deferral validity:** Still valid. The risk of deferral is that a future transitive dependency using one of these permissive licenses would be silently accepted, which is not a compliance risk given the license types involved. The appropriate resolution is to fold a `cargo deny list` audit and allowlist trim into the next `chore:` or `docs:` maintenance PR where the marginal cost is near zero.

---

## DEP-007 — syn v1 / v2 duplicate (PERSISTS, DEFERRED)

**Severity:** LOW (informational, expected ecosystem pattern)
**CWE:** N/A
**RUSTSEC:** N/A (no advisory against syn 1.0.109 or syn 2.0.117 in the 1159-advisory DB)

syn 1.0.109 and syn 2.0.117 both remain in Cargo.lock. The dependency chains are unchanged from maint-2026-07-08:

- **syn 1.0.109** (legacy) — pulled by `derive-into-owned` (via `pcap-file`) and `nom-derive-impl` (via `nom-derive` → `tls-parser`). Both are direct wirerust dependencies that have not yet migrated their proc-macro chains to syn 2.
- **syn 2.0.117** (current) — pulled by `clap_derive`, `serde_derive`, `thiserror-impl`, `zerocopy-derive`, `wasm-bindgen-macro-support`, `windows-implement`, `windows-interface`. The dominant version in the dependency graph.

**Build-time assessment:** Both syn versions are proc-macro compile-time dependencies only. No syn code executes at runtime. The duplication adds incremental build time (two separate compilations of syn) and nothing else.

**Runtime risk:** NONE. The compiled wirerust binary contains only the output of the proc-macros, not the syn crate itself.

**Upstream status:** No change. Resolution is entirely upstream: tls-parser and pcap-file must publish releases that migrate to syn 2 before the v1 copy can be dropped from the lock file. There is no wirerust-side fix available without replacing those dependencies.

**Deferral validity:** Still valid. No action warranted this sweep.

---

## Unmaintained / Unsound / Compromised Dependency Assessment

cargo audit scanned all 193 locked crates against the 1159-advisory RustSec database, which includes `unmaintained`, `yanked`, `unsound`, and CVE-class entries. Zero advisories of any category were returned. The `sources: ok` result from cargo deny confirms all sources are crates.io — no git-source or local-path dependencies.

| Crate | Version | Type | Status |
|---|---|---|---|
| `pcap-file` | 2.0.0 | Direct dep | Maintained — no advisory |
| `tls-parser` | 0.12.2 | Direct dep | Maintained — no advisory |
| `nom-derive` | 0.10.1 | Transitive | Maintained — no advisory |
| `syn` | 1.0.109 + 2.0.117 | Transitive (proc-macro) | Maintained (dtolnay); no advisory against either version |
| `indicatif` | 0.18.6 | Direct dep | Maintained; just bumped from 0.18.5 by Dependabot #386 — no advisory |
| `zerocopy` | 0.8.52 | Transitive | Maintained; actively patched; current scan clean |
| `anyhow` | 1.0.103 | Direct dep | Maintained; 0 advisories |

No compromised, yanked, or unsound dependencies detected.

---

## Delta vs maint-2026-07-08

| Dimension | maint-2026-07-08 | maint-2026-07-09 | Delta |
|---|---|---|---|
| cargo audit advisories | 0 | 0 | No change — CLEAN |
| Advisory DB entries | 1159 | 1159 | No change |
| cargo deny errors | 0 | 0 | No change |
| cargo deny warnings | 9 | 9 | No change — same DEP-006/DEP-007 set |
| New advisories against wirerust deps | 0 | 0 | No new exposure |
| syn versions | 1.0.109 / 2.0.117 | 1.0.109 / 2.0.117 | Unchanged (DEP-007 stable) |
| deny.toml unused licenses | 8 | 8 | Unchanged (DEP-006 stable) |
| indicatif | 0.18.5 | 0.18.6 | Bumped by Dependabot #386 (already merged) — no advisory, no security impact |

The only dependency change since the prior sweep is indicatif 0.18.5 → 0.18.6 (patch release, merged via Dependabot PR #386). The log confirms `indicatif v0.18.6` in the dependency chain at Cargo.lock:135. cargo audit reports zero advisories against 0.18.6.

---

## Summary

| Severity | Count | Finding IDs | Fix PR Required? |
|---|---|---|---|
| CRITICAL | 0 | — | — |
| HIGH | 0 | — | — |
| MEDIUM | 0 | — | — |
| LOW | 2 | DEP-006, DEP-007 | No — both deferred |

**No fix PR required. Audit CLEAN. Advisory DB entry count: 1159 (maint-2026-07-09).**
