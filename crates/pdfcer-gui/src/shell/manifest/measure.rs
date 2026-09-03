//! The **Measure** tab — *what am I measuring, and in what units?*
//!
//! `RIBBON_IA.md` §5.6. Two groups: Dimension, Scale.
//!
//! # This tab is small on purpose, and that is not the same as underfilled
//!
//! P2, from the salvage source and kept: **the ribbon picks the activity;
//! the sidebar holds that activity's controls.** The Measure tab arms
//! "Linear"; the group picker, scale entry, number format and drafting
//! standard live in the Tool Options pane. That is why this tab has few
//! controls, and:
//!
//! > The fix for an underfilled tab is never to move sidebar controls up
//! > into it.
//!
//! What the tab *is* short of is dimension **kinds**, and those are a
//! build, not a layout decision.
//!
//! # The group model is the thing not to dilute
//!
//! Named dimension groups, each carrying a shared scale, number format and
//! drafting standard, are — per `RIBBON_IA.md` §5.6 — genuinely better
//! than what the comparison product does. `measure.set_scale` sets the
//! *current group's* scale rather than a document-wide one, and
//! `measure.manage_groups` is where groups are created and inspected. Both
//! tooltips name the group explicitly, because an operator who thinks they
//! are setting a global scale will be surprised twice: once when a second
//! dimension reads differently, and once when they cannot find where it
//! was set.
//!
//! # What is absent, and one entry worth flagging
//!
//! `Angular`, and the whole **Quantity** (distance, perimeter, area,
//! count) and **Takeoff** (schedule panel, export CSV) groups, are **N**.
//! Area and Angular are the conspicuous absences for anyone doing takeoff
//! on a drawing.
//!
//! **`Two-line` is a C row and is at the top of the project's own queue.**
//! Core and CLI shipped and were measured; the gesture's entry point has
//! no caller. That makes it a shell-only task — the cheapest real command
//! on this list — and it is still absent here, because P3 is about what
//! the operator can reach, and an engine with no caller is not reachable.
//!
//! ★ **Both of those paragraphs are now out of date and are kept as
//! history**: `Two-line` shipped on 2026-08-14 and `Radius / diameter` was
//! armed the same day. The Dimension group therefore holds **four** items
//! rather than three, and the fourth is not a tool — `measure.finish` ends
//! the radius/diameter gesture, which is the only gesture on this tab with
//! no natural end. `RIBBON_IA.md` §5.6 does not name it, because the
//! problem it solves did not exist until the tool was armed; the reason it
//! is here rather than in a group of its own is at its own entry below.
//!
//! **`Aligned` is marked *partial G*** — the constraint exists inside the
//! linear tool, but there is no separate tool to arm. A partial that
//! cannot be armed is, from the ribbon's point of view, an **N**, so it is
//! in [`super::PLANNED`] with that reasoning recorded rather than emitted
//! as a button that would arm nothing.
//!
//! **`Calibrate from a known length` is also marked *partial G*** and is
//! treated the same way, and this is the least certain of the judgements
//! in this module: it is plausible that calibration is reachable today
//! through the scale entry rather than as its own command. If it is, the
//! fix is to move one line out of PLANNED and into the Scale group — which
//! is exactly what PLANNED exists to make findable.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Measure tab.
pub(super) fn tab() -> Tab {
    Tab::new("measure", ribbon::tab_measure())
        .with_question(ribbon::question_measure())
        .with_groups([
            // ---------------------------------------------------------------
            // Dimension — what kind of dimension the next gesture places.
            // ---------------------------------------------------------------
            group(
                "dimension",
                ribbon::group_measure_dimension(),
                [
                    command("measure.linear"),
                    command("measure.radius_diameter"),
                    command("measure.perimeter"),
                    command("measure.length"),
                    command("measure.two_line"),
                    // ★ **Finish** sits with the tools, not in its own group.
                    //
                    // It is not a fourth tool — it arms nothing — and a reader
                    // could reasonably expect it beside the thing it acts on
                    // rather than beside the things that arm. It is here
                    // because P2 says the ribbon picks the *activity*, and
                    // finishing a circle fit is part of the dimensioning
                    // activity: a group of its own for one command would be a
                    // caption and a divider spent on a control that is greyed
                    // whenever the operator is not mid-fit.
                    //
                    // It reads correctly in place, too. The group is now
                    // "which kind of dimension, and when this one is done",
                    // and the one command that is not a tool is the last in
                    // the row rather than lost among them.
                    command("measure.finish"),
                ],
            ),
            // ---------------------------------------------------------------
            // Scale — what those dimensions are read against.
            // ---------------------------------------------------------------
            group(
                "scale",
                ribbon::group_measure_scale(),
                [
                    command("measure.set_scale"),
                    command("measure.manage_groups"),
                ],
            ),
        ])
}
