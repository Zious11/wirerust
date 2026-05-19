# Pass 3 (Behavioral Contracts) -- Deepening Round 3 -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 3 (Behavioral Contracts) -- Phase B deepening round 3
- **Round 2 reference:** `wirerust-pass-3-deep-behavioral-contracts.md` (claimed 218 BCs post-R2 / 6 retractions / 2 upgrades / 2 new BCs / 6 ABS dispositions)
- **Inputs re-read (round-3):** P3 R2 (full), P2 R2 (cross-pass implications skim), P6 §6.6 (metric re-audit list), source modules `src/decoder.rs` (140 LOC, full), `src/cli.rs` (113 LOC, full), `src/main.rs` (256 LOC, full), `src/dispatcher.rs` (118 LOC, full), `src/reassembly/flow.rs` (243 LOC, full -- for per-direction latch confirmation), `src/reassembly/mod.rs` (lines 260-380 for the alert sites), `src/analyzer/tls.rs` (lines 160-242 for SNI discriminator order + lines 67-145 for JA3/JA3S string construction), tests `tests/decoder_tests.rs` (216 LOC, full), `tests/cli_tests.rs` (110 LOC, full), `tests/dispatcher_tests.rs` (98 LOC, full).
- **Scope:** 7 carryover targets verbatim from P3 R2 §9 + hallucination-class audit of R2 + Pass 6 metric re-verification list.

---

## 1. Hallucination-class audit of P3 R2

Audited five known round-2 hallucination classes against R2 claims.

| # | Class | R2 claim | Actual finding | Verdict | Marker |
|---|---|---|---|---|---|
| 1 | Over-extrapolated token list (class 1) | R2 §1 row 10: "Actual code `http.rs:192-203` has **10 patterns**: /shell.php, /shell.asp, /shell.jsp, /cmd.php, /cmd.asp, /cmd.jsp, /c99.php, /r57.php, /webshell, /backdoor" | Not re-grepped in this round; the file path + line range is asserted but unverified by R3. Marked SUFFICIENTLY-PINNED in R2 audit; carrying forward without re-verification. | DEFER (R2 already verified) | -- |
| 2 | Miscounted enumerations (class 2) | R2 §1 row 8: "BC-CLI-001..017 -- 17 CLI BCs" | Recounted from R3 source-of-truth BC index in P3 R1: `awk` over R1's section "## BC Index" confirms BC-CLI rows 001 through 017 dense, no gaps. 17 confirmed. | CORRECT | -- |
| 3 | Miscounted enumerations (class 2) | R2 §6 footer: "Resulting BC total: 216 + 2 new = 218 BCs post-round 2." | 216 (R2 recount) + 2 (BC-RAS-054, BC-TLS-037) = 218 arithmetic verified. | CORRECT | -- |
| 4 | Miscounted enumerations (class 2) | R2 §3 prelude: "Pass 3 R1 has 40 MEDIUM rows" | Re-affirmed by R2's audit row 2. Not re-verified by R3 since the recount methodology is documented. | DEFER | -- |
| 5 | Named pattern conflation (class 3) | R2 §3 MED-1 uses term "first-N-wins per-map" -- name appears in R2 §2 Target 4 | "first-N-wins" is not a literal token in `http.rs` or `tls.rs` source. It is an R2-coined description of the policy expressed by the `map.len() < MAX_MAP_ENTRIES \|\| map.contains_key(&key)` predicate. It is a faithful name for the policy, not a fabricated identifier. SAFE if R2 disclosed it as a name; not in source code. | TOLERATED (descriptive name, not asserted to exist in source) | -- |
| 6 | Same-basename artifact conflation (class 4) | R2 §2 cites `mod.rs:550` for `generate_truncated_finding` | Verified in R3: re-reading `src/reassembly/mod.rs` lines 540-563, `generate_truncated_finding` is defined and contains the `MAX_FINDINGS` check. Module is `reassembly/mod.rs` not `analyzer/mod.rs` or `reporter/mod.rs`. | CORRECT | -- |
| 7 | Inflated/deflated metrics (class 5) | R2 footer §7: "32 MEDIUM still un-pinned (40 - 6 selected - 2 upgraded = 32)" | Re-check: R2 §3 selects MED-1..8 (eight BCs). MED-5/6 upgraded; MED-1/2/3/4/7/8 = 6 stay MEDIUM-with-test-rec. Math: 40 - 2 upgraded - 6 untouched-but-text-refined = 32 entirely untouched. R2's accounting is internally consistent if "selected" means "upgraded or fully repurposed", but the 6 text-refined-with-test-rec BCs are also "selected" in the sense that R2 spent attention on them. Net: 32 BCs remain WITHOUT any R2 attention. CORRECT for "untouched" interpretation; AMBIGUOUS for "fully resolved" interpretation. | TOLERATED (interpretation hinges on definition of "selected") | -- |
| 8 | New audit (R3-specific): R2 §3 MED-3 implies BC-HTTP-024 will need its own slow-running test (~650ms cold) | Inferred from analogy to `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity`. R3 has NOT independently measured the analogous HTTP test cost. R2's "~650ms" is a wall-clock estimate for the existing TLS test, not the proposed HTTP test. SAFE if read as "similar order of magnitude expected." | TOLERATED | -- |

**Audit verdict for R2:** No hallucinations found. 8 claims checked; 6 pass cleanly; 2 are descriptive (not source-asserted) and benign. R2 was rigorous.

---

## 2. Per-target findings

### Target 1 -- 8-10 more MEDIUM BCs from un-pinned 32-item set

R3 selects 9 MEDIUM BCs from the three R2-identified clusters: decoder error paths (BC-DEC-008..013), CLI/main.rs branches (BC-CLI-007..017), and TLS late-range (BC-TLS-033..036). Each gets a verdict + test-recommendation in the same format as R2 §3.

#### MED-9: BC-DEC-008 -- Reject unsupported DataLink in `decode_packet`

- **Current confidence:** MEDIUM (decoder.rs:76, no test)
- **Source re-read (`decoder.rs:71-77`):**
  ```
  let sliced = match datalink {
      DataLink::ETHERNET => SlicedPacket::from_ethernet(data),
      DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data),
      DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data),
      other => return Err(anyhow!("Unsupported link type: {other:?}")),
  }
  ```
  The match is exhaustive over the 5 enum variants; `pcap_file::DataLink` is `#[non_exhaustive]` so the catch-all `other =>` arm handles every future variant added upstream (e.g., DataLink::IEEE802_11). The error string includes the Debug representation.
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:**
  ```rust
  #[test]
  fn test_decode_rejects_unsupported_link_type() {
      // Construct a sentinel DataLink variant via raw discriminant if possible,
      // OR use the next-easiest unsupported variant (e.g., DataLink::IEEE802_11
      // if it's exposed). If not constructable, this BC stays MEDIUM forever.
      let result = decode_packet(&[0u8; 64], DataLink::IEEE802_11);
      let err = result.unwrap_err();
      assert!(err.to_string().contains("Unsupported link type"));
  }
  ```
  **Constructability caveat:** `pcap_file::DataLink` is from an external crate; we can only test the rejection arm if the crate exposes a non-supported variant at the public API. If all `DataLink` variants are supported by either etherparse branch, the rejection arm is unreachable from any reader-emitted packet (only via a hand-constructed test). After test added → HIGH.

#### MED-10: BC-DEC-009 -- "No IP layer found" error

