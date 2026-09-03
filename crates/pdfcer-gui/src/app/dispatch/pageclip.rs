//! # `app::dispatch::pageclip` — **cut, copy and paste whole pages**
//!
//! `OPERATOR_REQUESTS.md` **O59**, item 2. Ken, 2026-08-29, to the engine
//! session: *"can you make sure we have cut, copy, and paste available for
//! everything and if not implement?"*
//!
//! ## ★★★ Why these are their own commands and NOT `Ctrl+C`
//!
//! This is the decision that shapes the whole module, and getting it the other
//! way round would have taken the clipboard away from the canvas permanently.
//!
//! Every `pages.*` verb takes its operand from one rule, written down once in
//! `panels::pages::ops::operands`: **the picked sheets when there are any, the
//! current page when there are none.** That fallback is right for Delete,
//! Rotate and Extract — with nothing picked they act on the sheet you are
//! looking at, which is a defined answer rather than a disabled state.
//!
//! It is fatal for a chord. A rung in `dispatch::clipboard`'s fork that asked
//! *"is there a page operand?"* would get **yes, always** — there is always a
//! current page — so `Ctrl+C` would copy a page instead of the shape the
//! operator had selected, for ever, and no state they could reach would give
//! the canvas its chord back.
//!
//! ⇒ So pages get **named commands** on the Pages tab and in the thumbnail
//! context menu, and the canvas keeps `Ctrl+C`. That is also what R8 asks for:
//! the capability exists because a command is registered, and it is reachable
//! by pointing at it rather than by knowing a rule.
//!
//! ★ Acrobat resolves the same collision by **focus** — `Ctrl+C` in its page
//! thumbnails copies pages. That is a legitimate answer and it is not available
//! here: this shell's thumbnails are a dock panel whose focus egui does not
//! model in a way a chord dispatcher can read, and inventing a focus notion to
//! serve one chord is a mechanism that would then own every other chord too.
//! Named and rejected rather than silently not done.
//!
//! ## Why copy is inline and cut raises an action
//!
//! `copy_pages` is `&self` — it changes no document — so it runs here, writes
//! the clipboard, and raises nothing. That is `dispatch::textcopy`'s rule
//! exactly: *"the action funnel exists for work that changes a document or that
//! must not happen mid-frame, and a copy is neither."*
//!
//! A **cut** does change one, so it is copy-then-`PageAction::DeletePages`:
//! the clip is captured first, and the existing delete arm — which already
//! resyncs the panel selection, clears the canvas and is one undo entry — does
//! the removal.
//!
//! ★★ The engine ships `cut_pages`, which does both in one call, and it is
//! **not** used. Not an oversight: the clipboard lives in `egui::Memory` and
//! the action applier has no `egui::Context`, so a single-call cut could not
//! put its own clip anywhere. Copy-then-delete costs one extra page-tree walk
//! and keeps the undo entry count at one, which was the property the engine's
//! verb existed to guarantee. Recorded here so the next reader does not
//! rediscover the constraint by trying.
//!
//! ## ★★ What the operator must be told, and when
//!
//! Two disclosures, and they are at opposite ends of the gesture because they
//! answer different questions:
//!
//! | when | what | why it cannot wait / cannot be earlier |
//! |---|---|---|
//! | **at the copy** | *"a form field was left behind"* | `PageClip::fields_dropped` — a field whose boxes straddle a copied and an uncopied sheet cannot travel, and the operator selected **pages**, not fields, so nothing they did says a field is about to go missing |
//! | **at the paste** | *"boxes arrived that nothing can fill"* | `InsertOutcome::orphaned_widgets` — a page's `/Annots` reaches its widgets and the `/AcroForm` that owns them does not travel, so they draw like fields and are dead. The engine measured two on its own smoke test |
//!
//! ★★★ The second is the one the engine flagged as *"the one that produces a
//! document that looks right and is not"*, and it is invisible by construction:
//! an orphaned widget draws exactly like a live field. There is no screenshot
//! that shows the difference, so the status row is the only place it can be
//! said — which is rule 4's surviving half, again.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::state::Status;

/// **Whether this module owns `id`.**
///
/// Listed rather than prefix-matched, for `dispatch::clipboard::handles`'
/// reason: `pages.*` holds six other verbs that are not this module's.
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(id, "pages.copy" | "pages.cut" | "pages.paste")
}

/// Route one page-clipboard command.
pub fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    match id {
        "pages.copy" => copy_or_cut(app, ctx, false, actions),
        "pages.cut" => copy_or_cut(app, ctx, true, actions),
        "pages.paste" => paste(app, ctx, actions),
        _ => {}
    }
}

