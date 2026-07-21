---
document_type: maintenance-findings
sweep: doc-drift
run_id: maint-2026-07-21
sweep_number: 2
producer: technical-writer
timestamp: 2026-07-21T00:00:00Z
base_commit: 1e967bad
---

# Documentation Drift Findings — maint-2026-07-21 Sweep 2

Sweep of README.md, docs/adr/, CLAUDE.md, CHANGELOG.md, and src/tests/bin/ TODO/FIXME/HACK
comments against develop HEAD (1e967bad).

## Summary Table

| ID | Severity | Location | Description | Suggested Fix | Status |
|----|----------|----------|-------------|---------------|--------|
| DOC-001 | HIGH | README.md (Features, Protocol table, Analyze flags, Architecture table) | IEC-104 (`--iec104`) analyzer absent from README entirely — missing Features bullet, Supported Protocol Analyzers row, Analyze flags entry, and JSON-counters section | Add IEC 60870-5-104 row to protocol table, Features bullet, `--iec104` to Analyze flags, and a dedicated IEC-104 section mirroring the DNP3/ARP/ENIP pattern | NEW |
| DOC-002 | MED | docs/adr/0001-content-first-stream-dispatch.md:36–65 | ADR-0001 `StreamDispatcher` struct snippet missing `iec104` field and `Iec104` enum variant; rule-order table shows "8. No match → `None`" but actual code has Rule 8 = port 2404 → `Iec104` | Add `iec104: Option<Iec104Analyzer>` field, `Iec104` variant in `DispatchTarget`, and update rule list to 9 entries (Rule 8 = port 2404 → Iec104) | NEW |
| DOC-003 | MED | docs/adr/0002-modular-protocol-analyzers.md:149–188 | ADR-0002 Existing Analyzers table and Deviations section do not include IEC-104 (`src/analyzer/iec104.rs`, v0.13.0) | Add IEC-104 row to the analyzer table; add an IEC-104 deviation entry describing its custom dispatch interface (mirrors DNP3 deviation pattern) | NEW |
| DOC-004 | MED | CLAUDE.md (Project References table, docs/adr/ row) | ADR-013 (IEC-104 stream dispatch and parser design) not listed in CLAUDE.md Project References table; table only enumerates ADRs 0001–0012 | Add "0013 IEC-104 stream dispatch and parser design" to the `docs/adr/` row in the Project References table | NEW |
| DOC-005 | LOW | src/cli.rs:259 | ENIP write-burst threshold doc-comment uses "1-second window"; adjacent Modbus arg at cli.rs:185 uses "1s window" — same format inconsistency as UNIT-FMT-5-20S-001 but for ENIP | Change "within any 1-second window" to "within any 1s window" at cli.rs:259 | NEW |

## Prior Findings Resolution (from maint-2026-07-11)

All 8 open findings from the 2026-07-11 sweep are **RESOLVED** as of HEAD 1e967bad:

| Prior ID | Resolved By | Evidence |
|----------|-------------|----------|
| PG-W-README-JSON-SCHEMA | maint-2026-07-11 sweep cleanup | README:233 and README:264 now correctly describe `analyzers[i].detail` — no `arp_summary` sub-key claimed |
| DOC-NEW-001 | maint-2026-07-11 sweep cleanup | docs/adr/0002:180 now reads "tech-debt item PC-020" (was PC-023) |
| ROUTE-B-DEFERRED NEW-003 | maint-2026-07-11 sweep cleanup | docs/adr/0001:40–43 now includes both `unclassified_port_counts` and `coverage_gaps_enabled` fields |
| CHANGELOG-D3-T0830-DRIFT-001 | maint-2026-07-11 sweep cleanup | CHANGELOG.md:1492–1494 now has strikethrough with correction: `mitre_techniques: []` per DF-VALIDATION-001 |
| ARP-RATE-INTDIV-DOC-001 | maint-2026-07-11 sweep cleanup | src/analyzer/arp.rs:1008 now includes "integer division; truncates fractional rates" note |
| DNP3-TUNING-BIDIR-001 | maint-2026-07-11 sweep cleanup | README:414–415 now includes unidirectional mirror-tap guidance |
| README-OPTIONS-L117-NEUTRAL-001 | maint-2026-07-11 sweep cleanup | README:117 now includes "at or above" and calibration note |
| UNIT-FMT-5-20S-001 | maint-2026-07-11 sweep cleanup | src/cli.rs:185 now uses "1s window" (was "1-second window"); a new instance appeared at cli.rs:259 (ENIP) — tracked as DOC-005 above |

