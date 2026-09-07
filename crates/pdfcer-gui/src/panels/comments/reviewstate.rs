//! # `panels::comments::reviewstate` — a comment's **review status**
//!
//! `/State` and `/StateModel` (§12.5.6.3, Table 171; 2.0's Table 174), read
//! into a per-reviewer history, shown on the row, filtered beside the existing
//! sort, and recorded through
//! [`pdfcer_core::edit::EditSession::add_review_state`].
//!
//! Everything this feature does that is not a `Vec` of `String`s or an
//! `egui::Ui` call lives here, in one file, for
//! [`super::filter`]'s reason: the interesting decisions are all
//! **classifications**, and a classification is only testable when it is
//! separable from the widget that shows it.
//!
//! ---
//!
//! ## ★★★ FACT ONE — A STATUS IS **APPENDED**, NOT SET
//!
//! This is the fact the whole module is shaped around, and it is the engine's
//! own, twice over (`edit.rs:26804`, `add_review_state`):
//!
//! > *"★★ THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES —
//! > §12.5.6.3 puts it on a **separate** `/Text` annotation that points at the
//! > reviewed one through `/IRT`, and says so with a `shall`. That is why this
//! > verb returns a new `ObjId` rather than mutating the target, and why
//! > nothing about the target changes."*
//!
//! > *"★★ AND A SECOND STATUS CHAINS ONTO THE FIRST, PER AUTHOR — The clause's
//! > last sentence is also a `shall`: 'Additional state changes shall be made
//! > by adding text annotations **in reply to the previous reply** for a given
//! > user.' So this verb walks the `/IRT` graph rooted at `target`, keeps the
//! > annotations whose `/T` matches `author`, and attaches to the deepest one —
//! > building a per-author chain rather than a star."*
//!
//! ⇒ **The file holds a log, not a field**, and three things follow that would
//! each be wrong under the other reading:
//!
//! 1. [`Statuses`] is `target → Vec<Recorded>` — **one entry per reviewer**,
//!    never one value per comment. Two people may hold opposite views of the
//!    same mark and both are in the file.
//! 2. Each entry carries [`Recorded::depth`], and the row prints it whenever it
//!    exceeds one. A panel that showed only the tip would be indistinguishable
//!    from a panel over a format that stores a mutable field, which is exactly
//!    what §12.5.6.3 is not.
//! 3. The control is called **Record status**, and its action is
//!    [`crate::app::actions::Action::RecordReviewState`]. Nothing in this
//!    feature is named *set*.
//!
//! ### ★★ There is deliberately NO resolver in the engine, and that is a
//! decision this module inherits rather than works around
//!
//! > *"Deciding which of several state annotations is current is left to the
//! > caller, as `pdfcer-gui` asked ('Give us the annotations and the keys; we
//! > will pick') — and the standard supports that: it says **nothing whatever**
//! > about ordering or currency. … `/M` is not required and empirically ties,
//! > so a `/M`-sorted resolver would be guessing. Shipping none is
//! > spec-correct."*
//!
//! So this module picks, and it picks the **only** ordering the standard does
//! define: the `/IRT` chain. The tip of a reviewer's chain is their current
//! status because §12.5.6.3 says each change replies to the previous one — not
//! because it is the newest by `/M`, which would be the guess the engine
//! refused to make. [`read`] implements exactly the walk `add_review_state`'s
//! own `deepest_state_for_author` implements, including its `MAX_CHAIN` bound,
//! and for the same reason: *"a `/IRT` cycle is legal syntax … and this must
//! terminate on a malformed file rather than hang."*
//!
//! ---
//!
//! ## ★★★ FACT TWO — THE ENGINE READS THE STRINGS **VERBATIM**
//!
//! `Annotation::state` (`annot.rs:480`) and `::state_model` (`annot.rs:492`)
//! are `Option<String>`, and the field's own doc says why it is not an enum:
//!
//! > *"Neither key carries a 'shall be one of' anywhere in either edition, so a
//! > value outside Table 171's vocabulary is **unhandled, not illegal**.
//! > Reporting it verbatim is therefore reading the file rather than tolerating
//! > it. `ReviewState` is the closed set pdfcer AUTHORS; this is the open set
//! > it reads."*
//!
//! ⇒ **The vocabulary is this shell's to present**, and an unrecognised value
//! must be *shown*, never normalised to the nearest thing pdfcer knows.
//! [`StateReading`] is that presentation, and it takes
//! [`crate::text::buttonaction`]'s posture rather than inventing a second one —
//! the reasoning is already written and reusing it is what keeps the two
//! surfaces saying the same thing about the same problem:
//!
//! | reading | what it means | what the row offers |
//! |---|---|---|
//! | [`StateReading::Modelled`] | Table 171's value, in Table 171's model | show it; **Record status** continues the same vocabulary |
//! | [`StateReading::Unmodelled`] | `/StateModel` is one pdfcer **authors**; `/State` is not in its vocabulary | show both verbatim; **Record status** still continues **that model** |
//! | [`StateReading::Foreign`] | `/StateModel` is a vocabulary pdfcer **will not author** | show both verbatim; pdfcer offers **nothing in it**, and says what it will do instead |
//! | [`StateReading::ModelMissing`] | `/State` present, `/StateModel` absent — Table 171's one non-conforming combination | show it as the malformed thing it is |
//!
//! ★★ **Collapsing `Unmodelled` into `Foreign` would grey a row pdfcer writes
//! happily.** A `/StateModel` of `Review` carrying a `/State` of `Deferred` is
//! a document pdfcer can add its own `Accepted` to — same model, same chain
//! rule, same file. Saying "pdfcer does not author this vocabulary" about it
//! would be this shell claiming an incapacity it does not have, which is the
//! failure `crate::text::buttonaction`'s four-state table exists to prevent.
//!
//! ★ And the difference is **visible**: [`StateReading::authorable_model`]
//! answers `Some` for the first three and `None` for `Foreign`, and that is the
//! one thing the row's note is keyed off.
//!
//! ---
//!
//! ## ★★ R8b — THE STATUS IS OFF-CANVAS, ALWAYS
//!
//! *"Fuzzy, never sneaky."* A review status is a **disclosure about a
//! comment**, not part of the comment's appearance, so it is drawn in this
//! panel and in the status row and **nowhere else**. A mark whose status is
//! `Rejected` is drawn on the page exactly as the file will draw it — no
//! badge, no tint, no dashed outline. The clause this is quoting is the
//! operator's own: *"the nagging and red flagging in the original GUI made for
//! a lot of extra bugs in the visibility when editing."*
//!
//! `crate::canvas::notepopup`'s header already refuses to show `/State` for the
//! same reason, and stays untouched by this Pass.
//!
//! ---
//!
//! ## ★ WHAT THE PANEL LOOKS LIKE AFTERWARDS, AND THE ROW NOBODY ORDERED
//!
//! A status annotation is a `/Text` with `/IRT`, `/State`, `/StateModel`, a
//! `/T` — and a **deliberately empty** `/Contents`: *"A status is not a
//! comment, and inventing 'Accepted' as the body would put a sentence in the
//! operator's comment list that the operator never wrote."*
//!
//! ⇒ The moment a status is recorded, a **blank row** appears in the Comments
//! list. Excluding it was considered and rejected: this panel's founding rule
//! is that *nothing is silently omitted*, and each of its three existing
//! exclusions is counted and disclosed in numbers. A fourth, silent one would
//! be the panel deciding what the document contains. So the row stays and
//! [`crate::text::reviewstate::row_is_a_status`] names it — and, because it is
//! a status rather than a comment, it is the one row that offers no **Record
//! status** control of its own (R83: an affordance nobody could want).
//!
//! ---
//!
//! ## Where the pieces live
//!
//! | | |
//! |---|---|
//! | reading the document | [`read`] → [`Statuses`] |
//! | classifying one value | [`StateReading::of`] |
//! | narrowing the list | [`StatusChoice`], [`Statuses::keeps`], [`narrow`] |
//! | the chooser beside the sort | [`status_strip`] |
//! | the row's status lines and its control | [`row_status`] |
//!
//! ## ★★ Every test below has been FALSIFIED, and one of them was vacuous
//!
//! Twenty-one guards, twenty-one mutations, one at a time, each restored from a
//! byte copy — never `git checkout`, because four other agents held uncommitted
//! work in this tree the day this was written. All twenty-one went red.
//!
//! ★★★ **The sweep earned its cost on the first run.**
//! `a_status_with_no_irt_reaches_no_target_but_is_still_read` asserted
//! `s.on(1).is_empty()` and stayed **green** when [`walk_up`] was broken to make
//! an `/IRT`-less status its own target — because that answer lands on
//! annotation *2*, and the assertion was only ever looking at *1*. A negative
//! assertion aimed at one address cannot see a wrong answer given at another.
//! It now asserts over the whole index. That test is the reason the rule is
//! *break the guard and watch it go red*, not *write the guard and watch it
//! pass*.
//!
//! ★ Two mutations are recorded in that test suite for a second reason: the
//! first fixture for [`MAX_CHAIN`] could not reach the bound at all (both
//! statuses in a two-cycle are superseded, so the walk never runs), and the
//! `#[non_exhaustive]` note on [`OFFERED`] exists because no `match` here can be
//! made to fail when the engine adds a variant.
//!
//! The filter *state* lives on [`super::filter::Filter`] rather than here, so
//! that **Show all** clears it and [`super::filter::Filter::is_narrowing`]
//! counts it — the two properties the panel's whole disclosure discipline hangs
//! off. What could **not** move there is the predicate: see
//! [`super::filter::Filter::status`] for why `keeps` cannot answer it.

