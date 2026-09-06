# Dependency Audit — SCAN Stage Summary

**Run:** maint-2026-09-05
**Stage:** SCAN only (raw tool output capture — NOT a severity analysis; that is the
security-reviewer's responsibility for the follow-on findings pass)
**Raw output:** `.factory/maintenance/dependency-audit-raw.log`

## Environment

- `rustc 1.98.1 (48a229cea 2026-09-01)`
- `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- Repo: single-crate Rust project, edition 2024, `main`/`develop` gitflow (ran from `develop`, clean tree)

## Tools Run

| Tool | Version | Installed via | Exit code | Result |
|------|---------|----------------|-----------|--------|
| `cargo audit` | cargo-audit-audit 0.22.1 | pre-existing (already on PATH — no install needed) | 0 | 0 advisories reported. Scanned 175 crate dependencies against 1239 loaded RustSec advisories. |
| `cargo deny check` | cargo-deny 0.19.6 | pre-existing (already on PATH — no install needed) | 0 | `advisories ok, bans ok, licenses ok, sources ok`. 1 warning (duplicate crate versions — see below). 0 errors. |
| `cargo update --dry-run` | (via cargo 1.98.1) | n/a | 0 | 54 packages report newer semver/lockfile-compatible versions available. Dry run only — `Cargo.lock` was NOT modified (confirmed via `git status`). |

Both `cargo-audit` and `cargo-deny` were already installed on this machine at session start; the
`cargo install cargo-audit cargo-deny --locked` step was not needed and was not run.

## Counts

- **cargo audit:** 0 vulnerability advisories reported (175 dependencies scanned, 1239 advisories loaded from `~/.cargo/advisory-db`).
- **cargo deny check:** 1 warning, 0 errors.
  - Warning type: `warning[duplicate]` — 2 duplicate lockfile entries for crate `syn` (v1.0.109 and v2.0.117), pulled in transitively via `derive-into-owned`/`pcap-file`/`nom-derive` (syn 1.x) vs. `clap_derive`/`serde_derive`/`thiserror-impl`/`wasm-bindgen-macro-support`/`zerocopy-derive` (syn 2.x). This is a `bans` lint (duplicate-version), not an advisory/license/source finding.
  - `advisories ok`, `bans ok` (warning-level only, not blocking), `licenses ok`, `sources ok`.
- **cargo update --dry-run:** 54 packages with available updates (within the existing `Cargo.lock` semver constraints — e.g. `syn v2.0.117 -> v2.0.119`/`v3.0.5`, `regex v1.12.3 -> v1.13.1`, `wasm-bindgen v0.2.117 -> v0.2.128`, `serde v1.0.228 -> v1.0.229`, etc.). 1 additional dependency reported "unchanged behind latest" (visible only with `--verbose`, not re-run for this scan). No major-version-gated updates surfaced by this dry run (dry run only shows lockfile-compatible bumps).

## Tools That Could Not Run

None. Both `cargo-audit` and `cargo-deny` were available and ran to completion; `cargo update --dry-run` also ran to completion.

## CRITICAL/HIGH Advisory Strings

Searched the full raw log (`grep -iE "critical|high|vulnerabilit|CVE-|RUSTSEC"`) — no CRITICAL or
HIGH severity advisory strings, no CVE-/RUSTSEC- IDs, and no reported vulnerabilities appeared.
The only matches were the routine startup/banner lines from `cargo audit` itself (fetching the
advisory database, "Scanning Cargo.lock for vulnerabilities (175 crate dependencies)") — these are
tool-invocation text, not findings.

## Notes for Follow-On (security-reviewer)

- The one `cargo deny` finding (duplicate `syn` versions) is a supply-chain hygiene/bloat signal,
  not a security advisory — flagging for the security-reviewer's severity classification pass per
  this sweep's contract (SCAN vs. severity-analysis separation).
- `Cargo.toml` and `Cargo.lock` were not modified by this sweep (verified via `git status --short`).
- No `git commit`/`push` performed — per instructions, state-manager owns committing factory artifacts.
