# Pass 0: Inventory -- wirerust -- Deepening Round 2

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 0 (Inventory) -- Phase B deepening, round 2
- **Methodology:** Mechanical re-counting per Pass 6 sec. 7 P0 plan. Same `find`/`wc`/`awk` discipline P3 R3 applied to fix Pass 6 BC arithmetic.
- **Inputs:**
  - P0 R1 broad-sweep (`wirerust-pass-0-inventory.md`)
  - Pass 6 sec. 7 P0 plan (lines 339-348)
  - Cross-pollination signal from P3 R3 (Pass 6 BC count was 137 stated, 216 actual -- inflation)

---

## 1. Five Hallucination Classes Audit of P0 R1

| Class | Severity | Finding |
|---|---|---|
| **Metric inflation** | CONFIRMED (under-count, not over-count) | P0 R1 sec. 7 said "0 inline `#[test]` in `src/`" -- actual is **11** in `src/reporter/terminal.rs` (lines 265-341). True total tests = 213, not 202. |
| **Metric inflation** | CONFIRMED (over-stated coverage) | P0 R1 sec. 4 priority table said `tests/http_integration_tests.rs` covers `http.pcap` / `http-full.cap`. Actual: only `http-full.cap`; `http.pcap` is **not referenced anywhere** in the tree. |
| **Speculation presented as inference** | CONFIRMED (false hypothesis) | P0 R1 sec. 3 and sec. 7 said `smb3.pcapng` is "presumably used as negative test." Git history (`4feba74`) reveals it was added "for future pcapng support" alongside 7 other Wireshark sample fixtures. It is unconsumed. |
| **Over-extrapolated open-question list** | PARTIAL | The 18 open questions in P0 R1 sec. 8 are mostly well-grounded; items 1-5 are direct code observations. Item 12 ("`smb3.pcapng` presumably negative test") is the only over-extrapolation -- it should have been "fixture exists but no test references it." |
| **Missed inline-test surface** | CONFIRMED | P0 R1 sec. 5 noted `src/reporter/terminal.rs` has a `#[cfg(test)]` block but said "no `#[test]` attributes matched -- the inline tests appear to be elsewhere or use a different gate." Wrong -- the 11 `#[test]` functions are in lines 265-341, gated by the `#[cfg(test)]` at line 261. R1's `awk` only scanned `tests/`. |
| **Cargo.lock byte-size** | CONFIRMED unchanged | 38,291 bytes, mtime 2026-04-07. |
| **38 .rs file count** | CONFIRMED unchanged | `find src tests -name '*.rs' \| wc -l` = 38. |
| **3868 src LOC** | CONFIRMED unchanged | Total = 3868. |
| **6021 test LOC** | CONFIRMED unchanged | Total = 6021. |
| **Superpowers 10+8** | CONFIRMED unchanged | 10 plans + 8 specs. |
| **CI 4-jobs** | CONFIRMED unchanged | 68 lines, 4 jobs. No new jobs (no codecov, no cargo-audit, no cargo-deny). |
| **Open questions count** | CONFIRMED 18 | No inflation. |

---

## 2. Per-Target Findings (from Pass 6 sec. 7 P0 plan)

### Target 1 -- Re-verify the 38 .rs file count
`find ... -name '*.rs' | wc -l` = **38**. UNCHANGED. P0 R1 was correct.

### Target 2 -- Audit `tests/fixtures/` (14 files) for consumer mapping

| Fixture | Size (B) | Consumed by | Verdict |
|---|---|---|---|
| `dns.cap` | 4,338 | (none) | **UNCONSUMED** -- added speculatively for "future protocol and threat analyzer testing." |
| `dns-remoteshell.pcap` | 25,005 | (none) | **UNCONSUMED** -- added for "DNS anomaly/exfiltration" future testing. |
| `http.pcap` | 247 | (none) | **UNCONSUMED** -- P0 R1 over-stated. |
| `http-full.cap` | 25,803 | `tests/http_integration_tests.rs:10` | CONSUMED. |
| `http-ooo.pcap` | 1,209 | `tests/linktype_integration_tests.rs:32` | CONSUMED. |
| `ipv6-ripng.pcap` | 20,264 | (none) | **UNCONSUMED**. |
| `segmented.pcap` | 33,144 | `tests/linktype_integration_tests.rs:19` | CONSUMED. |
| `slammer.pcap` | 458 | (none) | **UNCONSUMED**. |
| `smb3.pcapng` | 15,692 | (none) | **UNCONSUMED** -- added "for future pcapng support" per commit `4feba74`. P0 R1 hypothesis REFUTED -- `tests/reader_tests.rs` synthesizes pcapng bytes inline, it does not read this file. |
| `teardrop.cap` | 1,828 | (none) | **UNCONSUMED**. |
| `tls.pcap` | 25,057 | `tests/tls_integration_tests.rs:76`, `tests/linktype_integration_tests.rs:7` | CONSUMED. |
| `tls12-aes256gcm.pcap` | 2,064 | `tests/tls_integration_tests.rs:28,105` | CONSUMED. |
| `tls13-rfc8446.pcap` | 4,158 | `tests/tls_integration_tests.rs:54` | CONSUMED. |
| `v6-http.cap` | 9,159 | (none) | **UNCONSUMED**. |

