---
artifact: L2-cap-12
traces_to: ../domain-spec.md
cap_id: CAP-12
title: CLI Orchestration / Entry Point
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
anchor_justification: >
  CAP-12 covers the CLI orchestration / entry-point concern because main.rs
  and cli.rs implement the full user-facing surface (flag parsing, subcommand
  routing, per-target file expansion, the per-packet processing loop,
  Summary accumulation, and output-channel selection) -- none of which is
  covered by CAP-01 (ingestion: reading bytes from disk), CAP-11 (reporting:
  rendering output strings), or any other capability. Grounded in the
  product brief "single-binary, single-pass" identity (domain-spec.md
  section 1) and the L0 Entry layer description (domain-spec.md section 2).
---

# CAP-12: CLI Orchestration / Entry Point

## What the system does today

`main.rs` (C-1) and `cli.rs` (C-3) form the L0 entry layer. They are the
sole owners of argument parsing, subcommand dispatch, target resolution,
the per-target packet loop, `Summary` accumulation, and output routing.
`lib.rs` (C-2) is the crate root that re-exports the public module tree.

**Sources:** C-1 main.rs, C-2 lib.rs, C-3 cli.rs, C-17 summary.rs.
BC-CLI-*, BC-SUM-*.

**Scope note:** CAP-12 coordinates the other capabilities. It calls
CAP-01 (read packets), CAP-03 (decode), CAP-04/CAP-05 (reassembly +
dispatch), CAP-06..CAP-08 (analyzers), CAP-09..CAP-10 (finding emission
+ MITRE), and CAP-11 (reporter). It does NOT duplicate any of those
capabilities -- it is the wiring layer only.


## CLI argument parsing (cli.rs / C-3)

`Cli` is a clap-derive struct. Parsing is invoked via `Cli::parse()` at
main() entry. Two subcommands are declared:

```
Commands::Analyze { targets, dns, http, tls, all, mitre }
Commands::Summary { targets, hosts }
```

### Global flags (apply to both subcommands)

| Flag | Type | Default | Effect |
|---|---|---|---|
| --no-color | bool | false | disables ANSI color in TerminalReporter |
| --output-format | Option<OutputFormat> | None (terminal) | selects json or csv renderer |
| --json [FILE] | Option<Option<PathBuf>> | None | forces Json reporter; routes output to file if path given |
| --csv [FILE] | Option<Option<PathBuf>> | None | forces Csv reporter; mutually exclusive with --json |
| --reassemble | bool | false | forces reassembly on even when no stream analyzer is enabled |
| --no-reassemble | bool | false | forces reassembly off; warns if --http or --tls also given |
| --reassembly-depth | usize | 10 (MB) | per-direction stream buffer limit |
| --reassembly-memcap | usize | 1024 (MB) | global reassembly memory cap |
| --overlap-threshold | Option<u32> | None (50) | overrides ReassemblyConfig overlap_alert_threshold |
| --small-segment-threshold | Option<u32> | None (100) | overrides small_segment_alert_threshold |
| --small-segment-max-bytes | Option<u16> | None (16) | overrides small_segment_max_bytes |
| --small-segment-ignore-ports | Option<Vec<u16>> | None ([23,513]) | overrides exempt ports |
| --out-of-window-threshold | Option<u32> | None (100) | overrides out_of_window_alert_threshold |

LESSON-P1.04 ("no unwired flags"): every flag declared in `Cli` must be
consumed in main.rs. Five previously-unwired flags (--verbose, --threats,
--beacon, --filter, --services) were removed by this lesson. --hosts on
Summary was wired to gate per-host terminal output (LESSON-P1.03).

### Analyze-subcommand flags

| Flag | Type | Effect |
|---|---|---|
| targets | Vec<PathBuf> | one or more files or directories |
| --dns | bool | enables DnsAnalyzer |
| --http | bool | enables HttpAnalyzer + reassembly |
| --tls | bool | enables TlsAnalyzer + reassembly |
| --all / -a | bool | shorthand for --dns --http --tls |
| --mitre | bool | enables MITRE tactic-grouped output in TerminalReporter |

### Summary-subcommand flags

| Flag | Type | Effect |
|---|---|---|
| targets | Vec<PathBuf> | one or more files or directories |
| --hosts | bool | expands terminal "Hosts: N" into itemized per-IP list |

### --json / --csv mutual exclusion

clap enforces `conflicts_with = "csv"` on --json. Supplying both flags is
a parse-time error before any I/O occurs.


## Target / path expansion (resolve_targets in main.rs)

`resolve_targets(target: &Path)` is called for every element of `targets`
before packet processing:

1. If the path is a file: return it as-is (no extension filter applied).
2. If the path is a directory: read all entries; keep only files whose
   extension is exactly `pcap`. Extension `pcapng` is currently excluded
   (LESSON-P0.02: original behavior). **F2 note (2026-06-19):** pcapng is now
   SUPPORTED by reader.rs (BC-2.01.009). STORY-127 will update this glob to
   include `*.pcapng`. Until STORY-127 lands, pcapng files must be passed
   explicitly. Results are sorted alphabetically for deterministic ordering.
3. If the path is neither file nor directory: bail with
   "Target not found: <path>".

