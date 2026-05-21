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
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.012.md
input-hash: "529c948"
traces_to: .factory/stories/STORY-086.md
id: "HS-097"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.012
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

# Holdout Scenario: Non-Existent Target Path Produces Descriptive Error Message with Path Included

## Scenario

When an analyst accidentally specifies a target path that does not exist, the tool must
provide a clear, actionable error message that includes the offending path — so the analyst
can immediately see what was wrong.

1. The tool is invoked with a path to a file that does not exist:
   `wirerust analyze /tmp/nonexistent_capture_file.pcap`
2. The tool exits with a non-zero exit code.
3. The error message includes the text `"Target not found:"` followed by the path.
4. The path `/tmp/nonexistent_capture_file.pcap` appears verbatim in the error output.
5. No analysis is attempted; no partial output is produced.

Separately, for directory target validation:
6. A directory path that exists but contains no `.pcap` files produces an empty-result run
   (not an error about the directory not existing).
7. A target path that exists but is neither a file nor a directory (e.g., a broken symlink)
   may produce an error or be treated as "not found" — the key requirement is that the tool
   does not panic.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.012 | Postcondition 1: non-existent path → Err with "Target not found: <path>" | Items 1-5: error message format |
| BC-2.12.011 | Postcondition 5: empty directory → Ok(vec[]) not an error | Item 6: empty dir is not "not found" |

## Verification Approach

**Non-existent file:**
Run `wirerust analyze /tmp/wirerust_hs097_nonexistent.pcap` (ensure this path truly does not exist).
Assert exit code != 0.
Assert stderr contains `"Target not found:"`.
Assert stderr contains `"/tmp/wirerust_hs097_nonexistent.pcap"` (the full path).

**Empty directory:**
Create a temp directory with no `.pcap` files. Run `wirerust analyze <tempdir>`.
Assert: the tool does not exit with "not found" error; it proceeds with no files to process.
Assert: the output (if any) indicates zero packets processed or produces a minimal report.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Non-existent path produces the exact error format; empty directory produces empty-result run.
- **Edge case handling** (weight: 0.2): The specific path appears in the error message; not just a generic "file not found".
- **Error quality** (weight: 0.2): Error message is on stderr; exit code is non-zero; no panic.
- **Performance** (weight: 0.05): Immediate error; no filesystem scanning of non-existent path.
- **Data integrity** (weight: 0.05): Error message correctly identifies which target was not found when multiple targets are given.

## Edge Conditions

- Multiple targets given; only one does not exist: error identifies the specific missing path.
- A directory target that does not exist: same "Target not found:" error with the directory path.
- A file path with spaces or special characters: still quoted or reported correctly.
- A file that exists but is not a valid pcap: this is different — the file IS found; the decode error surfaces later.

## Failure Guidance

"HOLDOUT LOW: HS-097 (satisfaction: 0.XX) -- Non-existent target did not produce 'Target not found: <path>' error with the path included, or the tool panicked instead of producing a clean error message."
