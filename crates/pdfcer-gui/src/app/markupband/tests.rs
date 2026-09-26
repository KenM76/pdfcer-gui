//! # `app::markupband` tests — the Format ▸ Markup band's own assertions
//!
//! ## Why they live in a file of their own
//!
//! Module beside module, `#[cfg(test)] mod tests;` — the seam
//! `canvas::annotnodes` and `app::conditions` also take. It is a **subject**
//! seam rather than an arithmetic one: the parent draws controls, and this
//! file asserts what they decide.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/markupband/tests.md`.

// The *inner* attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy. `canvas::annotnodes::tests` carries the
// same line for the same reason.
#![cfg(test)]

use super::*;

/// **Every custom kind the manifest declares for this group is drawn here,
/// and every kind drawn here backs a registered command.**
#[test]
fn every_markup_kind_in_the_register_is_drawn_by_this_module() {
    let mut registry = egui_shell::commands::CommandRegistry::new();
    crate::shell::commands::register(&mut registry);
    for (id, kind, _) in crate::shell::manifest::CUSTOM_BACKED {
        let Some(mapped) = command_for(kind) else {
            // Not this module's kind — `file.recent` and the three Font
            // controls are the other entries.
            continue;
        };
        assert_eq!(
            mapped, *id,
            "`{kind}` is registered as backing `{id}` and this module draws it for `{mapped}`"
        );
        assert!(
            registry.get(id).is_some(),
            "`{id}` is drawn by this module and is not in the registry, so the control would \
             silently vanish"
        );
    }
}

/// The six kinds this module claims are exactly the six the manifest
/// declares — asserted as an **exact set**, not as six `contains`.
#[test]
fn this_module_draws_exactly_the_six_markup_kinds() {
    use crate::shell::manifest::{
        COLOUR_SWATCH, FONT_COLOUR, FONT_FACE, FONT_SIZE, MARKUP_DASH, MARKUP_ENDINGS, MARKUP_FILL,
        MARKUP_OPACITY, MARKUP_STROKE, MARKUP_WIDTH,
    };
    let mine: Vec<&str> = [
        MARKUP_STROKE,
        MARKUP_FILL,
        MARKUP_WIDTH,
        MARKUP_DASH,
        MARKUP_OPACITY,
        MARKUP_ENDINGS,
    ]
    .into_iter()
    .filter(|k| command_for(k).is_some())
    .collect();
    assert_eq!(
        mine,
        [
            MARKUP_STROKE,
            MARKUP_FILL,
            MARKUP_WIDTH,
            MARKUP_DASH,
            MARKUP_OPACITY,
            MARKUP_ENDINGS
        ]
    );
    for foreign in [COLOUR_SWATCH, FONT_FACE, FONT_SIZE, FONT_COLOUR] {
        assert!(
            command_for(foreign).is_none(),
            "`{foreign}` is not a Format ▸ Markup control and must not be claimed here"
        );
    }
    assert!(command_for("nonsense").is_none());
}

/// **Every parked edit sets exactly one field of `MarkupStyle`.**
#[test]
fn only_one_field_is_ever_set() {
    let cases = [
        MarkupEdit::Stroke(StyleEdit::Set(Color::Rgb(1.0, 0.0, 0.0))),
        MarkupEdit::Stroke(StyleEdit::Clear),
        MarkupEdit::Interior(StyleEdit::Set(Color::Gray(0.5))),
        MarkupEdit::Interior(StyleEdit::Clear),
        MarkupEdit::Width(2.5),
        MarkupEdit::Opacity(StyleEdit::Set(0.5)),
        MarkupEdit::Opacity(StyleEdit::Clear),
        MarkupEdit::Endings(StyleEdit::Set((LineEnding::None, LineEnding::OpenArrow))),
        MarkupEdit::Endings(StyleEdit::Clear),
        // Both arms of the dash, and `Clear` is not a filler case: it is
        // the *Solid* entry, and a bug that routed it to `None` would leave
        // the mark's existing dash in place while the chooser showed Solid.
        MarkupEdit::Dash(StyleEdit::Set(
            pdfcer_core::annot_author::BorderDash::new(vec![4.0, 2.0])
                .expect("[4 2] satisfies §8.4.3.6"),
        )),
        MarkupEdit::Dash(StyleEdit::Clear),
    ];
    // `&cases` and a `clone`, because `MarkupEdit` is not `Copy` —
    // `BorderDash` owns a `Vec<f64>`. The clone sits here rather than in
    // `into_style`'s signature: the production path constructs each edit once
    // and consumes it once, so the cost belongs in a test and not on the hot
    // path.
    for case in &cases {
        let style = case.clone().into_style();
        let set = usize::from(style.stroke.is_some())
            + usize::from(style.interior.is_some())
            + usize::from(style.width.is_some())
            + usize::from(style.opacity.is_some())
            + usize::from(style.dash.is_some())
            + usize::from(style.endings.is_some());
        assert_eq!(
            set, 1,
            "`{case:?}` sets {set} fields of MarkupStyle; it must set exactly one"
        );
        assert!(!style.is_empty(), "`{case:?}` produced a no-op override");
    }
}

