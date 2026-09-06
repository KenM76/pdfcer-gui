//! # `text::textedit` — every sentence the text-editing tool shows
//!
//! Consumed by `crate::canvas::textedit` and by the `Action::CommitTextEdit`
//! apply arm. Split out of [`super`] rather than added to it for the reason the
//! catalog's header gives: it is split by **area** from the first commit, so the
//! split never has to be done as a migration.
//!
//! ## Two things here are load-bearing rather than cosmetic
//!
//! **★★ [`shares_the_line_note`] is a DISCLOSURE, and it was a refusal until
//! 2026-08-19.** `DEFECTS.md` D4a records that the old shell handled a
//! cross-run selection by setting a flag that *"silently disables the whole
//! typing loop"* — the operator pressed keys and nothing happened. This shell
//! replaced the silence with a sentence, which was the right first move and the
//! wrong final one: **it still refused**, and on a CAD sheet, where a table row
//! is one show operator per cell, it refused nearly every click. The operator
//! reported text editing as not working twice, weeks apart, and was right both
//! times. The refusal is gone; the sentence stayed and changed tense.
//!
//! **[`pinned_tail_disclosure`] is owed under rule 4.** When the follower
//! disposition is `Pin`, the text after the edit does not make room, so a longer
//! replacement grows into it. The engine discloses this for `Reflow` and not for
//! `Pin` — from its side, pinning is what was asked for — so the sentence has to
//! come from here or from nowhere. It names *which* rule pinned, because
//! "right-aligned" and "rotated" are different facts about the operator's
//! document and only one of them is something they chose.

use crate::canvas::textedit::Refusal;
use crate::canvas::textedit::disposition::Reason;
use pdfcer_core::text_edit::BlockAlignment;

/// The sentence for a refusal to place a caret.
///
/// One function over the enum rather than one per variant, so a variant added to
/// [`Refusal`] is a compile error here instead of a caret that refuses silently.
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NoRun => {
            "There is no text where you clicked. Click on a word to put the cursor in it, or use \
             Add text to place new text here."
        }
        Refusal::NoText => {
            "pdfcer cannot read any text on this page. If it is a scan, run Tools > OCR first — \
             editing needs real text, not a picture of it."
        }
        // ★★★ THIS SENTENCE USED TO SAY SOMETHING ELSE, AND IT WAS RETIRED ON
        // 2026-08-20.
        //
        // It read: *"This text is inside a block placed by the program that
        // made the drawing, and pdfcer cannot edit inside one yet — only read
        // it."* `Pass 119.0` made that text editable the same evening the
        // sentence shipped, and on a CAD sheet it was **99 % of the text the
        // operator wants to edit** — his own estimate, and the reason the
        // engine escalated that Pass ahead of everything else.
        //
        // The episode is kept in the comment rather than deleted with the
        // string, because the durable lesson is the one about the guard: this
        // shell asked `TextRun::editability()` instead of matching on
        // provenance itself, so the day the capability landed the cost was one
        // deleted arm that a `#[deprecated]` attribute pointed straight at.
        //
        // ★ What replaces it is a genuinely different fact and therefore needs
        // genuinely different words. `/ActualText` is a producer-supplied
        // replacement string standing in for a span of glyphs — a ligature
        // written out, a logo given a name, a table cell given a reading. There
        // is no show operator behind it, so there is nothing to edit *in
        // place*: the text is not out of reach, there is nothing to reach for.
        //
        // The three obligations the old sentence carried still apply, and are
        // why this one is shaped as it is:
        //
        // 1. **It is not the operator's mistake.** They clicked on real text
        //    and it is real text. "There is no text where you clicked" — the
        //    nearest existing sentence — would be false, and would send them
        //    clicking round the sheet looking for a spot that works.
        // 2. **It says what is different about this text**, in words someone
        //    who has never heard of `/ActualText` can act on.
        // 3. **It says what they can do instead.** A refusal with no route is
        //    half a sentence.
        Refusal::NoAnchor => {
            "This text was supplied as a description rather than drawn as letters, so there is \
             nothing here to edit in place. Use Add text to put new text over it, or change it \
             in the program the drawing came from."
        }
    }
}

/// ★★ The multi-run **disclosure** — what `spans_runs()` used to refuse.
///
/// Until 2026-08-19 this sentence's ancestor was a *refusal*: a click whose
/// visual line was made of more than one show operator placed no caret at all,
/// and the sentence told the operator to *"click directly on the word you want
/// to change"* — advice that could not work, because the refusal was about the
/// **line**, not about where on it they clicked.
///
/// On a SolidWorks sheet — one show operator per table cell, one per title-block
/// field — that refused nearly every click. The operator reported the feature as
/// not working twice, weeks apart, and **he was right both times**.
///
/// The refusal is gone and **the disclosure is the half that was always
/// useful**. It says the same true thing in the same operator's terms — *a run
/// is not a thing anyone can see on a page; what they can see is that the line
/// is made of separate pieces* — and then says what pdfcer is going to do about
/// it instead of stopping.
///
/// Shown when the caret **lands**, not when the edit commits: rule 4's
/// *"announced before it is picked, not after"*, applied to a layout
/// consequence rather than to a geometric inference. The commit-time half is
/// [`pinned_tail_disclosure`], which says the same fact in the past tense.
#[must_use]
pub const fn shares_the_line_note() -> &'static str {
    "This line is drawn as several separate pieces. You are editing the piece you clicked; the \
     pieces beside it will stay exactly where they are, so a longer replacement may overlap \
     them."
}

/// The disclosure appended when the edit pinned the text after it.
///
/// Two sentences and no more, because it shares the status row with everything
/// else and R128 forbids that row growing. The first says what happened; the
/// second says what to watch for.
#[must_use]
pub fn pinned_tail_disclosure(reason: Reason) -> String {
    let because = match reason {
        Reason::Rotated => {
            "this text is rotated, so moving what follows it sideways would move it the wrong way"
        }
        // ★ The commonest reason on this operator's documents by a wide margin,
        // and the one whose wording matters most: he is looking at what appears
        // to be one line and pdfcer has just edited one piece of it.
        Reason::SharesTheLine => {
            "this line is drawn as several separate pieces and the others are not part of your edit"
        }
        Reason::Flush(BlockAlignment::Right) => "this text is right-aligned",
        Reason::Flush(BlockAlignment::Center) => "this text is centred",
        Reason::Flush(BlockAlignment::Justified) => "this text is justified",
        // `BlockAlignment` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every alignment that is not Left.
        Reason::Flush(_) => "the text after this one is lined up against something",
        // Unreachable — neither of these pins — and answered rather than
        // panicked, because a disclosure is not worth a crash in the frame that
        // is trying to draw. See `Reason::pins_the_tail`, which is the predicate
        // the caller gates on.
        Reason::LeftAligned | Reason::AlignmentUndetectable => {
            "the text after this one was kept \
                                                               in place"
        }
    };
    format!(
        "layout: the text after your edit was left exactly where it was, because {because}. If \
         what you typed is longer than what it replaced, it may now overlap — check the page \
         before saving."
    )
}

