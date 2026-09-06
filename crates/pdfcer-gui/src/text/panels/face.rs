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

// ===========================================================================
// ★★★ O141 — the offer that turns a refused character into a face that has it
// ===========================================================================
//
// The operator, 2026-09-05: *"if the character isn't available in a pdf are we
// able to change to a different font?"*
//
// Yes, and every piece already existed — the engine refuses by name, the
// refusal carries the character, and this chooser has offered the standard
// fourteen since `Pass 162.0`. **Nothing connected the refusal to the
// chooser.** These five strings are the connection's words; the surface is
// `crate::panels::properties::refusedchar`.
//
// ★ They live in THIS module rather than in `crate::text::textedit` because
// the surface they belong to *is* a face chooser: it draws the same two-group
// popup through `panels::properties::face::popup_body` and owes the same
// disclosure. Splitting the offer's words from the chooser's words is how two
// wordings of one act grow up beside each other, which is the divergence this
// module's own header exists to record.

/// The heading over the offer block.
///
/// ★★ It names the **character's** problem, not the font's, and not the
/// operator's. *"Unsupported font"* would be the engine's noun; *"That
/// character isn't available"* would be the operator's own phrasing handed back
/// to them without an answer. What an operator needs at the top of this block
/// is the fact that decides what they do next: this font, this character, no.
#[must_use]
pub const fn refused_char_heading() -> &'static str {
    "A character this font cannot type"
}

/// ★★★ **The sentence that names the character** — the half `Declined::line`
/// structurally cannot say.
///
/// The status bar's `⊗` slot returns `&'static str` and is truncated to 45 % of
/// the bar; a panel can interpolate and wrap. So the naming happens here, beside
/// the control that answers it, which is also where
/// `REVIEW_TRIAGE.md`'s *"every disclosure above the thing it qualifies"* wants
/// it.
///
/// # Why the FONT is named too
///
/// Because the operator's next question is *"which font?"*, and on a page with
/// four faces the answer decides whether they believe the block at all. It is
/// the shortened `/BaseFont` — the same spelling the chooser's rows use — so the
/// name in this sentence and the name in the list are the same string.
///
/// # ★★ Why *"the letters your page already prints"* and never *"subset"*
///
/// O141's framing, in the operator's own words: he asked this question without
/// the word, and *"the operator should be able to get from the refusal to a face
/// that can type it, without knowing what a subset is."* The clause also happens
/// to be the whole mechanism — a producer embeds the letters the page used and
/// no others — so nothing is lost by saying it in English.
#[must_use]
pub fn refused_char_named(character: char, font: &str) -> String {
    format!(
        "The “{character}” is not one of the letters {font} carries. Fonts inside a PDF usually \
         hold only the letters your page already prints, and pdfcer cannot add one to a font that \
         is already in the file."
    )
}

/// The instruction under [`refused_char_named`], and the label on the chooser.
///
/// # ★★★ It promised TWO gestures until 2026-09-05, and now promises one
///
/// It read: *"Pick a font that has the “q”, then click in the text and type it
/// again"* — an accurate description of a route that stopped one gesture short
/// of what the operator asked for. `Ctrl+Enter` calls `commit_into` and then
/// `abandon` whether or not the engine accepted, so the words he typed were
/// thrown away by the refusal and he had to produce them a second time from
/// memory.
///
/// They are kept now ([`crate::canvas::textedit::Committing`]) and travel with
/// the refusal, so **taking the offer re-applies the edit he already made**. The
/// sentence says so, because a control that quietly does more than it claims is
/// as hard to trust as one that does less.
#[must_use]
pub fn refused_char_offer(character: char) -> String {
    format!("Pick a font that has the “{character}” and pdfcer will put your change in with it:")
}

/// ★★★ **The honest limit on the offer**, filed rather than hidden.
///
/// `preview_font_resources` coverage-tests **the characters already in the
/// run**, not the one about to be typed. So a row in this list can be a face
/// that then refuses the operator's character, and the refusal is a sentence
/// rather than a greyed-out row.
///
/// # Why the list is not silently filtered to look confident
///
/// The standing ruling on this exact surface, taken from the Bold button: *"Do
/// not grey out a bold button. Offer it, and surface the disclosure."* Filtering
/// would need this crate to re-derive which face uses `WinAnsiEncoding`, which
/// two use a built-in symbolic one, and what that leaves unmapped —
/// `FontPreflight`'s own invariant (`R221`) forbids exactly that, and a second
/// copy of the rule in `pdfcer-gui` drifts from the commit path the first time
/// the rule changes. Filed at the engine as
/// `request_font_preflight_tests_the_text_that_is_there_not_the_text_about_to_be_typed.md`;
/// nothing is blocked on the reply.
///
/// ★ The sentence promises what happens on the bad case — *pdfcer will say so
/// and change nothing* — because a caveat that names a risk without naming its
/// consequence reads as a reason not to press the control.
#[must_use]
pub fn refused_char_untested(character: char) -> String {
    format!(
        "pdfcer checked these fonts against the words already here, not against the \
         “{character}”. If the one you pick cannot type it either, pdfcer will say so and change \
         nothing."
    )
}

