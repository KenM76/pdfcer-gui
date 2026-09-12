//! **One function per markup property row.** Each takes the value the
//! parent already read and offers exactly one edit.
//!
//! # Why this is its own file
//!
//! `super` reached 1503 lines on 2026-09-12 and `check-file-size` went red.
//! R2's instruction for that moment is to find the seam rather than raise the
//! limit, and the seam was already here: these seven functions know nothing
//! about each other, nothing about `MarkupSpec`, and nothing about whether
//! they should be drawn. They are handed a [`Current`] and an `AnnotTarget`
//! and they build a control.
//!
//! What the parent keeps is the judgement: `section` decides there is a
//! selection worth a panel, `markup_rows` reduces the annotation to a
//! [`Current`] and asks [`MarkupStyleSupport`] which of these rows the
//! subtype even has. **A row that should not exist is never called** — it is
//! not called and then disabled, which is R9 (`no placeholders`), and it is
//! why none of these functions takes a `support` argument.
//!
//! # The contract every row in this file honours
//!
//! 1. **It pushes an [`Action`], never an edit.** Nothing here touches an
//!    `EditSession`. The action goes through
//!    `app::actions::funnel::vector_edit`, which is the single place an
//!    engine refusal becomes a sentence the operator reads.
//! 2. **A `Set` and a `Clear` are different writes and both are offered**
//!    when the key is present. [`StyleEdit::Set`] writes the key;
//!    [`StyleEdit::Clear`] removes it and restores the standard's default.
//!    A control that could only `Set` makes the key a one-way door, and the
//!    difference shows in another viewer even when it does not show here.
//!    `colour_row` carries the full argument; the others cite it.
//! 3. **Clear is absent, not greyed, when there is nothing to clear.** Same
//!    rule, same reason — greying is reserved for *temporarily*
//!    unavailable.
//! 4. **It reads `super`'s vocabulary and defines none of its own.**
//!    [`Current`], [`MIN_WIDTH_PT`], [`MAX_WIDTH_PT`] and [`DASH_WIDTH`] all
//!    live in the parent. The child reaching up is what keeps one owner for
//!    each; a constant copied down here would be the start of two answers to
//!    the same question.
//!
//! # ⚠ `swatch_of` is NOT here, and it looks like it should be
//!
//! It converts an annotation's `/C` or `/IC` into something a swatch can
//! show and reports whether that cost a narrowing. Its callers are the
//! parent's gathering `match` and [`super::textannot`] — neither is a row.
//! It is a colour-space conversion, and a file about controls is the wrong
//! home for one.

use egui::Ui;
use pdfcer_core::annot_author::{Color, LineEnding};
use pdfcer_core::edit::{MarkupStyle, StyleEdit};

use crate::app::actions::Action;
use crate::text::panels::properties as t;

use super::{Current, DASH_WIDTH, MAX_WIDTH_PT, MIN_WIDTH_PT};

