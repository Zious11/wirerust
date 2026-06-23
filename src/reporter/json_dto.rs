//! DTO wrapper for per-finding JSON serialization with MITRE ATT&CK enrichment.
//!
//! `FindingJsonDto` wraps a `&Finding` and adds a derived `mitre_attack` array
//! (BC-2.11.035). The `#[serde(flatten)]` on the inner reference preserves all
//! existing finding keys; `mitre_attack` is purely additive.

use serde::Serialize;

use crate::findings::Finding;
use crate::mitre;

/// One element in the per-finding `mitre_attack` JSON array (BC-2.11.035).
///
/// Fields:
/// - `id`: always serialized — the technique ID verbatim from `mitre_techniques`.
/// - `name`: omitted when `None` (unknown technique ID).
/// - `tactic_id`: omitted when `None` (unknown technique ID or unmapped tactic).
/// - `tactic_name`: omitted when `None` (unknown technique ID).
/// - `reference`: always serialized — synthesized as
///   `"https://attack.mitre.org/techniques/{id}/"` for every ID.
#[derive(Debug, Serialize)]
pub(crate) struct MitreAttackEntry {
    pub(crate) id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tactic_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tactic_name: Option<String>,
    pub(crate) reference: String,
}

/// Serde wrapper that adds the `mitre_attack` enrichment array to a `Finding`
/// during JSON serialization (BC-2.11.035).
///
/// The `#[serde(flatten)]` on `inner` preserves every existing `Finding` field
/// unchanged (additive, non-breaking per BC-2.11.035 invariant 5). The
/// `mitre_attack` field is omitted when the vec is empty (`skip_serializing_if`
/// mirrors the `mitre_techniques` predicate, per BC-2.11.035 postcondition 4).
#[derive(Debug, Serialize)]
pub(crate) struct FindingJsonDto<'a> {
    #[serde(flatten)]
    pub(crate) inner: &'a Finding,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) mitre_attack: Vec<MitreAttackEntry>,
}

impl<'a> From<&'a Finding> for FindingJsonDto<'a> {
    fn from(finding: &'a Finding) -> Self {
        let mitre_attack = finding
            .mitre_techniques
            .iter()
            .map(|id| {
                let (name, tactic_name, tactic_id) = match mitre::technique_info(id) {
                    Some((n, tactic)) => (
                        Some(n),
                        Some(tactic.to_string()),
                        mitre::technique_tactic_id(id),
                    ),
                    None => (None, None, None),
                };
                MitreAttackEntry {
                    id: id.clone(),
                    name,
                    tactic_id,
                    tactic_name,
                    reference: format!("https://attack.mitre.org/techniques/{id}/"),
                }
            })
            .collect();
        Self {
            inner: finding,
            mitre_attack,
        }
    }
}
