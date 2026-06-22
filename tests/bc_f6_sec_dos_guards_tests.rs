//! F6 Security Hardening: DoS Guard Tests
//!
//! TDD test suite for F6-SEC-A (file-size gate, E-INP-014) and
//! F6-SEC-B (interface table cap, E-INP-015).
//!
//! BC coverage:
//!   BC-2.01.009 v1.8 PC3 + EC-011/EC-012  — file-size gate (F6-SEC-A)
//!   BC-2.01.011 v1.9 PC4 + EC-014         — interface cap (F6-SEC-B)
//!   BC-2.01.017 v1.7                       — error context
//!   ADR-009 rev 13 Decisions 27 + 28
//!
//! All tests start RED (before implementation) and must turn GREEN after
//! the implementation lands (TDD micro-commit protocol).
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` throughout.
//! `#![allow(non_snake_case)]` is required per the factory naming mandate.
#![allow(non_snake_case)]

use std::io::Cursor;

use wirerust::reader::PcapSource;

// ── Shared pcapng block-builder helpers (mirrors bc_2_01_story124_idb_tests.rs) ─

const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// Build a minimal LE SHB block (28 bytes).
fn le_shb() -> Vec<u8> {
    let mut v = Vec::with_capacity(28);
    v.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&28u32.to_le_bytes());
    v.extend_from_slice(&SHB_BOM_LE);
    v.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    v.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    v.extend_from_slice(&28u32.to_le_bytes()); // trailing btl
    assert_eq!(v.len(), 28);
    v
}

/// Build a minimal LE IDB block with the given linktype (no options).
///
/// linktype 1 = Ethernet (whitelisted by wirerust).
fn le_idb(linktype: u16) -> Vec<u8> {
    let btl: usize = 12 + 8; // outer(12) + fixed body(8) — no options
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // btl LE
    v.extend_from_slice(&linktype.to_le_bytes()); // linktype
    v.extend_from_slice(&0u16.to_le_bytes()); // reserved
    v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen (discarded)
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // trailing btl
    assert_eq!(v.len(), btl);
    v
}

// ─────────────────────────────────────────────────────────────────────────────
// F6-SEC-A — file-size gate (E-INP-014)
// BC-2.01.009 PC3 + EC-011/EC-012 / ADR-009 Decision 27
// ─────────────────────────────────────────────────────────────────────────────

/// Constants matching reader.rs (used to state boundaries clearly in tests).
///
/// 4 GiB limit per BC-2.01.009 EC-011 / ADR-009 Decision 27.
const MAX_PCAPNG_FILE_BYTES: u64 = 4_294_967_296;

/// F6-SEC-A positive: from_file on a sparse pcapng file of MAX+1 bytes must fail
/// with an error carrying "E-INP-014" and "too large".
///
/// BC-2.01.009 EC-011 — files exceeding 4 GiB are rejected before read_to_end.
/// Uses File::set_len (sparse extension) so the test is instantaneous and
/// consumes no real disk space beyond the sparse block.
#[test]
fn test_BC_2_01_009_file_size_gate_rejects_oversized_pcapng() {
    use std::fs::File;
    use std::io::Write;

    // Create a temp file containing a valid pcapng SHB (so magic probe succeeds)
    // then extend it past the 4 GiB limit using sparse allocation.
    let tmp = tempfile::NamedTempFile::new().expect("tempfile creation failed");
    let path = tmp.path().to_owned();

    // Write a valid LE SHB so the magic-byte probe identifies this as pcapng.
    {
        let mut f = File::create(&path).expect("create failed");
        f.write_all(&le_shb()).expect("write shb failed");
        // Sparse-extend to MAX+1 bytes (beyond the 4 GiB gate).
        f.set_len(MAX_PCAPNG_FILE_BYTES + 1)
            .expect("set_len failed (sparse extension)");
        f.flush().expect("flush failed");
    }

    let result = PcapSource::from_file(&path);
    assert!(result.is_err(), "expected Err for oversized pcapng, got Ok");
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.contains("E-INP-014"),
        "error must contain 'E-INP-014'; got: {msg}"
    );
    assert!(
        msg.contains("too large"),
        "error must contain 'too large'; got: {msg}"
    );
}

