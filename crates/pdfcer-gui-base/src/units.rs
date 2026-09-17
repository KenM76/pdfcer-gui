//! **The one length-conversion table for this program.**
//!
//! Every conversion between PDF points and a length an operator reads or types
//! goes through this module, and nowhere else declares a points-per-millimetre
//! constant, a `× 25.4 / 72.0` closure, or a `/ 72.0` DPI spelling.
//!
//! # Why this module exists at all
//!
//!
//! * six private constants under two different names — `PTS_PER_MM` in
//!   `panels/pages`, `panels/docprops` and `dialogs/insert_image`, `PT_PER_MM`
//!   in `dialogs/new_document`, `dialogs/page_size` and `text/page_size` — two
//!   of them declared `f32` and the rest `f64`;
//! * seven inline closures re-declaring `|pt: f64| pt * 25.4 / 72.0`, five of
//!   them in one file;
//! * and `pdfcer_core`'s own [`Unit::baseline_per_point`], which is the value
//!   all thirteen were approximating.
//!
//! That is not a tidiness complaint. Three concrete things went wrong:
//!
//! **1. The divide form and the multiply form are not the same arithmetic —
//! and there is a third form neither of them is.**
//! `x / (72.0 / 25.4)` and `x * 25.4 / 72.0` differ in IEEE-754, because
//! `72.0 / 25.4` is itself inexact, so rounding it once and dividing is not the
//! same as multiplying by an exact numerator and dividing by an exact
//! denominator. Today that is masked by every consumer rounding hard, which is
//! precisely the kind of masking that stops working the moment somebody shows a
//! decimal.
//!
//!
//! ```text
//!   engine  vs closure form    3_003 of 10_000 inputs disagree
//!   engine  vs divide  form      825 of 10_000 inputs disagree
//!   closure vs divide  form    3_441 of 10_000 inputs disagree
//! ```
//!
//! ⇒ **"Disagree in the last ulp" was the wrong shape of claim.** Any two of
//! the three agree on most inputs and part on some. A4's 841.89 pt gives the
//! identical answer down the bit in all three spellings, so a program holding
//! all three cannot be shown to be inconsistent by checking a page size. The
//! disagreement waits for a sheet nobody thought to test — which is exactly
//! how the rounding half of this defect reached the operator.
//!
//! **2. The rounding rule was inconsistent, and that one is visible.** See the
//! section below; it is the part that reached the operator.
//!
//! **3. Two paths converted in `f32`.** On a 14,400 pt A0-class sheet an `f32`
//! millimetre loses roughly three decimal digits against the `f64` answer. It
//! never showed, because both `f32` sites round to whole millimetres — but a
//! module whose whole purpose is that two surfaces agree cannot be built on a
//! width that depends on which surface you asked.
//!
//! # ★★★ The rounding rule, and why it is a decision rather than a default
//!
//! A whole-number length shown to the operator rounds **half away from zero**:
//! `2.5 → 3`, `-2.5 → -3`. That is [`whole`], and it is the only function in
//! this program permitted to turn a length into a whole number for display.
//!
//! The alternative was never chosen by anyone; it arrived by accident.
//! Rust's `{:.0}` format specifier rounds **half to even** — `2.5 → 2`,
//! `3.5 → 4` — which is the IEEE-754 default and an excellent rule for summing
//! long columns of numbers, because it does not accumulate an upward bias. It
//! is the wrong rule for a single sheet size, for two reasons: a CAD operator
//! reading a drawing expects the schoolroom rule, and, decisively, **it is not
//! what the other half of this program was already doing**.
//!
//! The defect it produced, measured rather than reasoned:
//!
//! ```text
//! a sheet authored at exactly 210.5 mm
//!   converts to 596.6929133858 pt (f64) / 596.6929321289 pt (f32)
//!   both convert back to exactly 210.5000000000 mm
//!
//!   page-thumbnail tooltip   {:.0}           ->  "210"
//!   print dialogue           .round() as i64 ->  "211"
//! ```
//!
//! One document, one sheet, two whole numbers, on two surfaces an operator can
//! have open at the same time.
//!
//! ⚠ **The precision difference is not what causes that**, and the first three
//! written accounts of this defect said it was. Both paths land on exactly
//! `210.5000000000`; the disagreement is entirely the rounding rule. The `f32`
//! problem is real and separate and is fixed here too, by this module being
//! `f64` throughout — but it contributed nothing to the number the operator
//! would have seen.
//!
//! # What this module does NOT cover, deliberately
//!
//! **Type size.** A font size printed as `12 pt` is the *typographic* point.
//! It is arithmetically the same 1/72 inch, and it must never be offered in
//! millimetres, metres or — the failure mode worth naming — kilometres. No
//! drawing or publishing program does this and Acrobat does not. Those
//! surfaces carry an in-source note pointing here; they are excluded, not
//! overlooked. See `UNIT_SURFACES.md` §4 for the list.
//!
//! **Dimension-group text height, arrow size and gap.** `text/dimension_groups`
//! argues these are *paper* sizes — "10 pt tall whatever the drawing is scaled
//! at" — and that reasoning is sound. They are provisionally excluded on the
//! same grounds as type size, but the exclusion sits inside the dimensioning
//! feature and is flagged in `UNIT_SURFACES.md` §4 as the one worth putting in
//! front of the operator rather than deciding silently.
//!
//! **Scale-aware conversion.** Everything here is the 1:1 *baseline*: it turns
//! a PDF point into a physical length on the sheet. Converting a sheet length
//! into a real-world length through a **ce dimension** group's scale is
//! `ScaleState::effective_scale` in the engine, and is a different question
//! with a different answer per group. (R8b rule 15: **ce dimensions** are the
//! ones this program authors; **pdf dimensions** are CAD-exported page content
//! it must not silently alter. Neither is what this module converts — this
//! converts *lengths*.)
//!
//! # Contract
//!
//! * Every function takes and returns `f64`. Callers holding `f32` (egui rects,
//!   mostly) convert in, not out: `mm_from_points(f64::from(rect.width()))`.
//! * Conversion is a single multiply by [`Unit::baseline_per_point`], so two
//!   call sites given the same input produce bit-identical results. That is the
//!   property the six private constants could not offer.
//! * No function here formats. They return numbers; the caller writes `{}` for
//!   a whole number and an explicit `{:.N}` for a decimal. **A length must not
//!   reach a format string as a bare `{:.0}`** — that is the half-to-even path
//!   this module exists to remove, and `tools/gates/check-unit-conversion.sh`
//!   fails the build on it.

