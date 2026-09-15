//! # `app::dispatch::measure` — arming a measure tool, and the two windows
//! behind it
//!
//! ## Why this is a module
//!
//! Every `measure.*` command is here, and they share two rules that must be
//! stated once: the capability they all gate on, and the way an authoring
//! group is resolved. Split across `super`'s match they were restated per arm,
//! and restated rules agree only by inspection.
//!
//! ★ [`active_group`] takes the **command id** so its trace line names the
//! command that fell back. That is the one thing a per-arm copy of the
//! resolution got right, and it is why the shared helper is not argument-free.
//!
//! ## ★ What the fallback is for, and why it is traced rather than silent
//!
//! `canvas::measure::active_group` returns `None` when the measure tool has
//! never been armed this session — there is no state in `egui::Memory` and no
//! group has been chosen. Substituting the default group is the **right**
//! answer for an operator who has drawn nothing, and the **wrong** one for
//! anybody whose state was somehow lost.
//!
//! Both look identical afterwards — *"a group got a scale"* — so the fallback
//! says so on the trace rather than being silent. That is the whole reason a
//! `None` is not quietly turned into a default at the source.
//!
//! ## The capability gate is one sentence, repeated per arm
//!
//! Every command here declines in a mode that cannot author a ce dimension, and
//! they must decline **alike**: a mode that cannot place a dimension has no
//! business calibrating the group they live in, creating one, or ending a fit.
//! Differing refusals for one capability read as arbitrary, so every arm below
//! traces `reason=mode-cannot-author-measure` and nothing else.

use egui::Context;

use crate::app::PdfcerApp;
use crate::app::actions::Action;

/// The ids this module owns.
///
/// ★ A predicate rather than a `match` in `super`, so the routing arm cannot
/// drift from the arms it routes to. `measure_for_command` already answers for
/// the tool-arming ids; the named ones are the `measure.*` commands that are
/// **not** tools, each for a reason its own arm records.
#[must_use]
pub(super) fn handles(id: &str) -> bool {
    matches!(
        id,
        "measure.set_scale" | "measure.manage_groups" | "measure.finish"
    ) || crate::shell::commands::measure_for_command(id).is_some()
}

