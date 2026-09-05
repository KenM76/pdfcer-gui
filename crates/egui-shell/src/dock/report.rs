//! The names the dock publishes its drawn rectangles under, and the sink
//! that receives them.
//!
//! # Why a rect stream exists at all
//!
//! `MODES_AND_PANELS.md` Part 2 lists three prerequisites before any of
//! the flexible-panel work, and the second is the one this module
//! serves:
//!
//! > **A screenshot oracle for panel layout.** Two recorded instances
//! > where a traced rect was correct and the control was still clipped
//! > out of its pane: *"layout/clipping defects have exactly one oracle:
//! > a rendered screenshot."*
//!
//! Note carefully what that says and what it does not. A rect stream is
//! **not** the oracle for legibility or for clipping; the RAG entry
//! `headless_trace_asserts_reached_not_visible_a_clipped_widget_needs_a_pixel_oracle`
//! makes the same point from the other side. What a rect stream *is* good
//! for is the class of assertion where the geometry is the whole
//! question — *did the overflow affordance get drawn inside the tab bar,
//! or past its right edge?* — and that is precisely failure mode #8.
//!
//! The dock publishes the rects that make the twelve failure modes
//! checkable from outside, and leaves the pixel questions to
//! `tools/ui-verify`.
//!
//! # And why the names are structural rather than opaque
//!
//! Every name here is built from the *position* in the layout, not from
//! a generated id. A harness reading `dock.left.0.1.tabbar` knows which
//! compartment it is looking at without holding a map, and two runs of
//! the same layout produce the same names. That matters more than it
//! sounds: the RAG entry
//! `scripted_click_coordinates_go_stale_when_a_dock_width_changes` records
//! a whole class of false defect caused by harness coordinates that
//! outlived the layout they were measured in. Structural names let a
//! harness *re-read* rather than remember.
//!
//! Panel-scoped names carry the [`super::PanelId`] instead, because a
//! panel's identity survives being dragged to a different compartment and
//! a harness looking for "the Pages panel" should not have to know where
//! the operator put it.

use egui::Rect;

use super::model::{DockSide, PanelId};

/// **One drawn region, as the dock reports it.**
///
/// # ★★★ Why this is a struct, and why it carries a second rectangle
///
/// Until 2026-09-04 the dock's sink was `FnMut(&str, Rect)` — a name and
/// a rectangle — and the application on the other end of it published
/// every one of those through its unconditional rect channel. That is
/// **a report about LAYOUT, and it was being read as a report about
/// VISIBILITY.** The two are not the same claim, and on this project the
/// gap between them has a body count: `D:/dev/rag/egui/` records
/// Bookmarks, Layers and Signatures shipping *unreachable in a real
/// build with every gate green*, each of them with a rail entry and a
/// perfectly healthy rectangle in the trace.
///
/// A consumer that wants to make the stronger claim — *the operator can
/// see this* — needs to know what the region was **clipped to**, because
/// "laid out at these coordinates" and "at least three fifths of it
/// survived the clip" are answerable only together. So the dock now
/// hands over the clip rectangle in force at the moment the region was
/// published, and what the consumer does with it is the consumer's
/// business (see this module's [`Reporter::report`] and, on the
/// application side, `pdfcer_gui::diag::ui_rect_visible`).
///
/// # ★★ Why a struct rather than a third positional parameter
///
/// `FnMut(&str, Rect, Rect)` was the obvious widening and it is the
/// dangerous one: **two adjacent parameters of the same type, whose
/// meanings are not symmetric.** A consumer that swaps them compiles,
/// runs, and produces plausible numbers — `clip.intersect(rect)` is
/// commutative, so the *intersection* is unchanged, but the denominator
/// a visibility fraction divides by is not. A region 20 % inside a huge
/// clip and a region containing a tiny clip would then be told apart
/// only by which way round the caller happened to write them, and the
/// failure is silent in exactly the way this whole change exists to
/// stop.
///
/// Named fields cannot be swapped by accident. They are also the
/// extension point: a fourth thing to report (a z-order, an "is this
/// enabled") adds a field rather than a fourth breaking change to every
/// call site in three crates.
///
/// Not `Copy`, not stored: it borrows the freshly formatted name and
/// lives only for the duration of one sink call.
pub struct RectReport<'a> {
    /// The structural name — see this module's header for the scheme.
    pub name: &'a str,
    /// **Where the region was laid out**, in the drawing viewport's
    /// coordinates. This is exactly what the old `FnMut(&str, Rect)`
    /// sink was handed, so a consumer that wants the old behaviour reads
    /// this field and ignores the next one.
    pub rect: Rect,
    /// **What the region was clipped to** — the `egui` clip rectangle in
    /// force on the `Ui` that drew it.
    ///
    /// Not the containing compartment and not the window: the clip is
    /// what `egui` will actually let paint through, which is the only
    /// one of the three that answers *can this be seen*.
    ///
    /// For most of the dock's stream this equals or contains
    /// [`Self::rect`], because the dock's geometry is a pure subdivision
    /// of the side panel it was given ([`super::plan::resolve_spans`]
    /// never lets a child fall off the end of its container). The cases
    /// where it does **not** are the interesting ones, and they are the
    /// reason the field exists.
    pub clip: Rect,
}

