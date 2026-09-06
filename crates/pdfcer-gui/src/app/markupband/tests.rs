//! # `app::markupband` tests — the Format ▸ Markup band's own assertions
//!
//! ## ★★ Why they live in a file of their own
//!
//! **R2.** `markupband.rs` crossed 1,500 lines on 2026-09-06, when two tracks
//! landed in it on the same afternoon: the line-style chooser and the deletion
//! of this crate's copy of the engine's subtype list. The seam taken is the
//! one `canvas::annotnodes` and `app::conditions` already use — module beside
//! module, `#[cfg(test)] mod tests;` — and it is a **subject** seam and not an
//! arithmetic one: what is left in the parent draws controls, what moved here
//! asserts what they decide.
//!
//! ## ★★★ What these can and cannot prove, stated first
//!
//! **They cannot prove an operator can restyle a mark.** Every test here calls
//! a function directly, and R1's whole point is that a passing unit test is not
//! a report of working software. Nothing below draws a pixel, opens a combo, or
//! reaches `set_markup_style`; `tools/ui-verify` is the instrument for that.
//!
//! What they DO prove, and it is the half this module was rewritten for:
//!
//! 1. **The engine's subtype list is ASKED, not restated.** Every visibility
//!    predicate is checked against `MarkupStyleSupport::for_subtype` rather
//!    than against a table kept here — see
//!    [`each_predicate_reads_the_engines_flag_and_nothing_else`], and
//!    `NO_SURFACE.md` §1a for why a table alone would be two copies of one
//!    constant that cannot disagree.
//! 2. **A parked edit sets exactly one field**, which no operator could
//!    report going wrong: a width that silently reverts one frame after it is
//!    set reads as *the drag did not take*.
//! 3. **`Clear` reaches the engine as `Clear`.** The fifth state is the one
//!    change whose whole effect is invisible on screen and visible only in the
//!    saved bytes, so a test is the only place it can be seen at all.
//!
//! ⚠ **Every negative assertion here is paired with a positive control**, per
//! the engine's own methodology note of 2026-09-06: its first foreign-appearance
//! test asserted `!appearance_rebaked` and *passed with the whole feature
//! disabled*, because a "not X" assertion is vacuous when the thing that would
//! produce X is absent.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy. `canvas::annotnodes::tests` carries the
// same line for the same reason; this file learned it the same way, by the gate
// biting the moment the split landed.
#![cfg(test)]

use super::*;

/// ★★★ **Every custom kind the manifest declares for this group is drawn
/// here, and every kind drawn here backs a registered command.**
///
/// The assertion that closes the gap `manifest::COLOUR_SWATCH`'s own doc
/// comment records: the manifest wrote a custom kind, **no renderer ever
/// matched it**, and the Markup ▸ Style group drew a caption over an empty
/// band for the whole of v0.1.0 with nothing anywhere reporting the
/// mismatch. The shell reserves the item's space, the application declines
/// to draw, and the only symptom is a gap.
///
/// It is asserted through `manifest::CUSTOM_BACKED`, which already pairs a
/// command id with the kind that draws it and is already tested against the
/// manifest in both directions. Reading it here makes the chain complete:
/// manifest → register → renderer → registry.
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
///
/// ★ A sixth kind added here and not to the manifest is a renderer arm
/// nothing can ever reach; a sixth added to the manifest and not here is the
/// empty-band defect above. Only an equality catches both.
///
/// ★★ It also asserts that this module does **not** claim the Font group's
/// three or the Markup tab's pen swatch. `COLOUR_SWATCH` is the one that
/// matters: it is a colour control called `colour_swatch` that sits two tabs
/// away and means the opposite thing about *when* — it chooses the colour of
/// the mark you are about to draw, where `MARKUP_STROKE` restyles the mark
/// you have selected. Claiming it here would put a document-editing verb
/// behind a control that edits `PdfcerApp::pen`.
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

