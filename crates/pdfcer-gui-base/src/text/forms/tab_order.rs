//! # `text::forms::tab_order` — every word the Tab-order section shows
//!
//!
//! It is also the block that grew, which is why it was the one to move. When
//! `EditSession::adopt_widget` shipped, the sentence about widgets no field
//! claims stopped being a report and became the heading over a remedy — see
//! [`tab_order_unclaimed`], [`tab_order_register`] and
//! `pdfcer_gui::panels::forms::tab_order::register`.
//!
//! Re-exported wholesale by [`super`], so every call site still spells these
//! `crate::text::forms::tab_order_*`. The split is a change to where the words
//! live and to nothing else; a rename on top of it would have made the diff
//! impossible to read as the mechanical move it is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/forms/tab_order.md`.

// ---------------------------------------------------------------------------
// Tab order — the read-only per-page widget sequence
//
// Every sentence here is a statement about the FILE. The argument behind each
// lives in `tabordermodel`'s header rather than being
// repeated per function: §1 (why `/Annots` order, and why paint order is worth
// saying), §4 (the primary-source reading of `/Tabs` — ISO 32000-2 Table 31 and
// §12.5.1, the two PDF 2.0 values, and the finding that `/Tabs` is NOT
// inheritable), §5 (the four things counted rather than listed).
//
// The rule that binds hardest: a list that silently showed the wrong sequence
// would be worse than no list. Three sentences below exist only to say "this is
// not the tab order", and each names where the real order comes from.
// ---------------------------------------------------------------------------

/// Heading for the tab-order section of the Forms panel.
#[must_use]
pub const fn tab_order_heading() -> &'static str {
    "Tab order"
}

/// The standing explanation, shown whenever the section is open.
#[must_use]
pub const fn tab_order_explainer() -> &'static str {
    "Each page below lists its form fields in the order the file lists the annotations on that \
     page. That is also the order they are painted, so a field further down a page's list is \
     drawn over one further up. Drag a row to move it; a line shows where it will land."
}

/// The count line at the top of the section.
#[must_use]
pub fn tab_order_count(pages: usize, widgets: usize) -> String {
    format!("{widgets} widget(s) listed across {pages} page(s).")
}

/// Shown when no page in the document lists a form-field widget.
#[must_use]
pub const fn tab_order_empty() -> &'static str {
    "No form-field widget is listed on any page of this document."
}

/// Disclosure for fields that have no widget anywhere.
#[must_use]
pub fn tab_order_fields_without_widgets(count: usize) -> String {
    format!(
        "{count} field(s) in this form have no widget on any page. Tab order belongs to a page, \
         so a field with nothing on a page has no position in one and is not listed here."
    )
}

/// One page's heading.
#[must_use]
pub fn tab_order_page_heading(page_number: usize, widgets: usize) -> String {
    format!("Page {page_number} — {widgets} field widget(s)")
}

/// Shown under a page heading when the page lists no form-field widget.
///
/// The page is still shown: its `/Tabs` state is a fact about the document, and
/// a gap in the page numbering would read as a bug.
#[must_use]
pub const fn tab_order_page_no_widgets() -> &'static str {
    "No form-field widget on this page."
}

/// The per-page navigation button.
#[must_use]
pub const fn tab_order_goto() -> &'static str {
    "Go to"
}

/// Its tooltip.
#[must_use]
pub fn tab_order_goto_tooltip(page_number: usize) -> String {
    format!("Show page {page_number} in the document view.")
}

/// One row: its position in the page's sequence, and what to call it.
#[must_use]
pub fn tab_order_row(position: usize, label: &str) -> String {
    format!("{position}. {label}")
}

/// The second line of a row: which page, and which of the field's widgets.
#[must_use]
pub fn tab_order_row_where(page_number: usize, widget: usize, widgets: usize) -> String {
    format!("page {page_number} · widget {widget} of {widgets}")
}

// --- What the file's `/Tabs` entry says, one sentence per state -------------

