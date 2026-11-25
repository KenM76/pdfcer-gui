//! # `preflightfix` — a Preflight panel fix, on its way to the edit funnel
//!
//! The panel cannot edit (it is handed `&OpenDoc`), so a Fix click raises
//! [`PreflightFix`] as an action; the action handler runs it through the edit
//! funnel and parks the engine's report in [`FixNote`] for the panel to show.

use pdfcer_core::preflight::{Fixup, PreflightReport, Profile, ProfileRule};

/// One Fix or Fix all click.
#[derive(Debug, Clone, PartialEq)]
pub struct PreflightFix {
    /// The profile the report was run against.
    pub profile: Profile,
    /// The run whose findings the fix aims at. Findings name objects by
    /// index, so the panel raises a fix only for a report of the document as
    /// it stands (same serial and edit epoch).
    pub report: PreflightReport,
    /// `Some(rule id)` fixes one rule; `None` fixes every rule with a fix.
    pub rule: Option<String>,
}

/// The last fix's report, as `FixupReport::to_text` wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixNote {
    /// The document's edit epoch once the fix had landed. A run started at or
    /// after this epoch supersedes the note.
    pub epoch: u64,
    /// The engine's plain-text account of every step.
    pub text: String,
}

/// The fixups a rule's Fix button runs: its own, else the engine's
/// suggestion. Empty means the panel draws no Fix button for the rule.
#[must_use]
pub fn rule_fixups(rule: &ProfileRule) -> Vec<Fixup> {
    if rule.fixups.is_empty() {
        Fixup::suggested(&rule.check).into_iter().collect()
    } else {
        rule.fixups.clone()
    }
}
