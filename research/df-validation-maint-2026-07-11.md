# DF-VALIDATION-001 Validation Pass — maint-2026-07-11

**Policy:** DF-VALIDATION-001 (no GitHub issue action from unvalidated findings).
**Validated against:** develop tree at commit `b5e1e15` (b5e1e155e37704296a8cb5951743cd5817a3f11d).
**Validator:** research pass, maint-2026-07-11.

Every claim below carries a `file:line` citation read directly from the current tree.

---

## Item 1 — ISSUE-102-PREMATURE-CLOSE-001 (P2)

**Register claim:** GitHub #102 was closed COMPLETED 2026-06-08 asserting a
`MAX_WEAK_CIPHER_EVIDENCE` cap was implemented, but risk-register R-001 asserts the cap is
absent from `src/` and remains OPEN. R-001 states: *"No `MAX_WEAK_CIPHER_EVIDENCE` truncation
cap exists"* and *"Source-code grep confirms no `MAX_WEAK_CIPHER_EVIDENCE` cap exists in
`src/` as of maint-2026-07-09 backfill validation — fix was never implemented."*
(`.factory/specs/domain-spec/risk-register.md:66`, `:75-78`).

### Verdict: **REFUTED** — the cap EXISTS in the current develop tree.

### Evidence

**(a) The cap is present in `src/`, under a different identifier name.**
The exact string `MAX_WEAK_CIPHER_EVIDENCE` is absent from `src/` and `tests/` (grep exit 1).
But the functional cap the register describes is present in the ClientHello weak-cipher path:

- `src/analyzer/tls.rs:635` — `const WEAK_CIPHER_EVIDENCE_CAP: usize = 64;`
- `src/analyzer/tls.rs:641` — `.take(WEAK_CIPHER_EVIDENCE_CAP)` bounds the evidence Vec.
- `src/analyzer/tls.rs:644-645` — `if total_weak > WEAK_CIPHER_EVIDENCE_CAP { weak.push(format!("(+{} more)", ...)); }` — the "+N more" elision marker.
- `src/analyzer/tls.rs:648-661` — this bounded `weak` Vec is the `evidence` field of the
  ClientHello weak-cipher `Finding` — exactly the finding R-001 names.

This is precisely the mitigation R-001 itself prescribes at `risk-register.md:70`:
*"Add `MAX_WEAK_CIPHER_EVIDENCE = 64` cap with `"+N more"` annotation."* Same cap value (64),
same elision marker, same finding — only the constant name differs
(`WEAK_CIPHER_EVIDENCE_CAP`, not the proposed `MAX_WEAK_CIPHER_EVIDENCE`).

