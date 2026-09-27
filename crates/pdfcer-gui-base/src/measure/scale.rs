//! # `measure::scale` — scale entry, and the dimension-group actions
//!
//! **Salvaged** from the old shell's `measure_tool.rs`
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\measure_tool.rs`, Pass 12.M2b), split at
//! that file's own section banners — see [`super::pick`] for the pick state
//! machines and [`super::state`] for the tool-entry container. The reasoning
//! below is the original's, carried across intact.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/measure/scale.md`.

use pdfcer_core::dimension::{
    FractionMode, LengthParseError, NumberFormat, ScaleEntry, ScalePreview, ScaleState, Unit,
    parse_length, preview_group_scale,
};
use pdfcer_core::vector::Point;

use super::pick::LinearPick;

// ---------------------------------------------------------------------------
// Scale entry — the two co-equal back-calc paths (ui-spec §4.2/§4.5)
// ---------------------------------------------------------------------------

/// The scale-entry sub-panel's working fields (ui-spec §4.2), shared by the
/// [`ScalePick`] dialog and the group-panel inline editor (ui-spec §5.2: ONE
/// scale-entry UI in the whole app). Two co-equal paths, one clearly
/// recommended:
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleEntryFields {
    /// `true` ⇒ the real-length path is selected (the recommended default);
    /// `false` ⇒ the direct-ratio path (ui-spec §4.2 `selectable_value`).
    pub use_real_length: bool,
    /// The typed real-world length for the real-length path, in [`Self::unit`].
    ///
    /// Derived from [`Self::real_length_text`] whenever that parses; it is the
    /// number the scale maths actually uses. Kept as the parsed value rather
    /// than re-parsing at commit time so that what the operator was SHOWN in
    /// the preview is definitionally what gets committed.
    pub real_length: f64,
    /// What the operator literally typed for the real length.
    ///
    /// A text field, not a numeric spinner, because the whole point of the
    /// scale-by-known-dimension workflow is to type the dimension as the
    /// drawing writes it — `55 5/8"`, `4'-7 1/2"`. A spinner forced the
    /// operator to convert to a decimal and pick a unit by hand, which is two
    /// chances to enter a number that is plausible and wrong. Parsed by
    /// [`pdfcer_core::dimension::parse_length`].
    pub real_length_text: String,
    /// The unit the real length is typed in / the ratio resolves to (becomes
    /// the group's top unit).
    pub unit: Unit,
    /// The paper side of the direct ratio (`1` in `1:100`).
    pub ratio_paper: f64,
    /// The real side of the direct ratio (`100` in `1:100`), in
    /// [`Self::real_unit`].
    pub ratio_real: f64,
    /// The unit [`Self::ratio_paper`] is measured in on the sheet (default
    /// [`Unit::Inch`]; PDF paper units are 1/72", disclosed — ui-spec §4.2).
    pub basis: Unit,
    /// The unit [`Self::ratio_real`] is measured in, so a drawing's stated
    /// `1" = 20'-0"` is typed as it is written. Equal to [`Self::basis`] makes
    /// the ratio unitless, `1 : 100`.
    pub real_unit: Unit,
    /// How the fractional part of every label in this group is displayed
    /// (Pass 25.5).
    ///
    /// `None` means "whatever the unit's default is" — the behaviour before
    /// this field existed, and still the right answer for an operator who
    /// never opens the display controls. `Some` is an explicit choice that
    /// must survive a unit change, which is why it is stored rather than
    /// re-derived from the unit each time.
    ///
    /// Exists because the operator asked for it directly: *"also want to be
    /// able to choose the units and display type - rounding, fraction, etc."*
    /// The unit was already selectable; the display type was hardcoded to
    /// `Unit::default_format()` at commit, so a drawing dimensioned in inches
    /// always read `55.63"` and could never read `55 5/8"` — the notation the
    /// drawing itself uses.
    pub fraction: Option<FractionMode>,
}

impl Default for ScaleEntryFields {
    fn default() -> Self {
        Self {
            // Real-length is the recommended, pre-selected path (ui-spec §4.2).
            use_real_length: true,
            real_length: 1.0,
            real_length_text: "1".to_owned(),
            unit: Unit::Meter,
            ratio_paper: 1.0,
            ratio_real: 100.0,
            basis: Unit::Inch,
            real_unit: Unit::Inch,
            fraction: None,
        }
    }
}

