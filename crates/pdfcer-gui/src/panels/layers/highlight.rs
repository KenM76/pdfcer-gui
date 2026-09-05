//! `panels::layers::highlight` — which layer the current selection is on.
//!
//! The operator's ask, verbatim: *"selecting an object highlights that
//! layer"*.
//!
//! # ★★★ THE ENGINE ANSWERED. This file is the second half arriving.
//!
//! Until 2026-09-04 the whole of this module's header was an argument for why
//! **only an annotation** could be answered for. The argument was correct and
//! is kept in outline below, because the *reason it was correct* is the reason
//! it stopped being correct:
//!
//! > `vector::decompose`'s walk counted `/OC` sections into
//! > `DecomposeDiagnostics::oc_sections` and threw the group identity away;
//! > `pdfcer-render`'s interpreter resolved the same reference and pushed a
//! > `bool`. Two places knew which layer an object was on, and neither could
//! > say.
//!
//! ★★ **The workaround was refused, and the refusal is what produced the
//! capability.** This shell *could* have re-tokenized the page with
//! `ContentStream::from_page`, kept its own `/OC` stack and indexed by
//! `VectorObject::tokens()` — about forty lines of public API. It was refused
//! because it would have been **a second implementation of `/OC` resolution
//! beside the engine's**, blind to OCMD `/VE` visibility policy, forking a
//! `pub(crate)` helper, and destined to disagree with the renderer on some
//! file nobody would ever debug. A request was filed instead. `Pass 250.0`
//! answered it **within hours**: `oc: Option<ObjId>` now sits on
//! `PathObject`, `TextObject` and `ImageObject`, read through
//! [`pdfcer_core::vector::VectorObject::oc`] and
//! [`pdfcer_core::vector::FormLeaf::oc`].
//!
//! ⇒ **A shell that nearly resolves optional content ships a defect and
//! silences the request that would have fixed it.** That is the transferable
//! half, and it is now this project's third instance.
//!
//! # What the engine gives, verified at `pdfcer-core` v0.38.0 (`b01964f`)
//!
//! | fact | `file:line` in `crates/pdfcer-core/src/vector/decompose.rs` |
//! |---|---|
//! | `PathObject::oc` | `386` |
//! | `TextObject::oc` | `484` |
//! | `ImageObject::oc` | `780` |
//! | `VectorObject::oc()` | `1064` |
//! | `FormLeaf::oc()` | `1300` |
//! | `DecomposeDiagnostics::oc_sections` | `1165` |
//! | `DecomposeDiagnostics::oc_unresolved` | `1171` |
//! | the walk that fills them (`current_oc`) | `1485` |
//! | `Annotation::oc` (the half that already worked) | `annot.rs` |
//!
//! And the contract on each `oc` field, quoted because this module's whole
//! three-valued design turns on it:
//!
//! > *"`None` means the object is on NO layer; it does NOT mean 'could not
//! > tell' (see `DecomposeDiagnostics::oc_unresolved`). Membership only: an
//! > OCMD is reported as its own `ObjId`, never expanded, and visibility is
//! > NOT resolved here."*
//!
//! # ★★★ THE TWO DIVERGENCES THE SECOND ROUTE FOUND
//!
//! This project's standing finding is that **adding a second route to a
//! capability audits it**. Building the page-object route beside the
//! annotation route found two places where the engine's answer is a *partial*
//! that its own type cannot express, because `Option<ObjId>` has no third
//! value:
//!
//! ### D1 — a form leaf does not inherit the `/OC` its `Do` was painted under
//!
//! [`pdfcer_core::vector::FormLeaf::oc`] delegates straight to the wrapped
//! object, and the engine's own doc comment names the gap: *"A page-level
//! `BDC /OC` enclosing the form's `Do` is NOT folded in here … a documented
//! partial for the nested case."* `collect_form_leaves` has `img.oc` in hand
//! — the enclosing form object's own membership, correctly resolved one line
//! earlier — and does not pass it down.
//!
//! **So a leaf inside a form on layer *Grid* reports `None`, which the field
//! contract defines as "on NO layer".** That is not a missing answer; it is a
//! *wrong* one, and it is the exact failure the operator's bar forbids:
//! highlighting nothing while asserting a fact.
//!
//! ⇒ **This module repairs the depth-1 case from the engine's own two
//! answers** — see [`for_leaf`] — by consulting the enclosing form object at
//! [`pdfcer_core::vector::FormLeaf::paint_order`]. That is composition of two
//! engine results, **not** a second `/OC` implementation: no content stream is
//! re-tokenized and no `/Properties` key is resolved here. At depth **> 1** an
//! intermediate form's own membership is unreachable — the nested form
//! container is deliberately dropped from `leaves` — so the honest answer is
//! [`Unresolved::NestedForm`] and it is stated rather than guessed.
//!
//! ### D2 — `oc_unresolved` does not count what happens inside a form
//!
//! `collect_form_leaves` bumps `form_cycles` and `form_depth_overflows` on the
//! page's diagnostics and **discards `nested.diagnostics` entirely**. So the
//! counter the engine names as *"how a shell tells the two apart"* is blind to
//! every form interior. A leaf under an unresolvable `BDC /OC` reports `None`
//! with nothing anywhere to contradict it.
//!
//! ⇒ Not worked around. Honouring the page counter for leaves as well is the
//! best available reading and is what [`for_leaf`] does; the residual is
//! **reported**, on the `ENGINE_BACKLOG.md` row and in the request follow-up,
//! rather than absorbed. Making every form-interior object [`Unresolved`]
//! instead was considered and rejected: it would retire the feature on exactly
//! the CAD drawings it was built for, in exchange for a case the engine
//! measured at 0.6 % of files carrying optional content at all.
//!
//! # ★★★ Why the answer is FIVE-valued, which is the whole design
//!
//! The obvious type is `Option<ObjId>`, and it is wrong here in a way that
//! matters more than usual.
//!
//! | | `Option` says | the truth |
//! |---|---|---|
//! | a stamp with no `/OC` | `None` | **on no layer** — a fact the engine established |
//! | a leaf three forms deep | `None` | **not known** — D1 above |
//! | two objects on two layers | one of them | **neither, alone** |
//!
//! Collapsing those makes the panel unable to distinguish *"this mark is on no
//! layer"* from *"nobody can tell you"* from *"your selection spans two"*, and
//! the operator reads every one of them as the first. The operator's own note
//! on this feature is the bar: **highlighting the wrong layer is worse than
//! highlighting none** — and asserting "on no layer" about an object whose
//! layer is merely unknown is the same class of wrong answer, arriving as
//! silence instead of as a highlight.
//!
//! ⇒ [`Membership`] has five variants and they form a **join semilattice**,
//! so a multi-object selection folds without anybody writing an order down
//! twice. See [`Membership::join`].
//!
//! # ★★★ The operator's own finding: THE UNIT OF SELECTION IS NOT HIS
//!
//! He measured it on his own drawing: **one PDF path object holds 6,681
//! anchors across half his sheet** (`RESUME.md`; `pdfcer object-list` on
//! `SW41177.pdf` p1 reports 4,405 / 4,972 / 6,681, the largest holding 1,194
//! subpaths across 550 × 500 pt).
//!
//! `/OC` membership is a property of a **marked-content section**, which wraps
//! *paint operators*. A `BDC /OC` cannot begin in the middle of a subpath, so
//! every subpath and every anchor of one `PathObject` shares that object's
//! membership by construction — the relation is exact at object granularity
//! and **has no finer form to be exact at**.
//!
//! ⇒ So descending to the Part or Point rung does not refine the answer, and
//! this module deliberately ignores [`crate::canvas::selection::Selection`]'s
//! `subpath` and `node`. What that costs the operator is real and is stated
//! off-canvas rather than implied: the thing he *thinks* he selected — the
//! circle he clicked — may be one of 1,194 subpaths in an object that spans
//! two title blocks, and the layer named is the **whole object's**. See
//! [`crate::text::panels::layers::layer_selection_granularity`], which the
//! panel shows whenever the selected object holds more than one part.
//!
//! # ★★★ Rule 4 — this is DISCLOSURE, and none of it touches the canvas
//!
//! Nothing is drawn differently on the page. No badge, no tint, no dashed
//! outline, no provisional layer painted over the selected content. The
//! selection handles are the *cursor* and are unchanged. Every statement this
//! module produces lands in two off-canvas places:
//!
//! | surface | what it says |
//! |---|---|
//! | the Layers panel row | a background plate on the layer the selection is on |
//! | the status bar's selection line | the layer's name, as a clause on the line that already names the object |
//!
//! ★★ The second exists because **the canvas is the primary surface, never a
//! panel.** The engine can now answer *"which layer is this on"*, so clicking
//! the object must be able to reach that answer with no panel open. A panel is
//! a supplement.
//!
//! # The reverse relation, deliberately not built
//!
//! *Does clicking a layer indicate its objects?* Now derivable — a scan of
//! `PageObjects::objects` filtering on `oc()` — and still not built here,
//! because indicating them means **marking the canvas**, which Rule 4 forbids
//! for an inference. The shape it would have to take (a count, off-canvas, in
//! the row's tooltip) is a separate decision and a separate landing.

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::PageObjects;

