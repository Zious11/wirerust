---
document_type: story
level: ops
story_id: STORY-193
title: "S7comm Dispatcher Integration: DispatchTarget::S7comm Rule 9 + Support Enum Catalog Promotion + --s7comm Flag (VP-004/VP-041 Amendments)"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 8
priority: P1
cycle: feature-s7comm
wave: 96
target_module: analyzer/s7comm
subsystems: [SS-05, SS-12, SS-18, SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-192]
blocks: [STORY-194]
behavioral_contracts: [BC-2.05.013, BC-2.18.003, BC-2.18.004, BC-2.18.005, BC-2.18.006]
verification_properties: [VP-004, VP-041]
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.013.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.005.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.006.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/cycles/feature-s7comm/f2-port102-model-validation.md
input-hash: "4a8261b"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-193: S7comm Dispatcher Integration: DispatchTarget::S7comm Rule 9 + Support Enum Catalog Promotion + --s7comm Flag

## Narrative

**As a** wirerust user and security analyst,
**I want** the S7comm analyzer wired into the stream dispatcher with a `--s7comm` CLI
flag, and the protocol catalog's port-102 four-way collision resolved via a per-entry
`Support` enum (RATIFIED option d),
**so that** the full S7comm passive analysis pipeline is activated end-to-end, with
classic S7comm correctly reported `Support::Supported`, S7comm-plus correctly reported
`Support::DetectionOnly`, and IEC 61850 MMS/ICCP-TASE.2 correctly remaining
`Support::KnownUnsupported` — all four sharing canonical port 102 without breaking the
catalog partition invariant.

This story performs the largest single diff of the epic: the `Support` enum touches all
~30 `KnownProtocol` literals in `src/protocols.rs` (not just the four port-102 entries),
per ADR-014 Decision 3's ratified design.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.05.013 | `classify()` Rule 9 — TCP Port 102 Returns a Single DispatchTarget::S7comm | Dispatcher-layer half of the port-102 split |
| BC-2.18.003 | `supported_protocols()` Returns Support::Supported Entries; `unsupported_protocols()` Returns the `!= Supported` Complement | Catalog derivation rewrite |
| BC-2.18.004 | Catalog Partition Invariant — Supported union Unsupported == KNOWN_PROTOCOLS, Disjoint | Partition preserved under the new derivation |
| BC-2.18.005 | Support Enum on KnownProtocol — Exhaustive, Compile-Time-Enforced Per-Entry Assignment | The `Support` enum itself; touches all ~30 literals |
| BC-2.18.006 | Port-102 Four-Way Support Assignment (S7comm=Supported, S7comm-plus=DetectionOnly, MMS/ICCP=KnownUnsupported) | Concrete per-entry assignments |

## Acceptance Criteria

### AC-193-001: DispatchTarget::S7comm variant added and Rule 9 routes port 102
(traces to BC-2.05.013 postcondition 1)
- Given the `StreamDispatcher` in `src/dispatcher.rs`
- When a TCP flow with `src_port == 102` or `dst_port == 102` is classified, and no
  higher-priority rule (Rules 1-8) has already matched
- Then `classify(data, flow_key)` returns `Some(DispatchTarget::S7comm)` (Rule 9,
  inserted after Rule 8/IEC-104; the former "no match" Rule 9 is renumbered Rule 10)
- The flow is routed to `S7commAnalyzer::on_data` (traces to BC-2.05.013
  postcondition 2)
- No other `DispatchTarget` variant is ever returned for TCP/102 by `classify()` itself
  — no `DispatchTarget::S7commPlus`/`Mms`/`Iccp` variant exists (traces to BC-2.05.013
  postcondition 3)
- **Test:** `test_BC_2_05_013_rule9_port_102_dispatches_s7comm`

