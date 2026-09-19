//! # `text::arrange` — every sentence the three ways of *arranging* something
//! on the page can owe
//!
//! Three gestures share this catalog because they share a subject — **something
//! that is already on the page, moved without being redrawn**:
//!
//! | gesture | module | what it changes |
//! |---|---|---|
//! | the **arrow-key nudge** | [`crate::canvas::moving::nudge`] | where the mark sits, by a whole point at a time |
//! | **Bring to front / Send to back** and their one-step twins | [`crate::app::dispatch::markup`] → [`crate::app::actions::reorder`] | which mark is drawn on top where two overlap |
//! | the **pointer drag** | [`crate::canvas::moving`] | where page content sits, continuously |
//!
//!
//! They are one file rather than two for the reason [`crate::text::rotating`]
//! and [`crate::text::resizing`] are two: those hold sentences for gestures
//! whose *arithmetic* differs and whose refusals therefore differ. These two
//! refuse for the **same three reasons in the same words** — no markup is
//! selected, the mark is locked, the mode may not author markup — and a reader
//! who found "this mark is locked" written twice, once per gesture, would be
//! reading the beginning of a drift.
//!
//! ## ★★★ Why the lock sentence is NOT
//! [`crate::text::panels::properties::markup_locked`]
//!
//! That one is on screen from the moment a locked mark is selected and it reads:
//!
//! > *"This mark is locked by the document, so its **appearance** cannot be
//! > changed here. You can still delete it."*
//!
//! It is a true sentence about **restyling**, which is the surface that draws
//! it. §12.5.3 Table 165 bit 8 is wider than that — *"do not allow the
//! annotation to be deleted or its properties (including position and size) to
//! be modified"* — so a nudge is refused by the same bit for a reason that
//! sentence does not state, and an operator who read *"its appearance cannot be
//! changed"* and then pressed an arrow would have been told about the wrong
//! half of the flag.
//!
//! ⇒ Borrowing it would have been the smaller edit and the misleading one. The
//! two sentences name the same bit and different consequences, which is the
//! case this project's *"one fact, one wording"* rule does **not** cover: the
//! fact is shared, the consequence is not.
//!
//! ★ Noted for whoever owns that string: it says *appearance* where the bit
//! says *appearance, position and size*. Correcting it is not this file's to
//! make.
//!
//! ## ★★ Why a refused nudge is worth a sentence at all, when a refused Delete
//! is not
//!
//! [`crate::canvas::keys`]' Delete rung declines a locked annotation **to the
//! trace**, and its argument is good: the Properties panel is already saying
//! why, from the moment the annotation was selected, so a status line would be
//! the same fact arriving later and worse.
//!
//! That argument does not transfer, and the difference is which sentence is on
//! screen. The panel's standing sentence for a locked mark is about *appearance*
//! (above) and its Delete row's is about *deleting*. **Neither one says a mark
//! cannot be moved.** So an operator who nudges a locked stamp has been told
//! nothing relevant before the press, and silence after it is the shape this
//! project keeps finding — a key that works everywhere else doing nothing here,
//! with no way on screen to learn why.
//!
//! ## ★★★ The one contradiction inside this file, named rather than left
//! for an operator to find
//!
//! [`not_a_markup`] answers an arrow key pressed on page content, and it says:
//!
//! > *“The arrow keys nudge a selected markup. **Drag this with the pointer
//! > instead.**”*
//!
//! [`run_has_no_position_of_its_own`] and [`run_would_drag_the_next_line`]
//! answer a drag on **one line inside a block of text** that this particular
//! file will not let pdfcer move. So there is exactly one selection — a text
//! run entered at the Part rung, in a document written that way — for which
//! the first sentence sends the operator straight into the second. Press an
//! arrow, be told to drag, drag, be told to press Escape.
//!
//!
//! ★★ **It is recorded and not “fixed”, deliberately.** The obvious repair is to
//! widen `not_a_markup` with an *unless it is one line of text* clause, and that
//! would make the common case — a whole object, a whole annotation, anything the
//! pointer really does move — read like a legal notice in order to pre-empt a
//! case the very next gesture explains properly. Widening a true sentence is
//! writing a new sentence, and the new one would be worse for everybody who is
//! not in the narrow case.
//!
//! ⇒ What the two owe each other is that they stay **findable together**, which
//! is what this file is for. Whoever reworks either one now has to read this
//! paragraph first.
//!
//! ## The rule every sentence follows
//!
//! [`crate::text::rotating`]'s, inherited verbatim: **name the thing the
//! operator can see, never the thing pdfcer models.** They can see a stamp, a
//! cloud, a highlight and the order two of them overlap in; they cannot see
//! `/Annots`, an `ObjId` or an indirect reference. A sentence in the file
//! format's vocabulary reads as an internal error, whatever it says.

