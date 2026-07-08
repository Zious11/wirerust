# Dependency Audit Findings — Maintenance Sweep 1 (ANALYSIS phase)

- **Date:** 2026-07-08
- **Run ID:** maint-2026-07-08
- **Producer:** security-reviewer
- **Crate version audited:** wirerust v0.11.5 (193 locked dependencies)
- **Tools:** cargo-audit 0.22.1, cargo-deny 0.19.6; cargo-outdated NOT INSTALLED
- **Advisory DB entries at scan time:** 1159 (delta: +1 from maint-2026-07-06 baseline of 1158)
- **Raw scan source:** `.factory/maintenance/dependency-audit-raw.log` (timestamp 2026-07-08T17:23:34Z)
- **Prior sweep reference:** this file, run maint-2026-07-06

---

## Overall Verdict: CLEAN

**cargo audit:** ZERO advisories against 193 locked crates.
**cargo deny:** ZERO errors. 9 warnings — all known, all already registered (DEP-006 × 8, DEP-007 × 1).
**cargo outdated:** SKIPPED — tool not installed; see note below.

No CRITICAL or HIGH findings. No MEDIUM findings. The audit is NON-BLOCKING for continued development. No immediate fix PR is required.

---

## Findings Table

| Finding ID | Advisory / CWE | Dependency | Severity | Status | Action |
|---|---|---|---|---|---|
| DEP-006 | N/A (hygiene) | deny.toml — 8 unused `license-not-encountered` entries | LOW (informational) | DEFERRED (registered maint-2026-07-06) | Remain deferred — see recommendation below |
| DEP-007 | N/A (ecosystem migration) | syn 1.0.109 + 2.0.117 duplicate | LOW (informational) | DEFERRED (registered maint-2026-07-06) | No action; upstream resolution |

**CRITICAL:** 0 | **HIGH:** 0 | **MEDIUM:** 0 | **LOW:** 2 (both pre-existing, deferred)

---

## Detailed Analysis

### Raw Log Verification

The raw log at `.factory/maintenance/dependency-audit-raw.log` (2026-07-08T17:23:34Z) confirms:

1. **cargo audit** loaded 1159 advisories and scanned all 193 Cargo.lock entries. The output contains no advisory blocks, no `RUSTSEC-` findings, no `[VULNERABILITY]` or `[WARNING]` entries. The implicit clean exit is consistent with the "advisories ok" summary line from cargo deny confirming advisory pass.

2. **cargo deny** emitted exactly 9 warnings, no errors:
   - 8× `warning[license-not-encountered]` for: `0BSD` (deny.toml:34), `Apache-2.0 WITH LLVM-exception` (deny.toml:25), `BSD-2-Clause` (deny.toml:26), `CC0-1.0` (deny.toml:32), `ISC` (deny.toml:28), `MPL-2.0` (deny.toml:33), `Unicode-DFS-2016` (deny.toml:30), `Zlib` (deny.toml:31) — all map to DEP-006.
   - 1× `warning[duplicate]` for `syn 1.0.109 / 2.0.117` at Cargo.lock:135 — maps to DEP-007.

3. **Terminator line:** `advisories ok, bans ok, licenses ok, sources ok` — all four cargo deny check categories passed with no errors. The log contains no buried errors or suppressed output. The 9 warnings are the complete and exhaustive set.

4. **cargo outdated:** Not installed; no outdated-version data available for this sweep. For reference, maint-2026-07-06 found 35 crates with available updates, of which all relevant security items (rand 0.8.6, zerocopy 0.8.52) were resolved by PR #304 (maint-2026-06-22). No new outdated-version intelligence can be produced without installing the tool.

---

### DEP-006 — Stale deny.toml license allowlist (8 license-not-encountered warnings)

**Status:** DEFERRED (registered maint-2026-07-06)
**Severity:** LOW (informational, no security implication)
**CWE:** Not applicable

Eight entries in `deny.toml` `[licenses] allow` are broader than the current dependency graph requires. The same 8 licenses flagged in maint-2026-07-06 are still unmatched: `0BSD`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `CC0-1.0`, `ISC`, `MPL-2.0`, `Unicode-DFS-2016`, `Zlib`. No new license-not-encountered entries appeared; no previously-flagged entries were resolved. The finding is stable and unchanged.

**Security implication:** None. All eight are permissive licenses compatible with MIT. The risk of inadvertently accepting a copyleft or hostile license through this vector is minimal.