/// The border colour.
///
/// ★ A **swatch plus a Clear**, not a swatch alone. `StyleEdit` has two arms
/// and they mean different things in the file: `Set` writes `/C`, and `Clear`
/// removes it, restoring the standard's default. A control that could only set
/// would make `/C` a one-way door — once an operator gave a mark a colour there
/// would be no way back to the file's own, and the difference is visible in
/// another viewer even when it is not visible here.
pub(super) fn colour_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.colour.rgb;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_colour_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Set(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    ))),
                    ..MarkupStyle::default()
                },
            });
        }
        // Absent when there is nothing to clear, rather than greyed: a Clear
        // beside a mark that has no `/C` is a control whose only possible
        // effect is an undo entry the operator did not earn.
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// **The interior colour, `/IC` — the Fill row.**
///
/// # ★★★ Why this exists at all, when the header used to argue against it
///
/// `MarkupStyle::interior` shipped in the engine with `set_markup_style` and had
/// **zero GUI callers** until 2026-09-06. The module header carries the argument
/// that kept it that way, and carries the correction beside it; the short form
/// is that *"a filled comment shape hides the drawing it is a comment about"* is
/// a sound reason to author `interior: None` and not a reason to refuse an
/// operator the ability to fill a shape they have already placed.
///
/// `canvas::markup::spec` is untouched by this. **No fill at author time, fill
/// available on restyle.**
///
/// # ★★ Absent for a shape with no interior — and the ENGINE says which
///
/// A `/Line`, an `/Ink`, a `/PolyLine` and a text markup have no interior for
/// `/IC` to mean anything in, so a Fill control there would be drawn, live, and
/// dropped on the floor. That is the same defect this session came to fix, one
/// control down, and R9's answer is the same: the row is absent, the way
/// [`width_row`] is absent for a highlight.
///
/// ⚠ **Corrected 2026-09-06.** The paragraph above used to justify the list
/// with *"`apply_markup_style` does not read `style.interior` on those arms"* —
/// a fact about the engine's source, restated here, where nothing checks it.
/// The list is now asked for: `MarkupStyleSupport::takes_interior`, through
/// [`Current::offers_fill`]. The old sentence was not wrong; it was a copy, and
/// a copy is what this project filed a request to be rid of.
///
/// # ★ The swatch shape mirrors [`colour_row`] exactly, including the Clear
///
/// Set writes `/IC`; Clear removes it and the shape is unfilled again. The one
/// addition is the word beside the swatch when there is no fill — a swatch
/// cannot show *absence*, and a black square next to "Fill" says the opposite of
/// the truth. See [`t::markup_fill_none`].
pub(super) fn fill_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_fill() {
        return;
    }
    let existing = current.interior.rgb;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_fill_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    interior: Some(StyleEdit::Set(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    ))),
                    ..MarkupStyle::default()
                },
            });
        }
        if existing.is_some() {
            // Absent when there is nothing to clear, for `colour_row`'s reason:
            // a Clear beside a shape with no `/IC` is a control whose only
            // possible effect is an undo entry the operator did not earn.
            if ui.button(t::markup_clear()).clicked() {
                actions.push(Action::SetMarkupStyle {
                    page: target.page,
                    id: target.id,
                    style: MarkupStyle {
                        interior: Some(StyleEdit::Clear),
                        ..MarkupStyle::default()
                    },
                });
            }
        } else {
            ui.label(egui::RichText::new(t::markup_fill_none()).small().weak());
        }
    });
}

