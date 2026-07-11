---
run-id: maint-2026-07-11
sweep: 1 (security analysis)
producer: security-reviewer
date: 2026-07-11
wirerust-version: 0.12.0
crate-count: 193
advisory-db-entries: 1159
tools: cargo-audit (implied by cargo deny advisories pass), cargo-deny 4/4 ok
raw-log: .factory/maintenance/dependency-audit-raw-2026-07-11.log
prior-sweep: maint-2026-07-08 (dependency-audit-findings.md)
---

# Dependency Audit Findings — Maintenance Sweep 1 (maint-2026-07-11)

## Overall Verdict: CLEAN

**cargo audit:** ZERO advisories against 193 locked crates (advisory DB: 1159 entries).  
**cargo deny:** ZERO errors. 1 warning — DEP-007 (syn duplicate), pre-existing and deferred.  
**cargo outdated:** NOT INSTALLED — skipped per raw log.

No CRITICAL, HIGH, or MEDIUM findings. No new advisories affect any wirerust dependency. The
audit is NON-BLOCKING for continued development. No fix PR required.

---

## Findings Classification Table

| Finding ID | Advisory / Category | Dependency | Severity | Status | Action |
|---|---|---|---|---|---|
| DEP-007 | N/A (ecosystem migration) | syn 1.0.109 + 2.0.117 duplicate | LOW (informational) | DEFERRED (registered maint-2026-07-06) | No action; upstream resolution |

**CRITICAL:** 0 | **HIGH:** 0 | **MEDIUM:** 0 | **LOW:** 1 (pre-existing, deferred)

> **DEP-006 note:** DEP-006 (8 unused deny.toml license allowlist entries) was RESOLVED in the
> maint-2026-07-09-doc-sweep PR. The raw log for this sweep shows only 1 cargo deny warning
> (the syn duplicate), confirming DEP-006 cleanup is in effect.

---

## Section 1 — Clean Classification Confirmation

The raw log at `.factory/maintenance/dependency-audit-raw-2026-07-11.log` confirms:

1. **Advisory scan:** "Loaded 1159 security advisories … Scanning Cargo.lock for vulnerabilities
   (193 crate dependencies)." No advisory blocks, no `RUSTSEC-` findings, no `[VULNERABILITY]`
   or `[WARNING]` entries appear in the output.

2. **cargo deny:** Exactly 1 warning — `warning[duplicate]` for syn 1.0.109 / 2.0.117 at
   Cargo.lock:135. This is DEP-007, pre-existing and deferred. No errors in any check category.

3. **Terminator line:** `advisories ok, bans ok, licenses ok, sources ok` — all four cargo deny
   check categories (advisories, bans, licenses, sources) passed with no errors. The 1 warning
   is exhaustive; no buried errors or suppressed output.

4. **cargo outdated:** Not installed; no outdated-version data produced this sweep. This is
   unchanged from maint-2026-07-08.

**Clean classification is confirmed.** No CRITICAL/HIGH/MEDIUM items. The sole registered finding
(DEP-007) is LOW/informational and pre-existing.

---

## Section 2 — Unmaintained / Abandoned Dependency Check

The flagged candidates are: `pcap-file` (v2.0.0), `tls-parser` (v0.12.2), and `nom-derive`
(v0.10.1, transitive via tls-parser). All three are direct or near-direct dependencies carrying
observable syn-v1 usage, which may indicate older codebase cadence.

### Baseline: RustSec advisory-DB result

The RustSec advisory database (1159 entries at scan time) explicitly includes advisories in the
`unmaintained`, `yanked`, and `unsound` categories in addition to CVE-class vulnerabilities.
The cargo-audit scan returned **zero advisories of any category** across all 193 locked crates.
This means **none of the three flagged crates carry a formal RUSTSEC `unmaintained` advisory**
as of the 1159-entry DB snapshot. A formal RUSTSEC unmaintained advisory is the strongest
available signal for abandonment risk in the Rust ecosystem.

### pcap-file v2.0.0

- **Locked version:** 2.0.0 (Cargo.lock). Cargo.toml requests `"2"` (caret: ≥2.0.0, <3.0.0).
- **RustSec status:** No advisory of any kind — confirmed by the clean scan.
- **Observable signals:** pcap-file pulls `derive-into-owned` v0.2.0, which in turn pulls
  syn v1.0.109. This chain indicates that either pcap-file or derive-into-owned has not migrated
  to syn 2; it is not itself a maintenance-concern signal, but suggests the dependency tree has
  not been freshly updated across the board.
- **Offline maintenance verdict:** INCONCLUSIVE. Publication date and release frequency cannot
  be verified without crates.io or GitHub access. No RUSTSEC advisory exists; no yanked version
  is present. The syn v1 chain is a hygiene note, not an abandonment signal.

### tls-parser v0.12.2

