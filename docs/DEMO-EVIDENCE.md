# Demo Evidence Conventions

## Purpose

`docs/demo-evidence/` archives per-acceptance-criteria evidence captured during story
PRs. Each story's evidence is recorded by the demo-recorder agent after all tests pass
and before the PR is opened, providing a permanent audit trail that a given AC was met
at the time of delivery.

## Scrub Placeholders (`<REPO-ROOT>`, `<HOME>`)

Files in `docs/demo-evidence/` may contain the tokens `<REPO-ROOT>` and `<HOME>`.
These are **scrub placeholders** — not environment variables and not intended for
substitution.

They were inserted by PR #376 (F-W70P2-002) to replace absolute host filesystem paths
(e.g. `/Users/<username>/Documents/GITHUB/wirerust` or `/home/<user>`) that were
present in 193 committed evidence files. The placeholders mark where machine-specific
paths appeared in the original recordings. Do not attempt to expand or substitute them.

## VHS `.tape` Scripts Are Archived Evidence

Some story directories contain `.tape` scripts (VHS terminal recordings). These are
**archived evidence only** — they are not replayable as-is.

The tape scripts referenced ephemeral `.worktrees/STORY-NNN` paths that existed only
during the story's TDD cycle. Those worktrees are removed after the PR merges. Running
a `.tape` script after worktree cleanup will fail with path-not-found errors.

The `.tape` files are retained as a record of what was demonstrated; the `.txt` and
`.md` artifact files in the same directory contain the actual captured output.

## Per-Story Layout Convention

Each story's evidence lives under its own subdirectory:

```
docs/demo-evidence/
    <STORY-ID>/
        evidence-report.md          # Summary table: AC → artifact → verdict
        AC-<NNN>-<slug>.txt         # Captured terminal output for each AC
        AC-<NNN>-<slug>.tape        # VHS script (archived; see note above)
        ...
```

Where `<STORY-ID>` is the story identifier (e.g. `STORY-149`) or holdout-scenario
identifier (e.g. `HS-043`). The `evidence-report.md` in each subdirectory is the
primary document; it lists every AC, links to its artifact, and records the
pass/fail/N/A verdict.
