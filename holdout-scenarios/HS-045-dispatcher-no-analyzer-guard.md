---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-033.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.008.md
input-hash: "d957cd6"
traces_to: .factory/stories/STORY-033.md
id: "HS-045"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-3"
behavioral_contracts:
  - BC-2.05.008
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Dispatcher With No Analyzers Configured Does Not Process Data

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A user wants to run wirerust in a mode where TCP reassembly operates but neither
HTTP nor TLS analysis is enabled (perhaps to generate only statistics about
flows, not protocol-level findings). The dispatcher should skip all classification
and content inspection entirely when no analyzers are configured.

1. The user runs: `wirerust analyze <any-pcap> --output-format json` with flags that
   disable all protocol analyzers (or wirerust is built with a configuration
   that omits analyzers).
2. The tool completes with exit code 0.
3. No classification, no content inspection, no routes/classification_attempts
   map updates occur — the dispatcher's early-return guard fires immediately.
4. The `unclassified_flows` counter is NOT incremented (the guard also applies
   to the unclassified counter when no analyzers are configured).
5. No HTTP or TLS findings appear in the output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.008 | postcondition 1-5: with http=None and tls=None, on_data returns immediately | No classification work done |
| BC-2.05.008 | invariant 1: early-return is first statement in on_data | Guard position is before route lookup |
| BC-2.05.008 | invariant 2: guard does NOT affect on_flow_close | Flow close cleanup still runs correctly |

## Verification Approach

```bash
wirerust analyze <any-pcap> --output-format json  # (no-analyzer build or mode)
```

Verify:
- Exit code is 0.
- No HTTP-level or TLS-level findings in output.
- If TCP-level findings (overlaps, evasion) exist in the pcap, those still appear
  (they come from the reassembly engine, not the dispatcher).
- `unclassified_flows` is 0 (no analyzers configured means no counting).

For a single-analyzer mode (http=Some, tls=None): verify the early-return does
NOT fire, and the HTTP analyzer runs correctly. TLS data on this setup produces
no TLS findings (analyzer is None) but does not crash.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): No-analyzer mode produces no
  protocol-level findings; on_data is effectively a no-op.
- **Edge case handling** (weight: 0.2): Single-analyzer mode (one of http/tls)
  works correctly; early-return does not fire.
- **Error quality** (weight: 0.2): No panic when TLS data arrives but tls=None.
- **Performance** (weight: 0.05): No-analyzer mode may be faster (no classify calls).
- **Data integrity** (weight: 0.05): TCP-level findings (reassembly engine) still
  present even in no-analyzer mode.

## Edge Conditions

- `http=None, tls=Some`: early-return does NOT fire; TLS analyzer runs.
- `http=Some, tls=None`: early-return does NOT fire; HTTP analyzer runs.
- `http=None, tls=None`: early-return fires; unclassified_flows stays 0.
- on_flow_close with no analyzers: routes.remove and attempts.remove still called
  (cleanup still runs); unclassified_flows NOT incremented.

## Failure Guidance

"HOLDOUT LOW: HS-045 (satisfaction: 0.XX) — the no-analyzer early-return guard
did not fire; the dispatcher performed classification work and incremented counters
when no analyzers were configured, wasting CPU and producing incorrect statistics."
