//! Regression tests for GitHub issue #342 — DNP3 / ENIP analyzers leak per-flow
//! state because `StreamDispatcher::on_flow_close` stubs out the DNP3 and ENIP
//! forwarding arms instead of calling the analyzers' own `on_flow_close` / purge
//! logic.
//!
//! ## Bug summary (issue #342)
//!
//! Finding SEC-005 (ENIP): `StreamDispatcher::on_flow_close` — `Some(DispatchTarget::Enip)`
//! arm — executes `let _ = reason;` and returns without calling
//! `EnipAnalyzer::on_flow_close`.  `EnipAnalyzer::on_flow_close` exists and correctly
//! removes the per-flow entry from `self.flows`, but it is never called via the
//! dispatcher path, so `flows` grows without bound as flows are opened and closed.
//!
//! Finding SEC-006 (DNP3): `StreamDispatcher::on_flow_close` — `Some(DispatchTarget::Dnp3)`
//! arm — executes `let _ = reason;` and returns without purging per-flow state.
//! `Dnp3Analyzer` has no `on_flow_close` method at all; its `flows` HashMap is
//! never pruned on flow close, growing without bound.
//!
//! ## TDD role
//!
//! These tests are written FIRST (Red Gate).  They MUST FAIL on the current
//! (buggy) code and MUST PASS once the implementer:
//!   1. Wires `EnipAnalyzer::on_flow_close` into the dispatcher's ENIP arm.
//!   2. Adds `Dnp3Analyzer::on_flow_close` (purges the flow entry from `self.flows`)
//!      and wires it into the dispatcher's DNP3 arm.
//!
//! ## Test seam
//!
//! Both `EnipAnalyzer::flows` and `Dnp3Analyzer::flows` are declared `pub`.
//! No additional test-only accessor is required — `analyzer.flows.len()` is the
//! observable.  This mirrors the existing ENIP unit-test convention used in
//! `tests/enip_analyzer_tests.rs` (e.g. `analyzer.flows.contains_key(&key)` in
//! `test_flow_close_removes_state`).

#![allow(non_snake_case)]

use std::net::IpAddr;

use wirerust::analyzer::dnp3::Dnp3Analyzer;
use wirerust::analyzer::enip::EnipAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        src_port,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        dst_port,
    )
}

/// Minimal 24-byte EtherNet/IP frame: command = 0x0065 (RegisterSession), length = 0.
///
/// 24 bytes is the fixed ENIP encapsulation header size.  `length` field (bytes 2–3,
/// LE) = 0 means no payload, which is the smallest valid RegisterSession request.
/// `EnipAnalyzer::on_data` lazily creates a per-flow entry on the first call, so
/// even a minimal frame is sufficient to seed `self.flows`.
fn minimal_enip_frame() -> Vec<u8> {
    let mut frame = vec![0u8; 24];
    // command = 0x0065 (RegisterSession), LE
    frame[0] = 0x65;
    frame[1] = 0x00;
    // length = 0 (no payload), LE — bytes 2–3 already 0
    frame
}

/// Minimal DNP3 link-layer frame (10 bytes — header only, no user data).
///
/// Sync=0x05 0x64, LENGTH=0x05 (minimum), CONTROL=0x44 (DIR=0, PRM=1, FC=4
/// UNCONFIRMED_USER_DATA), DEST=0x0003 (LE), SRC=0x0001 (LE), CRC=0x9A 0xC5.
///
/// `Dnp3Analyzer::on_data` creates a per-flow entry via `flows.entry(…).or_default()`
/// on every call, so even a minimal (parse-failing) frame seeds `self.flows`.
fn minimal_dnp3_frame() -> Vec<u8> {
    vec![
        0x05, 0x64, // sync
        0x05, // LENGTH = 5
        0x44, // CONTROL
        0x03, 0x00, // DEST = 3, LE
        0x01, 0x00, // SRC = 1, LE
        0x9A, 0xC5, // CRC (pre-computed for these header bytes)
    ]
}

// ---------------------------------------------------------------------------
// SEC-005 (ENIP): dispatcher must forward on_flow_close to EnipAnalyzer
// ---------------------------------------------------------------------------

