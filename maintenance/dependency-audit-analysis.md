# Dependency Audit — Severity Analysis

**Run:** maint-2026-09-05
**Stage:** T2 SEVERITY ANALYSIS (security-reviewer)
**Reviewer:** security-reviewer
**Inputs reviewed:**
- `.factory/maintenance/dependency-audit-raw.log`
- `.factory/maintenance/dependency-audit-scan-summary.md`
- `.github/dependabot.yml` (consulted to determine update-cadence disposition)

## Top-Line Verdict

**CLEAN — 0 actionable advisories. 1 LOW/informational finding (duplicate `syn`
versions, build-time only). No fix PR required this sweep. Routine `cargo update`
refresh is NOT warranted ad hoc — defer to Dependabot's existing cooldown-gated
cadence.**

No CRITICAL, HIGH, or MEDIUM findings. No CVE/RUSTSEC IDs apply.

---

## 1. Hidden Unmaintained/Unsound/Yanked Advisory Check

Reviewed the full raw `cargo audit` output (raw log lines 1–10) line by line, not
just the scan-summary's grep. `cargo audit` (cargo-audit-audit 0.22.1) prints
unmaintained-crate (RUSTSEC `*-unmaintained`), unsound (`*-unsound`), and yanked
notices as ordinary findings in its stdout even when they don't affect the audit's
pass/fail advisory count — none appear here. The complete non-banner output is:

```
Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
  Loaded 1239 security advisories (from /Users/zious/.cargo/advisory-db)
Updating crates.io index
Scanning Cargo.lock for vulnerabilities (175 crate dependencies)
```

followed immediately by `=== cargo audit exit code: 0 ===`. There is no
"Warning:" block (the format `cargo-audit` uses for yanked-crate and
unmaintained/unsound informational notices), no `ID`/`Crate`/`Title` advisory
table, and no RUSTSEC-YYYY-NNNN string anywhere in the tool's own output. Exit
code 0 with zero warning blocks confirms: no vulnerability advisories, no yanked
crates, no unmaintained-crate notices, no unsound notices.

**Disposition:** Confirmed clean — no hidden advisories of any class.

## 2. Duplicate-`syn` Finding Classification

- **Severity:** LOW (informational)
- **CWE:** Not applicable — this is a `cargo-deny` `bans[duplicate]` lockfile
  hygiene lint, not a vulnerability. No CWE/CVE/RUSTSEC ID applies because no
  advisory, unsound behavior, or exploitable condition is implicated.
- **OWASP:** Not applicable (no OWASP Top 10 category maps to build-time
  proc-macro duplication).

**Root cause (from raw log lines 13–83):** Two coexisting `syn` majors in
`Cargo.lock`:
- `syn v1.0.109` is pulled by:
  - `derive-into-owned v0.2.0` → `pcap-file v2.0.0` → `wirerust`
  - `nom-derive-impl v0.10.1` → `nom-derive v0.10.1` → `tls-parser v0.12.2` →
    `wirerust`
- `syn v2.0.117` is pulled by six independent consumers (`clap_derive`,
  `num_enum_derive`, `serde_derive`, `thiserror-impl`,
  `wasm-bindgen-macro-support`, `zerocopy-derive`), all standard/recent proc-
  macro ecosystem crates already on `syn` 2.x.

**Actionability now:** Not directly fixable by wirerust. The `syn` 1.x pull is
gated by two *upstream* crates' own dependency choices — `derive-into-owned`
(a transitive of `pcap-file`) and `nom-derive-impl` (a transitive of
`nom-derive`/`tls-parser`) — neither of which is a `wirerust` direct dependency
that this repo's `Cargo.toml` controls. `cargo update --dry-run` (raw log lines
89–149) does not remove the `syn v1.0.109` lockfile entry at all — it only
touches the `syn` 2.x line (`v2.0.117` → adds `v2.0.119` and `v3.0.5`),
confirming the 1.x pull is a genuine long-tail transitive with no available
lockfile-only remediation. Upgrading `pcap-file` or `tls-parser` to versions
that drop their `syn`-1.x-era macro-helper dependencies (if such versions
exist upstream) would be the only real fix, and that is a `Cargo.toml`
manifest change, not a lockfile refresh — out of scope for this sweep's
"log only" disposition.

**Impact if left as-is:** `syn` is a compile-time-only proc-macro parsing
dependency — it is never linked into the built `wirerust` binary and exposes
no runtime attack surface. The only cost of the duplication is marginal build-
time/disk bloat (two copies of a proc-macro crate compiled). This is textbook
supply-chain hygiene noise, not a security exposure.

## 3. Disposition Recommendation (per maintenance rubric)

| Finding | Severity | Action |
|---|---|---|
| `cargo audit`: 0 advisories | N/A | No action — clean baseline. |
| `cargo deny`: duplicate `syn` 1.0.109/2.0.117 | LOW | **Log only.** Not actionable via lockfile refresh (confirmed above); would require an upstream `pcap-file`/`tls-parser` manifest bump, which is out of scope for a dependency-audit sweep. Track as a drift item for a future architecture/dependency-upgrade story if `pcap-file` or `tls-parser` ship a syn-2.x-only release. |
| `cargo update --dry-run`: 54 packages have newer compatible versions | LOW/informational | **Defer to Dependabot — do not open an ad hoc `cargo update` refresh PR this sweep.** |

**Rationale for deferring the 54-package refresh to Dependabot:**
`.github/dependabot.yml` already runs a daily `cargo` ecosystem check with an
explicit cooldown policy (`default-days: 7`, `semver-minor-days: 7`,
`semver-patch-days: 7`, `semver-major-days: 30`) — a deliberate supply-chain
hardening measure so that a newly published version has time to "soak" before
this repo pulls it, letting yanked/post-publish-hijacked releases age out
first. Cooldown is explicitly bypassed for security advisories, but none exist
here (see §1). An ad hoc manual `cargo update` run today would bypass that
soak protection for every one of the 54 packages, including any that haven't
yet cleared their cooldown window — this would be working *against* the
project's own documented supply-chain-hardening policy for zero security
benefit, since `cargo audit` found nothing forcing an early bump. Additionally,
several of the reported bumps are not simple patch/minor churn — e.g.
`shlex v1.3.0 -> v2.0.1` (major) and the `syn v2.0.117` lockfile line being
replaced by both `syn v2.0.119` and a new `syn v3.0.5` entry — which are
exactly the kind of change the `semver-major-days: 30` cooldown tier exists to
gate. Bundling 54 packages (including majors) into one manual lockfile-only PR
this sweep would also make any regression harder to bisect than Dependabot's
per-package PR cadence.

**Recommendation:** No fix PR this sweep. Let Dependabot's existing
cooldown-gated cadence pick up the 54 available upgrades on its normal
schedule (patches/minors after 7 days, majors after 30 days from publish).
Re-run `cargo audit`/`cargo deny` at the next scheduled maintenance sweep.

## 4. CWE/CVE/RUSTSEC Citations

None apply. As expected/predicted by the scan facts: `cargo audit` returned 0
advisories against 175 scanned dependencies (1239 advisories loaded), and no
unmaintained/unsound/yanked notices were present in the raw tool output. The
duplicate-`syn` finding is a `cargo-deny` `bans[duplicate]` lint with no
CWE/CVE/RUSTSEC mapping (see §2).

---

**Files not modified:** `Cargo.toml`, `Cargo.lock` — no changes made, per
instructions. No git commit/push performed.