/// F6-SEC-A boundary: from_file on a pcapng file of exactly MAX bytes must succeed
/// (boundary is exclusive: only > MAX is rejected, not == MAX).
///
/// BC-2.01.009 EC-011 — limit is MAX bytes, inclusive (> MAX rejected).
///
/// NOTE: This test uses a sparse file of exactly MAX bytes. The SHB at the front
/// makes the magic probe pass; the rest is zero-filled by the OS. The pcapng
/// parser will fail on malformed trailing content but MUST NOT fail with E-INP-014.
/// We assert: no E-INP-014 in the error (or Ok if the parser happens to accept it).
#[test]
fn test_BC_2_01_009_file_size_gate_accepts_exactly_max_bytes() {
    use std::fs::File;
    use std::io::Write;

    let tmp = tempfile::NamedTempFile::new().expect("tempfile creation failed");
    let path = tmp.path().to_owned();

    {
        let mut f = File::create(&path).expect("create failed");
        f.write_all(&le_shb()).expect("write shb failed");
        // Sparse-extend to exactly MAX bytes — should NOT be rejected by the size gate.
        f.set_len(MAX_PCAPNG_FILE_BYTES)
            .expect("set_len failed (sparse extension)");
        f.flush().expect("flush failed");
    }

    let result = PcapSource::from_file(&path);
    // Either Ok (if the parser happens to accept the SHB-only file) or a non-014 error.
    if let Err(e) = result {
        let msg = format!("{:#}", e);
        assert!(
            !msg.contains("E-INP-014"),
            "file exactly at MAX ({MAX_PCAPNG_FILE_BYTES}) must NOT be rejected \
             with E-INP-014; got: {msg}"
        );
    }
    // Ok case: no assertion needed.
}

/// F6-SEC-A non-regression: a normal small pcapng file (SHB + 1 IDB) still parses OK.
///
/// BC-2.01.009 PC3 — the gate applies ONLY when size > MAX; small files are unaffected.
#[test]
fn test_BC_2_01_009_file_size_gate_normal_file_still_parses() {
    use std::fs::File;
    use std::io::Write;

    let tmp = tempfile::NamedTempFile::new().expect("tempfile creation failed");
    let path = tmp.path().to_owned();

    {
        let mut f = File::create(&path).expect("create failed");
        // SHB + 1 IDB (ETHERNET linktype) — a minimal valid pcapng capture.
        f.write_all(&le_shb()).expect("write shb failed");
        f.write_all(&le_idb(1)).expect("write idb failed"); // 1 = ETHERNET
        f.flush().expect("flush failed");
    }

    let result = PcapSource::from_file(&path);
    assert!(
        result.is_ok(),
        "small pcapng SHB+IDB must parse OK; got: {:?}",
        result.err()
    );
}

/// F6-SEC-A: from_pcap_reader (stream path) is NOT gated by file size — it has no
/// access to fs::metadata. Feeding a pcapng buffer via Cursor must succeed regardless.
///
/// BC-2.01.009 PC3 note — the gate applies only in the PATH-based from_file entry.
#[test]
fn test_BC_2_01_009_stream_path_not_gated_by_file_size() {
    // SHB + 1 IDB fed through Cursor (no file at all) — must parse OK.
    let mut buf = le_shb();
    buf.extend_from_slice(&le_idb(1)); // ETHERNET
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "stream path (Cursor) must not be gated by file size; got: {:?}",
        result.err()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F6-SEC-B — interface table cap (E-INP-015)
// BC-2.01.011 PC4 + EC-014 / ADR-009 Decision 28
// ─────────────────────────────────────────────────────────────────────────────

const MAX_INTERFACE_TABLE_ENTRIES: usize = 65_535;

/// Build a pcapng byte buffer containing one LE SHB + N identical LE IDBs.
///
/// linktype=1 (ETHERNET) is used for all IDBs (whitelisted; avoids multi-linktype error).
fn pcapng_with_n_idbs(n: usize) -> Vec<u8> {
    let idb = le_idb(1); // ETHERNET, no options
    let mut buf = le_shb();
    for _ in 0..n {
        buf.extend_from_slice(&idb);
    }
    buf
}

/// F6-SEC-B positive: 65536 IDBs (MAX+1) must fail with E-INP-015.
///
/// BC-2.01.011 PC4 — when interfaces.len() >= MAX_INTERFACE_TABLE_ENTRIES during
/// an IDB push, return E-INP-015 immediately.
///
/// Buffer size: SHB(28) + 65536 × IDB(20) = 28 + 1_310_720 ≈ 1.25 MB.
#[test]
fn test_BC_2_01_011_interface_cap_rejects_65536_idbs() {
    let buf = pcapng_with_n_idbs(MAX_INTERFACE_TABLE_ENTRIES + 1);
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_err(),
        "expected Err for 65536 IDBs, got Ok"
    );
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.contains("E-INP-015"),
        "error must contain 'E-INP-015'; got: {msg}"
    );
    assert!(
        msg.contains("65535"),
        "error must mention the limit 65535; got: {msg}"
    );
}

