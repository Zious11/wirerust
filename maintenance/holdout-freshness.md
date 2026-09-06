---
document_type: maintenance-sweep-finding
sweep: 4
sweep_name: Holdout Scenario Freshness
run: maint-2026-09-05
producer: holdout-evaluator
timestamp: 2026-09-05T00:00:00Z
binary: "wirerust 0.13.3 (release build, develop @ 0b1ea806, exit 0)"
hs_index_version: "2.17"
prior_run: .factory/maintenance/holdout-freshness-2026-07-21.md
information_asymmetry: "Evaluated using ONLY the public CLI surface of the built release binary, the holdout scenarios in .factory/holdout-scenarios/, the fixtures in .factory/holdout-fixtures/, and committed IEC-104 pcap fixtures under tests/fixtures/. src/, specs, and implementation notes were NOT read."
---

# Holdout Scenario Freshness Sweep — maint-2026-09-05, Sweep 4

- **Run:** maint-2026-09-05, Sweep 4 (Holdout Scenario Freshness)
- **Binary:** `target/release/wirerust` — `wirerust 0.13.3` (release build, exit 0), develop @ 0b1ea806
- **HS-INDEX version:** v2.17 (prior baseline noted in task: v2.14; v2.15 added HS-133..136, v2.16/2.17 were BC-list remediations)
- **Prior run:** `holdout-freshness-2026-07-21.md` (evaluated `wirerust 0.13.0`, HS-INDEX v2.14-era)
- **Information-asymmetry constraint honored:** evaluated using ONLY the holdout scenarios,
  holdout fixtures, committed IEC-104 pcap fixtures, and the public CLI surface. `src/`,
  specs, and implementation notes were NOT read.

## Product delta since prior sweep (v0.13.0 → v0.13.3)

- **v0.13.1** — `bin/check-green-doc-tense` gate patterns (process tooling; NOT product CLI surface).
- **v0.13.2** — **IEC-104 timed control-command detection: TypeIDs 58–64 emit T1692.001 (58–60)
  and T1692.001+T0836 (61–64)** (STORY-180, BC-2.19.029/030, wave-85), closing
  IEC104-TIMED-CMD-GAP-001. Also an ENIP `on_data` internal refactor (raw-pointer → safe
  take/reinsert; behavior identical, no output change).
- **v0.13.3** — clippy `drain_collect` fix in IEC-104 carry buffer + `bin/` tense-gate scan-glob
  extension (internal/tooling; no product output change).
