//! [`DockFrameReport`] — what one frame of the dock drew, and what the
//! operator did to it.
//!
//! The seam is the one [`crate::ribbon::frame_report`] uses one surface
//! over: **a report is a vocabulary, not a mechanism.** Nothing in this file
//! draws, lays out or decides anything; it is the set of nouns `Dock::show`
//! fills in and a harness reads back, and each one carries the argument for
//! why it is published separately from its neighbours rather than inferred
//! from them.
//!
//! ★ Two fields on it are worth reading together, because they are the pair a
//! reader will otherwise try to derive one from the other:
//! [`DockFrameReport::rail_show`] says what the rail DID about auto-hide, and
//! [`DockFrameReport::tab_strips_suppressed`] counts the stacks that drew no
//! tab bar. Both are about the same operator decision — *"the rail is my panel
//! switch"* — and neither implies the other: a rail can be inline beside a
//! stack that kept its tabs, and a hidden rail beside a stack that gave them
//! up.

use super::{DockSide, PanelId};

/// What one frame of the dock drew and what the operator did to it.
///
/// Returned by [`Dock::show`] and also kept on [`DockState`], because two
/// different callers want it: the frame's own caller, and a diagnostic
/// surface that runs later in the same frame and has no access to the
/// return value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockFrameReport {
    /// Every panel whose body was drawn — the active tab of every stack
    /// on every visible side.
    ///
    /// **This is the honest answer to "what is on screen".** A panel
    /// behind another tab is not in this list, and neither is one on a
    /// hidden side. An application deriving a toolbar toggle's selected
    /// state reads this rather than keeping a boolean of its own: a
    /// separate `panel_open` flag is a second copy of a fact the dock
    /// already owns, and the two disagree the first time a panel is closed
    /// by any route the flag's owner did not write.
    pub panels_drawn: Vec<PanelId>,
    /// How many tabs were moved into an overflow menu, across every
    /// stack.
    pub panels_overflowed: usize,
    /// How many overflow affordances were drawn.
    pub overflow_menus: usize,
    /// Which sides drew anything.
    pub sides_drawn: Vec<DockSide>,
    /// The panel whose tab the operator selected this frame, if any.
    pub activated: Option<PanelId>,
    /// The panel the operator closed this frame, if any.
    pub closed: Option<PanelId>,
    /// **The panel whose tab the operator dragged to a new position this
    /// frame**, if the move changed the order.
    ///
    /// `None` for a drag released where it started — which is a real outcome
    /// and not a failure, and the reason this is published separately from
    /// [`Self::tab_drag`]: one says a gesture is in flight, the other says it
    /// changed something.
    pub reordered: Option<PanelId>,
    /// **A tab drag in flight, and the boundary a release would use.**
    ///
    /// Set for every frame between the press passing `egui`'s drag threshold
    /// and the release, and `None` on the frame the release lands. The caret
    /// drawn on the strip is the same fact in paint; this is the checkable
    /// half, because a hairline between two near-identical labels is precise
    /// and cannot be asserted on.
    pub tab_drag: Option<super::drag::TabDragPreview>,
    /// **The panel the operator dragged into a different compartment this
    /// frame**, if the release moved it.
    ///
    /// Distinct from [`Self::reordered`], which is a permutation inside one
    /// stack. The two are the same gesture and different outcomes, and an
    /// application that told the operator "moved" for a reorder would be
    /// reporting a structural change that did not happen.
    pub moved: Option<PanelId>,
    /// **A drag held over a compartment, and where a release would put it.**
    ///
    /// Set for every frame the drop compass is painted and `None` on the frame
    /// the release lands — [`Self::moved`] is the release. The checkable half
    /// of an affordance whose visible form is a wash of colour, for
    /// [`Self::tab_drag`]'s reason.
    pub drop_preview: Option<super::overlay::DropPreview>,
    /// **A drag held clear of the dock, and the window a release would open.**
    ///
    /// Set for every frame the tear outline is painted and `None` on the frame
    /// the release lands — [`Self::floated`] is the release, whichever route
    /// raised it. [`Self::tab_drag`]'s reason for existing separately from the
    /// paint applies here unchanged.
    pub tear: Option<super::tear::TearPreview>,
    /// The panel the operator floated this frame, if any.
    pub floated: Option<PanelId>,
    /// The panel the operator docked back this frame, if any.
    pub docked: Option<PanelId>,
    /// Every panel the layout says is floating — whether or not a window
    /// was drawn for it.
    ///
    /// The *claim*. [`FloatFrameReport::drawn`] is the *fact*, and
    /// [`Self::floats_undrawn`] is the difference.
    pub floating: Vec<PanelId>,
    /// ★★★ **How many floating panels nothing drew last frame.**
    ///
    /// Zero in a correct application. Non-zero means panels are in the
    /// layout, are reported as on screen, and **are not on screen** —
    /// because [`Dock::show_floating`] was never called.
    ///
    /// # Why this field exists at all
    ///
    /// Floating is the one capability in this dock that needs **two**
    /// calls per frame instead of one: [`Dock::show`] for the docked
    /// panels, and [`Dock::show_floating`] for the windows. The second is
    /// separate because a child viewport must be opened from the
    /// application's top-level frame rather than from inside a side
    /// panel's layout closure.
    ///
    /// ⇒ That makes forgetting the second call a *silent* failure: panels
    /// that are laid out, publish a rectangle, and cannot be reached, with
    /// every gate green. `crate::dock::report`'s header states the rule —
    /// *a rect proves layout, not visibility* — and this field is that rule
    /// applied to a surface with no rect at all, because its window was
    /// never opened.
    ///
    /// An application asserts this is zero in its own frame test. It is
    /// measured against the **previous** frame's float report, because
    /// `show` runs before `show_floating`; so a genuine first frame
    /// reports the floats it is about to draw, and the value settles on
    /// the next one. A test therefore drives two frames — which is what a
    /// harness does anyway.
    pub floats_undrawn: usize,
    /// Whether the layout changed this frame and is therefore worth
    /// saving.
    ///
    /// An application persists on this rather than on a timer or on every
    /// frame. A layout file rewritten unconditionally can never be pinned:
    /// the operator who wants one arrangement kept is reduced to copying
    /// the file aside and back, which is the workaround
    /// `MODES_AND_PANELS.md` records the benchmarked application's users
    /// resorting to.
    pub layout_changed: bool,
    /// **What the rail did about auto-hide this frame** —
    /// [`crate::peek::Show`].
    ///
    /// [`crate::peek::Show::Inline`] is the ordinary rail, holding
    /// [`rail::WIDTH_PTS`] off the side. [`crate::peek::Show::Overlay`] means
    /// the operator has the rail's auto-hide on and the pointer has reached the
    /// sliver, so the strip is painted *over* the panel beside it — the panel's
    /// own width is identical in both hidden and revealed states, which is what
    /// stops the panel body reflowing under the pointer.
    /// [`crate::peek::Show::Hidden`] is that setting at rest: the sliver alone.
    ///
    /// ★ [`crate::peek::Show::Inline`] on a side with no rail at all, because
    /// "there is no rail" and "the rail is inline" are the same picture and the
    /// same layout. A check that wants the first asks whether the manifest has
    /// a rail; this field is about the setting.
    pub rail_show: crate::peek::Show,
    /// **How many stacks drew no tab strip because the rail switches between
    /// their panels.** See [`Dock::with_rail_reach`].
    ///
    /// ★★ Counted rather than inferred from the absence of `dock.tab.*`
    /// regions, because absence has three causes that a harness must not
    /// confuse: a suppressed strip, a stack of one panel (whose strip is drawn
    /// and holds a single tab), and a side that was not drawn at all. Only the
    /// first is this feature working.
    pub tab_strips_suppressed: usize,
}
