# PR Review — #305 `docs: fix documentation drift (maint-2026-06-22)`

**Verdict: APPROVE**

Reviewer: fresh-eyes pr-reviewer (cognitive diversity, different model family).
Scope: documentation-only sweep, 5 files, +347 / −19. No `.rs` or `Cargo.*` files
touched. Focus: documentation accuracy versus current source code.

## Summary

This is a clean, well-scoped documentation-drift correction. Every factual claim
changed by this PR was independently verified against the source tree at the PR's
head commit (`0659443`). All corrections are accurate, all newly documented behavior
matches the implementation, and no aspirational/future-tense claims describe
unimplemented features (DF-GREEN-DOC-TENSE-SWEEP: clean). I found zero BLOCKING
issues.

## Verification Performed (against worktree head `0659443`)

### ADR-0002 (modular protocol analyzers)
- **`AnalysisSummary.detail` → `BTreeMap`** — VERIFIED. `src/analyzer/mod.rs:52`
  declares `pub detail: BTreeMap<String, serde_json::Value>` (doc-comment at lines
  30/38 confirms the deliberate `HashMap → BTreeMap` change for determinism,
  LESSON-P2.09 / NFR DET-001). The old `HashMap` text was genuinely stale. Correct.
- **`on_data` 5th param `timestamp: u32`** — VERIFIED. `src/reassembly/handler.rs:48-56`
  `trait StreamHandler::on_data` has exactly the six-receiver-inclusive parameters
  `(&mut self, flow_key, direction, data, offset, timestamp: u32)`. The ADR's updated
  signature is an exact match. Correct.
- **`parse_error_count` reclassified to convention, not trait obligation** — VERIFIED.
  `grep` of `src/analyzer/mod.rs` shows `parse_error_count` is NOT in the
  `ProtocolAnalyzer` trait definition. It exists only as an inherent method on
  `src/analyzer/http.rs:183` and `src/analyzer/tls.rs:363`. Reclassifying it from
  "Yes (required)" to "Convention only" is factually correct and an improvement.

### ADR-0003 (reporting pipeline layering)
- **Function-name anchors replace stale line numbers** — VERIFIED. All five referenced
  functions exist in `src/main.rs`: `fn main` (91), `fn run_analyze` (142),
  `fn run_summary` (472), `fn collapse_findings_from_flag` (601),
  `fn grouping_from_flag` (610). The `grouping_from_flag` / `collapse_findings_from_flag`
  call sites are inside `run_analyze` (449) and at the `main()` call site (128),
  matching the prose. Replacing brittle line numbers with function-name anchors is
  strictly more durable; the removed line numbers (79-80, 107-108, 373, 383-390,
  451-454, 384, 511, 502) would have been drift magnets. Correct.

### README.md
- **Reader row: classic pcap + pcapng, 5 link types** — VERIFIED. `src/reader.rs`
  doc header (line 1) states "Pcap-format and pcapng-format capture-file reader";
  the whitelist at lines 934-938 (pcapng) and 1208-1212 (multi-IDB) enforces exactly
  `{Ethernet=1, Raw=101, IPv4=228, IPv6=229, LinuxSLL=113}` — 5 link types in both
  formats. The feature-bullet and architecture-table edits are accurate. Correct.

### CLAUDE.md
- ADR-0009 added to the ADR list — consistent with the new `docs/adr/0009-*.md` file.
- `.factory/` description updated from "logs only; STATE.md not yet initialized" to
  "STATE.md, stories, specs, research, maintenance logs" — the `.factory/` tree
  contains `code-delivery/`, `cycles/`, `convergence/`, `demo-evidence/` etc.,
  confirming the prior note was stale. Correct.

### ADR-0009 (NEW, pcapng reader design)
Spot-checked the load-bearing claims against `src/reader.rs`:
- **Decision 1 (`RawBlock`/`next_raw_block` via `pcap-file` 2.0.0, +0 crates)** —
  VERIFIED. `Cargo.toml:29` `pcap-file = "2"`; `src/reader.rs:1073` calls
  `parser.next_raw_block(src)`; comments at 982/994 confirm the `PcapNgParser::new`
  + `next_raw_block` path. The "reject high-level `EnhancedPacketBlock.timestamp`
  because it ignores `if_tsresol`" rationale matches the guard comment at line 367
  ("MUST NOT call `EnhancedPacketBlock::timestamp`").
- **Decision 6 (timestamp helper)** — VERIFIED. `pcapng_timestamp_to_secs_usecs`
  exists (line 369) with base-10/base-2 branches, `if_tsresol==6` fast path, and
  saturating arithmetic exactly as described.
- **Decisions 11/13/14/15** — VERIFIED. `is_pcapng: bool` (218), `skipped_blocks`/
  `opb_skipped` (209-212), `MAX_PCAPNG_FILE_BYTES = 4_294_967_296` (92),
  `MAX_INTERFACE_TABLE_ENTRIES = 65_535` (98), `decode_epb_body` (449) +
  `decode_epb_body_discriminant` twin (555) all present and matching.
- Error-code assignments (E-INP-008 through E-INP-015) are referenced in source
  with matching semantics. The Status is "Accepted" and all text is present-tense,
  describing shipped behavior — no aspirational claims.

## Checklist Outcomes

1. Diff coherence — PASS. Every change is a documentation-drift fix for maint-2026-06-22.
2. Description accuracy — PASS. PR body's change table matches the actual diff (5 files,
   +347/−19; the body states 347 insertions / 19 deletions — exact match).
3. Test coverage — N/A. Docs-only; no source lines changed. CI doc-gates apply only.
4. Demo evidence — N/A for a docs-only maintenance sweep (no behavioral ACs).
5. Commit quality — PASS. Conventional `docs(adr):` / `docs:` messages, clear scope.
6. Diff size — ACCEPTABLE. 319 of 347 added lines are the single new ADR-0009 file;
   the remaining 4 files are small surgical edits. Large-but-justified.
7. Missing changes — none identified. All eight drift findings (DOC-001..008) are addressed.
8. Dependency status — PASS. Standalone sweep on `develop` at dd3b069; no upstream PRs.

## Findings

### ADVISORY-1 (minor wording) — `docs/adr/0002-modular-protocol-analyzers.md:82`
The reclassified cell describes `parse_error_count()` as an "inherent method on each
analyzer struct." In the current tree the method exists only on the HTTP and TLS
analyzers (`src/analyzer/http.rs:183`, `src/analyzer/tls.rs:363`), not on dns/arp/
dnp3/modbus. "Each analyzer struct" slightly over-generalizes. This does not block:
the substantive change (demoting it from a *trait obligation* to a *convention*) is
factually correct and is itself the fix for the drift. Consider tightening to
"inherent method on analyzers that track parse failures (e.g. HTTP, TLS)" in a future
pass.

No BLOCKING findings. No other ADVISORY findings.

## DF-GREEN-DOC-TENSE-SWEEP

Clean. All edited and newly added documentation describes currently-implemented
behavior in present/past tense. ADR-0009 is marked "Accepted" and its decisions map
to shipped code in `src/reader.rs`. No future/aspirational tense describing
unimplemented features was found.