### AC-193-002: VP-004 classifier oracle updated atomically for DispatchTarget::S7comm
(traces to BC-2.05.013 invariant 4)
- Given the `#[cfg(kani)]` block in `src/dispatcher.rs` containing `classify_oracle`
- When `DispatchTarget::S7comm` is added to the dispatcher
- Then, in the SAME commit: (a) the `DispatchTarget::S7comm` variant is added, (b) Rule 9
  is added to `classify()`, (c) the mirrored `S7comm` arm is added to `classify_oracle`,
  (d) the early-exit guard is extended with `&& self.s7comm.is_none()`, (e) `S7comm`
  match arms are added to `on_data`/`on_flow_close`, and (f)
  `verify_content_first_precedence_exhaustive` is re-run and passes
- **Test:** `test_BC_2_05_013_vp004_oracle_atomic_six_step` (verifies all six steps
  landed together via code inspection + a passing Kani re-run)

### AC-193-003: Content-first precedence preserved — a TLS/HTTP signature on port 102 wins over Rule 9
(traces to BC-2.05.013 invariant 2, edge case EC-006)
- Given data on port 102 beginning with a TLS ClientHello signature (`0x16 0x03`)
- When `classify(data, flow_key)` is called
- Then it returns `Some(DispatchTarget::Tls)` (Rule 1), not `Some(DispatchTarget::S7comm)`
- **Test:** `test_BC_2_05_013_content_first_precedence_over_rule9`

### AC-193-004: Rule 8 (IEC-104) is unaffected by Rule 9's insertion
(traces to BC-2.05.013 invariant 6, edge case EC-009)
- Given a TCP flow on port 2404
- When `classify(data, flow_key)` is called
- Then it returns `Some(DispatchTarget::Iec104)`, not `Some(DispatchTarget::S7comm)` —
  confirms Rule 9's insertion does not shadow or regress Rule 8
- **Test:** `test_BC_2_05_013_rule8_iec104_unaffected_by_rule9`

### AC-193-005: `--s7comm` CLI flag enables S7comm analysis
- Given `cargo run -- --s7comm <pcap>`
- When wirerust processes a pcap containing S7comm traffic on port 102
- Then `S7commAnalyzer` is instantiated and registered with the dispatcher; without
  `--s7comm`, no S7comm analysis is performed
- **Test:** `test_s7comm_cli_flag_enables_analysis`

### AC-193-006: StreamDispatcher gains an `s7comm` field with early-exit guard extension
(traces to BC-2.05.013 invariant 4, sub-clause c)
- Given `StreamDispatcher` after this story
- When constructed with ONLY `s7comm` set (all other analyzers `None`)
- Then data on a port-102 flow reaches `S7commAnalyzer` — this catches a silent-drop bug
  if `self.s7comm.is_none()` were omitted from the early-exit guard
- **Test:** `test_s7comm_only_dispatcher_reaches_analyzer`

### AC-193-007: `Support` enum added to `KnownProtocol`; every literal supplies an explicit value
(traces to BC-2.18.005 postconditions 1-3)
- Given `src/protocols.rs`'s `KnownProtocol` struct after this story
- When inspected
- Then it has a new field `pub support: Support` where `Support` is
  `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` with exactly three variants
  (`Supported`, `KnownUnsupported`, `DetectionOnly`) and NO `Default` impl
- Every one of the ~30 `KnownProtocol` literals in `KNOWN_PROTOCOLS` supplies an
  explicit `support:` value — omitting it is a compile error
- **Test:** `cargo check` passing is itself the enforcement mechanism;
  `test_BC_2_18_005_all_known_protocols_have_explicit_support` (compile-time
  regression-guard via exhaustive struct construction)

### AC-193-008: 26 pre-existing entries' Support values exactly mirror their pre-feature status
(traces to BC-2.18.005 postcondition 4)
- Given the 26 `KnownProtocol` entries whose status is unchanged by this feature
- When their `support:` field is assigned
- Then Modbus, DNP3, ENIP, IEC-104, TLS, DNS, HTTP, ARP -> `Support::Supported`; every
  other pre-existing entry -> `Support::KnownUnsupported` — no behavioral change, only a
  change in how the fact is expressed
- **Test:** `test_BC_2_18_005_pre_existing_entries_support_unchanged`

