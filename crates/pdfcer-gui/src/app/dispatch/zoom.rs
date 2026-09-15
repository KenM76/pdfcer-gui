//! # `app::dispatch::zoom` — every `view.zoom_*` command
//!
//! ## The seam
//!
//! [`super`]'s subject is *"a command id becomes an intent"* across the whole
//! ribbon. This file's is the **zoom family's** share of it, and the two
//! change for different reasons: a new tab or a new dispatch convention
//! touches the parent, a new way of choosing a magnification touches this.
//!
//! ## ★ What is NOT here, and must not be added
//!
//! `view.zoom_in`, `view.zoom_out`, `view.next_page` and `view.prev_page` have
//! no arm in this file because **no such command is registered** — no catalog
//! entry, no manifest item, no `crate::text::commands` copy, no `RIBBON_IA.md`
//! row — so no token can reach one and an arm for it would be dead code
//! wearing a design pattern.
//!
//! All four verbs work, by the routes the specification puts them on:
//! `RIBBON_IA.md` §6 assigns *"zoom −/%/+, page ◀ n/N ▶"* to the **status
//! bar**, where `app::status`'s zoom group raises `ZoomIn`/`ZoomOut` and
//! `status::page_box` raises `NextPage`/`PrevPage`, and `app::keyboard` raises
//! all four from `Ctrl` `+`/`-` and `PageDown`/`PageUp`.
//!
//! ⇒ Registering them instead is a **ribbon** decision, not a dispatch one;
//! `shell::commands::reach::UNREACHED_ARMS` carries what it would take.

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
pub(crate) fn dispatch(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    match id {
        // ★ **The one arm here whose RETURN VALUE matters.**
        //
        // `ZoomOutcome` is `#[must_use]` precisely because its declining
        // variants are the point. Discarding it with a `let _ =` turns
        // "there is nothing to zoom to" into "the command did nothing",
        // which is the difference between a control that declines and one
        // that looks broken.
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
        // ★★ **Actual size is `ZoomTo(1.0)`, not `Fit(FitMode::None)`.**
        //
        // `Fit(FitMode::None)` only stops the per-frame re-fit and leaves
        // the zoom where it was, so it would pin whatever magnification
        // happened to be showing while this control promises one PDF point
        // per screen point.
        "view.zoom_actual" => actions.push(Action::ZoomTo(1.0)),
        "view.zoom_fit_page" => actions.push(Action::Fit(FitMode::Page)),
        "view.zoom_fit_width" => actions.push(Action::Fit(FitMode::Width)),
        "view.zoom_fit_height" => actions.push(Action::Fit(FitMode::Height)),
        // Unreachable: `handles` above is the guard that admitted this call,
        // and the two are one list. Spelled out rather than folded into a
        // catch-all so that an id added to `handles` and not to this match
        // fails loudly instead of silently doing nothing.
        // ui-text-exempt: a panic message, read from a stack trace.
        other => unreachable!("dispatch::zoom does not own {other}"),
    }
}
