//! The mode selector — an N-position segmented control on the right of
//! the tab-strip row.
//!
//! # What it is, from `MODES_AND_PANELS.md` Part 1
//!
//! ```text
//! [Open][Save][↶][↷]  File View Pages Edit Markup Measure Tools   ( Read ─ Review ─●─ Edit )  [⌃]
//! ```
//!
//! The positions are **ordered by capability** — each is a superset of
//! the one before — and the document is explicit about why that ordering
//! is chosen over three toggle buttons or a dropdown:
//!
//! > The ordering is the information, and it is what makes "slide left to
//! > calm the interface down" an obvious gesture rather than a learned
//! > one.
//!
//! And equally explicit about the trap that comes with saying "slider":
//!
//! > It must still render as a real segmented control with **all three
//! > labels visible** — not a bare track with a knob, where the available
//! > positions are invisible until you drag.
//!
//! That is the requirement this module is written against, and it is why
//! there is no `egui::Slider` anywhere in it. Every position is a labelled,
//! hit-testable, individually reported segment, and
//! `every_mode_gets_its_own_labelled_segment` asserts it.
//!
//! # Nothing here knows what "Read" means
//!
//! The control is generic over [`crate::manifest::Shell::modes`]. It
//! renders two positions or six with no change, and it contains no
//! literal `"read"`, `"review"` or `"edit"`. `SHELL_FRAMEWORK.md` §3:
//! *Read/Review/Edit is a **configuration**, not a built-in.* The
//! consequence worth stating is that a different application ships a
//! different set — *Draft · Proof · Press*, say — and gets the same
//! control with no code change at all.
//!
//! # Keyboard: a roving tab stop
//!
//! `MODES_AND_PANELS.md` Part 1, behavioural rule 6: *"the selector is a
//! real focusable control with arrow-key movement — not a mouse-only
//! affordance."*
//!
//! Implemented as the standard roving-tab-stop pattern, which `egui` 0.35
//! supports exactly and non-obviously:
//!
//! - `Sense::click()` is `CLICK | FOCUSABLE`; the bare `Sense::CLICK`
//!   flag is clickable and **not** focusable
//!   (`egui-0.35.0/src/sense.rs`).
//! - So only the **currently selected** segment is given
//!   `Sense::click()`. Tab therefore reaches the control once, not N
//!   times, which is what makes a six-mode selector bearable to traverse.
//! - Left/Right (and Home/End) move the selection while the control has
//!   focus, and focus is re-requested on the newly selected segment so
//!   the tab stop travels with it.
//!
//! The keys are taken with `consume_key`, so an arrow press that moved
//! the selector does not also scroll the panel underneath — the
//! double-action bug that makes a keyboard-driven interface feel
//! haunted.
//!
//! ## Movement clamps; it does not wrap
//!
//! [`move_index`] stops at the ends. A wrapping selector would turn one
//! Right press at *Edit* into *Read* — a jump from the most capable
//! stance to the least, in the control whose entire premise is that the
//! positions are ordered by capability. A slider does not wrap, and the
//! document's chosen metaphor is a slider.
//!
//! # R84 again
//!
//! The selected segment is distinguished by a **plate with a border**
//! and an **accent rule beneath it** — two shape cues — as well as by
//! its fill. See [`super::tabs`]'s header for why `RichText::strong()`
//! does not count as a non-colour cue in `egui`.

use egui::{Align2, Rect, Sense, Stroke, TextStyle, vec2};

use crate::manifest::Mode;

use super::a11y;
use super::ctx::Ctx;
use super::plan::MIN_ITEM_WIDTH;
use super::report;

/// Horizontal padding inside one segment, both sides together.
const SEGMENT_PADDING: f32 = 18.0;

/// The cues that distinguish the selected segment.
///
/// The same shape as [`super::tabs::TabCues`] and for the same reason:
/// R84 is a property of the *set* of cues, and a property of a set cannot
/// be asserted about expressions scattered through drawing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentCues {
    /// A border around the segment's plate. A **shape** cue.
    pub outlined: bool,
    /// An accent rule beneath the segment. A second **shape** cue.
    pub underline: bool,
    /// The plate fill. A **colour** cue.
    pub filled: bool,
}

