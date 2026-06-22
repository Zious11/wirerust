//! Local-only E2E corpus smoke test.
//!
//! This test iterates every capture file present in `tests/fixtures/local-samples/`
//! and asserts that none of them panic when passed through
//! [`wirerust::reader::PcapSource::from_file`] — the same entry point used by
//! `src/main.rs`. Both `Ok` and `Err` results are accepted; the contract is
//! strictly **no-panic / no-unwind**, not "must parse successfully."
//!
//! Several captures in the corpus are *documented* to return clean errors:
//! - `pcapng-spb-only.pcapng` → E-INP-010 (IfFcsLen IDB option rejection)
//! - `pcapng-example.pcapng`  → E-INP-011 (multi-IDB link-type conflict)
//! - `pcapng-many-interfaces.pcapng` → unsupported link type NULL
//!
//! These `Err` results are expected and count as PASS.
//!
//! # Fixture hygiene
//!
//! The captures are **gitignored** and not stored in the repository. To
//! reproduce them locally run:
//!
//! ```bash
//! bin/fetch-e2e-pcaps
//! ```
//!
//! This places 28 capture files under `tests/fixtures/local-samples/`.
//! When that directory is absent or empty the test self-skips (prints a notice
//! and returns `Ok`) so CI — which has no local-samples — stays green.
//!
//! # Decision-thread reference
//!
//! Implements decision-thread (c) from `.factory/STATE.md` / PCAP-CORPUS-001:
//! "a local-only test that iterates all fetched E2E captures asserting each
//! parses without panic."

use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use wirerust::reader::PcapSource;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all capture files (`.pcap`, `.pcapng`, `.cap`) from `dir`,
/// sorted for deterministic ordering. Non-capture files (e.g. `README.md`)
/// are silently skipped.
fn collect_captures(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            // Only regular files with a recognized capture extension.
            if !p.is_file() {
                return false;
            }
            matches!(
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase())
                    .as_deref(),
                Some("pcap" | "pcapng" | "cap")
            )
        })
        .collect();

    paths.sort();
    paths
}

// ---------------------------------------------------------------------------
// Outcome enum (for per-file summary reporting)
// ---------------------------------------------------------------------------

enum Outcome {
    Ok { packet_count: usize },
    Err(String),
    Panic(String),
}

// ---------------------------------------------------------------------------
// The smoke test
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_corpus_no_panic_across_all_local_captures() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local-samples");

    // ── Self-skip when fixtures are absent or empty ───────────────────────────
    if !fixtures_dir.exists() {
        eprintln!(
            "[e2e-corpus-smoke] SKIP: `tests/fixtures/local-samples/` not found. \
             Run `bin/fetch-e2e-pcaps` to populate it."
        );
        return;
    }

    let captures = collect_captures(&fixtures_dir);

    if captures.is_empty() {
        eprintln!(
            "[e2e-corpus-smoke] SKIP: `tests/fixtures/local-samples/` is empty. \
             Run `bin/fetch-e2e-pcaps` to populate it."
        );
        return;
    }

    // ── Exercise each capture ─────────────────────────────────────────────────
    let mut panic_files: Vec<String> = Vec::new();
    let mut ok_count = 0usize;
    let mut err_count = 0usize;

    eprintln!(
        "\n[e2e-corpus-smoke] Exercising {} capture(s):",
        captures.len()
    );

    for path in &captures {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_owned();

        // Wrap in catch_unwind so a single panicking capture doesn't abort the
        // whole run — we want to report all offenders in one go.
        let path_clone = path.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| PcapSource::from_file(&path_clone)));

        let outcome = match result {
            Ok(Ok(source)) => {
                let n = source.packets.len();
                ok_count += 1;
                Outcome::Ok { packet_count: n }
            }
            Ok(Err(e)) => {
                err_count += 1;
                Outcome::Err(format!("{e:#}"))
            }
            Err(payload) => {
                // Extract a human-readable message from the panic payload.
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    (*s).to_owned()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "<non-string panic payload>".to_owned()
                };
                panic_files.push(file_name.clone());
                Outcome::Panic(msg)
            }
        };

        // Per-file summary line.
        match &outcome {
            Outcome::Ok { packet_count } => {
                eprintln!("  OK  ({:>5} pkts)  {}", packet_count, file_name);
            }
            Outcome::Err(msg) => {
                // Truncate long error messages for readability.
                let short: String = msg.chars().take(120).collect();
                eprintln!("  ERR           {}  [{}]", file_name, short);
            }
            Outcome::Panic(msg) => {
                eprintln!("  PANIC         {}  [{}]", file_name, msg);
            }
        }
    }

    // ── Aggregate summary ─────────────────────────────────────────────────────
    eprintln!(
        "\n[e2e-corpus-smoke] Results: {} OK, {} ERR (expected), {} PANIC",
        ok_count,
        err_count,
        panic_files.len()
    );

    // At least one capture must have been exercised when the directory was
    // non-empty (guards against a bug in collect_captures silently skipping all).
    assert!(
        ok_count + err_count + panic_files.len() > 0,
        "collect_captures returned paths but no capture was actually exercised — \
         this is a bug in the test harness"
    );

    // The primary contract: no capture may panic.
    assert!(
        panic_files.is_empty(),
        "[e2e-corpus-smoke] {} capture(s) caused a panic: {}",
        panic_files.len(),
        panic_files.join(", ")
    );
}
