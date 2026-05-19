# Pass 4: NFR Catalog -- wirerust (Deepening Round 2)

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 4 (NFR Catalog) -- Deepening round 2
- **Inputs re-read:** P4 R1 (full, 394 lines), P3 R3 (BC-RAS-022 per-direction correction + MAX_FINDINGS counter design), P3 R4 (BC-FND-006 asymmetric Option serialization), P2 R3 (ClientHello weak-cipher Vec<String> cardinality bound), P6 sec.7 P4 deepening plan (lines 389-400), `Cargo.toml`, `.github/workflows/ci.yml`, `src/findings.rs` (full), `src/reassembly/mod.rs` (lines 50-105, 260-330 alert sites), `src/analyzer/tls.rs` (lines 51-56 cipher_name, 430-495 weak-cipher emission), `src/reader.rs` (RawPacket.timestamp_secs), all 12 saturating_* sites under `src/`.
- **Provenance method:** `git -C ... blame -L` on every threshold constant; `git -C ... log --format='%H %s%n%b' -n 1 <sha>` to capture commit-message rationale.
- **Confidence (this round):** HIGH for provenance findings (git blame is ground truth); HIGH for NFR-VIO dispositions (each grounded in a P4 R1 row); HIGH for cross-pollination deltas (each pinned to a specific R3/R4 line); MEDIUM for the recommended additions (NFR-RES-NEW dropped_findings counter and NFR-OBS-NEW JSON schema symmetry) which are forward-looking proposals.

---

## 1. Hallucination-Class Audit of P4 R1

Per the orchestrator directive, re-verify two specific claims in P4 R1.

### 1.1 "76 NFRs" claim -- P3 R3 flagged this for re-audit

- `awk '/^\| NFR-/ {print $2}' wirerust-pass-4-nfr-catalog.md | grep -v '\.\.' | sort -u | wc -l` => **86 distinct NFR-* IDs total**.
- Excluding the 10 NFR-VIO-* entries (which are violations, not NFRs themselves): **76 discrete NFR IDs.** Breakdown: 4 NFR-PERF + 8 NFR-SEC + 11 NFR-REL + 9 NFR-OBS + 21 NFR-RES + 11 NFR-MNT + 5 NFR-PORT + 5 NFR-SUP + 2 NFR-COMPAT = **76.** Matches the manifest header on P4 R1 line 8.
- Range-style table headers like `NFR-MNT-001..003`, `NFR-OBS-001..009`, `NFR-RES-002..004`, `NFR-RES-005..010`, `NFR-RES-011..018`, `NFR-SUP-001..005` appear in sec.6 (the explicit-vs-implicit comparison) as grouping shorthand, NOT as new IDs.
- **Verdict:** The "76 NFRs" claim is **ACCURATE.** Pass 6's reference is correct. Class-1 (fabricated count) hallucination = **none found.**

### 1.2 "28 magic numbers" claim -- second re-verify target

- Per `awk '/^## 3\./,/^## 4\./' | grep -c '^| \``: 35 raw table rows match. However, P4 R1's footnote at line 203 says "28 distinct numeric / set constants; several constants appear in more than one row (e.g. MAX_MAP_ENTRIES shared by HTTP & TLS) but are counted once." Distinct constants by name (de-duplicating `MAX_MAP_ENTRIES` HTTP+TLS, `Cli.reassembly_depth` duplicating `ReassemblyConfig::default().max_depth`, `Cli.reassembly_memcap` duplicating memcap default): the de-duplicated set comes to **roughly 31, not 28**. Class-1 mild count drift (under-count by ~3).
- **Verdict:** **MINOR DRIFT.** P4 R1's "28 distinct" footnote is slightly low; the true count is approximately 31 once you de-duplicate cross-NFR shared constants. Not load-bearing for any spec decision; flagged here for the next reader.

### 1.3 "13 saturating arithmetic sites" claim (NFR-REL-003)

- `grep -rn 'saturating_' src/ | wc -l` => **12 sites**, not 13. (Sites enumerated below in sec.6.)
- **Verdict:** **MINOR DRIFT.** P4 R1's NFR-REL-003 says 13; ground truth is 12. Class-1 under-by-one count drift. The catalog enumeration in sec.6 below is authoritative.

### 1.4 Other classes

- Class-2 fabricated NFR IDs: spot-checked NFR-RES-008, NFR-REL-011, NFR-SEC-008, NFR-MNT-011 -- all map to real file:line evidence. **None found.**
- Class-3 invented commit / ADR refs: ADR 0001/0002/0003 all exist at `docs/adr/`. **None found.**
- Class-4 same-basename conflation: `MAX_MAP_ENTRIES` appears in BOTH `src/analyzer/http.rs:11` AND `src/analyzer/tls.rs:15` (both 50_000) -- P4 R1 correctly cites both. **None found.**
- Class-5 stale numeric value: re-checked `MAX_FINDINGS=10_000` at mod.rs:18, `POISON_THRESHOLD=3` at http.rs:67, `MAX_RECORD_PAYLOAD=18_432` at tls.rs:18, `max_receive_window=1_048_576` at mod.rs:48, `max_depth=10*1024*1024` at mod.rs:43 -- all match current source. **None found.**

