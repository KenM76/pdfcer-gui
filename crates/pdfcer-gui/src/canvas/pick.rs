//! # `canvas::pick` — WHAT a click is allowed to land on
//!
//! ## The question this module answers, and the one it deliberately does not
//!
//! Every press on the page eventually asks two separate questions, and until
//! this module existed the shell only had a vocabulary for the second:
//!
//! | question | answered by |
//! |---|---|
//! | **May a click land on this *class* of thing at all?** | here — [`PickFilter`] |
//! | Where, in the stack of things it *may* land on, does this particular click land? | [`crate::canvas::input::probe`], [`crate::canvas::selection::annot`] |
//!
//! Those look similar and are not. The second is geometry: tolerance, paint
//! order, distance-to-segment, topmost-wins. The first is **operator intent**,
//! it does not vary with the pointer, and it is exactly the thing a CAD
//! package parks permanently on screen so it can be glanced at rather than
//! remembered.
//!
//! ## ★ Why this replaced a pair of ribbon buttons, in the operator's words
//!
//! `OPERATOR_REQUESTS.md` O17, 2026-08-21:
//!
//! > *"On the bottom bar I want a filter menu that pops up with all the
//! > options of what to enable selecting of — text, points, lines, etc — all
//! > the object types … This is to replace the wonky content edit text and
//! > edit objects menu at the top."*
//!
//! The diagnosis of *wonky* is worth carrying in the source, because it is a
//! statement about gesture design rather than about two buttons. Edit ▸
//! Content asks the operator to **declare an intention before pointing at
//! anything** — *I am now editing text* — and the hit test then obeys the
//! declaration rather than the drawing. Three consequences follow, all of them
//! reported:
//!
//! 1. The same click on the same pixel means different things depending on a
//!    control the operator is not looking at while they click.
//! 2. Making a class of thing reachable costs a mode change plus two levels of
//!    ribbon travel — `FEATURES.md` records the measured ritual for typing one
//!    character as **four steps**.
//! 3. The state is invisible at the moment it matters. A ribbon button pressed
//!    thirty seconds ago is not on screen while you aim.
//!
//! A filter on the status bar has none of those properties: it is always
//! visible, it is one click from anywhere, and it says what it is doing while
//! you do it.
//!
//! ## ★★ The invariant: a filter is SUBTRACTIVE, always
//!
//! **A [`PickFilter`] can only ever take candidates away. It can never make
//! something pickable that was not pickable without it.**
//!
//! This is the most important property here and it is worth being explicit
//! about why it was chosen, because the obvious alternative is tempting and
//! wrong.
//!
//! The tempting version: *"Nodes ON means a click lands directly on the
//! nearest node, without descending into the object first."* That would be a
//! real convenience, and it would also re-open a defect this project already
//! measured and closed. One CAD export in the fixture set holds **6,681
//! anchors in a single path object** and **1,194 subpaths**; offering all of
//! them to every press is what made the old ungated gesture unpredictable,
//! because the nearest anchor to a press routinely belongs to a subpath the
//! operator was not pointing at, with nothing drawn beforehand to say which.
//! [`crate::canvas::selection::SelectionLevel`] exists to scope that, and
//! [`crate::canvas::input::probe`] is built around the scoping.
//!
//! So the ladder is left exactly as it is, and the filter sits *outside* it:
//! it decides which rungs and which classes are **eligible**, and the existing
//! geometry decides which eligible thing wins. Direct one-click node picking
//! already has a home — the Node tool, `A`, which calls
//! [`crate::canvas::selection::SelectionState::click_direct`] and skips the
//! descent ritual by design. The filter governs whether that tool may pick,
//! not whether every press behaves like it.
//!
//! Two properties fall out of subtractiveness, and both are load-bearing:
//!
//! - **`PickFilter::default()` reproduces today's behaviour exactly**, so R6
//!   ("nothing regresses") holds by construction rather than by testing every
//!   path twice. The default is *everything the shell can currently pick*.
//! - **The filter can never contradict a capability.** It is an `AND`, not an
//!   override — see the next section.
//!
//! ## ★ The filter sits ABOVE the mode, which is not the same as replacing it
//!
//! O17 is explicit: *"In all three modes the filter is authoritative. A class
//! switched off in the filter is not selectable in Read, not selectable in
//! Review, not selectable in Edit."*
//!
//! That is a statement about one direction only. The composition is:
//!
//! ```text
//! pickable(class) = capability_allows(class, mode) && filter.allows(class)
//! ```
//!
//! **Both must be true.** Switching a class ON in the filter does not grant
//! Read mode the ability to edit content, and nothing here can. Capabilities
//! (`crate::app::modes::capability::Capabilities`) remain the mode's answer to
//! *what may be authored*; the filter is the operator's answer to *what I am
//! currently interested in pointing at*. They are different questions with
//! different owners, and collapsing them is how a filter turns into a hole in
//! the mode system.
//!
//! ## Why the class list is derived and not invented
//!
//! Every variant of [`PickClass`] corresponds to something the existing hit
//! test can already **distinguish**. That constraint is what keeps the popup
//! honest: a row the operator can switch off but which nothing consults is a
//! lie told once per session, and a class the hit test can separate but which
//! has no row is a thing the operator cannot reach.
//!
//! | [`PickClass`] | derived from | distinguished by |
//! |---|---|---|
//! | [`PickClass::Text`] | `VectorObject::Text` | `panels::objects::summary::object_kind` |
//! | [`PickClass::Path`] | `VectorObject::Path` | same |
//! | [`PickClass::Image`] | `ImageSource::Inline` / `ImageSource::XObject` | same |
//! | [`PickClass::FormXObject`] | `ImageSource::Form` | same |
//! | [`PickClass::Part`] | the `Part` rung | `ObjectModelProvider::part_kind` |
//! | [`PickClass::Node`] | the `Node` rung | `CanvasTargetProvider::nearest_node` |
//! | [`PickClass::Markup`] | `AnnotKind::Markup` | `canvas::selection::annot` |
//! | [`PickClass::CeDimension`] | `AnnotKind::CeDimension` | same |
//! | [`PickClass::FormField`] | `/Widget` annotations | `annot::selectable_on`'s exclusions |
//! | [`PickClass::Link`] | `/Link` annotations | same |
//! | [`PickClass::Characters`] | the character sweep | `canvas::textsel` |
//!
//! ### ★ Two rows are RUNGS, not object kinds, and they belong here anyway
//!
//! `Part` and `Node` are levels of
//! [`crate::canvas::selection::SelectionLevel`], not variants of any object
//! enum. Mixing them into one list with `Text` and `Image` is a category error
//! on the implementation's terms and is nonetheless correct on the operator's,
//! which is the side that matters for a control they read.
//!
//! From the pointing end, *"can I click a corner point?"* is the same shape of
//! question as *"can I click a piece of text?"* — both are *"is this kind of
//! thing live right now"*. The operator asked for them in one breath and in
//! one list: *"text, points, lines, etc"*. Splitting them across two popups to
//! honour an internal distinction would be the shell explaining its own
//! architecture to somebody who is trying to click a corner.
//!
//! ### One class is OFF by default, and it is not an oversight
//!
//! [`PickClass::Link`]. `annot::selectable_on` excludes `/Link` from selection
//! today, so there is no path by which a link can be picked, and a filter row
//! defaulting to ON would claim a capability that does not exist. It has a row
//! because links are a thing on the page the operator can see and will
//! eventually want to reach; it defaults OFF because subtractiveness means the
//! default must describe what the shell *does*, not what it should.
//!
//! When link picking lands, the default flips here and nowhere else.
//!
//! ## What this module does NOT do
//!
//! It never draws, never touches egui, never reads a pointer, never reaches a
//! document, and never decides which class won a click. It is a set of
//! booleans over a closed enum, plus the two mappings that connect that enum
//! to the classifiers that already exist. Every claim in this header is
//! therefore assertable in a unit test rather than hoped for in a running
//! window — which, per R1, is the *floor*, not the ceiling: the popup that
//! drives this has to be driven before any of it counts as working.

