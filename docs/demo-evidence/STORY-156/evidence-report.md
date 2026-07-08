# STORY-156 Demo Evidence Report

**Story:** STORY-156 — ARP Findings Output Unbounded-Cap Documentation + Regression Test (BC-2.16.016)
**Branch:** feature/STORY-156-arp-unbounded-doc
**Commit:** a61950f
**Date:** 2026-07-07
**Recorded by:** demo-recorder

---

## Evidence Coverage

| AC | Title | Artifact(s) | Verdict |
|----|-------|-------------|---------|
| AC-001 | `--arp` long_help documents UNBOUNDED findings (BC-2.16.016 PC-4) | `AC-001-arp-help-unbounded.gif` / `.webm` / `.tape` | PASS |
| AC-002 | No `MAX_FINDINGS` constant on ARP path (invariant-by-inspection, enforced via AC-003) | enforced by AC-003 regression test | PASS |
| AC-003 | `test_BC_2_16_016_arp_findings_vec_has_no_cap` — 10,001 findings, no cap | `AC-002-003-no-cap-regression.gif` / `.webm` / `.tape` | PASS |
| AC-004 success | `test_BC_2_16_016_summarize_has_no_dropped_findings_key` passes on production code | `AC-004-summarize-pass.gif` / `.webm` / `.tape` | PASS |
| AC-004 error path | Pin CAN fail: injected `dropped_findings` key triggers BC-citing failure | `AC-004-error-path-fail.gif` / `.webm` / `.tape` | DEMONSTRATED |

Overall story verdict: **PASS** (all mandatory ACs satisfied; error path demonstrates pin fidelity)

---

## AC-001 — `--arp` long_help documents unbounded findings

**Artifacts:** `AC-001-arp-help-unbounded.gif`, `AC-001-arp-help-unbounded.webm`, `AC-001-arp-help-unbounded.tape`

**Success path:** `wirerust analyze --help` output piped through `grep -B 3 -A 10 'UNBOUNDED'`
shows the `--arp` flag's long_help text containing:

```
Findings output is UNBOUNDED: unlike HTTP/TLS/Modbus/DNP3 analyzers (which cap
findings at 10,000 via the TCP reassembly layer), ARP operates at the Ethernet
link layer and bypasses that cap entirely. A capture with N spoof or storm events
produces up to N findings with no platform-imposed limit. Operators analyzing
adversarial or large captures should be aware that findings can grow proportionally
to the number of triggering frames.
```

Text confirms `BC-2.16.016 PC-4` (`long_help` attribute on `--arp` flag in `src/cli.rs`).
Delivered in commit `909d55c` (fix-pc-013-014-015 cycle).

**Error path:** N/A — this AC has no meaningful error path (the help flag always succeeds).

**Verdict: PASS**

---

## AC-002 — No `MAX_FINDINGS` constant on ARP path

**Enforcement:** Invariant verified by code inspection — no `const MAX_FINDINGS` definition
in `src/analyzer/arp.rs`. Enforced implicitly by the AC-003 regression test: if a
`MAX_FINDINGS` cap were introduced, `all_findings.len() == 10_001` would fail (plateau at cap).

**No separate recording required** — see AC-003.

**Verdict: PASS**

---

## AC-003 — `test_BC_2_16_016_arp_findings_vec_has_no_cap` (10,001 findings, no cap)

**Artifacts:** `AC-002-003-no-cap-regression.gif`, `AC-002-003-no-cap-regression.webm`, `AC-002-003-no-cap-regression.tape`

**Success path:** `cargo test bc_2_16_016_arp_findings_vec_has_no_cap` runs the test that:

1. Creates `ArpAnalyzer::new(spoof_threshold=1, storm_rate=u32::MAX)`
2. Synthesizes 20,002 ARP reply frames (10,001 distinct IPs × 2 frames each)
3. Accumulates all `Vec<Finding>` items into `all_findings`
4. Asserts `all_findings.len() == 10_001` (primary) and `> 10_000` (no-cap invariant)

Terminal output shows:

```
running 1 test
test analyzer::arp::bc_2_16_016::test_BC_2_16_016_arp_findings_vec_has_no_cap ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 0.04s
```