/// # ★★★ Why the dock reports a clip and the ribbon does not
///
/// This is the judgement in this change, and it is deliberately
/// **asymmetric**. [`crate::ribbon::RectSink`] and
/// [`crate::menu`]'s (which is the ribbon's, shared) are still
/// `FnMut(&str, Rect)`. That is not an unfinished migration; the two
/// surfaces are asked different questions and want different answers.
///
/// **The dock's rects are compartments, never content.** Every name
/// this module publishes is a subdivision of the side the dock drew:
/// [`super::plan::resolve_spans`] degrades to an equal split rather
/// than letting a child overflow its parent, and a body rectangle is
/// clipped into its stack before anything is drawn into it. Nothing
/// here is sized by what a panel decided to put inside it. So the
/// question *how much of this survived the clip* is always a question
/// about whether **the dock** is on screen, and never — as it would be
/// for a `ScrollArea`'s content — a question about whether the panel's
/// contents are long. The claim a consumer wants from this stream is
/// **reachability**, *can the operator get to this*, and reachability
/// is precisely what a rectangle alone cannot state.
///
/// ★ That the dock itself can miss is not hypothetical, and it does not
/// need a broken layout. [`super::plan::MIN_SIDE_WIDTH`] is a hard
/// floor that wins over the window
/// ([`super::DockLayout::drawn_side_width`] clamps *up* to it) and
/// [`egui::Panel`] honours an `exact_size` wider than the space it was
/// given, so in a window narrower than that floor the side is drawn at
/// the floor and clipped. Measured at a 120 pt window, with a 160 pt
/// floor:
///
/// ```text
/// dock.left             rect=[0..160]    clip=[0..120]    0.750 visible
/// dock.left.split.side  rect=[154..160]  clip=[0..120]    0.000 visible
/// dock.left.collapse    rect=[138..154]  clip=[0..120]    0.000 visible
/// dock.body.p0          rect=[0..154]    clip=[0..120]    0.779 visible
/// ```
///
/// Everything the dock puts at the side's **trailing edge** — the
/// splitter that resizes it and the chevron that minimises it — is off
/// screen with a perfectly ordinary rectangle. `D:/dev/rag/egui/`
/// records this project shipping Bookmarks, Layers and Signatures
/// unreachable in a real build with every gate green, each with a rail
/// entry and a healthy rectangle; `SHELL_LAYOUT_PROPOSAL.md` §5 makes
/// closing that gap a precondition for the panel rail, on the ground
/// that no check could otherwise tell a working rail from that defect.
///
/// **The ribbon's rects are content.** A group's rectangle is what
/// its controls laid out to, a caption's is a galley, and a menu
/// item's belongs to an [`egui::Area`] floating above everything with
/// a clip of its own. Those are the shape the RAG entry
/// `a_visibility_gated_region_disappears_when_the_section_is_taller_than_its_slot`
/// warns about: gate a content-sized region on visibility and it
/// vanishes from the trace *exactly when it is interesting*. The
/// ribbon also already has one documented silent-drop mechanism —
/// `a_ribbon_group_that_collapses_at_the_default_window_width_makes_a_driven_check_skip_forever`
/// — and stacking a second on the same stream multiplies the ways a
/// check can stop running without turning red.
///
/// ⚠ **The failure mode of getting this wrong is a SKIP, and a SKIP is
/// not red.** A consumer that filters on visibility drops regions
/// silently by design; over-apply the filter and checks that were
/// asserting something become checks that assert nothing, with no
/// signal anywhere. So the rule this crate follows is: **widen a
/// surface's sink when a consumer needs to make a reachability claim
/// about compartments, and leave it alone where the consumer is
/// asking "did this draw" or "where do I scroll to" about content.**
/// If the ribbon ever needs the clip, [`RectReport`] is the
/// shape to copy — and the decision has to be re-made per region
/// name, not per crate.
///
/// # In one line
///
/// **No longer identical in shape to [`crate::ribbon::RectSink`].** An
/// application driving both surfaces now writes two closures, which is the
/// honest spelling: it was always free to treat the two streams differently,
/// and it now has to decide that it does.
pub type RectSink<'a> = dyn FnMut(&RectReport<'_>) + 'a;