impl ScaleEntryFields {
    /// Re-read [`Self::real_length_text`], updating the parsed value and
    /// (when the text named one) the unit.
    pub fn sync_real_length(&mut self) -> Option<LengthParseError> {
        match parse_length(&self.real_length_text, self.unit) {
            Ok(p) => {
                self.real_length = p.value;
                if p.unit_from_text {
                    self.unit = p.unit;
                }
                None
            }
            Err(e) => Some(e),
        }
    }

    /// Fields seeded for a group-panel editor where NO reference line was
    /// drawn: the ratio path is the only usable one (the real-length path
    /// needs a drawn length), so it is pre-selected (ui-spec §7.2).
    #[must_use]
    pub fn for_group_panel() -> Self {
        Self {
            use_real_length: false,
            ..Self::default()
        }
    }

    /// **Fields seeded from a group's EXISTING scale** -- O192.
    ///
    /// # The defect this closes
    ///
    /// The Set-scale window is the one surface whose entire job is to change a
    /// number, and until 2026-09-13 it was the only surface in the application
    /// that could not see the number it was about to change. Both of the other
    /// constructors seed from nothing, so a group calibrated to `1:50` in
    /// inches opened a window reading `1:100` in metres, and pressing Accept
    /// without touching a control silently recalibrated the drawing. The
    /// operator's words were *"the set scale dialogue does not show me the
    /// scale that is already set"*; the sharper half of the report is the one
    /// he did not have to say, which is that a window pre-filled with a
    /// plausible wrong number is worse than one pre-filled with nothing.
    ///
    /// # What is seeded, and why the RATIO path
    ///
    /// The ratio path, always, whatever path the operator ends up using. A
    /// stored [`ScaleState`] is a single number -- display units per PDF point
    /// -- and it carries no memory of how it was entered. The real-length path
    /// cannot be reconstructed from it even in principle: `25 ft = 42.3 pt`
    /// and `50 ft = 84.6 pt` are the same scale, and choosing one of them to
    /// show would be pdfcer inventing a reference line the operator never
    /// drew. A ratio is the representation that survives the round trip
    /// without adding information, so a ratio is what is seeded.
    ///
    /// [`Self::use_real_length`] is therefore left at whatever the caller's
    /// situation implies and is **not** set here -- see the two call sites in
    /// `pdfcer_gui::dialogs::scale`. Seeding the ratio numbers costs nothing on
    /// the real-length path, because [`Self::entry`] reads only the three
    /// fields belonging to the path it selects.
    ///
    /// # The arithmetic, and why it is exact
    ///
    /// [`preview_group_scale`] builds the ratio path's scale as
    /// `(real / paper) * basis.baseline_per_point()`. Fixing `paper = 1` and
    /// `basis = format.unit` inverts that in one step:
    ///
    /// ```text
    /// real = scale / format.unit.baseline_per_point()
    /// ```
    ///
    /// Setting [`Self::unit`] to `format.unit` as well makes
    /// [`Self::in_display_unit`] the identity (`raw.unit == self.unit`), so
    /// the value that comes back out of [`Self::preview`] is the stored
    /// `scale` with **no** conversion applied to it in either direction. That
    /// is the property [`tests::a_calibrated_group_round_trips_through_the_seed`]
    /// asserts, and it is why this function sets the basis and the display
    /// unit together rather than leaving the basis at its inch default: with
    /// mismatched units the round trip is still arithmetically correct but is
    /// no longer exact in floating point, and an operator who opens the window
    /// and presses Accept without touching anything would move the drawing's
    /// scale by a few parts in 10^16. Nobody would ever see it. It would still
    /// be a document edit nobody asked for.
    ///
    /// # The three tri-state arms
    ///
    /// | stored state | seeded as | why |
    /// |---|---|---|
    /// | [`ScaleState::NeverSet`] | the [`Default`] ratio, untouched | there is no scale to show, and `NO_SCALE_DISCLOSURE` is what the window says instead |
    /// | [`ScaleState::OneToOne`] | `1 : 1` on the unit's own basis | `1:1` back-calculates to `baseline_per_point(unit)`, which is exactly what `effective_scale` answers for this arm |
    /// | [`ScaleState::Calibrated`] | `1 : scale / bpp(unit)` | the inversion above |
    ///
    /// The `NeverSet` arm deliberately does **not** invent a ratio. A group
    /// that has never been calibrated is a real state with a disclosure
    /// sentence the engine owns (`NO_SCALE_DISCLOSURE`), and pre-filling
    /// `1:100` over it would convert *"nobody has said what this drawing is
    /// at"* into *"this drawing is at 1:100"* -- which is the one thing the
    /// tri-state exists to keep distinguishable.
    ///
    /// A degenerate stored scale (zero, infinite, NaN -- unreachable
    /// through this application, reachable through a hand-edited
    /// `/PieceInfo`) falls back to the `NeverSet` seed rather than writing a
    /// non-finite number into a `DragValue`. An `egui::DragValue` holding a
    /// NaN is a control the operator cannot get out of.
    #[must_use]
    pub fn for_group(scale: ScaleState, format: NumberFormat) -> Self {
        let unit = format.unit;
        let base = Self {
            unit,
            basis: unit,
            real_unit: unit,
            // An explicit display choice already made for the group is a
            // choice, and re-offering the unit default over it would quietly
            // revert an operator who asked for eighths. `commit` reads this
            // field the same way.
            fraction: Some(format.fraction),
            ..Self::default()
        };
        let per_point = match scale {
            ScaleState::NeverSet => return base,
            ScaleState::OneToOne => unit.baseline_per_point(),
            ScaleState::Calibrated { .. } => match scale.effective_scale(unit) {
                Some(v) => v,
                // Unreachable: `effective_scale` is `None` only for
                // `NeverSet`, which the arm above already took. Spelled rather
                // than unwrapped because an unreachable panic in a dialog
                // constructor is still a panic in a dialog constructor.
                None => return base,
            },
        };
        let ratio_real = per_point / unit.baseline_per_point();
        if !(ratio_real.is_finite() && ratio_real > 0.0) {
            return base;
        }
        Self {
            ratio_paper: 1.0,
            ratio_real,
            ..base
        }
    }

