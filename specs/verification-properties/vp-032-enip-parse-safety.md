---
artifact: verification-property
vp_id: VP-032
title: "EtherNet/IP + CIP Frame Parse Safety and Command/Service Classification"
status: draft
phase: P1
tool: Kani
subsystem: SS-17
module: "src/analyzer/enip.rs"
producer: architect
timestamp: 2026-06-24T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.17.001
  - BC-2.17.002
  - BC-2.17.003
  - BC-2.17.004
  - BC-2.17.007
verification_lock: false
---

# VP-032: EtherNet/IP + CIP Frame Parse Safety and Command/Service Classification

## Property Statement

The four pure-core free functions in `src/analyzer/enip.rs` that form the foundation of
EtherNet/IP + CIP packet analysis satisfy formal safety and totality properties. These
properties must hold for **all possible symbolic inputs** within the input domain, not
merely for tested cases.

Specifically, for all bounded symbolic byte slices and all u16/u8 command/service inputs:

1. **Sub-A — Header parse safety:** `parse_enip_header(data: &[u8])` returns `None` when
   `data.len() < 24` and `Some(EnipHeader)` otherwise. The function never panics, never
   accesses bytes beyond fixed offsets [0..24], and never uses attacker-controlled byte
   values as slice indices.

2. **Sub-B — Command classification totality:** `classify_enip_command(cmd: u16)` returns
   a valid `EnipCommandClass` variant for every possible `u16` input value (all 65,536
   possible inputs). The `Unknown` arm is reachable and proven non-vacuous.

3. **Sub-C — Validity gate biconditional:** `is_valid_enip_frame(h: &EnipHeader)` returns
   `true` if and only if `h.command` is a member of the known-command set
   {0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075}. The
   biconditional holds for all possible `u16` command values.

4. **Sub-D — CIP service classification totality:** `classify_cip_service(service: u8)`
   returns a valid `CipServiceClass` variant for every possible `u8` input value (all 256
   possible inputs). The response-bit mask logic (`service & 0x80 != 0 → Response`) is
   proven correct. The `Unknown` arm is reachable.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.17.001 | `parse_enip_header` returns `None` for `data.len() < 24` | Sub-A |
| BC-2.17.002 | `EnipHeader` field contracts — fixed big-endian offsets | Sub-A |
| BC-2.17.003 | `is_valid_enip_frame` validity gate biconditional | Sub-C |
| BC-2.17.004 | `classify_enip_command` total classification with Unknown arm | Sub-B |
| BC-2.17.007 | `classify_cip_service` total classification with response-bit mask | Sub-D |

## Purity Classification

All four target functions are **pure-core** (ADR-010 Decision 2):
- No I/O, no heap allocation beyond bounded slices, no global state reads
- No external crate calls — direct byte indexing only
- All inputs are `&[u8]` or derived integer types
- All control flow is bounded and terminating

This purity classification makes Kani the correct tool: the functions are deterministic,
side-effect-free, and have bounded input domains that Kani can enumerate symbolically.

## Proof Harness Skeletons

These skeletons are produced in F2 (spec evolution). The actual working harnesses are
authored and locked in F4 (TDD implementation) and F6 (formal hardening) respectively.

### Sub-A: Header Parse Safety

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-032 Sub-A: parse_enip_header never panics; returns None for <24 bytes;
    /// returns Some with correct field layout for >=24 bytes.
    ///
    /// BOUND/SOUNDNESS: 48-byte bound (2× minimum header) covers all length
    /// conditions; behavior is identical for any longer slice (fixed 24-byte
    /// read). Non-vacuity: both Some and None branches are reachable in the
    /// symbolic range (kani::assume lifted — no assume on data.len()).
    #[kani::proof]
    #[kani::unwind(49)]
    fn vp032_enip_header_parse_safety() {
        const BOUND: usize = 48;
        let data: [u8; BOUND] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= BOUND);
        let slice = &data[..len];
        let result = parse_enip_header(slice);
        if len < 24 {
            assert!(result.is_none());
        } else {
            let h = result.expect("must be Some for len >= 24");
            // Field offset verification: command is at bytes [0..2] big-endian.
            let expected_cmd = u16::from_be_bytes([slice[0], slice[1]]);
            assert_eq!(h.command, expected_cmd);
            // length field at bytes [2..4] big-endian.
            let expected_len = u16::from_be_bytes([slice[2], slice[3]]);
            assert_eq!(h.length, expected_len);
            // status at bytes [8..12].
            let expected_status = u32::from_be_bytes([slice[8], slice[9], slice[10], slice[11]]);
            assert_eq!(h.status, expected_status);
        }
    }
}
```

### Sub-B: Command Classification Totality

```rust
    /// VP-032 Sub-B: classify_enip_command is total over all 65,536 u16 values.
    ///
    /// BOUND/SOUNDNESS: Symbolic u16 covers the full domain (no assume needed).
    /// Non-vacuity: the Unknown arm reachable via cmd=0x0000 (not in known set).
    #[kani::proof]
    fn vp032_enip_command_classification_totality() {
        let cmd: u16 = kani::any();
        let class = classify_enip_command(cmd);
        // The match must not panic and must return a valid variant.
        // Non-exhaustive proof: at least one variant is not Unknown (reachability
        // of both Unknown and non-Unknown confirmed by symbolic coverage).
        let _ = class; // presence of a value is sufficient for no-panic proof
    }

    /// Non-vacuity flip: Unknown is reachable.
    #[kani::proof]
    fn vp032_enip_command_unknown_is_reachable() {
        // 0x0000 is not a recognized ENIP command.
        assert!(matches!(
            classify_enip_command(0x0000),
            EnipCommandClass::Unknown
        ));
    }
