//! Verifies the delivered mutation-testing defenses for STORY-147 —
//! Repo-Local Mutation-Testing Defaults: `.cargo/mutants.toml` Timeout Floor
//! + CLAUDE.md Guidance.
//!
//! Covers AC-147-001..004 (v2.2). Deliberately dependency-free: rather than
//! pulling in a `toml` dev-dependency for a handful of key lookups, this file
//! uses a minimal line-oriented parser sufficient to answer "does
//! `.cargo/mutants.toml` set a generous `minimum_test_timeout` floor without
//! an invalid `jobs` key" and "does CLAUDE.md contain the required Mutation
//! testing note". These are file-content assertions only — no `cargo
//! mutants` invocation (it may not be installed in CI), no network, no
//! timing-dependent behavior.
//!
//! Background: PG-MUTANTS-JOBS-001 (fix-tls-clienthello-frag F6, 2026-07-01).
//! `cargo mutants --jobs 8` reported a false "0 missed" because infinite-loop
//! mutants pegged all cores, inflating other mutants' wall-clock past the
//! auto-timeout threshold. Only a `--jobs 1` re-run surfaced the real
//! survivors. See drbothen/vsdd-factory#654 for the upstream engine-default
//! tracking issue.
//!
//! Execution-evidence correction (STORY-147 v2.2, F-S147P1-002/-004/-005):
//! cargo-mutants reads ONLY `.cargo/mutants.toml` by default — a repo-root
//! `mutants.toml` and a `Cargo.toml [package.metadata.mutants]` table are
//! both silently ignored, not alternate read locations. `jobs` is not a
//! valid `Config` field; the parser is `#[serde(deny_unknown_fields)]`, so a
//! `jobs = 1` line in `.cargo/mutants.toml` would abort every mutation run
//! with a fatal parse error. Parallelism is CLI/env-only (`--jobs`,
//! `CARGO_MUTANTS_JOBS`); bare `cargo mutants` is already serial by default.
//! The real, config-settable defense is a timeout floor:
//! `minimum_test_timeout >= 300` (optionally paired with
//! `timeout_multiplier`).
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

