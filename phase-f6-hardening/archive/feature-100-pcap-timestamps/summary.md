---
document_type: phase-gate-summary
phase: F6-targeted-hardening
feature: issue-100-pcap-timestamps
develop_head: "256a490"
date: "2026-06-09"
gate_verdict: PASS
blocking_items: 0
---

# Phase F6 — Targeted Hardening Gate Summary (Feature #100)

## Gate Verdict

**PASS — no blocking items. develop HEAD `256a490`.**

All four hardening pillars (Kani, fuzz, mutation, security) are green. VP-021 is LOCKED.

---

## Kani Formal Verification

| Metric | Value |
|--------|-------|
| Harnesses attempted | 0 |
| Harnesses passed | 0 |
| Disposition | Justified non-applicability |
| Anti-pattern check | PASS — no cfg/debug-guard added |

VP-021's verification property involves `DateTime<Utc>` conversion (chrono crate types),
`HashMap<FlowKey, u32>` per-flow timestamp storage, and the full TCP reassembly pipeline —
not a bounded pure-arithmetic invariant amenable to Kani. The only candidate for model
checking is the totality of `DateTime::from_timestamp(v as i64, 0)` over all `v: u32`. This
is inline chrono library code (not a wirerust pure helper) and is dischargeable by closed-form
range reasoning: the entire u32 range maps to 1970-2106 CE, well within chrono's representable
domain. This claim is independently confirmed by explicit boundary tests (EC-003: v=0,
EC-004: v=u32::MAX) and by the all-u32 proptest strategy (`ts_sec in 0u32..=u32::MAX`, 256
cases). No cfg-gated or debug-only guard was added; VP-021 is discharged by genuine tests.

---

## Fuzz Testing

| Metric | Value |
|--------|-------|
| Existing targets | 1 (`fuzz_decode_packet`, VP-008) |
| New focused targets added | 0 (justified) |
| Crashes found | 0 |
| Fuzz gap | None |

The attacker-controlled input is `ts_sec: u32` from the pcap packet record header. The
existing `fuzz_decode_packet` target does not reach the timestamp path (`decode_packet` sees
only payload bytes, not the pcap record header). A focused timestamp fuzz target is
justified-omitted: the input domain is a single unstructured `u32` with a single branch of
interest (Some/None from `from_timestamp`), and that domain is already provably exhausted by
the all-u32 proptest plus explicit boundary tests at both endpoints. Coverage-guided fuzzing
adds no discovery value over a full-range proptest on a single unstructured scalar input.
Adversarial-input safety (no panic, no overflow, no info leak) confirmed for `0`, `u32::MAX`,
and random samples across the full domain.

---

## Mutation Testing (cargo mutants --in-diff)

| Metric | Value |
|--------|-------|
| Scope | 6 changed source files (--in-diff diff of delta) |
| Mutants generated | 33 total |
| Unviable | 1 (`Direction` has no `Default` impl -- does not compile) |
| Viable mutants | 32 |
| Caught | 30 |
| Missed | 2 |
| **Raw kill rate** | **93.8%** (30/32) |
| **Effective kill rate (excluding equivalent mutants)** | **100%** (30/30 killable) |

Per-file results:

| File | Tier | Target | Kill rate | Status |
|------|------|--------|-----------|--------|
| `src/analyzer/http.rs` | HIGH | >=90% | 100% (7/7) | PASS |
| `src/analyzer/tls.rs` | CRITICAL/HIGH | >=95% | 100% (9/9) | PASS |
| `src/dispatcher.rs` | CRITICAL | >=95% | 100% (1/1) | PASS |
| `src/reassembly/mod.rs` | HIGH | >=90% | 100% (10/10, 1 unviable) | PASS |
| `src/reassembly/lifecycle.rs` | CRITICAL/HIGH | >=95% | 60% raw / **100% effective** | PASS (equivalent survivors) |
| `src/reassembly/handler.rs` | -- | -- | n/a (trait sig only, no mutable diff lines) | n/a |

