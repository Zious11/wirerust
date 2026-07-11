# STORY-163 Authoring Evidence

**Story:** STORY-163 (wave-73, E-11, 2 pts)  
**Tasks:** AC-163-001 (docs-writer-dispatch-guidance.md) + AC-163-002 (pr-manager-merge-auth-guidance.md amendment)  
**Authoring date:** 2026-07-10  
**Author agent:** technical-writer (STORY-163 dispatch)

---

## Decoder.rs Verification Outcome

**Claim being verified:** "VLAN/QinQ/MACsec-tagged ARP frames are handled by the lax
path via `LaxLinkExtSlice::header_len()` and DO produce findings."

**Verdict: CONFIRMED**

Source reads performed before writing the AC-163-001(e) example:

| Region | What was confirmed |
|--------|-------------------|
| `src/decoder.rs:22` | Module-level doc — D-078/D-078b path-independence: non-Ethernet/IPv4 ARP yields `Err("Non-Ethernet/IPv4 ARP frame")` on both strict and lax paths. |
| `src/decoder.rs:157` | `DecodedFrame` doc — same D-078/D-078b path-independence statement in enum-level doc. |
| `src/decoder.rs:196-210` | Strict path ARP arm: `Some(NetSlice::Arp(arp))` is matched; `extract_arp_frame` called; valid Ethernet/IPv4 ARP → `Ok(DecodedFrame::Arp(f))`. VLAN-tagged frames with valid inner ARP payload reach this arm — findings ARE produced. |
| `src/decoder.rs:242-257` | Lax path ARP arm: `Some(LaxNetSlice::Arp(arp))` is matched for truncated VLAN-tagged ARP frames; `extract_arp_frame` called with same logic; findings ARE produced. D-078b path-independence applies. |
| `src/decoder.rs:266` | Inline comment: `D-078 / BC-2.16.009 PC3 / BC-2.16.015 PC-7a/7b` — offset derivation anchor. |
| `src/decoder.rs:291-313` | D-078 VLAN-offset fix: ARP payload offset = 14 bytes (Ethernet2 base) + `LaxLinkExtSlice::header_len()` summed over `lax.link_exts`. Lines 292-294 state: "A single 802.1Q VLAN tag adds 4 bytes (TCI + inner EtherType), QinQ adds 8, and MACsec adds its variable header length — all handled via `LaxLinkExtSlice::header_len()` without hardcoding." |

The inverted claim ("VLAN/QinQ/MACsec-tagged ARP frames produce NO findings") is
REFUTED by lines 196-210 and 242-257, which show both paths reach `extract_arp_frame`
and return `Ok(DecodedFrame::Arp(f))` for valid Ethernet/IPv4 ARP payloads inside
VLAN-tagged frames.

---

## Verification Command Transcripts

### AC-163-001 Verification

**Command 1: file existence**
```
test -f .factory/maintenance/docs-writer-dispatch-guidance.md
```
Output: `FILE EXISTS: YES` (exit 0)

**Command 2: grep for mandate text and REC-006/F-RA-P3-001 example**
```
grep -n "ground-truth\|citation mandate\|file:line\|inversion" \
  .factory/maintenance/docs-writer-dispatch-guidance.md
```
Output (line numbers from grep):
```
21:claims. Finding F-RA-P3-001 (adversary Pass 3, classified HIGH) caught the inversion
51:- **Code anchor:** `file:line` reference (e.g., `src/decoder.rs:291-313`)
56:The dispatch MUST name the expected ground-truth source files explicitly so the writer
73:code or spec behavior against the named file:line anchor before writing.
81:4. Record the `file:line` anchor alongside the claim before submitting the draft.
91:Paraphrasing the summary without verification is the direct cause of factual inversions.
105:1. Named ground-truth source files for this task:
112:   > Source: <file:line or spec anchor you Read>
137:### The incorrect draft (produced without ground-truth verification)
141:frames at all. This was a factual inversion of the code behavior.
144:required to cite a file:line anchor for each behavioral claim, and was not told which
147:### The ground-truth check (what the verification template requires)
195:- **REC-006:** Original sweep recommendation, one-liner that triggered the inversion.
```
Verdict: non-empty, contains mandate text (`ground-truth`, `file:line`, `inversion`) and
REC-006/F-RA-P3-001 example content. AC-163-001 PASS.

---

### AC-163-002 Verification

