//! # `app::reachout` — **does this document reach outside itself?**
//!
//! One question, asked once when a document opens, answered off-canvas.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/reachout.md`.

use pdfcer_core::forms::FormJavaScript;

/// **What a document reaches for, reduced to what is worth saying.**
///
/// A struct rather than the engine's whole `FormJavaScript`, because this
/// shell's question is narrower than the engine's: it asks *"does anything here
/// leave the document?"*, and most of that type's fields answer a different one
/// — how much of the document is click-activated, which triggers fired, how many
/// field-level hooks there are. `FormJavaScript` is `#[non_exhaustive]` and
/// grows; this projection is what keeps that growth from widening the
/// disclosure by accident.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReachOut {
    /// Actions that can send data somewhere — `/SubmitForm`, `/URI`, and the
    /// file-specification carriers the engine's reach table counts.
    pub network: usize,
    /// `/Launch` — actions that can start a program (§12.6.4.5).
    pub launch: usize,
    /// The document runs a script the moment it is opened.
    pub script_on_open: bool,
    /// The engine stopped walking before it finished.
    ///
    /// Disclosed, never swallowed. A count of zero from a walk that stopped
    /// early means *"nothing found so far"*, not *"nothing is there"*, and
    /// presenting the first as the second is the clean-bill-of-health failure —
    /// silence and safety are indistinguishable to a reader.
    pub truncated: bool,
}

impl ReachOut {
    /// Reduce the engine's inventory to the four facts this shell discloses.
    #[must_use]
    pub const fn of(scan: &FormJavaScript) -> Self {
        Self {
            network: scan.network_action_count,
            launch: scan.launch_action_count,
            script_on_open: scan.open_action_is_javascript,
            truncated: scan.scan_truncated,
        }
    }

    /// Whether there is anything at all to say.
    ///
    /// The overwhelmingly common answer is `false`, and that is the point: a
    /// disclosure that fires on every document is one nobody reads.
    #[must_use]
    pub const fn worth_saying(self) -> bool {
        self.network > 0 || self.launch > 0 || self.script_on_open || self.truncated
    }
}

/// **Scan a freshly opened document and return what it reaches for.**
///
/// # Cost, because this runs on every open
///
/// One graph walk, bounded by the engine's own `actions_scanned` ceiling — the
/// `scan_truncated` flag exists because that ceiling is real. It is the same
/// order of work as reading the outline, which this shell already does on open,
/// and it happens once rather than per frame.
///
/// ⇒ Measured rather than assumed is the standing rule here, and this one has
/// **not** been measured on the benchmark drawing. It is bounded by
/// construction and it runs once; if a 129,758-object sheet ever opens visibly
/// slower after this, the scan is the first thing to time.
#[must_use]
pub fn scan(session: &pdfcer_core::edit::EditSession) -> ReachOut {
    let view = session.view();
    let scan = pdfcer_core::forms::scan_javascript(&view);
    let out = ReachOut::of(&scan);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "reach-out network={} launch={} open_script={} truncated={} scanned={}",
            out.network, out.launch, out.script_on_open, out.truncated, scan.actions_scanned
        )
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ordinary form says NOTHING.
    #[test]
    fn field_level_scripts_alone_are_not_worth_saying() {
        let mut scan = FormJavaScript::default();
        scan.fields_with_calculate_script = 4;
        scan.fields_with_format_script = 9;
        scan.fields_with_validate_script = 2;
        scan.fields_with_keystroke_script = 1;
        scan.custom_scripts = 3;
        assert!(
            !ReachOut::of(&scan).worth_saying(),
            "an ordinary calculating form must produce no disclosure at all"
        );
    }

    /// Each of the four facts alone is enough to speak.
    #[test]
    fn every_reaching_fact_is_worth_saying_on_its_own() {
        // Built by MUTATION rather than by struct literal, because
        // `FormJavaScript` is `#[non_exhaustive]` — a field the engine adds
        // later must not break this test, which is exactly what that attribute
        // is for. `..Default::default()` does not help: the restriction is on
        // the literal, not on the fields named in it.
        let mut network = FormJavaScript::default();
        network.network_action_count = 1;
        let mut launch = FormJavaScript::default();
        launch.launch_action_count = 1;
        let mut on_open = FormJavaScript::default();
        on_open.open_action_is_javascript = true;
        // Truncation counts even with every counter at zero, and that is the
        // whole reason it is a field here. "Nothing found" from a walk that
        // stopped early is not an all-clear.
        let mut truncated = FormJavaScript::default();
        truncated.scan_truncated = true;

        for (name, scan) in [
            ("network", network),
            ("launch", launch),
            ("script on open", on_open),
            ("truncated", truncated),
        ] {
            assert!(
                ReachOut::of(&scan).worth_saying(),
                "{name} alone must produce a disclosure"
            );
        }
    }

    /// A clean document is silent.
    #[test]
    fn a_document_that_reaches_nowhere_says_nothing() {
        assert!(!ReachOut::default().worth_saying());
    }
}
