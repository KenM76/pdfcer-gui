//! # `text::forms::tab_order` — every word the Tab-order section shows
//!
//! Split out of [`super`] when R2's 1,500-line limit was reached, and the seam
//! was chosen rather than found: this is the one block in the forms catalog
//! whose sentences are about **the sequence a form is filled in** rather than
//! about a field's value, its refusals or its formatting.
//!
//! It is also the block that grew, which is why it was the one to move. When
//! `EditSession::adopt_widget` shipped, the sentence about widgets no field
//! claims stopped being a report and became the heading over a remedy — see
//! [`tab_order_unclaimed`], [`tab_order_register`] and
//! `crate::panels::forms::tab_order::register`.
//!
//! Re-exported wholesale by [`super`], so every call site still spells these
//! `crate::text::forms::tab_order_*`. The split is a change to where the words
//! live and to nothing else; a rename on top of it would have made the diff
//! impossible to read as the mechanical move it is.

// ---------------------------------------------------------------------------
// Tab order — the read-only per-page widget sequence
//
// Every sentence here is a statement about the FILE. The argument behind each
// lives in `crate::panels::forms::tab_order::model`'s header rather than being
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
///
/// Three load-bearing clauses: what the order *is*; that it is also **paint**
/// order, the fact that would make a future reorder consequential and that
/// nothing else on screen could tell you; and that this view changes nothing —
/// said in prose, because the alternative is a disabled control (`RIBBON_IA.md`
/// P3 forbids one).
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
///
/// Distinct from the panel's own empty states: this is reachable on a document
/// that *has* an `/AcroForm` full of fields, none of whose widgets any page
/// lists. Silence would read as a broken section rather than a fact.
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
///
/// `label` is the field's `/TU` when non-blank and its fully-qualified name
/// otherwise — the fill rows' preference, so the operator reads the string an
/// assistive technology speaks. The raw name is in [`form_field_row_tooltip`].
#[must_use]
pub fn tab_order_row(position: usize, label: &str) -> String {
    format!("{position}. {label}")
}

/// The second line of a row: which page, and which of the field's widgets.
///
/// Always drawn, including for a single-widget field. A field with widgets on
/// several pages **appears more than once** — correct rather than a duplicate,
/// because tab order is per page and a field is document-level — and a row that
/// named the widget only when there happened to be more than one would leave
/// the operator working out which case they were looking at.
#[must_use]
pub fn tab_order_row_where(page_number: usize, widget: usize, widgets: usize) -> String {
    format!("page {page_number} · widget {widget} of {widgets}")
}

// --- What the file's `/Tabs` entry says, one sentence per state -------------