use crate::canvas::selection::AnnotKind;
use crate::panels::objects::summary::ObjectKind;

/// One class of thing a click may be allowed to land on.
///
/// The list is closed and every variant is something the hit test can already
/// tell apart — see this module's header for the derivation table and for why
/// two of these are selection *rungs* rather than object kinds.
///
/// # Ordering
///
/// The declaration order is the **display order** of the popup, grouped the
/// way a person reads a drawing rather than the way the decomposer emits
/// objects: the marks on the page first (text, lines, pictures), then the
/// finer rungs inside them, then the things pdfcer or another program added on
/// top (markup, dimensions, fields, links), then the character sweep, which is
/// a different gesture wearing the same pointer.
///
/// Persisting relies on [`PickClass::token`], never on this order, so the
/// order may be changed for display reasons without invalidating a saved
/// filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PickClass {
    /// A `BT`…`ET` text object, picked as one whole object.
    ///
    /// Distinct from [`PickClass::Characters`]: this is *the text run as a
    /// thing you can move and restyle*, that is *the letters you sweep to
    /// copy*.
    Text,
    /// A path object — `re`/`m`/`l`/`c` followed by a painting operator.
    ///
    /// The operator's *"lines"*. Everything drawn as geometry lands here:
    /// leader lines, hatching, borders, the drawing itself.
    Path,
    /// A raster picture — an inline image (`BI`/`ID`/`EI`) or a `Do` on an
    /// image XObject.
    ///
    /// The two are one row because the difference is a storage detail the
    /// operator has no way to see and no reason to filter on separately.
    Image,
    /// A `Do` on a **form** XObject — an entire nested drawing treated as one
    /// opaque object.
    ///
    /// Its own row rather than folded into [`PickClass::Image`], because this
    /// is the single most common cause of *"why is the selection box so
    /// big?"* on a CAD sheet: a title block or a border that is one object
    /// holding a hundred visible marks. Being able to switch it off is
    /// precisely the relief that complaint asks for.
    FormXObject,
    /// The `Part` rung — a path's subpath, or a text object's show-operator
    /// run.
    ///
    /// A rung, not an object kind. Switching it off pins selection at whole
    /// objects: a double-click stops descending and the sheet behaves like a
    /// diagram of boxes rather than of geometry.
    Part,
    /// The `Node` rung — an anchor on a subpath. The operator's *"points"*.
    ///
    /// A rung, not an object kind. Off means anchors are never picked, by
    /// descent or by the Node tool, and no anchor is offered as a drag target.
    Node,
    /// An annotation pdfcer authored that is not a ce dimension — a shape, a
    /// note, a stamp, a text markup.
    Markup,
    /// A **ce dimension**: a `/Line` carrying `/IT /LineDimension` plus its
    /// record in the document's `/PieceInfo` sidecar.
    ///
    /// Its own row because it is the one class an operator measuring a drawing
    /// wants isolated — *"let me grab my own dimensions and nothing else"* is
    /// the whole reason a filter is useful on a dense sheet.
    CeDimension,
    /// A `/Widget` annotation — one form field on the page.
    ///
    /// ★ Note the asymmetry, which is deliberate: filling a field is **not**
    /// gated by mode (`Capabilities` leaves it alone, because Acrobat Reader
    /// fills forms), so this row is the only control over whether a click
    /// reaches a field at all.
    FormField,
    /// A `/Link` annotation.
    ///
    /// **Defaults OFF**, alone among these, because nothing can pick a link
    /// today — see the header. The row exists so the eventual capability has
    /// somewhere to appear rather than needing a popup redesign.
    Link,
    /// The character sweep — dragging across text to copy it.
    ///
    /// A gesture rather than an object, and it earns a row because it is the
    /// one thing that currently *competes* with object selection for a plain
    /// press: in Read and Review a click on the canvas is a sweep, not a
    /// selection. An operator who wants to click a picture in Read needs a way
    /// to say so, and this is it.
    Characters,
}

