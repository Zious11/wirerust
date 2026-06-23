# F6 Targeted Hardening — issue #64 mitre_attack JSON enrichment delta

- **Mode:** Feature Mode Phase F6 (vsdd-factory:phase-f6-targeted-hardening)
- **Agent:** formal-verifier
- **Date:** 2026-06-23
- **Tree:** develop @ `029725b536650354da45e48f1d3ee23ed3eb8909` (clean working tree; transient mutation/fuzz artifacts cleaned, nothing committed)
- **Crate:** wirerust v0.9.3

## Delta under hardening (scoped)

| File | Change |
|------|--------|
| `src/reporter/json_dto.rs` (NEW) | `FindingJsonDto<'a>` + `MitreAttackEntry` + `From<&Finding>` building the per-finding `mitre_attack` array (BC-2.11.035). |
| `src/reporter/json.rs` | Wires `FindingJsonDto::from` into `render` (`findings.iter().map(FindingJsonDto::from)`). |
| `src/mitre.rs` | 3 new `MitreTactic` variants (IcsDiscovery/IcsCollection/IcsCommandAndControl); `technique_tactic_id` arms (TA0102/TA0100/TA0101); 5 technique remaps; `Display`; `all_tactics_in_report_order()` → 20 tactics; extended `vp007_catalog_drift_guard`. |

## Summary table

| # | Task | Result |
|---|------|--------|
| 1 | Formal / Kani assessment + re-run | **PASS** — no new VP warranted (test-sufficiency CONFIRMED); 4/4 VP-007 Kani harnesses re-verified SUCCESSFUL. |
| 2 | Mutation testing (delta-scoped) | **PASS** — 49 caught / 53 viable = 92.5%; 100% of test-reachable mutants killed. 4 "survivors" are `#[cfg(kani)]` proof-harness false positives (not test gaps). 0 production-logic survivors. |
| 3 | Fuzz | **PASS** — no JSON-reporter fuzz target exists; mitre_attack path proven panic-free by reasoning; existing fuzz target regression smoke clean (no crashes). |
| 4 | Security scan (full tree) | **PASS** — `cargo audit` 0 vulnerabilities; `cargo deny check` advisories/bans/licenses/sources all OK. |
| 5 | Full regression | **PASS** — `cargo test --all-targets` green (exit 0), 0 failures. |

No findings route to test-writer or implementer. No mutation survivors that are real test gaps; no fuzz crashes.

---

## Task 1 — Formal verification assessment (CONFIRM prior F1/F2 decision)

**Decision: CONFIRMED — no new Kani VP is warranted for the delta. The path is test-sufficient.**

### Challenge applied to `json_dto.rs` `From<&Finding>` impl

The `From` impl was examined for any partial-function / overflow / panic / OOB obligation that would merit a Kani proof:

```rust
let mitre_attack = finding.mitre_techniques.iter().map(|id| {
    let (name, tactic_name, tactic_id) = match mitre::technique_info(id) {
        Some((n, tactic)) => (Some(n), Some(tactic.to_string()), mitre::technique_tactic_id(id)),
        None => (None, None, None),
    };
    MitreAttackEntry {
        id: id.clone(),
        name, tactic_id, tactic_name,
        reference: format!("https://attack.mitre.org/techniques/{id}/"),
    }
}).collect();
```

Panic-obligation audit — **no panic reachable on arbitrary `String` technique IDs**:

- `mitre_techniques: Vec<String>` — iterated by `.iter()`. **No indexing, no slicing** of the vec or of any `id`.
- `technique_info(id: &str)` is the already-Kani-verified (VP-007) catalog lookup: a closed-world `match` whose only catch-all is `_ => None`. Total, no panic, for any `&str`. Re-proven this run.
- `technique_tactic_id(id)` chains off `technique_tactic` → `technique_info`; same totality. Its inner `match` over `MitreTactic` is exhaustive (compile-enforced).
- Pure `Option`-chaining (`match … Some/None`). No `.unwrap()`, no `.expect()`, no `?`, no `[]`, no arithmetic.
- `id.clone()` — `String` clone, infallible (panics only on allocation failure, which is out of scope for the no-panic contract, consistent with the rest of the crate).
- `format!(".../{id}/")` — `Display` formatting of an arbitrary `String`; infallible, no indexing.
- `.collect::<Vec<_>>()` — infallible.

There is **no symbolic/unbounded input obligation** in this code that BMC could discharge more strongly than the type system + the existing VP-007 catalog proof already do. The only "input" is the set of technique-ID strings, and every string outside the 25-entry catalog deterministically takes the `None` arm (proven by `verify_unknown_id_returns_none_no_panic`, which uses the well-formed-but-unregistered canary `T9999`).

### New `mitre.rs` arms

