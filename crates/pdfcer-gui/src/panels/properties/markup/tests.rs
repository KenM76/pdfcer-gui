//! # `panels::properties::markup` tests — the restyle section's assertions
//!
//! ## ★★ Why they live in a file of their own
//!
//! **R2.** `markup.rs` crossed 1,500 lines on 2026-09-06, when the line-style
//! row and the deletion of this crate's copy of the engine's subtype list
//! landed in it on the same afternoon. The seam is `canvas::annotnodes`' —
//! module beside module, `#[cfg(test)] mod tests;` — and it is a subject seam:
//! the parent draws rows, this file asserts what decides them.
//!
//! ## ★★★ What these can and cannot prove
//!
//! They cannot prove an operator can restyle a mark; nothing here draws a
//! pixel or reaches `set_markup_style` (R1). What they prove is the set of
//! decisions the section takes before it draws:
//!
//! 1. **Reachability is asked of `spec_from_dict`** — the same call
//!    `set_markup_style` makes — so a mark this panel offers controls for is a
//!    mark the verb can act on.
//! 2. **Which properties a `/Subtype` takes is asked of the ENGINE**,
//!    `MarkupStyleSupport::for_subtype`, and not restated here. That is the
//!    workaround deleted on 2026-09-06.
//! 3. **A colour the swatch cannot show without converting says so**, which is
//!    the disclosure a CAD sheet's CMYK marks depend on.
//! 4. **The `/LE` removal is offered only when there is a `/LE` to remove**,
//!    which is the one fact `spec_from_dict` deliberately erases.
//!
//! ⚠ Every negative assertion is paired with a positive control, per the
//! engine's methodology note of 2026-09-06.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy. `canvas::annotnodes::tests` carries the
// same line for the same reason; this file learned it the same way, by the gate
// biting the moment the split landed.
#![cfg(test)]

use super::*;

// ★ `page_tree::Rect`, not `annot_author::Rect`. `annot_author` imports the
// type privately, so the path that reads naturally is not a path that
// resolves — the one place in this module where the engine's own module
// layout leaks.
use pdfcer_core::page_tree::Rect;

/// A `MarkupSpec::Square`, with whatever interior the caller wants.
fn square(interior: Option<Color>) -> MarkupSpec {
    MarkupSpec::Square {
        rect: Rect::from_corners(0.0, 0.0, 10.0, 10.0),
        border: Some(Color::Rgb(1.0, 0.0, 0.0)),
        interior,
        border_width: 2.0,
        border_effect: None,
    }
}

/// A [`Current`] for a spec and the `/Subtype` the file would carry it
/// under.
///
/// ★ The subtype is a **separate argument** rather than inferred from the
/// spec arm, and that is the shape of the whole change: the arm is where a
/// value comes from, the `/Subtype` is what the engine's capability
/// question is asked about, and a helper that derived the second from the
/// first would have put this module's deleted list straight back — in the
/// tests, where it could no longer be seen.
fn current(spec: &MarkupSpec, subtype: &[u8]) -> Current {
    Current::from_spec(
        Some(spec),
        MarkupStyleSupport::for_subtype(subtype),
        None,
        crate::canvas::markup::linestyle::DashReading::Solid,
        false,
    )
}

/// A `MarkupSpec::Line` with the given endings.
fn line(endings: (LineEnding, LineEnding)) -> MarkupSpec {
    MarkupSpec::Line {
        start: (0.0, 0.0),
        end: (10.0, 10.0),
        color: Color::Gray(0.0),
        width: 1.0,
        endings,
    }
}

/// Grey resolves to an equal-component swatch, and RGB round-trips —
/// neither of them flagged as narrowing.
///
/// Grey is not flagged because it is **lossless** in both directions —
/// `Gray(v)` and `Rgb(v, v, v)` are the same ink — where CMYK is not, which
/// is the distinction `swatch_of`'s own docs draw.
#[test]
fn a_swatch_shows_grey_and_rgb_without_calling_them_converted() {
    let red = swatch_of(Some(&Color::Rgb(1.0, 0.0, 0.0)));
    assert_eq!(red.rgb, Some([255, 0, 0]));
    assert!(!red.narrowed);

    let black = swatch_of(Some(&Color::Gray(0.0)));
    assert_eq!(black.rgb, Some([0, 0, 0]));
    assert!(!black.narrowed);

    assert_eq!(
        swatch_of(Some(&Color::Gray(1.0))).rgb,
        Some([255, 255, 255])
    );
    assert_eq!(swatch_of(None).rgb, None);
}

