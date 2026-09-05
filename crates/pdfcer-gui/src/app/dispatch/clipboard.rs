//! # `app::dispatch::clipboard` — cut, copy and the two pastes
//!
//! Four ids — `edit.cut`, `edit.copy`, `edit.paste`, `edit.paste_duplicate` —
//! over **three kinds of operand**, and the whole subject of this module is the
//! fork that decides which of them a keystroke is about.
//!
//! ## Why this is a module and not four match arms
//!
//! `super`'s file crossed R2's 1,500-line ceiling for the fourth time when
//! `edit.paste_duplicate` arrived on 2026-08-29. It joins [`super::pages`],
//! [`super::images`] and [`super::textcopy`] as the fourth application of the
//! same seam, and it is the right seam independently of the line count: the
//! three-way fork below is the *entire* logic here, and a reader trying to
//! answer *"what does Ctrl+C do?"* should find it in one screen rather than
//! interleaved with tool arming and zoom.
//!
//! ## ★★★ The fork, in priority order, and why each rung is where it is
//!
//! | rung | operand | who answers | why it is above the next |
//! |---|---|---|---|
//! | 1 | **swept text or a live text draft** | `canvas::textsel` / the widget | the operator made the narrower statement more recently, and every program in the class resolves it this way |
//! | 2 | **a selected form field** | [`crate::canvas::fieldclip`] | a `/Widget` is deliberately not an annotation selection here, so nothing below can see one |
//! | 3 | **an annotation, or page content** | [`crate::canvas::clipboard`] | the general case |
//!
//! Rung 1 is `text_owns_the_chord`, and its full argument — including why a
//! focused Find box counts even with no selection in it — lives on that
//! function beside the claim it enforces.
//!
//! ★★ **Rung 2 is the one that was missing**, and its absence was not a lossy
//! path but *no path at all*. `canvas::clipboard::copy` reads `doc.selection`;
//! a selected form field lives on `doc.selected_field`; so `Ctrl+C` over a
//! field with visible grips around it fell through to the content copy and
//! refused with *"nothing is selected"*. `DEFECTS.md` D4a's shape exactly: a
//! sentence describing a different world than the one on screen.
//!
//! ## ★★ The two pastes are two commands, not one command with a modifier
//!
//! **Ken, 2026-08-29:** *"ctrl v for paste as new. ctrl shift v for paste as
//! duplicate."* — `OPERATOR_REQUESTS.md` **O58**.
//!
//! They are separate ids because a command is the unit this shell can
//! *register*, *bind*, *place on a ribbon*, *put in a context menu* and
//! *withhold by mode* (R8). A single `edit.paste` that read the modifier keys
//! itself would be reachable only from the keyboard: there would be nothing to
//! put in the Edit menu beside Paste, nothing to grey out with an explanation
//! when the clipboard holds a markup rather than a field, and nothing for the
//! keymap editor to rebind.
//!
//! ★ `edit.paste_duplicate` over a **non-field** clipboard is not an error and
//! not a silent no-op: it falls through to the ordinary paste. A markup has no
//! second sense to duplicate into, so the honest answer to *"paste that as a
//! duplicate"* is the paste. Refusing would punish an operator for pressing the
//! more specific chord when the general one was all that applied.
//!
//! ## Mode gating, and why each gate reads a different thing
//!
//! - **Cut** is gated on *what is selected*, because a cut removes that thing.
//! - **Paste** is gated on *what is on the clipboard*, because a paste has no
//!   operand on the page to look at.
//! - **Copy** is gated on nothing. The operator's own ruling — *copying is not
//!   authoring* — the same line that put `file.copy_page_text` in Read mode.
//!
//! A form field is content: cutting or pasting one takes `edit_content`, the
//! same predicate the Delete key reads, because a form field is part of the
//! document rather than a comment on it.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::fieldclip::PasteAs;

/// **Whether this module owns `id`.**
///
/// Listed rather than prefix-matched. `edit.paste_in_place` is a *registered
/// absence* (`shell::manifest::registers`) and a prefix rule would silently
/// claim it the day someone made it real, routing it here with no body.
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(
        id,
        "edit.cut" | "edit.copy" | "edit.copy_as_vector" | "edit.paste" | "edit.paste_duplicate"
    )
}

