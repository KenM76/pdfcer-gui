//! The dock's owned layout model — the value that is arranged, saved,
//! restored, named and reset.
//!
//! # What a layout is
//!
//! ```text
//! DockLayout
//! ├── left : SideLayout          width_pts, visible
//! │   └── columns : [Column]     share
//! │       └── stacks : [Stack]   share
//! │           └── tabs : [PanelId], active
//! └── right : SideLayout
//! ```
//!
//! Four levels, and each one exists because a real arrangement needs it:
//!
//! - **Side** — left and right. `MODES_AND_PANELS.md`'s capability table
//!   lists two-sided docking as the baseline every peer product has.
//! - **Column** — several side-by-side columns *within* one side. The
//!   benchmarked application has this and VS Code does not; it is what
//!   lets an operator put a narrow navigator beside a wide inspector
//!   without giving up either.
//! - **Stack** — vertical compartments within a column, each with a
//!   draggable boundary. This is the shape the previous implementation's
//!   `default_left_tree` built by hand, and its reasoning is worth
//!   carrying over verbatim: *"reaching one surface must not hide another
//!   you are using AT THE SAME TIME"*. A stack boundary is how two
//!   surfaces stay visible together.
//! - **Tab** — several panels in one stack, one visible at a time. The
//!   compact form, for surfaces that are consulted in bursts rather than
//!   watched continuously.
//!
//! # Why the shell owns this type rather than borrowing a library's
//!
//! Three independent reasons, and the first two would each be sufficient.
//!
//! 1. **It versions independently of any layout engine.** A tiling
//!    library's own serialized form is a description of *its* internal
//!    state — tile ids, container kinds, simplification flags. Persisting
//!    that couples the operator's saved arrangement to a crate version.
//!    An upgrade then either migrates it or discards it, and the
//!    discarding kind is what produces "the application forgot my
//!    layout" bug reports.
//! 2. **It contains no ids that mean nothing across a restart.** Every
//!    identifier in this file is either a [`PanelId`] the *application*
//!    chose, or a positional index. There is no generated handle to
//!    resolve, so nothing can dangle. Compare with an engine's arena
//!    keys, which are meaningless in the next process and have to be
//!    remapped on load — a step that can fail, and whose failure mode is
//!    silent.
//! 3. **The persistence had to be hand-written anyway.** The layout
//!    engine considered for this project derives `serde` behind its
//!    *default* feature set, which this workspace disables. Writing the
//!    serialization over a model this crate owns is strictly better than
//!    writing it over a model it does not.
//!
//! # The one thing this file must never learn
//!
//! A [`PanelId`] is an **opaque string the application supplies**. This
//! crate does not know, and must never come to know, what any of them
//! mean. There is no `Panel::Thumbnails` variant here and there never
//! will be; `SHELL_FRAMEWORK.md` §2 states the rule and
//! `tools/gates/check-shell-purity.sh` enforces its negative half.
//!
//! The previous implementation used a closed `enum DockPanel`, and its
//! own doc comment records the cost of that choice honestly: every panel
//! the application could ever dock had to be a variant compiled into the
//! shell, and adding one meant editing the framework. Its tests then had
//! to sweep `DockPanel::ALL` to prove no variant had become unreachable —
//! a real invariant, and one that only exists because the type was
//! closed. Here, the equivalent invariant is
//! [`DockLayout::unregistered_panels`], which the application can ask
//! about at any time; the shell has no list to sweep because it has no
//! list.
//!
//! # Fail-soft is a property of the model, not only of the loader
//!
//! [`DockLayout::normalize`] exists so that *every* path into a layout —
//! a deserialized file, a workspace restore, a programmatic edit by the
//! application, a panel closed mid-drag — converges on the same set of
//! invariants:
//!
//! | Invariant | Repaired by |
//! |---|---|
//! | no stack holds zero tabs | dropping the stack |
//! | no column holds zero stacks | dropping the column |
//! | `active` indexes a real tab | clamping to the last tab |
//! | every share is finite and positive | replacing with an equal share |
//! | no panel is mounted twice | dropping the later mount |
//!
//! The last one deserves its reasoning stated, because the previous
//! implementation enforced it with a test rather than with code and said
//! why: two live copies of one surface each have *"its own scroll
//! position and its own idea of which tab is active"*, and any
//! `activate` call raises whichever it finds first. *"That is a
//! state-drift bug with no visible cause, so it is cheaper to forbid than
//! to debug."* That reasoning is carried across intact; what changed is
//! that the model now repairs it rather than a test merely detecting it,
//! because a hand-edited file is a source a test cannot reach.

use serde::{Deserialize, Serialize};

use super::plan::{self, MIN_SHARE};

/// A panel's identity: **an opaque string the application supplies.**
///
/// The shell stores it, compares it, serializes it and hands it back to
/// the application's body callback. It never interprets it. See this
/// module's header on why that is a hard rule rather than a style
/// preference.
///
/// Ordering and hashing are derived so a layout can be diffed and a set
/// of panels can be addressed cheaply; the ordering is lexicographic on
/// the id and carries no meaning of its own.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PanelId(String);

