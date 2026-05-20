---
artifact: L2-ent-05
traces_to: ../domain-spec.md
title: Entities -- Semantic Enums and Value Objects
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# Entities: Semantic Enums and Value Objects

This shard consolidates the 14 semantic enums and 12 value objects (VO-1..VO-12) from the
domain model. Source: pass-2-domain-model.md sections 2 and 4.

## 14 Semantic Enums

| Enum | File | Variants | #[non_exhaustive]? | Notes |
|---|---|---|---|---|
| Protocol (E-6) | decoder.rs | Tcp, Udp, Icmp, Other(u8) | No | Other carries raw IP-proto byte |
| TransportInfo (E-7) | decoder.rs | Tcp{..}, Udp{..}, None | No | Not Serialize |
| Verdict (E-23) | findings.rs | Likely, Unlikely, Inconclusive | YES | P2.10 / #76 |
| Confidence (E-24) | findings.rs | High, Medium, Low | YES | P2.10 / #76 |
| ThreatCategory (E-25) | findings.rs | 8 variants | YES | P2.10 / #76; LateralMovement/C2 never emitted |
| MitreTactic (E-27) | mitre.rs | 16 variants (14 Enterprise + 2 ICS) | YES | non_exhaustive since initial impl |
| FlowState (E-10) | flow.rs | New, SynSent, Established, Closing, Closed | No | Monotonic toward Closed |
| CloseReason (E-15) | handler.rs | Fin, Rst, Timeout, MemoryPressure | No | Ignored by analyzers |
| Direction (E-14) | handler.rs | ClientToServer, ServerToClient | No | Binary; no Unknown |
| InsertResult (E-13) | segment.rs | 9 variants | No | All 9 matched in engine |
| DispatchTarget (E-22) | dispatcher.rs | Http, Tls, None | No | Module-private; None: not cached until attempt cap, then cached permanently (two-phase, LESSON-P2.11) |
| OutputFormat (E-3) | cli.rs | Json, Csv | No | Both variants now wired |
| Commands (E-2) | cli.rs | Analyze{..}, Summary{..} | No | All fields wired as of P1.04 |
| SniValue (E-35) | tls.rs | Ascii, AsciiWithControl, NonAsciiUtf8, NonUtf8 | No | Module-private |


## 12 Value Objects (VO-1..VO-12)

### VO-1: FlowKey canonical ordering

**Invariant:** `FlowKey::new(ip_a, port_a, ip_b, port_b)` stores `(lower, upper)` where
`(lower_ip, lower_port) <= (upper_ip, upper_port)` using tuple comparison. BOTH fields of
the pair participate in the comparison together; independent per-field sorting would break
the invariant.

**Enforcement:** `src/reassembly/flow.rs:48` (`if (ip_a, port_a) <= (ip_b, port_b)`).
**Tests:** `test_flow_key_canonicalization` and `test_flow_key_same_ip_different_ports` in
`tests/reassembly_flow_tests.rs:7,23`.

### VO-2: Verdict is a non-exhaustive 3-arm enum

`#[non_exhaustive]` (P2.10 / #76). Downstream match consumers must include a `_` arm.
This allows new verdicts to be added in future without breaking dependent crates.

### VO-3: Confidence is a non-exhaustive 3-arm enum

`#[non_exhaustive]` (P2.10 / #76). Same contract as VO-2.

### VO-4: ThreatCategory is a non-exhaustive 8-arm enum

`#[non_exhaustive]` (P2.10 / #76). Now consistent with `MitreTactic` (VO-5). The prior
inconsistency (LESSON-P2.10) is closed.

### VO-5: MitreTactic is non-exhaustive

`#[non_exhaustive]` attribute at `src/mitre.rs:46`. Downstream matches must include a `_` arm.

### VO-6: MITRE technique ID format

IDs follow `TXXXX` (parent) or `TXXXX.NNN` (sub-technique) pattern. Unknown IDs passed to
`technique_info()` return `None`. Terminal renderer shows `<id> (unknown)`. JSON output
passes the raw string through without validation.

### VO-7: JA3 fingerprint hash

MD5 hex of `version,ciphers,extensions,curves,point_formats` with GREASE values filtered
(`val & 0x0F0F == 0x0A0A`). Enforcement: `src/analyzer/tls.rs:95-151` (`compute_ja3`). If GREASE is not
filtered, the same client produces different hashes per-connection (RFC 8701).

### VO-8: JA3S fingerprint hash

MD5 hex of `version,cipher,extensions` server-side. Enforcement: `src/analyzer/tls.rs:156-173` (`compute_ja3s`).

### VO-9: Finding.summary/evidence carries raw bytes

No escape applied at construction. ADR 0003. Enforcement: module header doc-comment at
`src/findings.rs:10-14` and `Finding::Display` doc at `src/findings.rs:148-156`.
Not mechanically enforced; convention violation would not be caught by the compiler.

### VO-10: Direction is a closed binary enum

`ClientToServer` / `ServerToClient` only. Ambiguous flows default to `ServerToClient` when
`initiator` is None (`flow.rs:214-220`, `direction()` method).

### VO-11: FlowState transitions monotonic toward Closed

`New -> SynSent -> Established -> Closing -> Closed`. RST direct-jumps to Closed. The only
non-monotonic path is `on_data_without_syn` (jumps to Established from New).

### VO-12: InsertResult::IsnMissing is a programming-error sentinel

Should never occur in production. If reached, segment is silently dropped. One-shot `eprintln!`
via `ISN_MISSING_WARNED: AtomicBool` at `src/reassembly/segment.rs:16`; the `eprintln!` is at
`src/reassembly/segment.rs:54-56`. No finding emitted, no stats counter incremented.
