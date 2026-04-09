# ADR 0003: Reporting Pipeline Layering — Data Layer is Raw, Display Layer Formats

**Status:** Accepted
**Date:** 2026-04-09
**Context:** PR #49 (issue #28) discovered a terminal injection vulnerability when untrusted bytes from network captures were interpolated into `Finding` strings rendered to the operator's terminal. Investigating the right fix surfaced a deeper architectural question about where formatting belongs in the reporting pipeline. PR #49 placed sanitization at the analyzer construction site, which solved the immediate problem but destroyed forensic data and required every analyzer to remember the rule. An audit (2026-04-08) found 7 unprotected interpolations in the HTTP analyzer with the same vulnerability class, demonstrating that construction-site rules don't propagate. This ADR establishes the architectural principle behind the right fix.

## Problem

Wirerust's reporting pipeline currently has a blurred boundary between *what* data the analyzers extract and *how* that data is presented. Several distinct concerns are entangled at the analyzer construction site:

- **Sanitization.** Untrusted bytes (TLS SNI hostnames, HTTP URIs, HTTP headers, etc.) flow through `String::from_utf8_lossy`, which preserves ASCII control bytes including ESC (`0x1b`). When the terminal reporter writes those bytes via `format!("{}", finding.summary)`, the analyst's terminal interprets the embedded ANSI escape sequences as commands. CWE-117 ("Improper Output Neutralization for Logs") covers this class.
- **Formatting.** Analyzers also pre-format human-readable text — `format!("Path traversal in URI: {uri}")` in HTTP, `cipher_name()` hex fallbacks in TLS, `truncate_uri()` length decisions, and so on. The same data is committed to one specific human-readable form before any reporter sees it.
- **Styling.** Color and bold/dim attributes are correctly applied by the terminal reporter, but the line text itself is built ad-hoc instead of being a clean transform of the raw `Finding`.

These share a single underlying problem: **formatting decisions are made at the data construction site, not at the rendering site.** The visible symptoms are:

1. **Terminal injection (the immediate vulnerability).** Control bytes from packet payloads reach the analyst's terminal because no display-layer escaping exists. Empirically demonstrated in PR #49 with an SNI of `b"\x1b[31m..."`.

2. **Forensic data loss.** PR #49's construction-site fix used `{:?}` (Debug formatter), which permanently replaces raw bytes in `Finding.summary` with their escaped form. Downstream consumers (JSON output, future CSV/SQLite/SIEM reporters) then see the escaped form forever. A Cyrillic SNI like `пример.рф` becomes `\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}.\u{440}\u{444}` in the JSON export — unreadable to a Russian-speaking analyst, and lossy for any tool that needs the original bytes.

3. **Tribal-knowledge enforcement.** Every new analyzer must remember the escape rule. The HTTP analyzer's 7 unprotected findings (added before PR #49) prove the rule never propagated. Future analyzers (Modbus, DNP3, SSH, SMB — issues #7, #8) would have to relearn it.

4. **Format coupling.** A future reporter that needs different formatting (HTML, SIEM JSON, CSV with its own escaping, localized alert text) would either have to undo and redo the construction-site formatting, or inherit the wrong context.

A single representation cannot serve both raw forensic data and human-readable formatted output. The pipeline needs a clear boundary.

## Decision

**The reporting pipeline is layered. The data layer (analyzers and the `Finding` struct) holds raw bytes. The display layer (each reporter) is responsible for all formatting that depends on the output medium — escaping, styling, truncation, localization.**

### The pipeline

```
Packet → Decoder → Dispatcher → Analyzer → Finding → Reporter → Output
                                            ─┬───            ─┬───
                                             │                └─ Display layer
                                             │                   (per-medium formatting)
                                             └─ Data layer
                                                (raw bytes, post-from_utf8_lossy)
```

The data layer is raw and forensic; the display layer formats for its medium and knows nothing about other layers.

### The layering rule

| Layer | Responsibility | Bytes are… |
|---|---|---|
| Analyzer | Extract data, build findings with raw strings | Raw (post-`from_utf8_lossy`) |
| `Finding` struct | Hold immutable forensic data | Raw |
| JSON reporter (`serde_json`) | Serialize for machine consumption | Escaped per RFC 8259 (automatic via serde) |
| Terminal reporter | Render for human display | Escaped per terminal-safety rules + styled |
| Future CSV / SQLite / HTML / SIEM reporters | Render for their format | Escape and format per their format's rules |