/// ★★★ **Every parked edit sets exactly ONE field of `MarkupStyle`.**
///
/// The rule `MarkupStyle`'s own doc states — *"a Format tab whose colour
/// picker also had to restate the current width would overwrite whatever
/// the operator had set from the other control"* — asserted rather than
/// trusted, because the failure it prevents has no symptom the operator
/// could report: a width silently reverting one frame after it was set
/// reads as *the drag did not take*.
///
/// ★ Counted by comparing against `MarkupStyle::default()` field by field,
/// which is the only way to state "exactly one" over a struct of `Option`s.
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
        // ★ Both arms of the dash, and `Clear` is not a filler case: it is
        // the *Solid* entry, and a bug that routed it to `None` would leave
        // the mark's existing dash in place while the chooser showed Solid.
        MarkupEdit::Dash(StyleEdit::Set(
            pdfcer_core::annot_author::BorderDash::new(vec![4.0, 2.0])
                .expect("[4 2] satisfies §8.4.3.6"),
        )),
        MarkupEdit::Dash(StyleEdit::Clear),
    ];
    // ★ `&cases` and a `clone`, because `MarkupEdit` stopped being `Copy`
    // when the dash arrived — `BorderDash` owns a `Vec<f64>`. This is the
    // one clone the change cost anywhere, it is in a test, and it is here
    // rather than in `into_style`'s signature: the production path
    // constructs each edit once and consumes it once, which is what the
    // reply's warning about cloning in a hot path was about.
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

/// ★★ **The arrowhead chooser changes WHICH ends, never WHAT SHAPE.**
///
/// The property that makes a four-entry list honest over a nine-value
/// field: a mark drawn with closed arrowheads keeps them when the operator
/// moves the head to the other end. Without it the chooser would answer a
/// question nobody asked, and the rewrite would be invisible here and
/// visible in another viewer.
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
///
/// ★ The **order** is the assertion worth making: the chooser lists them
/// fewest-endings-first, so an operator scanning for "both" finds it last
/// every time, and a reordering that put the common case first would be a
/// change to the control's shape rather than to its wording.
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
///
/// ★ Grey is accepted **in** and never produced **out**, which is the
/// asymmetry `srgb_to_colour` argues: reading `Gray(v)` as an equal-channel
/// swatch is lossless, and writing an equal-channel pick back as `Gray`
/// would be pdfcer choosing a colour space the operator did not ask for.
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

/// ★ The width range is the same one the markup pen offers, and the same
/// one the Properties panel offers.
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

/// A `Current` carrying every value a control could want, so that the only
/// thing left deciding whether one draws is [`Current::support`].
///
/// ★ Deliberately over-supplied: a width, an interior, a dash, an endings
/// pair and a `/LE` in the file, all at once, on a mark no real subtype
/// could be. That is the point — a test that fed each control only the
/// values its own subtype really carries could not tell "the engine's
/// answer hid it" from "there was no value to show".
fn every_value_present(subtype: &[u8]) -> Current {
    Current {
        support: MarkupStyleSupport::for_subtype(subtype),
        stroke: Some([0, 0, 0]),
        interior: Some([255, 255, 255]),
        interior_set: true,
        width: Some(2.0),
        // ★ `Solid` is a **value**, not an absence — `linestyle::read` is
        // total and every dictionary answers it — so there is no "dash
        // missing" state for this helper to over-supply. Which is why
        // `offers_dash` is `takes_border` alone.
        dash: crate::canvas::markup::linestyle::DashReading::Solid,
        alpha: Some(1.0),
        endings: Some((LineEnding::None, LineEnding::OpenArrow)),
        endings_key_present: true,
    }
}

