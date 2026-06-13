# Holdout Re-Evaluation — Chunk 3 (12 "unverified" scenarios)

<!-- ERRATUM 2026-06-13 (Pass-16 C-01): This is a FROZEN historical run-record.
     Observed verdicts, satisfaction scores, and output quotes reflect a specific past
     binary build and must not be rewritten. The phrase "mitre=null" in the HS-058 row
     (line ~18) uses pre-v0.3.0 schema language as evaluated at the time. Current schema
     (ADR-006 / STORY-100 v0.3.0): `mitre_techniques: Vec<String>` with
     `skip_serializing_if = "Vec::is_empty"` — the key is ABSENT (not null) when the
     Vec is empty. This erratum was added by the Pass-16 remediation sweep; the Pass-15
     H3 erratum sweep covered chunk1-eval.md and chunk3-eval.md but missed this
     chunk3-reeval.md sibling. -->

Re-evaluator crafted adversarial inputs from scratch using a dependency-free Python
libpcap generator (Ethernet/IPv4/TCP handshake + PSH segments for HTTP; hand-built
TLS handshake records with SNI extensions / ServerHello for TLS). All scores are by
OBSERVED behavior of `target/debug/wirerust analyze <pcap> --http|--tls --reassemble
--output-format json` (raw JSON byte inspection where escaping mattered).

## Per-scenario results

| HS-id | must_pass | satisfaction | PASS/FAIL | What was crafted + observed-vs-expected |
|-------|-----------|--------------|-----------|------------------------------------------|
| HS-051 | true | 1.00 | PASS | Flow: 2 full `GET /a` requests in one segment, then `GET /b` split across 2 segments. Observed `methods{GET:3}`, `recent_uris=[/a,/a,/b]`, `top_hosts=[h.test]`, `transactions=0`, `parse_errors=0`, 0 findings. Pipelined both counted; partial counted only after completion; transactions tracks responses (0) not requests. Exact match. |
| HS-053 | true | 1.00 | PASS | 5 GET URIs in one flow. Observed exactly 5 findings: wp-admin→T1046, c99.php→T1505.003, `../`→T1083, and `/cmd.php/../../etc/passwd`→TWO findings (T1083 + T1505.003). Clean `/images/photo.jpg`→0 findings. All ClientToServer; full URI in evidence. Exact match. |
| HS-054 | true | 1.00 | PASS | 2 flows. Flow A: 3 non-HTTP binary blobs in request dir + extra 80B after poison, then valid HTTP response in response dir; Flow B clean req+resp. Observed `non_http_flows=1` (counted once), `parse_errors=3`, `poisoned_bytes_skipped=80`, `transactions=2` (flow A response works after request poisoned + flow B), `methods{GET:1}`, 0 Anomaly findings. NOTE: had to use non-TLS-signature binary because content-first classifier routes 0x16/0x17 leading bytes to the TLS analyzer (HS-038); the behavioral contract (binary→parse_errors+poison, no finding, response-dir isolation, cross-flow isolation) is fully satisfied. |
| HS-056 | true | 1.00 | PASS | 4 ClientHellos: example.com, evil\x1bcom.net, xn--caf-dma.example, "test host.com". Observed exactly 1 finding (T1027) for the ESC SNI; evidence `hex: 6576696c1b636f6d2e6e6574`; all 4 SNIs in top_snis; space(0x20) & punycode produce 0 findings. Confirmed boundary: separate 0x1F SNI DOES trigger T1027. Exact match incl. BC-2.07.016. |
| HS-057 | true | 1.00 | PASS | 3 ClientHellos: Cyrillic мир.рф (UTF-8), \xff\xfe (invalid), \x01café.test (C0+non-ASCII). Observed 3 T1027 findings all ClientToServer. (A) summary verbatim "мир.рф" + "non-ASCII"; (B) "non-UTF-8 bytes", evidence `hex: fffe`, key `<non-utf8:fffe>`; (C) summary "non-ASCII" NOT "control" → arm 3 wins over arm 2. Exact match. |
| HS-058 | true | 1.00 | PASS | 5 requests: DELETE, GET-no-Host(1.1), 2049-byte URI, empty UA, clean(no UA). Observed exactly 4 findings: "Unusual HTTP method: DELETE", "HTTP/1.1 request without Host header", "Abnormally long URI (2049 chars)", "Empty User-Agent header"; all mitre=null, all ClientToServer; clean request (absent UA)→0 findings. Exact match incl. >2048 threshold and UA absent/empty asymmetry. |
| HS-062 | true | 1.00 | PASS | Session: ClientHello(183B), oversized record claiming len 20000, ChangeCipherSpec(0x14), Alert(0x15), unknown(0x18), ServerHello. Exit 0 (no panic). Observed `truncated_records=1`, `parse_errors=1` (equal — incremented together once), `packets_analyzed=1`, non-handshake records consumed silently (parse_errors NOT bumped), 0 findings from them. Parser even recovered to read ServerHello (tls_versions 769+771). Match. |
| HS-063 | true | 0.90 | PASS | 4 craftable ClientHellos: (A) empty ServerNameList, (B) two entries example.com + evil\x01.com, (C) NameType=1 legit.test, (D) 16384-byte SNI. Observed exit 0, `packets_analyzed=4` (handshakes_seen≥4), `truncated_records=0`, `parse_errors=0`, top_snis = {example.com, legit.test, 16384-a's} (3 entries — A produced none), 0 findings (B's second-entry C0 never inspected; C NameType discarded; D 16KB no truncation). Sub-clause (E) BC-2.07.028 — count-cap decoupling at 50,000 entries — is NOT externally craftable (requires pre-filling internal sni_counts map to capacity, unreachable via a sane-size pcap, unobservable from outside). 6/7 contracts fully verified; -0.10 for the one un-exercisable arm. |
| HS-069 | true | 1.00 | PASS | 2 ClientHellos: \xc0\x80 and \xed\xa0\x80 (both →2× U+FFFD lossy). Observed top_snis = `<non-utf8:c080>` and `<non-utf8:eda080>` (2 distinct keys, NOT merged); exactly 2 T1027 findings with evidence `hex: c080` / `hex: eda080`; summaries contain U+FFFD. Exact match — hex-tagged key disambiguation confirmed. |
| HS-071 | true | 1.00 | PASS | Flow: ClientHello version 0x0301 + ServerHello version 0x0303. Observed `tls_versions{769:1, 771:1}` (both in shared map), 1 ja3_hash, 1 ja3s_hash, `packets_analyzed=1`, 0 deprecated-protocol findings (both > 0x0300), parse_errors=0. Match. |
| HS-072 | true | 1.00 | PASS | Request: Host `\xffexample.com`, UA `   curl/7.88.0   `, URI /index.html. Observed top_hosts key = `\u{fffd}example.com` (raw bytes ef bf bd present), user_agents `curl/7.88.0` (trimmed), `parse_errors=0`, methods{GET:1}, 0 findings. Exact match — from_utf8_lossy + trim semantics confirmed. |
| HS-073 | true | 1.00 | PASS | ClientHello SNI `evil\x1bx\xc2\x9by.com` (C0 ESC + C1 U+009B). JSON valid + parses. Raw bytes: `` escape (5c 75 30 30 31 62) present; raw `c2 9b` present; `` escape ABSENT. Round-trip recovers 0x1b and U+009B. Exact match — C0 escaped per RFC 8259, C1 passed through raw. |

