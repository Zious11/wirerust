# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Rust 2024 edition (requires Rust 1.91+, enforced via `rust-version = "1.91"` in Cargo.toml). Single-crate project. Stable toolchain (no `rust-toolchain` pin — effective MSRV is whatever `dtolnay/rust-toolchain@stable` resolves to today in CI).

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
  - `release/<version>` for gitflow release branches (e.g. `release/0.2.0`)
  - `hotfix/<slug>` for urgent production fixes branched from `main`
- **Semantic PR titles enforced via CI** (`amannn/action-semantic-pull-request`). Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`. Scope is optional. Release PRs into `main` use an allowed type, e.g. `chore: release v0.2.0`.
- No local commit hooks (no lefthook/husky/commitlint config) — enforcement is CI-side only.

### Releasing to `main`

`main` is the release/stable branch. It is updated **only** through gitflow-proper merges — never by direct commits, direct merge pushes, or admin bypass.

**Normal release flow:**

1. Cut a `release/<version>` branch from `develop` (e.g. `release/0.2.0`).
2. Apply any release-only fixups (version bump, changelog) on that branch.
3. Open a Pull Request targeting `main`; merge after CI is green.

**Hotfix flow** (urgent production fix):

1. Cut a `hotfix/<slug>` branch from `main`.
2. Apply the fix; open a Pull Request targeting `main`; merge after CI is green.

**Tagging:**

- Release tags (`v<version>`, e.g. `v0.1.0`) are created on `main` **only after** the release or hotfix PR has merged — never before, and never on a direct push.

**Keeping branches in sync:**

- After a release or hotfix PR merges into `main`, ensure `develop` contains those commits. Merge `main` back into `develop` if needed so the two branches do not diverge.

## Public API Surface (W7.1 — deferred)

`cargo public-api` is the intended tool for tracking public API surface changes
(drift item W7.1). It requires a nightly toolchain (rustdoc JSON output) and a
committed `public-api.txt` baseline to diff against. Adding a reliably-green
gating CI job requires two steps: (1) generate and commit an initial baseline
on nightly, (2) add a `cargo public-api diff` step that compares future runs
against it. This two-step setup was deferred from the W11/W16 drift-hardening
pass to avoid introducing a flaky or non-gating stub. To implement: install
`cargo-public-api`, run `cargo +nightly public-api > public-api.txt`, commit the
baseline, then add a CI step that fails on unexpected surface changes.

## Input Hash Computation

Every factory story file (`.factory/stories/STORY-NNN.md`) carries an `input-hash:` YAML
frontmatter field. This hash detects when a story's source inputs (behavioral contracts, PRD)
have changed without the story being regenerated — i.e., spec drift.

### Canonical Algorithm

The **canonical implementation** is `bin/compute-input-hash` (Python 3, no third-party deps).
This tool defines the algorithm; there is no separate spec document.

1. Parse the story's YAML frontmatter and extract the `inputs:` list **in declaration order**.
2. For each path in `inputs:` (relative to repo root), read the raw file bytes.
3. Normalize line endings: `\r\n` → `\n`, then lone `\r` → `\n`.
4. Concatenate all normalized byte strings in declaration order — **no separators, no paths,
   contents only**.
5. Compute the MD5 digest of the concatenated bytes.
6. The `input-hash` is the **first 7 hex characters** of the hexdigest (lowercase).

Design rationale:
- MD5 via Python stdlib `hashlib` — fast, no dependencies, adequate for drift detection
  (this is not a security hash).
- First-7 chars matches git short-SHA convention.
- LF normalization makes the hash OS-independent (Windows CRLF checkout does not change it).
- Contents-only concatenation avoids false drift on file renames.

### Tool Usage

```bash
# Print the 7-char hash for one story
bin/compute-input-hash .factory/stories/STORY-001.md

# Scan all stories: compare stored vs computed, print MATCH/STALE table, exit 1 if any STALE
bin/compute-input-hash --scan

# Scan and rewrite all stale hashes in place (factory-artifacts branch step)
bin/compute-input-hash --write --scan

# Rewrite one story's hash
bin/compute-input-hash --write .factory/stories/STORY-001.md
```

### Repo Root Resolution

The tool resolves the repo root by searching for the directory that contains `.factory/`.
Override with `WIRERUST_REPO_ROOT=/path/to/repo` if auto-detection fails (rare).

### CI Gate Decision (deferred — no broken stub shipped)

`.factory/` (stories and BC inputs) lives on the `factory-artifacts` branch, **not** in the
`develop` tree. A `develop` CI job cannot see those files without a `git fetch` of the
factory-artifacts ref.

Following the same principle as the W7.1 public-API note above (no flaky/non-gating stubs),
the input-hash drift check is **not** wired into the develop CI pipeline. Instead:

- **Phase-4 entry (manual gate):** Before opening a Phase-4 holdout-evaluation run, execute
  `bin/compute-input-hash --scan` from a checkout where `.factory/` is mounted
  (i.e., from the repo root, which has the factory-artifacts worktree at `.factory/`).
  If any stories report STALE, re-generate them before proceeding.
- To add a reliable CI gate in the future: add a step that fetches the `factory-artifacts`
  ref and checks out `.factory/`, then runs `bin/compute-input-hash --scan`. Only do this
  when it can be made reliably green and non-flaky.

### Self-Test

```bash
python3 bin/test_compute_input_hash.py
```

Verifies: determinism, pinned known-fixture hash, CRLF/LF normalization equivalence,
lone-CR normalization, declaration-order sensitivity, and clear error on missing input.

## Deferred Findings

Deferred or open findings — STATE.md Drift Items, spec contradictions, and review/adversarial backlog items — MUST be validated by the research agent (`vsdd-factory:research-agent`) before being filed as GitHub issues. No issue is created from an unvalidated finding. The canonical, machine-enforced version of this rule is policy `DF-VALIDATION-001` in `.factory/policies.yaml`.

## Project References

| Path | Purpose |
|------|---------|
| `README.md` | Project overview |
| `docs/adr/` | Architecture Decision Records (0001 stream dispatch, 0002 modular analyzers, 0003 reporting pipeline, 0004 process-wide warning atomics) |
| `docs/superpowers/plans/` | Implementation plans (from the superpowers skill) |
| `docs/superpowers/specs/` | Specifications (from the superpowers skill) |
| `.github/workflows/ci.yml` | CI pipeline (test, clippy, fmt, semantic PR) |
| `.factory/` | VSDD factory artifacts (logs only; STATE.md not yet initialized) |