    /// The [`ScaleEntry`] these fields describe, given the optional drawn
    /// reference length `drawn_pdf_length` (points). Chooses the real-length
    /// path only when it is selected AND a drawn length is available; else the
    /// ratio path (which needs no line). This is what routes the group-panel
    /// (no line) path to Ratio and the [`ScalePick`] (line drawn) path to
    /// whichever the operator picked.
    #[must_use]
    pub fn entry(&self, drawn_pdf_length: Option<f64>) -> ScaleEntry {
        match (self.use_real_length, drawn_pdf_length) {
            (true, Some(drawn)) => ScaleEntry::RealLength {
                drawn_pdf_length: drawn,
                real_length: self.real_length,
                unit: self.unit,
            },
            // The engine's ratio is in one unit, so the real side is carried
            // into the paper side's: `1 in = 20 ft` becomes `1 : 240` inches.
            _ => ScaleEntry::Ratio {
                paper: self.ratio_paper,
                real: self.ratio_real * self.basis.baseline_per_point()
                    / self.real_unit.baseline_per_point(),
                basis: self.basis,
            },
        }
    }

    /// The live scale preview (ui-spec §4.2 "→ scale = 25.0 ft / 42.3 pt"),
    /// via the shipped [`preview_group_scale`] — pure, no mutation. `None`
    /// for a degenerate entry (Accept then shows nothing to commit).
    #[must_use]
    pub fn preview(&self, drawn_pdf_length: Option<f64>) -> Option<ScalePreview> {
        preview_group_scale(self.entry(drawn_pdf_length)).map(|raw| self.in_display_unit(raw))
    }

    /// Re-express an engine preview in the unit the operator asked to **see**
    /// dimensions in.
    fn in_display_unit(&self, raw: ScalePreview) -> ScalePreview {
        if raw.unit == self.unit {
            return raw;
        }
        let factor = self.unit.baseline_per_point() / raw.unit.baseline_per_point();
        ScalePreview {
            scale: raw.scale * factor,
            unit: self.unit,
            ratio_label: raw.ratio_label,
        }
    }

