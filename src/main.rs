//! `wirerust` binary entrypoint.
//!
//! Thin wrapper over the library API in [`wirerust`]. Parses the
//! [`Cli`] surface (see `src/cli.rs`), then dispatches to one of two
//! pipelines:
//!
//! - [`run_analyze`] — full per-packet decode + reassembly + dispatcher
//!   + per-protocol analyzers + reporter.
//! - [`run_summary`] — capture-level triage only (no analyzers, no
//!   reassembly), with optional per-host breakdown in the terminal
//!   reporter when `summary --hosts` is given (LESSON-P1.03).
//!
//! Both pipelines respect the `--json [<FILE>]`, `--csv [<FILE>]`, and
//! `--output-format {json,csv}` flags ([`resolve_format`]); each writes
//! to a file when a path is given, else to stdout ([`write_output`]).
//! `--json` and `--csv` are mutually exclusive (LESSON-P2.03).

use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::arp::ArpAnalyzer;
use wirerust::analyzer::dnp3::Dnp3Analyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::modbus::ModbusAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::cli::{Cli, Commands, OutputFormat};
use wirerust::decoder::{DecodedFrame, decode_packet};
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reader::PcapSource;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};
use wirerust::reporter::Reporter;
use wirerust::reporter::csv::CsvReporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
use wirerust::summary::Summary;

