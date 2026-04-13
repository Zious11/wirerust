# MITRE ATT&CK Mapping — Design Spec

**Issue:** [#5](https://github.com/zious11/wirerust/issues/5)
**Date:** 2026-04-13
**Status:** Draft

## Goal

Systematically map every finding to a MITRE ATT&CK technique (Enterprise + ICS matrices) and add a `--mitre` flag that regroups terminal output by tactic. The `Finding` struct already carries an `Option<String>` field named `mitre_technique`; most analyzers emit `None`. This design populates missing analyzers, adds a lookup module, and wires a CLI flag for tactic-grouped output.

## Non-goals

- CSV schema changes (owned by issue #4).
- JSON schema additions for `mitre_tactic` / `mitre_name` — deferred until a SIEM-ingestion consumer asks; handled later via a DTO over `Finding`.
- `--mitre-links` URLs to attack.mitre.org.
- Runtime STIX bundle ingestion or build-time code generation.
- `--group-by=...` orthogonal-flag abstraction (deferred until a second grouping axis exists).
- DNS analyzer technique assignment — DNS currently emits no findings; covered when issue #3 lands.

## Architecture

### New module: `src/mitre.rs`

```rust
pub enum MitreTactic {
    // Enterprise canonical order (MITRE ATT&CK v18, 14 tactics)
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
    // ICS-unique tactics (names that don't collide with Enterprise)
    IcsInhibitResponseFunction,
    IcsImpairProcessControl,
}

impl fmt::Display for MitreTactic {
    // Unprefixed canonical names per MITRE convention (Caldera, Atomic Red
    // Team, ATT&CK Navigator all render tactic names without matrix prefixes):
    //   CommandAndControl                 -> "Command and Control"
    //   DefenseEvasion                    -> "Defense Evasion"
    //   IcsInhibitResponseFunction        -> "Inhibit Response Function"
    //   IcsImpairProcessControl           -> "Impair Process Control"
}

pub fn technique_name(id: &str) -> Option<&'static str>;
pub fn technique_tactic(id: &str) -> Option<MitreTactic>;
pub fn all_tactics_in_report_order() -> &'static [MitreTactic];
```

**Enterprise/ICS tactic name collision — known limitation.** MITRE's Enterprise and ICS matrices share several tactic *names* (Persistence, Discovery, Command and Control, Lateral Movement, Collection, Impact) that have *different* `TA-####` IDs (e.g., Enterprise Discovery = TA0007; ICS Discovery = TA0111). This design unifies them under a single variant (e.g., `Discovery` covers both). Practical effect: an Enterprise T1046 finding and an ICS T0846 finding both render under a single "Discovery" section header. Acceptable for v1 — no consumer has asked for matrix-level distinction; can split into `EnterpriseDiscovery` / `IcsDiscovery` if demand appears. ICS-unique tactics (Inhibit Response Function, Impair Process Control, Evasion) get their own variants.

Both `technique_name` and `technique_tactic` are backed by exhaustive `match` statements. Perplexity-validated as idiomatic for ~15–20 static entries in Rust 2024; `phf` and `Lazy<HashMap>` add cost without benefit at this scale, and clippy does not warn on unused match arms.

### Data model: `mitre_technique` stays `Option<String>`

No change to `Finding`. Rationale (validated):

- Security tools with evolving external catalogs (MITRE ATT&CK, CVE, CAPEC) idiomatically store IDs as strings with validation at the boundary.
- Enum refactor would churn ~30 test fixtures + the JSON schema for marginal safety gain.
- Tactic is derived at render time (`technique_tactic(id)`) — single source of truth, impossible for technique and tactic to disagree.

### Terminal reporter

Without `--mitre` (default): output unchanged. `MITRE: T1046` line printed per finding if set.

With `--mitre`:

1. Replace the flat FINDINGS section with a grouped view.
2. Tactic section order = `all_tactics_in_report_order()` (MITRE Enterprise canonical kill-chain order: Reconnaissance → Resource Development → Initial Access → Execution → Persistence → Privilege Escalation → Defense Evasion → Credential Access → Discovery → Lateral Movement → Collection → Command and Control → Exfiltration → Impact → ICS-unique tactics → Uncategorized last).
3. Within each tactic, sort by **Verdict descending** (`Likely > Inconclusive > Unlikely`) then **Confidence descending** (`High > Medium > Low`) then **emission order**. Validated as the SIEM industry standard (Splunk, Elastic, QRadar, Sentinel, Sumo Logic all default to severity-desc; within-MITRE-tactic groups specifically follow this order).
4. Findings with `mitre_technique == None` OR an unknown ID go to the "Uncategorized" bucket at the end.
5. Per-finding MITRE line expands: `MITRE: T1046 — Network Service Discovery` (ID, em-dash, name).
6. Unknown IDs render as `MITRE: T9999 (unknown)`.

### CLI

Add to `Commands::Analyze` in `src/cli.rs`:

```rust
/// Group findings by MITRE ATT&CK tactic and show technique names
#[arg(long)]
pub mitre: bool,
```

Threaded through `src/dispatcher.rs` into `TerminalReporter` via a new constructor parameter (or field on an existing config struct, following the `use_color` pattern).

### Error handling for unknown IDs

- `technique_name` / `technique_tactic` return `Option`; `None` is the unknown-ID signal.
- At the reporter call site: `debug_assert!(technique_name(id).is_some(), "unknown MITRE id: {id}")`. Fires in `cargo test` (debug build), zero cost in release. Catches analyzer typos at CI time.
- Release behavior: render unknown IDs inline (`MITRE: T9999 (unknown)`) and bucket under Uncategorized. Never panic user-facing.
- Regression test `tests/mitre_coverage.rs`: a canonical list of every ID the codebase intentionally emits, with each asserted to resolve via `technique_name` + `technique_tactic`. The list is manually maintained; growing it is a required step when any analyzer adds a new technique ID.

## Pre-seeded techniques

Scope includes entries for currently-emitted IDs **plus** near-future IDs expected from backlog issues #3, #7, #8. Pre-seeding known-upstream catalog entries is not a YAGNI violation (Perplexity-validated); adding a match arm has zero maintenance cost and clippy does not warn on unused arms.

| ID | Name | Tactic | Status |
|---|---|---|---|
| T1027 | Obfuscated Files or Information | Defense Evasion | **new in this PR** (TLS) |
| T1036 | Masquerading | Defense Evasion | existing (reassembly) |
| T1040 | Network Sniffing | Credential Access | pre-seed (#3) |
| T1046 | Network Service Discovery | Discovery | existing (HTTP) |
| T1071 | Application Layer Protocol | Command and Control | pre-seed (#3) |
| T1071.001 | Web Protocols | Command and Control | pre-seed (#3) |
| T1071.004 | DNS | Command and Control | pre-seed (#3) |
| T1083 | File and Directory Discovery | Discovery | existing (HTTP) |
| T1499.002 | Service Exhaustion Flood | Impact | existing (HTTP) |
| T1505.003 | Web Shell | Persistence | existing (HTTP) |
| T1573 | Encrypted Channel | Command and Control | pre-seed (#3) |
| T0846 | Remote System Discovery | Discovery | pre-seed (#7/#8) |
| T0855 | Unauthorized Command Message | ICS Impair Process Control | pre-seed (#7 Modbus) |
| T0856 | Spoof Reporting Message | ICS Impair Process Control | pre-seed (#8 DNP3) |
| T0885 | Commonly Used Port | Command and Control | pre-seed (#7/#8) — verified not deprecated |

All Enterprise mappings verified against current MITRE ATT&CK (no revisions in 2024-2025). T0885 explicitly verified not deprecated (Enterprise's equivalent T1043 was deprecated in 2020; ICS retained T0885 separately, and a 2025 MITRE detection strategy DET0736 was added for it).

## TLS analyzer MITRE assignments

| Finding | Technique | Rationale |
|---|---|---|
| SNI contains ASCII control characters | **T1027** | Obfuscation via protocol field tampering. |
| SNI is ASCII but non-UTF-8 | **T1027** | Same. |
| SNI is valid UTF-8 but non-ASCII (control-char-free) | **T1027** | Same — RFC 6066 §3 requires ASCII. |
| Empty SNI | None (informational) | Not inherently malicious; benign scanners produce it. |
| SNI contains IP literal | None (informational) | Defer until correlated with C2 behavior. |
| Punycode / IDN SNI | None | IDN homograph detection is future work. |

T1027 over T1036 (Masquerading) is deliberate. Perplexity-validated: T1036 "requires an attacker-controlled element attempting to impersonate a legitimate one, not direct tampering with protocol payloads." SNI with control bytes does not impersonate — it corrupts. Reassembly's existing T1036 usage (segment overlap with differing replacement content) is correct for masquerading; SNI tampering is correctly T1027.

T1027 over T1071.001 is also deliberate. T1071.001 would overstate our detection — we see a malformed protocol field, not evidence of active C2 over HTTPS. Keep the technique aligned with what we actually detect.

## Testing strategy

- **Unit (`src/mitre.rs`)**: every seeded ID round-trips through `technique_name` and `technique_tactic`; `all_tactics_in_report_order` contains every enum variant exactly once; `MitreTactic::Display` matches expected human names.
- **Regression (`tests/mitre_coverage.rs`)**: canonical list of every ID the codebase emits, each asserted to resolve. Fails CI if an analyzer emits an ID not in the lookup.
- **Reporter (`tests/reporter_tests.rs`)**: with `show_mitre_grouping = true`, findings are grouped by tactic; within-group sort is verdict-desc → confidence-desc; unknown IDs render as `(unknown)` and bucket under Uncategorized; `None` techniques bucket under Uncategorized; name expansion includes the em-dash.
- **CLI integration (`tests/integration_tests.rs` or equivalent)**: `wirerust analyze --mitre FIXTURE.pcap` produces grouped output; `wirerust analyze FIXTURE.pcap` matches baseline (no MITRE grouping).
- **TLS analyzer (`tests/tls_analyzer_tests.rs`)**: three malformed-SNI cases now assert `mitre_technique == Some("T1027")`.

## Blast radius

**New files:**
- `src/mitre.rs` (~200 lines)
- `tests/mitre_coverage.rs`
- `tests/mitre_tests.rs`

**Modified files:**
- `src/lib.rs` — add `pub mod mitre;`
- `src/cli.rs` — add `mitre: bool` flag to `Commands::Analyze`
- `src/dispatcher.rs` — thread flag through to reporter
- `src/reporter/mod.rs` — constructor takes the flag (or a config struct)
- `src/reporter/terminal.rs` — grouping code path; `debug_assert!` at MITRE render site
- `src/analyzer/tls.rs` — 3 of 7 `mitre_technique: None` sites become `Some("T1027")`
- `tests/reporter_tests.rs` — add grouping-path tests
- `tests/tls_analyzer_tests.rs` — assert T1027 on the three findings

## Out of scope / follow-up issues (if demand appears)

- JSON schema DTO with computed `mitre_tactic`/`mitre_name` — file when a consumer asks.
- `--mitre-links` (attack.mitre.org URLs per finding).
- `--group-by=tactic|severity|...` orthogonal flag refactor.
- DNS technique mapping (waits on #3 beaconing).
- Build-time STIX bundle codegen for the match statements — unnecessary at current scale.

## Validation trail

Every substantive design decision in this spec was validated against Perplexity (and Context7 where a library was involved) before being written here:

- **Data model** (`Option<String>` vs typed enum): Perplexity recommends strings for security tooling with evolving external catalogs.
- **Tactic as derived vs stored**: Perplexity recommends derived / DTO pattern; STIX 2.1 uses normalized relationships (derived), Suricata EVE denormalizes. We pick derived; can add a JSON DTO later.
- **`--mitre` flag scope**: Perplexity leans orthogonal flags; we defer that abstraction (YAGNI — single grouping axis today) and document the intent.
- **TLS technique**: T1027 validated over T1036 and T1071.001.
- **Pre-seeding**: Perplexity says pre-populating known-upstream catalogs is not a YAGNI violation; clippy does not warn on unused match arms.
- **Error handling**: `debug_assert!` in reporter + `Option` return from lookup is the Perplexity-recommended pattern for internal-invariant static tables.
- **Grouped output layout**: replace flat (our case) matches opt-in semantics of `--mitre`.
- **Within-group sort order**: severity descending matches every major SIEM (Splunk, Elastic, QRadar, Sentinel, Sumo Logic).
- **MITRE tactic assignments**: all Enterprise techniques verified current as of 2024-2025; ICS T0885 verified not deprecated; ICS tactic names verified.
- **Enterprise canonical tactic ordering**: 14 tactics in kill-chain order (Reconnaissance → Resource Development → Initial Access → Execution → Persistence → Privilege Escalation → Defense Evasion → Credential Access → Discovery → Lateral Movement → Collection → Command and Control → Exfiltration → Impact) validated against MITRE ATT&CK v18.
- **Display convention**: unprefixed tactic names per MITRE convention (Caldera, Atomic Red Team, ATT&CK Navigator); disambiguation via tactic IDs is the standard, not name prefixes. Enterprise/ICS tactic name collision (e.g., Discovery exists in both matrices with different TA-IDs) treated as a v1 limitation documented inline.