use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;

/// **Why pdfcer cannot name the layer**, when it cannot.
///
/// # ★★★ Why the reason is carried rather than collapsed
///
/// It was collapsed until 2026-09-04, and the collapse was right then: with
/// *every* content object unanswerable, a sentence saying so would have been a
/// permanent apology printed on every selection in the program, and R9's
/// answer to an unavailable capability is to render nothing.
///
/// **That justification expired with `Pass 250.0`.** Every variant below is
/// now rare and specific — a malformed document, a nesting depth, a stale
/// index — and each one means something different for what the operator should
/// do next. A document whose `/OC` sections pdfcer cannot resolve is a fact
/// about *their file*; a leaf three forms deep is a fact about *pdfcer*. One
/// hedge covering both teaches them to ignore the line.
///
/// ⇒ **The reason to be silent was that the answer was always the same.**
/// When that stops being true, silence stops being honesty and becomes
/// withholding.
///
/// # ★★★ The declaration order is a PRIORITY order, and `Ord` is derived
///
/// Two selected objects can be unanswerable for two different reasons, and the
/// panel prints one sentence. Which one is not arbitrary: [`Membership::join`]
/// takes the **smaller**, so the variant declared first wins, and the order
/// below is *"which reason does the operator most need to read?"*
///
/// * **Nothing could be read at all** dominates every finer reason, because
///   none of the finer ones was even reached.
/// * **A nesting pdfcer cannot see through** outranks a malformation, because
///   it is specific to the thing they clicked and it is actionable — ungroup
///   the form.
/// * **A malformed page** outranks the two bookkeeping states, because it is a
///   fact about their file rather than about pdfcer's cache.
///
/// ★ Deriving `Ord` rather than writing a `rank()` is deliberate: a hand-written
/// ranking and a variant list are two places to state one order, and they drift.
/// Moving a variant is the whole edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unresolved {
    /// The page's content would not decompose at all, so there is no object
    /// model to ask. The same failure the renderer would hit; the Objects
    /// panel says so in its own words on the same frame.
    PageNotDecomposed,
    /// The object lives **more than one form XObject deep** and carries no
    /// `/OC` of its own, so its membership depends on an intermediate form's
    /// `/OC` that the leaf list does not carry. Divergence D1 in the module
    /// header.
    NestedForm,
    /// The page carries a `BDC /OC` marked-content section whose `/Pn` key did
    /// not resolve to an indirect `/Properties` entry —
    /// [`pdfcer_core::vector::DecomposeDiagnostics::oc_unresolved`] is
    /// non-zero.
    ///
    /// ★ **Page-scoped, not object-scoped**, because the counter is. The
    /// engine's own doc comment nominates it as the way a shell distinguishes
    /// *"on no layer"* from *"pdfcer could not name the group"*, and it counts
    /// per decomposition. So one unresolvable section demotes every `None` on
    /// that page. That is coarse and it is the safe direction: it withholds an
    /// answer rather than asserting a wrong one, on a page the file has
    /// already been shown to be wrong about.
    Malformed,
    /// The selection names an index the current decomposition does not have.
    ///
    /// Reachable for one frame after an edit that shortens the object list,
    /// before `SelectionState::resolve` re-resolves. Not expected, and said
    /// rather than papered over: an index that has outrun its model is exactly
    /// the condition that makes an edit act on the wrong object.
    Stale,
    /// Part of the selection is on a **page that is not the one on screen**,
    /// whose decomposition this shell does not hold.
    ///
    /// `SelectionState` keeps entries across a page change on purpose — that
    /// is what lets a selection survive navigating away and back — and
    /// `OpenDoc` caches exactly one page's object model. Decomposing another
    /// page to answer a readout would cost 469 ms on the operator's own
    /// benchmark sheet (`app::cache`), for a question he did not ask.
    OtherPage,
}

impl Unresolved {
    /// One stable word per reason, for the diagnostic channel.
    ///
    /// See [`Membership::kind`] for why this is not `{:?}`.
    // ui-text-exempt: a trace vocabulary, never displayed.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::PageNotDecomposed => "page-not-decomposed",
            Self::Malformed => "malformed",
            Self::NestedForm => "nested-form",
            Self::Stale => "stale",
            Self::OtherPage => "other-page",
        }
    }
}

