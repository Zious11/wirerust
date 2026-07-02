---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified:
  - "v1.1: F-F2P2-005 Pass-2 remediation — Invariant 6 added encoding ADR-012 Decision 10 (gap-classification orthogonal to enable_dns; can_decode() evaluated regardless of enable_dns flag). 2026-07-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.023: `--coverage-gaps` Flag Is Opt-In; NOT Auto-Enabled Under `analyze --all`; Appends CoverageGapsSummary When Set

## Description

`--coverage-gaps` is an explicit opt-in flag on the `analyze` subcommand. When set,
wirerust populates the `unclassified_port_counts` (TCP) and `udp_unclassified_counts` (UDP)
counters during the analysis run and appends a `CoverageGapsSummary` section to the output.
When NOT set — including when `--all` is used — neither counter is populated and no
`CoverageGapsSummary` section appears. This keeps `analyze --all` behavior unchanged for
existing downstream consumers of wirerust JSON output (ADR-012 Decision 8).

## Related BCs

- BC-2.05.010 — depends on (describes how the counters are populated when --coverage-gaps is set)
- BC-2.05.011 — depends on (exactness/monotonicity of the counters when --coverage-gaps is set)
- BC-2.12.024 — composes with (CoverageGapsSummary mandatory caveat content when --coverage-gaps is set)
- BC-2.12.015 — depends on (BC-2.12.015 covers the analysis summary output; `unclassified_port_counts()` accessor is injected into a new `CoverageGapsSummary` named section per ADR-012 Decision 9, NOT as a Finding)

## Preconditions

1. `wirerust analyze <file>` is invoked, with or without `--coverage-gaps`.
2. The `--all` flag may or may not be set.

## Postconditions

1. **With `--coverage-gaps` set:**
   - `coverage_gaps_enabled == true` on `StreamDispatcher` (and in the decode loop for UDP).
   - `unclassified_port_counts` (TCP) and `udp_unclassified_counts` (UDP) are populated during the analysis run per BC-2.05.010.
   - A `CoverageGapsSummary` section is appended to the analysis output (terminal or JSON) after all `Finding` entries.
   - The `CoverageGapsSummary` section includes the mandatory caveat text (BC-2.12.024).
   - Exit code is 0 on successful analysis.

2. **Without `--coverage-gaps` (including `analyze --all`):**
   - `coverage_gaps_enabled == false` (or the field/feature is absent).
   - Neither `unclassified_port_counts` nor `udp_unclassified_counts` is populated.
   - No `CoverageGapsSummary` section appears in the output.
   - `analyze --all` output is IDENTICAL to prior behavior (no new section, no new keys in JSON).
   - Exit code is unchanged.

3. **`--coverage-gaps` with `--json`:** `CoverageGapsSummary` appears as a JSON object in the analysis output under a `"coverage_gaps"` key (or equivalent named key); schema is a dict of `"<transport>/<port>": { "count": N, "state": "known-unsupported" | "unknown" | "known-supported" }` entries.

## Invariants

