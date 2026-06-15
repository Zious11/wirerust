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
use wirerust::reporter::terminal::TerminalReporter;
use wirerust::summary::Summary;

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
            modbus,
            modbus_write_burst_threshold,
            modbus_write_sustained_threshold,
            dnp3,
            dnp3_direct_operate_threshold,
            arp,
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
                *mitre,
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
    show_mitre_grouping: bool,
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

    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    // STORY-113: ArpAnalyzer — parameterless new() per Architecture Compliance Rule 3.
    // --arp flag-gating is wired below (BC-2.16.011).
    // --arp-spoof-threshold is added in STORY-114; --arp-storm-rate in STORY-115.
    let mut arp_analyzer = ArpAnalyzer::new();
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

    // Capture loop wrapped in an immediately-invoked closure so any `?`-bail
    // inside (e.g. unreadable pcap, malformed progress-bar template) is
    // captured as an `Err` *without* short-circuiting `run_analyze` itself.
    // This guarantees we always reach the reassembler `finalize` call below,
    // which is what `impl Drop for TcpReassembler` only warns about — see
    // LESSON-P0.03 / architecture smell #9 ("no-Drop / finalize-fragile").
    let capture_result: Result<()> = (|| {
        for target in targets {
            let pcap_files = resolve_targets(target)?;
            for path in &pcap_files {
                let source = PcapSource::from_file(path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;

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
                            // STORY-113: D11 malformed ARP notification (BC-2.16.009 PC6).
                            // malformed_frames is always incremented (AC-012).
                            // record_malformed handles the finding-emission gate internally;
                            // the caller unconditionally notifies the analyzer per BC-2.16.009.
                            arp_analyzer.record_malformed(raw.data.len());
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
        Ok(())
    })();

    summary.skipped_packets = total_decode_errors;

    // ALWAYS finalize the reassembler before propagating any capture error,
    // so the segment-limit summary finding and per-flow flush still happen
    // on the partial state captured before the bail.
    if let Some(ref mut reasm) = reassembler {
        reasm.finalize(&mut dispatcher);
        all_findings.extend(reasm.findings().to_vec());
    }

    capture_result?;

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
                show_mitre_grouping,
                // `analyze` does not expose a per-host breakdown flag —
                // that is `summary`-subcommand-only (LESSON-P1.03).
                show_hosts_breakdown: false,
            };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    write_output(&output, cli)?;
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

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            for raw in &source.packets {
                match decode_packet(&raw.data, source.datalink) {
                    Ok(DecodedFrame::Ip(parsed)) => {
                        summary.ingest(&parsed);
                    }
                    // STORY-111 stub: ARP frames not yet dispatched in summary path.
                    // STORY-112/113 will wire ArpAnalyzer here.
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
                show_mitre_grouping: false,
                show_hosts_breakdown,
            };
            reporter.render(&summary, &[], &[])
        }
    };

    write_output(&output, cli)?;
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
                && let Some(ext) = path.extension()
                && ext == "pcap"
            {
                files.push(path);
            }
        }
        files.sort();
        return Ok(files);
    }
    anyhow::bail!("Target not found: {}", target.display());
}
