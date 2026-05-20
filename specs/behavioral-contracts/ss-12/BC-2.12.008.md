---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.008: --all Enables dns/http/tls Together

## Description

In `run_analyze`, the boolean flags `enable_dns`, `enable_http`, and `enable_tls` are
computed as `*dns || *all`, `*http || *all`, and `*tls || *all` respectively (main.rs:57-58).
When `--all` is given, all three analyzers are enabled regardless of whether `--dns`, `--http`,
and `--tls` were individually specified. This is an OR-semantics enablement at the call site,
not in the clap `Commands::Analyze` struct itself.

## Preconditions

1. `run_analyze` is entered with a `Commands::Analyze` that has `all = true`.

## Postconditions

1. `enable_dns = true`, `enable_http = true`, `enable_tls = true` in `run_analyze`.
2. All three analyzers are constructed and attached to the dispatcher.
3. `dns_analyzer`, `http_analyzer`, `tls_analyzer` are all active for the packet loop.

## Invariants

1. The OR computation is at main.rs:57-58 inside the `Commands::Analyze` arm.
2. The `all` field in the clap struct is a plain `bool`; the OR expansion happens in
   main.rs, not in cli.rs.
3. The `--mitre` flag is NOT included in `--all`; it must be specified separately.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --all without --dns/--http/--tls | All three enabled |
| EC-002 | --dns --all | Equivalent to --all (OR semantics; dns=true regardless) |
| EC-003 | --all without --mitre | mitre=false; findings not grouped |
| EC-004 | Neither --all nor individual flags | No analyzers enabled |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Parsed: all=true | enable_dns=true, enable_http=true, enable_tls=true | happy-path |
| Parsed: all=false, dns=true | enable_dns=true, others per individual flags | happy-path |
| Parsed: all=false, all flags false | All analyzers disabled | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | --all enables dns/http/tls | unit: assert enable_* computed from all || individual (MEDIUM -- not directly tested; code is explicit) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- the --all flag is the shorthand CLI path for enabling the full analysis pipeline |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-008 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test passes --all and asserts all three analyzers activate) |

## Related BCs

- BC-2.12.001 -- depends on (--all is defined in the analyze subcommand)
- BC-2.12.009 -- composes with (needs_reassembly is true when http or tls is enabled)

## Architecture Anchors

- `src/main.rs:54-63` -- Commands::Analyze destructure with *dns || *all expansion

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:54-63` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code is explicit at main.rs:57: `*dns || *all`
- **inferred**: no direct test passes `--all` and asserts all three analyzers activate

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (flag evaluation is pure) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell (run_analyze creates analyzers and processes files) |

#### Refactoring Notes

To upgrade to HIGH: add `Cli::try_parse_from(["wirerust", "--all", "analyze", "x.pcap"])` and
assert that the resulting `Commands::Analyze { all: true, .. }` combined with the OR expansion
at main.rs:57 produces `enable_dns=true, enable_http=true, enable_tls=true`.
