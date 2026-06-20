---
document_type: session-checkpoints
cycle: feature-pcapng-reader
phase: F2
---

# Session Checkpoints Archive — feature-pcapng-reader

Archived session resume checkpoints (superseded). Latest checkpoint is in STATE.md.

---

## Archived: F2 PASS-9 CLEAN (0H/0C) — CLEAN-PASS 2/3 / PASS-9 REMEDIATED (D-163) / PASS-10 PENDING / F3 BLOCKED

**Archived when:** Pass-10 CLEAN (D-164) — F2 ADVERSARIAL CONVERGENCE achieved (clean-pass 3/3).

### PIPELINE STATUS: FEATURE MODE — F2 PASS-9 CLEAN (0C/0H/1M/3L) — CLEAN-PASS 2/3 (BC-5.39.001); PASS-9 FINDINGS REMEDIATED (MEDIUM-1 E-INP-009 PARAMETERIZED EPB+SPB; LOW-1/2/3 FIXED; D-163); ERROR-TAXONOMY v3.6; BC-2.01.012 v1.8 (PC6a/PC6b ANCHORS); BC-2.01.013 v1.9; HS-104 v1.5 (CASE E DEFENSE-IN-DEPTH); BC-INDEX v1.67; TRAJECTORY 23/24/17/13/13/13/12/8/4; PASS-10 PENDING (TARGETING CLEAN-PASS 3/3 → CONVERGENCE); F3 BLOCKED UNTIL PASS-10 CLEAN

Active cycle: **feature-pcapng-reader**. F2 pass-8 CLEAN (D-161)/focused re-audit CLEAN (D-162): CLEAN-PASS 1/3. F2 pass-9 CLEAN (D-163): 0C/0H/1M/3L — CLEAN-PASS 2/3. MEDIUM-1 (error-taxonomy v3.5→v3.6: E-INP-009 parameterized EPB message "EPB references interface_id=<id> but interface table is empty — no IDB has been parsed" + SPB message "SPB encountered but interface table is empty — no IDB has been parsed"); LOW-1 (SPB E-INP-009 message mandated by BC-2.01.013 PC5/AC-001, now cited in taxonomy); LOW-2 (HS-104 v1.4→v1.5: Case E downgraded — btl=47 crate alignment rejection primary path E-INP-010; PC6b defense-in-depth / unreachable on non-4-aligned block); LOW-3 (BC-2.01.012 v1.7→v1.8: PC6a/PC6b anchor labels added; PC9 dedup note); BC-2.01.013 v1.8→v1.9 (LOW-1 sibling audit trail). Novelty LOW. Trajectory 23/24/17/13/13/13/12/8/4. STORY-128 + STORY-127 scoped for F3. No in-flight story worktrees. No open PRs. **BEHAVIORAL DECISIONS TO SURFACE AT F2 HUMAN GATE: Decision 15 (interleaved-IDB reject → E-INP-013); Decision 16 (per-SHB reset dead-spec deferred); Decision 17 (IDB-parse precedence order); Decision 19 (zero-packet notice gating — amended rev 8: emission from main.rs, canonical format); Decision 20 (uniform block error-code rule); Decision 21 (if_tsoffset out-of-scope); Decision 22 (canonical spb_data_available=body.len()-4).**
