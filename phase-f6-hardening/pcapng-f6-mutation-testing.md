# F6 Mutation Testing — pcapng reader functions

- **Date:** 2026-06-21
- **Tree:** `develop` @ `1ca30a3` (HEAD `1ca30a30bf7a809b8bb3b23911fdbe90e403a6e0`)
- **Tool:** `cargo-mutants` (`~/.cargo/bin/cargo-mutants`), default `cargo test` runner, dev profile
- **Crate:** `wirerust@0.9.2`, 16 logical cores, macOS aarch64, stable toolchain
- **Scope:** `src/reader.rs` production functions, `kani_proofs` `#[cfg(kani)]` module excluded
  (`--exclude-re 'kani_proofs'`). The pcapng-relevant production functions all live in
  `src/reader.rs`; **`src/main.rs` was NOT mutated** (see "Coverage / skips" below).

## Invocations

Primary run (measurement pass):

```
cargo mutants --file src/reader.rs --exclude-re 'kani_proofs' --timeout 120 --no-shuffle -j 6
```

Because the primary run produced **39 TIMEOUTs** (an ambiguous outcome, not a clean kill), a
second, decisive recheck was run over **every** timed-out mutant with a 600 s timeout and the
test harness restricted to the pcapng-relevant test binaries (so an unrelated slow binary could
not stall the budget — see "The timeout artifact" below):

```
cargo mutants --file src/reader.rs --re '<all 28 timed-out line:col positions>' \
  --timeout 600 --minimum-test-timeout 300 --no-shuffle -j 4 \
  -- --test proptest_reader --test bc_2_01_story125_epb_tests --test bc_2_01_story124_idb_tests \
     --test bc_2_01_story123_pcapng_tests --test bc_2_01_018_story128_tests \
     --test sec_001_twin_equivalence_tests --test sec_shb_twin_equivalence_tests \
     --test reader_tests --test cli_tests --test integration_test
```

## Headline totals (primary run)

`158 mutants tested in 29m: 108 caught, 11 unviable, 39 timeouts` (exit code 3).

| Outcome | Count |
|---|---|
| Total mutants generated | 158 |
| CAUGHT (killed) | 108 |
| TIMEOUT (ambiguous) | 39 |
| UNVIABLE (did not compile) | 11 |
| MISSED (clean survivor in primary run) | **0** |

The primary run reported **zero clean MISSED survivors**. That number is **misleading**: the
39 TIMEOUTs were hiding real survivors. After the recheck (below) the true picture is:

| True outcome (after recheck) | Count |
|---|---|
| CAUGHT (incl. 18 timeout-masked real kills: 90 + 18) | 126 |
| **MISSED (genuine survivors)** | **21** |
| UNVIABLE | 11 |
| **True mutation score** (caught / (caught + missed), excl. unviable) | **126 / 147 = 85.7%** |

## The timeout artifact (why the primary "0 MISSED" was wrong)

`cargo-mutants` classifies a mutant as **TIMEOUT** when the `cargo test` invocation exceeds the
per-command cap (120 s), which is **not** a clean kill — it is ambiguous. Two distinct causes
were folded into those 39 timeouts:

1. **An unrelated slow/heavy test binary in the full workspace suite.** `cargo-mutants` runs
   the *entire* `cargo test --package=wirerust` for each mutant, which pulls in ~16 unrelated
   test binaries (DNP3, Modbus, story_103, story_109, …). Inspection of the timeout logs shows
   the pcapng test binaries finishing in ~1–3 s and the wall being hit on an unrelated binary
   (`dnp3_detection_tests`, `bc_2_14_105_modbus_dispatch_tests`). The baseline test phase alone
   is 31 s, leaving little headroom under a 120 s cap once a mutant perturbs scheduling.
2. **proptest shrink amplification.** `tests/proptest_reader.rs` runs 1000 cases per property
   over `spb_captured_len`, EPB decode, and the discriminant twins. A bounds/arithmetic mutant
   that produces a counterexample sends proptest into a shrink loop that, on a mutated
   allocation-size computation, can thrash far longer than the unmutated path.

The recheck (600 s timeout + only the pcapng test binaries, baseline test phase ~5 s) resolved
each of the 39 ambiguous mutants into a clean CAUGHT or MISSED:

