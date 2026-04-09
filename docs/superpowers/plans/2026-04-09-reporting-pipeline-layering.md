# Reporting Pipeline Layering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move finding output sanitization from the analyzer construction site to the terminal reporter, per ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`), fixing terminal injection in HTTP findings and restoring forensic data preservation in the `Finding` struct.

**Architecture:** The `Finding` struct stores raw post-`from_utf8_lossy` bytes; the terminal reporter escapes via a custom helper (~15 lines, built on stdlib `char::escape_default` gated by ASCII control / C1 control / backslash detection) immediately before writing. The JSON reporter is already safe via `serde_json`'s automatic RFC 8259 control-byte escaping. No new dependencies, no new types.

**Important mechanism note:** `str::escape_default` was the initial choice but was rejected during plan self-review after empirical verification showed it escapes *all* non-ASCII characters (Cyrillic, emoji) — same UX problem as the PR #49 Debug formatter. See ADR 0003's updated Mechanism section and commit `45cf649`. The custom helper iterates `s.chars()` and only escapes when the character is an ASCII control (C0 + DEL), a C1 control (`U+0080..=U+009F`), or `'\\'`. The C1 range was added in commit `7d2cd3c` after the initial round of plan execution; the flow described in Task 3 below matches the pre-C1 predicate for historical accuracy — see commit `7d2cd3c` for the final predicate.

**Tech Stack:** Rust 2024 edition, stdlib only for the escape primitive, `serde_json` for JSON output (already in tree), `owo_colors` for terminal styling (already in tree).

---

## File Structure

| File | Role | Change |
|------|------|--------|
| `docs/adr/0003-reporting-pipeline-layering.md` | The ADR | Already committed in `f4c6098`. No change in this plan. |
| `docs/adr/0002-modular-protocol-analyzers.md` | Analyzer pattern ADR | Add one cross-reference line pointing to ADR 0003 |
| `src/reporter/terminal.rs` | Terminal reporter | Add private `escape_for_terminal` helper + unit tests; apply to `f.summary` and each `ev` in the rendering loop |
| `src/findings.rs` | `Finding` struct | Add doc comment on `impl Display for Finding` warning that it produces raw text |
| `src/analyzer/tls.rs` | TLS analyzer | Replace `{hostname:?}` and `{lossy:?}` with `{hostname}` / `{lossy}`; update inline comments to reference ADR 0003 |
| `tests/tls_analyzer_tests.rs` | TLS tests | Update `test_non_utf8_sni_escapes_control_bytes_in_summary` to assert the RAW ESC byte is now preserved in `f.summary` (contract inversion) |
| `tests/reporter_tests.rs` | Reporter tests | Add end-to-end regression tests: terminal reporter escapes ESC bytes; JSON reporter preserves via serde's `\u001b`; `Finding.summary` keeps the raw byte |

No new code or runtime files are created as part of this implementation work. All implementation changes are edits or test additions inside existing files. (This plan document and `docs/adr/0003-reporting-pipeline-layering.md` are themselves new files committed as prerequisites before the implementation tasks begin.)

---

## Task 1: Cross-reference ADR 0003 from ADR 0002

**Files:**
- Modify: `docs/adr/0002-modular-protocol-analyzers.md`

- [ ] **Step 1: Locate the "Finding Generation Guidelines" section**

