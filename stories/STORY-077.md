---
document_type: story
story_id: "STORY-077"
epic_id: "E-8"
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.006.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.007.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.008.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.009.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.011.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.012.md
  - .factory/specs/prd.md
input-hash: "2f85054"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-076]
blocks: [STORY-078]
behavioral_contracts:
  - BC-2.11.006
  - BC-2.11.007
  - BC-2.11.008
  - BC-2.11.009
  - BC-2.11.010
  - BC-2.11.011
  - BC-2.11.012
verification_properties: [VP-012]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 21
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-SEC-001
  - NFR-SEC-002
  - NFR-SEC-003
  - NFR-OBS-009
implementation_strategy: brownfield-formalization
---

# STORY-077: TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety

## Narrative
- **As a** forensic analyst viewing wirerust output in a terminal
- **I want** all attacker-controlled bytes (C0, DEL, C1 range, backslash) in finding summaries, evidence, and analyzer detail values to be safely escaped before display — while printable ASCII and legitimate Unicode (Cyrillic, emoji) remain readable — and the skipped-packets warning to appear only when errors occurred
- **So that** no attacker-controlled pcap payload can inject terminal control sequences into my terminal emulator, and the output is clean when no decode errors occurred

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.006 | TerminalReporter Shows Skipped: N Packets Only When N > 0 |
| BC-2.11.007 | TerminalReporter Escapes C0+DEL+C1+Backslash in Finding Summary and Evidence |
| BC-2.11.008 | TerminalReporter Escape Preserves Printable ASCII and UTF-8 |
| BC-2.11.009 | TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved |
| BC-2.11.010 | TerminalReporter Escapes Both Summary AND Each Evidence Line |
| BC-2.11.011 | TerminalReporter Escapes Analyzer-Summary Detail Values |
| BC-2.11.012 | TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.006 postcondition 2)
When `Summary.skipped_packets = 0`, the `TerminalReporter::render` output does NOT contain the string "Skipped:".
- **Test:** `test_BC_2_11_006_skipped_packets_zero_no_line()`

### AC-002 (traces to BC-2.11.006 postcondition 1)
When `Summary.skipped_packets = 5`, the output contains `"Skipped: 5 packets (decode errors)"`.
- **Test:** `test_BC_2_11_006_skipped_packets_nonzero_line_present()`

### AC-003 (traces to BC-2.11.007 postcondition 1)
`escape_for_terminal` converts ESC (0x1B) to the escape sequence representation (via `char::escape_default`); raw 0x1B does NOT appear in the output.
- **Test:** `test_BC_2_11_007_esc_byte_escaped()`

### AC-004 (traces to BC-2.11.007 postcondition 2)
`escape_for_terminal` converts DEL (0x7F) to an escape sequence; raw 0x7F does NOT appear in the output.
- **Test:** `test_BC_2_11_007_del_escaped()`

### AC-005 (traces to BC-2.11.007 postcondition 4)
`escape_for_terminal` converts backslash (0x5C) to `\\` (double-backslash); the raw single backslash does NOT pass through.
- **Test:** `test_BC_2_11_007_backslash_escaped()`

### AC-006 (traces to BC-2.11.008 postcondition 1)
Printable ASCII characters (0x20-0x7E, excluding backslash 0x5C) pass through `escape_for_terminal` unchanged.
- **Test:** `test_BC_2_11_008_printable_ascii_preserved()`

### AC-007 (traces to BC-2.11.008 postcondition 2)
Cyrillic, emoji, and other non-ASCII Unicode codepoints at U+00A0 and above pass through `escape_for_terminal` unchanged.
- **Test:** `test_BC_2_11_008_cyrillic_and_emoji_preserved()`

### AC-008 (traces to BC-2.11.009 postcondition 1)
All codepoints in the range U+0080-U+009F (C1 range, inclusive) are replaced by `char::escape_default` output (e.g., U+0085 -> `\u{85}`, U+009B -> `\u{9b}`).
- **Test:** `test_BC_2_11_009_c1_range_escaped()`

### AC-009 (traces to BC-2.11.009 postcondition 2)
U+00A0 (NBSP, Non-Breaking Space) is NOT escaped by `escape_for_terminal`; it passes through as-is.
- **Test:** `test_BC_2_11_009_nbsp_u00a0_preserved()`

### AC-010 (traces to BC-2.11.009 invariant 2)
The boundary is inclusive: U+0080 escapes, U+009F escapes, U+00A0 does NOT escape. Both boundary values are verified.
- **Test:** `test_BC_2_11_009_c1_boundary_inclusive()`

### AC-011 (traces to BC-2.11.010 postcondition 1)
`TerminalReporter::render` applies `escape_for_terminal` to `Finding.summary` — raw C0/DEL/C1 bytes in the summary do not appear in the rendered output.
- **Test:** `test_BC_2_11_010_summary_is_escaped()`

