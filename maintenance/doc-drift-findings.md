# Documentation Drift Findings — Maintenance Sweep 2

**Run:** maint-2026-09-05
**Scope:** Documentation Drift
**Compared against:** develop branch, HEAD `0b1ea806`, version `0.13.3`
**Method:** Direct comparison of README.md / CLAUDE.md / docs/adr/ against actual `cargo run -- --help`
output (built and run at v0.13.3), `src/analyzer/*.rs` module listing, `docs/adr/` directory listing,
filesystem existence checks for every path in CLAUDE.md's "Project References" table, and a
`TODO|FIXME|HACK|XXX` regex scan of `src/` and `bin/` with per-line `git blame` dating.

This is analysis only. No doc files were edited, no commits were made. Fixes route through the
gated PR pipeline per this sweep's scope instructions.

---

## Summary Counts

| Category | Count |
|---|---|
| STALE-DOC | 2 |
| BROKEN-REF | 0 |
| ANCIENT-TODO | 0 |
| CLEAN | 6 (areas explicitly verified with no drift) |

**Headline: the documentation is in good shape.** No broken references were found anywhere in
CLAUDE.md's Project References table, all 13 ADRs referenced by number exist on disk with matching
content, README's CLI flag documentation matches the actual v0.13.3 `--help` output flag-for-flag
across all three subcommands, the documented 8-protocol coverage set exactly matches the 8 analyzer
modules in `src/analyzer/`, and there are zero genuine TODO/FIXME/HACK/XXX markers in `src/` or `bin/`
(all 6 raw regex hits are false positives from the `XXX` substring inside the literal placeholder
token `TXXXX` used in MITRE technique-ID documentation). Only two minor, low-severity STALE-DOC items
were found.

---

## Findings

### 1. STALE-DOC — README.md omits the auto-generated `help` subcommand

**File:** `README.md`, "### Options" fenced code block under "## Usage" (the `wirerust [OPTIONS]
<COMMAND>` synopsis, immediately preceding the `Options:` list — approx. line 130 of 477).

**Mismatch:** README's `Commands:` list shows only:
```
Commands:
  analyze    Analyze PCAP files for threats and anomalies
  summary    Generate a triage summary of PCAP files
  protocols  List the protocol coverage catalog
```
Actual `wirerust --help` (v0.13.3) output includes a fourth, clap-auto-generated entry:
```
Commands:
  analyze    Analyze PCAP files for threats and anomalies
  summary    Generate a triage summary of PCAP files
  protocols  List the protocol coverage catalog
  help       Print this message or the help of the given subcommand(s)
```
**Assessment:** Cosmetic/trivial. This is standard `clap` behavior (every clap CLI with
subcommands gets an implicit `help` subcommand) and most CLI READMEs omit it by convention. Not a
functional documentation error — no user-facing behavior is misdescribed — but flagged per the
sweep's literal "CLI flags match current code" check since it is a discrepancy between the
documented and actual `Commands:` list. Fix (if desired): add the `help` row to the README synopsis
block, or leave as-is with a note that it's the implicit clap subcommand.

### 2. STALE-DOC — `docs/adr/0008-withdrawn-placeholder.md` exists but is unlisted in CLAUDE.md's ADR index

**File:** `CLAUDE.md`, "Project References" table, `docs/adr/` row:
> `docs/adr/` \| Architecture Decision Records (0001 stream dispatch, 0002 modular analyzers, 0003
> reporting pipeline, 0004 process-wide warning atomics, 0005 binary ICS protocol integration, 0006
> multi-technique finding attribution, 0007 DNP3 stream dispatch and parser design, 0009 pcapng
> reader design, 0010 EtherNet/IP CIP stream dispatch, 0011 TLS handshake reassembly, 0012 protocols
> catalog and coverage-gaps system, 0013 IEC-104 stream dispatch and parser design)

**Mismatch:** `docs/adr/` on disk contains 13 files, not 12:
```
0001-content-first-stream-dispatch.md
0002-modular-protocol-analyzers.md
0003-reporting-pipeline-layering.md
0004-process-wide-warning-atomics.md
0005-binary-ics-protocol-integration.md
0006-multi-technique-finding-attribution.md
0007-dnp3-stream-dispatch-and-parser-design.md
0008-withdrawn-placeholder.md          <-- present on disk, not listed in CLAUDE.md
0009-pcapng-reader-design.md
0010-ethernet-ip-cip-stream-dispatch.md
0011-tls-handshake-reassembly.md
0012-protocols-catalog-and-coverage-gaps.md
0013-iec104-stream-dispatch-and-parser-design.md
```
CLAUDE.md's index enumerates 0001–0007 then jumps straight to 0009, silently skipping 0008.