1. `--all` does NOT imply `--coverage-gaps`. The two flags are independent. This is the primary behavioral invariant of this contract.
2. LESSON-P1.04 ("no unwired flags"): `--coverage-gaps` is fully wired — when set, it changes both the analysis phase (counter population) and the output phase (CoverageGapsSummary section).
3. The `CoverageGapsSummary` section is a NEW named report section (analogous to `reassembly_summary` in `AnalysisSummary`), NOT a set of `Finding` entries. This preserves the semantic correctness of the finding-severity-MITRE pipeline (ADR-012 Decision 9).
4. When `--coverage-gaps` is set, the existing output (Findings, AnalysisSummary) is UNCHANGED. `CoverageGapsSummary` is purely additive (appended after).
5. The `--coverage-gaps` flag is only valid on the `analyze` subcommand. `wirerust protocols --coverage-gaps` is a clap error.
6. (ADR-012 Decision 10) Gap-classification (the `dns_analyzer.can_decode()` exclusion gate for the UDP unclassified counter) is orthogonal to the `enable_dns` flag. When `--coverage-gaps` is active, `dns_analyzer.can_decode()` is evaluated for every UDP packet to determine whether it counts as unclassified, regardless of whether `--all` or DNS-enabling flags are set. The `enable_dns` flag gates DNS finding-emission only; it does NOT gate the gap-classification check. DNS/53 traffic is never counted in the UDP gap counter whether or not `enable_dns` is true.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `wirerust analyze <file>` (no --coverage-gaps) | No CoverageGapsSummary; output identical to pre-feature behavior |
| EC-002 | `wirerust analyze <file> --all` (no --coverage-gaps) | No CoverageGapsSummary; `--all` includes all analyzers but NOT gap detection |
| EC-003 | `wirerust analyze <file> --coverage-gaps` | CoverageGapsSummary appended; counters populated; exit 0 |
| EC-004 | `wirerust analyze <file> --all --coverage-gaps` | Both --all analyzers AND gap detection active; CoverageGapsSummary present |
| EC-005 | `wirerust analyze <file> --coverage-gaps` on empty pcap | CoverageGapsSummary present but with empty map; mandatory caveat text still present |
| EC-006 | `wirerust analyze <file> --coverage-gaps --json` | JSON output has `"coverage_gaps"` key alongside `"findings"` and `"summary"` |
| EC-007 | `wirerust protocols --coverage-gaps` | clap error (flag not valid on protocols subcommand); non-zero exit |
| EC-008 | Large pcap with many unclassified flows | CoverageGapsSummary bounded by port-space (at most 131,072 unique keys per BC-2.05.010 Invariant 6 — u16 range 0..=65535 = 65,536 values × 2 transports) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `wirerust analyze test.pcap` | No `CoverageGapsSummary` section; exit 0 | no-flag-default |
| `wirerust analyze test.pcap --all` | No `CoverageGapsSummary` section; exit 0 | all-no-gaps |
| `wirerust analyze test.pcap --coverage-gaps` | `CoverageGapsSummary` section present; mandatory caveat text present; exit 0 | happy-path |
| `wirerust analyze test.pcap --all --coverage-gaps` | Both all-analyzers active AND `CoverageGapsSummary`; exit 0 | combined |
| `wirerust analyze test.pcap --coverage-gaps --json` | JSON output contains `"coverage_gaps"` key | json-mode |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `--all` without `--coverage-gaps` produces no CoverageGapsSummary | integration: `test_BC_2_12_023_all_without_coverage_gaps` |
| — | `--coverage-gaps` produces CoverageGapsSummary section | integration: `test_BC_2_12_023_coverage_gaps_flag_produces_section` |
| — | `--coverage-gaps` populates counter (non-zero count after unclassified traffic) | integration: `test_BC_2_12_023_coverage_gaps_counts_unclassified` |
| — | JSON output has `"coverage_gaps"` key when --coverage-gaps set | integration: `test_BC_2_12_023_json_coverage_gaps_key` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md — `--coverage-gaps` is a CLI flag on the `analyze` subcommand that gates a new analysis feature; its opt-in design and independence from `--all` is a CLI Orchestration decision affecting the entry-point semantics of wirerust analyze |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (src/cli.rs — `--coverage-gaps` flag definition; src/main.rs — flag wiring to `coverage_gaps_enabled`); SS-05 (src/dispatcher.rs — `coverage_gaps_enabled` field controls counter population); SS-12 (src/main.rs — CoverageGapsSummary rendering) |
| ADR | ADR-012 Decision 8 (`--coverage-gaps` explicit flag; NOT auto under `--all`); Decision 9 (CoverageGapsSummary as named section, not Finding entries) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/cli.rs` — `Analyze` subcommand gains `--coverage-gaps: bool` flag; NOT in the `--all` expansion group
- `src/main.rs` — `run_analyze()` (or equivalent) passes `coverage_gaps: bool` to `StreamDispatcher::new()`
- `src/dispatcher.rs` — `StreamDispatcher` gains `coverage_gaps_enabled: bool` field; `on_flow_close` None-target arm gates counter increment on `self.coverage_gaps_enabled`
- `src/main.rs` — decode loop UDP path gates `udp_unclassified_counts` increment on `coverage_gaps_enabled`
- `src/main.rs` — after `run_analyze()` returns, if `coverage_gaps` flag is set: renders `CoverageGapsSummary` from merged counters

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

(None assigned yet — integration tests serve as verification)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | file read (pcap), stdout write (CLI output) — effectful |
| **Global state access** | controls whether StreamDispatcher populates unclassified_port_counts |
| **Deterministic** | yes (same pcap + same flags → same output) |
| **Thread safety** | single-threaded CLI |
| **Overall classification** | effectful (CLI + analysis pipeline) |
