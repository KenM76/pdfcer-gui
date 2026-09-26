//! # `shell::commands::catalog::modes` — the mode selector — Read, Review, Edit
//!
//!
//! ## The split is per TAB, and the reason it was refused before is gone
//!
//! [`super`]'s header argued against exactly this cut:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog/modes.md`.

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This band's commands, in ribbon order.
pub(super) fn band() -> Vec<Command> {
    vec![
        //
        // Not ribbon commands: the three positions of the selector, bound
        // to Ctrl+1/2/3. Always available — a mode is an interface-
        // complexity control, not a permission, and there is no document
        // state in which changing your own view stance should be refused.
        //
        // **No icons, and this is the one entry in the whole "which
        // commands get a glyph" question that is settled by the renderer
        // rather than by taste.** `egui_shell::ribbon::mode_selector` draws
        // the modes as **text segments** of an N-position segmented control,
        // taking each one's `Mode::label` from the manifest — it never looks
        // at a `Command`, and the module contains no icon path at all (the
        // string `icon` does not occur in the file). `MODES_AND_PANELS.md`
        // Part 1 is why: the control must render "as a real segmented control
        // with all three labels visible — not a bare track with a knob, where
        // the available positions are invisible until you drag."
        //
        // So a key here would resolve to art nothing draws. Worse, it would
        // look like a wiring bug to the next reader — a command that names a
        // glyph and never shows one — which is the failure mode the visible
        // slashed mark exists to make loud, arriving in the one place the
        // mark cannot appear.
        // ===================================================================
        command("mode.read", t::mode_read(), 900),
        command("mode.review", t::mode_review(), 901),
        command("mode.edit", t::mode_edit(), 902),
    ]
}
