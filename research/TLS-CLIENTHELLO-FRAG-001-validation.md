# DF-VALIDATION-001 Validation — TLS-CLIENTHELLO-FRAG-001

**Finding:** TLS ClientHello fragmented across multiple TLS records is not reassembled by wirerust's TLS analyzer, allowing SNI/JA3 evasion.
**Severity claim under review:** CRITICAL
**Source location:** `src/analyzer/tls.rs` ~L763–792 (`try_parse_records` → `parse_tls_plaintext` per single record)
**Validator:** vsdd-factory research-agent
**Date:** 2026-06-29
**Policy:** DF-VALIDATION-001 (`.factory/policies.yaml`) — finding must be validated against external sources before an issue is filed.

---

## VERDICT: CONFIRMED (technical defect real) — with severity DOWNGRADE recommended

- The behavioral claim is **CONFIRMED**: a TLS ClientHello may legally span multiple TLS records, wirerust parses one record at a time and only looks for a complete `ClientHello` inside a single record, and therefore a record-fragmented ClientHello will yield no SNI and no JA3. This is a genuine functional gap and a genuine evasion vector.
- The **CRITICAL** severity rating does **not** hold for a passive analyzer. Recommended severity: **HIGH** (functional/evasion defect, not a memory-safety or RCE class issue). See §4.