/// Dispatch one `measure.*` command.
pub(super) fn dispatch(app: &mut PdfcerApp, ctx: &Context, id: &str, actions: &mut Vec<Action>) {
    match id {
        // ★ **Set scale — recalibrate the group a dimension is measured in.**
        //
        // Gated on `author_measure` exactly as every arm here is: a mode that
        // cannot author a dimension has no business recalibrating the group
        // they live in.
        //
        // # ★ Which group, and why the fallback is traced
        //
        // The measure tool's active authoring group, when the tool has been
        // entered this session. When it has not, there is no state in
        // memory and the arm falls back to the default group — which is the
        // right answer for an operator who has drawn nothing yet, and the
        // WRONG one for anybody whose state was somehow lost. Both would
        // look identical afterwards ("a group got a scale"), so the
        // fallback says so in the trace rather than being silent.
        "measure.set_scale" => {
            if !app.capabilities().author_measure {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("command-declined id={id} reason=mode-cannot-author-measure")
                });
                return;
            }
            // ★ One resolution for every arm that needs a group, taking the
            // id so the fallback trace still names the command.
            let group = active_group(ctx, id);
            app.dialogs.open_scale(&app.status, group);
        }
        // ★ **Manage dimension groups**, on the operator's report: *"I still
        // can't get to edit dimension groups when I click on it."*
        //
        // Gated on `author_measure` for exactly the reason `measure.set_scale`
        // above is, and the two must stay the same sentence: a mode that
        // cannot author a ce dimension has no business creating the groups
        // they live in, or recalibrating one.
        "measure.manage_groups" => {
            if !app.capabilities().author_measure {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("command-declined id={id} reason=mode-cannot-author-measure")
                });
                return;
            }
            // ★ **A panel toggle, not a window**, on the operator's report:
            // *"the groups editor popup is too long for some screens so can't
            // close it"*. A window whose content outgrows the screen can carry
            // its own title bar — and its only ✕ — off the desktop.
            // `crate::panels::dimension_groups`' header has the whole account.
            //
            // ★ **No authoring group is resolved here**, unlike
            // `measure.set_scale` above, and the asymmetry is the point: a
            // panel is not constructed when it is shown, so it reads the
            // authoring group itself on every frame and keeps following it
            // until a row is picked. A one-shot seed computed at open time
            // would freeze the panel on whichever group was active when the
            // operator pressed the button.
            app.toggle_panel(crate::panels::Panel::DimensionGroups);
        }
        // ★ **Finish** — the ribbon half of the radius/diameter tool's ending.
        //
        // It must sit ahead of the tool arm below rather than inside it:
        // `measure_for_command` maps ids to *kinds*, this id names no kind, and
        // if it ever did, pressing Finish would toggle the tool off
        // (`arm_measure`'s same-kind-retires rule) instead of committing.
        //
        // The arm routes and does not compute. Everything about what a finish
        // *is* — whether there is a fit, which page it belongs to, which group
        // it joins, emptying the pick set afterwards — lives in
        // `canvas::measure::finish`, which is the same function the canvas's
        // double-click ending reaches. One commit path, two entrances; a second
        // derivation here is how the two endings would come to author different
        // dimensions.
        //
        // The capability check is not dead code even though the shipped
        // manifest cannot reach it — a customized manifest can bind a chord to
        // anything, and a mode that cannot author dimensions must not author
        // one just because the pick set predates the mode change.
        //
        // Both refusals are traced, and traced separately: "the mode says no"
        // and "there was nothing to finish" are different facts, and a reader
        // of a trace from a machine they cannot see should not have to guess
        // which kind of nothing happened.
        "measure.finish" => {
            if !app.capabilities().author_measure {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("command-declined id={id} reason=mode-cannot-author-measure")
                });
            } else if !crate::canvas::measure::finish(ctx, actions) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // Reachable only by a chord or a customized manifest:
                    // the ribbon control is greyed unless there is a
                    // non-degenerate fit, by the same predicate `finish`
                    // itself asks.
                    format!("command-declined id={id} reason=no-circle-fit-to-finish")
                });
            }
        }
        // ★ **The measure tools — one arm for every kind in
        // `MeasureKind::ALL`.** The id IS the kind, so one arm and one mapping
        // cannot come to disagree with a run of hand-written arms.
        //
        // **It arms a tool; it authors nothing.** A ce dimension is placed by
        // clicks that `crate::canvas::measure` takes, and only the pick that
        // completes one raises an `Action`.
        //
        // ★ `author_measure` and not `author_markup`. They are two flags
        // because `Capabilities::for_mode` derives each from whether the mode
        // is shown its own ribbon tab, so a manifest can offer markup without
        // dimensions or the reverse. One "authoring" flag would make that
        // undeclarable.
        id if crate::shell::commands::measure_for_command(id).is_some() => {
            if !app.capabilities().author_measure {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "command-declined id={id} reason=mode-cannot-author-measure"
                    )
                });
            } else if let Some(kind) = crate::shell::commands::measure_for_command(id) {
                let _ = crate::canvas::tool::arm_measure(ctx, kind);
            }
        }
        // Unreachable: `handles` is the predicate `super` routed on, and it
        // matches exactly the arms above. Spelled rather than `unreachable!()`
        // because a panic in a dispatcher takes the window with it, and a
        // traced no-op is the same information at none of the cost.
        other => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("measure-dispatch-unrouted id={other}")
        }),
    }
}

/// The group a measure command acts on, with the fallback traced.
///
/// See the module header: `None` means the measure tool has never been armed
/// this session, and substituting the default group is right for an operator
/// who has drawn nothing and wrong for anybody whose state was lost. The two
/// are indistinguishable afterwards, so the substitution says so.
///
/// `id` is in the trace line so a reader can tell which command fell back.
/// Without it the trace says a substitution happened and not what asked for it.
fn active_group(ctx: &Context, id: &str) -> pdfcer_core::dimension::GroupId {
    crate::canvas::measure::active_group(ctx).unwrap_or_else(|| {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("measure-group-fallback id={id} reason=no-measure-state")
        });
        pdfcer_core::dimension::DEFAULT_GROUP_ID
    })
}
