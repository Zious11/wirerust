use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::cli::{Cli, Commands, OutputFormat};
use wirerust::decoder::decode_packet;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reader::PcapSource;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};
use wirerust::reporter::Reporter;
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
            ..
        } => {
            run_analyze(
                targets,
                *dns || *all,
                *http || *all,
                *tls || *all,
                *mitre,
                use_color,
                &cli,
            )?;
        }
        Commands::Summary { targets, .. } => {
            run_summary(targets, use_color, &cli)?;
        }
    }

    Ok(())
}

fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    enable_http: bool,
    enable_tls: bool,
    show_mitre_grouping: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();
    let mut total_decode_errors: u64 = 0;

    let needs_reassembly = cli.reassemble || enable_http || enable_tls;
    let skip_reassembly = cli.no_reassemble;

    if (enable_http || enable_tls) && skip_reassembly {
        eprintln!(
            "Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped."
        );
    }

    let mut reassembler = if needs_reassembly && !skip_reassembly {
        let config = ReassemblyConfig {
            max_depth: cli.reassembly_depth * 1_048_576,
            memcap: cli.reassembly_memcap * 1_048_576,
            ..ReassemblyConfig::default()
        };
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
    let mut dispatcher = StreamDispatcher::new(http_analyzer, tls_analyzer);

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
                    Ok(parsed) => {
                        summary.ingest(&parsed);
                        if enable_dns && dns_analyzer.can_decode(&parsed) {
                            let findings = dns_analyzer.analyze(&parsed);
                            all_findings.extend(findings);
                        }
                        if let Some(ref mut reasm) = reassembler {
                            reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
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

    if let Some(ref mut reasm) = reassembler {
        reasm.finalize(&mut dispatcher);
        all_findings.extend(reasm.findings().to_vec());
    }

    if let Some(ref http) = dispatcher.http {
        all_findings.extend(http.findings());
    }
    if let Some(ref tls) = dispatcher.tls {
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
    if let Some(ref http) = dispatcher.http {
        analyzer_summaries.push(http.summarize());
    }
    if let Some(ref tls) = dispatcher.tls {
        analyzer_summaries.push(tls.summarize());
    }

    let output = match cli.output_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter {
                use_color,
                show_mitre_grouping,
            };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    println!("{output}");
    Ok(())
}

fn run_summary(targets: &[std::path::PathBuf], use_color: bool, cli: &Cli) -> Result<()> {
    let mut summary = Summary::new();
    let mut total_decode_errors: u64 = 0;

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            for raw in &source.packets {
                match decode_packet(&raw.data, source.datalink) {
                    Ok(parsed) => {
                        summary.ingest(&parsed);
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
            }
        }
    }
    summary.skipped_packets = total_decode_errors;

    let output = match cli.output_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &[], &[])
        }
        _ => {
            let reporter = TerminalReporter {
                use_color,
                show_mitre_grouping: false,
            };
            reporter.render(&summary, &[], &[])
        }
    };

    println!("{output}");
    Ok(())
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
                && (ext == "pcap" || ext == "pcapng")
            {
                files.push(path);
            }
        }
        files.sort();
        return Ok(files);
    }
    anyhow::bail!("Target not found: {}", target.display());
}