use std::collections::{BTreeMap, BTreeSet};

use pdfcer_core::annot::page_annotations;
use pdfcer_core::edit::ReviewState;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::text::reviewstate as t;

use super::model::CommentRow;

/// The region the per-row **Record status** control publishes.
///
/// Published for the **first row that offers it** and no other, exactly as
/// [`super::REGION_EDIT`] is — one name, one rectangle, and the first row is
/// the only deterministic choice on a list whose length depends on the file.
pub const REGION_RECORD: &str = "comments.record_status"; // ui-text-exempt: trace region name, never displayed

/// The region the status chooser in the filter strip publishes.
pub const REGION_STATUS_FILTER: &str = "comments.status_filter"; // ui-text-exempt: trace region name, never displayed

/// The seven states pdfcer authors, in the order the chooser offers them.
///
/// `Review`'s five first, then `Marked`'s two, because the `Review` model is
/// what a comment workflow uses and `Marked` is a two-value tick that Acrobat
/// surfaces separately. Grouped rather than alphabetical: the two models are
/// different vocabularies (`annot.rs:472-479` — *"a caller that wants the
/// effective state must read both fields together"*) and an alphabetical list
/// would interleave them into one seven-item menu that implies they are
/// alternatives within a single scale.
///
/// ⚠ **Nothing here can be compiler-exhaustive, and the reason is worth
/// stating rather than assuming.** [`ReviewState`] is `#[non_exhaustive]`, so a
/// `match` over it must carry a wildcard and an array over it must be written
/// by hand — a variant the engine adds cannot fail to compile here, and a unit
/// test that iterates *this* array cannot notice either. The unit test
/// `every_authored_state_is_recognised_as_modelled` pins the **count** at
/// Table 171's seven, which turns a silent addition into a failing assertion;
/// the instrument that actually reports the engine growing one is
/// `tools/gates/check-engine-api-drift.py`, which watches variants as well as
/// verbs and exists because two earlier gates watched only `EditSession`.
pub const OFFERED: [ReviewState; 7] = [
    ReviewState::Accepted,
    ReviewState::Rejected,
    ReviewState::Cancelled,
    ReviewState::Completed,
    ReviewState::None,
    ReviewState::Marked,
    ReviewState::Unmarked,
];

/// How far a `/IRT` chain is walked before it is abandoned.
///
/// The engine's own bound, by the same number and for the same stated reason:
/// *"a `/IRT` cycle is legal syntax (nothing in §12.5.6.2 forbids one) and this
/// must terminate on a malformed file rather than hang"*
/// (`edit.rs`, `deepest_state_for_author`). Matching it rather than picking a
/// different number matters: a shell that walked further than the engine would
/// show a history the engine will refuse to extend.
const MAX_CHAIN: usize = 64;

/// **What one `/State` + `/StateModel` pair says**, classified but never
/// normalised.
///
/// See the module header's table for the four readings and for why the middle
/// two are not one. Every variant keeps the file's own bytes, so
/// [`Self::raw`] can always answer *what does the document literally say*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateReading {
    /// Table 171's value **in Table 171's model for that value** — the closed
    /// set [`ReviewState`] holds, which is exactly what pdfcer authors.
    ///
    /// ★ The model has to agree. `/State (Accepted) /StateModel (Marked)` is
    /// **not** this variant: `Accepted` belongs to `Review`
    /// ([`ReviewState::model`] derives it, and `add_review_state` writes the
    /// derived one so *"the one non-conforming combination cannot be
    /// expressed"*), so a file pairing it with `Marked` is saying something
    /// pdfcer would not write. It reads as [`Self::Unmodelled`] — the value is
    /// unrecognised **in the model the file put it in**, which is the honest
    /// statement and the one that keeps the row's offer correct.
    Modelled(ReviewState),
    /// `/StateModel` is `Review` or `Marked` — a vocabulary pdfcer **authors**
    /// — and `/State` is not one of that model's values.
    ///
    /// Named, shown verbatim, and the row still offers to record: pdfcer can
    /// add its own status **in the same model**, which is the entire content of
    /// the `Unmodelled`-versus-`Foreign` distinction here.
    Unmodelled {
        /// `/State`, exactly as the file decoded.
        state: String,
        /// `/StateModel`, exactly as the file decoded.
        model: String,
    },
    /// `/StateModel` is neither of §12.5.6.3's two — a vocabulary pdfcer will
    /// not author.
    ///
    /// Legal: neither edition says *shall be one of*. Both values are shown and
    /// **nothing is offered in that model**; the row says what recording will
    /// do instead, which is to start a `Review` history beside it.
    Foreign {
        /// `/State`, exactly as the file decoded.
        state: String,
        /// `/StateModel`, exactly as the file decoded.
        model: String,
    },
    /// `/State` present, `/StateModel` absent.
    ///
    /// Table 171's **one non-conforming combination** —
    /// `Annotation::state_model`: *"`/StateModel` is 'Required if `State` is
    /// present, otherwise optional'. The converse does not hold … the only
    /// non-conforming combination is `/State` present with this absent."*
    ///
    /// Surfaced rather than repaired, the same way
    /// [`super::model::CommentRow::subtype`] surfaces a missing `/Subtype`.
    /// Guessing the model would decide whether `Marked` means *ticked* or is
    /// an unrecognised `Review` value, and the file gives no basis for either.
    ModelMissing(String),
}

