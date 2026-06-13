# Holdout Evaluation — Chunk 1 (HS-001..HS-024 subset, 20 scenarios)

- Evaluator: black-box holdout (information asymmetry enforced; no src/, tests/*.rs, specs, or BC files read)
- Binary: `target/debug/wirerust` built at develop (`cargo build` clean)
- Method: real binary run against crafted/fixture pcaps; observed stdout/stderr/exit codes/JSON only.
- Date: 2026-06-01
- erratum: 2026-06-13 (Pass-15 H3) — This file is a FROZEN HISTORICAL RUN-RECORD of the evaluation conducted on 2026-06-01. References to the scalar `mitre_technique` field in the per-scenario table (HS-007, HS-016, HS-017 rows) reflect the pre-v0.3.0 Finding schema as evaluated at that time. The current Finding schema uses `mitre_techniques: Vec<String>` per ADR-006/STORY-100 (v0.3.0). Historical content is preserved verbatim; this note provides schema context only.

## Per-Scenario Results

| HS-id | must_pass | satisfaction | PASS/FAIL | observed vs expected |
|-------|-----------|--------------|-----------|----------------------|
| HS-001 | true | 1.00 | PASS | Ethernet(1) accepted exit0; 802.11(105) rejected exit1 "Unsupported pcap link type: IEEE802_11"; pcapng rejected exit1 at header "wrong magic number" before packet loop; RAW(101)/IPV4(228) accepted. No stdout before rejection. |
| HS-002 | true | 1.00 | PASS | Header-only pcap exit0 zero counts; corrupt magic exit1 with context; 0-byte exit1 "unexpected end of file"; truncated mid-packet exit1 "Failed to read packet". No panic. |
| HS-003 | true | 1.00 | PASS | Eth IPv4 + RAW IPv4 show correct dotted-decimal src/dst; RAW IPv6(229) shows colon notation; <14-byte Eth frame -> skipped_packets=1, exit0, descriptive warning, no panic. |
| HS-004 | true | 1.00 | PASS | Linux SLL(113, 16-byte header) TCP decoded with correct IPs/protocol; ICMP->Icmp, OSPF->Other(89), UDP->Udp; packets_skipped_non_tcp=6 exact (3 UDP+2 ICMP+1 OSPF); no ICMP findings; no crash. |
| HS-006 | true | 0.70 | PASS | Format `[Category] VERDICT (CONFIDENCE) <sep> summary`: bracket category, uppercase LIKELY/INCONCLUSIVE, uppercase HIGH/MEDIUM/LOW in parens all correct. DEVIATION: verdict separator is ASCII hyphen `-`, NOT em-dash U+2014. BC-2.09.002 + rubric Data-integrity(0.3) require em-dash. (Note: em-dash IS used on the `MITRE: Tn — name` line and arrow → is used in flow strings, so it is a deliberate token choice for the one-liner.) |
| HS-007 | true | 1.00 | PASS | Required fields (category/verdict/confidence/summary) always present; timestamp NEVER present (absent not null); source_ip present only for reassembly findings, absent for HTTP/TLS; direction/mitre_technique present only when Some. Raw bytes in summary preserved (from_utf8_lossy), RFC8259 escaping. |
| HS-008 | true | 0.95 | PASS | `--mitre` groups under canonical headers `## Defense Evasion`, `## Discovery`, `## Uncategorized`; kill-chain order (Defense Evasion before Discovery) correct; T1036->Masquerading, T1027->Obfuscated Files or Information both under Defense Evasion; each tactic once; None-technique finding under Uncategorized. Could not directly emit a "Command and Control" finding to byte-verify that exact label, but all observed labels use canonical ATT&CK spacing. |
| HS-009 | true | 1.00 | PASS | All 5 emitted IDs resolve: T1036->Masquerading(Defense Evasion), T1083->File and Directory Discovery(Discovery), T1027->Obfuscated Files or Information(Defense Evasion), T1505.003->Web Shell(Persistence), T1046 not triggered but lookup pattern consistent. Sub-technique period handled (T1505.003 != T1505). No crash. NOTE: scenario text claims T1083->Reconnaissance; impl maps T1083->Discovery which is ATT&CK-v14-correct (scenario frontmatter error, not impl defect). |
| HS-011 | true | 0.90 | PASS | UDP QR-bit counting exact (verified 7q/4r and 30q/0r flood); findings ALWAYS 0 incl. flood; DNS over TCP port-53 IS counted (non-zero). Minor: TCP-DNS count slightly off (sent 1q/1r got 2q/3r) — likely 2-byte TCP length-prefix offset in QR parsing; does not affect the zero-findings guarantee or UDP accuracy. |
| HS-012 | true | 1.00 | PASS | packets_skipped_non_tcp=35 exact (30 UDP+5 ICMP); bytes_reassembled=160 exact (100 client+60 server payload, pure ACK contributed 0, NOT frame lengths); all-non-TCP -> skipped=10 bytes=0. Stats map has all expected keys. |
| HS-013 | true | 0.95 | PASS | Clean SYN/SYNACK/ACK/data/FIN session -> 0 findings, flows_fin=1; client RST -> flows_rst=1, 0 findings; server RST -> flows_rst=1, 0 findings (closes regardless of side). Direction tagging ClientToServer observed correct on findings. No spurious teardown warnings. |
| HS-014 | true | 1.00 | PASS | Mid-stream (no SYN) -> exit0, flows_partial=1, HTTP path-traversal finding still emitted (data reaches analyzers), ISN inferred, NO per-packet IsnMissing warnings (stderr clean); truncated session -> exit0, bytes_reassembled>0, finalize flushes, no hang. |
| HS-016 | true | 0.70 | PASS | Conflicting TCP overlap emits Anomaly/Likely/High + mitre_technique=T1036 (verified); exact-duplicate retransmit emits NO conflict (correct dedup/first-wins); no panic. DEVIATION: evidence field is fixed description "Retransmitted segment contains different data", NOT the raw conflicting byte sequences — rubric Data-integrity(0.3) and BC-2.04.037 expect original bytes preserved in evidence. (Other finding types e.g. SNI DO carry raw hex evidence.) |
| HS-017 | true | 0.95 | PASS | End-to-end pipeline produces well-formed findings: required fields present, timestamp absent on ALL findings, source_ip present for reassembly (e.g. 10.0.0.1 / IPv6) absent for HTTP/TLS; all mitre_technique IDs resolve to catalog names in --mitre; 0-finding captures still emit valid empty findings array. (Same array-vs-object analyzers-schema note as HS-023.) |
| HS-018 | true | 1.00 | PASS | TLS SNI w/ bytes 0xFF/0x07/0x09: JSON summary = from_utf8_lossy raw bytes (U+FFFD as raw \xef\xbf\xbd, 0x07 as , tab as \t = RFC8259 std, NOT terminal backslash-hex); JSON valid; terminal output of SAME finding uses terminal escaping (\u{7}, \t). Forensic-fidelity divergence between JSON and terminal exactly as specified. |
| HS-019 | true | 1.00 | PASS | ISN 0xFFFFFF00, two 128-byte segments straddling the 32-bit wrap: bytes_reassembled=256, 0 findings, 0 spurious overlaps/out-of-window; out-of-order-across-wrap delivery also 256 bytes 0 findings. Modular arithmetic correct. |
| HS-021 | true | 0.90 | PASS | Three-close pcap: FIN flow -> flows_fin=1; RST flow -> flows_rst=1; open-no-close flow flushed at finalize (counted in flows_completed); bytes_reassembled=12 (all 3 flows). 0 anomaly findings from clean closes. Running twice -> byte-identical JSON (finalize idempotent). Note: flows_expired=0 (finalize flush distinct from mid-capture expiry counter) — observable cleanup behavior is correct. |
| HS-022 | true | 1.00 | PASS | Malformed mix (truncated Eth, bad-IHL IPv4, non-IP ethertype, garbage) interleaved with 2 valid TCP: exit0, no panic, total_packets=2, skipped_packets=4. All-malformed file: exit0, total=0, skipped=3 (skipped==total allowance). skipped counter accurate. |
| HS-023 | true | 0.90 | PASS | Top-level summary/findings/analyzers all present; analyzers contains TCP Reassembly + DNS when --dns, only Reassembly without --dns, empty [] when none; summary has total_packets/total_bytes/skipped_packets; bytes_reassembled<=total_bytes; DNS-with-no-DNS shows {0,0}; 600KB pcap in 0.04s. DEVIATION: analyzers is a JSON ARRAY of {analyzer_name,detail} not a keyed object — scenario's literal jq path `.analyzers.dns.dns_queries` returns null; all data present under different shape. |
| HS-024 | true | 1.00 | PASS | Reassembly conflict finding includes source_ip = actual packet src (IPv4 10.0.0.1 AND IPv6 2001:db8::aaaa verified); HTTP & TLS findings have NO source_ip key (absent, not null). source_ip values are valid IP strings. |

## Chunk Summary

- Scenarios evaluated: 20 / 20 (all fully executed against the real binary; none unevaluable)
- must_pass scenarios: 20 (all 20 are must_pass)
- Chunk mean satisfaction: (1.00+1.00+1.00+1.00+0.70+1.00+0.95+1.00+0.90+1.00+0.95+1.00+0.70+0.95+1.00+1.00+0.90+1.00+0.90+1.00) / 20 = 18.95 / 20 = **0.9475**
- Min must-pass score: **0.70** — tied at HS-006 (em-dash separator) and HS-016 (evidence raw bytes)
- Count of must-pass scenarios below 0.6: **0**
- Gate (mean >= 0.85 AND every must-pass >= 0.60): **PASS**

## Cross-cutting Findings

1. **F-CHUNK1-A (HS-006, low severity, real):** Finding one-liner verdict separator is ASCII hyphen `-`, not em-dash U+2014 required by BC-2.09.002 and the rubric. Em-dash IS correctly used on the `MITRE: Tn — name` line, and the `→` flow arrow is correct, so this is an inconsistent/incorrect token choice in the one-liner formatter rather than a Unicode-output limitation. Drops HS-006 by the 0.3 data-integrity weight.

2. **F-CHUNK1-B (HS-016, low/medium severity, real):** Conflicting-overlap (T1036) finding evidence is a fixed description string ("Retransmitted segment contains different data") rather than the raw conflicting byte sequences. BC-2.04.037 and the HS-016 rubric (0.3 weight) expect the original/conflicting bytes preserved verbatim for forensic use. Note other detectors (SNI non-UTF-8) DO emit raw hex evidence, so the raw-byte contract is honored unevenly across finding types.

3. **F-CHUNK1-C (HS-011, very low severity, real):** DNS-over-TCP query/response counts are slightly inflated vs the messages sent (likely the 2-byte TCP DNS length prefix is not accounted for when locating the QR bit). UDP counting is exact. Does not violate the zero-findings guarantee.

4. **F-CHUNK1-D (schema, informational, affects HS-011/012/017/023 jq-paths only):** The JSON `analyzers` value is an ARRAY of `{analyzer_name, detail}` objects, whereas several scenarios' verification snippets assume a keyed object (`.analyzers.dns`, `.analyzers.reassembly`). All required data is present and correct; only the literal jq access path differs. Scored as a minor deviation, not a behavioral failure, because the scenarios describe data presence/values, which are all satisfied.

5. **Scenario-text vs ATT&CK (HS-009, not an impl defect):** HS-009 step 3 asserts T1083->Reconnaissance; the implementation maps T1083->Discovery, which matches MITRE ATT&CK Enterprise v14. The data-integrity rubric explicitly references ATT&CK v14, so the implementation is judged correct.

## Notes on Coverage / Reproduction
- All conflicting-overlap scenarios (HS-006/016/019/024) require segments to remain BUFFERED to exercise overlap logic; achieved by injecting a forward-gap segment so earlier segments are not flushed in-order, plus `--overlap-threshold 0` to surface the count-based companion finding. The conflicting-content T1036 finding fires independent of the count threshold.
- No panics ("thread 'main' panicked") were observed in any run across all 20 scenarios.
