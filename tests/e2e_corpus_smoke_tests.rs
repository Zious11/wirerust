//! Local-only E2E corpus smoke test — pinned-expectation regression guard.
//!
//! This test iterates every capture file present in `tests/fixtures/local-samples/`
//! and passes each through [`wirerust::reader::PcapSource::from_file`] — the same
//! entry point used by `src/main.rs`.
//!
//! # Pinned-expectation contract
//!
//! A static `EXPECTED` table maps each known capture filename to one of two
//! expected outcomes:
//!
//! - `Ok(packet_count)` — `PcapSource::from_file` returns `Ok` and
//!   `source.packets.len()` equals the pinned count exactly.
//! - `Err(substr)` — `from_file` returns `Err` and the error's `Display`
//!   string **contains** the pinned substring (stable, structured token
//!   preferred, e.g. `E-INP-010`).
//!
//! These pins are regression guards on **reader-level** packet counts (which
//! may differ from analyzer-level counts in the manifest). If any pinned
//! capture produces a different result, the test fails and names the offender.
//!
//! Files present on disk that are NOT in the table are still exercised for
//! no-panic (regression safety for dev-added captures) but do NOT cause a
//! test failure — they are reported as `UNPINNED`.
//!
//! # Self-skip behaviour
//!
//! When `tests/fixtures/local-samples/` is absent or has zero capture files,
//! the test prints a skip notice (mentioning `bin/fetch-e2e-pcaps`) and
//! passes immediately. CI — which has no local-samples — stays green.
//! **`#[ignore]` is not used** so the test remains in the default test run.
//!
//! # Reproducing fixtures
//!
//! ```bash
//! bin/fetch-e2e-pcaps
//! ```
//!
//! This places the 29 canonical captures under `tests/fixtures/local-samples/`.
//!
//! # Decision-thread reference
//!
//! Implements decision-thread (c) from `.factory/STATE.md` / PCAP-CORPUS-001:
//! "a local-only test that iterates all fetched E2E captures asserting each
//! parses without panic." The pinned expectation table strengthens this to
//! catch behavioral regressions (wrong packet count, changed error class), not
//! just panics.

use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use wirerust::reader::PcapSource;

// ---------------------------------------------------------------------------
// Pinned expectation table
// ---------------------------------------------------------------------------

/// The expected outcome for a single capture file.
#[derive(Debug)]
enum Expected {
    /// `from_file` must return `Ok` and `source.packets.len()` must equal this.
    OkCount(usize),
    /// `from_file` must return `Err` whose `Display` string contains this
    /// stable substring (typically a structured error code like `E-INP-010`).
    ErrContains(&'static str),
}

/// Canonical per-capture expectation table.
///
/// Keys are bare filenames (no directory prefix). Values were recorded from
/// the first verified run on the fetched corpus and are the authoritative
/// reader-level baseline.
const EXPECTED: &[(&str, Expected)] = &[
    // ── Ok captures (reader packet count) ─────────────────────────────────
    ("220703_arp-storm-nrb.pcapng", Expected::OkCount(622)),
    ("4SICS-GeekLounge-151020.pcap", Expected::OkCount(246137)),
    ("4SICS-GeekLounge-151021.pcap", Expected::OkCount(1253100)),
    ("4SICS-GeekLounge-151022.pcap", Expected::OkCount(2274747)),
    ("arp-baseline-16pkt.cap", Expected::OkCount(16)),
    ("arp-storm.pcap", Expected::OkCount(622)),
    ("arpspoof.pcap", Expected::OkCount(16285)),
    ("dhcp-big-endian.pcapng", Expected::OkCount(4)),
    ("dhcp-nanosecond-test.pcapng", Expected::OkCount(4)),
    ("dnp3dataset_capture.pcap", Expected::OkCount(26058)),
    ("dns-tunnel-dns2tcp.pcap", Expected::OkCount(26)),
    ("dns-tunnel-dnscat2.pcap", Expected::OkCount(24)),
    ("dns-tunnel-iodine-dmachard.pcap", Expected::OkCount(24)),
    ("dns-tunnel-iodine.pcap", Expected::OkCount(438)),
    ("dtls12-dsb.pcapng", Expected::OkCount(13)),
    ("gratuitous-arp-hsrp.cap", Expected::OkCount(6)),
    ("http-brotli-isb.pcapng", Expected::OkCount(10)),
    ("http-creds-set4.pcap", Expected::OkCount(170)),
    ("http-malspam-set6.pcap", Expected::OkCount(738)),
    ("http-ppa-baseline.cap", Expected::OkCount(43)),
    ("ip-frag-teardrop.cap", Expected::OkCount(17)),
    ("modbus-large.pcap", Expected::OkCount(85)),
    ("pcapng-comments.pcapng", Expected::OkCount(5)),
    ("pcapng-dhcp-little-endian.pcapng", Expected::OkCount(4)),
    ("ppa-arp.pcap", Expected::OkCount(2)),
    ("rsasnakeoil2.pcap", Expected::OkCount(58)),
    // ── Err captures (stable error substring) ─────────────────────────────
    // E-INP-011: multi-IDB link-type conflict (message ends with "(E-INP-011)")
    ("pcapng-example.pcapng", Expected::ErrContains("E-INP-011")),
    // NULL link type not supported; message leads with "Unsupported pcap link type: NULL"
    (
        "pcapng-many-interfaces.pcapng",
        Expected::ErrContains("Unsupported pcap link type: NULL"),
    ),
    // E-INP-010: IfFcsLen IDB option rejection
    ("pcapng-spb-only.pcapng", Expected::ErrContains("E-INP-010")),
];

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

/// Build a lookup from filename → `Expected` from the static table.
fn expected_map() -> std::collections::HashMap<&'static str, &'static Expected> {
    EXPECTED.iter().map(|(name, exp)| (*name, exp)).collect()
}

// ---------------------------------------------------------------------------
// Actual outcome (runtime)
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

