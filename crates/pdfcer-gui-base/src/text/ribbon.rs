//! # text::ribbon — the ribbon's *structural* strings
//!
//! Tab labels, the one-line question each tab exists to answer, group
//! captions, the three mode labels, and the words inside the band's own
//! non-button controls. Everything a person reads on the
//! ribbon that is **not** a command; command labels and tooltips live in
//! [`crate::text::commands`], which is a much longer file for a reason
//! that is worth stating: there are eight tabs and thirty-seven groups, and
//! there are a hundred and twenty commands. Splitting the catalog along that seam
//! keeps both halves navigable and both files inside the project's
//! 1,500-line ceiling.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/ribbon.md`.

// ---------------------------------------------------------------------------
// TAB LABELS
//
// `RIBBON_IA.md` §4's seven tabs, plus the contextual Format tab of §5.8.
// One rename is carried: `Review` becomes `Markup`, because what lives
// there is markup *authoring* — shapes, notes, stamps — and "Review"
// promises a review *workflow* (compare revisions, resolve comments, track
// changes) that pdfcer does not have and will want the name for when it
// does. `Markup` is also the word this project's audience uses.
// ---------------------------------------------------------------------------

/// The File tab's label.
#[must_use]
pub fn tab_file() -> &'static str {
    "File"
}

/// The View tab's label.
#[must_use]
pub fn tab_view() -> &'static str {
    "View"
}

/// The Pages tab's label.
#[must_use]
pub fn tab_pages() -> &'static str {
    "Pages"
}

/// The Edit tab's label.
#[must_use]
pub fn tab_edit() -> &'static str {
    "Edit"
}

/// The Markup tab's label — the salvage source's `Review`, renamed.
#[must_use]
pub fn tab_markup() -> &'static str {
    "Markup"
}

/// The Measure tab's label.
#[must_use]
pub fn tab_measure() -> &'static str {
    "Measure"
}

/// The Tools tab's label.
#[must_use]
pub fn tab_tools() -> &'static str {
    "Tools"
}

/// The contextual Format tab's label.
#[must_use]
pub fn tab_format() -> &'static str {
    "Format"
}

// ---------------------------------------------------------------------------
// TAB QUESTIONS
//
// Verbatim from `RIBBON_IA.md` §4's table, which is the specification.
// Changing one of these is a change to what the tab is *for*, and should
// be made in that document first.
// ---------------------------------------------------------------------------

/// The File tab's question.
#[must_use]
pub fn question_file() -> &'static str {
    "What do I do with the file as a whole, or with pdfcer itself?"
}

/// The View tab's question.
#[must_use]
pub fn question_view() -> &'static str {
    "What is on my screen, and how is the page laid out?"
}

/// The Pages tab's question.
#[must_use]
pub fn question_pages() -> &'static str {
    "What am I doing to the set of pages?"
}

/// The Edit tab's question.
#[must_use]
pub fn question_edit() -> &'static str {
    "What am I changing about content that is already there?"
}

/// The Markup tab's question.
#[must_use]
pub fn question_markup() -> &'static str {
    "What am I adding for someone else to read?"
}

/// The Measure tab's question.
#[must_use]
pub fn question_measure() -> &'static str {
    "What am I measuring, and in what units?"
}

/// The Tools tab's question.
#[must_use]
pub fn question_tools() -> &'static str {
    "What do I run across files, or configure once?"
}

/// The Format tab's question.
#[must_use]
pub fn question_format() -> &'static str {
    "What am I changing about the thing I have selected?"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — File
// ---------------------------------------------------------------------------

/// File ▸ File.
#[must_use]
pub fn group_file_file() -> &'static str {
    "File"
}

/// File ▸ Save.
#[must_use]
pub fn group_file_save() -> &'static str {
    "Save"
}

/// File ▸ Export.
#[must_use]
pub fn group_file_export() -> &'static str {
    "Export"
}

/// File ▸ Print.
#[must_use]
pub fn group_file_print() -> &'static str {
    "Print"
}

/// File ▸ Document — what is *inside* this file, as opposed to what to do
/// with the file itself.
#[must_use]
pub fn group_file_document() -> &'static str {
    "Document"
}

/// File ▸ Recognise — reading words out of a page image.
#[must_use]
pub fn group_file_recognise() -> &'static str {
    "Recognise"
}

/// File ▸ pdfcer — the application's own settings and help.
#[must_use]
pub fn group_file_pdfcer() -> &'static str {
    "pdfcer"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — View
// ---------------------------------------------------------------------------

/// View ▸ Page display — how many pages, and in what arrangement.
#[must_use]
pub fn group_view_page_display() -> &'static str {
    "Page display"
}

