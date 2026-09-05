# IEC-104 E2E Fixture-Count Gate-Entry Evidence

**Policy reference:** PG-W85-005 (wave-85 gate G1 retrospective)
**Evidence story:** STORY-182 (wave-86, E-11, 4 pts)
**Codification decision:** D-548 (2026-09-05)
**Referenced from:** CLAUDE.md `## Project References` table
**Referenced from (story frontmatter/traces_to):** `.factory/stories/STORY-182.md`

---

## Background

Wave-85 gate G1 FAILed on a stale-expectation assertion: an IEC-104 E2E test asserted
`31` findings when the correct value (post-STORY-180 timed-command detection) was `66`.
The root cause was not the stale assertion itself but the *class* of failure it exposed:
the assertion only ran — and only failed — on a fixture-bearing host. On a clean
checkout (no `tests/fixtures/local-samples/` populated), the affected test silently
skipped via a `fixture_present()` guard and reported `ok`, masking the drift from every
CI run except the one host where a human had manually placed the capture file.

STORY-182 closes the structural gap this exposed: at least one representative IEC-104
capture is **committed to the repository** (not gitignored, not fetched-on-demand), so
the E2E harness runs — and can fail — on every `cargo test` invocation, in every clean
checkout, including CI.

## Fixture Manifest vs. Committed Fixtures

Two distinct counts are tracked, and conflating them is the exact failure mode this
gate-entry evidence exists to prevent:

| Constant | Value | Meaning |
|----------|------:|---------|
| `FIXTURE_MANIFEST` | **4** entries | The full inventory of named IEC-104 capture fixtures the E2E harness knows about: `iec104.pcap`, `iec104-sq.pcapng`, `iec104-iti-diverse.pcap`, `iec104-iti-dissect.pcap`. This is a fixed literal — it does not change with what happens to be present on disk. |
| `COMMITTED_FIXTURES` | **1** entry | The subset of `FIXTURE_MANIFEST` that is tracked in git and therefore present in every clean checkout: `iec104-iti-diverse.pcap` only. |
| `FIXTURE_GATED_TESTS` | **4** | The number of E2E test functions whose pass/fail (not silent-skip) outcome is gated on fixture presence per the manifest above. |

### Committed fixture details

- **File:** `tests/fixtures/iec104-iti-diverse.pcap`
- **Size:** 13952 bytes
- **SHA-256:** `07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7`
- **Provenance / license:** CC-BY-4.0, sourced from ITI/ICS-Security-Tools (upstream
  attribution recorded in `tests/fixtures/README.md` §Licensing notice per AC-182-002).

The remaining 3 manifest entries (`iec104.pcap`, `iec104-sq.pcapng`,
`iec104-iti-dissect.pcap`) remain gitignored / locally-fetched-only per the
single-capture provenance ruling (wave-86 pass-7) — only `iec104-iti-diverse.pcap` is
committed.

## Why `#[ignore]` Was Rejected

The obvious alternative — marking fixture-dependent tests `#[ignore]` and requiring an
explicit `--include-ignored` flag — was considered and rejected:

- **Committed fixtures run on every `cargo test`, no `#[ignore]` needed.** Because
  `iec104-iti-diverse.pcap` is committed (not gitignored), the tests gated on it require
  no opt-in flag to execute — they run by default in every invocation, local or CI.
- **AC-182-005 hard-assert fails `cargo test` if a committed fixture is absent.** A
  `FIXTURE_MANIFEST.len()` exhaustiveness assertion (`test_fixture_manifest_report`)
  and a companion existence assertion for the committed fixture both run unconditionally;
  if the committed fixture is ever accidentally removed from the tree, `cargo test`
  itself fails — there is no silent-skip path to fall into.
- **The Task-8 wave-gate command surfaces the failure.** The wave-gate verification
  command:

  ```bash
  cargo test --all-targets 2>&1 | tee coverage-out.txt
  grep -qE "test result: ok" coverage-out.txt
  ```

  fails closed: if the committed-fixture hard-assert fails, `test result: ok` will not
  appear in the tee'd output and the `grep -qE` gate fails, blocking wave-gate entry.
  An `#[ignore]`-based design would have required a *second*, easily-forgotten
  `--include-ignored` invocation to ever exercise this path at all — reproducing the
  exact silent-omission failure mode STORY-182 exists to eliminate.

## Gate-Entry Rule

- **M = `FIXTURE_MANIFEST.len()` ≠ 4 BLOCKS gate entry** — a manifest-size drift (an
  entry added or removed from the named-fixture inventory without a corresponding
  story/PR) is treated as a hard gate failure, not a warning.
- **N = present-fixture count is evidence-only.** N is recorded alongside an explicit
  environment declaration — `local-samples absent` (N=1, committed fixture only) /
  `partial` (1 < N < 4) / `full` (N=4, all manifest fixtures present, e.g. a
  fixture-bearing host) — but does not by itself block or pass the gate. Any N in
  `{1, 2, 3, 4}` is legitimate depending on host; only M ≠ 4 is a defect signal.

## D-510 G1 Retrospective

The wave-85 gate G1 FAIL (STATE.md D-510, 2026-07-24) was traced to a stale-expectation
assertion (`31` vs. the correct `66` findings) that only ran to failure on a
fixture-bearing host — on a clean checkout the same assertion silently skipped and
reported `ok`. This retrospective motivated STORY-182's manifest-driven,
fail-if-missing design: rather than trying to catch every future stale-expectation edit
by review discipline alone, the harness now guarantees that *some* representative
capture — and therefore *some* real assertion surface — is exercised on every checkout,
independent of host fixture population.

---

**Status:** Evidence artifact — created at STORY-182 delivery (D-548, 2026-09-05).
No further updates expected unless `FIXTURE_MANIFEST` composition changes in a future
story.