/// ★★★ **A CMYK `/C` shows, and says it is a conversion.**
///
/// The defect this pins: before 2026-09-06 a CMYK mark reached the panel as
/// `None` — a default black swatch and, worse, **no Clear button**, so the
/// one lossless operation available on it was the one withheld. Both halves
/// are asserted, because fixing only the first would leave a converted
/// colour presented as though it were the file's own value.
///
/// Falsified twice: `narrowed: false` in the `Cmyk` arm turned the
/// disclosure assertion red, and collapsing that arm to
/// `Color::Cmyk(..) => Swatch::default()` — which is the behaviour that
/// shipped until 2026-09-06 — turned the value assertions red.
#[test]
fn a_cmyk_colour_is_shown_as_a_conversion_and_is_flagged_as_one() {
    // Pure cyan: 1 - min(1, 1 + 0) = 0 red, 1 - 0 = 1 green and blue.
    let cyan = swatch_of(Some(&Color::Cmyk(1.0, 0.0, 0.0, 0.0)));
    assert_eq!(cyan.rgb, Some([0, 255, 255]));
    assert!(
        cyan.narrowed,
        "a converted colour must announce itself, or the swatch is a quiet lie"
    );
    // Registration black: every channel plus K, clamped, is black.
    assert_eq!(
        swatch_of(Some(&Color::Cmyk(1.0, 1.0, 1.0, 1.0))).rgb,
        Some([0, 0, 0])
    );
}

/// ★★★ **A mark the style verb cannot reach gets NO rows.**
///
/// `spec_from_dict` answers `UnsupportedSubtype` for `/Text`, `/FreeText`
/// and `/Stamp` — verified against the engine source — and `None` here is
/// exactly what that refusal becomes on the way in. What this asserts is
/// that the refusal reaches the panel as a *reachability* verdict rather
/// than as "this mark has no colour", because the two produce different
/// screens: nothing plus a sentence, versus three live controls that cannot
/// commit.
///
/// Falsified by setting `restylable: true` in the `None` arm of
/// `from_spec`, which turned this red immediately.
#[test]
fn a_subtype_the_style_verb_refuses_offers_no_controls() {
    // ★ `/FreeText` is the honest subtype for this case: it is one of the
    // three `spec_from_dict` refuses, and `for_subtype` answers `false` to
    // everything for it, so both halves of the verdict come from the engine.
    let refused = Current::from_spec(
        None,
        MarkupStyleSupport::for_subtype(b"FreeText"),
        Some(0.5),
        crate::canvas::markup::linestyle::DashReading::Solid,
        false,
    );
    assert!(!refused.restylable);
    assert_eq!(refused.colour.rgb, None);
    assert_eq!(refused.width, None);
    assert!(!refused.offers_fill());
    assert_eq!(refused.endings, None);
    assert_eq!(
        refused.alpha, None,
        "an opacity under a heading whose controls are all withheld is decoration"
    );
}

/// …and a mark it CAN reach says so, on the same function.
///
/// The companion assertion, and it is the one that stops the fix above from
/// being "return false always", which would pass the test above and remove
/// the whole section from the application.
#[test]
fn a_subtype_the_style_verb_reads_offers_its_controls() {
    let ok = Current::from_spec(
        Some(&square(None)),
        MarkupStyleSupport::for_subtype(b"Square"),
        Some(0.5),
        crate::canvas::markup::linestyle::DashReading::Solid,
        false,
    );
    assert!(ok.restylable);
    assert_eq!(ok.colour.rgb, Some([255, 0, 0]));
    assert_eq!(ok.width, Some(2.0));
    assert_eq!(ok.alpha, Some(0.5));
}