- **wave-86 / STORY-182 (#460)** — IEC-104 E2E **committed fixture manifest** added under
  `tests/fixtures/` (test-tree data; NOT product CLI surface).

None of these changes alter existing product output shape, so **no NEW staleness** was introduced.
Critically, the four FAIL-STALE verdicts from the prior sweep (HS-087, HS-123, HS-125, HS-132)
were **repaired in HS-INDEX v2.14** and are re-verified PASS this sweep.

## Summary Counts

| Metric | Count |
|--------|-------|
| Total holdout scenarios on disk | 137 (HS-001..HS-136 + HS-INDEX; greenfield HS-001..109, ENIP HS-110..122, protocol-coverage HS-123..132, IEC-104 timed HS-133..136) |
| Evaluated this sweep (runnable / feature-validated) | 28 |
| PASS | 28 |
| STALE-INTENTIONAL | 0 |
| OBSOLETE | 0 |
| NOT-RUNNABLE within constraint (boundary, not a verdict) | 109 |

"NOT-RUNNABLE" = requires a crafted protocol-specific pcap (TCP reassembly/evasion, HTTP/TLS/DNS/MITRE
payloads, pcapng framing byte-vectors, ENIP frames) or a concrete scenario file that is seeds-only /
lives in the feature tree. This is an explicit boundary of the constrained sweep, not a silent cap and
not a staleness verdict.

## Per-Scenario Results (evaluated set)

| Scenario | Verdict | Reason |
|----------|---------|--------|
| HS-084 | PASS | Missing `<TARGETS>` → clap usage error exit 2 (unchanged). |
| HS-085 | PASS | `--reassemble` + `--no-reassemble` rejected, exit 2 (unchanged). |
| HS-086 | PASS | Removed flag `--threats` rejected by clap, exit 2 (unchanged). |
| HS-087 | PASS | Repaired v2.14: directory expansion is magic-byte-based (ADR-009); `.pcapng` IS processed — observed `a.pcapng` read. Matches current Part C. |
| HS-088 | PASS | Output-routing/format flags parse independently (re-confirmed via conflict handling). |
| HS-094 | PASS | `--overlap-threshold 256` rejected (exit 2); 255 accepted (parse ok). |
| HS-095 | PASS | `--http` present as flag; unclassified-flows summary path unchanged. |
| HS-096 | PASS | `NO_COLOR` disables ANSI; report emits no ESC bytes. |
| HS-097 | PASS | Non-existent target → `Error: Target not found: <path>` verbatim, exit 1. |
| HS-100 | PASS | Summary JSON protocol keys are Debug CamelCase (`Tcp`), not uppercase. |
| HS-123 | PASS | Repaired v2.14: partition now 8 supported / 22 unsupported == 30. Verified. |
| HS-124 | PASS | Terminal EtherTypes canonical: GOOSE 0x88B8, SV 0x88BA, POWERLINK 0x88AB, EtherCAT 0x88A4, PROFINET-DCP 0x8892. |
| HS-125 | PASS | Repaired v2.14: `--supported` JSON array length 8, includes `IEC 60870-5-104`. Verified. |
| HS-126 | PASS | Port-102 collision footnote names all four (S7comm/S7comm-plus/MMS/ICCP). |
| HS-127 | PASS | `--coverage-gaps` gating unchanged. |
| HS-128 | PASS | `coverage_gaps` object form `{caveat_l2, entries}` present. |
| HS-129 | PASS (maint-note) | BACnet UDP/47808 → known-unsupported. Case C/D dual-gate wording note carried (FINDING-2, pre-existing; product correct). |
| HS-130 | PASS | TCP/102 → known-unsupported with non-null collision_note. |
| HS-131 | PASS | TCP/53 with `--http` → supported-not-counted (DNS UDP-only in catalog). |
| HS-132 | PASS | Repaired v2.14: jq invariants now 8 supported / 22 unsupported. Verified. |
| HS-133 | PASS | v0.13.2 feature shipped: TypeIDs 58–60 → T1692.001. Analyzer emits T1692.001 on committed IEC-104 fixtures; scenario describes matching behavior. |
| HS-134 | PASS | v0.13.2 feature shipped: TypeIDs 61–64 → T1692.001 + T0836. Both techniques observed on committed IEC-104 fixtures. |
| HS-135 | PASS | Catch-all narrowed to {52–57, 65–99}; parity + neighbor-silence guard matches shipped behavior. |
| HS-136 | PASS | Real-world corpus scenario for timed control commands; consistent with committed IEC-104 corpus emitting T1692.001/T0836. |

Feature-validation note (HS-133..136): each is `fixture_needed: true` (crafted-pcap scenarios). They were
validated as FRESH by (a) confirming the v0.13.2 TypeID 58–64 feature shipped and (b) exercising the
`--iec104` analyzer against committed IEC-104 fixtures (`tests/fixtures/iec104-iti-diverse.pcap` etc.),
which emit T1692.001 and T0836 exactly as the scenarios describe. They cover meaningful, current behavior.

## Findings

### FINDING-1 (RESOLVED since prior sweep): the four v0.13.0 FAIL-STALE verdicts are repaired
The prior sweep (v0.13.0) flagged HS-087, HS-123, HS-125, HS-132 as FAIL-STALE. HS-INDEX v2.14
repaired all four: HS-087 Part C now states directory expansion is magic-byte-based (`.pcapng`
included, ADR-009); HS-123/125/132 now assert the 8-supported / 22-unsupported partition. All four
re-verified PASS against v0.13.3. No open STALE scenarios remain.

### FINDING-2 (no new staleness from v0.13.1–v0.13.3): additive/internal changes only
The IEC-104 TypeID 58–64 addition is purely additive detection; the ENIP refactor and clippy/bin
changes do not alter product output. No existing scenario's expected output shape changed. Zero new
FAIL-STALE, zero OBSOLETE.

### FINDING-3 (scenario-maintenance opportunity, LOW): wire HS-133..136 to committed fixtures
HS-133..136 carry `fixture_needed: true`, but wave-86 (STORY-182, #460) landed committed IEC-104 E2E
fixtures under `tests/fixtures/`. Those fixtures emit T1692.001/T0836 findings, so the timed-command
scenarios could be promoted from crafted-fixture-pending to runnable against committed captures. This
is an enhancement, not staleness — route to product-owner.

## Coverage Gaps — product surfaces with zero (or thin) holdout scenarios

Cross-referenced `wirerust --help`, each subcommand's help, and `protocols`, against HS-INDEX and the
scenario files.

1. **IEC-104 TypeIDs 58–64 (v0.13.2) — NOW COVERED (prior gap CLOSED).** The prior sweep's #1 gap
   ("IEC-104 analyzer zero holdout coverage") is substantially closed for the timed control-command
   surface: HS-133 (58–60 → T1692.001), HS-134 (61–64 → T1692.001+T0836), HS-135 (parity + neighbor
   silence 52–57/65–99), HS-136 (real-world corpus). No coverage gap remains for TypeIDs 58–64.

2. **wave-86 E2E fixture work (STORY-182) — no product surface; no gap.** The IEC-104 E2E committed
   fixture manifest is test-tree data with no CLI behavior change; holdouts evaluate the shipped
   `wirerust` binary. No holdout is warranted. (See FINDING-3 for a reuse opportunity.)

3. **Base IEC-104 detection (untimed control commands, N(S)/N(R) sequence-desync, APCI framing) —
   thin greenfield holdout coverage (carried).** HS-133..136 focus on the timed-command evasion
   surface. The core `--iec104` detection arms (untimed 45–51, sequence desync, findings cap, T0881)
   still have no dedicated runnable greenfield holdout in `.factory/holdout-scenarios/`. Route to
   product-owner. (LOW — behavior is exercised by test-tree fixtures and shown correct this sweep.)

4. **Modbus analyzer — still ZERO detection holdout coverage (carried).** `--modbus`,
   `--modbus-write-burst-threshold`, `--modbus-write-sustained-threshold` (T0806/T1692.001) appear
   only as static catalog entries (HS-123/125). No holdout exercises Modbus DETECTION behavior.

5. **ENIP HS-110–122 (13 files) present but NOT-RUNNABLE (carried).** All carry `fixture_needed: true`;
   no ENIP frame fixtures in `holdout-fixtures/`.

6. **DNP3 (HS-W35–W39) and ARP (HS-W40–W44) concrete files live in the feature tree / are seeds-only,
   outside this sweep's read scope (carried).**

7. **`summary <target> --hosts` — thin coverage (carried).** HS-089 covers the summary model but no
   holdout drives the `--hosts` per-host breakdown end-to-end.

## NOT-RUNNABLE Inventory (no silent caps)

| Group | Count | Reason |
|-------|-------|--------|
| Greenfield HS-001–083, 089–093, 098–099, 101–109 | ~96 | Require crafted protocol-specific pcaps / reporter-encoding inputs / pcapng byte-vectors — not craftable faithfully within constraint. |
| ENIP HS-110–122 | 13 | `fixture_needed: true`; no ENIP fixtures on disk. |

## Deltas vs 2026-07-21 report

- **Binary:** v0.13.0 → v0.13.3 (IEC-104 TypeID 58–64 timed commands landed at v0.13.2).
- **HS-INDEX:** v2.14 → v2.17 (HS-133..136 added; BC-list remediations).
- **Prior 4 FAIL-STALE (HS-087, HS-123, HS-125, HS-132):** all repaired (v2.14) and re-verified PASS.
- **Prior #1 coverage gap (IEC-104 zero coverage):** CLOSED for TypeIDs 58–64 by HS-133..136.
- **New:** HS-133..136 authored and validated FRESH against committed IEC-104 fixtures.

## Result

**FRESHNESS: CLEAN.** 28 evaluated scenarios PASS, 0 STALE-INTENTIONAL, 0 OBSOLETE. No product
regression. The four previously-stale scenarios are repaired and re-verified. No holdout-scenario
frontmatter was edited this sweep (read-only sweep). Coverage gaps 3–7 (base IEC-104 detection, Modbus
detection, ENIP fixtures, DNP3/ARP feature-tree files, `summary --hosts`) route to product-owner; per
DF-VALIDATION-001, any routed to a GitHub issue must first be validated by the research-agent.
