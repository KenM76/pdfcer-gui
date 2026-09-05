//! # `peek` — auto-hide, for any permanent chrome surface
//!
//! One three-state machine, shared by the ribbon's band and by the dock's left
//! rail, because the operator asked for the same behaviour on both in one
//! sentence (2026-09-05):
//!
//! > *"we should also add the capability to auto hide the ribbon until we hover
//! > over top of it… left rail should also have the option to auto hide as
//! > well."*
//!
//! ## ★★★ THE CONVENTION, AND WHERE IT COMES FROM
//!
//! This is a solved interaction in the product class and the model here is
//! taken from it rather than invented. **Microsoft Office's *Show Tabs*** —
//! the middle setting of its three-way *Ribbon Display Options*, beside *Show
//! Tabs and Commands* (never hide) and *Auto-hide Ribbon* (hide everything).
//! In *Show Tabs* the tab strip stays on screen permanently and only the
//! **band** goes; touching the strip brings the band back **over** the
//! document; it goes again when you leave it. VS Code's activity bar and
//! Acrobat's collapsing tool panes are the same shape one surface over: a
//! permanent, narrow trigger that never disappears, and a wide body that
//! overlays rather than displaces.
//!
//! Three properties are carried across deliberately, and each answers a way
//! this feature is usually got wrong:
//!
//! | Property | The failure it prevents |
//! |---|---|
//! | The **trigger is permanent** — a tab strip, an 8 pt sliver — and is never itself hidden | Office's *Auto-hide Ribbon* is the setting people get stuck in, because the thing you must touch to get out of it is invisible. A mode you cannot leave is a trap, and the operator was explicit that it must be *"the option to auto hide"* — an option has a way back. |
//! | The body **overlays**, it does not displace | A canvas that resizes on hover is nauseating, and every coordinate under the pointer moves as the pointer approaches. The whole point of hiding chrome is to give the document the room; giving it the room and then taking it back at 60 Hz is worse than not hiding at all. |
//! | Reveal can only be **started** by the trigger | See R128 below. |
//!
//! ## ⚠⚠ R128 — the feedback loop this shape invites, and the bound that closes it
//!
//! `D:/dev/rag/egui/a_surface_may_not_change_size_in_response_to_a_gesture_aimed_at_it.md`
//! records the incident: a caption drawn above a drop target moved the target
//! under the pointer, the size depended on the wording, the wording depended on
//! the pointer, and it presented as a hit-testing bug. The rule it produced —
//! **a surface may not change size in response to a gesture aimed at it** — is
//! exactly what an auto-hiding ribbon is in danger of violating: a surface whose
//! visibility depends on the pointer, whose position depends on the layout,
//! which depends on the surface.
//!
//! The RAG entry is also explicit that the fix is **a direction bound, not a
//! guard** (*"R128 needs a DIRECTION bound not a guard"*), because a
//! "don't ask twice" latch merely halves the oscillation's frequency.
//!
//! This module's bound, stated as an invariant rather than as prose:
//!
//! ```text
//! revealed(n+1)  =  in(trigger)  ∨  ( revealed(n) ∧ in(overlay(n)) )
//! ```
//!
//! Read the two terms:
//!
//! * **`in(trigger)` is the only way in.** The trigger rectangle is supplied by
//!   the caller and is required to be *independent of the overlay* — the
//!   ribbon's is its tab strip, whose height comes from
//!   [`crate::theme::Metrics`] and the mode selector, not from the band; the
//!   rail's is a sliver of [`Peek::MIN_TRIGGER_PTS`] that is reserved whether
//!   the rail is revealed or not. Nothing the overlay does can move it, so
//!   revealing cannot cause revealing.
//! * **`in(overlay)` can only keep it, never start it.** It is conjoined with
//!   `revealed(n)`, so on a frame where the surface is hidden the overlay term
//!   is false whatever the pointer is doing. The state is therefore *monotone
//!   decreasing* while the pointer is still: once it leaves the trigger it can
//!   only fall to hidden and stay there. There is no cycle to oscillate around.
//!
//! And the **floor**: [`Peek::resolve`] refuses a trigger thinner than
//! [`Peek::MIN_TRIGGER_PTS`] in either axis by reporting the surface *shown*
//! rather than hidden. A trigger too small to hit is how a surface becomes
//! unreachable, and this project has shipped three unreachable panels with
//! every gate green (`SHELL_LAYOUT_PROPOSAL.md` §5). Failing **open** is the
//! only safe direction: the worst case is chrome the operator wanted hidden,
//! which is visible and one setting away, rather than chrome they cannot get
//! back.
//!
//! ## ★★ The keyboard is not a pointer, and it holds the surface open
//!
//! [`Peek::resolve`] is also handed whether anything inside the overlay has
//! keyboard focus. Without that clause a keyboard user tabbing into a revealed
//! band loses it on the next frame — the pointer is nowhere near it — which is
//! the same class of defect as a control that is drawn and unclickable. It is a
//! **keep** term, conjoined with `revealed(n)` exactly like the overlay term,
//! so it cannot start a reveal either.
//!
//! ## What this module is not
//!
//! It holds no `egui` widgets and paints nothing. It is arithmetic over
//! rectangles and a pointer position, so it is swept across widths, pointer
//! paths and frame sequences in unit tests with no context at all — which is
//! how the invariant above is asserted rather than asserted-about.

