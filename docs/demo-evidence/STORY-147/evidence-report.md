# Demo Evidence Report — STORY-147

**Story:** Repo-Local Mutation-Testing Defaults: `.cargo/mutants.toml` Timeout Floor + CLAUDE.md Guidance
**Story ID:** STORY-147
**Wave:** 84
**Recorded:** 2026-07-19
**Recorded at commit:** `7ff84f56` (adversarial convergence COMPLETE — P6/P7/P8 clean)
**Toolchain:** VHS 0.11.0 (bash shell, Menlo font, Dracula theme) · cargo test · cargo-mutants 27.0.0

---

## Coverage Map

| Acceptance Criteria | Demo Artifact(s) | What It Shows |
|---|---|---|
| AC-147-001 (`.cargo/mutants.toml` exists, `minimum_test_timeout = 300`, no decoy at repo root) | `AC-147-001-config-file-timeout-floor.{gif,webm}` | `cat .cargo/mutants.toml` shows the shipped file with `minimum_test_timeout = 300` and no `jobs` key. Negative/decoy path: `ls mutants.toml` at the repo root fails with `No such file or directory` — confirming the repo-root decoy location does not exist. |
| AC-147-002 (guard test enforces content validity; real tool also rejects invalid config) | `AC-147-002-guard-test-success-negative-revert.{gif,webm}`, `AC-147-002-cargo-mutants-tool-enforcement.{gif,webm}` | See "Two artifacts for AC-147-002" below. |
| AC-147-003 (CLAUDE.md "### Mutation testing" section with required markers) | `AC-147-003-claude-md-mutation-section.{gif,webm}` | `cargo test ... test_AC_147_003` green, then `grep -A 12 "### Mutation testing" CLAUDE.md` renders the full section (low-parallelism guidance, `--jobs` warning, rationale incl. "false 0 missed"/"wall-clock", `PG-MUTANTS-JOBS-001` + `fix-tls-clienthello-frag`, `#654` upstream pointer, `minimum_test_timeout` note). Negative path: `sed` temporarily deletes the `PG-MUTANTS-JOBS-001` line — the guard test FAILS with `AC-147-003(c): CLAUDE.md does not reference the process-gap ID PG-MUTANTS-JOBS-001.` File is then reverted (`git checkout -- CLAUDE.md`) and the test is shown green again. |
| AC-147-004 (self-audit conjunction — both defenses present simultaneously) | `AC-147-004-conjunction-both-defenses.{gif,webm}` | `cargo test ... test_AC_147_004` green, then a combined view (`cat .cargo/mutants.toml` + `grep -A 3 '### Mutation testing' CLAUDE.md`) shows both defenses side by side. Negative path: injecting the invalid `jobs` key breaks the config defense — the conjunction test FAILS with `AC-147-004: first line of defense (...) is absent or invalid`. File is reverted and the test is shown green again, both defenses restored. |

### Two artifacts for AC-147-002

AC-147-002 covers two distinct claims that both needed real, separately-observable evidence:

1. **The guard test itself** (`repo_mutation_config_tests`) enforces content validity —
   `AC-147-002-guard-test-success-negative-revert`: baseline `cargo test` is 9/9 green;
   an invalid `jobs = 1` line is appended; re-running shows 3 FAILED with the
   allowlist/no-jobs fatal-key assertion messages (`` `.cargo/mutants.toml` contains a
   `jobs` key... would abort EVERY `cargo mutants` run with a FATAL parse error``); the
   file is reverted (`git checkout -- .cargo/mutants.toml`) and the suite is shown
   9/9 green again.
2. **The real cargo-mutants 27.0.0 binary** independently rejects the same invalid
   config — `AC-147-002-cargo-mutants-tool-enforcement`: with the same `jobs = 1`
   injected, `cargo mutants --list` fails with a real TOML parse error
   (`Error: parse toml from <repo>/.cargo/mutants.toml` / `TOML parse error at line 1,
   column 1`); the file is reverted and `cargo mutants --list` succeeds again, printing
   real mutant candidates (`src/main.rs:73:5: replace format_zero_packet_notice ->
   String with String::new()`, etc.).

The absolute host path cargo-mutants embeds in its own error message
(an absolute filesystem path ending in `/.cargo/mutants.toml`) is scrubbed live in the
terminal via a `sed`
pipe (`sed -E 's#/[A-Za-z0-9_./-]*/\.cargo/mutants\.toml#<repo>/.cargo/mutants.toml#'`)
before being displayed — the underlying error, its cause chain, and the TOML parser
diagnostic are real, unedited cargo-mutants output; only the leading path fragment is
rewritten (PG-W70-DEMO-SCRUB).

---

## Artifacts