/// ★★ **The Fill row follows the `/Subtype`, and the ENGINE is what maps
/// one to the other.**
///
/// A square, a circle, a polygon and a cloud have an interior; a line, an
/// ink stroke, a polyline and a text markup do not. The cloud is the
/// interesting one — it is a `/Polygon` in the file and a
/// `MarkupSpec::Cloud` here — and it is the case this test exists for.
///
/// ⚠ **Corrected 2026-09-06.** This doc used to justify the answer with
/// *"`apply_markup_style` reads `style.interior` on exactly the first
/// four"* and to call the alternative *"a subtype-string list"* that would
/// have got the cloud wrong. Both halves have moved: the mapping is now
/// `MarkupStyleSupport::for_subtype` — which **is** keyed on the subtype
/// string, and is right to be, because it is the engine's string keyed by
/// the engine — and this shell no longer restates what
/// `apply_markup_style` reads.
///
/// Falsified by pointing `offers_fill` at the interior swatch instead of at
/// `support.takes_interior`, which turned the line assertion red, and by
/// passing the cloud `b"Cloud"` — a name no file carries — which turned the
/// cloud assertion red and is the mistake the old comment feared.
#[test]
fn only_a_shape_with_an_interior_gets_a_fill_row() {
    assert!(current(&square(None), b"Square").offers_fill());
    assert!(
        current(
            &MarkupSpec::Cloud {
                vertices: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)],
                border: None,
                interior: None,
                width: 1.0,
                intensity: 1.0,
            },
            // ★ The `/Subtype` a revision cloud actually carries. That it
            // is a `/Polygon` in the file and a `MarkupSpec::Cloud` here is
            // the case the old subtype-string list would have got wrong,
            // and it is now the engine that gets it right.
            b"Polygon",
        )
        .offers_fill(),
        "a revision cloud is a /Polygon in the file and has an /IC"
    );
    assert!(
        !current(&line((LineEnding::None, LineEnding::None)), b"Line").offers_fill(),
        "a line has no interior for /IC to mean anything in"
    );
}

/// The Fill swatch reads back the interior it was given, and offers Clear
/// only when there is something to clear.
///
/// `interior.rgb.is_some()` is precisely the condition `fill_row` puts the
/// Clear button behind, so this is that button's guard asserted where a test
/// can reach it.
#[test]
fn the_fill_swatch_reads_back_the_interior_and_knows_when_it_is_absent() {
    let filled = current(&square(Some(Color::Rgb(0.0, 0.0, 1.0))), b"Square");
    assert_eq!(filled.interior.rgb, Some([0, 0, 255]));

    let unfilled = current(&square(None), b"Square");
    assert!(unfilled.offers_fill(), "the row draws");
    assert_eq!(unfilled.interior.rgb, None, "…with no Clear beside it");
}

/// ★★ **The line-ending choosers appear for a `/Line` and for nothing
/// else** — the set `MarkupStyleSupport::takes_endings` names, and the set
/// `EditError::StylePropertyNotApplicable` refuses everything outside of.
///
/// ⚠ **Corrected 2026-09-06.** *"which is the set `apply_markup_style` acts
/// on"* was this shell restating a fact about the engine's source. The
/// engine now publishes it, so the test asks rather than remembers — and
/// the pair being asserted is the two questions kept apart: `offers_endings`
/// (the engine's) and `endings` (the value, from the one spec arm that has
/// one).
///
/// Falsified by returning `Some((LineEnding::None, LineEnding::None))` from
/// every arm, which turned the square's value assertion red, and by
/// dropping `support.takes_endings` from `offers_endings`, which turned its
/// visibility assertion red.
#[test]
fn only_a_line_gets_the_ending_choosers() {
    let arrow = current(&line((LineEnding::OpenArrow, LineEnding::None)), b"Line");
    assert!(arrow.offers_endings());
    assert_eq!(
        arrow.endings,
        Some((LineEnding::OpenArrow, LineEnding::None)),
        "and it reads back what the file says, so the unchanged end is never stale"
    );
    let square = current(&square(None), b"Square");
    assert!(!square.offers_endings());
    assert_eq!(square.endings, None);
}