/// Why reflow declined on a page this session has already changed.
///
/// ★★★ The remedy is the sentence, not the refusal. `reflow_block` is planned
/// against the **base** document — it needs provenance the staging buffer does
/// not carry — so it refuses a page whose content object this session has
/// rewritten, by name, rather than mis-splicing. One typed character is enough
/// to trip it.
///
/// ★★ It says **save and reopen**, in those words, because that is the whole of
/// what an operator has to do and it is not guessable from *"cannot reflow"*.
/// A refusal naming a cause with no remedy is a sentence that leaves somebody
/// trying things — the rule `text::embed`'s blocker rows already follow.
///
/// ★ It does not apologise or call it a limitation. It is a correctness
/// property: the alternative to refusing is splicing base-relative byte offsets
/// into a stream that has moved, which corrupts the page silently.
#[must_use]
pub const fn reflow_after_edit() -> &'static str {
    "Reflowing a paragraph needs the document as it was when you opened it, so it cannot run \
     after other changes. Save this file and open it again, then reflow."
}

/// A reflow that ran and produced the same number of lines.
///
/// ★★ A correct outcome that reads as a failure without a sentence: the
/// paragraph already fitted its box, so re-wrapping it changed nothing visible.
/// Silence here is indistinguishable from a command that did not work, which is
/// the shape this project keeps finding.
#[must_use]
pub const fn reflow_unchanged() -> &'static str {
    "This paragraph already fitted its box, so re-wrapping it changed nothing."
}

/// A reflow asked for with no caret placed.
///
/// ★★★ **The three reflow refusals below are one design decision**: a
/// paragraph command whose operand is the caret has three ways to find no
/// operand, and each of them leaves the operator in a different place. Merging
/// them into one *"nothing to reflow"* would be shorter and would tell somebody
/// with the text tool armed but unclicked exactly nothing.
///
/// ★ It names the tool by the word on its button — *Edit text* — because
/// "place the caret" is our language and not theirs.
#[must_use]
pub const fn reflow_needs_caret() -> &'static str {
    "Click inside the paragraph you want to re-wrap first, using the Edit text tool, then choose \
     Reflow paragraph."
}

/// A reflow asked for while the caret is placing NEW text.
///
/// ★★ `Anchor::Origin` and `Anchor::Box` mean the operator clicked bare page:
/// they are composing text that is not on the page yet, so there is no
/// paragraph to re-wrap and there will not be one until they commit. The
/// sentence says that rather than implying they mis-clicked — they did not.
#[must_use]
pub const fn reflow_needs_existing_text() -> &'static str {
    "The caret is placing new text, so there is no paragraph on the page to re-wrap yet. Finish \
     this text, then click into a paragraph that is already on the page."
}

/// A caret on a run the block recogniser does not place in a paragraph.
///
/// ★★★ The honest one, and the one most likely to be met on the drawings this
/// program is for. A CAD title block is isolated cells, not prose: pdfcer finds
/// no paragraph because there is none, and a re-wrap of a two-word cell would
/// be meaningless even if it ran.
///
/// ★ It says what pdfcer concluded about the text rather than that something
/// failed, because nothing did.
#[must_use]
pub const fn reflow_no_block() -> &'static str {
    "This text is not laid out as a paragraph — it is a single line or an isolated label — so \
     there is nothing to re-wrap."
}

/// ★★★ **Every way a reflow can decline, as ONE type** — `OPERATOR_REQUESTS.md`
/// **O127**, defect 3.
///
/// # Why this enum exists, when five `&'static str` functions already did
///
/// Because the five functions were being written to **the wrong slot**, and
/// nothing in the type system could say so. `app::dispatch::text` and
/// `app::actions::textstyle` both called `crate::app::actions::record_note`,
/// which the status bar renders under **`⚑ About your last edit:`** — and
/// `app::status::decline`'s own header forbids exactly that for a decline:
///
/// > *"an operator who reads 'About your last edit' after a gesture that did
/// > nothing has been told a small lie confidently."*
///
/// The operator's report is the plain consequence. He pressed Reflow, the shell
/// declined every time, wrote a correct sentence into a slot that reads as a
/// footnote about something *earlier*, truncated it to 45 % of the bar, and he
/// reported *"I haven't seen the reflow option actually work with anything when
/// I press it."* **It was answering him. In the wrong voice, in the wrong
/// place, in the wrong tense.**
///
/// ⇒ Routing every cause through one enum makes the channel a property of the
/// type: `Declined::Reflow` can only be shown by `decline::show`, which wears
/// `⊗` and means *nothing happened*. A sixth cause added tomorrow cannot pick
/// the wrong slot, because there is no longer a slot to pick.
///
/// # ★★ The two halves, and why both are here
///
/// | | decided by | variants |
/// |---|---|---|
/// | **before** the engine is called | the shell, from the caret | [`Self::NeedsCaret`], [`Self::NeedsExistingText`], [`Self::NoBlock`] |
/// | **from the engine's answer**, since 2026-09-05 | `Pass 251.0` | [`Self::PageAlreadyEdited`] — it moved out of this column when the shell's over-broad `edit_epoch != 0` forecast was deleted; the engine now refuses the case that mattered (a page carrying a non-empty appended content stream) by name, and this variant words that answer |
/// | **by** the engine | `pdfcer-core` | [`Self::PageSetChanged`], [`Self::Encrypted`], [`Self::CannotTrace`], [`Self::Other`] |
///
/// The engine half used to reach the operator as
/// `crate::text::status::edit_declined_by_engine` — nine words, no cause, no
/// remedy — because `funnel::vector_edit`'s error arm traces the detail and
/// shows the generic line. These four are what that generic line was standing
/// in for, and each names a different thing to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflowRefusal {
    /// No caret at all: the operator pressed Reflow with nothing composing.
    NeedsCaret,
    /// The caret is on bare page (`Anchor::Origin` or `Anchor::Box`), so it is
    /// placing NEW text and there is no paragraph on the page yet.
    NeedsExistingText,
    /// The caret is in a run the block recogniser does not group into a
    /// paragraph — a title-block cell, a dimension label, an isolated note.
    NoBlock,
    /// This session has already changed the document.
    ///
    /// ★★★ **This gate is load-bearing and it is NOT merely conservative.**
    /// See [`reflow_after_edit`]'s own note: `EditSession::reflow_block` plans
    /// from the **base** document and then writes the result into the page's
    /// first content object, *emptying every other one*. Text added this
    /// session lives in one of those other content objects, so a reflow that
    /// ran would **silently delete it**. The engine's own guard does not cover
    /// that case — it only refuses when the *first* content object was
    /// rewritten — so this forecast is the only thing standing between the
    /// operator and losing work he can see on the page.
    PageAlreadyEdited,
    /// The engine's page-set guard: a page was added, removed or reordered, and
    /// reflow's planner is indexed against the base document's pages.
    PageSetChanged,
    /// The document is encrypted, which reflow refuses outright.
    Encrypted,
    /// The engine could not trace the paragraph's lines back to the operators
    /// that drew them (`ReflowApplyError::NoProvenance`, or an extraction that
    /// failed), so it will not re-wrap what it cannot address.
    CannotTrace,
    /// Anything else the engine returned. Named rather than merged into
    /// [`Self::CannotTrace`], because *"pdfcer could not"* and *"pdfcer would
    /// not"* are different admissions and only one of them has a remedy.
    Other,
}