use pdfcer_core::dimension::Unit;

/// Convert a length in PDF points into `unit`, at 1:1.
///
/// One multiply by the engine's exact factor. `points` is a PDF user-space
/// unit, which ISO 32000-2 §8.3.2.3 defines as 1/72 inch in the default user
/// space — this function assumes the default CTM, which is what every surface
/// in this program that shows a sheet size has already established by reading
/// the page's `/MediaBox`.
#[must_use]
#[inline]
pub fn from_points(points: f64, unit: Unit) -> f64 {
    points * unit.baseline_per_point()
}

/// Convert a length expressed in `unit` back into PDF points.
///
/// The exact inverse of [`from_points`] — one divide by the same factor, so a
/// round trip loses at most one rounding step rather than two independently
/// spelled ones.
#[must_use]
#[inline]
pub fn to_points(value: f64, unit: Unit) -> f64 {
    value / unit.baseline_per_point()
}

/// Points to millimetres.
///
/// Millimetres are given their own pair because they are what most surfaces in
/// this program actually want, and a named function reads better at the call
/// site than `from_points(w, Unit::Millimeter)` repeated eleven times. It is
/// the same arithmetic, not a second table.
#[must_use]
#[inline]
pub fn mm_from_points(points: f64) -> f64 {
    from_points(points, Unit::Millimeter)
}

/// Millimetres to points. Inverse of [`mm_from_points`].
#[must_use]
#[inline]
pub fn points_from_mm(mm: f64) -> f64 {
    to_points(mm, Unit::Millimeter)
}

/// Points to inches.
#[must_use]
#[inline]
pub fn inches_from_points(points: f64) -> f64 {
    from_points(points, Unit::Inch)
}

/// Inches to points. Inverse of [`inches_from_points`].
#[must_use]
#[inline]
pub fn points_from_inches(inches: f64) -> f64 {
    to_points(inches, Unit::Inch)
}

/// **The one rounding rule for a whole-number length shown to the operator.**
///
/// Half away from zero. `2.5 → 3`, `-2.5 → -3`, `2.4 → 2`.
///
/// This is `f64::round`'s rule, and the reason it gets a named wrapper rather
/// than a bare `.round() as i64` at each site is that the wrapper is
/// greppable: a reviewer can ask "which surfaces turn a length into a whole
/// number?" and get a complete answer, which was impossible while half the
/// sites spelled it `{:.0}` inside a format string.
///
/// ⚠ **Do not reach for `{:.0}`.** It rounds half to *even*, which disagrees
/// with this on every tie, and a tie is exactly what a half-millimetre sheet
/// produces. The module header has the measured case.
///
/// # Saturation
///
/// `as i64` on an out-of-range or non-finite `f64` saturates rather than
/// wrapping (Rust 1.45 onwards: `NaN → 0`, `+inf → i64::MAX`). A page big
/// enough to overflow an `i64` of millimetres is about 9×10^15 metres, so this
/// is a note for the reader rather than a guard anybody needs — but it is
/// written down because the previous spelling was `as i64` too, and a reader
/// checking whether the refactor changed overflow behaviour deserves the
/// answer without leaving the file.
#[must_use]
#[inline]
pub fn whole(value: f64) -> i64 {
    value.round() as i64
}

