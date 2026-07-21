# PR #431 Review — docs(maint): add IEC-104 documentation and fix doc drift

**Verdict: APPROVE** (3 non-blocking suggestions/nit)

Fresh-eyes review. Documentation-only PR + one doc-comment fix in `src/cli.rs`.
All 13 CI checks green. Scope matches stated intent (docs + doc-comment; no
behavioral change). No blocking findings.

## Independently verified

- **DOC-005 alignment is real.** `src/cli.rs:185` (Modbus write-burst arg) uses
  "within any 1s window"; changing the ENIP arg at line 259 from "1-second window"
  to "1s window" is a legitimate consistency fix. CHANGELOG target `src/cli.rs:259`
  is exact.
- **CHANGELOG obligation satisfied.** Only `src/cli.rs` triggers the changelog-gate
  (README / CLAUDE.md / docs/ are excluded). The single "Fixed" entry accurately and
  completely describes that one src change.
- **Row-verify mandate (PG-W74-PRDESC-ROW-VERIFY) correctly stated as not triggered.**
  No per-test results table, no aggregate test counts. Confirmed.
- **ADR-0013 exists** (`docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`)
  and "Decision 8" is a real section titled "Pure-core free-fn design for VP-044 Kani
  amenability" — the ADR-0002 cross-reference resolves.
- **ADR-0001 additions** (struct field / enum variant / rule table) match the existing
  `Enip` (Rule 7) formatting exactly. IEC-104 as Rule 8 with "No match" pushed to Rule 9
  is internally consistent and matches the README "Rule 8" prose.

## Findings

| Severity | Category | File | Finding | Suggestion |
|----------|----------|------|---------|------------|
| LOW (suggestion) | coherence/missing | docs/adr/0001-content-first-stream-dispatch.md:104 | Consequences bullet enumerates wrapped stream analyzers (`Option<HttpAnalyzer>` … `Option<EnipAnalyzer>`) but omits `Option<Iec104Analyzer>`. This is the exact drift class DOC-002 targets in this same file; the list was kept current through EnipAnalyzer (v0.11.0). | Append `Option<Iec104Analyzer>` for parity. Non-blocking. |
| LOW (suggestion) | coherence | docs/adr/0002-modular-protocol-analyzers.md:159 | Heading `### Deviations from generic traits (DNP3 and ARP)` already omitted EtherNet/IP (pre-existing) and now a 4th entry (IEC-104) is added without updating it. | Generalize heading (e.g. drop the parenthetical, or list all four). Pre-existing drift, adjacent to edits. |
| NIT | accuracy/precision | docs/adr/0002-modular-protocol-analyzers.md (Deviations, IEC-104 entry) | Rationale says inherent methods are used because of "pure-core / effectful-shell separation mandated by ADR-013 Decision 8 … incompatible with the `StreamHandler` trait's required method signatures." Decision 8 actually mandates that two *parser* helpers (`parse_apci_header`, `classify_frame_format`) be free `fn`s for Kani; it does not establish that the `on_data`/`on_flow_close` shell cannot implement `StreamHandler`. Thematically consistent with the sibling DNP3 entry (also cites Kani), so defensible. | Optionally tighten wording to separate parser-purity from the dispatch-interface deviation. |

## Notes (no change needed)

- The new IEC-104 deviation entry uses `ts: u32` while the adjacent EnipAnalyzer entry
  uses `timestamp: u32`. `ts` is correct — it matches the real IEC-104 signature. Each
  entry is accurate to its own analyzer.
- ADR reference style ("ADR-013", 3-digit) matches the existing "ADR-010" convention;
  ADR-0002 already mixes "ADR-010"/"ADR-0007", so no regression.

## Checklist disposition

1. Diff coherence — PASS (all changes relate to IEC-104 doc drift + one aligned doc-comment)
2. Description accuracy — PASS (finding table matches diff)
3. Test coverage — N/A (docs-only; no behavioral change)
4. Demo evidence — N/A (documentation maintenance PR, no ACs)
5. Commit quality — PASS (semantic title `docs(maint): …`)
6. Diff size — PASS (small, well under 500 lines)
7. Missing changes — 2 residual-drift gaps flagged above (LOW, non-blocking)
8. Dependency status — N/A (standalone maintenance PR)
