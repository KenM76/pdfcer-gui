//! **How each dialog is BUILT** — every `DialogsState::open_*` constructor, and
//! the two guards each of them applies before a window can exist.
//!
//! # 1. Why this file exists, and what the seam actually is
//!
//! `dialogs/mod.rs` had grown to 1,535 lines and tripped R2 (no source file
//! over 1,500 lines) on 2026-09-05, with three concurrent tracks each having
//! added a dialog to it. R2's own rule is that reaching the ceiling is the
//! signal to *find the seam*, not to raise the limit — and the seam here was
//! already visible in the shape of the `impl` block, which had for weeks
//! contained two families of method that share a receiver and nothing else:
//!
//! | family | job | where it lives now |
//! |---|---|---|
//! | `open_*` / `close_scale` | **build** a dialog from defaults, a picked path, or a document survey, and decide whether it may exist at all | **this file** |
//! | `ask_*` / `take_*_answer` / `show` | carry a **question** to the operator and its **answer** back to `PdfcerApp`, and drive the per-frame draw-and-drain loop | `dialogs/mod.rs` |
//!
//! That is not a mechanical halving. The two families have different callers
//! (dispatch for the first, the frame loop and the save funnel for the second),
//! different failure modes, and — most usefully — **different invariants**,
//! which are stated once here rather than repeated at twenty-one sites.
//!
//! # 2. The two guards every opener applies, and why they live HERE
//!
//! The shell's dispatch pattern is *"push the chord blind, gate the effect in
//! dispatch"*. A ribbon control registered `enabled_when("doc.open")` cannot be
//! pressed without a document and cannot be pressed twice in one frame; **a
//! keyboard chord bound to the same command id has neither property.** So both
//! conditions are enforced at the one place a dialog is ever constructed:
//!
//! - **No document, no dialog.** Otherwise the chord on an empty canvas builds
//!   a window that [`super::DialogsState::show`] closes again on its very next
//!   frame — and some of them do real work on the way, such as enumerating the
//!   spooler over the network for Print.
//! - **Already open means leave it alone.** These functions build from
//!   *defaults*. A second press part-way through configuring a job would
//!   silently discard the range, the scale, the copy count, the annotation
//!   scope — the operator's own settings, thrown away by the very shortcut
//!   pressed to look at them.
//!
//! Enforcing both here fixes the button and the chord **by construction**,
//! rather than by a condition duplicated at the keymap that can drift from the
//! one in dispatch. Each function's own doc comment says which of the two it
//! applies and, where it applies only one, why the other does not arise.
//!
//! # 3. What this file must NOT grow into
//!
//! An opener decides **whether** a dialog may exist and gathers **what it needs
//! to exist**. It does not decide what the dialog does, does not write to the
//! document, and does not answer its own question. When an opener starts to
//! need more than a survey of `Status`, that is the signal that the reasoning
//! belongs in the dialog module itself — the same rule `app/actions/OVERVIEW.md`
//! sets for action arms, and for the same reason: a constructor that knows the
//! semantics of the thing it constructs is a second place for those semantics
//! to live, and two places drift.

use super::{
    DialogsState, about, compact, diagnostics, embed, export_dxf, export_image, export_text,
    formfield, insert_image, insert_pages, new_document, ocr, print, protect, redact, scale,
    shortcuts, textannot, unembed,
};
use crate::app::state::Status;