/// Strip a trailing `# comment` from a TOML-ish value, then trim whitespace.
///
/// F-S147P2-002: this deliberately does NOT strip surrounding quotes. A
/// quoted value like `minimum_test_timeout = "300"` is a TOML *string*, not
/// an *integer* — cargo-mutants' `Config` struct types `minimum_test_timeout`
/// as an integer field, so a quoted `"300"` is a TOML type error the real
/// TOML parser rejects at config-load time, not a valid alternate spelling.
/// If this helper silently stripped the quotes, `"300".parse::<u64>()` would
/// succeed and the test would wrongly treat a broken config as valid. Numeric
/// keys (`minimum_test_timeout`, `timeout_multiplier`,
/// `build_timeout_multiplier`) MUST fail to parse when quoted; only
/// comments/whitespace are stripped here.
fn clean_value(raw: &str) -> String {
    let without_comment = match raw.find('#') {
        Some(idx) => &raw[..idx],
        None => raw,
    };
    without_comment.trim().to_string()
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

/// The path cargo-mutants actually reads a config file from by default —
/// and, per STORY-147 v2.2 / F-S147P1-002/-004, the ONLY such path. A
/// repo-root `mutants.toml` and a `Cargo.toml [package.metadata.mutants]`
/// table are both silently ignored by cargo-mutants and are deliberately
/// NOT checked anywhere in this file (see
/// https://mutants.rs/config-file.html).
fn dot_cargo_mutants_toml_path() -> PathBuf {
    repo_root().join(".cargo").join("mutants.toml")
}

/// Parsed evidence extracted from `.cargo/mutants.toml` content.
#[derive(Debug, Default)]
struct MutantsConfig {
    /// Present iff a `jobs = ...` line exists anywhere in the file. `jobs`
    /// is NOT a valid `Config` field (the parser is
    /// `#[serde(deny_unknown_fields)]`); its presence is a FATAL defect,
    /// not merely unhelpful — it would abort every `cargo mutants` run
    /// with a parse error. Parallelism is CLI/env-only (`--jobs`,
    /// `CARGO_MUTANTS_JOBS`).
    has_jobs_key: bool,
    /// `minimum_test_timeout` value, if the key is present and parses as
    /// an integer number of seconds.
    minimum_test_timeout: Option<u64>,
    /// `timeout_multiplier` (accepting the `build_timeout_multiplier`
    /// spelling too), if present and parses as a number. Optional pairing
    /// with `minimum_test_timeout` per AC-147-001.
    timeout_multiplier: Option<f64>,
}

fn scan_mutants_config(content: &str) -> MutantsConfig {
    let mut config = MutantsConfig::default();
    for line in content.lines() {
        if let Some((key, value)) = parse_key_value(line) {
            match key.as_str() {
                "jobs" => config.has_jobs_key = true,
                "minimum_test_timeout" => {
                    config.minimum_test_timeout = value.parse::<u64>().ok();
                }
                "timeout_multiplier" | "build_timeout_multiplier" => {
                    config.timeout_multiplier = value.parse::<f64>().ok();
                }
                _ => {}
            }
        }
    }
    config
}

/// Does the parsed config set a generous timeout floor at all (either key)?
/// Used by AC-147-001, which allows `timeout_multiplier` as an optional
/// pairing alongside `minimum_test_timeout`.
fn timeout_floor_present(config: &MutantsConfig) -> bool {
    config.minimum_test_timeout.is_some() || config.timeout_multiplier.is_some()
}

/// F-S147P2-001: the complete, authoritative set of top-level keys accepted
/// by cargo-mutants v27.1.0's `Config` struct (`src/config.rs`), which is
/// `#[serde(deny_unknown_fields)]`. Pinned to v27.1.0 — re-derive from
/// upstream `src/config.rs` before bumping the pinned cargo-mutants version.
/// A `jobs` key is deliberately absent from this list: `jobs` is CLI/env-only
/// (`--jobs`, `CARGO_MUTANTS_JOBS`), never a `Config` struct field. Any
/// top-level key in `.cargo/mutants.toml` that is not in this list would
/// cause `deny_unknown_fields` to fatally abort every `cargo mutants` run at
/// config-load time, before a single mutant is even generated.
const MUTANTS_TOML_V27_1_0_ALLOWED_KEYS: &[&str] = &[
    "additional_cargo_args",
    "additional_cargo_test_args",
    "all_features",
    "build_timeout_multiplier",
    "cap_lints",
    "copy_vcs",
    "copy_target",
    "error_values",
    "examine_globs",
    "examine_re",
    "exclude_globs",
    "exclude_re",
    "gitignore",
    "features",
    "minimum_test_timeout",
    "no_default_features",
    "output",
    "profile",
    "skip_calls",
    "skip_calls_defaults",
    "test_package",
    "test_workspace",
    "timeout_multiplier",
    "common",
];

/// Every top-level `key = value` line found in `content`, in file order.
/// Section headers (`[...]`), blank lines, and comment-only lines are
/// excluded by `parse_key_value`. Keys nested under a `[table]` header are
/// NOT distinguished from top-level keys by this line-oriented scan — see
/// the module doc comment for why a full TOML parser is deliberately not
/// used here; `.cargo/mutants.toml` as delivered has no `[table]` sections,
/// so this is not a false-negative risk in practice.
fn top_level_keys(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(parse_key_value)
        .map(|(key, _value)| key)
        .collect()
}

// ---------------------------------------------------------------------------
// AC-147-001: `.cargo/mutants.toml` — the ONLY location cargo-mutants reads
// config from by default — exists and sets a generous timeout floor
// (`minimum_test_timeout` >= 300, optionally paired with
// `timeout_multiplier`). A repo-root `mutants.toml` and a
// `Cargo.toml [package.metadata.mutants]` table are NOT valid substitutes —
// cargo-mutants silently ignores both, so they are not checked here at all.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_001_dot_cargo_mutants_toml_sets_timeout_floor() {
    let path = dot_cargo_mutants_toml_path();
    let content = read_if_exists(&path);

    assert!(
        content.is_some(),
        "AC-147-001: `.cargo/mutants.toml` does not exist at {path:?}. This is the ONLY \
         location cargo-mutants reads config from by default (see \
         https://mutants.rs/config-file.html) — a repo-root `mutants.toml` and a \
         `Cargo.toml [package.metadata.mutants]` table are both silently ignored and are NOT \
         valid substitutes."
    );

    let config = scan_mutants_config(content.as_deref().unwrap_or_default());
    assert!(
        timeout_floor_present(&config),
        "AC-147-001: `.cargo/mutants.toml` does not set a generous timeout floor — expected \
         `minimum_test_timeout >= 300` (optionally paired with `timeout_multiplier`); found \
         minimum_test_timeout={:?}, timeout_multiplier={:?}.",
        config.minimum_test_timeout,
        config.timeout_multiplier
    );
    assert!(
        config.minimum_test_timeout.is_some_and(|t| t >= 300),
        "AC-147-001: `.cargo/mutants.toml` `minimum_test_timeout` is not set to >= 300 (found \
         {:?}).",
        config.minimum_test_timeout
    );
}

