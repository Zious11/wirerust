---
document_type: maintenance-sweep-finding
sweep: 4
sweep_name: Holdout Scenario Freshness
run: maint-2026-07-21
producer: holdout-evaluator
timestamp: 2026-07-21T00:00:00Z
binary: "wirerust 0.13.0 (release build, exit 0)"
prior_run: .factory/maintenance/holdout-freshness-2026-07-11.md
information_asymmetry: "Evaluated using ONLY the public CLI surface of the built release binary, the holdout scenarios in .factory/holdout-scenarios/, and the fixtures in .factory/holdout-fixtures/. src/, specs, and implementation notes were NOT read."
---

# Holdout Scenario Freshness Sweep — maint-2026-07-21, Sweep 4

- **Run:** maint-2026-07-21, Sweep 4 (Holdout Scenario Freshness)
- **Binary:** `target/release/wirerust` — `wirerust 0.13.0` (release build, exit 0)
- **Prior run:** `holdout-freshness-2026-07-11.md` (evaluated `wirerust 0.12.0`)
- **Information-asymmetry constraint honored:** evaluated using ONLY the holdout
  scenarios (`.factory/holdout-scenarios/`), holdout fixtures
  (`.factory/holdout-fixtures/`), and the public CLI surface of the built binary.
  `src/`, specs, and implementation notes were NOT read.