/// Which optional-content group the current selection belongs to.
///
/// Five-valued on purpose — see the module header's table for the states an
/// `Option` would merge and why merging them produces a false statement rather
/// than a missing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// **Nothing is selected**, so there is no question to answer.
    ///
    /// Distinct from [`Self::Unknown`], which means something *is* selected
    /// and pdfcer cannot say. Both render nothing, and they are still
    /// different: a caller counting "how often can we not answer" must not
    /// count an empty canvas as a failure. It is also the **identity** of
    /// [`Self::join`], which is what lets a fold over an empty selection
    /// produce it without a special case.
    NothingSelected,
    /// **The selection is on this optional-content group.**
    ///
    /// The `ObjId` is comparable directly against `Layer::id`, against
    /// `OpenDoc::hidden_layers()` and against
    /// `annot::optional_content_default_off` — all four speak the same
    /// vocabulary, which is what makes the highlight a lookup rather than a
    /// translation.
    ///
    /// ★ It may name an **OCMD** rather than an OCG (§8.11.2.2): both an
    /// annotation's `/OC` and a content section's `/Pn` are allowed to be
    /// either, and the engine reports membership without expanding. An OCMD's
    /// id will not match any row, and the panel says so in words rather than
    /// letting the silence read as "no layer" — see
    /// [`crate::text::panels::layers::layer_selection_report`]'s off-list arm.
    /// Resolving it to "the first group it mentions" would highlight a layer
    /// that does not by itself decide whether the mark is drawn.
    Group(ObjId),
    /// **The selection is on no layer**, and the engine established that.
    ///
    /// A positive fact, and worth saying out loud: a drawing whose every mark
    /// is on a layer makes an unlayered stamp genuinely surprising, and an
    /// operator who has just switched a layer off and is wondering why their
    /// note is still there deserves to be told why.
    None,
    /// **pdfcer cannot say**, and this is why.
    ///
    /// ★ It is a *variant* rather than an absence so that a test can assert
    /// about it — "we stopped being able to answer" and "we never could" are
    /// distinguishable in the suite — and so that each cause can carry its own
    /// sentence.
    Unknown(Unresolved),
    /// **The selection spans more than one layer**, or mixes layered and
    /// unlayered objects.
    ///
    /// Above the atoms in [`Self::join`]'s lattice and **below**
    /// [`Self::Unknown`]. That ordering was the other way round for one
    /// afternoon, on the reasoning that *"this selection spans several layers"*
    /// is a positively established fact a later unanswerable member cannot take
    /// back — which is true, and which **broke associativity**:
    /// `(G₁⊔G₂)⊔U = Mixed` while `G₁⊔(G₂⊔U) = Unknown`, so the highlight would
    /// have depended on the order the selection happened to be folded in.
    /// `the_fold_does_not_depend_on_selection_order` caught it. See
    /// [`Self::join`].
    ///
    /// It highlights **nothing**. Highlighting every layer involved was
    /// considered: it is not wrong, but a panel with three plates in it reads
    /// as three selections, and the operator's bar ("highlighting the wrong
    /// layer is worse than highlighting none") makes the conservative reading
    /// the right one. The count is stated in words instead.
    Mixed,
}

impl Membership {
    /// The row this should emphasise, if any.
    #[must_use]
    pub const fn highlighted(self) -> Option<ObjId> {
        match self {
            Self::Group(id) => Some(id),
            Self::NothingSelected | Self::None | Self::Unknown(_) | Self::Mixed => None,
        }
    }

    /// **One stable word per state, for the diagnostic channel.**
    ///
    /// ★★★ Not `{:?}`. A derived `Debug` prints `Group(ObjId { number: 4,
    /// generation: 0 })` — braces and spaces, which the harness's `key=value`
    /// trace parser splits into five tokens and reads as none of them. Worse,
    /// it would change spelling the day a field is added to `ObjId`, silently
    /// retiring every check that matched on it.
    ///
    /// ★ The words are deliberately **not** the operator's sentences. A trace
    /// vocabulary that tracked the wording would make a copy-edit a harness
    /// break, and a check asserting on prose asserts on the wrong thing.
    // ui-text-exempt: a trace vocabulary, never displayed.
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::NothingSelected => "nothing",
            Self::Group(_) => "group",
            Self::None => "no-layer",
            Self::Unknown(_) => "unknown",
            Self::Mixed => "mixed",
        }
    }

    /// The reason word, for the states that have one.
    ///
    /// ★ `"-"` and not `""` for the states that have none. The trace parser
    /// gives structural meaning to a space, and an empty value would put two
    /// spaces where every other line has one — a shape difference that is
    /// invisible to a reader and is exactly the sort of thing a field parser
    /// gets subtly wrong. The line's shape must not vary with its content.
    // ui-text-exempt: a trace vocabulary, never displayed.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Unknown(why) => why.key(),
            Self::NothingSelected | Self::Group(_) | Self::None | Self::Mixed => "-",
        }
    }

    /// **Fold two answers about two objects into one answer about the set.**
    ///
    /// # The lattice, and why it must BE a lattice
    ///
    /// A multi-object selection has one membership only when every member
    /// agrees. Written as a chain of `if`s at the call site that rule drifts;
    /// written as an associative, commutative join with an identity it cannot,
    /// and a fold over any number of members needs no special case for zero or
    /// one.
    ///
    /// ```text
    ///                  Unknown(w)             ← some member could not be answered
    ///                       |
    ///                     Mixed               ← two members positively disagree
    ///                    /         ///             Group(a)  …   None          ← the atoms
    ///                    \     /
    ///                NothingSelected          ← identity
    /// ```
    ///
    /// ## ★★★ The ordering that is NOT obvious, and it cost a wrong design
    ///
    /// **`Mixed ⊔ Unknown = Unknown`**, not `Mixed`. The first draft had it the
    /// other way, and the argument was good: once `Group(a)` and `Group(b)` are
    /// both established, *"this selection spans several layers"* is a fact, and
    /// an unanswerable third member cannot un-establish it.
    ///
    /// **It is not associative.** `(G₁ ⊔ G₂) ⊔ U` is `Mixed ⊔ U = Mixed`, while
    /// `G₁ ⊔ (G₂ ⊔ U)` is `G₁ ⊔ U = Unknown`. A fold whose result depends on
    /// bracketing is a fold whose result depends on **the order the operator
    /// added objects to the selection** — so a marquee and three shift-clicks
    /// over the same three objects could light different rows. That is not a
    /// theoretical property: it is a highlight that flickers between two
    /// answers for reasons nobody could diagnose.
    ///
    /// ⇒ `Unknown` is the top. It is also the honest one: with a member nobody
    /// could resolve, *"spans several layers"* is true and **incomplete**, and
    /// this panel's standing rule is to withhold rather than to under-state.
    ///
    /// ## ★★ And `Unknown(a) ⊔ Unknown(b)` takes the SMALLER reason
    ///
    /// Not the left one. Two `Unknown`s with different reasons and a
    /// "first wins" rule is **not commutative** — `U(a) ⊔ U(b)` would differ
    /// from `U(b) ⊔ U(a)`, which is the same order-dependence one level down
    /// and would have been invisible to any test whose sample set held one
    /// `Unknown`. [`Unresolved`]'s declaration order is the priority order and
    /// its `Ord` is derived, so this is `min` and nothing else.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            // Identity, both ways round.
            (Self::NothingSelected, x) | (x, Self::NothingSelected) => x,
            // Top absorbs, and two tops merge by priority rather than by
            // position — see the doc comment.
            (Self::Unknown(a), Self::Unknown(b)) => Self::Unknown(if a <= b { a } else { b }),
            (Self::Unknown(why), _) | (_, Self::Unknown(why)) => Self::Unknown(why),
            (Self::Mixed, _) | (_, Self::Mixed) => Self::Mixed,
            // The atoms.
            (Self::Group(a), Self::Group(b)) if a == b => Self::Group(a),
            (Self::None, Self::None) => Self::None,
            (Self::Group(_) | Self::None, Self::Group(_) | Self::None) => Self::Mixed,
        }
    }
}

