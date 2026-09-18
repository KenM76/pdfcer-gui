//! `dock::state` — everything about the dock that outlives one frame.
//!
//! [`super::Dock`] is rebuilt every frame and holds nothing, so the arrangement
//! and every measurement or flag a later frame needs live here. The reason each
//! item is here rather than on [`DockFrameReport`] is the same in every case: a
//! report is rebuilt from nothing each frame, so it can hold a *claim about this
//! frame* and never a *memory of the last one*.
//!
//! The application owns one of these per dock and hands it to
//! [`super::Dock::show`] and then [`super::Dock::show_floating`]. Both write to
//! it, in that order.

use super::DockFrameReport;
use super::floatdrag::FloatDrag;
use super::geometry::DockGeometry;
use super::model::{DockLayout, PanelId};

/// The dock's live state: the arrangement, plus what the last frame did.
///
/// Held by the application across frames. `Clone` so a workspace can be
/// snapshotted; `PartialEq` so a test can assert a frame changed nothing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockState {
    pub(super) layout: DockLayout,
    pub(super) last_frame: DockFrameReport,
    /// How many float windows [`super::Dock::show_floating`] drew last frame.
    ///
    /// The counter half of [`DockFrameReport::floats_undrawn`]. It lives
    /// here rather than on the report because it has to survive from one
    /// frame to the next, and the report is rebuilt every frame — which is
    /// exactly the property that makes it the *claim* rather than the
    /// *fact*.
    pub(super) floats_drawn: usize,
    /// **Every panel [`super::Dock::show_floating`] has opened a window for and
    /// not yet cleaned up after.**
    ///
    /// The window remembers, in `egui`'s temporary data, that it has been
    /// opened, so that its stored position is asserted once and not re-asserted
    /// every frame. Something has to clear that flag when the window goes, and
    /// the set of windows *about to be drawn* cannot: a panel docked back by a
    /// route outside `show_floating` — the *Dock all* command, a drop
    /// [`super::Dock::show`] applied earlier in the same frame — has already
    /// left that set. This is the list that still contains it.
    pub(super) floats_seen: Vec<PanelId>,
    /// **The left rail's auto-hide state.** See [`crate::peek`].
    ///
    /// On [`DockState`] rather than on [`super::Dock`] because a `Dock` is built fresh
    /// every frame and this has to survive between them — the same reason
    /// [`Self::floats_drawn`] lives here. The *setting* half is an operator
    /// preference the application restores through [`Self::set_rail_auto_hide`];
    /// the *revealed* half is per-frame and is never persisted.
    pub(super) rail_peek: crate::peek::Peek,
    /// **Where every compartment was drawn on the last frame.** See
    /// [`geometry`], whose header carries when this is current and what it
    /// does not claim.
    pub(super) geometry: DockGeometry,
    /// **A float window the operator is carrying**, reported by the application
    /// and consumed by the next [`super::Dock::show`]. See
    /// [`Self::set_float_drag`].
    pub(super) float_drag: Option<FloatDrag>,
}

impl DockState {
    /// Start from an arrangement.
    ///
    /// The arrangement is normalized on the way in, so an application's
    /// built-in default cannot ship a stack with no tabs or a panel
    /// mounted twice. An application that wants to *know* whether its
    /// default needed repair asserts [`DockLayout::is_normalized`] in its
    /// own test suite — the same posture `manifest` takes towards its
    /// built-in layer, and for the same reason: a defect in a compiled-in
    /// constant should fail a test, not be quietly patched on every
    /// machine that runs it.
    #[must_use]
    pub fn new(layout: DockLayout) -> Self {
        let mut layout = layout;
        layout.normalize();
        Self {
            layout,
            last_frame: DockFrameReport::default(),
            floats_drawn: 0,
            floats_seen: Vec::new(),
            rail_peek: crate::peek::Peek::new(),
            geometry: DockGeometry::default(),
            float_drag: None,
        }
    }

