//! # `text::unshare` — every sentence "give this page its own copy" can say
//!
//! A refusal catalog and one disclosure, for
//! [`crate::app::actions::xobject`] and [`crate::app::dispatch::format`]. The
//! sibling of [`crate::text::rotating`] and [`crate::text::resizing`], written
//! on 2026-08-28 when `EDITABLE_SURFACES.md` found `EditSession::unshare_form`
//! implemented in the engine and called by nothing in this shell, and
//! `pdfcer-core` asked for it by name.
//!
//! ## ★★★ Why this feature needs the biggest refusal catalog on the canvas
//!
//! Because **the refusals are the feature's whole shape**, and because the
//! commonest one is not an error at all.
//!
//! `EditSession::unshare_form` takes `(page_index, form: ObjId)` and clones one
//! form XObject's stream so that this page — and only this page — names the
//! copy. Every other invocation site keeps naming the original and is left
//! byte-identical. That is a **structural** edit: it allocates an object, it
//! rewrites the page's `/Resources`, and it therefore runs the same guard
//! ladder every structural verb in the engine runs (encryption, certification,
//! `/Size` suppression) before it does anything at all.
//!
//! ⇒ So a control that says *"give this page its own copy"* declines for
//! **several distinct reasons** — the list is [`UnshareRefusal`]'s variants and
//! this sentence deliberately no longer counts them, for the reason
//! `tools/gates/run-all.sh`'s header spends six corrections on: *a number
//! written in prose beside the thing it counts is a claim that decays*. Two of
//! them — [`UnshareRefusal::Nested`] and [`UnshareRefusal::NotShared`] — are
//! considered design positions rather than limits, and none of them is
//! visible on the page. The operator presses a button, the drawing looks
//! identical (it *must*: the copy is byte-identical to the original, which is
//! the point), and without a sentence the only difference between success and
//! every failure is a status row that says nothing either way.
//!
//! ★★ This is the project's founding defect shape with the volume turned up:
//! *a gesture that is made, is refused, and reports nothing.* `DEFECTS.md` D4a.
//! And it is worse here than for a drag, because a successful unshare also
//! looks like nothing happened — see [`unshared`], which is why the success
//! path owes a sentence too.
//!
//! ## ★★ The vocabulary, decided once
//!
//! | the file's word | the operator's word here | why |
//! |---|---|---|
//! | form XObject | **drawing** / **the shared drawing** | §8.10.1's own illustration is a CAD system's standard component; the operator calls their title block a drawing, not an XObject |
//! | invocation | **place it is drawn** | an invocation is a `Do` operator; a place is something they can point at |
//! | page | **sheet** *(only where the fan-out is the subject)* | a 36-sheet drawing set is "sheets" in every room this software is used in. Elsewhere "page", because that is what the page box in the status bar says |
//! | `ObjId` | **not named at all** | see [`unshared`]: an object number is evidence, and evidence goes to the trace |
//!
//! [`crate::text::rotating`]'s rule is inherited unchanged: **name the thing
//! the operator can see, never the thing pdfcer models.** A refusal phrased in
//! the file format's vocabulary reads as an internal error, and an internal
//! error is a thing an operator reports rather than acts on.
//!
//! ## ★ What is deliberately NOT worded here
//!
//! **Nothing.** That is unusual in this directory and it is the point: every
//! `EditError` this verb can return has a variant below, including the three
//! that are unreachable on a well-formed file. [`crate::text::rotating`]'s
//! argument for keeping its two unreachable variants is the argument for all of
//! these — *"a routing bug with a sentence is a bug report; a routing bug
//! without one is a handle that does nothing"* — and this verb has more ways to
//! be routed wrongly than a rotation does, because its operand is derived
//! through two hops (a selected leaf, then that leaf's outermost enclosing
//! form) rather than being the thing that was clicked.

