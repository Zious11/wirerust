---
artifact: L2-cap-07
traces_to: ../domain-spec.md
cap_id: CAP-07
title: TLS Traffic Analysis
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-07: TLS Traffic Analysis

## What the system does today

`TlsAnalyzer` (E-33, C-13) implements `StreamHandler + StreamAnalyzer`. It buffers
reassembled TCP data per flow direction, parses TLS records using `tls-parser`, and emits
`Finding` objects for SNI anomalies, weak cipher suites, deprecated protocol versions, and
related issues.

**Sources:** C-13 analyzer/tls.rs (module-decomposition.md L3 Domain Layer). BC-TLS-001..037.

## Per-flow state: TlsFlowState (E-34)

```
TlsFlowState {
    client_buf:          Vec<u8>  (max MAX_BUF = 65,536 bytes)
    server_buf:          Vec<u8>  (max MAX_BUF = 65,536 bytes)
    client_hello_seen:   bool
    server_hello_seen:   bool
}
```

Once both `client_hello_seen` and `server_hello_seen` are true, `TlsFlowState::done()`
returns true and subsequent `on_data` calls early-exit without processing. `done()` is
defined at tls.rs:291-293; the early-exit guard in `on_data` is at tls.rs:721-724.
The state record persists in the HashMap until `on_flow_close` fires.

## Limit constants

| Constant | Value | Purpose |
|---|---|---|
| MAX_BUF | 65,536 bytes | Per-direction TLS record buffer |
| MAX_MAP_ENTRIES | 50,000 | Per-map cap for sni_counts/ja3_counts/etc. |
| MAX_RECORD_PAYLOAD | 18,432 bytes | Max TLS record payload accepted for parsing |

## SNI classification (4-way, INV-5)

`extract_sni` classifies an SNI byte sequence using an ordered match on `SniValue` (E-35):

```
1. from_utf8 OK AND s.is_ascii() AND !contains_c0_or_del(s) -> SniValue::Ascii      (silent)
2. from_utf8 OK AND s.is_ascii() AND contains_c0_or_del(s)  -> SniValue::AsciiWithControl  (T1027)
3. from_utf8 OK AND !s.is_ascii()                           -> SniValue::NonAsciiUtf8       (T1027)
4. from_utf8 Err                                            -> SniValue::NonUtf8             (T1027)
```

**Critical precedence rule (BC-TLS-037; pass-2 R3 Target 2):** The `is_ascii()` predicate is
the controlling gate. For SNI bytes that are valid UTF-8 but contain BOTH non-ASCII chars AND
C0/DEL control bytes (e.g., `caf\x01\xe9`), arm 3 fires (NonAsciiUtf8), NOT arm 2
(AsciiWithControl). The summary text says "non-ASCII characters"; the control-byte signal is
recoverable only from the hex evidence field. This is not a bug but an observable behavior
that SOC operators using summary-text search must know.

## JA3 / JA3S fingerprinting

- `compute_ja3`: MD5 of `version,cipher_suites,extensions,elliptic_curves,point_formats`
  with GREASE values (`val & 0x0F0F == 0x0A0A`) filtered per RFC 8701 (VO-7).
- `compute_ja3s`: MD5 of `version,cipher,extensions` server-side (VO-8).
- Both stored in `ja3_counts` / `ja3s_counts` HashMaps; surfaced in AnalysisSummary detail.

## Anomaly detections (7 TLS findings)

All TLS findings carry a `direction` tag (P2.08 / #77):

| Detection | Trigger | Finding | MITRE | Direction tag | Source lines |
|---|---|---|---|---|---|
| SNI AsciiWithControl | Arm 2 of extract_sni | Anomaly/Inconclusive/Low | T1027 | ClientToServer | tls.rs:426-448 |
| SNI NonAsciiUtf8 | Arm 3 of extract_sni | Anomaly/Inconclusive/Low | T1027 | ClientToServer | tls.rs:449-468 |
| SNI NonUtf8 | Arm 4 of extract_sni | Anomaly/Inconclusive/Low | T1027 | ClientToServer | tls.rs:469-489 |
| Weak ClientHello ciphers | ClientHello contains NULL/anon/export ciphers | Anomaly/Likely/High | none | ClientToServer | tls.rs:504-517 |
| Deprecated ClientHello version | SSLv2 or SSLv3 in ClientHello | Anomaly/Likely/High | none | ClientToServer | tls.rs:526-539 |
| Weak ServerHello cipher selected | ServerHello cipher is weak (NULL/anon/export/RC4) | Anomaly/Likely/Medium | none | ServerToClient | tls.rs:571-582 |
| Deprecated ServerHello version | SSLv2 or SSLv3 negotiated | Anomaly/Likely/High | none | ServerToClient | tls.rs:591-604 |

## Truncation instrumentation (CNV-PAT-002 conformance)

`TlsAnalyzer` carries `truncated_records: u64` (tls.rs:312), incremented at tls.rs:645 each
time a TLS record is discarded due to exceeding `MAX_RECORD_PAYLOAD`. The counter is surfaced
in `summarize()` at tls.rs:798-801 via `detail["truncated_records"]`. This was added by
P1.05 (#73). TlsAnalyzer now fully conforms to CNV-PAT-002 (silent-drop instrumentation
convention).

## Weak-cipher evidence cardinality (O-06)

The weak-cipher ClientHello finding at tls.rs:504-517 uses `evidence: weak` where `weak` is
a filtered `Vec<String>` of cipher names. This is the ONLY evidence vec in the codebase with
data-dependent cardinality. Upper bound: ~9,216 cipher names (MAX_RECORD_PAYLOAD / 2 bytes).
Worst-case Finding heap: ~270-500 KB. No per-cipher cap exists (domain-debt O-06).

## Statistics tracked

`sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts` (all bounded
by `MAX_MAP_ENTRIES`). `handshakes_seen: u64`. `parse_errors: u64`. `truncated_records: u64`.
`all_findings: Vec<Finding>` (unbounded -- no MAX_FINDINGS applied).

## BC references

BC-2.07.001..037 (37 contracts). Key: BC-2.07.014..020 (SNI 4-way classification),
BC-2.07.021..030 (JA3/JA3S), BC-2.07.031..036 (weak cipher / deprecated version),
BC-2.07.037 (SNI disambiguation for mixed control+non-ASCII).
Component: C-13 (src/analyzer/tls.rs) per module-decomposition.md.