**Confidence:** High on the spec/behavioral facts (primary RFC text verified verbatim; clear prior art). Medium on the precise severity label (severity is a judgment call about wirerust's threat model and the prevalence of the technique against this specific tool).

---

## Q1 — Is record-layer handshake fragmentation spec-permitted? Can ClientHello legally span multiple records?

**Answer: YES. Confirmed against primary sources (verbatim).**

### TLS 1.2 — RFC 5246 §6.2.1 (Fragmentation) — verified verbatim via direct fetch of datatracker

> "The record layer fragments information blocks into TLSPlaintext records carrying data in chunks of 2^14 bytes or less. Client message boundaries are not preserved in the record layer (i.e., multiple client messages of the same ContentType MAY be coalesced into a single TLSPlaintext record, **or a single message MAY be fragmented across several records**)."
> — RFC 5246 §6.2.1

This explicitly permits a single handshake message (e.g. ClientHello) to be split across several records.

### TLS 1.3 — RFC 8446 §5.1 (Record Layer) — quote confirmed against datatracker citation

> "Handshake messages MAY be coalesced into a single TLSPlaintext record, or fragmented across several records, provided that ... they ... do not span key changes."
> Plus the constraints: "Handshake messages MUST NOT be interleaved with other record types." and "Handshake messages MUST NOT span key changes."
> — RFC 8446 §5.1

TLS 1.3 carries the ClientHello unencrypted in a `TLSPlaintext` record (content type `handshake` / 0x16), exactly the path wirerust parses. Fragmentation across records within the same (initial, unencrypted) epoch is legal.

**Caveat / nuance worth recording:** Fragmentation is *legal* but *uncommon* in benign traffic. Typical browser/library ClientHellos are well under 2^14 bytes and are emitted as a single record, so most real flows are parsed correctly today. The defect manifests on (a) deliberately fragmented clients, and (b) genuinely large ClientHellos (many extensions, large key shares, ECH, post-quantum hybrids) that exceed an implementation's record-chunking threshold. The TLS-record-fragmentation circumvention research reports **>90% of TLS servers accept** record-fragmented ClientHellos, confirming this is mainstream-tolerated behavior, not an exotic corner case [3].

---

## Q2 — Is record-fragmented ClientHello a documented real-world evasion technique?

**Answer: YES against SNI-based DPI/censorship (strong prior art). Against JA3/JA4 specifically: technically valid but NOT well-documented as a named technique (incidental, not a published method).**

### SNI / DPI censorship — strong, named prior art
- **"Circumventing the GFW with TLS Record Fragmentation"** (UPB SysSec, 2023) — explicitly defines the technique, splits the ClientHello across records, and reports the GFW "is overchallenged with any kind of TLS record fragmentation"; "it suffices to place any byte of the ClientHello message into a different TLS record." Reports >90% of TLS servers accept the technique [3].
- **Niere et al., "Transport Layer Obscurity: Circumventing SNI Censorship on the TLS-Layer,"** IEEE S&P 2025 — formalizes record-layer manipulation; "for SNI-based censorship circumvention, fragmentations in and around the SNI extension are of particular interest"; fragmentation position is arbitrary [5][12]. ACM poster of the predecessor work [8].
- **QUIC analogue:** "Exposing and Circumventing SNI-based QUIC Censorship of the GFW" (USENIX Security 2025) — GFW "does not reassemble QUIC Initial packets that are split across more than one UDP datagram." Same parser-deficiency class (no higher-layer reassembly), confirming the general pattern that DPI engines often inspect a single lower-layer frame [9].

So: record-fragmented ClientHello as an **SNI-evasion / DPI-evasion** technique is documented, academically published, named, and tooled (research prototypes/proxies).

### JA3/JA4 evasion — nuance
Published JA3/JA4 evasion techniques focus on *content-level* manipulation (cipher/extension reordering, GREASE, mimicry, randomization) [6][7], **not** on record-layer fragmentation. However, JA3/JA4 computation requires a correct parse of the full ClientHello; if a fingerprinting tool only sees the first record and never reassembles, it will produce no fingerprint (or a wrong one). For wirerust specifically — which extracts JA3 only when it sees a complete ClientHello in one record — fragmentation **incidentally** suppresses JA3 too. This is a real consequence but should be described as "incidental JA3 evasion via the same gap," not as an independently documented JA3-evasion method. Do not overstate it.

**No specific CVE** was found that assigns a CVE ID to "fragmented ClientHello evasion" as a class. The body of evidence is academic papers + tool/issue-tracker reports rather than a CVE. This is consistent with it being a *detection-completeness* gap rather than a memory-safety vulnerability.

---

## Q3 — How do mature passive analyzers handle handshake reassembly across records? Is it standard/required?

**Answer: Record-layer handshake reassembly is required for correct SNI+JA3 extraction, and mature analyzers implement it.**

- **Suricata:** ships JA3/JA4 and SNI matching atop its TLS app-layer parser, which sits on its stream-reassembly engine. JA3 over certificate-bearing / multi-record handshakes only works if handshake bytes are reassembled across records; docs/forum guidance assume a complete ClientHello parse [13][15]. (No source states "we do not handle fragmentation" — the feature set implies reassembly.)
- **Wireshark/tshark:** the `tls` dissector has explicit reassembly of TLS records into higher-layer PDUs ("Reassemble TLS records spanning multiple TCP segments" / handshake reassembly), with `tls.*` handshake fields populated from reassembled data [18].
- **Zeek/Bro:** SSL/TLS analyzer logs SNI, cipher lists, cert chains — which requires handshake reassembly across records and segments (strong inference; no contradicting source found). *Flagged as inference, not a quoted source.*
- **Snort:** does **not** natively support JA3 [16]; its TLS preprocessor parses records, but record-fragmentation robustness is not documented. Less authoritative as a reference point.

**Conclusion:** Reassembling handshake messages across record boundaries is treated as the correct, expected behavior by the reference implementations. wirerust's single-record parse is below the de-facto standard for this protocol class. RFC 5246 implementation guidance itself warns implementers to "correctly handle handshake messages that are fragmented to multiple TLS records ... including corner cases" [2].

---

## Q4 — Severity & real-world prevalence: does CRITICAL hold?

**Answer: NO — recommend HIGH.**

Reasoning:
- **In the wild:** Deliberate record fragmentation is a real, published censorship-circumvention technique (GFW) and is broadly server-tolerated [3][5][8][9]. Benign large ClientHellos can also be fragmented. So the trigger is real, not theoretical.
- **But this is a passive analyzer.** The impact is **missed detection / evasion of classification** (no SNI label, no JA3 fingerprint for fragmented flows) — a *detection-completeness/integrity-of-results* failure. There is no memory-safety, RCE, privilege, or DoS dimension. The existing code is bounded (MAX_BUF cap, length-checked drains).
- **CRITICAL** in most security-severity rubrics implies remote code execution, auth bypass, or unbounded resource exhaustion. A coverage gap that lets a crafted flow avoid being fingerprinted does not meet that bar for a Wireshark-like passive tool.
- **HIGH** is appropriate: it is a security-relevant evasion of a core advertised capability (SNI classification VP-005/BC-2.09.007 and JA3 fingerprinting), the technique is documented and easy, and a determined adversary (or even a quirky-but-legitimate client) defeats the feature silently. Silent under-reporting is the worst property here.
- A reasonable argument for **MEDIUM** exists if wirerust's threat model treats fingerprinting as best-effort telemetry rather than an enforcement control. Given the BCs explicitly mandate SNI classification, HIGH is the better fit. Recorded as a defensible range: **HIGH (primary), MEDIUM (floor).**

---

## Q5 — False-positive / interaction risk with snaplen-truncated captures (READER cand-05)

A snaplen-truncated capture (e.g. pcapng EPB where `original_len > captured_len` and the trailing bytes are simply absent) produces a TCP byte stream that **ends early**. To wirerust's record loop this looks like an *incomplete* record: `buf_len < total_record_len` → the current code returns and waits (L702–705). A naive reassembly implementation that instead *assumes more fragments are coming and buffers indefinitely* could:
- Hold partial handshake state forever for a flow that will never complete (memory pressure / state leak), and
- Be unable to distinguish "legitimately fragmented, more records coming" from "capture truncated, nothing more is coming" — they are byte-for-byte identical at the point of truncation.

**Implication for the fix:** record-layer handshake reassembly MUST be bounded and must degrade gracefully:
1. Cap accumulated handshake-reassembly bytes (reuse/parallel the existing `MAX_BUF` discipline) — a fragmented ClientHello that never completes within N bytes is dropped, not buffered unboundedly.
2. Treat `on_flow_close` / end-of-capture with an incomplete handshake buffer as "no ClientHello observed" (the current implicit behavior), not as an error storm.
3. Do **not** emit a malformed/parse-error finding merely because a handshake message was incomplete at capture end — that would create false positives precisely on snaplen-truncated captures (the cand-05 concern). Keep the existing "incomplete → wait, then discard on close" semantics; only *add* cross-record concatenation for the case where the bytes are actually present.

This does not block the fix; it is a design constraint on it.

---

## Recommended minimal fix direction

Implement **TLS handshake-message reassembly across record boundaries** as a thin layer between record-draining and handshake parsing, for content type `0x16` (Handshake) only:

1. When draining a `0x16` record (currently L739–756), instead of parsing that record's payload standalone, **append its handshake-fragment bytes to a per-direction handshake accumulation buffer** in `TlsFlowState` (e.g. `client_hs_buf` / `server_hs_buf`), separate from the existing record-level `client_buf`/`server_buf`.
2. Parse handshake messages out of the *accumulated* buffer using the handshake protocol's own 1-byte type + 3-byte length header: only dispatch a `ClientHello`/`ServerHello` once `accumulated_len >= 4 + msg_body_len`. Consume exactly the bytes of each completed handshake message; leave the remainder for the next record (handles both fragmentation and coalescing — multiple handshake messages in one record).
3. Bound the accumulation buffer (mirror `MAX_BUF`); on overflow, abandon reassembly for that direction (do not buffer unboundedly — addresses Q5).
4. Preserve current semantics for incomplete/truncated input: incomplete handshake at flow close ⇒ no finding, not a parse error (addresses cand-05 false-positive risk).
5. Continue to skip non-`0x16` records exactly as today (the CR-010 guard-before-allocate path is unaffected).

Effort: localized to `tls.rs` (`try_parse_records` + `TlsFlowState`). The TLS-1.3 "MUST NOT span key changes / MUST NOT interleave" rules mean the unencrypted ClientHello is always a clean, single-content-type prefix — no epoch handling needed for the ClientHello case, which is the one that matters for SNI/JA3.

**Suggested test vectors:** ClientHello split byte-at-the-SNI-boundary across two `0x16` records; ClientHello split 1-byte-first-record (mirrors GitHub's observed fragmentation [14]); two handshake messages coalesced in one record; snaplen-truncated ClientHello at flow close (expect: no finding, no error inflation).

---

## Cited sources

Primary / verified:
- RFC 5246 (TLS 1.2) §6.2.1 — fragmentation rule, **fetched & quoted verbatim**: https://datatracker.ietf.org/doc/html/rfc5246
- RFC 8446 (TLS 1.3) §5.1 — coalesce/fragment + MUST NOT span key changes / interleave, quote confirmed: https://datatracker.ietf.org/doc/html/rfc8446
- RFC 6066 (SNI extension): https://www.rfc-editor.org/info/rfc6125/ (SNI/host_name context)

Prior art — evasion:
- [3] Circumventing the GFW with TLS Record Fragmentation (UPB SysSec, 2023): https://upb-syssec.github.io/blog/2023/record-fragmentation/
- [5] Niere et al., Circumventing SNI Censorship on the TLS-Layer (IEEE S&P 2025, PDF): https://censorbib.nymity.ch/pdf/Niere2025a.pdf
- [8] ACM DL entry (TLS record fragmentation poster/paper): https://dl.acm.org/doi/10.1145/3576915.3624372
- [9] Exposing and Circumventing SNI-based QUIC Censorship of the GFW (USENIX Sec 2025): https://gfw.report/publications/usenixsecurity25/en/
- [12] Niere publication record: https://scholar.google.com/citations?user=Z9F6e9cAAAAJ&hl=en

Analyzer behavior / fingerprinting:
- [13] Suricata forum — JA3 fingerprints: https://forum.suricata.io/t/ja3-fingerprints/2567
- [15] Suricata JA3 Lua lib docs: https://docs.suricata.io/en/latest/lua/libs/ja3.html
- [16] Netgate forum — Snort JA3 (unsupported): https://forum.netgate.com/topic/158804/can-is-snort-using-ja3-hashes
- [18] Wireshark TLS display filter reference: https://www.wireshark.org/docs/dfref/t/tls.html
- [6] Fingerprint.com — JA3 limitations / field manipulation: https://fingerprint.com/blog/limitations-ja3-fingerprinting-accurate-device-identification/
- [7] Browserless.io — JA3/JA4 detection & bypass: https://www.browserless.io/blog/tls-fingerprinting-explanation-detection-and-bypassing-it-in-playwright-and-puppeteer
- [17] Fingerprint.com — what is TLS fingerprinting: https://fingerprint.com/blog/what-is-tls-fingerprinting-transport-layer-security/

Implementation pitfalls / fragmented ClientHello in the wild:
- [11] Mbed TLS issue — handshake messages MUST NOT span key changes: https://github.com/Mbed-TLS/mbedtls/issues/10708
- [14] Kubernetes ingress-nginx — fragmented ClientHello (GitHub client, 1-byte TCP fragments): https://github.com/kubernetes/ingress-nginx/issues/11424

Unverified / inference (flagged): Zeek/Bro handshake-reassembly behavior is inferred from its logged TLS fields, not from a quoted Zeek source. Snort's record-fragmentation robustness is undocumented in sources found.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep: RFC fragmentation rules, GFW/SNI evasion prior art, analyzer reassembly behavior, JA3 evasion, prevalence (reasoning_effort=high) |
| Perplexity perplexity_ask | 1 | Verbatim RFC 8446 §5.1 sentence confirmation |
| WebFetch | 4 | Direct fetch of RFC 5246 §6.2.1 (verbatim quote obtained) and RFC 8446 §5.1 (truncated; fell back to perplexity_ask) |
| Read | 2 | Verified source behavior in src/analyzer/tls.rs L700–844 and read the large research result |
| Glob | 1 | Surveyed existing .factory/research for prior register context |
| Training data | 1 area | TLS layering / handshake header (1-byte type + 3-byte length) general structure — cross-checked against RFC text |

**Total MCP tool calls:** 2 (1 perplexity_research, 1 perplexity_ask). Plus 4 WebFetch.
**Training data reliance:** low — all load-bearing spec claims verified against primary RFC text; evasion/analyzer claims sourced to named papers/docs.
**Inconclusive items explicitly flagged:** no CVE ID for the technique class; Zeek behavior is inference; severity label is a judgment within a defensible HIGH–MEDIUM range.