impl PanelId {
    /// Wrap a string as a panel id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the id, yielding the string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for PanelId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for PanelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Which side of the window a dock occupies.
///
/// Two sides, deliberately, and not four. `MODES_AND_PANELS.md`'s peer
/// table shows one product with four edges and every other with two; a
/// top or bottom dock competes for space with the ribbon above and the
/// status bar below, and the shell already owns both of those. Adding
/// them later is a variant plus two match arms, and the serialized form
/// names sides by keyword rather than by index precisely so that adding
/// one does not renumber the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DockSide {
    /// The dock on the leading edge of the window.
    Left,
    /// The dock on the trailing edge of the window.
    Right,
}

impl DockSide {
    /// Both sides, in the order they are drawn.
    pub const ALL: [DockSide; 2] = [DockSide::Left, DockSide::Right];

    /// A stable lowercase key, used in diagnostics and rect names.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DockSide::Left => "left",
            DockSide::Right => "right",
        }
    }
}

impl std::fmt::Display for DockSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

/// One tabbed group of panels — a compartment with a tab bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stack {
    /// The panels in this group, in tab order.
    pub tabs: Vec<PanelId>,
    /// Which tab is active, as an index into [`Self::tabs`].
    ///
    /// Stored as an index rather than as a [`PanelId`] because the tab
    /// bar's arithmetic is index-based and a second representation is a
    /// second thing to keep in step. [`DockLayout::normalize`] clamps it,
    /// so a stale value from a hand-edited file selects the last tab
    /// rather than failing the load.
    pub active: usize,
    /// This stack's share of its column's height.
    ///
    /// A relative weight, not a fraction and not a pixel count. See
    /// `super::plan::resolve_spans` for why proportional storage is
    /// what makes an un-maximise survivable.
    pub share: f32,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            share: 1.0,
        }
    }
}

impl Stack {
    /// A stack holding one panel.
    #[must_use]
    pub fn new(panel: impl Into<PanelId>) -> Self {
        Self {
            tabs: vec![panel.into()],
            ..Self::default()
        }
    }

    /// A stack holding several panels, the first of them active.
    #[must_use]
    pub fn tabbed<I, P>(panels: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PanelId>,
    {
        Self {
            tabs: panels.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Set this stack's share of its column's height.
    #[must_use]
    pub fn with_share(mut self, share: f32) -> Self {
        self.share = share;
        self
    }

    /// Make `panel` the active tab, if it is in this stack.
    #[must_use]
    pub fn with_active(mut self, panel: &PanelId) -> Self {
        if let Some(i) = self.tabs.iter().position(|p| p == panel) {
            self.active = i;
        }
        self
    }

    /// The active panel, if the stack has any tabs.
    #[must_use]
    pub fn active_panel(&self) -> Option<&PanelId> {
        self.tabs
            .get(self.active.min(self.tabs.len().saturating_sub(1)))
    }
}

/// One vertical column of stacks within a dock side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Column {
    /// The stacks in this column, top to bottom.
    pub stacks: Vec<Stack>,
    /// This column's share of its side's width.
    pub share: f32,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            stacks: Vec::new(),
            share: 1.0,
        }
    }
}

impl Column {
    /// A column holding the given stacks, top to bottom.
    #[must_use]
    pub fn new(stacks: impl IntoIterator<Item = Stack>) -> Self {
        Self {
            stacks: stacks.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Set this column's share of its side's width.
    #[must_use]
    pub fn with_share(mut self, share: f32) -> Self {
        self.share = share;
        self
    }
}

/// One side's whole arrangement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SideLayout {
    /// The columns on this side, in drawing order.
    pub columns: Vec<Column>,
    /// The side's width in **points**, not as a fraction of the window.
    ///
    /// # Why absolute here and proportional everywhere else
    ///
    /// Everything *inside* a dock is stored proportionally, because the
    /// dock's own width changes and the compartments within it should
    /// keep their relationship when it does. The dock's outer width is
    /// the opposite case: it is set against the *content*, not against
    /// the window. An operator sizes the dock so a thumbnail rail shows
    /// two columns of thumbnails; that is a number of points, and it
    /// should not double when they plug in a wider monitor. A dock that
    /// grows when you maximise steals the space maximising was for.
    ///
    /// It is nonetheless clamped **at draw time** to
    /// [`super::plan::MAX_SIDE_FRACTION`] of the window, and that clamp
    /// is never written back here — see that constant's documentation for
    /// why the not-writing-back is the whole point.
    pub width_pts: f32,
    /// Whether the side is shown at all.
    ///
    /// A hidden side keeps its arrangement. Hiding is a view state, not a
    /// destruction — which is the same distinction the mode selector
    /// draws, and the reason "collapse the dock" and "reset the dock" are
    /// different commands with different consequences.
    pub visible: bool,
}

impl Default for SideLayout {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            width_pts: 280.0,
            visible: true,
        }
    }
}

impl SideLayout {
    /// A side holding the given columns.
    #[must_use]
    pub fn new(columns: impl IntoIterator<Item = Column>) -> Self {
        Self {
            columns: columns.into_iter().collect(),
            ..Self::default()
        }
    }

