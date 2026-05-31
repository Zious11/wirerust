# Demo Evidence Report — STORY-096

**Story:** STORY-096 — Absent Behavior Contracts: Removed Flags Rejected by clap
**Recorded:** 2026-05-31
**Binary:** `target/debug/wirerust` (built from worktree, branch `develop`)
**Tool:** VHS 0.11.0 (terminal recording)
**Product type:** CLI (Rust/clap)

---

## Coverage Summary

STORY-096 defines 10 acceptance criteria (AC-001 through AC-010) and 4 edge
cases (EC-001 through EC-004). This story proves **absence** of four removed
flags (`--threats`, `--beacon`, `--filter`, `--verbose`) from the CLI surface.

### Directly demonstrated via VHS recordings (externally observable at terminal)

| AC/EC | Observable signal | Recording |
|-------|-------------------|-----------|
| AC-001 | `--threats` → clap `unexpected argument` error | `AC-001-threats-flag-rejected` |
| AC-003 | `--beacon` → clap `unexpected argument` error | `AC-003-beacon-flag-rejected` |
| AC-005 | `--filter "tcp port 80"` → clap `unexpected argument` error | `AC-005-filter-flag-rejected` |
| AC-007 | `--verbose` → clap `unexpected argument` error | `AC-007-verbose-flag-rejected` |
| AC-008 | `-v` → clap `unexpected argument` error | `AC-008-verbose-short-flag-rejected` |
| AC-010 | `wirerust analyze test.pcap` → IO error (parse OK) | `AC-010-valid-invocation-unaffected` |
| EC-001 | `--threats` placed before subcommand → also rejected | `EC-001-threats-before-subcommand` |
| EC-002 | `--beacon` error fires before any analysis | within `AC-003-beacon-flag-rejected` |
| EC-003 | `--filter "tcp port 80"` space-separated BPF → error on `--filter` | `AC-005-filter-flag-rejected` |
| EC-004 | `wirerust analyze --http test.pcap` → IO error (parse OK) | `AC-010-valid-invocation-unaffected` |

### Not demonstrated as standalone VHS recordings (structural/source assertions)

These acceptance criteria express **mutation-resistant unit tests** that
introspect source code and `Cargo.toml`. They are not externally observable at
the terminal and cannot be meaningfully represented as terminal recordings.
They are verified by the test suite (`cargo test --all-targets`).

- **AC-002** (BC-2.13.001 invariant 1): No `threats`-related field in `src/cli.rs`.
  Verified by: `test_threats_field_absent_from_cli()` — grep-based assertion that
  `grep -n 'threats' src/cli.rs` returns nothing.

- **AC-004** (BC-2.13.002 invariant 2): No `C2BeaconAnalyzer` or equivalent struct
  in `src/`. Verified by: `test_beacon_analyzer_absent_from_src()` — grep-based
  assertion over the full `src/` tree.

- **AC-006** (BC-2.13.003 invariant 2): No BPF library in `Cargo.toml`; all packets
  from an accepted pcap are processed without pre-filtering. Verified by:
  `test_bpf_filter_absent_from_src()` — structural assertion on `Cargo.toml`.

- **AC-009** (BC-2.13.004 invariant 1): No `--verbose` or `-v` declaration in
  `src/cli.rs`. Verified by: `test_verbose_field_absent_from_cli()` — grep-based
  assertion that `grep -n 'verbose' src/cli.rs` returns nothing relevant.

---

## Acceptance Criteria — Recording Map

### AC-001 (BC-2.13.001 postcondition 1): `--threats` rejected as unknown argument

**Command:** `wirerust analyze --threats test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--threats' found

  tip: to pass '--threats' as a value, use '-- --threats'

Usage: wirerust analyze [OPTIONS] <TARGETS>...
```
This is the `UnknownArgument` error kind mandated by the postcondition.
**Artifacts:**
- `AC-001-threats-flag-rejected.gif`
- `AC-001-threats-flag-rejected.webm`
- `AC-001-threats-flag-rejected.tape`

---

### EC-001 (BC-2.13.001 postcondition 1, edge case): `--threats` before subcommand also rejected

