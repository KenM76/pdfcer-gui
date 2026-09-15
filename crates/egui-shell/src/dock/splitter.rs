//! The draggable boundary between two neighbouring compartments.
//!
//! # One unambiguous grab affordance — failure mode #1
//!
//! `MODES_AND_PANELS.md`'s failure-mode table opens with the most-reported
//! docking complaint in the benchmarked application: an OS title bar and an
//! application's own drag handle stacked vertically, so grabbing the wrong
//! one silently does nothing. Its rule — **one unambiguous grab
//! affordance** — governs every grab surface in this module, and three
//! things follow from it, all implemented here rather than left to the
//! caller:
//!
//! 1. **The hit target is larger than the line.** The painted rule is
//!    one or two points; the interactive rectangle is
//!    [`super::plan::SPLITTER_THICKNESS`]. A grab that requires precision
//!    is a grab that fails, and a failed grab is indistinguishable from a
//!    feature that does not exist.
//! 2. **The cursor changes on hover.** This is the affordance. `egui`'s
//!    `ResizeHorizontal` / `ResizeVertical` icons say *"this is draggable
//!    and in which direction"* before the operator commits to a drag, and
//!    they say it in the one vocabulary every desktop already shares.
//! 3. **The line highlights while hovered or dragged.** Failure mode #2 is
//!    weak drop feedback, and its rule is that feedback must encode the
//!    **outcome**, not merely that a drag is valid. A splitter's outcome is
//!    "this boundary will move", so the boundary itself is what lights up,
//!    rather than a cursor-following ghost that says only "a drag is in
//!    progress".
//!
//! # Why this returns a delta instead of mutating the model
//!
//! A splitter has no idea what it divides. It reports how far it was
//! dragged, in points, along its own axis, and the dock's renderer applies
//! that to exactly two spans through
//! `super::plan::drag_boundary` — the function whose entire job is
//! failure mode #7 (*a splitter affects its two neighbours only*).
//!
//! Keeping the widget ignorant of the model is what makes that
//! guarantee checkable. A splitter that reached into the layout could
//! renormalise every share on its way past, which is exactly how coupled
//! splitters happen, and no reading of this file would reveal it.
//!
//! # Double-click to equalise
//!
//! A double-click on a boundary divides the two neighbours evenly. It
//! costs one line and it is the cheapest possible answer to "I have
//! dragged this into a mess" short of a full reset — which, per
//! `RIBBON_IA.md`, must never be the only way back.

use egui::{Color32, CursorIcon, Id, Rect, Sense, Ui, Vec2};

use crate::theme::Theme;

/// Which way a splitter divides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    /// A vertical rule between two side-by-side columns; drags left and
    /// right.
    Horizontal,
    /// A horizontal rule between two stacked compartments; drags up and
    /// down.
    Vertical,
}

impl Axis {
    /// The cursor that says which way this boundary moves.
    fn cursor(self) -> CursorIcon {
        match self {
            Axis::Horizontal => CursorIcon::ResizeHorizontal,
            Axis::Vertical => CursorIcon::ResizeVertical,
        }
    }

    /// The component of a drag delta that matters to this axis.
    fn component(self, v: Vec2) -> f32 {
        match self {
            Axis::Horizontal => v.x,
            Axis::Vertical => v.y,
        }
    }
}

/// What one splitter did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SplitterOutcome {
    /// How far the boundary was dragged this frame, in points, along its
    /// own axis. Positive is right or down.
    pub delta: f32,
    /// Whether the operator asked for the two neighbours to be equalised.
    pub equalize: bool,
    /// Whether the splitter is being hovered or dragged, which the caller
    /// uses to decide whether a repaint is worth requesting.
    pub active: bool,
}

impl SplitterOutcome {
    /// Whether anything happened that the layout must respond to.
    pub(crate) fn changed(self) -> bool {
        self.equalize || self.delta.abs() > f32::EPSILON
    }
}

