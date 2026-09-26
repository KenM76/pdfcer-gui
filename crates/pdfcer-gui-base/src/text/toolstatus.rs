//! # `text::toolstatus` — the words of the one-line tool status
//!
//! `OPERATOR_REQUESTS.md` **O123**, in the operator's own words:
//!
//! > *"The Tool panel becomes a one-line tool status (name, one sentence,
//! > 'Put this tool down'); its buttons duplicate the ribbon and go."*
//!
//! ## Why a module of its own rather than four more functions in
//! [`crate::text::tool`]
//!
//! The two catalogues answer different questions. [`crate::text::tool`] holds
//! the vocabulary of the **Properties surface** that owns a tool's settings —
//! headings, per-tool teaching sentences, option labels. This module holds the
//! vocabulary of the **one-line strip** that says what is armed. Keeping them
//! apart means a reader asking *"what does the tool status say?"* is not
//! reading a file whose bulk is about a different surface.
//!
//! ## There are only two strings here, and that is the design
//!
//! The status line is **name · sentence · put-it-down**, and three of those
//! four things are already written down somewhere authoritative:
//!
//! | fragment | where it comes from | why not here |
//! |---|---|---|
//! | the tool's **name** | `pdfcer_gui::shell::menus::MenuHost::label`, i.e. the command registry | a second copy of a label drifts the first time either is reworded, invisibly, because nothing renders both at once |
//! | the **sentence** | [`crate::text::tool`]'s existing per-tool instructions | they were written for the armed block, they are correct, and re-writing them shorter would be an edit nobody asked for |
//! | **Put this tool down** | [`crate::text::tool::put_down_button`] | it is the same verb with the same argument behind it; see that function's |
//!
//! What is left is the **joiner** and the **hover**, and they are here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/toolstatus.md`.

/// The one line, assembled: what is armed, then what a press does with it.
#[must_use]
pub fn status_line(name: &str, sentence: &str) -> String {
    format!("{name} — {sentence}")
}

/// The hover on the status line.
#[must_use]
pub const fn status_tooltip() -> &'static str {
    "What you are holding. Its settings — font, size, colour, measuring \
     options, resize switches — are in Properties."
}
