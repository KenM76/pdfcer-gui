//! # `panels::attachments::clip` — **copy, cut and paste an embedded file**
//!
//! Found by `tools/gates/check-verb-coverage.sh` on its first honest run,
//! 2026-09-01: `copy_attachment`, `cut_attachment` and `paste_attachment`
//! shipped in `pdfcer-core` `Pass 173.0` and this shell named **none** of them.
//! So an attachment could not be moved from one open document to another, which
//! is an odd thing to be missing now that pdfcer is multi-document — and nothing
//! anywhere had written a sentence about it either way.
//!
//! ## Why the controls are in this panel and not on the ribbon
//!
//! The same argument `panels::bookmarks::clip` makes, and it is stronger here.
//! Every other attachment verb is in this panel — Attach, Save a copy, Remove —
//! because an attachment is only ever *seen* here. It is not on a page, it has
//! no `/Rect`, and no canvas gesture can select one.
//!
//! ⇒ So `Ctrl+C` and `Ctrl+V` are **not** wired to it. Those chords belong to
//! the canvas, and a chord whose meaning depends on which panel has focus is
//! the kind of thing this project's standing rule about conventional
//! interactions forbids inventing.
//!
//! ## ★★★ The question that MUST be asked before the press
//!
//! **Does the destination already have a file of that name?**
//!
//! `attach_file` builds its name-tree patch with
//! `entries.retain(|(k, _)| k != &name_bytes)` before pushing the new entry —
//! so a same-named attachment is **replaced**. Not refused. Not suffixed. The
//! existing entry is dropped from the tree and the new one takes its key.
//!
//! The old bytes survive in the earlier revision until a full rewrite, so it is
//! recoverable — and **nothing on screen would say it had happened**. That is
//! the third of the engine's *"produces a document that looks right and is
//! not"* shapes, and it gets the bookmark paste's treatment: asked beside the
//! button, while the operator can still choose.
//!
//! ★ A **statement**, not a confirmation. One paste is one `EditSession`
//! command and therefore one `Ctrl+Z`, which satisfies `HANDOFF.md`'s *confirmed
//! or clearly undoable*. What it must not be is silent.
//!
//! ## ★★ What CANNOT be asked in advance, and is filed
//!
//! A document whose `/EmbeddedFiles` root holds `/Kids` rather than `/Names`
//! refuses the attach entirely (`AttachmentTreeUnsupported`), and rightly:
//! inserting into a multi-node tree means repairing every `/Limits` range up
//! the chain, and getting that subtly wrong stops the document's *existing*
//! attachments resolving.
//!
//! `AttachmentNotes` reports six conditions and the tree's shape is not among
//! them. So that refusal arrives **after** the press, in words, through the
//! ordinary decline path. Honest, and one press worse than R9 wants; reported
//! to the engine rather than worked around.
//!
//! ## Where the Paste control lives
//!
//! At the **top of the panel**, above the list, and drawn **only when the
//! clipboard holds an attachment** — R9's rule that an unavailable capability
//! renders nothing rather than a greyed stub. Above rather than below because
//! the list can be long and a control at the bottom of a scrolled list is one
//! the operator has to hunt for.

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
///
/// ★★★ **Paired with [`REGION_FRESH`], and the pairing is not decoration.**
///
/// `crate::diag::ui_rect` is a **change log**: a region that stops being drawn
/// does not un-declare itself, so a harness cannot learn "this warning is not
/// showing" from the absence of the name. That is written up in
/// `D:/dev/rag/egui/a_change_log_ui_rect_trace_cannot_report_that_a_widget_stopped_being_drawn.md`
/// and this feature reproduced it within the hour: a driven check asserted the
/// warning was absent in the second document, and read the declaration this
/// panel had legitimately made in the FIRST one — where a file of that name
/// really was present — several frames earlier.
///
/// ⇒ So the control declares **one of two names**, always exactly one, and a
/// reader takes whichever came last. An absence assertion becomes a presence
/// assertion, which a change log can answer.
const REGION_REPLACES: &str = "attachments.paste.replaces"; // ui-text-exempt: a trace region name
/// The paste's "nothing will be displaced" state. See [`REGION_REPLACES`].
///
/// ★ It has **no visible text** — there is nothing to say, and a line reading
/// *"this will not replace anything"* on every paste is the noise that trains
/// an operator to stop reading the one that matters. It publishes the button's
/// own rectangle under a second name, which costs a trace line and no pixels.
const REGION_FRESH: &str = "attachments.paste.fresh"; // ui-text-exempt: a trace region name

