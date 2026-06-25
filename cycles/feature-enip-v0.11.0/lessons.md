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