/// **The Markup tab's Arrange group caption.**
///
/// # ★★ Why this one caption is not in [`crate::text::ribbon`] with the other
/// twenty
///
/// Every other group caption in the build lives there, and this one should
/// eventually join them. It is here because the Arrange group and this catalog
/// were written in the same hour by a track that does not own `text::ribbon`,
/// and a caption written into a file another track is editing is a merge
/// conflict for a five-word string.
///
/// ⇒ **Recorded as a debt rather than presented as a design.** The rule the rest
/// of the build follows — *one file for the ribbon's own words* — is the right
/// one, and moving this function into `text::ribbon` beside
/// `group_markup_style` is a two-line change the next session that touches that
/// file should make.
///
/// ★ The word itself: **Arrange**, which is what Illustrator, InDesign,
/// PowerPoint and Visio all call this group of four. Acrobat calls it nothing —
/// its four are loose in a context menu — so there is no parity reason to
/// deviate and four programs' worth of reason not to.
#[must_use]
pub const fn group_arrange() -> &'static str {
    "Arrange"
}

/// **Why an arrow key moved nothing**, in the shell's reading of the cases.
///
/// # ★ Four variants and not five — the one that is deliberately silent
///
/// A nudge with **nothing selected at all** produces no sentence and no
/// variant. That is not an oversight: the arrow keys are pressed constantly for
/// reasons that have nothing to do with a selection, and a status bar that said
/// *"select something first"* on every stray press would stop being read within
/// a minute. [`crate::canvas::moving::nudge`] returns before it reaches this
/// enum in that case, and the trace still records it.
///
/// The four below are all states the operator reached **by doing something** —
/// they selected a thing, or they are in a mode — so a press is a question and
/// deserves an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeRefusal {
    /// §12.5.3 Table 165 bit 8. The document says the interface may not change
    /// this annotation, and pdfcer honours it rather than letting the engine
    /// refuse — the same ruling [`crate::canvas::annotdrag`] makes for the
    /// pointer, so a key and a drag cannot disagree about what locked means.
    Locked,
    /// Something is selected on the page and it is **not a markup annotation** —
    /// a line of the drawing, a text run, an image.
    ///
    /// See [`not_a_markup`] for why this is worded as a limit of today's build
    /// rather than as a rule of the program.
    NotAMarkup,
    /// A **ce dimension** is selected (rule 15). It is markup, it is selected,
    /// and it is refused — because moving one has to re-measure it, which is a
    /// different verb.
    Dimension,
    /// The page's device transform will not invert, so there is no honest way
    /// to turn a keystroke into a displacement in the page's own units.
    ///
    /// The same condition under which both halves of the `viewer` bridge
    /// decline and under which a pointer drag refuses
    /// ([`crate::canvas::moving::page_delta`] answers `None`). A page like this
    /// cannot be drawn correctly either, so the operator is already looking at
    /// something wrong; the sentence exists so the arrow key is not the second
    /// mystery.
    DegeneratePage,
}

