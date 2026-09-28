//! # `dialogs::export_dxf` — the page's geometry, at a scale somebody can
//! defend
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/export_dxf.md`.

use egui::Ui;
use pdfcer_core::dimension::Unit;
use pdfcer_core::export::dxf::{
    DxfOptions, DxfScaleSuggestion, DxfText, DxfUnits, DxfVersion, suggest_scale_for_groups,
};
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::export_dxf as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-dxf"; // ui-text-exempt: trace region name, never displayed
/// The region the scale field publishes: the ratio's real-world number.
pub const REGION_SCALE: &str = "export-dxf.scale"; // ui-text-exempt: trace region name, never displayed
/// The region the scale-group picker publishes.
pub const REGION_GROUP: &str = "export-dxf.group"; // ui-text-exempt: trace region name, never displayed
/// The region ONE units radio publishes.
#[must_use]
pub const fn region_for_units(units: DxfUnits) -> &'static str {
    match units {
        // ui-text-exempt: trace region names, matched by tools/ui-verify and
        // never displayed.
        DxfUnits::Millimetres => "export-dxf.units.mm",
        DxfUnits::Inches => "export-dxf.units.inches",
    }
}
/// The region the units radio GROUP publishes — both together.
pub const REGION_UNITS: &str = "export-dxf.units"; // ui-text-exempt: trace region name, never displayed
/// The region the DXF version drop-down publishes.
pub const REGION_VERSION: &str = "export-dxf.version"; // ui-text-exempt: trace region name, never displayed
/// The versions the drop-down offers, oldest first.
const VERSIONS: [DxfVersion; 3] = [DxfVersion::R12, DxfVersion::R2000, DxfVersion::R2004];
/// The region the fit-arcs checkbox publishes.
pub const REGION_ARCS: &str = "export-dxf.arcs"; // ui-text-exempt: trace region name, never displayed
/// The region the write-text checkbox publishes.
pub const REGION_TEXT: &str = "export-dxf.text"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-dxf.export"; // ui-text-exempt: trace region name, never displayed

/// Which of the three answers `suggest_scale_for_groups` gave, as a stable
/// lowercase token.
#[must_use]
pub fn suggestion_key(suggestion: &DxfScaleSuggestion) -> &'static str {
    match suggestion {
        // ui-text-exempt: diagnostic trace tokens, never displayed
        DxfScaleSuggestion::Calibrated { .. } => "calibrated",
        DxfScaleSuggestion::Uncalibrated => "uncalibrated",
        DxfScaleSuggestion::Conflicting { .. } => "conflicting",
        // A fourth answer reports as unknown rather than as one of the three.
        // `scale_disclosure` carries the same arm for the same reason.
        // ui-text-exempt: clippy lint justification, never displayed
        #[allow(unreachable_patterns, reason = "belt to the enum's braces")]
        // ui-text-exempt: diagnostic trace token, never displayed
        _ => "unknown",
    }
}

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
    /// Also kept because it is **evidence**, not a default: the operator may
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
    /// Every calibrated scale group in the document, for the picker.
    groups: Vec<GroupScale>,
    /// The ratio row. `options.scale` is derived from it on every edit, and
    /// it is reseeded whenever something else writes `options.scale`.
    ratio: Ratio,
    /// Set by Export, consumed after the window's closure returns.
    export_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl ExportDxfDialog {
    /// Open the window for the page on screen.
    #[must_use]
    pub fn open(doc: &OpenDoc, remembered: &crate::app::prefs::ExportDxfPrefs) -> Self {
        let page_index = doc.view.page_index;
        let model = doc.session.dimension_model();
        // The page's OWN groups. See the module header for what the
        // document-wide query costs here.
        let groups = doc.session.dimension_groups_on_page(page_index);
        let suggestion = suggest_scale_for_groups(&model, &groups);
        let options = seeded_options(remembered, &suggestion);
        let calibrated = calibrated_groups(&model);
        let ratio = Ratio::of_scale(options.scale, options.units, None);

        let dialog = Self {
            page_index,
            suggestion,
            options,
            groups: calibrated,
            ratio,
            export_requested: false,
            close_requested: false,
        };

        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-dxf-open page={} groups={} suggestion={} scale={} units={} arcs={} text={} version={}",
                dialog.page_index,
                groups.len(),
                // Stable lowercase tokens, never `{:?}` — see
                // [`suggestion_key`], which states at length why the old
                // `{suggestion:?}` could not be asserted on.
                suggestion_key(&dialog.suggestion),
                dialog.options.scale,
                crate::app::prefs::exporting::dxf_units_key(dialog.options.units),
                u8::from(dialog.options.fit_arcs),
                crate::app::prefs::exporting::dxf_text_key(dialog.options.text),
                dialog.options.version.acadver(),
            )
        });
        dialog
    }

    /// **This window's state, reduced to what a different document would still
    /// want** — the producing half of `OPERATOR_REQUESTS.md` **O196**.
    fn habits(&self) -> crate::app::prefs::ExportDxfPrefs {
        crate::app::prefs::ExportDxfPrefs {
            units: self.options.units,
            fit_arcs: self.options.fit_arcs,
            text: self.options.text,
            version: self.options.version,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
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
            // O196, and the POSITION is the decision: the habits are
            // written when the operator presses Export, never when the window
            // closes. Closing without exporting is how a person says *"not
            // this"*. The argument is in
            // [`crate::dialogs::export_remembered`], stated once for all three
            // export windows.
            crate::dialogs::export_remembered::remember_dxf(self.habits(), prefs);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf-requested page={} scale={} units={} arcs={} text={} version={}",
                    self.page_index,
                    self.options.scale,
                    // Tokens, never `{:?}`: the same reduction the preferences
                    // file performs, so a check reading this line and a check
                    // reading the file cannot disagree.
                    crate::app::prefs::exporting::dxf_units_key(self.options.units),
                    u8::from(self.options.fit_arcs),
                    crate::app::prefs::exporting::dxf_text_key(self.options.text),
                    self.options.version.acadver(),
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
        let before = (self.options.scale, self.options.units);
        self.scale_disclosure(ui);
        if (self.options.scale, self.options.units) != before {
            self.ratio = Ratio::of_scale(self.options.scale, self.options.units, None);
        }
        self.group_row(ui);
        self.ratio_row(ui);
        ui.add_space(8.0);

        // --- units --------------------------------------------------------
        ui.label(t::units_heading());
        let start = ui.cursor();
        ui.horizontal(|ui| {
            for option in [DxfUnits::Millimetres, DxfUnits::Inches] {
                let response =
                    ui.radio_value(&mut self.options.units, option, t::units_name(option));
                // Each radio's OWN rectangle, for O196 — see
                // [`region_for_units`].
                crate::diag::ui_rect(region_for_units(option), response.rect);
            }
        });
        crate::diag::ui_rect(REGION_UNITS, start.union(ui.cursor()));
        ui.add_space(8.0);

        // --- version ------------------------------------------------------
        ui.horizontal(|ui| {
            ui.label(t::version_label());
            let combo = egui::ComboBox::from_id_salt(REGION_VERSION)
                .selected_text(t::version_name(self.options.version))
                .show_ui(ui, |ui| {
                    for version in VERSIONS {
                        ui.selectable_value(
                            &mut self.options.version,
                            version,
                            t::version_name(version),
                        );
                    }
                });
            crate::diag::ui_rect(REGION_VERSION, combo.response.rect);
        });
        ui.weak(t::version_hint(self.options.version));
        ui.add_space(8.0);

        // --- geometry -----------------------------------------------------
        ui.label(t::geometry_heading());
        let response = ui.checkbox(&mut self.options.fit_arcs, t::fit_arcs());
        crate::diag::ui_rect(REGION_ARCS, response.rect);
        ui.weak(t::fit_arcs_hint());

        // `DxfText` is a two-state enum and is presented as a checkbox,
        // because "write the text or not" is what the operator is deciding and
        // a radio pair would spend two rows saying it. Read and written through
        // the enum rather than mirrored into a local `bool`: a shadow copy is
        // how a window comes to show one thing and write another.
        let mut write_text = matches!(self.options.text, DxfText::Entities);
        let response = ui.checkbox(&mut write_text, t::write_text());
        crate::diag::ui_rect(REGION_TEXT, response.rect);
        if response.changed() {
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

    /// The picker over every calibrated scale group in the document. Absent
    /// when the document has none: an empty picker offers nothing.
    fn group_row(&mut self, ui: &mut Ui) {
        if self.groups.is_empty() {
            return;
        }
        let current = self.groups.iter().position(|g| {
            (g.scale - self.options.scale).abs() <= 1e-9 * g.scale.max(1.0)
                && DxfUnits::for_unit(g.unit) == self.options.units
        });
        let mut picked = None;
        ui.horizontal(|ui| {
            ui.label(t::group_label());
            let selected =
                current.map_or_else(|| t::group_typed().to_owned(), |i| self.groups[i].label());
            let combo = egui::ComboBox::from_id_salt(REGION_GROUP)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (i, group) in self.groups.iter().enumerate() {
                        if ui
                            .selectable_label(current == Some(i), group.label())
                            .clicked()
                        {
                            picked = Some(i);
                        }
                    }
                });
            let response = combo.response.on_hover_text(t::group_hover());
            crate::diag::ui_rect(REGION_GROUP, response.rect);
        });
        if let Some(i) = picked {
            let group = &self.groups[i];
            self.options.scale = group.scale;
            self.options.units = DxfUnits::for_unit(group.unit);
            self.ratio = Ratio::of_scale(group.scale, self.options.units, Some(group.unit));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf-group-picked group={} scale={}",
                    self.groups[i].name, self.options.scale
                )
            });
        }
    }

    /// `1 [in] on paper = 20 [ft] in reality`, the Set Scale window's row.
    /// Every edit rewrites `options.scale`; a world-unit change also moves
    /// the DXF units to that unit's measurement system.
    fn ratio_row(&mut self, ui: &mut Ui) {
        let before = self.ratio;
        ui.horizontal(|ui| {
            ui.label(t::ratio_label());
            let (paper, refusal) = entry::drag_value_unlabelled(
                ui,
                &mut self.ratio.paper,
                entry::Kind::Length(entry::LengthUnit::Of(self.ratio.basis)),
            );
            let _ = refusal.show(ui.add(paper.speed(0.01).range(0.0001..=10_000.0)));
            let _ = unit_combo(ui, BASIS_ID, &mut self.ratio.basis);
            ui.label(crate::text::scale::ratio_separator());
            let (real, refusal) = entry::drag_value_unlabelled(
                ui,
                &mut self.ratio.real,
                entry::Kind::Length(entry::LengthUnit::Of(self.ratio.real_unit)),
            );
            // Positive and finite: a zero or negative scale collapses or
            // mirrors the geometry, and no reading of one is meant.
            let real = refusal.show(ui.add(real.speed(0.01).range(0.0001..=1_000_000.0)));
            crate::diag::ui_rect(REGION_SCALE, real.rect);
            let _ = unit_combo(ui, REAL_UNIT_ID, &mut self.ratio.real_unit);
            ui.label(crate::text::scale::ratio_real_suffix());
        });
        ui.label(egui::RichText::new(t::ratio_hint()).small().weak());
        if self.ratio != before {
            if let Some(scale) = self.ratio.scale() {
                self.options.scale = scale;
            }
            if self.ratio.real_unit != before.real_unit {
                self.options.units = DxfUnits::for_unit(self.ratio.real_unit);
            }
        }
    }

    /// What pdfcer inferred, and what the operator should make of it.
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
                // Every candidate offered, none pre-selected. Selecting one
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

/// Widget ids of the ratio row's two unit pickers.
const BASIS_ID: &str = "export-dxf.basis"; // ui-text-exempt: widget id, never displayed
const REAL_UNIT_ID: &str = "export-dxf.real-unit"; // ui-text-exempt: widget id, never displayed

/// A unit picker over every unit a scale group can measure in.
fn unit_combo(ui: &mut Ui, id: &str, unit: &mut Unit) -> egui::Rect {
    egui::ComboBox::from_id_salt(id)
        .selected_text(crate::text::scale::unit_name(*unit))
        .show_ui(ui, |ui| {
            for option in Unit::all().iter().copied() {
                ui.selectable_value(unit, option, crate::text::scale::unit_name(option));
            }
        })
        .response
        .rect
}

/// One calibrated scale group, as the picker offers it.
struct GroupScale {
    /// The group's name.
    name: String,
    /// Real-world units per paper unit — `DxfOptions::scale`.
    scale: f64,
    /// The unit the group measures in; the ratio row's world side.
    unit: Unit,
}

impl GroupScale {
    fn label(&self) -> String {
        let ratio = Ratio::of_scale(self.scale, DxfUnits::for_unit(self.unit), Some(self.unit));
        t::group_choice(
            &self.name,
            crate::text::scale::unit_name(ratio.basis),
            ratio.real,
            crate::text::scale::unit_name(ratio.real_unit),
        )
    }
}

/// Every group in `model` with a scale set, in the model's order. Each is
/// asked of the engine alone, so its scale is the engine's arithmetic.
fn calibrated_groups(model: &pdfcer_core::dimension::DimensionModel) -> Vec<GroupScale> {
    model
        .groups()
        .iter()
        .filter_map(|group| match suggest_scale_for_groups(model, &[group.id]) {
            DxfScaleSuggestion::Calibrated { scale, .. } => Some(GroupScale {
                name: group.name.clone(),
                scale,
                unit: group.format.unit,
            }),
            _ => None,
        })
        .collect()
}

/// The ratio row's four fields: `paper basis = real real_unit`.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Ratio {
    paper: f64,
    basis: Unit,
    real: f64,
    real_unit: Unit,
}

impl Ratio {
    /// One paper unit of `units`' system (inch or millimetre) against its
    /// real-world length in `world`, or in the paper unit when `None`.
    fn of_scale(scale: f64, units: DxfUnits, world: Option<Unit>) -> Self {
        let basis = match units {
            DxfUnits::Inches => Unit::Inch,
            DxfUnits::Millimetres => Unit::Millimeter,
        };
        let real_unit = world.unwrap_or(basis);
        Self {
            paper: 1.0,
            basis,
            real: scale * real_unit.baseline_per_point() / basis.baseline_per_point(),
            real_unit,
        }
    }

    /// Real-world units per paper unit, or `None` when the row is degenerate.
    fn scale(&self) -> Option<f64> {
        let world = self.real / self.real_unit.baseline_per_point();
        let paper = self.paper / self.basis.baseline_per_point();
        let scale = world / paper;
        (scale.is_finite() && scale > 0.0).then_some(scale)
    }
}

/// The two seedings, in the order that matters — **the operator's habit
/// first, the page's own calibration second**.
#[must_use]
pub fn seeded_options(
    remembered: &crate::app::prefs::ExportDxfPrefs,
    suggestion: &DxfScaleSuggestion,
) -> DxfOptions {
    // ① The operator's habit. Every field `ExportDxfPrefs` carries, and no
    //    field it does not — `scale` and `arc_tolerance` stay at the engine's
    //    defaults here, and `scale` is then overwritten below where there is a
    //    measurement for it.
    //
    //    Written as a functional update rather than three assignments after a
    //    `default()`, because `clippy::field_reassign_with_default` is right
    //    about this one: the three-assignment form leaves a window in which
    //    `options` holds the engine's units rather than the operator's, and
    //    the whole subject of this function is which value is in that field
    //    when. Stating the habit AT construction means step ② below is the
    //    only overwrite in the function, which is exactly the claim
    //    `a_calibrated_page_overrules_the_remembered_units` makes.
    let mut options = DxfOptions {
        units: remembered.units,
        fit_arcs: remembered.fit_arcs,
        text: remembered.text,
        version: remembered.version,
        ..DxfOptions::default()
    };
    // ② The page's own calibration, which overrules the habit. A candidate is a
    //    group's WHOLE opinion — a 1:50 metre group and a 1:50 inch group are
    //    different answers wearing the same number — so the units come with the
    //    scale or neither does.
    if let DxfScaleSuggestion::Calibrated { scale, units, .. } = suggestion {
        options.scale = *scale;
        options.units = *units;
    }
    options
}

/// Open the window for `status`, or decline.
#[must_use]
pub fn open_for(
    status: &Status,
    remembered: &crate::app::prefs::ExportDxfPrefs,
) -> Option<ExportDxfDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(ExportDxfDialog::open(doc, remembered)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests — the ordering rule, and the trace token that makes it observable
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::prefs::ExportDxfPrefs;

    /// **A title-block ratio becomes the engine's dimensionless scale**, and
    /// back. 1 in = 20 ft is 240 inches per inch; 1 mm = 100 mm is 100.
    #[test]
    fn a_title_block_ratio_is_the_engines_scale() {
        let feet = Ratio {
            paper: 1.0,
            basis: Unit::Inch,
            real: 20.0,
            real_unit: Unit::DecimalFeet,
        };
        assert!((feet.scale().unwrap() - 240.0).abs() < 1e-9);
        let plain = Ratio {
            paper: 1.0,
            basis: Unit::Millimeter,
            real: 100.0,
            real_unit: Unit::Millimeter,
        };
        assert!((plain.scale().unwrap() - 100.0).abs() < 1e-9);
        let back = Ratio::of_scale(240.0, DxfUnits::Inches, Some(Unit::DecimalFeet));
        assert!((back.real - 20.0).abs() < 1e-9, "{back:?}");
        let zero = Ratio { real: 0.0, ..plain };
        assert_eq!(zero.scale(), None);
    }

    /// A calibrated answer: **inches**, at half scale, from a named group.
    ///
    /// Inches deliberately, because [`habits`] remembers millimetres. The two
    /// must disagree or nothing below can tell which of them won.
    fn calibrated_inches() -> DxfScaleSuggestion {
        DxfScaleSuggestion::Calibrated {
            scale: 0.5,
            units: DxfUnits::Inches,
            group: "plan".to_string(), // ui-text-exempt: test fixture, never displayed
            agreeing: 2,
        }
    }

    /// Habits that differ from the engine's defaults on **every** field.
    fn habits() -> ExportDxfPrefs {
        let engine = DxfOptions::default();
        let habits = ExportDxfPrefs {
            units: DxfUnits::Millimetres,
            fit_arcs: false,
            text: DxfText::Omit,
            version: DxfVersion::R12,
        };
        assert_ne!(
            habits.units, engine.units,
            "the fixture's units equal the engine's default, so these tests cannot distinguish a seeded window from an unseeded one"
        );
        assert_ne!(
            habits.fit_arcs, engine.fit_arcs,
            "the fixture's fit_arcs equals the engine's default, so the arcs assertions below are vacuous"
        );
        assert_ne!(
            habits.text, engine.text,
            "the fixture's text equals the engine's default, so the text assertions below are vacuous"
        );
        assert_ne!(
            habits.version, engine.version,
            "the fixture's version equals the engine's default, so the version assertion below is vacuous"
        );
        habits
    }

    /// **An uncalibrated page opens on the operator's habit.**
    ///
    /// O196's whole point — *"the export windows forget everything. every time
    /// I export a dxf I have to set it up again."* — on the window he named.
    #[test]
    fn an_uncalibrated_page_opens_on_the_remembered_habit() {
        let options = seeded_options(&habits(), &DxfScaleSuggestion::Uncalibrated);
        assert_eq!(options.units, DxfUnits::Millimetres);
        assert!(!options.fit_arcs);
        assert_eq!(options.text, DxfText::Omit);
        assert_eq!(options.version, DxfVersion::R12);
        // `scale` is deliberately NOT a remembered field, and there is no
        // measurement here to supply one, so it must stay where the engine put
        // it. A remembered scale would be a number from another drawing
        // presented as though it were about this one.
        assert!(
            (options.scale - DxfOptions::default().scale).abs() < f64::EPSILON,
            "an uncalibrated page must open on the engine's own scale, never on a figure carried over from a previous export"
        );
    }

    /// **A calibrated ce dimension group overrules the remembered units**,
    /// and this is the assertion the rest of O196 is dangerous without.
    #[test]
    fn a_calibrated_page_overrules_the_remembered_units() {
        let habits = habits();
        let suggestion = calibrated_inches();
        let DxfScaleSuggestion::Calibrated {
            units: measured, ..
        } = &suggestion
        else {
            unreachable!("the fixture is calibrated") // ui-text-exempt: test panic, never displayed
        };
        // The falsification guard. If the habit and the measurement agreed,
        // this test would pass under BOTH orderings and would assert nothing
        // while reading exactly like a test that did.
        assert_ne!(
            habits.units, *measured,
            "the habit and the measurement agree, so this test cannot tell which of them won"
        );

        let options = seeded_options(&habits, &suggestion);
        assert_eq!(
            options.units,
            DxfUnits::Inches,
            "the page's own calibration must beat the operator's habit — a habit is a statement about the operator, a calibration is a statement about the page"
        );
        assert!(
            (options.scale - 0.5).abs() < f64::EPSILON,
            "the measured scale must survive the seeding"
        );
        // The other two habits are untouched by a calibration, which speaks
        // only about units and scale. A seeding that reset them would be O196
        // half-undone by its own guard, and every assertion above would still
        // pass.
        assert!(!options.fit_arcs);
        assert_eq!(options.text, DxfText::Omit);
    }

    /// **A conflicting page leaves the habit standing.**
    #[test]
    fn a_conflicting_page_leaves_the_habit_standing() {
        let options = seeded_options(
            &habits(),
            &DxfScaleSuggestion::Conflicting {
                candidates: Vec::new(),
            },
        );
        assert_eq!(options.units, DxfUnits::Millimetres);
        assert!(
            (options.scale - DxfOptions::default().scale).abs() < f64::EPSILON,
            "a conflicting page must not seed a scale — picking one would be pdfcer answering a question it has just said it cannot answer, and the operator would find a plausible number already in the box"
        );
    }

    /// **Every kind of suggestion has its own stable, lowercase, payload-free
    /// token.**
    #[test]
    fn every_suggestion_kind_has_its_own_stable_token() {
        let kinds = [
            calibrated_inches(),
            DxfScaleSuggestion::Uncalibrated,
            DxfScaleSuggestion::Conflicting {
                candidates: Vec::new(),
            },
        ];
        let mut seen: Vec<&str> = Vec::new();
        for kind in &kinds {
            let token = suggestion_key(kind);
            assert!(!token.is_empty(), "an empty token discloses nothing");
            assert_eq!(
                token.to_lowercase(),
                token,
                "`{token}` is not lowercase, and `check-trace-names.py` forbids an uppercase trace token"
            );
            assert!(
                !token.contains('{') && !token.contains(' '),
                "`{token}` carries payload shape, which is exactly what the `Debug` formatting it replaced did wrong"
            );
            assert!(
                !seen.contains(&token),
                "`{token}` is used by two different kinds, so the trace cannot tell them apart"
            );
            seen.push(token);
        }
        assert_eq!(seen.len(), 3);
    }
}
