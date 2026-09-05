//! The **File** tab — *what do I do with the file as a whole, or with
//! pdfcer itself?*
//!
//! `RIBBON_IA.md` §5.1. Six groups: File, Save, Export, Print, Document,
//! pdfcer.
//!
//! ★ **`New` landed 2026-08-14** — see the File group below, and
//! `crate::app::blank` for where a blank document comes from when the engine
//! has no way to make one and has declared that it never will.
//!
//! # What this tab stopped being
//!
//! The salvage source's File tab was, in its own document's words, *"a
//! junk drawer"*: Properties, Copy this page's text, Copy the whole
//! document's text, Export DXF, Print, Reset layout, Settings, keyboard
//! shortcuts. Two of those are content operations and one is a view
//! operation. Meanwhile it had no New, no Recent, no Close — and no Open
//! and no Save, because those lived only on the quick-access toolbar and
//! the old reading of the one-command-one-tab rule forbade a tab from
//! mirroring them.
//!
//! It also had no **Recent**, which is the absence an operator meets on the
//! second document rather than the first: with no Open command at all, the
//! only way to look at a file was to start the process with it on the command
//! line. Both are here now — `Open…` as a command, `Recent ⌄` as the gallery
//! §5.1 specifies (see the group below for why that is an `Item::Custom`).
//!
//! Three things therefore happen here:
//!
//! 1. **`Open…` and `Save a copy…` appear on a tab**, under amendment P1a
//!    — *the QAT and the status bar are shortcut surfaces, not tabs; a
//!    command may appear on exactly one tab and additionally on the QAT.*
//!    A user who wants to open a file looks under File, and finding
//!    nothing there teaches them the ribbon is not where commands live.
//! 2. **Copy page text / copy document text leave**, to Edit ▸ Clipboard.
//!    Copying text out of a document is a content operation.
//!
//!    ★ **Reversed on 2026-08-14, by operator decision, and the reversal is
//!    recorded here rather than replacing the sentence above** — "this left
//!    and came back, for a better-stated reason" is a more useful fact than
//!    either half alone, and the returning reader who remembers the move
//!    needs to find it. They are now `file.copy_page_text` and
//!    `file.copy_document_text`, in **File ▸ Export**, and the Edit ▸
//!    Clipboard group that held them is deleted. The original argument was
//!    right that copying is a *content* operation and wrong to conclude that
//!    a content operation is an *authoring* one: copying reads the page and
//!    writes to the clipboard, changing nothing. See the Export group below
//!    for the full reasoning and for why they are in Export rather than in a
//!    Clipboard band of their own.
//! 3. **Reset layout leaves**, to View ▸ Window. It resets panel geometry.
//!
//! And one thing arrives: **Fonts**, from View ▸ Panels. The Fonts panel
//! answers *"what is inside this file"*, not *"what is on my screen"*, so
//! it sits with Properties as document-level inspection. `RIBBON_IA.md`
//! §5.1 flags this as a real improvement on the current build rather than
//! a re-parenting — the panel is good and nobody was going to find it
//! under View.
//!
//! # Why the Save group holds one command
//!
//! `Save` — the one that overwrites in place — cannot ship before autosave
//! and crash recovery exist; that dependency predates this document. Under
//! P3 it is therefore **absent**, not greyed with an explanatory tooltip,
//! and `Save a copy…` stands alone in the band. `Revert` is absent for the
//! same reason: it is meaningless without a save point to revert to.