/// ★★★ **What hides a control is the ENGINE's answer, not the shape of the
/// `MarkupSpec` arm this module read.**
///
/// This is the assertion that the workaround is gone. Until 2026-09-06 the
/// three answers came from `Current::read`'s `match`: `fillable` was set on
/// four arms, the width control was hidden by the `TextMarkup` arm making
/// no width assignment, and the chooser was hidden by `endings` being
/// `None`. Every one of those was a copy of a list `pdfcer-core` owns, and
/// this project filed it: *"the first subtype that gains or loses a border
/// is the day our copy is wrong and nothing tells us."*
///
/// ★ The `Current` fed in has **every value present** for every subtype, so
/// the values cannot be what differs. If a control is withheld here it is
/// because `MarkupStyleSupport::for_subtype` said so, and if a control is
/// offered it is for the same reason.
///
/// ★★ Both directions, per the engine's own methodology note: a `Highlight`
/// row asserting three `false`s would pass with the whole feature deleted
/// — every predicate returning `false` satisfies it — so the `Square`,
/// `Line` and `Ink` rows are the positive control that makes the negative
/// one mean something.
///
/// Falsified by restoring the old rule — `offers_width` reading only
/// `self.width.is_some()` turned the `Highlight` row red; `offers_fill`
/// reading `self.interior_set` turned the `Line` and `Ink` rows red.
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
        // and `set_markup_style` refuses a `dash` on a text markup by the
        // same predicate it refuses a `width` with (`edit.rs:26469`).
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
///
/// ★ The test above is a table, and a table is the very thing this session
/// deleted — so it needs this beside it. It asserts the **relation**: for
/// every subtype named above, each predicate equals the corresponding field
/// of `MarkupStyleSupport::for_subtype`, whatever that field happens to
/// say. The table pins today's behaviour; this pins the *source*, and the
/// day the engine changes an answer the table fails and this one does not,
/// which is how a reader is told which of the two to edit.
///
/// `NO_SURFACE.md` §1a's rule is the one being obeyed — *"two copies of one
/// constant cannot disagree"*, so assert a relation and not a magnitude.
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

/// ★★★ **The fifth state: `Clear` REMOVES `/LE`, and the four positions
/// WRITE it.**
///
/// The distinction the engine shipped `StyleEdit` for, asserted at the one
/// place this module decides it. `Set((None, None))` and `Clear` are the
/// pair that matters: they draw the same line and are different files, and
/// before 2026-09-06 only the first was expressible.
///
/// ★★ **Paired, deliberately.** The engine's reply records a first attempt
/// whose assertion was `!appearance_rebaked` and which *passed with the
/// whole feature disabled*, because a "not X" assertion is vacuous when the
/// thing that would produce X is absent. So *"Clear does not write an
/// array"* is not asserted on its own — the `Set` row asserting an array
/// **is** written sits beside it, and a `MarkupEdit::Endings` that had
/// stopped producing anything at all would fail that row.
///
/// Falsified by re-wrapping the payload as `StyleEdit::Set(..)` in
/// `into_style`, which is the code that shipped this morning: the `Clear`
/// row went red and the `Set` row stayed green.
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

/// ★★ **The removal is absent when there is no `/LE` to remove.**
///
/// [`fill`]'s Clear rule applied to the chooser: *a Clear beside a mark
/// that has nothing to clear is a control whose only possible effect is an
/// undo entry the operator did not earn.* It needs its own field because
/// `spec_from_dict` cannot answer it — Table 176's default means an absent
/// `/LE` and a written `[/None /None]` both read back as `(None, None)`,
/// which is right for a reader whose subject is the picture and useless to
/// a control whose subject is the difference.
///
/// ★ Both directions again. "Absent when the key is absent" alone would
/// pass with the action deleted.
///
/// Falsified by making `offers_endings_clear` return `self
/// .offers_endings()`, which turned the first assertion red.
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
///
/// ★ Asserted against `for_subtype(b"")` rather than against three literal
/// `false`s, for this module's whole reason: an all-`false` literal here
/// would be the last surviving copy of the engine's list, three entries
/// long.
#[test]
fn a_mark_that_could_not_be_read_offers_nothing() {
    let nothing = Current::default();
    assert_eq!(nothing.support, MarkupStyleSupport::for_subtype(b""));
    assert!(!nothing.offers_fill());
    assert!(!nothing.offers_width());
    assert!(!nothing.offers_endings());
    assert!(!nothing.offers_endings_clear());
}
