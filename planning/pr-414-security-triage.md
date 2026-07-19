---
document_type: security-triage
producer: security-reviewer-via-orchestrator
date: 2026-07-18
subject: "PR #414 ArcavenAE/wirerust ci/scorecard-guard → develop"
verdict: SAFE-WITH-CHANGES
---

# PR #414 Security Triage — OSSF Scorecard Workflow (ArcavenAE fork)

PR: #414, external fork arcaven/wirerust:ci/scorecard-guard → Zious11/wirerust:develop.
78 additions, 1 file: `.github/workflows/scorecards.yml`.
Adds the OSSF Scorecard supply-chain analysis workflow, opt-in behind `SCORECARD_ENABLED` repo variable. Same author (arcaven/Michael Pursifull) as PR #407 (triaged SAFE-WITH-CHANGES, D-472, 2026-07-18).

---

## VERDICT: SAFE-WITH-CHANGES (no blocking vulnerabilities; one external verification required before enabling)

---

## Verified claims (not taken on faith)

**Guard behavior — VERIFIED INERT:**
The `if: vars.SCORECARD_ENABLED == 'true'` condition is on the `analysis` job. When the repo variable is absent or any other value, GitHub evaluates the condition to false and the job is skipped. No runner executes meaningful work. This is the identical opt-in pattern used in PR #407 (`SIGNING_ENABLED`). Fork behavior: `vars.*` resolves to the fork's own repository variables; a fork without `SCORECARD_ENABLED=true` never triggers the job.

**SHA pins — ALL 5 REFS ARE 40-CHAR SHA PINNED:**

| Step | ref | SHA chars | version comment |
|------|-----|-----------|-----------------|
| Harden runner | `step-security/harden-runner@9af89fc71515a100421586dfdb3dc9c984fbf411` | 40 ✓ | `# v2.19.4` |
| Checkout | `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` | 40 ✓ | `# v7.0.0` |
| Scorecard | `ossf/scorecard-action@4eaacf0543bb3f2c246792bd56e8cdeffafb205a` | 40 ✓ | `# v2.4.3` |
| Upload artifact | `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | 40 ✓ | `# v7.0.1` |
| Upload SARIF | `github/codeql-action/upload-sarif@8aad20d150bbac5944a9f9d289da16a4b0d87c1e` | 40 ✓ | `# v4.36.2` |

The `actions/checkout` SHA (`9c091bb...`) is **internally confirmed**: it is identical to every checkout step already in `ci.yml`, which the upstream repo has been running in CI. Version comment accuracy for the remaining four actions cannot be verified from the diff alone and requires external resolution (see F1 below).

**No `pull_request_target`:** Confirmed absent. All three triggers (`branch_protection_rule`, `schedule`, `push`) run from base-repo copy; no fork-code-in-privileged-context risk.

**No secrets access:** Confirmed. Zero `secrets.*` references in the file. No environment variables proxying secrets. GITHUB_TOKEN is not referenced.

**No arbitrary code execution in run: steps:** Confirmed. The workflow contains zero `run:` blocks — every step is a `uses:` action reference. No shell injection vectors exist.

