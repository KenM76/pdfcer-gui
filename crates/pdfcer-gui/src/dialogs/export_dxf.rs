//! # `dialogs::export_dxf` — the page's geometry, at a scale somebody can
//! defend
//!
//! ## The gap this closes
//!
//! `file.export_dxf` was registered, drawn on File ▸ Export, marked `★ P3` in
//! `shell::commands::reach`'s `SCAFFOLDED` list, and its recorded reason was
//! **"No recorded reason anywhere. Scaffolded by omission, not by decision."**
//! It was the *first* entry in that list and one of only three with no reason
//! at all — the second of which, `edit.insert_image`, turned out the same way
//! yesterday: no blocker, only an entry nobody had looked at.
//!
//! `pdfcer-core`'s `export::dxf` has shipped the whole time, and the old shell
//! has the feature (`FEATURES.md`'s `gui` column, which is this project's
//! acceptance criteria).
//!
//! ## ★ The sentence the whole window is arranged around
//!
//! `DxfOptions::scale`'s own doc:
//!
//! > **This is the field the whole feature turns on.** Every generic PDF→DXF
//! > converter exports at paper scale and says nothing, so a **1:2 detail
//! > arrives at half size and looks plausible.**
//!
//! *Looks plausible* is the problem. A DXF at the wrong scale opens cleanly,
//! measures consistently, and is wrong, and the person who discovers it is
//! whoever cuts from it.
//!
//! pdfcer can do better than guess because it already has the operator's own
//! calibration — the ce dimensions they drew and the group scale they set — and
//! `suggest_scale_for_groups` is the query that turns that into an answer.
//!
//! ## ★ Three answers, and the window says which one it has
//!
//! `DxfScaleSuggestion` is deliberately not an `Option<f64>`:
//!
//! | | the window |
//! |---|---|
//! | `Calibrated` | seeds the field, and names **the group the number came from** — a bare figure is a claim the operator cannot check |
//! | `Uncalibrated` | seeds 1.0 and says in words that this is a **choice rather than a measurement**, and how to make it a measurement |
//! | `Conflicting` | lists every candidate and makes the operator pick. A sheet with a 1:50 plan and a 1:5 detail is a *correct drawing*; one DXF scale cannot serve both, and only the operator knows which half they are exporting for |
//!
//! ## ★ The PAGE-scoped query, not the document one
//!
//! `suggest_scale_for_groups` with `dimension_groups_on_page`, never
//! `suggest_scale`. The engine spells out what the document-wide one costs a
//! per-page export: *"a sheet set whose page 3 is a 1:5 detail will either
//! refuse a perfectly unambiguous page-1 export or — worse, when page 1 has no
//! calibration of its own — **silently export it at page 3's scale**."*
//!
//! That is the same defect the feature exists to prevent, arriving through the
//! front door.

use egui::Ui;
use pdfcer_core::export::dxf::{
    DxfOptions, DxfScaleSuggestion, DxfText, DxfUnits, suggest_scale_for_groups,
};

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::export_dxf as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-dxf"; // ui-text-exempt: trace region name, never displayed
/// The region the scale field publishes.
pub const REGION_SCALE: &str = "export-dxf.scale"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-dxf.export"; // ui-text-exempt: trace region name, never displayed

