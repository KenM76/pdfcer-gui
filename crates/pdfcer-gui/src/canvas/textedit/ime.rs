//! # `canvas::textedit::ime` — composed input (CJK, accents, the emoji panel)
//!
//! Contract: the draft is painted into the canvas rather than being a focused
//! egui widget, so nothing else asks the platform for an input method. While
//! a draft is shown and no text field holds the keyboard, [`show`] does, at
//! the draft's line box; without it Windows never starts a composition. The
//! composition in progress is held here, never in the draft, and drawn under
//! the line box, underlined, as a pre-commit affordance. A commit is typed
//! into the draft as a keystroke is (`edits::Keys::event`), so it meets the
//! same sieve, re-face and undo.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/ime.md`.

use egui::{Id, Rect};

const PREEDIT_KEY: &str = "pdfcer-textedit-ime-preedit"; // ui-text-exempt: a memory key, never displayed
const AREA_KEY: &str = "pdfcer-textedit-ime-area"; // ui-text-exempt: a memory key, never displayed
/// The composition box's region in the trace.
pub const REGION: &str = "text-ime-preedit"; // ui-text-exempt: trace region name, never displayed

/// Ask for an input method at `line` (screen space) and draw any composition
/// in progress under it. The candidate window opens at the line's start.
pub fn show(ui: &egui::Ui, ctx: &egui::Context, line: Rect) {
    // typing-guard-exempt: a focused text field asks for its own input method.
    if ctx.text_edit_focused() {
        return;
    }
    ctx.output_mut(|o| {
        o.ime = Some(egui::output::IMEOutput {
            rect: line,
            cursor_rect: Rect::from_min_max(line.left_top(), line.left_bottom()),
            should_interrupt_composition: false,
        });
    });
    announce(ctx, line);
    paint(ui, ctx, line);
}

/// Trace the input method's area when it moves.
fn announce(ctx: &egui::Context, line: Rect) {
    let last = ctx.data(|d| d.get_temp::<Rect>(Id::new(AREA_KEY)));
    if last == Some(line) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(Id::new(AREA_KEY), line));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-ime-area x={:.0} y={:.0} w={:.0} h={:.0}",
            line.left(),
            line.top(),
            line.width(),
            line.height()
        )
    });
}

/// Hold `text` as the composition in progress; empty ends it.
pub fn set_preedit(ctx: &egui::Context, text: &str) {
    ctx.data_mut(|d| {
        if text.is_empty() {
            d.remove::<String>(Id::new(PREEDIT_KEY));
        } else {
            d.insert_temp(Id::new(PREEDIT_KEY), text.to_owned());
        }
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-ime-preedit chars={}", text.chars().count())
    });
}

fn preedit(ctx: &egui::Context) -> Option<String> {
    ctx.data(|d| d.get_temp::<String>(Id::new(PREEDIT_KEY)))
}

/// The composition, on the editor's surface just under `line`, underlined.
fn paint(ui: &egui::Ui, ctx: &egui::Context, line: Rect) {
    let Some(text) = preedit(ctx) else {
        return;
    };
    let theme = egui_shell::theme::Theme::of(ctx);
    let painter = ui.painter();
    let font = egui::FontId::proportional((line.height() * 0.8).clamp(10.0, 28.0));
    let laid = painter.layout_no_wrap(text, font, theme.palette.text);
    let at = egui::pos2(line.left(), line.bottom() + 2.0);
    let rect = Rect::from_min_size(at, laid.rect.size() + egui::vec2(4.0, 4.0));
    painter.rect_filled(rect, 0.0, theme.palette.surface);
    painter.galley(at + egui::vec2(2.0, 2.0), laid, theme.palette.text);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0, theme.palette.accent),
    );
    crate::diag::ui_rect(REGION, rect);
}
