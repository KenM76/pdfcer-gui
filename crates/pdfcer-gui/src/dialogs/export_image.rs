//! # `dialogs::export_image` — a picture of the page, in a format that can
//! actually hold what is on it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/export_image.md`.
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
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{ImageFormat, ImagePlan, PageScope, resolve_pages};
use crate::app::prefs::{MAX_EXPORT_DPI, MAX_JPEG_QUALITY, MIN_EXPORT_DPI, MIN_JPEG_QUALITY};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_image as t;
use pdfcer_core::settings::presets::RenderStandard;
use pdfcer_render::export::Rgb;

/// A resolution: arithmetic, and `dpi` typed after it is the box's own unit.
const DPI: entry::Kind = entry::Kind::Number(&["dpi"]);

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-image"; // ui-text-exempt: trace region name, never displayed
/// The region the format radio GROUP publishes — all four radios together.
pub const REGION_FORMAT: &str = "export-image.format"; // ui-text-exempt: trace region name, never displayed

/// The region ONE format's radio publishes, so a driven check can press a
/// named format rather than a coordinate.
#[must_use]
pub const fn region_for_format(format: ImageFormat) -> &'static str {
    match format {
        // ui-text-exempt: trace region names, matched by tools/ui-verify and
        // never displayed. The DISPLAY name of a format is
        // `crate::text::export_image::format_name`.
        ImageFormat::Png => "export-image.format.png",
        ImageFormat::Jpeg => "export-image.format.jpeg",
        ImageFormat::Svg => "export-image.format.svg",
        ImageFormat::Emf => "export-image.format.emf",
    }
}
/// The region the resolution field publishes.
pub const REGION_DPI: &str = "export-image.dpi"; // ui-text-exempt: trace region name, never displayed
/// The region ONE page-scope radio publishes.
#[must_use]
pub const fn region_for_scope(scope: PageScope) -> &'static str {
    match scope {
        // ui-text-exempt: trace region names, matched by tools/ui-verify and
        // never displayed.
        PageScope::CurrentPage => "export-image.pages.current",
        PageScope::AllPages => "export-image.pages.all",
        PageScope::Typed => "export-image.pages.typed",
    }
}
/// The region the page-scope radio GROUP publishes — all three together, plus
/// the range box.
pub const REGION_PAGES: &str = "export-image.pages"; // ui-text-exempt: trace region name, never displayed
/// The region the transparency checkbox publishes.
pub const REGION_TRANSPARENT: &str = "export-image.transparent"; // ui-text-exempt: trace region name, never displayed
/// The region the JPEG quality field publishes.
pub const REGION_QUALITY: &str = "export-image.quality"; // ui-text-exempt: trace region name, never displayed
/// The keep-text checkbox, published only while SVG or EMF is selected.
pub const REGION_KEEP_TEXT: &str = "export-image.keep-text"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-image.export"; // ui-text-exempt: trace region name, never displayed
/// The region the background-colour field publishes.
pub const REGION_BACKGROUND: &str = "export-image.background"; // ui-text-exempt: trace region name, never displayed
/// The region the rendering-standard drop-down publishes.
pub const REGION_STANDARD: &str = "export-image.standard"; // ui-text-exempt: trace region name, never displayed
/// Each rendering-standard item publishes this followed by
/// `RenderStandard::as_str`, or `none` for the operator's own settings.
pub const STANDARD_ITEM_PREFIX: &str = "export-image.standard."; // ui-text-exempt: trace region name prefix, never displayed
/// Height kept below the scrolling body for the separator and button row.
const FOOTER_PTS: f32 = 44.0;
/// The body's least height: a negative `max_height` draws nothing at all.
const BODY_FLOOR_PTS: f32 = 80.0;

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
    /// **Not cleared when JPEG is selected.** The checkbox goes dead and says
    /// why; the stored answer stays as the operator left it, so choosing JPEG
    /// to look at the quality control and choosing PNG again does not silently
    /// turn transparency off. The refusal is what makes that safe: a plan built
    /// while JPEG is selected still carries `transparent`, and
    /// `ImagePlan::impossible` refuses it by name rather than flattening.
    transparent: bool,
    /// JPEG quality, carried whatever the format, for the same reason.
    quality: u8,
    /// SVG/EMF: keep text as text rather than outlines. Carried whatever the
    /// format, like `quality`.
    keep_text: bool,
    /// The background colour as typed; Export waits while it is not a colour.
    background_text: String,
    /// The rendering standard for this export only; never remembered, because
    /// a standard is a property of one deliverable, not of the operator.
    standard: Option<RenderStandard>,
    /// Set by Export, consumed after the window's closure returns.
    export_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl ExportImageDialog {
    /// Open the window for the document on screen, seeded from what the last
    /// export asked for.
    #[must_use]
    pub fn open(doc: &OpenDoc, remembered: &crate::app::prefs::ExportImagePrefs) -> Self {
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
        let dialog = Self {
            page_index,
            page_count,
            largest_pt,
            format: remembered.format,
            scope: remembered.scope,
            // Deliberately empty — see the note on this function.
            range_text: String::new(),
            dpi: remembered.dpi,
            transparent: remembered.transparent,
            quality: remembered.quality,
            keep_text: remembered.keep_text,
            background_text: remembered.background.to_hex(),
            standard: None,
            export_requested: false,
            close_requested: false,
        };

        // **Traced from the BUILT dialog, and the position of these
        // lines is the whole point of them.**
        //
        //
        // Reading `dialog.*` closes that: if a future edit drops a
        // `remembered` from one of the five assignments, this line changes and
        // the driven check goes red, which it could not do before.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-image-open page={} pages={} format={} scope={} \
                 dpi={} transparent={} quality={} keep_text={} background={}",
                dialog.page_index,
                dialog.page_count,
                // Stable lowercase tokens, never `{:?}`. This project's
                // standing lesson, and the preferences file's own `*_key`
                // functions are what produce them, so the token a check reads
                // here and the token on disk cannot drift.
                crate::app::prefs::exporting::image_format_key(dialog.format),
                crate::app::prefs::exporting::page_scope_key_or(
                    dialog.scope,
                    crate::app::prefs::ExportImagePrefs::default().scope,
                ),
                dialog.dpi,
                u8::from(dialog.transparent),
                dialog.quality,
                u8::from(dialog.keep_text),
                dialog.background_text,
            )
        });
        dialog
    }

    /// **This window's state, reduced to what a different document would
    /// still want** — the producing half of `OPERATOR_REQUESTS.md` **O196**.
    fn habits(&self) -> crate::app::prefs::ExportImagePrefs {
        crate::app::prefs::ExportImagePrefs {
            format: self.format,
            scope: self.scope,
            dpi: self.dpi,
            // ⚠ Stored whatever the format, and that is the same decision
            // the field itself documents: the checkbox goes dead under JPEG
            // but the answer is kept, so an operator who glanced at the
            // quality control and went back to PNG has not silently lost
            // transparency — now across a restart as well as across a radio
            // press.
            transparent: self.transparent,
            quality: self.quality,
            keep_text: self.keep_text,
            background: self.background().unwrap_or(Rgb::WHITE),
        }
    }

    /// The typed background colour, or `None` while it is not one.
    fn background(&self) -> Option<Rgb> {
        Rgb::parse_hex(&self.background_text).ok()
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-image", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 780.0),
            egui::vec2(360.0, 340.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            // The body scrolls above the button row, so Export stays on screen
            // when a standard's disclosures make the body taller than the window.
            let ready = egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height((ui.available_height() - FOOTER_PTS).max(BODY_FLOOR_PTS))
                .show(ui, |ui| self.body(ui))
                .inner;
            self.footer(ui, ready);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(plan) = self.plan()
        {
            // O196, and the POSITION is the decision: the habits are
            // written when the operator presses Export, never when the window
            // closes. Closing without exporting is how a person says *"not
            // this"*. The argument is in
            // [`crate::dialogs::export_remembered`], stated once for all three
            // export windows.
            crate::dialogs::export_remembered::remember_image(self.habits(), prefs);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-image-requested format={} pages={} dpi={} \
                     transparent={} quality={} keep_text={} background={} standard={}",
                    // A token, never `{:?}`: the same reduction the
                    // preferences file performs, so a check reading this line
                    // and a check reading the file cannot disagree.
                    crate::app::prefs::exporting::image_format_key(plan.format),
                    plan.pages.len(),
                    plan.dpi,
                    u8::from(plan.transparent),
                    plan.quality,
                    u8::from(plan.keep_text),
                    plan.background.to_hex(),
                    plan.standard.map_or("none", |s| s.as_str()),
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
    fn pages(&self) -> Option<Vec<usize>> {
        resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )
    }

    /// The plan, or `None` when there is nothing to export.
    fn plan(&self) -> Option<ImagePlan> {
        Some(ImagePlan {
            format: self.format,
            pages: self.pages()?,
            dpi: self.dpi,
            transparent: self.transparent,
            quality: self.quality,
            keep_text: self.keep_text,
            // A refused colour never reaches here: Export is disabled while
            // the field is not a colour (`Self::footer`).
            background: self.background()?,
            standard: self.standard,
        })
    }

    /// The whole window body; returns whether Export can run.
    fn body(&mut self, ui: &mut Ui) -> bool {
        ui.label(t::intro());
        ui.add_space(8.0);

        self.format_group(ui);
        ui.add_space(8.0);
        let pages = self.pages_group(ui);
        ui.add_space(8.0);
        self.resolution_group(ui);
        ui.add_space(8.0);
        self.background_group(ui);
        ui.add_space(8.0);
        self.standard_group(ui);
        if self.format.is_vector() {
            ui.add_space(8.0);
            self.text_group(ui);
        } else {
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
        pages.is_some() && self.background().is_some()
    }

    /// The separator and the button row, pinned below the scrolling body.
    fn footer(&mut self, ui: &mut Ui, ready: bool) {
        ui.separator();
        ui.horizontal(|ui| {
            // Disabled rather than absent when there is nothing to export.
            // P3's rule: a greyed control the operator can see, beside the
            // sentence saying why, teaches what to change; a control that
            // vanishes teaches that the window is unpredictable.
            let response = ui.add_enabled(ready, egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// Which of the four writers, and what each one is for.
    fn format_group(&mut self, ui: &mut Ui) {
        // No `.strong()` — R84 / DEFECTS.md D11.
        ui.label(t::format_heading());
        let start = ui.cursor();
        for format in ImageFormat::ALL {
            let response = ui.radio_value(&mut self.format, format, t::format_name(format));
            // Each radio's OWN rectangle, so `tools/ui-verify` can press a
            // named format instead of dividing the group's height by four. See
            // [`region_for_format`] for why a computed offset inside a
            // container is a check that eventually presses the wrong control.
            crate::diag::ui_rect(region_for_format(format), response.rect);
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
        let start = ui.cursor();
        // Each radio publishes its OWN rectangle, for the reason
        // [`region_for_scope`] states and [`Self::format_group`] twenty lines
        // above already honours: a check that presses the group's rectangle
        // plus a computed offset presses the wrong control the day the range
        // hint gains a line.
        //
        // These three are also what makes O196 assertable at all. The question
        // a remembered scope raises is *which radio is selected when the window
        // opens*, and that question cannot be put to a group — only to the
        // radio that answers it.
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::CurrentPage,
            t::pages_current(self.page_index.saturating_add(1)),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::CurrentPage), response.rect);
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::AllPages,
            t::pages_all(self.page_count),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::AllPages), response.rect);
        ui.horizontal(|ui| {
            let response = ui.radio_value(&mut self.scope, PageScope::Typed, t::pages_range());
            crate::diag::ui_rect(region_for_scope(PageScope::Typed), response.rect);
            // Typing in the box selects the radio. Without it an operator types
            // a range, presses Export and gets the current page — the classic
            // shape of this control getting it wrong, and one the print dialog
            // already avoids.
            if ui.text_edit_singleline(&mut self.range_text).changed() {
                self.scope = PageScope::Typed;
            }
        });
        // The union is taken HERE rather than after the hint, so the group's
        // rectangle is what [`REGION_PAGES`] says it is: the three radios plus
        // the range box. The hint below is a sentence *about* them, not one of
        // them, and a region that quietly includes explanatory prose is a
        // region a driven check can press and hit nothing.
        crate::diag::ui_rect(REGION_PAGES, start.union(ui.cursor()));
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
            let (widget, refusal) = entry::drag_value(ui, &mut self.dpi, DPI);
            let response = refusal.show(
                ui.add(
                    widget
                        .speed(1.0)
                        // Bounded by the control rather than by a sentence: unlike a
                        // scale, there is no reading of a zero or negative
                        // resolution an operator could have meant. The ceiling is
                        // generous — the real limit is the pixel count, which is
                        // page-size dependent and is disclosed below.
                        //
                        // Read from the constants rather than repeated as
                        // literals. Those constants are what the preferences file
                        // clamps a read value into, and while the two were written
                        // out separately the file could refuse a resolution this
                        // box will happily produce — which reaches the operator as
                        // pdfcer forgetting a setting they had just made, the exact
                        // complaint O196 answers.
                        // `the_dragvalue_ranges_are_the_constants_the_file_clamps_to`
                        // is the guard.
                        .range(MIN_EXPORT_DPI..=MAX_EXPORT_DPI),
                ),
            );
            crate::diag::ui_rect(REGION_DPI, response.rect);
        });
        ui.weak(t::dpi_hint(self.format));

        // The pixel count, live. A resolution is an abstraction and a pixel
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

    /// Whether the page's transparency survives — and, for JPEG, the
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
            // **The refusal, by name, beside the control that would offer
            // the impossible combination.** Not `weak`: this is the sentence
            // the operator's own parenthesis asked for, and a grey line under a
            // greyed checkbox is two ways of being ignored at once.
            //
            // The checkbox is drawn DEAD rather than hidden, deliberately. A
            // control that disappears when JPEG is selected leaves the operator
            // to conclude the option does not exist; one that greys with a
            // reason under it says which of the other formats to choose
            // instead, which is what they actually need to know.
            ui.label(t::jpeg_has_no_alpha());
        }
        if !(can && self.transparent) {
            self.colour_row(ui);
        }
    }

    /// The colour the page is flattened onto: a hex field and a swatch picker,
    /// which edit the same value.
    fn colour_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(t::background_colour_label());
            // escape-disposition: dialog-cancels
            let response =
                ui.add(egui::TextEdit::singleline(&mut self.background_text).desired_width(80.0));
            crate::diag::ui_rect(REGION_BACKGROUND, response.rect);
            let mut picked = self.background().unwrap_or(Rgb::WHITE);
            let mut srgb = [picked.r, picked.g, picked.b];
            if egui::widgets::color_picker::color_edit_button_srgb(ui, &mut srgb).changed() {
                picked = Rgb {
                    r: srgb[0],
                    g: srgb[1],
                    b: srgb[2],
                };
                self.background_text = picked.to_hex();
            }
        });
        if self.background().is_none() {
            ui.label(t::background_colour_refused());
        }
    }

    /// The rendering standard this export is drawn under, with the chosen
    /// standard's evidence and disclosures beneath it.
    fn standard_group(&mut self, ui: &mut Ui) {
        ui.label(t::standard_heading());
        let before = self.standard;
        let selected = self
            .standard
            .map_or(t::standard_own_settings(), RenderStandard::title);
        let combo = egui::ComboBox::from_id_salt(REGION_STANDARD)
            .selected_text(selected)
            .width(360.0)
            .show_ui(ui, |ui| {
                let own = ui.selectable_value(&mut self.standard, None, t::standard_own_settings());
                crate::diag::ui_rect(&format!("{STANDARD_ITEM_PREFIX}none"), own.rect); // ui-text-exempt: a trace region name
                for s in RenderStandard::all() {
                    let item = ui.selectable_value(&mut self.standard, Some(*s), s.title());
                    crate::diag::ui_rect(
                        &format!("{STANDARD_ITEM_PREFIX}{}", s.as_str()),
                        item.rect,
                    );
                }
            });
        crate::diag::ui_rect(REGION_STANDARD, combo.response.rect);
        if self.standard != before {
            let token = self.standard.map_or("none", |s| s.as_str());
            crate::diag::trace(|| format!("export-image-standard-chosen standard={token}")); // ui-text-exempt: diagnostic trace
        }
        ui.weak(t::standard_hint());
        if let Some(s) = self.standard {
            pdfcer_gui_base::settingspages::preset::standard_detail(ui, s);
        }
    }

    /// The SVG/EMF text choice: outlines (default) or text kept as text.
    fn text_group(&mut self, ui: &mut Ui) {
        ui.label(crate::text::export_keeptext::heading());
        let response = ui.checkbox(
            &mut self.keep_text,
            crate::text::export_keeptext::checkbox(),
        );
        crate::diag::ui_rect(REGION_KEEP_TEXT, response.rect);
        ui.weak(crate::text::export_keeptext::hint(
            self.format,
            self.keep_text,
        ));
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
            // The range is the preferences file's own, for the reason
            // [`Self::resolution_group`] states in full.
            let (widget, refusal) = entry::drag_value(ui, &mut self.quality, entry::Kind::Count);
            let response = refusal.show(ui.add(widget.range(MIN_JPEG_QUALITY..=MAX_JPEG_QUALITY)));
            crate::diag::ui_rect(REGION_QUALITY, response.rect);
        });
        ui.weak(t::quality_hint());
    }
}

/// Open the window for `status`, or decline.
#[must_use]
pub fn open_for(
    status: &Status,
    remembered: &crate::app::prefs::ExportImagePrefs,
) -> Option<ExportImageDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => {
            Some(ExportImageDialog::open(doc, remembered))
        }
        _ => None,
    }
}
