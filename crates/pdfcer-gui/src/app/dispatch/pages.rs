//! # `app::dispatch::pages` — the Pages tab's command arms
//!
//! Split out of [`super`] under **R2** on 2026-08-18, when
//! `pages.insert_from_file` took that file past 1,500 lines.
//!
//! ## The seam
//!
//! `super`'s subject is *"a command id becomes an intent"* across the whole
//! ribbon. This file's is the **Pages tab's** share of it, and the two change
//! for different reasons: a new tab or a new dispatch convention touches the
//! parent, a new page verb touches this.
//!
//! It also mirrors a split that already exists one layer down —
//! `actions::pages` holds the page verbs' *bodies* for the same reason — so a
//! reader following `pages.rotate_left` from the ribbon to the document now
//! meets one named file at each layer instead of two general ones.
//!
//! ## ★ What did NOT move, and why
//!
//! `page_operands` stays on `PdfcerApp` in [`super`]. It is the **operand
//! rule** — *which pages does a Pages command act on?* — and it is read by
//! arms on both sides of this split. Moving it here would put a shared rule
//! inside one of its consumers, which is the shape this project's own
//! `SelectionState::deletable_objects_on` comment refuses: *"a rule stated in
//! two places is a rule that drifts"*, and the drift here is destructive.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::state::Status;

/// Whether this file owns `id`.
///
/// `pub(crate)` rather than `pub(super)`: `shell::commands::reach`'s
/// `guard_claiming` calls it, because the reachability checker must be able to
/// EVALUATE every guard arm it finds — a guard it cannot evaluate is a place
/// commands could hide from the check that exists to find them.
///
/// A separate predicate rather than a `match` that returns `bool`, because the
/// caller is a guard on a match arm and the two must not be able to disagree
/// about the set: a command listed here and missing below would fall into this
/// arm and silently do nothing, which is the *"visible control, silently
/// inert"* failure this crate keeps finding.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    matches!(
        id,
        "pages.rotate_left"
            | "pages.rotate_right"
            | "pages.delete"
            | "pages.extract"
            | "pages.move_up"
            | "pages.move_down"
            | "pages.resize"
    )
}

