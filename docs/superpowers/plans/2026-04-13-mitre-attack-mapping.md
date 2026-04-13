# MITRE ATT&CK Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `src/mitre.rs` lookup module (tactics + technique names for Enterprise + ICS), assign T1027 to TLS malformed-SNI findings, and wire a `--mitre` flag on `analyze` that regroups terminal output by tactic.

**Architecture:** Keep `mitre_technique: Option<String>` on `Finding`. New `src/mitre.rs` exports `MitreTactic` enum (16 variants), `technique_name(id)`, `technique_tactic(id)`, and `all_tactics_in_report_order()` — all backed by exhaustive `match` statements. `TerminalReporter` grows a `show_mitre_grouping: bool` field; when true, the FINDINGS section is rebuilt as tactic-grouped sub-sections in MITRE canonical order, sorted verdict-desc → confidence-desc within each group. Unknown IDs render as `(unknown)` and bucket under "Uncategorized".

**Tech Stack:** Rust 2024 edition, clap v4 derive API, no new crate dependencies.

---

## File Structure

**New files:**
- `src/mitre.rs` — tactics enum + lookup fns (~200 lines)
- `tests/mitre_tests.rs` — unit + regression coverage

**Modified files:**
- `src/lib.rs` — register the new module
- `src/cli.rs` — add `mitre: bool` flag on `Commands::Analyze`
- `src/main.rs` — destructure `mitre` in the `Commands::Analyze` arm, thread into `run_analyze`, pass to `TerminalReporter`
- `src/reporter/terminal.rs` — add `show_mitre_grouping` field; implement the grouped-render code path
- `src/analyzer/tls.rs` — set `mitre_technique: Some("T1027".to_string())` on 3 of the 7 existing `None` sites
- `tests/reporter_tests.rs` — add grouped-render coverage
- `tests/tls_analyzer_tests.rs` — assert T1027 on the 3 malformed-SNI cases
- `tests/cli_tests.rs` — parse-test the `--mitre` flag

---

### Task 1: Create `src/mitre.rs` with `MitreTactic` enum, `Display`, and `all_tactics_in_report_order`

**Files:**
- Create: `src/mitre.rs`
- Modify: `src/lib.rs`
- Test: `tests/mitre_tests.rs` (create)

- [ ] **Step 1: Write the failing tests**

Create `tests/mitre_tests.rs`:

```rust
use wirerust::mitre::{MitreTactic, all_tactics_in_report_order};

#[test]
fn display_renders_enterprise_tactics_with_canonical_spacing() {
    assert_eq!(MitreTactic::CommandAndControl.to_string(), "Command and Control");
    assert_eq!(MitreTactic::DefenseEvasion.to_string(), "Defense Evasion");
    assert_eq!(MitreTactic::CredentialAccess.to_string(), "Credential Access");
    assert_eq!(MitreTactic::LateralMovement.to_string(), "Lateral Movement");
    assert_eq!(MitreTactic::PrivilegeEscalation.to_string(), "Privilege Escalation");
    assert_eq!(MitreTactic::InitialAccess.to_string(), "Initial Access");
    assert_eq!(MitreTactic::ResourceDevelopment.to_string(), "Resource Development");
    assert_eq!(MitreTactic::Reconnaissance.to_string(), "Reconnaissance");
    assert_eq!(MitreTactic::Execution.to_string(), "Execution");
    assert_eq!(MitreTactic::Persistence.to_string(), "Persistence");
    assert_eq!(MitreTactic::Discovery.to_string(), "Discovery");
    assert_eq!(MitreTactic::Collection.to_string(), "Collection");
    assert_eq!(MitreTactic::Exfiltration.to_string(), "Exfiltration");
    assert_eq!(MitreTactic::Impact.to_string(), "Impact");
}

#[test]
fn display_renders_ics_tactics_unprefixed() {
    assert_eq!(
        MitreTactic::IcsInhibitResponseFunction.to_string(),
        "Inhibit Response Function"
    );
    assert_eq!(
        MitreTactic::IcsImpairProcessControl.to_string(),
        "Impair Process Control"
    );
}

#[test]
fn report_order_starts_with_reconnaissance_and_ends_with_ics() {
    let tactics = all_tactics_in_report_order();
    assert_eq!(tactics.first(), Some(&MitreTactic::Reconnaissance));
    assert_eq!(
        tactics.last(),
        Some(&MitreTactic::IcsImpairProcessControl)
    );
}

#[test]
fn report_order_contains_every_variant_exactly_once() {
    let tactics = all_tactics_in_report_order();
    // Count by roundtripping the discriminant through Debug — avoids
    // needing an explicit variant-count constant on the enum.
    let mut seen: Vec<String> = tactics.iter().map(|t| format!("{t:?}")).collect();
    seen.sort();
    let before = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), before, "duplicate variant in report order");
    assert_eq!(before, 16, "expected 14 Enterprise + 2 ICS-unique = 16 variants");
}

#[test]
fn report_order_matches_enterprise_kill_chain_for_first_14() {
    let tactics = all_tactics_in_report_order();
    let enterprise = [
        MitreTactic::Reconnaissance,
        MitreTactic::ResourceDevelopment,
        MitreTactic::InitialAccess,
        MitreTactic::Execution,
        MitreTactic::Persistence,
        MitreTactic::PrivilegeEscalation,
        MitreTactic::DefenseEvasion,
        MitreTactic::CredentialAccess,
        MitreTactic::Discovery,
        MitreTactic::LateralMovement,
        MitreTactic::Collection,
        MitreTactic::CommandAndControl,
        MitreTactic::Exfiltration,
        MitreTactic::Impact,
    ];
    assert_eq!(&tactics[..14], &enterprise);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test mitre_tests`