/// Copy the operand sheets, and on a cut also raise their deletion.
///
/// ★ The copy runs **first and unconditionally**, so a cut whose delete is
/// refused still leaves the pages on the clipboard rather than losing them —
/// the opposite of `canvas::clipboard::cut`'s ordering, and deliberately so.
/// There the refusal is *about the thing being cut* and a half-executed cut
/// leaves a duplicate; here the delete arm's own gate is about the **document**
/// and a copy is harmless either way.
fn copy_or_cut(app: &mut PdfcerApp, ctx: &egui::Context, cutting: bool, actions: &mut Vec<Action>) {
    let Some(pages) = app.page_operands() else {
        return;
    };
    let Status::Open(doc) = &app.status else {
        return;
    };

    let clip = match doc.session.copy_pages(&pages) {
        Ok(clip) => clip,
        Err(e) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("pageclip-copy-refused n={} err={e}", pages.len())
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::pageclip::copy_refused(&e.to_string()),
            );
            return;
        }
    };

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "pageclip-copy pages={} bytes={} fields_dropped={} cut={cutting}",
            clip.pages,
            clip.bytes.len(),
            clip.fields_dropped
        )
    });

    // ★★ The disclosure the operator cannot see: they picked SHEETS, and a form
    // field whose boxes straddle a picked and an unpicked one is left behind.
    // Nothing about the thumbnails says so.
    if clip.fields_dropped > 0 {
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::pageclip::fields_dropped(clip.fields_dropped),
        );
    }

    crate::canvas::clipboard::store(
        ctx,
        crate::canvas::clipboard::Clipped::Pages {
            bytes: clip.bytes,
            count: clip.pages,
        },
    );
    // ★ The OS clipboard, for `canvas::clipboard::copy_content`'s reason and
    // not as a courtesy: this shell's paste is a command rather than a chord,
    // so `Event::Paste` is not in play here — but a person who copies pages and
    // then pastes into an email deserves an explanation rather than silence,
    // and the marker is what makes a pdfcer→pdfcer paste discoverable at all.
    ctx.copy_text(crate::text::pageclip::os_marker(clip.pages));

    if cutting {
        actions.push(Action::Page(PageAction::DeletePages { pages }));
    }
}

/// Paste the clipboard's pages after the current sheet.
///
/// # ★ Why after the current page, and not at the end
///
/// Because the operator is looking at a sheet, and *"put these here"* is what a
/// paste means everywhere else in this shell — a markup lands where the page is
/// showing, a form field lands under the pointer. Appending to the end would be
/// defensible only if the Pages panel had an insertion caret to point at, and
/// it does not.
///
/// Acrobat's own page paste offers Before/After/First/Last in a dialog. That is
/// a dialog on every paste, and the engine's `InsertPosition` carries all four
/// — so the other three are a *future control*, not a missing capability. This
/// picks the one an operator wants most of the time and says where it went.
fn paste(app: &mut PdfcerApp, ctx: &egui::Context, actions: &mut Vec<Action>) {
    let Some(crate::canvas::clipboard::Clipped::Pages { bytes, count }) =
        crate::canvas::clipboard::read(ctx)
    else {
        let Status::Open(doc) = &app.status else {
            return;
        };
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::pageclip::nothing_copied().to_owned(),
        );
        return;
    };
    let Status::Open(doc) = &app.status else {
        return;
    };
    let after = doc.view.page_index;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "pageclip-paste pages={count} after={after} bytes={}",
            bytes.len()
        )
    });
    actions.push(Action::Page(PageAction::PastePages { bytes, after }));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three ids, and none of the six `pages.*` verbs that are not ours.
    ///
    /// ★ The negative half is the half that matters. `dispatch::pages` owns
    /// `pages.delete` and this module raises `PageAction::DeletePages`, so a
    /// prefix rule here would claim the very command the cut delegates to and
    /// route it back into this module — a loop that compiles.
    #[test]
    fn handles_the_three_and_none_of_the_neighbours() {
        for id in ["pages.copy", "pages.cut", "pages.paste"] {
            assert!(handles(id), "{id} must route here");
        }
        for id in [
            "pages.delete",
            "pages.extract",
            "pages.rotate_left",
            "pages.rotate_right",
            "pages.move_up",
            "pages.move_down",
        ] {
            assert!(
                !handles(id),
                "★ {id} is `dispatch::pages`', not this module's"
            );
        }
        assert!(!handles("edit.copy"), "the canvas keeps its own chord");
    }
}