/// Route one clipboard command.
pub fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    match id {
        "edit.copy" | "edit.cut" => copy_or_cut(app, ctx, id, actions),
        // ★★ The copy-OUT takes no `ctx` and raises no `Action`, and both
        // absences are the point: it neither reads the internal clipboard nor
        // changes the document. It renders and it places. See `copy_as_vector`.
        "edit.copy_as_vector" => copy_as_vector(app),
        "edit.paste" => paste(app, ctx, id, PasteAs::NewField, actions),
        "edit.paste_duplicate" => paste(app, ctx, id, PasteAs::Duplicate, actions),
        _ => {}
    }
}

/// `Ctrl+C` and `Ctrl+X`, through the three-rung fork.
fn copy_or_cut(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let cutting = id == "edit.cut";

    // ★★ RUNG 1 — TEXT WINS. Defect O18: both handlers see the same
    // `Event::Copy` in the same frame, and until 2026-08-21 the object path ran
    // anyway and wrote its marker over what the text path had put on the
    // clipboard. The operator swept some text, pressed Ctrl+C, pasted into
    // Notepad and got "1 object copied from pdfcer".
    //
    // The collision is resolved in the BROADER verb because only this one can
    // see both operands. Cut is included deliberately: cutting swept page text
    // is not a thing pdfcer can do, so the right answer is nothing rather than
    // quietly cutting the object underneath it.
    if crate::canvas::clipboard::text_owns_the_chord(ctx, doc) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=text-owns-the-clipboard")
        });
        return;
    }

    // ★★★ RUNG 2 — A FORM FIELD. See the header for why this rung had to be
    // added rather than merely widened: nothing below can see a `/Widget`.
    if doc.selected_field.is_some() {
        let caps = app.capabilities();
        if cutting && !caps.edit_content {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("command-declined id={id} reason=mode-cannot-remove-field")
            });
            return;
        }
        let outcome = if cutting {
            crate::canvas::fieldclip::cut(ctx, doc, actions).map(|_| ())
        } else {
            crate::canvas::fieldclip::copy(ctx, doc).map(|_| ())
        };
        if let Err(refusal) = outcome {
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::fieldclip::refusal(&refusal),
            );
        }
        return;
    }

    // RUNG 3 — an annotation, or page content.
    //
    // ★★ The gate follows WHAT IS SELECTED, not the command. A cut removes
    // something, so it needs a mode that may remove that kind of thing.
    // Cutting an annotation needs `author_markup` (Review and Edit); cutting
    // page content needs `edit_content` (Edit alone), the same predicate the
    // Delete key is gated on and must be, because a cut IS a delete with a copy
    // in front of it. Asking `author_markup` for both would have let Review cut
    // a line off a drawing.
    let caps = app.capabilities();
    let content = doc.selection.annot().is_none() && !doc.selection.is_empty();
    let allowed = if content {
        caps.edit_content
    } else {
        caps.author_markup
    };
    if cutting && !allowed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "command-declined id={id} reason=mode-cannot-remove-{}",
                if content { "content" } else { "markup" }
            )
        });
        return;
    }
    // ★ Copy is permitted in every mode and cut is not, and the split is the
    // operator's own *copying is not authoring* ruling.
    let outcome = if cutting {
        crate::canvas::clipboard::cut(ctx, doc, actions).map(|_| ())
    } else {
        crate::canvas::clipboard::copy(ctx, doc).map(|_| ())
    };
    if let Err(refusal) = outcome {
        crate::app::actions::record_note(doc.edit_epoch, crate::text::clipboard::refusal(refusal));
    }
}