/// The sentence for a refused nudge, or `None` when the case is one this
/// catalog deliberately leaves silent.
///
/// ★ Returns `Option` even though every present variant has a sentence, so that
/// the *shape* of the answer matches [`crate::text::deleting::refusal`]'s —
/// eleven refusals there, three of which speak. A future variant that should
/// not speak is then a `None` arm rather than a change of signature at the one
/// call site.
#[must_use]
pub const fn nudge_refusal(why: NudgeRefusal) -> Option<&'static str> {
    Some(match why {
        NudgeRefusal::Locked => locked_cannot_move(),
        NudgeRefusal::NotAMarkup => not_a_markup(),
        NudgeRefusal::Dimension => dimension_use_the_pointer(),
        NudgeRefusal::DegeneratePage => degenerate_page(),
    })
}

/// **The mark is locked, so it will not move.**
///
/// Names the *document* as the thing that locked it, not pdfcer. That is the
/// truth (§12.5.3 bit 8 is written into the file by whoever produced it) and it
/// is also the only version an operator can act on: a program that refuses is a
/// bug report, and a document that forbids is a fact about the file they can
/// take up with whoever sent it.
///
/// Says **arrow keys or the pointer**, because the operator's next move is to
/// try dragging it, and finding out that fails too is a second discovery of one
/// fact.
#[must_use]
pub const fn locked_cannot_move() -> &'static str {
    "This mark is locked by the document, so it cannot be moved — by the arrow keys or by \
     dragging it."
}

/// **The arrow keys move a mark, and what is selected is not one.**
///
/// # ★★ Worded as *today*, and the reason is that it is true and will change
///
/// `crate::canvas::moving` moves page content on a drag through five verbs, and
/// none of them is out of reach of a keystroke in principle. What is missing is
/// [`crate::canvas::modelneed`]: the deeper rungs need the page's
/// decomposition, a frame only builds one when something has said it needs one,
/// and *"an arrow key was pressed"* is not yet one of the terms. A nudge that
/// reached those rungs without that term would be the `NoObjectModel` defect
/// this project has now shipped four times — a working verb, silently
/// unreachable, reported as a limit of the document.
///
/// ⇒ So this says *"drag it"*, which works today on everything the arrow keys
/// do not, rather than *"that cannot be moved"*, which would be false.
#[must_use]
pub const fn not_a_markup() -> &'static str {
    "The arrow keys nudge a selected markup. Drag this with the pointer instead."
}

/// **A ce dimension is selected**, and the arrow keys will not move it.
///
/// Rule 15 in one sentence of operator copy: a ce dimension is a *measurement*,
/// and moving one may change what it reads, so it travels through
/// `move_dimension` rather than `move_annotation`. `crate::canvas::dimdrag`
/// already owns that gesture for the pointer.
///
/// ★ The word "measurement" does the work. It says why this one thing behaves
/// differently without naming a verb, an `/IT` entry or a subtype — the rule
/// this file's header sets.
#[must_use]
pub const fn dimension_use_the_pointer() -> &'static str {
    "This is a measurement, and moving one re-measures it. Drag it with the pointer so the \
     value keeps up."
}

/// **The page's geometry will not invert.**
///
/// Deliberately does not say *"matrix"*, *"transform"* or *"invert"*. What the
/// operator can act on is that this page is malformed and that the rest of the
/// document is not, so the sentence says which page and stops.
#[must_use]
pub const fn degenerate_page() -> &'static str {
    "pdfcer cannot work out this page's geometry, so nothing on it can be nudged. Other pages \
     are unaffected."
}

// ===========================================================================
// The pointer drag — the one refusal out of eleven that had nothing to say
// ===========================================================================

