# Dependency Audit Findings — Maintenance Sweep 1 (ANALYSIS phase)

- **Date:** 2026-06-22
- **Crate version audited:** wirerust v0.9.3 (193 locked dependencies)
- **Raw scan source:** `.factory/maintenance/dependency-audit-raw.log`
- **Reviewer:** security-reviewer agent
- **Prior sweep reference:** `.factory/maintenance/dependency-audit.md` (sweep dated 2026-06-17, v0.7.1)

---

## Findings Table

| Finding ID | Advisory / CWE / CVE | Dependency | Severity | CLI-Reachable | Recommended Action | Auto-Fixable |
|---|---|---|---|---|---|---|
| DEP-001 | RUSTSEC-2026-0097 / CWE-119 (soundness) | rand 0.8.5 (transitive build-dep) | LOW | NO | `cargo update -p rand` (bumps to 0.8.6); remove `--ignore RUSTSEC-2026-0097` from CI after update | YES — patch-only semver bump |
| DEP-002 | N/A (supply-chain hygiene) | deny.toml — 8 stale license-not-encountered entries | LOW (informational) | NO | Optional: prune `allow` list to only actually-used licenses; run `cargo deny list` first | NO — requires manual inspection |
| DEP-003 | N/A (ecosystem migration pattern) | syn v1.0.109 + v2.0.117 duplicate | LOW (informational) | NO | No action; upstream (tls-parser, pcap-file) must migrate proc-macro chains to syn 2.x | NO — upstream dependency |
| DEP-004 | CWE-1104 (Use of Unmaintained Third-Party Components — partial analogy) | 35 crates with available patch/minor updates | LOW | NO | Full `cargo update`; validate CI green; shlex 2.0.1 major bump requires explicit verification (build dep only) | YES for 34/35; NO for shlex 2.0.1 major |
| DEP-005 | CWE-119 (precautionary — prior RUSTSEC-2023-0074 soundness history) | zerocopy 0.8.48 → 0.8.52 available | MEDIUM (precautionary) | NO | Upgrade via `cargo update` — no active advisory against 0.8.48 but zerocopy has a track record of post-release soundness fixes in sub-patches | YES — included in full `cargo update` |
| DEP-006 | N/A — dead direct production dependency | rayon v1.12.0 (Cargo.toml [dependencies], line 37) | LOW (tech debt) | NO | Remove `rayon = "1"` from `[dependencies]` in Cargo.toml — no usage in src/; tech debt item O-07 | YES — one-line Cargo.toml deletion + `cargo update` |

---

## Detailed Analysis

### DEP-001 — RUSTSEC-2026-0097: rand 0.8.5 unsound (transitive, build-dep only)

**Advisory:** RUSTSEC-2026-0097 (published 2026-04-09)
**Title:** "Rand is unsound with a custom logger using `rand::rng()`"
**CWE:** CWE-119 — Improper Restriction of Operations within Bounds of a Memory Buffer (soundness / undefined-behaviour category)
**OWASP:** Not applicable (not an application-layer vulnerability)

**Dependency chain:**
```
rand 0.8.5
└── phf_generator 0.11.3
    └── phf_codegen 0.11.3
        └── tls-parser 0.12.2   [build-dependency only — code generation step]
            └── wirerust 0.9.3
```

**Reachability / Exploitability Assessment:**