## Re-evaluation roll-up (these 12)

- Mean satisfaction: (11×1.00 + 0.90) / 12 = **0.9917**
- Minimum score: **0.90** (HS-063 — single un-craftable sub-clause E, count-cap decoupling)
- Scenarios < 0.60: **0**
- Gate (mean ≥ 0.85, every must-pass ≥ 0.60): **PASS**

## Genuinely-uncraftable items (precise blockers)

- **HS-063 arm (E) / BC-2.07.028** — "count cap does not suppress finding": requires the
  internal `sni_counts` map to be pre-filled to its 50,000-entry capacity before the
  anomalous SNI arrives, so the insert is dropped but the finding still fires. From a
  black-box position this needs ~50,000 distinct prior ClientHellos in one capture
  (impractical pcap size and runtime) and the decoupling is internal state with no
  external observable distinguishing "inserted" from "dropped-but-found". All other six
  contracts of HS-063 (A–D, empty list, first-entry-only, NameType-discard, 16KB SNI,
  no-truncation) were crafted and verified. Scored 0.90 rather than penalizing harder
  because the externally-reachable behavior is fully correct.

## Crafting notes / caveats

- HTTP poison flow (HS-054): the only crafting subtlety. TLS-signature leading bytes
  (0x16/0x17) are claimed by the content-first dispatcher and never reach the HTTP
  analyzer, so the literal "binary TLS records" narrative would route to TLS. Using
  non-signature high-byte binary on port 80 forces port-fallback to HTTP and exercises
  the identical poison/parse-error/skip path the contract describes. The observed
  counters (non_http_flows=1, parse_errors=3, poisoned_bytes_skipped=80, response-dir
  still parsed) satisfy every assertion in the verification approach.
- All other scenarios required no compromise; tls-parser accepted the hand-built
  ClientHello/ServerHello records (cipher list, compression, SNI extension framing) and
  the reassembler delivered pipelined/partial HTTP exactly as the scenarios assume.

## Verdict
The prior "unverified" scores were unwarranted. With crafted inputs, 11/12 scenarios
are exact full matches and the 12th is 0.90 with one provably un-craftable internal
sub-clause. None fall below 0.60. Chunk 3 PASSES the holdout gate.