/// Regression test for SEC-005 / issue #342.
///
/// After opening an ENIP flow (feeding one ENIP frame through the dispatcher
/// on port 44818) and then closing it via `dispatcher.on_flow_close`, the
/// `EnipAnalyzer`'s live per-flow entry count must drop to 0.
///
/// FAILS on current code because `StreamDispatcher::on_flow_close` stubs the
/// `DispatchTarget::Enip` arm (`let _ = reason; // no forwarding needed`),
/// so `EnipAnalyzer::on_flow_close` is never called and `self.flows` retains
/// the entry forever.
///
/// Traces: SEC-005, issue #342, BC-2.17.019 §P3 (implicit: flow-close forwarding
/// must mirror the on_data forwarding path).
#[test]
fn test_SEC_005_enip_flow_state_purged_on_dispatcher_flow_close() {
    let enip = EnipAnalyzer::new(50, 5);
    let mut dispatcher = StreamDispatcher::new(None, None, None, None, Some(enip), None);

    let fk = flow_key(60001, 44818); // port 44818 → DispatchTarget::Enip

    // Feed one ENIP frame to create per-flow state in EnipAnalyzer::flows.
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &minimal_enip_frame(),
        0,
        1_700_000_000,
    );

    // Pre-condition: the flow entry was created.
    {
        let enip = dispatcher
            .enip_analyzer()
            .expect("ENIP analyzer configured");
        assert_eq!(
            enip.flows.len(),
            1,
            "SEC-005 pre-condition: EnipAnalyzer must have 1 per-flow entry after on_data"
        );
    }

    // Signal flow close through the dispatcher — this is the path under test.
    // On current (buggy) code this is a no-op for the Enip arm; the implementer
    // must wire it to EnipAnalyzer::on_flow_close.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);

    // Post-condition: EnipAnalyzer must have 0 live per-flow entries.
    // FAILS TODAY: flows.len() == 1 because on_flow_close was never forwarded.
    let enip = dispatcher
        .enip_analyzer()
        .expect("ENIP analyzer configured");
    assert_eq!(
        enip.flows.len(),
        0,
        "SEC-005 / issue #342: EnipAnalyzer::flows must be empty after dispatcher \
         on_flow_close — per-flow state was leaked (dispatcher never called \
         EnipAnalyzer::on_flow_close)"
    );
}

/// Bounded-retention invariant for ENIP (SEC-005 / issue #342).
///
/// Open and close N distinct ENIP flows sequentially through the dispatcher.
/// After all flows are closed, the live per-flow entry count must be 0 (not N).
///
/// FAILS on current code because the dispatcher never forwards close events to
/// EnipAnalyzer, so `flows` accumulates one entry per open flow and is never pruned.
///
/// N = 50: small enough to run quickly, large enough to prove unbounded growth.
#[test]
fn test_SEC_005_enip_flow_state_bounded_retention_after_n_flows() {
    const N: u16 = 50;

    let enip = EnipAnalyzer::new(50, 5);
    let mut dispatcher = StreamDispatcher::new(None, None, None, None, Some(enip), None);

    for i in 0..N {
        let fk = flow_key(50000 + i, 44818);
        dispatcher.on_data(
            &fk,
            Direction::ClientToServer,
            &minimal_enip_frame(),
            0,
            1_700_000_000,
        );
        dispatcher.on_flow_close(&fk, CloseReason::Fin);
    }

    // Post-condition: retained flow-state count must be 0, not N.
    // FAILS TODAY: flows.len() == 50 (each close was a no-op).
    let enip = dispatcher
        .enip_analyzer()
        .expect("ENIP analyzer configured");
    assert_eq!(
        enip.flows.len(),
        0,
        "SEC-005 / issue #342: after opening+closing {N} ENIP flows, \
         EnipAnalyzer::flows must contain 0 live entries (bounded retention). \
         Non-zero count means per-flow state is leaking — dispatcher never \
         forwarded on_flow_close to the analyzer."
    );
}

// ---------------------------------------------------------------------------
// SEC-006 (DNP3): dispatcher must purge Dnp3Analyzer per-flow state on close
// ---------------------------------------------------------------------------

