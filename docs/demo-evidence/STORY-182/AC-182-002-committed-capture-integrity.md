# AC-182-002 — Committed Mandatory Capture: Presence, Size, and Integrity

**Claim:** `iec104-iti-diverse.pcap` (and ONLY that file) is committed directly to
`tests/fixtures/`, tracked by git, ≤ 100 KB, and matches the recorded sha256 in
`tests/fixtures/E2E-PCAPS.md`.

## Tracked-file check

Command:
```
git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap
```

Output:
```
tests/fixtures/iec104-iti-diverse.pcap
exit code: 0
```

`--error-unmatch` exits non-zero for untracked paths — exit 0 confirms the file is tracked.

## Size check

Command:
```
wc -c < tests/fixtures/iec104-iti-diverse.pcap
```

Output:
```
   13952
```

13952 bytes ≤ 102400 (100 KB) — size gate satisfied (`test "$(wc -c <...)" -le 102400 && echo SIZE_OK` → `SIZE_OK`).

## Integrity (sha256) check

Command:
```
shasum -a 256 tests/fixtures/iec104-iti-diverse.pcap
```

Output:
```
07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7  tests/fixtures/iec104-iti-diverse.pcap
```

Matches the value recorded in `tests/fixtures/E2E-PCAPS.md:358`
(`07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7`) exactly.
Verification command `test "$(shasum -a 256 ... | cut -d' ' -f1)" = "07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7" && echo HASH_MATCH` → `HASH_MATCH`.

## Non-committed files confirmed absent from tracked tree

`iec104.pcap`, `iec104-sq.pcapng` (Wireshark "not redistributed"), and
`iec104-iti-dissect.pcap` (F-009/D-524 — positive evidence of upstream-of-ITI origin) are
NOT present in `git ls-files tests/fixtures/` — only `iec104-iti-diverse.pcap` is the new
committed entry (all three live only in gitignored `tests/fixtures/local-samples/`, as shown
in AC-182-001's Environment A directory listing).

**Verdict: PASS** — file tracked, size within the hard 100 KB gate, sha256 integrity confirmed
against `E2E-PCAPS.md`.