---

### DEP-007 — syn 1.0.109 + 2.0.117 duplicate

**Status:** DEFERRED (registered maint-2026-07-06)
**Severity:** LOW (informational, expected ecosystem pattern)
**CWE:** Not applicable

syn 1.0.109 remains in the lock file via `derive-into-owned` (pcap-file) and `nom-derive-impl` (tls-parser/nom-derive). syn 2.0.117 is the dominant version pulled by clap, serde, thiserror, zerocopy, and wasm-bindgen chains. No change from maint-2026-07-06. No security advisory exists against either version.

---

### Unmaintained / Yanked / Compromised Check

cargo audit performed an explicit scan against all 193 locked crates using the RustSec advisory database. The database includes advisories in the `unmaintained`, `yanked`, and `unsound` categories in addition to CVE-class vulnerabilities. Zero advisories of any category were returned. No crates are flagged as unmaintained, yanked, or compromised as of the 1159-advisory DB snapshot at scan time.

---

## Diff vs maint-2026-07-06

| Dimension | maint-2026-07-06 | maint-2026-07-08 | Delta |
|---|---|---|---|
| cargo audit advisories | 0 (after RUSTSEC-2026-0204 fix PR #371) | 0 | No change — CLEAN |
| Advisory DB entries | 1158 (end-of-day, post-RUSTSEC-2026-0204) | 1159 | +1 new advisory (does not affect wirerust) |
| cargo deny errors | 0 | 0 | No change |
| cargo deny warnings | 9 | 9 | No change — same DEP-006/DEP-007 set |
| New advisories against wirerust deps | N/A | 0 | No new exposure |
| Resolved advisories since last sweep | N/A | 0 | Nothing to resolve |
| syn versions | 1.0.109 / 2.0.117 | 1.0.109 / 2.0.117 | Unchanged (DEP-007 stable) |
| deny.toml unused licenses | 8 | 8 | Unchanged (DEP-006 stable) |

**RUSTSEC-2026-0204 advisory-race lesson (carried forward):** The advisory-race pattern observed in maint-2026-07-06 (cargo-audit DB grew 1157→1158 mid-run, turning a CLEAN morning scan into a CI failure hours later) is now a standing monitoring note. This sweep records DB entry count 1159 at 2026-07-08T17:23:34Z. Future sweeps should compare their DB entry count against this baseline to detect new additions. A DB delta does not automatically mean new exposure; it means new advisories were published and the scan should be re-examined against the full crate list. A CLEAN cargo audit against an incremented DB count (as seen here: 1158→1159, still CLEAN) confirms no wirerust dependency was newly affected.

---

## DEP-006 Recommendation: Remain Deferred

**Question:** Should DEP-006 (deny.toml allowlist trim) be folded into this sweep's fix PR or remain deferred?

**Recommendation: Remain deferred.**

Rationale:
1. This sweep has zero actionable security findings. There is no fix PR to bundle DEP-006 with — creating a standalone `chore:` PR solely to trim 8 allowlist entries would be disproportionate effort for zero security gain.
2. DEP-006 requires running `cargo deny list` first to enumerate currently-active licenses before removing any entry, because removing a license that a transitive dependency actually uses (even if cargo deny does not currently warn about it) would break `cargo deny check`. This adds non-trivial investigation overhead.
3. The risk of deferral is minimal: all eight flagged licenses are permissive and MIT-compatible. A future dependency using one of these licenses would be silently accepted by `cargo deny`, but would not introduce a license-compliance risk.
4. The appropriate time to execute DEP-006 is when a `chore:` PR is already in flight for another reason (e.g., a doc-drift or tech-debt batch), so the fix can be bundled at near-zero marginal cost.

**Trigger condition for unblocking DEP-006:** Fold into the next `chore:` or `docs:` maintenance PR. Pre-condition: run `cargo deny list` to enumerate active licenses, verify the 8 flagged entries are genuinely absent, then remove them from deny.toml.

---

## Summary

| Severity | Count | Finding IDs | Fix PR Required? |
|---|---|---|---|
| CRITICAL | 0 | — | — |
| HIGH | 0 | — | — |
| MEDIUM | 0 | — | — |
| LOW | 2 | DEP-006, DEP-007 | No — both deferred |

**No fix PR required. Audit CLEAN. DB advisory count recorded: 1159 at 2026-07-08T17:23:34Z.**
