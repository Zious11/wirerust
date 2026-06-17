---
document_type: story
story_id: STORY-117
epic_id: E-17
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-16T00:00:00Z
phase: f3
points: 5
priority: P1
depends_on: [STORY-116]
blocks: []
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
wave: 46
# BC status: BC-2.16.009 v1.10 (EC-009 MACsec documented-limitation), BC-2.16.015 v1.9 (EC-009)
#             — authored 2026-06-16.
# tdd_mode: facade — this story delivers existing test files (no production code change);
#   delivery = merge PR #258 (test/arp-qinq-macsec-fixtures) after CI green. No todo!() stubs.
# Subsystem anchor: SS-16 owns this story's scope because MACsec ARP offset coverage
#   is part of the ARP decoder lax-path, which is an SS-16 / src/decoder.rs concern
#   per ARCH-INDEX Subsystem Registry.
inputs:
  - .factory/phase-f1-delta-analysis/e17-arp-qinq-macsec-offset-delta-analysis.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
input-hash: "c389b39"
---

# STORY-117: ARP MACsec Offset Documented-Limitation Coverage

## Narrative

- **As a** ICS/OT security analyst using wirerust on MACsec-protected networks
- **I want** the ARP decoder's behavior for MACsec-tagged frames to be formally verified
  by offset-assertion tests (no-SCI offset==22, SCI-present offset==30), D11 routing tests,
  and security-property guards (Modified/Encrypted payloads never reach the ARP truncation
  path), with a BC-documented limitation for real-traffic unverifiability
- **So that** the MACsec correctness boundary is explicitly recorded in behavioral contracts
  and regression-guarded against future etherparse version changes or SCI accounting bugs

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.009 | D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding |
| BC-2.16.015 | Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced; Analysis Gated on --arp |

## Delivery Context

