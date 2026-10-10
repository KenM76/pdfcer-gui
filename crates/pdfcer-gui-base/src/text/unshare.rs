//! # `text::unshare` — every sentence "give this page its own copy" can say
//!
//!
//! ## Why this feature needs the biggest refusal catalog on the canvas
//!
//! Because **the refusals are the feature's whole shape**, and because the
//! commonest one is not an error at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/unshare.md`.

/// **Why this page did not get its own copy of the shared drawing.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnshareRefusal {
    /// The document carries an enforced certification signature (§12.8.4,
    /// `/Perms /DocMDP`).
    ///
    /// `EditError::CertificationForbidsChange`. The variant an operator has
    /// no way whatsoever to guess at: a signed drawing looks exactly like an
    /// unsigned one on the canvas, and the sentence is the only surface that
    /// says otherwise.
    Certified,
    /// The file's trailer `/Size` is suppressing cross-reference entries, and
    /// creating the copy would raise `/Size` and expose them (§7.5.5).
    ///
    /// `EditError::ObjectCreationWouldExposeHiddenObjects`. The engine's own
    /// account of why this is refused rather than performed: *"the exposed
    /// objects are ones the operator did not touch and may not even parse; the
    /// document is frequently loadable **only** because the filter is hiding
    /// them."*
    ///
    /// The count is deliberately not carried — see the enum's docs.
    WouldExposeHiddenObjects,
    /// The drawing is reached on this page only from **inside another
    /// drawing**.
    ///
    /// `EditError::FormNestedInAnotherForm`. **The one refusal here that is
    /// a decision rather than a limit**, and the only one with a real remedy
    /// the operator can reach from where they are standing.
    ///
    /// Re-binding a nested invocation means editing the **parent** form, which
    /// may itself be shared — so the act's blast radius would depend on the
    /// document's nesting structure. `pdfcer-core`'s decision 076 states the
    /// principle and this shell agrees with it: *"a default whose semantics
    /// silently depend on the document's nesting structure is worse than one
    /// that always means the same thing."*
    ///
    /// ⇒ This shell should hit it **rarely**, because it always hands the verb
    /// `FormLeaf::containment[0]` — the outermost enclosing form, which is by
    /// construction invoked by the page rather than by another form. See
    /// `panels::objects::provider::ObjectModelProvider::containing_form_object`,
    /// whose whole doc comment is about that choice. If this sentence appears,
    /// either the decomposition disagrees with the page's own `/Resources`, or
    /// something has started passing `parent()`.
    Nested,
    /// No `/XObject` name in the page's resources resolves to that drawing.
    ///
    /// `EditError::FormNotOnPage`. Reachable through a stale operand: the
    /// selection is resolved when the command is dispatched, and an edit
    /// between that and the apply phase can have re-pointed the page. The
    /// sentence therefore sends the operator to select it again rather than
    /// implying the document is wrong.
    NotOnPage,
    /// The document has no unused object number left.
    ///
    /// `EditError::ObjectNumbersExhausted`. Effectively unreachable — it means
    /// a document at the 32-bit object-number ceiling — and kept for
    /// [`crate::text::rotating`]'s stated reason: the sentence's *existence* is
    /// a tripwire, and its cost is four lines.
    NumbersExhausted,
    /// Nothing that is selected on this page is drawn inside a shared drawing,
    /// so the verb has no operand.
    ///
    /// **Shell-side, raised before the engine is called**, and it is the
    /// only variant here that is not an `EditError`. It is the state the
    /// command's `enabled_when("selection.in_form")` greys the ribbon item for
    /// — and greying enforces nothing, because the context menu, a chord and a
    /// future script all reach the dispatcher without consulting it. This is
    /// what those routes get.
    ///
    /// It is deliberately **not**
    /// `pdfcer_gui::app::status::decline::Declined::InsideForm`, although that
    /// variant is one line away and is about the same fact. That sentence reads
    /// *"That object is inside a form — pdfcer cannot edit inside one yet"*,
    /// which is the report for a verb that refused **because** the selection is
    /// in a form. This verb refuses because it is **not**. Reusing the sentence
    /// would state the exact inverse of what happened.
    NothingInAForm,
    /// **Nothing else draws this drawing, so there is nothing to unshare.**
    ///
    /// Shell-side, like [`Self::NothingInAForm`], raised before the engine is
    /// called — and the second of the two variants here that is a **considered
    /// position rather than a limit**. Added 2026-08-29 for a defect that had
    /// shipped the day before: the command succeeded on a form invoked exactly
    /// once, and told the operator *"every other page still shares the
    /// original"* about a document that had no other page.
    ///
    /// # Why declining is the service and performing it is not
    ///
    /// `EditSession::unshare_form` has **no is-shared guard** — it checks
    /// encryption, certification, `/Size` suppression, form-not-on-page and
    /// nesting, and then allocates. On a one-page CAD sheet wrapped in a single
    /// form, which is the *ordinary* shape of this operator's exports and not
    /// an exotic case, that means:
    ///
    /// | what the operator gets | what it is worth |
    /// |---|---|
    /// | a newly allocated object holding a byte-identical clone | nothing |
    /// | a rewritten page `/Resources` | nothing |
    /// | an undo entry to step back over | less than nothing |
    /// | a **dirty document** where a clean one was open | a save prompt they did not earn |
    /// | a sentence asserting other pages share it | **false about their own file** |
    ///
    /// ⇒ A byte-identical copy and a dirty document for no benefit is not a
    /// service, and the sentence attached to it was the part that made it a
    /// defect rather than merely a waste. Declining changes nothing, costs one
    /// document walk, and replaces a false claim with a true one.
    ///
    /// # It is NOT a fault, and the sentence must not read as one
    ///
    /// This is the only variant in this enum where the operator has done
    /// nothing wrong, the document is in perfect health, and the answer to
    /// what they wanted is **"you already have it."** Every other sentence here
    /// closes by saying *the sharing is unchanged*, because the operator is
    /// about to type into something dangerous. This one closes by saying the
    /// opposite fact — *there is no sharing* — because the operator is about to
    /// type into something safe, and telling them to be careful would be as
    /// wrong as telling them nothing.
    ///
    /// # Why it is a decline and not a silent success
    ///
    /// R9. The condition is a **whole-document walk** and cannot be asked
    /// sixty times a second, so the control is not greyed on it — see
    /// `pdfcer_gui::app::actions::xobject::fanout`, whose doc comment carries the
    /// cost argument. A control that stays live must answer in words when it is
    /// pressed, and *"a refusal is a sentence, never a silence"* is this
    /// project's founding rule. Performing a pointless edit so that *something*
    /// happened would be the silence, wearing a success.
    ///
    /// # What "not shared" is measured as, exactly
    ///
    /// **No page other than this one draws it**, from
    /// `pdfcer_core::text_edit::invocation_set` — which is
    /// `InvocationSet::is_shared()` *plus* the case where every invocation is
    /// on this page. See `pdfcer_gui::app::actions::xobject::fanout` for why the
    /// engine's own predicate is not sufficient on its own: a form drawn three
    /// times on one sheet and nowhere else answers `is_shared() == true`, and
    /// unsharing it moves all three references to the copy and orphans the
    /// original — the same no-benefit edit, and no true sentence to describe
    /// it.
    ///
    /// And it is raised **only when the walk was complete**. A page whose
    /// scan hit the depth guard or a broken form makes the count a *lower
    /// bound*, and declining on a lower bound would be asserting *"nothing else
    /// draws it"* from a measurement that did not finish — the same class of
    /// defect this variant exists to fix, committed in the other direction.
    NotShared,
    /// Anything else the engine declined.
    ///
    /// A catch-all with a **hand-written** sentence, not a rendered error.
    /// `TextStyleRefusal::Other` and `RotateRefusal::Other` set the precedent
    /// and the reasoning is unchanged: wording a decline is catalog work per
    /// refusal, and the honest fallback says *nothing changed* rather than
    /// guessing at a cause.
    ///
    /// It covers `EditError::PageOutOfRange` and `EditError::PageTree`, both of
    /// which mean the page vector moved under a queued command — a state whose
    /// only honest operator-facing content is "nothing happened, try again".
    Other,
}

