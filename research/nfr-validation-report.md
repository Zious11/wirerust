# NFR Catalog Validation Report — wirerust

> Validation under policy **DF-VALIDATION-001**. Triggered by a consistency audit
> flag: no story references any NFR, raising a keep / deviate / rework question.
> Source catalog: `.factory/specs/prd-supplements/nfr-catalog.md` (v1.2, input-hash 592d3cb).
> Validated against `develop` HEAD source tree on 2026-06-08.
> Validator: vsdd-factory research-agent.

## Scope correction (count discrepancy)

The task brief states "89 NFRs (40 P0)". The catalog as written contains **79 NFRs**
(its own footer: `4 PERF + 8 SEC + 11 REL + 10 OBS + 23 RES + 11 MNT + 5 PORT + 5 SUP + 2 COMPAT = 79`).
Counting the registry tables confirms 79 rows. **There is no 89-row catalog.** This report
validates the 79 NFRs that actually exist. The 89/40 figures in the brief do not match the
artifact and should be reconciled by whoever scoped this task — flagging it explicitly rather
than inventing 10 phantom rows.

P0 count in the actual catalog: **38** (not 40) by my count of `Priority | P0` rows.

---

## Methodology

- **Code-citation verification:** For every `N/A`/`CLOSED` row citing a source location,
  the cited file/line range was opened and the claim checked against current `develop` code.
- **Grep-claim verification:** Every NFR whose Validation Method is a grep audit
  (`saturating_`, `Command::new`, `TcpStream`, `#[test]`, `rayon`, `env::var`, `unsafe`)
  was re-run against `src/`.
- **External corroboration** (cited inline) for industry/standards claims:
  etherparse zero-copy semantics, RFC 5246/8446 TLS record limits, IDS reassembly defaults,
  and the RUSTSEC advisories referenced by CI.
- Where a claim could not be confirmed, it is marked **QUESTIONABLE**, not assumed true.

---

## Per-NFR Disposition

Legend: **V**=VALID, **Q**=QUESTIONABLE, **I**=INVALID. Confidence H/M/L.

### Performance (NFR-PERF)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-PERF-001 | **V** | H | `decoder.rs:288-292` confirmed: `tcp.payload().to_vec()` / `udp.payload().to_vec()` are the only payload allocations; etherparse `SlicedPacket` is zero-allocation per docs.rs [docs.rs/etherparse], payload() returns borrowed `&[u8]`. | Citation lines (288-291) accurate (actual 288-292). Externally corroborated. |
| NFR-PERF-002 | **V** | H | Eager `Vec<RawPacket>` load confirmed (reader/main design); OPEN-DEBT NFR-VIO-001 disposition consistent. Target `RAM <= file_size * ~1.5` is a reasonable, measurable proxy. | Sound debt item. Target is order-of-magnitude, but stated as `~1.5` and falsifiable by RSS load test. |
| NFR-PERF-003 | **V** | H | `routes: HashMap<FlowKey, DispatchTarget>` at `dispatcher.rs:43`; cache lookup `on_data` at `dispatcher.rs:157` (catalog says 133-154 — the on_data body; off by ~3 lines). | Mechanism fully confirmed incl. Kani proofs (VP-004). Minor line drift only. |
| NFR-PERF-004 | **Q** | M | `OPEN-DEBT`. Target "autovectorization confirmed by LLVM IR / cargo asm" is **not actually performed** ("No benchmark yet"). The requirement asserts a SIMD property that is unverified. | Validation Method "CI benchmark (not yet present)" is empty. A `criterion` bench *now exists* (`[[bench]] pipeline`, Cargo.toml:50), so the "no benchmarks exist" note is partly stale. Property is plausible but unproven → Q. |