/// The name prefix every rect this module publishes begins with.
pub const PREFIX: &str = "dock";

/// The whole of one dock side.
#[must_use]
pub fn side(side: DockSide) -> String {
    format!("{PREFIX}.{}", side.key())
}

/// One column within a side.
#[must_use]
pub fn column(side: DockSide, column: usize) -> String {
    format!("{PREFIX}.{}.{column}", side.key())
}

/// One stack — tab bar and body together.
#[must_use]
pub fn stack(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}", side.key())
}

/// One stack's tab bar.
///
/// Published separately from the stack because failure mode #8 is a
/// statement about what fits *inside the bar*, and asserting it against
/// the whole compartment's rect would pass trivially.
#[must_use]
pub fn tab_bar(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}.tabbar", side.key())
}

/// One stack's overflow affordance.
///
/// **This is the rect failure mode #8 is asserted against.** At a width
/// where tabs are hidden, this must exist, have a positive area, and lie
/// within the tab bar published by [`tab_bar`].
#[must_use]
pub fn overflow(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}.overflow", side.key())
}

/// One tab button, named by the panel it selects.
#[must_use]
pub fn tab(panel: &PanelId) -> String {
    format!("{PREFIX}.tab.{panel}")
}

/// One panel's body region — the rectangle the application drew into.
#[must_use]
pub fn body(panel: &PanelId) -> String {
    format!("{PREFIX}.body.{panel}")
}

/// A splitter between two columns of a side.
#[must_use]
pub fn column_splitter(side: DockSide, boundary: usize) -> String {
    format!("{PREFIX}.{}.split.col.{boundary}", side.key())
}

/// A splitter between two stacks of a column.
#[must_use]
pub fn stack_splitter(side: DockSide, column: usize, boundary: usize) -> String {
    format!("{PREFIX}.{}.{column}.split.row.{boundary}", side.key())
}

/// The **collapse control** on an open side — the little tab that minimises it.
#[must_use]
pub fn collapse(side: DockSide) -> String {
    format!("{PREFIX}.{}.collapse", side.key())
}

/// **The permanent strip above a side's columns** — see [`super::banner`].
///
/// Published only on a side that actually reserved one, so its **absence**
/// is the evidence that no banner was drawn. That asymmetry is deliberate: a
/// name published unconditionally, with a constant height, would go on
/// reporting a healthy rectangle for a strip the caller had stopped drawing
/// into — which is the shape of defect this whole stream was widened to
/// close.
#[must_use]
pub fn banner(side: DockSide) -> String {
    format!("{PREFIX}.{}.banner", side.key())
}

