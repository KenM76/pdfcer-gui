//! # `app::reachout` — **does this document reach outside itself?**
//!
//! One question, asked once when a document opens, answered off-canvas.
//!
//! ## What this closes
//!
//! **Ken, 2026-08-30:** *"I think pdfcer added support for several button
//! features and protections for outgoing submits."*
//!
//! Half right, and the half that is right is this one. pdfcer cannot yet
//! **author** a button action — that is still in the engine's planned list, and
//! it is a policy decision rather than a missing verb. What it did ship is the
//! **detection** side, and this shell was not asking.
//!
//! ## ★★★ The engine's own account of why detection matters
//!
//! `Pass 133.0`, in their words:
//!
//! > `scan_javascript` exists to answer *"what would this document run in
//! > Acrobat/Reader?"* and it walked `/AA` **only** — but a widget's primary
//! > action lives in `/A`, so a push button that submits a form to a web server
//! > reported `js_network_actions=0` … **three surfaces, none disclosing it.**
//! >
//! > **The failure mode is what makes it urgent rather than merely incomplete:
//! > a check that under-reports reads as a clean bill of health**, because
//! > silence and safety are indistinguishable to the reader.
//!
//! They then fixed it *from the carrier set rather than from the symptom* — 17
//! carrier sites, 10 container types, 7 key names — including `/Next` chaining,
//! which makes a per-carrier scan **unsafe rather than incomplete**: a document
//! can put a benign `/GoTo` where a scanner looks and hang the `/SubmitForm` off
//! its `/Next`.
//!
//! ⇒ And this shell called none of it. The engine could tell an operator that
//! the drawing somebody just sent them will post data to a web server the
//! moment they press a button, and nothing on screen said so.
//!
//! ## What is disclosed, and what deliberately is not
//!
//! Three facts, and only when they are **true**:
//!
//! | fact | why it is worth a sentence |
//! |---|---|
//! | it can **submit data somewhere** | the operator's drawing is about to leave the building |
//! | it can **launch a program** | §12.6.4.5, and it is the one that is not about forms at all |
//! | it **runs a script when opened** | it has already run by the time they read this |
//!
//! **Not disclosed:** field-level calculate, format and validate scripts. A form
//! that computes a total is an ordinary form, and warning about it would train
//! the operator to dismiss the sentence that matters. `panels::forms` already
//! lists those for anybody who wants the inventory.
//!
//! ★ **`scan_truncated` is disclosed too**, and it is the subtle one: it means
//! the engine stopped walking. A truncated scan that reported *"nothing found"*
//! would be exactly the clean-bill-of-health failure their note names, so when
//! the walk gave up this says *"pdfcer could not finish checking"* rather than
//! implying an all-clear.
//!
//! ## ★★ Why it is a status line and not a dialog
//!
//! Because pdfcer **executes none of these**. NF4 is standing: actions are
//! recognised and round-tripped, never run. So nothing is about to happen, and
//! a modal that stopped the operator to say *"this document contains a submit
//! button"* would be alarm without a decision attached — the operator cannot
//! act on it at open time and the drawing is not doing anything.
//!
//! What they can do is *know*, before they hand the file on or press a button
//! in another viewer. That is a sentence, not a barrier.
//!
//! ★ Rule 4's shape exactly, one more time: **render normally, report
//! separately.** Nothing is drawn on the page and no button is marked.

use pdfcer_core::forms::FormJavaScript;

/// **What a document reaches for, reduced to what is worth saying.**
///
/// A struct rather than the engine's whole `FormJavaScript`, because this
/// shell's question is narrower than the engine's: it asks *"does anything here
/// leave the document?"*, and eleven of that type's sixteen fields answer a
/// different question.
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
    /// ★★ Disclosed, never swallowed. A truncated scan reporting *"nothing
    /// found"* is the clean-bill-of-health failure the engine's own note calls
    /// the urgent one — silence and safety are indistinguishable to a reader.
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
    /// ★ The overwhelmingly common answer is `false`, and that is the point: a
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

    /// ★★★ An ordinary form says NOTHING.
    ///
    /// The single most important property here. A form that computes a total
    /// carries calculate and format scripts, and a shell that warned about
    /// those would put a sentence on the status row of every real form an
    /// operator opens — which trains them to ignore the one that says their
    /// drawing is about to be posted to a web server.
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
        // ★ Built by MUTATION rather than by struct literal, because
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
        // ★★ Truncation counts even with every counter at zero, and that is the
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
