//! # `panels::attachments::clip` — **copy, cut and paste an embedded file**
//!
//! The three controls that let an attachment move from one open document to
//! another. `tools/gates/check-verb-coverage.sh` asserts that this shell names
//! `copy_attachment`, `cut_attachment` and `paste_attachment`, because an
//! engine verb no surface reaches is a capability the operator does not have.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/attachments/clip.md`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::state::OpenDoc;
use crate::canvas::clipboard::{Clipped, read, store};
use crate::text::attachclip as t;

/// The Copy control's rectangle, for a driven check.
const REGION_COPY: &str = "attachments.copy"; // ui-text-exempt: a trace region name, never displayed
/// The Cut control's rectangle.
const REGION_CUT: &str = "attachments.cut"; // ui-text-exempt: a trace region name, never displayed
/// The Paste control's rectangle — declared only while there is something to
/// paste, which is itself the assertion a check makes about R9 here.
const REGION_PASTE: &str = "attachments.paste"; // ui-text-exempt: a trace region name, never displayed
/// The replacement warning's rectangle, drawn when a file of that name is here.
const REGION_REPLACES: &str = "attachments.paste.replaces"; // ui-text-exempt: a trace region name
/// The paste's "nothing will be displaced" state. See [`REGION_REPLACES`].
const REGION_FRESH: &str = "attachments.paste.fresh"; // ui-text-exempt: a trace region name

/// Draw Copy and Cut for one row.
pub(super) fn row_controls(
    ui: &mut Ui,
    doc: &OpenDoc,
    key: &[u8],
    name: &str,
    can_remove: bool,
    published: &mut super::Published,
    actions: &mut Vec<Action>,
) {
    let copy = ui.button(t::copy_button()).on_hover_text(t::copy_tooltip());
    if !published.copy {
        crate::diag::ui_rect_visible(REGION_COPY, copy.rect, ui.clip_rect());
        published.copy = true;
    }
    if copy.clicked() {
        take(ui.ctx(), doc, key, name);
    }

    if can_remove {
        let cut = ui.button(t::cut_button()).on_hover_text(t::cut_tooltip());
        if !published.cut {
            crate::diag::ui_rect_visible(REGION_CUT, cut.rect, ui.clip_rect());
            published.cut = true;
        }
        if cut.clicked() {
            // COPY FIRST, and only then raise the delete — the rule
            // `cut_objects` states, unchanged here. A cut whose copy half fails
            // is refused with nothing removed; reversed, it would take the
            // attachment away with nothing on the clipboard, which is the one
            // outcome the operator cannot recover from by pasting.
            if take(ui.ctx(), doc, key, name) {
                actions.push(Action::Attachment(AttachmentAction::Detach {
                    key: key.to_vec(),
                    name: name.to_owned(),
                }));
            }
        }
    }
}

/// Read one attachment and park it on the clipboard. `true` if it went.
fn take(ctx: &egui::Context, doc: &OpenDoc, key: &[u8], name: &str) -> bool {
    match doc.session.copy_attachment(key) {
        Ok(clip) => {
            put(ctx, clip);
            true
        }
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("attachment-copy-refused name={name:?} why={why}")
            });
            crate::app::actions::record_note(doc.edit_epoch, why.to_string());
            false
        }
    }
}

/// Draw the Paste control, or nothing.
pub(super) fn paste_control(ui: &mut Ui, existing: &[String], actions: &mut Vec<Action>) {
    let Some(Clipped::Attachment(clip)) = read(ui.ctx()) else {
        // R9: nothing on the clipboard renders NOTHING, not a greyed button.
        // Greying is for the temporarily unavailable and this is not that — an
        // operator with an empty clipboard is not waiting for anything.
        return;
    };

    // The question, asked BEFORE the button rather than after the press.
    // Drawn above it, so it is read on the way to the control rather than after
    // the eye has already moved past.
    let replacing = existing.iter().any(|n| n == &clip.name);
    if replacing {
        ui.label(
            egui::RichText::new(t::replaces_note(&clip.name))
                .small()
                .weak(),
        );
    }

    let paste = ui
        .button(t::paste_button())
        .on_hover_text(t::paste_tooltip(&clip.name));
    crate::diag::ui_rect_visible(REGION_PASTE, paste.rect, ui.clip_rect());
    // One of the two, every frame. See `REGION_REPLACES`: this is what makes
    // "no file will be displaced" a statement a change-log trace can carry.
    crate::diag::ui_rect_visible(
        if replacing {
            REGION_REPLACES
        } else {
            REGION_FRESH
        },
        paste.rect,
        ui.clip_rect(),
    );
    if paste.clicked() {
        actions.push(Action::Attachment(AttachmentAction::Paste {
            // The clip travels with the action rather than being re-read at
            // apply time, for `FormEdit::Recompute`'s reason: what the operator
            // consented to is what was on screen when they pressed, and an
            // action is a complete statement of an intent. Re-reading would
            // make "what did they agree to?" depend on when it is asked.
            clip: Box::new((*clip).clone()),
            // Carried so the outcome sentence can be the right one of two
            // without asking the document a second time, after the write has
            // already changed the answer.
            replacing,
        }));
    }
}

/// Put a clip on the clipboard. Called from the apply phase, where the session
/// is reachable.
pub(crate) fn put(ctx: &egui::Context, clip: pdfcer_core::attachments::AttachmentClip) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "attachment-copied name={:?} bytes={}",
            clip.name,
            clip.bytes.len()
        )
    });
    store(ctx, Clipped::Attachment(Box::new(clip)));
}

#[cfg(test)]
mod tests {
    /// The five regions are named apart, so a driven check aiming at one
    /// cannot match another by prefix.
    #[test]
    fn the_regions_are_named_apart() {
        let all = [
            super::REGION_COPY,
            super::REGION_CUT,
            super::REGION_PASTE,
            super::REGION_REPLACES,
            super::REGION_FRESH,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }
}
