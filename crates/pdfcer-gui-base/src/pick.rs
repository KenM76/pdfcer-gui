//! # `pick` — WHAT a click is allowed to land on
//!
//! ## The question this module answers, and the one it deliberately does not
//!
//! Every press on the page eventually asks two separate questions, and until
//! this module existed the shell only had a vocabulary for the second:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pick.md`.

use crate::annotkind::AnnotKind;
use crate::objectsummary::ObjectKind;

/// One class of thing a click may be allowed to land on.
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
    /// Note the asymmetry, which is deliberate: filling a field is **not**
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
    #[must_use]
    pub const fn of_annot(kind: AnnotKind) -> PickClass {
        match kind {
            AnnotKind::Markup => PickClass::Markup,
            AnnotKind::CeDimension => PickClass::CeDimension,
        }
    }
}

/// Which classes of thing a click may currently land on.
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
    #[must_use]
    pub const fn all() -> Self {
        Self {
            allowed: [true; PickClass::COUNT],
        }
    }

    /// Nothing selectable at all.
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
