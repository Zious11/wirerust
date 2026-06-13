---
artifact: L2-cap-11
traces_to: ../domain-spec.md
cap_id: CAP-11
title: Reporting and Output
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.1"
modified:
  - date: 2026-06-12
    actor: product-owner
    reason: "Pass-10 remediation F-D10-L02: stale '16 MitreTactic variants' corrected to '17 MitreTactic variants (14 Enterprise + 3 ICS-unique incl. IcsImpact)' per cap-10 v1.6 and BC-2.10.004 v1.5."
---

# CAP-11: Reporting and Output

## What the system does today

`Reporter` (E-37) is a single-method trait with three implementations: `TerminalReporter`
(E-38), `JsonReporter` (E-39), and `CsvReporter` (E-39b). All output goes to stdout or a
file path via `write_output()` in main.rs.

**Sources:** C-18 (reporter/mod.rs), C-19 (reporter/json.rs), C-20 (reporter/terminal.rs),
unnumbered (reporter/csv.rs). BC-RPT-001..019.

## Reporter trait

```rust
pub trait Reporter {
    fn render(
        &self,
        summary:           &Summary,
        findings:          &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String;
}
```

Single immutable-input, owned-String-output method. Implementors own all escaping decisions
(ADR 0003).

## TerminalReporter (E-38)

Fields: `use_color: bool`, `show_mitre_grouping: bool`, `show_hosts_breakdown: bool`.

The `show_hosts_breakdown` field was added (P1.03) to wire the `--hosts` flag from the
`Summary` subcommand; it controls whether the hosts HashSet is rendered in the summary output.

**Escaping contract (INV-4 / ADR 0003):** `escape_for_terminal(s)` is called on all user-
controlled string content before printing. It escapes:
- C0 control bytes (0x00..0x1F) except CR and LF.
- DEL (0x7F).
- Non-CR-LF C1 control bytes (0x80..0x9F).
- Backslash (to `\\`).

Regular UTF-8 graphemes (including U+00A7 section sign) pass through unescaped.

**Rendering structure (BC-RPT-019 -- section order):**
1. Summary header (total packets, hosts, protocols, services).
2. Findings list (grouped by tactic if `show_mitre_grouping`; sorted by verdict_rank then confidence_rank otherwise).
3. Analyzer summaries.

**Color coding** (use_color=true):
- `Likely + High` -> red bold
- `Likely + _` -> yellow
- `Inconclusive` -> cyan
- `Unlikely` -> dimmed

Color testing is documented as intentionally untested (ADR 0003 amendment recommendation,
BC-RPT-018 keep-MEDIUM).

**MITRE tactic grouping:** rendered when `show_mitre_grouping = true`. Uses
`all_tactics_in_report_order()` for stable iteration over the 17 MitreTactic variants.

**U+2192 in output (BC-RAS-049):** The finalize segment-limit finding uses `->` in its
display path, which in context involves U+2192 (RIGHT ARROW). This is NOT ASCII `->`. Any
downstream `grep`-based pipeline assuming ASCII `->` will silently miss these findings.

## JsonReporter (E-39)

Unit struct (no fields). Renders via `serde_json::to_string_pretty` on the combined payload
`{ "summary": Summary, "findings": Vec<Finding>, "analyzers": Vec<AnalysisSummary> }`.

`to_string_pretty(...).unwrap()` is used; the unwrap is infallible by construction because
`serde_json::Value` cannot fail to serialize (BC-RPT-007 / pass-2 R2 Target 8 confirmed).

**JSON map key ordering (deterministic -- P2.09 / #76):** Statistics HashMaps in
`AnalysisSummary.detail` (e.g., `hosts`, `methods`, `status_codes`) are now serialized via
`BTreeMap` rather than `HashMap`, producing deterministic alphabetical key ordering across
runs. This was a non-determinism gap closed by the BTreeMap migration.

**JSON Option handling (symmetric -- P1.02 / #73):** All four Option fields on `Finding`
use `skip_serializing_if = "Option::is_none"`. No Option field ever serializes as `null`.
See CAP-09 for the full schema.

## CsvReporter (E-39b)

Implemented by P2.03 (#84). Unit struct. Produces a CSV string with a fixed 9-column header
(column order as declared in csv.rs:63-73):

```
category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp
```

**CSV injection neutralization:** The `neutralize_csv_injection()` function prefixes any
field value starting with `=`, `+`, `-`, `@`, TAB (`\t`), or CR (`\r`) with a single
quote `'`. This prevents spreadsheet formula injection when the CSV is opened in Excel or
Google Sheets (csv.rs:42).

**Field encoding:**
- All string fields are double-quoted by the `csv` crate (RFC 4180).
- `evidence`: all elements of the Vec are joined with `"; "` and placed in a single cell
  (csv.rs:81: `f.evidence.join("; ")`).
- `timestamp`: renders as empty string (all Finding timestamps are None; open item O-01).
- `direction`: renders as the Debug format of the Direction variant (e.g. "ClientToServer")
  if Some; empty string if None.

## Output routing

`main.rs` `write_output()` function selects the reporter and routes output:
- `--output-format json` or `--json <FILE>`: JsonReporter; output to file if path given, stdout otherwise.
- `--output-format csv` or `--csv <FILE>`: CsvReporter; output to file if path given, stdout otherwise.
- Default: TerminalReporter to stdout.

File path arguments in `--json <FILE>` and `--csv <FILE>` are now wired. Prior gap
(BC-ABS-006, BC-ABS-007) where file paths were ignored is closed.

## Summary rendering (E-36)

`Summary` is populated by `Summary::ingest(packet)` once per decoded packet. Fields:
`total_packets`, `total_bytes`, `skipped_packets`, `hosts: HashSet<IpAddr>`,
`protocols: HashMap<Protocol, u64>`, `services: HashMap<String, u64>`.

`services` is populated by `app_protocol_hint()` (port-based). `Summary.services` uses port
hints; `StreamDispatcher.routes` uses content-first dispatch. These two protocol attributions
can disagree for the same flow (LESSON-P3.01).

## BC references

BC-RPT-001..019: escaping (001-012), sort order (013-015), JSON infallibility (016),
section structure (017-019).