**Command:** `wirerust --threats analyze test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--threats' found

Usage: wirerust [OPTIONS] <COMMAND>

For more information, try '--help'.
```
Unknown flags are rejected regardless of position — before or after the
subcommand name.
**Artifacts:**
- `EC-001-threats-before-subcommand.gif`
- `EC-001-threats-before-subcommand.webm`
- `EC-001-threats-before-subcommand.tape`

---

### AC-003 / EC-002 (BC-2.13.002 postcondition 1): `--beacon` rejected; error fires before analysis

**Command:** `wirerust analyze --beacon test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--beacon' found

  tip: to pass '--beacon' as a value, use '-- --beacon'

Usage: wirerust analyze [OPTIONS] <TARGETS>...
```
EC-002 is satisfied: the error fires at clap parse time, before any analysis
begins — no partial analysis output precedes the error message.
**Artifacts:**
- `AC-003-beacon-flag-rejected.gif`
- `AC-003-beacon-flag-rejected.webm`
- `AC-003-beacon-flag-rejected.tape`

---

### AC-005 / EC-003 (BC-2.13.003 postcondition 1): `--filter` with space-separated BPF expression rejected

**Command:** `wirerust analyze --filter "tcp port 80" test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--filter' found

  tip: a similar argument exists: '--mitre'

Usage: wirerust analyze --mitre <TARGETS>...
```
EC-003 is satisfied: clap errors on `--filter` immediately; it does not attempt
to parse the BPF expression `"tcp port 80"` that follows.
**Artifacts:**
- `AC-005-filter-flag-rejected.gif`
- `AC-005-filter-flag-rejected.webm`
- `AC-005-filter-flag-rejected.tape`

---

### AC-007 (BC-2.13.004 postcondition 1): `--verbose` rejected as unknown argument

**Command:** `wirerust analyze --verbose test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--verbose' found

  tip: to pass '--verbose' as a value, use '-- --verbose'

Usage: wirerust analyze [OPTIONS] <TARGETS>...
```
**Artifacts:**
- `AC-007-verbose-flag-rejected.gif`
- `AC-007-verbose-flag-rejected.webm`
- `AC-007-verbose-flag-rejected.tape`

---

### AC-008 (BC-2.13.004 postcondition 1): Short form `-v` also rejected

**Command:** `wirerust analyze -v test.pcap`
**Observable evidence:** clap outputs:
```
error: unexpected argument '-v' found

  tip: to pass '-v' as a value, use '-- -v'

Usage: wirerust analyze [OPTIONS] <TARGETS>...
```
Both the long form (`--verbose`) and short form (`-v`) are absent from the CLI
surface; neither is declared.
**Artifacts:**
- `AC-008-verbose-short-flag-rejected.gif`
- `AC-008-verbose-short-flag-rejected.webm`
- `AC-008-verbose-short-flag-rejected.tape`

---

### AC-010 / EC-004 (BC-2.13.001 postcondition 3 / BC-2.13.002 postcondition 3): Valid invocations parse successfully

**Commands (both in one recording):**
1. `wirerust analyze test.pcap`
2. `wirerust analyze --http test.pcap`

**Observable evidence:** Both produce:
```
Error: Target not found: test.pcap
```
This is an IO-layer error, not a clap parse error. An IO error after argument
parsing proves that clap accepted the arguments and handed control to the
application. The absence of any `error: unexpected argument` or `Usage:` line
is the success signal.

Removed flags (`--threats`, `--beacon`, `--filter`, `--verbose`) are entirely
absent; their removal does not affect parsing of valid invocations.

EC-004 is satisfied: `wirerust analyze --http test.pcap` parses successfully
with `--http` accepted.
**Artifacts:**
- `AC-010-valid-invocation-unaffected.gif`
- `AC-010-valid-invocation-unaffected.webm`
- `AC-010-valid-invocation-unaffected.tape`

---

## Artifact Index