    /// The `(ScaleState, NumberFormat)` this entry commits as, for
    /// `EditSession::set_group_scale` (ui-spec §4.4 re-propagation). The
    /// back-calculated scale becomes [`ScaleState::Calibrated`]; the format is
    /// the entry unit's default (a calibrated group is never "1:1" or
    /// "never-set" — the tri-state's third state). `None` for a degenerate
    /// entry.
    #[must_use]
    pub fn commit(&self, drawn_pdf_length: Option<f64>) -> Option<(ScaleState, NumberFormat)> {
        let preview = self.preview(drawn_pdf_length)?;
        // An explicit display choice wins over the unit's default, and
        // survives a unit change — an operator who asked for eighths does not
        // want them silently reverted by switching from inches to feet.
        let format = match self.fraction {
            Some(fraction) => NumberFormat {
                unit: preview.unit,
                fraction,
                // The marker follows the group's standard, set by
                // `set_group_standard`, not by this dialog.
                decimal_marker: preview.unit.default_format().decimal_marker,
            },
            None => preview.unit.default_format(),
        };
        Some((
            ScaleState::Calibrated {
                scale: preview.scale,
            },
            format,
        ))
    }
}

/// The scale-dimension tool's state (ui-spec §4.1): draw a reference line with
/// the SAME [`super::pick::LinearPick`] mechanic as a linear dimension, then —
/// once both points are picked — switch to the scale-entry dialog
/// ([`Self::fields`]) keyed on the drawn line's length.
#[derive(Debug, Clone, PartialEq)]
pub struct ScalePick {
    /// The reference-line two-point pick (reused verbatim from the linear
    /// tool, ui-spec §4.1 — including H/V/aligned + snapping).
    pub line: LinearPick,
    /// The drawn reference line's measured length (points) once both points
    /// are picked — `Some` switches the property bar to the scale-entry
    /// dialog (ui-spec §4.1). `None` while still drawing the line.
    pub drawn_pdf_length: Option<f64>,
    /// The scale-entry dialog's fields (both paths available here, since a
    /// line was drawn).
    pub fields: ScaleEntryFields,
}

impl Default for ScalePick {
    fn default() -> Self {
        Self::new()
    }
}

impl ScalePick {
    /// A fresh scale pick, awaiting the reference line's first point.
    #[must_use]
    pub fn new() -> Self {
        Self {
            line: LinearPick::reference_line(),
            drawn_pdf_length: None,
            fields: ScaleEntryFields::default(),
        }
    }

    /// Register a committed (snapped) reference-line pick `p`. While the
    /// dialog is open ([`Self::drawn_pdf_length`] is `Some`) further picks are
    /// ignored (the operator is typing the scale, ui-spec §4.1). Otherwise the
    /// pick advances the line; when the line completes, the drawn length is
    /// recorded and the dialog opens. Returns `true` when the dialog just
    /// opened.
    pub fn commit_point(&mut self, p: Point) -> bool {
        if self.drawn_pdf_length.is_some() {
            return false;
        }
        if let Some(kind) = self.line.commit_point(p) {
            // The measured length of the just-drawn reference line, under its
            // own H/V/aligned constraint (kind.measured_points()).
            self.drawn_pdf_length = Some(kind.measured_points());
            true
        } else {
            false
        }
    }

    /// Whether the scale-entry dialog is open (both reference points picked).
    #[must_use]
    pub fn dialog_open(&self) -> bool {
        self.drawn_pdf_length.is_some()
    }

    /// The live scale preview for the current dialog fields + drawn length
    /// (ui-spec §4.2), or `None` if the dialog is closed or the entry is
    /// degenerate.
    #[must_use]
    pub fn preview(&self) -> Option<ScalePreview> {
        self.drawn_pdf_length
            .and_then(|_| self.fields.preview(self.drawn_pdf_length))
    }

    /// The `(ScaleState, NumberFormat)` an Accept commits (ui-spec §4.4),
    /// or `None` while the dialog is closed / the entry is degenerate.
    #[must_use]
    pub fn commit(&self) -> Option<(ScaleState, NumberFormat)> {
        self.drawn_pdf_length
            .and_then(|_| self.fields.commit(self.drawn_pdf_length))
    }

