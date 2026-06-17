---
document_type: story
story_id: STORY-116
epic_id: E-17
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-16T00:00:00Z
phase: f3
points: 3
priority: P1
depends_on: [STORY-115]
blocks: [STORY-117]
behavioral_contracts:
  - BC-2.16.009
  - BC-2.16.015
verification_properties: [VP-024]
tdd_mode: facade
target_module: decoder
subsystems: [SS-16]
estimated_days: 1
feature_id: e17-arp-qinq-macsec-offset-hardening
github_issue: 253
wave: 45
# BC status: BC-2.16.009 v1.10 (EC-008 QinQ offset, EC-009 MACsec observe-only probe),
#             BC-2.16.015 v1.9 (PC-7a QinQ offset 22, EC-008, EC-009) — authored 2026-06-16.
# tdd_mode: facade — this story delivers existing test files (no production code change);
#   delivery = merge PR #258 (test/arp-qinq-macsec-fixtures) after CI green. No todo!() stubs.
# Subsystem anchor: SS-16 owns this story's scope because QinQ ARP offset coverage
#   is part of the ARP decoder lax-path, which is an SS-16 / src/decoder.rs concern
#   per ARCH-INDEX Subsystem Registry.
inputs:
  - .factory/phase-f1-delta-analysis/e17-arp-qinq-macsec-offset-delta-analysis.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
input-hash: "c389b39"
---

# STORY-116: ARP QinQ (Double-Tag) Decoder Offset Coverage

## Narrative

- **As a** ICS/OT security analyst using wirerust on VLAN-segmented networks
- **I want** the ARP decoder to correctly classify QinQ double-tagged (outer 0x88a8 + inner
  0x8100) frames as either genuinely truncated (no false D11) or genuinely malformed (D11
  fires), with the offset formula `14 + Σ link_exts.header_len()` pinned by regression tests
- **So that** QinQ ARP coverage is behaviorally verified and the etherparse data-model
  assumption (two separate `Vlan` link-ext entries per QinQ frame) is guarded against future
  version drift

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.009 | D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding |
| BC-2.16.015 | Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced; Analysis Gated on --arp |

## Delivery Context