/// Draw Copy and Cut for one row.
///
/// `key` is the `/EmbeddedFiles` name-tree key — the thing the document is
/// addressed with. `name` is what the row displays. They are **different**, and
/// the distinction is `panels::attachments`' own: the key is bytes with no
/// declared encoding (§7.9.6) which producers mangle with numeric suffixes and
/// portfolio folder prefixes, so it is the right thing to address the document
/// with and the wrong thing to show a person.
///
/// ★ Cut is offered only where Remove is — a document-level attachment. A
/// page-level one is removed by deleting its note, and `detach_file` answers
/// `AttachmentNotFound` for it by name. Offering a Cut that could only refuse
/// would be an affordance for an act this code cannot perform.
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
            // ★★★ COPY FIRST, and only then raise the delete. `cut_objects`'
            // doc comment makes the argument and it applies unchanged: *"a
            // selection that cannot be copied is refused with nothing deleted.
            // Reversed, a cut whose copy half failed would take the objects
            // away with nothing on the clipboard — the one outcome the operator
            // cannot recover from by pasting."*
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
///
/// # ★★ Why this is here and not an `Action`, unlike almost everything else
///
/// `copy_attachment` is `&self` and commits nothing, and the panel already
/// holds `&OpenDoc`. Routing it through the queue would gain nothing and cost
/// the thing `canvas::clipboard::cut` is careful about: the copy has to be able
/// to **fail before the delete is raised**, and an action queued behind another
/// action cannot report back to the code that decides whether to queue the
/// second one.
///
/// ⇒ So Copy and Cut are widget-layer, exactly as `canvas::clipboard::copy` and
/// `canvas::fieldclip::copy` are, and only the delete crosses into the queue.
///
/// ★★★ **And that is why `EditSession::cut_attachment` is never called.** It
/// exists, it works, and it folds its two commands into one undo entry with a
/// private method — which a shell cannot reach. Here it does not matter: the
/// copy half commits nothing, so copy-then-`Detach` is already **one** command
/// and therefore one `Ctrl+Z`. The same argument `canvas::clipboard::cut`
/// records for `cut_objects`. Recorded in `EDITABLE_SURFACES.md`.
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
///
/// `existing` is every name already listed in **this** document, which is what
/// the replacement question is answered from. Passed in rather than re-derived
/// so the panel's listing and this control cannot disagree about what is here —
/// the same rule `rows::block_reason` states for the fillable/selectable split.
pub(super) fn paste_control(ui: &mut Ui, existing: &[String], actions: &mut Vec<Action>) {
    let Some(Clipped::Attachment(clip)) = read(ui.ctx()) else {
        // R9: nothing on the clipboard renders NOTHING, not a greyed button.
        // Greying is for the temporarily unavailable and this is not that — an
        // operator with an empty clipboard is not waiting for anything.
        return;
    };

    // ★★★ The question, asked BEFORE the button rather than after the press.
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
    // ★ One of the two, every frame. See `REGION_REPLACES`: this is what makes
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
            // ★ The clip travels with the action rather than being re-read at
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
    /// ★ The four regions are named apart, so a driven check aiming at one
    /// cannot match another by prefix.
    ///
    /// `attachments.paste` and `attachments.paste.replaces` deliberately share
    /// a stem — the harness's `declared_names(.., "attachments.paste")` lists
    /// both, which is wanted — but `declared(.., "attachments.paste")` is an
    /// exact match and resolves only the button.
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
