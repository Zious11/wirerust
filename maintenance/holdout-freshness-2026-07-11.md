# Holdout Scenario Freshness Sweep — maint-2026-07-11, Sweep 4

- **Run:** maint-2026-07-11, Sweep 4 (holdout scenario freshness)
- **Binary:** `target/release/wirerust` — `wirerust 0.12.0` (release build, exit 0)
- **HS index version:** v2.13-era (109 greenfield + 96 feature = 205 all-namespace)
- **Information-asymmetry constraint honored:** evaluated using ONLY the holdout scenarios
  (`.factory/holdout-scenarios/`), holdout fixtures (`.factory/holdout-fixtures/`), and the
  public CLI surface of the built binary. `src/`, specs, and implementation notes were NOT read.
- **develop delta since last full holdout eval:** ONE commit (b5e1e15, docs + bin-tooling,
  zero `src/` changes) — so product behavior is identical to the last full evaluation.
  Sweep prioritized breadth over depth.

## Summary Counts

| Metric | Count |
|--------|-------|
| Run | 20 |
| PASS | 19 |
| FAIL-STALE | 1 (HS-087) |
| FAIL-BUG-SUSPECT | 0 |
| NOT-RUNNABLE (within constraint) | 172 |
| Skipped (silent cap) | 0 |

"NOT-RUNNABLE" = the scenario requires a crafted protocol-specific pcap (TCP reassembly
vectors, HTTP/TLS/DNS/MITRE payloads, pcapng framing byte-vectors, ENIP frames) or a
scenario file that is not present in `.factory/holdout-scenarios/`. From the constrained
position (no `src/`, no protocol fixtures beyond the 8 in `holdout-fixtures/`), these cannot
be executed without risking false verdicts from an incorrectly hand-crafted fixture. This is
an explicit boundary of the sweep, not a silent cap.

## What Was Run (20 scenarios)

All must-pass. Runnable universe under the constraint = protocols catalog (HS-123–126),
coverage-gaps (HS-127–132), and CLI-parse greenfield scenarios that need no protocol-specific
payload (HS-084–088, 094–097, 100). Fixtures used: the 8 in `holdout-fixtures/` plus two
throwaway fixtures crafted from `make_holdout_fixtures.py` helpers (a payload-bearing
TCP/9600 flow and a minimal UDP/53 DNS query) to exercise the TCP-gap dual-gate and the
DNS supported-not-counted invariant.

