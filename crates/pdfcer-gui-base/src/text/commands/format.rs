//! # `text::commands::format` — **the Format tab's command copy**
//!
//! Every entry here is copy for a control on the **contextual Format tab** or
//! on the canvas context menu that shadows it — the tab `RIBBON_IA.md` §5.8
//! describes as carrying *"what a user changes while working"*. They share a
//! subject (the thing the operator just clicked), a lifetime (visible only
//! while something is selected) and a vocabulary, and several of them cite
//! each other. The `view::*` and `file::*` sibling modules are drawn on the
//! same rule.
//!
//! Re-exported by [`super`] with `pub use format::*`, so a caller writes
//! `crate::text::commands::format_delete()` and never names this module. The
//! sibling modules hold the same contract, which is what keeps the split
//! internal rather than an API boundary.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/format.md`.

use super::CommandText;

// ===========================================================================
// FORMAT TAB (contextual)
// ===========================================================================

/// `format.delete`
#[must_use]
pub const fn format_delete() -> CommandText {
    CommandText::new(
        "Delete",
        "Remove what is selected from the page. Undo reverses it.",
    )
}

/// `format.select_text_line`
#[must_use]
pub const fn format_select_text_line() -> CommandText {
    CommandText::new(
        "Select this line of text",
        "Select just the line you clicked, instead of the whole block. Delete then removes \
         just that line. Press Escape to go back to the whole block.",
    )
}

/// `format.merge_text_runs`
///
/// The tooltip is static, so it states the conditions the greyed row stands
/// for; the exact refusal reaches the status line if the press is refused.
#[must_use]
pub const fn format_merge_text_runs() -> CommandText {
    CommandText::new(
        "Merge text runs",
        "Join the selected pieces of one line of text into a single run, keeping the first \
         piece's font and stretching it over the same width. Greyed when the pieces are not \
         next to each other, differ in font, size, spacing or colour, or when merging would \
         move the text after them. Undo reverses it.",
    )
}

/// `format.dimension_diameter`
#[must_use]
pub const fn format_dimension_diameter() -> CommandText {
    CommandText::new(
        "Show as diameter",
        "Make this circular ce dimension measure the diameter, with a leader across the \
         circle. The same switch as Radius / Diameter in Properties. Undo reverses it.",
    )
}

/// `format.dimension_radius`
#[must_use]
pub const fn format_dimension_radius() -> CommandText {
    CommandText::new(
        "Show as radius",
        "Make this circular ce dimension measure the radius, with a leader from the centre. \
         The same switch as Radius / Diameter in Properties. Undo reverses it.",
    )
}

/// `format.dimension_area`
#[must_use]
pub const fn format_dimension_area() -> CommandText {
    CommandText::new(
        "Show area",
        "Make this closed outline report the area it encloses instead of the distance round it. The corners stay where they are. The same switch as Perimeter / Area in Properties. Undo reverses it.",
    )
}

/// `format.dimension_perimeter`
#[must_use]
pub const fn format_dimension_perimeter() -> CommandText {
    CommandText::new(
        "Show perimeter",
        "Make this closed outline report the distance round it instead of its area. The corners stay where they are. The same switch as Perimeter / Area in Properties. Undo reverses it.",
    )
}

/// `format.select_form`
#[must_use]
pub const fn format_select_form() -> CommandText {
    CommandText::new(
        "Select the form",
        "Select the form that contains what you have selected, so you have one object you \
         can move, delete or copy. Everything drawn inside it moves with it.",
    )
}

/// `format.unshare_form`
#[must_use]
pub const fn format_unshare_form() -> CommandText {
    CommandText::new(
        "Give this page its own copy",
        "If this drawing is also drawn on other pages, this gives the page its own copy so \
         changes here will not affect them. pdfcer checks when you press, and says what it found. \
         Everything looks exactly the same afterwards.",
    )
}

/// `format.properties`
#[must_use]
pub const fn format_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "Show the Properties panel for what is selected — for a dimension, its \
         group, what it measured, and every setting it inherits from its group \
         or overrides for itself.",
    )
}

