# Phase 6 Formal Hardening — Security / Supply-Chain Scan Summary

- **Date:** 2026-06-01
- **Commit (develop):** `48b61e5`
- **Tooling:** `cargo-audit 0.22.1`, `cargo-deny 0.19.6`
- **Mode:** READ-ONLY analysis (no source / Cargo.toml / Cargo.lock modifications)
- **Advisory DB:** RustSec advisory-db, 1100 advisories loaded
- **Crates scanned:** 203 dependency crates from `Cargo.lock`

---

## Verdict: PASS (no actionable advisories)

develop is **clean** on the security gate as CI enforces it. The two RustSec
findings below are both informational (unmaintained / unsound), not
CVE-severity vulnerabilities, and both are explicitly and intentionally
tolerated by repo config (`cargo audit --ignore` flags in CI + `deny.toml`
delegating advisories to the audit job). Local raw-default results match the
last CI posture once the CI-equivalent commands are used.

---

## 1. `cargo audit` (RustSec advisory DB scan of Cargo.lock)

**Raw `cargo audit` (no ignores):** exit 0, `2 allowed warnings found`.
Both are `Warning:` (informational) class, not `Vulnerability:` — so even
raw audit does not fail. Two advisories surfaced:

### RUSTSEC-2025-0119 — `number_prefix` unmaintained
- **Class / severity:** `unmaintained` (informational; no CVSS, not a vulnerability)
- **Crate / version:** `number_prefix` 0.4.0
- **Direct or transitive:** TRANSITIVE — `number_prefix 0.4.0 -> indicatif 0.17.11 -> wirerust 0.1.0`
- **Fixed version available:** NO. Advisory states "No safe upgrade is available!"
  Recommended alternative is the `unit-prefix` crate, but the dependency is
  pulled in by `indicatif`; no `indicatif` release currently switches off
  `number_prefix`. Resolution is upstream's responsibility (indicatif progress-bar dep).
- **Disposition:** Tolerated. CI suppresses via `cargo audit --ignore RUSTSEC-2025-0119`.

### RUSTSEC-2026-0097 — `rand` 0.8.5 unsound
- **Class / severity:** `unsound` (informational; not a CVE-severity vulnerability)
- **Crate / version:** `rand` 0.8.5
- **Direct or transitive:** TRANSITIVE — `rand 0.8.5 -> phf_generator 0.11.3 -> phf_codegen 0.11.3 -> tls-parser 0.12.2 -> wirerust 0.1.0`
- **Title:** "Rand is unsound with a custom logger using `rand::rng()`"
- **Fixed version available:** Upstream `rand` has newer releases, but the
  0.8.5 pin is imposed by `tls-parser`'s transitive chain (`phf_generator`).
  Upgrading is `tls-parser` upstream's responsibility; wirerust does not
  depend on `rand` directly. The unsound code path requires a custom logger
  plus a reseed race, which wirerust does not exercise.
- **Disposition:** Tolerated. CI suppresses via `cargo audit --ignore RUSTSEC-2026-0097`.

**CI-equivalent command** (`cargo audit --ignore RUSTSEC-2025-0119 --ignore RUSTSEC-2026-0097`): **exit 0, CLEAN.**

---

## 2. `cargo deny check` (per section)

`deny.toml` is present at repo root. Its `[advisories]` table is intentionally
minimal — the file's own comments state advisory scanning is delegated to the
`cargo audit` job to avoid double-reporting. CI accordingly runs
`cargo deny check bans licenses sources` (NOT the default `check`, which would
also run `advisories`).

| Section | Default `cargo deny check` | CI command (`check bans licenses sources`) |
|---|---|---|
| advisories | **FAILED** (1 error) — see note | not run by CI (delegated to `cargo audit`) |
| bans | ok (2 duplicate warnings) | ok |
| licenses | ok (8 unmatched-allowance warnings) | ok |
| sources | ok | ok |

### advisories (default run only) — FAILED
- `error[unmaintained]: number_prefix crate is unmaintained` (RUSTSEC-2025-0119).
- This is the SAME finding as cargo-audit above. cargo-deny escalates
  `unmaintained` to an **error** by default, whereas cargo-audit treats it as a
  warning. This is why a naive `cargo deny check` exits 1 locally.