/// Format the BC-2.01.009 PC6 zero-packet notice for a valid file with 0 packets.
///
/// The notice has:
///   - Base phrase: "notice: <path>: 0 packets read from <pcap|pcapng> file"
///     - "pcapng file" when the file magic is the pcapng SHB magic (0x0A0D0D0A).
///     - "pcap file" for any classic pcap magic.
///   - Optional generic segment (G > 0): " (G block(s) skipped as unsupported)"
///     where G = source.skipped_blocks.saturating_sub(source.opb_skipped).
///   - Optional OPB clause (source.opb_skipped > 0):
///     "(includes N obsolete Packet Block(s) whose data was not analyzed;
///     re-save with mergecap)"
///
/// Both segments are independently gated.  When both are true, both appear
/// space-separated after the base phrase.  When neither gate is true (G==0
/// and opb_skipped==0), only the base phrase is emitted (HS-108 Case F).
///
/// ADR-009 Decision 19 / BC-2.01.009 PC6 / STORY-128 C-1 (OPB clause) +
/// M-1 (classic-pcap wording) adversarial findings.
fn format_zero_packet_notice(path: &std::path::Path, source: &PcapSource) -> String {
    // Discriminant carried from the magic-byte probe in from_pcap_reader —
    // no second file open needed (BC-2.01.009 PC6 / Decision 19 / F-F5P1-003).
    let file_kind = if source.is_pcapng {
        "pcapng file"
    } else {
        "pcap file"
    };

    let base = format!(
        "notice: {}: 0 packets read from {file_kind}",
        path.display()
    );

    // G = generic block count (skipped but not OPB-counted).
    let g = source.skipped_blocks.saturating_sub(source.opb_skipped);
    let n = source.opb_skipped;

    match (g > 0, n > 0) {
        (false, false) => base,
        (true, false) => format!("{base} ({g} block(s) skipped as unsupported)"),
        (false, true) => format!(
            "{base} (includes {n} obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"
        ),
        (true, true) => format!(
            "{base} ({g} block(s) skipped as unsupported) (includes {n} obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"
        ),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let use_color = !cli.no_color && std::env::var("NO_COLOR").is_err();

    match &cli.command {
        Commands::Analyze {
            targets,
            dns,
            http,
            tls,
            all,
            mitre,
            no_collapse,
            modbus,
            modbus_write_burst_threshold,
            modbus_write_sustained_threshold,
            dnp3,
            dnp3_direct_operate_threshold,
            arp,
            arp_spoof_threshold,
            arp_storm_rate,
        } => {
            run_analyze(
                targets,
                *dns || *all,
                *http || *all,
                *tls || *all,
                *modbus || *all,
                *modbus_write_burst_threshold,
                *modbus_write_sustained_threshold,
                *dnp3 || *all,
                *dnp3_direct_operate_threshold,
                *arp || *all,
                *arp_spoof_threshold,
                *arp_storm_rate,
                *mitre,
                collapse_findings_from_flag(*no_collapse),
                use_color,
                &cli,
            )?;
        }
        Commands::Summary { targets, hosts } => {
            run_summary(targets, *hosts, use_color, &cli)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    enable_http: bool,
    enable_tls: bool,
    enable_modbus: bool,
    modbus_write_burst_threshold: u32,
    modbus_write_sustained_threshold: u32,
    enable_dnp3: bool,
    dnp3_direct_operate_threshold: u32,
    enable_arp: bool,
    arp_spoof_threshold: u32,
    arp_storm_rate: u32,
    show_mitre_grouping: bool,
    collapse_findings: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    // BC-2.14.024 §P3a/P3b: validate thresholds before constructing the analyzer.
    if modbus_write_burst_threshold == 0 {
        anyhow::bail!("--modbus-write-burst-threshold must be >= 1 (got 0)");
    }
    if modbus_write_sustained_threshold == 0 {
        anyhow::bail!("--modbus-write-sustained-threshold must be >= 1 (got 0)");
    }
    // BC-2.16.008 EC-006 / BC-2.16.012 EC-004 / BC-2.16.013 EC-004 / D-074:
    // Reject zero thresholds before constructing the ARP analyzer, mirroring
    // the modbus guards above.  Value 0 is rejected here so that the D3 storm
    // detector and spoof detector never operate with degenerate parameters.
    if arp_storm_rate == 0 {
        anyhow::bail!("--arp-storm-rate must be >= 1 (got 0)");
    }
    if arp_spoof_threshold == 0 {
        anyhow::bail!("--arp-spoof-threshold must be >= 1 (got 0)");
    }

    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    // STORY-115: ArpAnalyzer::new(spoof_threshold, storm_rate).
    // arp_spoof_threshold is wired from --arp-spoof-threshold (BC-2.16.012).
    // arp_storm_rate is wired from --arp-storm-rate (BC-2.16.013; STORY-115).
    // Default 50 applies when flag is absent (clap default_value_t = 50).
    // --arp flag-gating is wired below (BC-2.16.011).
    let mut arp_analyzer = ArpAnalyzer::new(arp_spoof_threshold, arp_storm_rate);
    let mut all_findings = Vec::new();
    let mut total_decode_errors: u64 = 0;

    let skip_reassembly = cli.no_reassemble;

    // BC-2.14.023 §P4: needs_reassembly includes enable_modbus / enable_dnp3 so that
    // --modbus or --dnp3 alone (without --http or --tls) activates the reassembly engine.
    let needs_reassembly =
        cli.reassemble || enable_http || enable_tls || enable_modbus || enable_dnp3;

    if (enable_http || enable_tls) && skip_reassembly {
        eprintln!(
            "Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped."
        );
    }
    // BC-2.14.023 §P2 sub-case (EC-001): warn and omit Modbus when reassembly disabled.
    if enable_modbus && skip_reassembly {
        eprintln!(
            "WARNING: --modbus requires stream reassembly; ignoring --modbus \
             (pass --reassemble or omit --no-reassemble)"
        );
    }
    // BC-2.15.021: warn and omit DNP3 when reassembly disabled.
    if enable_dnp3 && skip_reassembly {
        eprintln!(
            "WARNING: --dnp3 requires stream reassembly; ignoring --dnp3 \
             (pass --reassemble or omit --no-reassemble)"
        );
    }

    let mut reassembler = if needs_reassembly && !skip_reassembly {
        let mut config = ReassemblyConfig {
            max_depth: cli.reassembly_depth * 1_048_576,
            memcap: cli.reassembly_memcap * 1_048_576,
            ..ReassemblyConfig::default()
        };
        // LESSON-P2.05: anomaly thresholds are CLI-overridable; an
        // absent flag leaves the `ReassemblyConfig::default()` value.
        if let Some(v) = cli.overlap_threshold {
            config.overlap_alert_threshold = v;
        }
        if let Some(v) = cli.small_segment_threshold {
            config.small_segment_alert_threshold = v;
        }
        if let Some(v) = cli.small_segment_max_bytes {
            config.small_segment_max_bytes = v;
        }
        if let Some(v) = cli.small_segment_ignore_ports.clone() {
            config.small_segment_ignore_ports = v;
        }
        if let Some(v) = cli.out_of_window_threshold {
            config.out_of_window_alert_threshold = v;
        }
        // HS-043 fix: wire --flow-timeout into the config (BC-2.04.013 v1.5).
        // u64 → u32 saturating cast: values above u32::MAX (~136 years) clamp
        // to u32::MAX, which is a safe, non-silent default for any real capture.
        config.flow_timeout_secs = cli.flow_timeout.min(u64::from(u32::MAX)) as u32;
        Some(TcpReassembler::new(config))
    } else {
        None
    };

    let http_analyzer = if enable_http && !skip_reassembly {
        Some(HttpAnalyzer::new())
    } else {
        None
    };
    let tls_analyzer = if enable_tls && !skip_reassembly {
        Some(TlsAnalyzer::new())
    } else {
        None
    };
    // BC-2.14.023 §P2: construct ModbusAnalyzer only when enabled AND reassembly is on.
    let modbus_analyzer: Option<ModbusAnalyzer> = if enable_modbus && !skip_reassembly {
        Some(ModbusAnalyzer::new(
            modbus_write_burst_threshold,
            modbus_write_sustained_threshold,
        ))
    } else {
        None
    };
    // BC-2.15.021 / BC-2.15.017: construct Dnp3Analyzer only when enabled AND reassembly is on.
    // Forward the CLI-parsed dnp3_direct_operate_threshold to Dnp3Analyzer::new().
    // When flag omitted, clap default = DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT (10).
    // AC-006: threshold wired; AC-007: 0 fires immediately, MAX never fires;
    // AC-008: threshold echoed in T1692.001 summary string "(threshold N)".
    let dnp3_analyzer: Option<Dnp3Analyzer> = if enable_dnp3 && !skip_reassembly {
        Some(Dnp3Analyzer::new(dnp3_direct_operate_threshold))
    } else {
        None
    };

    let mut dispatcher =
        StreamDispatcher::new(http_analyzer, tls_analyzer, modbus_analyzer, dnp3_analyzer);

    // STORY-128 (BC-2.01.018 AC-002 / ADR-009 Decision 12): per-file error isolation.
    //
    // The closure pattern has been replaced with a direct loop that catches per-file
    // reader errors via `match` rather than `?`-propagation.  This guarantees:
    //   - A bad file (any Err from PcapSource::from_file) never aborts the batch.
    //   - The reassembler `finalize` call below is always reached (loop never bails).
    //   - `any_error` accumulates whether any file failed; exit code is set after the loop.
    //
    // LESSON-P0.03 / architecture smell #9 ("no-Drop / finalize-fragile") is preserved:
    // the loop body never `?`-bails, so `reassembler.finalize()` is always reached.
    let mut any_error = false;
    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            // Per-file isolation: catch reader Err, report, continue (BC-2.01.018 AC-002).
            let source = match PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {}: {e:#}", path.display());
                    any_error = true;
                    continue;
                }
            };

            // ADR-009 Decision 19 / BC-2.01.009 PC6: emit zero-packet notice when
            // a valid file contains no packets (e.g. SHB-only pcapng).
            // Full format includes gated OPB clause and generic-skip segment.
            // See format_zero_packet_notice for BC-2.01.009 PC6 specification.
            if source.packets.is_empty() {
                eprintln!("{}", format_zero_packet_notice(path, &source));
                continue;
            }

            let pb = ProgressBar::new(source.packets.len() as u64);
            pb.set_style(ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40} {pos}/{len} packets",
            )?);

            for raw in &source.packets {
                match decode_packet(&raw.data, source.datalink) {
                    Ok(DecodedFrame::Ip(parsed)) => {
                        summary.ingest(&parsed);
                        if enable_dns && dns_analyzer.can_decode(&parsed) {
                            let findings = dns_analyzer.analyze(&parsed);
                            all_findings.extend(findings);
                        }
                        if let Some(ref mut reasm) = reassembler {
                            reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
                        }
                    }
                    // STORY-113: ArpAnalyzer wiring (BC-2.16.011; AC-015/AC-016).
                    // ARP frames NEVER reach StreamDispatcher (Forbidden Dependency;
                    // arp-architecture-delta.md §1; BC-2.16.015 Invariant 2).
                    // process_arp is called only when --arp is active (enable_arp gate).
                    Ok(DecodedFrame::Arp(arp_frame)) => {
                        if enable_arp {
                            let ts = raw.timestamp_secs;
                            let findings = arp_analyzer.process_arp(&arp_frame, ts);
                            all_findings.extend(findings);
                        }
                    }
                    Err(ref e) if e.to_string().contains("Non-Ethernet/IPv4 ARP frame") => {
                        // STORY-113: D11 malformed ARP notification (BC-2.16.009 PC3/PC4).
                        // malformed_frames increments unconditionally (BC-2.16.009 PC4).
                        // When --arp active: record_malformed emits D11 Finding and
                        // increments both malformed_frames and malformed_findings (AC-012).
                        // When --arp absent: only malformed_frames increments; no finding
                        // emitted and malformed_findings unchanged (BC-2.16.009 PC4, ADR-008
                        // Decision 7, BC-2.16.010 key 11).
                        if enable_arp {
                            all_findings.extend(arp_analyzer.record_malformed(raw.data.len()));
                        } else {
                            arp_analyzer.malformed_frames += 1;
                        }
                    }
                    Err(e) => {
                        if total_decode_errors == 0 {
                            eprintln!(
                                "Warning: failed to decode packet ({e}). Further errors counted silently."
                            );
                        }
                        total_decode_errors += 1;
                    }
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
        }
    }

    summary.skipped_packets = total_decode_errors;

    // ALWAYS finalize the reassembler — the loop never ?-bails, so this is
    // always reached regardless of per-file errors (preserves finalize guarantee).
    if let Some(ref mut reasm) = reassembler {
        reasm.finalize(&mut dispatcher);
        all_findings.extend(reasm.findings().to_vec());
    }

    if let Some(http) = dispatcher.http_analyzer() {
        all_findings.extend(http.findings());
    }
    if let Some(tls) = dispatcher.tls_analyzer() {
        all_findings.extend(tls.findings());
    }

    let mut analyzer_summaries = Vec::new();
    if let Some(ref reasm) = reassembler {
        let mut reasm_summary = reasm.summarize();
        reasm_summary.detail.insert(
            "unclassified_flows".to_string(),
            serde_json::json!(dispatcher.unclassified_flows()),
        );
        analyzer_summaries.push(reasm_summary);
    }
    if enable_dns {
        analyzer_summaries.push(dns_analyzer.summarize());
    }
    if let Some(http) = dispatcher.http_analyzer() {
        analyzer_summaries.push(http.summarize());
    }
    if let Some(tls) = dispatcher.tls_analyzer() {
        analyzer_summaries.push(tls.summarize());
    }

    // BC-2.14.023 §P5: post-finalize — collect Modbus findings and summary.
    if let Some(modbus) = dispatcher.take_modbus_analyzer() {
        all_findings.extend(modbus.all_findings.iter().cloned());
        analyzer_summaries.push(modbus.summarize());
    }

    // BC-2.15.021: post-finalize — collect DNP3 findings and summary.
    if let Some(dnp3) = dispatcher.take_dnp3_analyzer() {
        all_findings.extend(dnp3.all_findings.iter().cloned());
        analyzer_summaries.push(dnp3.summarize());
    }

    // STORY-113: ARP post-capture summary (BC-2.16.011 PC7/8; AC-016).
    // summarize() and push to analyzer_summaries only when --arp is active.
    // Mirrors the Modbus/DNP3 pattern per BC-2.16.010 Invariant 4.
    if enable_arp {
        analyzer_summaries.push(arp_analyzer.summarize());
    }

    let resolved_format = resolve_format(cli);
    let output = match resolved_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        Some(OutputFormat::Csv) => {
            let reporter = CsvReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter {
                use_color,
                // `analyze` does not expose a per-host breakdown flag —
                // that is `summary`-subcommand-only (LESSON-P1.03).
                show_hosts_breakdown: false,
                // BC-2.11.028 / BC-2.11.030: orthogonal 2-if struct wiring (STORY-119/B CLI flip).
                // --mitre alone (show_mitre_grouping=true, collapse_findings=true) → {Grouped, Collapsed}.
                // --mitre --no-collapse (show_mitre_grouping=true, collapse_findings=false) → {Grouped, Expanded}.
                // default (show_mitre_grouping=false, collapse_findings=true) → {Flat, Collapsed}.
                // --no-collapse only (show_mitre_grouping=false, collapse_findings=false) → {Flat, Expanded}.
                render: FindingsRender::new(
                    grouping_from_flag(show_mitre_grouping),
                    if collapse_findings {
                        Collapse::Collapsed
                    } else {
                        Collapse::Expanded
                    },
                ),
            };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    write_output(&output, cli)?;

    // STORY-128: exit code 1 iff any file failed (BC-2.01.018 AC-002).
    // Deferred until after write_output so the summary is always emitted
    // even when some files in the batch failed (good output is never suppressed).
    if any_error {
        std::process::exit(1);
    }
    Ok(())
}

fn run_summary(
    targets: &[std::path::PathBuf],
    show_hosts_breakdown: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut total_decode_errors: u64 = 0;

    // STORY-128 (BC-2.01.018 AC-002 / ADR-009 Decision 12): per-file error isolation
    // for run_summary, mirroring the pattern in run_analyze.
    let mut any_error = false;
    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            // Per-file isolation: catch reader Err, report, continue.
            let source = match PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {}: {e:#}", path.display());
                    any_error = true;
                    continue;
                }
            };

            // ADR-009 Decision 19 / BC-2.01.009 PC6: zero-packet notice for valid
            // zero-packet files.  Full format with gated OPB clause and generic-skip
            // segment.  See format_zero_packet_notice for specification.
            if source.packets.is_empty() {
                eprintln!("{}", format_zero_packet_notice(path, &source));
                continue;
            }

            for raw in &source.packets {
                match decode_packet(&raw.data, source.datalink) {
                    Ok(DecodedFrame::Ip(parsed)) => {
                        summary.ingest(&parsed);
                    }
                    // ArpAnalyzer is not wired in the summary subcommand path;
                    // ARP security analysis runs only under `analyze --arp`.
                    Ok(DecodedFrame::Arp(_arp_frame)) => {}
                    Err(e) => {
                        if total_decode_errors == 0 {
                            eprintln!(
                                "Warning: failed to decode packet ({e}). Further errors counted silently."
                            );
                        }
                        total_decode_errors += 1;
                    }
                }
            }
        }
    }
    summary.skipped_packets = total_decode_errors;

    let resolved_format = resolve_format(cli);
    let output = match resolved_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &[], &[])
        }
        Some(OutputFormat::Csv) => {
            let reporter = CsvReporter;
            reporter.render(&summary, &[], &[])
        }
        _ => {
            let reporter = TerminalReporter {
                use_color,
                show_hosts_breakdown,
                // BC-2.11.028 invariant 4: render field is inert for run_summary — no FINDINGS section.
                render: FindingsRender::new(Grouping::Flat, Collapse::Collapsed),
            };
            reporter.render(&summary, &[], &[])
        }
    };

    write_output(&output, cli)?;

    // STORY-128: exit code 1 iff any file failed (BC-2.01.018 AC-002).
    // Deferred until after write_output so the summary is always emitted.
    if any_error {
        std::process::exit(1);
    }
    Ok(())
}