/// **The arrowhead chooser changes *which* ends, never *what shape*.**
#[test]
fn changing_which_ends_preserves_the_arrowhead_shape() {
    use LineEnding::{ClosedArrow, None as NoEnd, OpenArrow};
    // A closed head at the end, moved to both ends: still closed.
    let pair = (NoEnd, ClosedArrow);
    assert_eq!(Ends::of(pair), Ends::End);
    assert_eq!(
        Ends::Both.applied(arrow_shape(pair)),
        (ClosedArrow, ClosedArrow)
    );
    // An open head, moved to the start: still open.
    let pair = (NoEnd, OpenArrow);
    assert_eq!(Ends::of(pair), Ends::End);
    assert_eq!(Ends::Start.applied(arrow_shape(pair)), (OpenArrow, NoEnd));
    // A plain line given its first head gets the one pdfcer's pen authors.
    let pair = (NoEnd, NoEnd);
    assert_eq!(Ends::of(pair), Ends::None);
    assert_eq!(Ends::End.applied(arrow_shape(pair)), (NoEnd, OpenArrow));
    // Every position round-trips through `of`, whichever shape it is in.
    for shape in [OpenArrow, ClosedArrow] {
        for &ends in Ends::ALL {
            assert_eq!(Ends::of(ends.applied(shape)), ends);
        }
    }
}

/// The four positions have four distinct, non-empty labels, in the
/// documented order.
#[test]
fn the_four_arrowhead_positions_are_named_and_ordered() {
    assert_eq!(
        Ends::ALL,
        [Ends::None, Ends::Start, Ends::End, Ends::Both].as_slice()
    );
    let mut labels: Vec<&str> = Ends::ALL.iter().map(|e| e.label()).collect();
    for label in &labels {
        assert!(!label.trim().is_empty());
    }
    let total = labels.len();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), total, "two positions share a label");
}

/// A swatch shows only colours it can show without converting, and an sRGB
/// pick round-trips back out as `DeviceRGB`.
#[test]
fn a_swatch_shows_only_colours_it_can_show_without_converting() {
    assert_eq!(rgb_of(Color::Rgb(1.0, 0.0, 0.0)), Some([255, 0, 0]));
    assert_eq!(rgb_of(Color::Gray(0.0)), Some([0, 0, 0]));
    assert_eq!(rgb_of(Color::Gray(1.0)), Some([255, 255, 255]));
    assert_eq!(rgb_of(Color::Cmyk(0.0, 0.0, 0.0, 1.0)), None);
    assert_eq!(
        srgb_to_colour([128, 128, 128]),
        Color::Rgb(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0)
    );
    assert_eq!(rgb_of(srgb_to_colour([1, 2, 3])), Some([1, 2, 3]));
}

/// The width range is the same one the markup pen offers, and the same
/// one the Properties panel offers.
#[test]
fn the_width_range_matches_the_pen_that_authors() {
    assert!((MIN_WIDTH_PT - crate::canvas::markup::pen::MIN_WIDTH_PTS).abs() < f64::EPSILON);
    assert!((MAX_WIDTH_PT - crate::canvas::markup::pen::MAX_WIDTH_PTS).abs() < f64::EPSILON);
}

/// A `Current` carrying every value a control could want, so that the only
/// thing left deciding whether one draws is [`Current::support`].
fn every_value_present(subtype: &[u8]) -> Current {
    Current {
        support: MarkupStyleSupport::for_subtype(subtype),
        stroke: Some([0, 0, 0]),
        interior: Some([255, 255, 255]),
        interior_set: true,
        width: Some(2.0),
        // `Solid` is a **value**, not an absence — `linestyle::read` is
        // total and every dictionary answers it — so there is no "dash
        // missing" state for this helper to over-supply. Which is why
        // `offers_dash` is `takes_border` alone.
        dash: crate::canvas::markup::linestyle::DashReading::Solid,
        alpha: Some(1.0),
        endings: Some((LineEnding::None, LineEnding::OpenArrow)),
        endings_key_present: true,
    }
}

