---
document_type: phase-7-convergence-report
cycle: v0.1.0-greenfield-spec
product: wirerust
mode: brownfield
develop_head: 0855f25
assessment_date: 2026-06-08
producer: architect
verdict: CONVERGED
traces_to: STATE.md
---

# Phase 7 — Convergence Report
## wirerust v0.1.0-greenfield-spec

**Overall Verdict: CONVERGED**

All 7 dimensions assessed. 6 PASS / 1 CONCERN. The single CONCERN (Performance) is
non-blocking: the bench harness exists, P0 NFRs are verified by code review + criterion
results, and the open item (NFR-PERF-004 autovectorization LLVM-IR confirmation) is a
documented P1 open-debt item with no release gating. No dimension is FAIL.

---

## Per-Dimension Assessment

| # | Dimension | Verdict | Summary |
|---|-----------|---------|---------|
| 1 | Spec Convergence | **PASS** | Adversary gate 3/3; consistency CONSISTENT (8/8 remediation items PASS); all 217 BCs traced to stories; 20 VPs fully indexed |
| 2 | Test Convergence | **PASS** | 1126 tests green / 0 failed / 0 skipped-of-concern; coverage spans all 48 delivered stories; no disabled test blocks |
| 3 | Implementation Convergence | **PASS** | Phase-5 adversary gate 3/3 (14 passes, 4 fix-PRs merged); clippy clean; fmt clean; develop HEAD stable at 0855f25 |
| 4 | Verification Convergence | **PASS** | 20 VPs locked; 8 Kani proven; 6 proptest; 1 fuzz (21.7M execs, 0 crashes); mutation kill-rate targets met all modules; security posture clean |
| 5 | Visual Convergence | **PASS** | 451 VHS demo files across 22 unique story directories; per-AC terminal recordings present for all key stories; appropriate standard for a CLI tool |
| 6 | Performance Convergence | **CONCERN** | P0 NFRs (PERF-001/003) verified by code review; criterion harness built and results present; no numeric perf budget gate recorded; NFR-PERF-004 SIMD autovectorization unverified (P1 open-debt) |
| 7 | Documentation Convergence | **PASS** | 4 ADRs (0001-0004) current; README reflects actual behavior (one known cosmetic debt); no CHANGELOG.md (release-prep item, not convergence blocker) |

---

## Detailed Evidence

### Dimension 1 — Spec Convergence: PASS

**Evidence:**
- Adversarial spec-convergence gate: 3/3 consecutive clean passes (passes 31/32/33 in Phase 1).
  Journey: 33 total adversarial passes, findings decayed from 17 (pass 1) to 0 (passes 31–33).
- Phase-5 whole-implementation adversarial convergence: 3/3 clean (passes 12/13/14 of 14 total).
- Fresh-context consistency re-audit (2026-06-08): CONSISTENT — 8/8 remediation items PASS.
  Two pre-documented LOW non-blocking findings retained:
  - LOW-1: nfr-story-map.md summary metric shows 38 but catalog has 40 stories (stale summary counter; underlying mapping correct)
  - LOW-2: STORY-INDEX 48-vs-49 prose (pre-existing/documented; STORY-091 is draft tooling story, not product)
- Input-hash drift: MATCH=48 / STALE=0 (STORY-091 `inputs:[]` ERROR is by-design; empty inputs hash d41d8cd is expected behavior)
- 217 BCs across 20 L2 domain-spec shards; all traced to stories and modules.
- 20 VPs fully catalogued in VP-INDEX.md (version 2.0, status: verified).
- All 3 source-of-truth invariants (ARCH-INDEX Subsystem Registry, BC H1 headings, VP-INDEX) verified consistent.

**Gaps:** None blocking. LOW-1 and LOW-2 are documented pre-existing/cosmetic.

---

### Dimension 2 — Test Convergence: PASS

