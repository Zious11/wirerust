---
document_type: security-triage
producer: security-reviewer-via-orchestrator
date: 2026-07-18
subject: "PR #407 ArcavenAE/wirerust feat/fork-friendly-release-ops → develop"
verdict: SAFE-WITH-CHANGES
---

# PR #407 Security Triage — Fork-Friendly Release Ops (ArcavenAE fork)
PR: #407, external fork ArcavenAE/wirerust:feat/fork-friendly-release-ops → Zious11/wirerust:develop. 2221 additions, 15 files: .github/workflows/{sign-and-publish,backfill-release,sync-upstream,signing-guard}.yml + .github/local-workflows.txt + Formula/*.rb (5) + packaging/Info.plist + scripts/{check-signing-workflow-injection,create-app,create-dmg,create-pkg}.sh. Adds opt-in macOS signing/notarization, Homebrew tap publishing (5 channels), scheduled fork→upstream sync, and an always-on CWE-77 signing-guard CI workflow.
## VERDICT: SAFE-WITH-CHANGES (no blocking vulnerabilities)
## Verified author claims (not taken on faith):
- All `uses:` 40-char SHA-pinned (passes action-pin-gate). dtolnay/rust-toolchain pinned to master SHA fa04a1451ff1842e2626ccb99004d0195b455a88 — stricter than @stable; NOTE maintainer should confirm that SHA is genuine dtolnay master + honors `toolchain:` input (not verifiable from diff).
- CWE-77 env-binding: all untrusted context expressions in secret-bearing run: blocks are env-bound; only allowlisted values (github.repository/sha/run_id, matrix.*, runner.*) inline. Enforced structurally by the PR's own 524-line YAML-aware Python scanner (yaml.safe_load, fails-closed, self-test); scanner itself safe (no net, read-only).
- No pull_request_target anywhere (signing-guard uses pull_request → runs from base-repo copy, fork cannot substitute scanner).
- Inert-by-default VERIFIED: all sign/publish/backfill jobs gated on vars.SIGNING_ENABLED=='true'; sync gated on vars.SYNC_UPSTREAM_REPO!=''; with no repo vars set, only the read-only signing-guard linter runs. No secrets/publishing without maintainer opt-in.
- Provenance: no unexpected egress; only Apple CA CDN + github.com + Apple notarization; no curl|bash; harden-runner egress-policy: audit on all jobs.
## Findings:
- F1 LOW CWE-200: stable-sign workflow_run trigger has repo-secret access by design; safe as-is but upstream `Release` workflow trigger must stay restricted to protected-branch v* tags (process/doc requirement).
- F2 LOW CWE-77: matrix.* inline interpolation — author-defined literals, allowlisted, no exploit path; no change.
- F3 LOW: scripts/create-app.sh:236 unquoted $VERSION in sed (metachar hygiene) — escape before any user-supplied version; REQUIRED hygiene fix.
- F4 INFO governance: bundle id com.arcavenae.wirerust hardcoded (Info.plist:8, create-pkg.sh:306) — change to upstream reverse-domain before enabling signing.
- F5 INFO: sync-upstream Sync-Tags step unconditional within the (var-gated) sync job — optional if-guard.
- F6 INFO: harden-runner egress-policy: audit (not block) — industry standard for signing; optional hardening.
## Required-before-merge (if adopted): (1) confirm/doc upstream Release trigger restricted to protected v* tags; (2) sed-escape $VERSION in create-app.sh; (3) resolve com.arcavenae.wirerust bundle id (placeholder or upstream domain).
## DISPOSITION: DEFERRED by human (2026-07-18). PR left OPEN, no action. Governance question (does upstream want fork release-ops infra) unresolved. This triage preserved so security review is NOT re-done on resume.