impl SegmentCues {
    /// How many cues survive greyscale.
    pub fn non_colour_cues(self) -> usize {
        usize::from(self.outlined) + usize::from(self.underline)
    }
}

/// The cues for a segment in a given state.
pub fn segment_cues(selected: bool) -> SegmentCues {
    SegmentCues {
        outlined: selected,
        underline: selected,
        filled: selected,
    }
}

/// A keyboard movement request within the selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// One position toward the first.
    Prev,
    /// One position toward the last.
    Next,
    /// The first position.
    First,
    /// The last position.
    Last,
}

/// Apply a keyboard movement, clamped at both ends.
///
/// See the module header on why this clamps rather than wraps. `len == 0`
/// is answered with `0` rather than a panic: an empty selector is not
/// drawn at all, so the value is never used, and a `panic!` in the paint
/// loop for an empty manifest would be a poor trade.
pub fn move_index(current: usize, movement: Move, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    // `current` is clamped too, not only the result. A caller cannot
    // normally hand in an out-of-range index — `selected_index` never
    // produces one — but "cannot normally" is not an argument for a
    // function running in the paint loop against a manifest an operator
    // edited, and `saturating_sub` alone would answer `1` for a
    // one-position selector.
    let current = current.min(last);
    match movement {
        Move::Prev => current.saturating_sub(1),
        Move::Next => (current + 1).min(last),
        Move::First => 0,
        Move::Last => last,
    }
}

/// The label one mode shows. Never empty — a nameless position in a
/// segmented control is indistinguishable from a gap in the track.
pub(crate) fn mode_label(mode: &Mode) -> &str {
    match mode.label.as_deref() {
        Some(l) if !l.trim().is_empty() => l,
        _ if !mode.id.is_empty() => &mode.id,
        _ => "(unnamed mode)",
    }
}

/// The index of the selected mode, defaulting to the first.
///
/// An unknown id resolves to the first position rather than to "nothing
/// selected", because a segmented control with no segment selected shows
/// the operator a state that is not one of the states the control offers.
pub(crate) fn selected_index(modes: &[Mode], selected: Option<&str>) -> usize {
    selected
        .and_then(|id| modes.iter().position(|m| m.id == id))
        .unwrap_or(0)
}

/// The width of each segment, and the total.
///
/// Every segment is the width of the widest label, so the control reads
/// as evenly divided track rather than as a row of differently sized
/// buttons — which is what makes it look like one control with positions
/// rather than like three buttons that happen to be adjacent.
pub(crate) fn segment_widths(labels: &[&str], measure: impl Fn(&str) -> f32) -> (f32, f32) {
    let widest = labels
        .iter()
        .map(|l| measure(l))
        .fold(0.0_f32, f32::max)
        .max(MIN_ITEM_WIDTH);
    let each = widest + SEGMENT_PADDING;
    (each, each * labels.len() as f32)
}

/// Measure the track the way [`render`] will, so the tab-strip row can
/// reserve it before anything is drawn.
///
/// Returns `(segment_width, natural_total_width)` — the *uncompressed*
/// numbers. [`fit_track`] applies the compression, and it does so inside
/// [`render`] against the room the row actually granted.
///
/// This exists because the reservation and the rendering must agree.
/// [`super::plan::plan_strip_row`] subtracts this figure from the row
/// before the tabs are planned; if the selector then measured itself
/// differently it would either overhang the tabs or leave a gap, and the
/// tab plan would be wrong by the difference.
pub(crate) fn measure_track(ui: &egui::Ui, modes: &[Mode]) -> (f32, f32) {
    if modes.is_empty() {
        return (0.0, 0.0);
    }
    let labels: Vec<&str> = modes.iter().map(mode_label).collect();
    let font = TextStyle::Button.resolve(ui.style());
    segment_widths(&labels, |s| {
        ui.ctx().fonts_mut(|f| {
            f.layout_no_wrap(s.to_owned(), font.clone(), egui::Color32::PLACEHOLDER)
                .size()
                .x
        })
    })
}