/// View ▸ Render — how the page is turned into pixels.
#[must_use]
pub fn group_view_render() -> &'static str {
    "Render"
}

/// View ▸ Navigate — how a drag on the page behaves.
#[must_use]
pub fn group_view_navigate() -> &'static str {
    "Navigate"
}

/// The left rail's **Select** group caption — `OPERATOR_REQUESTS.md` O123.
#[must_use]
pub fn group_rail_select() -> &'static str {
    "Select"
}

/// The left rail's **Rotate** group caption — `OPERATOR_REQUESTS.md` O126.
///
/// *"also add rotate pages to that area, and those should be available in
/// every mode including read."*
#[must_use]
pub fn group_rail_rotate() -> &'static str {
    "Rotate"
}

/// View ▸ Zoom.
#[must_use]
pub fn group_view_zoom() -> &'static str {
    "Zoom"
}

/// View ▸ Display — what is drawn *over* the page.
#[must_use]
pub fn group_view_display() -> &'static str {
    "Display"
}

/// View ▸ Panels.
#[must_use]
pub fn group_view_panels() -> &'static str {
    "Panels"
}

/// View ▸ Window — the shape of the application, not of the document.
#[must_use]
pub fn group_view_window() -> &'static str {
    "Window"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Pages
// ---------------------------------------------------------------------------

/// Pages ▸ Insert.
#[must_use]
pub fn group_pages_insert() -> &'static str {
    "Insert"
}

/// Pages ▸ Clipboard.
#[must_use]
pub fn group_pages_clipboard() -> &'static str {
    "Clipboard"
}

/// Pages ▸ Organise.
#[must_use]
pub fn group_pages_organise() -> &'static str {
    "Organise"
}

/// Pages ▸ Transform.
#[must_use]
pub fn group_pages_transform() -> &'static str {
    "Transform"
}

/// Caption of the Pages ▸ Stamp group.
#[must_use]
pub fn group_pages_stamp() -> &'static str {
    "Stamp"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Edit
// ---------------------------------------------------------------------------

/// Edit ▸ Content.
#[must_use]
pub fn group_edit_content() -> &'static str {
    "Content"
}

/// Edit ▸ Insert.
#[must_use]
pub fn group_edit_insert() -> &'static str {
    "Insert"
}

/// Edit ▸ Arrange.
#[must_use]
pub fn group_edit_arrange() -> &'static str {
    "Arrange"
}

/// Edit ▸ Clipboard.
#[must_use]
pub fn group_edit_clipboard() -> &'static str {
    "Clipboard"
}

//
// Its group held exactly two commands, `edit.copy_page_text` and
// `edit.copy_document_text`; the operator moved both to File ▸ Export
// (`file.copy_page_text`, `file.copy_document_text`), which left the band
// empty, and an empty band is the placeholder `RIBBON_IA.md` P3 forbids. The
// group went, so its caption goes with it.
//
// Kept as a comment rather than as a dead `pub fn`: a caption nothing draws is
// an operator-visible string that no reviewer can review in place and that the
// string gates cannot judge, and the next author of an object clipboard needs
// the word — which is right here — rather than a function that already
// compiles and quietly encourages reusing a group that was deleted for a
// reason.

/// Edit ▸ Forms.
#[must_use]
pub fn group_edit_forms() -> &'static str {
    "Forms"
}

/// Edit ▸ Protect.
#[must_use]
pub fn group_edit_protect() -> &'static str {
    "Protect"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Markup
// ---------------------------------------------------------------------------

/// Markup ▸ Shapes.
#[must_use]
pub fn group_markup_shapes() -> &'static str {
    "Shapes"
}

/// Markup ▸ Text markup — markup that attaches to words already on the
/// page, as opposed to shapes drawn over it.
#[must_use]
pub fn group_markup_text() -> &'static str {
    "Text markup"
}

/// Markup ▸ Notes.
#[must_use]
pub fn group_markup_notes() -> &'static str {
    "Notes"
}

/// Markup ▸ Style — the style the *next* markup will be placed with.
#[must_use]
pub fn group_markup_style() -> &'static str {
    "Style"
}

/// Markup ▸ Comments.
#[must_use]
pub fn group_markup_comments() -> &'static str {
    "Comments"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Measure
// ---------------------------------------------------------------------------

/// Measure ▸ Dimension.
#[must_use]
pub fn group_measure_dimension() -> &'static str {
    "Dimension"
}

/// Measure ▸ Scale.
#[must_use]
pub fn group_measure_scale() -> &'static str {
    "Scale"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Tools
// ---------------------------------------------------------------------------

