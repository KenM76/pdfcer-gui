//! # `text::xfapreview` — the words for a dynamic XFA form
//!
//! The open-time sentence a dynamic form earns, and the copy of the form
//! preview window `pdfcer_gui::dialogs::xfa_preview` draws for
//! `view.form_preview`.

use pdfcer_core::xfa::{LayoutWarning, MergeWarningKind, PaintNoteKind};

/// The status-line sentence when a dynamic XFA form opens: the pages shown
/// are the file's stand-in, and the computed ones are open beside them.
#[must_use]
pub fn opened_dynamic() -> &'static str {
    "This is a dynamic form: the page shown is the file's stand-in for programs \
     without a form engine. The form's own pages are open in Form preview; Use these \
     pages there makes them the document's pages."
}

/// The window's title.
#[must_use]
pub fn title() -> &'static str {
    "Form preview"
}

/// The standing caveat above the pages.
#[must_use]
pub fn caveat() -> &'static str {
    "These are the form's own pages, drawn from its layout with the values its data \
     holds. Click a field to fill it in: Enter writes the value into the form, and Ctrl+Z takes it back."
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
    "Nothing."
}

/// One kind of drawing note, with how many items it touched.
#[must_use]
pub fn paint_note(kind: PaintNoteKind, count: usize) -> String {
    let what = match kind {
        PaintNoteKind::Content => "rich text, captions or non-text content not drawn",
        PaintNoteKind::Stroke => "borders of an unknown style drawn solid",
        PaintNoteKind::FillType => "fills not drawn",
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
        PaintNoteKind::StrokeApproximated => "3-D borders drawn as plain lines",
        PaintNoteKind::Shape => "line slopes, arc angles or line ends not understood, drawn plain",
        PaintNoteKind::Rotate => "turns other than quarter turns, or on sections, not applied",
        PaintNoteKind::Script => "page numbers shown from a script that was not run",
        PaintNoteKind::Choice => "list values shown as saved, or list items not filled in",
        PaintNoteKind::Picture => "values shown partly unformatted, or with English names",
        PaintNoteKind::Barcode => "barcodes shown as their text",
        _ => "drawn differently from the form",
    };
    format!("{count} × {what}")
}

/// One layout warning, worded.
#[must_use]
pub fn layout_warning(warning: &LayoutWarning) -> &'static str {
    match warning {
        LayoutWarning::InappropriateLayout { .. } => {
            "A table row outside a table is placed by position."
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

/// How many values the form's data filled in, above the notes, with the
/// fields left empty and the data nothing took when there are any.
#[must_use]
pub fn values_merged(values: usize, has_data: bool, unfilled: usize, unbound: usize) -> String {
    let filled = filled_count(values, has_data);
    match (has_data, unfilled, unbound) {
        (false, _, _) | (true, 0, 0) => filled,
        (true, u, d) => {
            format!("{filled} {u} field(s) took no value; {d} data value(s) fill nothing.")
        }
    }
}

fn filled_count(values: usize, has_data: bool) -> String {
    match (has_data, values) {
        (false, _) => {
            "The form carries no filled-in data, so its fields are drawn empty.".to_owned()
        }
        (true, 0) => {
            "The form's data fills none of the fields this preview can match yet.".to_owned()
        }
        (true, 1) => "1 value filled in from the form's data.".to_owned(),
        (true, n) => format!("{n} values filled in from the form's data."),
    }
}

/// One kind of data reference that was not followed, with how many fields
/// it left at their default value.
#[must_use]
pub fn merge_warning(kind: MergeWarningKind, count: usize) -> String {
    let unfollowed = match kind {
        MergeWarningKind::MixedGlobal => {
            return format!(
                "{count} × values shared between global and ordinary copies of a field, shared anyway"
            );
        }
        MergeWarningKind::ExcessData => {
            return format!(
                "{count} × data groups left over once a repeating section reached its most copies, not shown"
            );
        }
        MergeWarningKind::InstanceLimit => {
            return format!("{count} × repeating sections stopped at the preview's copy limit");
        }
        MergeWarningKind::Unresolved => "fields whose data reference finds no data",
        MergeWarningKind::WrongType => "fields whose data reference finds the wrong kind of data",
        MergeWarningKind::Unsupported => {
            "fields whose data reference uses a form this preview cannot follow yet"
        }
        MergeWarningKind::TooLong => "fields whose data reference is too long to follow",
        _ => "fields whose data reference could not be followed",
    };
    format!("{count} × {unfollowed}, drawn with their default value")
}

/// The heading over the choices XFA leaves open.
#[must_use]
pub fn choices_heading() -> &'static str {
    "Reading choices"
}

/// What the choices are, under their heading.
#[must_use]
pub fn choices_caveat() -> &'static str {
    "Where the XFA standard leaves a choice open. The first answer of each follows \
     the standard's own text; changing one redraws the form."
}

/// The five choices the XFA standard leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceKey {
    /// `MergeOptions::scope_search`.
    ScopeSearch,
    /// `MergeOptions::bare_ref`.
    BareRef,
    /// `MergeOptions::data_attributes`.
    DataAttributes,
    /// `PaintOptions::three_d_stroke`.
    ThreeDStroke,
    /// `PaintOptions::check_caption`.
    CheckCaption,
}