/// Fit the track into the room the row actually has.
///
/// # ★ Why this exists — the same failure mode as the overflow affordance
///
/// [`super`]'s module header states the rule this enforces: *"two things
/// on this ribbon must never be squeezed out by content: the mode selector
/// and the overflow affordance."* Laying the selector out first, from the
/// right edge, achieves that against **content**. It does nothing about
/// the case where the selector alone is wider than the row.
///
/// `egui` answers `allocate_exact_size` on a right-to-left layout by
/// extending **leftwards past the edge of the container**. So a track that
/// does not fit is not clipped, not shrunk and not warned about: it is
/// placed with its left portion off screen. At a 180 pt viewport with real
/// font metrics, a three-position *Read · Review · Edit* selector measures
/// 189 pt and the first position lands at x = −9 — present in the layout,
/// unreachable with a mouse, and invisible in every test that measured
/// text as zero-width.
///
/// So the shortfall is spent on the **segments' width** instead of on
/// their position: every position stays on screen and stays clickable,
/// and the labels crowd. That is the same trade the overflow affordance
/// makes (see [`super::band`]) and it is made for the same reason — a
/// control the operator cannot reach has failed completely, whereas a
/// control whose label is tight has failed cosmetically.
///
/// `room` that is not finite or not positive means "no constraint known";
/// the natural size is returned unchanged, because clamping to a bogus
/// number would shrink a control that had plenty of space.
///
/// Returns `(segment_width, total_width)`, and the caller discloses a
/// shrink through the verification channel — see [`render`].
pub(crate) fn fit_track(each: f32, positions: usize, room: f32) -> (f32, f32) {
    let total = each * positions as f32;
    if positions == 0 || !room.is_finite() || room <= 0.0 || total <= room {
        return (each, total);
    }
    let each = room / positions as f32;
    (each, each * positions as f32)
}

