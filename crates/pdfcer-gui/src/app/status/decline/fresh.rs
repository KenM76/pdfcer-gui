//! # `app::status::decline::fresh` — is the sentence still true?
//!
//! ★★★ **The retirement predicate, and the one fact-pair it reads**, split out
//! of [`super`] on 2026-09-12 under R2. That file stood at 1,460 lines against
//! the 1,500-line ceiling with a form-field refusal inbound that needed a
//! variant, two match arms, a recorder and their reasons — the **fourth** time
//! this module has met the ceiling, after `floor`, `textedit`/`record` and
//! `line`.
//!
//! ## The seam
//!
//! [`super`]'s header sets itself two jobs: *what is a decline* and *how long
//! does it owe its sentence*. Every split so far has taken a question that was
//! neither of those — `floor` answers one about somebody else's protocol,
//! `record` answers *who says one*, `line` answers *in what words*. This one is
//! different and is the better cut for it: it takes **the whole of the second
//! job**, leaving [`super`] the enum and the store.
//!
//! | question | answered in |
//! |---|---|
//! | what is a decline? | [`super`] — the [`Declined`] enum and its 40-odd variants |
//! | who says one, and from which phase? | `decline::record`, `decline::floor`, `decline::textedit`, `decline::clipboard` |
//! | in what words? | `decline::line` |
//! | **is it still true?** | **here** |
//! | where is it stored, and who clears it? | [`super`] — `LAST`, [`super::retire`], [`super::live`] |
//!
//! ## ★★ Why the PURE half, specifically
//!
//! This is the project's standing split, quoted in `still_true`'s own doc from
//! `crate::viewer`'s header: *"this module is unit-testable and the widget code
//! is not."* [`super::live`] needs an `egui::Context` and an
//! [`OpenDoc`](crate::app::state::OpenDoc) to gather its facts and therefore
//! cannot be asserted headlessly; `still_true` takes four plain values and can
//! be, which is why `decline/tests.rs` calls it directly several hundred times.
//! Moving the testable half out leaves each file with one testability story
//! instead of two.
//!
//! ★ So the rule for what may be added here: **nothing that needs a document,
//! a context or a frame.** The parameter list is the contract — see
//! `still_true`'s own note on why it is a list of named facts rather than a
//! `&OpenDoc` — and a function here that could go and ask its own question
//! would dissolve that contract from the inside.
//!
//! ## ★★★ The two arguments every arm in here is making, and they are not the
//! same argument
//!
//! A reader adding a variant will reach for `=> true` and should know which
//! `true` they mean, because the wrong one is invisible:
//!
//! - **"the FILE cannot change under it"** — a certification, whether an
//!   appearance is pdfcer's own, whether an annotation is a fixed-size marker.
//!   The sentence stays true because the fact it reports is a property of the
//!   document, and the document does not change while the operator reads the
//!   status bar.
//! - **"NOTHING HAPPENED"** — a refusal raised before a byte was staged or an
//!   undo entry pushed. There is no later state for the sentence to be stale
//!   against, because there is no later state.
//!
//! Both retire the same way — by [`super::retire`], on the operator's next
//! command — and that shared ending is what makes them easy to conflate. They
//! diverge the moment somebody adds a variant whose fact **is** re-askable: put
//! that one in the wrong group and the bar keeps a sentence the document has
//! already contradicted, which is the one failure mode this whole module exists
//! to prevent.
//!
//! ★ And a third group worth naming because it is the one people reach for and
//! should not: **re-asking by walking the document.** Every predicate in here
//! runs on every frame the bar draws. `parse_acroform`, a page walk, a text
//! extraction — none of them belong, and a fact that can only be re-derived
//! that way is a fact that should be answered by `true` plus [`super::retire`].

use super::Declined;
use crate::app::state::OpenDoc;