/// **A drag on one line inside a block of text, refused** — `OPERATOR_REQUESTS.md`
/// O188, 2026-09-15.
///
/// # ★★★ Why this refusal gets a sentence when nine of its ten siblings do not
///
/// [`crate::canvas::moving::Refusal`] has eleven variants and
/// [`crate::canvas::moving::Refusal::worded`] hands exactly two of them to the
/// status bar. The standing argument for the silence of the other nine is in
/// that function, and it is good: *nothing selected*, *the drag never travelled*
/// and *no part entered* all describe states the operator put themselves in and
/// can see, and a bar that narrates the obvious stops being read.
///
/// This one fails that test on every clause. The operator has an outline drawn
/// round one label, presses inside it, drags it across the sheet, and **nothing
/// happens with no sentence anywhere**. They did not make a mistake, they cannot
/// see the cause, and from where they sit dragging is simply broken — which is
/// this project's founding defect shape, arriving on the newest gesture.
///
///
///
/// ★★ **The distinction is not pedantry, it decides what he does next.**
/// *pdfcer cannot move a line* invites him to go looking for a newer build, a
/// setting, or a different program. *This line's position is not written down
/// in this file* tells him it is this drawing, that the very same gesture will
/// work on the next one, and that the remedy in the second clause is the whole
/// of the answer rather than a consolation. A sentence that got those the wrong
/// way round would send him hunting for something that does not exist — the
/// failure O188 was raised about, wearing a different hat.
///
/// # What each one is answering, in the operator's terms
///
/// A block of text is a run of instructions, and the position of each line in
/// it may either be **stated** or **inherited from where the line before it
/// finished** (ISO 32000-1 §9.4.2 — but he cannot see that and the sentences
/// never say it). Where it is stated, there is a number to change and the drag
/// works. Where it is inherited:
///
/// * [`run_has_no_position_of_its_own`] — **this** line's position is
///   inherited, so there is no number to change. Moving it would mean writing
///   one, which is an edit to how the file is structured and not the edit he
///   asked for.
/// * [`run_would_drag_the_next_line`] — the **next** line's position is
///   inherited from this one, so moving this one moves that one too. pdfcer
///   will not silently move something the operator did not select.
///
/// ★★ Both are asked **before the press is accepted**, by
/// `canvas::moving::eligible` through
/// `ObjectModelProvider::text_run_move_refusal_of`, which calls the engine's
/// own `text_run_move_refusal` — the function the planner runs first. So the
/// sentence and the outcome cannot disagree, and no ghost outline is ever drawn
/// for a drag that will not commit.
///
/// # ★★ The order of the clauses is the argument, not the style
///
/// | clause | what it is doing |
/// |---|---|
/// | *“That line”* / *“The line after this one”* | names what was under the pointer, because the outline does not distinguish a line from a block |
/// | *“takes its position from —”* | **the cause, in terms he can act on**: a fact about this document |
/// | *“press Escape to select the whole block of text and drag that”* | R83: a refusal that names its remedy |
///
/// The retired wording led with an **offer** — *Delete removes that line on
/// its own* — because O188 asked to take a title block apart and the removing
/// half had shipped unnoticed. That offer is gone from both sentences, on
/// purpose: the moving half has now shipped too, he has found both, and a line
/// spent advertising Delete is a line not spent on the only thing these two
/// still have to explain, which is why *this* line is the exception.
///
/// ★ It is also a claim this pair could no longer make honestly. Delete's
/// own refusal is the mirror of [`run_would_drag_the_next_line`]'s — the same
/// §9.4.2 inheritance blocks both — so *Delete removes that line on its own*
/// would be false for a selection reachable from here, and an offer that is
/// sometimes false is worse than no offer. A capability claim in product copy
/// needs the same citation a limitation claim does.
///
/// # Tense: present now, and the retirement ruling survives the change
///
/// The retired sentence was past tense (*“That drag **was** on”*) because it
/// reported a gesture, and a report of a past moment stays true whatever the
/// next frame does. These two are **present** tense, and are on firmer ground
/// for it: they state a property of the document, which nothing but an edit can
/// change — not pressing Escape, and not selecting something else.
///
/// ★★ That does **not** reopen the retirement rule.
/// `app::status::decline::fresh` still answers `true` for both, and the trap it
/// is avoiding is unchanged: the remedy these sentences name is *press Escape*,
/// and a predicate keyed on *is a line still selected?* would delete the
/// instruction at the instant the operator began to follow it. The tense
/// argument got stronger; the ruling it was supporting did not move.
///
/// # The one capability claim left, and it is measured
///
/// **Escape ascends exactly one rung.** Decision 025's L1, in
/// [`crate::canvas::keys`]. From the Part rung it lands on the whole text
/// object — which `move_objects`/`transform_objects` really does move, so the
/// remedy is a gesture that works and not a hope.
///
/// ★ Says **block of text** and **line** rather than *show operator*, *run*
/// or *`Tj`*; says *takes its position from* rather than *`Td`*, *`Tm`* or
/// *text-space displacement*. The operator can see a block of text and a line
/// inside it; they cannot see any of the rest, and a sentence in the file
/// format's vocabulary reads as an internal error whatever it says — this
/// file's standing rule.
#[must_use]
pub const fn run_has_no_position_of_its_own() -> &'static str {
    "That line takes its position from the line before it, so this document has no position \
     for it that pdfcer could change — press Escape to select the whole block of text and \
     drag that."
}