    /// A side holding one column of one stack of one panel — the
    /// commonest starting arrangement.
    #[must_use]
    pub fn single(panel: impl Into<PanelId>) -> Self {
        Self::new([Column::new([Stack::new(panel)])])
    }

    /// An empty, invisible side. A dock the application does not use at
    /// all.
    #[must_use]
    pub fn none() -> Self {
        Self {
            columns: Vec::new(),
            visible: false,
            ..Self::default()
        }
    }

    /// Set the side's width in points.
    #[must_use]
    pub fn with_width(mut self, width_pts: f32) -> Self {
        self.width_pts = width_pts;
        self
    }

    /// Set whether the side is shown.
    #[must_use]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Whether this side has anything to draw.
    ///
    /// A side with no columns draws **nothing at all** — not an empty
    /// panel with a border. An empty container that still takes space is
    /// how an application ends up with a permanent grey stripe nobody can
    /// remove, and it is the same defect as a ribbon group with no items
    /// still drawing its caption.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.iter().all(|c| c.stacks.is_empty())
    }

    /// Every panel mounted on this side, in layout order.
    pub fn panels(&self) -> impl Iterator<Item = &PanelId> {
        self.columns
            .iter()
            .flat_map(|c| c.stacks.iter())
            .flat_map(|s| s.tabs.iter())
    }
}

/// Where a panel sits in a layout.
///
/// Returned by [`DockLayout::find`]. Positional rather than by handle,
/// because a handle would be exactly the kind of identifier this model
/// exists not to have — see the module header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelAddress {
    /// Which dock side.
    pub side: DockSide,
    /// Index of the column within the side.
    pub column: usize,
    /// Index of the stack within the column.
    pub stack: usize,
    /// Index of the tab within the stack.
    pub tab: usize,
}

/// The whole dock arrangement: both sides, plus anything torn out of
/// them into a window of its own.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockLayout {
    /// The leading-edge dock.
    pub left: SideLayout,
    /// The trailing-edge dock.
    pub right: SideLayout,
    /// ★★★ **The panels that are not in either dock, because the operator
    /// floated them.**
    ///
    /// A floated panel is **removed from its side** and listed here. It is
    /// deliberately not a third `SideLayout` and not a flag on a `Stack`:
    /// a floating window has no column, no share and no neighbour to
    /// splitter against, so every field of the docked model would be a
    /// field that meant nothing. What it *does* have — a desktop position,
    /// a size, and the place it came from — is exactly
    /// [`super::float::FloatingPanel`], and nothing else.
    ///
    /// ★★ **The invariant that makes the two representations safe to hold
    /// at once: a panel is in exactly one of them.** A panel both mounted
    /// on a side and listed here would be drawn twice in one frame, in two
    /// windows, from two `Ui`s that each believe they own it — and
    /// `egui` would give the two the same widget ids. [`Self::normalize`]
    /// repairs it by dropping the *float*, and
    /// [`Self::panels`] counts both lists so that `contains`, `mount` and
    /// `unregistered_panels` cannot be fooled by a panel that is floating.
    ///
    /// ★ `#[serde(default)]` on the struct is what makes this field
    /// backward-compatible with every `layout.ron` written before it
    /// existed: an old file simply has no floats, which is the truth.
    pub floating: Vec<super::float::FloatingPanel>,
}

