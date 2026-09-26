//! # `canvas::forms::textbox` — the live text box laid over a widget rectangle
//!
//! One mechanism, two callers: [`super::editor`]'s `/Tx` path and
//! [`super::choosing`]'s editable combo box. Both lay an `egui::TextEdit` over
//! a field's own rectangle on the page, and both have to answer the same three
//! questions about how it is dressed — which font, which end of the box the
//! text is set against, and what colour the box is.
//!
//! ## Contract
//!
//! [`lay`] builds and places the editor and returns its `Response`. It owns
//! **appearance only**. Requesting focus, seating the caret, reading Escape,
//! deciding what a commit is and what to do with the typed string all stay
//! with the caller, because the two callers answer them differently: a text
//! field commits `FillText` on focus loss, an editable combo commits
//! `SetChoice` on Enter and has a popup list to close first.
//!
//! ## Why the dressing is shared and the lifecycle is not
//!
//! The three appearance rules below are *properties of the document*, so two
//! independent statements of them is two places for a form to be drawn with
//! the wrong quadding or a field to flash white. The lifecycle rules are
//! *properties of the interaction*, and the two interactions genuinely differ.
//! Sharing the first and not the second is the seam, not a compromise.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms/textbox.md`.

use egui::{Id, Rect, Ui};
use pdfcer_core::vartext::Quadding;

use super::boxes::{editor_align, editor_font_size};

/// Everything [`lay`] needs that is not the draft itself.
pub(super) struct Spec<'a> {
    /// The editor's `egui` id — [`super::Focus::editor_id`] for both callers,
    /// so the caret state survives between frames.
    pub(super) id: Id,
    /// Where to put it, in screen space.
    pub(super) rect: Rect,
    /// `/Q`, resolved through the field tree by `pdfcer-core`.
    pub(super) align: Quadding,
    /// `/MK` `/BG` as sRGB, when the widget has one.
    pub(super) fill: Option<[f32; 3]>,
    /// `/Ff` `Multiline`. Always `false` for a combo box.
    pub(super) multiline: bool,
    /// `/Ff` `Password`. Always `false` for a combo box.
    pub(super) password: bool,
    /// The field's name, when this frame is the one that should report what
    /// the tint resolved to. `None` on every other frame — see [`lay`]'s .
    pub(super) trace: Option<&'a str>,
}

/// Build the editor, dress it from the document, and place it.
pub(super) fn lay(ui: &mut Ui, draft: &mut String, spec: &Spec<'_>) -> egui::Response {
    let ctx = ui.ctx().clone();
    let font = egui::FontId::proportional(editor_font_size(spec.rect.height()));
    let halign = editor_align(spec.align);
    let tint = spec
        .fill
        .and_then(|srgb| egui_shell::theme::Theme::foreign_fill_pair(&ctx, srgb));

    let mut edit = if spec.multiline {
        // escape-disposition: commits — `canvas::forms`' own `Key::Escape` arm
        // runs ahead of the `lost_focus` branch and writes the draft.
        egui::TextEdit::multiline(draft)
    } else {
        // escape-disposition: commits — same arm in `canvas::forms`; a password
        // field is no less the operator's typing for being drawn as dots.
        egui::TextEdit::singleline(draft).password(spec.password)
    }
    .id(spec.id)
    .horizontal_align(halign)
    .font(egui::FontSelection::from(font));
    if let Some((fill, ink)) = tint {
        edit = edit.background_color(fill).text_color(ink);
    }

    if let Some(field) = spec.trace {
        crate::diag::trace(|| match (spec.fill, tint) {
            (None, _) => {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-editor-tint field={field} bg=absent")
            }
            (Some(srgb), Some((_, ink))) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={field} bg={:.3},{:.3},{:.3} ink={},{},{}",
                srgb[0],
                srgb[1],
                srgb[2],
                ink.r(),
                ink.g(),
                ink.b()
            ),
            (Some(srgb), None) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={field} bg={:.3},{:.3},{:.3} declined=unreadable",
                srgb[0], srgb[1], srgb[2]
            ),
        });
    }
    ui.put(spec.rect, edit)
}

/// Put the caret at the end of the draft, or select the whole of it.
pub(super) fn seat(ctx: &egui::Context, id: Id, draft: &str, select_all: bool) {
    let end = egui::text::CCursor::new(draft.chars().count());
    let range = if select_all {
        egui::text::CCursorRange::two(egui::text::CCursor::new(0), end)
    } else {
        egui::text::CCursorRange::one(end)
    };
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        state.cursor.set_char_range(Some(range));
        egui::TextEdit::store_state(ctx, id, state);
    }
}
