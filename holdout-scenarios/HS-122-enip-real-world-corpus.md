---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.010.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.011.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.018.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.019.md
  - .factory/stories/STORY-134.md
  - .factory/stories/STORY-135.md
  - .factory/stories/STORY-138.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-122"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.010
  - BC-2.17.011
  - BC-2.17.018
  - BC-2.17.019
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Two real-world corpus pcap sources needed: (1) a known-good ENIP device commissioning/normal operation capture (no attack commands — expected result: no T0858/T0816 findings); (2) a known-problematic pcap with ENIP ListIdentity scans or CIP Stop commands (expected result: T0846 and/or T0858 findings). See public corpus sources listed in scenario body."
---

# Holdout Scenario: Real-World ENIP Corpus — Known-Good (Low False Positive) and Known-Problematic (Known Findings)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

This is the real-world corpus holdout for the EtherNet/IP analyzer, required by the product
specification's real-world corpus mandate. It exercises two public corpus sources:

1. **Known-good corpus:** A publicly-available PCAP of normal EtherNet/IP device enumeration
   and commissioning traffic (no CIP Stop, No CIP Reset, no write burst) from an ICS lab or
   documented research environment. Expected result: zero T0858/T0816/T0836 findings (no
   attack commands), and at most a few T0846 findings if ListIdentity frames are present
   (ListIdentity is normal during device enumeration). The false-positive rate for
   attack-technique findings (T0858/T0816/T0836) must be zero on this corpus.

2. **Known-problematic corpus:** A publicly-available PCAP or researcher-released capture
   that includes EtherNet/IP ListIdentity scans, CIP service exploration, or documented
   attack activity (e.g., a pentest capture or an ICS research tool trace). Expected result:
   specific known detections (T0846 for ListIdentity, T0888 for Identity Object reads) must
   appear. The true-positive rate for recon technique findings must be non-zero.

### Corpus Sources (Public, Reproducible)

**Known-good source options (evaluator selects one):**

1. **Wireshark EtherNet/IP sample captures** — The Wireshark project maintains a public PCAP
   archive at https://wiki.wireshark.org/SampleCaptures and https://github.com/khenderick/
   wireshark-sample-pcaps. ENIP/CIP samples (search "EtherNet/IP") typically contain
   normal device enumeration and implicit I/O traffic. These are well-maintained and
   widely used; false-positive risk is very low.
   URL: https://wiki.wireshark.org/Protocols/enip (and linked sample files).

2. **ENIP/CIP samples from the ControlThings/Shodan-adjacent research community** — Packet
   captures released alongside EtherNet/IP security research papers (e.g., the Project
   Basecamp captures from Digital Bond, if publicly available). These are from legitimate
   research environments, typically without attack commands.

**Known-problematic source options (evaluator selects one):**