    /// Discard the whole gesture (Escape stage 1 / Reject, ui-spec §1.3):
    /// forget the reference line and close the dialog, keeping the operator's
    /// typed dialog values (so a mis-drawn line is cheap to redo).
    pub fn clear(&mut self) {
        self.line.clear();
        self.drawn_pdf_length = None;
    }

    /// Whether a gesture is in progress (a point picked or the dialog open —
    /// a discardable gesture).
    #[must_use]
    pub fn in_progress(&self) -> bool {
        self.line.in_progress() || self.drawn_pdf_length.is_some()
    }
}

// ---------------------------------------------------------------------------
// Group-panel actions — each maps to exactly one shipped EditSession command
// ---------------------------------------------------------------------------

//
// It was a salvaged, fully-tested, **zero-caller** enum: four variants naming
// four `EditSession` group verbs, carried across whole in the Phase 7 salvage
// and never wired to anything. Its own doc comment recorded, correctly, that
// rename and delete were absent from the engine and deliberately not
// reimplemented here.
//
// What replaced it is `pdfcer_gui::app::actions::dimensions::DimensionAction`,
// which names the same four verbs and four more, and — the part `GroupAction`
// never had — **reaches the document**, through the action funnel, as one undo
// entry per operator gesture. Keeping both would have been two vocabularies for
// one set of verbs, which is the drift this crate has corrected five times.
//
// The deletion is recorded rather than silent because the enum was *evidence*:
// it is why `shell::commands::reach`'s scaffold entry for
// `measure.manage_groups` cited "two of four verbs do not exist" for months.
// That citation was accurate when written and outlived its own subject.

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
mod tests {

    /// **A6: "Show dimensions in" reaches the scale on the ratio path.**
    #[test]
    fn the_display_unit_converts_the_scale_rather_than_relabelling_it() {
        // 1:100 on the default inch basis. One point is 100/72 inch.
        let mut fields = ScaleEntryFields {
            use_real_length: false,
            ratio_paper: 1.0,
            ratio_real: 100.0,
            basis: Unit::Inch,
            unit: Unit::Inch,
            ..ScaleEntryFields::default()
        };

        let (inch_state, inch_format) = fields.commit(None).expect("a ratio commits");
        fields.unit = Unit::Meter;
        let (metre_state, metre_format) = fields.commit(None).expect("a ratio commits");

        assert_eq!(inch_format.unit, Unit::Inch);
        assert_eq!(
            metre_format.unit,
            Unit::Meter,
            "the dropdown must reach the committed format"
        );

        // 720 points is ten inches of paper, so 1000 inches of ground.
        const POINTS: f64 = 720.0;
        let inches = POINTS * inch_state.effective_scale(Unit::Inch).expect("calibrated");
        let metres = POINTS
            * metre_state
                .effective_scale(Unit::Meter)
                .expect("calibrated");
        assert!(
            (inches - 1000.0).abs() < 1e-9,
            "1:100 at 720 pt is 1000 inches; got {inches}"
        );
        // ORACLE, NOT A CONVERSION: 25.4 is the ANSWER here, not a step. 1000
        // inches is 25.4 m because an inch is 0.0254 m by definition, and this
        // assertion's whole value is that it arrived at that number without
        // going through `units.rs` — which is the code path the scale entry
        // itself uses. An assertion routed through the table would be
        // asserting the table equals the table.
        assert!(
            (metres - 25.4).abs() < 1e-9,
            "the SAME distance is 25.4 m. Got {metres} — which is what a build that relabelled \
             the format without converting the scale produces, and it would write that number \
             onto every dimension in the drawing."
        );
    }

    /// The real-length path is unaffected, and that is asserted rather than
    /// assumed: `entry` passes `self.unit` straight through there, so the
    /// conversion must be the identity and must not quietly scale twice.
    #[test]
    fn the_real_length_path_is_untouched_by_the_conversion() {
        let fields = ScaleEntryFields {
            use_real_length: true,
            real_length: 25.0,
            unit: Unit::DecimalFeet,
            ..ScaleEntryFields::default()
        };
        let preview = fields.preview(Some(42.3)).expect("a real-length preview");
        assert_eq!(preview.unit, Unit::DecimalFeet);
        assert!(
            (preview.scale - 25.0 / 42.3).abs() < 1e-12,
            "a 42.3 pt line called 25 ft is 25/42.3 ft per point; got {}",
            preview.scale
        );
    }