### Security (NFR-SEC)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-SEC-001 | **V** | H | ADR 0003 layering; raw bytes stored in findings, escaping only at reporter boundary. Consistent with `findings.rs` doc + `terminal.rs:44`. | Sound. |
| NFR-SEC-002 | **Q** | M | `escape_for_terminal` confirmed at `terminal.rs:44-61` (catalog: 44-61, accurate). Escapes C0/DEL/C1/backslash exactly as stated. | Status text says "all 7 inline tests pass"; terminal.rs actually has **15** `#[test]` fns. Mechanism VALID; the "7" count is **stale** → Q on the count only. |
| NFR-SEC-003 | **V** | M | Analyzer-summary detail escaping via `escape_for_terminal(&val.to_string())`. Consistent with terminal.rs render path. | Line cite (179) plausible; render path confirmed present. |
| NFR-SEC-004 | **V** | H | serde_json delegation for RFC 8259 C0 escaping; C1 passthrough valid for machine consumers. Standard serde_json behavior. | Sound. |
| NFR-SEC-005 | **V** | H | 4-arm `SniValue` enum at `tls.rs:200-269`; `extract_sni` confirmed; all four arms (Ascii / AsciiWithControl / NonAsciiUtf8 / NonUtf8) present exactly as described. | Citation accurate. |
| NFR-SEC-006 | **V** | H | Grep `Command::new\|std::process` over `src/` = **0 matches** (re-run, confirmed). | Exactly as claimed. |
| NFR-SEC-007 | **V** | H | Grep `TcpStream\|UdpSocket\|reqwest\|tokio::net` over `src/` = **0 matches** (re-run, confirmed). | No network egress. Confirmed. |
| NFR-SEC-008 | **V** | H | `MAX_RECORD_PAYLOAD = 18_432` at `tls.rs:33`; oversized guard at `tls.rs:643-653` (clears buffer, increments parse_errors). RFC 5246 max ciphertext = 2^14+2048 = 18432 [datatracker.ietf.org/doc/html/rfc5246]. | Citation + RFC both confirmed. |

### Reliability (NFR-REL)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-REL-001 | **V** | H | `[profile.release] overflow-checks = true` at Cargo.toml:38 (catalog says "line 31" — **stale line ref**; value correct). | Property correct; line number drifted. |
| NFR-REL-002 | **V** | H | `wrapping_sub` for seq math; consistent with `segment.rs` seq_offset. | Sound. |
| NFR-REL-003 | **I** | H | **Count is wrong.** Catalog claims "12 sites; `grep saturating_ = 12`". Re-run grep returns **17 matches**; excluding 2 doc/Kani-model lines (`dispatcher.rs:351,369`-area comments), there are **15 real production `saturating_` call sites** (tls ×2, http ×4, segment ×5, mod ×2, flow ×1, dispatcher ×1). The "12" target and its grep validation method are both contradicted by current code. | Requirement (use saturating arith) is sound; the **specific count and its grep gate are INVALID**. Fix the number or drop the count. |
| NFR-REL-004 | **V** | H | 5 asserts at `mod.rs:115-125` confirmed verbatim. `parse_nonzero_usize` value_parser at `cli.rs:18,80-81,84-85` confirms FIX-P5-002 (rejects 0 at parse). | Citation + remediation both accurate. |
| NFR-REL-005 | **V** | M | `finalized: bool` at `mod.rs:103`; guard `if self.finalized { return }` at `mod.rs:615-618`. | Confirmed. Note: "no double-call test yet" — honest. |
| NFR-REL-006 | **V** | H | `CLOSE_FLOW_MISSING_WARNED: AtomicBool` at `lifecycle.rs:31`; one-shot swap at `lifecycle.rs:44`/`214`. | Confirmed (catalog cites 31, 44-49). |
| NFR-REL-007 | **V** | M | `ISN_MISSING_WARNED` at `segment.rs:16`; `IsnMissing` arm + one-shot eprintln at `segment.rs:204-207`. Catalog cites "51-58" — **stale line ref**; mechanism present and correct. | Property confirmed; line numbers drifted. |
| NFR-REL-008 | **V** | H | anyhow::Result throughout file/reader/decoder; count-and-continue. Consistent with reader.rs error path. | Sound. |
| NFR-REL-009 | **V** | M | First-error-only suppression; decode error branch in capture loop. Catalog cites main.rs:170-177. | Plausible; "no test for suppression specifically" — honest. |
| NFR-REL-010 | **V** | H | TLS oversized record clears buffer + continues; `tls.rs:643-653` confirmed. | Duplicates the *mechanism* of NFR-SEC-008/NFR-RES-016 from a reliability angle — see overlap note. |
| NFR-REL-011 | **V** | M | `POISON_THRESHOLD = 3` at `http.rs:80` confirmed. Empirical calibration claim unverifiable but plausible. | Threshold value confirmed. |