### AC-193-009: Port-102 four entries receive the ratified per-entry assignment
(traces to BC-2.18.006 postconditions 1-4)
- Given the four `KNOWN_PROTOCOLS` entries with `canonical_ports: &[102]`
- When their `support:` field is assigned
- Then S7comm -> `Support::Supported`; S7comm-plus -> `Support::DetectionOnly`; IEC 61850
  MMS -> `Support::KnownUnsupported`; ICCP/TASE.2 -> `Support::KnownUnsupported`
- **Test:** `test_BC_2_18_006_port_102_four_way_assignment`

### AC-193-010: `supported_protocols()`/`unsupported_protocols()` rewritten to filter on Support, not SUPPORTED_PORTS
(traces to BC-2.18.003 postconditions 1-2)
- Given `src/protocols.rs`'s `supported_protocols()` and `unsupported_protocols()`
  functions after this story
- When called
- Then `supported_protocols()` returns exactly the entries where `p.support ==
  Support::Supported`; `unsupported_protocols()` returns the complement `p.support !=
  Support::Supported` — critically NOT `p.support == Support::KnownUnsupported`, which
  would silently drop every `DetectionOnly` entry from both sets
- The ARP special-case (`|| p.name == "ARP"`) is removed entirely — ARP's `Supported`
  status is now expressed directly via its `support:` field (traces to BC-2.18.003
  postcondition 3)
- **Test:** `test_BC_2_18_003_supported_protocols_filters_on_support_field`,
  `test_BC_2_18_003_unsupported_is_not_supported_complement_not_known_unsupported_equality`
  (the canonical `!=` vs `==` regression guard)

### AC-193-011: S7comm-plus (DetectionOnly) is retained in unsupported_protocols(), not dropped from both sets
(traces to BC-2.18.006 postcondition 6, BC-2.18.003 edge case EC-007/EC-008)
- Given S7comm-plus's `support == Support::DetectionOnly`
- When `unsupported_protocols()` is called
- Then S7comm-plus appears in the result — this is the canonical regression this story's
  Invariant 3 exists to prevent (a naive `== KnownUnsupported` implementation would
  silently exclude it from both sets, breaking the partition invariant)
- **Test:** `test_BC_2_18_006_s7comm_plus_retained_in_unsupported`

### AC-193-012: Partition invariant preserved after all ~30 literals gain explicit Support values
(traces to BC-2.18.004 postconditions 1-5)
- Given the amended `supported_protocols()`/`unsupported_protocols()` derivation
- When the VP-041 proptest harnesses run (`proptest_vp041_oracle_cross_check`,
  `proptest_vp041_partition_invariant`)
- Then `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` (union
  completeness), `supported_protocols() ∩ unsupported_protocols() == ∅` (disjoint), and
  the counting invariant `supported.len() + unsupported.len() == KNOWN_PROTOCOLS.len()`
  holds, exercised with S7comm-plus's `DetectionOnly` entry present as a concrete
  non-vacuous case
- **Test:** amended `proptest_vp041_oracle_cross_check`, amended
  `proptest_vp041_partition_invariant`

### AC-193-013: The dynamic coverage-gap classifier (`lookup_protocol_state`) is explicitly and provably unchanged
(traces to BC-2.18.006 postcondition 7, non-goal)
- Given `main.rs::lookup_protocol_state`'s existing behavior for `(Tcp, 102)` gap-flows
  before this feature
- When this feature lands
- Then its behavior is byte-for-byte unchanged — it continues to attribute an
  unclassified port-102 gap flow to whichever port-102 catalog entry it matches first by
  declaration order, regardless of whether the underlying traffic is genuinely S7comm,
  S7comm-plus, MMS, or ICCP; this is a documented, pre-existing limitation this cycle's
  static-catalog fix does not and cannot resolve, deferred explicitly to a future F4
  cycle
