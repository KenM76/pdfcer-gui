//! # `text::panels::face` — every string the face chooser shows
//!
//! One control, two surfaces, one catalog. The Properties panel's *This text*
//! section and the ribbon's Format ▸ Font group draw the **same** face chooser
//! through [`crate::panels::properties::face`], so its wording lives in its own
//! module rather than inside [`super::properties`] — where it had been until
//! 2026-08-29, and where it was beginning to be the largest single subject in a
//! 1,466-line file.
//!
//! ## ★★★ Why this module exists at all, and what it is obliged to say
//!
//! `pdfcer-core` v0.15.0 (`Pass 162.0`) closed the last of the four things the
//! operator named as not fully editable. Its release note, verbatim:
//!
//! > **FONTS** — text can be restyled to a face the document **DOES NOT
//! > CONTAIN**, for the fourteen faces every PDF reader is required to have.
//! > pdfcer authors the font resource on demand, with widths, embedding
//! > nothing. A face outside those fourteen still refuses by name — that needs
//! > a real font program.
//!
//! Three clauses in that note become three obligations on the wording here, and
//! every string below discharges one of them:
//!
//! 1. **"a face the document does not contain"** — the chooser now offers two
//!    *kinds* of row, and they are different acts. Choosing a face the page
//!    already carries changes a `Tf` operand and nothing else. Choosing one of
//!    the fourteen makes pdfcer **write a new object into the operator's file**.
//!    An operator who cannot tell those apart has been handed a control that
//!    does two different things under one appearance. [`face_group_on_page`]
//!    and [`face_group_addable`] are the two headings that separate them.
//!
//! 2. **"embedding nothing"** — [`face_addable_disclosure`], and it is the
//!    reason this module has a header this long. See its own doc comment.
//!
//! 3. **"a face outside those fourteen still refuses by name"** — not a string
//!    in this module, because that refusal is a *status-bar* sentence and lives
//!    with the others in
//!    [`crate::text::status::selection::TextStyleRefusal::FaceNotOnPage`],
//!    whose wording was corrected in the same change. It is named here so the
//!    reader of this header can find it.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//! - **Never state a capability the build does not have** — and, the half this
//!   module had to learn, never keep stating a *limit* the build no longer has.

/// Label for the face chooser, in the Properties panel.
///
/// ★ The **ribbon's** copy of this control has no label beside it: the group's
/// caption already says *Font*, the control shows the current face, and Word's
/// own font-name box carries no label for the same two reasons. A label there
/// would be the third occurrence of the word within one inch of ribbon. In the
/// panel the rows are stacked and each needs its noun.
#[must_use]
pub const fn text_face_label() -> &'static str {
    "Font"
}

/// Shown in the face chooser when **nothing at all** can be offered for this
/// run.
///
/// ★★★ **A real state, and one that got rarer rather than going away.**
///
/// Before the pre-flight (`Pass 142.1`) the chooser listed every `/BaseFont` on
/// the page and an operator found out which ones could not work by pressing
/// them. After it, the list held only faces `set_font` had already accepted
/// **for this run** — and on a page where that set was empty, an empty combo
/// read as a broken control, so this sentence was written.
///
/// ★★ Since `Pass 162.0` the list also carries the fourteen standard faces, so
/// reaching this sentence now means something stronger than it used to: not one
/// font on this page can show these characters **and** every one of the fourteen
/// is either already on the page in a form that cannot show them, or was not
/// offered. In practice that is a run of characters no `WinAnsi`-encoded face
/// covers — a symbol font's own glyphs, most often a title-block logo.
///
/// ★ It names the reason at the level an operator can act on: the fonts are
/// there, and what they cannot do is show *these characters*. That is why a
/// title-block label in a symbol font offers nothing while the paragraph beside
/// it offers four.
#[must_use]
pub const fn text_face_none() -> &'static str {
    "No other font can show these characters — not the ones on this page, and not the standard \
     fourteen."
}

/// Hover for a face whose `/BaseFont` is shared by a second resource.
///
/// ★★ Two rows reading identically is otherwise indistinguishable from a bug,
/// and the survey behind the Fonts panel found **two subsets of one face in
/// 87 % of embedding files** — so this is the routine case, not the exotic one.
/// The operator has a real choice between them and pdfcer reaches the one the
/// row is about, by resource key rather than by name.
#[must_use]
pub const fn text_face_ambiguous() -> &'static str {
    "This page carries two fonts with this name — two subsets of one face. Choosing this \
     row uses this one."
}

/// The heading over the rows the **page already carries**.
///
/// ★ *"On this page"* rather than *"In this document"*, and the difference is
/// the engine's rather than a preference. `preview_font_resources` enumerates
/// the `/Font` resources of **one page's** resource dictionary — §7.8.3 makes a
/// resource name local to the stream it is used from — so a face on page 4 is
/// not offered here and would not be found by `set_font` if it were. A heading
/// saying *document* would be describing a scope the answer below it does not
/// have.
///
/// ★★ The heading is drawn **even when there is only one group**, and that is
/// deliberate: the operator's question is *which of these will change my file*,
/// and a list whose two halves are labelled only when both are present teaches
/// them to read the labels sometimes.
#[must_use]
pub const fn face_group_on_page() -> &'static str {
    "On this page"
}

/// The heading over the rows pdfcer would **add to the document**.
///
/// ★★★ It is worded as an **act**, not as a category. *"Standard fonts"* would
/// be the librarian's heading and would leave the operator to work out that
/// picking one writes to their file; *"pdfcer can add"* says what the click does
/// before it is clicked, which is R83's whole shape — the operator learns before
/// the gesture rather than from a disclosure after it.
#[must_use]
pub const fn face_group_addable() -> &'static str {
    "pdfcer can add these"
}

