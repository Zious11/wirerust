# STORY-124 Demo Evidence Report

**Story:** STORY-124 — IDB Parse (Link Type + if_tsresol), Interface Whitelist, and Multi-IDB Agreement  
**Epic:** E-19  
**Wave:** 52  
**Recorded:** 2026-06-20  
**Toolchain:** VHS v0.11.0 (terminal recordings), Python 3 (pcapng fixture generation)

---

## Coverage Summary

| Recording | AC(s) Covered | BC(s) | E-INP Code | Path |
|-----------|--------------|-------|------------|------|
| `AC-008-multi-idb-conflict.gif/.webm` | AC-008 | BC-2.01.018 PC2 | E-INP-011 | Error |
| `AC-009-multi-idb-same-type.gif/.webm` | AC-009 | BC-2.01.018 PC1 | None (success) | Success |
| `AC-007-non-whitelisted-idb.gif/.webm` | AC-007 | BC-2.01.016 PC2 | E-INP-001 | Error |
| `AC-001-be-idb-options.gif/.webm` | AC-001, AC-002 | BC-2.01.011 PC1, PC2 | None (success) | Success |
| `AC-TEST-idb-test-suite.gif/.webm` | AC-001..AC-011, VP-030 | All 3 BCs | All E-INP codes | Both |

---

## Recordings

### AC-008: Multi-IDB Link-Type Conflict → E-INP-011
**Files:** `AC-008-multi-idb-conflict.gif`, `AC-008-multi-idb-conflict.webm`  
**Tape:** `AC-008-multi-idb-conflict.tape`  
**Fixture:** `fixtures/multi_idb_conflict.pcapng`

Demonstrates the headline UX improvement: a pcapng with two IDBs carrying *different* whitelisted
link types (interface 0 = ETHERNET, interface 1 = LINUX_SLL) triggers E-INP-011 with the
`tcpdump -i any` / `single link type per file` remediation hint.

**Expected output excerpt:**
```
Error: Failed to read .../multi_idb_conflict.pcapng

Caused by:
    pcapng multi-interface link-type conflict: interface 0 has ETHERNET, interface 1 has LINUX_SLL
    (hint: this commonly occurs with 'tcpdump -i any' captures that mix link types;
     wirerust requires a single link type per file) (E-INP-011)
```

**Traces to:** BC-2.01.018 PC2, AC-008, EC-005

---

### AC-009: Multi-IDB Same Link Type → Accepted
**Files:** `AC-009-multi-idb-same-type.gif`, `AC-009-multi-idb-same-type.webm`  
**Tape:** `AC-009-multi-idb-same-type.tape`  
**Fixture:** `fixtures/multi_idb_same_type.pcapng`

Demonstrates the agreement-satisfied path: two IDBs both carrying ETHERNET link type produce no
error; `PcapSource.datalink = ETHERNET` and analysis proceeds normally (WIRERUST TRIAGE REPORT
displayed with 0 packets since the fixture has no EPBs).

**Traces to:** BC-2.01.018 PC1, AC-009, EC-004

---

### AC-007: Non-Whitelisted IDB Link Type → E-INP-001
**Files:** `AC-007-non-whitelisted-idb.gif`, `AC-007-non-whitelisted-idb.webm`  
**Tape:** `AC-007-non-whitelisted-idb.tape`  
**Fixture:** `fixtures/non_whitelisted_idb.pcapng`

Demonstrates graceful rejection of a pcapng IDB with link type IEEE802_11 (code 105). No panic;
error lists all 5 supported types: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228),
IPv6 (229). Whitelist check fires at Decision 17 check #2.

**Expected output excerpt:**
```
Error: Failed to read .../non_whitelisted_idb.pcapng

Caused by:
    Unsupported pcap link type: IEEE802_11. Supported: Ethernet (1), Raw IP (101),
    Linux Cooked (113), IPv4 (228), IPv6 (229)
```

**Traces to:** BC-2.01.016 PC2, AC-007, EC-006

---

### AC-001/AC-002: Big-Endian pcapng with IDB Options → Accepted
**Files:** `AC-001-be-idb-options.gif`, `AC-001-be-idb-options.webm`  
**Tape:** `AC-001-be-idb-options.tape`  
**Fixture:** `fixtures/big_endian_with_idb_options.pcapng`

