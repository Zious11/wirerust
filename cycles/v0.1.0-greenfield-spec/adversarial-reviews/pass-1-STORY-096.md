# Adversarial Review — STORY-096 (Implementation) — Pass 1

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, facade mode) |
| Scope | STORY-096 — `tests/cli_story_096_tests.rs` + traceability to BC-2.13.001/002/003/004, STORY-096.md |
| Strategy under review | brownfield-formalization (zero src changes) — tests prove ABSENCE of removed flags |
| Artifacts read | STORY-096.md; BC-2.13.001/002/003/004; tests/cli_story_096_tests.rs; src/cli.rs; src/findings.rs; Cargo.toml; src/analyzer/*; dispatcher.rs; lib.rs; main.rs; policies.yaml |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 1 |
| Date | 2026-05-31 |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree HEAD | 553bbb2 |
| Base develop | c2445dc |
| Verdict | **MEDIUM** (1 MEDIUM mutation-resistance gap; 0 Critical/High) |

## Checkout Guard (DF-ADVERSARY-CHECKOUT-GUARD-001)

- Branch assertion: `git branch --show-current` = `feature/STORY-096-absent-flag-rejection` — OK (not develop).
- Grep-count assertion: `grep -c '#[test]'` = 14 test functions = 10 AC + 4 EC — matches story. OK.
- Factory artifacts read from main-repo path `/Users/.../wirerust/.factory/...`. OK.

## Supplied / Self-Run Evidence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001)

| Axis | Result |
|------|--------|
| `cargo test --test cli_story_096_tests` | 14 passed; 0 failed |
| `cargo clippy --test cli_story_096_tests -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| Live mutation AC-002 (insert `pub threats` into Cli struct) | test **FAILED** as required — mutation-resistant |
| Live mutation AC-006 (add BPF-capable `pcap` crate to Cargo.toml) | test **PASSED** (false-pass) — see F-W24-S096-P1-001 |

## Method

Fresh-context attack focused on the CRITICAL axis named in the dispatch:
mutation-resistance of the structural-absence tests (AC-002/004/006/009). For
each, I (a) enumerated realistic reintroduction vectors, (b) checked them
against the test's `.contains()` predicates statically, and (c) ran live
mutations against the actual worktree source for the two highest-value cases.
I also verified the LESSON-P1.04 comment (which names all four flags) does not
cause false passes.

## False-Pass Protection (LESSON-P1.04 comment) — VERIFIED CLEAN

`src/cli.rs:24-35` LESSON-P1.04 comment names `--verbose`, `--threats`,
`--beacon`, `--filter` as removed; `src/findings.rs:96` contains "beaconing".
All 14 tests pass against the live source, and the live AC-002 mutation proves
the comment does NOT create a false positive: the comment writes `--threats`
(leading dash, no `pub`, no indented `threats:`), so the field-declaration
predicates do not match it. **This design goal is met.**

## Mutation-Resistance Findings

### Structural tests that ARE mutation-resistant (confirmed)

| AC | Test | Realistic reintroduction | Caught? |
|----|------|--------------------------|---------|
| AC-002 | `test_threats_field_absent_from_cli` | `pub threats: bool` struct field / 8-space-indented `threats:` enum field | YES (live-verified FAIL) |
| AC-004 | `test_beacon_analyzer_absent_from_src` | `struct BeaconAnalyzer` / `C2BeaconAnalyzer` / `impl …` | YES for these exact names |
| AC-009 | `test_verbose_field_absent_from_cli` | `#[arg(short, long)] pub verbose: bool` | YES via `pub verbose` predicate |

### F-W24-S096-P1-001 [MEDIUM] — AC-006 misses the BPF-capable `pcap` crate (false-pass)

**Test:** `test_bpf_filter_absent_from_src`
**Predicates:** asserts absence of `"pcap-filter"`, `"\"bpf\""`, `"libpcap"`;
asserts presence of `"pcap-file"`.

**Defect:** The canonical Rust crate that provides BPF filtering is named
`pcap` (its `Capture::filter()` compiles and applies a BPF expression).
Adding `pcap = "2.2"` to Cargo.toml satisfies none of the three forbidden
substrings — `pcap-filter`, `"bpf"`, `libpcap` are all absent from the string
`pcap = "2.2"`. **Live-verified:** with `pcap = "2.2"` added, the test still
PASSES. This is the most likely BPF reintroduction vector, and BC-2.13.003's
own Refactoring Notes name it explicitly: *"BPF filtering would require
integration with a BPF library (e.g., pcap crate's filter API)."* The test
therefore fails to be mutation-resistant against the exact reintroduction the
BC anticipates.

Secondary evasions in the same predicate set (static-verified, lower priority):
`bpf-sys` / `rust-bpf` style crate names also evade (no quoted `"bpf"`),
though `nix` with a `"bpf"` feature would be caught incidentally.

**Suggested fix (test-writer):** add an assertion that no Cargo.toml dependency
line begins with `pcap ` / `pcap =` / `pcap = {` (the BPF-capable `pcap` crate),
distinct from the permitted `pcap-file`. E.g. assert the source contains no
regex-equivalent `^pcap\b(?!-file)` dependency key. A simple robust form:
assert `!cargo_toml.lines().any(|l| l.trim_start().starts_with("pcap ") || l.trim_start().starts_with("pcap="))`.
Keep the existing `pcap-file` sanity guard.

## Non-Findings / Observations

- **O-1 (non-finding):** AC-004 evades on differently-named analyzer structs
  (`BeaconDetector`, `C2Analyzer`, bare `struct Beacon`) and on a new file
  `src/analyzer/beacon.rs` that is not in the `include_str!` set. This is an
  inherent limit of name-pinned structural absence tests and is consistent with
  the story's stated scope (the BC invariant names `C2BeaconAnalyzer` "or
  equivalent struct"). Acceptable for a facade absence proof; noted as residual
  weakness, not a blocking finding, because reintroducing beacon detection would
  also require wiring + a `--beacon` flag that AC-003 would catch.
- **O-2 (non-finding):** AC-009 `short = 'v'` literal check would not catch a
  derived `#[arg(short, long)]` short flag, but the sibling `pub verbose`
  predicate catches that same reintroduction, so the invariant is preserved.
- **O-3 (non-finding):** EC-001..004 are described in the story EC table but not
  cited via AC `**Test:**` lines; DF-AC-TEST-NAME-SYNC-001 applies to AC
  citations only. All 10 AC citations resolve 1:1 to existing fns — SYNC OK.
- **O-4 (non-finding):** Story FSR says modify `tests/cli_tests.rs`, but tests
  landed in a dedicated `tests/cli_story_096_tests.rs` per DF-TEST-NAMESPACE-001.
  The file header documents this deviation; consistent with policy. Story FSR row
  is slightly stale but non-blocking (cosmetic).

## Policy Rubric Compliance

| Policy | Verdict |
|--------|---------|
| DF-AC-TEST-NAME-SYNC-001 | PASS — 10/10 AC citations resolve to exactly one fn |
| DF-TEST-NAMESPACE-001 | PASS — all tests wrapped in `mod story_096` |
| DF-SIBLING-SWEEP-001 | N/A this pass (no remediation yet) |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | satisfied (see guard block) |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | satisfied (cargo evidence self-run) |

## Verdict

**MEDIUM** — one mutation-resistance gap (F-W24-S096-P1-001, AC-006 false-pass
on the BPF-capable `pcap` crate). The other three structural absence tests
(AC-002/004/009) are mutation-resistant for their primary reintroduction
vectors, and the LESSON-P1.04 false-pass concern is empirically cleared. Fix
F-W24-S096-P1-001, then re-run for Pass 2.