/// The page carries no `/Tabs`, and neither does any ancestor.
#[must_use]
pub const fn tab_order_no_tabs_entry() -> &'static str {
    "This page has no /Tabs entry, so the file does not say which tab order to use. In practice \
     viewers follow the order the annotations are listed in, which is the order shown here."
}

/// `/Tabs /R` — row order, derived from where the fields sit.
#[must_use]
pub const fn tab_order_tabs_row() -> &'static str {
    "⚠ This page asks for row order (/Tabs /R): a viewer visits the fields in rows across the \
     page. That order is worked out from where the fields sit rather than stored in the file, so \
     the sequence shown here is the order the annotations are listed in and is NOT the tab order."
}

/// `/Tabs /C` — column order, derived from where the fields sit.
#[must_use]
pub const fn tab_order_tabs_column() -> &'static str {
    "⚠ This page asks for column order (/Tabs /C): a viewer visits the fields in columns down the \
     page. That order is worked out from where the fields sit rather than stored in the file, so \
     the sequence shown here is the order the annotations are listed in and is NOT the tab order."
}

/// `/Tabs /S` — structure order, derived from the tag tree.
#[must_use]
pub const fn tab_order_tabs_structure() -> &'static str {
    "⚠ This page asks for structure order (/Tabs /S): a viewer visits the fields in the order \
     they appear in the document's tag tree. That order is worked out from the tags rather than \
     stored in the file, so the sequence shown here is the order the annotations are listed in \
     and is NOT the tab order."
}

/// `/Tabs /A` — annotation-array order (PDF 2.0). This list *is* the order.
#[must_use]
pub const fn tab_order_tabs_annots_array() -> &'static str {
    "This page asks for annotation-array order (/Tabs /A), which the PDF standard defines as the \
     order the annotations are listed in — the order shown here."
}

/// `/Tabs /W` — widget order (PDF 2.0). This list *is* the order, for the
/// fields.
#[must_use]
pub const fn tab_order_tabs_widgets() -> &'static str {
    "This page asks for widget order (/Tabs /W): the form fields first, in the order the \
     annotations are listed in, then everything else. The sequence shown here is that order."
}

/// A `/Tabs` name this build does not recognise, carried verbatim.
///
/// Says "may not be", not "is not": claiming the sequence wrong would be as
/// much an invention as claiming it right, about a name nobody has defined.
#[must_use]
pub fn tab_order_tabs_unrecognised(name: &str) -> String {
    format!(
        "⚠ This page names a tab order pdfcer does not recognise (/Tabs /{name}). The sequence \
         shown here is the order the annotations are listed in, which may not be it."
    )
}

/// A `/Tabs` on an ancestor page-tree node, which does **not** reach the page.
#[must_use]
pub fn tab_order_tabs_on_ancestor(name: &str) -> String {
    format!(
        "This page has no /Tabs entry of its own; a page-tree node above it carries /Tabs \
         /{name}. The PDF standard lists only Resources, MediaBox, CropBox and Rotate as \
         inheritable page attributes, so pdfcer reads this page as naming no tab order — but a \
         viewer that inherited the entry would use that one."
    )
}

// --- What could not be listed, counted per page ----------------------------

/// Widgets on this page that no listed field claims — the heading over the
/// rows that can now do something about it.
#[must_use]
pub fn tab_order_unclaimed(count: usize) -> String {
    if count == 1 {
        "\u{26a0} 1 box on this page is drawn as a form control that no field claims. It cannot be \
         filled until it is registered."
            .to_owned()
    } else {
        format!(
            "\u{26a0} {count} boxes on this page are drawn as form controls that no field claims. \
             They \
             cannot be filled until they are registered."
        )
    }
}

/// One unclaimed widget's row: where it sits in the tab sequence.
#[must_use]
pub fn tab_order_unclaimed_row(page_number: usize, position: usize) -> String {
    format!("Page {page_number}, box {position} in the tab order")
}