// ---------------------------------------------------------------------------
// AC-147-002: file-content verification only (no `cargo mutants` invocation
// required — see F-S147P1-005). `.cargo/mutants.toml` exists at the exact
// path cargo-mutants reads, contains NO `jobs` key (a FATAL defect under
// `#[serde(deny_unknown_fields)]` — it would abort every run with a parse
// error, not merely fail to help with parallelism), the timeout-floor keys
// present parse as `key = <number>`, and `minimum_test_timeout >= 300` is
// set. A repo-root `mutants.toml` decoy must NOT exist: such a file is
// silently ignored by cargo-mutants but misleads a developer into thinking
// it configures anything.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_002_config_content_valid_and_no_decoy_present() {
    let path = dot_cargo_mutants_toml_path();
    let content = read_if_exists(&path);

    assert!(
        content.is_some(),
        "AC-147-002: `.cargo/mutants.toml` does not exist at the exact path cargo-mutants \
         reads by default ({path:?})."
    );

    let content = content.unwrap_or_default();
    let config = scan_mutants_config(&content);

    assert!(
        !config.has_jobs_key,
        "AC-147-002: `.cargo/mutants.toml` contains a `jobs` key. `jobs` is NOT a valid \
         `Config` field — the parser is `#[serde(deny_unknown_fields)]`, so this would abort \
         EVERY `cargo mutants` run with a FATAL parse error, not merely fail to configure \
         parallelism. Parallelism is CLI/env-only (`--jobs`, `CARGO_MUTANTS_JOBS`)."
    );

    assert!(
        config.minimum_test_timeout.is_some() || config.timeout_multiplier.is_some(),
        "AC-147-002: `.cargo/mutants.toml` does not contain a `minimum_test_timeout` or \
         `timeout_multiplier` line that parses as `key = <number>` — no documented-valid \
         timeout-floor key was found."
    );

    assert!(
        config.minimum_test_timeout.is_some_and(|t| t >= 300),
        "AC-147-002: `.cargo/mutants.toml` does not set `minimum_test_timeout >= 300` (found \
         {:?}).",
        config.minimum_test_timeout
    );

    let decoy_path = repo_root().join("mutants.toml");
    assert!(
        !decoy_path.exists(),
        "AC-147-002 (decoy-absence): a repo-root `mutants.toml` exists at {decoy_path:?}. \
         cargo-mutants does NOT read this location by default — an inert decoy file here \
         misleads developers into believing it configures mutation testing when it silently \
         does nothing."
    );
}

