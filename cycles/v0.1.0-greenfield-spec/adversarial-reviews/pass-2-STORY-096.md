# Adversarial Review — STORY-096 (Implementation) — Pass 2

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, facade mode) |
| Scope | STORY-096 — `tests/cli_story_096_tests.rs` + traceability to BC-2.13.001/002/003/004, STORY-096.md |
| Strategy under review | brownfield-formalization (zero src changes) — tests prove ABSENCE of removed flags |
| Artifacts read | STORY-096.md; BC-2.13.001/002/003/004; tests/cli_story_096_tests.rs; src/cli.rs; src/findings.rs; Cargo.toml; src/analyzer/*; dispatcher.rs; lib.rs; main.rs; policies.yaml |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 2 |
| Date | 2026-05-31 |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree HEAD | abc4b4b (base develop c2445dc) |
| Verdict | **MEDIUM** (1 MEDIUM mutation-resistance gap; incomplete prior fix; 0 Critical/High) |

## Checkout Guard (DF-ADVERSARY-CHECKOUT-GUARD-001)

- Branch assertion: `git branch --show-current` = `feature/STORY-096-absent-flag-rejection` — OK (not develop).
- Worktree-base attestation: reviewing worktree at HEAD abc4b4b, base develop c2445dc (`c2445dc test(cli): formalize output-format + reassembly-config flag parsing (STORY-087)`).
- Grep-count assertion: `grep -c '#[test]'` = 14 test functions (10 AC + 4 EC) — matches story. OK.
- Factory artifacts read from main-repo path `/Users/.../wirerust/.factory/...`. OK.

## Supplied / Self-Run Evidence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001)

| Axis | Result |
|------|--------|
| `cargo test --test cli_story_096_tests` | 14 passed; 0 failed |
| `cargo clippy --test cli_story_096_tests -- -D warnings` | clean |
| `cargo fmt --check` | clean (exit 0) |
| Live mutation A — `pcap = "2.2"` (inline) | test **FAILED** — caught (pass-1 fix works) |
| Live mutation B — `[dependencies.pcap]` table header | test **FAILED** — caught |
| Live mutation C — `pcap = { version = "2.2" }` inline-table | test **FAILED** — caught |
| Live mutation D — `pcap="2.2"` (no space) | test **FAILED** — caught |
| Live mutation E — `pub threats: bool` into Cli struct | test **FAILED** — caught (AC-002) |
| Live mutation F — `pub verbose: bool` into Cli struct | test **FAILED** — caught (AC-009) |
| Live mutation G — `struct BeaconAnalyzer` into analyzer | test **FAILED** — caught (AC-004) |
| **Live mutation H — `bpf = "0.1"` standalone crate** | test **PASSED** (false-pass) — see F-W24-S096-P2-001 |
| **Live mutation I — `bpf-sys = "0.5"` BPF-binding crate** | test **PASSED** (false-pass) — see F-W24-S096-P2-001 |
| **Live mutation K — `pcap.version = "2.2"` dotted key in `[dependencies]`** | test **PASSED** (false-pass) — see F-W24-S096-P2-001; `cargo metadata` confirms it resolves the REAL `pcap` crate |

(Mutation J — dotted `pcap.version` under a second `[dependencies]` header — discarded as an invalid Cargo.toml duplicate-key error, not a valid reintroduction vector.)

## Method

Fresh-context attack. Re-derived the four BC invariants from BC-2.13.001..004 without
reading the pass-1 review's conclusions. The critical facade quality gate is
mutation-resistance of the structural-absence tests. I enumerated dependency-key
syntaxes that declare the BPF-capable `pcap` crate (which BC-2.13.003 Refactoring
Notes name as the reintroduction vector) and the standalone `bpf`/`bpf-sys` crates,
then ran each as a LIVE mutation against the actual worktree Cargo.toml. I also
re-ran the structural mutations for AC-002/004/009 to confirm they remain
mutation-resistant.

## Prior-Fix Propagation Audit (S-7.01)

The pass-1 fix added a line-by-line `has_pcap_dep` predicate to
`test_bpf_filter_absent_from_src` (tests/cli_story_096_tests.rs:293-300) matching:
`pcap =`, `pcap=`, `[dependencies.pcap]`, `[dependencies.pcap.`. Live mutations A–D
confirm those four forms are now caught — the primary F-W24-S096-P1-001 vector
(inline `pcap = "2.2"`) is resolved. **However the fix is incomplete**: it does not
cover the idiomatic TOML *dotted-key* dependency form, and the sibling `"\"bpf\""`
predicate (unchanged since pass 1) still misses unquoted BPF crate keys. See
F-W24-S096-P2-001.

## Critical Findings

None.

## Important Findings

### F-W24-S096-P2-001 [MEDIUM, confidence HIGH] — AC-006 still false-passes on (a) the dotted-key `pcap` dependency form and (b) standalone `bpf`/`bpf-sys` crate keys

**Test:** `test_bpf_filter_absent_from_src` (tests/cli_story_096_tests.rs:260-316)
**BC:** BC-2.13.003 invariant 2 ("No BPF expression evaluation exists in `src/`") +
Refactoring Notes (the `pcap` crate's filter API is the named reintroduction vector).

**Defect (a) — dotted-key `pcap` evasion (same crate as F-W24-S096-P1-001):**
The `has_pcap_dep` predicate matches `pcap =` / `pcap=` / `[dependencies.pcap]` /
`[dependencies.pcap.` but NOT the dotted form:
```toml
[dependencies]
pcap.version = "2.2"     # valid TOML; declares the real `pcap` crate
```
Live-verified (mutation K): inserting `pcap.version = "2.2"` into the existing
`[dependencies]` table leaves the test PASSING, and `cargo metadata` confirms the
dependency resolves to the **real `pcap` crate** (`"name":"pcap"`). The dotted form
is idiomatic Cargo syntax (commonly used with `pcap.optional`, `pcap.features`), so
this is a realistic reintroduction vector that the pass-1 fix did not close. This is
the SAME invariant the pass-1 fix targeted — the fix enumerated dependency-key
syntaxes but omitted the dotted form, leaving the BC-named vector partially
reachable.

**Defect (b) — unquoted `bpf` / `bpf-sys` crate keys evade:**
The predicate `!cargo_toml.contains("\"bpf\"")` requires the literal quoted string
`"bpf"`. A standalone BPF crate is declared as `bpf = "0.1"` — the left side is the
unquoted key `bpf` and the right side is the version string `"0.1"`, so the substring
`"bpf"` (quote-b-p-f-quote) never appears. Live-verified: mutation H (`bpf = "0.1"`)
and mutation I (`bpf-sys = "0.5"`) both leave the test PASSING. The `"\"bpf\""`
predicate as written only catches a `bpf` token that appears *inside* a quoted string
(e.g., a feature named `"bpf"`), not a `bpf`-keyed dependency.

**Why MEDIUM (not HIGH):** the canonical `pcap` inline form named by the BC IS now
caught (pass-1 fix), and reintroducing real BPF filtering would also require wiring +
a `--filter` flag that AC-005/EC-003 catch at parse time. But the facade quality gate
for this story is mutation-resistance, and three live reintroduction vectors for the
absence invariant currently false-pass — including a second syntax for the exact
`pcap` crate the BC names. Blast radius = 1 test, but it is the gate test.

**Suggested fix (test-writer):** strengthen `has_pcap_dep` to also match the dotted
form, and replace the `"\"bpf\""` substring check with a dependency-KEY check
symmetric to the `pcap` logic. Concretely, evaluate each trimmed Cargo.toml line for a
forbidden dependency key:
```rust
let forbidden_key = |line: &str, crate_name: &str| {
    // inline:  `name =`, `name=`, dotted `name.` ; table: `[dependencies.name]` / `.`
    line.starts_with(&format!("{crate_name} ="))
        || line.starts_with(&format!("{crate_name}="))
        || line.starts_with(&format!("{crate_name}."))
        || line.starts_with(&format!("[dependencies.{crate_name}]"))
        || line.starts_with(&format!("[dependencies.{crate_name}."))
};
```
Apply to `pcap`, `bpf`, `bpf-sys`, `libpcap`, `pcap-filter` (keeping `pcap-file` as
the permitted sanity-guard crate — note `pcap-file =` must NOT be matched by the
`pcap` key, which the `pcap =`/`pcap.`/`pcap=` anchoring already guarantees since
`pcap-file` starts with `pcap-`, not `pcap ` / `pcap.` / `pcap=`). Verify the fix
against live mutations H, I, K before re-running Pass 3.

**Sibling-sweep note (DF-SIBLING-SWEEP-001):** the dotted-key and unquoted-key gaps
are the same pattern across all four forbidden-crate predicates in this one test. The
fix MUST handle all forbidden crate names uniformly in a single burst (not just
`pcap`), or the next adversary pass will find the residual `libpcap.version` /
`pcap-filter.version` dotted evasions.

## Observations

- **O-1 (non-finding):** AC-002/004/009 structural mutations (E/F/G) all caught live —
  `pub threats`, `pub verbose`, `struct BeaconAnalyzer` reintroductions fail the tests.
  Mutation-resistant for their primary vectors. (Residual name-pinned limits for
  differently-named analyzer structs remain an inherent, accepted property of
  name-pinned absence tests — reintroduction would also require a `--beacon` flag that
  AC-003 catches.)
- **O-2 (non-finding):** LESSON-P1.04 comment (src/cli.rs:24-35) names all four removed
  flags as text; live mutation E confirms it does not false-positive the AC-002
  field-declaration predicates. findings.rs:96 "beaconing" doc-comment does not match
  the `struct BeaconAnalyzer` predicate (AC-004). False-pass protection holds.
- **O-3 (non-finding):** All 10 AC `**Test:**` citations resolve 1:1 to existing
  `fn test_*` (DF-AC-TEST-NAME-SYNC-001 PASS). All 14 tests inside `mod story_096`
  (DF-TEST-NAMESPACE-001 PASS).
- **O-4 (non-finding):** Story FSR (line 173) cites `tests/cli_tests.rs`; actual file is
  dedicated `tests/cli_story_096_tests.rs` per DF-TEST-NAMESPACE-001. File header
  documents the deviation. Cosmetic, non-blocking (matches STORY-086/087 disposition).

## Policy Rubric Compliance

| Policy | Verdict |
|--------|---------|
| DF-AC-TEST-NAME-SYNC-001 (MEDIUM) | PASS — 10/10 AC citations resolve to exactly one fn |
| DF-TEST-NAMESPACE-001 (MEDIUM) | PASS — all 14 tests wrapped in `mod story_096` |
| DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH) | satisfied (guard block above) |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM) | satisfied (cargo + 10 live mutations self-run) |
| DF-SIBLING-SWEEP-001 (CRITICAL) | flagged for the fix — all 4 forbidden-crate predicates share the dotted/unquoted-key gap; fix all uniformly |
| DF-VALIDATION-001 (HIGH) | N/A — test-only finding, fixed in place, not a deferred GitHub-issue finding |

## Novelty Assessment

Novelty: **HIGH** — F-W24-S096-P2-001 is a genuinely new finding (dotted-key + unquoted
`bpf`-key evasions), not a reword of F-W24-S096-P1-001. It is the same INVARIANT class
(BPF-crate absence) but distinct, live-verified syntactic vectors the pass-1 fix did
not cover. Per Trajectory Monotonicity, this signals the pass-1 fix was incomplete
(enumerated syntaxes rather than parsing dependency keys structurally). The structural
AC-002/004/009 tests are confirmed mutation-resistant.

## Verdict

**NOT CONVERGED after Pass 2.** One MEDIUM mutation-resistance gap
(F-W24-S096-P2-001): the AC-006 BPF-absence test still false-passes on (a) the
idiomatic dotted-key `pcap.version` form of the exact crate BC-2.13.003 names, and
(b) standalone `bpf`/`bpf-sys` crate keys. Fix uniformly across all forbidden-crate
predicates, re-verify against live mutations H/I/K, then re-run Pass 3. Minimum 3
consecutive clean passes required.

## Post-Pass-2 Remediation (orchestrator, same cycle) — FIXED

Root-cause fix applied to `test_bpf_filter_absent_from_src` (not a symptom patch):
replaced the enumerated literal-syntax checks with a single structural
`declares_dep(crate_name)` dependency-KEY matcher applied uniformly to every
forbidden crate `["pcap", "pcap-filter", "bpf", "bpf-sys", "libpcap"]`
(DF-SIBLING-SWEEP-001 — all forbidden keys fixed in one burst). The matcher detects a
key across ALL TOML dependency syntaxes: inline (`name =` / `name=`), dotted
(`name.`), and table headers (`dependencies.name]` / `dependencies.name.`, including
`[build-dependencies.…]` / `[dev-dependencies.…]`). A positive sanity guard asserts
`declares_dep("pcap-file")` is true (proving the matcher recognizes real keys), and
the `pcap`-anchored prefixes are never a prefix of `pcap-file`, so `pcap-file` is
never mis-flagged as `pcap`.

Verification evidence:
- 14/14 tests pass; clippy `-D warnings` clean; `cargo fmt --check` clean.
- Live cargo mutations with resolvable versions (`pcap = "2.2"` inline, inline-table,
  no-space, and `pcap.version = "2.2"` dotted) → all test FAILED (caught).
- Standalone predicate unit-check over 19 cases (5 forbidden crates × inline / dotted /
  table / nested-header / build-dep forms + `pcap-file` negative controls) → 0
  failures: every forbidden form matched, every `pcap-file`-vs-`pcap` negative control
  correctly NOT matched, `pcap-file` self-match true.
- Only `tests/cli_story_096_tests.rs` modified — zero src changes (facade discipline
  preserved).

F-W24-S096-P2-001 disposition: **RESOLVED**. Re-run from Pass 3 for clean-pass
counting (3 consecutive clean required).