/// Regression test for SEC-006 / issue #342.
///
/// After opening a DNP3 flow (feeding one DNP3 frame through the dispatcher
/// on port 20000) and then closing it via `dispatcher.on_flow_close`, the
/// `Dnp3Analyzer`'s live per-flow entry count must drop to 0.
///
/// FAILS on current code because `StreamDispatcher::on_flow_close` stubs the
/// `DispatchTarget::Dnp3` arm (`let _ = reason; // no forwarding needed`), and
/// `Dnp3Analyzer` has NO `on_flow_close` method at all — so `self.flows` is
/// never pruned on flow close.
///
/// Traces: SEC-006, issue #342, BC-2.15.021 §P3 (implicit: flow-close forwarding
/// must mirror the on_data forwarding path, matching Modbus behaviour).
#[test]
fn test_SEC_006_dnp3_flow_state_purged_on_dispatcher_flow_close() {
    let dnp3 = Dnp3Analyzer::new(10);
    let mut dispatcher = StreamDispatcher::new(None, None, None, Some(dnp3), None, None);

    let fk = flow_key(60002, 20000); // port 20000 → DispatchTarget::Dnp3

    // Feed one DNP3 frame to create per-flow state in Dnp3Analyzer::flows.
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &minimal_dnp3_frame(),
        0,
        1_700_000_000,
    );

    // Pre-condition: the flow entry was created.
    {
        let dnp3 = dispatcher
            .dnp3_analyzer()
            .expect("DNP3 analyzer configured");
        assert_eq!(
            dnp3.flows.len(),
            1,
            "SEC-006 pre-condition: Dnp3Analyzer must have 1 per-flow entry after on_data"
        );
    }

    // Signal flow close through the dispatcher — this is the path under test.
    // On current (buggy) code this is a no-op for the Dnp3 arm; the implementer
    // must add Dnp3Analyzer::on_flow_close and wire it here.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);

    // Post-condition: Dnp3Analyzer must have 0 live per-flow entries.
    // FAILS TODAY: flows.len() == 1 because on_flow_close was never forwarded and
    // Dnp3Analyzer has no purge path at all.
    let dnp3 = dispatcher
        .dnp3_analyzer()
        .expect("DNP3 analyzer configured");
    assert_eq!(
        dnp3.flows.len(),
        0,
        "SEC-006 / issue #342: Dnp3Analyzer::flows must be empty after dispatcher \
         on_flow_close — per-flow state was leaked (dispatcher never purged the \
         DNP3 flow entry; Dnp3Analyzer has no on_flow_close method)"
    );
}

/// Bounded-retention invariant for DNP3 (SEC-006 / issue #342).
///
/// Open and close N distinct DNP3 flows sequentially through the dispatcher.
/// After all flows are closed, the live per-flow entry count must be 0 (not N).
///
/// FAILS on current code because the dispatcher never purges DNP3 flow state
/// and `Dnp3Analyzer` has no `on_flow_close` — so `flows` grows monotonically.
///
/// N = 50: matches the ENIP bounded-retention test for consistency.
#[test]
fn test_SEC_006_dnp3_flow_state_bounded_retention_after_n_flows() {
    const N: u16 = 50;

    let dnp3 = Dnp3Analyzer::new(10);
    let mut dispatcher = StreamDispatcher::new(None, None, None, Some(dnp3), None, None);

    for i in 0..N {
        let fk = flow_key(50000 + i, 20000);
        dispatcher.on_data(
            &fk,
            Direction::ClientToServer,
            &minimal_dnp3_frame(),
            0,
            1_700_000_000,
        );
        dispatcher.on_flow_close(&fk, CloseReason::Fin);
    }

    // Post-condition: retained flow-state count must be 0, not N.
    // FAILS TODAY: flows.len() == 50 (each close was a no-op; no purge path exists).
    let dnp3 = dispatcher
        .dnp3_analyzer()
        .expect("DNP3 analyzer configured");
    assert_eq!(
        dnp3.flows.len(),
        0,
        "SEC-006 / issue #342: after opening+closing {N} DNP3 flows, \
         Dnp3Analyzer::flows must contain 0 live entries (bounded retention). \
         Non-zero count means per-flow state is leaking — dispatcher never \
         purges DNP3 flow state and Dnp3Analyzer has no on_flow_close method."
    );
}