Expected: compile error — `wirerust::mitre` does not exist.

- [ ] **Step 3: Create `src/mitre.rs`**

```rust
//! MITRE ATT&CK technique-ID → name / tactic lookup module.
//!
//! Backed by exhaustive `match` statements; zero runtime dependencies.
//! See `docs/superpowers/specs/2026-04-13-mitre-attack-mapping-design.md`
//! for the full design rationale.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MitreTactic {
    // Enterprise canonical order (MITRE ATT&CK v18, 14 tactics).
    Reconnaissance,
    ResourceDevelopment,
    InitialAccess,
    Execution,
    Persistence,
    PrivilegeEscalation,
    DefenseEvasion,
    CredentialAccess,
    Discovery,
    LateralMovement,
    Collection,
    CommandAndControl,
    Exfiltration,
    Impact,
    // ICS-unique tactics (names that don't collide with Enterprise).
    IcsInhibitResponseFunction,
    IcsImpairProcessControl,
}

impl fmt::Display for MitreTactic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MitreTactic::Reconnaissance => "Reconnaissance",
            MitreTactic::ResourceDevelopment => "Resource Development",
            MitreTactic::InitialAccess => "Initial Access",
            MitreTactic::Execution => "Execution",
            MitreTactic::Persistence => "Persistence",
            MitreTactic::PrivilegeEscalation => "Privilege Escalation",
            MitreTactic::DefenseEvasion => "Defense Evasion",
            MitreTactic::CredentialAccess => "Credential Access",
            MitreTactic::Discovery => "Discovery",
            MitreTactic::LateralMovement => "Lateral Movement",
            MitreTactic::Collection => "Collection",
            MitreTactic::CommandAndControl => "Command and Control",
            MitreTactic::Exfiltration => "Exfiltration",
            MitreTactic::Impact => "Impact",
            MitreTactic::IcsInhibitResponseFunction => "Inhibit Response Function",
            MitreTactic::IcsImpairProcessControl => "Impair Process Control",
        };
        f.write_str(name)
    }
}

/// Returns all tactics in MITRE canonical kill-chain order, with ICS-unique
/// tactics appended last. Used by the terminal reporter to produce a stable
/// section order when grouping findings by tactic.
pub fn all_tactics_in_report_order() -> &'static [MitreTactic] {
    &[
        MitreTactic::Reconnaissance,
        MitreTactic::ResourceDevelopment,
        MitreTactic::InitialAccess,
        MitreTactic::Execution,
        MitreTactic::Persistence,
        MitreTactic::PrivilegeEscalation,
        MitreTactic::DefenseEvasion,
        MitreTactic::CredentialAccess,
        MitreTactic::Discovery,
        MitreTactic::LateralMovement,
        MitreTactic::Collection,
        MitreTactic::CommandAndControl,
        MitreTactic::Exfiltration,
        MitreTactic::Impact,
        MitreTactic::IcsInhibitResponseFunction,
        MitreTactic::IcsImpairProcessControl,
    ]
}
```

- [ ] **Step 4: Register the module in `src/lib.rs`**

Edit `src/lib.rs` — add `pub mod mitre;` alphabetically between `findings` and `reader`:

```rust
pub mod analyzer;
pub mod cli;
pub mod decoder;
pub mod dispatcher;
pub mod findings;
pub mod mitre;
pub mod reader;
pub mod reassembly;
pub mod reporter;
pub mod summary;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test mitre_tests`
Expected: all 5 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/mitre.rs src/lib.rs tests/mitre_tests.rs
git commit -m "feat(mitre): add MitreTactic enum with canonical display + ordering"
```

---

### Task 2: Add `technique_name` and `technique_tactic` lookup functions

**Files:**
- Modify: `src/mitre.rs`
- Test: `tests/mitre_tests.rs`

- [ ] **Step 1: Append failing tests to `tests/mitre_tests.rs`**

```rust
use wirerust::mitre::{technique_name, technique_tactic};