**(b) The fix landed the same day #102 was closed — closure was NOT premature.**
- Fix commit: `d22b9fe` (2026-06-08 14:24:57 -0500), *"fix(tls): cap weak-cipher evidence
  vec at 64 with elision marker (#102 hardening)"*.
- #102 closed COMPLETED 2026-06-08 — the same calendar day the fix was committed. The closure
  matches the fix, not a premature/administrative close.
- The cap has been continuously present through HEAD (`git grep` at `HEAD` returns the
  constant at `tls.rs:630,635,641,644,645`; commit `c4eb1f4` on 2026-07-08 restructured the
  file but preserved the cap).

**(c) Root cause of the register's false claim.**
The maint-2026-07-09 backfill grepped for the *proposed* identifier `MAX_WEAK_CIPHER_EVIDENCE`
(the name in the mitigation text), not the *shipped* identifier `WEAK_CIPHER_EVIDENCE_CAP`. The
grep returned nothing and the author concluded "fix never implemented." This is a false negative
from searching the wrong symbol name; the fix had in fact shipped a month earlier (2026-06-08).

### Recommended disposition

- **Do NOT re-open #102 and do NOT file a fresh issue.** The cap exists and #102 was closed
  correctly, same-day as the landing fix (`d22b9fe`).
- **Correct risk-register R-001** (`risk-register.md:54-81`): its status is factually wrong
  against `b5e1e15`. Recommend transitioning R-001 `open → resolved`, citing `d22b9fe` and
  `src/analyzer/tls.rs:635`, and correcting the "no cap exists / fix never implemented" text
  and the grep note at `:66,:75-78`. Note the constant name is `WEAK_CIPHER_EVIDENCE_CAP`.
- This is a spec-artifact (register) correction, not a GitHub-issue action; no issue is
  created. NFR-RES-023 should be re-checked against the shipped cap.

---

## Item 2 — CHANGELOG-D3-T0830-DRIFT-001 (LOW)

**Register claim:** A CHANGELOG released-section entry (register cited ~line 670) says a finding
class is "Attributed to T0830", but the emitting code emits `mitre_techniques: []`.

### Verdict: **CONFIRMED** — drift is real.

### Evidence

**The drifted entry is the D3 ARP-storm class** (the register's ~line 670 is stale; the
current location is line 806). The only T0830 references in CHANGELOG.md are at lines 552,
801, and 806:

- `CHANGELOG.md:805-806` (inside the released `## [0.7.0] - 2026-06-16` section, header at
  `:793`): *"**D3 ARP storms** — high-rate ARP flood detection ... Attributed to **T0830**."*

**The emitting code emits an empty technique list.** The D3 storm `Finding` is constructed in
`ArpAnalyzer::detect_storm`:

- `src/analyzer/arp.rs:1050-1073` — the storm `Finding` (summary "D3: ARP storm detected …").
- `src/analyzer/arp.rs:1068-1069` — `// T0814 withheld per DF-VALIDATION-001 / BC-2.16.008 Invariant 3.` followed by `mitre_techniques: vec![],`.

Corroborating in-code confirmation that D3 storm carries `[]`:
- Module doc `src/analyzer/arp.rs:15-16`: *"D3 storm detection (BC-2.16.008) emits
  MEDIUM/Anomaly findings with `mitre_techniques: []` (T0814 withheld per DF-VALIDATION-001)."*
- Method doc `src/analyzer/arp.rs:1008`.
- Tests `src/analyzer/arp.rs:4104,4130-4141` (`test_d3_finding_has_empty_mitre_techniques`)
  and `:4224-4229` assert D3 findings have empty `mitre_techniques` and must NOT contain T0814.

**Scope note — only the D3 line is drifted.** The adjacent D1 entry at `CHANGELOG.md:801`
("D1 ARP spoofing … Attributed to **T0830 Adversary-in-the-Middle** and **T1557.002**")
matches code exactly: `src/analyzer/arp.rs:906` emits
`mitre_techniques: vec!["T0830", "T1557.002"]`. Line 552 is a tactic-reclassification note,
not a finding-class attribution. So the drift is isolated to the D3 storm entry at line 806.

Note the CHANGELOG cites **T0830**, whereas the code comment discusses withholding **T0814** —
the two texts do not even reference the same technique, but the result is unambiguous: the D3
storm finding emits `[]` while the released CHANGELOG claims a T0830 attribution.

### Recommended disposition

- **LOW severity, documentation-only, in a released section.** The entry sits in the shipped
  `[0.7.0]` history (`CHANGELOG.md:793`), which is normally immutable historical record.
- Recommend a minimal **CHANGELOG errata/correction**: strike or footnote the "Attributed to
  **T0830**" clause on the D3 ARP-storm line (`:806`) to reflect that D3 storm findings carry
  `mitre_techniques: []` (T0814 withheld per DF-VALIDATION-001 / BC-2.16.008 Invariant 3). No
  code change; the code is the intended behavior.
- If filed, this is a `docs`-type fix; the drift is validated here per DF-VALIDATION-001.

---

## Summary

| Item | Verdict | Key evidence | Disposition |
|------|---------|--------------|-------------|
| 1 — ISSUE-102-PREMATURE-CLOSE-001 | **REFUTED** | cap present at `src/analyzer/tls.rs:635` (`WEAK_CIPHER_EVIDENCE_CAP = 64`); fix `d22b9fe` landed 2026-06-08, same day #102 closed | No re-open, no new issue; correct risk-register R-001 (open→resolved) — register grepped the wrong symbol name |
| 2 — CHANGELOG-D3-T0830-DRIFT-001 | **CONFIRMED** | `CHANGELOG.md:806` says D3 storms "Attributed to T0830"; `src/analyzer/arp.rs:1069` emits `mitre_techniques: vec![]` | LOW; optional docs errata on released `[0.7.0]` CHANGELOG line; no code change |

---

## Sweep 1 follow-up: dependency maintenance cadence

Offline dependency analysis marked `pcap-file`, `tls-parser`, and `nom-derive` INCONCLUSIVE
for maintenance status. Checked each against crates.io (registry API) and its source repo
(GitHub API) on 2026-07-11. Locked versions per `Cargo.lock`: `pcap-file 2.0.0`,
`tls-parser 0.12.2`, `nom-derive 0.10.1`. "Months ago" are relative to 2026-07-11.
Rubric: ACTIVE / SLOW-BUT-MAINTAINED / STALE (>18 months no release) / ABANDONED.

### pcap-file — **ACTIVE**

- **Direct dependency** (`Cargo.toml:29`, `pcap-file = "2"`; locked at 2.0.0).
- Repo: https://github.com/courvoif/pcap-file (not archived).
- Latest stable release: **2.0.0, 2023-02-01**; but a **3.0.0-rc.2 pre-release on 2026-05-06**
  (~2 months ago) shows active development toward a 3.0 line.
- Latest commit: **2026-05-08** ("Merge PR #65 … linktypes-to-303"), repo `pushed_at`
  2026-05-31 (~1.5 months ago).
- Backlog: 2 open issues, 1 open PR — low and current.
- **Verdict: ACTIVE.** Recent commits, an in-flight 3.0 RC, and a small backlog. The pinned
  2.0.0 is stable/older but the project is clearly maintained. (Sources: crates.io API
  `/crates/pcap-file`; GitHub API `repos/courvoif/pcap-file`, retrieved 2026-07-11.)

### tls-parser — **SLOW-BUT-MAINTAINED**

- **Direct dependency** (`Cargo.toml:18`, `tls-parser = "0.12"`; locked at 0.12.2).
- Repo: https://github.com/rusticata/tls-parser (not archived); maintained by the rusticata
  org (nom-based security parsers used by Suricata).
- Latest release: **0.12.2, 2024-09-09** — **~22 months ago**, which *does* cross the
  >18-month "no release" STALE threshold on release cadence alone.
- Latest commit: **2025-08-13** ("cargo update … closes #91"), repo `pushed_at` 2025-11-24
  (~8 months ago) — commit activity continued well after the last release.
- Backlog: 13 open issues, 10 open PRs.
- **Verdict: SLOW-BUT-MAINTAINED.** Release cadence has crossed the 18-month line, but the
  repo received maintenance commits within the last ~11 months and the maintainer org is
  active, so it is functionally maintained rather than stale/abandoned. Flag for re-check at
  the next sweep: if no release/commit lands by ~2026-Q4 it slides toward STALE. (Sources:
  crates.io API `/crates/tls-parser`; GitHub API `repos/rusticata/tls-parser`, 2026-07-11.)

### nom-derive — **STALE** (not abandoned)

- **Transitive dependency only** — pulled in by `tls-parser` (`Cargo.lock:1315`); not a
  direct entry in `Cargo.toml`. Locked at 0.10.1.
- Repo: https://github.com/rust-bakery/nom-derive (not archived); rust-bakery is the nom
  parser org.
- Latest release: **0.10.1, 2023-03-20** — **~28 months ago**, well past the 18-month STALE
  threshold.
- Latest commit: **2025-07-29** ("Fix clippy (stable) warnings"), repo `pushed_at` same date
  (~12 months ago) — occasional maintenance only.
- Backlog: 14 open issues, 0 open PRs.
- **Verdict: STALE.** No release in ~28 months meets the STALE definition. It is *not*
  ABANDONED — the repo received a maintenance commit ~12 months ago and the org is reputable —
  but releases have effectively stopped. Low practical risk here: it is transitive, its
  surface is a proc-macro used at build time, and its version is governed by `tls-parser`'s
  choice rather than ours. (Sources: crates.io API `/crates/nom-derive`; GitHub API
  `repos/rust-bakery/nom-derive`, 2026-07-11.)

### Follow-up summary

| Crate | Locked | Latest release | Last commit | Verdict |
|-------|--------|----------------|-------------|---------|
| pcap-file (direct) | 2.0.0 | 3.0.0-rc.2 (2026-05-06); stable 2.0.0 (2023-02-01) | 2026-05-08 | **ACTIVE** |
| tls-parser (direct) | 0.12.2 | 0.12.2 (2024-09-09, ~22 mo) | 2025-08-13 | **SLOW-BUT-MAINTAINED** |
| nom-derive (transitive via tls-parser) | 0.10.1 | 0.10.1 (2023-03-20, ~28 mo) | 2025-07-29 | **STALE** (not abandoned) |

No dependency changes made; no commits. `cargo audit` was clean as of the maint-2026-07-09
sweep (risk-register R-007 resolved), so none of these currently carry an open RUSTSEC advisory.