// ---------------------------------------------------------------------------
// F-S147P2-001: `.cargo/mutants.toml` contains ONLY keys recognized by
// cargo-mutants v27.1.0's `#[serde(deny_unknown_fields)]` `Config` struct.
// This test is the AC-147-002 anchor for that "no other unrecognized key"
// clause: the earlier "only check for `jobs`" logic caught the one
// concretely-known-bad key but would silently pass ANY other unrecognized
// key (e.g. a typo like `minumum_test_timeout`, or a plausible-sounding but
// nonexistent field) — each of which is equally fatal: `deny_unknown_fields`
// aborts every `cargo mutants` run with a parse error before any mutant
// testing occurs. This test replaces "absence of one bad key" with
// "membership in the full allowlist" as the guard.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_002_config_keys_are_all_in_v27_1_0_allowlist() {
    let path = dot_cargo_mutants_toml_path();
    let content = read_if_exists(&path);

    assert!(
        content.is_some(),
        "F-S147P2-001: `.cargo/mutants.toml` does not exist at {path:?}."
    );

    let content = content.unwrap_or_default();
    let unrecognized: Vec<String> = top_level_keys(&content)
        .into_iter()
        .filter(|k| !MUTANTS_TOML_V27_1_0_ALLOWED_KEYS.contains(&k.as_str()))
        .collect();

    assert!(
        unrecognized.is_empty(),
        "F-S147P2-001: `.cargo/mutants.toml` contains key(s) {unrecognized:?} not recognized by \
         cargo-mutants v27.1.0's `Config` struct (allowlist pinned to v27.1.0 `src/config.rs`, \
         `#[serde(deny_unknown_fields)]`). An unrecognized key would FATALLY ABORT every \
         `cargo mutants` run at config-load time — not merely fail to configure something. \
         Allowed keys: {MUTANTS_TOML_V27_1_0_ALLOWED_KEYS:?}."
    );

    // Distinct explicit `jobs` check kept alongside the allowlist check: this
    // gives a clearer, more specific failure message for the one concretely
    // known historical incident (PG-MUTANTS-JOBS-001) than the generic
    // allowlist-membership message above would on its own.
    let config = scan_mutants_config(&content);
    assert!(
        !config.has_jobs_key,
        "F-S147P2-001: `.cargo/mutants.toml` contains a `jobs` key. `jobs` is NOT a valid \
         `Config` field — it is CLI/env-only (`--jobs`, `CARGO_MUTANTS_JOBS`). This would abort \
         EVERY `cargo mutants` run with a FATAL parse error under `deny_unknown_fields`."
    );
}

