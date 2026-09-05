//! [`DockFrameReport`] — what one frame of the dock drew, and what the
//! operator did to it.
//!
//! Split out of `dock/mod.rs` on 2026-09-05, when that file crossed R2's
//! 1,500-line limit. The seam is the one [`crate::ribbon::frame_report`]
//! already uses one surface over: **a report is a vocabulary, not a
//! mechanism.** Nothing in this file draws, lays out or decides anything; it
//! is the set of nouns `Dock::show` fills in and a harness reads back, and each
//! one carries the argument for why it is published separately from its
//! neighbours rather than inferred from them.
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
    /// state should read this rather than keeping a boolean of its own;
    /// the previous implementation records a `properties_open` flag that
    /// *"could disagree with what was on screen"*, which is the defect
    /// the field exists to make unnecessary.
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
    /// ⇒ That makes forgetting it a *silent* failure of exactly the class
    /// this project has already shipped: three panels that were laid out,
    /// published a rectangle, and could not be reached, with every gate
    /// green. `crate::dock::report`'s header records the response —
    /// *a rect proves layout, not visibility* — and this field is the same
    /// response for a surface that has no rect at all because its window
    /// was never opened.
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
    /// An application persists on this rather than on a timer: writing a
    /// layout file on every frame is what makes the benchmarked
    /// application's own layout file *"rewritten on every exit"* and its
    /// community's workaround — copying the file aside and back — as
    /// awkward as `MODES_AND_PANELS.md` records it being.
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