impl StateReading {
    /// **Classify one pair.** `state` is `/State`; `model` is `/StateModel`.
    ///
    /// Matching is **exact and case-sensitive**, because Table 171's values are
    /// text strings rather than names and `ReviewState::as_str` writes exactly
    /// those bytes. `accepted` is therefore [`Self::Unmodelled`] and is shown
    /// as the file wrote it — which is the whole posture of this feature: a
    /// case-folding match would be this shell normalising a value the engine
    /// deliberately did not.
    #[must_use]
    pub fn of(state: &str, model: Option<&str>) -> Self {
        let Some(model) = model else {
            return Self::ModelMissing(state.to_owned());
        };
        // Filtered by the model FIRST, so a `Review` value paired with the
        // `Marked` model reads as unrecognised-in-that-model rather than as
        // modelled — see `Self::Modelled`'s note.
        let known = OFFERED
            .iter()
            .find(|s| s.model() == model && s.as_str() == state);
        match (known, model) {
            (Some(s), _) => Self::Modelled(*s),
            // `model()` is the authority on which models pdfcer authors, so
            // the set is derived from the enum rather than written out again —
            // a third model added to `ReviewState` would land here without an
            // edit.
            (None, m) if OFFERED.iter().any(|s| s.model() == m) => Self::Unmodelled {
                state: state.to_owned(),
                model: m.to_owned(),
            },
            (None, m) => Self::Foreign {
                state: state.to_owned(),
                model: m.to_owned(),
            },
        }
    }

    /// **What the document literally says** in `/State`.
    ///
    /// The key the status filter matches on, so the chooser can offer a value
    /// pdfcer has never heard of and filtering to it works — which is the
    /// operator-facing consequence of the engine reading these keys verbatim.
    #[must_use]
    pub fn raw(&self) -> &str {
        match self {
            Self::Modelled(s) => s.as_str(),
            Self::Unmodelled { state, .. }
            | Self::Foreign { state, .. }
            | Self::ModelMissing(state) => state,
        }
    }

    /// **The `/StateModel` pdfcer would author into, given this reading** —
    /// `None` for [`Self::Foreign`] alone.
    ///
    /// ★★★ This is where the `Unmodelled`/`Foreign` distinction actually
    /// **bites**, and it is the reason the two are separate variants rather
    /// than one "unrecognised". `Some` means *pdfcer can continue this
    /// vocabulary*; `None` means it cannot, and the row says so instead of
    /// pretending. [`crate::text::buttonaction`]'s *"offer to replace"* versus
    /// *"offer nothing"*, translated to a verb that appends.
    ///
    /// [`Self::ModelMissing`] answers `Some("Review")`: the file did not say
    /// which vocabulary, so nothing is being continued and pdfcer records in
    /// its own default. That is a guess about **what to write next**, not about
    /// what the file means — the reading itself still says the model is
    /// missing.
    #[must_use]
    pub fn authorable_model(&self) -> Option<&str> {
        match self {
            Self::Modelled(s) => Some(s.model()),
            Self::Unmodelled { model, .. } => Some(model),
            Self::Foreign { .. } => None,
            Self::ModelMissing(_) => Some(ReviewState::Accepted.model()),
        }
    }

    /// The sentence the operator reads for this reading.
    ///
    /// A method rather than a `match` at the call site because two surfaces
    /// draw it — the row and the *this row is a status* caption — and a second
    /// copy is a second place for a variant to be forgotten.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Modelled(s) => t::state_name(*s).to_owned(),
            Self::Unmodelled { state, model } => t::state_unmodelled(state, model),
            Self::Foreign { state, model } => t::state_foreign(state, model),
            Self::ModelMissing(state) => t::state_without_model(state),
        }
    }
}

/// **One reviewer's current status on one comment**, plus how much history
/// stands behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// `/T` on the state annotation — the reviewer, when the file names one.
    ///
    /// `None` is not *anonymous*: `add_review_state` chains on `/T` equality
    /// (`edit.rs`, `deepest_state_for_author`), so an unnamed status can never
    /// be continued and always stands alone. [`crate::text::reviewstate`]'s
    /// [`crate::text::reviewstate::row_status_unsigned`] says exactly that.
    pub who: Option<String>,
    /// What that reviewer's most recent status says.
    pub reading: StateReading,
    /// **How many statuses this reviewer has recorded on this comment**, the
    /// tip included. `1` for a first status.
    ///
    /// The number [`pdfcer_core::edit::ReviewStateAdded::chain_depth`] reports
    /// after a write, computed here for what is already in the file. Printed
    /// whenever it exceeds one — see the module header's point 2.
    pub depth: usize,
}

/// **Every review status in the document**, indexed two ways.
///
/// Built once per frame by [`read`], for [`super::model::ce_dimension_annots`]'
/// reason: it is one walk of the page tree, and asking it per row would make
/// the panel quadratic in a document's annotation count.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Statuses {
    /// Reviewed annotation → one entry per reviewer, tip of that reviewer's
    /// chain. Sorted for a stable draw; see [`read`].
    by_target: BTreeMap<ObjId, Vec<Recorded>>,
    /// Annotation that **is** a status → what it says. The index behind the
    /// *this row is a status* caption; see the module header.
    is_status: BTreeMap<ObjId, StateReading>,
}

impl Statuses {
    /// What is recorded on one annotation, newest-per-reviewer.
    ///
    /// Empty for a row with no id: `/IRT` is an indirect reference, so nothing
    /// can point at an annotation written as a direct dictionary — see
    /// [`super::model::CommentRow::id`].
    #[must_use]
    pub fn on(&self, id: Option<ObjId>) -> &[Recorded] {
        id.and_then(|id| self.by_target.get(&id))
            .map_or(&[], Vec::as_slice)
    }

    /// What this annotation says, when the annotation **is** a status.
    #[must_use]
    pub fn as_status(&self, id: Option<ObjId>) -> Option<&StateReading> {
        id.and_then(|id| self.is_status.get(&id))
    }

    /// **Every distinct `/State` value the chooser should offer**, in the
    /// file's own spelling, alphabetically.
    ///
    /// Derived from the document rather than from [`OFFERED`], and that is the
    /// decision: the engine reads these keys verbatim, so a file may carry
    /// `Deferred` and a reviewer must be able to filter to it. A menu of
    /// pdfcer's own seven would be this shell telling the operator their
    /// document does not contain what it contains.
    ///
    /// ★ Only **tip** values appear. A value that occurs solely in the middle
    /// of somebody's chain is a status they have since superseded, and offering
    /// it would filter to zero rows — an entry that can only disappoint.
    ///
    /// The same rule [`super::filter::subtypes`] follows for `/Subtype`: the
    /// file's own spelling, never a friendly relabelling.
    #[must_use]
    pub fn values(&self) -> Vec<String> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for recorded in self.by_target.values() {
            for r in recorded {
                seen.insert(r.reading.raw());
            }
        }
        seen.into_iter().map(str::to_owned).collect()
    }

    /// **Does one row survive a status filter?**
    ///
    /// Split from [`narrow`] so the rule can be asserted against a single row,
    /// exactly as [`super::filter::Filter::keeps`] is.
    ///
    /// ★ `Is` matches if **any** reviewer's current status is that value, not
    /// if every one is. Two people may disagree, and a reviewer asking *"what
    /// has been rejected"* wants the mark somebody rejected even though
    /// somebody else accepted it — the conservative direction on a work list is
    /// to show the thing that may need attention.
    #[must_use]
    pub fn keeps(&self, row: &CommentRow, choice: Option<&StatusChoice>) -> bool {
        match choice {
            None => true,
            Some(StatusChoice::Unrecorded) => self.on(row.id).is_empty(),
            Some(StatusChoice::Is(value)) => {
                self.on(row.id).iter().any(|r| r.reading.raw() == value)
            }
        }
    }
}