| File | Type | Covers |
|------|------|--------|
| `AC-001-threats-flag-rejected.gif` | GIF | AC-001 (error path) |
| `AC-001-threats-flag-rejected.webm` | WebM | AC-001 (error path) |
| `AC-001-threats-flag-rejected.tape` | VHS script | AC-001 (error path) |
| `EC-001-threats-before-subcommand.gif` | GIF | EC-001 (error path) |
| `EC-001-threats-before-subcommand.webm` | WebM | EC-001 (error path) |
| `EC-001-threats-before-subcommand.tape` | VHS script | EC-001 (error path) |
| `AC-003-beacon-flag-rejected.gif` | GIF | AC-003, EC-002 (error path) |
| `AC-003-beacon-flag-rejected.webm` | WebM | AC-003, EC-002 (error path) |
| `AC-003-beacon-flag-rejected.tape` | VHS script | AC-003, EC-002 (error path) |
| `AC-005-filter-flag-rejected.gif` | GIF | AC-005, EC-003 (error path) |
| `AC-005-filter-flag-rejected.webm` | WebM | AC-005, EC-003 (error path) |
| `AC-005-filter-flag-rejected.tape` | VHS script | AC-005, EC-003 (error path) |
| `AC-007-verbose-flag-rejected.gif` | GIF | AC-007 (error path) |
| `AC-007-verbose-flag-rejected.webm` | WebM | AC-007 (error path) |
| `AC-007-verbose-flag-rejected.tape` | VHS script | AC-007 (error path) |
| `AC-008-verbose-short-flag-rejected.gif` | GIF | AC-008 (error path) |
| `AC-008-verbose-short-flag-rejected.webm` | WebM | AC-008 (error path) |
| `AC-008-verbose-short-flag-rejected.tape` | VHS script | AC-008 (error path) |
| `AC-010-valid-invocation-unaffected.gif` | GIF | AC-010, EC-004 (success path) |
| `AC-010-valid-invocation-unaffected.webm` | WebM | AC-010, EC-004 (success path) |
| `AC-010-valid-invocation-unaffected.tape` | VHS script | AC-010, EC-004 (success path) |

---

## AC Coverage Matrix

| AC/EC | Demoed | Method | Notes |
|-------|--------|--------|-------|
| AC-001 | Yes | `AC-001-threats-flag-rejected` | clap `UnknownArgument` error |
| AC-002 | No — unit test only | `test_threats_field_absent_from_cli()` | Grep-based source assertion; not observable at terminal |
| AC-003 | Yes | `AC-003-beacon-flag-rejected` | clap `UnknownArgument` error |
| AC-004 | No — unit test only | `test_beacon_analyzer_absent_from_src()` | Grep-based source assertion; not observable at terminal |
| AC-005 | Yes | `AC-005-filter-flag-rejected` | clap `UnknownArgument` error |
| AC-006 | No — unit test only | `test_bpf_filter_absent_from_src()` | Cargo.toml structural assertion; not observable at terminal |
| AC-007 | Yes | `AC-007-verbose-flag-rejected` | clap `UnknownArgument` error |
| AC-008 | Yes | `AC-008-verbose-short-flag-rejected` | clap `UnknownArgument` error for `-v` |
| AC-009 | No — unit test only | `test_verbose_field_absent_from_cli()` | Grep-based source assertion; not observable at terminal |
| AC-010 | Yes | `AC-010-valid-invocation-unaffected` | IO error proves parse succeeded |
| EC-001 | Yes | `EC-001-threats-before-subcommand` | Global position also rejected |
| EC-002 | Yes | `AC-003-beacon-flag-rejected` | Error fires at parse time; no analysis output |
| EC-003 | Yes | `AC-005-filter-flag-rejected` | BPF expression not parsed after `--filter` error |
| EC-004 | Yes | `AC-010-valid-invocation-unaffected` | `--http` accepted; IO error confirms parse success |

**Demoed:** AC-001, AC-003, AC-005, AC-007, AC-008, AC-010, EC-001, EC-002, EC-003, EC-004 (10 of 14)
**Unit-test only (structural absence — not observable at terminal):** AC-002, AC-004, AC-006, AC-009

---

## Interpretation Note

For STORY-096, every "error path" demo deliberately shows a clap parse failure.
The signal that a removed flag is correctly absent is the presence of:
```
error: unexpected argument '--<flag>' found
```
with a `Usage:` line. The converse — the "success path" for AC-010/EC-004 —
shows an IO-layer `Error: Target not found:` message, which proves clap accepted
the invocation and passed control past the parse stage. A clap error would
have printed `Usage:` instead.

The four purely structural ACs (AC-002, AC-004, AC-006, AC-009) cannot be
represented as terminal recordings because absence of a source field or
dependency produces no observable terminal output. These are correctly covered
by mutation-resistant unit tests that grep source files and Cargo.toml.
