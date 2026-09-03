//! # `canvas::measure::scale` — scale entry, and the dimension-group actions
//!
//! **Salvaged** from the old shell's `measure_tool.rs`
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\measure_tool.rs`, Pass 12.M2b), split at
//! that file's own section banners — see [`super::pick`] for the pick state
//! machines and [`super::state`] for the tool-entry container. The reasoning
//! below is the original's, carried across intact.
//!
//! The **pure, GUI-free** half of the scale-dimension tool: the reference-line
//! pick's dialog state, the two co-equal scale-entry paths and their back-calc
//! plumbing, and the group-panel action set. Like its siblings it is expressed
//! over `pdfcer-core` types and **never over egui**, so every transition is
//! unit-tested here without a live frame — the discipline that let the old
//! file be salvaged rather than rewritten.
//!
//! ## What this module owns vs. what the shipped engine owns (REUSE, never reimplement)
//!
//! This module contains **zero** scale arithmetic, unit conversion, length
//! parsing or storage. It owns *which* engine call to make and *what the
//! operator typed*; the numbers come from the already-shipped
//! `pdfcer-core::dimension`:
//!
//! - [`preview_group_scale`] (12.M2) — the scale back-calc for **both** entry
//!   paths. [`ScaleEntryFields::preview`] chooses the [`ScaleEntry`] variant
//!   and hands it over; it never divides anything itself. This is what makes a
//!   canvas-calibrated group and a CLI-calibrated group the same number.
//! - [`parse_length`] — `55 5/8"`, `4'-7 1/2"`. The scale field is a TEXT field
//!   precisely so the operator can type the dimension the way the drawing
//!   writes it, and the grammar for that lives in core, once.
//! - [`ScaleState`] / [`NumberFormat`] / [`Unit`] / [`FractionMode`] — the
//!   stored tri-state and display model handed to
//!   `EditSession::set_group_scale`. Constructed here, defined there.
//! - [`GroupId`] / [`DimStandard`] — the identities the group verbs name. Each
//!   is a **mapping onto exactly one shipped `EditSession` command**, which is
//!   what keeps "I made a group" and "I hid a layer" one `Ctrl+Z` each
//!   (ui-spec §5.4). They travel as
//!   `crate::app::actions::dimensions::DimensionAction` — this module builds
//!   the *values* they carry and never raises one itself.
//!
//! The pick half of that list — `constrained_second_point`, `measured_length`,
//! `fit_circle_taubin`, `author_from_two_lines` and the `DimensionKind`
//! byte-equivalence argument — is stated in full on [`super::pick`], which is
//! where the only callers of those live.
//!
//! ## The three tools' state
//!
//! Split across this file and its two siblings, but it is one model and reads
//! as one:
//!
//! - [`super::pick::LinearPick`] — the A→B two-click state machine (ui-spec
//!   §2.1), shared **verbatim** by [`ScalePick`]'s reference line (§4.1). Not a
//!   copy and not a parallel implementation: [`ScalePick::line`] *is* a
//!   `LinearPick`, constructed by [`super::pick::LinearPick::reference_line`],
//!   which is the same machine with its third placing click switched off.
//! - [`super::pick::CircularPick`] — the tool's OWN object pick-set (ui-spec
//!   §3.1), live-refit on every toggle (§3.2).
//! - [`ScalePick`] + [`ScaleEntryFields`] — draw a reference line, then the
//!   two co-equal scale-entry paths (real-length recommended, ratio) that
//!   back-calc through [`preview_group_scale`] (§4).
//! - [`super::state::MeasureState`] — the container built on tool entry that
//!   holds all of the above.
//!
//! Everything is `pdfcer-gui`-internal; `cargo tree -p pdfcer-core` is
//! unaffected (this module is not in core), and it adds no dependency.
//!
//! ## Adaptations made on the way across
//!
//! **None that change behaviour.** No arithmetic, no transition and no engine
//! call was touched; the only edits are module paths for the types that now
//! live in [`super::pick`] and [`super::state`]. The one thing worth stating
//! is what was *checked*: every `pdfcer-core` item named above still exists at
//! the engine HEAD this workspace builds against, with the same signature, so
//! nothing here is an adaptation to a moved API.

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
///
/// - **Real length (recommended, default):** the operator typed the drawn
///   reference line's real length + unit; back-calc `scale = real /
///   drawn_pdf_length` — needs a drawn line, so it is offered only where one
///   exists ([`ScalePick`]).
/// - **Direct ratio:** `paper : real` on a disclosed paper-unit basis;
///   needs no drawn line, so it is the path the group panel uses to set a
///   scale by typing alone (ui-spec §7.2 accessibility win).
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
    /// The real side of the direct ratio (`100` in `1:100`).
    pub ratio_real: f64,
    /// The paper-unit basis for the ratio path (default [`Unit::Inch`]; PDF
    /// paper units are 1/72", disclosed — ui-spec §4.2).
    pub basis: Unit,
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
            fraction: None,
        }
    }
}