The unsound condition requires two simultaneous conditions:
1. A custom global logger that calls `rand::rng()` from within a log macro invocation (i.e., the `log` crate's global logger is user-supplied and invokes rand internally).
2. A concurrent thread racing on rand's internal state during a reseed operation.

wirerust satisfies NEITHER condition:
- rand 0.8.5 is consumed exclusively at **build time** by phf_codegen, which uses it to generate static perfect-hash tables. The rand crate does not appear in the production binary's dependency tree — it is a build-script tool, not a runtime crate.
- wirerust does not install a custom global logger; it uses indicatif and owo-colors for terminal output, neither of which routes through `log` or calls `rand::rng()`.
- The produced binary has no code path that touches rand at runtime.

**Verdict: NOT REACHABLE. Severity is LOW (advisory class is `unsound`, not a CVSS-scored CVE). Runtime exploitability is NIL.**

**Prior disposition:** ACCEPTED-TRANSITIVE since F6 hardening cycle. CI suppresses via `--ignore RUSTSEC-2026-0097`.

**New status for v0.9.3 sweep:** rand 0.8.6 is available as a semver-compatible patch bump and resolves the advisory. The fix is now trivially self-serviceable via `cargo update -p rand`. Recommend applying in the next patch batch and removing the CI suppression.

---

### DEP-002 — Stale deny.toml license allowlist (8 license-not-encountered warnings)

**Severity:** LOW (informational, no security implication)
**CWE:** Not applicable
**OWASP:** Not applicable

Eight entries in `deny.toml` `[licenses] allow` are broader than the current dependency graph requires. No crate uses these licenses today: `0BSD`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `CC0-1.0`, `ISC`, `MPL-2.0`, `Unicode-DFS-2016`, `Zlib`.

**Security implication:** None. Overly broad allowlists are a hygiene concern — a future dependency using one of these licenses would be silently accepted — but all eight are permissive licenses compatible with MIT. The risk of inadvertently accepting a copyleft or hostile license via this vector is minimal.

**Recommendation:** Defer to next license-audit sweep. When pruning, use `cargo deny list` to enumerate actually-encountered licenses first; removing a license that a transitive dep silently uses would break `cargo deny check`.

---

### DEP-003 — syn v1/v2 duplicate

**Severity:** LOW (informational, expected ecosystem pattern)
**CWE:** Not applicable
**OWASP:** Not applicable

syn 1.0.109 is pulled by `derive-into-owned` (via pcap-file) and `nom-derive-impl` (via nom-derive / tls-parser). syn 2.0.117 is pulled by clap, serde, thiserror, zerocopy, and wasm-bindgen chains. This is the standard Rust ecosystem proc-macro migration pattern — no security concern; no advisory against either version.

**Recommendation:** No action. Resolution is upstream's responsibility (tls-parser, pcap-file). syn 2.0.118 would arrive automatically with `cargo update`.

---

### DEP-004 — 35 crates with available patch/minor updates

**Severity:** LOW (supply-chain freshness)
**CWE:** CWE-1104 (Use of Unmaintained Third-Party Components — partial analogy for stale patch versions)

35 crates have patch or minor updates available. Most are routine. Notable items:
- `rand 0.8.5 → 0.8.6` — resolves DEP-001
- `zerocopy 0.8.48 → 0.8.52` — see DEP-005
- `shlex 1.3.0 → 2.0.1` — major version bump; build dep only (via `cc` crate); verify CI green before accepting
- `syn 2.0.117 → 2.0.118` — safe proc-macro chain patch
- `wasm-bindgen 0.2.117 → 0.2.125` — dev-dep only (criterion/plotters chain)

**Recommendation:** Run `cargo update` and `cargo test --all-targets` to confirm no regressions. shlex 2.0.1 is the only item requiring CI-green confirmation before acceptance.

---

### DEP-005 — zerocopy 0.8.48 (precautionary; no active advisory)

**Severity:** MEDIUM (precautionary — no active CVE)
**CWE:** CWE-119 — Improper Restriction of Operations within Bounds of a Memory Buffer (memory safety concern; established by prior RUSTSEC-2023-0074 in this crate's history)

zerocopy 0.8.52 is available. cargo audit reports **zero active advisories** against zerocopy 0.8.48 as of this scan. This is a **precautionary** finding based on zerocopy's demonstrated pattern of discovering post-release soundness issues in sub-patches (most notably RUSTSEC-2023-0074, since resolved). A gap of four patch versions (.48 → .52) in a crate with this history warrants proactive updating.

**Exploitability:** Not currently exploitable — no active advisory exists against 0.8.48. Upgrading is prudent.

**Recommendation:** Include in `cargo update` batch. HIGH priority within that batch given zerocopy's soundness history.

---

### DEP-006 — rayon: dead direct production dependency (tech debt O-07)

**Severity:** LOW (tech debt — dead code in dependency tree; no security attack surface created by an unused dep)
**CWE:** Not directly applicable. CWE-561 (Dead Code) is the closest analogue for unnecessary dependency inclusion.
**OWASP:** Not applicable (no runtime attack surface)

**Finding:** `rayon = "1"` appears at line 37 of `Cargo.toml` in the `[dependencies]` section (production dependencies, not `[dev-dependencies]`). The resolved version is rayon v1.12.0.

**Usage investigation:**
- `grep -rn "rayon\|par_iter\|par_bridge\|ParallelIterator\|into_par_iter\|par_chunks\|par_extend\|rayon::" src/` — **zero matches**
- `grep -rn "rayon" benches/ tests/` — **zero matches**
- The cargo tree shows rayon is also pulled in by criterion (dev-dependency), but criterion's pull of rayon is independent of the Cargo.toml direct declaration.

**Verdict: DEAD DIRECT PRODUCTION DEPENDENCY. rayon has zero usage in `src/` and has been confirmed unused across multiple prior release cycles (v0.5.0, v0.6.0, v0.7.x per tech-debt-check.md entry O-07). The direct dependency declaration in `[dependencies]` adds rayon to the production binary link graph unnecessarily.**

**Context from tech-debt-check.md (O-07):** Issue #6 ("Add parallel file processing with rayon") is the intended future consumer. If issue #6 is not on the near-term roadmap, the dependency should be removed. If it IS planned, the entry should be moved to a `# Planned: issue #6` comment with the dep removed until actually implemented.

**Auto-fixable:** YES — single-line deletion from Cargo.toml (`rayon = "1"` at line 37). Low-risk change; rayon will still be transitively present via criterion in the dev build, so benchmarks are unaffected. The production binary drops the rayon link.

**Recommended action:** Remove `rayon = "1"` from `[dependencies]`. If issue #6 is actively planned, add a code comment above the bench that links the intent. The dx-engineer or implementer can execute this as a `chore:` PR.

---

## O-07 Disposition (Tech Debt Resolution)

Tech debt item **O-07** (`rayon` declared in `[dependencies]`, zero usage in `src/`) is **confirmed as a genuine, auto-fixable finding**.

| Attribute | Value |
|---|---|
| Item | O-07 |
| Status | OPEN — confirmed genuine |
| Auto-fixable | YES (one-line Cargo.toml deletion) |
| Complexity | Trivial — no code changes required, only Cargo.toml edit |
| Risk | LOW — rayon remains available transitively via criterion; benches unaffected |
| Blocker | None — safe to file as a `chore:` PR immediately |

---

## Overall Verdict

**Highest severity:** MEDIUM (DEP-005 — zerocopy precautionary; no active CVE)

**CRITICAL findings:** 0
**HIGH findings:** 0
**MEDIUM findings:** 1 (DEP-005 — precautionary, no active advisory)
**LOW findings:** 5 (DEP-001 through DEP-004, DEP-006)

**No CRITICAL or HIGH findings. No finding triggers an immediate fix PR requirement. The audit is NON-BLOCKING for continued development.**

The only recommended-priority actions are:
1. `cargo update -p rand` (resolves DEP-001; low-risk patch bump; enables CI suppression removal)
2. `cargo update` for the full batch including zerocopy (DEP-004/DEP-005)
3. Remove `rayon = "1"` from Cargo.toml `[dependencies]` (O-07; one-line chore PR)