/// One open choice: its label and its two answers, the default first.
#[must_use]
pub fn choice(key: ChoiceKey) -> (&'static str, [&'static str; 2]) {
    match key {
        ChoiceKey::ScopeSearch => (
            "A short data name is looked for",
            [
                "In its group, then each enclosing one",
                "In its own group only",
            ],
        ),
        ChoiceKey::BareRef => (
            "A binding with a reference but no match",
            ["Is ignored", "Is followed as a data reference"],
        ),
        ChoiceKey::DataAttributes => (
            "Attributes in the form's data",
            ["Are values", "Are not data"],
        ),
        ChoiceKey::ThreeDStroke => ("3-D borders", ["Bevelled", "Plain lines"]),
        ChoiceKey::CheckCaption => (
            "A check box caption with no set width",
            ["Fits its text", "Takes everything but the box"],
        ),
    }
}

/// The Close button.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

/// The button that makes the previewed pages the document's own.
#[must_use]
pub fn use_pages() -> &'static str {
    "Use these pages"
}

/// What pressing [`use_pages`] does, on hover.
#[must_use]
pub fn use_pages_hover() -> &'static str {
    "Replace the file's stand-in pages with these pages, drawn with the reading choices above. \
     The values become part of the drawing and can no longer be filled in. Ctrl+Z undoes it; \
     Save writes it."
}

/// The status line after the pages were written.
#[must_use]
pub fn pages_used(written: usize, replaced: usize, fields_dropped: usize) -> String {
    let fields = if fields_dropped == 0 {
        String::new()
    } else {
        format!(" {fields_dropped} field(s) of the stand-in page went with it.")
    };
    format!(
        "Replaced the file's {replaced} stand-in page(s) with the form's {written} page(s). Every \
         page is pdfcer's own layout of the form, and its values are drawn, not fillable.{fields} \
         Press Ctrl+Z to undo."
    )
}

/// The typefaces set in a standard face instead, as `(typeface, face)`.
#[must_use]
pub fn pages_fallbacks(fallbacks: &[(String, String)]) -> String {
    let list: Vec<String> = fallbacks
        .iter()
        .map(|(t, f)| format!("{t} as {f}"))
        .collect();
    format!(
        "No matching face among the fonts pdfcer draws with, so drawn in a standard face: {}.",
        list.join(", ")
    )
}

/// The faces named but not embedded, so another viewer substitutes.
#[must_use]
pub fn pages_not_embedded(faces: &[String]) -> String {
    format!(
        "Not embedded, so another viewer may draw them in a different face: {}.",
        faces.join(", ")
    )
}

/// The form's usage-rights signature went with the form.
#[must_use]
pub fn pages_usage_rights_removed() -> &'static str {
    "The form's usage-rights signature was removed with the form it signed."
}

/// What the written pages draw differently, one kind per entry.
#[must_use]
pub fn pages_notes(notes: &[String]) -> String {
    format!("What the pages leave out: {}.", notes.join("; "))
}

/// Why Use these pages wrote nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormPagesRefusal {
    /// The document is not a dynamic form.
    NotDynamic,
    /// The file carries this many signatures, which new pages would orphan.
    Signed(usize),
    /// A certification forbids changing the pages.
    Certified,
    /// The form could not be read or laid out.
    NotLaidOut,
}

impl FormPagesRefusal {
    /// The sentence for this refusal.
    #[must_use]
    pub fn line(self) -> std::borrow::Cow<'static, str> {
        use std::borrow::Cow;
        match self {
            Self::NotDynamic => {
                Cow::Borrowed("This form's pages are already the document's: nothing was replaced.")
            }
            Self::Signed(n) => Cow::Owned(format!(
                "This form carries {n} signature(s), which replacing its pages would break: \
                 nothing was replaced."
            )),
            Self::Certified => Cow::Borrowed(
                "This form's certification forbids changing its pages: nothing was replaced.",
            ),
            Self::NotLaidOut => Cow::Borrowed(
                "The form could not be laid out, so its pages could not be used: nothing was \
                 replaced.",
            ),
        }
    }
}

/// A field's hover in the preview: what a click does.
#[must_use]
pub fn fill_hover(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("{n}: click to fill in"),
        None => "Click to fill in".to_owned(),
    }
}

/// The status line after a value was written into the form's data.
#[must_use]
pub fn filled(name: Option<&str>, shared: usize, created: &[String]) -> String {
    let field = name.map_or_else(|| "the field".to_owned(), |n| n.to_owned());
    let shared = match shared {
        0 => String::new(),
        n => format!(" {n} other field(s) share its data and show it too."),
    };
    let created = if created.is_empty() {
        String::new()
    } else {
        format!(
            " The form's data had no place for it, so pdfcer added: {}.",
            created.join(", ")
        )
    };
    format!("Filled in {field}.{shared}{created} Press Ctrl+Z to undo.")
}
