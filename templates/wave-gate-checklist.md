# Wave Gate Checklist

Use this checklist before declaring a wave gate closed. Each item must be
checked or explicitly marked N/A with a justification.

## Cycle Artifact Identity (wave-72 and forward; AC-158-003)

All cycle artifacts for **this wave and forward** must carry `story_id:` and
`bcs:` YAML frontmatter fields before the gate closes. Run the lint tool
against every NEW artifact created this wave:

```bash
bin/lint-cycle-artifact .factory/cycles/wave-NNN/STORY-NNN/<artifact>.md
```

- [ ] Every new cycle artifact for this wave passes `bin/lint-cycle-artifact`.

**Scope note:** Wave-71-and-earlier artifacts are outside lint scope by
design. Running `bin/lint-cycle-artifact` against them will fail rule (1)
(missing frontmatter) — this is expected and those artifacts are NOT required
to be retroactively updated. The lint tool targets wave-72-and-forward cycle
artifacts only.

## Code-Review Artifact (AC-158-006, PG-W71-CODEREVIEW-ARTIFACT)

A `cycles/wave-NNN/wave-gate/code-review.md` artifact MUST be written
enumerating every MINOR and NIT finding from the gate-level code review
together with its disposition (accepted / deferred / fixed).

- [ ] `cycles/wave-NNN/wave-gate/code-review.md` exists.
- [ ] File enumerates MINOR and NIT findings with dispositions, or contains a
      "No findings" note if the gate-level review produced zero findings.

## Standard Gate Checks

- [ ] All story tests pass (`cargo test --all-targets`).
- [ ] Clippy clean (`cargo clippy --all-targets -- -D warnings`).
- [ ] Format clean (`cargo fmt --check`).
- [ ] CHANGELOG.md has an `[Unreleased]` entry covering all `src/`, `Cargo.toml`,
      and `bin/` changes in this wave (AC-158-001, `changelog-gate` CI job).
- [ ] All PRs for this wave have merged to `develop`.
- [ ] STATE.md drift items reviewed and updated.
- [ ] Any new deferred findings documented in the backlog or STATE.md.
