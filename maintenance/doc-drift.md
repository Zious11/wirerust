# Doc Drift Scan — maint-2026-07-09

**Run ID:** maint-2026-07-09
**Date:** 2026-07-09
**Branch/HEAD:** develop @ 716054a (v0.11.5 + wave-72 unreleased)
**Scope:** README.md, CLAUDE.md, docs/adr/0001–0012, src/lib.rs crate doc, CHANGELOG.md
**Delta emphasis:** PRs #388–#391 (wave-72: STORY-158 CHANGELOG gate, STORY-159 ADR-012,
STORY-160 JSON enum casing + schema_version, STORY-161 proof_file_hash docs) + #386
(indicatif bump)

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 0     |
| MEDIUM   | 1     |
| LOW      | 1     |
| INFO     | 2     |
| **Total**| **4** |

All 8 findings from the prior sweep (maint-2026-07-06, N-1 through N-8) and NEW-001 are
fully resolved. See the disposition table below.

---

## Prior Findings Disposition

| ID  | Severity | Description | Status |
|-----|----------|-------------|--------|
| N-1 | HIGH | README `protocols` subcommand entirely absent | **FIXED** — PR #369 added full protocols section (README lines 127–149) |
| N-2 | MED | `--json [FILE]` and `--csv [FILE]` flags undocumented | **FIXED** — PR #369 added to Options block (README lines 88–89) with usage example (line 57) |
| N-3 | MED | lib.rs crate doc step 6 missing EtherNet/IP | **FIXED** — `src/lib.rs:22` now reads "DNS / HTTP / TLS / Modbus / DNP3 / ARP / EtherNet/IP" |
| N-4 | MED | ADR 0002 Existing Analyzers table missing EtherNet/IP row | **FIXED** — ADR 0002 now has EtherNet/IP row and EnipAnalyzer deviation note in §Deviations |
| N-5 | MED | New observability counters (ARP/Modbus/HTTP/TLS) undocumented | **FIXED** — PR #369 added counter documentation to all four analyzer sections in README |
| N-6 | LOW | ADR 0001 missing EtherNet/IP Rule 7 in struct/enum/rules snippets | **FIXED** — ADR 0001 now has `enip: Option<EnipAnalyzer>`, `Enip` variant, Rule 7 (Port 44818 → Enip), Rule 8 (No match → None) |
| N-7 | LOW | 3 residual `RED:` comments in `tests/enip_analyzer_tests.rs` | **FIXED** — `grep -n "RED:" tests/enip_analyzer_tests.rs` returns no output; all removed |
| N-8 | LOW | CHANGELOG v0.4.0 T0855 entry without remap annotation (persisting from prior sweep) | **FIXED** — CHANGELOG line 907 now includes `(→ remapped to T1692.001 in v0.5.0)` |
| NEW-001 | HIGH | ADR-012 referenced in 38 code/test lines but no public document existed | **RESOLVED** — PR #388 (STORY-159) created `docs/adr/0012-protocols-catalog-and-coverage-gaps.md`; CLAUDE.md Project References table updated |

---

## MEDIUM Severity

### DD-001 — README: `--coverage-gaps` analyze flag undocumented

**File:** `README.md`, "Analyze flags" section (lines 104–124)

**What's stale:** The `--coverage-gaps` flag was introduced in v0.11.2 (STORY-154, PR #355)
and enables the tri-state `CoverageGapsSummary` report, which classifies each observed
protocol port as `covered`, `gap` (known-unsupported), or `unclassified` (no catalog
entry). It is one of two significant v0.11.2 additions — the other being the `protocols`
subcommand, which was fixed as N-1.

`--coverage-gaps` does not appear anywhere in the README "Analyze flags" block or in any
explanatory prose. The `protocols` subcommand has its own section (lines 127–149); the
`CoverageGapsSummary` report triggered by `--coverage-gaps` has none.

Verified absent: `grep "coverage.gaps" README.md` returns no results.