    /// **Report a float window being carried back over the dock**, once per
    /// frame, for as long as the gesture lasts.
    ///
    /// The extension point that lets the dock offer its drop grammar to a
    /// gesture whose pointer it cannot see — see [`super::floatdrag`], whose
    /// header carries why that pointer is the application's to supply and what
    /// space it must be in.
    ///
    /// **Consumed, not held.** The next [`super::Dock::show`] takes it, so a
    /// caller that stops renewing the report ends the gesture with nothing
    /// landed. That is what makes a window closed mid-drag, or a platform that
    /// lost the pointer, safe by construction rather than by a cancel path
    /// somebody has to remember to call.
    pub fn set_float_drag(&mut self, drag: Option<FloatDrag>) {
        self.float_drag = drag;
    }

    /// The current arrangement.
    #[must_use]
    pub fn layout(&self) -> &DockLayout {
        &self.layout
    }

    /// **Whether the rail hides itself until the pointer reaches its edge.**
    ///
    /// See [`crate::peek`] for the model, and [`rail::PEEK_WIDTH_PTS`] for the
    /// sliver that is reserved in its place — the strip never disappears
    /// entirely, because it is the only route to some panels and a rail that
    /// vanished would take them with it.
    #[must_use]
    pub fn rail_auto_hide(&self) -> crate::peek::AutoHide {
        self.rail_peek.mode()
    }

    /// Turn the rail's auto-hide on or off.
    pub fn set_rail_auto_hide(&mut self, mode: crate::peek::AutoHide) {
        self.rail_peek.set_mode(mode);
    }

    /// Push a stored preference in once per frame without disturbing the
    /// reveal. See [`crate::ribbon::RibbonState::sync_auto_hide`], which
    /// carries the whole argument for why the frame-loop call is a different
    /// method from the operator-action one.
    pub fn sync_rail_auto_hide(&mut self, mode: crate::peek::AutoHide) {
        if self.rail_peek.mode() != mode {
            self.rail_peek.set_mode(mode);
        }
    }

    /// Whether the rail was drawn at its full width on the last frame.
    #[must_use]
    pub fn rail_is_revealed(&self) -> bool {
        self.rail_peek.is_revealed()
    }

    /// The current arrangement, mutably.
    ///
    /// The application's route to everything [`DockLayout`] can do —
    /// mounting a panel, hiding a side, applying a workspace, resetting a
    /// scope. Deliberately **not** normalized on the way out: a caller
    /// making several edits should not pay a repair pass per edit. Call
    /// [`Self::normalize`] when the edits are finished, or let the next
    /// [`super::Dock::show`] do it.
    pub fn layout_mut(&mut self) -> &mut DockLayout {
        &mut self.layout
    }

    /// Replace the arrangement wholesale — how a named workspace is
    /// applied.
    pub fn set_layout(&mut self, layout: DockLayout) {
        self.layout = layout;
        self.layout.normalize();
    }

    /// Repair every structural invariant. See [`DockLayout::normalize`].
    pub fn normalize(&mut self) {
        self.layout.normalize();
    }

    /// What the last frame drew.
    #[must_use]
    pub fn last_frame(&self) -> &DockFrameReport {
        &self.last_frame
    }

    /// **Where every compartment was drawn on the last frame.**
    ///
    /// Rebuilt from nothing each frame, so it holds no rect for a compartment
    /// that has stopped being drawn — but it is one frame old, and a rect one
    /// frame old is indistinguishable from a current one by inspection. See
    /// [`geometry`].
    #[must_use]
    pub fn geometry(&self) -> &DockGeometry {
        &self.geometry
    }

    /// Bring a panel to the front of its stack, revealing its side.
    ///
    /// Returns `false` if the panel is not mounted — never an error, see
    /// [`DockLayout::activate`].
    pub fn activate(&mut self, panel: &PanelId) -> bool {
        self.layout.activate(panel)
    }

    /// Whether a panel's body is actually being drawn.
    #[must_use]
    pub fn is_on_screen(&self, panel: &PanelId) -> bool {
        self.layout.is_on_screen(panel)
    }
}
