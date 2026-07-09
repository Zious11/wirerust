# Wave-72 Integration Demo: DEMO-003 — Gates + Governance
# Evidences: Action-pin-gate (23 SHA-pinned refs, 0 violations) + ADR-0012 10-decision check
#            + VP-024 proof_file_hash + CLAUDE.md Two Hash Disciplines (STORY-161 governance)
#
# RECORDING NOTE: Transcript is primary evidence (see wave-71 VHS limitation note).
# This tape is retained as a runnable reference with <repo> placeholder.

Output DEMO-003-gates-governance.gif
Output DEMO-003-gates-governance.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set WaitTimeout 60s

# Setup: navigate to repo root
Hide
Type "cd <repo>"
Enter
Sleep 500ms
Show

Type "# Wave-72 Demo 3a — Action-pin-gate: 23 remote action refs validated"
Enter
Sleep 400ms

Type "python3 -c \"import re,glob; sha=re.compile(r'uses:.*@[0-9a-f]{40}'); wfs=glob.glob('.github/workflows/*.yml'); hits=[l.strip() for f in wfs for l in open(f) if sha.search(l)]; print(f'{len(hits)} SHA-pinned remote action refs validated')\""
Enter
Wait+Screen /validated/
Sleep 800ms

Type "# Demo 3b — ADR-0012: ten-decision check"
Enter
Sleep 300ms

Type "grep -c '^### Decision' docs/adr/0012-protocols-catalog-and-coverage-gaps.md"
Enter
Wait+Screen /10/
Sleep 500ms

Type "grep '^### Decision' docs/adr/0012-protocols-catalog-and-coverage-gaps.md | cat"
Enter
Wait+Screen /Decision 10/
Sleep 800ms

Type "# Demo 3c — VP-024 proof_file_hash (STORY-161 FU-F6-KANI-CLEANUP discharge)"
Enter
Sleep 300ms

Type "grep '^proof_file_hash:' .factory/specs/verification-properties/vp-024-arp-parse-safety.md"
Enter
Wait+Screen /proof_file_hash/
Sleep 800ms

Type "# Demo 3d — CLAUDE.md Two Hash Disciplines codification"
Enter
Sleep 300ms

Type "grep -A 8 'Two Hash Disciplines' CLAUDE.md | cat"
Enter
Wait+Screen /proof_file_hash/
Sleep 2s