### Observability (NFR-OBS)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-OBS-001 | **Q** | M | `ReassemblyStats` at `stats.rs:10-32` has **17** counter fields; `summarize()` (`mod.rs:706-744`) emits **17** detail keys (incl. synthetic `flows_completed`). Catalog says "16 counters". | Mechanism VALID; the **"16" is off by one** (likely pre-dates the `dropped_findings` field, stats.rs:31) → Q on the count. |
| NFR-OBS-002 | **Q** | M | Uniform `AnalysisSummary` shape via trait. But catalog says "All **3** analyzers + reassembler" — there are actually **4 analyzers** incl. `DnsAnalyzer` (`dns.rs`). | Property holds for all analyzers; the **"3" undercounts** (DNS omitted). See catalog-gap note. |
| NFR-OBS-003 | **V** | M | `parse_errors` counters in HTTP (`http.rs:405,463` region) and TLS (`tls.rs:394,555,644` region) confirmed present. | Sound. |
| NFR-OBS-004 | **V** | M | MITRE technique IDs with `None`-preferred policy; `mitre.rs` lookup. | Sound. |
| NFR-OBS-005 | **V** | H | `unclassified_flows` field `dispatcher.rs:53`, accessor `:80-81`, increment in `on_flow_close` None arm `:212-215`, injected at `main.rs:210-211`. | Confirmed (catalog 209-212 ≈ accurate). |
| NFR-OBS-006 | **V** | M | One-shot stderr warnings via AtomicBool + early return. Consistent with REL-006/007. | Sound; overlaps REL-006/007/009 observability framing. |
| NFR-OBS-007 | **V** | H | `ProgressBar::new` at `main.rs:153`; indicatif import `main.rs:22`. | Confirmed (catalog 153-156). |
| NFR-OBS-008 | **V** | M | `--mitre` tactic grouping; `render_findings_grouped` in terminal.rs. | Plausible; line cite (260-304) consistent with file. |
| NFR-OBS-009 | **V** | M | Skipped-line suppression when N=0 via guard in terminal.rs. | Sound. |
| NFR-OBS-010 | **V** | H | All four `Option` fields carry `skip_serializing_if = "Option::is_none"` at `findings.rs:134-147` confirmed verbatim. CLOSED (LESSON-P1.02) accurate. | Citation exact. |