/// **Why this page did not get its own copy of the shared drawing.**
///
/// # ★★★ A `Copy` enum rather than the engine's own `Display`
///
/// [`crate::text::status::TextStyleRefusal`]'s reason, adopted unchanged and
/// for the third time: a `format!` of an `EditError` would route **diagnostic
/// prose into the UI**, which `tools/gates/check-ui-strings.sh`'s exclusion 3
/// names in as many words — *"this exclusion is not permission to route UI text
/// through an error type."* An enum keeps
/// [`crate::app::status::decline::Declined`] `Copy`, keeps its `line()`
/// returning `&'static str`, and keeps every operator-visible word in this
/// file under **R1**.
///
/// # ★★ The one variant that carries no number, and why that is deliberate
///
/// [`Self::WouldExposeHiddenObjects`] is raised by the engine with a `count` —
/// how many cross-reference entries the file's `/Size` is currently hiding —
/// and this enum drops it. Two reasons, either sufficient:
///
/// 1. Carrying it would make this type non-`Copy`-friendly in the sense that
///    matters: `Declined::line()` returns `&'static str`, and a counted
///    sentence needs a `String` and an allocation on a path that runs while a
///    status bar is being laid out.
/// 2. **The number is not actionable and is barely meaningful to the reader.**
///    "17 hidden cross-reference entries" tells an operator nothing they can
///    do. What they can act on is *this file is damaged in a way that makes it
///    unsafe to add anything to*, and that is what the sentence says. The count
///    goes to the trace, where evidence belongs — the same split
///    `canvas::textedit::report` makes for `followers_repositioned`.
///
/// # ★ Ordering
///
/// The variants are in **the order the engine checks them**, which is also the
/// order of decreasing "this is about the whole document" and increasing "this
/// is about what you just clicked". A reader comparing this enum against
/// `EditSession::unshare_form`'s body should be able to walk both top to bottom
/// together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnshareRefusal {
    /// The document carries `/Encrypt` (§7.6).
    ///
    /// `EditError::DocumentEncrypted`, the engine's first guard. Reachable on
    /// an ordinary file — plenty of drawing sets ship with an owner password
    /// set for printing — and completely invisible on the canvas, which is why
    /// it is worded rather than traced.
    Encrypted,
    /// The document carries an enforced certification signature (§12.8.4,
    /// `/Perms /DocMDP`).
    ///
    /// `EditError::CertificationForbidsChange`. ★ The variant an operator has
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
    /// ★ The count is deliberately not carried — see the enum's docs.
    WouldExposeHiddenObjects,
    /// The drawing is reached on this page only from **inside another
    /// drawing**.
    ///
    /// `EditError::FormNestedInAnotherForm`. ★★★ **The one refusal here that is
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
    /// ★★ **Shell-side, raised before the engine is called**, and it is the
    /// only variant here that is not an `EditError`. It is the state the
    /// command's `enabled_when("selection.in_form")` greys the ribbon item for
    /// — and greying enforces nothing, because the context menu, a chord and a
    /// future script all reach the dispatcher without consulting it. This is
    /// what those routes get.
    ///
    /// ★ It is deliberately **not**
    /// [`crate::app::status::decline::Declined::InsideForm`], although that
    /// variant is one line away and is about the same fact. That sentence reads
    /// *"That object is inside a form — pdfcer cannot edit inside one yet"*,
    /// which is the report for a verb that refused **because** the selection is
    /// in a form. This verb refuses because it is **not**. Reusing the sentence
    /// would state the exact inverse of what happened.
    NothingInAForm,
    /// ★★★ **Nothing else draws this drawing, so there is nothing to unshare.**
    ///
    /// Shell-side, like [`Self::NothingInAForm`], raised before the engine is
    /// called — and the second of the two variants here that is a **considered
    /// position rather than a limit**. Added 2026-08-29 for a defect that had
    /// shipped the day before: the command succeeded on a form invoked exactly
    /// once, and told the operator *"every other page still shares the
    /// original"* about a document that had no other page.
    ///
    /// # ★★★ Why declining is the service and performing it is not
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
    /// # ★★★ It is NOT a fault, and the sentence must not read as one
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
    /// # ★★ Why it is a decline and not a silent success
    ///
    /// R9. The condition is a **whole-document walk** and cannot be asked
    /// sixty times a second, so the control is not greyed on it — see
    /// `crate::app::actions::xobject::fanout`, whose doc comment carries the
    /// cost argument. A control that stays live must answer in words when it is
    /// pressed, and *"a refusal is a sentence, never a silence"* is this
    /// project's founding rule. Performing a pointless edit so that *something*
    /// happened would be the silence, wearing a success.
    ///
    /// # ★ What "not shared" is measured as, exactly
    ///
    /// **No page other than this one draws it**, from
    /// `pdfcer_core::text_edit::invocation_set` — which is
    /// `InvocationSet::is_shared()` *plus* the case where every invocation is
    /// on this page. See `crate::app::actions::xobject::fanout` for why the
    /// engine's own predicate is not sufficient on its own: a form drawn three
    /// times on one sheet and nowhere else answers `is_shared() == true`, and
    /// unsharing it moves all three references to the copy and orphans the
    /// original — the same no-benefit edit, and no true sentence to describe
    /// it.
    ///
    /// ★★ And it is raised **only when the walk was complete**. A page whose
    /// scan hit the depth guard or a broken form makes the count a *lower
    /// bound*, and declining on a lower bound would be asserting *"nothing else
    /// draws it"* from a measurement that did not finish — the same class of
    /// defect this variant exists to fix, committed in the other direction.
    NotShared,
    /// Anything else the engine declined.
    ///
    /// ★ A catch-all with a **hand-written** sentence, not a rendered error.
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
    ///
    /// # ★★★ Every one of them ends by saying the sharing is unchanged
    ///
    /// That clause is not padding, and it is the clause that took the longest
    /// to get right. The operator pressed this button **because they are about
    /// to edit something**, and the thing they need to know after a refusal is
    /// not "it failed" — it is ***do not now go and type into that title
    /// block***, because doing so will change thirty-six sheets.
    ///
    /// A refusal that says only "pdfcer could not do that" leaves them believing
    /// the safe state might have been reached. Every sentence below therefore
    /// closes the loop explicitly: the page still shares the drawing.
    ///
    /// ★★★ **[`Self::NotShared`] is the one exception, and it is the same rule
    /// rather than a break from it.** The clause exists to tell the operator
    /// *what is true about the sharing before they type*. For every other
    /// variant that fact is "you still share it"; for that one it is "there was
    /// never anything to share", which is the same clause with the opposite
    /// value and is exactly as load-bearing. What is forbidden is a sentence
    /// that leaves the question open, not a particular answer to it — and the
    /// test below (`every_refusal_says_where_the_sharing_stands`) asserts the
    /// property that way rather than pinning one of the two answers.
    ///
    /// # ★★ Remedy first where there is one
    ///
    /// [`crate::text::resizing`]'s rule, inherited: the operator is looking at
    /// something that did not happen, and the useful half is *what to do now*.
    /// [`Self::Nested`] and [`Self::NothingInAForm`] both name a next act;
    /// the rest have none, and none is invented for them.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            // ★ "Encrypted" is a word the operator will have met — it is what
            // the password dialog in every other reader calls it — and the
            // limit is placed on pdfcer, not on the document, because the file
            // is not malformed and there is nothing in it to fix.
            Self::Encrypted => {
                "This document is encrypted, and pdfcer cannot add anything to an encrypted file \
                 yet. This page still shares that drawing with every other page that uses it."
            }
            // ★★ "Signed", not "certified" — `RotateRefusal::Certified` made
            // the same call and the argument is the same: the operator's word
            // for what happened to the file is that somebody signed it. And it
            // says the limit is the DOCUMENT's, because an operator told only
            // "cannot" goes looking for a setting to change.
            Self::Certified => {
                "This document has been signed, and the signature does not allow a change of this \
                 kind. pdfcer copied nothing, so this page still shares that drawing."
            }
            // ★★ It says the file is DAMAGED, in those words, because that is
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
            // ★★★ The remedy is the whole sentence. "Select the form" is the
            // command one row above this one in the same menu, so the operator
            // is told to do a thing they can see.
            //
            // ★ It names the CONSEQUENCE of the alternative rather than
            // forbidding it: an operator who genuinely wants every sheet to
            // change is doing nothing wrong, and this feature exists to make
            // that a choice instead of an accident.
            Self::Nested => {
                "That drawing is drawn from inside another one, so giving this page its own copy \
                 would mean copying the outer drawing too — and that one may be shared as well. \
                 Use Select the form first to pick the outer one, or edit in place and accept \
                 that every page using it changes."
            }
            // ★ It sends them to re-select rather than reporting a fault,
            // because the reachable cause is a stale operand — the page changed
            // between the click and the command draining — and "select it again
            // and press this again" is a complete instruction.
            Self::NotOnPage => {
                "That drawing is not on this page any more, so there was nothing to copy. Select \
                 something inside it again and try once more."
            }
            // ★ No remedy, because there is none short of rebuilding the file
            // in another tool. What it does say is the one thing that is true
            // and useful: nothing was changed.
            Self::NumbersExhausted => {
                "This file has no room left for another object, so pdfcer could not make the copy. \
                 Nothing was changed, and this page still shares that drawing."
            }
            // ★★ It explains what the command is FOR in the same breath as
            // refusing, because the reachable route to this state is a chord or
            // a menu row on a selection that has nothing to do with forms — an
            // operator who has not yet learned what the command does. The
            // second clause is the instruction.
            Self::NothingInAForm => {
                "Nothing you have selected is drawn inside a shared drawing, so there is nothing \
                 to give this page a copy of. Click something inside the title block or border \
                 first, then use this."
            }
            // ★★★ The one sentence in this file that reports GOOD NEWS, and
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
            // ★ It deliberately does NOT say "pdfcer could not" or "there was
            // nothing to copy" as its opening clause. Both are true and both
            // put the reader in a failure frame for an outcome that is a pass.
            Self::NotShared => {
                "This drawing is only used on this page, so it already belongs to this page \
                 alone. Nothing was copied and nothing was changed — editing it here changes no \
                 other page."
            }
            // ★ No cause named, because none is known. It says the page is
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
///
/// # ★★★ Why a SUCCESS owes a sentence at all, which is unusual
///
/// Most disclosures in this crate exist because a consequence is invisible.
/// This one exists because **the whole act is invisible, by design**.
///
/// `EditSession::unshare_form` clones the form stream verbatim — the engine's
/// own comment says the copy *"carries the ORIGINAL's value verbatim, span and
/// all"*, so unsharing costs no duplicated bytes until the copy is actually
/// edited. The page therefore renders **pixel-for-pixel identically** before and
/// after. Nothing moves, nothing changes colour, nothing appears or disappears.
///
/// ⇒ Without a sentence, the operator's evidence that the command worked is
/// indistinguishable from their evidence that it did nothing — which is the
/// same state as an unworded refusal, arriving through the success path. R8b
/// rule 4 as narrowed by pdfcer's decision 059 (*render normally, report
/// separately*) applies with unusual force: there is nothing to render.
///
/// # ★★ What the engine asked a shell to say, verbatim
///
/// [`pdfcer_core::edit::UnshareFormReport`] is documented as naming the copy and
/// how many references moved *"so a shell can say what happened rather than
/// only that it worked"*. This is that sentence.
///
/// # ★★ What it does NOT say: the object numbers
///
/// `UnshareFormReport::original` and `::copy` are `ObjId`s, and neither reaches
/// the status row. That is the split `canvas::textedit::report` states as a
/// rule and this file follows: *a number about a content stream is evidence, not
/// a disclosure.* An operator cannot act on "object 47"; a driven check can, and
/// a regression then names itself. Both numbers go to the trace from
/// `app::actions::xobject`, where the object-clipboard and text-edit arms
/// already send theirs.
///
/// # The plural, and why it is a branch rather than a format string
///
/// `references_moved` is *"usually 1. Greater than 1 when the page invoked the
/// same form under several names"* — a real case on CAD output, where one title
/// block is drawn once per view. The two sentences are genuinely different
/// statements, not one sentence with a number in it:
///
/// - at 1, the operator needs to know the change is now local to this page;
/// - above 1, they additionally need to know that **all** the places this page
///   draws it moved together, because the alternative reading — "one of the
///   three title blocks on this sheet is now private and two are not" — would be
///   a genuinely alarming and genuinely wrong thing to infer, and it is exactly
///   what an operator who knows the page draws it three times will infer from
///   silence.
///
/// That plurality is the engine's decision, stated in the verb's own docs: *"the
/// unit of this operation is the PAGE"*. The sentence says so.
///
/// # ★★★ The second axis, added 2026-08-29: how many OTHER pages, measured
///
/// The sentence used to end *"every other page still shares the original"* in
/// both branches, unconditionally, on a command that had never asked how many
/// other pages there were. On a single-invocation form that clause was **false
/// about the operator's own file**; on a genuinely shared one it was
/// indistinguishable from the false version, so the operator who *did* have a
/// thirty-six-sheet title block learned nothing from it either. One
/// unconditional clause managed to be both a lie and useless.
///
/// [`Fanout`] carries the measurement, and every claim about other pages is now
/// made from it or not made at all. See its docs for the three shapes and for
/// why "at least" is not a hedge.
#[must_use]
pub fn unshared(references_moved: usize, fanout: Fanout) -> String {
    // ★ Two independent clauses, assembled rather than nested, because they
    // answer two different questions and a four-arm `match` over their product
    // would repeat each half twice and let the copies drift.
    //
    // Clause 1 — what happened ON THIS PAGE.
    let here = if references_moved > 1 {
        // ★ The count is named because it is the whole point of this branch —
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
///
/// # ★★★ Why this type exists rather than two loose parameters
///
/// Because the two fields are only ever meaningful **together**, and read apart
/// they produce the exact sentence this type was introduced to delete. `3`
/// alone says *"three other pages draw it"*; `3` with `lower_bound` says *"at
/// least three, and pdfcer could not finish looking"*. A caller handed two bare
/// arguments eventually passes them in the wrong order or forgets the second,
/// and the symptom of forgetting the second is **an under-count presented as a
/// total** — which `pdfcer_core::text_edit::invocation_set`'s own documentation
/// calls *"the same class of defect as a silent edit"*.
///
/// It is the same argument [`crate::app::status::decline::History`] makes for
/// pairing undo and redo: *"a caller that had to pass two loose booleans in the
/// right order would eventually pass them in the wrong one."*
///
/// # ★★★ Where the numbers come from, and what they are NOT
///
/// `crate::app::actions::xobject::fanout` walks the document once, on the
/// press, through `pdfcer_core::text_edit::invocation_set`. Both fields are read
/// off the returned `InvocationSet`:
///
/// | field | source | measured **before** or **after** the copy |
/// |---|---|---|
/// | [`Self::other_pages`] | `set.pages`, minus this page | **before** |
/// | [`Self::lower_bound`] | `InvocationSet::is_lower_bound()` | **before** |
///
/// ★★ *Before* is load-bearing and is not an implementation detail. After the
/// copy is made, this page's invocations name the copy, and a walk run then
/// would report the original's fan-out with this page already subtracted. The
/// number the operator needs — *how many sheets are still on the original* — is
/// the same either way only because the subtraction is done here rather than by
/// the file. Measuring after would give the right answer for the wrong reason
/// and would break the moment the verb's granularity changed.
///
/// # ★★ "At least" is a statement of fact, not a hedge
///
/// `InvocationSet::is_lower_bound()` is true when some page's scan hit the
/// depth guard or a form pdfcer could not decode. Those pages may or may not
/// draw this form; nothing in the count knows. Printing the bare number would
/// present an under-count as a total, and an operator who reads *"2 other pages
/// keep the original"* on a document where the true answer is nine will not
/// check the other seven. So the sentence says *at least*, and it is the
/// **honest** wording rather than the cautious one.
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
    ///
    /// # The three shapes, and why none of them is a format string with a
    /// # number in it
    ///
    /// | measurement | clause |
    /// |---|---|
    /// | nothing to say (`other_pages == 0`) | *"anywhere else it is drawn keeps the original."* |
    /// | exactly one | *"1 other page that draws it keeps the original."* |
    /// | more than one | *"N other pages that draw it keep the original."* |
    ///
    /// ★★ The zero case does **not** print "0 other pages" and does not claim
    /// there are none. It is reachable only from an incomplete walk (or from a
    /// form the walk could not see at all), and on an incomplete walk *"no
    /// other page draws it"* is precisely the claim that has not been measured.
    /// The wording it uses instead — *anywhere else it is drawn* — is true
    /// whether that set is empty or not, which is the only kind of sentence a
    /// failed measurement is entitled to.
    ///
    /// ★ Verb agreement is why one and many are separate arms rather than one
    /// `format!` with a pluralised noun: *"1 other page … keeps"* against *"3
    /// other pages … keep"* differ in two places, and "page(s) … keep(s)" is
    /// the shape [`crate::text`]'s rules exist to keep out of an operator's
    /// status bar.
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
///
/// # ★★★ Why the shell adds a sentence to the engine's own list
///
/// `pdfcer-core` already puts a `"SHARED CONTENT: …"` sentence into
/// `text_edit::EditReport::disclosures`, worded for direct display, and
/// `canvas::textedit::report`'s header rules — correctly — that **re-wording it
/// here would be a second account of one fact, free to drift**. Nothing below
/// re-words it. This is a second, *different* fact, and it is one the engine
/// cannot state: **pdfcer-core does not know what this shell's commands are
/// called.**
///
/// The engine's sentence says *what happened* — the edit changed every place
/// this form is drawn, because the standard binds a form to no page and there is
/// exactly one stream holding those glyphs. Complete, and true. What it cannot
/// say is *what to do about it*, because the answer is the name of a control in
/// a program it has never seen.
///
/// ⇒ The precedent for appending is already in the same apply arm:
/// `crate::text::textedit::pinned_tail_disclosure` is a shell-authored sentence
/// pushed onto the engine's list, for the same shape of reason — the engine says
/// nothing about a pinned tail because from its side pinning is what was asked
/// for, and the operator still owes it.
///
/// # ★★★ The sequence is UNDO first, and getting that wrong would be a lie
///
/// This is the sentence's load-bearing clause and it is worth the paragraph.
///
/// The naive remedy — *"press Unshare now"* — **does not work, and would make
/// things worse.** The edit has already been written into the one shared stream
/// object; every page that draws it already shows the change. Unsharing at that
/// point copies the **already-edited** stream to this page and re-points this
/// page at it. The other thirty-five sheets keep the original object, which is
/// the one that was edited. The operator would end up with the change on every
/// sheet **and** a redundant private copy, and a sentence that told them to do
/// that would have caused the damage it was warning about.
///
/// The order that works is **undo, unshare, edit again**, and it is stated in
/// that order with no room to read it otherwise.
///
/// # ★★ Why it is worded as a future-tense offer, not a warning
///
/// Because at the moment it is read, the fan-out has already happened and may
/// well have been wanted — §8.10.1's whole purpose for the feature is that one
/// component appears on many sheets, and a drawing-office correction to a title
/// block is *supposed* to reach all of them. This shell must not imply the
/// operator has made a mistake. It names the option they did not know they had.
///
/// # ★ Why it is appended rather than replacing the engine's sentence
///
/// The engine's sentence carries `InvocationSet::describe()` — the actual
/// counts, "3 pages, 5 places" or whatever this document is — and that is the
/// fact that makes the disclosure *startling*, which is the property
/// `canvas::textedit::report` says it is meant to have. Dropping it to make room
/// for a remedy would trade the alarming half for the useful half. Both, in the
/// engine's order: what happened, then what to do.
#[must_use]
pub fn shared_content_remedy() -> String {
    "To change this page on its own instead, undo, then use Give this page its own copy, then \
     make the change again."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **Every refusal is a sentence, and none of them is empty.**
    ///
    /// The check that a variant added later cannot ship silent — the founding
    /// rule applied to the enum that exists to serve it. Copied deliberately
    /// from `crate::text::rotating::tests::every_refusal_is_a_sentence` rather
    /// than generalised: a shared helper would be one more thing to keep in
    /// step, and the assertion is three lines.
    #[test]
    fn every_refusal_is_a_sentence() {
        for why in [
            UnshareRefusal::Encrypted,
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

    /// ★★★ **Every refusal says the sharing is unchanged**, which is the one
    /// clause the operator acts on.
    ///
    /// The reachable failure this pins: somebody adds a variant, writes a
    /// perfectly good sentence explaining *why* pdfcer declined, and leaves out
    /// the half that stops the operator going straight on to type into a title
    /// block that is still shared by thirty-six sheets. The refusal would read
    /// as complete and would omit the only part that prevents damage.
    ///
    /// ★ Asserted on a **word**, not on a phrase, because the wording of each
    /// sentence is deliberately different and pinning a phrase would either
    /// force a set of identical endings or fail on the first rewrite. Every
    /// sentence must mention what is *shared* or that nothing *changed*; that
    /// is the property, and it is the weakest assertion that still catches the
    /// omission.
    ///
    /// ★★ [`UnshareRefusal::NotShared`] satisfies it by saying *nothing was
    /// changed* while asserting the opposite about the sharing — see
    /// [`UnshareRefusal::line`]'s docs. The property is *"the sentence settles
    /// where the sharing stands"*, not *"the sentence says the page still
    /// shares it"*, and the name below says so.
    #[test]
    fn every_refusal_says_where_the_sharing_stands() {
        for why in [
            UnshareRefusal::Encrypted,
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

    /// ★★ **The plural branch names the count and the singular does not.**
    ///
    /// Both halves matter. A build that dropped the count on the multi-name
    /// case would leave an operator who knows the sheet draws its title block
    /// three times to guess whether one of the three moved or all of them; a
    /// build that printed "1 place" on the ordinary case would put a number in
    /// front of every operator for no reason at all.
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

    /// ★★★ **The disclosure never claims other pages share it unless the walk
    /// measured some** — the defect this file was corrected for on 2026-08-29.
    ///
    /// The shipped sentence ended *"every other page still shares the
    /// original"* in both branches, on a command that had never counted the
    /// invocations. This pins the property that made it a defect rather than a
    /// wording preference: a claim about other pages appears **only** when
    /// [`Fanout::other_pages`] is non-zero.
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

    /// ★★★ **An incomplete walk says "at least", and a complete one does not.**
    ///
    /// `InvocationSet::is_lower_bound`'s own documentation is the reason:
    /// *"an under-count presented as a total is the same class of defect as a
    /// silent edit."* Both directions are asserted, because a build that said
    /// "at least" unconditionally would be hedging a number it had actually
    /// measured, which teaches the operator to discount the hedge on the one
    /// document where it means something.
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

    /// ★★ **The singular clause agrees with its verb.**
    ///
    /// One other page "keeps" the original; three "keep" it. The arm exists
    /// because the alternative — one `format!` with a pluralised noun — puts
    /// "page(s) … keep(s)" on an operator's status bar, and this directory's
    /// rules exist to keep that out of it.
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

    /// ★★★ **The "not shared" decline does not read as a fault.**
    ///
    /// The sentence reports the one outcome in this enum where the operator did
    /// nothing wrong and the document is in perfect health, and the whole
    /// reason it is a *decline* rather than a pointless structural edit is that
    /// the truthful sentence is better than the edit. A rewrite that opened it
    /// with "pdfcer could not" would keep every fact and lose that.
    ///
    /// ★ Asserted on the absence of failure vocabulary and the presence of the
    /// two facts the operator acts on — *it is already private* and *nothing
    /// was changed* — rather than on the sentence itself, which is free to be
    /// reworded.
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

    /// ★★★ **The shared-content remedy states undo BEFORE unshare.**
    ///
    /// The one assertion in this file that pins a *sequence* rather than a
    /// property, and it is pinned because getting it backwards produces a
    /// sentence that reads perfectly and causes the damage it warns about: at
    /// the moment this is read the edit is already in the shared stream, so
    /// unsharing first copies the edited version and leaves every other page
    /// changed as well. See [`shared_content_remedy`]'s docs for the full
    /// account.
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