/// **What the reviewer has narrowed the status to.**
///
/// Two entries plus the absent case, and the third is the one the filter exists
/// for — see [`crate::text::reviewstate::filter_status_unrecorded`]. `None` on
/// [`super::filter::Filter::status`] is *any status, and none*.
///
/// ★ [`Self::Unrecorded`] is **not** a `/State` of `None`. Table 171 makes
/// `None` a writable value in the `Review` model and `Annotation::state`'s doc
/// is explicit that it *"is a writable value, not a spelling of 'the key is
/// absent'"*. A reviewer who withdrew their status has done something; a
/// comment nobody has looked at has not. Folding them would hide the second
/// behind the first, and the second is the whole work list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusChoice {
    /// Only comments carrying **no status at all**.
    Unrecorded,
    /// Only comments where some reviewer's current status is this exact
    /// `/State` string.
    Is(String),
}

/// **Read every review status in the document.**
///
/// `graph` must be the session view, not the file on disk — this panel's rule,
/// stated in [`super::model`]'s header: a status recorded thirty seconds ago
/// and not yet saved must be on the row.
///
/// # The algorithm, and why it is the engine's rather than a simpler one
///
/// 1. Collect every annotation on every page that carries a `/State` **and**
///    has an object id. An id is required because the chain is built out of
///    `/IRT` references, which are indirect by construction.
/// 2. A state annotation is a **tip** when no other state annotation *by the
///    same `/T`* replies to it. That is §12.5.6.3's *"in reply to the previous
///    reply for a given user"* read backwards, and it is the only definition of
///    "current" the standard supplies — see the module header on why `/M` is
///    not used.
/// 3. From each tip, walk `/IRT` **upward** through same-author state
///    annotations, counting them, until the parent is not one. That parent is
///    the annotation being reviewed.
///
/// ★ Same-author comparison treats an absent `/T` as **never equal**, matching
/// `deepest_state_for_author`'s `a.title.as_deref() == Some(author)` exactly.
/// An unsigned status therefore neither continues a chain nor is continued by
/// one, which is a fact about the file and is what
/// [`crate::text::reviewstate::row_status_unsigned`] tells the operator.
///
/// ★ A state annotation with **no** `/IRT` describes nothing — §12.5.6.3 puts
/// the state on an annotation *"that refers to the original annotation by means
/// of its `IRT` entry"* — so it is recorded in [`Statuses::is_status`], where
/// it will still name itself on its own row, and reaches no target. Dropping it
/// silently would be the panel hiding a malformed annotation.
///
/// # Cost
///
/// One walk of every page's `/Annots` — the same walk
/// [`super::model::collect`] does — plus a pass over the state annotations
/// alone, which on any real document is a handful. The tip test is quadratic in
/// the number of **state annotations**, not in the number of annotations; a
/// document with a thousand statuses does a million cheap `Option<ObjId>`
/// comparisons once a frame, and a document with none does nothing at all.
#[must_use]
pub fn read<G: ObjectGraph + ?Sized>(graph: &G, pages: &[Page]) -> Statuses {
    let states: Vec<StateAnnot> = pages
        .iter()
        .flat_map(|page| page_annotations(graph, page.id))
        .filter_map(|a| {
            // Both are required, and neither is a judgement: `/State` is what
            // makes this a status at all, and an object id is what lets a
            // `/IRT` — an indirect reference by construction — name it.
            let (id, state) = (a.id?, a.state.as_deref()?);
            Some(StateAnnot {
                id,
                who: a.title.as_deref().map(str::trim).map(str::to_owned),
                irt: a.in_reply_to,
                reading: StateReading::of(state, a.state_model.as_deref()),
            })
        })
        .collect();
    let read = assemble(&states);
    // ★★★ THE ONLY ORACLE FOR THIS FEATURE THAT A SCREENSHOT CANNOT GIVE.
    //
    // A review status is **invisible on the page** by construction — R8b puts it
    // off-canvas, and the annotation carrying it has an empty `/Contents` — so a
    // driven run of `tools/ui-verify` has nothing to look at except this line
    // and the rectangles the two controls publish. Three numbers, because the
    // three failures they separate all look identical from outside:
    //
    // | field | the question | the failure it catches |
    // |---|---|---|
    // | `states` | how many `/State` annotations does the document carry | the panel read the wrong revision, or read nothing |
    // | `reviewed` | how many COMMENTS have a status | every chain collapsed to its own target, or none reached one |
    // | `values` | how many distinct statuses can be filtered to | the chooser is empty on a document that has statuses |
    //
    // ★ `reviewed` is the one worth having. `states` and `values` would both be
    // right on a build whose `/IRT` walk was broken; `reviewed` is the number
    // that changes when a chain fails to reach the comment it describes, and
    // that is the failure the engine calls invisible.
    //
    // `comments-status`, not `comments-panel`: the panel's own summary line owns
    // that token, and `TraceLog::last(name)` matches the FIRST token, so two
    // lines under one name would shadow each other. Same rule
    // `tools/gates/check-trace-names.py` enforces against the edit funnel.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "comments-status states={} reviewed={} values={}",
            states.len(),
            read.by_target.len(),
            read.values().len()
        )
    });
    read
}

/// **One `/State`-carrying annotation**, reduced to the four facts the chain
/// walk and the row need.
///
/// # ★ Why the walk does not run over `pdfcer_core::annot::Annotation`
///
/// Two reasons, and the second is the one that matters.
///
/// 1. `Annotation` carries twenty-four fields, none of which but these four is
///    read here, and it derives no `Default` — so every fixture would be a
///    twenty-four-field literal that changes whenever the engine's read model
///    grows a field. That is a test suite bound to the engine's *shape* rather
///    than to its *behaviour*.
/// 2. ★★ **The `/T` trim happens exactly once**, on the way in.
///    `deepest_state_for_author` compares `/T` bytes as the file carries them,
///    and this panel's `super::keeps_author_name` trims — so a chain rule
///    written against the raw field and a display rule written against the
///    trimmed one would disagree about whether `"Ken "` and `"Ken"` are one
///    person. Normalising at the boundary makes that unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StateAnnot {
    /// The state annotation's own object id.
    id: ObjId,
    /// `/T`, trimmed. `None` when absent — see [`same_author`].
    who: Option<String>,
    /// `/IRT` — what this status is a reply to.
    irt: Option<ObjId>,
    /// What `/State` + `/StateModel` say.
    reading: StateReading,
}