**Summary:** 6 fixture-file references against **5 unique consumed fixtures** (`http-full.cap`, `http-ooo.pcap`, `segmented.pcap`, `tls.pcap`, `tls12-aes256gcm.pcap`, `tls13-rfc8446.pcap`). **8 dead fixtures** awaiting future analyzer work. Dead-fixture disk total = **~76 KB**.

### Target 3 -- `///` doc-line count per src file (Pass 5 drift-hotspot)

| File | Doc lines (`///`+`//!`) | `pub` items | Doc-per-pub ratio | LOC | Hotspot? |
|---|---|---|---|---|---|
| `src/analyzer/tls.rs` | 73 | 8 | 9.1 | 750 | HIGH coverage |
| `src/reporter/terminal.rs` | 42 | 3 | 14.0 | 350 | HIGH coverage |
| `src/mitre.rs` | 26 | 5 | 5.2 | 144 | HIGH coverage |
| `src/reassembly/mod.rs` | 24 | 36 | 0.67 | 564 | MEDIUM |
| `src/cli.rs` | 23 | 13 | 1.8 | 113 | MEDIUM |
| `src/findings.rs` | 9 | 12 | 0.75 | 92 | MEDIUM |
| `src/reassembly/segment.rs` | 5 | 3 | 1.67 | 240 | LOW |
| `src/analyzer/mod.rs` | 4 | 8 | 0.5 | 31 | LOW |
| `src/analyzer/http.rs` | 3 | 10 | 0.3 | 535 | **DRIFT HOTSPOT** |
| `src/decoder.rs` | 1 | 11 | 0.09 | 140 | **DRIFT HOTSPOT** |
| `src/analyzer/dns.rs` | 0 | 2 | 0.0 | 81 | **DRIFT HOTSPOT** |
| `src/dispatcher.rs` | 0 | 5 | 0.0 | 118 | **DRIFT HOTSPOT** (subject of ADR 0001, zero in-source doc) |
| `src/main.rs` | 0 | 0 | -- | 256 | OK |
| `src/lib.rs` | 0 | 10 | 0.0 | 10 | LOW |
| `src/reader.rs` | 0 | 9 | 0.0 | 58 | **DRIFT HOTSPOT** |
| `src/summary.rs` | 0 | 9 | 0.0 | 61 | **DRIFT HOTSPOT** |
| `src/reassembly/flow.rs` | 0 | 47 | 0.0 | 243 | **MAJOR DRIFT HOTSPOT** |
| `src/reassembly/handler.rs` | 0 | 4 | 0.0 | 29 | **DRIFT HOTSPOT** |
| `src/reporter/mod.rs` | 0 | 3 | 0.0 | 15 | LOW |
| `src/reporter/json.rs` | 0 | 1 | 0.0 | 38 | LOW |

Crate-wide totals: 210 doc lines / 199 pub items = ~1.05 doc per pub, skewed by 3 well-documented modules holding 141/210 = 67% of all in-source doc lines. **9 files have zero doc lines.**

### Target 4 -- CI workflow audit since 2026-05-19
`git log --since='2026-05-19'` = empty. No commits since 2026-05-19. `ci.yml` unchanged (68 lines, 4 jobs). No security audit job (no `cargo audit`, `cargo deny`, dependabot, codeql).

### Target 5 -- Cargo.lock byte-size
**38,291 bytes, mtime 2026-04-07.** Matches P0 R1 exactly. Dep graph unchanged.

### Target 6 -- `docs/superpowers/` count
10 plans + 8 specs. Unchanged.

---

## 3. Refined Inventory Deltas (vs. P0 R1)