The 3 new `MitreTactic` variants and the TA0102/TA0100/TA0101 `technique_tactic_id` arms add only literal match arms returning `&'static str`. No partial functions, arithmetic, or indexing introduced. The exhaustive `match tactic { … }` in `technique_tactic_id` is **compile-time total** — a new variant without a TA-ID arm is a build error, not a runtime panic. Drift is additionally caught at test time by `vp007_catalog_drift_guard` (the TA-ID half asserts every seeded ID returns `Some` from `technique_tactic_id`).

### Test-sufficiency justification (rigorous)

The delta is covered by three independent mechanical guardrails, making a new VP redundant:

1. **Existing VP-007 Kani proofs (4 harnesses)** already prove the catalog lookup the `From` impl depends on is total and panic-free over the closed ID space — including the unknown-ID `None` corollary. The `From` impl is a pure, branch-free-of-panics consumer of that proven core.
2. **`vp007_catalog_drift_guard`** (in-crate `#[test]`) sweeps the *entire finite ID grammar* (~10.01M IDs: `T[0-9]{4}` and `T[0-9]{4}.[0-9]{3}`), asserting forward+backward completeness of `technique_info` against `SEEDED_TECHNIQUE_IDS`, AND that every seeded ID resolves in `technique_tactic_id` (the BC-2.11.035 Catalog-Extension drift guard), AND the `T9999` canary returns `None` from both. This is an exhaustive enumeration over the catalog's whole key space — strictly stronger than a representative-sample Kani harness would be for the catalog dimension.
3. **13 BC-2.11.035 reporter tests** (`tests/reporter_json_tests.rs`) cover the exact branch set a Kani harness or mutation would target: the fully-resolved 5-field object (known ID), the unknown-ID partial object (`name`/`tactic_id`/`tactic_name` absent, `id`/`reference` present), the empty-`Vec` `skip_serializing_if` omission, multi-tag arrays, and the synthesized reference URL.

**Conclusion:** the `mitre_attack` path is pure `Option`-chaining over a Kani-verified, drift-guarded, exhaustively-swept static catalog. There is no residual partial-function / overflow / panic obligation that a new Kani proof would discharge. Classification **"test-sufficient"** stands.

### VP-007 Kani re-run (relevant subset)

```
cargo kani --harness verify_all_seeded_ids_match_format \
           --harness verify_all_seeded_ids_resolve \
           --harness verify_all_emitted_ids_resolve \
           --harness verify_unknown_id_returns_none_no_panic
```

```
SUMMARY:
VERIFICATION:- SUCCESSFUL
Complete - 4 successfully verified harnesses, 0 failures, 4 total.
```

All four VP-007 harnesses remain **proven / green** post-delta. (cargo-kani 0.67.0.)

---

## Task 2 — Mutation testing (cargo-mutants 27.0.0, delta-scoped)

```
cargo mutants --file src/reporter/json_dto.rs --file src/mitre.rs \
  --timeout 60 --minimum-test-timeout 30 \
  -- --lib --test mitre_tests --test reporter_json_tests \
     --test bc_2_09_100_multitag_tests --test bc_2_16_story114_arp_tests
```

> Note on methodology: a first run with the default (full-workspace) test phase timed out
> on the unmutated baseline (the full integration-test suite exceeds the per-test timeout).
> The run above scopes the test phase to the delta-relevant targets (lib unit tests incl. the
> drift guard, plus the four mitre/reporter integration binaries), which makes the baseline
> pass in ~3s and exercises every test that touches the delta.

**Official result:** `58 mutants tested in 13m: 4 missed, 49 caught, 5 unviable`

| Outcome | Count |
|---------|-------|
| Caught | 49 |
| Missed | 4 |
| Unviable (did not compile) | 5 |
| Timeout | 0 |
| **Viable (caught + missed)** | **53** |
| **Kill rate (caught / viable)** | **92.5%** |
| **Test-reachable kill rate** (excluding the 4 cfg(kani) false positives) | **100%** |

### The 4 "missed" mutants are false positives — NOT test gaps

All four survivors are bodies of the `#[cfg(kani)] mod kani_proofs` proof harnesses:

```
MISSED src/mitre.rs:317  replace kani_proofs::verify_all_seeded_ids_match_format with ()
MISSED src/mitre.rs:326  replace kani_proofs::verify_all_seeded_ids_resolve with ()
MISSED src/mitre.rs:336  replace kani_proofs::verify_all_emitted_ids_resolve with ()
MISSED src/mitre.rs:355  replace kani_proofs::verify_unknown_id_returns_none_no_panic with ()
```

These functions are gated behind `#[cfg(kani)]`. Under `cargo test` they **do not exist**, so no unit test can possibly observe them — and cargo-mutants does not run the Kani engine. They are verified by `cargo kani` (Task 1: 4/4 SUCCESSFUL). Mutating a proof harness to `()` is unobservable to `cargo test` by construction. **No test should be added** — doing so would be meaningless; the correct verifier (Kani) already covers them and passed.