/// **The answer for one page object**, from its `/OC` and the page's honesty.
///
/// A pure function over the two values that decide it, so the rule can be
/// tested without a document, a decomposition or an egui frame — and so that
/// the leaf rule beside it is visibly the *same* rule plus its two extra
/// clauses rather than a second, similar one.
///
/// `page_malformed` is
/// [`pdfcer_core::vector::DecomposeDiagnostics::oc_unresolved`]` > 0`. It
/// demotes a `None` and leaves a `Some` alone, which is the asymmetry the
/// counter's own doc comment describes: an unresolvable section produces `oc
/// == None`, never a wrong group, so a positively named group is unaffected by
/// one existing elsewhere on the page.
#[must_use]
pub const fn for_object(oc: Option<ObjId>, page_malformed: bool) -> Membership {
    match oc {
        Some(id) => Membership::Group(id),
        None if page_malformed => Membership::Unknown(Unresolved::Malformed),
        None => Membership::None,
    }
}

/// **The answer for one object inside a form XObject** — the repair for
/// divergence D1.
///
/// # The three clauses, and what each is worth
///
/// 1. **The leaf's own `/OC` wins outright.** It is the innermost membership,
///    which is what `current_oc` resolves and what the renderer honours. Depth
///    is irrelevant to it.
/// 2. **At depth 1, an absent one falls back to the enclosing form object's.**
///    That object is `PageObjects::objects[leaf.paint_order]`, decomposed by
///    the page walk, and its `oc` was resolved by the same engine code that
///    resolved every other object on the page — including the `xobject_oc`
///    §8.11.3.3 case. Composing the two is not a second implementation; it is
///    reading an answer the engine already computed and did not thread through.
/// 3. **Deeper than that, say so.** `collect_form_leaves` drops the nested
///    form *container* from the leaf list on purpose (it would otherwise put a
///    second page-sized hit target back into the list built to remove the
///    first), so an intermediate form's own `/OC` has no representative
///    anywhere in `PageObjects`. There is nothing to compose, and guessing
///    with the outermost would name a group an inner `/OC` may have overridden.
///
/// ★ `outermost` is deliberately named for what it *is* rather than "parent":
/// `paint_order` is the **outermost** enclosing form's index in the page's own
/// list, carried unchanged down the recursion. At depth 1 outermost and parent
/// coincide, which is exactly why clause 2 is fenced to depth 1.
#[must_use]
pub const fn for_leaf(
    own: Option<ObjId>,
    depth: usize,
    outermost: Option<ObjId>,
    page_malformed: bool,
) -> Membership {
    match own {
        Some(id) => Membership::Group(id),
        // Clause 3 first among the `None`s: a nesting we cannot see through
        // outranks a page-level counter, because it is the more specific
        // reason and it is the one the operator can act on (ungroup the form).
        None if depth > 1 => Membership::Unknown(Unresolved::NestedForm),
        None => for_object(outermost, page_malformed),
    }
}

/// The answer for one selected target, resolved against a page's model.
///
/// Both index spaces are resolved **by the [`TargetId`] itself** rather than
/// by a caller that had to remember which list it was holding — the same
/// discipline `app::status::selected` uses, and for the same reason: the two
/// spaces are both `u64` and a mix-up is silent.
fn for_target(model: &PageObjects, target: TargetId) -> Membership {
    let page_malformed = model.diagnostics.oc_unresolved > 0;
    match target {
        TargetId::Object(i) => match usize::try_from(i).ok().and_then(|i| model.objects.get(i)) {
            Some(object) => for_object(object.oc(), page_malformed),
            None => Membership::Unknown(Unresolved::Stale),
        },
        TargetId::Leaf(i) => match usize::try_from(i).ok().and_then(|i| model.leaves.get(i)) {
            Some(leaf) => for_leaf(
                leaf.oc(),
                leaf.containment.len(),
                model
                    .objects
                    .get(leaf.paint_order)
                    .and_then(pdfcer_core::vector::VectorObject::oc),
                page_malformed,
            ),
            None => Membership::Unknown(Unresolved::Stale),
        },
    }
}

/// **Which layer is the current selection on?**
///
/// # The order of the arms, which is the whole of the routine
///
/// 1. **An annotation is selected** → ask the engine. `Annotation::oc` is the
///    §8.11.3.3 reference, and `None` there means *on no layer* — the engine
///    has read the annotation and the entry is absent, which is a fact rather
///    than an inability.
/// 2. **Content is selected** → fold [`for_target`] over every selected target
///    on the page whose model this shell holds, and join
///    [`Unresolved::OtherPage`] if any entry names another page.
/// 3. **Nothing is selected** → [`Membership::NothingSelected`].
///
/// `SelectionState` enforces that 1 and 2 are mutually exclusive — its `annot`
/// field exists *"because the two are mutually exclusive and that must be
/// enforced by a type, not remembered"* — so the order between the first two
/// arms cannot decide anything, and it is written annotation-first only
/// because it is the shorter arm.
///
/// # ★ The empty-target guard, which is not redundant with `is_empty`
///
/// `is_empty()` is false while entries exist on **another** page, and
/// `targets_on(current)` is then empty. Folding an empty iterator yields
/// [`Membership::NothingSelected`], which would report *"nothing is
/// selected"* about a selection that exists — the shape of wrong answer this
/// whole type is built to make unrepresentable. So the off-page entries are
/// counted and joined explicitly.
///
/// # Cost
///
/// For an annotation: one `page_annotations` read of its page, per frame,
/// while one is selected — the same call the Comments panel makes for *every*
/// page on every frame.
///
/// For content: a borrow of `OpenDoc`'s existing decomposition cache and an
/// index per selected object. **No new decomposition**; `page_objects()` is
/// the model the canvas overlay, the Objects panel and the status bar already
/// read on the same frame, keyed on the engine's own
/// `page_content_generation` digest. It is deliberately not cached further: a
/// cache keyed on a selection plus an edit epoch is a third thing to keep in
/// step for a vector index.
#[must_use]
pub fn resolve(doc: &OpenDoc) -> Membership {
    let Some(annot) = doc.selection.annot() else {
        return resolve_content(doc);
    };
    let view = doc.session.view();
    // `pages_in` and not `EditSession::pages()`: the panel holds a shared
    // `&OpenDoc` and the session's own accessor takes `&mut self`. This is the
    // same call `panels::comments` makes over the same view.
    let Ok(pages) = pdfcer_core::page_tree::pages_in(&view) else {
        // The page tree would not resolve. Emphatically not `None`: a document
        // we cannot read the structure of is the exact case the absent variant
        // exists for.
        return Membership::Unknown(Unresolved::PageNotDecomposed);
    };
    let Some(page) = pages.get(annot.target.page) else {
        // The selection names a page the current revision does not have —
        // reachable for one frame after a page delete, before the selection is
        // re-resolved.
        return Membership::Unknown(Unresolved::Stale);
    };
    match pdfcer_core::annot::page_annotations(&view, page.id)
        .into_iter()
        .find(|a| a.id == Some(annot.target.id))
    {
        Some(a) => match a.oc {
            Some(oc) => Membership::Group(oc),
            None => Membership::None,
        },
        // Selected, but no longer in the page's `/Annots`. Same reasoning as
        // the missing page.
        None => Membership::Unknown(Unresolved::Stale),
    }
}

