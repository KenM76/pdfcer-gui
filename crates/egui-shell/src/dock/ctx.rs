//! The per-frame render context every dock surface is handed, and the
//! intent queue that keeps the layout immutable while it is being drawn.
//!
//! # Why a context struct rather than seven parameters
//!
//! The side renderer, the tab bar, the overflow menu and the splitters
//! all need the same five things: the panel registry, the theme, the rect
//! reporter, an id salt, and somewhere to record what the operator did.
//! Threading those as parameters makes every new capability a signature
//! change in four files, and — worse — makes it possible for one surface
//! to be handed a *different* registry than another, producing a dock
//! whose tabs and whose menu disagree about which panels exist. One
//! value, constructed once per frame, removes both. This mirrors
//! [`crate::ribbon::ctx`] deliberately, so a reader who has understood
//! one has understood the other.
//!
//! # ★ Why the layout is read-only while it is drawn
//!
//! This is the design decision in this file that matters most, and it is
//! not a style preference — it is what makes failure mode #6 (*restore,
//! do not recompute*) mechanically true rather than merely intended.
//!
//! An immediate-mode dock is a loop over the layout that also *responds*
//! to clicks on the layout. The obvious spelling mutates as it goes:
//! click a tab, set `stack.active`, carry on drawing. That spelling has
//! three problems, in increasing order of how long they take to find:
//!
//! 1. **Borrow gymnastics.** The body callback is the application's, it
//!    holds `&mut` to the application, and the layout is being iterated.
//!    Every solution is a `mem::replace` dance — which the previous
//!    implementation had to do, and documented as a gotcha future readers
//!    would otherwise rediscover: *"`Tree<Pane>` derives only
//!    `Clone, PartialEq` — not `Default` — so `std::mem::take` will not
//!    compile."*
//! 2. **Half-applied frames.** A tab activated in the middle of a draw is
//!    visible to the compartments drawn after it and invisible to the
//!    ones drawn before, so one frame shows two different truths.
//! 3. **Silent write-back.** Once mutation during drawing is normal, a
//!    resize handler that stores a *computed* span back into a share
//!    looks exactly like every other line in the file. That is the
//!    failure-mode-#6 defect, and no code reading finds it, because
//!    nothing about it looks wrong.
//!
//! So: the frame renders from an immutable snapshot, records
//! `Intent`s, and the intents are applied afterwards, in one place,
//! by one function. The layout is `&` for the whole of the draw, and
//! `&mut` for four lines afterwards. Problem 3 becomes impossible by
//! construction — there is no `&mut` in scope where a span is computed —
//! and `the_layout_survives_a_round_trip_through_a_narrow_window` is the
//! test that says so.
//!
//! The cost is that a splitter drag lands on the next frame rather than
//! this one. `egui` is already requesting a repaint for the duration of a
//! drag, so this is one frame of latency during a pointer-down gesture
//! and is not perceptible.

use egui::Id;

use super::model::{DockSide, PanelId, PanelRegistry};
use super::report::Reporter;
use super::tab_menu::TabMenuHandler;
use crate::theme::Theme;