// ---------------------------------------------------------------------------
// AC-147-003: CLAUDE.md contains a "Mutation testing" note covering:
//   (a) recommended invocation stays low-parallelism (bare `cargo mutants`,
//       already serial by default, or explicit `--jobs 1` /
//       `CARGO_MUTANTS_JOBS=1`) and WARNS that a high `--jobs` (e.g. 8)
//       caused the PG-MUTANTS-JOBS-001 incident and that no config file can
//       override an explicit CLI `--jobs` flag
//   (b) rationale (infinite-loop mutants inflate wall-clock past auto-timeout
//       -> false "0 missed")
//   (c) PG-MUTANTS-JOBS-001 and the fix-tls-clienthello-frag F6 cycle
//       (F-S147P1-003)
//   (d) drbothen/vsdd-factory#654 pointer
//   (e) the config-file defense is a `.cargo/mutants.toml`
//       `minimum_test_timeout` timeout floor, not a parallelism default —
//       `jobs` is not a config key at all
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_003_claude_md_has_mutation_testing_section() {
    let claude_md = read_if_exists(&repo_root().join("CLAUDE.md")).unwrap_or_default();
    let lower = claude_md.to_lowercase();

    assert!(
        claude_md.contains("Mutation testing"),
        "AC-147-003: CLAUDE.md is missing a \"Mutation testing\" section/heading."
    );
    assert!(
        claude_md.contains("--jobs"),
        "AC-147-003(a): CLAUDE.md does not mention `--jobs` at all (recommended invocation \
         and/or the high-`--jobs` warning)."
    );
    assert!(
        lower.contains("serial"),
        "AC-147-003(a): CLAUDE.md does not state that bare `cargo mutants` is already serial \
         by default."
    );
    // Tightened per Pass-2 test-strength observation: the original
    // `lower.contains("timeout")` disjunct was a tautology against this
    // file's own content, because `minimum_test_timeout` (asserted a few
    // lines below as AC-147-003(e)'s marker) always contains the substring
    // "timeout" — so this assertion could never actually fail on its own,
    // regardless of whether the rationale was explained at all. Requiring
    // "0 missed" AND an explicit causal-mechanism marker
    // ("infinite-loop" or "wall-clock") ties the assertion to the actual
    // rationale prose instead of a substring that is present for unrelated
    // reasons.
    assert!(
        lower.contains("false")
            && lower.contains("0 missed")
            && (lower.contains("infinite-loop") || lower.contains("wall-clock")),
        "AC-147-003(b): CLAUDE.md does not explain the rationale — infinite-loop mutants \
         inflating wall-clock past the auto-timeout threshold, producing a false \"0 missed\" \
         result. Expected \"false\" + \"0 missed\" + (\"infinite-loop\" or \"wall-clock\") all \
         present."
    );
    assert!(
        claude_md.contains("PG-MUTANTS-JOBS-001"),
        "AC-147-003(c): CLAUDE.md does not reference the process-gap ID PG-MUTANTS-JOBS-001."
    );
    assert!(
        claude_md.contains("fix-tls-clienthello-frag"),
        "AC-147-003(c): CLAUDE.md does not reference the fix-tls-clienthello-frag F6 cycle \
         (F-S147P1-003)."
    );
    assert!(
        claude_md.contains("#654"),
        "AC-147-003(d): CLAUDE.md does not reference drbothen/vsdd-factory#654 as the \
         upstream engine-default tracking issue pointer."
    );
    assert!(
        claude_md.contains("minimum_test_timeout"),
        "AC-147-003(e): CLAUDE.md does not note that the config-file defense is a \
         `.cargo/mutants.toml` `minimum_test_timeout` timeout floor, not a parallelism \
         default."
    );
}

// ---------------------------------------------------------------------------
// AC-147-004 (self-audit): after this story ships, a developer running
// `cargo mutants` from a fresh checkout will not silently receive a
// false-clean result due to load-induced timeouts. Conjunction of the two
// REAL defenses per v2.2: first line of defense — the `.cargo/mutants.toml`
// timeout floor (machine-enforced, no `jobs` key, `minimum_test_timeout >=
// 300`); second line of defense — the CLAUDE.md note (human-facing guidance
// against explicit high `--jobs`). This test only verifies the mechanical
// precondition that both defenses actually exist; the human-facing
// self-audit narrative is recorded in the story's demo-evidence step, not
// re-derived here.
// ---------------------------------------------------------------------------
#[test]
fn test_AC_147_004_both_real_defenses_present_simultaneously() {
    let dot_cargo_content = read_if_exists(&dot_cargo_mutants_toml_path());
    let config_defense = dot_cargo_content
        .as_deref()
        .map(scan_mutants_config)
        .is_some_and(|c| !c.has_jobs_key && c.minimum_test_timeout.is_some_and(|t| t >= 300));

    let claude_md = read_if_exists(&repo_root().join("CLAUDE.md")).unwrap_or_default();
    let doc_defense =
        claude_md.contains("Mutation testing") && claude_md.contains("PG-MUTANTS-JOBS-001");

    assert!(
        config_defense,
        "AC-147-004: first line of defense (`.cargo/mutants.toml` timeout floor — no `jobs` \
         key, `minimum_test_timeout >= 300`) is absent or invalid — see AC-147-001/002 \
         failures for detail."
    );
    assert!(
        doc_defense,
        "AC-147-004: second line of defense (CLAUDE.md \"Mutation testing\" note referencing \
         PG-MUTANTS-JOBS-001) is absent — see AC-147-003 failures for detail."
    );
    assert!(
        config_defense && doc_defense,
        "AC-147-004: self-audit failed — a fresh checkout today does NOT have both real \
         defenses in place simultaneously (config_defense={config_defense}, \
         doc_defense={doc_defense}), so `cargo mutants` run bare could still silently produce \
         a false \"0 missed\" result under load."
    );
}