### Immediate scope: terminal-safe escaping

The first concrete consequence of the layering rule — and the motivating problem — is terminal-safe escaping. The terminal reporter calls `str::escape_default` (Rust stdlib) on every `Finding.summary` and every entry in `Finding.evidence` immediately before writing them to the output buffer.

`str::escape_default` escapes:
- C0 control bytes (`0x00`–`0x1f`, including ESC `0x1b`, BEL `0x07`, LF `0x0a`, CR `0x0d`)
- DEL (`0x7f`)
- backslash (`\\`)

It preserves:
- All printable ASCII
- All valid non-ASCII Unicode (Cyrillic, CJK, emoji, etc.)

The output is always valid UTF-8 and contains no bytes that a modern terminal will interpret as control sequences.

C1 control bytes (`0x80`–`0x9f`) are not a coverage gap. `Finding.summary` and `Finding.evidence` are `String` (guaranteed valid UTF-8) populated via `String::from_utf8_lossy` (which replaces invalid bytes with `U+FFFD`). A standalone byte in the C1 range cannot appear in a valid UTF-8 string, so C0 + DEL coverage is complete.

### Other formatting concerns (same principle, deferred scope)

Other things that currently happen at the analyzer construction site also belong in the display layer under this principle, but are intentionally NOT moved by the PR that introduces this ADR:

- **Truncation.** HTTP analyzer's `truncate_uri()` decides display length at construction. A future change could move length decisions to reporters, letting JSON consumers see full URIs while terminal consumers see truncated ones.
- **Cipher name hex fallback.** TLS analyzer's `cipher_name()` formats unknown cipher IDs as hex strings at construction. Could move to display.
- **Verdict / category / confidence text.** The `Display` impls on `Verdict`, `Confidence`, and `ThreatCategory` produce English strings directly. A future localization concern would move these to reporter-owned mappings.

These are noted to record the principle, not to commit to fixing them now. Each can be revisited if a concrete need appears — e.g., when adding a CSV reporter that wants full URIs, or an HTML reporter that wants different styling. YAGNI applies. This ADR establishes the boundary; subsequent work can push more responsibilities across it as needed.

### Where escaping does NOT happen

- **At the analyzer / construction site.** Analyzers store raw bytes in `Finding.summary` and `Finding.evidence`. They do not call `escape_default`, do not use `{:?}`, do not pre-escape anything.
- **In `Finding`'s `Display` impl.** It produces raw text and is documented as such. The terminal reporter does not use it (it builds output directly from struct fields and applies the helper).
- **In the JSON reporter.** `serde_json` already escapes per RFC 8259, so JSON output is safe with no extra work.

## Alternatives Considered

### Construction-site escaping (the PR #49 approach)

Each analyzer escapes untrusted bytes before they enter the `Finding` struct, e.g. via the Debug formatter `{hostname:?}`.

- **Pro:** Visible at the danger point.
- **Con (forensics):** Permanently replaces raw bytes with escaped form. JSON consumers and future reporters lose the original data. Cyrillic SNIs become hex blobs across all output paths.
- **Con (correctness):** Easy to forget. Every new analyzer must remember the rule. The HTTP analyzer's 7 unprotected findings prove this — the rule was tribal knowledge from one PR and didn't propagate.
- **Con (encoding once for many sinks):** OWASP guidance is explicit — encode at display time, not at storage time, because a single piece of data may render to multiple sinks (terminal, JSON, CSV, log file) each needing different escaping. Encoding at construction either breaks one sink or fails to protect another.
- **Con (architectural):** Conflates the data layer with the display layer. Once a project does this, every new reporter inherits the wrong shape.
- **Rejected.** Violates the layering principle and destroys forensic data.

### A wrapper type (`Untrusted<T>`)

Define a newtype like `pub struct Untrusted<'a>(&'a str)` with an `impl Display` that escapes, and require analyzers to wrap untrusted values: `format!("URI: {}", Untrusted(&parsed.uri))`.

- **Pro:** Type-system enforcement; impossible to forget.
- **Con:** Still construction-site escaping in disguise. The wrapped value either gets stored escaped (forensics loss) or the wrapper threads through the entire reporting pipeline (invasive).
- **Con:** Adds API surface without fixing the layering problem.
- **Rejected.** The complexity isn't justified once the display-time approach is adopted.