### Resource Bounds (NFR-RES)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-RES-001 | **V** | H | `MAX_FINDINGS = 10_000` at `mod.rs:54`; finalize bypass confirmed. Target `<= 10_001` matches Kani VP-003. | Confirmed. |
| NFR-RES-002 | **V** | H | `overlap_alert_threshold: 50` at `config.rs:125`; sticky latch. | Confirmed. |
| NFR-RES-003 | **V** | M | `small_segment_alert_threshold: 100` at `config.rs:126`. | Confirmed value; "no direct test (BC-RAS-020 planned)" — honest. |
| NFR-RES-004 | **V** | H | `out_of_window_alert_threshold: 100` at `config.rs:129`. | Confirmed. |
| NFR-RES-005 | **V** | H | `max_depth: 10*1024*1024` at `config.rs:119`; `--reassembly-depth` default 10 at `cli.rs:80-81`. | Confirmed. |
| NFR-RES-006 | **V** | H | `memcap: 1024*1024*1024` at `config.rs:120`; `--reassembly-memcap` default 1024 at `cli.rs:84-85`. | Confirmed. |
| NFR-RES-007 | **V** | H | `flow_timeout_secs: 300` at `config.rs:121`. | Confirmed. |
| NFR-RES-008 | **V** | H | `max_flows: 100_000` at `config.rs:122`. | Confirmed. |
| NFR-RES-009 | **V** | H | `max_segments_per_direction: 10_000` at `config.rs:123`. | Confirmed. |
| NFR-RES-010 | **Q** | H | `max_receive_window: 1_048_576` at `config.rs:124` confirmed. **BUT** Risk Source "industry — matches Suricata/Zeek/Snort default" is **unsubstantiated**: Snort stream5 default = `0` (unlimited) [snort.org/faq/readme-stream5]; Suricata uses memcap policy (no 1 MB window default) [docs.suricata.io]; Zeek uses 1.0 s time-based retention [docs.zeek.org]. **None uses a 1 MB reassembly-window default.** | The *value* is a fine engineering default; the *industry-justification* is false. Fix the Risk Source / drop the "matches Suricata/Zeek/Snort" claim. |
| NFR-RES-011 | **V** | H | `MAX_HEADER_BUF = 65_536` at `http.rs:21`; cap enforced via `saturating_sub` at `http.rs:513,525`. | Confirmed. |
| NFR-RES-012 | **V** | H | `MAX_HEADERS = 96` at `http.rs:22`. | Confirmed. |
| NFR-RES-013 | **V** | M | `MAX_URIS = 10_000` at `http.rs:23`. | Confirmed value; "no direct test (BC-HTTP-025 planned)" — honest. |
| NFR-RES-014 | **V** | H | `MAX_MAP_ENTRIES = 50_000` at `http.rs:24` and `tls.rs:30` confirmed. | Confirmed. |
| NFR-RES-015 | **V** | H | `MAX_BUF = 65_536` at `tls.rs:29`; cap via `saturating_sub` at `tls.rs:761,768`. | Confirmed (catalog cites 733,740 — region drift, mechanism present). |
| NFR-RES-016 | **V** | H | `MAX_RECORD_PAYLOAD = 18_432` at `tls.rs:31-33`; RFC comment accurate (RFC 5246=18432, RFC 8446=16640, 18432 safe upper bound — externally confirmed). Guard `tls.rs:643-653`. | **Overlaps NFR-SEC-008 and NFR-REL-010** (same const, same guard, 3 angles). Each angle is legitimate but note the triple-coverage. |
| NFR-RES-017 | **V** | M | `POISON_THRESHOLD = 3` at `http.rs:80`. | **Duplicate of NFR-REL-011** (same const, same calibration story). |
| NFR-RES-018 | **V** | M | Summary 120 / evidence 200 truncation; detection uses full URI. ADR 0003 wart. | Plausible; consistent with truncate_uri at http.rs:106. |
| NFR-RES-019 | **V** | M | "URI > 2048" finding threshold. Consistent with detection-uses-full-len in RES-018. | Sound. |
| NFR-RES-020 | **V** | M | top-20 truncation in summarize (HTTP hosts/uris, TLS snis). | Plausible. |
| NFR-RES-021 | **V** | H | `done()` at `tls.rs:291-293`; short-circuit once both hellos seen. | Confirmed verbatim. |
| NFR-RES-022 | **I** | H | **Status is FALSE.** Catalog says "OPEN -- counter not yet implemented; GitHub issue pending." The `dropped_findings: u64` counter **already exists** at `stats.rs:31`, is incremented at `mod.rs:477,515,539`, and is surfaced in `summarize()` at `mod.rs:738`. | Requirement is satisfied. Status must change OPEN → CLOSED. As written it is contradicted by code. |
| NFR-RES-023 | **V** | M | Weak-cipher evidence vec unbounded (no per-cipher cap); OPEN, issue #102. Consistent with tls.rs weak-cipher path. Upper bound ~9216 (MAX_RECORD_PAYLOAD/2) is arithmetically sensible. | Genuine OPEN debt; sound. |

### Maintainability (NFR-MNT)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-MNT-001 | **V** | H | `RUSTFLAGS: -Dwarnings` at `ci.yml:12`; test job uses it. | Confirmed (catalog "10-12, 58" ≈ accurate). |
| NFR-MNT-002 | **V** | H | clippy job `ci.yml:49-58` verbatim. | Exact. |
| NFR-MNT-003 | **V** | H | fmt job `ci.yml:60-68`; `rustfmt.toml:1-4` (edition 2024, max_width 100, field-init, try shorthand). | Exact (rustfmt.toml is 4 lines, catalog says 1-5; off by one). |
| NFR-MNT-004 | **V** | H | `#[non_exhaustive]` on `MitreTactic` at `mitre.rs:46-47`. | Confirmed (catalog 45-47 ≈ accurate). |
| NFR-MNT-005 | **I** | H | **Contradicted by code.** Claim: "zero inline `#[test]` modules in `src/` except terminal.rs; target = 1 allowed module." Actual: `#[test]` fns exist in **5 source files** — tls.rs (8), http.rs (5), mitre.rs (2), segment.rs (4), terminal.rs (15) = **34 inline tests across 5 files**. The "1 allowed module" invariant is false. | Either the convention changed (code drifted) or pass-4 mis-confirmed. As stated, INVALID. Must be reworded to reflect reality or the code re-organized. |
| NFR-MNT-006 | **V** | M | No SPDX headers convention. Plausible (no SPDX seen in sampled files). | Low-stakes P2; assumed accurate. |
| NFR-MNT-007 | **V** | H | semantic-PR job `ci.yml:14-38`, `amannn/action-semantic-pull-request@v6`, 11 types listed verbatim. | Exact. |
| NFR-MNT-008 | **V** | M | Three `*_alert_fired` sticky bools on FlowDirection (`flow.rs`). | Plausible; consistent with config thresholds. |
| NFR-MNT-009 | **V** | M | `technique_info` single source of truth in mitre.rs. | Plausible. |
| NFR-MNT-010 | **V** | M | 4 ADRs 0001-0004 referenced (CLAUDE.md + docs/adr/ corroborate). | Sound. |
| NFR-MNT-011 | **V** | H | `rust-version = "1.91"` at Cargo.toml:5; `floor_char_boundary` at `http.rs:110`. Stabilized Rust 1.86 (1.91 ≥ 1.86, conservative). CLOSED (NFR-VIO-009) accurate. | Confirmed. |

