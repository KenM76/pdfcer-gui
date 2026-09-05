//! # `text::commands::file` — the File tab's command copy
//!
//! Split out of [`super`] on 2026-09-02 under R2, when that file reached the
//! 1,500-line ceiling and Save As could not be added without one. The seam is
//! the one this directory already uses: [`super::annotate`] and [`super::view`]
//! are the same split for their own tabs, so this is the third of a shape
//! rather than a new idea.
//!
//! ★ Only the entries that need to be here have moved. The rest of the File
//! tab's copy stays in [`super`] until the next thing pushes it over, because
//! moving text that nothing is asking about would be churn dressed as tidying —
//! and every move is a chance for a string to lose its `ui-text-exempt` context
//! or its place in a gate's scan.

use super::CommandText;

/// ★★★ **Export image** — `OPERATOR_REQUESTS.md` **O120**, the operator's own
/// ask of 2026-09-03: *"can you add the ability to export page(es) to png, jpg,
/// svg."*
///
/// # The label is `RIBBON_IA.md` §5.1's, and the tooltip is not the label again
///
/// §5.1's Export band already carried the row — *Export image… (PNG/JPEG/TIFF,
/// DPI picker)* — so the label is settled rather than chosen. What the tooltip
/// has to add is the thing a label cannot say and an operator cannot find out
/// by pressing: **which formats are actually offered**, because that is the
/// decision they are about to make and one of the three names in the IA row is
/// not one that shipped. TIFF has no encoder in the engine; SVG arrived after
/// the row was written and is the one that makes his second sentence — *"copy
/// and paste vector graphics into word or inkscape"* — possible at all.
///
/// ★★ **And it names transparency**, because that is the half of the request he
/// put in a parenthesis and an exclamation mark — *"(including transparency
/// where supported!)"* — and a tooltip that omitted it would leave him pressing
/// the control to find out whether the thing he asked for by name is there.
pub const fn file_export_image() -> CommandText {
    CommandText::new(
        "Export image…",
        "Save pages as PNG, JPEG or SVG pictures, at a resolution you choose \
         and with the page's transparency kept where the format allows it.",
    )
}

/// ★★★ **Export text** — the operator's ask of 2026-09-04: *"also the engine
/// can export PDFs as text. we should have export/import for that."*
///
/// # ★★ The tooltip's job is the difference from its two NEIGHBOURS
///
/// This control sits in a band that already contains `Copy page text` and
/// `Copy document text`, and an operator scanning it is entitled to know why
/// there are three ways to get words out of a drawing. The answer is one word —
/// **a file** — so the tooltip leads with it.
///
/// # ★★★ And it names the one thing that makes the export empty
///
/// A scanned drawing has no text layer, so this export finds nothing on it. The
/// receipt says so afterwards and names `Recognise text`, but a tooltip that
/// warned nobody would let an operator press the control, wait for an
/// extraction, and be told no. The sentence is short and it is the one an
/// operator with a plotted sheet needs before pressing.
///
/// ⇒ **The tooltip deliberately says nothing about importing text**, because
/// nothing imports text: `pdfcer-core` offers no route from a text file back
/// into a PDF. See `crate::app::actions::exporttext`'s header. A tooltip that
/// mentioned a round trip would be the promise R9 forbids.
pub const fn file_export_text() -> CommandText {
    CommandText::new(
        "Export text…",
        "Write the words on the page — or on every page — to a plain text file, \
         encoded as UTF-8. Only the words travel: layout, fonts and position do \
         not, so a table arrives as lines. A scanned page carries no words to \
         export; use Recognise text on it first.",
    )
}

/// **Save As** — `OPERATOR_REQUESTS.md` O95.
///
/// ★★ The tooltip's job is the **difference from its neighbour**: the two labels
/// are one word apart and the acts are not. It says so in the tooltip rather
/// than only in the receipt, because the receipt arrives after the bytes are
/// written, and a control whose consequence is explained only afterwards is one
/// the operator learns by being surprised.
pub const fn file_save_as() -> CommandText {
    CommandText::new(
        "Save as…",
        "Write the document to a file you choose, and carry on editing THAT \
         file — the next Save goes to it, not to the original. Use Save a copy \
         instead when you want a snapshot to send somewhere and want to keep \
         editing the file you already have.",
    )
}
