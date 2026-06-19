# Review Findings — F7-R2 CLI Hardening

**PR:** #273
**Branch:** fix/f7-r2-cli-hardening
**Date:** 2026-06-19

## Convergence Table

| Cycle | Reviewer | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|----------|-------|-----------|---------|
| 1 | pr-manager (self-verify) | 0 | 0 | 0 | 0 | APPROVE |

## Self-Verification Results (Cycle 1)

### (a) Provenance Sweep Completeness — PASS
```
grep -n '///' src/cli.rs | grep -E 'BC-[0-9]|STORY-[0-9]|LESSON-'
# (empty — no violations)
```

### (b) #[non_exhaustive] + Constructor — PASS
- `#[non_exhaustive]` applied at terminal.rs:121 (Grouping), 129 (Collapse), 142 (FindingsRender)
- `FindingsRender::new` constructor at terminal.rs:154
- Zero actual code-level struct-literal construction sites in tests/ — only `//` comments and string literals reference the old form
- `cargo check --all-targets` compiles clean (definitive proof)
- 91 `FindingsRender::new` call sites (2 in main.rs, rest in tests/)

### (c) CI Gate Scope — PASS
- Gate scans `src/cli.rs` only (not over-broad)
- Regex: `^\s*///.*(BC-[0-9]|STORY-[0-9]|LESSON-)` — correctly targets `///` lines only, not `//`
- SHA-pinned checkout: `df4cb1c069e1874edd31b4311f1884172cec0e10 # v6.0.3`
- Action-pin-gate confirms SHA ref passes supply-chain check

### Security Surface — CLEAR
- No runtime code logic changes
- No network/auth/injection surface affected
- CI grep job uses `set -euo pipefail`, no untrusted inputs
- No OWASP Top 10 applicability

## CI Results (PR #273, run 27846044553)

| Check | Result | Time |
|-------|--------|------|
| Action pin gate | PASS | 4s |
| Audit | PASS | 10s |
| Clippy | PASS | 21s |
| Deny | PASS | 20s |
| Format | PASS | 7s |
| Fuzz build | PASS | 1m18s |
| **Help-provenance gate** | **PASS** | **6s** |
| Semantic PR | PASS | 3s |
| Test | PASS | 38s |
| Trust-boundary gate | PASS | 4s |

All 10 checks green. New gate `help-provenance-gate` passes.

## Status
**READY FOR HUMAN MERGE GATE** — awaiting explicit merge authorization.