#[test]
fn technique_name_resolves_every_seeded_id() {
    assert_eq!(technique_name("T1027"), Some("Obfuscated Files or Information"));
    assert_eq!(technique_name("T1036"), Some("Masquerading"));
    assert_eq!(technique_name("T1040"), Some("Network Sniffing"));
    assert_eq!(technique_name("T1046"), Some("Network Service Discovery"));
    assert_eq!(technique_name("T1071"), Some("Application Layer Protocol"));
    assert_eq!(technique_name("T1071.001"), Some("Web Protocols"));
    assert_eq!(technique_name("T1071.004"), Some("DNS"));
    assert_eq!(technique_name("T1083"), Some("File and Directory Discovery"));
    assert_eq!(technique_name("T1499.002"), Some("Service Exhaustion Flood"));
    assert_eq!(technique_name("T1505.003"), Some("Web Shell"));
    assert_eq!(technique_name("T1573"), Some("Encrypted Channel"));
    assert_eq!(technique_name("T0846"), Some("Remote System Discovery"));
    assert_eq!(technique_name("T0855"), Some("Unauthorized Command Message"));
    assert_eq!(technique_name("T0856"), Some("Spoof Reporting Message"));
    assert_eq!(technique_name("T0885"), Some("Commonly Used Port"));
}

#[test]
fn technique_name_returns_none_for_unknown_ids() {
    assert_eq!(technique_name("T9999"), None);
    assert_eq!(technique_name(""), None);
    assert_eq!(technique_name("T1046.999"), None);
    assert_eq!(technique_name("garbage"), None);
}

#[test]
fn technique_tactic_matches_spec_table() {
    assert_eq!(technique_tactic("T1027"), Some(MitreTactic::DefenseEvasion));
    assert_eq!(technique_tactic("T1036"), Some(MitreTactic::DefenseEvasion));
    assert_eq!(technique_tactic("T1040"), Some(MitreTactic::CredentialAccess));
    assert_eq!(technique_tactic("T1046"), Some(MitreTactic::Discovery));
    assert_eq!(technique_tactic("T1071"), Some(MitreTactic::CommandAndControl));
    assert_eq!(technique_tactic("T1071.001"), Some(MitreTactic::CommandAndControl));
    assert_eq!(technique_tactic("T1071.004"), Some(MitreTactic::CommandAndControl));
    assert_eq!(technique_tactic("T1083"), Some(MitreTactic::Discovery));
    assert_eq!(technique_tactic("T1499.002"), Some(MitreTactic::Impact));
    assert_eq!(technique_tactic("T1505.003"), Some(MitreTactic::Persistence));
    assert_eq!(technique_tactic("T1573"), Some(MitreTactic::CommandAndControl));
    assert_eq!(technique_tactic("T0846"), Some(MitreTactic::Discovery));
    assert_eq!(
        technique_tactic("T0855"),
        Some(MitreTactic::IcsImpairProcessControl)
    );
    assert_eq!(
        technique_tactic("T0856"),
        Some(MitreTactic::IcsImpairProcessControl)
    );
    assert_eq!(technique_tactic("T0885"), Some(MitreTactic::CommandAndControl));
}