- **Test:** `test_BC_2_18_006_lookup_protocol_state_unchanged_regression_guard`

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `DispatchTarget::S7comm` | SS-05 dispatcher | `src/dispatcher.rs` | N/A (enum variant) |
| Rule 9 classify arm | SS-05 dispatcher | `src/dispatcher.rs` | Pure (classification fn) |
| VP-004 oracle update | SS-05 Kani | `src/dispatcher.rs` | `#[cfg(kani)]` |
| `--s7comm` flag | SS-12 CLI | `src/cli.rs` | Effectful (CLI parsing) |
| `main.rs` wiring | SS-12 entry | `src/main.rs` | Effectful |
| `Support` enum, `KnownProtocol.support` field | SS-18 protocols | `src/protocols.rs` | N/A (compile-time data model) |
| `supported_protocols()`/`unsupported_protocols()` (rewritten) | SS-18 protocols | `src/protocols.rs` | Pure |
| `S7commAnalyzer` registration | SS-21 | `src/analyzer/mod.rs` | Effectful |

Subsystem anchors:
- SS-05 owns dispatch (Rule 9, `DispatchTarget::S7comm` per ARCH-INDEX.md §SS-05)
- SS-12 owns CLI flags (`--s7comm` per ARCH-INDEX.md §SS-12)
- SS-18 owns the protocol catalog (`Support` enum per ARCH-INDEX.md §SS-18)
- SS-21 owns the S7comm analyzer's dispatcher registration

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `classify()` Rule 9, `classify_oracle` | pure-core | Pure classification function; no I/O |
| `supported_protocols()`, `unsupported_protocols()` | pure-core | Filter over `&'static KNOWN_PROTOCOLS`; no mutable state |
| `StreamDispatcher::on_data`/`on_flow_close` (s7comm arms) | effectful-shell | Routes data to `S7commAnalyzer` |
| `--s7comm` CLI wiring in `main.rs` | effectful-shell | Instantiates and registers the analyzer |

## VP-004 Six-Step Atomic Obligation (ADR-014 Decision 2)

**CRITICAL: all six steps below MUST land in a SINGLE git commit.**

```
1. Add DispatchTarget::S7comm variant to the DispatchTarget enum
2. Add the port-102 arm to classify() (Rule 9, after Rule 8 IEC-104)
3. Add the corresponding DispatchTarget::S7comm arm to classify_oracle in
   #[cfg(kani)] mod kani_proofs, mirroring production classify() syntactically
4. Extend the early-exit guard to include self.s7comm.is_none()
5. Add S7comm match arms to on_data and on_flow_close
6. Re-run verify_content_first_precedence_exhaustive and confirm VERIFICATION SUCCESSFUL
```

Failure to update `classify_oracle` atomically invalidates the VP-004 proof.

## VP-041 Amendment (proptest, re-anchored to the Support field)

`proptest_vp041_oracle_cross_check`'s independent oracle changes from
`(any canonical_port in SUPPORTED_PORTS) OR entry.name == "ARP"` to
`entry.support == Support::Supported` (still computed independently, still non-vacuous —
does NOT call `supported_protocols()`/`unsupported_protocols()`).
`proptest_vp041_partition_invariant` is exercised with S7comm-plus's `DetectionOnly`
entry present as a concrete regression case for the `!= Supported` vs
`== KnownUnsupported` distinction.

## Tasks

- [ ] `src/dispatcher.rs`: add `DispatchTarget::S7comm` variant + Rule 9 arm matching
      `[flow_key.lower_port(), flow_key.upper_port()].contains(&102)`
- [ ] `src/dispatcher.rs`: update `classify_oracle` with the mirrored Rule 9 arm (same
      commit as the variant + Rule 9 — the six-step atomic obligation above)
- [ ] `src/dispatcher.rs`: add `s7comm: Option<S7commAnalyzer>` field; extend `new()`;
      extend the early-exit guard; add `S7comm` arms to `on_data`/`on_flow_close`;
      update the module doc-comment's rule ladder (Rule 9 -> 102/S7comm, former Rule 9
      "no match" -> Rule 10)