/// `Ctrl+V` and `Ctrl+Shift+V`.
fn paste(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    mode: PasteAs,
    actions: &mut Vec<Action>,
) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let clipped = crate::canvas::clipboard::read(ctx);

    // ★ The gate follows WHAT IS ON THE CLIPBOARD, for the same reason the
    // cut's follows what is selected: a paste has no operand on the page to
    // look at, so the clipboard is the only honest source.
    let caps = app.capabilities();
    let allowed = match &clipped {
        Some(crate::canvas::clipboard::Clipped::Content { .. }) => caps.edit_content,
        // ★ A form field is document content, not a comment on it, so it takes
        // the content gate rather than the markup one. Review may annotate a
        // drawing; it may not add a fillable box to it.
        Some(crate::canvas::clipboard::Clipped::FormField(_)) => caps.edit_content,
        // An empty clipboard takes the markup gate, so the refusal an operator
        // gets in Read is the mode's rather than "nothing has been copied" —
        // which would be true and useless, because copying something would not
        // help.
        _ => caps.author_markup,
    };
    if !allowed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-paste-here")
        });
        return;
    }

    let page = doc.view.page_index;
    let epoch = doc.edit_epoch;

    // ★★★ **WHERE THE POINTER IS, IN PDF USER SPACE** —
    // `OPERATOR_REQUESTS.md` O73: *"When I cut or copy an object, when I paste
    // it should paste where the mouse cursor is sitting."*
    //
    // Computed once, here, before the three forks below, so a markup paste, a
    // content paste and a field paste cannot land in three different places
    // for three different reasons.
    //
    // ★★ `zoom::anchor_point` answers the "what if the pointer is not over the
    // canvas?" question and it is not a new rule: it honours the pointer only
    // while it lies inside the viewport, and otherwise returns the viewport's
    // own **centre**. So a `Ctrl+V` pressed while the pointer is over a dock,
    // over the ribbon or off the window pastes into the middle of what the
    // operator is looking at — which is the conventional answer — and it is
    // the SAME function the zoom anchor uses, so there is one rule about where
    // the pointer counts rather than two that can drift apart.
    //
    // ★ The page comes from the recorded frame rather than from
    // `doc.view.page_index`, so a paste aimed at a strip page lands on the
    // sheet the operator is pointing at. `None` — the canvas has never drawn —
    // falls every paste back to the offset rule it used before today.
    let target = crate::canvas::zoom::last_frame(ctx).and_then(|f| {
        let canvas = crate::canvas::zoom::anchor_point(ctx.pointer_latest_pos(), &f);
        crate::viewer::canvas_to_pdf_space(canvas, doc.pages.get(f.page)?)
    });

    if matches!(
        clipped,
        Some(crate::canvas::clipboard::Clipped::FormField(_))
    ) {
        // ★★★ THE ONE THING WORTH SAYING *BEFORE* THE PRESS, and it is the only
        // pre-press disclosure this shell owes on a paste.
        //
        // Everything else a paste carries or drops is reported AFTER, by the
        // engine, through `FieldPasteOutcome::disclosures` — which is
        // authoritative because it reports what the operation did rather than
        // what the shell intended. This one cannot wait, because a field in a
        // calculation chain looks identical on the page to one that is not: the
        // operator has no way to know a script is coming until it has come.
        //
        // ★ The engine deliberately does NOT resolve the field names inside the
        // script, and says so. Acrobat is documented silently dropping a copied
        // JavaScript reference to a field the target lacks — discovered only on
        // reopen — and naming the uncertainty beats half-analysing it.
        if crate::canvas::fieldclip::carries_actions(ctx) == Some(true) {
            crate::app::actions::record_note(
                epoch,
                crate::text::fieldclip::brings_a_script().to_owned(),
            );
        }
        if let Err(refusal) = crate::canvas::fieldclip::paste(ctx, doc, page, mode, target, actions)
        {
            crate::app::actions::record_note(epoch, crate::text::fieldclip::refusal(&refusal));
        }
        return;
    }

    // ★ `edit.paste_duplicate` over a markup or over page content falls through
    // to the ordinary paste rather than refusing. See the header: neither has a
    // second sense to duplicate into, so the paste is the honest answer to the
    // more specific chord.
    if let Err(refusal) = crate::canvas::clipboard::paste(ctx, page, target, actions) {
        crate::app::actions::record_note(epoch, crate::text::clipboard::refusal(refusal));
    }
}