/// Build the two indexes from the document's state annotations.
///
/// Separate from [`read`] so the whole classification can be asserted without a
/// fixture document — [`super::model`]'s seam, at this module's scale.
fn assemble(states: &[StateAnnot]) -> Statuses {
    let mut out = Statuses::default();
    for s in states {
        out.is_status.insert(s.id, s.reading.clone());
        // A tip is a status nobody **by the same name** has replied to.
        // `same_author` guards the whole test: an unsigned status is never
        // anybody's parent, so it can never be superseded.
        if states
            .iter()
            .any(|t| t.irt == Some(s.id) && same_author(t, s))
        {
            continue;
        }
        let (target, depth) = walk_up(states, s);
        let Some(target) = target else {
            continue;
        };
        out.by_target.entry(target).or_default().push(Recorded {
            who: s.who.clone().filter(|w| !w.is_empty()),
            reading: s.reading.clone(),
            depth,
        });
    }

    // ★ Sorted for a STABLE draw. The input is in page-then-`/Annots` order,
    // which is stable in itself — but two reviewers' tips arrive in whatever
    // order each happened to be placed, and a list of statuses that reshuffled
    // between frames reads as the panel flickering. The rule is
    // `super::filter::apply`'s, restated: an unsigned entry sorts LAST, because
    // a reader scanning for a name should not have to read past the rows that
    // carry none.
    for recorded in out.by_target.values_mut() {
        recorded.sort_by_key(|r| {
            let who = r.who.clone().unwrap_or_default();
            (
                who.is_empty(),
                who.to_lowercase(),
                r.reading.raw().to_owned(),
            )
        });
    }
    out
}

/// Whether two state annotations belong to the same reviewer's chain.
///
/// An absent `/T` is **never** equal to anything, including another absent one
/// — `deepest_state_for_author` compares `a.title.as_deref() == Some(author)`,
/// and `author` is a `&str`, so `None` can never match. Mirrored exactly rather
/// than improved: a shell that chained unsigned statuses would show a history
/// the engine will not extend.
fn same_author(a: &StateAnnot, b: &StateAnnot) -> bool {
    a.who.is_some() && a.who == b.who
}

/// Walk a chain from its tip to the annotation it reviews.
///
/// Returns the reviewed annotation and the chain's depth (`1` for a lone
/// status). `None` when the tip has no `/IRT` at all — a status that refers to
/// nothing describes nothing, and §12.5.6.3 defines the state as living on an
/// annotation *"that refers to the original annotation by means of its `IRT`
/// entry"*.
///
/// The walk is depth-bounded at [`MAX_CHAIN`]: a bounded stop rather than a
/// hang, on a `/IRT` cycle the standard does not forbid.
fn walk_up(states: &[StateAnnot], tip: &StateAnnot) -> (Option<ObjId>, usize) {
    let mut current = tip;
    let mut depth = 1usize;
    loop {
        let Some(parent_id) = current.irt else {
            return (None, depth);
        };
        let parent = states
            .iter()
            .find(|p| p.id == parent_id && same_author(p, tip));
        match parent {
            Some(p) if depth < MAX_CHAIN => {
                current = p;
                depth += 1;
            }
            // Either the parent is not this reviewer's status — so it is the
            // annotation being reviewed — or the chain is longer than the
            // engine itself will walk, in which case reporting the deepest
            // point reached is the honest answer and matches what
            // `add_review_state` would attach to.
            _ => return (Some(parent_id), depth),
        }
    }
}

/// **Narrow a list of rows by status**, after [`super::filter::apply`] has
/// narrowed and ordered it.
///
/// ★ Ordering is deliberately untouched here. [`super::filter::apply`] owns it,
/// and a second sort would be a second answer to a question that already has
/// one — the failure [`super::model`]'s header names for the *list* ordering
/// and the same argument applies to the *filter* pipeline.
#[must_use]
pub fn narrow(
    rows: Vec<CommentRow>,
    statuses: &Statuses,
    choice: Option<&StatusChoice>,
) -> Vec<CommentRow> {
    rows.into_iter()
        .filter(|r| statuses.keeps(r, choice))
        .collect()
}

/// **The status chooser**, drawn beneath the existing filter strip.
///
/// # ★ Built from the document, not from [`OFFERED`]
///
/// [`Statuses::values`] carries the argument: the engine reads `/State`
/// verbatim, so the values a reviewer needs to filter by are whatever the file
/// contains. What is offered here and what may be **recorded** are therefore
/// two different lists, and deliberately so — [`record_control`] offers
/// pdfcer's seven because those are the ones it can write.
///
/// # ★★ Drawn only when the document has any status at all
///
/// R9's shape applied to a control that would do nothing: on a document nobody
/// has reviewed, this chooser's only entries are *Any status* and *No status
/// recorded*, and the second selects every row. It is not greyed — greying
/// implies a temporary condition — it is simply not there, and it appears the
/// moment the first status is recorded, which also makes its arrival a wordless
/// statement that something happened.
pub fn status_strip(ui: &mut egui::Ui, statuses: &Statuses, chosen: &mut Option<StatusChoice>) {
    let values = statuses.values();
    if values.is_empty() {
        // ★ …and the filter is LIFTED, not merely hidden. A chooser that
        // vanished while still narrowing would leave rows hidden with no
        // control to restore them — the trap version of R9, and the exact
        // failure `super::filter`'s *Show all* exists to prevent.
        *chosen = None;
        return;
    }
    let selected = match chosen.as_ref() {
        None => t::filter_status_label().to_owned(),
        Some(StatusChoice::Unrecorded) => t::filter_status_unrecorded().to_owned(),
        Some(StatusChoice::Is(v)) => v.clone(),
    };
    ui.horizontal_wrapped(|ui| {
        let combo = egui::ComboBox::from_id_salt("comments-filter-status") // ui-text-exempt: internal widget id, never displayed
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.selectable_value(chosen, None, t::filter_status_any());
                ui.selectable_value(
                    chosen,
                    Some(StatusChoice::Unrecorded),
                    t::filter_status_unrecorded(),
                );
                for value in &values {
                    ui.selectable_value(chosen, Some(StatusChoice::Is(value.clone())), value);
                }
            })
            .response;
        crate::diag::ui_rect_visible(REGION_STATUS_FILTER, combo.rect, ui.clip_rect());
        combo.on_hover_text(t::filter_status_label());
    });
}