impl ScaleEntryFields {
    /// Re-read [`Self::real_length_text`], updating the parsed value and
    /// (when the text named one) the unit.
    ///
    /// Returns the parse error for display, or `None` when it parsed. Called
    /// on every keystroke, so the operator sees what pdfcer understood while
    /// they are still looking at the field rather than after committing.
    ///
    /// # Why a failed parse leaves the previous value alone
    ///
    /// Mid-typing, `55 5/` is not a length. Zeroing the value on every
    /// intermediate keystroke would make the live scale preview flicker
    /// through garbage, and — worse — would leave a *stale* preview looking
    /// authoritative if the operator stopped typing at that moment. Instead
    /// the last good value is held and the error is shown, so the preview and
    /// the message never disagree about whether the input is usable.
    ///
    /// # Why the unit dropdown moves only when the text names a unit
    ///
    /// Typing `55 5/8"` says inches; the dropdown should follow, or the
    /// operator has to say the same thing twice. Typing a bare `55.625` says
    /// nothing about units, and moving the dropdown then would be the tool
    /// second-guessing a choice the operator already made.
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
            _ => ScaleEntry::Ratio {
                paper: self.ratio_paper,
                real: self.ratio_real,
                basis: self.basis,
            },
        }
    }

    /// The live scale preview (ui-spec §4.2 "→ scale = 25.0 ft / 42.3 pt"),
    /// via the shipped [`preview_group_scale`] — pure, no mutation. `None`
    /// for a degenerate entry (Accept then shows nothing to commit).
    #[must_use]
    pub fn preview(&self, drawn_pdf_length: Option<f64>) -> Option<ScalePreview> {
        preview_group_scale(self.entry(drawn_pdf_length))
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

// ★ **`GroupAction` was DELETED on 2026-08-18, and this note is what stands in
// its place.**
//
// It was a salvaged, fully-tested, **zero-caller** enum: four variants naming
// four `EditSession` group verbs, carried across whole in the Phase 7 salvage
// and never wired to anything. Its own doc comment recorded, correctly, that
// rename and delete were absent from the engine and deliberately not
// reimplemented here.
//
// What replaced it is `crate::app::actions::dimensions::DimensionAction`,
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
    ///
    /// Without this the display type was pinned to `Unit::default_format()`,
    /// so a drawing dimensioned in inches always read `55.63"` and could never
    /// read `55 5/8"` — the notation the drawing uses, and the notation the
    /// scale field already ACCEPTS as input.
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
        let preview = fields.preview(None).expect("a ratio preview with no line");
        assert!((preview.scale - 100.0 / 72.0).abs() < 1e-12);
        assert_eq!(preview.ratio_label, "1:100");
        // for_group_panel() pre-selects the ratio path.
        assert!(!ScaleEntryFields::for_group_panel().use_real_length);
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
