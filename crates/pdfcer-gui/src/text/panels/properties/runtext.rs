//! Words for the text section's render-mode chooser (`G034`) and the
//! single-run width field (`G038`).

/// Label for the render-mode chooser.
#[must_use]
pub const fn text_render_label() -> &'static str {
    "Drawn as"
}

/// What the render-mode chooser shows before a mode is picked. The PDF does
/// not record a mode per run that pdfcer reads back, so nothing is claimed.
#[must_use]
pub const fn text_render_choose() -> &'static str {
    "Choose…"
}

/// The render-mode chooser's hover text.
#[must_use]
pub const fn text_render_hint() -> &'static str {
    "How the selected text is drawn. Invisible text is still searchable and selectable, which is how an OCR text layer is stored."
}

/// The name of render mode `mode` (`Tr` 0..=7) in the chooser.
#[must_use]
pub const fn text_render_mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Visible (filled)",
        1 => "Outlined",
        2 => "Filled and outlined",
        3 => "Invisible (OCR text)",
        4 => "Filled, and clips what follows",
        5 => "Outlined, and clips what follows",
        6 => "Filled and outlined, and clips what follows",
        _ => "Invisible, and clips what follows",
    }
}

/// Heading for the single-run width field.
#[must_use]
pub const fn run_width_heading() -> &'static str {
    "Fit to width"
}

/// Label for the width field.
#[must_use]
pub const fn run_width_label() -> &'static str {
    "Width"
}

/// The width field's hover text when it can be used.
#[must_use]
pub const fn run_width_hint() -> &'static str {
    "Type the width this piece of text should take up, in points. pdfcer stretches or squeezes the letters to fit and keeps the text after it where it is."
}

/// Hover text: the selection is not exactly one piece of text.
#[must_use]
pub const fn run_width_needs_one_run() -> &'static str {
    "Select one line of a text object that is a single piece of text to set its width. A line made of several pieces, or several lines, cannot be fitted as one."
}

/// Hover text: the text sits inside a form or container, which this verb does
/// not reach.
#[must_use]
pub const fn run_width_inside_form() -> &'static str {
    "Text inside a form or group cannot be fitted to a width yet."
}

/// Hover text: the run has no length along its line on the page.
#[must_use]
pub const fn run_width_no_baseline() -> &'static str {
    "This text has no length along its line on the page, so it has no width to set."
}

/// Hover text: any other structural refusal.
#[must_use]
pub const fn run_width_unavailable() -> &'static str {
    "pdfcer cannot fit this text to a width."
}