impl UnshareRefusal {
    /// The sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            // "Signed", not "certified" — `RotateRefusal::Certified` made
            // the same call and the argument is the same: the operator's word
            // for what happened to the file is that somebody signed it. And it
            // says the limit is the DOCUMENT's, because an operator told only
            // "cannot" goes looking for a setting to change.
            Self::Certified => {
                "This document has been signed, and the signature does not allow a change of this \
                 kind. pdfcer copied nothing, so this page still shares that drawing."
            }
            // It says the file is DAMAGED, in those words, because that is
            // the actionable fact and because the alternative reading — "pdfcer
            // is being fussy" — invites somebody to go looking for an override.
            // There is none, and there should be none: the hidden objects are
            // ones nothing in this document points at and some of them may not
            // parse at all.
            Self::WouldExposeHiddenObjects => {
                "This file's index is holding back entries that are damaged or unreadable, and \
                 adding anything to the file would expose them. pdfcer copied nothing, so this \
                 page still shares that drawing."
            }
            // The remedy is the whole sentence. "Select the form" is the
            // command one row above this one in the same menu, so the operator
            // is told to do a thing they can see.
            //
            // It names the CONSEQUENCE of the alternative rather than
            // forbidding it: an operator who genuinely wants every sheet to
            // change is doing nothing wrong, and this feature exists to make
            // that a choice instead of an accident.
            Self::Nested => {
                "That drawing is drawn from inside another one, so giving this page its own copy \
                 would mean copying the outer drawing too — and that one may be shared as well. \
                 Use Select the form first to pick the outer one, or edit in place and accept \
                 that every page using it changes."
            }
            // It sends them to re-select rather than reporting a fault,
            // because the reachable cause is a stale operand — the page changed
            // between the click and the command draining — and "select it again
            // and press this again" is a complete instruction.
            Self::NotOnPage => {
                "That drawing is not on this page any more, so there was nothing to copy. Select \
                 something inside it again and try once more."
            }
            // No remedy, because there is none short of rebuilding the file
            // in another tool. What it does say is the one thing that is true
            // and useful: nothing was changed.
            Self::NumbersExhausted => {
                "This file has no room left for another object, so pdfcer could not make the copy. \
                 Nothing was changed, and this page still shares that drawing."
            }
            // It explains what the command is FOR in the same breath as
            // refusing, because the reachable route to this state is a chord or
            // a menu row on a selection that has nothing to do with forms — an
            // operator who has not yet learned what the command does. The
            // second clause is the instruction.
            Self::NothingInAForm => {
                "Nothing you have selected is drawn inside a shared drawing, so there is nothing \
                 to give this page a copy of. Click something inside the title block or border \
                 first, then use this."
            }
            // The one sentence in this file that reports GOOD NEWS, and
            // every word of it is chosen so it cannot be read as a fault.
            //
            // "only used here" — the measured fact, in the operator's terms.
            // "already belongs to this page alone" — the state they were
            // pressing the button to reach, stated as already true, so the
            // reading is *you have it* rather than *you cannot have it*.
            // "nothing was changed" — the clause every sentence here carries;
            // it also promises the document is still clean, which is half of
            // why declining beats performing a byte-identical copy.
            // "changes no other page" — the permission. The operator pressed
            // this because they are about to edit, and the useful half of the
            // answer is that they may now go ahead.
            //
            // It deliberately does NOT say "pdfcer could not" or "there was
            // nothing to copy" as its opening clause. Both are true and both
            // put the reader in a failure frame for an outcome that is a pass.
            Self::NotShared => {
                "This drawing is only used on this page, so it already belongs to this page \
                 alone. Nothing was copied and nothing was changed — editing it here changes no \
                 other page."
            }
            // No cause named, because none is known. It says the page is
            // exactly as it was, and — the clause every sentence here carries —
            // that the sharing is untouched.
            Self::Other => {
                "pdfcer could not give this page its own copy, and it changed nothing. This page \
                 still shares that drawing with every other page that uses it."
            }
        }
    }
}