/// ★★ **What the block says on the frame the face swap lands** — the second
/// half of the route, now carried out rather than described.
///
/// The swap is an edit, so it retires the offer; without this the block would
/// vanish at the moment the operator most needs to be told what happened, and
/// they would be left looking at a page that changed for a reason nothing named.
///
/// ★ It names the face that is now in force, because that is the one fact the
/// canvas cannot show them: on a metric-compatible swap — `Arimo-Bold` to
/// `Helvetica-Bold` moved the operator's own line by 0.005 pt — the page looks
/// exactly as it did, and a block saying nothing would leave them unsure whether
/// anything happened at all.
///
/// ★★★ It said *"Click in it and type the “q” again"* until 2026-09-05. It does
/// not any more, because the block re-applies the edit itself on this very
/// frame — and an instruction to do something the program has already done is
/// worse than none: the operator follows it, types the character into a document
/// that already has it, and gets a second copy.
#[must_use]
pub fn refused_char_swapped(character: char, font: &str) -> String {
    format!("This text is now set in {font}, and pdfcer is putting your “{character}” in with it.")
}

/// ★★ **The one state that still asks the operator to type it again**, and it
/// is the wording every other sentence here retired on 2026-09-05.
///
/// Reached when the refusal arrived with **no carried words** —
/// `RefusedCharacter::typed` is `None`, which happens if the plan that produced
/// the refusal named a different `(page, run)` than the refusal did. That is a
/// disagreement between two facts about one commit, and this shell declines to
/// paper over it: rather than guess at what the operator typed, it swaps the
/// face (which is real and useful on its own) and asks for the character again.
///
/// ★ So this is not a leftover. It is the honest sentence for the one state in
/// which the instruction is true, and keeping it is what lets
/// [`refused_char_swapped`] and [`refused_char_blocked`] be unambiguous about
/// their own states. A single sentence covering all three would have to hedge,
/// and a hedged instruction is one the operator cannot follow.
#[must_use]
pub fn refused_char_swapped_type_again(character: char, font: &str) -> String {
    format!(
        "This text is now set in {font}. Click in it and type the \u{201c}{character}\u{201d} \
         again, and it will go in."
    )
}

