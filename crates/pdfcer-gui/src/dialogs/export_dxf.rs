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
/// ★★ The region ONE units radio publishes.
///
/// ★ These exist for `OPERATOR_REQUESTS.md` **O196**. Until this window
/// remembered anything, a driven check had nothing to assert about a radio
/// beyond *"it is drawn"*; now the question is which one is **selected on
/// open**, and that cannot be asked of a group rectangle.
///
/// It matters more here than anywhere else in the three export windows: the
/// units radio is the control that is wrong by 25.4× when it is wrong, and the
/// resulting DXF opens cleanly and measures consistently.
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
/// The region the fit-arcs checkbox publishes.
pub const REGION_ARCS: &str = "export-dxf.arcs"; // ui-text-exempt: trace region name, never displayed
/// The region the write-text checkbox publishes.
pub const REGION_TEXT: &str = "export-dxf.text"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-dxf.export"; // ui-text-exempt: trace region name, never displayed

/// ★★ Which of the three answers `suggest_scale_for_groups` gave, as a stable
/// lowercase token.
///
/// # Why this exists rather than a `{:?}` on the suggestion
///
///
/// **It is this project's standing lesson about `Debug` in a machine-read
/// field.** `Calibrated` carries a scale, a unit, a group name and an agreement
/// count; `Conflicting` carries a whole vector of candidates. A check grepping
/// for `suggestion=Calibrated` misses `suggestion=Calibrated { scale: 0.5, …`
/// **while quoting the truth in its own failure message** — a confident false
/// negative that reads as an application defect.
///
/// **And the payload varies with the document.** A trace whose text depends on
/// which ce dimension groups happen to be on the page is a trace a check cannot
/// assert on at all, which in practice means the check asserts only that the
/// window opened — a claim satisfied by every build ever shipped, including the
/// one O196 exists to replace.
///
/// ⇒ The *kind* is what a check needs, because the kind is what decides whether
/// the scale field was seeded by a measurement or left at the operator's habit.
/// The numbers are already on the same line, spelled as numbers.
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
    ///
    ///
    /// > *"the export windows forget everything. **every time I export a dxf I
    /// > have to set it up again.**"*
    ///
    /// This is the window he named. Three of its four answers — units, arcs,
    /// text — are now seeded from the last DXF export; the fourth, the scale,
    /// deliberately is **not** a preference at all, and the argument for that
    /// omission is the most important sentence in
    /// [`crate::app::prefs::ExportDxfPrefs`]'s module. Read it there.
    ///
    /// # ★★★ THE ORDERING RULE, and it is the one part of O196 that can be
    /// wrong by 25.4× and silent
    ///
    /// The operator's habit is written **first**; a calibrated ce dimension
    /// group on this page overwrites it **second**.
    ///
    /// Both halves are load-bearing and they are not symmetrical:
    ///
    /// - A habit is a statement about *the operator* — the units their
    ///   downstream tool wants. It is the right answer on every page that has
    ///   nothing better to offer, which is most pages.
    /// - A calibration is a statement about *this page* — the units the drawing
    ///   was actually dimensioned in, measured from ce dimensions the operator
    ///   drew themselves. Where it exists it is not an opinion, and a
    ///   remembered habit must not be allowed to beat it.
    ///
    /// ⇒ Getting the two lines the other way round would let a habit of inches
    /// silently export a metric-calibrated page at 25.4× — a DXF that opens
    /// cleanly, measures consistently, and is wrong, discovered by whoever cuts
    /// from it. That is the exact failure this window was built to prevent,
    /// arriving through the door O196 opened.
    ///
    /// `dialogs::export_dxf::tests::a_calibrated_page_overrules_the_remembered_units`
    /// is the guard, and it exists because this rule is two adjacent
    /// assignments whose order nothing else enforces.
    #[must_use]
    pub fn open(doc: &OpenDoc, remembered: &crate::app::prefs::ExportDxfPrefs) -> Self {
        let page_index = doc.view.page_index;
        let model = doc.session.dimension_model();
        // ★ The page's OWN groups. See the module header for what the
        // document-wide query costs here.
        let groups = doc.session.dimension_groups_on_page(page_index);
        let suggestion = suggest_scale_for_groups(&model, &groups);
        let options = seeded_options(remembered, &suggestion);

        let dialog = Self {
            page_index,
            suggestion,
            options,
            export_requested: false,
            close_requested: false,
        };

        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-dxf-open page={} groups={} suggestion={} scale={} units={} arcs={} text={}",
                dialog.page_index,
                groups.len(),
                // ★ Stable lowercase tokens, never `{:?}` — see
                // [`suggestion_key`], which states at length why the old
                // `{suggestion:?}` could not be asserted on.
                suggestion_key(&dialog.suggestion),
                dialog.options.scale,
                crate::app::prefs::exporting::dxf_units_key(dialog.options.units),
                u8::from(dialog.options.fit_arcs),
                crate::app::prefs::exporting::dxf_text_key(dialog.options.text),
            )
        });
        dialog
    }

    /// **This window's state, reduced to what a different document would still
    /// want** — the producing half of `OPERATOR_REQUESTS.md` **O196**.
    ///
    /// The membership rule and the argument for every inclusion and every
    /// omission live on [`crate::app::prefs::ExportDxfPrefs`], which is the type
    /// this returns; this function is only the projection. It is one struct
    /// literal with **no `..Default::default()`**, so a field added to
    /// `ExportDxfPrefs` is a compile error here rather than a preference written
    /// to disk as its own default and never actually remembered.
    ///
    /// ⚠ **`scale` is not here, and it is not an oversight.** It is the one
    /// field of `DxfOptions` this window edits that must be derived per page
    /// rather than carried between them.
    fn habits(&self) -> crate::app::prefs::ExportDxfPrefs {
        crate::app::prefs::ExportDxfPrefs {
            units: self.options.units,
            fit_arcs: self.options.fit_arcs,
            text: self.options.text,
        }
    }

    /// Draw it. Returns `false` when it should close.
    ///
    /// Takes `&mut Prefs` for O196 alone: the Export press writes this window's
    /// habits to the preferences file before the action is pushed. See
    /// [`crate::dialogs::export_remembered`] for why it happens at the press
    /// and not at the close.
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
            // ★★★ O196, and the POSITION is the decision: the habits are
            // written when the operator presses Export, never when the window
            // closes. Closing without exporting is how a person says *"not
            // this"*. The argument is in
            // [`crate::dialogs::export_remembered`], stated once for all three
            // export windows.
            crate::dialogs::export_remembered::remember_dxf(self.habits(), prefs);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf-requested page={} scale={} units={} arcs={} text={}",
                    self.page_index,
                    self.options.scale,
                    // Tokens, never `{:?}`: the same reduction the preferences
                    // file performs, so a check reading this line and a check
                    // reading the file cannot disagree.
                    crate::app::prefs::exporting::dxf_units_key(self.options.units),
                    u8::from(self.options.fit_arcs),
                    crate::app::prefs::exporting::dxf_text_key(self.options.text),
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
        let start = ui.cursor();
        ui.horizontal(|ui| {
            for option in [DxfUnits::Millimetres, DxfUnits::Inches] {
                let response =
                    ui.radio_value(&mut self.options.units, option, t::units_name(option));
                // ★ Each radio's OWN rectangle, for O196 — see
                // [`region_for_units`].
                crate::diag::ui_rect(region_for_units(option), response.rect);
            }
        });
        crate::diag::ui_rect(REGION_UNITS, start.union(ui.cursor()));
        ui.add_space(8.0);

        // --- geometry -----------------------------------------------------
        ui.label(t::geometry_heading());
        let response = ui.checkbox(&mut self.options.fit_arcs, t::fit_arcs());
        crate::diag::ui_rect(REGION_ARCS, response.rect);
        ui.weak(t::fit_arcs_hint());

        // ★ `DxfText` is a two-state enum and is presented as a checkbox,
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