/// The page carries no `/Tabs`, and neither does any ancestor.
///
/// ★ **Reported as absent, and given no mode name** — not "manual", not
/// "unspecified". `D:\Dev\pdfcer`'s roadmap records what Acrobat's "Unspecified"
/// tab-order state mechanically denotes as **unsourced after two attempts**, so
/// a label here would assert what nobody has been able to support. What is said
/// instead is what the file says, plus the operationally useful half.
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
///
/// ★ The correction argued in `crate::panels::forms::tab_order::model`'s §4:
/// ISO 32000-2 Table 31 marks `Rotate` "(Optional; inheritable)" and `Tabs`
/// merely "(Optional; PDF 1.5)", and the table's preamble makes every unmarked
/// attribute non-inheritable.
///
/// Both halves have to be said. Treating the ancestor's value as the page's own
/// asserts an inheritance the standard denies; saying "no /Tabs" over a file
/// that plainly has one two levels up hides a fact that changes what another
/// viewer might do.
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
///
/// # ★ This sentence used to end in a guess, and the guess has been replaced
/// by a route
///
/// The old wording finished *"if the form declares entries pdfcer could not
/// read, these may be theirs"* — a speculation offered because there was
/// nothing better to offer. Nothing could be done with an unclaimed widget, so
/// the only honest thing left was to speculate about where it came from.
///
/// `EditSession::adopt_widget` shipped 2026-08-19, so an unclaimed widget is
/// now a **chore with a button** rather than a curiosity with a theory. The
/// sentence says what the boxes are; the rows below it do the rest.
///
/// The inline-field-roots note is still one line away in
/// [`forms_inline_field_roots_note`] and is still not re-counted here — two
/// numbers about two things, related out loud rather than added together.
///
/// # Why it keeps its warning glyph now that it has a remedy
///
/// Because the remedy is a **chore the operator has not done yet**, not a
/// reassurance. Until they press Register, the boxes on the page in front of
/// them still cannot be filled, and that is a warning whether or not a button
/// sits under it. `RIBBON_IA.md` R84 also requires the glyph independently: the
/// sentence is drawn in `warn_fg_color`, and a warning carried by colour alone
/// is invisible to a reader who cannot distinguish it.
///
/// # Why it says "cannot be filled" rather than "are broken"
///
/// Because that is the operator-visible fact and it is the one that surprises.
/// The box **draws**. It has a border, it has a background, it looks exactly
/// like the field beside it. What it does not have is a name any filling verb
/// can address, so clicking it and typing produces nothing and no message.
/// This project's own recurring failure — a visible control that is silently
/// inert — arriving through a document rather than through a ribbon.
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
///
/// The position is what an operator uses to **find** it — press Tab that many
/// times and watch the focus ring land — which is the only handle they have,
/// because the thing has no name by definition. That is also why the row is
/// worth drawing at all rather than leaving the heading's count to stand: a
/// count cannot be pressed, and it cannot be pointed at either.
#[must_use]
pub fn tab_order_unclaimed_row(page_number: usize, position: usize) -> String {
    format!("Page {page_number}, box {position} in the tab order")
}

/// The hint over the name box beside an unclaimed widget.
///
/// ★ Says what an empty box means, because empty is the common and correct
/// answer and a blank field with no hint reads as "required".
///
/// Most unclaimed widgets are **merged field-widgets** (§12.7.3.1): one
/// dictionary serving as both, carrying its own `/T`, `/FT` and `/V`. The
/// engine measured a real form and found 11 of 13 in that shape. For those,
/// registering with no name recovers the field exactly as it was — the name is
/// already in the file, and typing one would *override* it rather than supply
/// something missing.
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
///
/// The one that stops the numbering reading as wrong. §12.5.1's tab order is
/// over **annotations**, not form fields: a link occupies a position in the
/// sequence, so a list of widgets is the fields *in* it, not the sequence.
#[must_use]
pub fn tab_order_other_annots(count: usize) -> String {
    format!(
        "{count} other annotation(s) on this page — links, notes, markup — are visited when \
         tabbing as well. This list is form fields only, so the numbers above are their order \
         among the fields rather than among everything on the page."
    )
}

/// The Register button when pdfcer knows what name it will use.
///
/// # ★ The name is in the FILE and not on screen, which is the whole point
///
/// A merged field-widget (SS12.7.3.1) carries its own `/T`. Registering it with
/// a blank name box recovers that name — a string the operator has never seen,
/// because nothing in the panel could show it: the widget belongs to no field,
/// so no field row names it.
///
/// The engine put it exactly right when the preview shipped: *"Register as
/// `Address`" is a decision; "Register" is a guess.*
#[must_use]
pub fn tab_order_register_as(name: &str) -> String {
    format!("Register as \u{201c}{name}\u{201d}")
}

/// Hover on a Register control pdfcer already knows would refuse, because the
/// widget carries no name and none has been typed.
///
/// ★ Says what typing a name will **produce**, not that one is required. The
/// distinction is the whole of it: this box was a bare kid, its name, field
/// type, radio flags and value all lived in a dictionary that is not in this
/// document, and a name typed here **creates a new field**. It does not recover
/// the old one.
#[must_use]
pub const fn tab_order_register_needs_a_name() -> &'static str {
    "This box carries no name of its own. Type one to make it a new, empty field — its original \
     name and type are not in this file."
}

/// Hover on a Register control whose refusal is that the name is taken.
///
/// The short form of [`crate::text::status::adopt_declined_name_taken`], which
/// is the sentence shown after a press. Both exist because they are read at
/// different moments: this one while the operator is still typing, that one
/// after they have committed to a name.
#[must_use]
pub const fn tab_order_register_name_taken() -> &'static str {
    "Another field already uses that name. Two fields with one name are one field with two \
     boxes, so pdfcer needs a different one."
}