Read `docs/adr/0002-modular-protocol-analyzers.md` and find the "Finding Generation Guidelines" section (around line 103-109 in the current file — it's the bulleted list starting with "Generate findings only for unambiguous security concerns").

- [ ] **Step 2: Add cross-reference bullet**

Add one new bullet at the end of the guidelines list:

```markdown
- **Output sanitization is a reporter responsibility, not an analyzer responsibility.** Store raw bytes (post-`from_utf8_lossy`) in `Finding.summary` and `Finding.evidence`. Do not escape, debug-format, or otherwise pre-encode untrusted bytes at the analyzer. See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the full layering principle.
```

- [ ] **Step 3: Commit**

```bash
git add docs/adr/0002-modular-protocol-analyzers.md
git commit -m "docs(adr): cross-reference ADR 0003 from ADR 0002

Point analyzer authors at the layering principle so they know output
sanitization is the reporter's responsibility, not theirs."
```

---

## Task 2: Add Finding::Display doc warning

**Files:**
- Modify: `src/findings.rs:72`

- [ ] **Step 1: Locate `impl fmt::Display for Finding`**

Open `src/findings.rs`. The impl block starts at line 72.

- [ ] **Step 2: Add doc comment above the impl block**

Insert these lines directly above `impl fmt::Display for Finding {`:

```rust
/// Produces the raw text representation of a finding for logging, debugging,
/// or machine-readable output. **Not safe for direct terminal display** — the
/// `summary` field may contain attacker-controlled bytes from packet payloads
/// (including ASCII control codes like ESC `0x1b`) that a terminal would
/// interpret as control sequences. For safe terminal rendering, use the
/// terminal reporter (`src/reporter/terminal.rs`), which applies its
/// `escape_for_terminal` helper to every `summary` and `evidence` entry
/// before writing to the output buffer. See ADR 0003
/// (`docs/adr/0003-reporting-pipeline-layering.md`) for the full layering
/// principle.
```

- [ ] **Step 3: Run existing `test_finding_display`**

```bash
cargo test --test findings_tests test_finding_display
```

Expected: PASS (no behavioral change; this is a doc-only edit).

- [ ] **Step 4: Commit**

```bash
git add src/findings.rs
git commit -m "docs(findings): warn that Finding::Display is not terminal-safe

Doc comment on the impl Display block explaining that the raw text
representation is for logging/debugging only and may contain
attacker-controlled control bytes. Terminal rendering should go
through the terminal reporter, which escapes per ADR 0003."
```

---

## Task 3: Add `escape_for_terminal` helper with unit tests

**Files:**
- Modify: `src/reporter/terminal.rs` (add helper and `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing unit tests**

At the bottom of `src/reporter/terminal.rs`, add a `#[cfg(test)] mod tests` block. **Note on expected output format:** `char::escape_default` uses `\u{<hex>}` (minimal hex, no leading zeros) for control bytes without a short escape — so ESC 0x1b renders as `\u{1b}`, BEL 0x07 renders as `\u{7}`, DEL 0x7f renders as `\u{7f}`. It uses the short forms `\n`, `\r`, `\t` for newline/CR/tab, and `\\` for backslash. These expectations were verified empirically (see commit `45cf649`).

```rust
#[cfg(test)]
mod tests {
    use super::escape_for_terminal;

    #[test]
    fn escapes_esc_byte() {
        assert_eq!(
            escape_for_terminal("\x1b[31mRED\x1b[0m"),
            "\\u{1b}[31mRED\\u{1b}[0m"
        );
    }

    #[test]
    fn escapes_bel_and_del() {
        assert_eq!(
            escape_for_terminal("ring\x07bye\x7f"),
            "ring\\u{7}bye\\u{7f}"
        );
    }

    #[test]
    fn escapes_tab_newline_cr_as_short_forms() {
        // char::escape_default uses short escapes for these three.
        assert_eq!(
            escape_for_terminal("tab\there\nnewline\rreturn"),
            "tab\\there\\nnewline\\rreturn"
        );
    }

    #[test]
    fn escapes_backslash() {
        assert_eq!(escape_for_terminal("a\\b"), "a\\\\b");
    }

    #[test]
    fn preserves_printable_ascii() {
        assert_eq!(
            escape_for_terminal("hello world 123 !@#"),
            "hello world 123 !@#"
        );
    }

    #[test]
    fn preserves_cyrillic() {
        assert_eq!(escape_for_terminal("пример.рф"), "пример.рф");
    }

    #[test]
    fn preserves_emoji() {
        assert_eq!(escape_for_terminal("crab 🦀 rust"), "crab 🦀 rust");
    }

    #[test]
    fn mixed_content_escapes_only_dangerous_bytes() {
        // Cyrillic + ESC injection + emoji — Cyrillic and emoji must survive,
        // only the ESC sequence should be escaped.
        assert_eq!(
            escape_for_terminal("пример\x1b[31m🦀"),
            "пример\\u{1b}[31m🦀"
        );
    }

    #[test]
    fn empty_string_is_empty() {
        assert_eq!(escape_for_terminal(""), "");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --lib reporter::terminal::tests 2>&1 | tail -20
```

Expected: compile error — `escape_for_terminal` not in scope. This confirms the tests are wired up and the function doesn't exist yet. (Private `#[cfg(test)] mod tests` blocks inside `src/reporter/terminal.rs` are lib tests, not integration tests — use `--lib` alone without `--test`.)

- [ ] **Step 3: Implement `escape_for_terminal`**

At the top of `src/reporter/terminal.rs`, immediately after the `use` statements and before `pub struct TerminalReporter`, add:

```rust
/// Escape control bytes (C0 + DEL + backslash) for safe terminal display.
///
/// Iterates the input string's characters and applies `char::escape_default`
/// only when the character matches `char::is_ascii_control()` or is a
/// backslash. All other characters — printable ASCII and valid non-ASCII
/// Unicode (Cyrillic, CJK, emoji) — pass through unchanged.
///
/// Why not `str::escape_default`? It routes *every* character through
/// `char::escape_default`, which escapes non-ASCII as `\u{...}` and
/// would mangle a Cyrillic hostname like `пример.рф` into
/// `\u{43f}\u{440}...`. See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`)
/// for the layering rationale and the empirical verification.
fn escape_for_terminal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_control() || c == '\\' {
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

- [ ] **Step 4: Run the helper unit tests**

```bash
cargo test --lib reporter::terminal::tests 2>&1 | tail -20
```

Expected: all 9 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/reporter/terminal.rs
git commit -m "feat(reporter): add escape_for_terminal helper

Private helper in src/reporter/terminal.rs built on stdlib
char::escape_default, gated by char::is_ascii_control || backslash.
Escapes C0 control bytes, DEL, and backslash while preserving all
valid non-ASCII Unicode (Cyrillic, CJK, emoji). Unit tested for ESC,
BEL, DEL, tab/newline/CR short escapes, backslash, printable ASCII,
Cyrillic, emoji, mixed content, and empty string.

Not yet applied to the rendering loop — that's the next commit, so
the helper can land with dedicated tests before it's integrated.

See ADR 0003 for the layering rationale."
```

---

## Task 4: Apply helper in terminal reporter rendering loop

**Files:**
- Modify: `src/reporter/terminal.rs:63-82` (the `FINDINGS` rendering block)
- Modify: `tests/reporter_tests.rs` (add a failing test first)

- [ ] **Step 1: Write a failing test in `tests/reporter_tests.rs`**

Add this test at the end of `tests/reporter_tests.rs`:

```rust
#[test]
fn test_terminal_reporter_escapes_esc_bytes_in_summary() {
    // Regression: a Finding whose summary contains an ESC byte must not
    // propagate the raw byte to terminal output, where it would be
    // interpreted as an ANSI escape sequence. Per ADR 0003, the terminal
    // reporter is responsible for this escaping.
    let reporter = TerminalReporter { use_color: false };
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "attacker payload: \x1b[31mRED\x1b[0m".into(),
        evidence: vec!["raw evidence: \x1b[32mGREEN".into()],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    }];

    let output = reporter.render(&summary, &findings, &[]);

    assert!(
        !output.as_bytes().contains(&0x1b),
        "terminal output must not contain raw ESC (0x1b) bytes, got: {output:?}"
    );
    assert!(
        output.contains("\\u{1b}[31mRED"),
        "terminal output should contain escaped form of ESC sequence in summary, got: {output}"
    );
    assert!(
        output.contains("\\u{1b}[32mGREEN"),
        "terminal output should contain escaped form in evidence line, got: {output}"
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cargo test --test reporter_tests test_terminal_reporter_escapes_esc_bytes_in_summary 2>&1 | tail -15
```

Expected: FAIL — the assertion `!output.as_bytes().contains(&0x1b)` fails because the current renderer interpolates `f.summary` and `ev` raw.

- [ ] **Step 3: Wrap `f.summary` and each `ev` in the render loop**

In `src/reporter/terminal.rs`, find the findings rendering block (around lines 60-86). The current code reads:

```rust
        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            for f in findings {
                let line = format!(
                    "[{}] {} ({}) - {}",
                    f.category, f.verdict, f.confidence, f.summary
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
                    out.push_str(&format!("    > {ev}\n"));
                }
                if let Some(ref t) = f.mitre_technique {
                    out.push_str(&format!("    MITRE: {t}\n"));
                }
            }
            out.push('\n');
        }
```

Change it to wrap `f.summary` and each `ev` through `escape_for_terminal`:

```rust
        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            for f in findings {
                // Per ADR 0003: the Finding struct stores raw bytes; the
                // terminal reporter is responsible for escaping untrusted
                // content (summary + evidence) before writing to a TTY.
                let safe_summary = escape_for_terminal(&f.summary);
                let line = format!(
                    "[{}] {} ({}) - {}",
                    f.category, f.verdict, f.confidence, safe_summary
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
                    let safe_ev = escape_for_terminal(ev);
                    out.push_str(&format!("    > {safe_ev}\n"));
                }
                if let Some(ref t) = f.mitre_technique {
                    out.push_str(&format!("    MITRE: {t}\n"));
                }
            }
            out.push('\n');
        }
