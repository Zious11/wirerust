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
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.005.md
input-hash: "529c948"
traces_to: .factory/stories/STORY-086.md
id: "HS-094"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Reassembly Threshold Flags Enforce Numeric Ranges at Parse Time

## Scenario

Reassembly threshold flags have documented numeric ranges enforced by clap's value_parser.
An analyst who passes an out-of-range value must receive an immediate parse error — not a
silent clamp or runtime crash.

1. `wirerust --overlap-threshold 256 analyze test.pcap` is rejected with a non-zero exit
   code. The value 256 is out of the 0-255 range for this flag.
2. `wirerust --overlap-threshold 255 analyze test.pcap` is accepted (255 is the maximum).
3. `wirerust --overlap-threshold 0 analyze test.pcap` is accepted (0 is valid).
4. `wirerust --reassembly-depth 10 analyze test.pcap` uses the explicit value 10.
5. When `--reassembly-depth` is absent, the default is 10 (can be verified via JSON output
   showing the configuration in use, or via code inspection).
6. When `--reassembly-memcap` is absent, the default is 1024.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.005 | Postcondition 6: --overlap-threshold rejected above 255 | Item 1: 256 is rejected |
| BC-2.12.005 | Postcondition 3: --reassembly-depth default is 10 | Item 5: default behavior |
| BC-2.12.005 | Postcondition 4: --reassembly-memcap default is 1024 | Item 6: default behavior |

## Verification Approach

**Range rejection:**
Run `wirerust --overlap-threshold 256 analyze test.pcap` in a subprocess.
Assert exit code != 0.
Assert stderr contains an error about the value being out of range.

Run `wirerust --overlap-threshold 255 analyze test.pcap`.
Assert exit code is not a parse error (it may error due to test.pcap not existing — that
is acceptable; the point is that clap accepted the flag value).

**Default values:**
At the unit test level, call `Cli::try_parse_from(["wirerust", "analyze", "test.pcap"])`.
Assert `cli.reassembly_depth == 10`.
Assert `cli.reassembly_memcap == 1024`.
Assert `cli.overlap_threshold.is_none()`.

**Valid boundary:**
Call `Cli::try_parse_from(["wirerust", "--overlap-threshold", "0", "analyze", "test.pcap"])`.
Assert it succeeds.
Assert `cli.overlap_threshold == Some(0)`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Range enforcement is correct at both boundary points (255 accepted, 256 rejected); defaults are correct.
- **Edge case handling** (weight: 0.2): Boundary values (0 and 255) are both accepted; 256 is not.
- **Error quality** (weight: 0.2): The out-of-range error message identifies the flag and the allowed range.
- **Performance** (weight: 0.05): Immediate parse-time rejection.
- **Data integrity** (weight: 0.05): Other flags are not affected by the range check on this one.

## Edge Conditions

- `--small-segment-threshold` has a different range (0-2048); its boundary should also be tested.
- `--reassembly-depth 0` is valid (0 disables depth limit or sets it to the minimum).
- All threshold flags absent: defaults are used without error.
- `--small-segment-ignore-ports 23,513`: comma-delimited parsing produces `[23, 513]`.

## Failure Guidance

"HOLDOUT LOW: HS-094 (satisfaction: 0.XX) -- The --overlap-threshold flag did not enforce its 0-255 range at parse time (256 was silently accepted), or the reassembly depth/memcap defaults were wrong."
