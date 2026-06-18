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

The first concrete consequence of the layering rule — and the motivating problem — is terminal-safe escaping. The terminal reporter defines a private `escape_for_terminal` helper that iterates the input `str`'s characters and applies `char::escape_default()` only to characters matching `char::is_ascii_control()`, the C1 control range `U+0080..=U+009F`, or the backslash. All other characters are passed through unchanged:

```rust
fn escape_for_terminal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_control() || ('\u{80}'..='\u{9f}').contains(&c) || c == '\\' {
            for e in c.escape_default() {
                out.push(e);
            }
        } else {
            out.push(c);
        }
    }
    out
}
```

The helper escapes:
- C0 control bytes (`0x00`–`0x1f`, including ESC `0x1b`, BEL `0x07`, LF `0x0a`, CR `0x0d`) — rendered as `\u{1b}`, `\n`, `\t`, etc. via `char::escape_default`
- DEL (`0x7f`) — rendered as `\u{7f}`
- C1 control codepoints (`U+0080`–`U+009F`, including NEL `U+0085` and CSI `U+009B`) — rendered as `\u{85}`, `\u{9b}`, etc. via `char::escape_default`. See "C1 control codepoints" below for the rationale.
- backslash (`\\`)

It preserves:
- All printable ASCII
- All valid non-ASCII Unicode (Cyrillic, CJK, emoji, etc.) — passed through as raw UTF-8 bytes

**Why not stdlib `str::escape_default`?** Empirical verification (2026-04-09) showed that `str::escape_default` internally escapes *every* non-ASCII character via `char::escape_default`, so a Cyrillic hostname like `пример.рф` would become `\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}.\u{440}\u{444}` — the same UX problem as the Debug formatter in PR #49. This contradicted an earlier Perplexity answer (which conflated `str::escape_default` with the byte-oriented `std::ascii::escape_default`) and was caught during plan self-review. The custom helper is ~8 lines, stdlib-only, and does exactly what we need.

The output is always valid UTF-8 and contains no bytes that a modern terminal will interpret as control sequences.