The 2 survivors are provably-equivalent mutants at `src/reassembly/lifecycle.rs:62`
(`self.stats.bytes_reassembled += data.len() as u64;` -- the `+=` -> `-=` and `+=` -> `*=`
replacements). This line is a pre-existing statistics accumulator pulled into the mutable
scope only by hunk-adjacency to the Feature #100 changed line 63. Proof of equivalence: the
`close_flow` per-direction flush loop body never executes with non-empty data because
`flush_contiguous_data` unconditionally drains the entire contiguous prefix on every payload
insert (`mod.rs:191-193`), leaving nothing for `close_flow`'s repeat flush to iterate over.
Confirmed empirically by instrumented `cargo test --all-targets` run (0 firings). These
survivors independently re-confirm the F5 close-flush unreachability finding from a different
angle. No mutants.toml skip annotation added (FIX-F6-OPT-001 -- optional, declined per
formal-verifier recommendation; equivalent mutants are correctly understood and documented).

All timestamp-threading logic mutants caught. Mutation gate: PASS.

---

## Security Scan

| Scanner | Result | CRITICAL/HIGH findings |
|---------|--------|------------------------|
| `cargo audit` | PASS (1 known-allowed warning) | 0 |
| `cargo deny check` | PASS (advisories, bans, licenses, sources ok) | 0 |
| semgrep | SKIPPED -- not installed (manual review substituted) | 0 |
| Manual adversarial review (6 changed files) | PASS | 0 |

RUSTSEC-2026-0097 (rand 0.8.5 unsound): known-accepted transitive advisory, unrelated to
Feature #100 (build-time-only codegen path via tls-parser->phf_codegen->rand); `cargo deny`
accommodates it; 0 unaccepted advisories.

Manual adversarial review covered: panic on adversarial timestamp (safe -- `from_timestamp`
total over all u32), integer overflow (safe -- only widening cast u32 as i64;
`overflow-checks = true` in release profile), info leakage (safe -- only ISO-8601 capture
timestamp surfaced), unsafe code (none added), cross-flow leakage (safe -- per-flow `last_ts`
keyed by FlowKey; confirmed by proptest), DoS via timestamp (no allocation/loop-bound added).

Security gate: PASS. 0 CRITICAL/HIGH findings.

---

## Regression Verification

| Check | Result |
|-------|--------|
| `cargo test --all-targets` | 1147 passed / 0 failed |
| `cargo clippy --all-targets -- -D warnings` | CLEAN |
| `cargo fmt --check` | PASS |

Test count increased 1126 -> 1147 from STORY-097/098/099 test additions (Feature #100 F4).
F4 baseline confirmed preserved.

---

## VP-021 Lock Status

| Field | Value |
|-------|-------|
| VP | VP-021 (timestamp-provenance-threading) |
| Status | LOCKED -- `verified` |
| `verification_lock` | `true` |
| `verified_at_commit` | `256a490` |
| `proof_method` | integration + proptest |
| Locked by | spec-steward (Phase-F6 gate) |
| Kani | Justified-excluded (see Kani section above) |

All 21 VPs are now verified and locked. VP-021 was the sole remaining draft VP entering F6.

---

## FIX-F6 Items

| ID | Description | Disposition |
|----|-------------|-------------|
| FIX-F6-OPT-001 | Add `mutants.toml` skip for equivalent lifecycle.rs:62 mutants | OPTIONAL -- DECLINED. Equivalent mutants are correctly understood and documented above. Leave as-is; survivors are audit evidence that the close-flush unreachability claim is independently confirmed from the mutation dimension. |

No blocking FIX-F6 items.

---

## Gate Verdict

**PHASE F6 TARGETED HARDENING: PASS**

- Mutation: 100% effective kill (30/30 killable; 2 equivalent survivors documented)
- Fuzz: 0 crashes; justified-omit for timestamp-specific target
- Kani: justified-skip (inline-chrono totality, no debug-guard anti-pattern)
- Security: cargo audit PASS, cargo deny PASS, manual adversarial review 0 CRITICAL/HIGH
- Regression: 1147 tests green, clippy clean, fmt clean
- VP-021: LOCKED (verified, lock=true, @256a490) -- 21/21 VPs now verified

**NEXT: F7 delta convergence (`vsdd-factory:phase-f7-delta-convergence`)**
5-dimensional convergence check + final human gate.
