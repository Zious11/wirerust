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