```

- [ ] **Step 4: Run the new test to verify it passes**

```bash
cargo test --test reporter_tests test_terminal_reporter_escapes_esc_bytes_in_summary 2>&1 | tail -10
```

Expected: PASS. Also run all reporter tests to make sure nothing else broke:

```bash
cargo test --test reporter_tests 2>&1 | tail -15
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/reporter/terminal.rs tests/reporter_tests.rs
git commit -m "feat(reporter): escape finding summary and evidence at terminal render

Apply escape_for_terminal to every Finding.summary and every evidence
entry in the terminal reporter's rendering loop. This moves output
sanitization from the analyzer construction site (where PR #49 placed
it) to the display layer, per ADR 0003.

JSON output remains unchanged and is already safe via serde_json's
automatic RFC 8259 escaping. HTTP findings (path traversal, web shell,
admin panel, unusual method, etc.) — previously vulnerable to terminal
injection via attacker-controlled URIs — are fixed by this change with
zero analyzer-side code modifications."
```

---

## Task 5: Roll back TLS Debug formatter escaping

**Files:**
- Modify: `tests/tls_analyzer_tests.rs:397-438` (invert `test_non_utf8_sni_escapes_control_bytes_in_summary`)
- Modify: `src/analyzer/tls.rs:343-376` (remove `{:?}` Debug-format interpolation)

- [ ] **Step 1: Invert the failing-contract test first**

Replace the body of `test_non_utf8_sni_escapes_control_bytes_in_summary` in `tests/tls_analyzer_tests.rs` (lines 396-438) with its new contract. Rename it to reflect the new meaning:

```rust
#[test]
fn test_non_utf8_sni_preserves_raw_bytes_in_summary() {
    // Per ADR 0003: the Finding struct is the data layer — it stores the
    // raw post-from_utf8_lossy bytes from the attacker's SNI, including
    // any ASCII control codes. Terminal-safety is the reporter's job, not
    // the analyzer's. This test enforces that contract: raw ESC must
    // survive to the struct; downstream rendering tests (in reporter
    // tests) verify the terminal reporter escapes it on display.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // 0xff makes from_utf8 fail; 0x1b [ 3 1 m is the ANSI "red" CSI sequence;
    // "pwnd" is the visible payload an attacker would inject.
    let raw_sni: &[u8] = &[0xff, 0x1b, b'[', b'3', b'1', b'm', b'p', b'w', b'n', b'd'];
    let record = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let findings = analyzer.findings();
    let f = findings
        .iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("expected non-UTF-8 SNI finding");

    // The summary MUST contain the raw ESC byte — the analyzer does not
    // escape. Forensic preservation is a load-bearing property of the
    // data layer (ADR 0003).
    assert!(
        f.summary.as_bytes().contains(&0x1b),
        "summary must preserve raw ESC byte for forensics, got: {:?}",
        f.summary
    );
    // And it must NOT contain the Debug-formatted escape form (which
    // would indicate a regression to construction-site escaping).
    assert!(
        !f.summary.contains("\\u{1b}"),
        "summary must not contain Debug-formatted escape (regression to construction-site), got: {}",
        f.summary
    );

    // Hex evidence is unchanged — that's the lossless record.
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("ff1b5b33316d70776e64")),
        "expected raw bytes in hex evidence, got: {:?}",
        f.evidence
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cargo test --test tls_analyzer_tests test_non_utf8_sni_preserves_raw_bytes_in_summary 2>&1 | tail -15
```

Expected: FAIL — the current TLS code still uses `{lossy:?}`, so `f.summary` contains `\u{1b}` (escaped) instead of the raw `0x1b` byte.

- [ ] **Step 3: Remove the Debug formatter in `src/analyzer/tls.rs`**

In `src/analyzer/tls.rs`, find the NonAsciiUtf8 match arm (around lines 338-356). The current code has:

```rust
                SniValue::NonAsciiUtf8 { hostname, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Use Debug formatter ({:?}) to escape any control codepoints
                        // that might survive UTF-8 decoding (e.g. U+0085 NEL); the
                        // hostname here is valid UTF-8 but printable-script content
                        // is not guaranteed.
                        summary: format!(
                            "TLS SNI contains non-ASCII characters (RFC 6066 requires \
                             A-labels per RFC 5890): {hostname:?}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: None,
                        source_ip: None,
                        timestamp: None,
                    });
                }