impl Declined {
    /// Whether this decline still describes the application in front of the
    /// operator.
    ///
    /// **Pure, and that is the point** — the project's standing split
    /// (`crate::viewer`'s header: *"this module is unit-testable and the widget
    /// code is not"*). Every property of the retirement rule that can be wrong
    /// is decided here and asserted headlessly; [`live`] adds only "go and ask
    /// the two questions".
    ///
    /// The facts are named as booleans rather than taken as a `&OpenDoc`
    /// so that the caller is forced to state *which* question it asked. All
    /// are asked through the same predicates that produced the decline in the
    /// first place, which is what stops a second spelling of "is there
    /// anything to frame?" drifting away from the first.
    ///
    /// # ★ Why a fourth parameter rather than a `&OpenDoc`
    ///
    /// [`History`] arrived with the undo wiring and needed a third fact — *is
    /// there anything on the stack now?* — which is where the temptation to
    /// collapse the list into the document it is all read from is strongest.
    /// The list stays, for the reason it was a list to begin with: a
    /// `&OpenDoc` here would make this function able to ask **any** question,
    /// and the one property that makes it worth testing is that every question
    /// it asks was asked by the code that produced the decline. The parameters
    /// are the contract; [`live`] is the only place allowed to go and get them.
    ///
    /// The two history variants take their fact as *one* [`History`] pair
    /// rather than as two more booleans, so a caller cannot transpose them —
    /// and each arm below names the field it reads, so neither can read the
    /// other's stack.
    #[must_use]
    pub(super) fn still_true(
        &self,
        has_bounds: bool,
        canvas_has_drawn: bool,
        history: History,
        selection_in_form: bool,
    ) -> bool {
        match self {
            // The operator has selected something framable: the sentence is
            // now history, and a stale explanation beside a live control is
            // worse than none — it attaches a refusal to a state that would
            // not produce one.
            Self::NothingToFrame => !has_bounds,
            // The page has drawn. The remedy happened on its own, without the
            // operator doing anything, which is exactly what the sentence
            // promised ("…has not finished drawing").
            Self::CanvasNotDrawn => !canvas_has_drawn,
            // ★ Neither fact is about this one, and there is no third fact to
            // add. A write that failed stays failed until the operator does
            // something about it, and what they do about it is a *command* —
            // which `retire` catches. See the variant's own docs; the two
            // parameters are deliberately ignored rather than being joined by a
            // third that would always be `true`.
            // ★ `ResizeNotRebuildable` joins them: whether an appearance can
            // be rebuilt is a property of the FILE, and it does not change
            // while the operator reads the status bar. What retires it is their
            // next act — including, in the good case, ticking the switch the
            // sentence just named.
            Self::SaveFailed
            | Self::SettingsNotSaved
            | Self::ResizeNotRebuildable { .. }
            | Self::ResizeFixedSizeMarker { .. } => true,
            // ★ `true`, with the others whose state cannot change between two
            // frames. A document's certification is a property of the file: it
            // does not lapse while the operator looks at the status bar, and
            // the only thing that would retire this sentence is opening a
            // different document — which retires every sentence.
            //
            // Deliberately NOT re-asked through `flatten_refusal`. It is a
            // certification census over the whole document, and putting it in
            // the per-frame path that decides whether a status line is still
            // true would pay for it sixty times a second to learn an answer
            // that never moves.
            // ★ `FieldDeleteRefused` joins it on the identical argument with
            // `deletion_refusal` substituted for `flatten_refusal`: `/Encrypt`
            // and a certification signature are properties of the FILE, neither
            // lapses while the operator reads the status bar, and the only
            // thing that would retire the sentence is opening a different
            // document — which retires every sentence. Deliberately NOT
            // re-asked through `deletion_refusal`: that is a signature census
            // over the whole document, and putting it in the per-frame path
            // that decides whether a status line is still true would pay for it
            // sixty times a second to learn an answer that never moves.
            // ★★ The two field-group declines are `true` for a DIFFERENT reason
            // from their neighbours above, and the difference is worth the
            // separate arm rather than an extra `|`.
            //
            // Those are true because the FILE cannot change under them. These
            // are true because **nothing happened**: the preview refused or the
            // deletion refused, so the epoch did not move, the form is as it
            // was, and there is no state for a later frame to find the sentence
            // stale against. What retires them is the operator's next act,
            // which is what retires every decline.
            // ★★ The two bookmark declines join the field-group pair on the
            // *second* argument rather than the first, and it is worth saying
            // which: **nothing happened.** The move was never made or was
            // refused, so the epoch did not move, the outline is as it was, and
            // there is no state for a later frame to find the sentence stale
            // against. Deliberately NOT re-asked against the tree: a walk of
            // the outline to decide whether a status line is still true would
            // pay for a `read_outline` sixty times a second to learn an answer
            // that cannot change without a command — and a command is what
            // `retire` catches.
            // ★ The mode is not going to change between one frame and the
            // next without a **command**, and a command is exactly what
            // `retire` catches — so there is no live predicate to re-ask, on
            // the identical argument the five below it make. Note the
            // asymmetry with `NothingToFrame`: a selection can appear while
            // the operator reads, a mode cannot.
            Self::NodeToolNeedsEditMode
            | Self::FlattenCertified
            | Self::FieldDeleteRefused
            | Self::FieldGroupPreviewRefused
            | Self::FieldGroupDeleteRefused
            // ★ Same ruling, and the sharpest case of it. The stamps folder
            // could genuinely change again while he reads the sentence —
            // Acrobat is what rewrites it — but re-asking would mean rescanning
            // the folder every frame to retire a line he is still reading, and
            // the sentence's own remedy is *reopen this window*, which is a
            // command and is what `retire` catches.
            | Self::CustomStampUnavailable(_)
            | Self::BookmarkMoveIntoOwnSubtree
            | Self::BookmarkMoveRefused
            | Self::VertexEditRefused(_)
            | Self::MarkupNodeRefused(_) => true,
            // ★ Same ruling, third and fourth cases. A name is not going to
            // stop being taken, and a widget is not going to grow a `/T`,
            // between one frame and the next. Both are corrected by the
            // operator doing something — typing a different name and pressing
            // Register again — and pressing Register is a command, which
            // `retire` catches.
            Self::FieldNameTaken | Self::WidgetHasNoName => true,
            // ★★ Same ruling, fifth case — but on the OTHER of the two
            // arguments this function keeps making, and the difference is
            // worth an arm rather than another `|`.
            //
            // The two above are true because the FILE cannot change under them.
            // This one is true because **nothing happened**: the engine refused
            // inside `place_new_field_deferred`, before it staged a byte and
            // before it pushed an undo entry, so the epoch did not move, the
            // form is exactly as it was, and there is no later state for the
            // sentence to be found stale against. What retires it is the
            // operator retyping the name and pressing OK — a command, which
            // `retire` catches.
            //
            // ★ Deliberately NOT re-asked by looking the named field up again.
            // That is a `parse_acroform` walk of the whole document, and
            // putting it in the per-frame path that decides whether a status
            // line is still true would pay for it sixty times a second to learn
            // an answer that cannot change without a command.
            // ★ Joined rather than given an arm of its own, and the
            // reasoning above is why: this is the SAME argument, not merely
            // the same answer. `DottedPartialName` is raised before the verb
            // resolves anything, so no byte was staged and no undo entry was
            // pushed — there is no later state for the sentence to be found
            // stale against, and the operator's next command retires it
            // through `retire`.
            //
            // ★★ It is also the **only** one of the two whose fact could not
            // change even in principle. `FieldPathCrossesTerminal` depends on
            // what the document contains (is `Order` a terminal?) and is true
            // here because re-asking costs a `parse_acroform` walk per frame,
            // not because the answer is fixed. This one depends on nothing but
            // the string: a `/T` cannot contain a period whatever the document
            // holds. If a future reader ever makes the first one re-askable,
            // this one still belongs exactly where it is.
            Self::FieldPathCrossesTerminal(_) | Self::DottedPartialName(_) => true,
            // ★★ Same ruling, and here the temptation to key on
            // `selection_in_form` is strongest: all but one of the sentences are
            // about the DOCUMENT (encrypted, signed, damaged index, nested
            // drawing, drawn nowhere else), none of which changes while the bar
            // is read, and the last sends the operator to click INSIDE a form —
            // so re-asking would delete the instruction as they began to follow it.
            Self::Unshare(_) => true,
            // ★ The stack filled up. Something was authored — or, for redo,
            // something was undone — and the sentence is now history, exactly
            // as `NothingToFrame` is once something is selected. The operator
            // reaches this without invoking any command, which is why the
            // filter is needed at all: `retire` would not have run.
            Self::NothingToUndo => !history.can_undo,
            Self::NothingToRedo => !history.can_redo,
            // ★ True while the operator is still looking at the selection the
            // sentence is about. Selecting something else — including the
            // containing form, which is the remedy the sentence exists to send
            // them to — ends it, without any command being invoked and so
            // without `retire` running. That is precisely the case the filter
            // exists for, and it is the same shape as `NothingToFrame`.
            // ★★★ **One arm per fact, because the two facts have opposite
            // predicates** — and this line asserting one of them for both is
            // what kept `Select containing form`'s refusal invisible.
            //
            // `NotAPath` is about the selection the operator is looking at, so
            // selecting something else ends it — including selecting the
            // containing form, which is the remedy the sentence sends them to.
            // Same shape as `NothingToFrame`, and it needs the filter because
            // the operator reaches the remedy without invoking a command, so
            // `retire` would never run.
            Self::InsideForm(crate::text::status::InsideFormRefusal::NotAPath) => selection_in_form,
            // ★★ `true`, and the inversion is the point. This sentence is
            // recorded precisely BECAUSE nothing form-interior is selected, so
            // filtering it on `selection_in_form` discarded it on the frame it
            // was written — every time, in every build, since the verb
            // shipped. The operator pressed the control and got silence.
            //
            // ★ Retired by `retire` on the operator's next command, like every
            // other stable sentence in this enum.
            Self::InsideForm(crate::text::status::InsideFormRefusal::NoContainingForm) => true,
            // ★ See the variant's docs: nothing on the frame can make a
            // restyle refusal stop being a true report of what happened when
            // the operator pressed the control. `retire` ends it.
            Self::TextStyle(_) => true,
            // ★ Same ruling again, and each of the four refusals earns it
            // separately: a document does not stop being signed, a sidecar does
            // not grow a record, and a routing bug does not fix itself, between
            // one frame and the next. The remedy is always something the
            // operator *does* — and doing it is a command, which `retire`
            // catches.
            Self::Rotate(_) => true,
            // ★★★ `true`, and NOT on the stability argument its neighbours in
            // this arm use — see the variant's own docs, which spend a section
            // refusing to claim it. An unexplained refusal's causes are unknown
            // by construction and some of them do change under the operator.
            //
            // What earns it is that the sentence is in the **past tense**: it
            // reports what happened when the operator pressed, so nothing on a
            // later frame can falsify it — only make it stale, which is
            // `retire`'s job. And there is nothing to re-ask: the live-predicate
            // class exists for declines that can be put back to *the predicate
            // that produced them*, and this one was produced by an error value
            // this shell is not permitted to interpret.
            Self::EditRefused => true,
            // ★ Both on the TENSE argument, not the stability one: several of
            // these causes are live predicates the operator changes in a click,
            // so `EditRefused`'s reasoning applies rather than its neighbours'.
            // The sentence reports the press; `retire` owns stale.
            Self::Reflow(_) | Self::EnterCannotSplit => true,
            // ★ On the TENSE argument too, and here it is the ONLY argument
            // available: the condition this reports is the one the sentence
            // asks the operator to change. A live predicate would retire the
            // explanation at the instant they acted on it.
            Self::ClipboardMode(_) => true,
            // ★★ On the TENSE argument, and inheriting `EditRefused`'s section
            // wholesale — this variant is that one with a cause attached, so
            // the retirement reasoning is unchanged by the payload. It reports
            // what the engine answered about a request that no longer exists;
            // there is no predicate to re-ask, and `retire` owns stale.
            Self::EditText(_) => true,
        }
    }
}