C1 control codepoints (U+0080–U+009F) are also dangerous: U+009B (CSI) is the 8-bit equivalent of ESC[ and is interpreted as a Control Sequence Introducer by terminals in DEC S8C1T mode, and U+0085 (NEL) has similar semantics. **These codepoints CAN appear in a valid UTF-8 `String`**, encoded as two-byte sequences (e.g., `0xC2 0x9B` for U+009B). `String::from_utf8_lossy` passes such sequences through unchanged — they are valid UTF-8. An earlier draft of this ADR claimed that "a standalone byte in the 0x80–0x9f range cannot appear in a valid UTF-8 string"; that's correct for a single raw byte but misleading for the codepoint, which is the relevant question for a `String` produced by `from_utf8_lossy`.

The helper therefore escapes both C0 + DEL (via `char::is_ascii_control()`) AND the C1 range `U+0080..=U+009F` (via an explicit range check), plus backslash. Modern terminals in UTF-8 mode default to NOT interpreting C1 codepoints as controls (xterm, iTerm2, gnome-terminal, Windows Terminal, VS Code, tmux all decode UTF-8 first), so the practical threat is narrow; but S8C1T exists, can be enabled, and some legacy terminals honor it. Escaping C1 is cheap insurance and closes the gap the earlier reasoning overlooked.

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

### Stdlib `str::escape_default` (or `char::escape_debug`, which is equivalent for this purpose)

Apply the stdlib method unconditionally to every character.

- **Pro:** Stdlib, zero dependency.
- **Con:** Escapes *all* non-ASCII characters, not just control bytes. A Cyrillic hostname like `пример.рф` becomes `\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}.\u{440}\u{444}` — same UX problem as the Debug formatter. Verified empirically 2026-04-09. This was the original choice in an earlier draft of this ADR, reversed during plan self-review when the assumption was checked against actual stdlib behavior.
- **Rejected.** The custom helper adds ~8 lines of code to gate `escape_default` on `is_ascii_control()` and avoid mangling valid Unicode.

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
- **A small custom helper is the right primitive.** Built on stdlib `char::escape_default` + `char::is_ascii_control` plus an explicit C1 range check, ~15 lines, no dependency. Gates the escape on control-ness so valid Unicode (Cyrillic, CJK, emoji) passes through unchanged. Escapes exactly the characters that constitute the threat (C0 + C1 + DEL + backslash). The stdlib `str::escape_default` method was considered and rejected (it mangles all non-ASCII).
- **Validated.** OWASP encoding guidance and RFC 8259 + serde_json behavior confirmed via Perplexity 2026-04-08. The escape primitive was re-verified empirically (`rustc`-compiled program, 2026-04-09) after an initial Perplexity answer about `str::escape_default` turned out to be wrong — see Validation.

## Consequences

### File-level changes required by the introducing PR

| File | Change |
|------|--------|
| `src/reporter/terminal.rs` | Add a private `escape_for_terminal(s: &str) -> String` helper at file scope that iterates `s.chars()`, applies `char::escape_default()` for chars that are ASCII controls (C0 + DEL), C1 controls (`U+0080..=U+009F`), or backslash, and passes all other chars through. Apply it to `f.summary` (line ~65, where `f.summary` is interpolated into the line `format!`) and to each `ev` in `f.evidence` (line ~81) before writing to the output buffer. The helper is private to the terminal reporter — other reporters that need it (e.g., a future CSV reporter) implement at their own boundary, since each output medium has different escaping rules. |
| `src/analyzer/tls.rs` | **Done.** Raw hostname/lossy interpolation (`{hostname}` / `{lossy}`) is already in place. Inline doc comments reference this ADR at the emission sites. |
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

- TLS findings for non-ASCII / non-UTF-8 SNI hostnames will display readably in the terminal: a Cyrillic SNI like `пример.рф` will appear as `пример.рф` (not `\u{43f}\u{440}...`). Embedded control bytes still get escaped — an ESC `0x1b` renders as `\u{1b}` via `char::escape_default`.
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

## Display-Layer Aggregation (Issue #259 — v0.8.0)

**Status of this subsection:** Accepted (F1 gate decisions locked 2026-06-17)

### Context

On captures containing many repeated low-value findings — the canonical case is the HTTP
empty-User-Agent anomaly — the FINDINGS section floods with thousands of identical lines,
one per matching request. This is a direct consequence of the analyzer-layer principle: the
HTTP analyzer correctly emits one `Finding` per anomalous request (each has distinct
evidence). The flooding is a display-layer problem, not an emission-layer problem.

ADR 0003's governing principle — "the data layer holds raw bytes; the display layer formats
for its medium" — already provides the correct answer: collapsing repeated findings for
human readability is another display-layer transform that MUST NOT mutate the canonical
`Finding` stream.

### Decision

**Aggregate identical findings at the terminal-display layer only.** The `Finding` stream
and all machine-readable reporters (JSON, CSV) are unchanged. `TerminalReporter::render`
applies a private collapse pass before rendering when `collapse_findings` is `true`.

### Aggregation Key

Two findings belong to the same collapsed group when they are identical on all four
semantic fields: `(category, verdict, confidence, summary)`. Fields that vary per-instance
(`evidence`, `mitre_techniques`, `source_ip`, `timestamp`, `direction`) are NOT part of
the key. These per-instance fields are intentionally excluded because keying on `evidence`
would prevent collapse — every request has a distinct URI in its evidence line. Excluding
them is the direct analogue of Wireshark Expert Information keying on `(severity, message
text)` rather than on the per-frame detail.

The key is broader than Wireshark's two-field key: `category` and `confidence` are
included to prevent collapsing semantically distinct finding types that happen to share a
summary string (e.g., `Anomaly/INCONCLUSIVE/LOW` and `Reconnaissance/INCONCLUSIVE/LOW` are
distinct event types even if their summary texts are identical).

### Count Display and Evidence Sampling

A collapsed group of N findings renders as a single header line. The count is always
displayed:

- N = 1: no count suffix (singleton renders identically to current behavior)
- N ≥ 2: `(xN)` suffix appended after the summary, e.g.,
  `  [Anomaly] INCONCLUSIVE (LOW) - Empty User-Agent header (x3142)`
  (two leading spaces per `out.push_str("  {colored}\n")` in BC-2.11.026 PC-1/PC-2)

The ` (xN)` suffix is part of the string that is passed to the color function — it is appended
**before** colorization, not after the ANSI reset. The color ladder applied to the pre-suffix
string is the same verdict/confidence ladder used by the existing `render_finding_prefix`
(terminal.rs:209-221): `Likely/High → red().bold()`, `Likely/other → yellow`,
`Possible → yellow`, `Inconclusive → cyan`, `Unlikely → dimmed`. See BC-2.11.026 PC-6.

Evidence sampling: a collapsed group retains at most K = 3 evidence lines, taken from the
first min(N, K) findings in the group (in original emission order) — purely positional. From
each inspected member, `evidence[0]` is emitted if the member's evidence vec is non-empty;
an empty-evidence member contributes 0 lines and the window does NOT slide past it to the
next member. The total evidence lines rendered is therefore at most K but may be less if any
inspected member has an empty evidence vec (e.g., N=5, member[0] empty, K=3 → inspects
members[0,1,2], member[0] contributes 0, total = 2 lines, NOT 3). Evidence lines beyond the
window are discarded for terminal display only; they remain present in the full `Finding`
structs passed to JSON and CSV reporters. K = 3 is a hardcoded named constant
(`COLLAPSE_EVIDENCE_SAMPLES`). It is not configurable per CLI flag to keep the surface
small; a future ADR may revisit this if operator feedback indicates the need.

The `escape_for_terminal` invariant (VP-012) is unchanged. The collapse path calls
`escape_for_terminal` directly on each sampled evidence line (per BC-2.11.026 PC-4
observable line-order contract) — it does NOT delegate to `render_finding_prefix`'s evidence
loop (that loop renders all entries of one finding; the collapse path samples evidence[0]
across up to K different member findings). The escape FUNCTION guarantee is preserved;
there is no bypass of the escape helper.

### Default-On with `--no-collapse` Opt-Out

Collapse is **default-on** for terminal output. A `--no-collapse` flag on the
`analyze` subcommand reverts to one-line-per-finding behavior (the behavior before
v0.8.0). This flag is scoped to `Commands::Analyze` only; the `summary` subcommand has no
findings section and is unaffected.

**Rationale for default-on:** The flooding scenario is the primary motivation. Terminal
output is not a machine-readable contract — that role belongs to `--json` and `--csv`.
Defaulting to the better human experience matches the Wireshark Expert Information model
(aggregated view is the default; expanding to per-packet detail requires a user action).
Making collapse opt-in would require explicit discovery of the flag and would not address
the alert-fatigue problem for new users.

**Non-goal:** This is NOT the syslog "last message repeated N times" anti-pattern. syslog
collapses the canonical record (there is no separate raw stream), which destroys forensic
accuracy and is widely criticized (see validation report §3.1 sources [9][16]). wirerust
preserves every raw `Finding`; the collapse is strictly a terminal-display lens.

### Flat Mode Only for v0.8.0 (STORY-118)

Collapse applies only when `show_mitre_grouping = false` (the default flat mode).

When `--mitre` is active, findings are already organized into tactic buckets by
`render_findings_grouped`. Applying collapse within each bucket requires a non-trivial
interaction with the tactic-sort path (`BC-2.11.014`). For v0.8.0 scope control, grouped
mode is excluded from collapse: the `render_findings_grouped` path renders each finding
individually as today, regardless of the `collapse_findings` flag.

This is a deliberate scope boundary, not a design principle. When both
`show_mitre_grouping = true` and `collapse_findings = true`, collapse silently does not
apply to grouped output. Grouped-mode collapse is deferred to STORY-119 in a follow-on
cycle.

### Binding Rule

> **Rule 4 (display-layer aggregation):** Aggregation of repeated findings for human
> readability belongs in `TerminalReporter` only. JSON and CSV consumers MUST receive
> the complete, unaggregated `&[Finding]` slice. No aggregation pass MAY be applied
> upstream of the multi-reporter dispatch in `main.rs`. Any future reporter that wants
> its own aggregation implements it privately at its own render boundary.

### Alternatives Considered

**Opt-in (`--collapse` flag, default-off):** Preserves backward compatibility for any
script that counts terminal output lines. Rejected because the flooding problem persists
until operators discover the flag, and terminal output is not a machine-readable contract.
The F1 gate confirmed this as the most consequential UX choice and locked default-on.

**Threshold (`--collapse-threshold N`, collapse only when count ≥ N):** Adds a
configurable minimum repeat count before collapsing. Rejected because singletons already
render without a count suffix under the always-collapse design, making the effective
behavior identical to a threshold of 1, and the flag adds CLI surface without benefit.
The F1 gate locked always-collapse (no threshold).

**Collapse within each MITRE tactic bucket (`--mitre` + collapse):** Feasible but
non-trivial — requires a collapse pass after bucketing and sorting but before rendering
each bucket. Deferred to STORY-119 per F1 gate OQ-3 resolution (flat mode only for
v0.8.0).

### Precedents

This decision is directly validated by the analysis-time aggregation pattern in mature
network-security tooling, documented in
`.factory/research/issue-259-finding-collapse-validation.md` (Wireshark Expert Information
[sources 7, 8, 15], ntopng Alerts Explorer [13, 14], Splunk `dedup` [10], SIEM aggregation
stage [17]). The syslog "last message repeated N times" pattern [9, 16] is the explicit
counter-example — it collapses the canonical record rather than providing a display-layer
lens, and is widely criticized and routinely disabled. wirerust's design matches the
Wireshark model and avoids the syslog anti-pattern.

---

## Render-Mode Enum (Issue #62 — v0.9.0)

**Status of this subsection:** Accepted (F2 spec evolution 2026-06-17; F3 scope correction 2026-06-18)

### Context

v0.8.0 shipped `TerminalReporter` with four boolean fields:

```rust
pub struct TerminalReporter {
    pub use_color: bool,
    pub show_mitre_grouping: bool,
    pub show_hosts_breakdown: bool,
    pub collapse_findings: bool,
}
```

The issue #62 trigger condition ("when a 3rd render flag is added") fired when STORY-118
(issue #259) added `collapse_findings`. Two concrete illegal-state violations resulted:

1. **Nonsensical combination** (`show_mitre_grouping = true && collapse_findings = true`)
   is a representable struct value. The type permits it; the code silently ignores
   `collapse_findings` when `show_mitre_grouping` is true (dispatch-order enforcement only).
   The BC-2.11.025 invariant is encoded in comments and dispatch order, not the type system.

2. **Inert-value comment** at `main.rs` `run_summary`: `collapse_findings: true` with a
   comment explaining the value does not matter. The type cannot express "irrelevant in this
   context."

### Decision

**Replace `show_mitre_grouping: bool` and `collapse_findings: bool` with
`render: FindingsRender` — a three-variant enum that makes the mutually-exclusive
rendering modes unrepresentable as invalid combinations.**

```rust
/// Governs which rendering path the FINDINGS section uses.
///
/// Replaces `show_mitre_grouping: bool` + `collapse_findings: bool`
/// from v0.8.0. The previous struct admitted `show_mitre_grouping = true
/// && collapse_findings = true`, which was silently handled by dispatch
/// order but was never a valid state.
///
/// BC-2.11.013 (Grouped), BC-2.11.025–028 (FlatCollapsed), default (FlatExpanded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingsRender {
    /// Group findings by MITRE tactic (`--mitre` flag).
    /// Corresponds to the previous `show_mitre_grouping = true`.
    Grouped,
    /// Collapse repeated findings into counted groups (default, v0.8.0+).
    /// Corresponds to the previous `collapse_findings = true, show_mitre_grouping = false`.
    FlatCollapsed,
    /// One display line per raw finding (pre-v0.8.0 behavior, `--no-collapse`).
    /// Corresponds to the previous `collapse_findings = false, show_mitre_grouping = false`.
    FlatExpanded,
}

pub struct TerminalReporter {
    pub use_color: bool,
    pub show_hosts_breakdown: bool,
    pub render: FindingsRender,
}
```

### Rationale: Illegal-State Elimination is the Primary Driver

The decisive justification is **illegal-state elimination** — making the
`show_mitre_grouping = true && collapse_findings = true` combination unrepresentable at the
type level. Rust's `match` exhaustiveness then enforces that every call site handles all
three modes explicitly, replacing the fragile `if/else if` dispatch chain.

The Clippy `fn_params_excessive_bools` lint (`max-fn-params-bools` default: `3`) provides
corroborating tooling consensus (the current four bools exceed the machine-enforced
threshold) but is not the primary driver. The illegal-state argument is decisive on its own:
two mutually-exclusive bools encode `2^2 = 4` representable states, of which only 3 are
valid. The enum encodes exactly 3 states and no others.

This is confirmed by external research (`.factory/research/issue-62-enum-modes-design-validation.md`):
- Rust API Guidelines C-NEWTYPE recommends deliberate types when invariants exist.
- "Parse, don't validate" (Alexis King): enums over bool-pairs when combinations matter.
- Clippy's machine-enforced default threshold verified directly against rust-lang.org docs.

### Why Orthogonal Flags Stay as Bools

`use_color` and `show_hosts_breakdown` are **orthogonal** — all four combinations are valid
and meaningful:

- `use_color` applies uniformly across every output section (headers, findings, warnings).
  It is controlled by `--no-color` and terminal detection, independent of how findings are
  grouped. It is not part of the findings-render axis.
- `show_hosts_breakdown` gates the HOSTS section, which is rendered before the FINDINGS
  section and is independent of it. It is used by the `summary` subcommand, which never
  renders a FINDINGS section at all — making `FindingsRender` irrelevant for that path.

Folding either into `FindingsRender` would be a category error: it would create combinations
that are semantically incoherent (e.g., `FindingsRender::GroupedWithColor`) and would
introduce new illegal states rather than eliminating them.

The hybrid design — one enum for the mutually-exclusive axis, two bools for orthogonal
toggles — is confirmed as the idiomatic Rust recommendation by research Q3: "Keep genuinely
orthogonal booleans as separate, clearly-named fields while extracting only the
mutually-exclusive axis into an enum."

### Migration Map

Every construction site translates old bool pairs to enum variants as follows:

| Old fields | New field | Notes |
|-----------|-----------|-------|
| `show_mitre_grouping: true, collapse_findings: false` | `render: FindingsRender::Grouped` | — |
| `show_mitre_grouping: true, collapse_findings: true` | `render: FindingsRender::Grouped` | Was previously nonsensical; grouped wins per dispatch order |
| `show_mitre_grouping: false, collapse_findings: true` | `render: FindingsRender::FlatCollapsed` | — |
| `show_mitre_grouping: false, collapse_findings: false` | `render: FindingsRender::FlatExpanded` | Pre-v0.8.0 behavior |

The `--mitre` / `--no-collapse` → bool resolution stays at the `main()` call site
(`src/main.rs` lines 79-80), unchanged by this refactor:

```rust
// main() call site — unchanged:
*mitre,                              // → show_mitre_grouping: bool
collapse_findings_from_flag(*no_collapse),  // → collapse_findings: bool
```

Inside `run_analyze`, the in-scope parameters are `show_mitre_grouping: bool` and
`collapse_findings: bool` (function signature `src/main.rs` lines 107-108).
The `run_analyze` signature is unchanged. The bool → enum translation at the
`TerminalReporter` construction site (`src/main.rs` ~line 373) becomes:

```rust
// at the TerminalReporter construction site inside run_analyze;
// show_mitre_grouping and collapse_findings are the in-scope params:
render: if show_mitre_grouping {
    FindingsRender::Grouped
} else if collapse_findings {
    FindingsRender::FlatCollapsed
} else {
    FindingsRender::FlatExpanded
},
```

`collapse_findings_from_flag` is unchanged. `show_mitre_grouping` is `true` exactly
when `*mitre` is `true`; `collapse_findings` is `true` exactly when `!no_collapse`
is `true`. The observable behavior is identical to the migration table above.

The `run_summary` inert-value site (`collapse_findings: true` with comment) becomes
`render: FindingsRender::FlatCollapsed` — structurally expressing "if this reporter
were ever used to render findings, it would use the v0.8.0 default."

### Semver Consequence: v0.8.x → v0.9.0

Removing the public fields `show_mitre_grouping` and `collapse_findings` and adding the
public field `render: FindingsRender` is a **breaking change** to the public struct API.
Under Cargo's SemVer model and RFC 1105, removing or replacing a reachable public field is
classified as a major (breaking) change. For a `0.y.z` crate, the `y` component is the
breaking component; `0.8.x → 0.9.0` is therefore the correct and required version bump.

This is confirmed by research Q4 (verified directly against `doc.rust-lang.org/cargo/
reference/semver.html` and RFC 1105). The caret specifier `"0.8.x"` in `Cargo.toml`
resolves to `>=0.8.x, <0.9.0`, so consumers pinned in the 0.8.x line will not auto-receive
0.9.0 — the intended containment behavior for a breaking change.

The `cargo-semver-checks` `struct_field_missing` lint will fire as expected when run against
the 0.8.x baseline. This is correct, not a defect. The recommendation from research is to
run `cargo-semver-checks` in the release flow to make the classification machine-visible.

### `Default` Derive: Deliberate Omission

`Default` is **NOT derived** on `FindingsRender`. Rationale:

RFC 3107 permits `#[derive(Default)]` with `#[default]` on a unit variant. The natural
candidate would be `FlatCollapsed` (matching today's default `analyze` behavior). However:

- Deriving `Default` makes the default variant part of the public stability commitment.
  Changing it post-0.9.0 would be a silent behavioral break not caught by the compiler or
  `cargo-semver-checks`.
- The current codebase has exactly two construction paths (`run_analyze` and `run_summary`),
  both of which set `render` explicitly. There is no site that would benefit from a
  `Default::default()` call — all sites carry enough context to pick the correct variant.
- Explicit construction is preferable here: `render: FindingsRender::FlatCollapsed` at each
  site documents the intent, whereas `Default::default()` would obscure which variant is
  being selected.

If a future caller needs a default (e.g., a test helper builder pattern), `Default` can be
added then as a documented, deliberate API commitment. It is backwards-compatible to add
`Default` in a later minor release; it is not backwards-compatible to change the default
variant after the fact.

### Binding Rule

> **Rule 5 (render-mode type):** `TerminalReporter`'s findings rendering mode MUST be
> expressed as `FindingsRender`. Adding a new rendering mode requires adding a new variant
> to `FindingsRender` and updating all exhaustive `match` arms. A bool field on
> `TerminalReporter` that encodes a mutually-exclusive rendering mode is prohibited; such a
> field MUST be folded into `FindingsRender` or a successor enum. Orthogonal toggles
> (properties that do not create illegal states when combined with existing fields) MAY
> remain as bool fields.

### Alternatives Considered

**Pre-split for STORY-119 (grouped-mode collapse):** Add `GroupedCollapsed` and
`GroupedExpanded` variants now, anticipating a future feature. Rejected as YAGNI — the
STORY-119 cycle will have its own F1/F2 and can amend the enum at that time. The current
three-variant enum exactly models the current three modes with no phantom states.

**Builder / typestate pattern:** A `TerminalReporterBuilder` with typestate enforcement.
Warranted for multi-step construction with cross-field invariants. Rejected: `TerminalReporter`
construction is one-shot mode selection with no sequenced protocol. A plain enum is the
correct, minimal tool.

**Remain as bools, add documentation only:** The existing dispatch-order invariant is already
documented in comments and BCs. Rejected: the illegal state is still constructible; the
compiler provides no enforcement; new contributors can silently violate the invariant. The
type-system fix is strictly superior.

---

## Validation

This decision was validated through targeted Perplexity queries on 2026-04-08:

- **Output encoding placement:** OWASP guidance (XSS prevention cheat sheet, CWE-117 log injection) recommends encoding at display time, not at storage time. Encoding at construction creates premature commitment to one output context and breaks others. Confirmed.
- **`serde_json` control byte handling:** `serde_json` automatically escapes control bytes (including ESC `0x1b`) as `\u001b`, per RFC 8259 §7. JSON output is safe with no analyzer code. Confirmed.
- **Escape primitive selection:** An initial Perplexity answer claimed `str::escape_default` preserves multi-byte UTF-8. A follow-up empirical check (`rustc`-compiled program, 2026-04-09) falsified this: `str::escape_default` internally routes every character through `char::escape_default`, which escapes *all* non-ASCII Unicode as `\u{...}`. A custom helper iterating `chars()` and gating `escape_default` on `is_ascii_control() || '\\'` was then verified empirically against the same test inputs (ESC, BEL, DEL, backslash, Cyrillic, emoji, mixed content, tabs/newlines/CR) and produced the correct output: control bytes escaped as `\u{<hex>}`, backslash doubled, Cyrillic and emoji preserved byte-for-byte.
- **C1 control codepoint risk:** C1 codepoints (U+0080–U+009F) CAN appear as valid UTF-8 in a `String` — encoded as two-byte sequences (e.g., 0xC2 0x9B for U+009B, the 8-bit CSI). The earlier claim that "a standalone byte in the 0x80–0x9f range cannot appear" was correct for a single raw byte but conflates the distinction between byte and codepoint. The helper now explicitly escapes the C1 range via a range check, in addition to C0 + DEL via `char::is_ascii_control()`. Empirical verification (2026-04-09) confirmed that a Cyrillic SNI containing 0xC2 0x9B bytes (U+009B) produces a String whose char iteration yields U+009B unescaped by the old predicate; the fix adds a range check to catch it.
