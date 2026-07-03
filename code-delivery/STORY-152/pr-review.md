# PR #353 Review — STORY-152 `wirerust protocols` subcommand

**Verdict:** APPROVE
**Blocking findings:** 0
**Non-blocking findings:** 0
**Cosmetic findings:** 1

Fresh-eyes review of the diff against `develop`. All three behavioral contracts
(BC-2.12.022 v1.0, BC-2.18.001 v1.4, BC-2.18.002 v1.1) are implemented in a way
that matches the frozen contract text, and the test suite exercises each contract
clause with named regression guards.

## Summary of what was verified

### BC-2.12.022 — subcommand dispatch and `--json` flag
- `Commands::Protocols { all, supported, unsupported }` is registered in `src/cli.rs:281-296`
  with `conflicts_with_all` on every flag pair, so clap rejects any two-flag
  combination before dispatch (Invariant 2, EC-006).
- The dispatch arm in `src/main.rs:151-164` maps `supported → Supported`,
  `unsupported → Unsupported`, and everything else (including bare `--all` and
  the default no-flag case) to `ProtocolFilter::All`. The `all` flag is
  intentionally consumed via `..` because it collapses to the default branch —
  correct per Invariant 3.
- `run_protocols` receives `cli.json.is_some()` — matches the frozen "boolean
  presence, path ignored" model documented in the PR description.
- `run_protocols` (`src/main.rs:562-575`) calls only the catalog functions
  (`all_protocols`, `supported_protocols`, `unsupported_protocols`) and the two
  render functions. No `StreamDispatcher`, no `analyzer::*`, no `reassembly::*`
  imports leak into the dispatch path (Invariant 5).
- `analyze` dispatch is untouched — AC-152-008 regression safety holds.

### BC-2.18.001 — terminal catalog table
- Header + separator row is followed by data rows in catalog-declaration order.
- Transport column emits `"[L2]"` for `Transport::LinkLayer` (PC-7 / Invariant 4).
- Port column emits an em-dash (U+2014) for LinkLayer entries and comma-joined
  port list otherwise (PC-3).
- EtherType column emits `"0x{XXXX} ({decimal})"` for `Some(_)` and em-dash for
  `None`, so ARP (ethertype `None`) shows em-dash and GOOSE shows
  `"0x88B8 (35000)"` (PC-5, EC-004).
- Supported column: rendered as `"yes"`/`"no"` derived by `is_protocol_supported`.
- Port-102 footnote is emitted iff any of `S7comm`, `S7comm-plus`, `IEC 61850 MMS`,
  `ICCP/TASE.2` appear in the printed set, and the single footnote string names
  all four explicitly (PC-6, AC-152-004). Regression-guarded by
  `test_BC_2_18_001_port102_footnote_names_all_four` and the absence test
  `test_BC_2_18_001_port102_footnote_absent_supported`.
- L2/LinkLayer note is emitted iff any LinkLayer entry appears in the printed set
  (PC-7).

### BC-2.18.002 — JSON output schema
- Root object is `{"protocols": [...]}` — matches Invariant 1.
- Each element carries exactly the seven required fields: `name`, `category`,
  `transport`, `canonical_ports`, `ethertype`, `port_detectable`, `supported`.
- `category` renders as `"ICS"` or `"IT"` only — no `"L2"` value (ADR-012 Decision 7).
- `transport` renders as `"TCP"` / `"UDP"` / `"LinkLayer"`.
- `canonical_ports` is `[]` for LinkLayer entries; `ethertype` is `null` for
  entries with `ethertype: None` (including ARP).
- Emission order equals catalog-declaration order — asserted by
  `test_BC_2_18_002_json_declaration_order`.
- `supported` is derived, not stored on `KnownProtocol` — consistent with the
  PR description note (3).

### Canonical values (DF-CANONICAL-FRAME-HOLDOUT-001)
Independently verified against the diff and asserted by tests:
| Protocol | Expected | Present in code | Asserted by test |
|---|---|---|---|
| GOOSE EtherType | 0x88B8 (35000) | `src/protocols.rs:259` | `test_BC_2_18_001_goose_ethertype_display`, `test_BC_2_18_002_goose_json_canonical` |
| POWERLINK EtherType | 0x88AB (34987) | `src/protocols.rs:299` | `test_BC_2_18_001_powerlink_ethertype_display` |
| BACnet/IP UDP | 47808 | `src/protocols.rs:207` | `test_BC_2_18_002_bacnet_json_canonical` |
| Modbus/TCP TCP | 502 | `src/protocols.rs:101` | `test_BC_2_18_002_modbus_json_canonical` |

### Test coverage and diff coherence
- 25 `#[test]` functions in `tests/integration_tests.rs` (1022 new lines) —
  covers the exit-0 dispatch case, mutual-exclusion error path, filter row-count
  derivations, both footnote presence and absence, both render modes for the
  three key canonical protocols, JSON schema shape, and default-equals-all.
- The two `Commands::Protocols {..} => panic!()` arms added to
  `tests/cli_story_086_tests.rs` and `tests/cli_story_096_tests.rs` are the
  mechanical exhaustive-match cost of introducing a new `Commands` variant.
  They are correctly kept as `panic!()` because those test files never invoke
  the `protocols` subcommand — flagged intentional in the PR context.
- Diff size (1268 additions, 1 deletion) is dominated by tests (1022 lines) and
  is entirely on-story.

## Findings

### Cosmetic (1)

**COSMETIC — Redundant ARP short-circuit in `is_protocol_supported`**
File: `src/main.rs:580-587`

`is_protocol_supported` starts with `if p.name == "ARP" { return true; }` and
then falls through to `supported_protocols().iter().any(|sp| sp.name == p.name)`.
But `supported_protocols()` already applies the `|| p.name == "ARP"` special case
internally, so the explicit branch is dead in the sense that removing it would
not change the return value for any input. Keeping it is defensible as
readability (the ARP path is documented on both sides), but it does mean the
ARP invariant is now enforced in two places instead of one. Optional to fix;
no behavioral consequence.

There is also a mild efficiency remark on the same function: it is called once
per printed row and each call re-materializes `supported_protocols()` (an
allocating `Vec` build over 30 catalog entries). At N=30 this is invisible;
if the catalog grows, caching the supported-name set once per render would be
the natural next step. Not a defect for this PR.

## Convergence checklist

- [x] Diff coherence — all changes serve STORY-152
- [x] Description accuracy — PR body matches the diff
- [x] Test coverage — 25 tests covering all three BCs, including error paths and canonical values
- [x] Commit quality — conventional format, story ID, TDD-shaped commit sequence (RED → GREEN → refinement)
- [x] Diff size — reasonable; the bulk is tests
- [x] Missing changes — none detected; every AC-152-NNN has a corresponding named test
- [x] Dependency status — STORY-152 sits atop the protocol-catalog module (`src/protocols.rs`) already on `develop`
- [x] Architecture compliance — pure CLI → catalog → render; no analyzer/dispatcher imports (BC-2.12.022 Invariant 5)

## Verdict

**APPROVE.** No blocking findings. The one cosmetic redundancy above is
optional. Safe to merge.