impl DockLayout {
    /// An arrangement with two empty, invisible sides.
    ///
    /// Not [`Default`], which gives two *visible* empty sides — the
    /// difference matters because `Default` is what a deserializer
    /// reaches for when a field is missing, and a file that omits the
    /// right dock should get a right dock that draws nothing rather than
    /// one that draws a grey stripe.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            left: SideLayout::none(),
            right: SideLayout::none(),
            floating: Vec::new(),
        }
    }

    /// An arrangement with the given sides.
    #[must_use]
    pub fn new(left: SideLayout, right: SideLayout) -> Self {
        Self {
            left,
            right,
            floating: Vec::new(),
        }
    }

    /// One side, by name.
    #[must_use]
    pub fn side(&self, side: DockSide) -> &SideLayout {
        match side {
            DockSide::Left => &self.left,
            DockSide::Right => &self.right,
        }
    }

    /// One side, mutably.
    pub fn side_mut(&mut self, side: DockSide) -> &mut SideLayout {
        match side {
            DockSide::Left => &mut self.left,
            DockSide::Right => &mut self.right,
        }
    }

    /// Every panel this layout holds — **docked or floating** — in layout
    /// order, floats last.
    ///
    /// ★★ The floats are included, and that is the whole reason this is
    /// worth a doc comment. Three consumers depend on it and all three
    /// would be wrong without it:
    ///
    /// * [`Self::contains`], and through it [`Self::mount`] — so choosing
    ///   a floating panel from a View ▸ Panels menu cannot mount a second
    ///   copy of it into the dock while the first is still in a window.
    /// * [`Self::unregistered_panels`] — so a float naming a panel this
    ///   build cannot draw is reported, rather than being a window that
    ///   opens with nothing in it.
    /// * A caller asking *"what does this layout hold"* before saving a
    ///   workspace's `known_panels`.
    ///
    /// [`Self::docked_panels`] is the narrower question, for the callers
    /// that genuinely mean *in a stack*.
    pub fn panels(&self) -> impl Iterator<Item = &PanelId> {
        self.docked_panels()
            .chain(self.floating.iter().map(|f| &f.panel))
    }

    /// Every panel **mounted in a stack**, in layout order — floats
    /// excluded.
    ///
    /// The question the drawing code asks, because a float is drawn by
    /// [`super::floatwin`] and not by any side, column or stack.
    pub fn docked_panels(&self) -> impl Iterator<Item = &PanelId> {
        DockSide::ALL
            .into_iter()
            .flat_map(|s| self.side(s).panels())
    }

    /// Whether `panel` is mounted in a stack. See [`Self::docked_panels`].
    #[must_use]
    pub fn contains_docked(&self, panel: &PanelId) -> bool {
        self.docked_panels().any(|p| p == panel)
    }

    /// Whether `panel` is anywhere in this layout — docked or floating.
    #[must_use]
    pub fn contains(&self, panel: &PanelId) -> bool {
        self.panels().any(|p| p == panel)
    }

    /// Where `panel` sits, if it is mounted.
    #[must_use]
    pub fn find(&self, panel: &PanelId) -> Option<PanelAddress> {
        for side in DockSide::ALL {
            for (ci, column) in self.side(side).columns.iter().enumerate() {
                for (si, stack) in column.stacks.iter().enumerate() {
                    if let Some(ti) = stack.tabs.iter().position(|p| p == panel) {
                        return Some(PanelAddress {
                            side,
                            column: ci,
                            stack: si,
                            tab: ti,
                        });
                    }
                }
            }
        }
        None
    }

    /// Whether `panel` is the **active tab of its own stack**, i.e.
    /// whether its body is being drawn.
    ///
    /// The honest answer to "is the operator looking at this panel", and
    /// the query a command like "show Properties" must consult rather
    /// than keeping a boolean of its own. The previous implementation
    /// records why: a separate `properties_open` flag *"could disagree
    /// with what was on screen"*, and a control whose selected state is a
    /// stale copy of the truth is worse than one with no state at all.
    ///
    /// Note the deliberate limit of the claim: it does not consider
    /// whether the side is visible, because that is a second, separately
    /// meaningful fact. [`Self::is_on_screen`] answers the conjunction.
    #[must_use]
    pub fn is_active(&self, panel: &PanelId) -> bool {
        self.find(panel)
            .is_some_and(|a| self.side(a.side).columns[a.column].stacks[a.stack].active == a.tab)
    }

    /// Whether `panel` is genuinely on screen: floating, or the active
    /// tab of its stack on a visible side.
    ///
    /// ★★ **The floating arm is first and it is unconditional.** A float
    /// has no side, so nothing about either dock can hide it — collapsing
    /// the side it came from does not, and neither does another panel
    /// being the front tab of the stack it used to be in. An
    /// implementation that consulted [`super::float::DockHome::side`]
    /// here would answer "not on screen" about a window the operator is
    /// looking at, and every toolbar toggle reading this would go dark
    /// while its panel stayed open.
    #[must_use]
    pub fn is_on_screen(&self, panel: &PanelId) -> bool {
        if self.is_floating(panel) {
            return true;
        }
        self.find(panel).is_some_and(|a| self.side(a.side).visible) && self.is_active(panel)
    }

    /// Make `panel` the active tab of its stack.
    ///
    /// Returns `false` if the panel is not mounted, which **must not be
    /// an error**: the caller's fallback is to mount it or to restore a
    /// default arrangement, not to refuse. The previous implementation
    /// made the same choice and stated the same reason, and it survives a
    /// change of engine because it is a statement about the caller's
    /// options rather than about the tree.
    ///
    /// Also makes the side visible, because "show me the Layers panel"
    /// meaning "select its tab inside a dock you cannot see" is a command
    /// that from the operator's side did nothing at all.
    pub fn activate(&mut self, panel: &PanelId) -> bool {
        // ★★ A floating panel is already the only thing in its window, so
        // there is no tab to select and no side to reveal — but the answer
        // is `true`, not `false`. `false` means *"not mounted, fall back
        // to mounting it"*, and falling back here would put a second copy
        // of a panel into the dock while its window was still open. What
        // "activate a float" additionally *should* do — raise the window
        // in front of whatever is covering it — is a viewport command, not
        // a layout edit, so it belongs to `super::floatwin`, which reads
        // `DockFrameReport::activated` to find out.
        if self.is_floating(panel) {
            return true;
        }
        let Some(a) = self.find(panel) else {
            return false;
        };
        let side = self.side_mut(a.side);
        side.visible = true;
        side.columns[a.column].stacks[a.stack].active = a.tab;
        true
    }

    /// Remove `panel` from the layout, pruning whatever it leaves empty.
    ///
    /// Returns `false` if it was not mounted. Closing the active tab
    /// selects the previous one rather than the next, which is what keeps
    /// a run of closes moving leftwards along the bar instead of
    /// marching through the tabs the operator has not touched.
    pub fn close(&mut self, panel: &PanelId) -> bool {
        // ★★★ **Closing a FLOATING panel is a close, not a dock-and-close.**
        //
        // The operator pressed a close control on a window; the panel goes
        // away and its float entry goes with it. A leaked entry would be
        // drawn as a window every frame for a panel that every "is it
        // open" query answers `false` about — so nothing would ever offer
        // to close it again.
        //
        // Why this is not "dock it back, then close it" — which would let
        // a later reopen find the old home — is argued at length in
        // [`super::float`]'s state-machine table: a close that silently
        // rearranges the dock behind the window it just shut is a side
        // effect nothing announced.
        if let Some(i) = self.floating.iter().position(|f| &f.panel == panel) {
            self.floating.remove(i);
            return true;
        }
        let Some(a) = self.find(panel) else {
            return false;
        };
        let side = self.side_mut(a.side);
        let stack = &mut side.columns[a.column].stacks[a.stack];
        stack.tabs.remove(a.tab);
        if stack.active >= a.tab && stack.active > 0 {
            stack.active -= 1;
        }
        self.normalize();
        true
    }

    /// Add `panel` as a new tab in the stack at `address`, or as a new
    /// stack if the column is empty, or as a new column if the side is.
    ///
    /// The permissive shape is deliberate: this is what an application
    /// calls when it wants a panel *somewhere sensible* after the
    /// operator's own arrangement has moved on. A version that refused an
    /// out-of-range address would push that fallback logic into every
    /// caller, where it would be written five times and differently.
    pub fn mount(
        &mut self,
        side: DockSide,
        column: usize,
        stack: usize,
        panel: impl Into<PanelId>,
    ) {
        let panel = panel.into();
        if self.contains(&panel) {
            return;
        }
        let s = self.side_mut(side);
        if s.columns.is_empty() {
            s.columns.push(Column::default());
        }
        let ci = column.min(s.columns.len() - 1);
        if s.columns[ci].stacks.is_empty() {
            s.columns[ci].stacks.push(Stack::default());
        }
        let si = stack.min(s.columns[ci].stacks.len() - 1);
        s.columns[ci].stacks[si].tabs.push(panel);
    }

    /// Every mounted panel whose id the catalog does not recognise.
    ///
    /// The application's equivalent of the previous implementation's
    /// `DockPanel::ALL` sweep — the question *"is anything mounted that
    /// nothing can draw?"* — asked from the side that actually knows the
    /// answer. The shell has no list of its own to sweep, because it has
    /// no list.
    #[must_use]
    pub fn unregistered_panels(&self, catalog: &dyn PanelCatalog) -> Vec<PanelId> {
        self.panels()
            .filter(|p| !catalog.contains(p.as_str()))
            .cloned()
            .collect()
    }

    /// Repair every structural invariant, in place.
    ///
    /// See the module header for the table of what is repaired and why.
    /// Returns nothing: a caller that wants to know *what* was repaired
    /// uses [`crate::layout`]'s loader, which performs the same repairs
    /// item by item and reports each one as a
    /// [`crate::layout::LayoutSkip`]. This method is the silent form, for
    /// the paths where there is no operator to tell — an application's
    /// own programmatic edit, or a close that emptied a column.
    pub fn normalize(&mut self) {
        let mut seen: std::collections::BTreeSet<PanelId> = std::collections::BTreeSet::new();
        for side in DockSide::ALL {
            let s = self.side_mut(side);
            if !s.width_pts.is_finite() || s.width_pts <= 0.0 {
                s.width_pts = SideLayout::default().width_pts;
            }
            for column in &mut s.columns {
                if !column.share.is_finite() || column.share <= MIN_SHARE {
                    column.share = 1.0;
                }
                for stack in &mut column.stacks {
                    if !stack.share.is_finite() || stack.share <= MIN_SHARE {
                        stack.share = 1.0;
                    }
                    stack.tabs.retain(|p| seen.insert(p.clone()));
                    if stack.tabs.is_empty() {
                        stack.active = 0;
                    } else {
                        stack.active = stack.active.min(stack.tabs.len() - 1);
                    }
                }
                column.stacks.retain(|s| !s.tabs.is_empty());
            }
            s.columns.retain(|c| !c.stacks.is_empty());
        }
        // ★ AFTER the sides, and the order is load-bearing: the duplicate
        // rule below drops a float whose panel is also docked, and "also
        // docked" has to be asked of the tree *after* the tree has had its
        // own duplicates removed. Asking first would let a panel mounted
        // twice keep a float that the surviving mount then collides with.
        super::float::normalize_floats(self);
    }

    /// Whether this layout satisfies every invariant [`Self::normalize`]
    /// repairs.
    ///
    /// Exists so a test can assert *"normalize is idempotent"* and so an
    /// application can assert its own built-in default is already clean
    /// rather than relying on a repair pass to make it so — the same
    /// posture `manifest`'s merge takes towards the built-in layer, and
    /// for the same reason: a defect in a compiled-in constant should
    /// fail a test, not be quietly patched on every machine that runs it.
    #[must_use]
    pub fn is_normalized(&self) -> bool {
        let mut probe = self.clone();
        probe.normalize();
        probe == *self
    }

    /// The width a side should be **drawn** at, given the window width.
    ///
    /// Applies the presentation clamp described on
    /// [`super::plan::MAX_SIDE_FRACTION`]. Deliberately a pure function
    /// taking `&self`: it cannot write the clamped value back even by
    /// accident, which is the property failure mode #6 turns on.
    #[must_use]
    pub fn drawn_side_width(&self, side: DockSide, window_width: f32) -> f32 {
        let window = plan::sane_length(window_width);
        let stored = plan::sane_length(self.side(side).width_pts);
        let ceiling = (window * plan::MAX_SIDE_FRACTION).max(plan::MIN_SIDE_WIDTH);
        stored.clamp(plan::MIN_SIDE_WIDTH, ceiling)
    }
}