    /// The preview the operator READS and the value that gets COMMITTED are
    /// the same number, because `commit` goes through `preview`.
    #[test]
    fn what_the_preview_shows_is_what_the_commit_stores() {
        let fields = ScaleEntryFields {
            use_real_length: false,
            ratio_paper: 1.0,
            ratio_real: 50.0,
            basis: Unit::Inch,
            unit: Unit::Millimeter,
            ..ScaleEntryFields::default()
        };
        let preview = fields.preview(None).expect("a ratio preview");
        let (state, format) = fields.commit(None).expect("and it commits");
        assert_eq!(format.unit, preview.unit);
        assert_eq!(
            state.effective_scale(preview.unit),
            Some(preview.scale),
            "the committed scale must be the previewed one"
        );
    }
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    // ---- display format (Pass 25.5) -------------------------------------

    /// A fields set that commits: real-length path, 100 units over a drawn
    /// line of 200 pt.
    fn calibrated(unit: Unit) -> ScaleEntryFields {
        ScaleEntryFields {
            use_real_length: true,
            real_length: 100.0,
            real_length_text: "100".to_owned(),
            unit,
            ..ScaleEntryFields::default()
        }
    }

    #[test]
    fn with_no_explicit_choice_the_units_default_format_is_used() {
        let f = calibrated(Unit::Inch);
        let (_scale, format) = f.commit(Some(200.0)).expect("commits");
        assert_eq!(
            format,
            Unit::Inch.default_format(),
            "an operator who never opens the display controls must get the \
             unchanged behaviour"
        );
    }

    /// **The operator's ask.** An explicit fraction choice reaches the format.
    #[test]
    fn an_explicit_fraction_choice_is_what_commits() {
        let mut f = calibrated(Unit::Inch);
        f.fraction = Some(FractionMode::Fraction {
            denominator: 16,
            reduce: false,
        });
        let (_scale, format) = f.commit(Some(200.0)).expect("commits");
        assert_eq!(
            format.fraction,
            FractionMode::Fraction {
                denominator: 16,
                reduce: false
            }
        );
        assert_eq!(format.unit, Unit::Inch);
    }

    #[test]
    fn an_explicit_choice_survives_a_unit_change() {
        // Choosing eighths and then switching unit must not silently revert to
        // that unit's default notation — the operator asked for a notation,
        // not for a notation-on-this-unit.
        let mut f = calibrated(Unit::Inch);
        f.fraction = Some(FractionMode::Fraction {
            denominator: 8,
            reduce: true,
        });
        f.unit = Unit::FeetInches;
        let (_scale, format) = f.commit(Some(200.0)).expect("commits");
        assert_eq!(format.unit, Unit::FeetInches);
        assert_eq!(
            format.fraction,
            FractionMode::Fraction {
                denominator: 8,
                reduce: true
            }
        );
    }

    // ---- Scale dialog back-calc plumbing (ui-spec §4) -------------------

    #[test]
    fn scale_pick_draws_a_line_then_opens_the_dialog() {
        let mut sp = ScalePick::new();
        assert!(!sp.dialog_open());
        // First reference point: no dialog yet.
        assert!(!sp.commit_point(p(0.0, 0.0)));
        assert!(!sp.dialog_open());
        // Second reference point: the dialog opens, drawn length recorded.
        assert!(sp.commit_point(p(42.3, 0.0)));
        assert!(sp.dialog_open());
        assert!((sp.drawn_pdf_length.unwrap() - 42.3).abs() < 1e-9);
        // Further picks are ignored while the dialog is open.
        assert!(!sp.commit_point(p(99.0, 99.0)));
    }

