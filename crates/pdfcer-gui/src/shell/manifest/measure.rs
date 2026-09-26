//! The **Measure** tab — *what am I measuring, and in what units?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/manifest/measure.md`.

use super::{command, group, large};
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
                    // **Finish** sits with the tools, not in its own group.
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
                [large("measure.set_scale"), large("measure.manage_groups")],
            ),
        ])
}
