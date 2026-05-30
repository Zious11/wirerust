---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.11.007
bcs:
  - BC-2.11.007
  - BC-2.11.008
  - BC-2.11.009
  - BC-2.11.010
  - BC-2.11.011
  - BC-2.11.012
module: src/reporter/terminal.rs
proof_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: Wave-21 wave-level consistency lens — SS-11 reporter VP proof-method family harmonization (DF-SIBLING-SWEEP-001; sibling of the 2026-05-30 VP-020 correction): VP-012 frontmatter proof_method=proptest confirmed authoritative (Unicode input space is unbounded; proptest rationale preserved); consuming BC VP-table rows corrected from unit: to proptest to achieve 3-layer agreement — 2026-05-30"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-012: escape_for_terminal Correctness

## Property Statement

The `escape_for_terminal(s: &str) -> String` function satisfies:

1. **No dangerous bytes survive:** The output string contains no raw bytes
   in the C0 control range (U+0000-U+001F, including ESC U+001B), no DEL
   (U+007F), no C1 control codepoints (U+0080-U+009F), and no backslash
   (U+005C). All such characters are replaced with their `\u{...}` or `\n`,
   `\t`, etc. escape forms.

2. **Printable ASCII survives unchanged:** All characters in U+0020-U+007E
   (except backslash U+005C) pass through unmodified.

3. **Valid non-ASCII Unicode > U+009F survives unchanged:** Cyrillic, CJK,
   emoji, and other non-ASCII Unicode characters that are not in the C1
   range pass through as raw UTF-8 bytes.

4. **Output is always valid UTF-8.** The returned String is well-formed.

5. **Backslash is escaped** (doubled or escaped as `\\`) so that the
   output does not contain raw backslash bytes.

## Source Contract

- **Primary BC:** BC-2.11.007 -- TerminalReporter Escapes C0+DEL+C1+Backslash in Summary and Evidence
- **Postcondition:** No C0/DEL/C1 or backslash in output; all other characters pass through
- **Invariant:** INV-4 / ADR 0003 (display-layer separation)
- **Related BC:** BC-2.11.008 -- TerminalReporter Escape Preserves Printable ASCII and UTF-8
- **Related BC:** BC-2.11.009 -- TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved
- **Related BC:** BC-2.11.010 -- TerminalReporter Escapes Both Summary AND Each Evidence Line
- **Related BC:** BC-2.11.011 -- TerminalReporter Escapes Analyzer-Summary Detail Values
- **Related BC:** BC-2.11.012 -- TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary String inputs | Full Unicode space including multi-byte sequences, C1 codepoints, mixed content |

proptest is preferred over Kani here because the input space is the full Unicode
string space and the helper iterates over `chars()`. Kani would require a very
large unwind bound for typical string lengths.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn prop_no_dangerous_bytes_survive(s: String) {
            let escaped = escape_for_terminal(&s);
            for c in escaped.chars() {
                // No C0 (0x00-0x1F)
                prop_assert!(!c.is_ascii_control(),
                    "C0 control char U+{:04X} survived", c as u32);
                // No C1 (0x80-0x9F)
                prop_assert!(!(('\u{80}'..='\u{9f}').contains(&c)),
                    "C1 control char U+{:04X} survived", c as u32);
                // No backslash
                prop_assert!(c != '\\',
                    "raw backslash survived");
            }
        }

        #[test]
        fn prop_printable_ascii_unchanged(s: String) {
            // Build a string of only printable ASCII (excluding backslash)
            let ascii_only: String = s.chars()
                .filter(|c| c.is_ascii() && !c.is_ascii_control() && *c != '\\')
                .collect();
            let escaped = escape_for_terminal(&ascii_only);
            prop_assert_eq!(escaped, ascii_only,
                "printable ASCII was modified");
        }

        #[test]
        fn prop_non_ascii_unicode_above_c1_unchanged(s: String) {
            // Filter to only non-ASCII chars above U+009F (safe Unicode)
            let unicode_only: String = s.chars()
                .filter(|c| !c.is_ascii() && *c > '\u{9f}')
                .collect();
            let escaped = escape_for_terminal(&unicode_only);
            prop_assert_eq!(escaped, unicode_only,
                "safe non-ASCII Unicode was escaped");
        }

        #[test]
        fn prop_output_is_valid_utf8(s: String) {
            let escaped = escape_for_terminal(&s);
            // String is always valid UTF-8 by type; this is a runtime assertion
            prop_assert!(std::str::from_utf8(escaped.as_bytes()).is_ok());
        }
    }

    // Fixed regression tests (from ADR 0003 validation)
    #[test]
    fn esc_byte_is_escaped() {
        let s = "\x1b[31mRED\x1b[0m";
        let out = escape_for_terminal(s);
        assert!(!out.contains('\x1b'), "ESC survived in output: {}", out);
        assert!(out.contains("\\u{1b}") || out.contains("\\x1b"),
            "ESC not escaped as expected: {}", out);
    }

    #[test]
    fn cyrillic_preserved() {
        let s = "\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}"; // "primer" in Cyrillic
        let out = escape_for_terminal(s);
        assert_eq!(out, s, "Cyrillic was modified");
    }

    #[test]
    fn c1_csi_escaped() {
        // U+009B (CSI) encoded as UTF-8: 0xC2 0x9B
        let s = "\u{9b}";
        let out = escape_for_terminal(s);
        assert!(!out.contains('\u{9b}'), "CSI survived: {}", out);
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded | Full String space; proptest handles with shrinking |
| Proof complexity | Low | Single pass over chars(); simple predicate checks |
| Tool support | High | escape_for_terminal is a private pure helper; exported for testing |
| Estimated proof time | < 30 seconds for 1000 cases | Fast character iteration |

## Source Location

`src/reporter/terminal.rs` -- private `escape_for_terminal(s: &str) -> String` helper.
Applied to `f.summary` and each `ev` in `f.evidence` when rendering findings.
Also applied to analyzer summary detail values (BC-2.11.011).

Empirically validated 2026-04-09 per ADR 0003.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
