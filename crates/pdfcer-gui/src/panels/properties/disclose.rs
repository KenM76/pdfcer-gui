//! # `panels::properties::disclose` — what pdfcer last worked out, at a width
//! it fits in
//!
//! This is the Tool panel's Block C, moved intact. It renders
//! [`crate::app::actions::disclosure::last_edit_disclosure`] — the slot that
//! carries, among other things, **the text tools' refusal sentences**.
//!
//! ## ★★★ The problem this block exists to solve, restated because it did not
//! go away when the panel did
//!
//! `crate::text::textedit::refusal` writes three good sentences, is tested by
//! `every_refusal_says_something`, and its own module records the trap: they
//! were aimed at the **status bar**, and *"it shares the status row with
//! everything else and R128 forbids that row growing."* `Refusal::SpansRuns` is
//! **47 words**. It has never been readable.
//!
//! That is very likely the actual cause of the operator's *"no text editing or
//! adding text on the canvas"*: on a dense CAD sheet the first click of
//! `edit.text` lands where the operator *wants* text rather than where text
//! *is*, so `Refusal::NoRun` is the likely first outcome — and a decline nobody
//! can read teaches an operator that the feature does not exist.
//!
//! ## ★★★ Why Properties is a permitted home, and the status bar is not
//!
//! `SHELL_LAYOUT_PROPOSAL.md` §3 named this the hard blocker for the one-line
//! strip: a 28 pt row is a row, and *"moving the disclosure back into one puts
//! a 47-word sentence into the surface that was measured to be unable to hold
//! it."* That objection stands, and the strip does **not** carry it.
//!
//! Checked rather than assumed, because the operator asked for it to be:
//!
//! * **The status bar already carries this exact slot.**
//!   `crate::app::status::disclosure::edit_disclosure` reads the same
//!   `last_edit_disclosure` and draws it through `disclosure_line`. So the
//!   question is not *may the bar mention this* — it already does.
//! * **What the bar may not do is make it readable.** `disclosure_line`'s own
//!   doc lists the four rules that keep R128 closed, and two of them are
//!   `truncate()` rather than wrapping — *"wrapping is how a one-row bar
//!   becomes a two-row bar, which is the feedback loop with extra steps"* — and
//!   a **bounded** sub-region so a long sentence cannot push the navigation
//!   controls off the right of the bar. A 47-word sentence rendered under those
//!   rules is a hover, not a disclosure.
//! * ⇒ **So the decline slot in the status bar is not the answer**, and it is
//!   not being asked to become one. It keeps its elided line, unchanged. What
//!   moves here is the *readable* copy.
//!
//! **A dock panel's width is the dock's**, decided before the body draws, so
//! text wrapped inside it cannot drive a width and R128 does not apply — the
//! identical property the Tool panel relied on, and the one
//! `panels::dimension_groups` used to retire the Manage-groups window's growth
//! loop.
//!
//! ## ★★ And why THIS panel rather than any other
//!
//! There is a precedent in this very module and it is exact.
//! [`super::annotdelete`]'s call site records it: *"It is the one section here
//! whose subject is a control that lives somewhere else … and none of those can
//! hold a sentence. R9 sends a permanently-refused capability's explanation to
//! the surface that describes what is selected, and this is that surface."*
//!
//! A refusal is a fact about the last thing the operator tried to do to the
//! document in front of them. Properties is where this application already puts
//! facts of that shape.
//!
//! ## ★ FIRST in the panel, where Block C was LAST in the Tool panel
//!
//! The one thing that changed in the move, and it changed on a written rule.
//! `REVIEW_TRIAGE.md`: *"Every disclosure above the list, without exception …
//! a caveat below a list arrives after the operator has already drawn a
//! conclusion."* Block C was last because the Tool panel's other blocks were
//! about the *next* gesture and this is about the *last* one. In Properties
//! everything below is a description of what is selected, and a refusal read
//! after that description arrives too late to explain it.
//!
//! ## It renders NOTHING when there is nothing, heading included
//!
//! Not *"No notes."*, not an empty heading. R9, and also honesty: a heading
//! present on every frame trains an operator to stop reading the region under
//! it, which would waste the one surface a disclosure has.

use crate::app::state::OpenDoc;

/// The region this block publishes when it has something to say.
///
/// ★ Published only on the frames it draws, so its **absence** is the evidence
/// that there is no disclosure — which is the distinction a driven check about
/// a refusal is actually asking about.
pub const REGION: &str = "properties.disclosures"; // ui-text-exempt: trace region name, never displayed

/// Draw the disclosure block, and say whether it drew.
pub(super) fn section(ui: &mut egui::Ui, doc: &OpenDoc) -> bool {
    let Some(disclosure) = crate::app::actions::disclosure::last_edit_disclosure(doc.edit_epoch)
    else {
        return false;
    };
    if disclosure.notes.is_empty() {
        return false;
    }
    ui.label(crate::text::tool::disclosures_heading());
    // ★ `ui_rect_visible`, not `ui_rect`. This panel is one `ScrollArea`, and a
    // rect published for a scrolled-out region gets CLICKED by the harness —
    // `geometry::section`'s recorded precedent, and the whole reason the dock's
    // stream was widened on 2026-09-04.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    for note in &disclosure.notes {
        // ★ VERBATIM, and wrapped rather than elided. `ui-spec` §6's standing
        // rule and the whole reason this block is in a panel rather than in the
        // status row: a disclosure that has been shortened to fit is a
        // disclosure that has been edited by the program disclosing it.
        ui.label(egui::RichText::new(note).small());
    }
    ui.separator();
    true
}

#[cfg(test)]
mod tests {
    /// The region is the Properties panel's, not the retired Tool panel's.
    ///
    /// ★ Worth a test because the move is the sort that leaves a name behind:
    /// a check still sweeping `tool.disclosures` would find nothing and SKIP,
    /// and a SKIP is not red.
    #[test]
    fn the_region_moved_with_the_block() {
        assert_eq!(super::REGION, "properties.disclosures");
        assert!(!super::REGION.starts_with("tool."));
    }
}