- [ ] `src/cli.rs`: add `--s7comm` bool flag to `CliArgs`
- [ ] `src/main.rs`: wire `--s7comm` to instantiate + register `S7commAnalyzer`
- [ ] `src/protocols.rs`: add `pub enum Support { Supported, KnownUnsupported,
      DetectionOnly }` (derive `Debug, Clone, Copy, PartialEq, Eq`; NO `Default`); add
      `pub support: Support` field to `KnownProtocol`; assign an explicit `support:`
      value to ALL ~30 `KNOWN_PROTOCOLS` literals per the mapping table (26 pre-existing
      entries mirror their prior status exactly; the four port-102 entries per
      BC-2.18.006)
- [ ] `src/protocols.rs`: rewrite `supported_protocols()` to
      `KNOWN_PROTOCOLS.iter().filter(|p| p.support == Support::Supported).collect()`;
      rewrite `unsupported_protocols()` to filter `p.support != Support::Supported`
      (the complement, NOT `== Support::KnownUnsupported`); remove the ARP
      special-case entirely
- [ ] Update `tests/protocols_tests.rs`'s port-102-collision REGRESSION-GUARD test to
      assert the NEW outcome: S7comm is supported; S7comm-plus/MMS/ICCP are all three
      unsupported (do NOT delete the test — rewrite its assertion)
- [ ] Update `SUPPORTED_PORTS`'s doc-comment to note it is retired as the derivation
      mechanism for `supported_protocols()`/`unsupported_protocols()` as of this story
      (it may persist informationally; full removal is deferred per ADR-014 Decision 3)
- [ ] Amend `proptest_vp041_oracle_cross_check` and `proptest_vp041_partition_invariant`
      per the VP-041 Amendment section above
- [ ] Add a doc-comment note on `main.rs::lookup_protocol_state` citing the F4-deferred
      dynamic-gap-classifier limitation (ADR-014 Decision 3 critical caveat)
- [ ] Write integration tests: one per AC, named `test_BC_2_05_013_*`,
      `test_BC_2_18_003_*`, `test_BC_2_18_004_*`, `test_BC_2_18_005_*`,
      `test_BC_2_18_006_*`
- [ ] Verify `cargo test --all-targets` passes
- [ ] Verify `cargo kani --harness verify_content_first_precedence_exhaustive` passes
      (full formal re-run confirmed in STORY-194; a local check here is recommended)
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the `--s7comm` flag
      and `[Unreleased] > Changed` describing the `Support` enum catalog rewrite, before
      creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.05.013 | `src_port=102, dst_port=102` (port appears on both) | `Some(S7comm)` |
| EC-002 | BC-2.05.013 | `src_port=101, dst_port=103` (port 102 not present) | `None` from Rule 9 (falls through to Rule 10) |
| EC-003 | BC-2.18.003 | ARP's `support` field is `Support::Supported`, declared directly | ARP appears in `supported_protocols()`; no port-intersection special case exists anymore |
| EC-004 | BC-2.18.005 | A developer adds a new `KnownProtocol` literal without a `support:` field | Compile error — "missing field `support`" |
| EC-005 | BC-2.18.005 | `KnownProtocol { support: Support::default(), .. }` attempted | Compile error — `Support` has no `Default` impl |
| EC-006 | BC-2.18.006 | `--coverage-gaps` run against genuine S7comm-plus traffic on port 102 with no `--s7comm` flag | `lookup_protocol_state` reports the gap-flow using its pre-existing, unfixed first-declaration-order attribution — documented, unfixed, not a regression |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~7,500 |
| BC files (5 cross-subsystem BCs, high density) | ~9,000 |
| ADR-014 (Decisions 2, 3, 9) + f2-port102-model-validation.md | ~14,000 |
| src/dispatcher.rs (existing) | ~5,000 |
| src/protocols.rs (existing, ~30 literals) | ~7,000 |
| src/cli.rs + src/main.rs | ~3,000 |
| Test files delta | ~4,000 |
| **Total** | **~49,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~25%** (near the top end of budget — if needed, load only the specific protocols.rs sections under diff, not the full ~30-literal catalog file, since 26 of the 30 edits are mechanical field additions) |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-186 through STORY-192 | `S7commAnalyzer` fully built: carry buffers, four-way dispatch, classification, MITRE emission | `S7commAnalyzer` is feature-complete going into this story | This story is the first to touch `dispatcher.rs`, `cli.rs`, `main.rs`, and `protocols.rs` — all previously untouched by this epic; the `Support` enum diff is unusually large (~30 literals) compared to the IEC-104 precedent's single `SUPPORTED_PORTS.push(2404)`-style change — budget extra review time for this story specifically |

