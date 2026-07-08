# Security Review — STORY-157

**PR:** #380 — fix(tooling): input-hash empty-inputs + inline-comment handling + hook-divergence docs (STORY-157)
**Branch:** feature/STORY-157-process-gap-codifications
**Reviewed:** 2026-07-08
**Verdict:** APPROVE — 0 CRITICAL, 0 HIGH findings

## Finding Counts

| Severity | Count | New in PR | Pre-existing |
|----------|-------|-----------|-------------|
| CRITICAL | 0 | — | — |
| HIGH | 0 | — | — |
| MEDIUM | 0 | — | — |
| LOW | 2 | 1 | 1 |
| INFO | 2 | 0 | 2 |

## Findings

### SEC-001 (LOW, NEW): Comment-Stripping Enables Previously-Blocked Path Traversal

**CWE:** CWE-22 (Path Traversal)
**OWASP:** A01:2021 — Broken Access Control

The new comment-stripping code in `parse_inputs()` allows a crafted `inputs:` entry such as
`  - ../../etc/shadow  # RETIRED` to resolve outside the repo after the comment is stripped.
Before this PR, the literal comment text appended to the path caused immediate file-not-found.

**Exploitability:** Requires write access to `factory-artifacts` branch; output is only a
7-character hex hash (not file contents); the tool is an internal developer CLI with no
untrusted callers; no CI privilege escalation path.

**Disposition:** ACCEPTED — does not block merge. Proposed follow-up: validate stripped path
is repo-relative and contains no `../` path components before resolution.

### SEC-002 (LOW, PRE-EXISTING): Unanchored repo_root / rel_path Construction

**CWE:** CWE-22 (Path Traversal)

`abs_path = repo_root / rel_path` — Python's `PurePath.__truediv__` silently discards
`repo_root` when `rel_path` is an absolute path. Pre-existing behavior, not introduced by
this PR. SEC-001 mitigation would also fix this.

**Disposition:** PRE-EXISTING — accepted.

### SEC-003 (INFO, PRE-EXISTING): exec() in Test Harness

**CWE:** CWE-95

`exec(compile(...))` in `bin/test_compute_input_hash.py` loads peer script. Already annotated
`# noqa: S102`. No new risk from this PR's additions.

**Disposition:** PRE-EXISTING — accepted.

### SEC-004 (INFO, PRE-EXISTING): Full Paths in Error Messages

**CWE:** CWE-209

Absolute filesystem paths (including developer home directory) emitted in error messages.
Pre-existing behavior.

**Disposition:** PRE-EXISTING — accepted.

## OWASP Top 10 Assessment

| Category | Applicable | Finding |
|----------|-----------|---------|
| A01 Broken Access Control | Partial | SEC-001, SEC-002 (LOW) |
| A02 Cryptographic Failures | No | MD5 documented as drift-detection only |
| A03 Injection | Marginal | SEC-003 pre-existing |
| A04–A10 | No | No auth, no third-party deps, no network I/O |

## Conclusion

No CRITICAL or HIGH findings. Merge not blocked by security review. SEC-001 deferred to
maintenance backlog for follow-up hardening (path validation guard).