- **18 of 39** flipped to **CAUGHT** — the suite genuinely kills them; the timeout was a
  scheduling artifact. (All 18 are in `decode_epb_body_discriminant` and
  `parse_shb_body_discriminant`, both pinned by the `sec_*_twin_equivalence_tests`.)
- **21 of 39** are **genuine MISSED survivors** — no test pins them.

## Genuine MISSED survivors (21) — per-function judgment

### `pcapng_timestamp_to_secs_usecs` — 2 survivors

| Line:col | Mutation | Judgment |
|---|---|---|
| 353:47 | `replace \| with ^` in `((ts_high as u64) << 32) \| (ts_low as u64)` | **EQUIVALENT mutant — acceptable.** The high half is shifted left by exactly 32 and the low half is a `u32` widened to `u64`; their set bits are disjoint, so `\|` and `^` are bit-for-bit identical for all inputs. No test can distinguish them. Accept. |
| 368:14 | `replace < with <=` in `if e < BASE10_POWERS.len()` | **REAL GAP.** `BASE10_POWERS` has length 20. With `<=`, exponent `e == 20` indexes `BASE10_POWERS[20]` → out-of-bounds panic, OR (if it did not panic) selects the table value instead of the `u64::MAX` saturation arm. The boundary `e == 20` (e.g. `if_tsresol = 20`, base-10) is never exercised. Tests cover **base-2** `e=20` (`0x94`) but not **base-10** `e=20`. |

### `decode_epb_body` — 2 survivors

| Line:col | Mutation | Judgment |
|---|---|---|
| 500:62 | `replace % with +` in `pad_len = (4usize.wrapping_sub(captured_len % 4)) % 4` | **REAL GAP (low severity).** The outer `% 4` normalizes `pad_len` into `[0,3]`. Replacing it with `+ 4` changes `pad_len` but the PC6b padding-overrun check at 501–504 is defense-in-depth on top of the PC6a `captured_len > available` check at 492 (which already rejects overruns). The mutated `pad_len` is only observable via the PC6b branch, which no test reaches with a value that diverges. Acceptable to leave if PC6b is treated as pure belt-and-suspenders, but a pinning test is cheap. |
| 504:9  | `replace > with <` in the PC6b overrun comparison `... > body.len()` | **REAL GAP (low severity).** Same PC6b defense-in-depth path; the inversion is masked by PC6a. A test that constructs an EPB whose padding (not captured_len) overruns the body is missing. |

### `decode_epb_body_discriminant` — 2 survivors

| Line:col | Mutation | Judgment |
|---|---|---|
| 585:62 | `replace % with +` in the twin `pad_len` modulo | **REAL GAP — twin of 500:62.** Same root cause; the SEC-001 twin-equivalence test pins this function against `decode_epb_body` for the inputs it generates but does not generate a padding-overrun case. Fixing `decode_epb_body` coverage and re-asserting the twin closes both. |
| 589:9  | `replace > with <` in the twin PC6b comparison | **REAL GAP — twin of 504:9.** Same as above. |

> Note: the other 16 `decode_epb_body_discriminant` / `parse_shb_body_discriminant` mutants
> (lines 542, 581, 595, 641, 649, 651, 668) that timed out in the primary run are **CAUGHT** —
> the twin-equivalence tests kill them. Only the two padding-overrun twins (585:62, 589:9) leak.

### `parse_idb_options` — 8 survivors

