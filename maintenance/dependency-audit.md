# Dependency Audit Analysis — Maintenance Sweep 1b

- **Date:** 2026-06-17
- **Sweep:** Maintenance Sweep 1 (post-v0.7.1 steady-state)
- **Raw scan source:** `.factory/maintenance/dependency-audit-raw.log`
- **Crates scanned:** 193 (Cargo.lock)
- **Tooling:** cargo-audit (advisory DB 1134 advisories), cargo-deny, cargo update --dry-run
- **Semgrep:** NOT INSTALLED (skipped)
- **cargo-outdated:** NOT INSTALLED (cargo update --dry-run used as substitute)

---

## Overall Verdict: NON-BLOCKING

Zero CVE-severity vulnerabilities. One accepted informational advisory
(RUSTSEC-2026-0097) with a now-available trivial fix. No compromised or
unmaintained dependencies blocking release. No findings require immediate
remediation before the next development cycle.

---

## Finding Classification

### FINDING-001 — RUSTSEC-2026-0097: rand 0.8.5 unsound (transitive, build-dep)

- **Severity:** LOW
- **RUSTSEC ID:** RUSTSEC-2026-0097
- **CWE:** CWE-119 (Improper Restriction of Operations within Bounds of Memory
  Buffer) / soundness violation — undefined behavior possible in specific
  concurrent-logger scenario
- **Crate / version:** rand 0.8.5
- **Dependency path:** rand 0.8.5 → phf_generator 0.11.3 → phf_codegen 0.11.3
  → tls-parser 0.12.2 → wirerust 0.7.1
- **Advisory class:** `unsound` (informational warning; not a vulnerability with
  CVSS score)
- **Fixed version available:** YES — rand 0.8.6 (confirmed by `cargo update
  --dry-run` output: `Updating rand v0.8.5 -> v0.8.6`)
- **Prior disposition:** ACCEPTED-TRANSITIVE since F6 hardening (2026-06-01),
  documented in `.factory/cycles/v0.1.0-greenfield-spec/hardening/security-scan-summary.md`
  §RUSTSEC-2026-0097. CI suppresses via `--ignore RUSTSEC-2026-0097`
  (`continue-on-error: true` audit job).

**Exploitability assessment:**

The unsound condition requires (a) a custom global logger that calls
`rand::rng()` from within a log macro invocation, AND (b) a concurrent reseed
race. wirerust:

1. Does not depend on `rand` directly (it is a build-time transitive dep via
   phf_codegen, a code-generation tool that runs only at `cargo build` time).
2. Does not install a custom logger.
3. The phf_codegen usage produces static perfect-hash tables compiled into the
   binary; rand is consumed entirely at build time, not at runtime.

Runtime exploitability: NIL. The unsound path cannot be triggered in any
wirerust process instance. The finding is a build-environment concern only.

**New information since prior acceptance:** rand 0.8.6 is now available and
reachable via `cargo update -p rand` without any Cargo.toml changes (semver
compatible patch bump). This changes the disposition from "upstream must fix"
to "trivially self-serviceable."

**Fix recommendation:** Run `cargo update -p rand` to resolve to 0.8.6. This
is automatable and low-risk (patch bump; no API changes). After updating,
remove `--ignore RUSTSEC-2026-0097` from the CI audit command in
`.github/workflows/ci.yml`.

**Classification rationale:** Severity remains LOW (not HIGH/CRITICAL) because
the code path is build-only and non-exploitable at runtime. The advisory is
formally `unsound`, not a vulnerability. Upgrading is RECOMMENDED but not
blocking.

---

### FINDING-002 — cargo-deny: 8 license-not-encountered warnings

- **Severity:** LOW (informational, hygiene only)
- **Advisory class:** `license-not-encountered` (cargo-deny warning, not an error)
- **Detail:** 8 entries in `deny.toml` `[licenses] allow` list are unused — no
  crate in the current dependency graph carries these licenses:
  0BSD, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, CC0-1.0, ISC, MPL-2.0,
  Unicode-DFS-2016, Zlib.
- **Security implication:** NONE. These are allowlist entries that are broader
  than strictly needed. No disallowed license is present; the `licenses` section
  passes cleanly. An overly broad allowlist is a minor hygiene concern — if a
  future dependency adds one of these licenses it would pass silently — but the
  licensed types are all permissive and compatible with MIT.
- **OWASP:** Not applicable (supply-chain hygiene, not a vulnerability class).
- **CWE:** Not applicable.
- **Fix:** Optional. Pruning the `allow` list to only licenses actually used
  tightens the policy, but must be done carefully: removing a license from the
  list that a transitive dep quietly uses would break `cargo deny check`. The
  correct approach is `cargo deny list` to enumerate actual licenses, then trim.
  **Not automatable without inspection.** Defer to next license-audit sweep.

