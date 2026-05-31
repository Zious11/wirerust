# Adversarial Review Index — STORY-096 (Implementation)

| Field | Value |
|-------|-------|
| Target | STORY-096 — Absent-behavior contracts: removed flags rejected by clap |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree | .worktrees/STORY-096 |
| Worktree HEAD | 553bbb2 (base develop c2445dc) |
| Strategy | brownfield-formalization (zero src changes); facade mode (no Red Gate; mutation-resistance is the quality gate) |
| Cycle | v0.1.0-greenfield-spec |
| Wave | 24 |
| BCs | BC-2.13.001, BC-2.13.002, BC-2.13.003, BC-2.13.004 |
| VP | none |
| Test file | tests/cli_story_096_tests.rs (14 tests: AC-001..010 + EC-001..004) |
| Status | **CONVERGED** — 3 consecutive clean passes (4,5,6) after fixing 3 MEDIUM mutation-resistance gaps (passes 1,2,3) |

## Pass Summary

| Pass | Attack vector | Findings | Max severity | File |
|------|---------------|----------|--------------|------|
| 1 | Mutation-resistance of structural-absence tests (AC-002/004/006/009) + LESSON-P1.04 false-pass + toolchain pairing + live mutations | 1 MEDIUM | MEDIUM | pass-1-STORY-096.md |
| 2 | AC-006 BPF dependency-key predicate (dotted-key + unquoted `bpf` evasions) | 1 MEDIUM | MEDIUM | pass-2-STORY-096.md |
| 3 | AC-004 `include_str!` file-set coverage (struct in unscanned `src/` file) + clap-rejection correctness + field-absence joint resistance | 1 MEDIUM | MEDIUM | pass-3-STORY-096.md |
| 4 | Prior-fix re-verify (S-7.01) + target-cfg dependency tables + type-alias + brittleness | 0 | — (clean) | pass-4-STORY-096.md |
| 5 | Prior-fix re-verify + flag-rejection mutation-resistance (`--filter`/`--beacon`) + positive-parse (AC-010/EC-004) compile-coupling | 0 | — (clean) | pass-5-STORY-096.md |
| 6 | Prior-fix re-verify + `[patch.crates-io]` pcap + tab-indent field + EC-002 dns-validity | 0 | — (clean) | pass-6-STORY-096.md |

Trajectory: 1 MED → 1 MED → 1 MED → CLEAN → CLEAN → CLEAN. Monotonic decrease to zero.
3 consecutive clean passes (4,5,6) → convergence minimum MET.

## Findings Register

| ID | Sev | Summary | Blocking | Disposition |
|----|-----|---------|----------|-------------|
| F-W24-S096-P1-001 | MEDIUM | AC-006 false-passes when the BPF-capable `pcap` crate (inline form) is added to Cargo.toml — predicates only blocked `pcap-filter`/`"bpf"`/`libpcap`. Live-verified false-pass. | Yes | **RESOLVED** (pass-1 fix: line-by-line `pcap`-key match for inline/table forms) — superseded by P2-001 structural matcher |
| F-W24-S096-P2-001 | MEDIUM | AC-006 STILL false-passed on (a) the idiomatic dotted-key `pcap.version = "..."` form of the same `pcap` crate and (b) standalone unquoted `bpf`/`bpf-sys` dependency keys. Live-verified (`cargo metadata` confirms dotted form resolves the real `pcap` crate). Pass-1 fix was incomplete (enumerated syntaxes, not structural). | Yes | **RESOLVED** (pass-2 fix: structural `declares_dep` key matcher over `pcap`/`pcap-filter`/`bpf`/`bpf-sys`/`libpcap` across inline/dotted/table syntaxes; 19-case unit check + live mutations verified; pcap-file positive sanity guard) |
| F-W24-S096-P3-001 | MEDIUM | AC-004 only scanned 7 hand-listed `src/` files; `C2BeaconAnalyzer` reintroduced in any of the other 15 files (e.g. `src/summary.rs`) or a NEW file (`src/analyzer/beacon.rs`) false-passed, violating BC-2.13.002 invariant 2 (scope = all of `src/`) silently. Live-verified (2 vectors). | Yes | **RESOLVED** (pass-3 fix: runtime recursive walk over all `src/**/*.rs` via `CARGO_MANIFEST_DIR`, with a `>=20`-file positive-coverage guard; 4 live vectors verified caught) |
| (O) | — | AC-004 inert `type BeaconAnalyzer = DnsAnalyzer;` alias evades struct/impl predicates (pass 4) | No | Non-finding — alias declares no struct + no detection logic; accepted name-pinned-absence limit |
| (O) | — | Story FSR cites `tests/cli_tests.rs`; actual file is dedicated `tests/cli_story_096_tests.rs` per DF-TEST-NAMESPACE-001 | No | Non-finding — namespace policy supersedes FSR row (matches STORY-086/087 disposition) |

