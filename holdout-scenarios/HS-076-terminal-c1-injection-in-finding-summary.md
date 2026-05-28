---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-076.md
  - .factory/stories/STORY-077.md
  - .factory/stories/STORY-078.md
  - .factory/stories/STORY-079.md
  - .factory/stories/STORY-080.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.007.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.009.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.012.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-076"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.007
  - BC-2.11.009
  - BC-2.11.010
  - BC-2.11.012
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Terminal Output Contains No Raw C1 Control Bytes When Finding Summary Has Attacker-Injected CSI

## Scenario

A pcap file is processed that produces a finding whose summary string contains the two-byte
UTF-8 encoding of U+009B (CSI — Control Sequence Introducer), which is `\xc2\x9b`. This byte
sequence can trigger terminal emulator state changes that re-map subsequent keystrokes.

1. The tool is invoked with the terminal (default) output format against a pcap that yields
   at least one finding.
2. The finding summary field was crafted to contain U+009B (the C1 CSI codepoint, encoded
   as `\xc2\x9b` in UTF-8).
3. The tool produces output on stdout.
4. The raw bytes `\xc2\x9b` (UTF-8 C1 CSI) do NOT appear anywhere in the stdout output.
5. Instead, the CSI codepoint appears in escaped form (e.g., `\u{9b}`).
6. All other parts of the finding (ASCII summary text, verdict, category) appear correctly.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.007 | Postcondition 3: C1 bytes (0x80-0x9F) are replaced | The U+009B byte in the summary must not survive to stdout |
| BC-2.11.009 | Postcondition 1: U+0080-U+009F all escaped | CSI (U+009B) is inside the C1 range boundary |
| BC-2.11.010 | Postcondition 1: escape applied to Finding.summary | The summary field (not just evidence) is the injection vector here |
| BC-2.11.012 | Postcondition 1: end-to-end C1 CSI in path-traversal finding escaped | The full render pipeline is exercised |

## Verification Approach

Construct or obtain a pcap that produces a path-traversal finding. Inject U+009B into the
finding summary by using a crafted HTTP request path containing `\xc2\x9b` (UTF-8 C1 CSI).

Run:
```
wirerust analyze <crafted.pcap>
```

Capture stdout and scan the raw bytes:
- Assert that the byte sequence `\xc2\x9b` does NOT appear anywhere in stdout.
- Assert that the string `\u{9b}` DOES appear in stdout (the escaped form).
- Assert that other summary text surrounding the injected byte appears correctly.

If constructing a crafted pcap is impractical, test the reporter layer directly by constructing
a Finding with `summary = "path/../\u{9b}secret"` and invoking TerminalReporter::render,
then scanning the returned string for raw `\xc2\x9b`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Raw `\xc2\x9b` bytes are completely absent from output; escaped representation is present.
- **Edge case handling** (weight: 0.2): The character before and after U+009B in the summary appear unchanged.
- **Error quality** (weight: 0.1): No panic, no corrupted output — the tool exits cleanly.
- **Performance** (weight: 0.1): Output is produced in under 5 seconds.
- **Data integrity** (weight: 0.1): The rest of the finding's fields (verdict, category, evidence) are unaffected.

## Edge Conditions

- U+009B is exactly at the boundary of the C1 range (U+009F is the last escaped; U+00A0 is not escaped).
- The test should also verify that U+00A0 (NBSP), placed in the same summary, passes through unescaped — testing that the escape function does not over-escape.
- An empty evidence list should not cause a crash.

## Failure Guidance

"HOLDOUT LOW: HS-076 (satisfaction: 0.XX) -- Raw C1 CSI bytes from a crafted finding summary appeared unescaped in terminal output, leaving the terminal vulnerable to control-sequence injection."