/// Turn one Pages command into an intent.
///
/// `id` is guaranteed to be one [`handles`] claims — the caller's arm is
/// guarded on it — so the fall-through below is unreachable and says so rather
/// than guessing.
/// ★ **`&mut` since `pages.resize` landed**, and the widening is deliberate
/// rather than incidental. Every other arm here builds an `Action` and pushes
/// it; that one opens a window, which lives on `PdfcerApp::dialogs`. The
/// alternative was to put the arm in [`super`] beside `pages.merge_into` and
/// `pages.insert_from_file`, which are in that file for exactly this reason —
/// and [`super`] is at **1,500 of its 1,500 lines**, so there is no room to.
/// Widening the receiver costs the call site nothing (it is already inside a
/// `&mut self` method and reborrows) and keeps every Pages arm in the file
/// named after the Pages tab.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    match id {
        // ★★★ **Change the paper the picked sheets sit on.**
        //
        // The only arm here that opens a window rather than pushing an
        // `Action`, and it has to: the operator is being asked TWO questions,
        // and the second one is not a choice — it is whether he has understood
        // that a smaller sheet CROPS his drawing rather than shrinking it. See
        // `crate::dialogs::page_size`, whose header carries the measurement.
        //
        // ★ The operand list is resolved by the same `page_operands` every
        // other arm calls, and it is resolved BEFORE `app.dialogs` is borrowed
        // mutably — `page_operands` takes `&self` and returns an owned `Vec`,
        // so that borrow is over by the time the window is built. `status` and
        // `dialogs` are then two disjoint fields, which is the one shape the
        // borrow checker allows here and the reason this is written as field
        // access rather than as a method on `PdfcerApp`.
        "pages.resize" => {
            let Some(pages) = app.page_operands() else {
                return;
            };
            let Status::Open(doc) = &app.status else {
                // Unreachable: `page_operands` already returned `None` for a
                // closed document and traced why. Spelled out rather than
                // `unreachable!()`, because a panic-free binary must not depend
                // on another function's early return staying where it is.
                return;
            };
            app.dialogs.open_page_size(doc, &pages);
        }
        // ===============================================================
        // ★★ THE PAGE VERBS — six commands, one operand rule, five arms
        //
        // Every one of these was registered, drawn on the Pages tab, listed
        // in the page tile's context menu and — for four of them — bound to
        // a chord (`[`, `]`, `Alt+Up`, `Alt+Down`), with **no arm at all**.
        // Every press traced `command-unimplemented`, which is defect D1's
        // shape six times over, and `FEATURES.md` claimed the panel shipped
        // *"a context menu of the six page verbs"*.
        //
        // # The operand is the panel's multi-select, asked for once
        //
        // `crate::panels::pages::ops::operands` is the single place that
        // rule is written down — the picked sheets when there are any, the
        // current page when there are none — and every arm below calls it.
        // That is `SelectionState::deletable_objects_on`'s precedent, and
        // it exists for the same reason: *two statements of a destructive
        // rule is one too many*.
        //
        // ★ Note which selection is NOT read here.
        // `crate::panels::PanelsState::selected_pages` is the **page**
        // selection; `selection.any` and `doc.selection` are the **object**
        // one. `crate::panels::pages::select`'s header carries the table of
        // how they differ, and the consequence at this site is that no
        // `pages.*` command is gated on `selection.any` and none should be:
        // with nothing picked these act on the current page, which is a
        // defined answer and not a disabled state. **No new `enabled_when`
        // condition was needed**, so §5's fifth obligation does not apply.
        //
        // # The arms route; they do not compute
        //
        // Each builds one `Action` from one pure function and pushes it.
        // The permutation arithmetic, the edge refusals and the operand
        // fallback are all in `ops`, under unit test, because index
        // arithmetic that looks obviously right and is off by one at the
        // boundary is exactly what a `match` limb cannot be reviewed for.
        // ===============================================================
        "pages.rotate_left" | "pages.rotate_right" => {
            // The id IS the operand, exactly as it is for the page-display
            // radio and the markup tools: one arm, one mapping, rather than
            // two arms that can come to disagree about which way round the
            // signs go. `-90` is anticlockwise, which is what `rotate-ccw`
            // draws and what `crate::text::commands::pages_rotate_left`
            // promises.
            let delta = if id == "pages.rotate_left" { -90 } else { 90 };
            if let Some(pages) = app.page_operands() {
                actions.push(Action::Page(PageAction::RotatePages { pages, delta }));
            }
        }
        "pages.delete" => {
            if let Some(pages) = app.page_operands() {
                actions.push(Action::Page(PageAction::DeletePages { pages }));
            }
        }
        "pages.extract" => {
            if let Some(pages) = app.page_operands() {
                actions.push(Action::Page(PageAction::ExtractPages { pages }));
            }
        }
        // ★ **The two move verbs, and the one arm in this family that can
        // decline.**
        //
        // `move_order` returns a permutation or a refusal, and the refusal
        // is deliberate rather than an omission: `EditSession::reorder_pages`
        // would *accept* the identity permutation and return `Ok(())`
        // having recorded nothing, so handing it one would produce a
        // control the operator pressed that changed nothing, said nothing
        // and bumped no epoch. That is the defect class this project is
        // named after, so the engine is never asked a question whose answer
        // is "nothing".
        //
        // ★ **It is traced and not worded, and that is a scope statement
        // rather than a judgement that it should not be.** The surface for
        // a worded decline is `crate::app::status::decline`, which was
        // being rewritten by the concurrent undo/redo work while this
        // landed; adding two variants to `Declined` mid-rewrite is how two
        // sessions produce one broken file. The two refusals carry
        // *distinct* reason tokens so the follow-up is a mapping rather
        // than an investigation: `at-the-edge` wants a sentence naming the
        // boundary, `nothing-to-move` one naming the document.
        //
        // The two are traced separately because they are different facts
        // with different remedies — pick a different sheet, or there is
        // nothing to be done — and a reader of a trace from a machine they
        // cannot see should not have to guess which nothing happened. That
        // is the same rule `measure.finish` and `markup.finish` follow four
        // arms above.
        "pages.move_up" | "pages.move_down" => {
            use crate::panels::pages::ops::{MoveDirection, move_order};
            let direction = if id == "pages.move_up" {
                MoveDirection::Up
            } else {
                MoveDirection::Down
            };
            let Some(pages) = app.page_operands() else {
                return;
            };
            let count = match &app.status {
                Status::Open(doc) => doc.pages.len(),
                // Unreachable: `page_operands` returned `Some`, which it
                // only does for an open document with pages. Answered
                // rather than unwrapped because a panic in a dispatch arm
                // takes the operator's document with it.
                _ => 0,
            };
            match move_order(&pages, count, direction) {
                Ok(order) => actions.push(Action::Page(PageAction::ReorderPages { order })),
                Err(reason) => crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "command-declined id={id} reason={} n={}",
                        match reason {
                            crate::panels::pages::ops::MoveRefusal::AtTheEdge => "at-the-edge",
                            crate::panels::pages::ops::MoveRefusal::NothingToMove =>
                                "nothing-to-move",
                        },
                        pages.len()
                    )
                }),
            }
        }
        // ui-text-exempt: a panic message, read from a stack trace by whoever
        // added an id to `handles` and not to this match. Never rendered.
        other => unreachable!("`handles` claimed {other} and this match does not"),
    }
}