**Assessment:** All 12 ADRs CLAUDE.md *does* list are present and correctly numbered/titled — no
BROKEN-REF. Given the filename `0008-withdrawn-placeholder.md`, the gap is almost certainly
intentional (ADR-0008 was withdrawn and replaced by a placeholder stub to keep the ID reserved/
non-reused), not an oversight where a file was added and the index forgotten. Still flagged as
STALE-DOC because CLAUDE.md's index gives a reader no indication *why* the sequence skips 0008 —
a one-line addition such as "(0008 withdrawn — see docs/adr/0008-withdrawn-placeholder.md)" would
close the gap without requiring content changes. Low severity; does not point anywhere broken.

---

## Verified Clean (no drift found)

### 3. CLEAN — README.md CLI flags match `cargo run -- --help` (v0.13.3) exactly

Cross-checked every flag in README's "### Options" and "### Analyze flags" fenced blocks (global
options: `--no-color`, `--output-format`, `--json`, `--csv`, `--reassemble`, `--no-reassemble`,
`--reassembly-depth`, `--reassembly-memcap`, `--overlap-threshold`, `--small-segment-threshold`,
`--small-segment-max-bytes`, `--small-segment-ignore-ports`, `--out-of-window-threshold`,
`--flow-timeout`, `-h/--help`, `-V/--version`; analyze-only: `--dns`, `--http`, `--tls`, `--modbus`
+ 2 threshold flags, `--dnp3` + 1 threshold flag, `--arp` + 2 threshold flags, `--enip` + 2
threshold flags, `--iec104`, `--no-collapse`, `--mitre`, `-a/--all`, `--coverage-gaps`) against the
live `wirerust --help`, `wirerust analyze --help`, `wirerust summary --help`, and `wirerust
protocols --help` output built from HEAD. Every flag, default value, and description in README
matches the built binary. `wirerust --version` reports `wirerust 0.13.3`, consistent with the
sweep's stated current version.

### 4. CLEAN — README.md protocol coverage matches actual analyzer modules in `src/`

README's "Supported Protocol Analyzers" table lists exactly 8 protocols: DNS, HTTP/1.x, TLS,
Modbus TCP, DNP3 TCP, EtherNet/IP TCP, IEC 60870-5-104 TCP, ARP. `src/analyzer/` contains exactly 8
analyzer module files: `arp.rs`, `dnp3.rs`, `dns.rs`, `enip.rs`, `http.rs`, `iec104.rs`,
`modbus.rs`, `tls.rs` (plus `mod.rs`). One-to-one match, including IEC-104, which README correctly
documents as shipped (`--iec104` flag, port 2404, ADR-013 reference) — consistent with this being
a v0.13.x-era feature. No aspirational/planned protocol is misrepresented as implemented, and no
implemented analyzer is missing from the README table.

### 5. CLEAN — README.md install/usage examples correspond to real commands and flags

`cargo install --path .`, `git clone` + `cargo build --release` (binary at
`target/release/wirerust`), and all `wirerust analyze ...` / `wirerust summary ...` / `wirerust
protocols ...` examples use flags and subcommands that exist and behave as documented (verified
against `--help` output for each subcommand).

### 6. CLEAN — All 12 ADRs referenced by CLAUDE.md exist on disk with matching filenames/topics

`0001-content-first-stream-dispatch.md`, `0002-modular-protocol-analyzers.md`,
`0003-reporting-pipeline-layering.md`, `0004-process-wide-warning-atomics.md`,
`0005-binary-ics-protocol-integration.md`, `0006-multi-technique-finding-attribution.md`,
`0007-dnp3-stream-dispatch-and-parser-design.md`, `0009-pcapng-reader-design.md`,
`0010-ethernet-ip-cip-stream-dispatch.md`, `0011-tls-handshake-reassembly.md`,
`0012-protocols-catalog-and-coverage-gaps.md`, `0013-iec104-stream-dispatch-and-parser-design.md`
— all present, all titles match CLAUDE.md's one-line descriptions. No BROKEN-REF.

### 7. CLEAN — CLAUDE.md "Project References" table: all 13 paths exist