/// The rail a collapsed side leaves behind — the way back.
#[must_use]
pub fn rail(side: DockSide) -> String {
    format!("{PREFIX}.{}.rail", side.key())
}

/// ★ The **tool rail** — the permanent vertical strip down a side's outer
/// edge, carrying the panel tabs and the tool groups. `OPERATOR_REQUESTS.md`
/// O123 part 7.
///
/// Deliberately **not** [`rail`], which names a different surface: the sliver
/// a *collapsed* side leaves behind as the way back. Two surfaces sharing one
/// trace name is how a driven check reads the wrong one — recorded in
/// `D:/dev/rag/egui/two_trace_lines_sharing_an_event_name_make_a_check_read_the_wrong_one.md`.
///
/// Published only on a side that actually reserved one, so its **absence** is
/// the evidence that no rail was drawn — [`banner`]'s asymmetry, for
/// [`banner`]'s reason.
#[must_use]
pub fn tool_rail(side: DockSide) -> String {
    format!("{PREFIX}.{}.toolrail", side.key())
}

/// The splitter between a side and the central area.
#[must_use]
pub fn side_splitter(side: DockSide) -> String {
    format!("{PREFIX}.{}.split.side", side.key())
}

/// Holds the application's rect sink, if there is one.
///
/// A struct rather than a bare `Option` so the "do not format the name
/// unless someone is listening" rule lives in one place. Every call site
/// in the dock goes through it, and the rule matters here more than in
/// the ribbon: a dock draws a name per tab per stack per column per side
/// per frame, and `format!` on a hot path with nobody reading it is pure
/// waste.
pub struct Reporter<'a> {
    sink: Option<&'a mut RectSink<'a>>,
}

impl<'a> Reporter<'a> {
    /// Wrap a sink, or nothing.
    #[must_use]
    pub fn new(sink: Option<&'a mut RectSink<'a>>) -> Self {
        Self { sink }
    }

    /// Whether anyone is listening.
    ///
    /// Public so a caller can skip an expensive *measurement* — not only
    /// an allocation — when nothing will read it.
    #[must_use]
    pub fn is_listening(&self) -> bool {
        self.sink.is_some()
    }

