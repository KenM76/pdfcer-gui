//! # `app::dispatch::zoom` — every `view.zoom_*` command
//!
//! Split out of [`super`] under **R2** on 2026-08-24, when O29's third fit
//! mode took that file past 1,500 lines.
//!
//! ## The seam
//!
//! [`super`]'s subject is *"a command id becomes an intent"* across the whole
//! ribbon. This file's is the **zoom family's** share of it, and the two
//! change for different reasons: a new tab or a new dispatch convention
//! touches the parent, a new way of choosing a magnification touches this.
//!
//! It is also the family with the most to say per arm. Two of the six route to
//! `canvas::zoom` and one of those is *the one arm in the whole match whose
//! return value matters*; three are one-liners that exist only because the
//! difference between them is a `FitMode`; and the sixth carries the
//! distinction that was once a live defect — `ZoomTo(1.0)` is actual size and
//! `Fit(FitMode::None)` is not.
//!
//! ## ★ What did NOT move
//!
//! `view.zoom_in` and `view.zoom_out` are not here, because they are not
//! anywhere: their arms were **deleted**, since no such command is registered
//! and an arm no token can reach is dead code wearing a design pattern.
//! `RIBBON_IA.md` §6 puts those two on the status bar, which raises the
//! actions directly. `shell::commands::reach::UNREACHED_ARMS` carries what
//! registering them would take.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::viewer::FitMode;

/// Whether this file owns `id`.
///
/// `pub(crate)` rather than `pub(super)`: `shell::commands::reach`'s
/// `guard_claiming` calls it, because the reachability checker must be able to
/// EVALUATE every guard arm it finds — a guard it cannot evaluate is a place
/// commands could hide from the check that exists to find them.
///
/// A separate predicate rather than a `match` returning `bool`, for the reason
/// [`super::pages::handles`] gives: the caller is a guard on a match arm, and
/// the guard and the body must not be able to disagree about what is claimed.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    matches!(
        id,
        "view.zoom_selection"
            | "view.zoom_region"
            | "view.zoom_actual"
            | "view.zoom_fit_page"
            | "view.zoom_fit_width"
            | "view.zoom_fit_height"
    )
}

/// Route one zoom command.
///
/// The arms are the ones that used to sit in [`super`], carried across
/// unchanged along with the reasoning above each of them — which is the point
/// of an R2 split: the file moves, the argument moves with it.
pub(crate) fn dispatch(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    match id {
        // ★ …and this is the one arm whose RETURN VALUE matters.
        //
        // `ZoomOutcome` is `#[must_use]` precisely because its declining
        // variants are the point, and this arm used to discard it with a
        // `let _ =` — which is how "there is nothing to zoom to" became
        // "the command did nothing", the difference between a control that
        // declines and one that looks broken. `FEATURES.md` recorded the
        // gap as *traced and greyed but never worded*.
        //
        // The outcome is now carried into `status::decline`, which decides
        // whether it is a decline at all (a ceiling-clamped zoom is a
        // partial grant and is not worded), which sentence it gets, and
        // how long it lives. This arm decides none of that; it routes.
        //
        // The command is gated on `selection.bounds`, so a pressable
        // control usually has something to frame. The no-bounds answer is
        // still reachable two ways, and the second is why the sentence
        // exists: **by chord**, since a keymap reaches any command from any
        // state, and **in the race** where the bounds evaporate between the
        // frame that drew the enabled control and the frame that applied
        // it. In that second case the operator clicked something that was
        // offered to them and got nothing, which is exactly the situation
        // that must not be answered with silence.
        "view.zoom_selection" => {
            if let Status::Open(doc) = &mut app.status {
                let outcome = crate::canvas::zoom::zoom_to_selection(
                    ctx,
                    doc,
                    crate::canvas::CANVAS_MARGIN,
                    app.prefs.max_zoom_percent,
                    actions,
                );
                crate::app::status::decline::record(outcome);
            }
        }
        // Arms; does not act. The canvas disarms it when the drag ends,
        // so there is no "turn it off" arm to write.
        "view.zoom_region" => crate::canvas::zoom::arm_region_zoom(ctx),
        // ★ **`view.zoom_in`, `view.zoom_out`, `view.next_page` and
        // `view.prev_page` had arms here and no longer do.**
        //
        // Recorded rather than silently removed, because a reader who finds
        // `Action::ZoomIn` with no arm should not have to wonder whether one
        // was forgotten. None of the four ids is registered — no catalog
        // entry, no manifest item, no `crate::text::commands` copy, no
        // `RIBBON_IA.md` row — so **no token existed and no operator gesture
        // ever reached one.** They were the mirror of the defect
        // `shell::commands::reach` was built for: there, a control with no
        // arm; here, an arm with no control.
        //
        // All four verbs work, by the routes the specification actually puts
        // them on. `RIBBON_IA.md` §6 assigns *"zoom −/%/+, page ◀ n/N ▶"* to
        // the **status bar**, and that is where they are: `app::status`'s
        // zoom group raises `ZoomIn`/`ZoomOut`, `status::page_box` raises
        // `NextPage`/`PrevPage`, and `app::keyboard` raises all four from
        // `Ctrl` `+`/`-` and `PageDown`/`PageUp`. Deleting the arms removed
        // duplicate entrances, not behaviour.
        //
        // This arm's own neighbour states the rule they broke, and it is
        // quoted here rather than paraphrased: `format.delete` refuses an
        // `edit.delete` arm because it would be *"an arm no token can ever
        // reach — dead code wearing a design pattern, which is what the
        // no-placeholders invariant forbids."* Registering the four instead
        // was the other available answer and is a **ribbon** decision, not a
        // dispatch one; `shell::commands::reach::UNREACHED_ARMS` carries what
        // it would take.
        //
        // `ZoomTo(1.0)`, not `Fit(FitMode::None)`. The latter only stops the
        // per-frame re-fit and leaves the zoom where it was, so this
        // control used to pin whatever magnification happened to be
        // showing while promising one PDF point per screen point.
        "view.zoom_actual" => actions.push(Action::ZoomTo(1.0)),
        "view.zoom_fit_page" => actions.push(Action::Fit(FitMode::Page)),
        "view.zoom_fit_width" => actions.push(Action::Fit(FitMode::Width)),
        "view.zoom_fit_height" => actions.push(Action::Fit(FitMode::Height)),
        // Unreachable: `handles` above is the guard that admitted this call,
        // and the two are one list. Spelled out rather than folded into a
        // catch-all so that a seventh id added to `handles` and not to this
        // match fails loudly instead of silently doing nothing.
        // ui-text-exempt: a panic message, read from a stack trace.
        other => unreachable!("dispatch::zoom does not own {other}"),
    }
}
