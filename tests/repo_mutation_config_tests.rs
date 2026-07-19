//! Red-Gate tests for STORY-147 — Repo-Local Mutation-Testing Defaults.
//!
//! Covers AC-147-001..004. Deliberately dependency-free: rather than pulling in
//! a `toml` dev-dependency for a two-key lookup, this file uses a minimal
//! line-oriented parser sufficient to answer "does this config file set a low
//! default job count / generous timeout" and "does CLAUDE.md contain the
//! required Mutation testing note". These are file-content assertions only —
//! no `cargo mutants` invocation (it may not be installed in CI), no network,
//! no timing-dependent behavior.
//!
//! Background: PG-MUTANTS-JOBS-001 (fix-tls-clienthello-frag F6, 2026-07-01).
//! `cargo mutants --jobs 8` reported a false "0 missed" because infinite-loop
//! mutants pegged all cores, inflating other mutants' wall-clock past the
//! auto-timeout threshold. Only a `--jobs 1` re-run surfaced the real
//! survivors. See drbothen/vsdd-factory#654 for the upstream engine-default
//! tracking issue.
//!
//! As of this writing (Red Gate, pre-implementation) none of the deliverables
//! exist on disk, so every test below is expected to FAIL.
//!
//! Test naming uses the AC-based pattern `test_AC_147_NNN_<assertion>()`,
//! matching the repo-wide `test_BC_S_SS_NNN_...` convention (uppercase
//! `AC`/BC segments). This is a deliberate deviation from Rust's snake_case
//! convention — see `tests/bc_2_01_story001_tests.rs` for precedent.
#![allow(non_snake_case)]

use std::fs;
use std::path::{Path, PathBuf};

/// Repo root, resolved via the manifest dir env var baked in at compile time
/// (works regardless of the directory `cargo test` is invoked from).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_if_exists(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Strip a trailing `# comment` from a TOML-ish value, then trim whitespace
/// and a single layer of surrounding quotes.
fn clean_value(raw: &str) -> String {
    let without_comment = match raw.find('#') {
        Some(idx) => &raw[..idx],
        None => raw,
    };
    without_comment.trim().trim_matches('"').to_string()
}

/// Parse a single `key = value` line (TOML-ish, not a full parser). Returns
/// `None` for blank lines, comments, or section headers (`[...]`).
fn parse_key_value(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let (key, value) = trimmed.split_once('=')?;
    Some((key.trim().to_string(), clean_value(value)))
}

/// Extract the body of a `[header]` TOML table (everything up to the next
/// line that starts a new table, or EOF). Returns `None` if the header is
/// absent. Deliberately naive — sufficient for the small config files this
/// story introduces; not a general TOML parser.
fn extract_table<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    let header_line = format!("[{header}]");
    let mut offset = 0usize;
    for line in content.lines() {
        if line.trim() == header_line {
            let body_start = offset + line.len();
            let body_start = content[body_start..]
                .find('\n')
                .map(|i| body_start + i + 1)
                .unwrap_or(content.len());
            let rest = &content[body_start..];
            let body_end = rest
                .lines()
                .scan(0usize, |pos, l| {
                    let start = *pos;
                    *pos += l.len() + 1;
                    Some((start, l))
                })
                .find(|(_, l)| l.trim_start().starts_with('['))
                .map(|(start, _)| start)
                .unwrap_or(rest.len());
            return Some(&rest[..body_end]);
        }
        offset += line.len() + 1;
    }
    None
}

/// Outcome of scanning a candidate config body for a low-parallelism default.
#[derive(Debug)]
struct LowParallelismSignal {
    jobs: Option<u32>,
    timeout_seconds: Option<u64>,
}

fn scan_low_parallelism(body: &str) -> LowParallelismSignal {
    let mut jobs = None;
    let mut timeout_seconds = None;
    for line in body.lines() {
        if let Some((key, value)) = parse_key_value(line) {
            match key.as_str() {
                "jobs" => jobs = value.parse::<u32>().ok(),
                // cargo-mutants config surfaces a per-mutant timeout under a
                // few possible key spellings depending on version; accept any
                // of them as evidence of the "generous timeout" defense.
                "timeout" | "minimum_test_timeout" | "test_timeout" => {
                    timeout_seconds = value.parse::<u64>().ok();
                }
                _ => {}
            }
        }
    }
    LowParallelismSignal {
        jobs,
        timeout_seconds,
    }
}

