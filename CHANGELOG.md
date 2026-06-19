# Changelog

All notable changes to wirerust are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version numbers follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.9.0] - 2026-06-19

### Changed (BREAKING)

- **`TerminalReporter` findings-render mode: two bools → `FindingsRender` enum → `FindingsRender`
  struct of two orthogonal enums (STORY-120 PR #266, STORY-122/A PR #268).**
  This entry supersedes the three-variant enum description that shipped in an earlier 0.9.0
  pre-release entry.

  *Phase 1 (STORY-120, PR #266):* The `show_mitre_grouping: bool` and `collapse_findings: bool`
  public fields on `TerminalReporter` were removed and replaced by a single `render: FindingsRender`
  field typed as a three-variant enum (`Grouped`, `FlatCollapsed`, `FlatExpanded`).

  *Phase 2 (STORY-122/A, PR #268):* `FindingsRender` was reshaped from a three-variant enum into
  a **struct of two orthogonal enums**: `{ grouping: Grouping, collapse: Collapse }`. The
  `Grouping` enum has variants `Grouped` and `Flat`; the `Collapse` enum has variants `Collapsed`
  and `Expanded`. All four combinations are valid. The three named enum variants (`Grouped`,
  `FlatCollapsed`, `FlatExpanded`) no longer exist. Per RFC 1105 this is an additional breaking
  change: any code that matched or constructed the three-variant enum must migrate to the
  two-field struct. The 0.8.x → 0.9.0 minor bump covers both phases.

### Changed

- **`--mitre` now collapses identical findings within each MITRE tactic bucket by default
  (STORY-119/B, PR #269).** When `--mitre` is passed, `wirerust analyze` routes output through
  the new `render_findings_grouped_collapsed` path, which groups identical findings (same category,
  verdict, confidence, summary) within each tactic bucket into a single line with a `(xN)` count
  suffix and up to K=3 representative evidence samples. Singletons render without a count suffix.
  Terminal output for `--mitre` is **no longer byte-identical** to the pre-0.9.0 grouped output.
  JSON and CSV output are unaffected.

- **`--no-collapse` is now dual-scope (STORY-119/B, PR #269).** Previously `--no-collapse`
  suppressed collapse only in flat (non-`--mitre`) mode. It now suppresses collapse in both flat
  and grouped (`--mitre`) modes. Passing `--no-collapse` restores one-line-per-finding output
  regardless of whether `--mitre` is also passed.

## [0.8.0] - 2026-06-17

### Added

- `--no-collapse` flag for `wirerust analyze` to opt out of terminal finding-collapse (closes
  #259, STORY-118). Pass `--no-collapse` to restore the pre-v0.8.0 one-line-per-finding output.

### Changed

- **Terminal `analyze` output now collapses repeated findings by default.** Findings that share
  the same (category, verdict, confidence, summary) are collapsed into a single line with a
  `(xN)` count suffix and up to 3 representative evidence samples (K=3). This is a
  **display-layer-only behavioral change**: JSON and CSV output are unaffected, and
  `--mitre`-grouped mode was unchanged in 0.8.0; grouped-mode collapse shipped in 0.9.0.
  Pass `--no-collapse` to disable. Governed by ADR-0003 Display-Layer Aggregation.

## [0.7.1] - 2026-06-17

### Added

- Regression test coverage for VLAN / QinQ (802.1ad double-tag) / MACsec link-extension ARP
  offset handling — 10 tests across `tests/bc_2_16_qinq_macsec_offset_tests.rs` and
  `tests/bc_2_16_e17_macsec_offset_tests.rs` (issue #253, STORY-116/117). Includes an
  off-by-8 SCI-accounting guard for MACsec-tagged ARP.

### Notes

- No runtime behavior change: the VLAN/QinQ/MACsec offset handling itself shipped in 0.7.0;
  this release adds regression guards. MACsec-over-ARP offset correctness is proven by
  etherparse source + upstream proptests + synthetic tests and is documented as an
  evidence-backed limitation (no public on-wire MACsec+ARP capture exists).

## [0.7.0] - 2026-06-16

### Added

- **ARP Security Analyzer** (issue #9, epic E-16) for link-layer and OT network forensics.
  Detects five threat classes with MITRE ATT&CK attribution:

  - **D1 ARP spoofing** — binding-conflict detection with MEDIUM→HIGH severity escalation
    (configurable `--arp-spoof-threshold`, default 3 conflicts). Attributed to **T0830
    Adversary-in-the-Middle** and **T1557.002 ARP Cache Poisoning**.
  - **D2 Gratuitous ARP (GARP)** — unsolicited GARP frames flagged as Possible; binding-conflict
    GARP (GARP where the announced MAC differs from the established binding) escalated to Likely.
  - **D3 ARP storms** — high-rate ARP flood detection (configurable `--arp-storm-rate`, default
    50 frames/window). Attributed to **T0830**.
  - **D11 Malformed ARP frames** — strict + lax/snaplen-truncated ARP parsing; frames that fail
    both passes are flagged as malformed-protocol anomalies.
  - **D12 L2/L3 MAC mismatch** — Ethernet source MAC vs. ARP sender hardware address mismatch
    detection, flagging potential header spoofing.

  New CLI flags: `--arp` (enable; also included in `-a`/`--all`), `--arp-spoof-threshold N`,
  `--arp-storm-rate N`. Binding-table LRU cap: 65 536 entries; storm-counter LRU cap: 4 096
  entries.

  Implemented across STORY-111..115 (PRs #236, #238, #239, #240, #241) with formal hardening
  in PRs #242–#251.

### Changed

- Migrated the packet decoder from **etherparse 0.16 to 0.20** (`DecodedFrame{Ip,Arp}` model).
  Strict and lax/snaplen-truncated ARP parsing added; VLAN/QinQ/MACsec link-extension offset
  handling included.
- Bumped **chrono 0.4.44 → 0.4.45** (#237).

### Verified

- **VP-024 ARP parse-safety and binding-cap** formally verified: 5 Kani proof harnesses proven
  correct, cargo-fuzz 16.2 M executions / 0 crashes, cargo-mutants 98.9 % kill rate on the
  ARP delta.

## [0.6.0] - 2026-06-12

### Added

- **DNP3 TCP protocol analyzer** for ICS/OT network forensics (Feature #8, PRs #219–#231).
  Analyzes TCP streams on port 20000 per IEEE Std 1815-2012 (DNP3); dispatched as Rule 6 in the
  stream dispatcher after content-signature rules (TLS record, HTTP prefix) and port rules for
  TLS, HTTP, and Modbus — it never misclassifies TLS or HTTP traffic
  (BC-2.15.021 INV-2, ADR-007 Decision 1).

  Parses the 10-byte DNP3 data-link layer header: sync bytes, LENGTH, CONTROL, DEST/SRC link
  addresses (little-endian per IEEE 1815-2012 §8.2). Classifies application-layer function codes
  into six classes: Read, Write, Control, Restart, Management, Response. Per-flow state with a
  292-byte carry-buffer frame-walk handles fragmented TCP delivery and desync detection.

  Emits findings mapped to **5 MITRE ATT&CK for ICS techniques**:

  - **T1692.001** Unauthorized Message: Command Message — direct-operate burst (Control-class FCs
    exceed the per-flow threshold within a 60-second detection window), unexpected master source
    (Control FC from a source address not in the established master set), and broadcast control
    command (Control FC to a DNP3 broadcast destination address)
  - **T1691.001** Block Operational Technology Message: Command Message — Control-class requests
    that receive no matching RESPONSE (FC 0x81) within 10 seconds contribute to a block-event
    counter; fires when >= 3 block events accumulate within the 300-second correlation window
  - **T0827** Loss of Control — fires when the combined count of restart events and block-command
    events reaches >= 3 within the 300-second correlation window (co-emitted after T0814 or
    T1691.001)
  - **T0814** Denial of Service — emitted per cold/warm restart command (FC 0x0D / FC 0x0E), and
    as a malformed-frame anomaly when >= 3 parse-invalid frames are observed within the 300-second
    correlation window
  - **T0836** Modify Parameter — emitted per WRITE command (FC 0x02)

  Additional T0814 trigger sources (Inhibit Response Function):
  - DISABLE_UNSOLICITED (FC 0x15): verdict Likely / confidence Medium — alarm suppression /
    event-blinding primitive; emitted per occurrence.
  - ENABLE_UNSOLICITED (FC 0x14): verdict Possible / confidence Low — unsolicited reporting
    control; emitted per occurrence; also sets the per-flow context flag that suppresses the
    unsolicited-response anomaly.
  - Unsolicited-response anomaly: UNSOLICITED_RESPONSE (FC 0x82) arrives on a flow where
    ENABLE_UNSOLICITED was never observed and no solicited exchange has been seen; verdict
    Possible / confidence Low; one-shot per flow (T0814).

  Bounded-resource design: per-flow state capped at 64 tracked master addresses, 256 pending
  requests, and 10,000 total findings; 300-second correlation window with six windowed counters
  reset together (ADR-007 Decision 4).

- **CLI flags for the DNP3 analyzer:**
  - `--dnp3` — enable DNP3 TCP analysis (also included in `-a`/`--all`; default-off,
    BC-2.15.021)
  - `--dnp3-direct-operate-threshold N` — per-flow direct-operate burst threshold; fires T1692.001
    when Control-class FC count exceeds N within the 60-second detection window (default: 10,
    BC-2.15.017)

- **Dispatcher Rule 6** — Port-20000 classification added to the stream dispatcher as Rule 6
  (STORY-110, ADR-007 Decision 1). Fires after content-signature rules (Rules 1–2) and port rules
  for TLS/HTTP/Modbus (Rules 3–5), preserving the VP-004 port-precedence invariant.

- **`MitreTactic::IcsImpact` tactic variant** — new variant added to the `MitreTactic` enum
  (STORY-109, VP-007 obligation). Maps to the MITRE ATT&CK for ICS "Impact" tactic (TA0105).
  Used exclusively by T0827 "Loss of Control". Added atomically with the T0827 emission branch
  and the `technique_info("T0827")` catalog entry.

- **`T1691.001` and `T0827` catalog entries** — two new technique IDs seeded in the static MITRE
  catalog (`technique_info`): T1691.001 "Block Operational Technology Message: Command Message"
  (IcsInhibitResponseFunction) and T0827 "Loss of Control" (IcsImpact). Total catalog size: 23
  technique IDs (STORY-109, VP-007).

- **Formal verification and quality assurance for the DNP3 analyzer:**
  - VP-023 (Kani): parse safety sub-properties A–D: all-input range, FC totality, frame-length
    bounds, carry-buffer progress.
  - Fuzz testing: `fuzz_dnp3_parse` target added (PR #229).
  - Mutation testing: 100% effective kill rate on the detection core including edge cases for
    window-seeding (PR #231).

- **T0814 full detection surface documented** (DRIFT-DNP3-DOC-T0814-COMPLETENESS-001). The DNP3
  T0814 "Denial of Service / Inhibit Response Function" technique is emitted from five trigger
  sources: cold/warm restart command (FC 0x0D/0x0E; verdict Likely/High), DISABLE_UNSOLICITED
  (FC 0x15; verdict Likely/Medium), ENABLE_UNSOLICITED (FC 0x14; verdict Possible/Low),
  unsolicited-response anomaly (FC 0x82 on a flow with no prior ENABLE_UNSOLICITED; verdict
  Possible/Low), and malformed-frame anomaly (>= 3 parse-invalid frames in the 300s window;
  verdict Possible/Low). README and CHANGELOG now enumerate all five sources.

## [0.5.0] - 2026-06-10

### Fixed

- **Behavioral change — emitted output:** Remapped revoked MITRE ATT&CK-ICS techniques to their
  replacement IDs in the pinned ics-attack-19.1 catalog (issue #222):
  - `T0855` "Unauthorized Command Message" → **`T1692.001`** "Unauthorized Message: Command Message"
    (ICS sub-technique under parent T1692 "Unauthorized Message"). **Behavioral change:** Modbus
    findings now emit `T1692.001` instead of `T0855` in the `mitre_techniques` field of all JSON,
    terminal, and CSV output. Tactic (IcsImpairProcessControl) and co-emission ordering are unchanged.
  - `T0856` "Spoof Reporting Message" → **`T1692.002`** "Unauthorized Message: Reporting Message"
    (ICS sub-technique under T1692). Catalog-only (seeded, never emitted); no emitted output affected.

## [0.4.0] - 2026-06-10

### Added

- **Modbus TCP protocol analyzer** for ICS/OT network forensics (Feature #7, issue #7, PRs #211–#218).
  Detects Modbus traffic on port 502; parses the MBAP header (transaction ID, protocol ID, length,
  unit ID) and function code; per-flow transaction correlation with bounded pending-table (request /
  response matching). Emits findings mapped to **7 MITRE ATT&CK for ICS techniques**:
  - T0855 Unauthorized Command Message (write-class function codes)
  - T0836 Modify Parameter (write-register / write-coil)
  - T0835 Manipulate I/O Image (force-listen-only, write-multiple coils)
  - T0831 Manipulation of Control (mask write register, write file record)
  - T0806 Brute Force I/O (sustained coil/register write flooding)
  - T0814 Denial of Service (exception-burst flooding pattern)
  - T0888 Remote System Information Discovery (FC-scanning / register-map enumeration via exception
    burst on recon function codes 0x01/0x02)

  Multi-tag co-emission: one finding per write PDU carrying the union of applicable techniques.
  Dual-window write-rate detection: burst threshold (>20 writes/1 s, configurable) + sustained
  threshold (>10 writes/s over ≥2 s, configurable). Exception-burst anomaly detection triggers
  T0888 on recon-code exception runs. Per-analyzer summary reports function-code distribution,
  write count, exception count, and PDU count.

- **CLI flags for the Modbus analyzer:**
  - `--modbus` — enable Modbus TCP analysis (also included in `-a`/`--all`)
  - `--modbus-write-burst-threshold N` — burst detection threshold (default 20 writes/1 s)
  - `--modbus-write-sustained-threshold N` — sustained-rate threshold (default 10 writes/s over ≥2 s)

- **Dispatcher port-502 classification** — Rule 5 in the stream dispatcher classifies port-502
  flows for Modbus after content-signature rules and the 443/8443/80/8080 port rules; it never
  steals HTTP or TLS traffic (VP-004 port-precedence invariant preserved, PR #214).

- **Formal verification and quality assurance for the Modbus analyzer:**
  - VP-022 (Kani): MBAP parse safety, function-code classification totality, exception-code
    biconditional invariant.
  - Fuzz testing: 3.7 M executions, 0 crashes (PR #216).
  - Mutation testing: 100 % effective kill rate on the detection core (PR #216).
  - E2E integration: pcap fixture + end-to-end flow tests (PR #217).
  - T0888 blemish fix: exception-burst correctly emits T0888 for recon function codes 0x01/0x02
    (PR #218, BC-2.14.019).

- **Architecture records:**
  - ADR-005 — Binary ICS protocol integration strategy.
  - ADR-006 — Multi-technique Finding attribution model.

## [0.3.0] - 2026-06-09

### Changed (BREAKING)

- **Finding MITRE attribution: scalar → array (ECS-aligned).** `Finding.mitre_technique: Option<String>` has been renamed to `mitre_techniques: Vec<String>`. In JSON output the field is now `"mitre_techniques"` (an array); it is omitted entirely when empty. Downstream JSON consumers must update to read an array instead of a scalar. In CSV output the column is renamed `mitre_techniques`; multiple values are semicolon-joined (e.g. `T0855;T0836`); a single value is written without a separator; an empty value is an empty string. The terminal reporter now renders `MITRE: T0855, T0836` for multi-technique findings and groups by the first technique's tactic. This aligns the schema with Elastic ECS `threat.technique.id` (PR #209, STORY-100/101).

- **JSON report envelope: new fields.** Every JSON report now includes two top-level envelope fields: `"mitre_domain": "ics-attack"` and `"mitre_attack_version": "ics-attack-19.1"`. The domain is constant (wirerust targets the ATT&CK for ICS matrix). The version is pinned to ATT&CK for ICS v19.1 (released 2026-04-28), which covers all 21 seeded technique IDs including the 6 staged ICS entries (STORY-101, PR #209).

### Migration

Downstream consumers of wirerust JSON or CSV output must update for this release:

- **JSON**: The finding attribute changed from `"mitre_technique": "T1027"` (string, may be absent) to `"mitre_techniques": ["T1027"]` (array, omitted when empty). Update any field reads to `obj["mitre_techniques"][0]` for single-technique findings or iterate the array for multi-technique ones.
- **CSV**: Column 6 changed from `mitre_technique` to `mitre_techniques`. Multi-value cells are semicolon-joined; split on `";"` to get individual technique IDs.
- **JSON envelope**: Two new top-level keys (`mitre_domain`, `mitre_attack_version`) are now always present. If your parser requires a strict fixed key set, add these two keys to your allowlist.

### Added

- **MITRE ICS catalog expanded.** The technique catalog grew from 15 to 21 seeded entries. Six new ICS technique IDs are staged for the upcoming Modbus analyzer (STORY-104): T0836 (Modify Parameter), T0814 (Deny Control), T0806 (Brute Force I/O), T0835 (Manipulate I/O Image), T0831 (Manipulation of Control), T0888 (Remote System Information Discovery). T0855 (Unauthorized Command Message) is now emitted by the TLS analyzer. Total emitted count: 13 (6 Enterprise + 7 ICS), up from 6 emitted in v0.2.0 (PR #209, STORY-100/101).

## [0.2.0] - 2026-06-09

### Added

- **Finding timestamp provenance** — every `Finding` now carries a
  `capture_ts` field populated with the pcap capture-relative timestamp of
  the packet that triggered the finding. The timestamp is threaded from the
  pcap reader through `StreamHandler::on_data` all the way to each Finding
  emission site in the TLS and HTTP analyzers. It is surfaced as an RFC 3339
  string in JSON output and as a new `timestamp` column in CSV output
  (#100; PRs #197, #198, #199; BC-2.04.055, BC-2.09.007, VP-021).
  Segment-limit summary findings intentionally carry no timestamp (correct
  by design).

### Fixed

- SNI control-byte summary now correctly surfaces control bytes in the
  human-readable finding for mixed control + non-ASCII values (#104, PR #194).
- Weak-cipher evidence vector is capped at 64 entries with an elision marker
  to prevent unbounded growth on adversarial captures (#102, PR #195).

### CI / Build / Supply-chain

- Migrated release workflow actions from Node 20 to Node 24 with fresh
  SHA-pinned refs (`upload-artifact` v7.0.1, `download-artifact` v8.0.1,
  `softprops/action-gh-release` v3.0.0); added Dependabot tracking for
  workflow actions (PR #192).
- SHA-pinned all remaining CI actions (`actions/checkout`, `rust-cache`,
  `cargo-deny`, `amannn/action-semantic-pull-request`) and added the
  **action-pin-gate** enforcement job that fails CI if any action ref is
  not a 40-char hex SHA (PR #196).
- Test and spec hardening for timestamp provenance: exact-value assertions
  replacing approximate checks, stale doc-comment corrections (PRs #200, #201).

## [0.1.0] - 2026-06-08

### Added

**Core pipeline**

- PCAP reader supporting five link types: Ethernet (1), Raw IP (101), Linux
  Cooked / SLL (113), IPv4 (228), and IPv6 (229). Snaplen-truncated captures
  (e.g. `tcpdump -s 96`) are accepted via the unvalidated raw-record path.
  pcapng is not supported.
- Zero-copy L2–L4 packet decoding via `etherparse`. The full capture is loaded
  into memory as a `Vec<RawPacket>` before analysis; available RAM determines
  the practical file-size limit.
- Single-pass analysis pipeline: Reader → Decoder → Analyzers → Reporter,
  producing host/service/protocol summaries and threat findings in one pass.
- Directory expansion: pass a directory path and wirerust processes every
  `.pcap` file found within it (`.pcapng` files are excluded).

**TCP stream reassembly engine**

- Forensic-grade TCP stream reassembly with a first-wins overlap policy
  (earlier-arriving data wins on byte conflicts).
- Configurable per-direction depth limit (`--reassembly-depth`, default 10 MB)
  and global memory cap (`--reassembly-memcap`, default 1024 MB).
- Evasion and anomaly detection: overlapping-segment counting
  (`--overlap-threshold`, default 50 per flow direction), consecutive
  small-segment detection (`--small-segment-threshold`, default 100 run
  length; `--small-segment-max-bytes`, default 16 B), and out-of-window
  segment counting (`--out-of-window-threshold`, default 100).
- Interactive-protocol port exemption from small-segment detection (default:
  ports 23 and 513; overridable via `--small-segment-ignore-ports`).
- Idle-flow expiry: flows silent longer than `--flow-timeout` seconds
  (default 300) are evicted from the flow table.
- Reassembly statistics surfaced in all output formats: bytes reassembled,
  segment-limit drops, overlap count, out-of-window count, and small-segment
  count.

**Protocol analyzers**

- DNS analyzer: traffic statistics including query/response counts,
  top queried hostnames, and query-type distribution.
- HTTP/1.x analyzer (requires TCP reassembly): stream-level request and
  response parsing with detection for path traversal sequences, web-shell
  indicators, unusual HTTP methods, missing or empty Host headers, and other
  header anomalies. Parse-error isolation prevents one poisoned stream from
  affecting other flows.
- TLS analyzer: ClientHello and ServerHello parsing; SNI extraction and
  classification (clean ASCII, ASCII control bytes C0/DEL, valid non-ASCII
  UTF-8, non-UTF-8 bytes); JA3 and JA3S fingerprinting with GREASE
  value filtering; weak cipher detection; deprecated SSL 2.0 and 3.0
  detection.
- Stream dispatcher: content-first protocol classification (TLS record
  signature, HTTP prefix, then port-based fallback) with classification
  caching and a configurable retry budget (`max_classification_attempts`).

**Threat detection and MITRE ATT&CK**

- Finding system with verdict, confidence score, source IP, direction tag,
  and optional MITRE ATT&CK technique ID.
- Static MITRE ATT&CK catalog mapping technique IDs (T-format) to tactic and
  technique name, consumed by the terminal reporter when `--mitre` is passed.
- `--mitre` flag groups terminal output by ATT&CK tactic with technique names
  displayed alongside each finding.

**Output formats and CLI**

- Colored terminal reporter with MITRE tactic grouping, top-SNI and top-host
  tables, reassembly statistics section, and skipped-packet accounting.
  Deterministic tie-ordering for top-SNI and top-host tables.
- JSON reporter: structured output with deterministic field ordering,
  `skipped_packets` counter, and `dropped_findings` counter. `#[non_exhaustive]`
  on public enums for forward compatibility.
- CSV reporter: 9-column findings table (tactic, verdict, confidence,
  source IP, destination IP, port, protocol, description, MITRE technique).
  CSV-injection neutralization applied to all string fields. Evidence strings
  joined with a pipe separator.
- Output routing: `--output-format json|csv` writes to stdout; `--json [FILE]`
  and `--csv [FILE]` write to a file (or stdout if no path is given).
  `--json` and `--csv` are mutually exclusive.
- `analyze` subcommand with `--dns`, `--http`, `--tls`, `--mitre`, and
  `-a/--all` flags. HTTP analysis automatically enables TCP reassembly.
- `summary` subcommand with optional `--hosts` flag for a per-host IP
  breakdown. Outputs total packets, bytes, protocol distribution, and
  service-hint counts.
- `--no-color` flag disables ANSI color globally.
- Zero, non-integer, or out-of-range values for `--reassembly-depth` and
  `--reassembly-memcap` are rejected at argument-parse time.

**Observability**

- `dropped_findings` counter tracks findings discarded when the per-analyzer
  cap is reached; surfaced in JSON output.
- `skipped_packets` counter tracks packets skipped during decode; surfaced in
  all output formats.
- `truncated_records` counter tracks snaplen-truncated records; surfaced in
  JSON output.
- Criterion micro-benchmarks for hot paths in the decoder and reassembly
  engine.

### Security

- Bumped `indicatif` from 0.17 to 0.18 to transitively drop the unmaintained
  `number_prefix` crate (RUSTSEC-2025-0119).
- `cargo audit` and `cargo deny` supply-chain checks added to CI.
- Release profile enables `overflow-checks = true` so integer overflows are
  caught in release builds.
- Output sanitization in the terminal reporter guards against C1 control bytes
  in packet-derived strings.

[Unreleased]: https://github.com/Zious11/wirerust/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/Zious11/wirerust/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Zious11/wirerust/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/Zious11/wirerust/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/Zious11/wirerust/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Zious11/wirerust/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Zious11/wirerust/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Zious11/wirerust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Zious11/wirerust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Zious11/wirerust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Zious11/wirerust/releases/tag/v0.1.0