impl ReflowRefusal {
    /// The sentence for this cause.
    ///
    /// One function over the enum rather than one per variant, for
    /// [`refusal`]'s reason: a variant added without a sentence is a compile
    /// error rather than a control that declines silently.
    ///
    /// ★ The first four forward to the free functions that already existed and
    /// are already tested, so no sentence is written twice. What changed for
    /// them is the **channel**, not the words.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::NeedsCaret => reflow_needs_caret(),
            Self::NeedsExistingText => reflow_needs_existing_text(),
            Self::NoBlock => reflow_no_block(),
            Self::PageAlreadyEdited => reflow_after_edit(),
            // ★ Deliberately the same remedy as `PageAlreadyEdited` and
            // deliberately not the same sentence: the operator did something
            // different to get here, and a sentence that named the wrong cause
            // would send them looking for an edit they did not make.
            Self::PageSetChanged => {
                "Reflowing a paragraph needs the pages as they were when you opened the file, and \
                 pages have been added, removed or reordered since. Save this file and open it \
                 again, then reflow."
            }
            Self::Encrypted => {
                "This document is encrypted, so pdfcer cannot re-write its text. Remove the \
                 protection first, using Protect > Remove security."
            }
            Self::CannotTrace => {
                "pdfcer cannot tell which parts of the page drew these lines, so it will not \
                 re-wrap them — re-wrapping text it cannot address would move the wrong words."
            }
            Self::Other => {
                "pdfcer could not re-wrap this paragraph, and your document has not been changed."
            }
        }
    }
}

/// ★★★ **Why Enter did not make a new line in text that is already on the
/// page** — `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// The operator: *"can the enter key create new lines when we are editing or
/// creating text?"*
///
/// **Creating: yes, everywhere, as of this change.** A dragged box and a
/// clicked point both take a line break on Enter and commit on Ctrl+Enter.
///
/// **Editing text already on the page: no, and it is the FILE that says so.**
/// `EditSession::edit_text` replaces the string inside one show operator, and a
/// show operator cannot contain a line break — `\n` has no code in any of the
/// standard encodings, so the engine refuses it by name (`Refusal`,
/// `TargetAbsent`, character `'\n'`) rather than dropping it. A PDF has no
/// paragraph: each visible line is its own operator at its own absolute
/// position, so splitting a line in two is not an edit, it is authoring a
/// second line somewhere.
///
/// ★★ So this is a **decline with a route**, not an apology. It says what
/// cannot happen, why, and the two things that can: finish the edit, or place
/// new text in a box that wraps. Silence here — which is what the shell did
/// before, by quietly committing instead — is the founding defect class of this
/// project: the key was pressed, something else happened, and nothing said so.
#[must_use]
pub const fn enter_cannot_split_existing_text() -> &'static str {
    "Text already on the page is drawn one line at a time, so a line cannot be split in two \
     here. Press Ctrl+Enter to finish this edit, or use Add text and drag a box for text that \
     wraps."
}

/// The disclosure owed when a **clicked** text draft turns out to be
/// multi-line.
///
/// # ★★★ Rule 4: an inference the operator cannot see owes an off-canvas report
///
/// A click has no extent, so a point text has no width to wrap against — which
/// is why dragging a box was the multi-line gesture in the first place. Once
/// Enter inserts a line break at a clicked caret, the commit needs a box
/// anyway, and this shell derives one: from the click across to the right edge
/// of the sheet, and down to the bottom.
///
/// That width is **not invented** — it is the page's own crop box, which is a
/// fact about the operator's document rather than a number this shell chose —
/// but it is still an inference, and the operator cannot see a rectangle that
/// is not drawn. What they *can* see is the consequence: a line long enough
/// will wrap at the sheet edge rather than running off it.
///
/// ★ It is shown once, at the commit, and not while typing: the draft is still
/// a draft until then, and a sentence about how something will be placed is
/// noise until it has been placed.
#[must_use]
pub const fn point_text_became_a_block() -> &'static str {
    "Your text has more than one line, so it was placed as a block: it starts where you clicked \
     and wraps at the right-hand edge of the sheet. Drag a box with Add text to choose your own \
     width."
}

// ===========================================================================
// ★★★ Why an edit the operator committed did not happen — O140
// ===========================================================================

