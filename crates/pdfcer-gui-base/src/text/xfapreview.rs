//! # `text::xfapreview` — the words for a dynamic XFA form
//!
//! The open-time sentence a dynamic form earns, and the copy of the form
//! preview window `pdfcer_gui::dialogs::xfa_preview` draws for
//! `view.form_preview`.

use pdfcer_core::xfa::{LayoutWarning, PaintNoteKind};

/// The status-line sentence when a dynamic XFA form opens: the pages shown
/// are the file's stand-in, and where the computed ones are.
#[must_use]
pub fn opened_dynamic() -> &'static str {
    "This is a dynamic form: the page shown is the file's stand-in for programs \
     without a form engine. View > Form preview draws the form's own pages."
}

/// The window's title.
#[must_use]
pub fn title() -> &'static str {
    "Form preview"
}

/// The standing caveat above the pages.
#[must_use]
pub fn caveat() -> &'static str {
    "These are the form's own pages, drawn from its layout. Filled-in values are \
     not shown yet, and nothing here is saved to the file."
}

/// Shown while the layout is computed.
#[must_use]
pub fn working() -> &'static str {
    "Laying out the form…"
}

/// The form could not be laid out or drawn.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("The form could not be drawn: {why}")
}

/// The file says it is a dynamic form but carries no template.
#[must_use]
pub fn no_template() -> &'static str {
    "The file says it is a dynamic form but carries no form template to draw."
}

/// The heading over one computed page.
#[must_use]
pub fn page_heading(number: usize, of: usize) -> String {
    format!("Page {number} of {of}")
}

/// The heading over what was left out or approximated.
#[must_use]
pub fn notes_heading() -> &'static str {
    "What this preview leaves out"
}

/// Nothing was left out.
#[must_use]
pub fn notes_clean() -> &'static str {
    "Nothing beyond the filled-in values."
}

/// One kind of drawing note, with how many items it touched.
#[must_use]
pub fn paint_note(kind: PaintNoteKind, count: usize) -> String {
    let what = match kind {
        PaintNoteKind::Content => "rich text, captions or non-text content not drawn",
        PaintNoteKind::Stroke => "dashed or 3-D borders not drawn",
        PaintNoteKind::FillType => "gradient or pattern fills not drawn",
        PaintNoteKind::Corner => "rounded corners drawn square",
        PaintNoteKind::OpenBreak => "borders split across pages drawn closed",
        PaintNoteKind::Color => "unreadable colours replaced by the default",
        PaintNoteKind::Font => "fonts drawn in the nearest standard face",
        PaintNoteKind::Text => {
            "text drawn approximately (unsupported characters, \
                                 alignment, or lines cut off)"
        }
        PaintNoteKind::Image => "pictures not found in the file or not readable",
        PaintNoteKind::ImageTransparency => "picture transparency flattened onto white",
        _ => "drawn differently from the form",
    };
    format!("{count} × {what}")
}

/// One layout warning, worded.
#[must_use]
pub fn layout_warning(warning: &LayoutWarning) -> &'static str {
    match warning {
        LayoutWarning::FlowedLayoutPending { .. } => {
            "A table or row is placed by position; table layout is not supported yet."
        }
        LayoutWarning::QualifiedPagination => {
            "Odd/even or first/last page rules are laid out in plain order."
        }
        LayoutWarning::UnboundedBoilerplate { .. } => "A repeating page background is drawn once.",
        LayoutWarning::InvalidBreakTarget { .. } => {
            "A page break names no single page or area; it went to the next one."
        }
        LayoutWarning::InvalidOverflowTarget { .. }
        | LayoutWarning::InvalidBookendTarget { .. } => {
            "A header, footer or overflow target names nothing usable and was skipped."
        }
        LayoutWarning::MinGreaterThanMax { .. } => {
            "A size whose minimum exceeded its maximum had the two swapped."
        }
        LayoutWarning::NegativeCoordinate { .. } => "An item sat at a negative position.",
        LayoutWarning::UnresolvedMeasurement { .. } => {
            "A measurement in an unknown unit was ignored."
        }
        LayoutWarning::ContentDiscarded { .. } => {
            "The form ran out of pages; content after that point is not shown."
        }
        LayoutWarning::ContentOverflow { .. } => {
            "An item too tall for its page runs past the bottom edge."
        }
        LayoutWarning::PageAreaKeep { .. } => "A keep-together rule naming a page was ignored.",
        LayoutWarning::PageLimit => "The preview stops at its page limit.",
        LayoutWarning::ItemLimit => "The preview stops at its item limit.",
        _ => "The layout differs from the form in a way this preview cannot name.",
    }
}

/// The Close button.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}