#[test]
fn technique_tactic_returns_none_for_unknown_ids() {
    assert_eq!(technique_tactic("T9999"), None);
    assert_eq!(technique_tactic(""), None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test mitre_tests`
Expected: compile error — `technique_name` / `technique_tactic` undefined.

- [ ] **Step 3: Append the lookup fns to `src/mitre.rs`**

```rust
/// Resolves a MITRE ATT&CK technique ID to its human-readable name.
///
/// Returns `None` for unknown IDs; callers that treat unknowns as
/// programming errors should `debug_assert!` at their call site.
/// The canonical ID format is `TXXXX` for parent techniques and
/// `TXXXX.NNN` for sub-techniques (period separator, three-digit
/// suffix), used consistently across Enterprise, ICS, and Mobile
/// matrices and in STIX 2.1 bundles.
pub fn technique_name(id: &str) -> Option<&'static str> {
    let name = match id {
        // Enterprise.
        "T1027" => "Obfuscated Files or Information",
        "T1036" => "Masquerading",
        "T1040" => "Network Sniffing",
        "T1046" => "Network Service Discovery",
        "T1071" => "Application Layer Protocol",
        "T1071.001" => "Web Protocols",
        "T1071.004" => "DNS",
        "T1083" => "File and Directory Discovery",
        "T1499.002" => "Service Exhaustion Flood",
        "T1505.003" => "Web Shell",
        "T1573" => "Encrypted Channel",
        // ICS.
        "T0846" => "Remote System Discovery",
        "T0855" => "Unauthorized Command Message",
        "T0856" => "Spoof Reporting Message",
        "T0885" => "Commonly Used Port",
        _ => return None,
    };
    Some(name)
}

/// Resolves a MITRE ATT&CK technique ID to its parent tactic.
///
/// For IDs shared in name between Enterprise and ICS (Discovery,
/// Command and Control, etc.) this returns the unified variant — see
/// the spec for the v1 limitation rationale.
pub fn technique_tactic(id: &str) -> Option<MitreTactic> {
    let tactic = match id {
        // Enterprise.
        "T1027" | "T1036" => MitreTactic::DefenseEvasion,
        "T1040" => MitreTactic::CredentialAccess,
        "T1046" | "T1083" => MitreTactic::Discovery,
        "T1071" | "T1071.001" | "T1071.004" | "T1573" => MitreTactic::CommandAndControl,
        "T1499.002" => MitreTactic::Impact,
        "T1505.003" => MitreTactic::Persistence,
        // ICS.
        "T0846" => MitreTactic::Discovery,
        "T0855" | "T0856" => MitreTactic::IcsImpairProcessControl,
        "T0885" => MitreTactic::CommandAndControl,
        _ => return None,
    };
    Some(tactic)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test mitre_tests`
Expected: all tests in `mitre_tests.rs` PASS.

- [ ] **Step 5: Commit**

```bash
git add src/mitre.rs tests/mitre_tests.rs
git commit -m "feat(mitre): add technique_name and technique_tactic lookups"
```

---

### Task 3: Add the coverage regression test

**Files:**
- Test: `tests/mitre_tests.rs`

- [ ] **Step 1: Append the canonical-coverage test to `tests/mitre_tests.rs`**

This test encodes the exact set of IDs the codebase intentionally emits. When any analyzer adds a new `mitre_technique: Some("…")` site, the author must add the ID here — failing to do so fails CI, which is the whole point.

```rust
#[test]
fn every_emitted_technique_id_is_known() {
    // Canonical list of every mitre_technique Some(...) value the codebase
    // emits today. When you add a new emission site in an analyzer or
    // reassembly handler, add the ID here too. Missing entries = CI failure.
    let emitted_ids = [
        // src/analyzer/http.rs
        "T1083",
        "T1505.003",
        "T1046",
        "T1499.002",
        // src/analyzer/tls.rs (added in this feature)
        "T1027",
        // src/reassembly/mod.rs
        "T1036",
    ];

    for id in emitted_ids {
        assert!(
            technique_name(id).is_some(),
            "analyzer emits {id} but technique_name({id}) returned None",
        );
        assert!(
            technique_tactic(id).is_some(),
            "analyzer emits {id} but technique_tactic({id}) returned None",
        );
    }
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test --test mitre_tests every_emitted_technique_id_is_known`
Expected: PASS (the IDs listed already have entries from Task 2, including T1027 which will be wired into TLS in Task 4).

- [ ] **Step 3: Commit**

```bash
git add tests/mitre_tests.rs
git commit -m "test(mitre): add coverage regression test for emitted technique IDs"
```

---

### Task 4: Assign T1027 to the three TLS malformed-SNI findings

**Files:**
- Modify: `src/analyzer/tls.rs`
- Test: `tests/tls_analyzer_tests.rs`

- [ ] **Step 1: Write the failing TLS tests**

Append to `tests/tls_analyzer_tests.rs` (after the existing SNI tests; check the file for a suitable insertion point):

```rust
#[test]
fn ascii_control_sni_finding_sets_mitre_t1027() {
    let esc_hostname = b"foo\x1bbar.example.com";
    let bytes = build_client_hello_ascii_bytes(esc_hostname);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&make_flow_key(), Direction::ClientToServer, &bytes, 0);

    let control_finding = analyzer
        .findings()
        .iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .expect("expected an ASCII-control SNI finding");
    assert_eq!(
        control_finding.mitre_technique.as_deref(),
        Some("T1027"),
        "malformed-SNI finding must be mapped to T1027 (Obfuscated Files or Information)",
    );
}

#[test]
fn non_ascii_utf8_sni_finding_sets_mitre_t1027() {
    // Cyrillic hostname — valid UTF-8 but non-ASCII, so RFC 6066 A-label
    // violation path.
    let bytes = build_client_hello_ascii_bytes("пример.рф".as_bytes());
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&make_flow_key(), Direction::ClientToServer, &bytes, 0);

    let finding = analyzer
        .findings()
        .iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("expected a non-ASCII SNI finding");
    assert_eq!(finding.mitre_technique.as_deref(), Some("T1027"));
}

#[test]
fn non_utf8_sni_finding_sets_mitre_t1027() {
    // Truncated UTF-8 sequence (0xc3 without continuation) embedded in
    // otherwise-ASCII host.
    let bytes = build_client_hello_ascii_bytes(&[b'f', b'o', b'o', 0xc3, b'.', b'c', b'o', b'm']);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&make_flow_key(), Direction::ClientToServer, &bytes, 0);

    let finding = analyzer
        .findings()
        .iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("expected a non-UTF-8 SNI finding");
    assert_eq!(finding.mitre_technique.as_deref(), Some("T1027"));
}
```

Note: `build_client_hello_ascii_bytes`, `make_flow_key`, and `Direction` already exist in the test module from prior work on issue #54. If a helper is missing, follow the existing pattern in the file.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test tls_analyzer_tests ascii_control_sni_finding_sets_mitre_t1027 non_ascii_utf8_sni_finding_sets_mitre_t1027 non_utf8_sni_finding_sets_mitre_t1027`
Expected: FAIL — `mitre_technique` is currently `None` for all three.