/// **A drag on a line that the NEXT line's position is measured from** — the
/// second of O188's two refusals.
///
///
/// ★★ **What this one has to get across that its twin does not** is that the
/// refusal is protecting something. The other sentence reports an absence; this
/// one reports a consequence — the drag *could* be performed and pdfcer is
/// declining, because carrying it out would move a line the operator did not
/// select and did not look at. Said badly, that reads as the program being
/// timid. Said as *moving this line would drag that one with it*, it reads as
/// the program noticing something he could not see, which is what happened.
///
/// ★ **Not** *“the next line is attached to this one”*, which is the obvious
/// plain-English rendering and is wrong in the direction that matters: it
/// suggests a relationship he could detach, and there is no such control. *Takes
/// its position from* names a one-way dependency that is a fact about the file.
#[must_use]
pub const fn run_would_drag_the_next_line() -> &'static str {
    "The line after this one takes its position from this one, so moving it would drag that \
     line along too — press Escape to select the whole block of text and drag that."
}

/// **A drag on a line written in several pieces, one of which has no position
/// of its own** — O214's refusal, and the third of the set.
///
/// ★★★ **The distinction it exists to draw is invisible on the page.** A
/// producer may write one visual line as several show operators; on the
/// operator's own sheet the line `#2 USE SPACERS 8 9 10 11 IF REQUIRED.` is
/// nine of them. Nothing renders differently, so *written in several pieces*
/// is a fact he cannot check and would not believe unstated — which is why
/// the sentence says it plainly first, before saying what follows from it.
///
/// ★★ **Why not reuse [`run_has_no_position_of_its_own`]**, which is the
/// same engine refusal. Because that sentence says *that line takes its
/// position from the line before it*, and here that is **false**: the line
/// has a position of its own, and a join inside it does not. An operator told
/// the false version examines the row above, finds nothing wrong with it, and
/// concludes pdfcer is guessing — the exact failure the pair above was split
/// to avoid.
///
/// ★ Same remedy, and safe for the same reason: Escape ascends one rung to
/// the whole text object, which `transform_objects` really does move.
#[must_use]
pub const fn line_piece_has_no_position_of_its_own() -> &'static str {
    "This line is written in several pieces and one of them takes its position from \
     the piece before it, so moving the line would leave that piece behind — \
     press Escape to select the whole block of text and drag that."
}

/// Said when a line's pieces could not be folded into one undo entry.
///
/// A visual line may be several show operators, and moving or deleting it is
/// therefore several engine commands folded into one with
/// `EditSession::coalesce_last`. That fold answers `false` only when the undo
/// stack was shorter than the count asked for: every piece was changed, and
/// only the grouping failed. Saying so is the whole of the correct response —
/// retrying would apply the change a second time, and silence would leave an
/// operator who presses Undo once, sees the line half-moved, and concludes
/// Undo is broken.
#[must_use]
pub fn line_takes_several_undo_presses(pieces: usize) -> String {
    format!(
        "That line is written in {pieces} pieces, so taking this back needs \
         {pieces} presses of Undo rather than one."
    )
}