/// Something the operator did that the layout must respond to.
///
/// Collected during a frame and applied after it — see the module
/// header. A struct-like enum rather than a callback so the whole set of
/// mutations the dock can perform is **one readable list**, which is what
/// makes "nothing else writes to the layout" a claim a reviewer can
/// check.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Intent {
    /// Make a panel the active tab of its stack.
    Activate(PanelId),
    /// Remove a panel from the layout.
    Close(PanelId),
    /// ★★★ **Tear a panel out into a window of its own.**
    ///
    /// An intent rather than a direct write for the same reason every
    /// other one here is — the layout is mutated in exactly one place,
    /// after the frame has drawn — and for one reason peculiar to this
    /// one: floating a panel *removes it from the tree*, so a direct write
    /// mid-frame would pull a compartment out from under a body that had
    /// already been laid out into it.
    Float(PanelId),
    /// **Put a floating panel back where it came from.**
    ///
    /// The mirror of [`Self::Float`]. Raised by the float window's own
    /// header control and by the application's dock command.
    Dock(PanelId),
    /// **Remember where the operator has left a float window.**
    ///
    /// Raised once per frame per open float window, from the geometry the
    /// platform reports. Applying it is a no-op when nothing moved — see
    /// [`crate::dock::DockLayout::set_float_geometry`], whose return value
    /// is what stops a still window marking the layout dirty sixty times a
    /// second and saving `layout.ron` on every one of them.
    FloatGeometry {
        /// Which floating panel.
        panel: PanelId,
        /// Its window's outer position in desktop points, if the platform
        /// reported one. `None` is *"not placed yet"*, never *"at zero"*.
        pos: Option<[f32; 2]>,
        /// Its window's inner size in points.
        size: [f32; 2],
    },
    /// ★★ **Collapse a side, or bring it back.**
    ///
    /// The operator's ask of 2026-08-20: *"add the little tabs that allow the
    /// left and right panels to be minimized."*
    ///
    /// An intent rather than a direct write for the reason every other one here
    /// is: the layout is mutated in exactly one place, after the frame has
    /// drawn. A control that flipped `visible` mid-frame would change the width
    /// of a panel that has already laid out inside it.
    ///
    /// # Why it carries no target state
    ///
    /// A toggle, not a setter. Two controls raise it — the collapse chevron on
    /// an open side and the rail on a shut one — and neither can be pressed in
    /// the state the other lives in, so "what should it become" is always the
    /// opposite of what it is. A `Set(side, bool)` would let the two disagree.
    ToggleSide(DockSide),
    /// Move the boundary between two columns of a side.
    DragColumns {
        /// Which side.
        side: DockSide,
        /// The boundary index: between column `boundary` and `boundary + 1`.
        boundary: usize,
        /// How far, in points.
        delta: f32,
    },
    /// Give two neighbouring columns equal width.
    EqualizeColumns {
        /// Which side.
        side: DockSide,
        /// The boundary index.
        boundary: usize,
    },
    /// Move the boundary between two stacks of a column.
    DragStacks {
        /// Which side.
        side: DockSide,
        /// Which column.
        column: usize,
        /// The boundary index.
        boundary: usize,
        /// How far, in points.
        delta: f32,
    },
    /// Give two neighbouring stacks equal height.
    EqualizeStacks {
        /// Which side.
        side: DockSide,
        /// Which column.
        column: usize,
        /// The boundary index.
        boundary: usize,
    },
    /// Change a whole side's width.
    ///
    /// The only intent that writes an **absolute** number, and the only
    /// one that touches [`super::model::SideLayout::width_pts`]. See that
    /// field's documentation for why the dock's outer width is the one
    /// dimension stored in points.
    DragSide {
        /// Which side.
        side: DockSide,
        /// How far, in points, positive meaning wider.
        delta: f32,
    },
}

/// Everything a dock surface needs for one frame.
///
/// Not `Clone`, not `Copy`, and never stored: it borrows the
/// application's registry and sink for the duration of one
/// [`super::Dock::show`] call and is dropped at the end of it.
pub(crate) struct Ctx<'a> {
    /// What the application can draw. A layout may only reference these.
    pub registry: Option<&'a PanelRegistry>,
    /// The look in force, read from the `egui` context once per frame.
    pub theme: Theme,
    /// Where drawn rectangles are published, if anywhere.
    pub reporter: Reporter<'a>,
    /// Distinguishes two docks in one window; see [`Self::id`].
    pub id_salt: Id,
    /// Who owns a tab's secondary click, if not the dock.
    ///
    /// `None` — the overwhelmingly common case, and the one every existing
    /// consumer is in — means the dock draws its own **Close** menu, as it
    /// always has. `Some` means the application asked for the tab's
    /// `Response` and the dock steps aside entirely, because a `Response`
    /// has exactly one popup id and therefore exactly one owner. See
    /// [`super::tab_menu`] for the whole argument.
    ///
    /// Carried on the context rather than threaded as a parameter for the
    /// reason in this module's header: the alternative is a signature
    /// change in four files per capability.
    pub tab_menu: Option<&'a mut TabMenuHandler<'a>>,
    /// What the operator did, in the order they did it.
    pub intents: Vec<Intent>,
}

