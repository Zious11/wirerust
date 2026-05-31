# Demo Evidence Report — STORY-087

**Story:** STORY-087 — Output Format Flags and Reassembly Configuration Flags
**Recorded:** 2026-05-31
**Binary:** `target/debug/wirerust` (built from worktree at commit on `develop`)
**Tool:** VHS 0.11.0 (terminal recording)
**Product type:** CLI (Rust/clap)

---

## Coverage Summary

STORY-087 defines 12 acceptance criteria (AC-001 through AC-012) and 5 edge cases
(EC-001 through EC-005).

**Directly demonstrated (externally observable clap behavior):**
AC-001, AC-002, AC-004, AC-008, AC-009, AC-010, AC-011, AC-012, EC-001, EC-003.

**Not demonstrated as recordings (internal struct assertions only):**

- **AC-003** (`--output-format` absent → `output_format = None`) — the binary
  accepts any valid invocation without `--output-format` (indistinguishable from
  any other accepted invocation at the terminal); the `None` value is a struct
  field assertion verifiable only via `Cli::try_parse_from` in unit tests.
  Covered by `test_output_format_absent_is_none()`.

- **AC-005** (`--reassembly-depth` absent → `reassembly_depth = 10`) — the
  default value is a struct field assertion; the binary produces identical terminal
  output with or without the flag when the file is absent. Covered by
  `test_reassembly_depth_default_is_10()`.

- **AC-006** (`--reassembly-memcap` absent → `reassembly_memcap = 1024`) — same
  reasoning as AC-005. Covered by `test_reassembly_memcap_default_is_1024()`.

- **AC-007** (threshold flags are `None` when absent, `Some(value)` when present)
  — the `None` path is a struct field assertion; the `Some` path for
  `--overlap-threshold` is demonstrated in EC-003. Covered by
  `test_reassembly_threshold_flags_default_none()`.

- **EC-002** (`--small-segment-max-bytes 0` → `Some(0)`) — accepted invocation
  with IO-layer error; indistinguishable from AC-009 at the terminal level; covered
  by the unit test suite.

- **EC-004** (`--output-format` and `--json` together → `--json` wins via
  `resolve_format`) — tested in STORY-089 per story spec; out of scope here.

- **EC-005** (no reassembly flags at all → `reassemble = false`, `no_reassemble =
  false`, `depth = 10`, `memcap = 1024`) — struct field defaults; not
  externally observable. Covered by the unit test suite.

---

## Interpretation Note

For this story, every "success path" demo produces `Error: Target not found:
x.pcap`. This is correct and expected: the demo file does not exist on disk. An
IO-layer error after argument parsing proves that clap accepted the arguments and
passed control to the application. A clap parse error would print `error: ...`
with a `Usage:` line — the absence of that output is the success signal.

---

## Acceptance Criteria — Recording Map

### AC-001 (BC-2.12.004 postcondition 1): --output-format json accepted

**Scenario:** `wirerust --output-format json summary x.pcap`
**Observable evidence:** Binary outputs `Error: Target not found: x.pcap` — an
IO-layer error, not a clap parse error. Clap accepted `json` as a valid
`OutputFormat` variant and passed control to the application.
**Artifacts:**
- `AC-001-002-output-format-json-csv.gif`
- `AC-001-002-output-format-json-csv.webm`
- `AC-001-002-output-format-json-csv.tape`

---

### AC-002 (BC-2.12.004 postcondition 2): --output-format csv accepted

**Scenario:** `wirerust --output-format csv summary x.pcap`
**Observable evidence:** IO error proves clap accepted `csv` as a valid variant.
Captured in the same tape as AC-001 (second invocation).
**Artifacts:**
- `AC-001-002-output-format-json-csv.gif`
- `AC-001-002-output-format-json-csv.webm`
- `AC-001-002-output-format-json-csv.tape`

---

### AC-004 (BC-2.12.004 postcondition 4): --output-format xml → clap error

**Scenario:** `wirerust --output-format xml summary x.pcap`
**Observable evidence:** clap outputs:
```
error: invalid value 'xml' for '--output-format <OUTPUT_FORMAT>'
  [possible values: json, csv]
```
This is the `ValueEnum` rejection mandated by BC-2.12.004 invariant 1.
**Artifacts:**
- `AC-004-output-format-invalid-xml.gif`
- `AC-004-output-format-invalid-xml.webm`
- `AC-004-output-format-invalid-xml.tape`

---

### AC-008 (BC-2.12.005 postcondition 6): --overlap-threshold 256 → clap range error

**Scenario:** `wirerust --overlap-threshold 256 analyze x.pcap`
**Observable evidence:** clap outputs:
```
error: invalid value '256' for '--overlap-threshold <OVERLAP_THRESHOLD>': 256 is not in 0..=255
```
The `value_parser = clap::value_parser!(u32).range(0..=255)` constraint is
enforced by clap before the application receives control.
**Artifacts:**
- `AC-008-overlap-threshold-out-of-range.gif`
- `AC-008-overlap-threshold-out-of-range.webm`
- `AC-008-overlap-threshold-out-of-range.tape`

---

### AC-009 (BC-2.12.005 invariant 3): --small-segment-ignore-ports comma-delimited

**Scenario:** `wirerust --small-segment-ignore-ports 23,513 analyze x.pcap`
**Observable evidence:** IO error (`Error: Target not found: x.pcap`) proves
clap parsed `23,513` as a comma-delimited `Vec<u16>` and accepted the argument.
**Artifacts:**
- `AC-009-small-segment-ignore-ports.gif`
- `AC-009-small-segment-ignore-ports.webm`
- `AC-009-small-segment-ignore-ports.tape`