/// ★★★ **The disclosure this feature owes**, said once, where the choice is
/// made.
///
/// # The inference the operator cannot see
///
/// `pdfcer-core`'s own release note for `Pass 162.0`: pdfcer *"authors the font
/// resource on demand, with widths, embedding nothing."* §9.6.2.2 permits that
/// for exactly these fourteen faces — a four-key dictionary with no
/// `/FontFile`, no `/FontDescriptor`, and no glyph outlines anywhere in the
/// file.
///
/// ⇒ **The text is then drawn with the reader's own copy of that face.** Which
/// is invisible on this screen, because the copy this machine renders with is
/// the one the operator is looking at, and visible on somebody else's machine,
/// where it is a different copy. That is rule 4's surviving half stated as
/// plainly as it can be: *an inference the operator cannot see still owes an
/// off-canvas report.* A screenshot of the canvas here and a screenshot of the
/// same file opened elsewhere may genuinely differ, and nothing on this canvas
/// can say so — so the sentence has to.
///
/// # ★★ Once, and where they choose
///
/// It is a **visible label under the group heading**, not a hover, and not a
/// hover repeated on each of fourteen rows. Fourteen copies of one sentence is
/// a nag; a hover is a sentence the operator has to go looking for, and this one
/// is owed to every operator who opens the list, including the one who chooses
/// nothing. It is drawn only when at least one addable row is present, so a page
/// carrying all fourteen already never shows it.
///
/// # What each clause is doing, and why none of them is decoration
///
/// | clause | the fact, and why it is owed |
/// |---|---|
/// | *"adds it to the document"* | the act. A row in a font menu does not otherwise read as a write. |
/// | *"the face's name and its letter widths — not the font program"* | what is actually written. It is also the answer to *"will my file get big?"*, without quoting a byte count this shell has not measured. |
/// | *"drawn with each reader's own copy"* | ★★★ the inference above. The clause the whole disclosure exists for. |
/// | *"Every PDF reader carries these fourteen, so it will always show"* | the reassurance that keeps the clause above from reading as a warning against using the feature. Sourced from the engine's release note — *"the fourteen faces every PDF reader is required to have"* — and not from a general claim about readers. |
/// | *"on another machine the letters may be set a little differently"* | the consequence, in the operator's terms. Not "metrics may vary": what they will see is a line that wraps one word earlier. |
///
/// ★ It does **not** promise that the fourteen render *identically* everywhere.
/// They do not — that is the entire content of the third clause — and a
/// sentence claiming they did would be the comfortable version of this
/// disclosure rather than the true one.
#[must_use]
pub const fn face_addable_disclosure() -> &'static str {
    "Choosing one of these adds it to the document. pdfcer writes the face's name and its letter \
     widths, not the font program, so the text is drawn with each reader's own copy of that \
     face. Every PDF reader carries these fourteen, so it will always show; on another machine \
     the letters may be set a little differently from what you see here."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The disclosure carries all three facts it exists to carry.**
    ///
    /// Asserted by content rather than by exact text, because the wording will
    /// be improved and the facts must not be lost in the improving. Each of the
    /// three is a separate obligation and each has its own way of going missing:
    ///
    /// 1. **The act** — that choosing the row writes to the file. Lost if the
    ///    sentence is ever rewritten as a description of what the fourteen
    ///    *are*.
    /// 2. **Not embedded** — the fact `pdfcer-core` states and this shell relays.
    ///    Lost first, because it is the least comfortable clause.
    /// 3. **The reader's own copy** — the inference the operator cannot see, and
    ///    the whole reason rule 4 puts this sentence on screen rather than in a
    ///    doc comment.
    #[test]
    fn the_disclosure_states_the_act_the_omission_and_the_consequence() {
        let line = face_addable_disclosure();
        assert!(line.contains("adds it to the document"), "{line}");
        assert!(line.contains("not the font program"), "{line}");
        assert!(line.contains("reader's own copy"), "{line}");
    }

    /// ★★ **The two group headings are not paraphrases of each other.**
    ///
    /// They are the only thing distinguishing two rows that may read
    /// identically — `Helvetica` the page carries and `Helvetica` pdfcer would
    /// add are one string apart on screen and two different acts in the file.
    /// A pair of headings differing by a word an operator skims past would put
    /// the whole distinction back where it was before this change: nowhere.
    #[test]
    fn the_two_group_headings_say_different_things() {
        assert_ne!(face_group_on_page(), face_group_addable());
        // The addable heading must read as an act pdfcer performs. "Standard
        // fonts" would pass the inequality above and fail the operator.
        assert!(
            face_group_addable().contains("add"),
            "the addable heading must name the act: {}",
            face_group_addable()
        );
    }

    /// ★ **The empty-list sentence accounts for BOTH sources**, since the
    /// standard fourteen joined the page's own fonts.
    ///
    /// It said *"No other font on this page can show these characters"* until
    /// 2026-08-29, which was exhaustive when the page was the only source and
    /// became a half-answer the moment it was not. An operator reading the old
    /// sentence beside a list that elsewhere offers `Times-Roman` out of thin
    /// air would reasonably ask why it is not offered here.
    #[test]
    fn the_empty_sentence_accounts_for_the_standard_fourteen() {
        let line = text_face_none();
        assert!(line.contains("this page"), "{line}");
        assert!(line.contains("fourteen"), "{line}");
    }
}