- **Product delta since 2026-07-11 (v0.12.0 → v0.13.0):** the **IEC-104 feature**
  landed (STORY-167..STORY-174, waves 76–84; back-merged in #418). This added the
  `--iec104` analyzer (IEC 60870-5-104, TCP/2404), a T0881 MITRE catalog entry, and
  promoted **IEC 60870-5-104 to a *supported* protocol** in the coverage catalog.
  The catalog partition shifted from **7 supported / 23 unsupported** (v0.12.0) to
  **8 supported / 22 unsupported** (v0.13.0); total stays 30.

## Summary Counts

| Metric | Count |
|--------|-------|
| Run this sweep | 22 |
| PASS | 18 |
| FAIL-STALE (intentional change) | 4 (HS-087, HS-123, HS-125, HS-132) |
| FAIL-BUG-SUSPECT | 0 |
| NOT-RUNNABLE (within constraint) | 172 |
| Skipped (silent cap) | 0 |

"NOT-RUNNABLE" = scenario requires a crafted protocol-specific pcap (TCP
reassembly/evasion vectors, HTTP/TLS/DNS/MITRE payloads, pcapng framing byte-vectors,
ENIP/IEC-104 frames) or a scenario file not present in `.factory/holdout-scenarios/`.
From the constrained position (no `src/`, only the fixtures in `holdout-fixtures/`)
these cannot be executed without risking false verdicts from an incorrectly hand-crafted
fixture. This is an explicit boundary of the sweep, not a silent cap.

## What Was Run (22 scenarios)

Runnable universe under the constraint = protocols catalog (HS-123–126), coverage-gaps
(HS-127–132), and CLI-parse greenfield scenarios that need no protocol-specific payload
(HS-084–088, 094–097, 100). Fixtures used: the 8 in `holdout-fixtures/` plus a throwaway
`.pcapng`-named copy of `hs-empty.pcap` to exercise HS-087 Part C directory expansion.

| HS ID | Verdict | Note |
|-------|---------|------|
| HS-084 | PASS | Missing `<TARGETS>` → exit 2 (clap usage error). |
| HS-085 | PASS | `--reassemble` + `--no-reassemble` rejected (exit 2). |
| HS-086 | PASS | Removed flag `--threats` rejected by clap (exit 2). |
| HS-088 | PASS | Output-routing precedence unchanged by v0.13.0 (re-confirmed via HS-085 conflict handling). Part C decode-error counting not run (needs malformed pcap). |
| HS-094 | PASS | `--overlap-threshold 256` rejected (exit 2); 255 accepted (exit 0). |
| HS-095 | PASS | `--http` on unclassified TCP/9600 flow → `unclassified_flows` present in reassembly summary JSON. |
| HS-096 | PASS | `NO_COLOR=""` → 0 ESC bytes in report. |
| HS-097 | PASS | Non-existent target → `Error: Target not found: /tmp/...` verbatim, exit 1. |
| HS-100 | PASS | `summary --output-format json` protocol keys are Debug CamelCase (`["Udp"]`), not uppercase. |
| HS-124 | PASS | Terminal EtherTypes canonical: GOOSE 0x88B8 (35000), SV 0x88BA (35002), POWERLINK 0x88AB (34987), EtherCAT 0x88A4 (34980), PROFINET RT/DCP 0x8892 (34962). |
| HS-126 | PASS | Port-102 collision footnote names all four (S7comm/S7comm-plus/MMS/ICCP) on default; absent on `--supported`. |
| HS-127 | PASS | `--all` alone has no `coverage_gaps` key; `--coverage-gaps` adds it; `protocols --coverage-gaps` exits 2. |
| HS-128 | PASS | `coverage_gaps` object form `{caveat_l2, entries}`; `caveat_l2` non-null on empty pcap with 0 entries. |
| HS-129 | PASS (maint-note carried) | Case A: BACnet UDP/47808 → `known-unsupported`/`BACnet/IP`. Case C/D dual-gate note from FINDING-2 (2026-07-11) still applies — Case C/D commands omit the analyzer flag. |
| HS-130 | PASS | With `--http`: TCP/102 → `known-unsupported` with non-null `collision_note`. |
| HS-131 | PASS | TCP/53 with `--http` → state `unknown` (DNS UDP-only in catalog; supported-not-counted). |
| HS-087 | **FAIL-STALE** | Part C stale (carried from 2026-07-11): directory expansion INCLUDES `.pcapng` (observed `only.pcapng` processed) — greenfield v0.1.0 ".pcap only" expectation predates ADR-009 pcapng-reader. NOT YET FIXED. See FINDING-1. |
| HS-123 | **FAIL-STALE (NEW)** | Asserts partition "7 supported + 23 unsupported == 30" (Case B expects exactly 7 rows, Case C exactly 23). Observed at v0.13.0: **8 supported / 22 unsupported**. IEC 60870-5-104 promoted to supported. See FINDING-3. |
| HS-125 | **FAIL-STALE (NEW)** | Case E asserts `--supported` JSON array length **7**. Observed **8** (adds `IEC 60870-5-104`). Cases B/C (BACnet unsupported, Modbus supported) still hold. See FINDING-3. |
| HS-132 | **FAIL-STALE (NEW)** | Corpus/gap assertions PASS (BACnet UDP/47808 → known-unsupported, count 8; `protocols` exits 0 with 30 entries). But its jq invariants `supported==7` and `unsupported==23` are now `8`/`22`. See FINDING-3. |

## Findings

### FINDING-3 (FAIL-STALE, NEW this run): IEC-104 promotion broke the 7/23 partition invariant in HS-123, HS-125, HS-132
The v0.13.0 IEC-104 feature (STORY-167..174) added `IEC 60870-5-104` (TCP/2404) as the
8th **supported** protocol. The catalog is now **30 total = 8 supported + 22 unsupported**.
Three scenarios hard-code the prior 7/23 split:
- **HS-123** Case B ("exactly 7 rows"), Case C ("exactly 23 rows"), Case G / Success
  Criteria ("7 + 23 == 30"), and its BC-2.18.003/004 traceability rows.
- **HS-125** Case E ("`--supported` JSON: Array Length 7").
- **HS-132** Success-criteria jq assertions `supported == 7`, `unsupported == 23`.

This is an **intentional-change staleness**, not a product bug — IEC-104 detection is a
shipped, deliberate feature and its catalog entry (`TCP`/`[2404]`/`supported: true`) is
correct. Recommended fix: update HS-123/125/132 to `8 supported / 22 unsupported` and add
`IEC 60870-5-104` to the expected supported-set enumeration. A maintainer with `src`/spec
access should confirm 8/22 is the intended post-IEC-104 partition before the scenarios are
rewritten.

### FINDING-1 (FAIL-STALE, carried from 2026-07-11, still open): HS-087 Part C directory expansion
HS-087 Part C still asserts ".pcap only, .pcapng excluded" / uppercase `.PCAP` excluded.
Observed at v0.13.0: a directory containing only `only.pcapng` → the file IS processed
(`notice: .../only.pcapng: 0 packets read`). This is the intended consequence of the
pcapng-reader feature (ADR-009). The 2026-07-11 sweep already flagged this; it has NOT been
fixed in the intervening 10 days. Recommended fix unchanged: update Part C + Edge Conditions
to state directory expansion includes `.pcap` and `.pcapng` and is case-insensitive on the
extension.

### FINDING-2 (scenario maintenance, carried from 2026-07-11): HS-129 Case C/D under-specify the dual-gate
Unchanged from prior sweep. HS-129 Case C/D verification commands run without an active TCP
analyzer flag, so as-written they produce zero TCP entries and would misscore the (correct)
product. HS-130/HS-131 correctly add `--http`. Recommended fix: add `--all`/`--http` to
HS-129 Case C/D commands. Product behavior is correct.

## Coverage Gaps — product surfaces with zero (or thin) holdout scenarios

Derived from `wirerust --help`, each subcommand's help, and `protocols`, cross-referenced
against HS-INDEX and the scenario files in `.factory/holdout-scenarios/`.

1. **IEC-104 analyzer (`--iec104`) — ZERO holdout coverage (NEW, highest-value gap).**
   The v0.13.0 IEC 60870-5-104 analyzer (TCP/2404; APCI parser, frame discrimination,
   ASDU header extraction, control-command detection, N(S)/N(R) sequence-desync detection,
   findings cap, T0881 MITRE catalog entry — STORY-167..174) has **no runnable holdout
   scenario** in `.factory/holdout-scenarios/`. The only `iec104`/`2404` grep hits in the
   holdout tree are the unrelated TCP/102 ISO-on-TCP collision footnotes (HS-126/HS-130).
   HS-INDEX has no IEC-104 section, and `wave-holdout-scenarios/` stops at wave-47 (IEC-104
   is waves 76–84). e2e fixtures exist in the test tree (#416) but are not exposed as
   holdout scenarios. **This is the primary coverage gap this sweep surfaces.**

2. **wave-84 `bin/` tooling — no holdout coverage (expected; noted per task).**
   `bin/compute-input-hash`, `bin/validate-citations` (path:line:anchor symbol assertion,
   STORY-166), and the green-doc-tense / changelog gates are process-internal factory
   tooling, not product CLI surface. Holdout scenarios evaluate the shipped `wirerust`
   binary, so this tooling is out of holdout scope by design — it is exercised by
   `python3 bin/test_compute_input_hash.py` and the `bin-selftest` CI job, not holdouts.
   No action recommended; recorded here to answer the task's explicit coverage question.

3. **Modbus analyzer — still ZERO detection coverage (carried).** `--modbus`,
   `--modbus-write-burst-threshold`, `--modbus-write-sustained-threshold` (T0806/T1692.001)
   appear only as a static catalog entry (HS-123/125). No holdout exercises Modbus DETECTION
   behavior. Unchanged from 2026-07-11.

4. **DNP3 (HS-W35–W39) and ARP (HS-W40–W44) scenario files not present in
   `.factory/holdout-scenarios/`** (carried). Registered in HS-INDEX but concrete files live
   in the feature tree / are seeds-only, outside this sweep's read scope.

5. **ENIP HS-110–122 (13 files) present but NOT-RUNNABLE** (carried). All carry
   `fixture_needed: true`; no ENIP frame fixtures in `holdout-fixtures/`.

6. **`summary <target> --hosts` — thin coverage** (carried). HS-089 covers the summary model
   but no holdout drives the `--hosts` per-host breakdown end-to-end.

## NOT-RUNNABLE Inventory (no silent caps)

| Group | Count | Reason |
|-------|-------|--------|
| Greenfield HS-001–083, 089–093, 098–099, 101–109 | ~98 | Require crafted protocol-specific pcaps / reporter-encoding inputs / pcapng byte-vectors — not craftable faithfully within constraint. |
| ENIP HS-110–122 | 13 | `fixture_needed: true`; no ENIP fixtures on disk. |
| DNP3 HS-W35–W39 | 32 | Concrete scenario files not in `.factory/holdout-scenarios/`. |
| ARP HS-W40–W44 | 28 | Seeds only; concrete scenarios not in allowed scope. |
| IEC-104 | (0 files) | No holdout scenarios authored — see Coverage Gap 1. |

## Deltas vs 2026-07-11 report

- **Binary:** v0.12.0 → v0.13.0 (IEC-104 feature landed).
- **NEW FAIL-STALE:** HS-123, HS-125, HS-132 (were PASS at v0.12.0 with 7/23; now stale at
  8/22 due to the intentional IEC-104 supported-protocol promotion — FINDING-3).
- **STILL OPEN:** HS-087 Part C (FINDING-1) and HS-129 dual-gate note (FINDING-2) were both
  flagged 2026-07-11 and remain unaddressed.
- **NEW coverage gap:** IEC-104 analyzer has zero holdout coverage (Coverage Gap 1) — did
  not exist as a product surface at the prior sweep.

## Recommendation

No product regression found; all four FAIL-STALE verdicts trace to the intentional v0.13.0
IEC-104 feature or the earlier pcapng-reader feature, not to defects. Scenario-maintenance
work warranted: (a) update HS-123/125/132 to the 8/22 partition (FINDING-3); (b) finally fix
HS-087 Part C (FINDING-1, now 10 days open); (c) add the analyzer flag to HS-129 Case C/D
(FINDING-2); (d) author IEC-104 holdout scenarios (Coverage Gap 1). Per repo policy
DF-VALIDATION-001, any of these routed to a GitHub issue must first be validated by the
research-agent.