- **Not a CI gate:** CI never runs the `advisories` section under deny (it runs
  only `bans licenses sources`). So this does not represent a develop regression.

### bans — ok
- `wildcards = "deny"`: clean (no wildcard version requirements).
- `multiple-versions = "warn"`: 2 duplicate-version warnings (non-blocking, `skip`/`skip-tree` empty):
  - `syn` 1.0.109 vs 2.0.117 — v1 via `pcap-file`/`tls-parser` proc-macro chains; v2 via clap/serde/etc. Unavoidable across the dep graph.
  - `windows-sys` 0.59.0 vs 0.61.2 — v0.59 via `console`/`indicatif`; v0.61 via clap/rustix/etc.
- Both are `warn`-level by policy, so the section passes.

### licenses — ok
- 8 `license-not-encountered` warnings: `0BSD`, `Apache-2.0 WITH LLVM-exception`,
  `BSD-2-Clause`, `CC0-1.0`, `ISC`, `MPL-2.0`, `Unicode-DFS-2016`, `Zlib`.
- These are allow-list entries in `deny.toml` that no crate in the current
  graph actually uses — pure hygiene warnings (the allow list is broader than
  strictly needed). No disallowed license detected; section passes.
- `confidence-threshold = 0.93`, `exceptions = []` — no per-crate overrides needed.

### sources — ok
- `unknown-registry = "deny"`, `unknown-git = "deny"`; only
  `https://github.com/rust-lang/crates.io-index` allowed. All deps from
  crates.io; no git or local-path deps. Section passes.

**CI-equivalent command** (`cargo deny check bans licenses sources`): **exit 0, all three sections ok.**

---

## 3. Cross-check vs CI posture

CI (`.github/workflows/ci.yml`) runs two jobs:

- **`audit`** — `continue-on-error: true` (informational on PRs), runs
  `cargo audit --ignore RUSTSEC-2025-0119 --ignore RUSTSEC-2026-0097`.
  Reproduced locally: **exit 0, CLEAN.**
- **`deny`** — blocking, via `EmbarkStudios/cargo-deny-action@v2` with
  `command: check bans licenses sources` (advisories deliberately omitted).
  Reproduced locally: **exit 0, all sections ok.**

**Match confirmed.** Local results match the CI posture: develop is clean on
the enforced security gate. The only "failure" (default `cargo deny check`
advisories -> error on `number_prefix`) is outside the CI gate by design and
is the same finding already triaged/ignored in the audit job.

### Config-tolerated exceptions (intentional, documented)
1. **RUSTSEC-2025-0119** (`number_prefix` unmaintained) — ignored in CI audit; no upstream fix; transitive via `indicatif`.
2. **RUSTSEC-2026-0097** (`rand` 0.8.5 unsound) — ignored in CI audit; transitive via `tls-parser`; upstream's fix.
3. cargo-deny `advisories` section excluded from CI deny command (delegated to audit job per `deny.toml` design).
4. Duplicate-version warnings (`syn`, `windows-sys`) — `multiple-versions = "warn"` (non-blocking).
5. License allow-list breadth — 8 unused-but-allowed licenses produce hygiene warnings only.

---

## 4. Triage of findings (no fixes applied)

| ID | Severity class | Crate | Dep type | Fixed version? | Routing |
|---|---|---|---|---|---|
| RUSTSEC-2025-0119 | unmaintained (informational) | number_prefix 0.4.0 | transitive (via indicatif) | NO (no safe upgrade) | No action; revisit when indicatif drops number_prefix |
| RUSTSEC-2026-0097 | unsound (informational) | rand 0.8.5 | transitive (via tls-parser) | upstream tls-parser must bump | No action; path not exercised by wirerust |

Neither is a CVE-severity vulnerability. No HIGH/CRITICAL findings — no
security-reviewer escalation required. Were a real CVE-severity advisory
present, it would route to a dedicated fix-PR with security-reviewer triage
rather than being addressed in this read-only scan.

---

## Evidence

- This file: `.factory/cycles/v0.1.0-greenfield-spec/hardening/security-scan-summary.md`
- Source config: `deny.toml` (repo root), `.github/workflows/ci.yml` (jobs `audit`, `deny`)