Actual `wirerust analyze --help` shows:
```
--coverage-gaps
    Enable per-port unclassified traffic gap detection (opt-in)
```

**Design note (ADR-012 Decision 8):** `--coverage-gaps` is deliberately excluded from
`--all` (opt-in) to avoid silent behavioral drift for downstream JSON consumers. The README
should document this design choice alongside the flag description.

**Suggested fix:** Add an entry to the "Analyze flags" block and a prose note in or near
the "List protocol coverage" section describing the `CoverageGapsSummary` output, that it
is opt-in (not enabled by `--all`), and a brief usage example:
```bash
wirerust analyze capture.pcap --all --coverage-gaps
```

---

## LOW Severity

### DD-002 — README: DNP3 observability counters missing from feature documentation

**File:** `README.md`, DNP3 feature bullet (line 14) and DNP3 TCP Analyzer section
(lines 192–215)

**What's stale:** Three observability counters for the DNP3 analyzer were added in v0.11.5
(PR #370, BC-2.15.016/020/022). The README was updated in PR #369 (also v0.11.5) to
document the equivalent counters for ARP (`bindings_evicted`, `storm_counters_evicted`),
Modbus (`dropped_transactions`), HTTP (`dropped_map_entries`), and TLS (`dropped_map_entries`).
The DNP3 counters landed in a subsequent PR (#370) within the same release and were not
included.

Three counters present in DNP3 `summarize()` JSON output but absent from README:

| Counter | Exposed in | Purpose |
|---------|-----------|---------|
| `dropped_findings` | DNP3 summary | Fires at each of 11 `MAX_FINDINGS` cap-check sites (cap: 10 000) |
| `master_addrs_dropped` | DNP3 summary | New master addresses silently ignored after `MAX_MASTER_ADDRS = 64` cap is full |
| `pending_requests_evicted` | DNP3 summary | LRU evictions from the pending-requests table |

The ENIP section at README line 271 does document `dropped_findings` for that analyzer,
so the pattern exists; DNP3 is the only stream analyzer lacking its counter documentation.

**Suggested fix:** Add a "JSON output counters" note to the DNP3 TCP Analyzer section
(mirroring the ARP section's `bindings_evicted` / `storm_counters_evicted` format at
README lines 238–243) and a brief mention in the feature bullet at line 14.

---

## INFO

### DD-003 — README: `schema_version` envelope field and lowercase JSON enum casing not documented

**File:** `README.md` (no specific line; affects JSON output description)

**What's stale:** CHANGELOG `[Unreleased]` (targeting v0.12.0) documents two BREAKING
changes to JSON output:

1. `verdict`, `confidence`, and `category` values changed from PascalCase (`"Likely"`,
   `"High"`, `"LateralMovement"`) to lowercase/snake_case (`"likely"`, `"high"`,
   `"lateral_movement"`).
2. Every JSON report now includes `"schema_version": "2"` as a top-level envelope field.
   Absence of this field signals the pre-v0.12.0 format.

The README describes JSON export capabilities but has no mention of these schema-level
changes or the `schema_version` field. The changes are already implemented in the
`develop` binary (Cargo.toml still at 0.11.5, but code is live).

**Disposition:** Low urgency — the CHANGELOG `[Unreleased]` section is the authoritative
disclosure for pre-release consumers. The README should be updated when v0.12.0 is cut.
This finding is filed now so it is not missed at release time.

**Suggested fix at v0.12.0 cut:** Add a "JSON schema" note to the README under the
"Multiple outputs" feature or a new JSON output subsection. Document `schema_version`,
the lowercase casing convention, and the `Direction` heterogeneity carve-out
(`ClientToServer`/`ServerToClient` retain PascalCase per BC-2.11.036 scope).

---

### DD-004 — ADR numbering gap (ADR-008 absent from docs/adr/)

**File:** `docs/adr/` directory

**What's stale:** The ADR sequence reads 0001–0007, 0009–0012. ADR-0008 does not exist.
No CLAUDE.md Project References entry refers to ADR-008. This gap has been present
across all three prior sweeps without annotation.

**Likely cause:** ADR-008 was reserved or drafted but withdrawn before publication. The
gap has no operational impact.

**Suggested fix (optional):** Add a placeholder file `docs/adr/0008-withdrawn.md` noting
"This ADR number was reserved but not used. The original topic was subsumed by [ADR-XXX]
or withdrawn." This prevents future contributors from assuming the gap is an oversight.

---

## No Issues Found

- **CLAUDE.md build/test/lint commands** — verified correct (`cargo check`, `cargo build`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`).
- **CLAUDE.md CI description** — `changelog-gate` job, SHA-pin policy, and `action-pin-gate`
  existence guard all consistent with `.github/workflows/ci.yml` as hardened by PR #391.
- **CLAUDE.md input-hash section** — algorithm description, edge cases (empty inputs,
  inline comment stripping, Python 3.10+ floor), and PG-HASH-HOOK-DIVERGENCE note all
  verified accurate against `bin/compute-input-hash`.
- **CLAUDE.md "Two Hash Disciplines" section** — added by PR #390 (STORY-161); correctly
  distinguishes `input-hash` (MD5-first-7, advisory) from `proof_file_hash` (SHA-256
  mini-Merkle, integrity anchor).
- **CLAUDE.md Project References ADR list** — includes ADR-012 (added by PR #388).
- **ADR-012** — `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` is thorough and
  consistent with `src/cli.rs` `Commands::Protocols` definition and `ProtocolFilter` enum.
- **ADR-0001** — struct snippet, DispatchTarget enum, and 8-rule classification table all
  accurate including ENIP Rule 7 and `unclassified_port_counts` field (added for
  coverage-gaps feature, Decision 6 Clarification present in ADR-012).
- **ADR-0002** — Existing Analyzers table complete for all 7 analyzers; Deviations section
  covers DNP3, ARP, and EnipAnalyzer inherent-method dispatch.
- **README analyze flags** — all flags verified against `wirerust analyze --help` output
  except the DD-001 `--coverage-gaps` gap. Metavar names (`<FMT>` vs `<OUTPUT_FORMAT>`,
  `<FILE>` vs `<JSON>`/`<CSV>`) differ cosmetically from the actual help strings; this is
  intentional human-readable shorthand, not a behavioral discrepancy.
- **README JSON output examples** — no explicit JSON output block shows PascalCase
  verdict/confidence/category values; no stale example drift from the v0.12.0 casing
  change.
- **CHANGELOG `[Unreleased]`** — covers all five PRs merged since prior sweep: #388
  (STORY-159 ADR-012), #389 (STORY-160 JSON enum casing), #390 (STORY-161 proof_file_hash),
  #391 (action-pin-gate hardening), and #386 (indicatif bump). All user-facing changes
  correctly attributed and the BREAKING CHANGE notice for v0.12.0 is present.
- **`cargo run -- --help` vs README "Options" block** — all 14 global flags documented;
  no flag present in the binary but absent from README, and no flag documented in README
  that does not exist in the binary.
- **lib.rs crate doc** — step 6 analyzer list matches all 7 protocol analyzers.
- **README feature bullets / protocol coverage table** — accurate for all 7 analyzers
  (ports, flags, MITRE technique IDs).

---

## Remediation Priority

| Priority | Finding | Severity | Effort |
|----------|---------|----------|--------|
| 1 | DD-001: `--coverage-gaps` absent from README | MED | Low (1 flag entry + 1 paragraph + usage example) |
| 2 | DD-002: DNP3 observability counters missing | LOW | Trivial (3-row table in DNP3 section, 1 sentence in feature bullet) |
| 3 | DD-003: `schema_version` / JSON casing undocumented | INFO | Low — defer to v0.12.0 release cut |
| 4 | DD-004: ADR-008 numbering gap | INFO | Trivial — optional placeholder file |
