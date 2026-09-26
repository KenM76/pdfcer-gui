//! The **Tools** tab — *what do I run across files, or configure once?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/manifest/tools.md`.

use super::{command, group, large};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Tools tab.
pub(super) fn tab() -> Tab {
    Tab::new("tools", ribbon::tab_tools())
        .with_question(ribbon::question_tools())
        .with_groups([
            // ---------------------------------------------------------------
            // Batch — jobs that produce new files. `Batch print…` is **N**.
            // ---------------------------------------------------------------
            group(
                "batch",
                ribbon::group_tools_batch(),
                // One item, and that is correct rather than unfinished.
                // `tools.split_files` is unregistered until the boundary
                // chooser exists (R9, O68); merge is wired and stays.
                //
                // The band is NOT deleted for being short. The rule that
                // deletes a band is emptiness — an empty captioned band is a
                // caption offering nothing — and a band of one is a band of
                // one.
                //
                // Large, per the mockup's big `Merge files…`, and free here
                // because a one-item group is the case the `large` helper
                // always allowed.
                [large("tools.merge_files")],
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
                [large("tools.render_diagnostics")],
            ),
        ])
}