/// Whether an id names a panel the application can actually draw.
///
/// The exact shape of [`crate::manifest::CommandCatalog`], and for the
/// same reason: it lets a layout be loaded, validated and diffed by a
/// tool that has no application at all — a schema linter, a diff viewer,
/// a harness inspecting a saved workspace without linking the binary.
pub trait PanelCatalog {
    /// Whether this id names a panel that can be drawn.
    fn contains(&self, id: &str) -> bool;
}

/// A catalog that accepts every id.
///
/// For tests and for tooling that has no registry. Using it in
/// production would disable the check that turns a stale panel id into a
/// disclosed skip instead of an empty compartment, which is why it is a
/// named type at a call site rather than a default.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyPanel;

impl PanelCatalog for AnyPanel {
    fn contains(&self, _id: &str) -> bool {
        true
    }
}

/// What the application knows about one dockable panel.
///
/// The shell needs three strings to draw a tab: a label, a tooltip, and
/// the id it already has. It needs nothing else, and asking for nothing
/// else is what keeps [`PanelId`] opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelInfo {
    /// The id this entry describes.
    pub id: PanelId,
    /// The tab's label.
    pub label: String,
    /// What the panel is **for**, and *when to reach for it* — never a
    /// restatement of the label.
    ///
    /// This is also the tab's accessible name, which is why it carries
    /// the burden: a screen-reader user hearing "Pages" learns nothing a
    /// sighted user does not already see, whereas "Pages — jump to a
    /// page, reorder or rotate sheets" is the information. The previous
    /// implementation asserted the distinction with a test that a tooltip
    /// must be meaningfully longer than its label; that test is carried
    /// across as [`PanelRegistry::thin_tooltips`], which the application
    /// calls in *its* suite because it owns the strings.
    pub tooltip: String,
}