### AC-012 (traces to BC-2.11.010 postcondition 2)
`TerminalReporter::render` applies `escape_for_terminal` to EACH entry in `Finding.evidence` independently — raw C0/DEL/C1 bytes in evidence do not appear in the rendered output.
- **Test:** `test_BC_2_11_010_evidence_each_entry_is_escaped()`

### AC-013 (traces to BC-2.11.011 postcondition 1)
`TerminalReporter::render` applies `escape_for_terminal` to each value in `AnalysisSummary.detail` (converted via `val.to_string()` first). A C1 CSI byte (U+009B) in a detail value is escaped to `\u{9b}` in the output.
- **Test:** `test_BC_2_11_011_analyzer_detail_c1_escaped()`

### AC-014 (traces to BC-2.11.012 postcondition 1)
End-to-end: an HTTP path-traversal `Finding` whose `summary` contains U+009B produces terminal output where U+009B appears as `\u{9b}`, not as raw 0xC2 0x9B bytes.
- **Test:** `test_BC_2_11_012_http_finding_c1_end_to_end()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| escape_for_terminal | src/reporter/terminal.rs:44-61 | pure |
| TerminalReporter::render (header section) | src/reporter/terminal.rs:83-110 | pure |
| render_finding_prefix | src/reporter/terminal.rs:196-218 | pure |
| analyzer summary detail loop | src/reporter/terminal.rs:164-174 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | skipped_packets = 0 | No "Skipped:" line |
| EC-002 | skipped_packets = u64::MAX | "Skipped: ... packets" present |
| EC-003 | ESC (0x1B) in summary | Escaped via char::escape_default |
| EC-004 | DEL (0x7F) | Escaped |
| EC-005 | Backslash in summary | `\\` |
| EC-006 | C1 CSI (U+009B) | `\u{9b}` |
| EC-007 | U+00A0 (NBSP) | Passes through unchanged |
| EC-008 | Cyrillic in evidence | Preserved |
| EC-009 | Emoji in analyzer detail | Preserved |
| EC-010 | Empty evidence Vec | No evidence lines; no crash |
| EC-011 | Control byte in both summary AND evidence | Both independently escaped |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| escape_for_terminal | pure | Pure string transformation; no I/O; no global state |
| TerminalReporter::render | pure | Returns owned String; no I/O |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| src/reporter/terminal.rs (escape_for_terminal + render sections) | ~3,500 |
| BC files (7 BCs) | ~7,000 |
| tests/reporter_terminal_tests.rs (escape and skipped tests) | ~1,500 |
| Tool outputs overhead | ~500 |
| **Total** | **~15,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7.8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-014 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/reporter/terminal.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `escape_for_terminal` predicate: `c.is_ascii_control() || ('\u{80}'..='\u{9f}').contains(&c) || c == '\\'`
5. [ ] Confirm `escape_for_terminal` has exactly ONE production call site (TerminalReporter only)
6. [ ] Confirm skipped_packets guard is `if summary.skipped_packets > 0` at terminal.rs:94
7. [ ] Confirm escape called at render_finding_prefix for summary (line 197) and evidence (line 216)
8. [ ] Confirm escape called at analyzer detail loop line 172 (`escape_for_terminal(&val.to_string())`)
9. [ ] Run `cargo test --all-targets` to confirm green

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-076 | JsonReporter does NOT escape; delegates to serde_json | Escaping is TerminalReporter-only per ADR 0003 / INV-4 | C1 bytes that serde_json passes through must be re-escaped by TerminalReporter (BC-2.11.011 rationale) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `escape_for_terminal` has exactly ONE production call site — inside TerminalReporter | BC-2.11.007 invariant 1 | Grep codebase for `escape_for_terminal`; only terminal.rs may contain production call sites |
| C1 predicate is `('\u{80}'..='\u{9f}').contains(&c)` — inclusive on both ends | BC-2.11.009 invariant 1 | Code review of terminal.rs:52 |
| `escape_for_terminal` is applied to BOTH `f.summary` AND each entry in `f.evidence` | BC-2.11.010 invariant 2 | Code review: two separate call sites in render_finding_prefix |
| `escape_for_terminal` is applied to `val.to_string()` for ALL analyzer detail values | BC-2.11.011 invariant 1 | Code review: terminal.rs:172 |
| No C1 exception exists within U+0080-U+009F — the ENTIRE range is escaped | BC-2.11.007 postcondition 3 | Test boundary values: U+0080, U+009F, U+00A0 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| owo_colors | (per Cargo.lock) | `OwoColorize` for colorized output (use_color=true paths) |
| std::char::escape_default | stdlib | C0/C1/DEL escape sequences |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reporter/terminal.rs | verify/modify | escape_for_terminal (lines 44-61), render (83-178), render_finding_prefix (196-218), detail loop (164-174) |
| tests/reporter_terminal_tests.rs | create or modify | AC-001 through AC-014 tests |

## Revision History

| Version | Date | Change |
|---------|------|--------|
| v1.2 | 2026-05-30 | corrected test-file citation reporter_tests.rs → reporter_terminal_tests.rs (FSR + Token Budget rows); Wave-21 wave-level traceability finding F-W21-TRACE-001 |