// ---------------------------------------------------------------------------
// Scanner self-checks (Pass-2 guard-hardening). These exercise
// `clean_value`/`parse_key_value`/`scan_mutants_config`/`top_level_keys`
// directly against synthetic in-memory TOML-ish strings — NOT against the
// repo's real `.cargo/mutants.toml` — so they prove the guards themselves
// actually discriminate good from bad input, rather than merely happening to
// pass against a currently-correct file.
// ---------------------------------------------------------------------------

/// F-S147P2-002 regression guard: a quoted numeric value is a TOML *string*,
/// which cargo-mutants' `Config` struct (typed `minimum_test_timeout: usize`)
/// rejects as a type error. If `clean_value` stripped quotes before numeric
/// parsing, this synthetic `"300"` would wrongly parse as `Some(300)`.
#[test]
fn test_F_S147P2_002_quoted_minimum_test_timeout_does_not_parse_as_valid() {
    let synthetic = "minimum_test_timeout = \"300\"\n";
    let config = scan_mutants_config(synthetic);
    assert!(
        config.minimum_test_timeout.is_none(),
        "F-S147P2-002: `minimum_test_timeout = \"300\"` (a quoted TOML string) parsed as a \
         valid integer timeout ({:?}) — `clean_value` must not strip quotes before numeric \
         parsing, since a quoted numeric is a TOML type error cargo-mutants rejects.",
        config.minimum_test_timeout
    );
}

/// Companion positive case: the same key, unquoted, must still parse — this
/// guards against a fix that over-corrects by breaking the happy path too.
#[test]
fn test_F_S147P2_002_unquoted_minimum_test_timeout_still_parses() {
    let synthetic = "minimum_test_timeout = 300\n";
    let config = scan_mutants_config(synthetic);
    assert_eq!(
        config.minimum_test_timeout,
        Some(300),
        "F-S147P2-002 regression: an unquoted `minimum_test_timeout = 300` must still parse as \
         `Some(300)` — quote-strictness must not break the valid, unquoted form."
    );
}

/// F-S147P2-001 regression guard: an unrecognized key (neither a real
/// v27.1.0 `Config` field nor `jobs`) must be flagged by the allowlist scan.
/// The prior "only check for `jobs`" logic would have silently missed this.
#[test]
fn test_F_S147P2_001_allowlist_scan_flags_unrecognized_key() {
    let synthetic = "minimum_test_timeout = 300\nbogus_typo_field = true\n";
    let unrecognized: Vec<String> = top_level_keys(synthetic)
        .into_iter()
        .filter(|k| !MUTANTS_TOML_V27_1_0_ALLOWED_KEYS.contains(&k.as_str()))
        .collect();
    assert_eq!(
        unrecognized,
        vec!["bogus_typo_field".to_string()],
        "F-S147P2-001: the allowlist scan did not flag a synthetic unrecognized key \
         (`bogus_typo_field`) that is neither `jobs` nor a real v27.1.0 `Config` field."
    );
}

/// Companion positive case: every key in the pinned allowlist itself must be
/// accepted as recognized (i.e. the allowlist scan has no false positives
/// against its own reference list).
#[test]
fn test_F_S147P2_001_allowlist_scan_accepts_all_pinned_v27_1_0_keys() {
    let synthetic: String = MUTANTS_TOML_V27_1_0_ALLOWED_KEYS
        .iter()
        .map(|k| format!("{k} = true\n"))
        .collect();
    let unrecognized: Vec<String> = top_level_keys(&synthetic)
        .into_iter()
        .filter(|k| !MUTANTS_TOML_V27_1_0_ALLOWED_KEYS.contains(&k.as_str()))
        .collect();
    assert!(
        unrecognized.is_empty(),
        "F-S147P2-001: the allowlist scan false-flagged pinned v27.1.0 key(s) {unrecognized:?} \
         as unrecognized."
    );
}