    #[test]
    fn scale_real_length_path_back_calcs_via_the_engine() {
        let mut sp = ScalePick::new();
        sp.commit_point(p(0.0, 0.0));
        sp.commit_point(p(42.3, 0.0)); // 42.3 pt reference line
        sp.fields.use_real_length = true;
        sp.fields.real_length = 25.0;
        sp.fields.unit = Unit::DecimalFeet;
        let preview = sp.preview().expect("a real-length preview");
        assert!((preview.scale - 25.0 / 42.3).abs() < 1e-12);
        assert_eq!(preview.unit, Unit::DecimalFeet);
        // Commit resolves to a Calibrated tri-state + the unit's default format.
        let (state, format) = sp.commit().unwrap();
        assert!(matches!(state, ScaleState::Calibrated { .. }));
        assert_eq!(format.unit, Unit::DecimalFeet);
    }

    /// **This test used to pin the defect**, and the correction is worth
    /// more than the assertion.
    #[test]
    fn scale_ratio_path_needs_no_drawn_line() {
        // The group-panel path: ratio entry with no reference line.
        let fields = ScaleEntryFields {
            use_real_length: false,
            ratio_paper: 1.0,
            ratio_real: 100.0,
            basis: Unit::Inch,
            ..ScaleEntryFields::default()
        };
        assert_eq!(
            fields.unit,
            Unit::Meter,
            "the default display unit is metres against an INCH basis — the two disagreeing is \
             the whole subject of this test"
        );
        let preview = fields.preview(None).expect("a ratio preview with no line");
        assert_eq!(preview.unit, Unit::Meter, "the operator asked for metres");
        // ORACLE, NOT A CONVERSION: the expected metres-per-point, computed
        // here by hand from the definition of the inch. The subject of this
        // test is whether the display unit reaches the scale at all, so the
        // comparison has to come from somewhere the code under test cannot.
        let in_inches = 100.0 / 72.0;
        let expected = in_inches * 0.0254;
        assert!(
            (preview.scale - expected).abs() < 1e-12,
            "1:100 on an inch basis is {in_inches} in/pt, which is {expected} m/pt. Got \
             {}. If this is {in_inches}, the display unit has stopped reaching the scale and \
             the preview is labelling inches as metres.",
            preview.scale
        );
        assert_eq!(preview.ratio_label, "1:100");
        // for_group_panel() pre-selects the ratio path.
        assert!(!ScaleEntryFields::for_group_panel().use_real_length);
    }

    /// **The independent calibration**, hand-stated rather than round tripped.
    #[test]
    fn the_seeded_ratio_reads_one_to_one_hundred_for_a_group_stored_at_that_scale() {
        let format = NumberFormat {
            unit: Unit::Inch,
            ..Unit::Inch.default_format()
        };
        let fields = ScaleEntryFields::for_group(
            ScaleState::Calibrated {
                scale: 100.0 / 72.0,
            },
            format,
        );
        assert!(
            (fields.ratio_paper - 1.0).abs() < 1e-12,
            "paper side should be 1, was {}",
            fields.ratio_paper
        );
        assert!(
            (fields.ratio_real - 100.0).abs() < 1e-9,
            "real side should be 100, was {}",
            fields.ratio_real
        );
        assert_eq!(fields.basis, Unit::Inch, "basis must follow the group unit");
        assert_eq!(fields.unit, Unit::Inch, "display unit must follow it too");
        // And the label the operator reads, from the engine, not from here.
        let preview = fields.preview(None).expect("a 1:100 seed must preview");
        assert_eq!(preview.ratio_label, "1:100");
    }

    /// **The round trip is EXACT**, not merely close -- which is the whole
    /// reason [`ScaleEntryFields::for_group`] sets `basis` and `unit` together.
    #[test]
    fn a_calibrated_group_round_trips_through_the_seed() {
        for unit in Unit::all().iter().copied() {
            // A scale with no special structure, so an accidental identity
            // cannot pass for a correct inversion.
            let stored = 0.013_777_31_f64 * unit.baseline_per_point();
            let format = NumberFormat {
                unit,
                ..unit.default_format()
            };
            let fields =
                ScaleEntryFields::for_group(ScaleState::Calibrated { scale: stored }, format);
            let preview = fields
                .preview(None)
                .unwrap_or_else(|| panic!("{unit:?}: a seeded group must preview"));
            assert_eq!(preview.unit, unit, "{unit:?}: unit must survive the seed");
            assert!(
                (preview.scale - stored).abs() <= f64::EPSILON * stored.abs() * 4.0,
                "{unit:?}: round trip moved the scale from {stored} to {}",
                preview.scale
            );
        }
    }