/// Determine the output format.
///
/// Precedence (highest to lowest):
/// 1. `--json [<FILE>]` forces `OutputFormat::Json`.
/// 2. `--csv [<FILE>]` forces `OutputFormat::Csv`. (`--json` and `--csv`
///    are mutually exclusive — clap rejects the combination.)
/// 3. `--output-format <fmt>` is honored as-is.
/// 4. Default (terminal table) when no flag is given.
fn resolve_format(cli: &Cli) -> Option<OutputFormat> {
    if cli.json.is_some() {
        Some(OutputFormat::Json)
    } else if cli.csv.is_some() {
        Some(OutputFormat::Csv)
    } else {
        cli.output_format
    }
}

/// Write rendered output to a file (if `--json <FILE>` or `--csv <FILE>`
/// was given with a path) or to stdout otherwise.
///
/// `--json` and `--csv` are mutually exclusive (enforced by clap), so at
/// most one of the file-path arms can be active.
fn write_output(output: &str, cli: &Cli) -> Result<()> {
    match (&cli.json, &cli.csv) {
        (Some(Some(path)), _) => std::fs::write(path, output)
            .with_context(|| format!("Failed to write JSON output to {}", path.display())),
        (_, Some(Some(path))) => std::fs::write(path, output)
            .with_context(|| format!("Failed to write CSV output to {}", path.display())),
        _ => {
            println!("{output}");
            Ok(())
        }
    }
}

