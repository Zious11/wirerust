//! # wirerust
//!
//! Fast PCAP forensics and network triage CLI library. The binary is a thin
//! wrapper (`src/main.rs`) over the public API exposed here.
//!
//! ## Pipeline
//!
//! 1. **[`reader`]** parses pcap-format capture files into raw frames.
//! 2. **[`decoder`]** turns each frame into a [`decoder::DecodedFrame`] —
//!    either a [`decoder::DecodedFrame::Ip`] wrapping a [`decoder::ParsedPacket`]
//!    (link-layer → IP → TCP/UDP) or a [`decoder::DecodedFrame::Arp`] wrapping
//!    an [`decoder::ArpFrame`] (link-layer → ARP, for EtherType 0x0806 frames).
//!    No application-layer parsing is done here.
//! 3. **[`summary`]** accumulates capture-level totals (packets, bytes, hosts,
//!    services, protocols).
//! 4. **[`reassembly`]** drives TCP stream reassembly with overlap / out-of-window
//!    / segment-limit accounting; pushes contiguous data to a
//!    [`reassembly::handler::StreamHandler`].
//! 5. **[`dispatcher`]** classifies each TCP stream by content peek (TLS `0x16
//!    0x03` vs HTTP method prefix) and routes data to the matching protocol
//!    analyzer.
//! 6. **[`analyzer`]** (DNS / HTTP / TLS) emits per-flow [`findings::Finding`]s.
//! 7. **[`reporter`]** renders the aggregate [`summary::Summary`] +
//!    `Vec<Finding>` + per-analyzer summaries to either a terminal table or
//!    JSON ([`reporter::Reporter`] is the trait; ADR 0003 governs the
//!    raw-data-vs-display-layer escaping boundary).
//!
//! The [`mitre`] module is a zero-dependency static catalog of MITRE ATT&CK
//! technique IDs → tactic + name, consumed by the reporter when `--mitre`
//! grouping is enabled.
//!
//! ## Documentation convention (LESSON-P1.06 phased rollout)
//!
//! `#![warn(missing_docs)]` is enabled crate-wide as the long-term default.
//! Submodules that have not yet been fully audited carry a local
//! `#[allow(missing_docs)]` (or `#![allow(missing_docs)]` at the file head)
//! so the global gate does not break CI under `-D warnings`. As each module
//! is audited, its local allow attribute is removed. Track residual scope
//! in the technical-debt register; new public items added to an audited
//! module MUST carry a doc comment.
#![warn(missing_docs)]
// LESSON-P2.02: enforce inlined format-string arguments
// (`format!("{x}")` not `format!("{}", x)`) across the crate.
// Combined with CI's `-D warnings`, this turns any regression
// into a build failure.
#![warn(clippy::uninlined_format_args)]

#[allow(missing_docs)]
pub mod analyzer;
#[allow(missing_docs)]
pub mod cli;
#[allow(missing_docs)]
pub mod decoder;
#[allow(missing_docs)]
pub mod dispatcher;
#[allow(missing_docs)]
pub mod findings;
#[allow(missing_docs)]
pub mod mitre;
#[allow(missing_docs)]
pub mod reader;
#[allow(missing_docs)]
pub mod reassembly;
#[allow(missing_docs)]
pub mod reporter;
#[allow(missing_docs)]
pub mod summary;