impl PickClass {
    /// Every class, in display order. The popup renders exactly this.
    pub const ALL: [PickClass; 11] = [
        PickClass::Text,
        PickClass::Path,
        PickClass::Image,
        PickClass::FormXObject,
        PickClass::Part,
        PickClass::Node,
        PickClass::Markup,
        PickClass::CeDimension,
        PickClass::FormField,
        PickClass::Link,
        PickClass::Characters,
    ];

    /// How many classes there are. The width of [`PickFilter`]'s array.
    pub const COUNT: usize = PickClass::ALL.len();

    /// This class's slot in [`PickFilter`]'s array.
    ///
    /// A `match` rather than a cast, so adding a variant is a compile error
    /// here instead of a silently wrong index.
    #[must_use]
    const fn index(self) -> usize {
        match self {
            PickClass::Text => 0,
            PickClass::Path => 1,
            PickClass::Image => 2,
            PickClass::FormXObject => 3,
            PickClass::Part => 4,
            PickClass::Node => 5,
            PickClass::Markup => 6,
            PickClass::CeDimension => 7,
            PickClass::FormField => 8,
            PickClass::Link => 9,
            PickClass::Characters => 10,
        }
    }

    /// The stable identifier this class is **persisted** under.
    ///
    /// Not a label — see `crate::text::pick` for what the operator reads.
    /// Persisting by name rather than by bit position is what lets the display
    /// order above be rearranged, and lets a new class be inserted anywhere,
    /// without silently re-interpreting a saved file as a different set of
    /// choices.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Text => "text",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Path => "path",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Image => "image",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::FormXObject => "form-xobject",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Part => "part",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Node => "node",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Markup => "markup",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::CeDimension => "ce-dimension",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::FormField => "form-field",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Link => "link",
            // ui-text-exempt: stable persistence identifiers, never displayed.
            PickClass::Characters => "characters",
        }
    }

    /// The class a persisted [`PickClass::token`] names, or `None` if nothing
    /// does.
    ///
    /// `None` is not an error at the call site: see [`PickFilter::from_tokens`]
    /// for why an unrecognised token is skipped rather than rejected.
    #[must_use]
    pub fn from_token(token: &str) -> Option<PickClass> {
        PickClass::ALL.into_iter().find(|c| c.token() == token)
    }

    /// Whether this class is picked at all in a shell with no operator
    /// preference saved.
    ///
    /// **This function is the R6 guarantee.** It must answer `true` for every
    /// class the shell can pick today and `false` for every class it cannot,
    /// so that a fresh install behaves exactly as the shell behaved before the
    /// filter existed. Changing an answer here is changing default behaviour,
    /// and is a decision rather than a tidy-up.
    #[must_use]
    pub const fn on_by_default(self) -> bool {
        match self {
            // Nothing can pick a `/Link` today — `annot::selectable_on`
            // excludes the subtype outright. A row defaulting to ON would
            // promise a capability that does not exist.
            PickClass::Link => false,
            _ => true,
        }
    }

    /// The class a decomposed page object belongs to.
    ///
    /// ★ Takes an [`ObjectKind`] rather than a `VectorObject`, so that
    /// `panels::objects::summary::object_kind` stays **the** classifier. That
    /// module's header is explicit that a second kind classifier is the exact
    /// divergence it exists to prevent, and this is where a second one would
    /// otherwise have been written.
    #[must_use]
    pub const fn of_object(kind: ObjectKind) -> PickClass {
        match kind {
            ObjectKind::Path => PickClass::Path,
            ObjectKind::Text => PickClass::Text,
            ObjectKind::InlineImage | ObjectKind::ImageXObject => PickClass::Image,
            ObjectKind::FormXObject => PickClass::FormXObject,
        }
    }

    /// The class a selectable annotation belongs to.
    ///
    /// Only covers the two kinds [`AnnotKind`] distinguishes. `/Widget` and
    /// `/Link` never reach an `AnnotKind` — `annot::selectable_on` drops them
    /// before one is built — which is why [`PickClass::FormField`] and
    /// [`PickClass::Link`] have no arm here and are consulted at their own call
    /// sites instead.
    #[must_use]
    pub const fn of_annot(kind: AnnotKind) -> PickClass {
        match kind {
            AnnotKind::Markup => PickClass::Markup,
            AnnotKind::CeDimension => PickClass::CeDimension,
        }
    }
}