impl DialogsState {
    /// Open the print dialog for the document in `status`.
    ///
    /// **The dispatch target for the `file.print` command.** The command is
    /// registered `enabled_when("doc.open")`, so the ribbon button cannot be
    /// pressed without a document — but a keyboard chord bound to the same id
    /// has neither that guard nor the button's once-per-frame property, and
    /// the shell's own dispatch pattern is *"push the chord blind, gate the
    /// effect in dispatch"*. Both conditions are therefore enforced **here**,
    /// at the one place the dialog is ever built, which fixes the button and
    /// the chord by construction rather than by a condition duplicated at the
    /// keymap:
    ///
    /// - **No document, no dialog.** Without this, the chord on an empty
    ///   canvas would enumerate the spooler — a blocking call on a network
    ///   printer — to populate a window [`Self::show`] closes again on its
    ///   very next frame.
    /// - **Already open means leave it alone.** This function *builds* a
    ///   dialog from defaults. A second press part-way through configuring a
    ///   job would silently reset the range, the scale, the copy count and the
    ///   annotation scope — the operator's own settings, discarded by the
    ///   shortcut they pressed to look at them.
    pub fn open_print(&mut self, status: &Status) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.print.is_some() {
            return;
        }
        self.print = Some(print::PrintDialog::open(doc));
    }

    /// Open the Recognise-text dialog for the document in `status`.
    ///
    /// **The dispatch target for the `file.ocr` command**, and it applies the
    /// same two guards [`Self::open_print`] documents, for the same two
    /// reasons: the ribbon control is gated on `doc.pages` and a chord bound to
    /// the same id is not, so both are fixed here at the one place the dialog
    /// is built.
    ///
    /// The already-open guard is the stronger of the two here. A second press
    /// while a recognition is running would abandon a live worker thread and
    /// start another beside it, and a second press *after* one finished would
    /// discard recognised bytes the operator has not saved yet — several
    /// seconds of work and an unwritten document, thrown away by the shortcut
    /// they pressed to look at it.
    pub fn open_ocr(&mut self, status: &Status, picked: Vec<usize>) {
        if self.ocr.is_some() {
            return;
        }
        self.ocr = ocr::open_for(status, picked);
    }

    /// Open the Apply-redactions dialog for the document in `status`.
    ///
    /// **The dispatch target for the `edit.redact_apply` command**, and it
    /// applies the same two guards [`Self::open_print`] documents — the ribbon
    /// control is gated on `doc.pages` and a chord bound to the same id is not.
    ///
    /// ★ Both guards are load-bearing here in a way they are not elsewhere,
    /// because [`redact::RedactDialog::open`] **runs the whole removal**.
    ///
    /// - **No document, no dialog.** Without this, an invocation over an empty
    ///   shell would build a window that [`Self::show`] closes again on its very
    ///   next frame — a control that visibly flickers rather than one that
    ///   declines.
    /// - **Already open means leave it alone**, and this is the strong one. A
    ///   second press would re-run a full rewrite of the document *and* discard
    ///   the operator's two acknowledgements — throwing away the reading they
    ///   have just done on the one report in this program that has to be read.
    ///   Worse, it would silently replace a report computed against the marks as
    ///   they were with one computed against the marks as they are now, which is
    ///   the difference between the numbers on screen and the bytes that would
    ///   be written.
    pub fn open_redact(&mut self, status: &Status) {
        if self.redact.is_some() {
            return;
        }
        self.redact = redact::open_for(status);
    }

    /// Open the Encrypt / Permissions window — `file.encrypt` and
    /// `file.permissions`, which differ only in the [`crate::protect::Task`].
    ///
    /// Both guards [`Self::open_print`] documents are real, and the already-open
    /// one is the strong one: rebuilding would silently discard four password
    /// boxes and a permission list on a second press of a double-clicked button.
    /// It also declines a press of the *other* control while the window is up,
    /// which is right — switching the task under a half-filled form would change
    /// which job is about to be done without changing what is in the boxes.
    pub fn open_protect(&mut self, status: &Status, task: crate::protect::Task) {
        if self.protect.is_some() {
            return;
        }
        self.protect = protect::open_for(status, task);
    }

    /// Open the Render-diagnostics report for the document in `status`.
    ///
    /// **The dispatch target for the `tools.render_diagnostics` command**, and
    /// it applies the same two guards [`Self::open_print`] documents, for the
    /// same two reasons: the ribbon control is gated on `doc.open` and a chord
    /// bound to the same id is not.
    ///
    /// The no-document guard is the sharper of the two here. Without it a chord
    /// on an empty canvas would build a window that [`Self::show`] closes again
    /// on its very next frame — a control that visibly flickers rather than one
    /// that visibly declines, which is the harder of the two to diagnose.
    ///
    /// The already-open guard costs nothing (there is no configuration to
    /// discard) and is kept for About's reason: rebuilding would move the
    /// window back to the centre and the findings list back to the top, which
    /// for an operator half-way down a census reads as the program losing their
    /// place.
    ///
    /// ★ Note what it does **not** guard on: whether anything has been
    /// rasterized. `doc.open` is the registered predicate, and a document with
    /// no texture yet is precisely when an operator asks what the renderer did
    /// — so the dialog opens and *says* that nothing has been drawn, rather
    /// than the command silently doing nothing.
    /// Open the Set-scale dialog on `group`.
    ///
    /// The already-open guard is the same one every dialog here has, and it
    /// matters more than usual: a second press must not discard a ratio the
    /// operator has half typed, and re-opening would also re-capture the active
    /// group — so a group change made while the dialog was up would silently
    /// redirect the calibration.
    pub fn open_scale(&mut self, status: &Status, group: pdfcer_core::dimension::GroupId) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        if self.scale.is_some() {
            return;
        }
        self.scale = Some(scale::ScaleDialog::open(group));
    }

    pub fn open_scale_calibrated(
        &mut self,
        status: &Status,
        group: pdfcer_core::dimension::GroupId,
        drawn_pdf_length: f64,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.scale = Some(scale::ScaleDialog::calibrated(group, drawn_pdf_length));
    }

    /// Close the Set-scale dialog, whatever state it is in.
    ///
    /// Used when the operator asks to measure on the drawing: the window has to
    /// get out of the way of the page they are about to click on.
    pub fn close_scale(&mut self) {
        self.scale = None;
        self.text_annot = None;
    }

    /// **Open the text-annotation dialog for a just-placed annotation.**
    ///
    /// Raised by `Action::BeginTextAnnot`, which the canvas pushes on the
    /// gesture that finishes placing.
    ///
    /// ★ It REPLACES an open dialog rather than refusing, unlike
    /// [`Self::open_scale`]. The situations are opposite: that guard protects a
    /// half-typed value from a second ribbon press, and here a second placing
    /// gesture is the operator plainly saying they want to annotate somewhere
    /// else. Refusing would leave them looking at a window describing a box
    /// they have moved on from.
    pub fn open_text_annot(
        &mut self,
        status: &Status,
        page: usize,
        kind: crate::canvas::textannot::TextAnnotKind,
        rect: pdfcer_core::page_tree::Rect,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = Some(textannot::TextAnnotDialog::open(page, kind, rect));
    }

    /// **Open the placement dialog for a form control just put on the page.**
    ///
    /// Raised by `Action::BeginFormField`, which the canvas pushes on the click
    /// or drag-release that finishes placing, and by nothing else.
    ///
    /// ★ It REPLACES an open dialog rather than refusing, for the reason
    /// [`Self::open_text_annot`] gives: a second placing gesture is the operator
    /// plainly saying they want a control somewhere else, and refusing would
    /// leave them looking at a window describing a rectangle they have moved on
    /// from. What it costs is the abandoned draft, which authored nothing.
    pub fn open_form_field(
        &mut self,
        status: &Status,
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        draft: crate::canvas::formfield::Draft,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.form_field = Some(formfield::FormFieldDialog::open(page, rect, draft));
    }

    pub fn open_diagnostics(&mut self, status: &Status) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        if self.diagnostics.is_some() {
            return;
        }
        self.diagnostics = Some(diagnostics::DiagnosticsDialog::open());
    }

    /// Open the About dialog.
    ///
    /// **The dispatch target for the `file.about` command.** Unlike
    /// [`Self::open_print`] it takes no [`Status`], because it needs none:
    /// About describes the application, and the application is always there.
    /// The command is registered with no `enabled_when` for the same reason.
    ///
    /// The already-open guard is kept, and for a slightly different reason
    /// than print's: this dialog holds no configuration to discard, so
    /// rebuilding it would lose nothing — but it would *move* the window back
    /// to the centre and reset its scroll position, which for an operator
    /// half-way down the attribution list reads as the program losing their
    /// place.
    pub fn open_about(&mut self) {
        if self.about.is_some() {
            return;
        }
        self.about = Some(about::AboutDialog::open());
    }

    /// Open the sized-New dialog.
    ///
    /// **The dispatch target for `file.new_from_template`.** Like
    /// [`Self::open_about`] it takes no [`Status`] and the command is
    /// registered with no `enabled_when`: New is the command an empty shell
    /// exists to offer, and gating it on a document would grey the one control
    /// that answers *"there is nothing here"*.
    ///
    /// The already-open guard is print's rather than About's: this window holds
    /// a size, an orientation and two typed numbers, and a second press of the
    /// ribbon control part-way through would silently reset all four to A4
    /// portrait — the operator's own choices, discarded by the control they
    /// pressed to look at them.
    /// Open the insert dialog for `path`, having counted its pages.
    ///
    /// **The dispatch target for `pages.insert_from_file`, after the picker.**
    ///
    /// # ★ Why the page count is read here and not in the dialog
    ///
    /// Because a file that will not open must be reported **instead of** the
    /// dialog, not after the operator has filled one in. Opening a window that
    /// says "0 pages" and refuses its own commit button would be a surface
    /// asking a question that cannot be answered.
    ///
    /// The load is cheap relative to what follows — the same file is opened
    /// again by the insert itself — and a document parsed twice is the honest
    /// trade for a dialog that can state a fact before the operator commits.
    pub fn open_insert_pages(&mut self, path: std::path::PathBuf, current_page: usize) {
        if self.insert_pages.is_some() {
            return;
        }
        let count = match pdfcer_core::document::Document::load(&path) {
            Ok(doc) => pdfcer_core::page_tree::pages(&doc).map_or(0, |p| p.len()),
            Err(error) => {
                let detail = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "insert-picked-unreadable path={path:?} reason={detail}"
                    )
                });
                0
            }
        };
        if count == 0 {
            // Nothing to ask about. The refusal is the status-bar sentence the
            // insert path already owns, so the operator meets one voice rather
            // than a dialog and then a note.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("insert-declined path={path:?} reason=no-pages")
            });
            return;
        }
        self.insert_pages = Some(insert_pages::InsertPagesDialog::open(
            path,
            count,
            current_page,
        ));
    }

    pub fn open_new_document(&mut self) {
        if self.new_document.is_some() {
            return;
        }
        self.new_document = Some(new_document::NewDocumentDialog::open());
    }

    /// Open the Export-DXF window for the page on screen.
    ///
    /// **The dispatch target for the `file.export_dxf` command.** The two guards
    /// [`Self::open_print`] documents apply, and the no-document one is real
    /// rather than ceremonial: the window's scale suggestion is computed from
    /// the document's dimension model at construction, so there is nothing to
    /// build without one.
    /// Open the keyboard reference.
    ///
    /// **The dispatch target for `file.shortcuts`.** No document guard, unlike
    /// every other `open_*` here: the window lists key bindings, which exist
    /// whether or not a file is open, and refusing it on an empty canvas would
    /// hide it from exactly the operator most likely to want it.
    pub fn open_shortcuts(&mut self) {
        if self.shortcuts.is_some() {
            return;
        }
        self.shortcuts = Some(shortcuts::ShortcutsDialog::open());
    }

    pub fn open_export_dxf(&mut self, status: &Status) {
        if self.export_dxf.is_some() {
            return;
        }
        self.export_dxf = export_dxf::open_for(status);
    }

    /// **The dispatch target for the `file.export_text` command**, with
    /// [`Self::open_export_dxf`]'s two guards and for its reasons.
    pub fn open_export_text(&mut self, status: &Status) {
        if self.export_text.is_some() {
            return;
        }
        self.export_text = export_text::open_for(status);
    }

    /// Open the Export-image window for the open document.
    ///
    /// **The dispatch target for the `file.export_image` command.** The two
    /// guards [`Self::open_print`] documents apply, and the no-document one is
    /// real rather than ceremonial: the window measures the largest page at
    /// construction to promise a pixel count, and there is nothing to measure
    /// without one.
    pub fn open_export_image(&mut self, status: &Status) {
        if self.export_image.is_some() {
            return;
        }
        self.export_image = export_image::open_for(status);
    }

    /// Open the Embed-fonts window, and say so when there is nothing to open.
    ///
    /// **The dispatch target for `tools.embed_fonts`.** The two guards
    /// [`Self::open_print`] documents apply.
    ///
    /// ## ★★ It returns a sentence, unlike every other `open_*` here
    ///
    /// Because this is the one command whose honest answer is often *"there is
    /// nothing to do"* - a document whose fonts are all embedded is the normal
    /// case, not an error - and a window that opened to say so would be a modal
    /// the operator has to dismiss to learn they did not need it. So the
    /// construction is allowed to decline, and the decline becomes a line the
    /// caller records where every other outcome of a command is recorded.
    ///
    /// ★ `Some(String)` rather than a `bool`, for the reason
    /// `prefs::fonts::add` returns one: the caller has two different things to
    /// say - *"nothing is missing"* and *"you have no font folders"* - and it
    /// cannot tell them apart from a flag.
    pub fn open_embed_fonts(
        &mut self,
        status: &Status,
        folders: &[std::path::PathBuf],
    ) -> Option<String> {
        if self.embed.is_some() {
            return None;
        }
        let Status::Open(_) = status else {
            return None;
        };
        self.embed = embed::open_for(status, folders);
        if self.embed.is_some() {
            return None;
        }
        // ★★★ ONE sentence now, and the branch that used to be here was
        // FALSIFIED by O47 rather than simplified away.
        //
        // It read: with no folders configured, say *"pdfcer has no font folders,
        // so it cannot embed anything."* True until 2026-08-28. Since the
        // operator answered O47 with *"yes"*, pdfcer's own standard-14 faces
        // answer when nothing of theirs can — so a document with a missing
        // Helvetica and no folders at all now **opens the window** instead of
        // declining, and the only thing a decline can mean is that there was
        // nothing to do.
        //
        // ⇒ A decline message is a claim about why, and the reasons a program
        // declines change under it. This one would have kept telling operators
        // to configure a folder they no longer need, at the exact moment they
        // were most likely to believe it.
        Some(crate::text::embed::nothing_missing().to_owned())
    }

    /// Open the compacted-copy window, or answer why the engine refused.
    ///
    /// **The dispatch target for `file.save_compacted`.** Returns `Some` only
    /// for a refusal, which is the same shape [`Self::open_embed_fonts`] uses
    /// and for the same reason: the caller records the sentence where every
    /// other outcome of a command is recorded, and a refusal nobody surfaces is
    /// a button that does nothing.
    ///
    /// ★ Unlike the two font commands, this **cannot** decline for want of
    /// anything to do. A file with nothing to reclaim still gets the window,
    /// which says so — an operator who asked for a copy is owed one even when it
    /// comes out the same size.
    pub fn open_compact(&mut self, status: &Status) -> Option<String> {
        if self.compact.is_some() {
            return None;
        }
        match compact::open_for(status)? {
            Ok(dialog) => {
                self.compact = Some(dialog);
                None
            }
            Err(sentence) => Some(sentence),
        }
    }

    /// Open the Remove-fonts window, and say so when there is nothing to open.
    ///
    /// **The dispatch target for `tools.unembed_fonts`.** The same shape as
    /// [`Self::open_embed_fonts`] and for the same reason: a document with
    /// nothing removable is an ordinary document, not an error, and a window
    /// saying so is a modal an operator dismisses to learn they did not need
    /// it.
    ///
    /// ★ It takes no folder list. Removal needs no donor - it deletes what the
    /// document already carries - which is the whole asymmetry between the two
    /// commands and the reason only one of them was blocked on a preference.
    pub fn open_unembed_fonts(&mut self, status: &Status) -> Option<String> {
        if self.unembed.is_some() {
            return None;
        }
        let Status::Open(_) = status else {
            return None;
        };
        self.unembed = unembed::open_for(status);
        if self.unembed.is_some() {
            return None;
        }
        Some(crate::text::unembed::nothing_removable().to_owned())
    }

    /// Open the Insert-image window for an already-imported picture.
    ///
    /// **The dispatch target for the `edit.insert_image` command**, reached only
    /// after the file has been chosen AND imported — see that arm for why the
    /// import happens first.
    ///
    /// The already-open guard matters here the way it matters for OCR: a second
    /// press would discard a placement the operator has typed and replace the
    /// imported bytes with another file's, so the window they pressed the
    /// shortcut to look at would come back describing a different picture.
    pub fn open_insert_image(
        &mut self,
        status: &Status,
        image: std::sync::Arc<pdfcer_core::image_import::ImportedImage>,
        name: String,
    ) {
        if self.insert_image.is_some() {
            return;
        }
        self.insert_image = insert_image::open_for(status, image, name);
    }
}