/// Maps the `--no-collapse` opt-out flag to the `TerminalReporter` `collapse_findings`
/// field (default-on per BC-2.11.028).
///
/// When the flag is absent (`no_collapse = false`), collapse is ON (`true`).
/// When `--no-collapse` is passed (`no_collapse = true`), collapse is OFF (`false`).
fn collapse_findings_from_flag(no_collapse: bool) -> bool {
    !no_collapse
}

/// Maps the `--mitre` flag to the `FindingsRender` `grouping` field
/// (BC-2.11.030 PC-2 through PC-5).
///
/// When `--mitre` is present (`show_mitre_grouping = true`), grouping is `Grouped`.
/// When `--mitre` is absent (`show_mitre_grouping = false`), grouping is `Flat`.
fn grouping_from_flag(show_mitre_grouping: bool) -> Grouping {
    if show_mitre_grouping {
        Grouping::Grouped
    } else {
        Grouping::Flat
    }
}

/// Reads the first 4 bytes of a file for magic-byte content detection.
///
/// Returns `None` if the file cannot be opened, the file has fewer than 4
/// readable bytes, or any I/O error occurs.  The probe is intentionally
/// silent — errors must not abort a directory scan.
///
/// Called by `resolve_targets` (BC-2.12.011 Inv5 / STORY-127).
/// `pub(crate)` so the unit test suite in `tests/` can call it directly.
pub(crate) fn read_magic(path: &std::path::Path) -> Option<[u8; 4]> {
    use std::io::Read as _;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf).ok()?;
    Some(buf)
}