---

## Finding Detail

### DOC-001 — HIGH — NEW

**Location:** README.md (multiple sections)

**Description:** The IEC 60870-5-104 (IEC-104) analyzer shipped as the primary feature of v0.13.0
(2026-07-18) and is the eighth protocol analyzer in wirerust. It is entirely absent from README.md.
The CLI flag `--iec104` exists at `src/cli.rs:256`, the analyzer module is at `src/analyzer/iec104.rs`,
and it was wired into the dispatcher as Rule 8 (port 2404) by STORY-173 (PR #408). The README has
no bullet in the Features section, no row in the Supported Protocol Analyzers table, no entry in the
Analyze flags section, and no dedicated analyzer-level section (compare: DNP3, ARP, and EtherNet/IP
each have a dedicated section at README.md:206–304).

**Evidence:**
- `grep -n "iec104\|IEC-104\|2404\|T0881" README.md` returns empty
- `src/cli.rs:252–256`: `--iec104` flag present (IEC 60870-5-104, port 2404)
- `src/analyzer/mod.rs:19`: `pub mod iec104;`
- `src/dispatcher.rs:29`: Rule 8 = port 2404 → `Iec104`
- README.md Supported Protocol Analyzers table (lines 196–204): ends at ARP with no IEC-104 row
- README.md Analyze flags (lines 104–125): ends at `--coverage-gaps` with no `--iec104` entry

**Suggested fix:** Add to README.md:
1. Features bullet: "**IEC 60870-5-104 forensics** — ICS/OT threat detection on port 2404; parses APCI header and ASDU structure; detects MITRE ATT&CK ICS techniques T1692.001, T0836, T0827, T0881, T0814; U-frame session state machine with STARTDT/STOPDT/TESTFR tracking; N(S) sequence desync detection; `dropped_findings` and `flows_analyzed` JSON counters; enabled via `--iec104`"
2. Supported Protocol Analyzers table row: `| IEC 60870-5-104 TCP | 2404 | --iec104 | off | T1692.001, T0836, T0827, T0881, T0814 |`
3. Analyze flags entry: `--iec104   Analyze IEC 60870-5-104 TCP traffic (port 2404, default-off; included in --all)`
4. A dedicated IEC-104 section (analogous to the existing DNP3/ARP/ENIP sections) covering: detection table, CLI flags, JSON output counters (`dropped_findings`, `flows_analyzed`)

---

### DOC-002 — MED — NEW

**Location:** docs/adr/0001-content-first-stream-dispatch.md:36–65

**Description:** The `StreamDispatcher` struct snippet and rule-order table in ADR-0001 are stale
following the IEC-104 dispatcher integration (STORY-173, PR #408, v0.13.0). Three specific gaps:

1. The struct snippet (lines 36–43) is missing `iec104: Option<Iec104Analyzer>` — the field added by
   STORY-173 alongside the existing `modbus`, `dnp3`, and `enip` fields.
2. The `DispatchTarget` enum (lines 46–53) is missing the `Iec104` variant.
3. The classification rule table (lines 58–65) shows "8. No match → `None`" as the terminal entry.
   The actual dispatcher (`src/dispatcher.rs:26–29`) has Rule 8 = port 2404 → `Iec104`, with no
   explicit "no match" rule (implicit default when none of the 8 rules match).

**Evidence:**
- docs/adr/0001:36–38: `modbus`, `dnp3`, `enip` fields present; no `iec104` field
- docs/adr/0001:58–65: rule 8 = "No match → `None`"
- src/dispatcher.rs:26–29 (module doc): "8. Port 2404 → `DispatchTarget::Iec104` ← Rule 8 (STORY-173, ADR-013)"
- src/dispatcher.rs:63–64: `/// Port-2404 IEC 60870-5-104 TCP flows (Rule 8, BC-2.05.012). Added in STORY-173.` / `Iec104,`
- src/dispatcher.rs:102: `iec104: Option<Iec104Analyzer>,`

**Suggested fix:**
1. Add `iec104: Option<Iec104Analyzer>,  // Rule 8: port-2404 flows (ADR-013)` to the struct snippet after the `enip` line
2. Add `Iec104,` variant to the `DispatchTarget` enum
3. Update the rule table: renumber "8. No match → None" to "9. No match → `None`" and insert "8. Port 2404 → `Iec104` (ADR-013)"

---

### DOC-003 — MED — NEW

**Location:** docs/adr/0002-modular-protocol-analyzers.md:149–188

**Description:** The Existing Analyzers table (lines 149–157) and the Deviations section (lines
159–188) do not include the IEC-104 analyzer added in v0.13.0 (STORY-167–STORY-173, PR #408).

The table currently ends at EtherNet/IP (v0.11.0). `src/analyzer/iec104.rs` exists and uses a
custom dispatch interface (inherent methods `on_data(flow_key, data, ts, direction)` and
`on_flow_close(flow_key)` called directly by the dispatcher, not via `StreamHandler`
trait) — the same deviation pattern as DNP3.

The Deviations section (lines 159–188) lists DNP3, ARP, and EtherNet/IP deviations but has no
IEC-104 entry, leaving the deviation undocumented in the ADR that governs the analyzer pattern.

**Evidence:**
- docs/adr/0002:149–157: table ends at `EtherNet/IP | custom dispatch interface ... | src/analyzer/enip.rs | v0.11.0`
- `grep -n "Iec104\|iec104" docs/adr/0002-modular-protocol-analyzers.md` returns empty
- src/analyzer/iec104.rs: `Iec104Analyzer` with `on_data(flow_key: &FlowKey, data: &[u8], ts: DateTime<Utc>, direction: Direction)` and `on_flow_close(flow_key: &FlowKey)`
- src/dispatcher.rs: calls `iec104.on_data(flow_key, data, ts, direction)` directly (not via trait)

**Suggested fix:**
1. Add row to Existing Analyzers table: `| IEC 60870-5-104 | custom dispatch interface (see ADR-013 and §Deviations below) | src/analyzer/iec104.rs | v0.13.0 |`
2. Add IEC-104 entry to Deviations section describing: inherent `on_data(flow_key, data, ts, direction)` and `on_flow_close(flow_key)` methods called directly by dispatcher, not via `StreamHandler` trait; rationale in ADR-013

---

### DOC-004 — MED — NEW

**Location:** CLAUDE.md (Project References table, `docs/adr/` row)

**Description:** The `docs/adr/` row in the CLAUDE.md Project References table lists ADRs
0001–0012 but omits ADR-013. ADR-013 was added as part of the IEC-104 feature tree
(accepted 2026-07-13) and exists at `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`.
Omitting it from the table means AI agents operating from CLAUDE.md context will not know ADR-013
exists, potentially missing design rationale when working on IEC-104 code.

**Evidence:**
- CLAUDE.md `docs/adr/` row: "...0012 protocols catalog and coverage-gaps system" — no 0013 entry
- `ls docs/adr/0013-iec104-stream-dispatch-and-parser-design.md` → file exists

**Suggested fix:** Append "0013 IEC-104 stream dispatch and parser design" to the parenthetical list
in the `docs/adr/` row of the Project References table.

---

### DOC-005 — LOW — NEW

**Location:** src/cli.rs:259

**Description:** The ENIP write-burst threshold CLI arg doc-comment at line 259 uses
"within any 1-second window" (hyphenated, spelled out) while the adjacent Modbus write-burst
threshold arg at line 185 uses "within any 1s window" (abbreviated). This is the same
UNIT-FMT-5-20S-001 pattern that was fixed for cli.rs:185 in the 2026-07-11 cleanup but was
not applied to the ENIP arg (which predates that fix or was not covered by it).

**Evidence:**
- src/cli.rs:259: `/// requests are observed within any 1-second window. Default: 50.`
- src/cli.rs:185: `/// write-class FCs are observed within any 1s window.` (already fixed)

**Suggested fix:** Change `"1-second window"` to `"1s window"` at cli.rs:259.

---

## Checks Completed Without New Findings

### 1. README — Installation, Usage, Existing Analyzer Docs

- All non-IEC-104 content in the README is accurate as of HEAD: install/build commands, subcommand list, global options block (all flags/defaults match `cargo run -- --help`), analyze flags for DNS/HTTP/TLS/Modbus/DNP3/ARP/ENIP, protocol coverage table for those seven analyzers, capture format docs, architecture diagram, JSON schema descriptions for ENIP (`enip_summary`), DNP3 counters, ARP counters.
- The `--arp-storm-rate` description at README:117 is accurate ("at or above", calibration note).
- The DNP3 mirror-tap guidance at README:414–415 is present and accurate.
- The ARP JSON counters description at README:233 and README:264 correctly says `analyzers[i].detail`.

### 2. docs/adr/ — Non-IEC-104 Cross-References

- All 13 ADR files exist (`docs/adr/0001–0013`; 0008 is the withdrawn placeholder; 0013 added v0.13.0).
- ADR-0001 fields `unclassified_port_counts` and `coverage_gaps_enabled` present at lines 40–43 (DOC-NEW-003 RESOLVED).
- ADR-0002 tech-debt item reference at line 180 reads "PC-020" (DOC-NEW-001 RESOLVED).
- ADR-0001 `DispatchTarget` enum and struct snippet are accurate for Rules 1–7 and the two coverage-gap fields; only the IEC-104 Rule 8 addition is missing (DOC-002 above).

### 3. CLAUDE.md — File/Path References

- All `.factory/maintenance/` files listed in CLAUDE.md exist on the current branch: `demo-evidence-scrub-gate.md`, `pr-manager-merge-auth-guidance.md`, `docs-writer-dispatch-guidance.md`, `breaking-change-delivery-protocol.md`, `pr-description-row-verify-mandate.md`, `delivery-doc-currency-protocol.md`.
- `docs/superpowers/plans/` and `docs/superpowers/specs/` exist.
- `bin/` tools listed in CLAUDE.md all exist: `compute-input-hash`, `test_compute_input_hash.py`, `check-green-doc-tense`, `validate-citations`, `changelog-gate-check`, `lint-cycle-artifact`.
- CI job names referenced in CLAUDE.md (`changelog-gate`, `action-pin-gate`) match `.github/workflows/ci.yml` (lines 505 and 339 respectively).
- Mutation-testing guidance in CLAUDE.md (PG-MUTANTS-JOBS-001 section) references `.cargo/mutants.toml` — that file now exists (added by PR #421, STORY-147).
- ADR-013 is omitted from the docs/adr/ row (DOC-004 above).

### 4. CHANGELOG — [Unreleased] vs Post-v0.13.0 Merges

Commits since v0.13.0 (2026-07-18) that touch `src/`, `Cargo.toml`, or `bin/`:
- PR #429 (bin/check-green-doc-tense, bin/test_gitignore_mutants_glob.py): covered by [Unreleased] "Fixed" wave-84 CR-002/CR-005/CR-006/SEC-003 entry.
- PR #427 (bin/check-green-doc-tense, bin/test_gitignore_mutants_glob.py, .gitignore, STORY-176): covered by [Unreleased] "Added" STORY-176 entry.
- PR #426 (bin/validate-citations, bin/test_validate_citations.py, STORY-166): covered by [Unreleased] "Added" STORY-166 entry.
- PR #421 (.cargo/mutants.toml, CLAUDE.md, tests/, docs/): does NOT touch `src/`, `Cargo.toml`, or `bin/` — changelog-gate not triggered; no CHANGELOG obligation.
- PR #420 (Cargo.lock only, no Cargo.toml change): per PR #420 description "No `Cargo.toml` changes → changelog-gate CI job is not triggered". CHANGELOG gate not triggered; no CHANGELOG obligation.
- PR #419 (docs/demo-evidence only): does not touch src/Cargo.toml/bin/ — no CHANGELOG obligation.

All three PRs (#426, #427, #429) that do touch `bin/` have corresponding [Unreleased] entries. CHANGELOG obligation is satisfied.

### 5. TODO/FIXME/HACK Comments in src/, tests/, bin/

- `grep -rn "TODO|FIXME|HACK" src/` → empty (no such comments in src/).
- `grep -rn "TODO|FIXME|HACK" bin/` → empty (no such comments in bin/).
- `grep -rn "HACK" tests/` → two matches (`reporter_terminal_tests.rs:606`, `reporter_tests.rs:798`, `:1847`) — all are ANSI-escape-code test string literals (`"31mHACKED"` simulating an HTTP path with an embedded CSI sequence), not code comments. Not a finding.