### The 5 unviable mutants (excluded from kill rate by convention)

These failed to compile (cargo-mutants synthesized `Default::default()` for types with no `Default`):

- `src/reporter/json_dto.rs:50` — `<impl From<&Finding> for FindingJsonDto>::from -> Self with Default::default()` — `FindingJsonDto` has no `Default` (holds a `&Finding` reference). This is the *only* mutant cargo-mutants could generate for `json_dto.rs`, and it is structurally uncompilable. The `From` impl's correctness is instead covered by the 13 BC-2.11.035 reporter tests (known-ID 5 fields, unknown-ID partial, empty-vec omission, multi-tag, reference URL).
- 4 in `src/mitre.rs` — `Default::default()` / `Some(Default::default())` on `MitreTactic` (no `Default` derive). Uncompilable, hence not behavior-bearing.

### Production-logic coverage (all caught)

Every viable production mutant in the delta was killed, including:

- `technique_info` → `None`, and **deletion of each of the 25 match arms** (incl. the new ICS IDs T0830, T0888, T1691.001, T0827, T1557.002) — all caught (drift guard + mitre/reporter tests).
- `technique_tactic` → `None` — caught.
- `technique_tactic_id` → `None` / `Some("")` / `Some("xyzzy")` — caught (the TA-ID drift-guard half + reporter `tactic_id` assertions).
- `<MitreTactic as Display>::fmt` → `Ok(Default::default())` — caught.
- `all_tactics_in_report_order` → `Vec::leak(Vec::new())` (empty slice) — caught.

**No real test gaps. No remediation routed.**

---

## Task 3 — Fuzz

### Coverage assessment

`fuzz/fuzz_targets/` contains 4 targets: `fuzz_decode_packet` (VP-008), `fuzz_dnp3_parse` (VP-023), `fuzz_modbus_parse` (VP-022), `fuzz_pcapng_reader` (VP-028). **None exercises the JSON reporter or `Finding` serialization** — they all cover decode/parse ingress paths. So there is no existing target that drives the `mitre_attack` enrichment path.

### Panic-freedom assertion for the mitre_attack path (reasoning, in lieu of a dedicated target)

The `From<&Finding>` enrichment path is panic-free on **arbitrary `String` technique IDs**, by construction (see Task 1 audit): the input is `Vec<String>`, consumed by `.iter().map(...)` with **no indexing, no slicing, no `.unwrap()`/`.expect()`/`?`, and no arithmetic**. Every ID flows into `technique_info` (Kani-proven total, `_ => None` catch-all), `format!`, and `String::clone`. An arbitrary/garbage/empty/very-long/non-ASCII technique-ID string deterministically takes the `None` arm and still yields a well-formed `MitreAttackEntry { id, reference }` with the optional fields omitted. A dedicated libFuzzer target would add no reachable panic surface over what the type system + VP-007 proof already guarantee. (A future `fuzz_json_reporter` target driving arbitrary `Finding` vectors is a reasonable but non-blocking enhancement.)

### Regression smoke (existing target, bounded)

`cargo +nightly fuzz run fuzz_decode_packet -- -max_total_time=90 -rss_limit_mb=4096`

Result: **no panics, no crashes, no leaks** within the bounded session — the fuzz harness infrastructure is healthy on develop @ 029725b. (No `fuzz/artifacts/**` crash files produced.)

---

## Task 4 — Security scan (full tree)

### `cargo audit` (cargo-audit 0.22.1)

Scanned `Cargo.lock` (193 crate dependencies) against 1138 advisories. **0 vulnerabilities, 0 warnings.** Clean.

### `cargo deny check` (cargo-deny 0.19.6)

```
advisories ok, bans ok, licenses ok, sources ok
```

All four checks pass. No HIGH/CRITICAL findings; no security-reviewer escalation required. Confirms CI's supply-chain gate post-merge.

---

## Task 5 — Full regression

`cargo test --all-targets` on develop @ 029725b — **exit 0, all green.** Lib unit tests (87 incl. the 10.01M-ID `vp007_catalog_drift_guard` sweep, 1.7s), all integration binaries (TLS, DNP3, Modbus, ARP, reassembly, pcapng, reader, reporter), and benches all pass. 0 failures across the suite.

---

## Convergence — F6 COMPLETE

| Criterion | Status |
|-----------|--------|
| Kani VPs (relevant subset) proven | PASS (4/4 VERIFICATION SUCCESSFUL) |
| New VP needed for delta? | No — test-sufficient, justified |
| Mutation kill rate | 92.5% viable / 100% test-reachable (4 survivors are cfg(kani) false positives; 0 real gaps) |
| Fuzz | No crashes; mitre_attack path panic-free by construction |
| Security (audit + deny) | PASS — clean |
| Full regression | PASS — green |
| Routes to test-writer/implementer | None |

**PHASE F6 COMPLETE.** No code committed; working tree clean.