/// Draw the selector and report the mode the operator chose, if it
/// changed.
///
/// Returns `None` when nothing changed, or when there are no modes at all
/// — an application with no modes gets no selector, rather than a control
/// with one position that does nothing.
pub(crate) fn render(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    modes: &[Mode],
    selected: Option<&str>,
) -> Option<String> {
    if modes.is_empty() {
        return None;
    }

    let labels: Vec<&str> = modes.iter().map(mode_label).collect();
    let font = TextStyle::Button.resolve(ui.style());
    let (segment_w, natural_total_w) = measure_track(ui, modes);

    // ★ Fit the track to the row before allocating it. See `fit_track`:
    // an over-wide track is not clipped by `egui`, it is placed off the
    // left edge of the row, and a control that is off screen is a control
    // that is not there.
    let room = ui.available_width();
    let (segment_w, total_w) = fit_track(segment_w, modes.len(), room);
    if total_w < natural_total_w {
        crate::verify::event("ribbon-mode-selector-compressed")
            .kv("natural", format!("{natural_total_w:.1}"))
            .kv("room", format!("{room:.1}"))
            .kv("positions", modes.len().to_string())
            .emit();
    }

    let height = ctx.theme.metrics.control_height;
    let (track, _) = ui.allocate_exact_size(vec2(total_w, height), Sense::hover());
    ctx.reporter.report_static(track, report::mode_selector());

    let ids: Vec<egui::Id> = modes.iter().map(|m| ctx.id("mode", &m.id)).collect();
    let mut index = selected_index(modes, selected);

    // ---------------------------------------------------------------
    // Keyboard, BEFORE the segments are drawn.
    //
    // The sense a segment is given depends on whether it is selected, so
    // the selection has to be settled first — otherwise an arrow press
    // would move the selection onto a segment that had already been
    // registered as non-focusable, and the tab stop would be lost for a
    // frame.
    // ---------------------------------------------------------------
    let focused_here = ui
        .memory(egui::Memory::focused)
        .is_some_and(|f| ids.contains(&f));
    let mut moved = false;
    if focused_here {
        for (key, movement) in [
            (egui::Key::ArrowLeft, Move::Prev),
            (egui::Key::ArrowRight, Move::Next),
            (egui::Key::Home, Move::First),
            (egui::Key::End, Move::Last),
        ] {
            // `consume_key` rather than `key_pressed`: an arrow that moved
            // the selector must not also scroll whatever is behind it.
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                let next = move_index(index, movement, modes.len());
                if next != index {
                    index = next;
                    moved = true;
                }
            }
        }
    }

    let mut chosen: Option<String> = None;
    let painter = ui.painter().clone();

    for (i, mode) in modes.iter().enumerate() {
        let rect = Rect::from_min_size(
            egui::pos2(track.left() + segment_w * i as f32, track.top()),
            vec2(segment_w, height),
        );
        let is_selected = i == index;
        let cues = segment_cues(is_selected);

        // ★ The roving tab stop. Only the selected segment is focusable;
        // the rest are clickable and skipped by Tab. See the module
        // header — `Sense::CLICK` without `FOCUSABLE` is the whole
        // mechanism and it is not obvious from `egui`'s documentation.
        let sense = if is_selected {
            Sense::click()
        } else {
            Sense::CLICK
        };
        let response = ui.interact(rect, ids[i], sense);

        let visuals = ui.style().interact(&response);
        if cues.filled {
            painter.rect_filled(
                rect.shrink(1.0),
                ctx.theme.metrics.corner_radius,
                ctx.theme.palette.accent,
            );
        } else if response.hovered() {
            painter.rect_filled(
                rect.shrink(1.0),
                ctx.theme.metrics.corner_radius,
                ctx.theme.palette.panel,
            );
        }
        if cues.outlined {
            painter.rect_stroke(
                rect.shrink(1.0),
                ctx.theme.metrics.corner_radius,
                Stroke::new(1.0, ctx.theme.palette.outline),
                egui::StrokeKind::Inside,
            );
        }
        if cues.underline {
            let y = rect.bottom() - 1.0;
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 4.0, y),
                    egui::pos2(rect.right() - 4.0, y),
                ],
                Stroke::new(2.0, ctx.theme.palette.accent),
            );
        }
        if response.has_focus() {
            // A focus ring, so keyboard operation is visible. Drawn
            // outside the plate so it is never confused with the
            // selection border.
            //
            // ★★★ `palette.accent`, spelled out — it was
            // `ui.visuals().selection.stroke` until 2026-09-04, and that read
            // would have silently inverted this ring.
            //
            // `visuals.selection` is `egui`'s SELECTED-WIDGET channel: the
            // fill a selected control is painted with and the ink that reads
            // ON that fill. Defect T2 (`REVIEW_TRIAGE.md` §2b) re-pointed it at
            // the pair it is named for — `accent` and `on_accent` — because
            // this theme had been handing it to the canvas, which made every
            // bare `selectable_label(true, …)` in the application unreadable.
            //
            // The moment it carries `on_accent`, reading it here is defect D2's
            // exact shape a fourth time: `on_accent` is a *plate* colour,
            // near-white under the light presets, and this ring is drawn
            // OUTSIDE the plate, on the ribbon's own background. Near-white on
            // near-white — luminance gap 5 under Airy. The ring's real role is
            // "the accent, on chrome", which is `palette.accent` and always was;
            // the old spelling merely reached it by an address that has now
            // moved. Same colour, correct name.
            painter.rect_stroke(
                rect.expand(1.0),
                ctx.theme.metrics.corner_radius,
                Stroke::new(1.0, ctx.theme.palette.accent),
                egui::StrokeKind::Outside,
            );
        }

        let text_colour = if cues.filled {
            ctx.theme.palette.on_accent
        } else {
            visuals.text_color()
        };
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            labels[i],
            font.clone(),
            text_colour,
        );

        a11y::describe_mode_segment(&response, labels[i], is_selected);
        ctx.reporter.report(rect, || report::mode_segment(&mode.id));

        if response.clicked() {
            chosen = Some(mode.id.clone());
        }
    }

    if moved {
        // Carry the tab stop to the newly selected segment, which was
        // registered as focusable a few lines above.
        ui.memory_mut(|m| m.request_focus(ids[index]));
        chosen = Some(modes[index].id.clone());
    }

    if let Some(id) = &chosen {
        crate::verify::event("ribbon-mode-selected")
            .kv("mode", id)
            .emit();
    }
    chosen.filter(|id| Some(id.as_str()) != selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modes() -> Vec<Mode> {
        vec![
            Mode::new("read", "Read", ["file", "view"]),
            Mode::new("review", "Review", ["file", "view", "markup"]),
            Mode::new("edit", "Edit", ["file", "view", "edit"]),
        ]
    }

    /// **★ Arrow-key movement clamps at both ends rather than wrapping.**
    ///
    /// The positions are ordered by capability. A wrap would turn one
    /// Right press at the most capable stance into the least capable one
    /// — in the control whose entire premise, per
    /// `MODES_AND_PANELS.md` Part 1, is that *"the ordering is the
    /// information"*. A slider does not wrap, and a slider is the
    /// document's chosen metaphor.
    #[test]
    fn arrow_movement_clamps_rather_than_wrapping() {
        assert_eq!(move_index(0, Move::Prev, 3), 0, "already at the first");
        assert_eq!(move_index(2, Move::Next, 3), 2, "already at the last");
        assert_eq!(move_index(1, Move::Prev, 3), 0);
        assert_eq!(move_index(1, Move::Next, 3), 2);
        assert_eq!(move_index(2, Move::First, 3), 0);
        assert_eq!(move_index(0, Move::Last, 3), 2);
    }

    /// Movement is total: no length, no index and no key produces a panic
    /// or an out-of-range answer.
    ///
    /// This runs in the paint loop against a manifest an operator may
    /// have edited, so "cannot happen" is not a sufficient argument.
    #[test]
    fn movement_is_total_over_every_length_and_index() {
        for len in 0..6_usize {
            for current in 0..8_usize {
                for movement in [Move::Prev, Move::Next, Move::First, Move::Last] {
                    let next = move_index(current, movement, len);
                    if len > 0 {
                        assert!(
                            next < len,
                            "len={len} current={current} {movement:?} → {next}"
                        );
                    } else {
                        assert_eq!(next, 0);
                    }
                }
            }
        }
    }

    /// **★ Every mode gets its own labelled segment — no bare track.**
    ///
    /// `MODES_AND_PANELS.md` Part 1 forbids *"a bare track with a knob,
    /// where the available positions are invisible until you drag."* The
    /// checkable form of that is: N modes produce N segments, each with a
    /// non-empty label and a positive width, and the total is exactly the
    /// sum. A knob-and-track implementation fails on the segment count.
    #[test]
    fn every_mode_gets_its_own_labelled_segment() {
        let modes = modes();
        let labels: Vec<&str> = modes.iter().map(mode_label).collect();
        assert_eq!(labels, ["Read", "Review", "Edit"]);
        for l in &labels {
            assert!(!l.trim().is_empty());
        }

        let (each, total) = segment_widths(&labels, |s| s.chars().count() as f32 * 7.0);
        assert!(each > 0.0);
        assert_eq!(
            total,
            each * 3.0,
            "the track is exactly its segments; there is no spare knob space"
        );
    }

    /// The control is generic over the manifest: two positions, or six,
    /// or a completely different vocabulary.
    ///
    /// The point of the test is the *absence* of Read/Review/Edit from
    /// this crate. `SHELL_FRAMEWORK.md` §3 requires it, and a shell that
    /// hard-coded three stances would be un-reusable in exactly the way
    /// the whole design exists to avoid.
    #[test]
    fn the_selector_is_generic_over_the_manifests_modes() {
        let press = [
            Mode::new("draft", "Draft", ["a"]),
            Mode::new("proof", "Proof", ["a"]),
            Mode::new("press", "Press", ["a"]),
            Mode::new("archive", "Archive", ["a"]),
        ];
        let labels: Vec<&str> = press.iter().map(mode_label).collect();
        let (each, total) = segment_widths(&labels, |s| s.chars().count() as f32 * 7.0);
        assert_eq!(total, each * 4.0, "four positions, four segments");
        assert_eq!(selected_index(&press, Some("press")), 2);
    }

    /// A segment label is never empty, whatever the manifest says.
    ///
    /// An unlabelled position in a segmented control is indistinguishable
    /// from a gap in the track, which is the exact failure the "all
    /// labels visible" rule names.
    #[test]
    fn a_segment_label_is_never_empty() {
        assert_eq!(mode_label(&Mode::patch("review")), "review");
        let blank = Mode {
            id: "x".to_owned(),
            label: Some("  ".to_owned()),
            tabs: None,
        };
        assert_eq!(mode_label(&blank), "x");
        let nameless = Mode {
            id: String::new(),
            label: None,
            tabs: None,
        };
        assert_eq!(mode_label(&nameless), "(unnamed mode)");
    }

    /// An unknown selection resolves to the first position, not to
    /// "nothing selected".
    ///
    /// A segmented control with no segment selected shows a state that is
    /// not one of the states the control offers, and the operator has no
    /// way to find out which one they are actually in.
    #[test]
    fn an_unknown_selection_falls_back_to_the_first_position() {
        let modes = modes();
        assert_eq!(selected_index(&modes, Some("review")), 1);
        assert_eq!(selected_index(&modes, Some("no-such-mode")), 0);
        assert_eq!(selected_index(&modes, None), 0);
    }

    /// **★ A track that does not fit is compressed, never pushed off the
    /// edge.**
    ///
    /// `MODES_AND_PANELS.md` Part 1 requires every position to be visible
    /// and operable; [`super`]'s header adds that the selector is one of
    /// the two controls that must never be squeezed out. `egui` answers an
    /// over-wide `allocate_exact_size` in a right-to-left layout by
    /// extending past the container's left edge, so without this clamp the
    /// first position lands at a negative x — drawn, reported and
    /// unclickable.
    ///
    /// With no font data every label measures zero, the track is always
    /// 3 × `MIN_ITEM_WIDTH + SEGMENT_PADDING`, and no realistic row is
    /// ever narrower than that — which is why this needed a *pure* test
    /// rather than only a rendered one.
    #[test]
    fn a_track_that_does_not_fit_is_compressed_rather_than_pushed_off_screen() {
        // Fits: nothing changes, and in particular the control does not
        // stretch to fill the row.
        assert_eq!(fit_track(60.0, 3, 400.0), (60.0, 180.0));
        assert_eq!(fit_track(60.0, 3, 180.0), (60.0, 180.0), "exactly fitting");

        // Does not fit: the total becomes the room, so the track's left
        // edge is the row's left edge rather than a negative coordinate.
        let (each, total) = fit_track(63.0, 3, 180.0);
        assert_eq!(total, 180.0, "the track is never wider than the row");
        assert_eq!(
            each, 60.0,
            "the shortfall is shared equally by the positions"
        );
        assert!(each > 0.0, "every position keeps a clickable width");

        // Unknown or absurd room is not a constraint: an unbounded
        // container reports an infinite width, and clamping to it would
        // produce a NaN track.
        assert_eq!(fit_track(60.0, 3, f32::INFINITY), (60.0, 180.0));
        assert_eq!(fit_track(60.0, 3, f32::NAN), (60.0, 180.0));
        assert_eq!(fit_track(60.0, 3, -5.0), (60.0, 180.0));
        assert_eq!(
            fit_track(60.0, 0, 10.0),
            (60.0, 0.0),
            "no positions, no track"
        );

        // Total compression is still total coverage: whatever the room,
        // the positions tile it exactly, with no gap and no overhang.
        for room in (1..400).map(|r| r as f32) {
            let (each, total) = fit_track(63.0, 3, room);
            assert!(
                (each * 3.0 - total).abs() < 0.001,
                "room={room}: the segments must tile the track exactly"
            );
            assert!(total <= room.max(189.0) + 0.001, "room={room}");
        }
    }

    /// R84 for the selector: the selected segment carries two shape cues
    /// as well as its fill.
    #[test]
    fn the_selected_segment_is_distinguished_by_more_than_colour() {
        let selected = segment_cues(true);
        assert!(
            selected.non_colour_cues() >= 2,
            "R84: the selected position carries only {} cue(s) that survive greyscale",
            selected.non_colour_cues()
        );
        assert_eq!(segment_cues(false).non_colour_cues(), 0);
        assert_ne!(selected, segment_cues(false));
    }
}