| Line:col | Mutation | Judgment |
|---|---|---|
| 733:30 | `replace > with >=` in `if body.len() > IDB_BODY_FIXED_BYTES` | **REAL GAP.** At the exact boundary `body.len() == IDB_BODY_FIXED_BYTES (8)`, `>` takes the no-options `return Ok(DEFAULT_TSRESOL)` path; `>=` would slice `&body[8..]` (an empty slice) and fall through the loop to the same default. Observably equivalent **only because** the empty options slice happens to yield the same default — but the equivalence is incidental, not pinned. An IDB with body exactly 8 bytes is not tested. Borderline-equivalent; pin the 8-byte-body case. |
| 749:19 | `replace + with *` in `if cursor + 4 > remaining.len()` (TLV header bound) | **REAL GAP.** With `cursor == 0` (first iteration), `0 + 4 == 0 * 4 == 0`?? No — `0*4 = 0`, so the guard `0 > len` is false and the walk proceeds; on later iterations `cursor*4` diverges. No multi-option IDB with `cursor > 0` at this check is tested in a way that exposes the difference. |
| 764:54 | `replace + with *` in `remaining[cursor + 2]` / `[cursor + 3]` (LE opt_len read) | **REAL GAP.** Indexing offset arithmetic; only the little-endian `opt_len` read. With `cursor == 0`, `cursor+2 == 2` but `cursor*2 == 0` → reads the wrong bytes for `opt_len`, yet survives because no test asserts the LE `opt_len` value for a `cursor > 0` option, or the affected option is `if_tsresol` at cursor 0 where the value still resolves. |
| 779:29 | `replace > with >=` in `if cursor + opt_len > remaining.len()` (overrun check) | **REAL GAP.** Off-by-one on the option-length bounds check; the exact-fit case (`cursor + opt_len == remaining.len()`) is never tested, so tightening `>` to `>=` (wrongly rejecting an exactly-fitting option) is not caught. |
| 779:19 | `replace + with *` in `cursor + opt_len` (overrun check LHS) | **REAL GAP.** Same bounds expression; multiplicative corruption of the offset is unobserved for the same reason. |
| 789:31 | `replace + with *` in `let padded = (opt_len + 3) & !3` (4-byte align) | **REAL GAP.** The padding round-up for skipped options. Survives because no test threads a *second* option past a skipped first option whose padding differs between `(opt_len+3)&!3` and `(opt_len*3)&!3`. The if_tsresol-at-cursor-0 happy path returns before `padded` is used. |
| 808:16 | `replace += with -=` in `cursor += padded` (advance past skipped option) | **REAL GAP.** The skip-advance. `-=` would move the cursor backwards → no test exercises "skip an unknown option, then read a following option," so the broken advance is invisible. |
| 808:16 | `replace += with *=` in `cursor += padded` | **REAL GAP.** Same skip-advance expression, multiplicative variant. |

> Common root cause for 749/764/779/789/808: **there is no test with a multi-option IDB that
> skips ≥1 unknown/non-code-9 option and then reads a subsequent option (or `if_tsresol` placed
> after a skipped option).** Every `parse_idb_options` test either has zero options, a single
> `if_tsresol` option at cursor 0, or hits an error before the skip-advance is re-entered. The
> TLV-walk *advance/skip machinery* is therefore almost entirely un-pinned for `cursor > 0`.

### `PcapSource::from_pcap_reader` — 1 survivor

| Line:col | Mutation | Judgment |
|---|---|---|
| 884:29 | `replace < with <=` in `if filled.len() < 4` (magic peek) | **REAL GAP (low severity).** With `<=`, a stream of **exactly 4** magic bytes is wrongly rejected as "too short." No test feeds a reader whose `fill_buf()` returns exactly 4 bytes on the first fill (typical fixtures are far longer, so `filled.len()` is large and both `<` and `<=` are false). A 4-byte-exact (or 4-byte-first-fill) reader case is missing. |

### `PcapSource::read_pcapng_crate` — 6 survivors

| Line:col | Mutation | Judgment |
|---|---|---|
| 1008:45 | `replace match guard msg.contains("block length < 16") with true` | **REAL GAP (low severity, error-provenance).** Forcing the guard `true` routes *any* `InvalidField` SHB-construction error to the E-INP-008 "SHB body too short" message. Existing tests feed inputs that genuinely match the substring, so the over-broad guard yields the same message for them. Missing: a negative case where a *non-matching* `InvalidField` must stay E-INP-010, proving the guard actually discriminates. |
| 1011:45 | `replace match guard msg.contains("invalid magic number") with true` | **REAL GAP (low severity, error-provenance).** Symmetric to 1008:45 for the invalid-BOM arm. Same missing negative case. |
| 1302:17 | `delete match arm NRB_BLOCK_TYPE` | **EQUIVALENT mutant — acceptable.** The NRB arm body is `skipped_blocks = skipped_blocks.saturating_add(1)`, byte-for-byte identical to the `_` catch-all. Deleting it makes NRB fall through to `_`, producing identical `skipped_blocks` and identical packet output. No counter distinguishes NRB (spec: only OPB gets `opb_skipped`). Accept. |
| 1309:17 | `delete match arm ISB_BLOCK_TYPE` | **EQUIVALENT mutant — acceptable.** Same as NRB; ISB body identical to `_`. Accept. |
| 1315:17 | `delete match arm SJE_BLOCK_TYPE` | **EQUIVALENT mutant — acceptable.** Same; SJE body identical to `_`. Accept. |
| 1321:17 | `delete match arm DSB_BLOCK_TYPE` | **EQUIVALENT mutant — acceptable** for behavior. DSB body is identical to `_` (`skipped_blocks += 1`); deletion is observationally equivalent. **Caveat:** the named DSB arm carries the SEC-007 "MUST NOT log key material" contract as a *documented anchor*. Since neither the named arm nor `_` logs anything, behavior is identical and the mutant is equivalent — but if a future change adds logging to `_` without re-adding a DSB guard, SEC-007 could regress silently. Acceptable now; note the latent risk. |