| HS ID | Verdict | Note |
|-------|---------|------|
| HS-084 | PASS | Missing-target exit 2; `--no-color` honored both placements (0 ESC bytes vs 8 with color); `--mitre` alone does not imply analyzers (exit 0). |
| HS-085 | PASS | `--reassemble`+`--no-reassemble` rejected both orders (exit 2); `--output-format json` valid + has `findings`; csv 9-col header; `xml` rejected; default terminal header present. |
| HS-086 | PASS | `--threats/--beacon/--verbose/-v/--filter` all exit 2; pre-subcommand placement also rejected. |
| HS-087 | **FAIL-STALE** | Part A PASS (`--all` does not imply `--mitre`); Part B PASS (single `--no-reassemble`+`--http` warning on stderr, no HTTP in stdout). Part C stale: directory expansion now INCLUDES `.pcapng` (only.pcapng was processed) and accepts uppercase `.PCAP` — the greenfield v0.1.0 expectation ".pcap only, .pcapng and .PCAP excluded" predates the intentional pcapng-reader feature (ADR-009, HS-101–109). See FINDING-1. |
| HS-088 | PASS | Part A `--json` wins over `--output-format csv` (output begins `{`); Part B file routing writes valid JSON with `findings`, no stdout duplication. Part C (decode-error counting) not run — needs malformed pcap. |
| HS-094 | PASS | `--overlap-threshold 256` rejected (exit 2); 255 and 0 accepted. |
| HS-095 | PASS | `--http` on unclassified TCP flow: `TCP Reassembly` analyzer detail has `unclassified_flows` (value 2); `--dns`-only run has no `unclassified_flows` key anywhere. |
| HS-096 | PASS | `NO_COLOR=""` produces 0 ESC bytes while report content ("WIRERUST TRIAGE REPORT") remains. |
| HS-097 | PASS | Non-existent target: `Error: Target not found: /tmp/...` verbatim path, exit 1. |
| HS-100 | PASS | `summary.protocols` keys are Debug CamelCase `["Tcp","Udp"]`, not uppercase. |
| HS-123 | PASS | Partition 30 = 7 supported + 23 unsupported; ARP+DNS supported, BACnet not; `--supported --unsupported` and spurious positional both exit 2; supported/unsupported name sets disjoint. |
| HS-124 | PASS | Canonical EtherTypes: GOOSE 0x88B8(35000), POWERLINK 0x88AB(34987), EtherCAT 0x88A4(34980), PROFINET RT/DCP 0x8892(34962), SV 0x88BA(35002); ARP EtherType em-dash; >=5 `[L2]` rows. |
| HS-125 | PASS | JSON canonical: BACnet UDP/[47808]/unsupported, Modbus TCP/[502]/supported, GOOSE ethertype=35000 integer/LinkLayer/ICS; categories only ICS+IT; no port_detectable:false entry with non-empty ports. |
| HS-126 | PASS | Port-102 footnote names all four (S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2); absent for `--supported`. |
| HS-127 | PASS | `--all` alone has no `coverage_gaps` key; `--coverage-gaps` adds it; `protocols --coverage-gaps` exits 2; JSON object has `caveat_l2`+`entries`; terminal `CoverageGapsSummary` section present. |
| HS-128 | PASS | L2 caveat is a non-null string on empty pcap with 0 entries; `coverage_gaps` is object form `{caveat_l2,entries}`; caveat still present with non-empty entries. |
| HS-129 | PASS (maint-note) | Case A UDP/47808 = known-unsupported/BACnet/IP OK. Case C/D (TCP/47808 = unknown; combined = 2 entries) are CORRECT only when an analyzer is active (`--all`/`--http`); the scenario's Case C/D verification commands omit the analyzer flag required by the documented dual-gate. Product behavior is correct. See FINDING-2. |
| HS-130 | PASS | With `--http`: TCP/102 = known-unsupported with `collision_note` naming all four; no footnote when no TCP/102 present; dual-gate Case D (no analyzer → no TCP/102 entry) holds. |
| HS-131 | PASS | Crafted UDP/53 DNS query is NOT counted as a gap even without `--dns` (supported-not-counted); TCP/53 with `--http` = state `unknown`. Cases A/B/C verified; Case D follows. |
| HS-132 | PASS | Catalog 30/7/23 with BACnet transport UDP; BACnet corpus (`hs-bacnet-corpus.pcap`) yields UDP/47808 known-unsupported/BACnet/IP count 8; `--all --coverage-gaps` exits 0 (no crash). |

## Findings

### FINDING-1 (FAIL-STALE): HS-087 Part C — directory expansion no longer excludes `.pcapng`/`.PCAP`
HS-087 is a greenfield v0.1.0 scenario. Its Part C asserts "Only `.pcap` files ... are
processed (not `.pcapng`, not files in subdirs)" and its Edge Conditions assert
"`.PCAP` (uppercase): excluded from directory expansion (case-sensitive check)."

Observed against 0.12.0:
- A directory containing only `only.pcapng` → the file IS selected and processed
  (`notice: .../only.pcapng: 0 packets read`).
- A directory containing `X.PCAP` (uppercase) → also selected and processed.
- Subdirectory files are still NOT recursed (non-recursive behavior intact).

`.pcapng` inclusion is the direct, intended consequence of the pcapng-reader feature
(ADR-009; holdouts HS-101–109). The greenfield Part C expectation predates that feature and
is now stale. Recommended fix: update HS-087 Part C and Edge Conditions to state that
directory expansion includes `.pcap` and `.pcapng` and is case-insensitive on the extension.
The uppercase-`.PCAP` acceptance should be confirmed as intended by a maintainer with src
access (it is plausibly deliberate ergonomics, but was not separately verifiable here).

### FINDING-2 (scenario maintenance, not a product bug): HS-129 Case C/D commands under-specify the dual-gate
TCP coverage-gap entries require BOTH `--coverage-gaps` AND at least one active TCP analyzer
(the dual-gate; explicitly documented in HS-130 Case D and Case B/C commands, which include
`--http`/`--all`). UDP gaps (e.g. BACnet UDP/47808) fire without an analyzer because the UDP
decode loop always attempts classification.

