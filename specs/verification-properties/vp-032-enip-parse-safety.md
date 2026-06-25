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

2. **Sub-B — Command classification biconditional (non-vacuous):** For all 65,536 possible
   `u16` values, `classify_enip_command(cmd) == Unknown` if and only if `cmd` is not a
   member of `KNOWN_COMMANDS`. This biconditional simultaneously proves totality, Unknown
   reachability, and named-variant reachability without separate flip proofs (DF-KANI-NONVACUITY-001).

3. **Sub-C — Validity gate biconditional:** `is_valid_enip_frame(h: &EnipHeader)` returns
   `true` if and only if `h.command` is a member of the known-command set
   {0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075}. The
   biconditional holds for all possible `u16` command values.

4. **Sub-D — CIP service classification totality (strengthened):** Two harnesses:
   (a) Primary biconditional — for all 256 `u8` inputs, `classify_cip_service(service)
   == Response` iff `service & 0x80 != 0`. (b) Request-range partition — over the request
   range 0x00..=0x7F, every service byte is either a named CIP service or `Unknown`;
   the named-vs-Unknown partition is exhaustive and correct, proving the `Unknown` arm is
   non-vacuous (DF-KANI-NONVACUITY-001).

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.17.001 | `parse_enip_header` returns `None` for `data.len() < 24` | Sub-A |
| BC-2.17.002 | `EnipHeader` field contracts — fixed little-endian offsets | Sub-A |
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