/// **What the command log says right now** — the fact
/// [`Declined::NothingToUndo`] and [`Declined::NothingToRedo`] are retired by.
///
/// A pair rather than two parameters because they are read together, from one
/// borrow of one session, and because a caller that had to pass two loose
/// booleans in the right order would eventually pass them in the wrong one —
/// and the symptom would be a sentence that retires when the *other* stack
/// fills, which reads exactly like a sentence that retires correctly.
///
/// Both are asked through `EditSession`'s own predicates, which is the same
/// pair `crate::app::conditions` publishes `undo.available`/`redo.available`
/// from and the same pair `crate::app::actions`' history arm declines on. Three
/// readers, one derivation: the control cannot be greyed while the sentence
/// says the opposite.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct History {
    /// `EditSession::can_undo` — something has been changed and not taken back.
    pub(crate) can_undo: bool,
    /// `EditSession::can_redo` — something has been taken back and not
    /// re-applied, and no command has been recorded since.
    pub(crate) can_redo: bool,
}

impl History {
    /// What the open document's session currently says.
    ///
    /// The one derivation, so the bar cannot learn this from a different
    /// question than the one that produced the sentence.
    #[must_use]
    pub(super) fn of(doc: &OpenDoc) -> Self {
        Self {
            can_undo: doc.session.can_undo(),
            can_redo: doc.session.can_redo(),
        }
    }
}