## Policy Compliance (verification steps executed)

| Policy | Result |
|--------|--------|
| DF-AC-TEST-NAME-SYNC-001 (MEDIUM) | PASS — all 10 AC `**Test:**` citations resolve to exactly one `fn test_*` |
| DF-TEST-NAMESPACE-001 (MEDIUM) | PASS — all 14 tests wrapped in `mod story_096`; zero flat-namespace functions |
| DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH) | satisfied — branch + worktree-base + grep-count assertions in every pass guard block |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM) | satisfied — cargo test/clippy/fmt + 25+ live mutations + ErrorKind probe + standalone predicate unit checks self-run across passes |
| DF-SIBLING-SWEEP-001 (CRITICAL) | satisfied — pass-2 fix applied uniformly to ALL 5 forbidden-crate keys in one burst; pass-3 fix covers ALL `src/` files |
| DF-VALIDATION-001 (HIGH) | N/A in scope (test-only; all 3 findings fixed in place, none filed as deferred GitHub issues) |

## Build/Test Evidence (final, post-convergence)

- `cargo test --test cli_story_096_tests` → 14 passed; 0 failed (all 6 passes)
- `cargo clippy --test cli_story_096_tests -- -D warnings` → clean
- `cargo fmt --check` → clean (exit 0)
- `git diff --stat` → only `tests/cli_story_096_tests.rs` modified (zero src changes — facade discipline intact across all fixes)
- Mutation-resistance verified live: `pub threats`/`pub verbose` field reintro (top-level + `Commands::Analyze`, 4-space/8-space/tab indent) → caught; `struct C2BeaconAnalyzer` in scanned + unscanned + new + deeply-nested `src/` files → caught; `pcap` crate inline/no-space/inline-table/dotted/table-header/target-cfg/patch-table → caught; `bpf`/`bpf-sys`/`libpcap`/`pcap-filter` keys → caught (19-case predicate unit check); `--filter`/`--beacon`/`--threats`/`--verbose`/`-v` reintroduced-as-valid → rejection tests fail; `--http` removal → compile error (caught)
- ErrorKind probe: every rejection test fails with `UnknownArgument` naming the SPECIFIC flag; valid invocations parse OK; missing-target yields distinct `MissingRequiredArgument` (tests are discriminating, not accidentally green)

## Verdict

**CONVERGED.** Trajectory 1 MED → 1 MED → 1 MED → CLEAN → CLEAN → CLEAN (monotonic
decrease to zero; passes 4/5/6 are the 3 consecutive clean passes). Three MEDIUM
mutation-resistance gaps found and fixed at root cause:
- **P1/P2-001 (AC-006):** the BPF-absence predicate was narrower than BC-2.13.003's
  "absent everywhere" scope — fixed with a structural `declares_dep` dependency-key
  matcher over all 5 forbidden crate names across every TOML syntax.
- **P3-001 (AC-004):** the beacon-struct-absence test scanned 7 of 24 `src/` files —
  fixed with a runtime full-`src/`-tree walk plus a positive-coverage guard.

Both fixes verified mutation-resistant under repeated fresh-context re-attack (S-7.01
propagation re-checked every pass). The story's 14 tests are mutation-resistant on all
substantive axes; the only residue is the inherent name-pinned-absence limit (inert
type alias), consistent with the facade strategy. Ready for delivery.