    let exp_map = expected_map();

    // ── Per-file tracking ─────────────────────────────────────────────────────
    let mut panic_files: Vec<String> = Vec::new();
    let mut mismatches: Vec<String> = Vec::new();
    let mut verified_count = 0usize;

    eprintln!(
        "\n[e2e-corpus-smoke] Exercising {} capture(s) against pinned expectation table:",
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
                Outcome::Ok { packet_count: n }
            }
            Ok(Err(e)) => Outcome::Err(format!("{e:#}")),
            Err(payload) => {
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

        // ── Compare against pinned expectation ────────────────────────────
        let label = match exp_map.get(file_name.as_str()) {
            Some(Expected::OkCount(expected_n)) => {
                // File is in the table — verify it.
                verified_count += 1;
                match &outcome {
                    Outcome::Ok { packet_count } if packet_count == expected_n => {
                        format!("OK({packet_count})")
                    }
                    Outcome::Ok { packet_count } => {
                        let msg = format!(
                            "{file_name}: expected Ok({expected_n}) but got Ok({packet_count})"
                        );
                        mismatches.push(msg);
                        format!("MISMATCH(exp Ok({expected_n})/act Ok({packet_count}))")
                    }
                    Outcome::Err(e) => {
                        let short: String = e.chars().take(80).collect();
                        let msg =
                            format!("{file_name}: expected Ok({expected_n}) but got Err({short})");
                        mismatches.push(msg);
                        format!("MISMATCH(exp Ok({expected_n})/act ERR)")
                    }
                    Outcome::Panic(p) => {
                        // Already recorded in panic_files above.
                        format!("PANIC({p})")
                    }
                }
            }
            Some(Expected::ErrContains(substr)) => {
                verified_count += 1;
                match &outcome {
                    Outcome::Err(e) if e.contains(substr) => {
                        format!("ERR({substr})")
                    }
                    Outcome::Err(e) => {
                        let short: String = e.chars().take(80).collect();
                        let msg = format!(
                            "{file_name}: expected Err containing {substr:?} but \
                             got Err({short})"
                        );
                        mismatches.push(msg);
                        format!("MISMATCH(exp ERR({substr})/act ERR(different))")
                    }
                    Outcome::Ok { packet_count } => {
                        let msg = format!(
                            "{file_name}: expected Err containing {substr:?} but \
                             got Ok({packet_count})"
                        );
                        mismatches.push(msg);
                        format!("MISMATCH(exp ERR({substr})/act Ok({packet_count}))")
                    }
                    Outcome::Panic(p) => {
                        format!("PANIC({p})")
                    }
                }
            }
            None => {
                // Unknown capture — still exercise for no-panic but don't fail.
                match &outcome {
                    Outcome::Ok { packet_count } => {
                        format!("UNPINNED Ok({packet_count}) -- add to expected table")
                    }
                    Outcome::Err(e) => {
                        let short: String = e.chars().take(80).collect();
                        format!("UNPINNED ERR({short}) -- add to expected table")
                    }
                    Outcome::Panic(p) => {
                        format!("UNPINNED PANIC({p})")
                    }
                }
            }
        };

        eprintln!("  {}  {}", label, file_name);
    }

    // ── Aggregate summary ─────────────────────────────────────────────────────
    eprintln!(
        "\n[e2e-corpus-smoke] Verified {verified_count} pinned entries, \
         {} mismatch(es), {} panic(s)",
        mismatches.len(),
        panic_files.len()
    );

    // When fixtures were present, at least one expected entry must have been
    // verified (guards against a bug where collect_captures skips everything or
    // the EXPECTED table is somehow empty).
    assert!(
        verified_count > 0,
        "[e2e-corpus-smoke] No pinned captures were verified — either the \
         EXPECTED table is empty or no expected filenames matched anything on \
         disk. This is a test-harness bug."
    );

    // Primary contract: no capture may panic.
    assert!(
        panic_files.is_empty(),
        "[e2e-corpus-smoke] {} capture(s) caused a panic: {}",
        panic_files.len(),
        panic_files.join(", ")
    );

    // Regression contract: every pinned expectation that is present on disk
    // must match exactly. Collect all failures and report them together.
    assert!(
        mismatches.is_empty(),
        "[e2e-corpus-smoke] {} pinned expectation(s) did not match:\n  - {}",
        mismatches.len(),
        mismatches.join("\n  - ")
    );
}
