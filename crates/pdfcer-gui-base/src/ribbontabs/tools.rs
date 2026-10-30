//! The **Tools** tab — *what do I run across files, or configure once?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/tools.md`.

use super::{command, group, large};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Tools tab.
pub fn tab() -> Tab {
    Tab::new("tools", ribbon::tab_tools())
        .with_icon("tool")
        .with_question(ribbon::question_tools())
        .with_groups([
            // ---------------------------------------------------------------
            // Batch — jobs that run across files chosen on disk.
            // ---------------------------------------------------------------
            group(
                "batch",
                ribbon::group_tools_batch(),
                // Commands over files on disk, not the open document.
                // `tools.split_files` is unregistered until the split window
                // takes files from disk (R9, O68).
                [
                    large("tools.merge_files"),
                    large("tools.bates_files"),
                    large("tools.batch_print"),
                ],
            ),
            // ---------------------------------------------------------------
            // Compare — the open document against an older version of it.
            // ---------------------------------------------------------------
            group(
                "compare",
                ribbon::group_tools_compare(),
                [large("tools.compare")],
            ),
            // ---------------------------------------------------------------
            // Fonts — configured once, rarely touched.
            //
            // Font folders is a session-scoped setting (the folders are
            // remembered for the session only), and embed/unembed act on
            // the open document. They share a band because they are the
            // same subject from the operator's side: what happens when a
            // document needs a typeface.
            //
            // The Fonts *panel* is not here — it is on File ▸ Document,
            // because it describes what is inside the file. These are the
            // two verbs; that is the inventory.
            // ---------------------------------------------------------------
            group(
                "fonts",
                ribbon::group_tools_fonts(),
                [
                    // Large — the mockup's `Font folders…` big, with the two
                    // embed verbs in a column beside it, and first in the
                    // group as the `large` helper requires.
                    large("tools.font_folders"),
                    command("tools.embed_fonts"),
                    command("tools.unembed_fonts"),
                ],
            ),
            // ---------------------------------------------------------------
            // Diagnostics.
            // ---------------------------------------------------------------
            group(
                "diagnostics",
                ribbon::group_tools_diagnostics(),
                [large("tools.render_diagnostics"), large("tools.ink_picker")],
            ),
        ])
}
