---
document_type: f5-scoped-adversarial-trajectory
cycle: feature-arp-v0.7.0
producer: state-manager
created: 2026-06-16T00:00:00Z
---

# F5 Scoped Adversarial Trajectory — ARP Security Analyzer

Gate: 3 consecutive fresh-context passes with zero findings (any severity) over the full ARP
delta (develop HEAD on 2d2fadf: STORY-111..115 + D-074..D-078b remediation suite).

**Current status: 0/3 — re-run pending on 2d2fadf after D-078/D-078b code changes.**

---

## Pass Summary Table

| Pass | Date | Findings | Severity | Counter | Outcome |
|------|------|----------|----------|---------|---------|
| Pass 1 (on bcb1bd6) | 2026-06-15 | 0 + O-A obs | O-A LOW obs | 1/3 (voided) | CLEAN — but O-A observation; human adjudicated as FIX |
| Pass 2 (on bcb1bd6) | 2026-06-15 | 0 | — | 2/3 (voided) | CLEAN — streak voided by D-078/D-078b code change |

**Streak RESET to 0/3 after D-078 (PR #247) + D-078b (PR #248) merged to develop (2d2fadf).**

---

## Pre-Reset Passes (bcb1bd6 — NOW VOIDED)

### Pass 1 (2026-06-15, on bcb1bd6)

**Develop HEAD:** bcb1bd6 (PR #246 O-1 rename-revert; F4 wave-level adversarial gate SATISFIED)
**Adversary stance:** fresh-context, ARP delta scope

**Findings:** 0 formal findings.

**Observation O-A (LOW — detection-semantics seam):**
A malformed ARP frame that fails `extract_arp_frame`'s 4-part type/size guard should produce a
D11 malformed finding regardless of which decode path (strict/lax) routes it. The lax `None`
arm (lax.net==None, stop_err==Layer::Arp) silently dropped such frames without emitting D11.
This was a LOW detection-semantics seam — an adversarially-crafted lax-path ARP could avoid
D11 classification.

**Adjudication:** Human chose FIX (2026-06-15/16). D-078 issued.

**Counter:** 1/3 → VOIDED by code change.

---

### Pass 2 (2026-06-15, on bcb1bd6)

**Develop HEAD:** bcb1bd6
**Adversary stance:** fresh-context, ARP delta scope

**Findings:** 0

**Counter:** 2/3 → VOIDED by code change.

---

## D-078 / D-078b Remediation (Streak Reset)

### D-078 (PR #247, merge 92c1561)

**Root cause of O-A:** The lax `None` arm (lax.net==None, stop_err==Layer::Arp) did not inspect
the ARP frame — it simply passed the frame along without malformed-ness detection, silently
skipping the D11 path for lax-routed ARP frames with bad type or size fields.

**Spec correction note:** Initial fix hypothesis was "lax builds slice + extract None" — this
was mechanically IMPOSSIBLE (lax path cannot build an ARP slice if it stopped at Layer::Arp).
The actual mechanism is a None-arm raw peek. Spec was corrected twice before reaching the
correct mechanism description.

**Spec artifacts changed:**
- BC-2.16.009 v1.4 → v1.6 (lax-None path D11 coverage; actual raw-peek mechanism)
- BC-2.16.015 v1.3 → v1.5 (path-independence invariant; corrected mechanism)
- STORY-111 v1.4 → v1.6 (reflects corrected BC versions)
- STORY-112 v1.4 → v1.6 (reflects corrected BC versions)

**Code fix:** Lax `None` arm now bounds-checked-peeks the raw 8-byte ARP fixed header
(offset from lax.link Ethernet2). Bad hw_type/proto_type → D11 "Non-Ethernet/IPv4 ARP frame".
Valid-but-truncated or non-Ethernet → "truncated ARP frame" decode-error.

**Security review:** CLEAR. CWE-693 D11-evasion pathway closed. Bounds-safe (no panic).

---

### D-078b (PR #248, merge 2d2fadf)

**Rationale:** Completion / defensive path-independence. The sibling lax `Some(LaxNetSlice::Arp)`
arm was found to also route `extract_arp_frame` returning `None` through a non-D11 path.

**Structural note:** The `Some(LaxNetSlice::Arp)` arm is structurally unreachable via
integration — etherparse raises `SliceError::Len` (which populates `lax.net = None`) BEFORE
it can populate `lax.net = Some(LaxNetSlice::Arp)`. The arm exists for exhaustiveness but
cannot be triggered in practice. Documented in `tests/bc_2_16_d078b_lax_some_arm_tests.rs`.

**Additional:** Decoder.rs doc-comment correctness sweep (3 loci corrected).

**Streak reset:** D-078 + D-078b constitute code changes to the ARP detection path post-F5
P1/P2 CLEAN runs. Per BC-5.39.001 (code change after clean pass resets streak), F5 counter
is reset to 0/3.

---

## Process Gap Recorded

**PG-ARP-FIX-MECHANISM-FIRST (OPEN — Cycle-Closing Checklist candidate):**

When adjudicating a fix, the spec was written from an INCORRECT mechanism hypothesis
("lax builds slice + extract None" is impossible) before the code mechanism was verified.
This caused:
1. Two rounds of spec+story correction (BC v1.4→v1.6) as the correct mechanism was discovered.
2. A sibling seam (D-078b) discovered only at PR review — not in the original fix burst.

Lesson: verify the ACTUAL code/library mechanism (e.g., via a quick probe or code read) BEFORE
writing fix specs. When fixing one arm of a branch, sweep ALL sibling arms in the same burst.

Recorded OPEN. To be added to the factory Cycle-Closing Checklist.

---

## F-1 — VLAN-Offset Regression (Second Streak Reset)

### F5 Pass 1/3 Re-Run (2026-06-16, on 2d2fadf)

**Develop HEAD:** 2d2fadf (PR #248 D-078b)
**Adversary stance:** fresh-context, ARP delta scope (scoped to STORY-111..115 + D-077/D-078/D-078b paths)

**Findings:** 1 MEDIUM finding — **F-1: VLAN-offset false-positive D11**

**F-1 [MEDIUM] — lax None-arm arp_offset hard-codes Ethernet2 baseline (offset 14), ignoring `lax.link_exts`:**

The D-078 fix raw-peeks the ARP fixed header at byte offset 14 (standard Ethernet2 frame
header = 14 bytes). For VLAN-tagged ARP (802.1Q), the frame has an additional 4-byte tag
between the Ethernet header and the payload — so the ARP fixed header starts at offset 18,
not 14. The fix read the 802.1Q TCI bytes (bytes 14-15) as the ARP hw_type field, which
contains the VLAN priority + CFI + VLAN ID — values that never match ETHERNET (0x0001) —
causing a false-positive D11 "Non-Ethernet/IPv4 ARP frame" for every VLAN-tagged ARP frame.

**Root cause:** D-078 fix hand-rolled offset/parsing (raw byte peek) without accounting for
the library's link extension model (`lax.link_exts`). The correct offset is
`14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum()`, which correctly handles:
- Standard Ethernet2: offset 14 (no extensions)
- Single VLAN (802.1Q): offset 18 (+4 bytes)
- QinQ (802.1Q + 802.1ad): offset 22 (+8 bytes)
- MACsec: offset varies per ext header_len()

**Security review:** CLEAN. No security regression (false-positive D11 is a detection-quality
defect, not a security evasion).

**Adjudication:** FIX.

**Fix:**
- `arp_offset = 14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum::<usize>()`
- BC-2.16.015 v1.5 → v1.6 (VLAN-offset clause added to lax-None path invariant)
- BC-2.16.009 v1.6 → v1.7 (VLAN-offset computation documented in lax-None arm EC)
- New tests: `tests/bc_2_16_d078_vlan_offset_tests.rs` (4 tests: standard/single-VLAN/QinQ/MACsec)

**PR:** #249 (merge 079013d)

**Counter:** F5 streak reset AGAIN to 0/3 by code change.

---

### Meta-Lesson (D-F1)

This is the third streak reset in F5, and the second caused by a fix-induced regression:
- O-A (LOW) found on bcb1bd6 → FIX → D-078 + D-078b (2 PRs, streak reset #1)
- D-078 fix introduced F-1 MEDIUM → FIX → PR #249 (1 more PR, streak reset #2)
- Total: 1 LOW O-A observation → 3 PRs + 1 MEDIUM regression

**Lesson for PG-ARP-FIX-MECHANISM-FIRST:** A fix that hand-rolls offset/parsing logic (vs
delegating to the library's abstraction) MUST be stress-tested against the library's full
input model BEFORE shipping. In this case, `lax.link_exts` is the correct abstraction;
raw peeking at offset 14 without it is library-model-ignorant.

**Meta-lesson for adjudication:** LOW-severity observations should be evaluated against
fix-induced-regression risk before adjudicating FIX. When a fix requires hand-rolling
logic that mirrors what a library already abstracts, the risk of offset/size drift is high.
Documentation of the risk (vs fix) is a valid alternative adjudication path for LOW findings.

---

## Current Status

**arp_f5_scoped_adversary_convergence_counter: 0/3 (re-run in progress on 079013d after F-1/VLAN fix)**

Next action: F5 scoped-adversarial re-run on develop HEAD 079013d.
- Counter starts at 0/3.
- Scope: full ARP delta (STORY-111..115) + D-077 type-reject path + D-078/D-078b lax-arm D11
  paths + F-1 VLAN-offset fix + all 16 SS-16 BCs (BC-2.16.001..015 v1.7/v1.6 current).
- Pass file: append to this document as "Pass 1/3 (079013d restart)" etc.

Trajectory shorthand (full history):
`P1-CLEAN(bcb1bd6;O-A-obs)→P2-CLEAN(bcb1bd6)→[D-078+D-078b RESET]→F-1-MEDIUM(2d2fadf)→[F-1-fix RESET]→0/3-pending(079013d)`