use super::{command, group, large};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The File tab.
pub(super) fn tab() -> Tab {
    Tab::new("file", ribbon::tab_file())
        .with_question(ribbon::question_file())
        .with_groups([
            // ---------------------------------------------------------------
            // File — getting a document in and out of the application.
            //
            // ★ **`New` shipped on 2026-08-14** and its `PLANNED` entry ("N —
            // a blank or from-template document. pdfcer has no
            // document-creation path at all.") is retired. The second half of
            // that sentence is still true of the engine and always will be —
            // `pdfcer-core`'s `document.rs:10-19` names it an invariant — so
            // New does not create a document, it **opens one**: a 443-byte
            // blank-A4 template that ships as an asset and goes through
            // `Document::from_bytes`. `crate::app::blank` carries the whole
            // argument, including why filing a feature request for
            // `Document::blank(…)` would have been asking the engine to break
            // its own named invariant.
            //
            // It is **first** in the band. All three reference applications
            // open their File menu with New and follow it with Open, and the
            // order is the useful one as well as the conventional one: the two
            // ways to bring a document into the window, then the way to put
            // one away.
            //
            // What is here is now exactly what a first-time user looks for:
            // make a document, open a document, reopen one they had, put it
            // away.
            //
            // §5.1 specifies this row as "New (blank / from template)".
            // ★ **BOTH halves ship as of 2026-08-18.** The blank half is
            // `file.new`; the from-template half is `file.new_from_template`,
            // which is where the page-size choice lives, and the split is
            // Inkscape's own: `Ctrl+N` makes a document, `Ctrl+Alt+N` chooses
            // what kind. The second one was in `PLANNED` until
            // `EditSession::set_media_box` shipped — see
            // `crate::app::blank`'s §3a for what it was blocked on and why the
            // ten-assets alternative was refused rather than built.
            //
            // ★ `Recent ⌄` is an `Item::Custom`, not a command item, and that
            // is the only structural oddity on this tab.
            //
            // §5.1 specifies it as `Recent ⌄` — a *gallery*, not a button —
            // and a `Command` item can only ever render as a button. The
            // command behind it (`file.recent`) is registered and does the
            // opening; the item is what asks WHICH of the ten documents, and
            // `crate::app::recent::menu` draws it through the renderer
            // `egui_shell::ribbon::Ribbon::with_custom_items` exists for.
            // `super::CUSTOM_BACKED` records the arrangement, so the command
            // is not mistaken for an orphan by the reachability check that
            // walks command ids.
            //
            // It sits BETWEEN Open and Close, which is the specified order and
            // also the useful one: the two ways to get a document in, then the
            // way to put one away.
            // ---------------------------------------------------------------
            group(
                "file",
                ribbon::group_file_file(),
                [
                    // ★ Large, 2026-09-04 — `mockups/pdfcer-shell.html`
                    // draws `New` and `Open…` as the File group's two big
                    // controls and wraps the rest into one column beside
                    // them. They are already the leading run of this group,
                    // so `sizing`'s hoist is a no-op and the rendered order
                    // is the mockup's exactly: [New][Open…] then the column
                    // [New from template…][Recent ⌄][Close].
                    large("file.new"),
                    large("file.open"),
                    command("file.new_from_template"),
                    Item::custom(super::RECENT_FILES),
                    command("file.close"),
                ],
            ),
            // ---------------------------------------------------------------
            // Save — see the module header on why this band has one item.
            // ---------------------------------------------------------------
            // ---------------------------------------------------------------
            // Recognise — OCR.
            //
            // ★ `RIBBON_IA.md` §5.7 puts this on **Tools**, and it is here
            // instead. The whole argument is in `super::tools`'s header, where a
            // reader looking for it in the specified place will find it; the
            // short version is that Read's tab list is `["file", "view"]`, so a
            // command on Tools is unreachable in the one mode the operator
            // specifically asked for it in.
            //
            // Between File and Save, which is where it belongs on its own terms:
            // this band's neighbours are the verbs that make a document exist
            // (open) and the verbs that write one out (save, export), and OCR is
            // the second kind. Its product is a new file and it never touches
            // this one.
            // ---------------------------------------------------------------
            group(
                "recognise",
                ribbon::group_file_recognise(),
                [large("file.ocr")],
            ),
            group(
                "save",
                ribbon::group_file_save(),
                // ★★ Third and last in the group, which is the order of
                // increasing consequence: overwrite what you have, write
                // another one beside it, write another one that has dropped
                // things. A destructive-adjacent command at the bottom of a
                // group is one an operator arrives at deliberately.
                [
                    // ★ Large — the mockup's `Save` big, with the three
                    // qualified saves in a column beside it. First in the
                    // group already, so the hoist is a no-op.
                    large("file.save"),
                    // ★ **Save as** between Save and Save a copy** — 2026-09-02,
                    // O95 — and the position is the group's own stated order of
                    // increasing consequence, not the end of the list.
                    //
                    // Save writes the file you have. Save As writes a different
                    // file **and moves you to it**. Save a copy writes a
                    // different file and leaves you where you are. Ordered by
                    // "how far from the file I am editing does this leave me",
                    // Save As sits between them — and it is also where Word,
                    // Acrobat and every other editor put it, which is the
                    // stronger argument: this is not a place to be original.
                    command("file.save_as"),
                    command("file.save_copy"),
                    command("file.save_compacted"),
                ],
            ),
            // ---------------------------------------------------------------
            // Export — writing this document out as something else.
            //
            // `Export form data` moves here from the Forms pane: it writes
            // a file, which makes it an export, and leaving it inside a
            // panel meant only an operator who already had the panel open
            // could find it.
            //
            // Export image (PNG/JPEG/TIFF with a DPI picker) and Export
            // text are **C** — `pdfcer-core` does both and neither has a
            // GUI surface. They are the cheapest wins on this tab and they
            // are still absent until the shell exists, because a **C** row
            // is an engine, not a command.
            //
            // ★ **The two text-copy commands are back, and this time on the
            // right band.** Operator decision, 2026-08-14: `edit.copy_page_text`
            // and `edit.copy_document_text` became `file.copy_page_text` and
            // `file.copy_document_text`.
            //
            // The module header above still records that they LEFT this tab, and
            // that record stands rather than being edited away — the reasoning
            // then was *"copying text out of a document is a content
            // operation"*, and it was arguing against the junk-drawer File tab
            // this ribbon replaced. What it got wrong is that a content
            // operation is not necessarily an **authoring** operation. Copying
            // reads the page and writes to the clipboard; it cannot change a
            // byte of the document. So it belongs on the tab every mode shows,
            // for the same reason `edit.form_fill` became `view.panel_forms`:
            // *filling is not authoring*, and neither is copying.
            //
            // What made the difference visible was the chord/mode gate
            // (`crate::app::modes::capability::offers_command`), which refused
            // `Ctrl+Shift+C` in Read — a mode whose whole standard is Acrobat
            // Reader, which copies text.
            //
            // They are in **Export**, not in a Clipboard group of their own, and
            // that is the substantive half of the decision. An export is
            // *content of this document, written out to somewhere that is not
            // this document*; DXF writes geometry to a file, form data writes
            // filled values to a file, and these two write text to the
            // clipboard. Only the destination differs, and the labels carry it.
            // A one-purpose `clipboard` band on this tab would have been the
            // Edit tab's now-deleted group moved sideways, and would have
            // implied an object clipboard that does not exist (`edit.cut`,
            // `edit.copy`, `edit.paste` are all **N** — see `super::PLANNED`).
            //
            // Page before document, matching the Edit tab's order and the
            // useful one: the narrow, instant, chord-bound verb first; the
            // whole-document one — which can block the window on a long file,
            // as its tooltip says — second.
            // ---------------------------------------------------------------
            group(
                "export",
                ribbon::group_file_export(),
                [
                    command("file.export_dxf"),
                    // ★ O120, 2026-09-04. Second in the band, directly after
                    // the other export that writes a picture of the page's own
                    // content — and before the form-data pair, which is a round
                    // trip and reads as one. §5.1's own Export table has the
                    // image row second for the same reason.
                    command("file.export_image"),
                    // ★★★ Export text, 2026-09-04. Third, directly after the
                    // two exports that write a derivative of the page's own
                    // content and before the form-data pair, which is a round
                    // trip and reads as one.
                    //
                    // ★ Its natural neighbours are the two copy-text verbs at
                    // the end of this band, and it is deliberately NOT beside
                    // them. Those two write to the **clipboard**; this writes a
                    // file, which is what every control from `export_dxf` to
                    // `import_form_data` does. Grouping by destination keeps the
                    // band readable in one pass — four file verbs, then the two
                    // clipboard ones — where grouping by subject would put a
                    // file write between two clipboard writes and leave the band
                    // with no order at all.
                    command("file.export_text"),
                    command("file.export_form_data"),
                    // ★ Import directly after export, in that order, because
                    // the pair is a round trip and an operator meets the half
                    // they will do first. It is also the order of increasing
                    // consequence: exporting reads, importing writes.
                    command("file.import_form_data"),
                    command("file.copy_page_text"),
                    command("file.copy_document_text"),
                ],
            ),
            // ---------------------------------------------------------------
            // ★★★ SECURITY — `OPERATOR_REQUESTS.md` **O119**, approved and
            // wired 2026-09-04: *"yes add encryption and permissions"*.
            //
            // # Placement: immediately after Export, and it is the mockup's
            //
            // `mockups/pdfcer-shell.html` draws it exactly here — between Export
            // and Print — and the operator approved that mockup. Its own comment
            // gives the reason, which is his framing of the question rather than
            // a taxonomy: O119 asks *"do you want to protect a drawing before you
            // send it out?"*, so the band sits immediately after the band that
            // sends it out.
            //
            // Why a NEW GROUP on **File** rather than a row in Edit ▸ Protect,
            // which is where a reader would first look: every other command on
            // the Edit tab is an undoable edit to page CONTENT, and these two are
            // neither. `EDITABLE_SURFACES.md` calls `set_encryption` *"a save
            // transform, not an undoable edit"*. `crate::shell::commands::catalog::file`
            // carries the full argument at the registrations.
            //
            // # Both LARGE, matching the mockup
            //
            // Two controls in a band of two, and both are consequential enough to
            // be found rather than scanned for: one puts a password on the
            // operator's drawing and the other decides what a recipient's reader
            // is asked to allow. The mockup draws both `big`.
            //
            // ★ The group's caption is `crate::text::protect::group_file_security`
            // rather than a `ribbon::` sibling, and the full path is written out
            // so the seam is visible at the call site. Its own doc gives the
            // reason: when a feature's copy is one subject and one module, the
            // caption is part of that subject.
            // ---------------------------------------------------------------
            group(
                "security",
                crate::text::protect::group_file_security(),
                [large("file.encrypt"), large("file.permissions")],
            ),
            // ---------------------------------------------------------------
            // Print. Imposition (n-up / booklet / poster) is **C**.
            // ---------------------------------------------------------------
            group("print", ribbon::group_file_print(), [large("file.print")]),
            // ---------------------------------------------------------------
            // Document — inspection of what is inside the file.
            //
            // `Security` is **N** and would sit third. Its absence is
            // visible in a way worth noting: a band called Document that
            // cannot tell you whether a document is encrypted is doing
            // half its job, and the status bar carries that fact today.
            // ---------------------------------------------------------------
            group(
                "document",
                ribbon::group_file_document(),
                [command("file.properties"), command("file.fonts")],
            ),
            // ---------------------------------------------------------------
            // pdfcer — the application's own settings, help and identity.
            //
            // Three controls, all of them about the PROGRAM rather than about
            // the document, which is what the group's caption says and what
            // decides membership. None carries an `enabled_when`: an operator
            // with nothing open still has a version to check and a licence to
            // read.
            //
            // ★ `file.about` shipped on 2026-08-14 and its `PLANNED` entry
            // ("N — there is no about box.") is retired. It is not a courtesy
            // control: it is the in-application half of the attribution
            // surface that the operator's decision to ship CC-BY-SA-4.0 OCR
            // model weights requires. See `crate::text::about` and
            // `crate::dialogs::about`.
            //
            // It goes LAST in the group, which is where every reference
            // application puts it — Acrobat, Inkscape and SolidWorks all end
            // their Help menu with About, and none of them opens with it.
            // ---------------------------------------------------------------
            group(
                "pdfcer",
                ribbon::group_file_pdfcer(),
                [
                    // ★ Large — the mockup's `Settings…` big. First in the
                    // group already.
                    large("file.settings"),
                    command("file.shortcuts"),
                    command("file.about"),
                ],
            ),
        ])
}