- [ ] **Step 3: Update the three finding emissions in `src/analyzer/tls.rs`**

Three sites, around lines 397, 416, 435 — change `mitre_technique: None` to `mitre_technique: Some("T1027".to_string())`. Each site is inside the `SniValue` match arm for `AsciiWithControl`, `NonAsciiUtf8`, `NonUtf8`. Leave the other four `None` sites (weak-cipher, SSL-deprecation x2, server-weak-cipher) unchanged — they're informational crypto-strength findings, not protocol-field tampering.

Replace the `AsciiWithControl` arm's `mitre_technique: None,` with:

```rust
                        mitre_technique: Some("T1027".to_string()),
```

Replace the `NonAsciiUtf8` arm's `mitre_technique: None,` with:

```rust
                        mitre_technique: Some("T1027".to_string()),
```

Replace the `NonUtf8` arm's `mitre_technique: None,` with:

```rust
                        mitre_technique: Some("T1027".to_string()),
```

- [ ] **Step 4: Run the failing tests — now expecting PASS**

Run: `cargo test --test tls_analyzer_tests`
Expected: all new tests PASS; no previously-passing tests broken.

- [ ] **Step 5: Confirm coverage regression test still passes**

Run: `cargo test --test mitre_tests every_emitted_technique_id_is_known`
Expected: PASS. (T1027 was already in the canonical list.)

- [ ] **Step 6: Commit**

```bash
git add src/analyzer/tls.rs tests/tls_analyzer_tests.rs
git commit -m "feat(tls): map malformed SNI findings to MITRE T1027 (Obfuscated Files or Information)"
```

---

### Task 5: Add the `--mitre` flag to `Commands::Analyze`

**Files:**
- Modify: `src/cli.rs`
- Test: `tests/cli_tests.rs`

- [ ] **Step 1: Write the failing CLI parse test**

Append to `tests/cli_tests.rs`:

```rust
#[test]
fn test_mitre_flag_parses_on_analyze() {
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap", "--mitre"]);
    match cli.command {
        Commands::Analyze { mitre, .. } => assert!(mitre),
        _ => panic!("Expected Analyze command"),
    }
}

#[test]
fn test_mitre_flag_defaults_false() {
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap"]);
    match cli.command {
        Commands::Analyze { mitre, .. } => assert!(!mitre),
        _ => panic!("Expected Analyze command"),
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test cli_tests test_mitre_flag`
Expected: compile error — `Commands::Analyze` has no field `mitre`.

- [ ] **Step 3: Add the field to `Commands::Analyze` in `src/cli.rs`**

Insert the flag inside the `Analyze` variant, between `beacon` and `all` (matches ordering convention of grouped analyzer flags):

```rust
        /// Detect C2 beaconing patterns
        #[arg(long)]
        beacon: bool,

        /// Group findings by MITRE ATT&CK tactic and show technique names
        #[arg(long)]
        mitre: bool,

        /// Run all analyzers
        #[arg(short, long)]
        all: bool,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test cli_tests`
Expected: all tests PASS, including the two new ones.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs tests/cli_tests.rs
git commit -m "feat(cli): add --mitre flag to analyze subcommand"
```

---

### Task 6: Add `show_mitre_grouping` field to `TerminalReporter` (default-false; no behavior change yet)

**Files:**
- Modify: `src/reporter/terminal.rs`
- Modify: `src/main.rs`

The goal of this task is to extend the reporter's shape without yet changing behavior, so the entire existing test suite keeps passing. The rendering logic lands in Task 8.

- [ ] **Step 1: Modify `TerminalReporter` in `src/reporter/terminal.rs`**

Change the struct definition:

```rust
pub struct TerminalReporter {
    pub use_color: bool,
    /// When true, regroup the FINDINGS section by MITRE tactic and expand
    /// the per-finding MITRE line to include the technique name.
    pub show_mitre_grouping: bool,
}
```

- [ ] **Step 2: Update the two call sites in `src/main.rs`**

Both at lines 175 and 218 — pass `show_mitre_grouping: false` for now. The analyze path will be re-wired in Task 7; the summary path keeps `false` permanently (summary never renders findings).

`src/main.rs:175` — inside `run_analyze`:

```rust
            let reporter = TerminalReporter { use_color, show_mitre_grouping: false };
```

`src/main.rs:218` — inside `run_summary`:

```rust
            let reporter = TerminalReporter { use_color, show_mitre_grouping: false };
