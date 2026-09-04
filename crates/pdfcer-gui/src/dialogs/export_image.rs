//! # `dialogs::export_image` — a picture of the page, in a format that can
//! actually hold what is on it
//!
//! ## The gap this closes — `OPERATOR_REQUESTS.md` **O120**
//!
//! The operator, 2026-09-03, verbatim:
//!
//! > *"can you add the ability to export page(es) to png, jpg, svg. note that
//! > there had better be full support (including transparency where
//! > supported!). Also I'd like to be able to copy and paste anything to other
//! > software - like copy and paste vector graphics into word or inkscape for
//! > example if possible."*
//!
//! He said it to the **engine** side, which shipped all of it the same day and
//! sent a note — *"informational, no reply needed; consume when convenient"* —
//! that nothing here was required to read. `RIBBON_IA.md` §5.1 had carried the
//! row (`Export image… (PNG/JPEG/TIFF, DPI picker)`, marked **C**) since the
//! ribbon was specified, and `shell::manifest::registers`' `PLANNED` entry said
//! *"needs a DPI picker and a save dialog; no engine work"*. Both were right and
//! neither was a gate, so it stayed unbuilt for a day after it had been asked
//! for out loud.
//!
//! ★ **Two departures from that IA row, stated rather than made quietly:**
//!
//! * **TIFF is not offered.** `pdfcer-render::export` encodes PNG and JPEG and
//!   nothing else. The IA row predates the export module; offering a format
//!   with no encoder behind it would be a control that fails on press.
//! * **SVG is offered, and the IA row does not mention it.** The row predates
//!   `pdfcer_render::svg` entirely. It belongs in this window rather than in a
//!   second one because the question the operator is answering — *what kind of
//!   picture of this page do I want* — is one question, and the answer that
//!   makes his own stated example work (*"copy and paste vector graphics into
//!   word or inkscape"*) is the vector one. A separate `Export SVG…` command
//!   would put the right answer behind a control he would have to know existed.
//!
//! ## ★★★ The sentence the whole window is arranged around
//!
//! *"full support (including transparency where supported!)"* — **the
//! parenthesis is the instruction.** It concedes that one of the three cannot
//! do it and asks pdfcer to be the thing that says which.
//!
//! So the Background group is not a checkbox with a hint. It is a checkbox that
//! **goes dead when JPEG is selected, with the reason under it in words**, and
//! the reason names the format, names what would otherwise happen, and names
//! the two formats that can. `crate::app::actions::imageexport::ImagePlan::
//! impossible` then refuses the same combination a second time at the writer,
//! because a guard that lives only in a window is a guard that a keymap, a
//! restored plan or a later window can walk past.
//!
//! ## ★★ Everything the window can be wrong about, it says BEFORE the picker
//!
//! `export_dxf`'s ordering rule — *"the operator is never asked where to put a
//! file that turns out to be empty"* — generalises here into four live
//! disclosures, each drawn beside the control that causes it:
//!
//! | shown | because |
//! |---|---|
//! | the pixel size of the largest page | a resolution is an abstraction; a pixel count is the thing that lands on disk |
//! | a resolution past `MAX_PIXMAP_EDGE` | the engine refuses it, and a refusal after a save dialog is a wasted answer |
//! | a typed range naming no page | fixable in the box in front of them |
//! | the multi-file naming pattern | a save dialog cannot say *"the name you type is a stem"* |
//!
//! ## ★ Why the render is not previewed
//!
//! `dialogs::print::preview` draws the page because a print job **places**
//! it — margins, scaling, a clip that will happen — and the preview is the only
//! way to see the placement. An image export places nothing. What comes out is
//! what the canvas is already showing, at a different number of pixels, so a
//! preview here would be a second, smaller copy of the canvas: cost without a
//! claim. What an operator cannot see and is therefore owed is **the pixel
//! count** and **what could not be expressed exactly**, and both are given in
//! words.
//!
//! ## Rule 15
//!
//! This window offers a **resolution** and never a scale. The distinction is
//! not pedantry here: `dialogs::export_dxf` is an entire window built to
//! establish a scale from the **ce dimensions** the operator has drawn, because
//! a DXF at the wrong scale opens cleanly and is wrong. A picture has no scale
//! to get wrong — it is a picture of a page at a stated size — so nothing here
//! reads the dimensioning model, and nothing here consults **pdf dimensions**
//! either.
//!
//! ## conventions: dialogs
//!
//! Corpus: `ui-conventions/dialogs.md`.
//!
//! - G1 is-an-os-window: **SATISFIED** — [`crate::dialogs::host::Host`], which
//!   is `show_viewport_immediate`. The operator's 2026-08-20 report (*"locked
//!   within the boundaries of the program's window"*) is answered by the host
//!   rather than by anything here; this window simply uses it, as `export_dxf`
//!   and `compact` do.
//! - G2 use-the-os-dialog: **SATISFIED where one exists** — the save picker is
//!   the system's, through `crate::app::files::pick_save_path`. This window is
//!   pdfcer's own because the choices in it (which format, which pages, what
//!   resolution, whether transparency survives) are choices only pdfcer has.
//! - G3 owned-by-the-app: **SATISFIED** by the host, which parents the viewport
//!   to the application window.
//! - G4 enter-accepts-escape-cancels: **PARTIAL** — Escape closes, through the
//!   host. Enter is not wired as the affirmative default and no button is drawn
//!   as the default. That is the whole directory's gap rather than this
//!   window's (`insert_image` records it identically) and fixing it in one
//!   dialog would make the other nine inconsistent; it belongs in
//!   [`crate::dialogs::host`].
//! - G5 keyboard-reachable: **PARTIAL** — every control is a standard egui
//!   widget and therefore tab-reachable, but egui's tab order is positional and
//!   nothing here asserts that focus starts in the format group or that the
//!   modal traps it. Directory-wide gap, same owner as G4.
//! - G6 remembers-position: **SATISFIED** — the host remembers a dialog's
//!   position and size across openings within a session.
//! - G7 destructive-verbs-named: **NOT APPLICABLE, and deliberately so.**
//!   Nothing here is destructive: an export cannot change the document (see
//!   `app::actions::export`'s header on why that is what makes these verbs a
//!   family). The one thing it *can* overwrite is a file on disk, and that
//!   confirmation is the system save dialog's, which G2 says is where it
//!   belongs.
//! - G8 cancel-is-silent: **SATISFIED** — Cancel closes and records nothing,
//!   and a cancelled save picker likewise returns without a sentence
//!   (`crate::app::files::Picked::Cancelled`'s own doc: *"a complete, correct,
//!   uninteresting outcome"*).
//! - G9 nothing-blocks-silently: **PARTIAL, and the honest reading is that this
//!   window makes it worse than most.** The export runs on the UI thread in the
//!   apply phase, so a fifty-page 600 DPI run freezes the window with no
//!   progress. What keeps that from being a defect today is that the same
//!   thread already renders every page the canvas shows, at the same cost per
//!   page, and this window states the pixel count before the press so the size
//!   of the wait is predictable. A background export with a progress bar is the
//!   right answer and it is a change to `app::actions`, not to this file.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{ImageFormat, ImagePlan, PageScope, resolve_pages};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_image as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-image"; // ui-text-exempt: trace region name, never displayed
/// The region the format radios publish.
pub const REGION_FORMAT: &str = "export-image.format"; // ui-text-exempt: trace region name, never displayed
/// The region the resolution field publishes.
pub const REGION_DPI: &str = "export-image.dpi"; // ui-text-exempt: trace region name, never displayed
/// The region the transparency checkbox publishes.
pub const REGION_TRANSPARENT: &str = "export-image.transparent"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-image.export"; // ui-text-exempt: trace region name, never displayed

