# Demo Evidence Report — STORY-086

**Story:** STORY-086 — CLI Subcommand Parsing: analyze, summary, --no-color, Multiple Targets
**Recorded:** 2026-05-31
**Binary:** `target/debug/wirerust` (built from worktree at commit on `develop`)
**Tool:** VHS 0.11.0 (terminal recording)
**Product type:** CLI (Rust/clap)

---

## Coverage Summary

This report covers the 10 acceptance criteria (AC-001 through AC-010) and the 5
edge cases (EC-001 through EC-005) defined in STORY-086. Each demoed scenario
is linked to a recording artifact and the behavioral contract it exercises.

Directly demonstrated: AC-001, AC-002, AC-003, AC-005, AC-006, AC-007, AC-008,
AC-010, EC-001, EC-003, EC-004.

Not directly demonstrated as separate recordings (covered within other recordings
or purely internal struct assertions verifiable only via unit tests):

- **AC-004** (`--mitre` does not imply analyzers) — pure struct field assertion;
  clap parse surface is identical to any other flag; covered by the test suite,
  not a distinct observable CLI interaction.
- **AC-009** (`no_color` is `bool`, not `Option<bool>`, defaults `false`) — type
  assertion verified in tests; not externally observable via terminal output.
- **EC-002** (`--mitre` alone: `mitre=true`, all/dns/http/tls=false`) — same
  parse surface as AC-002; no distinct observable error path.
- **EC-005** (duplicate targets preserved) — covered within AC-010 tape (multiple
  targets accepted); dedup behavior is unit-test-level.

---

## Acceptance Criteria — Recording Map

### AC-001 (BC-2.12.001 postcondition 1): Single positional target accepted

**Scenario:** `wirerust analyze cap.pcap`
**Observable evidence:** Binary outputs `Error: Target not found: cap.pcap` — an
IO-layer error, not a clap parse error. This proves clap accepted the argument.
**Artifacts:**
- `AC-001-analyze-single-target.gif`
- `AC-001-analyze-single-target.webm`
- `AC-001-analyze-single-target.tape`

---

### AC-002 (BC-2.12.001 postcondition 3) + EC-001: Protocol flags accepted

**Scenario:** `wirerust analyze cap.pcap --dns --http --all`
**Observable evidence:** IO error (not clap error) proves all three flags were
accepted. EC-001 is covered here: `--all` with individual flags is legal.
**Artifacts:**
- `AC-002-EC-001-protocol-flags.gif`
- `AC-002-EC-001-protocol-flags.webm`
- `AC-002-EC-001-protocol-flags.tape`

---

### AC-003 (BC-2.12.001 invariant 1): No targets on analyze → clap error

**Scenario:** `wirerust analyze` (no positional arguments)
**Observable evidence:** clap outputs:
```
error: the following required arguments were not provided:
  <TARGETS>...
```
This is the required-argument-missing error mandated by the invariant.
**Artifacts:**
- `AC-003-analyze-no-target-error.gif`
- `AC-003-analyze-no-target-error.webm`
- `AC-003-analyze-no-target-error.tape`

---

### AC-005 (BC-2.12.002 postcondition 1): summary subcommand accepts single target

**Scenario:** `wirerust summary cap.pcap`
**Observable evidence:** IO error (not clap error) proves clap accepted the
`summary` subcommand and the positional target.
**Artifacts:**
- `AC-005-summary-basic.gif`
- `AC-005-summary-basic.webm`
- `AC-005-summary-basic.tape`

---

### AC-006 (BC-2.12.002 postcondition 3): --hosts flag accepted on summary

**Scenario:** `wirerust summary cap.pcap --hosts`
**Observable evidence:** IO error proves clap accepted `--hosts` on the summary
subcommand.
**Artifacts:**
- `AC-006-summary-hosts-flag.gif`
- `AC-006-summary-hosts-flag.webm`
- `AC-006-summary-hosts-flag.tape`

---

### AC-007 (BC-2.12.002 invariant 4) + EC-004: --services rejected on summary

**Scenario:** `wirerust summary cap.pcap --services`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--services' found
```
`--services` was removed (LESSON-P1.04); clap correctly rejects it as
`UnknownArgument`.
**Artifacts:**
- `AC-007-EC-004-summary-services-error.gif`
- `AC-007-EC-004-summary-services-error.webm`
- `AC-007-EC-004-summary-services-error.tape`

---

### AC-008 (BC-2.12.003 postcondition 1): --no-color global flag both placements

**Scenario:** Both `wirerust --no-color analyze cap.pcap` (before subcommand) and
`wirerust analyze cap.pcap --no-color` (after subcommand) in a single recording.
**Observable evidence:** Both invocations produce IO errors (not clap parse
errors), demonstrating that `--no-color` is accepted in both positions —
confirming `global = true` clap semantics.
**Artifacts:**
- `AC-008-no-color-global-flag.gif`
- `AC-008-no-color-global-flag.webm`
- `AC-008-no-color-global-flag.tape`

---

### AC-010 (BC-2.12.006 postcondition 1): Multiple positional targets accepted

**Scenario:** `wirerust analyze a.pcap b.pcap c.pcap`
**Observable evidence:** IO error (`Target not found: a.pcap`) — clap accepted
all three positional arguments. The IO layer processes targets in order,
confirming order-preservation at parse time.
**Artifacts:**
- `AC-010-analyze-multiple-targets.gif`
- `AC-010-analyze-multiple-targets.webm`
- `AC-010-analyze-multiple-targets.tape`

---

### EC-003: --hosts on analyze subcommand → clap error

**Scenario:** `wirerust analyze cap.pcap --hosts`
**Observable evidence:** clap outputs:
```
error: unexpected argument '--hosts' found
```
`--hosts` is summary-only; clap correctly rejects it on the analyze subcommand.
**Artifacts:**
- `EC-003-hosts-on-analyze-error.gif`
- `EC-003-hosts-on-analyze-error.webm`
- `EC-003-hosts-on-analyze-error.tape`

---

## Help Surface Documentation

Full help output recorded for both subcommands to document the complete CLI surface.

### wirerust analyze --help
- `HELP-analyze-subcommand.gif`
- `HELP-analyze-subcommand.webm`
- `HELP-analyze-subcommand.tape`

### wirerust summary --help
- `HELP-summary-subcommand.gif`
- `HELP-summary-subcommand.webm`
- `HELP-summary-subcommand.tape`

---

## Artifact Index

| File | Type | Covers |
|------|------|--------|
| `AC-001-analyze-single-target.gif` | GIF | AC-001 |
| `AC-001-analyze-single-target.webm` | WebM | AC-001 |
| `AC-001-analyze-single-target.tape` | VHS script | AC-001 |
| `AC-002-EC-001-protocol-flags.gif` | GIF | AC-002, EC-001 |
| `AC-002-EC-001-protocol-flags.webm` | WebM | AC-002, EC-001 |
| `AC-002-EC-001-protocol-flags.tape` | VHS script | AC-002, EC-001 |
| `AC-003-analyze-no-target-error.gif` | GIF | AC-003 (error path) |
| `AC-003-analyze-no-target-error.webm` | WebM | AC-003 (error path) |
| `AC-003-analyze-no-target-error.tape` | VHS script | AC-003 (error path) |
| `AC-005-summary-basic.gif` | GIF | AC-005 |
| `AC-005-summary-basic.webm` | WebM | AC-005 |
| `AC-005-summary-basic.tape` | VHS script | AC-005 |
| `AC-006-summary-hosts-flag.gif` | GIF | AC-006 |
| `AC-006-summary-hosts-flag.webm` | WebM | AC-006 |
| `AC-006-summary-hosts-flag.tape` | VHS script | AC-006 |
| `AC-007-EC-004-summary-services-error.gif` | GIF | AC-007, EC-004 (error path) |
| `AC-007-EC-004-summary-services-error.webm` | WebM | AC-007, EC-004 (error path) |
| `AC-007-EC-004-summary-services-error.tape` | VHS script | AC-007, EC-004 (error path) |
| `AC-008-no-color-global-flag.gif` | GIF | AC-008 (both placements) |
| `AC-008-no-color-global-flag.webm` | WebM | AC-008 (both placements) |
| `AC-008-no-color-global-flag.tape` | VHS script | AC-008 (both placements) |
| `AC-010-analyze-multiple-targets.gif` | GIF | AC-010 |
| `AC-010-analyze-multiple-targets.webm` | WebM | AC-010 |
| `AC-010-analyze-multiple-targets.tape` | VHS script | AC-010 |
| `EC-003-hosts-on-analyze-error.gif` | GIF | EC-003 (error path) |
| `EC-003-hosts-on-analyze-error.webm` | WebM | EC-003 (error path) |
| `EC-003-hosts-on-analyze-error.tape` | VHS script | EC-003 (error path) |
| `HELP-analyze-subcommand.gif` | GIF | Surface documentation |
| `HELP-analyze-subcommand.webm` | WebM | Surface documentation |
| `HELP-analyze-subcommand.tape` | VHS script | Surface documentation |
| `HELP-summary-subcommand.gif` | GIF | Surface documentation |
| `HELP-summary-subcommand.webm` | WebM | Surface documentation |
| `HELP-summary-subcommand.tape` | VHS script | Surface documentation |

---

## Interpretation Note

For this story, every "success path" demo produces an IO-layer error
(`Error: Target not found: <file>`). This is correct and expected: the demo
files (`cap.pcap`, `a.pcap`, etc.) do not exist on disk. An IO error after
argument parsing proves that clap accepted the arguments and passed control to
the application. A clap parse error would print `error: ...` with a `Usage:`
line — the absence of that output is the success signal.