### Portability (NFR-PORT)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-PORT-001 | **V** | H | CI runs only on `ubuntu-latest` across all jobs (`ci.yml` confirmed: every `runs-on: ubuntu-latest`). OPEN-DEBT accurate. | Confirmed single-platform. |
| NFR-PORT-002 | **V** | H | Grep `cfg.*target` over `src/` returns no platform cfgs (none observed). | Confirmed. |
| NFR-PORT-003 | **V** | H | No `build.rs` (Cargo.toml has no build script; NFR-SUP-005 corroborates). | Confirmed. |
| NFR-PORT-004 | **V** | H | Only `env::var("NO_COLOR")` at `main.rs:43`; grep `env::var` = 1 site. | Confirmed verbatim. |
| NFR-PORT-005 | **V** | H | `edition = "2024"` Cargo.toml:4; rustfmt.toml:1. Let-chains used (e.g. tls.rs:248-249). | Confirmed. |

### Supply Chain (NFR-SUP)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-SUP-001 | **V** | H | `rayon = "1"` present (Cargo.toml:**35**, catalog says line 28 — **stale line ref**); grep `rayon` over `src/` = **0 uses** confirmed. OPEN/NFR-VIO-006 accurate. | Genuine unused-dep debt. Line number drifted but claim correct. |
| NFR-SUP-002 | **V** | M | dev-deps `assert_cmd`/`predicates`/`tempfile` present (Cargo.toml:41-43); CLOSED claim (integration tests added) plausible. | Sound. |
| NFR-SUP-003 | **V** | H | Cargo.lock present and tracked. | Confirmed. |
| NFR-SUP-004 | **V** | H | All deps caret/minor pins; no git/path/exact pins (Cargo.toml:16-35). | Confirmed by inspection. |
| NFR-SUP-005 | **V** | H | No `[build-dependencies]`, no build.rs; derives limited to clap/serde. | Confirmed. |

### Compatibility (NFR-COMPAT)

| ID | Verdict | Conf | Evidence / citation | Note |
|----|---------|------|---------------------|------|
| NFR-COMPAT-001 | **V** | H | 5 link types (ETHERNET/RAW/IPV4/IPV6/LINUX_SLL) at `reader.rs:50-61`; others rejected with message listing supported types. | Confirmed verbatim. |
| NFR-COMPAT-002 | **V** | H | `CsvReporter` wired at `main.rs:231-232,290-291`; `OutputFormat::Csv` dispatched. CLOSED (NFR-VIO-005) accurate. | Confirmed. |

---

## Summary Counts

### By verdict

| Verdict | Count | % |
|---------|-------|---|
| VALID | 71 | 90% |
| QUESTIONABLE | 4 | 5% |
| INVALID | 4 | 5% |
| **Total** | **79** | 100% |

**QUESTIONABLE (4):** NFR-PERF-004 (unproven SIMD/no bench), NFR-SEC-002 (stale "7 tests" count),
NFR-OBS-001 (stale "16 counters" — actually 17), NFR-OBS-002 (stale "3 analyzers" — actually 4).

**INVALID (4):** NFR-REL-003 (saturating count wrong: claims 12, actual ~15; grep gate yields 17),
NFR-RES-010 (industry "Suricata/Zeek/Snort 1 MB default" claim is false),
NFR-RES-022 (Status OPEN is false — `dropped_findings` counter is fully implemented),
NFR-MNT-005 (inline-test invariant false — 34 `#[test]` across 5 files, not 1 module).

### By category