/// Tools ▸ Batch — jobs that produce *new* files.
#[must_use]
pub fn group_tools_batch() -> &'static str {
    "Batch"
}

/// Tools ▸ Fonts.
#[must_use]
pub fn group_tools_fonts() -> &'static str {
    "Fonts"
}

/// Tools ▸ Diagnostics.
#[must_use]
pub fn group_tools_diagnostics() -> &'static str {
    "Diagnostics"
}

// ---------------------------------------------------------------------------
// GROUP CAPTIONS — Format (contextual)
// ---------------------------------------------------------------------------

/// Format ▸ Font.
#[must_use]
pub fn group_format_font() -> &'static str {
    "Font"
}

/// Format ▸ Selection.
#[must_use]
pub fn group_format_selection() -> &'static str {
    "Selection"
}

/// Format ▸ Markup — the controls that restyle a mark already on the page.
#[must_use]
pub fn group_format_markup() -> &'static str {
    "Markup"
}

// ---------------------------------------------------------------------------
// FORMAT ▸ MARKUP — the words inside the band's own controls
//
// **Only the strings with no Properties-panel twin live here.** The
// panel's *This mark* section (`panels::properties::markup`) already names the
// width suffix, the opacity suffix, the Clear button and the locked sentence,
// and `app::markupband` reads all four from `crate::text::panels::properties`
// exactly as `app::fontband` reads its own from there.
//
// That is deliberate and it is the same argument `fontband` makes: the two
// surfaces restyle one annotation through one verb, so a word that differed
// between them would be two names for one property — and the ribbon's copy is
// the one an operator meets first, so it is the copy that would teach them the
// wrong name. What is below is what the panel has no equivalent of, because the
// panel does not offer a fill or an arrowhead at all.
// ---------------------------------------------------------------------------

/// The fill swatch's *no fill* state — `MarkupStyle::interior` set to
/// `StyleEdit::Clear`.
#[must_use]
pub fn markup_no_fill() -> &'static str {
    "No fill"
}

/// The four positions the arrowhead chooser offers, in the order it draws them.
#[must_use]
pub fn markup_endings_none() -> &'static str {
    "No arrowheads"
}

/// See [`markup_endings_none`].
#[must_use]
pub fn markup_endings_start() -> &'static str {
    "At the start"
}

/// See [`markup_endings_none`].
#[must_use]
pub fn markup_endings_end() -> &'static str {
    "At the end"
}

/// See [`markup_endings_none`].
#[must_use]
pub fn markup_endings_both() -> &'static str {
    "At both ends"
}

// ---------------------------------------------------------------------------
// MODE LABELS
//
// `MODES_AND_PANELS.md` Part 1. The three positions of the selector at the
// far right of the tab row, ordered by capability: each mode's tab set is
// a superset of the one before it, and the ordering is the information the
// control conveys.
// ---------------------------------------------------------------------------

/// The Read mode's label — a PDF viewer that authors nothing.
#[must_use]
pub fn mode_read() -> &'static str {
    "Read"
}

/// The Review mode's label — the markup stance, plus page operations.
#[must_use]
pub fn mode_review() -> &'static str {
    "Review"
}

/// The Edit mode's label — everything.
#[must_use]
pub fn mode_edit() -> &'static str {
    "Edit"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tab has a question, and every question is a question.
    #[test]
    fn every_tab_question_is_one_question() {
        for q in [
            question_file(),
            question_view(),
            question_pages(),
            question_edit(),
            question_markup(),
            question_measure(),
            question_tools(),
            question_format(),
        ] {
            assert!(q.ends_with('?'), "not a question: {q}");
            assert_eq!(
                q.matches('?').count(),
                1,
                "a tab that needs two questions is carrying two jobs: {q}"
            );
        }
    }

    /// Tab labels are distinct.
    #[test]
    fn tab_labels_are_distinct() {
        let labels = [
            tab_file(),
            tab_view(),
            tab_pages(),
            tab_edit(),
            tab_markup(),
            tab_measure(),
            tab_tools(),
            tab_format(),
        ];
        let mut sorted = labels;
        sorted.sort_unstable();
        let before = sorted.len();
        let mut deduped = sorted.to_vec();
        deduped.dedup();
        assert_eq!(deduped.len(), before, "two tabs share a label: {labels:?}");
    }

    /// The mode labels are the three `MODES_AND_PANELS.md` names, in
    /// capability order.
    #[test]
    fn the_three_modes_are_named_in_capability_order() {
        assert_eq!(
            [mode_read(), mode_review(), mode_edit()],
            ["Read", "Review", "Edit"]
        );
    }
}
