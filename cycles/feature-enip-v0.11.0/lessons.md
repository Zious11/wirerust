# Lessons Learned — feature-enip-v0.11.0

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

---

## [process-gap] PROPAGATION-LAG-001 — Mechanical changed-value sibling-grep gate

**Status:** OPEN — needs follow-up story or human-approved deferral before cycle CLOSE.
**Engine-improvement candidate:** ENGINE-PROPAGATION-GREP-GATE-001

**Observation:**

Across F2 adversary Passes 1–3, the recurring failure mode was single-value
spec changes (BE→LE endianness, write-burst default 20→50, variant count
16→15, ForwardClose title correction, BC-2.17.025 addition) failing to
propagate to the FULL sibling set:

- BC body (the directly edited file)
- BC-INDEX row (count + version)
- BC-INDEX changelog prose
- PRD §2.17 table entry
- verification-architecture.md
- verification-coverage-matrix.md
- VP-INDEX descriptive entries
- capability file (cap-17-*.md)
- sibling CLI BCs (e.g., BC-2.17.020 mirroring BC-2.17.012 threshold)

5 of 10 Pass-3 adversary findings were propagation-lag of already-correct
values, not new defects in the core content. Pass-3 also discovered 3 bonus
BE residues (VP-INDEX:119, verification-architecture:110, cap-17:20) that the
adversary itself missed — found only by orchestrator exhaustive grep.

**Pattern:** Each pass finds the same class of defect at a different layer of
the sibling set. Adversary novelty HIGH every pass because new sibling slots
keep surfacing.

**Root cause:** No mechanical enforcement that "when value V in artifact A
changes, all artifacts that contain V are updated atomically."

**Recommendation:**

Add a mechanical pre-convergence "changed-value grep gate" — for each LOCKED
value-change recorded in a BC-INDEX changelog entry or ADR decision row, grep
all sibling artifacts for the OLD value (analogous to
`bin/compute-input-hash --scan` drift detection). The gate should:

1. Extract (old_value, new_value) pairs from the cycle's fix-directives or
   BC-INDEX changelog.
2. For each old_value, run a corpus-wide grep over the full sibling set.
3. Fail (block commit) if any old_value survives in a non-historical context.
4. Emit a remediation list: file:line for every surviving instance.

This closes the propagation-lag class as a mechanical gate rather than relying
on adversary sampling or orchestrator manual grep.

**Analogous tool already in repo:** `bin/compute-input-hash --scan` detects
when story source inputs have changed without the story being regenerated.
Same principle: detect stale values before they reach the gate.

**Next action:** ENGINE-PROPAGATION-GREP-GATE-001 is in STATE.md OPEN ITEMS.
Human decision: (a) create a follow-up story for engine tooling, (b) document
as a factory policy and handle procedurally, or (c) defer with explicit
rationale. Must be resolved before cycle CLOSE (S-7.02).

---

## [codified] WARN-LOG-CRATE-001 — Spec/ADR asserted warn!/log-crate convention that does not exist

**Status:** CODIFIED — in-place fix applied; no follow-up story required (re-evaluate at cycle close per S-7.02).
**Found at:** STORY-131 adversarial Pass-3 MEDIUM finding M-1.
**Decision:** D-236.

**Observation:**

ADR-010 Decision 9 (original) stated: "emit a WARNING" for the `--enip` reassembly-guard
path. STORY-131.md Library & Framework Requirements (original) asserted: "log ≥ 0.4 (already
present): `warn!(...)`". STORY-138.md Library & Framework Requirements (original) stated:
"`log::warn!` for MAX_FINDINGS warning — already in project."

None of these claims were true. The wirerust project has no `log` crate dependency
(confirmed by `Cargo.toml`). All existing analyzer reassembly-guard warnings use `eprintln!`
to stderr (Modbus and DNP3 guards in `src/main.rs`).

**Impact:** Had an implementer followed the original spec, they would have introduced a
phantom `log` crate dependency (compile error) or attempted to `use log::warn` and discovered
the missing import at first `cargo check`.

**Root cause:** Spec and story authors asserted a library/framework claim ("log crate already
present") without verifying against `Cargo.toml`. The claim was plausible (many Rust projects
use the log crate) but false for this project.

**Fix applied (D-236 burst):**
- ADR-010 Decision 9: "emit a WARNING" → "emit via `eprintln!` to stderr ... no `log` crate
  dependency" (root source, M-1 root fix).
- STORY-131.md Library & Framework Requirements: removed `log ≥ 0.4` claim; replaced with
  "No `log` crate (project has no `log` dependency): emit via `eprintln!`...".
- STORY-138.md Library & Framework Requirements: removed `log::warn!` claim; replaced with
  "No `log` crate (project has no `log` dependency): emit via `eprintln!`...".

**Process improvement:**

Spec and story authors MUST verify library/framework claims against `Cargo.toml` (and
`Cargo.lock`) before asserting "already present". The canonical check:

```bash
grep -E 'log|tracing|env_logger' Cargo.toml
```

