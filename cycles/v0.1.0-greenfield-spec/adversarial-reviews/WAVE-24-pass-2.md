# Adversarial Review — WAVE-24 Pass 2

- **Target:** WAVE-LEVEL convergence — STORY-087 + STORY-096 (Wave 24)
- **Scope:** full
- **Develop tip reviewed:** `9954d44` (unchanged from pass 1 — frozen tip)
- **Lenses:** CONSISTENCY, INTEGRATION-STATIC, TRACEABILITY — re-attacked with fresh angles
- **Date:** 2026-05-31
- **Fresh-context disclosure:** same harness limitation as pass 1 (recorded there). Pass 2 deliberately probed angles pass 1 did not exercise (test-falseness / mutation-resistance / EC-scenario-match / empirical per-test attribution) to maximize novelty.

## Fresh Angles Probed This Pass

Pass 1 established the static scaffolding is sound. Pass 2 attacks the harder
question: **are the tests genuine discriminators, or could they false-green?**
And: **do the EC citations reproduce the EXACT BC scenarios (W12.L1 lesson)?**

| Angle | Probe | Result |
|-------|-------|--------|
| 096 src-walk coverage guard correctness | `find src -name '*.rs' \| wc -l` vs guard `>=20` (comment claims 24) | **24 files**; guard `>=20` is a sound non-trivial lower bound that catches a mis-rooted/empty walk. ✓ |
| 096 EC-002 genuine discriminator | Is `--dns` (in `analyze --dns --beacon`) a VALID flag, so the error attributes to `--beacon` not `--dns`? | `dns: bool #[arg(long)]` exists on `Analyze` (cli.rs:121-122) — `--dns` valid; error genuinely fires on `--beacon`. Not an accidental pass. ✓ |
| 096 AC-009 `-v`/`-a` claim | Docstring claims `-a` short is taken by `--all`, hence `-v` is undeclared | `#[arg(short, long)] all: bool` (cli.rs:137-138) confirms `-a`=`--all`; no `-v`. Claim accurate. ✓ |
| 096 EC-004 enum match | `Analyze { http, targets, .. }` arm reachable + `Summary` arm is the panic branch | `Commands::Analyze { targets, dns, http, tls, mitre, all }` + `Summary { targets, hosts }` (cli.rs:113-160) — match arms valid, exhaustive. ✓ |
| EC-scenario-match 096 (W12.L1) | Does `test_EC_001_threats_before_subcommand` reproduce the EXACT BC EC scenario? | BC-2.13.001 **EC-002** = `wirerust --threats analyze test.pcap`; test argv = `["wirerust","--threats","analyze","test.pcap"]` — exact match, and docstring correctly cites "BC-2.13.001 EC-002" (not EC-001). ✓ |
| EC-scenario-match 087 (W12.L1) | Do 087 EC docstrings cite the RIGHT EC source given story-EC ≠ BC-EC numbering? | See F-W24-P2-001 below — docstrings correctly self-scope to `STORY-087 EC-NNN`; no mis-citation, but the numbering divergence is a latent trap (NOTE). |
| Empirical per-test attribution | Ran both test binaries isolated | 087: 16/16 green; 096: 14/14 green; each named test passes independently. ✓ |
| Absence-test mutation resistance | Would adding `pub threats: bool` / `struct C2BeaconAnalyzer` actually break the tests? | 096 `test_threats_field_absent_from_cli` checks `pub threats` + `long="threats"` + indented `threats:`; `test_beacon_analyzer_absent_from_src` recursively walks all 24 src files for `struct/impl (C2)?BeaconAnalyzer`; `test_bpf_filter_absent` uses structural TOML dependency-KEY matching with a positive sanity-guard on `pcap-file`. All three are mutation-resistant by construction and explicitly avoid LESSON-P1.04 comment false-positives. ✓ |

## Findings

### F-W24-P2-001 — Story-087 EC numbering diverges from BC-2.12.005 EC numbering for the same scenarios (latent reader trap)  [LOW / NOTE]
- **Lens:** TRACEABILITY / CONSISTENCY.
- **Evidence:** Same physical scenarios carry different EC IDs in the two documents:
  - `--small-segment-max-bytes 0` → STORY-087 **EC-002**, but BC-2.12.005 **EC-005**.
  - `no reassembly flags` → STORY-087 **EC-005**, but BC-2.12.005 **EC-001**.
  - `--overlap-threshold 255` → STORY-087 **EC-003**; BC-2.12.005 has no matching EC (its EC-003 is the *256-reject*, which the story routes through AC-008 instead).
- **Mitigating fact (why this is only a NOTE, not an actionable defect):** the test docstrings are scrupulously correct — they cite `STORY-087 EC-NNN` (the story's own EC table) for the four EC tests, and `BC-2.12.007 EC-003` for AC-012. There is **no mis-citation**; the W12.L1 EC-scenario-match discipline is satisfied because every cited EC ID resolves to the table it names and the argv reproduces that exact scenario.
- **Why record it:** a future maintainer cross-referencing "EC-005" across the two docs could conflate the no-flags case (story) with the max-bytes case (BC). This is a latent trap, not a current error.
- **Severity rationale:** LOW/NOTE — zero correctness impact; the disambiguation is already explicit in code. No fix required for convergence; if desired, a one-line clarifying note in STORY-087's EC table ("story-local IDs; see BC-2.12.005 for BC-side EC numbering") would close the trap.

### (Carried) F-W24-P1-001 — STATE.md/STORY-INDEX stale (STORY-096 merged, recorded pending)  [MEDIUM]
- **Status at pass 2:** STILL OPEN. STATE.md L37/40/41 + STORY-INDEX L77 unchanged (state-manager update not yet dispatched). Re-confirmed against tip `9954d44` (096 merged via #165). Carried forward; not re-counted as a new finding (monotonicity).

### (Carried) F-W24-P1-002 — FSR rows cite `cli_tests.rs` in both stories  [LOW]
- **Status at pass 2:** STILL OPEN (doc drift, non-blocking). Carried forward.

## New defects this pass: **1** (F-W24-P2-001, LOW/NOTE).
## Carried-open: 2 (1 MEDIUM, 1 LOW — both documentation/state, non-test-correctness).

## Trajectory
- Pass 1: 2 findings (1 MEDIUM, 1 LOW).
- Pass 2: 1 NEW finding (LOW/NOTE) + 2 carried-open.
- **New-defect trajectory 2 → 1 (decreasing).** No new MEDIUM/HIGH/CRITICAL. The fresh-angle attack on test-falseness and EC-scenario-match found the tests to be genuine, mutation-resistant discriminators with exact scenario reproduction — the strongest possible evidence the wave deliverable is sound. Remaining open items are all documentation/state reconciliation.

## Policy spot-checks (deltas from pass 1)
- DF-AC-TEST-NAME-SYNC-001 v2 (unique-resolution + line-anchor): re-verified — no AC `**Test:**` line carries a line anchor; all resolve to a unique fn. ✓
- W12.L1 EC-scenario-match sub-rule: explicitly verified for both stories — satisfied (see angles table). ✓
- DF-TEST-NAMESPACE-001 mutation-resistance: the 096 absence tests are the strongest namespace-independent absence proofs (recursive src walk + structural TOML key match). ✓