/// ★★★ The two seedings, in the order that matters — **the operator's habit
/// first, the page's own calibration second**.
///
/// Lifted out of [`ExportDxfDialog::open`] so the ordering rule has something a
/// unit test can call. `open` needs an [`OpenDoc`] and therefore an
/// `EditSession` and therefore a real document, which is exactly the amount of
/// scaffolding that stops the one rule in this file worth a test from having
/// one. Here it is a pure function of two values.
///
/// The argument for the order is on [`ExportDxfDialog::open`] and is not
/// repeated; in one sentence: **a habit is a statement about the operator and a
/// calibration is a statement about the page, so the page wins where it speaks
/// at all.**
///
/// `Conflicting` is deliberately NOT seeded from the first candidate. Picking
/// one would be pdfcer answering a question it has just said it cannot answer,
/// and the operator would find a plausible number already in the box. Note what
/// that means for the units: on a conflicting page the operator's remembered
/// units survive, because nothing has overruled them — the window then asks
/// which candidate, and choosing one writes both halves of that candidate's
/// opinion.
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
    ///
    /// ★ The anti-vacuity fixture, and it is asserted rather than trusted. A
    /// `seeded_options` that ignored `remembered` entirely and returned
    /// `DxfOptions::default()` would satisfy every assertion below if the
    /// fixture happened to equal that default. So each field is checked
    /// against the engine's own value here, once, and the whole module fails
    /// loudly if the engine ever moves a default onto this fixture.
    ///
    /// That is this project's standing lesson about a check whose input was
    /// chosen for convenience: what is fed in is part of the assertion.
    fn habits() -> ExportDxfPrefs {
        let engine = DxfOptions::default();
        let habits = ExportDxfPrefs {
            units: DxfUnits::Millimetres,
            fit_arcs: false,
            text: DxfText::Omit,
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
        // `scale` is deliberately NOT a remembered field, and there is no
        // measurement here to supply one, so it must stay where the engine put
        // it. A remembered scale would be a number from another drawing
        // presented as though it were about this one.
        assert!(
            (options.scale - DxfOptions::default().scale).abs() < f64::EPSILON,
            "an uncalibrated page must open on the engine's own scale, never on a figure carried over from a previous export"
        );
    }

    /// ★★★ **A calibrated ce dimension group overrules the remembered units**,
    /// and this is the assertion the rest of O196 is dangerous without.
    ///
    /// The operator habitually exports millimetres. This page was dimensioned
    /// in inches and pdfcer can prove it. If the habit won, the DXF would be
    /// out by 25.4× — and it would open cleanly, measure consistently, and be
    /// wrong, which is the failure this module exists to prevent. The person
    /// who discovers it is whoever cuts from the file.
    ///
    /// **Before O196 this could not happen**, because there was no habit: the
    /// window always started from `DxfOptions::default()` and a calibration
    /// only ever overwrote a default. Remembering is what created the ordering
    /// question, so remembering is what owes it a test.
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
        // ★ The falsification guard. If the habit and the measurement agreed,
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
        // ★ The other two habits are untouched by a calibration, which speaks
        // only about units and scale. A seeding that reset them would be O196
        // half-undone by its own guard, and every assertion above would still
        // pass.
        assert!(!options.fit_arcs);
        assert_eq!(options.text, DxfText::Omit);
    }

    /// **A conflicting page leaves the habit standing.**
    ///
    /// Nothing has overruled it: the window is about to say it cannot choose,
    /// and a window that cannot choose a scale has not thereby learned
    /// anything about units either.
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
    ///
    /// The `export-dxf-open` trace is machine-read by `ui-verify`, and
    /// [`suggestion_key`] records at length why a `{suggestion:?}` was not good
    /// enough: the `Debug` payload carries a group name that varies with the
    /// document, so a substring check degrades to *"the window opened"* — a
    /// claim satisfied by every build ever shipped, including the one O196
    /// exists to replace.
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