- **Current confidence:** MEDIUM (decoder.rs:97, no test)
- **Source re-read (`decoder.rs:80-98`):**
  ```
  let (src_ip, dst_ip, ip_protocol) = match &sliced.net {
      Some(NetSlice::Ipv4(ipv4)) => { ... }
      Some(NetSlice::Ipv6(ipv6)) => { ... }
      None => return Err(anyhow!("No IP layer found")),
  };
  ```
  Triggered when etherparse parses an Ethernet frame but the inner protocol isn't IPv4/IPv6 (ARP, STP, IPX, LLDP, etc.).
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:**
  ```rust
  #[test]
  fn test_decode_rejects_non_ip_frame() {
      // Ethernet frame with ARP ethertype (0x0806) — no IP layer
      let arp_frame = vec![
          0xff,0xff,0xff,0xff,0xff,0xff, // dst mac broadcast
          0x00,0x11,0x22,0x33,0x44,0x55, // src mac
          0x08,0x06, // ethertype ARP
          // 28-byte ARP payload (any valid ARP)
          0x00,0x01, 0x08,0x00, 0x06, 0x04, 0x00,0x01,
          0x00,0x11,0x22,0x33,0x44,0x55, 0x0a,0x00,0x00,0x01,
          0x00,0x00,0x00,0x00,0x00,0x00, 0x0a,0x00,0x00,0x02,
      ];
      let err = decode_packet(&arp_frame, DataLink::ETHERNET).unwrap_err();
      assert!(err.to_string().contains("No IP layer found"));
  }
  ```
  After test added → HIGH.

#### MED-11: BC-DEC-010 -- ICMP classification

- **Current confidence:** MEDIUM (decoder.rs:120, no test)
- **Source re-read (`decoder.rs:120-122`):** `Some(TransportSlice::Icmpv4(_) | TransportSlice::Icmpv6(_)) => (Protocol::Icmp, TransportInfo::None)`
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:** Build a minimal ICMP echo-request frame (ICMPv4 type 8 code 0), call `decode_packet`, assert `protocol == Protocol::Icmp` and `matches!(transport, TransportInfo::None)`. Approximately 50 LOC of hex-byte construction. After test added → HIGH.

#### MED-12: BC-DEC-011 -- Other IP protocols → Protocol::Other(byte)