// ---------------------------------------------------------------------------
// The Font group — `RIBBON_IA.md` §5.8's "Text run" row
//
// **Every tooltip below has to read correctly in TWO states**, and that is the
// constraint that shapes all five of them.
//
// `egui_shell::ribbon::control::render_command` shows a command's tooltip with
// `on_hover_text` when the control is enabled and `on_disabled_hover_text`
// when it is not — the **same string**. These five are enabled only while a
// text range is swept (`selection.text`), which is *not* the state an operator
// is in when they go looking for them: they have clicked a piece of text with
// the Select tool, the Format tab has appeared, and the Font controls are
// greyed.
//
// So each tooltip says what the control does **and how to give it something to
// act on**. That second clause is not padding — it is the answer to O37's own
// admission that *"you must press T first and nothing on screen says so"*, and
// a greyed control an operator can hover is the one surface in this
// application that can say it at the moment the question is asked.
//
// It is a **statement**, not a tip. `crate::text::tool`'s rule 2 —
// *"every sentence states a fact about the program, never a tip"* — is why
// these read "Sweeping text with the Text tool chooses what this applies to"
// rather than "Try sweeping some text!".
// ---------------------------------------------------------------------------

/// `format.font`
#[must_use]
pub const fn format_font() -> CommandText {
    CommandText::new(
        "Font",
        "Set the selected text in another of the fonts this page already carries. Sweeping \
         text with the Text tool (T) chooses what it applies to.",
    )
}

/// `format.font_size`
#[must_use]
pub const fn format_font_size() -> CommandText {
    CommandText::new(
        "Size",
        "Set the size of the selected text, in points. Sweeping text with the Text tool (T) \
         chooses what it applies to.",
    )
}

/// `format.bold`
#[must_use]
pub const fn format_bold() -> CommandText {
    CommandText::new(
        "Bold",
        "Set the selected text in bold, or take bold off text that has it — the page's \
         real bold face where it has one, and thickened letters with a note in the status \
         bar where it does not. While typing, it applies to the selected characters, the \
         word at the caret, or what you type next. Ctrl+B.",
    )
}

/// `format.italic`
#[must_use]
pub const fn format_italic() -> CommandText {
    CommandText::new(
        "Italic",
        "Slant the selected text, or take italic off text that has it — the page's real \
         italic face where it has one, and slanted letters with a note in the status bar \
         where it does not. While typing, it applies to the selected characters, the word \
         at the caret, or what you type next. Ctrl+I.",
    )
}

/// `format.underline`
#[must_use]
pub const fn format_underline() -> CommandText {
    CommandText::new(
        "Underline",
        "Draw a line under the selected characters, in the text's own colour. The line is \
         page content of its own: it does not move if the text is later moved or re-wrapped. \
         While typing, it applies to the selected characters, the word at the caret, or what \
         you type next. Ctrl+U.",
    )
}

/// `format.strikethrough`
#[must_use]
pub const fn format_strikethrough() -> CommandText {
    CommandText::new(
        "Strikethrough",
        "Draw a line through the selected characters, in the text's own colour. The line is \
         page content of its own: it does not move if the text is later moved or re-wrapped. \
         While typing, it applies to the selected characters, the word at the caret, or what \
         you type next.",
    )
}

/// `format.align_left`
#[must_use]
pub const fn format_align_left() -> CommandText {
    CommandText::new(
        "Align Left",
        "Line the paragraph up on its left edge. Applies to the paragraph the caret is in, or \
         to every paragraph the selection touches.",
    )
}

/// `format.align_centre`
#[must_use]
pub const fn format_align_centre() -> CommandText {
    CommandText::new(
        "Centre",
        "Centre each line of the paragraph in its box. Applies to the paragraph the caret is \
         in, or to every paragraph the selection touches.",
    )
}

/// `format.align_right`
#[must_use]
pub const fn format_align_right() -> CommandText {
    CommandText::new(
        "Align Right",
        "Line the paragraph up on its right edge. Applies to the paragraph the caret is in, or \
         to every paragraph the selection touches.",
    )
}

/// `format.align_justify`
#[must_use]
pub const fn format_align_justify() -> CommandText {
    CommandText::new(
        "Justify",
        "Spread each line but the last to both edges of the paragraph's box. Applies to the \
         paragraph the caret is in, or to every paragraph the selection touches.",
    )
}

/// `format.font_colour`
#[must_use]
pub const fn format_font_colour() -> CommandText {
    CommandText::new(
        "Colour",
        "Set the colour of the selected text. Text painted in CMYK or a spot colour is left \
         alone, so a drawing's ink is not converted to screen colour behind your back. \
         Sweeping text with the Text tool (T) chooses what it applies to.",
    )
}