This story formalizes the AC-to-test-to-BC traceability for 4 tests that **already exist**
on branch `test/arp-qinq-macsec-fixtures` (PR #258):
`tests/bc_2_16_qinq_macsec_offset_tests.rs`.

**F4 implementation = merge PR #258 after CI green.** No `src/` production code change.
No `todo!()` stubs. The `tdd_mode: facade` flag reflects this: the tests were written
as behavioral verification of the already-correct offset formula, not as TDD drivers for
new production code.

The 4th test (`test_BC_2_16_015_macsec_arp_lax_parse_probe`) lives in this file but is
observe-only for MACsec shape/no-panic. MACsec offset assertions belong to STORY-117.

## Acceptance Criteria

### AC-001 (traces to BC-2.16.015 postcondition 7b / EC-008 — QinQ benign truncation MUST NOT produce D11)
A QinQ double-tagged ARP frame with a valid Ethernet/IPv4 fixed header (htype=0x0001,
ptype=0x0800, hlen=6, plen=4) at offset 22 (14 Ethernet + 4 outer 802.1Q + 4 inner 802.1Q)
but no variable section produces `Err("truncated ARP frame")` and `malformed_findings == 0`.

The `lax.link_exts` for this frame MUST contain exactly TWO `Vlan` entries (no `VlanDouble`
variant in etherparse 0.20.2), with `Σ header_len() == 8`, so the decoder reads ARP at
the correct offset 22 and classifies the benign header as genuine truncation.

- **Test:** `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11`
  (in `tests/bc_2_16_qinq_macsec_offset_tests.rs`)

### AC-002 (traces to BC-2.16.009 Precondition 3 / Postcondition 1 / EC-008 — QinQ malformed hlen=8 MUST route to D11)
A QinQ double-tagged ARP frame with the same outer framing (offset 22) but with `hlen=8`
(non-Ethernet hardware-address length) in the ARP fixed header produces
`Err("Non-Ethernet/IPv4 ARP frame")`, `malformed_findings >= 1`, and the D11 finding
has `category == Anomaly`, `mitre_techniques: []`.

- **Test:** `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11`
  (in `tests/bc_2_16_qinq_macsec_offset_tests.rs`)

### AC-003 (traces to BC-2.16.015 EC-008 — offset-formula pin for QinQ data-model)
For a QinQ double-tagged frame, `lax.link_exts.len() == 2`, all entries are
`LaxLinkExtSlice::Vlan`, each reports `header_len() == 4`, and `Σ header_len() == 8`.
For a single-VLAN control frame, `lax.link_exts.len() == 1` and `Σ header_len() == 4`.
This test pins the etherparse 0.20.2 data-model assumption against future version drift.

- **Test:** `test_BC_2_16_015_qinq_link_exts_offset_formula_pin`
  (in `tests/bc_2_16_qinq_macsec_offset_tests.rs`)

### AC-004 (traces to BC-2.16.015 EC-009(a) — MACsec observe-only probe: no-panic and shape guard)
A MACsec Unmodified/no-SCI ARP frame (EtherType 0x88E5) constructed with
`MacsecHeader { sci: None, ptype: Unmodified(ARP) }` produces `lax.link_exts` with exactly
ONE `Macsec` entry, `MacsecHeader::header_len() == 8`, and
`LaxMacsecSlice.header.header_len() == 8` (the two APIs agree). No panic occurs.

This test is **observe-only for offset**: it confirms no-panic and that the MACsec
link-ext shape is correct, but does NOT assert `arp_offset == 22`. Offset assertions
for MACsec are owned by STORY-117 / `tests/bc_2_16_e17_macsec_offset_tests.rs`.

- **Test:** `test_BC_2_16_015_macsec_arp_lax_parse_probe`
  (in `tests/bc_2_16_qinq_macsec_offset_tests.rs`)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `decode_packet` lax None arm — QinQ offset formula | `src/decoder.rs` | Pure core (reads bytes) |
| `LaxLinkExtSlice::Vlan::header_len()` delegation | etherparse 0.20.2 (external) | Pure (arithmetic) |
| `ArpAnalyzer::record_malformed` | `src/analyzer/arp.rs` | Pure core (stateful) |

Architecture section references: `architecture/module-decomposition.md` (SS-16 C-23, `src/decoder.rs`
lax-path ARP routing); `arp-architecture-delta.md` §2.2 (QinQ offset formula, link_exts sum).

## Forbidden Dependencies

- `tests/bc_2_16_qinq_macsec_offset_tests.rs` MUST NOT import from `tests/bc_2_16_e17_macsec_offset_tests.rs`
  or share fixture builders with it. The two test files have distinct concerns (QinQ + observe-only MACsec
  probe vs. MACsec offset assertions) and must remain independently compilable.
- No `src/` production code changes are permitted in this story. Any PR touching `src/decoder.rs` or
  `src/analyzer/arp.rs` for this story indicates scope creep that violates E-17 F1 scope decision.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | QinQ benign truncated (hlen=6, no variable section) | `"truncated ARP frame"`, no D11 — AC-001 |
| EC-002 | QinQ malformed (hlen=8) | `"Non-Ethernet/IPv4 ARP frame"`, D11 LOW/Anomaly — AC-002 |
| EC-003 | etherparse QinQ representation: two Vlan entries | `lax.link_exts.len() == 2`, `Σ == 8` — AC-003 |
| EC-004 | Single-VLAN control case (regression baseline) | `lax.link_exts.len() == 1`, `Σ == 4` — AC-003 |
| EC-005 | MACsec Unmodified/no-SCI probe | one Macsec link_ext, `header_len() == 8`, no panic — AC-004 |

## Tasks

1. Confirm the 4 tests in `tests/bc_2_16_qinq_macsec_offset_tests.rs` are passing on
   branch `test/arp-qinq-macsec-fixtures`.
2. Confirm `cargo test --all-targets` green and `cargo clippy --all-targets -- -D warnings` clean.
3. Confirm `cargo fmt --check` clean.
4. Merge PR #258 after review approval and CI green.
5. (Post-merge) Compute and update `input-hash:` in this story file via
   `bin/compute-input-hash --write .factory/stories/STORY-116.md`.

No `src/` changes. No `todo!()` stubs. This is a test-verification story.

## Test Plan

| AC | Test | Type | File |
|----|------|------|------|
| AC-001 | `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11` | Integration (decoder) | `tests/bc_2_16_qinq_macsec_offset_tests.rs` |
| AC-002 | `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11` | Integration (decoder) | `tests/bc_2_16_qinq_macsec_offset_tests.rs` |
| AC-003 | `test_BC_2_16_015_qinq_link_exts_offset_formula_pin` | Unit (etherparse model pin) | `tests/bc_2_16_qinq_macsec_offset_tests.rs` |
| AC-004 | `test_BC_2_16_015_macsec_arp_lax_parse_probe` | Integration (observe-only) | `tests/bc_2_16_qinq_macsec_offset_tests.rs` |

## Previous Story Intelligence

STORY-115 (last story in E-16) completed the ARP Security Analyzer for v0.7.0:
- D3 storm detection, `--arp-storm-rate` CLI flag, `storm_findings` summary key.
- `src/analyzer/arp.rs` is fully implemented; `ArpAnalyzer::record_malformed()` is available.
- `src/decoder.rs` QinQ offset formula (`14 + Σ link_exts.header_len()`) was shipped in E-16
  but only the single-VLAN case was previously regression-tested.
- etherparse 0.20.2 is the pinned version (from `Cargo.toml`); no upgrade in E-17.

**Key lesson from E-16:** The lax-path ARP offset formula is already correct for QinQ. This
story adds regression tests to prevent future regressions; it does not fix a defect.

## Architecture Compliance Rules

Derived from `arp-architecture-delta.md` §2.2 and BC-2.16.015 v1.9 PC-7a/7b:

1. **Offset formula is `14 + Σ link_exts.header_len()`** — not hardcoded per tag type.
   The formula generalizes over any `LaxLinkExtSlice` chain. Tests must verify the sum,
   not a hardcoded constant.
2. **QinQ is two separate `Vlan` entries in etherparse 0.20.2** — no `VlanDouble` variant.
   AC-003 / `test_BC_2_16_015_qinq_link_exts_offset_formula_pin` is the regression guard.
3. **D11 quality parity** — QinQ D11 findings must have `category == Anomaly` and
   `mitre_techniques: []` (same rules as D-078 single-VLAN tests).
4. **MACsec probe is observe-only in this file** — offset assertions for MACsec belong
   exclusively to `tests/bc_2_16_e17_macsec_offset_tests.rs` (STORY-117).
5. **VP-024 is LOCKED (v2.4)** — this story appends a lifecycle note to VP-024 to record
   that QinQ tests cover the lax-path offset formula that VP-024 Sub-A does not directly
   exercise. No proof-level change. Do NOT modify VP-024 proof content.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `etherparse` | 0.20.2 | QinQ represented as two Vlan link-exts; `LaxLinkExtSlice::Vlan::header_len() == 4`; no `VlanDouble` variant. Re-verify on any bump. |
| `pcap_file` | same as existing | `DataLink::ETHERNET` for test fixture decoding |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` | **Deliver as-is** (already on PR #258 branch) | 4 tests: AC-001/002/003/004 |
| `src/decoder.rs` | No change | Offset formula already correct |
| `src/analyzer/arp.rs` | No change | `record_malformed()` already implemented |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~3,500 |
| BC files (2 BCs: BC-2.16.009 v1.10, BC-2.16.015 v1.9) | ~6,000 |
| F1 delta analysis §2.1 + §6 | ~2,000 |
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` (4 tests) | ~3,000 |
| `src/decoder.rs` lax-path (reference read) | ~1,500 |
| Tool outputs (cargo test for 4 tests) | ~500 |
| **Total estimated** | **~16,500** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-115]` — STORY-115 is the last E-16 story and ships the complete
  `ArpAnalyzer` including `record_malformed()`. STORY-116 test harness calls
  `ArpAnalyzer::record_malformed()` directly; that method must exist before the tests can
  compile. STORY-116 is blocked on E-16 completion (v0.7.0).
- `blocks: [STORY-117]` — STORY-117 depends on the same PR branch (#258) and its MACsec
  offset assertion tests reference the same ARP lax-path formula being verified here. The
  logical sequencing (QinQ first, MACsec second) matches BC authoring order and F1 §6 recommendation.
