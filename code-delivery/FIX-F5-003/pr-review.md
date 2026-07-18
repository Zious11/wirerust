# PR #413 Review — `docs(FIX-F5-003): correct fabricated demo-evidence JSON across IEC-104 feature tree`

**Verdict: APPROVED** · Docs-only · No security review required (no source/behavioral change)

Branch: `fix/FIX-F5-003` → `develop`. All checks verified against the PR branch
(`origin/fix/FIX-F5-003`) via `git show`, since the review worktree is checked out on
`develop` (pre-merge).

## Source of truth (enum definitions)

- `src/findings.rs:100` `ThreatCategory` — Reconnaissance, LateralMovement, C2, Exfiltration,
  CredentialAccess, Persistence, Execution, **Anomaly**, Suspicious, **Impact**. No `Protocol`.
  serde `rename_all = "snake_case"` → JSON `"impact"`, `"anomaly"`.
- `src/findings.rs:33` `Verdict` — Likely, Unlikely, Inconclusive, **Possible**. No `Anomaly`.
  serde `rename_all = "lowercase"` → JSON `"possible"`.
- `src/findings.rs:68` `Confidence` — **High**, **Medium**, Low. `High` is a real Rust variant;
  serde `rename_all = "lowercase"` → JSON `"high"`/`"medium"` (old docs' `"High"` JSON casing was
  the fabrication).
- `src/reassembly/handler.rs:25` `Direction` — `ClientToServer`, `ServerToClient`; no serde rename
  → JSON PascalCase `"ClientToServer"`/`"ServerToClient"`.

## Check results

| # | Check | Verdict |
|---|-------|---------|
| 1 | Docs-only (zero `.rs` source changes) | PASS |
| 2 | Row-verify JSON/enum values vs `findings.rs` (PG-W74) | PASS |
| 3 | Zero residual fabrications in FIX-P4-001 | PASS |
| 4 | CHANGELOG scope-claim accuracy | PASS |
| 5 | Tree-wide enum sweep | PASS |

**Check 1 — Docs-only:** `git diff origin/develop...origin/fix/FIX-F5-003 --name-only` =
`CHANGELOG.md` + `docs/demo-evidence/FIX-P4-001/{AC-P4-001-test-results.txt,
demo-json-serialization.rs, evidence-report.md}`. Zero files under `src/`, `tests/`, `bin/`.
The `.rs` file lives under `docs/demo-evidence/`, not source.

**Check 2 — Row-verify (PG-W74):** All corrected values map to real variants at real emit sites:
- Example 1 (N(S) desync, C2S): `ThreatCategory::Impact` / `Verdict::Possible` /
  `Confidence::Medium` / `Direction::ClientToServer` / `T1692.001` → JSON
  `"impact"`/`"possible"`/`"medium"`/`"ClientToServer"`.
- Example 2 (malformed LEN, S2C): `ThreatCategory::Anomaly` / `Possible` / `Medium` /
  `ServerToClient` / `T0814` → JSON `"anomaly"`/`"possible"`/`"medium"`/`"ServerToClient"`.
- Example 3 (carry overflow, None direction): `Anomaly`/`Possible`/`Medium` / `T0814`; optional
  fields None → keys omitted via `skip_serializing_if`.
- MITRE: N(S)-desync = **T1692.001** (not T0881); malformed-LEN and carry-overflow = **T0814**.
  `Direction` import path corrected to `wirerust::reassembly::handler::Direction`.

**Check 3 — Residual fabrications (PR branch):** `"Protocol"`/`"Anomaly"`/`"High"`/`T0881` — all
three FIX-P4-001 files CLEAN on `origin/fix/FIX-F5-003`.

**Check 4 — CHANGELOG:** New `[Unreleased] → Fixed` FIX-F5-003 entry present and accurately scoped
(docs + CHANGELOG only). FIX-F5-002 entry corrected from "FIX-F5-001 and FIX-P4-001 evidence
artifacts" to "FIX-F5-001 evidence artifacts only" (`CHANGELOG.md:74`); FIX-F5-003 entry documents
the correction (`CHANGELOG.md:43-47`). Satisfies AC-158-001 CHANGELOG gate.

**Check 5 — Tree-wide sweep:** Only hit across all `docs/demo-evidence/` is
`docs/demo-evidence/STORY-160/evidence-report.md:85`, asserting `Confidence::High → "HIGH"` and
`ThreatCategory::LateralMovement.to_string() → "LateralMovement"` — a legitimate, correct reference
to real Rust variants and their `Display` output (not fabricated JSON). Out of scope; not a defect.

## Findings

None (no blocking, suggestion, or nit).

Optional observation (non-blocking): the CHANGELOG parenthetical at `CHANGELOG.md:15`
("`Confidence::High` → serde gives \"High\" but real serde output is \"high\"") is slightly awkward —
`Confidence::High` is a real variant; only the JSON casing `"High"` was fabricated. Intent is
accurate; no change required.

## Overall verdict: APPROVED

Docs-only, correctly scoped, all fabricated enum variants and misattributed MITRE techniques
corrected against real emit sites, CHANGELOG scope claims accurate, and zero residual fabrications
on the PR branch.