use egui::{Pos2, Rect};

/// Whether a surface that *can* auto-hide currently is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AutoHide {
    /// The surface is part of the layout, always. The default, and the
    /// setting an operator who has never heard of this feature is in.
    #[default]
    Off,
    /// The surface is hidden until the pointer reaches its trigger, and is
    /// then drawn as an **overlay** so nothing beneath it moves.
    OnHover,
}

impl AutoHide {
    /// Serde's `skip_serializing_if` predicate for the default.
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::Off
    }

    /// `true` when the surface hides itself.
    #[must_use]
    pub fn is_on(self) -> bool {
        self == Self::OnHover
    }

    /// The other setting, for a toggle command.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Off => Self::OnHover,
            Self::OnHover => Self::Off,
        }
    }
}

/// What a surface should do this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Show {
    /// Draw it in the layout, taking its own room. [`AutoHide::Off`], or the
    /// floor in [`Peek::resolve`] refusing an unhittable trigger.
    ///
    /// The default, matching [`AutoHide::Off`]: a surface nobody has
    /// configured is a surface that is simply there.
    #[default]
    Inline,
    /// Draw it as an overlay, anchored to the trigger, taking no room.
    Overlay,
    /// Draw nothing but the trigger.
    Hidden,
}

impl Show {
    /// Whether the surface's body is drawn at all this frame.
    #[must_use]
    pub fn draws_body(self) -> bool {
        !matches!(self, Self::Hidden)
    }

    /// Whether the body takes room from the layout.
    #[must_use]
    pub fn takes_room(self) -> bool {
        matches!(self, Self::Inline)
    }
}

/// The auto-hide state of one surface, across frames.
///
/// Cheap and `Copy`-free only because it holds a `Rect`; an application keeps
/// one per surface beside that surface's other presentation state (the ribbon's
/// in [`crate::ribbon::RibbonState`], the rail's in
/// [`crate::dock::DockState`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Peek {
    mode: AutoHide,
    /// Last frame's answer. The `revealed(n)` term of the invariant.
    revealed: bool,
    /// Where the overlay was drawn last frame, in screen points. `None` on any
    /// frame it was not drawn — which is what makes the overlay term false
    /// whenever the surface is hidden, independently of the conjunction.
    overlay: Option<Rect>,
}