/// **Disclosure: this page now has its own copy, and here is what moved.**
#[must_use]
pub fn unshared(references_moved: usize, fanout: Fanout) -> String {
    // Two independent clauses, assembled rather than nested, because they
    // answer two different questions and a four-arm `match` over their product
    // would repeat each half twice and let the copies drift.
    //
    // Clause 1 — what happened ON THIS PAGE.
    let here = if references_moved > 1 {
        // The count is named because it is the whole point of this branch —
        // it is the number the operator would otherwise have to trust — and
        // because it is a count of things they can see on the sheet in front of
        // them, which is what separates a disclosure from evidence.
        format!(
            "This page now has its own copy of that drawing, and all {references_moved} places \
             this page draws it use the copy."
        )
    } else {
        "This page now has its own copy of that drawing.".to_owned()
    };
    // Clause 2 — what is true ELSEWHERE, and it is only ever said from the
    // measurement.
    let elsewhere = fanout.other_pages_clause();
    format!("{here} Editing it from here changes this page only; {elsewhere}")
}

/// **How widely the drawing was drawn, measured before the copy was made.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fanout {
    /// Distinct pages **other than the one that got the copy** that draw the
    /// original, as measured before the copy was made.
    ///
    /// Zero is reachable in the success path only when the walk did not finish
    /// (`lower_bound`) or found the form nowhere at all — a complete walk that
    /// finds no other page is a [`UnshareRefusal::NotShared`] decline and never
    /// reaches this type. See [`Self::other_pages_clause`].
    pub other_pages: usize,
    /// The walk was incomplete, so [`Self::other_pages`] is a floor rather than
    /// a total, and every sentence built from it says *at least*.
    pub lower_bound: bool,
}