/// Which classes of thing a click may currently land on.
///
/// `Copy`, eleven booleans wide, cheap enough to pass by value into every hit
/// test on every frame — which is the point. A filter that had to be borrowed
/// or looked up would grow call sites that skip it, and a hit test that skips
/// the filter is precisely the *"visible control, silently inert"* failure
/// convention C7 names.
///
/// # The array, rather than a bitmask
///
/// A `u16` of flags would be smaller and would persist as one number. It would
/// also make every read a shift-and-mask whose correctness depends on a
/// constant matching a variant, and it would tempt a future reader into
/// serialising the raw integer — which is the one representation that cannot
/// survive inserting a class in the middle. Eleven `bool`s cost eleven bytes
/// and are read by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickFilter {
    /// Indexed by [`PickClass::index`]. Private: every access goes through
    /// [`PickFilter::allows`] or [`PickFilter::set`], so there is one place a
    /// future "…and also check X" can be added and no way to miss it.
    allowed: [bool; PickClass::COUNT],
}

impl Default for PickFilter {
    /// The filter a shell with no saved preference starts with: exactly what
    /// the shell could pick before this module existed.
    ///
    /// See [`PickClass::on_by_default`] — that function is the whole of the
    /// definition, and this one only walks it.
    fn default() -> Self {
        let mut allowed = [false; PickClass::COUNT];
        let mut i = 0;
        while i < PickClass::COUNT {
            let class = PickClass::ALL[i];
            allowed[class.index()] = class.on_by_default();
            i += 1;
        }
        Self { allowed }
    }
}