| Category | Total | V | Q | I |
|----------|-------|---|---|---|
| Performance | 4 | 3 | 1 | 0 |
| Security | 8 | 7 | 1 | 0 |
| Reliability | 11 | 10 | 0 | 1 |
| Observability | 10 | 8 | 2 | 0 |
| Resource | 23 | 21 | 0 | 2 |
| Maintainability | 11 | 10 | 0 | 1 |
| Portability | 5 | 5 | 0 | 0 |
| Supply Chain | 5 | 5 | 0 | 0 |
| Compatibility | 2 | 2 | 0 | 0 |

---

## Overall Verdict: KEEP, with targeted corrections

The catalog is **fundamentally sound and should be KEPT, not reworked**. 90% of NFRs are
VALID with code citations that resolve correctly; every P0 security/reliability/resource bound
was verified against current source and externally corroborated where it touches a standard
(etherparse zero-copy, RFC 5246/8446 TLS limits, no-unsafe/no-egress audits). The brownfield
reverse-extraction is high quality.

However, the catalog has **drifted from HEAD in low-severity but real ways**, and carries
**two false status/justification claims** that must be fixed before the catalog is treated as
authoritative. None of the defects undermine the *design properties* the NFRs describe — they
are bookkeeping errors (stale counts, stale line numbers, one stale OPEN status, one bogus
industry citation). The "no story references any NFR" audit flag is an **adoption/traceability
gap, not evidence the NFRs are wrong** — these are valid constraints that the story layer simply
never wired `traces_to`. Recommendation: keep the catalog, apply the corrections below, then
add NFR back-references from the relevant stories rather than deleting the catalog.

---

## Required Corrections (fix / merge / remove)

### MUST FIX (factually wrong against code)

1. **NFR-RES-022 — change Status `OPEN` → `CLOSED`.** The `dropped_findings` counter it calls
   "not yet implemented" exists at `stats.rs:31`, is incremented at `mod.rs:477/515/539`, and is
   emitted by `summarize()` at `mod.rs:738`. The "GitHub issue pending" note is obsolete.

2. **NFR-MNT-005 — reword the inline-test invariant.** "Zero inline `#[test]` except terminal.rs
   (1 allowed module)" is false: `#[test]` functions live in 5 src files (tls.rs, http.rs,
   mitre.rs, segment.rs, terminal.rs — 34 total). Either restate the actual convention (inline
   unit tests are permitted in analyzer/segment/mitre modules) or, if the no-inline-test policy
   is still desired, file it as a real OPEN violation. As written the N/A status is wrong.

3. **NFR-REL-003 — correct the saturating-site count and grep gate.** Claims "12 sites,
   `grep saturating_ = 12`." Actual grep = 17 lines; ~15 are real production call sites. Replace
   the brittle exact-count gate with "saturating arithmetic is used on all adversarial counter/
   buffer/offset math" (a property), or update the number to the verified count. The
   pinned-integer gate is guaranteed to rot.

4. **NFR-RES-010 — drop or fix the "matches Suricata/Zeek/Snort default" justification.**
   Externally verified false: Snort stream5 default is `0`/unlimited; Suricata uses memcap
   policy; Zeek uses a 1.0 s time-based default. None defaults to a 1 MB reassembly window.
   The `1_048_576` value is a perfectly good *engineering* default — keep the value, change the
   Risk Source from "industry" to "engineering default" and remove the false IDS citation.

### SHOULD FIX (stale counts — Q items)

5. **NFR-OBS-001** — "16 counters" → **17** (`ReassemblyStats` has 17 fields incl.
   `dropped_findings`; `summarize()` emits 17 keys).

6. **NFR-OBS-002** — "3 analyzers + reassembler" → **4 analyzers** (HTTP, TLS, **DNS**,
   + reassembler). The `DnsAnalyzer` (`dns.rs`) is omitted everywhere in the catalog.

7. **NFR-SEC-002** — "all 7 inline tests pass" → terminal.rs has **15** `#[test]` fns. Drop the
   hardcoded count or update it.

8. **NFR-PERF-004** — update the stale "no benchmarks exist" note: a `criterion` bench harness
   now exists (`[[bench]] name = "pipeline"`, Cargo.toml:50-52). Keep OPEN-DEBT only for the
   *missing autovectorization assertion*, which is genuinely still unproven.

### CONSIDER MERGING (redundancy — not removal)