```

- [ ] **Step 3: Verify nothing broke**

Run: `cargo test`
Expected: all existing tests still PASS.

- [ ] **Step 4: Commit**

```bash
git add src/reporter/terminal.rs src/main.rs
git commit -m "refactor(reporter): add show_mitre_grouping field (default false, no behavior change)"
```

---

### Task 7: Thread `--mitre` through `run_analyze` into the reporter

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Destructure `mitre` in the `Commands::Analyze` arm**

Update `src/main.rs:28-44`:

```rust
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
```

- [ ] **Step 2: Update `run_analyze`'s signature and reporter construction**

Update the function signature (around line 53) and the reporter construction (around line 175):

```rust
fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    enable_http: bool,
    enable_tls: bool,
    show_mitre_grouping: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
```

And at the terminal-reporter construction site inside `run_analyze`:

```rust
        _ => {
            let reporter = TerminalReporter { use_color, show_mitre_grouping };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
```

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: clean build; all tests still pass (no rendering change yet; flag currently has no effect when set).

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): thread --mitre flag into TerminalReporter"
```

---

### Task 8: Implement grouped rendering (sort, unknown handling, name expansion)

**Files:**
- Modify: `src/reporter/terminal.rs`
- Test: `tests/reporter_tests.rs`

This is the biggest task. It covers four coupled behaviors that live in one render path: tactic grouping, within-group sort order, unknown/None bucketing, and per-finding MITRE line name expansion.

- [ ] **Step 1: Add failing tests to `tests/reporter_tests.rs`**

Check the top of `tests/reporter_tests.rs` for existing imports and helper fns. You will need `Finding`, `Verdict`, `Confidence`, `ThreatCategory`, and `TerminalReporter`. Add the following tests:

```rust
use wirerust::mitre::MitreTactic;

fn base_finding_with_mitre(
    technique: Option<&str>,
    verdict: Verdict,
    confidence: Confidence,
    summary: &str,
) -> Finding {
    Finding {
        category: ThreatCategory::Anomaly,
        verdict,
        confidence,
        summary: summary.to_string(),
        evidence: vec![],
        mitre_technique: technique.map(|s| s.to_string()),
        source_ip: None,
        timestamp: None,
    }
}

#[test]
fn mitre_grouping_emits_tactic_headers_in_canonical_order() {
    let findings = vec![
        // Impact — should appear before ICS but after earlier tactics.
        base_finding_with_mitre(Some("T1499.002"), Verdict::Likely, Confidence::High, "dos"),
        // Discovery — earlier in kill-chain than Impact.
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "scan"),
        // ICS Impair — last Enterprise/ICS tactic rendered.
        base_finding_with_mitre(Some("T0855"), Verdict::Likely, Confidence::High, "ics"),
    ];

    let reporter = TerminalReporter { use_color: false, show_mitre_grouping: true };
    let out = reporter.render(&empty_summary(), &findings, &[]);

    let discovery_pos = out.find("Discovery").expect("missing Discovery header");
    let impact_pos = out.find("Impact").expect("missing Impact header");
    let ics_pos = out.find("Impair Process Control").expect("missing ICS header");
    assert!(discovery_pos < impact_pos, "Discovery must come before Impact");
    assert!(impact_pos < ics_pos, "Impact must come before ICS tactics");
}

#[test]
fn mitre_grouping_sorts_within_tactic_by_verdict_then_confidence() {
    let findings = vec![
        base_finding_with_mitre(Some("T1046"), Verdict::Unlikely, Confidence::High, "third"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::Medium, "second"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "first"),
        base_finding_with_mitre(Some("T1046"), Verdict::Inconclusive, Confidence::Low, "fourth_ish"),
    ];
    let reporter = TerminalReporter { use_color: false, show_mitre_grouping: true };
    let out = reporter.render(&empty_summary(), &findings, &[]);

    let p1 = out.find("first").expect("first missing");
    let p2 = out.find("second").expect("second missing");
    let p3 = out.find("fourth_ish").expect("fourth_ish missing");
    let p4 = out.find("third").expect("third missing");
    assert!(p1 < p2 && p2 < p3 && p3 < p4, "verdict/confidence sort wrong: {out}");
}

#[test]
fn mitre_grouping_buckets_none_and_unknown_under_uncategorized() {
    let findings = vec![
        base_finding_with_mitre(None, Verdict::Likely, Confidence::High, "no_id_finding"),
        base_finding_with_mitre(Some("T9999"), Verdict::Likely, Confidence::High, "unknown_id_finding"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "known_finding"),
    ];
    let reporter = TerminalReporter { use_color: false, show_mitre_grouping: true };
    let out = reporter.render(&empty_summary(), &findings, &[]);

    let uncat_pos = out.find("Uncategorized").expect("missing Uncategorized section");
    let no_id_pos = out.find("no_id_finding").expect("missing no-id finding");
    let unknown_pos = out.find("unknown_id_finding").expect("missing unknown-id finding");
    let known_pos = out.find("known_finding").expect("missing known finding");

    assert!(known_pos < uncat_pos, "Uncategorized must come after known tactics");
    assert!(uncat_pos < no_id_pos && uncat_pos < unknown_pos);
    assert!(out.contains("T9999 (unknown)"), "unknown ID must render with '(unknown)' label");
}

#[test]
fn mitre_grouping_expands_per_finding_line_with_technique_name() {
    let findings = vec![
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "scan"),
    ];
    let reporter = TerminalReporter { use_color: false, show_mitre_grouping: true };
    let out = reporter.render(&empty_summary(), &findings, &[]);
    assert!(
        out.contains("MITRE: T1046 — Network Service Discovery"),
        "expected em-dash-expanded MITRE line, got: {out}",
    );
}

#[test]
fn default_rendering_unchanged_when_mitre_flag_off() {
    let findings = vec![
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "scan"),
    ];
    let reporter = TerminalReporter { use_color: false, show_mitre_grouping: false };
    let out = reporter.render(&empty_summary(), &findings, &[]);
    // No tactic headers; no em-dash expansion; plain "MITRE: T1046" line.
    assert!(out.contains("MITRE: T1046"));
    assert!(!out.contains("—"), "em-dash should not appear in default render");
    assert!(!out.contains("Uncategorized"));
}
```

If `empty_summary()` is not already a helper in `reporter_tests.rs`, it is defined in that file from prior tests — confirm it exists or use the existing pattern to construct a `Summary::new()` directly.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test reporter_tests mitre_grouping default_rendering_unchanged`
Expected: FAIL — no grouping implemented yet; default-rendering-unchanged may pass depending on the exact assertion, but the new ones will all fail.

- [ ] **Step 3: Implement the grouped render path in `src/reporter/terminal.rs`**

Modify the `Reporter::render` impl for `TerminalReporter`. Replace the single `if !findings.is_empty() { ... }` FINDINGS block with a branch on `self.show_mitre_grouping`.

Add the following imports at the top of `src/reporter/terminal.rs`:

```rust
use crate::mitre::{MitreTactic, all_tactics_in_report_order, technique_name, technique_tactic};
```

Replace the existing findings block. The existing block lives around lines 98–132; keep it as the `!show_mitre_grouping` branch. Insert the grouped branch alongside:

```rust
        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            if self.show_mitre_grouping {
                self.render_findings_grouped(&mut out, findings);
            } else {
                for f in findings {
                    self.render_finding_flat(&mut out, f);
                }
            }
            out.push('\n');
        }