```

Replace with (remove the `:?` and update the comment):

```rust
                SniValue::NonAsciiUtf8 { hostname, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw hostname interpolation — the data layer stores raw
                        // bytes per ADR 0003. Terminal-safety (escaping control
                        // codes, etc.) is applied by the terminal reporter at
                        // render time, not here.
                        summary: format!(
                            "TLS SNI contains non-ASCII characters (RFC 6066 requires \
                             A-labels per RFC 5890): {hostname}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: None,
                        source_ip: None,
                        timestamp: None,
                    });
                }
```

Now find the NonUtf8 match arm (around lines 357-376) with the current code:

```rust
                SniValue::NonUtf8 { lossy, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Use Debug formatter ({:?}) to escape control bytes (e.g.
                        // ESC 0x1b) that String::from_utf8_lossy preserves but the
                        // analyst's terminal would interpret as ANSI control
                        // sequences. Without this an attacker could craft a
                        // malformed SNI like b"\x1b[31m..." that recolors or
                        // overwrites the rendered finding line.
                        summary: format!(
                            "TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy:?}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: None,
                        source_ip: None,
                        timestamp: None,
                    });
                }
```

Replace with (remove the `:?` and update the comment):

```rust
                SniValue::NonUtf8 { lossy, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw lossy interpolation — the data layer stores raw
                        // bytes (including any embedded ASCII control codes) per
                        // ADR 0003. The terminal reporter is responsible for
                        // escaping these for safe display; JSON output is already
                        // safe via serde_json's automatic RFC 8259 escaping.
                        summary: format!(
                            "TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: None,
                        source_ip: None,
                        timestamp: None,
                    });
                }
