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
//! | the tool's **name** | [`crate::shell::menus::MenuHost::label`], i.e. the command registry | a second copy of a label drifts the first time either is reworded, invisibly, because nothing renders both at once |
//! | the **sentence** | [`crate::text::tool`]'s existing per-tool instructions | they were written for the armed block, they are correct, and re-writing them shorter would be an edit nobody asked for |
//! | **Put this tool down** | [`crate::text::tool::put_down_button`] | it is the same verb with the same argument behind it; see that function's |
//!
//! What is left is the **joiner** and the **hover**, and they are here.

/// The one line, assembled: what is armed, then what a press does with it.
///
/// # Why an em dash and not the mock's middle dot
///
/// `mockups/pdfcer-shell.html` renders *"Select · click to pick · drag to
/// marquee"* — a name and two gesture fragments, all separated by `·`. That
/// shape would need a compressed gesture string per tool, none of which exist,
/// and it makes the name look like a third fragment rather than the subject of
/// the line.
///
/// An em dash says *this is the thing, and this is what it does*, which is
/// the actual relationship, and it lets the existing per-tool sentences be
/// used verbatim. The mock is a design reference; where it disagrees with a
/// sentence that is already written and tested, the sentence wins.
///
/// `name` is never formatted into the sentence and the sentence is never
/// truncated here. Truncation is the **strip's** business — it has a clip
/// rectangle and the caller elides against it — because a catalog function
/// that shortened its own output would put a layout decision in a file with
/// no way to measure one.
#[must_use]
pub fn status_line(name: &str, sentence: &str) -> String {
    format!("{name} — {sentence}")
}

/// The hover on the status line.
///
/// # It says where the controls are, because that is the strip's one hazard
///
/// Properties owns the text pen's font, size and colour, the measure
/// pick-list and the three scale switches. An operator hunting for them
/// reaches for the armed tool first, finds one line, and reasonably concludes
/// the capability was removed — the failure this project names
/// *"The feature works. He could not find it."*
///
/// So the strip's hover is not a description of the strip. It is a pointer to
/// the surface that owns the controls.
#[must_use]
pub const fn status_tooltip() -> &'static str {
    "What you are holding. Its settings — font, size, colour, measuring \
     options, resize switches — are in Properties."
}
