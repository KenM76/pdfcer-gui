//! # `text::textedit` — every sentence the text-editing tool shows
//!
//!
//! ## Two things here are load-bearing rather than cosmetic
//!
//! **[`shares_the_line_note`] is a DISCLOSURE, and it was a refusal until
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/textedit.md`.

use crate::editmodel::disposition::Reason;
use crate::editmodel::refusal::Refusal;
use pdfcer_core::text_edit::BlockAlignment;

// ===========================================================================
// The commit-refusal family, re-exported
// ===========================================================================
//
pub use super::editrefusal::{
    EditRefusal, RefusedCharacter, font_has_two_glyphs_for, font_lacks_the_character,
    run_cannot_take,
};

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
            "pdfcer cannot read any text on this page. If it is a scan, File > Recognise text… \
             reads its words so they can be found and copied."
        }
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
        // What replaces it is a genuinely different fact and therefore needs
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
        //
        // The three obligations `Refusal::NoAnchor`'s sentence set are met the
        // same way, and the second is the hard one here:
        //
        // 1. **It is not his mistake.** He clicked on real text and it is real
        //    text, still on his page, still printing.
        // 2. **It says what is different about this text**, without the word
        //    *encoding*, without *subset*, and without a clause number. What is
        //    true and sayable is that pdfcer can read this text but cannot
        //    write in the alphabet it was drawn in — which is also exactly why
        //    the rest of the document may edit normally, and that contrast is
        //    the half he works out for himself if nobody says it.
        // 3. **It says what he can do instead**, and both routes are real:
        //    `Add text` writes the engine's bundled face over the top, and
        //    Properties' face chooser is the same control the character-level
        //    refusals send him to. Neither is invented — R9's rule for a
        //    sentence: a remedy named is a claim about the build.
        //
        // ⚠ It deliberately does not say *"this font has no usable
        // encoding"*. That is the engine's `RunRepertoire::reason`, it is on
        // the trace, and it belongs there.
        Refusal::NoUsableEncoding => {
            "pdfcer can read this text but cannot write in it — the font it is drawn with \
             does not spell any letter pdfcer could put back. Other text in this document may \
             still edit normally. Use Add text to write over it, or open Properties to give \
             this line a face pdfcer can type in."
        }
        // It promises no editing after recognition: recognised words are
        // invisible text behind the picture, and typing into them would not
        // change what prints.
        Refusal::PictureOfText => {
            "This page is a picture of text, not text, so there is nothing here to type into. \
             Recognise text reads its words so they can be found and copied."
        }
    }
}

/// The multi-run **disclosure** — what `spans_runs()` used to refuse.
#[must_use]
pub const fn shares_the_line_note() -> &'static str {
    "This line is drawn as several separate pieces. You are editing the piece you clicked; the \
     pieces beside it will stay exactly where they are, so a longer replacement may overlap \
     them."
}

/// The disclosure appended when the edit pinned the text after it.
#[must_use]
pub fn pinned_tail_disclosure(reason: Reason) -> String {
    let because = match reason {
        Reason::Rotated => {
            "this text is rotated, so moving what follows it sideways would move it the wrong way"
        }
        // The commonest reason on this operator's documents by a wide margin,
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

/// Why reflow declined on a page carrying text this session ADDED.
#[must_use]
pub const fn reflow_after_edit() -> &'static str {
    "You added text to this page in this session, and re-wrapping a paragraph now would \
     drop it, so pdfcer refuses. Save this file and open it again, then reflow."
}

/// A reflow that ran and produced the same number of lines.
#[must_use]
pub const fn reflow_unchanged() -> &'static str {
    "This paragraph already fitted its box, so re-wrapping it changed nothing."
}

/// An alignment applied to a paragraph of one line, which has no width of its
/// own to align within.
#[must_use]
pub const fn align_single_line() -> &'static str {
    "This paragraph is one line, so it has no width of its own to align within. Nothing moved."
}

/// A reflow asked for with no caret placed.
#[must_use]
pub const fn reflow_needs_caret() -> &'static str {
    "Click inside the paragraph you want to re-wrap first, using the Edit text tool, then choose \
     Reflow paragraph."
}

/// A reflow asked for while the caret is placing NEW text.
#[must_use]
pub const fn reflow_needs_existing_text() -> &'static str {
    "The caret is placing new text, so there is no paragraph on the page to re-wrap yet. Finish \
     this text, then click into a paragraph that is already on the page."
}

/// A caret on a run the block recogniser does not place in a paragraph.
#[must_use]
pub const fn reflow_no_block() -> &'static str {
    "This text is not laid out as a paragraph — it is a single line or an isolated label — so \
     there is nothing to re-wrap."
}

/// **Every way a reflow can decline, as ONE type** — `OPERATOR_REQUESTS.md`
/// **O127**, defect 3.
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
    /// The page carries a non-empty EXTRA content stream, so reflow would drop
    /// the text in it.
    ///
    /// ⚠⚠⚠ **UNREACHABLE at engine `025d703d`, kept deliberately.** The
    /// machine-readable claim, which `check-unreachable-refusals` re-measures
    /// against the pinned engine source on every commit:
    ///
    /// UNREACHABLE-FROM: pdfcer_core::text_edit::ReflowApplyError::PageEditedThisSession @ 025d703d
    ///
    /// # The chain, because the unreachability is two links long
    ///
    ///
    /// ⇒ So the arm stays (`ReflowDecline` is exhaustive and compiler-proved,
    /// so an arm is mandatory), the variant stays, [`reflow_after_edit`]'s
    /// sentence stays and is still tested — and **no operator can see any of
    /// it at this pin.**
    ///
    /// # Why it is written down rather than merely true
    ///
    /// This is the **third** unreachable-but-kept refusal in this one enum
    /// ([`Self::PageSetChanged`] is the second) and each of the first two was
    /// found by a reader stumbling on it, months late, while looking for
    /// something else. A sentence nobody has noticed is unreachable is worse
    /// than a deleted one: it is quoted in reasoning about what the program
    /// does, and it reads as measured because it compiles.
    ///
    /// ⇒ Hence the `UNREACHABLE-FROM:` marker above, which is an instrument
    /// rather than a note. It names the engine symbol the claim depends on;
    /// the gate resolves the engine through `Cargo.lock`, so it follows the
    /// pin, and it goes RED the day that symbol gains a constructor — i.e.
    /// the day this paragraph must be deleted. *A tripwire keyed on your own
    /// intention is not a tripwire.*
    ///
    /// # The hazard the variant was written for is real and has not gone
    ///
    /// `reflow_block` re-emits the page's first content object and the commit
    /// sweep empties every other one. What changed is that the engine's
    /// planner now reads the SESSION's graph (its `Pass 257.0`), so a run
    /// appended this session is in the plan's source and survives the re-emit.
    /// The guard had also been firing on pages nobody had edited, because its
    /// condition was structural — *does `contents[1..]` hold a non-empty
    /// stream* — which ISO 32000-1 §7.8.2 permits a producer to author and
    /// which SOLIDWORKS does routinely. That was `request_G015`, filed from
    /// `SW41177.pdf`, whose title sheet carries eight producer-authored
    /// streams and was refused with *"text was added to this page this
    /// session"* on a freshly opened file.
    ///
    /// Earlier history, kept because the shape recurs: until 2026-09-14
    /// this comment described the SHELL's `edit_epoch != 0` forecast and
    /// called it *"the only thing standing between the operator and losing
    /// work he can see on the page"*. That forecast was deleted on 2026-09-05,
    /// the day `Pass 251.0` made the engine refuse the case by name; the
    /// sentence outlived the mechanism by nine days. **A doc comment that
    /// argues for a guard is a claim the guard exists.**
    PageAlreadyEdited,
    /// The engine's page-set guard: a page was added, removed or reordered, and
    /// reflow's planner is indexed against the base document's pages.
    PageSetChanged,
    /// The engine declined and gave no cause this shell may act on.
    ///
    /// **Added 2026-09-07 for the causes `ReflowApplyError` does not
    /// discriminate, and it has narrowed twice since.**
    /// `ReflowApplyError::Unsupported(String)` packs the remainder into one
    /// variant with no discriminant — from *"the block's CTM has a degenerate
    /// (zero) scale"* to a producer quirk with no name — and this shell will
    /// not parse another crate's prose to guess which.
    ///
    /// So the sentence says the one thing true of every remaining case and
    /// **offers no remedy**. Vague, deliberately: the alternative is a remedy
    /// that is wrong most of the time, which is what the wording before it did
    /// when it sent the operator hunting for a page reordering that never
    /// happened.
    ///
    /// **The two narrowings, because this paragraph asked for them and then
    /// did not notice they arrived** (corrected 2026-09-14). It used to say the
    /// engine carries *"ten distinct refusals in one variant"* and to promise
    /// that *"when a discriminant lands, the one recoverable case gets
    /// `PageAlreadyEdited` back"*. Two landed:
    ///
    /// * `PageEditedThisSession` — the recoverable one, exactly as forecast.
    ///   [`Self::PageAlreadyEdited`] was reached from it for seven days.
    ///   ⚠ **Nothing constructs it at engine `025d703d`** (`G015` deleted the
    ///   guard), so the forecast arrived, was honoured, and then expired —
    ///   which is why the claim now carries a gate instead of a sentence.
    /// * `Refused(encoding::Refusal)` — carrying an `RInvTrigger`, which is how
    ///   [`Self::FontIsComposite`] tells `R-INV-4` from the other seven.
    ///
    /// The forecast was right and the count is stale, which is the ordinary
    /// way a doc comment goes wrong: it described the other crate's shape, the
    /// other crate changed shape, and nothing in this one failed to compile.
    EngineDeclined,
    /// The paragraph is drawn in a **composite (Type 0 / CIDFont)** font, and
    /// within-block reflow of composite text is a deferred engine feature
    /// (`R-INV-4`, FF-E).
    ///
    /// # Why this earns its own variant instead of the honest general one
    ///
    /// Because the engine named it, and because on a real drawing it is not the
    /// exception. `SW41177.pdf` — the 36-sheet SOLIDWORKS set `O198` is about
    /// — sets its body text in `AQHZBV+CenturyGothic`, a CIDFont, so
    /// [`Self::EngineDeclined`]'s *"something about how this page was drawn"*
    /// was the answer to **every** reflow the operator attempted on it. A
    /// sentence that vague, shown that consistently, reads as the feature being
    /// broken rather than as one font class being out of scope.
    ///
    /// **The engine hands this over structurally, so no prose is parsed.**
    /// `ReflowApplyError::Refused` carries an `encoding::Refusal` whose
    /// `trigger` is an `RInvTrigger`, and `reflow_apply`'s `refuse_if_composite`
    /// is the ONLY site in that module that constructs one — always with
    /// `RInvTrigger::Composite`. The mapping matches on the trigger anyway, not
    /// on the variant, so if the engine ever widens that gate to another
    /// `R-INV-*` this sentence stops being shown rather than becoming wrong.
    /// *A tripwire keyed on the other side's data survives the other side
    /// changing; one keyed on our reading of it does not.*
    ///
    /// **It offers no remedy because there is none.** Not "choose another
    /// font" — re-setting the face of a whole CAD paragraph to make a re-wrap
    /// possible would change how the drawing looks, which is a far larger act
    /// than the one that was asked for. R9's rule holds: say what happened and
    /// stop.
    FontIsComposite,
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
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::NeedsCaret => reflow_needs_caret(),
            Self::NeedsExistingText => reflow_needs_existing_text(),
            Self::NoBlock => reflow_no_block(),
            // ⚠ UNREACHABLE at engine `025d703d` — `G015` deleted the only
            // producer of `ReflowApplyError::PageEditedThisSession`, which is
            // the only producer of the decline this variant is reached from.
            // Kept for the same reason `PageSetChanged` below is kept, and
            // watched by `check-unreachable-refusals` rather than by a reader.
            Self::PageAlreadyEdited => reflow_after_edit(),
            // Deliberately the same remedy as `PageAlreadyEdited` and
            // deliberately not the same sentence: the operator did something
            // different to get here, and a sentence that named the wrong cause
            // would send them looking for an edit they did not make.
            // ⚠ UNREACHABLE at engine `527b1523` and kept deliberately. Both
            // engine cases it was written for — *"the page's content was
            // already edited this session"* and *"the page set was changed this
            // session"* — were removed by `Pass 257.0` on 2026-09-06, and until
            // 2026-09-07 `reflow_refusal` mapped every `Unsupported` here, so
            // this sentence was shown for ten causes and correct for none of
            // them. It is kept rather than deleted because the guard it
            // describes is real PDF behaviour that a future engine may reinstate
            // by name, and because the sentence is already tested.
            Self::PageSetChanged => {
                "Reflowing a paragraph needs the pages as they were when you opened the file, and \
                 pages have been added, removed or reordered since. Save this file and open it \
                 again, then reflow."
            }
            // The honest general refusal. See `ReflowRefusal::EngineDeclined`
            // for why it names no cause and offers no remedy: the engine packs
            // ten causes into one `Unsupported(String)` with no discriminant,
            // and one invented cause shown ten times is what this replaced.
            Self::EngineDeclined => {
                "pdfcer will not re-wrap this paragraph. Something about how this page was drawn \
                 stops it doing so safely, and your document has not been changed."
            }
            Self::FontIsComposite => {
                "pdfcer cannot re-wrap this paragraph: it is drawn in a font that stores more \
                 than one byte per character, and re-wrapping that kind of text is not built \
                 yet. Your document has not been changed."
            }
            Self::Encrypted => {
                "This document is encrypted, so pdfcer cannot re-write its text. To take the \
                 protection off, use Encrypt… and choose Remove the protection entirely."
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

/// **Why Enter did not make a new line in text that is already on the
/// page**: Enter opens the line's paragraph for re-writing, and these are the
/// cases where that cannot be done.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnterRefusal {
    /// The line is not part of a paragraph pdfcer recognises.
    NoParagraph,
    /// The paragraph mixes fonts, sizes or colours, which a re-write would
    /// flatten into its first line's look.
    MixedLooks,
    /// The engine will not re-write the paragraph; `detail` is its reason.
    Unrewritable { detail: String },
}

impl EnterRefusal {
    /// The status-line sentence.
    #[must_use]
    pub fn line(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::NoParagraph => "pdfcer does not recognise a paragraph around this line, so it \
                                  cannot start a new line in it. Press Ctrl+Enter to finish this \
                                  edit, or use Add text and drag a box for text that wraps."
                .into(),
            Self::MixedLooks => "This paragraph mixes fonts, sizes or colours, and starting a new \
                                 line would set all of it in its first line's look, so it was \
                                 left as it is. Press Ctrl+Enter to finish this edit."
                .into(),
            Self::Unrewritable { detail } => format!(
                "pdfcer cannot re-write this paragraph, so it cannot start a new line in it: \
                 {detail}. Press Ctrl+Enter to finish this edit."
            )
            .into(),
        }
    }
}

/// The disclosure owed when text was added with the pen's invisible switch
/// on, beside the engine's own about rendering mode 3.
#[must_use]
pub const fn added_invisible() -> &'static str {
    "The invisible text is not part of the page's recognised-text layer: removing that layer \
     leaves it in place."
}

/// The disclosure owed when a **clicked** text draft turns out to be
/// multi-line.
#[must_use]
pub const fn point_text_became_a_block() -> &'static str {
    "Your text has more than one line, so it was placed as a block: it starts where you clicked \
     and wraps at the right-hand edge of the sheet. Drag a box with Add text to choose your own \
     width."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The multi-run note says what pdfcer WILL DO, not what it refuses.**
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

    /// **Sharing the line PINS**, and that is the property the whole fix
    /// rests on.
    #[test]
    fn sharing_the_line_pins_the_neighbours() {
        assert!(Reason::SharesTheLine.pins_the_tail());
        let s = pinned_tail_disclosure(Reason::SharesTheLine);
        assert!(s.contains("several separate pieces"), "{s:?}");
    }

    /// **Each pinning reason gets its own explanation.**
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

    /// **Every reflow cause has its own sentence, and no two are the
    /// same.**
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

    /// **The two "save and reopen" causes both carry the remedy**, and it is
    /// the whole of what the operator has to do.
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

    /// **Every Enter refusal names the keyboard route to finish the edit**, and
    /// the no-paragraph one names the gesture that does wrap.
    #[test]
    fn the_enter_refusal_offers_a_keyboard_route_and_a_gesture_route() {
        for why in [
            EnterRefusal::NoParagraph,
            EnterRefusal::MixedLooks,
            EnterRefusal::Unrewritable {
                detail: "x".to_owned(),
            },
        ] {
            let s = why.line();
            assert!(s.contains("Ctrl+Enter"), "{why:?}: {s:?}");
        }
        assert!(EnterRefusal::NoParagraph.line().contains("drag a box"));
    }

    /// **The point-text disclosure says where the width came from.**
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

    /// **Every refusal has a sentence, and none of them is empty.**
    #[test]
    fn every_refusal_says_something() {
        for r in [
            Refusal::NoRun,
            Refusal::NoText,
            Refusal::NoAnchor,
            Refusal::NoUsableEncoding,
            Refusal::PictureOfText,
        ] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }
}