/// The Export-DXF window's live state.
pub struct ExportDxfDialog {
    /// The page being exported, frozen at open.
    ///
    /// Frozen for the reason every page-scoped dialog here freezes it: an
    /// operator who opens this on page 7 and pages away must not export page 9.
    /// The window says which page, so the choice is checkable.
    page_index: usize,
    /// What pdfcer inferred, kept so the disclosure can be redrawn without
    /// re-querying the model every frame.
    ///
    /// ★ Also kept because it is **evidence**, not a default: the operator may
    /// type over the scale, and the sentence naming where the suggestion came
    /// from stays true and stays on screen. A window that forgot its own
    /// inference the moment it was overridden would leave the operator unable
    /// to check what they had just decided against.
    suggestion: DxfScaleSuggestion,
    /// The options as they will be handed to the engine.
    ///
    /// The engine's own struct, edited in place. Nothing here mirrors it into
    /// local fields — a shadow copy is how a window comes to show one thing and
    /// write another, and `DxfOptions` is already exactly the shape the writer
    /// takes.
    options: DxfOptions,
    /// Set by Export, consumed after the window's closure returns.
    export_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl ExportDxfDialog {
    /// Open the window for the page on screen.
    ///
    /// The suggestion is computed **once, here**, from the page's own dimension
    /// groups. Re-querying per frame would be a decomposition walk and a model
    /// clone sixty times a second for an answer that cannot change while a
    /// modal window is up.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Self {
        let page_index = doc.view.page_index;
        let model = doc.session.dimension_model();
        // ★ The page's OWN groups. See the module header for what the
        // document-wide query costs here.
        let groups = doc.session.dimension_groups_on_page(page_index);
        let suggestion = suggest_scale_for_groups(&model, &groups);

        let mut options = DxfOptions::default();
        // Seeded from the inference where there is one. `Conflicting` is
        // deliberately NOT seeded from the first candidate: picking one would be
        // pdfcer answering a question it has just said it cannot answer, and the
        // operator would find a plausible number already in the box.
        if let DxfScaleSuggestion::Calibrated { scale, units, .. } = &suggestion {
            options.scale = *scale;
            options.units = *units;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-dxf-open page={page_index} groups={} suggestion={suggestion:?}",
                groups.len()
            )
        });
        Self {
            page_index,
            suggestion,
            options,
            export_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        // ★ ITS OWN OS WINDOW as of 2026-08-21. The size below is an opening
        // bid rather than a measurement: this dialog was `resizable(false)`
        // with no declared size, so egui sized it to its content and no number
        // for it existed anywhere. `dialogs::host` grows the window to fit what
        // the body actually draws — see [`crate::dialogs::host::Host::fit`] for
        // why that is safer than thirteen guessed numbers, and for the two
        // guards that keep it from oscillating.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-dxf", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(420.0, 560.0),
            egui::vec2(340.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf-requested page={} scale={} units={:?} arcs={} text={:?}",
                    self.page_index,
                    self.options.scale,
                    self.options.units,
                    u8::from(self.options.fit_arcs),
                    self.options.text
                )
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Dxf {
                    page: self.page_index,
                    options: self.options,
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        ui.label(t::page_line(self.page_index.saturating_add(1)));
        ui.add_space(8.0);

        // --- scale --------------------------------------------------------
        // No `.strong()` — R84 / DEFECTS.md D11.
        ui.label(t::scale_heading());
        self.scale_disclosure(ui);
        ui.horizontal(|ui| {
            ui.label(t::scale_label());
            let response = ui.add(
                egui::DragValue::new(&mut self.options.scale)
                    .speed(0.01)
                    // Positive and finite. A zero or negative scale produces a
                    // DXF whose geometry is collapsed or mirrored — refused by
                    // the control's range rather than by a sentence, because
                    // unlike a placement rectangle there is no reading of a
                    // negative scale that an operator could have meant.
                    .range(0.000_001..=1_000_000.0),
            );
            crate::diag::ui_rect(REGION_SCALE, response.rect);
        });
        ui.add_space(8.0);

        // --- units --------------------------------------------------------
        ui.label(t::units_heading());
        ui.horizontal(|ui| {
            for option in [DxfUnits::Millimetres, DxfUnits::Inches] {
                ui.radio_value(&mut self.options.units, option, t::units_name(option));
            }
        });
        ui.add_space(8.0);

        // --- geometry -----------------------------------------------------
        ui.label(t::geometry_heading());
        ui.checkbox(&mut self.options.fit_arcs, t::fit_arcs());
        ui.weak(t::fit_arcs_hint());

        // ★ `DxfText` is a two-state enum and is presented as a checkbox,
        // because "write the text or not" is what the operator is deciding and
        // a radio pair would spend two rows saying it. Read and written through
        // the enum rather than mirrored into a local `bool`: a shadow copy is
        // how a window comes to show one thing and write another.
        let mut write_text = matches!(self.options.text, DxfText::Entities);
        if ui.checkbox(&mut write_text, t::write_text()).changed() {
            self.options.text = if write_text {
                DxfText::Entities
            } else {
                DxfText::Omit
            };
        }
        ui.weak(t::write_text_hint());
        ui.add_space(8.0);

        // --- commit --------------------------------------------------------
        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.button(t::export_button());
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// What pdfcer inferred, and what the operator should make of it.
    ///
    /// Drawn **above** the field rather than below it, because it is the reason
    /// the number in the field is what it is — and a caveat under a control is
    /// a caveat read after the control has been used.
    fn scale_disclosure(&mut self, ui: &mut Ui) {
        match &self.suggestion {
            DxfScaleSuggestion::Calibrated {
                scale,
                group,
                agreeing,
                ..
            } => {
                ui.weak(t::scale_from_group(*scale, group, *agreeing));
            }
            DxfScaleSuggestion::Uncalibrated => {
                // Not `weak`: this is the one disclosure in the window that
                // changes what an operator should do, and the fact that a
                // default is a choice rather than a finding is exactly the
                // thing a quiet grey line gets skipped over.
                ui.label(t::scale_uncalibrated());
            }
            DxfScaleSuggestion::Conflicting { candidates } => {
                ui.label(t::scale_conflicting(candidates.len()));
                // ★ Every candidate offered, none pre-selected. Selecting one
                // writes BOTH the scale and its units, because a candidate is a
                // group's whole opinion — a 1:50 metre group and a 1:50 inch
                // group are different answers wearing the same number.
                for candidate in candidates {
                    let chosen = (self.options.scale - candidate.scale).abs() < f64::EPSILON
                        && self.options.units == candidate.units;
                    if ui
                        .radio(
                            chosen,
                            t::scale_candidate(candidate.scale, &candidate.group),
                        )
                        .clicked()
                    {
                        self.options.scale = candidate.scale;
                        self.options.units = candidate.units;
                    }
                }
            }
            // `DxfScaleSuggestion` is `#[non_exhaustive]`-shaped in spirit and
            // may not be today; the arm is written so a fourth answer renders
            // the same honest sentence as "pdfcer does not know" rather than
            // nothing at all.
            // ui-text-exempt: clippy lint justification, never displayed
            #[allow(unreachable_patterns, reason = "belt to the enum's braces")]
            // ui-text-exempt: lint justification, never displayed
            _ => {
                ui.label(t::scale_uncalibrated());
            }
        }
    }
}

/// Open the window for `status`, or decline.
#[must_use]
pub fn open_for(status: &Status) -> Option<ExportDxfDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(ExportDxfDialog::open(doc)),
        _ => None,
    }
}