| Metric | P0 R1 | P0 R2 verified | Delta |
|---|---|---|---|
| `.rs` files | 38 | 38 | 0 |
| src LOC | 3,868 | 3,868 | 0 |
| test LOC | 6,021 | 6,021 | 0 |
| Total Rust LOC | 9,889 | 9,889 | 0 |
| Inline `#[test]` in `src/` | 0 | **11** | **+11** |
| `#[test]` in `tests/` | 202 | 202 | 0 |
| **Total test functions** | **202** | **213** | **+11 CORRECTION** |
| Fixtures consumed | 14 (implied) | **5** of 14 | **-9 mis-implication** |
| `smb3.pcapng` purpose | "negative test" | "future pcapng support" | REFUTED |
| `http.pcap` consumer | listed | unconsumed | -1 CORRECTION |
| Cargo.lock bytes | 38,291 | 38,291 | 0 |
| Superpowers files | 10+8 | 10+8 | 0 |
| CI jobs | 4 | 4 | 0 |

---

## 4. New Open Questions Surfaced by Round 2

19. **Dead-fixture liability.** 8 of 14 fixtures (76 KB) are checked in but never read. Pass 5: record convention "speculative fixture staging"; Pass 4: note repo-size cost.

20. **Inline reporter tests are not orphaned.** `cargo test --all-targets` does exercise them in CI -- they were merely uninventoried. Pass 5: note inconsistent test-placement convention (1 of 20 src files uses inline `#[cfg(test)]`).

21. **ADR-vs-source documentation gap.** `docs/adr/0001`'s dispatcher rationale exists; `src/dispatcher.rs` has 0 `///` lines.

22. **`reassembly/flow.rs` has 47 `pub` items with zero documentation** -- canonical drift hotspot. Pass 3 BCs about `FlowKey`/`FlowState` lean entirely on test names.

23. **No supply-chain scanning in CI.** Missing security NFR for Pass 4.

---

## 5. Delta Summary

- **Metric corrections:** 4 (inline test +11, total tests 202->213, fixture consumption 14->5/14, `http.pcap` removed).
- **Refuted hypothesis:** 1 (`smb3.pcapng`).
- **New drift hotspots:** 4 primary + 5 secondary.
- **New open questions:** 5 (items 19-23).
- **Confirmed-unchanged:** 38 .rs files; 3868 src LOC; 6021 test LOC; Cargo.lock 38,291 B; superpowers 10+8; ci.yml 4 jobs.
- **Remaining gaps:** None for Pass 0.

---

## 6. Novelty Assessment

**Novelty: SUBSTANTIVE.**

Test count moves 202->213 (+5.4%) -- same class of error P3 R2 caught in Pass 3 R1 (137->216). Eight of fourteen fixtures are dead; any spec citing "fixture-coverage" must be rewritten with 5/14 = 36% consumption. The `smb3.pcapng` hypothesis is refuted by commit message. Four drift hotspots are newly named with quantitative ratios. Each finding changes a specification claim a downstream skill would emit.

---

## 7. Pass 0 Convergence Declaration

**Pass 0 has NOT converged at round 2.** Round 2 produced substantive numeric corrections plus a refuted hypothesis. A round 3 is unlikely to be needed -- defer to Pass 8 reconciliation; trigger round 3 only if Pass 8 surfaces further numeric drift against P0 R2's revised figures.

---

## 8. State Checkpoint

```yaml
pass: 0
round: 2
status: complete
files_scanned: 38_rust + 21_markdown + 14_pcap_fixtures + 1_ci_yaml + 1_cargo_lock
total_source_loc: 3868
total_test_loc: 6021
total_test_functions: 213    # CORRECTED (was 202; +11 inline in terminal.rs)
fixtures_total: 14
fixtures_consumed: 5
fixtures_dead: 8
adrs: 3
ci_jobs: 4
cargo_lock_bytes: 38291
superpowers_plans: 10
superpowers_specs: 8
new_open_questions: 5
hallucinations_found: 3
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_round: defer to Pass 8 reconciliation
```

---

## 9. Orchestrator Note (100 words)

P0 R2 corrected three metric errors in R1. Total `#[test]` count is 213, not 202 -- R1 missed 11 inline tests in `src/reporter/terminal.rs:265-341`. Only 5 of 14 pcap fixtures are consumed by tests; 8 are dead-staged for future analyzers (commit `4feba74`). The `smb3.pcapng` "negative test" hypothesis is refuted -- it was added "for future pcapng support." Identified 4 documentation drift hotspots, led by `reassembly/flow.rs` (47 pub items, 0 doc lines). All R1 unchanged claims (LOC, file count, Cargo.lock size, CI jobs, superpowers count) were re-verified clean. Novelty: SUBSTANTIVE.