impl Peek {
    /// **The smallest trigger this module will accept**, in points, in either
    /// axis.
    ///
    /// Eight, which is the width Windows gives a window's resize border and
    /// about what VS Code gives its collapsed sidebar edge — a strip a pointer
    /// finds without being aimed. Below it [`Self::resolve`] reports
    /// [`Show::Inline`] and the surface simply does not hide: see the module
    /// header on why the floor fails **open**.
    pub const MIN_TRIGGER_PTS: f32 = 8.0;

    /// **How far outside the trigger or the overlay the pointer may stray
    /// before the surface closes**, in points.
    ///
    /// Four. Without a grace margin the surface closes on the one-pixel seam
    /// between the trigger and the overlay it anchors — a gap that exists
    /// because two rectangles that share an edge do not both contain the points
    /// on it — and the symptom is a band that flickers as the pointer crosses
    /// from the tab strip into the group beneath it. It is added to a
    /// **containment test**, never to a laid-out rectangle, so it changes no
    /// geometry and cannot reach the layout.
    pub const GRACE_PTS: f32 = 4.0;

    /// A surface that does not hide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The setting.
    #[must_use]
    pub fn mode(&self) -> AutoHide {
        self.mode
    }

    /// Change the setting.
    ///
    /// Switching **on** hides the surface immediately unless the pointer is
    /// already over its trigger, which the next [`Self::resolve`] decides.
    /// Switching **off** shows it, and clears the reveal so that turning the
    /// setting on again does not inherit a stale `true`.
    pub fn set_mode(&mut self, mode: AutoHide) {
        self.mode = mode;
        self.revealed = false;
        self.overlay = None;
    }

    /// Whether the surface's body was drawn on the last resolved frame.
    ///
    /// For a report line and for a driven check; **not** for a layout decision,
    /// which must use this frame's [`Self::resolve`].
    #[must_use]
    pub fn is_revealed(&self) -> bool {
        self.revealed
    }

    /// **Decide what the surface does this frame.**
    ///
    /// `trigger` is the always-present rectangle that reveals the surface —
    /// the ribbon's tab strip, the rail's sliver. It **must not be derived
    /// from the surface's own body**; see the module header, and see
    /// [`Self::MIN_TRIGGER_PTS`] for what happens when it is degenerate.
    ///
    /// `pointer` is the latest pointer position, or `None` when the pointer is
    /// outside the window — which closes the surface, because a pointer that
    /// has left cannot be hovering anything.
    ///
    /// `holds_focus` is whether a widget inside the overlay currently has
    /// keyboard focus. A **keep** term only.
    pub fn resolve(&mut self, trigger: Rect, pointer: Option<Pos2>, holds_focus: bool) -> Show {
        if !self.mode.is_on() {
            self.revealed = true;
            self.overlay = None;
            return Show::Inline;
        }

        // ★ THE FLOOR. A trigger too small to hit makes the surface
        // unreachable, so the surface stops hiding rather than becoming
        // unreachable. Checked before anything else, because every clause
        // below assumes the operator has a way in.
        if !trigger.is_finite()
            || trigger.width() < Self::MIN_TRIGGER_PTS
            || trigger.height() < Self::MIN_TRIGGER_PTS
        {
            self.revealed = true;
            self.overlay = None;
            return Show::Inline;
        }

        // ★★★ THE DIRECTION BOUND, in one expression. `in(trigger)` is the
        // only disjunct that can be true on a frame where `self.revealed` is
        // false, so a reveal can only ever be STARTED by the trigger — which
        // no overlay can move.
        let in_trigger = pointer.is_some_and(|p| trigger.expand(Self::GRACE_PTS).contains(p));
        let in_overlay = self.revealed
            && self
                .overlay
                .is_some_and(|r| pointer.is_some_and(|p| r.expand(Self::GRACE_PTS).contains(p)));
        let kept_by_keyboard = self.revealed && holds_focus;

        self.revealed = in_trigger || in_overlay || kept_by_keyboard;
        if !self.revealed {
            self.overlay = None;
        }
        if self.revealed {
            Show::Overlay
        } else {
            Show::Hidden
        }
    }

