---
document_type: cycle-lessons
cycle_id: fix-tls-clienthello-frag
version: "1.0"
status: draft
producer: state-manager
timestamp: 2026-07-01T00:00:00Z
---

# Lessons Learned — fix-tls-clienthello-frag

## Process-Gap Dispositions (S-7.02 cycle-close)

### [codified] PG-MUTANTS-JOBS-001 — cargo-mutants high --jobs masks survivors as load-induced timeouts

**Category:** Mutation-testing methodology  
**Cycle phase:** F6 targeted hardening  
**Disposition:** CODIFIED → STORY-147 (draft, E-11, 3 pts)

**Root cause:** During F6, `cargo mutants --jobs 8` was used to validate mutation coverage on
the TLS reassembly suite. The run reported "0 missed mutants", which appeared clean. However,
two real surviving mutants at tls.rs:950:59 and tls.rs:1030:67 were hidden: infinite-loop
mutants pegged all 8 cores, inflating other mutants' wall-clock past the auto-timeout threshold
and producing false timeouts instead of real coverage signals. Only a subsequent `--jobs 1`
re-run surfaced the actual survivors. Thirteen real mutation gaps were then closed by
`mod f6_hardening`; two provably-equivalent survivors were documented and retained.

**Lesson:** On suites containing infinite-loop mutants (common in carry-buffer reassembly code),
high parallelism causes innocent mutants to timeout alongside the infinite-loop ones, producing a
false "0 missed" result. Safe practice: run `cargo mutants` at `--jobs 1` (or set a generous
`--timeout`) as the first pass. High `--jobs` may be used for speed-scouting only if a clean
`--jobs 1` baseline is confirmed first.

**Codification:** STORY-147 will add a `mutants.toml` at the repo root setting `jobs = 1` as
the default and a CLAUDE.md "Mutation testing" note explaining why. This makes the safe
invocation the path of least resistance for all future contributors.

**References:** STATE.md D-314 (2026-07-01); `.factory/cycles/fix-tls-clienthello-frag/burst-log.md` F6 narrative.

---

### [deferral] PG-BC-ANCHOR-VALIDATION-001 — no automated line-anchor validation; drift recurs each cycle

**Category:** Tooling / spec governance  
**Cycle phase:** F5 scoped adversarial (recurred) + F7 drift item  
**Disposition:** JUSTIFIED DEFERRAL — target: next maintenance sweep

**Root cause:** BC files use line-number anchors (e.g., `tls.rs:124`) that drift whenever tls.rs
grows. This was observed again during fix-tls-clienthello-frag as the file added ~300 lines. The
adversarial passes caught several stale anchors (re-anchored 7 BCs in F5). There is currently no
automated check to flag this between cycles.

**Why deferred:** Fixing this properly requires either (a) a CI/maintenance-sweep script that
resolves each anchor against the current source and fails on stale line numbers, or (b) a policy
switch to symbol-only anchors (function/struct name) which never go stale. Both are non-trivial
tooling changes. Neither is blocking the security fix shipping in this cycle. Grouped with the
related open item BC-ANCHOR-DRIFT-OUTOFCYCLE-001 (STATE.md) for the next maintenance sweep.

**Target action:** Next maintenance sweep — scope: automated symbol-line anchor resolver or
policy revision to symbol-only-anchor form across all BC files.

---

### [deferral] DF-KANI-NONVACUITY-001-PROPTEST-GAP — no proptest/unit analog for the Kani non-vacuity policy

**Category:** Formal verification / policy coverage  
**Cycle phase:** F6 Kani harness authoring  
**Disposition:** JUSTIFIED DEFERRAL — target: next Kani VP authoring session

**Root cause:** The factory policy DF-KANI-NONVACUITY-001 in policies.yaml requires non-vacuity
confirmation for Kani harnesses. This was satisfied for VP-039 (manually confirmed, three
harnesses PASS with non-vacuous witnesses). However, no proptest or unit test encodes the
non-vacuity requirement as an executable regression guard — so a future accidental harness
regression would not be caught below the Kani-run threshold.

**Why deferred:** Severity is LOW — non-vacuity was manually confirmed and documented this cycle.
Adding a proptest analog requires authoring a test that constructs a counterexample witness
independently; this is non-trivial for Kani-specific semantics and is best done alongside the
next VP authoring pass (when the harness is fresh in context). No current gap in coverage exists.

**Target action:** Next Kani VP authoring session — add a proptest or unit analog that asserts
non-vacuity of the harness (e.g., by verifying the concrete witness property holds for at least
one valid input).

---

## Methodology Note — F6 Mutation-Testing Protocol Lesson

The high-`--jobs` false-clean trap (PG-MUTANTS-JOBS-001 above) establishes the following
protocol for future F6 phases on this codebase:

1. **First pass always at `--jobs 1`** (or `mutants.toml` default once STORY-147 ships).
   This is the ground-truth pass. Accept a longer wall-clock time in exchange for correct signals.
2. If the first pass is clean, a high-`--jobs` confirmatory pass is optional and advisory only.
3. If a high-`--jobs` pass shows more survivors than `--jobs 1`, treat the `--jobs 1` result as
   authoritative. The delta is load-induced noise.
4. Document any provably-equivalent survivors with an explanatory comment in the test module
   (pattern: `// SURVIVOR-EQUIV: <reason>` alongside the nearest test assertion).

This protocol was applied successfully in fix-tls-clienthello-frag F6, closing 13 real mutation
gaps and retaining 2 documented provably-equivalent survivors.
