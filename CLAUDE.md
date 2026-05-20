# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Rust 2024 edition (requires Rust 1.85+, stabilized 2025-02-20). Single-crate project. Stable toolchain (no `rust-toolchain` pin — effective MSRV is whatever `dtolnay/rust-toolchain@stable` resolves to today in CI).

## Build & Test

```bash
# Build
cargo check                 # fast type-check loop
cargo build                 # debug
cargo build --release       # release profile has overflow-checks = true

# Run the CLI
cargo run -- --help

# Test — matches CI
cargo test --all-targets
cargo test <name_substring> # single test by substring match
# Note: --all-targets does NOT run doctests. Add `cargo test --doc` if any exist.

# Lint — matches CI (warnings are errors)
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt                   # apply (single-crate; --all is a workspace flag, harmless here)
cargo fmt --check           # CI gate
```

CI sets `RUSTFLAGS=-Dwarnings`. `rustfmt.toml` pins edition 2024, `max_width = 100`, field-init and try shorthand.

## Git Workflow

- **Default branch is `develop`** (git-flow). Branch from `develop`; PRs target `develop`. `main` exists but is the release/stable branch.
- **Branch naming** (observed):
  - `feature/<name>` for plain features
  - `worktree-issue-<n>-<slug>` for issue-scoped worktree branches
  - `worktree-<slug>` for ad-hoc worktree branches
  - `<type>/<slug>` — a semantic-PR-aligned pattern (e.g. `fix/sni-bounds`,
    `docs/adr-cleanup`), where `<type>` is one of the allowed
    semantic-PR types listed below. Equivalent to `feature/<name>` but
    generalized beyond `feat`.
- **Semantic PR titles enforced via CI** (`amannn/action-semantic-pull-request`). Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`. Scope is optional.
- No local commit hooks (no lefthook/husky/commitlint config) — enforcement is CI-side only.

## Deferred Findings

Deferred or open findings — STATE.md Drift Items, spec contradictions, and review/adversarial backlog items — MUST be validated by the research agent (`vsdd-factory:research-agent`) before being filed as GitHub issues. No issue is created from an unvalidated finding. The canonical, machine-enforced version of this rule is policy `DF-VALIDATION-001` in `.factory/policies.yaml`.

## Project References

| Path | Purpose |
|------|---------|
| `README.md` | Project overview |
| `docs/adr/` | Architecture Decision Records (0001 stream dispatch, 0002 modular analyzers, 0003 reporting pipeline) |
| `docs/superpowers/plans/` | Implementation plans (from the superpowers skill) |
| `docs/superpowers/specs/` | Specifications (from the superpowers skill) |
| `.github/workflows/ci.yml` | CI pipeline (test, clippy, fmt, semantic PR) |
| `.factory/` | VSDD factory artifacts (logs only; STATE.md not yet initialized) |
