//! # `app::dispatch::measure` — arming a measure tool, and the two windows
//! behind it
//!
//! ## Why this is a module
//!
//! **R2** put it here and a duplication kept it here. Three of the four
//! `measure.*` commands resolve the **active authoring group** out of
//! `egui::Memory`, with the same traced fallback for the case where no measure
//! tool has been entered this session — and that resolution was written twice
//! in `super`'s match before the move, each copy with its own trace line.
//!
//! Duplicating it was defended at the time on the grounds that *"the fallback's
//! trace line names its own command, and a shared helper would have to be told
//! which."* That was true and it was the wrong conclusion: a helper **taking
//! the command id** names it better than two copies do, because the two copies
//! can only agree by inspection.
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
//! ## The capability gate is one sentence, four times
//!
//! Every command here declines in a mode that cannot author a ce dimension, and
//! they must decline **alike**: a mode that cannot place a dimension has no
//! business calibrating the group they live in, creating one, or ending a fit.
//! Four different refusals for one capability would read as arbitrary.

use egui::Context;

use crate::app::PdfcerApp;
use crate::app::actions::Action;

/// The ids this module owns.
///
/// ★ A predicate rather than a `match` in `super`, so the routing arm cannot
/// drift from the arms it routes to. `measure_for_command` already answers for
/// the tool-arming ids; the three named ones are the commands that are **not**
/// tools, each for a reason its own arm records.
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
        // ★ **The measure tools — one arm for all four.**
        //
        // The same shape as the markup arm below, and it declines in a mode
        // that does not author dimensions for the same reasons — see there.
        // Read grants neither; **Review grants this one and not that one**,
        // which is why they are two capabilities rather than an "authoring"
        // flag.
        //
        // **It arms a tool; it authors nothing.** A ce dimension is placed
        // by clicks that `crate::canvas::measure` takes, and only the pick
        // that completes one raises an `Action`.
        // ★ **Finish** — the ribbon half of the radius/diameter tool's
        // ending, and the one `measure.*` command that is not a tool.
        //
        // It must sit ahead of the arm below rather than inside it:
        // `measure_for_command` maps ids to *kinds*, this id names no kind,
        // and if it ever did, pressing Finish would toggle the tool off
        // (`arm_measure`'s same-kind-retires rule) instead of committing.
        //
        // The arm routes and does not compute. Everything about what a
        // finish *is* — whether there is a fit, which page it belongs to,
        // which group it joins, emptying the pick set afterwards — lives in
        // `canvas::measure::finish`, which is the same function the
        // canvas's double-click ending reaches. One commit path, two
        // entrances; a second derivation here is exactly how the two
        // endings would come to author different dimensions.
        //
        // The capability check mirrors the tool arm below. It is
        // unreachable through the shipped manifest — Read is shown File and
        // View alone, and no chord binds a `measure.*` id — but a
        // customized manifest can bind a chord to anything, and a mode that
        // cannot author dimensions must not author one because the pick set
        // predates the mode change.
        //
        // Both refusals are traced, and they are traced separately: "the
        // mode says no" and "there was nothing to finish" are different
        // facts, and a reader of a trace from a machine they cannot see
        // should not have to guess which kind of nothing happened.
        // ★ Set scale — the arm `shell::commands::reach` called *"the
        // clearest statement of a missing arm in the crate"*.
        //
        // Gated on `author_measure` exactly as `measure.finish` is, and for
        // the same reason: a mode that cannot author a dimension has no
        // business recalibrating the group they live in, and the two
        // refusals must be the same sentence or the mode gate reads as
        // arbitrary.
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
            // ★ One resolution, taking the id — see the module header for
            // why two hand-written copies were the wrong answer even though
            // each traced correctly.
            let group = active_group(ctx, id);
            app.dialogs.open_scale(&app.status, group);
        }
        // ★ **Manage dimension groups.** Registered, drawn on Measure ▸ Scale
        // and inert for the whole life of this build until 2026-08-18 —
        // the operator's *"I still can't get to edit dimension groups when
        // I click on it."*
        //
        // Gated on `author_measure` for exactly the reason `measure.
        // set_scale` above is, and the two must stay the same sentence: a
        // mode that cannot author a ce dimension has no business creating
        // the groups they live in, or recalibrating one, and two different
        // refusals for one capability read as arbitrary.
        //
        // The group is resolved the same way too — the measure tool's
        // active authoring group, with the same traced fallback — so the
        // window opens on the group the operator is drawing into rather
        // than on whichever is first in the model. Duplicating the
        // resolution rather than factoring it out is deliberate at two
        // call sites: the fallback's trace line names its own command, and
        // a shared helper would have to be told which.
        "measure.manage_groups" => {
            if !app.capabilities().author_measure {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("command-declined id={id} reason=mode-cannot-author-measure")
                });
                return;
            }
            // ★ **A panel toggle since 2026-08-19, not a window.**
            //
            // It opened `dialogs::dimension_groups` until then, and the
            // operator's report is why it does not any more: *"the groups
            // editor popup is too long for some screens so can't close it"*.
            // A window whose content outgrows the screen can carry its own
            // title bar — and its only ✕ — off the desktop.
            // `crate::panels::dimension_groups`' header has the whole account.
            //
            // The authoring-group resolution that used to be here is **gone
            // rather than moved**, and that is the interesting half. It existed
            // so the window could open on the group the operator was drawing
            // into; a panel is not constructed when it is shown, so it reads
            // the authoring group itself, on every frame, and keeps following
            // it until a row is picked. A one-shot seed computed at open time
            // could not have done that.
            app.toggle_panel(crate::panels::Panel::DimensionGroups);
        }
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
/// `id` is in the trace line because that is what the two hand-written copies
/// of this used to get right — and the only thing they got right that a shared
/// helper had to keep.
fn active_group(ctx: &Context, id: &str) -> pdfcer_core::dimension::GroupId {
    crate::canvas::measure::active_group(ctx).unwrap_or_else(|| {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("measure-group-fallback id={id} reason=no-measure-state")
        });
        pdfcer_core::dimension::DEFAULT_GROUP_ID
    })
}