This story formalizes the AC-to-test-to-BC traceability for 6 tests that **already exist**
on branch `test/arp-qinq-macsec-fixtures` (PR #258):
`tests/bc_2_16_e17_macsec_offset_tests.rs`.

**F4 implementation = merge PR #258 after CI green.** No `src/` production code change.
No `todo!()` stubs. The `tdd_mode: facade` flag reflects this: the tests were written as
synthetic assertion probes for the already-correct MACsec offset formula, not as TDD
drivers for new production code.

**Documented limitation (EC-009(c)):** No public on-wire MACsec-over-ARP pcap exists.
Offset arithmetic is proven by etherparse source, upstream proptest, and the 4 synthetic
offset/D11 tests in this story. What remains unverified is solely the existence and behavior
of MACsec-over-ARP in real captured traffic. No code change is planned until a failing
real-world test demonstrates a defect.

## Acceptance Criteria

### AC-001 (traces to BC-2.16.015 EC-009(a) — MACsec no-SCI Unmodified offset 22, benign truncation MUST NOT produce D11)
A MACsec Unmodified/no-SCI frame (EtherType 0x88E5, `header_len() == 8`) carrying a
benign truncated ARP payload (htype=0x0001, hlen=6, no variable section) yields
`decode_packet` result `Err("truncated ARP frame")` and `malformed_findings == 0`.

The computed `arp_offset = 14 + 8 = 22` (BC-2.16.015 v1.9 EC-009 documented value).
`Σ link_exts.header_len() == 8` confirms the no-SCI formula. The ARP bytes at offset 22
have `htype == 0x0001` and `hlen == 6`, so genuine truncation is correctly identified.

- **Test:** `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

### AC-002 (traces to BC-2.16.009 Precondition 3 / Postcondition 1 / EC-009(a) — MACsec no-SCI Unmodified malformed hlen=8 MUST route to D11)
A MACsec Unmodified/no-SCI frame carrying an ARP fixed header with `hlen=8` at offset 22
produces `Err("Non-Ethernet/IPv4 ARP frame")`, `malformed_findings >= 1`, D11 finding has
`category == Anomaly`, `mitre_techniques: []`.

- **Test:** `test_BC_2_16_009_macsec_no_sci_unmodified_arp_malformed_hlen8_routes_to_d11`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

### AC-003 (traces to BC-2.16.015 EC-009(a) — MACsec SCI-present Unmodified offset 30, benign truncation MUST NOT produce D11)
A MACsec Unmodified/SCI-present frame (`sci = Some(u64)`, `header_len() == 16`) carrying a
benign truncated ARP payload yields `Err("truncated ARP frame")` and `malformed_findings == 0`.

The computed `arp_offset = 14 + 16 = 30` (BC-2.16.015 v1.9 EC-009 documented value).
`Σ link_exts.header_len() == 16` confirms the SCI-present formula (6 SecTag + 8 SCI +
2 next-EtherType). The 8 SCI bytes at frame[22..30] do NOT read as `htype == 0x0001`
(off-by-8 guard): if `header_len()` returned 8 instead of 16, the decoder would read SCI
bytes as ARP and produce a false D11.

**This is the spec-backing test for BC-2.16.015 v1.9 EC-009 offset=30.**

- **Test:** `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

### AC-004 (traces to BC-2.16.009 Precondition 3 / Postcondition 1 / EC-009(a) — MACsec SCI-present Unmodified malformed hlen=8 MUST route to D11)
A MACsec Unmodified/SCI-present frame carrying an ARP fixed header with `hlen=8` at offset 30
produces `Err("Non-Ethernet/IPv4 ARP frame")`, `malformed_findings >= 1`, D11 finding has
`category == Anomaly`, `mitre_techniques: []`.

- **Test:** `test_BC_2_16_009_macsec_sci_present_unmodified_arp_malformed_hlen8_routes_to_d11`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

### AC-005 (traces to BC-2.16.015 EC-009(b) — Modified/no-SCI opaque payload MUST NOT reach Layer::Arp)
A MACsec Modified/no-SCI frame (no inner EtherType, opaque payload) does NOT produce
`lax.stop_err == Layer::Arp`. The `LaxMacsecPayloadSlice::Modified { .. }` variant is
confirmed, and the lax cursor does not advance to an inner ARP layer.
`Σ link_exts.header_len() == 6` (SecTag only, no next_EtherType).

This is a **security-property guard**: if etherparse changed its behavior to expose opaque
ciphertext as a readable ARP layer, the decoder would attempt to classify ciphertext as ARP
fields. This test guards that invariant.

- **Test:** `test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

### AC-006 (traces to BC-2.16.015 EC-009(b) — Modified/SCI-present opaque payload MUST NOT reach Layer::Arp)
A MACsec Modified/SCI-present frame (`sci = Some(u64)`, `header_len() == 14`, opaque payload)
does NOT produce `lax.stop_err == Layer::Arp`. Same security property as AC-005 for the
SCI-present variant. `Σ link_exts.header_len() == 14` (6 SecTag + 8 SCI, no next_EtherType).

- **Test:** `test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable`
  (in `tests/bc_2_16_e17_macsec_offset_tests.rs`)

## Documented Limitation (EC-009(c))

The following limitation is explicitly recorded in BC-2.16.009 v1.10 EC-009(c) and
BC-2.16.015 v1.9 EC-009(c):

> No public on-wire MACsec-over-ARP PCAP capture exists (deep web sweep: Wireshark
> SampleCaptures wiki, packetlife, cloudshark, GitHub fixtures — none carry Unmodified
> MACsec with inner ARP). What remains unverified is solely the existence and behavior
> of MACsec-over-ARP in real captured traffic. MACsec decapsulation commonly occurs at
> the NIC before pcap capture, so MACsec-tagged frames may not appear in practice. This
> boundary is DOCUMENTED-UNVERIFIED; no code change is planned until a failing real-world
> test demonstrates a defect.

This is not a correctness gap — the offset formula is structurally proven by etherparse
source (`macsec_header_slice.rs:246-248`), upstream proptest (`macsec_header.rs:340-347`),
and the 4 synthetic offset/D11 tests in this story (AC-001 through AC-004).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `decode_packet` lax None arm — MACsec offset formula | `src/decoder.rs` | Pure core (reads bytes) |
| `LaxLinkExtSlice::Macsec::header_len()` delegation | etherparse 0.20.2 (external) | Pure (arithmetic) |
| `LaxMacsecPayloadSlice::Modified` variant check | etherparse 0.20.2 (external) | Pure (pattern match) |
| `ArpAnalyzer::record_malformed` | `src/analyzer/arp.rs` | Pure core (stateful) |

Architecture section references: `architecture/module-decomposition.md` (SS-16 C-23, `src/decoder.rs`
lax-path ARP routing); `arp-architecture-delta.md` §2.2 (MACsec offset formula, EC-009).

## Forbidden Dependencies

- `tests/bc_2_16_e17_macsec_offset_tests.rs` MUST NOT assert offset correctness using the
  observe-only probe in `tests/bc_2_16_qinq_macsec_offset_tests.rs`. The e17 file contains
  its own fixture builders and directly asserts `arp_offset == 22` (no-SCI) and `arp_offset == 30`
  (SCI-present). The qinq file's MACsec probe is intentionally observe-only.
- No `src/` production code changes are permitted in this story. Any PR touching `src/decoder.rs`
  or `src/analyzer/arp.rs` for this story indicates scope creep that violates E-17 F1 scope decision.
- AC-005 and AC-006 MUST match on `LaxMacsecPayloadSlice::Modified { .. }`, NOT on `LaxMacsecPayloadSlice::Unmodified`. A test that uses the wrong variant silently passes without proving the security property.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | MACsec no-SCI Unmodified, benign ARP (hlen=6) at offset 22 | `"truncated ARP frame"`, no D11 — AC-001 |
| EC-002 | MACsec no-SCI Unmodified, malformed ARP (hlen=8) at offset 22 | `"Non-Ethernet/IPv4 ARP frame"`, D11 — AC-002 |
| EC-003 | MACsec SCI-present Unmodified, benign ARP (hlen=6) at offset 30 | `"truncated ARP frame"`, no D11 — AC-003; off-by-8 guard |
| EC-004 | MACsec SCI-present Unmodified, malformed ARP (hlen=8) at offset 30 | D11 fires at correct offset — AC-004 |
| EC-005 | MACsec no-SCI Modified (opaque payload) | `stop_err != Layer::Arp`; ARP path unreachable — AC-005 |
| EC-006 | MACsec SCI-present Modified (opaque payload) | `stop_err != Layer::Arp`; ARP path unreachable — AC-006 |
| EC-007 | Real-world MACsec+ARP traffic | DOCUMENTED-UNVERIFIED (EC-009(c)); no pcap fixture available |

## Tasks

1. Confirm the 6 tests in `tests/bc_2_16_e17_macsec_offset_tests.rs` are passing on
   branch `test/arp-qinq-macsec-fixtures`.
2. Confirm `cargo test --all-targets` green for all 10 new tests (4 from STORY-116 + 6
   from this story) and all existing tests remain green.
3. Confirm `cargo clippy --all-targets -- -D warnings` clean.
4. Confirm `cargo fmt --check` clean.
5. Merge PR #258 after review approval and CI green (this is the shared delivery with STORY-116).
6. (Post-merge) ~~Update `arp-architecture-delta.md` to v1.18~~ — NO-OP: arch-delta is already
   at v1.19; the E-17 MACsec documented-limitation changelog entry was recorded in the F2/F3
   backlink burst. No further update required.
7. (Post-merge) Compute and update `input-hash:` in this story file via
   `bin/compute-input-hash --write .factory/stories/STORY-117.md`.

No `src/` changes. No `todo!()` stubs. This is a test-verification + BC-text story.

## Test Plan

| AC | Test | Type | File |
|----|------|------|------|
| AC-001 | `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` | Integration (decoder) | `tests/bc_2_16_e17_macsec_offset_tests.rs` |
| AC-002 | `test_BC_2_16_009_macsec_no_sci_unmodified_arp_malformed_hlen8_routes_to_d11` | Integration (decoder) | `tests/bc_2_16_e17_macsec_offset_tests.rs` |
| AC-003 | `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` | Integration (decoder; spec-backing test) | `tests/bc_2_16_e17_macsec_offset_tests.rs` |
| AC-004 | `test_BC_2_16_009_macsec_sci_present_unmodified_arp_malformed_hlen8_routes_to_d11` | Integration (decoder) | `tests/bc_2_16_e17_macsec_offset_tests.rs` |
| AC-005 | `test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable` | Security-property guard | `tests/bc_2_16_e17_macsec_offset_tests.rs` |
| AC-006 | `test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable` | Security-property guard | `tests/bc_2_16_e17_macsec_offset_tests.rs` |

## Previous Story Intelligence

STORY-116 (predecessor in E-17) established:
- QinQ offset coverage is complete (4 tests in `bc_2_16_qinq_macsec_offset_tests.rs` passing).
- The observe-only MACsec probe (`test_BC_2_16_015_macsec_arp_lax_parse_probe`) confirms
  `MacsecHeader::header_len() == 8` for no-SCI, but does NOT assert an offset value.
- The shared PR #258 branch carries both test files; CI gate covers all 10 tests together.

**Key lesson from the E-17 F3 decomposition:** The MACsec correctness question was the
central adjudication of E-17 F1. The outcome is DOCUMENTED-LIMITATION (no code change).
This story's AC-003 is the most critical test — it guards against the off-by-8 SCI
accounting risk and empirically backs the BC-2.16.015 v1.9 EC-009 offset=30 claim.

**No MITRE tagging:** D11 findings from MACsec-framed malformed ARP carry
`mitre_techniques: []` for the same reason as all D11 findings — DF-VALIDATION-001
requires live validation before attaching T0814 or any other technique.

## Architecture Compliance Rules

Derived from `arp-architecture-delta.md` §2.2 and BC-2.16.009 v1.10 / BC-2.16.015 v1.9 EC-009:

1. **Offset formula `14 + Σ header_len()` is correct for all reachable MACsec variants** —
   no-SCI Unmodified: `header_len() == 8` (6 SecTag + 2 next-EtherType); SCI-present Unmodified:
   `header_len() == 16` (6 SecTag + 8 SCI + 2 next-EtherType). The SCI bytes ARE included per
   etherparse `macsec_header_slice.rs:246-248`. Verify on any etherparse version bump.
2. **Modified/Encrypted MACsec is safe by construction** — `LaxMacsecPayloadSlice::Modified`
   causes the lax driver to execute `return result` before any inner-ARP parse block.
   `stop_err == Layer::Arp` is unreachable for these variants. AC-005 and AC-006 guard this.
3. **D11 quality parity** — MACsec D11 findings must have `category == Anomaly` and
   `mitre_techniques: []` (same rules as all other D11/malformed tests).
4. **VP-024 is LOCKED (v2.4)** — lifecycle append note only. The Kani harnesses are not
   re-run (no `src/decoder.rs` change). VP-024 LOCKED status is not affected.
5. **etherparse line numbers are volatile** — references to `macsec_header_slice.rs:246-248`
   and `lax_packet_headers.rs:364-373` are for citation only. Tests guard runtime behavior
   via `header_len()` assertions, not source-line assertions.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `etherparse` | 0.20.2 | `MacsecHeader::header_len()`: 8 (no-SCI Unmodified), 16 (SCI Unmodified), 6 (no-SCI Modified), 14 (SCI Modified). Re-verify on any bump. |
| `pcap_file` | same as existing | `DataLink::ETHERNET` for test fixture decoding |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/bc_2_16_e17_macsec_offset_tests.rs` | **Deliver as-is** (already on PR #258 branch) | 6 tests: AC-001 through AC-006 |
| `src/decoder.rs` | No change | MACsec offset formula already correct |
| `src/analyzer/arp.rs` | No change | `record_malformed()` already implemented |
| `arp-architecture-delta.md` | No change (already at v1.19 — E-17 entry recorded in F3 backlink burst) | NO-OP |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (2 BCs: BC-2.16.009 v1.10, BC-2.16.015 v1.9) | ~6,000 |
| F1 delta analysis §3 + §6 (MACsec adjudication + story preview) | ~3,000 |
| `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests) | ~5,000 |
| `src/decoder.rs` lax-path (reference read, MACsec section) | ~1,500 |
| Tool outputs (cargo test for 6 tests) | ~500 |
| **Total estimated** | **~20,000** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-116]` — STORY-116 covers QinQ and includes the observe-only MACsec
  probe that confirms `MacsecHeader::header_len() == 8`. STORY-117 builds on this foundation
  with full offset-assertion tests. The two stories share PR #258; logical sequencing
  (QinQ first) matches F1 §6 recommendation. Both stories must be present before the PR
  is merged to ensure the complete 10-test suite is reviewed together.
- `blocks: []` — STORY-117 is the final story in E-17. No downstream E-17 stories depend
  on it. Phase-F5 scoped adversarial review follows STORY-117's merge.
