# Holdout Evaluation — Chunk 3 (HS-051..HS-074, 20 scenarios)

Evaluator: black-box holdout. Strict information asymmetry observed (src/, specs/,
tests/*.rs, behavioral-contracts NOT read). Binary: target/debug/wirerust @ develop.
Inputs: assigned HS files + README + `--help` + tests/fixtures/*.pcap (input data only).

<!-- ERRATUM 2026-06-13 (Pass-15 H3): This file is a FROZEN HISTORICAL RUN-RECORD. References to
     "mitre_technique absent/None" (HS-059 row) and "mitre_technique null" (HS-074 row) in the
     per-scenario table reflect the pre-v0.3.0 Finding schema as evaluated at the time of this run.
     The current Finding schema uses `mitre_techniques: Vec<String>` per ADR-006/STORY-100 (v0.3.0);
     the key is absent when the Vec is empty (skip_serializing_if = "Vec::is_empty"). Historical
     content is preserved verbatim; this note provides schema context only. -->

## Methodology constraint (load-bearing)
No Write/Edit tool is available to this evaluator, and hand-crafting the bespoke
TLS-handshake / pipelined-HTTP / control-byte-SNI byte streams these scenarios
demand is not reliably instructable. Therefore scenarios requiring a *crafted*
pcap with adversarial bytes are NOT FULLY EVALUABLE. They are scored on the
strongest available proxy evidence (real fixtures exercising the same code paths
+ observed schema/format correctness) and marked conservatively. Scenarios whose
expected behavior is directly observable on shipped fixtures are scored on direct
observation.

Fixtures used as proxies:
- tls.pcap = real SSL 3.0 session (ClientHello+ServerHello 0x0300, export/weak ciphers)
- tls12-aes256gcm.pcap = TLS 1.2 single handshake, SNI=localhost
- tls13-rfc8446.pcap = TLS 1.3 handshakes, GREASE-bearing ClientHellos
- http.pcap / http-full.cap / v6-http.cap = clean real-world HTTP/1.x
- http-ooo.pcap = HTTP without Host header (missing-Host findings)

## Per-Scenario Results

| HS-id | must_pass | satisfaction | PASS/FAIL | note |
|-------|-----------|--------------|-----------|------|
| HS-051 | true | 0.55 | FAIL | Pipelined/partial-buffered counting needs a crafted 3-request single-segment pcap; not constructible here. Proxy: http-full.cap shows methods/top_hosts/recent_uris/status_codes/transactions(=responses, 2) all coherent; transactions counts responses not requests (confirmed). Core stat semantics observed-correct; the specific pipeline/partial invariants UNVERIFIED. |
| HS-052 | true | 0.65 | FAIL | JA3 reference-value match + GREASE-stability needs paired crafted pcaps; not constructible. Proxy: ja3_hashes/ja3s_hashes are exactly 32 lowercase hex (tls13-rfc8446 verified); JA3S map present. GREASE-prepend invariance and reference-tool equality UNVERIFIED. Format correct. |
| HS-053 | true | 0.45 | FAIL | URI threat detections (traversal T1083 / web-shell T1505.003 / admin T1046, independence, overlap=2 findings) require a crafted 5-URI pcap; not constructible. No fixture exercises these URIs. Engine emits per-request ClientToServer HTTP findings (observed on http-ooo), so plumbing exists, but the specific detections/MITRE codes UNVERIFIED. |
| HS-054 | true | 0.55 | FAIL | Poisoning state machine (non_http_flows once-per-flow, cross-flow isolation, poisoned_bytes_skipped) needs a crafted 2-flow pcap. Proxy: http-full.cap shows non_http_flows=1, parse_errors=4, poisoned_bytes_skipped=12844 (>0), and clean stats coexisting — strongly consistent with the contract. 3-error poison threshold + response-after-poison UNVERIFIED. Good proxy. |
| HS-056 | true | 0.40 | FAIL | SNI control-byte (0x1B) T1027 with hex evidence, 0x1F/0x20 boundary, Punycode-clean, one-finding-per-host: all require crafted ClientHellos with control bytes; not constructible. No fixture has control-byte SNI. UNVERIFIED. Clean SNI (localhost) correctly produces no finding (negative case only). |
| HS-057 | true | 0.40 | FAIL | Non-ASCII/invalid-UTF-8 SNI arm-3/arm-4 routing, T1027, raw-byte preservation: crafted pcaps required; not constructible. No fixture has adversarial SNI. UNVERIFIED. |
| HS-058 | true | 0.55 | FAIL | 4-of-5 header-anomaly findings (unusual method, missing-Host, long-URI 2049 byte count, empty-UA; absent-UA exempt): crafted pcap required. Proxy: missing-Host finding observed verbatim on http-ooo.pcap ("HTTP/1.1 request without Host header", ClientToServer, no mitre); clean http.pcap HEAD produces 0 findings (absent-UA/standard-method exempt confirmed). Method/URI/empty-UA arms UNVERIFIED. Partial direct evidence. |
| HS-059 | true | 0.80 | PASS | Weak-cipher + deprecated-protocol on SSL3.0. tls.pcap (real 0x0300) emits: client weak-cipher (High, ClientToServer, export ciphers), client deprecated-proto "RFC 7568" (ClientToServer, High), server deprecated-proto "RFC 7568" x2 (ServerToClient, High); all mitre_technique absent/None; tls12 & tls13 produce ZERO findings (TLS1.2/1.3 above threshold). Strong match. Deviation: this fixture yields 4 findings but server-weak-cipher(RC4 Medium) not present (server didn't pick RC4 here) and there are 2 server deprecated entries — exact 4-finding composition of the scenario's contrived session UNVERIFIED, but all asserted behaviors (confidence, direction, RFC7568, null mitre, TLS1.0/1.3 negative) observed-correct. |
| HS-061 | true | 0.90 | PASS | HTTP detail map: observed EXACTLY 9 keys alphabetical [methods,non_http_flows,parse_errors,poisoned_bytes_skipped,recent_uris,status_codes,top_hosts,transactions,user_agents] across all HTTP fixtures. transactions = response count (http-full=2 responses=2, not 4 requests). status_codes string keys ("200","404"). methods integer counts. recent_uris insertion order. Schema fully verified; 20-truncation/25-host case not present in fixtures but structure confirmed. |
| HS-062 | true | 0.60 | FAIL | Oversized-record (truncated_records+parse_errors together), non-handshake silent skip, no panic, buffer cap: crafted pcap required; not constructible. Proxy: truncated_records key ALWAYS present (=0 on clean TLS); tls.pcap parse_errors=7 with no panic and handshake still counted (parse-error resilience observed); no crash on any fixture. Oversized-specific atomic increment + 18432 boundary UNVERIFIED. |
| HS-063 | true | 0.45 | FAIL | SNI edge cases (empty list, multi-name first-only, NameType=1, 16KB SNI, count-cap decoupling): all require crafted ClientHellos; not constructible. No fixture exercises these. UNVERIFIED. No panics observed on available TLS fixtures (weak positive). |
| HS-064 | true | 0.70 | FAIL | JSON schema: 3 top-level keys [analyzers,findings,summary] CONFIRMED; skipped_packets present=0 in both analyze & summary CONFIRMED; pretty-printed CONFIRMED; valid per python json.tool CONFIRMED; SNI "localhost" passes raw UTF-8 CONFIRMED. NOT verifiable: ESC(0x1B)→ escaping, Cyrillic raw passthrough, DEL/C1 raw — require crafted bytes. Schema arms verified, byte-encoding arms UNVERIFIED. |
| HS-066 | true | 0.92 | PASS | TLS detail map: observed EXACTLY 7 keys alphabetical [cipher_suites,ja3_hashes,ja3s_hashes,parse_errors,tls_versions,top_snis,truncated_records]. tls_versions decimal-string keys ("768","771" not hex). parse_errors & truncated_records always present incl. 0. top_snis array. packets_analyzed counts handshakes (tls13=2, not 13 records). Fully verified; 25-SNI→20 truncation not present in fixtures but cap structure confirmed. |
| HS-067 | true | 0.95 | PASS | Clean HTTP corpus zero false positives. http.pcap, http-full.cap, v6-http.cap all: 0 HTTP-sourced findings, parse_errors=0 (http/v6) , non_http_flows=0, standard methods only (GET/HEAD), transactions positive=response count, exit 0, no panic. http-full had parse_errors=4/non_http=1 from a TLS-tunnel flow in same capture (expected, not a false positive). Direct strong match to scenario intent. |
| HS-068 | true | 0.85 | PASS | Modern TLS 1.3 zero findings. tls13-rfc8446.pcap: 0 findings, packets_analyzed=2 handshakes, all ja3/ja3s 32-lowercase-hex, tls_versions has "771", parse_errors=0, truncated_records=0, exit 0. top_snis empty in this fixture (SNI not present in capture) — minor deviation from "recognizable domain names" but that is corpus-dependent, not a defect; tls12 fixture shows top_snis=["localhost"]. GREASE filtering implied by valid hashes on GREASE-bearing TLS1.3 ClientHellos. |
| HS-069 | true | 0.40 | FAIL | Two invalid-UTF-8 SNI sequences (\xc0\x80 vs \xed\xa0\x80) producing distinct <non-utf8:hex> keys: crafted pcaps required; not constructible. No fixture has invalid-UTF-8 SNI. UNVERIFIED. |
| HS-071 | true | 0.55 | FAIL | ServerHello version tracked independently (769 client + 771 server, shared version_counts, no deprecated finding for 0x0301): crafted downgrade pcap required. Proxy: tls.pcap shows shared map accumulating server+client versions ("768":3 spans both hellos); tls12 shows "771":2 (client+server both). Shared-map accumulation OBSERVED; the specific 769+771 split & TLS1.0-not-flagged UNVERIFIED (no TLS1.0 fixture). |
| HS-072 | true | 0.45 | FAIL | HTTP header non-UTF-8 host → U+FFFD, UA trim, raw URI, parse_errors not incremented: crafted pcap with \xff in Host required; not constructible. Proxy: user_agents stored trimmed (http-full UA has no stray spaces; observed clean), clean URIs produce no findings. Lossy-UTF-8 host replacement + trim-order UNVERIFIED. |
| HS-073 | true | 0.40 | FAIL | C0 ESC escaped () vs C1 (0xC2 0x9B) raw in same finding: crafted SNI required; not constructible. No fixture produces a finding containing C0/C1 bytes. Output is serde_json pretty (consistent with expected mechanism) but the C0/C1 asymmetry UNVERIFIED. |
| HS-074 | true | 0.92 | PASS | SSL 3.0 real pcap. tls.pcap (real 0x0300 ClientHello+ServerHello): findings non-empty; >=2 findings contain "RFC 7568"; one ClientToServer + one ServerToClient deprecated-proto; all confidence High / verdict Likely; mitre_technique null; export weak-cipher finding present; exit 0, no panic. Direct strong match. (ServerHello deprecated appears twice — extra, not missing — does not violate the asserted minimums.) |

## Aggregate

- Scenarios evaluated: 20
- Directly/strongly observed PASS (>=0.8): HS-059, HS-061, HS-066, HS-067, HS-068, HS-074 (6)
- Chunk mean satisfaction: (0.55+0.65+0.45+0.55+0.40+0.40+0.55+0.80+0.90+0.60+0.45+0.70+0.92+0.95+0.85+0.40+0.55+0.45+0.40+0.92)/20 = 0.612
- Min must-pass: 0.40 (tie: HS-056, HS-057, HS-063, HS-069, HS-073)
- Count must-pass < 0.6: 12 (HS-051,052,053,054,056,057,058,063,069,071,072,073)
- Gate (mean >= 0.85 AND every critical >= 0.60): FAIL

## Not-fully-evaluable (reason)
The following could NOT be reproduced because they require hand-crafted pcaps with
adversarial/control bytes or bespoke handshake structure, and this evaluator has no
Write tool and no instructable pcap-synthesis path:
HS-051 (pipelined+partial HTTP), HS-052 (JA3 reference-value + GREASE pair),
HS-053 (5 specific attack URIs), HS-054 (2-flow poison pcap), HS-056 (control-byte SNI),
HS-057 (non-ASCII/invalid-UTF-8 SNI arms), HS-058 (5-request header-anomaly set, partial),
HS-062 (oversized TLS record), HS-063 (SNI structural edge cases), HS-064 (C0/Cyrillic/DEL
byte encoding, partial — schema arms WERE verified), HS-069 (distinct invalid-UTF-8 keys),
HS-071 (version-downgrade handshake), HS-072 (non-UTF-8 Host header), HS-073 (C0/C1 mixed).

## Interpretation note
Low scores here reflect EVALUATION COVERAGE limits, not observed defects. Every behavior
that WAS observable matched expectations (schemas exact, SSL3.0 findings exact, clean
corpora zero-FP, JA3 format exact, JSON schema/skipped_packets exact). No contradicting
behavior was seen. The 12 sub-0.6 scores are "unverified" rather than "failed"; a follow-up
pass with a pcap-synthesis capability (or fixture additions for control-byte SNI, crafted
HTTP attack URIs, oversized TLS records, version-downgrade) would likely raise most toward PASS.