// ===========================================================================
// Z-order — what an Arrange command has to disclose
// ===========================================================================

/// **The mark was already where the command would have put it.**
///
/// ★★★ The sentence [`crate::app::actions::reorder::reorder_annotations`]
/// deliberately does **not** have, and the difference between the two callers is
/// the whole argument for it.
///
/// The engine reports `moved == 0` for *"the order given was the order the page
/// already had"*, and that module's own doc calls it **a success with nothing to
/// say** — correctly, because its caller is a drag in a tab-order list that
/// ended where it started, and an operator who dropped a row back on itself
/// needs no commentary.
///
/// A **command** is not a drag. The operator picked *Bring to front* off a
/// ribbon and pressed it deliberately; a press that changes nothing on screen
/// and says nothing is indistinguishable from a broken button, which is this
/// project's founding defect wearing a ribbon control. So the same engine
/// outcome earns a sentence here and none there — not two policies, but one
/// policy (*say what the operator could not otherwise learn*) meeting two
/// gestures.
///
/// `front` chooses which end is named, because *"already at the front"* and
/// *"already at the back"* are different facts and an operator who pressed the
/// wrong one of the pair needs to know which they pressed.
#[must_use]
pub const fn already_there(front: bool) -> &'static str {
    if front {
        "This mark was already in front of everything else on the page."
    } else {
        "This mark was already behind everything else on the page."
    }
}

/// **The mark is locked, so pdfcer leaves its depth alone.**
///
/// # ★★★ A shell decision, and the one place this feature goes beyond the spec
///
/// §12.5.3 Table 165 bit 8 says *"do not allow the annotation to be deleted or
/// its properties (including position and size) to be modified by the user"*. A
/// mark's place in `/Annots` is arguably **not** one of its properties — it is a
/// property of the *page's array* — and `EditSession::reorder_annotations` does
/// not consult the bit at all. So the engine would have permitted this, and
/// pdfcer refuses it.
///
/// The reason is what *locked* means to the person reading the drawing rather
/// than to the clause: an operator who sees a mark refuse to be dragged, refuse
/// to be nudged, refuse to be restyled and refuse to be deleted, and then
/// watches it jump in front of everything on a *Bring to front*, has learned
/// that "locked" means five different things depending on which control they
/// press. The conservative reading is one rule they can hold.
///
/// ⇒ Recorded rather than assumed, because the opposite decision is defensible
/// and somebody will want to re-open it. What would settle it is the reference
/// application: Acrobat greys its whole Arrange submenu for a locked comment,
/// which is the behaviour this matches.
///
/// ★ A separate sentence from [`locked_cannot_move`] because they refuse
/// different things and an operator who pressed *Send to back* and read *"it
/// cannot be moved"* would think the program had misheard them.
#[must_use]
pub const fn locked_cannot_arrange() -> &'static str {
    "This mark is locked by the document, so pdfcer leaves its place in the drawing order \
     alone."
}

/// **Some of this page's annotations cannot be reordered, and stayed put.**
///
/// The `pinned` disclosure, in the operator's terms. `AnnotsReorder::pinned`
/// counts entries written into the page as **direct dictionaries** — they have
/// no object id, so nothing can name them in a new order, and the engine holds
/// them at the index they had while everything else flows around them.
///
/// ★★★ This is the disclosure the whole command owes, and it is the one the
/// brief for this work singled out: *"a z-order command that silently did not
/// take is exactly the failure this project keeps finding."* The mark may
/// genuinely still be behind something after a Bring to front, and the only
/// honest report is to say so **and** say why, because the operator's next act
/// otherwise is to press it again.
///
/// Says *"written into the page in a way that has no name"* rather than *"a
/// direct dictionary"*. The second is the truth and the first is the same truth
/// in a vocabulary an operator has.
#[must_use]
pub fn pinned(count: usize) -> String {
    if count == 1 {
        "One annotation on this page is written into the page itself rather than as a \
         separate object, so it has no name to be reordered by and has stayed where it was — \
         this mark may still be behind it."
            .to_owned()
    } else {
        format!(
            "{count} annotations on this page are written into the page itself rather than as \
             separate objects, so they have no name to be reordered by and have stayed where \
             they were — this mark may still be behind them."
        )
    }
}