impl Ctx<'_> {
    /// A stable widget id for one part of one compartment.
    ///
    /// Every interactive element in the dock derives its id from the
    /// **structural address** — side, column, stack, role — rather than
    /// from a counter or from the panel's id. That is deliberate and it
    /// is the property `egui` needs: an id that changed when a tab was
    /// activated would end the in-flight interaction that caused the
    /// activation, and an id that changed when a panel moved would reset
    /// a splitter drag mid-gesture.
    ///
    /// The previous implementation names the same hazard about its
    /// engine's tree id: *"changing it between frames would silently
    /// reset every in-flight interaction."*
    pub(crate) fn id(&self, role: &str, side: DockSide, column: usize, stack: usize) -> Id {
        self.id_salt.with((role, side, column, stack))
    }

    /// The label and tooltip for a panel, falling back to the id itself.
    ///
    /// **Falling back rather than skipping is load-bearing.** A panel
    /// mounted in the layout but absent from the registry is normally
    /// dropped at load time with a disclosed reason
    /// ([`crate::layout`]), so reaching this fallback means the
    /// application registered panels *after* loading, or mutated the
    /// layout by hand. Drawing the raw id is ugly and truthful; drawing
    /// nothing would give a tab with no name, which is
    /// indistinguishable from a rendering fault and impossible to report
    /// usefully.
    pub(crate) fn describe(&self, panel: &PanelId) -> (String, String) {
        match self.registry.and_then(|r| r.get(panel.as_str())) {
            Some(info) => (info.label.clone(), info.accessible_name().to_owned()),
            None => (panel.as_str().to_owned(), panel.as_str().to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Preset;

    fn ctx() -> Ctx<'static> {
        Ctx {
            registry: None,
            theme: Theme::new(Preset::Quiet),
            reporter: Reporter::new(None),
            id_salt: Id::new("dock-test"),
            tab_menu: None,
            intents: Vec::new(),
        }
    }

    /// Two compartments never share an id, or `egui` would treat a click
    /// on one as a click on the other.
    #[test]
    fn structurally_distinct_compartments_get_distinct_ids() {
        let c = ctx();
        let a = c.id("tabbar", DockSide::Left, 0, 0);
        assert_ne!(a, c.id("tabbar", DockSide::Right, 0, 0));
        assert_ne!(a, c.id("tabbar", DockSide::Left, 1, 0));
        assert_ne!(a, c.id("tabbar", DockSide::Left, 0, 1));
        assert_ne!(a, c.id("overflow", DockSide::Left, 0, 0));
    }

    /// The same compartment gets the same id on every frame, so an
    /// in-flight drag is not cancelled by a redraw.
    #[test]
    fn the_same_compartment_gets_a_stable_id_across_frames() {
        assert_eq!(
            ctx().id("split", DockSide::Left, 2, 0),
            ctx().id("split", DockSide::Left, 2, 0)
        );
    }

    /// An unregistered panel still gets a name — its own id — rather than
    /// an unnamed tab that reads as a rendering fault.
    #[test]
    fn an_unregistered_panel_falls_back_to_its_own_id() {
        let (label, name) = ctx().describe(&PanelId::new("mystery"));
        assert_eq!(label, "mystery");
        assert_eq!(name, "mystery");
    }

    /// A registered panel gets its label, and its accessible name is the
    /// tooltip when it has one.
    #[test]
    fn a_registered_panel_gets_its_label_and_its_purpose_tooltip() {
        let mut registry = PanelRegistry::new();
        registry.register(
            super::super::model::PanelInfo::new("pages", "Pages")
                .with_tooltip("Pages — jump to a page, reorder or rotate sheets"),
        );
        let c = Ctx {
            registry: Some(&registry),
            theme: Theme::new(Preset::Quiet),
            reporter: Reporter::new(None),
            id_salt: Id::new("dock-test"),
            tab_menu: None,
            intents: Vec::new(),
        };
        let (label, name) = c.describe(&PanelId::new("pages"));
        assert_eq!(label, "Pages");
        assert!(
            name.len() > label.len(),
            "the announced name is the purpose"
        );
    }
}
