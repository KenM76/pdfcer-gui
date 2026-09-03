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
