//! **One function per markup property row.** Each takes the value the
//! parent already read and offers exactly one edit.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/markup/rows.md`.

use egui::Ui;
use pdfcer_core::annot_author::{Color, LineEnding};
use pdfcer_core::edit::{MarkupStyle, StyleEdit};
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::text::panels::properties as t;

use super::{Current, DASH_WIDTH, MAX_WIDTH_PT, MIN_WIDTH_PT};

/// The border colour.
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
        // The `Option` from `LineStyle::style_edit` is answered by raising
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
pub(super) fn width_row(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ABSENT rather than greyed when the mark has no border to widen — a
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
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut width,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let response = refusal.show(
            ui.add(
                widget
                    .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                    .speed(0.1)
                    .suffix(t::markup_width_suffix()),
            ),
        );
        // `drag_stopped` and `lost_focus`, not `changed`. A `DragValue` reports
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
    // **The fifth state — and yes, it belongs here as well as on the tab.**
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
    // It is a **button on its own row** rather than an entry in the two
    // choosers, and the reason is the tab's reason one level down: the choosers
    // answer *what shape at this end*, and `LineEnding::None` is already an
    // answer to that. A removal offered as a fourth shape would be a second
    // entry drawing exactly what *No end* draws, which is a distinction a
    // drafter cannot check by looking.
    //
    // Absent when there is no `/LE` to remove — `Current::offers_endings_clear`
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
                endings: Some(StyleEdit::Set(chosen)),
                ..MarkupStyle::default()
            },
        });
    }
}

/// One line-ending chooser, labelled.
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
pub(super) const ALL_ENDINGS: [LineEnding; 3] = [
    LineEnding::None,
    LineEnding::OpenArrow,
    LineEnding::ClosedArrow,
];

/// The constant opacity, `/CA`.
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
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let response = refusal.show(
            ui.add(
                widget
                    .range(0..=100)
                    .speed(1.0)
                    .suffix(t::markup_opacity_suffix()),
            ),
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