/// **The border line style, `/BS` `/S` and `/D` — the Line style row.**
///
/// # ★★★ Why this exists, and what it took to make it SAFE
///
/// `RIBBON_IA.md` §5.8's Markup row lists eight controls and this was the
/// eighth. It read **⛔ no engine verb exists**, and that was true: `MarkupStyle`
/// carried colour, interior, width, opacity and endings and had no dash field at
/// all, so there was nothing for a control to reach.
///
/// The verb arrived on the afternoon of 2026-09-06 with two others, and the
/// **other two are what make this one safe to offer**:
///
/// * `/BS` `/S` and `/D` are read back on the way IN, so a restyle that does not
///   mention the dash **preserves** it — including a dash pdfcer never authored
///   (`pdfcer-core` `edit.rs:4396-4425`);
/// * `MarkupOptions::dash` authors one, so a shape can be *drawn* dashed rather
///   than drawn and then corrected.
///
/// ⇒ Before that, a dashed mark in the operator's file was silently converted to
/// a solid one the first time anything re-baked its appearance. The engine's
/// reply records that this was **wider than this shell reported**: the recolour
/// path was named, and `resize_annotation`, `reshape_annotation` and authoring
/// solidified a dash too — so it was reachable by dragging a resize handle or a
/// vertex, not only by pressing the colour swatch. All four carry it now. A
/// Line style control over the old engine would have been a control its
/// neighbours undid.
///
/// # ★★ The "way back to the default" is an ENTRY, not a Clear button
///
/// [`colour_row`] and [`fill_row`] each put `StyleEdit::Clear` behind its own
/// button, because in both cases the cleared state is *the absence of a key* and
/// has no name in a list: a swatch cannot show *no colour*. A border's cleared
/// state does have a name. `Clear` makes it **solid**, solid is Table 166's own
/// `/S` default, and *Solid* is the chooser's first entry — so a separate button
/// would be a second spelling of one act, and the two would eventually be
/// pressed expecting different things.
///
/// ★ It is also why the button's absence rule does not apply. A Clear beside a
/// mark with nothing to clear is *"a control whose only possible effect is an
/// undo entry the operator did not earn"*; a **Solid** entry beside a mark that
/// is already solid is simply the entry that is currently selected, and
/// [`crate::canvas::markup::linestyle::chooser`] reports nothing when the
/// current entry is picked again.
///
/// # ★ Absent for a subtype with no border
///
/// [`Current::offers_dash`], which is `MarkupStyleSupport::takes_border` and
/// nothing else — a highlight is a colour wash and has no `/BS` to dash. R9, and
/// the same answer [`width_row`] gives.
pub(super) fn dash_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_dash() {
        return;
    }
    ui.horizontal(|ui| {
        ui.label(t::markup_line_style_label());
        let picked = crate::canvas::markup::linestyle::chooser(
            ui,
            // ui-text-exempt: internal widget id, never displayed
            "properties-markup-line-style",
            current.dash,
            DASH_WIDTH,
        );
        // ★ The `Option` from `LineStyle::style_edit` is answered by raising
        // NOTHING — no action, no undo entry, no substituted pattern. It is
        // unreachable for the four offered styles
        // (`linestyle::tests::every_offered_pattern_is_one_the_engine_accepts`),
        // and writing Table 166's default in its place would be this panel
        // choosing a pattern the operator did not.
        if let Some(edit) = picked.and_then(crate::canvas::markup::linestyle::LineStyle::style_edit)
        {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    dash: Some(edit),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// The border width.
///
/// ⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`**, and
/// the engine says so in its own doc: the rectangle is derived from the
/// geometry plus a margin that contains the stroke and any arrowheads, so a
/// wider pen needs a bigger box. That is disclosed in [`t::markup_note`]
/// rather than here, because it is true of the section and not of this control
/// alone.
///
/// ★ Moved here on 2026-09-12. It sat above `dash_row`, run
/// together with that item's doc comment — so it documented `dash_row`
/// and this function had none. See `tools/gates/check-orphan-docs.py`.
pub(super) fn width_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ★ ABSENT rather than greyed when the mark has no border to widen — a
    // highlight is `/QuadPoints` and has nothing to stroke. R9: an unavailable
    // capability renders nothing. A greyed spinner here would be pdfcer
    // implying that a highlight could have a line width if only something were
    // different, and nothing is.
    if !current.offers_width() {
        return;
    }
    let Some(mut width) = current.width else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(t::markup_width_label());
        let response = ui.add(
            egui::DragValue::new(&mut width)
                .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                .speed(0.1)
                .suffix(t::markup_width_suffix()),
        );
        // ★ `drag_stopped` and `lost_focus`, not `changed`. A `DragValue` reports
        // a change on every pixel of a drag, and each one here is a
        // content-stream rewrite plus an undo entry — so a single drag across
        // the control would leave forty entries on the stack and re-plan the
        // annotation forty times. The colour swatch above needs no such guard:
        // it opens a popup and reports once, on the operator's pick.
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    width: Some(width),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// **The two line endings, `/LE` — what makes an arrow an arrow.**
///
/// # ★★★ Why this exists, when the header used to argue against it
///
/// `MarkupStyle::endings` shipped with `set_markup_style` and had **zero GUI
/// callers** until 2026-09-06. The header carries the overturned argument in
/// full; the short form is that a `/Line` with `/LE [/None /None]` is not a
/// different *kind* of mark — same `/Subtype`, same geometry, same verb — and
/// §12.5.6.7 treats the endings as style, which is why the engine put them in
/// the style struct rather than in a reshape.
///
/// # ★★ Two controls, ONE dictionary property — and why that is not a breach of
/// this module's "one field per action" rule
///
/// The rule at the top of this file forbids assembling a whole `MarkupStyle`
/// from what the widgets happen to show, because two controls read a frame apart
/// will disagree and the later action will silently revert the earlier one.
/// `/LE` is a **single two-element array** (Table 176) and
/// `MarkupStyle::endings` is a single `Option<(LineEnding, LineEnding)>`, so
/// changing one end necessarily sends both — there is no field that carries half
/// of it.
///
/// ★ That is still one field of one property, and it is still safe, for the
/// reason the rule actually rests on: the unchanged half comes from
/// [`Current::endings`], which was read **from the session this frame** through
/// `spec_from_dict`. It is not a widget's remembered value and cannot be stale.
/// The failure the rule prevents needs a second control holding a copy of a
/// value it set earlier, and neither of these two holds anything.
///
/// # ★ `/Line` only, and absent otherwise — the engine's answer, since
/// 2026-09-06
///
/// `MarkupStyleSupport::takes_endings` is `true` for `/Line` and for nothing
/// else, and [`Current::offers_endings`] is what asks it. A chooser on a
/// polygon would be live and would be **refused** —
/// `EditError::StylePropertyNotApplicable` at `edit.rs:26478` — so R9 says
/// absent and the engine agrees in writing.
///
/// ⚠ This paragraph used to read *"[`Current::endings`] is `Some` for a
/// `MarkupSpec::Line` and nothing else, which matches `apply_markup_style`
/// exactly."* True, and a restatement of the engine's list inside a shell. The
/// spec arm still supplies the **pair**, because that is a value; it no longer
/// decides whether the control exists.
pub(super) fn endings_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_endings() {
        return;
    }
    let Some((start, end)) = current.endings else {
        return;
    };
    let mut chosen = (start, end);
    ending_chooser(
        ui,
        t::markup_line_start_label(),
        "properties-markup-line-start", // ui-text-exempt: internal widget id, never displayed
        &mut chosen.0,
    );
    ending_chooser(
        ui,
        t::markup_line_end_label(),
        "properties-markup-line-end", // ui-text-exempt: internal widget id, never displayed
        &mut chosen.1,
    );
    // ★★★ **The fifth state — and yes, it belongs here as well as on the tab.**
    //
    // Three arguments, and the third is the one that settles it:
    //
    // 1. **§5.8's division of labour.** The panel *"carries everything"*; the
    //    tab carries what is reached for mid-gesture. A capability the tab has
    //    and the panel lacks is the one direction that rule forbids outright.
    // 2. **Consistency inside this section.** `/C`, `/IC` and `/CA` each offer
    //    a Clear on the same terms — present only when there is a key to
    //    remove. `/LE` was the odd one out **solely** because
    //    `MarkupStyle::endings` was a bare `Option` and the removal could not
    //    be expressed. That reason is gone, so the exception should go with it.
    // 3. **This is the surface an operator is on when the question arises.**
    //    *"Does this file still match the one my client sent me"* is asked
    //    while looking at an annotation's properties, not while reaching across
    //    a ribbon mid-drag.
    //
    // ★ It is a **button on its own row** rather than an entry in the two
    // choosers, and the reason is the tab's reason one level down: the choosers
    // answer *what shape at this end*, and `LineEnding::None` is already an
    // answer to that. A removal offered as a fourth shape would be a second
    // entry drawing exactly what *No end* draws, which is a distinction a
    // drafter cannot check by looking.
    //
    // ★ Absent when there is no `/LE` to remove — `Current::offers_endings_clear`
    // — which is `colour_row`'s rule and the same sentence: a Clear beside a
    // mark that has nothing to clear is a control whose only possible effect is
    // an undo entry the operator did not earn.
    if current.offers_endings_clear()
        && ui
            .button(t::markup_endings_clear())
            .on_hover_text(t::markup_endings_clear_hint())
            .clicked()
    {
        actions.push(Action::SetMarkupStyle {
            page: target.page,
            id: target.id,
            style: MarkupStyle {
                endings: Some(StyleEdit::Clear),
                ..MarkupStyle::default()
            },
        });
    }
    ui.label(
        egui::RichText::new(t::markup_line_ending_note())
            .small()
            .weak(),
    );
    if chosen != (start, end) {
        actions.push(Action::SetMarkupStyle {
            page: target.page,
            id: target.id,
            style: MarkupStyle {
                // ★ `StyleEdit::Set` since 2026-09-06 — the arm that WRITES
                // `/LE`, including `Set((None, None))` when the operator sets
                // both ends to *No end*. That is deliberately not the same act
                // as the Clear above: it states "no arrowheads" in the file
                // where Clear takes the statement out. Same picture, different
                // bytes; see `t::markup_endings_clear`.
                endings: Some(StyleEdit::Set(chosen)),
                ..MarkupStyle::default()
            },
        });
    }
}

/// One line-ending chooser, labelled.
///
/// ★ The list is [`ALL_ENDINGS`] rather than a literal written at each call
/// site, so the two choosers cannot come to offer different sets — and so that
/// an ending the engine learns to draw appears in both by editing one constant
/// whose exhaustiveness the compiler checks.
pub(super) fn ending_chooser(ui: &mut Ui, label: &str, id: &str, value: &mut LineEnding) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(t::markup_line_ending_name(*value))
            .show_ui(ui, |ui| {
                for ending in ALL_ENDINGS {
                    ui.selectable_value(value, ending, t::markup_line_ending_name(ending));
                }
            });
    });
}

