use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::cli::{Cli, Commands, OutputFormat};
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::terminal::TerminalReporter;
use wirerust::reporter::Reporter;
use wirerust::summary::Summary;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let use_color = !cli.no_color && std::env::var("NO_COLOR").is_err();

    match &cli.command {
        Commands::Analyze { targets, dns, all, .. } => {
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

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let pb = ProgressBar::new(source.packets.len() as u64);
            pb.set_style(
                ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos}/{len} packets")?
            );

            for raw in &source.packets {
                if let Ok(parsed) = decode_packet(&raw.data) {
                    summary.ingest(&parsed);
                    if enable_dns && dns_analyzer.can_decode(&parsed) {
                        let findings = dns_analyzer.analyze(&parsed);
                        all_findings.extend(findings);
                    }
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
        }
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

fn run_summary(
    targets: &[std::path::PathBuf],
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)?;
            for raw in &source.packets {
                if let Ok(parsed) = decode_packet(&raw.data) {
                    summary.ingest(&parsed);
                }
            }
        }
    }

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
                    && (ext == "pcap" || ext == "pcapng") {
                        files.push(path);
                    }
        }
        files.sort();
        return Ok(files);
    }
    anyhow::bail!("Target not found: {}", target.display());
}