---

### AC-010 (BC-2.12.007 postcondition 1): --reassemble --no-reassemble → conflict

**Scenario:** `wirerust --reassemble --no-reassemble analyze x.pcap`
**Observable evidence:** clap outputs:
```
error: the argument '--reassemble' cannot be used with '--no-reassemble'
```
The `conflicts_with = "no_reassemble"` declaration on `--reassemble` produces
an `ArgumentConflict` error kind as required by BC-2.12.007.
**Artifacts:**
- `AC-010-011-reassemble-conflict.gif`
- `AC-010-011-reassemble-conflict.webm`
- `AC-010-011-reassemble-conflict.tape`

---

### AC-011 (BC-2.12.007 invariant 1): --no-reassemble --reassemble → also conflict

**Scenario:** `wirerust --no-reassemble --reassemble analyze x.pcap`
**Observable evidence:** clap outputs:
```
error: the argument '--no-reassemble' cannot be used with '--reassemble'
```
The conflict is bidirectional (clap makes `conflicts_with` symmetric); reversed
order also returns `Err(ArgumentConflict)`. Captured in the same tape as
AC-010 (second invocation).
**Artifacts:**
- `AC-010-011-reassemble-conflict.gif`
- `AC-010-011-reassemble-conflict.webm`
- `AC-010-011-reassemble-conflict.tape`

---

### AC-012 (BC-2.12.007 edge case EC-003): --reassemble alone → accepted

**Scenario:** `wirerust --reassemble analyze x.pcap`
**Observable evidence:** IO error proves clap accepted `--reassemble` without
`--no-reassemble`; `cli.reassemble = true` is confirmed by the absence of a
conflict error.
**Artifacts:**
- `AC-012-reassemble-alone.gif`
- `AC-012-reassemble-alone.webm`
- `AC-012-reassemble-alone.tape`

---

## Edge Cases — Recording Map

### EC-001: --reassembly-depth 0 → accepted

**Scenario:** `wirerust --reassembly-depth 0 analyze x.pcap`
**Observable evidence:** IO error proves 0 is a valid `u64` for this flag;
`reassembly_depth = 0` is accepted by clap.
**Artifacts:**
- `EC-001-reassembly-depth-zero.gif`
- `EC-001-reassembly-depth-zero.webm`
- `EC-001-reassembly-depth-zero.tape`

---

### EC-003: --overlap-threshold 255 (max boundary) → accepted

**Scenario:** `wirerust --overlap-threshold 255 analyze x.pcap`
**Observable evidence:** IO error proves 255 (the top of the `0..=255` range) is
accepted by clap; `overlap_threshold = Some(255)`.
**Artifacts:**
- `EC-003-overlap-threshold-max.gif`
- `EC-003-overlap-threshold-max.webm`
- `EC-003-overlap-threshold-max.tape`

---

## Help Surface Documentation

Full top-level `wirerust --help` recorded to document the global flags surface
including `--output-format`, `--reassemble`, `--no-reassemble`, and all threshold
flags added in this story.

- `HELP-global-flags.gif`
- `HELP-global-flags.webm`
- `HELP-global-flags.tape`

---

## Artifact Index

| File | Type | Covers |
|------|------|--------|
| `AC-001-002-output-format-json-csv.gif` | GIF | AC-001, AC-002 |
| `AC-001-002-output-format-json-csv.webm` | WebM | AC-001, AC-002 |
| `AC-001-002-output-format-json-csv.tape` | VHS script | AC-001, AC-002 |
| `AC-004-output-format-invalid-xml.gif` | GIF | AC-004 (error path) |
| `AC-004-output-format-invalid-xml.webm` | WebM | AC-004 (error path) |
| `AC-004-output-format-invalid-xml.tape` | VHS script | AC-004 (error path) |
| `AC-008-overlap-threshold-out-of-range.gif` | GIF | AC-008 (error path) |
| `AC-008-overlap-threshold-out-of-range.webm` | WebM | AC-008 (error path) |
| `AC-008-overlap-threshold-out-of-range.tape` | VHS script | AC-008 (error path) |
| `AC-009-small-segment-ignore-ports.gif` | GIF | AC-009 |
| `AC-009-small-segment-ignore-ports.webm` | WebM | AC-009 |
| `AC-009-small-segment-ignore-ports.tape` | VHS script | AC-009 |
| `AC-010-011-reassemble-conflict.gif` | GIF | AC-010, AC-011 (error path, both orders) |
| `AC-010-011-reassemble-conflict.webm` | WebM | AC-010, AC-011 (error path, both orders) |
| `AC-010-011-reassemble-conflict.tape` | VHS script | AC-010, AC-011 |
| `AC-012-reassemble-alone.gif` | GIF | AC-012 |
| `AC-012-reassemble-alone.webm` | WebM | AC-012 |
| `AC-012-reassemble-alone.tape` | VHS script | AC-012 |
| `EC-001-reassembly-depth-zero.gif` | GIF | EC-001 |
| `EC-001-reassembly-depth-zero.webm` | WebM | EC-001 |
| `EC-001-reassembly-depth-zero.tape` | VHS script | EC-001 |
| `EC-003-overlap-threshold-max.gif` | GIF | EC-003 |
| `EC-003-overlap-threshold-max.webm` | WebM | EC-003 |
| `EC-003-overlap-threshold-max.tape` | VHS script | EC-003 |
| `HELP-global-flags.gif` | GIF | Surface documentation |
| `HELP-global-flags.webm` | WebM | Surface documentation |
| `HELP-global-flags.tape` | VHS script | Surface documentation |