    /// Publish a rect under a lazily-formatted name, together with the
    /// clip rectangle in force where it was drawn.
    ///
    /// # ★★ Why this takes the `Ui` rather than a `clip: Rect`
    ///
    /// The clip is not a parameter a call site should be *choosing*; it
    /// is a fact about the `Ui` the region was drawn into, and the only
    /// correct value is `ui.clip_rect()`. Passing the `Ui` makes that
    /// the only value obtainable, which closes two holes at once:
    ///
    /// 1. **The swap.** `report(rect, clip, name)` puts two `Rect`s side
    ///    by side with no type to tell them apart. Every one of this
    ///    module's dozen call sites would have been one transposition
    ///    away from a consumer computing a visibility fraction against
    ///    the wrong denominator — silently, with plausible output. See
    ///    [`RectReport`]'s note on the same hazard at the sink end.
    /// 2. **The stale clip.** A call site that captured a clip early and
    ///    reported late would report a rectangle from one `Ui` against a
    ///    clip from another. Asking the `Ui` at the moment of
    ///    publication cannot go stale.
    ///
    /// The cost is that this module now knows about [`egui::Ui`] and not
    /// only [`Rect`]. That is a widening of an `egui` dependency the
    /// crate already has from end to end, and it names nothing outside
    /// `egui` — R7 (`tools/gates/check-shell-purity.sh`) is about
    /// `pdfcer-*`, and a clip rectangle is as domain-free as the
    /// rectangle beside it.
    ///
    /// # Which `Ui` to pass
    ///
    /// **The one whose clip the region is actually subject to**, which is
    /// not always the one that drew the region's contents. A panel body
    /// is drawn in a child `Ui` clipped tighter still
    /// ([`super::Dock::show`]'s `draw_stack`), but the question a
    /// consumer asks of `dock.body.…` is *is this compartment on
    /// screen*, so the compartment's own `Ui` is the right one to ask.
    /// Reporting against the child's clip would compare the body
    /// rectangle to a clip derived from itself, which is the
    /// tautology `visible == 1.0` dressed up as a measurement.
    pub fn report(&mut self, ui: &egui::Ui, rect: Rect, name: impl FnOnce() -> String) {
        if let Some(sink) = self.sink.as_deref_mut() {
            sink(&RectReport {
                name: &name(),
                rect,
                clip: ui.clip_rect(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every published name begins with the prefix, so a harness can
    /// separate the dock's stream from the ribbon's.
    #[test]
    fn every_name_carries_the_prefix() {
        let names = [
            side(DockSide::Left),
            column(DockSide::Right, 0),
            stack(DockSide::Left, 1, 2),
            tab_bar(DockSide::Left, 0, 0),
            overflow(DockSide::Right, 0, 0),
            tab(&PanelId::new("pages")),
            body(&PanelId::new("pages")),
            column_splitter(DockSide::Left, 0),
            stack_splitter(DockSide::Left, 0, 1),
            side_splitter(DockSide::Right),
            banner(DockSide::Right),
        ];
        for name in &names {
            assert!(name.starts_with(PREFIX), "{name} is not prefixed");
        }
    }

    /// Names distinguish the two sides and the two axes of splitter, so a
    /// harness cannot mistake one for another.
    #[test]
    fn names_are_distinct_across_sides_and_axes() {
        assert_ne!(side(DockSide::Left), side(DockSide::Right));
        assert_ne!(
            column_splitter(DockSide::Left, 0),
            stack_splitter(DockSide::Left, 0, 0)
        );
        assert_ne!(stack(DockSide::Left, 0, 1), stack(DockSide::Left, 1, 0));
    }

    /// A panel-scoped name follows the panel rather than its position, so
    /// a harness that finds "the Pages panel" keeps finding it after the
    /// operator moves it.
    #[test]
    fn panel_names_do_not_encode_a_position() {
        let name = tab(&PanelId::new("pages"));
        assert_eq!(name, "dock.tab.pages");
        assert!(!name.contains("left"));
    }

    /// A reporter with no sink formats nothing.
    #[test]
    fn a_silent_reporter_does_not_format_names() {
        in_a_ui(|ui| {
            let mut reporter = Reporter::new(None);
            assert!(!reporter.is_listening());
            reporter.report(ui, Rect::ZERO, || panic!("the name was formatted"));
        });
    }

    /// ★★★ **The clip a region is reported against is the one in force on
    /// the `Ui` that drew it — not the region, and not the window.**
    ///
    /// This is the property the whole widening exists for, and it is
    /// worth a test of its own because the failure mode is invisible: a
    /// reporter that handed back `rect` as its own clip, or the screen
    /// rectangle as everybody's clip, would make every consumer's
    /// visibility fraction come out at exactly 1.0 and every check built
    /// on it green forever.
    #[test]
    fn a_report_carries_the_clip_in_force_not_the_region_itself() {
        let region = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 200.0));
        let clip = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(50.0, 50.0));
        let mut seen: Vec<(String, Rect, Rect)> = Vec::new();
        in_a_ui(|ui| {
            ui.set_clip_rect(clip);
            let mut sink = |r: &RectReport<'_>| seen.push((r.name.to_owned(), r.rect, r.clip));
            let mut reporter = Reporter::new(Some(&mut sink));
            reporter.report(ui, region, || "probe".to_owned());
        });
        assert_eq!(seen.len(), 1, "exactly one report");
        let (name, rect, reported_clip) = &seen[0];
        assert_eq!(name, "probe");
        assert_eq!(*rect, region, "the region is reported unchanged");
        assert_eq!(
            *reported_clip, clip,
            "the clip must be the Ui's, not the region and not the window"
        );
    }

    /// Run `f` against a real root `Ui`, because [`Reporter::report`]
    /// reads a clip rectangle and only a `Ui` has one.
    fn in_a_ui(f: impl FnOnce(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(f) = f.take() {
                f(ui);
            }
        });
    }
}
