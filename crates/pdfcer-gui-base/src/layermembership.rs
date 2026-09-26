//! # `layermembership` — which layer a selection belongs to, as a value
//!
//! The verdict and its pure rules, kept apart from the code that reads a
//! document so the Layers panel copy can name them.
//! `pdfcer_gui::panels::layers::highlight` resolves a selection to one.

use pdfcer_core::object::ObjId;

/// **Why pdfcer cannot name the layer**, when it cannot.
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
    /// `pdfcer_core::vector::DecomposeDiagnostics::oc_unresolved` is
    /// non-zero.
    ///
    /// **Page-scoped, not object-scoped**, because the counter is. The
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
    /// See `Membership::kind` for why this is not `{:?}`.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// **Nothing is selected**, so there is no question to answer.
    ///
    /// Distinct from `Self::Unknown`, which means something *is* selected
    /// and pdfcer cannot say. Both render nothing, and they are still
    /// different: a caller counting "how often can we not answer" must not
    /// count an empty canvas as a failure. It is also the **identity** of
    /// `Self::join`, which is what lets a fold over an empty selection
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
    /// It may name an **OCMD** rather than an OCG (§8.11.2.2): both an
    /// annotation's `/OC` and a content section's `/Pn` are allowed to be
    /// either, and the engine reports membership without expanding. An OCMD's
    /// id will not match any row, and the panel says so in words rather than
    /// letting the silence read as "no layer" — see
    /// `crate::text::panels::layers::layer_selection_report`'s off-list arm.
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
    /// It is a *variant* rather than an absence so that a test can assert
    /// about it — "we stopped being able to answer" and "we never could" are
    /// distinguishable in the suite — and so that each cause can carry its own
    /// sentence.
    Unknown(Unresolved),
    /// **The selection spans more than one layer**, or mixes layered and
    /// unlayered objects.
    ///
    /// Above the atoms in `Self::join`'s lattice and **below**
    /// `Self::Unknown`. That ordering was the other way round for one
    /// afternoon, on the reasoning that *"this selection spans several layers"*
    /// is a positively established fact a later unanswerable member cannot take
    /// back — which is true, and which **broke associativity**:
    /// `(G₁⊔G₂)⊔U = Mixed` while `G₁⊔(G₂⊔U) = Unknown`, so the highlight would
    /// have depended on the order the selection happened to be folded in.
    /// `the_fold_does_not_depend_on_selection_order` caught it. See
    /// `Self::join`.
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
    /// Not `{:?}`. A derived `Debug` prints `Group(ObjId { number: 4,
    /// generation: 0 })` — braces and spaces, which the harness's `key=value`
    /// trace parser splits into five tokens and reads as none of them. Worse,
    /// it would change spelling the day a field is added to `ObjId`, silently
    /// retiring every check that matched on it.
    ///
    /// The words are deliberately **not** the operator's sentences. A trace
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
    /// `"-"` and not `""` for the states that have none. The trace parser
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
    /// ## The ordering that is NOT obvious, and it cost a wrong design
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
    /// ## And `Unknown(a) ⊔ Unknown(b)` takes the SMALLER reason
    ///
    /// Not the left one. Two `Unknown`s with different reasons and a
    /// "first wins" rule is **not commutative** — `U(a) ⊔ U(b)` would differ
    /// from `U(b) ⊔ U(a)`, which is the same order-dependence one level down
    /// and would have been invisible to any test whose sample set held one
    /// `Unknown`. `Unresolved`'s declaration order is the priority order and
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