**Evidence:**
- `cargo test --all-targets` on develop @ 0855f25: **1126 passed / 0 failed / 0 errors**.
- `cargo clippy --all-targets -- -D warnings`: clean (RUSTFLAGS=-Dwarnings enforced).
- `cargo fmt --check`: clean.
- Test corpus spans all 48 delivered stories and all 11 epics across 27 waves.
- Holdout evaluation (Phase 4): mean 0.949 across 80/100 scenarios; 0 must-pass below 0.6;
  one genuine defect (HS-043) found and fixed (PR #171/#172); Chunk 3 re-evaluation
  confirmed 0.9917 mean after evaluator-coverage artifacts resolved.
- No `#[ignore]` or skipped test blocks identified in any test module.
- STORY-091 (tooling; draft/deferred) has no test file and is excluded from coverage totals
  by design.
- All Phase-3 TDD wave gates passed (27/27); stories 48/48 delivered.

**Gaps:** None.

---

### Dimension 3 — Implementation Convergence: PASS

**Evidence:**
- Phase-5 adversarial refinement: 14 whole-implementation fresh-context passes;
  findings trajectory `MED→MED→HIGH+LOW→MED→ZERO→HIGH+MED→MED+LOW→HIGH→ZERO→MED+MED+LOW→MED+LOW→LOW→ZERO→ZERO`.
  Final state: ADVERSARY GATE 3/3 SATISFIED at pass 14.
- 4 fix-PRs merged during Phase 5: #173 (reassembly zero-validator), #174 (deterministic
  output ordering), #175 (HS-043 test-doc coherence), plus earlier #171/#172 (HS-043 fix).
- develop HEAD 0855f25 = origin/develop (working tree clean, no open PRs).
- `cargo clippy --all-targets -- -D warnings`: 0 warnings (CI gate enforced).
- `cargo fmt --check`: clean.
- All 4 ADRs (0001 stream dispatch, 0002 modular analyzers, 0003 reporting pipeline,
  0004 process-wide warning atomics) are in-force and respected in implementation.
- Rust 2024 edition; stable toolchain; overflow-checks=true in release profile.

**Gaps:** None.

---

### Dimension 4 — Verification Convergence: PASS

**Evidence:**

