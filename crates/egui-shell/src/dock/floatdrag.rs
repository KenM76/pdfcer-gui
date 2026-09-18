//! `dock::floatdrag` — carrying a float window back over the dock.
//!
//! The fourth of the dock's drag gestures and the return leg of the third:
//! [`super::tear`] carries a tab out into a window, this carries the window
//! back and drops it into the compartment **the operator aimed at** rather than
//! the one it was torn from.
//!
//! # Why the pointer is handed in rather than read
//!
//! A float is drawn in a child viewport, so while its window is being carried
//! the pointer that matters is over the *application* window and `egui`'s
//! pointer in this context is not it. Locating it is a question about window
//! origins, and `egui-shell` links no windowing crate, so the answer is
//! supplied once per frame through
//! [`super::state::DockState::set_float_drag`] and consumed by
//! [`super::Dock::show`]. An application that cannot answer reports nothing,
//! draws nothing, and still has the command route home.
//!
//! The report is consumed rather than read, so a gesture ends the moment the
//! caller stops renewing it: a window closed mid-drag, or a platform that lost
//! the pointer, lands nothing instead of leaving a live offer nobody can
//! dismiss.
//!
//! # It draws nothing of its own
//!
//! [`super::overlay::offer`] draws the compass and publishes the preview — the
//! same five zones over the same compartment, resolved by the same grammar and
//! previewed by the same replay. Two drawing paths for one question are how two
//! affordances come to promise two different outcomes, which is the disclosure
//! defect the compass exists to prevent.

use egui::Pos2;

use super::ctx::{Ctx, Intent};
use super::model::{DockLayout, PanelId};
use super::overlay;

/// **A float window the operator is carrying, and where they are pointing.**
///
/// Supplied through [`super::state::DockState::set_float_drag`] once per frame
/// for as long as the gesture lasts. See the module header.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatDrag {
    /// The floating panel being carried.
    ///
    /// A panel that is not floating is declined rather than asserted on: a
    /// caller reads its pointer a frame behind the layout it reports against,
    /// so a drag whose window closed is an ordinary race and not a caller
    /// error.
    pub panel: PanelId,
    /// **Where the operator is pointing, in the application window's own
    /// screen points** — the space [`super::Dock::show`] records its geometry
    /// in, and therefore the space a drop resolves in.
    ///
    /// Not desktop points. [`super::float::FloatingPanel::pos_pts`] and
    /// [`super::tear::TearPreview::at_pts`] are in desktop points because they
    /// place a *window*; this places a *drop*, against rectangles measured
    /// inside the application window. The conversion is the caller's, because
    /// only it knows where its own window is.
    pub pointer: Pos2,
    /// **Whether the operator let go on this frame**, which is the frame the
    /// drop is applied.
    ///
    /// Carried rather than inferred from the report ceasing, for the reason
    /// [`super::drag::settle`] reads a button release rather than the absence
    /// of a drag: a gesture that ended because the caller stopped reporting it
    /// must land nothing, and after the fact the two are indistinguishable.
    pub released: bool,
}

/// **Offer the drop for a float being carried, and apply it on release.**
///
/// Runs from [`super::Dock::show`] after [`super::overlay::draw`] and
/// [`super::tear::draw`], so a frame on which the pointer's own gesture
/// published has already published.
pub(super) fn draw(ui: &egui::Ui, ctx: &mut Ctx<'_>, layout: &DockLayout) {
    let Some(drag) = ctx.float_drag.take() else {
        return;
    };
    // ★ The stand-down, and unlike [`super::tear::draw`]'s it is reachable.
    //
    // The other three affordances read one `egui` pointer and are mutually
    // exclusive by geometry. This one reads a pointer the *caller* supplies, so
    // nothing about the physics of a single mouse stops a caller reporting a
    // float drag on a frame where a tab is also being dragged. The gesture the
    // shell sensed for itself wins.
    if ctx.tab_drag.is_some() || ctx.drop_preview.is_some() || ctx.tear.is_some() {
        return;
    }
    if !layout.is_floating(&drag.panel) {
        return;
    }
    overlay::offer(ui, ctx, layout, &drag.panel, drag.pointer);
    if !drag.released {
        return;
    }
    // Taken rather than read, as [`super::drag::settle`] takes it: the frame a
    // drop lands is not a frame on which an offer still stands, and a report
    // carrying both would be promising a release that has already happened.
    let Some(preview) = ctx.drop_preview.take() else {
        // Released clear of every compartment, so the window stays where it was
        // let go — what dragging a window ordinarily does, and
        // [`super::floatwin`] has already recorded its geometry.
        return;
    };
    ctx.intents.push(Intent::MovePanel {
        panel: preview.panel,
        target: preview.landing.target,
    });
}