/// **What hides a control is the *engine's* answer, not the shape of the
/// `MarkupSpec` arm this module read.**
#[test]
fn the_engines_answer_is_what_hides_a_control_not_the_spec_arm() {
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
        // Not a markup shape at all. `for_subtype`'s documented
        // conservative direction: unrecognised supports nothing.
        (b"FreeText", false, false, false),
    ];
    for &(subtype, fill, width, endings) in expected {
        let name = String::from_utf8_lossy(subtype).into_owned();
        let current = every_value_present(subtype);
        assert_eq!(
            current.offers_fill(),
            fill,
            "/{name}: the Fill swatch must follow MarkupStyleSupport::takes_interior"
        );
        assert_eq!(
            current.offers_width(),
            width,
            "/{name}: the width field must follow MarkupStyleSupport::takes_border"
        );
        // The line-style chooser is the fourth reader of the same answer —
        // `/BS` `/D` is a border property, so `takes_border` governs it too,
        // and `pdfcer_core::edit::EditSession::set_markup_style` refuses a
        // `dash` on a text markup by the same `takes_border` test it refuses a
        // `width` with, raising `EditError::StylePropertyNotApplicable`.
        assert_eq!(
            current.offers_dash(),
            width,
            "/{name}: the line-style chooser must follow \
             MarkupStyleSupport::takes_border, like the width beside it"
        );
        assert_eq!(
            current.offers_endings(),
            endings,
            "/{name}: the arrowhead chooser must follow \
             MarkupStyleSupport::takes_endings"
        );
    }
}

/// …and the engine is asked, rather than a table like the one above being
/// kept here and consulted.
#[test]
fn each_predicate_reads_the_engines_flag_and_nothing_else() {
    for subtype in [
        &b"Square"[..],
        b"Circle",
        b"Polygon",
        b"Line",
        b"PolyLine",
        b"Ink",
        b"Highlight",
        b"Underline",
        b"StrikeOut",
        b"Squiggly",
        b"FreeText",
        b"",
    ] {
        let support = MarkupStyleSupport::for_subtype(subtype);
        let current = every_value_present(subtype);
        assert_eq!(current.offers_fill(), support.takes_interior);
        assert_eq!(current.offers_width(), support.takes_border);
        assert_eq!(current.offers_dash(), support.takes_border);
        assert_eq!(current.offers_endings(), support.takes_endings);
    }
}

/// **The fifth state: `Clear` *removes* `/LE`, and the four positions
/// *write* it.**
#[test]
fn clearing_the_arrowheads_removes_the_key_where_choosing_a_position_writes_it() {
    use LineEnding::{None as NoEnd, OpenArrow};

    // The positive control. Every one of the four positions writes /LE.
    for &ends in Ends::ALL {
        let style = MarkupEdit::Endings(StyleEdit::Set(ends.applied(OpenArrow))).into_style();
        assert!(
            matches!(style.endings, Some(StyleEdit::Set(_))),
            "{ends:?} must WRITE /LE, or the chooser has stopped working and \
             the Clear assertion below proves nothing"
        );
    }
    // …including the one that draws no heads, which is the whole point:
    // "no arrowheads" is a written array, not an absent key.
    assert_eq!(
        MarkupEdit::Endings(StyleEdit::Set((NoEnd, NoEnd)))
            .into_style()
            .endings,
        Some(StyleEdit::Set((NoEnd, NoEnd))),
        "\"No arrowheads\" states /LE [/None /None]; it does not remove the key"
    );
    // And the fifth state, which is the other one.
    assert_eq!(
        MarkupEdit::Endings(StyleEdit::Clear).into_style().endings,
        Some(StyleEdit::Clear),
        "Clear must reach the engine as Clear — it is what removes /LE"
    );
}

/// **The removal is absent when there is no `/LE` to remove.**
#[test]
fn the_removal_is_offered_only_when_the_file_carries_a_line_ending_entry() {
    let mut line = every_value_present(b"Line");
    line.endings_key_present = false;
    assert!(
        !line.offers_endings_clear(),
        "a /Line with no /LE has nothing to remove"
    );
    line.endings_key_present = true;
    assert!(
        line.offers_endings_clear(),
        "a /Line that carries /LE must be able to give it back"
    );

    // And never on a subtype with no chooser to put it in, whatever the
    // dictionary happens to hold.
    let mut highlight = every_value_present(b"Highlight");
    highlight.endings_key_present = true;
    assert!(!highlight.offers_endings_clear());
}

/// The default `Current` — what a mark whose dictionary could not be read
/// gets — offers nothing, and gets that answer from the engine.
#[test]
fn a_mark_that_could_not_be_read_offers_nothing() {
    let nothing = Current::default();
    assert_eq!(nothing.support, MarkupStyleSupport::for_subtype(b""));
    assert!(!nothing.offers_fill());
    assert!(!nothing.offers_width());
    assert!(!nothing.offers_endings());
    assert!(!nothing.offers_endings_clear());
}