/// **One row's status lines, and the control that records another.**
///
/// Called from [`super::body`]'s row loop, after [`super::row`] has drawn the
/// comment itself, so the status sits with the row's other disclosures and
/// above nothing.
///
/// # What is drawn, and the condition on each
///
/// | line | when |
/// |---|---|
/// | *this row is a status* | the annotation **is** a state annotation — see the module header on the blank row |
/// | one line per reviewer | that reviewer has a status on this comment |
/// | the history count | that reviewer has more than one |
/// | *no status recorded* | only under the **No status recorded** filter, where the row's emptiness is the answer to the question asked |
/// | the *Record status* chooser | the stance authors markup, the row has an id, and it is neither a ce dimension nor itself a status |
///
/// # ★★ Why the control is withheld in three cases, and each is R83
///
/// R83 — *an affordance that cannot be honoured is not drawn* — rather than R9,
/// with one exception noted below:
///
/// - **No object id.** `/IRT` is an indirect reference; a direct dictionary in
///   `/Annots` cannot be referred to. `super::delete_control` declines for the
///   same reason.
/// - **A ce dimension.** `add_review_state` routes through `add_reply`, whose
///   errors include `AnnotationIsCeDimension` — the engine refuses by name.
///   Rule 15: it is named as a ce dimension on the row and offered nothing that
///   would be refused.
/// - **The row is itself a status.** Nothing forbids it — §12.5.6.3 chains
///   states onto states — but a status *about a status* is not a thing a
///   reviewer means to create, and the row already says what it is.
///
/// And the **stance** gate is R9, not R83, exactly as `super::delete_control`'s
/// is: a mode that does not author markup renders nothing rather than something
/// greyed, because a stance is not a temporary condition and the mode selector
/// is the visible explanation.
///
/// `controls_drawn` is tallied into the same counter the Delete button and the
/// note editor use, so `super::note::CommentsUi::writing_controls_drawn`
/// keeps its meaning — *how many writing controls the panel drew* — and the
/// headless assertion that Read offers none covers this control too, without
/// that test having to learn it exists.
pub fn row_status(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    statuses: &Statuses,
    ctx: RowStatusCtx<'_>,
) {
    // The row that IS a status, named before anything else on it: an operator
    // looking at a blank comment needs the explanation before they conclude the
    // panel is broken.
    if let Some(reading) = statuses.as_status(comment.id) {
        ui.label(
            egui::RichText::new(t::row_is_a_status(&reading.label()))
                .small()
                .weak(),
        );
    }

    let recorded = statuses.on(comment.id);
    for entry in recorded {
        let status = entry.reading.label();
        let line = match entry.who.as_deref().filter(|w| !w.is_empty()) {
            Some(who) => t::row_status_by(who, &status),
            None => t::row_status_unsigned(&status),
        };
        ui.label(egui::RichText::new(line).small().weak());
        // ★ Only when there IS history. Printing "1 status recorded" beside
        // every row would be a caption with the same information content as
        // nothing at all — this panel's rule for its other four disclosures.
        if entry.depth > 1 {
            ui.label(
                egui::RichText::new(t::row_status_history(entry.depth))
                    .small()
                    .weak(),
            );
        }
    }

    // ★ Said only where the operator ASKED the question. Under the *No status
    // recorded* filter the emptiness is the answer and the caption confirms it;
    // on an unfiltered list it would repeat "nothing has happened here" on
    // forty rows.
    if recorded.is_empty() && ctx.unreviewed_asked {
        ui.label(egui::RichText::new(t::row_status_none()).small().weak());
    }

    // ★★ The `Foreign` note, and this is the one place the fourth reading
    // changes what the operator is told. Drawn when EVERY status on the row is
    // in a model pdfcer will not author — if any reading has an authorable
    // model, recording continues a vocabulary that is already here and there is
    // nothing to explain.
    if !recorded.is_empty()
        && recorded
            .iter()
            .all(|r| r.reading.authorable_model().is_none())
    {
        ui.label(
            egui::RichText::new(t::record_other_model_note())
                .small()
                .weak(),
        );
    }

    record_control(ui, comment, statuses, ctx);
}

/// What [`row_status`] needs from the frame around it.
///
/// A struct rather than four loose parameters, for [`super::RowSink`]'s reason:
/// a call site passing `true, false, &mut x, &mut y` positionally is one
/// transposition away from a control that draws in the wrong stance.
pub struct RowStatusCtx<'a> {
    /// Whether the current mode authors markup —
    /// `crate::canvas::tool::capabilities(..).author_markup`, asked once per
    /// frame by [`super::body`] and passed down.
    pub authoring: bool,
    /// Whether the operator has filtered to **No status recorded**, which is
    /// the only condition under which an unreviewed row says so.
    pub unreviewed_asked: bool,
    /// Whether [`REGION_RECORD`] has already been published this frame. One
    /// name, one row — see that constant.
    pub published: &'a mut bool,
    /// Tallied into `super::note::CommentsUi::writing_controls_drawn`.
    pub controls_drawn: &'a mut u32,
    /// The status one row asked to record, if any. One `Option` rather than a
    /// `Vec` for [`super::RowSink`]'s reason: two rows cannot be pressed in one
    /// frame.
    pub verb: &'a mut Option<(ObjId, ReviewState)>,
}