### `bstr::ByteSlice::escape_default` from the `bstr` crate

A third-party crate offering byte-slice escaping.

- **Pro:** Handles raw `&[u8]` directly.
- **Con:** Treats input as raw bytes and hex-escapes anything `> 0x7f`, including UTF-8 continuation bytes. A Cyrillic hostname becomes `\x{d0}\x{9f}\x{d1}\x{80}\x{d0}\x{b8}...` — unreadable.
- **Con:** Adds a dependency for a problem stdlib solves.
- **Rejected.** Same UX problem as the Debug formatter, plus a dependency.

### `char::escape_debug` (per-char Debug)

Iterate characters and apply `escape_debug` to each.

- **Pro:** Preserves printable ASCII.
- **Con:** Escapes all non-ASCII Unicode as `\u{...}` — same UX problem as `{:?}`.
- **Rejected.**

### Stripping vs. escaping

Drop dangerous bytes entirely instead of escaping them.

- **Pro:** Slightly shorter output.
- **Con:** Loses information. An attacker who embeds `\x1b[31mHACK\x1b[0m` in a hostname has done something noteworthy; the analyst should see *what* the attacker did, not just that "something" was stripped. Escaping preserves the evidence.
- **Rejected.**

### A separate sanitized view on `Finding`

Add a `Finding::display_summary()` method that returns the escaped form, and have the terminal reporter call it. Keep the raw `summary` field for JSON consumers.

- **Pro:** API discoverable on the type.
- **Con:** Couples the type to one specific display medium (terminal). The CSV reporter would need its own method, the HTML reporter another, etc. Method count grows with reporter count.
- **Con:** Encourages new analyzer authors to look at `display_summary()` and infer that the raw `summary` field is "the one that needs care" — same tribal-knowledge problem as construction-site escaping.
- **Rejected.** Reporters owning their own escaping is cleaner.

## Rationale

- **Matches OWASP guidance.** Output encoding belongs in the rendering layer, not the storage layer (CWE-117, OWASP XSS / output encoding cheat sheet). A single piece of alert data may render to multiple sinks; each needs its own escaping rules.
- **Matches the layering of `serde`.** `Finding` already implements `#[derive(Serialize)]`. The JSON reporter delegates escaping to serde — that's display-layer escaping for the JSON medium. Doing the same thing for the terminal medium is symmetric and unsurprising.
- **Preserves forensic data.** The raw bytes are kept in the `Finding` struct, available for JSON export, future reporters, and downstream tooling. An analyst exporting to JSON sees the actual SNI bytes (with serde's standard JSON escaping); only the terminal reporter applies terminal-safe escaping.
- **Single point of enforcement per medium.** Future analyzers don't need to remember any rule. Adding a new analyzer requires zero terminal-safety awareness. A new reporter (CSV, HTML, etc.) gets one place to apply its own escaping.
- **Extensible.** When a future need appears — localization, HTML rendering, different truncation per medium — the pipeline already has the boundary in place. The work is in the display layer, not in every analyzer.
- **`str::escape_default` is the right primitive for the immediate fix.** Stdlib (no dependency), preserves valid Unicode, and escapes exactly the bytes that constitute the threat (C0 + DEL). Cross-checked against `char::escape_debug` (mangles Unicode), `bstr::escape_default` (mangles Unicode), and `{:?}` Debug formatter (mangles Unicode).
- **Validated.** OWASP encoding guidance, RFC 8259 + serde_json behavior, and `str::escape_default` semantics all confirmed via Perplexity 2026-04-08.

## Consequences

### File-level changes required by the introducing PR

| File | Change |
|------|--------|
| `src/reporter/terminal.rs` | Add a private `escape_for_terminal(s: &str) -> String` helper at file scope that returns `s.escape_default().collect()`. Apply it to `f.summary` (line ~65, where `f.summary` is interpolated into the line `format!`) and to each `ev` in `f.evidence` (line ~81) before writing to the output buffer. The helper is private to the terminal reporter — other reporters that need it (e.g., a future CSV reporter) implement at their own boundary, since each output medium has different escaping rules. |
| `src/analyzer/tls.rs` | Replace `{hostname:?}` (line ~349) and `{lossy:?}` (line ~369) with `{hostname}` / `{lossy}`. Update the inline doc comments that explain *why* the Debug formatter was used; replace them with a pointer to this ADR. |
| `src/findings.rs` | Add a `///` doc comment on `impl Display for Finding` noting that it produces RAW text and is NOT safe for terminal display; consumers wanting safe display should go through the terminal reporter. |
| `src/analyzer/http.rs` | **No changes required.** Existing raw interpolations are now correct under the new policy. |
| `src/analyzer/dns.rs` | **No changes required.** DNS analyzer's `analyze()` returns `Vec::new()` — emits no findings. |
| `docs/adr/0002-modular-protocol-analyzers.md` | Add a cross-reference in the "Finding Generation Guidelines" section pointing to this ADR, so readers of the analyzer pattern doc also see the layering principle. |

### Tests required

- Unit test on the helper covering: ESC (`0x1b`), BEL (`0x07`), DEL (`0x7f`), backslash, Cyrillic preservation, emoji preservation, mixed content.
- End-to-end regression test: build a `Finding` whose `summary` contains a literal `\x1b[31mRED\x1b[0m` byte sequence. Assert that:
  - the terminal reporter's output contains no raw `0x1b` byte and contains the escaped form,
  - the JSON reporter's output contains `\u001b` (serde's escaping),
  - the `Finding.summary` field on the struct still contains the literal `0x1b` byte (forensic preservation).

