# PR #420 Review — build(deps): soaked dependency bumps 2026-07-19

**Verdict: APPROVE** — no blocking findings. Two non-blocking description-accuracy
discrepancies in the PR body (the actual lockfile diff is clean, forward-only,
and verified).

Branch: `chore/dep-soak-sweep-2026-07-19` → `develop`. Only file changed: `Cargo.lock`.

---

## 1. Diff coherence — PASS
- Only `Cargo.lock` changed (54 insertions / 249 deletions). No `Cargo.toml`, no `src/`, no `bin/`.
- **0 package blocks added** — no unexpected crates entered the tree.
- **All 24 version-line changes go forward** — no regression to an older version.
- Package count verified independently: develop **193** → HEAD **175** = net **−18**, matching the claim.

## 2. Spot-verified bumps (4 checked against crates.io publish dates; soak measured to 2026-07-19)

| Crate | Old removed | New present | Published | Soak claimed / actual | Yanked? |
|-------|-------------|-------------|-----------|-----------------------|---------|
| etherparse | 0.20.2 ✓ | 0.20.3 ✓ | 2026-07-04 | 15 / **15 exact** | no |
| libc | 0.2.184 ✓ | 0.2.186 ✓ | 2026-04-23 | 87 / **87 exact** | no |
| zerocopy | 0.8.52 ✓ | 0.8.54 ✓ | 2026-07-08 | 11 / **11 exact** | no |
| crossbeam-deque | 0.8.6 ✓ | 0.8.7 ✓ | 2026-07-06 | 13 / **13 exact** | no |

All four: old version removed, new version present, ≥8-day soak satisfied (D-417),
not yanked. Soak-day arithmetic is precise, which raises confidence in the
un-spot-checked rows.

## 3. Removed-crate list — count TRUE, enumeration imprecise (NIT, non-blocking)
- Diff removes exactly **18 package blocks**; net dependency reduction is **18** (193→175). Headline count accurate.
- Enumeration imperfections in the body:
  - **`wit-bindgen 0.51.0` is listed as removed but was NOT removed** — it remains
    in the lock; only its `dependencies = ["wit-bindgen-rust-macro"]` edge was
    dropped. Its 3 sub-crates (`wit-bindgen-core`/`-rust`/`-rust-macro`) were
    genuinely removed.
  - The enumerated list names **17** items while claiming 18; two actual removals
    (`getrandom 0.4.2` — described only as the resolution "driver" — and
    `hashbrown 0.15.5` — folded into the bump table) are not in the enumerated set.
  - Cosmetic labeling only; the actual 18 removed blocks and −18 net are correct.

## 4. Bump count — description discrepancy (SUGGESTION, non-blocking)
- Body header claims **"26 crate versions bumped"**, but the diff contains **24
  version-line changes** and the body's own table has **24 rows**. Per
  PG-W74-PRDESC-ROW-VERIFY the claimed aggregate (26) does not match actual output
  (24). Recommend correcting the header to 24. Does not affect merge safety.

## 5. Security justification — SOUND
For a lockfile-only bump with no source changes, the security surface is entirely
"which exact versions are locked." `cargo audit` (RUSTSEC advisories against locked
versions) plus `cargo deny check` (advisories + bans + licenses + sources) are
precisely the right and sufficient evidence. Reported results (0 advisories,
175 deps, all four deny checks OK) are the correct gates. Results are self-reported
in the PR body; CI re-runs them on push, which is the appropriate backstop.

## 6. CHANGELOG exemption — ACCURATE (verified against CI, not just docs)
Confirmed against `.github/workflows/ci.yml` changelog-gate: trigger regex is
`^(src/|Cargo\.toml$|bin/)`. `Cargo.toml$` is anchored, so `Cargo.lock` does not
match, and there is no `src/`/`bin/` change. A Cargo.lock-only PR does not trip the
gate. Exemption per AC-158-001 is correct.

---

## Finding summary

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| suggestion | description | Body claims "26 crate versions bumped"; diff + body table show 24. | Correct header to 24. |
| nit | description | `wit-bindgen 0.51.0` listed as removed but remains in lock (only its dep edge dropped); removed list enumerates 17 while claiming 18. | Re-word: 3 wit-bindgen sub-crates removed, parent retained; reconcile enumeration with the true 18 removed blocks. |

No blocking findings. Lockfile change is coherent, forward-only, adds no crates,
regresses no versions; all spot-checked soak claims exact; security evidence sound;
changelog exemption accurate. Recommend merge after CI green (a one-line body
correction for the two description nits is optional, not required).
