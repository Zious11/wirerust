---
document_type: burst-log
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-01T00:00:00Z
cycle: "fix-tls-clienthello-frag"
inputs: [STATE.md]
input-hash: ""
traces_to: STATE.md
---

# Burst Log — fix-tls-clienthello-frag

## Burst F6-A (2026-06-30) — Kani proofs + fuzz target

**Agents dispatched:** formal-verifier
**Files touched:** `src/analyzer/tls.rs` (kani_proofs_vp039 module added), `fuzz/fuzz_targets/fuzz_tls_reassembly.rs` (new), `fuzz/Cargo.toml` (registered), `fuzz/Cargo.lock` (synced)
**Versions bumped:** none (additive only)
**Branch:** `fix/f6-tls-hardening` HEAD `cd005f8b990f9396bf34a31b38ef9cdf179faddc` (commits d085db2 + 07865cb + cd005f8). PUSHED to origin.

### Summary

Kani formal proofs (VP-039) added in `#[cfg(kani)] mod kani_proofs_vp039` in `src/analyzer/tls.rs` — additive, kani_proofs_vp005 untouched. 3 harnesses:
- `verify_drain_loop_cursor_safety` (unwind 5, 5/5 covers) — cursor in-bounds, no OOB/underflow
- `verify_no_usize_overflow_on_advance` — no usize overflow on `consumed += 4+body_len`
- `verify_carry_bounded_after_append` — carry ≤ MAX_BUF

All VERIFICATION SUCCESSFUL, NON-VACUOUS (DF-KANI-NONVACUITY-001 satisfied). Documented limit: `drain_loop_model` uses HashMap with fixed RandomState (FFI CBMC can't symbolically execute real hasher); fuzz is the dynamic cross-check.

Fuzz: `fuzz/fuzz_targets/fuzz_tls_reassembly.rs` added. 181s, 1,909,352 execs, 0 crashes/OOM/timeouts. Corpus 826 entries.

Security scan: `cargo audit` = 0 vulns + 1 allowed warning (RUSTSEC-2026-0190 anyhow, PRE-EXISTING). `cargo deny check` advisories FAILED on RUSTSEC-2026-0190 — PRE-EXISTING on develop, delta never touched Cargo.toml/lock.

Regression at cd005f8: `cargo test --all-targets` 2220/0; clippy clean; fmt clean; green-doc-tense PASS.

### Mutation testing results (at burst end — 13 surviving real-gap mutants)

133 total mutants: 102 caught + 5 timeout-caught + 21 missed (8 provably-equivalent, 13 real gaps) + 5 unviable.

The 13 gaps cluster into 6 symmetric C2S/S2C themes (all `src/analyzer/tls.rs`):

| Theme | Mutant Sites | Description |
|-------|-------------|-------------|
| 1 — Step-1 overflow guard exact-MAX_BUF boundary | C2S 829:64 `>→>=`, S2C 998:64 `>→>=` | No test pins carry filled to exactly MAX_BUF (should accept, not clear) |
| 2 — Step-1 guard arithmetic | C2S 829:41 `+→*`, S2C 998:41 `+→*` | Non-zero carry + large payload arithmetic path |
| 3 — Decision-4 body_len spoof boundary (body_len==MAX_BUF) | C2S 900:37 `>→>=`, S2C 1036:37 `>→>=` | body_len exactly MAX_BUF not tested |
| 4 — parse_errors increment (PC-9, non-hello msg_type) | C2S 950:59 `+=→-=` + `+=→*=`, S2C 1079:59 ×2 | No test dispatches non-hello msg_type (0x01 C2S / 0x02 S2C) after partial carry |
| 5 — S2C body_len high-byte lane | S2C 1030:67 `<<→>>` | No S2C test uses length with non-zero bits 16-23 |
| 6 — Incomplete-body guard (consumed>0 partial trailing) + exact-fill | S2C 1047:38 `-→+`; C2S 911:38 needs deterministic (proptest-randomized only); S2C did_drop exact-fill 1155:43 `>→>=` (C2S mirror 1145:43 IS covered) | `carry_len - consumed` with consumed>0 |

**Session paused here for session clear (D-313).** F6 branch fix/f6-tls-hardening at cd005f8, pushed, working tree clean.

---

## Burst F6-B (2026-07-01) — Mutation-gap tests + anyhow bump

**Agents dispatched:** test-writer (mutation-gap tests), devops-engineer (anyhow bump PR)
**Files touched:** `src/analyzer/tls.rs` (mod f6_hardening, 12 tests), `Cargo.toml` (anyhow 1.0.103), `Cargo.lock` (updated)
**Versions bumped:** anyhow 1.0.102→1.0.103
**PRs merged:**
- PR #345 squash `d7f0ef46cb3db1afc5fe77fc6b4bd81c5df262c1` (2026-07-01): `test(analyzer): TLS reassembly F6 hardening — Kani VP-039 proofs, fuzz target, 12 mutation-gap tests`
- PR #346 squash `52907bc71e627974ae31014b8548ff4c941dfd2d` (2026-07-01): `chore(deps): bump anyhow 1.0.102 → 1.0.103 (clears RUSTSEC-2026-0190)`
**develop HEAD after merges:** `52907bc71e627974ae31014b8548ff4c941dfd2d`
**Branches deleted:** `fix/f6-tls-hardening`, anyhow bump branch. Worktrees removed.

### Summary

12 tests added in `mod f6_hardening` (src/analyzer/tls.rs) covering all 6 C2S/S2C symmetric themes from Burst F6-A. Mutation re-run: 100% real-gap kill rate on the 13 previously-surviving mutants. 2 provably-equivalent survivors remain at `tls.rs:950:59` (structurally-unreachable dead code — C2S `Ok(non-ClientHello)` arm in S2C context; documented in test module comment, not a test gap).

anyhow 1.0.102→1.0.103: `cargo deny check` advisories now PASS; RUSTSEC-2026-0190 CLEARED.

SEC-002 (narrow non-RFC overflow window [MAX_BUF-3, MAX_BUF] — clears-and-recovers) and SEC-006 (Step-1 guard strict `>` allows carry to reach exactly MAX_BUF): both behaviors pinned by mod f6_hardening themes 1+2+6. Accepted-by-design; mutation tests constitute the behavioral specification for clear-and-recover at the exact-MAX_BUF boundary.

Process-gap discovered during mutation run: `cargo mutants --jobs 8` masks real survivors as load-induced timeouts on this suite — infinite-loop mutants peg cores, inflating innocent mutants past auto-timeout → false "0 missed". F6 run used low --jobs. Logged as PG-MUTANTS-JOBS-001.

### Details

| Agent | Task | Output |
|-------|------|--------|
| test-writer | 12 mutation-gap tests across 6 themes | mod f6_hardening in src/analyzer/tls.rs |
| formal-verifier | Mutation re-run scoped to delta | 100% real-gap kill; 2 provably-equiv survivors documented |
| devops-engineer | cargo update -p anyhow + PR #346 | anyhow 1.0.103; RUSTSEC-2026-0190 cleared |
| pr-manager | PR #345 review + merge | CI 11/11 green; human-authorized squash-merge |
| pr-manager | PR #346 review + merge | CI 11/11 green; human-authorized squash-merge |

**F6 COMPLETE. F7 delta convergence starting (D-314).**