**Net hallucination audit:** Two minor count drifts (28->31 distinct constants; 13->12 saturating sites). No fabricated IDs, no fabricated provenance, no class-3/4/5 errors.

---

## 2. Target 1 -- Provenance Research on 4 Alert Thresholds

Goal: find the PR / commit that introduced each threshold and capture the author's stated rationale. Method: `git -C /Users/zious/Documents/GITHUB/wirerust blame -L <lines> <file>` then `git -C ... log --format='%H %s%n%b' -n 1 <sha>`.

### 2.1 Per-constant provenance table

| Constant | Value | File:line | Introducing commit | PR | Commit message rationale | Verdict on "inferred" label |
|---|---|---|---|---|---|---|
| `OVERLAP_ALERT_THRESHOLD` | 50 | `src/reassembly/mod.rs:15` | `7beaca6` "feat: add TCP stream reassembly engine (#10)" 2026-04-06 | #10 | Bullet in commit body: "Excessive overlaps >50 (evasion attempt finding)". No further numeric justification. | **PARTIAL.** Threshold is stated as policy (">50 == evasion attempt") but the value itself (why 50, not 10 or 200) is not justified in the commit. Provenance does not change the "inferred" label. |
| `SMALL_SEGMENT_ALERT_THRESHOLD` | 2048 | `src/reassembly/mod.rs:16` | `7beaca6` "feat: add TCP stream reassembly engine (#10)" 2026-04-06 | #10 | Bullet: "Small segment floods >2048 (evasion attempt finding)". No further numeric justification. | **PARTIAL.** Same as above -- threshold framing present, value not justified. "Inferred" stays. |
| `OUT_OF_WINDOW_ALERT_THRESHOLD` | 100 | `src/reassembly/mod.rs:17` | `199ed0e` "feat: add threshold-based alert for out-of-window segments (#47)" 2026-04-07 | #47 (fixes #32) | Body: "Generate Anomaly Finding (Inconclusive/Low) when count > 100 per direction". No further justification. **Per-direction is explicitly stated.** | **PARTIAL.** Value not justified; ALSO confirms (independently of P3 R3) that the latch is per-direction. |
| `MAX_FINDINGS` | 10_000 | `src/reassembly/mod.rs:18` | `7beaca6` (#10) 2026-04-06 | #10 | Not enumerated in commit body bullets. ADR 0002 (committed later) is the documented source. | ADR 0002 cited in P4 R1 stands. |
| `POISON_THRESHOLD` | 3 | `src/analyzer/http.rs:67` | `4282188` "perf: poison non-HTTP flows to avoid repeated parse-fail-clear cycles (#42)" 2026-04-07 | #42 (fixes #18) | Body: "Use POISON_THRESHOLD (3 consecutive errors) to tolerate mid-stream joins where initial segments are body data... Reduces parse_errors from 14 to 3 on http-full.cap fixture (the 3 remaining are legitimate first-attempt failures before threshold is reached)". **First numerically-justified threshold in the codebase** -- empirically calibrated against a real fixture. | **DOWNGRADE** "inferred" -> "**empirical, fixture-calibrated**". This is the only threshold backed by data. |

### 2.2 Calibration recommendation for the 3 still-unjustified thresholds

P3 R3 left "calibration against benign traffic" as the open recommendation. Given that POISON_THRESHOLD=3 was calibrated empirically against `tests/fixtures/http-full.cap` and the technique worked (reduced false-positive parse errors from 14 to 3), the same approach should apply to OVERLAP/SMALL_SEG/OUT_OF_WIN. Concrete recipe:

1. Collect a corpus of 5-10 known-benign captures (e.g. a public CTU-13 normal-day capture, a CICIDS-2017 Monday, etc.).
2. Run wirerust with `--reassemble` and instrument (or temporarily add a `debug_assert!`) to print the max per-direction `overlap_count`, `small_segment_count`, `out_of_window_count` reached per flow.
3. The 99th-percentile values across the corpus form the empirical floor; thresholds should sit at 2-3x that floor to avoid alert-storm on legitimate traffic.
4. Add the chosen value + commit-message bullet citing the calibration capture.

**Without this calibration**, the three thresholds remain documented as "inferred / round number" with no defensible operational baseline. This is acceptable for a triage tool but worth surfacing to anyone tuning wirerust as a long-running detector.

---

## 3. Target 2 -- `rust-version = "1.86"` in Cargo.toml

`Cargo.toml` does **not** declare `rust-version` (re-verified by reading the manifest). NFR-VIO-009 in P4 R1 is correct.

**Recommendation (firm):** add to `Cargo.toml`:

```toml
[package]
name = "wirerust"
version = "0.1.0"
edition = "2024"
rust-version = "1.86"   # required by `floor_char_boundary` (stabilized 2025-04-03); see src/analyzer/http.rs:97
description = "Fast PCAP forensics and network triage CLI tool"
license = "MIT"
```

Cost: **S** (single-line addition + matching CI matrix dimension if MSRV testing is desired).

Rationale -- "document if not adding": The argument against is "we ship a binary and have no SemVer obligation to downstream library consumers." That is true but irrelevant; the value of `rust-version` here is **user-facing error quality**. A user with rustc 1.85 today gets a confusing compile error pointing at `floor_char_boundary` (a stdlib function they didn't write); with `rust-version = "1.86"` declared, cargo emits a clear MSRV-mismatch diagnostic at manifest-load time. There is no downside.

**Add this.** Disposition = **fix.**

---

## 4. Target 3 -- NFR Violation Disposition Audit (10 NFR-VIOs)

Each NFR-VIO from P4 R1 sec.7 gets: disposition (fix / document-and-accept), cost (S / M / L), and rationale.

| ID | Violation | Disposition | Cost | Rationale |
|---|---|---|---|---|
| **NFR-VIO-001** | "Multi-GB captures" README claim vs. eager `Vec<RawPacket>` load. | **document-and-accept** | S | The eager load IS the design (single-pass, simple, bounded by pcap file size which is itself bounded). Real fix would require streaming refactor across reader/decoder/dispatcher (L cost). Acceptable mitigation: README amendment to clarify "tens-of-GB on a machine with N GB RAM" rather than claiming streaming. |
| **NFR-VIO-002** | `resolve_targets` globs `*.pcapng` but reader rejects pcapng. | **fix** | S | Two lines in `src/main.rs:245-247`. Either remove `pcapng` from the glob OR make the per-file error visible with a "pcapng not yet supported" message instead of falling through to the generic first-error-only-print path. Recommend the former (less surprising). |
| **NFR-VIO-003** | 7 unwired CLI flags (`--threats`, `--beacon`, `--filter`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>`). | **fix (mixed)** | M | Subdivide: (a) `--filter`, `--verbose`, `--hosts`, `--services`, `--threats`, `--beacon` should be deferred behind `clap`'s `.hide(true)` until implemented OR removed from `Cli` entirely; (b) `--json <FILE>` and `--csv <FILE>` -- see NFR-VIO-004. The CLI surface advertising unimplemented behavior is worse than an honest narrower CLI. |
| **NFR-VIO-004** | `--json/--csv <FILE>` writes to stdout, not the file. | **fix** | S | One match arm and one `std::fs::write` call. The schema (`Option<Option<PathBuf>>`) already supports "with optional destination"; wire it. |
| **NFR-VIO-005** | `OutputFormat::Csv` declared but unwired; `--output-format csv` falls through to terminal. | **fix** | M | Either (a) build the `CsvReporter` -- cost M -- or (b) remove the enum variant and the `csv` crate dep (NFR-VIO-006 then is partially resolved). Choice depends on roadmap; the current state where `--output-format csv` silently downgrades to terminal is a user-trap. |
| **NFR-VIO-006** | `rayon` declared, zero uses. | **fix** | S | Remove from `Cargo.toml` (single-line). The roadmap mention is documentation, not code-relevant. Reintroduce when actually wiring parallelism. |
| **NFR-VIO-007** | `assert_cmd`, `predicates`, `tempfile` dev-deps, zero uses. | **document-and-accept** OR **fix** | S | Two choices: (a) write the first `assert_cmd`-based end-to-end binary test (recommended -- there is currently zero E2E coverage of the spawned binary path), or (b) remove the deps. Both are S. Recommend (a). |
| **NFR-VIO-008** | `serde_json::to_string_pretty(&output).unwrap()` paper-cut. | **document-and-accept** | S | Convert `.unwrap()` -> `.expect("Finding serialization is infallible by construction")` to make the invariant a comment. Bigger refactor (Reporter::render -> Result<String>) is not justified for an infallible-by-construction path. |
| **NFR-VIO-009** | Effective MSRV 1.86 undeclared. | **fix** | S | See sec.3 above. Single-line addition to Cargo.toml. |
| **NFR-VIO-010** | CI only on `ubuntu-latest`. | **document-and-accept** with caveat | S/M | Adding a matrix dimension for `macos-latest` and `windows-latest` (cost S in CI YAML, M in real CI minute-spend) catches platform regressions in `etherparse`/`pcap-file`. Recommended **if** the project's install path is `cargo install --path .` for non-Linux users (which the README implies). Until then, document the Linux-only stance in the README. |

### 4.1 Summary

- **Fix (firm recommendation):** NFR-VIO-002, NFR-VIO-004, NFR-VIO-006, NFR-VIO-009. All cost S, all single-PR-sized.
- **Fix (effort-dependent):** NFR-VIO-003 (M), NFR-VIO-005 (M).
- **Document and accept:** NFR-VIO-001 (architecture), NFR-VIO-007 (write E2E test or drop deps), NFR-VIO-008 (cosmetic), NFR-VIO-010 (matrix expansion is org-cost decision).
- Net: **4 firm fixes, 2 conditional fixes, 4 document/accept.** No NFR-VIO requires a multi-day refactor.

---

## 5. Target 4 -- Consolidate `MAX_BUF` and `MAX_HEADER_BUF`?

Both constants equal `65_536` (64 KB):

- `src/analyzer/http.rs:8` -- `const MAX_HEADER_BUF: usize = 65_536;` (HTTP per-direction header-only buffer cap)
- `src/analyzer/tls.rs:14` -- `const MAX_BUF: usize = 65_536;` (TLS per-direction record-assembly buffer)

`git blame` shows independent introduction (HTTP analyzer commit `11aa920d` 2026-04-06; TLS analyzer commit `3200bf3a` 2026-04-07). Each is local to its analyzer's `const` block. ADR 0001 sec."Broadcast to All Analyzers" describes the HTTP buffer descriptively ("HTTP already buffers up to 64KB per flow direction") but doesn't mandate that TLS match.

### Recommendation: **DO NOT consolidate.** Document instead.

Rationale:
- The two values are semantically different ceilings: `MAX_HEADER_BUF` bounds *parsed HTTP header bytes* (lines until `CRLF CRLF`), while `MAX_BUF` bounds *raw TLS record bytes before parsing*. The fact that they happen to be 64 KB today is a coincidence of "1 MTU window of headers" and "TLS record fits comfortably in 64 KB".
- Coupling them under a shared constant would create a false ownership relationship -- if a future PR needed to raise `MAX_BUF` to 128 KB for TLS 1.3 handshake fragmentation, raising the shared constant would also widen HTTP's header window, which is a different policy decision.
- Each analyzer's constant block (`http.rs:8-11`, `tls.rs:14-18`) is the right scope for the value.

**Action:** add a one-line comment on each constant cross-referencing the other (`// Coincidentally matches MAX_BUF in TLS analyzer; the two ceilings are independent policy.`). Cost: **S**. Disposition: **document.**

---

## 6. Target 5 -- Saturating Arithmetic Site Audit (NFR-REL-003)

P4 R1 claimed 13 sites; ground truth from `grep -rn 'saturating_' src/`:

| # | Site | Operand type | Upstream input bound | Necessity verdict |
|---|---|---|---|---|
| 1 | `reassembly/flow.rs:228` | `fin_count: u32 .saturating_add(1)` | TCP FIN flag from packets in a single flow direction; unbounded in principle (an adversary can send 2^32+1 FIN packets). | **Necessary.** Without saturation, release build panics on overflow per NFR-REL-001. |
| 2 | `reassembly/segment.rs:53` | `u64 .saturating_add(window as u64)` | `max_receive_window` is bounded by `ReassemblyConfig` (default 1 MB, asserted >0 in ctor); `base_offset` is a u64 stream offset (could grow large on long-lived flows). | **Necessary.** Stream offset wrap-into-window-check would otherwise overflow on multi-TB flows. |
| 3 | `reassembly/segment.rs:54` | `out_of_window_count: u32 .saturating_add(1)` | Counter; same situation as #1. | **Necessary.** |
| 4 | `reassembly/segment.rs:69` | `usize .saturating_sub(reassembled_bytes)` | `max_depth` config-bounded; `reassembled_bytes` could exceed it briefly during truncation. | **Necessary** (subtract-with-underflow protection). |
| 5 | `reassembly/segment.rs:83` | `usize .saturating_sub(reassembled+buffered)` | Same. | **Necessary.** |
| 6 | `reassembly/mod.rs:229` | `usize .saturating_sub(before_insert)` | `buffered_bytes` is monotonic across insertion; the subtraction *should* never underflow (the post-insert is always >= pre-insert), so the saturation is a defensive belt. | **Belt-and-suspenders.** Could be `assert!(after >= before); after - before` instead, but saturating is forgiving. **Acceptable as-is.** |
| 7 | `analyzer/tls.rs:677` | `usize .saturating_sub(client_buf.len())` | `MAX_BUF=65_536` is the constant; `client_buf.len() <= MAX_BUF` is the invariant (enforced two lines later by `.truncate`). Underflow impossible. | **Defensive.** Same as #6; acceptable. |
| 8 | `analyzer/tls.rs:684` | Same for `server_buf`. | Same. | **Defensive.** |
| 9 | `analyzer/http.rs:341` | `request_error_count: u8 .saturating_add(1)` | u8 counter incremented per parse error. Threshold is 3 (POISON_THRESHOLD), so realistic max is ~3, but with mid-stream-join + non-HTTP traffic it could increment many times before the poison short-circuits the loop. **Saturation matters because u8 saturates at 255**; without it, the 256th error would panic in release. | **Necessary.** |
| 10 | `analyzer/http.rs:399` | Same for response. | Same. | **Necessary.** |
| 11 | `analyzer/http.rs:445` | `MAX_HEADER_BUF .saturating_sub(request_buf.len())` | Invariant: `request_buf.len() <= MAX_HEADER_BUF` (enforced two lines later by `.extend` + `.truncate`). | **Defensive.** Same as #6. |
| 12 | `analyzer/http.rs:457` | Same for response. | Same. | **Defensive.** |

### Findings

- **True count: 12, not 13.** P4 R1's NFR-REL-003 should be corrected to "12 sites".
- **4 sites are strictly necessary** (1, 3, 9, 10) -- u32 / u8 counters that could realistically saturate on adversarial input.
- **4 sites are necessary for offset / depth arithmetic** (2, 4, 5) and one mid-tier (6).
- **4 sites are defensive belt-and-suspenders** (6 conceptually, 7, 8, 11, 12) on subtractions where the invariant should already prevent underflow. These are NOT incorrect, but a reader auditing for redundancy could remove them and rely on the surrounding invariants. Keeping them is cheap and aligns with `overflow-checks=true`'s defensive posture. **No change recommended.**

---

## 7. Target 6 -- Finding.timestamp Policy Decision

P2 R2/R3 and P3 R4 converged on the same observation: `Finding.timestamp` is *universally None* across all 22 emission sites in `src/`:

- `reassembly/mod.rs`: lines 286, 305, 329, 415, 545, 561 -- 6 sites, all `None`.
- `analyzer/tls.rs`: lines 405, 424, 443, 471, 492, 534, 555 -- 7 sites, all `None`.
- `analyzer/http.rs`: lines 187, 216, 231, 246, plus emission sites for 404 / long URI / web shell etc. -- 9 sites, all `None`.
- **Zero `timestamp: Some(...)` emissions anywhere in `src/`.**

Meanwhile, the data is available: `RawPacket.timestamp_secs: u32` is read from the pcap header in `src/reader.rs:43` and threaded through `process_packet(packet, timestamp, handler)` at `src/reassembly/mod.rs:111`. The pipeline carries the timestamp; the Finding constructor just doesn't consume it.

### Two options

**Option A -- Wire it.**
- Convert `RawPacket.timestamp_secs: u32` to `DateTime<Utc>` (already a dep) via `DateTime::from_timestamp(secs as i64, 0)`. Plumb through `StreamHandler::on_data(flow_key, dir, data, timestamp)` (existing signature already includes a timestamp arg in the engine -- analyzers ignore it today).
- Cost: **M.** ~22 emission sites updated to `timestamp: Some(now)`, signature updates on three analyzers and one helper. Plus one new unit test verifying the round-trip.
- Benefit: forensic findings carry packet-time provenance. SIEM consumers can correlate Finding.timestamp with the originating pcap moment instead of guessing.

**Option B -- Deprecate the field.**
- Remove `timestamp: Option<DateTime<Utc>>` from `Finding`; drop the `chrono` dep (or keep it transitively for `serde`).
- Cost: **S.** ~22 emission sites lose the field; downstream JSON schema breaks for any consumer that *expected* the field (none today).
- Benefit: smaller struct, no dead field, no dependency on chrono if removable.

### Recommendation: **Option A.**

Rationale: forensic tools live and die by timestamp provenance. The field's existence in the struct (committed since project inception) is documented intent; deletion would be the wrong direction. The fact that `chrono` is already a declared dep (`Cargo.toml:21` with `serde` feature) confirms the intent. The plumbing cost (M) is one PR.

**Two caveats:**
1. The Y2106 issue noted in Pass 3 BC-RDR-005 (u32 timestamp wraps in 2106) is moot for `DateTime<Utc>` (i64-backed) -- BUT the *input* `u32` wraps. The conversion `DateTime::from_timestamp(secs as i64, 0)` silently maps high-u32 values to dates after year 2106 correctly, so this is not a wraparound trap.
2. There is currently no system-time fallback -- if a future user wires *live capture* mode (no pcap header timestamp), the `RawPacket.timestamp_secs = 0` sentinel would map to 1970-01-01. Not a concern today (offline-only).

---

## 8. Target 7 -- Benchmark Survey + `criterion` Recommendation

Re-verified by `find /Users/zious/Documents/GITHUB/wirerust -name benches -type d` (no output) and `grep -nE 'criterion|\[\[bench\]\]' Cargo.toml` (no output).

- **No `benches/` directory exists.**
- **No `criterion` (or any benchmarking) dependency.**
- **No `[[bench]]` target in `Cargo.toml`.**

P4 R1 sec.1 already flags this: "MEDIUM for performance (no benchmarks in repo; 'fast' is a marketing claim not a measured threshold)". NFR-PERF-001..004 are policy NFRs (zero-copy parsing choice; single-pass; cache-on-first-hit; SIMD slice cmp) -- the *policy* is sound, but no benchmark pins the *outcome*.

### Recommendation: add `criterion` for the reassembly hot path.

The reassembly engine is the load-bearing perf component (every TCP packet flows through `process_packet`, segment-insert, contiguous flush). Three benchmark scenarios that would directly validate the "multi-GB captures" README claim:

1. `bench_reassemble_100k_segments_ordered` -- baseline `insert_segment` throughput.
2. `bench_reassemble_100k_segments_random_order` -- BTreeMap insert + flush behavior on OOO.
3. `bench_overlap_first_wins_floods` -- worst-case overlap-detection cost (P3 R3 noted the SIMD slice-cmp dependence).

Cost: **M.** New `[dev-dependencies] criterion = "0.5"`, new `benches/reassembly.rs`, optional `[[bench]] name = "reassembly" harness = false` in `Cargo.toml`. CI gating on benchmarks is a separate decision (don't gate on perf regressions until baselines stabilize).

**Add this if perf claims are to remain load-bearing.** Without benchmarks, NFR-PERF stays MEDIUM-confidence in perpetuity.

---

## 9. Target 8 -- `cargo audit` / `cargo deny` CI Recommendation

Re-verified by `grep -nE 'cargo audit|cargo deny|dependency-review' .github/workflows/ci.yml` (no output). The CI is `cargo test` + `cargo clippy` + `cargo fmt --check` only. Supply-chain auditing is **not automated.**

NFR-SUP-001..005 (manifest hygiene) are all observed properties of the manifest; none of them gate against future regressions. If a transitive dep ships a known CVE tomorrow, wirerust's CI does not surface it.

### Recommendation: add a `security` job to `.github/workflows/ci.yml`.

```yaml
security:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: rustsec/audit-check@v2.0.0   # cargo-audit wrapper, fails on CVE
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
```

Optional second pass with `cargo-deny` for license-policy enforcement and unmaintained-crate detection (`EmbarkStudios/cargo-deny-action@v2`).

Cost: **S.** ~10 lines of YAML. The downside is CI sometimes failing on a transitive CVE the project can't immediately fix; this is the desired feedback loop.

**Add this.** Disposition: **fix.**

---

## 10. Cross-Pollination -- NFR Updates from P3 R3 / R4 / P2 R3

Apply the targeted updates from the orchestrator's cross-pollination directive.

### 10.1 NFR-RES-002/003/004 -- per-direction correction (from P3 R3)

P4 R1 said these alerts fire "per-flow-direction" in the NFR-statement column already (e.g. NFR-RES-002: "Per-flow-direction overlap-anomaly alert fires when overlap_count > 50 and exactly ONCE (sticky overlap_alert_fired)"). So the text is **already correct.** BUT the worst-case alert cardinality is not stated: the implication of "once per direction" combined with 3 alert types is **6 findings worst case per bidirectional flow** (3 types x 2 directions). P3 R3 explicitly corrected BC-RAS-022 on this; P4 R1's NFR rows do not derive the implication.

**Refinement to apply (DELTA):** the NFR statements for 002/003/004 should append: "Worst-case alert cardinality per bidirectional flow is 6 findings (this alert type x 2 directions x the latching invariant), distributed across overlap/small-segment/out-of-window. Combined with NFR-RES-001 (MAX_FINDINGS=10_000 cap), a single attacker-driven flow consumes at most 6/10_000 = 0.06% of the global findings budget per anomaly direction-type."

### 10.2 NFR-RES-NEW-022 -- propose `dropped_findings: u64` counter

From P3 R3 Target 3: MAX_FINDINGS silent drops are observable today only as "stats.findings == 10000" with no signal of how many were dropped. Counter design:

```rust
// In ReassemblyStats:
/// Findings dropped because the engine already accumulated MAX_FINDINGS;
/// observable via summarize() detail.
pub dropped_findings: u64,
```

Incremented in `generate_*_finding` paths at `mod.rs:272/291/310/534` when `self.findings.len() >= MAX_FINDINGS` returns early; surfaced via `summarize().detail["dropped_findings"]`.

**Add NFR-RES-022 to the catalog:**

| ID | Category | NFR statement | Where encoded | Numeric value | Rationale | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-RES-022 (PROPOSED) | Resource bounds | When MAX_FINDINGS is reached, further finding pushes are silently dropped; a `dropped_findings: u64` counter on `ReassemblyStats` makes the drop observable via `summarize().detail`. | NEW: `src/reassembly/mod.rs:ReassemblyStats` add field; increment in `generate_*_finding` guards (lines 272/291/310/534/550). | u64, monotonic | Closes the silent-truncation gap. ADR 0002's "the counter is the signal" pattern already applies to parse_errors and out_of_window_count. | Runtime + test (test signature in P3 R3 sec.Target 3) | NEW: `test_dropped_findings_counter_increments_when_cap_reached` (drafted P3 R3) |

### 10.3 NFR-RES-NEW-023 -- weak-cipher Finding heap bound

From P2 R3 sec.13 + P3 R4 (cross-reference): the ClientHello weak-cipher emission at `src/analyzer/tls.rs:454-473` uses `evidence: weak` where `weak: Vec<String>` is built by `ch.ciphers.iter().filter(is_weak_cipher).map(cipher_name).collect()`. The cipher list is bounded only by `MAX_RECORD_PAYLOAD=18_432` / 2 bytes-per-cipher = **9216 ciphers max per ClientHello**. Each `cipher_name` is up to ~30 chars (e.g. `"TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256"` = 44 chars). Worst-case heap = 9216 x (24-byte String header + 30 bytes) ~= **~500KB** (P3 R4 said ~270KB which assumed shorter names; either bound is within the same order of magnitude).

This is the **only Finding emission site with data-dependent evidence cardinality** (P2 R3 sec.13 confirmed). No per-cipher cap exists.

**Add NFR-RES-023 to the catalog:**

| ID | Category | NFR statement | Where encoded | Numeric value | Rationale | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-RES-023 (PROPOSED) | Resource bounds | ClientHello weak-cipher Finding evidence vec is the only data-dependent-cardinality evidence vector in the codebase. Upper bound is ~9216 entries (MAX_RECORD_PAYLOAD / 2-bytes-per-cipher); worst-case Finding heap ~270-500KB. No per-cipher cap exists. | `src/analyzer/tls.rs:454-473` | bounded by MAX_RECORD_PAYLOAD=18_432 | The 9216-entry upper bound is implicit in MAX_RECORD_PAYLOAD; no explicit guard. Pathological ClientHello with mostly-weak ciphers produces the worst case. | Runtime (transitive via NFR-RES-016) | None today; recommend a stress test asserting Finding.evidence.len() <= 9216 |

**Recommendation:** add `const MAX_WEAK_CIPHER_EVIDENCE: usize = 64;` and truncate `weak` to `[0..MAX_WEAK_CIPHER_EVIDENCE]` with an "+N more" suffix entry. 64 covers every realistic weak-cipher list (the entire IANA weak-cipher set is ~50 entries); pathological inputs are bounded.

### 10.4 NFR-OBS-NEW-010 -- Finding JSON schema asymmetry (from P3 R4 BC-FND-006)

P3 R4 confirmed that `Finding.timestamp` carries `#[serde(skip_serializing_if = "Option::is_none")]` but `mitre_technique` and `source_ip` do not (re-verified by reading `src/findings.rs:66-69`). JSON output for a Finding with `mitre_technique: None` includes `"mitre_technique": null`; for `source_ip: None` includes `"source_ip": null`; but timestamp == None is *omitted entirely*.

This is a contract for any downstream JSON consumer: `mitre_technique` and `source_ip` are *always present* keys (null-or-string); `timestamp` is *optional* key (present-or-absent).

**Add NFR-OBS-010 to the catalog:**

| ID | Category | NFR statement | Where encoded | Numeric value | Rationale | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-OBS-010 (NEW) | Observability | `Finding`'s JSON schema is **asymmetric** on Option fields: `mitre_technique: Option<String>` and `source_ip: Option<IpAddr>` always serialize (as `null` when None); only `timestamp: Option<DateTime<Utc>>` carries `skip_serializing_if = "Option::is_none"` and may be omitted. Downstream consumers must handle key-present-but-null for mitre/source_ip and key-absent for timestamp. | `src/findings.rs:60-70` (Serialize derive + the lone skip annotation at line 68) | -- | Almost certainly unintentional asymmetry. Recommend either (a) add `skip_serializing_if` to mitre_technique and source_ip for symmetry, OR (b) remove it from timestamp so all 3 fields always serialize. Eng decision. | Compile-time (derive Serialize) | None today; P3 R4 recommended a `test_finding_serializes_with_or_without_options` snapshot test |

### 10.5 Summary of cross-pollination deltas

| NFR | Status | Action |
|---|---|---|
| NFR-RES-002/003/004 | TEXT REFINED | Append "worst-case 6 findings per bidirectional flow" implication. |
| NFR-RES-022 | NEW (proposed) | Add `dropped_findings: u64` counter to ReassemblyStats. |
| NFR-RES-023 | NEW | Document weak-cipher Finding heap upper bound (~270-500KB); recommend per-cipher truncation cap. |
| NFR-OBS-010 | NEW | Document Finding JSON schema Option-serialization asymmetry. |

Net: **4 NFR additions/refinements** from cross-pollination -- all SUBSTANTIVE.

---

## 11. Refined NFR Count

| Category | P4 R1 count | This round delta | New count |
|---|---|---|---|
| NFR-PERF | 4 | 0 | 4 |
| NFR-SEC | 8 | 0 | 8 |
| NFR-REL | 11 | 0 | 11 |
| NFR-OBS | 9 | +1 (NFR-OBS-010 JSON schema asymmetry) | 10 |
| NFR-RES | 21 | +2 (NFR-RES-022 dropped_findings counter; NFR-RES-023 weak-cipher heap bound) + 3 text-refined | 23 |
| NFR-MNT | 11 | 0 | 11 |
| NFR-PORT | 5 | 0 | 5 |
| NFR-SUP | 5 | 0 | 5 |
| NFR-COMPAT | 2 | 0 | 2 |
| **Total NFRs** | **76** | **+3 new, +3 text-refined** | **79** |
| NFR-VIO-* | 10 | dispositions assigned (no new violations) | 10 |
| Magic numbers (de-dup) | "28" claimed | ~31 actual de-duplicated | ~31 |
| Saturating sites | "13" claimed | 12 actual | 12 |

---

## Delta Summary

- **New items added:** NFR-OBS-010 (JSON schema asymmetry), NFR-RES-022 (dropped_findings counter recommendation), NFR-RES-023 (weak-cipher Finding heap bound).
- **Existing items refined:** NFR-RES-002/003/004 (worst-case alert cardinality clarification); NFR-REL-003 count corrected to 12; numeric index count refined to ~31.
- **Provenance research:** 5 thresholds resolved (4 commits + 1 PR each); POISON_THRESHOLD downgraded from "inferred" to "empirical, fixture-calibrated" (the only such threshold).
- **NFR-VIO dispositions:** 4 firm fixes (002, 004, 006, 009), 2 conditional fixes (003, 005), 4 document-and-accept (001, 007, 008, 010).
- **Concrete fixes recommended:**
  - Add `rust-version = "1.86"` to Cargo.toml (NFR-VIO-009).
  - Remove `*.pcapng` from `resolve_targets` glob (NFR-VIO-002).
  - Wire `--json <FILE>` to actually write to file (NFR-VIO-004).
  - Drop unused `rayon` dep (NFR-VIO-006).
  - Add `cargo audit` CI job (NFR-SUP gap).
  - Add `criterion` benches/reassembly.rs (NFR-PERF measurability).
  - Add `dropped_findings: u64` to ReassemblyStats (NFR-RES-022).
  - Add per-cipher truncation cap for weak-cipher Finding (NFR-RES-023).
  - Decide JSON schema symmetry: skip_serializing_if on all Options or none (NFR-OBS-010).

- **Remaining gaps:** All 8 carryover targets from P6 sec.7 P4 are now addressed. No new gaps surfaced that warrant a third deepening round.

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification -- would removing this round's findings change how you'd spec the system?
- **YES.** Three concrete changes to the system spec: (1) NFR-OBS-010 is a JSON schema contract every downstream consumer must encode; without it, "all Option fields in Finding may be null or absent" would be wrongly assumed symmetric. (2) NFR-RES-022 (dropped_findings counter) is a missing-observability NFR with a drafted implementation; absent this round, MAX_FINDINGS silent truncation remains undetectable in production. (3) NFR-RES-023 weak-cipher Finding heap bound documents a previously-untracked memory ceiling (~270-500KB) that bounds wirerust's worst-case Finding payload. The provenance research independently downgraded one threshold ("inferred" -> "empirical") and confirmed three remain unjustified -- this is direct input to a future calibration PR. The 10-VIO disposition matrix becomes a concrete remediation backlog.

Findings that were merely confirmations (saturating arithmetic site count drift, "76 NFRs" verification, MAX_BUF/MAX_HEADER_BUF non-consolidation decision) are NITPICK-grade and noted but not load-bearing.

## Convergence Declaration

**Pass 4 has converged.** This round produced 3 new NFRs + 3 text-refinements + 1 calibration-recommendation backlog + 1 disposition matrix for all 10 violations -- substantive material that closes the carryover gaps from P6 sec.7. A third deepening round would yield only confirmations and edge-case nitpicks; the remaining open questions (calibrate the 3 unjustified thresholds against benign-traffic; choose JSON schema symmetry; choose Option A vs B for Finding.timestamp wiring) are **engineering decisions**, not analysis tasks. Pass 8 deep synthesis should consume P4 R1 + this round as the complete NFR corpus.

## State Checkpoint

```yaml
pass: 4
round: 2
status: complete
files_re_read_p4_r1: 1
files_re_read_p3_r3: 1
files_re_read_p3_r4: 1
files_re_read_p2_r3: 1
files_re_read_source: 7
git_blame_commits_investigated: 5
nfrs_total_after_round: 79
nfrs_new_this_round: 3
nfrs_text_refined: 3
nfr_violations_dispositioned: 10
hallucination_classes_audited: 5
hallucination_drifts_found: 2 (count drift: 28 -> ~31 distinct constants; 13 -> 12 saturating sites)
hallucination_fabrications_found: 0
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
converged: true
next_pass: 8 (deep synthesis -- P4 corpus complete with R1 + R2)
```