If the crate is absent from `Cargo.toml`, it MUST NOT be cited as "already present".
In-place fix + sweep is sufficient when the error is caught pre-implementation (as here).
A follow-up story would only be needed if the false claim had been implemented and shipped.

**Analogous prior lesson:** PROPAGATION-LAG-001 — same class (spec assertion not grounded
in the actual codebase). The common thread: spec/story authoring operates on assumptions
rather than verified codebase facts. The mechanical fix for dependency claims is a one-line
`Cargo.toml` grep, not an adversarial pass.

---

## [codified] [process-gap] GREEN-DOC-TENSE-TEST-HEADER-001 — DF-GREEN-DOC-TENSE sweep misses the test-module header doc-comment block

**Status:** CODIFIED — justified deferral; self-improvement story scope to be set at cycle close alongside STORY-091/STORY-121 wave assignment (S-7.02). Tracked in STATE.md OPEN ITEMS as GREEN-DOC-TENSE-TEST-HEADER-STORY.
**Found at:** STORY-132 adversarial Pass 1, HIGH finding H-1 (DF-GREEN-DOC-TENSE violation in test module header).
**Prior recurrences:** STORY-130 Pass 1 (F-130-01, HIGH), STORY-131 Pass 1 (H1, HIGH) — 3 consecutive stories in feature-enip-v0.11.0, each caught only at adversarial Pass 1 (HIGH), then fixed.
**Decision:** D-239.

**Observation:**

The DF-GREEN-DOC-TENSE-SWEEP policy (v2, HIGH) requires that all doc-comments in changed
files use present-indicative ("Returns", "Parses", "Emits") rather than future/aspirational
prose ("will return", "should parse", "TODO", "RED GATE", "MUST FAIL"). The implementer
and test-writer GREEN step applies this sweep to src/ function doc-comments and file-level
module comments. However, in all three STORY-130/131/132 deliveries, the per-story
**test-module header block** — the `//!` doc-comment directly above `mod <story_name>` in
the test file, and the `// STORY-NNN — <title>` section header comment — was written (or
left from stub templates) in future-tense or imperative language and was NOT caught during
the GREEN sweep. Each instance was a HIGH finding at adversarial Pass 1.

**Impact:** Recurrence at HIGH severity forces a remediation burst between Pass 1 and Pass
2, extending convergence by one pass (3 passes needed; effectively pushes Pass 2/3/4 to
Pass 2/3/4 after the fix). Three consecutive occurrences (STORY-130/131/132) signals a
systematic gap, not a one-off oversight.

**Root cause:**

The DF-GREEN-DOC-TENSE sweep checklist (and any GREEN-step reminder) focuses on:

1. `src/` function doc-comments (`///` and `//!` at module level in src files)
2. File-level module doc-comments in test files

It does NOT explicitly call out the per-story **test-module header block** as a distinct
sweep target:

- The inner `mod <story_name>` block inside `tests/enip_analyzer_tests.rs` (or a similar
  file) carries a `//!` doc-comment describing the test module.
- The outer `// ===== STORY-NNN =====` or `/// STORY-NNN — <title>` section header comment
  above it is the entry point.
