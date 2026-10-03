//! # `app::dispatch::clipboard` — cut, copy, the two pastes, and duplicate
//!
//! Four ids — `edit.cut`, `edit.copy`, `edit.paste`, `edit.paste_duplicate` —
//! over **three kinds of operand**, and the whole subject of this module is the
//! fork that decides which of them a keystroke is about.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/clipboard.md`.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::fieldclip::PasteAs;

/// **Whether this module owns `id`.**
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(
        id,
        "edit.cut"
            | "edit.copy"
            | "edit.copy_as_vector"
            | "edit.paste"
            | "edit.paste_duplicate"
            // `edit.duplicate`, 2026-09-06 — and it is the one id here that
            // never touches the clipboard. It is routed to this module anyway
            // because *"make another one of this"* is what the operator was
            // doing with Copy-then-Paste before it existed, and because the
            // three-rung fork at the head of this file is the machinery it
            // needs: which operand does the gesture mean?
            | "edit.duplicate"
            // Another program's picture as a stamp: `dispatch::ospaste`.
            | "markup.paste_image_stamp"
    ) || super::ospaste::pages::handles(id)
}

/// Route one clipboard command.
pub fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    match id {
        "edit.copy" | "edit.cut" => copy_or_cut(app, ctx, id, actions),
        // The copy-OUT takes no `ctx` and raises no `Action`, and both
        // absences are the point: it neither reads the internal clipboard nor
        // changes the document. It renders and it places. See `copy_as_vector`.
        "edit.copy_as_vector" => copy_as_vector(app),
        "edit.paste" => paste(app, ctx, id, PasteAs::NewField, actions),
        "edit.paste_duplicate" => paste(app, ctx, id, PasteAs::Duplicate, actions),
        // Takes no `ctx`, like the copy-OUT and for the mirror reason: it
        // neither reads nor writes the clipboard, so there is nothing in
        // `egui`'s memory for it to consult. Its whole operand is the
        // selection.
        "edit.duplicate" => duplicate(app, id, actions),
        "markup.paste_image_stamp" => super::ospaste::paste_stamp(app, ctx, id, actions),
        _ => super::ospaste::pages::dispatch(app, id, actions),
    }
}

/// **`edit.duplicate`** — a second copy of the selected comment, offset,
/// **without using the clipboard**. `Ctrl+D`.
fn duplicate(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let caps = app.capabilities();
    let Status::Open(doc) = &app.status else {
        return;
    };
    let epoch = doc.edit_epoch;
    if !caps.author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-author-markup")
        });
        crate::app::status::decline::record_mode_refusal(
            crate::text::clipboard::ModeRefusal::DuplicateMarkup,
        );
        return;
    }
    if let Err(refusal) = crate::canvas::annotclip::duplicate(doc, actions) {
        crate::app::actions::record_note(epoch, crate::text::clipboard::refusal(refusal));
    }
}

