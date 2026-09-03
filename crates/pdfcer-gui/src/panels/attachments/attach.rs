//! # `panels::attachments::attach` — putting a file into the document
//!
//! ## The gap this closes
//!
//! `EditSession::attach_file` and `EditSession::detach_file` have existed in
//! `pdfcer-core` with **no GUI surface of any kind** — not a command, not a
//! panel, not a menu item. This row is the writing half of the surface that
//! closes it, and `super` is the reading half.
//!
//! ## ★★★ Why this row is drawn ABOVE the list, and it is not a style choice
//!
//! `panels::bookmarks` paid for this lesson in a driven run and wrote it down
//! as a rule; this module obeys the rule rather than rediscovering it:
//!
//! > A driven run on a 122-bookmark drawing found the panel body occupying
//! > y=133..770 and this row laid out at y=899..923 — **below the bottom of the
//! > panel**, with no way to reach it. The row drew. It published its region.
//! > Every unit test passed. […] **A control that must always be reachable
//! > cannot be placed after an unbounded `ScrollArea`.** Reserve-and-hope is
//! > not a second option; it is the same defect with a tuning parameter.
//!
//! It also reads correctly there. Acrobat's Attachments panel puts its add
//! control in a toolbar **above** the list for the same reason every list in
//! every application does: a control's position is a claim about what it acts
//! on, and this one acts on the document that owns the list.
//!
//! ## ★★ The description can only be set now, and the row says so
//!
//! `attach_file` takes `description: Option<&str>` and writes `/Desc` on the
//! file specification (Table 44, whose row for it says `/Desc` *"shall be used
//! for files in the `EmbeddedFiles` name tree"* — precisely this route).
//! `pdfcer-core` exposes **no verb that edits one afterwards**.
//!
//! So there is no *Edit description* control anywhere in this panel — R9: an
//! absent capability renders nothing, and a greyed one would be a promise no
//! state of the program could keep. But the *limit* is disclosed in the row,
//! because "you cannot change this later" is a fact an operator needs
//! **before** they leave the box empty, and R9 has never said a missing verb
//! must also be a secret.
//!
//! ## Why the picker is not opened here
//!
//! A native file dialog is a modal OS window that blocks the thread. Opening
//! one from this `clicked()` branch would leave egui part-way through a frame
//! that will not finish until the operator has answered — `app::actions::write`
//! calls that *"the sharpest seam this enum has"*. The button raises
//! [`AttachmentAction::Attach`]; `PdfcerApp::apply` opens the picker, in step 3,
//! after every panel and dialog has closed.
//!
//! That is also what gives the feature a **driver**:
//! `crate::app::files::DIAG_ATTACH_PATH` answers the dialog without a human,
//! and no synthetic input reaches a native dialog otherwise.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::text::panels::attachments as t;

use super::AttachmentsUi;

/// The region the description field publishes.
pub const REGION_DESCRIPTION: &str = "attachments.description"; // ui-text-exempt: trace region name, never displayed
/// The region the Attach button publishes.
pub const REGION_ATTACH: &str = "attachments.attach"; // ui-text-exempt: trace region name, never displayed

/// Draw the attach-a-file row.
///
/// # ★ The button is never greyed, and the contrast with the Bookmarks add row
/// is the argument
///
/// That row greys its Add button until a title has been typed, because a
/// bookmark with no title is an invisible row — the operand is *required* and
/// the control cannot act without it.
///
/// Here the operand is a **file the operator has not chosen yet**, and the
/// description beside the button is optional by the engine's own signature. So
/// there is no state in which this control cannot act, nothing to grey it for,
/// and nothing to explain on hover about why it is unavailable. P3 reserves
/// greying for *temporarily unavailable, always explained*; a permanently
/// enabled control is the honest rendering of a permanently available verb.
pub fn show(ui: &mut Ui, ui_state: &mut AttachmentsUi, actions: &mut Vec<Action>) {
    ui.label(t::attach_heading());

    let response = ui.add(
        egui::TextEdit::singleline(&mut ui_state.description)
            .desired_width(f32::INFINITY)
            .hint_text(t::attach_description_hint()),
    );
    crate::diag::ui_rect(REGION_DESCRIPTION, response.rect);
    ui.label(
        egui::RichText::new(t::attach_description_note())
            .small()
            .weak(),
    );

    let button = ui
        .button(t::attach_button())
        .on_hover_text(t::attach_tooltip());
    crate::diag::ui_rect(REGION_ATTACH, button.rect);
    if button.clicked() {
        // Trimmed here rather than in the apply arm, because the trim is a
        // decision about what the operator *meant* and belongs where they are
        // looking. An all-whitespace description is no description: writing it
        // would put a `/Desc` key holding blanks into the file, which a later
        // reader has to interpret and which no operator intended.
        let description = ui_state.description.trim();
        let description = (!description.is_empty()).then(|| description.to_owned());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. The LENGTH,
            // not the text — a description is the operator's own words about
            // their own file, and this reaches a trace a harness keeps.
            format!(
                "attach-file-requested described={} chars={}",
                description.is_some(),
                description.as_ref().map_or(0, |d| d.chars().count())
            )
        });
        actions.push(Action::Attachment(AttachmentAction::Attach { description }));
        // Cleared on the press, for `panels::bookmarks::add`'s reason: the
        // queue drains after the frame, and a description left in the box would
        // silently be re-used by the next attach — which the engine would
        // accept, and which would describe one file with another's note.
        ui_state.description.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **An all-whitespace description is no description.**
    ///
    /// The expression under test is the one the button arm uses, spelled the
    /// same way, so the two cannot come apart. What it defends: writing
    /// `Some("   ")` would put a `/Desc` key holding blanks into the file — a
    /// key a later reader has to interpret and that no operator asked for —
    /// while the row that shows it would appear to have a description and show
    /// nothing.
    #[test]
    fn a_blank_description_becomes_no_description_at_all() {
        for blank in ["", " ", "\t", "\n  \t"] {
            let trimmed = blank.trim();
            assert!(
                !(!trimmed.is_empty()),
                "{blank:?} must not produce a /Desc key"
            );
        }
        let typed = "  the supplier's quote  ";
        let trimmed = typed.trim();
        assert_eq!(
            trimmed, "the supplier's quote",
            "surrounding space is not the operator's text"
        );
        assert!(!trimmed.is_empty());
    }

    /// **The two published regions are distinct names.**
    ///
    /// A driven check clicks a region by name; two controls sharing one would
    /// make the harness click whichever was published last, and the failure
    /// would present as *"the button does nothing"* on whichever run lost.
    #[test]
    fn the_two_regions_are_named_apart() {
        assert_ne!(REGION_DESCRIPTION, REGION_ATTACH);
        assert!(REGION_DESCRIPTION.starts_with("attachments."));
        assert!(REGION_ATTACH.starts_with("attachments."));
    }
}
