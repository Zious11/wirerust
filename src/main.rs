use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::cli::{Cli, Commands, OutputFormat};
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};
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
            targets, dns, all, ..
        } => {
            run_analyze(targets, *dns || *all, use_color, &cli)?;
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
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();
    let mut total_decode_errors: u64 = 0;

    // Determine if reassembly is needed
    let needs_reassembly = cli.reassemble; // Will expand when HTTP/TLS analyzers added
    let skip_reassembly = cli.no_reassemble;

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

    struct NullHandler;
    impl StreamHandler for NullHandler {
        fn on_data(&mut self, _: &FlowKey, _: Direction, _: &[u8], _: u64) {}
        fn on_flow_close(&mut self, _: &FlowKey, _: CloseReason) {}
    }
    let mut stream_handler = NullHandler;

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
                            reasm.process_packet(&parsed, raw.timestamp_secs, &mut stream_handler);
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
        reasm.finalize(&mut stream_handler);
        all_findings.extend(reasm.findings().to_vec());
    }

    let analyzer_summaries = if enable_dns {
        vec![dns_analyzer.summarize()]
    } else {
        vec![]
    };

    let output = match cli.output_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter { use_color };
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
            let source = PcapSource::from_file(path)?;
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
            let reporter = TerminalReporter { use_color };
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