/// ★★★ **Why a committed text edit was refused** — `OPERATOR_REQUESTS.md`
/// **O140**, and the sentence that replaces `edit_declined_by_engine`'s nine
/// cause-free words for this one verb.
///
/// The operator, 2026-09-05, on his own quote:
///
/// > *"on page 2 there is a spelling mistake — clien instead of client. if I
/// > try to edit the edit is not accepted. **the lines I added below `price)`
/// > are editable, but everything else that existed when I got the pdf is
/// > not.**"*
///
/// He met a caret that took his keystrokes, a Ctrl+Enter that did nothing, and
/// a status line reading *"That change was refused, and the document is
/// unchanged."* — which is true, and tells him nothing he could act on. His own
/// second sentence is a better diagnosis than anything the program gave him.
///
/// # ★★★ Where the categories come from, and why not from here
///
/// `crate::text::status::edit_declined_by_engine`'s documentation says, in as
/// many words, that it *"is written to be **deleted**… The day `EditError`
/// gains a `kind()`, this becomes four sentences that name the four buckets."*
///
/// **That day arrived and nothing here had noticed.** `pdfcer-core` shipped
/// `text_edit::RefusalKind` — four variants, deliberately **not**
/// `#[non_exhaustive]` so a front end may match it exhaustively and have the
/// compiler prove the sentences are complete — at revision `b1033ab`, in direct
/// answer to this project's 2026-09-04 request. It sat unconsumed because both
/// gates that watch the engine are keyed on `EditSession`'s **verbs**: a new
/// *type* is invisible to `check-verb-coverage.sh` and to
/// `check-engine-backlog.sh` alike.
///
/// ⇒ Four of the five variants below are `RefusalKind`'s, one-for-one. Nothing
/// here re-derives the engine's reasoning, matches on its error variants or
/// greps its `Display` prose — the three things that entry forbids. The mapping
/// lives beside the errors, in their crate, and moves with them.
///
/// # ★★★ The fifth variant, and why the shell is entitled to it
///
/// [`Self::SplitAcrossPieces`] is **not** a `RefusalKind`. It is a fact this
/// shell measured before it asked, and it is the one his document actually
/// hits.
///
/// `canvas::textedit::plan` calls `pin::spans_one_operator` and traces the
/// answer as `edit-text-pin … one_operator=…`. On his file that is **false**:
/// the producer emitted **one show operator per glyph** — the page-2 content
/// stream is a run of `(\x00\x17) Tj  16.07 0 Td  (\x00\x11) Tj  8.03 0 Td …`,
/// one two-byte code and its own `Td` per letter — so no single operator holds
/// more than one character. The shell therefore sends the reconstructed `find`
/// rather than the whole-operator form (see `plan`'s own note: the
/// whole-operator form on a split run would replace one fragment's text with
/// the whole replacement and leave the others painting their old glyphs —
/// *visible corruption reported as success*), and a 36-character `find` cannot
/// match inside a one-character operator. The engine answers
/// `EditError::NoMatch`, whose `RefusalKind` is `RefusalKind::NotFound`.
///
/// ★★ **`NotFound` is the honest engine answer and the wrong operator
/// sentence.** *"pdfcer couldn't find what the edit named"* is
/// indistinguishable from *"your search string is wrong"* — and he did not type
/// a search string; he clicked a word he can see. The shell knows the piece of
/// information that turns that answer into a true one, and no other layer does:
/// **it placed the caret on that run itself, from its own extraction, and it
/// knows the run is split.** So the join is made here, at the only place both
/// facts exist.
///
/// # ★★★ What this variant is NOT, and the forecast that was falsified
///
/// The feature request filed against the engine on the same day names the cause
/// as `Identity-H` — *"the document's original faces are `Type0/CIDFontType2`
/// `Identity-H`, `verdict=blocked-identity`… the only character information is
/// a one-way `/ToUnicode`"* — and asks for that map to be inverted.
///
/// **Measured, and it is not the cause.** `Pass 29.0` made composite runs
/// editable whenever their `/ToUnicode` inverts, and on his own file, with the
/// engine's own CLI:
///
/// ```text
/// pdfcer edit-text --page 2 --find "n" --replace "t"
///   → base_font=AAAAAA+Arimo-Bold  …  OK
/// ```
///
/// — an `Identity-H`, `blocked-identity`, embedded `CIDFontType2` face, edited
/// end to end, read back out of the saved file. A **one-character** find
/// matches inside a one-character operator; a five-character one cannot. The
/// font was never the obstacle.
///
/// ⇒ **So this shell must not forecast on the font.** A guard reading
/// *"`Identity-H` ⇒ refuse"* would withhold the caret on text pdfcer can edit,
/// on every document a modern producer makes, silently — which is
/// `Refusal::InsideForm`'s episode repeated with a different predicate. The
/// falsifying fixture is `pdfcer-core`'s own
/// `fixtures/synthetic/text/composite-editable.pdf`: identical `list-fonts`
/// verdict to every face in his document, and
/// `an_invertible_composite_run_is_editable_end_to_end` asserts it edits.
///
/// # ★★ Why the caret is still OFFERED, against R9's first reading
///
/// R9 says a control that fails on press is worse than one that is not there,
/// and the obvious application is *"do not put a caret on a split run"*. It is
/// the wrong application here, for a reason that is a property of the caret
/// rather than of this refusal: **the caret anchors more than the replace.**
/// `edit.reflow_block` resolves its operand from `Anchor::Run`, and
/// `format_text` restyles through the same pin without ever needing a `find` —
/// both work on a split run. Withdrawing the anchor because one of the three
/// verbs it feeds is certain to refuse would take reflow and restyling away
/// from every run in this document to save one wasted keystroke in it.
///
/// ⇒ The caret stays; the **silence** goes. That is the whole of O140.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditRefusal {
    /// ★★★ **The line was written one piece at a time and the change spans
    /// more than one piece** — the operator's own document, and the case
    /// `RefusalKind` cannot name because it is not the engine's to see.
    ///
    /// Raised only when the engine answered `RefusalKind::NotFound` *and*
    /// `pin::spans_one_operator` had already said `false`. Both halves are
    /// required: a split run whose refusal is a font refusal gets the font
    /// sentence, because that is what actually stopped it.
    SplitAcrossPieces,
    /// `RefusalKind::UnsupportedFont` — the code↔glyph relation is
    /// unrecoverable, or a substitute could not cover the text.
    ///
    /// ★★ Since O141 this is the arm for the refusals that name **no**
    /// character: R-INV-2 (a symbolic font whose encoding lives inside a
    /// program `pdfcer-core` does not parse), R-INV-3 and R-INV-4. Nothing
    /// about the run's *content* stopped those, so no other face would help and
    /// no offer is made. See [`Self::FontLacksTheCharacter`] for its twin.
    UnsupportedFont,
    /// ★★★ **The font carries the letters the page prints and not the one the
    /// operator just typed** — `OPERATOR_REQUESTS.md` **O141**, 2026-09-05.
    ///
    /// > *"if the character isn't available in a pdf are we able to change to a
    /// > different font?"*
    ///
    /// # Why this is a SIXTH sentence and not a rewording of the fifth
    ///
    /// The engine collapses both into `RefusalKind::UnsupportedFont` —
    /// `EditError::Refused(_)` maps there wholesale — and from the shell's chair
    /// they are two different situations with two different answers:
    ///
    /// | what happened | what the operator can do |
    /// |---|---|
    /// | [`Self::UnsupportedFont`] — R-INV-2/3/4, the font's code↔glyph relation is unrecoverable | nothing this build offers |
    /// | this — R-INV-1/6/7/8, the relation is fine and this **character** is not in it | **change the face**, which pdfcer can already do |
    ///
    /// Wording them the same would tell an operator who could fix their document
    /// in two clicks that pdfcer cannot edit their text — a confident wrong
    /// reason, which `RefusalKind`'s own header calls strictly worse than the
    /// silence it replaced.
    ///
    /// # ★★ The discriminant is the engine's, and it is a DATUM rather than a
    /// second taxonomy
    ///
    /// `crate::app::status::decline::record_edit_text_refusal` reads
    /// `Refusal::character` off `EditError::Refused`. That is not this crate
    /// re-deriving the engine's reasoning — the category still comes from
    /// `RefusalKind` and nothing here inspects a trigger id or greps a `Display`
    /// string. It reads **one field the coarse kind structurally cannot carry**,
    /// on the identical licence `one_operator` has: the sentence needs a fact
    /// the category does not hold.
    ///
    /// # ★ The character is NOT in this sentence, and that is a layout fact
    ///
    /// `Declined::line` returns `&'static str`, so the status bar's `⊗` slot
    /// cannot interpolate a runtime character, and `disclosure_line` truncates
    /// it to 45 % of the bar besides. The named character and the chooser that
    /// answers it are in `panels::properties::refusedchar`, which is a panel and
    /// can hold both. This sentence's job is to name the obstacle and say where
    /// the answer is.
    FontLacksTheCharacter,
    /// `RefusalKind::StructureFrozen` — encryption, an enforced certification
    /// signature, or a suppressed object set.
    DocumentProtected,
    /// `RefusalKind::NotFound` on a run the shell had measured as a **single**
    /// operator, so the split explanation is unavailable and the honest reading
    /// is that the page moved under the caret.
    TextMovedAway,
    /// `RefusalKind::Other` — an invalid parameter, an unbuilt combination, a
    /// parse or save failure. The engine's own words are on the trace.
    Unstated,
}