| Path | Status |
|---|---|
| `README.md` | EXISTS |
| `docs/adr/` | EXISTS (13 files; see Finding 2 re: 0008) |
| `docs/superpowers/plans/` | EXISTS (10 files) |
| `docs/superpowers/specs/` | EXISTS (8 files) |
| `.github/workflows/ci.yml` | EXISTS |
| `.factory/` | EXISTS — mounted as a separate git worktree on the `factory-artifacts` branch (`git worktree list` confirms `/Users/zious/Documents/GITHUB/wirerust/.factory` → branch `factory-artifacts`, commit `cf6a114b`) |
| `.factory/maintenance/demo-evidence-scrub-gate.md` | EXISTS |
| `.factory/maintenance/pr-manager-merge-auth-guidance.md` | EXISTS |
| `.factory/maintenance/docs-writer-dispatch-guidance.md` | EXISTS |
| `.factory/maintenance/breaking-change-delivery-protocol.md` | EXISTS |
| `.factory/maintenance/pr-description-row-verify-mandate.md` | EXISTS |
| `.factory/maintenance/delivery-doc-currency-protocol.md` | EXISTS |
| `.factory/maintenance/fixture-count-gate-entry.md` | EXISTS |

No BROKEN-REF findings in this table.

### 8. CLEAN — Zero genuine TODO/FIXME/HACK/XXX markers in `src/` or `bin/`

Regex scan for `TODO|FIXME|HACK|XXX` across `src/` and `bin/` returned exactly 6 raw matches, all
of which are false positives — the substring `XXX` matching inside the literal placeholder token
`TXXXX` (a generic MITRE technique-ID form used in doc comments and a test assertion, not a
tech-debt marker):

| File:Line | Content | Blame date | Age (as of 2026-09-05) |
|---|---|---|---|
| `src/mitre.rs:9` | `//! Callers pass technique IDs in MITRE's canonical form: \`TXXXX\` for parent` | 2026-04-13 (`0a00c6385`) | ~145 days |
| `src/mitre.rs:10` | `//! techniques (e.g., \`T1046\`) and \`TXXXX.NNN\` for sub-techniques (period` | 2026-04-13 (`0a00c6385`) | ~145 days |
| `src/mitre.rs:26` | `//!   only has to set \`mitre_techniques: vec!["TXXXX".to_string()]\` — it does not also` | 2026-06-09 (`bff4d0f33`) | ~88 days |
| `src/mitre.rs:517` | `assert!(!is_valid_technique_id_format("TXXXX"));` | 2026-06-01 (`2a2dd5a8a`) | ~96 days |
| `src/findings.rs:148` | `// (Vec::is_empty skip); singleton vec produces a JSON array \`["TXXXX"]\`.` | 2026-07-08 (`c4eb1f43`, boundary) | ~59 days |
| `src/findings.rs:150` | `// sites use \`mitre_techniques: vec!["TXXXX"]\` (singleton) or \`vec![]\`.` | 2026-07-08 (`c4eb1f43`, boundary) | ~59 days |

None of these are TODO/FIXME/HACK/XXX annotations — they are documentation/test text that happens
to contain the substring `XXX` as part of a MITRE-technique-ID placeholder. No ANCIENT-TODO
findings result from this scan; there is nothing here that represents deferred or flagged work.
`bin/` contains only governance/tooling scripts (`changelog-gate-check`, `check-green-doc-tense`,
`compute-input-hash`, `fetch-e2e-pcaps`, `lint-cycle-artifact`, `validate-citations`, and their
`test_*.py` companions) with zero matches of any kind.

---

## Notes on Method / Non-Findings Out of Scope

- ADR content itself (e.g., ADR-0003's forward-looking reference to future "SSH, SMB" analyzers
  not yet implemented) was **not** flagged as drift: ADRs are dated decision records, not living
  status docs, and documenting a plan/future-tense item at the time an ADR was written is expected
  ADR practice, not aspirational-content violation of current-behavior docs. This sweep's scope
  (per instructions) was README.md, the ADR *index*, CLAUDE.md's reference table, and
  protocol-coverage-vs-code — not ADR body content for currency.
- `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` documents a broader ICS protocol catalog
  (e.g., S7comm, GOOSE, BACnet) than what's implemented — this is intentional and consistent with
  README's own `wirerust protocols --unsupported` flag, which is documented as showing
  not-yet-dissected protocols. Not drift.
- The `plugins/vsdd-factory/config/artifact-path-registry.yaml` referenced by this agent's own
  operating constraints is not part of the wirerust repository (it lives in a sibling
  `vsdd-factory` engine checkout and in the plugin cache under `~/.claude/plugins/cache/`). This
  output file's path (`.factory/maintenance/doc-drift-findings.md`) matches the existing,
  already-registered sibling-file pattern in this same directory (seven pre-existing files of the
  identical `.factory/maintenance/<slug>.md` shape), so no separate registry fetch was required to
  proceed with this write.