/// **Form fields on this page changed their tab order.**
///
/// ★★★ The exact inverse of
/// [`crate::text::forms::reorder_moved_non_widgets`], and writing both down is
/// the point.
///
/// `/Annots` order is two things at once: **paint order** for every annotation,
/// and the **tab sequence** for the form fields among them. So the surprise runs
/// in whichever direction the operator was not looking:
///
/// | the operator did | the engine reports | the surprise |
/// |---|---|---|
/// | dragged a **tab order** | `non_widgets_moved` | the drawing order changed |
/// | pressed **Bring to front** | `moved - non_widgets_moved` | the tab order changed |
///
/// The second number is not a field of `AnnotsReorder`; it is that subtraction,
/// and it is computed at the one call site rather than added to the engine's
/// type, because it is a fact about *this* caller's intent rather than about the
/// reorder.
///
/// ★ Only said when the count is non-zero, which on a page with no form fields
/// is every time. A drawing sheet gets no sentence about tab order.
#[must_use]
pub fn tab_order_changed(count: usize) -> String {
    if count == 1 {
        "One form field on this page changed its place in the tab order, because the drawing \
         order and the tab order are the same list."
            .to_owned()
    } else {
        format!(
            "{count} form fields on this page changed their place in the tab order, because \
             the drawing order and the tab order are the same list."
        )
    }
}

/// **The page shared its annotation list, and pdfcer copied it first.**
///
/// `AnnotsReorder::array_copied`. Nothing is wrong and nothing was lost — the
/// copy is what stops the *other* page's drawing order changing behind its back
/// — but it is a structural change to the file nobody asked for, and rule 4's
/// surviving half is that a consequence the operator cannot see still owes a
/// report.
///
/// ★ Deliberately not [`crate::text::forms::reorder_copied_shared_array`]'s
/// wording re-used. That one is written for somebody who was arranging a tab
/// order; this one is written for somebody who was arranging marks, and the
/// noun it has to use is different. Two sentences about one engine flag, in two
/// vocabularies, is the case this file's header admits.
#[must_use]
pub const fn copied_shared_list() -> &'static str {
    "This page shared its list of annotations with another page. pdfcer copied the list first, \
     so the other page's drawing order is unchanged."
}