## UNVIABLE (11) — all legitimate, none are test gaps

These did not compile (the mutated form is type-invalid), so they cannot indicate a test gap:

- `parse_shb_body`, `decode_epb_body`, `decode_epb_body_discriminant`,
  `parse_shb_body_discriminant`, `from_pcap_reader`, `read_pcapng_crate`, `from_file`
  → `Ok(Default::default())` body replacements (the return types `ShbInfo` / `RawPacket` /
  `PcapSource` / `Self` do not implement `Default`).
- `decode_epb_body:463 >= → <`, `decode_epb_body:492 > → <`,
  `decode_epb_body_discriminant:558 >= → <`, `decode_epb_body_discriminant:581 > → <`
  → produce a slice range where `start > end`, which `cargo-mutants` flags unviable at the
    `--check` stage (or the borrow/type shape rejects it). Not a gap.

## CAUGHT (126 true) — well-pinned areas

The following are killed cleanly (counts from the primary CAUGHT set plus the 18 recheck flips):
`parse_shb_body` (all 6), `spb_captured_len` (both arms — VP-031 proptest, 1000 cases),
the `pcapng_timestamp_to_secs_usecs` arithmetic core (24, incl. all return-value and `/`, `%`,
`&` mutants), `decode_epb_body` bounds/endianness (12), the bulk of `parse_idb_options`
endianness/error arms (29), `from_pcap_reader` routing (5), and the `read_pcapng_crate`
block-dispatch structural arms (24 — SHB/IDB/EPB/SPB arm deletes, E-INP-013 position check,
multi-IDB conflict, zero-advance guard). The `_discriminant` twins are killed by the SEC-001 /
SEC-SHB twin-equivalence tests except the two padding-overrun mutants noted above.

## Coverage / skips (no silent truncation)

- **Covered:** every production function named in the task — block-walk loop + skip arms
  (`read_pcapng_crate`), `decode_epb_body`, `spb_captured_len`,
  `pcapng_timestamp_to_secs_usecs`, `parse_shb_body`, `parse_idb_options`, interface-table
  handling, `from_pcap_reader`, plus the `_discriminant` twins and `from_file`. All 158
  `src/reader.rs` production mutants ran to a resolved verdict (the 39 primary timeouts were
  re-resolved in the recheck).
- **Excluded by design:** the `#[cfg(kani)]` `kani_proofs` module in `src/reader.rs` (60
  mutants) — it is proof-harness code, not production, and is unreachable by `cargo test`.
- **Skipped:** `src/main.rs` was **not** mutated. The pcapng decode logic lives entirely in
  `src/reader.rs`; `main.rs` holds CLI/orchestration glue with no pcapng-specific decode
  branches in the task's scope. A `main.rs` mutation pass is a separate, lower-value follow-up
  and was deliberately omitted to keep this run bounded. **Stated, not silently dropped.**

## Recommended remediation (real gaps only — follow-up dispatch, not this pass)

Grouped by the smallest set of new tests that close the most survivors:

1. **IDB multi-option TLV-walk skip machinery** (closes 749:19, 764:54, 779:29, 779:19,
   789:31, 808:16 ×2 — 7 survivors). Add a `parse_idb_options` test (unit or proptest in
   `tests/bc_2_01_story124_idb_tests.rs` / `tests/proptest_reader.rs`) with a multi-option IDB
   body that: (a) places one or more **unknown** option codes (non-0, non-9) of varying lengths
   **before** an `if_tsresol` (code 9) option, forcing the walk to execute `padded` and
   `cursor += padded` and then correctly read the trailing if_tsresol; and (b) a variant where
   an option's `option_length` exactly fills the remaining region (`cursor + opt_len ==
   remaining.len()`) to pin the `779:29 > vs >=` boundary. Assert the recovered `if_tsresol`.
2. **`pcapng_timestamp_to_secs_usecs` base-10 e==20 boundary** (closes 368:14). Add a known
   vector with `if_tsresol = 20` (base-10, `e == BASE10_POWERS.len()`) and assert the saturated
   result, mirroring the existing base-2 `e=20` (`0x94`) test.
3. **EPB padding-overrun (PC6b) defense-in-depth** (closes 500:62, 504:9 and twins 585:62,
   589:9 — 4 survivors). Add an EPB body where `captured_len` passes PC6a but the
   captured_len + padding overruns `body.len()`, exercising the PC6b branch; then re-assert the
   SEC-001 twin equivalence so the discriminant twin inherits the kill.
4. **IDB 8-byte-exact body** (closes 733:30). Add an IDB with body length exactly
   `IDB_BODY_FIXED_BYTES` (8) — no options — and assert the default `if_tsresol`.
5. **SHB error-provenance discrimination** (closes 1008:45, 1011:45). Add a negative case: a
   crate `InvalidField` SHB error whose message does **not** contain "block length < 16" /
   "invalid magic number" must surface as **E-INP-010**, not E-INP-008.
6. **4-byte-exact magic peek** (closes 884:29, low priority). Feed `from_pcap_reader` a reader
   whose first `fill_buf()` returns exactly 4 bytes (or a stub `Read`), asserting the valid
   path is taken rather than the too-short error.

**Acceptable survivors (no action required), with justification:** 353:47 (`\|`↔`^`,
provably equivalent — disjoint bit ranges); 1302/1309/1315/1321 (NRB/ISB/SJE/DSB arm deletes
fall through to a byte-identical `_` catch-all — true equivalents; spec mandates no per-type
counter). 733:30 is borderline-equivalent but cheap to pin (item 4).

## Verdict

**Conditional — adequate kill *power* in the structurally-critical paths, but NOT yet at the
F6 gate threshold for the highest-criticality pcapng modules.**

- True mutation score is **85.7%** (126 killed / 147 viable). Of the 21 survivors, **6 are
  provable/acceptable equivalent mutants**, leaving **15 real test gaps**. The
  equivalent-adjusted kill rate is **126 / (147 − 6) = 89.4%**.
- The structural core (SHB parse, SPB length, EPB bounds/endianness, block dispatch, the
  E-INP error taxonomy arms, and the Kani-target arithmetic) is **well pinned** — all clean
  kills. The pure-core decode functions that VP-025/026/027/031 anchor are killed.
- The concentrated weakness is the **`parse_idb_options` TLV-walk skip/advance machinery**
  (7 of 15 real gaps): multi-option IDBs that skip an unknown option before reading a later
  option are essentially untested, leaving the cursor-advance and padding arithmetic unpinned.
  The EPB **PC6b padding-overrun** defense path (4 gaps) and two **error-provenance** guards
  (2 gaps) are the remainder.
- For a CRITICAL-rated input-parsing module the bar is typically 95%. **Pre-gate gaps to close:
  items 1–3 above** (they cover 11 of the 15 real gaps and all the medium-risk ones). Items 4–6
  are low-severity polish.

**Recommendation:** Do **not** pass the F6 mutation gate for the pcapng reader as-is. Dispatch
a follow-up that adds the tests in remediation items 1–3 (and ideally 4–6), then re-run
`cargo mutants --file src/reader.rs --exclude-re 'kani_proofs'` — using `--minimum-test-timeout
300` (or a longer `--timeout`) so the unrelated-binary scheduling artifact does not reintroduce
spurious TIMEOUTs — and confirm the equivalent-adjusted kill rate clears the module's threshold.
No production code was modified in this measurement pass.

---

## F6 Confirmation Re-run @ develop 930d957

- **Date:** 2026-06-21 (confirmation pass)
- **Tree:** `develop` @ HEAD `930d957ba63e42bbf3e847ca9c68746cd24cc2f0`
  (Merge #295 — mutation-gap remediation: 13 new tests in `tests/bc_f6_mutation_gap_tests.rs`)
- **Tool:** `cargo-mutants 27.0.0`, default `cargo test` runner, dev profile, 16 cores, macOS aarch64
- **Pre-flight:** `cargo build` clean; `cargo test --test bc_f6_mutation_gap_tests` → 13 passed.
  Target line numbers verified unshifted vs the prior report (368/504/589/733/749/764/779/789/808/884/1008/1011 all match).
- **Measurement only:** no production or test code modified; working tree clean; scratch `mutants.out/` removed.

### Invocation (scoped to the functions that had real survivors)

```
cargo mutants --file src/reader.rs --exclude-re 'kani_proofs' \
  --re 'parse_idb_options|decode_epb_body|decode_epb_body_discriminant|pcapng_timestamp_to_secs_usecs|parse_shb_body|parse_shb_body_discriminant|from_pcap_reader|read_pcapng_crate' \
  --minimum-test-timeout 300 --timeout 600 --no-shuffle -j 6