```

Refactor the per-finding rendering logic — the existing body of the inner `for f in findings { ... }` — into a new helper `render_finding_flat`:

```rust
impl TerminalReporter {
    fn render_finding_flat(&self, out: &mut String, f: &Finding) {
        let escaped_summary = escape_for_terminal(&f.summary);
        let line = format!(
            "[{}] {} ({}) - {}",
            f.category, f.verdict, f.confidence, escaped_summary
        );
        let colored = if self.use_color {
            match f.verdict {
                Verdict::Likely => match f.confidence {
                    Confidence::High => line.red().bold().to_string(),
                    _ => line.yellow().to_string(),
                },
                Verdict::Inconclusive => line.cyan().to_string(),
                Verdict::Unlikely => line.dimmed().to_string(),
            }
        } else {
            line
        };
        out.push_str(&format!("  {colored}\n"));
        for ev in &f.evidence {
            let escaped_ev = escape_for_terminal(ev);
            out.push_str(&format!("    > {escaped_ev}\n"));
        }
        if let Some(ref t) = f.mitre_technique {
            out.push_str(&format!("    MITRE: {t}\n"));
        }
    }

    fn render_finding_grouped(&self, out: &mut String, f: &Finding) {
        // Same body as render_finding_flat, but the MITRE line expands to
        // include the technique name when resolvable.
        let escaped_summary = escape_for_terminal(&f.summary);
        let line = format!(
            "[{}] {} ({}) - {}",
            f.category, f.verdict, f.confidence, escaped_summary
        );
        let colored = if self.use_color {
            match f.verdict {
                Verdict::Likely => match f.confidence {
                    Confidence::High => line.red().bold().to_string(),
                    _ => line.yellow().to_string(),
                },
                Verdict::Inconclusive => line.cyan().to_string(),
                Verdict::Unlikely => line.dimmed().to_string(),
            }
        } else {
            line
        };
        out.push_str(&format!("  {colored}\n"));
        for ev in &f.evidence {
            let escaped_ev = escape_for_terminal(ev);
            out.push_str(&format!("    > {escaped_ev}\n"));
        }
        if let Some(ref id) = f.mitre_technique {
            debug_assert!(
                technique_name(id).is_some() || cfg!(not(debug_assertions)),
                "MITRE technique id {id} is not in the lookup — update src/mitre.rs and tests/mitre_tests.rs",
            );
            match technique_name(id) {
                Some(name) => out.push_str(&format!("    MITRE: {id} — {name}\n")),
                None => out.push_str(&format!("    MITRE: {id} (unknown)\n")),
            }
        }
    }

