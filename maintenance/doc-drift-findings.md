# Documentation Drift Findings — Maintenance Sweep 3

**Run ID:** maint-2026-07-08
**Date:** 2026-07-08
**Producer:** technical-writer
**Branch/HEAD:** develop @ b642c0f (v0.11.5 + 9 unreleased commits)
**Scope:** README.md, CLAUDE.md, docs/adr/0001–0011, CHANGELOG.md Unreleased section,
           TODO/FIXME/HACK scan in src/ and tests/, STORY-154-TESTCOUNT-COMMENT-001 backlog item
**Prior sweep reference:** `.factory/maintenance/doc-drift-findings.md` (Sweep 2, 2026-06-22,
           develop @ dd3b069 / v0.9.3)

---

## Prior Sweep Status

All 10 findings from Sweep 2 (DOC-001 through DOC-010) are resolved at HEAD b642c0f.

| Prior ID | Summary | Status |
|----------|---------|--------|
| DOC-001 | CLAUDE.md STATE.md described as "not yet initialized" | FIXED — now reads "VSDD factory artifacts (STATE.md, stories, specs, research, maintenance logs)" |
| DOC-002 | ADR-009 referenced 40+ times in reader.rs but file did not exist | FIXED — docs/adr/0009-pcapng-reader-design.md now exists |
| DOC-003 | README Architecture table said "pcap files" — omitted pcapng | FIXED — now reads "Parse classic pcap and pcapng files (5 link types; both formats)" |
| DOC-004 | README Features bullet said "pcap formats" — pcapng omitted | FIXED — now reads "in both classic pcap and pcapng captures" |
| DOC-005 | ADR 0002 `detail` field type HashMap vs BTreeMap | FIXED — both occurrences now read BTreeMap |
| DOC-006 | ADR 0002 StreamHandler::on_data missing `timestamp: u32` parameter | FIXED — parameter present in ADR snippet |
| DOC-007 | ADR 0003 stale main.rs line numbers in Grouped-Mode Collapse section | FIXED — replaced with function-name anchors (src/main.rs `run_analyze`, etc.) |
| DOC-008 | ADR 0002 `parse_error_count()` listed as "Required" — is convention only | FIXED — table column now reads "Convention only" |
| DOC-009 | ADR 0001 StreamDispatcher struct snippet missing modbus/dnp3 fields | FIXED — snippet updated with modbus/dnp3/enip/unclassified_flows fields (PR #369) |
| DOC-010 | Cargo.toml rayon declared but unused | FIXED — rayon entry removed from Cargo.toml |

---

## Summary

| Severity | Count | Classification |
|----------|-------|----------------|
| HIGH     | 1     | MANUAL         |
| MED      | 1     | FIXABLE-AUTO   |
| LOW      | 2     | FIXABLE-AUTO   |
| INFO     | 1     | INFO           |
| **Total**| **5** | —              |

All findings are documentation-only. No runtime behavior is affected.

---

## Findings

| ID | File | Severity | Classification | Issue |
|----|------|----------|----------------|-------|
| NEW-001 | `docs/adr/` + `CLAUDE.md` | HIGH | MANUAL | ADR-012 referenced 38× in src/ and tests/ but no `docs/adr/0012-*.md` file exists; CLAUDE.md Project References ADR table also omits it |
| NEW-002 | `README.md` | MED | FIXABLE-AUTO | `--coverage-gaps` analyze flag (shipped v0.11.2 / STORY-154 / PR #355) absent from README Analyze flags section |
| NEW-003 | `docs/adr/0001-content-first-stream-dispatch.md` | LOW | FIXABLE-AUTO | StreamDispatcher struct snippet missing `unclassified_port_counts` and `coverage_gaps_enabled` fields added by STORY-153 |
| NEW-004 | `tests/integration_tests.rs` | LOW | FIXABLE-AUTO | Line 1161 comment "All 20 tests pass" in `mod story_154`; actual count is 22 (open backlog item STORY-154-TESTCOUNT-COMMENT-001, count was 21 at prior check, now 22) |
| NEW-005 | `CHANGELOG.md` | INFO | INFO | `indicatif 0.18.4 → 0.18.5` chore dep bump (PR #375 / commit 6e1b682) is among the 9 unreleased commits since v0.11.5 but has no Unreleased CHANGELOG entry; conventional for patch-only dep bumps |

---

## Detailed Findings

### NEW-001 — docs/adr/: ADR-012 referenced in source but file does not exist (HIGH, MANUAL)

**Files:** `src/protocols.rs`, `src/dispatcher.rs`, `src/main.rs`, `tests/protocols_tests.rs`,
`tests/dispatcher_tests.rs` (38 total occurrences); `docs/adr/` contains only 0001–0007, 0009–0011.
`CLAUDE.md` Project References table lists ADRs 0001–0007, 0009–0011 and does not mention ADR-012.

**Issue:** The protocols catalog system introduced in STORY-151/152/153/154 (v0.11.2) makes
extensive references to ADR-012 throughout its constants, field doc-comments, and inline comments.
Representative occurrences (from grep):

- `src/protocols.rs:13` — `"Exactly two variants — no L2 variant (ADR-012 Decision 7)."`
- `src/protocols.rs:69` — `"classify() is PERMANENT and BY DESIGN (ADR-012 Decision 5)."`
- `src/dispatcher.rs:44` — `"(BC-2.05.010 PC-4, Invariant 1; ADR-012 Decision 6)."`
- `src/dispatcher.rs:98` — `"(STORY-153, BC-2.05.010 PC-1, ADR-012 Decision 6 Clarification)."`
- `src/main.rs:195` — `"(AC-154-002/003/007; ADR-012 Decision 9)."`

From the reference content, ADR-012 covers at minimum the following design decisions:
- Decision 1: Catalog structure for KNOWN_PROTOCOLS
- Decision 2: Suricata-derived vocabulary for protocol classification
- Decision 3: L2-transport-only protocols (EtherCAT, POWERLINK, etc.)
- Decision 4: ARP special case (link-layer, no port)
- Decision 5: `classify()` function behavior as permanent and by design
- Decision 6: `unclassified_port_counts` counter scoping and the `coverage_gaps_enabled` feature flag
- Decision 7: `TransportProto` enum has exactly two variants (no L2 variant)
- Decision 9: `--coverage-gaps` output as purely additive (Findings + AnalysisSummary unchanged)
- Decision 10: `dns_handles` evaluated regardless of `enable_dns`

No `docs/adr/0008-*.md` file has ever existed (the sequence jumps from 0007 to 0009 intentionally —
ADR-008 was skipped). ADR-012 is the current missing file.

**Severity rationale:** Same class as Sweep 2 DOC-002 (ADR-009 missing, rated HIGH). A contributor
reading any ADR-012 inline citation cannot find the decision record. The protocols and coverage-gaps
subsystem is new (v0.11.2) and has no architectural narrative accessible to maintainers.

**Classification:** MANUAL — requires authoring a new ADR document, not a text substitution.

**Suggested action:** Author `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` capturing
the protocols catalog design decisions referenced by the 38 inline citations. Simultaneously
update the CLAUDE.md Project References table to add:
```
| `docs/adr/` | … 0012 protocols catalog and coverage-gaps system |
```

---

### NEW-002 — README.md: `--coverage-gaps` analyze flag undocumented (MED, FIXABLE-AUTO)

**File:** `README.md`, Analyze flags subsection (lines 106–124)

**Issue:** The `--coverage-gaps` flag for the `analyze` subcommand was shipped in v0.11.2
(STORY-154, PR #355) and is present in `src/cli.rs` at line 268:
```rust
/// Enable per-port unclassified traffic gap detection (opt-in)
#[arg(long)]
coverage_gaps: bool,
```
The flag is described in the CHANGELOG v0.11.2 entry ("analyze --coverage-gaps flag — tri-state
CoverageGapsSummary report"). However, the README Analyze flags section does not list it at all.
A user consulting the README for the full set of analyze flags will not learn that
`--coverage-gaps` exists.

The flag generates a `CoverageGapsSummary` section classifying observed traffic into three states:
`covered`, `gap`, and `unclassified`. It is opt-in and orthogonal to `--all`.

**Suggested fix:** Add the following line to the README Analyze flags block, after `--enip-error-burst-threshold N`:
```
--coverage-gaps                        Enable per-port unclassified traffic gap detection; produces a CoverageGapsSummary section classifying observed protocols as covered, gap, or unclassified (default-off; does not affect Findings or AnalysisSummary output)
```

---

### NEW-003 — ADR 0001: StreamDispatcher struct snippet missing STORY-153 fields (LOW, FIXABLE-AUTO)

**File:** `docs/adr/0001-content-first-stream-dispatch.md`, struct code snippet (lines 28–50)

**Issue:** The struct snippet in ADR-0001 was updated in PR #369 (v0.11.5) to add
modbus/dnp3/enip/unclassified_flows. However, STORY-153 (PR #352, v0.11.2) added two additional
fields to `StreamDispatcher` that are not in the snippet:

```rust
// Actual fields in src/dispatcher.rs (lines 97–104) — absent from ADR snippet:
/// Per-(TransportProto, port) counts for TCP flows that close as DispatchTarget::None
/// (STORY-153, BC-2.05.010 PC-1, ADR-012 Decision 6 Clarification).
unclassified_port_counts: HashMap<(TransportProto, u16), u64>,
/// Feature flag: when true, the per-port unclassified_port_counts counter is populated
/// in the on_flow_close None-target arm (STORY-153, BC-2.05.010 PC-1).
coverage_gaps_enabled: bool,
```

The ADR comment at line 101 documents these fields. The ADR struct snippet does not include them,
leaving the snippet one version behind the actual struct shape.

**Suggested fix:** Append the two fields to the struct snippet in ADR-0001 with a comment
noting they were added by STORY-153 / ADR-012 Decision 6:
```rust
    // Added STORY-153 (ADR-012 Decision 6): per-port gap-detection counters
    unclassified_port_counts: HashMap<(TransportProto, u16), u64>,
    coverage_gaps_enabled: bool,
```

---

### NEW-004 — tests/integration_tests.rs: story_154 test count comment stale (LOW, FIXABLE-AUTO)

**File:** `tests/integration_tests.rs`, line 1161

**Stale text:**
```
//   All 20 tests pass. `--coverage-gaps` is fully implemented and wired.
```

**Reality:** `mod story_154` now contains **22** tests (verified by `grep -c "#\[test\]"`).
The comment was written when the initial STORY-154 implementation was committed (PR #355,
author Jared Richards, committer-time 1783149080). Two additional tests were added after that
initial commit:
- `test_BC_2_12_024_json_entry_port102_collision_note`
- `test_BC_2_12_024_json_entry_unknown_state`  *(and possibly others)*

Full current test list in story_154 (22 tests):
1. test_BC_2_12_023_all_without_coverage_gaps
2. test_BC_2_12_023_all_with_coverage_gaps_combination
3. test_BC_2_12_023_protocols_coverage_gaps_error
4. test_BC_2_12_023_no_coverage_gaps_no_section
5. test_BC_2_12_023_coverage_gaps_counts_unclassified
6. test_BC_2_12_023_coverage_gaps_flag_produces_section
7. test_BC_2_12_023_json_coverage_gaps_key
8. test_BC_2_12_024_l2_caveat_always_present
9. test_BC_2_12_024_port102_footnote_on_tcp102_traffic
10. test_BC_2_12_024_port102_footnote_absent_without_tcp102
11. test_BC_2_12_024_port102_note_names_all_four
12. test_BC_2_12_024_bacnet_known_unsupported
13. test_BC_2_12_024_unknown_port_state
14. test_BC_2_12_024_tcp_47808_is_unknown
15. test_BC_2_12_024_tcp_53_is_unknown
16. test_BC_2_12_024_tcp_502_absent_from_gap_report
17. test_BC_2_12_024_json_has_caveat_field
18. test_BC_2_12_024_json_entry_bacnet_schema
19. test_BC_2_12_024_json_entry_port102_collision_note
20. test_BC_2_12_024_json_entry_unknown_state
21. test_BC_2_12_024_empty_entries_message
22. test_BC_2_12_023_coverage_gaps_purely_additive

**Notes on backlog item STORY-154-TESTCOUNT-COMMENT-001:** This item was open in the prior
maintenance period. At that time it was at "21 integration tests." The count has grown to 22.
The backlog description should be updated to reflect the current count.

**Suggested fix:** Update line 1161 in tests/integration_tests.rs to:
```rust
//   All 22 tests pass. `--coverage-gaps` is fully implemented and wired.
```

---

### NEW-005 — CHANGELOG.md: indicatif patch bump absent from Unreleased section (INFO)

**File:** `CHANGELOG.md`, `## [Unreleased]` section

**Issue:** There are 9 unreleased commits since v0.11.5 (tag). Commit `6e1b682` (PR #375,
"chore(deps): bump indicatif from 0.18.4 to 0.18.5") has no corresponding entry in the
Unreleased section. The other 8 commits are accounted for: the code-change commits (PR #374,
#376, #378, #379, #380) are documented by content entries, and the docs/wave commits
(PR #377, #381) and the back-merge commit (PR #373) do not themselves add new changelog entries.

**Classification:** INFO — patch-level transitive dependency bumps are conventionally excluded
from user-facing changelogs (consistent with previous versions of this file; see v0.11.1
"Bumped anyhow 1.0.102 → 1.0.103" as the threshold: that was a security advisory bump and
was included; this is a pure patch bump with no advisory). No action required.

---

## Items Confirmed Accurate (Sweep 3)

The following were checked and found to be correct at HEAD b642c0f:

- All CLAUDE.md referenced paths exist: `bin/compute-input-hash`, `.factory/maintenance/demo-evidence-scrub-gate.md`, `.factory/maintenance/pr-manager-merge-auth-guidance.md`, `.factory/policies.yaml`.
- ADR files 0001–0007, 0009–0011 all exist and match the CLAUDE.md Project References table (10 ADRs listed, 10 files present; ADR-0008 intentionally skipped in sequence).
- `src/analyzer/` contains: arp.rs, dnp3.rs, dns.rs, enip.rs, http.rs, mod.rs, modbus.rs, tls.rs — matches the 7 analyzers documented in README and ADR 0002 (EtherNet/IP added as of v0.11.0).
- `src/lib.rs` module-level docs list DNS / HTTP / TLS / Modbus / DNP3 / ARP / EtherNet/IP — accurate.
- No TODO/FIXME/HACK inline comments found in `src/` or `tests/` (three occurrences of the string "HACKED" are test-data strings, not code comments).
- CHANGELOG Unreleased section has entries for all substantive unreleased PRs: STORY-149 (PR #374), PR #376 scrub, STORY-156 (PR #378), STORY-150 (PR #379), STORY-157 (PR #380). Consistent.
- README `protocols` subcommand documentation present and accurate (added by PR #369 fix for prior sweep finding).
- CLAUDE.md rust-version 1.91 / Rust 2024 edition / single-crate notes accurate.
- CLAUDE.md Input Hash Algorithm section and PG-HASH-HOOK-DIVERGENCE divergence note accurate and current.
- `docs/superpowers/plans/` and `docs/superpowers/specs/` both exist.
- README Architecture diagram/table, Supported Capture Formats section, and EtherNet/IP subsection descriptions are accurate.
- ADR-0001 struct snippet includes modbus, dnp3, enip, unclassified_flows — accurate for those fields (NEW-003 notes the two STORY-153 fields still absent).
- ADR-0002 Existing Analyzers table lists 7 analyzers including EtherNet/IP with Deviations section — accurate.
- ADR-0002 StreamHandler::on_data includes `timestamp: u32` — accurate.
- ADR-0002 `parse_error_count()` described as "Convention only" — accurate.
- rayon dependency is absent from Cargo.toml — DOC-010 remains fixed.
- ADR-0003 Grouped-Mode Collapse section uses function-name anchors, not line numbers — DOC-007 remains fixed.