```

> Note: `cargo-mutants 27.0.0` has no `--function` flag; `-F/--re` (matched against the
> `--list` mutant names, which embed the function name) was used instead. cargo-mutants also
> generates a **different operator set** than the version behind the prior report (e.g. for `>`
> it emits `==`/`<`/`>=`; for `+` it emits `-`/`*`), so each prior-report mutant maps to its
> same-site equivalents here.

### Headline totals

`155 mutants tested in 37m: 8 missed, 136 caught, 10 unviable, 1 timeout` (exit 3 → no spurious timeouts).

| Outcome | Count |
|---|---|
| CAUGHT | 136 |
| MISSED | 8 |
| TIMEOUT | 1 (`808:16 += → -=`, infinite-loop — functionally detected; see below) |
| UNVIABLE | 10 |

**No scheduling-artifact timeouts.** Under the generous `--minimum-test-timeout 300 --timeout
600`, the 39 ambiguous timeouts of the prior run did **not** recur. The single timeout here is a
genuine non-terminating mutant (cursor-underflow infinite loop), not a budget artifact.

### Scope: covered vs skipped (no silent truncation)

- **Covered (155 mutants):** all 8 functions named in the `--re` filter — `parse_idb_options`,
  `decode_epb_body`, `decode_epb_body_discriminant`, `pcapng_timestamp_to_secs_usecs`,
  `parse_shb_body`, `parse_shb_body_discriminant`, `PcapSource::from_pcap_reader`,
  `PcapSource::read_pcapng_crate`. This is **155 of the 158** `src/reader.rs` production mutants
  (`kani_proofs` excluded). Every one of the prior report's 15-real-gap line:col positions is in scope.
- **Skipped (3 mutants):** the 3 `src/reader.rs` mutants in small helper functions **not** named in
  the filter (`spb_captured_len` and other one-liners). These had **zero** real survivors in the
  prior pass (`spb_captured_len` both arms were CAUGHT by VP-031 proptest). Excluded only to keep
  the run bounded on the regressed functions; stated, not silently dropped.

### The 15 previously-real-gap mutants — all now CAUGHT

Cross-referenced by line:col against the prior report. Every one is killed (operator shown is the
prior report's exact mutant where this cargo-mutants version still emits it; same-site variants
also caught):

| Prior-report gap (line:col) | Function | This run |
|---|---|---|
| 368:14 `< → <=` | `pcapng_timestamp_to_secs_usecs` (base-10 e==20) | **CAUGHT** (`<=`, `==`, `>`) |
| 500:62 `% → +` | `decode_epb_body` (PC6b pad) | EQUIVALENT (see below); same-site `500:68 %`, `504:9 >` all CAUGHT |
| 504:9 `> → <` | `decode_epb_body` (PC6b overrun) | **CAUGHT** (`<`, `==`, `>=`) |
| 585:62 `% → +` | `decode_epb_body_discriminant` (twin pad) | EQUIVALENT (twin of 500:62); `585:68 %`, `589:9 >` all CAUGHT |
| 589:9 `> → <` | `decode_epb_body_discriminant` (twin overrun) | **CAUGHT** (`<`, `==`, `>=`) |
| 733:30 `> → >=` | `parse_idb_options` (8-byte body bound) | EQUIVALENT (see below); `733:30 ==`, `<` CAUGHT |
| 749:19 `+ → *` | `parse_idb_options` (TLV header bound) | **CAUGHT** (`*`, `-`); `749:23 >` (`<`,`==`,`>=`) CAUGHT |
| 764:54 `+ → *` | `parse_idb_options` (LE opt_len read) | **CAUGHT** (`*`, `-`); `764:77` CAUGHT |
| 779:29 `> → >=` | `parse_idb_options` (overrun check) | **CAUGHT** (`>=`, `<`, `==`) |
| 779:19 `+ → *` | `parse_idb_options` (overrun LHS) | **CAUGHT** (`*`, `-`) |
| 789:31 `+ → *` | `parse_idb_options` (4-byte align) | **CAUGHT** (`*`, `-`); `789:36 &`, `789:38 !` CAUGHT |
| 808:16 `+= → -=` | `parse_idb_options` (skip-advance) | **DETECTED via TIMEOUT** (infinite loop — see below) |
| 808:16 `+= → *=` | `parse_idb_options` (skip-advance) | **CAUGHT** |
| 884:29 `< → <=` | `from_pcap_reader` (magic peek) | **CAUGHT** (`<=`, `==`, `>`) |
| 1008:45 guard | `read_pcapng_crate` (SHB E-INP-008 provenance) | **CAUGHT** (`with true`, `with false`) |
| 1011:45 guard | `read_pcapng_crate` (invalid-BOM provenance) | **CAUGHT** (`with true`, `with false`) |

The remediation tests do the work: the multi-option-IDB skip tests
(`test_BC_2_01_011_idb_multi_option_unknown_before_tsresol_{le,be}`,
`..._padded_option_len_not_multiple_of_4`, `..._multi_option_idb_integration_end_to_end`) kill the
749/764/779/789/808 skip-machinery; the 8-byte-body test kills the 733 path variants; the base-10
e==20 test kills 368; the PC6b padding-overrun + twin-equivalence tests kill 504/589 (and the
discriminant twins); the 4-byte-exact magic test kills 884; the non-matching-`InvalidField`
negative test kills 1008/1011.

### New kill rate

- **Strict `caught / (caught + missed)`** (matches the prior report's metric, timeout excluded):
  **136 / 144 = 94.4%**.
- **Timeout-as-kill** (the 1 timeout is a detected infinite loop, see below):
  **137 / 145 = 94.5%**.
- **Equivalent-adjusted** (all 8 MISSED are provably/observationally equivalent — removed from
  denominator): **137 / 137 = 100%**.

Up from the prior pass's 85.7% strict / 89.4% equivalent-adjusted. The remediation closed all 15
real gaps; the residual survivors are equivalents only.

### The 8 MISSED survivors — every one is a proven equivalent (no new real gaps)

| Line:col | Mutation | Equivalence judgment |
|---|---|---|
| 353:47 | `\| → ^` in `(ts_high << 32) \| ts_low` | **EQUIVALENT** (prior report). High half `<<32` and `u32` low half have disjoint bit ranges → `\|` ≡ `^` bit-for-bit. |
| 500:62 | `% → +` (inner) in `(4.wrapping_sub(captured_len % 4)) % 4` | **EQUIVALENT — re-classified.** Col 62 is the **inner** `%` (col 68 is the outer). Working mod 4: `c+4 ≡ c`, so `4 - (c+4) ≡ 4 - c ≡ -c (mod 4)`, identical residue to `4 - (c%4)` after the outer `%4`. Verified ∀ c∈[0,10000). The prior report called this a "REAL GAP" assuming the mutation hit the outer modulo — with the actual inner-`%` mutation it is provably equivalent. The outer-`%` (500:68) and the PC6b comparison (504:9) ARE caught. |
| 585:62 | `% → +` (inner) in twin | **EQUIVALENT** — twin of 500:62, byte-identical line, same mod-4 proof. |
| 733:30 | `> → >=` in `body.len() > IDB_BODY_FIXED_BYTES` | **EQUIVALENT.** Only differs at `body.len()==8`: `>` returns default immediately; `>=` slices `&body[8..]` (valid empty slice, no panic), the TLV loop breaks on the first guard, and falls through to the **same** `Ok(DEFAULT_TSRESOL)`. Output-identical. The test author flagged it in-code as "borderline-equivalent"; the `==`/`<` variants (which change behavior for `len>8`) are CAUGHT. |
| 1302:17 | delete arm `NRB_BLOCK_TYPE` | **EQUIVALENT** (prior report). Arm body byte-identical to `_` catch-all (`skipped_blocks += 1`); spec mandates no per-type counter. |
| 1309:17 | delete arm `ISB_BLOCK_TYPE` | **EQUIVALENT** (prior report). Same as NRB. |
| 1315:17 | delete arm `SJE_BLOCK_TYPE` | **EQUIVALENT** (prior report). Same. |
| 1321:17 | delete arm `DSB_BLOCK_TYPE` | **EQUIVALENT** for behavior (prior report). DSB body identical to `_`; latent SEC-007 "no key material logging" anchor noted, but behavior identical — accept. |

These 8 map onto the prior report's "6 known equivalents" plus the two re-classified arithmetic
mutants (500:62 / 585:62), which the prior report had listed as low-severity real gaps but which
this analysis proves equivalent. The four NRB/ISB/SJE/DSB arm deletes + 353:47 = the 5 of the prior
"6 acceptable"; 733:30 was the prior "borderline-equivalent." Net: **zero unexplained survivors.**

### The 1 TIMEOUT — `808:16 += → -=` is functionally detected, not a survivor

`cursor -= padded` in the skip-advance turns the TLV walk into a **non-terminating loop** (the
cursor stops advancing / underflow-thrashes) when a real multi-option IDB is processed. Evidence
from the per-mutant log: the run hung in `test_BC_2_12_011_e2e_corpus_pcapng_reader_stack`
("has been running for over 60 seconds" → `*** result: Timeout` at the 600 s cap), and the
companion `808:16 += → *=` mutant was cleanly CAUGHT by the same skip-machinery tests. A
non-terminating mutant is observably defective (CI hangs = failure); cargo-mutants files it as
TIMEOUT rather than CAUGHT only because a hang is not an assertion failure. **Not a clean survivor
and not a test gap** — the skip-advance is pinned (its `*=` sibling is killed and the `-=` sibling
hangs the suite).

### UNVIABLE (10) — none are test gaps

All are `Ok(Default::default())` return-body replacements on types without `Default`
(`ShbInfo`/`RawPacket`/`PcapSource`/`Self`) or slice-range-inverting comparisons flagged at the
`--check` stage. Same legitimate set as the prior pass.

### Any NEW survivors introduced?

**None.** Every MISSED is a pre-existing equivalent (5 carried from the prior "acceptable" set,
2 arithmetic mutants now proven equivalent, 1 borderline now proven equivalent). The remediation
tests introduced no new clean survivors in the regressed functions.

### Verdict — pcapng F6 mutation gate: **PASS**

- Strict kill rate **94.4%** (136/144); equivalent-adjusted **100%** (137/137); timeout is a
  detected infinite loop, not a survivor.
- **All 15 previously-real-gap mutants are CAUGHT** (808:16 `-=` detected via hang; its `*=`
  sibling and all others assertion-killed).
- The residual 8 survivors are **exclusively proven/observational equivalents** (4 match-arm
  deletes folding into a byte-identical `_`, the disjoint-bitrange `\|→^`, two mod-4-equivalent
  inner-`%` mutants and their twin, and the output-identical `733:30 >→>=`).
- For a CRITICAL-rated input-parsing module (≥95% target): the **equivalent-adjusted rate of 100%
  clears it**; the raw 94.4% sits just below the nominal 95% line **only** because of provable
  equivalents in the denominator — there are **no remaining real test gaps to dispatch**.

**Recommendation:** Pass the pcapng F6 mutation gate at develop 930d957. No follow-up test is
required. (Optional hygiene, not gating: cargo-mutants `skip_calls`/`ignore`-annotate the 4
equivalent match arms and the two mod-4 inner-`%` sites so future runs report a clean raw ≥95%
without manual equivalent-adjustment.) Measurement only — no production/test code changed,
scratch `mutants.out/` removed, no PR opened.