1. **Cpppo EtherNet/IP simulator test captures** — The `cpppo` Python EtherNet/IP
   implementation (https://github.com/pjkundert/cpppo) includes integration tests that
   generate CIP service traffic including GetAttribute reads to the Identity Object (class
   0x01). These captures are expected to trigger T0888 (Identity Object read) and/or T0846
   (ListIdentity) but NOT T0858 or T0816 (no Stop or Reset in normal cpppo operation).

2. **NMAP EtherNet/IP probe traces** — NMAP includes an EtherNet/IP discovery module that
   sends ListIdentity requests. A capture of an NMAP scan against an EtherNet/IP device
   would produce T0846 findings. Any security researcher with NMAP can reproduce this.

3. **CyberSecure ICS challenge captures** — If the evaluator has access to
   ICS-focused CTF or public dataset captures (e.g., from Defcon ICS Village or
   S4 Conference), those may include documented CIP Stop or Reset sequences. The evaluator
   should use captures with documented expected detections.

### Case A — Known-Good Corpus: Zero Attack-Technique Findings (False Positive Check)

1. The evaluator obtains a public EtherNet/IP normal-operation PCAP (see sources above).
2. The evaluator runs: `wirerust analyze <known_good_enip.pcap> --enip --json`
3. The tool exits 0.
4. Expected outcome:
   - **ZERO T0858 findings** (no CIP Stop in normal commissioning traffic).
   - **ZERO T0816 findings** (no CIP Reset in normal traffic).
   - **ZERO T0836 findings** (no write burst in normal traffic; a few individual SetAttribute
     operations may exist, but they are below the 50-write-per-second threshold).
   - T0846 findings are ALLOWED (ListIdentity scans are common during commissioning and
     are expected, not false positives). Their presence confirms the analyzer is running.
   - T0888 findings are ALLOWED if Identity Object reads are present in the corpus.
   - T0814 findings must be ZERO unless the corpus genuinely contains malformed ENIP frames.
5. **Verdict:** Zero false T0858/T0816/T0836 findings on known-good commissioning corpus.

### Case B — Known-Problematic Corpus: Expected Detections Present (True Positive Check)

1. The evaluator obtains a public EtherNet/IP scan/research PCAP (see sources above).
2. The evaluator runs: `wirerust analyze <known_problematic_enip.pcap> --enip --json`
3. The tool exits 0.
4. Expected outcome (specific expected findings depend on the chosen corpus):
   - If the corpus contains ListIdentity frames: **AT LEAST ONE T0846 finding** must appear.
   - If the corpus contains CIP Identity Object GetAttribute reads: **AT LEAST ONE T0888
     finding** must appear.
   - If the corpus contains a documented CIP Stop: **AT LEAST ONE T0858 finding** must appear.
   - The evaluator selects a corpus with at least one of the above documented behaviors and
     verifies the corresponding finding appears.
5. **Verdict:** At least one known detection fires on known-problematic corpus.

### Case C — Tool Handles Real Captures Without Panic

Both Case A and Case B must complete without panic, OOM, or crash. Real-world PCAP files
may contain fragmented TCP segments, retransmissions, or other artifacts. The analyzer must
process them gracefully.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.010 | T0846 emitted per-flow for ListIdentity | Case B: T0846 appears on scan corpus |
| BC-2.17.011 | T0858 NOT emitted in normal operation | Case A: T0858 absent on known-good corpus |
| BC-2.17.018 | T0814 NOT emitted without actual malformed frames | Case A: T0814 absent on well-formed normal corpus |
| BC-2.17.019 | Port 44818 dispatched to ENIP analyzer | Cases A/B: ENIP analysis runs on port 44818 traffic |

<!-- HIDDEN TRACEABILITY: Real-world corpus holdout per product specification mandate; case A tests false-positive rate for attack-technique findings; case B tests true-positive rate -->

## Fixture Creation Obligation

**F4 is NOT required to create synthetic fixtures for this scenario.** Instead, F4 must
identify and document which public PCAP corpus was used for each case, including:
- The exact URL or repository path of the chosen corpus.
- The checksum (MD5 or SHA256) of the PCAP file used.
- The expected findings list for Case B, verified by manual inspection.

If no suitable public corpus is available at F4 time, the evaluator may create realistic
synthetic corpus fixtures using cpppo, the Wireshark EtherNet/IP dissector test vectors, or
NMAP EtherNet/IP probe captures from a local testbed. Document the fixture-generation method.

## Verification Approach

```bash
# Case A (known-good):
wirerust analyze known_good_enip.pcap --enip --json | jq '[.findings[] | select(.mitre_techniques | contains(["T0858"]))] | length'
# Expect: 0

wirerust analyze known_good_enip.pcap --enip --json | jq '[.findings[] | select(.mitre_techniques | contains(["T0816"]))] | length'
# Expect: 0

wirerust analyze known_good_enip.pcap --enip --json | jq '[.findings[] | select(.mitre_techniques | contains(["T0836"]))] | length'
# Expect: 0

# Case B (known-problematic — adjust technique to match chosen corpus):
wirerust analyze known_problematic_enip.pcap --enip --json | jq '[.findings[] | select(.mitre_techniques | contains(["T0846"]))] | length'
# Expect: >= 1
```

## Evaluation Rubric

- **False positive rate for attack findings (T0858/T0816/T0836) on known-good** (weight: 0.45):
  Zero false positives on commissioning traffic. Any T0858/T0816 finding on a corpus with no
  attack commands is a HIGH-severity false positive.
- **True positive rate on known-problematic** (weight: 0.35): At least one expected detection
  fires. If zero findings appear on a corpus with documented ListIdentity/CIP scans, the
  analyzer is not running or a detection path is broken.
- **No crash on real-world captures** (weight: 0.20): Both cases exit 0 without panic. Real
  PCAP may have fragmentation, retransmits, or overlapping segments.

## Failure Guidance

"HOLDOUT LOW: HS-122 — real-world corpus failure. If Case A produces T0858/T0816 findings
on normal commissioning traffic, the detection threshold is too low or there is a false CIP
Stop/Reset classification (verify classify_cip_service and the 0x00B2-only gate). If Case B
produces zero T0846 on a known-scan corpus, either the ENIP analyzer is not receiving
traffic (port dispatch broken), or the ListIdentity classifier is not working (verify
command=0x0063 classification). If the tool panics on real-world PCAP, see BC-2.17.016
(frame-walk loop robustness) and BC-2.17.001 (truncation safety)."