/// **A trap network is on this page and has to stay last.**
///
/// ISO 32000-1 §12.5.6.21, restated §14.11.6.2 with the reason: the trap
/// network prints after everything else, so it is the last entry of `/Annots`
/// and no permutation may move it. `EditSession::reorder_annotations` refuses
/// `TrapNetMustStayLast` for a list that tries; this shell never builds such a
/// list, and holds the entry last itself.
///
/// So *Bring to front* on such a page puts the mark in front of everything the
/// operator can see, and behind one thing they cannot. That is a true statement
/// about where their mark now is, and saying nothing would leave a Bring to
/// front that visibly worked and technically did not.
///
/// ★ Says **"prepress"**, because that is the word a drawing office uses for
/// the thing a trap network is part of, and the sentence has to explain why an
/// invisible annotation outranks the operator's own.
#[must_use]
pub const fn trap_net_stays_last() -> &'static str {
    "A prepress trap network on this page must stay last of all, so this mark is now in front \
     of everything except that."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal has a sentence, and no two of them are the same one.**
    ///
    /// The failure this catches is the copy-paste one: four arms of a `match`,
    /// three distinct strings, and the fourth silently telling the operator
    /// about a state they are not in. It is exactly the shape
    /// `text::commands::tests::no_two_commands_share_a_label` catches one file
    /// over, and it shipped there once.
    #[test]
    fn every_nudge_refusal_has_its_own_sentence() {
        let all = [
            NudgeRefusal::Locked,
            NudgeRefusal::NotAMarkup,
            NudgeRefusal::Dimension,
            NudgeRefusal::DegeneratePage,
        ];
        let mut sentences: Vec<&str> = all
            .iter()
            .map(|why| nudge_refusal(*why).expect("every variant speaks today"))
            .collect();
        let total = sentences.len();
        sentences.sort_unstable();
        sentences.dedup();
        assert_eq!(
            sentences.len(),
            total,
            "two refusals share a sentence — one of them is telling the operator about a \
             state they are not in"
        );
    }

    /// **No sentence here names a thing the operator cannot see.**
    ///
    /// The rule the header sets, asserted rather than trusted. The probe list is
    /// the vocabulary of the file format, which is what leaks: every one of
    /// these words is in the doc comments above — correctly, because those are
    /// for a reader of the code — and the test is what keeps the two registers
    /// apart.
    #[test]
    fn no_sentence_speaks_in_the_file_formats_vocabulary() {
        let sentences = [
            locked_cannot_move(),
            not_a_markup(),
            dimension_use_the_pointer(),
            degenerate_page(),
            already_there(true),
            already_there(false),
            locked_cannot_arrange(),
            copied_shared_list(),
            trap_net_stays_last(),
            pinned(1).leak(),
            pinned(4).leak(),
            tab_order_changed(1).leak(),
            tab_order_changed(4).leak(),
        ];
        for text in sentences {
            for probe in [
                "/Annots",
                "ObjId",
                "indirect",
                "dictionary",
                "subtype",
                "/Rect",
                "matrix",
                "permutation",
                "/F ",
                "flag",
            ] {
                assert!(
                    !text.contains(probe),
                    "`{probe}` is the file format's vocabulary, not the operator's: {text:?}"
                );
            }
        }
    }

    /// **The two ends of the pair read differently.**
    ///
    /// `already_there(true)` and `already_there(false)` answer two different
    /// commands, and an operator who pressed *Send to back* and read *"already
    /// in front"* would reasonably conclude the button was mis-wired.
    #[test]
    fn front_and_back_are_not_the_same_sentence() {
        assert_ne!(already_there(true), already_there(false));
        assert!(already_there(true).contains("in front"));
        assert!(already_there(false).contains("behind"));
    }

    /// **The counted sentences agree in number.**
    ///
    /// One of each pair is a singular written out and the other a plural built
    /// from a format string; a build that used the plural for one would read
    /// *"1 annotations"*, which is the sort of thing an operator screenshots.
    #[test]
    fn the_counted_sentences_agree_in_number() {
        assert!(pinned(1).starts_with("One annotation "));
        assert!(pinned(3).starts_with("3 annotations "));
        assert!(tab_order_changed(1).starts_with("One form field "));
        assert!(tab_order_changed(2).starts_with("2 form fields "));
    }

    /// **A refusal blames the document, never the operator and never pdfcer.**
    ///
    /// The lock sentence is the one that could most easily read as a program
    /// failure, and it is the one an operator meets on somebody else's drawing.
    #[test]
    fn the_lock_sentences_name_the_document() {
        for text in [locked_cannot_move(), locked_cannot_arrange()] {
            assert!(text.contains("locked by the document"));
            for alarm in ["error", "failed", "cannot be done", "unsupported"] {
                assert!(!text.to_lowercase().contains(alarm), "{text}");
            }
        }
        assert_ne!(
            locked_cannot_move(),
            locked_cannot_arrange(),
            "the two refuse different things — an operator who pressed Send to back and read \
             'it cannot be moved' would think the program had misheard them"
        );
    }
}