**Formal Proofs (Kani — 8 VPs):**
- VP-001 (FlowKey Canonical Ordering), VP-002 (First-Wins Overlap Policy — JUSTIFIED→PROVEN,
  PR #183, `select_gaps` extracted to pure function, 180 Kani checks SUCCESSFUL),
  VP-003 (MAX_FINDINGS Cap), VP-004 (Content-First Dispatch Precedence), VP-005 (SNI
  4-Way Ordered Classification), VP-007 (MITRE Technique Format + Catalog Completeness),
  VP-009 (FlowState Machine Validity), VP-015 (sequence arithmetic).
  All 8 status: `verified`, `verification_lock: true`, `proof_completed_date: 2026-06-02`.

**Property-Based Testing (proptest — 6 VPs):**
- VP-006 (HTTP Poison Monotonicity), VP-010 (buffered_bytes Invariant), VP-011
  (flush_contiguous Monotonicity), VP-012, VP-013, VP-014. All verified (PR #179).

**Fuzzing (cargo-fuzz — 1 VP):**
- VP-008 (`decode_packet` Never Panics): 21.7M executions, 0 crashes, 0 panics, 0 OOM.
  1813-seed corpus. Ran on CI-pinned nightly-2026-05-21. (PR #182)

**Mutation Testing:**
- SS-06 (http.rs): 100% kill rate (117 viable mutants).
- SS-07 (tls.rs): 100% kill rate (166 viable mutants); SNI CRITICAL sub-target met (100%).
- SS-08 (dns.rs): 100% kill rate (24 viable mutants).
- SS-09 (findings.rs): 100% kill rate (4 viable mutants).
- SS-10 (mitre.rs): 90% kill rate (40 viable mutants; target ≥90% HIGH).
- SS-04 (reassembly — flow.rs, mod.rs, segment.rs): 16 mutation survivors killed via PR #184;
  targets met across all reassembly modules after Phase-6 close.

**Test-Sufficient (5 VPs):**
- VP-016 through VP-020 (reporter properties, output format invariants): integration/unit
  test coverage; verified through Phase-3/5 holdout.

**Security:**
- RUSTSEC-2025-0119 (`number_prefix` unmaintained): FIXED by bumping indicatif 0.17→0.18
  (PR #185). `--ignore` entry removed.
- RUSTSEC-2026-0097 (`rand` 0.8.5 unsound via tls-parser): ACCEPTED-TRANSITIVE.
  Unreachable code path (build-time codegen, deterministic seed). `--ignore` retained.
  Revisit when tls-parser bumps phf to 0.12+.
- `cargo audit` (CI-equivalent, ignores applied): exit 0, clean.
- `cargo deny check bans licenses sources` (CI-equivalent): exit 0, all sections ok.
- No HIGH/CRITICAL CVE-severity advisories open.

**Gaps:** None blocking.

---

### Dimension 5 — Visual Convergence: PASS

**Note on scope:** wirerust is a CLI/library tool (no GUI). The appropriate visual
convergence standard is VHS terminal recordings demonstrating CLI behavior per AC,
not web UI screenshots or visual regression tests. This assessment applies that standard.

**Evidence:**
- **451 demo files (`.tape`/`.gif`/`.webm`) across 22 unique story-level directories**
  within `.factory/cycles/`. Files confirmed gitignored per D-005 (2026-05-21 directive:
  demos stay local, never committed to factory-artifacts).
- Demo coverage spans multiple cycles:
  - `v0.1.0-greenfield-spec/`: STORY-001 (10 ACs), STORY-002 (8+ ACs), STORY-003 (2+ ACs),
    STORY-004, STORY-069/070, wave-11 story-021, wave-12 story-031, wave-13 story-032,
    wave-14 story-033, wave-15 story-041/051 — all with `evidence-report.md` documenting
    per-AC pass results.
  - `phase-3-tdd/demos/`: story-016 (full BC-035/036/038/043/047 + proptest suite),
    story-017 (full BC-018..022/037 suite), story-018 (full BC-023..046 suite), story-013
    (mid-stream join demo).
  - `wave-7-story-014/`, `wave-8-story-015/`, `wave-8-story-019/`: per-story demo indices.
- **FIX-P5-002** (zero-depth/memcap rejection): 3 full AC recordings (tape + gif + webm)
  plus evidence-report.md in `.factory/demo-evidence/` (the formal Phase-5 fix evidence
  location).
- **STORY-057** (SNI edge cases): evidence-report.md with 13-AC per-AC pass table (903/903
  tests green), stored in `.factory/demo-evidence/STORY-057/`.

**Regarding the "sparse demo-evidence/" observation:** The `.factory/demo-evidence/`
directory has only 2 entries because demos are primarily stored under
`.factory/cycles/<story>/demos/` or `.factory/cycles/phase-3-tdd/demos/<story>/`.
The `demo-evidence/` path is used for formal fix-evidence during Phase-5 adversarial
remediation. The sparse count in `demo-evidence/` is NOT a visual-convergence gap — it
reflects the factory's actual storage topology. Total demo footprint is 451 files across
22 directories, which is substantial coverage for a CLI tool.

**Gaps:** Not all 48 stories have a recorded `.tape` file (approximately 22/48 story
directories have explicit tape/gif evidence). The majority of remaining stories have
`evidence-report.md` pass tables as their primary AC verification artifact, supplemented
by the 1126-test green suite. For a CLI library tool in a brownfield formalization cycle
(target == reference), test-pass evidence is the primary AC artifact; full-suite tape
coverage on every story is aspirational, not required for convergence.

---

### Dimension 6 — Performance Convergence: CONCERN

**Evidence:**

**P0 NFRs (VERIFIED):**
- NFR-PERF-001 (zero-copy packet decoding via `etherscape::SlicedPacket`): P0, verified by
  code review — `src/decoder.rs:288-291` confirms only `tcp.payload().to_vec()` / `udp.payload().to_vec()`; L2-L3 slice path is zero-allocation. Status: N/A (no open-debt).
- NFR-PERF-003 (content-first dispatch: O(1) per-flow re-classification via HashMap cache):
  P0, verified by code review — `routes: HashMap<FlowKey, DispatchTarget>` at
  `src/dispatcher.rs:43`; cache lookup in `on_data` at `src/dispatcher.rs:133-154`.
  Status: N/A (no open-debt).

**Criterion bench harness (EXISTS, results present):**
- Harness: `benches/pipeline.rs`, `[[bench]] name = "pipeline"` in Cargo.toml.
- Three benchmark groups: `decode`, `summary`, `reassembly`. Results stored in
  `target/criterion/`.
- Sample decode results (`segmented.pcap`): mean 1440 ns/iter (95% CI: 1424–1458 ns).
- Sample reassembly results (`segmented.pcap`): mean 4907 ns/iter (95% CI: 4876–4944 ns).
- No numeric perf budget is defined in the NFR catalog. The criterion harness exists to
  establish baselines and catch regressions, not to gate against a specific throughput SLA.

**Open NFR debt (P1 — non-blocking):**
- NFR-PERF-002 (single-pass eager load — RAM <= pcap_size * ~1.5): P1, OPEN-DEBT.
  README claim "multi-GB captures" overstates capability when RAM is constrained.
  Documented as NFR-VIO-001. No test validates the 1.5x multiplier claim. Streaming
  refactor is tracked as O-01 class debt (next cycle).
- NFR-PERF-004 (SIMD autovectorization confirmation via LLVM IR or `cargo asm`): P1,
  OPEN-DEBT. Bench harness exercises the hot reassembly path; overlap detection uses
  slice equality (`segment_data[..] != existing_data[..]`). Autovectorization is design
  intent (comment at `segment.rs:124`) but LLVM-IR confirmation has not been recorded.
  No criterion measurement is defined against an autovectorization budget.

**Gap:** No numeric perf-budget gate result is recorded (no "throughput must exceed X
Mpps" pass/fail). The two open P1 items (NFR-PERF-002/004) are documented, tracked, and
explicitly out-of-scope for v0.1.0. The criterion harness provides regression detection
baseline. The absence of a formal perf-budget gate result is a CONCERN but not a blocker:
no NFR defines a hard numeric SLA for v0.1.0.

**Remediation route (if human gate requires a stronger verdict):**
1. Run `cargo bench` once on the release SHA, record the criterion HTML summary path in
   this report — upgrades the evidence from "harness exists" to "baseline recorded."
2. Optionally annotate NFR-PERF-004 with a `cargo asm` snippet for `segment_data[..] != existing_data[..]` showing SIMD instructions — closes the autovectorization claim formally.
3. Update NFR-PERF-002 with an RSS measurement on a large fixture to bound the 1.5x claim.

These are scope-appropriate for release-prep or an early feature cycle, not a gate blocker.

---

### Dimension 7 — Documentation Convergence: PASS

**Evidence:**
- **4 ADRs present and current:**
  - ADR 0001 (`content-first-stream-dispatch.md`): Accepted; covers StreamDispatcher
    classification logic, per-flow caching. Accurately reflects `src/dispatcher.rs`.
  - ADR 0002 (`modular-protocol-analyzers.md`): Accepted; covers analyzer trait/plugin model.
  - ADR 0003 (`reporting-pipeline-layering.md`): Accepted; covers ADR-0003 pure-core
    reporter separation from effectful shell — consistent with purity boundary map.
  - ADR 0004 (`process-wide-warning-atomics.md`): Accepted; retroactive ADR codifying
    `static AtomicBool` one-shot tripwire pattern. Accurately reflects reassembly/mod.rs,
    lifecycle.rs, segment.rs.
- **README.md** reflects actual CLI behavior: correct `--help` flags, correct feature
  descriptions, accurate install instructions. All major features (PCAP ingestion,
  protocol analysis, TCP reassembly, multi-link-type, MITRE mapping, JSON output) documented.
- **One known doc debt (NFR-VIO-001):** README line "built for multi-GB captures" overstates
  capability given the eager-load design (NFR-PERF-002 violation). This is a pre-existing,
  catalogued violation. Cosmetic fix is a release-prep item.
- NFR catalog v1.3 (80 NFRs) is current and validated.
- nfr-story-map.md v1.1 authored; all P0 NFRs mapped to stories (Criterion-38 CLOSED).
- 7 architecture section files at status:verified, version:1.1.

**Missing artifacts (release-prep items, NOT convergence blockers):**
- `CHANGELOG.md`: Not present. No VSDD phase requires a CHANGELOG before convergence;
  this is a release-tooling artifact that belongs in the release phase after the human gate.
- `.factory/release-config.yaml`: Not present. Required for the release skill to generate
  tag annotations, GitHub Release notes, and crates.io publish parameters. Pre-release prep,
  not convergence criteria.

**Gap:** README "multi-GB captures" claim is cosmetic debt (NFR-VIO-001, documented). Both
missing artifacts (CHANGELOG, release-config.yaml) are release-prep items deferred
by design.

---

## Summary: Must-Fix-Before-Gate vs. Deferrable

### Must fix before human gate: NONE

There are no blocking items that require remediation before the human Phase-7 gate.
All convergence gates are met across all 7 dimensions (6 PASS, 1 CONCERN where the
concern is non-blocking P1 open-debt).

### Deferrable to release-prep (post-gate, before v0.1.0 tag)

| ID | Item | Dimension | Action |
|----|------|-----------|--------|
| R-1 | `CHANGELOG.md` missing | Documentation | Author CHANGELOG.md summarizing v0.1.0 features before cutting the release tag. Not a convergence gate. |
| R-2 | `.factory/release-config.yaml` missing | Documentation | Create release-config.yaml with tag name, release notes template, crates.io metadata. Required by release skill. |
| R-3 | README "multi-GB captures" claim (NFR-VIO-001) | Documentation/Performance | Update README to accurately state eager-load memory model ("requires RAM ~= pcap size") or qualify with "for typical captures." Pre-existing documented debt. |

### Deferred to next cycle (O-01 / P1 class)

| ID | Item | Dimension | Notes |
|----|------|-----------|-------|
| D-1 | NFR-PERF-004 SIMD autovectorization LLVM-IR confirmation | Performance | P1 open-debt; bench harness present; design intent documented at `segment.rs:124`. Next cycle or inter-phase. |
| D-2 | NFR-PERF-002 RAM bound validation (load test with large pcap) | Performance | P1 open-debt; streaming refactor is O-01 class debt. Next cycle. |
| D-3 | STORY-091 (anchor-validation tooling) | Spec | Draft P1 story; deferred next cycle per STATE.md backlog. |
| D-4 | RUSTSEC-2026-0097 revisit | Verification | Revisit when tls-parser bumps phf to 0.12+ (upstream-only fix). |
| D-5 | Phase-5 secondary-review tech-debt P3 items (CR-002/003/005/006/007/009/012) | Implementation | See tech-debt-register.md. Non-blocking for v0.1.0. |

---

## Overall Verdict

**CONVERGED.**

The wirerust v0.1.0-greenfield-spec cycle has satisfied all Phase-7 convergence
requirements across 7 dimensions. The product is ready for the human gate review.
Following human gate approval, release-prep items R-1 through R-3 should be completed
before cutting the v0.1.0 tag.

| Gate Criterion | Status |
|----------------|--------|
| Spec adversary gate (3/3 clean passes) | SATISFIED |
| Implementation adversary gate (3/3 clean passes) | SATISFIED |
| Consistency audit (8/8 PASS) | SATISFIED |
| Test suite (0 failures) | SATISFIED |
| Verification (20 VPs LOCKED) | SATISFIED |
| Mutation kill-rate targets (all modules meet tier) | SATISFIED |
| Security (no open HIGH/CRITICAL advisories) | SATISFIED |
| Visual/CLI demo evidence | SATISFIED |
| Performance P0 NFRs | SATISFIED (P1 open-debt documented) |
| Documentation (ADRs, README current) | SATISFIED (release artifacts deferred) |