/// ★★★ **What the block says when the swap landed and the character still would
/// not go in** — the third state, added 2026-09-05, and **re-worded on
/// 2026-09-06 when the one cause it used to name stopped existing.**
///
/// # ★★★ What this sentence used to say, and why saying it now would be a lie
///
/// Until engine v0.41.0 it read, in full:
///
/// > *"…pdfcer cannot type into a font it has just added to a file until that
/// > file has been saved and opened again. Save this document, open it, and
/// > type the "q" once more — it will go in then. This limit is pdfcer's own
/// > and is on the list to fix."*
///
/// Every clause of that was true and measured. `EditSession::format_text`
/// allocated the new `/Font` in the session overlay; `EditSession::edit_text`
/// planned with `plan_edit(&self.base, …)`; `resolve_font_dict` dereferenced
/// the run's `Tf` name through the base revision and answered `None`. The save
/// and the reopen were not a guess at a remedy — they were literally what two
/// invocations of `pdfcer.exe` did, and
/// `fixtures/subset-font-floor.PROVENANCE.md` records the four commands.
///
/// **`Pass 257.0` (engine `5e95805`, released in v0.41.0) removed that cause.**
/// Every text-edit planner takes `&DocumentView<'_>`, every `EditSession` verb
/// passes `self.view()`, and there is no coercion from `&Document`, so the
/// class is a compile error rather than a refusal.
/// [`crate::canvas::textedit::facewall`] now asserts the **success** in both
/// request shapes, with no save and no reopen between the verbs.
///
/// # ★★★ So why does this sentence still exist? Because the STATE does.
///
/// [`crate::canvas::textedit::facewall`]'s own module header instructed its
/// reader to **delete** this function and the `retried` arm that shows it, on
/// the reasoning that the retype would now always land and the state would
/// become unreachable. **That reasoning is wrong, and the error is worth
/// naming**, because it is the same shape as the defect the sentence was about.
///
/// The block is not reached by recognising this cause. It is reached by
/// **arithmetic**, as the section below explains: *the retype was raised and
/// the document did not change*. That condition is agnostic about **why**, and
/// there are other whys — an offered face that turns out not to cover the
/// character after all, a run whose operators the pinned request cannot span, a
/// document the engine declines for a reason nothing here has met yet. Deleting
/// the arm would convert every one of those into **silence**, which is the
/// standing cross-cutting defect this project already has on its own list
/// (*"every engine refusal reaches the operator as SILENCE"*).
///
/// ⇒ **A state and its explanation have different lifetimes.** The explanation
/// expired; the state did not. So the cause is struck and the state keeps its
/// voice — which is the conservative direction, because an operator told *"it
/// did not go in"* can still act, and one told nothing cannot.
///
/// # ★★ Why this state is reached by ARITHMETIC and not by reading the error
///
/// The block never inspects the engine's refusal. It knows what it asked for and
/// it watches `doc.edit_epoch`: the swap moves it once, and had the retype
/// landed it would move again and the block would retire on the next frame. So a
/// block still on screen one frame after the retype **is** the retype having
/// been refused — no prose grepped, no second copy of the engine's taxonomy,
/// which is precisely what `app::status::decline::textedit`'s header forbids and
/// what a string match on *"unresolvable"* would have been.
///
/// ★ That mechanism is exactly why the old wording was a hazard the moment the
/// engine moved: **a surface that cannot see a cause must not name one.** It
/// named the only cause anybody had met, which read as a measurement and was an
/// inference. The replacement says what the block actually knows — the face
/// changed, the character did not go in — and stops there.
///
/// # What it must NOT do
///
/// It must not say *"something went wrong"*: the face swap **did** land, that
/// is a real change to his document, and a sentence that implies otherwise
/// would send him looking for damage that is not there. And it must not
/// prescribe save-and-reopen, which is now a remedy for nothing and would cost
/// him two gestures to learn that.
#[must_use]
pub fn refused_char_blocked(character: char, font: &str) -> String {
    format!(
        "This text is now set in {font} — that part worked and is in your document. The \
         “{character}” still would not go in, and pdfcer has not been told why. Nothing was \
         damaged: the text is exactly as it was, in the new face. Try typing the \
         “{character}” again, and if it is still refused the reason will be named in the \
         status bar."
    )
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

    /// ★★★ **The offer names the character, every time, in every sentence that
    /// mentions it** — `OPERATOR_REQUESTS.md` O141.
    ///
    /// The whole of what the status bar cannot do. A block that said *"a
    /// character in this text"* would have moved the refusal to a wider surface
    /// and added nothing: the operator already knows they typed something, and
    /// what they do not know is **which** keystroke the document refused — on a
    /// pasted line it can be a character they never saw themselves type.
    ///
    /// ★ Asserted over a character outside ASCII on purpose. A build that
    /// formatted with `{:?}` or escaped for a byte-oriented surface would print
    /// `'\u{20ac}'` and pass a test written against `'q'`.
    #[test]
    fn every_sentence_in_the_offer_names_the_character_itself() {
        for line in [
            refused_char_named('€', "Arimo-Bold"),
            refused_char_offer('€'),
            refused_char_untested('€'),
            refused_char_swapped('€', "Helvetica-Bold"),
        ] {
            assert!(
                line.contains('€'),
                "the character is the one fact the status bar cannot carry: {line}"
            );
            assert!(
                !line.contains("20ac") && !line.contains("20AC"),
                "a code point is not what the operator typed: {line}"
            );
        }
    }

    /// ★★ **Both font-naming sentences name the font**, and they name two
    /// different ones.
    ///
    /// [`refused_char_named`] names the face that **refused**;
    /// [`refused_char_swapped`] names the face that is **now in force**. A build
    /// that fed either the wrong one would tell the operator that the font they
    /// just chose is the font that cannot type their character — which reads as
    /// the feature not working, on a swap that worked.
    #[test]
    fn the_offer_names_the_font_that_refused_and_the_font_that_replaced_it() {
        let refused = refused_char_named('q', "AAAAAA+Arimo-Bold");
        assert!(refused.contains("AAAAAA+Arimo-Bold"), "{refused}");
        let swapped = refused_char_swapped('q', "Helvetica-Bold");
        assert!(swapped.contains("Helvetica-Bold"), "{swapped}");
        assert!(
            !swapped.contains("cannot"),
            "the follow-up reports a success and must not read like a second refusal: {swapped}"
        );
    }

    /// ★★★ **The offer promises that pdfcer will finish the job, and neither
    /// sentence tells the operator to retype anything.**
    ///
    /// # This test asserted the OPPOSITE until 2026-09-05, and the inversion is
    /// the point
    ///
    /// It was `the_offer_says_to_type_the_character_again`, and it required both
    /// sentences to contain *"type it again"* — a correct assertion about a
    /// route that stopped one gesture short of what the operator asked for.
    /// `Ctrl+Enter` threw his draft away on the refusal, so choosing a face left
    /// him to produce his own edit a second time from memory.
    ///
    /// The block re-applies it now
    /// ([`crate::panels::properties::refusedchar`]), so the instruction is not
    /// merely unnecessary — it is **harmful**: an operator who follows it types
    /// the character into a document that already has it and gets two.
    ///
    /// ★ Asserted in the negative as well as the positive, because a build that
    /// re-applied the edit *and* kept the old wording would pass a
    /// promise-only test while producing exactly that double edit.
    #[test]
    fn the_offer_promises_pdfcer_finishes_the_job_and_never_asks_for_a_retype() {
        let offer = refused_char_offer('%');
        assert!(
            offer.contains("pdfcer will put your change in"),
            "the operator must know his edit is coming with the swap, or the font list \
             reads as a route that ends in a list: {offer}"
        );
        let swapped = refused_char_swapped('%', "Courier");
        assert!(
            swapped.contains("putting your “%” in"),
            "the follow-up must report what is happening to his edit: {swapped}"
        );
        for line in [offer, swapped] {
            assert!(
                !line.contains("again"),
                "asking for a retype after the retype has been made produces a second \
                 copy of the character: {line}"
            );
        }
    }

    /// ★★★ **The third state names the obstacle, the remedy and whose limit it
    /// is** — and it is the one sentence here that asks the operator to type
    /// again, because in that state it is true.
    ///
    /// The face swap landed and the character still would not go in — and since
    /// 2026-09-06 **pdfcer does not know why**, because `Pass 257.0` removed the
    /// only cause this state had ever been observed to have and the block is
    /// reached by arithmetic rather than by recognising a refusal.
    ///
    /// ★★★ So the obligations asserted here changed direction. The old test
    /// required the sentence to carry the *remedy* (*"saved and opened again"*).
    /// This one requires it to carry **no cause at all**, which is the harder
    /// property to hold: a later session improving the wording will be tempted
    /// to put an explanation back, and the explanation it reaches for will be
    /// the one written in the git history of this very file — which is now
    /// false. The negative assertions below exist to stop exactly that.
    #[test]
    fn the_blocked_sentence_names_no_cause_it_cannot_see() {
        let line = refused_char_blocked('%', "Courier");
        assert!(
            line.contains("Courier"),
            "the face that IS in force, because it really is in force: {line}"
        );
        assert!(
            line.contains("not been told why"),
            "the block cannot see the refusal — it infers one from the edit epoch not \
             moving — so the sentence must say so rather than pick a cause: {line}"
        );
        assert!(
            !line.contains("saved and opened again") && !line.contains("Save this document"),
            "★ save-and-reopen was the remedy for the base-revision font resolution the \
             engine fixed in v0.41.0 (Pass 257.0). Prescribing it now costs the operator \
             two gestures to learn it does nothing: {line}"
        );
        assert!(
            !line.contains("cannot type into a font it has just added"),
            "★★ and it must not name that cause either. It was measured, it was true, \
             and it stopped being true — which is why this assertion is here rather \
             than a comment: {line}"
        );
        assert!(
            line.contains("Nothing was damaged"),
            "the swap DID reach his document, so an operator reading a failure notice \
             must be told what survived or he will go looking for damage: {line}"
        );
    }

    /// ★★ **The caveat names the limit AND what happens when it bites.**
    ///
    /// `preview_font_resources` tests the text that is there, not the text about
    /// to be typed, so a row can still refuse. Saying so without saying that the
    /// document survives it would make the caveat read as a reason not to press
    /// the control — which is how an honest disclosure turns into a deterrent.
    #[test]
    fn the_caveat_says_what_happens_when_the_face_refuses_too() {
        let line = refused_char_untested('€');
        assert!(line.contains("not against"), "{line}");
        assert!(
            line.contains("change nothing"),
            "the consequence is the half that keeps this from reading as a warning: {line}"
        );
    }
}