    /// **A never-set group is seeded with nothing**, and that is the point.
    #[test]
    fn a_never_set_group_is_seeded_with_the_default_ratio_and_not_a_guess() {
        let format = NumberFormat {
            unit: Unit::Millimeter,
            ..Unit::Millimeter.default_format()
        };
        let fields = ScaleEntryFields::for_group(ScaleState::NeverSet, format);
        let default = ScaleEntryFields::default();
        assert!((fields.ratio_paper - default.ratio_paper).abs() < 1e-12);
        assert!((fields.ratio_real - default.ratio_real).abs() < 1e-12);
        // The unit and basis DO follow the group even here: the group's unit is
        // a real choice somebody made, and it is not a claim about scale.
        assert_eq!(fields.unit, Unit::Millimeter);
        assert_eq!(fields.basis, Unit::Millimeter);
    }

    /// **A 1:1 group seeds as `1 : 1`**, in its own unit rather than in inches.
    #[test]
    fn a_one_to_one_group_seeds_as_one_to_one_in_its_own_unit() {
        let format = NumberFormat {
            unit: Unit::DecimalFeet,
            ..Unit::DecimalFeet.default_format()
        };
        let fields = ScaleEntryFields::for_group(ScaleState::OneToOne, format);
        assert!((fields.ratio_paper - 1.0).abs() < 1e-12);
        assert!(
            (fields.ratio_real - 1.0).abs() < 1e-12,
            "real side should be 1, was {}",
            fields.ratio_real
        );
        let preview = fields.preview(None).expect("1:1 must preview");
        assert_eq!(preview.ratio_label, "1:1");
        assert!(
            (preview.scale - Unit::DecimalFeet.baseline_per_point()).abs() < 1e-12,
            "a 1:1 seed must back-calculate to the unit's own baseline"
        );
    }

    /// **A degenerate stored scale falls back rather than poisoning a control.**
    #[test]
    fn a_degenerate_stored_scale_seeds_the_default_instead_of_a_nan() {
        let format = NumberFormat {
            unit: Unit::Meter,
            ..Unit::Meter.default_format()
        };
        for bad in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
            let fields = ScaleEntryFields::for_group(ScaleState::Calibrated { scale: bad }, format);
            assert!(
                fields.ratio_real.is_finite() && fields.ratio_real > 0.0,
                "stored scale {bad} produced an unusable ratio {}",
                fields.ratio_real
            );
            assert!(
                fields.preview(None).is_some(),
                "stored scale {bad} must still preview"
            );
        }
    }

    /// O242: each side of the ratio carries its own unit. Hand oracle: one
    /// inch is 72 pt, so `1 in = 20 ft` is 20/72 ft per point.
    #[test]
    fn a_ratio_with_a_unit_on_each_side_reads_as_the_drawing_states_it() {
        let fields = ScaleEntryFields {
            use_real_length: false,
            ratio_paper: 1.0,
            ratio_real: 20.0,
            basis: Unit::Inch,
            real_unit: Unit::DecimalFeet,
            unit: Unit::DecimalFeet,
            ..ScaleEntryFields::default()
        };
        let preview = fields.preview(None).expect("a ratio preview");
        assert_eq!(preview.unit, Unit::DecimalFeet);
        let expected = 20.0 / 72.0;
        assert!(
            (preview.scale - expected).abs() < 1e-12,
            "1 in = 20 ft is {expected} ft/pt, got {}",
            preview.scale
        );
        assert_eq!(preview.ratio_label, "1:240");
    }

    #[test]
    fn scale_entry_routes_to_ratio_when_no_line_even_if_real_length_selected() {
        // Real-length selected but no drawn length → falls back to the ratio
        // entry (the only computable one), never a degenerate real-length call.
        let fields = ScaleEntryFields {
            use_real_length: true,
            ..ScaleEntryFields::default()
        };
        assert!(matches!(fields.entry(None), ScaleEntry::Ratio { .. }));
        assert!(matches!(
            fields.entry(Some(42.3)),
            ScaleEntry::RealLength { .. }
        ));
    }
}