/// ★★★ **The chooser's list covers every ending the engine can draw.**
///
/// `LineEnding` has no `ALL` of its own, so [`ALL_ENDINGS`] is written by
/// hand — and a hand-written list of an enum's variants is exactly the thing
/// that goes stale. This `match` has **no wildcard**: an ending the engine
/// gains fails to compile here, and the fix is to add it to both places at
/// once.
///
/// The count is asserted too, so a duplicated entry (three names, two
/// distinct variants) is caught as well as a missing one.
#[test]
fn the_ending_list_covers_every_variant_the_engine_has() {
    for ending in ALL_ENDINGS {
        // Exhaustive by construction — no `_` arm.
        match ending {
            LineEnding::None | LineEnding::OpenArrow | LineEnding::ClosedArrow => {}
        }
    }
    let mut seen: Vec<LineEnding> = Vec::new();
    for ending in ALL_ENDINGS {
        assert!(!seen.contains(&ending), "an ending is offered twice");
        seen.push(ending);
    }
    assert_eq!(seen.len(), 3, "Table 176's three that pdfcer authors");
}

/// ★ The width range is the same one the markup pen offers.
///
/// Two ranges for one quantity would let an operator author a 2 pt mark and
/// then be unable to set 2 pt on it — or, worse, set a width here the pen
/// could not have produced, so a document would carry marks the shell
/// cannot make.
#[test]
fn the_width_range_matches_the_pen_that_authors() {
    assert!((MIN_WIDTH_PT - crate::canvas::markup::pen::MIN_WIDTH_PTS).abs() < f64::EPSILON);
    assert!((MAX_WIDTH_PT - crate::canvas::markup::pen::MAX_WIDTH_PTS).abs() < f64::EPSILON);
}

/// ★★★ **What hides a row is the ENGINE's answer, not the shape of the
/// `MarkupSpec` arm this module read.**
///
/// The assertion that the workaround is gone. Until 2026-09-06 the Fill row
/// was decided by a four-arm `match` here, the width row by the
/// `TextMarkup` arm handing over no width, and the choosers by `endings`
/// being `None` off the `Line` arm — three restatements of a list
/// `pdfcer-core` owns, filed as: *"the first subtype that gains or loses a
/// border is the day our copy is wrong and nothing tells us."*
///
/// ★ Every `Current` here is built with **every value present**, so a value
/// cannot be what differs. A row withheld below is withheld because
/// `MarkupStyleSupport::for_subtype` said so.
///
/// ★★ Both directions, per the engine's methodology note: the `Highlight`
/// row alone would pass with every predicate hard-coded to `false`. The
/// `Square` and `Line` rows are the positive control that makes it mean
/// something.
///
/// Falsified by restoring the old rules — `offers_width` reading only
/// `self.width.is_some()` turned the `Highlight` row red, and `offers_fill`
/// reading whether the interior swatch was set turned the `Line` row red.
#[test]
fn the_engines_answer_is_what_hides_a_row_not_the_spec_arm() {
    /// Every value a row could want, on a mark no real subtype could be.
    fn over_supplied(subtype: &[u8]) -> Current {
        Current {
            restylable: true,
            support: MarkupStyleSupport::for_subtype(subtype),
            colour: Swatch {
                rgb: Some([0, 0, 0]),
                narrowed: false,
            },
            interior: Swatch {
                rgb: Some([255, 255, 255]),
                narrowed: false,
            },
            width: Some(2.0),
            // ★ `Solid` is a value, not an absence — `linestyle::read` is
            // total — so there is no "no dash" state to over-supply, which
            // is why `offers_dash` is `takes_border` and nothing else.
            dash: crate::canvas::markup::linestyle::DashReading::Solid,
            alpha: Some(1.0),
            endings: Some((LineEnding::None, LineEnding::OpenArrow)),
            endings_key_present: true,
        }
    }

    // (subtype, fill, width, endings)
    let expected: &[(&[u8], bool, bool, bool)] = &[
        (b"Square", true, true, false),
        (b"Circle", true, true, false),
        (b"Polygon", true, true, false),
        (b"Line", false, true, true),
        (b"PolyLine", false, true, false),
        (b"Ink", false, true, false),
        (b"Highlight", false, false, false),
        (b"Underline", false, false, false),
        (b"StrikeOut", false, false, false),
        (b"Squiggly", false, false, false),
        (b"FreeText", false, false, false),
    ];
    for &(subtype, fill, width, endings) in expected {
        let name = String::from_utf8_lossy(subtype).into_owned();
        let c = over_supplied(subtype);
        assert_eq!(c.offers_fill(), fill, "/{name}: Fill row");
        assert_eq!(c.offers_width(), width, "/{name}: width row");
        // The line-style chooser is the fourth reader of the same answer:
        // `/BS` `/D` is a border property, and `set_markup_style` refuses a
        // `dash` on a text markup by the same predicate it refuses a
        // `width` with (`edit.rs:26469`).
        assert_eq!(c.offers_dash(), width, "/{name}: line-style row");
        assert_eq!(c.offers_endings(), endings, "/{name}: ending choosers");
        // …and each predicate is the engine's flag, not a table kept here.
        // `NO_SURFACE.md` §1a: assert the relation, not the magnitude, or
        // two copies of one constant will simply agree with each other.
        let support = MarkupStyleSupport::for_subtype(subtype);
        assert_eq!(c.offers_fill(), support.takes_interior);
        assert_eq!(c.offers_width(), support.takes_border);
        assert_eq!(c.offers_dash(), support.takes_border);
        assert_eq!(c.offers_endings(), support.takes_endings);
    }
}