- **Locked version:** 0.12.2 (Cargo.lock). Cargo.toml requests `"0.12"`.
- **RustSec status:** No advisory of any kind — confirmed by the clean scan.
- **Observable signals:** tls-parser pulls `nom-derive` v0.10.1, which pulls `nom-derive-impl`
  v0.10.1, which pulls syn v1.0.109. The version peg at `"0.12"` is intentional per Cargo.toml
  comments; the project is sensitive to this dep's API contract.
- **Offline maintenance verdict:** INCONCLUSIVE. tls-parser is from the rusticata project family
  (historically active in Rust network parsing), but release cadence cannot be verified offline.
  No RUSTSEC advisory; no yanked version. The syn v1 chain is the same hygiene note as above.

### nom-derive v0.10.1

- **Locked version:** 0.10.1 (Cargo.lock). This is a transitive dependency via tls-parser.
- **RustSec status:** No advisory of any kind — confirmed by the clean scan.
- **Observable signals:** nom-derive-impl pulls syn v1.0.109, consistent with pre-syn-2 codebase.
- **Offline maintenance verdict:** INCONCLUSIVE. Transitive dep; no direct maintainability
  exposure to wirerust beyond the tls-parser chain. No RUSTSEC advisory.

### Summary

| Crate | Version | Formal RUSTSEC | Offline Date Check | Verdict |
|---|---|---|---|---|
| pcap-file | 2.0.0 | None | Cannot verify | INCONCLUSIVE — no active advisory |
| tls-parser | 0.12.2 | None | Cannot verify | INCONCLUSIVE — no active advisory |
| nom-derive | 0.10.1 (transitive) | None | Cannot verify | INCONCLUSIVE — no active advisory |

**No unmaintained flag raised for any crate.** The RustSec CLEAN result (which covers the
`unmaintained` category) is the strongest available offline signal; absence of formal advisory
means no action is warranted this sweep. If online research capability becomes available, a
follow-on check against crates.io release history for pcap-file and tls-parser (>18 months
since last release is the threshold) would convert INCONCLUSIVE to a definitive verdict.

---

## Section 3 — Advisory-Race Note (DB Delta Tracking)

**Advisory DB at scan time: 1159 entries.**

Comparison against prior sweeps:

| Sweep | DB entries | Delta |
|---|---|---|
| maint-2026-07-06 | 1158 (end-of-day, post-RUSTSEC-2026-0204) | baseline |
| maint-2026-07-08 | 1159 | +1 |
| **maint-2026-07-11 (this sweep)** | **1159** | **0 (stable)** |

The DB count is stable at 1159 between the 2026-07-08 and 2026-07-11 sweeps. No new advisories
were published against the RustSec DB in this window. A CLEAN result against a stable DB count
carries higher confidence than a CLEAN result against a growing DB (no new advisory raced in
during the scan window).

**RUSTSEC-2026-0204 precedent (carried forward):** The advisory-race pattern observed in
maint-2026-07-06 (DB grew 1157→1158 mid-run, crossbeam-epoch advisory published after the
morning scan but before CI ran on the fix PR) is the standing monitoring model. The DB entry
count recorded here (1159 at 2026-07-11) serves as the baseline for future delta diagnosis on
any fix-PR CI run. A DB count increase on a subsequent CI run does not automatically indicate
new exposure to wirerust; the new advisory(ies) must be checked against the full Cargo.lock
crate list before concluding impact.

---

## Diff vs maint-2026-07-08

| Dimension | maint-2026-07-08 | maint-2026-07-11 | Delta |
|---|---|---|---|
| cargo audit advisories | 0 | 0 | No change — CLEAN |
| Advisory DB entries | 1159 | 1159 | Stable (+0) |
| cargo deny errors | 0 | 0 | No change |
| cargo deny warnings | 9 (DEP-006 ×8 + DEP-007 ×1) | 1 (DEP-007 ×1) | −8 (DEP-006 RESOLVED maint-2026-07-09) |
| New advisories against wirerust deps | 0 | 0 | No new exposure |
| syn versions | 1.0.109 / 2.0.117 | 1.0.109 / 2.0.117 | Unchanged (DEP-007 stable) |
| deny.toml unused licenses | 8 (deferred at that sweep) | 0 (resolved) | RESOLVED maint-2026-07-09 |

---

## Summary

| Severity | Count | Finding IDs | Fix PR Required? |
|---|---|---|---|
| CRITICAL | 0 | — | — |
| HIGH | 0 | — | — |
| MEDIUM | 0 | — | — |
| LOW | 1 | DEP-007 | No — deferred |

**No fix PR required. Audit CLEAN. Advisory DB count recorded: 1159 at scan time (2026-07-11,
stable vs maint-2026-07-08).**

Unmaintained-dep check: pcap-file v2.0.0, tls-parser v0.12.2, nom-derive v0.10.1 all returned
INCONCLUSIVE (offline only; no RUSTSEC `unmaintained` advisory for any crate; advisory DB scan
is clean across all categories). No flag raised; no action required this sweep.
