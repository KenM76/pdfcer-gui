//! The **Tools** tab — *what do I run across files, or configure once?*
//!
//! `RIBBON_IA.md` §5.7. Three groups: Batch, Fonts, Diagnostics.
//!
//! # What this tab became
//!
//! In the salvage source it was three groups of one control each — a batch
//! *panel* toggle, font folders, and redaction — and on a 1936 px window
//! that left well over a thousand pixels of empty band. The reorganisation
//! does two things:
//!
//! 1. **Redact leaves**, to Edit ▸ Protect, where a user editing a
//!    document will actually look for it.
//! 2. **The batch panel's contents surface as commands.** `Merge files…`
//!    and `Split files…` were reachable only by opening a pane and then
//!    choosing within it; the pane stays, and the ribbon becomes the path
//!    that can be found without knowing it is there. The pane's third
//!    job — inserting pages from another file — is not here, because
//!    that one changes *this* document and belongs on Pages.
//!
//! The result is a tab defined by a rule rather than by leftovers: things
//! that either operate on files **other than the open one**, or are
//! configured once and rarely touched.
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
//! It is currently a run of text in the status bar. That surface is for
//! the controls a user touches constantly, and a diagnostic readout is
//! neither a control nor constant — it is a thing you go and look at when
//! something is wrong. Moving it here also gives it room to be more than
//! one line.
//!
//! # ★★ Recognise shipped 2026-08-14, and it is NOT on this tab
//!
//! `RIBBON_IA.md` §5.7 specifies **Tools ▸ Recognise ▸ OCR…**, and this file
//! carried it for about an hour before the placement was found to be
//! unbuildable. The command is `file.ocr`, on **File ▸ Recognise**, and the
//! reason is the operator's:
//!
//! > *"if in read mode ocr should still be available, but it will prompt to
//! > save changes as save as instead of save."*
//!
//! **Read's tab list is `["file", "view"]`.** A command on the Tools tab is
//! therefore not merely inconvenient in Read, it is *unreachable* — no tab, no
//! band, no control, and `modes::capability::offers_command` would refuse a
//! chord for it too. Shipping OCR here would have satisfied the specification
//! and broken the instruction.
//!
//! ### This is the THIRD time, and the fix is the same one both previous times
//!
//! `HANDOFF.md` states the pattern outright: *"a chord refused in a mode where
//! the operator plainly needs it is evidence that the command's tab is wrong,
//! not that the gate needs an exception. That is twice this has happened and
//! twice the fix has been a tab move."* It was `edit.form_fill` →
//! `view.panel_forms`, then `edit.copy_page_text` → `file.copy_page_text`.
//! This is the third, and it needed no new machinery because the pattern was
//! already written down.
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
//! ### ★ What is left for the operator to rule on
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
                // ★★★ **`tools.split_files` LEFT this band on 2026-08-31** —
                // O68. Merge is wired and stays; Split is unregistered until
                // the boundary chooser exists (R9), so the band is one item.
                //
                // ★ The band is NOT deleted, unlike Edit ▸ Clipboard and
                // View ▸ Render before it, because it is not empty: it has a
                // live member. An empty captioned band is a caption offering
                // nothing; a band of one is a band of one.
                [command("tools.merge_files")],
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
            // Note that the Fonts *panel* is not here — it moved to File ▸
            // Document, because it describes what is inside the file.
            // These are the two verbs; that is the inventory.
            // ---------------------------------------------------------------
            group(
                "fonts",
                ribbon::group_tools_fonts(),
                [
                    command("tools.font_folders"),
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