impl PanelInfo {
    /// Describe a panel.
    #[must_use]
    pub fn new(id: impl Into<PanelId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tooltip: String::new(),
        }
    }

    /// Add the purpose tooltip.
    #[must_use]
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    /// The tab's accessible name: the tooltip when there is one, the
    /// label otherwise.
    #[must_use]
    pub fn accessible_name(&self) -> &str {
        if self.tooltip.is_empty() {
            &self.label
        } else {
            &self.tooltip
        }
    }
}

/// Everything the application can dock.
///
/// Populated at runtime, exactly like [`crate::commands::CommandRegistry`]
/// — and for the reason `SHELL_FRAMEWORK.md` §5b gives: *a capability's
/// presence is expressed by registering it, and by nothing else.* A panel
/// belonging to a feature that was compiled out is simply not registered,
/// its saved mount is dropped with a disclosed reason, and no `#[cfg]`
/// appears anywhere in this crate.
#[derive(Debug, Clone, Default)]
pub struct PanelRegistry {
    by_id: std::collections::BTreeMap<String, PanelInfo>,
}

impl PanelRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one panel, replacing any earlier entry with the same id.
    ///
    /// Replacement rather than rejection, unlike the command registry: a
    /// command's handler is behaviour and two of them is a genuine
    /// conflict, whereas a panel entry is three strings and the last
    /// caller wins harmlessly. An application that wants strictness can
    /// check [`Self::get`] first.
    pub fn register(&mut self, panel: PanelInfo) {
        self.by_id.insert(panel.id.as_str().to_owned(), panel);
    }

    /// Register several panels.
    pub fn register_all(&mut self, panels: impl IntoIterator<Item = PanelInfo>) {
        for p in panels {
            self.register(p);
        }
    }

    /// Look one up.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&PanelInfo> {
        self.by_id.get(id)
    }

    /// Every registered panel, in id order.
    pub fn iter(&self) -> impl Iterator<Item = &PanelInfo> {
        self.by_id.values()
    }

    /// How many panels are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Every registered panel whose tooltip does not add information
    /// beyond its label.
    ///
    /// A helper the *application* asserts on, because the application
    /// owns the strings. Carried across from the previous
    /// implementation's `every_panel_tooltip_adds_information_beyond_its_label`,
    /// which encoded a real rule: a tooltip states **when to reach for**
    /// a surface, and a tooltip that restates the label has spent a
    /// disclosure opportunity on nothing.
    ///
    /// The threshold — twenty characters beyond the label — is the same
    /// one that test used. It is a heuristic and it is deliberately
    /// generous; its job is to catch `tooltip: "Pages"`, not to grade
    /// prose.
    #[must_use]
    pub fn thin_tooltips(&self) -> Vec<&PanelInfo> {
        self.iter()
            .filter(|p| {
                p.tooltip.is_empty()
                    || p.tooltip == p.label
                    || p.tooltip.chars().count() <= p.label.chars().count() + 20
            })
            .collect()
    }
}