/// One candidate location cargo-mutants is documented to read config from,
/// paired with the extracted config body (if the location exists at all).
struct ConfigCandidate {
    location: &'static str,
    body: Option<String>,
}

fn candidate_locations() -> Vec<ConfigCandidate> {
    let root = repo_root();

    let repo_root_mutants_toml = read_if_exists(&root.join("mutants.toml"));

    let dot_cargo_mutants_toml = read_if_exists(&root.join(".cargo").join("mutants.toml"));

    let cargo_toml_metadata = read_if_exists(&root.join("Cargo.toml"))
        .as_deref()
        .and_then(|content| extract_table(content, "package.metadata.mutants"))
        .map(str::to_string);

    vec![
        ConfigCandidate {
            location: "mutants.toml (repo root)",
            body: repo_root_mutants_toml,
        },
        ConfigCandidate {
            location: ".cargo/mutants.toml",
            body: dot_cargo_mutants_toml,
        },
        ConfigCandidate {
            location: "Cargo.toml [package.metadata.mutants]",
            body: cargo_toml_metadata,
        },
    ]
}

/// Find the first candidate location that both exists and sets a low
/// parallelism default (jobs <= 2, or an explicit generous timeout).
fn find_active_low_parallelism_config() -> Option<(&'static str, LowParallelismSignal)> {
    for candidate in candidate_locations() {
        if let Some(body) = candidate.body {
            let signal = scan_low_parallelism(&body);
            let jobs_ok = signal.jobs.is_some_and(|j| j <= 2);
            // "Generous" per-mutant timeout: require it to be explicitly
            // configured to something well above cargo-mutants' auto-timeout
            // heuristic (a few tens of seconds) — 300s is a conservative
            // floor for "clearly intentional, not accidental".
            let timeout_ok = signal.timeout_seconds.is_some_and(|t| t >= 300);
            if jobs_ok || timeout_ok {
                return Some((candidate.location, signal));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// AC-147-001: A mutants.toml file exists at the repo root (or a
// [package.metadata.mutants] table exists in Cargo.toml) that sets a low
// default job count (<= 2) or a generous per-mutant timeout.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_001_low_parallelism_mutation_config_exists() {
    let found = find_active_low_parallelism_config();
    assert!(
        found.is_some(),
        "AC-147-001: no mutation-testing config sets a low-parallelism default. \
         Expected one of: repo-root mutants.toml, .cargo/mutants.toml, or a \
         [package.metadata.mutants] table in Cargo.toml, containing either \
         `jobs = <=2` or an explicit generous per-mutant timeout (>= 300s). \
         None of the checked locations exist yet on this tree (checked: {:?}).",
        candidate_locations()
            .iter()
            .map(|c| c.location)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-147-002: Running `cargo mutants` without --jobs on this codebase uses
// the configured low-parallelism default. We do not shell out to
// `cargo mutants` (may be uninstalled in CI); instead we assert the config
// lives at a location cargo-mutants is documented to actually read, and that
// the low-parallelism value is genuinely present in that file's content
// (not merely mentioned in a comment or docs elsewhere).
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_002_low_parallelism_value_active_at_cargo_mutants_read_location() {
    const READABLE_LOCATIONS: &[&str] = &[
        "mutants.toml (repo root)",
        ".cargo/mutants.toml",
        "Cargo.toml [package.metadata.mutants]",
    ];

    let found = find_active_low_parallelism_config();
    match found {
        Some((location, signal)) => {
            assert!(
                READABLE_LOCATIONS.contains(&location),
                "AC-147-002: low-parallelism config found at '{location}', which is not \
                 one of the locations cargo-mutants documents reading config from: {READABLE_LOCATIONS:?}"
            );
            assert!(
                signal.jobs.is_some_and(|j| j <= 2)
                    || signal.timeout_seconds.is_some_and(|t| t >= 300),
                "AC-147-002: config at '{location}' was located but the parsed value is not \
                 actually active (jobs={:?}, timeout_seconds={:?}) — expected jobs <= 2 or \
                 timeout_seconds >= 300",
                signal.jobs,
                signal.timeout_seconds
            );
        }
        None => {
            panic!(
                "AC-147-002: no cargo-mutants-readable config location (repo-root mutants.toml, \
                 .cargo/mutants.toml, or Cargo.toml [package.metadata.mutants]) currently sets an \
                 active low-parallelism default. cargo-mutants invoked bare (no --jobs flag) would \
                 fall back to its own high-parallelism default on this tree today."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-147-003: CLAUDE.md contains a "Mutation testing" note covering:
//   (a) recommended invocation (--jobs 1 or equivalent --timeout guidance)
//   (b) rationale (infinite-loop mutants inflate wall-clock past auto-timeout
//       -> false "0 missed")
//   (c) PG-MUTANTS-JOBS-001 reference
//   (d) drbothen/vsdd-factory#654 pointer
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_003_claude_md_has_mutation_testing_section() {
    let claude_md = read_if_exists(&repo_root().join("CLAUDE.md")).unwrap_or_default();

    assert!(
        claude_md.contains("Mutation testing"),
        "AC-147-003(a): CLAUDE.md is missing a \"Mutation testing\" section/heading."
    );
    assert!(
        claude_md.contains("--jobs 1"),
        "AC-147-003(a): CLAUDE.md does not mention the recommended `--jobs 1` invocation \
         (or equivalent --timeout guidance marker)."
    );
    assert!(
        claude_md.to_lowercase().contains("false")
            && (claude_md.to_lowercase().contains("0 missed")
                || claude_md.to_lowercase().contains("timeout")),
        "AC-147-003(b): CLAUDE.md does not explain the rationale — infinite-loop mutants \
         inflating wall-clock past the auto-timeout threshold, producing a false \"0 missed\" result."
    );
    assert!(
        claude_md.contains("PG-MUTANTS-JOBS-001"),
        "AC-147-003(c): CLAUDE.md does not reference the process-gap ID PG-MUTANTS-JOBS-001."
    );
    assert!(
        claude_md.contains("#654"),
        "AC-147-003(d): CLAUDE.md does not reference drbothen/vsdd-factory#654 as the \
         upstream engine-default tracking issue pointer."
    );
}

// ---------------------------------------------------------------------------
// AC-147-004 (self-audit): after this story ships, a developer running
// `cargo mutants` from a fresh checkout will not silently receive a
// false-clean result due to load-induced timeouts. This is the conjunction
// of the two file-level defenses: the repo-root config default (first line
// of defense) AND the CLAUDE.md note (second line of defense), both present
// simultaneously. The human-facing self-audit narrative itself is recorded
// in the story's demo-evidence step, not re-derived here — this test only
// verifies the mechanical precondition that both defenses actually exist.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_004_both_defenses_present_simultaneously() {
    let config_defense = find_active_low_parallelism_config().is_some();

    let claude_md = read_if_exists(&repo_root().join("CLAUDE.md")).unwrap_or_default();
    let doc_defense =
        claude_md.contains("Mutation testing") && claude_md.contains("PG-MUTANTS-JOBS-001");

    assert!(
        config_defense,
        "AC-147-004: first line of defense (repo-root low-parallelism mutation config) is \
         absent — see AC-147-001/002 failures for detail."
    );
    assert!(
        doc_defense,
        "AC-147-004: second line of defense (CLAUDE.md \"Mutation testing\" note referencing \
         PG-MUTANTS-JOBS-001) is absent — see AC-147-003 failures for detail."
    );
    assert!(
        config_defense && doc_defense,
        "AC-147-004: self-audit failed — a fresh checkout today does NOT have both defenses \
         in place simultaneously (config_defense={config_defense}, doc_defense={doc_defense}), \
         so `cargo mutants` run bare could still silently produce a false \"0 missed\" result \
         under load."
    );
}