/// Draw one splitter in `rect` and report what the operator did to it.
///
/// `rect` is the **interactive** rectangle — the full
/// [`super::plan::SPLITTER_THICKNESS`] — and the painted rule is a
/// centred sliver of it. See the module header on why those are
/// different sizes.
pub(crate) fn splitter(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    axis: Axis,
    theme: &Theme,
) -> SplitterOutcome {
    // A zero-area rect happens for one frame when a compartment is
    // closed while its splitter is on screen. Interacting with it would
    // register a widget at an impossible position, and `egui` would then
    // hold focus somewhere unreachable.
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return SplitterOutcome::default();
    }

    let response = ui.interact(rect, id, Sense::click_and_drag());
    let active = response.hovered() || response.dragged();

    if active {
        ui.ctx().set_cursor_icon(axis.cursor());
    }

    // The painted rule: thin when idle, the full thickness when the
    // operator is on it. The *width* change is the feedback, not only the
    // colour: a colour-only cue disappears in greyscale and for a
    // colour-blind operator, so a state that matters carries a second,
    // non-colour channel.
    let (thickness, colour) = if active {
        (rect.size().min_elem(), theme.palette.accent)
    } else {
        (1.0_f32, idle_colour(theme))
    };
    let painted = match axis {
        Axis::Horizontal => {
            Rect::from_center_size(rect.center(), Vec2::new(thickness, rect.height()))
        }
        Axis::Vertical => Rect::from_center_size(rect.center(), Vec2::new(rect.width(), thickness)),
    };
    ui.painter().rect_filled(painted, 0.0, colour);

    // A splitter is a control, and a control that is reachable by keyboard
    // but unnamed to AccessKit is the worst accessibility outcome: the
    // focus ring lands on it and the screen reader has nothing to say.
    // `egui` 0.35 has no separator or splitter `WidgetType`, so the role
    // cannot be supplied; the name can, and is.
    let name = match axis {
        Axis::Horizontal => "Column divider — drag to resize",
        Axis::Vertical => "Row divider — drag to resize",
    };
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, name.to_owned()));

    SplitterOutcome {
        delta: if response.dragged() {
            axis.component(response.drag_delta())
        } else {
            0.0
        },
        equalize: response.double_clicked(),
        active,
    }
}

/// The colour a splitter paints when idle, exposed for the contrast
/// check.
///
/// A separator drawn in a colour indistinguishable from the panel it
/// sits on is invisible, and an invisible boundary is one the operator
/// never learns is draggable. The theme's own contrast gate covers text
/// pairs; this is the one non-text pair the dock adds, and it is checked
/// by `the_idle_splitter_is_distinguishable_from_the_panel`.
#[must_use]
pub(crate) fn idle_colour(theme: &Theme) -> Color32 {
    theme.palette.outline
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Preset;

    /// A drag delta is read along the splitter's own axis and nothing
    /// else, so a diagonal mouse movement on a column divider does not
    /// leak into the vertical arrangement.
    #[test]
    fn a_drag_is_read_along_one_axis_only() {
        let v = Vec2::new(12.0, -30.0);
        assert_eq!(Axis::Horizontal.component(v), 12.0);
        assert_eq!(Axis::Vertical.component(v), -30.0);
    }

    /// Each axis offers the cursor that names its direction — the
    /// affordance that answers failure mode #1.
    #[test]
    fn each_axis_offers_a_direction_naming_cursor() {
        assert_eq!(Axis::Horizontal.cursor(), CursorIcon::ResizeHorizontal);
        assert_eq!(Axis::Vertical.cursor(), CursorIcon::ResizeVertical);
        assert_ne!(Axis::Horizontal.cursor(), Axis::Vertical.cursor());
    }

    /// An outcome with no drag and no double-click asks the layout for
    /// nothing, so a merely-hovered splitter cannot mark a layout dirty.
    #[test]
    fn hovering_alone_does_not_change_the_layout() {
        let hovered = SplitterOutcome {
            delta: 0.0,
            equalize: false,
            active: true,
        };
        assert!(!hovered.changed());
        assert!(
            SplitterOutcome {
                delta: 3.0,
                ..hovered
            }
            .changed()
        );
        assert!(
            SplitterOutcome {
                equalize: true,
                ..hovered
            }
            .changed()
        );
    }

    /// ★ **The idle splitter is visible against the panel it divides.**
    ///
    /// A boundary drawn in the panel's own colour is invisible, and an
    /// invisible boundary is never discovered to be draggable — the
    /// silent half of failure mode #1. Checked for every shipped preset,
    /// because a colour pair that holds in one theme and collapses in
    /// another passes any check that looks at a single palette.
    #[test]
    fn the_idle_splitter_is_distinguishable_from_the_panel() {
        for preset in Preset::ALL {
            let theme = Theme::new(*preset);
            let line = idle_colour(&theme);
            let panel = theme.palette.panel;
            let distance = (i32::from(line.r()) - i32::from(panel.r())).abs()
                + (i32::from(line.g()) - i32::from(panel.g())).abs()
                + (i32::from(line.b()) - i32::from(panel.b())).abs();
            assert!(
                distance > 30,
                "{preset:?}: the divider {line:?} is invisible on the panel {panel:?}"
            );
        }
    }

    /// A zero-area splitter is not interacted with at all.
    ///
    /// It happens for exactly one frame when a compartment closes while
    /// its splitter is on screen, and registering a widget at an
    /// impossible position would leave `egui` holding focus somewhere
    /// unreachable.
    #[test]
    fn a_zero_area_splitter_does_nothing() {
        let ctx = egui::Context::default();
        let mut outcome = SplitterOutcome::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            outcome = splitter(
                ui,
                Id::new("probe"),
                Rect::from_min_size(egui::Pos2::ZERO, Vec2::ZERO),
                Axis::Horizontal,
                &Theme::new(Preset::Quiet),
            );
        });
        assert!(!outcome.changed());
        assert!(!outcome.active);
    }
}