HS-129's Case C/D verification commands run `wirerust analyze <tcp>.pcap --coverage-gaps
--json` with NO analyzer flag, so as-written they produce zero TCP entries and would score
the (correct) product as failing. HS-130 and HS-131 Case C correctly add `--http`. Recommended
fix: add `--all` (or `--http`) to HS-129 Case C and Case D verification commands and to its
fixture guidance, for consistency with HS-130/HS-131. Product behavior is correct once the
flag is present (verified: TCP/47808 → unknown with no `name`; combined → 2 entries).

Note the observable UDP/TCP asymmetry itself (UDP gaps report without an analyzer; TCP gaps
require one) is a possible operator-usability point worth a maintainer glance, but it matches
the documented dual-gate and is not classified as a defect here.

## Coverage Gaps — CLI-observable features with zero (or thin) holdout scenarios

Derived from `wirerust --help` and each subcommand's help, cross-referenced against the
registered HS-INDEX and the scenario files in `.factory/holdout-scenarios/`.

1. **Modbus analyzer — ZERO holdout coverage (highest-value gap).**
   `analyze --modbus`, `--modbus-write-burst-threshold`, `--modbus-write-sustained-threshold`
   (port-502 dissection; T0806/T1692.001 write-burst/sustained detections) is a shipped ICS
   analyzer (SS-14; referenced in the ARP gate's "regression on SS-02/SS-05/SS-14/SS-15").
   "modbus" appears in the holdout set ONLY as a static catalog entry (HS-123/HS-125) and
   incidentally in HS-120 (ENIP). No holdout scenario exercises Modbus DETECTION behavior.
   The HS-INDEX `feature_holdout_seeds` lists dnp3/arp/finding-collapse/enip/protocol-coverage
   but NOT modbus.

2. **DNP3 (HS-W35-*..HS-W39-*, 32) and ARP (HS-W40-*..HS-W44-*, 28) scenario files are not
   present in `.factory/holdout-scenarios/`.** They are registered in HS-INDEX but the concrete
   scenario files live in the feature tree (`.factory/feature/wave-holdout-scenarios/`), which
   is outside this sweep's allowed read scope. From the holdout-scenarios directory alone these
   analyzer behaviors have no runnable files; ARP is additionally "seeds only" per the index.

3. **`summary` subcommand + `--hosts` flag — thin coverage.** HS-089 covers the summary MODEL
   (ingest, unique_hosts sort/dedup) and HS-005 references it, but no scenario drives the
   `wirerust summary <target> --hosts` subcommand invocation end-to-end (the `--hosts` per-host
   breakdown expansion is unexercised by any holdout).

4. **ENIP (HS-110–122, 13 files) are present but NOT-RUNNABLE in this sweep** — all carry
   `fixture_needed: true` and there are no ENIP fixtures in `.factory/holdout-fixtures/` (only
   BACnet/TCP/DNS/empty fixtures for HS-127–132). A full ENIP re-eval requires the ENIP frame
   fixtures to be added to the holdout-fixtures set.

## NOT-RUNNABLE Inventory (no silent caps)

| Group | Count | Reason |
|-------|-------|--------|
| Greenfield HS-001–083, 089, 090–093, 098–099, 101–109 | 98 must-pass | Require crafted protocol-specific pcaps (TCP reassembly/evasion vectors, HTTP/TLS/DNS/MITRE payloads, reporter-encoding inputs, pcapng framing byte-vectors). Not craftable faithfully within the constraint without risking false verdicts. |
| Greenfield HS-025 | 1 should-pass | ICS tactic display; needs crafted finding input. |
| ENIP HS-110–122 | 13 must-pass | `fixture_needed: true`; no ENIP fixtures on disk. |
| DNP3 HS-W35–W39 | 32 | Concrete scenario files not in `.factory/holdout-scenarios/` (feature tree). |
| ARP HS-W40–W44 | 28 | Seeds only; concrete scenarios not authored/not in allowed scope. |

## Recommendation

No product regression found. The one product-side observation (case-insensitive/`.pcapng`
directory expansion) is consistent with the intended pcapng feature. Two scenario-maintenance
fixes are warranted (HS-087 Part C staleness; HS-129 Case C/D missing analyzer flag), and one
real test-coverage gap should be filed: author Modbus analyzer holdout scenarios (write-burst
/ sustained-rate detection, port-502 dispatch). Per repo policy DF-VALIDATION-001, any of these
routed to a GitHub issue must first be validated by the research-agent.