/// Hover on a Register control pdfcer cannot pre-judge.
///
/// The catch-all for refusals this surface believes are unreachable. It says
/// the control is unavailable and does not guess why — a wrong reason is worse
/// than none, and by construction reaching here means the listing and the
/// engine disagree about what this widget is, which is a fault to find in the
/// trace rather than a chore to hand to an operator.
#[must_use]
pub const fn tab_order_register_unavailable() -> &'static str {
    "pdfcer cannot register this box, and the reason is not one this panel expects. The details \
     are in the diagnostic trace."
}

/// Hover on a Register control that will succeed and produce a typeless field.
///
/// # ★★ The disclosure that would otherwise arrive too late
///
/// `/FT` is inheritable. A widget that was a field's kid could inherit its
/// type; once it is registered as a **top-level** field there is nothing left
/// above it, so it has no type at all and no viewer knows how to render or fill
/// it.
///
/// The registration still succeeds. So without this the operator presses a
/// button, is told it worked, and has a box that still cannot be filled — which
/// is rule 4's case exactly: an inference they cannot see, owed a sentence
/// precisely because nothing on the page looks wrong.
///
/// Said **before** the press now that `adopt_preview` makes it knowable.
/// Telling somebody afterwards tells them their successful action did not do
/// what they wanted.
#[must_use]
pub const fn tab_order_register_no_type() -> &'static str {
    "This will register, and the field will still have no type — so no viewer will know how to \
     fill it. The type lived in the field definition this box lost."
}

/// One unclaimed widget's line when pdfcer already knows the name it would
/// register under.
///
/// # ★ Why the name is here and not on the button
///
/// A label wraps to the pane width; a button does not. These rows live in a
/// dock panel about 314 pt wide, and a button reading *"Register as
/// CustomerName"* beside a name box is wider than that — it runs off the
/// right-hand edge and takes the next row down with it.
///
/// What the pre-flight was asked for is that the name be **visible before the
/// press**, not that it be printed on the control. On the line immediately
/// above the button it is both visible and wrappable.
#[must_use]
pub fn tab_order_unclaimed_row_named(page_number: usize, position: usize, name: &str) -> String {
    format!("Page {page_number}, box {position} — will register as \u{201c}{name}\u{201d}")
}

/// **A reorder moved things that are not form fields** — `OPERATOR_REQUESTS.md`
/// O99.
///
/// ★★★ The disclosure the operator would never predict. `/Annots` order is
/// **paint order** as well as tab order, so arranging a tab sequence can change
/// which annotation is drawn on top where two overlap. They asked to reorder a
/// list of fields and got a z-order change; the sentence says so in their terms
/// rather than in the file's.
#[must_use]
pub fn reorder_moved_non_widgets(count: usize) -> String {
    format!(
        "{count} other annotation(s) on this page — links, comments or markup — moved as \
         well. The order you set is also the order things are drawn in, so where two of them \
         overlap, which one is on top may have changed."
    )
}

/// **Some entries could not be moved** — O99.
///
/// ★ A list that did not fully take, said rather than discovered. These are
/// entries written into the page as direct dictionaries: they have no object id
/// to be named by, so they stay where they are and the rest flow around them.
/// Rare, and produced by a handful of writers.
#[must_use]
pub fn reorder_pinned(count: usize) -> String {
    format!(
        "{count} item(s) on this page could not be moved and stayed where they were — they \
         are written into the page in a form that has no name to refer to. Everything else \
         moved around them."
    )
}

/// **The page's annotation list was shared and had to be copied** — O99.
///
/// ★ Nothing is wrong and nothing is lost. It is a structural change to the file
/// the operator did not ask for, which is the whole reason it is disclosed: this
/// project's rule is that a side effect they cannot see still owes a sentence
/// off-canvas.
#[must_use]
pub const fn reorder_copied_shared_array() -> &'static str {
    "This page shared its annotation list with another page, so the list was copied before it \
     was reordered. The other page is unchanged."
}