/// The content half of [`resolve`], split out so the annotation arm stays
/// readable and so the fold has somewhere to be commented.
fn resolve_content(doc: &OpenDoc) -> Membership {
    if doc.selection.is_empty() {
        return Membership::NothingSelected;
    }
    let page = doc.view.page_index;
    let targets = doc.selection.targets_on(page);
    // ★ See the doc comment: entries on another page are a real state, and
    // folding only the current page's would report an empty answer about a
    // non-empty selection.
    let elsewhere = doc.selection.entries().iter().any(|e| e.page != page);
    let off_page = if elsewhere {
        Membership::Unknown(Unresolved::OtherPage)
    } else {
        Membership::NothingSelected
    };

    let Some(provider) = doc.page_objects() else {
        // The page would not decompose. The Objects panel says so in words on
        // the same frame; this line is why the layer row is not lit.
        return Membership::Unknown(Unresolved::PageNotDecomposed).join(off_page);
    };
    let model = provider.page_objects();
    targets
        .into_iter()
        .map(|t| for_target(model, t))
        .fold(off_page, Membership::join)
}

/// **How many parts the one selected object holds**, when that number is a
/// reason to distrust the word "selected".
///
/// `Some(n)`, `n > 1`, only when exactly one object is selected and it is a
/// path with several subpaths. `None` otherwise — including for a
/// multi-object selection, where the mismatch is already obvious from the
/// count in the status line.
///
/// # ★★★ Why a readout owes this at all
///
/// The operator, on his own drawing: **one PDF path object holds 6,681
/// anchors across half his sheet.** `pdfcer object-list` on `SW41177.pdf` p1
/// reports three objects of 4,405, 4,972 and 6,681 anchors, the largest
/// holding **1,194 subpaths** over 550 × 500 pt. He clicks a circle; pdfcer
/// selects the object the circle is a subpath of.
///
/// The layer named is that object's, and it is **correct** — `/OC` wraps paint
/// operators, so every subpath of one object shares one membership by
/// construction (module header). But *"this is on layer Grid"* said about
/// something the operator believes is a single circle is a true sentence he
/// will read as a claim about the circle, and on his files it is a claim about
/// a thousand other curves as well.
///
/// ⇒ **The precision is real and the granularity is not his.** Stating the
/// part count is how the sentence stops over-promising, and it is stated
/// **off-canvas** — Rule 4 forbids marking the drawing to express it.
///
/// ★ It is a count and not a hedge. *"This may be part of a larger object"*
/// would be a permanent disclaimer; *"this object holds 1,194 parts"* is a
/// measurement he can act on, and it is silent on the overwhelmingly common
/// object that holds one.
#[must_use]
pub fn parts_in_selected_object(doc: &OpenDoc) -> Option<usize> {
    if doc.selection.annot().is_some() {
        return None;
    }
    let page = doc.view.page_index;
    let targets = doc.selection.targets_on(page);
    let [target] = targets.as_slice() else {
        return None;
    };
    if doc.selection.entries().iter().any(|e| e.page != page) {
        return None;
    }
    let provider = doc.page_objects()?;
    let model = provider.page_objects();
    let object = match *target {
        TargetId::Object(i) => model.objects.get(usize::try_from(i).ok()?)?,
        TargetId::Leaf(i) => &model.leaves.get(usize::try_from(i).ok()?)?.object,
    };
    match object {
        pdfcer_core::vector::VectorObject::Path(p) if p.subpaths.len() > 1 => {
            Some(p.subpaths.len())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::OpenDoc;
    use crate::panels::objects::test_support::engine_fixture;

    fn oc(n: u32) -> ObjId {
        ObjId::new(n, 0)
    }

    /// ★★★ The one fixture in either corpus that can falsify this feature.
    ///
    /// `layers/painted-layers.pdf` is fourteen objects of hand-written syntax
    /// carrying **four** optional-content groups and, critically, **an object
    /// painted after every `EMC`** — so it holds both halves of the relation:
    ///
    /// ```text
    /// /OC /L1 BDC  0 0 0 rg 60 60 120 120 re f  EMC     <- "Visible Box"
    /// /OC /L2 BDC  0 0 0 rg 400 60 120 120 re f
    ///   /OC /L4 BDC 0 0 0 rg 400 220 120 120 re f EMC   <- "Nested Inner", innermost wins
    /// EMC
    /// /OC /L3 BDC  0 0 300 792 re W n EMC
    /// 0.5 g 0 600 612 60 re f                           <- ★ on NO layer
    /// ```
    ///
    /// ★★ A fixture whose every object shares one layer would make *"the
    /// answer follows the selection"* true of a build that ignores the
    /// selection entirely. That is this project's vacuous-pass shape, and it
    /// is why these tests use two objects with different answers rather than
    /// one with a right answer.
    fn painted_layers() -> OpenDoc {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages)
    }

    /// The paint-order index of the object whose page bbox is **exactly**
    /// this rectangle.
    ///
    /// ★ Chosen by **geometry**, never by `oc()`. Picking the object by the
    /// very field under test would make the assertion circular — it would
    /// prove that `resolve` returns what `oc()` returns, which is a restatement
    /// rather than a test.
    ///
    /// ★★★ **Exact bounds and not "contains this point", and the difference is
    /// a defect this helper already had.** The first version took a point and
    /// took the FIRST object covering it, which is how
    /// `an_object_outside_every_section_is_on_no_layer` came to select the
    /// fixture's `0 0 300 792 re W n` **clip path** — a real `PathObject`, on
    /// layer *Clip Only*, whose bbox covers most of the page and which is
    /// painted before the grey bar. The test failed with `Group(6)` where it
    /// expected `None`, and the report would have been *"the shell reports the
    /// wrong layer"* about a shell that was right and a helper that was
    /// pointing at something else.
    ///
    /// ⇒ *Ask what the test SAMPLED before asking what is broken.* A rectangle
    /// names one object; a point names whichever of several the helper's
    /// tie-break happened to reach.
    fn index_of_bbox(doc: &OpenDoc, min: (f64, f64), max: (f64, f64)) -> TargetId {
        let provider = doc.page_objects().expect("the fixture decomposes");
        let model = provider.page_objects();
        let near = |a: f64, b: f64| (a - b).abs() < 0.01;
        let mut found = model.objects.iter().enumerate().filter(|(_, o)| {
            let b = o.page_bbox();
            near(b.min.x, min.0)
                && near(b.min.y, min.1)
                && near(b.max.x, max.0)
                && near(b.max.y, max.1)
        });
        let (i, _) = found.next().unwrap_or_else(|| {
            panic!("no object has bounds {min:?}..{max:?} — the fixture has changed")
        });
        assert!(
            found.next().is_none(),
            "two objects share the bounds {min:?}..{max:?}, so this helper names neither of them"
        );
        TargetId::Object(u64::try_from(i).expect("an index fits"))
    }

    /// The layer name `resolve` lands on, resolved the way the panel resolves
    /// it — through the document's own `/OCProperties` list.
    fn resolved_name(doc: &OpenDoc) -> Option<String> {
        let read = pdfcer_core::layers::read_layers(&doc.session.view());
        resolve(doc)
            .highlighted()
            .and_then(|id| crate::panels::layers::layer_name_for(&read, id))
    }

    /// ★★★ **Selecting a page object names the layer it is painted on.**
    ///
    /// The operator's ask, in a headless test: click the square inside
    /// `/OC /L1` and the answer is *Visible Box*. Before `Pass 250.0` this was
    /// unanswerable and this module returned `Unknown` for every content
    /// object.
    #[test]
    fn selecting_a_page_object_names_the_layer_it_is_painted_on() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (60.0, 60.0), (180.0, 180.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(
            resolved_name(&open).as_deref(),
            Some("Visible Box"),
            "the square at (60,60)-(180,180) is painted inside `/OC /L1`, whose /Name is \
             'Visible Box'"
        );
    }

    /// ★★★ **…and an object painted outside every section is reported as on
    /// no layer, not as unknown and not as somebody else's layer.**
    ///
    /// This is the half a build cannot fake. Everything in the test above
    /// passes against an implementation that ignores the selection and always
    /// answers with the first group in the document; nothing here does.
    #[test]
    fn an_object_outside_every_section_is_on_no_layer() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (0.0, 600.0), (612.0, 660.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(
            resolve(&open),
            Membership::None,
            "the grey bar is painted after every EMC, so it is on no optional-content group — \
             and `None` here is a POSITIVE fact, distinct from `Unknown`"
        );
        assert_eq!(resolved_name(&open), None, "nothing may be highlighted");
    }

    /// ★★ **The innermost `/OC` wins**, which is what `current_oc` resolves
    /// and what the renderer honours.
    ///
    /// The square at (400,220) sits inside `/OC /L2 BDC … /OC /L4 BDC … EMC
    /// EMC`. Answering *Hidden Box* — the outer section — would be a wrong
    /// highlight rather than a missing one, and the operator's own bar rates
    /// that worse.
    ///
    /// ★ It is selected directly rather than clicked, because both L2 and L3
    /// are in the document's `/OFF` array: this object is in the model and is
    /// not drawn. The membership relation is what is under test, not the
    /// visibility one.
    #[test]
    fn the_innermost_section_is_the_layer_not_the_outer_one() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (400.0, 220.0), (520.0, 340.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(resolved_name(&open).as_deref(), Some("Nested Inner"));
    }

    /// ★★ **The granularity line stays silent on an ordinary object.**
    ///
    /// It is a measurement, not a disclaimer: the sentence exists because one
    /// object on the operator's own drawing holds 1,194 subpaths, and a
    /// warning printed on every single-part rectangle would teach him to skip
    /// the line before he ever met one that mattered.
    #[test]
    fn a_one_part_object_says_nothing_about_its_parts() {
        let mut open = painted_layers();
        let target = index_of_bbox(&open, (60.0, 60.0), (180.0, 180.0));
        open.selection.select_only(0, target, "test");
        assert_eq!(parts_in_selected_object(&open), None);
    }

    /// ★★★ **The two points `ui-verify selecting_an_object_names_its_layer`
    /// aims at, pinned headlessly.**
    ///
    /// # Why a unit test owns a driven check's coordinates
    ///
    /// Because a miscalibrated aim is this project's most expensive harness
    /// failure and it is **silent**: the check clicks, something is selected,
    /// an answer comes back, and the report is an articulate paragraph about
    /// the wrong object. `RESUME.md` records it three times in one day, and
    /// once as *"a wrong aim that happens to hit is a green result reporting
    /// nothing"*.
    ///
    /// This test asks the engine's own deep hit test — the one the canvas
    /// uses — what is under each point, and pins the answer. It runs in the
    /// sweep with no window open, so the driven check can never be quietly
    /// aiming at something else.
    ///
    /// # ★★ What it established, and it was not obvious
    ///
    /// The fixture's `0 0 300 792 re W n` **clip path** decomposes into a real
    /// `PathObject` on layer *Clip Only* whose bbox covers `(0,0)..(300,792)`
    /// — so it sits under BOTH aim points geometrically. It is **not** a hit
    /// candidate, because `n` paints nothing and `hit_test_point_deep` answers
    /// with ink rather than with bounds. Measured, not assumed: the sibling
    /// helper `index_of_bbox` had already been caught selecting that very
    /// object by bounds.
    #[test]
    fn the_driven_checks_two_aim_points_land_where_it_thinks() {
        let open = painted_layers();
        let provider = open.page_objects().expect("the fixture decomposes");
        let model = provider.page_objects();
        let read = pdfcer_core::layers::read_layers(&open.session.view());

        let layer_at = |x: f64, y: f64| -> Option<String> {
            let hits = pdfcer_core::vector::hit_test_point_deep(
                model,
                pdfcer_core::vector::Point::new(x, y),
                3.0,
            );
            let first = hits.first().copied().expect("the point must hit something");
            let i = match first {
                pdfcer_core::vector::HitTarget::Object(i) => i,
                pdfcer_core::vector::HitTarget::Leaf(_) => {
                    panic!("this fixture has no forms, so a leaf hit means the fixture changed")
                }
            };
            model.objects[i]
                .oc()
                .and_then(|id| crate::panels::layers::layer_name_for(&read, id))
        };

        assert_eq!(
            layer_at(120.0, 120.0).as_deref(),
            Some("Visible Box"),
            "the check's first click must land on the square inside `/OC /L1`"
        );
        assert_eq!(
            layer_at(150.0, 630.0),
            None,
            "the check's second click must land on the grey bar, which is on NO layer, and NOT on the `Clip Only` path whose bbox also covers this point"
        );
    }

    /// **Nothing selected is nothing selected**, on a layered document — the
    /// state that must not be confused with "on no layer".
    #[test]
    fn an_empty_selection_answers_the_empty_answer() {
        let open = painted_layers();
        assert_eq!(resolve(&open), Membership::NothingSelected);
    }

    /// ★★★ **`Unknown` and `None` are different values**, which is the whole
    /// reason this type exists rather than an `Option<ObjId>`.
    ///
    /// If this ever fails to compile because the two were merged, the panel
    /// has lost the ability to distinguish *"this mark is on no layer"* from
    /// *"nobody can tell you"* — and the operator reads the second as the
    /// first.
    #[test]
    fn not_on_a_layer_is_not_the_same_answer_as_cannot_tell() {
        assert_ne!(Membership::None, Membership::Unknown(Unresolved::Stale));
        assert_ne!(
            Membership::NothingSelected,
            Membership::Unknown(Unresolved::Stale)
        );
        assert_ne!(Membership::None, Membership::NothingSelected);
        assert_ne!(Membership::None, Membership::Mixed);
    }

    /// ★★ **The reason is part of the answer.**
    ///
    /// Two `Unknown`s with different causes must not compare equal, or the
    /// panel could print one reason while holding another and no test would
    /// see it.
    #[test]
    fn two_reasons_are_two_answers() {
        assert_ne!(
            Membership::Unknown(Unresolved::NestedForm),
            Membership::Unknown(Unresolved::Malformed)
        );
    }

    /// **Only a known group highlights a row.**
    ///
    /// The operator's bar, made mechanical: highlighting the wrong layer is
    /// worse than highlighting none, so every state that is not a *positively
    /// established* group must highlight nothing.
    #[test]
    fn only_a_known_group_highlights_anything() {
        assert_eq!(Membership::Group(oc(7)).highlighted(), Some(oc(7)));
        assert_eq!(Membership::None.highlighted(), None);
        assert_eq!(
            Membership::Unknown(Unresolved::NestedForm).highlighted(),
            None
        );
        assert_eq!(Membership::NothingSelected.highlighted(), None);
        assert_eq!(Membership::Mixed.highlighted(), None);
    }

    /// ★★ **The trace vocabulary is one word per state, and no word is
    /// empty.**
    ///
    /// `ui-verify selecting_an_object_names_its_layer` matches on `answer=`,
    /// so these strings are a harness contract rather than decoration. Two
    /// states sharing a word would make the check unable to tell them apart —
    /// and the pair that matters is `no-layer` against `unknown`, which is the
    /// whole distinction this type exists for arriving in the diagnostic
    /// channel.
    ///
    /// The empty-string assertion is about line SHAPE: a field whose value is
    /// empty puts two spaces where every other line has one, and a parser that
    /// gives a space structural meaning is entitled to read that differently.
    #[test]
    fn the_trace_vocabulary_separates_every_state() {
        let words = [
            Membership::NothingSelected.kind(),
            Membership::Group(oc(1)).kind(),
            Membership::None.kind(),
            Membership::Unknown(Unresolved::Stale).kind(),
            Membership::Mixed.kind(),
        ];
        for (i, a) in words.iter().enumerate() {
            assert!(!a.is_empty());
            assert!(!a.contains(' '), "a trace word with a space in it: {a}");
            for b in words.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        let reasons = [
            Unresolved::PageNotDecomposed,
            Unresolved::Malformed,
            Unresolved::NestedForm,
            Unresolved::Stale,
            Unresolved::OtherPage,
        ];
        for (i, a) in reasons.iter().enumerate() {
            assert_eq!(Membership::Unknown(*a).reason(), a.key());
            for b in reasons.iter().skip(i + 1) {
                assert_ne!(a.key(), b.key());
            }
        }
        // Every state that has no reason still prints one, so the line's shape
        // does not vary with the answer.
        for m in [
            Membership::NothingSelected,
            Membership::Group(oc(1)),
            Membership::None,
            Membership::Mixed,
        ] {
            assert!(!m.reason().is_empty(), "{m:?} prints an empty reason");
        }
    }

    /// **A group id round-trips**, so the panel's row lookup is a comparison
    /// rather than a translation.
    #[test]
    fn the_group_id_is_the_one_the_layers_list_speaks() {
        let id = oc(42);
        assert_eq!(Membership::Group(id).highlighted(), Some(id));
    }

    // -------------------------------------------------------------------
    // The per-object rule.
    // -------------------------------------------------------------------

    /// **A resolved `/OC` is the layer, and a page's malformation elsewhere
    /// does not taint it.**
    #[test]
    fn a_named_group_is_the_answer_even_on_a_malformed_page() {
        assert_eq!(for_object(Some(oc(4)), false), Membership::Group(oc(4)));
        assert_eq!(for_object(Some(oc(4)), true), Membership::Group(oc(4)));
    }

    /// ★★★ **`None` means "on no layer" only while the page's `/OC` sections
    /// all resolved.**
    ///
    /// This is the engine's own contract consumed: *"`oc_unresolved` … is how
    /// a shell tells the two apart"*. Without this arm an unnameable group
    /// renders as the positive claim *"not on a layer"*.
    #[test]
    fn an_unresolvable_section_demotes_no_layer_to_cannot_tell() {
        assert_eq!(for_object(None, false), Membership::None);
        assert_eq!(
            for_object(None, true),
            Membership::Unknown(Unresolved::Malformed)
        );
    }

    // -------------------------------------------------------------------
    // The form-leaf rule — divergence D1.
    // -------------------------------------------------------------------

    /// **A leaf's own `/OC` wins at any depth.**
    #[test]
    fn a_leafs_own_group_wins_over_the_form_it_is_in() {
        assert_eq!(
            for_leaf(Some(oc(9)), 1, Some(oc(4)), false),
            Membership::Group(oc(9))
        );
        assert_eq!(
            for_leaf(Some(oc(9)), 3, Some(oc(4)), false),
            Membership::Group(oc(9))
        );
    }

    /// ★★★ **D1, repaired: a leaf one form deep inherits the layer its `Do`
    /// was painted under.**
    ///
    /// `FormLeaf::oc()` delegates to the wrapped object and the engine's own
    /// doc comment calls the omission *"a documented partial"*. Without this
    /// arm, everything inside a form on layer *Grid* reports **"on no
    /// layer"** — a wrong positive, not a missing answer.
    #[test]
    fn a_leaf_one_form_deep_inherits_the_forms_layer() {
        assert_eq!(
            for_leaf(None, 1, Some(oc(4)), false),
            Membership::Group(oc(4))
        );
    }

    /// **…and a leaf in an unlayered form is genuinely on no layer.**
    ///
    /// The other direction of the same arm, and the one that stops the repair
    /// from becoming "everything inside a form is on a layer".
    #[test]
    fn a_leaf_in_an_unlayered_form_is_on_no_layer() {
        assert_eq!(for_leaf(None, 1, None, false), Membership::None);
    }

    /// ★★★ **Deeper than one form, pdfcer says so rather than guessing.**
    ///
    /// The intermediate form's own `/OC` has no representative in
    /// `PageObjects` — `collect_form_leaves` drops nested containers — so the
    /// outermost form's group is *not* evidence about this leaf. Answering
    /// `Group(4)` here would be the exact wrong-highlight the operator's bar
    /// forbids.
    #[test]
    fn a_leaf_two_forms_deep_is_not_guessed_from_the_outer_one() {
        assert_eq!(
            for_leaf(None, 2, Some(oc(4)), false),
            Membership::Unknown(Unresolved::NestedForm)
        );
        assert_eq!(
            for_leaf(None, 2, None, false),
            Membership::Unknown(Unresolved::NestedForm)
        );
    }

    /// **The nesting reason outranks the malformation reason**, because it is
    /// the more specific of the two and the one the operator can act on.
    #[test]
    fn the_nesting_reason_is_the_one_reported() {
        assert_eq!(
            for_leaf(None, 4, None, true),
            Membership::Unknown(Unresolved::NestedForm)
        );
    }

    // -------------------------------------------------------------------
    // The fold.
    // -------------------------------------------------------------------

    /// **Nothing selected is the identity**, so a fold needs no empty case.
    #[test]
    fn nothing_selected_is_the_folds_identity() {
        let g = Membership::Group(oc(1));
        assert_eq!(Membership::NothingSelected.join(g), g);
        assert_eq!(g.join(Membership::NothingSelected), g);
        assert_eq!(
            Membership::NothingSelected.join(Membership::NothingSelected),
            Membership::NothingSelected
        );
    }

    /// **Agreement survives; disagreement becomes `Mixed`.**
    #[test]
    fn two_objects_on_one_layer_stay_on_one_layer() {
        let a = Membership::Group(oc(1));
        let b = Membership::Group(oc(2));
        assert_eq!(a.join(a), a);
        assert_eq!(a.join(b), Membership::Mixed);
        assert_eq!(b.join(a), Membership::Mixed);
    }

    /// ★★ **A layered object and an unlayered one are `Mixed`, not the
    /// layer.**
    ///
    /// This is the marquee case on the operator's own drawings and the one a
    /// naive `find_map` implementation gets wrong: it would report the first
    /// group it saw and light a row, claiming a whole selection is on a layer
    /// half of it is not on.
    #[test]
    fn a_layered_and_an_unlayered_object_are_mixed() {
        assert_eq!(
            Membership::Group(oc(1)).join(Membership::None),
            Membership::Mixed
        );
        assert_eq!(
            Membership::None.join(Membership::Group(oc(1))),
            Membership::Mixed
        );
        assert_eq!(Membership::None.join(Membership::None), Membership::None);
    }

    /// ★★★ **One unanswerable member makes the whole answer unanswerable.**
    ///
    /// The highlight is read as a claim about the *selection*, so it may not
    /// survive a member nobody could resolve.
    #[test]
    fn an_unknown_member_withholds_the_whole_answer() {
        let u = Membership::Unknown(Unresolved::NestedForm);
        assert_eq!(Membership::Group(oc(1)).join(u), u);
        assert_eq!(u.join(Membership::Group(oc(1))), u);
        assert_eq!(Membership::None.join(u), u);
    }

    /// ★★★ **…including over an established disagreement, and this assertion
    /// is INVERTED from the one that was written first.**
    ///
    /// The first draft asserted `Mixed ⊔ Unknown = Mixed`, on the reasoning
    /// that *"this selection spans several layers"* is a fact an unanswerable
    /// third member cannot take back. The reasoning is sound and the rule is
    /// **not associative** — `the_fold_does_not_depend_on_selection_order`
    /// found it within a minute of being written, with the message
    /// *"join is not associative: Group(1) Group(2) Unknown(Stale) — left:
    /// Mixed, right: Unknown(Stale)"*.
    ///
    /// ⇒ A highlight that depends on the order the operator added objects to
    /// the selection is a highlight that flickers for reasons nobody can
    /// diagnose. `Unknown` is the top. See [`Membership::join`].
    #[test]
    fn an_unknown_outranks_even_an_established_disagreement() {
        let u = Membership::Unknown(Unresolved::Stale);
        assert_eq!(Membership::Mixed.join(u), u);
        assert_eq!(u.join(Membership::Mixed), u);
    }

    /// ★★ **Two unanswerable members merge by PRIORITY, not by position.**
    ///
    /// `Unknown(a) ⊔ Unknown(b)` taking the left operand would be
    /// non-commutative, which is the same order-dependence one level down —
    /// and invisible to any property test whose sample set held a single
    /// `Unknown`, which the first one did.
    #[test]
    fn two_reasons_merge_by_priority_rather_than_by_position() {
        let read = Membership::Unknown(Unresolved::PageNotDecomposed);
        let stale = Membership::Unknown(Unresolved::Stale);
        assert_eq!(read.join(stale), read);
        assert_eq!(stale.join(read), read);
        assert!(
            Unresolved::PageNotDecomposed < Unresolved::Stale,
            "the declaration order IS the priority order — see `Unresolved`"
        );
    }

    /// **The join is commutative and associative**, which is what makes the
    /// answer independent of the order `targets_on` happens to return.
    ///
    /// Exhaustive over a representative set rather than argued in prose: a
    /// fold whose result depended on selection order would produce a highlight
    /// that flickered between two rows as the operator added objects, and no
    /// single-pair test would catch it.
    #[test]
    fn the_fold_does_not_depend_on_selection_order() {
        // ★★★ TWO different `Unknown`s, deliberately. The first version of
        // this array held one, and with one the commutativity of
        // `Unknown ⊔ Unknown` is unobservable — a "first operand wins" rule
        // would have passed. The associativity failure it DID catch was found
        // by the `Mixed` entry sitting beside a `Group` pair; both entries are
        // load-bearing and neither is decoration.
        let all = [
            Membership::NothingSelected,
            Membership::Group(oc(1)),
            Membership::Group(oc(2)),
            Membership::None,
            Membership::Unknown(Unresolved::PageNotDecomposed),
            Membership::Unknown(Unresolved::Stale),
            Membership::Mixed,
        ];
        for a in all {
            for b in all {
                assert_eq!(a.join(b), b.join(a), "join is not commutative: {a:?} {b:?}");
                for c in all {
                    assert_eq!(
                        a.join(b).join(c),
                        a.join(b.join(c)),
                        "join is not associative: {a:?} {b:?} {c:?}"
                    );
                }
            }
        }
    }
}