**Command: grep for new section heading and STORY-163 citation**
```
grep -n "Harness-Classifier\|subagent.*denied\|SUBAGENT\|PG-MERGE-AUTH-SUBAGENT" \
  .factory/maintenance/pr-manager-merge-auth-guidance.md
```
Output:
```
89:## Harness-Classifier Halt: Subagent Merge Denied
91:**Policy reference:** PG-MERGE-AUTH-SUBAGENT-CLASSIFIER
161:  (PG-MERGE-AUTH-SUBAGENT-CLASSIFIER — root precedent codified as STORY-163 AC-163-002.)
198:- **PG-MERGE-AUTH-SUBAGENT-CLASSIFIER:** Root process-gap for harness-classifier halt
```
Verdict: non-empty, contains section heading "Harness-Classifier Halt: Subagent Merge
Denied" (line 89) and STORY-163 citation (line 161). AC-163-002 PASS.

---

## Claims-Citation Table

### AC-163-001 (docs-writer-dispatch-guidance.md) behavioral claims

| Behavioral claim written in the document | File:line anchor Read to verify |
|------------------------------------------|--------------------------------|
| VLAN/QinQ/MACsec-tagged ARP frames reach `extract_arp_frame` on the strict path and produce findings | `src/decoder.rs:196-210` |
| VLAN/QinQ/MACsec-tagged ARP frames reach `extract_arp_frame` on the lax path and produce findings | `src/decoder.rs:242-257` |
| Valid Ethernet/IPv4 ARP payloads produce `Ok(DecodedFrame::Arp(f))` — findings generated | `src/decoder.rs:205, 251` |
| `LaxLinkExtSlice::header_len()` computes correct ARP payload offset for VLAN, QinQ, and MACsec | `src/decoder.rs:291-313` |
| D-078/D-078b establishes path-independence for non-Ethernet/IPv4 ARP on both strict and lax paths | `src/decoder.rs:22, 157` |
| F-RA-P3-001 was classified HIGH (inverted VLAN/QinQ/MACsec claim) in adversary Pass 3 | `.factory/maintenance/sweep-report-2026-07-09.md:326` (HIGH severity attribution); `.factory/code-delivery/maint-2026-07-09/pr-review.md:99-101` (finding existence and resolution) |
| REC-006 one-liner reads "README § Known Limitations: add one sentence on MACsec/VLAN ARP offset detection limitation" | `.factory/maintenance/sweep-report-2026-07-09.md:202` |

### AC-163-002 (pr-manager-merge-auth-guidance.md amendment) behavioral claims