    /// Record where the overlay was actually drawn.
    ///
    /// Called by the surface **after** it has drawn, with the rectangle the
    /// body occupied. Until it is called the overlay term of the invariant is
    /// false, which is the safe direction: a surface that drew and forgot to
    /// report closes as soon as the pointer leaves the trigger, rather than
    /// staying open over a rectangle nobody can see.
    pub fn record_overlay(&mut self, rect: Rect) {
        if self.revealed {
            self.overlay = Some(rect);
        }
    }

    /// Where the overlay was drawn last frame, if it was.
    ///
    /// Published for the region a harness asserts against — see
    /// `crate::verify` — and for a check that wants to prove the overlay does
    /// not move the surface beneath it.
    #[must_use]
    pub fn overlay(&self) -> Option<Rect> {
        self.overlay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn strip() -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(1400.0, 34.0))
    }

    fn band() -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 34.0), vec2(1400.0, 96.0))
    }

    /// Off means inline, whatever the pointer is doing.
    #[test]
    fn a_surface_that_does_not_hide_is_always_inline() {
        let mut peek = Peek::new();
        assert_eq!(peek.resolve(strip(), None, false), Show::Inline);
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), false),
            Show::Inline
        );
        assert!(peek.is_revealed());
    }

    /// The trigger reveals; leaving both rectangles hides.
    #[test]
    fn the_trigger_reveals_and_leaving_hides() {
        let mut peek = Peek::new();
        peek.set_mode(AutoHide::OnHover);

        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), false),
            Show::Hidden,
            "the pointer is over the document"
        );
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 20.0)), false),
            Show::Overlay,
            "the pointer reached the tab strip"
        );
        peek.record_overlay(band());
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 100.0)), false),
            Show::Overlay,
            "the pointer moved down into the band it just revealed"
        );
        peek.record_overlay(band());
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), false),
            Show::Hidden,
            "and away again"
        );
    }

    /// ★★★ **THE DIRECTION BOUND.** The overlay's rectangle can never *start* a
    /// reveal — only keep one alive.
    ///
    /// Planted state rather than a default one: `overlay` is written by hand to
    /// last frame's band while `revealed` is false, which is the exact
    /// configuration a "remember where it was" implementation reaches after the
    /// pointer leaves and comes back. A pointer standing in the middle of that
    /// remembered rectangle must NOT bring the band back, or the band's own
    /// area becomes a second trigger — and a trigger whose position depends on
    /// the thing it triggers is R128's loop.
    #[test]
    fn a_remembered_overlay_cannot_start_a_reveal() {
        let mut peek = Peek {
            mode: AutoHide::OnHover,
            revealed: false,
            overlay: Some(band()),
        };
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 100.0)), false),
            Show::Hidden,
            "a point inside last frame's band, with the band hidden, must stay hidden"
        );
        assert_eq!(peek.overlay(), None, "and the stale rectangle is dropped");
    }

    /// ★★ The state cannot oscillate with the pointer held still.
    ///
    /// Swept over a grid of pointer positions covering the strip, the band and
    /// the document, forty frames each. Two consecutive frames with the same
    /// input must give the same answer — which is what "monotone decreasing
    /// while the pointer is still" means operationally, and is the property a
    /// "don't ask twice" guard would only appear to have.
    #[test]
    fn a_still_pointer_settles_and_stays_settled() {
        for x in [-10.0_f32, 0.0, 700.0, 1399.0, 1500.0] {
            for y in [-10.0_f32, 0.0, 20.0, 34.0, 100.0, 130.0, 600.0] {
                let mut peek = Peek::new();
                peek.set_mode(AutoHide::OnHover);
                let mut answers = Vec::new();
                for _ in 0..40 {
                    let show = peek.resolve(strip(), Some(pos2(x, y)), false);
                    if show == Show::Overlay {
                        peek.record_overlay(band());
                    }
                    answers.push(show);
                }
                let settled = *answers.last().expect("forty frames");
                assert!(
                    answers[2..].iter().all(|a| *a == settled),
                    "at ({x}, {y}) the surface never settled: {answers:?}"
                );
            }
        }
    }

    /// ★★★ **THE FLOOR fails OPEN.** A trigger too small to hit does not hide
    /// the surface; it stops the surface hiding.
    ///
    /// Walked across the whole width series rather than at the two endpoints,
    /// because a floor asserted only at 0 and at 8 would pass for an
    /// implementation that used `<= 0.0`.
    #[test]
    fn a_trigger_too_small_to_hit_makes_the_surface_stop_hiding() {
        for thin in [0.0_f32, 0.5, 1.0, 2.0, 4.0, 7.0, 7.99] {
            let mut peek = Peek::new();
            peek.set_mode(AutoHide::OnHover);
            let sliver = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(thin, 800.0));
            assert_eq!(
                peek.resolve(sliver, Some(pos2(700.0, 600.0)), false),
                Show::Inline,
                "a {thin} pt trigger is not hittable, so the surface must not hide behind it"
            );
        }
        // …and at the floor itself it hides again.
        let mut peek = Peek::new();
        peek.set_mode(AutoHide::OnHover);
        let sliver = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(Peek::MIN_TRIGGER_PTS, 800.0));
        assert_eq!(
            peek.resolve(sliver, Some(pos2(700.0, 600.0)), false),
            Show::Hidden
        );
    }

    /// A pointer that has left the window closes the surface.
    #[test]
    fn no_pointer_closes_it() {
        let mut peek = Peek::new();
        peek.set_mode(AutoHide::OnHover);
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 20.0)), false),
            Show::Overlay
        );
        peek.record_overlay(band());
        assert_eq!(peek.resolve(strip(), None, false), Show::Hidden);
    }

    /// Keyboard focus keeps a revealed surface open and cannot open a hidden
    /// one.
    #[test]
    fn focus_keeps_it_open_but_cannot_open_it() {
        let mut peek = Peek::new();
        peek.set_mode(AutoHide::OnHover);

        // Hidden, and focus claims otherwise: still hidden.
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), true),
            Show::Hidden,
            "focus is a KEEP term; it cannot start a reveal"
        );

        // Revealed by the trigger, then the pointer leaves while a control
        // inside holds the keyboard.
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 20.0)), false),
            Show::Overlay
        );
        peek.record_overlay(band());
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), true),
            Show::Overlay,
            "a keyboard user inside the band does not lose it to the pointer"
        );
    }

    /// Turning the setting off shows the surface and forgets the reveal.
    #[test]
    fn turning_it_off_shows_the_surface() {
        let mut peek = Peek::new();
        peek.set_mode(AutoHide::OnHover);
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 20.0)), false),
            Show::Overlay
        );
        peek.record_overlay(band());

        peek.set_mode(AutoHide::Off);
        assert_eq!(
            peek.resolve(strip(), Some(pos2(700.0, 600.0)), false),
            Show::Inline
        );
        assert_eq!(peek.overlay(), None);
    }

    /// The two settings are each other's toggle, and `Off` is the default.
    #[test]
    fn the_default_does_not_hide() {
        assert_eq!(AutoHide::default(), AutoHide::Off);
        assert!(AutoHide::default().is_default());
        assert_eq!(AutoHide::Off.toggled(), AutoHide::OnHover);
        assert_eq!(AutoHide::OnHover.toggled(), AutoHide::Off);
        assert!(!AutoHide::Off.is_on());
        assert!(AutoHide::OnHover.is_on());
    }

    /// `Show`'s two questions are not the same question.
    #[test]
    fn an_overlay_draws_its_body_and_takes_no_room() {
        assert!(Show::Inline.draws_body() && Show::Inline.takes_room());
        assert!(Show::Overlay.draws_body() && !Show::Overlay.takes_room());
        assert!(!Show::Hidden.draws_body() && !Show::Hidden.takes_room());
    }
}