```

- [ ] **Step 4: Run the inverted test to verify it passes**

```bash
cargo test --test tls_analyzer_tests test_non_utf8_sni_preserves_raw_bytes_in_summary 2>&1 | tail -10
```

Expected: PASS. The raw ESC byte now survives to `f.summary`.

- [ ] **Step 5: Run the full TLS test suite to confirm no other tests broke**

```bash
cargo test --test tls_analyzer_tests 2>&1 | tail -20
```

Expected: all tests PASS. The non-ASCII tests (Cyrillic, emoji, café.example) don't assert on escaped form directly — they check for substrings like `"RFC 6066"` and `"non-ASCII characters"` which are still present in the summary. The change only affects how the user-controlled `{hostname}` / `{lossy}` portion is interpolated.

- [ ] **Step 6: Commit**

```bash
git add src/analyzer/tls.rs tests/tls_analyzer_tests.rs
git commit -m "fix(tls): roll back construction-site escaping of SNI summaries

Remove {:?} Debug-format interpolation in both NonAsciiUtf8 and NonUtf8
SNI finding arms. The data layer now stores raw post-from_utf8_lossy
bytes; the terminal reporter is responsible for escaping at render
time per ADR 0003.

Inverts test_non_utf8_sni_escapes_control_bytes_in_summary — renamed
to test_non_utf8_sni_preserves_raw_bytes_in_summary — to enforce the
new contract: raw ESC bytes MUST survive to Finding.summary. A
separate regression test (next commit) verifies the end-to-end
pipeline: terminal reporter escapes, JSON reporter preserves via
serde, struct holds the raw byte.