/// `Ctrl+C` and `Ctrl+X`, through the three-rung fork.
fn copy_or_cut(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    // RUNG 0 — a laid snapshot box is what Copy is about. Cut falls through.
    if id == "edit.copy" && super::snapshotclip::owns_copy(app) {
        super::snapshotclip::copy(app);
        return;
    }
    let Status::Open(doc) = &app.status else {
        return;
    };
    let cutting = id == "edit.cut";

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

    // RUNG 2 — A FORM FIELD. See the header for why this rung had to be
    // added rather than merely widened: nothing below can see a `/Widget`.
    if doc.selected_field.is_some() {
        let caps = app.capabilities();
        if cutting && !caps.edit_content {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("command-declined id={id} reason=mode-cannot-remove-field")
            });
            crate::app::status::decline::record_mode_refusal(
                crate::text::clipboard::ModeRefusal::CutField,
            );
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
    // The gate follows WHAT IS SELECTED, not the command. A cut removes
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
        crate::app::status::decline::record_mode_refusal(if content {
            crate::text::clipboard::ModeRefusal::CutContent
        } else {
            crate::text::clipboard::ModeRefusal::CutMarkup
        });
        return;
    }
    // Copy is permitted in every mode and cut is not, and the split is the
    // operator's own *copying is not authoring* ruling.
    let outcome = if cutting {
        crate::canvas::clipboard::cut(ctx, doc, actions)
    } else {
        crate::canvas::clipboard::copy(ctx, doc)
    };
    match outcome {
        Err(refusal) => crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::clipboard::refusal(refusal),
        ),
        // **A PARTIAL COPY SAYS SO** — rule 4, "fuzzy never sneaky",
        // applied to the clipboard.
        //
        // A copy that took three of four selected things looks *identical* to
        // one that took all four: nothing errors, the marker goes on the OS
        // clipboard, and the operator finds out when they paste — or, worse,
        // does not, because what went missing was a comment's author and
        // opacity rather than a shape.
        //
        // It is said HERE and not in `canvas::clipboard`, whose standing
        // contract is that it changes no document and words no decline. The
        // clip carries the facts (`left_behind`, `thin`) and this is the layer
        // that has a status row.
        //
        // It is said on the status row rather than drawn on the canvas,
        // which is the other half of rule 4: **the pasted mark must render
        // exactly as a saved one will**, so a partial paste gets no badge, no
        // tint and no provisional styling. The disclosure lives off-canvas or
        // it is a lie about the document.
        Ok(crate::canvas::clipboard::Clipped::Selection {
            left_behind, thin, ..
        }) if !left_behind.is_empty() || thin > 0 => {
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::clipboard::partial_copy(&left_behind, thin),
            );
        }
        Ok(_) => {}
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
    if let Some(incoming) = super::ospaste::newer(ctx, clipped.is_some()) {
        super::ospaste::paste(app, ctx, id, incoming, actions);
        return;
    }

    // The gate follows WHAT IS ON THE CLIPBOARD, for the same reason the
    // cut's follows what is selected: a paste has no operand on the page to
    // look at, so the clipboard is the only honest source.
    let caps = app.capabilities();
    //
    let (allowed, refusal) = match &clipped {
        //
        //
        // The stricter gate wins on a mixed clip, and it has to: pasting one
        // is one act, so a mode that may not add a line to a drawing may not
        // add three lines and a cloud either. Asking `author_markup` for the
        // whole thing would let Review paste geometry.
        Some(crate::canvas::clipboard::Clipped::Selection { count, .. }) => {
            if *count > 0 {
                (
                    caps.edit_content,
                    crate::text::clipboard::ModeRefusal::PasteContent,
                )
            } else {
                (
                    caps.author_markup,
                    crate::text::clipboard::ModeRefusal::PasteMarkup,
                )
            }
        }
        // A form field is document content, not a comment on it, so it takes
        // the content gate rather than the markup one. Review may annotate a
        // drawing; it may not add a fillable box to it.
        Some(crate::canvas::clipboard::Clipped::FormField(_)) => (
            caps.edit_content,
            crate::text::clipboard::ModeRefusal::PasteField,
        ),
        // An empty clipboard takes the markup gate, so the refusal an operator
        // gets in Read is the mode's rather than "nothing has been copied" —
        // which would be true and useless, because copying something would not
        // help.
        _ => (
            caps.author_markup,
            crate::text::clipboard::ModeRefusal::PasteMarkup,
        ),
    };
    if !allowed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-paste-here")
        });
        //
        // Until today this `return` was the whole answer: a trace line, and
        // nothing on any surface. It was *reachable only in Read*, because
        // `offers_command` refused the chord in Review before it got here — so
        // the silence was hidden behind a different defect. Opening the chord
        // to every mode makes this the path an operator actually takes when
        // they paste a drawing's geometry into Review, and a keypress that does
        // nothing and says nothing is this project's founding defect class.
        crate::app::status::decline::record_mode_refusal(refusal);
        return;
    }

    let page = doc.view.page_index;
    let epoch = doc.edit_epoch;

    // **WHERE THE POINTER IS, IN PDF USER SPACE** —
    // `OPERATOR_REQUESTS.md` O73: *"When I cut or copy an object, when I paste
    // it should paste where the mouse cursor is sitting."*
    //
    // Computed once, here, before the three forks below, so a markup paste, a
    // content paste and a field paste cannot land in three different places
    // for three different reasons.
    //
    // `zoom::anchor_point` answers the "what if the pointer is not over the
    // canvas?" question and it is not a new rule: it honours the pointer only
    // while it lies inside the viewport, and otherwise returns the viewport's
    // own **centre**. So a `Ctrl+V` pressed while the pointer is over a dock,
    // over the ribbon or off the window pastes into the middle of what the
    // operator is looking at — which is the conventional answer — and it is
    // the SAME function the zoom anchor uses, so there is one rule about where
    // the pointer counts rather than two that can drift apart.
    //
    // The page comes from the recorded frame rather than from
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
        // THE ONE THING WORTH SAYING *BEFORE* THE PRESS, and it is the only
        // pre-press disclosure this shell owes on a paste.
        //
        // Everything else a paste carries or drops is reported AFTER, by the
        // engine, through `FieldPasteOutcome::disclosures` — which is
        // authoritative because it reports what the operation did rather than
        // what the shell intended. This one cannot wait, because a field in a
        // calculation chain looks identical on the page to one that is not: the
        // operator has no way to know a script is coming until it has come.
        //
        // The engine deliberately does NOT resolve the field names inside the
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

    // `edit.paste_duplicate` over a markup or over page content falls through
    // to the ordinary paste rather than refusing. See the header: neither has a
    // second sense to duplicate into, so the paste is the honest answer to the
    // more specific chord.
    if let Err(refusal) = crate::canvas::clipboard::paste(ctx, page, target, actions) {
        crate::app::actions::record_note(epoch, crate::text::clipboard::refusal(refusal));
    }
}

/// **`edit.copy_as_vector`** — put the page, or the selection on it, on the
/// operating system's clipboard as **editable geometry**.
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
    #[test]
    fn handles_the_six_and_not_the_registered_absence() {
        for id in [
            "edit.cut",
            "edit.copy",
            "edit.copy_as_vector",
            "edit.paste",
            "edit.paste_duplicate",
            "edit.duplicate",
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