/// Every line ending pdfcer can draw, in the order the choosers offer them.
///
/// ★★ `annot_author::LineEnding` has no `ALL` of its own — unlike `ArrowForm`,
/// which the ce-dimension panel iterates — so this list is written here, and
/// `the_ending_list_covers_every_variant_the_engine_has` is what stops it
/// drifting: that test `match`es an exhaustive set of variants with **no
/// wildcard**, so an ending the engine gains fails to compile here rather than
/// quietly going missing from the chooser.
///
/// `None` first because it is the state an operator reaches for when they want
/// a plain line, and because Table 176 lists it first.
pub(super) const ALL_ENDINGS: [LineEnding; 3] = [
    LineEnding::None,
    LineEnding::OpenArrow,
    LineEnding::ClosedArrow,
];

/// The constant opacity, `/CA`.
///
/// ★★ **This is the control `NO_SURFACE.md` recorded as "blocked on the engine"
/// for weeks, and the blocker was false.** `set_markup_style` has taken an
/// opacity since it shipped and writes `/CA` clamped to `0.0..=1.0`; the row
/// that said otherwise was a claim about a repository this project does not
/// build, and it could not fail a test. See `NO_SURFACE.md` §1b.
///
/// Shown as a **percentage**, because that is the unit every other application
/// an operator has used states opacity in, and `/CA`'s own `0.0..=1.0` is a
/// file-format detail they should never meet.
pub(super) fn opacity_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.alpha;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut percent = (existing.unwrap_or(1.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    ui.horizontal(|ui| {
        ui.label(t::markup_opacity_label());
        let response = ui.add(
            egui::DragValue::new(&mut percent)
                .range(0..=100)
                .speed(1.0)
                .suffix(t::markup_opacity_suffix()),
        );
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Set(f64::from(percent) / 100.0)),
                    ..MarkupStyle::default()
                },
            });
        }
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}