/// Magic byte sets for content-based capture file detection (BC-2.12.011).
///
/// Exactly 5 values per BC-2.12.011 Invariant 2.  A 6th value requires a BC revision.
const CAPTURE_MAGICS: [[u8; 4]; 5] = [
    [0xA1, 0xB2, 0xC3, 0xD4], // classic pcap LE microsecond
    [0xD4, 0xC3, 0xB2, 0xA1], // classic pcap BE microsecond
    [0xA1, 0xB2, 0x3C, 0x4D], // classic pcap LE nanosecond
    [0x4D, 0x3C, 0xB2, 0xA1], // classic pcap BE nanosecond
    [0x0A, 0x0D, 0x0D, 0x0A], // pcapng SHB
];

fn resolve_targets(target: &Path) -> Result<Vec<std::path::PathBuf>> {
    if target.is_file() {
        return Ok(vec![target.to_path_buf()]);
    }
    if target.is_dir() {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(target)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(magic) = read_magic(&path)
                && CAPTURE_MAGICS.contains(&magic)
            {
                files.push(path);
            }
        }
        files.sort();
        return Ok(files);
    }
    anyhow::bail!("Target not found: {}", target.display());
}

#[cfg(test)]
mod tests {
    use super::{collapse_findings_from_flag, grouping_from_flag, read_magic};
    use wirerust::reporter::terminal::Grouping;