The pcap-only directory filter is a guard, not an error: non-.pcap files
in a directory are silently skipped.


## Per-target packet processing loop (run_analyze)

For each resolved pcap file, main.rs:

1. Calls `PcapSource::from_file(path)` -- CAP-01.
2. Constructs an `indicatif::ProgressBar` sized to `source.packets.len()`.
3. Iterates over `source.packets` (Vec<RawPacket>) sequentially:
   a. `decode_packet(&raw.data, source.datalink)` -- CAP-03.
   b. On decode success: `summary.ingest(&parsed)` (see Summary below).
   c. If --dns active: `dns_analyzer.analyze(&parsed)` -- CAP-08.
   d. If reassembler active: `reasm.process_packet(...)` -- CAP-04/CAP-05.
   e. On decode failure: first failure logs to stderr; subsequent failures
      are silently counted into `total_decode_errors`.
4. After the loop: `summary.skipped_packets = total_decode_errors`.

The loop is wrapped in an immediately-invoked closure so a bail (e.g.
unreadable file mid-loop) does not skip the mandatory `reasm.finalize()`
call below. See LESSON-P0.03.


## Reassembler finalization guarantee

After the per-target loop (whether or not it bailed), if a reassembler
was constructed:

```
reasm.finalize(&mut dispatcher);
all_findings.extend(reasm.findings());
```

`finalize()` flushes remaining open flows and emits the segment-limit
summary Finding. It must be called explicitly; `impl Drop` only logs a
warning eprintln if the object is dropped un-finalized (added P0.03 /#72).

After finalize, `capture_result?` propagates any loop error.


## run_summary pipeline

`run_summary` is a lighter variant:

1. resolve_targets -> PcapSource::from_file for each path.
2. For each raw packet: decode_packet -> summary.ingest. No reassembly,
   no analyzers.
3. summary.skipped_packets = total_decode_errors.
4. resolve_format -> reporter -> write_output.

No ProgressBar is constructed in run_summary.


## Summary accumulation (summary.rs / C-17)

`Summary` (E-36) maintains rolling capture-level totals.

`Summary::ingest(&mut self, packet: &ParsedPacket)`:
- `total_packets += 1`
- `total_bytes += packet.packet_len as u64`
- `hosts.insert(packet.src_ip)` and `hosts.insert(packet.dst_ip)`
- `protocols.entry(packet.protocol).or_insert(0) += 1`
- If `packet.app_protocol_hint()` is Some: `services.entry(svc).or_insert(0) += 1`

`hosts` is a `HashSet<IpAddr>` (deduplicating). `protocols` and `services`
are `HashMap` counters. `services` uses port-based hints only
(LESSON-P3.01: may disagree with content-first dispatch in CAP-05).

`Summary::unique_hosts()` returns a sorted `Vec<IpAddr>` for stable output.


## Output-format selection (resolve_format / write_output)

`resolve_format(cli)` applies precedence:

1. `cli.json.is_some()` -> OutputFormat::Json
2. `cli.csv.is_some()` -> OutputFormat::Csv
3. `cli.output_format` (explicit --output-format flag)
4. None -> terminal (default)

`write_output(output, cli)` routes the rendered string:
- `--json <FILE>`: `fs::write(path, output)`
- `--csv <FILE>`: `fs::write(path, output)`
- Otherwise: `println!("{output}")` to stdout

No stderr is used for normal output. Error context strings are produced
via `anyhow::Context` and propagate to main()'s `-> Result<()>`.


## Color / NO_COLOR resolution

`use_color = !cli.no_color && std::env::var("NO_COLOR").is_err()`

Evaluated once at main() entry before subcommand dispatch. Both the
explicit `--no-color` flag and the `NO_COLOR` environment variable
(per the no-color.org convention) suppress color.


## Exit-code semantics

`main()` returns `Result<()>`. The Rust runtime maps:
- `Ok(())` -> exit code 0.
- `Err(e)` -> prints the anyhow error chain to stderr and exits with
  a non-zero code (typically 1, runtime-dependent).

No explicit `std::process::exit()` call exists. Exit code 0 does NOT
distinguish "findings found" from "no findings found" -- it only signals
that the pipeline completed without an unrecoverable error.


## Overlap with other capabilities -- non-overlap confirmation

- CAP-01 (PCAP File Ingestion): CAP-01 owns reading bytes from disk into
  `PcapSource`. CAP-12 owns choosing which files to read (target resolution
  and the per-file loop). CAP-12 calls CAP-01; it does not reimplement it.
- CAP-11 (Reporting and Output): CAP-11 owns reporter trait implementations
  (TerminalReporter, JsonReporter, CsvReporter) and the render() contract.
  CAP-12 owns choosing which reporter to instantiate and where to send the
  rendered string (stdout vs. file). CAP-12 calls CAP-11; it does not
  reimplement rendering logic.


## BC references

BC-CLI-*: flag parsing, subcommand routing, mutual-exclusion enforcement,
NO_COLOR resolution, unwired-flag audit (LESSON-P1.04).
BC-SUM-*: Summary::ingest field semantics, unique_hosts ordering,
services/protocols divergence (LESSON-P3.01), skipped_packets assignment.