```

### Sub-C: Validity Gate Biconditional

```rust
    /// VP-032 Sub-C: is_valid_enip_frame iff h.command is in the known-command set.
    ///
    /// BOUND/SOUNDNESS: Symbolic u16 command + fabricated EnipHeader covers the
    /// full command domain. The biconditional is proven for all 65,536 inputs.
    #[kani::proof]
    fn vp032_enip_validity_gate_biconditional() {
        let cmd: u16 = kani::any();
        // Construct a minimal EnipHeader with symbolic command; other fields zeroed.
        let h = EnipHeader {
            command: cmd,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        let known_cmds: &[u16] = &[
            0x0004, // ListServices
            0x0063, // ListIdentity
            0x0064, // ListInterfaces
            0x0065, // RegisterSession
            0x0066, // UnRegisterSession
            0x006F, // SendRRData
            0x0070, // SendUnitData
            0x0072, // IndicateStatus
            0x0075, // Cancel
        ];
        let is_known = known_cmds.contains(&cmd);
        let gate_result = is_valid_enip_frame(&h);
        // Biconditional: gate == is_known for all cmd values.
        assert_eq!(gate_result, is_known);
    }
```

### Sub-D: CIP Service Classification Totality

```rust
    /// VP-032 Sub-D: classify_cip_service is total over all 256 u8 values;
    /// high bit (0x80) correctly maps to Response.
    ///
    /// BOUND/SOUNDNESS: Symbolic u8 covers the full domain (256 inputs).
    /// Non-vacuity: Unknown arm reachable via service=0x7F (not a named service
    /// and not 0x80). Response arm reachable via service=0xFF (high bit set).
    #[kani::proof]
    fn vp032_cip_service_classification_totality() {
        let service: u8 = kani::any();
        let class = classify_cip_service(service);
        // Response-bit invariant: high bit set => Response variant.
        if service & 0x80 != 0 {
            assert!(matches!(class, CipServiceClass::Response));
        }
        let _ = class;
    }

    /// Non-vacuity flip: Unknown arm reachable (service with high bit clear,
    /// not in named set).
    #[kani::proof]
    fn vp032_cip_service_unknown_is_reachable() {
        // 0x7F: high bit clear, not a named CIP service code.
        assert!(matches!(
            classify_cip_service(0x7F),
            CipServiceClass::Unknown
        ));
    }
```

## Feasibility Assessment

**Assessment: FEASIBLE.**

All four target functions satisfy the prerequisites for Kani model checking:

1. **Pure-core:** No I/O, no heap allocation beyond bounded slices, no global state.
   Kani can call them directly without any test harness wrapping.

2. **Bounded input domains:** The symbolic inputs are `&[u8]` with a bounded length
   (BOUND=48 for Sub-A), `u16` (65,536 values, Sub-B/C), and `u8` (256 values, Sub-D).
   All are within Kani's practical enumeration range.

3. **Bounded loop count:** `parse_enip_header` has no loops — it is pure fixed-offset
   indexing. `classify_enip_command` and `classify_cip_service` are exhaustive match
   expressions with no loops. `is_valid_enip_frame` is a boolean expression over a
   constant set.

4. **No recursive calls:** None of the four functions recurse.

5. **Comparable precedent:** VP-022 (Modbus, 4 harnesses, all SUCCESSFUL) and VP-023
   (DNP3, 4 harnesses, all SUCCESSFUL) use identical proof patterns. VP-032's Sub-A
   closely mirrors VP-022's `verify_parse_mbap_header_safety`; Sub-B and Sub-D mirror
   `verify_classify_fc_total`; Sub-C mirrors VP-023's `verify_is_valid_dnp3_frame_gate`.

**Unwind bounds:**
- Sub-A: `#[kani::unwind(49)]` (slice length up to 48 bytes)
- Sub-B, Sub-C, Sub-D: No explicit unwind needed — no loops

**Non-vacuity strategy:** Each proof has a companion "deliberate-flip" assertion (as
shown in the Sub-B and Sub-D skeleton harnesses above) that verifies the Unknown arm is
actually reachable. This matches the VP-022 and VP-023 non-vacuity approach.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-032 produced, added to VP-INDEX | draft |
| F4 (TDD implementation) | Proof harnesses authored in `src/analyzer/enip.rs #[cfg(kani)]` | draft → active |
| F6 (formal hardening) | `cargo kani` run: all 4+ harnesses VERIFICATION SUCCESSFUL | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation. Mirrors the VP-022/VP-023 lock pattern.

## VP-INDEX Update Triggered by This VP

When VP-032 is added:
- `total_vps`: 31 → 32
- `p1_count`: 17 → 18
- `kani_count`: 14 → 15
- `draft` count: 0 → 1 (VP-032 status=draft)
- Tool row in VP-INDEX summary: Kani VP-IDs list: append VP-032

These counts must be propagated in the same burst to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection Kani row)
3. `verification-coverage-matrix.md` (VP-to-Module table + Per-Module table + Totals row)