Mirrors STORY-173's role in the IEC-104 epic (dispatcher integration + catalog
promotion + CLI flag), but with a materially larger catalog diff due to the
per-entry `Support` enum replacing the pure-intersection `SUPPORTED_PORTS` derivation
(ADR-014 Decision 3, RATIFIED option d, superseding this ADR's own original
recommendation of a smaller name-keyed exclusion list).

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 2**: Rule 9 is port-only, fires after Rule 8, before the
  renumbered Rule 10 "no match" fallthrough. The VP-004 six-step atomic obligation is
  non-negotiable — a Rule 9 addition without the oracle update invalidates the proof.
- **ADR-014 Decision 3 (RATIFIED)**: the port-102 catalog-model fix is the per-entry
  `Support` enum (option d), not the ADR's own originally-recommended name-keyed
  exclusion list (option b) — superseded per the independent validation in
  `f2-port102-model-validation.md`. `unsupported_protocols()` MUST filter `!=
  Support::Supported`, never `== Support::KnownUnsupported`.
- **ADR-014 Decision 3's critical caveat**: this story fixes only the STATIC catalog
  partition. `main.rs::lookup_protocol_state` (the DYNAMIC coverage-gap classifier)
  remains unfixed and is explicitly deferred to F4 — do not attempt to fix it in this
  story.
- Pure/effectful boundary: `classify()`, `classify_oracle`,
  `supported_protocols()`/`unsupported_protocols()` remain pure; dispatcher wiring and
  CLI flag handling are the effectful shell.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Enum variant, match arm, struct field addition |
| kani | Latest via `cargo kani` | VP-004 oracle update verification |
| proptest | 1 (pinned in `Cargo.toml`) | VP-041 amended partition/oracle-cross-check harnesses |

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/dispatcher.rs` | MODIFY | Add `DispatchTarget::S7comm`; Rule 9 port-102 arm; VP-004 oracle update; `s7comm` field; `on_data`/`on_flow_close` arms; module doc rule-ladder renumber |
| `src/protocols.rs` | MODIFY | Add `Support` enum + `KnownProtocol.support` field; assign explicit values to ALL ~30 literals; rewrite `supported_protocols()`/`unsupported_protocols()`; remove ARP special case |
| `src/cli.rs` | MODIFY | Add `--s7comm` bool flag |
| `src/main.rs` | MODIFY | Wire `--s7comm` -> instantiate + register `S7commAnalyzer`; doc-comment note on `lookup_protocol_state`'s F4-deferred limitation |
| `src/analyzer/mod.rs` | MODIFY | `pub use s7comm::S7commAnalyzer;` (public export) |
| `tests/protocols_tests.rs` | MODIFY | Rewrite the port-102-collision REGRESSION-GUARD test's assertion; amend VP-041 harnesses |
| `tests/dispatcher_tests.rs` (or equivalent) | MODIFY | Add BC-2.05.013 dispatcher integration tests |

## Forbidden Dependencies

- `Support` MUST NOT derive `Default` — this reintroduces the unsafe-omission failure
  mode the design exists to prevent
- `protocols.rs` MUST NOT depend on `dispatcher.rs` at any point — the `Support` enum
  introduces no such coupling (this was the reason ADR-014's option (c),
  `dispatch_target: Option<&'static str>`, was rejected)
- PARTIAL VP-004 six-step registration is forbidden: any single missing step fails the
  Kani drift-guard

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — DispatchTarget::S7comm Rule 9, VP-004 six-step atomic, `--s7comm` CLI flag, `Support` enum catalog rewrite across all ~30 KnownProtocol literals, VP-041 amendment, AC-193-001..013. |
