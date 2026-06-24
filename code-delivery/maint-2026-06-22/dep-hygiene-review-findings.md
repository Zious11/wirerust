# Review Findings — maint-2026-06-22 dep hygiene PR #304

**PR:** #304 https://github.com/Zious11/wirerust/pull/304
**Branch:** chore/deps-maint-2026-06-22
**Base:** develop @ dd3b069
**Date:** 2026-06-22

---

## Pre-Triage Facts (gathered by pr-manager from direct file inspection)

| Fact | Evidence | Status |
|------|----------|--------|
| rayon zero src/benches/tests usage | grep -r rayon src/ benches/ tests/ → 0 hits | CONFIRMED |
| rayon remains in Cargo.lock | criterion dev-dep (line 333 in lock) is only parent; awk scan confirmed | NOT A BUG |
| No rayon feature flags | grep Cargo.toml features → no rayon references | CONFIRMED |
| All uses: SHA-pinned | 13 non-exempt uses: lines, all 40-char hex SHA, direct read confirmed | CONFIRMED |
| action-pin-gate CI | PASS | CONFIRMED |
| cargo audit CI exit 0 | 0 advisories — rand 0.8.6 clears RUSTSEC-2026-0097 | CONFIRMED |
| number_prefix absent | unit-prefix present, number_prefix absent; ci.yml comment accurate | CONFIRMED |
| wirerust package entry | rayon fully removed from wirerust's dep list in Cargo.lock | CONFIRMED |
| phf_generator | lists rand 0.8.6 (build-dep path intact) | CONFIRMED |
| Production deps | tls-parser, httparse, etherparse, pcap-file — none list rayon | CONFIRMED |
| rand dual presence | 0.8.6 (phf_generator build-dep) + 0.9.4 (proptest) — pre-existing on develop | CONFIRMED |
| Cargo.toml formatting | File ends with newline; [dependencies] → [profile.release] boundary clean | CONFIRMED |
| deny.toml | No rayon/rand/zerocopy skip entries affected; multiple-versions=warn (not deny) | CONFIRMED |
| All 10 CI jobs | PASS | CONFIRMED |

---

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|-------|-----------|---------|
| 1 | 9 (all INFO/ACCEPTED) | 0 | 0 needed | 0 blocking | APPROVE |

---

## Cycle 1 Findings

| ID | Finding | Source | Severity | Blocking? | Disposition |
|----|---------|--------|----------|-----------|-------------|
| F-01 | RUSTSEC-2026-0097 cleared: rand 0.8.6 confirmed clean (cargo audit exit 0, 0 advisories) | security-review | INFO | No | CLOSED — advisory resolved |
| F-02 | continue-on-error: true on audit job — new advisories won't block CI | adversary | INFO | No | ACCEPTED — intentional design, documented in ci.yml lines 124–126; scheduled run catches zero-days |
| F-03 | rayon remains in Cargo.lock (criterion dev-dep, transitive) | adversary | INFO | No | CLOSED — correct behavior; no production dep pulls rayon |
| F-04 | rand 0.8.6 / 0.9.4 dual presence in Cargo.lock | adversary | INFO | No | CLOSED — pre-existing on develop; Deny CI PASSED |
| F-05 | zerocopy 0.8.48→0.8.52 — no RUSTSEC advisories in range | security-review | INFO | No | CLOSED — cargo audit exit 0, Deny CI PASS |
| F-06 | All uses: action pins still 40-char SHA | security+adversary | PASS | — | CONFIRMED — action-pin-gate CI PASS + direct file read |
| F-07 | ci.yml comment loses build-dep/exploit-condition context from old comment | pr-reviewer+adversary | LOW | No | ACCEPTED — advisory ID and resolution date retained; detail loss is minor, not blocking |
| F-08 | deny.toml stale license entries (DEP-002) not addressed | adversary | INFO | No | OUT-OF-SCOPE — explicitly deferred per sweep report |
| F-09 | No CHANGELOG entry for RUSTSEC clearance | adversary | INFO | No | ACCEPTED — chore/ sweep PRs do not add CHANGELOG entries; Keep-a-Changelog for user-visible changes only |

**Blocking findings: 0**

### Cycle 1 Verdicts

| Reviewer | Verdict | Agent ID | Basis |
|----------|---------|----------|-------|
| pr-reviewer | APPROVE | a2f99fe3256f0f3a7 | wirerust Cargo.lock entry: rayon absent confirmed. awk: only criterion (L333) lists rayon. ci.yml L144–148 comment accurate. Cargo.toml formatting clean. 7 findings, all INFO/PASS/NON-BLOCKING. |
| security-reviewer | CLEAN / APPROVE | a42c432f4ac9739d0 | rand 0.8.6 at lock L1014; 0.8.5 fully absent. phf_generator uses rand 0.8.6 (L854). cargo audit exit 0, 0 advisories. All action pins 40-char SHA confirmed. continue-on-error documented at L124-126. No OWASP concerns. 5 findings, all INFO/PASS. |
| adversary | CLEAN | a8e6da81af16dd53f | awk confirms only criterion (L333) lists rayon — zero production deps. rand dual presence pre-existing. deny.toml multiple-versions=warn, no stale entries. ADV-4 (comment context loss, LOW) is the single notable item; non-blocking. 5 findings, max severity LOW. |

---

## CI Gate

| Job | Status |
|-----|--------|
| Action pin gate | PASS |
| Audit | PASS |
| Clippy | PASS |
| Deny | PASS |
| Format | PASS |
| Fuzz build | PASS |
| Help-provenance gate | PASS |
| Semantic PR | PASS |
| Test | PASS |
| Trust-boundary | PASS |

**CI verdict: ALL 10 JOBS GREEN**

---

## Dependency Check (Step 7)

No upstream story dependencies. Standalone maintenance sweep PR. N/A.

## Merge Authorization

auto_merge: false (per maintenance-config.yaml). Awaiting human merge approval.