Behavioral change visible to JSON consumers: Cyrillic SNIs now appear
as readable Unicode (\"пример.рф\") instead of hex-escaped
placeholders (\"\\u{43f}\\u{440}...\")."
```

---

## Task 6: End-to-end regression test for the layering contract

**Files:**
- Modify: `tests/reporter_tests.rs` (add one integration test)

- [ ] **Step 1: Write the end-to-end test**

Add this test at the end of `tests/reporter_tests.rs`:

```rust
#[test]
fn test_output_sanitization_layering_contract() {
    // End-to-end contract test for ADR 0003. A single Finding flows through
    // the data layer and both reporters; all three assertions must hold:
    //   1. The struct itself keeps the raw ESC byte (forensic layer).
    //   2. The terminal reporter escapes the ESC byte (terminal display layer).
    //   3. The JSON reporter escapes via serde's RFC 8259 \u001b form (JSON layer).
    //
    // Any future regression that breaks one of these — e.g., re-introducing
    // construction-site escaping, removing the terminal reporter's helper,
    // or swapping to a JSON crate that doesn't escape control chars — will
    // fail this test.
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "attacker payload: \x1b[31mRED\x1b[0m".into(),
        evidence: vec!["ev: \x1b[32mGREEN".into()],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    };

    // Layer 1: the struct preserves the raw ESC byte (forensic ground truth).
    assert!(
        finding.summary.as_bytes().contains(&0x1b),
        "Finding.summary must preserve raw ESC for forensics"
    );
    assert!(
        finding.evidence[0].as_bytes().contains(&0x1b),
        "Finding.evidence must preserve raw ESC for forensics"
    );

    // Layer 2: terminal reporter escapes on display.
    let terminal_output = TerminalReporter { use_color: false }.render(
        &Summary::new(),
        std::slice::from_ref(&finding),
        &[],
    );
    assert!(
        !terminal_output.as_bytes().contains(&0x1b),
        "terminal reporter must not emit raw ESC bytes, got: {terminal_output:?}"
    );
    assert!(
        terminal_output.contains("\\u{1b}[31mRED"),
        "terminal reporter should emit the escaped summary form, got: {terminal_output}"
    );
    assert!(
        terminal_output.contains("\\u{1b}[32mGREEN"),
        "terminal reporter should emit the escaped evidence form, got: {terminal_output}"
    );

    // Layer 3: JSON reporter escapes via serde's RFC 8259 \u001b form.
    let json_output = JsonReporter.render(
        &Summary::new(),
        std::slice::from_ref(&finding),
        &[],
    );
    assert!(
        !json_output.as_bytes().contains(&0x1b),
        "JSON reporter must not emit raw ESC bytes, got: {json_output:?}"
    );
    assert!(
        json_output.contains("\\u001b"),
        "JSON reporter should serialize ESC as \\u001b per RFC 8259, got: {json_output}"
    );
    // Round-trip through serde_json::from_str: the deserialized summary
    // must match the original raw ESC byte. This proves the JSON escape
    // is reversible, which is what downstream tooling relies on.
    let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    let parsed_summary = parsed["findings"][0]["summary"].as_str().unwrap();
    assert_eq!(parsed_summary, finding.summary);
}
```

- [ ] **Step 2: Run the test**

```bash
cargo test --test reporter_tests test_output_sanitization_layering_contract 2>&1 | tail -15
```

Expected: PASS immediately — Tasks 3, 4, and 5 already established the behavior. This test codifies the contract so it can't silently regress.

- [ ] **Step 3: Commit**

```bash
git add tests/reporter_tests.rs
git commit -m "test(reporter): end-to-end contract test for ADR 0003 layering

Single Finding flows through the data layer and both reporters. Asserts:

  1. Finding struct preserves raw ESC byte (forensic ground truth).
  2. Terminal reporter escapes ESC to \\u{1b} on display.
  3. JSON reporter serializes ESC as \\u001b per RFC 8259.
  4. JSON round-trips back to the original raw byte (reversibility).

Any regression that re-introduces construction-site escaping, removes
the terminal reporter's helper, or changes the JSON path will fail
this test."
```

---

## Task 7: Full test suite and clippy

**Files:** none modified

- [ ] **Step 1: Run cargo test**

```bash
cargo test --all 2>&1 | tail -30
```

Expected: all tests PASS. Count total — should be approximately the same as before plus the new tests added in this plan (9 helper unit tests in `src/reporter/terminal.rs`, 1 new reporter test, 1 new end-to-end test, plus 1 inverted TLS test — net +11 new, +1 rewritten).

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

Expected: zero warnings. The new code should be idiomatic Rust.

- [ ] **Step 3: Run cargo fmt check (no changes)**

```bash
cargo fmt -- --check 2>&1 | tail -10
```

Expected: zero diff. If formatting drifted, run `cargo fmt` and include the changes in the next commit.

- [ ] **Step 4: No commit unless fmt flagged changes**

If Step 3 reported no changes, no commit is needed. If it did:

```bash
cargo fmt
git add -u
git commit -m "chore: cargo fmt"
```

---

## Summary of commits

After all tasks complete, the branch should have (on top of `develop`):

1. `f4c6098` docs(adr): add ADR 0003 — reporting pipeline layering *(already committed before plan execution)*
2. docs(adr): cross-reference ADR 0003 from ADR 0002 *(Task 1)*
3. docs(findings): warn that Finding::Display is not terminal-safe *(Task 2)*
4. feat(reporter): add escape_for_terminal helper *(Task 3)*
5. feat(reporter): escape finding summary and evidence at terminal render *(Task 4)*
6. fix(tls): roll back construction-site escaping of SNI summaries *(Task 5)*
7. test(reporter): end-to-end contract test for ADR 0003 layering *(Task 6)*
8. (optional) chore: cargo fmt *(Task 7, only if needed)*

Total: 6–7 commits beyond the ADR, ~100 lines of production-code change, ~120 lines of new/modified test code.

## Post-implementation

After all tasks complete and tests pass, proceed to:
- Multi-agent PR review (`/pr-review-toolkit:review-pr`)
- Apply review triage pattern
- Push branch + create PR + request Copilot review
- Address Copilot feedback