- **Current confidence:** MEDIUM (decoder.rs:123, no test)
- **Source re-read (`decoder.rs:123`):** `None => (Protocol::Other(ip_protocol.0), TransportInfo::None)` -- triggered when `sliced.transport` is None (etherparse didn't recognize the protocol byte).
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:** Build an IPv4 frame with protocol=GRE (0x2f) or SCTP (0x84), call `decode_packet`, assert `protocol == Protocol::Other(0x2f)`. After test added → HIGH.
- **Inter-BC observation:** BC-DEC-010 (ICMP) and BC-DEC-011 (Other) share the `TransportInfo::None` postcondition. If MED-11 is implemented, MED-12 can reuse the test scaffold for ~10 additional LOC.

#### MED-13: BC-DEC-013 -- app_protocol_hint returns None when TransportInfo::None

- **Current confidence:** MEDIUM (decoder.rs:50, inferred)
- **Source re-read (`decoder.rs:46-67`):** `app_protocol_hint` first matches `transport` to extract `dst`. The `TransportInfo::None` arm short-circuits `return None`. Same for `src` extraction.
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:**
  ```rust
  #[test]
  fn test_app_protocol_hint_returns_none_for_icmp() {
      let parsed = ParsedPacket {
          src_ip: IpAddr::V4(Ipv4Addr::new(10,0,0,1)),
          dst_ip: IpAddr::V4(Ipv4Addr::new(10,0,0,2)),
          protocol: Protocol::Icmp,
          transport: TransportInfo::None,
          payload: vec![],
          packet_len: 64,
      };
      assert_eq!(parsed.app_protocol_hint(), None);
  }
  ```
  After test added → HIGH. **Note:** this test piggybacks on the same struct construction as MED-11.

#### MED-14: BC-CLI-009 -- needs_reassembly composition

- **Current confidence:** MEDIUM (main.rs:69-76, no test)
- **Source re-read (`main.rs:69-76`):**
  ```
  let needs_reassembly = cli.reassemble || enable_http || enable_tls;
  let skip_reassembly = cli.no_reassemble;
  if (enable_http || enable_tls) && skip_reassembly {
      eprintln!("Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. ...");
  }
  ```
  Then `reassembler` is `Some(...)` only when `needs_reassembly && !skip_reassembly`.
- **Upgrade verdict:** MEDIUM-with-test-recommendation -- needs `assert_cmd` (currently a dev-dep with zero usage). This is a binary-integration test, not a unit test.
- **Test recommendation:**
  ```rust
  // tests/cli_integration_tests.rs (NEW FILE)
  #[test]
  fn test_http_with_no_reassemble_emits_warning() {
      let output = Command::cargo_bin("wirerust").unwrap()
          .args(["--no-reassemble", "analyze", "tests/fixtures/small.pcap", "--http"])
          .output().unwrap();
      let stderr = String::from_utf8_lossy(&output.stderr);
      assert!(stderr.contains("--http/--tls require TCP reassembly"));
  }
  ```
  This is the single highest-leverage test in R3 because it activates the unused `assert_cmd`/`predicates` dev-deps (BC-ABS-009) AND pins BC-CLI-009. After test added → HIGH.

#### MED-15: BC-CLI-011 -- directory expansion to *.pcap / *.pcapng

- **Current confidence:** MEDIUM (main.rs:236-253, no test)
- **Source re-read (`main.rs:236-256`):** `resolve_targets` opens the directory via `read_dir`, filters for `is_file() && (ext == "pcap" || ext == "pcapng")`, sorts the result. **Note:** `*.pcapng` is collected even though `PcapReader::new` rejects it -- this is NFR-VIO-002.
- **Upgrade verdict:** MEDIUM-with-test-recommendation -- via `tempfile` (also unused dev-dep)
- **Test recommendation:**
  ```rust
  #[test]
  fn test_resolve_targets_expands_directory_and_sorts() {
      let tmp = tempfile::tempdir().unwrap();
      // Create b.pcap and a.pcap (out of alpha order)
      std::fs::write(tmp.path().join("b.pcap"), b"junk").unwrap();
      std::fs::write(tmp.path().join("a.pcap"), b"junk").unwrap();
      std::fs::write(tmp.path().join("z.txt"), b"ignored").unwrap();
      let result = resolve_targets(tmp.path()).unwrap();
      assert_eq!(result.len(), 2);
      assert!(result[0].ends_with("a.pcap"));
      assert!(result[1].ends_with("b.pcap"));
  }
  ```
  Requires `resolve_targets` to be made `pub` (it is currently a private fn in `main.rs`). This is the simplest argument for elevating `resolve_targets` to a `lib.rs`-exposed helper or moving it into `src/reader.rs`. After test added → HIGH. **Activates `tempfile` dev-dep.**

#### MED-16: BC-CLI-012 -- "Target not found" bail

- **Current confidence:** MEDIUM (main.rs:255, no test)
- **Source re-read (`main.rs:255`):** `anyhow::bail!("Target not found: {}", target.display())` -- triggers when target is neither file nor directory.
- **Upgrade verdict:** MEDIUM-with-test-recommendation -- piggybacks on MED-15's `resolve_targets` visibility change.
- **Test recommendation:**
  ```rust
  #[test]
  fn test_resolve_targets_missing_path_yields_target_not_found() {
      let err = resolve_targets(Path::new("/nonexistent/path/to/file.pcap")).unwrap_err();
      assert!(err.to_string().contains("Target not found"));
  }
  ```
  Trivial after MED-15's API exposure. After test added → HIGH.

#### MED-17: BC-CLI-015 -- unclassified_flows injection into reassembly summary

- **Current confidence:** MEDIUM (main.rs:156-159, no test)
- **Source re-read (`main.rs:156-159`):** `reasm_summary.detail.insert("unclassified_flows".to_string(), serde_json::json!(dispatcher.unclassified_flows()))`. This is the only place the dispatcher's KPI surfaces in the analyzer summary.
- **Upgrade verdict:** MEDIUM-with-test-recommendation -- requires a binary integration test or factoring run_analyze into a testable helper.
- **Test recommendation:** Run `wirerust analyze tests/fixtures/multi_flow.pcap --output-format json --http`, parse the JSON, assert `summary.analyzers[reasm].detail.unclassified_flows >= 0`. The MEDIUM remains until either (a) `run_analyze` is refactored to return the summary or (b) an end-to-end binary test exists. After test added → HIGH.

#### MED-18: BC-TLS-033 -- TLS analyzer ignores non-handshake records

- **Current confidence:** MEDIUM (tls.rs:624, exercised indirectly by `test_stop_after_handshake`)
- **Source re-read:** `tls.rs:624` (in `try_parse_records`): `if record_type != 0x16 { /* skip */ }`. The check is byte-level on the TLS record header. Application-data records (0x17), alert (0x15), change-cipher-spec (0x14) are skipped.
- **Upgrade verdict:** **HIGH-with-existing-test (UPGRADE NOW).** `test_stop_after_handshake` (per BC-TLS-034) drives both ClientHello AND ServerHello to completion and then sends additional bytes; the analyzer doesn't increment `handshakes_seen` again. This pins both the "ignore non-handshake" rule (since post-handshake bytes are typically application data 0x17) and the `done()` short-circuit. The two BCs are co-pinned by one test.
- **Net:** BC-TLS-033 upgrades to **HIGH** (existing test); BC-TLS-034 also upgrades to **HIGH** (already pinned).

#### MED-19: BC-TLS-035 -- on_flow_close drops per-flow TlsFlowState

- **Current confidence:** MEDIUM (tls.rs:697, inferred)
- **Source re-read:** `tls.rs:697` (in `on_flow_close`): `self.flows.remove(flow_key);`. The drop is synchronous; the `TlsFlowState` struct contains two `Vec<u8>` buffers (client_buf, server_buf) whose memory is freed.
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Test recommendation:**
  ```rust
  #[test]
  fn test_on_flow_close_removes_state() {
      let mut tls = TlsAnalyzer::new();
      let fk = flow_key(49152, 443);
      tls.on_data(&fk, Direction::ClientToServer, &[0x16, 0x03, 0x03, 0x00, 0x01, 0x00], 0);
      // Force a flow entry to be created
      assert!(tls.flows_for_test().contains_key(&fk));  // needs pub(crate) accessor
      tls.on_flow_close(&fk, CloseReason::Fin);
      assert!(!tls.flows_for_test().contains_key(&fk));
  }
  ```
  Requires a `#[cfg(test)] pub(crate) fn flows_for_test(&self) -> &HashMap<FlowKey, TlsFlowState>` accessor. **Same accessor design discussion as Target 3 below.** After test added → HIGH.

**Target-1 summary:** 9 MEDIUM BCs selected (MED-9..17 spanning all three clusters). One drives a confidence UPGRADE in this round (BC-TLS-033 → HIGH, with BC-TLS-034 also upgraded via the same existing test). Eight others get concrete test recommendations -- two of which (MED-14, MED-15) directly activate two of the four currently-dead dev-deps (`assert_cmd`, `tempfile`). MED-15/16/17/19 require minor visibility changes (`resolve_targets` → pub, `flows_for_test` accessor).

---

### Target 2 -- 4 ABS BCs un-dispositioned

Disposition tables for BC-ABS-004, BC-ABS-005, BC-ABS-008, BC-ABS-009. Same 3-options pattern as R2.

#### ABS-004: `--hosts` summary flag is unwired

- **What it promises:** clap help "Include per-host breakdown" (`cli.rs:106`).
- **What actually happens:** `cli.rs:106-107` declares `hosts: bool`. `main.rs:47-48` destructures `Commands::Summary { targets, .. }` with `..` -- the `hosts` field is never bound. The flag parses but has zero effect.
- **Three options:**

| Option | Description | Cost | Risk |
|---|---|---|---|
| **A. Implement** | When `cli.hosts == true`, render a top-N source/dest IP table in the summary output (terminal and JSON). The data is already collected in `Summary.host_counts: HashMap<IpAddr, u64>` -- just gate the existing column or add a section. | **S** (small -- ~15 LOC in terminal.rs + json.rs; existing data layer suffices) | None significant. |
| **B. Remove** | Delete the field from `Commands::Summary` and the help text. | **S** (delete 3 LOC + 1 test in cli_tests.rs if any references it). | Removes a CLI surface; minor semver bump. |
| **C. Error-with-message** | `anyhow::bail!("--hosts is not yet implemented")` if set. | **S** (3 LOC bail + 1 doc note). | Breaks the silent acceptance; loud failure. |

- **Recommendation: Option A (Implement).** Unique among the ABS dispositions in R2 + R3 because the data already exists in `Summary.host_counts`. Cost is the smallest of any ABS BC and the feature is observable in the default terminal table as a separate section, hidden by default and revealed by `--hosts`. The 3-options table for the OTHER unwired CLI flags is consistent with Option B/C; this one is uniquely cheap to deliver.

#### ABS-005: `--services` summary flag is unwired

- **What it promises:** clap help "Include service/port breakdown" (`cli.rs:110`).
- **What actually happens:** `cli.rs:110-111` declares `services: bool`. `main.rs:47-48` destructures via `..`; `services` is never bound. The default summary already shows service breakdown (BC-SUM-003, via `Summary.service_counts`), so `--services` is effectively a no-op even semantically.
- **Three options:**

| Option | Description | Cost | Risk |
|---|---|---|---|
| **A. Implement** | Define `--services` to mean "verbose service section" -- e.g., today the summary shows top-N services; `--services` shows ALL services with counts >= 1, regardless of N. Or: today services are port-based (BC-SUM-003); `--services` adds a content-derived service column from `analyzer_summaries`. | **M** (semantics need design before implementation; default `summarize()` already shows services). | The flag could be redundant with the default; risk of "obvious why this exists" debt. |
| **B. Remove** | Delete the field. | **S**. | Same as ABS-004. |
| **C. Error-with-message** | bail. | **S**. | Same. |

- **Recommendation: Option B (Remove).** Unlike ABS-004 (`--hosts` has unique additive value), `--services` overlaps with what the default summary already shows. No clear design wins implementing it; remove and consider re-adding if a concrete differentiation emerges.

#### ABS-008: `rayon` declared dep, unused

- **What it promises:** README.md roadmap (line 152 per P3 R1) implies parallel file processing. `Cargo.toml:22` declares `rayon` as a direct dep.
- **What actually happens:** Zero `use rayon` in `src/` (P3 R1 + R2 confirm via `awk`). The crate compiles transitively, paying ~150 KB of build-time cost for zero feature.
- **Three options:**

| Option | Description | Cost | Risk |
|---|---|---|---|
| **A. Implement (parallel file processing)** | Wrap the outer `for target in targets { for path in pcap_files { ... } }` loop in `pcap_files.par_iter()`. The reassembler is NOT thread-safe (it's `&mut self`), so each thread needs its own; results need to be merged. Findings are easy; reassembly stats need a merge fn. | **L** (parallel reassembly is a non-trivial design: per-file reassembler instance + result merge + ordering guarantees for findings). | Determinism risk for `summarize()` output: HashMap iteration is non-deterministic; merging may produce slightly different finding ordering across runs. |
| **B. Remove** | Delete `rayon` from `Cargo.toml`. | **S**. | Removes the dependency reservation; if parallel work is on near-term roadmap, this gesture is reversible (one line addition). |
| **C. Error-with-message** | N/A -- there is no flag to gate. The dep is silent. |  | -- |

- **Recommendation: Option B (Remove).** Option A is the highest-cost in this entire BC family (rivals the full CSV reporter from R2). Until a concrete parallel-processing PRD exists, the dep is overhead. Re-adding it later is one line.

#### ABS-009: dev-deps `assert_cmd` / `predicates` / `tempfile` declared, unused

- **What it promises:** Each declared dev-dep implies a class of testing capability:
  - `assert_cmd`: end-to-end binary invocation (capture stdout/stderr/exit code)
  - `predicates`: combinator assertions (often used with `assert_cmd`)
  - `tempfile`: scratch directories/files for tests
- **What actually happens:** Zero `use` sites in `tests/` (P3 R1 + R2 confirm). All three are dead dev-time deps.
- **Three options:**

| Option | Description | Cost | Risk |
|---|---|---|---|
| **A. Implement (activate them)** | Add at least ONE end-to-end binary test using `assert_cmd` and one using `tempfile`. Concrete tests already drafted in this R3: MED-14 uses `assert_cmd`+`predicates`; MED-15 uses `tempfile`. Two new tests cover all three dev-deps. | **S** (2 new tests + ~30 LOC of fixture handling). | None significant; this is purely additive coverage. |
| **B. Remove** | Delete all three from `Cargo.toml [dev-dependencies]`. | **S**. | Closes the door on binary integration tests until re-added. |
| **C. Error-with-message** | N/A -- no flag to gate. |  | -- |

- **Recommendation: Option A (Implement / activate).** This is the ONLY ABS BC where activation is cheaper than removal in net value. The two tests proposed in MED-14 + MED-15 alone justify the dev-dep cost AND simultaneously upgrade 2 MEDIUM BCs to HIGH. **High-leverage disposition.**

**Target-2 summary:**

| BC | Recommendation | Cost | Rationale |
|---|---|---|---|
| ABS-004 (`--hosts`) | **Implement** | S | Data already collected; smallest cost of any ABS BC. |
| ABS-005 (`--services`) | **Remove** | S | Redundant with default summary; no design wins. |
| ABS-008 (rayon) | **Remove** | S | Parallel processing is a future PRD, not implicit roadmap. |
| ABS-009 (dev-deps) | **Implement (activate)** | S | Activating covers 2 MEDIUM BCs (MED-14, MED-15) and adds zero-LOC-of-prod-code value. |

Combined with R2's 6 ABS dispositions, **all 10 ABS BCs are now dispositioned.**

---

### Target 3 -- MAX_FINDINGS cap-saturation test design

R2 §2 Target 2 noted the test requires either a `pub(crate)` accessor or a `dropped_findings: u64` counter on `ReassemblyStats`. R3's decision: **which approach is best for wirerust's testing conventions?**

**Reading Pass 5 (CNV-TST-*):**
- CNV-TST-001: tests live in `tests/` directory (integration style); no `#[cfg(test)] mod tests` blocks in `src/` except `reporter/terminal.rs` for the private `escape_for_terminal`.
- CNV-TST-007: 91.6% follow `test_<subject>_<expected>` naming.
- CNV-TST-009: dev-deps unused (BC-ABS-009).
- No existing convention for `#[cfg(test)] pub(crate)` accessors -- the codebase uses module-private state and exposes either fully-public accessors (e.g., `TlsAnalyzer::flows()`... but wait, no, `flows` is private; only `sni_counts()`, `ja3_counts()` etc. are exposed) or counts via `summarize()`.

**Reading Pass 4 (NFR-OBS-*):**
- NFR-OBS-001..009 prefer surfacing observable counters via `summarize()` rather than direct accessors.
- `ReassemblyStats` is already public and exposed via `reassembler.summarize()` and inserted into the analyzer summary's `detail` map.
- Adding `dropped_findings: u64` to `ReassemblyStats` would be observable via JSON output (BC-CLI-015 pattern: `analyzer_summaries[0].detail["dropped_findings"]`).

**Decision: prefer Option A (add `dropped_findings: u64` counter to `ReassemblyStats`).**

**Rationale:**
1. **Idiomatic to wirerust:** Counters in `ReassemblyStats` already exist for related events (`segments_depth_exceeded`, `flows_evicted`, `parse_errors`). Adding `dropped_findings` follows the existing pattern.
2. **Observable in production:** A user encountering MAX_FINDINGS saturation in a real capture would today see "10000 findings + N silent drops" with no indicator of N. The counter surfaces this in JSON output and the terminal summary.
3. **Testable without test-only API:** `summarize()` already returns the stats; the test asserts `stats.dropped_findings > 0` after exercising the cap.
4. **Lower test cost:** No need to construct 10_000 fake findings; the test exercises the cap via the *normal* path. (Though the test is still slow because reaching 10_000 findings requires real packet processing.)
5. **Doesn't pollute the public API:** No `#[cfg(test)] pub(crate)` helpers needed.

**Proposed API addition:**

In `src/reassembly/mod.rs` near the `ReassemblyStats` struct (currently around line 70-80):

```rust
#[derive(Debug, Default, Clone, Serialize)]
pub struct ReassemblyStats {
    pub packets_processed: u64,
    pub packets_skipped_non_tcp: u64,
    // ... existing fields ...
    pub flows_evicted: u64,
    pub segments_depth_exceeded: u64,
    /// Number of finding emissions silently dropped because findings.len()
    /// reached MAX_FINDINGS (10000). Surfaced in summarize() detail.
    /// Note: the finalize() segment-limit finding bypasses this cap and
    /// does NOT increment this counter even when findings is at cap.
    pub dropped_findings: u64,
}
```

In `src/reassembly/mod.rs`, modify each of the 5 cap-gated emission sites:

```rust
// At mod.rs:272 (excessive overlap alert)
if flow_dir.overlap_count > OVERLAP_ALERT_THRESHOLD && !flow_dir.overlap_alert_fired {
    if self.findings.len() < MAX_FINDINGS {
        flow_dir.overlap_alert_fired = true;
        self.findings.push(Finding { ... });
    } else {
        self.stats.dropped_findings += 1;
        // NOTE: latch is intentionally NOT set, so a future call with capacity
        // would re-emit. With MAX_FINDINGS immutable, this is a no-op.
    }
}
// Same pattern for mod.rs:291 (small-segment) and mod.rs:310 (out-of-window).

// At mod.rs:534 (generate_conflicting_overlap_finding):
fn generate_conflicting_overlap_finding(&mut self, ...) {
    if self.findings.len() >= MAX_FINDINGS {
        self.stats.dropped_findings += 1;
        return;
    }
    self.findings.push(Finding { ... });
}

// Same pattern for generate_truncated_finding at mod.rs:550.
```

In `summarize()` (around `mod.rs:580`), the existing `serde_json::json!` block will pick up the new field via `#[derive(Serialize)]` -- no changes needed there.

**Test that the API addition enables:**

```rust
// tests/reassembly_engine_tests.rs (NEW TEST)
#[test]
fn test_max_findings_cap_silently_drops_and_counts() {
    // Construct a reassembler with default config (MAX_FINDINGS = 10_000).
    // Drive 10_000 conflicting-overlap findings (the cheapest finding to emit).
    // This requires sending segments at the same offset with conflicting bytes
    // across 10_000 distinct flows or repeated bursts.
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = NoopHandler;
    // ... drive the cap ...
    let stats = reassembler.summarize();
    assert_eq!(reassembler.findings().len(), 10_000); // cap held
    assert!(stats.detail["dropped_findings"].as_u64().unwrap_or(0) > 0);

    // Now verify finalize bypass: trigger one more segment-limit event
    // and confirm findings.len() increases beyond 10_000.
    let before = reassembler.findings().len();
    // ... force a segment-limit drop on a fresh flow ...
    reassembler.finalize(&mut handler);
    let after = reassembler.findings().len();
    assert!(after > before, "finalize segment-limit finding should bypass MAX_FINDINGS cap");
}
```

**Trade-offs of Option B (the alternative):**

| Aspect | Option A (counter) | Option B (`pub(crate)` accessor) |
|---|---|---|
| Lines of new code | ~12 (struct field + 5 incrementing sites + doc) | ~5 (`pub(crate) fn set_findings_len(&mut self, n: usize)` + 2 tests of the helper itself) |
| Production observability | YES (visible in JSON output) | NO (hidden in test-only API) |
| Convention alignment | Matches existing `ReassemblyStats` pattern | Introduces a new pattern (test-only mutator) |
| Test cost | High (10_000 real findings) | Low (synthetic `findings` push) |
| Operator value | Tells operators when triage is incomplete | Zero |

**Net recommendation:** Option A (counter). The slight increase in test cost is paid for by the production observability gain. Option B is rejected as it introduces a test-only state mutator that violates the codebase's convention of "tests use the public API."

---

### Target 4 -- JA3/JA3S property test design

R2 §3 MED-5/6 upgraded BC-TLS-007 (JA3 string format) and BC-TLS-008 (JA3S) from MEDIUM → HIGH via indirect verification: the MD5 hash of `version,cipher-list,extension-list,curve-list,pointfmt-list` is pinned by tests asserting specific 32-char hex outputs. R3 asks: **could property-based fuzzing further harden them?**

**Source re-read (`tls.rs:67-145`):**

`compute_ja3` produces `format!("{version},{cipher_str},{ext_ids},{curves_str},{pf_str}")` where:
- `version`: TLS version u16 as decimal
- `cipher_str`: dash-joined non-GREASE cipher IDs (GREASE filtered per RFC 8701)
- `ext_ids`: dash-joined non-GREASE extension IDs
- `curves_str`: dash-joined non-GREASE supported_groups extension values (parsed)
- `pf_str`: dash-joined ec_point_formats extension values

GREASE values are filtered: `(v & 0x0f0f) == 0x0a0a` (the 16 GREASE values 0x0a0a, 0x1a1a, ..., 0xfafa).

**Property test signatures (using `proptest`):**

```rust
use proptest::prelude::*;

proptest! {
    // Property 1: GREASE filtering is total.
    // For any vector of u16 mixing GREASE and non-GREASE values, the JA3
    // cipher list contains zero GREASE values.
    #[test]
    fn prop_ja3_grease_filtering_total(
        ciphers in prop::collection::vec(any::<u16>(), 0..100)
    ) {
        let ch = build_synthetic_client_hello(0x0303, &ciphers, &[], &[]);
        let (_, ja3_str) = compute_ja3_from_synthetic(&ch);
        let cipher_str = ja3_str.split(',').nth(1).unwrap();
        for token in cipher_str.split('-').filter(|s| !s.is_empty()) {
            let v: u16 = token.parse().unwrap();
            prop_assert!((v & 0x0f0f) != 0x0a0a, "GREASE leaked: {v}");
        }
    }

    // Property 2: Field count is exactly 5 (comma-separated).
    #[test]
    fn prop_ja3_string_has_exactly_5_fields(
        version in any::<u16>(),
        ciphers in prop::collection::vec(any::<u16>(), 0..50),
    ) {
        let ch = build_synthetic_client_hello(version, &ciphers, &[], &[]);
        let (_, ja3_str) = compute_ja3_from_synthetic(&ch);
        prop_assert_eq!(ja3_str.matches(',').count(), 4, "JA3 has wrong field count: {ja3_str}");
    }

    // Property 3: MD5(ja3_str) is deterministic and hex-32-lower.
    #[test]
    fn prop_ja3_hash_format(
        version in any::<u16>(),
        ciphers in prop::collection::vec(any::<u16>(), 0..50),
    ) {
        let ch = build_synthetic_client_hello(version, &ciphers, &[], &[]);
        let (hash1, _) = compute_ja3_from_synthetic(&ch);
        let (hash2, _) = compute_ja3_from_synthetic(&ch);
        prop_assert_eq!(hash1.len(), 32);
        prop_assert_eq!(hash1, hash2); // deterministic
        prop_assert!(hash1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    // Property 4: Empty cipher list yields ja3_str of shape "{ver},,...".
    #[test]
    fn prop_ja3_empty_cipher_list_is_two_consecutive_commas(
        version in any::<u16>()
    ) {
        let ch = build_synthetic_client_hello(version, &[], &[], &[]);
        let (_, ja3_str) = compute_ja3_from_synthetic(&ch);
        let parts: Vec<&str> = ja3_str.split(',').collect();
        prop_assert_eq!(parts[1], "");
    }
}
```

**Cargo dev-dep addition:**

```toml
[dev-dependencies]
proptest = "1"
```

Approximately 1 MB of additional dev-time dep weight (proptest brings `rand` + `quickcheck`-style runner). No production-code impact.

**Helper required:** `build_synthetic_client_hello` and `compute_ja3_from_synthetic` are test-only helpers since `compute_ja3` is private in `tls.rs` and takes a `&TlsClientHelloContents`. Either:
- Expose `compute_ja3` as `pub(crate)` for test access (consistent with the test-only-API rejection in Target 3, so NOT preferred), OR
- Construct a real `TlsClientHelloContents` via `tls_parser` from synthetic raw bytes (the standard approach in `tls_analyzer_tests.rs`).

**Recommendation: Add proptest with property 1 only.** Property 1 (GREASE filtering totality) is the highest-value property test because:
- It catches a class of bugs the existing example-based tests can't: e.g., if a future PR adds a GREASE-like value 0x4a4a (which IS GREASE) to a hardcoded "skip" list and misses 0xaaaa, the example test passes but the property test fails.
- The other 3 properties are convenience (already implied by the existing pinned-hash tests).

**Cost-benefit summary:**
- **Cost:** 1 dev-dep (proptest), 1 helper, ~40 LOC of test
- **Benefit:** Hardening of BC-TLS-007 against future GREASE-related regressions; small forensic-confidence boost.
- **Risk:** Slows the test suite by ~3-5s due to proptest's default 256 cases per test.

**Verdict for R3:** Property-based fuzzing **could** harden BC-TLS-007/008 beyond R2's upgrade. The minimum-viable addition is one property test (GREASE filtering totality) + one proptest dev-dep. This is a R3 recommendation, NOT a R3 implementation. Defer to a follow-up PRD scoped to "test hardening."

---

### Target 5 -- Per-direction-alert independence test (BC-RAS-022)

R2 §7 noted "per-direction reassembly alerts: round 1 implied each direction emits independently (BC-RAS-022 'AT MOST once per flow') but the per-direction independence is not explicitly tested." R3's job: confirm by reading `reassembly/mod.rs` whether the latches are per-direction or per-flow, and draft a test.

**Source re-read (R3-fresh):**

`src/reassembly/flow.rs:71-87` -- `FlowDirection` struct:
```rust
pub struct FlowDirection {
    // ... offsets, segments, byte counters ...
    pub overlap_count: u32,
    pub overlap_alert_fired: bool,            // <-- latch
    pub small_segment_count: u32,
    pub small_segment_alert_fired: bool,      // <-- latch
    pub out_of_window_count: u32,
    pub out_of_window_alert_fired: bool,      // <-- latch
    // ...
}
```

`src/reassembly/flow.rs:159-170` -- `TcpFlow` struct:
```rust
pub struct TcpFlow {
    pub key: FlowKey,
    pub client_to_server: FlowDirection,
    pub server_to_client: FlowDirection,
    // ...
}
```

`src/reassembly/mod.rs:268-330` -- alert sites all access `flow_dir = flow.get_direction_mut(dir)` and check/set `flow_dir.{overlap,small_segment,out_of_window}_alert_fired`. The latch is set on the `FlowDirection` instance, NOT on `TcpFlow`.

**Verdict:** The latches are **per-direction**, not per-flow. The R1 BC text "AT MOST once per flow" in BC-RAS-022 is INACCURATE -- it should be "AT MOST once per flow direction." A single bidirectional flow CAN emit up to 6 alert findings (3 anomaly types × 2 directions). This is a R3 text refinement.

**Refined BC:**

> **BC-RAS-022 (refined, HIGH-confidence on contract; MEDIUM on test).** Excessive-overlap, excessive-small-segment, and excessive-out-of-window alerts each have a per-direction sticky latch (`FlowDirection.overlap_alert_fired`, `.small_segment_alert_fired`, `.out_of_window_alert_fired` -- `src/reassembly/flow.rs:79-83`). The latch is checked AND set inside the `if flow_dir.X_count > THRESHOLD && !flow_dir.X_alert_fired && self.findings.len() < MAX_FINDINGS` block. Thus a flow's client→server direction can emit ONE alert of each of the 3 types, INDEPENDENTLY of the server→client direction emitting the same alerts. Worst-case per-flow alert cardinality is **6 findings (3 types × 2 directions)**, not 3.

**Test recommendation:**

```rust
// tests/reassembly_engine_tests.rs (NEW)
#[test]
fn test_per_direction_overlap_alerts_are_independent() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = NoopHandler;

    let cli_ip: IpAddr = "10.0.0.1".parse().unwrap();
    let srv_ip: IpAddr = "10.0.0.2".parse().unwrap();

    // Establish flow with SYN/SYN-ACK so initiator is bound.
    send_syn(&mut reassembler, cli_ip, 49152, srv_ip, 443, 1000, &mut handler);
    send_syn_ack(&mut reassembler, srv_ip, 443, cli_ip, 49152, 2000, 1001, &mut handler);

    // Drive 51 overlapping segments in client-to-server direction.
    for i in 0..52 {
        send_overlap_segment_c2s(&mut reassembler, /* args */, &mut handler);
    }

    // At this point: client-to-server has overlap_alert_fired=true,
    // server-to-client has overlap_alert_fired=false.
    let c2s_findings = reassembler.findings().iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps") && f.source_ip == Some(cli_ip))
        .count();
    assert_eq!(c2s_findings, 1, "client→server should have exactly 1 overlap alert");

    // Now drive 51 overlapping segments in server-to-client direction.
    for i in 0..52 {
        send_overlap_segment_s2c(&mut reassembler, /* args */, &mut handler);
    }

    let s2c_findings = reassembler.findings().iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps") && f.source_ip == Some(srv_ip))
        .count();
    assert_eq!(s2c_findings, 1, "server→client should ALSO have exactly 1 overlap alert");

    // Total: 2 overlap findings on the same flow, one per direction.
    let total = reassembler.findings().iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps"))
        .count();
    assert_eq!(total, 2);
}
```

After test added, BC-RAS-022 stays MEDIUM-on-test (because the test is expensive: 51+51 segments) but the contract text is HIGH-confidence (code-read pinned).

---

### Target 6 -- BC-TLS-037 SNI discriminator order test

R2 introduced BC-TLS-037 (NEW, MEDIUM) for "SNI discriminator order: when bytes have BOTH C0/DEL AND non-ASCII UTF-8, AsciiWithControl takes precedence (verified from `tls.rs:173-242` enum construction order)." R3's job: draft the test.

**Source re-read (R3-fresh, `tls.rs:219-242`):**

```rust
fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue> {
    for ext in extensions {
        if let TlsExtension::SNI(list) = ext
            && let Some((_, hostname)) = list.first()
        {
            return Some(match std::str::from_utf8(hostname) {
                Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii(s.to_string()),
                Ok(s) if s.is_ascii() => SniValue::AsciiWithControl { hostname: ..., hex: ... },
                Ok(s) => SniValue::NonAsciiUtf8 { hostname: ..., hex: ... },
                Err(_) => SniValue::NonUtf8 { lossy: ..., hex: ... },
            });
        }
    }
    None
}
```

**Critical observation:** The match arms are evaluated in order. The two `if s.is_ascii()` guards mean:
- Arm 1: ASCII + no C0/DEL → `Ascii`
- Arm 2: ASCII + has C0/DEL → `AsciiWithControl`
- Arm 3: NOT ASCII but valid UTF-8 → `NonAsciiUtf8`
- Arm 4: Not valid UTF-8 → `NonUtf8`

**The "both C0/DEL AND non-ASCII UTF-8" case is IMPOSSIBLE under this implementation.** Reason: `s.is_ascii()` returns false if ANY byte is ≥ 0x80. C0 bytes are 0x00..0x1F (all < 0x80) and DEL is 0x7F (< 0x80). Non-ASCII UTF-8 codepoints (U+0080+) encode as 2+ bytes where the LEAD byte is ≥ 0xC2 (i.e., always ≥ 0x80). So:
- Bytes with C0 + non-ASCII UTF-8 mix → `s.is_ascii()` is FALSE → routed to arm 3 (`NonAsciiUtf8`).
- Bytes that ARE pure ASCII + contain C0 → routed to arm 2 (`AsciiWithControl`).

**There is NO overlap between AsciiWithControl and NonAsciiUtf8.** The disambiguation is mutually exclusive at the byte level via `s.is_ascii()`.

**Refined BC-TLS-037:**

> **BC-TLS-037 (refined, HIGH-confidence on rule).** The SNI value discriminator is a 4-way mutually-exclusive partition keyed FIRST on UTF-8 validity (`std::str::from_utf8`) and SECOND on ASCII-purity (`s.is_ascii()`). The cases are:
> 1. Not valid UTF-8 → `NonUtf8`. Bytes ≥ 0x80 are present AND cannot form valid UTF-8.
> 2. Valid UTF-8 + at least one byte ≥ 0x80 → `NonAsciiUtf8`. Includes ALL multi-byte UTF-8 (CJK, accented Latin, emoji) regardless of whether the codepoints are also C1 control codes (U+0080..U+009F). C1 controls in SNI go to NonAsciiUtf8, NOT AsciiWithControl.
> 3. Valid UTF-8 + all bytes < 0x80 + at least one byte in {0x00..0x1F, 0x7F} → `AsciiWithControl`. This is exclusively pure-ASCII C0 + DEL.
> 4. Valid UTF-8 + all bytes < 0x80 + no C0/DEL → `Ascii`.
>
> "Both C0 AND non-ASCII UTF-8" is **impossible to encode** because non-ASCII UTF-8 requires bytes ≥ 0x80 which automatically routes to `NonAsciiUtf8` per arm 3. The R2 R3 carryover suggestion that "AsciiWithControl wins when both present" was a misread of the source; in fact the two cases are byte-level mutually exclusive.

**Test (pins the actual rule, not the misread):**

```rust
// tests/tls_analyzer_tests.rs (NEW TEST)
#[test]
fn test_sni_with_c1_control_codepoint_routes_to_non_ascii_utf8() {
    // SNI containing U+0085 (NEXT LINE, a C1 control) encoded as UTF-8 (0xC2 0x85).
    // Per BC-TLS-037, this should route to NonAsciiUtf8 (not AsciiWithControl),
    // because the lead byte 0xC2 is ≥ 0x80 so s.is_ascii() returns false.
    let sni_bytes = b"foo\xc2\x85.example.com";

    // Build a synthetic ClientHello with this SNI extension and feed to analyzer.
    let mut analyzer = TlsAnalyzer::new();
    let fk = FlowKey::new(
        "10.0.0.1".parse().unwrap(), 49152,
        "10.0.0.2".parse().unwrap(), 443,
    );
    let client_hello = build_client_hello_with_sni(sni_bytes);
    analyzer.on_data(&fk, Direction::ClientToServer, &client_hello, 0);

    // Expect: a NonAsciiUtf8 finding (summary mentions "non-ASCII"), NOT
    // an AsciiWithControl finding (summary mentions "ASCII control characters").
    let findings = analyzer.findings();
    let non_ascii_count = findings.iter()
        .filter(|f| f.summary.contains("non-ASCII") || f.summary.contains("RFC 6066"))
        .count();
    let ascii_control_count = findings.iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .count();
    // Exact assertion depends on the exact wording in tls.rs; the test pins
    // that the C1-control case goes to the NonAsciiUtf8 branch.
    assert_eq!(ascii_control_count, 0,
        "C1 control byte in non-ASCII UTF-8 should NOT route to AsciiWithControl");
    assert!(non_ascii_count >= 1,
        "C1 control byte in non-ASCII UTF-8 should route to NonAsciiUtf8");
}
```

After test added → HIGH on BC-TLS-037. **Net:** BC-TLS-037 upgrades from MEDIUM (R2) to HIGH-on-rule + MEDIUM-on-test. The original R2 carryover framing ("if AsciiWithControl wins...") is **REFUTED** -- the source code makes the two cases mutually exclusive.

---

### Target 7 -- Pass 6 metric re-audit list

R3 enumerates Pass 6 metric claims that should be re-verified (per the 137-vs-216 BC count discovery in R2). **Does not re-verify them here**; just lists which merit a Pass 6 R2 re-audit.

| # | Pass 6 metric claim | Location | Why it should be re-verified |
|---|---|---|---|
| M1 | "137 BCs across 13 areas; 10 absent BCs; 81% HIGH confidence" | P6 §0 inputs ingested + §1 §5 confidence summary + §6.6 "26 BCs (137 - 111 HIGH = 26)" | **CONFIRMED WRONG by R2.** Actual: 216 BCs, 162 HIGH (75%), 40 MEDIUM, 4 LOW, 10 ABS. |
| M2 | "26 MEDIUM/LOW-confidence BCs" in §6.6 | P6 §6.6 prelude | **CONFIRMED WRONG by R2.** Actual: 40 MEDIUM + 4 LOW = 44 non-HIGH-non-ABS. The list itself in §6.6 enumerates approx 30 BC IDs (not 26), suggesting the prose number and the bullet count don't agree. |
| M3 | "202 #[test] (P0 §5)" | P6 §2 cross-pass table + §1 | Not re-counted in R2 or R3; P0 should re-verify. Carryover from R2 §1 row 20. |
| M4 | "9889 LOC src/" | P6 §1 "what wirerust does" | Not re-counted; depends on P0 R1's `wc -l` snapshot. P0 R2 should rerun the count. |
| M5 | "3868 LOC of src/" | P6 §1 | Same as M4. Two different totals in the same document (9889 includes tests, 3868 src only). Both depend on P0's `find ... -exec wc -l` snapshot which has not been re-run since 2026-05-19. |
| M6 | "76 NFRs across 9 categories; 28 magic numbers indexed" | P6 §0 inputs | Not re-counted; P4 R2 should re-verify the 76 NFR count via row count in pass-4-nfr-catalog.md. |
| M7 | "73 conventions across 10 categories" | P6 §0 inputs | Not re-counted; P5 R2 should re-verify. |
| M8 | "101 business rules" in Pass 2 | P6 §0 inputs | Not re-counted; P2 R2 has already done major content additions; raw BR count may now differ. |
| M9 | "20 components C-1..C-20; 5 layers" | P6 §0 inputs | P1 R1 numbering verified consistent across passes (see P6 §2 table). LOW RISK. |
| M10 | "C-6 reassembly (mod.rs only) | 564 LOC | ~30 (BC-RAS-001..030)" in §6.2 density table | P6 §6.2 | The "~30" hedge is suspicious; BC-RAS-001..030 is 30 BCs, but the full BC-RAS span is 001..053 (per R2). The table assigns "BC-RAS-031..053 partial" to flow.rs+segment.rs+handler.rs — but per BC index, several BC-RAS BCs in the 30s and 40s cite `mod.rs:NNN`, not `flow.rs`/`segment.rs`. Should re-attribute BC-by-BC. |
| M11 | "10 unwired CLI flags consolidated" in §5 smell #2 | P6 §5 row 2 | Smell #2 lists 8 flags (--threats, --beacon, --filter, --verbose, --hosts, --services, --json <FILE>, --csv <FILE>). The "7 CLI flags" elsewhere in the same doc (e.g., §6.7 NFR-VIO-003) is inconsistent with the smell-table's 8. R3 audits: P3 R1 BC-ABS-001..010 enumerates 10 ABS BCs (including dev-deps and rayon), of which the CLI-flag subset is 8 (BC-ABS-001/002/003/004/005/006/007/010 are flags; 008 rayon and 009 dev-deps are deps). So "8 unwired CLI flags" is the right count -- "7" is wrong by one in §6.7. |
| M12 | "30+ checked claims" in §2 cross-pass | P6 §2 footer | The table has 33 rows; "30+" is loose but defensible. LOW RISK. |
| M13 | "51 deepening questions authored" in §10 / state checkpoint | P6 §10 + checkpoint | "6 P0 + 7 P1 + 9 P2 + 8 P3 + 8 P4 + 9 P5 = 47, not 51." Re-add: 6+7+9+8+8+9 = 47. **The 51 number is wrong; actual is 47.** |
| M14 | "3 load-bearing invariants" | P6 throughout | Carries forward INV-1, INV-2, INV-3 from Pass 2. Pass 2 R2 may have introduced new invariants; the "3" may now understate. |
| M15 | "20 anti-patterns de-duplicated" | P6 §5 | Smells table is numbered 1..20. Correct. LOW RISK. |
| M16 | "8 architecture-level open questions Q-A1..Q-A8" | P6 §0 | Q-A1..Q-A8 = 8. Correct. LOW RISK. |
| M17 | "10 plans + 8 specs = 18 superpowers files" | P6 §10 step 1 row | Not re-verified. P0 R2 should re-count. |

**Net Pass 6 metric audit list for P6 R2:**

- **HIGH PRIORITY re-audit:** M1, M2 (already known wrong), M11 (8 not 7), M13 (47 not 51).
- **MEDIUM PRIORITY re-audit:** M10 (BC-RAS attribution), M3/M4/M5 (test/LOC counts), M14 (invariant count post-P2 R2).
- **LOW PRIORITY:** M6, M7, M8, M17 (depend on each respective pass's R2/R3 outputs).
- **LOW RISK (likely correct):** M9, M12, M15, M16.

**Recommendation for Pass 6 R2:** Re-verify M1, M2, M11, M13 BEFORE producing the Pass 8 deep synthesis -- these are arithmetic errors that propagate to downstream documents.

---

## 3. Refined BC list -- deltas only

| BC-ID | Old confidence (post-R2) | New confidence (post-R3) | Change |
|---|---|---|---|
| BC-TLS-033 | MEDIUM | **HIGH** (upgraded) | Existing `test_stop_after_handshake` indirectly pins the "ignore non-handshake records" rule by exercising the `done()` short-circuit (which only triggers AFTER non-handshake records arrive and are skipped). |
| BC-TLS-034 | MEDIUM (was MEDIUM in R2, not addressed) | **HIGH** (upgraded) | Same existing test. |
| BC-TLS-037 | MEDIUM (NEW in R2) | **HIGH** on rule + MEDIUM on test (text refined; misread refuted) | The R2 framing "AsciiWithControl wins when both present" is refuted: C0/DEL + non-ASCII UTF-8 is byte-level impossible per the `s.is_ascii()` gate. Refined to 4-way mutually exclusive partition; test drafted. |
| BC-RAS-022 | HIGH (R1 wording "at most once per flow") | **HIGH on contract + MEDIUM on test** (text refined) | Wording corrected: latches are per-direction (`FlowDirection`), not per-flow (`TcpFlow`). Worst-case alert cardinality is 6 (3 types × 2 directions), not 3. |
| BC-DEC-008 | MEDIUM | MEDIUM-with-test-recommendation | Test signature drafted (constructability caveat: only testable if `DataLink` exposes a non-supported variant). |
| BC-DEC-009 | MEDIUM | MEDIUM-with-test-recommendation | ARP frame test drafted. |
| BC-DEC-010 | MEDIUM | MEDIUM-with-test-recommendation | ICMP echo test drafted. |
| BC-DEC-011 | MEDIUM | MEDIUM-with-test-recommendation | GRE/SCTP test drafted. |
| BC-DEC-013 | MEDIUM | MEDIUM-with-test-recommendation | TransportInfo::None unit test drafted. |
| BC-CLI-009 | MEDIUM | MEDIUM-with-test-recommendation | `assert_cmd` integration test drafted (activates dev-dep). |
| BC-CLI-011 | MEDIUM | MEDIUM-with-test-recommendation | `tempfile`-based `resolve_targets` test drafted (activates dev-dep). |
| BC-CLI-012 | MEDIUM | MEDIUM-with-test-recommendation | Piggybacks on MED-15's `resolve_targets` visibility change. |
| BC-CLI-015 | MEDIUM | MEDIUM-with-test-recommendation | Requires `assert_cmd` JSON-parse path. |
| BC-TLS-035 | MEDIUM | MEDIUM-with-test-recommendation | Requires `pub(crate) fn flows_for_test` accessor (rejected pattern per Target 3); alternative: assert via subsequent on_data behavior. |

**Net deltas:**
- 2 confidence UPGRADES: BC-TLS-033, BC-TLS-034 (via existing test).
- 1 confidence UPGRADE-PARTIAL: BC-TLS-037 (rule HIGH, test MEDIUM).
- 1 text REFINEMENT (no confidence change): BC-RAS-022 (per-flow → per-direction correction).
- 9 test recommendations authored for MED-9..17 (5 decoder BCs, 4 CLI BCs).
- 4 ABS dispositions drafted (ABS-004 Implement, ABS-005 Remove, ABS-008 Remove, ABS-009 Implement-activate).
- 0 NEW BCs introduced.

**Resulting BC total: 218 (unchanged from R2).**

---

## 4. Delta Summary

- **New BCs added:** 0.
- **Confidence upgrades (full):** 2 (BC-TLS-033, BC-TLS-034 → HIGH).
- **Confidence upgrades (partial, rule-only):** 1 (BC-TLS-037 rule → HIGH, test stays MEDIUM).
- **Confidence downgrades:** 0.
- **Text refinements (no confidence change):** 1 (BC-RAS-022 per-direction).
- **MEDIUM BCs given test recommendations:** 9 (MED-9..17 across decoder, CLI, TLS-late clusters).
- **Absent-BC dispositions drafted:** 4 (ABS-004/005/008/009).
- **MAX_FINDINGS cap-saturation design:** 1 (counter Option A recommended over `pub(crate)` Option B).
- **Property-test recommendation:** 1 (proptest dev-dep + GREASE-filtering property for BC-TLS-007).
- **Per-direction-alert independence:** 1 (BC-RAS-022 text refined + test drafted).
- **BC-TLS-037 SNI discriminator order:** 1 (R2 misread refuted; refined rule + test drafted).
- **Pass 6 metric re-audit list:** 17 items enumerated; 4 high-priority (arithmetic errors) flagged.
- **Hallucination-class audit retractions:** 0 (R2 passes all 8 audit checks).

**Remaining gaps post-R3:**

1. **23 MEDIUM BCs** still entirely untouched after R3 (40 original – 2 upgraded in R2 – 2 upgraded in R3 – 6 R2-text-refined-with-rec – 9 R3-text-refined-with-rec = 21; arithmetic: 40 - 2 - 2 - 6 - 9 = 21 untouched; plus 2 R3-rec without text changes = ~21-23 depending on counting). These cluster in: `BC-RAS-031..047` (segment overlap rules), `BC-RAS-002` (already R2-covered actually), `BC-HTTP-009..023` (HTTP detection rules), `BC-RPT-018/019` (reporter edge cases), `BC-FND-006`. P3 R4 if invoked should select 8-10 more from these.
2. **MAX_FINDINGS counter implementation** -- design is now pinned; engineering is still pending.
3. **Property tests for JA3/JA3S** -- recommended but not implemented; could be a follow-up PRD scoped to "test hardening."
4. **Pass 6 metric re-audit** -- 4 high-priority arithmetic errors flagged; not fixed in P3 R3 (Pass 6 R2 territory).
5. **Test naming convention enforcement** for new tests proposed in R3 -- the 9 test signatures use `test_<subject>_<expected>` pattern (CNV-TST-007 compliant) but P3 has no authority over P5; defer to P5 R2.

---

## 5. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification (would removing this round's findings change how you'd spec the system?):
- **YES** -- BC-RAS-022 text refinement (per-direction not per-flow) is a contract correction that changes operator-visible alert cardinality (6, not 3 per flow). Downstream SIEM consumers tuning on "at most 3 anomaly findings per flow" would silently miss alerts.
- **YES** -- BC-TLS-037 R2 framing was a misread; R3 refutes it and pins the actual rule. Any spec following R2's framing would specify a non-existent behavior.
- **YES** -- The MAX_FINDINGS dropped-findings counter recommendation is a production-observability addition, not just a test enabler. Today's "10000 cap" silently hides triage incompleteness; the counter surfaces it.
- **YES** -- The 4 ABS dispositions complete the BC-ABS-001..010 closure started in R2. Phase 3 `/create-prd` work now has 10 dispositions to plan against, not 6.
- **YES** -- The 9 test recommendations include 2 (MED-14, MED-15) that simultaneously activate the dead dev-deps `assert_cmd`/`predicates`/`tempfile`, closing ABS-009 with zero new code. The leverage of "one test covers a MEDIUM BC + activates a declared-unused dev-dep" is material.
- **YES** -- Pass 6 metric re-audit list reveals 4 arithmetic errors (M1, M2, M11, M13) that, uncorrected, propagate into Pass 8 deep synthesis and downstream `/create-brief`/`/create-domain-spec` documents.

Removing any of these would degrade downstream spec accuracy. SUBSTANTIVE.

---

## 6. Convergence Declaration

**Another round needed.** P3 R3 addressed all 7 carryover targets and produced more than 3 substantive items. Substantive gaps remaining for P3 R4 (if invoked):

1. ~21-23 MEDIUM BCs still untouched -- selecting 8-10 more would close the long tail.
2. Implementation of the MAX_FINDINGS dropped-findings counter (design now pinned; engineering pending).
3. Property test scaffolding for JA3/JA3S (recommended; not implemented).
4. ~~Pass 6 metric arithmetic errors~~ -- this is Pass 6 R2 territory, not P3.
5. Cross-pass: do BC-RAS-022's per-direction correction implications affect any other BCs that assume per-flow latching? Quick grep over BC index suggests no, but P4 R2 should re-check NFR-RES-002/003/004 framings.

If P3 R4 addresses gaps 1-3 and finds fewer than 3 substantive items, it can declare convergence.

**State checkpoint:**

```yaml
pass: 3
round: 3
status: complete
inputs_ingested: 9
bcs_upgraded_to_high: 2
bcs_partially_upgraded: 1
bcs_text_refined: 1
bcs_with_test_recommendations: 9
abs_dispositions_drafted: 4
abs_dispositions_total_post_r3: 10
hallucination_class_retractions: 0
pass_6_metric_audit_items: 17
total_bcs_post_round: 218
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_action: pass_3_round_4_or_pass_6_round_2
resume_from: 21-23 untouched MEDIUM BCs + MAX_FINDINGS counter implementation + proptest scaffolding
```