    fn render_findings_grouped(&self, out: &mut String, findings: &[Finding]) {
        // Bucket findings by tactic. Preserve emission order as tertiary
        // tie-breaker by attaching the original index.
        let mut buckets: std::collections::HashMap<Option<MitreTactic>, Vec<(usize, &Finding)>> =
            std::collections::HashMap::new();
        for (i, f) in findings.iter().enumerate() {
            let tactic = f
                .mitre_technique
                .as_deref()
                .and_then(technique_tactic);
            buckets.entry(tactic).or_default().push((i, f));
        }

        // Severity-desc sort within each bucket: Likely > Inconclusive >
        // Unlikely, High > Medium > Low, then emission order.
        fn verdict_rank(v: Verdict) -> u8 {
            match v {
                Verdict::Likely => 0,
                Verdict::Inconclusive => 1,
                Verdict::Unlikely => 2,
            }
        }
        fn confidence_rank(c: Confidence) -> u8 {
            match c {
                Confidence::High => 0,
                Confidence::Medium => 1,
                Confidence::Low => 2,
            }
        }
        for (_, items) in buckets.iter_mut() {
            items.sort_by_key(|(idx, f)| (verdict_rank(f.verdict), confidence_rank(f.confidence), *idx));
        }

        // Emit known tactics in canonical kill-chain order, then the
        // Uncategorized bucket (None key) last.
        for tactic in all_tactics_in_report_order() {
            if let Some(items) = buckets.get(&Some(*tactic)) {
                out.push_str(&format!("  ## {tactic}\n"));
                for (_, f) in items {
                    self.render_finding_grouped(out, f);
                }
            }
        }
        if let Some(items) = buckets.get(&None) {
            out.push_str("  ## Uncategorized\n");
            for (_, f) in items {
                self.render_finding_grouped(out, f);
            }
        }
    }
}
```

The `debug_assert!` is structured so that in release (`debug_assertions` off) the assertion short-circuits to `true` and is eliminated by the compiler — catching typos in `cargo test` runs but never panicking for users.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test reporter_tests`
Expected: all new tests PASS; all existing reporter tests still PASS.

- [ ] **Step 5: Full local test sweep**

Run: `cargo test`
Expected: all tests in the workspace PASS.

- [ ] **Step 6: Commit**

```bash
git add src/reporter/terminal.rs tests/reporter_tests.rs
git commit -m "feat(reporter): group findings by MITRE tactic when --mitre is set"
```

---

### Task 9: Full CI-equivalent local check + final commit

**Files:** (fmt fixes across any files touched)

- [ ] **Step 1: Run rustfmt check**

Run: `cargo fmt --all -- --check`
Expected: no output (all files formatted).

If it reports unformatted files: run `cargo fmt --all`, inspect the diff, and stage the changes.

- [ ] **Step 2: Run clippy with all targets + warnings-as-errors**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

If warnings appear, fix them at the root cause. Do NOT use `#[allow(...)]` unless the pattern is already present elsewhere in the file for the same lint.

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: 100% PASS.

- [ ] **Step 4: Commit fmt/clippy fixes (if any)**

```bash
git add -u
git commit -m "style: apply rustfmt"
```

Only if Step 1 or 2 produced changes. If not, skip.

- [ ] **Step 5: Confirm the branch is clean and ready for PR review**

Run: `git status && git log --oneline origin/develop..HEAD`
Expected: clean working tree; commit log shows the task-per-commit sequence.

The feature branch is now ready for local PR review (`/pr-review-toolkit:review-pr`) followed by the iterate-until-clean loop described in the validated-feature-lifecycle skill.

---

## Self-Review Checklist (already applied to this plan)

- Spec coverage: every section in the spec (mitre module, Option<String> data model, terminal reporter grouping, --mitre flag, error handling, pre-seeded techniques, TLS T1027 assignment, testing strategy) maps to at least one task.
- Placeholder scan: every code step contains actual code. No "TBD" or "similar to above."
- Type consistency: `show_mitre_grouping: bool`, `technique_name(id: &str) -> Option<&'static str>`, `technique_tactic(id: &str) -> Option<MitreTactic>`, and `MitreTactic` variant names used identically across all tasks.
- Clarifications the executor should watch for:
  - `src/main.rs` — not `src/dispatcher.rs` — is where `Commands::Analyze` is destructured (the spec's wording was inherited from an earlier draft where a command dispatcher existed).
  - The TLS analyzer has 7 `mitre_technique: None` sites; only 3 (the `SniValue` match arms for `AsciiWithControl`, `NonAsciiUtf8`, `NonUtf8`) get `Some("T1027")`. The other 4 (weak ciphers, SSL deprecation x2, server weak cipher) stay `None` — they're informational crypto-strength findings, not tampering.