/// F6-SEC-B boundary: exactly 65535 IDBs (MAX) must be accepted.
///
/// BC-2.01.011 PC4 — the cap is > MAX, so MAX itself is valid.
#[test]
fn test_BC_2_01_011_interface_cap_accepts_exactly_65535_idbs() {
    let buf = pcapng_with_n_idbs(MAX_INTERFACE_TABLE_ENTRIES);
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "exactly 65535 IDBs must be accepted; got: {:?}",
        result.err()
    );
}

/// F6-SEC-B non-regression: a normal 1-IDB pcapng still parses OK.
///
/// BC-2.01.011 PC4 — the guard must not affect ordinary captures.
#[test]
fn test_BC_2_01_011_interface_cap_normal_file_unaffected() {
    let buf = pcapng_with_n_idbs(1);
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "single-IDB pcapng must parse OK; got: {:?}",
        result.err()
    );
}

/// F6-SEC-B exact error message: must carry the verbatim E-INP-015 taxonomy string.
///
/// BC-2.01.017 v1.7 — exact message: "pcapng interface table too large: exceeds
/// limit of 65535 interfaces (E-INP-015)".
#[test]
fn test_BC_2_01_011_interface_cap_exact_error_message() {
    let buf = pcapng_with_n_idbs(MAX_INTERFACE_TABLE_ENTRIES + 1);
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    let msg = format!("{:#}", result.expect_err("expected Err for 65536 IDBs"));
    // Exact taxonomy message (from error-taxonomy v3.8 / BC-2.01.017 v1.7).
    assert!(
        msg.contains("pcapng interface table too large"),
        "error must contain 'pcapng interface table too large'; got: {msg}"
    );
    assert!(
        msg.contains("exceeds limit of 65535 interfaces"),
        "error must contain 'exceeds limit of 65535 interfaces'; got: {msg}"
    );
    assert!(
        msg.contains("E-INP-015"),
        "error must contain 'E-INP-015'; got: {msg}"
    );
}

/// F6-SEC-A exact error message: must carry the verbatim E-INP-014 taxonomy string.
///
/// BC-2.01.017 v1.7 — exact message:
/// "pcapng file too large: {size} bytes exceeds limit of {limit} bytes (E-INP-014);
/// use a streaming tool or split the capture"
#[test]
fn test_BC_2_01_009_file_size_gate_exact_error_message() {
    use std::fs::File;
    use std::io::Write;

    let tmp = tempfile::NamedTempFile::new().expect("tempfile creation failed");
    let path = tmp.path().to_owned();

    {
        let mut f = File::create(&path).expect("create failed");
        f.write_all(&le_shb()).expect("write shb failed");
        f.set_len(MAX_PCAPNG_FILE_BYTES + 1)
            .expect("set_len failed");
        f.flush().expect("flush failed");
    }

    let result = PcapSource::from_file(&path);
    let msg = format!("{:#}", result.expect_err("expected Err for oversized pcapng"));

    // Exact framing from error-taxonomy v3.8 / BC-2.01.017 v1.7.
    assert!(
        msg.contains("pcapng file too large"),
        "error must start with 'pcapng file too large'; got: {msg}"
    );
    assert!(
        msg.contains("bytes exceeds limit of"),
        "error must contain 'bytes exceeds limit of'; got: {msg}"
    );
    assert!(
        msg.contains("bytes (E-INP-014)"),
        "error must contain 'bytes (E-INP-014)'; got: {msg}"
    );
    assert!(
        msg.contains("use a streaming tool or split the capture"),
        "error must contain 'use a streaming tool or split the capture'; got: {msg}"
    );
}