/// The Export-image window's live state.
pub struct ExportImageDialog {
    /// The page that was on screen when the window opened.
    ///
    /// Frozen for `ExportDxfDialog`'s stated reason: *"an operator who opens
    /// this on page 7 and pages away must not export page 9."* The **This page
    /// only** radio names the number, so the choice stays checkable.
    page_index: usize,
    /// How many pages the document had when the window opened, for the same
    /// reason — the range is validated against the document the operator was
    /// looking at.
    page_count: usize,
    /// The largest page's size in points, for the live pixel-count line.
    ///
    /// The **largest**, not the current one: the line is a promise about the
    /// biggest file the run will produce, and a mixed sheet set whose page 3 is
    /// an A0 would otherwise be described by its A4 cover.
    largest_pt: (f32, f32),
    /// Which writer.
    format: ImageFormat,
    /// Which pages.
    scope: PageScope,
    /// The typed range, kept across scope changes so switching to **Every
    /// page** and back does not lose what was typed.
    range_text: String,
    /// Dots per inch.
    dpi: f32,
    /// Whether the page's own transparency survives.
    ///
    /// ★ **Not cleared when JPEG is selected.** The checkbox goes dead and says
    /// why; the stored answer stays as the operator left it, so choosing JPEG
    /// to look at the quality control and choosing PNG again does not silently
    /// turn transparency off. The refusal is what makes that safe: a plan built
    /// while JPEG is selected still carries `transparent`, and
    /// `ImagePlan::impossible` refuses it by name rather than flattening.
    transparent: bool,
    /// JPEG quality, carried whatever the format, for the same reason.
    quality: u8,
    /// Set by Export, consumed after the window's closure returns.
    export_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl ExportImageDialog {
    /// Open the window for the document on screen.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Self {
        let page_index = doc.view.page_index;
        let page_count = doc.pages.len();
        // The same measurement the canvas and the print preview take, so the
        // pixel count this window promises and the pixmap the export produces
        // cannot disagree by a box choice.
        let largest_pt = doc
            .pages
            .iter()
            .map(crate::viewer::page_extent_pts)
            .fold((0.0_f32, 0.0_f32), |acc, (w, h)| {
                (acc.0.max(w), acc.1.max(h))
            });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-image-open page={page_index} pages={page_count}")
        });
        Self {
            page_index,
            page_count,
            largest_pt,
            format: ImageFormat::Png,
            scope: PageScope::CurrentPage,
            range_text: String::new(),
            // 300, print grade. The same default the engine's `SvgOptions`
            // takes and for the same reason it states: *"an embedded raster
            // cannot be re-sampled later"*. A screen-grade default would make
            // the common case (a drawing going into a document that will be
            // printed) the case the operator has to remember to fix.
            dpi: 300.0,
            // ★ **Transparency ON by default**, and that is the operator's own
            // instruction rather than a taste: *"there had better be full
            // support (including transparency where supported!)"*. A default of
            // white would make the feature he asked for the one he has to find.
            transparent: true,
            // `JpegOptions::default()`'s own 90, and the engine states why: it
            // is where `jpeg-encoder` stops subsampling chroma, which for line
            // art and text is the difference between crisp and smeared colour
            // edges. Mirrored rather than read because `JpegOptions` is
            // `#[non_exhaustive]` and this is a `u8` in a window, not an
            // options struct.
            quality: 90,
            export_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-image", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 640.0),
            egui::vec2(360.0, 340.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(plan) = self.plan()
        {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-image-requested format={:?} pages={} dpi={} transparent={} quality={}",
                    plan.format,
                    plan.pages.len(),
                    plan.dpi,
                    u8::from(plan.transparent),
                    plan.quality
                )
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Image { plan },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The pages this window is currently offering, or `None` when the typed
    /// range names none.
    ///
    /// Called twice per frame — once to decide whether Export is usable, once
    /// to word the multi-file line — and once more on the press. Cheap: it is a
    /// parse of a short string, and computing it live is what keeps the button
    /// and the sentence beside the box from ever disagreeing.
    fn pages(&self) -> Option<Vec<usize>> {
        resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )
    }

    /// The plan, or `None` when there is nothing to export.
    ///
    /// ★ Deliberately does **not** refuse an impossible combination. The window
    /// prevents it (the checkbox is dead while JPEG is selected) and the writer
    /// refuses it by name; a third refusal here would silently drop the press
    /// with no sentence anywhere, which is the one outcome worse than either.
    fn plan(&self) -> Option<ImagePlan> {
        Some(ImagePlan {
            format: self.format,
            pages: self.pages()?,
            dpi: self.dpi,
            transparent: self.transparent,
            quality: self.quality,
        })
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        self.format_group(ui);
        ui.add_space(8.0);
        let pages = self.pages_group(ui);
        ui.add_space(8.0);
        self.resolution_group(ui);
        ui.add_space(8.0);
        self.background_group(ui);
        if !self.format.is_vector() {
            ui.add_space(8.0);
            self.quality_group(ui);
        }
        ui.add_space(8.0);

        // The naming rule, before the picker rather than after it. A save
        // dialog has no way to say "the name you type is a stem", and an
        // operator who did not expect it goes looking for a file that is not
        // there.
        if let Some(pages) = &pages
            && pages.len() > 1
        {
            let example = crate::app::actions::imageexport::output_path(
                std::path::Path::new("drawing"), // ui-text-exempt: an example STEM, joined into a catalog sentence
                self.format,
                pages[0],
                true,
            );
            ui.weak(t::multi_page_naming(
                pages.len(),
                &example.display().to_string(),
            ));
            ui.add_space(8.0);
        }

        ui.separator();
        ui.horizontal(|ui| {
            // ★ Disabled rather than absent when there is nothing to export.
            // P3's rule: a greyed control the operator can see, beside the
            // sentence saying why, teaches what to change; a control that
            // vanishes teaches that the window is unpredictable.
            let response = ui.add_enabled(pages.is_some(), egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// Which of the three writers, and what each one is for.
    fn format_group(&mut self, ui: &mut Ui) {
        // No `.strong()` — R84 / DEFECTS.md D11.
        ui.label(t::format_heading());
        let start = ui.cursor();
        for format in ImageFormat::ALL {
            ui.radio_value(&mut self.format, format, t::format_name(format));
            // The hint under each radio rather than only under the selected
            // one: the operator is CHOOSING, and a hint that appears only after
            // the choice is a hint about a decision already made.
            ui.weak(t::format_hint(format));
        }
        crate::diag::ui_rect(REGION_FORMAT, start.union(ui.cursor()));
    }

    /// Which pages, and — live — whether the typed range names any.
    ///
    /// Returns the resolved list so the caller can word the multi-file line and
    /// grey the Export button from one answer rather than from three.
    fn pages_group(&mut self, ui: &mut Ui) -> Option<Vec<usize>> {
        ui.label(t::pages_heading());
        ui.radio_value(
            &mut self.scope,
            PageScope::CurrentPage,
            t::pages_current(self.page_index.saturating_add(1)),
        );
        ui.radio_value(
            &mut self.scope,
            PageScope::AllPages,
            t::pages_all(self.page_count),
        );
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.scope, PageScope::Typed, t::pages_range());
            // Typing in the box selects the radio. Without it an operator types
            // a range, presses Export and gets the current page — the classic
            // shape of this control getting it wrong, and one the print dialog
            // already avoids.
            if ui.text_edit_singleline(&mut self.range_text).changed() {
                self.scope = PageScope::Typed;
            }
        });
        ui.weak(t::pages_range_hint());

        let pages = self.pages();
        // The refusal is drawn only for the typed case. "This page only" and
        // "Every page" can fail solely on an empty document, which the command's
        // own `doc.pages` predicate already excludes, and a sentence explaining
        // an unreachable state is noise that trains the operator to skip the bar.
        if pages.is_none() && self.scope == PageScope::Typed {
            ui.label(t::pages_range_invalid(self.page_count));
        }
        pages
    }

    /// How many dots to the inch, and what that costs in pixels.
    fn resolution_group(&mut self, ui: &mut Ui) {
        ui.label(t::dpi_heading());
        ui.horizontal(|ui| {
            ui.label(t::dpi_label());
            let response = ui.add(
                egui::DragValue::new(&mut self.dpi)
                    .speed(1.0)
                    // Bounded by the control rather than by a sentence: unlike a
                    // scale, there is no reading of a zero or negative
                    // resolution an operator could have meant. The ceiling is
                    // generous — the real limit is the pixel count, which is
                    // page-size dependent and is disclosed below.
                    .range(1.0..=4800.0),
            );
            crate::diag::ui_rect(REGION_DPI, response.rect);
        });
        ui.weak(t::dpi_hint(self.format));

        // ★ The pixel count, live. A resolution is an abstraction and a pixel
        // count is the file. Shown for the vector case too — an SVG's embedded
        // rasters are sampled at exactly this size, so the number is a real
        // statement about the file's weight there as well.
        let (w, h) = crate::app::actions::imageexport::pixel_size(
            self.largest_pt.0,
            self.largest_pt.1,
            self.dpi,
        );
        let limit = pdfcer_render::MAX_PIXMAP_EDGE;
        if w > limit || h > limit {
            // Not `weak`. This is the one line in the window that says a press
            // will fail, and a quiet grey line is what an operator skips.
            ui.label(t::dpi_too_large(w, h, limit));
        } else {
            ui.weak(t::dpi_pixels(w, h));
        }
    }

    /// ★★★ Whether the page's transparency survives — and, for JPEG, the
    /// refusal by name.
    fn background_group(&mut self, ui: &mut Ui) {
        ui.label(t::background_heading());
        let can = self.format.can_be_transparent();
        let response = ui.add_enabled(
            can,
            egui::Checkbox::new(&mut self.transparent, t::keep_transparency()),
        );
        crate::diag::ui_rect(REGION_TRANSPARENT, response.rect);

        if can {
            ui.weak(if self.transparent {
                t::keep_transparency_hint()
            } else {
                t::flatten_hint()
            });
        } else {
            // ★★★ **The refusal, by name, beside the control that would offer
            // the impossible combination.** Not `weak`: this is the sentence
            // the operator's own parenthesis asked for, and a grey line under a
            // greyed checkbox is two ways of being ignored at once.
            //
            // The checkbox is drawn DEAD rather than hidden, deliberately. A
            // control that disappears when JPEG is selected leaves the operator
            // to conclude the option does not exist; one that greys with a
            // reason under it says which of the three formats to choose
            // instead, which is what they actually need to know.
            ui.label(t::jpeg_has_no_alpha());
        }
    }

    /// How hard the JPEG encoder is allowed to squeeze.
    fn quality_group(&mut self, ui: &mut Ui) {
        // Drawn only for JPEG — see [`Self::body`]. PNG is lossless and SVG is
        // not pixels at all, so the control would be inert for two of three and
        // an inert control is a promise the format does not keep.
        if self.format != ImageFormat::Jpeg {
            return;
        }
        ui.label(t::quality_heading());
        ui.horizontal(|ui| {
            ui.label(t::quality_label());
            // The engine clamps rather than refusing, and says why; the control
            // holds the same range so the clamp is never reached from here.
            ui.add(egui::DragValue::new(&mut self.quality).range(1..=100));
        });
        ui.weak(t::quality_hint());
    }
}

/// Open the window for `status`, or decline.
///
/// The `doc.pages` guard is the command's too, and it is real rather than
/// ceremonial: every control in the window is a statement about a page, and the
/// largest-page measurement has nothing to fold over on an empty document.
#[must_use]
pub fn open_for(status: &Status) -> Option<ExportImageDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(ExportImageDialog::open(doc)),
        _ => None,
    }
}
