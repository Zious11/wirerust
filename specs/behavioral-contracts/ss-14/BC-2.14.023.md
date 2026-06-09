---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.023: --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly

## Description

The `analyze` subcommand exposes a `--modbus` boolean flag (default false / off). When
`--modbus` is present, `ModbusAnalyzer` is constructed and wired into the `StreamDispatcher`.
When `--all` is present, its expansion includes `--modbus` (boolean OR semantics identical to
the existing `--http` / `--tls` / `--dns` expansion). Modbus analysis requires stream
reassembly because Modbus TCP ADUs may span multiple TCP segments; if reassembly is disabled
(via `--no-reassemble`), `ModbusAnalyzer` is NOT constructed even if `--modbus` is present,
and a warning is emitted to stderr.

## Preconditions

1. User invokes `wirerust analyze <target> [flags]`.
2. The analyze subcommand is defined in `src/cli.rs` using clap's `#[arg(long)]` derive.
3. The `--modbus` flag is declared as `modbus: bool` (default false — clap's default for `bool`).
4. The `--all` flag is declared as `all: bool` (already exists; its expansion logic is in
   `main.rs::run_analyze`).

## Postconditions

### P1: `--modbus` absent, `--all` absent
1. `enable_modbus = false`.
2. `modbus_analyzer = None`.
3. `StreamDispatcher::new` receives `modbus: None`.
4. No Modbus PDUs are analyzed; port-502 flows receive `DispatchTarget::None`.

### P2: `--modbus` present (without `--all`)
1. `enable_modbus = true`.
2. If `skip_reassembly == false` (reassembly is on):
   - `ModbusAnalyzer::new(modbus_write_threshold)` is constructed.
   - `modbus_analyzer = Some(modbus_analyzer)`.
   - `StreamDispatcher::new` receives `modbus: Some(modbus_analyzer)`.
3. If `skip_reassembly == true` (reassembly is off via `--no-reassemble`):
   - A warning is emitted to stderr: `"WARNING: --modbus requires stream reassembly; ignoring --modbus (pass --reassemble or omit --no-reassemble)"`.
   - `modbus_analyzer = None`.
   - `StreamDispatcher::new` receives `modbus: None`.
   - No Modbus analysis occurs silently — the warning is the notification.

### P3: `--all` present (without explicit `--modbus`)
1. `enable_modbus = *modbus || *all` evaluates to `true`.
2. All conditions from P2 apply identically (same `skip_reassembly` guard).

### P4: `needs_reassembly` expansion
1. `needs_reassembly = enable_http || enable_tls || enable_modbus`.
2. When `--modbus` is present and `--no-reassemble` is absent, `needs_reassembly == true`,
   and the reassembly engine is initialized.
3. `--modbus` alone (without `--http` or `--tls`) is sufficient to trigger reassembly.
   This prevents the "silent analysis" regression where Modbus data arrives but is never
   delivered to `ModbusAnalyzer` because the reassembler was not started.

### P5: Post-finalize collection
1. After the packet loop and `reassembler.finalize()`, the dispatcher's Modbus analyzer is
   collected via `dispatcher.take_modbus_analyzer()`.
2. If `Some(modbus)` is returned, `modbus.findings()` are appended to `all_findings` and
   `modbus.summarize()` is pushed to `analyzer_summaries`.
3. If `None` is returned (Modbus was disabled), no findings or summaries are added.

## Invariants

1. **`--modbus` is default-off**: `modbus: bool` with no `default_value_t` annotation in clap
   defaults to `false`. A bare invocation without `--modbus` or `--all` produces no Modbus
   analysis.
2. **`--all` semantics are boolean OR**: `enable_modbus = *modbus || *all`. This is identical
   to the pattern used for `--http`, `--tls`, and `--dns` in the existing codebase.
3. **Modbus requires reassembly**: `ModbusAnalyzer` is constructed only when
   `enable_modbus && !skip_reassembly`. This is a hard invariant — Modbus TCP is a stream
   protocol; without reassembly, individual TCP segments may contain partial MBAP headers and
   the analyzer would produce incorrect results.
4. **`needs_reassembly` includes Modbus**: the expression `enable_http || enable_tls || enable_modbus`
   must include `enable_modbus` so that `--modbus` alone activates the reassembly engine. Omitting
   this term is the "silent analysis" regression risk flagged in F1 delta analysis §4.
5. **Warning on skip**: when `enable_modbus && skip_reassembly`, a warning is printed and the
   analyzer is silently omitted. No error, no exit-code change — consistent with how other
   analyzers handle the `--no-reassemble` case (they are simply not constructed).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--modbus` and `--no-reassemble` together | Warning printed to stderr; `modbus_analyzer = None`; no Modbus findings; exit 0 |
| EC-002 | `--all` and `--no-reassemble` together | Same as EC-001 plus HTTP and TLS are also omitted (all analyzers that require reassembly are skipped); warning printed per-analyzer |
| EC-003 | `--modbus` alone (no `--http`, `--tls`, `--dns`) | `needs_reassembly = true`; reassembly engine starts; Modbus analyzer constructed; port-502 flows analyzed |
| EC-004 | `--modbus` with `--reassemble` explicitly | Equivalent to EC-003; `skip_reassembly = false`; analyzer constructed |
| EC-005 | `--modbus` and `--http` together | Both analyzers constructed; `needs_reassembly = true` (already true from `enable_http`); both receive their respective flows |
| EC-006 | Neither `--modbus` nor `--all` | `enable_modbus = false`; `modbus_analyzer = None`; port-502 flows receive `DispatchTarget::None` |
| EC-007 | `--all` with `--no-reassemble` | All reassembly-dependent analyzers (http, tls, modbus) produce `None`; only DNS (packet-level) may still produce summaries if `--dns` / `--all` enables it |
| EC-008 | Empty pcap with `--modbus` | Analyzer constructed; zero PDUs processed; `summarize()` returns zeroed stats; `findings()` returns empty vec; exit 0 |

## Canonical Test Vectors

| Setup | Expected Behavior | Category |
|-------|------------------|----------|
| `wirerust analyze test.pcap --modbus` | `ModbusAnalyzer::new(10)` constructed (default threshold); findings collected post-finalize | happy-path: flag present |
| `wirerust analyze test.pcap` (no flags) | `modbus_analyzer = None`; no Modbus section in output | happy-path: default off |
| `wirerust analyze test.pcap --all` | `enable_modbus = true`; analyzer constructed with default threshold | happy-path: `--all` expansion |
| `wirerust analyze test.pcap --modbus --no-reassemble` | Warning to stderr; `modbus_analyzer = None`; output contains no Modbus section; exit 0 | edge-case: skip-reassembly guard |
| `wirerust analyze test.pcap --modbus` with port-502 traffic | Port-502 flows routed to ModbusAnalyzer; findings emitted | integration: dispatch + analysis |
| `wirerust analyze test.pcap --modbus` with no port-502 traffic | `pdu_count = 0`; `write_count = 0`; zero findings; non-502 flows unaffected | integration: no Modbus traffic |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | (extended) `classify_oracle` returns `DispatchTarget::Modbus` for port-502 flows not matching content rules; `verify_content_first_precedence_exhaustive` covers port-502 branch | Kani: extended oracle test per architecture-delta.md §3.6 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the CLI enablement contract for the ICS analysis capability, specifying exactly how the operator activates Modbus TCP protocol analysis and the conditions under which it is silently omitted |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — indirectly: Modbus dispatch only fires when reassembly is active, preserving the content-first ordering for all other analyzers) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22); SS-12 (cli.rs, main.rs); SS-05 (dispatcher.rs) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | N/A (CLI wiring contract, not a detection) |

## Related BCs

- BC-2.14.024 — depends on (this BC enables the analyzer; BC-2.14.024 configures its threshold parameter)
- BC-2.14.025 — depends on (this BC enables the analyzer; BC-2.14.025 defines how the dispatcher routes to it)
- BC-2.12.008 — composes with (`--all` expansion pattern: this BC adds `enable_modbus` to the existing `enable_http || enable_tls || enable_dns` pattern)
- BC-2.12.009 — composes with (`needs_reassembly` logic: this BC adds `|| enable_modbus` to the existing expression)
- BC-2.05.008 — composes with (dispatcher early-exit guard: now also checks `self.modbus.is_none()`)

## Architecture Anchors

- `src/cli.rs` — `modbus: bool` field in `Commands::Analyze` variant (new `#[arg(long)]`)
- `src/main.rs` — `let enable_modbus = *modbus || *all;`
- `src/main.rs` — `let needs_reassembly = enable_http || enable_tls || enable_modbus;`
- `src/main.rs` — `let modbus_analyzer = if enable_modbus && !skip_reassembly { Some(ModbusAnalyzer::new(modbus_write_threshold)) } else { None };`
- `src/main.rs` — `if let Some(modbus) = dispatcher.take_modbus_analyzer() { ... }`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §5` — CLI integration four-step wiring pattern

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-004 — (extended) `classify_oracle` must cover port-502 → `DispatchTarget::Modbus` per architecture-delta.md §3.6

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §5.1 (CLI flags); architecture-delta.md §5.2 (four-step wiring pattern, `skip_reassembly` note: "Omitting step 2 is the 'silent analysis' regression risk") |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | stderr warning (side effect: `--modbus` + `--no-reassemble` case only) |
| **Deterministic** | yes — same flags always produce same analyzer configuration |
| **Overall classification** | effectful shell (flag parsing mutates clap state; analyzer construction is a one-shot side effect) |