- Stub-architect templates may emit these in future-tense by convention ("These tests will
  verify…", "STORY-NNN will implement…"); if the implementer does not sweep them during
  GREEN, the future-tense language ships into the PR.

**Fix applied (each recurrence):**

In-place: replace future-tense language in the test-module header with present-indicative
("These tests verify…", "STORY-NNN implements…"). Swept alongside the `src/` doc-comment
pass after each fix.

**Recommended follow-up:**

1. **Immediate (in-procedure):** Update the DF-GREEN-DOC-TENSE sweep GREEN step (wherever
   it appears in story templates and the implementer's checklist) to explicitly list:
   - The `mod <story_name>` inner `//!` doc-comment block.
   - Any `// ===== STORY-NNN =====` or `/// STORY-NNN —` section-header comments at the
     top of the test-module scope.
   as mandatory sweep targets, distinct from the file-level module doc and the `src/`
   function docs.

2. **Mechanical gate (self-improvement story, tracked as GREEN-DOC-TENSE-TEST-HEADER-STORY):**
   Add a pre-PR grep gate over changed test files for known future-tense markers:
   ```bash
   grep -rn '\bwill\b\|\bshould\b\|RED GATE\|MUST FAIL\|todo!()\|will implement\|will verify' \
     tests/ --include="*.rs"
   ```
   Any match in a changed test file's module-header block should fail the GREEN gate and
   require immediate tense correction. Alternatively, stub-architect templates can emit
   GREEN-ready headers by convention (present-indicative from birth), which removes the
   sweep obligation entirely.

3. **Stub-architect convention:** If stub templates always emit present-indicative headers
   (`//! Verifies...`, `// STORY-NNN — ...`), the GREEN sweep need not revisit them. This
   is the zero-friction fix: fix the source (templates), not the detector.

**Analogous prior lesson:** WARN-LOG-CRATE-001 — same class (authoring operates on an
assumption rather than a verified fact). Here the "assumption" is that the stub template
output is already GREEN-compliant. The mechanical fix is either a template change or a grep
gate, not an adversarial pass.

---

## [codified] [process-gap] MITRE-CATALOG-ADR-AUTHORITATIVE-001 — ADR is authoritative for MITRE catalog (id, name, tactic) tuples; story prose is not

**Status:** CODIFIED — justified deferral; evaluate at cycle close whether to add a name-correctness pin to `mitre_tests.rs` (analogous follow-up to the tactic-id pin already in place). No separate follow-up story opened at this time.
**Found at:** STORY-133 adversarial Pass 1 — 2 CRITICAL + 2 HIGH.
**Decision:** D-240.

**Observation:**

STORY-133 Pass 1 found 4 findings rooted in a single factual error: the story prose
carried the wrong MITRE catalog mapping for T1693.001.

- **Story prose (wrong):** `name: "Exploit Public-Facing Application: EtherNet/IP"`, `tactic: MitreTactic::IcsInitialAccess`
- **ADR-010 Decision 7 (authoritative):** `name: "Modify Firmware: System Firmware"`, `tactic: MitreTactic::IcsInhibitResponseFunction` (TA0107)

The implementation followed the story prose, producing:
1. `technique_info("T1693.001")` returning wrong name — CRITICAL (factually wrong catalog entry).
2. `technique_info("T1693.001")` returning wrong tactic — CRITICAL (wrong TA-id: IcsInitialAccess vs IcsInhibitResponseFunction/TA0107).
3. Test `test_technique_info_t1693_001` pinned the wrong name "Exploit Public-Facing Application: EtherNet/IP" — HIGH (test validating incorrect behavior).
4. No executable gate existed on the tactic assignment — HIGH (IcsInitialAccess variant added but correct variant IcsInhibitResponseFunction mandated by ADR not asserted).

**Impact:** Had the implementation shipped without adversarial pass, the MITRE catalog would
have contained a factually incorrect technique name and a wrong tactic assignment for T1693.001,
visible in all user-facing JSON output. A subsequent sibling sweep of STORY-INDEX.md,
dependency-graph.md, and BC-2.17.007 confirmed no other stories or BCs carried the defect
(they reference T1693.001 by ID only, not by name or tactic). The defect was isolated to
STORY-133.md. All 4 issues fixed at code commit `ffca717` (impl + test pin +
`mitre_tests.rs` authoritative-TA-id pin-table extension + stale-count function renames +
RED-tense scrub) and story prose fixed in this burst.

**Root cause:**

Story authors transcribed MITRE technique metadata (id, name, tactic) from memory or
secondary sources rather than cross-checking each new catalog entry against the project's
authoritative source: ADR-010 Decision 7 and the enip-architecture-delta research. The
correct mapping exists unambiguously in ADR-010 Decision 7; the story author did not read it
before writing the technique_info AC.

**Fix applied (D-240):**
- STORY-133.md lines ~68, 123, 147–148, 224: four T1693.001 prose references corrected to
  authoritative name "Modify Firmware: System Firmware" and tactic IcsInhibitResponseFunction (TA0107).
- Code @ffca717: `technique_info("T1693.001")` implementation, test pin, and
  `mitre_tests.rs` pin-table extended with T1693.001 → TA0107 authoritative assertion.
- VP-007 invariants intact post-fix: SEEDED 28, EMITTED 20, T1693.001 excluded from EMITTED,
  revoked T0855/T0856/T0857 absent.

**Process improvement (mandatory for all future MITRE-seeding stories):**

When a story introduces or updates a MITRE catalog entry (any `technique_info` arm,
any entry in `SEEDED`, any entry in `EMITTED_IDS`), the implementer MUST:

1. Cross-check the **(id, name, tactic)** tuple against **ADR-010 Decision 7** and the
   relevant research file — not against story prose alone.
2. Verify tactic assignment against the authoritative tactic-id map in the same ADR.
3. Extend the `mitre_tests.rs` authoritative-TA-id pin-table with a `(technique_id → TA-id)`
   assertion for every new technique introduced in lockstep (sibling-sweep obligation).

The `mitre_tests::test_ics_techniques_resolve_authoritative_tactic_ids` pin-table is now the
mechanical gate for tactic correctness. It must be extended for EVERY future MITRE-seeding
story — this is the executable equivalent of the "cross-check ADR" obligation.

**Recommended follow-up (justified deferral):**

Evaluate at cycle close (S-7.02) whether to add a **name-correctness pin** to `mitre_tests.rs`
alongside the existing tactic-id pin — i.e., `(technique_id → expected_name)` assertions.
This would catch the wrong-name CRITICAL at test time rather than requiring an adversarial
pass. Deferral is justified because: (a) the tactic-id pin (now extended) already catches the
most operationally significant error (wrong TA-id drives wrong tactic taxonomy); (b) name
correctness can be visually verified from the catalog at story-write time with the new
cross-check obligation; (c) the name-pin adds test maintenance cost for all future technique
renames in the upstream MITRE catalog. Resolve this evaluation at cycle close alongside
ENGINE-PROPAGATION-GREP-GATE-001 and GREEN-DOC-TENSE-TEST-HEADER-STORY.
