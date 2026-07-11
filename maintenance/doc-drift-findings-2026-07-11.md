---
document_type: maintenance-findings
sweep: doc-drift
run_id: maint-2026-07-11
sweep_number: 2
producer: sweep2-doc-drift-agent
timestamp: 2026-07-11T00:00:00Z
base_commit: b5e1e15
---

# Documentation Drift Findings — maint-2026-07-11 Sweep 2

Sweep of README.md, docs/adr/, CHANGELOG.md, CLAUDE.md, and src/ TODO/FIXME/HACK
comments against current develop HEAD (b5e1e15).

## Summary Table

| ID | Severity | Location | Description | Fixable |
|----|----------|----------|-------------|---------|
| PG-W-README-JSON-SCHEMA | MEDIUM | README.md:263 | ARP JSON key described as `arp_summary`; counters are actually flat in `analyzers[i].detail` | FIXABLE-AUTO |
| DOC-NEW-001 | LOW | docs/adr/0002-modular-protocol-analyzers.md:180 | ADR-0002 references "tech-debt item PC-023" but the correct ID is PC-020 | FIXABLE-AUTO |
| ROUTE-B-DEFERRED NEW-003 | LOW | docs/adr/0001-content-first-stream-dispatch.md:28–40 | ADR-0001 `StreamDispatcher` struct snippet missing `unclassified_port_counts` and `coverage_gaps_enabled` fields | FIXABLE-AUTO |
| CHANGELOG-D3-T0830-DRIFT-001 | LOW | CHANGELOG.md:806 | v0.7.0 entry claims D3 ARP storms "Attributed to T0830"; current code emits `mitre_techniques: []` | FIXABLE-AUTO |
| ARP-RATE-INTDIV-DOC-001 | LOW | src/analyzer/arp.rs:1006 | `detect_storm` doc-comment formula `count_in_window / max(1, elapsed)` does not note integer truncation | MANUAL |
| DNP3-TUNING-BIDIR-001 | LOW | README.md:408–412 | DNP3 threshold-tuning guidance does not state its bidirectional flow assumption | MANUAL |
| UNIT-FMT-5-20S-001 | LOW | src/cli.rs:185, 192 | Modbus CLI arg doc-comments mix "1-second" and ">= 2s" unit formats | FIXABLE-AUTO |
| README-OPTIONS-L117-NEUTRAL-001 | LOW | README.md:117 | `--arp-storm-rate` description says "threshold" without stating the directional semantics ("at or above") | MANUAL |

## Resolved/Clean Checks

