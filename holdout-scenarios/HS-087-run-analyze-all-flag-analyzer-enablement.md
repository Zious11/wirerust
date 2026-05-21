---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.008.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.009.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.010.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md
input-hash: "529c948"
traces_to: .factory/stories/STORY-086.md
id: "HS-087"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.008
  - BC-2.12.009
  - BC-2.12.010
  - BC-2.12.011
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: --all Enables All Three Analyzers; --no-reassemble Produces Warning and Skips HTTP/TLS

## Scenario

The `--all` flag is a convenience shorthand that should activate DNS, HTTP, and TLS analysis
simultaneously. The `--no-reassemble` flag, when combined with stream-based analyzers, must
produce an explicit warning so the analyst understands that stream analysis will be skipped.

**Part A — --all flag:**
1. The tool is invoked with `wirerust analyze --all <pcap>`.
2. The output contains sections from DNS, HTTP, and TLS analyzers (or their absence is
   reported if no relevant traffic was found).
3. The `--mitre` flag is NOT implied by `--all`; no MITRE grouping section headers appear
   unless `--mitre` is also given.

**Part B — --no-reassemble with --http warning:**
1. The tool is invoked with `wirerust --no-reassemble analyze --http <pcap>`.
2. A warning appears on stderr: the message mentions that HTTP/TLS require TCP reassembly
   but --no-reassemble is set, and that stream analysis will be skipped.
3. The warning appears exactly once (not repeated per packet or per file).
4. DNS analysis proceeds normally if `--dns` was also given.
5. The main output (stdout) does not contain HTTP analyzer results.

**Part C — directory expansion:**
1. The tool is invoked with a directory path as the target.
2. Only `.pcap` files in that directory are processed (not `.pcapng`, not files in subdirs).
3. Files are processed in lexicographic order.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.008 | Postcondition 1: --all enables dns, http, tls | Part A: all three analyzers active |
| BC-2.12.008 | Invariant 3: --mitre not in --all expansion | Part A: no MITRE headers without explicit --mitre |
| BC-2.12.009 | Postcondition 5: warning on --no-reassemble + HTTP/TLS | Part B: exact warning text |
| BC-2.12.009 | Postcondition 4: HTTP/TLS not constructed when no-reassemble | Part B: no HTTP output |
| BC-2.12.009 | Postcondition 6: DNS independent of reassembly | Part B: DNS proceeds even with --no-reassemble |
| BC-2.12.011 | Postcondition 1-2: .pcap only, sorted | Part C: directory expansion behavior |

## Verification Approach

**Part A:**
Run `wirerust analyze --all <pcap-with-mixed-traffic>`. In the output, look for:
- DNS statistics section or mention of DNS packet counts.
- HTTP request/response section.
- TLS/JA3 section.
- Assert no MITRE tactic section headers unless --mitre is added.

**Part B:**
Run `wirerust --no-reassemble analyze --http --dns <pcap>` and capture both stdout and stderr separately.
Assert: stderr contains the phrase "no-reassemble" and "stream analysis" (or similar) exactly once.
Assert: stdout does not contain HTTP analyzer results.
Assert: stdout contains DNS results (if any DNS traffic in pcap).

**Part C:**
Create a temp dir with `a.pcap`, `b.pcapng`, `sub/c.pcap`. Run `wirerust analyze <tempdir>`.
Assert: only `a.pcap` is processed; `b.pcapng` and `sub/c.pcap` are not.
Assert: processing order is alphabetical.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): All three parts work as described.
- **Edge case handling** (weight: 0.2): --mitre exclusion from --all; directory non-recursive.
- **Error quality** (weight: 0.2): Warning appears exactly once, on stderr, with correct content.
- **Performance** (weight: 0.05): Directory expansion completes promptly.
- **Data integrity** (weight: 0.1): Files processed in lexicographic order.

## Edge Conditions

- `--no-reassemble` without `--http`/`--tls`: no warning is emitted.
- Directory with no `.pcap` files: tool proceeds with empty file list (no error).
- `.PCAP` (uppercase): excluded from directory expansion (case-sensitive check).

## Failure Guidance

"HOLDOUT LOW: HS-087 (satisfaction: 0.XX) -- --all did not enable all three analyzers, the --no-reassemble warning was missing or appeared multiple times, or directory expansion included non-.pcap files."