impl Fanout {
    /// The half of the disclosure that is about **other pages**.
    #[must_use]
    fn other_pages_clause(self) -> String {
        let at_least = if self.lower_bound { "at least " } else { "" };
        match self.other_pages {
            0 => "anywhere else it is drawn keeps the original.".to_owned(),
            1 => format!("{at_least}1 other page that draws it keeps the original."),
            n => format!("{at_least}{n} other pages that draw it keep the original."),
        }
    }
}

/// **Disclosure appended to a text edit that changed shared content: how to
/// avoid it next time.**
#[must_use]
pub fn shared_content_remedy() -> String {
    "To change this page on its own instead, undo, then use Give this page its own copy, then \
     make the change again."
        .to_owned()
}

/// [`shared_content_remedy`] when an edit to a placed drawing showed at more
/// than one place: `invocations` draws of it on `pages` pages.
#[must_use]
pub fn remedy_if_shared(invocations: usize, pages: usize) -> Option<String> {
    (invocations > 1 || pages > 1).then(shared_content_remedy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal is a sentence, and none of them is empty.**
    #[test]
    fn every_refusal_is_a_sentence() {
        for why in [
            UnshareRefusal::Certified,
            UnshareRefusal::WouldExposeHiddenObjects,
            UnshareRefusal::Nested,
            UnshareRefusal::NotOnPage,
            UnshareRefusal::NumbersExhausted,
            UnshareRefusal::NothingInAForm,
            UnshareRefusal::NotShared,
            UnshareRefusal::Other,
        ] {
            let line = why.line();
            assert!(!line.is_empty(), "{why:?} has no sentence");
            assert!(
                line.ends_with('.'),
                "{why:?} is not a sentence — the founding rule is that a refusal IS one"
            );
        }
    }

    /// **Every refusal says the sharing is unchanged**, which is the one
    /// clause the operator acts on.
    #[test]
    fn every_refusal_says_where_the_sharing_stands() {
        for why in [
            UnshareRefusal::Certified,
            UnshareRefusal::WouldExposeHiddenObjects,
            UnshareRefusal::Nested,
            UnshareRefusal::NotOnPage,
            UnshareRefusal::NumbersExhausted,
            UnshareRefusal::NothingInAForm,
            UnshareRefusal::NotShared,
            UnshareRefusal::Other,
        ] {
            let line = why.line().to_lowercase();
            assert!(
                line.contains("shares")
                    || line.contains("shared")
                    || line.contains("changed")
                    || line.contains("nothing to copy"),
                "{why:?} does not tell the operator that the sharing is untouched, which is the \
                 clause that stops them editing a title block they still share"
            );
        }
    }

    /// **The plural branch names the count and the singular does not.**
    #[test]
    fn the_disclosure_names_a_count_only_when_there_is_one_to_name() {
        let quiet = Fanout {
            other_pages: 0,
            lower_bound: true,
        };
        let one = unshared(1, quiet);
        assert!(!one.contains('1'), "the ordinary case must carry no count");
        assert!(one.contains("this page only"));

        let three = unshared(3, quiet);
        assert!(three.contains('3'), "the multi-name case must say how many");
        assert!(three.contains("this page only"));
    }

    /// **The disclosure never claims other pages share it unless the walk
    /// measured some** — the defect this file was corrected for on 2026-08-29.
    #[test]
    fn the_disclosure_claims_other_pages_only_when_it_measured_some() {
        let none = unshared(
            1,
            Fanout {
                other_pages: 0,
                lower_bound: true,
            },
        );
        assert!(
            !none.contains("other page"),
            "a walk that measured no other page must not name one: {none}"
        );
        assert!(
            none.contains("anywhere else it is drawn"),
            "an incomplete walk still owes a clause that is true either way: {none}"
        );

        let some = unshared(
            1,
            Fanout {
                other_pages: 35,
                lower_bound: false,
            },
        );
        assert!(
            some.contains("35 other pages that draw it keep the original"),
            "a measured fan-out must be stated with its number: {some}"
        );
    }

    /// **An incomplete walk says "at least", and a complete one does not.**
    #[test]
    fn a_lower_bound_is_said_to_be_one_and_a_total_is_not() {
        let bounded = unshared(
            1,
            Fanout {
                other_pages: 2,
                lower_bound: true,
            },
        );
        assert!(
            bounded.contains("at least 2 other pages"),
            "a lower bound must be worded as one: {bounded}"
        );

        let total = unshared(
            1,
            Fanout {
                other_pages: 2,
                lower_bound: false,
            },
        );
        assert!(
            !total.contains("at least"),
            "a complete walk must state its total plainly: {total}"
        );
        assert!(total.contains("2 other pages that draw it keep the original"));
    }

    /// **The singular clause agrees with its verb.**
    #[test]
    fn the_single_other_page_reads_as_english() {
        let one = unshared(
            1,
            Fanout {
                other_pages: 1,
                lower_bound: false,
            },
        );
        assert!(
            one.contains("1 other page that draws it keeps the original"),
            "the singular arm must agree with its verb: {one}"
        );
    }

    /// **The "not shared" decline does not read as a fault.**
    #[test]
    fn the_not_shared_decline_is_good_news() {
        let line = UnshareRefusal::NotShared.line();
        let lower = line.to_lowercase();
        for weasel in ["could not", "failed", "cannot", "error", "sorry"] {
            assert!(
                !lower.contains(weasel),
                "the not-shared decline reads as a fault ({weasel:?}), and it is not one: {line}"
            );
        }
        assert!(
            lower.contains("this page alone") || lower.contains("only used on this page"),
            "it must say the drawing is already private to this page: {line}"
        );
        assert!(
            lower.contains("nothing was changed"),
            "it must promise the document is untouched — declining beats a byte-identical copy \
             precisely because the document stays clean: {line}"
        );
    }

    /// **The shared-content remedy states undo BEFORE unshare.**
    #[test]
    fn the_remedy_puts_undo_first() {
        let line = shared_content_remedy();
        let undo = line.find("undo").expect("the remedy names undo");
        let copy = line
            .find("own copy")
            .expect("the remedy names the unshare command");
        assert!(
            undo < copy,
            "the remedy must say undo FIRST — unsharing after the edit copies the edited stream \
             and leaves every other page changed too"
        );
    }
}
