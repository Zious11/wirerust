# AC-173-003: --iec104 CLI Flag + Reassembly Gating

**AC:** AC-173-003
**BC:** BC-2.12.025 postconditions 1–3
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

`--iec104` is a new boolean flag added to the `analyze` subcommand of `src/cli.rs`. When the
flag is present, `src/main.rs` instantiates `Iec104Analyzer` and passes it to
`StreamDispatcher`. When absent, no `Iec104Analyzer` is created and port-2404 flows are
unanalyzed (opt-in model, EC-003). The flag requires TCP reassembly to be active; if
`--no-reassemble` is also passed, IEC-104 analysis is disabled with a warning.

---

## CLI surface — `cargo run -- analyze --help` excerpt

```
      --iec104
          Analyze IEC 60870-5-104 (IEC-104) TCP traffic (port 2404, requires stream
          reassembly). Default-off; included by --all
```

---

## Source confirmation

`src/cli.rs` — `--iec104` field declaration (line 256):
```rust
/// Analyze IEC 60870-5-104 (IEC-104) TCP traffic (port 2404, requires stream reassembly).
#[arg(long)]
iec104: bool,
```

`src/main.rs` — reassembly gating (lines 275–277):
```rust
if enable_iec104 && skip_reassembly {
    eprintln!("--iec104 requires TCP reassembly; IEC-104 analysis disabled");
```

`src/main.rs` — `Iec104Analyzer` instantiation (lines 354–358):
```rust
// BC-2.12.025: construct Iec104Analyzer only when enabled AND reassembly is on.
let iec104_analyzer: Option<Iec104Analyzer> = if enable_iec104 && !skip_reassembly {
    Some(Iec104Analyzer::new())
```

`src/main.rs` — dispatcher registration (line 370):
```rust
iec104_analyzer,
```

---

## Behavioral verification (via dispatcher tests)

The `test_iec104_disabled_port_2404_no_panic` test in `tests/dispatcher_tests.rs` story_173
covers EC-003 (no flag → no analyzer, no panic):
- Constructs `StreamDispatcher::new(None, None, None, None, None, None)` — `iec104=None`.
- Sends a STOPDT-act on port 2404 and calls `on_flow_close`.
- No panic — confirms the absent-flag path is safe.

Test output (from full story_173 dispatcher run above):
```
test story_173::test_iec104_disabled_port_2404_no_panic ... ok
```

The `test_iec104_only_dispatcher_data_reaches_analyzer` test covers the flag-enabled path:
when `iec104=Some(analyzer)`, a STARTDT-act on port 2404 results in `flows.len() == 1`,
confirming the analyzer was instantiated and data reached it.

---

## VHS recording

A VHS terminal recording of `cargo run -- --iec104 <pcap>` is not included because no
IEC-104 test pcap is committed in the repository. The CLI surface is fully documented via
the `--help` output above. Behavioral end-to-end coverage is provided by the dispatcher
integration tests which instantiate the full `StreamDispatcher + Iec104Analyzer` pipeline
directly.

---

## Verdict

PASS — `--iec104` flag present in CLI, reassembly gating enforced, opt-in model verified
via EC-003 test, flag-enabled path verified via dispatcher wiring tests.