impl EditRefusal {
    /// **Classify one refused text edit**, from the engine's coarse kind and
    /// the one thing the shell knows that the engine cannot.
    ///
    /// # ★★★ The order of the arms is the design
    ///
    /// The engine's category wins wherever it has one, and the shell's fact is
    /// consulted **only** inside `NotFound` — the single bucket where the
    /// engine's answer is true and unusable at the same time.
    ///
    /// That ordering is what keeps this from being a second taxonomy. A split
    /// run whose edit was stopped by a font gets
    /// [`Self::UnsupportedFont`], not [`Self::SplitAcrossPieces`], because the
    /// font is what actually stopped it and the split is merely also true. The
    /// failure mode of the opposite ordering is precisely the one
    /// `RefusalKind`'s own header warns about: *telling the operator the wrong
    /// reason, which is strictly worse than the silence it replaced.*
    ///
    /// # ★★ `one_operator` is a measurement, and `true` is its "not measured"
    ///
    /// See [`crate::canvas::textedit::Plan::one_operator`]. It is `true` when
    /// the plan could not read provenance at all, so an unmeasured run falls to
    /// [`Self::TextMovedAway`] — a sentence that is honest about a page that
    /// moved and never claims a structure this shell did not observe.
    /// # ★★★ `names_a_character` is the second fact the category cannot hold
    ///
    /// `OPERATOR_REQUESTS.md` O141. `true` when the engine's refusal carried a
    /// `Refusal::character` — i.e. the inverse-encoding gate stopped on **one
    /// scalar it has no code for** (R-INV-1, 6, 7, 8) rather than on the font's
    /// whole code↔glyph relation being unreadable (R-INV-2, 3, 4). Both arrive
    /// as `RefusalKind::UnsupportedFont`, and only the first has a remedy.
    ///
    /// ★ It is consulted **only** inside `UnsupportedFont`, on the same rule the
    /// `NotFound` split follows: the engine's category wins wherever it has one,
    /// and a shell-side fact is allowed to sharpen exactly the bucket where the
    /// engine's answer is true and unusable at the same time.
    #[must_use]
    pub const fn of(
        kind: pdfcer_core::text_edit::RefusalKind,
        one_operator: bool,
        names_a_character: bool,
    ) -> Self {
        use pdfcer_core::text_edit::RefusalKind as K;
        match kind {
            K::UnsupportedFont if names_a_character => Self::FontLacksTheCharacter,
            K::UnsupportedFont => Self::UnsupportedFont,
            K::StructureFrozen => Self::DocumentProtected,
            K::NotFound if !one_operator => Self::SplitAcrossPieces,
            K::NotFound => Self::TextMovedAway,
            K::Other => Self::Unstated,
        }
    }

