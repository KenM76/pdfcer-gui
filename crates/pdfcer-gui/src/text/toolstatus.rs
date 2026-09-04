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
//! Two reasons, and the second is the load-bearing one.
//!
//! 1. `crate::text::tool` is 890 lines and R2 caps a file at 1,500. That is
//!    housekeeping.
//! 2. **The two catalogues answer different questions and one of them is
//!    about to shrink.** `text::tool` holds the *panel's* vocabulary — its
//!    headings, its per-tool teaching sentences, its option labels. Most of
//!    those survive O123 by **moving to Properties**, and the handful that do
//!    not are the ones the panel's buttons carried. A new module is where the
//!    strings that are genuinely new live, so that the day somebody asks
//!    *"what did the tool status say?"* they are not reading a file whose
//!    other 880 lines are about a surface that no longer exists.
//!
//! ## ★ There are only two strings here, and that is the design
//!
//! The status line is **name · sentence · put-it-down**, and three of those
//! four things are already written down somewhere authoritative:
//!
//! | fragment | where it comes from | why not here |
//! |---|---|---|
//! | the tool's **name** | [`crate::shell::menus::MenuHost::label`], i.e. the command registry | a second copy of a label drifts the first time either is reworded, invisibly, because nothing renders both at once |
//! | the **sentence** | [`crate::text::tool`]'s existing per-tool instructions | they were written for the armed block, they are correct, and re-writing them shorter would be an edit nobody asked for |
//! | **Put this tool down** | [`crate::text::tool::put_down_button`] | it is the same verb with the same argument behind it; see that function's ★ |
//!
//! What is left is the **joiner** and the **hover**, and they are here.

/// The one line, assembled: what is armed, then what a press does with it.
///
/// # ★★ Why an em dash and not the mock's middle dot
///
/// `mockups/pdfcer-shell.html` renders *"Select · click to pick · drag to
/// marquee"* — a name and two gesture fragments, all separated by `·`. That
/// shape needs nine new compressed strings that do not exist
/// (`SHELL_LAYOUT_PROPOSAL.md` §3.3 measured it at half a day plus a
/// shortening decision), and it makes the name look like a third fragment
/// rather than the subject of the line.
///
/// An em dash says *this is the thing, and this is what it does*, which is
/// the actual relationship, and it lets the existing sentences be used
/// verbatim. The mock is a design reference; where it disagrees with a
/// sentence that has already been written and tested, the sentence wins.
///
/// ★ `name` is never formatted into the sentence and the sentence is never
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
/// # ★ It says where the controls went, because that is this change's one
/// real hazard
///
/// The Tool panel used to hold the text pen's font, size and colour, the
/// measure pick-list and the three scale switches. They are not gone — they
/// are in Properties — but an operator who knew where they were will look
/// here first, find one line, and reasonably conclude the capability was
/// removed. That is the exact failure this project already has a name for:
/// *"The feature works. He could not find it."*
///
/// So the strip's hover is not a description of the strip. It is a pointer to
/// the surface that now owns the controls.
#[must_use]
pub const fn status_tooltip() -> &'static str {
    "What you are holding. Its settings — font, size, colour, measuring \
     options, resize switches — are in Properties."
}