/// The hint over the name box beside an unclaimed widget.
#[must_use]
pub const fn tab_order_register_name_hint() -> &'static str {
    "Name — leave blank to keep the name the box already carries"
}

/// The button that registers one unclaimed widget.
#[must_use]
pub const fn tab_order_register() -> &'static str {
    "Register"
}

/// Widgets written into `/Annots` as values rather than as references.
#[must_use]
pub fn tab_order_anonymous(count: usize) -> String {
    format!(
        "⚠ {count} widget(s) on this page are written into the page as values rather than as \
         references, which the PDF standard does not allow. They have no identity that could be \
         matched to a field, so they are counted here rather than listed."
    )
}

/// Non-widget annotations on the page, which are in the tab sequence too.
#[must_use]
pub fn tab_order_other_annots(count: usize) -> String {
    format!(
        "{count} other annotation(s) on this page — links, notes, markup — are visited when \
         tabbing as well. This list is form fields only, so the numbers above are their order \
         among the fields rather than among everything on the page."
    )
}

/// The Register button when pdfcer knows what name it will use.
#[must_use]
pub fn tab_order_register_as(name: &str) -> String {
    format!("Register as \u{201c}{name}\u{201d}")
}

/// Hover on a Register control pdfcer already knows would refuse, because the
/// widget carries no name and none has been typed.
#[must_use]
pub const fn tab_order_register_needs_a_name() -> &'static str {
    "This box carries no name of its own. Type one to make it a new, empty field — its original \
     name and type are not in this file."
}

/// Hover on a Register control whose refusal is that the name is taken.
#[must_use]
pub const fn tab_order_register_name_taken() -> &'static str {
    "Another field already uses that name. Two fields with one name are one field with two \
     boxes, so pdfcer needs a different one."
}

/// Hover on a Register control refused because the typed name is a PATH.
#[must_use]
pub const fn tab_order_register_name_is_a_path() -> &'static str {
    "A dot separates a field from its parent, so this name would be read as a path to somewhere else. Type a name with no dots in it."
}

/// Hover on a Register control refused because the typed name has a bare dot.
#[must_use]
pub const fn tab_order_register_name_has_a_bare_dot() -> &'static str {
    "That name has a dot with nothing beside it. Remove the dot, or put a name on both sides of it."
}

/// Hover on a Register control pdfcer cannot pre-judge.
#[must_use]
pub const fn tab_order_register_unavailable() -> &'static str {
    "pdfcer cannot register this box, and the reason is not one this panel expects. The details \
     are in the diagnostic trace."
}

/// Hover on a Register control that will succeed and produce a typeless field.
#[must_use]
pub const fn tab_order_register_no_type() -> &'static str {
    "This will register, and the field will still have no type — so no viewer will know how to \
     fill it. The type lived in the field definition this box lost."
}

/// One unclaimed widget's line when pdfcer already knows the name it would
/// register under.
#[must_use]
pub fn tab_order_unclaimed_row_named(page_number: usize, position: usize, name: &str) -> String {
    format!("Page {page_number}, box {position} — will register as \u{201c}{name}\u{201d}")
}

/// **A reorder moved things that are not form fields** — `OPERATOR_REQUESTS.md`
/// O99.
#[must_use]
pub fn reorder_moved_non_widgets(count: usize) -> String {
    format!(
        "{count} other annotation(s) on this page — links, comments or markup — moved as \
         well. The order you set is also the order things are drawn in, so where two of them \
         overlap, which one is on top may have changed."
    )
}

/// **Some entries could not be moved** — O99.
#[must_use]
pub fn reorder_pinned(count: usize) -> String {
    format!(
        "{count} item(s) on this page could not be moved and stayed where they were — they \
         are written into the page in a form that has no name to refer to. Everything else \
         moved around them."
    )
}

/// **The page's annotation list was shared and had to be copied** — O99.
#[must_use]
pub const fn reorder_copied_shared_array() -> &'static str {
    "This page shared its annotation list with another page, so the list was copied before it \
     was reordered. The other page is unchanged."
}