/// ★★★ **The removal is offered only when `/LE` is actually in the file.**
///
/// The fifth state's guard, and the field it needs exists because
/// `spec_from_dict` cannot answer the question: Table 176's default makes
/// an absent `/LE` and a written `[/None /None]` read back identically, so
/// [`Current::endings`] is `Some((None, None))` either way. Only the
/// dictionary knows, which is why [`Current::read`] looks.
///
/// ★ Both directions. *"Absent when the key is absent"* on its own passes
/// with the button deleted, which is precisely the vacuous shape the
/// engine's reply warned about.
///
/// Falsified by making `offers_endings_clear` return `self.offers_endings()`,
/// which turned the first assertion red, and by making it return `false`,
/// which turned the second red.
#[test]
fn the_removal_is_offered_only_when_the_file_carries_a_line_ending_entry() {
    let plain = line((LineEnding::None, LineEnding::None));
    let support = MarkupStyleSupport::for_subtype(b"Line");

    let no_key = Current::from_spec(
        Some(&plain),
        support,
        None,
        crate::canvas::markup::linestyle::DashReading::Solid,
        false,
    );
    assert!(
        no_key.offers_endings(),
        "the choosers still draw — a /Line can always be GIVEN arrowheads"
    );
    assert!(
        !no_key.offers_endings_clear(),
        "a /Line with no /LE has nothing to remove, and a Clear there could \
         only ever earn an undo entry the operator did not ask for"
    );

    let with_key = Current::from_spec(
        Some(&plain),
        support,
        None,
        crate::canvas::markup::linestyle::DashReading::Solid,
        true,
    );
    assert!(
        with_key.offers_endings_clear(),
        "a /Line that carries /LE must be able to give it back — same \
         picture, and the file goes out as it came in"
    );

    // …and never on a subtype with no choosers to sit under, whatever the
    // dictionary happens to hold.
    let highlight = Current::from_spec(
        Some(&MarkupSpec::TextMarkup {
            kind: pdfcer_core::annot_author::TextMarkupKind::Highlight,
            quads: vec![pdfcer_core::annot_author::Quad::from_rect(
                Rect::from_corners(0.0, 0.0, 10.0, 10.0),
            )],
            color: Color::Rgb(1.0, 1.0, 0.0),
        }),
        MarkupStyleSupport::for_subtype(b"Highlight"),
        None,
        crate::canvas::markup::linestyle::DashReading::Solid,
        true,
    );
    assert!(!highlight.offers_endings_clear());
}

/// A mark whose dictionary could not be read offers nothing, and gets that
/// answer from the engine rather than from three literal `false`s.
///
/// ★ An all-`false` literal in [`Current::default`] would have been the last
/// surviving copy of the engine's list in this module, three entries long.
#[test]
fn a_mark_that_could_not_be_read_offers_nothing() {
    let nothing = Current::default();
    assert_eq!(nothing.support, MarkupStyleSupport::for_subtype(b""));
    assert!(!nothing.restylable);
    assert!(!nothing.offers_fill());
    assert!(!nothing.offers_width());
    assert!(!nothing.offers_endings());
    assert!(!nothing.offers_endings_clear());
}