impl PanelCatalog for PanelRegistry {
    fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A three-column, four-panel arrangement used by several tests.
    fn sample() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::new("pages"), Stack::tabbed(["layers", "bookmarks"])]),
                Column::new([Stack::new("tools")]),
            ]),
            SideLayout::single("objects"),
        )
    }

    #[test]
    fn a_panel_id_is_an_opaque_string_the_shell_does_not_interpret() {
        let id = PanelId::new("anything.at.all");
        assert_eq!(id.as_str(), "anything.at.all");
        assert_eq!(id.to_string(), "anything.at.all");
        assert_eq!(PanelId::from("x"), PanelId::from("x".to_owned()));
    }

    #[test]
    fn every_mounted_panel_is_findable_and_the_address_round_trips() {
        let layout = sample();
        for panel in ["pages", "layers", "bookmarks", "tools", "objects"] {
            let id = PanelId::new(panel);
            let a = layout
                .find(&id)
                .unwrap_or_else(|| panic!("{panel} unmounted"));
            let stack = &layout.side(a.side).columns[a.column].stacks[a.stack];
            assert_eq!(stack.tabs[a.tab], id);
        }
        assert!(layout.find(&PanelId::new("nope")).is_none());
    }

    /// A backgrounded tab reports `false`, and activation raises it.
    ///
    /// Carried across from the previous implementation, which kept the
    /// equivalent test deliberately after its default layout stopped
    /// exercising it: *"without this the function would be effectively
    /// untested and a future edit could break it with every test still
    /// green."*
    #[test]
    fn a_backgrounded_panel_can_be_brought_forward() {
        let mut layout = sample();
        let bookmarks = PanelId::new("bookmarks");
        assert!(!layout.is_active(&bookmarks), "starts behind `layers`");
        assert!(layout.activate(&bookmarks));
        assert!(layout.is_active(&bookmarks));
        assert!(!layout.is_active(&PanelId::new("layers")));
    }

    /// Activating an unmounted panel reports failure instead of
    /// panicking — the caller's fallback is to mount it, not to refuse.
    #[test]
    fn activating_an_unmounted_panel_reports_failure_instead_of_panicking() {
        let mut layout = DockLayout::empty();
        assert!(!layout.activate(&PanelId::new("ghost")));
        assert!(!layout.is_active(&PanelId::new("ghost")));
    }

    /// Activation also reveals a hidden side, because selecting a tab
    /// inside an invisible dock is a command that did nothing.
    #[test]
    fn activation_reveals_a_hidden_side() {
        let mut layout = sample();
        layout.right.visible = false;
        let objects = PanelId::new("objects");
        assert!(layout.is_active(&objects), "active within its own stack");
        assert!(!layout.is_on_screen(&objects), "but the side is hidden");
        assert!(layout.activate(&objects));
        assert!(layout.is_on_screen(&objects));
    }

    /// Closing the last tab of a stack prunes the stack; closing the last
    /// stack of a column prunes the column.
    #[test]
    fn closing_a_panel_prunes_whatever_it_leaves_empty() {
        let mut layout = sample();
        assert!(layout.close(&PanelId::new("tools")));
        assert_eq!(layout.left.columns.len(), 1, "the empty column went too");
        assert!(!layout.contains(&PanelId::new("tools")));
        assert!(!layout.close(&PanelId::new("tools")), "already gone");
    }

    /// Closing the active tab selects the tab to its **left**, so a run
    /// of closes walks back along the bar rather than marching through
    /// tabs the operator has not touched.
    #[test]
    fn closing_the_active_tab_selects_the_one_to_its_left() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(["a", "b", "c"])])]),
            SideLayout::none(),
        );
        assert!(layout.activate(&PanelId::new("c")));
        assert!(layout.close(&PanelId::new("c")));
        assert!(layout.is_active(&PanelId::new("b")));
    }

    /// ★ **A panel cannot be mounted twice.**
    ///
    /// Two live copies of one surface each have their own scroll position
    /// and their own idea of which tab is active, and `activate` raises
    /// whichever it finds first. The previous implementation forbade this
    /// with a test; here the model repairs it, because the input that
    /// causes it is a hand-edited file that no test of the defaults can
    /// reach.
    #[test]
    fn a_panel_mounted_twice_keeps_only_its_first_mount() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::new("pages"), Stack::new("pages")])]),
            SideLayout::single("pages"),
        );
        layout.normalize();
        assert_eq!(layout.panels().count(), 1);
        assert_eq!(layout.left.columns[0].stacks.len(), 1);
        assert!(layout.right.is_empty(), "the duplicate on the right went");
    }

    /// `mount` refuses to create a second copy, for the same reason.
    #[test]
    fn mounting_an_already_mounted_panel_is_a_no_op() {
        let mut layout = sample();
        let before = layout.clone();
        layout.mount(DockSide::Right, 0, 0, "pages");
        assert_eq!(layout, before);
    }

    /// `mount` into an empty side creates the column and stack it needs
    /// rather than refusing an out-of-range address.
    #[test]
    fn mounting_into_an_empty_side_creates_the_container_it_needs() {
        let mut layout = DockLayout::empty();
        layout.mount(DockSide::Right, 7, 9, "objects");
        assert!(layout.contains(&PanelId::new("objects")));
        assert_eq!(layout.right.columns.len(), 1);
    }

    /// Normalization is idempotent, and a clean layout is unchanged by
    /// it.
    #[test]
    fn normalization_is_idempotent_and_leaves_a_clean_layout_alone() {
        let layout = sample();
        assert!(layout.is_normalized(), "the sample is already clean");
        let mut once = layout.clone();
        once.normalize();
        let mut twice = once.clone();
        twice.normalize();
        assert_eq!(once, twice);
    }

    /// Every degenerate numeric field is repaired to something drawable.
    #[test]
    fn degenerate_shares_and_widths_are_repaired() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::new("a").with_share(f32::NAN)])]).with_width(-4.0),
            SideLayout::none(),
        );
        layout.left.columns[0].share = 0.0;
        layout.normalize();
        assert!(layout.left.width_pts > 0.0);
        assert!(layout.left.columns[0].share > 0.0);
        assert!(layout.left.columns[0].stacks[0].share.is_finite());
    }

    /// An `active` index past the end selects the last tab rather than
    /// failing.
    #[test]
    fn an_out_of_range_active_index_is_clamped() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(["a", "b"])])]),
            SideLayout::none(),
        );
        layout.left.columns[0].stacks[0].active = 47;
        layout.normalize();
        assert_eq!(layout.left.columns[0].stacks[0].active, 1);
        assert!(layout.is_active(&PanelId::new("b")));
    }

    /// The drawn width is clamped to the window without the model
    /// learning about it — the un-maximise survival property, at the one
    /// place a *stored* number meets a *window* number.
    #[test]
    fn the_drawn_side_width_is_clamped_without_the_model_changing() {
        let layout = DockLayout::new(
            SideLayout::single("a").with_width(900.0),
            SideLayout::none(),
        );
        let narrow = layout.drawn_side_width(DockSide::Left, 1000.0);
        assert!(narrow < 900.0, "clamped on a small window: {narrow}");
        assert_eq!(layout.left.width_pts, 900.0, "the model is untouched");
        let wide = layout.drawn_side_width(DockSide::Left, 4000.0);
        assert!((wide - 900.0).abs() < 0.01, "restored in full: {wide}");
    }

    /// A width below the floor is raised, so a saved layout from a build
    /// with a smaller minimum still yields a grabbable dock.
    #[test]
    fn a_stored_width_below_the_floor_is_raised_at_draw_time() {
        let layout = DockLayout::new(SideLayout::single("a").with_width(10.0), SideLayout::none());
        assert!(layout.drawn_side_width(DockSide::Left, 1280.0) >= plan::MIN_SIDE_WIDTH);
    }

    /// The registry answers the catalog question, and an unregistered
    /// mount is reportable.
    #[test]
    fn the_registry_is_the_only_authority_on_what_can_be_drawn() {
        let mut registry = PanelRegistry::new();
        registry.register(PanelInfo::new("pages", "Pages"));
        let layout = sample();
        let missing = layout.unregistered_panels(&registry);
        assert!(missing.contains(&PanelId::new("objects")));
        assert!(!missing.contains(&PanelId::new("pages")));
        assert!(layout.unregistered_panels(&AnyPanel).is_empty());
    }

    /// A tooltip that restates its label is reported as thin, and a real
    /// one is not.
    #[test]
    fn a_tooltip_that_restates_its_label_is_reported_as_thin() {
        let mut registry = PanelRegistry::new();
        registry.register(PanelInfo::new("a", "Pages").with_tooltip("Pages"));
        registry.register(
            PanelInfo::new("b", "Layers")
                .with_tooltip("Layers — show, hide and reorder the optional content groups"),
        );
        let thin = registry.thin_tooltips();
        assert_eq!(thin.len(), 1);
        assert_eq!(thin[0].id, PanelId::new("a"));
    }

    /// An accessible name falls back to the label rather than being
    /// empty, because an unnamed tab is the worst accessibility outcome:
    /// reachable by keyboard and announcing nothing.
    #[test]
    fn an_accessible_name_falls_back_to_the_label() {
        assert_eq!(PanelInfo::new("a", "Pages").accessible_name(), "Pages");
    }

    /// An empty side draws nothing at all, rather than an empty bordered
    /// stripe nobody can remove.
    #[test]
    fn a_side_with_no_columns_is_empty() {
        assert!(SideLayout::none().is_empty());
        assert!(SideLayout::new([Column::new([])]).is_empty());
        assert!(!SideLayout::single("a").is_empty());
    }
}