/// The **Record status** chooser for one row. See [`row_status`] for the five
/// conditions under which it is not drawn.
fn record_control(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    statuses: &Statuses,
    ctx: RowStatusCtx<'_>,
) {
    if !ctx.authoring || comment.is_ce_dimension {
        return;
    }
    let Some(id) = comment.id else {
        return;
    };
    if statuses.as_status(Some(id)).is_some() {
        return;
    }
    let mut chosen: Option<ReviewState> = None;
    let combo = egui::ComboBox::from_id_salt("comments-record-status") // ui-text-exempt: internal widget id, never displayed
        .selected_text(t::record_placeholder())
        .show_ui(ui, |ui| {
            // Grouped by model, with each model's own name above it. The two
            // are different vocabularies rather than seven points on one scale
            // — `annot.rs:472-479`: "a caller that wants the effective state
            // must read both fields together" — and a flat list would invite
            // the operator to read `Marked` as a sixth `Review` value.
            //
            // The headings are `ReviewState::model()`'s own words, which are
            // the strings the file carries, so they are not copy this shell
            // invented and there is nothing for the catalog to hold.
            let mut model: Option<&str> = None;
            for state in OFFERED {
                if model != Some(state.model()) {
                    model = Some(state.model());
                    ui.label(egui::RichText::new(state.model()).small().weak());
                }
                if ui.selectable_label(false, t::state_name(state)).clicked() {
                    chosen = Some(state);
                }
            }
        })
        .response;
    *ctx.controls_drawn += 1;
    // `ui_rect_visible`, not `ui_rect`: these rows live in a `ScrollArea` and a
    // control scrolled out of view still reports a rect. See
    // `super::REGION_EDIT`.
    if !*ctx.published {
        crate::diag::ui_rect_visible(REGION_RECORD, combo.rect, ui.clip_rect());
        *ctx.published = true;
    }
    combo.on_hover_text(t::record_tooltip());
    if let Some(state) = chosen {
        *ctx.verb = Some((id, state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One state annotation, named by object number, exactly as [`read`] would
    /// have reduced it.
    fn annot(
        num: u32,
        title: Option<&str>,
        irt: Option<u32>,
        state: &str,
        model: Option<&str>,
    ) -> StateAnnot {
        StateAnnot {
            id: ObjId::new(num, 0),
            who: title.map(str::trim).map(str::to_owned),
            irt: irt.map(|n| ObjId::new(n, 0)),
            reading: StateReading::of(state, model),
        }
    }

    /// The classification under test, run over hand-built statuses.
    ///
    /// [`assemble`] is called rather than re-implemented, which is the point of
    /// splitting it out of [`read`]: a helper that repeated the algorithm would
    /// be a test of the helper.
    fn statuses(annots: &[StateAnnot]) -> Statuses {
        assemble(annots)
    }

    fn row(num: u32) -> CommentRow {
        CommentRow {
            page_index: 0,
            id: Some(ObjId::new(num, 0)),
            subtype: "Square".to_owned(),
            is_ce_dimension: false,
            note: super::super::model::Note::Absent,
            author: None,
            modified: None,
            suppressed: false,
            appearance_unresolved: false,
            relation: None,
            in_reply_to: None,
        }
    }

    // ---------------------------------------------------------------------
    // The vocabulary: what is recognised, and what is shown rather than
    // normalised.
    // ---------------------------------------------------------------------

    /// ★★★ **THE CONTROL FOR THE NEGATIVE BELOW.**
    ///
    /// A test asserting *an unknown state is not normalised* is **vacuous if
    /// nothing is ever recognised** — a `StateReading::of` that returned
    /// `Unmodelled` for every input on earth would pass it. So every one of
    /// the seven values pdfcer authors is round-tripped through the classifier
    /// here, and this test is what gives the next one its meaning.
    ///
    /// The trailing count assertion is the one instrument this side of the
    /// boundary that can notice [`OFFERED`] falling behind: `ReviewState` is
    /// `#[non_exhaustive]`, so no `match` and no iteration over the array can
    /// be made to fail when the engine adds a variant. See [`OFFERED`].
    #[test]
    fn every_authored_state_is_recognised_as_modelled() {
        for state in OFFERED {
            assert_eq!(
                StateReading::of(state.as_str(), Some(state.model())),
                StateReading::Modelled(state),
                "{} in the {} model",
                state.as_str(),
                state.model()
            );
        }
        assert_eq!(
            OFFERED.len(),
            7,
            "Table 171 gives seven values in two models"
        );
    }

    /// ★★★ **An unrecognised value in a model pdfcer authors is SHOWN, not
    /// normalised** — and pdfcer still offers to continue that model.
    ///
    /// The `Unmodelled` half of `crate::text::buttonaction`'s table. Both
    /// halves are asserted: the value survives verbatim, **and**
    /// `authorable_model` is `Some`, because collapsing this into `Foreign`
    /// would grey a row pdfcer writes happily.
    #[test]
    fn an_unknown_value_in_a_known_model_is_shown_verbatim() {
        let reading = StateReading::of("Deferred", Some("Review"));
        assert_eq!(
            reading,
            StateReading::Unmodelled {
                state: "Deferred".to_owned(),
                model: "Review".to_owned(),
            }
        );
        assert_eq!(reading.raw(), "Deferred");
        assert!(reading.label().contains("Deferred"), "{}", reading.label());
        assert_eq!(
            reading.authorable_model(),
            Some("Review"),
            "pdfcer authors the Review model and must still offer to record in it"
        );
    }

    /// ★★ **An unknown MODEL is foreign, and pdfcer offers nothing in it.**
    ///
    /// The other half of the distinction. `authorable_model` is `None`, which
    /// is the single observable the row's note is keyed off.
    #[test]
    fn an_unknown_model_is_foreign_and_offers_nothing() {
        let reading = StateReading::of("Sealed", Some("Approval"));
        assert_eq!(
            reading,
            StateReading::Foreign {
                state: "Sealed".to_owned(),
                model: "Approval".to_owned(),
            }
        );
        assert_eq!(reading.authorable_model(), None);
        let label = reading.label();
        assert!(
            label.contains("Sealed") && label.contains("Approval"),
            "{label}"
        );
    }

    /// ★ **A Review value in the Marked model is not modelled**, because
    /// `ReviewState::model` derives the pairing and pdfcer cannot express this
    /// one.
    ///
    /// Asserted because the obvious implementation matches `/State` alone and
    /// would report `Modelled(Accepted)` for a file saying something pdfcer
    /// would never write.
    #[test]
    fn a_state_paired_with_the_wrong_model_is_unmodelled() {
        assert_eq!(
            StateReading::of("Accepted", Some("Marked")),
            StateReading::Unmodelled {
                state: "Accepted".to_owned(),
                model: "Marked".to_owned(),
            }
        );
    }

    /// ★ Matching is **exact**: a case variant is a value pdfcer did not decode.
    #[test]
    fn matching_is_case_sensitive() {
        assert!(matches!(
            StateReading::of("accepted", Some("Review")),
            StateReading::Unmodelled { .. }
        ));
    }

    /// ★ `/State` with no `/StateModel` — Table 171's one non-conforming
    /// combination — is surfaced rather than guessed at.
    #[test]
    fn a_state_without_a_model_is_named_as_such() {
        let reading = StateReading::of("Accepted", None);
        assert_eq!(reading, StateReading::ModelMissing("Accepted".to_owned()));
        assert_eq!(reading.raw(), "Accepted");
    }

    /// ★★ `None` is a **value**, not the absence of one, and the two are
    /// different entries the operator can act on.
    ///
    /// `Annotation::state`: *"`None` is a writable value, not a spelling of
    /// 'the key is absent'."* A row carrying `/State (None)` must not be
    /// selected by the **No status recorded** filter.
    #[test]
    fn a_state_of_none_is_recorded_and_not_unrecorded() {
        let doc = [annot(2, Some("Ken"), Some(1), "None", Some("Review"))];
        let s = statuses(&doc);
        assert_eq!(s.on(Some(ObjId::new(1, 0))).len(), 1);
        assert!(
            !s.keeps(&row(1), Some(&StatusChoice::Unrecorded)),
            "a withdrawn status is not an unreviewed comment"
        );
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("None".to_owned()))));
    }

    // ---------------------------------------------------------------------
    // The chain: appended, not set.
    // ---------------------------------------------------------------------

    /// ★★★ **A SECOND STATUS DOES NOT REPLACE THE FIRST — it chains onto it,
    /// and the panel reports the depth.**
    ///
    /// The property the whole feature is shaped around. Annotation 1 is the
    /// comment; 2 is Ken's first status; 3 replies to 2 with his second. Only
    /// the tip is shown, and `depth` is 2 — which is what
    /// `crate::text::reviewstate::row_status_history` prints so the operator
    /// cannot mistake a log for a field.
    #[test]
    fn a_reviewers_second_status_chains_and_the_depth_is_reported() {
        let doc = [
            annot(2, Some("Ken"), Some(1), "Rejected", Some("Review")),
            annot(3, Some("Ken"), Some(2), "Accepted", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 1, "one entry per reviewer, not one per status");
        assert_eq!(on[0].reading, StateReading::Modelled(ReviewState::Accepted));
        assert_eq!(on[0].depth, 2);
    }

    /// ★★ **Two reviewers are two entries, not one winner.**
    ///
    /// §12.5.6.3 chains per user, so the file genuinely holds both. A resolver
    /// that picked one would be inventing the currency rule the engine
    /// explicitly refused to ship.
    #[test]
    fn two_reviewers_are_both_reported() {
        let doc = [
            annot(2, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(3, Some("Jo"), Some(1), "Rejected", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 2);
        assert_eq!(on[0].who.as_deref(), Some("Jo"), "sorted by name");
        assert_eq!(on[1].who.as_deref(), Some("Ken"));
    }

    /// ★★ **An unsigned status never chains**, mirroring the engine's own
    /// `title.as_deref() == Some(author)`.
    ///
    /// Both statuses stay tips and both are reported, because neither can be
    /// the other's parent. A shell that chained them would show a history
    /// `add_review_state` will not extend.
    #[test]
    fn an_unsigned_status_never_chains() {
        let doc = [
            annot(2, None, Some(1), "Accepted", Some("Review")),
            annot(3, None, Some(2), "Rejected", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 1, "only the one whose /IRT points at the comment");
        assert_eq!(on[0].depth, 1);
        assert!(on[0].who.is_none());
        // The second one refers to a state annotation, not to the comment, so
        // it is reported against THAT.
        assert_eq!(s.on(Some(ObjId::new(2, 0))).len(), 1);
    }

    /// ★★ A `/IRT` **cycle** terminates rather than hanging, and the walk is
    /// what has to survive it.
    ///
    /// Legal syntax — nothing in §12.5.6.2 forbids one — and the engine bounds
    /// its own walk at 64 for exactly this reason.
    ///
    /// ★ **The obvious fixture does not test the bound**, which is worth
    /// recording because the first attempt was that fixture: two statuses
    /// pointing at each other are BOTH superseded, so [`assemble`] skips both
    /// before [`walk_up`] is ever called and removing [`MAX_CHAIN`] leaves the
    /// test green. Three are needed — 3 is a tip nobody replies to, and its
    /// chain runs into a 4↔5 loop that only the bound stops. Delete
    /// `depth < MAX_CHAIN` and this test does not fail; it **hangs**, which is
    /// the failure it is asserting the absence of.
    #[test]
    fn an_irt_cycle_terminates() {
        let doc = [
            annot(3, Some("Ken"), Some(4), "Accepted", Some("Review")),
            annot(4, Some("Ken"), Some(5), "Rejected", Some("Review")),
            annot(5, Some("Ken"), Some(4), "Completed", Some("Review")),
        ];
        let s = statuses(&doc);
        // 3 is the only tip: 4 is replied to by both 3 and 5, and 5 by 4.
        // The walk from 3 runs 4 → 5 → 4 → … and stops at the bound, reporting
        // the deepest point it reached — the honest answer, and the same place
        // `add_review_state` would attach to.
        let reported: usize = [4u32, 5]
            .iter()
            .map(|n| s.on(Some(ObjId::new(*n, 0))).len())
            .sum();
        assert_eq!(reported, 1, "the bounded walk reported exactly one target");
        let depth = [4u32, 5]
            .iter()
            .filter_map(|n| s.on(Some(ObjId::new(*n, 0))).first())
            .map(|r| r.depth)
            .next()
            .expect("the walk reported a target");
        assert_eq!(
            depth, MAX_CHAIN,
            "the walk stopped at the bound, not before"
        );
    }

    /// ★ A status with **no `/IRT`** describes nothing, and is still named on
    /// its own row rather than dropped.
    ///
    /// ★★★ **The first version of this test was VACUOUS, and the mutation
    /// sweep is what found it.** It asserted `s.on(1).is_empty()` — that the
    /// status did not reach annotation 1 — and stayed **green** when
    /// [`walk_up`] was broken to return `Some(current.id)` for an `/IRT`-less
    /// status, because that makes the status reach annotation *2* instead. A
    /// negative assertion aimed at one address cannot see a wrong answer given
    /// at another.
    ///
    /// ⇒ The assertion is now over the WHOLE index: this status reaches **no
    /// target at all**. `by_target` is reached directly rather than through
    /// [`Statuses::on`] for exactly that reason — `on` can only be asked about
    /// an id somebody already suspects.
    #[test]
    fn a_status_with_no_irt_reaches_no_target_but_is_still_read() {
        let doc = [annot(2, Some("Ken"), None, "Accepted", Some("Review"))];
        let s = statuses(&doc);
        assert!(
            s.by_target.is_empty(),
            "a status with no /IRT reached {:?}",
            s.by_target.keys().collect::<Vec<_>>()
        );
        assert!(
            s.as_status(Some(ObjId::new(2, 0))).is_some(),
            "the row must still be able to say what it is"
        );
    }

    /// **A status annotation knows it is one**, which is what stops the blank
    /// row it creates reading as a defect.
    #[test]
    fn a_status_annotation_is_identifiable_as_one() {
        let doc = [annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))];
        let s = statuses(&doc);
        assert_eq!(
            s.as_status(Some(ObjId::new(2, 0))),
            Some(&StateReading::Modelled(ReviewState::Accepted))
        );
        assert_eq!(s.as_status(Some(ObjId::new(1, 0))), None);
    }

    // ---------------------------------------------------------------------
    // The filter.
    // ---------------------------------------------------------------------

    /// **No status filter hides nothing** — the default this panel has always
    /// had.
    #[test]
    fn no_status_filter_keeps_every_row() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let rows = vec![row(1), row(5)];
        assert_eq!(narrow(rows, &s, None).len(), 2);
    }

    /// Filtering to a value keeps the comments carrying it and only those.
    #[test]
    fn filtering_to_a_status_keeps_only_those_comments() {
        let s = statuses(&[
            annot(3, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(4, Some("Ken"), Some(2), "Rejected", Some("Review")),
        ]);
        let rows = vec![row(1), row(2), row(5)];
        let kept = narrow(rows, &s, Some(&StatusChoice::Is("Accepted".to_owned())));
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, Some(ObjId::new(1, 0)));
    }

    /// ★★ **The work-list filter**: the comments nobody has reviewed.
    #[test]
    fn filtering_to_unrecorded_keeps_the_untouched_comments() {
        let s = statuses(&[annot(3, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let rows = vec![row(1), row(2), row(5)];
        let kept = narrow(rows, &s, Some(&StatusChoice::Unrecorded));
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|r| r.id != Some(ObjId::new(1, 0))));
    }

    /// ★ **A value pdfcer never heard of is filterable**, in the file's own
    /// spelling — the operator-facing consequence of the engine reading
    /// `/State` verbatim.
    #[test]
    fn an_unmodelled_value_is_offered_by_the_chooser_and_filters() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Deferred", Some("Review"))]);
        assert_eq!(s.values(), vec!["Deferred"]);
        let kept = narrow(
            vec![row(1), row(9)],
            &s,
            Some(&StatusChoice::Is("Deferred".to_owned())),
        );
        assert_eq!(kept.len(), 1);
    }

    /// ★ The chooser offers **tip** values only. A status somebody has since
    /// superseded would be a menu entry that filters to nothing.
    #[test]
    fn the_chooser_does_not_offer_a_superseded_value() {
        let s = statuses(&[
            annot(2, Some("Ken"), Some(1), "Rejected", Some("Review")),
            annot(3, Some("Ken"), Some(2), "Accepted", Some("Review")),
        ]);
        assert_eq!(s.values(), vec!["Accepted"]);
    }

    /// ★ **Any reviewer, not every reviewer.** Two people disagreeing must both
    /// be findable, because a work list's job is to surface what may need
    /// attention.
    #[test]
    fn a_status_filter_matches_if_any_reviewer_holds_it() {
        let s = statuses(&[
            annot(2, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(3, Some("Jo"), Some(1), "Rejected", Some("Review")),
        ]);
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("Rejected".to_owned()))));
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("Accepted".to_owned()))));
    }

    /// A row with **no object id** carries no status, because nothing can point
    /// at it — and it therefore counts as unreviewed rather than as an error.
    #[test]
    fn a_row_with_no_id_is_unreviewed() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let mut r = row(1);
        r.id = None;
        assert!(s.on(r.id).is_empty());
        assert!(s.keeps(&r, Some(&StatusChoice::Unrecorded)));
    }

    /// **An empty document has nothing to offer**, which is what lets
    /// [`status_strip`] withhold the chooser under R9.
    #[test]
    fn a_document_with_no_statuses_offers_no_values() {
        assert!(statuses(&[]).values().is_empty());
    }
}
