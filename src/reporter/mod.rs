//! Output reporters and the [`Reporter`] trait.
//!
//! Two concrete implementations: [`json::JsonReporter`] (machine-readable
//! emission for downstream tooling, no escaping) and
//! [`terminal::TerminalReporter`] (TTY-friendly table with ADR 0003
//! control-byte escaping).
//!
//! Each implementation receives the same three inputs — a [`Summary`], a
//! `&[Finding]`, and a `&[AnalysisSummary]` — and produces a `String`.
//! See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the
//! escaping-layer rationale and the security-bug regression it prevented.

pub mod json;
pub mod terminal;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::summary::Summary;

pub trait Reporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String;
}