    /// BC-2.11.028: flag absent (false) → collapse ON (true);
    /// --no-collapse present (true) → collapse OFF (false).
    /// Guards against a polarity inversion in the wiring.
    #[test]
    fn test_bc_2_11_028_collapse_flag_polarity() {
        // Default: --no-collapse not passed → collapse should be enabled.
        assert!(
            collapse_findings_from_flag(false),
            "flag absent (no_collapse=false) must yield collapse_findings=true"
        );
        // Opt-out: --no-collapse passed → collapse should be disabled.
        assert!(
            !collapse_findings_from_flag(true),
            "--no-collapse (no_collapse=true) must yield collapse_findings=false"
        );
    }

    /// BC-2.11.030 PC-2/PC-4: `--mitre` flag polarity guard.
    ///
    /// Guards against swapping `Grouped`/`Flat` in `grouping_from_flag` —
    /// a regression that would ship silently if the construction-site mapping
    /// were only tested via tautological literal copies.
    #[test]
    fn test_bc_2_11_030_grouping_flag_polarity() {
        // --mitre present (show_mitre_grouping=true) → Grouped.
        assert_eq!(
            grouping_from_flag(true),
            Grouping::Grouped,
            "--mitre present (show_mitre_grouping=true) must yield Grouping::Grouped"
        );
        // --mitre absent (show_mitre_grouping=false) → Flat.
        assert_eq!(
            grouping_from_flag(false),
            Grouping::Flat,
            "--mitre absent (show_mitre_grouping=false) must yield Grouping::Flat"
        );
    }

