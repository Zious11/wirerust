# Dependency Audit Analysis — Maintenance Sweep 1b

- **Run ID:** maint-2026-07-06
- **Date:** 2026-07-06
- **Project:** wirerust
- **Branch:** develop @ f7460b4 (v0.11.4)
- **Raw scan source:** `.factory/maintenance/dependency-audit-raw.log`
- **Crates scanned:** 193 (Cargo.lock)
- **Advisory DB loaded:** 1157 advisories (cargo-audit 0.22.1)
- **cargo-deny:** 0.19.6 — exit 0, all 4 checks passed
- **cargo-outdated:** NOT INSTALLED (skipped)
- **semgrep:** NOT INSTALLED (skipped)

---

## Overall Verdict: CLEAN

Zero security advisories. Zero cargo-deny errors. All prior-cycle security findings
(RUSTSEC-2026-0097 rand, RUSTSEC-2026-0190 anyhow, zerocopy precautionary) are
confirmed cleared. Two informational/hygiene LOW items persist from the prior cycle
(license allowlist bloat, syn v1/v2 duplicate). No new findings introduced.

**Finding counts by severity: CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 2**

---

## Finding Classification

### FINDING-001 — cargo-deny: 8 license-not-encountered warnings (PERSISTS)

- **Severity:** LOW (informational, hygiene only)
- **Advisory class:** `license-not-encountered` (cargo-deny warning, not an error)
- **CWE:** N/A
- **Status:** UNCHANGED from prior cycle. Same 8 unused allowlist entries.
- **Detail:** 8 entries in `deny.toml` `[licenses] allow` remain unused — no crate in
  the current dependency graph carries these licenses:
  `0BSD`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `CC0-1.0`, `ISC`,
  `MPL-2.0`, `Unicode-DFS-2016`, `Zlib`.
- **Security implication:** NONE. The `licenses` check exits clean. An overly-broad
  allowlist is a minor hygiene concern — a future dependency carrying one of these
  permissive licenses would slip through silently — but all listed types are permissive
  and MIT-compatible. No disallowed license is present.
- **Fix:** Optional and deferred. Trim the allowlist by running `cargo deny list` to
  enumerate licenses actually in use, then removing unused entries. Must be done
  carefully: removing an entry that a transitive dep quietly carries breaks
  `cargo deny check`. Not automatable without inspection.

---

### FINDING-002 — cargo-deny: syn v1 / v2 duplicate (PERSISTS)

- **Severity:** LOW (informational, expected)
- **Advisory class:** `duplicate` (cargo-deny `multiple-versions = "warn"`)
- **CWE:** N/A
- **Status:** UNCHANGED from prior cycle. Same versions.
- **Detail:** syn 1.0.109 and syn 2.0.117 both present in Cargo.lock.
  - syn 1.0.109 is pulled in via `derive-into-owned` (pcap-file) and `nom-derive-impl`
    (nom-derive / tls-parser) — both direct dependencies of wirerust that have not yet
    migrated their proc-macro chains to syn 2.
  - syn 2.0.117 is pulled in by clap, serde, thiserror, zerocopy, wasm-bindgen, and
    related proc-macro crates in the main dependency tree.
- **Build-time cost assessment:** Both syn versions are proc-macro compile-time
  dependencies only. The duplication adds incremental build time (two separate
  compilations of syn) but has zero runtime footprint. The compiled binary contains
  only the output of the proc-macros, not syn itself.
- **Runtime risk:** NONE. No syn code executes at runtime. cargo audit reports 0
  advisories against either syn version in the current 1157-advisory database.
- **Fix:** No action warranted. Resolution is upstream's responsibility (tls-parser,
  pcap-file migrating to syn 2). Will resolve naturally when those crates publish
  updated releases.

---

## Prior-Cycle Findings: Regression Confirmation

| Prior Finding | Description | Status |
|---|---|---|
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (build-dep) | **CLEARED** — Cargo.lock shows rand 0.8.6; cargo audit exit 0, 0 advisories |
| RUSTSEC-2026-0190 | anyhow advisory (cleared earlier) | **CONFIRMED CLEAR** — anyhow 1.0.103 in Cargo.lock; 0 advisories in current scan |
| Zerocopy precautionary (0.8.48) | No active advisory; precautionary update recommended | **CLEARED** — Cargo.lock shows zerocopy 0.8.52 |

All three prior-cycle items are resolved or confirmed clean. No regression detected.

The `--ignore RUSTSEC-2026-0097` suppression in `.github/workflows/ci.yml` (if still
present) can be removed, as the advisory no longer triggers against rand 0.8.6.

---

## Compromised or Unmaintained Dependency Assessment

| Crate | Version | Type | Status | Evidence |
|---|---|---|---|---|
| `pcap-file` | 2.0.0 | Direct dep | **Maintained — no advisory** | cargo audit exit 0, 0 advisories; no unmaintained or yanked advisory in 1157-advisory DB |
| `tls-parser` | 0.12.2 | Direct dep | **Maintained — no advisory** | cargo audit exit 0, 0 advisories; no unmaintained advisory in DB; nom-based rusticata project remains active |
| `rand` | 0.8.6 | Transitive (build) | **Maintained; advisory resolved** | RUSTSEC-2026-0097 no longer triggered; 0.8.6 is the latest 0.8.x; 0.9.x branch also present in Cargo.lock |
| `zerocopy` | 0.8.52 | Transitive | **Maintained; actively patched** | Prior soundness history; current scan clean at .52 |
| `syn` | 1.0.109 + 2.0.117 | Transitive (proc-macro) | **Maintained (dtolnay)** | No advisory; v1 is LTS-supported |
| `anyhow` | 1.0.103 | Direct dep | **Maintained** | 0 advisories; RUSTSEC-2026-0190 confirmed not re-triggered |

No compromised dependencies detected. No supply-chain injection indicators. All sources
are crates.io (`sources: ok` per cargo-deny). No git or local-path dependencies.

---

## Scan Coverage Gaps (Tooling Observations)

These are tooling gaps, not vulnerabilities or findings.

- **cargo-outdated not installed:** Patch/minor version currency of the dependency tree
  was not assessed this sweep. The prior cycle used `cargo update --dry-run` as a
  substitute; that was not repeated here. For a production crate, periodic `cargo update`
  to absorb patch fixes is prudent maintenance. The prior cycle's recommended updates
  (rand 0.8.6, zerocopy 0.8.52) are confirmed landed; other updates from that batch
  (e.g., syn 2.0.117 — not bumped to 2.0.118) may still be available but were not
  assessed.
- **semgrep not installed:** Static SAST coverage for source-code patterns (hardcoded
  secrets, injection, unsafe block auditing) was not performed. For a Rust CLI tool with
  no network-facing server surface, the gap is limited in practice. `cargo clippy
  --all-targets -D warnings` (enforced in CI) covers a significant subset of Rust-specific
  lint patterns. Recommend installing for the next formal hardening phase.

---

## Summary Table

| ID | Finding | Severity | CWE | Fix Available | Blocking? |
|---|---|---|---|---|---|
| FINDING-001 | 8 license-not-encountered warnings in deny.toml | LOW | N/A | Optional prune | NO |
| FINDING-002 | syn v1 (1.0.109) / v2 (2.0.117) duplicate — build-time only | LOW | N/A | Upstream (tls-parser / pcap-file) | NO |

**No CRITICAL, HIGH, or MEDIUM findings. Overall verdict: CLEAN.**