/// Points to whole millimetres, using [`whole`]'s rounding rule.
///
#[must_use]
#[inline]
pub fn whole_mm_from_points(points: f64) -> i64 {
    whole(mm_from_points(points))
}

/// The render scale that produces `dpi` dots per inch from a PDF page.
///
/// A PDF point is 1/72 inch, so rendering at scale `s` yields `72 × s` dots per
/// inch and the inverse is `dpi / 72`. Three sites spelled this by hand
/// (`ocr`, the image-export action, the print preview); they agree with each
/// other and always did, so this is consolidation rather than a fix — but it is
/// the same physical constant as the rest of this table and leaving it outside
/// would have re-created the problem in a second place.
#[must_use]
#[inline]
pub fn scale_from_dpi(dpi: f64) -> f64 {
    dpi / 72.0
}

/// Dots per inch as **pixels per metre**, for image formats that record
/// physical resolution in SI (PNG `pHYs`, BMP, and the DIB header the Windows
/// clipboard carries).
///
/// `dpi / 0.0254` — one inch is 0.0254 m exactly, by the international inch
/// definition the engine's unit table also uses.
#[must_use]
#[inline]
pub fn pixels_per_metre(dpi: f64) -> f64 {
    dpi / 0.0254
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ There are **three** spellings of this conversion, not two, and
    /// which pair disagrees depends on the input.
    ///
    /// `UNIT_SURFACES.md` §3 enumerated two: the multiply form
    /// `pt * 25.4 / 72.0` that seven closures used, and the divide form
    /// `pt / (72.0 / 25.4)` that six private `*_PER_MM` constants used. This
    /// test was originally written to assert that the module matched the
    /// multiply form, "because that is what the engine does".
    ///
    /// **It is not what the engine does, and the test failed on its first run.**
    /// The engine's [`Unit::baseline_per_point`] returns `25.4 / 72.0`: one
    /// constant, divided once, folded at compile time. Multiplying by it is a
    /// third spelling:
    ///
    /// ```text
    /// points = 1234.567891
    ///   pt * 25.4 / 72.0        435.52811710277774   the seven closures
    ///   pt * (25.4 / 72.0)      435.5281171027777    the engine, and this module
    ///   pt / (72.0 / 25.4)      435.5281171027777    the six constants
    /// ```
    ///
    ///
    /// ```text
    ///   engine  vs closure form    3_003 of 10_000 inputs disagree
    ///   engine  vs divide  form      825 of 10_000 inputs disagree
    ///   closure vs divide  form    3_441 of 10_000 inputs disagree
    /// ```
    ///
    /// ⇒ That is a sharper statement of the module's premise than §3's
    /// original one. It is not that one spelling was wrong; it is that **any two
    /// of the three agree on most inputs and part on some**, so a program
    /// holding all three cannot be shown to be inconsistent by testing a page
    /// size — A4's 841.89 pt gives the identical answer down the bit in all
    /// three. The disagreement waits for a sheet nobody thought to check.
    ///
    /// ⇒ The module's behaviour is right and stays. Agreement with
    /// `pdfcer-core` is the whole reason to route through [`Unit`] rather than
    /// declare a fourteenth constant here: any length that crosses the crate
    /// boundary — a **ce dimension**'s label, a scale, a page size handed back
    /// to the engine — has to be the number the engine computed, not a number
    /// that rounds to the same place most of the time. So the assertion READS
    /// the engine's value instead of transcribing it; a transcription would pass
    /// on the day it was written and go silently wrong the day the engine
    /// changed.
    #[test]
    fn the_conversion_is_the_engines_own_constant_not_a_transcription_of_it() {
        // 1. Bit-identical to the engine, at an input with a long binary
        //    expansion. This is the assertion that matters; the sweep below is
        //    about the premise, not about this module's behaviour.
        let points = 1_234.567_891_f64;
        assert_eq!(
            mm_from_points(points),
            points * Unit::Millimeter.baseline_per_point(),
            "this module must be bit-identical to the engine, which is the \
             tie-breaker whenever a length crosses the crate boundary"
        );

        // 2. All three spellings genuinely diverge. Swept rather than sampled,
        //    because the first version of this test sampled and picked an input
        //    where two of the three happened to agree.
        let (mut eng_vs_closure, mut eng_vs_divide, mut closure_vs_divide) = (0_u32, 0_u32, 0_u32);
        for i in 0..10_000_u32 {
            let points = f64::from(i) + 0.567_8;
            let engine_form = points * Unit::Millimeter.baseline_per_point();
            let closure_form = points * 25.4 / 72.0;
            let constant_form = points / (72.0 / 25.4);

            if engine_form != closure_form {
                eng_vs_closure += 1;
            }
            if engine_form != constant_form {
                eng_vs_divide += 1;
            }
            if closure_form != constant_form {
                closure_vs_divide += 1;
            }
        }

        // Deliberately `> 0` rather than the measured counts. The counts are in
        // the doc comment above, dated, as a record of what this arithmetic did
        // on the day; pinning them here would turn a legitimate engine change
        // into a failure whose message says nothing about what actually moved.
        assert!(
            eng_vs_closure > 0,
            "engine form vs the seven closures never disagreed in 10 000 inputs \
             — §3's first finding is stale and the header needs rewriting"
        );
        assert!(
            eng_vs_divide > 0,
            "engine form vs the six private constants never disagreed in 10 000 \
             inputs — §3's first finding is stale"
        );
        assert!(
            closure_vs_divide > 0,
            "the two spellings §3 actually enumerated never disagreed — that \
             finding was the reason this module was written"
        );
    }

    /// ★★★ The regression test for the defect that reached the operator.
    ///
    /// A sheet authored at exactly 210.5 mm produced `210` on one surface and
    /// `211` on another. Both surfaces call [`whole_mm_from_points`] now.
    #[test]
    fn a_half_millimetre_sheet_rounds_up_everywhere() {
        let points = points_from_mm(210.5);

        // The round trip is exact — this is the part the first three written
        // accounts of the defect got wrong, blaming precision for a
        // disagreement that was entirely about the rounding rule.
        assert_eq!(
            mm_from_points(points),
            210.5,
            "210.5 mm must round-trip exactly; if it does not, the rounding-rule \
             argument in this module's header is no longer the whole story"
        );

        assert_eq!(whole_mm_from_points(points), 211, "half away from zero");

        // And the rule this module bans, spelled out, so the test states the
        // difference rather than merely asserting the survivor.
        assert_eq!(
            format!("{:.0}", mm_from_points(points)),
            "210",
            "`{{:.0}}` rounds half to even and is why the two surfaces disagreed"
        );
    }

    /// Half away from zero is a claim about **both** signs, and a test that
    /// only tries positives is not testing the rule.
    #[test]
    fn whole_rounds_half_away_from_zero_in_both_directions() {
        assert_eq!(whole(2.5), 3);
        assert_eq!(whole(3.5), 4, "not 4-from-even — 3.5 rounds up on any rule");
        assert_eq!(whole(-2.5), -3, "away from zero, not toward it");
        assert_eq!(whole(2.4), 2);
        assert_eq!(whole(-2.4), -2);
        assert_eq!(
            whole(0.5),
            1,
            "the tie nearest zero, where half-to-even says 0"
        );
    }

    /// The unit-generic pair must agree with the millimetre pair, or the
    /// convenience wrappers have become a second table.
    #[test]
    fn the_millimetre_helpers_are_the_generic_pair() {
        let points = 596.692_913_385_8_f64;
        assert_eq!(
            mm_from_points(points),
            from_points(points, Unit::Millimeter)
        );
        assert_eq!(points_from_mm(210.5), to_points(210.5, Unit::Millimeter));
        assert_eq!(
            inches_from_points(72.0),
            1.0,
            "72 points is one inch by definition"
        );
        assert_eq!(points_from_inches(1.0), 72.0);
    }

    /// Every unit the engine offers must convert without this module needing an
    /// edit. This reads `Unit::all()` rather than a hand-written list, because
    /// a private copy of an enumeration inside a test named "every" is the
    /// defect this project has now found seven times.
    #[test]
    fn every_engine_unit_round_trips() {
        let units = Unit::all();
        assert!(
            units.len() >= 8,
            "`Unit::all()` returned {} units; a short list would make the loop \
             below a green test that asserted nothing",
            units.len()
        );

        for unit in units.iter().copied() {
            let points = 1234.5_f64;
            let back = to_points(from_points(points, unit), unit);
            assert!(
                (back - points).abs() < 1e-9,
                "{unit:?} does not round-trip: {points} -> {back}"
            );
        }
    }

    /// The DPI spellings are consolidation, not correction — so the test's job
    /// is to prove they still produce what the three hand-written sites did.
    #[test]
    fn the_dpi_spellings_match_what_they_replaced() {
        assert_eq!(scale_from_dpi(144.0), 2.0, "144 dpi is twice nominal");
        assert_eq!(scale_from_dpi(72.0), 1.0);
        assert_eq!(pixels_per_metre(300.0), 300.0 / 0.0254);
    }
}