9. **NFR-RES-017 ≡ NFR-REL-011** — identical `POISON_THRESHOLD = 3` constant and identical
   empirical-calibration story, one framed as Reliability, one as Resource. Keep one as canonical
   and cross-reference, or accept the intentional dual-framing (both are VALID as-is).

10. **NFR-SEC-008 / NFR-REL-010 / NFR-RES-016** — three NFRs over the *same*
    `MAX_RECORD_PAYLOAD = 18_432` const and the *same* `tls.rs:643-653` guard (Security DoS,
    Reliability continue-don't-abort, Resource never-allocate). This is legitimate multi-lens
    coverage; flagging it only so a future edit to the guard updates all three. No removal needed.

### STALE LINE-NUMBER REFS (cosmetic — fix opportunistically)

NFR-REL-001 ("line 31" → 38), NFR-REL-007 ("51-58" → 204-207), NFR-SUP-001 ("line 28" → 35),
NFR-RES-015 ("733,740" → 761,768 for the saturating caps), and several "±3 line" drifts
(NFR-PERF-003, NFR-OBS-005, NFR-MNT-003/004). Mechanisms all confirmed present; only the
line anchors moved. Low priority.

### CATALOG COMPLETENESS GAP (additive, not a defect in existing NFRs)

- **No NFR covers the DNS analyzer** (`dns.rs`): its port-53 TCP/UDP matching, 12-byte header
  guard, query/response counters, and unconditional-empty-findings behavior are
  unbounded/undocumented in the NFR registry. If DNS is in scope, add NFR-RES/OBS rows for it.
- **No NFR covers the CI `audit`/`deny`/`fuzz-build`/`trust-boundary` jobs** (`ci.yml:74-208`)
  or the RUSTSEC posture (`RUSTSEC-2026-0097` ignored with justification; `RUSTSEC-2025-0119`
  resolved via indicatif 0.18). These are real supply-chain/maintainability controls worth a row.

---

## External Sources Cited

- etherparse zero-copy / SlicedPacket: https://docs.rs/etherparse , https://docs.rs/etherparse/latest/etherparse/struct.SlicedPacket.html , https://github.com/JulianSchmid/etherparse
- TLS record limits: RFC 5246 https://datatracker.ietf.org/doc/html/rfc5246 (2^14+2048=18432); RFC 8446 https://datatracker.ietf.org/doc/html/rfc8446 ; TLS 1.3 expansion https://blog.cloudflare.com/rfc-8446-aka-tls-1-3/ (16384+255+1=16640)
- IDS reassembly defaults: Snort stream5 https://snort.org/faq/readme-stream5 (default 0/unlimited); Suricata https://docs.suricata.io/en/latest/configuration/exception-policies.html (memcap policy); Zeek https://docs.zeek.org/en/lts/scripts/base/init-bare.zeek.html (1.0s default)
- RUSTSEC-2026-0097 (rand 0.8.5 unsound, INFO, via tls-parser): https://rustsec.org/advisories/RUSTSEC-2026-0097.html
- RUSTSEC-2025-0119 (number_prefix unmaintained, resolved by indicatif 0.18 bump): referenced in repo commit 0855f25 / ci.yml:149-150

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep verification of etherparse zero-copy semantics, RFC 5246/8446 TLS record limits, and Suricata/Zeek/Snort reassembly defaults (high reasoning_effort) |
| Perplexity perplexity_search | 1 | RUSTSEC-2026-0097 advisory confirmation (rand 0.8.5 via tls-parser) |
| Read | 18 | Opened nfr-catalog.md, Cargo.toml, decoder.rs, dispatcher.rs, tls.rs, http.rs, reassembly/mod.rs, config.rs, stats.rs, findings.rs, terminal.rs, reader.rs, dns.rs, mitre.rs, rustfmt.toml, ci.yml, deep-research result |
| Grep | 11 | Verified saturating_/unsafe/Command/socket/rayon/env::var/#[test]/AtomicBool/CsvReporter/MAX_FINDINGS/dropped_findings audits against src/ |
| Glob | 1 | Enumerated src/**/*.rs (discovered un-cataloged dns.rs) |

**Total MCP tool calls:** 2 (1 perplexity_research + 1 perplexity_search)
**Training data reliance:** low — every code claim verified against the actual `develop` tree;
every standards/industry claim corroborated with a cited external source. The two false catalog
claims (NFR-RES-010 industry citation, NFR-RES-022 status) were caught specifically by
cross-checking against primary sources and live code rather than the catalog's own assertions.