Demonstrates that a genuine big-endian pcapng (SHB BOM = `1A 2B 3C 4D`) with a BE IDB
(ETHERNET linktype encoded as `00 01` — non-palindromic, a LE-reader would misread as 0x0100=256)
is correctly parsed. The IDB also carries an `if_tsresol` option (code=9, nanoseconds) in BE
encoding, which is correctly extracted. Proves the BE-options fix converged in adversarial review.

**Traces to:** BC-2.01.011 PC1 (linktype), PC2 (if_tsresol), Invariant 4 (endianness from SHB BOM)

---

### AC-TEST: Full IDB Test Suite (27 Tests, All ACs)
**Files:** `AC-TEST-idb-test-suite.gif`, `AC-TEST-idb-test-suite.webm`  
**Tape:** `AC-TEST-idb-test-suite.tape`

Runs `cargo test --test bc_2_01_story124_idb_tests` showing all 27 IDB tests pass:

- AC-001: linktype extraction, BE endianness correction
- AC-002: `if_tsresol` extraction and default (6)
- AC-003: body truncated → E-INP-008, nonzero reserved → E-INP-008
- AC-004: options TLV bounds-check, `if_tsresol` wrong length → E-INP-008 (F-M5)
- AC-005: Late IDB after packet → E-INP-013
- AC-006: Three-level precedence (E-INP-013 wins over E-INP-011)
- AC-007: Whitelist enforcement (5 whitelisted pass; IEEE802_11 → E-INP-001)
- AC-008: Multi-IDB conflict → E-INP-011 (2-IDB and 3-IDB variants)
- AC-009: Same-linktype multi-IDB → accepted
- AC-010: No-panic fuzz over arbitrary IDB bytes
- AC-011: VP-030 proptest (whitelisted DataLink values, agreement totality)

**Result:** `test result: ok. 27 passed; 0 failed; 0 ignored`

**Traces to:** All ACs (AC-001 through AC-011), all three BCs (BC-2.01.011, BC-2.01.016, BC-2.01.018), VP-030

---

## Fixtures

All fixtures are in `fixtures/` and generated by `/tmp/gen_pcapng_fixtures.py`
(pure Python 3, no third-party deps, inline pcapng byte construction mirroring the test helpers
in `tests/bc_2_01_story124_idb_tests.rs`).

| Fixture | Contents | Used By |
|---------|----------|---------|
| `multi_idb_conflict.pcapng` | LE SHB + IDB(ETHERNET) + IDB(LINUX_SLL) | AC-008 |
| `multi_idb_same_type.pcapng` | LE SHB + IDB(ETHERNET) + IDB(ETHERNET) | AC-009 |
| `non_whitelisted_idb.pcapng` | LE SHB + IDB(IEEE802_11) | AC-007 |
| `big_endian_with_idb_options.pcapng` | BE SHB + BE IDB(ETHERNET) + if_tsresol=9 | AC-001/002 |
| `three_idb_third_conflict.pcapng` | LE SHB + IDB(ETH) + IDB(ETH) + IDB(RAW) | (supplemental) |

---

## ACs Not Visually Demo-able via CLI

The following ACs are tested exclusively via Rust unit tests and cannot be demonstrated through
the `wirerust analyze` CLI (this is by design — they exercise internal parsing state):

| AC | Reason | Evidence |
|----|--------|---------|
| AC-003 | Body truncation / reserved-field errors surface as generic I/O errors at CLI layer; E-INP-008 is the Rust error message, not a formatted CLI output | `AC-TEST-idb-test-suite.gif` shows 27/27 passing |
| AC-004 | Options TLV bounds-check is a pure-core helper path (`parse_idb_options`); no CLI fixture exposes this path without custom byte manipulation | Same |
| AC-005 | Late IDB detection requires SHB+IDB+EPB+IDB sequence; the fixture would parse and produce E-INP-013 but the error message format is identical to the CLI error shown in AC-007 | Same |
| AC-006 | Three-level precedence is verified by the test `test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict`; the precedence ordering is a code-path property, not a visible output distinction | Same |
| AC-010 | No-panic fuzz (proptest over random bytes) cannot be meaningfully animated | Same |
| AC-011 | VP-030 proptest over whitelisted DataLink enums is a code-path property | Same |

All AC-TEST recordings show these pass in the test suite.