impl PickFilter {
    /// Every class on, including the ones that are off by default.
    ///
    /// ★ Deliberately **not** the same as [`PickFilter::default`], and the
    /// difference is the honest one: `default()` describes what the shell can
    /// do, `all()` describes what the popup can express. `Link` is on here and
    /// off there. Switching it on still picks nothing until link picking
    /// exists, which is a truth about the shell rather than about this type.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            allowed: [true; PickClass::COUNT],
        }
    }

    /// Nothing selectable at all.
    ///
    /// A legitimate state, not a degenerate one: it is how an operator says
    /// *"I am panning and reading, do not let me grab anything by accident"*,
    /// which on a dense drawing is a real request. The popup must therefore
    /// **not** guard against it — but it must make it obvious, because a
    /// canvas that has stopped responding to clicks is otherwise
    /// indistinguishable from a broken one.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            allowed: [false; PickClass::COUNT],
        }
    }

    /// Whether a click may land on `class`.
    ///
    /// The one read. Everything that hit-tests asks this and nothing reads the
    /// array directly.
    #[must_use]
    pub const fn allows(&self, class: PickClass) -> bool {
        self.allowed[class.index()]
    }

    /// Turn one class on or off, returning the new filter.
    // ui-text-exempt: a compiler lint message, read by developers in `cargo
    // build` output and never rendered by the application.
    #[must_use = "PickFilter is Copy; this returns a new filter and does not mutate in place"]
    pub const fn with(mut self, class: PickClass, on: bool) -> Self {
        self.allowed[class.index()] = on;
        self
    }

    /// Turn one class on or off, in place.
    pub const fn set(&mut self, class: PickClass, on: bool) {
        self.allowed[class.index()] = on;
    }

    /// Flip one class.
    pub const fn toggle(&mut self, class: PickClass) {
        self.allowed[class.index()] = !self.allowed[class.index()];
    }

    /// Whether every class is on.
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.allowed.iter().all(|on| *on)
    }

    /// Whether no class is on — the state in which a click can select nothing.
    ///
    /// Exposed so the status bar can *say so*. An operator who has switched
    /// everything off and forgotten will otherwise report the canvas as
    /// broken, and they will be right to.
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.allowed.iter().all(|on| !*on)
    }

    /// How many classes are on. For the status bar's summary.
    #[must_use]
    pub fn count(&self) -> usize {
        self.allowed.iter().filter(|on| **on).count()
    }

    /// Every class that is currently on, in display order.
    #[must_use]
    pub fn enabled(&self) -> Vec<PickClass> {
        PickClass::ALL
            .into_iter()
            .filter(|c| self.allows(*c))
            .collect()
    }

    /// Serialise to a space-separated list of the tokens that are **on**.
    ///
    /// # Why the enabled set and not a full assignment
    ///
    /// A `text=1 path=0 …` form would round-trip more obviously and would also
    /// force a decision this format gets to avoid: what a *missing* key means
    /// after a new class is added. Recording only what is on makes the answer
    /// structural — a class the file does not mention was not on when the file
    /// was written — and see [`PickFilter::from_tokens`] for why that is still
    /// not quite the whole answer.
    #[must_use]
    pub fn to_tokens(&self) -> String {
        self.enabled()
            .into_iter()
            .map(PickClass::token)
            .collect::<Vec<_>>()
            // ui-text-exempt: the separator of an on-disk persistence format,
            // never displayed. `from_tokens` splits on any whitespace.
            .join(" ")
    }

    /// Parse what [`PickFilter::to_tokens`] wrote.
    ///
    /// # ★ The three decisions in this function, none of them obvious
    ///
    /// **1. An unrecognised token is skipped, not rejected.** A file written by
    /// a newer build naming a class this one has never heard of is not corrupt;
    /// it is from the future. Rejecting the file would discard ten good choices
    /// because of one unknown eleventh, and would do it silently at startup,
    /// which is the worst possible moment.
    ///
    /// **2. A class the file does not mention is OFF, not defaulted.** This is
    /// the opposite of decision 1 and it is deliberate. Once a file exists it is
    /// a complete statement of what the operator switched on; falling back to
    /// the default for an unmentioned class would resurrect classes the operator
    /// had explicitly turned off, every restart, which is the exact *"a
    /// rearrangeable thing that forgets is worse than a fixed one"* failure
    /// `crate::app::persistence` was written to avoid.
    ///
    /// **3. Empty input yields [`PickFilter::none`], not
    /// [`PickFilter::default`].** It follows from decision 2 and is called out
    /// because it looks like a bug and is not: an operator who switched every
    /// class off and quit gets their canvas back exactly as they left it. **The
    /// caller decides what "no file at all" means** — that is a different
    /// condition from "an empty file", and only the caller can tell them apart.
    #[must_use]
    pub fn from_tokens(text: &str) -> Self {
        let mut filter = Self::none();
        for token in text.split_whitespace() {
            if let Some(class) = PickClass::from_token(token) {
                filter.set(class, true);
            }
        }
        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R6, stated as a test: a fresh shell picks everything it could pick
    /// before the filter existed.
    #[test]
    fn the_default_allows_everything_the_shell_can_currently_pick() {
        let filter = PickFilter::default();
        assert!(filter.allows(PickClass::Text));
        assert!(filter.allows(PickClass::Path));
        assert!(filter.allows(PickClass::Image));
        assert!(filter.allows(PickClass::FormXObject));
        assert!(filter.allows(PickClass::Part));
        assert!(filter.allows(PickClass::Node));
        assert!(filter.allows(PickClass::Markup));
        assert!(filter.allows(PickClass::CeDimension));
        assert!(filter.allows(PickClass::FormField));
        assert!(filter.allows(PickClass::Characters));
    }

    /// The one class that is off by default, and the reason is that nothing can
    /// pick it. If link picking lands and this test is not updated, the failure
    /// is loud and points at the right paragraph.
    #[test]
    fn links_are_off_by_default_because_nothing_can_pick_one_yet() {
        assert!(!PickFilter::default().allows(PickClass::Link));
        assert!(!PickClass::Link.on_by_default());
    }

    /// `all()` and `default()` are different kinds of claim and must not
    /// silently converge — see `PickFilter::all`'s doc comment.
    #[test]
    fn all_is_not_the_same_as_default() {
        assert_ne!(PickFilter::all(), PickFilter::default());
        assert!(PickFilter::all().is_all());
        assert!(!PickFilter::default().is_all());
    }

    /// Every class has a distinct slot. A duplicated index would make two rows
    /// of the popup control one boolean, which reads as one of them being
    /// broken.
    #[test]
    fn every_class_has_its_own_slot() {
        let mut seen = std::collections::BTreeSet::new();
        for class in PickClass::ALL {
            assert!(seen.insert(class.index()), "duplicate index for {class:?}");
            assert!(class.index() < PickClass::COUNT);
        }
        assert_eq!(seen.len(), PickClass::COUNT);
    }

    /// Every class has a distinct persistence token, and it round-trips.
    #[test]
    fn every_class_has_its_own_token_and_it_round_trips() {
        let mut seen = std::collections::BTreeSet::new();
        for class in PickClass::ALL {
            assert!(
                seen.insert(class.token()),
                "duplicate token for {class:?}: {}",
                class.token()
            );
            assert_eq!(PickClass::from_token(class.token()), Some(class));
        }
    }

    /// Setting one class must not disturb another. Trivially true of an array
    /// and emphatically not of the bitmask this deliberately is not.
    #[test]
    fn toggling_one_class_leaves_the_others_alone() {
        let mut filter = PickFilter::all();
        filter.set(PickClass::Path, false);
        assert!(!filter.allows(PickClass::Path));
        for class in PickClass::ALL {
            if class != PickClass::Path {
                assert!(filter.allows(class), "{class:?} was disturbed");
            }
        }
    }

    #[test]
    fn toggle_flips_and_flips_back() {
        let mut filter = PickFilter::default();
        let before = filter.allows(PickClass::Node);
        filter.toggle(PickClass::Node);
        assert_ne!(filter.allows(PickClass::Node), before);
        filter.toggle(PickClass::Node);
        assert_eq!(filter.allows(PickClass::Node), before);
    }

    #[test]
    fn a_filter_round_trips_through_its_tokens() {
        let filter = PickFilter::default()
            .with(PickClass::Path, false)
            .with(PickClass::Link, true);
        assert_eq!(PickFilter::from_tokens(&filter.to_tokens()), filter);
    }

    /// Decision 1 in `from_tokens`: a class from a newer build is skipped and
    /// the rest of the line survives.
    #[test]
    fn an_unknown_token_is_skipped_and_the_rest_of_the_line_survives() {
        // ui-text-exempt: persistence tokens under test, never displayed.
        let filter = PickFilter::from_tokens("text something-from-a-newer-build path");
        assert!(filter.allows(PickClass::Text));
        assert!(filter.allows(PickClass::Path));
        assert!(!filter.allows(PickClass::Image));
        assert_eq!(filter.count(), 2);
    }

    /// Decision 2: an unmentioned class is OFF, so a class the operator turned
    /// off stays off across a restart instead of being resurrected by the
    /// default.
    #[test]
    fn an_unmentioned_class_is_off_rather_than_defaulted() {
        // ui-text-exempt: persistence token under test, never displayed.
        let filter = PickFilter::from_tokens("text");
        assert!(filter.allows(PickClass::Text));
        for class in PickClass::ALL {
            if class != PickClass::Text {
                assert!(!filter.allows(class), "{class:?} was resurrected");
            }
        }
    }

    /// Decision 3: empty input is "the operator switched everything off", not
    /// "there is no preference". Only the caller can tell those apart.
    #[test]
    fn empty_input_means_nothing_selectable_not_the_default() {
        let filter = PickFilter::from_tokens("   ");
        assert!(filter.is_none());
        assert_ne!(filter, PickFilter::default());
    }

    #[test]
    fn nothing_selectable_is_representable_and_reports_itself() {
        let filter = PickFilter::none();
        assert!(filter.is_none());
        assert_eq!(filter.count(), 0);
        assert!(filter.enabled().is_empty());
        for class in PickClass::ALL {
            assert!(!filter.allows(class));
        }
    }

    #[test]
    fn count_and_enabled_agree_with_allows() {
        let filter = PickFilter::default();
        assert_eq!(filter.count(), filter.enabled().len());
        assert_eq!(filter.count(), PickClass::COUNT - 1); // every class but Link
        for class in filter.enabled() {
            assert!(filter.allows(class));
        }
    }

    /// The object classifier must agree with `panels::objects::summary`, which
    /// is the single classifier this deliberately delegates to.
    #[test]
    fn object_kinds_map_to_the_class_the_operator_would_name() {
        assert_eq!(PickClass::of_object(ObjectKind::Path), PickClass::Path);
        assert_eq!(PickClass::of_object(ObjectKind::Text), PickClass::Text);
        assert_eq!(
            PickClass::of_object(ObjectKind::InlineImage),
            PickClass::Image
        );
        assert_eq!(
            PickClass::of_object(ObjectKind::ImageXObject),
            PickClass::Image
        );
        assert_eq!(
            PickClass::of_object(ObjectKind::FormXObject),
            PickClass::FormXObject
        );
    }

    /// A form XObject must NOT collapse into `Image`. It is its own row for a
    /// reported reason — the oversized selection box on a CAD title block — and
    /// collapsing it would silently remove the relief.
    #[test]
    fn a_form_xobject_is_not_an_image() {
        assert_ne!(
            PickClass::of_object(ObjectKind::FormXObject),
            PickClass::of_object(ObjectKind::ImageXObject)
        );
    }

    #[test]
    fn annot_kinds_map_to_their_own_rows() {
        assert_eq!(PickClass::of_annot(AnnotKind::Markup), PickClass::Markup);
        assert_eq!(
            PickClass::of_annot(AnnotKind::CeDimension),
            PickClass::CeDimension
        );
    }

    /// `ALL` must actually be all of them. A variant added to the enum and
    /// forgotten here would be a class with no popup row — invisible, and
    /// therefore unreachable.
    #[test]
    fn all_lists_every_variant_exactly_once() {
        let unique: std::collections::BTreeSet<_> = PickClass::ALL.into_iter().collect();
        assert_eq!(unique.len(), PickClass::ALL.len());
        assert_eq!(PickClass::COUNT, 11);
    }
}