**No unexpected egress destinations:** The only external endpoints are GitHub's code-scanning API (upload-sarif step) and OpenSSF's public REST API (scorecard-action publish_results). Both are well-known, documented endpoints for this workflow type. harden-runner egress-policy is `audit` (same as PR #407; see F4 below).

---

## Permissions analysis

Top-level declaration: `permissions: read-all` (least-privilege default). Job-level overrides follow the minimal standard set for OSSF scorecard:

| Permission | Level | Justification |
|------------|-------|---------------|
| `security-events: write` | REQUIRED | Upload SARIF to GitHub code-scanning dashboard |
| `id-token: write` | REQUIRED | OIDC token for publishing results to OpenSSF API (badge) |
| `contents: read` | read | Checkout repo source |
| `actions: read` | read | Scorecard reads action run history |
| `issues: read` | read | GraphQL ListCommits query |
| `pull-requests: read` | read | read-only |
| `checks: read` | read | SAST tool detection |

`contents: write` is absent. No permission escalation beyond what scorecard documents as required. The `id-token: write` grants OIDC token issuance scoped to the Actions environment; its only externally observable effect is authenticating to the OpenSSF API to publish badge/results data.

---

## Findings

**F1 — VERIFY (CWE-494 Download of Code Without Integrity Check — version comment authenticity)**
Location: `scorecards.yml:42,48,52,55,68` (ossf/scorecard-action, step-security/harden-runner, actions/upload-artifact, github/codeql-action/upload-sarif version comments).
The actual code executed is determined solely by the SHA, not the comment — a wrong comment does not change what runs. However, the PR description claims these are authoritative versions, and a comment lying about the version would obscure traceability. The `actions/checkout` SHA is confirmed via internal cross-reference with `ci.yml`. The other four SHAs must be resolved against their upstream GitHub repos before enabling the workflow:
- `ossf/scorecard-action` v2.4.3: verify `https://github.com/ossf/scorecard-action/releases/tag/v2.4.3`
- `step-security/harden-runner` v2.19.4: verify `https://github.com/step-security/harden-runner/releases/tag/v2.19.4`
- `actions/upload-artifact` v7.0.1: verify `https://github.com/actions/upload-artifact/releases/tag/v7.0.1`
- `github/codeql-action/upload-sarif` v4.36.2: verify `https://github.com/github/codeql-action/releases/tag/v4.36.2`
Severity: VERIFY before enabling. Not a blocker for merge, but resolution should be documented before setting `SCORECARD_ENABLED=true`.

**F2 — LOW (CWE-200 Exposure of Sensitive Information)**
Location: `scorecards.yml:57-64` (`publish_results: true`).
When `SCORECARD_ENABLED=true` and the repository is public, scorecard publishes security posture findings (branch protection status, code-review policies, CI pipeline characteristics, contributor reputation scores) to OpenSSF's public REST API. For a public repository this is the intended behavior and enables the OpenSSF badge. For a private repository the action disables it automatically (as noted in the workflow comment). This is opt-in by design but should be documented in any internal runbook for enabling `SCORECARD_ENABLED`.

**F3 — INFO (CWE-400 Uncontrolled Resource Consumption)**
Location: `scorecards.yml:12-13` (`schedule: cron: '20 7 * * 2'`).
The cron trigger is at workflow level; the `if:` guard is at job level. When `SCORECARD_ENABLED` is unset, GitHub still queues a runner weekly to evaluate the condition, then immediately skips the job. This is not a security issue — it consumes a negligible runner slot (seconds) per week with no secrets or data access. Could be eliminated by adding `workflow_dispatch` and moving `schedule` into a conditional trigger, but this is a cosmetic concern, not required.

**F4 — INFO (egress-policy: audit vs. block)**
Location: `scorecards.yml:39-41`.
harden-runner is set to `egress-policy: audit` rather than `block`. This is the same posture as PR #407 and is industry-standard for signing/scanning workflows that contact multiple known endpoints. Not a finding; noted for completeness and consistency with #407's F6.

---

## Action-pin-gate compatibility

PASSES. The action-pin-gate script (`ci.yml:339-433`) scans all `*.yml` files in `.github/workflows/` against `^[0-9a-f]{40}$`. All 5 `uses:` refs in `scorecards.yml` satisfy this regex. No allowlist entry required — scorecard-action is SHA-pinned, not mutable-ref. Adding `scorecards.yml` increases `VALIDATED` count by 5, which also satisfies the positive-coverage assertion.

---

## Changelog gate

`scorecards.yml` lives under `.github/`. The changelog-gate explicitly excludes `.github/` from its trigger set (ci.yml lines 491-493: "process-internal; CI config changes are not product behavior changes"). No CHANGELOG entry required for this PR.

---

## Overlap and interaction with PR #407

No file conflicts. PR #407 adds `{sign-and-publish,backfill-release,sync-upstream,signing-guard}.yml`; PR #414 adds `scorecards.yml` — fully disjoint.

Both PRs use the `vars.*` opt-in guard pattern — they are consistent and additive. If both are adopted, the CI surface has two independent opt-in workflows with no shared state.

PR #407's `signing-guard.yml` contains a Python CWE-77 scanner that scans all workflow files for injection vulnerabilities. It would scan `scorecards.yml` on any push. Since `scorecards.yml` has zero `run:` blocks and no `${{ github.* }}` inline interpolation in run contexts, the scanner would produce zero violations.

Merge order does not matter — neither PR depends on the other.

---

## Required before adoption

1. **External SHA verification (F1 — before setting SCORECARD_ENABLED=true):** Resolve the four unconfirmed version comments against upstream GitHub release tags and document the resolution. The checkout SHA is already confirmed via ci.yml internal cross-reference.
2. **SCORECARD_ENABLED enablement runbook note (F2 — documentation):** Document that enabling `SCORECARD_ENABLED=true` on a public repository will publish security-posture findings to OpenSSF's public API. Not a code change; a process note for whoever flips the variable.

No code changes to `scorecards.yml` itself are required for merge. The workflow is structurally sound and passes all CI gates.

---

## Summary disposition

The workflow follows all repo security policies (SHA-pinned actions, read-all default permissions, no pull_request_target, no secrets, no run: injection surface, job-level opt-in guard). It is more constrained than PR #407 (no shell scripts, no signing flows, single analysis workflow). The only required action before activating the workflow is external SHA verification for four of five action refs, which is a pre-enablement step, not a pre-merge blocker.

VERDICT: **SAFE-WITH-CHANGES** — safe to merge; external SHA verification and enablement documentation required before setting `SCORECARD_ENABLED=true`.

---

## F1 External SHA Verification — 2026-07-18

Method: extracted the exact 40-char SHAs from `gh pr diff 414`, then resolved each against the
upstream repo via the GitHub API (`git/refs/tags/<tag>`), dereferencing annotated tag objects to
their target commit (`git/tags/<sha>`). Latest-release status via `releases/latest`; advisories via
`repos/<owner>/<repo>/security-advisories` with per-vulnerability version-range inspection.

| Action | CLAIMED tag | Pinned SHA | Tag type | ACTUAL commit SHA for tag | VERDICT |
|--------|-------------|-----------|----------|---------------------------|---------|
| ossf/scorecard-action | v2.4.3 | `4eaacf0543bb3f2c246792bd56e8cdeffafb205a` | annotated → `99c09fe…` | `4eaacf0543bb3f2c246792bd56e8cdeffafb205a` | MATCH |
| step-security/harden-runner | v2.19.4 | `9af89fc71515a100421586dfdb3dc9c984fbf411` | lightweight | `9af89fc71515a100421586dfdb3dc9c984fbf411` | MATCH |
| actions/upload-artifact | v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | lightweight | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | MATCH |
| github/codeql-action/upload-sarif | v4.36.2 | `8aad20d150bbac5944a9f9d289da16a4b0d87c1e` | annotated → `1a818fd…` | `8aad20d150bbac5944a9f9d289da16a4b0d87c1e` | MATCH |

(`actions/checkout@9c091bb…` v7.0.0 was already internally confirmed via `ci.yml`; not re-verified here.)

**Latest-release / advisory notes:**
- `ossf/scorecard-action` v2.4.3 — is the current latest release. 0 published advisories.
- `actions/upload-artifact` v7.0.1 — is the current latest release. 0 published advisories.
- `step-security/harden-runner` v2.19.4 — NOT latest (latest is v2.20.0), but current. 5 published
  advisories exist, all patched at ≤ v2.16.0 (egress-policy/DoH/DoT bypass, disable-sudo evasion,
  setup.ts command injection). v2.19.4 post-dates every patched version → NOT AFFECTED.
- `github/codeql-action` v4.36.2 — a valid, recent v4-series tag resolving to the pinned SHA
  (`releases/latest` reports the separate `codeql-bundle-*` tag scheme). 2 published advisories
  (PAT in debug artifacts, patched 3.28.3/2.20.3; token-visibility in CodeQL runner, patched
  bundle-20210304) affect only far-older versions → NOT AFFECTED.

**Sources:** GitHub REST API v3 (`api.github.com/repos/<owner>/<repo>/git/refs/tags/*`,
`git/tags/*`, `releases/latest`, `security-advisories`), queried via authenticated `gh api`
2026-07-18. Tag-ref resolution is authoritative; annotated tags were dereferenced through their
tag object to the target commit.

**F1 STATUS: RESOLVED-CLEAN** — all four unconfirmed version comments are truthful; each pinned SHA
is exactly the commit the claimed release tag points to, and no known advisory affects any pinned
version. Pre-enablement SHA verification (Required-before-adoption item 1) is satisfied.
