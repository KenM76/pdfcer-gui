//! [`FrameReport`] — what one rendered ribbon frame contained.
//!
//! # Why the shell publishes this rather than only drawing
//!
//! Two audiences, and neither is served by a renderer that returns nothing
//! but tokens.
//!
//! - **This crate's own tests.** The caption invariant is *"every rendered
//!   group emitted a caption"*, and asserting it needs the two counts.
//!   Asserting it through the rect sink would work, but it would test the
//!   reporting channel as well as the invariant, and a test that fails for
//!   two reasons tells you neither.
//! - **A verification harness.** `ui-verify` drives a live application and
//!   needs to know whether a group is in the band or in the overflow menu
//!   — and, since 2026-08-13, whether a *tab* is in the strip or in the
//!   strip's own menu — before it can assert anything about it.
//!
//! Every field describes the frame that has just been drawn, not the state
//! the next one will be drawn from; that is in [`super::RibbonState`].
//!
//! # ★ The distinction this file exists to keep: *drawn* is not *planned*
//!
//! There are two ways to count "how many things are in the menu", they are
//! not the same number, and mixing them cost a defect that hid behind an
//! arithmetic underflow.
//!
//! - **What the plan decided** — [`FrameReport::groups_overflowed`],
//!   [`FrameReport::tabs_overflowed`]. True every frame, whether or not
//!   the menu is open.
//! - **What was drawn** — [`FrameReport::groups_rendered`] minus
//!   [`FrameReport::groups_in_band`]. Zero on every frame the menu is
//!   shut, because a closed popup draws nothing.
//!
//! Code that treats the plan's number as a count of what was drawn
//! subtracts one from the other and underflows the moment text has a
//! width. That is not hypothetical; it is where
//! [`FrameReport::groups_in_band`] came from. Both counts are therefore
//! published, separately, with the difference written down here.

/// What one rendered frame contained. See this module's header.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrameReport {
    /// The tab whose band was drawn.
    pub active_tab: Option<String>,
    /// The mode whose tab list was used.
    pub mode: Option<String>,
    /// How many tabs are visible this frame, contextual ones included.
    ///
    /// The total: strip plus overflow menu. Always equal to
    /// [`Self::tabs_in_strip`] + [`Self::tabs_overflowed`], which
    /// `no_tab_is_lost_between_the_strip_and_its_menu` asserts — a tab
    /// that is in neither is a tab the operator cannot reach at all, and
    /// that is `MODES_AND_PANELS.md` failure mode #8 stated as a count.
    pub tabs_visible: usize,
    /// How many tabs were drawn in the strip itself.
    pub tabs_in_strip: usize,
    /// How many tabs the plan moved into the strip's overflow menu.
    ///
    /// A *plan* number, not a drawn one — see this module's header. The
    /// **active** tab is never among them: it is pinned into the strip by
    /// [`super::plan::plan_tab_strip`], because a strip that hides the tab
    /// you are looking at is worse than one that hides the others.
    pub tabs_overflowed: usize,
    /// Whether the tab strip's overflow affordance was drawn.
    pub tab_overflow_visible: bool,
    /// Whether the strip **collapsed**: too narrow to hold a tab and an
    /// affordance at once, so it shows the affordance alone and the menu
    /// holds every tab including the active one.
    ///
    /// The one state in which [`Self::tabs_in_strip`] is zero while tabs
    /// exist, and the one state in which the pin described on
    /// [`Self::tabs_overflowed`] does not hold. Published rather than
    /// inferred, because "no tabs are in the strip" and "the strip gave up
    /// the pin to keep every tab reachable" are the same observation and
    /// very different facts. See
    /// [`super::plan::plan_tab_strip`]'s collapse section for why the
    /// alternative is worse.
    pub tab_strip_collapsed: bool,
    /// The `egui::Id` of the tab strip's overflow affordance, when one was
    /// drawn.
    ///
    /// Published so a harness — or this crate's own tests — can ask `egui`
    /// itself whether the control is hit-testable. A rectangle proves a
    /// thing was allocated; only a hit test proves it can be reached.
    pub tab_overflow_id: Option<egui::Id>,
    /// How many groups were drawn — in the band and in the band's overflow
    /// menu together.
    pub groups_rendered: usize,
    /// How many of those were drawn in the band itself.
    ///
    /// Always ≤ [`Self::groups_rendered`], **by construction rather than
    /// by arithmetic**: it is the value that counter had reached when the
    /// band's own loop finished. So `groups_rendered − groups_in_band` is
    /// "how many were drawn in the open menu" and can never underflow.
    ///
    /// See this module's header for why that is deliberately not the same
    /// question as [`Self::groups_overflowed`].
    pub groups_in_band: usize,
    /// How many captions were emitted. **Always equal to
    /// [`Self::groups_rendered`]**; see [`super::band`].
    pub captions_emitted: usize,
    /// How many groups the plan moved into the band's overflow menu.
    pub groups_overflowed: usize,
    /// Whether the band's overflow affordance was drawn.
    pub overflow_visible: bool,
    /// The `egui::Id` of the band's overflow affordance, when one was
    /// drawn.
    pub overflow_id: Option<egui::Id>,
    /// How many commands the operator invoked.
    pub commands_invoked: usize,
    /// **What the band did about auto-hide this frame** —
    /// [`crate::peek::Show`].
    ///
    /// [`crate::peek::Show::Inline`] is the ordinary ribbon: the band is part
    /// of the layout and the application's top panel is as tall as it.
    /// [`crate::peek::Show::Overlay`] means the operator has auto-hide on and
    /// the pointer is over the tab strip, so the band is painted *over* the
    /// document and the top panel is only the strip.
    /// [`crate::peek::Show::Hidden`] is the resting state of that setting.
    ///
    /// ★★ Published because the three are **indistinguishable from the other
    /// counts on this report**. `groups_rendered` is the same for an inline
    /// band and an overlaid one, and zero for a hidden band and for a tab
    /// whose every group went into the overflow menu. A driven check that
    /// asserted "the band drew nothing" could not tell auto-hide working from
    /// a ribbon that had lost its groups —
    /// `D:/dev/rag/egui/a_driven_checks_oracle_must_be_able_to_distinguish_the_defect_from_the_fix.md`
    /// is that exact failure, on this exact surface.
    pub band_show: crate::peek::Show,
}
