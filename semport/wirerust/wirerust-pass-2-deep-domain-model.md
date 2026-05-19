# Pass 2 (Domain Model) -- Deepening Round 2 -- wirerust

- Project: wirerust
- Source path: /Users/zious/Documents/GITHUB/wirerust/
- Generated: 2026-05-19
- Pass: 2 (Domain Model) -- Phase B deepening, round 2
- Builds on: wirerust-pass-2-domain-model.md (R1 broad-sweep), wirerust-pass-3-behavioral-contracts.md, wirerust-pass-6-synthesis.md
- Scope: Exactly the 8 carryover targets named in the orchestrator brief. Net-new findings only.

---

## 1. Hallucination-class audit of Pass 2 R1

| # | R1 claim | Class | Verdict |
|---|---|---|---|
| 1 | "MitreTactic 14 enterprise + 2 ICS = 16 variants" (E-27, sec 4) | 2 (miscounted enumerations) | confirmed -- `awk` count over `pub enum MitreTactic { ... }` returns 16; the orchestrator brief's "7 + 4 ICS = 11" was a distractor, not an R1 claim |
| 2 | "InsertResult has 9 variants" (E-13) | 2 (miscounted) | confirmed -- 9 variants visible in segment.rs:7-18 and matched at mod.rs:232-265 |
| 3 | "ThreatCategory closed 8-arm enum" (VO-4) | 2 (miscounted) | confirmed -- findings.rs:41-51 lists exactly 8: Reconnaissance, LateralMovement, C2, Exfiltration, CredentialAccess, Execution, Persistence, Anomaly |
| 4 | "DispatchTarget Http/Tls/None -- module-private" (E-22) | 4 (same-basename conflation across the three mod.rs) | confirmed -- the enum lives in src/dispatcher.rs not in any mod.rs; no conflation occurred |
| 5 | "T1040 and T1071 and T1071.004 and T1573 defined but NEVER emitted" (BR-M-2) | 1 (over-extrapolated tokens) | confirmed -- direct grep across src/ shows zero `Some("T1040"...)` / `Some("T1071"...)` / `Some("T1071.004"...)` / `Some("T1573"...)` literals at any `mitre_technique:` site. The two textual references to `T1071.001` (tls.rs:383) and `T1027` (tls.rs:379) are comments, not emissions |
| 6 | "LateralMovement and C2 are unused -- no analyzer emits them" (E-25) | 1 (over-extrapolated) | **partial retraction: CONV-ABS-1.** Re-grep shows `tests/findings_tests.rs:23` uses `ThreatCategory::C2` (in `test_finding_display`); zero `src/` emission sites for either variant remains correct. R1's verb "no analyzer emits" is correct; the broader phrasing "are unused" overstates -- a unit test references C2 to assert Display formatting. LateralMovement appears only in mitre.rs MitreTactic (different type) and tests/mitre_tests.rs. Net: tests reference C2, no production code emits either |
| 7 | "16 enterprise (14 + 2 ICS)" framed as if all 16 are seeded technique IDs | 5 (inflated metrics) | confirmed -- the 16 is the *tactic* variant count, not the technique-ID count. The technique_info match has exactly 15 IDs (T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573, T0846, T0855, T0856, T0885 = 15). R1 text BR-M-2 says "exactly 16 IDs" -- **this is wrong by one**: counting the source rows it is 15. **Retract as CONV-ABS-2**. |
| 8 | "raw post-from_utf8_lossy bytes" rule named throughout (VO-9, BR-RP-1, INV-3) | 3 (named pattern conflation) | confirmed -- the phrase appears in source doc comment (findings.rs:72-80) and ADR 0003; no invention. |
| 9 | "InsertResult::IsnMissing engine handler is no-op apart from eprintln" (VO-12) | 1 (over-extrapolated) | confirmed -- mod.rs:261-264 in fact runs an empty match arm with a "Programming error" comment. The eprintln is emitted from segment.rs, not the engine. No retraction needed -- R1 was precise. |
| 10 | "Finding.timestamp is reserved but unused; always None across all emission sites" (sec 10 Time policy) | 5 (metrics) | confirmed -- recount: grep `timestamp: None` over `src/` yields 25 occurrences (3 in reassembly/mod.rs alert findings, 2 generate_*_finding, 1 finalize, 9 http.rs, 7 tls.rs, plus 3 in tests-only fixtures). Production-source emission sites with `timestamp:` keyed value: 22. All 22 are `None`. R1's claim holds. |
| 11 | "DnsAnalyzer::analyze returns Vec::new() unconditionally" (BR-DNS-3) | 1 | confirmed |
| 12 | "Engine emits 5 distinct anomaly findings" (P6 anchor sec 3) | 5 | confirmed -- conflicting-overlap (mod.rs:537-547), truncated/depth (549-563), excessive-overlap-alert (275-287), excessive-small-segment (294-306), excessive-out-of-window (315-330), finalize segment-limit (400-417) = 6 distinct sites, sharing the verdict tuples for "5 distinct phenomena." R1 phrasing "5 distinct anomalies" maps to 5 *phenomena* not 6 sites; not a retraction, but a documentation refinement. |