- **ROUTE-B-DEFERRED NEW-002 RESOLVED**: `--coverage-gaps` flag and "Coverage gap detection" section were added to README.md by commit e3ca2bc (#393, maint-2026-07-09 Route A sweep). The tech-debt register still shows ROUTE-B-DEFERRED as "DEFERRED" and should be updated to reflect NEW-002 RESOLVED / NEW-003 OPEN.
- **CLAUDE.md file references**: All referenced paths exist on develop — `README.md`, `docs/adr/0001–0012`, `docs/superpowers/plans/`, `docs/superpowers/specs/`, `.github/workflows/ci.yml`, `bin/compute-input-hash`, `bin/test_compute_input_hash.py`, `bin/check-green-doc-tense`, `bin/lint-cycle-artifact`. The `.factory/` paths are on factory-artifacts as documented.
- **CLAUDE.md commands**: `cargo check`, `cargo build`, `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `bin/compute-input-hash` are all valid on current HEAD.
- **CHANGELOG [Unreleased] accuracy**: The single post-v0.12.0 commit (b5e1e15, "docs: LMR-003 template-conformance exemption + check-green-doc-tense guard tests (#395)") modifies `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`, and `CHANGELOG.md`. The [Unreleased] entry accurately describes `_find_repo_root` helper extraction and five new hermetic self-tests (STORY-162, wave-73).
- **TODO/FIXME/HACK comments in src/**: `grep -rn "TODO\|FIXME\|HACK" src/` returned empty — none present.
- **ADR cross-reference integrity**: All 12 ADR files (0001–0012; 0008 is the withdrawn placeholder) exist in `docs/adr/`. The CLAUDE.md Project References table correctly excludes 0008 and lists the remaining 11.
- **ADR-0002 trait-deviation table accuracy**: The Deviations section correctly describes DNP3 (`on_data` without Direction parameter), ARP (inherent `process_arp` method), and ENIP (inherent `on_data` with `FlowKey` clone) — all verified against current `src/dispatcher.rs`.
- **ENIP `enip_summary` nesting**: `enip.rs:1581` confirms `detail.insert("enip_summary", ...)` — ENIP genuinely nests its output under `analyzers[i].detail.enip_summary`. README line 294 and the Features bullet at line 16 are accurate for ENIP.

---

## Finding Detail

### PG-W-README-JSON-SCHEMA — MEDIUM — FIXABLE-AUTO

**Location:** README.md:263

**Description:** The ARP analyzer section states JSON output counters are "present in `arp_summary`" but the ARP analyzer does not use a nested `arp_summary` key. ARP counters (`bindings_evicted`, `storm_counters_evicted`, etc.) are emitted flat inside `analyzers[i].detail` like all other analyzers — not under a nested sub-key.

**Evidence:**
- README.md:263: `JSON output counters (present in arp_summary when using --json / --output-format json):`
- src/analyzer/arp.rs:809: `analyzer_name: "ARP".to_string()` with flat `detail` BTreeMap (no nested `arp_summary` sub-key inserted anywhere in `summarize()`)
- src/reporter/json.rs:76: `"analyzers": analyzer_summaries` — flat array; no special-casing for ARP
- Compare: src/analyzer/enip.rs:1581 — ENIP explicitly inserts `"enip_summary"` as a nested object in `detail`, creating a genuine `analyzers[i].detail.enip_summary` path. ARP has no equivalent nesting.

**Asymmetry note:** ENIP genuinely uses `analyzers[i].detail.enip_summary` (a nested object). All other analyzers (ARP, DNP3, HTTP, TLS, Modbus) expose their counters flat at `analyzers[i].detail.<key>`. The README ARP section incorrectly implies an `arp_summary` sub-key that mirrors the ENIP pattern but does not exist.

**Proposed fix:** Change README.md:263 from:
```
JSON output counters (present in `arp_summary` when using `--json` / `--output-format json`):
```
to:
```
JSON output counters (present in the ARP analyzer's `detail` object in JSON output, at
`analyzers[i].detail`, when using `--json` / `--output-format json`):
```
This mirrors the phrasing already used correctly in the DNP3 section at README.md:232–233.

---

### DOC-NEW-001 — LOW — FIXABLE-AUTO

**Location:** docs/adr/0002-modular-protocol-analyzers.md:180

**Description:** The ADR-0002 Deviations section for `EnipAnalyzer` references "tech-debt item PC-023," but no item with that ID exists in `.factory/tech-debt-register.md`. The correct ID for the ENIP StreamHandler deviation is PC-020.

**Evidence:**
- docs/adr/0002-modular-protocol-analyzers.md:180: `The deviation from the generic StreamAnalyzer trait is purely that EnipAnalyzer was not retrofitted to that interface (see tech-debt item PC-023).`
- `.factory/tech-debt-register.md`: PC-023 does not appear in the register. PC-020 ("EnipAnalyzer does not implement StreamHandler/StreamAnalyzer traits; dispatcher calls `enip.on_data(flow_key.clone(), ...)` incurring per-packet FlowKey heap allocation") is the matching entry.

**Proposed fix:** Change `(see tech-debt item PC-023)` to `(see tech-debt item PC-020)` at docs/adr/0002-modular-protocol-analyzers.md:180.

---

### ROUTE-B-DEFERRED NEW-003 — LOW — FIXABLE-AUTO

**Location:** docs/adr/0001-content-first-stream-dispatch.md:28–40 (StreamDispatcher struct snippet)

**Description:** The `StreamDispatcher` struct snippet in ADR-0001 is stale. It shows the pre-v0.11.2 fields and is missing two fields added by STORY-153 (feature-protocol-coverage cycle, v0.11.2): `unclassified_port_counts` and `coverage_gaps_enabled`.

**Evidence:**
- docs/adr/0001-content-first-stream-dispatch.md:28–40: struct snippet ends with `unclassified_flows: u64,` with no `unclassified_port_counts` or `coverage_gaps_enabled` fields.
- `grep -n "unclassified_port_counts\|coverage_gaps_enabled" docs/adr/0001-content-first-stream-dispatch.md` returns empty.
- src/dispatcher.rs:99–104: both fields present — `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` and `coverage_gaps_enabled: bool`.

**Note:** ROUTE-B-DEFERRED NEW-002 (README missing `--coverage-gaps` flag) was RESOLVED by commit e3ca2bc (PR #393, maint-2026-07-09). The tech-debt register ROUTE-B-DEFERRED entry should be updated to reflect NEW-002 RESOLVED / NEW-003 OPEN.

**Proposed fix:** Add the two fields to the ADR-0001 struct snippet after `unclassified_flows: u64,`:
```rust
/// Per-port unclassified-flow counter; populated when coverage_gaps_enabled=true.
unclassified_port_counts: HashMap<(TransportProto, u16), u64>,
/// Feature flag: when true, unclassified_port_counts is populated on flow close.
coverage_gaps_enabled: bool,
```

---

### CHANGELOG-D3-T0830-DRIFT-001 — LOW — FIXABLE-AUTO

**Location:** CHANGELOG.md:806 (v0.7.0 section)

**Description:** The v0.7.0 CHANGELOG entry for the ARP analyzer says D3 ARP storms are "Attributed to **T0830**." The current code emits `mitre_techniques: []` for D3 findings — T0830 attribution is explicitly withheld per DF-VALIDATION-001 / BC-2.16.008 Invariant 3. The historical CHANGELOG entry was not updated when the attribution was removed.

**Evidence:**
- CHANGELOG.md:806: `D3 ARP storms — high-rate ARP flood detection (configurable --arp-storm-rate, default 50 frames/window). Attributed to **T0830**.`
- src/analyzer/arp.rs:15 (module doc): `D3 storm detection (BC-2.16.008) emits MEDIUM/Anomaly findings with mitre_techniques: [] (T0814 withheld per DF-VALIDATION-001) when source MAC rate reaches storm_rate or more.`
- README.md:269–270: `[^1]: D3 storm findings emit mitre_techniques: [] (no technique attributed). T0814 attribution is pending validation per DF-VALIDATION-001 / BC-2.16.008 Invariant 3.`

**Proposed fix:** Add a correction note inline in the CHANGELOG v0.7.0 D3 entry (do not remove the historical claim, add a parenthetical):
```
- **D3 ARP storms** — high-rate ARP flood detection (configurable `--arp-storm-rate`, default
  50 frames/window). ~~Attributed to **T0830**.~~ (Corrected: D3 findings emit
  `mitre_techniques: []` — T0814 attribution withheld per DF-VALIDATION-001 / BC-2.16.008
  Invariant 3. See v0.7.0 shipping state vs. current behavior.)
```

---

### ARP-RATE-INTDIV-DOC-001 — LOW — MANUAL

**Location:** src/analyzer/arp.rs:1006 (detect_storm doc-comment, Step 3)

**Description:** The `detect_storm` doc-comment at line 1006 shows the rate formula as `count_in_window / max(1, ts - window_start_ts)` without noting that this is integer division. The implementation at arp.rs:1040 performs `entry.count_in_window / denominator` where both operands are `u64`. Integer truncation means a MAC sending 29 frames in 2 seconds yields rate = 14 (not 14.5), which affects when the threshold fires at non-integer rates. A reader accustomed to languages with default float division could misread the semantics.

**Evidence:**
- src/analyzer/arp.rs:1006: `rate = count_in_window / max(1, ts - window_start_ts).` (no truncation note)
- src/analyzer/arp.rs:1039–1040: `let denominator = elapsed.max(1) as u64; let rate = entry.count_in_window / denominator;` — integer division.

**Proposed fix:** Add "integer division; truncates fractional rates" to the Step 3 formula comment:
```
///   rate = count_in_window / max(1, ts - window_start_ts)  [integer division; truncates fractional rates].
```

---

### DNP3-TUNING-BIDIR-001 — LOW — MANUAL

**Location:** README.md:408–412 (Known Limitations, "DNP3 direct-operate burst threshold" paragraph)

**Description:** The README Known Limitations section provides DNP3 threshold-tuning guidance ("This value was chosen to tolerate routine maintenance while catching commissioning-speed attacks; quiet OT segments may need a lower value (3–5)") but does not state that the `--dnp3-direct-operate-threshold` counter tracks per-flow events. In a unidirectional mirror-tap deployment where only one direction is captured, control-class FCs may be undercounted relative to a full-capture deployment, which could require threshold adjustment. An operator applying the default guidance on a mirror tap may see false negatives.

**Evidence:**
- README.md:408–412: guidance present without bidirectional assumption note.
- src/cli.rs:204–207: `--dnp3-direct-operate-threshold` docstring also does not mention capture directionality.

**Proposed fix (doc polish):** Add a one-sentence note to the README Known Limitations DNP3 paragraph and/or the `--dnp3-direct-operate-threshold` CLI arg doc-comment: "Note: thresholds count per-flow control events across both directions; a unidirectional mirror-tap deployment captures only one direction, which halves the observable FC rate — operators on mirror taps may need to halve the threshold to compensate."

---

### UNIT-FMT-5-20S-001 — LOW — FIXABLE-AUTO

**Location:** src/cli.rs:185, 192

**Description:** The Modbus CLI arg doc-comments use inconsistent unit-format strings for time windows. The write-burst threshold arg uses "1-second" (spelled out), while the write-sustained threshold arg uses ">= 2s" (abbreviated). These are different formats for the same concept (a time-duration window) in adjacent args.

**Evidence:**
- src/cli.rs:185: `write-class FCs are observed within any 1-second window.`
- src/cli.rs:192: `write-FC rate exceeds M writes/second over a contiguous window of >= 2s.`

**Proposed fix:** Standardize to abbreviated format throughout (`1s window`, `>= 2s window`) or spell out throughout. The abbreviated form is consistent with Clap's auto-formatting (e.g., "default: 300") and the README options block. Suggest: change "within any 1-second window" to "within any 1s window" (cli.rs:185).

---

### README-OPTIONS-L117-NEUTRAL-001 — LOW — MANUAL

**Location:** README.md:117 (Analyze flags section)

**Description:** The `--arp-storm-rate` flag description says "ARP storm frames/second per source MAC threshold (default: 50)" without stating that the finding fires *at or above* the threshold (i.e., `>=`). The phrasing "threshold" alone is ambiguous about whether firing is strict `>` or `>=`. The CLI arg doc-comment in cli.rs:241 correctly states "frames/second per source MAC at or above which a MEDIUM/Anomaly storm finding is emitted" with the `>=` semantics explicit.

**Evidence:**
- README.md:117: `--arp-storm-rate N                     ARP storm frames/second per source MAC threshold (default: 50)` — no "at or above" qualifier.
- src/cli.rs:241–244: `D3 storm rate threshold: frames/second per source MAC at or above which a MEDIUM/Anomaly storm finding is emitted. Default: 50 (wirerust engineering default — not derived from any external standard).` — explicit `>=` semantics and calibration note.

**Proposed fix:** Update README.md:117 to match the cli.rs language: "ARP storm rate (frames/second per source MAC at or above which a storm finding is emitted; default: 50; engineering default — not derived from any external standard)." This is a longer description but makes the `>=` semantics clear and matches the Known Limitations section text.

---

## Checks Completed Without Findings

### 1. README — Installation and Usage

- `cargo install --path .` and `cargo build --release` are valid; binary path `target/release/wirerust` is accurate (README.md:36).
- All subcommands documented in README match `src/cli.rs`: `analyze`, `summary`, `protocols` all present.
- Global flags in README Options block (README.md:85–102) match cli.rs global `Cli` struct fields. All default values and ranges match.
- Analyze flags in README.md:106–125 match cli.rs `Commands::Analyze` fields. The `--coverage-gaps` flag added by e3ca2bc is present at README.md:124.
- `protocols` subcommand flags (`--all`, `--supported`, `--unsupported`, global `--json`) match cli.rs:293–302.
- DNP3 JSON counters documented in README.md:232–240 (`dropped_findings`, `master_addrs_dropped`, `pending_requests_evicted`) match src/analyzer/dnp3.rs summarize() output (confirmed via tech-debt register PC-016/017 RESOLVED and PR #370).
- ENIP `enip_summary` 7-key schema documented at README.md:294–296 matches enip.rs:1548–1576 (`command_distribution`, `total_pdu_count`, `parse_errors`, `write_count`, `error_count`, `flows_analyzed`, `dropped_findings`).

### 2. docs/adr/ — Module Lists and Cross-References

- All 12 ADRs (0001–0012) are present. ADR-0008 is correctly marked as a withdrawn placeholder.
- ADR-0001 rule order (8 rules) matches current dispatcher: TLS sig → HTTP sig → port 443/8443 → port 80/8080 → port 502 → port 20000 → port 44818 → None.
- ADR-0001 `DispatchTarget` enum variants (`Http`, `Tls`, `Modbus`, `Dnp3`, `Enip`, `None`) match dispatcher source.
- ADR-0002 analyzer table (DNS, HTTP, TLS, Modbus, DNP3, ARP, EtherNet/IP) matches `src/analyzer/` module listing.
- ADR-0002 Deviations section accurately describes DNP3 and ARP custom interfaces. ENIP deviation is correctly noted (aside from the PC-020/PC-023 ID error — DOC-NEW-001 above).

### 3. CHANGELOG — [Unreleased] vs. commits since v0.12.0

Only one commit since v0.12.0 merge (f1e0c36): b5e1e15 ("docs: LMR-003 template-conformance exemption + check-green-doc-tense guard tests"). This commit modifies `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`, `CHANGELOG.md`, and `docs/demo-evidence/STORY-162/`. The [Unreleased] section accurately describes the changes: `_find_repo_root` helper extraction and five new hermetic tests (STORY-162, wave-73, F-W72G-P2-OBS-001). The CHANGELOG gate AC-158-001 is satisfied since `bin/` was modified.

### 4. CLAUDE.md — File References

All files and directories referenced in CLAUDE.md were verified to exist at their stated paths on the develop branch:
- `docs/adr/` (all 12 files) ✓
- `docs/superpowers/plans/`, `docs/superpowers/specs/` ✓
- `.github/workflows/ci.yml`, `.github/workflows/release.yml` ✓
- `bin/compute-input-hash`, `bin/test_compute_input_hash.py`, `bin/check-green-doc-tense`, `bin/lint-cycle-artifact` ✓
- `.factory/maintenance/demo-evidence-scrub-gate.md`, `.factory/maintenance/pr-manager-merge-auth-guidance.md` — on factory-artifacts branch as documented ✓

### 5. TODO/FIXME/HACK Comments in src/

`grep -rn "TODO|FIXME|HACK" src/` returned empty — no such comments in src/ as of HEAD (b5e1e15).

### 6. Known Open Items Re-Verified

| Debt ID | Status | Notes |
|---------|--------|-------|
| PG-W-README-JSON-SCHEMA | OPEN (re-confirmed) | ARP `arp_summary` key claim still present at README.md:263 |
| ROUTE-B-DEFERRED NEW-002 | **RESOLVED** by e3ca2bc (#393) | README --coverage-gaps flag documented |
| ROUTE-B-DEFERRED NEW-003 | OPEN (re-confirmed) | ADR-0001 struct snippet still missing two fields |
| CHANGELOG-D3-T0830-DRIFT-001 | OPEN (re-confirmed) | CHANGELOG.md:806 still claims T0830 for D3 |
| ARP-RATE-INTDIV-DOC-001 | OPEN (re-confirmed) | No truncation note at arp.rs:1006 |
| DNP3-TUNING-BIDIR-001 | OPEN (re-confirmed) | README Known Limitations still lacks bidirectional note |
| UNIT-FMT-5-20S-001 | OPEN (re-confirmed) | cli.rs:185 "1-second" vs cli.rs:192 "2s" |
| README-OPTIONS-L117-NEUTRAL-001 | OPEN (re-confirmed) | README.md:117 lacks ">= semantics" and calibration note |
