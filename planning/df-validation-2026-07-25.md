---
document_type: validation-report
level: ops
version: "1.0"
status: complete
producer: research-agent
date: 2026-07-25
timestamp: 2026-07-25T00:00:00Z
policy: DF-VALIDATION-001
cycle: "wave-084 + wave-085 process-gap batch"
inputs:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-084/process-gap-ledger.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/vsdd-factory-upstream-issues.md
input-hash: "[live-state]"
traces_to: STATE.md
---

# DF-VALIDATION-001 Validation Report — PG-W84 + PG-W85 Deferred Batch

Scope: 14 deferred process-gap findings (PG-W84-001/002/003/004/005/006/008/010/012 and
PG-W85-001..005). PG-W84-007, -009, -011 were FIXED in-cycle and are out of scope.

Upstream tracker state at validation time: `drbothen/vsdd-factory`, issues #126–#763
(live API queried 2026-07-25; the local snapshot at
`.factory/planning/vsdd-factory-upstream-issues.md` is dated 2026-07-19 / #687 and was
extended live for #688–#763 during this pass).

PG→lesson mapping was re-derived from the `Closes:` lines in
`.factory/cycles/wave-084/lessons.md` (lesson numbering is shuffled relative to PG IDs);
the mapping supplied in the dispatch matched the file exactly and is used as-is.

---

## 1. Summary Disposition Table

| Finding | Classification | Verdict | Covering issue(s) | Confidence |
|---------|---------------|---------|-------------------|------------|
| PG-W84-001 — stale inline version markers in story bodies | UPSTREAM | DUPLICATE | [#749](https://github.com/drbothen/vsdd-factory/issues/749) (primary); [#550](https://github.com/drbothen/vsdd-factory/issues/550), [#682](https://github.com/drbothen/vsdd-factory/issues/682) (sub-case) | MEDIUM |
| PG-W84-002 — sub-agent message-routing breakage → missing artifact + backfill commit | UPSTREAM | DUPLICATE | [#457](https://github.com/drbothen/vsdd-factory/issues/457) (primary); [#592](https://github.com/drbothen/vsdd-factory/issues/592), [#619](https://github.com/drbothen/vsdd-factory/issues/619), [#211](https://github.com/drbothen/vsdd-factory/issues/211) | HIGH |
| PG-W84-003 — burst-log Dim-1 file-count template understates ride-along files | UPSTREAM | DUPLICATE | [#681](https://github.com/drbothen/vsdd-factory/issues/681) | MEDIUM |
| PG-W84-004 — STATE.md write-path hook cascade lacks unified error reporting | UPSTREAM | DUPLICATE | [#572](https://github.com/drbothen/vsdd-factory/issues/572) (primary); [#616](https://github.com/drbothen/vsdd-factory/issues/616) | HIGH |
| PG-W84-005 — validate-pr-review-posted false-positive on self-authored PRs | UPSTREAM | DUPLICATE | [#651](https://github.com/drbothen/vsdd-factory/issues/651) (near-exact); [#626](https://github.com/drbothen/vsdd-factory/issues/626) | HIGH |
| PG-W84-006 — pr-manager-completion-guard pressures step-9 fabrication pre-merge | UPSTREAM | **NOVEL-UPSTREAM** | adjacent: [#707](https://github.com/drbothen/vsdd-factory/issues/707), [#673](https://github.com/drbothen/vsdd-factory/issues/673), [#756](https://github.com/drbothen/vsdd-factory/issues/756) | MEDIUM |
| PG-W84-008 — PR-description commit count composed before final fixup commit | UPSTREAM | DUPLICATE | [#663](https://github.com/drbothen/vsdd-factory/issues/663) | MEDIUM |
| PG-W84-010 — `bin/check-green-doc-tense` Rust-only scan blind spot for `bin/*.py` | LOCAL | LOCAL-CARRY-FORWARD | no upstream filing; class tracked upstream at [#339](https://github.com/drbothen/vsdd-factory/issues/339), [#599](https://github.com/drbothen/vsdd-factory/issues/599) | HIGH |
| PG-W84-012 — `bin-selftest` CI job absent from develop required-status-checks | LOCAL | LOCAL-CARRY-FORWARD | no upstream filing; analogues [#257](https://github.com/drbothen/vsdd-factory/issues/257), [#349](https://github.com/drbothen/vsdd-factory/issues/349) | MEDIUM |
| PG-W85-001 — holdout-evaluation report template has no structured caveat block | UPSTREAM | **NOVEL-UPSTREAM** | adjacent: [#458](https://github.com/drbothen/vsdd-factory/issues/458), [#617](https://github.com/drbothen/vsdd-factory/issues/617) | MEDIUM |
| PG-W85-002 — multi-document factory-artifact sibling-sweep discipline | UPSTREAM (class) / LOCAL (codification) | DUPLICATE | [#470](https://github.com/drbothen/vsdd-factory/issues/470), [#507](https://github.com/drbothen/vsdd-factory/issues/507) (+ [#504](https://github.com/drbothen/vsdd-factory/issues/504)/[#505](https://github.com/drbothen/vsdd-factory/issues/505)/[#506](https://github.com/drbothen/vsdd-factory/issues/506)), [#299](https://github.com/drbothen/vsdd-factory/issues/299), [#216](https://github.com/drbothen/vsdd-factory/issues/216) | HIGH |
| PG-W85-003 — green-doc-tense gate misses `Expected RED:` / `currently falls through` | LOCAL | LOCAL-CARRY-FORWARD | no upstream filing; class tracked upstream at [#682](https://github.com/drbothen/vsdd-factory/issues/682) | HIGH |
| PG-W85-004 — pr-manager attempted self-approval on its own PR | UPSTREAM | DUPLICATE | [#626](https://github.com/drbothen/vsdd-factory/issues/626) (primary); [#696](https://github.com/drbothen/vsdd-factory/issues/696), [#651](https://github.com/drbothen/vsdd-factory/issues/651) | HIGH |
| PG-W85-005 — gitignored machine-local e2e fixtures → false-green `cargo test` | LOCAL | LOCAL-CARRY-FORWARD | no upstream filing; adjacent class [#694](https://github.com/drbothen/vsdd-factory/issues/694) | HIGH |

**Rollup:** 8 DUPLICATE, 2 NOVEL-UPSTREAM, 4 LOCAL-CARRY-FORWARD, 0 ALREADY-FIXED,
0 INCONCLUSIVE.

**Upstream issues to file: 2** — PG-W84-006, PG-W85-001.
**Upstream issues NOT to file: 7** (comment on the covering issue instead where a
sub-case is additive: #749, #681, #663).
**Local wirerust items: 4** (PG-W84-010, PG-W84-012, PG-W85-003, PG-W85-005) — plus the
DF-SIBLING-SWEEP-001 policy extension for PG-W85-002.

---

## 2. Per-Finding Rationale

### PG-W84-001 — stale inline version markers recur; automated lint candidate

**Classification: UPSTREAM.** The proposed remedy is a lint over factory story artifacts,
which is engine-owned surface (story-writer agent + story template + hooks).

**Mechanism-accuracy caveat (raised by this validation).** The ledger and L-W84-004
describe the remedy as *"a lint check comparing `## Story v<N.M>` header vs frontmatter
`version:` field."* I could not substantiate that a `## Story v<N.M>` body heading exists
as a convention:

- `templates/story-template.md` (plugin rc.23) declares `version: "1.1"` in frontmatter
  only. Grepping its heading set (`^#`, `^## `) yields no version-bearing heading and no
  `## Changelog` section at all.
- `.factory/stories/STORY-166.md` — one of the two cited instances — contains no
  `## Story vN.M` heading. Its version-bearing body text is (a) prose provenance
  ("re-estimated from 5 at the v1.1 engine/project ..." at line 382), (b) a cross-artifact
  cite ("STORY-166 registration (v3.57)" at line 443), and (c) changelog table rows.

So the *class* is real and recurring (stale in-body version qualifiers surviving spec
evolution), but the *mechanism* named in the ledger is not the mechanism observed. The
real class is version-qualifier drift in prose, live-claim vs frozen-provenance.

**Duplicate check.** [#749](https://github.com/drbothen/vsdd-factory/issues/749)
("Version-qualifier drift in prose citations is invisible to input-hash drift detection;
no propagation on spec version bump") is the covering issue. Its proposed fix —
*"a version-citation-coherence check: on a spec-doc version bump, flag dependent artifacts
carrying the prior version qualifier as a live 'current-version claim' (vs a frozen
provenance/changelog anchor)"* — subsumes the wave-84 instances, including self-version
markers, because the discriminator it asks for (live claim vs provenance anchor) is exactly
what distinguishes a stale `v2.1` marker from a legitimate historical reference. #749
explicitly invites dedupe: *"If maintainers consider these the same surface, please
link/dedupe."* Adjacent:
[#550](https://github.com/drbothen/vsdd-factory/issues/550) (story-prose citation drift
taxes convergence one finding per round) and
[#682](https://github.com/drbothen/vsdd-factory/issues/682), whose body records the same
sub-case: *"version-tag citations in comments became outdated (a clause attributed to spec
version X when it was actually version Y); one cleanup pass missed instances that a
subsequent pass identified."*

**Verdict: DUPLICATE (#749). Confidence MEDIUM** — MEDIUM rather than HIGH because the
finding's stated mechanism is unsubstantiated, so the dedupe is against the corrected
class, not against the finding as written. Recommended action: add a comment to #749
recording the self-version-marker sub-case (a story body citing *its own* prior version)
and the wave-84 recurrence count; do not open a new issue.

---

### PG-W84-002 — sub-agent message-routing breakage → missing artifacts, backfill commits

**Classification: UPSTREAM.** Agent message-routing protocol and dispatch-template
contract are engine-owned.

**Duplicate check.** The orchestrator's candidates were #457 and #258. Verified:

- [#457](https://github.com/drbothen/vsdd-factory/issues/457) — *"process-gap(subagents):
  completed-but-unreported liveness gap — work finishes, report never delivered without an
  explicit ping."* Body: *"Subagents complete their work fully, then go idle without
  delivering their report. The output exists (edits committed, findings written, verdict
  formed) but is never sent back to the orchestrator. An explicit orchestrator ping
  recovers it immediately."* Evidence table lists 3 instances across adversary,
  state-manager, and consistency-validator in one session. Validated mitigation is exactly
  the wave-84 workaround: *"deliver your report via SendMessage before going idle."* This
  is the same defect as PG-W84-002. **CONFIRMED DUPLICATE.**
- [#258](https://github.com/drbothen/vsdd-factory/issues/258) — *"Agent dispatch returns
  synthesized '[Request interrupted by user for tool use]' with no real user input;
  orchestrator mis-routes to 'pause' instead of retry."* Different mechanism (synthesized
  interrupt string + orchestrator mis-routing), not the silent-idle case. **NOT a
  duplicate**; the orchestrator's preliminary #258 candidate is rejected.

Also adjacent and worth cross-referencing:
[#592](https://github.com/drbothen/vsdd-factory/issues/592) (verdicts delivered via async
teammate channel can be dropped by API error),
[#619](https://github.com/drbothen/vsdd-factory/issues/619) (no instruction-sequencing
protocol for async delegation), [#211](https://github.com/drbothen/vsdd-factory/issues/211)
(agent cannot persist its own report; round-trips through orchestrator — the direct cause
of the artifact needing a backfill commit).

**Verdict: DUPLICATE (#457; #258 rejected). Confidence HIGH.**

---

### PG-W84-003 — burst-log Dim-1 file-count template understates ride-along files

**Classification: UPSTREAM.** `burst-log-template.md` and the `validate-burst-log` hook are
both plugin-owned.

**Duplicate check.** No candidate was supplied by orchestrator triage; one was found.
[#681](https://github.com/drbothen/vsdd-factory/issues/681) —
*"validate-burst-log enforces an undocumented engine-dogfood schema that conflicts with the
shipped burst-log template (dual-validator deadlock)."* Its body enumerates the validator's
required blocks and names the exact surface at issue: *"Files touched **with cardinality
checks**"* is listed as a validator requirement that the shipped template does not carry.
Its suggested resolution #1 is *"Consolidate to a single schema: either update
`burst-log-template.md` to match the validator's canonical form (with Dim blocks documented
for general use across projects) or relax the validator"*, and #2 is *"Define Dim-1 through
Dim-7 in project-agnostic terms."* PG-W84-003 is the narrow instance of that gap: the
template's Dim-1 prose does not tell the agent that ride-along files
(`session-checkpoints.md`, `process-gap-ledger.md`) count toward the cardinality the
validator checks.

**Verdict: DUPLICATE (#681). Confidence MEDIUM** — MEDIUM because #681 frames the gap as
schema divergence + undocumented Dim semantics rather than specifically as ride-along
under-counting; a maintainer could reasonably treat the ride-along guidance as a distinct
docs task. Recommended action: comment on #681 with the ride-along-file cardinality
instance rather than opening a new issue.

---

### PG-W84-004 — STATE.md write-path hook cascade lacks unified error reporting

**Classification: UPSTREAM.** All three hooks (`verify-state-timestamp-refresh`,
`validate-dispatch-advance`, `validate-state-pin-freshness`) are plugin-registered
PostToolUse hooks.

**Duplicate check.** Orchestrator candidate #572 — **CONFIRMED, near-exact.**
[#572](https://github.com/drbothen/vsdd-factory/issues/572) body: *"The only discovery
mechanism is: attempt a write → hook rejects with one error → fix that one thing → attempt
again → next validator rejects → repeat"*; *"Rejection-driven discovery is order-dependent
since each hook reports only its own failure."* Its suggested fix list contains precisely
the remedy PG-W84-004 asks for: *"A `--check-all` pre-flight mode would validate a candidate
STATE.md against all registered validators in one pass, returning the complete violation
list so convergence happens in one iteration."* It also records the same cost profile
(92 tool calls / ~34 minutes in one burst, majority in rejection-fix-retry).

Adjacent: [#616](https://github.com/drbothen/vsdd-factory/issues/616) (PostToolUse
validators evaluate whole-file pre-existing conditions, not the edit delta).

**Verdict: DUPLICATE (#572). Confidence HIGH.**

---

### PG-W84-005 — validate-pr-review-posted hook false-positive for self-authored PRs

**Classification: UPSTREAM.** `validate-pr-review-posted` is a plugin SubagentStop hook.

**Duplicate check.** Orchestrator candidates #626 and #696 were supplied; the tighter match
is [#651](https://github.com/drbothen/vsdd-factory/issues/651) —
*"bug(hooks): validate-pr-review-posted demands approve/request-changes — structurally
unreachable on single-identity projects; mis-detects `gh pr review --comment`."* Body:
*"Expected: the hook recognizes a posted formal COMMENTED review (carrying an explicit
disposition marker) as satisfying the review-posted gate on projects where
approve/request-changes is structurally unreachable. Actual: the hook unconditionally
hard-requires `--approve`/`--request-changes`."* That is PG-W84-005 verbatim, including the
"COMMENTED + explicit artifact = review of record" convention.
[#626](https://github.com/drbothen/vsdd-factory/issues/626) is the underlying-constraint
issue that #651 itself cites (GitHub forbids author self-approve; `reviewDecision` stays
empty on single-identity factories).

**Verdict: DUPLICATE (#651 primary, #626 root constraint). Confidence HIGH.**
The orchestrator's #696 candidate is a valid cross-reference but is scoped to the 9-step
process assumption rather than the hook, so #651 is the correct dedupe target.

---

### PG-W84-006 — pr-manager-completion-guard pressures step-9 fabrication before merge

**Classification: UPSTREAM.** Verified against the installed plugin: `hooks-registry.toml`
line 1204 registers `pr-manager-completion-guard` as a `SubagentStop` WASM hook
(`hook-plugins/pr-manager-completion-guard.wasm`, priority 920, advisory-block mode).

**Duplicate check.** Orchestrator candidate #707 — **verified, NOT a duplicate.**
[#707](https://github.com/drbothen/vsdd-factory/issues/707) is the same hook but the
*inverse* failure mode: there all 9 steps were genuinely complete and the guard failed to
recognise them because it *"appears to count `STEP_COMPLETE` markers per stop-event,"*
forcing redundant re-emission. #707 self-grades LOW / *"friction/redundancy issue, not a
correctness failure"* and explicitly states *"this is not a stuck/false-completion."*
PG-W84-006 is the correctness-relevant direction: the guard demanded step-9 (merge
confirmation) when the merge had **not** happened, producing pressure to assert a false
external fact. Distinct remedy too: #707 asks for cumulative marker accounting;
PG-W84-006 asks for the guard's firing condition to be gated on observed merge state
(`gh pr view --json mergeStateStatus`).

[#673](https://github.com/drbothen/vsdd-factory/issues/673) is the correct *parent class*
(*"blocking validators' fix-instructions prescribe example/placeholder text that is false
for the target project (coerced fabrication/structure)"* — *"a blocking validator's error
message is an instruction an LLM agent will follow literally under pressure"*), but its
three manifestations are all `validate-dispatch-advance` /
`validate-template-compliance` text-transcription cases; it does not cover a
completion-guard demanding an unmet external state.
[#756](https://github.com/drbothen/vsdd-factory/issues/756) is adjacent (pr-manager
lifecycle ran past its dispatch scope) but is the opposite polarity — over-execution, not
premature-completion pressure.

**Verdict: NOVEL-UPSTREAM. Confidence MEDIUM** — MEDIUM because the guard's internal
firing condition was not read from source (the hook ships as a WASM binary; only its
registry entry is inspectable), so the root-cause attribution is behavioural inference from
one observation, and because a maintainer may elect to fold this into #707 as a second
failure mode of the same guard. Filing at observation grade with an explicit dedupe
invitation is the right disposition. Severity in-repo was graded HIGH (fabrication
pressure on a merge-state claim), which justifies filing rather than deferring.

**Draft issue title:**
`process-gap(hooks/pr-manager): pr-manager-completion-guard demands step-9 merge-confirmation at SubagentStop before the merge exists — completion pressure on an unmet external fact (fabrication risk; inverse of #707)`

**Draft issue body:**
> During a per-story PR delivery (single-identity factory, rc.23), the `pr-manager` agent
> stopped mid-flow before the merge had landed, and the `pr-manager-completion-guard`
> SubagentStop hook applied completion pressure for step-9 (merge confirmation) — a step
> whose truth condition depends on an external fact the agent could not yet observe as
> satisfied. The agent correctly refused to record step-9 as complete, but the pressure
> pattern is a fabrication hazard: a less-careful agent complying would write a false
> merge-completion claim into the delivery record, and merge-completion claims are exactly
> the kind of record that downstream state and audit artifacts trust without re-derivation.
> This is the correctness-relevant inverse of #707, which documents the same guard failing
> to *recognise* genuinely-completed steps due to per-stop-event marker accounting; #673 is
> the parent class (blocking-hook pressure induces literal compliance with false text).
> Requested behaviour: the guard must not emit step-9 completion pressure until the merge
> is independently observable — e.g. gate the step-9 branch of the guard on
> `gh pr view <n> --json state,mergeStateStatus,mergedAt` reporting a merged state, and
> treat a stop-event during a legitimate in-progress phase (waiting on CI, awaiting review
> convergence) as "flow in progress", not "steps missing". Confidence note: the root-cause
> attribution is behavioural inference from one observation — the guard ships as a WASM
> binary, so its firing condition was not read from source. If maintainers consider this
> the same surface as #707, please link/dedupe.

---

### PG-W84-008 — PR description commit-count composed before final fixup commit

**Classification: UPSTREAM.** pr-manager step-1 (create-PR / compose description) is an
engine-owned agent workflow.

**Duplicate check.** No candidate was supplied; one was found.
[#663](https://github.com/drbothen/vsdd-factory/issues/663) —
*"identifiers retyped from memory instead of copy-pasted from command output — SHAs, test
names, commit lists silently corrupted."* Its evidence includes *"A per-file commit list
reconstructed from memory: included one commit that never touched the file and omitted 3
that did (ground truth: `git log --follow`)"*, and its adopted remedy #3 is *"Commit lists
derive from `git log --follow` output, never memory."* PG-W84-008 (PR #426 claimed 10
commits, squash base held 11) is the temporal-staleness variant of the same class: the
count *was* derived from command output, but at a point in time before the final fixup
commit existed, and was not re-derived at posting time. Same remedy family
(re-derive from the tool at composition time; never carry a derived count forward).

**Verdict: DUPLICATE (#663). Confidence MEDIUM** — MEDIUM because the mechanisms differ
(temporal staleness vs memory-retype) and a maintainer might treat "re-count immediately
before posting" as a distinct pr-manager step. Given LOW/cosmetic in-repo severity, a new
upstream issue is not justified; recommended action is a comment on #663 adding the
composed-before-final-commit sub-case.

---

### PG-W84-010 — `bin/check-green-doc-tense` Rust-only scan blind spot for `bin/*.py`

**Classification: LOCAL.** Verified from source: `bin/check-green-doc-tense` is a
wirerust-authored Python tool (delivered by STORY-176). Its file-discovery step is
`git ls-files -- tests/*.rs src/**/*.rs` (line 477) followed by an
`if line.endswith(".rs")` filter (line 490), and its module docstring states the scope
explicitly (line 4: *"Scans tracked test files (tests/\*.rs and src/\*\*/\*.rs cfg(test)
modules)"*). There is no plugin template from which this pattern originates — the plugin
ships no green-doc-tense gate; the *class* of gate was inspired by the upstream defect
class in #682, but the implementation, its glob, and its pattern set are entirely local.
The finding therefore does not implicate engine surface.

**Duplicate check.** No upstream filing is warranted. For the record, the generic class is
already tracked upstream twice:
[#339](https://github.com/drbothen/vsdd-factory/issues/339) (*"consistency and
rename-residual checks scoped to a file-type allowlist silently skip prose/rationale/ADR
docs"* — the identical shape: allowlist-scoped scanner cannot police the surfaces outside
its allowlist) and [#599](https://github.com/drbothen/vsdd-factory/issues/599)
(*"agent-authored lint gates silently ignore file args and under-scan — 'verified 0
violations' can be vacuous"*).

**Verdict: LOCAL-CARRY-FORWARD. Confidence HIGH.** Remains a wirerust maintenance item:
extend the discovery glob to `bin/*.py` (or, more generally, to all tracked text surfaces
the gate's patterns are meaningful for) and add a self-application smoke row proving the
gate flags stale prose in its own harness. Note the composition with PG-W85-003: both are
scope/pattern extensions to the same script and should be delivered as one story, because
extending the glob to `bin/*.py` without the extended pattern set would still miss the
`Expected RED:` class that lives there.

---

### PG-W84-012 — `bin-selftest` CI job absent from develop required-status-checks

**Classification: LOCAL.** GitHub branch-protection / ruleset configuration for
`Zious11/wirerust` develop is repo-owned, not engine-owned. Verified that the job exists:
`.github/workflows/ci.yml` line 473 defines `bin-selftest` (`name: Bin selftest suites`).

**Fact-verification limitation (flagged).** I could not independently confirm the
"absent from required-status-checks" half of the finding: branch-protection and ruleset
reads require an authenticated GitHub API call, and this agent has no `Bash`/`gh` access
and WebFetch cannot authenticate. The claim rests on the ledger's record of adversary
observation Obs-P7-2 (STORY-176 pass 7). This is why confidence is MEDIUM rather than HIGH.
**Recommended pre-filing step:** run
`gh api repos/Zious11/wirerust/branches/develop/protection --jq '.required_status_checks.contexts'`
and `gh api repos/Zious11/wirerust/rulesets` before opening the local issue, so the issue
carries the actual context list.

**External corroboration (MCP research, 2026-07-25).** The remediation is not simply "add
the job to the required list" — the research pass surfaced the standard pitfall and the
standard pattern:

- GitHub's required-status-check state machine leaves a required check that never runs in a
  permanently "not yet run" state, so *conditionally-triggered* jobs must not be marked
  required or PRs block indefinitely (GitHub Docs on status checks and protected branches;
  community discussions 26698 / 12395 / 52652 / 183360).
- The recommended patterns are (a) a job that **always runs** and skips heavy work
  internally, or (b) a terminal **aggregator / "all-green" gate job** that provides one
  stable required context representing "all required checks are green" — which also
  immunises the config against job renames (a renamed job silently stops satisfying a
  required context).

Since `bin-selftest` is an unconditional `runs-on: ubuntu-latest` job with no `if:` guard,
pattern (a) already holds and direct registration is safe; the aggregator pattern is the
more durable option if wirerust's job list keeps growing.

Upstream analogues, for reference only (no filing):
[#257](https://github.com/drbothen/vsdd-factory/issues/257) (required_status_checks.contexts
uses workflow filename instead of the GitHub-reported check name — branch protection
silently bypassed) and [#349](https://github.com/drbothen/vsdd-factory/issues/349)
(plugin-pack branch-protection contexts don't match CI job names).

**Verdict: LOCAL-CARRY-FORWARD. Confidence MEDIUM** (classification HIGH; the underlying
"not registered" fact is MEDIUM pending the `gh api` read above).

---

### PG-W85-001 — plugin holdout-evaluation template omits structured caveat block

**Classification: UPSTREAM.** `templates/holdout-evaluation-report-template.md` is
plugin-owned.

**Direct verification of the defect (this validation).** I read the installed template at
`/Users/zious/.claude/plugins/cache/claude-mp/vsdd-factory/1.0.0-rc.23/templates/holdout-evaluation-report-template.md`.
It has exactly four body sections plus a verdict: `## Overall Metrics`,
`## Per-Scenario Scores` (a table), `## Low-Satisfaction Scenarios (score < 0.85)` — the
only per-scenario structured block, with an `### HS-NNN` sub-heading — `## Evidence
Summary`, `## Final Verdict`. **There is no caveat / evaluability / data-availability
section anywhere in the template, and the only structured per-scenario slot is gated on
`score < 0.85`.**

This *confirms the finding but corrects its stated mechanism*: L-W85 describes it as
*"a structural heading defect ... caused the holdout agent to omit the required 'corpus
availability caveat' block."* It is not a heading-hierarchy defect — the block does not
exist in the template at all, and a scenario that scores ≥ 0.85 while carrying a
corpus-availability caveat (HS-136) has no structured place to go, so it necessarily
degrades to a plain table row. The corrected mechanism is a **missing section class**, and
the defect is deterministic rather than incidental.

**Duplicate check.** No candidate was supplied; targeted searches
(`holdout+template`, `holdout-evaluation+heading+caveat+corpus`) returned zero direct hits.
The nearest issue is [#458](https://github.com/drbothen/vsdd-factory/issues/458)
(*"evaluability constraints — hardware/human-gated criteria need a first-class category and
a resume-queue"*), whose body names the same hazard — *"Silent-truncation risk. An evaluator
could quietly omit unevaluable criteria without reporting them — the gate passes on
incomplete evaluation misrepresented as thorough"* — and asks for structured output. But
#458 is scoped to scenario frontmatter (`evaluability:`), scoring denominators, and a
`human-verification-queue.md` artifact; it never mentions the evaluation *report* template,
and its trigger class is hardware/human gating rather than data-corpus availability.
[#617](https://github.com/drbothen/vsdd-factory/issues/617) (template matching is
filename-based with no scope awareness) is adjacent to template governance but a different
defect. The report-template fix is small, concrete, independently landable, and unclaimed.

**Verdict: NOVEL-UPSTREAM. Confidence MEDIUM** — the template gap is verified HIGH
(read from disk), but MEDIUM overall because maintainers may reasonably fold it into #458
as that issue's reporting-surface half. The issue must be filed cross-referencing #458 and
must correct the "heading defect" framing.

**Draft issue title:**
`bug(templates/holdout-evaluator): holdout-evaluation-report-template has no evaluability/data-availability caveat section — a caveated scenario scoring >= 0.85 degrades to a plain table row (reporting-surface half of #458)`

**Draft issue body:**
> `templates/holdout-evaluation-report-template.md` (rc.23) provides exactly one structured
> per-scenario block, `## Low-Satisfaction Scenarios (score < 0.85)` with `### HS-NNN`
> sub-headings. There is no section for evaluability or data-availability caveats. Observed
> consequence: a holdout scenario that scored **above** the 0.85 threshold while resting on
> a partially-unavailable input corpus (a machine-local, git-ignored capture corpus) had
> nowhere structured to record that caveat, so the evaluator emitted a plain score row and
> the caveat was lost from the report structure — the score itself was correct, only the
> documentation structure degraded. This is the reporting-surface counterpart to #458, which
> asks for evaluability as a first-class category and names the same hazard
> ("silent-truncation risk: an evaluator could quietly omit unevaluable criteria without
> reporting them"), but #458 is scoped to scenario frontmatter, denominator arithmetic, and
> a human-verification queue and does not touch the report template. Requested fix: add a
> mandatory `## Evaluability & Input-Availability Caveats` section to the report template
> with a per-scenario `### HS-NNN` sub-block (caveat class, affected criteria, what was and
> was not exercised, effect on the score), decoupled from the `< 0.85` gate so that
> above-threshold caveated scenarios are captured; and add an "N/A — no caveats" convention
> so the section is never silently dropped. Note for the maintainer: this was originally
> reported downstream as a "heading hierarchy defect"; on inspection the section does not
> exist in the template at all, so the omission is deterministic, not an agent lapse.

---

### PG-W85-002 — multi-document sibling-sweep discipline for factory-artifact loci

**Classification: UPSTREAM for the defect class; LOCAL for the codification.** The remedy
the lesson proposes is *"add a factory-artifact sibling class to DF-SIBLING-SWEEP-001"* —
DF-SIBLING-SWEEP-001 lives in this repo's `.factory/policies.yaml`, so the actionable work
is local policy text. The underlying agent-behaviour defect is engine-owned.

**Duplicate check.** Orchestrator candidates #470, #216, #507, #445, #387 — all verified
relevant; the covering set is well-established and dense:

- [#470](https://github.com/drbothen/vsdd-factory/issues/470) — *"remediation delivers
  finding's exact scope but does not sweep sibling artifacts — seven-consecutive-pass
  pattern with two recursive-inside-codification recurrences + third-order failure."*
  This is the primary covering issue and is a direct match for "remediation covered the
  primary locus, missed the siblings."
- [#507](https://github.com/drbothen/vsdd-factory/issues/507) — *"peer-artifact sweep when
  applying prose fixes"*, explicit successor to
  [#504](https://github.com/drbothen/vsdd-factory/issues/504) (*"preventive-sweep prompt
  should span ALL story artifacts (spec + tests + VPs + ADRs), not just test files"*),
  [#505](https://github.com/drbothen/vsdd-factory/issues/505) (story pseudocode +
  Architecture Mapping / File Structure Requirements tables) and
  [#506](https://github.com/drbothen/vsdd-factory/issues/506) (BC self-referential
  metadata as a first-class sweep target). This #504→#505→#506→#507 chain *is* the
  "multi-document factory-artifact siblings for the same fact" class, enumerated
  incrementally.
- [#299](https://github.com/drbothen/vsdd-factory/issues/299) — *"fixes to a value
  duplicated across N surfaces not propagated to all surfaces; no mandatory set-equality
  guard."*
- [#216](https://github.com/drbothen/vsdd-factory/issues/216) — *"standardize 'enumerate
  before fix' sweep template to prevent incomplete sibling propagation"* (the remedy issue).
- [#445](https://github.com/drbothen/vsdd-factory/issues/445) and
  [#387](https://github.com/drbothen/vsdd-factory/issues/387) are valid narrower
  cross-references (STORY-bump sweep propagation; duplicated normative call-sequence
  sketches drifting independently under partial fixes).

The lesson's own premise — *"DF-SIBLING-SWEEP-001 covers known sibling classes in source;
the gap class of 'multi-document factory-artifact siblings for the same fact' is not
explicitly covered"* — is a statement about the **local policy's** coverage, not about the
upstream tracker's. Upstream, the class is covered five times over.

**Verdict: DUPLICATE (#470 primary; #507/#504/#505/#506 chain; #299, #216). Confidence
HIGH.** No upstream filing. The local DF-SIBLING-SWEEP-001 extension may proceed as policy
codification with no GitHub issue required (it is a policy edit, not a defect fix); if a
tracking issue is desired it is a wirerust-local one.

---

### PG-W85-003 — green-doc-tense gate misses `Expected RED:` / `currently falls through`

**Classification: LOCAL.** Same tool as PG-W84-010 — `bin/check-green-doc-tense`, a
wirerust-authored script whose pattern set (`_VIOLATION_PATTERNS`, line 217) is entirely
local. Verified the gap directly: grepping the script for `Expected RED` and
`falls through` returns **no matches**, confirming neither phrase class is in the pattern
set, which is why the gate exited 0 on the 9 stale sites the STORY-180 pass-1 adversary
found.

**Duplicate check.** Orchestrator candidate #682 — verified and **relevant as the class
owner, not as a duplicate of the local tooling task.**
[#682](https://github.com/drbothen/vsdd-factory/issues/682) is
*"stale RED-gate / todo!() docstrings survive the Red→Green transition and are not caught by
any gate."* Its suggested fix #1 is precisely a phrase-pattern lint: *"scan test docstrings
and comments for present-tense RED-gate indicators ('Status: RED', 'RED GATE:', 'todo!()
stub', 'FAILS until', 'Red Gate stub') and verify whether the identified symbol still
contains a todo!()/unimplemented!() implementation."* wirerust already *has* that gate
locally; the upstream ask is for the engine to ship one. Extending wirerust's local pattern
set is therefore not an upstream defect — the engine has no pattern set to extend. Note
that #682 also records the same incomplete-propagation shape wirerust observed
(*"Pass N+1 — discovered the same category of stale docstrings in a related inline test
module in a different file that the initial cleanup had not addressed"*).

**Verdict: LOCAL-CARRY-FORWARD. Confidence HIGH.** Remains a wirerust tooling item: add
zero-false-positive patterns for the `Expected RED:` heading class and the
`currently falls through` body-phrase class. Deliver jointly with PG-W84-010 (same script,
composed scope) — and consider contributing the resulting pattern set back to #682 as
field data, since #682's own pattern list omits both classes.

> **CORRECTION (2026-07-25, F-W86S-P3-014):** The causal claim "which is why the gate exited
> 0 on the 9 stale sites the STORY-180 pass-1 adversary found" is **falsified by the primary
> finding record**. `cycles/wave-085/STORY-180/convergence-report.md` lines 63-66 documents
> that the 9 stale sites used `currently asserts`, `is expected to`, and similar RED-phase
> phrasing — phrase classes that `bin/check-green-doc-tense` does **not** cover, but are
> distinct from `Expected RED:` and `currently falls through`. The grep confirmation in this
> section ("grepping the script for `Expected RED` and `falls through` returns no matches")
> verified the gap for those two labels without verifying them as the actual stale text.
>
> The broader verdict (LOCAL-CARRY-FORWARD, same script, deliver with PG-W84-010) remains
> valid. The scope was corrected in wave-86 pass-2/pass-3: STORY-183 v1.2 (DF-GREEN-DOC-
> TENSE-SWEEP v3) dropped Patterns 30/31 (`currently falls through`, `is expected to`) from
> the TIER-1 automated set, re-tiered them TIER-2, and STORY-183 v1.3 (DF-GREEN-DOC-TENSE-
> SWEEP v4) re-anchored the TIER-1 set to patterns verified by grep evidence.
>
> **This is the third instance of the lesson-summary-vs-finding-record failure mode** flagged
> in Cross-Finding Observation 3 of this report (§3, finding 3): "Two findings' stated
> mechanisms did not survive verification." PG-W85-003 was captured from the lesson-summary
> description rather than from the convergence-report primary record, causing the mechanism
> ("phrase class") to diverge from ground truth. The proposed fix — requiring a
> *verified locus* field (file path + line, read at capture time) — would have prevented this
> propagation.

---

### PG-W85-004 — pr-manager must not attempt self-approval on its own PR

**Classification: UPSTREAM.** The pr-manager dispatch prompt and the 9-step review
playbook are plugin-owned (`agents/pr-manager.md`).

**Duplicate check.** Orchestrator candidates #696 and #626 — both verified; #626 is the
primary.

- [#626](https://github.com/drbothen/vsdd-factory/issues/626) — *"review playbooks
  prescribe `gh pr review --approve/--request-changes`, which GitHub forbids from the PR
  author's account — single-identity factories can never produce a formal reviewDecision."*
  This is the exact root cause of the wave-85 observation: the agent issued
  `gh pr review --approve` on a PR it authored **because the playbook prescribes that
  command**. PG-W85-004's proposed fix (a MUST-NOT-self-review pre-check in the dispatch
  prompt) is one of the two obvious remedies for #626; the other (stop prescribing
  `--approve`) is #626's own framing. Same surface, same fix locus.
- [#696](https://github.com/drbothen/vsdd-factory/issues/696) — *"single-account
  deployments cannot post the formal PR approval the 9-step process assumes"* — the
  process-level restatement.
- [#651](https://github.com/drbothen/vsdd-factory/issues/651) — the hook-side consequence
  (already the covering issue for PG-W84-005). Worth noting that PG-W84-005 and PG-W85-004
  are the two ends of one engine contradiction: the playbook prescribes an approval the
  platform forbids (#626), and the hook then blocks on that approval's absence (#651).

**Verdict: DUPLICATE (#626 primary; #696, #651). Confidence HIGH.** No new upstream issue.
Recommended action: comment on #626 with the wave-85 datum — that the GitHub two-party
harness guard caught the attempt, that no approval landed, and that the requested
prompt-level remedy is an explicit reviewer-identity ≠ author pre-check — since #626
currently argues from the platform constraint rather than from an observed agent attempt.

---

### PG-W85-005 — gitignored machine-local e2e fixtures produce false-green `cargo test`

**Classification: LOCAL.** The root cause is wirerust's own decision to git-ignore the
large real-capture IEC-104 ITI corpus, and the local e2e harness's runtime fixture
discovery. All three candidate fixes named in the lesson — (a) a gate-entry fixture-count
sweep, (b) a fixture manifest with skip reporting, (c) committing small representative
fixtures — are local. No engine surface is implicated by the finding as written.

**Duplicate check.** No candidate was supplied; targeted search
(`gitignored+fixtures+tests+skipped`) returned zero results. The nearest upstream issue is
[#694](https://github.com/drbothen/vsdd-factory/issues/694) —
*"a verification-obligation test gated behind a build feature the CI test job doesn't enable
is silently excluded from CI."* Its diagnosis is the same *class* as PG-W85-005 —
*"the CI test step reports 'N passed; 0 failed' and looks authoritative; nothing surfaces
'these M feature-gated tests were not compiled'"*, and it labels the class a
**coverage-exclusion gap, not an assertion-strength gap**. The mechanism differs
(build-feature gating vs data-fixture availability), but the invariant both want is
identical: an environment-dependent reduction in the executed test set must be loud, not
silent. Related-but-distinct upstream items:
[#331](https://github.com/drbothen/vsdd-factory/issues/331) (no check that documented
build/test commands are non-vacuous — wrong path runs 0 tests, exits green) and
[#334](https://github.com/drbothen/vsdd-factory/issues/334) (silent zero-match globs that
vacuously pass).

**External corroboration (MCP research, 2026-07-25).** The research pass confirms this is a
recognised Rust-ecosystem hazard rather than a wirerust idiosyncrasy, and that the tooling
does not solve it for you:

- libtest has no first-class "skipped" status — a runtime early-return is indistinguishable
  from a pass from the harness's perspective, so a host missing fixtures reports fewer
  passes and still exits 0.
- `cargo-nextest` does document skip reporting and no-tests-discovered behaviour: with
  `--no-tests=fail` (recent versions), a run that discovers no tests exits with advisory
  code `NO_TESTS_RUN`, *"making the absence of tests a loud condition."* But nextest
  *"leaves fixture management"* to the project — it detects *zero* tests, not *fewer than
  expected* tests, so it does not cover the 31-vs-66 case.
- The recommended mitigations found were exactly the lesson's own candidates:
  **assert an expected fixture/test count**, prefer `#[ignore]` (a reported state) over a
  silent runtime early-return, manage the corpus explicitly (git-lfs, or commit small
  representative fixtures), and use manifest-based fixture inventories with skip reporting.

That independent convergence on options (a) and (b) is good evidence the local remedy is
sound. **Recommendation for the local item:** combine (b) a fixture manifest whose absent
entries emit a loud, machine-readable skip report, with (a) a gate-entry sweep that asserts
the manifest-derived expected count — a count-only assertion without a manifest will
itself go stale as the corpus grows.

**Verdict: LOCAL-CARRY-FORWARD. Confidence HIGH.** No upstream filing is required. If the
wave gate wants engine-side support, the honest framing would be a narrow companion comment
on #694 asking that wave-gate entry require a test-count baseline assertion — but that is
discretionary and is not implied by the finding as written.

---

## 3. Cross-Finding Observations

1. **Two findings share one engine contradiction.** PG-W84-005 (#651) and PG-W85-004
   (#626) are the hook side and the playbook side of the same single-identity-factory
   defect: the playbook prescribes an approval GitHub forbids from the author account, and
   the hook then blocks on that approval's absence. When commenting upstream, cross-link
   them so a maintainer fixing one does not leave the other live.

2. **Two findings share one local script.** PG-W84-010 (scan glob) and PG-W85-003 (pattern
   set) both target `bin/check-green-doc-tense` and should be one story. Delivering the
   glob extension alone would extend the scan to `bin/*.py` without the patterns that
   actually live there; delivering the patterns alone would leave the Python harness
   unscanned. Note that a story touching `bin/` triggers the CLAUDE.md AC-158-001 CHANGELOG
   obligation and, per L-W84-003 / AC-165-001, any new `bin/test_*.py` must be wired into
   the `bin-selftest` CI job at delivery time.

3. **Two findings' stated mechanisms did not survive verification** (PG-W84-001's
   `## Story v<N.M>` header; PG-W85-001's "heading hierarchy defect"). In both cases the
   underlying gap is real but differently shaped. Recommend that the process-gap ledger
   template require a *verified locus* field (file path + line, read at capture time) for
   any finding that names a mechanism, so DF-VALIDATION-001 is not re-deriving the
   mechanism from scratch. This is itself a candidate local process improvement rather than
   an upstream filing.

4. **Duplicate-rate signal.** 8 of 14 (57%) deferred findings were already tracked
   upstream, and 4 of the 6 orchestrator-supplied candidate sets were confirmed. Two
   preliminary candidates were **rejected** on inspection (#258 for PG-W84-002; #707 for
   PG-W84-006) — both would have suppressed a real filing or mis-attributed a root cause,
   which is the concrete value DF-VALIDATION-001 delivered on this batch.

---

## 4. Recommended Actions

**File upstream (2):**

| # | Finding | Target |
|---|---------|--------|
| 1 | PG-W84-006 | new issue on `drbothen/vsdd-factory`, cross-ref #707 / #673 / #756 |
| 2 | PG-W85-001 | new issue on `drbothen/vsdd-factory`, cross-ref #458 / #617 |

**Comment on an existing upstream issue instead of filing (4):**

| Finding | Target issue | Additive content |
|---------|-------------|------------------|
| PG-W84-001 | #749 | self-version-marker sub-case; wave-84 recurrence count (3+) |
| PG-W84-003 | #681 | ride-along-file cardinality instance (session-checkpoints.md, process-gap-ledger.md) |
| PG-W84-008 | #663 | composed-before-final-commit temporal-staleness sub-case (PR #426: claimed 10, base held 11) |
| PG-W85-004 | #626 | observed agent attempt + requested reviewer-identity ≠ author pre-check |

**No action upstream (3):** PG-W84-002 (#457), PG-W84-004 (#572), PG-W84-005 (#651),
PG-W85-002 (#470 + chain) are sufficiently covered as-is.

**Local wirerust items (4 + 1 policy):**

| Finding | Local action | Pre-filing step |
|---------|-------------|-----------------|
| PG-W84-010 + PG-W85-003 | one story: extend `bin/check-green-doc-tense` glob to `bin/*.py` **and** add `Expected RED:` / `currently falls through` zero-FP patterns; add self-application smoke row | none |
| PG-W84-012 | register `bin-selftest` as a required status check (safe — job is unconditional) or adopt an aggregator gate job | `gh api repos/Zious11/wirerust/branches/develop/protection` + `gh api .../rulesets` to capture the actual context list |
| PG-W85-005 | fixture manifest with loud skip reporting + gate-entry expected-count assertion derived from the manifest | none |
| PG-W85-002 | extend DF-SIBLING-SWEEP-001 in `.factory/policies.yaml` with a factory-artifact sibling class (policy edit; no issue required) | none |

---

## 5. Confidence & Limitations

- **Upstream tracker coverage.** Live-queried #126–#763. The local snapshot covered
  #126–#687; issues #688–#763 were enumerated live via the GitHub search API across three
  date windows. A small number of numbers in the #705–#745 band resolved to pull requests
  rather than issues and were not individually inspected; a duplicate hiding there is
  possible but unlikely given that keyword searches across the whole repo were also run per
  finding.
- **WASM hooks are opaque.** `pr-manager-completion-guard` and the STATE.md validators ship
  as compiled `.wasm` plugins. Their registration is inspectable
  (`hooks-registry.toml`) but their firing conditions are not, so PG-W84-004 and PG-W84-006
  root causes rest on observed behaviour plus upstream issue bodies, not source reads. This
  is the reason PG-W84-006 is MEDIUM rather than HIGH.
- **Branch protection unverifiable from this agent.** No `Bash`/`gh` and no authenticated
  WebFetch, so PG-W84-012's "not in required-status-checks" fact is carried from the ledger.
  A `gh api` read is prescribed before the local issue is opened.
- **No finding is INCONCLUSIVE.** All 14 reached a verdict. Two carry an explicit
  mechanism-correction (PG-W84-001, PG-W85-001) and one carries an explicit
  fact-verification gap (PG-W84-012).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | `reasoning_effort: high`, two-part query: (1) GitHub required-status-checks vs rulesets, the "required check that never runs blocks forever" pitfall, and aggregator/always-run gate-job patterns (for PG-W84-012); (2) Rust `cargo test` silent fixture-absence false-green — expected-count assertions, libtest's missing skipped status, cargo-nextest `--no-tests` / `NO_TESTS_RUN`, `#[ignore]` vs runtime early-return, git-lfs vs committed representative fixtures, manifest-based fixture inventories (for PG-W85-005). Result exceeded the tool's inline token cap and was read from the spilled result file via targeted extraction. |
| Perplexity perplexity_reason | 0 | not needed — synthesis was over primary-source issue bodies and on-disk artifacts |
| Perplexity perplexity_search | 0 | GitHub's own search API gave better precision for issue-tracker dedupe than a general web ranker |
| Perplexity perplexity_ask | 0 | no ≤2-sentence factual lookups arose |
| Context7 | 0 | no library-API questions in scope |
| Tavily tavily_search / research / extract / crawl / map | 0 | Tavily is the cross-validation layer; upstream dedupe was validated against the authoritative GitHub API directly, and the two external best-practice claims were corroborated inside the deep-research pass by GitHub Docs + community discussions + Rust/nextest docs |
| WebFetch | 26 | GitHub API: 4 issue-enumeration calls (all-issues page 1; `created:>2026-07-15`; `created:2026-07-18..21`; `created:2026-07-21..24`), 10 keyword searches (stale version marker; self-review/self-approve; completion guard + fabricate; burst-log; story version header/frontmatter drift; holdout+template; holdout-evaluation+heading+caveat+corpus; PR description commit count; gitignored fixtures tests skipped; step-9 merge guard premature; STEP_COMPLETE guard; sibling sweep factory artifact multi-document), and 11 full issue-body reads (#457, #458, #572, #663, #673, #681, #682, #694, #707, #749, #651) |
| WebSearch | 0 | superseded by direct GitHub API queries |
| Read | 6 | wave-084 lessons.md; wave-085 lessons.md; wave-084 process-gap-ledger.md; `.factory/planning/vsdd-factory-upstream-issues.md` (2 pages); plugin `holdout-evaluation-report-template.md` |
| Glob | 4 | locate process-gap ledger, prior df-validation reports, plugin holdout templates, plugin story templates |
| Grep | 7 | verify plugin `story-template.md` heading/version structure; verify `## Story v<N.M>` absence in STORY-166 and across `.factory/stories/`; verify `pr-manager-completion-guard` registration in `hooks-registry.toml`; verify `bin-selftest` job in `ci.yml`; verify `bin/check-green-doc-tense` `.rs`-only glob and absence of `Expected RED` / `falls through` patterns; extract passages from the spilled deep-research result |
| Training data | 0 areas | Every verdict is anchored to a cited issue URL, an on-disk artifact read during this pass, or the MCP research output. No version numbers, issue numbers, or hook names were asserted from model knowledge. |

**Total MCP tool calls:** 1 (`perplexity_research`, high effort — succeeded; output spilled
to a result file and read by targeted extraction).

**Deviation note per the research-agent mandate.** `perplexity_research` was used once, for
the only two questions in this batch that are genuinely external-knowledge questions
(GitHub branch-protection semantics; Rust fixture-absence false-green patterns). The other
twelve findings are **duplicate-adjudication against a specific issue tracker**, for which
the authoritative source is `api.github.com` itself, not a web-search synthesiser — a
Perplexity summary of a GitHub issue would be a lower-fidelity copy of a document I can
read directly, and would risk hallucinated issue numbers, the single most damaging error
class in a dedupe report. All 26 WebFetch calls target `api.github.com` primary data.

**Training data reliance:** low — 0 claims rest on model knowledge; 11 upstream issue
bodies were read verbatim, 6 on-disk artifacts were read, and 7 greps verified template,
hook-registry, CI-job, and script-internals claims first-hand. The two findings whose
originally-stated mechanisms failed verification are flagged inline rather than
silently corrected.

**Residual risk:** MEDIUM-confidence verdicts (PG-W84-001, -003, -006, -008, -012,
PG-W85-001) should be re-checked by the filing agent immediately before filing/commenting,
since the upstream tracker moves ~3 issues/day and a covering issue could land in the
interval.