**Out-of-scope note (F-P9-002):** `parse_cip_header` and `parse_cpf_items` are NOT Kani
targets in VP-032 — VP-032 Sub-A/B/C/D cover only the four functions enumerated above.
Both `parse_cip_header` and `parse_cpf_items` are attacker-facing length-driven parsers
with an F6 cargo-fuzz no-panic / bounds-safety obligation analogous to VP-028 (pcapng
reader fuzz). No new VP number is required. See ADR-010 Decision 8 DEFERRED list and
enip-architecture-delta.md §4.3 for the authoritative fuzz obligation record.

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
            // Field offset verification: command is at bytes [0..2] LITTLE-endian (ODVA).
            // F-ENIP-001: ENIP encapsulation header is little-endian throughout.
            let expected_cmd = u16::from_le_bytes([slice[0], slice[1]]);
            assert_eq!(h.command, expected_cmd);
            // length field at bytes [2..4] little-endian.
            let expected_len = u16::from_le_bytes([slice[2], slice[3]]);
            assert_eq!(h.length, expected_len);
            // status at bytes [8..12] little-endian.
            let expected_status = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]);
            assert_eq!(h.status, expected_status);
        }
    }
}
```

### Sub-B: Command Classification Biconditional (non-vacuous)

```rust
    /// VP-032 Sub-B: classify_enip_command(cmd) == Unknown iff cmd is not in KNOWN_COMMANDS.
    ///
    /// BOUND/SOUNDNESS: Symbolic u16 covers the full domain. Non-vacuity:
    /// the biconditional simultaneously proves both arms — Unknown reachable (any cmd
    /// not in KNOWN_COMMANDS) and non-Unknown reachable (any known cmd). No kani::assume
    /// on cmd; both branches are reachable by Kani without any constraint.
    ///
    /// Mirrors VP-032 Sub-C biconditional structure (DF-KANI-NONVACUITY-001).
    #[kani::proof]
    fn vp032_enip_command_classification_biconditional() {
        const KNOWN_COMMANDS: &[u16] = &[
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
        let cmd: u16 = kani::any();
        let is_unknown = matches!(classify_enip_command(cmd), EnipCommandClass::Unknown);
        let not_in_known = !KNOWN_COMMANDS.contains(&cmd);
        // Biconditional: Unknown iff not in known set. Mirrors Sub-C structure.
        assert_eq!(is_unknown, not_in_known);
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

### Sub-D: CIP Service Classification Totality (strengthened)

```rust
    /// VP-032 Sub-D (primary): classify_cip_service is total over all 256 u8 values;
    /// high bit (0x80) correctly maps to Response for all inputs in 0x80..=0xFF.
    ///
    /// BOUND/SOUNDNESS: Symbolic u8 covers the full domain (256 inputs).
    /// Non-vacuity: Both Response (0xFF) and non-Response (0x01) arms reachable
    /// without any kani::assume constraints.
    #[kani::proof]
    fn vp032_cip_service_classification_totality() {
        let service: u8 = kani::any();
        let class = classify_cip_service(service);
        // Response-bit invariant: high bit set => Response variant (and vice-versa).
        // Biconditional over the full 256-value domain.
        let is_response = matches!(class, CipServiceClass::Response);
        assert_eq!(is_response, service & 0x80 != 0);
    }

    /// VP-032 Sub-D (partition): over the request range 0x00..=0x7F, every service byte
    /// is either a named CIP service OR Unknown — the named-vs-Unknown partition is
    /// exhaustive and correct. This proves that the Unknown arm is non-vacuous (reachable)
    /// and that no named variant is accidentally unreachable within the request range.
    ///
    /// BOUND/SOUNDNESS: Constrained to 0x00..=0x7F (request range). Response arm is
    /// excluded from this proof (already covered by the primary harness above).
    #[kani::proof]
    fn vp032_cip_service_request_partition() {
        const NAMED_SERVICES: &[u8] = &[
            0x01, // GetAttributesAll
            0x02, // SetAttributesAll
            0x03, // GetAttributeList
            0x04, // SetAttributeList
            0x05, // Reset
            0x07, // Stop (Change Operating Mode)
            0x0A, // MultipleServicePacket
            0x0E, // GetAttributeSingle
            0x10, // SetAttributeSingle
            0x4B, // GetAndClear
            0x4E, // ForwardClose
            0x54, // ForwardOpen
            0x5B, // LargeForwardOpen
        ];
        let service: u8 = kani::any();
        // Restrict to request range (high bit clear).
        kani::assume(service & 0x80 == 0);
        let class = classify_cip_service(service);
        // Must NOT be Response (high bit clear means request).
        assert!(!matches!(class, CipServiceClass::Response));
        // Must be either a named service or Unknown.
        let is_named = NAMED_SERVICES.contains(&service);
        let is_unknown = matches!(class, CipServiceClass::Unknown);
        // Named iff not Unknown — exhaustive partition over 0x00..=0x7F.
        assert_eq!(is_named, !is_unknown);
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
   closely mirrors VP-022's `verify_parse_mbap_header_safety`; Sub-B mirrors the
   Sub-C biconditional structure; Sub-C mirrors VP-023's `verify_is_valid_dnp3_frame_gate`;
   Sub-D (primary + partition) extends the DNP3 pattern with a range-constrained proof.

**Unwind bounds:**
- Sub-A: `#[kani::unwind(49)]` (slice length up to 48 bytes)
- Sub-B, Sub-C, Sub-D (both harnesses): No explicit unwind needed — no loops

**Non-vacuity strategy (DF-KANI-NONVACUITY-001):** Sub-B and Sub-C use biconditional
proofs (`assert_eq!(is_X, expected_bool)`) that simultaneously cover both arms without
separate flip proofs — the biconditional fails if either arm is vacuous (all inputs
taking one path). Sub-D uses a primary biconditional (Response vs. non-Response) plus a
range-constrained partition proof (0x00..=0x7F) to prove the named-vs-Unknown boundary.
Sub-A's non-vacuity is implicit: the unwind(49) bound covers both len<24 (returns None)
and len>=24 (returns Some) branches in the same proof.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-032 produced, added to VP-INDEX | draft |
| F4 (TDD implementation) | Proof harnesses authored in `src/analyzer/enip.rs #[cfg(kani)]` | draft → active |
| F6 (formal hardening) | `cargo kani` run: all 5 harnesses VERIFICATION SUCCESSFUL | active → verified |

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