```
docs/demo-evidence/STORY-147/
  AC-147-001-config-file-timeout-floor.tape        — VHS script source
  AC-147-001-config-file-timeout-floor.gif         — 100K GIF recording
  AC-147-001-config-file-timeout-floor.webm        — 97K WebM recording

  AC-147-002-guard-test-success-negative-revert.tape  — VHS script source
  AC-147-002-guard-test-success-negative-revert.gif   — 2.3M GIF recording
  AC-147-002-guard-test-success-negative-revert.webm  — 1.1M WebM recording

  AC-147-002-cargo-mutants-tool-enforcement.tape   — VHS script source
  AC-147-002-cargo-mutants-tool-enforcement.gif    — 136K GIF recording
  AC-147-002-cargo-mutants-tool-enforcement.webm   — 148K WebM recording

  AC-147-003-claude-md-mutation-section.tape       — VHS script source
  AC-147-003-claude-md-mutation-section.gif        — 888K GIF recording
  AC-147-003-claude-md-mutation-section.webm       — 836K WebM recording

  AC-147-004-conjunction-both-defenses.tape        — VHS script source
  AC-147-004-conjunction-both-defenses.gif         — 1.1M GIF recording
  AC-147-004-conjunction-both-defenses.webm        — 1.1M WebM recording

  evidence-report.md                               — this file
```

---

## Notes on Recording Approach

- **Toolchain choice:** STORY-147 is a config/documentation story (`.cargo/mutants.toml`
  + a `CLAUDE.md` note), not a CLI product with its own binary UX. Evidence is recorded
  as VHS terminal sessions exercising the real verification surfaces: the shipped
  config file, the guard test suite (`tests/repo_mutation_config_tests.rs`, 9 tests),
  the `CLAUDE.md` section, and the real installed `cargo-mutants` 27.0.0 binary.
- **Shell choice:** tapes use `Set Shell "bash"`, not zsh. An earlier zsh recording pass
  showed that inline `# comment` narration lines are NOT treated as no-op comments by
  an interactive zsh session in this environment (`interactive_comments` is off by
  default), which produced garbled `zsh: ... not found` errors in the terminal
  transcript. Bash treats `#` as a comment start unconditionally, so all five tapes in
  this story use bash for narration lines to type cleanly.
- **Negative-path methodology:** every tape's negative path is a *real* mutation of the
  actual shipped files (`.cargo/mutants.toml` or `CLAUDE.md`) inside the recording
  itself — appending `jobs = 1`, or deleting the `PG-MUTANTS-JOBS-001` line via `sed`
  — followed by a real re-run of the test suite or `cargo mutants --list`, followed by
  a real `git checkout --` revert and a real re-run confirming green again. No output
  in any tape is hand-written; every panic message, TOML parse error, and pass/fail
  count is genuine tool output captured live during recording.
- **Worktree hygiene:** the worktree (`.worktrees/STORY-147/`) was confirmed `git status
  --short` clean before and after every recording session — each tape's revert step
  (`git checkout -- .cargo/mutants.toml` / `git checkout -- CLAUDE.md`) is the SAME
  mechanism used to restore the repo to a clean state outside the recording, so no
  manual cleanup was needed between tapes.
- **`<REPO-ROOT>` placeholder:** each tape's `Hide`-block `cd` line uses a
  `<REPO-ROOT>` placeholder in the committed `.tape` source (per PG-W70-DEMO-SCRUB /
  repo convention — see `docs/demo-evidence/STORY-130/*.tape`,
  `docs/demo-evidence/STORY-159/*.tape`). During actual recording this was substituted
  with the real local worktree path, but that substitution happens entirely inside a
  `Hide` block followed by a `clear`, so the real path is never visible in the
  rendered GIF/webm frames — confirmed by frame-by-frame inspection of each recording.

---

## Scrub-Gate Result (PG-W70-DEMO-SCRUB)

Gate command (per `.factory/maintenance/demo-evidence-scrub-gate.md`) run from repo
root, scoped to this story's evidence directory — pattern matches absolute host home
paths and tilde-form home references:

```
$ grep -rE '<abs-user-path>|<abs-home-path>|<tilde-home>' docs/demo-evidence/STORY-147/
(no output — exit code 1)
```

Zero matches in all `.tape` source files and no plain-text path leakage identified in
the rendered recordings (frame-by-frame spot checks on every tape; the one real
absolute-path leak — cargo-mutants' own error message in
`AC-147-002-cargo-mutants-tool-enforcement` — is scrubbed live via `sed` before
display, per "Two artifacts for AC-147-002" above). The repo-wide (unscoped) gate run
does surface pre-existing matches in unrelated, previously-merged story evidence
(`docs/demo-evidence/STORY-090/evidence-report.md`,
`docs/demo-evidence/STORY-052/*.tape`) — these predate this delivery and are out of
scope for STORY-147.

---

## AC Coverage Status

| AC | Covered | Vehicle |
|----|---------|---------|
| AC-147-001 | Yes | `AC-147-001-config-file-timeout-floor` (success: `cat` shows `minimum_test_timeout = 300`; negative: `ls mutants.toml` decoy-absence at repo root) |
| AC-147-002 | Yes | `AC-147-002-guard-test-success-negative-revert` (guard test 9/9 green → 3 FAILED on invalid `jobs` key → reverted → 9/9 green) + `AC-147-002-cargo-mutants-tool-enforcement` (real `cargo mutants --list` fatal TOML error on invalid key → reverted → real mutant listing succeeds) |
| AC-147-003 | Yes | `AC-147-003-claude-md-mutation-section` (guard test green + `grep -A 12` full section text; negative: dropping `PG-MUTANTS-JOBS-001` line fails the guard test; reverted → green) |
| AC-147-004 | Yes | `AC-147-004-conjunction-both-defenses` (conjunction test green + combined config/doc view; negative: breaking the config defense fails the conjunction test; reverted → both defenses green again) |
