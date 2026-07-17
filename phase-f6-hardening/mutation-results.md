# Phase F6 — Mutation Testing Results (feature-iec104 delta)

**Feature:** IEC-104 passive analyzer (STORY-167..174)
**develop HEAD:** `b36b884`
**Date:** 2026-07-17
**cargo-mutants version:** 27.0.0 (`~/.cargo/bin/cargo-mutants`)
**Scope:** `cargo mutants --file src/analyzer/iec104.rs`
**Threshold:** F6 skill ≥ 90% kill on changed file. STORY-174 baseline was 117/122 = 95.9%;
emit-site changes (FIX-P4-001 / FIX-F5-001) added mutable lines → 164 mutants now.

---

## Headline result

| Metric | Value |
|--------|-------|
| Total mutants | 164 |
| Unviable (don't compile) | 6 |
| Proof-harness mutants (`#[cfg(kani)]`, out of production scope) | 35 |
| **Production mutants (viable)** | **123** |
| Production caught | 112 |
| Production killed by non-termination (timeout) | 6 |
| Production survivors (all equivalent) | 5 |
| **Production kill rate** | **118/123 = 95.9%** |
| Kill rate excluding equivalent mutants | 118/118 = **100%** |
| Killable (real-gap) survivors | **0** |

Mutation gate: **PASS** (95.9% ≥ 90%; all 5 survivors proven equivalent; 0 killable gaps).
Result matches the STORY-174 baseline (95.9%) exactly — the emit-site fixes did not
introduce any new killable survivors.

---

## Methodology note — first run was invalid (measurement artifact), re-run scoped

The initial run used `-j4` with cargo-mutants' default full-suite test command
(`cargo test`, ~2600 tests) and the auto-set 185 s test timeout. On this 16-core
machine, four concurrent full-suite build+test cycles oversubscribed CPU, so many
runs exceeded 185 s and were recorded as **TIMEOUT** — including mutants that are
actually caught. Proof: the `extract_ns >> → <<` mutant was recorded as TIMEOUT in
the `-j4` run, but when applied manually it fails 4 tests in 0.07 s (clearly CAUGHT).
Because a genuinely-*missed* mutant's all-passing full-suite run would likewise have
exceeded 185 s, the `-j4` run's `missed=0` could not be trusted — real survivors
would be hidden among the 96 timeouts.

The run was redone with the test command scoped to the fast, relevant set
(`-- --lib --test iec104_analyzer_tests`, ~318 tests, ~2 s), `-j4`, `--timeout 60`.
Each run now completes in seconds, so caught mutants fail fast and only genuine
non-termination reaches the timeout. This scoped run is the authoritative result
above. (Scoping is safe for over-reporting: any mutant killable only by an
out-of-scope test would show as MISSED and be caught in triage — the opposite,
unsafe direction of the `-j4` artifact.) The invalid run is preserved at
`mutants.out.j4-invalid/`.

---

## Timeout mutants (6) — genuine non-termination = KILLED

All 6 are loop-advance operator mutations in `Iec104Analyzer::on_data` that break
frame-walk termination:

- `1239:25 += → -=` and `+= → *=` (start-byte / stub advance)
- `1289:25 += → -=` and `+= → *=` (malformed-LEN advance)
- `1352:21 += → -=` and `+= → *=` (`pos += frame_len` frame advance)

Reversing/scaling the position advance makes `while pos < buf.len()` never
terminate → infinite loop → 60 s timeout. This is detection (the mutation produces
observable non-termination) and corroborates VP-047's loop-termination property
(the real code terminated on 2.64M fuzz inputs). Counted as killed.

## Production survivors (5) — all EQUIVALENT

| Site | Mutation | Equivalence rationale |
|------|----------|-----------------------|
| `866:9` | delete arm `100 \| 101 \| 103` in `detect_iec104_threats` | TypeIDs 100/101/103 are in [1,127]; deleting the explicit benign arm makes them fall through to the catch-all `_ => {}` (line 915), which also emits no finding. Identical observable behavior. The arm is documentation of intent, not a behavioral branch. |
| `949:25` | `\| → ^` in `extract_ns` | `((cf1 as u16) >> 1)` occupies bits 0-6; `((cf2 as u16) << 7)` occupies bits 7-14. Operands are bit-disjoint, so OR ≡ XOR for all inputs. |
| `967:25` | `\| → ^` in `extract_nr` | Same bit-disjoint reasoning as extract_ns. |
| `1195:32` | `> → >=` on `carry.len() > MAX_IEC104_CARRY_BYTES` (255) | Residual stashed is always `remaining.len() < frame_len ≤ 255`, i.e. ≤ 254 (line 1295); a 255-byte complete frame is walked off, not stashed. So `carry.len()` can never equal 255 at the check via the public `on_data` API — the `>`/`>=` boundary is unreachable. |
| `1358:33` | `> → >=` on `local_findings.len() > remaining_cap` | At the boundary `len == remaining_cap`: the `>=` branch computes `dropped += (len - remaining_cap) = 0` (no change) and `truncate(remaining_cap)` is a no-op (length already equals cap). Both branches keep all findings with the same drop count — identical observable state. |

None is killable by any black-box test; no FIX-F6 test is warranted.

## Proof-harness mutants (35 missed) — OUT OF PRODUCTION SCOPE

35 missed mutants are inside `mod kani_proofs` (lines 1480-1531), gated by
`#[cfg(kani)]`. Under `cargo test` the `kani` cfg is off, so this module is not
compiled and mutating it cannot change any test outcome — cargo-mutants records
them all as MISSED. They mutate proof-harness code, not production logic, and the
harness itself is verified by Kani (VP-044 `verify_parse_apci_header_safety`:
VERIFICATION SUCCESSFUL, 0 of 89 failed — see kani-results.md). These are excluded
from the production kill-rate.

Follow-up (non-blocking): configure cargo-mutants to skip `#[cfg(kani)]` blocks
(e.g. an `exclude_re`/`skip` entry in a `mutants.toml`) so future runs don't
enumerate proof-harness sites.

## Unviable mutants (6)

Return-type replacements that don't typecheck (no `Default`, or wrong shape):
`classify_frame_format`, `process_u_frame`, `parse_apci_header`, `parse_asdu`,
`track_ns_desync`, `summarize`. Excluded from denominator per standard practice.

## Verdict

Mutation gate: **PASS** — production kill rate 95.9% (≥90%), 100% of non-equivalent
viable production mutants killed, 0 killable survivors, 0 FIX-F6 test gaps.