/// ★★★ **`edit.copy_as_vector`** — put the page, or the selection on it, on the
/// operating system's clipboard as **editable geometry**.
///
/// `OPERATOR_REQUESTS.md` **O120**, 2026-09-03: *"Also I'd like to be able to
/// copy and paste anything to other software - like copy and paste vector
/// graphics into word or inkscape for example if possible."*
///
/// # ★★ Why this is a fifth id and not a modifier on `edit.copy`
///
/// The same argument the two pastes make one screen up, and it holds harder
/// here: **a command is the unit this shell can register, bind, place on a
/// ribbon, put in a menu and withhold by mode.** A modifier read inside
/// `copy_or_cut` would be reachable from the keyboard alone — nothing to draw
/// in the Clipboard group, nothing to name in a tooltip, and nothing for an
/// operator to discover. This is a *discoverability* feature as much as a
/// capability one: the operator did not know pdfcer could do it, which is why
/// he asked.
///
/// ⇒ And the two verbs genuinely differ in what they produce. `edit.copy` puts
/// an internal clip plus a picture on the clipboard, for pasting back into
/// pdfcer. This puts four public formats on it, for pasting into somebody
/// else's program, and touches the internal clipboard not at all — so a copy-out
/// does not destroy what the operator had copied for an in-pdfcer paste.
///
/// # ★★ It says something on SUCCESS, which no other clipboard verb here does
///
/// Because it alone has two possible operands and the button cannot show which
/// was taken: the selection if there is one, the whole page otherwise. An
/// operator who selected three parts and silently got the sheet finds out in
/// Word, minutes later. `text::clipboard::copied_as_vector` carries the wording
/// and the argument.
///
/// # No mode gate, deliberately
///
/// *Copying is not authoring* — the operator's own ruling, the same line that
/// leaves `edit.copy` ungated above and put `file.copy_page_text` in Read. The
/// Edit tab is not shown outside Edit mode, so the control is absent rather than
/// refusing there; that is visibility doing the work, which is the rule
/// `app::modes` states.
fn copy_as_vector(app: &mut PdfcerApp) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let epoch = doc.edit_epoch;
    match crate::clipboard::place::copy_out(doc) {
        Ok(placed) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed. The FORMAT
                // NAMES go here and not on the status row: they are wire
                // identifiers a developer greps for, and an operator can act on
                // none of them.
                format!(
                    "clipboard-copy-out selection={} formats={}",
                    placed.selection,
                    placed.formats.join(",")
                )
            });
            crate::app::actions::record_note(
                epoch,
                crate::text::clipboard::copied_as_vector(placed.selection, placed.formats.len()),
            );
        }
        Err(refusal) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("clipboard-copy-out-refused reason={refusal:?}")
            });
            crate::app::actions::record_note(
                epoch,
                crate::text::clipboard::copy_out_refusal(&refusal),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five ids, and nothing adjacent.
    ///
    /// `edit.paste_in_place` is the trap this test exists for: it is a
    /// registered ABSENCE, and a prefix rule would claim it the day it became
    /// real, routing it here with no body and no failure.
    ///
    /// ★ Four until 2026-09-04, when `edit.copy_as_vector` joined. It is listed
    /// here rather than trusted to the `edit.` prefix for the same reason the
    /// absence is: `shell::commands::reach` proves every registered id is
    /// routed by reading THIS function, so an id that only a prefix would have
    /// claimed is an id nothing proves has a body.
    #[test]
    fn handles_the_five_and_not_the_registered_absence() {
        for id in [
            "edit.cut",
            "edit.copy",
            "edit.copy_as_vector",
            "edit.paste",
            "edit.paste_duplicate",
        ] {
            assert!(handles(id), "{id} must route here");
        }
        assert!(
            !handles("edit.paste_in_place"),
            "★ a registered ABSENCE must not be claimed by a prefix rule"
        );
        assert!(!handles("file.copy_page_text"), "that is textcopy's");
        assert!(!handles("edit.delete"));
    }
}