| Behavioral claim written in the document | File:line anchor Read to verify |
|------------------------------------------|--------------------------------|
| Orchestrator executed `gh pr merge` in the main thread under direct user authorization (PR #393) | `.factory/cycles/maint-2026-07-09/lessons.md:32` (L-002 Resolution) |
| pr-manager's prior merge attempt was correctly denied by harness classifier (authorization relayed via teammate-message only) | `.factory/cycles/maint-2026-07-09/lessons.md:28` (L-002 Observation) |
| D-401 case is about orchestrator `AUTHORIZE_MERGE=yes` not being a human grant (policy question) | `.factory/maintenance/pr-manager-merge-auth-guidance.md:12-14` (Background section) |
| pr-manager step-9 cleanup completed after orchestrator confirmed merge SHA | `.factory/cycles/maint-2026-07-09/lessons.md:32` (L-002 Resolution) |

> **Correction (2026-07-11, adversary F-S163P1-001):** three AC-163-002 citations
> originally pointed to `pr-review.md:332-333` (nonexistent — file is 111 lines);
> re-anchored to `lessons.md` after verification. This defect instance is itself an
> argument for the mechanical citation validator (see F-S163P1 observations).

---

## Anchor-Precision Sweep (2026-07-11, post-F-S163P2-001)

**Mandate:** Every file:line citation in both guidance docs and every row of the
claims-citation table above was opened at the cited line and the line text was
confirmed to support the attached claim.

**Finding that triggered this sweep:** F-S163P2-001 (MEDIUM) — the claim
"F-RA-P3-001 classified HIGH" was anchored to
`.factory/code-delivery/maint-2026-07-09/pr-review.md` which names F-RA-P3-001 in
a list at lines 99-101 but never attributes the HIGH severity label. The verifiable
HIGH attribution is at `.factory/maintenance/sweep-report-2026-07-09.md:326`.

### Verification Table

| Citation | Claimed in | Line text verified | Verdict | Correction |
|----------|------------|-------------------|---------|------------|
| `src/decoder.rs:22` | authoring-evidence row 100; guidance §5 table | `//! and lax paths (D-078/D-078b, BC-2.16.009 v1.6 — path-independence, D11).` | YES | — |
| `src/decoder.rs:157` | authoring-evidence row 100; guidance §5 table | `/// on both strict and lax paths (D-078/D-078b, BC-2.16.009 v1.6 — D11);` | YES | — |
| `src/decoder.rs:196-210` | authoring-evidence row 96; guidance §5 table | `Some(NetSlice::Arp(arp))` strict path arm; line 205 `Some(f) => Ok(DecodedFrame::Arp(f))` | YES | — |
| `src/decoder.rs:205` | authoring-evidence row 98; guidance §5 table | `Some(f) => Ok(DecodedFrame::Arp(f)),` — strict path result | YES | — |
| `src/decoder.rs:242-257` | authoring-evidence row 97; guidance §5 table | `Some(LaxNetSlice::Arp(arp))` lax path arm; line 251 `Some(f) => Ok(DecodedFrame::Arp(f))` | YES | — |
| `src/decoder.rs:251` | authoring-evidence row 98; guidance §5 table | `Some(f) => Ok(DecodedFrame::Arp(f)),` — lax path result | YES | — |
| `src/decoder.rs:266` | authoring-evidence row 25 (decoder table); guidance §5 narrative | `// D-078 / BC-2.16.009 PC3 / BC-2.16.015 PC-7a/7b:` | YES | — |
| `src/decoder.rs:291-313` | authoring-evidence row 99; guidance §5 table | Line 291: `// BC-2.16.015 v1.6 / BC-2.16.009 v1.7 / D-078 VLAN-offset fix).`; line 292-294: VLAN/QinQ/MACsec offsets; line 307-313: `LaxLinkExtSlice::header_len()` implementation | YES | — |
| `sweep-report-2026-07-09.md:202` | authoring-evidence row 102 | `\| REC-006 \| LOW (XS) \| README § Known Limitations: add one sentence on MACsec/VLAN ARP offset detection limitation (3rd sweep carry-forward) \|` | YES | — |
| `sweep-report-2026-07-09.md:326` | authoring-evidence row 101 (corrected); guidance Background line ~21 (added); guidance Reference block (added) | `- F-RA-P3-001 HIGH: inverted VLAN/QinQ/MACsec claim — docs-writer paraphrased REC-006 one-liner...` | YES | This line is the authoritative HIGH-severity source; added to all three anchoring sites |
| `pr-review.md:99-101` | authoring-evidence row 101 (pre-correction) | Lines 99-101 list F-RA-P3-001 among historical findings without the HIGH label: `F-RA-P3-001 inverted VLAN/QinQ/MACsec claim` | NO for HIGH claim | Re-anchored HIGH attribution to sweep-report:326; pr-review.md:99-101 retained as evidence of finding existence and resolution only |
| `lessons.md:28` | authoring-evidence row 109 | `**Observation:** During the PR #393 merge step (2026-07-10), the harness auto-mode permission classifier denied \`gh pr merge\` when executed by pr-manager...` | YES | — |
| `lessons.md:32` | authoring-evidence rows 108, 111 | `**Resolution (this run):** The orchestrator executed \`gh pr merge\` in the main conversation thread under direct user authorization given in that thread...` | YES | — |
| `pr-manager-merge-auth-guidance.md:12-14` | authoring-evidence row 110 | Lines 12-14: Background — `During wave-70 (2026-07-07), the boundary between orchestrator-autonomous merges...AUTHORIZE_MERGE=yes flag...D-401 denied...` | YES | — |
| `lessons.md:24-34` | pr-manager-merge-auth-guidance.md lines 168, 206 | Line 24: `### L-002 — Harness-Classifier Halt: Subagent Merge Denied`; line 34: `**Codified:** STORY-163 AC-163-002` — full L-002 entry covered | YES | — |

**Summary:** 1 defective citation found and corrected (row 101 / HIGH attribution).
All other 14 citations verified as accurate. No additional corrections required.

---

## No-Code-Change Confirmation

Files created or modified:
- `.factory/maintenance/docs-writer-dispatch-guidance.md` — NEW (factory-artifacts branch artifact)
- `.factory/maintenance/pr-manager-merge-auth-guidance.md` — AMENDED (factory-artifacts branch artifact)
- `.factory/cycles/wave-73/STORY-163/authoring-evidence.md` — NEW (this file; evidence artifact)

Files NOT modified:
- `src/` — no Rust source files touched
- `tests/` — no test files touched
- `.github/` — no CI YAML touched
- `Cargo.toml` / `Cargo.lock` — not touched
- `CLAUDE.md` — not touched
- Any story files other than reading STORY-163.md (not modified)