---

### FINDING-003 — cargo-deny: syn v1 / v2 duplicate

- **Severity:** LOW (informational, expected)
- **Advisory class:** `duplicate` (cargo-deny `multiple-versions = "warn"`)
- **Detail:** syn 1.0.109 and syn 2.0.117 both present. v1 is pulled in by
  `derive-into-owned` (via pcap-file) and `nom-derive-impl` (via nom-derive /
  tls-parser). v2 is pulled in by clap, serde, thiserror, zerocopy, and
  wasm-bindgen chains.
- **Security implication:** NONE. Proc-macro version duplication is a standard
  ecosystem migration pattern. The v1/v2 coexistence is inevitable while
  tls-parser and pcap-file have not migrated their proc-macro chains. cargo
  audit shows no advisory against either syn version.
- **CWE:** Not applicable.
- **Fix:** No action warranted. Resolution is upstream's responsibility (tls-parser,
  pcap-file). Will resolve naturally as those crates update. Confirm syn 2.0.118
  lands when `cargo update` is run (dry-run shows `syn v2.0.117 -> v2.0.118`).

---

### FINDING-004 — 35 crates with available updates (cargo update --dry-run)

- **Severity:** LOW (maintenance hygiene; no known CVEs in updated set)
- **CWE:** Not applicable (no specific vulnerability; general supply-chain
  freshness concern — CWE-1104: Use of Unmaintained Third-Party Components,
  partial analogy for stale patch versions).
- **Detail:** 35 crates have patch or minor updates available. Six new transitive
  crates would be added (futures-core, futures-task, futures-util,
  pin-project-lite, slab, wit-bindgen). Notable individual updates:
  - `rand 0.8.5 → 0.8.6` — resolves RUSTSEC-2026-0097 (see FINDING-001).
  - `zerocopy 0.8.48 → 0.8.52` — zerocopy has a history of soundness advisories;
    updating is prudent.
  - `shlex 1.3.0 → 2.0.1` — major version bump (2.0.x); requires judgment on
    whether dependents can absorb a breaking change. Likely a dev/build dep
    (used by cc crate). Verify before updating.
  - `wasm-bindgen 0.2.117 → 0.2.125` — dev-dep (criterion/plotters chain; not
    in production binary).
  - `syn 2.0.117 → 2.0.118` — proc-macro chain; safe patch bump.
  - `bitflags 2.11.0 → 2.13.0` — minor bump; low risk.
  - `hashbrown 0.16.1 → 0.17.1` — minor bump; check if any direct hashbrown
    usage exists (likely transitive only).
- **New transitive crates assessment:**
  - futures-core / futures-task / futures-util — likely added by fastrand 2.4.x
    or similar; these are well-maintained core async primitives (futures-rs).
    No known CVEs. Acceptable addition.
  - pin-project-lite — widely used, extremely stable, no known CVEs.
  - slab — widely used allocator primitive, no known CVEs.
  - wit-bindgen 0.57.1 — WASM interface types; added via winnow 1.0.3. Warrants
    brief inspection: is winnow actually used, or only a transitive of toml_edit?
    toml_edit is a dev/build dep (cargo tooling chain, not in production binary).
    wit-bindgen would thus be a dev-only transitive addition.
- **Fix:** Run `cargo update` (all safe patch/minor bumps). Then run
  `cargo test --all-targets` to confirm no regressions. The `shlex 2.0.1`
  major-version bump is the only item requiring judgment — `shlex` is used by
  `cc` (C compiler wrapper, build dep only). Semver-major from 1.x to 2.x means
  API may change, but since it is a build dep (not in the production binary), the
  blast radius is limited. Automatable for all except shlex major bump (requires
  CI green confirmation).

---

### FINDING-005 — zerocopy update available (0.8.48 → 0.8.52)

- **Severity:** MEDIUM (precautionary — zerocopy has prior soundness CVE history)
- **CWE:** CWE-119 (potential memory safety if unsound version used); prior
  RUSTSEC-2023-0074 established zerocopy soundness advisory precedent.
- **Detail:** zerocopy 0.8.48 is current in Cargo.lock. zerocopy 0.8.52 is
  available. zerocopy has a documented history of soundness advisories (most
  notably RUSTSEC-2023-0074, since fixed). The gap from .48 to .52 within 0.8.x
  suggests several patch fixes have landed.