Net retractions: CONV-ABS-1 (LateralMovement/C2 "unused" overstated -- tests use C2), CONV-ABS-2 (technique_info "16 IDs" -- correct count is 15).

---

## 2. Findings on each of the 8 carryover targets

### Target 1 -- INV-1: enumerate every emitted MITRE ID and cross-reference against mitre.rs

**Emission sites (exhaustive, grep `Some("T` across src/analyzer/*, src/reassembly/*):**

| File:Line | Literal ID emitted | Context | In technique_info? |
|---|---|---|---|
| src/reassembly/mod.rs:284 | "T1036" | excessive-overlap alert finding | yes (Masquerading / DefenseEvasion) |
| src/reassembly/mod.rs:543 | "T1036" | generate_conflicting_overlap_finding | yes |
| src/analyzer/http.rs:185 | "T1083" | path traversal detection | yes (File and Directory Discovery / Discovery) |
| src/analyzer/http.rs:214 | "T1505.003" | web shell access | yes (Web Shell / Persistence) |
| src/analyzer/http.rs:229 | "T1046" | admin panel access | yes (Network Service Discovery / Discovery) |
| src/analyzer/http.rs:357 | "T1499.002" | request-side too-many-headers | yes (Service Exhaustion Flood / Impact) |
| src/analyzer/http.rs:415 | "T1499.002" | response-side too-many-headers | yes |
| src/analyzer/tls.rs:403 | "T1027" | SNI AsciiWithControl | yes (Obfuscated Files or Information / DefenseEvasion) |
| src/analyzer/tls.rs:422 | "T1027" | SNI NonAsciiUtf8 | yes |
| src/analyzer/tls.rs:441 | "T1027" | SNI NonUtf8 | yes |

**Distinct emitted set: {T1027, T1036, T1046, T1083, T1499.002, T1505.003}** -- 6 IDs.

**Defined-in-mitre.rs set (technique_info match arms, mitre.rs:99-129):** {T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573, T0846, T0855, T0856, T0885} -- 15 IDs (R1 said 16; correction in CONV-ABS-2).

**Orphans (emitted but not in lookup):** **none.** Every emitted literal resolves to a `(name, tactic)` row.

**Catalogued-but-unused set (in mitre.rs, never emitted):** {T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885} -- **9 IDs**, not 4 (R1 said 4). The R1 omission of T1071.001 and the four ICS IDs (T0846/T0855/T0856/T0885) is a **counting drift**. Pass 2 R1 listed only the four CommandAndControl-tactic IDs as "catalogued but never emitted"; in fact all four ICS technique IDs are also unused. **Net-new finding.**

**INV-1 guard mechanism (re-read tests/mitre_tests.rs:184-217):** The hand-curated array at line 195-205 contains exactly six IDs (`T1083, T1505.003, T1046, T1499.002, T1027, T1036`), matching the emitted set. The test only verifies forward direction (every-listed-ID-resolves). It does NOT verify (a) that newly-emitted IDs are in the array, nor (b) that no emission exists that the array doesn't list. The drift risk R1 named is real and the orchestrator's INV-1 framing is preserved.

### Target 2 -- INV-2: which IDs need upstream MITRE ATT&CK cross-check

The 15 hand-curated IDs in mitre.rs:99-129 each have:
- An ID token
- A human-readable name
- A `MitreTactic` mapping

All three facets are exposed to the upstream-drift risk. Specific items to cross-check against the upstream MITRE STIX 2.1 enterprise + ICS bundles:

| ID | Curated name | Curated tactic | Upstream drift risks |
|---|---|---|---|
| T1027 | "Obfuscated Files or Information" | DefenseEvasion | Sub-techniques T1027.001..T1027.014 added in newer ATT&CK versions -- wirerust references the parent only; if upstream renames, our terminal grouped view shows stale text |
| T1036 | "Masquerading" | DefenseEvasion | Same as above; sub-techniques exist |
| T1040 | "Network Sniffing" | CredentialAccess | name and tactic verified against ATT&CK v15+; recommended check |
| T1046 | "Network Service Discovery" | Discovery | renamed by MITRE from "Network Service Scanning" at some point -- needs confirmation we have the *current* name |
| T1071 | "Application Layer Protocol" | CommandAndControl | parent technique; sub-techniques T1071.001..T1071.005 |
| T1071.001 | "Web Protocols" | CommandAndControl | confirm exact name |
| T1071.004 | "DNS" | CommandAndControl | confirm exact name |
| T1083 | "File and Directory Discovery" | Discovery | confirm |
| T1499.002 | "Service Exhaustion Flood" | Impact | confirm |
| T1505.003 | "Web Shell" | Persistence | confirm parent T1505 is still "Server Software Component" |
| T1573 | "Encrypted Channel" | CommandAndControl | sub-techniques T1573.001/.002 exist (Symmetric/Asymmetric Cryptography) |
| T0846 | "Remote System Discovery" | Discovery (deliberately merged into enterprise) | ICS matrix uses TA0102; merge into enterprise Discovery is a wirerust design choice (mitre.rs:115-119) |
| T0855 | "Unauthorized Command Message" | IcsImpairProcessControl | confirm |
| T0856 | "Spoof Reporting Message" | IcsImpairProcessControl | confirm |
| T0885 | "Commonly Used Port" | CommandAndControl (deliberately merged) | confirm |

**Recommendation (do not fix):** cross-reference at minimum the 6 actively-emitted IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003) against `https://attack.mitre.org/techniques/enterprise/`; ICS IDs less critical because none are emitted.

### Target 3 -- INV-3: demonstrate silent violation, propose RawBytesString newtype

**Demonstration of silent violation.** Current `Finding.summary: String` is a bare `String`. If a hypothetical new analyzer wrote:

```rust
self.all_findings.push(Finding {
    category: ThreatCategory::Anomaly,
    verdict: Verdict::Likely,
    confidence: Confidence::Medium,
    // BUG: Debug-format escapes the attacker-controlled bytes
    summary: format!("Suspicious SNI {:?}", attacker_bytes),
    evidence: vec![format!("hex: {:?}", attacker_bytes)],
    mitre_technique: None,
    source_ip: None,
    timestamp: None,
});
```

then a Cyrillic SNI `пример.рф` becomes `"\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}.\u{440}\u{444}"` in `summary` -- and this escaped form is what ends up in JSON output verbatim (the JSON reporter does no further transformation; the doubled backslash means `\u{43f}` in the Rust string becomes the literal seven characters `\u{43f}` in JSON, not the Cyrillic codepoint). Every reporter from that day forward shows mangled forensic data -- Russian-speaking analysts can no longer read the SNI. The same trap was hit by PR #49 (ADR 0003 audit notes).

The compiler accepts this construction silently because `format!("{:?}", _)` returns a `String`, which is what `summary` requires.

**Where this would slip in.** The exact code-review checkpoints that would catch this today:
- Pass 5 convention CNV-PAT raw-vs-display (paper-only)
- tests/reporter_tests.rs control-byte SNI regression (catches existing analyzers but not new ones)
- ADR 0003 doc comment on `Finding`'s `Display` impl (paper-only)

None of these are mechanical. A new analyzer added by a contributor who never reads ADR 0003 will not trigger any failure.

**Proposed type-level enforcement (description only -- do not implement).**

```text
pub struct RawBytesString(String);

impl RawBytesString {
    pub fn from_lossy(bytes: &[u8]) -> Self { Self(String::from_utf8_lossy(bytes).into_owned()) }
    pub fn from_format_args(s: String) -> Self { Self(s) }  // explicit "I claim this is raw"
    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Debug for RawBytesString {
    // Debug must NOT escape -- it returns the raw underlying chars verbatim,
    // surrounded by a thin marker like `RawBytes("...")` for source-level
    // disambiguation but without char::escape_default invocation on the inner
    // text. Concretely: write!(f, "RawBytes({})", self.0).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

impl fmt::Display for RawBytesString {
    // Display also passes raw.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

impl Serialize for RawBytesString {
    // Delegates to String -- serde_json owns the JSON RFC 8259 escape.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { self.0.serialize(s) }
}
```

Then change `Finding.summary: String` to `Finding.summary: RawBytesString` and `evidence: Vec<String>` to `Vec<RawBytesString>`. Analyzer authors must call `RawBytesString::from_format_args(format!(...))` explicitly; if they accidentally write `format!("{:?}", bytes)` and pass the result, the Debug impl on `RawBytesString` is also raw-preserving, so the escape mistake is neutralized at the type level. The contract becomes mechanical -- you cannot build a `RawBytesString` whose inner content is char-escaped without going through a constructor that has the contract in its docs.

**Properties this design preserves:**
- ADR 0003 raw-data invariant: enforced at type construction time
- JSON RFC 8259 escaping: still delegated to serde
- Terminal escaping: TerminalReporter calls `escape_for_terminal(rbs.as_str())` -- one extra accessor, no behavior change
- Backwards-compat for tests: the test fixtures can use a `From<String>` impl

### Target 4 -- ThreatCategory::LateralMovement and ::C2 disposition

**Grep confirmation:**
- `ThreatCategory::C2` is referenced exactly **once** in src/ and that is the definition (`findings.rs:45`). It is referenced once in tests/ (`tests/findings_tests.rs:23` in `test_finding_display`).
- `ThreatCategory::LateralMovement` is referenced exactly **once** in src/ and that is the definition (`findings.rs:44`). It has zero references in tests/.

**Production emission sites: zero** for both. Confirmed by R1 and re-confirmed here.

**Disposition options (do not pick):**

1. **Delete both variants.** Pros: shrinks the closed enum to 6 variants matching actual emission breadth; removes dead UI surface; makes `ThreatCategory` align with what analyzers actually classify. Cons: tests/findings_tests.rs:23 breaks; downstream JSON consumers who happen to anticipate these strings would silently lose them; the README/docs section listing "threats categorized" may reference these. Removal is a SemVer-major (enum is `pub`, not `#[non_exhaustive]`).

2. **Wire to future detector.** Pros: encodes design intent for upcoming beacon-detector (CLI flag `--beacon` is unwired, P6 anti-pattern #2; Q-A8 mentions "future TLS-on-port-80 covert"). C2 is the natural category for beacon, periodic-callback, and DGA detections. LateralMovement is the natural category for SMB, RDP, WMI lateral-move signals (note: there is a `smb3.pcapng` fixture). Cons: keeps two dead variants until that work lands; adds carrying cost to every reporter switch.

3. **Leave for forward-compat.** Pros: zero churn; the enum is small; consumers who Debug-format see the variant names; removing-then-adding-back would be a SemVer-major plus a SemVer-major. Cons: makes the enum a fiction relative to actual behavior; convention drift hotspot.

**Asymmetry note (new):** the parallel asymmetry is that `MitreTactic` is `#[non_exhaustive]` (mitre.rs:22) precisely so that adding variants is non-breaking. `ThreatCategory` is not, which means option 2 ("wire to future detector") doesn't actually need to keep the variants today -- they could be deleted and re-added when the detector lands, *if* `ThreatCategory` were also `#[non_exhaustive]`. The fact that `ThreatCategory` is closed makes "delete now, re-add later" carry a SemVer cost it wouldn't otherwise. This raises a P5-relevant question: should `ThreatCategory` also be `#[non_exhaustive]`?

### Target 5 -- catalogued but never emitted: verify full set; identify inverse

**Catalogued (technique_info match arms):** 15 IDs as listed in Target 1.

**Emitted (Some("T...") literals):** 6 distinct IDs: T1027, T1036, T1046, T1083, T1499.002, T1505.003.

**Catalogued-but-never-emitted (9 IDs, full list):**

| ID | Curated name | Notes -- why might it be staged? |
|---|---|---|
| T1040 | Network Sniffing | candidate for future passive-recon detector; no current emission |
| T1071 | Application Layer Protocol | parent tech; if a detector ever emits the parent rather than a child sub-technique |
| T1071.001 | Web Protocols | natural successor to the existing HTTP analyzer covert-channel detector that doesn't yet exist; comment at tls.rs:383 reads "abuse over the channel (T1071.001)" but the corresponding emission was intentionally **not** added (it would be T1027 instead per the SNI conformance design) |
| T1071.004 | DNS | candidate for future DNS-tunneling detector (DnsAnalyzer is counter-only today) |
| T1573 | Encrypted Channel | candidate for TLS-on-non-standard-port covert detector |
| T0846 | Remote System Discovery (ICS) | no ICS detector exists today; staged for ICS protocol analyzers |
| T0855 | Unauthorized Command Message (ICS) | same |
| T0856 | Spoof Reporting Message (ICS) | same |
| T0885 | Commonly Used Port (ICS) | same |

**Inverse check (emitted-but-not-catalogued, which would be a bug):** **zero.** Every emitted literal is in the catalogue. The hand-curated test at tests/mitre_tests.rs:185-217 also gates against this for the 6 currently-emitted IDs.

**Cross-reference with INV-1:** R1's claim ("T1040, T1071, T1071.004, T1573 unused") was incomplete -- it missed T1071.001 and all 4 ICS IDs. **Net-new finding: the unused-catalogue set is 9, not 4.** The ICS variants raise a separate question: is wirerust deliberately staging ICS support, or are these vestigial?

### Target 6 -- Finding.timestamp tabulation

**Every production emission site with explicit `timestamp:` field (22 sites, all `None`):**

| File:Line | Finding kind | timestamp value |
|---|---|---|
| src/reassembly/mod.rs:286 | overlap-alert | None |
| src/reassembly/mod.rs:305 | small-segment-alert | None |
| src/reassembly/mod.rs:329 | out-of-window-alert | None |
| src/reassembly/mod.rs:415 | finalize segment-limit summary | None |
| src/reassembly/mod.rs:545 | generate_conflicting_overlap_finding | None |
| src/reassembly/mod.rs:561 | generate_truncated_finding | None |
| src/analyzer/http.rs:187 | path traversal | None |
| src/analyzer/http.rs:216 | web shell access | None |
| src/analyzer/http.rs:231 | admin panel access | None |
| src/analyzer/http.rs:246 | unusual HTTP method | None |
| src/analyzer/http.rs:260 | HTTP/1.1 without Host | None |
| src/analyzer/http.rs:274 | long URI | None |
| src/analyzer/http.rs:288 | empty User-Agent | None |
| src/analyzer/http.rs:359 | request too-many-headers | None |
| src/analyzer/http.rs:417 | response too-many-headers | None |
| src/analyzer/tls.rs:405 | SNI AsciiWithControl (T1027) | None |
| src/analyzer/tls.rs:424 | SNI NonAsciiUtf8 (T1027) | None |
| src/analyzer/tls.rs:443 | SNI NonUtf8 (T1027) | None |
| src/analyzer/tls.rs:471 | ClientHello weak cipher | None |
| src/analyzer/tls.rs:492 | ClientHello deprecated version | None |
| src/analyzer/tls.rs:534 | ServerHello weak cipher | None |
| src/analyzer/tls.rs:555 | ServerHello deprecated version | None |

Total: 22 production emission sites; 22 of 22 set `timestamp: None`.

**JSON serialization shape when None.** findings.rs:68-69 reads `#[serde(skip_serializing_if = "Option::is_none")] pub timestamp: Option<DateTime<Utc>>`. So when timestamp is None, the `timestamp` key is **omitted entirely from JSON**. The serde derive on `Finding` uses default rules for other fields, so all other Option fields (`mitre_technique`, `source_ip`) currently render as JSON `null` when None (no skip attribute). **Net-new finding: serialization is asymmetric** -- `timestamp` is special-cased to be elided while `mitre_technique` and `source_ip` render as `null`. Downstream JSON consumers cannot infer whether `timestamp` is "always-None" or "occasionally-populated" by inspecting one finding -- it's gone from the wire format. This affects API stability claims.

**Terminal output change if populated.** Re-read reporter/terminal.rs:157-205: the `render_finding_prefix` and `render_finding_flat`/`render_finding_grouped` functions write `[category] verdict (confidence) - summary`, evidence lines, and an optional `MITRE:` line. **They never reference `f.timestamp`.** So even if `timestamp` were populated, terminal output would not change. The field is fully ignored at the display layer in both reporters today; populating it would only affect JSON output (and only by re-introducing the `timestamp` key in the JSON envelope).

**Where to source a value.** ParsedPacket (decoder.rs:34-42) does not carry a timestamp. Only `RawPacket.timestamp_secs: u32` at the reader layer carries time; it is passed to `TcpReassembler::process_packet` as the `timestamp: u32` parameter (mod.rs:111) but it is not propagated into emitted `Finding`s. To wire `Finding.timestamp` would require either (a) plumbing `timestamp_secs` into `ParsedPacket` and then into every analyzer's `on_data`/`analyze`, or (b) caching the most-recent timestamp on each per-flow state container. Both are non-trivial.

### Target 7 -- main.rs accepts .pcapng even though reader rejects: error message trace

**Resolution path (main.rs:236-256):**
```rust
fn resolve_targets(target: &Path) -> Result<Vec<std::path::PathBuf>> {
    if target.is_file() { return Ok(vec![target.to_path_buf()]); }   // line 237-239
    if target.is_dir() {
        // collect *.pcap or *.pcapng files (line 247)
        ...
    }
    anyhow::bail!("Target not found: {}", target.display());
}
```

**Open path (main.rs:104-105):**
```rust
let source = PcapSource::from_file(path)
    .with_context(|| format!("Failed to read {}", path.display()))?;
```

**Inner failure (reader.rs:21-22):**
```rust
let mut pcap_reader = PcapReader::new(reader).context("Failed to parse pcap header")?;
```

**Trace.** When a `.pcapng` file is collected, `PcapReader::new(reader)` is called against the classic-pcap parser; pcapng's section-header-block magic `0x0A0D0D0A` does not match the pcap magic `0xa1b2c3d4` (or its byte-swapped form), so pcap-file returns an error. `anyhow::Context::context("Failed to parse pcap header")` wraps it; then main.rs wraps again with `"Failed to read <path>"`; then `?` propagates to `run_analyze`'s `Result<()>` return; then main returns the error.

**User-visible message (run anyhow's default chain renderer):**
```
Error: Failed to read path/to/capture.pcapng

Caused by:
    0: Failed to parse pcap header
    1: <inner pcap-file error message, e.g., "Invalid pcap file: bad magic">
```

**Specificity assessment.** The error chain says "Failed to parse pcap header" -- it does NOT say "this is pcapng; wirerust only supports classic pcap." A user who passed `*.pcapng` will see a header-parse error and may reasonably conclude their file is corrupt rather than format-incompatible. The error message is **generic**, not pcapng-specific.

**Behavior-from-cli surface.** Net-new finding: the broken behavior is asymmetric -- for a single-file target (`wirerust analyze foo.pcapng`), `resolve_targets` returns `[foo.pcapng]` without extension filtering (line 237-239 returns the path unchanged for any file). For a directory target, `resolve_targets` filters to `.pcap` or `.pcapng` extensions (line 247). Either way, the reader rejects pcapng. The mismatch is: single-file accepts any extension (including `.unknown`); directory expansion accepts only `.pcap` or `.pcapng`. So passing a `.pcap.gz` file directly would hit `PcapReader::new` which would emit the same generic header-parse error; passing a directory containing `.pcap.gz` would silently drop them.

**Recommendation surface (do not fix):** make the error message pcapng-aware ("wirerust does not currently support pcapng; convert with `editcap -F libpcap`") or filter `.pcapng` out of the directory glob and bail at resolution time with a clear message.

### Target 8 -- JsonReporter::render .unwrap() at reporter/json.rs:36

**Re-read (json.rs:24-36):**
```rust
let output = json!({
    "summary": {
        "total_packets": summary.total_packets,
        "total_bytes": summary.total_bytes,
        "skipped_packets": summary.skipped_packets,
        "unique_hosts": summary.unique_hosts(),
        "protocols": protocols,            // HashMap<String, u64>
        "services": summary.service_counts(),  // HashMap<String, u64>
    },
    "findings": findings,                   // &[Finding]
    "analyzers": analyzer_summaries,        // &[AnalysisSummary]
});
serde_json::to_string_pretty(&output).unwrap()
```

**Failure modes of `serde_json::to_string_pretty`:**
1. **Serialize trait failure on a constituent type.** This is the only path to an `Err`. `serde_json::Error` is returned when a serializer reports its own error.
2. **I/O failure on the underlying writer.** `to_string_pretty` builds a `Vec<u8>` writer internally that cannot fail.
3. **Non-string Map key.** `serde_json` requires map keys to serialize as strings. A `HashMap<Protocol, _>` would error -- but the reporter already converts to `HashMap<String, u64>` at lines 17-22 specifically to avoid this. This was a deliberate design choice; the comment at line 17 reads "Convert Protocol (non-string) keys to strings for JSON compatibility."

**What's passed to `to_string_pretty`:**
- `summary.total_packets/total_bytes/skipped_packets: u64` -- numeric, infallible.
- `summary.unique_hosts(): Vec<IpAddr>` -- IpAddr serializes via Display to a string, infallible.
- `protocols: HashMap<String, u64>` -- string-keyed, infallible.
- `services: HashMap<String, u64>` -- same.
- `findings: &[Finding]` where `Finding` is `#[derive(Serialize)]` over: 3 closed enums (string-tagged), 2 String fields, 1 `Option<String>`, 1 `Option<IpAddr>`, 1 `Option<DateTime<Utc>>` -- all infallible.
- `analyzer_summaries: &[AnalysisSummary]` where the inner `detail: HashMap<String, serde_json::Value>` already-IS valid JSON-in-memory; serializing a `Value` cannot fail.

**Verdict: the `.unwrap()` is infallible-by-construction** given the current type system. There is no input a packet capture could provide that would cause `to_string_pretty` to return `Err` here, because the failure modes (non-string keys, custom Serialize impls that return errors, I/O on the writer) are all eliminated upstream.

**Conditions under which it would panic in the future (drift risk):**
- If `Finding` ever grows a field whose `Serialize` impl can fail (e.g., a custom newtype that validates its bytes during serialization). Today none exist.
- If `summary.protocol_counts()` ever returns a `HashMap` whose `K` is not string-serializable -- but the explicit conversion at line 17-22 was added precisely to prevent this.
- If `analyzer_summaries.detail` ever holds a `serde_json::Value` constructed from invalid data -- but `serde_json::Value` is by construction valid JSON.

**Recommendation surface (do not fix):** convert the call to `.expect("JsonReporter inputs are infallible-by-construction; see ADR 0003")` so the panic message is informative if the invariant ever drifts; or return `Result<String>` from `Reporter::render` and let `JsonReporter` propagate. Pass 4 marked it MEDIUM-confidence violation -- this audit converts it to **LOW** (NFR-VIO-008 is a documentation paper-cut, not a runtime risk).

**Net-new finding (subtle):** the existing `protocols: HashMap<String, u64>` conversion at lines 17-22 uses `format!("{k:?}")` -- this means JSON output keys are the **Debug-format** of `Protocol`. For `Protocol::Other(u8)` this produces strings like `"Other(56)"` not `"Other:56"` or `"56"`. Downstream JSON consumers must understand they're seeing Rust Debug output as map keys -- which is a forensic-tooling oddity, not a panic risk, but it's worth flagging for the eventual JSON schema work.

---

## 3. Delta Summary

Net-new findings vs Pass 2 R1:

- **Catalogued-but-unused MITRE set is 9 IDs, not 4** (R1 missed T1071.001 and four ICS IDs T0846/T0855/T0856/T0885). Target 5.
- **technique_info has 15 IDs, not 16** (R1 BR-M-2 counts wrong). CONV-ABS-2. Target 1.
- **ThreatCategory::C2 is referenced by tests/findings_tests.rs:23** -- R1's "unused" overstated. CONV-ABS-1. Target 4.
- **Finding.timestamp is special-cased in JSON serialization** (`skip_serializing_if = "Option::is_none"`); other Option fields (mitre_technique, source_ip) are not. Asymmetric wire-format shape. Target 6.
- **Terminal reporter never references Finding.timestamp** -- populating the field would have zero terminal-output effect. Target 6.
- **resolve_targets is asymmetric**: single-file accepts any extension; directory expansion accepts only .pcap/.pcapng. Mismatch with reader's classic-pcap-only contract is visible only via generic "Failed to parse pcap header" error. Target 7.
- **JsonReporter::render unwrap is genuinely infallible-by-construction** given the current type system; the `.unwrap()` is a documentation paper-cut, not a runtime risk. Target 8.
- **JSON protocol-map keys are Debug-format strings** (e.g., `"Other(56)"`) -- a forensic-tooling oddity inherited from line 17-22 conversion. Target 8.
- **ThreatCategory closed-enum vs MitreTactic non_exhaustive asymmetry** raises a P5-relevant question: should ThreatCategory also be non_exhaustive? Target 4 (subsidiary).
- **Proposed RawBytesString newtype design** with raw-preserving Debug impl -- describes how INV-3 could become type-level enforced without changing JSON or terminal output behavior. Target 3.

Substantive count: 9 net-new findings. CONV-ABS retractions: 2.

---

## 4. Novelty Assessment

Novelty: SUBSTANTIVE

The Target-5 finding (9-IDs-unused, not 4) materially expands the unused-catalogue set R1 reported. The Target-1 retraction (15 IDs not 16) corrects an enum-count claim. The Target-6 finding (`timestamp` is silently elided from JSON; terminal ignores it) changes the spec for the JSON wire format -- it is not just refinement, it discloses an undocumented serialization-shape decision. The Target-3 RawBytesString proposal is a concrete formalization of INV-3 that the broader passes can build on. The Target-7 trace produces a specific user-visible error string that the brief and PRD work will need to cite. These would change how someone spec'd the system -- the test for SUBSTANTIVE is met.

---

## 5. Remaining gaps / next candidate scope (Pass 2 R3 if needed)

The 8 carryover targets are addressed. Remaining domain-model gaps that a Pass 2 R3 could pick up:

1. **Function-level depth on the 22 emission sites' exact summary template strings.** R2 tabulated the locations and IDs; it did not record the verbatim template for each summary. A spec-creation step would want a `(emission_site -> exact summary template -> exact evidence array shape)` matrix.
2. **SniValue disambiguation rules** when an SNI has both control bytes AND non-ASCII UTF-8 (e.g., `caf\x01\xc3\xa9`) -- which variant wins? P6 §6.4 flagged this as missing from R1; not part of the 8 R2 targets. Re-read tls.rs:173-242 needed.
3. **HttpFlowState reset rules** when `_poisoned` becomes true: is `counted_as_non_http` ever reset? Per-direction or per-flow? P6 §6.4 flagged.
4. **Branch-ordering in dispatcher** when a 5-byte buffer starts with `\x16\x03\x01GET` -- does TLS win or HTTP? P6 §6.4 flagged.
5. **Drop impls audit** for the 10 state containers -- are any relying on explicit cleanup (finalize, on_flow_close) for correctness vs convenience?
6. **ParsedRequest borrow lifetimes** -- are `host`/`user_agent` Options encoding header-presence-with-empty vs header-absent? P6 §6.4 flagged.

If Pass 2 R3 is run, the candidate scope is items 1-3 (highest value: emission-site verbatim shapes, SniValue disambiguation, HttpFlowState reset semantics). Items 4-6 are lower yield.

---

## State Checkpoint

```yaml
pass: 2
round: 2
status: complete
sub_pass: deep_domain_model
targets_addressed: 8
net_new_findings: 9
conv_abs_retractions: 2
hallucination_classes_audited: 5
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_action: pass_2_round_3_optional_or_proceed_to_other_passes
resume_from: null
```
