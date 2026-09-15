//! The **Tools** tab — *what do I run across files, or configure once?*
//!
//! `RIBBON_IA.md` §5.7. Three groups: Batch, Fonts, Diagnostics.
//!
//! # The rule that decides membership
//!
//! A control belongs on Tools when it either operates on files **other than
//! the open one**, or is configured once and rarely touched. That is what
//! keeps the tab from becoming the place leftovers go.
//!
//! Two consequences worth stating, because both look like omissions:
//!
//! 1. **Redact is not here**; it is Edit ▸ Protect, where someone editing a
//!    document looks for it.
//! 2. **The batch pane's jobs are also ribbon commands.** The pane stays —
//!    the ribbon is the path that can be found without knowing the pane is
//!    there. Its third job, inserting pages from another file, is on Pages
//!    instead, because that one changes *this* document.
//!
//! # The Pages/Tools line, restated because it is the one that gets blurred
//!
//! `pages.split` and `tools.split_files` are two commands, not one command
//! twice. So are `pages.merge_into` and `tools.merge_files`. The
//! distinction is which document changes:
//!
//! - **Pages** — this document's page set changes. Undoable. Respects the
//!   thumbnail rail's selection.
//! - **Tools** — new files are produced. This document is untouched. The
//!   inputs are chosen from disk.
//!
//! Both tooltips point at the other, so an operator who reached for the
//! wrong one is told where the right one lives rather than getting a
//! dialog that asks the wrong question.
//!
//! # Render diagnostics belongs here rather than in the status bar
//!
//! The status bar is for the controls a user touches constantly, and a
//! diagnostic readout is neither a control nor constant — it is a thing you go
//! and look at when something is wrong. A tab also gives it room to be more
//! than one line.
//!
//! # Recognise is NOT on this tab
//!
//! `RIBBON_IA.md` §5.7 specifies **Tools ▸ Recognise ▸ OCR…**. That placement
//! is unbuildable; the command is `file.ocr`, on **File ▸ Recognise**, and the
//! reason is the operator's:
//!
//! > *"if in read mode ocr should still be available, but it will prompt to
//! > save changes as save as instead of save."*
//!
//! **Read's tab list is `["file", "view"]`.** A command on the Tools tab is
//! therefore not merely inconvenient in Read, it is *unreachable* — no tab, no
//! band, no control, and `modes::capability::offers_command` would refuse a
//! chord for it too. Shipping OCR here would satisfy the specification and
//! break the instruction.
//!
//! ### The rule this is an instance of
//!
//! **A chord refused in a mode where the operator plainly needs it is evidence
//! that the command's tab is wrong, not that the gate needs an exception.** The
//! remedy is a tab move, not a capability exemption, and it needs no new
//! machinery.
//!
//! ### Why FILE, and not View
//!
//! Read is shown two tabs and only one of them can be right. View answers
//! *"what is on my screen, and how is the page laid out?"* — OCR changes
//! neither; the page renders pixel-identically afterwards, which is the whole
//! point of the mode-3 sandwich. File answers *"what do I do with the file as a
//! whole?"*, and OCR's product is **a new file**. That is the same sentence
//! that moved the two text-copy commands onto File ▸ Export, and it is the
//! operator's own general rule for Read: *Read may produce a new document; it
//! may not modify this one.*
//!
//! ### What is left for the operator to rule on
//!
//! This departs from a written specification, so it is flagged rather than
//! quietly done. If §5.7's Tools placement is preferred, the instruction it
//! collides with has to be settled instead — either Read gains the Tools tab
//! (which would also hand it batch merge, split and font embedding, all of
//! which genuinely author) or OCR is not available in Read after all.
//!
//! # What is absent
//!
//! `Batch print…`, the whole **Compare** group, and the whole **Validate**
//! group (PDF/A validate & convert, Optimise) are **N**. Compare is the one
//! absence an AEC reviewer will name first, and it is a large build; it is an
//! open question in `RIBBON_IA.md` §8 rather than a scheduled item, and
//! [`super::PLANNED`] records it as such.

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