The reassembly-layer `MAX_FINDINGS = 10,000` (in `src/reassembly/mod.rs`) does NOT apply to
the ARP link-layer path — confirmed by 10,001 findings returned without truncation.

**Verdict: PASS**

---

## AC-004 — `test_BC_2_16_016_summarize_has_no_dropped_findings_key`

### Success path

**Artifacts:** `AC-004-summarize-pass.gif`, `AC-004-summarize-pass.webm`, `AC-004-summarize-pass.tape`

`cargo test test_BC_2_16_016_summarize_has_no_dropped_findings_key` on production code:

- EC-001 (zero frames): fresh analyzer `summarize()` has no `"dropped_findings"` key
- EC-003 (>10,000 events): after 10,001 D1 spoof findings, `summarize()` still has no
  `"dropped_findings"` key

Terminal output shows:

```
running 1 test
test analyzer::arp::bc_2_16_016::test_BC_2_16_016_summarize_has_no_dropped_findings_key ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 0.04s
```

**Verdict: PASS**

### Error path — injection demo

**Artifacts:** `AC-004-error-path-fail.gif`, `AC-004-error-path-fail.webm`, `AC-004-error-path-fail.tape`

To demonstrate the pin can detect a regression, `src/analyzer/arp.rs` `summarize()` was
temporarily modified to inject:

```rust
// DEMO-INJECTION: simulate accidental dropped_findings key (do NOT commit)
detail.insert(
    "dropped_findings".to_string(),
    serde_json::json!(0),
);
```

With this injection active, the test fails with the expected BC-citing message:

```
test analyzer::arp::bc_2_16_016::test_BC_2_16_016_summarize_has_no_dropped_findings_key ... FAILED
BC-2.16.016 PC-2/3 (zero-frame): summarize() must NOT emit 'dropped_findings' on a fresh
analyzer. Keys present: ["bindings_evicted", "bindings_tracked", "dropped_findings", ...]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 96 filtered out
```

After recording, `src/analyzer/arp.rs` was restored via `git checkout -- src/analyzer/arp.rs`.
Production code is unchanged — `git diff origin/develop..HEAD -- src/` shows only the
`mod bc_2_16_016` test addition in `src/analyzer/arp.rs` (commit `7e4fe6d`).

**Verdict: DEMONSTRATED** (pin fires correctly on regression injection)

---

## File Index

```
docs/demo-evidence/STORY-156/
  AC-001-arp-help-unbounded.gif         -- --arp help showing UNBOUNDED text
  AC-001-arp-help-unbounded.webm        -- same, archival format
  AC-001-arp-help-unbounded.tape        -- VHS script (archived; <REPO-ROOT> placeholder)
  AC-002-003-no-cap-regression.gif      -- 10,001-findings no-cap regression test passing
  AC-002-003-no-cap-regression.webm     -- same, archival format
  AC-002-003-no-cap-regression.tape     -- VHS script (archived; <REPO-ROOT> placeholder)
  AC-004-summarize-pass.gif             -- summarize() no-dropped_findings test passing
  AC-004-summarize-pass.webm            -- same, archival format
  AC-004-summarize-pass.tape            -- VHS script (archived; <REPO-ROOT> placeholder)
  AC-004-error-path-fail.gif            -- injected dropped_findings key causes test failure
  AC-004-error-path-fail.webm           -- same, archival format
  AC-004-error-path-fail.tape           -- VHS script (archived; <REPO-ROOT> placeholder)
  evidence-report.md                    -- this file
```

---

## Recording Notes

- VHS 0.x (Charm CLI) used for all recordings. Font: Menlo (macOS system font). Theme: Dracula.
- Tape files use `<REPO-ROOT>` and `<HOME>` placeholders per PG-W70-DEMO-SCRUB convention —
  no absolute host paths are committed.
- Test binaries were pre-compiled (`cargo test --no-run`) before recording to avoid
  compilation noise in the captured terminal output.
- AC-004 error-path injection was performed and restored in the same recording session;
  `git diff HEAD -- src/` was confirmed clean before commit.
- All recordings run on the story worktree at HEAD `a61950f` (branch
  `feature/STORY-156-arp-unbounded-doc`).