    /// F-F5P5-002 — read_magic robustness: a file with fewer than 4 bytes returns None
    /// (BC-2.12.011 Inv5 / EC-007).  A single `Read::read` may legally return <4 bytes
    /// even for a ≥4-byte file; `read_exact` eliminates that race without changing the
    /// short-file or unreadable-file semantics.
    #[test]
    fn test_read_magic_short_file_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("short.bin");
        std::fs::write(&path, b"\xA1\xB2\xC3").expect("write short file");

        // A 3-byte file — cannot fill 4-byte magic buffer → must return None.
        assert_eq!(
            read_magic(&path),
            None,
            "read_magic on a <4-byte file must return None (BC-2.12.011 EC-007)"
        );
    }

    /// F-F5P5-002 — read_magic robustness: a ≥4-byte file with a known magic returns
    /// exactly those 4 bytes (BC-2.12.011 Inv5 — reads the FIRST 4 BYTES).
    #[test]
    fn test_read_magic_valid_file_returns_first_four_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("capture.pcapng");
        // pcapng SHB magic followed by additional bytes — read_magic must return exactly
        // the first 4 bytes and not be influenced by the bytes that follow.
        let content: &[u8] = &[0x0A, 0x0D, 0x0D, 0x0A, 0xFF, 0xFF, 0xFF, 0xFF];
        std::fs::write(&path, content).expect("write capture file");

        assert_eq!(
            read_magic(&path),
            Some([0x0A, 0x0D, 0x0D, 0x0A]),
            "read_magic must return the first 4 bytes of a ≥4-byte file (BC-2.12.011 Inv5)"
        );
    }
}
