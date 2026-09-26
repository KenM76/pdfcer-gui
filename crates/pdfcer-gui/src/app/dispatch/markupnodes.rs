//! # `app::dispatch::markupnodes` — the three commands about a markup shape's
//! **points**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/markupnodes.md`.

use egui::Context;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;

/// Whether `id` is one of the point commands this module dispatches.
///
/// ★ Named `claims` rather than `handles` because
/// `shell::commands::reach::guards::EVALUATED_GUARDS` is a set of **function
/// names** read out of `dispatch.rs`'s syntax tree and asserted equal to the
/// set the reachability checker evaluates. A guard function whose name is not
/// in that set is a place commands can hide from the check that exists to find
/// them, so a new module reuses an evaluated name rather than inventing one.
///
/// ★★ A predicate paired with [`dispatch`] over the same ids, which is two
/// statements of one set — the shape this crate usually refuses. It is accepted
/// here only because the two sit adjacent in one small file. **Nothing
/// mechanical welds them**: the `match` below ends in a `_ => {}`, so an id
/// added here and not there is a control that does nothing and says nothing.
/// Add to both, in the same edit.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    matches!(
        id,
        "markup.finish" | "markup.add_node" | "markup.remove_node"
    )
}

/// Dispatch one of the point commands.
pub(super) fn dispatch(app: &mut PdfcerApp, ctx: &Context, id: &str, actions: &mut Vec<Action>) {
    // ★ The capability, asked once for every arm. See the module header: they
    // decline alike because they are one capability, and the trace names the
    // command so a reader can still tell which was pressed.
    if !app.capabilities().author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-author-markup")
        });
        return;
    }
    match id {
        // ★ **Finish** — `measure.finish`'s twin, deliberately down to the
        // shape of this arm, because it answers the identical problem: PolyLine
        // and Polygon are runs of clicks with no natural end, exactly as the
        // radius/diameter pick set has none. The settled answer is **two
        // endings through one commit path**. A double-click on the canvas is
        // the other half and is the one most operators will use; this is the
        // discoverable one, and the one that works when the last vertex sits
        // somewhere awkward to double-click.
        //
        // It must not be reached by `markup_for_command`'s arm in `super`:
        // that mapping takes ids to *kinds*, this id names no kind, and if it
        // ever did, pressing Finish would toggle the tool off (`arm_markup`'s
        // same-kind-retires rule) instead of committing. Moving it into a
        // module of its own makes that ordering constraint structural rather
        // than a comment — `super`'s guard arm for this module sits above the
        // one for that mapping, and neither can now be reordered by accident
        // within one `match`.
        //
        // The arm routes and does not compute. Everything about what a finish
        // *is* — whether the run is long enough for its kind, which page it
        // belongs to, emptying it afterwards — lives in
        // `canvas::markup::vertex::finish`, which is the same commit path the
        // canvas's double-click reaches. One commit path, two entrances; a
        // second derivation here is exactly how the two endings would come to
        // author different annotations.
        "markup.finish" => {
            if !crate::canvas::markup::vertex::finish(ctx, actions, app.pen) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // Reachable only by a chord or a customized manifest: the
                    // ribbon control is greyed unless there is a run long
                    // enough for its kind, by the same predicate `finish`
                    // itself asks.
                    format!("command-declined id={id} reason=no-vertex-run-to-finish")
                });
            }
        }
        // ★★★ **The two node commands.** One arm, because the id IS the
        // operand's direction — add or remove — and everything else about the
        // operand is the parked pick that `action_for` reads.
        //
        // ★★ A `let else` rather than a `match` enumerating every `Status`, on
        // `super`'s own stated reasoning for the text-mark arm: every state but
        // `Open` is *no document*, therefore no selection, therefore no shape
        // to reshape, and that is the only property this arm reads. A new
        // failure state arriving later does not have to be classified here.
        //
        // ★ It raises **nothing** when the row it came from has stopped being
        // live — an undo between the frame the menu was drawn and the frame it
        // was pressed. `action_for` re-asks the engine and traces the decline;
        // it does not put a sentence on the status row, because a menu that
        // closed is not a surface an explanation can arrive on and the greyed
        // row was already R9's answer.
        "markup.add_node" | "markup.remove_node" => {
            let Status::Open(doc) = &app.status else {
                return;
            };
            if let Some(action) = crate::canvas::annotnodes::menu::action_for(
                ctx,
                doc,
                &doc.selection,
                id == "markup.add_node",
            ) {
                actions.push(action);
            }
        }
        // ★ No catch-all that does anything: [`claims`] is the only route in,
        // and it and this `match` are the two statements of one set the header
        // above accepts. An id that reached here without being in that list
        // would be a bug in `super`'s routing, and doing nothing is the
        // honest response — `super`'s own catch-all traces
        // `command-unimplemented` for everything nobody claimed.
        _ => {}
    }
}