- **Current advisory status:** cargo audit reports 0 vulnerabilities against
  zerocopy 0.8.48 — there is NO active advisory against .48 as of today's DB
  scan (1134 advisories loaded 2026-06-17). This finding is PRECAUTIONARY, not
  reactive to a known CVE.
- **Exploitability:** Not currently exploitable — no active advisory exists.
  Upgrading to .52 is prudent given zerocopy's history of discovering post-release
  unsoundness in sub-patches.
- **Fix:** Included in `cargo update` (patch bump; automatable). HIGH priority
  within the update batch given zerocopy's soundness history, even absent an
  active advisory.

---

## Compromised or Unmaintained Dependency Check

| Crate | Status | Evidence |
|---|---|---|
| `rand` 0.8.5 | Maintained (0.8.6 available, 0.9.x branch active) | No unmaintained advisory |
| `tls-parser` 0.12.2 | Maintained (active on crates.io) | No unmaintained advisory |
| `phf_codegen` 0.11.3 | Maintained | No advisory |
| `zerocopy` 0.8.48 | Maintained; actively patched | Prior soundness history; current scan clean |
| `syn` 1.0.109 | Maintained (dtolnay; v1 LTS) | No advisory |
| `number_prefix` | **Previously flagged RUSTSEC-2025-0119 unmaintained** | RESOLVED: indicatif 0.17→0.18 bump in prior cycle replaced number_prefix with unit-prefix; no longer present in dep graph (absent from current scan output — 0 unmaintained warnings in this sweep) |

No compromised dependencies detected. No supply-chain injection indicators. All
sources are crates.io (`sources: ok` per cargo-deny). No git or local-path deps.

---

## Recommended Actions

| Priority | Action | Automatable | Effort |
|---|---|---|---|
| P1 — RECOMMENDED | `cargo update -p rand` (resolves RUSTSEC-2026-0097, bumps to 0.8.6) | YES — `cargo update -p rand` | < 5 min |
| P1 — RECOMMENDED | `cargo update -p zerocopy` (precautionary; .48 → .52) | YES — included in full `cargo update` | < 5 min |
| P2 — RECOMMENDED | Full `cargo update` (all 35 patch/minor bumps) + CI verify | YES for most; shlex 2.0.1 requires CI green confirmation | ~30 min total |
| P3 — DEFERRED | Trim `deny.toml` license allowlist to actually-used licenses | NO — requires `cargo deny list` audit + judgment | Low-urgency |
| P3 — DEFERRED | `shlex 1.3.0 → 2.0.1` (major bump, build dep only) | Verify-required — run `cargo build` and CI | Part of full update batch |

After P1/P2 are complete:
- Remove `--ignore RUSTSEC-2026-0097` from `.github/workflows/ci.yml` audit command.
- Commit Cargo.lock with updated checksums.
- Run `cargo test --all-targets` to confirm no regression.

---

## Summary Table

| ID | Finding | Severity | CWE | Fix Available | Blocking? |
|---|---|---|---|---|---|
| FINDING-001 | RUSTSEC-2026-0097: rand 0.8.5 unsound (build-dep, not runtime) | LOW | CWE-119 (soundness) | YES — `cargo update -p rand` | NO (accepted-transitive; build-only) |
| FINDING-002 | 8 license-not-encountered warnings in deny.toml | LOW | N/A | Optional prune | NO |
| FINDING-003 | syn v1/v2 duplicate | LOW | N/A | Upstream (tls-parser/pcap-file) | NO |
| FINDING-004 | 35 crates with available updates | LOW | CWE-1104 (partial) | YES — `cargo update` | NO |
| FINDING-005 | zerocopy 0.8.48 (precautionary; no active advisory) | MEDIUM | CWE-119 (precautionary) | YES — `cargo update` | NO |

**No CRITICAL or HIGH findings. Overall verdict: NON-BLOCKING.**

---

## Notes on Scan Coverage Gaps

- **semgrep:** Not installed. Static SAST coverage for source-code patterns
  (hardcoded secrets, injection, unsafe usage) was not performed this sweep.
  For a Rust CLI tool with no network-facing server surface, the gap is limited
  in practice. Recommend installing for the next formal hardening phase.
- **cargo-outdated:** Not installed. `cargo update --dry-run` was used as a
  substitute and provides equivalent major/minor/patch availability data.
  cargo-outdated would additionally surface SemVer-incompatible (major) updates,
  which the dry-run does not show (it only shows compatible updates). No action
  blocked by this gap.
- **Transitive-only rand dependency:** wirerust does not directly use rand.
  The dependency is phf_codegen (build-time code generation). If tls-parser
  were ever replaced or updated to a phf that pins rand 0.9.x, FINDING-001
  would disappear automatically.