    /// The sentence for this cause.
    ///
    /// One function over the enum rather than one per variant, for
    /// [`refusal`]'s and [`ReflowRefusal::line`]'s reason: a variant added
    /// without a sentence is a compile error rather than a commit that refuses
    /// silently.
    ///
    /// # ★★ Every sentence is FRONT-LOADED, and that is a layout fact
    ///
    /// `app::status::disclosure::disclosure_line` draws the decline with
    /// `.truncate()` and hangs the whole text on hover. So the first clause is
    /// what most operators read, and it must carry the claim that matters:
    /// **pdfcer cannot**, not *you did something wrong*. The cause, the
    /// contrast and the remedy follow it in that order.
    ///
    /// # ★ Two of them name his contrast explicitly
    ///
    /// He noticed it before the program told him: *"the lines I added below
    /// `price)` are editable, but everything else that existed when I got the
    /// pdf is not."* A sentence that explains the refusal and ignores the
    /// contrast reads as evasive, because he has already worked out that the
    /// two cases differ and is waiting to hear why.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            // ★★★ The remedy clause is the one this project checked before
            // writing. `pdfcer-core` can edit a piece: `--find "n" --replace
            // "t"` succeeded on this very run. What it cannot do is address ONE
            // piece from a caret — the shell has no gesture that names a single
            // glyph operator, and `EditRequest` carries one pin per request. So
            // there is no sequence of clicks that gets him his `t` today, and
            // the sentence says exactly that rather than inventing one.
            //
            // ⚠ It deliberately does NOT say "delete it and retype it". That
            // was checked too: `add_text` writes the engine's bundled
            // Helvetica, so retyping a line of AbrilFatface or Arimo replaces
            // his typography with a substitute at a position he would have to
            // find by eye — and there is no verb here that deletes a text run
            // in the first place. Offering it would be a workaround that does
            // not exist.
            Self::SplitAcrossPieces => {
                "pdfcer cannot change these words. The program that made this file wrote the line \
                 one letter at a time, and pdfcer rewrites a whole piece of text at once — so \
                 there is no piece here that holds the word you are correcting. Text you added \
                 with pdfcer is written a line at a time, which is why those lines do edit. Your \
                 document is unchanged; this limit is on the list to fix."
            }
            Self::UnsupportedFont => {
                "pdfcer cannot write new letters into this text. Its font records what each shape \
                 looks like but not which letter it is, so pdfcer cannot spell a letter that is \
                 not already there. Text you added with pdfcer uses a font it can spell in, which \
                 is why those lines do edit. Your document is unchanged."
            }
            // ★★★ O141. The one decline in this catalog whose whole purpose is
            // to hand the operator on to a control, so the last clause names
            // that control and the panel it is in.
            //
            // ⚠ It deliberately does NOT name the character. `Declined::line`
            // is `&'static str` and `disclosure_line` truncates the slot to
            // 45 % of the bar; a sentence that promised to name a character and
            // then could not would be worse than one that sends the operator
            // to the surface that does. `panels::properties::refusedchar` names
            // it, offers the faces, and carries rule 4's disclosure.
            //
            // ★ "the letters your page prints" rather than "the subset": the
            // operator asked this question without the word, and O141's own
            // framing is that they should not have to learn it.
            Self::FontLacksTheCharacter => {
                "pdfcer cannot type that character into this text. The font here was built with \
                 only the letters your page already prints, and pdfcer cannot add one to a font \
                 that is already inside the file. Your document is unchanged — open Properties, \
                 which names the character and offers the faces that can type it."
            }
            Self::DocumentProtected => {
                "This document's protection does not allow its text to be changed, so pdfcer left \
                 it alone. If you have the password, remove the protection first with Protect > \
                 Remove security, then edit."
            }
            Self::TextMovedAway => {
                "pdfcer could not find the text this edit named — the page has moved on since the \
                 cursor was placed there. Click in the words again and make the change a second \
                 time. Your document is unchanged."
            }
            // ★ The one variant with no cause to name, and it is deliberately
            // the SAME sentence the whole verb used to show. Where the engine
            // says only "other", inventing a cause here would be the "second
            // copy of their taxonomy that drifts and then tells the operator
            // the WRONG reason" that `RefusalKind` exists to prevent.
            Self::Unstated => crate::text::status::edit_declined_by_engine(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every variant**, so a sixth added without a sentence, without an
    /// obstacle in its first clause, or promising a remedy this build does not
    /// have, goes red rather than shipping.
    ///
    /// ★ A hand-written list inside a completeness sweep is a shape this
    /// project has been bitten by three times, and it is tolerable here for one
    /// reason: `EditRefusal::line`'s `match` is exhaustive, so a new variant is
    /// a **compile error** in the catalog before it can be a gap in this list.
    /// The list is a convenience over a closed set, not the closure itself.
    const EVERY: [EditRefusal; 6] = [
        EditRefusal::SplitAcrossPieces,
        EditRefusal::UnsupportedFont,
        EditRefusal::FontLacksTheCharacter,
        EditRefusal::DocumentProtected,
        EditRefusal::TextMovedAway,
        EditRefusal::Unstated,
    ];

    /// **No sentence opens with the operator.** His report is *"the edit is not
    /// accepted"*, and a sentence beginning with what he did reads as a
    /// correction of him rather than an admission by the program.
    ///
    /// ★ Checked on the first clause because that is the part the status bar
    /// actually shows — `disclosure_line` truncates and hangs the rest on
    /// hover.
    #[test]
    fn no_edit_refusal_opens_by_naming_what_the_operator_did() {
        for why in EVERY {
            let first = why.line().split('.').next().unwrap_or_default().to_owned();
            assert!(
                !first.starts_with("You ") && !first.starts_with("Your "),
                "a decline that opens with the operator reads as a correction of them: {first:?}"
            );
        }
    }

    /// ★★★ **Every sentence this shell WORDED names the obstacle in its first
    /// clause** — and the exception proves the rule rather than weakening it.
    ///
    /// The first version of this test asserted it over all five variants and
    /// went red on [`EditRefusal::Unstated`], whose line is
    /// *"That change was refused, and the document is unchanged."* — the
    /// pre-O140 sentence, deliberately reused byte for byte.
    ///
    /// ⇒ **The test was asserting more than the design promises.** `Unstated`
    /// is the arm where the engine says only *other*, and its whole point is
    /// that this shell has nothing to add; naming an obstacle there would be
    /// the invented cause `RefusalKind` exists to prevent. So the obligation
    /// belongs to the four categorised sentences, and is stated over exactly
    /// those — with the fifth pinned separately, by
    /// [`Self::the_uncategorised_case_keeps_the_sentence_it_always_had`], to
    /// the string it is supposed to be.
    ///
    /// [`Self::the_uncategorised_case_keeps_the_sentence_it_always_had`]: the_uncategorised_case_keeps_the_sentence_it_always_had
    #[test]
    fn every_categorised_refusal_names_the_obstacle_in_its_first_clause() {
        for why in [
            EditRefusal::SplitAcrossPieces,
            EditRefusal::UnsupportedFont,
            EditRefusal::FontLacksTheCharacter,
            EditRefusal::DocumentProtected,
            EditRefusal::TextMovedAway,
        ] {
            let first = why.line().split('.').next().unwrap_or_default().to_owned();
            assert!(
                first.contains("pdfcer") || first.contains("document's protection"),
                "the visible clause must name the program or the document as the obstacle: \
                 {first:?}"
            );
        }
    }

    /// ★★★ **Every mapping from the engine's four buckets, exhaustively.**
    ///
    /// The compiler already proves `EditRefusal::of` handles every
    /// `RefusalKind` — that is what `RefusalKind` not being `#[non_exhaustive]`
    /// buys, and it is why the engine committed to it. What the compiler cannot
    /// prove is that the **`NotFound` split is wired the right way round**, and
    /// getting it backwards is the failure mode that matters most here: the
    /// operator would be told his page had moved when his line is written one
    /// letter at a time, or the reverse.
    #[test]
    fn the_engines_four_buckets_map_to_six_sentences_and_two_of_them_split() {
        use pdfcer_core::text_edit::RefusalKind as K;
        // The two that ignore both shell-side facts, asserted every way so a
        // stray condition added to either goes red.
        for one in [true, false] {
            for named in [true, false] {
                assert_eq!(
                    EditRefusal::of(K::StructureFrozen, one, named),
                    EditRefusal::DocumentProtected
                );
                assert_eq!(EditRefusal::of(K::Other, one, named), EditRefusal::Unstated);
            }
        }
        // ★ And the two that do not. `one_operator == false` means the run is
        // written in several pieces, which is the operator's own document.
        assert_eq!(
            EditRefusal::of(K::NotFound, false, false),
            EditRefusal::SplitAcrossPieces,
            "a split run's NotFound is the split, not a page that moved"
        );
        assert_eq!(
            EditRefusal::of(K::NotFound, true, false),
            EditRefusal::TextMovedAway,
            "a single-operator run's NotFound has no split to blame, and claiming one \
             would be a structure this shell did not observe"
        );
    }

    /// ★★★ **The font bucket splits on whether the engine named a character**
    /// — `OPERATOR_REQUESTS.md` O141, and it is the mapping a wrong build gets
    /// backwards.
    ///
    /// `EditError::Refused(_)` maps to `RefusalKind::UnsupportedFont`
    /// **wholesale**, so R-INV-1 (*"this font has no glyph for '€'"* — a
    /// character the subset does not carry, and a face swap fixes it) and
    /// R-INV-2 (*"its code↔glyph relation lives inside the embedded program,
    /// which pdfcer-core does not parse"* — and no face swap helps, because the
    /// run cannot be re-encoded at all) arrive as the same category.
    ///
    /// Getting it round the wrong way costs in both directions: an operator two
    /// clicks from their `€` is told pdfcer cannot edit the text, or an operator
    /// with an unreadable font is sent to a chooser that will refuse every row.
    /// `Refusal::character` is `Some` for exactly the first family and `None`
    /// for the second, which is why the datum is read rather than the trigger id.
    ///
    /// ★ Asserted across `one_operator` as well, because the run's provenance
    /// has nothing to do with the font's repertoire and a condition that crept
    /// in would make the sentence depend on how the producer emitted the line.
    #[test]
    fn a_font_refusal_that_names_a_character_is_the_one_with_a_way_out() {
        use pdfcer_core::text_edit::RefusalKind as K;
        for one in [true, false] {
            assert_eq!(
                EditRefusal::of(K::UnsupportedFont, one, true),
                EditRefusal::FontLacksTheCharacter,
                "a refusal naming one character is a repertoire fact, and changing the face \
                 is the remedy pdfcer already has"
            );
            assert_eq!(
                EditRefusal::of(K::UnsupportedFont, one, false),
                EditRefusal::UnsupportedFont,
                "a refusal naming NO character is about the font's whole code-to-glyph \
                 relation, and no other face makes this run re-encodable"
            );
        }
    }

    /// ★★ **The sentence that has a way out says where the way out is**, and it
    /// is the only decline in this catalog that hands the operator to a control.
    ///
    /// Without the last clause O141 is answered with a better diagnosis and no
    /// route — which is exactly the state O141 was filed about: *"That last
    /// clause is the answer to your question, and it is buried in an error
    /// message."*
    #[test]
    fn the_missing_character_sentence_names_the_surface_that_answers_it() {
        let s = EditRefusal::FontLacksTheCharacter.line();
        assert!(
            s.contains("Properties"),
            "a decline with a remedy must name where the remedy is: {s:?}"
        );
        assert!(
            s.contains("faces that can type it"),
            "and it must say what the operator will find there, or the route reads as a \
             suggestion to go and look: {s:?}"
        );
        assert!(
            s.contains("unchanged"),
            "the first question after a refused edit is always whether it took: {s:?}"
        );
        // ★ And it must NOT claim to name the character, because it cannot:
        // `Declined::line` is `&'static str`. A sentence promising a character
        // it does not carry is the shape of falsehood this catalog's tests are
        // for.
        assert!(
            !s.contains('“') && !s.contains('”'),
            "the quoted character belongs to the panel, which can interpolate one: {s:?}"
        );
    }

    /// ★★★ **The two font-shaped refusals answer his contrast.**
    ///
    /// *"the lines I added below `price)` are editable, but everything else
    /// that existed when I got the pdf is not."* He had the diagnosis before
    /// the program did. A sentence that explains the refusal and says nothing
    /// about why his own lines behave differently leaves the one question he
    /// actually asked unanswered.
    #[test]
    fn the_two_content_refusals_explain_why_his_own_added_lines_edit() {
        for why in [EditRefusal::SplitAcrossPieces, EditRefusal::UnsupportedFont] {
            let s = why.line();
            assert!(
                s.contains("added with pdfcer"),
                "his contrast is unaddressed: {s:?}"
            );
            assert!(s.contains("do edit"), "and it must say which way: {s:?}");
        }
    }

    /// **No sentence promises a workaround.**
    ///
    /// ⚠ Verified before it was written, not assumed: there is no verb in this
    /// shell that deletes a text run, and `add_text` writes the engine's
    /// bundled Helvetica — so *"delete it and retype it"* would cost the
    /// operator his typography and his position, and is not offered. A remedy
    /// named in a decline is a promise, and a promise that does not resolve is
    /// worse than the silence it replaced.
    #[test]
    fn no_edit_refusal_offers_a_remedy_this_build_does_not_have() {
        for why in EVERY {
            let s = why.line().to_ascii_lowercase();
            assert!(
                !s.contains("retype") && !s.contains("delete it"),
                "neither deletion nor retyping is available, so neither may be suggested: {s:?}"
            );
        }
    }

    /// ★★ **The split-run sentence says the document is intact and does not
    /// pretend there is something to do.**
    ///
    /// The failure this ends is not *"I was not told why"* — it is *"I do not
    /// know whether it took"*. `edit_declined_by_engine`'s own documentation
    /// carries the argument; this variant inherits the obligation.
    #[test]
    fn the_split_run_sentence_says_the_document_is_unchanged() {
        let s = EditRefusal::SplitAcrossPieces.line();
        assert!(s.contains("unchanged"), "{s:?}");
        assert!(
            s.contains("one letter at a time"),
            "the cause has to be in his terms, not in show operators: {s:?}"
        );
    }

    /// **`Unstated` is the old sentence, unchanged.** Where the engine says
    /// only *other*, this shell has nothing to add — and adding something would
    /// be the invented cause `RefusalKind` exists to prevent.
    #[test]
    fn the_uncategorised_case_keeps_the_sentence_it_always_had() {
        assert_eq!(
            EditRefusal::Unstated.line(),
            crate::text::status::edit_declined_by_engine()
        );
    }

    /// ★ **Every refusal has a sentence, and none of them is empty.**
    ///
    /// The whole point of the module: the old shell's answer to the cross-run
    /// case was no sentence at all.
    #[test]
    fn every_refusal_says_something() {
        for r in [Refusal::NoRun, Refusal::NoText] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }

    /// ★★ **The multi-run note says what pdfcer WILL DO, not what it refuses.**
    ///
    /// Its ancestor asserted `s.contains("Click directly on the word")` — advice
    /// that could not work, because the refusal was about the *line* and not
    /// about where on it the operator clicked. The property that replaces it is
    /// the one that matters now: the sentence must name **the consequence the
    /// operator cannot otherwise see**, which is that the neighbouring pieces
    /// will not move.
    #[test]
    fn the_multi_run_note_says_what_happens_rather_than_refusing() {
        let s = shares_the_line_note();
        assert!(
            s.contains("stay exactly where they are"),
            "the note must say what happens to the pieces the operator did NOT edit — that is \
             the whole of what they cannot see: {s:?}"
        );
        assert!(
            s.contains("overlap"),
            "and it must name the cost, because a longer replacement growing into the next cell \
             is the one thing this decision can produce that the operator would call a bug: {s:?}"
        );
        // …and it does not use the word the engine uses, which names nothing
        // the operator can see on their page.
        assert!(!s.contains("run"), "'run' is a PDF term, not an operator's");
    }

    /// ★ **Sharing the line PINS**, and that is the property the whole fix
    /// rests on.
    ///
    /// If this reason ever reflowed, editing one cell of a SolidWorks parts
    /// table would slide every cell after it sideways — content the operator did
    /// not touch, moved by an edit that did not mention it. Asserted here rather
    /// than only in `disposition`'s own tests because this module is where the
    /// sentence promising it lives, and a sentence and a behaviour that disagree
    /// is worse than either alone.
    #[test]
    fn sharing_the_line_pins_the_neighbours() {
        assert!(Reason::SharesTheLine.pins_the_tail());
        let s = pinned_tail_disclosure(Reason::SharesTheLine);
        assert!(s.contains("several separate pieces"), "{s:?}");
    }

    /// ★ **Each pinning reason gets its own explanation.**
    ///
    /// A single generic sentence would be the cheaper implementation and would
    /// be wrong for both cases: "right-aligned" is something the operator's
    /// document is, and "rotated" is something they can see, and the remedy
    /// differs.
    #[test]
    fn each_pinning_reason_explains_itself_differently() {
        let rotated = pinned_tail_disclosure(Reason::Rotated);
        let right = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Right));
        let centre = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Center));
        assert!(rotated.contains("rotated"));
        assert!(right.contains("right-aligned"));
        assert!(centre.contains("centred"));
        assert_ne!(rotated, right);
        assert_ne!(right, centre);
    }

    /// ★★★ **Every reflow cause has its own sentence, and no two are the
    /// same.**
    ///
    /// The property the enum exists for. Before O127 the four shell-side causes
    /// went to one channel and the four engine-side ones collapsed into nine
    /// generic words — so the operator could press Reflow for four genuinely
    /// different reasons and be told the same nothing. A duplicate here would
    /// be that failure re-arriving with a type in front of it.
    #[test]
    fn every_reflow_cause_says_something_of_its_own() {
        let all = [
            ReflowRefusal::NeedsCaret,
            ReflowRefusal::NeedsExistingText,
            ReflowRefusal::NoBlock,
            ReflowRefusal::PageAlreadyEdited,
            ReflowRefusal::PageSetChanged,
            ReflowRefusal::Encrypted,
            ReflowRefusal::CannotTrace,
            ReflowRefusal::Other,
        ];
        for why in all {
            let s = why.line();
            assert!(s.len() > 40, "{why:?} needs a real sentence, got {s:?}");
            assert!(s.ends_with('.'), "{why:?} is prose: {s:?}");
        }
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(
                    a.line(),
                    b.line(),
                    "{a:?} and {b:?} say the same thing, so one of the two causes is \
                     unreportable and the operator cannot tell which happened"
                );
            }
        }
    }

    /// ★★ **The two "save and reopen" causes both carry the remedy**, and it is
    /// the whole of what the operator has to do.
    ///
    /// A refusal naming a cause with no route is half a sentence — the rule
    /// `text::embed`'s blocker rows already follow — and *"save this file and
    /// open it again"* is not guessable from *"cannot reflow"*.
    #[test]
    fn the_two_stale_plan_causes_name_the_remedy() {
        for why in [
            ReflowRefusal::PageAlreadyEdited,
            ReflowRefusal::PageSetChanged,
        ] {
            let s = why.line();
            assert!(
                s.contains("open it again"),
                "{why:?} must say what to do, not only what went wrong: {s:?}"
            );
        }
    }

    /// ★★★ **Enter's refusal names the remedy, and names BOTH halves of it.**
    ///
    /// The sentence has to carry the keyboard route as well as the gesture
    /// route, because O127's brief is explicit that commit must not be reachable
    /// only by mouse: an operator told *"use Add text"* and nothing else has
    /// been given a way to place text and no way to finish the edit they are
    /// already in.
    #[test]
    fn the_enter_refusal_offers_a_keyboard_route_and_a_gesture_route() {
        let s = enter_cannot_split_existing_text();
        assert!(
            s.contains("Ctrl+Enter"),
            "commit must be reachable from the keyboard, and the sentence is where the \
             operator learns the chord: {s:?}"
        );
        assert!(
            s.contains("drag a box"),
            "and the way to get a line break at all is a box, which is the FILE's rule \
             rather than a preference: {s:?}"
        );
    }

    /// ★★ **The point-text disclosure says where the width came from.**
    ///
    /// Rule 4's obligation, and the reason the sentence is longer than *"placed
    /// as a block"*: the operator can see two lines of text and cannot see the
    /// rectangle they were laid into, so the one fact they need is what decides
    /// where a long line will break.
    #[test]
    fn the_point_text_disclosure_names_the_edge_it_wraps_at() {
        let s = point_text_became_a_block();
        assert!(s.contains("edge of the sheet"), "{s:?}");
        assert!(
            s.contains("Drag a box"),
            "and it offers the gesture that puts the width back in the operator's hands: {s:?}"
        );
    }

    /// **The disclosure warns about the cost it exists to disclose.** Without
    /// the overlap sentence this would be a note about an internal choice
    /// rather than a warning the operator can act on.
    #[test]
    fn the_disclosure_names_the_cost_and_not_just_the_choice() {
        let s = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Right));
        assert!(s.contains("overlap"), "the cost of a pin is an overlap");
        assert!(
            s.contains("before saving"),
            "and there is a moment to check"
        );
    }
}