### Behavioral changes visible to users

- TLS findings for non-ASCII / non-UTF-8 SNI hostnames will display readably in the terminal: a Cyrillic SNI like `пример.рф` will appear as `пример.рф` (not `\u{43f}\u{440}...`). Embedded ESC bytes still get escaped to `\x1b`.
- TLS findings in JSON output will contain the raw hostname (with serde's standard JSON string escaping for control bytes only). Downstream tooling that previously saw `\u{...}` literals will now see the actual UTF-8 hostname.
- HTTP findings (path traversal, web shell, admin panel, unusual method, missing host, long URI, empty UA) gain terminal-safety. Previously, an attacker could embed terminal control sequences in a URI and have them rendered live in the analyst's report. They can no longer.

### Non-changes (intentional)

- `Finding` struct shape is unchanged. No new fields, no wrapper types, no per-medium accessors.
- The JSON reporter is unchanged.
- The DNS and HTTP analyzers are unchanged.
- Truncation, cipher-name formatting, and verdict-text English formatting stay at construction site for now (deferred per "Other formatting concerns" above).
- No new dependencies.

### Binding rules for future contributors

> **Rule 1 (analyzer authors):** Analyzers MUST store untrusted bytes raw in `Finding.summary` and `Finding.evidence`. They MUST NOT escape, debug-format, or otherwise pre-encode untrusted bytes before assigning to those fields.
>
> **Rule 2 (reporter authors):** Each reporter MUST apply medium-appropriate escaping at its render boundary. The terminal reporter escapes for terminal-safety; the JSON reporter relies on `serde_json`'s automatic RFC 8259 escaping; future reporters apply their own format's rules.
>
> **Rule 3 (display-layer formatting in general):** New formatting concerns that depend on the output medium (truncation, styling, localization, etc.) belong in the reporter, not in the analyzer. When in doubt, push it across the boundary.

## Validation

This decision was validated through targeted Perplexity queries on 2026-04-08:

- **Output encoding placement:** OWASP guidance (XSS prevention cheat sheet, CWE-117 log injection) recommends encoding at display time, not at storage time. Encoding at construction creates premature commitment to one output context and breaks others. Confirmed.
- **`serde_json` control byte handling:** `serde_json` automatically escapes control bytes (including ESC `0x1b`) as `\u001b`, per RFC 8259 §7. JSON output is safe with no analyzer code. Confirmed.
- **`str::escape_default` semantics:** Confirmed against Rust stdlib behavior — uses `char::is_ascii_control` internally, escapes C0 + DEL + backslash, preserves all valid non-ASCII Unicode as raw UTF-8 bytes. Compared against `char::escape_debug`, `bstr::ByteSlice::escape_default`, and Debug formatter `{:?}`; `str::escape_default` is the only stdlib option that preserves multi-byte UTF-8.
- **C1 control byte risk:** Standalone C1 bytes (`0x80`–`0x9f`) cannot appear in a valid UTF-8 `String`. Since `Finding.summary` is `String` and analyzers populate it via `String::from_utf8_lossy`, C0 + DEL coverage is complete.
